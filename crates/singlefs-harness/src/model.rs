//! 理想模型（里程碑「覆盖写、释放、回退与复用」增补 3 第 2 件）：一个只住内存的模型，记每一版提交的内容、每条根属于哪个实例、实例表的行、回退下界 F，
//! 回答「冷启动该读回哪一版」「回退到这条根该不该被拒」「这个错误该不该出现」；第 1 件的每一步拿实现的结局与它比（比的那层胶水在
//! `model_comparison.rs`，那里可以用 `singlefs_core` 的类型）。
//!
//! 只 `use singlefs_format`，不 `use singlefs_core` / `singlefs_checker`（D13（验证路线） 已定项 5：只共享一份从 kb 生成的常量）：代号、实例、
//! 角色、根环落点、F 的生效值、抬 F 的上限、候选集、内容装不装得下都照条款另写一份，不照抄实现的算法。
//!
//! 不建崩溃：第 1 件的历史里没有断电、没有设备错，每一步的写都落完，所以环里最新的那条根就是最后写出的那一条、journal 里没有要施加的记录
//! （可写挂载写的上一个实例那一行 W 恒 0）。第 3 件（崩溃注入）接上时要加「这个崩溃状态恢复到的版本在不在允许集合里」。
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
    INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE, INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT,
    INODE_INTERNAL_ENTRY, INODE_LEAF_RECORDS, INSTANCE_TABLE_PAGE_RECORDS,
    JOURNAL_RING_DEFAULT_BYTES, JOURNAL_RING_START_SLOT, LINEAGE_CONTAINER_RECORDS,
    MAPPING_ENTRY_BYTES, MAPPING_KEY_BYTES, NODE_BYTES, NODE_POINTER_BYTES,
    NONCE_MAC_ALGORITHM_RESERVED_BYTES, ROOT_RING_REGIONS, ROOT_RING_REGION_DEVICES,
    ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM, SLOT_BYTES,
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
    Data,
    ExtentRoot,
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
            | ModelUnitRole::InodeRoot
            | ModelUnitRole::AllocationTree
            | ModelUnitRole::AccountingTree
            | ModelUnitRole::MappingTree
            | ModelUnitRole::TreeTable
            | ModelUnitRole::DeletedInodesRoot
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
    /// 墓碑树开放容器与根（追加一条墓碑），加固定点单元；数据单元与 extent 树照抄。
    UnlinkIntent,
    /// 不是最后一批的删除批（D8（核心索引结构） 已定项 16 第 2 条：剩下的单元多于产品批大小 144 时从尾部释放 144 个）：
    /// 留下的数据单元照抄（分配代不变）；重写被截掉那一片下段叶与它的祖先（下段节点不对应角色，占槽按下段层数取上界）、
    /// extent 上段根、inode 叶与根（记录的 size 改成剩下单元数 × 净荷容量，nlink 仍 0），加固定点单元；
    /// `deleted_inodes` 那条 key 与墓碑树不动。这一版仍是删除中的那一版。
    DeletionBatchKeepingDataUnits,
    /// 最后一批（D8（核心索引结构） 已定项 16 第 2 条）：剩下的单元不多于产品批大小 144 时意图之后（或上一批之后）的下一步——释放剩下的单元、
    /// 删 inode 记录（树里只有 inode 1，叶容器一片都不剩）、删 `deleted_inodes` 那条 key（删空取根指针置零、不写它的节点），
    /// 同一次发布；重写 extent 零条目根与 inode 零条目根，加固定点单元。唯一那片叶容器身份消失，退役树开放容器追加一条退役记录、
    /// 连根重写（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 3 条）。
    FinalDeletionBatch,
    /// 建一个 size 0 的 inode（`transaction::publish_new_inodes`，count 1；D8（核心索引结构） 已定项 16 第 5 条：删完之后只许建 inode、
    /// 写行与空发布），新记录落在最左那片叶容器里（删完之后建的前 233 条，[`inode_leaf_container_position_of_a_new_record`]）：
    /// inode 叶与根，加固定点单元；数据单元与 extent 树根照抄（新建的 inode 长度 0）。
    /// 执行器只在第一个文件已删完的一版上调（`history.rs` 的 `HistoryOperation::CreateInode`）。
    NewInode,
    /// 同 [`ModelPublishKind::NewInode`]，新记录落在最左那片之外的那一片里（删完之后建的第 234 条起）：只重写那一片与 inode 根，
    /// 左边那几片一个字节都不动、分配代不变（D8（核心索引结构） 已定项 6：分裂点取末尾，左半保留全部原有记录与身份；
    /// 「更新一条记录 = COW 叶 + 全部祖先」）。
    NewInodeInALeafContainerRightOfTheLeftmost(ModelInodeLeafContainerRightOfTheLeftmost),
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
            | ModelPublishKind::UnlinkIntent
            | ModelPublishKind::DeletionBatchKeepingDataUnits
            | ModelPublishKind::FinalDeletionBatch => false,
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
            | ModelPublishKind::UnlinkIntent
            | ModelPublishKind::DeletionBatchKeepingDataUnits
            | ModelPublishKind::FinalDeletionBatch
            | ModelPublishKind::NewInode
            | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_) => false,
        }
    }

    /// 这次发布的记录承载几个事务（事务号接着本实例的计数往下取）：写文件内容的两种一个数据单元一个事务（D16（发布语义） 已定项 5
    /// 的切分纪律，`data_units_of_the_new_content` 是这次写的内容切成几个单元）；删除类发布与建 inode 各一个事务；
    /// 空发布、写行、暖机、抬 F、回退的记录事务号 0（D23（journal 的角色与格式） 已定项 19 ①）。
    fn transactions_carried(self, data_units_of_the_new_content: u64) -> u64 {
        match self {
            ModelPublishKind::FirstFileVersion | ModelPublishKind::OverwriteFileVersion => {
                data_units_of_the_new_content
            }
            ModelPublishKind::UnlinkIntent
            | ModelPublishKind::DeletionBatchKeepingDataUnits
            | ModelPublishKind::FinalDeletionBatch
            | ModelPublishKind::NewInode
            | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_) => 1,
            ModelPublishKind::RowsOnFileVersion
            | ModelPublishKind::EmptyOnFileVersion
            | ModelPublishKind::RowsOnVersionWithoutFile
            | ModelPublishKind::ZeroUnit => 0,
        }
    }

    /// 这次发布之后这一版不再有的角色：最后一批释放数据单元、删掉唯一那片 inode 叶容器、`deleted_inodes` 删空取根指针置零。
    fn removed_roles(self) -> &'static [ModelUnitRole] {
        match self {
            ModelPublishKind::FinalDeletionBatch => &[
                ModelUnitRole::Data,
                ModelUnitRole::InodeLeaf,
                ModelUnitRole::DeletedInodesRoot,
            ],
            ModelPublishKind::FirstFileVersion
            | ModelPublishKind::OverwriteFileVersion
            | ModelPublishKind::RowsOnFileVersion
            | ModelPublishKind::EmptyOnFileVersion
            | ModelPublishKind::RowsOnVersionWithoutFile
            | ModelPublishKind::ZeroUnit
            | ModelPublishKind::UnlinkIntent
            | ModelPublishKind::DeletionBatchKeepingDataUnits
            | ModelPublishKind::NewInode
            | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_) => &[],
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
            ModelPublishKind::FirstFileVersion
            | ModelPublishKind::OverwriteFileVersion
            | ModelPublishKind::RowsOnFileVersion
            | ModelPublishKind::EmptyOnFileVersion
            | ModelPublishKind::RowsOnVersionWithoutFile
            | ModelPublishKind::ZeroUnit
            | ModelPublishKind::UnlinkIntent
            | ModelPublishKind::DeletionBatchKeepingDataUnits
            | ModelPublishKind::FinalDeletionBatch
            | ModelPublishKind::NewInode => {}
        }
        roles
    }

    /// 这次发布重写的、由种类定死的角色（建 inode 落在最左那片之外时那一片不在这里，[`ModelPublishKind::rewritten_roles`] 另加）。
    /// 分配记录树（`ModelUnitRole::AllocationTree`）在这里只记它的根：根之下重写几个节点按位置寻址现算上界
    /// （`IdealModel::allocation_record_tree_nodes_rewritten_upper_bound`），模型不记是哪几个。
    fn rewritten_roles_fixed_by_the_kind(self) -> &'static [ModelUnitRole] {
        match self {
            ModelPublishKind::FirstFileVersion | ModelPublishKind::OverwriteFileVersion => &[
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
            ModelPublishKind::UnlinkIntent => &[
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
            ModelPublishKind::DeletionBatchKeepingDataUnits => &[
                ModelUnitRole::ExtentRoot,
                ModelUnitRole::InodeLeaf,
                ModelUnitRole::InodeRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::MappingTree,
                ModelUnitRole::TreeTable,
            ],
            ModelPublishKind::FinalDeletionBatch => &[
                ModelUnitRole::ExtentRoot,
                ModelUnitRole::InodeRoot,
                ModelUnitRole::AllocationTree,
                ModelUnitRole::AccountingTree,
                ModelUnitRole::RetirementLeaf,
                ModelUnitRole::RetirementRoot,
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
        }
    }
}

/// 一版的文件：写出它那次发布的身份与内容。树表里 inode 树、extent 树的根指针由写出它的那次发布唯一定（指针带诞生代号），
/// 所以「两条根的这两棵树的根指针相同」⟺「两条根的文件是同一次发布写的」（D16（发布语义） 已定项 1「非空」从盘上怎么认）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelFileVersion {
    pub written_by: ModelRootKey,
    pub content: Rc<[u8]>,
}

