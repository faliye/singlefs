//! 单元角色与发布种类：一次发布重写哪些角色的单元、删去哪些，连同跟着发布种类走的几样枚举。

use super::*;

/// inode 树里最左那片之外的一片叶容器：它在树里左起第几片（从 0 数，最左那片是 0，这里恒不小于 1）。
/// 只经 [`ModelUnitRole::inode_leaf_container_at`] 造，第 0 片造不出这个类型（那一片是 [`ModelUnitRole::InodeLeaf`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModelInodeLeafContainerRightOfTheLeftmost {
    pub(super) position_from_the_leftmost: NonZeroU64,
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
    pub(super) fn is_a_node_of_the_extent_upper_segment(self) -> bool {
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
pub(super) enum ModelPublishKind {
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
    /// 这一版算一个新状态（「照字节比，搬根算非空」，[`ModelRoot::user_visible_root_relocations`]）。
    /// `intent_after_the_batch` 说这一批之后意图照抄还是同一次发布删掉（这一批搬空了 R）。
    /// 用户改动发成之后修只坏一份的那一批（`mounted_session::RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing`，D4（校验和位置） 已定项 4）走同一条搬迁路径，也是这一种：
    /// 意图照抄，读到实例表片只坏一份时同一次发布重写整条实例表链（`instance_table`）；整理的批恒照抄实例表。
    CompactionBatch {
        units_at_most: u64,
        intent_after_the_batch: ModelCompactionIntentAfterTheBatch,
        instance_table: ModelInstanceTableOfTheBatch,
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
    /// 不重写尾单元而上段叶条目的标签要变时（缩短把尾部的洞整个截掉：标签 3、4 换回 1、2）只为换标签重写 extent 上段根（
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
pub(super) enum ModelSizeChangeOfAnInodeWithoutDataUnits {
    /// N ≥ S，上段里已有它那一条标签 0、或改完 size 仍是 0（上段里本来没有它，照旧没有）：extent 树整棵照抄。
    WithoutShrinkingCarryingTheExtentTree,
    /// N > S，上段里还没有它那一条（size 0、没改过 size）：这次起写它那一条标签 0、之后一直留着（粘住的标签 0），
    /// extent 上段按这次之后有条目的号重排、它那一串重写。
    WithoutShrinkingWritingTheTagZeroEntry,
    /// N < S：截断意图（K1、K2），logged_ops 签一条截断条目（根这次重写）；extent 树整棵照抄（一个单元都没有：尾单元不重写、标签留 0）。
    TruncationIntent,
}

/// 删除那几步删的是哪个号（判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M1、M3：意图、批、最后一批对任一号）。
/// 封闭集合：第一个文件的内容、单元与截断字段记在 [`ModelVersionContents`] 里，别的文件记在 [`ModelRoot::inodes_other_than_the_first`] 里，
/// 两处各走各的。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ModelDeletedInode {
    FirstFile,
    OtherThanTheFirst(u64),
}

impl ModelDeletedInode {
    pub(super) fn of_the_inode_number(inode: u64) -> Self {
        if inode == FIRST_INODE_NUMBER {
            ModelDeletedInode::FirstFile
        } else {
            ModelDeletedInode::OtherThanTheFirst(inode)
        }
    }

    pub(super) fn inode_number(self) -> u64 {
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

/// 上段叶条目的标签（D8（核心索引结构） 已定项 14；判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K5 加 3、4 声明尾部的洞）。
/// 模型按条款自己写一份，不引实现的常量（D13（验证路线） 已定项 5）。封闭集合。
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

    /// 这个标签与存在的单元数、size 罩的单元数对不对得上（的 T-3，模型这一侧的写法，第六节 S1 收成唯一）：标签说的内联 / 下段
    /// 与存在的单元数相符；标签 1、2 不许有洞（单元数等于 size 罩的），标签 3、4、0 至少有一个洞单元（少于它）；有截断条目时 size 之外可以
    /// 多挂着单元，那时 size 之内没有洞，只许标签 1、2（size 之外挂着的不算洞单元）。
    pub(super) fn admits(
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

/// 搬迁的一批里实例表怎样（`transaction::publish_the_relocations_and_the_repair_of_the_exempt_units`）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelInstanceTableOfTheBatch {
    /// 照抄上一版那条链。
    Carried,
    /// 读到实例表片只坏一份：行不变，整条链重写到新落点（D4（校验和位置） 已定项 4「豁免映射的三类」）。
    RewrittenWithTheSameRows,
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
    pub(super) fn asks_the_snapshot_lifetime_judgement(self) -> bool {
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
pub(super) fn inode_leaf_container_position_of_a_new_record(records_before: u64) -> u64 {
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
    pub(super) fn leaves_an_inode_number_watermark_step(self) -> bool {
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
    pub(super) fn rewrites_the_instance_table(self) -> bool {
        match self {
            ModelPublishKind::RowsOnFileVersion
            | ModelPublishKind::RowsOnVersionWithoutFile
            | ModelPublishKind::CompactionBatch {
                instance_table: ModelInstanceTableOfTheBatch::RewrittenWithTheSameRows,
                ..
            } => true,
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
            | ModelPublishKind::CompactionBatch {
                instance_table: ModelInstanceTableOfTheBatch::Carried,
                ..
            }
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
    pub(super) fn transactions_carried(self, data_units_of_the_new_content: u64) -> u64 {
        match self {
            // 按 inode 号整份写第一个文件之外的文件同一条切分纪律：`data_units_of_the_new_content` 是那个文件这次写的单元数。
            ModelPublishKind::FirstFileVersion
            | ModelPublishKind::OverwriteFileVersion
            | ModelPublishKind::WholeWriteOfAnInodeOtherThanTheFirst { .. } => {
                data_units_of_the_new_content
            }
            // 按偏移写同一条切分纪律（C310「一条记录装一个事务」）：这次重写的每个单元一个事务。
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
    pub(super) fn removed_roles(self) -> &'static [ModelUnitRole] {
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
    pub(super) fn rewritten_roles(self) -> Vec<ModelUnitRole> {
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
            // 修只坏一份的那一批重写实例表（`rewrites_the_instance_table`）：种类定死的那几个是搬迁共用的固定点单元，实例表在这里另加。
            ModelPublishKind::CompactionBatch {
                instance_table: ModelInstanceTableOfTheBatch::RewrittenWithTheSameRows,
                ..
            } => roles.push(ModelUnitRole::InstanceTable),
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
            | ModelPublishKind::CompactionBatch {
                instance_table: ModelInstanceTableOfTheBatch::Carried,
                ..
            }
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
    pub(super) fn rewritten_roles_fixed_by_the_kind(self) -> &'static [ModelUnitRole] {
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
            // （C512（树表 0 条的一版上被换下的实例表记在哪没有条款）：根指针住根记录、不进树表）。
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
            // 把尾部的洞整个截掉、尾单元不用重写：只为换标签重写 extent 上段根。
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

#[cfg(test)]
mod tests {
    use super::*;

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

    /// 上段叶条目的标签（第六节 S1 收成唯一）：一个单元内联、多个单元下段根，存在的少于 size 罩的就是尾部有洞（标签 3、4、0）；
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

    /// 搬迁的一批只有换实例表那一种才重写实例表：角色记这次写，第 1 片起的占槽也算（`rewrites_the_instance_table`）；照抄实例表的整理的批两样都不沾。
    #[test]
    fn only_a_batch_repairing_the_instance_table_rewrites_the_instance_table() {
        let batch_with = |instance_table| ModelPublishKind::CompactionBatch {
            units_at_most: 1,
            intent_after_the_batch: ModelCompactionIntentAfterTheBatch::CarriedOn,
            instance_table,
        };
        let repairing = batch_with(ModelInstanceTableOfTheBatch::RewrittenWithTheSameRows);
        assert!(repairing.rewrites_the_instance_table());
        assert!(repairing
            .rewritten_roles()
            .contains(&ModelUnitRole::InstanceTable));
        let carrying = batch_with(ModelInstanceTableOfTheBatch::Carried);
        assert!(!carrying.rewrites_the_instance_table());
        assert!(!carrying
            .rewritten_roles()
            .contains(&ModelUnitRole::InstanceTable));
    }
}
