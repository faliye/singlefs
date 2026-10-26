//! 理想模型（里程碑「覆盖写、释放、回退与复用」增补 3 第 2 件）：一个只住内存的模型，记每一版提交的内容、每条根属于哪个实例、实例表的行、回退下界 F，
//! 回答「冷启动该读回哪一版」「回退到这条根该不该被拒」「这个错误该不该出现」；第 1 件的每一步拿实现的结局与它比（比的那层胶水在
//! `model_comparison.rs`，那里可以用 `singlefs_core` 的类型）。
//!
//! 只 `use singlefs_format`，不 `use singlefs_core` / `singlefs_checker`（D13（验证路线） 已定项 5：只共享一份从 kb 生成的常量）：代号、实例、
//! 角色、根环落点、F 的生效值、抬 F 的上限、候选集、内容装不装得下都照条款另写一份，不照抄实现的算法。
//!
//! 不建崩溃：第 1 件的历史里没有断电、没有设备错，每一步的写都落完，所以环里最新的那条根就是最后写出的那一条、journal 里没有要施加的记录
//! （可写挂载写的上一个实例那一行 W 恒 0）。第 3 件（崩溃注入）判「这个崩溃状态恢复到的版本在不在允许集合里」用 [`crash_recovery_disagreement`]。
//!
//! 条款把答案留给实现取上界的地方（容量墙），模型答「允许拒绝的区间」，不照抄实现的上界算法；打回重议那几处（增补 2 收口表第 ①、② 行）
//! 照代码今天的读法写，每一处标「预想，跟收口表第 X 行」。
//!
//! 模型不记落点：单元落在哪个槽、分配记录写得对不对（回收门槛、释放时改没改写记录、复用时罩住的槽删没删）归池级 checker 判，
//! 模型只拿实现交回的每个单元那几条记录比「每块盘一条、仍分配、分配代等于写它的那次发布」（增补 3 第 2 件代码三方第一轮判决第一节 M4 那一格：
//! 回收门槛差一、释放时不改写记录这几条变异，只留模型时三段都判不出，checker 都判红）。分配记录的真条数模型同样不记；分配记录树按绝对槽号
//! 按位置寻址之后（D8（核心索引结构） 已定项 14）没有「一个节点装不下」那道墙，模型也就没有那条拒绝理由。

use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU64;
use std::rc::Rc;

use singlefs_format::{
    ACCOUNTING_ENTRY_BYTES, ACCOUNTING_KEY_BYTES, ALLOCATION_RECORD_TREE_INTERNAL_FANOUT,
    ALLOCATION_RECORD_TREE_LEAF_SLOTS, CLUSTER_SEGMENT_SLOTS, DATA_UNIT_BYTES,
    DATA_UNIT_PAYLOAD_OFFSET, EXTENT_TREE_INTERNAL_FANOUT, EXTENT_TREE_LOWER_LEAF_DATA_UNITS,
    EXTENT_TREE_UPPER_LEAF_INODES, INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE,
    INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT, INODE_INTERNAL_ENTRY, INODE_LEAF_RECORDS,
    INSTANCE_TABLE_PAGE_RECORDS, JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES,
    JOURNAL_RING_START_SLOT, LINEAGE_CONTAINER_RECORDS, MAPPING_ENTRY_BYTES, MAPPING_KEY_BYTES,
    NODE_BYTES, NODE_POINTER_BYTES, NONCE_MAC_ALGORITHM_RESERVED_BYTES, ROOT_RING_REGIONS,
    ROOT_RING_REGION_DEVICES, ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM, SLOT_BYTES,
    TREE_IDENTIFIER_EXTENT, TREE_IDENTIFIER_INODE,
};

/// 模型里的 checkpoint 号（D16（发布语义） 已定项 6：每次发布 + 1）。与实现的 `CheckpointTxg` 各自声明（D13（验证路线） 已定项 5）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModelCheckpointTxg(pub u64);

/// 模型里的实例代号（D23（journal 的角色与格式） 已定项 16：取号 = 见过的最大 + 1）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModelInstanceGeneration(pub u64);

/// 模型里的 jsn 计数器（D23（journal 的角色与格式） 已定项 14 第 3 条：全池接着走、换实例不归零）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModelJournalCounter(pub u64);

/// 模型里的盘。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModelDeviceIdentity(pub u32);

/// 一条根的身份。择根按 txg 为主、实例代号破平局（D22（单元原子性怎么合成） 已定项 7）：字段次序就是比较次序。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModelRootKey {
    pub checkpoint_txg: ModelCheckpointTxg,
    pub instance: ModelInstanceGeneration,
}

/// mkfs 的实例代号与第 0 代根的 txg（D23（journal 的角色与格式） 已定项 16：mkfs 写 0，第一次取号取 1）。
const MAKE_FILESYSTEM_INSTANCE: ModelInstanceGeneration = ModelInstanceGeneration(0);
const MAKE_FILESYSTEM_TXG: ModelCheckpointTxg = ModelCheckpointTxg(0);

/// 记账树里不带设备维的行与每块盘各一组的行（D5（快照 / 空间记账机制） 已定项 8：inode 号水位、待删占用、已承诺预留；每盘已分配、空闲、
/// 不可回收、defer 待释放、碎片段数、全空聚簇段数）。
const POOL_WIDE_ACCOUNTING_ROWS: u64 = 3;
const ACCOUNTING_ROWS_PER_DEVICE: u64 = 6;

/// inode 树里最左那片之外的一片叶容器：它在树里左起第几片（从 0 数，最左那片是 0，这里恒不小于 1）。
/// 只经 [`ModelUnitRole::inode_leaf_container_at`] 造，第 0 片造不出这个类型（那一片是 [`ModelUnitRole::InodeLeaf`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModelInodeLeafContainerRightOfTheLeftmost {
    position_from_the_leftmost: NonZeroU64,
}

impl ModelInodeLeafContainerRightOfTheLeftmost {
    /// 它在树里左起第几片（从 0 数，恒不小于 1）。
    #[must_use]
    pub fn position_from_the_leftmost(self) -> u64 {
        self.position_from_the_leftmost.get()
    }
}

/// 一版里的一个角色：第一个文件版本的八个单元（D3（空间分配） 已定项 10 ⑤ 的次序）与实例表单元（D18（块里携带什么信息） 已定项 11）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModelUnitRole {
    /// 文件的数据单元：一个文件有几个就有几个，各自的分配代按单元号记在 [`ModelRoot::data_unit_written_at`]，不进
    /// [`ModelRoot::role_written_at`]（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K4：截断只重写尾单元之后，
    /// 一个文件的数据单元不再同一个出生 txg）。
    Data,
    ExtentRoot,
    /// extent 树上段根之下的一个节点（上段长过层之后才有），按它在上段里的位置各是一个角色：上段的形状只由有条目的 inode 号集合定
    /// （判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M2，规范形），一次发布重写哪几个、照抄哪几个按
    /// [`extent_upper_node_written_at_after`] 的计划，各节点的分配代模型逐个记、逐个比。
    ExtentUpperNodeBelowTheRoot(ModelExtentUpperNodePosition),
    /// inode 树最左那片叶容器：第一个文件那条记录住它，删完之后建的 inode 前 233 条也住它。
    InodeLeaf,
    /// inode 树最左那片之外的每一片叶容器各是一个角色（删完之后建满 233 条起才有，D8（核心索引结构） 已定项 6：插入落在最右那片、
    /// 满了在末尾分裂、左半不动）：各片的分配代不同（只有新记录落的那一片这次重写），模型逐片记、逐片比。
    InodeLeafRightOfTheLeftmost(ModelInodeLeafContainerRightOfTheLeftmost),
    InodeRoot,
    AllocationTree,
    AccountingTree,
    MappingTree,
    TreeTable,
    InstanceTable,
    /// `deleted_inodes` 的根（D8（核心索引结构） 已定项 16 第 1 条）：队列里有号的那几版才有。
    DeletedInodesRoot,
    /// logged_ops 树的根（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1：单树、一条整理意图一个条目，模型罩的历史里
    /// 至多一条、根兼叶一个节点）：签发那次发布写、之后照抄（分配代 = 签发那次的 txg）、删意图那次根指针置零，有活意图的那几版才有。
    LoggedOperationsRoot,
    /// 墓碑树的开放容器（D18（块里携带什么信息） 已定项 24 第 1 条）：模型罩的历史里墓碑至多几十条，恒只有容器 1 那一片。
    TombstoneLeaf,
    /// 墓碑树的根。
    TombstoneRoot,
    /// 退役树的开放容器（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 5 条，照墓碑树）：模型罩的历史里只有删第一个文件
    /// 那次最后一批让唯一那片叶容器身份消失、追加一条退役记录，退役树恒只有容器 1 那一片。随现行那一版：管理员回退不取回退目标的。
    RetirementLeaf,
    /// 退役树的根。
    RetirementRoot,
}

impl ModelUnitRole {
    /// inode 树左起第 `position_from_the_leftmost` 片叶容器（从 0 数）的角色：第 0 片是 [`ModelUnitRole::InodeLeaf`]，
    /// 之后每一片是 [`ModelUnitRole::InodeLeafRightOfTheLeftmost`]。
    #[must_use]
    pub fn inode_leaf_container_at(position_from_the_leftmost: u64) -> ModelUnitRole {
        match NonZeroU64::new(position_from_the_leftmost) {
            None => ModelUnitRole::InodeLeaf,
            Some(position_from_the_leftmost) => ModelUnitRole::InodeLeafRightOfTheLeftmost(
                ModelInodeLeafContainerRightOfTheLeftmost {
                    position_from_the_leftmost,
                },
            ),
        }
    }

    /// extent 树上段的一个节点（根或根之下的）：这几个角色的分配代按规范形计划整组定（[`extent_upper_node_written_at_after`]）。
    fn is_a_node_of_the_extent_upper_segment(self) -> bool {
        match self {
            ModelUnitRole::ExtentRoot | ModelUnitRole::ExtentUpperNodeBelowTheRoot(_) => true,
            ModelUnitRole::Data
            | ModelUnitRole::InodeLeaf
            | ModelUnitRole::InodeLeafRightOfTheLeftmost(_)
            | ModelUnitRole::InodeRoot
            | ModelUnitRole::AllocationTree
            | ModelUnitRole::AccountingTree
            | ModelUnitRole::MappingTree
            | ModelUnitRole::TreeTable
            | ModelUnitRole::InstanceTable
            | ModelUnitRole::DeletedInodesRoot
            | ModelUnitRole::LoggedOperationsRoot
            | ModelUnitRole::TombstoneLeaf
            | ModelUnitRole::TombstoneRoot
            | ModelUnitRole::RetirementLeaf
            | ModelUnitRole::RetirementRoot => false,
        }
    }

    /// 占几个槽：数据单元与打包记录单元（inode 叶、实例表）恒 32 KiB（D4（校验和位置） 已定项 5），索引节点恒 16 KiB
    /// （D8（核心索引结构） 已定项 2），落点粒度 16 KiB（D3（空间分配） 已定项 7）。
    #[must_use]
    pub fn span_in_slots(self) -> u64 {
        match self {
            ModelUnitRole::Data
            | ModelUnitRole::InodeLeaf
            | ModelUnitRole::InodeLeafRightOfTheLeftmost(_)
            | ModelUnitRole::InstanceTable
            | ModelUnitRole::TombstoneLeaf
            | ModelUnitRole::RetirementLeaf => DATA_UNIT_BYTES / SLOT_BYTES,
            ModelUnitRole::ExtentRoot
            | ModelUnitRole::ExtentUpperNodeBelowTheRoot(_)
            | ModelUnitRole::InodeRoot
            | ModelUnitRole::AllocationTree
            | ModelUnitRole::AccountingTree
            | ModelUnitRole::MappingTree
            | ModelUnitRole::TreeTable
            | ModelUnitRole::DeletedInodesRoot
            | ModelUnitRole::LoggedOperationsRoot
            | ModelUnitRole::TombstoneRoot
            | ModelUnitRole::RetirementRoot => NODE_BYTES / SLOT_BYTES,
        }
    }
}

/// 一次发布重写哪些角色。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ModelPublishKind {
    /// 第一个文件版本：四个文件角色，加固定点单元——四个固定点角色（分配记录树、记账树、映射树、树表），分配记录树按位置寻址
    /// 一次重写根与根之下要改的那几个节点（D8（核心索引结构） 已定项 14；新池几何下 5 个，固定点合计 8 个单元，字节表八），
    /// D16（发布语义） 已定项 9：记账行每发布重写，连带记账树节点、分配记录、映射条目与树表单元。与覆盖写重写的角色相同，分开只为一件事——它走 `previous: None`，这一版的内存态从头建。
    FirstFileVersion,
    /// 整份重写文件内容：覆盖写（一个数据单元以内）与顺序写（跨几个数据单元，`transaction::publish_sequential_write`）。
    /// 与第一个文件版本重写的角色相同，接在上一版的内存态后面；内容的每个数据单元都在这次写（同一个分配代），
    /// 多于一个单元时 extent 树下段的节点（叶罩 144 个单元、内部扇出 147，D8（核心索引结构） 已定项 14）也在这次写，
    /// 一个单元一个事务（D16（发布语义） 已定项 5 的切分纪律）。
    OverwriteFileVersion,
    /// 写行：实例表单元（D18（块里携带什么信息） 已定项 11：每次可写挂载都写行）加固定点单元（同第一个文件版本那几族，D16（发布语义） 已定项 9）。
    RowsOnFileVersion,
    /// 树表不是 0 条时的空发布（暖机、抬 F）：固定点单元（同第一个文件版本那几族，D16（发布语义） 已定项 9）。
    EmptyOnFileVersion,
    /// 树表 0 条的一版上写行：只重写实例表这一个单元（D18（块里携带什么信息） 已定项 11 要求每次可写挂载都写行；
    /// 这一版没有记账树，D16（发布语义） 已定项 9 的那五样——记账行、记账树节点、分配记录、映射条目、树表单元——一样都不写）。
    RowsOnVersionWithoutFile,
    /// 树表 0 条上的空发布：零单元（D16（发布语义） 已定项 9「树表 0 条 ⇒ 零单元」）。
    ZeroUnit,
    /// unlink 的意图那次（D8（核心索引结构） 已定项 16 第 1 条）：inode 叶与根（nlink 写 0）、`deleted_inodes` 根（加 key）、
    /// 墓碑树开放容器与根（追加一条墓碑），加固定点单元；数据单元与 extent 树照抄。`deleted` 是删的那个号（判决
    /// `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M1：D8（核心索引结构） 已定项 16 第 1、2 条对任一 inode 同一条）；
    /// 模型只答 inode 树只有一片叶容器的一版（那条记录住最左那片）。
    UnlinkIntent { deleted: ModelDeletedInode },
    /// 不是最后一批的删除批（D8（核心索引结构） 已定项 16 第 2 条：删的那个号剩下的单元多于产品批大小 144 时从尾部释放 144 个）：
    /// 留下的数据单元照抄（分配代不变）；重写那个文件被截掉那一片下段叶与它的祖先（下段节点不对应角色，占槽按下段层数取上界）、
    /// extent 上段那个号那一串（连根）、inode 叶与根（记录的 size 改成剩下单元数 × 净荷容量，nlink 仍 0），加固定点单元；
    /// `deleted_inodes` 那条 key 与墓碑树不动。这一版仍是删除中的那一版。
    DeletionBatchKeepingDataUnits { deleted: ModelDeletedInode },
    /// 最后一批（D8（核心索引结构） 已定项 16 第 2 条）：删的那个号剩下的单元不多于产品批大小 144 时意图之后（或上一批之后）的下一步——
    /// 释放剩下的单元、删 inode 记录、删 `deleted_inodes` 那条 key（删空取根指针置零、不写它的节点），同一次发布；inode 根与固定点单元重写。
    /// `inode_tree_after` 说删掉那条记录之后 inode 树空没空（空了那片叶容器身份消失，退役树开放容器追加一条退役记录、连根重写，判决
    /// `research/prompts/m3-retire-r1-main-verification.md` 第三节第 3 条；没空的那片叶容器重写），`extent_upper_root` 说上段的根这次重写还是照抄
    /// （规范形：那个号出了有条目的号集合，降层那次上一版里降后层级的第 0 个节点照抄成根；它本来没有条目时上段整段照抄）。
    FinalDeletionBatch {
        deleted: ModelDeletedInode,
        inode_tree_after: ModelInodeTreeAfterTheFinalDeletionBatch,
        extent_upper_root: ModelExtentUpperRootOfTheFinalDeletionBatch,
    },
    /// 建一个 size 0 的 inode（`transaction::publish_new_inodes`，count 1；D8（核心索引结构） 已定项 16 第 5 条：删完之后只许建 inode、
    /// 写行与空发布），新记录落在最左那片叶容器里（删完之后建的前 233 条，[`inode_leaf_container_position_of_a_new_record`]）：
    /// inode 叶与根，加固定点单元；数据单元与 extent 树根照抄（新建的 inode 长度 0）。
    /// 执行器只在第一个文件已删完的一版上调（`history.rs` 的 `HistoryOperation::CreateInode`）。
    NewInode,
    /// 同 [`ModelPublishKind::NewInode`]，新记录落在最左那片之外的那一片里（删完之后建的第 234 条起）：只重写那一片与 inode 根，
    /// 左边那几片一个字节都不动、分配代不变（D8（核心索引结构） 已定项 6：分裂点取末尾，左半保留全部原有记录与身份；
    /// 「更新一条记录 = COW 叶 + 全部祖先」）。
    NewInodeInALeafContainerRightOfTheLeftmost(ModelInodeLeafContainerRightOfTheLeftmost),
    /// 建一个只读快照那次独立发布（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S2）：文件内容与 inode 树都不动，
    /// 只重写固定点单元（同有树的一版上的空发布那几族），树表追加一对快照条目、活头的 `previous_snapshot_txg` 换成这次的 txg；一个事务。
    SnapshotCreation,
    /// 删快照的意图那次（判决 S4）：只重写固定点单元，按建的先后第 `position` 个快照带上「销毁中」、它的前驱传给派生链上的下一侧；
    /// 一条 deadlist 条目都不动；一个事务。
    SnapshotDestructionIntent { position: usize },
    /// 删快照意图之后、不是最后一批的一批（判决 S4）：只重写固定点单元（连同 deadlist 树），放掉第 `position` 个快照那一段里能放的条目中的
    /// `entries_released` 条；一个事务。
    SnapshotDestructionBatch {
        position: usize,
        entries_released: u64,
    },
    /// 删快照的最后一批：放掉剩下能放的全部条目，同一次发布删掉第 `position` 个快照那一对条目；一个事务。
    SnapshotDestructionFinalBatch { position: usize },
    /// 整理的一批（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3，`transaction::publish_relocation_batch`）：至多
    /// `units_at_most` 个进映射的单元原样字节搬到目的地，映射条目换位置、目的地的分配代写单元的诞生代（D26（后台整理与放置回收） 已定项 5 第 2 条）——
    /// 搬走的单元分配代不变，模型里它们的角色照抄；只重写固定点单元（同删快照那几次），文件内容、extent 树、inode 树、快照与 deadlist 都不动
    /// （被快照钉住的单元照搬，留存的映射条目跟着换位置，条数不变）；一个事务。模型不答搬哪几个、搬几个（那由选段与区域扫描定，判决 G7「只比
    /// 成功操作的内容」）：一批搬了几个实现交回几个，模型只按至多 `units_at_most` 个两槽单元给占槽的上界。
    /// 搬走的是用户可见树的根（inode 树根、extent 树根，活头的或快照条目抄的）时，树表里那条根指针换成映射位置、盘上字节变了，
    /// 这一版算一个新状态（用户 2026-10-15 定「照字节比，搬根算非空」，[`ModelRoot::user_visible_root_relocations`]）。
    /// `intent_after_the_batch` 说这一批之后意图照抄还是同一次发布删掉（这一批搬空了 R）。
    CompactionBatch {
        units_at_most: u64,
        intent_after_the_batch: ModelCompactionIntentAfterTheBatch,
    },
    /// 签发一条整理意图（判决 G1；丁 3 交回的读法第 1 条：签发单独一次发布，不搬单元）：logged_ops 新写一个根装这一条
    /// （[`ModelUnitRole::LoggedOperationsRoot`]，分配代 = 这次的 txg），只重写固定点单元，文件内容、extent 树、inode 树、快照与 deadlist 都不动；
    /// 一个事务（丁 3 读法第 6 条：照建快照取这个实例的下一个号）。
    CompactionIntentIssue,
    /// 不搬单元、只删 logged_ops 那一条意图的一次发布（判决 G1、G4；丁 3 读法第 2、3 条）：R 已按记录全空时意图达成、单独删
    /// （[`ModelCompactionIntentEnding::CompletedWithTheRegionEmptyByTheRecords`]），R 里有隔离槽、区域扫描拒了 R、一个能搬的都没有时
    /// 丢掉它（[`ModelCompactionIntentEnding::Dropped`]）。logged_ops 删空、根指针置零，只重写固定点单元；一个事务。
    CompactionIntentDeletion { ending: ModelCompactionIntentEnding },
    /// 截断的意图那次（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K1、K2「先落目标」、K3）：缩短到 N 一律走
    /// 「意图 + 若干批 + 最后一批」（K11）；意图那次 inode 记录的 size 写成 N（inode 叶与根），logged_ops 签一条截断条目（种类码 2，
    /// logged_ops 根这次重写），N 落在单元 k 中间、单元 k 的声明长度不等于 N − k × 净荷时重写单元 k（截到 0 留一个声明长度 0 的单元），
    /// 连同罩着它的下段叶到根与 extent 上段根；不放单元，加固定点单元；一个事务。这一版读回 N 字节。
    /// 不重写尾单元而上段叶条目的标签要变时（缩短把尾部的洞整个截掉：标签 3、4 换回 1、2）只为换标签重写 extent 上段根（判决第六节 S1：
    /// 换标签写在意图那次，目标态的 size、尾单元、标签一起写下）。`extent_tree` 说是三种里的哪一种。
    TruncationIntent {
        extent_tree: ModelExtentTreeChangeOfASizeChange,
    },
    /// 截断意图之后、不是最后一批的一批（K2）：size 之外还挂着的单元多于批大小 144 时从尾部放 144 个，不碰 inode 记录；重写被截掉那一片
    /// 下段叶与它的祖先、extent 上段根，加固定点单元；截断条目照抄。一个事务。
    TruncationBatch,
    /// 截断的最后一批（K2、K11）：放完 size 之外还挂着的单元（可以 0 个），同一次发布删截断条目；不碰 inode 记录。放了单元时重写下段被截那一片
    /// 与祖先、extent 上段根（下段只剩一个单元时整段换下、指针内联进上段）；放 0 个时 extent 树不动。加固定点单元；一个事务。
    FinalTruncationBatch {
        release: ModelFinalTruncationBatchRelease,
        logged_operations_after: ModelLoggedOperationsAfterTheTruncationEntryIsDeleted,
    },
    /// 不缩短的截断（N ≥ S，K5 的变长与 K12 的 N = S）：不立意图、一次发布做完，inode 记录的 size 写成 N（inode 叶与根），加固定点单元；
    /// 最后一个存在的单元声明长度小于 min(N − 起点, 净荷) 时把它补零重写到那个长度（连同下段路径与 extent 上段根），之后到 N 都是洞、不写单元；
    /// 不重写尾单元而上段叶条目的洞标签要变时只重写 extent 上段根（`extent_tree` 说是哪一种）。一个事务。
    SizeChangeWithoutShrinking {
        extent_tree: ModelExtentTreeChangeOfASizeChange,
    },
    /// 按 inode 号整份写第一个文件之外的一个文件（`transaction::publish_sequential_write_of_an_inode` 与
    /// `publish_overwrite_of_an_inode`，里程碑「删除与多次 COW」一·1）：那个文件的每个数据单元与它的下段整份重写（一个单元一个事务，
    /// D16（发布语义） 已定项 5），extent 树上段按这次之后有条目的 inode 号定形状、写的那一串重写（上段根之下的节点与下段节点都不对应角色，
    /// 占槽与映射条目按形状现算），装着它那条记录的 inode 叶容器（`inode_leaf_container`）与 inode 根，加固定点单元；第一个文件与别的文件的
    /// 数据单元、下段照抄。
    WholeWriteOfAnInodeOtherThanTheFirst { inode_leaf_container: ModelUnitRole },
    /// 第一个文件之外一个没有数据单元的 inode 改 size（`transaction::publish_size_change`，判决
    /// `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S2、S3、S6：任意活 inode 都能缩短与变长）：一个单元都不写、不放，
    /// 装着它那条记录的 inode 叶容器（`inode_leaf_container`）与 inode 根（size 写成 `new_size_in_bytes`），加固定点单元；`change` 说是哪一种
    /// （截断意图连 logged_ops 根，第一次写上粘住的标签 0 连 extent 上段那一串）。一个事务。带数据单元的第一个文件之外的文件改 size 是
    /// [`ModelPublishKind::SizeChangeOfAnInodeHoldingDataUnits`]。
    SizeChangeOfAnInodeWithoutDataUnits {
        inode: u64,
        inode_leaf_container: ModelUnitRole,
        new_size_in_bytes: u64,
        change: ModelSizeChangeOfAnInodeWithoutDataUnits,
    },
    /// 那个 inode 截断的最后一批（K2、K11）：放 0 个单元、同一次发布删截断条目；不碰 inode 记录、不碰 extent 树（标签 0 那一条照旧留着）。
    /// 加固定点单元；一个事务。
    FinalTruncationBatchOfAnInodeWithoutDataUnits {
        inode: u64,
        logged_operations_after: ModelLoggedOperationsAfterTheTruncationEntryIsDeleted,
    },
    /// 第一个文件之外一个带数据单元的 inode 改 size（`transaction::publish_size_change`，判决
    /// `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S3：与第一个文件同一套截断与变长，K2、K3、K5、K12 推到任意活 inode）：
    /// 装着它那条记录的 inode 叶容器（`inode_leaf_container`）与 inode 根（size 写成 `new_size_in_bytes`），加固定点单元；`direction` 说是截断意图
    /// （连 logged_ops 根签截断条目，那条记录的截断字段按这次放不放单元写或照抄）还是不缩短的那一次，`extent_tree` 说那个文件的尾单元与它那一条上段
    /// 叶条目怎样（尾单元重写时连罩着它的下段叶到根；上段叶条目要变时上段按规范形重写它那一串）。一个事务。
    SizeChangeOfAnInodeHoldingDataUnits {
        inode: u64,
        inode_leaf_container: ModelUnitRole,
        new_size_in_bytes: u64,
        direction: ModelDirectionOfASizeChange,
        extent_tree: ModelExtentTreeChangeOfASizeChange,
    },
    /// 那个 inode 截断意图之后、不是最后一批的一批（K2）：size 之外还挂着的单元多于批大小 144 时从尾部放 144 个，不碰 inode 记录；重写它被截掉那一片
    /// 下段叶与祖先、上段它那一串，加固定点单元；截断条目照抄。一个事务。
    TruncationBatchOfAnInodeHoldingDataUnits { inode: u64 },
    /// 那个 inode 截断的最后一批（K2、K11）：放完 size 之外还挂着的（可以 0 个），同一次发布删截断条目；不碰 inode 记录。放了单元时重写下段被截那一片
    /// 与祖先、上段它那一串，放 0 个时 extent 树不动。加固定点单元；一个事务。
    FinalTruncationBatchOfAnInodeHoldingDataUnits {
        inode: u64,
        release: ModelFinalTruncationBatchRelease,
        logged_operations_after: ModelLoggedOperationsAfterTheTruncationEntryIsDeleted,
    },
    /// 按偏移写第一个文件（`transaction::publish_partial_write`，判决 `research/prompts/m4-partial-write-r1-main-verification.md` 第二节）：
    /// 从单元 `first_rewritten_data_unit` 起到写之后最后一个单元（写之后 `data_units_after` 个）一起重写（R4：写到偏移 o 时把 o 之后的单元一起重写），
    /// 一个单元一个事务；罩着它们的下段节点与新长出来的下段节点（不对应角色，按位置另记）、extent 上段根、inode 叶与根（size 写成写之后的，
    /// 截断字段照抄，R5），加固定点单元。内容、size、空隙读 0 由读回的整份带进来（[`IdealModel::answer_partial_write`]）。
    PartialWriteOfTheFirstFile {
        first_rewritten_data_unit: u64,
        data_units_after: u64,
    },
    /// 按偏移写第一个文件之外的一个文件（同上一种，R6：任意活 inode）：那个文件的单元与下段按单元号另记，extent 上段它那一串、装着它那条记录的
    /// inode 叶容器（`inode_leaf_container`）与 inode 根，加固定点单元；第一个文件照抄。
    PartialWriteOfAnInodeOtherThanTheFirst {
        inode: u64,
        inode_leaf_container: ModelUnitRole,
        first_rewritten_data_unit: u64,
        data_units_after: u64,
    },
}

/// 一次改 size 缩不缩短（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K1、K5、K12）：N < S 走截断意图，N ≥ S 一次发布做完。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelDirectionOfASizeChange {
    TruncationIntent,
    WithoutShrinking,
}

/// 第一个文件之外一个没有数据单元的 inode 改 size 是哪一种（[`ModelPublishKind::SizeChangeOfAnInodeWithoutDataUnits`]）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ModelSizeChangeOfAnInodeWithoutDataUnits {
    /// N ≥ S，上段里已有它那一条标签 0、或改完 size 仍是 0（上段里本来没有它，照旧没有）：extent 树整棵照抄。
    WithoutShrinkingCarryingTheExtentTree,
    /// N > S，上段里还没有它那一条（size 0、没改过 size）：这次起写它那一条标签 0、之后一直留着（粘住的标签 0，判决第二节 S2），
    /// extent 上段按这次之后有条目的号重排、它那一串重写。
    WithoutShrinkingWritingTheTagZeroEntry,
    /// N < S：截断意图（K1、K2），logged_ops 签一条截断条目（根这次重写）；extent 树整棵照抄（一个单元都没有：尾单元不重写、标签留 0）。
    TruncationIntent,
}

/// 删除那几步删的是哪个号（判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M1、M3：意图、批、最后一批对任一号）。
/// 封闭集合：第一个文件的内容、单元与截断字段记在 [`ModelVersionContents`] 里，别的文件记在 [`ModelRoot::inodes_other_than_the_first`] 里，
/// 两处各走各的。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ModelDeletedInode {
    FirstFile,
    OtherThanTheFirst(u64),
}

impl ModelDeletedInode {
    fn of_the_inode_number(inode: u64) -> Self {
        if inode == FIRST_INODE_NUMBER {
            ModelDeletedInode::FirstFile
        } else {
            ModelDeletedInode::OtherThanTheFirst(inode)
        }
    }

    fn inode_number(self) -> u64 {
        match self {
            ModelDeletedInode::FirstFile => FIRST_INODE_NUMBER,
            ModelDeletedInode::OtherThanTheFirst(inode) => inode,
        }
    }
}

/// 删除的最后一批删掉那条记录之后 inode 树还有没有记录（D8（核心索引结构） 已定项 16 第 5 条）。模型只答树里只有一片叶容器的一版
/// （删一条不并、不借，D8（核心索引结构） 已定项 6 的合并只在有邻片时发生）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelInodeTreeAfterTheFinalDeletionBatch {
    /// 删的是树里最后一条：那片叶容器身份消失（退役），inode 树写零条目根。
    Emptied,
    /// 树里还有别的记录：那片叶容器去掉一条、重写，身份不变。
    StillHoldsRecords,
}

/// 删除的最后一批之后 extent 上段的根这次重写还是照抄（规范形，判决 m3-multi-file-unlink-r1 第三节 M2）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelExtentUpperRootOfTheFinalDeletionBatch {
    /// 根罩着删掉的那个号（层级没降），或有条目的号集合删空了（零条目根兼叶）。
    Rewritten,
    /// 降层：上一版里降后层级的第 0 个节点照抄成根；或删的号本来就没有条目，上段整段照抄。
    CarriedFromThePreviousVersion,
}

/// 截断的最后一批放没放单元（K2「最后一批放完剩下的（可以 0 个）」）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelFinalTruncationBatchRelease {
    NoDataUnit,
    AtLeastOneDataUnit,
}

/// 截断的最后一批删掉截断条目之后 logged_ops 里还剩什么（K1：截断条目与整理意图同住 logged_ops）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelLoggedOperationsAfterTheTruncationEntryIsDeleted {
    /// 删空：树表条目的根指针置零，这一版没有 logged_ops 根。
    Emptied,
    /// 还装着那一条活整理意图：logged_ops 根这次重写。
    StillHoldsTheCompactionIntent,
}

/// 改 size 的那一次发布（截断意图与不缩短的截断）对 extent 树做了什么（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K3、K5，
/// 第六节 S1）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelExtentTreeChangeOfASizeChange {
    /// 尾单元重写：截断意图重写单元 ⌈N ÷ 净荷⌉.max(1) − 1（K3），不缩短的截断补零重写最后一个存在的单元（K5）；那个单元、罩着它的下段叶到根
    /// 与 extent 上段根这次写。
    TailDataUnitRewritten,
    /// 单元不动，上段叶条目的洞标签变了：变长把标签 1、2 换成 3、4（K5），缩短把洞整个截掉时 3、4 换回 1、2（S1）。只重写 extent 上段根。
    OnlyTheUpperLeafEntryLabel,
    /// extent 树不动（N = S，或尾单元不用重写、洞标签不变）。
    Untouched,
}

/// 上段叶条目的标签（D8（核心索引结构） 已定项 14；判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K5 加 3、4 声明尾部的洞，
/// 用户 2026-10-16 定）。模型按条款自己写一份，不引实现的常量（D13（验证路线） 已定项 5）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModelExtentUpperLeafEntryLabel {
    /// 标签 0：没有数据单元；配 size > 0 是整个文件都是洞。
    NoDataUnit,
    /// 标签 1：下段根。
    LowerSegmentRoot,
    /// 标签 2：内联单元 0。
    InlineDataUnit,
    /// 标签 3：下段根，下段最后一个单元之后到 size 是洞。
    LowerSegmentRootThenTailHole,
    /// 标签 4：内联单元 0，单元 0 之后到 size 是洞。
    InlineDataUnitThenTailHole,
}

impl ModelExtentUpperLeafEntryLabel {
    /// 盘上的标签值（K5）。
    #[must_use]
    pub fn tag(self) -> u8 {
        match self {
            ModelExtentUpperLeafEntryLabel::NoDataUnit => 0,
            ModelExtentUpperLeafEntryLabel::LowerSegmentRoot => 1,
            ModelExtentUpperLeafEntryLabel::InlineDataUnit => 2,
            ModelExtentUpperLeafEntryLabel::LowerSegmentRootThenTailHole => 3,
            ModelExtentUpperLeafEntryLabel::InlineDataUnitThenTailHole => 4,
        }
    }

    /// 写上段叶条目那一刻按存在的单元数与 size 罩的单元数（⌈size ÷ 净荷⌉.max(1)）写哪个标签：没有单元是 0；一个单元内联、多个单元下段根；
    /// 存在的少于 size 罩的就是尾部有洞（K5：只有标签 3、4、0 许单元数少于它）。截断途中 size 之外还挂着单元（存在的多于罩的）时写 1 或 2。
    #[must_use]
    pub fn written_for(data_units_present: u64, data_units_within_the_size: u64) -> Self {
        let has_a_tail_hole = data_units_present < data_units_within_the_size;
        match (data_units_present, has_a_tail_hole) {
            (0, true | false) => ModelExtentUpperLeafEntryLabel::NoDataUnit,
            (1, false) => ModelExtentUpperLeafEntryLabel::InlineDataUnit,
            (1, true) => ModelExtentUpperLeafEntryLabel::InlineDataUnitThenTailHole,
            (_, false) => ModelExtentUpperLeafEntryLabel::LowerSegmentRoot,
            (_, true) => ModelExtentUpperLeafEntryLabel::LowerSegmentRootThenTailHole,
        }
    }

    /// 带洞的那三个（K5：只有它们许单元数少于 size 罩的）。
    #[must_use]
    pub fn declares_a_hole(self) -> bool {
        match self {
            ModelExtentUpperLeafEntryLabel::NoDataUnit
            | ModelExtentUpperLeafEntryLabel::LowerSegmentRootThenTailHole
            | ModelExtentUpperLeafEntryLabel::InlineDataUnitThenTailHole => true,
            ModelExtentUpperLeafEntryLabel::LowerSegmentRoot
            | ModelExtentUpperLeafEntryLabel::InlineDataUnit => false,
        }
    }

    /// 这个标签与存在的单元数、size 罩的单元数对不对得上（判决第三节 K7 的 T-3，模型这一侧的写法，第六节 S1 收成唯一）：标签说的内联 / 下段
    /// 与存在的单元数相符；标签 1、2 不许有洞（单元数等于 size 罩的），标签 3、4、0 至少有一个洞单元（少于它）；有截断条目时 size 之外可以
    /// 多挂着单元，那时 size 之内没有洞，只许标签 1、2（size 之外挂着的不算洞单元）。
    fn admits(
        self,
        data_units_present: u64,
        data_units_within_the_size: u64,
        is_a_truncation_entry_logged: bool,
    ) -> bool {
        let shape_matches = match self {
            ModelExtentUpperLeafEntryLabel::NoDataUnit => data_units_present == 0,
            ModelExtentUpperLeafEntryLabel::InlineDataUnit
            | ModelExtentUpperLeafEntryLabel::InlineDataUnitThenTailHole => data_units_present == 1,
            ModelExtentUpperLeafEntryLabel::LowerSegmentRoot
            | ModelExtentUpperLeafEntryLabel::LowerSegmentRootThenTailHole => {
                data_units_present >= 2
            }
        };
        let count_matches =
            if is_a_truncation_entry_logged && data_units_present > data_units_within_the_size {
                !self.declares_a_hole()
            } else if self.declares_a_hole() {
                data_units_present < data_units_within_the_size
            } else {
                data_units_present == data_units_within_the_size
            };
        shape_matches && count_matches
    }
}

/// 整理的一批之后意图怎样（D8（核心索引结构） 已定项 5「删除意图必须与最后一批工作在同一个事务里提交」）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelCompactionIntentAfterTheBatch {
    /// 这一批之后 R 里还有未释放的记录：logged_ops 照抄。
    CarriedOn,
    /// 这一批搬空了 R（按记录）：同一次发布删意图。
    DeletedWithTheLastBatch,
}

/// 一条整理意图不随一批、单独一次发布离开 logged_ops 的两种。封闭集合。模型不记分配记录，分不出是哪一种：执行器按实现交回的这一版
/// R 按记录全空没有交给模型（[`ModelCompactionPublishSeen`]），两种的根一样，只计数分开。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelCompactionIntentEnding {
    /// R 已按记录全空：意图达成（判决 G4 停机谓词）。
    CompletedWithTheRegionEmptyByTheRecords,
    /// R 里还有未释放的记录而推不下去（隔离、区域扫描拒、一个能搬的都没有）：丢掉（攻方 K-c；判决 G2）。
    Dropped,
}

impl ModelUnitRole {
    /// 换下时问不问快照生死判定（判决 S3：只有用户可见子树的六类角色问，其余一律直接进 defer）：模型的角色里数据单元、extent 根、
    /// inode 叶容器与 inode 根是那六类里的四类（extent 树下段节点不对应角色，按位置另记；上段根之下的节点各是一个角色，同上段根问）。
    fn asks_the_snapshot_lifetime_judgement(self) -> bool {
        match self {
            ModelUnitRole::Data
            | ModelUnitRole::ExtentRoot
            | ModelUnitRole::ExtentUpperNodeBelowTheRoot(_)
            | ModelUnitRole::InodeLeaf
            | ModelUnitRole::InodeLeafRightOfTheLeftmost(_)
            | ModelUnitRole::InodeRoot => true,
            ModelUnitRole::AllocationTree
            | ModelUnitRole::AccountingTree
            | ModelUnitRole::MappingTree
            | ModelUnitRole::TreeTable
            | ModelUnitRole::InstanceTable
            | ModelUnitRole::DeletedInodesRoot
            | ModelUnitRole::LoggedOperationsRoot
            | ModelUnitRole::TombstoneLeaf
            | ModelUnitRole::TombstoneRoot
            | ModelUnitRole::RetirementLeaf
            | ModelUnitRole::RetirementRoot => false,
        }
    }
}

/// inode 号的宽：内部节点条目里的分隔 key 8 字节（D8（核心索引结构） 已定项 6：条目 = 分隔 key 8 + 身份引用 26 + 子指针 86）。
const INODE_TREE_KEY_BYTES: u64 = 8;

/// 删完第一个文件之后建的第 `records_before` 条（从 0 数，也就是这次之前树里已有几条）落在 inode 树左起第几片叶容器（从 0 数）。
/// D8（核心索引结构） 已定项 6：插入永远落在最右的叶，满 233 条（`INODE_LEAF_RECORDS`）就在末尾分裂、左半不动；模型罩的历史里
/// 这棵树删完第一个文件之后是空的、之后只经建 inode 往右追加，所以左边每片都装满 233 条、第 k 条落在第 ⌊k ÷ 233⌋ 片。
///
/// # Panics
/// 落点超出一个码 2 根装得下的叶容器数（扇出 135，D8（核心索引结构） 已定项 6）：根要长一层，实现今天在那里拒
/// （`MoreLeafContainersThanOneRootNodeHolds`），模型罩的历史建不到三万多个 inode。
fn inode_leaf_container_position_of_a_new_record(records_before: u64) -> u64 {
    let position_from_the_leftmost = records_before / INODE_LEAF_RECORDS;
    let leaf_containers_one_root_holds =
        index_node_entry_capacity(INODE_TREE_KEY_BYTES, INODE_INTERNAL_ENTRY);
    assert!(
        position_from_the_leftmost < leaf_containers_one_root_holds,
        "模型今天只罩 inode 树根之下直接挂叶容器的历史（一个根装 {leaf_containers_one_root_holds} 片）：第 {records_before} 条落在第 {position_from_the_leftmost} 片"
    );
    position_from_the_leftmost
}

/// 装得下 `rows` 行要几片（D18（块里携带什么信息） 已定项 11：一片 370 条记录，链指针记录恒为一片的最后一条、数据行 369；
/// 0 行也是一片）。模型按条款自己算，不调实现的那一份。
#[must_use]
pub fn instance_table_pages_for_rows(rows: usize) -> u64 {
    let rows_per_page = INSTANCE_TABLE_PAGE_RECORDS - 1;
    u64::try_from(rows)
        .expect("行数装得进 u64")
        .div_ceil(rows_per_page)
        .max(1)
}

impl ModelPublishKind {
    /// 这次发布让 inode 号水位变了、给记账留一级台阶行（D5（快照 / 空间记账机制） 已定项 2 射程「inode 号水位的台阶行」）：
    /// 第一个文件那次（inode 1 落下，水位从没有记账树到 2）与建 inode 那两种（每次加 1）。别的发布不发号；管理员回退那次的水位
    /// 取环里没被崩溃恢复抛弃的根的最大值，模型罩的历史里那就是现行那一版的（D23（journal 的角色与格式） 已定项 14「水位」那一格），不留新台阶。
    fn leaves_an_inode_number_watermark_step(self) -> bool {
        match self {
            ModelPublishKind::FirstFileVersion
            | ModelPublishKind::NewInode
            | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_) => true,
            ModelPublishKind::OverwriteFileVersion
            | ModelPublishKind::RowsOnFileVersion
            | ModelPublishKind::EmptyOnFileVersion
            | ModelPublishKind::RowsOnVersionWithoutFile
            | ModelPublishKind::ZeroUnit
            | ModelPublishKind::UnlinkIntent { .. }
            | ModelPublishKind::DeletionBatchKeepingDataUnits { .. }
            | ModelPublishKind::FinalDeletionBatch { .. }
            | ModelPublishKind::SnapshotCreation
            | ModelPublishKind::SnapshotDestructionIntent { .. }
            | ModelPublishKind::SnapshotDestructionBatch { .. }
            | ModelPublishKind::SnapshotDestructionFinalBatch { .. }
            | ModelPublishKind::CompactionBatch { .. }
            | ModelPublishKind::CompactionIntentIssue
            | ModelPublishKind::CompactionIntentDeletion { .. }
            | ModelPublishKind::TruncationIntent { .. }
            | ModelPublishKind::TruncationBatch
            | ModelPublishKind::FinalTruncationBatch { .. }
            | ModelPublishKind::SizeChangeWithoutShrinking { .. }
            | ModelPublishKind::WholeWriteOfAnInodeOtherThanTheFirst { .. }
            | ModelPublishKind::SizeChangeOfAnInodeWithoutDataUnits { .. }
            | ModelPublishKind::SizeChangeOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::TruncationBatchOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::PartialWriteOfTheFirstFile { .. }
            | ModelPublishKind::PartialWriteOfAnInodeOtherThanTheFirst { .. }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeWithoutDataUnits { .. } => false,
        }
    }

    /// 这次发布重写不重写实例表：写行那两种重写整条链，别的照抄。
    fn rewrites_the_instance_table(self) -> bool {
        match self {
            ModelPublishKind::RowsOnFileVersion | ModelPublishKind::RowsOnVersionWithoutFile => {
                true
            }
            ModelPublishKind::FirstFileVersion
            | ModelPublishKind::OverwriteFileVersion
            | ModelPublishKind::EmptyOnFileVersion
            | ModelPublishKind::ZeroUnit
            | ModelPublishKind::UnlinkIntent { .. }
            | ModelPublishKind::DeletionBatchKeepingDataUnits { .. }
            | ModelPublishKind::FinalDeletionBatch { .. }
            | ModelPublishKind::NewInode
            | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_)
            | ModelPublishKind::SnapshotCreation
            | ModelPublishKind::SnapshotDestructionIntent { .. }
            | ModelPublishKind::SnapshotDestructionBatch { .. }
            | ModelPublishKind::SnapshotDestructionFinalBatch { .. }
            | ModelPublishKind::CompactionBatch { .. }
            | ModelPublishKind::CompactionIntentIssue
            | ModelPublishKind::CompactionIntentDeletion { .. }
            | ModelPublishKind::TruncationIntent { .. }
            | ModelPublishKind::TruncationBatch
            | ModelPublishKind::FinalTruncationBatch { .. }
            | ModelPublishKind::SizeChangeWithoutShrinking { .. }
            | ModelPublishKind::WholeWriteOfAnInodeOtherThanTheFirst { .. }
            | ModelPublishKind::SizeChangeOfAnInodeWithoutDataUnits { .. }
            | ModelPublishKind::SizeChangeOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::TruncationBatchOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::PartialWriteOfTheFirstFile { .. }
            | ModelPublishKind::PartialWriteOfAnInodeOtherThanTheFirst { .. }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeWithoutDataUnits { .. } => false,
        }
    }

    /// 这次发布的记录承载几个事务（事务号接着本实例的计数往下取）：写文件内容的两种一个数据单元一个事务（D16（发布语义） 已定项 5
    /// 的切分纪律，`data_units_of_the_new_content` 是这次写的内容切成几个单元）；删除类发布与建 inode 各一个事务；
    /// 空发布、写行、暖机、抬 F、回退的记录事务号 0（D23（journal 的角色与格式） 已定项 19 ①）。
    fn transactions_carried(self, data_units_of_the_new_content: u64) -> u64 {
        match self {
            // 按 inode 号整份写第一个文件之外的文件同一条切分纪律：`data_units_of_the_new_content` 是那个文件这次写的单元数。
            ModelPublishKind::FirstFileVersion
            | ModelPublishKind::OverwriteFileVersion
            | ModelPublishKind::WholeWriteOfAnInodeOtherThanTheFirst { .. } => {
                data_units_of_the_new_content
            }
            // 按偏移写同一条切分纪律（C310 用户定案「一条记录装一个事务」）：这次重写的每个单元一个事务。
            ModelPublishKind::PartialWriteOfTheFirstFile {
                first_rewritten_data_unit,
                data_units_after,
            }
            | ModelPublishKind::PartialWriteOfAnInodeOtherThanTheFirst {
                first_rewritten_data_unit,
                data_units_after,
                ..
            } => data_units_after - first_rewritten_data_unit,
            ModelPublishKind::UnlinkIntent { .. }
            | ModelPublishKind::DeletionBatchKeepingDataUnits { .. }
            | ModelPublishKind::FinalDeletionBatch { .. }
            | ModelPublishKind::NewInode
            | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_) => 1,
            // 建快照与删快照的意图、每一批各一个事务（`transaction::publish_create_snapshot` 与删除那几次共用的计划，事务号接着实例内计数）；
            // 整理的一批同样一个事务号（`transaction::publish_relocation_batch`）。
            ModelPublishKind::SnapshotCreation
            | ModelPublishKind::SnapshotDestructionIntent { .. }
            | ModelPublishKind::SnapshotDestructionBatch { .. }
            | ModelPublishKind::SnapshotDestructionFinalBatch { .. }
            | ModelPublishKind::CompactionBatch { .. }
            | ModelPublishKind::CompactionIntentIssue
            | ModelPublishKind::CompactionIntentDeletion { .. } => 1,
            // 截断的意图、每一批与不缩短的那一次各一个事务：至多写一个单元的用户数据（尾单元），D16（发布语义） 已定项 5 末句随
            // 判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K12 补上「截断的尾单元算」。
            ModelPublishKind::TruncationIntent { .. }
            | ModelPublishKind::TruncationBatch
            | ModelPublishKind::FinalTruncationBatch { .. }
            | ModelPublishKind::SizeChangeWithoutShrinking { .. }
            | ModelPublishKind::SizeChangeOfAnInodeWithoutDataUnits { .. }
            | ModelPublishKind::SizeChangeOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::TruncationBatchOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeWithoutDataUnits { .. } => 1,
            ModelPublishKind::RowsOnFileVersion
            | ModelPublishKind::EmptyOnFileVersion
            | ModelPublishKind::RowsOnVersionWithoutFile
            | ModelPublishKind::ZeroUnit => 0,
        }
    }

    /// 这次发布之后这一版不再有的角色（数据单元不在这里，按单元号另记，[`ModelRoot::data_unit_written_at`]）：最后一批 `deleted_inodes`
    /// 删空取根指针置零，删的是树里最后一条时连唯一那片 inode 叶容器；删整理意图、截断最后一批删空 logged_ops 时取根指针置零。
    /// extent 上段根之下的节点不在这里（按规范形计划另算，[`extent_upper_node_written_at_after`]）。
    fn removed_roles(self) -> &'static [ModelUnitRole] {
        match self {
            ModelPublishKind::FinalDeletionBatch {
                inode_tree_after: ModelInodeTreeAfterTheFinalDeletionBatch::Emptied,
                ..
            } => &[ModelUnitRole::InodeLeaf, ModelUnitRole::DeletedInodesRoot],
            ModelPublishKind::FinalDeletionBatch {
                inode_tree_after: ModelInodeTreeAfterTheFinalDeletionBatch::StillHoldsRecords,
                ..
            } => &[ModelUnitRole::DeletedInodesRoot],
            ModelPublishKind::FirstFileVersion
            | ModelPublishKind::OverwriteFileVersion
            | ModelPublishKind::RowsOnFileVersion
            | ModelPublishKind::EmptyOnFileVersion
            | ModelPublishKind::RowsOnVersionWithoutFile
            | ModelPublishKind::ZeroUnit
            | ModelPublishKind::UnlinkIntent { .. }
            | ModelPublishKind::DeletionBatchKeepingDataUnits { .. }
            | ModelPublishKind::NewInode
            | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_)
            | ModelPublishKind::SnapshotCreation
            | ModelPublishKind::SnapshotDestructionIntent { .. }
            | ModelPublishKind::SnapshotDestructionBatch { .. }
            | ModelPublishKind::SnapshotDestructionFinalBatch { .. }
            | ModelPublishKind::CompactionIntentIssue
            | ModelPublishKind::CompactionBatch {
                intent_after_the_batch: ModelCompactionIntentAfterTheBatch::CarriedOn,
                ..
            }
            | ModelPublishKind::TruncationIntent { .. }
            | ModelPublishKind::TruncationBatch
            | ModelPublishKind::FinalTruncationBatch {
                logged_operations_after:
                    ModelLoggedOperationsAfterTheTruncationEntryIsDeleted::StillHoldsTheCompactionIntent,
                ..
            }
            | ModelPublishKind::SizeChangeWithoutShrinking { .. }
            | ModelPublishKind::PartialWriteOfTheFirstFile { .. }
            | ModelPublishKind::PartialWriteOfAnInodeOtherThanTheFirst { .. }
            | ModelPublishKind::WholeWriteOfAnInodeOtherThanTheFirst { .. }
            | ModelPublishKind::SizeChangeOfAnInodeWithoutDataUnits { .. }
            | ModelPublishKind::SizeChangeOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::TruncationBatchOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeHoldingDataUnits {
                logged_operations_after:
                    ModelLoggedOperationsAfterTheTruncationEntryIsDeleted::StillHoldsTheCompactionIntent,
                ..
            }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeWithoutDataUnits {
                logged_operations_after:
                    ModelLoggedOperationsAfterTheTruncationEntryIsDeleted::StillHoldsTheCompactionIntent,
                ..
            } => &[],
            // 删意图（随最后一批、单独删、丢）：logged_ops 删空，树表条目的根指针置零，这一版没有这个角色（判决 G1）；截断的最后一批删掉截断条目、
            // logged_ops 里没有整理意图时同样删空（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K1、K2）。
            ModelPublishKind::CompactionBatch {
                intent_after_the_batch: ModelCompactionIntentAfterTheBatch::DeletedWithTheLastBatch,
                ..
            }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeHoldingDataUnits {
                logged_operations_after:
                    ModelLoggedOperationsAfterTheTruncationEntryIsDeleted::Emptied,
                ..
            }
            | ModelPublishKind::CompactionIntentDeletion { .. }
            | ModelPublishKind::FinalTruncationBatch {
                logged_operations_after:
                    ModelLoggedOperationsAfterTheTruncationEntryIsDeleted::Emptied,
                ..
            }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeWithoutDataUnits {
                logged_operations_after:
                    ModelLoggedOperationsAfterTheTruncationEntryIsDeleted::Emptied,
                ..
            } => &[ModelUnitRole::LoggedOperationsRoot],
        }
    }

    /// 这次发布重写的角色：种类定死的那几个（[`ModelPublishKind::rewritten_roles_fixed_by_the_kind`]），建 inode 落在最左那片之外时
    /// 再加新记录落的那一片。
    fn rewritten_roles(self) -> Vec<ModelUnitRole> {
        let mut roles = self.rewritten_roles_fixed_by_the_kind().to_vec();
        match self {
            ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(leaf_container) => {
                roles.push(ModelUnitRole::InodeLeafRightOfTheLeftmost(leaf_container));
            }
            ModelPublishKind::WholeWriteOfAnInodeOtherThanTheFirst {
                inode_leaf_container,
            }
            | ModelPublishKind::SizeChangeOfAnInodeWithoutDataUnits {
                inode_leaf_container,
                ..
            }
            | ModelPublishKind::SizeChangeOfAnInodeHoldingDataUnits {
                inode_leaf_container,
                ..
            }
            | ModelPublishKind::PartialWriteOfAnInodeOtherThanTheFirst {
                inode_leaf_container,
                ..
            } => roles.push(inode_leaf_container),
            ModelPublishKind::FirstFileVersion
            | ModelPublishKind::OverwriteFileVersion
            | ModelPublishKind::RowsOnFileVersion
            | ModelPublishKind::EmptyOnFileVersion
            | ModelPublishKind::RowsOnVersionWithoutFile
            | ModelPublishKind::ZeroUnit
            | ModelPublishKind::UnlinkIntent { .. }
            | ModelPublishKind::DeletionBatchKeepingDataUnits { .. }
            | ModelPublishKind::FinalDeletionBatch { .. }
            | ModelPublishKind::NewInode
            | ModelPublishKind::SnapshotCreation
            | ModelPublishKind::SnapshotDestructionIntent { .. }
            | ModelPublishKind::SnapshotDestructionBatch { .. }
            | ModelPublishKind::SnapshotDestructionFinalBatch { .. }
            | ModelPublishKind::CompactionBatch { .. }
            | ModelPublishKind::CompactionIntentIssue
            | ModelPublishKind::CompactionIntentDeletion { .. }
            | ModelPublishKind::TruncationIntent { .. }
            | ModelPublishKind::TruncationBatch
            | ModelPublishKind::FinalTruncationBatch { .. }
            | ModelPublishKind::SizeChangeWithoutShrinking { .. }
            | ModelPublishKind::PartialWriteOfTheFirstFile { .. }
            | ModelPublishKind::TruncationBatchOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeWithoutDataUnits { .. } => {}
        }
        roles
    }

    /// 这次发布重写的、由种类定死的角色（建 inode 落在最左那片之外时那一片不在这里，[`ModelPublishKind::rewritten_roles`] 另加）。
    /// 分配记录树（`ModelUnitRole::AllocationTree`）在这里只记它的根：根之下重写几个节点按位置寻址现算上界
    /// （`IdealModel::allocation_record_tree_nodes_rewritten_upper_bound`），模型不记是哪几个。
    fn rewritten_roles_fixed_by_the_kind(self) -> &'static [ModelUnitRole] {
        match self {
            // 按偏移写第一个文件（判决 m4-partial-write-r1 第二节 R4）：重写的那一段数据单元（个数按单元号另数）、extent 上段根、inode 叶与根、固定点单元，
            // 与整份写同一组角色。
            ModelPublishKind::FirstFileVersion
            | ModelPublishKind::OverwriteFileVersion
            | ModelPublishKind::PartialWriteOfTheFirstFile { .. } => &[
                ModelUnitRole::Data,
                ModelUnitRole::ExtentRoot,
                ModelUnitRole::InodeLeaf,
                ModelUnitRole::InodeRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            ModelPublishKind::RowsOnFileVersion => &[
                ModelUnitRole::InstanceTable,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            ModelPublishKind::EmptyOnFileVersion => &[
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            // 树表 0 条的一版上写行：实例表，加这一版自己那片分配记录树
            // （C512（树表 0 条的一版上被换下的单元记在哪），2026-09-23 用户定案：根指针住根记录、不进树表）。
            ModelPublishKind::RowsOnVersionWithoutFile => {
                &[ModelUnitRole::InstanceTable, ModelUnitRole::AllocationTree]
            }
            ModelPublishKind::ZeroUnit => &[],
            ModelPublishKind::UnlinkIntent { .. } => &[
                ModelUnitRole::InodeLeaf,
                ModelUnitRole::InodeRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::DeletedInodesRoot,
                ModelUnitRole::TombstoneLeaf,
                ModelUnitRole::TombstoneRoot,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            ModelPublishKind::DeletionBatchKeepingDataUnits { .. } => &[
                ModelUnitRole::ExtentRoot,
                ModelUnitRole::InodeLeaf,
                ModelUnitRole::InodeRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            // 删的是树里最后一条：唯一那片叶容器退役（退役树追加一条），inode 树写零条目根。
            ModelPublishKind::FinalDeletionBatch {
                inode_tree_after: ModelInodeTreeAfterTheFinalDeletionBatch::Emptied,
                extent_upper_root: ModelExtentUpperRootOfTheFinalDeletionBatch::Rewritten,
                ..
            } => &[
                ModelUnitRole::ExtentRoot,
                ModelUnitRole::InodeRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::RetirementLeaf,
                ModelUnitRole::RetirementRoot,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            ModelPublishKind::FinalDeletionBatch {
                inode_tree_after: ModelInodeTreeAfterTheFinalDeletionBatch::Emptied,
                extent_upper_root:
                    ModelExtentUpperRootOfTheFinalDeletionBatch::CarriedFromThePreviousVersion,
                ..
            } => &[
                ModelUnitRole::InodeRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::RetirementLeaf,
                ModelUnitRole::RetirementRoot,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            // 树里还有别的记录：那片叶容器去掉一条、重写。
            ModelPublishKind::FinalDeletionBatch {
                inode_tree_after: ModelInodeTreeAfterTheFinalDeletionBatch::StillHoldsRecords,
                extent_upper_root: ModelExtentUpperRootOfTheFinalDeletionBatch::Rewritten,
                ..
            } => &[
                ModelUnitRole::ExtentRoot,
                ModelUnitRole::InodeLeaf,
                ModelUnitRole::InodeRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            ModelPublishKind::FinalDeletionBatch {
                inode_tree_after: ModelInodeTreeAfterTheFinalDeletionBatch::StillHoldsRecords,
                extent_upper_root:
                    ModelExtentUpperRootOfTheFinalDeletionBatch::CarriedFromThePreviousVersion,
                ..
            } => &[
                ModelUnitRole::InodeLeaf,
                ModelUnitRole::InodeRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            ModelPublishKind::NewInode => &[
                ModelUnitRole::InodeLeaf,
                ModelUnitRole::InodeRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            // 新记录落的那一片（最左那片之外）由 `rewritten_roles` 另加；最左那片不动。
            ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_) => &[
                ModelUnitRole::InodeRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            // 建快照与删快照的几次、整理的一批：文件内容、extent 树与 inode 树都照抄（`file` 空、不建 inode，判决 S2、S4、G3），只重写固定点单元；
            // deadlist 树的节点不对应角色（同记账树根之下的节点），占槽与映射条目按上界另算；整理搬走的单元角色照抄（分配代 = 诞生代）。
            ModelPublishKind::SnapshotCreation
            | ModelPublishKind::SnapshotDestructionIntent { .. }
            | ModelPublishKind::SnapshotDestructionBatch { .. }
            | ModelPublishKind::SnapshotDestructionFinalBatch { .. }
            | ModelPublishKind::CompactionBatch { .. }
            | ModelPublishKind::CompactionIntentDeletion { .. } => &[
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            // 签发：只把意图写进 logged_ops（新写它的根，判决 G1），不搬单元；文件内容、extent 树、inode 树、快照照抄，加固定点单元
            // （丁 3 交回的读法第 1 条：签发单独一次发布）。
            ModelPublishKind::CompactionIntentIssue => &[
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::LoggedOperationsRoot,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            // 截断意图（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K1、K2、K3）：inode 叶与根（size 写成 N）、logged_ops 根
            // （签截断条目）、固定点单元；重写尾单元时加那个数据单元与 extent 上段根（罩着它的下段节点不对应角色，按位置另记）。
            ModelPublishKind::TruncationIntent {
                extent_tree: ModelExtentTreeChangeOfASizeChange::TailDataUnitRewritten,
            } => &[
                ModelUnitRole::Data,
                ModelUnitRole::ExtentRoot,
                ModelUnitRole::InodeLeaf,
                ModelUnitRole::InodeRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::LoggedOperationsRoot,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            // 把尾部的洞整个截掉、尾单元不用重写：只为换标签重写 extent 上段根（判决第六节 S1）。
            ModelPublishKind::TruncationIntent {
                extent_tree: ModelExtentTreeChangeOfASizeChange::OnlyTheUpperLeafEntryLabel,
            } => &[
                ModelUnitRole::ExtentRoot,
                ModelUnitRole::InodeLeaf,
                ModelUnitRole::InodeRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::LoggedOperationsRoot,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            ModelPublishKind::TruncationIntent {
                extent_tree: ModelExtentTreeChangeOfASizeChange::Untouched,
            } => &[
                ModelUnitRole::InodeLeaf,
                ModelUnitRole::InodeRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::LoggedOperationsRoot,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            // 截断的批不碰 inode 记录（K2）：extent 上段根（被截那一片下段节点与祖先按位置另记）、固定点单元；截断条目照抄。
            // 最后一批放了单元的同样重写 extent 上段根。
            ModelPublishKind::TruncationBatch
            | ModelPublishKind::FinalTruncationBatch {
                release: ModelFinalTruncationBatchRelease::AtLeastOneDataUnit,
                logged_operations_after:
                    ModelLoggedOperationsAfterTheTruncationEntryIsDeleted::Emptied,
            } => &[
                ModelUnitRole::ExtentRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            // logged_ops 删掉截断条目之后还装着整理意图：它的根这次重写。
            ModelPublishKind::FinalTruncationBatch {
                release: ModelFinalTruncationBatchRelease::AtLeastOneDataUnit,
                logged_operations_after:
                    ModelLoggedOperationsAfterTheTruncationEntryIsDeleted::StillHoldsTheCompactionIntent,
            } => &[
                ModelUnitRole::ExtentRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::LoggedOperationsRoot,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            // 放 0 个单元的最后一批：extent 树不动，只删截断条目、写固定点单元。
            ModelPublishKind::FinalTruncationBatch {
                release: ModelFinalTruncationBatchRelease::NoDataUnit,
                logged_operations_after:
                    ModelLoggedOperationsAfterTheTruncationEntryIsDeleted::Emptied,
            } => &[
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            ModelPublishKind::FinalTruncationBatch {
                release: ModelFinalTruncationBatchRelease::NoDataUnit,
                logged_operations_after:
                    ModelLoggedOperationsAfterTheTruncationEntryIsDeleted::StillHoldsTheCompactionIntent,
            } => &[
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::LoggedOperationsRoot,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            // 不缩短的截断（K5、K12）：inode 叶与根（size 写成 N）、固定点单元；补零重写尾单元时加那个数据单元与 extent 上段根，
            // 只换洞标签时只加 extent 上段根。
            ModelPublishKind::SizeChangeWithoutShrinking {
                extent_tree: ModelExtentTreeChangeOfASizeChange::TailDataUnitRewritten,
            } => &[
                ModelUnitRole::Data,
                ModelUnitRole::ExtentRoot,
                ModelUnitRole::InodeLeaf,
                ModelUnitRole::InodeRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            ModelPublishKind::SizeChangeWithoutShrinking {
                extent_tree: ModelExtentTreeChangeOfASizeChange::OnlyTheUpperLeafEntryLabel,
            } => &[
                ModelUnitRole::ExtentRoot,
                ModelUnitRole::InodeLeaf,
                ModelUnitRole::InodeRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            ModelPublishKind::SizeChangeWithoutShrinking {
                extent_tree: ModelExtentTreeChangeOfASizeChange::Untouched,
            } => &[
                ModelUnitRole::InodeLeaf,
                ModelUnitRole::InodeRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            // 第一个文件的数据单元一个不写（数据单元这个角色说的是第一个文件的）；装着那条记录的叶容器由 `rewritten_roles` 另加。
            // 没有数据单元的 inode 第一次写上粘住的标签 0 同样重写 extent 上段它那一串（判决 m4-multi-inode-size-r1 第二节 S2）。
            ModelPublishKind::WholeWriteOfAnInodeOtherThanTheFirst { .. }
            | ModelPublishKind::PartialWriteOfAnInodeOtherThanTheFirst { .. }
            | ModelPublishKind::SizeChangeOfAnInodeWithoutDataUnits {
                change:
                    ModelSizeChangeOfAnInodeWithoutDataUnits::WithoutShrinkingWritingTheTagZeroEntry,
                ..
            } => &[
                ModelUnitRole::ExtentRoot,
                ModelUnitRole::InodeRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            // 没有数据单元的 inode 改 size 而 extent 树整棵照抄：只有 inode 叶（`rewritten_roles` 另加）与根、固定点单元。
            ModelPublishKind::SizeChangeOfAnInodeWithoutDataUnits {
                change:
                    ModelSizeChangeOfAnInodeWithoutDataUnits::WithoutShrinkingCarryingTheExtentTree,
                ..
            } => &[
                ModelUnitRole::InodeRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            // 它的截断意图：inode 叶（另加）与根、logged_ops 根（签截断条目）、固定点单元。
            ModelPublishKind::SizeChangeOfAnInodeWithoutDataUnits {
                change: ModelSizeChangeOfAnInodeWithoutDataUnits::TruncationIntent,
                ..
            } => &[
                ModelUnitRole::InodeRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::LoggedOperationsRoot,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            // 它截断的最后一批（放 0 个）：只删截断条目、写固定点单元，同第一个文件放 0 个的最后一批。
            ModelPublishKind::FinalTruncationBatchOfAnInodeWithoutDataUnits {
                logged_operations_after:
                    ModelLoggedOperationsAfterTheTruncationEntryIsDeleted::Emptied,
                ..
            } => &[
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            ModelPublishKind::FinalTruncationBatchOfAnInodeWithoutDataUnits {
                logged_operations_after:
                    ModelLoggedOperationsAfterTheTruncationEntryIsDeleted::StillHoldsTheCompactionIntent,
                ..
            } => &[
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::LoggedOperationsRoot,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            // 带数据单元的那种 inode 截断意图（判决 m4-multi-inode-size-r1 第二节 S3）：inode 叶（`rewritten_roles` 另加）与根、logged_ops 根（签截断条目）、
            // 固定点单元；尾单元重写或它那一条上段叶条目要换（标签）时连 extent 上段它那一串（根这次重写；它那个文件的数据单元与下段节点按位置另记）。
            ModelPublishKind::SizeChangeOfAnInodeHoldingDataUnits {
                direction: ModelDirectionOfASizeChange::TruncationIntent,
                extent_tree:
                    ModelExtentTreeChangeOfASizeChange::TailDataUnitRewritten
                    | ModelExtentTreeChangeOfASizeChange::OnlyTheUpperLeafEntryLabel,
                ..
            } => &[
                ModelUnitRole::ExtentRoot,
                ModelUnitRole::InodeRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::LoggedOperationsRoot,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            ModelPublishKind::SizeChangeOfAnInodeHoldingDataUnits {
                direction: ModelDirectionOfASizeChange::TruncationIntent,
                extent_tree: ModelExtentTreeChangeOfASizeChange::Untouched,
                ..
            } => &[
                ModelUnitRole::InodeRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::LoggedOperationsRoot,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            // 它不缩短的那一次（K5、K12）：同第一个文件，只是叶容器另加、数据单元按位置另记。
            ModelPublishKind::SizeChangeOfAnInodeHoldingDataUnits {
                direction: ModelDirectionOfASizeChange::WithoutShrinking,
                extent_tree:
                    ModelExtentTreeChangeOfASizeChange::TailDataUnitRewritten
                    | ModelExtentTreeChangeOfASizeChange::OnlyTheUpperLeafEntryLabel,
                ..
            } => &[
                ModelUnitRole::ExtentRoot,
                ModelUnitRole::InodeRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            ModelPublishKind::SizeChangeOfAnInodeHoldingDataUnits {
                direction: ModelDirectionOfASizeChange::WithoutShrinking,
                extent_tree: ModelExtentTreeChangeOfASizeChange::Untouched,
                ..
            } => &[
                ModelUnitRole::InodeRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            // 它截断的批与放了单元的最后一批不碰 inode 记录（K2）：extent 上段它那一串、固定点单元；截断条目删空 logged_ops 时不写它的根。
            ModelPublishKind::TruncationBatchOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeHoldingDataUnits {
                release: ModelFinalTruncationBatchRelease::AtLeastOneDataUnit,
                logged_operations_after:
                    ModelLoggedOperationsAfterTheTruncationEntryIsDeleted::Emptied,
                ..
            } => &[
                ModelUnitRole::ExtentRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            ModelPublishKind::FinalTruncationBatchOfAnInodeHoldingDataUnits {
                release: ModelFinalTruncationBatchRelease::AtLeastOneDataUnit,
                logged_operations_after:
                    ModelLoggedOperationsAfterTheTruncationEntryIsDeleted::StillHoldsTheCompactionIntent,
                ..
            } => &[
                ModelUnitRole::ExtentRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::LoggedOperationsRoot,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            ModelPublishKind::FinalTruncationBatchOfAnInodeHoldingDataUnits {
                release: ModelFinalTruncationBatchRelease::NoDataUnit,
                logged_operations_after:
                    ModelLoggedOperationsAfterTheTruncationEntryIsDeleted::Emptied,
                ..
            } => &[
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            ModelPublishKind::FinalTruncationBatchOfAnInodeHoldingDataUnits {
                release: ModelFinalTruncationBatchRelease::NoDataUnit,
                logged_operations_after:
                    ModelLoggedOperationsAfterTheTruncationEntryIsDeleted::StillHoldsTheCompactionIntent,
                ..
            } => &[
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::LoggedOperationsRoot,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
        }
    }
}

/// 第一个文件那条 inode 记录里的截断字段（判决 `research/prompts/m4-trunc-r2-main-verification.md` 第三节 R1、R2、R5，第六节用户定案）：
/// 记录偏移 120 起两段 8 字节，T（截断字段的 txg）与 N_w（截断字段的 size）。截断意图那次，这次放单元之前存在的单元数多于 ⌈N ÷ 净荷⌉.max(1)
/// 时写成（这次 txg, N），否则照抄上一版（[`ModelTruncationField::after_a_truncation_intent`]）；各批、最后一批、变长与 N = S、unlink 的意图与各批
/// 照抄；整份写（顺序写、覆盖写）清零；回退随回退目标那一版（R5，`contents` 整个取回退目标的）。模型按条款自己写一份，不引实现的算法
/// （D13（验证路线） 已定项 5）。封闭集合：没写过的两段都是 0（R1「T = 0 ⇒ N_w = 0」），写过的 T 是那次截断意图的 txg、恒不为 0。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelTruncationField {
    /// 没写过，或整份写清过：T = 0、N_w = 0。
    NotWritten,
    /// 一次放了单元的截断意图写的：T 是那次意图的 txg，N_w 是那次截到的 size。
    WrittenByATruncationIntent {
        truncation_txg: ModelCheckpointTxg,
        size_in_bytes: u64,
    },
}

impl ModelTruncationField {
    /// 截断意图那次之后的字段（用户 2026-10-17 定案「放了单元才写、整个替换」）：`data_units_present_before` 是这次放单元之前存在的单元数
    /// （截断意图接在带活文件的一版后面，那一版存在的单元数），多于截后 size 罩的 ⌈N ÷ 净荷⌉.max(1) 个时写成（这次 txg, N），否则照抄 `previous`
    /// ——截到洞里不放单元、截到 0 而只有一个单元的都照抄。
    #[must_use]
    pub fn after_a_truncation_intent(
        previous: ModelTruncationField,
        data_units_present_before: u64,
        new_size_in_bytes: u64,
        intent_txg: ModelCheckpointTxg,
    ) -> ModelTruncationField {
        if data_units_present_before > data_units_of_content(new_size_in_bytes) {
            ModelTruncationField::WrittenByATruncationIntent {
                truncation_txg: intent_txg,
                size_in_bytes: new_size_in_bytes,
            }
        } else {
            previous
        }
    }

    /// 盘上那两段的值（R1：没写过的两段都是 0）。
    #[must_use]
    pub fn recorded_values(self) -> ObservedTruncationField {
        match self {
            ModelTruncationField::NotWritten => ObservedTruncationField {
                truncation_txg: ModelCheckpointTxg(0),
                size_in_bytes: 0,
            },
            ModelTruncationField::WrittenByATruncationIntent {
                truncation_txg,
                size_in_bytes,
            } => ObservedTruncationField {
                truncation_txg,
                size_in_bytes,
            },
        }
    }
}

/// 实现写出的第一个文件那条 inode 记录里截断字段那两段的原样值（R1：偏移 120 是 T、偏移 128 是 N_w，各 8 字节小端），胶水从记录的字节里取，
/// 不判合法性（T = 0 而 N_w ≠ 0 也原样交回，模型判对不上）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObservedTruncationField {
    pub truncation_txg: ModelCheckpointTxg,
    pub size_in_bytes: u64,
}

/// 一版的文件：最近写它的 inode 树、extent 树根指针的那次发布的身份与内容。树表里这两棵树的根指针由写出它们的那次发布唯一定（指针带诞生代号），
/// 所以「两条根的这两棵树的根指针相同」⟺「两条根的 `written_by` 相同」（D16（发布语义） 已定项 1「非空」从盘上怎么认）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelFileVersion {
    /// 最近一次写这一版 inode 树或 extent 树根指针的那次发布：写内容的、截断意图、截断的批（只写 extent 树，判决
    /// `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K2「批不碰 inode 记录」）、放了单元的最后一批、不缩短的截断。
    pub written_by: ModelRootKey,
    /// 最近一次写 inode 1 那条记录（inode 树的根）的那次发布：截断的批与最后一批不写它（K2），这时与 `written_by` 不同；
    /// 建快照抄的码 14 根指针是它写的（[`ModelVersionContents::inode_tree_written_by`]）。
    pub inode_record_written_by: ModelRootKey,
    /// 读回的整份字节：size 就是它的长度，洞读 0（K5「读者读洞答 0、不读盘」）。截断途中是目标那 N 字节（K2「先落目标」）。
    pub content: Rc<[u8]>,
    /// 盘上存在的数据单元数（判决第三节 K8）：从单元 0 起连号；不多于 ⌈size ÷ 净荷⌉.max(1)，少于它时尾部是洞（K5）；截断途中 size 之外还挂着
    /// 没放的单元时多于它（K2）。
    pub data_units_present: u64,
    /// 上段叶条目现在的标签（K5）：最近一次重写 extent 上段根时按那一刻的单元数与 size 写的（[`ModelExtentUpperLeafEntryLabel::written_for`]），
    /// 之后不重写上段根的发布照抄。
    pub extent_upper_leaf_entry_label: ModelExtentUpperLeafEntryLabel,
    /// 这一版第一个文件那条 inode 记录里的截断字段（[`ModelTruncationField`]）。
    pub truncation_field: ModelTruncationField,
}

impl ModelFileVersion {
    /// size 罩几个数据单元（⌈size ÷ 净荷⌉.max(1)，[`data_units_of_content`]）。
    #[must_use]
    pub fn data_units_within_the_size(&self) -> u64 {
        data_units_of_content(u64::try_from(self.content.len()).expect("内容长度"))
    }

    /// 存在的单元 `index` 的声明长度：min(size − 起点, 净荷)。截断途中 size 之外还挂着的单元不问（读者不读它们，K2）；带洞的文件存在的单元都是满的
    /// （变长把最后一个存在的单元补零到 min(N − 起点, 净荷)，K5），这个式子同样对。
    fn declared_length_of_the_data_unit(&self, index: u64) -> u64 {
        let payload = data_unit_payload_capacity_in_bytes();
        u64::try_from(self.content.len())
            .expect("内容长度")
            .saturating_sub(index * payload)
            .min(payload)
    }
}

/// 第一个文件之外的一个 inode 在一版里的样子（[`ModelRoot::inodes_other_than_the_first`]）：建 inode 那次建出来（size 0、没有数据单元、
/// extent 树上段里没有它的条目），之后经按 inode 号写数据的两个入口从偏移 0 整份写（里程碑「删除与多次 COW」一·1，用户 2026-10-18 定
/// 「先给 core 加多 inode 写数据」：每次整份重写全部数据单元、同一个分配代，长度 0 的内容也写一个声明长度 0 的单元），也经改 size 缩短与变长
/// （判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S3、S5，用户 2026-10-26 定射程「任意活 inode 都能缩短与变长」：
/// 与第一个文件同一套，尾单元重写、尾部的洞、截断途中 size 之外挂着的单元都照 K2、K3、K5）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelInodeOtherThanTheFirst {
    /// 读回的整份字节：size 就是它的长度，洞读 0（K5）。没写过数据、没改过 size 的是空的；截断途中是目标那 N 字节（K2「先落目标」）。
    pub content: Rc<[u8]>,
    /// 它每个存在的数据单元是哪次发布写的，按单元号（第 i 个是单元 i）：整份写那次全是那一次的 txg，改 size 重写尾单元之后那一个是那一次的
    /// （K4 推到任意 inode：换下时各按各的判快照生死），截断的批与 unlink 的批从尾部去掉。没写过数据的是空的。
    pub data_unit_written_at: Rc<Vec<ModelCheckpointTxg>>,
    /// 它的 extent 树下段每个节点的分配代（换下时按它判钉不钉，判决 m4-snapshot-r1 S3、m4-trunc-r1 K4）：整份写那次整段重写，改 size 与各批只重写罩着
    /// 重写的或放掉的单元的那几个；只剩一个单元时没有下段（同第一个文件的 [`ModelRoot::extent_lower_node_written_at`]）。
    pub extent_lower_node_written_at:
        Rc<BTreeMap<ModelExtentLowerNodePosition, ModelCheckpointTxg>>,
    /// 它那条记录里的截断字段（判决 `research/prompts/m4-trunc-r2-main-verification.md` 第三节 R1、R2 推到任意 inode）：建出来与整份写之后没写过，
    /// 截断意图那次按放不放单元写或照抄（[`ModelTruncationField::after_a_truncation_intent`]），别的照抄；逐版逐个比（判决 m4-multi-inode-size-r1
    /// 第二节 S5「截断字段按 inode 比」）。
    pub truncation_field: ModelTruncationField,
    /// 它那条记录偏移 8 的对象出生代：建它那次发布的 txg（D8（核心索引结构） 已定项 6；按 inode 号整份写不改它）。
    /// K-U5 的 β 时序级读法按它判（[`ModelRefusalReason::DataWriteToAnInodeBornAfterTheFirstFileWasDeleted`]）。
    pub object_birth: ModelCheckpointTxg,
    /// 活着还是删除中（判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M1：删任一号）。
    pub liveness: ModelInodeLiveness,
    /// 上段里有它那一条标签 0（一个单元都没有）：改 size 到 size > 0 那一次写上、之后缩短、缩到 0、再变长都留着，到 unlink 最后一批才去掉
    /// （粘住的标签 0，判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S2）。建出来没改过 size 的是 `false`。
    pub keeps_a_tag_zero_upper_leaf_entry: bool,
    /// 这一版 logged_ops 里点名它的那一条截断条目的签发 txg（截断意图那次写进、最后一批删掉，判决 m4-trunc-r1 K1、K2 推到任意活 inode）；
    /// 不在截断途中为 `None`。
    pub truncation_issued_txg: Option<ModelCheckpointTxg>,
}

/// 第一个文件之外的一个 inode 活着还是删除中（D8（核心索引结构） 已定项 16 第 1、2 条：意图那次 nlink 写 0、号进 `deleted_inodes`，
/// 不是最后一批的批每批从尾部放 144 个单元，最后一批删掉记录）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelInodeLiveness {
    Live,
    /// unlink 的意图那次之后、最后一批之前：记录 nlink = 0（读不出），还剩 `data_units_remaining` 个数据单元（单元号 0 起连着，
    /// 分配代仍按 [`ModelInodeOtherThanTheFirst::data_unit_written_at`] 记，个数与它相同）。
    BeingDeleted {
        data_units_remaining: u64,
    },
}

impl ModelInodeOtherThanTheFirst {
    /// 盘上存在的数据单元数（按单元号记的分配代有几个）：整份写过的至少一个，没写过的 0 个；带洞的少于 size 罩的、截断途中多于它（K2、K5）；
    /// 删除中的是还剩的。
    ///
    /// # Panics
    /// 删除中而还剩的单元数与按单元号记的个数不同（unlink 的批两样一起减）。
    #[must_use]
    pub fn data_units_present(&self) -> u64 {
        let data_units_present = u64::try_from(self.data_unit_written_at.len()).expect("单元数");
        match self.liveness {
            ModelInodeLiveness::Live => {}
            ModelInodeLiveness::BeingDeleted {
                data_units_remaining,
            } => assert_eq!(
                data_units_remaining, data_units_present,
                "删除中的那个号还剩的单元数与按单元号记的个数相同（unlink 的批两样一起减）"
            ),
        }
        data_units_present
    }

    /// size 罩几个数据单元（⌈size ÷ 净荷⌉.max(1)，[`data_units_of_content`]）。
    #[must_use]
    pub fn data_units_within_the_size(&self) -> u64 {
        data_units_of_content(u64::try_from(self.content.len()).expect("内容长度"))
    }

    /// 存在的单元 `index` 的声明长度：min(size − 起点, 净荷)（同 [`ModelFileVersion`] 那一份：带洞的文件存在的单元都是满的，截断途中 size 之外
    /// 还挂着的不问）。
    fn declared_length_of_the_data_unit(&self, index: u64) -> u64 {
        let payload = data_unit_payload_capacity_in_bytes();
        u64::try_from(self.content.len())
            .expect("内容长度")
            .saturating_sub(index * payload)
            .min(payload)
    }

    /// 上段里有它的条目：有数据单元，或粘住的标签 0（[`ModelInodeOtherThanTheFirst::keeps_a_tag_zero_upper_leaf_entry`]）。
    #[must_use]
    pub fn has_an_extent_upper_leaf_entry(&self) -> bool {
        self.data_units_present() > 0 || self.keeps_a_tag_zero_upper_leaf_entry
    }

    /// 活着（nlink 不是 0）：读得出、按 inode 号写与 unlink 点得到它。
    #[must_use]
    pub fn is_live(&self) -> bool {
        match self.liveness {
            ModelInodeLiveness::Live => true,
            ModelInodeLiveness::BeingDeleted { .. } => false,
        }
    }
}

/// extent 树上段一个节点的位置（D8（核心索引结构） 已定项 14：按 inode 号按位置寻址、不分裂，叶罩 143 个号、内部扇出 147；层级 0 是叶，
/// 同层从 inode 0 起数第几个）。模型自己一份，不引实现的类型（D13（验证路线） 已定项 5）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModelExtentUpperNodePosition {
    pub level: u64,
    pub index_in_level: u64,
}

/// 上段第 `level` 层一个节点罩几个 inode 号：143 × 147^level（饱和在 u64 的上界）。
fn extent_upper_span_in_inodes(level: u64) -> u64 {
    (0..level).fold(EXTENT_TREE_UPPER_LEAF_INODES, |span, _| {
        span.saturating_mul(EXTENT_TREE_INTERNAL_FANOUT)
    })
}

impl ModelExtentUpperNodePosition {
    /// 这个位置罩不罩 inode 号 `inode`。
    fn covers(self, inode: u64) -> bool {
        inode / extent_upper_span_in_inodes(self.level) == self.index_in_level
    }
}

/// 上段的规范形（判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M2）：根层级是罩得住有条目最大号的最低一层
/// （[`extent_upper_segment_root_level`]），节点是每个有条目的号从它那片叶到根那一串的并；集合空时只有层级 0 第 0 格那个零条目根兼叶。
/// 模型按条款自己算一份，不调实现的计划（D13（验证路线） 已定项 5）。
#[must_use]
pub fn extent_upper_segment_positions(
    inodes_with_an_entry: &BTreeSet<u64>,
) -> BTreeSet<ModelExtentUpperNodePosition> {
    let root_level = extent_upper_segment_root_level(inodes_with_an_entry);
    if inodes_with_an_entry.is_empty() {
        return BTreeSet::from([ModelExtentUpperNodePosition {
            level: 0,
            index_in_level: 0,
        }]);
    }
    inodes_with_an_entry
        .iter()
        .flat_map(|inode| {
            (0..=root_level).map(move |level| ModelExtentUpperNodePosition {
                level,
                index_in_level: inode / extent_upper_span_in_inodes(level),
            })
        })
        .collect()
}

/// 一次发布之后上段每个节点的分配代（规范形计划，判决第三节 M2「一次发布里进出 S 或条目变了的号那一串上留在新形状里的节点与新出现的节点重写，
/// 别的照抄」）：新形状里的一个位置，上一版没有它、或它罩着 `inodes_whose_entry_changed` 里的一个号，分配代取这次的 `checkpoint_txg`；
/// 别的照抄上一版的分配代。长高那次旧根就这样照抄成根之下的节点，降层那次上一版里降后层级的第 0 个节点照抄成根。
#[must_use]
pub fn extent_upper_node_written_at_after(
    previous: &BTreeMap<ModelExtentUpperNodePosition, ModelCheckpointTxg>,
    inodes_with_an_entry_after: &BTreeSet<u64>,
    inodes_whose_entry_changed: &BTreeSet<u64>,
    checkpoint_txg: ModelCheckpointTxg,
) -> BTreeMap<ModelExtentUpperNodePosition, ModelCheckpointTxg> {
    extent_upper_segment_positions(inodes_with_an_entry_after)
        .into_iter()
        .map(|position| {
            let covers_a_changed_inode = inodes_whose_entry_changed
                .iter()
                .any(|inode| position.covers(*inode));
            let written_at = match previous.get(&position) {
                Some(previous_written_at) if !covers_a_changed_inode => *previous_written_at,
                Some(_) | None => checkpoint_txg,
            };
            (position, written_at)
        })
        .collect()
}

/// extent 树上段的根在第几层（D8（核心索引结构） 已定项 14：叶罩 143 个 inode 号、内部扇出 147，按位置寻址、不分裂；层级是罩得住有条目的
/// 最大 inode 号的最低一层，只有叶兼根是 0 层，一条条目都没有的也是 0 层）：按 inode 号写长高、删除降层那两形看它变没变。
/// `inodes_with_an_entry`：这一版有数据单元的文件的 inode 号（第一个文件有单元时连 inode 1）。
#[must_use]
pub fn extent_upper_segment_root_level(inodes_with_an_entry: &BTreeSet<u64>) -> u64 {
    let Some(largest_inode_with_an_entry) = inodes_with_an_entry.last().copied() else {
        return 0;
    };
    let mut level = 0;
    let mut span_in_inodes = EXTENT_TREE_UPPER_LEAF_INODES;
    // 迭代上界：每一层的罩宽乘 147，罩宽超过最大的号时停（u64 里至多十来层）；跨轮带的是这一层的罩宽与层级。
    while span_in_inodes <= largest_inode_with_an_entry {
        level += 1;
        span_in_inodes = span_in_inodes.saturating_mul(EXTENT_TREE_INTERNAL_FANOUT);
    }
    level
}

/// 一版里有什么（D8（核心索引结构） 已定项 16 第 4、5 条：树表 0 条的、带文件的、有树而文件集合为空的；第一个文件之外的 inode 另记在
/// [`ModelRoot::inodes_other_than_the_first`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModelVersionContents {
    /// 树表 0 条（第 0 代根、暖机根、零单元发布）：没有树、没有文件。
    WithoutTrees,
    /// inode 1 是活记录：读回它的内容。
    LiveFile(ModelFileVersion),
    /// unlink 的意图那次之后、最后一批之前：inode 1 的记录 nlink = 0、号在 `deleted_inodes` 里，读回「没有文件」，它的单元还在。
    /// `trees_written_by`：写出这一版 inode 树、extent 树根指针的那次发布（意图那次或上一个不是最后一批的删除批；回退到这一版的根照抄它）。
    /// `data_units_remaining`：这一版里这个文件还剩几个数据单元（意图那次 = 文件的单元数，每个不是最后一批的删除批少 144 个）：
    /// 下一步是不是最后一批由它定（D8（核心索引结构） 已定项 16 第 2 条）。
    /// `truncation_field`：删除中那条记录里的截断字段，unlink 的意图与各批照抄 unlink 之前那一版的（判决
    /// `research/prompts/m4-trunc-r2-main-verification.md` 第三节 R2）。
    FileBeingDeleted {
        trees_written_by: ModelRootKey,
        data_units_remaining: u64,
        truncation_field: ModelTruncationField,
    },
    /// 最后一批之后：有树而文件集合为空（`deleted_inodes` 空、inode 树与 extent 树零条目根），读回「没有文件」。
    TreesAndNoFile { trees_written_by: ModelRootKey },
    /// 第一个文件已删完而 inode 树里还有别的记录（删完之后建过 inode，或删第一个文件时树里还有别的 inode，判决
    /// `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M3）：没有 inode 1，读回「没有文件」；别的文件记在
    /// [`ModelRoot::inodes_other_than_the_first`]（可以带数据、可以删除中）。实现把它归带文件的一版（inode 树有条目，D8（核心索引结构） 已定项 16
    /// 第 5 条的判定不成立）。
    /// `trees_written_by`：最近一次写这一版 inode 树或 extent 树根指针的那次发布（回退到这一版的根照抄它）。
    /// `inode_tree_written_by`：最近一次写 inode 树的那次发布：别的 inode 截断的批与放了单元的最后一批只写 extent 树（K2「批不碰 inode 记录」，判决
    /// m4-multi-inode-size-r1 第二节 S3），这时与 `trees_written_by` 不同；建快照抄的码 14 根指针是它写的。
    /// `inode_records_in_the_tree`：这一版 inode 树里有几条记录（就是第一个文件之外的 inode 数）：下一个建的落在哪一片叶容器由它定
    /// （[`inode_leaf_container_position_of_a_new_record`]）。
    InodesWithoutTheFirstFile {
        trees_written_by: ModelRootKey,
        inode_tree_written_by: ModelRootKey,
        inode_records_in_the_tree: u64,
    },
    /// 截断的意图那次之后、最后一批之前（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K2「先落目标」）：inode 1 的记录 size 已是
    /// 目标 N，读回 N 字节（`file.content`）；logged_ops 里有一条截断条目（签发 txg 是 `truncation_issued_txg`，目标 size 是 `file.content` 的长度）；
    /// size 之外还挂着 `file.data_units_present` − ⌈N ÷ 净荷⌉.max(1) 个没放的单元，下一步是不是最后一批由它定。
    FileBeingTruncated {
        file: ModelFileVersion,
        truncation_issued_txg: ModelCheckpointTxg,
    },
}

impl ModelVersionContents {
    /// 树表不是 0 条。
    #[must_use]
    pub fn has_trees(&self) -> bool {
        match self {
            ModelVersionContents::WithoutTrees => false,
            ModelVersionContents::LiveFile(_)
            | ModelVersionContents::FileBeingDeleted { .. }
            | ModelVersionContents::TreesAndNoFile { .. }
            | ModelVersionContents::InodesWithoutTheFirstFile { .. }
            | ModelVersionContents::FileBeingTruncated { .. } => true,
        }
    }

    /// 读得出的那个文件（inode 1 是活记录时；截断途中读回目标那 N 字节，K2）。
    #[must_use]
    pub fn live_file(&self) -> Option<&ModelFileVersion> {
        match self {
            ModelVersionContents::LiveFile(file)
            | ModelVersionContents::FileBeingTruncated { file, .. } => Some(file),
            ModelVersionContents::WithoutTrees
            | ModelVersionContents::FileBeingDeleted { .. }
            | ModelVersionContents::TreesAndNoFile { .. }
            | ModelVersionContents::InodesWithoutTheFirstFile { .. } => None,
        }
    }

    /// 这一版盘上存在的数据单元数：带文件的与截断途中的取文件记的（截断途中连 size 之外还挂着的），删除中的取还剩的，别的没有数据单元。
    #[must_use]
    pub fn data_units_present(&self) -> u64 {
        match self {
            ModelVersionContents::LiveFile(file)
            | ModelVersionContents::FileBeingTruncated { file, .. } => file.data_units_present,
            ModelVersionContents::FileBeingDeleted {
                data_units_remaining,
                ..
            } => *data_units_remaining,
            ModelVersionContents::WithoutTrees
            | ModelVersionContents::TreesAndNoFile { .. }
            | ModelVersionContents::InodesWithoutTheFirstFile { .. } => 0,
        }
    }

    /// 这一版的用户可见树（inode 树、extent 树）的根指针是谁写的：树表 0 条时 None。树表里这两棵树的根指针由写出它们的那次发布唯一定
    /// （指针带诞生代号），「两条根的这两棵树的根指针相同」⟺「是同一次发布写的」（D16（发布语义） 已定项 1「非空」从盘上怎么认）。
    fn user_visible_trees_written_by(&self) -> Option<ModelRootKey> {
        match self {
            ModelVersionContents::WithoutTrees => None,
            ModelVersionContents::LiveFile(file)
            | ModelVersionContents::FileBeingTruncated { file, .. } => Some(file.written_by),
            ModelVersionContents::FileBeingDeleted {
                trees_written_by, ..
            }
            | ModelVersionContents::TreesAndNoFile { trees_written_by }
            | ModelVersionContents::InodesWithoutTheFirstFile {
                trees_written_by, ..
            } => Some(*trees_written_by),
        }
    }

    /// 这一版 inode 树的根是谁写的（建快照抄进码 14 的那条根指针）：树表 0 条时 None。截断的批与最后一批只写 extent 树（K2），
    /// 这时与 [`ModelVersionContents::user_visible_trees_written_by`] 不同；删除的各步与建 inode 都写 inode 树，两者相同。
    fn inode_tree_written_by(&self) -> Option<ModelRootKey> {
        match self {
            ModelVersionContents::LiveFile(file)
            | ModelVersionContents::FileBeingTruncated { file, .. } => {
                Some(file.inode_record_written_by)
            }
            ModelVersionContents::InodesWithoutTheFirstFile {
                inode_tree_written_by,
                ..
            } => Some(*inode_tree_written_by),
            ModelVersionContents::WithoutTrees
            | ModelVersionContents::FileBeingDeleted { .. }
            | ModelVersionContents::TreesAndNoFile { .. } => self.user_visible_trees_written_by(),
        }
    }

    /// 这一版第一个文件那条 inode 记录里的截断字段：带活文件的、截断途中的与删除中的那几版有那条记录；树表 0 条的、删完的与删完之后建过 inode 的
    /// 没有那条记录，None。
    #[must_use]
    pub fn first_file_record_truncation_field(&self) -> Option<ModelTruncationField> {
        match self {
            ModelVersionContents::LiveFile(file)
            | ModelVersionContents::FileBeingTruncated { file, .. } => Some(file.truncation_field),
            ModelVersionContents::FileBeingDeleted {
                truncation_field, ..
            } => Some(*truncation_field),
            ModelVersionContents::WithoutTrees
            | ModelVersionContents::TreesAndNoFile { .. }
            | ModelVersionContents::InodesWithoutTheFirstFile { .. } => None,
        }
    }

    /// 截断途中这一版 logged_ops 里那条截断条目的签发 txg（K1：key = 种类 2 + 签发 txg）；不在截断途中为 None。
    #[must_use]
    pub fn logged_truncation_issued_txg(&self) -> Option<ModelCheckpointTxg> {
        match self {
            ModelVersionContents::FileBeingTruncated {
                truncation_issued_txg,
                ..
            } => Some(*truncation_issued_txg),
            ModelVersionContents::WithoutTrees
            | ModelVersionContents::LiveFile(_)
            | ModelVersionContents::FileBeingDeleted { .. }
            | ModelVersionContents::TreesAndNoFile { .. }
            | ModelVersionContents::InodesWithoutTheFirstFile { .. } => None,
        }
    }

    /// 删完第一个文件之后建的 inode 在这一版 inode 树里有几条：有树而文件集合为空的一版 0 条；
    /// 别的几种不是第一个文件已删完的一版，None（建 inode 只接在这两种后面）。
    fn inode_records_created_after_the_first_file_was_deleted(&self) -> Option<u64> {
        match self {
            ModelVersionContents::TreesAndNoFile { .. } => Some(0),
            ModelVersionContents::InodesWithoutTheFirstFile {
                inode_records_in_the_tree,
                ..
            } => Some(*inode_records_in_the_tree),
            ModelVersionContents::WithoutTrees
            | ModelVersionContents::LiveFile(_)
            | ModelVersionContents::FileBeingDeleted { .. }
            | ModelVersionContents::FileBeingTruncated { .. } => None,
        }
    }

    /// 有树、文件集合不空的那一版（带活文件的、删除中的、截断途中的、删完之后建过 inode 的）：可写会话里覆盖写、回退、抬 F 的执行器只在实现报「带文件」时调，
    /// 实现把删除中的那一版（`deleted_inodes` 不空）与只有 size 0 的 inode 的那一版（inode 树有条目）也归带文件。
    fn is_a_version_with_a_file_or_a_file_being_deleted(&self) -> bool {
        match self {
            ModelVersionContents::LiveFile(_)
            | ModelVersionContents::FileBeingDeleted { .. }
            | ModelVersionContents::InodesWithoutTheFirstFile { .. }
            | ModelVersionContents::FileBeingTruncated { .. } => true,
            ModelVersionContents::WithoutTrees | ModelVersionContents::TreesAndNoFile { .. } => {
                false
            }
        }
    }
}

/// 实例表的一行 (i, T, W)（D18（块里携带什么信息） 已定项 11）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModelInstanceRow {
    pub instance: ModelInstanceGeneration,
    pub selected_root_txg: ModelCheckpointTxg,
    pub applied_transaction_high_water: u64,
}

/// 一条已提交的根与它指着的那一版。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelRoot {
    pub key: ModelRootKey,
    pub journal_counter: ModelJournalCounter,
    pub rollback_floor: ModelCheckpointTxg,
    /// 这一版里有什么：树表 0 条、带文件、删除中、有树而文件集合为空。
    pub contents: ModelVersionContents,
    pub instance_table_rows: Rc<Vec<ModelInstanceRow>>,
    /// 这一版每个角色的单元是哪次发布写的：它的分配记录的分配代就是这个 txg（D3（空间分配） 已定项 3 / 7：value = 分配代；
    /// COW：单元只在分配它的那次发布里写）。mkfs 写的实例表与第 0 版树表记 txg 0。数据单元不在这里，按单元号记在 `data_unit_written_at`。
    pub role_written_at: BTreeMap<ModelUnitRole, ModelCheckpointTxg>,
    /// 这一版每个存在的数据单元是哪次发布写的，按单元号（第 i 个是单元 i，长度 = [`ModelVersionContents::data_units_present`]）：
    /// 分配代与映射 key 里的出生 txg 都是它（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K4：截断只重写尾单元之后
    /// 一个文件的数据单元不再同一个出生 txg，换下时各按各的判快照生死）。
    pub data_unit_written_at: Rc<Vec<ModelCheckpointTxg>>,
    /// 每盘已占槽数的上界：沿来路每次发布写出的单元的槽数之和，释放与回收一概不扣（分配记录树每次重写的节点数取按位置寻址现算的上界）。
    pub occupied_slots_upper_bound_per_device: u64,
    /// 这一版中央映射树节点数的上界（[`central_mapping_tree_nodes_upper_bound`] 沿来路带下来，每盘一个节点占一个槽）；
    /// 树表 0 条的一版没有这棵树，记 0。
    pub central_mapping_tree_nodes_upper_bound: u64,
    /// 这一版记账里 inode 号水位那一统计量的台阶行有几行（D5（快照 / 空间记账机制） 已定项 2 射程「inode 号水位的台阶行」）：
    /// 水位变了的每次发布留一级，之后每次写记账行的发布照抄（管理员回退那次照抄现行那一版的，不取回退目标的）。
    /// 模型不记水位的值，按发布的种类认水位变没变（[`ModelPublishKind::leaves_an_inode_number_watermark_step`]）。
    /// 没写过记账树的一版记 0。
    pub inode_number_watermark_steps: u64,
    /// 这一版记账树的行数：写它的那次发布写的池级三行、每块盘六行与照抄的台阶行（[`IdealModel::accounting_rows_of_a_publish_on`]）；
    /// 没写过记账树的一版记 0。
    pub accounting_rows: u64,
    /// 这一版记账树节点数的上界（[`accounting_tree_nodes_upper_bound`] 沿来路带下来，每盘一个节点占一个槽）：台阶行多到一片叶装不下时
    /// 记账树分裂、长出根之下的节点（D8（核心索引结构） 已定项 11）。根之下的节点不对应角色，模型不记 key、不记是哪几个（同中央映射树）；
    /// 写记账树的发布按它带占槽、按它数进映射条目。没写过记账树的一版记 0。
    pub accounting_tree_nodes_upper_bound: u64,
    /// 写出这条根的那次发布的记录里最大的事务号：写文件内容的发布（第一个文件、覆盖写、顺序写）一事务一单元（D16（发布语义） 已定项 5）
    /// ⇒ 内容切成几个数据单元就承载几个事务、接着实例内计数往下取，这里记最后那一个号；删除类发布与建 inode 各一个事务；
    /// 空发布、写行、暖机、抬 F、回退的记录事务号 0（D23（journal 的角色与格式） 已定项 19 ①），这里记 0。
    /// 恢复施加这次发布的记录时，W 按它取（D23（journal 的角色与格式） 已定项 14 第 4 条；空发布的 0 不进 max）。
    pub highest_transaction_number_of_its_publish: u64,
    /// 这个实例到这条根为止发过的最大事务号：事务号按实例计数、从 1 起（D23（journal 的角色与格式） 已定项 7），
    /// 换了实例（写行那次发布）从 0 重新数。
    pub highest_transaction_number_in_its_instance: u64,
    /// 这一版树表里的只读快照，按（诞生 txg, 树号）升序——就是建的先后（判决 `research/prompts/m4-snapshot-r1-main-verification.md`
    /// 第三节 S1、S2：每次建快照是一次独立发布，txg 各加一）。树表 0 条的一版一个都没有。
    pub snapshots: Rc<Vec<ModelSnapshot>>,
    /// 活头 inode 树那条条目的 `previous_snapshot_txg`（D5（快照 / 空间记账机制） 已定项 9：最近一个快照的 S.txg，还没打过快照为 0；
    /// 删最后一个快照的意图那次换成被删那一个的前驱，判决 S4）。树表 0 条的一版没有那条条目，记 0、不比。
    pub previous_snapshot_txg_of_the_live_head: ModelCheckpointTxg,
    /// 这一版的 deadlist 条目（判决 S3），次序不论：换下的用户可见单元出生 txg 不晚于活头前驱的进这里、不释放。
    pub deadlist: Rc<Vec<ModelDeadlistEntry>>,
    /// 这一版留在中央映射树里、被活快照钉住的映射条目条数（判决 S3 的 α）：每钉住一个单元留一条，删快照的批放掉一条删一条。
    /// 与 deadlist 条目数各按各的来路算（条款说两者一一对应，模型两样都比）。
    pub pinned_mapping_entries: u64,
    /// 这一版 deadlist 树节点数的上界（[`deadlist_tree_nodes_upper_bound`] 沿来路带下来；零根时 0）。
    pub deadlist_tree_nodes_upper_bound: u64,
    /// 这一版第一个文件 extent 树下段每个节点的分配代（换下时按它判钉不钉，判决 S3）：写内容那次整段重写，不是最后一批的删除批只重写
    /// 罩着被截掉那一段的节点（K-R3-1）、整个落在留下那一段里的照抄，最后一批整段换下。模型罩的版本里只有第一个文件有下段。
    pub extent_lower_node_written_at:
        Rc<BTreeMap<ModelExtentLowerNodePosition, ModelCheckpointTxg>>,
    /// 这一版 logged_ops 里那一条活整理意图（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1：全池至多一条）；没有为 `None`。
    /// 签发那次写进、之后每次发布照抄（管理员回退随现行那一版，丁 3 交回的读法第 9 条）、删意图那次没了；崩溃恢复与可写挂载落到哪一版就是那一版的。
    pub logged_compaction_intent: Option<ModelCompactionIntent>,
    /// 这一版引用着的用户可见树的根单元（[`ModelRoot::user_visible_root_units`]）里被整理搬过的，最后一次搬它的那次发布的 txg：树表里指着它的
    /// 根指针从那次起写的是映射位置，盘上字节与搬之前不同（用户 2026-10-15 定「照字节比，搬根算非空」）。只记这一版还引用着的单元。
    pub user_visible_root_relocations: Rc<BTreeMap<ModelUserVisibleRootUnit, ModelCheckpointTxg>>,
    /// 这一版 inode 树里第一个文件之外的每个活着的 inode，按 inode 号（建 inode 那次建、按 inode 号写数据那次整份写，
    /// [`ModelInodeOtherThanTheFirst`]）：随用户可见树走——别的发布照抄，管理员回退取回退目标那一版的。树表 0 条的一版是空的。
    pub inodes_other_than_the_first: Rc<BTreeMap<u64, ModelInodeOtherThanTheFirst>>,
    /// 这一版的 inode 号水位：下一个建的 inode 取的号（D5（快照 / 空间记账机制） 已定项 8 池级三行之一）。没写过记账树的一版 0；第一个文件那次
    /// 发掉 inode 1、水位 2；建 inode 每建一个加一；管理员回退那次取现行那一版的（[`ModelPublishKind::leaves_an_inode_number_watermark_step`]
    /// 那一段：环里没被崩溃恢复抛弃的根的最大值，模型罩的历史里就是现行那一版的），不取回退目标的；别的发布照抄。
    pub inode_number_watermark: u64,
    /// 这一版父链上 inode 1 那条整对象墓碑的死亡 txg（D18（块里携带什么信息） 已定项 24：死亡 txg = 意图那次发布的 txg）：unlink 第一个文件的
    /// 意图那次写成这次的 txg，别的发布照抄；管理员回退那次取回退目标那一版的——回退那次的父是回退目标（谱系记录读作「T_rb 的父是 T_old」），
    /// 现行那一版在回退目标之后删第一个文件留下的墓碑虽然随现行那一版留在墓碑树里，却在被回退抛下的那一段上，不管这一版（代码轮判决
    /// `research/prompts/m3-close-w5-code-r1-main-verification.md` 第三节第 1 条：β 只认这一版父链上的墓碑）。模型不记墓碑树与谱系树的条目，
    /// 照「一条父链上第一个文件至多删一次」直接带着这一条走。父链上没删过第一个文件为 `None`。
    /// K-U5 的 β 时序级读法拿它与目标的对象出生代比（用户 2026-10-19 定）。
    pub death_txg_of_the_first_file: Option<ModelCheckpointTxg>,
}

impl ModelRoot {
    /// 这一版 inode 树里有几条记录：第一个文件那条（活着、删除中、截断途中都在）加第一个文件之外的每个 inode。建 inode 的新记录落在哪一片叶容器
    /// 由它定（[`inode_leaf_container_position_of_a_new_record`]）。
    fn inode_records_in_the_tree(&self) -> u64 {
        let first_file_record = match self.contents {
            ModelVersionContents::LiveFile(_)
            | ModelVersionContents::FileBeingDeleted { .. }
            | ModelVersionContents::FileBeingTruncated { .. } => 1,
            ModelVersionContents::WithoutTrees
            | ModelVersionContents::TreesAndNoFile { .. }
            | ModelVersionContents::InodesWithoutTheFirstFile { .. } => 0,
        };
        first_file_record + u64::try_from(self.inodes_other_than_the_first.len()).expect("条数")
    }

    /// 这一版 extent 树上段里有条目的 inode 号（[`inodes_with_an_extent_upper_leaf_entry`]）。
    #[must_use]
    pub fn inodes_with_an_extent_upper_leaf_entry(&self) -> BTreeSet<u64> {
        inodes_with_an_extent_upper_leaf_entry(&self.contents, &self.inodes_other_than_the_first)
    }

    /// 这一版 logged_ops 里那条截断条目的签发 txg（K1：全池至多一条）：截断途中的第一个文件，或截断条目点名的第一个文件之外的 inode
    /// （判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S3）；不在截断途中为 `None`。
    #[must_use]
    pub fn logged_truncation_issued_txg(&self) -> Option<ModelCheckpointTxg> {
        self.contents.logged_truncation_issued_txg().or_else(|| {
            self.inodes_other_than_the_first
                .values()
                .find_map(|inode| inode.truncation_issued_txg)
        })
    }

    /// 这一版第一个文件之外有没有文件带数据单元：快照、截断、整理那几臂在这一形上照答（判决
    /// `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S5），计数按它分出「别的文件带数据时」那几格。
    #[must_use]
    pub fn has_a_file_other_than_the_first_holding_data_units(&self) -> bool {
        self.inodes_other_than_the_first
            .values()
            .any(|inode| inode.data_units_present() > 0)
    }

    /// 这一版删除中的那个号（`deleted_inodes` 至多一条 key，D8（核心索引结构） 已定项 16 第 6 条）与它还剩几个数据单元；没有删除中的为 `None`。
    fn inode_being_deleted(&self) -> Option<(ModelDeletedInode, u64)> {
        if let ModelVersionContents::FileBeingDeleted {
            data_units_remaining,
            ..
        } = self.contents
        {
            return Some((ModelDeletedInode::FirstFile, data_units_remaining));
        }
        self.inodes_other_than_the_first
            .iter()
            .find_map(|(number, inode)| match inode.liveness {
                ModelInodeLiveness::BeingDeleted {
                    data_units_remaining,
                } => Some((
                    ModelDeletedInode::OtherThanTheFirst(*number),
                    data_units_remaining,
                )),
                ModelInodeLiveness::Live => None,
            })
    }

    /// 这一版 extent 树上段每个节点的分配代：根（罩得住有条目最大号的最低一层的第 0 个）取 [`ModelUnitRole::ExtentRoot`] 的，根之下的取
    /// [`ModelUnitRole::ExtentUpperNodeBelowTheRoot`] 那几个的。树表 0 条的一版没有上段，空的。
    fn extent_upper_node_written_at(
        &self,
    ) -> BTreeMap<ModelExtentUpperNodePosition, ModelCheckpointTxg> {
        Self::extent_upper_node_written_at_of(
            &self.role_written_at,
            &self.inodes_with_an_extent_upper_leaf_entry(),
        )
    }

    /// 按角色记的分配代（`role_written_at`）与这一版有条目的号（`inodes_with_an_entry`）拼出 extent 树上段每个节点的分配代
    /// （[`ModelRoot::extent_upper_node_written_at`]）：发布写新根时那一版的 [`ModelRoot`] 还没拼好，用这一份。
    fn extent_upper_node_written_at_of(
        role_written_at: &BTreeMap<ModelUnitRole, ModelCheckpointTxg>,
        inodes_with_an_entry: &BTreeSet<u64>,
    ) -> BTreeMap<ModelExtentUpperNodePosition, ModelCheckpointTxg> {
        let root_level = extent_upper_segment_root_level(inodes_with_an_entry);
        role_written_at
            .iter()
            .filter_map(|(role, written_at)| match role {
                ModelUnitRole::ExtentRoot => Some((
                    ModelExtentUpperNodePosition {
                        level: root_level,
                        index_in_level: 0,
                    },
                    *written_at,
                )),
                ModelUnitRole::ExtentUpperNodeBelowTheRoot(position) => {
                    Some((*position, *written_at))
                }
                ModelUnitRole::Data
                | ModelUnitRole::InodeLeaf
                | ModelUnitRole::InodeLeafRightOfTheLeftmost(_)
                | ModelUnitRole::InodeRoot
                | ModelUnitRole::AllocationTree
                | ModelUnitRole::AccountingTree
                | ModelUnitRole::MappingTree
                | ModelUnitRole::TreeTable
                | ModelUnitRole::InstanceTable
                | ModelUnitRole::DeletedInodesRoot
                | ModelUnitRole::LoggedOperationsRoot
                | ModelUnitRole::TombstoneLeaf
                | ModelUnitRole::TombstoneRoot
                | ModelUnitRole::RetirementLeaf
                | ModelUnitRole::RetirementRoot => None,
            })
            .collect()
    }
}

/// 第一个文件的 inode 号（D8（核心索引结构） 已定项 16：第一版的文件是 inode 1）。模型自己写一份，不引实现的常量（D13（验证路线） 已定项 5）。
const FIRST_INODE_NUMBER: u64 = 1;

/// 一版 extent 树上段里有条目的 inode 号：第一个文件有数据单元时（活着、截断途中、删除中还剩单元）连 inode 1，加第一个文件之外有数据单元的
/// 每个文件（建出来没写过数据的没有条目）。
fn inodes_with_an_extent_upper_leaf_entry(
    contents: &ModelVersionContents,
    inodes_other_than_the_first: &BTreeMap<u64, ModelInodeOtherThanTheFirst>,
) -> BTreeSet<u64> {
    let first_file = (contents.data_units_present() > 0).then_some(FIRST_INODE_NUMBER);
    first_file
        .into_iter()
        .chain(
            inodes_other_than_the_first
                .iter()
                .filter(|(_, inode)| inode.has_an_extent_upper_leaf_entry())
                .map(|(number, _)| *number),
        )
        .collect()
}

/// 一版的中央映射树里第一个文件之外的文件那几条条目（D19（块指针的结构与宽度预算） 已定项 8：每个数据单元、extent 树每个节点各一条）：
/// 每个有数据单元的文件，单元数加它下段的节点数（[`extent_lower_segment_nodes`]）。
fn central_mapping_entries_of_the_files_other_than_the_first(
    inodes_other_than_the_first: &BTreeMap<u64, ModelInodeOtherThanTheFirst>,
) -> u64 {
    inodes_other_than_the_first
        .values()
        .map(|inode| {
            let data_units = inode.data_units_present();
            data_units + extent_lower_segment_nodes(data_units)
        })
        .sum()
}

/// 一个整理区域 R（判决 G1：一个聚簇段，起始槽 64 对齐、槽数 64；两盘同槽时罩住每块盘上的同号槽，「R 按记录全空」按 `device` 那块盘的记录判）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModelCompactionRegion {
    pub device: ModelDeviceIdentity,
    pub start_slot: u64,
    pub slot_count: u64,
}

/// logged_ops 里的一条整理意图，模型记、比的那几样（判决 G1 的字段表）。`segs0` 记签发那次发布之前按记录口径的全空聚簇段数，停机谓词
/// 「全局差值」相对它比（判决 `research/prompts/m4-segment-code-r2-main-verification.md` 第三节 H3，用户 2026-10-16 定「取签发之前」）；
/// 模型不记分配记录、数不出它，这里不记、不比。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModelCompactionIntent {
    /// key 里的签发 txg：签发那次发布的 txg。
    pub issued_txg: ModelCheckpointTxg,
    pub region: ModelCompactionRegion,
    /// `c0`：签发那一刻的分配代，就是签发那次发布接在后面的那一版的 txg。
    pub allocation_generation_at_issue: ModelCheckpointTxg,
}

/// 用户可见的两种树（D16（发布语义） 已定项 1「非空」比的条目：活头与快照的 inode 树根、extent 树根）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModelUserVisibleTree {
    InodeTree,
    ExtentTree,
}

/// 用户可见树的一个根单元：哪一种树的根、出生 txg（写它的那次发布）。一次发布至多写一种树的一个根，这两样定一个单元；活头与快照条目抄的是
/// 同一个单元时是同一个值（搬它，两条根指针一起变）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModelUserVisibleRootUnit {
    pub tree: ModelUserVisibleTree,
    pub birth_txg: ModelCheckpointTxg,
}

impl ModelRoot {
    /// 这一版引用着的用户可见树的根单元，按「非空」比的次序：活头的 inode 树根、extent 树根（树表 0 条的一版没有），再是每个快照的两个
    /// （按建的先后）。活头的取这一版那个角色的分配代（= 出生 txg，D26（后台整理与放置回收） 已定项 5 第 2 条：搬迁不改它）。
    #[must_use]
    pub fn user_visible_root_units(&self) -> Vec<ModelUserVisibleRootUnit> {
        let live_head = [
            (ModelUserVisibleTree::InodeTree, ModelUnitRole::InodeRoot),
            (ModelUserVisibleTree::ExtentTree, ModelUnitRole::ExtentRoot),
        ]
        .into_iter()
        .filter_map(|(tree, role)| {
            self.role_written_at
                .get(&role)
                .map(|birth_txg| ModelUserVisibleRootUnit {
                    tree,
                    birth_txg: *birth_txg,
                })
        });
        let of_the_snapshots = self.snapshots.iter().flat_map(|snapshot| {
            [
                ModelUserVisibleRootUnit {
                    tree: ModelUserVisibleTree::InodeTree,
                    birth_txg: snapshot.inode_tree_written_by.checkpoint_txg,
                },
                ModelUserVisibleRootUnit {
                    tree: ModelUserVisibleTree::ExtentTree,
                    birth_txg: snapshot.extent_root_written_at,
                },
            ]
        });
        live_head.chain(of_the_snapshots).collect()
    }

    /// 这条根的用户可见树（inode 树、extent 树）的根指针是谁写的：树表 0 条时 None。
    fn user_visible_trees_written_by(&self) -> Option<ModelRootKey> {
        self.contents.user_visible_trees_written_by()
    }

    /// 「非空」与「4 个不同状态」比的那一样（`mount::user_visible_trees_changed`，判决 `research/prompts/m4-snapshot-r1-main-verification.md`
    /// 之后用户可见的树连同快照那几对条目的根指针）：inode 树、extent 树是谁写的，加树表里每个快照两条根指针抄的是谁，按表里的次序。
    /// 只比根指针，不比「销毁中」记号与前驱：删快照的意图那次与不是最后一批的批不改它，建快照与最后一批改它。
    /// 整理搬过的根单元（[`ModelRoot::user_visible_root_relocations`]）按最后一次搬它的 txg 进来：搬一次根指针就换一次映射位置，字节变了就是新状态
    /// （用户 2026-10-15 定「照字节比，搬根算非空」）。
    fn user_visible_state(&self) -> ModelUserVisibleState {
        ModelUserVisibleState {
            trees_written_by: self.user_visible_trees_written_by(),
            snapshot_roots: self
                .snapshots
                .iter()
                .map(|snapshot| {
                    (
                        snapshot.inode_tree_written_by,
                        snapshot.extent_root_written_at,
                    )
                })
                .collect(),
            root_relocations: self
                .user_visible_root_units()
                .iter()
                .map(|unit| self.user_visible_root_relocations.get(unit).copied())
                .collect(),
        }
    }
}

/// [`ModelRoot::user_visible_state`] 交回的那一样：两条根相等 ⟺ 它们在盘上的用户可见根指针（连同快照条目的根指针）逐字节相同。
#[derive(Clone, Debug, PartialEq, Eq)]
struct ModelUserVisibleState {
    trees_written_by: Option<ModelRootKey>,
    snapshot_roots: Vec<(ModelRootKey, ModelCheckpointTxg)>,
    /// [`ModelRoot::user_visible_root_units`] 逐个：最后一次搬它的 txg，没搬过为 `None`。
    root_relocations: Vec<Option<ModelCheckpointTxg>>,
}

/// 池的几何：盘的身份与每块盘的字节数（与实现的 mkfs 参数由调用方各给一份）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelPoolGeometry {
    pub devices: Vec<ModelDeviceIdentity>,
    pub device_size_in_bytes: u64,
}

/// 根环里的一个槽：第 n 次发布写区域 `n mod R` 的槽 `(n div R) mod S`（D22（单元原子性怎么合成） 已定项 2 / 已定项 16）。
/// 公开是为了让对拍拿实现报的那个槽（`MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`）与它比：
/// 两边各自声明（D13（验证路线） 已定项 5），胶水在 `model_comparison.rs` 里换过来。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModelRingPosition {
    pub region: u64,
    pub slot_in_region: u64,
}

/// 一个单元在一块盘上的那一份占的槽：从 `first_slot` 起 `span_in_slots` 个槽。公开是为了让对拍拿实现报的那一份
/// （`RollbackError::IsolatedUnitCopyUnreadableOrMismatched`、`RollbackError::ResurrectedUnitCopyUnreadableOrMismatched`）
/// 与被吞的写落在哪比（故障注入，`fault_injection::left_by_the_lying_device`）；胶水在 `model_comparison.rs` 里换过来。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModelUnitCopyLocation {
    pub device: ModelDeviceIdentity,
    pub span_in_slots: u32,
    pub first_slot: u64,
}

impl ModelUnitCopyLocation {
    /// 这一份在盘上占的字节区间 [起, 止)。
    ///
    /// # Panics
    /// 槽号乘槽宽越过 u64（盘上读来的槽号不会这么大，越过就是胶水换错了）。
    #[must_use]
    pub fn byte_range_on_the_device(self) -> std::ops::Range<u64> {
        let start = self
            .first_slot
            .checked_mul(SLOT_BYTES)
            .expect("槽号乘槽宽装得进 u64：槽号来自盘上的位置条目");
        let length = u64::from(self.span_in_slots)
            .checked_mul(SLOT_BYTES)
            .expect("一个单元两槽以内");
        start..start.checked_add(length).expect("单元末端装得进 u64")
    }
}

/// 模型默认建的池的 S：显式给 S 的建池入口（场景、随机历史、故障注入）取的那个值，[`IdealModel::after_make_filesystem`] 与
/// [`IdealModel::after_make_filesystem_on_a_journal_ring_of`] 照它建。按别的 S 建的池，模型经
/// [`IdealModel::after_make_filesystem_on_a_journal_ring_with_slots_per_region`] 记下那个 S，环几何跟着它算
/// （m3-close-c506 三轮判决第三节「harness 模型跟着 S 走」）；一个模型建好之后 S 不变（S 是系统不可变配置）。
const MODEL_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 =
    ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM;

/// 第 n 次发布落在区域 `n mod R` 的槽 `(n div R) mod S`，S 是这个模型建池时的那个（[`IdealModel`] 的 `slots_per_region`）。
fn ring_position_of(
    checkpoint_txg: ModelCheckpointTxg,
    slots_per_region: u64,
) -> ModelRingPosition {
    ModelRingPosition {
        region: checkpoint_txg.0 % ROOT_RING_REGIONS,
        slot_in_region: (checkpoint_txg.0 / ROOT_RING_REGIONS) % slots_per_region,
    }
}

/// 一条根落在哪块盘：区域的设备归属第一版写死 0 / 1 / 0（D2（RAID 条带策略） 已定项 7；D16（发布语义） 已定项 8）。
fn device_holding_the_root_of(checkpoint_txg: ModelCheckpointTxg) -> ModelDeviceIdentity {
    let region = usize::try_from(checkpoint_txg.0 % ROOT_RING_REGIONS).expect("区域号小于 3");
    ModelDeviceIdentity(ROOT_RING_REGION_DEVICES[region])
}

/// 码 2 索引节点含 key 区间与预留位的头宽：key 区间之外的 86、key 区间两个 key、预留位 29 三段相加
/// （D8（核心索引结构） 已定项 11；D18（块里携带什么信息） 已定项 16 / 已定项 18）。
///
/// 模型自己的一份式子，不调实现、也不调 checker 的那一份（D13（验证路线） 已定项 5：`singlefs-format` 只放标量；
/// 2026-09-27 用户定案「三方各算一份 + 交叉断言」）；三份在 key 宽的全部取值上连起来比的是
/// `crates/singlefs-harness/tests/index_node_header_width_computed_three_ways_agrees_for_every_key_width.rs`。
#[must_use]
pub fn index_node_header_bytes(key_width_in_bytes: u64) -> u64 {
    let key_range_bytes = key_width_in_bytes * INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT;
    INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE + key_range_bytes + NONCE_MAC_ALGORITHM_RESERVED_BYTES
}

/// 一个索引节点装几条定宽条目：(节点 − 头) ÷ 条目宽（D8（核心索引结构） 已定项 2 / 已定项 11）。
fn index_node_entry_capacity(key_width_in_bytes: u64, entry_width_in_bytes: u64) -> u64 {
    (NODE_BYTES - index_node_header_bytes(key_width_in_bytes)) / entry_width_in_bytes
}

/// 一个数据单元装得下多少字节的用户内容：32768 含头与预留位（D4（校验和位置） 已定项 5；D18（块里携带什么信息） 已定项 16），
/// 第一版没有扩展点声明值。
#[must_use]
pub fn data_unit_payload_capacity_in_bytes() -> u64 {
    DATA_UNIT_BYTES - DATA_UNIT_PAYLOAD_OFFSET
}

/// 一份内容切成几个数据单元（D16（发布语义） 已定项 5 的切分纪律：一事务一单元，按净荷容量切）：长度除净荷容量向上取整，
/// 长度 0 也是一个单元（声明长度 0 的数据单元）。模型自己的一份式子，不调实现的切分（D13（验证路线） 已定项 5）。
#[must_use]
pub fn data_units_of_content(content_length_in_bytes: u64) -> u64 {
    content_length_in_bytes
        .div_ceil(data_unit_payload_capacity_in_bytes())
        .max(1)
}

/// 一个文件有 `data_units` 个数据单元时 extent 树下段有几个节点（D8（核心索引结构） 已定项 14：只有一个单元时不建下段、
/// 那个指针内联在上段叶条目里；下段按单元号按位置寻址，叶罩 144 个单元，内部节点扇出 147，往上数到只剩一个节点为止）。
#[must_use]
pub fn extent_lower_segment_nodes(data_units: u64) -> u64 {
    if data_units <= 1 {
        return 0;
    }
    let mut nodes_on_this_level = data_units.div_ceil(EXTENT_TREE_LOWER_LEAF_DATA_UNITS);
    let mut nodes = nodes_on_this_level;
    // 迭代上界：每一层的节点数除扇出 147 向上取整，严格变少直到 1。
    while nodes_on_this_level > 1 {
        nodes_on_this_level = nodes_on_this_level.div_ceil(EXTENT_TREE_INTERNAL_FANOUT);
        nodes += nodes_on_this_level;
    }
    nodes
}

/// 一个文件有 `data_units` 个数据单元时 extent 树下段有几层（同 [`extent_lower_segment_nodes`] 的数法；没有下段是 0 层）：
/// 不是最后一批的删除批只重写被截掉那一片下段叶与它的祖先，每层至多一个（K-R3-1），占槽的上界按截之前的层数取。
#[must_use]
pub fn extent_lower_segment_levels(data_units: u64) -> u64 {
    if data_units <= 1 {
        return 0;
    }
    let mut nodes_on_this_level = data_units.div_ceil(EXTENT_TREE_LOWER_LEAF_DATA_UNITS);
    let mut levels = 1;
    // 迭代上界同 `extent_lower_segment_nodes`。
    while nodes_on_this_level > 1 {
        nodes_on_this_level = nodes_on_this_level.div_ceil(EXTENT_TREE_INTERNAL_FANOUT);
        levels += 1;
    }
    levels
}

/// 中央映射树一片叶装几条映射条目：(节点 − 头) ÷ 映射条目宽（D8（核心索引结构） 已定项 11；条目 55，D19（块指针的结构与宽度预算） 已定项 6）。
fn central_mapping_tree_leaf_entry_capacity() -> u64 {
    index_node_entry_capacity(MAPPING_KEY_BYTES, MAPPING_ENTRY_BYTES)
}

/// 中央映射树一个内部节点装几个孩子：内部条目 = 本树 key + 子指针 86（D8（核心索引结构） 已定项 11）。
fn central_mapping_tree_internal_child_capacity() -> u64 {
    index_node_entry_capacity(MAPPING_KEY_BYTES, MAPPING_KEY_BYTES + NODE_POINTER_BYTES)
}

/// 一个装不下的节点从中间切开之后，切出来的右半至少几条：装不下时是容量 + 1 条，从中间切两半各至少 ⌊(容量 + 1) ÷ 2⌋ = ⌈容量 ÷ 2⌉
/// （D8（核心索引结构） 已定项 11 ②）。
fn smallest_half_of_a_split(capacity: u64) -> u64 {
    capacity.div_ceil(2)
}

/// 中央映射树一次发布之后至多几个节点（D8（核心索引结构） 已定项 11 ②③ 与「一次发布里先删后插」）。
///
/// 节点数不是这一版条目数的函数：叶装不下时从中间切，收缩只摘空节点、不合并，删得只剩几条的叶照样留着，形状随来路变
/// （同一组条目，来路上长过的树可以比一次建起来的多几个节点）。模型不记 key、不照抄实现的分裂（D13（验证路线） 已定项 5），
/// 沿来路带一个上界：
/// - 先删：只摘空节点、根只剩一个孩子时降高，节点数不增；
/// - 后插：设叶切了 s 次、内部节点切了 t 次。切出来的右半至少 ⌊(容量 + 1) ÷ 2⌋（叶 147 条、内部 72 个孩子，
///   [`smallest_half_of_a_split`]），之后这一段只插不删，到这次结束仍至少这么多、彼此不相交：147 s ≤ 这一版的条目数 E，
///   72 t ≤ 这次之后的节点数 N − 1（根之外每个节点恰是一个内部节点的孩子）；根切开再长一个新根，至多 s + t 次。
///
/// ⇒ N ≤ N_前 + 2 (s + t) ≤ N_前 + 2 ⌊E ÷ 147⌋ + 2 ⌊(N − 1) ÷ 72⌋。右边随 N 涨得比 N 慢，满足它的 N 有个最大值：
/// 从 2 (N_前 + 2 ⌊E ÷ 147⌋)（那里已不满足）往下迭代，每一步都仍是 N 的上界，停在不动点（同分配记录树那一份的取法）。
///
/// `nodes_before`：上一版这棵树节点数的上界；这一版从空树建起时取 1（实现的规划从一片空的根兼叶起插）。
///
/// # Panics
/// `nodes_before` 为 0：一棵有条目的树至少一个节点，调用方给的是上一版的上界或 1。
#[must_use]
pub fn central_mapping_tree_nodes_upper_bound(nodes_before: u64, entries: u64) -> u64 {
    assert!(
        nodes_before >= 1,
        "上一版的树至少一个节点（从空树建起也从一片空的根兼叶起插）"
    );
    let nodes_without_internal_splits = nodes_before
        + 2 * (entries / smallest_half_of_a_split(central_mapping_tree_leaf_entry_capacity()));
    let internal_split_bound = |nodes: u64| {
        nodes_without_internal_splits
            + 2 * ((nodes - 1)
                / smallest_half_of_a_split(central_mapping_tree_internal_child_capacity()))
    };
    let mut bound = 2 * nodes_without_internal_splits;
    // 迭代上界：bound 每一轮严格变小，不小于 1。跨轮带的是「这次之后的节点数不超过 bound」。
    loop {
        let next = internal_split_bound(bound);
        if next >= bound {
            return bound;
        }
        bound = next;
    }
}

/// 记账树一片叶装几行：(节点 − 头) ÷ 记账条目宽（D8（核心索引结构） 已定项 11；条目 34，D5（快照 / 空间记账机制） 已定项 5）。
fn accounting_tree_leaf_entry_capacity() -> u64 {
    index_node_entry_capacity(ACCOUNTING_KEY_BYTES, ACCOUNTING_ENTRY_BYTES)
}

/// 记账树一个内部节点装几个孩子：内部条目 = 本树 key + 子指针 86（D8（核心索引结构） 已定项 11）。
fn accounting_tree_internal_child_capacity() -> u64 {
    index_node_entry_capacity(
        ACCOUNTING_KEY_BYTES,
        ACCOUNTING_KEY_BYTES + NODE_POINTER_BYTES,
    )
}

/// 记账树一次写记账行的发布之后至多几个节点（D8（核心索引结构） 已定项 11 ②③ 与「一次发布里先删后插」）。
///
/// 节点数不是行数的函数（收缩只摘空节点、不合并，形状随来路变）；模型不记 key、不照抄实现的分裂（D13（验证路线） 已定项 5），
/// 同中央映射树沿来路带一个上界（[`central_mapping_tree_nodes_upper_bound`]），这里多数一半：
/// - 先删：只摘空节点、根只剩一个孩子时降高，节点数不增；
/// - 后插：设叶切了 s 次、内部节点切了 t 次、根切了 r 次（切开的根上面再长一个新根）。每切一次多一个节点，根切再多一个：
///   N ≤ N_前 + s + t + r。
/// - 切开的两半各至少 ⌊(容量 + 1) ÷ 2⌋（叶 239 行、内部 75 个孩子，[`smallest_half_of_a_split`]），之后这一段只插不删，到这次结束仍至少这么多。
///   每切一次，这次结束时「切出来的」叶至少多一片（切原有的叶多两片、切切出来的叶多一片），s ≥ 1 时至少 s + 1 片、彼此不相交：
///   (s + 1) × 239 ≤ 这次之后的行数 R。内部节点同一条：(t + 1) × 75 ≤ N − 1（根之外每个节点恰是一个内部节点的孩子）。
///   没有叶切就没有内部节点切（内部节点只在孩子切开时多一个孩子）：s = 0 ⇒ t = 0。
/// - 根切：根兼叶切开那一次是叶切，之后的根都是内部节点、它切开算在 t 里：r ≤ min(s + t, 1 + t)。
///
/// ⇒ 从一个节点长到 R = 478 行（两块盘 463 级台阶，D5 已定项 2 射程）：s ≤ 1、t = 0、r ≤ 1，N ≤ 3，恰是实现长成的根加两片叶。
/// 右边随 N 涨得比 N 慢，满足它的 N 有个最大值：从 2 (N_前 + 2 s + 1)（那里已不满足）往下迭代，每一步都仍是 N 的上界，停在不动点。
///
/// `nodes_before`：上一版这棵树节点数的上界；上一版没有记账树时取 1（实现的规划从一片空的根兼叶起插）。
///
/// # Panics
/// `nodes_before` 为 0：一棵有行的树至少一个节点，调用方给的是上一版的上界或 1。
#[must_use]
pub fn accounting_tree_nodes_upper_bound(nodes_before: u64, rows: u64) -> u64 {
    assert!(
        nodes_before >= 1,
        "上一版的树至少一个节点（从空树建起也从一片空的根兼叶起插）"
    );
    let leaf_splits_bound =
        (rows / smallest_half_of_a_split(accounting_tree_leaf_entry_capacity())).saturating_sub(1);
    let nodes_after_the_splits_bound = |nodes: u64| {
        let internal_splits_bound = if leaf_splits_bound == 0 {
            0
        } else {
            ((nodes - 1) / smallest_half_of_a_split(accounting_tree_internal_child_capacity()))
                .saturating_sub(1)
        };
        let root_splits_bound =
            (leaf_splits_bound + internal_splits_bound).min(1 + internal_splits_bound);
        nodes_before + leaf_splits_bound + internal_splits_bound + root_splits_bound
    };
    let mut bound = 2 * (nodes_before + 2 * leaf_splits_bound + 1);
    // 迭代上界：bound 每一轮严格变小，不小于 nodes_before。跨轮带的是「这次之后的节点数不超过 bound」。
    loop {
        let next = nodes_after_the_splits_bound(bound);
        if next >= bound {
            return bound;
        }
        bound = next;
    }
}

/// deadlist 树一片叶装几条条目：(节点 − 头) ÷ key 宽（value 空；判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3「叶 376」）。
fn deadlist_tree_leaf_entry_capacity() -> u64 {
    index_node_entry_capacity(DEADLIST_KEY_BYTES, DEADLIST_KEY_BYTES)
}

/// deadlist 树一个内部节点装几个孩子：内部条目 = 本树 key + 子指针 86（判决 S3「内部 125」）。
fn deadlist_tree_internal_child_capacity() -> u64 {
    index_node_entry_capacity(DEADLIST_KEY_BYTES, DEADLIST_KEY_BYTES + NODE_POINTER_BYTES)
}

/// deadlist 树一次发布之后至多几个节点：没有条目时是零根、一个节点都没有；有条目时同中央映射树那一份的取法
/// （[`central_mapping_tree_nodes_upper_bound`]：先删只摘空节点、后插从中间切，沿来路带一个上界），容量换成 deadlist 树的。
/// `nodes_before`：上一版这棵树节点数的上界（零根时 0，从一片空的根兼叶起插按 1 算）。
#[must_use]
pub fn deadlist_tree_nodes_upper_bound(nodes_before: u64, entries: u64) -> u64 {
    if entries == 0 {
        return 0;
    }
    let nodes_without_internal_splits = nodes_before.max(1)
        + 2 * (entries / smallest_half_of_a_split(deadlist_tree_leaf_entry_capacity()));
    let internal_split_bound = |nodes: u64| {
        nodes_without_internal_splits
            + 2 * ((nodes - 1) / smallest_half_of_a_split(deadlist_tree_internal_child_capacity()))
    };
    let mut bound = 2 * nodes_without_internal_splits;
    // 迭代上界：bound 每一轮严格变小，不小于 1。跨轮带的是「这次之后的节点数不超过 bound」。
    loop {
        let next = internal_split_bound(bound);
        if next >= bound {
            return bound;
        }
        bound = next;
    }
}

/// 一个 `data_units` 个单元（单元号 0 .. data_units − 1、没有洞）的文件，extent 树下段的全部位置（D8（核心索引结构） 已定项 14：
/// 只有一个单元时不建下段；第 L 层一个节点罩 144 × 147^L 个单元，从叶往上数到只剩一个节点的那一层为止）。
#[must_use]
pub fn extent_lower_node_positions(data_units: u64) -> Vec<ModelExtentLowerNodePosition> {
    let mut positions = Vec::new();
    if data_units <= 1 {
        return positions;
    }
    let mut level = 0;
    let mut span_in_data_units = EXTENT_TREE_LOWER_LEAF_DATA_UNITS;
    // 迭代上界：每一层的罩宽乘 147，节点数严格变少直到 1。
    loop {
        let nodes_in_this_level = data_units.div_ceil(span_in_data_units);
        positions.extend((0..nodes_in_this_level).map(|index_in_level| {
            ModelExtentLowerNodePosition {
                level,
                index_in_level,
            }
        }));
        if nodes_in_this_level == 1 {
            return positions;
        }
        level += 1;
        span_in_data_units *= EXTENT_TREE_INTERNAL_FANOUT;
    }
}

/// 下段一个位置罩的最后一个单元号（按满罩宽算，不按这个文件截到哪）：删除批判一个节点是不是整个落在留下那一段里用它
/// （`unlink::plan_the_extent_tree_after_a_deletion_batch` 的条款 K-R3-1）。
fn last_data_unit_under(position: ModelExtentLowerNodePosition) -> u64 {
    let span_in_data_units = (0..position.level)
        .fold(EXTENT_TREE_LOWER_LEAF_DATA_UNITS, |span, _| {
            span.saturating_mul(EXTENT_TREE_INTERNAL_FANOUT)
        });
    position
        .index_in_level
        .saturating_mul(span_in_data_units)
        .saturating_add(span_in_data_units - 1)
}

/// 下段一个位置罩的第一个单元号（同 [`last_data_unit_under`] 按满罩宽算）。
fn first_data_unit_under(position: ModelExtentLowerNodePosition) -> u64 {
    let span_in_data_units = (0..position.level)
        .fold(EXTENT_TREE_LOWER_LEAF_DATA_UNITS, |span, _| {
            span.saturating_mul(EXTENT_TREE_INTERNAL_FANOUT)
        });
    position.index_in_level.saturating_mul(span_in_data_units)
}

/// 一版的中央映射树里除分配记录树的节点之外装几条条目的上界（D19（块指针的结构与宽度预算） 已定项 8：进映射的是数据单元、extent 树每个节点、
/// inode 叶容器与根、分配记录树每个节点、记账树每个节点、`deleted_inodes` 根、墓碑树与谱系树的容器与索引节点；
/// 映射树自己的节点、树表、实例表豁免）。分配记录树的节点数由调用方按它的上界另加；记账树的节点按调用方给的这一版的上界
/// （`accounting_tree_nodes`，[`accounting_tree_nodes_upper_bound`]）数。
///
/// 谱系树模型不记（管理员回退往它上面追加一条，D18（块里携带什么信息） 已定项 24 第 5 条）：一条时间线上每次发布至多是一次回退，
/// 回退至多 txg 次，一片容器装 1359 条，加根；没回退过的一版没有这棵树，多算的两条只让上界松一点。
///
/// # Panics
/// 谱系容器的上界多于一片：模型罩的历史一条时间线上发布不到 1359 次，谱系树根之下不长层。
fn central_mapping_entries_other_than_the_allocation_record_tree(
    contents: &ModelVersionContents,
    role_written_at: &BTreeMap<ModelUnitRole, ModelCheckpointTxg>,
    accounting_tree_nodes: u64,
    checkpoint_txg: ModelCheckpointTxg,
) -> u64 {
    // 存在的单元（带洞的文件少于 size 罩的、截断途中连 size 之外还挂着的，判决 `research/prompts/m4-trunc-r1-main-verification.md`
    // 第三节 K2、K5）各一条，下段按存在的单元建。
    let data_units = contents.data_units_present();
    let entries_of_one_node_roles: u64 = role_written_at
        .keys()
        .map(|role| match role {
            // 记账树：根与根之下的每个节点各一条（台阶行多到一片叶装不下时长层），按这一版节点数的上界数。
            ModelUnitRole::AccountingTree => accounting_tree_nodes,
            // 模型罩的一版里这几个角色各一个单元（上段每个节点各是一个角色、每片 inode 叶容器各是一个角色、墓碑树一片容器加根）。
            ModelUnitRole::ExtentRoot
            | ModelUnitRole::ExtentUpperNodeBelowTheRoot(_)
            | ModelUnitRole::InodeLeaf
            | ModelUnitRole::InodeLeafRightOfTheLeftmost(_)
            | ModelUnitRole::InodeRoot
            | ModelUnitRole::DeletedInodesRoot
            | ModelUnitRole::LoggedOperationsRoot
            | ModelUnitRole::TombstoneLeaf
            | ModelUnitRole::TombstoneRoot
            | ModelUnitRole::RetirementLeaf
            | ModelUnitRole::RetirementRoot => 1,
            // 数据单元按这一版的单元数另数，分配记录树的节点由调用方按上界另加。
            ModelUnitRole::Data | ModelUnitRole::AllocationTree => 0,
            // 豁免映射（D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12）。
            ModelUnitRole::MappingTree
            | ModelUnitRole::TreeTable
            | ModelUnitRole::InstanceTable => 0,
        })
        .sum();
    let lineage_containers = checkpoint_txg.0.div_ceil(LINEAGE_CONTAINER_RECORDS);
    assert!(
        lineage_containers <= 1,
        "模型罩的历史一条时间线上发布不到 1359 次（谱系树至多一片容器）：txg {}",
        checkpoint_txg.0
    );
    let lineage_root = u64::from(lineage_containers > 0);
    data_units
        + extent_lower_segment_nodes(data_units)
        + entries_of_one_node_roles
        + lineage_containers
        + lineage_root
}

/// 一次发布写出的那一版的用户可见树（inode 树、extent 树、`deleted_inodes`）从哪来。
#[derive(Clone, Copy, Debug)]
enum UserVisibleTreesOfTheNewVersion<'target> {
    /// 这次发布按它的种类写出或照抄上一版的。
    AsThisPublishLeavesThem,
    /// 管理员回退那次：取回退目标那一版的（D23（journal 的角色与格式） 已定项 14）。
    TakenFromTheRollbackTarget(&'target ModelRoot),
}

/// 一次发布对第一个文件之外的 inode（[`ModelRoot::inodes_other_than_the_first`]）做了什么。
#[derive(Clone, Copy, Debug)]
enum InodesOtherThanTheFirstChangeOfThePublish<'content> {
    /// 照抄上一版的（管理员回退那次另取回退目标那一版的）。
    Carried,
    /// 建 `count` 个 size 0 的 inode（建 inode 的两种发布）：号从上一版的水位起连着发，水位加 `count`。
    Created { count: u64 },
    /// 把 `inode` 那个文件整份写成 `content`（[`ModelPublishKind::WholeWriteOfAnInodeOtherThanTheFirst`]）。
    WholeWrite { inode: u64, content: &'content [u8] },
    /// 按偏移写 `inode` 那个文件之后它读回 `content_after`（[`ModelPublishKind::PartialWriteOfAnInodeOtherThanTheFirst`]）：截断字段照抄，
    /// 单元与下段按种类带的那一段另改。
    PartialWrite {
        inode: u64,
        content_after: &'content [u8],
    },
}

/// 一次发布之后这一版的快照那几样（[`IdealModel::snapshot_bookkeeping_of_the_publish`]）。
struct SnapshotBookkeepingOfThePublish {
    snapshots: Rc<Vec<ModelSnapshot>>,
    previous_snapshot_txg_of_the_live_head: ModelCheckpointTxg,
    deadlist: Rc<Vec<ModelDeadlistEntry>>,
    pinned_mapping_entries: u64,
    deadlist_tree_nodes_upper_bound: u64,
    /// 这次至多写几个 deadlist 树的节点（占槽用）。
    deadlist_tree_nodes_written: u64,
    extent_lower_node_written_at: Rc<BTreeMap<ModelExtentLowerNodePosition, ModelCheckpointTxg>>,
}

/// 一次不是回退的发布对数据单元做了什么（[`IdealModel::data_unit_change_of_the_publish`]）：之后存在几个、这次写了哪几个。被换下的是上一版存在的单元里
/// 这次重写的与单元号不小于 `data_units_after` 的（尾部放掉的）；这次新有的单元都是这次写的。
#[derive(Clone, Debug, PartialEq, Eq)]
struct DataUnitChangeOfThePublish {
    data_units_before: u64,
    data_units_after: u64,
    /// 这次写的单元号：写内容的整份 0 .. 新单元数，截断与不缩短的截断至多一个尾单元，别的发布空；都小于 `data_units_after`。
    rewritten: std::ops::Range<u64>,
}

impl DataUnitChangeOfThePublish {
    /// 上一版存在、这次换下的单元号：重写的，与尾部放掉的。
    fn replaced(&self) -> impl Iterator<Item = u64> + '_ {
        (0..self.data_units_before)
            .filter(|index| *index >= self.data_units_after || self.rewritten.contains(index))
    }

    /// 这次之后每个存在的单元的分配代：重写的取这次的 txg，别的照抄上一版的。
    ///
    /// # Panics
    /// 这次新有的单元不是这次写的（单元只能经写出现）。
    fn data_unit_written_at_after(
        &self,
        previous: &[ModelCheckpointTxg],
        checkpoint_txg: ModelCheckpointTxg,
    ) -> Vec<ModelCheckpointTxg> {
        (0..self.data_units_after)
            .map(|index| {
                if self.rewritten.contains(&index) {
                    checkpoint_txg
                } else {
                    *previous
                        .get(usize::try_from(index).expect("单元号装得进 usize"))
                        .expect("这次没写的单元是上一版就存在的（单元只能经写出现）")
                }
            })
            .collect()
    }
}

/// 按偏移写的是哪个文件（[`IdealModel::answer_partial_write`]）：第一个文件的内容记在 [`ModelFileVersion`]，别的文件记在
/// [`ModelInodeOtherThanTheFirst`]，两处各走各的。
#[derive(Clone, Copy, Debug)]
enum ModelFileWrittenAtAnOffset<'version> {
    TheFirstFile(&'version ModelFileVersion),
    OtherThanTheFirst {
        inode_leaf_container: ModelUnitRole,
        file: &'version ModelInodeOtherThanTheFirst,
    },
}

/// 一次按偏移写 `[offset_in_bytes, end_exclusive_in_bytes)` 从第几个单元起重写、写之后有几个单元（判决
/// `research/prompts/m4-partial-write-r1-main-verification.md` 第二节 R4、R5；模型按条款自己算一份，不调实现的计划，D13（验证路线） 已定项 5）：
/// 写之后的单元数取写之前的与这次写到的最后一个单元号加一里大的（单元从 0 起连号，洞只在尾部）；重写从偏移所在的单元起（R4：写到 o 时把
/// o 之后的单元一起重写）——偏移落在最后一个存在的单元之后（空隙、洞里、紧接末尾）时，最后一个存在的单元没满一个净荷就从它起（它要补满），
/// 否则从写之前的单元数起——到写之后最后一个单元为止。`declared_length_of_the_last_existing_unit` 是写之前最后一个存在的单元的声明长度
/// （一个单元都没有为 `None`）。`end_exclusive_in_bytes` 大于 `offset_in_bytes`（这次至少一个字节）。
fn model_partial_write_rewritten_data_units(
    data_units_before: u64,
    declared_length_of_the_last_existing_unit: Option<u64>,
    offset_in_bytes: u64,
    end_exclusive_in_bytes: u64,
) -> (u64, u64) {
    let payload = data_unit_payload_capacity_in_bytes();
    let first_written_data_unit = offset_in_bytes / payload;
    let last_written_data_unit = (end_exclusive_in_bytes - 1) / payload;
    let data_units_after = data_units_before.max(last_written_data_unit + 1);
    let first_rewritten_data_unit = if first_written_data_unit < data_units_before {
        first_written_data_unit
    } else {
        match declared_length_of_the_last_existing_unit {
            Some(declared_length) if declared_length < payload => data_units_before - 1,
            Some(_) | None => data_units_before,
        }
    };
    (first_rewritten_data_unit, data_units_after)
}

/// 整份重写文件内容的入口按哪个契约收内容：覆盖写一个数据单元以内（装不下拒，D4（校验和位置） 已定项 5），顺序写不按单元数拒。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WholeFileWriteContract {
    OneDataUnit,
    AnyNumberOfDataUnits,
}

/// 删除那几步每批释放几个数据单元：产品取一片下段叶罩的 144 个（D8（核心索引结构） 已定项 16 第 2 条，K 取
/// `EXTENT_TREE_LOWER_LEAF_DATA_UNITS`）。模型只罩产品批大小：随机历史的挂载不装只供测试压小批大小的开关。
const MODEL_DELETION_BATCH_DATA_UNITS: u64 = EXTENT_TREE_LOWER_LEAF_DATA_UNITS;

/// 模型对拒绝理由的分法：按条款说的「为什么不能做」分，不按实现的错误成员分（胶水把成员映射过来）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModelRefusalReason {
    /// 内容装不进一个数据单元（D4（校验和位置） 已定项 5）。
    ContentExceedsDataUnitPayload,
    /// 第一个文件版本要建在上面的那一版的根，与交给它的上一条记录说的不是同一版：新根的 txg 从那条根接着算、jsn 从那条记录接着算，
    /// 两者对不上时接上去会盖在别的发布上。健康的历史里走不到（执行器恒拿现行那一版的根与记录）。
    FirstFileVersionDoesNotFollowTheVersionItBuildsOn,
    /// 第一个文件版本要建在上面的那一版已经有过文件版本：那条路径按「树还没建起来」写，树表条目的诞生 txg 与
    /// inode 1 的对象出生代都会取这次的 txg，与环里那些旧根记的对不上（I-9.14、I-9.10）。再写一版走覆盖写。
    FirstFileVersionOnAVersionThatAlreadyHasAFile,
    /// 单元区装不下（D28（挂载期承诺量） 已定项 1 的准入；模型答允许拒绝的区间）。
    UnitAreaWall,
    /// 回退目标不在根环里（D23（journal 的角色与格式） 已定项 14：候选集是根环里的根）。
    RollbackTargetNotInRing,
    /// 回退目标的 txg 低于 F_生效（D16（发布语义） 已定项 1「回退候选集」）。
    RollbackTargetBelowEffectiveFloor,
    /// 回退目标在被抛弃的时间线上（D23（journal 的角色与格式） 已定项 14：有行 (i, Ti, Wi) 且 T > Ti）。
    RollbackTargetOnAbandonedTimeline,
    /// 回退目标那一版树表 0 条（D23（journal 的角色与格式） 已定项 14：候选集只收带文件的根）。
    RollbackTargetWithoutFile,
    /// 要建立新实例的那一版树表 0 条、而它的实例表已经不是 mkfs 那一片（上一次挂载在这一版上写过行）。此前是必拒的一格；
    /// C512（树表 0 条的一版上被换下的单元记在哪） 2026-09-23 定案之后写行那次发布建起这一版自己的分配记录树、账取得回来，
    /// 这一格不再拒，模型的答案里一条都不列（`IdealModel::answer_establishing_an_instance` 那一段注释）。留着这个成员只为
    /// 胶水把实现在手造镜像上的那一拒（`MountError::VersionWithoutFileNotWrittenByMakeFilesystem`）映射过来：实现要是在随机历史里
    /// 拒在这一格，对拍报「模型说该成、实现拒了」。
    VersionWithoutFileNotWrittenByMakeFilesystem,
    /// 要抬的 F 超过上限（D16（发布语义） 已定项 1「抬 F 的上限」）。
    FloorAboveCeiling,
    /// 可写挂载读到的样子里有比所选那一版新的发布读不出、系统配置见证过它，重读一次仍读不出（C554 乙：用户 2026-09-27 定
    /// 「乙 重读后再判」，`research/prompts/c554-fix-forks.md` #1「挂载时有一条根（或它引用的账）暂时读不出」；重读次数取自 D16（发布语义） 已定项 1「根槽这一次读坏」那一行的
    /// 「重读一次」，挂进 D23（journal 的角色与格式） 已定项 14）：取号之前拒，盘上逐字节不变。
    NewerPublishWitnessedBySystemConfigurationStillUnreadableAfterOneReread,
    /// 现行那一版有树而文件集合为空（D8（核心索引结构） 已定项 16 第 5 条：删完之后只许建 inode（size 0）、写行与空发布），
    /// 这次改动不是建 inode：覆盖写、unlink 先走空队列（走空把删除中的那一版走成这一形），再拒（`UserChangeRefused::NoFileVersionToChange`）。
    /// 建快照与删快照不在此列（判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F5）。
    OnlyNewInodesOnAVersionWithTreesAndNoFile,
    /// 现行那一版带着只读快照：第一版有快照就拒回退（用户 2026-10-13 定，判决 `research/prompts/m4-snapshot-r1-main-verification.md`
    /// 第三节 S8 与第七节岔路 3）。
    RollbackWithSnapshotsWhichTheFirstVersionRefuses,
    /// 现行那一版已经有 [`MODEL_SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION`] 个只读快照：第 11 个拒（用户 2026-10-13 定「10」，判决
    /// `research/prompts/m4-snapshot-r1-main-verification.md` 第七节岔路 1），在任何写之前。
    MoreSnapshotsThanTheFirstVersionAllows,
    /// 截断到的长度 N 按整单元向上取整（⌈N ÷ 净荷⌉ × 净荷）溢出 u64：在任何写之前拒（判决
    /// `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K12）。
    TruncationSizeWhoseWholeDataUnitsOverflow,
    /// 第一个文件已删完的一版上按 inode 号给删完之后才建的 inode 写数据：D8（核心索引结构） 已定项 16 射程「删完之后不再建带数据的文件
    /// （K-U5，记欠账）」，用户 2026-10-19 定「β 时序级」——目标的对象出生代晚于 inode 1 墓碑的死亡 txg 才拒，删之前就有的别的文件照常能写；
    /// 在任何写之前拒。删第一个文件时树里还有的别的 inode 照常能写（[`ModelInodeOtherThanTheFirst::object_birth`] 不晚于那个死亡 txg）。
    DataWriteToAnInodeBornAfterTheFirstFileWasDeleted,
    /// 读—改—写要读回的旧数据单元读盘读不回、或读回来核不过（D4（校验和位置） 已定项 4：「读不回来、或读回来 MAC 验不过时，那个部分写失败」；
    /// 判决 `research/prompts/m4-partial-write-r1-main-verification.md` 第二节 R3「读盘」，截断与变长重写尾单元那条路同一读法）：在任何写之前拒，
    /// 不拿零填充凑一个合法的零。健康的内存盘上读得回每一个单元，模型的答案里一条都不列：实现在健康的历史里拒在这一格，对拍报「模型说该成、实现拒了」。
    OldDataUnitToRewriteNotReadBack,
}

impl ModelRefusalReason {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            ModelRefusalReason::ContentExceedsDataUnitPayload => "内容装不进一个数据单元",
            ModelRefusalReason::FirstFileVersionDoesNotFollowTheVersionItBuildsOn => {
                "第一个文件版本接不上它要建在上面的那一版"
            }
            ModelRefusalReason::FirstFileVersionOnAVersionThatAlreadyHasAFile => {
                "要建在上面的那一版已经有过文件版本（再写一版走覆盖写）"
            }
            ModelRefusalReason::UnitAreaWall => "单元区装不下",
            ModelRefusalReason::RollbackTargetNotInRing => "回退目标不在根环里",
            ModelRefusalReason::RollbackTargetBelowEffectiveFloor => "回退目标低于 F_生效",
            ModelRefusalReason::RollbackTargetOnAbandonedTimeline => "回退目标在被抛弃的时间线上",
            ModelRefusalReason::RollbackTargetWithoutFile => "回退目标那一版没有文件",
            ModelRefusalReason::VersionWithoutFileNotWrittenByMakeFilesystem => {
                "树表 0 条、而实例表已经不是 mkfs 那一片（条款没写被换下的那一片记在哪）"
            }
            ModelRefusalReason::FloorAboveCeiling => "要抬的 F 超过上限",
            ModelRefusalReason::NewerPublishWitnessedBySystemConfigurationStillUnreadableAfterOneReread => {
                "系统配置见证过比所选那一版新的发布、重读一次仍读不出（C554 乙）"
            }
            ModelRefusalReason::OnlyNewInodesOnAVersionWithTreesAndNoFile => {
                "有树而文件集合为空的一版只许建 inode"
            }
            ModelRefusalReason::RollbackWithSnapshotsWhichTheFirstVersionRefuses => {
                "带着只读快照的回退（第一版有快照就拒）"
            }
            ModelRefusalReason::MoreSnapshotsThanTheFirstVersionAllows => {
                "已经有 10 个只读快照（第一版至多 10 个）"
            }
            ModelRefusalReason::TruncationSizeWhoseWholeDataUnitsOverflow => {
                "截断到的长度按整单元向上取整溢出 u64"
            }
            ModelRefusalReason::DataWriteToAnInodeBornAfterTheFirstFileWasDeleted => {
                "删完第一个文件之后才建的 inode 写数据（K-U5 的 β 时序级读法，第一版不建带数据的文件）"
            }
            ModelRefusalReason::OldDataUnitToRewriteNotReadBack => {
                "读—改—写读不回要重写的旧数据单元（D4 已定项 4：读不回就失败）"
            }
        }
    }

    /// 容量墙：模型只答允许拒绝的区间（按这一步计划的每次发布之后的上界判），不答必须拒。
    fn is_capacity_wall_with_an_interval(self) -> bool {
        match self {
            ModelRefusalReason::UnitAreaWall => true,
            ModelRefusalReason::ContentExceedsDataUnitPayload
            | ModelRefusalReason::FirstFileVersionDoesNotFollowTheVersionItBuildsOn
            | ModelRefusalReason::FirstFileVersionOnAVersionThatAlreadyHasAFile
            | ModelRefusalReason::RollbackTargetNotInRing
            | ModelRefusalReason::RollbackTargetBelowEffectiveFloor
            | ModelRefusalReason::RollbackTargetOnAbandonedTimeline
            | ModelRefusalReason::RollbackTargetWithoutFile
            | ModelRefusalReason::VersionWithoutFileNotWrittenByMakeFilesystem
            | ModelRefusalReason::FloorAboveCeiling
            | ModelRefusalReason::NewerPublishWitnessedBySystemConfigurationStillUnreadableAfterOneReread
            | ModelRefusalReason::OnlyNewInodesOnAVersionWithTreesAndNoFile
            | ModelRefusalReason::RollbackWithSnapshotsWhichTheFirstVersionRefuses
            | ModelRefusalReason::MoreSnapshotsThanTheFirstVersionAllows
            | ModelRefusalReason::TruncationSizeWhoseWholeDataUnitsOverflow
            | ModelRefusalReason::DataWriteToAnInodeBornAfterTheFirstFileWasDeleted
            | ModelRefusalReason::OldDataUnitToRewriteNotReadBack => false,
        }
    }
}

/// 冷启动该读回什么（D23（journal 的角色与格式） 已定项 14：所选根 = 环里 (txg, 实例) 最大的那条；不建崩溃，没有要施加的记录）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModelReadBack {
    NoFile {
        root: ModelRootKey,
    },
    FileRead {
        root: ModelRootKey,
        content: Rc<[u8]>,
    },
}

/// 模型问的是哪一种操作（报告用）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModelOperationKind {
    PublishFirstFile,
    PublishOverwrite,
    PublishSequentialWrite,
    PublishWithoutUnits,
    MountWritable,
    RollbackWhileMounted,
    RaiseRollbackFloor,
    Unmount,
    ColdStartRecover,
    PublishUnlinkIntent,
    PublishDeletionStep,
    PublishNewInode,
    PublishSnapshotCreation,
    PublishSnapshotDestructionIntent,
    /// 删快照意图之后的一批（不是最后一批的与最后一批）：会话与挂载续做的走空走它们。
    PublishSnapshotDestructionStep,
    /// 整理的一批（`mounted_session::MountedSession::run_compaction`，判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G7、G8），
    /// 照抄意图的与同一次发布删意图的最后一批。
    PublishCompactionBatch,
    /// 签发一条整理意图（判决 G1）。
    PublishCompactionIntentIssue,
    /// 不搬单元、R 已按记录全空，单独删意图（意图达成，判决 G4）。
    PublishCompactionIntentCompletion,
    /// 不搬单元、R 里还有未释放的记录，丢掉意图（攻方 K-c；判决 G2）。
    PublishCompactionIntentDrop,
    /// 截断这次用户改动（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K2、K5、K11、K12）：缩短发截断意图，之后各批由
    /// [`ModelOperationKind::PublishTruncationStep`] 答；不缩短（变长与 N = S）一次发布做完。
    PublishTruncation,
    /// 截断意图之后的一批（不是最后一批的与最后一批）：会话与挂载续做的走空走它们（K1：次序在 `deleted_inodes`、销毁中的快照之后）。
    PublishTruncationStep,
    /// 按 inode 号整份写第一个文件之外的一个文件（里程碑「删除与多次 COW」一·1）；点名 inode 1 的照覆盖写与顺序写那两种。
    PublishWholeWriteOfAnInodeOtherThanTheFirst,
    /// 按偏移写一个活 inode（判决 `research/prompts/m4-partial-write-r1-main-verification.md` 第二节，[`IdealModel::answer_partial_write`]）：
    /// 第一个文件与别的文件同一种操作。
    PublishPartialWrite,
}

/// 执行器交给模型的、实现交回的一次整理发布看得见的几样（[`IdealModel::answer_compaction_publish`] 按它与模型自己记的活意图定是哪一种）。
/// 哪几个单元搬、搬几个、什么时候签发、什么时候丢由实现的选段与区域扫描定（判决 G7「只比成功操作的内容」），模型不答，只判这几样合不合条款。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelCompactionPublishSeen {
    /// 一批至多搬几个（执行器装的批大小）：占槽上界按它取。
    pub units_at_most: u64,
    /// 这一次搬了几个单元。
    pub units_relocated: u64,
    /// 搬的里头用户可见树的根单元（活头与快照条目引用的 inode 树根、extent 树根），胶水从实现交回的树表条目与搬迁表认。
    pub user_visible_root_units_relocated: Vec<ModelUserVisibleRootUnit>,
    /// 发布之后这一版的 logged_ops 里还有没有活整理意图。
    pub is_an_intent_logged_after_the_publish: bool,
    /// 模型那条活意图的 R 在实现交回的这一版里按记录全空没有（模型没有活意图时 false）。
    pub is_the_region_of_the_live_intent_empty_by_the_records_after_the_publish: bool,
    /// 执行器强制选的那一段（`CompactionSwitches::forced_region`）：签发的 R 就是它。
    pub region_forced_for_the_issue: ModelCompactionRegion,
}

fn describe_compaction_publish_seen(seen: &ModelCompactionPublishSeen) -> String {
    format!(
        "搬了 {} 个单元（其中用户可见树的根 {:?}），发布之后 logged_ops 里{}活意图，活意图的 R 按记录{}",
        seen.units_relocated,
        seen.user_visible_root_units_relocated,
        if seen.is_an_intent_logged_after_the_publish {
            "有"
        } else {
            "没有"
        },
        if seen.is_the_region_of_the_live_intent_empty_by_the_records_after_the_publish {
            "全空"
        } else {
            "不空（或没有活意图）"
        }
    )
}

/// 一步里计划的一次发布之后的上界（单元区墙区间的上端按它判）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PlannedPublishUpperBounds {
    occupied_slots_per_device: u64,
}

/// 模型对一步操作的答案。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelAnswer {
    pub operation: ModelOperationKind,
    /// 条款要求拒绝的理由：非空就必须拒，理由是这里的任一条或区间允许的任一条墙。
    pub required_refusals: BTreeSet<ModelRefusalReason>,
    /// 做成时写出的根，按次序。
    pub expected_roots: Vec<ModelRoot>,
    /// 做成时这次挂载取到的实例代号与写的行。
    pub expected_mount: Option<(ModelInstanceGeneration, Vec<ModelInstanceRow>)>,
    /// 冷启动该读回什么。
    pub expected_read_back: Option<ModelReadBack>,
    /// 抬 F 时模型算的上限（D16（发布语义） 已定项 1），拒与成都要与实现报的相等。
    pub rollback_floor_ceiling: Option<ModelCheckpointTxg>,
    /// 管理员回退的目标（只有回退那一步有）：做成时数「回退到 F_生效 那一代」的次数要它。
    pub rollback_target: Option<ModelRootKey>,
    planned_publish_upper_bounds: Vec<PlannedPublishUpperBounds>,
    /// 容量墙在第一次写之前一串判完（可写挂载、回退、抬 F——实现在任何写之前把那一串整串预演一遍）还是逐次发布判（发布）。
    walls_are_judged_before_the_first_write: bool,
    session_after_success: ModelSessionAfterSuccess,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ModelSessionAfterSuccess {
    /// 会话照旧开着，现行版本换成最后写出的那条根。
    StaysOpen,
    /// 这次挂载开了一个新会话，实例是这个。
    OpenedAs(ModelInstanceGeneration),
    /// 会话关着（冷启动）。
    Closed,
}

/// 实现写出的一条根，胶水从实现交回的东西里取。每一版都拿它的文件、实例表与分配记录跟模型那一版比（代码审阅第 12 条：
/// 此前只比有没有文件、只遍历实现交回的角色，内容只在冷启动那一步比）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservedRoot {
    pub key: ModelRootKey,
    pub journal_counter: ModelJournalCounter,
    pub rollback_floor: ModelCheckpointTxg,
    pub file: ObservedFile,
    pub instance_table: ObservedInstanceTable,
    pub unit_allocation_records: ObservedUnitAllocationRecords,
    /// 这一版的快照那几样（判决 S7）：活头前驱、快照列表、deadlist 条目数、被钉映射条目数。
    pub snapshot_state: ObservedSnapshotState,
    /// 这一版 logged_ops 里那一条活整理意图（胶水从实现交回的 logged_ops 条目取；树表 0 条的一版没有）。
    pub logged_compaction_intent: Option<ModelCompactionIntent>,
    /// 这一版第一个文件那条 inode 记录里截断字段那两段（胶水从记录的字节里取，[`ObservedTruncationField`]）；这一版没有那条记录（树表 0 条、
    /// 删完、删完之后建过 inode）为 None。
    pub first_file_record_truncation_field: Option<ObservedTruncationField>,
    /// 这一版 inode 树里第一个文件之外的每个活着的 inode（nlink 不是 0）：号 → 读回的内容（按 inode 号写数据写出来的，里程碑「删除与多次 COW」一·1）。
    /// 胶水从实现交回的那个文件的数据单元按单元号接起来、接到它那条记录的 size；没有数据单元、size 0 的是空内容。树表 0 条的一版是空表。
    pub inodes_other_than_the_first: BTreeMap<u64, ObservedFile>,
    /// 这一版 inode 树里第一个文件之外每条记录（活着的与删除中的）里截断字段那两段：号 → 原样值（胶水从记录的字节里取，[`ObservedTruncationField`]；
    /// 判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S5「截断字段按 inode 比」）。树表 0 条的一版是空表。
    pub truncation_fields_of_the_inodes_other_than_the_first:
        BTreeMap<u64, ObservedTruncationField>,
}

/// 实现交回的这一版的文件。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObservedFile {
    /// 树表 0 条：这一版没有文件。
    NoFile,
    /// 从这一版的数据单元按单元号接起来解出的内容（整份，不取摘要：模型只用 std 与格式常量，没有与胶水共用的哈希）。
    Decoded(Vec<u8>),
    /// 这一版带文件，而胶水从实现交回的单元里解不出它的内容（数据单元不在、解不开）。
    Undecodable { what: String },
}

impl ObservedFile {
    #[must_use]
    pub fn has_file(&self) -> bool {
        match self {
            ObservedFile::NoFile => false,
            ObservedFile::Decoded(_) | ObservedFile::Undecodable { .. } => true,
        }
    }
}

/// 实现交回的这一版的实例表。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObservedInstanceTable {
    /// 从这一版的实例表单元解出来的整张表，各片的行按链上的次序接起来。
    Rows(Vec<ModelInstanceRow>),
    /// 实现交回的东西里没有这一版整条实例表链的字节，比不了（`why` 写是哪一种；模型照计数，不判）。只许出现在这次没重写实例表的那几版
    /// （mkfs 之后第一次重写之前、从盘上重建且多于一片、零单元发布）：这次重写了实例表链的那一版交回它，模型判「这一版的实例表」对不上。
    NotInTheOutput { why: &'static str },
    /// 字节在，解不开。
    Undecodable { what: String },
}

/// 实现交回的这一版的分配记录。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObservedUnitAllocationRecords {
    /// 这一版每个单元在这一版分配记录里的那几条，每块盘各一条：实现交回的这一版的全部角色与全部数据单元（带文件的一版、树表 0 条的一版上写行的那一版）。
    EveryRoleOfTheVersion(ObservedAllocationRecordsOfEveryUnit),
    /// 实现的输出不带这一版的分配记录（树表 0 条的一版上的零单元发布：一个字节都不写、不经分配器），只交回这次发布重写了哪几个角色。
    /// 模型说这次重写了角色的那一版这么交回，判「单元的分配代」对不上。同一次挂载里前一版交回过分配记录的，执行器用的胶水从它往下带、
    /// 交回 [`ObservedUnitAllocationRecords::EveryRoleOfTheVersion`]（实审 B3c-4）；执行器里这一臂只剩一次挂载里第一版就是零单元发布的那几版。
    RewrittenRolesOnly(BTreeSet<ModelUnitRole>),
    /// 零单元发布交回的根换了分配记录树根指针（该照抄同一次挂载里前一版的那一条，胶水往下带时核出来的，实审 B3c-4）：
    /// 这一版的分配记录说不上是哪一份，模型判「单元的分配代」对不上，不当比不了。`what` 写两条指针。
    AllocationRecordTreeRootChangedByAZeroUnitPublish { what: String },
}

/// 实现交回的这一版每个单元的分配记录（[`ObservedUnitAllocationRecords::EveryRoleOfTheVersion`]）：数据单元按单元号各一项，别的角色各一项。
/// 数据单元按单元号交（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K4：截断只重写尾单元之后一个文件的数据单元分配代不同，
/// 模型按单元号逐个比，单元号与分配代的对应判得出）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservedAllocationRecordsOfEveryUnit {
    /// 数据单元之外的每个角色：这里不该有 [`ModelUnitRole::Data`]（数据单元在 `data_units` 里按单元号交），有就判对不上。
    pub roles: Vec<(ModelUnitRole, Vec<ObservedAllocationRecord>)>,
    /// 文件的每个数据单元：单元号与它的那几条分配记录。
    pub data_units: Vec<(u64, Vec<ObservedAllocationRecord>)>,
    /// 第一个文件之外的文件的每个数据单元：(inode 号, 单元号) 与它的那几条分配记录。
    pub data_units_of_the_files_other_than_the_first:
        Vec<((u64, u64), Vec<ObservedAllocationRecord>)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObservedAllocationRecord {
    pub device: ModelDeviceIdentity,
    pub generation: ModelCheckpointTxg,
    pub is_released: bool,
}

/// 实现冷启动读回了什么。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObservedReadBack {
    NoFile {
        root: ModelRootKey,
    },
    FileRead {
        root: ModelRootKey,
        content: Vec<u8>,
    },
    Failed {
        what: String,
    },
}

/// 实现做成了什么。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObservedEffect {
    /// 一次或几次发布（发布、抬 F）；抬 F 连同实现报的上限。
    Publishes {
        roots: Vec<ObservedRoot>,
        reported_ceiling: Option<ModelCheckpointTxg>,
    },
    Mount {
        instance: ModelInstanceGeneration,
        rows_written: Vec<ModelInstanceRow>,
        roots: Vec<ObservedRoot>,
    },
    ColdStart {
        read_back: ObservedReadBack,
    },
}

/// 实现的错误成员映射到模型的理由：`Explained` 里是这个成员说的那一条，`Unexplained` 是模型没有对应理由的成员（I/O、盘坏、
/// 第一版不支持的池形状……）。一个成员只映射到一条：此前回退「不在候选集里」映射成「两条之一」，实现报错了是哪一条模型分不出
/// （增补 3 第 2 件代码三方第一轮判决第三节第 2 条）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObservedRefusalReason {
    Explained(ModelRefusalReason),
    Unexplained,
}

/// 实现这一步的结局。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObservedOutcome {
    Succeeded(ObservedEffect),
    Refused {
        member: String,
        reason: ObservedRefusalReason,
        /// 拒之前已经做完的发布次数（抬 F 半路被拒时才可能非 0）。
        publishes_completed: usize,
        /// 录制流在这一步里有没有多出写或屏障。
        wrote_anything: bool,
        /// 实现报的上限（抬 F 被上限拒时）。
        reported_ceiling: Option<ModelCheckpointTxg>,
        /// 实现报「算抬 F 的上限、或挂着时回退判候选集时这个知道住着根的根环槽读坏、重读仍坏」时点名的那个槽
        /// （D16（发布语义） 已定项 1「根槽这一次读坏」那一行）；别的拒绝都是 None。
        reported_root_ring_slot_still_bad_after_one_reread: Option<ModelRingPosition>,
    },
}

/// 模型与实现对不上的是哪一格。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModelDisagreementAspect {
    SessionState,
    RefusedWhenModelRequiresSuccess,
    SucceededWhenModelRequiresRefusal,
    RefusalReason,
    WroteBeforeRefusing,
    PublishCount,
    RootIdentity,
    JournalCounter,
    RollbackFloor,
    FilePresence,
    FileContent,
    InstanceTableOfTheVersion,
    AllocationGeneration,
    InstanceGeneration,
    InstanceRows,
    RollbackFloorCeiling,
    ColdStartReadBack,
    /// 活头 inode 树那条条目的 `previous_snapshot_txg`（判决 S7「活头前驱」）。
    LiveHeadPreviousSnapshotTxg,
    /// 树表里的快照列表（判决 S7「快照列表」）：个数、每个的诞生 txg、前驱、销毁中记号、两条根指针抄的是谁。
    Snapshots,
    /// deadlist 条目，按盘上 key 的序逐条比（判决 S7 的条目数加删快照一批放的是哪几条，[`ModelDeadlistEntry`]）。
    DeadlistEntries,
    /// 留在中央映射树里、被活快照钉住的映射条目条数（判决 S7「被钉映射条目数」）。
    PinnedMappingEntries,
    /// 读快照读出的内容。
    SnapshotReadBack,
    /// 一次整理发布对意图做的事（签发、照抄、随最后一批删、单独删）与模型记的活意图、R 按记录全空没有对不上（判决 G1、G4；D8（核心索引结构） 已定项 5）。
    CompactionIntentOfThePublish,
    /// 这一版 logged_ops 里的整理意图（有没有、签发 txg、R、`c0`）。
    LoggedCompactionIntent,
    /// 这一版第一个文件那条 inode 记录里的截断字段（有没有那条记录、T、N_w；判决 `research/prompts/m4-trunc-r2-main-verification.md`
    /// 第三节 R1、R2）。
    FirstFileRecordTruncationField,
    /// 这一版第一个文件之外的 inode：有哪几个号、每个读回的内容（里程碑「删除与多次 COW」一·1）。
    InodesOtherThanTheFirst,
    /// 胶水交来的做成结局（[`ObservedEffect`] 的种类）不是这一种操作做成时交的那一种（冷启动只交冷启动读回、可写挂载只交挂载、
    /// 发布类只交发布）：接错了的胶水在判字段之前就拒，不拿空根列表比过去记成一次成功
    /// （`records/2026-10-24-singlefs-harness-恢复验证审计.md`「H03：模型允许冷启动答案被空发布结果替代」）。
    EffectKindOfTheOperation,
    NotModeled,
}

impl ModelDisagreementAspect {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            ModelDisagreementAspect::SessionState => "会话状态",
            ModelDisagreementAspect::RefusedWhenModelRequiresSuccess => "模型说该成、实现拒了",
            ModelDisagreementAspect::SucceededWhenModelRequiresRefusal => "模型说该拒、实现做成了",
            ModelDisagreementAspect::RefusalReason => "拒绝的理由",
            ModelDisagreementAspect::WroteBeforeRefusing => "拒绝之前写了盘",
            ModelDisagreementAspect::PublishCount => "写出的根的条数",
            ModelDisagreementAspect::RootIdentity => "根的 (txg, 实例)",
            ModelDisagreementAspect::JournalCounter => "jsn",
            ModelDisagreementAspect::RollbackFloor => "根带的 F",
            ModelDisagreementAspect::FilePresence => "这一版有没有文件",
            ModelDisagreementAspect::FileContent => "这一版的文件内容",
            ModelDisagreementAspect::InstanceTableOfTheVersion => "这一版的实例表",
            ModelDisagreementAspect::AllocationGeneration => "单元的分配代",
            ModelDisagreementAspect::InstanceGeneration => "取到的实例代号",
            ModelDisagreementAspect::InstanceRows => "写的实例表行",
            ModelDisagreementAspect::RollbackFloorCeiling => "抬 F 的上限",
            ModelDisagreementAspect::ColdStartReadBack => "冷启动读回",
            ModelDisagreementAspect::LiveHeadPreviousSnapshotTxg => "活头的 previous_snapshot_txg",
            ModelDisagreementAspect::Snapshots => "这一版的快照列表",
            ModelDisagreementAspect::DeadlistEntries => "这一版的 deadlist 条目（按 key 逐条）",
            ModelDisagreementAspect::PinnedMappingEntries => "这一版被钉住的映射条目数",
            ModelDisagreementAspect::SnapshotReadBack => "读快照读出的内容",
            ModelDisagreementAspect::CompactionIntentOfThePublish => "这一次整理发布对意图做的事",
            ModelDisagreementAspect::LoggedCompactionIntent => "这一版 logged_ops 里的整理意图",
            ModelDisagreementAspect::FirstFileRecordTruncationField => {
                "这一版第一个文件那条 inode 记录里的截断字段"
            }
            ModelDisagreementAspect::InodesOtherThanTheFirst => {
                "这一版第一个文件之外的 inode 与它们的内容"
            }
            ModelDisagreementAspect::EffectKindOfTheOperation => {
                "交来的做成结局不是这一种操作交的那一种"
            }
            ModelDisagreementAspect::NotModeled => "模型答不了",
        }
    }
}

/// 实现拒的时候点名的那一处读不出（D16（发布语义） 已定项 1「根槽这一次读坏」那一行；D23（journal 的角色与格式） 已定项 14「在任何写之前拒」那一格）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelReportedUnreadablePlace {
    /// 算抬 F 的上限、或挂着时回退判候选集读根环时，这个进程知道住着根的那一槽读坏、重读仍坏。
    /// 故障注入按它比「被吞的根槽写落在哪个槽」（实八）。
    RootRingSlotStillBadAfterOneReread(ModelRingPosition),
    /// 管理员回退逐盘验时回退目标引用的一个单元（复活集里的或隔离的）在一块盘上的那一份读不出或整单元 CRC-32C 对不上
    /// （执行器在回退被拒那一处经 [`ModelDisagreement::note_the_unit_copy_a_rollback_refused_on`] 记上）。
    /// 故障注入按它比「被吞的写落在哪」（`research/prompts/code-backlog-b2-code-r1-main-verification.md`「更正」一节 A4）。
    UnitCopyUnreadableOrMismatched(ModelUnitCopyLocation),
}

/// 模型与实现对不上：哪一格、模型答了什么、实现做了什么。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelDisagreement {
    pub aspect: ModelDisagreementAspect,
    pub model_answer: String,
    pub implementation_answer: String,
    /// 对不上的是抬 F 的上限（[`ModelDisagreementAspect::RollbackFloorCeiling`]）时实现报的那个数；别的格都是 None。
    /// 故障注入按它比「模型去掉被吞的根之后重算的上限」（实七，主 agent 2026-09-26 定的按注入点认）。
    pub implementation_reported_ceiling: Option<ModelCheckpointTxg>,
    /// 对不上的是「模型说该成、实现拒了」或「拒绝的理由」，而实现拒的是一处读不出、拒之前一个写都没发时它点名的那一处
    /// （[`ModelReportedUnreadablePlace`] 的两种）；别的格都是 None。故障注入按它比被吞的写落在哪。
    pub implementation_reported_unreadable_place: Option<ModelReportedUnreadablePlace>,
    /// 对不上的是「写的实例表行」（[`ModelDisagreementAspect::InstanceRows`]）时实现写的那几行；别的格都是 None。
    /// 故障注入按它比「被吞根槽的那几条根由记录重建之后模型重算的行」（实八，D23（journal 的角色与格式） 已定项 14 第 4 条）。
    pub implementation_rows_written: Option<Vec<ModelInstanceRow>>,
}

impl ModelDisagreement {
    /// 只带哪一格与两边的话，实现报的上限、读不出的那一处、写的行都是 None。
    #[must_use]
    pub fn new(
        aspect: ModelDisagreementAspect,
        model_answer: String,
        implementation_answer: String,
    ) -> Self {
        Self {
            aspect,
            model_answer,
            implementation_answer,
            implementation_reported_ceiling: None,
            implementation_reported_unreadable_place: None,
            implementation_rows_written: None,
        }
    }

    /// 管理员回退被拒、实现点名了「回退目标引用的一个单元在一块盘上的那一份读不出或对不上」时，把那一份记上：
    /// 只在对不上的是「模型说该成、实现拒了」或「拒绝的理由」、拒之前一个写都没发时记（mount.rs 那两个成员的契约：在任何写之前返回；
    /// 拒之前写了盘的，故障注入不许拿这一份去认）。别的格不动。
    pub fn note_the_unit_copy_a_rollback_refused_on(
        &mut self,
        unit_copy: Option<ModelUnitCopyLocation>,
        refused_before_any_write: bool,
    ) {
        let aspect_is_a_refusal_the_model_did_not_expect = match self.aspect {
            ModelDisagreementAspect::RefusedWhenModelRequiresSuccess
            | ModelDisagreementAspect::RefusalReason => true,
            ModelDisagreementAspect::SessionState
            | ModelDisagreementAspect::SucceededWhenModelRequiresRefusal
            | ModelDisagreementAspect::WroteBeforeRefusing
            | ModelDisagreementAspect::PublishCount
            | ModelDisagreementAspect::RootIdentity
            | ModelDisagreementAspect::JournalCounter
            | ModelDisagreementAspect::RollbackFloor
            | ModelDisagreementAspect::FilePresence
            | ModelDisagreementAspect::FileContent
            | ModelDisagreementAspect::InstanceTableOfTheVersion
            | ModelDisagreementAspect::AllocationGeneration
            | ModelDisagreementAspect::InstanceGeneration
            | ModelDisagreementAspect::InstanceRows
            | ModelDisagreementAspect::RollbackFloorCeiling
            | ModelDisagreementAspect::ColdStartReadBack
            | ModelDisagreementAspect::Snapshots
            | ModelDisagreementAspect::LiveHeadPreviousSnapshotTxg
            | ModelDisagreementAspect::DeadlistEntries
            | ModelDisagreementAspect::PinnedMappingEntries
            | ModelDisagreementAspect::SnapshotReadBack
            | ModelDisagreementAspect::LoggedCompactionIntent
            | ModelDisagreementAspect::CompactionIntentOfThePublish
            | ModelDisagreementAspect::FirstFileRecordTruncationField
            | ModelDisagreementAspect::InodesOtherThanTheFirst
            | ModelDisagreementAspect::EffectKindOfTheOperation
            | ModelDisagreementAspect::NotModeled => false,
        };
        if aspect_is_a_refusal_the_model_did_not_expect && refused_before_any_write {
            if let Some(unit_copy) = unit_copy {
                self.implementation_reported_unreadable_place =
                    Some(ModelReportedUnreadablePlace::UnitCopyUnreadableOrMismatched(unit_copy));
            }
        }
    }
}

/// 一步判完之后的计数：模型怎么答的、比了多少格。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ModelJudgementCounts {
    pub required_refusals_matched: u64,
    pub permitted_refusals_taken: u64,
    pub successes_matched: u64,
    pub roots_compared: u64,
    pub allocation_records_compared: u64,
    /// 比过「这次重写了哪几个角色」的次数（树表 0 条的一版上的零单元发布：它的输出不带分配记录）。
    pub rewritten_role_sets_compared: u64,
    /// 每一版比过文件内容的次数（带文件、胶水解得出内容的那几版）。
    pub file_contents_compared: u64,
    /// 每一版比过整张实例表的次数。
    pub instance_tables_compared: u64,
    /// 实现交回的东西里没有这一版整条实例表链的字节、比不了的次数（[`ObservedInstanceTable::NotInTheOutput`]）。
    pub instance_tables_not_in_the_output: u64,
    pub cold_start_contents_compared: u64,
    pub ceilings_compared: u64,
    /// 回退做成、目标的 txg 正好等于 F_生效且 F_生效 > 0（B2 那一格跑到了）。
    pub rollbacks_accepted_at_the_effective_floor: u64,
    /// 单元区墙拒（实现报每块盘上都没有合政策的落点）、在允许拒绝的区间里而放行的次数（小盘那一段的落点拒绝真的走到了、判过）。
    pub unit_area_wall_refusals_in_the_interval: u64,
    /// 比过、对得上的「不是最后一批的删除批」（文件多于一片下段叶的 144 个单元时才有，D8（核心索引结构） 已定项 16 第 2 条；代码轮 U13）。
    pub deletion_batches_keeping_data_units_matched: u64,
    /// 会话里一次用户改动之前先走空、走了至少两步，每一步都比过、对得上的次数（执行器数，D8（核心索引结构） 已定项 16 第 6 条；
    /// 代码轮第二轮判决第二节 N3「走空多步」那一格）。
    pub walks_of_several_steps_before_a_user_change_matched: u64,
    /// 会话里一次用户改动之前先走空时，走空的删除类发布被空间拒、推了抬 F 再试，推的那几串与走空的每一步都比过、对得上的次数
    /// （执行器数；代码轮第二轮判决第二节 N3「走空被空间拒」那一格）。
    pub walks_before_a_user_change_refused_for_space_matched: u64,
    /// 比过快照那几样（活头前驱、快照列表、deadlist 条目数、被钉映射条目数）的版数。
    pub snapshot_states_compared: u64,
    /// 比过的快照条数（逐版逐个加）。
    pub snapshots_compared: u64,
    /// 比过的版里 deadlist 不空的那几版（被快照钉住的单元真的走到了）。
    pub versions_with_deadlist_entries_compared: u64,
    /// 比过、对得上的读快照（读出的内容逐字节比）。
    pub snapshot_read_backs_compared: u64,
    /// 比过、对得上的删快照的批里不是最后一批的那几批（那个快照的条目多于一批 144 条时才有，判决 S4）。
    pub snapshot_destruction_batches_keeping_the_snapshot_matched: u64,
    /// 比过、对得上的整理的批（判决 G7「只比成功操作的内容」：每一批写出的那一版逐项与搬之前那一版的内容相同）。
    pub compaction_batches_matched: u64,
    /// 比过、对得上的签发（判决 G1）。
    pub compaction_intents_issued_matched: u64,
    /// 比过、对得上的同一次发布删意图的最后一批（D8（核心索引结构） 已定项 5）。
    pub compaction_intents_deleted_with_the_last_batch_matched: u64,
    /// 比过、对得上的单独删意图（R 已按记录全空，意图达成）。
    pub compaction_intents_deleted_without_a_batch_matched: u64,
    /// 比过、对得上的丢意图。
    pub compaction_intents_dropped_matched: u64,
    /// 比过 logged_ops 里那一条活整理意图的版数（有活意图的那几版）。
    pub logged_compaction_intents_compared: u64,
    /// 比过、对得上的搬了用户可见树根单元的整理发布（这一版算一个新状态，用户 2026-10-15 定「照字节比，搬根算非空」）。
    pub compaction_publishes_relocating_a_user_visible_root_matched: u64,
    /// 比过、对得上的截断意图（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K2）。
    pub truncation_intents_matched: u64,
    /// 比过、对得上的截断的批里不是最后一批的那几批（size 之外挂着的单元多于一批 144 个时才有）。
    pub truncation_batches_before_the_final_batch_matched: u64,
    /// 比过、对得上的截断最后一批。
    pub final_truncation_batches_matched: u64,
    /// 比过、对得上的不缩短的截断（变长与 N = S，K5、K12）。
    pub size_changes_without_shrinking_matched: u64,
    /// 比过第一个文件那条 inode 记录里截断字段的版数（有那条记录的那几版，判决 `research/prompts/m4-trunc-r2-main-verification.md` 第三节 R2）。
    pub truncation_fields_compared: u64,
    /// 其中模型记的截断字段写过（T ≠ 0）的版数。
    pub written_truncation_fields_compared: u64,
    /// 比过的第一个文件之外的 inode（逐版逐个加，里程碑「删除与多次 COW」一·1）。
    pub inodes_other_than_the_first_compared: u64,
    /// 其中有数据单元、逐字节比过内容的（逐版逐个加）。
    pub contents_of_files_other_than_the_first_compared: u64,
    /// 比过、对得上的按 inode 号写第一个文件之外的文件。
    pub writes_of_an_inode_other_than_the_first_matched: u64,
    /// 其中这次让 extent 树上段的根升了一层的（写的号罩不进原来的根）。
    pub writes_growing_the_extent_upper_segment_matched: u64,
    /// 其中写的号落在上段第 0 片叶之外的（号不小于一片叶罩的 143 个）。
    pub writes_past_the_first_extent_upper_leaf_matched: u64,
    /// 比过、对得上的「删完第一个文件之后给建的 inode 写数据」那一拒（K-U5）。
    pub data_writes_refused_after_the_first_file_was_deleted_matched: u64,
    /// 比过、对得上的按偏移写第一个文件（判决 `research/prompts/m4-partial-write-r1-main-verification.md` 第二节）。
    pub partial_writes_of_the_first_file_matched: u64,
    /// 比过、对得上的按偏移写第一个文件之外的文件（第二节 R6）。
    pub partial_writes_of_an_inode_other_than_the_first_matched: u64,
    /// 比过、对得上的删第一个文件之外的号的最后一批（判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M4「删别的号」）。
    pub deletions_of_an_inode_other_than_the_first_matched: u64,
    /// 其中接在第一个文件已删完的一版后面的（代码轮判决 `research/prompts/m3-close-w5-code-r1-main-verification.md` 第三节第 3 条：
    /// 随机历史在第一个文件删完之后也按号删别的文件）。
    pub deletions_of_an_inode_other_than_the_first_after_the_first_file_was_deleted_matched: u64,
    /// 比过、对得上的删第一个文件的最后一批、之后树里还有别的 inode 的（第三节 M4「删第一个文件而别的还在」，这一版归带文件的一版）。
    pub deletions_of_the_first_file_beside_other_inodes_matched: u64,
    /// 比过、对得上的删到文件集合为空的最后一批（第三节 M4「删到空」：之后是有树而文件集合为空的一版）。
    pub final_deletion_batches_emptying_the_version_matched: u64,
    /// 比过、对得上的让 extent 上段根降了层的最后一批（第三节 M4「上段降层」：上一版里降后层级的第 0 个节点照抄成根）。
    pub deletions_lowering_the_extent_upper_segment_matched: u64,
    /// 比过第一个文件之外的记录里截断字段的条数（逐版逐条加，判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S5）。
    pub truncation_fields_of_the_inodes_other_than_the_first_compared: u64,
    /// 其中模型记的截断字段写过（T ≠ 0）的条数。
    pub written_truncation_fields_of_the_inodes_other_than_the_first_compared: u64,
    /// 比过、对得上的带数据单元的别的 inode 改 size 那一次（截断意图与不缩短的那一次，第二节 S3）。
    pub size_changes_of_inodes_other_than_the_first_holding_data_units_matched: u64,
    /// 比过、对得上的带数据单元的别的 inode 截断的批与最后一批。
    pub truncation_steps_of_inodes_other_than_the_first_holding_data_units_matched: u64,
    /// 上面两样里接在有快照的一版后面的（快照下截断与变长）。
    pub size_changes_of_inodes_other_than_the_first_under_a_snapshot_matched: u64,
    /// 上面两样里接在 logged_ops 里有活整理意图的一版后面的（整理时截断）。
    pub size_changes_of_inodes_other_than_the_first_beside_a_live_compaction_intent_matched: u64,
    /// 比过、对得上的建快照里接在别的文件带数据单元的一版后面的。
    pub snapshots_created_beside_files_other_than_the_first_holding_data_units_matched: u64,
    /// 比过、对得上的整理的签发与批里接在别的文件带数据单元的一版后面的。
    pub compaction_publishes_beside_files_other_than_the_first_holding_data_units_matched: u64,
}

impl ModelJudgementCounts {
    /// 把另一份逐项加进来：按字段拆开写全，新加一个字段而这里没加，编译不过。
    pub fn add(&mut self, other: &ModelJudgementCounts) {
        let ModelJudgementCounts {
            required_refusals_matched,
            permitted_refusals_taken,
            successes_matched,
            roots_compared,
            allocation_records_compared,
            rewritten_role_sets_compared,
            file_contents_compared,
            instance_tables_compared,
            instance_tables_not_in_the_output,
            cold_start_contents_compared,
            ceilings_compared,
            rollbacks_accepted_at_the_effective_floor,
            unit_area_wall_refusals_in_the_interval,
            deletion_batches_keeping_data_units_matched,
            walks_of_several_steps_before_a_user_change_matched,
            walks_before_a_user_change_refused_for_space_matched,
            snapshot_states_compared,
            snapshots_compared,
            versions_with_deadlist_entries_compared,
            snapshot_read_backs_compared,
            snapshot_destruction_batches_keeping_the_snapshot_matched,
            compaction_batches_matched,
            compaction_intents_issued_matched,
            compaction_intents_deleted_with_the_last_batch_matched,
            compaction_intents_deleted_without_a_batch_matched,
            compaction_intents_dropped_matched,
            logged_compaction_intents_compared,
            compaction_publishes_relocating_a_user_visible_root_matched,
            truncation_intents_matched,
            truncation_batches_before_the_final_batch_matched,
            final_truncation_batches_matched,
            size_changes_without_shrinking_matched,
            truncation_fields_compared,
            written_truncation_fields_compared,
            inodes_other_than_the_first_compared,
            contents_of_files_other_than_the_first_compared,
            writes_of_an_inode_other_than_the_first_matched,
            writes_growing_the_extent_upper_segment_matched,
            writes_past_the_first_extent_upper_leaf_matched,
            data_writes_refused_after_the_first_file_was_deleted_matched,
            partial_writes_of_the_first_file_matched,
            partial_writes_of_an_inode_other_than_the_first_matched,
            deletions_of_an_inode_other_than_the_first_matched,
            deletions_of_an_inode_other_than_the_first_after_the_first_file_was_deleted_matched,
            deletions_of_the_first_file_beside_other_inodes_matched,
            final_deletion_batches_emptying_the_version_matched,
            deletions_lowering_the_extent_upper_segment_matched,
            truncation_fields_of_the_inodes_other_than_the_first_compared,
            written_truncation_fields_of_the_inodes_other_than_the_first_compared,
            size_changes_of_inodes_other_than_the_first_holding_data_units_matched,
            truncation_steps_of_inodes_other_than_the_first_holding_data_units_matched,
            size_changes_of_inodes_other_than_the_first_under_a_snapshot_matched,
            size_changes_of_inodes_other_than_the_first_beside_a_live_compaction_intent_matched,
            snapshots_created_beside_files_other_than_the_first_holding_data_units_matched,
            compaction_publishes_beside_files_other_than_the_first_holding_data_units_matched,
        } = other;
        self.required_refusals_matched += required_refusals_matched;
        self.permitted_refusals_taken += permitted_refusals_taken;
        self.successes_matched += successes_matched;
        self.roots_compared += roots_compared;
        self.allocation_records_compared += allocation_records_compared;
        self.rewritten_role_sets_compared += rewritten_role_sets_compared;
        self.file_contents_compared += file_contents_compared;
        self.instance_tables_compared += instance_tables_compared;
        self.instance_tables_not_in_the_output += instance_tables_not_in_the_output;
        self.cold_start_contents_compared += cold_start_contents_compared;
        self.ceilings_compared += ceilings_compared;
        self.rollbacks_accepted_at_the_effective_floor += rollbacks_accepted_at_the_effective_floor;
        self.unit_area_wall_refusals_in_the_interval += unit_area_wall_refusals_in_the_interval;
        self.deletion_batches_keeping_data_units_matched +=
            deletion_batches_keeping_data_units_matched;
        self.walks_of_several_steps_before_a_user_change_matched +=
            walks_of_several_steps_before_a_user_change_matched;
        self.walks_before_a_user_change_refused_for_space_matched +=
            walks_before_a_user_change_refused_for_space_matched;
        self.snapshot_states_compared += snapshot_states_compared;
        self.snapshots_compared += snapshots_compared;
        self.versions_with_deadlist_entries_compared += versions_with_deadlist_entries_compared;
        self.snapshot_read_backs_compared += snapshot_read_backs_compared;
        self.snapshot_destruction_batches_keeping_the_snapshot_matched +=
            snapshot_destruction_batches_keeping_the_snapshot_matched;
        self.compaction_batches_matched += compaction_batches_matched;
        self.compaction_intents_issued_matched += compaction_intents_issued_matched;
        self.compaction_intents_deleted_with_the_last_batch_matched +=
            compaction_intents_deleted_with_the_last_batch_matched;
        self.compaction_intents_deleted_without_a_batch_matched +=
            compaction_intents_deleted_without_a_batch_matched;
        self.compaction_intents_dropped_matched += compaction_intents_dropped_matched;
        self.logged_compaction_intents_compared += logged_compaction_intents_compared;
        self.compaction_publishes_relocating_a_user_visible_root_matched +=
            compaction_publishes_relocating_a_user_visible_root_matched;
        self.truncation_intents_matched += truncation_intents_matched;
        self.truncation_batches_before_the_final_batch_matched +=
            truncation_batches_before_the_final_batch_matched;
        self.final_truncation_batches_matched += final_truncation_batches_matched;
        self.size_changes_without_shrinking_matched += size_changes_without_shrinking_matched;
        self.truncation_fields_compared += truncation_fields_compared;
        self.written_truncation_fields_compared += written_truncation_fields_compared;
        self.inodes_other_than_the_first_compared += inodes_other_than_the_first_compared;
        self.contents_of_files_other_than_the_first_compared +=
            contents_of_files_other_than_the_first_compared;
        self.writes_of_an_inode_other_than_the_first_matched +=
            writes_of_an_inode_other_than_the_first_matched;
        self.writes_growing_the_extent_upper_segment_matched +=
            writes_growing_the_extent_upper_segment_matched;
        self.writes_past_the_first_extent_upper_leaf_matched +=
            writes_past_the_first_extent_upper_leaf_matched;
        self.data_writes_refused_after_the_first_file_was_deleted_matched +=
            data_writes_refused_after_the_first_file_was_deleted_matched;
        self.partial_writes_of_the_first_file_matched += partial_writes_of_the_first_file_matched;
        self.partial_writes_of_an_inode_other_than_the_first_matched +=
            partial_writes_of_an_inode_other_than_the_first_matched;
        self.deletions_of_an_inode_other_than_the_first_matched +=
            deletions_of_an_inode_other_than_the_first_matched;
        self.deletions_of_an_inode_other_than_the_first_after_the_first_file_was_deleted_matched +=
            deletions_of_an_inode_other_than_the_first_after_the_first_file_was_deleted_matched;
        self.deletions_of_the_first_file_beside_other_inodes_matched +=
            deletions_of_the_first_file_beside_other_inodes_matched;
        self.final_deletion_batches_emptying_the_version_matched +=
            final_deletion_batches_emptying_the_version_matched;
        self.deletions_lowering_the_extent_upper_segment_matched +=
            deletions_lowering_the_extent_upper_segment_matched;
        self.truncation_fields_of_the_inodes_other_than_the_first_compared +=
            truncation_fields_of_the_inodes_other_than_the_first_compared;
        self.written_truncation_fields_of_the_inodes_other_than_the_first_compared +=
            written_truncation_fields_of_the_inodes_other_than_the_first_compared;
        self.size_changes_of_inodes_other_than_the_first_holding_data_units_matched +=
            size_changes_of_inodes_other_than_the_first_holding_data_units_matched;
        self.truncation_steps_of_inodes_other_than_the_first_holding_data_units_matched +=
            truncation_steps_of_inodes_other_than_the_first_holding_data_units_matched;
        self.size_changes_of_inodes_other_than_the_first_under_a_snapshot_matched +=
            size_changes_of_inodes_other_than_the_first_under_a_snapshot_matched;
        self.size_changes_of_inodes_other_than_the_first_beside_a_live_compaction_intent_matched +=
            size_changes_of_inodes_other_than_the_first_beside_a_live_compaction_intent_matched;
        self.snapshots_created_beside_files_other_than_the_first_holding_data_units_matched +=
            snapshots_created_beside_files_other_than_the_first_holding_data_units_matched;
        self.compaction_publishes_beside_files_other_than_the_first_holding_data_units_matched +=
            compaction_publishes_beside_files_other_than_the_first_holding_data_units_matched;
    }
}

/// 一个可写会话：实例与接下来的发布要接在后面的那一版。
#[derive(Clone, Debug, PartialEq, Eq)]
struct ModelSession {
    instance: ModelInstanceGeneration,
    current: ModelRoot,
}

/// 理想模型的状态。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdealModel {
    geometry: ModelPoolGeometry,
    /// 单元区起始槽号：journal 环末尾的下一个槽（D3（空间分配） 已定项 10 ④，随环长走；D23（journal 的角色与格式） 已定项 19 ③）。
    /// 模型自己按环长算（[`model_unit_area_start_slot`]），不调实现的那一份（D13（验证路线） 已定项 5）。
    unit_area_start_slot: u64,
    /// 每区槽数 S：建池时给的那个，之后不变（[`ring_position_of`] 按它落槽）。
    slots_per_region: u64,
    ring: BTreeMap<ModelRingPosition, ModelRoot>,
    /// 取过的最大实例代号（系统配置里的号；取号先于任何带新号的写，D23（journal 的角色与格式） 已定项 16）。
    highest_acquired_instance: ModelInstanceGeneration,
    /// 发布过的最大 txg 与 jsn：不建崩溃，环里全部根与全部记录里的最大值就是它们（D23（journal 的角色与格式） 已定项 14 第 3 条）。
    highest_published_txg: ModelCheckpointTxg,
    highest_journal_counter: ModelJournalCounter,
    session: Option<ModelSession>,
    /// mkfs 之后写进根环的根一共几条（每写一条加一，第 0 代根不算）：执行器按每一步前后的差量一步写出几条根。
    roots_written_since_make_filesystem: u64,
}

/// 模型默认建的池的根环槽数 R × S（[`ROOT_RING_REGIONS`] 个区域、每区 [`MODEL_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM`] 个槽）：
/// 根按 txg 落进其中一个槽，一步里写出的根多于它，先写的就被同一步后写的盖掉。按别的 S 建的模型环长是 R 乘那个 S。
pub const MODEL_ROOT_RING_SLOTS: u64 =
    ROOT_RING_REGIONS * MODEL_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM;

/// 一版里的一个只读快照：树表里那一对码 14 / 15 条目（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S1、S2、S4）。
/// 快照随根走（每条根带着它那一版树表里的快照），不随模型走：崩溃恢复、可写挂载落到哪一版，就是那一版的快照。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelSnapshot {
    /// S.txg：建它那次发布的 checkpoint_txg（码 14 的诞生 txg，判决 S2）。
    pub birth_txg: ModelCheckpointTxg,
    /// 码 14 条目的 `previous_snapshot_txg`：派生链上排在它前面那一个的诞生 txg，没有为 0（D5（快照 / 空间记账机制） 已定项 9）；
    /// 删排在它前面那一个的意图那次换成被删那一个的前驱（判决 S4）。
    pub previous_snapshot_txg: ModelCheckpointTxg,
    /// 码 14 flags 位 0「销毁中」：删它的意图那次之后、最后一批之前（判决 S4）。
    pub is_being_destroyed: bool,
    /// 码 14 的根指针抄的是建它那一刻活头 inode 树的根：那个根是哪次发布写的（指针头部的出生 txg 与指针的实例代号）。
    pub inode_tree_written_by: ModelRootKey,
    /// 码 15 的根指针抄的是建它那一刻活头 extent 树的根：那个根的分配代（指针头部的出生 txg）。
    pub extent_root_written_at: ModelCheckpointTxg,
    /// 建它那一刻现行那一版里读得出的文件内容（那一版没有活文件时 `None`）：读快照照它答。
    pub content_at_creation: Option<Rc<[u8]>>,
}

/// 一条 deadlist 条目（D5（快照 / 空间记账机制） 已定项 12，判决 S3）：一个被活快照钉住、换下时没释放的用户可见单元。条目的 key 是
/// 头的树号 + 纪元 + 中央映射 key，映射 key 打头三段是类标签、出生树、出生 txg（D19（块指针的结构与宽度预算） 已定项 6）；头恒是活头 inode 树
/// （第一版只有一个可写头）。模型记纪元与映射 key 的打头三段，字段次序就是盘上 key 逐字段比的次序：删快照一批放「按 key 升序的前 144 条」
/// （判决 S4），模型照这个序挑。映射 key 末段（码 1 的写序、码 2 / 码 3 的实例代号与出生序号）模型不记：前三段相同的几条（同一次发布写的
/// 几个数据单元、几个下段节点）在模型里是同一个值，一批放其中哪几条模型分不出，逐条比时它们互相顶替。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModelDeadlistEntry {
    /// 块被杀那一刻活头的 `previous_snapshot_txg`。
    pub epoch: ModelCheckpointTxg,
    /// 映射 key 的类标签（[`MODEL_MAPPING_KEY_UNIT_CLASS_DATA`] 那三个）。
    pub mapping_key_unit_class: u8,
    /// 映射 key 的出生树：单元所在那棵树的树号（extent 树或 inode 树）。
    pub mapping_key_birth_tree: u64,
    /// 映射 key 的出生 txg：写这个单元的那次发布的 txg。
    pub birth_txg: ModelCheckpointTxg,
}

/// 中央映射 key 的类标签（D19（块指针的结构与宽度预算） 已定项 6：码 1 数据单元、码 2 索引节点、码 3 打包记录容器）。模型按条款自己写一份，
/// 不引实现的常量（D13（验证路线） 已定项 5）。
pub const MODEL_MAPPING_KEY_UNIT_CLASS_DATA: u8 = 1;
pub const MODEL_MAPPING_KEY_UNIT_CLASS_INDEX_NODE: u8 = 2;
pub const MODEL_MAPPING_KEY_UNIT_CLASS_PACKED_RECORDS: u8 = 3;

/// 一个换下的用户可见单元在映射 key 里的打头三段（[`ModelDeadlistEntry`] 去掉纪元）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ModelReplacedUnit {
    mapping_key_unit_class: u8,
    mapping_key_birth_tree: u64,
    birth_txg: ModelCheckpointTxg,
}

impl ModelUnitRole {
    /// 这个角色的单元在映射 key 里的类标签与出生树：数据单元、extent 根与 extent 下段节点生在 extent 树，inode 根与叶容器生在 inode 树；
    /// 叶容器是打包记录容器。只问问快照生死判定的那几类（[`ModelUnitRole::asks_the_snapshot_lifetime_judgement`]），别的角色交 `None`。
    fn mapping_key_unit_class_and_birth_tree(self) -> Option<(u8, u64)> {
        match self {
            ModelUnitRole::Data => {
                Some((MODEL_MAPPING_KEY_UNIT_CLASS_DATA, TREE_IDENTIFIER_EXTENT))
            }
            ModelUnitRole::ExtentRoot | ModelUnitRole::ExtentUpperNodeBelowTheRoot(_) => Some((
                MODEL_MAPPING_KEY_UNIT_CLASS_INDEX_NODE,
                TREE_IDENTIFIER_EXTENT,
            )),
            ModelUnitRole::InodeRoot => Some((
                MODEL_MAPPING_KEY_UNIT_CLASS_INDEX_NODE,
                TREE_IDENTIFIER_INODE,
            )),
            ModelUnitRole::InodeLeaf | ModelUnitRole::InodeLeafRightOfTheLeftmost(_) => Some((
                MODEL_MAPPING_KEY_UNIT_CLASS_PACKED_RECORDS,
                TREE_IDENTIFIER_INODE,
            )),
            ModelUnitRole::AllocationTree
            | ModelUnitRole::AccountingTree
            | ModelUnitRole::MappingTree
            | ModelUnitRole::TreeTable
            | ModelUnitRole::InstanceTable
            | ModelUnitRole::DeletedInodesRoot
            | ModelUnitRole::LoggedOperationsRoot
            | ModelUnitRole::TombstoneLeaf
            | ModelUnitRole::TombstoneRoot
            | ModelUnitRole::RetirementLeaf
            | ModelUnitRole::RetirementRoot => None,
        }
    }
}

/// extent 树下段一个节点的位置（D8（核心索引结构） 已定项 14：按单元号按位置寻址，层级 0 是叶，同层从单元 0 起数第几个）：
/// 一个没有洞的文件有几个单元就定了下段有哪几个位置。模型自己一份，不引实现的类型（D13（验证路线） 已定项 5）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModelExtentLowerNodePosition {
    pub level: u32,
    pub index_in_level: u64,
}

/// 第一版同时活着的只读快照至多几个：用户 2026-10-13 定「10」（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第七节岔路 1）。
/// 模型按条款自己写一份，不引实现的常量（D13（验证路线） 已定项 5）。
pub const MODEL_SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION: usize = 10;

/// 删快照每批放几条 deadlist 条目：与 unlink 每批的数据单元数同一个数（判决 S4「批大小 K = 144」）；模型只罩产品批大小
/// （随机历史的挂载不装只供测试压小批大小的开关，同 [`MODEL_DELETION_BATCH_DATA_UNITS`]）。
const MODEL_SNAPSHOT_DESTRUCTION_BATCH_ENTRIES: u64 = EXTENT_TREE_LOWER_LEAF_DATA_UNITS;

/// deadlist 树的 key 宽：头树号 8 + 纪元 8 + 映射 key（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3「key 43」），value 空。
const DEADLIST_KEY_BYTES: u64 = 8 + 8 + MAPPING_KEY_BYTES;

/// 一次读快照，模型说该读出什么（[`IdealModel::judge_snapshot_read_back`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModelSnapshotReadBack {
    /// 建快照那一刻没有活文件。
    NoFile,
    File(Rc<[u8]>),
}

/// 实现读一个快照读出了什么（`recovery::read_the_first_file_of_a_snapshot` 的结局，胶水换过来）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObservedSnapshotReadBack {
    NoFile,
    File(Vec<u8>),
    /// 树表里没有点名的那条码 14。
    NoSnapshotWithThisTree,
    /// 读失败（树、叶容器或数据单元读不出、解不开、判红）。
    Failed {
        what: String,
    },
}

/// 实现交回的这一版树表里的一个只读快照（胶水从码 14 / 15 条目取）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservedSnapshot {
    pub birth_txg: ModelCheckpointTxg,
    pub previous_snapshot_txg: ModelCheckpointTxg,
    pub is_being_destroyed: bool,
    /// 码 14 根指针头部的出生 txg 与指针的实例代号。
    pub inode_tree_written_by: ModelRootKey,
    /// 码 15 根指针头部的出生 txg。
    pub extent_root_written_at: ModelCheckpointTxg,
}

/// 实现交回的这一版的快照那几样（判决 S7「活头前驱、deadlist 条目数、被钉映射条目数、快照列表」）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservedSnapshotState {
    /// 活头 inode 树那条条目的 `previous_snapshot_txg`；树表 0 条的一版没有那条条目，是 `None`。
    pub live_head_previous_snapshot_txg: Option<ModelCheckpointTxg>,
    /// 树表里的快照，按（诞生 txg, 树号）升序（派生链的次序）。
    pub snapshots: Vec<ObservedSnapshot>,
    /// 这一版的 deadlist 条目（胶水从每条的纪元与映射 key 的打头三段取），次序不论：模型排好再逐条比。
    pub deadlist_entries: Vec<ModelDeadlistEntry>,
    /// 留在中央映射树里、被活快照钉住的映射条目条数（`TransactionOutput::mapping_entries_kept_for_the_snapshots`）。
    pub pinned_mapping_entries: u64,
}

impl ObservedSnapshotState {
    /// 树表 0 条的一版：没有活头 inode 树那条条目、没有快照、没有 deadlist。
    pub const OF_A_VERSION_WITHOUT_TREES: ObservedSnapshotState = ObservedSnapshotState {
        live_head_previous_snapshot_txg: None,
        snapshots: Vec::new(),
        deadlist_entries: Vec::new(),
        pinned_mapping_entries: 0,
    };
}
/// 环长 `journal_ring_bytes` 下的单元区起始槽号：journal 环从槽 [`JOURNAL_RING_START_SLOT`] 起，末尾不落在槽边界上时取下一个整槽
/// （D3（空间分配） 已定项 10 ④「第一版 = journal 环末尾的下一个槽」）。模型自己的一份式子（D13（验证路线） 已定项 5）。
fn model_unit_area_start_slot(journal_ring_bytes: u64) -> u64 {
    JOURNAL_RING_START_SLOT + journal_ring_bytes.div_ceil(SLOT_BYTES)
}

impl IdealModel {
    /// mkfs 之后（默认环长 768 MiB 的池，单元区从槽 50176 起）：见 [`IdealModel::after_make_filesystem_on_a_journal_ring_of`]。
    #[must_use]
    pub fn after_make_filesystem(geometry: ModelPoolGeometry) -> Self {
        Self::after_make_filesystem_on_a_journal_ring_of(geometry, JOURNAL_RING_DEFAULT_BYTES)
    }

    /// mkfs 之后：根环里只有第 0 代根（实例 0、txg 0、F 0、树表 0 条、实例表没有行），mkfs 写了实例表与第 0 版树表两个单元，
    /// 没有可写会话。单元区从环长 `journal_ring_bytes` 末尾的下一个槽起（[`model_unit_area_start_slot`]）。
    #[must_use]
    pub fn after_make_filesystem_on_a_journal_ring_of(
        geometry: ModelPoolGeometry,
        journal_ring_bytes: u64,
    ) -> Self {
        Self::after_make_filesystem_on_a_journal_ring_with_slots_per_region(
            geometry,
            journal_ring_bytes,
            MODEL_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM,
        )
    }

    /// 同 [`IdealModel::after_make_filesystem_on_a_journal_ring_of`]，每区槽数是 `slots_per_region`（实现那一侧建池时给的 S）：
    /// 根按这个 S 落槽，环长 R × S。
    ///
    /// # Panics
    /// `slots_per_region` 是 0：落槽按它取模。
    #[must_use]
    pub fn after_make_filesystem_on_a_journal_ring_with_slots_per_region(
        geometry: ModelPoolGeometry,
        journal_ring_bytes: u64,
        slots_per_region: u64,
    ) -> Self {
        assert!(slots_per_region > 0, "每区槽数 S 不是 0：落槽按它取模");
        let genesis = ModelRoot {
            key: ModelRootKey {
                checkpoint_txg: MAKE_FILESYSTEM_TXG,
                instance: MAKE_FILESYSTEM_INSTANCE,
            },
            journal_counter: ModelJournalCounter(0),
            rollback_floor: ModelCheckpointTxg(0),
            contents: ModelVersionContents::WithoutTrees,
            instance_table_rows: Rc::new(Vec::new()),
            role_written_at: BTreeMap::from([
                (ModelUnitRole::InstanceTable, MAKE_FILESYSTEM_TXG),
                (ModelUnitRole::TreeTable, MAKE_FILESYSTEM_TXG),
            ]),
            data_unit_written_at: Rc::new(Vec::new()),
            occupied_slots_upper_bound_per_device: ModelUnitRole::InstanceTable.span_in_slots()
                + ModelUnitRole::TreeTable.span_in_slots(),
            central_mapping_tree_nodes_upper_bound: 0,
            inode_number_watermark_steps: 0,
            accounting_rows: 0,
            accounting_tree_nodes_upper_bound: 0,
            highest_transaction_number_of_its_publish: 0,
            highest_transaction_number_in_its_instance: 0,
            snapshots: Rc::new(Vec::new()),
            previous_snapshot_txg_of_the_live_head: ModelCheckpointTxg(0),
            deadlist: Rc::new(Vec::new()),
            pinned_mapping_entries: 0,
            deadlist_tree_nodes_upper_bound: 0,
            extent_lower_node_written_at: Rc::new(BTreeMap::new()),
            logged_compaction_intent: None,
            user_visible_root_relocations: Rc::new(BTreeMap::new()),
            inodes_other_than_the_first: Rc::new(BTreeMap::new()),
            inode_number_watermark: 0,
            death_txg_of_the_first_file: None,
        };
        let mut ring = BTreeMap::new();
        ring.insert(
            ring_position_of(MAKE_FILESYSTEM_TXG, slots_per_region),
            genesis,
        );
        Self {
            geometry,
            unit_area_start_slot: model_unit_area_start_slot(journal_ring_bytes),
            slots_per_region,
            ring,
            highest_acquired_instance: MAKE_FILESYSTEM_INSTANCE,
            highest_published_txg: MAKE_FILESYSTEM_TXG,
            highest_journal_counter: ModelJournalCounter(0),
            session: None,
            roots_written_since_make_filesystem: 0,
        }
    }

    /// mkfs 之后写进根环的根一共几条（第 0 代根不算）。
    #[must_use]
    pub fn roots_written_since_make_filesystem(&self) -> u64 {
        self.roots_written_since_make_filesystem
    }

    /// mkfs 同一个进程里取号、暖机（D16（发布语义） 已定项 8：连推空发布到本实例的根覆盖每块盘；树表 0 条 ⇒ 零单元），会话开着、
    /// 接下来发第一个文件版本。与可写挂载只做过 mkfs 的池同一个形状，只是不经恢复。
    ///
    /// # Panics
    /// 模型不在 mkfs 刚做完的状态（这一步只在起点调）。
    pub fn acquire_and_warm_up_in_the_make_filesystem_process(&mut self) {
        assert!(
            self.session.is_none() && self.highest_acquired_instance == MAKE_FILESYSTEM_INSTANCE,
            "只在 mkfs 刚做完、还没取过号时调"
        );
        let answer = self.answer_establishing_an_instance(
            ModelOperationKind::MountWritable,
            &self.newest_root().clone(),
            None,
        );
        assert!(
            answer.required_refusals.is_empty(),
            "刚 mkfs 的池取号暖机：条款不要求拒（{:?}）",
            answer.required_refusals
        );
        self.advance_by_success(&answer);
    }

    /// 可写会话现行那一版树表里的只读快照，按建的先后（没有可写会话时 None）：执行器按它挑删哪一个、读哪一个，用例拿它看模型记的样子。
    #[must_use]
    pub fn snapshots_of_the_current_version(&self) -> Option<&[ModelSnapshot]> {
        self.session
            .as_ref()
            .map(|session| session.current.snapshots.as_slice())
    }

    /// 关掉这个进程的可写会话（可写挂载、回退、冷启动之前实现都先关）。
    pub fn close_session(&mut self) {
        self.session = None;
    }

    #[must_use]
    pub fn has_writable_session(&self) -> bool {
        self.session.is_some()
    }

    /// 模型此刻根环里的每一条根：身份与它下面的文件内容（树表 0 条是 None）。
    /// 崩溃注入（增补 3 第 3 件）每一步之后并一次，攒成「模型提交过的每一版」的目录，再拿它判崩溃状态恢复到的那一版
    /// （[`crash_recovery_disagreement`]）。根按 txg 落槽、同一步写的根 txg 连号，一步里写出的根不多于根环槽数
    /// （[`MODEL_ROOT_RING_SLOTS`]，R × S）时，这一步写的每一条到这一步做完都还在环里，每步并一次不漏。
    /// 一步写几条根没有写死的常数上界，按今天的代码数：可写挂载是写行一次、暖机至多 R 次（循环上限）、为空间推的抬 F
    /// （每次准入连写行在内至多 `mount::PUBLISHES_PER_ADMISSION_AT_MOST` 次发布，推的可以是几串）；抬 F 一串至多 R 次空发布；
    /// 会话里一次用户改动先走空删除队列（每批一次发布）、被空间拒就推抬 F，再发这次改动，次数随队列长度。随机历史里量到过一步 9 条
    /// （一次可写挂载推了两串抬 F）。不靠数出来的上界，执行器每一步之后量这一步写出的根数（[`Self::roots_written_since_make_filesystem`]
    /// 的差）、断言不多于根环槽数（`history::execute_history_with` 那一族，`HistoryTally::most_roots_written_by_the_model_in_one_step`）。
    #[must_use]
    pub fn committed_versions(&self) -> Vec<(ModelRootKey, Option<Rc<[u8]>>)> {
        self.ring
            .values()
            .map(|root| {
                (
                    root.key,
                    root.contents
                        .live_file()
                        .map(|file| Rc::clone(&file.content)),
                )
            })
            .collect()
    }

    /// 环里每条已提交的根与它那一版第一个文件之外每个活着的 inode 读得出的内容（判决
    /// `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S5「崩溃读回按 inode 给允许集合」）：崩溃之后恢复落到哪一版，那一版里
    /// 每个别的文件只许读出这里记的那一份（截断途中是目标那 N 字节，K2）。删除中的读不出，不在里面。
    #[must_use]
    pub fn committed_versions_of_the_inodes_other_than_the_first(
        &self,
    ) -> Vec<(ModelRootKey, ModelContentsOfTheInodesOtherThanTheFirst)> {
        self.ring
            .values()
            .map(|root| {
                (
                    root.key,
                    root.inodes_other_than_the_first
                        .iter()
                        .filter(|(_, inode)| inode.is_live())
                        .map(|(number, inode)| (*number, Rc::clone(&inode.content)))
                        .collect(),
                )
            })
            .collect()
    }

    fn newest_root(&self) -> &ModelRoot {
        self.ring
            .values()
            .max_by_key(|root| root.key)
            .expect("根环里至少有 mkfs 的第 0 代根：不建崩溃，没有读不出的根槽")
    }

    fn root_with_key(&self, key: ModelRootKey) -> Option<&ModelRoot> {
        self.ring.values().find(|root| root.key == key)
    }

    /// F_生效 = 各幸存盘最新持久有效根所带 F 的最大值（D16（发布语义） 已定项 1「生效」根那一半）：有效 = 按最新那条根指着的实例表
    /// 不被抛弃；每块盘取落在它上面的有效根里 (txg, 实例) 最大的那一条。一条根都没有时 0。
    /// 系统配置里的那一份 F（SysPre）模型不建：不建故障时抬 F 那一串要么在任何写之前被拒、要么推到每块盘，系统配置里的 F 与
    /// 那一串的根带的相同，两半取大不改答案。被抛弃时间线上的根带的 F 算不算，D16 写着仍开着；这里照「有效根」的字面，不算。
    #[must_use]
    pub fn effective_rollback_floor(&self) -> ModelCheckpointTxg {
        let newest_table = Rc::clone(&self.newest_root().instance_table_rows);
        let mut newest_valid_root_per_device: BTreeMap<ModelDeviceIdentity, &ModelRoot> =
            BTreeMap::new();
        for root in self.ring.values() {
            if Self::is_abandoned_by(root, &newest_table) {
                continue;
            }
            let newest = newest_valid_root_per_device
                .entry(device_holding_the_root_of(root.key.checkpoint_txg))
                .or_insert(root);
            if root.key > newest.key {
                *newest = root;
            }
        }
        newest_valid_root_per_device
            .values()
            .map(|root| root.rollback_floor)
            .max()
            .unwrap_or(ModelCheckpointTxg(0))
    }

    /// 按实例表判是不是被抛弃的：表里有它那个实例的行 (i, Ti, Wi) 且 txg > Ti（D23（journal 的角色与格式） 已定项 14）。
    /// 用哪一张表：最新那条根指着的（恢复与回退择的就是它；不建崩溃时它也是会话的现行版本）。
    fn is_abandoned_by(root: &ModelRoot, table: &[ModelInstanceRow]) -> bool {
        table.iter().any(|row| {
            row.instance == root.key.instance && root.key.checkpoint_txg > row.selected_root_txg
        })
    }

    /// 抬 F 的上限（D16（发布语义） 已定项 1）：min(每块盘上最新的有效根, 第 4 新的非空有效根)；非空有效根不足 4 个时取最旧的有效根。
    /// 有效 = 按最新根的实例表不被抛弃 ∧ txg ≥ 现行的 F（「非空」从盘上怎么认，2026-09-17 用户定案；B2 的窗口那一项 2026-10-04 RGall 拿掉）；现行的 F 取 F_生效（不建崩溃时与
    /// 现行根带的相等）。非空 = inode 树、extent 树的根指针与前一条有效根（按 (txg, 实例) 排）的不同，最旧的有效根与「没有这两棵树」比。
    /// 一块盘上没有有效根时不算它（条款没写这一格；不建崩溃时抬 F 与暖机都推到每块盘都有，走不到）。预想，跟收口表第 ② 行
    /// （候选集下界用哪个 F、被抛弃根带的 F 算不算都在那一行里重议）。
    #[must_use]
    pub fn rollback_floor_ceiling(&self) -> Option<ModelCheckpointTxg> {
        let current_floor = self.effective_rollback_floor();
        let newest_table = Rc::clone(&self.newest_root().instance_table_rows);
        // 上限与「非空」在环里全部有效根上取：B2（用户 2026-10-03 定）原来只取一圈窗口里的，用户 2026-10-04 定 RGall 与回收门槛一起退回整个环。
        let mut valid: Vec<&ModelRoot> = self
            .ring
            .values()
            .filter(|root| {
                !Self::is_abandoned_by(root, &newest_table)
                    && root.key.checkpoint_txg >= current_floor
            })
            .collect();
        valid.sort_by_key(|root| root.key);
        let mut newest_valid_per_device: BTreeMap<ModelDeviceIdentity, ModelCheckpointTxg> =
            BTreeMap::new();
        for root in &valid {
            let newest = newest_valid_per_device
                .entry(device_holding_the_root_of(root.key.checkpoint_txg))
                .or_insert(root.key.checkpoint_txg);
            *newest = (*newest).max(root.key.checkpoint_txg);
        }
        let newest_on_every_device = newest_valid_per_device.values().copied().min()?;
        // 「非空」比的是两棵用户可见树连同树表里快照那几对条目的根指针（`mount::user_visible_trees_changed`）：建快照那次与删快照的
        // 最后一批算非空，意图那次与不是最后一批的批不算（只改记号、前驱与 deadlist）；整理搬了这几个根单元之一的那一批算非空（根指针换成映射位置，
        // 用户 2026-10-15 定「照字节比，搬根算非空」），签发、删意图与只搬别的单元的那几次不算。
        let mut non_empty: Vec<(ModelRootKey, ModelUserVisibleState)> = Vec::new();
        let mut previous_state = ModelUserVisibleState {
            trees_written_by: None,
            snapshot_roots: Vec::new(),
            root_relocations: Vec::new(),
        };
        for root in &valid {
            let state = root.user_visible_state();
            if state != previous_state {
                non_empty.push((root.key, state.clone()));
            }
            previous_state = state;
        }
        // 「4 个不同状态」去重（「非空」从盘上怎么认那一段末尾）：从新到旧数，状态（两棵用户可见树是哪次发布写的、快照条目抄的是谁）与比它新、
        // 已计入的根相同的不计入——管理员回退发出的根状态同回退目标。去重之后不足 4 个取最旧的有效根（根环容量边界）。
        non_empty.sort_unstable_by_key(|(key, _)| std::cmp::Reverse(*key));
        let mut counted_states: Vec<ModelUserVisibleState> = Vec::new();
        let mut distinct_newest_first: Vec<ModelCheckpointTxg> = Vec::new();
        for (key, state) in non_empty {
            if !counted_states.contains(&state) {
                counted_states.push(state);
                distinct_newest_first.push(key.checkpoint_txg);
            }
        }
        let fourth_newest_non_empty = match distinct_newest_first.get(3) {
            Some(fourth) => *fourth,
            None => valid.first()?.key.checkpoint_txg,
        };
        Some(newest_on_every_device.min(fourth_newest_non_empty))
    }

    /// 把 `removed` 那几条根从根环里拿掉之后算的抬 F 的上限（[`IdealModel::rollback_floor_ceiling`] 同一个算法）：说谎的设备吞掉了
    /// 那几条根的根槽写，实现从盘上现读根环看不见它们（故障注入按注入点认「抬 F 的上限」那一格，实七）。
    #[must_use]
    pub fn rollback_floor_ceiling_without_the_roots(
        &self,
        removed: &[ModelRootKey],
    ) -> Option<ModelCheckpointTxg> {
        let mut view_without_the_roots = self.clone();
        view_without_the_roots
            .ring
            .retain(|_, root| !removed.contains(&root.key));
        if view_without_the_roots.ring.is_empty() {
            return None;
        }
        view_without_the_roots.rollback_floor_ceiling()
    }

    /// 分配记录树一次发布至多重写几个节点（每个节点每盘占一个槽）。
    ///
    /// 树的形状照 D8（核心索引结构） 已定项 14 另算一份：按绝对槽号按位置寻址，叶罩 812 个槽，内部节点罩 169 个孩子那么宽的一段；
    /// 根按盘分路，根的层号取「每块盘按下一层的宽度切出来的段数之和装得进一个根」的最低一层（实现员的取法，交回里写明）。
    ///
    /// 重写的是根，与这次之后还装着记录、内容变了的节点。模型不记哪几片变了，取「这次之后可能装着记录的节点」的全数：
    /// 记录只住在落点上；落点按 D3（空间分配） 已定项 10 从最低处取（开新段取最低的全空段，回落取最低的空槽），每个落点至多多开一个
    /// 64 槽的段 ⇒ 沿来路写过 P 个槽之后，落点都在单元区起点往后 `64 × P` 个槽里（单元区墙那一格同一个读法）。
    /// 这次重写的节点自己也占槽，P 里含着要求的那个数：设重写数为 A，A ≤ g(A)（g 是「写过 P₀ + A 个槽之后可能装着记录的节点数」，
    /// 对 A 单调不减），从 g(∞)（单元区里的全部节点）起往下迭代，每一步都仍是 A 的上界，停在不动点。
    fn allocation_record_tree_nodes_rewritten_upper_bound(
        &self,
        occupied_slots_before_the_tree_nodes: u64,
    ) -> u64 {
        let device_slots = self.geometry.device_size_in_bytes / SLOT_BYTES;
        let device_count = u64::try_from(self.geometry.devices.len()).expect("盘数");
        let span_at_level = |level: u32| {
            ALLOCATION_RECORD_TREE_LEAF_SLOTS
                .saturating_mul(ALLOCATION_RECORD_TREE_INTERNAL_FANOUT.saturating_pow(level))
        };
        let root_level = (1_u32..)
            .find(|level| {
                device_count * device_slots.div_ceil(span_at_level(level - 1))
                    <= ALLOCATION_RECORD_TREE_INTERNAL_FANOUT
            })
            .expect("层号往上走，每块盘切出来的段数终归是 1，盘数不到 169 就装得进一个根");
        let nodes_that_may_hold_records = |occupied_slots: u64| {
            let reach = self
                .unit_area_start_slot
                .saturating_add(CLUSTER_SEGMENT_SLOTS.saturating_mul(occupied_slots))
                .min(device_slots);
            let per_device: u64 = if reach <= self.unit_area_start_slot {
                0
            } else {
                (0..root_level)
                    .map(|level| {
                        let span = span_at_level(level);
                        (reach - 1) / span - self.unit_area_start_slot / span + 1
                    })
                    .sum()
            };
            1 + device_count * per_device
        };
        let mut bound = nodes_that_may_hold_records(u64::MAX);
        loop {
            let next = nodes_that_may_hold_records(
                occupied_slots_before_the_tree_nodes.saturating_add(bound),
            );
            if next >= bound {
                return bound;
            }
            bound = next;
        }
    }

    /// 这次发布之后中央映射树节点数的上界，与分配记录树这次至多重写几个节点
    /// （[`IdealModel::allocation_record_tree_nodes_rewritten_upper_bound`]）。
    ///
    /// 两个量互相牵连：映射树装着分配记录树每个节点的一条条目（D19（块指针的结构与宽度预算） 已定项 8），分配记录树可能装着记录的节点数
    /// 又随映射树多写的节点占的槽涨。两个都对另一个单调不减：从「分配记录树罩满单元区」那一头起交替往下迭代，
    /// 每一步都仍是两者的上界，停在映射树节点数不再变小的那一步。
    ///
    /// `occupied_slots_before_the_two_trees`：沿来路写过的槽，加这次除映射树根之下的节点与分配记录树之外写的槽（映射树的根已算在里面）。
    fn central_mapping_and_allocation_record_tree_nodes_upper_bounds(
        &self,
        central_mapping_tree_nodes_before: u64,
        central_mapping_entries_other_than_the_allocation_record_tree: u64,
        occupied_slots_before_the_two_trees: u64,
    ) -> (u64, u64) {
        let mapping_nodes_given = |allocation_record_tree_nodes: u64| {
            central_mapping_tree_nodes_upper_bound(
                central_mapping_tree_nodes_before,
                central_mapping_entries_other_than_the_allocation_record_tree
                    + allocation_record_tree_nodes,
            )
        };
        let allocation_record_tree_nodes_given = |mapping_nodes: u64| {
            self.allocation_record_tree_nodes_rewritten_upper_bound(
                occupied_slots_before_the_two_trees
                    + (mapping_nodes - 1) * ModelUnitRole::MappingTree.span_in_slots(),
            )
        };
        let mut mapping_nodes =
            mapping_nodes_given(self.allocation_record_tree_nodes_rewritten_upper_bound(u64::MAX));
        // 迭代上界：映射树节点数每一轮严格变小、不小于 1。跨轮带的是「两者的真值都不超过这一轮的 (映射树, 分配记录树)」。
        loop {
            let allocation_record_tree_nodes = allocation_record_tree_nodes_given(mapping_nodes);
            let next = mapping_nodes_given(allocation_record_tree_nodes);
            if next >= mapping_nodes {
                return (mapping_nodes, allocation_record_tree_nodes);
            }
            mapping_nodes = next;
        }
    }

    /// 接在 `previous` 后面、写记账行的一次发布写几行：池级三行加每块盘六行（D5（快照 / 空间记账机制） 已定项 8），加照抄的
    /// inode 号水位台阶行（D5 已定项 2 射程「inode 号水位的台阶行」）——这次自己那一行是不是新的一级台阶不改行数（它就是池级那一行）。
    fn accounting_rows_of_a_publish_on(&self, previous: &ModelRoot) -> u64 {
        let device_count = u64::try_from(self.geometry.devices.len()).expect("盘数");
        POOL_WIDE_ACCOUNTING_ROWS
            + ACCOUNTING_ROWS_PER_DEVICE * device_count
            + previous.inode_number_watermark_steps
    }

    /// 可写会话的现行版本（接下来的发布要接在它后面的那一版）；没有可写会话时 None。对拍的用例拿它的记账行数与记账树节点数的上界
    /// 看模型在哪一步判记账树长一层、那次占几个槽。
    #[must_use]
    pub fn current_version_in_the_session(&self) -> Option<&ModelRoot> {
        self.session.as_ref().map(|session| &session.current)
    }

    /// 单元区每块盘的槽数。
    fn unit_area_slots_per_device(&self) -> u64 {
        (self.geometry.device_size_in_bytes / SLOT_BYTES).saturating_sub(self.unit_area_start_slot)
    }

    /// 接在 `previous` 后面的一次发布写出的根（D16（发布语义） 已定项 6：txg + 1；jsn 接着全池计数器）。
    #[allow(
        clippy::too_many_arguments,
        reason = "一条根的每个身份字段各是一个参数，收成结构体只会多一层没人验的名字"
    )]
    fn next_root(
        &self,
        previous: &ModelRoot,
        instance: ModelInstanceGeneration,
        checkpoint_txg: ModelCheckpointTxg,
        journal_counter: ModelJournalCounter,
        kind: ModelPublishKind,
        rollback_floor: ModelCheckpointTxg,
        new_file_content: Option<&[u8]>,
        instance_table_rows: Rc<Vec<ModelInstanceRow>>,
    ) -> ModelRoot {
        self.next_root_with_the_user_visible_trees(
            previous,
            instance,
            checkpoint_txg,
            journal_counter,
            kind,
            rollback_floor,
            new_file_content,
            instance_table_rows,
            UserVisibleTreesOfTheNewVersion::AsThisPublishLeavesThem,
            InodesOtherThanTheFirstChangeOfThePublish::Carried,
        )
    }

    /// 同 [`IdealModel::next_root`]，另说这一版的用户可见树从哪来：管理员回退那次发布取回退目标那一版的。
    /// 中央映射树装的是这一版的条目，占槽上界要按换过之后的这一版算，所以换在算占槽之前做。
    #[allow(
        clippy::too_many_arguments,
        reason = "一条根的每个身份字段各是一个参数，收成结构体只会多一层没人验的名字"
    )]
    fn next_root_with_the_user_visible_trees(
        &self,
        previous: &ModelRoot,
        instance: ModelInstanceGeneration,
        checkpoint_txg: ModelCheckpointTxg,
        journal_counter: ModelJournalCounter,
        kind: ModelPublishKind,
        rollback_floor: ModelCheckpointTxg,
        new_file_content: Option<&[u8]>,
        instance_table_rows: Rc<Vec<ModelInstanceRow>>,
        user_visible_trees: UserVisibleTreesOfTheNewVersion<'_>,
        inodes_other_than_the_first_change: InodesOtherThanTheFirstChangeOfThePublish<'_>,
    ) -> ModelRoot {
        let key = ModelRootKey {
            checkpoint_txg,
            instance,
        };
        let creates_inodes = matches!(
            kind,
            ModelPublishKind::NewInode
                | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_)
        );
        let writes_an_inode_other_than_the_first = matches!(
            kind,
            ModelPublishKind::WholeWriteOfAnInodeOtherThanTheFirst { .. }
        );
        let writes_an_inode_other_than_the_first_at_an_offset = matches!(
            kind,
            ModelPublishKind::PartialWriteOfAnInodeOtherThanTheFirst { .. }
        );
        assert_eq!(
            (
                creates_inodes,
                writes_an_inode_other_than_the_first,
                writes_an_inode_other_than_the_first_at_an_offset
            ),
            match inodes_other_than_the_first_change {
                InodesOtherThanTheFirstChangeOfThePublish::Carried => (false, false, false),
                InodesOtherThanTheFirstChangeOfThePublish::Created { .. } => (true, false, false),
                InodesOtherThanTheFirstChangeOfThePublish::WholeWrite { .. } => (false, true, false),
                InodesOtherThanTheFirstChangeOfThePublish::PartialWrite { .. } => {
                    (false, false, true)
                }
            },
            "建 inode 的两种恰好带建了几个、按 inode 号写那一种与按偏移写别的文件那一种恰好带写了谁、写成什么，别的发布照抄：{kind:?}"
        );
        // 这次之后第一个文件之外的 inode 与水位（回退那次下面再换成回退目标那一版的 inode）。
        let mut inodes_other_than_the_first = previous.inodes_other_than_the_first.as_ref().clone();
        let mut inode_number_watermark = match kind {
            // 第一个文件那次发掉 inode 1（它接在树表 0 条的一版后面，`answer_publish_first_file` 判过有树的一版必拒）。
            ModelPublishKind::FirstFileVersion => FIRST_INODE_NUMBER + 1,
            ModelPublishKind::OverwriteFileVersion
            | ModelPublishKind::RowsOnFileVersion
            | ModelPublishKind::EmptyOnFileVersion
            | ModelPublishKind::RowsOnVersionWithoutFile
            | ModelPublishKind::ZeroUnit
            | ModelPublishKind::UnlinkIntent { .. }
            | ModelPublishKind::DeletionBatchKeepingDataUnits { .. }
            | ModelPublishKind::FinalDeletionBatch { .. }
            | ModelPublishKind::NewInode
            | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_)
            | ModelPublishKind::SnapshotCreation
            | ModelPublishKind::SnapshotDestructionIntent { .. }
            | ModelPublishKind::SnapshotDestructionBatch { .. }
            | ModelPublishKind::SnapshotDestructionFinalBatch { .. }
            | ModelPublishKind::CompactionBatch { .. }
            | ModelPublishKind::CompactionIntentIssue
            | ModelPublishKind::CompactionIntentDeletion { .. }
            | ModelPublishKind::TruncationIntent { .. }
            | ModelPublishKind::TruncationBatch
            | ModelPublishKind::FinalTruncationBatch { .. }
            | ModelPublishKind::SizeChangeWithoutShrinking { .. }
            | ModelPublishKind::WholeWriteOfAnInodeOtherThanTheFirst { .. }
            | ModelPublishKind::SizeChangeOfAnInodeWithoutDataUnits { .. }
            | ModelPublishKind::SizeChangeOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::TruncationBatchOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::PartialWriteOfTheFirstFile { .. }
            | ModelPublishKind::PartialWriteOfAnInodeOtherThanTheFirst { .. }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeWithoutDataUnits { .. } => {
                previous.inode_number_watermark
            }
        };
        // 删第一个文件之外的一个号（判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M1）：意图那次它成了删除中的、
        // 还剩它全部的单元；不是最后一批的批从尾部放 144 个；最后一批删掉它那条记录。删第一个文件的那几步不动这张表。
        match kind {
            ModelPublishKind::UnlinkIntent {
                deleted: ModelDeletedInode::OtherThanTheFirst(inode),
            } => {
                let deleted = inodes_other_than_the_first
                    .get_mut(&inode)
                    .expect("意图点名的号是这一版树里第一个文件之外活着的号（`answer_unlink_intent_of_an_inode` 判过）");
                deleted.liveness = ModelInodeLiveness::BeingDeleted {
                    data_units_remaining: deleted.data_units_present(),
                };
            }
            ModelPublishKind::DeletionBatchKeepingDataUnits {
                deleted: ModelDeletedInode::OtherThanTheFirst(inode),
            } => {
                let deleted = inodes_other_than_the_first
                    .get_mut(&inode)
                    .expect("删除批点名的号是这一版删除中的那个（`answer_deletion_step` 判过）");
                let ModelInodeLiveness::BeingDeleted {
                    data_units_remaining,
                } = deleted.liveness
                else {
                    panic!("删除批点名的号是这一版删除中的那个（`answer_deletion_step` 判过）：{deleted:?}");
                };
                deleted.liveness = ModelInodeLiveness::BeingDeleted {
                    data_units_remaining: data_units_remaining - MODEL_DELETION_BATCH_DATA_UNITS,
                };
            }
            ModelPublishKind::FinalDeletionBatch {
                deleted: ModelDeletedInode::OtherThanTheFirst(inode),
                ..
            } => {
                inodes_other_than_the_first
                    .remove(&inode)
                    .expect("最后一批点名的号是这一版删除中的那个（`answer_deletion_step` 判过）");
            }
            ModelPublishKind::UnlinkIntent {
                deleted: ModelDeletedInode::FirstFile,
            }
            | ModelPublishKind::DeletionBatchKeepingDataUnits {
                deleted: ModelDeletedInode::FirstFile,
            }
            | ModelPublishKind::FinalDeletionBatch {
                deleted: ModelDeletedInode::FirstFile,
                ..
            }
            | ModelPublishKind::FirstFileVersion
            | ModelPublishKind::OverwriteFileVersion
            | ModelPublishKind::RowsOnFileVersion
            | ModelPublishKind::EmptyOnFileVersion
            | ModelPublishKind::RowsOnVersionWithoutFile
            | ModelPublishKind::ZeroUnit
            | ModelPublishKind::NewInode
            | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_)
            | ModelPublishKind::SnapshotCreation
            | ModelPublishKind::SnapshotDestructionIntent { .. }
            | ModelPublishKind::SnapshotDestructionBatch { .. }
            | ModelPublishKind::SnapshotDestructionFinalBatch { .. }
            | ModelPublishKind::CompactionBatch { .. }
            | ModelPublishKind::CompactionIntentIssue
            | ModelPublishKind::CompactionIntentDeletion { .. }
            | ModelPublishKind::TruncationIntent { .. }
            | ModelPublishKind::TruncationBatch
            | ModelPublishKind::FinalTruncationBatch { .. }
            | ModelPublishKind::SizeChangeWithoutShrinking { .. }
            | ModelPublishKind::WholeWriteOfAnInodeOtherThanTheFirst { .. }
            // 按偏移写：第一个文件的内容由读回的整份带进来，别的文件的内容由 `inodes_other_than_the_first_change` 带进来（下面）。
            | ModelPublishKind::PartialWriteOfTheFirstFile { .. }
            | ModelPublishKind::PartialWriteOfAnInodeOtherThanTheFirst { .. } => {}
            // 没有数据单元的 inode 改 size（判决 m4-multi-inode-size-r1 第二节 S2、S3）：读回 N 个 0；第一次写上粘住的标签 0 的那一次起它在上段里
            // 有条目；截断意图那次 logged_ops 里那一条截断条目点名它。
            ModelPublishKind::SizeChangeOfAnInodeWithoutDataUnits {
                inode,
                new_size_in_bytes,
                change,
                ..
            } => {
                let changed = inodes_other_than_the_first.get_mut(&inode).expect(
                    "改 size 点名的号是这一版树里第一个文件之外活着、没有数据单元的号（`answer_size_change_of_an_inode_other_than_the_first` 判过）",
                );
                changed.content = Rc::from(vec![
                    0u8;
                    usize::try_from(new_size_in_bytes)
                        .expect("答法判过 N 装得进这个进程的地址空间")
                ]);
                match change {
                    ModelSizeChangeOfAnInodeWithoutDataUnits::WithoutShrinkingWritingTheTagZeroEntry => {
                        changed.keeps_a_tag_zero_upper_leaf_entry = true;
                    }
                    ModelSizeChangeOfAnInodeWithoutDataUnits::TruncationIntent => {
                        changed.truncation_issued_txg = Some(checkpoint_txg);
                    }
                    ModelSizeChangeOfAnInodeWithoutDataUnits::WithoutShrinkingCarryingTheExtentTree => {}
                }
            }
            ModelPublishKind::FinalTruncationBatchOfAnInodeWithoutDataUnits { inode, .. } => {
                inodes_other_than_the_first
                    .get_mut(&inode)
                    .expect("截断的最后一批点名的号是这一版截断条目点名的那个（`answer_truncation_step` 判过）")
                    .truncation_issued_txg = None;
            }
            // 带数据单元的 inode 改 size（判决 m4-multi-inode-size-r1 第二节 S3、S5）：读回截到 N、或补 0 到 N 的内容；截断意图那次 logged_ops 里那一条
            // 截断条目点名它，它那条记录的截断字段按这次放单元之前存在的单元数写或照抄（判决 m4-trunc-r2 第六节用户定案）。单元与下段在下面按单元号改。
            ModelPublishKind::SizeChangeOfAnInodeHoldingDataUnits {
                inode,
                new_size_in_bytes,
                direction,
                ..
            } => {
                let changed = inodes_other_than_the_first.get_mut(&inode).expect(
                    "改 size 点名的号是这一版树里第一个文件之外活着、带数据单元的号（`answer_size_change_of_an_inode_other_than_the_first` 判过）",
                );
                let mut content = changed.content.to_vec();
                content.resize(
                    usize::try_from(new_size_in_bytes)
                        .expect("答法判过 N 装得进这个进程的地址空间"),
                    0,
                );
                changed.content = Rc::from(content);
                match direction {
                    ModelDirectionOfASizeChange::TruncationIntent => {
                        changed.truncation_field = ModelTruncationField::after_a_truncation_intent(
                            changed.truncation_field,
                            changed.data_units_present(),
                            new_size_in_bytes,
                            checkpoint_txg,
                        );
                        changed.truncation_issued_txg = Some(checkpoint_txg);
                    }
                    ModelDirectionOfASizeChange::WithoutShrinking => {}
                }
            }
            // 它截断的批不碰记录：内容与截断条目照抄（单元与下段在下面按单元号改）。
            ModelPublishKind::TruncationBatchOfAnInodeHoldingDataUnits { .. } => {}
            ModelPublishKind::FinalTruncationBatchOfAnInodeHoldingDataUnits { inode, .. } => {
                inodes_other_than_the_first
                    .get_mut(&inode)
                    .expect("截断的最后一批点名的号是这一版截断条目点名的那个（`answer_truncation_step` 判过）")
                    .truncation_issued_txg = None;
            }
        }
        // 这一版父链上 inode 1 的死亡 txg：删第一个文件的意图那次写成这次的；别的照抄上一版的，管理员回退那次下面再换成回退目标那一版的。
        let death_txg_of_the_first_file_unless_rolled_back = match kind {
            ModelPublishKind::UnlinkIntent {
                deleted: ModelDeletedInode::FirstFile,
            } => Some(checkpoint_txg),
            ModelPublishKind::UnlinkIntent {
                deleted: ModelDeletedInode::OtherThanTheFirst(_),
            }
            | ModelPublishKind::DeletionBatchKeepingDataUnits { .. }
            | ModelPublishKind::FinalDeletionBatch { .. }
            | ModelPublishKind::FirstFileVersion
            | ModelPublishKind::OverwriteFileVersion
            | ModelPublishKind::RowsOnFileVersion
            | ModelPublishKind::EmptyOnFileVersion
            | ModelPublishKind::RowsOnVersionWithoutFile
            | ModelPublishKind::ZeroUnit
            | ModelPublishKind::NewInode
            | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_)
            | ModelPublishKind::SnapshotCreation
            | ModelPublishKind::SnapshotDestructionIntent { .. }
            | ModelPublishKind::SnapshotDestructionBatch { .. }
            | ModelPublishKind::SnapshotDestructionFinalBatch { .. }
            | ModelPublishKind::CompactionBatch { .. }
            | ModelPublishKind::CompactionIntentIssue
            | ModelPublishKind::CompactionIntentDeletion { .. }
            | ModelPublishKind::TruncationIntent { .. }
            | ModelPublishKind::TruncationBatch
            | ModelPublishKind::FinalTruncationBatch { .. }
            | ModelPublishKind::SizeChangeWithoutShrinking { .. }
            | ModelPublishKind::WholeWriteOfAnInodeOtherThanTheFirst { .. }
            | ModelPublishKind::SizeChangeOfAnInodeWithoutDataUnits { .. }
            | ModelPublishKind::SizeChangeOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::TruncationBatchOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::PartialWriteOfTheFirstFile { .. }
            | ModelPublishKind::PartialWriteOfAnInodeOtherThanTheFirst { .. }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeWithoutDataUnits { .. } => {
                previous.death_txg_of_the_first_file
            }
        };
        // 管理员回退那次的父是回退目标（谱系记录 (T_rb, T_old)）：父链上 inode 1 的死亡 txg 取回退目标那一版的，现行那一版在回退目标之后留下的
        // 墓碑落在被抛下的那一段里（β 只认这一版父链上的墓碑，代码轮判决 m3-close-w5-code-r1 第三节第 1 条）。
        let death_txg_of_the_first_file = match user_visible_trees {
            UserVisibleTreesOfTheNewVersion::AsThisPublishLeavesThem => {
                death_txg_of_the_first_file_unless_rolled_back
            }
            UserVisibleTreesOfTheNewVersion::TakenFromTheRollbackTarget(target_root) => {
                target_root.death_txg_of_the_first_file
            }
        };
        match inodes_other_than_the_first_change {
            InodesOtherThanTheFirstChangeOfThePublish::Carried => {}
            InodesOtherThanTheFirstChangeOfThePublish::Created { count } => {
                for new_inode in inode_number_watermark..inode_number_watermark + count {
                    inodes_other_than_the_first.insert(
                        new_inode,
                        ModelInodeOtherThanTheFirst {
                            content: Rc::from(Vec::new()),
                            data_unit_written_at: Rc::new(Vec::new()),
                            extent_lower_node_written_at: Rc::new(BTreeMap::new()),
                            truncation_field: ModelTruncationField::NotWritten,
                            object_birth: checkpoint_txg,
                            liveness: ModelInodeLiveness::Live,
                            keeps_a_tag_zero_upper_leaf_entry: false,
                            truncation_issued_txg: None,
                        },
                    );
                }
                inode_number_watermark += count;
            }
            InodesOtherThanTheFirstChangeOfThePublish::WholeWrite { inode, content } => {
                let written = inodes_other_than_the_first.get_mut(&inode).expect(
                    "按 inode 号写只点名这一版树里第一个文件之外活着的号（`answer_whole_write_of_an_inode` 判过）",
                );
                written.content = Rc::from(content);
                // 整份写清零（判决 `research/prompts/m4-trunc-r2-main-verification.md` 第三节 R2 推到任意 inode：`transaction::inode_record_of_file_version`）；
                // 单元与下段在下面按单元号改（全是这次的）。
                written.truncation_field = ModelTruncationField::NotWritten;
            }
            // 按偏移写（判决 m4-partial-write-r1 第二节 R5）：截断字段照抄（T、N_w 部分写不碰）；单元与下段在下面按单元号改（重写的那一段是这次的）。
            InodesOtherThanTheFirstChangeOfThePublish::PartialWrite {
                inode,
                content_after,
            } => {
                inodes_other_than_the_first
                    .get_mut(&inode)
                    .expect(
                        "按偏移写只点名这一版树里第一个文件之外活着的号（`answer_partial_write` 判过）",
                    )
                    .content = Rc::from(content_after);
            }
        }
        let data_units_written_to_the_file_other_than_the_first =
            match inodes_other_than_the_first_change {
                InodesOtherThanTheFirstChangeOfThePublish::WholeWrite { content, .. } => {
                    data_units_of_content(u64::try_from(content.len()).expect("内容长度"))
                }
                // 按偏移写的单元与下段节点在下面按单元号数占槽（同改 size）。
                InodesOtherThanTheFirstChangeOfThePublish::Carried
                | InodesOtherThanTheFirstChangeOfThePublish::Created { .. }
                | InodesOtherThanTheFirstChangeOfThePublish::PartialWrite { .. } => 0,
            };
        // 第一个文件之外那个文件这次的单元与下段（判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S3、S5，K4 推到任意 inode）：
        // 整份写、改 size、截断的批、unlink 的批与最后一批各按它自己上一版的单元与下段算（同第一个文件那一份，
        // [`IdealModel::data_units_and_lower_nodes_after_a_data_unit_change`]），换下的各按出生 txg 交给快照那一段判钉不钉。改 size 与截断那几次写的
        // 单元与下段节点在这里数占槽（整份写与 unlink 的批照旧在下面按形状数）。
        let mut replaced_units_besides_the_first_file: Vec<ModelReplacedUnit> = Vec::new();
        let mut slots_written_for_the_size_of_an_inode_other_than_the_first = 0;
        if let Some((changed_inode, change)) =
            Self::data_unit_change_of_an_inode_other_than_the_first(
                previous,
                kind,
                inodes_other_than_the_first_change,
            )
        {
            let before = previous
                .inodes_other_than_the_first
                .get(&changed_inode)
                .expect("这次动的号是上一版树里第一个文件之外的号（答法判过）");
            let (lower_nodes_after, replaced) =
                Self::data_units_and_lower_nodes_after_a_data_unit_change(
                    &before.data_unit_written_at,
                    &before.extent_lower_node_written_at,
                    &change,
                    checkpoint_txg,
                );
            if matches!(
                kind,
                ModelPublishKind::SizeChangeOfAnInodeHoldingDataUnits { .. }
                    | ModelPublishKind::TruncationBatchOfAnInodeHoldingDataUnits { .. }
                    | ModelPublishKind::FinalTruncationBatchOfAnInodeHoldingDataUnits { .. }
                    | ModelPublishKind::PartialWriteOfAnInodeOtherThanTheFirst { .. }
            ) {
                let lower_nodes_written = lower_nodes_after
                    .values()
                    .filter(|written_at| **written_at == checkpoint_txg)
                    .count();
                slots_written_for_the_size_of_an_inode_other_than_the_first =
                    (change.rewritten.end - change.rewritten.start)
                        * ModelUnitRole::Data.span_in_slots()
                        + u64::try_from(lower_nodes_written).expect("节点数")
                            * (NODE_BYTES / SLOT_BYTES);
            }
            replaced_units_besides_the_first_file = replaced;
            if let Some(after) = inodes_other_than_the_first.get_mut(&changed_inode) {
                after.data_unit_written_at = Rc::new(
                    change.data_unit_written_at_after(&before.data_unit_written_at, checkpoint_txg),
                );
                after.extent_lower_node_written_at = Rc::new(lower_nodes_after);
            }
        }
        let mut role_written_at = previous.role_written_at.clone();
        for role in kind.removed_roles() {
            role_written_at.remove(role);
        }
        let data_unit_change =
            Self::data_unit_change_of_the_publish(previous, kind, new_file_content);
        // 上一版删除中的那个号（第一个文件或别的文件）还剩几个单元：不是最后一批的删除批截的是它的下段。
        let data_units_remaining_before_this_publish = previous
            .inode_being_deleted()
            .map(|(_, data_units_remaining)| data_units_remaining);
        let rewritten = kind.rewritten_roles();
        // 数据单元按单元号另记（`data_unit_written_at`），不进按角色记的这张表。
        for role in rewritten
            .iter()
            .filter(|role| **role != ModelUnitRole::Data)
        {
            role_written_at.insert(*role, checkpoint_txg);
        }
        let rewrites_the_extent_upper_root = rewritten.contains(&ModelUnitRole::ExtentRoot);
        // 写上段叶条目的发布按这次之后存在的单元数与 size 写标签，不写的照抄（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K5）。
        let extent_upper_leaf_entry_label_after =
            |previous_file: Option<&ModelFileVersion>, size_in_bytes: usize| match (
                rewrites_the_extent_upper_root,
                previous_file,
            ) {
                (true, Some(_) | None) => ModelExtentUpperLeafEntryLabel::written_for(
                    data_unit_change.data_units_after,
                    data_units_of_content(u64::try_from(size_in_bytes).expect("内容长度")),
                ),
                (false, Some(previous_file)) => previous_file.extent_upper_leaf_entry_label,
                (false, None) => {
                    panic!("不重写 extent 上段根的发布接在带文件的一版后面，照抄它的标签：{kind:?}")
                }
            };
        let mut contents = match (kind, new_file_content) {
            (
                ModelPublishKind::FirstFileVersion | ModelPublishKind::OverwriteFileVersion,
                Some(content),
            ) => ModelVersionContents::LiveFile(ModelFileVersion {
                written_by: key,
                inode_record_written_by: key,
                content: Rc::from(content),
                data_units_present: data_unit_change.data_units_after,
                extent_upper_leaf_entry_label: extent_upper_leaf_entry_label_after(
                    None,
                    content.len(),
                ),
                // 整份写清零（判决 `research/prompts/m4-trunc-r2-main-verification.md` 第三节 R2、第六节用户定案）。
                truncation_field: ModelTruncationField::NotWritten,
            }),
            // 截断意图（K2「先落目标」）：size 写成 N、读回 N 字节，单元一个不放；只在带活文件的一版上发（`answer_truncation` 判过）。
            // 截断字段按这次放单元之前存在的单元数写或照抄（判决 m4-trunc-r2 第六节用户定案）。
            (ModelPublishKind::TruncationIntent { .. }, Some(content)) => {
                let ModelVersionContents::LiveFile(previous_file) = &previous.contents else {
                    panic!(
                        "截断意图只接在带活文件的一版后面（`answer_truncation` 判过）：{:?}",
                        previous.contents
                    );
                };
                ModelVersionContents::FileBeingTruncated {
                    file: ModelFileVersion {
                        written_by: key,
                        inode_record_written_by: key,
                        content: Rc::from(content),
                        data_units_present: data_unit_change.data_units_after,
                        extent_upper_leaf_entry_label: extent_upper_leaf_entry_label_after(
                            Some(previous_file),
                            content.len(),
                        ),
                        truncation_field: ModelTruncationField::after_a_truncation_intent(
                            previous_file.truncation_field,
                            previous_file.data_units_present,
                            u64::try_from(content.len()).expect("内容长度"),
                            checkpoint_txg,
                        ),
                    },
                    truncation_issued_txg: checkpoint_txg,
                }
            }
            // 截断的批（K2）：不碰 inode 记录，内容与截断条目照抄，从尾部放 144 个。
            (ModelPublishKind::TruncationBatch, None) => {
                let ModelVersionContents::FileBeingTruncated {
                    file: previous_file,
                    truncation_issued_txg,
                } = &previous.contents
                else {
                    panic!(
                        "截断的批只接在截断途中的那一版后面（`answer_truncation_step` 判过）：{:?}",
                        previous.contents
                    );
                };
                ModelVersionContents::FileBeingTruncated {
                    file: ModelFileVersion {
                        written_by: key,
                        inode_record_written_by: previous_file.inode_record_written_by,
                        content: Rc::clone(&previous_file.content),
                        data_units_present: data_unit_change.data_units_after,
                        extent_upper_leaf_entry_label: extent_upper_leaf_entry_label_after(
                            Some(previous_file),
                            previous_file.content.len(),
                        ),
                        truncation_field: previous_file.truncation_field,
                    },
                    truncation_issued_txg: *truncation_issued_txg,
                }
            }
            // 截断的最后一批（K2、K11）：放完 size 之外的单元、删截断条目，之后是带活文件的一版；放 0 个时 extent 树不动、用户可见树的根指针照抄。
            (ModelPublishKind::FinalTruncationBatch { release, .. }, None) => {
                let ModelVersionContents::FileBeingTruncated {
                    file: previous_file,
                    ..
                } = &previous.contents
                else {
                    panic!(
                        "截断的最后一批只接在截断途中的那一版后面（`answer_truncation_step` 判过）：{:?}",
                        previous.contents
                    );
                };
                ModelVersionContents::LiveFile(ModelFileVersion {
                    written_by: match release {
                        ModelFinalTruncationBatchRelease::AtLeastOneDataUnit => key,
                        ModelFinalTruncationBatchRelease::NoDataUnit => previous_file.written_by,
                    },
                    inode_record_written_by: previous_file.inode_record_written_by,
                    content: Rc::clone(&previous_file.content),
                    data_units_present: data_unit_change.data_units_after,
                    extent_upper_leaf_entry_label: extent_upper_leaf_entry_label_after(
                        Some(previous_file),
                        previous_file.content.len(),
                    ),
                    truncation_field: previous_file.truncation_field,
                })
            }
            // 不缩短的截断（K5、K12）：size 写成 N、读回补 0 到 N 的内容，单元一个不放、不新增。
            (ModelPublishKind::SizeChangeWithoutShrinking { .. }, Some(content)) => {
                let ModelVersionContents::LiveFile(previous_file) = &previous.contents else {
                    panic!(
                        "不缩短的截断只接在带活文件的一版后面（`answer_truncation` 判过）：{:?}",
                        previous.contents
                    );
                };
                ModelVersionContents::LiveFile(ModelFileVersion {
                    written_by: key,
                    inode_record_written_by: key,
                    content: Rc::from(content),
                    data_units_present: data_unit_change.data_units_after,
                    extent_upper_leaf_entry_label: extent_upper_leaf_entry_label_after(
                        Some(previous_file),
                        content.len(),
                    ),
                    // 变长与 N = S 照抄（判决 m4-trunc-r2 第三节 R2）。
                    truncation_field: previous_file.truncation_field,
                })
            }
            // 按偏移写第一个文件（判决 m4-partial-write-r1 第二节 R4、R5）：读回按偏移盖过、空隙读 0 的整份，size 是写之后的，单元数是写之后的，
            // 标签按写之后的单元数与 size 换（重写了上段根），截断字段照抄；只接在带活文件的一版后面（`answer_partial_write` 判过）。
            (ModelPublishKind::PartialWriteOfTheFirstFile { .. }, Some(content)) => {
                let ModelVersionContents::LiveFile(previous_file) = &previous.contents else {
                    panic!(
                        "按偏移写第一个文件只接在带活文件的一版后面（`answer_partial_write` 判过）：{:?}",
                        previous.contents
                    );
                };
                ModelVersionContents::LiveFile(ModelFileVersion {
                    written_by: key,
                    inode_record_written_by: key,
                    content: Rc::from(content),
                    data_units_present: data_unit_change.data_units_after,
                    extent_upper_leaf_entry_label: extent_upper_leaf_entry_label_after(
                        Some(previous_file),
                        content.len(),
                    ),
                    truncation_field: previous_file.truncation_field,
                })
            }
            // unlink 的意图与各批照抄截断字段（判决 m4-trunc-r2 第三节 R2）：意图只接在带活文件的一版后面（`answer_unlink_intent` 判过），
            // 各批只接在删除中的那一版后面（`answer_deletion_step` 判过）。
            (
                ModelPublishKind::UnlinkIntent {
                    deleted: ModelDeletedInode::FirstFile,
                },
                None,
            ) => {
                let truncation_field_of_the_unlinked_record = previous
                    .contents
                    .live_file()
                    .expect("unlink 的意图只接在带活文件的一版后面（`answer_unlink_intent` 判过）")
                    .truncation_field;
                ModelVersionContents::FileBeingDeleted {
                    trees_written_by: key,
                    data_units_remaining: data_unit_change.data_units_after,
                    truncation_field: truncation_field_of_the_unlinked_record,
                }
            }
            (
                ModelPublishKind::DeletionBatchKeepingDataUnits {
                    deleted: ModelDeletedInode::FirstFile,
                },
                None,
            ) => {
                let ModelVersionContents::FileBeingDeleted {
                    truncation_field, ..
                } = previous.contents
                else {
                    panic!(
                        "不是最后一批的删除批只接在删除中的那一版后面（`answer_deletion_step` 判过）：{:?}",
                        previous.contents
                    );
                };
                ModelVersionContents::FileBeingDeleted {
                    trees_written_by: key,
                    data_units_remaining: data_units_remaining_before_this_publish
                        .expect("不是最后一批的删除批只接在删除中的那一版后面（`answer_deletion_step` 判过）")
                        - MODEL_DELETION_BATCH_DATA_UNITS,
                    truncation_field,
                }
            }
            // 删第一个文件的最后一批：树里还有别的 inode 时是带文件的一版（D8（核心索引结构） 已定项 16 第 5 条，判决第三节 M3：不管 inode 1 在不在），
            // 没有了才是有树而文件集合为空的一版。
            (
                ModelPublishKind::FinalDeletionBatch {
                    deleted: ModelDeletedInode::FirstFile,
                    ..
                },
                None,
            ) => Self::contents_without_the_first_file(
                key,
                u64::try_from(inodes_other_than_the_first.len()).expect("条数"),
            ),
            // 删第一个文件之外的号的意图、批与最后一批：第一个文件（活着或已删完）照旧，inode 树换了一版（这次写的），批与最后一批连 extent 树。
            (
                ModelPublishKind::UnlinkIntent {
                    deleted: ModelDeletedInode::OtherThanTheFirst(_),
                }
                | ModelPublishKind::DeletionBatchKeepingDataUnits {
                    deleted: ModelDeletedInode::OtherThanTheFirst(_),
                }
                | ModelPublishKind::FinalDeletionBatch {
                    deleted: ModelDeletedInode::OtherThanTheFirst(_),
                    ..
                },
                None,
            ) => Self::contents_whose_trees_a_change_of_another_inode_rewrote(
                &previous.contents,
                key,
                u64::try_from(inodes_other_than_the_first.len()).expect("条数"),
                kind,
            ),
            // 建 inode：第一个文件活着时文件照旧，inode 树换了一版（这次写的），用户可见树的写者换成这次；删完之后建的数着树里几条。
            (
                ModelPublishKind::NewInode
                | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_),
                None,
            ) => match &previous.contents {
                ModelVersionContents::LiveFile(previous_file) => {
                    ModelVersionContents::LiveFile(Self::file_version_whose_trees_this_publish_rewrote(
                        previous_file,
                        key,
                    ))
                }
                ModelVersionContents::WithoutTrees
                | ModelVersionContents::FileBeingDeleted { .. }
                | ModelVersionContents::TreesAndNoFile { .. }
                | ModelVersionContents::InodesWithoutTheFirstFile { .. }
                | ModelVersionContents::FileBeingTruncated { .. } => {
                    let created = match inodes_other_than_the_first_change {
                        InodesOtherThanTheFirstChangeOfThePublish::Created { count } => count,
                        InodesOtherThanTheFirstChangeOfThePublish::Carried
                        | InodesOtherThanTheFirstChangeOfThePublish::WholeWrite { .. }
                        | InodesOtherThanTheFirstChangeOfThePublish::PartialWrite { .. } => {
                            unreachable!("建 inode 的两种恰好带建了几个（上面断言过）")
                        }
                    };
                    ModelVersionContents::InodesWithoutTheFirstFile {
                        trees_written_by: key,
                        inode_tree_written_by: key,
                        inode_records_in_the_tree: previous
                            .contents
                            .inode_records_created_after_the_first_file_was_deleted()
                            .expect("建 inode 只接在第一个文件活着或已删完的一版后面（`answer_create_inodes` 判过）")
                            + created,
                    }
                }
            },
            // 按 inode 号写第一个文件之外的文件：第一个文件的内容、单元、标签照旧，inode 树与 extent 树换了一版（这次写的）。
            // 第一个文件已删完时只写删之前就有的 inode（K-U5 β，`answer_whole_write_of_an_inode` 判过）。
            (ModelPublishKind::WholeWriteOfAnInodeOtherThanTheFirst { .. }, None)
            // 按偏移写第一个文件之外的文件：同按 inode 号写，第一个文件照旧，inode 树与 extent 树换了一版（这次写的）。
            | (ModelPublishKind::PartialWriteOfAnInodeOtherThanTheFirst { .. }, None)
            // 没有数据单元的 inode 改 size（判决 m4-multi-inode-size-r1 第二节 S3）：同按 inode 号写，第一个文件照旧，inode 树换了一版（这次写的）。
            | (ModelPublishKind::SizeChangeOfAnInodeWithoutDataUnits { .. }, None)
            // 带数据单元的那种改 size：inode 树换了一版（记录的 size），尾单元或标签变了时连 extent 树。
            | (ModelPublishKind::SizeChangeOfAnInodeHoldingDataUnits { .. }, None) => {
                Self::contents_whose_trees_a_change_of_another_inode_rewrote(
                    &previous.contents,
                    key,
                    u64::try_from(inodes_other_than_the_first.len()).expect("条数"),
                    kind,
                )
            }
            // 带数据单元的那种 inode 截断的批与放了单元的最后一批只写 extent 树（K2「批不碰 inode 记录」）：用户可见树里只有 extent 树的根指针换成这次写的。
            (
                ModelPublishKind::TruncationBatchOfAnInodeHoldingDataUnits { .. }
                | ModelPublishKind::FinalTruncationBatchOfAnInodeHoldingDataUnits {
                    release: ModelFinalTruncationBatchRelease::AtLeastOneDataUnit,
                    ..
                },
                None,
            ) => Self::contents_whose_extent_tree_a_truncation_batch_of_another_inode_rewrote(
                &previous.contents,
                key,
                kind,
            ),
            // 放 0 个单元的最后一批不碰 inode 树、不碰 extent 树。
            (
                ModelPublishKind::FinalTruncationBatchOfAnInodeHoldingDataUnits {
                    release: ModelFinalTruncationBatchRelease::NoDataUnit,
                    ..
                },
                None,
            ) => previous.contents.clone(),
            // 它截断的最后一批不碰 inode 树、不碰 extent 树：用户可见树的根指针照抄。
            (ModelPublishKind::FinalTruncationBatchOfAnInodeWithoutDataUnits { .. }, None) => {
                previous.contents.clone()
            }
            (
                ModelPublishKind::RowsOnFileVersion
                | ModelPublishKind::EmptyOnFileVersion
                | ModelPublishKind::RowsOnVersionWithoutFile
                | ModelPublishKind::ZeroUnit,
                None,
            ) => previous.contents.clone(),
            // 建快照与删快照的几次不动文件内容、extent 树与 inode 树（判决 S2、S4）。
            (
                ModelPublishKind::SnapshotCreation
                | ModelPublishKind::SnapshotDestructionIntent { .. }
                | ModelPublishKind::SnapshotDestructionBatch { .. }
                | ModelPublishKind::SnapshotDestructionFinalBatch { .. }

                | ModelPublishKind::CompactionBatch { .. }
                | ModelPublishKind::CompactionIntentIssue
                | ModelPublishKind::CompactionIntentDeletion { .. },
                None,
            ) => previous.contents.clone(),
            (
                ModelPublishKind::FirstFileVersion
                | ModelPublishKind::OverwriteFileVersion
                | ModelPublishKind::TruncationIntent { .. }
                | ModelPublishKind::SizeChangeWithoutShrinking { .. }
                | ModelPublishKind::PartialWriteOfTheFirstFile { .. },
                None,
            )
            | (
                ModelPublishKind::TruncationBatch | ModelPublishKind::FinalTruncationBatch { .. },
                Some(_),
            )
            | (
                ModelPublishKind::RowsOnFileVersion
                | ModelPublishKind::EmptyOnFileVersion
                | ModelPublishKind::RowsOnVersionWithoutFile
                | ModelPublishKind::ZeroUnit
                | ModelPublishKind::UnlinkIntent { .. }
                | ModelPublishKind::DeletionBatchKeepingDataUnits { .. }
                | ModelPublishKind::FinalDeletionBatch { .. }
                | ModelPublishKind::NewInode
                | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_)
                | ModelPublishKind::SnapshotCreation
                | ModelPublishKind::SnapshotDestructionIntent { .. }
                | ModelPublishKind::SnapshotDestructionBatch { .. }
                | ModelPublishKind::SnapshotDestructionFinalBatch { .. }

                | ModelPublishKind::CompactionBatch { .. }
                | ModelPublishKind::CompactionIntentIssue
                | ModelPublishKind::CompactionIntentDeletion { .. }
                // 按 inode 号写的内容走 `InodesOtherThanTheFirstChangeOfThePublish::WholeWrite`，这个参数说的是第一个文件。
                | ModelPublishKind::WholeWriteOfAnInodeOtherThanTheFirst { .. }
                // 没有数据单元的 inode 改成的 size 在种类里带着，这个参数说的是第一个文件。
                | ModelPublishKind::SizeChangeOfAnInodeWithoutDataUnits { .. }
                | ModelPublishKind::SizeChangeOfAnInodeHoldingDataUnits { .. }
                | ModelPublishKind::TruncationBatchOfAnInodeHoldingDataUnits { .. }
                | ModelPublishKind::FinalTruncationBatchOfAnInodeHoldingDataUnits { .. }
                | ModelPublishKind::FinalTruncationBatchOfAnInodeWithoutDataUnits { .. }
                // 按偏移写别的文件的内容走 `InodesOtherThanTheFirstChangeOfThePublish::PartialWrite`，这个参数说的是第一个文件。
                | ModelPublishKind::PartialWriteOfAnInodeOtherThanTheFirst { .. },
                Some(_),
            ) => panic!(
                "带内容的只有写文件内容的两种、截断意图、不缩短的截断与按偏移写第一个文件（读回的整份），别的都不带：{kind:?}"
            ),
        };
        // extent 树上段（规范形，判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M2）：按这次之后有条目的号集合与
        // 这次条目变了的号，模型自己的计划（[`extent_upper_node_written_at_after`]）定每个节点这次重写还是照抄；根与根之下的每个节点各是一个角色。
        // 种类定死的角色里有没有上段根，与计划说根这次重写没有，要一致。树表 0 条的一版没有上段。回退那次下面整组换成目标那一版的。
        let inodes_with_an_entry_after =
            inodes_with_an_extent_upper_leaf_entry(&contents, &inodes_other_than_the_first);
        let inodes_whose_entry_changed =
            Self::inodes_whose_extent_upper_leaf_entry_the_publish_changes(
                kind,
                inodes_other_than_the_first_change,
                &previous.inodes_with_an_extent_upper_leaf_entry(),
            );
        role_written_at.retain(|role, _| !role.is_a_node_of_the_extent_upper_segment());
        if contents.has_trees() {
            let root_level_after = extent_upper_segment_root_level(&inodes_with_an_entry_after);
            for (position, written_at) in extent_upper_node_written_at_after(
                &previous.extent_upper_node_written_at(),
                &inodes_with_an_entry_after,
                &inodes_whose_entry_changed,
                checkpoint_txg,
            ) {
                let role = if position.level == root_level_after {
                    ModelUnitRole::ExtentRoot
                } else {
                    ModelUnitRole::ExtentUpperNodeBelowTheRoot(position)
                };
                role_written_at.insert(role, written_at);
            }
            assert_eq!(
                role_written_at.get(&ModelUnitRole::ExtentRoot) == Some(&checkpoint_txg),
                rewrites_the_extent_upper_root,
                "种类定死的角色里有没有 extent 上段根，与规范形计划说根这次重写没有，要一致：{kind:?}，有条目的号 {inodes_with_an_entry_after:?}，变了的 {inodes_whose_entry_changed:?}"
            );
        }
        // extent 上段换下的节点按位置认（根与根之下的一样，判决 m4-snapshot-r1 S3：上段节点问快照生死判定）：上一版的一个位置，这一版没有它、或它这次
        // 重写了。长高那次旧根照抄成根之下的节点、降层那次根之下的节点照抄成根，都不算换下。
        let extent_upper_nodes_after = ModelRoot::extent_upper_node_written_at_of(
            &role_written_at,
            &inodes_with_an_entry_after,
        );
        let extent_upper_nodes_replaced: Vec<ModelReplacedUnit> = previous
            .extent_upper_node_written_at()
            .into_iter()
            .filter(|(position, written_at)| {
                extent_upper_nodes_after.get(position) != Some(written_at)
            })
            .map(|(_, written_at)| ModelReplacedUnit {
                mapping_key_unit_class: MODEL_MAPPING_KEY_UNIT_CLASS_INDEX_NODE,
                mapping_key_birth_tree: TREE_IDENTIFIER_EXTENT,
                birth_txg: written_at,
            })
            .collect();
        let mut data_unit_written_at = data_unit_change
            .data_unit_written_at_after(&previous.data_unit_written_at, checkpoint_txg);
        let mut rollback_rewrites_the_logged_operations_root = false;
        match user_visible_trees {
            UserVisibleTreesOfTheNewVersion::AsThisPublishLeavesThem => {}
            UserVisibleTreesOfTheNewVersion::TakenFromTheRollbackTarget(target_root) => {
                contents.clone_from(&target_root.contents);
                data_unit_written_at.clone_from(&target_root.data_unit_written_at);
                // 第一个文件之外的 inode 随 inode 树、extent 树取目标那一版的（记录、数据单元、下段都是那一版的）；水位不取目标的（照现行那一版）。
                inodes_other_than_the_first
                    .clone_from(target_root.inodes_other_than_the_first.as_ref());
                // 回退时 logged_ops 按种类分来源（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K1）：截断条目取回退目标那一版的
                // （随上面换过的 `contents`），整理意图随现行那一版。截断条目与现行那一版的不同时 logged_ops 这次重写，删空了取根指针置零。
                let truncation_entry_of_the_target = target_root.logged_truncation_issued_txg();
                if truncation_entry_of_the_target != previous.logged_truncation_issued_txg() {
                    if truncation_entry_of_the_target.is_some()
                        || previous.logged_compaction_intent.is_some()
                    {
                        role_written_at.insert(ModelUnitRole::LoggedOperationsRoot, checkpoint_txg);
                        rollback_rewrites_the_logged_operations_root = true;
                    } else {
                        role_written_at.remove(&ModelUnitRole::LoggedOperationsRoot);
                    }
                }
                // 新根的 inode 树、extent 树与 `deleted_inodes` 取回退目标那一版的（D23（journal 的角色与格式） 已定项 14）：
                // 目标那一版有的角色取它的分配代，没有的（删完的一版没有叶容器与 `deleted_inodes` 根）这一版也没有；数据单元整组取目标那一版的
                // （上面）；墓碑树与退役树随现行那一版。
                // extent 上段的根与根之下的节点同样整组取目标那一版的。
                role_written_at.retain(|role, _| !role.is_a_node_of_the_extent_upper_segment());
                for (role, written_at) in target_root
                    .role_written_at
                    .iter()
                    .filter(|(role, _)| role.is_a_node_of_the_extent_upper_segment())
                {
                    role_written_at.insert(*role, *written_at);
                }
                for role in [
                    ModelUnitRole::InodeLeaf,
                    ModelUnitRole::InodeRoot,
                    ModelUnitRole::DeletedInodesRoot,
                ] {
                    match target_root.role_written_at.get(&role) {
                        Some(written_at) => {
                            role_written_at.insert(role, *written_at);
                        }
                        None => {
                            role_written_at.remove(&role);
                        }
                    }
                }
                // 最左那片之外的叶容器同样整组取目标那一版的：现行那一版有、目标那一版没有的那几片这一版也没有。
                role_written_at.retain(|role, _| {
                    !matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(_))
                });
                role_written_at.extend(
                    target_root
                        .role_written_at
                        .iter()
                        .filter(|(role, _)| {
                            matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(_))
                        })
                        .map(|(role, written_at)| (*role, *written_at)),
                );
            }
        }
        assert_eq!(
            u64::try_from(data_unit_written_at.len()).expect("单元数"),
            contents.data_units_present(),
            "按单元号记的分配代与这一版存在的单元数一一对应：{kind:?}"
        );
        if let Some(file) = contents.live_file() {
            assert!(
                file.extent_upper_leaf_entry_label.admits(
                    file.data_units_present,
                    file.data_units_within_the_size(),
                    contents.logged_truncation_issued_txg().is_some(),
                ),
                "上段叶条目的标签 {:?} 与存在的 {} 个单元、size {} 对得上（判决第三节 K5、K7 的 T-3）：{kind:?}",
                file.extent_upper_leaf_entry_label,
                file.data_units_present,
                file.content.len()
            );
        }
        let snapshot_bookkeeping = Self::snapshot_bookkeeping_of_the_publish(
            previous,
            kind,
            checkpoint_txg,
            user_visible_trees,
            &data_unit_change,
            extent_upper_nodes_replaced,
            replaced_units_besides_the_first_file,
        );
        // 记账行：写记账树的发布照抄上一版的台阶行（回退那次的 `previous` 是现行那一版，台阶从它带），水位变了的再留一级；
        // 不写记账树的发布（树表 0 条的一版上的零单元发布）这一版没有记账树。记账树节点数的上界按这次写的行数从上一版的上界往下带
        // （回退那次实现从现行那一版的形状规划，`previous` 就是它）。
        let (inode_number_watermark_steps, accounting_rows, accounting_tree_nodes) =
            if rewritten.contains(&ModelUnitRole::AccountingTree) {
                let accounting_rows = self.accounting_rows_of_a_publish_on(previous);
                (
                    previous.inode_number_watermark_steps
                        + u64::from(kind.leaves_an_inode_number_watermark_step()),
                    accounting_rows,
                    accounting_tree_nodes_upper_bound(
                        previous.accounting_tree_nodes_upper_bound.max(1),
                        accounting_rows,
                    ),
                )
            } else {
                (
                    previous.inode_number_watermark_steps,
                    previous.accounting_rows,
                    previous.accounting_tree_nodes_upper_bound,
                )
            };
        let mut slots_written = 0;
        let mut rewrites_the_allocation_record_tree = false;
        for role in &rewritten {
            match role {
                ModelUnitRole::AllocationTree => rewrites_the_allocation_record_tree = true,
                // 这次写的数据单元：写内容的是内容的每一个，截断与不缩短的截断是那一个尾单元。
                ModelUnitRole::Data => {
                    slots_written += role.span_in_slots()
                        * (data_unit_change.rewritten.end - data_unit_change.rewritten.start);
                }
                // 记账树一次至多重写这一版的全部节点（根与根之下的，每个节点每盘占一个槽）：根之下的节点不对应角色，按上界算。
                ModelUnitRole::AccountingTree => {
                    slots_written += role.span_in_slots() * accounting_tree_nodes;
                }
                // 中央映射树在这里只记它的根：根之下的节点数按上界另算（下面）。extent 上段根之下的节点不在种类定死的角色里，
                // 按这一版的形状另算（下面）。
                ModelUnitRole::ExtentRoot
                | ModelUnitRole::ExtentUpperNodeBelowTheRoot(_)
                | ModelUnitRole::InodeLeaf
                | ModelUnitRole::InodeLeafRightOfTheLeftmost(_)
                | ModelUnitRole::InodeRoot
                | ModelUnitRole::MappingTree
                | ModelUnitRole::TreeTable
                | ModelUnitRole::InstanceTable
                | ModelUnitRole::DeletedInodesRoot
                | ModelUnitRole::LoggedOperationsRoot
                | ModelUnitRole::TombstoneLeaf
                | ModelUnitRole::TombstoneRoot
                | ModelUnitRole::RetirementLeaf
                | ModelUnitRole::RetirementRoot => slots_written += role.span_in_slots(),
            }
        }
        // 回退那次截断条目取回退目标那一版的、与现行那一版不同时 logged_ops 根这次重写（K1）。
        if rollback_rewrites_the_logged_operations_root {
            slots_written += ModelUnitRole::LoggedOperationsRoot.span_in_slots();
        }
        // extent 树下段的节点不对应角色（同分配记录树根之下的节点：分配代与位置归池级 checker 判），占槽按条款现算：
        // 写内容的两种重写整个下段；不是最后一批的删除批只重写被截掉那一片下段叶与它的祖先，每层至多一个；截断的批与放了单元的最后一批同
        // （按截之前的层数取上界）；重写尾单元的截断意图与不缩短的截断重写罩着它的那一片下段叶到根，每层一个（判决
        // `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K3）。
        let extent_lower_segment_nodes_written = match kind {
            ModelPublishKind::FirstFileVersion | ModelPublishKind::OverwriteFileVersion => {
                extent_lower_segment_nodes(data_unit_change.data_units_after)
            }
            ModelPublishKind::DeletionBatchKeepingDataUnits { .. } => {
                extent_lower_segment_levels(data_units_remaining_before_this_publish.expect(
                    "不是最后一批的删除批只接在删除中的那一版后面（`answer_deletion_step` 判过）",
                ))
            }
            ModelPublishKind::TruncationBatch
            | ModelPublishKind::FinalTruncationBatch {
                release: ModelFinalTruncationBatchRelease::AtLeastOneDataUnit,
                ..
            } => extent_lower_segment_levels(data_unit_change.data_units_before),
            ModelPublishKind::TruncationIntent {
                extent_tree: ModelExtentTreeChangeOfASizeChange::TailDataUnitRewritten,
            }
            | ModelPublishKind::SizeChangeWithoutShrinking {
                extent_tree: ModelExtentTreeChangeOfASizeChange::TailDataUnitRewritten,
            } => extent_lower_segment_levels(data_unit_change.data_units_after),
            // 按偏移写第一个文件（判决 m4-partial-write-r1 第二节 R4）：罩着重写那一段的下段节点与新长出来的（上面按「内容变了才写」算好的那一份里
            // 分配代是这次的那几个）。
            ModelPublishKind::PartialWriteOfTheFirstFile { .. } => u64::try_from(
                snapshot_bookkeeping
                    .extent_lower_node_written_at
                    .values()
                    .filter(|written_at| **written_at == checkpoint_txg)
                    .count(),
            )
            .expect("节点数"),
            ModelPublishKind::TruncationIntent {
                extent_tree:
                    ModelExtentTreeChangeOfASizeChange::OnlyTheUpperLeafEntryLabel
                    | ModelExtentTreeChangeOfASizeChange::Untouched,
            }
            | ModelPublishKind::FinalTruncationBatch {
                release: ModelFinalTruncationBatchRelease::NoDataUnit,
                ..
            }
            | ModelPublishKind::SizeChangeWithoutShrinking {
                extent_tree:
                    ModelExtentTreeChangeOfASizeChange::OnlyTheUpperLeafEntryLabel
                    | ModelExtentTreeChangeOfASizeChange::Untouched,
            }
            | ModelPublishKind::RowsOnFileVersion
            | ModelPublishKind::EmptyOnFileVersion
            | ModelPublishKind::RowsOnVersionWithoutFile
            | ModelPublishKind::ZeroUnit
            | ModelPublishKind::UnlinkIntent { .. }
            | ModelPublishKind::FinalDeletionBatch { .. }
            | ModelPublishKind::NewInode
            | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_)
            | ModelPublishKind::SnapshotCreation
            | ModelPublishKind::SnapshotDestructionIntent { .. }
            | ModelPublishKind::SnapshotDestructionBatch { .. }
            | ModelPublishKind::SnapshotDestructionFinalBatch { .. }
            | ModelPublishKind::CompactionBatch { .. }
            | ModelPublishKind::CompactionIntentIssue
            | ModelPublishKind::CompactionIntentDeletion { .. }
            // 第一个文件的下段一个节点都不写；写的那个文件的单元与下段在下面另算。
            | ModelPublishKind::WholeWriteOfAnInodeOtherThanTheFirst { .. }
            | ModelPublishKind::SizeChangeOfAnInodeWithoutDataUnits { .. }
            | ModelPublishKind::SizeChangeOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::TruncationBatchOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::PartialWriteOfAnInodeOtherThanTheFirst { .. }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeWithoutDataUnits { .. } => 0,
        };
        slots_written += extent_lower_segment_nodes_written * (NODE_BYTES / SLOT_BYTES);
        // 按 inode 号写第一个文件之外的文件：那个文件的每个数据单元与它的整个下段这次写（同第一个文件的整份写，D8（核心索引结构） 已定项 14）。
        slots_written += data_units_written_to_the_file_other_than_the_first
            * ModelUnitRole::Data.span_in_slots()
            + extent_lower_segment_nodes(data_units_written_to_the_file_other_than_the_first)
                * (NODE_BYTES / SLOT_BYTES);
        // 第一个文件之外那个文件改 size、截断的批与最后一批写的尾单元与下段节点（上面按单元号算出来的）。
        slots_written += slots_written_for_the_size_of_an_inode_other_than_the_first;
        // extent 树上段根之下的节点（上段长过层之后才有）各是一个角色，不在种类定死的角色里：这次重写的（规范形计划给它们这次的 txg）各占一个节点的槽。
        // 只有第一个文件的历史里一个都没有，那几组的占槽不变。
        let extent_upper_nodes_below_the_root_rewritten = role_written_at
            .iter()
            .filter(|(role, written_at)| {
                matches!(role, ModelUnitRole::ExtentUpperNodeBelowTheRoot(_))
                    && **written_at == checkpoint_txg
            })
            .count();
        slots_written += u64::try_from(extent_upper_nodes_below_the_root_rewritten)
            .expect("节点数")
            * (NODE_BYTES / SLOT_BYTES);
        // 整理的一批写目的地：至多 `units_at_most` 个单元，每个至多占一个数据单元那么宽（32 KiB 两槽，节点一槽）。
        let relocation_destination_slots_upper_bound = match kind {
            ModelPublishKind::CompactionBatch { units_at_most, .. } => {
                units_at_most * ModelUnitRole::Data.span_in_slots()
            }
            ModelPublishKind::FirstFileVersion
            | ModelPublishKind::OverwriteFileVersion
            | ModelPublishKind::RowsOnFileVersion
            | ModelPublishKind::EmptyOnFileVersion
            | ModelPublishKind::RowsOnVersionWithoutFile
            | ModelPublishKind::ZeroUnit
            | ModelPublishKind::UnlinkIntent { .. }
            | ModelPublishKind::DeletionBatchKeepingDataUnits { .. }
            | ModelPublishKind::FinalDeletionBatch { .. }
            | ModelPublishKind::NewInode
            | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_)
            | ModelPublishKind::SnapshotCreation
            | ModelPublishKind::SnapshotDestructionIntent { .. }
            | ModelPublishKind::SnapshotDestructionBatch { .. }
            | ModelPublishKind::SnapshotDestructionFinalBatch { .. }
            | ModelPublishKind::CompactionIntentIssue
            | ModelPublishKind::CompactionIntentDeletion { .. }
            | ModelPublishKind::TruncationIntent { .. }
            | ModelPublishKind::TruncationBatch
            | ModelPublishKind::FinalTruncationBatch { .. }
            | ModelPublishKind::SizeChangeWithoutShrinking { .. }
            | ModelPublishKind::WholeWriteOfAnInodeOtherThanTheFirst { .. }
            | ModelPublishKind::SizeChangeOfAnInodeWithoutDataUnits { .. }
            | ModelPublishKind::SizeChangeOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::TruncationBatchOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::PartialWriteOfTheFirstFile { .. }
            | ModelPublishKind::PartialWriteOfAnInodeOtherThanTheFirst { .. }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeWithoutDataUnits { .. } => 0,
        };
        slots_written += relocation_destination_slots_upper_bound;
        // deadlist 树的节点（码 2 节点，每盘占一个槽）：这一版或上一版有条目时按这一版节点数的上界算它这次全写一遍。
        slots_written +=
            snapshot_bookkeeping.deadlist_tree_nodes_written * (NODE_BYTES / SLOT_BYTES);
        // 实例表链多于一片时第 1 片起每一片也是一个重写的单元：`rewritten_roles` 里实例表只记一个角色，多出来的几片在这里补上。
        let instance_table_pages_after_the_first = if kind.rewrites_the_instance_table() {
            instance_table_pages_for_rows(instance_table_rows.len()) - 1
        } else {
            0
        };
        slots_written +=
            instance_table_pages_after_the_first * ModelUnitRole::InstanceTable.span_in_slots();
        // 有树的一版每次发布都重写中央映射树（根恒在 `rewritten_roles` 里）与分配记录树；一次至多重写这一版的全部节点。
        // 树表 0 条的一版没有映射树，分配记录树照写行那次重写不重写算。
        let central_mapping_tree_nodes_upper_bound = if contents.has_trees() {
            assert!(
                rewritten.contains(&ModelUnitRole::MappingTree)
                    && rewrites_the_allocation_record_tree,
                "有树的一版每次发布都重写映射树与分配记录树：{kind:?}"
            );
            let (mapping_nodes, allocation_record_tree_nodes) = self
                .central_mapping_and_allocation_record_tree_nodes_upper_bounds(
                    previous.central_mapping_tree_nodes_upper_bound.max(1),
                    central_mapping_entries_other_than_the_allocation_record_tree(
                        &contents,
                        &role_written_at,
                        accounting_tree_nodes,
                        checkpoint_txg,
                    ) + central_mapping_entries_of_the_files_other_than_the_first(
                        &inodes_other_than_the_first,
                    ) + snapshot_bookkeeping.pinned_mapping_entries
                        + snapshot_bookkeeping.deadlist_tree_nodes_upper_bound,
                    previous.occupied_slots_upper_bound_per_device + slots_written,
                );
            slots_written += (mapping_nodes - 1) * ModelUnitRole::MappingTree.span_in_slots()
                + allocation_record_tree_nodes * ModelUnitRole::AllocationTree.span_in_slots();
            mapping_nodes
        } else {
            if rewrites_the_allocation_record_tree {
                slots_written += self.allocation_record_tree_nodes_rewritten_upper_bound(
                    previous.occupied_slots_upper_bound_per_device + slots_written,
                ) * ModelUnitRole::AllocationTree.span_in_slots();
            }
            0
        };
        let highest_transaction_number_before_this_publish = if previous.key.instance == instance {
            previous.highest_transaction_number_in_its_instance
        } else {
            0
        };
        let transactions_of_this_publish = kind.transactions_carried(
            data_unit_change.rewritten.end - data_unit_change.rewritten.start
                + data_units_written_to_the_file_other_than_the_first,
        );
        let highest_transaction_number_of_its_publish = if transactions_of_this_publish > 0 {
            highest_transaction_number_before_this_publish + transactions_of_this_publish
        } else {
            0
        };
        // 一个事务一条记录（C310（事务切分纪律与记录数口径打架） 用户定案「一条记录装一个事务连同它的元数据」），不承载事务的发布也写一条；
        // jsn 一条记录加一（D23（journal 的角色与格式） 已定项 7），根带的是这次发布末条记录的 jsn。`journal_counter` 是这次第一条记录的。
        // 末条再跨记录（D23 已定项 17）只在最后一个事务要点名的项多于一条记录装得下的 64 项时才有，模型罩的发布点名的项远不到。
        let records_of_this_publish = transactions_of_this_publish.max(1);
        // logged_ops 照抄上一版（回退那次的 `previous` 是现行那一版：随现行那一版），删意图的几种删空；签发那一条由
        // [`IdealModel::answer_compaction_publish`] 装上。
        let logged_compaction_intent = if kind
            .removed_roles()
            .contains(&ModelUnitRole::LoggedOperationsRoot)
        {
            None
        } else {
            previous.logged_compaction_intent
        };
        // 搬过的根单元：照抄上一版的，只留这一版还引用着的；这一批搬的由 [`IdealModel::answer_compaction_publish`] 记上。
        // 回退那次只取目标那一版的（判决 `research/prompts/m4-segment-code-r2-main-verification.md` 第三节 H1）：实现回退时树表里用户可见树
        // 那几条根指针整条取目标那一版的字节，目标那一版之后才搬的根单元在新根里仍指搬之前的位置，现行那一版「最后一次搬它」的记录不并进来。
        let mut relocations_carried = match user_visible_trees {
            UserVisibleTreesOfTheNewVersion::AsThisPublishLeavesThem => {
                previous.user_visible_root_relocations.as_ref().clone()
            }
            UserVisibleTreesOfTheNewVersion::TakenFromTheRollbackTarget(target_root) => {
                target_root.user_visible_root_relocations.as_ref().clone()
            }
        };
        let mut root = ModelRoot {
            key,
            journal_counter: ModelJournalCounter(journal_counter.0 + records_of_this_publish - 1),
            rollback_floor,
            contents,
            instance_table_rows,
            role_written_at,
            data_unit_written_at: Rc::new(data_unit_written_at),
            occupied_slots_upper_bound_per_device: previous.occupied_slots_upper_bound_per_device
                + slots_written,
            central_mapping_tree_nodes_upper_bound,
            inode_number_watermark_steps,
            accounting_rows,
            accounting_tree_nodes_upper_bound: accounting_tree_nodes,
            highest_transaction_number_of_its_publish,
            highest_transaction_number_in_its_instance:
                highest_transaction_number_before_this_publish
                    .max(highest_transaction_number_of_its_publish),
            snapshots: snapshot_bookkeeping.snapshots,
            previous_snapshot_txg_of_the_live_head: snapshot_bookkeeping
                .previous_snapshot_txg_of_the_live_head,
            deadlist: snapshot_bookkeeping.deadlist,
            pinned_mapping_entries: snapshot_bookkeeping.pinned_mapping_entries,
            deadlist_tree_nodes_upper_bound: snapshot_bookkeeping.deadlist_tree_nodes_upper_bound,
            extent_lower_node_written_at: snapshot_bookkeeping.extent_lower_node_written_at,
            logged_compaction_intent,
            user_visible_root_relocations: Rc::new(BTreeMap::new()),
            inodes_other_than_the_first: Rc::new(inodes_other_than_the_first),
            inode_number_watermark,
            death_txg_of_the_first_file,
        };
        let referenced: BTreeSet<ModelUserVisibleRootUnit> =
            root.user_visible_root_units().into_iter().collect();
        relocations_carried.retain(|unit, _| referenced.contains(unit));
        root.user_visible_root_relocations = Rc::new(relocations_carried);
        root
    }

    /// 这次发布之后这一版的快照那几样（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3、S4 与 S7）：
    /// - 这次换下的用户可见单元（[`ModelUnitRole::asks_the_snapshot_lifetime_judgement`] 那几类，加 extent 树下段被换下的节点）按出生 txg 与
    ///   上一版活头的 `previous_snapshot_txg`（p）判：出生 txg ≤ p 的进 deadlist（纪元 = p）、留一条映射条目，其余照旧释放（D5 已定项 19）。
    ///   换下的数据单元（[`DataUnitChangeOfThePublish::replaced`]）：覆盖写换下上一版的全部、不是最后一批的删除批与截断的批从尾部放 144 个、
    ///   最后一批放剩下的、截断意图与不缩短的截断换下重写的那一个尾单元；第一个文件版本没有上一版，什么都不换下。每个按它自己的出生 txg 判
    ///   （判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K4）。
    /// - 建快照追加一个，活头前驱换成这次的 txg；删快照的意图给第 `position` 个带上「销毁中」、把它的前驱传给下一侧；每一批放掉它那一段
    ///   （纪元落在 [它的诞生 txg, 下一侧的诞生 txg 或这一批的 txg)、出生 txg 晚于它的前驱）里的条目，最后一批放掉剩下的、删掉它。
    /// - 管理员回退那次：回退在现行那一版或目标那一版带快照时拒（判决 S8），走到这里两版都没有快照、活头前驱是 0，一个单元都不钉；
    ///   extent 树下段取目标那一版的。
    ///
    /// `extent_upper_nodes_replaced`：extent 上段按位置换下的节点；`replaced_units_besides_the_first_file`：这次换下的第一个文件之外那个文件的数据单元与
    /// 下段节点（判决 m4-multi-inode-size-r1 第二节 S3、S5：快照下别的文件换下的单元同样判钉不钉）。回退那次两样都不用（两版都没有快照）。
    #[allow(
        clippy::too_many_arguments,
        reason = "一次发布换下的单元按来处各是一个参数，收成结构体只会多一层没人验的名字"
    )]
    fn snapshot_bookkeeping_of_the_publish(
        previous: &ModelRoot,
        kind: ModelPublishKind,
        checkpoint_txg: ModelCheckpointTxg,
        user_visible_trees: UserVisibleTreesOfTheNewVersion<'_>,
        data_unit_change: &DataUnitChangeOfThePublish,
        extent_upper_nodes_replaced: Vec<ModelReplacedUnit>,
        replaced_units_besides_the_first_file: Vec<ModelReplacedUnit>,
    ) -> SnapshotBookkeepingOfThePublish {
        let mut snapshots = previous.snapshots.as_ref().clone();
        let mut previous_snapshot_txg_of_the_live_head =
            previous.previous_snapshot_txg_of_the_live_head;
        let mut deadlist = previous.deadlist.as_ref().clone();
        let mut pinned_mapping_entries = previous.pinned_mapping_entries;
        let (extent_lower_node_written_at, births_of_the_replaced_units) = match user_visible_trees
        {
            UserVisibleTreesOfTheNewVersion::TakenFromTheRollbackTarget(target_root) => {
                assert!(
                    previous.snapshots.is_empty() && target_root.snapshots.is_empty(),
                    "回退在现行那一版或目标那一版带快照时拒（`answer_rollback_while_mounted` 判过）：现行 {:?}，目标 {:?}",
                    previous.snapshots,
                    target_root.snapshots
                );
                (
                    Rc::clone(&target_root.extent_lower_node_written_at),
                    Vec::new(),
                )
            }
            UserVisibleTreesOfTheNewVersion::AsThisPublishLeavesThem => {
                let (extent_lower_node_written_at, mut births) =
                    Self::units_replaced_by_the_publish(
                        previous,
                        kind,
                        checkpoint_txg,
                        data_unit_change,
                        extent_upper_nodes_replaced,
                    );
                births.extend(replaced_units_besides_the_first_file);
                (extent_lower_node_written_at, births)
            }
        };
        let live_head_previous_snapshot_txg = previous.previous_snapshot_txg_of_the_live_head;
        for replaced in births_of_the_replaced_units {
            if replaced.birth_txg <= live_head_previous_snapshot_txg {
                deadlist.push(ModelDeadlistEntry {
                    epoch: live_head_previous_snapshot_txg,
                    mapping_key_unit_class: replaced.mapping_key_unit_class,
                    mapping_key_birth_tree: replaced.mapping_key_birth_tree,
                    birth_txg: replaced.birth_txg,
                });
                pinned_mapping_entries += 1;
            }
        }
        match kind {
            ModelPublishKind::SnapshotCreation => {
                snapshots.push(ModelSnapshot {
                    birth_txg: checkpoint_txg,
                    previous_snapshot_txg: previous.previous_snapshot_txg_of_the_live_head,
                    is_being_destroyed: false,
                    // 码 14 抄的是活头 inode 树的根：截断的批与最后一批只写 extent 树（判决
                    // `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K2），那之后它与用户可见树最近的写者不同。
                    inode_tree_written_by: previous
                        .contents
                        .inode_tree_written_by()
                        .expect("建快照只接在有树的一版后面（`answer_create_snapshot` 判过）"),
                    extent_root_written_at: *previous
                        .role_written_at
                        .get(&ModelUnitRole::ExtentRoot)
                        .expect("有树的一版恒有 extent 树的根（删完的一版是零条目根）"),
                    content_at_creation: previous
                        .contents
                        .live_file()
                        .map(|file| Rc::clone(&file.content)),
                });
                previous_snapshot_txg_of_the_live_head = checkpoint_txg;
            }
            ModelPublishKind::SnapshotDestructionIntent { position } => {
                let destroyed = snapshots
                    .get_mut(position)
                    .expect("意图点名的快照在上一版里（`answer_destroy_snapshot` 判过）");
                destroyed.is_being_destroyed = true;
                let predecessor = destroyed.previous_snapshot_txg;
                match snapshots.get_mut(position + 1) {
                    Some(next_snapshot) => next_snapshot.previous_snapshot_txg = predecessor,
                    None => previous_snapshot_txg_of_the_live_head = predecessor,
                }
            }
            ModelPublishKind::SnapshotDestructionBatch {
                position,
                entries_released,
            } => {
                let released: Vec<ModelDeadlistEntry> =
                    Self::deadlist_entries_the_destruction_releases(
                        previous,
                        position,
                        checkpoint_txg,
                    )
                    .into_iter()
                    .take(usize::try_from(entries_released).expect("一批至多 144 条"))
                    .collect();
                Self::remove_released_entries(&mut deadlist, &released);
                pinned_mapping_entries -= u64::try_from(released.len()).expect("条数");
            }
            ModelPublishKind::SnapshotDestructionFinalBatch { position } => {
                let released = Self::deadlist_entries_the_destruction_releases(
                    previous,
                    position,
                    checkpoint_txg,
                );
                Self::remove_released_entries(&mut deadlist, &released);
                pinned_mapping_entries -= u64::try_from(released.len()).expect("条数");
                snapshots.remove(position);
            }
            ModelPublishKind::FirstFileVersion
            | ModelPublishKind::OverwriteFileVersion
            | ModelPublishKind::RowsOnFileVersion
            | ModelPublishKind::EmptyOnFileVersion
            | ModelPublishKind::RowsOnVersionWithoutFile
            | ModelPublishKind::ZeroUnit
            | ModelPublishKind::UnlinkIntent { .. }
            | ModelPublishKind::DeletionBatchKeepingDataUnits { .. }
            | ModelPublishKind::FinalDeletionBatch { .. }
            | ModelPublishKind::NewInode
            | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_)
            | ModelPublishKind::CompactionBatch { .. }
            | ModelPublishKind::CompactionIntentIssue
            | ModelPublishKind::CompactionIntentDeletion { .. }
            | ModelPublishKind::TruncationIntent { .. }
            | ModelPublishKind::TruncationBatch
            | ModelPublishKind::FinalTruncationBatch { .. }
            | ModelPublishKind::SizeChangeWithoutShrinking { .. }
            | ModelPublishKind::WholeWriteOfAnInodeOtherThanTheFirst { .. }
            | ModelPublishKind::SizeChangeOfAnInodeWithoutDataUnits { .. }
            | ModelPublishKind::SizeChangeOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::TruncationBatchOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::PartialWriteOfTheFirstFile { .. }
            | ModelPublishKind::PartialWriteOfAnInodeOtherThanTheFirst { .. }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeWithoutDataUnits { .. } => {}
        }
        let deadlist_tree_nodes_upper_bound = deadlist_tree_nodes_upper_bound(
            previous.deadlist_tree_nodes_upper_bound,
            u64::try_from(deadlist.len()).expect("条数"),
        );
        let deadlist_tree_nodes_written = if previous.deadlist.is_empty() && deadlist.is_empty() {
            0
        } else {
            deadlist_tree_nodes_upper_bound
        };
        SnapshotBookkeepingOfThePublish {
            snapshots: Rc::new(snapshots),
            previous_snapshot_txg_of_the_live_head,
            deadlist: Rc::new(deadlist),
            pinned_mapping_entries,
            deadlist_tree_nodes_upper_bound,
            deadlist_tree_nodes_written,
            extent_lower_node_written_at,
        }
    }

    /// 一次不是回退的发布对数据单元做了什么（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K2、K3、K5 与 D8（核心索引结构）
    /// 已定项 16 第 2 条）：写内容的整份重写；删除的批与截断的批从尾部放 144 个，删除的最后一批放光、截断的最后一批放到只剩 size 罩的那几个；
    /// 截断意图重写尾单元时写单元 ⌈N ÷ 净荷⌉.max(1) − 1，不缩短的截断补零重写时写最后一个存在的单元；别的发布不动数据单元。
    /// 第一个文件活着时这次发布只换了 inode 树（建 inode）或 inode 树与 extent 树（按 inode 号写第一个文件之外的文件）：第一个文件的内容、单元、
    /// 上段叶条目的标签与截断字段照旧，两棵用户可见树的根指针换成这次写的（[`ModelFileVersion::written_by`]、
    /// [`ModelFileVersion::inode_record_written_by`]：「非空」与建快照抄的码 14 按它们认）。
    fn file_version_whose_trees_this_publish_rewrote(
        previous_file: &ModelFileVersion,
        key: ModelRootKey,
    ) -> ModelFileVersion {
        ModelFileVersion {
            written_by: key,
            inode_record_written_by: key,
            content: Rc::clone(&previous_file.content),
            data_units_present: previous_file.data_units_present,
            extent_upper_leaf_entry_label: previous_file.extent_upper_leaf_entry_label,
            truncation_field: previous_file.truncation_field,
        }
    }

    /// 这次发布让 extent 上段里哪几个号的条目变了（进了有条目的号集合、出了它、或条目内容变了；规范形计划里罩着它们的节点重写，判决
    /// `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M2）：按 inode 号写那个号；删除批删的那个号；最后一批删的那个号
    /// （它本来有条目时，`inodes_with_an_entry_before` 是上一版有条目的号）；意图不动 extent 树；只写第一个文件的那几种（写内容、截断那几样）在
    /// 重写上段根时是 inode 1；别的发布一个都没有。
    fn inodes_whose_extent_upper_leaf_entry_the_publish_changes(
        kind: ModelPublishKind,
        inodes_other_than_the_first_change: InodesOtherThanTheFirstChangeOfThePublish<'_>,
        inodes_with_an_entry_before: &BTreeSet<u64>,
    ) -> BTreeSet<u64> {
        let first_file_when_the_kind_rewrites_the_upper_root = || {
            if kind
                .rewritten_roles_fixed_by_the_kind()
                .contains(&ModelUnitRole::ExtentRoot)
            {
                BTreeSet::from([FIRST_INODE_NUMBER])
            } else {
                BTreeSet::new()
            }
        };
        match kind {
            ModelPublishKind::WholeWriteOfAnInodeOtherThanTheFirst { .. } => {
                match inodes_other_than_the_first_change {
                    InodesOtherThanTheFirstChangeOfThePublish::WholeWrite { inode, .. } => {
                        BTreeSet::from([inode])
                    }
                    InodesOtherThanTheFirstChangeOfThePublish::Carried
                    | InodesOtherThanTheFirstChangeOfThePublish::Created { .. }
                    | InodesOtherThanTheFirstChangeOfThePublish::PartialWrite { .. } => {
                        panic!("按 inode 号写那一种恰好带写了谁（`next_root_with_the_user_visible_trees` 开头断言过）")
                    }
                }
            }
            // 按偏移写别的文件：重写了它的单元，它那一条上段叶条目变了（判决 m4-partial-write-r1 第二节 R4、R6）。
            ModelPublishKind::PartialWriteOfAnInodeOtherThanTheFirst { inode, .. } => {
                BTreeSet::from([inode])
            }
            ModelPublishKind::UnlinkIntent { .. } => BTreeSet::new(),
            // 没有数据单元的 inode 第一次写上粘住的标签 0：它进了有条目的号集合；别的改 size 一条条目都不动（判决 m4-multi-inode-size-r1 第二节 S2）。
            ModelPublishKind::SizeChangeOfAnInodeWithoutDataUnits {
                inode,
                change:
                    ModelSizeChangeOfAnInodeWithoutDataUnits::WithoutShrinkingWritingTheTagZeroEntry,
                ..
            } => BTreeSet::from([inode]),
            ModelPublishKind::SizeChangeOfAnInodeWithoutDataUnits {
                change:
                    ModelSizeChangeOfAnInodeWithoutDataUnits::WithoutShrinkingCarryingTheExtentTree
                    | ModelSizeChangeOfAnInodeWithoutDataUnits::TruncationIntent,
                ..
            }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeWithoutDataUnits { .. } => {
                BTreeSet::new()
            }
            // 带数据单元的那种改 size：尾单元重写或标签要换时它那一条变了（`transaction::SizeChangeOfTheFile::rewrites_the_upper_leaf_entry`
            // 那一读法：有单元重写、单元数变了、或尾部有没有洞变了）；它截断的批与放了单元的最后一批单元数变了。
            ModelPublishKind::SizeChangeOfAnInodeHoldingDataUnits {
                inode,
                extent_tree:
                    ModelExtentTreeChangeOfASizeChange::TailDataUnitRewritten
                    | ModelExtentTreeChangeOfASizeChange::OnlyTheUpperLeafEntryLabel,
                ..
            }
            | ModelPublishKind::TruncationBatchOfAnInodeHoldingDataUnits { inode }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeHoldingDataUnits {
                inode,
                release: ModelFinalTruncationBatchRelease::AtLeastOneDataUnit,
                ..
            } => BTreeSet::from([inode]),
            ModelPublishKind::SizeChangeOfAnInodeHoldingDataUnits {
                extent_tree: ModelExtentTreeChangeOfASizeChange::Untouched,
                ..
            }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeHoldingDataUnits {
                release: ModelFinalTruncationBatchRelease::NoDataUnit,
                ..
            } => BTreeSet::new(),
            ModelPublishKind::DeletionBatchKeepingDataUnits { deleted } => {
                BTreeSet::from([deleted.inode_number()])
            }
            ModelPublishKind::FinalDeletionBatch { deleted, .. } => {
                if inodes_with_an_entry_before.contains(&deleted.inode_number()) {
                    BTreeSet::from([deleted.inode_number()])
                } else {
                    BTreeSet::new()
                }
            }
            ModelPublishKind::FirstFileVersion
            | ModelPublishKind::OverwriteFileVersion
            | ModelPublishKind::RowsOnFileVersion
            | ModelPublishKind::EmptyOnFileVersion
            | ModelPublishKind::RowsOnVersionWithoutFile
            | ModelPublishKind::ZeroUnit
            | ModelPublishKind::NewInode
            | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_)
            | ModelPublishKind::SnapshotCreation
            | ModelPublishKind::SnapshotDestructionIntent { .. }
            | ModelPublishKind::SnapshotDestructionBatch { .. }
            | ModelPublishKind::SnapshotDestructionFinalBatch { .. }
            | ModelPublishKind::CompactionBatch { .. }
            | ModelPublishKind::CompactionIntentIssue
            | ModelPublishKind::CompactionIntentDeletion { .. }
            | ModelPublishKind::TruncationIntent { .. }
            | ModelPublishKind::TruncationBatch
            | ModelPublishKind::FinalTruncationBatch { .. }
            | ModelPublishKind::SizeChangeWithoutShrinking { .. }
            | ModelPublishKind::PartialWriteOfTheFirstFile { .. } => {
                first_file_when_the_kind_rewrites_the_upper_root()
            }
        }
    }

    /// 第一个文件已删完（树里没有 inode 1 的记录）的一版：树里还有 `inode_records_in_the_tree` 条别的 inode 的记录时是带文件的一版
    /// （D8（核心索引结构） 已定项 16 第 5 条，判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M3：inode 树还有任一记录的一版
    /// 是带文件的一版，不管 inode 1 在不在），一条都没有才是有树而文件集合为空的一版。`key` 是写出这一版 inode 树的那次发布。
    fn contents_without_the_first_file(
        key: ModelRootKey,
        inode_records_in_the_tree: u64,
    ) -> ModelVersionContents {
        if inode_records_in_the_tree == 0 {
            ModelVersionContents::TreesAndNoFile {
                trees_written_by: key,
            }
        } else {
            ModelVersionContents::InodesWithoutTheFirstFile {
                trees_written_by: key,
                inode_tree_written_by: key,
                inode_records_in_the_tree,
            }
        }
    }

    /// 一次只动第一个文件之外的一个 inode 的发布（按 inode 号写、删它的意图、批与最后一批）之后这一版里有什么：第一个文件活着时它的内容、
    /// 单元、标签与截断字段照旧，两棵用户可见树的根指针换成这次写的；第一个文件已删完时按这次之后树里还有几条记录
    /// （`inode_records_in_the_tree_after`，就是第一个文件之外的 inode 数）归带文件的一版或有树而文件集合为空的一版。
    ///
    /// # Panics
    /// 上一版不是这两种（树表 0 条、有树而文件集合为空、第一个文件删除中或截断途中）：答法只在这两种上发这几种发布（会话先走空）。
    /// 第一个文件之外一个 inode 截断的批与放了单元的最后一批之后这一版里有什么（K2「批不碰 inode 记录」，判决 m4-multi-inode-size-r1 第二节 S3）：
    /// 只写 extent 树，第一个文件活着时它的内容、单元、标签与截断字段照旧、只有 extent 树那条根指针换成这次写的
    /// （[`ModelFileVersion::written_by`] 换、[`ModelFileVersion::inode_record_written_by`] 照抄）；第一个文件已删完时同样只换
    /// `trees_written_by`、inode 树的写者照抄。
    ///
    /// # Panics
    /// 上一版不是这两种：截断条目点名别的 inode 只在第一个文件活着、或已删完而树里还有别的 inode 的一版上（会话先走空）。
    fn contents_whose_extent_tree_a_truncation_batch_of_another_inode_rewrote(
        previous_contents: &ModelVersionContents,
        key: ModelRootKey,
        kind: ModelPublishKind,
    ) -> ModelVersionContents {
        match previous_contents {
            ModelVersionContents::LiveFile(previous_file) => {
                ModelVersionContents::LiveFile(ModelFileVersion {
                    written_by: key,
                    inode_record_written_by: previous_file.inode_record_written_by,
                    content: Rc::clone(&previous_file.content),
                    data_units_present: previous_file.data_units_present,
                    extent_upper_leaf_entry_label: previous_file.extent_upper_leaf_entry_label,
                    truncation_field: previous_file.truncation_field,
                })
            }
            ModelVersionContents::InodesWithoutTheFirstFile {
                inode_tree_written_by,
                inode_records_in_the_tree,
                ..
            } => ModelVersionContents::InodesWithoutTheFirstFile {
                trees_written_by: key,
                inode_tree_written_by: *inode_tree_written_by,
                inode_records_in_the_tree: *inode_records_in_the_tree,
            },
            ModelVersionContents::WithoutTrees
            | ModelVersionContents::TreesAndNoFile { .. }
            | ModelVersionContents::FileBeingDeleted { .. }
            | ModelVersionContents::FileBeingTruncated { .. } => panic!(
                "别的 inode 截断的批只接在第一个文件活着、或已删完而树里还有别的 inode 的一版后面（`answer_truncation_step` 判过）：{kind:?} 接在 {previous_contents:?}"
            ),
        }
    }

    fn contents_whose_trees_a_change_of_another_inode_rewrote(
        previous_contents: &ModelVersionContents,
        key: ModelRootKey,
        inode_records_in_the_tree_after: u64,
        kind: ModelPublishKind,
    ) -> ModelVersionContents {
        match previous_contents {
            ModelVersionContents::LiveFile(previous_file) => ModelVersionContents::LiveFile(
                Self::file_version_whose_trees_this_publish_rewrote(previous_file, key),
            ),
            ModelVersionContents::InodesWithoutTheFirstFile { .. } => {
                Self::contents_without_the_first_file(key, inode_records_in_the_tree_after)
            }
            ModelVersionContents::WithoutTrees
            | ModelVersionContents::TreesAndNoFile { .. }
            | ModelVersionContents::FileBeingDeleted { .. }
            | ModelVersionContents::FileBeingTruncated { .. } => panic!(
                "只动第一个文件之外一个 inode 的发布只接在第一个文件活着、或已删完而树里还有别的 inode 的一版后面（答法判过）：{kind:?} 接在 {previous_contents:?}"
            ),
        }
    }

    /// 一次不是回退的发布对第一个文件之外哪个文件的数据单元做了什么（同第一个文件那一份 [`IdealModel::data_unit_change_of_the_publish`]，判决
    /// `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S3）：按 inode 号整份写那个文件全部重写；带数据单元的那种改 size 截断意图重写
    /// 单元 ⌈N ÷ 净荷⌉.max(1) − 1、不缩短的补零重写最后一个存在的单元（尾单元要重写时），一个都不放；它截断的批从尾部放 144 个、最后一批放到只剩 size
    /// 罩的那几个；删它的批从尾部放 144 个、最后一批放光。别的发布不动第一个文件之外的单元，交 `None`。
    ///
    /// # Panics
    /// 点名的号不在上一版里（答法判过），或按 inode 号写那一种没带写了谁。
    fn data_unit_change_of_an_inode_other_than_the_first(
        previous: &ModelRoot,
        kind: ModelPublishKind,
        inodes_other_than_the_first_change: InodesOtherThanTheFirstChangeOfThePublish<'_>,
    ) -> Option<(u64, DataUnitChangeOfThePublish)> {
        let inode_before = |inode: u64| {
            previous
                .inodes_other_than_the_first
                .get(&inode)
                .expect("这次动的号是上一版树里第一个文件之外的号（答法判过）")
        };
        let keeping_the_count = |inode: u64, rewritten: std::ops::Range<u64>| {
            let data_units_before = inode_before(inode).data_units_present();
            (
                inode,
                DataUnitChangeOfThePublish {
                    data_units_before,
                    data_units_after: data_units_before,
                    rewritten,
                },
            )
        };
        let releasing_down_to = |inode: u64, data_units_after: u64| {
            (
                inode,
                DataUnitChangeOfThePublish {
                    data_units_before: inode_before(inode).data_units_present(),
                    data_units_after,
                    rewritten: 0..0,
                },
            )
        };
        match kind {
            ModelPublishKind::WholeWriteOfAnInodeOtherThanTheFirst { .. } => {
                let InodesOtherThanTheFirstChangeOfThePublish::WholeWrite { inode, content } =
                    inodes_other_than_the_first_change
                else {
                    panic!("按 inode 号写那一种恰好带写了谁（`next_root_with_the_user_visible_trees` 开头断言过）");
                };
                let data_units_after =
                    data_units_of_content(u64::try_from(content.len()).expect("内容长度"));
                Some((
                    inode,
                    DataUnitChangeOfThePublish {
                        data_units_before: inode_before(inode).data_units_present(),
                        data_units_after,
                        rewritten: 0..data_units_after,
                    },
                ))
            }
            ModelPublishKind::SizeChangeOfAnInodeHoldingDataUnits {
                inode,
                new_size_in_bytes,
                direction,
                extent_tree,
                ..
            } => {
                let data_units_before = inode_before(inode).data_units_present();
                let rewritten = match (extent_tree, direction) {
                    (
                        ModelExtentTreeChangeOfASizeChange::TailDataUnitRewritten,
                        ModelDirectionOfASizeChange::TruncationIntent,
                    ) => {
                        let tail_data_unit = data_units_of_content(new_size_in_bytes) - 1;
                        tail_data_unit..tail_data_unit + 1
                    }
                    (
                        ModelExtentTreeChangeOfASizeChange::TailDataUnitRewritten,
                        ModelDirectionOfASizeChange::WithoutShrinking,
                    ) => data_units_before - 1..data_units_before,
                    (
                        ModelExtentTreeChangeOfASizeChange::OnlyTheUpperLeafEntryLabel
                        | ModelExtentTreeChangeOfASizeChange::Untouched,
                        ModelDirectionOfASizeChange::TruncationIntent
                        | ModelDirectionOfASizeChange::WithoutShrinking,
                    ) => 0..0,
                };
                Some(keeping_the_count(inode, rewritten))
            }
            // 按偏移写（判决 m4-partial-write-r1 第二节 R4、R5）：从 `first_rewritten_data_unit` 起到写之后最后一个单元一起重写，一个都不放。
            ModelPublishKind::PartialWriteOfAnInodeOtherThanTheFirst {
                inode,
                first_rewritten_data_unit,
                data_units_after,
                ..
            } => Some((
                inode,
                DataUnitChangeOfThePublish {
                    data_units_before: inode_before(inode).data_units_present(),
                    data_units_after,
                    rewritten: first_rewritten_data_unit..data_units_after,
                },
            )),
            ModelPublishKind::TruncationBatchOfAnInodeHoldingDataUnits { inode }
            | ModelPublishKind::DeletionBatchKeepingDataUnits {
                deleted: ModelDeletedInode::OtherThanTheFirst(inode),
            } => Some(releasing_down_to(
                inode,
                inode_before(inode).data_units_present() - MODEL_DELETION_BATCH_DATA_UNITS,
            )),
            ModelPublishKind::FinalTruncationBatchOfAnInodeHoldingDataUnits { inode, .. } => {
                let truncated = inode_before(inode);
                Some(releasing_down_to(
                    inode,
                    truncated
                        .data_units_present()
                        .min(truncated.data_units_within_the_size()),
                ))
            }
            ModelPublishKind::FinalDeletionBatch {
                deleted: ModelDeletedInode::OtherThanTheFirst(inode),
                ..
            } => Some(releasing_down_to(inode, 0)),
            ModelPublishKind::FirstFileVersion
            | ModelPublishKind::OverwriteFileVersion
            | ModelPublishKind::RowsOnFileVersion
            | ModelPublishKind::EmptyOnFileVersion
            | ModelPublishKind::RowsOnVersionWithoutFile
            | ModelPublishKind::ZeroUnit
            | ModelPublishKind::UnlinkIntent { .. }
            | ModelPublishKind::DeletionBatchKeepingDataUnits {
                deleted: ModelDeletedInode::FirstFile,
            }
            | ModelPublishKind::FinalDeletionBatch {
                deleted: ModelDeletedInode::FirstFile,
                ..
            }
            | ModelPublishKind::NewInode
            | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_)
            | ModelPublishKind::SnapshotCreation
            | ModelPublishKind::SnapshotDestructionIntent { .. }
            | ModelPublishKind::SnapshotDestructionBatch { .. }
            | ModelPublishKind::SnapshotDestructionFinalBatch { .. }
            | ModelPublishKind::CompactionBatch { .. }
            | ModelPublishKind::CompactionIntentIssue
            | ModelPublishKind::CompactionIntentDeletion { .. }
            | ModelPublishKind::TruncationIntent { .. }
            | ModelPublishKind::TruncationBatch
            | ModelPublishKind::FinalTruncationBatch { .. }
            | ModelPublishKind::SizeChangeWithoutShrinking { .. }
            | ModelPublishKind::SizeChangeOfAnInodeWithoutDataUnits { .. }
            | ModelPublishKind::PartialWriteOfTheFirstFile { .. }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeWithoutDataUnits { .. } => None,
        }
    }

    fn data_unit_change_of_the_publish(
        previous: &ModelRoot,
        kind: ModelPublishKind,
        new_file_content: Option<&[u8]>,
    ) -> DataUnitChangeOfThePublish {
        let data_units_before = u64::try_from(previous.data_unit_written_at.len()).expect("单元数");
        let unchanged = DataUnitChangeOfThePublish {
            data_units_before,
            data_units_after: data_units_before,
            rewritten: 0..0,
        };
        let data_units_of_the_new_content = || {
            data_units_of_content(
                u64::try_from(
                    new_file_content
                        .expect("写内容的、截断意图与不缩短的截断带读回的整份")
                        .len(),
                )
                .expect("内容长度"),
            )
        };
        match kind {
            ModelPublishKind::FirstFileVersion | ModelPublishKind::OverwriteFileVersion => {
                let data_units_after = data_units_of_the_new_content();
                DataUnitChangeOfThePublish {
                    data_units_before,
                    data_units_after,
                    rewritten: 0..data_units_after,
                }
            }
            // 按偏移写第一个文件（判决 m4-partial-write-r1 第二节 R4、R5）：从 `first_rewritten_data_unit` 起到写之后最后一个单元一起重写，一个都不放。
            ModelPublishKind::PartialWriteOfTheFirstFile {
                first_rewritten_data_unit,
                data_units_after,
            } => DataUnitChangeOfThePublish {
                data_units_before,
                data_units_after,
                rewritten: first_rewritten_data_unit..data_units_after,
            },
            ModelPublishKind::DeletionBatchKeepingDataUnits {
                deleted: ModelDeletedInode::FirstFile,
            }
            | ModelPublishKind::TruncationBatch => DataUnitChangeOfThePublish {
                data_units_before,
                data_units_after: data_units_before - MODEL_DELETION_BATCH_DATA_UNITS,
                rewritten: 0..0,
            },
            ModelPublishKind::FinalDeletionBatch {
                deleted: ModelDeletedInode::FirstFile,
                ..
            } => DataUnitChangeOfThePublish {
                data_units_before,
                data_units_after: 0,
                rewritten: 0..0,
            },
            // 删第一个文件之外的号：这里只记第一个文件的单元，它们一个不动（那个号的单元记在它自己那一项里）。
            ModelPublishKind::DeletionBatchKeepingDataUnits {
                deleted: ModelDeletedInode::OtherThanTheFirst(_),
            }
            | ModelPublishKind::FinalDeletionBatch {
                deleted: ModelDeletedInode::OtherThanTheFirst(_),
                ..
            } => unchanged,
            // 截回洞里的（存在的单元不多于 size 罩的）一个都不放（K5、K11）。
            ModelPublishKind::FinalTruncationBatch { .. } => DataUnitChangeOfThePublish {
                data_units_before,
                data_units_after: data_units_before.min(
                    previous
                        .contents
                        .live_file()
                        .expect("截断的最后一批只接在截断途中的那一版后面（`answer_truncation_step` 判过）")
                        .data_units_within_the_size(),
                ),
                rewritten: 0..0,
            },
            ModelPublishKind::TruncationIntent {
                extent_tree: ModelExtentTreeChangeOfASizeChange::TailDataUnitRewritten,
            } => {
                let tail_data_unit = data_units_of_the_new_content() - 1;
                DataUnitChangeOfThePublish {
                    data_units_before,
                    data_units_after: data_units_before,
                    rewritten: tail_data_unit..tail_data_unit + 1,
                }
            }
            ModelPublishKind::SizeChangeWithoutShrinking {
                extent_tree: ModelExtentTreeChangeOfASizeChange::TailDataUnitRewritten,
            } => DataUnitChangeOfThePublish {
                data_units_before,
                data_units_after: data_units_before,
                rewritten: data_units_before - 1..data_units_before,
            },
            ModelPublishKind::TruncationIntent {
                extent_tree:
                    ModelExtentTreeChangeOfASizeChange::OnlyTheUpperLeafEntryLabel
                    | ModelExtentTreeChangeOfASizeChange::Untouched,
            }
            | ModelPublishKind::SizeChangeWithoutShrinking {
                extent_tree:
                    ModelExtentTreeChangeOfASizeChange::OnlyTheUpperLeafEntryLabel
                    | ModelExtentTreeChangeOfASizeChange::Untouched,
            }
            | ModelPublishKind::RowsOnFileVersion
            | ModelPublishKind::EmptyOnFileVersion
            | ModelPublishKind::RowsOnVersionWithoutFile
            | ModelPublishKind::ZeroUnit
            | ModelPublishKind::UnlinkIntent { .. }
            | ModelPublishKind::NewInode
            | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_)
            | ModelPublishKind::SnapshotCreation
            | ModelPublishKind::SnapshotDestructionIntent { .. }
            | ModelPublishKind::SnapshotDestructionBatch { .. }
            | ModelPublishKind::SnapshotDestructionFinalBatch { .. }
            | ModelPublishKind::CompactionBatch { .. }
            | ModelPublishKind::CompactionIntentIssue
            | ModelPublishKind::CompactionIntentDeletion { .. }
            // 按 inode 号写第一个文件之外的文件：这里只记第一个文件的单元，它们一个不动。
            | ModelPublishKind::WholeWriteOfAnInodeOtherThanTheFirst { .. }
            // 没有数据单元的 inode 改 size：一个单元都不动（第一个文件的也不动）。
            | ModelPublishKind::SizeChangeOfAnInodeWithoutDataUnits { .. }
            | ModelPublishKind::SizeChangeOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::TruncationBatchOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeHoldingDataUnits { .. }
            // 按偏移写第一个文件之外的文件：这里只记第一个文件的单元，它们一个不动。
            | ModelPublishKind::PartialWriteOfAnInodeOtherThanTheFirst { .. }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeWithoutDataUnits { .. } => unchanged,
        }
    }

    /// 一次不是回退的发布之后第一个文件 extent 树下段每个节点的分配代，与这次换下的用户可见单元各自的出生 txg（一个单元一项）：第一个文件的数据单元与
    /// 下段节点（[`IdealModel::data_units_and_lower_nodes_after_a_data_unit_change`]）、extent 上段按位置换下的节点（`extent_upper_nodes_replaced`，
    /// 调用方在 `next_root_with_the_user_visible_trees` 里认好，根与根之下的一样），与这次重写或删掉的 inode 树那几个角色；第一个文件之外的文件的
    /// 单元另算，由调用方并进来。
    fn units_replaced_by_the_publish(
        previous: &ModelRoot,
        kind: ModelPublishKind,
        checkpoint_txg: ModelCheckpointTxg,
        data_unit_change: &DataUnitChangeOfThePublish,
        extent_upper_nodes_replaced: Vec<ModelReplacedUnit>,
    ) -> (
        Rc<BTreeMap<ModelExtentLowerNodePosition, ModelCheckpointTxg>>,
        Vec<ModelReplacedUnit>,
    ) {
        let (lower_nodes_after, mut births) =
            Self::data_units_and_lower_nodes_after_a_data_unit_change(
                &previous.data_unit_written_at,
                &previous.extent_lower_node_written_at,
                data_unit_change,
                checkpoint_txg,
            );
        // extent 上段换下的节点（按位置认，调用方算好的），排在 inode 树那几个角色之前（同按角色次序认上段根时的次序）。
        births.extend(extent_upper_nodes_replaced);
        // 别的用户可见角色：这次重写或删掉、上一版里有的（第一个文件版本那一版上没有它们）。数据单元上面按个数另数，extent 上段的节点按位置另认。
        let roles_rewritten_or_removed: BTreeSet<ModelUnitRole> = kind
            .rewritten_roles()
            .into_iter()
            .chain(kind.removed_roles().iter().copied())
            .filter(|role| {
                role.asks_the_snapshot_lifetime_judgement()
                    && *role != ModelUnitRole::Data
                    && !role.is_a_node_of_the_extent_upper_segment()
            })
            .collect();
        births.extend(roles_rewritten_or_removed.iter().filter_map(|role| {
            let birth_txg = previous.role_written_at.get(role).copied()?;
            let (mapping_key_unit_class, mapping_key_birth_tree) = role
                .mapping_key_unit_class_and_birth_tree()
                .expect("问快照生死判定的角色都有映射 key 的类标签与出生树");
            Some(ModelReplacedUnit {
                mapping_key_unit_class,
                mapping_key_birth_tree,
                birth_txg,
            })
        }));
        (Rc::new(lower_nodes_after), births)
    }

    /// 一个文件这次的数据单元变化（`data_unit_change`）之后它的 extent 树下段每个节点的分配代，与这次换下的数据单元和下段节点各自的出生 txg
    /// （一个单元一项）：第一个文件与第一个文件之外的文件各交自己上一版的单元（`previous_data_unit_written_at`）与下段（`previous_lower_nodes`）。
    ///
    /// 下段节点按「内容变了才写」（COW）：这次之后存在的单元的每个位置，上一版没有的、罩着这次写的单元的、罩着这次从尾部放掉的单元的
    /// （孩子少了），这次写；别的照抄上一版的分配代。写内容的整份重写因此整段重写；删除批只重写被截掉那一片下段叶与它的祖先（K-R3-1）；
    /// 截断意图重写罩着尾单元 k 的那一片到根（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K3「把罩着被重写单元 k 的那片
    /// 下段叶到根恒并进重写集」，留下的单元数是 144 的倍数时也一样）；只剩一个单元时不建下段（D8（核心索引结构） 已定项 14），整段换下。
    fn data_units_and_lower_nodes_after_a_data_unit_change(
        previous_data_unit_written_at: &[ModelCheckpointTxg],
        previous_lower_nodes: &BTreeMap<ModelExtentLowerNodePosition, ModelCheckpointTxg>,
        data_unit_change: &DataUnitChangeOfThePublish,
        checkpoint_txg: ModelCheckpointTxg,
    ) -> (
        BTreeMap<ModelExtentLowerNodePosition, ModelCheckpointTxg>,
        Vec<ModelReplacedUnit>,
    ) {
        let rewritten = &data_unit_change.rewritten;
        let lower_nodes_after: BTreeMap<ModelExtentLowerNodePosition, ModelCheckpointTxg> =
            extent_lower_node_positions(data_unit_change.data_units_after)
                .into_iter()
                .map(|position| {
                    let first_data_unit = first_data_unit_under(position);
                    let last_data_unit = last_data_unit_under(position);
                    let covers_a_rewritten_data_unit = !rewritten.is_empty()
                        && rewritten.start <= last_data_unit
                        && first_data_unit < rewritten.end;
                    let covers_a_released_data_unit = data_unit_change.data_units_after
                        < data_unit_change.data_units_before
                        && last_data_unit >= data_unit_change.data_units_after;
                    let written_at = match previous_lower_nodes.get(&position) {
                        Some(previous_written_at)
                            if !covers_a_rewritten_data_unit && !covers_a_released_data_unit =>
                        {
                            *previous_written_at
                        }
                        Some(_) | None => checkpoint_txg,
                    };
                    (position, written_at)
                })
                .collect();
        // 换下的数据单元各按自己的出生 txg（K4：截断只重写尾单元之后一个文件的数据单元不再同一个出生 txg）。
        let mut births: Vec<ModelReplacedUnit> = data_unit_change
            .replaced()
            .map(|index| ModelReplacedUnit {
                mapping_key_unit_class: MODEL_MAPPING_KEY_UNIT_CLASS_DATA,
                mapping_key_birth_tree: TREE_IDENTIFIER_EXTENT,
                birth_txg: *previous_data_unit_written_at
                    .get(usize::try_from(index).expect("单元号装得进 usize"))
                    .expect("换下的单元是上一版存在的"),
            })
            .collect();
        // 下段节点（extent 树的索引节点）：上一版有、这一版同一个位置上分配代没变的照抄，别的都是这次换下的（整段重写、截掉、删光）。
        births.extend(
            previous_lower_nodes
                .iter()
                .filter(|(position, written_at)| {
                    lower_nodes_after.get(*position) != Some(*written_at)
                })
                .map(|(_, written_at)| ModelReplacedUnit {
                    mapping_key_unit_class: MODEL_MAPPING_KEY_UNIT_CLASS_INDEX_NODE,
                    mapping_key_birth_tree: TREE_IDENTIFIER_EXTENT,
                    birth_txg: *written_at,
                }),
        );
        (lower_nodes_after, births)
    }

    /// 删按建的先后第 `position` 个快照、这一批（txg `txg_of_this_batch`）能放的 deadlist 条目（判决 S4）：纪元落在 [lo, hi) 里、出生 txg 晚于
    /// 它的前驱 P——lo = 它的诞生 txg，hi = 下一侧是快照时取那一个的诞生 txg、是活头时取这一批的 txg（实现取的那一个，丙 2 主体交回的读法第 3 条）。
    /// 按盘上 key 的序排（[`ModelDeadlistEntry`] 的字段次序：纪元、映射 key 的类标签、出生树、出生 txg）：不是最后一批的一批放排在最前的 144 条，
    /// 与实现「按 key 升序的前一批」逐条对得上（映射 key 末段模型不记，前三段相同的几条互相顶替）。
    fn deadlist_entries_the_destruction_releases(
        version: &ModelRoot,
        position: usize,
        txg_of_this_batch: ModelCheckpointTxg,
    ) -> Vec<ModelDeadlistEntry> {
        let destroyed = version
            .snapshots
            .get(position)
            .expect("删的快照在这一版里（`answer_snapshot_destruction_step` 判过）");
        let lowest_epoch = destroyed.birth_txg;
        let epoch_beyond = version
            .snapshots
            .get(position + 1)
            .map_or(txg_of_this_batch, |next_snapshot| next_snapshot.birth_txg);
        let mut releasable: Vec<ModelDeadlistEntry> = version
            .deadlist
            .iter()
            .filter(|entry| {
                (lowest_epoch..epoch_beyond).contains(&entry.epoch)
                    && entry.birth_txg > destroyed.previous_snapshot_txg
            })
            .copied()
            .collect();
        releasable.sort_unstable();
        releasable
    }

    /// 从 deadlist 里拿掉放掉的那几条（同样的条目可以有几条：同一次发布写的几个数据单元钉在同一个纪元上，各拿一条）。
    fn remove_released_entries(
        deadlist: &mut Vec<ModelDeadlistEntry>,
        released: &[ModelDeadlistEntry],
    ) {
        for entry in released {
            let index = deadlist
                .iter()
                .position(|kept| kept == entry)
                .expect("放掉的条目是从这一版的 deadlist 里挑的");
            deadlist.swap_remove(index);
        }
    }

    fn upper_bounds_of(roots: &[ModelRoot]) -> Vec<PlannedPublishUpperBounds> {
        roots
            .iter()
            .map(|root| PlannedPublishUpperBounds {
                occupied_slots_per_device: root.occupied_slots_upper_bound_per_device,
            })
            .collect()
    }

    fn open_session(&self) -> Result<&ModelSession, ModelDisagreement> {
        self.session.as_ref().ok_or_else(|| {
            ModelDisagreement::new(
                ModelDisagreementAspect::SessionState,
                "模型里这个进程没有可写会话".to_string(),
                "实现的会话开着（执行器调了要会话的入口）".to_string(),
            )
        })
    }

    /// 发布第一个文件版本（`transaction::publish_first_file`）。
    ///
    /// # Errors
    /// 模型里没有可写会话：会话状态与实现对不上。
    pub fn answer_publish_first_file(
        &self,
        content: &[u8],
    ) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        let mut required_refusals = BTreeSet::new();
        // 第一个文件版本接在现行那一版后面：txg 与 jsn 各加一，不取 `NEW_POOL_FILE_CREATION_TXG`——那个常量只管 mkfs
        // 同一个进程里那条流（2026-09-23 用户定案）。执行器恒拿现行那一版的根与记录，所以
        // `FirstFileVersionDoesNotFollowTheVersionItBuildsOn` 在随机历史里走不到、模型一次都不要求它。
        // 现行那一版已经有过文件版本：这条路径按「树还没建起来」写，再走一次会重新建树、重新发对象出生代，
        // 与环里那些旧根记的对不上（I-9.14、I-9.10）——再写一版走覆盖写。
        if current.contents.has_trees() {
            required_refusals
                .insert(ModelRefusalReason::FirstFileVersionOnAVersionThatAlreadyHasAFile);
        }
        if u64::try_from(content.len()).expect("内容长度") > data_unit_payload_capacity_in_bytes()
        {
            required_refusals.insert(ModelRefusalReason::ContentExceedsDataUnitPayload);
        }
        let root = self.next_root(
            current,
            session.instance,
            ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1),
            ModelJournalCounter(current.journal_counter.0 + 1),
            ModelPublishKind::FirstFileVersion,
            current.rollback_floor,
            Some(content),
            Rc::clone(&current.instance_table_rows),
        );
        Ok(self.answer_for_publishes(
            ModelOperationKind::PublishFirstFile,
            required_refusals,
            vec![root],
        ))
    }

    /// 覆盖写（`transaction::publish_overwrite`）：接在现行的带文件的一版后面。现行那一版有树而文件集合为空时（执行器在删除中的那一版上调，
    /// 会话先走空、把它走成这一形）必须拒（[`ModelRefusalReason::OnlyNewInodesOnAVersionWithTreesAndNoFile`]）。
    ///
    /// # Errors
    /// 模型里没有可写会话，或现行版本没有文件、也不是有树而文件集合为空（执行器只在带文件或删除中时调）：会话状态与实现对不上。
    pub fn answer_publish_overwrite(
        &self,
        content: &[u8],
    ) -> Result<ModelAnswer, ModelDisagreement> {
        self.answer_rewriting_the_whole_file(
            content,
            ModelOperationKind::PublishOverwrite,
            WholeFileWriteContract::OneDataUnit,
        )
    }

    /// 顺序写（`transaction::publish_sequential_write`，经会话的 `UserChange::SequentialWrite`）：同覆盖写接在现行的带文件的一版后面、
    /// 整份重写文件内容，不按单元数拒——内容切成几个数据单元就写几个（一个单元一个事务），多于一个时连同 extent 树下段一起写
    /// （D8（核心索引结构） 已定项 14）。现行那一版有树而文件集合为空时同覆盖写必须拒。
    ///
    /// # Errors
    /// 同 [`IdealModel::answer_publish_overwrite`]。
    pub fn answer_publish_sequential_write(
        &self,
        content: &[u8],
    ) -> Result<ModelAnswer, ModelDisagreement> {
        self.answer_rewriting_the_whole_file(
            content,
            ModelOperationKind::PublishSequentialWrite,
            WholeFileWriteContract::AnyNumberOfDataUnits,
        )
    }

    /// 覆盖写与顺序写共用的答法：两者只差入口的契约（一个数据单元以内，还是不按单元数拒）。
    fn answer_rewriting_the_whole_file(
        &self,
        content: &[u8],
        operation: ModelOperationKind,
        contract: WholeFileWriteContract,
    ) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        if let Some(refused) =
            self.answer_refusing_a_change_on_a_version_with_trees_and_no_file(operation)
        {
            return Ok(refused);
        }
        // 截断途中的那一版读得出文件，但会话先走空（截断的各批），走到这里现行版本只会是带活文件的一版。
        if !matches!(current.contents, ModelVersionContents::LiveFile(_)) {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::SessionState,
                format!(
                    "模型里现行版本 {:?} 没有读得出的文件：{:?}",
                    current.key, current.contents
                ),
                "实现的现行版本带文件、队列空（执行器调了覆盖写或顺序写）".to_string(),
            ));
        }
        let mut required_refusals = BTreeSet::new();
        match contract {
            WholeFileWriteContract::OneDataUnit => {
                if u64::try_from(content.len()).expect("内容长度")
                    > data_unit_payload_capacity_in_bytes()
                {
                    required_refusals.insert(ModelRefusalReason::ContentExceedsDataUnitPayload);
                }
            }
            WholeFileWriteContract::AnyNumberOfDataUnits => {}
        }
        let root = self.next_root(
            current,
            session.instance,
            ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1),
            ModelJournalCounter(self.highest_journal_counter.0 + 1),
            ModelPublishKind::OverwriteFileVersion,
            current.rollback_floor,
            Some(content),
            Rc::clone(&current.instance_table_rows),
        );
        Ok(self.answer_for_publishes(operation, required_refusals, vec![root]))
    }

    /// 零单元发布（`transaction::publish_without_units`）：接在现行的树表 0 条的一版后面。
    ///
    /// # Errors
    /// 模型里没有可写会话，或现行版本带文件（执行器只在树表 0 条时调）：会话状态与实现对不上。
    pub fn answer_publish_without_units(&self) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        if current.contents.has_trees() {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::SessionState,
                format!("模型里现行版本 {:?} 有树", current.key),
                "实现的现行版本树表 0 条（执行器调了零单元发布）".to_string(),
            ));
        }
        let root = self.next_root(
            current,
            session.instance,
            ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1),
            ModelJournalCounter(self.highest_journal_counter.0 + 1),
            ModelPublishKind::ZeroUnit,
            current.rollback_floor,
            None,
            Rc::clone(&current.instance_table_rows),
        );
        Ok(self.answer_for_publishes(
            ModelOperationKind::PublishWithoutUnits,
            BTreeSet::new(),
            vec![root],
        ))
    }

    /// unlink 第一个文件的意图那次（`mounted_session::UserChange::Unlink`，inode 1）：同 [`IdealModel::answer_unlink_intent_of_an_inode`]。
    ///
    /// # Errors
    /// 同 [`IdealModel::answer_unlink_intent_of_an_inode`]。
    pub fn answer_unlink_intent(&self) -> Result<ModelAnswer, ModelDisagreement> {
        self.answer_unlink_intent_of_an_inode(FIRST_INODE_NUMBER)
    }

    /// unlink `inode` 的意图那次（`mounted_session::UserChange::Unlink`，D8（核心索引结构） 已定项 16 第 1 条；判决
    /// `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M1、M3：对任一号同一条）：接在现行那一版后面，那条记录 nlink 写 0、
    /// `deleted_inodes` 加 key、墓碑树追加一条；inode 1 的这一版读回「没有文件」，别的号这一版读不出它、第一个文件照旧。会话接着走空（每一批由
    /// [`IdealModel::answer_deletion_step`] 答）；两次之间会话为空间推的抬 F 由执行器按 txg 次序插进来比。逐次发布判墙。
    /// 有树而文件集合为空的一版上必拒（[`ModelRefusalReason::OnlyNewInodesOnAVersionWithTreesAndNoFile`]）。
    ///
    /// # Errors
    /// 模型里没有可写会话，或 `inode` 不是现行那一版树里活着的号（执行器只点名实现现行那一版里活着的号，会话先走空）：会话状态与实现对不上；
    /// 现行那一版 inode 树多于一片叶容器（删一条要吸收或借，模型的删除那一臂只答一片的，执行器在那一形上不调）：[`ModelDisagreementAspect::NotModeled`]。
    pub fn answer_unlink_intent_of_an_inode(
        &self,
        inode: u64,
    ) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        if let Some(refused) = self.answer_refusing_a_change_on_a_version_with_trees_and_no_file(
            ModelOperationKind::PublishUnlinkIntent,
        ) {
            return Ok(refused);
        }
        if current.inode_records_in_the_tree() > INODE_LEAF_RECORDS {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::NotModeled,
                format!(
                    "模型里现行版本 {:?} 的 inode 树有 {} 条记录、多于一片叶容器：删除那一臂只答一片的（删一条不吸收、不借）",
                    current.key,
                    current.inode_records_in_the_tree()
                ),
                format!("执行器调了 unlink inode {inode}"),
            ));
        }
        let deleted = ModelDeletedInode::of_the_inode_number(inode);
        let is_a_live_record = match deleted {
            ModelDeletedInode::FirstFile => {
                matches!(current.contents, ModelVersionContents::LiveFile(_))
            }
            ModelDeletedInode::OtherThanTheFirst(other) => {
                matches!(
                    current.contents,
                    ModelVersionContents::LiveFile(_)
                        | ModelVersionContents::InodesWithoutTheFirstFile { .. }
                ) && current
                    .inodes_other_than_the_first
                    .get(&other)
                    .is_some_and(ModelInodeOtherThanTheFirst::is_live)
            }
        };
        if !is_a_live_record {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::SessionState,
                format!(
                    "模型里现行版本 {:?} 没有活着的 inode {inode}：{:?}，第一个文件之外的 {:?}",
                    current.key,
                    current.contents,
                    current
                        .inodes_other_than_the_first
                        .keys()
                        .collect::<Vec<_>>()
                ),
                format!("实现的现行版本带文件、队列空（执行器调了 unlink inode {inode}）"),
            ));
        }
        let root = self.next_root(
            current,
            session.instance,
            ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1),
            ModelJournalCounter(self.highest_journal_counter.0 + 1),
            ModelPublishKind::UnlinkIntent { deleted },
            current.rollback_floor,
            None,
            Rc::clone(&current.instance_table_rows),
        );
        Ok(self.answer_for_publishes(
            ModelOperationKind::PublishUnlinkIntent,
            BTreeSet::new(),
            vec![root],
        ))
    }

    /// 走空 `deleted_inodes` 的一步（D8（核心索引结构） 已定项 16 第 2、6 条：会话里 unlink 之后、每次用户改动之前，挂载续做；判决
    /// `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M1：对任一号同一条，剩几个单元取那个号自己的）：删除中的那个号还剩的
    /// 数据单元多于产品批大小 144 就是一个不是最后一批的删除批（从尾部释放 144 个，之后仍是删除中的那一版），不多于就是最后一批——删掉那条记录：
    /// 树里还有别的记录时是带文件的一版（删第一个文件而别的文件还在也是，第三节 M3），一条都不剩才是有树而文件集合为空的一版；extent 上段按规范形
    /// 收缩（[`extent_upper_node_written_at_after`]：降层那次根照抄）。下一步只由现行那一版定，挂载续做与一路做下来同一个答法。
    ///
    /// # Errors
    /// 模型里没有可写会话，或现行版本没有删除中的号（执行器只在实现交回了一步删除时问）：会话状态与实现对不上；inode 树多于一片叶容器：
    /// [`ModelDisagreementAspect::NotModeled`]（同 [`IdealModel::answer_unlink_intent_of_an_inode`]）。
    pub fn answer_deletion_step(&self) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        let Some((deleted, data_units_remaining)) = current.inode_being_deleted() else {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::SessionState,
                format!(
                    "模型里现行版本 {:?} 没有删除中的号：{:?}",
                    current.key, current.contents
                ),
                "实现交回了走空 deleted_inodes 的一步".to_string(),
            ));
        };
        if current.inode_records_in_the_tree() > INODE_LEAF_RECORDS {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::NotModeled,
                format!(
                    "模型里现行版本 {:?} 的 inode 树有 {} 条记录、多于一片叶容器：删除那一臂只答一片的",
                    current.key,
                    current.inode_records_in_the_tree()
                ),
                "实现交回了走空 deleted_inodes 的一步".to_string(),
            ));
        }
        let step = if data_units_remaining > MODEL_DELETION_BATCH_DATA_UNITS {
            ModelPublishKind::DeletionBatchKeepingDataUnits { deleted }
        } else {
            let inode_tree_after = if current.inode_records_in_the_tree() == 1 {
                ModelInodeTreeAfterTheFinalDeletionBatch::Emptied
            } else {
                ModelInodeTreeAfterTheFinalDeletionBatch::StillHoldsRecords
            };
            // 上段的根这次重写还是照抄：按规范形计划先算一遍（`next_root_with_the_user_visible_trees` 里同一份计划再算一遍，断言种类与它一致）。
            let txg_of_the_batch = ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1);
            let inodes_with_an_entry_before = current.inodes_with_an_extent_upper_leaf_entry();
            let mut inodes_with_an_entry_after = inodes_with_an_entry_before.clone();
            let inodes_whose_entry_changed =
                if inodes_with_an_entry_after.remove(&deleted.inode_number()) {
                    BTreeSet::from([deleted.inode_number()])
                } else {
                    BTreeSet::new()
                };
            let root_after = ModelExtentUpperNodePosition {
                level: extent_upper_segment_root_level(&inodes_with_an_entry_after),
                index_in_level: 0,
            };
            let root_is_rewritten = extent_upper_node_written_at_after(
                &current.extent_upper_node_written_at(),
                &inodes_with_an_entry_after,
                &inodes_whose_entry_changed,
                txg_of_the_batch,
            )
            .get(&root_after)
                == Some(&txg_of_the_batch);
            let extent_upper_root = if root_is_rewritten {
                ModelExtentUpperRootOfTheFinalDeletionBatch::Rewritten
            } else {
                ModelExtentUpperRootOfTheFinalDeletionBatch::CarriedFromThePreviousVersion
            };
            ModelPublishKind::FinalDeletionBatch {
                deleted,
                inode_tree_after,
                extent_upper_root,
            }
        };
        let root = self.next_root(
            current,
            session.instance,
            ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1),
            ModelJournalCounter(self.highest_journal_counter.0 + 1),
            step,
            current.rollback_floor,
            None,
            Rc::clone(&current.instance_table_rows),
        );
        Ok(self.answer_for_publishes(
            ModelOperationKind::PublishDeletionStep,
            BTreeSet::new(),
            vec![root],
        ))
    }

    /// 建一个 size 0 的 inode（`mounted_session::UserChange::NewInodes`，count 1）：接在第一个文件已删完的一版后面
    /// （有树而文件集合为空、或删完之后建过 inode 的），这一版仍读回「没有文件」。会话先走空（队列空，一个写都不发）。逐次发布判墙。
    ///
    /// # Errors
    /// 模型里没有可写会话，或现行版本里第一个文件还在、树表 0 条（执行器只在第一个文件已删完时调）：会话状态与实现对不上。
    pub fn answer_create_inode(&self) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        match current.contents {
            ModelVersionContents::TreesAndNoFile { .. }
            | ModelVersionContents::InodesWithoutTheFirstFile { .. } => {}
            ModelVersionContents::WithoutTrees
            | ModelVersionContents::LiveFile(_)
            | ModelVersionContents::FileBeingDeleted { .. }
            | ModelVersionContents::FileBeingTruncated { .. } => {
                return Err(ModelDisagreement::new(
                    ModelDisagreementAspect::SessionState,
                    format!(
                        "模型里现行版本 {:?} 不是第一个文件已删完的一版：{:?}",
                        current.key, current.contents
                    ),
                    "实现的现行版本第一个文件已删完（执行器调了建 inode）".to_string(),
                ));
            }
        }
        self.answer_create_inodes(1)
    }

    /// 建 `count` 个 size 0 的 inode（`mounted_session::UserChange::NewInodes`）：接在第一个文件活着的一版（在第一个文件旁边建：按 inode 号写数据
    /// 先要有别的 inode，里程碑「删除与多次 COW」一·1）或第一个文件已删完的一版后面。号从这一版的水位起连着发（[`ModelRoot::inode_number_watermark`]），
    /// 新记录追加在最右那片叶容器、满 233 条在末尾分裂（D8（核心索引结构） 已定项 6），一次一个事务、留一级水位台阶。会话先走空（队列空，一个写都不发）。
    /// 逐次发布判墙。
    ///
    /// # Errors
    /// 模型里没有可写会话，或现行版本树表 0 条、删除中、截断途中（会话先走空，执行器在走空之后才问这一次）：会话状态与实现对不上；`count` 是 0、
    /// 或这几条新记录跨了叶容器（一次要重写两片，模型只答一片：执行器把 `count` 压到最右那片装得下的条数）：[`ModelDisagreementAspect::NotModeled`]。
    pub fn answer_create_inodes(&self, count: u64) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        match current.contents {
            ModelVersionContents::LiveFile(_)
            | ModelVersionContents::TreesAndNoFile { .. }
            | ModelVersionContents::InodesWithoutTheFirstFile { .. } => {}
            ModelVersionContents::WithoutTrees
            | ModelVersionContents::FileBeingDeleted { .. }
            | ModelVersionContents::FileBeingTruncated { .. } => {
                return Err(ModelDisagreement::new(
                    ModelDisagreementAspect::SessionState,
                    format!(
                        "模型里现行版本 {:?} 不是第一个文件活着或已删完、队列空的一版：{:?}",
                        current.key, current.contents
                    ),
                    "实现的现行版本有树、队列空（执行器调了建 inode）".to_string(),
                ));
            }
        }
        let inode_records_before = current.inode_records_in_the_tree();
        // 新记录落在哪一片叶容器定这次重写哪一片（D8（核心索引结构） 已定项 6）：最左那片之外的那几片各是一个角色。
        let position_of_the_first_new_record =
            inode_leaf_container_position_of_a_new_record(inode_records_before);
        let fits_in_one_leaf_container = count > 0
            && inode_leaf_container_position_of_a_new_record(inode_records_before + count - 1)
                == position_of_the_first_new_record;
        if !fits_in_one_leaf_container {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::NotModeled,
                format!(
                    "一次建 {count} 个 inode、树里原有 {inode_records_before} 条：新记录不落在同一片叶容器里，模型只答一次重写一片的"
                ),
                "执行器调了建 inode".to_string(),
            ));
        }
        let kind = match NonZeroU64::new(position_of_the_first_new_record) {
            None => ModelPublishKind::NewInode,
            Some(position_from_the_leftmost) => {
                ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(
                    ModelInodeLeafContainerRightOfTheLeftmost {
                        position_from_the_leftmost,
                    },
                )
            }
        };
        let root = self.next_root_with_the_user_visible_trees(
            current,
            session.instance,
            ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1),
            ModelJournalCounter(self.highest_journal_counter.0 + 1),
            kind,
            current.rollback_floor,
            None,
            Rc::clone(&current.instance_table_rows),
            UserVisibleTreesOfTheNewVersion::AsThisPublishLeavesThem,
            InodesOtherThanTheFirstChangeOfThePublish::Created { count },
        );
        Ok(self.answer_for_publishes(
            ModelOperationKind::PublishNewInode,
            BTreeSet::new(),
            vec![root],
        ))
    }

    /// 按 inode 号覆盖写（`mounted_session::UserChange::OverwriteOfAnInode`，`transaction::publish_overwrite_of_an_inode`）：契约是一个数据单元，
    /// 装不下必须拒。答法见 [`IdealModel::answer_whole_write_of_an_inode`]。
    ///
    /// # Errors
    /// 同 [`IdealModel::answer_whole_write_of_an_inode`]。
    pub fn answer_overwrite_of_an_inode(
        &self,
        inode: u64,
        content: &[u8],
    ) -> Result<ModelAnswer, ModelDisagreement> {
        self.answer_whole_write_of_an_inode(inode, content, WholeFileWriteContract::OneDataUnit)
    }

    /// 按 inode 号顺序写（`mounted_session::UserChange::SequentialWriteOfAnInode`，`transaction::publish_sequential_write_of_an_inode`）：
    /// 不按单元数拒。答法见 [`IdealModel::answer_whole_write_of_an_inode`]。
    ///
    /// # Errors
    /// 同 [`IdealModel::answer_whole_write_of_an_inode`]。
    pub fn answer_sequential_write_of_an_inode(
        &self,
        inode: u64,
        content: &[u8],
    ) -> Result<ModelAnswer, ModelDisagreement> {
        self.answer_whole_write_of_an_inode(
            inode,
            content,
            WholeFileWriteContract::AnyNumberOfDataUnits,
        )
    }

    /// 按 inode 号整份写 `inode` 那个文件（里程碑「删除与多次 COW」一·1）：
    /// - `inode` 是第一个文件：同覆盖写与顺序写（[`IdealModel::answer_publish_overwrite`]，两个入口在实现里就是那两条路）；
    /// - 现行那一版有树而文件集合为空：必拒（[`ModelRefusalReason::OnlyNewInodesOnAVersionWithTreesAndNoFile`]，会话拒成「没有文件版本可改」）；
    /// - 第一个文件已删完、目标是删完之后才建的：必拒（[`ModelRefusalReason::DataWriteToAnInodeBornAfterTheFirstFileWasDeleted`]，D8（核心索引结构）
    ///   已定项 16 射程「删完之后不再建带数据的文件（K-U5，记欠账）」，用户 2026-10-19 定「β 时序级」：对象出生代晚于 inode 1 墓碑死亡 txg 的才拒）；
    /// - 第一个文件活着，或已删完而目标是删之前就有的：那个文件整份换成 `content`（[`ModelPublishKind::WholeWriteOfAnInodeOtherThanTheFirst`]），
    ///   一个数据单元的契约装不下时必拒。
    ///
    /// # Errors
    /// 模型里没有可写会话；现行版本树表 0 条、删除中、截断途中，或 `inode` 不是这一版树里第一个文件之外活着的号（执行器只点名实现
    /// 现行那一版树里活着的号）：会话状态与实现对不上。
    fn answer_whole_write_of_an_inode(
        &self,
        inode: u64,
        content: &[u8],
        contract: WholeFileWriteContract,
    ) -> Result<ModelAnswer, ModelDisagreement> {
        if inode == FIRST_INODE_NUMBER {
            let operation = match contract {
                WholeFileWriteContract::OneDataUnit => ModelOperationKind::PublishOverwrite,
                WholeFileWriteContract::AnyNumberOfDataUnits => {
                    ModelOperationKind::PublishSequentialWrite
                }
            };
            return self.answer_rewriting_the_whole_file(content, operation, contract);
        }
        let operation = ModelOperationKind::PublishWholeWriteOfAnInodeOtherThanTheFirst;
        let session = self.open_session()?;
        let current = &session.current;
        if let Some(refused) =
            self.answer_refusing_a_change_on_a_version_with_trees_and_no_file(operation)
        {
            return Ok(refused);
        }
        let first_file_records_in_the_tree = match current.contents {
            ModelVersionContents::LiveFile(_) => 1,
            ModelVersionContents::InodesWithoutTheFirstFile { .. } => 0,
            ModelVersionContents::WithoutTrees
            | ModelVersionContents::TreesAndNoFile { .. }
            | ModelVersionContents::FileBeingDeleted { .. }
            | ModelVersionContents::FileBeingTruncated { .. } => {
                return Err(ModelDisagreement::new(
                    ModelDisagreementAspect::SessionState,
                    format!(
                        "模型里现行版本 {:?} 不是第一个文件活着、或已删完而树里还有别的 inode 的一版：{:?}",
                        current.key, current.contents
                    ),
                    format!("实现的现行版本树里有 inode {inode}（执行器按 inode 号写它）"),
                ));
            }
        };
        // 记录按号排：第一个文件活着时 inode 1 在最前，第一个文件之外的按号升序接在后面（删除中的那一条还在树里，也算）；第 r 条（从 0 数）在左起第
        // ⌊r ÷ 233⌋ 片叶容器里（建 inode 只往右追加、满了在末尾分裂，[`inode_leaf_container_position_of_a_new_record`]；删除只在树里只有一片叶容器时做，
        // 删完仍是一片，这条数法不破）。
        let Some((position_among_the_inodes_other_than_the_first, written_inode)) = current
            .inodes_other_than_the_first
            .iter()
            .enumerate()
            .find(|(_, (number, other_inode))| **number == inode && other_inode.is_live())
            .map(|(position, (_, other_inode))| (position, other_inode))
        else {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::SessionState,
                format!(
                    "模型里现行版本 {:?} 第一个文件之外的 inode 是 {:?}，没有活着的 {inode}",
                    current.key,
                    current
                        .inodes_other_than_the_first
                        .keys()
                        .collect::<Vec<_>>()
                ),
                format!("实现的现行版本树里有 inode {inode}（执行器按 inode 号写它）"),
            ));
        };
        let record_position_in_the_tree = first_file_records_in_the_tree
            + u64::try_from(position_among_the_inodes_other_than_the_first).expect("条数");
        let inode_leaf_container = ModelUnitRole::inode_leaf_container_at(
            record_position_in_the_tree / INODE_LEAF_RECORDS,
        );
        let mut required_refusals = BTreeSet::new();
        match contract {
            WholeFileWriteContract::OneDataUnit => {
                if u64::try_from(content.len()).expect("内容长度")
                    > data_unit_payload_capacity_in_bytes()
                {
                    required_refusals.insert(ModelRefusalReason::ContentExceedsDataUnitPayload);
                }
            }
            WholeFileWriteContract::AnyNumberOfDataUnits => {}
        }
        // K-U5 的 β 时序级读法（用户 2026-10-19 定）：第一个文件那条记录不在这一版里、墓碑树里有 inode 1 的墓碑、目标的对象出生代晚于它的死亡 txg
        // 才拒，在任何写之前；删之前就有的别的文件照常能写。
        let is_born_after_the_first_file_was_deleted = first_file_records_in_the_tree == 0
            && current
                .death_txg_of_the_first_file
                .is_some_and(|death_txg| written_inode.object_birth > death_txg);
        if is_born_after_the_first_file_was_deleted {
            required_refusals
                .insert(ModelRefusalReason::DataWriteToAnInodeBornAfterTheFirstFileWasDeleted);
            return Ok(self.answer_for_publishes(operation, required_refusals, Vec::new()));
        }
        let root = self.next_root_with_the_user_visible_trees(
            current,
            session.instance,
            ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1),
            ModelJournalCounter(self.highest_journal_counter.0 + 1),
            ModelPublishKind::WholeWriteOfAnInodeOtherThanTheFirst {
                inode_leaf_container,
            },
            current.rollback_floor,
            None,
            Rc::clone(&current.instance_table_rows),
            UserVisibleTreesOfTheNewVersion::AsThisPublishLeavesThem,
            InodesOtherThanTheFirstChangeOfThePublish::WholeWrite { inode, content },
        );
        Ok(self.answer_for_publishes(operation, required_refusals, vec![root]))
    }

    /// 按偏移写 `inode` 那个文件（`mounted_session::UserChange::WriteAtOffset`，`transaction::publish_partial_write`；判决
    /// `research/prompts/m4-partial-write-r1-main-verification.md` 第二节）：会话先走空（执行器把走空的那几步按「走空的一步」比过之后才问这一次），
    /// 再接在现行那一版后面发一次，逐次发布判墙：
    /// - 现行那一版有树而文件集合为空：必拒（[`ModelRefusalReason::OnlyNewInodesOnAVersionWithTreesAndNoFile`]，会话拒成「没有文件版本可改」）；
    /// - `inode` 不是第一个文件、第一个文件已删完而它是删完之后才建的：必拒（[`ModelRefusalReason::DataWriteToAnInodeBornAfterTheFirstFileWasDeleted`]，
    ///   K-U5 的 β 时序级读法，同按 inode 号整份写）；
    /// - 写的终点越过 u64、或 ⌈写之后的 size ÷ 净荷⌉ × 净荷越过 u64：必拒（[`ModelRefusalReason::TruncationSizeWhoseWholeDataUnitsOverflow`]）；
    /// - 别的：读回的内容在偏移处盖上这次的字节，size = max(写之前, 终点)，写之前的末尾到偏移之间读 0、洞读 0（R5）；重写哪一段照
    ///   [`model_partial_write_rewritten_data_units`]（R4）；截断字段照抄（R5）。
    ///
    /// 不答「一次发布切出来的记录多于在飞上限」那一拒：模型不记 journal 环的槽数，执行器取的偏移离末尾至多几个净荷，重写的单元数远在上限之下。
    ///
    /// # Errors
    /// 模型里没有可写会话；现行版本树表 0 条、删除中、截断途中，或 `inode` 不是这一版树里活着的号（执行器只点名实现现行那一版树里活着的号）：
    /// 会话状态与实现对不上。这次的字节是空的（长度 0 的写条款没写，实现在类型上写不出来）、写的终点装不进这个进程的地址空间：
    /// [`ModelDisagreementAspect::NotModeled`]。
    pub fn answer_partial_write(
        &self,
        inode: u64,
        offset_in_bytes: u64,
        bytes: &[u8],
    ) -> Result<ModelAnswer, ModelDisagreement> {
        let operation = ModelOperationKind::PublishPartialWrite;
        let session = self.open_session()?;
        let current = &session.current;
        if let Some(refused) =
            self.answer_refusing_a_change_on_a_version_with_trees_and_no_file(operation)
        {
            return Ok(refused);
        }
        if bytes.is_empty() {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::NotModeled,
                "长度 0 的按偏移写条款没写（判决 m4-partial-write-r1 没判），模型不答".to_string(),
                format!("执行器按偏移写 inode {inode}、0 字节"),
            ));
        }
        let written = if inode == FIRST_INODE_NUMBER {
            // 截断途中的那一版读得出文件，但会话先走空（截断的各批），走到这里现行版本只会是带活文件的一版。
            let ModelVersionContents::LiveFile(file) = &current.contents else {
                return Err(ModelDisagreement::new(
                    ModelDisagreementAspect::SessionState,
                    format!(
                        "模型里现行版本 {:?} 没有带活文件、队列空的 inode 1：{:?}",
                        current.key, current.contents
                    ),
                    "实现的现行版本带文件、队列空（执行器按偏移写 inode 1）".to_string(),
                ));
            };
            ModelFileWrittenAtAnOffset::TheFirstFile(file)
        } else {
            let first_file_records_in_the_tree = match current.contents {
                ModelVersionContents::LiveFile(_) => 1,
                ModelVersionContents::InodesWithoutTheFirstFile { .. } => 0,
                ModelVersionContents::WithoutTrees
                | ModelVersionContents::TreesAndNoFile { .. }
                | ModelVersionContents::FileBeingDeleted { .. }
                | ModelVersionContents::FileBeingTruncated { .. } => {
                    return Err(ModelDisagreement::new(
                        ModelDisagreementAspect::SessionState,
                        format!(
                            "模型里现行版本 {:?} 不是第一个文件活着、或已删完而树里还有别的 inode 的一版：{:?}",
                            current.key, current.contents
                        ),
                        format!("实现的现行版本树里有 inode {inode}（执行器按偏移写它）"),
                    ));
                }
            };
            // 记录按号排，第 r 条在左起第 ⌊r ÷ 233⌋ 片叶容器里（同按 inode 号整份写，[`IdealModel::answer_whole_write_of_an_inode`]）。
            let Some((position_among_the_inodes_other_than_the_first, written_inode)) = current
                .inodes_other_than_the_first
                .iter()
                .enumerate()
                .find(|(_, (number, other_inode))| **number == inode && other_inode.is_live())
                .map(|(position, (_, other_inode))| (position, other_inode))
            else {
                return Err(ModelDisagreement::new(
                    ModelDisagreementAspect::SessionState,
                    format!(
                        "模型里现行版本 {:?} 第一个文件之外的 inode 是 {:?}，没有活着的 {inode}",
                        current.key,
                        current
                            .inodes_other_than_the_first
                            .keys()
                            .collect::<Vec<_>>()
                    ),
                    format!("实现的现行版本树里有 inode {inode}（执行器按偏移写它）"),
                ));
            };
            // K-U5 的 β 时序级读法（用户 2026-10-19 定）：同按 inode 号整份写，在任何写之前拒。
            let is_born_after_the_first_file_was_deleted = first_file_records_in_the_tree == 0
                && current
                    .death_txg_of_the_first_file
                    .is_some_and(|death_txg| death_txg < written_inode.object_birth);
            if is_born_after_the_first_file_was_deleted {
                return Ok(self.answer_for_publishes(
                    operation,
                    BTreeSet::from([
                        ModelRefusalReason::DataWriteToAnInodeBornAfterTheFirstFileWasDeleted,
                    ]),
                    Vec::new(),
                ));
            }
            let record_position_in_the_tree = first_file_records_in_the_tree
                + u64::try_from(position_among_the_inodes_other_than_the_first).expect("条数");
            ModelFileWrittenAtAnOffset::OtherThanTheFirst {
                inode_leaf_container: ModelUnitRole::inode_leaf_container_at(
                    record_position_in_the_tree / INODE_LEAF_RECORDS,
                ),
                file: written_inode,
            }
        };
        let (content_before, data_units_before, declared_length_of_the_last_existing_unit) =
            match written {
                ModelFileWrittenAtAnOffset::TheFirstFile(file) => (
                    &file.content,
                    file.data_units_present,
                    file.data_units_present
                        .checked_sub(1)
                        .map(|last| file.declared_length_of_the_data_unit(last)),
                ),
                ModelFileWrittenAtAnOffset::OtherThanTheFirst { file, .. } => (
                    &file.content,
                    file.data_units_present(),
                    file.data_units_present()
                        .checked_sub(1)
                        .map(|last| file.declared_length_of_the_data_unit(last)),
                ),
            };
        let size_before = u64::try_from(content_before.len()).expect("内容长度");
        let length_in_bytes = u64::try_from(bytes.len()).expect("一段内存里的字节数装得进 u64");
        let payload = data_unit_payload_capacity_in_bytes();
        let end_exclusive = offset_in_bytes.checked_add(length_in_bytes);
        let size_after_rounds_up_within_the_range = end_exclusive.is_some_and(|end| {
            end.max(size_before)
                .div_ceil(payload)
                .checked_mul(payload)
                .is_some()
        });
        let Some(end_exclusive) = end_exclusive.filter(|_| size_after_rounds_up_within_the_range)
        else {
            return Ok(self.answer_for_publishes(
                operation,
                BTreeSet::from([ModelRefusalReason::TruncationSizeWhoseWholeDataUnitsOverflow]),
                Vec::new(),
            ));
        };
        let (Ok(offset_in_memory), Ok(end_in_memory)) = (
            usize::try_from(offset_in_bytes),
            usize::try_from(end_exclusive),
        ) else {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::NotModeled,
                format!("模型照字节记内容，写到 {end_exclusive} 字节装不进这个进程的地址空间"),
                format!("执行器按偏移写 inode {inode}"),
            ));
        };
        let mut content_after = content_before.to_vec();
        if content_after.len() < end_in_memory {
            content_after.resize(end_in_memory, 0);
        }
        content_after[offset_in_memory..end_in_memory].copy_from_slice(bytes);
        let (first_rewritten_data_unit, data_units_after) =
            model_partial_write_rewritten_data_units(
                data_units_before,
                declared_length_of_the_last_existing_unit,
                offset_in_bytes,
                end_exclusive,
            );
        let checkpoint_txg = ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1);
        let journal_counter = ModelJournalCounter(self.highest_journal_counter.0 + 1);
        let root = match written {
            ModelFileWrittenAtAnOffset::TheFirstFile(_) => self.next_root(
                current,
                session.instance,
                checkpoint_txg,
                journal_counter,
                ModelPublishKind::PartialWriteOfTheFirstFile {
                    first_rewritten_data_unit,
                    data_units_after,
                },
                current.rollback_floor,
                Some(&content_after),
                Rc::clone(&current.instance_table_rows),
            ),
            ModelFileWrittenAtAnOffset::OtherThanTheFirst {
                inode_leaf_container,
                ..
            } => self.next_root_with_the_user_visible_trees(
                current,
                session.instance,
                checkpoint_txg,
                journal_counter,
                ModelPublishKind::PartialWriteOfAnInodeOtherThanTheFirst {
                    inode,
                    inode_leaf_container,
                    first_rewritten_data_unit,
                    data_units_after,
                },
                current.rollback_floor,
                None,
                Rc::clone(&current.instance_table_rows),
                UserVisibleTreesOfTheNewVersion::AsThisPublishLeavesThem,
                InodesOtherThanTheFirstChangeOfThePublish::PartialWrite {
                    inode,
                    content_after: &content_after,
                },
            ),
        };
        Ok(self.answer_for_publishes(operation, BTreeSet::new(), vec![root]))
    }

    /// 建一个只读快照（`mounted_session::UserChange::CreateSnapshot`，判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S2）：
    /// 会话先走空（执行器把走空的那几步按「走空的一步」比过之后才问这一次），再接在现行那一版后面发一次独立发布。现行那一版有树而文件集合为空时
    /// 照建，快照读出「没有文件」（用户 2026-10-16 定「放开」，判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F5）；
    /// 已经有 10 个快照时必须拒（第七节岔路 1）。逐次发布判墙。
    ///
    /// # Errors
    /// 模型里没有可写会话，或现行版本树表 0 条（执行器只在有树的一版上调）：会话状态与实现对不上。
    pub fn answer_create_snapshot(&self) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        if !current.contents.has_trees() {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::SessionState,
                format!("模型里现行版本 {:?} 树表 0 条", current.key),
                "实现的现行版本有树（执行器调了建快照）".to_string(),
            ));
        }
        let mut required_refusals = BTreeSet::new();
        if current.snapshots.len() >= MODEL_SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION {
            required_refusals.insert(ModelRefusalReason::MoreSnapshotsThanTheFirstVersionAllows);
        }
        let root = self.next_root(
            current,
            session.instance,
            ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1),
            ModelJournalCounter(self.highest_journal_counter.0 + 1),
            ModelPublishKind::SnapshotCreation,
            current.rollback_floor,
            None,
            Rc::clone(&current.instance_table_rows),
        );
        Ok(self.answer_for_publishes(
            ModelOperationKind::PublishSnapshotCreation,
            required_refusals,
            vec![root],
        ))
    }

    /// 删按建的先后第 `position` 个快照的意图那次（`mounted_session::UserChange::DestroySnapshot`，判决第三节 S4 与第七节岔路 2、4）：
    /// 会话先走空，再接在现行那一版后面发意图；之后的每一批是走空的一步（[`IdealModel::answer_snapshot_destruction_step`]）。
    /// 现行那一版有树而文件集合为空时照样删（删快照不依赖活头里有没有文件，D8（核心索引结构） 已定项 16 第 5 条那一句管的是写文件的改动；
    /// 丙 2 交判第 5 条主 agent 定是缺陷，会话放行），与建快照相同（判决 `research/prompts/m4-segment-code-r1-main-verification.md`
    /// 第三节 F5）。逐次发布判墙。
    ///
    /// # Errors
    /// 模型里没有可写会话、现行版本树表 0 条、没有第 `position` 个快照、或现行那一版已有快照在销毁中（会话先把它走完，走到这里说明模型与实现
    /// 对不上）：会话状态与实现对不上。
    pub fn answer_destroy_snapshot(
        &self,
        position: usize,
    ) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        if !current.contents.has_trees()
            || position >= current.snapshots.len()
            || current
                .snapshots
                .iter()
                .any(|snapshot| snapshot.is_being_destroyed)
        {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::SessionState,
                format!(
                    "模型里现行版本 {:?} 没有能删的第 {position} 个快照：{:?}",
                    current.key, current.snapshots
                ),
                "实现的现行版本里有这个快照、没有销毁中的（执行器调了删快照）".to_string(),
            ));
        }
        let root = self.next_root(
            current,
            session.instance,
            ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1),
            ModelJournalCounter(self.highest_journal_counter.0 + 1),
            ModelPublishKind::SnapshotDestructionIntent { position },
            current.rollback_floor,
            None,
            Rc::clone(&current.instance_table_rows),
        );
        Ok(self.answer_for_publishes(
            ModelOperationKind::PublishSnapshotDestructionIntent,
            BTreeSet::new(),
            vec![root],
        ))
    }

    /// 走空销毁中那个快照的一步（判决 S4：会话里删快照的意图之后、每次用户改动之前，挂载续做）：它那一段这一批能放的条目多于一批（144 条）
    /// 就是不是最后一批的一批，放 144 条；不多于就是最后一批，放剩下的、删掉它。下一步只由现行那一版定。
    ///
    /// # Errors
    /// 模型里没有可写会话，或现行版本没有销毁中的快照：会话状态与实现对不上。
    pub fn answer_snapshot_destruction_step(&self) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        let Some(position) = current
            .snapshots
            .iter()
            .position(|snapshot| snapshot.is_being_destroyed)
        else {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::SessionState,
                format!(
                    "模型里现行版本 {:?} 没有销毁中的快照：{:?}",
                    current.key, current.snapshots
                ),
                "实现交回了走空销毁中快照的一步".to_string(),
            ));
        };
        let checkpoint_txg = ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1);
        let releasable = u64::try_from(
            Self::deadlist_entries_the_destruction_releases(current, position, checkpoint_txg)
                .len(),
        )
        .expect("条数");
        let step = if releasable > MODEL_SNAPSHOT_DESTRUCTION_BATCH_ENTRIES {
            ModelPublishKind::SnapshotDestructionBatch {
                position,
                entries_released: MODEL_SNAPSHOT_DESTRUCTION_BATCH_ENTRIES,
            }
        } else {
            ModelPublishKind::SnapshotDestructionFinalBatch { position }
        };
        let root = self.next_root(
            current,
            session.instance,
            checkpoint_txg,
            ModelJournalCounter(self.highest_journal_counter.0 + 1),
            step,
            current.rollback_floor,
            None,
            Rc::clone(&current.instance_table_rows),
        );
        Ok(self.answer_for_publishes(
            ModelOperationKind::PublishSnapshotDestructionStep,
            BTreeSet::new(),
            vec![root],
        ))
    }

    /// 走空的下一步（`transaction::walk_the_deletion_queue_until_it_is_empty`：`deleted_inodes` 先走空，再走销毁中的快照，再走截断，判决
    /// `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K1 的次序）：现行那一版有删除中的文件就是删除的一步
    /// （[`IdealModel::answer_deletion_step`]），否则有销毁中的快照就是删快照的一批（[`IdealModel::answer_snapshot_destruction_step`]），
    /// 否则在截断途中就是截断的一批（[`IdealModel::answer_truncation_step`]）。
    ///
    /// # Errors
    /// 模型里没有可写会话，或现行版本三样都没有：会话状态与实现对不上。
    pub fn answer_next_step_of_the_walk(&self) -> Result<ModelAnswer, ModelDisagreement> {
        if self.current_version_has_a_file_being_deleted() {
            self.answer_deletion_step()
        } else if self.current_version_has_a_snapshot_being_destroyed()
            || !self.current_version_has_a_file_being_truncated()
        {
            self.answer_snapshot_destruction_step()
        } else {
            self.answer_truncation_step()
        }
    }

    /// 整理的一次发布（`mounted_session::MountedSession::run_compaction` 发成的一版，判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节
    /// G1、G3、G4、G7；丁 3 交回的读法第 1–3 条）：接在现行那一版后面，txg 与 jsn 各加一、一个事务、F 照抄，只重写固定点单元，用户可见的内容、
    /// 快照、deadlist、每个搬走的单元的分配代都与上一版相同。是哪一种由模型记的活意图与执行器交来的这一版看得见的几样（`seen`）定：
    /// - 模型这一版没有活意图：只能是签发（不搬单元、发布之后 logged_ops 里有一条）——R 取执行器强制的那一段、`c0` 取上一版的 txg、
    ///   签发 txg 取这一次的（[`ModelPublishKind::CompactionIntentIssue`]）；别的样子都对不上（全池至多一条，先签发才搬）。
    /// - 有活意图、搬了单元：发布之后意图还在就是照抄意图的一批，这时 R 按记录还不能全空（搬空了 R 就要同一次发布删意图，D8（核心索引结构）
    ///   已定项 5）；意图没了就是最后一批同发布删意图，这时 R 要按记录全空（判决 G4 停机谓词「删意图那一刻 R 按记录全空」）。
    /// - 有活意图、不搬单元：意图要没了（单独删：R 按记录全空是达成，不空是丢），意图还在就对不上（这一次什么都没做）。照抄意图的一批至少搬一个
    ///   R 里真要搬的单元（判决 `research/prompts/m4-segment-code-r2-main-verification.md` 第三节 H4，用户 2026-10-16 定「规划时补足」）：计划跳过这次
    ///   发布本来就重写的节点，一个都没搬而照抄意图的那一次不许发；R 里只剩这次发布本来就重写的节点时那一次不搬单元、同一次发布删意图，发布之后
    ///   R 按记录全空，就是意图达成那一种。
    ///
    /// 搬的里有用户可见树的根单元（`seen.user_visible_root_units_relocated`）时把这一次记进那几个单元的搬迁 txg，这一版算一个新状态
    /// （[`ModelRoot::user_visible_root_relocations`]）；它们要是这一版引用着的。模型不答推几次、停在哪（判决 G7「只比成功操作的内容」）：
    /// 执行器按实现交回的每一版各问一次。先走空的删除续做由 [`IdealModel::answer_next_step_of_the_walk`] 答。逐次发布判墙。
    ///
    /// # Errors
    /// 模型里没有可写会话、现行版本树表 0 条、现行那一版还有要走空的：会话状态对不上；这一版对意图做的事与上面那几条对不上：
    /// [`ModelDisagreementAspect::CompactionIntentOfThePublish`]；搬的根单元模型这一版不引用：[`ModelDisagreementAspect::NotModeled`]。
    pub fn answer_compaction_publish(
        &self,
        seen: &ModelCompactionPublishSeen,
    ) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        if !current.contents.has_trees() || self.current_version_has_a_step_to_walk() {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::SessionState,
                format!(
                    "模型里现行版本 {:?} 不是能整理的一版（树表 0 条或还有要走空的）：{:?}",
                    current.key, current.contents
                ),
                "实现交回了整理的一次发布".to_string(),
            ));
        }
        let checkpoint_txg = ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1);
        let does_the_publish_relocate_units = seen.units_relocated > 0;
        let refused_step = |model_answer: String| {
            Err(ModelDisagreement::new(
                ModelDisagreementAspect::CompactionIntentOfThePublish,
                format!("{:?} 之后的这一次整理发布：{model_answer}", current.key),
                describe_compaction_publish_seen(seen),
            ))
        };
        let (kind, operation, issued) = match current.logged_compaction_intent {
            None => {
                if does_the_publish_relocate_units || !seen.is_an_intent_logged_after_the_publish {
                    return refused_step(
                        "这一版没有活整理意图：整理只能先签发（只把意图写进 logged_ops、不搬单元，判决 G1）".to_string(),
                    );
                }
                (
                    ModelPublishKind::CompactionIntentIssue,
                    ModelOperationKind::PublishCompactionIntentIssue,
                    Some(ModelCompactionIntent {
                        issued_txg: checkpoint_txg,
                        region: seen.region_forced_for_the_issue,
                        allocation_generation_at_issue: current.key.checkpoint_txg,
                    }),
                )
            }
            Some(live) => {
                let is_the_region_empty =
                    seen.is_the_region_of_the_live_intent_empty_by_the_records_after_the_publish;
                match (
                    does_the_publish_relocate_units,
                    seen.is_an_intent_logged_after_the_publish,
                    is_the_region_empty,
                ) {
                    (true, true, false) => (
                        ModelPublishKind::CompactionBatch {
                            units_at_most: seen.units_at_most,
                            intent_after_the_batch: ModelCompactionIntentAfterTheBatch::CarriedOn,
                        },
                        ModelOperationKind::PublishCompactionBatch,
                        None,
                    ),
                    (true, true, true) => {
                        return refused_step(format!(
                            "这一批搬空了活意图的 R {:?}（按记录）：意图要与最后一批同一次发布删（D8（核心索引结构） 已定项 5）",
                            live.region
                        ));
                    }
                    (true, false, true) => (
                        ModelPublishKind::CompactionBatch {
                            units_at_most: seen.units_at_most,
                            intent_after_the_batch:
                                ModelCompactionIntentAfterTheBatch::DeletedWithTheLastBatch,
                        },
                        ModelOperationKind::PublishCompactionBatch,
                        None,
                    ),
                    (true, false, false) => {
                        return refused_step(format!(
                            "删意图那一刻活意图的 R {:?} 要按记录全空（判决 G4 停机谓词）",
                            live.region
                        ));
                    }
                    (false, false, true) => (
                        ModelPublishKind::CompactionIntentDeletion {
                            ending:
                                ModelCompactionIntentEnding::CompletedWithTheRegionEmptyByTheRecords,
                        },
                        ModelOperationKind::PublishCompactionIntentCompletion,
                        None,
                    ),
                    (false, false, false) => (
                        ModelPublishKind::CompactionIntentDeletion {
                            ending: ModelCompactionIntentEnding::Dropped,
                        },
                        ModelOperationKind::PublishCompactionIntentDrop,
                        None,
                    ),
                    (false, true, true | false) => {
                        return refused_step(format!(
                            "活意图 {live:?} 在：一次整理发布要么搬一批（至少搬一个 R 里真要搬的单元，判决 H4）、要么删掉意图"
                        ));
                    }
                }
            }
        };
        let referenced: BTreeSet<ModelUserVisibleRootUnit> =
            current.user_visible_root_units().into_iter().collect();
        if let Some(unreferenced) = seen
            .user_visible_root_units_relocated
            .iter()
            .find(|unit| !referenced.contains(unit))
        {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::NotModeled,
                format!("{:?} 引用着的用户可见树根单元 {referenced:?}", current.key),
                format!("实现搬了根单元 {unreferenced:?}"),
            ));
        }
        let mut root = self.next_root(
            current,
            session.instance,
            checkpoint_txg,
            ModelJournalCounter(self.highest_journal_counter.0 + 1),
            kind,
            current.rollback_floor,
            None,
            Rc::clone(&current.instance_table_rows),
        );
        if let Some(intent) = issued {
            root.logged_compaction_intent = Some(intent);
        }
        if !seen.user_visible_root_units_relocated.is_empty() {
            let mut relocations = root.user_visible_root_relocations.as_ref().clone();
            for unit in &seen.user_visible_root_units_relocated {
                relocations.insert(*unit, checkpoint_txg);
            }
            root.user_visible_root_relocations = Rc::new(relocations);
        }
        Ok(self.answer_for_publishes(operation, BTreeSet::new(), vec![root]))
    }

    /// 可写会话现行那一版 logged_ops 里那一条活整理意图（没有可写会话或没有活意图时 None）：执行器拿它的 R 判实现交回的那一版 R 按记录全空没有。
    #[must_use]
    pub fn live_compaction_intent_of_the_current_version(&self) -> Option<ModelCompactionIntent> {
        self.session
            .as_ref()
            .and_then(|session| session.current.logged_compaction_intent)
    }

    /// 模型的现行版本还有没有要走空的（删除中的文件、销毁中的快照，或截断途中的文件，判决
    /// `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K1「截断进共用走空」）：执行器拿它分一次会话交回的那几版里哪一版是走空的一步。
    #[must_use]
    pub fn current_version_has_a_step_to_walk(&self) -> bool {
        self.current_version_has_a_file_being_deleted()
            || self.current_version_has_a_snapshot_being_destroyed()
            || self.current_version_has_a_file_being_truncated()
    }

    /// 模型的现行版本有没有销毁中的快照。
    fn current_version_has_a_snapshot_being_destroyed(&self) -> bool {
        self.session.as_ref().is_some_and(|session| {
            session
                .current
                .snapshots
                .iter()
                .any(|snapshot| snapshot.is_being_destroyed)
        })
    }

    /// 模型的现行版本是不是截断途中（logged_ops 里有一条截断条目，判决第三节 K1、K2）。
    #[must_use]
    pub fn current_version_has_a_file_being_truncated(&self) -> bool {
        self.session
            .as_ref()
            .is_some_and(|session| session.current.logged_truncation_issued_txg().is_some())
    }

    /// 截断这次用户改动（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K2、K3、K5、K11、K12；`mounted_session::UserChange::Truncate`
    /// 接线归下一批）：会话先走空（执行器把走空的那几步按「走空的一步」比过之后才问这一次），再接在现行那一版后面发：
    /// - N 按整单元向上取整溢出 u64：在任何写之前必须拒（[`ModelRefusalReason::TruncationSizeWhoseWholeDataUnitsOverflow`]，K12）；
    /// - 现行那一版有树而文件集合为空：同覆盖写必须拒（[`ModelRefusalReason::OnlyNewInodesOnAVersionWithTreesAndNoFile`]）；
    /// - N < S：发截断意图（[`ModelPublishKind::TruncationIntent`]），之后的各批是走空的一步（[`IdealModel::answer_truncation_step`]）；
    ///   N 落在存在的单元 k 中间、单元 k 的声明长度不等于 N − k × 净荷时重写它（K3）；不重写而上段叶条目的标签要变（缩短把尾部的洞整个截掉）
    ///   时意图那次只为换标签重写上段根（判决第六节 S1）；
    /// - N ≥ S：一次只改记录的发布（[`ModelPublishKind::SizeChangeWithoutShrinking`]；N = S 照 K12 发、不拒）：最后一个存在的单元声明长度小于
    ///   min(N − 起点, 净荷) 时补零重写它，不重写而洞标签要变时只重写 extent 上段根（K5）。
    ///
    /// 模型照字节记内容，N 要装得进这个进程的内存：执行器按单元的类取 N（缩到 0、单元边界、单元中间、单元内变长、跨单元变长、变长后截回洞里）。
    /// 第一个文件之外的 inode 改 size 由 [`IdealModel::answer_size_change_of_an_inode_other_than_the_first`] 答。
    ///
    /// # Errors
    /// 模型里没有可写会话，或现行版本没有带活文件的 inode 1（树表 0 条、删除中、截断途中、删完之后只建过 inode：执行器只在实现报带文件、队列空时调）：
    /// 会话状态与实现对不上；N 装不进这个进程的地址空间：[`ModelDisagreementAspect::NotModeled`]。
    pub fn answer_truncation(
        &self,
        new_size_in_bytes: u64,
    ) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        let payload = data_unit_payload_capacity_in_bytes();
        if new_size_in_bytes
            .div_ceil(payload)
            .checked_mul(payload)
            .is_none()
        {
            return Ok(self.answer_for_publishes(
                ModelOperationKind::PublishTruncation,
                BTreeSet::from([ModelRefusalReason::TruncationSizeWhoseWholeDataUnitsOverflow]),
                Vec::new(),
            ));
        }
        if let Some(refused) = self.answer_refusing_a_change_on_a_version_with_trees_and_no_file(
            ModelOperationKind::PublishTruncation,
        ) {
            return Ok(refused);
        }
        let ModelVersionContents::LiveFile(file) = &current.contents else {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::SessionState,
                format!(
                    "模型里现行版本 {:?} 没有带活文件、队列空的 inode 1：{:?}",
                    current.key, current.contents
                ),
                "实现的现行版本带文件、队列空（执行器调了截断）".to_string(),
            ));
        };
        let size_before = u64::try_from(file.content.len()).expect("内容长度");
        let new_size = usize::try_from(new_size_in_bytes).map_err(|_| {
            ModelDisagreement::new(
                ModelDisagreementAspect::NotModeled,
                format!("模型照字节记内容，{new_size_in_bytes} 字节装不进这个进程的地址空间"),
                "执行器调了截断".to_string(),
            )
        })?;
        let checkpoint_txg = ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1);
        let journal_counter = ModelJournalCounter(self.highest_journal_counter.0 + 1);
        let root = if new_size_in_bytes < size_before {
            let data_units_within_the_new_size = data_units_of_content(new_size_in_bytes);
            let tail_data_unit = data_units_within_the_new_size - 1;
            let tail_data_unit_declared_length = new_size_in_bytes - tail_data_unit * payload;
            // 意图那次把目标态的 size、尾单元、标签一起写下（K2「先落目标」，判决第六节 S1）：尾单元要重写就连上段根一起写；不重写而标签与
            // 目标态对不上（缩短把尾部的洞整个截掉，3、4 换回 1、2）就只为换标签重写上段根。size 之外还挂着的单元不算洞（S1），标签按
            // 存在的单元数与 N 罩的单元数取（[`ModelExtentUpperLeafEntryLabel::written_for`]）。
            let extent_tree = if tail_data_unit < file.data_units_present
                && file.declared_length_of_the_data_unit(tail_data_unit)
                    != tail_data_unit_declared_length
            {
                ModelExtentTreeChangeOfASizeChange::TailDataUnitRewritten
            } else if ModelExtentUpperLeafEntryLabel::written_for(
                file.data_units_present,
                data_units_within_the_new_size,
            ) != file.extent_upper_leaf_entry_label
            {
                ModelExtentTreeChangeOfASizeChange::OnlyTheUpperLeafEntryLabel
            } else {
                ModelExtentTreeChangeOfASizeChange::Untouched
            };
            self.next_root(
                current,
                session.instance,
                checkpoint_txg,
                journal_counter,
                ModelPublishKind::TruncationIntent { extent_tree },
                current.rollback_floor,
                Some(&file.content[..new_size]),
                Rc::clone(&current.instance_table_rows),
            )
        } else {
            let mut content = file.content.to_vec();
            content.resize(new_size, 0);
            let last_data_unit = file.data_units_present - 1;
            let last_data_unit_length_after =
                (new_size_in_bytes - last_data_unit * payload).min(payload);
            let extent_tree = if file.declared_length_of_the_data_unit(last_data_unit)
                < last_data_unit_length_after
            {
                ModelExtentTreeChangeOfASizeChange::TailDataUnitRewritten
            } else if ModelExtentUpperLeafEntryLabel::written_for(
                file.data_units_present,
                data_units_of_content(new_size_in_bytes),
            ) != file.extent_upper_leaf_entry_label
            {
                ModelExtentTreeChangeOfASizeChange::OnlyTheUpperLeafEntryLabel
            } else {
                ModelExtentTreeChangeOfASizeChange::Untouched
            };
            self.next_root(
                current,
                session.instance,
                checkpoint_txg,
                journal_counter,
                ModelPublishKind::SizeChangeWithoutShrinking { extent_tree },
                current.rollback_floor,
                Some(&content),
                Rc::clone(&current.instance_table_rows),
            )
        };
        Ok(self.answer_for_publishes(
            ModelOperationKind::PublishTruncation,
            BTreeSet::new(),
            vec![root],
        ))
    }

    /// 第一个文件之外的一个 inode 改 size（`UserChange::Truncate` 点名的不是 inode 1；判决
    /// `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S2、S3、S6：任意活 inode 都能缩短与变长，删完第一个文件之后才建的
    /// inode 也照这一条办）：先判 N 按整单元向上取整溢不溢出（K12，与第一个文件同一个次序），再接在现行那一版后面发：
    /// - N < S：截断意图（[`ModelSizeChangeOfAnInodeWithoutDataUnits::TruncationIntent`]），之后的最后一批是走空的一步
    ///   （[`IdealModel::answer_truncation_step`]，放 0 个单元）；
    /// - N ≥ S：一次只改记录的发布；上段里还没有它那一条而 N > 0 时这次起写粘住的标签 0
    ///   （[`ModelSizeChangeOfAnInodeWithoutDataUnits::WithoutShrinkingWritingTheTagZeroEntry`]），否则 extent 树整棵照抄。
    ///
    /// 没有数据单元的那种读回 N 个 0；带数据单元的照第一个文件那一套（[`ModelPublishKind::SizeChangeOfAnInodeHoldingDataUnits`]：N < S 走截断意图、
    /// 落在单元 k 中间而声明长度不等于 N − k × 净荷时重写单元 k，N ≥ S 一次发布、最后一个存在的单元声明长度小于 min(N − 起点, 净荷) 时补零重写，
    /// 不重写而洞标签要变时只重写它那一串上段，判决 m4-multi-inode-size-r1 第二节 S3、S5）。
    ///
    /// # Errors
    /// 模型里没有可写会话、现行版本不是第一个文件活着或已删完而树里还有别的 inode 的一版、`inode` 不是这一版树里第一个文件之外活着的号：
    /// 会话状态与实现对不上；N 装不进这个进程的地址空间：[`ModelDisagreementAspect::NotModeled`]。
    pub fn answer_size_change_of_an_inode_other_than_the_first(
        &self,
        inode: u64,
        new_size_in_bytes: u64,
    ) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        let payload = data_unit_payload_capacity_in_bytes();
        if new_size_in_bytes
            .div_ceil(payload)
            .checked_mul(payload)
            .is_none()
        {
            return Ok(self.answer_for_publishes(
                ModelOperationKind::PublishTruncation,
                BTreeSet::from([ModelRefusalReason::TruncationSizeWhoseWholeDataUnitsOverflow]),
                Vec::new(),
            ));
        }
        let first_file_records_in_the_tree = match current.contents {
            ModelVersionContents::LiveFile(_) => 1,
            ModelVersionContents::InodesWithoutTheFirstFile { .. } => 0,
            ModelVersionContents::WithoutTrees
            | ModelVersionContents::TreesAndNoFile { .. }
            | ModelVersionContents::FileBeingDeleted { .. }
            | ModelVersionContents::FileBeingTruncated { .. } => {
                return Err(ModelDisagreement::new(
                    ModelDisagreementAspect::SessionState,
                    format!(
                        "模型里现行版本 {:?} 不是第一个文件活着、或已删完而树里还有别的 inode 的一版：{:?}",
                        current.key, current.contents
                    ),
                    format!("实现的现行版本树里有 inode {inode}（执行器点名它改 size）"),
                ));
            }
        };
        // 记录按号排，第 r 条在左起第 ⌊r ÷ 233⌋ 片叶容器里（同按 inode 号写，[`IdealModel::answer_whole_write_of_an_inode`]）。
        let Some((position_among_the_inodes_other_than_the_first, changed_inode)) = current
            .inodes_other_than_the_first
            .iter()
            .enumerate()
            .find(|(_, (number, other_inode))| **number == inode && other_inode.is_live())
            .map(|(position, (_, other_inode))| (position, other_inode))
        else {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::SessionState,
                format!(
                    "模型里现行版本 {:?} 第一个文件之外的 inode 是 {:?}，没有活着的 {inode}",
                    current.key,
                    current
                        .inodes_other_than_the_first
                        .keys()
                        .collect::<Vec<_>>()
                ),
                format!("实现的现行版本树里有 inode {inode}（执行器点名它改 size）"),
            ));
        };
        if usize::try_from(new_size_in_bytes).is_err() {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::NotModeled,
                format!("模型照字节记内容，{new_size_in_bytes} 字节装不进这个进程的地址空间"),
                format!("执行器点名 inode {inode} 改 size"),
            ));
        }
        let record_position_in_the_tree = first_file_records_in_the_tree
            + u64::try_from(position_among_the_inodes_other_than_the_first).expect("条数");
        let inode_leaf_container = ModelUnitRole::inode_leaf_container_at(
            record_position_in_the_tree / INODE_LEAF_RECORDS,
        );
        let size_before = u64::try_from(changed_inode.content.len()).expect("内容长度");
        if changed_inode.data_units_present() > 0 {
            let direction = if new_size_in_bytes < size_before {
                ModelDirectionOfASizeChange::TruncationIntent
            } else {
                ModelDirectionOfASizeChange::WithoutShrinking
            };
            let root = self.next_root(
                current,
                session.instance,
                ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1),
                ModelJournalCounter(self.highest_journal_counter.0 + 1),
                ModelPublishKind::SizeChangeOfAnInodeHoldingDataUnits {
                    inode,
                    inode_leaf_container,
                    new_size_in_bytes,
                    direction,
                    extent_tree:
                        Self::extent_tree_change_of_a_size_change_of_an_inode_holding_data_units(
                            changed_inode,
                            new_size_in_bytes,
                            direction,
                        ),
                },
                current.rollback_floor,
                None,
                Rc::clone(&current.instance_table_rows),
            );
            return Ok(self.answer_for_publishes(
                ModelOperationKind::PublishTruncation,
                BTreeSet::new(),
                vec![root],
            ));
        }
        let change = if new_size_in_bytes < size_before {
            ModelSizeChangeOfAnInodeWithoutDataUnits::TruncationIntent
        } else if new_size_in_bytes > 0 && !changed_inode.keeps_a_tag_zero_upper_leaf_entry {
            ModelSizeChangeOfAnInodeWithoutDataUnits::WithoutShrinkingWritingTheTagZeroEntry
        } else {
            ModelSizeChangeOfAnInodeWithoutDataUnits::WithoutShrinkingCarryingTheExtentTree
        };
        let root = self.next_root(
            current,
            session.instance,
            ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1),
            ModelJournalCounter(self.highest_journal_counter.0 + 1),
            ModelPublishKind::SizeChangeOfAnInodeWithoutDataUnits {
                inode,
                inode_leaf_container,
                new_size_in_bytes,
                change,
            },
            current.rollback_floor,
            None,
            Rc::clone(&current.instance_table_rows),
        );
        Ok(self.answer_for_publishes(
            ModelOperationKind::PublishTruncation,
            BTreeSet::new(),
            vec![root],
        ))
    }

    /// 带数据单元的别的 inode 改 size 那一次对它的尾单元与上段叶条目做了什么（同第一个文件，[`IdealModel::answer_truncation`]；判决
    /// m4-multi-inode-size-r1 第二节 S3）：截断意图落在存在的单元 k 中间、单元 k 的声明长度不等于 N − k × 净荷时重写单元 k（K3），不缩短的那一次最后一个
    /// 存在的单元声明长度小于 min(N − 起点, 净荷) 时补零重写它（K5）；不重写而尾部有没有洞变了（标签要换，S1、K5）时只重写它那一串上段。
    /// 它那一条上段叶条目的标签恒是按存在的单元数与 size 写的那一个（[`ModelExtentUpperLeafEntryLabel::written_for`]：每一次改了单元数、尾单元或 size
    /// 罩的单元数的发布都重写它那一条），所以按改之前与之后的 size 各取一个比。
    fn extent_tree_change_of_a_size_change_of_an_inode_holding_data_units(
        changed_inode: &ModelInodeOtherThanTheFirst,
        new_size_in_bytes: u64,
        direction: ModelDirectionOfASizeChange,
    ) -> ModelExtentTreeChangeOfASizeChange {
        let payload = data_unit_payload_capacity_in_bytes();
        let data_units_present = changed_inode.data_units_present();
        let data_units_within_the_new_size = data_units_of_content(new_size_in_bytes);
        let tail_data_unit_is_rewritten = match direction {
            ModelDirectionOfASizeChange::TruncationIntent => {
                let tail_data_unit = data_units_within_the_new_size - 1;
                tail_data_unit < data_units_present
                    && changed_inode.declared_length_of_the_data_unit(tail_data_unit)
                        != new_size_in_bytes - tail_data_unit * payload
            }
            ModelDirectionOfASizeChange::WithoutShrinking => {
                let last_data_unit = data_units_present - 1;
                changed_inode.declared_length_of_the_data_unit(last_data_unit)
                    < (new_size_in_bytes - last_data_unit * payload).min(payload)
            }
        };
        if tail_data_unit_is_rewritten {
            ModelExtentTreeChangeOfASizeChange::TailDataUnitRewritten
        } else if ModelExtentUpperLeafEntryLabel::written_for(
            data_units_present,
            data_units_within_the_new_size,
        ) != ModelExtentUpperLeafEntryLabel::written_for(
            data_units_present,
            changed_inode.data_units_within_the_size(),
        ) {
            ModelExtentTreeChangeOfASizeChange::OnlyTheUpperLeafEntryLabel
        } else {
            ModelExtentTreeChangeOfASizeChange::Untouched
        }
    }

    /// 走空截断的一步（判决第三节 K2、K11：会话里截断意图之后、每次用户改动之前，挂载续做）：size 之外还挂着的单元多于批大小 144（与 unlink
    /// 共用）就是不是最后一批的一批，从尾部放 144 个；不多于就是最后一批，放完剩下的（可以 0 个）、同一次发布删截断条目。下一步只由现行那一版定。
    ///
    /// # Errors
    /// 模型里没有可写会话，或现行版本不在截断途中：会话状态与实现对不上。
    pub fn answer_truncation_step(&self) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        // 截断条目点名第一个文件之外的 inode（判决 m4-multi-inode-size-r1 第二节 S3）：没有数据单元的 size 之外一个单元都没有，下一步就是放 0 个的最后一批；
        // 带数据单元的同第一个文件，size 之外还挂着的多于批大小 144 就从尾部放 144 个，不多于就是最后一批（可以 0 个）。
        if let Some((inode, truncated_inode)) = current
            .inodes_other_than_the_first
            .iter()
            .find(|(_, other_inode)| other_inode.truncation_issued_txg.is_some())
            .map(|(number, other_inode)| (*number, other_inode))
        {
            let logged_operations_after = if current.logged_compaction_intent.is_some() {
                ModelLoggedOperationsAfterTheTruncationEntryIsDeleted::StillHoldsTheCompactionIntent
            } else {
                ModelLoggedOperationsAfterTheTruncationEntryIsDeleted::Emptied
            };
            let data_units_beyond_the_size = truncated_inode
                .data_units_present()
                .saturating_sub(truncated_inode.data_units_within_the_size());
            let step = if truncated_inode.data_units_present() == 0 {
                ModelPublishKind::FinalTruncationBatchOfAnInodeWithoutDataUnits {
                    inode,
                    logged_operations_after,
                }
            } else if data_units_beyond_the_size > MODEL_DELETION_BATCH_DATA_UNITS {
                ModelPublishKind::TruncationBatchOfAnInodeHoldingDataUnits { inode }
            } else {
                ModelPublishKind::FinalTruncationBatchOfAnInodeHoldingDataUnits {
                    inode,
                    release: if data_units_beyond_the_size > 0 {
                        ModelFinalTruncationBatchRelease::AtLeastOneDataUnit
                    } else {
                        ModelFinalTruncationBatchRelease::NoDataUnit
                    },
                    logged_operations_after,
                }
            };
            let root = self.next_root(
                current,
                session.instance,
                ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1),
                ModelJournalCounter(self.highest_journal_counter.0 + 1),
                step,
                current.rollback_floor,
                None,
                Rc::clone(&current.instance_table_rows),
            );
            return Ok(self.answer_for_publishes(
                ModelOperationKind::PublishTruncationStep,
                BTreeSet::new(),
                vec![root],
            ));
        }
        let ModelVersionContents::FileBeingTruncated { file, .. } = &current.contents else {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::SessionState,
                format!(
                    "模型里现行版本 {:?} 不在截断途中：{:?}",
                    current.key, current.contents
                ),
                "实现交回了走空截断的一步".to_string(),
            ));
        };
        // 截回洞里的（存在的单元不多于 size 罩的）size 之外一个都没有。
        let data_units_beyond_the_size = file
            .data_units_present
            .saturating_sub(file.data_units_within_the_size());
        let step = if data_units_beyond_the_size > MODEL_DELETION_BATCH_DATA_UNITS {
            ModelPublishKind::TruncationBatch
        } else {
            ModelPublishKind::FinalTruncationBatch {
                release: if data_units_beyond_the_size > 0 {
                    ModelFinalTruncationBatchRelease::AtLeastOneDataUnit
                } else {
                    ModelFinalTruncationBatchRelease::NoDataUnit
                },
                logged_operations_after: if current.logged_compaction_intent.is_some() {
                    ModelLoggedOperationsAfterTheTruncationEntryIsDeleted::StillHoldsTheCompactionIntent
                } else {
                    ModelLoggedOperationsAfterTheTruncationEntryIsDeleted::Emptied
                },
            }
        };
        let root = self.next_root(
            current,
            session.instance,
            ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1),
            ModelJournalCounter(self.highest_journal_counter.0 + 1),
            step,
            current.rollback_floor,
            None,
            Rc::clone(&current.instance_table_rows),
        );
        Ok(self.answer_for_publishes(
            ModelOperationKind::PublishTruncationStep,
            BTreeSet::new(),
            vec![root],
        ))
    }

    /// 读按建的先后第 `position` 个快照（`recovery::read_the_first_file_of_a_snapshot` 经现行那一版的根，判决第三节 S7「读快照」）：
    /// 读出的要是建它那一刻读得出的文件内容（那一刻没有活文件就是「没有文件」）。读不写盘，模型不动。
    ///
    /// # Errors
    /// 模型里没有可写会话、没有第 `position` 个快照：会话状态与实现对不上；读出的与模型记的不同：读快照那一格对不上。
    pub fn judge_snapshot_read_back(
        &self,
        position: usize,
        observed: &ObservedSnapshotReadBack,
    ) -> Result<ModelJudgementCounts, ModelDisagreement> {
        let session = self.open_session()?;
        let Some(snapshot) = session.current.snapshots.get(position) else {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::SessionState,
                format!(
                    "模型里现行版本 {:?} 没有第 {position} 个快照：{:?}",
                    session.current.key, session.current.snapshots
                ),
                "实现的现行版本里有这个快照（执行器读了它）".to_string(),
            ));
        };
        let expected = match &snapshot.content_at_creation {
            Some(content) => ModelSnapshotReadBack::File(Rc::clone(content)),
            None => ModelSnapshotReadBack::NoFile,
        };
        let matches = match (&expected, observed) {
            (ModelSnapshotReadBack::NoFile, ObservedSnapshotReadBack::NoFile) => true,
            (
                ModelSnapshotReadBack::File(expected_content),
                ObservedSnapshotReadBack::File(content),
            ) => expected_content.as_ref() == content.as_slice(),
            (
                ModelSnapshotReadBack::NoFile | ModelSnapshotReadBack::File(_),
                ObservedSnapshotReadBack::NoFile
                | ObservedSnapshotReadBack::File(_)
                | ObservedSnapshotReadBack::NoSnapshotWithThisTree
                | ObservedSnapshotReadBack::Failed { .. },
            ) => false,
        };
        if !matches {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::SnapshotReadBack,
                format!(
                    "第 {position} 个快照（诞生 txg {}）读出 {}",
                    snapshot.birth_txg.0,
                    describe_snapshot_read_back(&expected)
                ),
                describe_observed_snapshot_read_back(observed),
            ));
        }
        let mut counts = ModelJudgementCounts::default();
        counts.snapshot_read_backs_compared += 1;
        Ok(counts)
    }

    /// 现行那一版有树而文件集合为空时，这次不是建 inode 的改动（覆盖写、unlink）必须拒（D8（核心索引结构） 已定项 16 第 5 条，
    /// [`ModelRefusalReason::OnlyNewInodesOnAVersionWithTreesAndNoFile`]），一条根都不写；别的版本交 `None`，照各自的答法。
    fn answer_refusing_a_change_on_a_version_with_trees_and_no_file(
        &self,
        operation: ModelOperationKind,
    ) -> Option<ModelAnswer> {
        let session = self.session.as_ref()?;
        match session.current.contents {
            ModelVersionContents::TreesAndNoFile { .. } => Some(self.answer_for_publishes(
                operation,
                BTreeSet::from([ModelRefusalReason::OnlyNewInodesOnAVersionWithTreesAndNoFile]),
                Vec::new(),
            )),
            ModelVersionContents::WithoutTrees
            | ModelVersionContents::LiveFile(_)
            | ModelVersionContents::FileBeingDeleted { .. }
            | ModelVersionContents::InodesWithoutTheFirstFile { .. }
            | ModelVersionContents::FileBeingTruncated { .. } => None,
        }
    }

    /// 模型的现行版本有没有删除中的号（`deleted_inodes` 不空：第一个文件或别的文件）：执行器拿它分一次会话交回的那几版里哪一版是走空的一步。
    #[must_use]
    pub fn current_version_has_a_file_being_deleted(&self) -> bool {
        self.session
            .as_ref()
            .is_some_and(|session| session.current.inode_being_deleted().is_some())
    }

    fn answer_for_publishes(
        &self,
        operation: ModelOperationKind,
        required_refusals: BTreeSet<ModelRefusalReason>,
        roots: Vec<ModelRoot>,
    ) -> ModelAnswer {
        ModelAnswer {
            operation,
            required_refusals,
            planned_publish_upper_bounds: Self::upper_bounds_of(&roots),
            expected_roots: roots,
            expected_mount: None,
            expected_read_back: None,
            rollback_floor_ceiling: None,
            rollback_target: None,
            walls_are_judged_before_the_first_write: false,
            session_after_success: ModelSessionAfterSuccess::StaysOpen,
        }
    }

    /// 可写挂载（`mount::mount_writable`）：所选根 = 环里最新的那条（不建崩溃，没有要施加的记录）。调之前先 `close_session`。
    #[must_use]
    pub fn answer_mount_writable(&self) -> ModelAnswer {
        let chosen = self.newest_root().clone();
        let previous_row = ModelInstanceRow {
            instance: chosen.key.instance,
            selected_root_txg: chosen.key.checkpoint_txg,
            // W = 这次恢复施加的记录里最大的事务号（D23（journal 的角色与格式） 已定项 14 第 4 条）：不建崩溃，一条都不施加。
            applied_transaction_high_water: 0,
        };
        self.answer_establishing_an_instance(
            ModelOperationKind::MountWritable,
            &chosen,
            Some(previous_row),
        )
    }

    /// 崩溃恢复抛弃根（执行器的 `HistoryOperation::CrashRecoveryAbandoningTheNewestRoot`）：挂载期间环里最新那条根的根槽与它那次发布
    /// 点名的单元整次挂载读回全 0。那条根是这个进程发的，系统配置在它的根槽 FUA 之后轮换（D16（发布语义） 已定项 7），见证着它；
    /// 读阶段落到它前一条根、判出系统配置见证过比所选那一版新的发布，重读一次仍全 0，取号之前拒可写（C554 乙，用户 2026-09-27 定，
    /// `research/prompts/c554-fix-forks.md` #1「挂载时有一条根（或它引用的账）暂时读不出」；重读一次取自 D16（发布语义） 已定项 1「根槽这一次读坏」那一行）：一个字节都不写，
    /// 模型状态不变（会话照旧关着，根环、F、取过的号都不动）。系统配置没见证到最新那条根的那一形（它那次发布的轮换没落盘）乙罩不到、
    /// 照旧抛弃它，这一步造不出，模型不答。调之前先 `close_session`。
    ///
    /// # Errors
    /// 环里只有一条根：读阶段没有前一条可落（执行器在盘上的根环少于两条时不调，这一支只在模型与盘上的根环对不上时走到）。
    pub fn answer_mount_writable_with_the_newest_root_unreadable(
        &self,
    ) -> Result<ModelAnswer, ModelDisagreement> {
        let newest = self.newest_root().key;
        if self.ring.len() < 2 {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::NotModeled,
                format!("模型根环里只有 {newest:?} 一条根，崩溃恢复没有前一条根可落"),
                "执行器按盘上的根环调了崩溃恢复抛弃根".to_string(),
            ));
        }
        let mut answer = self.answer_for_publishes(
            ModelOperationKind::MountWritable,
            BTreeSet::from([
                ModelRefusalReason::NewerPublishWitnessedBySystemConfigurationStillUnreadableAfterOneReread,
            ]),
            Vec::new(),
        );
        // 可写挂载在读阶段判完、取号之前拒：拒之前一个写都不发（实现的契约：`NewerStateStillUnreadableAfterOneReread` 取号之前交回）。
        answer.walls_are_judged_before_the_first_write = true;
        Ok(answer)
    }

    /// 可写挂载，而 `roots_whose_root_slot_is_missing` 那几条根的根槽写没落盘（说谎的设备吞了；它们的记录与单元都落了）：
    /// 择根落到除它们之外 (txg, 实例) 最大的那一条（所选根），恢复再施加 (实例代号, checkpoint_txg) 严格大于所选根、与所选根同实例的
    /// 那几次发布的记录（前缀不跨实例边界，D23（journal 的角色与格式） 已定项 14 注 1），由记录重建出它们；上一个实例那一行的 T 取
    /// 实际走到的那条根，W 取施加的那几次发布的记录里最大的事务号（已定项 14 第 4 条；空发布的事务号 0 不进 max，已定项 19 ①）。
    /// 名单里的根不在模型的根环里、或比所选根旧的，不改答案。调之前先 `close_session`。
    /// 故障注入拿它认「写的实例表行」那一形（实八）：实现照条款写 W，[`IdealModel::answer_mount_writable`] 不建崩溃、答 W = 0。
    ///
    /// # Errors
    /// 名单把根环里的根拿光了：择根没有一条可落。
    pub fn answer_mount_writable_with_the_roots_rebuilt_from_their_records(
        &self,
        roots_whose_root_slot_is_missing: &[ModelRootKey],
    ) -> Result<ModelAnswer, ModelDisagreement> {
        let mut view_without_the_missing_roots = self.clone();
        view_without_the_missing_roots
            .ring
            .retain(|_, root| !roots_whose_root_slot_is_missing.contains(&root.key));
        if view_without_the_missing_roots.ring.is_empty() {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::NotModeled,
                format!(
                    "模型根环里的根全在根槽没落盘的名单里（{roots_whose_root_slot_is_missing:?}），择根没有一条可落"
                ),
                "执行器调了可写挂载".to_string(),
            ));
        }
        let selected = view_without_the_missing_roots.newest_root().clone();
        let mut newer_than_the_selected: Vec<&ModelRoot> = self
            .ring
            .values()
            .filter(|root| root.key > selected.key)
            .collect();
        newer_than_the_selected.sort_by_key(|root| root.key);
        // 迭代上界是根环的槽数；跨轮带的是走到的那条根与 W；提前出口只有「下一条换了实例」（前缀不跨实例边界）。
        let mut effective = &selected;
        let mut applied_transaction_high_water = 0;
        for rebuilt in newer_than_the_selected {
            if rebuilt.key.instance != selected.key.instance {
                break;
            }
            applied_transaction_high_water = applied_transaction_high_water
                .max(rebuilt.highest_transaction_number_of_its_publish);
            effective = rebuilt;
        }
        let previous_row = ModelInstanceRow {
            instance: effective.key.instance,
            selected_root_txg: effective.key.checkpoint_txg,
            applied_transaction_high_water,
        };
        Ok(self.answer_establishing_an_instance(
            ModelOperationKind::MountWritable,
            effective,
            Some(previous_row),
        ))
    }

    /// 管理员回退（`mount::roll_back_by_a_forward_publish`，D23（journal 的角色与格式） 已定项 14）：挂着时的一次向前发布。
    /// 候选集 = 根环里按现行那一版的实例表判仍然有效 ∧ txg ≥ F_生效 ∧ 带文件；做成时写一条根——txg 与 jsn 接着现行那一版，
    /// 实例、实例表、F 照现行那一版的，文件与用户可见的几个角色（数据、extent 根、inode 叶与根）的分配代取回退目标那一版的，
    /// 固定点单元（分配记录树、记账树、映射树、树表）这次重写。模型不建坏盘，「在任何写之前拒」的账与单元那几格走不到。
    ///
    /// # Errors
    /// 模型里没有可写会话，或现行版本没有文件（执行器只在带文件时调）。
    pub fn answer_rollback_while_mounted(
        &self,
        target: ModelRootKey,
    ) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        if !current
            .contents
            .is_a_version_with_a_file_or_a_file_being_deleted()
        {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::NotModeled,
                format!(
                    "模型里现行版本 {:?} 没有文件：{:?}",
                    current.key, current.contents
                ),
                "实现的现行版本带文件（执行器调了回退）".to_string(),
            ));
        }
        let mut required_refusals = BTreeSet::new();
        // 第一版有快照就拒回退（判决 S8、岔路 3）：现行那一版带着快照（销毁中的也算）时在任何读之前拒，别的候选判据不再看。
        if !current.snapshots.is_empty() {
            required_refusals
                .insert(ModelRefusalReason::RollbackWithSnapshotsWhichTheFirstVersionRefuses);
            let mut answer = self.answer_for_publishes(
                ModelOperationKind::RollbackWhileMounted,
                required_refusals,
                Vec::new(),
            );
            answer.rollback_target = Some(target);
            answer.walls_are_judged_before_the_first_write = true;
            return Ok(answer);
        }
        let target_root = self.root_with_key(target);
        match target_root {
            None => {
                required_refusals.insert(ModelRefusalReason::RollbackTargetNotInRing);
            }
            Some(target_root) => {
                if target.checkpoint_txg < self.effective_rollback_floor() {
                    required_refusals.insert(ModelRefusalReason::RollbackTargetBelowEffectiveFloor);
                }
                if Self::is_abandoned_by(target_root, &current.instance_table_rows) {
                    required_refusals.insert(ModelRefusalReason::RollbackTargetOnAbandonedTimeline);
                }
                if !target_root.contents.has_trees() {
                    required_refusals.insert(ModelRefusalReason::RollbackTargetWithoutFile);
                }
                // 目标那一版带快照：实现在候选集那一判之后、把目标那一版重建出来再判（`mount::roll_back_by_a_forward_publish`），
                // 所以只在候选判据都过了时才是这一条。
                if required_refusals.is_empty() && !target_root.snapshots.is_empty() {
                    required_refusals.insert(
                        ModelRefusalReason::RollbackWithSnapshotsWhichTheFirstVersionRefuses,
                    );
                }
            }
        }
        let roots = match (target_root, required_refusals.is_empty()) {
            (Some(target_root), true) => {
                let rollback_txg = ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1);
                let rollback_root = self.next_root_with_the_user_visible_trees(
                    current,
                    session.instance,
                    rollback_txg,
                    ModelJournalCounter(self.highest_journal_counter.0 + 1),
                    ModelPublishKind::EmptyOnFileVersion,
                    current.rollback_floor,
                    None,
                    Rc::clone(&current.instance_table_rows),
                    UserVisibleTreesOfTheNewVersion::TakenFromTheRollbackTarget(target_root),
                    InodesOtherThanTheFirstChangeOfThePublish::Carried,
                );
                vec![rollback_root]
            }
            (Some(_) | None, _) => Vec::new(),
        };
        let mut answer = self.answer_for_publishes(
            ModelOperationKind::RollbackWhileMounted,
            required_refusals,
            roots,
        );
        answer.rollback_target = Some(target);
        // 回退那一步在任何写之前判完它的拒（候选集、账、单元），做成就是一次发布。
        answer.walls_are_judged_before_the_first_write = true;
        Ok(answer)
    }

    /// 可写挂载与回退共用的后半段：取号、写行那次发布、暖机到本实例的根覆盖每块盘（D16（发布语义） 已定项 8 戊，至多 R 次）。
    /// `previous_row` 为 None 只在 mkfs 同一个进程里（不写行）。
    fn answer_establishing_an_instance(
        &self,
        operation: ModelOperationKind,
        base: &ModelRoot,
        previous_row: Option<ModelInstanceRow>,
    ) -> ModelAnswer {
        let instance = ModelInstanceGeneration(self.highest_acquired_instance.0 + 1);
        // 行：给 [max(上一个实例, 1), 新实例) 里每个实例一行——上一个实例那一行，中间实例 (i, 0, 0)；实例 0 不写行
        // （D18（块里携带什么信息） 已定项 11；D23（journal 的角色与格式） 已定项 14）。
        let rows_to_write: Vec<ModelInstanceRow> = match previous_row {
            None => Vec::new(),
            Some(previous_row) => (previous_row.instance.0.max(1)..instance.0)
                .map(|row_instance| {
                    if row_instance == previous_row.instance.0 {
                        previous_row
                    } else {
                        ModelInstanceRow {
                            instance: ModelInstanceGeneration(row_instance),
                            selected_root_txg: ModelCheckpointTxg(0),
                            applied_transaction_high_water: 0,
                        }
                    }
                })
                .collect(),
        };
        // 新实例的第一次发布的 txg 与 jsn：环里全部根与全部记录里的最大值 + 1（D23（journal 的角色与格式） 已定项 14 第 3 条）。
        let first_txg = ModelCheckpointTxg(self.highest_published_txg.0 + 1);
        let first_journal_counter = ModelJournalCounter(self.highest_journal_counter.0 + 1);
        // 新实例的根带的 F = 恢复后生效的 F（D16（发布语义） 已定项 1「生效」）：预想，跟收口表第 ② 行。
        let rollback_floor = self.effective_rollback_floor();
        let required_refusals = BTreeSet::new();
        let has_trees = base.contents.has_trees();
        // 树表 0 条、而这一版的实例表已经不是 mkfs 那一片（上一次挂载在这一版上写过行）**此前是必拒的一格**：
        // 重建账时只剩根记录那两条指针，被换下的那一片成了空闲槽。C512（树表 0 条的一版上被换下的单元记在哪）
        // 2026-09-23 定案之后，写行那次发布建起这一版自己的分配记录树、根指针住根记录 ⇒ 账取得回来，这一格不再拒。
        // 模型因此**一条都不列**：实现要是还在这一格上拒，对拍当场报「模型说该成、实现拒了」——
        // 那正是这条定案要盯住的回退面。`MountError::VersionWithoutFileNotWrittenByMakeFilesystem` 今天只剩
        // 「树表或实例表不是 mkfs 写的那一版、而这一版又没有自己的分配记录树」那种手造镜像走得到，随机历史里造不出来。
        // 实例表多于一片不再拒（用户 2026-09-24 定尾片先、一片写满 369 行再开下一片）：写行那次发布整条链重写，
        // 有几片就多几个实例表单元（`next_root` 按这次之后的行数现算片数）。
        let table_after: Rc<Vec<ModelInstanceRow>> = if rows_to_write.is_empty() {
            Rc::clone(&base.instance_table_rows)
        } else {
            let mut rows = base.instance_table_rows.as_ref().clone();
            rows.extend_from_slice(&rows_to_write);
            Rc::new(rows)
        };
        let (row_kind, warm_up_kind) = match (has_trees, rows_to_write.is_empty()) {
            (true, _) => (
                ModelPublishKind::RowsOnFileVersion,
                ModelPublishKind::EmptyOnFileVersion,
            ),
            // 树表 0 条、要写的行不为空：只重写实例表那一个单元；之后的暖机仍是零单元。
            (false, false) => (
                ModelPublishKind::RowsOnVersionWithoutFile,
                ModelPublishKind::ZeroUnit,
            ),
            (false, true) => (ModelPublishKind::ZeroUnit, ModelPublishKind::ZeroUnit),
        };
        let mut roots = vec![self.next_root(
            base,
            instance,
            first_txg,
            first_journal_counter,
            row_kind,
            rollback_floor,
            None,
            Rc::clone(&table_after),
        )];
        let mut covered: BTreeSet<ModelDeviceIdentity> =
            BTreeSet::from([device_holding_the_root_of(first_txg)]);
        let mut warm_up_publishes: u64 = 0;
        while self
            .geometry
            .devices
            .iter()
            .any(|device| !covered.contains(device))
            && warm_up_publishes < ROOT_RING_REGIONS
        {
            let previous = roots.last().expect("至少有写行那一次").clone();
            let next_txg = ModelCheckpointTxg(previous.key.checkpoint_txg.0 + 1);
            covered.insert(device_holding_the_root_of(next_txg));
            roots.push(self.next_root(
                &previous,
                instance,
                next_txg,
                ModelJournalCounter(previous.journal_counter.0 + 1),
                warm_up_kind,
                rollback_floor,
                None,
                Rc::clone(&table_after),
            ));
            warm_up_publishes += 1;
        }
        let mut answer = self.answer_for_publishes(operation, required_refusals, roots);
        answer.walls_are_judged_before_the_first_write = true;
        answer.expected_mount = Some((instance, rows_to_write));
        answer.session_after_success = ModelSessionAfterSuccess::OpenedAs(instance);
        answer
    }

    /// 抬 F（`mount::raise_rollback_floor`）到 `new_floor`：推空发布直到每块盘上都有带新 F 的根（D16（发布语义） 已定项 1「生效」），
    /// 至多 R 次。
    ///
    /// # Errors
    /// 模型里没有可写会话或现行版本没有文件（执行器只在带文件时调）；`new_floor` 低于现行的 F（条款没写往下抬，生成器只取现行 F 及以上，
    /// 走到这里说明模型与实现的 F 已经对不上）。
    pub fn answer_raise_rollback_floor(
        &self,
        new_floor: ModelCheckpointTxg,
    ) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        // 抬 F 的空发布要重写固定点单元，接在有树的一版后面：执行器只在带文件（含删除中）的一版上调，
        // 挂载与会话为空间推的那几串还会接在有树而文件集合为空的一版后面。
        if !current.contents.has_trees() {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::SessionState,
                format!("模型里现行版本 {:?} 树表 0 条", current.key),
                "实现的现行版本有树（执行器调了抬 F，或挂载、会话推了抬 F）".to_string(),
            ));
        }
        if new_floor < current.rollback_floor {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::NotModeled,
                format!(
                    "往下抬 F（{} < 现行 {}）条款没写，模型不答",
                    new_floor.0, current.rollback_floor.0
                ),
                "执行器按实现的现行 F 取了目标".to_string(),
            ));
        }
        let ceiling = self.rollback_floor_ceiling();
        let mut required_refusals = BTreeSet::new();
        match ceiling {
            Some(ceiling) if new_floor > ceiling => {
                required_refusals.insert(ModelRefusalReason::FloorAboveCeiling);
            }
            Some(_) => {}
            None => {
                return Err(ModelDisagreement::new(
                    ModelDisagreementAspect::NotModeled,
                    "模型里一条有效根都没有，上限算不出".to_string(),
                    "实现调了抬 F".to_string(),
                ))
            }
        }
        let roots = self.empty_publishes_carrying_the_floor_to_every_device(session, new_floor);
        let mut answer = self.answer_for_publishes(
            ModelOperationKind::RaiseRollbackFloor,
            required_refusals,
            roots,
        );
        answer.rollback_floor_ceiling = ceiling;
        // 实现在任何写之前把这一串整串预演一遍（`mount::raise_rollback_floor`），哪一次撞墙都在第一次写之前拒、一次都不发。
        answer.walls_are_judged_before_the_first_write = true;
        Ok(answer)
    }

    /// 抬 F 那一串（D16（发布语义） 已定项 1「抬 F 那一串」：准入与卸载共用）接在会话现行那一版后面写出的根：带 `new_floor` 的空发布
    /// （有树的一版上重写固定点单元），推到每块盘上都有一条为止（「生效」那一行），至多 R 次。
    /// 先写系统配置那一步不属于任何一次发布（已定项 7），模型不建系统配置，这里只有根。
    fn empty_publishes_carrying_the_floor_to_every_device(
        &self,
        session: &ModelSession,
        new_floor: ModelCheckpointTxg,
    ) -> Vec<ModelRoot> {
        let current = &session.current;
        let mut roots: Vec<ModelRoot> = Vec::new();
        let mut covered: BTreeSet<ModelDeviceIdentity> = BTreeSet::new();
        // 迭代上界是根环区域数 R；跨轮带的是已写出的根与已罩到的盘；没有提前出口。
        while self
            .geometry
            .devices
            .iter()
            .any(|device| !covered.contains(device))
            && u64::try_from(roots.len()).expect("次数") < ROOT_RING_REGIONS
        {
            let previous = roots.last().unwrap_or(current).clone();
            let next_txg = ModelCheckpointTxg(previous.key.checkpoint_txg.0 + 1);
            covered.insert(device_holding_the_root_of(next_txg));
            let journal_counter = if roots.is_empty() {
                ModelJournalCounter(self.highest_journal_counter.0 + 1)
            } else {
                ModelJournalCounter(previous.journal_counter.0 + 1)
            };
            roots.push(self.next_root(
                &previous,
                session.instance,
                next_txg,
                journal_counter,
                ModelPublishKind::EmptyOnFileVersion,
                new_floor,
                None,
                Rc::clone(&previous.instance_table_rows),
            ));
        }
        roots
    }

    /// 正常卸载（`mount::unmount`，D16（发布语义） 已定项 1「正常卸载」）：推一串空发布把 F 抬到现行那一版的 txg，走抬 F 那一串
    /// （推到每块盘上都有一条带新 F 的根，第一版几何上 2 或 3 次），不另判上限（「抬 F 的上限」那一行「卸载抬 F」）；
    /// 做完之后这个进程不再有可写会话。
    ///
    /// 现行那一版树表 0 条（还没写过文件）：一个字节都不写、F 不动，答零条根、会话关
    /// （用户 2026-10-04 弹窗定，选项「乙：不推，条款照实现写」，2026-09-26 那条定案；与 `mount::Unmounted::NothingWrittenOnAVersionWithoutFile` 对得上）。
    /// 回退候选要带文件，而这一版在它的时间线上还没有带文件的版本，抬 F 买不到东西。
    ///
    /// 模型不比的两样：这一串的根在 flags 里带卸载记号（D22（单元原子性怎么合成） 已定项 7）——[`ObservedRoot`] 不带 flags；
    /// 先写系统配置那一步——模型不建系统配置。容量墙同抬 F：实现在任何写之前把这一串整串预演一遍，模型答允许拒绝的区间。
    ///
    /// # Errors
    /// 模型里没有可写会话：会话状态与实现对不上。
    pub fn answer_unmount(&self) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        let roots = if current.contents.has_trees() {
            let floor_raised_to_the_current_txg = current.key.checkpoint_txg;
            self.empty_publishes_carrying_the_floor_to_every_device(
                session,
                floor_raised_to_the_current_txg,
            )
        } else {
            Vec::new()
        };
        let mut answer =
            self.answer_for_publishes(ModelOperationKind::Unmount, BTreeSet::new(), roots);
        // 实现在任何写之前把这一串整串预演一遍（`mount::unmount` 走的就是抬 F 那一串），哪一次撞墙都在第一次写之前拒、一次都不发。
        answer.walls_are_judged_before_the_first_write = true;
        answer.session_after_success = ModelSessionAfterSuccess::Closed;
        Ok(answer)
    }

    /// 冷启动读回（`recovery::recover`，看 journal）：所选根 = 环里 (txg, 实例) 最大的那条，读回它那一版的文件。调之前先 `close_session`。
    #[must_use]
    pub fn answer_cold_start_recover(&self) -> ModelAnswer {
        let newest = self.newest_root();
        let read_back = match newest.contents.live_file() {
            Some(file) => ModelReadBack::FileRead {
                root: newest.key,
                content: Rc::clone(&file.content),
            },
            None => ModelReadBack::NoFile { root: newest.key },
        };
        ModelAnswer {
            operation: ModelOperationKind::ColdStartRecover,
            required_refusals: BTreeSet::new(),
            expected_roots: Vec::new(),
            expected_mount: None,
            expected_read_back: Some(read_back),
            rollback_floor_ceiling: None,
            rollback_target: None,
            planned_publish_upper_bounds: Vec::new(),
            walls_are_judged_before_the_first_write: true,
            session_after_success: ModelSessionAfterSuccess::Closed,
        }
    }

    /// 容量墙的区间（收口表第 39 行那种：条款把答案留给实现取上界，模型答允许拒绝的区间）。分配记录树按位置寻址之后
    /// （D8（核心索引结构） 已定项 14）没有「一个节点装不下」那道墙，剩单元区这一道：
    /// - 单元区：真条数那一头不判；上界那一头是占槽上界 × 一个聚簇段的槽数超过单元区（每个落点最坏独占一段：已分配 + defer ≤ 2 × 上界，
    ///   保留池 10 + 7 c_max 与切换预留（D16（发布语义） 已定项 1；D28（挂载期承诺量） 已定项 3）在 64 倍里）。预想：D28 已定项 1 的准入式子
    ///   第一版没实现，各项没有现值。
    fn capacity_wall_is_permitted(
        &self,
        reason: ModelRefusalReason,
        answer: &ModelAnswer,
        publishes_completed: usize,
    ) -> bool {
        // 一串判完的（可写挂载、回退、抬 F）看计划里的每一次；逐次判的（发布）只看拒的那一次。
        let judged_publishes: &[PlannedPublishUpperBounds] =
            if answer.walls_are_judged_before_the_first_write {
                &answer.planned_publish_upper_bounds
            } else {
                answer
                    .planned_publish_upper_bounds
                    .get(publishes_completed..=publishes_completed)
                    .unwrap_or(&[])
            };
        match reason {
            ModelRefusalReason::UnitAreaWall => judged_publishes.iter().any(|planned| {
                planned.occupied_slots_per_device * CLUSTER_SEGMENT_SLOTS
                    > self.unit_area_slots_per_device()
            }),
            ModelRefusalReason::ContentExceedsDataUnitPayload
            | ModelRefusalReason::FirstFileVersionDoesNotFollowTheVersionItBuildsOn
            | ModelRefusalReason::FirstFileVersionOnAVersionThatAlreadyHasAFile
            | ModelRefusalReason::RollbackTargetNotInRing
            | ModelRefusalReason::RollbackTargetBelowEffectiveFloor
            | ModelRefusalReason::RollbackTargetOnAbandonedTimeline
            | ModelRefusalReason::RollbackTargetWithoutFile
            | ModelRefusalReason::VersionWithoutFileNotWrittenByMakeFilesystem
            | ModelRefusalReason::FloorAboveCeiling
            | ModelRefusalReason::NewerPublishWitnessedBySystemConfigurationStillUnreadableAfterOneReread
            | ModelRefusalReason::OnlyNewInodesOnAVersionWithTreesAndNoFile
            | ModelRefusalReason::RollbackWithSnapshotsWhichTheFirstVersionRefuses
            | ModelRefusalReason::MoreSnapshotsThanTheFirstVersionAllows
            | ModelRefusalReason::TruncationSizeWhoseWholeDataUnitsOverflow
            | ModelRefusalReason::DataWriteToAnInodeBornAfterTheFirstFileWasDeleted
            | ModelRefusalReason::OldDataUnitToRewriteNotReadBack => false,
        }
    }

    /// 拿实现的结局与模型的答案比；对得上就按模型自己算的结果往前走（不按实现交回的），对不上交回哪一格。
    ///
    /// # Errors
    /// 对不上的第一格。
    pub fn judge_and_advance(
        &mut self,
        answer: &ModelAnswer,
        observed: &ObservedOutcome,
    ) -> Result<ModelJudgementCounts, ModelDisagreement> {
        let mut counts = ModelJudgementCounts::default();
        match observed {
            ObservedOutcome::Refused {
                member,
                reason,
                publishes_completed,
                wrote_anything,
                reported_ceiling,
                reported_root_ring_slot_still_bad_after_one_reread,
            } => {
                self.judge_reported_ceiling(answer, *reported_ceiling, &mut counts)?;
                let accepted = match reason {
                    ObservedRefusalReason::Explained(candidate) => {
                        Some(*candidate).filter(|candidate| {
                            answer.required_refusals.contains(candidate)
                                || (candidate.is_capacity_wall_with_an_interval()
                                    && self.capacity_wall_is_permitted(
                                        *candidate,
                                        answer,
                                        *publishes_completed,
                                    ))
                        })
                    }
                    ObservedRefusalReason::Unexplained => None,
                };
                let Some(accepted_reason) = accepted else {
                    let aspect = if answer.required_refusals.is_empty() {
                        ModelDisagreementAspect::RefusedWhenModelRequiresSuccess
                    } else {
                        ModelDisagreementAspect::RefusalReason
                    };
                    // 实现报的读坏的根环槽只在「拒之前一个写都没发、一次发布都没做」时带出去（mount.rs 那个成员的契约：
                    // 在回收、影子账与任何写之前返回）；拒之前写了盘的，故障注入不许拿这个槽去认。
                    let refused_before_any_write = !*wrote_anything && *publishes_completed == 0;
                    let mut disagreement = ModelDisagreement::new(
                        aspect,
                        describe_answer(answer),
                        format!("拒了：{member}"),
                    );
                    let root_ring_slot_reported_before_any_write =
                        reported_root_ring_slot_still_bad_after_one_reread
                            .filter(|_| refused_before_any_write);
                    disagreement.implementation_reported_unreadable_place =
                        root_ring_slot_reported_before_any_write
                            .map(ModelReportedUnreadablePlace::RootRingSlotStillBadAfterOneReread);
                    return Err(disagreement);
                };
                let partial_publishes_allowed = !answer.walls_are_judged_before_the_first_write
                    && accepted_reason.is_capacity_wall_with_an_interval();
                if *publishes_completed > 0 && !partial_publishes_allowed {
                    return Err(ModelDisagreement::new(
                        ModelDisagreementAspect::PublishCount,
                        format!("{}：拒之前一次发布都不做", accepted_reason.name()),
                        format!("拒之前做了 {publishes_completed} 次发布（{member}）"),
                    ));
                }
                if *wrote_anything && *publishes_completed == 0 {
                    return Err(ModelDisagreement::new(
                        ModelDisagreementAspect::WroteBeforeRefusing,
                        format!("{}：拒绝之前盘上一个字节都不动", accepted_reason.name()),
                        format!("拒了（{member}），录制流里多了写或屏障"),
                    ));
                }
                let Some(completed) = answer.expected_roots.get(..*publishes_completed) else {
                    return Err(ModelDisagreement::new(
                        ModelDisagreementAspect::PublishCount,
                        format!("至多 {} 次发布", answer.expected_roots.len()),
                        format!("拒之前做了 {publishes_completed} 次发布（{member}）"),
                    ));
                };
                if answer.required_refusals.contains(&accepted_reason) {
                    counts.required_refusals_matched += 1;
                } else {
                    counts.permitted_refusals_taken += 1;
                }
                if accepted_reason == ModelRefusalReason::UnitAreaWall {
                    counts.unit_area_wall_refusals_in_the_interval += 1;
                }
                if accepted_reason
                    == ModelRefusalReason::DataWriteToAnInodeBornAfterTheFirstFileWasDeleted
                {
                    counts.data_writes_refused_after_the_first_file_was_deleted_matched += 1;
                }
                for root in completed {
                    self.write_root(root.clone());
                }
                if let (Some(session), Some(last)) = (self.session.as_mut(), completed.last()) {
                    session.current = last.clone();
                }
                Ok(counts)
            }
            ObservedOutcome::Succeeded(effect) => {
                if !answer.required_refusals.is_empty() {
                    return Err(ModelDisagreement::new(
                        ModelDisagreementAspect::SucceededWhenModelRequiresRefusal,
                        describe_answer(answer),
                        "做成了".to_string(),
                    ));
                }
                self.judge_effect(answer, effect, &mut counts)?;
                counts.successes_matched += 1;
                if answer.operation == ModelOperationKind::PublishDeletionStep {
                    if answer
                        .expected_roots
                        .iter()
                        .any(|root| root.inode_being_deleted().is_some())
                    {
                        counts.deletion_batches_keeping_data_units_matched += 1;
                    } else if let (Some(after), Some(session)) =
                        (answer.expected_roots.first(), self.session.as_ref())
                    {
                        Self::count_the_shape_of_a_final_deletion_batch(
                            &session.current,
                            after,
                            &mut counts,
                        );
                    }
                }
                if answer.operation == ModelOperationKind::PublishSnapshotDestructionStep
                    && answer.expected_roots.iter().any(|root| {
                        root.snapshots
                            .iter()
                            .any(|snapshot| snapshot.is_being_destroyed)
                    })
                {
                    counts.snapshot_destruction_batches_keeping_the_snapshot_matched += 1;
                }
                match answer.operation {
                    ModelOperationKind::PublishCompactionBatch => {
                        counts.compaction_batches_matched += 1;
                        if answer
                            .expected_roots
                            .iter()
                            .any(|root| root.logged_compaction_intent.is_none())
                        {
                            counts.compaction_intents_deleted_with_the_last_batch_matched += 1;
                        }
                    }
                    ModelOperationKind::PublishCompactionIntentIssue => {
                        counts.compaction_intents_issued_matched += 1;
                    }
                    ModelOperationKind::PublishCompactionIntentCompletion => {
                        counts.compaction_intents_deleted_without_a_batch_matched += 1;
                    }
                    ModelOperationKind::PublishCompactionIntentDrop => {
                        counts.compaction_intents_dropped_matched += 1;
                    }
                    ModelOperationKind::PublishFirstFile
                    | ModelOperationKind::PublishOverwrite
                    | ModelOperationKind::PublishSequentialWrite
                    | ModelOperationKind::PublishWithoutUnits
                    | ModelOperationKind::MountWritable
                    | ModelOperationKind::RollbackWhileMounted
                    | ModelOperationKind::RaiseRollbackFloor
                    | ModelOperationKind::Unmount
                    | ModelOperationKind::ColdStartRecover
                    | ModelOperationKind::PublishUnlinkIntent
                    | ModelOperationKind::PublishDeletionStep
                    | ModelOperationKind::PublishNewInode
                    | ModelOperationKind::PublishSnapshotCreation
                    | ModelOperationKind::PublishSnapshotDestructionIntent
                    | ModelOperationKind::PublishSnapshotDestructionStep => {}
                    // 按 inode 号写第一个文件之外的文件：数这次长没长高、写的号落没落在上段第 0 片叶之外（这一组取样点要罩的两形）。
                    ModelOperationKind::PublishWholeWriteOfAnInodeOtherThanTheFirst => {
                        counts.writes_of_an_inode_other_than_the_first_matched += 1;
                        if let (Some(written_version), Some(session)) =
                            (answer.expected_roots.first(), self.session.as_ref())
                        {
                            let level_before = extent_upper_segment_root_level(
                                &session.current.inodes_with_an_extent_upper_leaf_entry(),
                            );
                            let level_after = extent_upper_segment_root_level(
                                &written_version.inodes_with_an_extent_upper_leaf_entry(),
                            );
                            if level_after > level_before {
                                counts.writes_growing_the_extent_upper_segment_matched += 1;
                            }
                            let written_inode = written_version
                                .inodes_other_than_the_first
                                .iter()
                                .find(|(_, inode)| {
                                    inode
                                        .data_unit_written_at
                                        .contains(&written_version.key.checkpoint_txg)
                                })
                                .map(|(number, _)| *number);
                            if written_inode
                                .is_some_and(|number| number >= EXTENT_TREE_UPPER_LEAF_INODES)
                            {
                                counts.writes_past_the_first_extent_upper_leaf_matched += 1;
                            }
                        }
                    }
                    // 按偏移写：第一个文件的单元这次变了的是写第一个文件，没变的是写别的文件。
                    ModelOperationKind::PublishPartialWrite => {
                        if let (Some(written_version), Some(session)) =
                            (answer.expected_roots.first(), self.session.as_ref())
                        {
                            if written_version.data_unit_written_at
                                == session.current.data_unit_written_at
                            {
                                counts.partial_writes_of_an_inode_other_than_the_first_matched += 1;
                            } else {
                                counts.partial_writes_of_the_first_file_matched += 1;
                            }
                        }
                    }
                    // 截断（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K2、K5）：缩短发的意图之后这一版在截断途中，
                    // 不缩短的那一次之后是带活文件的一版。
                    ModelOperationKind::PublishTruncation => {
                        if answer
                            .expected_roots
                            .iter()
                            .any(|root| root.logged_truncation_issued_txg().is_some())
                        {
                            counts.truncation_intents_matched += 1;
                        } else {
                            counts.size_changes_without_shrinking_matched += 1;
                        }
                    }
                    // 截断的一批之后仍在截断途中的是不是最后一批的那一批，之后是带活文件的一版的是最后一批。
                    ModelOperationKind::PublishTruncationStep => {
                        if answer
                            .expected_roots
                            .iter()
                            .any(|root| root.logged_truncation_issued_txg().is_some())
                        {
                            counts.truncation_batches_before_the_final_batch_matched += 1;
                        } else {
                            counts.final_truncation_batches_matched += 1;
                        }
                    }
                }
                if answer.expected_roots.iter().any(|root| {
                    root.user_visible_root_relocations
                        .values()
                        .any(|relocated_at| *relocated_at == root.key.checkpoint_txg)
                }) {
                    counts.compaction_publishes_relocating_a_user_visible_root_matched += 1;
                }
                self.count_the_changes_beside_files_other_than_the_first(answer, &mut counts);
                self.advance_by_success(answer);
                Ok(counts)
            }
        }
    }

    /// 一次比过、对得上的改动在第一个文件之外有文件带数据单元的一版上是哪几格（判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md`
    /// 第二节 S5 的验收：别的 inode 截断、变长、快照下截断、整理时截断），逐格计数：改 size 那一次与截断的各批改的是带数据单元的别的 inode（它的内容
    /// 或单元这次变了），其中这一版有快照的、logged_ops 里有活整理意图的另数；别的文件带数据单元时建快照、整理发布各数一次。
    fn count_the_changes_beside_files_other_than_the_first(
        &self,
        answer: &ModelAnswer,
        counts: &mut ModelJudgementCounts,
    ) {
        let (Some(session), Some(after)) = (self.session.as_ref(), answer.expected_roots.first())
        else {
            return;
        };
        let before = &session.current;
        let an_inode_holding_data_units_changed =
            before
                .inodes_other_than_the_first
                .iter()
                .any(|(number, inode_before)| {
                    after
                        .inodes_other_than_the_first
                        .get(number)
                        .is_some_and(|inode_after| {
                            (inode_before.data_units_present() > 0
                                || inode_after.data_units_present() > 0)
                                && (inode_after.content != inode_before.content
                                    || inode_after.data_unit_written_at
                                        != inode_before.data_unit_written_at)
                        })
                });
        match answer.operation {
            ModelOperationKind::PublishTruncation | ModelOperationKind::PublishTruncationStep
                if an_inode_holding_data_units_changed =>
            {
                if answer.operation == ModelOperationKind::PublishTruncation {
                    counts
                        .size_changes_of_inodes_other_than_the_first_holding_data_units_matched +=
                        1;
                } else {
                    counts
                        .truncation_steps_of_inodes_other_than_the_first_holding_data_units_matched +=
                        1;
                }
                if !before.snapshots.is_empty() {
                    counts.size_changes_of_inodes_other_than_the_first_under_a_snapshot_matched +=
                        1;
                }
                if before.logged_compaction_intent.is_some() {
                    counts
                        .size_changes_of_inodes_other_than_the_first_beside_a_live_compaction_intent_matched +=
                        1;
                }
            }
            ModelOperationKind::PublishSnapshotCreation
                if before.has_a_file_other_than_the_first_holding_data_units() =>
            {
                counts.snapshots_created_beside_files_other_than_the_first_holding_data_units_matched +=
                    1;
            }
            ModelOperationKind::PublishCompactionBatch
            | ModelOperationKind::PublishCompactionIntentIssue
                if before.has_a_file_other_than_the_first_holding_data_units() =>
            {
                counts
                    .compaction_publishes_beside_files_other_than_the_first_holding_data_units_matched +=
                    1;
            }
            ModelOperationKind::PublishFirstFile
            | ModelOperationKind::PublishOverwrite
            | ModelOperationKind::PublishSequentialWrite
            | ModelOperationKind::PublishWithoutUnits
            | ModelOperationKind::MountWritable
            | ModelOperationKind::RollbackWhileMounted
            | ModelOperationKind::RaiseRollbackFloor
            | ModelOperationKind::Unmount
            | ModelOperationKind::ColdStartRecover
            | ModelOperationKind::PublishUnlinkIntent
            | ModelOperationKind::PublishDeletionStep
            | ModelOperationKind::PublishNewInode
            | ModelOperationKind::PublishSnapshotCreation
            | ModelOperationKind::PublishSnapshotDestructionIntent
            | ModelOperationKind::PublishSnapshotDestructionStep
            | ModelOperationKind::PublishCompactionBatch
            | ModelOperationKind::PublishCompactionIntentIssue
            | ModelOperationKind::PublishCompactionIntentCompletion
            | ModelOperationKind::PublishCompactionIntentDrop
            | ModelOperationKind::PublishTruncation
            | ModelOperationKind::PublishTruncationStep
            | ModelOperationKind::PublishWholeWriteOfAnInodeOtherThanTheFirst
            | ModelOperationKind::PublishPartialWrite => {}
        }
    }

    /// 一次比过、对得上的删除最后一批（`before` 是它接在后面的那一版，`after` 是它写出的那一版）落在判决第三节 M4 那四形里的哪几形，逐形计数：
    /// 删的是第一个文件之外的号、删第一个文件而别的 inode 还在、删到文件集合为空、extent 上段的根降了层。
    fn count_the_shape_of_a_final_deletion_batch(
        before: &ModelRoot,
        after: &ModelRoot,
        counts: &mut ModelJudgementCounts,
    ) {
        let Some((deleted, _)) = before.inode_being_deleted() else {
            return;
        };
        match deleted {
            ModelDeletedInode::OtherThanTheFirst(_) => {
                counts.deletions_of_an_inode_other_than_the_first_matched += 1;
                if matches!(
                    before.contents,
                    ModelVersionContents::InodesWithoutTheFirstFile { .. }
                ) {
                    counts
                        .deletions_of_an_inode_other_than_the_first_after_the_first_file_was_deleted_matched +=
                        1;
                }
            }
            ModelDeletedInode::FirstFile => {
                if !after.inodes_other_than_the_first.is_empty() {
                    counts.deletions_of_the_first_file_beside_other_inodes_matched += 1;
                }
            }
        }
        if matches!(after.contents, ModelVersionContents::TreesAndNoFile { .. }) {
            counts.final_deletion_batches_emptying_the_version_matched += 1;
        }
        if extent_upper_segment_root_level(&after.inodes_with_an_extent_upper_leaf_entry())
            < extent_upper_segment_root_level(&before.inodes_with_an_extent_upper_leaf_entry())
        {
            counts.deletions_lowering_the_extent_upper_segment_matched += 1;
        }
    }

    fn judge_reported_ceiling(
        &self,
        answer: &ModelAnswer,
        reported_ceiling: Option<ModelCheckpointTxg>,
        counts: &mut ModelJudgementCounts,
    ) -> Result<(), ModelDisagreement> {
        let Some(reported) = reported_ceiling else {
            return Ok(());
        };
        counts.ceilings_compared += 1;
        if answer.rollback_floor_ceiling == Some(reported) {
            Ok(())
        } else {
            Err(ModelDisagreement {
                aspect: ModelDisagreementAspect::RollbackFloorCeiling,
                model_answer: format!(
                    "上限 {:?}（D16（发布语义） 已定项 1）",
                    answer.rollback_floor_ceiling.map(|ceiling| ceiling.0)
                ),
                implementation_answer: format!("实现报上限 {}", reported.0),
                implementation_reported_ceiling: Some(reported),
                implementation_reported_unreadable_place: None,
                implementation_rows_written: None,
            })
        }
    }

    /// 这一种操作做成时胶水交的是不是这一种结局。兼容表照各个 `answer_*` 答的种类与 `history.rs`、`model_comparison.rs` 的胶水现在交的那一种：
    /// 冷启动（`answer_cold_start_recover`）只收冷启动读回；可写挂载（`answer_mount_writable` 一族）只收挂载；
    /// 别的种类——各种发布、挂着时回退（`answer_rollback_while_mounted`）、抬 F（`answer_raise_rollback_floor`）、
    /// 正常卸载（`answer_unmount`，胶水 `model_comparison::observed_unmount` 交带新 F 的空发布）——只收发布。
    /// 新加一种操作不在这里两头都写上就编译不过（不写通配臂）。
    fn operation_hands_in_this_kind_of_effect(
        operation: ModelOperationKind,
        effect: &ObservedEffect,
    ) -> bool {
        match (operation, effect) {
            (ModelOperationKind::ColdStartRecover, ObservedEffect::ColdStart { .. })
            | (ModelOperationKind::MountWritable, ObservedEffect::Mount { .. }) => true,
            (
                ModelOperationKind::ColdStartRecover,
                ObservedEffect::Publishes { .. } | ObservedEffect::Mount { .. },
            )
            | (
                ModelOperationKind::MountWritable,
                ObservedEffect::Publishes { .. } | ObservedEffect::ColdStart { .. },
            ) => false,
            (
                ModelOperationKind::PublishFirstFile
                | ModelOperationKind::PublishOverwrite
                | ModelOperationKind::PublishSequentialWrite
                | ModelOperationKind::PublishWithoutUnits
                | ModelOperationKind::RollbackWhileMounted
                | ModelOperationKind::RaiseRollbackFloor
                | ModelOperationKind::Unmount
                | ModelOperationKind::PublishUnlinkIntent
                | ModelOperationKind::PublishDeletionStep
                | ModelOperationKind::PublishNewInode
                | ModelOperationKind::PublishSnapshotCreation
                | ModelOperationKind::PublishSnapshotDestructionIntent
                | ModelOperationKind::PublishSnapshotDestructionStep
                | ModelOperationKind::PublishCompactionBatch
                | ModelOperationKind::PublishCompactionIntentIssue
                | ModelOperationKind::PublishCompactionIntentCompletion
                | ModelOperationKind::PublishCompactionIntentDrop
                | ModelOperationKind::PublishTruncation
                | ModelOperationKind::PublishTruncationStep
                | ModelOperationKind::PublishWholeWriteOfAnInodeOtherThanTheFirst
                | ModelOperationKind::PublishPartialWrite,
                ObservedEffect::Publishes { .. },
            ) => true,
            (
                ModelOperationKind::PublishFirstFile
                | ModelOperationKind::PublishOverwrite
                | ModelOperationKind::PublishSequentialWrite
                | ModelOperationKind::PublishWithoutUnits
                | ModelOperationKind::RollbackWhileMounted
                | ModelOperationKind::RaiseRollbackFloor
                | ModelOperationKind::Unmount
                | ModelOperationKind::PublishUnlinkIntent
                | ModelOperationKind::PublishDeletionStep
                | ModelOperationKind::PublishNewInode
                | ModelOperationKind::PublishSnapshotCreation
                | ModelOperationKind::PublishSnapshotDestructionIntent
                | ModelOperationKind::PublishSnapshotDestructionStep
                | ModelOperationKind::PublishCompactionBatch
                | ModelOperationKind::PublishCompactionIntentIssue
                | ModelOperationKind::PublishCompactionIntentCompletion
                | ModelOperationKind::PublishCompactionIntentDrop
                | ModelOperationKind::PublishTruncation
                | ModelOperationKind::PublishTruncationStep
                | ModelOperationKind::PublishWholeWriteOfAnInodeOtherThanTheFirst
                | ModelOperationKind::PublishPartialWrite,
                ObservedEffect::Mount { .. } | ObservedEffect::ColdStart { .. },
            ) => false,
        }
    }

    fn judge_effect(
        &self,
        answer: &ModelAnswer,
        effect: &ObservedEffect,
        counts: &mut ModelJudgementCounts,
    ) -> Result<(), ModelDisagreement> {
        if !Self::operation_hands_in_this_kind_of_effect(answer.operation, effect) {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::EffectKindOfTheOperation,
                format!("{:?} 做成时胶水交的不是这一种结局", answer.operation),
                format!("胶水交来的是{}", describe_observed_effect_kind(effect)),
            ));
        }
        match effect {
            ObservedEffect::Publishes {
                roots,
                reported_ceiling,
            } => {
                self.judge_reported_ceiling(answer, *reported_ceiling, counts)?;
                if answer.operation == ModelOperationKind::RaiseRollbackFloor
                    && reported_ceiling.is_none()
                {
                    return Err(ModelDisagreement::new(
                        ModelDisagreementAspect::RollbackFloorCeiling,
                        "抬 F 做成要报上限".to_string(),
                        "胶水没交上限".to_string(),
                    ));
                }
                if let Some(target) = answer.rollback_target {
                    let floor = self.effective_rollback_floor();
                    if floor.0 > 0 && target.checkpoint_txg == floor {
                        counts.rollbacks_accepted_at_the_effective_floor += 1;
                    }
                }
                self.judge_roots(&answer.expected_roots, roots, counts)
            }
            ObservedEffect::Mount {
                instance,
                rows_written,
                roots,
            } => {
                let Some((expected_instance, expected_rows)) = &answer.expected_mount else {
                    return Err(ModelDisagreement::new(
                        ModelDisagreementAspect::NotModeled,
                        describe_answer(answer),
                        "实现交回的是挂载".to_string(),
                    ));
                };
                if instance != expected_instance {
                    return Err(ModelDisagreement::new(
                        ModelDisagreementAspect::InstanceGeneration,
                        format!("取号 {}", expected_instance.0),
                        format!("取号 {}", instance.0),
                    ));
                }
                if rows_written != expected_rows {
                    let mut disagreement = ModelDisagreement::new(
                        ModelDisagreementAspect::InstanceRows,
                        format!("写行 {expected_rows:?}"),
                        format!("写行 {rows_written:?}"),
                    );
                    disagreement.implementation_rows_written = Some(rows_written.clone());
                    return Err(disagreement);
                }
                self.judge_roots(&answer.expected_roots, roots, counts)
            }
            ObservedEffect::ColdStart { read_back } => {
                let Some(expected) = &answer.expected_read_back else {
                    return Err(ModelDisagreement::new(
                        ModelDisagreementAspect::NotModeled,
                        describe_answer(answer),
                        "实现交回的是冷启动读回".to_string(),
                    ));
                };
                let matches = match (expected, read_back) {
                    (
                        ModelReadBack::NoFile {
                            root: expected_root,
                        },
                        ObservedReadBack::NoFile { root },
                    ) => expected_root == root,
                    (
                        ModelReadBack::FileRead {
                            root: expected_root,
                            content: expected_content,
                        },
                        ObservedReadBack::FileRead { root, content },
                    ) => {
                        counts.cold_start_contents_compared += 1;
                        expected_root == root && expected_content.as_ref() == content.as_slice()
                    }
                    (
                        ModelReadBack::NoFile { .. } | ModelReadBack::FileRead { .. },
                        ObservedReadBack::NoFile { .. }
                        | ObservedReadBack::FileRead { .. }
                        | ObservedReadBack::Failed { .. },
                    ) => false,
                };
                if matches {
                    Ok(())
                } else {
                    Err(ModelDisagreement::new(
                        ModelDisagreementAspect::ColdStartReadBack,
                        describe_read_back(expected),
                        describe_observed_read_back(read_back),
                    ))
                }
            }
        }
    }

    fn judge_roots(
        &self,
        expected_roots: &[ModelRoot],
        observed_roots: &[ObservedRoot],
        counts: &mut ModelJudgementCounts,
    ) -> Result<(), ModelDisagreement> {
        if expected_roots.len() != observed_roots.len() {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::PublishCount,
                format!(
                    "写出 {} 条根：{:?}",
                    expected_roots.len(),
                    expected_roots
                        .iter()
                        .map(|root| root.key)
                        .collect::<Vec<_>>()
                ),
                format!(
                    "写出 {} 条根：{:?}",
                    observed_roots.len(),
                    observed_roots
                        .iter()
                        .map(|root| root.key)
                        .collect::<Vec<_>>()
                ),
            ));
        }
        for (expected, observed) in expected_roots.iter().zip(observed_roots) {
            counts.roots_compared += 1;
            if expected.key != observed.key {
                return Err(ModelDisagreement::new(
                    ModelDisagreementAspect::RootIdentity,
                    format!("{:?}", expected.key),
                    format!("{:?}", observed.key),
                ));
            }
            if expected.journal_counter != observed.journal_counter {
                return Err(ModelDisagreement::new(
                    ModelDisagreementAspect::JournalCounter,
                    format!("{:?} 的 jsn {}", expected.key, expected.journal_counter.0),
                    format!("jsn {}", observed.journal_counter.0),
                ));
            }
            if expected.rollback_floor != observed.rollback_floor {
                return Err(ModelDisagreement::new(
                    ModelDisagreementAspect::RollbackFloor,
                    format!("{:?} 带 F {}", expected.key, expected.rollback_floor.0),
                    format!("带 F {}", observed.rollback_floor.0),
                ));
            }
            if expected.contents.live_file().is_some() != observed.file.has_file() {
                return Err(ModelDisagreement::new(
                    ModelDisagreementAspect::FilePresence,
                    format!("{:?} 里有什么：{:?}", expected.key, expected.contents),
                    format!("有文件：{}", observed.file.has_file()),
                ));
            }
            Self::judge_file_content(expected, &observed.file, counts)?;
            Self::judge_instance_table(expected, &observed.instance_table, counts)?;
            self.judge_allocation_generations(expected, observed, counts)?;
            Self::judge_snapshot_state(expected, &observed.snapshot_state, counts)?;
            Self::judge_logged_compaction_intent(expected, observed, counts)?;
            Self::judge_first_file_record_truncation_field(expected, observed, counts)?;
            Self::judge_inodes_other_than_the_first(expected, observed, counts)?;
            Self::judge_truncation_fields_of_the_inodes_other_than_the_first(
                expected, observed, counts,
            )?;
        }
        Ok(())
    }

    /// 这一版第一个文件之外每条记录里的截断字段与模型记的相同（判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S5「截断字段
    /// 按 inode 比」，m4-trunc-r2 第三节 R1、R2 推到任意 inode）：号的集合一样（活着的与删除中的记录都在树里），每条的 T、N_w 两段一样。
    fn judge_truncation_fields_of_the_inodes_other_than_the_first(
        expected: &ModelRoot,
        observed: &ObservedRoot,
        counts: &mut ModelJudgementCounts,
    ) -> Result<(), ModelDisagreement> {
        let expected_fields: BTreeMap<u64, ObservedTruncationField> = expected
            .inodes_other_than_the_first
            .iter()
            .map(|(number, inode)| (*number, inode.truncation_field.recorded_values()))
            .collect();
        if expected_fields != observed.truncation_fields_of_the_inodes_other_than_the_first {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::InodesOtherThanTheFirst,
                format!(
                    "{:?} 第一个文件之外每条记录的截断字段 {expected_fields:?}",
                    expected.key
                ),
                format!(
                    "{:?}",
                    observed.truncation_fields_of_the_inodes_other_than_the_first
                ),
            ));
        }
        for inode in expected.inodes_other_than_the_first.values() {
            counts.truncation_fields_of_the_inodes_other_than_the_first_compared += 1;
            match inode.truncation_field {
                ModelTruncationField::WrittenByATruncationIntent { .. } => {
                    counts.written_truncation_fields_of_the_inodes_other_than_the_first_compared +=
                        1;
                }
                ModelTruncationField::NotWritten => {}
            }
        }
        Ok(())
    }

    /// 这一版第一个文件之外的 inode 与模型记的相同（里程碑「删除与多次 COW」一·1）：号的集合一样，每个读回的内容逐字节一样
    /// （没写过数据的读回空）。写错了 inode（写成别的号、写成第一个文件）这里与第一个文件的内容那一格必有一格对不上。
    fn judge_inodes_other_than_the_first(
        expected: &ModelRoot,
        observed: &ObservedRoot,
        counts: &mut ModelJudgementCounts,
    ) -> Result<(), ModelDisagreement> {
        // 删除中的那一个（nlink = 0）读不出，实现那一侧也不交（`model_comparison::observed_inodes_other_than_the_first_of_file_version`），
        // 它还剩的单元按分配代另比。
        let expected_numbers: Vec<u64> = expected
            .inodes_other_than_the_first
            .iter()
            .filter(|(_, inode)| inode.is_live())
            .map(|(number, _)| *number)
            .collect();
        let observed_numbers: Vec<u64> = observed
            .inodes_other_than_the_first
            .keys()
            .copied()
            .collect();
        if expected_numbers != observed_numbers {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::InodesOtherThanTheFirst,
                format!(
                    "{:?} 第一个文件之外的 inode {expected_numbers:?}（水位 {}）",
                    expected.key, expected.inode_number_watermark
                ),
                format!("{observed_numbers:?}"),
            ));
        }
        for (number, inode) in expected
            .inodes_other_than_the_first
            .iter()
            .filter(|(_, inode)| inode.is_live())
        {
            let observed_content = observed
                .inodes_other_than_the_first
                .get(number)
                .expect("上面比过两边的号一样");
            match observed_content {
                ObservedFile::Decoded(content) if content.as_slice() == inode.content.as_ref() => {}
                ObservedFile::Decoded(_)
                | ObservedFile::NoFile
                | ObservedFile::Undecodable { .. } => {
                    let described = match observed_content {
                        ObservedFile::Decoded(content) => {
                            format!("{} 字节（末字节 {:?}）", content.len(), content.last())
                        }
                        ObservedFile::NoFile => "读不出这个文件".to_string(),
                        ObservedFile::Undecodable { what } => format!("解不出内容：{what}"),
                    };
                    return Err(ModelDisagreement::new(
                        ModelDisagreementAspect::InodesOtherThanTheFirst,
                        format!(
                            "{:?} 的 inode {number}（各单元的分配代 {:?}）读回 {} 字节（末字节 {:?}）",
                            expected.key,
                            inode.data_unit_written_at,
                            inode.content.len(),
                            inode.content.last()
                        ),
                        described,
                    ));
                }
            }
            counts.inodes_other_than_the_first_compared += 1;
            if !inode.data_unit_written_at.is_empty() {
                counts.contents_of_files_other_than_the_first_compared += 1;
            }
        }
        Ok(())
    }

    /// 这一版第一个文件那条 inode 记录里的截断字段与模型记的相同（判决 `research/prompts/m4-trunc-r2-main-verification.md` 第三节 R1、R2、R5，
    /// 第六节用户定案）：有没有那条记录、T、N_w 两段的值。
    fn judge_first_file_record_truncation_field(
        expected: &ModelRoot,
        observed: &ObservedRoot,
        counts: &mut ModelJudgementCounts,
    ) -> Result<(), ModelDisagreement> {
        let expected_values = expected
            .contents
            .first_file_record_truncation_field()
            .map(ModelTruncationField::recorded_values);
        if expected_values != observed.first_file_record_truncation_field {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::FirstFileRecordTruncationField,
                format!(
                    "{:?} 第一个文件那条记录的截断字段 {:?}（{:?}）",
                    expected.key,
                    expected_values,
                    expected.contents.first_file_record_truncation_field()
                ),
                format!("{:?}", observed.first_file_record_truncation_field),
            ));
        }
        match expected.contents.first_file_record_truncation_field() {
            Some(ModelTruncationField::WrittenByATruncationIntent { .. }) => {
                counts.truncation_fields_compared += 1;
                counts.written_truncation_fields_compared += 1;
            }
            Some(ModelTruncationField::NotWritten) => counts.truncation_fields_compared += 1,
            None => {}
        }
        Ok(())
    }

    /// 这一版 logged_ops 里的整理意图与模型记的相同：有没有、签发 txg、R、`c0`（判决 G1 的字段表；`segs0` 不比）。
    fn judge_logged_compaction_intent(
        expected: &ModelRoot,
        observed: &ObservedRoot,
        counts: &mut ModelJudgementCounts,
    ) -> Result<(), ModelDisagreement> {
        if expected.logged_compaction_intent != observed.logged_compaction_intent {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::LoggedCompactionIntent,
                format!(
                    "{:?} 的 logged_ops 整理意图 {:?}",
                    expected.key, expected.logged_compaction_intent
                ),
                format!("{:?}", observed.logged_compaction_intent),
            ));
        }
        if expected.logged_compaction_intent.is_some() {
            counts.logged_compaction_intents_compared += 1;
        }
        Ok(())
    }

    /// 这一版的快照那几样与模型记的逐项相同（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S7）：活头前驱（树表 0 条的一版
    /// 没有那条条目）、快照列表（个数、每个的诞生 txg、前驱、销毁中记号、两条根指针抄的是谁）、deadlist 条目数、被钉映射条目数，各是一格。
    fn judge_snapshot_state(
        expected: &ModelRoot,
        observed: &ObservedSnapshotState,
        counts: &mut ModelJudgementCounts,
    ) -> Result<(), ModelDisagreement> {
        let expected_live_head_previous_snapshot_txg = expected
            .contents
            .has_trees()
            .then_some(expected.previous_snapshot_txg_of_the_live_head);
        if expected_live_head_previous_snapshot_txg != observed.live_head_previous_snapshot_txg {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::LiveHeadPreviousSnapshotTxg,
                format!(
                    "{:?} 的活头 previous_snapshot_txg {:?}",
                    expected.key,
                    expected_live_head_previous_snapshot_txg.map(|txg| txg.0)
                ),
                format!(
                    "{:?}",
                    observed.live_head_previous_snapshot_txg.map(|txg| txg.0)
                ),
            ));
        }
        let expected_snapshots: Vec<ObservedSnapshot> = expected
            .snapshots
            .iter()
            .map(|snapshot| ObservedSnapshot {
                birth_txg: snapshot.birth_txg,
                previous_snapshot_txg: snapshot.previous_snapshot_txg,
                is_being_destroyed: snapshot.is_being_destroyed,
                inode_tree_written_by: snapshot.inode_tree_written_by,
                extent_root_written_at: snapshot.extent_root_written_at,
            })
            .collect();
        if expected_snapshots != observed.snapshots {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::Snapshots,
                format!("{:?} 的快照 {expected_snapshots:?}", expected.key),
                format!("{:?}", observed.snapshots),
            ));
        }
        let mut expected_deadlist_entries = expected.deadlist.as_ref().clone();
        expected_deadlist_entries.sort_unstable();
        let mut observed_deadlist_entries = observed.deadlist_entries.clone();
        observed_deadlist_entries.sort_unstable();
        if expected_deadlist_entries != observed_deadlist_entries {
            let first_difference = expected_deadlist_entries
                .iter()
                .zip(&observed_deadlist_entries)
                .position(|(expected_entry, observed_entry)| expected_entry != observed_entry)
                .unwrap_or_else(|| {
                    expected_deadlist_entries
                        .len()
                        .min(observed_deadlist_entries.len())
                });
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::DeadlistEntries,
                format!(
                    "{:?} 的 deadlist {} 条，按 key 排第 {first_difference} 条起：{:?}",
                    expected.key,
                    expected_deadlist_entries.len(),
                    &expected_deadlist_entries[first_difference..]
                ),
                format!(
                    "{} 条，第 {first_difference} 条起：{:?}",
                    observed_deadlist_entries.len(),
                    &observed_deadlist_entries[first_difference..]
                ),
            ));
        }
        if expected.pinned_mapping_entries != observed.pinned_mapping_entries {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::PinnedMappingEntries,
                format!(
                    "{:?} 被钉住的映射条目 {} 条",
                    expected.key, expected.pinned_mapping_entries
                ),
                format!("{} 条", observed.pinned_mapping_entries),
            ));
        }
        counts.snapshot_states_compared += 1;
        counts.snapshots_compared += u64::try_from(expected_snapshots.len()).expect("个数");
        if !expected_deadlist_entries.is_empty() {
            counts.versions_with_deadlist_entries_compared += 1;
        }
        Ok(())
    }

    /// 这一版的文件内容与模型记的逐字节相同（有没有文件已经比过）。
    fn judge_file_content(
        expected: &ModelRoot,
        observed: &ObservedFile,
        counts: &mut ModelJudgementCounts,
    ) -> Result<(), ModelDisagreement> {
        match (expected.contents.live_file(), observed) {
            (None, ObservedFile::NoFile) => Ok(()),
            (Some(file), ObservedFile::Decoded(content)) => {
                counts.file_contents_compared += 1;
                if file.content.as_ref() == content.as_slice() {
                    Ok(())
                } else {
                    Err(ModelDisagreement::new(
                        ModelDisagreementAspect::FileContent,
                        format!(
                            "{:?} 的文件是 {:?} 写的 {} 字节（末字节 {:?}）",
                            expected.key,
                            file.written_by,
                            file.content.len(),
                            file.content.last()
                        ),
                        format!("{} 字节（末字节 {:?}）", content.len(), content.last()),
                    ))
                }
            }
            (Some(file), ObservedFile::Undecodable { what }) => Err(ModelDisagreement::new(
                ModelDisagreementAspect::FileContent,
                format!(
                    "{:?} 的文件是 {:?} 写的 {} 字节",
                    expected.key,
                    file.written_by,
                    file.content.len()
                ),
                format!("交回的单元里解不出内容：{what}"),
            )),
            (None, ObservedFile::Decoded(_) | ObservedFile::Undecodable { .. })
            | (Some(_), ObservedFile::NoFile) => Err(ModelDisagreement::new(
                ModelDisagreementAspect::FilePresence,
                format!("{:?} 里有什么：{:?}", expected.key, expected.contents),
                format!("有文件：{}", observed.has_file()),
            )),
        }
    }

    /// 这一版的整张实例表与模型记的逐行相同；实现交回的东西里没有这一版整条链的字节时只计数——只许出现在这次没重写实例表的那几版
    /// （mkfs 之后第一次重写之前、从盘上重建且多于一片、零单元发布）。这次重写了实例表链的那一版（写行：带文件的与树表 0 条的）
    /// 实现写出了整条链，比不了就是对不上（代码审阅第 12 条：每一版比实例表）。
    fn judge_instance_table(
        expected: &ModelRoot,
        observed: &ObservedInstanceTable,
        counts: &mut ModelJudgementCounts,
    ) -> Result<(), ModelDisagreement> {
        match observed {
            ObservedInstanceTable::Rows(rows) => {
                counts.instance_tables_compared += 1;
                if expected.instance_table_rows.as_slice() == rows.as_slice() {
                    Ok(())
                } else {
                    Err(ModelDisagreement::new(
                        ModelDisagreementAspect::InstanceTableOfTheVersion,
                        format!(
                            "{:?} 的实例表 {:?}",
                            expected.key,
                            expected.instance_table_rows.as_slice()
                        ),
                        format!("{rows:?}"),
                    ))
                }
            }
            ObservedInstanceTable::NotInTheOutput { why } => {
                let rewritten_by_this_publish =
                    expected.role_written_at.get(&ModelUnitRole::InstanceTable)
                        == Some(&expected.key.checkpoint_txg);
                if rewritten_by_this_publish {
                    return Err(ModelDisagreement::new(
                        ModelDisagreementAspect::InstanceTableOfTheVersion,
                        format!(
                            "{:?} 这次重写了实例表链（{:?}）：实现要交回它的字节",
                            expected.key,
                            expected.instance_table_rows.as_slice()
                        ),
                        format!("比不了：{why}"),
                    ));
                }
                counts.instance_tables_not_in_the_output += 1;
                Ok(())
            }
            ObservedInstanceTable::Undecodable { what } => Err(ModelDisagreement::new(
                ModelDisagreementAspect::InstanceTableOfTheVersion,
                format!(
                    "{:?} 的实例表 {:?}",
                    expected.key,
                    expected.instance_table_rows.as_slice()
                ),
                format!("交回的实例表单元解不开：{what}"),
            )),
        }
    }

    /// 这一版每个单元的分配记录：每块盘各一条、仍分配着、分配代等于写它的那次发布的 txg（D3（空间分配） 已定项 3 / 7）。
    /// 两个方向都比（代码审阅第 12 条）：实现交回的每个角色这一版都要有，模型这一版的每个角色实现都要交回。
    /// 零单元发布（树表 0 条的一版上）输出不带分配记录，比的是这次发布重写了哪几个角色：模型那一侧是分配代等于这次 txg 的那几个，
    /// 零单元发布一个都没有；模型说这次重写了角色的那一版（写行、带文件的各种发布）只交回重写的角色，就是对不上。
    fn judge_allocation_generations(
        &self,
        expected: &ModelRoot,
        observed: &ObservedRoot,
        counts: &mut ModelJudgementCounts,
    ) -> Result<(), ModelDisagreement> {
        let observed_records = match &observed.unit_allocation_records {
            ObservedUnitAllocationRecords::EveryRoleOfTheVersion(records) => records,
            ObservedUnitAllocationRecords::RewrittenRolesOnly(rewritten) => {
                let rewritten_in_the_model: BTreeSet<ModelUnitRole> = expected
                    .role_written_at
                    .iter()
                    .filter(|(_, written_at)| **written_at == expected.key.checkpoint_txg)
                    .map(|(role, _)| *role)
                    .chain(
                        expected
                            .data_unit_written_at
                            .contains(&expected.key.checkpoint_txg)
                            .then_some(ModelUnitRole::Data),
                    )
                    .collect();
                counts.rewritten_role_sets_compared += 1;
                if !rewritten_in_the_model.is_empty() {
                    return Err(ModelDisagreement::new(
                        ModelDisagreementAspect::AllocationGeneration,
                        format!(
                            "{:?} 这次重写了 {rewritten_in_the_model:?}：实现要交回这一版每个角色的分配记录",
                            expected.key
                        ),
                        format!("只交回了重写的角色 {rewritten:?}"),
                    ));
                }
                if rewritten_in_the_model == *rewritten {
                    return Ok(());
                }
                return Err(ModelDisagreement::new(
                    ModelDisagreementAspect::AllocationGeneration,
                    format!(
                        "{:?} 这次重写（分配代等于这次 txg）的角色 {rewritten_in_the_model:?}",
                        expected.key
                    ),
                    format!("重写了 {rewritten:?}"),
                ));
            }
            ObservedUnitAllocationRecords::AllocationRecordTreeRootChangedByAZeroUnitPublish {
                what,
            } => {
                return Err(ModelDisagreement::new(
                    ModelDisagreementAspect::AllocationGeneration,
                    format!(
                        "{:?} 的分配记录就是同一次挂载里前一版那一份（零单元发布照抄分配记录树根指针）",
                        expected.key
                    ),
                    what.clone(),
                ));
            }
        };
        let ObservedAllocationRecordsOfEveryUnit {
            roles: observed_records,
            data_units: observed_data_units,
            data_units_of_the_files_other_than_the_first: observed_data_units_of_the_other_files,
        } = observed_records;
        let handed_in: BTreeSet<ModelUnitRole> =
            observed_records.iter().map(|(role, _)| *role).collect();
        if let Some(role) = expected
            .role_written_at
            .keys()
            .find(|role| !handed_in.contains(role))
        {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::AllocationGeneration,
                format!(
                    "{:?} 这一版有 {role:?}（{:?} 写的），实现要交回它的分配记录",
                    expected.key,
                    expected.role_written_at.get(role)
                ),
                format!("只交回了 {handed_in:?}"),
            ));
        }
        for (role, records) in observed_records {
            let written_at = match role {
                // 数据单元按单元号交（`data_units`），混在角色里交的说不上是哪一个。
                ModelUnitRole::Data => {
                    return Err(ModelDisagreement::new(
                        ModelDisagreementAspect::AllocationGeneration,
                        format!(
                            "{:?} 的数据单元按单元号逐个交（{} 个）",
                            expected.key,
                            expected.data_unit_written_at.len()
                        ),
                        format!("在角色里交了一个不带单元号的数据单元：{records:?}"),
                    ));
                }
                ModelUnitRole::ExtentRoot
                | ModelUnitRole::ExtentUpperNodeBelowTheRoot(_)
                | ModelUnitRole::InodeLeaf
                | ModelUnitRole::InodeLeafRightOfTheLeftmost(_)
                | ModelUnitRole::InodeRoot
                | ModelUnitRole::AllocationTree
                | ModelUnitRole::AccountingTree
                | ModelUnitRole::MappingTree
                | ModelUnitRole::TreeTable
                | ModelUnitRole::InstanceTable
                | ModelUnitRole::DeletedInodesRoot
                | ModelUnitRole::LoggedOperationsRoot
                | ModelUnitRole::TombstoneLeaf
                | ModelUnitRole::TombstoneRoot
                | ModelUnitRole::RetirementLeaf
                | ModelUnitRole::RetirementRoot => {
                    let Some(written_at) = expected.role_written_at.get(role) else {
                        return Err(ModelDisagreement::new(
                            ModelDisagreementAspect::AllocationGeneration,
                            format!("{:?} 这一版没有 {role:?} 这个角色", expected.key),
                            format!("实现交回了 {role:?} 的单元"),
                        ));
                    };
                    *written_at
                }
            };
            counts.allocation_records_compared += u64::try_from(records.len()).expect("条数");
            if !self.allocation_records_are_one_per_device_allocated_at(records, written_at) {
                return Err(ModelDisagreement::new(
                    ModelDisagreementAspect::AllocationGeneration,
                    format!(
                        "{:?} 的 {role:?}：每块盘一条、仍分配、分配代 {}（写它的那次发布）",
                        expected.key, written_at.0
                    ),
                    format!("{records:?}"),
                ));
            }
        }
        // 数据单元按单元号逐个比（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K4：截断只重写尾单元之后各单元的分配代不同）：
        // 交回的单元号恰好是 0 .. 模型这一版的单元数、各一项，单元 i 的分配代等于模型按单元号记的第 i 个。
        let mut data_units_handed_in: Vec<u64> = observed_data_units
            .iter()
            .map(|(index, _)| *index)
            .collect();
        data_units_handed_in.sort_unstable();
        let data_units_in_the_model: Vec<u64> =
            (0..u64::try_from(expected.data_unit_written_at.len()).expect("单元数")).collect();
        if data_units_handed_in != data_units_in_the_model {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::AllocationGeneration,
                format!(
                    "{:?} 这一版有 {} 个数据单元（单元号 0 起连着、各一项），实现要交回它们的分配记录",
                    expected.key,
                    expected.data_unit_written_at.len()
                ),
                format!("交回的单元号（排过序）{data_units_handed_in:?}"),
            ));
        }
        for (index, records) in observed_data_units {
            let written_at = *expected
                .data_unit_written_at
                .get(usize::try_from(*index).expect("单元号装得进 usize"))
                .expect("上面判过交回的单元号恰好是 0 .. 模型这一版的单元数");
            counts.allocation_records_compared += u64::try_from(records.len()).expect("条数");
            let data_unit_records_match =
                self.allocation_records_are_one_per_device_allocated_at(records, written_at);
            if !data_unit_records_match {
                return Err(ModelDisagreement::new(
                    ModelDisagreementAspect::AllocationGeneration,
                    format!(
                        "{:?} 的数据单元 {index}：每块盘一条、仍分配、分配代 {}（写它的那次发布）",
                        expected.key, written_at.0
                    ),
                    format!("{records:?}"),
                ));
            }
        }
        // 第一个文件之外的文件的数据单元按 (inode 号, 单元号) 逐个比（里程碑「删除与多次 COW」一·1）：交回的恰好是模型这一版每个有数据单元的文件的
        // 0 .. 单元数、各一项，单元 i 的分配代等于模型按单元号记的第 i 个（整份写那次全是它，改 size 重写的尾单元是那一次的，判决 m4-multi-inode-size-r1
        // 第二节 S3）。
        let mut other_data_units_handed_in: Vec<(u64, u64)> =
            observed_data_units_of_the_other_files
                .iter()
                .map(|(inode_and_index, _)| *inode_and_index)
                .collect();
        other_data_units_handed_in.sort_unstable();
        let other_data_units_in_the_model: Vec<(u64, u64)> = expected
            .inodes_other_than_the_first
            .iter()
            .flat_map(|(number, inode)| {
                (0..inode.data_units_present()).map(move |index| (*number, index))
            })
            .collect();
        if other_data_units_handed_in != other_data_units_in_the_model {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::AllocationGeneration,
                format!(
                    "{:?} 第一个文件之外的文件的数据单元 (inode 号, 单元号) {other_data_units_in_the_model:?}，实现要交回它们的分配记录",
                    expected.key
                ),
                format!("交回的（排过序）{other_data_units_handed_in:?}"),
            ));
        }
        for ((number, index), records) in observed_data_units_of_the_other_files {
            let written_at = expected
                .inodes_other_than_the_first
                .get(number)
                .and_then(|inode| {
                    inode
                        .data_unit_written_at
                        .get(usize::try_from(*index).expect("这个文件的单元号装得进 usize"))
                        .copied()
                })
                .expect(
                    "上面判过交回的 (inode 号, 单元号) 恰好是模型这一版有数据单元的文件的那几个",
                );
            counts.allocation_records_compared += u64::try_from(records.len()).expect("条数");
            if !self.allocation_records_are_one_per_device_allocated_at(records, written_at) {
                return Err(ModelDisagreement::new(
                    ModelDisagreementAspect::AllocationGeneration,
                    format!(
                        "{:?} 的 inode {number} 数据单元 {index}：每块盘一条、仍分配、分配代 {}（整份写它的那次发布）",
                        expected.key, written_at.0
                    ),
                    format!("{records:?}"),
                ));
            }
        }
        Ok(())
    }

    /// 一个单元的那几条分配记录：每块盘恰好一条、仍分配着、分配代等于写它的那次发布的 txg（D3（空间分配） 已定项 3 / 7）。
    fn allocation_records_are_one_per_device_allocated_at(
        &self,
        records: &[ObservedAllocationRecord],
        written_at: ModelCheckpointTxg,
    ) -> bool {
        let devices_expected: BTreeSet<ModelDeviceIdentity> =
            self.geometry.devices.iter().copied().collect();
        let devices_seen: BTreeSet<ModelDeviceIdentity> =
            records.iter().map(|record| record.device).collect();
        records.len() == self.geometry.devices.len()
            && devices_seen == devices_expected
            && records
                .iter()
                .all(|record| !record.is_released && record.generation == written_at)
    }

    fn write_root(&mut self, root: ModelRoot) {
        self.roots_written_since_make_filesystem += 1;
        self.highest_published_txg = self.highest_published_txg.max(root.key.checkpoint_txg);
        self.highest_journal_counter = self.highest_journal_counter.max(root.journal_counter);
        self.ring.insert(
            ring_position_of(root.key.checkpoint_txg, self.slots_per_region),
            root,
        );
    }

    fn advance_by_success(&mut self, answer: &ModelAnswer) {
        for root in &answer.expected_roots {
            self.write_root(root.clone());
        }
        match answer.session_after_success {
            ModelSessionAfterSuccess::StaysOpen => {
                if let (Some(session), Some(last)) =
                    (self.session.as_mut(), answer.expected_roots.last())
                {
                    session.current = last.clone();
                }
            }
            ModelSessionAfterSuccess::OpenedAs(instance) => {
                self.highest_acquired_instance = instance;
                let current = answer
                    .expected_roots
                    .last()
                    .expect("挂载至少写出写行那一次")
                    .clone();
                self.session = Some(ModelSession { instance, current });
            }
            ModelSessionAfterSuccess::Closed => self.session = None,
        }
    }
}

/// 崩溃之后恢复到的这一版允不允许（增补 3 第 3 件：崩溃注入）。模型这一侧给的是「提交过哪些版、每一版的内容是什么」
/// （[`IdealModel::committed_versions`] 攒出来的目录），崩溃点那一侧给的是「盘上已持久的最新根槽是哪条根」
/// （从截断了的录制流里数出来，不经模型也不经实现的择根）。四条判据：
/// ① 走读不许失败；
/// ② 择到的根必须是模型提交过的某一版——读回一版从没提交过的，是 D13（验证路线） 已定项 7 那条「记录流里没有的东西不许出现在盘上」
///    在恢复这一侧的形态；
/// ③ 盘上已持久的最新根槽是 (T, i) 时，择到的根按 (txg, 实例) 不许比它旧（D22（单元原子性怎么合成） 已定项 7 的择新序：
///    择新 txg 为主、平局按实例代号高者赢）；
/// ④ 读回的内容要与模型记的那一版逐字节相同，那一版树表 0 条时只许报没有文件。
///
/// 与崩溃点重放的 `crash::oracle_violation_for_versions` 分开写：那一份的版本表由调用方按固定脚本手写，这一份问的是模型
/// （D13（验证路线） 已定项 5：模型不与 `singlefs-core` 共用代码）。那一份在「走到一条没发布过的更新的根上报没有文件」那一格只在
/// 有更旧的带文件版本时才判红，这一份认得每一条根有没有文件，两个方向都判得出。
#[must_use]
pub fn crash_recovery_disagreement(
    committed_versions: &BTreeMap<ModelRootKey, Option<Rc<[u8]>>>,
    newest_persisted_root: Option<ModelRootKey>,
    read_back: &ObservedReadBack,
) -> Option<ModelDisagreement> {
    let (chosen, content_read_back) = match read_back {
        ObservedReadBack::Failed { what } => {
            return Some(ModelDisagreement::new(
                ModelDisagreementAspect::ColdStartReadBack,
                "崩溃之后的冷启动要走到一条模型提交过的根上，不许失败".to_string(),
                format!("走读失败：{what}"),
            ))
        }
        ObservedReadBack::NoFile { root } => (*root, None),
        ObservedReadBack::FileRead { root, content } => (*root, Some(content)),
    };
    if let Some(persisted) = newest_persisted_root {
        if chosen < persisted {
            return Some(ModelDisagreement::new(
                ModelDisagreementAspect::ColdStartReadBack,
                format!(
                    "盘上已持久的最新根槽是实例 {} 第 {} 代，不许恢复到比它旧的根",
                    persisted.instance.0, persisted.checkpoint_txg.0
                ),
                format!(
                    "走到实例 {} 第 {} 代根",
                    chosen.instance.0, chosen.checkpoint_txg.0
                ),
            ));
        }
    }
    let Some(committed) = committed_versions.get(&chosen) else {
        return Some(ModelDisagreement::new(
            ModelDisagreementAspect::ColdStartReadBack,
            "择到的根要是模型提交过的某一版".to_string(),
            format!(
                "走到实例 {} 第 {} 代根，模型从没提交过这一版（{}）",
                chosen.instance.0,
                chosen.checkpoint_txg.0,
                describe_observed_read_back(read_back)
            ),
        ));
    };
    match (content_read_back, committed) {
        (None, None) => None,
        (None, Some(content)) => Some(ModelDisagreement::new(
            ModelDisagreementAspect::ColdStartReadBack,
            format!(
                "实例 {} 第 {} 代根下面有文件（{} 字节）",
                chosen.instance.0,
                chosen.checkpoint_txg.0,
                content.len()
            ),
            "报了没有文件".to_string(),
        )),
        (Some(content), None) => Some(ModelDisagreement::new(
            ModelDisagreementAspect::ColdStartReadBack,
            format!(
                "实例 {} 第 {} 代根树表 0 条，没有文件",
                chosen.instance.0, chosen.checkpoint_txg.0
            ),
            format!("读回了 {} 字节", content.len()),
        )),
        (Some(content), Some(committed_content)) => (content.as_slice() != &committed_content[..])
            .then(|| {
                ModelDisagreement::new(
                    ModelDisagreementAspect::ColdStartReadBack,
                    format!(
                        "实例 {} 第 {} 代根下面是 {} 字节（末字节 {:?}）",
                        chosen.instance.0,
                        chosen.checkpoint_txg.0,
                        committed_content.len(),
                        committed_content.last()
                    ),
                    format!("读回 {} 字节（末字节 {:?}）", content.len(), content.last()),
                )
            }),
    }
}

/// 一版里第一个文件之外每个活着的 inode 读得出的内容：号 → 内容（[`IdealModel::committed_versions_of_the_inodes_other_than_the_first`]）。
pub type ModelContentsOfTheInodesOtherThanTheFirst = BTreeMap<u64, Rc<[u8]>>;

/// 崩溃之后恢复落到 `chosen` 那一版时，第一个文件之外每个活着的 inode 读出的（`read_back`，胶水按号交）在不在允许集合里（判决
/// `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S5「崩溃读回按 inode 给允许集合」，同第一个文件那一份
/// [`crash_recovery_disagreement`]）：`chosen` 要是模型提交过的某一版，那一版里活着的号与读出的号一样，每个读出的逐字节等于
/// [`IdealModel::committed_versions_of_the_inodes_other_than_the_first`] 记的那一份。都在交 `None`。
#[must_use]
pub fn crash_recovery_disagreement_of_the_inodes_other_than_the_first(
    committed_versions: &BTreeMap<ModelRootKey, ModelContentsOfTheInodesOtherThanTheFirst>,
    chosen: ModelRootKey,
    read_back: &BTreeMap<u64, ObservedFile>,
) -> Option<ModelDisagreement> {
    let Some(committed) = committed_versions.get(&chosen) else {
        return Some(ModelDisagreement::new(
            ModelDisagreementAspect::ColdStartReadBack,
            "择到的根要是模型提交过的某一版".to_string(),
            format!(
                "走到实例 {} 第 {} 代根，模型从没提交过这一版",
                chosen.instance.0, chosen.checkpoint_txg.0
            ),
        ));
    };
    let committed_numbers: Vec<u64> = committed.keys().copied().collect();
    let read_numbers: Vec<u64> = read_back.keys().copied().collect();
    if committed_numbers != read_numbers {
        return Some(ModelDisagreement::new(
            ModelDisagreementAspect::ColdStartReadBack,
            format!(
                "实例 {} 第 {} 代根下面第一个文件之外活着的 inode {committed_numbers:?}",
                chosen.instance.0, chosen.checkpoint_txg.0
            ),
            format!("读出的号 {read_numbers:?}"),
        ));
    }
    committed.iter().find_map(|(number, content)| {
        let read = read_back.get(number).expect("上面比过两边的号一样");
        match read {
            ObservedFile::Decoded(bytes) if bytes.as_slice() == content.as_ref() => None,
            ObservedFile::Decoded(_) | ObservedFile::NoFile | ObservedFile::Undecodable { .. } => {
                Some(ModelDisagreement::new(
                    ModelDisagreementAspect::ColdStartReadBack,
                    format!(
                        "实例 {} 第 {} 代根下面 inode {number} 是 {} 字节（末字节 {:?}）",
                        chosen.instance.0,
                        chosen.checkpoint_txg.0,
                        content.len(),
                        content.last()
                    ),
                    match read {
                        ObservedFile::Decoded(bytes) => {
                            format!("读出 {} 字节（末字节 {:?}）", bytes.len(), bytes.last())
                        }
                        ObservedFile::NoFile => "读不出这个文件".to_string(),
                        ObservedFile::Undecodable { what } => format!("解不出内容：{what}"),
                    },
                ))
            }
        }
    })
}

fn describe_answer(answer: &ModelAnswer) -> String {
    if answer.required_refusals.is_empty() {
        format!(
            "{:?} 该成（写出 {:?}；允许拒的只有容量墙区间）",
            answer.operation,
            answer
                .expected_roots
                .iter()
                .map(|root| (root.key.checkpoint_txg.0, root.key.instance.0))
                .collect::<Vec<_>>()
        )
    } else {
        let required: Vec<&'static str> = answer
            .required_refusals
            .iter()
            .map(|reason| reason.name())
            .collect();
        format!("{:?} 该拒：{required:?}", answer.operation)
    }
}

fn describe_snapshot_read_back(read_back: &ModelSnapshotReadBack) -> String {
    match read_back {
        ModelSnapshotReadBack::NoFile => "没有文件".to_string(),
        ModelSnapshotReadBack::File(content) => {
            format!("{} 字节（末字节 {:?}）", content.len(), content.last())
        }
    }
}

fn describe_observed_snapshot_read_back(read_back: &ObservedSnapshotReadBack) -> String {
    match read_back {
        ObservedSnapshotReadBack::NoFile => "读出没有文件".to_string(),
        ObservedSnapshotReadBack::File(content) => {
            format!("读出 {} 字节（末字节 {:?}）", content.len(), content.last())
        }
        ObservedSnapshotReadBack::NoSnapshotWithThisTree => "树表里没有这个快照".to_string(),
        ObservedSnapshotReadBack::Failed { what } => format!("读失败：{what}"),
    }
}

fn describe_read_back(read_back: &ModelReadBack) -> String {
    match read_back {
        ModelReadBack::NoFile { root } => format!("所选根 {root:?}，没有文件"),
        ModelReadBack::FileRead { root, content } => format!(
            "所选根 {root:?}，读回 {} 字节（末字节 {:?}）",
            content.len(),
            content.last()
        ),
    }
}

fn describe_observed_read_back(read_back: &ObservedReadBack) -> String {
    match read_back {
        ObservedReadBack::NoFile { root } => format!("所选根 {root:?}，没有文件"),
        ObservedReadBack::FileRead { root, content } => format!(
            "所选根 {root:?}，读回 {} 字节（末字节 {:?}）",
            content.len(),
            content.last()
        ),
        ObservedReadBack::Failed { what } => format!("读回失败：{what}"),
    }
}

/// 交来的做成结局是哪一种（判兼容那一格的报告用）。
fn describe_observed_effect_kind(effect: &ObservedEffect) -> &'static str {
    match effect {
        ObservedEffect::Publishes { .. } => "发布（Publishes）",
        ObservedEffect::Mount { .. } => "挂载（Mount）",
        ObservedEffect::ColdStart { .. } => "冷启动读回（ColdStart）",
    }
}

/// 攒批的会话的语义模型（D16（发布语义） 已定项 2、10；判决 `research/prompts/m4-publish-trigger-r1-main-verification.md` 第二节「返回即落盘」「RefFS」
/// 两格：随机历史与模型对拍「返回即落盘」的假设改成按会话报告的发布点判）：收下的改动先进待发、不改「盘上读得到」的那一份，只有会话报告一次发布时
/// 待发才按收下的次序施加到那一份上（同一个文件的几次整份写，后到的盖掉先到的；建 inode 按收下的次序从水位起发号）。
///
/// 触发照条款另写一份：待发非空 ∧（now − 上次发布的读数 ≥ `T_time` ∨ 待发 journal 字节 ≥ 有效 `T_dirty`），读数倒退当作没过时间、上次发布的读数不回拨，
/// 第一次见到的读数当基线。待发要写多少 journal 字节模型算不出（要走实现的固定点），取实现报的每一次改动单独发布要写的字节，只核它的下界：
/// 每一次改动至少一条记录（元数据改动也计入，D16 已定项 5，用户 2026-10-26 定），批内同一个文件写 K 次照 K 次加、不折叠。
///
/// 只 `use singlefs_format`（同本文件开头那一条，D13（验证路线） 已定项 5）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelOfThePendingChanges {
    time_threshold_in_milliseconds: u64,
    dirty_threshold_in_journal_bytes: u64,
    reading_of_the_last_publish_in_milliseconds: Option<u64>,
    published: ModelFilesReadableAfterThePublishes,
    pending: Vec<ModelPendingChange>,
    journal_bytes_of_the_pending_changes: u64,
}

/// 最近一次发布之后盘上读得到的文件：每个 inode 号的整份内容（建出来还没写过的是空的）与下一个要发的 inode 号。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelFilesReadableAfterThePublishes {
    pub contents_by_inode: BTreeMap<u64, Vec<u8>>,
    pub inode_number_watermark: u64,
}

/// 收进待发的一次改动。封闭集合。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModelPendingChange {
    WholeWrite { inode: u64, content: Vec<u8> },
    NewInodes { count: u64 },
}

/// 模型判的这一次触发是哪一支先到。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelPublishTriggerCause {
    TimeThreshold,
    DirtyThreshold,
    BothThresholds,
}

/// 实现报的一次改动单独发布要写的 journal 字节低于一条记录：元数据改动也要写记录（D16（发布语义） 已定项 5「元数据改动也计入」），一次发布至少一条。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModelJournalBytesOfOneChangeBelowOneRecord {
    pub reported: u64,
    pub at_least: u64,
}

impl ModelOfThePendingChanges {
    /// 一次发布之后起步：盘上读得到 `published`，待发空，还没见过读数。两个阈值是这次挂载的有效值（时间毫秒、脏量 journal 字节）。
    #[must_use]
    pub fn after_a_publish(
        published: ModelFilesReadableAfterThePublishes,
        time_threshold_in_milliseconds: u64,
        dirty_threshold_in_journal_bytes: u64,
    ) -> Self {
        Self {
            time_threshold_in_milliseconds,
            dirty_threshold_in_journal_bytes,
            reading_of_the_last_publish_in_milliseconds: None,
            published,
            pending: Vec::new(),
            journal_bytes_of_the_pending_changes: 0,
        }
    }

    /// 最近一次发布之后盘上读得到的文件。
    #[must_use]
    pub fn published(&self) -> &ModelFilesReadableAfterThePublishes {
        &self.published
    }

    #[must_use]
    pub fn pending_is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    /// 待发里那几次改动要写的 journal 字节之和（实现报的逐次相加）。
    #[must_use]
    pub fn journal_bytes_of_the_pending_changes(&self) -> u64 {
        self.journal_bytes_of_the_pending_changes
    }

    /// 一次调用交进来的读数：还没见过读数时它是基线。
    pub fn note_a_reading(&mut self, now_in_milliseconds: u64) {
        if self.reading_of_the_last_publish_in_milliseconds.is_none() {
            self.reading_of_the_last_publish_in_milliseconds = Some(now_in_milliseconds);
        }
    }

    /// 收下一次改动，脏量加上实现报的这一次单独发布要写的 journal 字节。
    ///
    /// # Errors
    /// 报的字节低于一条记录（[`ModelJournalBytesOfOneChangeBelowOneRecord`]）：模型不动。
    pub fn accept(
        &mut self,
        change: ModelPendingChange,
        journal_bytes_of_this_change_reported: u64,
    ) -> Result<(), ModelJournalBytesOfOneChangeBelowOneRecord> {
        if journal_bytes_of_this_change_reported < JOURNAL_RECORD_BYTES {
            return Err(ModelJournalBytesOfOneChangeBelowOneRecord {
                reported: journal_bytes_of_this_change_reported,
                at_least: JOURNAL_RECORD_BYTES,
            });
        }
        self.pending.push(change);
        self.journal_bytes_of_the_pending_changes = self
            .journal_bytes_of_the_pending_changes
            .saturating_add(journal_bytes_of_this_change_reported);
        Ok(())
    }

    /// 此刻（`now_in_milliseconds`）该不该触发、哪一支先到。待发空时不触发（空闲时不推空发布）。
    #[must_use]
    pub fn expected_trigger(&self, now_in_milliseconds: u64) -> Option<ModelPublishTriggerCause> {
        if self.pending.is_empty() {
            return None;
        }
        let milliseconds_since_the_last_publish = self
            .reading_of_the_last_publish_in_milliseconds
            .map_or(0, |last| now_in_milliseconds.saturating_sub(last));
        let is_time_reached =
            milliseconds_since_the_last_publish >= self.time_threshold_in_milliseconds;
        let is_dirty_reached =
            self.journal_bytes_of_the_pending_changes >= self.dirty_threshold_in_journal_bytes;
        match (is_time_reached, is_dirty_reached) {
            (true, true) => Some(ModelPublishTriggerCause::BothThresholds),
            (true, false) => Some(ModelPublishTriggerCause::TimeThreshold),
            (false, true) => Some(ModelPublishTriggerCause::DirtyThreshold),
            (false, false) => None,
        }
    }

    /// 会话报了一次排空待发的发布：待发按收下的次序施加到盘上读得到的那一份上，清空；上次发布的读数取这一次的（不回拨）。
    pub fn publish_the_pending_changes(&mut self, now_in_milliseconds: u64) {
        for change in std::mem::take(&mut self.pending) {
            match change {
                ModelPendingChange::WholeWrite { inode, content } => {
                    self.published.contents_by_inode.insert(inode, content);
                }
                ModelPendingChange::NewInodes { count } => {
                    for _created in 0..count {
                        self.published
                            .contents_by_inode
                            .insert(self.published.inode_number_watermark, Vec::new());
                        self.published.inode_number_watermark += 1;
                    }
                }
            }
        }
        self.journal_bytes_of_the_pending_changes = 0;
        self.note_a_publish(now_in_milliseconds);
    }

    /// 会话报了一次不排空待发的发布（待发本来就空时的单发）：只记读数。
    pub fn note_a_publish(&mut self, now_in_milliseconds: u64) {
        self.reading_of_the_last_publish_in_milliseconds = Some(
            self.reading_of_the_last_publish_in_milliseconds
                .map_or(now_in_milliseconds, |last| last.max(now_in_milliseconds)),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 模型跟着建池时的 S 落槽（m3-close-c506 三轮判决第三节「harness 模型跟着 S 走」）：txg 12 在 S = 4 的模型里落区域 0 槽 0、
    /// 盖掉第 0 代根；在 S = 8 的模型里落区域 0 槽 4，第 0 代根还在环里。
    #[test]
    fn the_model_places_roots_by_the_slots_per_region_it_was_built_with() {
        let geometry = ModelPoolGeometry {
            devices: vec![ModelDeviceIdentity(0), ModelDeviceIdentity(1)],
            device_size_in_bytes: 4 << 30,
        };
        let txgs_in_the_ring_after_writing_txg_twelve = |slots_per_region: u64| {
            let mut model =
                IdealModel::after_make_filesystem_on_a_journal_ring_with_slots_per_region(
                    geometry.clone(),
                    JOURNAL_RING_DEFAULT_BYTES,
                    slots_per_region,
                );
            let mut root_of_txg_twelve = model.newest_root().clone();
            root_of_txg_twelve.key.checkpoint_txg = ModelCheckpointTxg(12);
            model.write_root(root_of_txg_twelve);
            model
                .committed_versions()
                .iter()
                .map(|(key, _)| key.checkpoint_txg.0)
                .collect::<Vec<u64>>()
        };
        assert_eq!(
            txgs_in_the_ring_after_writing_txg_twelve(4),
            vec![12],
            "S = 4：环长 12，txg 12 绕回区域 0 槽 0，盖掉第 0 代根"
        );
        assert_eq!(
            txgs_in_the_ring_after_writing_txg_twelve(8),
            vec![0, 12],
            "S = 8：环长 24，txg 12 落区域 0 槽 4，第 0 代根还在"
        );
    }

    fn two_device_model() -> IdealModel {
        IdealModel::after_make_filesystem(ModelPoolGeometry {
            devices: vec![ModelDeviceIdentity(0), ModelDeviceIdentity(1)],
            device_size_in_bytes: 4 << 30,
        })
    }

    fn succeed(model: &mut IdealModel, answer: &ModelAnswer) {
        assert!(
            answer.required_refusals.is_empty(),
            "这一步模型要求拒：{:?}",
            answer.required_refusals
        );
        model.advance_by_success(answer);
    }

    fn key(checkpoint_txg: u64, instance: u64) -> ModelRootKey {
        ModelRootKey {
            checkpoint_txg: ModelCheckpointTxg(checkpoint_txg),
            instance: ModelInstanceGeneration(instance),
        }
    }

    /// mkfs、同一进程取号暖机、第一个文件（txg 3）、可写挂载（实例 2：写行 txg 4、暖机 txg 5）、覆盖写 txg 6–9，内容是 txg 号那一个字节。
    fn model_after_four_overwrites_in_a_second_instance() -> IdealModel {
        let mut model = two_device_model();
        model.acquire_and_warm_up_in_the_make_filesystem_process();
        let first = model
            .answer_publish_first_file(&[3])
            .expect("会话开着、现行 txg 2");
        succeed(&mut model, &first);
        model.close_session();
        let mount = model.answer_mount_writable();
        succeed(&mut model, &mount);
        for content_byte in 6_u8..=9 {
            let overwrite = model
                .answer_publish_overwrite(&[content_byte])
                .expect("会话开着、现行版本带文件");
            succeed(&mut model, &overwrite);
        }
        model
    }

    /// 实例表多于一片（用户 2026-09-24 定尾片先、一片写满 369 行再开下一片）：可写挂载不再拒，写行那次发布整条链重写，
    /// 链上每一片都是一个重写的单元、各占两个槽。第一个文件之后号推到 370（等于连着 369 次取号之后崩溃），
    /// 下一次可写挂载取 371、写 [1, 371) 共 370 行 ⇒ 两片：写行那次发布占 2 × 2 + 3（记账树、映射树、树表）槽，
    /// 加分配记录树这次至多重写的节点数。
    #[test]
    fn mount_that_writes_more_rows_than_one_page_holds_counts_every_page_of_the_instance_table_chain(
    ) {
        let mut model = two_device_model();
        model.acquire_and_warm_up_in_the_make_filesystem_process();
        let first = model
            .answer_publish_first_file(&[3])
            .expect("会话开着、现行 txg 2");
        succeed(&mut model, &first);
        model.close_session();
        model.highest_acquired_instance = ModelInstanceGeneration(370);
        let mount = model.answer_mount_writable();
        assert!(mount.required_refusals.is_empty(), "{mount:?}");
        let (instance, rows_written) = mount.expected_mount.clone().expect("挂载做成");
        assert_eq!(
            (instance, rows_written.len()),
            (ModelInstanceGeneration(371), 370)
        );
        let row_publish = &mount.expected_roots[0];
        assert_eq!(row_publish.instance_table_rows.len(), 370);
        let slots_before = model.newest_root().occupied_slots_upper_bound_per_device;
        let slots_outside_the_allocation_record_tree = 2 * 2 + 3;
        assert_eq!(
            row_publish.occupied_slots_upper_bound_per_device - slots_before,
            slots_outside_the_allocation_record_tree
                + model.allocation_record_tree_nodes_rewritten_upper_bound(
                    slots_before + slots_outside_the_allocation_record_tree
                ),
            "两片实例表各 2 槽、三个固定点单元各 1 槽，加分配记录树这次至多重写的节点"
        );
    }

    /// 分配记录树一次发布至多重写几个节点（D8（核心索引结构） 已定项 14：叶罩 812 槽、内部扇出 169）：4 GiB 两块盘，根在第 2 层；
    /// 第一个文件版本之前每盘写过 3 槽（mkfs 的实例表 2、树表 1），这次树之外写 9 槽（数据 2、extent 根 1、inode 叶 2、inode 根 1、
    /// 记账树、映射树、树表各 1）。设这次重写 A 个节点：落点都在单元区起点 50176 往后 64 × (12 + A) 槽里。
    /// A = 9 时那一段到 51520，罩叶 61..=63 三片、第 1 层 1 个 ⇒ 1 + 2 × 4 = 9，是不动点；从单元区全部 529 个节点往下迭代停在这里。
    #[test]
    fn allocation_record_tree_nodes_rewritten_by_the_first_file_version_on_4_gib_are_at_most_nine()
    {
        let model = two_device_model();
        assert_eq!(
            model.allocation_record_tree_nodes_rewritten_upper_bound(u64::MAX / 128),
            1 + 2 * (262 + 2),
            "单元区里的全部节点：每盘叶 61..=322、第 1 层 0..=1，加根"
        );
        assert_eq!(
            model.allocation_record_tree_nodes_rewritten_upper_bound(3 + 9),
            9
        );
        let mut model = model;
        model.acquire_and_warm_up_in_the_make_filesystem_process();
        let slots_before = model.newest_root().occupied_slots_upper_bound_per_device;
        assert_eq!(slots_before, 3, "暖机是零单元发布，一个槽都不占");
        let first = model
            .answer_publish_first_file(&[3])
            .expect("会话开着、现行 txg 2");
        assert_eq!(
            first.expected_roots[0].occupied_slots_upper_bound_per_device,
            3 + 9 + 9
        );
    }

    /// 内容装不装得下按 32768 含头与预留位算（D4（校验和位置） 已定项 5）：正好装满不拒，多一个字节才拒。
    #[test]
    fn content_of_exactly_the_payload_capacity_is_accepted_and_one_byte_more_is_refused() {
        assert_eq!(data_unit_payload_capacity_in_bytes(), 32768 - 105 - 29);
        let model = model_after_four_overwrites_in_a_second_instance();
        let capacity = usize::try_from(data_unit_payload_capacity_in_bytes()).expect("三万多");
        let fits = model
            .answer_publish_overwrite(&vec![7; capacity])
            .expect("会话开着");
        assert!(fits.required_refusals.is_empty(), "{fits:?}");
        let exceeds = model
            .answer_publish_overwrite(&vec![7; capacity + 1])
            .expect("会话开着");
        assert_eq!(
            exceeds.required_refusals,
            BTreeSet::from([ModelRefusalReason::ContentExceedsDataUnitPayload])
        );
    }

    /// 抬 F 的上限（D16（发布语义） 已定项 1）：有效根 txg 0–9，非空的是 3（有了文件）与 6–9（覆盖写），4、5（写行、暖机）照抄 3 的文件、
    /// 0–2 没有文件，都不算。第 4 新的非空根是 6；每块盘上最新的有效根：盘 0 是 9（区域 0），盘 1 是 7（区域 1）⇒ 上限 min(7, 6) = 6。
    /// 抬到 6 不拒，抬到 7 必须拒。
    #[test]
    fn the_ceiling_is_the_fourth_newest_non_empty_root_capped_by_the_newest_root_on_every_device() {
        let model = model_after_four_overwrites_in_a_second_instance();
        assert_eq!(model.rollback_floor_ceiling(), Some(ModelCheckpointTxg(6)));
        let at_the_ceiling = model
            .answer_raise_rollback_floor(ModelCheckpointTxg(6))
            .expect("会话开着");
        assert!(
            at_the_ceiling.required_refusals.is_empty(),
            "{at_the_ceiling:?}"
        );
        assert_eq!(
            at_the_ceiling
                .expected_roots
                .iter()
                .map(|root| root.key)
                .collect::<Vec<_>>(),
            vec![key(10, 2), key(11, 2)],
            "txg 10 落盘 1、txg 11 落盘 0：两块盘都有带新 F 的根就停"
        );
        let above = model
            .answer_raise_rollback_floor(ModelCheckpointTxg(7))
            .expect("会话开着");
        assert_eq!(
            above.required_refusals,
            BTreeSet::from([ModelRefusalReason::FloorAboveCeiling])
        );
    }

    /// 抬 F 到 6 之后 F_生效 = 6：回退到 txg 6 的根是候选（txg ≥ F_生效，D16（发布语义） 已定项 1「回退候选集」），txg 5 的必须拒。
    /// 回退是挂着时的一次向前发布（D23（journal 的角色与格式） 已定项 14）：写一条根 (12, 2)，实例、F 照现行那一版，文件是 txg 6 写的那一版、
    /// 数据单元的分配代写回 6；实例 2 的 7–11 不被抛弃，之后再回退到 (9, 2) 照样是候选。冷启动读回最新那条根——回退写的那一版。
    #[test]
    fn the_forward_rollback_to_the_effective_floor_publishes_one_root_and_leaves_the_newer_roots_candidates(
    ) {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        let raise = model
            .answer_raise_rollback_floor(ModelCheckpointTxg(6))
            .expect("会话开着");
        succeed(&mut model, &raise);
        assert_eq!(model.effective_rollback_floor(), ModelCheckpointTxg(6));
        assert_eq!(
            model
                .answer_rollback_while_mounted(key(5, 2))
                .expect("会话开着、现行版本带文件")
                .required_refusals,
            BTreeSet::from([ModelRefusalReason::RollbackTargetBelowEffectiveFloor])
        );
        assert_eq!(
            model
                .answer_rollback_while_mounted(key(2, 1))
                .expect("会话开着、现行版本带文件")
                .required_refusals,
            BTreeSet::from([
                ModelRefusalReason::RollbackTargetBelowEffectiveFloor,
                ModelRefusalReason::RollbackTargetWithoutFile
            ]),
            "暖机根 (2, 1) 在 F 之下、也没有文件"
        );
        let rollback = model
            .answer_rollback_while_mounted(key(6, 2))
            .expect("会话开着、现行版本带文件");
        assert_eq!(
            rollback
                .expected_roots
                .iter()
                .map(|root| root.key)
                .collect::<Vec<_>>(),
            vec![key(12, 2)],
            "一次发布，接着现行那一版（txg 11）、实例照旧"
        );
        let rollback_root = &rollback.expected_roots[0];
        assert_eq!(rollback_root.rollback_floor, ModelCheckpointTxg(6));
        assert_eq!(
            rollback_root
                .contents
                .live_file()
                .map(|file| file.written_by),
            Some(key(6, 2))
        );
        assert_eq!(
            rollback_root.data_unit_written_at.as_slice(),
            [ModelCheckpointTxg(6)],
            "复活的数据单元分配代写回目标那一版账里的"
        );
        assert_eq!(
            rollback_root.role_written_at.get(&ModelUnitRole::TreeTable),
            Some(&ModelCheckpointTxg(12)),
            "固定点单元这次重写"
        );
        succeed(&mut model, &rollback);
        assert!(
            model
                .answer_rollback_while_mounted(key(9, 2))
                .expect("会话开着、现行版本带文件")
                .required_refusals
                .is_empty(),
            "向前发布不抛弃 (9, 2)"
        );
        model.close_session();
        let cold_start = model.answer_cold_start_recover();
        let Some(ModelReadBack::FileRead { root, content }) = cold_start.expected_read_back else {
            panic!("回退那一版带文件：{cold_start:?}");
        };
        assert_eq!(root, key(12, 2));
        assert_eq!(content.as_ref(), &[6]);
    }

    /// 「4 个不同状态」去重（D16（发布语义） 已定项 1「「非空」从盘上怎么认」末段）：实例 2 覆盖写到 txg 9（各一个状态）之后回退到 (7, 2)，
    /// 回退写的 (10, 2) 状态同 txg 7。从新到旧计入 10、9、8，7 与 10 同一个状态不计，第 4 个是 6；每块盘上最新的有效根取小是 9
    /// （盘 0 最新 9、盘 1 最新 10）⇒ 上限 6。不去重时第 4 个是 7。
    #[test]
    fn after_a_forward_rollback_the_ceiling_counts_the_repeated_state_once() {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        let rollback = model
            .answer_rollback_while_mounted(key(7, 2))
            .expect("会话开着、现行版本带文件");
        succeed(&mut model, &rollback);
        assert_eq!(model.rollback_floor_ceiling(), Some(ModelCheckpointTxg(6)));
    }

    /// F_生效 = 各幸存盘最新持久有效根所带 F 的最大值（D16（发布语义） 已定项 1「生效」）：抬 F 半路停在第一次（txg 10 落盘 1）时，
    /// 盘 1 上最新的有效根就带着 6，F_生效 已是 6。
    #[test]
    fn raised_floor_carried_by_the_newest_root_of_one_device_takes_effect() {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        let raise = model
            .answer_raise_rollback_floor(ModelCheckpointTxg(6))
            .expect("会话开着");
        let first_publish_only = raise.expected_roots[..1].to_vec();
        for root in first_publish_only {
            model.write_root(root);
        }
        assert_eq!(model.effective_rollback_floor(), ModelCheckpointTxg(6));
    }

    /// 单元的分配代是写它的那次发布的 txg（D3（空间分配） 已定项 3 / 7）：写行那次（txg 4）照抄的数据单元分配代 3，写成 4 就对不上；
    /// 这次重写的实例表分配代 4。
    #[test]
    fn carried_unit_keeps_the_generation_of_the_publish_that_wrote_it() {
        let mut model = two_device_model();
        model.acquire_and_warm_up_in_the_make_filesystem_process();
        let first = model.answer_publish_first_file(&[3]).expect("会话开着");
        succeed(&mut model, &first);
        model.close_session();
        let mount = model.answer_mount_writable();
        let row_publish = &mount.expected_roots[0];
        let records = |generation: u64| {
            [ModelDeviceIdentity(0), ModelDeviceIdentity(1)]
                .into_iter()
                .map(|device| ObservedAllocationRecord {
                    device,
                    generation: ModelCheckpointTxg(generation),
                    is_released: false,
                })
                .collect::<Vec<_>>()
        };
        let observed = |data_generation: u64| ObservedRoot {
            key: row_publish.key,
            journal_counter: row_publish.journal_counter,
            rollback_floor: row_publish.rollback_floor,
            file: ObservedFile::Decoded(vec![3]),
            instance_table: ObservedInstanceTable::Rows(
                row_publish.instance_table_rows.as_ref().clone(),
            ),
            unit_allocation_records: ObservedUnitAllocationRecords::EveryRoleOfTheVersion(
                ObservedAllocationRecordsOfEveryUnit {
                    roles: row_publish
                        .role_written_at
                        .iter()
                        .map(|(role, written_at)| (*role, records(written_at.0)))
                        .collect(),
                    data_units: (0_u64..)
                        .zip(row_publish.data_unit_written_at.iter())
                        .map(|(index, _)| (index, records(data_generation)))
                        .collect(),
                    data_units_of_the_files_other_than_the_first: Vec::new(),
                },
            ),
            snapshot_state: snapshot_state_as_the_model_wrote(row_publish),
            logged_compaction_intent: row_publish.logged_compaction_intent,
            first_file_record_truncation_field: truncation_field_as_the_model_wrote(row_publish),
            inodes_other_than_the_first: BTreeMap::new(),
            truncation_fields_of_the_inodes_other_than_the_first: BTreeMap::new(),
        };
        let mut counts = ModelJudgementCounts::default();
        assert_eq!(
            model.judge_allocation_generations(row_publish, &observed(3), &mut counts),
            Ok(())
        );
        assert_eq!(
            counts.allocation_records_compared,
            2 * 9,
            "写行那一版九个角色，每个角色每块盘一条"
        );
        let wrong = model
            .judge_allocation_generations(row_publish, &observed(4), &mut counts)
            .expect_err("照抄的数据单元分配代写成了这次的 txg");
        assert_eq!(wrong.aspect, ModelDisagreementAspect::AllocationGeneration);
    }

    /// 实现对这一版的交回原样照模型那一版写：文件内容、整张实例表、每个角色每块盘一条仍分配、代是写它的那次发布。
    fn observed_exactly_as_the_model_wrote(root: &ModelRoot) -> ObservedRoot {
        let records = |generation: ModelCheckpointTxg| {
            [ModelDeviceIdentity(0), ModelDeviceIdentity(1)]
                .into_iter()
                .map(|device| ObservedAllocationRecord {
                    device,
                    generation,
                    is_released: false,
                })
                .collect::<Vec<_>>()
        };
        ObservedRoot {
            key: root.key,
            journal_counter: root.journal_counter,
            rollback_floor: root.rollback_floor,
            file: match root.contents.live_file() {
                Some(file) => ObservedFile::Decoded(file.content.to_vec()),
                None => ObservedFile::NoFile,
            },
            instance_table: ObservedInstanceTable::Rows(root.instance_table_rows.as_ref().clone()),
            unit_allocation_records: ObservedUnitAllocationRecords::EveryRoleOfTheVersion(
                ObservedAllocationRecordsOfEveryUnit {
                    roles: root
                        .role_written_at
                        .iter()
                        .map(|(role, written_at)| (*role, records(*written_at)))
                        .collect(),
                    data_units: (0_u64..)
                        .zip(root.data_unit_written_at.iter())
                        .map(|(index, written_at)| (index, records(*written_at)))
                        .collect(),
                    data_units_of_the_files_other_than_the_first: root
                        .inodes_other_than_the_first
                        .iter()
                        .flat_map(|(number, inode)| {
                            (0_u64..).zip(inode.data_unit_written_at.iter()).map(
                                move |(index, written_at)| ((*number, index), records(*written_at)),
                            )
                        })
                        .collect(),
                },
            ),
            snapshot_state: snapshot_state_as_the_model_wrote(root),
            logged_compaction_intent: root.logged_compaction_intent,
            first_file_record_truncation_field: truncation_field_as_the_model_wrote(root),
            inodes_other_than_the_first: root
                .inodes_other_than_the_first
                .iter()
                .map(|(number, inode)| (*number, ObservedFile::Decoded(inode.content.to_vec())))
                .collect(),
            truncation_fields_of_the_inodes_other_than_the_first: root
                .inodes_other_than_the_first
                .iter()
                .map(|(number, inode)| (*number, inode.truncation_field.recorded_values()))
                .collect(),
        }
    }

    /// 第一个文件那条记录里的截断字段照模型记的原样交回（没有那条记录的一版交 None）。
    fn truncation_field_as_the_model_wrote(root: &ModelRoot) -> Option<ObservedTruncationField> {
        root.contents
            .first_file_record_truncation_field()
            .map(ModelTruncationField::recorded_values)
    }

    /// 这一版的快照那几样照模型记的原样交回（树表 0 条的一版没有活头那条条目）。
    fn snapshot_state_as_the_model_wrote(root: &ModelRoot) -> ObservedSnapshotState {
        ObservedSnapshotState {
            live_head_previous_snapshot_txg: root
                .contents
                .has_trees()
                .then_some(root.previous_snapshot_txg_of_the_live_head),
            snapshots: root
                .snapshots
                .iter()
                .map(|snapshot| ObservedSnapshot {
                    birth_txg: snapshot.birth_txg,
                    previous_snapshot_txg: snapshot.previous_snapshot_txg,
                    is_being_destroyed: snapshot.is_being_destroyed,
                    inode_tree_written_by: snapshot.inode_tree_written_by,
                    extent_root_written_at: snapshot.extent_root_written_at,
                })
                .collect(),
            deadlist_entries: root.deadlist.as_ref().clone(),
            pinned_mapping_entries: root.pinned_mapping_entries,
        }
    }

    /// 可写挂载写行那一版（txg 4）：第一个文件（txg 3，内容 [3]）之后关掉、再挂一次。
    fn row_publish_of_a_mount_after_the_first_file() -> (IdealModel, ModelRoot) {
        let mut model = two_device_model();
        model.acquire_and_warm_up_in_the_make_filesystem_process();
        let first = model.answer_publish_first_file(&[3]).expect("会话开着");
        succeed(&mut model, &first);
        model.close_session();
        let mount = model.answer_mount_writable();
        let row_publish = mount.expected_roots[0].clone();
        (model, row_publish)
    }

    /// 分配代对拍两个方向都比（代码审阅第 12 条）：模型这一版有、实现没交回的角色也报——此前只遍历实现交回的角色，
    /// 写行那一版九个角色只交两个也判对得上。
    #[test]
    fn a_role_of_the_version_that_the_implementation_did_not_hand_in_is_reported() {
        let (model, row_publish) = row_publish_of_a_mount_after_the_first_file();
        let mut observed = observed_exactly_as_the_model_wrote(&row_publish);
        let mut counts = ModelJudgementCounts::default();
        assert_eq!(
            model.judge_roots(
                std::slice::from_ref(&row_publish),
                std::slice::from_ref(&observed),
                &mut counts
            ),
            Ok(()),
            "原样交回九个角色对得上"
        );
        let ObservedUnitAllocationRecords::EveryRoleOfTheVersion(records) =
            &mut observed.unit_allocation_records
        else {
            panic!("照模型写的是带文件的那一版的全部角色");
        };
        records
            .roles
            .retain(|(role, _)| *role != ModelUnitRole::InodeRoot);
        let missing = model
            .judge_roots(
                std::slice::from_ref(&row_publish),
                std::slice::from_ref(&observed),
                &mut counts,
            )
            .expect_err("inode 根这一版有，实现没交");
        assert_eq!(
            missing.aspect,
            ModelDisagreementAspect::AllocationGeneration
        );
        assert!(
            missing.model_answer.contains("InodeRoot"),
            "点名漏交的那个角色：{missing:?}"
        );
    }

    /// 数据单元按单元号逐个比（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K4）：漏交一个单元、单元号重了，都报单元的分配代
    /// 对不上，点名交回的单元号。
    #[test]
    fn a_data_unit_the_implementation_did_not_hand_in_or_handed_in_twice_is_reported() {
        let (model, row_publish) = row_publish_of_a_mount_after_the_first_file();
        let mut observed = observed_exactly_as_the_model_wrote(&row_publish);
        let ObservedUnitAllocationRecords::EveryRoleOfTheVersion(records) =
            &mut observed.unit_allocation_records
        else {
            panic!("照模型写的是带文件的那一版的全部单元");
        };
        let only_data_unit = records.data_units.pop().expect("第一个文件一个数据单元");
        let mut counts = ModelJudgementCounts::default();
        let missing = model
            .judge_allocation_generations(&row_publish, &observed, &mut counts)
            .expect_err("单元 0 这一版有，实现没交");
        assert_eq!(
            missing.aspect,
            ModelDisagreementAspect::AllocationGeneration
        );
        assert!(
            missing.implementation_answer.contains("[]"),
            "点名交回的单元号：{missing:?}"
        );
        let ObservedUnitAllocationRecords::EveryRoleOfTheVersion(records_handed_in_twice) =
            &mut observed.unit_allocation_records
        else {
            panic!("照模型写的是带文件的那一版的全部单元");
        };
        records_handed_in_twice
            .data_units
            .push(only_data_unit.clone());
        records_handed_in_twice.data_units.push(only_data_unit);
        let twice = model
            .judge_allocation_generations(&row_publish, &observed, &mut counts)
            .expect_err("单元 0 交了两次");
        assert!(
            twice.implementation_answer.contains("[0, 0]"),
            "点名交回的单元号：{twice:?}"
        );
    }

    /// 树表 0 条的一版上：零单元发布（暖机）一个字节都不写，输出不带实例表与分配记录，比「这次重写了哪几个角色」（空集），
    /// 多报一个就报；写行那一版（这次重写实例表与这一版自己的分配记录树）写出了整条实例表链与整版分配记录，交回「比不了」的两臂
    /// 就是对不上（代码审阅第 12 条：每一版比实例表、比分配代）。
    #[test]
    fn only_a_zero_unit_publish_on_a_version_without_file_may_hand_in_the_rewritten_roles_only() {
        let mut model = two_device_model();
        model.acquire_and_warm_up_in_the_make_filesystem_process();
        model.close_session();
        let mount = model.answer_mount_writable();
        let row_publish = mount.expected_roots[0].clone();
        let zero_unit_publish = mount.expected_roots[1].clone();
        assert!(
            !row_publish.contents.has_trees() && !zero_unit_publish.contents.has_trees(),
            "起点那一版树表 0 条"
        );
        let observed_with = |root: &ModelRoot,
                             instance_table: ObservedInstanceTable,
                             roles: &[ModelUnitRole]| ObservedRoot {
            key: root.key,
            journal_counter: root.journal_counter,
            rollback_floor: root.rollback_floor,
            file: ObservedFile::NoFile,
            instance_table,
            unit_allocation_records: ObservedUnitAllocationRecords::RewrittenRolesOnly(
                roles.iter().copied().collect(),
            ),
            snapshot_state: ObservedSnapshotState::OF_A_VERSION_WITHOUT_TREES,
            logged_compaction_intent: None,
            first_file_record_truncation_field: None,
            inodes_other_than_the_first: BTreeMap::new(),
            truncation_fields_of_the_inodes_other_than_the_first: BTreeMap::new(),
        };
        let mut counts = ModelJudgementCounts::default();
        assert_eq!(
            model.judge_roots(
                std::slice::from_ref(&zero_unit_publish),
                &[observed_with(
                    &zero_unit_publish,
                    ObservedInstanceTable::NotInTheOutput { why: "用例" },
                    &[]
                )],
                &mut counts
            ),
            Ok(())
        );
        assert_eq!(counts.rewritten_role_sets_compared, 1);
        assert_eq!(counts.instance_tables_not_in_the_output, 1);
        let one_role_too_many = model
            .judge_roots(
                std::slice::from_ref(&zero_unit_publish),
                &[observed_with(
                    &zero_unit_publish,
                    ObservedInstanceTable::NotInTheOutput { why: "用例" },
                    &[ModelUnitRole::InstanceTable],
                )],
                &mut counts,
            )
            .expect_err("零单元发布多报一个重写的角色");
        assert_eq!(
            one_role_too_many.aspect,
            ModelDisagreementAspect::AllocationGeneration
        );
        let row_publish_without_its_instance_table = model
            .judge_roots(
                std::slice::from_ref(&row_publish),
                &[observed_with(
                    &row_publish,
                    ObservedInstanceTable::NotInTheOutput { why: "用例" },
                    &[ModelUnitRole::InstanceTable, ModelUnitRole::AllocationTree],
                )],
                &mut counts,
            )
            .expect_err("写行那一版交回「实例表比不了」");
        assert_eq!(
            row_publish_without_its_instance_table.aspect,
            ModelDisagreementAspect::InstanceTableOfTheVersion
        );
        let row_publish_with_the_rewritten_roles_only = model
            .judge_roots(
                std::slice::from_ref(&row_publish),
                &[observed_with(
                    &row_publish,
                    ObservedInstanceTable::Rows(row_publish.instance_table_rows.as_ref().clone()),
                    &[ModelUnitRole::InstanceTable, ModelUnitRole::AllocationTree],
                )],
                &mut counts,
            )
            .expect_err("写行那一版只交回重写的角色（角色集合与模型的相同）");
        assert_eq!(
            row_publish_with_the_rewritten_roles_only.aspect,
            ModelDisagreementAspect::AllocationGeneration
        );
        assert_eq!(
            counts.instance_tables_not_in_the_output, 2,
            "写行那一版的「比不了」没有计进比不了的次数"
        );
    }

    /// 每一版都比文件内容（代码审阅第 12 条：此前内容只在冷启动那一步比）：写行那一版照抄第一个文件的内容 [3]。
    #[test]
    fn each_version_compares_its_file_content() {
        let (model, row_publish) = row_publish_of_a_mount_after_the_first_file();
        let mut counts = ModelJudgementCounts::default();
        let mut observed = observed_exactly_as_the_model_wrote(&row_publish);
        assert_eq!(
            model.judge_roots(
                std::slice::from_ref(&row_publish),
                std::slice::from_ref(&observed),
                &mut counts
            ),
            Ok(())
        );
        assert_eq!(counts.file_contents_compared, 1);
        for wrong in [
            ObservedFile::Decoded(vec![4]),
            ObservedFile::Undecodable {
                what: "用例".to_string(),
            },
        ] {
            observed.file = wrong.clone();
            let disagreement = model
                .judge_roots(
                    std::slice::from_ref(&row_publish),
                    std::slice::from_ref(&observed),
                    &mut counts,
                )
                .expect_err("内容不是 [3]");
            assert_eq!(
                disagreement.aspect,
                ModelDisagreementAspect::FileContent,
                "{wrong:?}"
            );
        }
    }

    /// 每一版都比整张实例表：写行那一版有实例 1 那一行；少了它报。输出里没有实例表字节的，这次没重写实例表的那一版（暖机）只计数不判，
    /// 这次重写了实例表链的那一版（写行）报「这一版的实例表」（实审 B3c-3：写行那一版写出了整条链，比不了就是对不上）。
    #[test]
    fn each_version_compares_its_instance_table() {
        let (model, row_publish) = row_publish_of_a_mount_after_the_first_file();
        assert_eq!(row_publish.instance_table_rows.len(), 1, "实例 1 那一行");
        let warm_up = model.answer_mount_writable().expected_roots[1].clone();
        assert_eq!(
            warm_up.role_written_at.get(&ModelUnitRole::InstanceTable),
            row_publish
                .role_written_at
                .get(&ModelUnitRole::InstanceTable),
            "暖机那一版照抄写行那一版的实例表，不重写它"
        );
        let mut counts = ModelJudgementCounts::default();
        let mut observed = observed_exactly_as_the_model_wrote(&row_publish);
        assert_eq!(
            model.judge_roots(
                std::slice::from_ref(&row_publish),
                std::slice::from_ref(&observed),
                &mut counts
            ),
            Ok(())
        );
        assert_eq!(counts.instance_tables_compared, 1);
        observed.instance_table = ObservedInstanceTable::Rows(Vec::new());
        let disagreement = model
            .judge_roots(
                std::slice::from_ref(&row_publish),
                std::slice::from_ref(&observed),
                &mut counts,
            )
            .expect_err("实例表丢了那一行");
        assert_eq!(
            disagreement.aspect,
            ModelDisagreementAspect::InstanceTableOfTheVersion
        );
        observed.instance_table = ObservedInstanceTable::NotInTheOutput { why: "用例" };
        let row_publish_not_in_the_output = model
            .judge_roots(
                std::slice::from_ref(&row_publish),
                std::slice::from_ref(&observed),
                &mut counts,
            )
            .expect_err("写行那一版这次重写了实例表链，交回「比不了」");
        assert_eq!(
            row_publish_not_in_the_output.aspect,
            ModelDisagreementAspect::InstanceTableOfTheVersion
        );
        assert_eq!(counts.instance_tables_not_in_the_output, 0);
        let mut warm_up_observed = observed_exactly_as_the_model_wrote(&warm_up);
        warm_up_observed.instance_table = ObservedInstanceTable::NotInTheOutput { why: "用例" };
        assert_eq!(
            model.judge_roots(
                std::slice::from_ref(&warm_up),
                std::slice::from_ref(&warm_up_observed),
                &mut counts
            ),
            Ok(())
        );
        assert_eq!(counts.instance_tables_not_in_the_output, 1);
    }

    fn refusal_of_the_one_reread(wrote_anything: bool) -> ObservedOutcome {
        ObservedOutcome::Refused {
            member: "MountError::NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion)".to_string(),
            reason: ObservedRefusalReason::Explained(
                ModelRefusalReason::NewerPublishWitnessedBySystemConfigurationStillUnreadableAfterOneReread,
            ),
            publishes_completed: 0,
            wrote_anything,
            reported_ceiling: None,
            reported_root_ring_slot_still_bad_after_one_reread: None,
        }
    }

    /// 崩溃恢复抛弃根那一步（C554 乙，用户 2026-09-27 定）：最新那条根 (2, 9) 是这个会话发的、系统配置见证过，整次挂载读不出
    /// ⇒ 模型答拒可写，理由就是「重读一次仍读不出」这一条，一次发布都不做。实现照这样拒（取号之前、录制流一步不多）对得上、
    /// 模型状态逐项不变；实现做成了、拒之前写了盘、拒的理由不是这一条，各对不上在自己那一格。
    #[test]
    fn crash_recovery_that_cannot_read_the_witnessed_newest_root_is_refused_before_any_write_and_leaves_the_model_unchanged(
    ) {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        model.close_session();
        assert_eq!(model.newest_root().key, key(9, 2));
        let answer = model
            .answer_mount_writable_with_the_newest_root_unreadable()
            .expect("环里 (2, 9) 之前还有根");
        assert_eq!(
            answer.required_refusals,
            BTreeSet::from([
                ModelRefusalReason::NewerPublishWitnessedBySystemConfigurationStillUnreadableAfterOneReread
            ]),
            "系统配置见证过的最新那条根重读一次仍读不出：拒可写"
        );
        assert_eq!(
            (answer.expected_roots.len(), &answer.expected_mount),
            (0, &None),
            "拒了就一条根都不写、不取号"
        );
        let model_before = model.clone();
        let counts = model
            .judge_and_advance(&answer, &refusal_of_the_one_reread(false))
            .expect("照答案拒：取号之前、一个字节都没写");
        assert_eq!(counts.required_refusals_matched, 1);
        assert_eq!(model, model_before, "拒了之后模型状态逐项不变");
        let wrote_before_refusing = model
            .clone()
            .judge_and_advance(&answer, &refusal_of_the_one_reread(true))
            .expect_err("拒之前录制流里多了写");
        assert_eq!(
            wrote_before_refusing.aspect,
            ModelDisagreementAspect::WroteBeforeRefusing
        );
        let other_reason = model
            .clone()
            .judge_and_advance(
                &answer,
                &ObservedOutcome::Refused {
                    member: "MountError::NewerStateStillUnreadableAfterOneReread(InstanceTableOfTheNewestRootForTheShadowLedger)".to_string(),
                    reason: ObservedRefusalReason::Unexplained,
                    publishes_completed: 0,
                    wrote_anything: false,
                    reported_ceiling: None,
                    reported_root_ring_slot_still_bad_after_one_reread: None,
                },
            )
            .expect_err("拒的不是「重读一次仍读不出」那一条");
        assert_eq!(other_reason.aspect, ModelDisagreementAspect::RefusalReason);
        let succeeded = model
            .clone()
            .judge_and_advance(
                &answer,
                &ObservedOutcome::Succeeded(ObservedEffect::Mount {
                    instance: ModelInstanceGeneration(3),
                    rows_written: Vec::new(),
                    roots: Vec::new(),
                }),
            )
            .expect_err("实现照旧抛弃最新那条根、挂载做成");
        assert_eq!(
            succeeded.aspect,
            ModelDisagreementAspect::SucceededWhenModelRequiresRefusal
        );
    }

    /// 单元区墙的区间按单元区起点随环长走算（C475（非默认环长下单元区起点取编译期常量），实审 A3b 第 15 条）：6 MiB 的环，单元区从槽 1408 起；
    /// 盘只比默认环长那一档的单元区起点（编译期常量那个 50176）多 1000 槽。第一个文件之后一次覆盖写的上界乘 64 落在两道墙之间：
    /// 按随环长走的起点，单元区有四万九千多槽，离墙还远，实现报单元区墙就是对不上；按编译期常量取起点，单元区只剩 1000 槽，同一个拒绝落在区间里放行。
    /// 随机历史的单元区墙取样点（两块单元区 256 槽的小盘、准入关掉的那一段）一次单元区墙的拒绝都没有，两个起点在它上面判得一样（取样点不敏感）。
    #[test]
    fn a_unit_area_wall_refusal_far_below_the_wall_that_follows_the_ring_is_judged_a_disagreement()
    {
        let journal_ring_bytes = 6 << 20;
        let slots_past_the_unit_area_start_of_the_default_ring = 1000;
        let device_slots = model_unit_area_start_slot(JOURNAL_RING_DEFAULT_BYTES)
            + slots_past_the_unit_area_start_of_the_default_ring;
        let mut model = IdealModel::after_make_filesystem_on_a_journal_ring_of(
            ModelPoolGeometry {
                devices: vec![ModelDeviceIdentity(0), ModelDeviceIdentity(1)],
                device_size_in_bytes: device_slots * SLOT_BYTES,
            },
            journal_ring_bytes,
        );
        assert_eq!(
            model.unit_area_slots_per_device(),
            device_slots - 1408,
            "6 MiB 的环末尾的下一个槽 1024 + 384 = 1408 起"
        );
        model.acquire_and_warm_up_in_the_make_filesystem_process();
        let first = model
            .answer_publish_first_file(&[3])
            .expect("会话开着、现行 txg 2");
        succeed(&mut model, &first);
        let overwrite = model
            .answer_publish_overwrite(&[4])
            .expect("会话开着、现行版本带文件");
        let planned_slots = overwrite.planned_publish_upper_bounds[0].occupied_slots_per_device;
        assert!(
            planned_slots * CLUSTER_SEGMENT_SLOTS
                > slots_past_the_unit_area_start_of_the_default_ring
                && planned_slots * CLUSTER_SEGMENT_SLOTS <= device_slots - 1408,
            "取样点：上界 {planned_slots} 槽乘 64 落在两道墙之间"
        );
        let disagreement = model
            .judge_and_advance(
                &overwrite,
                &ObservedOutcome::Refused {
                    member: "PublishError::PlacementRefused(NoPlacementOnAnyDevice)".to_string(),
                    reason: ObservedRefusalReason::Explained(ModelRefusalReason::UnitAreaWall),
                    publishes_completed: 0,
                    wrote_anything: false,
                    reported_ceiling: None,
                    reported_root_ring_slot_still_bad_after_one_reread: None,
                },
            )
            .expect_err("离墙还远：单元区墙的拒绝不在允许的区间里");
        assert_eq!(
            disagreement.aspect,
            ModelDisagreementAspect::RefusedWhenModelRequiresSuccess
        );
    }

    /// extent 树下段的节点数与层数（D8（核心索引结构） 已定项 14：一个单元内联、不建下段；叶罩 144 个单元、内部扇出 147）：
    /// 模型按它给顺序写与删除批的占槽上界。
    #[test]
    fn the_lower_segment_of_a_file_has_a_leaf_per_144_data_units_and_an_internal_node_per_147_children(
    ) {
        assert_eq!(
            [1, 2, 144, 145, 288, 289].map(|data_units| (
                extent_lower_segment_nodes(data_units),
                extent_lower_segment_levels(data_units)
            )),
            [(0, 0), (1, 1), (1, 1), (2 + 1, 2), (2 + 1, 2), (3 + 1, 2)]
        );
        assert_eq!(
            (
                extent_lower_segment_nodes(144 * 147 + 1),
                extent_lower_segment_levels(144 * 147 + 1)
            ),
            (148 + 2 + 1, 3),
            "148 片叶要两个第一层节点，再一个根"
        );
    }

    /// 中央映射树节点数的上界（D8（核心索引结构） 已定项 11：叶装 294 条、内部节点装 143 个孩子，装不下从中间切，切出来的右半至少 147 条 / 72 个孩子；
    /// 收缩只摘空节点）：条目不到 147 条时一次都切不了，上界照上一版的；每多 147 条至多多切一片叶、多长一个新根；上一版长过的节点不往回收。
    #[test]
    fn central_mapping_tree_nodes_grow_by_two_per_147_entries_and_keep_the_previous_bound() {
        assert_eq!(
            (
                central_mapping_tree_leaf_entry_capacity(),
                central_mapping_tree_internal_child_capacity(),
                smallest_half_of_a_split(central_mapping_tree_leaf_entry_capacity()),
                smallest_half_of_a_split(central_mapping_tree_internal_child_capacity()),
            ),
            (294, 143, 147, 72)
        );
        assert_eq!(
            [(1, 146), (1, 147), (1, 294), (5, 10), (1, 147 * 72)]
                .map(|(nodes_before, entries)| central_mapping_tree_nodes_upper_bound(
                    nodes_before,
                    entries
                )),
            [1, 1 + 2, 1 + 2 * 2, 5, 1 + 2 * 72 + 2 * 2],
            "147 × 72 条：叶至多切 72 次，内部节点再切的次数 t 满足 72 t ≤ N − 1，停在 N = 149（t = 2）"
        );
    }

    /// 顺序写一个跨三片下段叶的文件（292 个单元）：映射条目 302 条加分配记录树的节点，多于一片叶装的 294 条，映射树从中间切开。
    /// 4 GiB 两块盘、第一个文件之后（每盘写过 21 槽，映射树一个节点）：这次映射树之外写 595 槽（数据 2 × 292、下段 3 片叶加 1 个内部节点、
    /// extent 根 1、inode 叶 2、inode 根 1、记账树、映射树的根、树表各 1）；分配记录树至多 123 个节点，条目至多 302 + 123 = 425，
    /// 叶至多切 2 次 ⇒ 映射树至多 1 + 2 × 2 = 5 个节点，根之下 4 个各占一槽：这次占槽上界 595 + 4 + 123 = 722。
    /// 条目里 302 = 数据 292 + 下段 4 + extent 根、inode 叶、inode 根、记账树各 1 + 谱系树按 txg 4 至多一片容器加根 2。
    #[test]
    fn a_sequential_write_of_three_lower_leaves_counts_the_central_mapping_tree_nodes_below_the_root_in_the_occupied_slots(
    ) {
        let mut model = two_device_model();
        model.acquire_and_warm_up_in_the_make_filesystem_process();
        let first = model
            .answer_publish_first_file(&[3])
            .expect("会话开着、现行 txg 2");
        succeed(&mut model, &first);
        let slots_before = model.newest_root().occupied_slots_upper_bound_per_device;
        assert_eq!(
            (
                slots_before,
                model.newest_root().central_mapping_tree_nodes_upper_bound
            ),
            (3 + 9 + 9, 1),
            "第一个文件之后：映射条目十来条，映射树一个节点"
        );
        let data_units = 2 * EXTENT_TREE_LOWER_LEAF_DATA_UNITS + 4;
        let capacity = data_unit_payload_capacity_in_bytes();
        let content =
            vec![5_u8; usize::try_from((data_units - 1) * capacity + 1).expect("九兆多字节")];
        assert_eq!(
            data_units_of_content(u64::try_from(content.len()).expect("内容长度")),
            292
        );
        let sequential_write = model
            .answer_publish_sequential_write(&content)
            .expect("会话开着、现行版本带文件");
        assert!(
            sequential_write.required_refusals.is_empty(),
            "{:?}",
            sequential_write.required_refusals
        );
        let written = &sequential_write.expected_roots[0];
        let slots_outside_the_two_trees = 2 * 292 + (3 + 1) + 1 + 2 + 1 + 1 + 1 + 1;
        assert_eq!(
            (
                written.central_mapping_tree_nodes_upper_bound,
                model.allocation_record_tree_nodes_rewritten_upper_bound(
                    slots_before + slots_outside_the_two_trees + 4
                ),
                written.occupied_slots_upper_bound_per_device - slots_before,
            ),
            (5, 123, slots_outside_the_two_trees + 4 + 123),
            "映射树至多 5 个节点（根之下 4 个），分配记录树至多 123 个节点：这次占槽上界 722"
        );
    }

    /// 记账树节点数的上界（D8（核心索引结构） 已定项 11：叶装 477 行、内部节点装 150 个孩子，装不下从中间切，切开的两半各至少 239 行 / 75 个孩子；
    /// 收缩只摘空节点）：不到 2 × 239 = 478 行一次都切不了，上界照上一版的；从一个节点长到 478 行至多切一片叶、根切一次，3 个节点；
    /// 717 = 3 × 239 行至多切两片叶，4 个；76 × 239 行 76 片叶挂一个根，77 个；151 × 239 行 151 片叶一个根装不下，根切一次、长成两个内部节点加新根，154 个。
    /// 上一版长过的节点不往回收（3 个节点、477 行仍是 3），长过之后每多切一片叶多两个（叶一个、根一个的上界）。
    #[test]
    fn accounting_tree_nodes_grow_to_three_at_478_rows_and_keep_the_previous_bound() {
        assert_eq!(
            (
                accounting_tree_leaf_entry_capacity(),
                accounting_tree_internal_child_capacity(),
                smallest_half_of_a_split(accounting_tree_leaf_entry_capacity()),
                smallest_half_of_a_split(accounting_tree_internal_child_capacity()),
            ),
            (477, 150, 239, 75)
        );
        assert_eq!(
            [
                (1, 15),
                (1, 477),
                (1, 478),
                (1, 3 * 239),
                (1, 76 * 239),
                (1, 151 * 239),
                (3, 477),
                (3, 479),
            ]
            .map(|(nodes_before, rows)| accounting_tree_nodes_upper_bound(nodes_before, rows)),
            [1, 1, 3, 4, 77, 154, 3, 5],
            "478 行：叶切 1 次、根切 1 次；151 × 239 行：叶切 150 次、内部节点切 1 次（2 × 75 ≤ 153）、根切 2 次之内取 1 + t = 2"
        );
    }

    /// 两块 4 GiB 盘：第一个文件、unlink、最后一批之后一次建一个 inode，第 462 个那一版写 477 行、记账树一个节点；第 463 个那次照抄 463 级台阶、
    /// 写 478 行，记账树长成根加两片叶（上界 3）。这次占槽上界：分配记录树与映射树之外写 8 槽（新记录落的第 2 片 inode 叶容器 2、inode 根 1、
    /// 记账树 3 个节点各 1、映射树的根 1、树表 1），加映射树根之下的节点与分配记录树按上界算的节点。
    #[test]
    fn the_inode_creation_whose_watermark_step_crosses_one_accounting_leaf_counts_the_nodes_below_the_root_in_the_occupied_slots(
    ) {
        let mut model = two_device_model();
        model.acquire_and_warm_up_in_the_make_filesystem_process();
        let first = model
            .answer_publish_first_file(&[3])
            .expect("会话开着、现行 txg 2");
        succeed(&mut model, &first);
        let intent = model
            .answer_unlink_intent()
            .expect("会话开着、现行版本带活文件");
        succeed(&mut model, &intent);
        let final_batch = model
            .answer_deletion_step()
            .expect("会话开着、现行版本删除中");
        succeed(&mut model, &final_batch);
        for _ in 0..462 {
            let created = model
                .answer_create_inode()
                .expect("会话开着、第一个文件已删完");
            succeed(&mut model, &created);
        }
        let before = model.newest_root().clone();
        let created = model
            .answer_create_inode()
            .expect("会话开着、第一个文件已删完");
        assert!(
            created.required_refusals.is_empty(),
            "{:?}",
            created.required_refusals
        );
        let written = &created.expected_roots[0];
        let slots_outside_the_two_trees = 2 + 1 + 3 + 1 + 1;
        let mapping_nodes_below_the_root = written.central_mapping_tree_nodes_upper_bound - 1;
        assert_eq!(
            (
                (before.accounting_rows, before.accounting_tree_nodes_upper_bound),
                (written.accounting_rows, written.accounting_tree_nodes_upper_bound),
                written.occupied_slots_upper_bound_per_device
                    - before.occupied_slots_upper_bound_per_device,
            ),
            (
                (477, 1),
                (478, 3),
                slots_outside_the_two_trees
                    + mapping_nodes_below_the_root
                    + model.allocation_record_tree_nodes_rewritten_upper_bound(
                        before.occupied_slots_upper_bound_per_device
                            + slots_outside_the_two_trees
                            + mapping_nodes_below_the_root
                    ),
            ),
            "第 462 个那一版 477 行一个节点；第 463 个那次 478 行、根加两片叶，三个节点各占一槽进这次的占槽上界"
        );
    }

    /// 一个码 2 根装 135 片叶容器（D8（核心索引结构） 已定项 6）：删完之后建的第 233 × 135 − 1 条（从 0 数）落在最后那片（第 134 片），
    /// 第 233 × 135 条要第 136 片、根要长一层，模型的落点式子在那里 panic（判决 `research/prompts/m3-formal-code-b25-r1-main-verification.md`
    /// K1 第四行：保留 `assert!`、不答拒；core 在同一条拒，`MoreLeafContainersThanOneRootNodeHolds`）。
    #[test]
    fn the_inode_leaf_container_position_reaches_the_last_of_135_leaf_containers_and_panics_one_record_beyond(
    ) {
        const LEAF_CONTAINERS_ONE_CODE_TWO_ROOT_HOLDS: u64 = 135;
        assert_eq!(
            index_node_entry_capacity(INODE_TREE_KEY_BYTES, INODE_INTERNAL_ENTRY),
            LEAF_CONTAINERS_ONE_CODE_TWO_ROOT_HOLDS,
            "模型按条款算的码 2 根扇出是 135"
        );
        let records_one_root_holds = INODE_LEAF_RECORDS * LEAF_CONTAINERS_ONE_CODE_TWO_ROOT_HOLDS;
        assert_eq!(
            inode_leaf_container_position_of_a_new_record(records_one_root_holds - 1),
            LEAF_CONTAINERS_ONE_CODE_TWO_ROOT_HOLDS - 1,
            "第 233 × 135 − 1 条落在第 134 片"
        );
        let one_record_beyond = std::panic::catch_unwind(|| {
            inode_leaf_container_position_of_a_new_record(records_one_root_holds)
        });
        let payload =
            one_record_beyond.expect_err("第 233 × 135 条要第 136 片：模型的落点式子 panic");
        let message = payload
            .downcast_ref::<String>()
            .expect("assert! 带格式参数，panic 的载荷是 String");
        assert!(
            message.contains(&format!(
                "第 {records_one_root_holds} 条落在第 {LEAF_CONTAINERS_ONE_CODE_TWO_ROOT_HOLDS} 片"
            )),
            "panic 的是落点式子那一条 assert!：{message}"
        );
    }

    /// 一版里最左那片之外的 inode 叶容器角色与各自的分配代。
    fn inode_leaf_containers_right_of_the_leftmost(
        root: &ModelRoot,
    ) -> BTreeMap<ModelUnitRole, ModelCheckpointTxg> {
        root.role_written_at
            .iter()
            .filter(|(role, _)| matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(_)))
            .map(|(role, written_at)| (*role, *written_at))
            .collect()
    }

    /// 只供测试的一版：照 `base` 抄，inode 树换成 `leaf_containers` 片叶容器（从 1 数，最左那片算 1），最左那片之外的第 k 片记成
    /// txg `first_written_at + k` 写的（各片分配代互不相同，回退取错一片就看得出来）。
    fn version_with_inode_leaf_containers(
        base: &ModelRoot,
        leaf_containers: u64,
        first_written_at: ModelCheckpointTxg,
    ) -> ModelRoot {
        let mut version = base.clone();
        version
            .role_written_at
            .retain(|role, _| !matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(_)));
        for position_from_the_leftmost in 1..leaf_containers {
            version.role_written_at.insert(
                ModelUnitRole::inode_leaf_container_at(position_from_the_leftmost),
                ModelCheckpointTxg(first_written_at.0 + position_from_the_leftmost),
            );
        }
        version
    }

    /// 模型回退那次发布对最左那片之外的叶容器的处置（`next_root_with_the_user_visible_trees` 取回退目标那一支里丢现行那几片、取目标那几片的两句），
    /// 结构单测（第 30 批判决 `research/prompts/m3-formal-code-b30-r1-main-verification.md` 第二节 M1「第 k 片都对」那一行的改法 B4）：
    /// 现行与目标各取 1、2、3、4、5、30、100、120、135 片（135 是一个码 2 根装得下的上界，D8（核心索引结构） 已定项 6）的每一种组合，回退写出的那一版
    /// 最左那片之外的叶容器（角色与分配代）恰好等于目标那一版的——现行有、目标没有的那几片这一版也没有，目标的每一片都取回、分配代取目标的。
    /// 30、100、120 罩只按片数触发、不按位置触发的错（最左之外各 29、99、119 片）：1–5 与 135 之间没有取样点时，只在目标装 60–118 片才错的回退照绿
    /// （第 32–33 批代码轮判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第二节 P1「B4」那一行，`.claude/rules/mutation-sampling.md` 第三类）；
    /// 只有 100 时，最左之外的片数落在 6–58、5–98、100–133 里才错的回退照绿（第 34 批代码轮判决
    /// `research/prompts/m3-formal-code-b34-r1-main-verification.md` 第二节 P5「B4」那一行），30 落进前两个窗口、120 落进第三个。
    /// 按取样点逐片往外加历史用例追不完（回退只取回或只丢某几片，只有建到那几片之外的历史抓得到），这一条不跑历史。
    #[test]
    fn a_rollback_takes_exactly_the_inode_leaf_containers_right_of_the_leftmost_of_its_target_for_every_pair_of_container_counts(
    ) {
        const CONTAINER_COUNTS: [u64; 9] = [1, 2, 3, 4, 5, 30, 100, 120, 135];
        let model = model_after_four_overwrites_in_a_second_instance();
        let newest = model.newest_root().clone();
        let mut pairs_judged = 0;
        for current_leaf_containers in CONTAINER_COUNTS {
            for target_leaf_containers in CONTAINER_COUNTS {
                let target = version_with_inode_leaf_containers(
                    &newest,
                    target_leaf_containers,
                    ModelCheckpointTxg(1000),
                );
                let current = version_with_inode_leaf_containers(
                    &newest,
                    current_leaf_containers,
                    ModelCheckpointTxg(2000),
                );
                let after_the_rollback = model.next_root_with_the_user_visible_trees(
                    &current,
                    current.key.instance,
                    ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1),
                    ModelJournalCounter(model.highest_journal_counter.0 + 1),
                    ModelPublishKind::EmptyOnFileVersion,
                    current.rollback_floor,
                    None,
                    Rc::clone(&current.instance_table_rows),
                    UserVisibleTreesOfTheNewVersion::TakenFromTheRollbackTarget(&target),
                    InodesOtherThanTheFirstChangeOfThePublish::Carried,
                );
                assert_eq!(
                    inode_leaf_containers_right_of_the_leftmost(&after_the_rollback),
                    inode_leaf_containers_right_of_the_leftmost(&target),
                    "现行 {current_leaf_containers} 片、目标 {target_leaf_containers} 片：回退写出的那一版最左那片之外的叶容器要恰好是目标那一版的"
                );
                assert_eq!(
                    inode_leaf_containers_right_of_the_leftmost(&target).len(),
                    usize::try_from(target_leaf_containers - 1).expect("片数小于 135"),
                    "搭建：目标那一版最左那片之外有 {} 片",
                    target_leaf_containers - 1
                );
                pairs_judged += 1;
            }
        }
        assert_eq!(
            pairs_judged,
            CONTAINER_COUNTS.len() * CONTAINER_COUNTS.len()
        );
    }

    /// 模型里现行那一版的快照（会话开着）。
    fn current_snapshots(model: &IdealModel) -> Vec<ModelSnapshot> {
        model
            .snapshots_of_the_current_version()
            .expect("会话开着")
            .to_vec()
    }

    /// 建快照进根环（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S2、S7）：会话现行那一版带文件（内容 [9]，txg 9），
    /// 连建十个，每个是一次独立发布（txg 10 到 19）、各占一个事务；第 k 个的前驱是第 k − 1 个的诞生 txg（第一个是 0），活头前驱是最后一个的；
    /// 第十一个要求拒成 `MoreSnapshotsThanTheFirstVersionAllows`（第七节岔路 1 的「10」）。之后覆盖写成 [10]，记下的快照内容不跟着变。
    #[test]
    fn the_model_publishes_ten_snapshots_into_the_ring_and_requires_the_eleventh_to_be_refused() {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        for snapshot_index in 0..MODEL_SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION {
            let creation = model
                .answer_create_snapshot()
                .expect("会话开着、现行那一版带文件");
            assert!(
                creation.required_refusals.is_empty(),
                "第 {snapshot_index} 个照建：{:?}",
                creation.required_refusals
            );
            succeed(&mut model, &creation);
        }
        let eleventh = model.answer_create_snapshot().expect("会话开着");
        assert_eq!(
            eleventh.required_refusals,
            BTreeSet::from([ModelRefusalReason::MoreSnapshotsThanTheFirstVersionAllows]),
            "第十一个拒"
        );
        let snapshots = current_snapshots(&model);
        assert_eq!(
            snapshots.len(),
            MODEL_SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION
        );
        for (snapshot, birth_txg) in snapshots.iter().zip(10_u64..) {
            assert_eq!(
                snapshot.birth_txg,
                ModelCheckpointTxg(birth_txg),
                "S.txg 是建它那次发布的 txg"
            );
            assert_eq!(
                snapshot.previous_snapshot_txg,
                ModelCheckpointTxg(if birth_txg == 10 { 0 } else { birth_txg - 1 }),
                "前驱是排在它前面那一个的诞生 txg"
            );
            assert_eq!(
                snapshot.inode_tree_written_by,
                key(9, 2),
                "两条根指针抄的是 txg 9 那一版的"
            );
        }
        let current = model.current_version_in_the_session().expect("会话开着");
        assert_eq!(
            current.previous_snapshot_txg_of_the_live_head,
            ModelCheckpointTxg(19),
            "活头前驱是最后一个快照的诞生 txg"
        );
        assert_eq!(
            current.highest_transaction_number_in_its_instance,
            4 + 10,
            "四次覆盖写各一个事务，十次建快照各一个事务"
        );
        let overwrite = model
            .answer_publish_overwrite(&[10])
            .expect("会话开着、现行版本带文件");
        succeed(&mut model, &overwrite);
        assert_eq!(
            current_snapshots(&model)
                .iter()
                .map(|snapshot| snapshot.content_at_creation.clone())
                .collect::<Vec<_>>(),
            vec![Some(Rc::from(vec![9_u8])); MODEL_SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION],
            "十个快照记的都是建时的内容 [9]，之后的覆盖写不改它们"
        );
    }

    /// 快照那几样逐项比（判决 S7）：实现交回的这一版照模型记的原样时对得上；活头前驱、快照列表、deadlist 条目数、被钉映射条目数各改一样，
    /// 各报自己那一格。
    #[test]
    fn each_snapshot_field_of_a_version_that_differs_is_reported_as_its_own_aspect() {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        let creation = model.answer_create_snapshot().expect("建快照");
        succeed(&mut model, &creation);
        let overwrite = model.answer_publish_overwrite(&[10]).expect("覆盖写");
        succeed(&mut model, &overwrite);
        let version = model
            .current_version_in_the_session()
            .expect("会话开着")
            .clone();
        let aspect_of = |observed: ObservedRoot| {
            model
                .judge_roots(
                    std::slice::from_ref(&version),
                    &[observed],
                    &mut ModelJudgementCounts::default(),
                )
                .map_err(|disagreement| disagreement.aspect)
        };
        assert_eq!(
            aspect_of(observed_exactly_as_the_model_wrote(&version)),
            Ok(())
        );
        let mut wrong_live_head = observed_exactly_as_the_model_wrote(&version);
        wrong_live_head
            .snapshot_state
            .live_head_previous_snapshot_txg = Some(ModelCheckpointTxg(0));
        assert_eq!(
            aspect_of(wrong_live_head),
            Err(ModelDisagreementAspect::LiveHeadPreviousSnapshotTxg)
        );
        let mut wrong_snapshots = observed_exactly_as_the_model_wrote(&version);
        wrong_snapshots.snapshot_state.snapshots[0].is_being_destroyed = true;
        assert_eq!(
            aspect_of(wrong_snapshots),
            Err(ModelDisagreementAspect::Snapshots)
        );
        let mut wrong_deadlist = observed_exactly_as_the_model_wrote(&version);
        wrong_deadlist
            .snapshot_state
            .deadlist_entries
            .push(ModelDeadlistEntry {
                epoch: ModelCheckpointTxg(0),
                mapping_key_unit_class: MODEL_MAPPING_KEY_UNIT_CLASS_DATA,
                mapping_key_birth_tree: TREE_IDENTIFIER_EXTENT,
                birth_txg: ModelCheckpointTxg(0),
            });
        assert_eq!(
            aspect_of(wrong_deadlist),
            Err(ModelDisagreementAspect::DeadlistEntries)
        );
        // 条数相同、一条的类标签不同（一批放错一条就是这个样子）：逐条比照样报 deadlist 那一格。
        let mut wrong_deadlist_entry = observed_exactly_as_the_model_wrote(&version);
        wrong_deadlist_entry.snapshot_state.deadlist_entries[0].mapping_key_unit_class =
            MODEL_MAPPING_KEY_UNIT_CLASS_PACKED_RECORDS;
        assert_eq!(
            aspect_of(wrong_deadlist_entry),
            Err(ModelDisagreementAspect::DeadlistEntries),
            "条数相同、有一条不同也报"
        );
        let mut wrong_pinned_mapping_entries = observed_exactly_as_the_model_wrote(&version);
        wrong_pinned_mapping_entries
            .snapshot_state
            .pinned_mapping_entries += 1;
        assert_eq!(
            aspect_of(wrong_pinned_mapping_entries),
            Err(ModelDisagreementAspect::PinnedMappingEntries)
        );
    }

    /// 快照下覆盖写钉住被换下的用户可见单元、删快照放掉它们（判决第三节 S3、S4）：快照建在 txg 10（内容 [9] 是 txg 9 写的），txg 11 覆盖写换下
    /// 数据单元、extent 根、inode 叶与根四个（都生在 txg 9 ≤ 10），四条 deadlist 条目、四条被钉的映射条目，纪元 10；读快照仍读出 [9]。
    /// 删它：意图那次（txg 12）带上销毁中、把前驱 0 传给活头，一条都不放；下一批（txg 13）能放的四条不多于一批，是最后一批，放掉四条、删掉快照。
    #[test]
    fn the_model_pins_the_units_a_snapshot_references_and_releases_them_when_the_snapshot_is_destroyed(
    ) {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        let creation = model.answer_create_snapshot().expect("建快照");
        succeed(&mut model, &creation);
        let overwrite = model.answer_publish_overwrite(&[10]).expect("覆盖写");
        succeed(&mut model, &overwrite);
        let after_the_overwrite = model
            .current_version_in_the_session()
            .expect("会话开着")
            .clone();
        assert_eq!(
            after_the_overwrite.deadlist.as_ref(),
            &[
                ModelDeadlistEntry {
                    epoch: ModelCheckpointTxg(10),
                    mapping_key_unit_class: MODEL_MAPPING_KEY_UNIT_CLASS_DATA,
                    mapping_key_birth_tree: TREE_IDENTIFIER_EXTENT,
                    birth_txg: ModelCheckpointTxg(9),
                },
                ModelDeadlistEntry {
                    epoch: ModelCheckpointTxg(10),
                    mapping_key_unit_class: MODEL_MAPPING_KEY_UNIT_CLASS_INDEX_NODE,
                    mapping_key_birth_tree: TREE_IDENTIFIER_EXTENT,
                    birth_txg: ModelCheckpointTxg(9),
                },
                ModelDeadlistEntry {
                    epoch: ModelCheckpointTxg(10),
                    mapping_key_unit_class: MODEL_MAPPING_KEY_UNIT_CLASS_PACKED_RECORDS,
                    mapping_key_birth_tree: TREE_IDENTIFIER_INODE,
                    birth_txg: ModelCheckpointTxg(9),
                },
                ModelDeadlistEntry {
                    epoch: ModelCheckpointTxg(10),
                    mapping_key_unit_class: MODEL_MAPPING_KEY_UNIT_CLASS_INDEX_NODE,
                    mapping_key_birth_tree: TREE_IDENTIFIER_INODE,
                    birth_txg: ModelCheckpointTxg(9),
                },
            ],
            "数据单元、extent 根、inode 叶与根各一条，纪元是活头前驱 10（数据单元先进，叶容器与根按角色次序）"
        );
        assert_eq!(
            after_the_overwrite.pinned_mapping_entries, 4,
            "四条映射条目留着"
        );
        assert_eq!(
            model
                .judge_snapshot_read_back(0, &ObservedSnapshotReadBack::File(vec![9]))
                .map(|counts| counts.snapshot_read_backs_compared),
            Ok(1),
            "覆盖写之后快照仍读出建时的 [9]"
        );
        assert_eq!(
            model
                .judge_snapshot_read_back(0, &ObservedSnapshotReadBack::File(vec![10]))
                .map_err(|disagreement| disagreement.aspect),
            Err(ModelDisagreementAspect::SnapshotReadBack),
            "读出活头的 [10] 是读快照那一格对不上"
        );
        let intent = model.answer_destroy_snapshot(0).expect("删快照的意图");
        succeed(&mut model, &intent);
        let after_the_intent = model
            .current_version_in_the_session()
            .expect("会话开着")
            .clone();
        assert!(
            after_the_intent.snapshots[0].is_being_destroyed,
            "带上销毁中"
        );
        assert_eq!(
            after_the_intent.previous_snapshot_txg_of_the_live_head,
            ModelCheckpointTxg(0),
            "被删那一个的前驱 0 传给活头"
        );
        assert_eq!(after_the_intent.deadlist.len(), 4, "意图那次一条都不放");
        assert!(
            model.current_version_has_a_step_to_walk(),
            "销毁中的快照要走空"
        );
        let final_batch = model.answer_next_step_of_the_walk().expect("走空的一步");
        assert_eq!(
            final_batch.operation,
            ModelOperationKind::PublishSnapshotDestructionStep
        );
        succeed(&mut model, &final_batch);
        let after_the_final_batch = model.current_version_in_the_session().expect("会话开着");
        assert!(
            after_the_final_batch.snapshots.is_empty(),
            "最后一批删掉那一对条目"
        );
        assert!(after_the_final_batch.deadlist.is_empty(), "四条都放掉");
        assert_eq!(
            after_the_final_batch.pinned_mapping_entries, 0,
            "留存的映射条目跟着删"
        );
        assert!(!model.current_version_has_a_step_to_walk(), "走空走完");
    }

    /// 快照删到一半（意图之后）的那一版与删完的那一版比「非空」（`mount::user_visible_trees_changed` 连同快照条目的根指针）：建快照那次换了状态，
    /// 意图那次只改记号与前驱、状态不变，最后一批删掉那一对条目又换了状态。
    #[test]
    fn a_snapshot_creation_and_the_final_destruction_batch_change_the_user_visible_state_and_the_intent_does_not(
    ) {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        let before = model
            .current_version_in_the_session()
            .expect("会话开着")
            .user_visible_state();
        let creation = model.answer_create_snapshot().expect("建快照");
        succeed(&mut model, &creation);
        let created = model
            .current_version_in_the_session()
            .expect("会话开着")
            .user_visible_state();
        assert_ne!(created, before, "多了一对快照根：非空");
        let intent = model.answer_destroy_snapshot(0).expect("意图");
        succeed(&mut model, &intent);
        let intended = model
            .current_version_in_the_session()
            .expect("会话开着")
            .user_visible_state();
        assert_eq!(intended, created, "意图那次不算非空");
        let final_batch = model.answer_next_step_of_the_walk().expect("最后一批");
        succeed(&mut model, &final_batch);
        let destroyed = model
            .current_version_in_the_session()
            .expect("会话开着")
            .user_visible_state();
        assert_ne!(destroyed, intended, "删掉那一对：非空");
        assert_eq!(destroyed, before, "删完回到建快照之前的状态");
    }

    /// 整理的一次发布看得见的几样（[`ModelCompactionPublishSeen`]）：R 取 0 号盘第一个文件那次发布的那一段。
    fn compaction_publish_seen(
        units_relocated: u64,
        is_an_intent_logged_after_the_publish: bool,
        is_the_region_empty: bool,
        user_visible_root_units_relocated: Vec<ModelUserVisibleRootUnit>,
    ) -> ModelCompactionPublishSeen {
        ModelCompactionPublishSeen {
            units_at_most: 2,
            units_relocated,
            user_visible_root_units_relocated,
            is_an_intent_logged_after_the_publish,
            is_the_region_of_the_live_intent_empty_by_the_records_after_the_publish:
                is_the_region_empty,
            region_forced_for_the_issue: ModelCompactionRegion {
                device: ModelDeviceIdentity(0),
                start_slot: 50_240,
                slot_count: CLUSTER_SEGMENT_SLOTS,
            },
        }
    }

    fn aspect_of_the_compaction_publish(
        model: &IdealModel,
        seen: &ModelCompactionPublishSeen,
    ) -> Result<ModelOperationKind, ModelDisagreementAspect> {
        model
            .answer_compaction_publish(seen)
            .map(|answer| answer.operation)
            .map_err(|disagreement| disagreement.aspect)
    }

    fn succeed_with_the_compaction_publish(
        model: &mut IdealModel,
        seen: &ModelCompactionPublishSeen,
    ) -> ModelRoot {
        let answer = model
            .answer_compaction_publish(seen)
            .expect("这一次整理发布合条款");
        succeed(model, &answer);
        model
            .current_version_in_the_session()
            .expect("会话开着")
            .clone()
    }

    /// 整理意图的三态（判决 G1、G4；D8（核心索引结构） 已定项 5）在模型里：没有活意图时只认签发（不搬、之后 logged_ops 里有一条：签发 txg、R、`c0`
    /// 是上一版的 txg、logged_ops 根的分配代 = 签发那次），签发之后再签发、搬空 R 而不删意图、删意图而 R 不空、不搬也不删都对不上；
    /// 照抄意图的一批、随最后一批删意图、单独删意图（R 已空：达成）、丢意图（R 不空）四种各认一次，删了之后这一版没有 logged_ops 根。
    #[test]
    fn the_model_takes_the_issue_first_and_lets_the_intent_leave_logged_ops_only_in_the_ways_the_clauses_allow(
    ) {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        assert_eq!(
            aspect_of_the_compaction_publish(
                &model,
                &compaction_publish_seen(2, true, false, Vec::new())
            ),
            Err(ModelDisagreementAspect::CompactionIntentOfThePublish),
            "没有活意图：先签发才搬"
        );
        assert_eq!(
            aspect_of_the_compaction_publish(
                &model,
                &compaction_publish_seen(0, false, false, Vec::new())
            ),
            Err(ModelDisagreementAspect::CompactionIntentOfThePublish),
            "没有活意图：签发之后 logged_ops 里要有它"
        );
        let issue = compaction_publish_seen(0, true, false, Vec::new());
        assert_eq!(
            aspect_of_the_compaction_publish(&model, &issue),
            Ok(ModelOperationKind::PublishCompactionIntentIssue)
        );
        let issued = succeed_with_the_compaction_publish(&mut model, &issue);
        let intent = ModelCompactionIntent {
            issued_txg: ModelCheckpointTxg(10),
            region: issue.region_forced_for_the_issue,
            allocation_generation_at_issue: ModelCheckpointTxg(9),
        };
        assert_eq!(
            issued.logged_compaction_intent,
            Some(intent),
            "签发 txg 10、c0 9"
        );
        assert_eq!(
            issued
                .role_written_at
                .get(&ModelUnitRole::LoggedOperationsRoot),
            Some(&ModelCheckpointTxg(10)),
            "logged_ops 根是签发那次写的"
        );
        assert_eq!(
            aspect_of_the_compaction_publish(&model, &issue),
            Err(ModelDisagreementAspect::CompactionIntentOfThePublish),
            "活意图在：不再签发（全池至多一条）"
        );
        assert_eq!(
            aspect_of_the_compaction_publish(
                &model,
                &compaction_publish_seen(2, true, true, Vec::new())
            ),
            Err(ModelDisagreementAspect::CompactionIntentOfThePublish),
            "搬空了 R 而意图照抄"
        );
        assert_eq!(
            aspect_of_the_compaction_publish(
                &model,
                &compaction_publish_seen(2, false, false, Vec::new())
            ),
            Err(ModelDisagreementAspect::CompactionIntentOfThePublish),
            "删意图而 R 不空"
        );
        let carried = succeed_with_the_compaction_publish(
            &mut model,
            &compaction_publish_seen(2, true, false, Vec::new()),
        );
        assert_eq!(
            carried.logged_compaction_intent,
            Some(intent),
            "照抄意图的一批"
        );
        assert_eq!(
            carried
                .role_written_at
                .get(&ModelUnitRole::LoggedOperationsRoot),
            Some(&ModelCheckpointTxg(10)),
            "logged_ops 根照抄"
        );
        let last_batch = succeed_with_the_compaction_publish(
            &mut model,
            &compaction_publish_seen(1, false, true, Vec::new()),
        );
        assert_eq!(
            last_batch.logged_compaction_intent, None,
            "最后一批同发布删意图"
        );
        assert!(
            !last_batch
                .role_written_at
                .contains_key(&ModelUnitRole::LoggedOperationsRoot),
            "删空之后没有 logged_ops 根"
        );
        succeed_with_the_compaction_publish(&mut model, &issue);
        assert_eq!(
            aspect_of_the_compaction_publish(
                &model,
                &compaction_publish_seen(0, true, false, Vec::new())
            ),
            Err(ModelDisagreementAspect::CompactionIntentOfThePublish),
            "活意图在：不搬也不删"
        );
        let completion = compaction_publish_seen(0, false, true, Vec::new());
        assert_eq!(
            aspect_of_the_compaction_publish(&model, &completion),
            Ok(ModelOperationKind::PublishCompactionIntentCompletion)
        );
        let completed = succeed_with_the_compaction_publish(&mut model, &completion);
        assert_eq!(
            completed.logged_compaction_intent, None,
            "R 已空：单独删意图"
        );
        succeed_with_the_compaction_publish(&mut model, &issue);
        let drop = compaction_publish_seen(0, false, false, Vec::new());
        assert_eq!(
            aspect_of_the_compaction_publish(&model, &drop),
            Ok(ModelOperationKind::PublishCompactionIntentDrop)
        );
        let dropped = succeed_with_the_compaction_publish(&mut model, &drop);
        assert_eq!(dropped.logged_compaction_intent, None, "R 不空：丢意图");
        assert!(
            !dropped
                .role_written_at
                .contains_key(&ModelUnitRole::LoggedOperationsRoot),
            "丢了之后没有 logged_ops 根"
        );
        assert_eq!(
            dropped.key.checkpoint_txg,
            ModelCheckpointTxg(16),
            "七次整理发布各一次 txg"
        );
    }

    /// 搬根算非空（用户 2026-10-15 定「照字节比，搬根算非空」）：签发与只搬别的单元的一批不换状态，搬了活头 inode 树根的那一批换了状态，
    /// 抬 F 的上限跟着从第 4 新的非空根 6 挪到 7（不算它时还是 6）；搬一个这一版不引用的根单元模型答不了。
    #[test]
    fn a_compaction_publish_relocating_a_user_visible_root_is_a_new_state_for_the_ceiling() {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        let state_of = |judged_model: &IdealModel| {
            judged_model
                .current_version_in_the_session()
                .expect("会话开着")
                .user_visible_state()
        };
        let before = state_of(&model);
        assert_eq!(
            model.rollback_floor_ceiling(),
            Some(ModelCheckpointTxg(6)),
            "非空 9、8、7、6、3：第 4 新的是 6"
        );
        succeed_with_the_compaction_publish(
            &mut model,
            &compaction_publish_seen(0, true, false, Vec::new()),
        );
        succeed_with_the_compaction_publish(
            &mut model,
            &compaction_publish_seen(2, true, false, Vec::new()),
        );
        assert_eq!(state_of(&model), before, "签发、只搬别的单元：状态不变");
        assert_eq!(model.rollback_floor_ceiling(), Some(ModelCheckpointTxg(6)));
        let inode_root_of_the_live_head = ModelUserVisibleRootUnit {
            tree: ModelUserVisibleTree::InodeTree,
            birth_txg: ModelCheckpointTxg(9),
        };
        let unreferenced_root = ModelUserVisibleRootUnit {
            tree: ModelUserVisibleTree::InodeTree,
            birth_txg: ModelCheckpointTxg(3),
        };
        assert_eq!(
            aspect_of_the_compaction_publish(
                &model,
                &compaction_publish_seen(1, true, false, vec![unreferenced_root])
            ),
            Err(ModelDisagreementAspect::NotModeled),
            "这一版不引用 txg 3 写的 inode 树根"
        );
        let relocated = succeed_with_the_compaction_publish(
            &mut model,
            &compaction_publish_seen(1, true, false, vec![inode_root_of_the_live_head]),
        );
        assert_eq!(
            relocated
                .user_visible_root_relocations
                .get(&inode_root_of_the_live_head),
            Some(&ModelCheckpointTxg(12)),
            "搬它的那一次记进去"
        );
        assert_ne!(state_of(&model), before, "搬了 inode 树根：非空");
        assert_eq!(
            model.rollback_floor_ceiling(),
            Some(ModelCheckpointTxg(7)),
            "非空 12、9、8、7：第 4 新的是 7"
        );
    }

    /// 一批至少搬一个（判决 `research/prompts/m4-segment-code-r2-main-verification.md` 第三节 H4，用户 2026-10-16 定「规划时补足」）：签发之后，
    /// 一个没搬、意图照抄、R 不空的那一次（计划里只有这次发布本来就重写的节点）对不上、模型不动；搬了一个的照抄意图的一批认；R 里只剩这次发布
    /// 本来就重写的节点时那一次不搬单元、同一次发布删意图、发布之后 R 按记录全空，认成意图达成：这一版没有 logged_ops 根，用户可见状态不变。
    #[test]
    fn a_compaction_publish_carrying_the_intent_relocates_at_least_one_unit_and_one_relocating_none_while_the_region_empties_deletes_the_intent(
    ) {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        let before = model
            .current_version_in_the_session()
            .expect("会话开着")
            .user_visible_state();
        succeed_with_the_compaction_publish(
            &mut model,
            &compaction_publish_seen(0, true, false, Vec::new()),
        );
        let after_the_issue = model.clone();
        assert_eq!(
            aspect_of_the_compaction_publish(
                &model,
                &compaction_publish_seen(0, true, false, Vec::new())
            ),
            Err(ModelDisagreementAspect::CompactionIntentOfThePublish),
            "一个没搬、意图照抄、R 不空：照抄意图的一批至少搬一个"
        );
        assert_eq!(model, after_the_issue, "判不合时模型不动");
        let carried = succeed_with_the_compaction_publish(
            &mut model,
            &compaction_publish_seen(1, true, false, Vec::new()),
        );
        assert_eq!(carried.key, key(11, 2), "搬了一个的照抄意图的一批");
        let only_nodes_the_publish_rewrites = compaction_publish_seen(0, false, true, Vec::new());
        assert_eq!(
            aspect_of_the_compaction_publish(&model, &only_nodes_the_publish_rewrites),
            Ok(ModelOperationKind::PublishCompactionIntentCompletion),
            "R 里只剩这次发布本来就重写的节点：不搬单元、同一次发布删意图，R 按记录全空是达成"
        );
        let completed =
            succeed_with_the_compaction_publish(&mut model, &only_nodes_the_publish_rewrites);
        assert_eq!(completed.key, key(12, 2));
        assert_eq!(completed.logged_compaction_intent, None, "意图删了");
        assert!(
            !completed
                .role_written_at
                .contains_key(&ModelUnitRole::LoggedOperationsRoot),
            "删空之后没有 logged_ops 根"
        );
        assert_eq!(
            completed.user_visible_state(),
            before,
            "整理不改用户可见状态"
        );
    }

    /// 回退那一版搬过的根只取目标那一版的（判决 `research/prompts/m4-segment-code-r2-main-verification.md` 第三节 H1）：签发（txg 10）之后
    /// 三批各搬一次活头 inode 树根（txg 11、12、13，每次一个新状态），回退到签发那一版 (10, 2) 写 (14, 2)。实现回退时树表根指针取目标那一版的字节，
    /// 新根与 (10, 2) 同一个状态、与它前一条 (13, 2) 不同：(14, 2) 不带「最后一次搬它」的记录，算非空；从新到旧数 14、13、12、11（9 与 14 同一个状态不计），
    /// 第 4 新的是 11，两块盘上最新的取小是 13 ⇒ 上限 11。把现行那一版的记录并进来时 14 与 13 同一个状态、不算非空，数成 13、12、11、9，上限 9。
    #[test]
    fn a_rollback_after_batches_relocated_a_root_the_target_references_carries_only_the_relocations_of_the_target(
    ) {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        succeed_with_the_compaction_publish(
            &mut model,
            &compaction_publish_seen(0, true, false, Vec::new()),
        );
        let target = model
            .current_version_in_the_session()
            .expect("会话开着")
            .clone();
        assert_eq!(target.key, key(10, 2), "签发那一版是回退目标");
        let inode_root_of_the_live_head = ModelUserVisibleRootUnit {
            tree: ModelUserVisibleTree::InodeTree,
            birth_txg: ModelCheckpointTxg(9),
        };
        for relocation_txg in 11_u64..=13 {
            let relocated = succeed_with_the_compaction_publish(
                &mut model,
                &compaction_publish_seen(1, true, false, vec![inode_root_of_the_live_head]),
            );
            assert_eq!(
                relocated.key,
                key(relocation_txg, 2),
                "照抄意图的一批各一次 txg"
            );
        }
        let before_the_rollback = model
            .current_version_in_the_session()
            .expect("会话开着")
            .clone();
        assert_eq!(
            before_the_rollback
                .user_visible_root_relocations
                .get(&inode_root_of_the_live_head),
            Some(&ModelCheckpointTxg(13)),
            "现行那一版最后一次搬 inode 树根是 txg 13"
        );
        let rollback = model
            .answer_rollback_while_mounted(key(10, 2))
            .expect("会话开着、现行版本带文件");
        assert!(rollback.required_refusals.is_empty(), "{rollback:?}");
        let rollback_root = rollback.expected_roots[0].clone();
        assert_eq!(rollback_root.key, key(14, 2));
        assert_eq!(
            rollback_root.user_visible_root_relocations, target.user_visible_root_relocations,
            "搬过的根只取目标那一版的（目标那一版之后才搬的不并进来）"
        );
        assert_eq!(
            rollback_root.user_visible_state(),
            target.user_visible_state(),
            "回退那一版与目标那一版同一个状态"
        );
        assert_ne!(
            rollback_root.user_visible_state(),
            before_the_rollback.user_visible_state(),
            "回退那一版与它前一条（搬了根的那一批）不同：非空"
        );
        succeed(&mut model, &rollback);
        assert_eq!(
            model.rollback_floor_ceiling(),
            Some(ModelCheckpointTxg(11)),
            "非空从新到旧 14、13、12、11：第 4 新的是 11"
        );
    }

    /// 回退牵涉的两版带快照就拒（判决第三节 S8、岔路 3）：现行那一版带着快照时要求拒成 `RollbackWithSnapshotsWhichTheFirstVersionRefuses`、
    /// 一条根都不写；删掉快照之后回退到建快照那一版（txg 10，树表里有那一对）同样要求拒；回退到建快照之前的 (6, 2) 不再因为快照拒。
    #[test]
    fn the_model_requires_a_rollback_from_or_to_a_version_carrying_a_snapshot_to_be_refused() {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        let creation = model.answer_create_snapshot().expect("建快照");
        succeed(&mut model, &creation);
        let from_the_snapshot = model
            .answer_rollback_while_mounted(key(6, 2))
            .expect("会话开着、现行版本带文件");
        assert_eq!(
            from_the_snapshot.required_refusals,
            BTreeSet::from([ModelRefusalReason::RollbackWithSnapshotsWhichTheFirstVersionRefuses]),
            "现行那一版带着快照：要求拒"
        );
        assert!(from_the_snapshot.expected_roots.is_empty(), "一条根都不写");
        let intent = model.answer_destroy_snapshot(0).expect("意图");
        succeed(&mut model, &intent);
        let final_batch = model.answer_next_step_of_the_walk().expect("最后一批");
        succeed(&mut model, &final_batch);
        let to_the_snapshot = model
            .answer_rollback_while_mounted(key(10, 2))
            .expect("会话开着、现行版本带文件");
        assert_eq!(
            to_the_snapshot.required_refusals,
            BTreeSet::from([ModelRefusalReason::RollbackWithSnapshotsWhichTheFirstVersionRefuses]),
            "目标那一版带着快照：要求拒"
        );
        let before_the_snapshot = model
            .answer_rollback_while_mounted(key(6, 2))
            .expect("会话开着、现行版本带文件");
        assert!(
            !before_the_snapshot
                .required_refusals
                .contains(&ModelRefusalReason::RollbackWithSnapshotsWhichTheFirstVersionRefuses),
            "两版都没有快照：不因为快照拒：{:?}",
            before_the_snapshot.required_refusals
        );
    }

    /// 没有可写会话时建快照答不了（会话状态对不上）；会话开着而现行那一版树表 0 条（mkfs 之后取号暖机、还没发第一个文件）同样答不了：
    /// 执行器只在有树的一版上调。
    #[test]
    fn the_model_does_not_answer_a_snapshot_creation_without_a_writable_session_or_on_a_version_without_trees(
    ) {
        let mut model = two_device_model();
        assert_eq!(
            model
                .answer_create_snapshot()
                .map_err(|disagreement| disagreement.aspect),
            Err(ModelDisagreementAspect::SessionState)
        );
        model.acquire_and_warm_up_in_the_make_filesystem_process();
        assert_eq!(
            model
                .answer_create_snapshot()
                .map_err(|disagreement| disagreement.aspect),
            Err(ModelDisagreementAspect::SessionState),
            "树表 0 条的一版"
        );
    }

    /// 文件集合为空时建快照照建（用户 2026-10-16 定「放开」，判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F5）：
    /// 第一个文件（txg 3）、unlink 意图与最后一批之后那一版有树而文件集合为空，建快照不要求拒、记的内容是「没有文件」；之后建一个 inode，
    /// 读快照读出「没有文件」，读出一个文件对不上。
    #[test]
    fn a_snapshot_created_on_a_version_with_trees_and_no_file_reads_no_file_after_an_inode_is_created(
    ) {
        let mut model = two_device_model();
        model.acquire_and_warm_up_in_the_make_filesystem_process();
        let first = model
            .answer_publish_first_file(&[3])
            .expect("会话开着、现行 txg 2");
        succeed(&mut model, &first);
        let intent = model
            .answer_unlink_intent()
            .expect("会话开着、现行版本带活文件");
        succeed(&mut model, &intent);
        let final_batch = model
            .answer_deletion_step()
            .expect("会话开着、现行版本删除中");
        succeed(&mut model, &final_batch);
        let creation = model
            .answer_create_snapshot()
            .expect("会话开着、现行那一版有树");
        assert!(
            creation.required_refusals.is_empty(),
            "有树而文件集合为空时照建：{:?}",
            creation.required_refusals
        );
        succeed(&mut model, &creation);
        assert_eq!(
            current_snapshots(&model)
                .iter()
                .map(|snapshot| snapshot.content_at_creation.clone())
                .collect::<Vec<_>>(),
            vec![None],
            "快照记的是建它那一刻的「没有文件」"
        );
        let created = model
            .answer_create_inode()
            .expect("会话开着、第一个文件已删完");
        succeed(&mut model, &created);
        assert_eq!(
            model
                .judge_snapshot_read_back(0, &ObservedSnapshotReadBack::NoFile)
                .map(|counts| counts.snapshot_read_backs_compared)
                .map_err(|disagreement| disagreement.aspect),
            Ok(1),
            "读快照读出「没有文件」"
        );
        assert_eq!(
            model
                .judge_snapshot_read_back(0, &ObservedSnapshotReadBack::File(vec![3]))
                .map_err(|disagreement| disagreement.aspect),
            Err(ModelDisagreementAspect::SnapshotReadBack),
            "读出一个文件对不上"
        );
    }

    /// 快照个数相同、活头相同、快照抄的根不同的两版，用户可见状态（「非空」与「4 个不同状态」比的那一样）要分得出（判决
    /// `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F7，攻方 T11 那条用例）：建快照（抄活头 txg 9 的根）、
    /// 覆盖写成活头 txg 11——这一版记为删之前；删掉那个快照（意图与最后一批）、再建一个（抄活头 txg 11 的根）——这一版记为重建之后。
    /// 两版的活头都是 txg 11 写的、都只有一个快照、都没搬过根，只差快照抄的根。
    #[test]
    fn two_versions_with_one_snapshot_each_differ_in_the_user_visible_state_when_the_snapshots_copy_different_roots(
    ) {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        let creation = model.answer_create_snapshot().expect("建快照");
        succeed(&mut model, &creation);
        let overwrite = model.answer_publish_overwrite(&[42]).expect("覆盖写");
        succeed(&mut model, &overwrite);
        let before_the_destruction = model
            .current_version_in_the_session()
            .expect("会话开着")
            .user_visible_state();
        let intent = model.answer_destroy_snapshot(0).expect("意图");
        succeed(&mut model, &intent);
        let final_batch = model.answer_next_step_of_the_walk().expect("最后一批");
        succeed(&mut model, &final_batch);
        let recreation = model.answer_create_snapshot().expect("再建快照");
        succeed(&mut model, &recreation);
        let after_the_recreation = model
            .current_version_in_the_session()
            .expect("会话开着")
            .user_visible_state();
        assert_eq!(
            (
                &before_the_destruction.trees_written_by,
                &before_the_destruction.root_relocations,
            ),
            (
                &after_the_recreation.trees_written_by,
                &after_the_recreation.root_relocations,
            ),
            "两版的活头相同、快照个数相同（根单元逐个都没搬过）"
        );
        assert_ne!(
            before_the_destruction, after_the_recreation,
            "快照抄的根不同：用户可见状态不同"
        );
    }

    /// extent 树下段的位置（D8（核心索引结构） 已定项 14）：一个单元不建下段；145 个单元两片叶加一个根；三片叶的那一档三片加根。
    #[test]
    fn extent_lower_node_positions_follow_the_data_unit_count() {
        assert!(extent_lower_node_positions(1).is_empty());
        let leaf = |index_in_level| ModelExtentLowerNodePosition {
            level: 0,
            index_in_level,
        };
        let root = ModelExtentLowerNodePosition {
            level: 1,
            index_in_level: 0,
        };
        assert_eq!(extent_lower_node_positions(2), vec![leaf(0)]);
        assert_eq!(
            extent_lower_node_positions(EXTENT_TREE_LOWER_LEAF_DATA_UNITS + 1),
            vec![leaf(0), leaf(1), root]
        );
        assert_eq!(
            extent_lower_node_positions(2 * EXTENT_TREE_LOWER_LEAF_DATA_UNITS + 4),
            vec![leaf(0), leaf(1), leaf(2), root]
        );
        assert_eq!(
            u64::try_from(
                extent_lower_node_positions(2 * EXTENT_TREE_LOWER_LEAF_DATA_UNITS + 4).len()
            )
            .expect("个数"),
            extent_lower_segment_nodes(2 * EXTENT_TREE_LOWER_LEAF_DATA_UNITS + 4),
            "与按层数个数的那一份式子数出来的相同"
        );
    }

    /// 一个数据单元装多少字节的用户内容（净荷）。
    fn payload() -> usize {
        usize::try_from(data_unit_payload_capacity_in_bytes()).expect("三万多")
    }

    /// `length` 字节、没有一个字节是 0 的内容（补的 0 与洞读出的 0 看得出来）。
    fn content_without_zeros(length: usize) -> Vec<u8> {
        (0..length)
            .map(|index| u8::try_from(index % 251).expect("小于 251") + 1)
            .collect()
    }

    /// 实例 2 覆盖写到 txg 9 之后，顺序写一个 `length` 字节的文件（txg 10）。
    fn model_with_a_file_of(length: usize) -> IdealModel {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        let write = model
            .answer_publish_sequential_write(&content_without_zeros(length))
            .expect("会话开着、现行版本带文件");
        succeed(&mut model, &write);
        model
    }

    fn current_version(model: &IdealModel) -> ModelRoot {
        model
            .current_version_in_the_session()
            .expect("会话开着")
            .clone()
    }

    /// 截到 `new_size` 字节（缩短是意图那次，不缩短是那一次只改记录的发布），做成，交回写出的那一版。
    fn truncate_to(model: &mut IdealModel, new_size: usize) -> ModelRoot {
        let answer = model
            .answer_truncation(u64::try_from(new_size).expect("长度"))
            .expect("会话开着、现行版本带活文件");
        assert_eq!(answer.operation, ModelOperationKind::PublishTruncation);
        assert_eq!(answer.expected_roots.len(), 1, "截断这次用户改动一次发布");
        succeed(model, &answer);
        current_version(model)
    }

    /// 走空的下一步，做成，交回写出的那一版。
    fn walk_one_step(model: &mut IdealModel) -> ModelRoot {
        let answer = model
            .answer_next_step_of_the_walk()
            .expect("现行版本有要走空的");
        succeed(model, &answer);
        current_version(model)
    }

    /// 这一版里分配代等于它自己那次发布的 txg 的角色（这次重写的），数据单元有一个是这次写的就算上 `Data`。
    fn roles_rewritten_by_the_publish(root: &ModelRoot) -> BTreeSet<ModelUnitRole> {
        root.role_written_at
            .iter()
            .filter(|(_, written_at)| **written_at == root.key.checkpoint_txg)
            .map(|(role, _)| *role)
            .chain(
                root.data_unit_written_at
                    .contains(&root.key.checkpoint_txg)
                    .then_some(ModelUnitRole::Data),
            )
            .collect()
    }

    fn file_of(root: &ModelRoot) -> &ModelFileVersion {
        root.contents.live_file().expect("这一版读得出文件")
    }

    /// 每种截断发布重写哪些角色、删掉哪些角色，钉绝对值（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K1、K2、K3、K5、K8）：
    /// 意图写 inode 叶与根、logged_ops 根，重写尾单元的另写数据单元与 extent 上段根；批不碰 inode 记录；最后一批放了单元的写 extent 上段根、
    /// 删空 logged_ops 的删掉它的根、还装着整理意图的重写它；不缩短的写 inode 叶与根，补零重写尾单元的另写数据单元与上段根、只换洞标签的只另写上段根。
    #[test]
    fn every_truncation_publish_kind_rewrites_and_removes_exactly_the_roles_the_clauses_name() {
        let emptied = ModelLoggedOperationsAfterTheTruncationEntryIsDeleted::Emptied;
        let still_holds =
            ModelLoggedOperationsAfterTheTruncationEntryIsDeleted::StillHoldsTheCompactionIntent;
        let cases: [(ModelPublishKind, Vec<ModelUnitRole>, Vec<ModelUnitRole>); 11] = [
            (
                ModelPublishKind::TruncationIntent {
                    extent_tree: ModelExtentTreeChangeOfASizeChange::TailDataUnitRewritten,
                },
                vec![
                    ModelUnitRole::Data,
                    ModelUnitRole::ExtentRoot,
                    ModelUnitRole::InodeLeaf,
                    ModelUnitRole::InodeRoot,
                    ModelUnitRole::AllocationTree,
                    ModelUnitRole::AccountingTree,
                    ModelUnitRole::LoggedOperationsRoot,
                    ModelUnitRole::MappingTree,
                    ModelUnitRole::TreeTable,
                ],
                vec![],
            ),
            (
                ModelPublishKind::TruncationIntent {
                    extent_tree: ModelExtentTreeChangeOfASizeChange::OnlyTheUpperLeafEntryLabel,
                },
                vec![
                    ModelUnitRole::ExtentRoot,
                    ModelUnitRole::InodeLeaf,
                    ModelUnitRole::InodeRoot,
                    ModelUnitRole::AllocationTree,
                    ModelUnitRole::AccountingTree,
                    ModelUnitRole::LoggedOperationsRoot,
                    ModelUnitRole::MappingTree,
                    ModelUnitRole::TreeTable,
                ],
                vec![],
            ),
            (
                ModelPublishKind::TruncationIntent {
                    extent_tree: ModelExtentTreeChangeOfASizeChange::Untouched,
                },
                vec![
                    ModelUnitRole::InodeLeaf,
                    ModelUnitRole::InodeRoot,
                    ModelUnitRole::AllocationTree,
                    ModelUnitRole::AccountingTree,
                    ModelUnitRole::LoggedOperationsRoot,
                    ModelUnitRole::MappingTree,
                    ModelUnitRole::TreeTable,
                ],
                vec![],
            ),
            (
                ModelPublishKind::TruncationBatch,
                vec![
                    ModelUnitRole::ExtentRoot,
                    ModelUnitRole::AllocationTree,
                    ModelUnitRole::AccountingTree,
                    ModelUnitRole::MappingTree,
                    ModelUnitRole::TreeTable,
                ],
                vec![],
            ),
            (
                ModelPublishKind::FinalTruncationBatch {
                    release: ModelFinalTruncationBatchRelease::AtLeastOneDataUnit,
                    logged_operations_after: emptied,
                },
                vec![
                    ModelUnitRole::ExtentRoot,
                    ModelUnitRole::AllocationTree,
                    ModelUnitRole::AccountingTree,
                    ModelUnitRole::MappingTree,
                    ModelUnitRole::TreeTable,
                ],
                vec![ModelUnitRole::LoggedOperationsRoot],
            ),
            (
                ModelPublishKind::FinalTruncationBatch {
                    release: ModelFinalTruncationBatchRelease::AtLeastOneDataUnit,
                    logged_operations_after: still_holds,
                },
                vec![
                    ModelUnitRole::ExtentRoot,
                    ModelUnitRole::AllocationTree,
                    ModelUnitRole::AccountingTree,
                    ModelUnitRole::LoggedOperationsRoot,
                    ModelUnitRole::MappingTree,
                    ModelUnitRole::TreeTable,
                ],
                vec![],
            ),
            (
                ModelPublishKind::FinalTruncationBatch {
                    release: ModelFinalTruncationBatchRelease::NoDataUnit,
                    logged_operations_after: emptied,
                },
                vec![
                    ModelUnitRole::AllocationTree,
                    ModelUnitRole::AccountingTree,
                    ModelUnitRole::MappingTree,
                    ModelUnitRole::TreeTable,
                ],
                vec![ModelUnitRole::LoggedOperationsRoot],
            ),
            (
                ModelPublishKind::FinalTruncationBatch {
                    release: ModelFinalTruncationBatchRelease::NoDataUnit,
                    logged_operations_after: still_holds,
                },
                vec![
                    ModelUnitRole::AllocationTree,
                    ModelUnitRole::AccountingTree,
                    ModelUnitRole::LoggedOperationsRoot,
                    ModelUnitRole::MappingTree,
                    ModelUnitRole::TreeTable,
                ],
                vec![],
            ),
            (
                ModelPublishKind::SizeChangeWithoutShrinking {
                    extent_tree: ModelExtentTreeChangeOfASizeChange::TailDataUnitRewritten,
                },
                vec![
                    ModelUnitRole::Data,
                    ModelUnitRole::ExtentRoot,
                    ModelUnitRole::InodeLeaf,
                    ModelUnitRole::InodeRoot,
                    ModelUnitRole::AllocationTree,
                    ModelUnitRole::AccountingTree,
                    ModelUnitRole::MappingTree,
                    ModelUnitRole::TreeTable,
                ],
                vec![],
            ),
            (
                ModelPublishKind::SizeChangeWithoutShrinking {
                    extent_tree: ModelExtentTreeChangeOfASizeChange::OnlyTheUpperLeafEntryLabel,
                },
                vec![
                    ModelUnitRole::ExtentRoot,
                    ModelUnitRole::InodeLeaf,
                    ModelUnitRole::InodeRoot,
                    ModelUnitRole::AllocationTree,
                    ModelUnitRole::AccountingTree,
                    ModelUnitRole::MappingTree,
                    ModelUnitRole::TreeTable,
                ],
                vec![],
            ),
            (
                ModelPublishKind::SizeChangeWithoutShrinking {
                    extent_tree: ModelExtentTreeChangeOfASizeChange::Untouched,
                },
                vec![
                    ModelUnitRole::InodeLeaf,
                    ModelUnitRole::InodeRoot,
                    ModelUnitRole::AllocationTree,
                    ModelUnitRole::AccountingTree,
                    ModelUnitRole::MappingTree,
                    ModelUnitRole::TreeTable,
                ],
                vec![],
            ),
        ];
        for (kind, rewritten, removed) in cases {
            assert_eq!(kind.rewritten_roles(), rewritten, "{kind:?} 重写的角色");
            assert_eq!(
                kind.removed_roles().to_vec(),
                removed,
                "{kind:?} 删掉的角色"
            );
            assert_eq!(kind.transactions_carried(0), 1, "{kind:?} 一个事务");
            assert!(!kind.rewrites_the_instance_table(), "{kind:?} 不重写实例表");
            assert!(
                !kind.leaves_an_inode_number_watermark_step(),
                "{kind:?} 不发 inode 号"
            );
        }
    }

    /// 上段叶条目的标签（判决第三节 K5，第六节 S1 收成唯一）：一个单元内联、多个单元下段根，存在的少于 size 罩的就是尾部有洞（标签 3、4、0）；
    /// 标签 1、2 不许洞，标签 3、4、0 至少一个洞单元，截断途中 size 之外可以多挂着，那时只许 1、2（size 之外挂着的不算洞单元）。
    #[test]
    fn the_upper_leaf_entry_label_declares_the_tail_hole_with_tags_three_four_and_zero_only() {
        let cases = [
            (0, 1, ModelExtentUpperLeafEntryLabel::NoDataUnit, 0),
            (1, 1, ModelExtentUpperLeafEntryLabel::InlineDataUnit, 2),
            (
                1,
                3,
                ModelExtentUpperLeafEntryLabel::InlineDataUnitThenTailHole,
                4,
            ),
            (2, 2, ModelExtentUpperLeafEntryLabel::LowerSegmentRoot, 1),
            (
                2,
                5,
                ModelExtentUpperLeafEntryLabel::LowerSegmentRootThenTailHole,
                3,
            ),
            (300, 1, ModelExtentUpperLeafEntryLabel::LowerSegmentRoot, 1),
        ];
        for (present, within_the_size, label, tag) in cases {
            assert_eq!(
                ModelExtentUpperLeafEntryLabel::written_for(present, within_the_size),
                label,
                "存在 {present} 个、size 罩 {within_the_size} 个"
            );
            assert_eq!(label.tag(), tag, "{label:?} 的盘上标签值");
        }
        assert!(ModelExtentUpperLeafEntryLabel::InlineDataUnit.admits(1, 1, false));
        assert!(!ModelExtentUpperLeafEntryLabel::InlineDataUnit.admits(1, 2, false));
        assert!(!ModelExtentUpperLeafEntryLabel::InlineDataUnitThenTailHole.admits(1, 1, false));
        assert!(!ModelExtentUpperLeafEntryLabel::LowerSegmentRoot.admits(3, 2, false));
        assert!(ModelExtentUpperLeafEntryLabel::LowerSegmentRoot.admits(3, 2, true));
        assert!(
            !ModelExtentUpperLeafEntryLabel::LowerSegmentRootThenTailHole.admits(3, 2, true),
            "截断途中 size 之外挂着的不算洞单元：标签 3 没有洞，不许（判决第六节 S1）"
        );
        assert!(ModelExtentUpperLeafEntryLabel::NoDataUnit.admits(0, 4, false));
    }

    /// 截到 0（判决第三节 K2、K3、K11）：300 个单元的文件，意图那次 size 写 0、单元 0 重写成声明长度 0、一个都不放；之后 299 个挂在 size 之外，
    /// 两批各从尾部放 144 个（不碰 inode 记录），最后一批放剩下的 11 个、删截断条目（logged_ops 删空），留一个单元、内联。
    #[test]
    fn truncating_to_zero_issues_the_intent_then_two_batches_of_144_then_the_final_batch_leaving_one_inline_unit(
    ) {
        let mut model = model_with_a_file_of(300 * payload() - 5);
        let intent = truncate_to(&mut model, 0);
        let ModelVersionContents::FileBeingTruncated {
            truncation_issued_txg,
            ..
        } = &intent.contents
        else {
            panic!("意图之后在截断途中：{:?}", intent.contents);
        };
        assert_eq!(*truncation_issued_txg, intent.key.checkpoint_txg);
        assert_eq!(file_of(&intent).content.len(), 0, "先落目标：读回 0 字节");
        assert_eq!(file_of(&intent).data_units_present, 300);
        assert_eq!(
            file_of(&intent).extent_upper_leaf_entry_label,
            ModelExtentUpperLeafEntryLabel::LowerSegmentRoot
        );
        assert_eq!(
            roles_rewritten_by_the_publish(&intent),
            BTreeSet::from([
                ModelUnitRole::Data,
                ModelUnitRole::ExtentRoot,
                ModelUnitRole::InodeLeaf,
                ModelUnitRole::InodeRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::LoggedOperationsRoot,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ])
        );
        assert_eq!(
            intent.data_unit_written_at[0], intent.key.checkpoint_txg,
            "单元 0 重写成声明长度 0"
        );
        assert_eq!(intent.data_unit_written_at[1], ModelCheckpointTxg(10));
        assert!(
            model.current_version_has_a_step_to_walk(),
            "截断进共用走空（K1）：下一次用户改动之前先把各批走完"
        );
        let batch_rewritten = BTreeSet::from([
            ModelUnitRole::ExtentRoot,
            ModelUnitRole::AllocationTree,
            ModelUnitRole::AccountingTree,
            ModelUnitRole::MappingTree,
            ModelUnitRole::TreeTable,
        ]);
        let first_batch = walk_one_step(&mut model);
        assert_eq!(file_of(&first_batch).data_units_present, 156);
        assert_eq!(
            roles_rewritten_by_the_publish(&first_batch),
            batch_rewritten
        );
        assert!(model.current_version_has_a_file_being_truncated());
        let second_batch = walk_one_step(&mut model);
        assert_eq!(file_of(&second_batch).data_units_present, 12);
        let final_batch = walk_one_step(&mut model);
        let ModelVersionContents::LiveFile(file) = &final_batch.contents else {
            panic!("最后一批之后是带活文件的一版：{:?}", final_batch.contents);
        };
        assert_eq!((file.content.len(), file.data_units_present), (0, 1));
        assert_eq!(
            file.extent_upper_leaf_entry_label,
            ModelExtentUpperLeafEntryLabel::InlineDataUnit
        );
        assert_eq!(
            roles_rewritten_by_the_publish(&final_batch),
            batch_rewritten
        );
        assert!(
            !final_batch
                .role_written_at
                .contains_key(&ModelUnitRole::LoggedOperationsRoot),
            "截断条目删掉、logged_ops 删空"
        );
        assert!(
            final_batch.extent_lower_node_written_at.is_empty(),
            "一个单元不建下段"
        );
        assert_eq!(
            final_batch.data_unit_written_at.as_slice(),
            [intent.key.checkpoint_txg]
        );
        assert!(!model.current_version_has_a_step_to_walk());
    }

    /// 截到单元中间（判决第三节 K2、K3）：三个单元的文件截到 1.5 个净荷，意图那次重写单元 1（声明长度半个净荷）、不放单元；最后一批放单元 2、
    /// 不碰 inode 记录。之后建快照，码 14 抄的 inode 树根是意图那次写的（批只写 extent 树），不是最后一批。
    #[test]
    fn truncating_into_the_middle_of_a_unit_rewrites_that_unit_at_the_intent_and_the_final_batch_leaves_the_inode_tree_of_the_intent(
    ) {
        let mut model = model_with_a_file_of(2 * payload() + payload() / 2);
        let new_size = payload() + payload() / 2;
        let intent = truncate_to(&mut model, new_size);
        assert_eq!(
            file_of(&intent).content.as_ref(),
            &content_without_zeros(new_size)[..],
            "先落目标：读回截后那 N 字节"
        );
        assert_eq!(
            intent.data_unit_written_at.as_slice(),
            [
                ModelCheckpointTxg(10),
                intent.key.checkpoint_txg,
                ModelCheckpointTxg(10)
            ],
            "只重写单元 1，单元 2 还挂在 size 之外"
        );
        let final_batch = walk_one_step(&mut model);
        let file = file_of(&final_batch);
        assert_eq!(file.data_units_present, 2);
        assert_eq!(
            file.extent_upper_leaf_entry_label,
            ModelExtentUpperLeafEntryLabel::LowerSegmentRoot
        );
        assert_eq!(file.written_by, final_batch.key, "最后一批写了 extent 树");
        assert_eq!(
            file.inode_record_written_by, intent.key,
            "最后一批不碰 inode 记录"
        );
        assert_eq!(
            final_batch.role_written_at.get(&ModelUnitRole::InodeRoot),
            Some(&intent.key.checkpoint_txg)
        );
        assert!(!final_batch
            .role_written_at
            .contains_key(&ModelUnitRole::LoggedOperationsRoot));
        let snapshot = model.answer_create_snapshot().expect("会话开着、有树");
        succeed(&mut model, &snapshot);
        let snapshots = current_snapshots(&model);
        assert_eq!(
            snapshots[0].inode_tree_written_by, intent.key,
            "码 14 抄的 inode 树根是意图那次写的"
        );
    }

    /// 截到留下 144 个单元（判决第三节 K3「把罩着被重写单元 k 的那片下段叶到根恒并进重写集」，攻方 W3 打中 2 那一形）：150 个单元截到单元 143 中间，
    /// 意图那次重写单元 143 与罩着它的叶 0、下段根；最后一批放单元 144–149，留下 144 个、下段只剩叶 0，它的分配代是意图那次的（罩着重写的单元 143）。
    #[test]
    fn truncating_to_leave_144_units_keeps_the_leaf_the_intent_rewrote_for_the_tail_unit() {
        let mut model = model_with_a_file_of(150 * payload());
        let new_size = 143 * payload() + 7;
        let intent = truncate_to(&mut model, new_size);
        let leaf = |index_in_level| ModelExtentLowerNodePosition {
            level: 0,
            index_in_level,
        };
        let lower_root = ModelExtentLowerNodePosition {
            level: 1,
            index_in_level: 0,
        };
        assert_eq!(
            intent.extent_lower_node_written_at.as_ref(),
            &BTreeMap::from([
                (leaf(0), intent.key.checkpoint_txg),
                (leaf(1), ModelCheckpointTxg(10)),
                (lower_root, intent.key.checkpoint_txg),
            ]),
            "意图那次重写罩着单元 143 的叶 0 与下段根"
        );
        let final_batch = walk_one_step(&mut model);
        assert_eq!(file_of(&final_batch).data_units_present, 144);
        assert_eq!(
            final_batch.extent_lower_node_written_at.as_ref(),
            &BTreeMap::from([(leaf(0), intent.key.checkpoint_txg)]),
            "留下的叶 0 照抄意图那次写的"
        );
        assert_eq!(
            final_batch.data_unit_written_at[143],
            intent.key.checkpoint_txg
        );
        assert_eq!(
            final_batch.data_unit_written_at[142],
            ModelCheckpointTxg(10)
        );
    }

    /// 变长到单元内（判决第三节 K5）：100 字节的文件变长到 200 字节，一次发布做完：单元 0 补零重写到 200、标签照旧内联。
    #[test]
    fn growing_within_the_unit_rewrites_it_zero_padded_in_one_publish() {
        let mut model = model_with_a_file_of(100);
        let grown = truncate_to(&mut model, 200);
        let ModelVersionContents::LiveFile(file) = &grown.contents else {
            panic!("不缩短的截断一次发布做完：{:?}", grown.contents);
        };
        let mut expected = content_without_zeros(100);
        expected.resize(200, 0);
        assert_eq!(file.content.as_ref(), &expected[..]);
        assert_eq!(
            (file.data_units_present, file.extent_upper_leaf_entry_label),
            (1, ModelExtentUpperLeafEntryLabel::InlineDataUnit)
        );
        assert_eq!(
            roles_rewritten_by_the_publish(&grown),
            BTreeSet::from([
                ModelUnitRole::Data,
                ModelUnitRole::ExtentRoot,
                ModelUnitRole::InodeLeaf,
                ModelUnitRole::InodeRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ])
        );
        assert!(!model.current_version_has_a_step_to_walk(), "变长不立意图");
    }

    /// 变长跨单元（判决第三节 K5）：100 字节的文件变长到两个半净荷：单元 0 补零重写到满、之后到 size 是洞（标签 4），不写新单元。
    #[test]
    fn growing_across_units_fills_the_last_unit_and_declares_the_tail_hole() {
        let mut model = model_with_a_file_of(100);
        let new_size = 2 * payload() + 5;
        let grown = truncate_to(&mut model, new_size);
        let file = file_of(&grown);
        assert_eq!(file.content.len(), new_size);
        assert!(file.content[100..].iter().all(|byte| *byte == 0), "洞读 0");
        assert_eq!(
            (file.data_units_present, file.extent_upper_leaf_entry_label),
            (
                1,
                ModelExtentUpperLeafEntryLabel::InlineDataUnitThenTailHole
            )
        );
        assert_eq!(
            grown.data_unit_written_at.as_slice(),
            [grown.key.checkpoint_txg]
        );
        assert!(roles_rewritten_by_the_publish(&grown).contains(&ModelUnitRole::ExtentRoot));
    }

    /// 最后一个单元已满的文件变长（判决第三节 K5）：两个满单元变长一个字节，单元不动，只把上段叶条目的标签 1 换成 3（只重写 extent 上段根）。
    #[test]
    fn growing_a_file_whose_last_unit_is_full_only_relabels_the_upper_leaf_entry() {
        let mut model = model_with_a_file_of(2 * payload());
        let grown = truncate_to(&mut model, 2 * payload() + 1);
        assert_eq!(
            file_of(&grown).extent_upper_leaf_entry_label,
            ModelExtentUpperLeafEntryLabel::LowerSegmentRootThenTailHole
        );
        assert_eq!(
            roles_rewritten_by_the_publish(&grown),
            BTreeSet::from([
                ModelUnitRole::ExtentRoot,
                ModelUnitRole::InodeLeaf,
                ModelUnitRole::InodeRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ])
        );
        assert_eq!(
            grown.data_unit_written_at.as_slice(),
            [ModelCheckpointTxg(10), ModelCheckpointTxg(10)]
        );
    }

    /// 变长后截回洞里（判决第三节 K5、K11）：100 字节变长到 5 个净荷（单元 0 补满、之后是洞），再截到 3 个净荷多 7 字节——落在洞里：
    /// 意图那次不重写单元、extent 树不动；最后一批没有单元可放（照走单一路径），也不动 extent 树，只删截断条目；标签仍是 4。
    #[test]
    fn truncating_back_into_the_hole_after_growing_issues_the_intent_and_a_final_batch_releasing_nothing(
    ) {
        let mut model = model_with_a_file_of(100);
        let grown = truncate_to(&mut model, 5 * payload());
        let new_size = 3 * payload() + 7;
        let intent = truncate_to(&mut model, new_size);
        assert_eq!(
            roles_rewritten_by_the_publish(&intent),
            BTreeSet::from([
                ModelUnitRole::InodeLeaf,
                ModelUnitRole::InodeRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::LoggedOperationsRoot,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ])
        );
        assert!(
            model.current_version_has_a_file_being_truncated(),
            "缩短一律立意图"
        );
        let final_batch = walk_one_step(&mut model);
        assert_eq!(
            roles_rewritten_by_the_publish(&final_batch),
            BTreeSet::from([
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ])
        );
        let file = file_of(&final_batch);
        assert_eq!(file.content.as_ref(), &file_of(&grown).content[..new_size]);
        assert_eq!(
            (file.data_units_present, file.extent_upper_leaf_entry_label),
            (
                1,
                ModelExtentUpperLeafEntryLabel::InlineDataUnitThenTailHole
            )
        );
        assert_eq!(file.written_by, intent.key, "最后一批没写用户可见树");
    }

    /// 缩短把洞整个截掉、而尾单元不用重写、最后一批没有单元可放（变长到三个净荷之后截到恰好一个净荷，判决第六节 S1）：换标签写在意图那次——
    /// 意图那次只为换标签重写 extent 上段根（单元不写），标签 4 换回 2；最后一批不动 extent 树、只删截断条目，标签照抄 2。
    #[test]
    fn cutting_the_whole_hole_at_the_end_of_the_last_unit_relabels_the_upper_leaf_entry_at_the_intent(
    ) {
        let mut model = model_with_a_file_of(100);
        let grown = truncate_to(&mut model, 3 * payload());
        assert_eq!(
            file_of(&grown).extent_upper_leaf_entry_label,
            ModelExtentUpperLeafEntryLabel::InlineDataUnitThenTailHole
        );
        let intent = truncate_to(&mut model, payload());
        assert_eq!(
            roles_rewritten_by_the_publish(&intent),
            BTreeSet::from([
                ModelUnitRole::ExtentRoot,
                ModelUnitRole::InodeLeaf,
                ModelUnitRole::InodeRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::LoggedOperationsRoot,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ]),
            "意图那次为换标签重写上段根、不写数据单元"
        );
        assert_eq!(
            file_of(&intent).extent_upper_leaf_entry_label,
            ModelExtentUpperLeafEntryLabel::InlineDataUnit,
            "标签在意图那次换回 2"
        );
        let final_batch = walk_one_step(&mut model);
        assert!(!roles_rewritten_by_the_publish(&final_batch).contains(&ModelUnitRole::ExtentRoot));
        assert_eq!(
            (
                file_of(&final_batch).data_units_present,
                file_of(&final_batch).extent_upper_leaf_entry_label
            ),
            (1, ModelExtentUpperLeafEntryLabel::InlineDataUnit)
        );
    }

    /// 三个满单元的文件变长到五个净荷（标签 3），再截到恰好一个净荷（判决第六节 S1）：尾单元 0 不用重写，size 之外挂着单元 1、2，洞整个截掉——
    /// 意图那次换标签（3 → 1：size 之外挂着的不算洞单元），之后最后一批放掉两个单元、单元只剩一个，标签照写成 2。
    #[test]
    fn cutting_the_whole_hole_and_two_data_units_relabels_at_the_intent_and_the_final_batch_inlines_the_last_unit(
    ) {
        let mut model = model_with_a_file_of(3 * payload());
        truncate_to(&mut model, 5 * payload());
        let intent = truncate_to(&mut model, payload());
        assert!(
            roles_rewritten_by_the_publish(&intent).contains(&ModelUnitRole::ExtentRoot)
                && !roles_rewritten_by_the_publish(&intent).contains(&ModelUnitRole::Data),
            "意图那次只为换标签重写上段根：{:?}",
            roles_rewritten_by_the_publish(&intent)
        );
        assert_eq!(
            (
                file_of(&intent).data_units_present,
                file_of(&intent).extent_upper_leaf_entry_label
            ),
            (3, ModelExtentUpperLeafEntryLabel::LowerSegmentRoot)
        );
        let final_batch = walk_one_step(&mut model);
        assert_eq!(
            (
                file_of(&final_batch).data_units_present,
                file_of(&final_batch).extent_upper_leaf_entry_label
            ),
            (1, ModelExtentUpperLeafEntryLabel::InlineDataUnit)
        );
    }

    /// 第一个文件删完之后建的 inode 改 size（判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S2、S3、S6：
    /// 删了「第一个文件之外改 size」那一拒）：N 按整单元向上取整溢出 u64 的先按溢出拒、一条根都不写；变长到 size > 0 发一次只改记录的发布、
    /// 这次起写粘住的标签 0（extent 上段根重写、上段有条目的号是 {2}）、读回 N 个 0；再变长只改记录、上段整棵照抄；截短发截断意图（logged_ops 根
    /// 重写、这一版在截断途中），走空的下一步是放 0 个单元的最后一批（删掉截断条目），标签 0 那一条照旧留着。
    #[test]
    fn a_size_change_of_an_inode_created_after_the_first_file_was_deleted_keeps_a_tag_zero_entry_and_truncates_through_an_intent(
    ) {
        let mut model = model_with_a_file_of(100);
        let unlink = model
            .answer_unlink_intent()
            .expect("会话开着、现行版本带活文件");
        succeed(&mut model, &unlink);
        let final_batch = model.answer_next_step_of_the_walk().expect("删除中");
        succeed(&mut model, &final_batch);
        let create = model.answer_create_inode().expect("删完之后建 inode");
        succeed(&mut model, &create);
        let overflow = model
            .answer_size_change_of_an_inode_other_than_the_first(2, u64::MAX)
            .expect("现行版本是删完之后建过 inode 的一版");
        assert_eq!(
            overflow.required_refusals,
            BTreeSet::from([ModelRefusalReason::TruncationSizeWhoseWholeDataUnitsOverflow])
        );
        assert!(overflow.expected_roots.is_empty());
        let extension = model
            .answer_size_change_of_an_inode_other_than_the_first(2, 7)
            .expect("现行版本是删完之后建过 inode 的一版");
        succeed(&mut model, &extension);
        let extended = current_version(&model);
        assert!(
            roles_rewritten_by_the_publish(&extended).contains(&ModelUnitRole::ExtentRoot),
            "第一次写上标签 0：extent 上段根重写：{:?}",
            roles_rewritten_by_the_publish(&extended)
        );
        assert_eq!(
            extended.inodes_with_an_extent_upper_leaf_entry(),
            BTreeSet::from([2])
        );
        assert_eq!(
            extended.inodes_other_than_the_first[&2].content.as_ref(),
            &[0u8; 7][..]
        );
        let again = model
            .answer_size_change_of_an_inode_other_than_the_first(2, 9)
            .expect("现行版本是删完之后建过 inode 的一版");
        succeed(&mut model, &again);
        let extended_again = current_version(&model);
        assert!(
            !roles_rewritten_by_the_publish(&extended_again).contains(&ModelUnitRole::ExtentRoot),
            "标签 0 已经在：再变长只改记录、上段整棵照抄：{:?}",
            roles_rewritten_by_the_publish(&extended_again)
        );
        let intent = model
            .answer_size_change_of_an_inode_other_than_the_first(2, 3)
            .expect("现行版本是删完之后建过 inode 的一版");
        succeed(&mut model, &intent);
        let truncated = current_version(&model);
        assert!(
            roles_rewritten_by_the_publish(&truncated)
                .contains(&ModelUnitRole::LoggedOperationsRoot)
                && truncated.logged_truncation_issued_txg() == Some(truncated.key.checkpoint_txg),
            "截短发截断意图：logged_ops 根重写、截断条目点名这一版的 txg：{:?}",
            roles_rewritten_by_the_publish(&truncated)
        );
        assert!(model.current_version_has_a_file_being_truncated());
        let last_batch = walk_one_step(&mut model);
        assert_eq!(last_batch.logged_truncation_issued_txg(), None);
        assert!(!roles_rewritten_by_the_publish(&last_batch).contains(&ModelUnitRole::ExtentRoot));
        assert_eq!(
            (
                last_batch.inodes_with_an_extent_upper_leaf_entry(),
                last_batch.inodes_other_than_the_first[&2].content.len()
            ),
            (BTreeSet::from([2]), 3),
            "最后一批放 0 个单元、标签 0 那一条照旧留着，读回 3 个 0"
        );
    }

    /// N = S（判决第三节 K12）：发一次只改记录的发布，不拒、不动 extent 树。
    #[test]
    fn truncating_to_the_same_size_publishes_one_record_only_change() {
        let mut model = model_with_a_file_of(payload() + 3);
        let same = truncate_to(&mut model, payload() + 3);
        assert_eq!(
            roles_rewritten_by_the_publish(&same),
            BTreeSet::from([
                ModelUnitRole::InodeLeaf,
                ModelUnitRole::InodeRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ])
        );
    }

    /// N 按整单元向上取整溢出 u64（判决第三节 K12）：在任何写之前必须拒，一条根都不写。
    #[test]
    fn a_truncation_size_whose_whole_data_units_overflow_is_refused_before_any_write() {
        let model = model_with_a_file_of(100);
        let answer = model.answer_truncation(u64::MAX).expect("会话开着");
        assert_eq!(
            answer.required_refusals,
            BTreeSet::from([ModelRefusalReason::TruncationSizeWhoseWholeDataUnitsOverflow])
        );
        assert!(answer.expected_roots.is_empty());
    }

    /// 回退到截断意图之后、最后一批之前的那一版（判决第三节 K1：logged_ops 里截断条目取回退目标那一版的）：现行那一版截断已做完、logged_ops 删空，
    /// 回退那次把目标的截断条目带回来、logged_ops 根这次重写；之后走空接着做截断的最后一批，再删空。
    #[test]
    fn a_rollback_to_a_version_in_the_middle_of_a_truncation_takes_its_truncation_entry_and_resumes_the_walk(
    ) {
        let mut model = model_with_a_file_of(2 * payload() + 9);
        let intent = truncate_to(&mut model, payload() + 1);
        walk_one_step(&mut model);
        let rollback = model
            .answer_rollback_while_mounted(intent.key)
            .expect("会话开着、现行版本带文件");
        succeed(&mut model, &rollback);
        let rolled_back = current_version(&model);
        assert_eq!(
            rolled_back.contents.logged_truncation_issued_txg(),
            Some(intent.key.checkpoint_txg)
        );
        assert_eq!(
            rolled_back
                .role_written_at
                .get(&ModelUnitRole::LoggedOperationsRoot),
            Some(&rolled_back.key.checkpoint_txg),
            "logged_ops 换成带截断条目的那一份"
        );
        assert_eq!(
            rolled_back.data_unit_written_at,
            intent.data_unit_written_at
        );
        let final_batch = walk_one_step(&mut model);
        assert_eq!(file_of(&final_batch).data_units_present, 2);
        assert!(!final_batch
            .role_written_at
            .contains_key(&ModelUnitRole::LoggedOperationsRoot));
    }

    /// 快照下截断再整份覆盖写（判决第三节 K4，攻方 W4 那段历史）：顺序写三个单元（txg 10）→ 建快照（11）→ 截到 1.5 个净荷，单元 1 重写（12）
    /// → 最后一批放单元 2（13）→ 快照下整份覆盖写（14）。覆盖写换下的单元 0 出生 10、不晚于快照，进 deadlist；单元 1 出生 12、晚于快照，直接释放。
    #[test]
    fn an_overwrite_under_a_snapshot_after_a_truncation_pins_unit_zero_and_releases_the_unit_the_truncation_rewrote(
    ) {
        let mut model = model_with_a_file_of(2 * payload() + payload() / 2);
        let snapshot = model.answer_create_snapshot().expect("会话开着、有树");
        succeed(&mut model, &snapshot);
        let intent = truncate_to(&mut model, payload() + payload() / 2);
        assert_eq!(intent.key.checkpoint_txg, ModelCheckpointTxg(12));
        let final_batch = walk_one_step(&mut model);
        let overwrite = model
            .answer_publish_sequential_write(&content_without_zeros(2 * payload()))
            .expect("会话开着、现行版本带文件");
        succeed(&mut model, &overwrite);
        let after_the_overwrite = current_version(&model);
        let data_unit_births = |root: &ModelRoot| {
            let mut births: Vec<ModelCheckpointTxg> = root
                .deadlist
                .iter()
                .filter(|entry| entry.mapping_key_unit_class == MODEL_MAPPING_KEY_UNIT_CLASS_DATA)
                .map(|entry| entry.birth_txg)
                .collect();
            births.sort_unstable();
            births
        };
        assert_eq!(
            data_unit_births(&final_batch),
            vec![ModelCheckpointTxg(10), ModelCheckpointTxg(10)],
            "截断换下的旧单元 1 与放掉的单元 2 都出生在快照之前"
        );
        assert_eq!(
            data_unit_births(&after_the_overwrite),
            vec![
                ModelCheckpointTxg(10),
                ModelCheckpointTxg(10),
                ModelCheckpointTxg(10)
            ],
            "覆盖写只把单元 0 记进 deadlist，单元 1（出生 12）直接释放"
        );
    }

    /// 这一版第一个文件那条记录的截断字段（模型记的），没有那条记录时 panic。
    fn truncation_field_of(root: &ModelRoot) -> ModelTruncationField {
        root.contents
            .first_file_record_truncation_field()
            .expect("这一版有第一个文件那条记录")
    }

    fn truncation_field_written_at(txg: u64, size_in_bytes: usize) -> ModelTruncationField {
        ModelTruncationField::WrittenByATruncationIntent {
            truncation_txg: ModelCheckpointTxg(txg),
            size_in_bytes: u64::try_from(size_in_bytes).expect("长度"),
        }
    }

    /// 放了单元的截断意图写字段（判决 `research/prompts/m4-trunc-r2-main-verification.md` 第六节用户定案「放了单元才写、整个替换」）：
    /// 三个单元的文件（顺序写 txg 10，字段没写过）截到 1 个净荷 + 5 字节，这次放单元之前存在 3 个、多于截后 size 罩的 2 个，意图那次（txg 11）
    /// 写成（11, P + 5）；之后再截到 7 字节（存在 2 个、多于罩的 1 个），整个替换成（13, 7），不取 min、不留上一次的。
    #[test]
    fn a_truncation_intent_releasing_data_units_writes_its_txg_and_the_new_size_replacing_the_previous_field(
    ) {
        let mut model = model_with_a_file_of(3 * payload());
        assert_eq!(
            truncation_field_of(&current_version(&model)),
            ModelTruncationField::NotWritten,
            "顺序写之后字段没写过"
        );
        let intent = truncate_to(&mut model, payload() + 5);
        assert_eq!(intent.key.checkpoint_txg, ModelCheckpointTxg(11));
        assert_eq!(
            truncation_field_of(&intent),
            truncation_field_written_at(11, payload() + 5)
        );
        assert_eq!(
            truncation_field_of(&intent).recorded_values(),
            ObservedTruncationField {
                truncation_txg: ModelCheckpointTxg(11),
                size_in_bytes: u64::try_from(payload() + 5).expect("长度"),
            },
            "盘上两段：T 是意图那次的 txg，N_w 是截到的 size"
        );
        walk_one_step(&mut model);
        let second_intent = truncate_to(&mut model, 7);
        assert_eq!(second_intent.key.checkpoint_txg, ModelCheckpointTxg(13));
        assert_eq!(
            truncation_field_of(&second_intent),
            truncation_field_written_at(13, 7),
            "第二次放了单元的截断整个替换"
        );
    }

    /// 截到洞里、选择子落在洞里更靠后（攻方 `research/prompts/m4-trunc-r2-opus-output.md`「打中一」那一形）：五个单元的文件（txg 10）截到 1 个净荷
    /// （txg 11 意图写（11, P），txg 12 最后一批放单元 1–4）→ 变长到 5 个净荷（txg 13，照抄）→ 截到 3 个净荷（txg 14：存在 1 个、不多于罩的 3 个，
    /// 一个不放，照抄（11, P），不换成「最近一次」的（14, 3P））→ 最后一批不放单元（照抄）→ 再变长（照抄）。
    #[test]
    fn a_truncation_intent_into_the_hole_releasing_no_data_unit_copies_the_field_of_the_earlier_truncation(
    ) {
        let mut model = model_with_a_file_of(5 * payload());
        truncate_to(&mut model, payload());
        let final_batch = walk_one_step(&mut model);
        assert_eq!(file_of(&final_batch).data_units_present, 1);
        let grown = truncate_to(&mut model, 5 * payload());
        assert_eq!(
            truncation_field_of(&grown),
            truncation_field_written_at(11, payload())
        );
        let intent_into_the_hole = truncate_to(&mut model, 3 * payload());
        assert_eq!(
            intent_into_the_hole.key.checkpoint_txg,
            ModelCheckpointTxg(14)
        );
        assert_eq!(
            truncation_field_of(&intent_into_the_hole),
            truncation_field_written_at(11, payload()),
            "截到洞里一个不放：照抄第一次截断写的"
        );
        let final_batch_releasing_nothing = walk_one_step(&mut model);
        assert_eq!(
            truncation_field_of(&final_batch_releasing_nothing),
            truncation_field_written_at(11, payload())
        );
        let grown_again = truncate_to(&mut model, 5 * payload());
        assert_eq!(
            truncation_field_of(&grown_again),
            truncation_field_written_at(11, payload())
        );
    }

    /// 截到 0 而文件只有一个单元（攻方「打中三」那条历史的前三步）：半个净荷的文件截到 0，存在 1 个、不多于 ⌈0 ÷ 净荷⌉.max(1) = 1 个，一个不放，
    /// 照抄（没写过）；之后变长到 1 个净荷 + 1 字节、截到 1 个净荷，都照抄，字段始终没写过。
    #[test]
    fn a_truncation_to_zero_of_a_one_unit_file_releases_nothing_and_copies_the_field() {
        let mut model = model_with_a_file_of(payload() / 2);
        let intent = truncate_to(&mut model, 0);
        assert_eq!(
            truncation_field_of(&intent),
            ModelTruncationField::NotWritten
        );
        walk_one_step(&mut model);
        let grown = truncate_to(&mut model, payload() + 1);
        assert_eq!(
            truncation_field_of(&grown),
            ModelTruncationField::NotWritten
        );
        let cut_to_one_payload = truncate_to(&mut model, payload());
        assert_eq!(
            truncation_field_of(&cut_to_one_payload),
            ModelTruncationField::NotWritten
        );
    }

    /// 截断的各批与最后一批不碰记录、照抄字段（判决 m4-trunc-r2 第三节 R2）：300 个单元截到 0（txg 11 意图写（11, 0)），两批（txg 12、13）与
    /// 最后一批（txg 14）都是（11, 0）。
    #[test]
    fn the_truncation_batches_and_the_final_batch_copy_the_field_the_intent_wrote() {
        let mut model = model_with_a_file_of(300 * payload());
        let intent = truncate_to(&mut model, 0);
        assert_eq!(
            truncation_field_of(&intent),
            truncation_field_written_at(11, 0)
        );
        for expected_txg in [12, 13] {
            let batch = walk_one_step(&mut model);
            assert_eq!(batch.key.checkpoint_txg, ModelCheckpointTxg(expected_txg));
            assert!(model.current_version_has_a_file_being_truncated());
            assert_eq!(
                truncation_field_of(&batch),
                truncation_field_written_at(11, 0)
            );
        }
        let final_batch = walk_one_step(&mut model);
        assert_eq!(final_batch.key.checkpoint_txg, ModelCheckpointTxg(14));
        assert!(matches!(
            final_batch.contents,
            ModelVersionContents::LiveFile(_)
        ));
        assert_eq!(
            truncation_field_of(&final_batch),
            truncation_field_written_at(11, 0)
        );
    }

    /// 变长、N = S 与只重写固定点单元的发布（建快照）照抄字段（判决 m4-trunc-r2 第三节 R2）：两个单元截到 7 字节（txg 11 写（11, 7)，txg 12
    /// 最后一批）→ 跨单元变长（txg 13）→ N = S（txg 14）→ 建快照（txg 15），都是（11, 7）。
    #[test]
    fn growing_the_same_size_and_a_snapshot_creation_copy_the_field() {
        let mut model = model_with_a_file_of(2 * payload());
        truncate_to(&mut model, 7);
        walk_one_step(&mut model);
        let grown = truncate_to(&mut model, 3 * payload() + 1);
        assert_eq!(grown.key.checkpoint_txg, ModelCheckpointTxg(13));
        assert_eq!(
            truncation_field_of(&grown),
            truncation_field_written_at(11, 7)
        );
        let same_size = truncate_to(&mut model, 3 * payload() + 1);
        assert_eq!(same_size.key.checkpoint_txg, ModelCheckpointTxg(14));
        assert_eq!(
            truncation_field_of(&same_size),
            truncation_field_written_at(11, 7)
        );
        let snapshot = model.answer_create_snapshot().expect("会话开着、有树");
        succeed(&mut model, &snapshot);
        let after_the_snapshot = current_version(&model);
        assert_eq!(
            after_the_snapshot.key.checkpoint_txg,
            ModelCheckpointTxg(15)
        );
        assert_eq!(
            truncation_field_of(&after_the_snapshot),
            truncation_field_written_at(11, 7)
        );
    }

    /// 整份写清零（判决 m4-trunc-r2 第三节 R2，用户定案取清零）：截过的文件（字段（11, 7)）覆盖写一个单元以内（txg 13）清零；再截一次写上
    /// （txg 14、15），顺序写跨几个单元（txg 16）也清零。
    #[test]
    fn an_overwrite_and_a_sequential_write_clear_the_field() {
        let mut model = model_with_a_file_of(2 * payload());
        truncate_to(&mut model, 7);
        walk_one_step(&mut model);
        let overwrite = model
            .answer_publish_overwrite(&content_without_zeros(100))
            .expect("会话开着、现行版本带文件");
        succeed(&mut model, &overwrite);
        let overwritten = current_version(&model);
        assert_eq!(overwritten.key.checkpoint_txg, ModelCheckpointTxg(13));
        assert_eq!(
            truncation_field_of(&overwritten),
            ModelTruncationField::NotWritten
        );
        let sequential = model
            .answer_publish_sequential_write(&content_without_zeros(3 * payload()))
            .expect("会话开着、现行版本带文件");
        succeed(&mut model, &sequential);
        let intent = truncate_to(&mut model, 9);
        assert_eq!(
            truncation_field_of(&intent),
            truncation_field_written_at(15, 9)
        );
        walk_one_step(&mut model);
        let sequential_again = model
            .answer_publish_sequential_write(&content_without_zeros(2 * payload()))
            .expect("会话开着、现行版本带文件");
        succeed(&mut model, &sequential_again);
        let rewritten = current_version(&model);
        assert_eq!(rewritten.key.checkpoint_txg, ModelCheckpointTxg(17));
        assert_eq!(
            truncation_field_of(&rewritten),
            ModelTruncationField::NotWritten
        );
    }

    /// unlink 的意图与各批照抄字段（判决 m4-trunc-r2 第三节 R2）：300 个单元截到 150 个净荷（txg 11 写（11, 150 P），一批放 144 个、最后一批放 6 个）
    /// → unlink 意图（txg 14，删除中、150 个单元）→ 不是最后一批的删除批（txg 15）都是（11, 150 P）；最后一批之后没有第一个文件那条记录。
    #[test]
    fn the_unlink_intent_and_the_deletion_batches_copy_the_field_until_the_record_is_deleted() {
        let mut model = model_with_a_file_of(300 * payload());
        truncate_to(&mut model, 150 * payload());
        walk_one_step(&mut model);
        walk_one_step(&mut model);
        let unlink = model
            .answer_unlink_intent()
            .expect("会话开着、现行版本带活文件");
        succeed(&mut model, &unlink);
        let unlink_intent = current_version(&model);
        assert_eq!(unlink_intent.key.checkpoint_txg, ModelCheckpointTxg(14));
        assert!(matches!(
            unlink_intent.contents,
            ModelVersionContents::FileBeingDeleted {
                data_units_remaining: 150,
                ..
            }
        ));
        assert_eq!(
            truncation_field_of(&unlink_intent),
            truncation_field_written_at(11, 150 * payload())
        );
        let deletion_batch = walk_one_step(&mut model);
        assert!(matches!(
            deletion_batch.contents,
            ModelVersionContents::FileBeingDeleted {
                data_units_remaining: 6,
                ..
            }
        ));
        assert_eq!(
            truncation_field_of(&deletion_batch),
            truncation_field_written_at(11, 150 * payload())
        );
        let final_deletion_batch = walk_one_step(&mut model);
        assert_eq!(
            final_deletion_batch
                .contents
                .first_file_record_truncation_field(),
            None,
            "最后一批删掉第一个文件那条记录"
        );
    }

    /// 回退随回退目标那一版（判决 m4-trunc-r2 第三节 R5）：三个单元的文件（txg 10，字段没写过）截到 7 字节（txg 11 写（11, 7)、txg 12 最后一批）
    /// → 回退到写字段之前那一版（txg 10，txg 13 那次回退）：字段没写过，不把现行那一版的（11, 7）并回去；之后变长（txg 14）照抄，仍没写过。
    /// 再回退到意图那一版（txg 11）取回（11, 7）。
    #[test]
    fn a_rollback_takes_the_field_of_its_target_and_the_growth_after_it_copies_that() {
        let mut model = model_with_a_file_of(3 * payload());
        let before_the_truncation = current_version(&model);
        let intent = truncate_to(&mut model, 7);
        walk_one_step(&mut model);
        let rollback = model
            .answer_rollback_while_mounted(before_the_truncation.key)
            .expect("会话开着、现行版本带文件");
        succeed(&mut model, &rollback);
        let rolled_back = current_version(&model);
        assert_eq!(rolled_back.key.checkpoint_txg, ModelCheckpointTxg(13));
        assert_eq!(
            truncation_field_of(&rolled_back),
            ModelTruncationField::NotWritten,
            "回退到写字段之前那一版：字段随它"
        );
        let grown = truncate_to(&mut model, 4 * payload());
        assert_eq!(grown.key.checkpoint_txg, ModelCheckpointTxg(14));
        assert_eq!(
            truncation_field_of(&grown),
            ModelTruncationField::NotWritten
        );
        let rollback_to_the_intent = model
            .answer_rollback_while_mounted(intent.key)
            .expect("会话开着、现行版本带文件");
        succeed(&mut model, &rollback_to_the_intent);
        assert_eq!(
            truncation_field_of(&current_version(&model)),
            truncation_field_written_at(11, 7),
            "回退到意图那一版取回它的字段"
        );
    }

    /// 对拍比字段（判决 m4-trunc-r2 第三节 R6「理想模型记字段并在对拍里比」）：截断意图那一版实现交回的记录里两段全零（没写字段）时报
    /// 截断字段对不上；照模型写的交回对得上、计数加一；模型这一版有那条记录而实现交回没有，也报。
    #[test]
    fn a_truncation_field_the_implementation_did_not_write_is_reported() {
        let model = model_with_a_file_of(3 * payload());
        let answer = model
            .answer_truncation(u64::try_from(payload() + 5).expect("长度"))
            .expect("会话开着、现行版本带活文件");
        let intent = answer.expected_roots[0].clone();
        let mut observed = observed_exactly_as_the_model_wrote(&intent);
        let mut counts = ModelJudgementCounts::default();
        assert_eq!(
            IdealModel::judge_first_file_record_truncation_field(&intent, &observed, &mut counts),
            Ok(())
        );
        assert_eq!(
            (
                counts.truncation_fields_compared,
                counts.written_truncation_fields_compared
            ),
            (1, 1)
        );
        observed.first_file_record_truncation_field =
            Some(ModelTruncationField::NotWritten.recorded_values());
        let not_written =
            IdealModel::judge_first_file_record_truncation_field(&intent, &observed, &mut counts)
                .expect_err("实现没写字段");
        assert_eq!(
            not_written.aspect,
            ModelDisagreementAspect::FirstFileRecordTruncationField
        );
        observed.first_file_record_truncation_field = None;
        let missing =
            IdealModel::judge_first_file_record_truncation_field(&intent, &observed, &mut counts)
                .expect_err("模型这一版有那条记录，实现交了没有");
        assert_eq!(
            missing.aspect,
            ModelDisagreementAspect::FirstFileRecordTruncationField
        );
    }

    fn upper_position(level: u64, index_in_level: u64) -> ModelExtentUpperNodePosition {
        ModelExtentUpperNodePosition {
            level,
            index_in_level,
        }
    }

    /// 模型自己那一份上段规范形计划（判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M2）走正推模型的四种走法
    /// （`research/prompts/m3-multi-file-unlink-r1-sonnet-model/out/run.out` 第 12、13、15、16 行，S = {1, 500}，产品几何）：
    /// 写 500 长高（旧根 (0, 0) 照抄成根之下、(0, 3) 与新根 (1, 0) 这次写）；删 500 降层（上一版的 (0, 0) 照抄成根，分配代不变）；
    /// 删 1（(0, 3) 照抄、根 (1, 0) 少了孩子重写）；删空（零条目根兼叶 (0, 0) 重写）。
    #[test]
    fn the_model_upper_plan_grows_lowers_and_empties_the_segment_as_the_forward_model_does() {
        let written_at_three = ModelCheckpointTxg(3);
        let written_at_five = ModelCheckpointTxg(5);
        let this_publish = ModelCheckpointTxg(9);
        let only_the_first_file = BTreeMap::from([(upper_position(0, 0), written_at_three)]);
        assert_eq!(
            extent_upper_node_written_at_after(
                &only_the_first_file,
                &BTreeSet::from([1, 500]),
                &BTreeSet::from([500]),
                this_publish,
            ),
            BTreeMap::from([
                (upper_position(0, 0), written_at_three),
                (upper_position(0, 3), this_publish),
                (upper_position(1, 0), this_publish),
            ]),
            "写 500 长高：旧根照抄成根之下的节点，500 那一串新写"
        );
        let both_files = BTreeMap::from([
            (upper_position(0, 0), written_at_three),
            (upper_position(0, 3), written_at_five),
            (upper_position(1, 0), written_at_five),
        ]);
        assert_eq!(
            extent_upper_node_written_at_after(
                &both_files,
                &BTreeSet::from([1]),
                &BTreeSet::from([500]),
                this_publish,
            ),
            BTreeMap::from([(upper_position(0, 0), written_at_three)]),
            "删 500 降层：上一版的第 0 片叶照抄成根"
        );
        assert_eq!(
            extent_upper_node_written_at_after(
                &both_files,
                &BTreeSet::from([500]),
                &BTreeSet::from([1]),
                this_publish,
            ),
            BTreeMap::from([
                (upper_position(0, 3), written_at_five),
                (upper_position(1, 0), this_publish),
            ]),
            "删 1：500 那片叶照抄，根少了孩子重写，第 0 片叶摘掉"
        );
        assert_eq!(
            extent_upper_node_written_at_after(
                &only_the_first_file,
                &BTreeSet::new(),
                &BTreeSet::from([1]),
                this_publish,
            ),
            BTreeMap::from([(upper_position(0, 0), this_publish)]),
            "删空：零条目根兼叶重写"
        );
    }

    /// 带数据单元的别的 inode 截短（判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S3、S5）：第一个文件旁边建 inode 2、
    /// 按号写三个满单元，截到 100 字节：意图那次单元 0 重写（分配代是这次的）、三个单元都还挂着、截断字段写成（这次 txg, 100）、截断条目点名它；
    /// 走空的下一步是放掉单元 1、2 的最后一批（删掉截断条目，剩一个单元）。逐版比时别的 inode 的截断字段与模型记的不同就报对不上
    /// （截断字段按 inode 比）。判别力：走空截断的一步把别的 inode 当没有单元（`crates/mutations.tsv` 一行）时最后一批照放 0 个、单元数断言红。
    #[test]
    fn truncating_another_inode_holding_data_units_rewrites_its_tail_unit_writes_its_truncation_field_and_releases_the_units_beyond(
    ) {
        let mut model = two_device_model();
        model.acquire_and_warm_up_in_the_make_filesystem_process();
        let first = model.answer_publish_first_file(&[3]).expect("会话开着");
        succeed(&mut model, &first);
        let created = model
            .answer_create_inodes(1)
            .expect("第一个文件旁边建 inode 2");
        succeed(&mut model, &created);
        let payload = data_unit_payload_capacity_in_bytes();
        let content = vec![5_u8; usize::try_from(3 * payload).expect("长度")];
        let written = model
            .answer_sequential_write_of_an_inode(2, &content)
            .expect("按号写 inode 2 三个满单元");
        succeed(&mut model, &written);
        let intent = model
            .answer_size_change_of_an_inode_other_than_the_first(2, 100)
            .expect("带数据单元的别的 inode 截到 100");
        let intent_root = intent.expected_roots[0].clone();
        let truncated = &intent_root.inodes_other_than_the_first[&2];
        assert_eq!(
            (
                truncated.content.len(),
                truncated.data_unit_written_at.as_slice(),
                truncated.truncation_field,
                truncated.truncation_issued_txg,
            ),
            (
                100,
                [
                    intent_root.key.checkpoint_txg,
                    written.expected_roots[0].key.checkpoint_txg,
                    written.expected_roots[0].key.checkpoint_txg,
                ]
                .as_slice(),
                ModelTruncationField::WrittenByATruncationIntent {
                    truncation_txg: intent_root.key.checkpoint_txg,
                    size_in_bytes: 100,
                },
                Some(intent_root.key.checkpoint_txg),
            ),
            "意图那次：读回 100 字节、单元 0 重写、三个单元都挂着、截断字段写成（这次 txg, 100）、截断条目点名它"
        );
        let mut counts = ModelJudgementCounts::default();
        let mut observed = observed_exactly_as_the_model_wrote(&intent_root);
        assert_eq!(
            model.judge_roots(
                std::slice::from_ref(&intent_root),
                std::slice::from_ref(&observed),
                &mut counts
            ),
            Ok(())
        );
        assert_eq!(
            (
                counts.truncation_fields_of_the_inodes_other_than_the_first_compared,
                counts.written_truncation_fields_of_the_inodes_other_than_the_first_compared,
            ),
            (1, 1),
            "inode 2 的截断字段比过、写过"
        );
        observed
            .truncation_fields_of_the_inodes_other_than_the_first
            .insert(
                2,
                ObservedTruncationField {
                    truncation_txg: ModelCheckpointTxg(0),
                    size_in_bytes: 0,
                },
            );
        let wrong = model
            .judge_roots(
                std::slice::from_ref(&intent_root),
                std::slice::from_ref(&observed),
                &mut counts,
            )
            .expect_err("inode 2 的截断字段交成没写过");
        assert_eq!(
            wrong.aspect,
            ModelDisagreementAspect::InodesOtherThanTheFirst
        );
        succeed(&mut model, &intent);
        assert!(model.current_version_has_a_file_being_truncated());
        let final_batch = model
            .answer_next_step_of_the_walk()
            .expect("走空截断的下一步");
        let after = &final_batch.expected_roots[0];
        assert_eq!(
            (
                after.inodes_other_than_the_first[&2].data_units_present(),
                after.logged_truncation_issued_txg(),
                after.role_written_at.get(&ModelUnitRole::ExtentRoot),
            ),
            (1, None, Some(&after.key.checkpoint_txg)),
            "最后一批放掉单元 1、2（剩一个，内联）、删掉截断条目、上段它那一串重写"
        );
    }
}