/// 一版里有什么（D8（核心索引结构） 已定项 16 第 4、5 条：树表 0 条的、带文件的、有树而文件集合为空的；模型只罩 inode 1 一个文件）。
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
    FileBeingDeleted {
        trees_written_by: ModelRootKey,
        data_units_remaining: u64,
    },
    /// 最后一批之后：有树而文件集合为空（`deleted_inodes` 空、inode 树与 extent 树零条目根），读回「没有文件」。
    TreesAndNoFile { trees_written_by: ModelRootKey },
    /// 删完第一个文件之后建过 inode：inode 树里只有 size 0 的记录、没有 inode 1，没有数据单元、队列空，读回「没有文件」；
    /// 实现把它归带文件的一版（inode 树有条目，D8（核心索引结构） 已定项 16 第 5 条的判定不成立）。
    /// `trees_written_by`：最近一次建 inode 那次发布（回退到这一版的根照抄它）。
    /// `inode_records_in_the_tree`：这一版 inode 树里有几条记录（删完之后建了几个）：下一个建的落在哪一片叶容器由它定
    /// （[`inode_leaf_container_position_of_a_new_record`]）。
    InodesWithoutTheFirstFile {
        trees_written_by: ModelRootKey,
        inode_records_in_the_tree: u64,
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
            | ModelVersionContents::InodesWithoutTheFirstFile { .. } => true,
        }
    }

    /// 读得出的那个文件（inode 1 是活记录时）。
    #[must_use]
    pub fn live_file(&self) -> Option<&ModelFileVersion> {
        match self {
            ModelVersionContents::LiveFile(file) => Some(file),
            ModelVersionContents::WithoutTrees
            | ModelVersionContents::FileBeingDeleted { .. }
            | ModelVersionContents::TreesAndNoFile { .. }
            | ModelVersionContents::InodesWithoutTheFirstFile { .. } => None,
        }
    }

    /// 这一版的用户可见树（inode 树、extent 树）的根指针是谁写的：树表 0 条时 None。树表里这两棵树的根指针由写出它们的那次发布唯一定
    /// （指针带诞生代号），「两条根的这两棵树的根指针相同」⟺「是同一次发布写的」（D16（发布语义） 已定项 1「非空」从盘上怎么认）。
    fn user_visible_trees_written_by(&self) -> Option<ModelRootKey> {
        match self {
            ModelVersionContents::WithoutTrees => None,
            ModelVersionContents::LiveFile(file) => Some(file.written_by),
            ModelVersionContents::FileBeingDeleted {
                trees_written_by, ..
            }
            | ModelVersionContents::TreesAndNoFile { trees_written_by }
            | ModelVersionContents::InodesWithoutTheFirstFile {
                trees_written_by, ..
            } => Some(*trees_written_by),
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
            | ModelVersionContents::FileBeingDeleted { .. } => None,
        }
    }

    /// 有树、文件集合不空的那一版（带活文件的、删除中的、删完之后建过 inode 的）：可写会话里覆盖写、回退、抬 F 的执行器只在实现报「带文件」时调，
    /// 实现把删除中的那一版（`deleted_inodes` 不空）与只有 size 0 的 inode 的那一版（inode 树有条目）也归带文件。
    fn is_a_version_with_a_file_or_a_file_being_deleted(&self) -> bool {
        match self {
            ModelVersionContents::LiveFile(_)
            | ModelVersionContents::FileBeingDeleted { .. }
            | ModelVersionContents::InodesWithoutTheFirstFile { .. } => true,
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
    /// COW：单元只在分配它的那次发布里写）。mkfs 写的实例表与第 0 版树表记 txg 0。
    pub role_written_at: BTreeMap<ModelUnitRole, ModelCheckpointTxg>,
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
}

impl ModelRoot {
    /// 这条根的用户可见树（inode 树、extent 树）的根指针是谁写的：树表 0 条时 None。
    fn user_visible_trees_written_by(&self) -> Option<ModelRootKey> {
        self.contents.user_visible_trees_written_by()
    }
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

/// S 取 mkfs 那一档：模型只建 mkfs 默认参数的池，从不改 S（改 S 的池由
/// `crates/singlefs-harness/tests/system_configuration_slots_per_region.rs` 直接在实现上跑）。
/// 写成「mkfs 写的那个值」而不是「每区槽数 S」，是为了让「模型假定 S 不变」这件事在名字里就看得见。
const MODEL_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 =
    ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM;

fn ring_position_of(checkpoint_txg: ModelCheckpointTxg) -> ModelRingPosition {
    ModelRingPosition {
        region: checkpoint_txg.0 % ROOT_RING_REGIONS,
        slot_in_region: (checkpoint_txg.0 / ROOT_RING_REGIONS)
            % MODEL_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM,
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
    let data_units = match contents {
        ModelVersionContents::LiveFile(file) => {
            data_units_of_content(u64::try_from(file.content.len()).expect("内容长度"))
        }
        ModelVersionContents::FileBeingDeleted {
            data_units_remaining,
            ..
        } => *data_units_remaining,
        ModelVersionContents::WithoutTrees
        | ModelVersionContents::TreesAndNoFile { .. }
        | ModelVersionContents::InodesWithoutTheFirstFile { .. } => 0,
    };
    let entries_of_one_node_roles: u64 = role_written_at
        .keys()
        .map(|role| match role {
            // 记账树：根与根之下的每个节点各一条（台阶行多到一片叶装不下时长层），按这一版节点数的上界数。
            ModelUnitRole::AccountingTree => accounting_tree_nodes,
            // 模型罩的一版里这几个角色各一个单元（上段只有根兼叶、每片 inode 叶容器各是一个角色、墓碑树一片容器加根）。
            ModelUnitRole::ExtentRoot
            | ModelUnitRole::InodeLeaf
            | ModelUnitRole::InodeLeafRightOfTheLeftmost(_)
            | ModelUnitRole::InodeRoot
            | ModelUnitRole::DeletedInodesRoot
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
    /// 要建立新实例的那一版树表 0 条、而它的实例表已经不是 mkfs 那一片（上一次挂载在这一版上写过行）：
    /// 被换下的那一片记在哪没有条款——树表 0 条 ⇒ 这一版没有分配记录树，那次释放只住内存，重开之后账取不回来，
    /// 而根环里更旧的候选根还指着它。第一版不支持（设计空白，交主 agent）。
    VersionWithoutFileNotWrittenByMakeFilesystem,
    /// 要抬的 F 超过上限（D16（发布语义） 已定项 1「抬 F 的上限」）。
    FloorAboveCeiling,
    /// 可写挂载读到的样子里有比所选那一版新的发布读不出、系统配置见证过它，重读一次仍读不出（C554 乙：用户 2026-09-27 定
    /// 「乙 重读后再判」，`research/prompts/c554-fix-forks.md` #1「挂载时有一条根（或它引用的账）暂时读不出」；重读次数取自 D16（发布语义） 已定项 1「根槽这一次读坏」那一行的
    /// 「重读一次」，挂进 D23（journal 的角色与格式） 已定项 14）：取号之前拒，盘上逐字节不变。
    NewerPublishWitnessedBySystemConfigurationStillUnreadableAfterOneReread,
    /// 现行那一版有树而文件集合为空（D8（核心索引结构） 已定项 16 第 5 条：删完之后只许建 inode（size 0）、写行与空发布），
    /// 这次改动不是建 inode：覆盖写、unlink 先走空队列（走空把删除中的那一版走成这一形），再拒（`UserChangeRefused::NoFileVersionToChange`）。
    OnlyNewInodesOnAVersionWithTreesAndNoFile,
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
            | ModelRefusalReason::OnlyNewInodesOnAVersionWithTreesAndNoFile => false,
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
    /// 不是容量墙、也允许拒绝的理由（照代码今天的读法划的「第一版不支持」区，条款没写）。
    pub permitted_refusals: BTreeSet<ModelRefusalReason>,
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
    /// 这一版每个角色的单元在这一版分配记录里的那几条，每块盘各一条：实现交回的这一版的全部角色（带文件的一版、树表 0 条的一版上写行的那一版）。
    EveryRoleOfTheVersion(Vec<(ModelUnitRole, Vec<ObservedAllocationRecord>)>),
    /// 实现的输出不带这一版的分配记录（树表 0 条的一版上的零单元发布：一个字节都不写、不经分配器），只交回这次发布重写了哪几个角色。
    /// 模型说这次重写了角色的那一版这么交回，判「单元的分配代」对不上。同一次挂载里前一版交回过分配记录的，执行器用的胶水从它往下带、
    /// 交回 [`ObservedUnitAllocationRecords::EveryRoleOfTheVersion`]（实审 B3c-4）；执行器里这一臂只剩一次挂载里第一版就是零单元发布的那几版。
    RewrittenRolesOnly(BTreeSet<ModelUnitRole>),
    /// 零单元发布交回的根换了分配记录树根指针（该照抄同一次挂载里前一版的那一条，胶水往下带时核出来的，实审 B3c-4）：
    /// 这一版的分配记录说不上是哪一份，模型判「单元的分配代」对不上，不当比不了。`what` 写两条指针。
    AllocationRecordTreeRootChangedByAZeroUnitPublish { what: String },
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
            ModelDisagreementAspect::NotModeled => "模型答不了",
        }
    }
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
    /// 对不上的是「模型说该成、实现拒了」或「拒绝的理由」，而实现拒的是「根环槽读坏、重读仍坏」（抬 F 算上限、挂着时回退判候选集都算）
    /// 时它点名的那个槽；别的格都是 None。
    /// 故障注入按它比「被吞的根槽写落在哪个槽」（实八，D16（发布语义） 已定项 1「根槽这一次读坏」那一行）。
    pub implementation_reported_root_ring_slot_still_bad_after_one_reread:
        Option<ModelRingPosition>,
    /// 对不上的是「写的实例表行」（[`ModelDisagreementAspect::InstanceRows`]）时实现写的那几行；别的格都是 None。
    /// 故障注入按它比「被吞根槽的那几条根由记录重建之后模型重算的行」（实八，D23（journal 的角色与格式） 已定项 14 第 4 条）。
    pub implementation_rows_written: Option<Vec<ModelInstanceRow>>,
}

impl ModelDisagreement {
    /// 只带哪一格与两边的话，实现报的上限、读坏的根环槽、写的行都是 None。
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
            implementation_reported_root_ring_slot_still_bad_after_one_reread: None,
            implementation_rows_written: None,
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
    ring: BTreeMap<ModelRingPosition, ModelRoot>,
    /// 取过的最大实例代号（系统配置里的号；取号先于任何带新号的写，D23（journal 的角色与格式） 已定项 16）。
    highest_acquired_instance: ModelInstanceGeneration,
    /// 发布过的最大 txg 与 jsn：不建崩溃，环里全部根与全部记录里的最大值就是它们（D23（journal 的角色与格式） 已定项 14 第 3 条）。
    highest_published_txg: ModelCheckpointTxg,
    highest_journal_counter: ModelJournalCounter,
    session: Option<ModelSession>,
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
            occupied_slots_upper_bound_per_device: ModelUnitRole::InstanceTable.span_in_slots()
                + ModelUnitRole::TreeTable.span_in_slots(),
            central_mapping_tree_nodes_upper_bound: 0,
            inode_number_watermark_steps: 0,
            accounting_rows: 0,
            accounting_tree_nodes_upper_bound: 0,
            highest_transaction_number_of_its_publish: 0,
            highest_transaction_number_in_its_instance: 0,
        };
        let mut ring = BTreeMap::new();
        ring.insert(ring_position_of(MAKE_FILESYSTEM_TXG), genesis);
        Self {
            geometry,
            unit_area_start_slot: model_unit_area_start_slot(journal_ring_bytes),
            ring,
            highest_acquired_instance: MAKE_FILESYSTEM_INSTANCE,
            highest_published_txg: MAKE_FILESYSTEM_TXG,
            highest_journal_counter: ModelJournalCounter(0),
            session: None,
        }
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
    /// （[`crash_recovery_disagreement`]）。并不掉：根环有 R × S = 24 个槽，一步至多写出四条根（可写挂载的写行加暖机至多三次、
    /// 抬 F 两次空发布），下一步之前必然还在环里。
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
        let mut non_empty: Vec<(ModelRootKey, Option<ModelRootKey>)> = Vec::new();
        let mut previous_trees_written_by: Option<ModelRootKey> = None;
        for root in &valid {
            let trees_written_by = root.user_visible_trees_written_by();
            if trees_written_by != previous_trees_written_by {
                non_empty.push((root.key, trees_written_by));
            }
            previous_trees_written_by = trees_written_by;
        }
        // 「4 个不同状态」去重（「非空」从盘上怎么认那一段末尾）：从新到旧数，状态（两棵用户可见树是哪次发布写的）与比它新、
        // 已计入的根相同的不计入——管理员回退发出的根状态同回退目标。去重之后不足 4 个取最旧的有效根（根环容量边界）。
        non_empty.sort_unstable_by_key(|(key, _)| std::cmp::Reverse(*key));
        let mut counted_states: Vec<Option<ModelRootKey>> = Vec::new();
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

    /// 接在可写会话的现行版本后面、写记账行的下一次发布写几行（[`IdealModel::accounting_rows_of_a_publish_on`]）；没有可写会话时 None。
    #[must_use]
    pub fn accounting_rows_of_the_next_publish_in_the_session(&self) -> Option<u64> {
        self.session
            .as_ref()
            .map(|session| self.accounting_rows_of_a_publish_on(&session.current))
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
    ) -> ModelRoot {
        let key = ModelRootKey {
            checkpoint_txg,
            instance,
        };
        let mut role_written_at = previous.role_written_at.clone();
        for role in kind.removed_roles() {
            role_written_at.remove(role);
        }
        let data_units_of_the_new_content = new_file_content.map_or(0, |content| {
            data_units_of_content(u64::try_from(content.len()).expect("内容长度"))
        });
        let data_units_remaining_before_this_publish = match previous.contents {
            ModelVersionContents::FileBeingDeleted {
                data_units_remaining,
                ..
            } => Some(data_units_remaining),
            ModelVersionContents::WithoutTrees
            | ModelVersionContents::LiveFile(_)
            | ModelVersionContents::TreesAndNoFile { .. }
            | ModelVersionContents::InodesWithoutTheFirstFile { .. } => None,
        };
        let rewritten = kind.rewritten_roles();
        for role in &rewritten {
            role_written_at.insert(*role, checkpoint_txg);
        }
        let mut contents = match (kind, new_file_content) {
            (
                ModelPublishKind::FirstFileVersion | ModelPublishKind::OverwriteFileVersion,
                Some(content),
            ) => ModelVersionContents::LiveFile(ModelFileVersion {
                written_by: key,
                content: Rc::from(content),
            }),
            (ModelPublishKind::UnlinkIntent, None) => ModelVersionContents::FileBeingDeleted {
                trees_written_by: key,
                data_units_remaining: previous.contents.live_file().map_or_else(
                    || panic!("意图那次只接在带活文件的一版后面（`answer_unlink_intent` 判过）：{:?}", previous.contents),
                    |file| data_units_of_content(u64::try_from(file.content.len()).expect("内容长度")),
                ),
            },
            (ModelPublishKind::DeletionBatchKeepingDataUnits, None) => {
                ModelVersionContents::FileBeingDeleted {
                    trees_written_by: key,
                    data_units_remaining: data_units_remaining_before_this_publish
                        .expect("不是最后一批的删除批只接在删除中的那一版后面（`answer_deletion_step` 判过）")
                        - MODEL_DELETION_BATCH_DATA_UNITS,
                }
            }
            (ModelPublishKind::FinalDeletionBatch, None) => ModelVersionContents::TreesAndNoFile {
                trees_written_by: key,
            },
            (
                ModelPublishKind::NewInode
                | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_),
                None,
            ) => ModelVersionContents::InodesWithoutTheFirstFile {
                trees_written_by: key,
                inode_records_in_the_tree: previous
                    .contents
                    .inode_records_created_after_the_first_file_was_deleted()
                    .expect("建 inode 只接在第一个文件已删完的一版后面（`answer_create_inode` 判过）")
                    + 1,
            },
            (
                ModelPublishKind::RowsOnFileVersion
                | ModelPublishKind::EmptyOnFileVersion
                | ModelPublishKind::RowsOnVersionWithoutFile
                | ModelPublishKind::ZeroUnit,
                None,
            ) => previous.contents.clone(),
            (ModelPublishKind::FirstFileVersion | ModelPublishKind::OverwriteFileVersion, None)
            | (
                ModelPublishKind::RowsOnFileVersion
                | ModelPublishKind::EmptyOnFileVersion
                | ModelPublishKind::RowsOnVersionWithoutFile
                | ModelPublishKind::ZeroUnit
                | ModelPublishKind::UnlinkIntent
                | ModelPublishKind::DeletionBatchKeepingDataUnits
                | ModelPublishKind::FinalDeletionBatch
                | ModelPublishKind::NewInode
                | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_),
                Some(_),
            ) => panic!("只有写文件内容的两种发布带内容，别的都不带：{kind:?}"),
        };
        match user_visible_trees {
            UserVisibleTreesOfTheNewVersion::AsThisPublishLeavesThem => {}
            UserVisibleTreesOfTheNewVersion::TakenFromTheRollbackTarget(target_root) => {
                contents.clone_from(&target_root.contents);
                // 新根的 inode 树、extent 树与 `deleted_inodes` 取回退目标那一版的（D23（journal 的角色与格式） 已定项 14）：
                // 目标那一版有的角色取它的分配代，没有的（删完的一版没有数据单元、叶容器与 `deleted_inodes` 根）这一版也没有；
                // 墓碑树与退役树随现行那一版。
                for role in [
                    ModelUnitRole::Data,
                    ModelUnitRole::ExtentRoot,
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
                // 内容的每个数据单元都在这次写（模型把它们记成同一个角色、同一个分配代）。
                ModelUnitRole::Data => {
                    slots_written += role.span_in_slots() * data_units_of_the_new_content;
                }
                // 记账树一次至多重写这一版的全部节点（根与根之下的，每个节点每盘占一个槽）：根之下的节点不对应角色，按上界算。
                ModelUnitRole::AccountingTree => {
                    slots_written += role.span_in_slots() * accounting_tree_nodes;
                }
                // 中央映射树在这里只记它的根：根之下的节点数按上界另算（下面）。
                ModelUnitRole::ExtentRoot
                | ModelUnitRole::InodeLeaf
                | ModelUnitRole::InodeLeafRightOfTheLeftmost(_)
                | ModelUnitRole::InodeRoot
                | ModelUnitRole::MappingTree
                | ModelUnitRole::TreeTable
                | ModelUnitRole::InstanceTable
                | ModelUnitRole::DeletedInodesRoot
                | ModelUnitRole::TombstoneLeaf
                | ModelUnitRole::TombstoneRoot
                | ModelUnitRole::RetirementLeaf
                | ModelUnitRole::RetirementRoot => slots_written += role.span_in_slots(),
            }
        }
        // extent 树下段的节点不对应角色（同分配记录树根之下的节点：分配代与位置归池级 checker 判），占槽按条款现算：
        // 写内容的两种重写整个下段；不是最后一批的删除批只重写被截掉那一片下段叶与它的祖先，每层至多一个。
        let extent_lower_segment_nodes_written = match kind {
            ModelPublishKind::FirstFileVersion | ModelPublishKind::OverwriteFileVersion => {
                extent_lower_segment_nodes(data_units_of_the_new_content)
            }
            ModelPublishKind::DeletionBatchKeepingDataUnits => {
                extent_lower_segment_levels(data_units_remaining_before_this_publish.expect(
                    "不是最后一批的删除批只接在删除中的那一版后面（`answer_deletion_step` 判过）",
                ))
            }
            ModelPublishKind::RowsOnFileVersion
            | ModelPublishKind::EmptyOnFileVersion
            | ModelPublishKind::RowsOnVersionWithoutFile
            | ModelPublishKind::ZeroUnit
            | ModelPublishKind::UnlinkIntent
            | ModelPublishKind::FinalDeletionBatch
            | ModelPublishKind::NewInode
            | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_) => 0,
        };
        slots_written += extent_lower_segment_nodes_written * (NODE_BYTES / SLOT_BYTES);
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
                    ),
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
        let transactions_of_this_publish = kind.transactions_carried(data_units_of_the_new_content);
        let highest_transaction_number_of_its_publish = if transactions_of_this_publish > 0 {
            highest_transaction_number_before_this_publish + transactions_of_this_publish
        } else {
            0
        };
        // 一个事务一条记录（C310（事务切分纪律与记录数口径打架） 用户定案「一条记录装一个事务连同它的元数据」），不承载事务的发布也写一条；
        // jsn 一条记录加一（D23（journal 的角色与格式） 已定项 7），根带的是这次发布末条记录的 jsn。`journal_counter` 是这次第一条记录的。
        // 末条再跨记录（D23 已定项 17）只在最后一个事务要点名的项多于一条记录装得下的 64 项时才有，模型罩的发布点名的项远不到。
        let records_of_this_publish = transactions_of_this_publish.max(1);
        ModelRoot {
            key,
            journal_counter: ModelJournalCounter(journal_counter.0 + records_of_this_publish - 1),
            rollback_floor,
            contents,
            instance_table_rows,
            role_written_at,
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
            BTreeSet::new(),
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
        if current.contents.live_file().is_none() {
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
        Ok(self.answer_for_publishes(operation, required_refusals, BTreeSet::new(), vec![root]))
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
            BTreeSet::new(),
            vec![root],
        ))
    }

    /// unlink 第一个文件的意图那次（`mounted_session::UserChange::Unlink`，D8（核心索引结构） 已定项 16 第 1 条）：接在现行那一版后面，
    /// inode 1 的记录 nlink 写 0、`deleted_inodes` 加 key、墓碑树追加一条；这一版读回「没有文件」。会话接着走空（每一批由
    /// [`IdealModel::answer_deletion_step`] 答）；两次之间会话为空间推的抬 F 由执行器按 txg 次序插进来比。逐次发布判墙。
    ///
    /// # Errors
    /// 模型里没有可写会话，或现行版本没有读得出的文件（执行器只在实现报带文件、队列空时调）：会话状态与实现对不上。
    pub fn answer_unlink_intent(&self) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        if let Some(refused) = self.answer_refusing_a_change_on_a_version_with_trees_and_no_file(
            ModelOperationKind::PublishUnlinkIntent,
        ) {
            return Ok(refused);
        }
        if current.contents.live_file().is_none() {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::SessionState,
                format!(
                    "模型里现行版本 {:?} 没有读得出的文件：{:?}",
                    current.key, current.contents
                ),
                "实现的现行版本带文件、队列空（执行器调了 unlink）".to_string(),
            ));
        }
        let root = self.next_root(
            current,
            session.instance,
            ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1),
            ModelJournalCounter(self.highest_journal_counter.0 + 1),
            ModelPublishKind::UnlinkIntent,
            current.rollback_floor,
            None,
            Rc::clone(&current.instance_table_rows),
        );
        Ok(self.answer_for_publishes(
            ModelOperationKind::PublishUnlinkIntent,
            BTreeSet::new(),
            BTreeSet::new(),
            vec![root],
        ))
    }

    /// 走空 `deleted_inodes` 的一步（D8（核心索引结构） 已定项 16 第 2、6 条：会话里 unlink 之后、每次用户改动之前，挂载续做）：
    /// 删除中的那一版这个文件还剩的数据单元多于产品批大小 144 就是一个不是最后一批的删除批（从尾部释放 144 个，之后仍是删除中的那一版），
    /// 不多于就是最后一批，之后这一版有树而文件集合为空。下一步只由现行那一版定，挂载续做与一路做下来同一个答法。
    ///
    /// # Errors
    /// 模型里没有可写会话，或现行版本没有删除中的文件（执行器只在实现交回了一步删除时问）：会话状态与实现对不上。
    pub fn answer_deletion_step(&self) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        let ModelVersionContents::FileBeingDeleted {
            data_units_remaining,
            ..
        } = current.contents
        else {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::SessionState,
                format!(
                    "模型里现行版本 {:?} 没有删除中的文件：{:?}",
                    current.key, current.contents
                ),
                "实现交回了走空 deleted_inodes 的一步".to_string(),
            ));
        };
        let step = if data_units_remaining > MODEL_DELETION_BATCH_DATA_UNITS {
            ModelPublishKind::DeletionBatchKeepingDataUnits
        } else {
            ModelPublishKind::FinalDeletionBatch
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
            | ModelVersionContents::FileBeingDeleted { .. } => {
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
        let inode_records_before = current
            .contents
            .inode_records_created_after_the_first_file_was_deleted()
            .expect("上面只放过第一个文件已删完的两种");
        // 新记录落在哪一片叶容器定这次重写哪一片（D8（核心索引结构） 已定项 6）：最左那片之外的那几片各是一个角色。
        let kind = match NonZeroU64::new(inode_leaf_container_position_of_a_new_record(
            inode_records_before,
        )) {
            None => ModelPublishKind::NewInode,
            Some(position_from_the_leftmost) => {
                ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(
                    ModelInodeLeafContainerRightOfTheLeftmost {
                        position_from_the_leftmost,
                    },
                )
            }
        };
        let root = self.next_root(
            current,
            session.instance,
            ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1),
            ModelJournalCounter(self.highest_journal_counter.0 + 1),
            kind,
            current.rollback_floor,
            None,
            Rc::clone(&current.instance_table_rows),
        );
        Ok(self.answer_for_publishes(
            ModelOperationKind::PublishNewInode,
            BTreeSet::new(),
            BTreeSet::new(),
            vec![root],
        ))
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
                BTreeSet::new(),
                Vec::new(),
            )),
            ModelVersionContents::WithoutTrees
            | ModelVersionContents::LiveFile(_)
            | ModelVersionContents::FileBeingDeleted { .. }
            | ModelVersionContents::InodesWithoutTheFirstFile { .. } => None,
        }
    }

    /// 模型的现行版本有没有删除中的文件（`deleted_inodes` 不空）：执行器拿它分一次会话交回的那几版里哪一版是走空的一步。
    #[must_use]
    pub fn current_version_has_a_file_being_deleted(&self) -> bool {
        self.session.as_ref().is_some_and(|session| {
            matches!(
                session.current.contents,
                ModelVersionContents::FileBeingDeleted { .. }
            )
        })
    }

    fn answer_for_publishes(
        &self,
        operation: ModelOperationKind,
        required_refusals: BTreeSet<ModelRefusalReason>,
        permitted_refusals: BTreeSet<ModelRefusalReason>,
        roots: Vec<ModelRoot>,
    ) -> ModelAnswer {
        ModelAnswer {
            operation,
            required_refusals,
            permitted_refusals,
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
            BTreeSet::new(),
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
                );
                vec![rollback_root]
            }
            (Some(_) | None, _) => Vec::new(),
        };
        let mut answer = self.answer_for_publishes(
            ModelOperationKind::RollbackWhileMounted,
            required_refusals,
            BTreeSet::new(),
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
        let mut answer =
            self.answer_for_publishes(operation, required_refusals, BTreeSet::new(), roots);
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
        let permitted_refusals = BTreeSet::new();
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
            permitted_refusals,
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
        let mut answer = self.answer_for_publishes(
            ModelOperationKind::Unmount,
            BTreeSet::new(),
            BTreeSet::new(),
            roots,
        );
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
            permitted_refusals: BTreeSet::new(),
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
            | ModelRefusalReason::OnlyNewInodesOnAVersionWithTreesAndNoFile => false,
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
                                || answer.permitted_refusals.contains(candidate)
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
                    disagreement
                        .implementation_reported_root_ring_slot_still_bad_after_one_reread =
                        reported_root_ring_slot_still_bad_after_one_reread
                            .filter(|_| refused_before_any_write);
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
                if answer.operation == ModelOperationKind::PublishDeletionStep
                    && answer.expected_roots.iter().any(|root| {
                        matches!(root.contents, ModelVersionContents::FileBeingDeleted { .. })
                    })
                {
                    counts.deletion_batches_keeping_data_units_matched += 1;
                }
                self.advance_by_success(answer);
                Ok(counts)
            }
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
                implementation_reported_root_ring_slot_still_bad_after_one_reread: None,
                implementation_rows_written: None,
            })
        }
    }

    fn judge_effect(
        &self,
        answer: &ModelAnswer,
        effect: &ObservedEffect,
        counts: &mut ModelJudgementCounts,
    ) -> Result<(), ModelDisagreement> {
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
            let Some(written_at) = expected.role_written_at.get(role) else {
                return Err(ModelDisagreement::new(
                    ModelDisagreementAspect::AllocationGeneration,
                    format!("{:?} 这一版没有 {role:?} 这个角色", expected.key),
                    format!("实现交回了 {role:?} 的单元"),
                ));
            };
            let devices_seen: BTreeSet<ModelDeviceIdentity> =
                records.iter().map(|record| record.device).collect();
            let devices_expected: BTreeSet<ModelDeviceIdentity> =
                self.geometry.devices.iter().copied().collect();
            let well_formed = records.len() == self.geometry.devices.len()
                && devices_seen == devices_expected
                && records
                    .iter()
                    .all(|record| !record.is_released && record.generation == *written_at);
            counts.allocation_records_compared += u64::try_from(records.len()).expect("条数");
            if !well_formed {
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
        Ok(())
    }

    fn write_root(&mut self, root: ModelRoot) {
        self.highest_published_txg = self.highest_published_txg.max(root.key.checkpoint_txg);
        self.highest_journal_counter = self.highest_journal_counter.max(root.journal_counter);
        self.ring
            .insert(ring_position_of(root.key.checkpoint_txg), root);
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

fn describe_answer(answer: &ModelAnswer) -> String {
    if answer.required_refusals.is_empty() {
        let permitted: Vec<&'static str> = answer
            .permitted_refusals
            .iter()
            .map(|reason| reason.name())
            .collect();
        format!(
            "{:?} 该成（写出 {:?}；允许拒的只有容量墙区间与 {permitted:?}）",
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

#[cfg(test)]
mod tests {
    use super::*;

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
            rollback_root.role_written_at.get(&ModelUnitRole::Data),
            Some(&ModelCheckpointTxg(6)),
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
                row_publish
                    .role_written_at
                    .iter()
                    .map(|(role, written_at)| {
                        let generation = if *role == ModelUnitRole::Data {
                            data_generation
                        } else {
                            written_at.0
                        };
                        (*role, records(generation))
                    })
                    .collect(),
            ),
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
                root.role_written_at
                    .iter()
                    .map(|(role, written_at)| (*role, records(*written_at)))
                    .collect(),
            ),
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
        records.retain(|(role, _)| *role != ModelUnitRole::InodeRoot);
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
}
