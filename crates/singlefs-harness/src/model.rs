//! 理想模型（里程碑「第二个事务」增补 3 第 2 件）：一个只住内存的模型，记每一版提交的内容、每条根属于哪个实例、实例表的行、回退下界 F，
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
//! 回收门槛差一、释放时不改写记录这几条变异，只留模型时三段都判不出，checker 都判红）。分配记录的真条数模型同样不记：分配记录墙拒时，
//! 执行器按 checker 的解析从镜像上现数，交给模型判区间的下端（同一判决第三节第 1 条）。

use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use singlefs_format::{
    index_node_header_bytes, ACCOUNTING_ENTRY_BYTES, ACCOUNTING_KEY_BYTES, ALLOCATION_RECORD_BYTES,
    ALLOCATION_RECORD_KEY_BYTES, CLUSTER_SEGMENT_SLOTS, DATA_UNIT_BYTES, DATA_UNIT_PAYLOAD_OFFSET,
    INSTANCE_TABLE_PAGE_RECORDS, NODE_BYTES, ROOT_RING_REGIONS, ROOT_RING_REGION_DEVICES,
    ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM, SLOT_BYTES, UNIT_AREA_START_SLOT,
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

/// 一版里的一个角色：第一个文件版本的八个单元（D3（空间分配） 已定项 10 ⑤ 的次序）与实例表单元（D18（块里携带什么信息） 已定项 11）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModelUnitRole {
    Data,
    ExtentRoot,
    InodeLeaf,
    InodeRoot,
    AllocationTree,
    AccountingTree,
    MappingTree,
    TreeTable,
    InstanceTable,
}

impl ModelUnitRole {
    /// 占几个槽：数据单元与打包记录单元（inode 叶、实例表）恒 32 KiB（D4（校验和位置） 已定项 5），索引节点恒 16 KiB
    /// （D8（核心索引结构） 已定项 2），落点粒度 16 KiB（D3（空间分配） 已定项 7）。
    #[must_use]
    pub fn span_in_slots(self) -> u64 {
        match self {
            ModelUnitRole::Data | ModelUnitRole::InodeLeaf | ModelUnitRole::InstanceTable => {
                DATA_UNIT_BYTES / SLOT_BYTES
            }
            ModelUnitRole::ExtentRoot
            | ModelUnitRole::InodeRoot
            | ModelUnitRole::AllocationTree
            | ModelUnitRole::AccountingTree
            | ModelUnitRole::MappingTree
            | ModelUnitRole::TreeTable => NODE_BYTES / SLOT_BYTES,
        }
    }
}

/// 一次发布重写哪些角色。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ModelPublishKind {
    /// 第一个文件版本：四个文件角色，加四个固定点单元（D16（发布语义） 已定项 9：记账行每发布重写，连带记账树节点、分配记录、
    /// 映射条目与树表单元）。与覆盖写重写的角色相同，分开只为一件事——它走 `previous: None`，这一版的内存态从头建。
    FirstFileVersion,
    /// 覆盖写：与第一个文件版本重写的角色相同，接在上一版的内存态后面。
    OverwriteFileVersion,
    /// 写行：实例表单元（D18（块里携带什么信息） 已定项 11：每次可写挂载都写行）加四个固定点单元（D16（发布语义） 已定项 9）。
    RowsOnFileVersion,
    /// 树表不是 0 条时的空发布（暖机、抬 F）：四个固定点单元（D16（发布语义） 已定项 9）。
    EmptyOnFileVersion,
    /// 树表 0 条的一版上写行：只重写实例表这一个单元（D18（块里携带什么信息） 已定项 11 要求每次可写挂载都写行；
    /// 这一版没有记账树，D16（发布语义） 已定项 9 的那五样——记账行、记账树节点、分配记录、映射条目、树表单元——一样都不写）。
    RowsOnVersionWithoutFile,
    /// 树表 0 条上的空发布：零单元（D16（发布语义） 已定项 9「树表 0 条 ⇒ 零单元」）。
    ZeroUnit,
}

impl ModelPublishKind {
    /// 这次发布往分配记录树里加几条（每块盘一条）：重写的每个角色各一条。
    /// 零单元发布一个字节都不写，一条不加；树表 0 条的一版上写行那次**建起这一版自己的分配记录树**
    /// （C512（树表 0 条的一版上被换下的单元记在哪），2026-09-23 用户定案），实例表与那片节点各一条。
    fn allocation_records_added_per_device(self) -> u64 {
        match self {
            ModelPublishKind::FirstFileVersion
            | ModelPublishKind::OverwriteFileVersion
            | ModelPublishKind::RowsOnFileVersion
            | ModelPublishKind::EmptyOnFileVersion
            | ModelPublishKind::RowsOnVersionWithoutFile => {
                u64::try_from(self.rewritten_roles().len()).expect("至多九个角色")
            }
            ModelPublishKind::ZeroUnit => 0,
        }
    }

    fn rewritten_roles(self) -> &'static [ModelUnitRole] {
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

/// 实例表的一行 (i, T, W) 与回退标志（D18（块里携带什么信息） 已定项 11；D23（journal 的角色与格式） 已定项 14）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModelInstanceRow {
    pub instance: ModelInstanceGeneration,
    pub selected_root_txg: ModelCheckpointTxg,
    pub applied_transaction_high_water: u64,
    pub is_rollback: bool,
}

/// 一条已提交的根与它指着的那一版。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelRoot {
    pub key: ModelRootKey,
    pub journal_counter: ModelJournalCounter,
    pub rollback_floor: ModelCheckpointTxg,
    /// 树表 0 条（第 0 代根、暖机根、零单元发布）时没有文件。
    pub file: Option<ModelFileVersion>,
    pub instance_table_rows: Rc<Vec<ModelInstanceRow>>,
    /// 这一版每个角色的单元是哪次发布写的：它的分配记录的分配代就是这个 txg（D3（空间分配） 已定项 3 / 7：value = 分配代；
    /// COW：单元只在分配它的那次发布里写）。mkfs 写的实例表与第 0 版树表记 txg 0。
    pub role_written_at: BTreeMap<ModelUnitRole, ModelCheckpointTxg>,
    /// 这一版之后分配记录条数的上界：mkfs 两个单元每盘各一条，沿这一版的来路每次发布每个重写的角色每盘至多加一条
    /// （D3（空间分配） 已定项 7：一条记一个单元、释放只改写不删）。回收、复用只会让真数比它小。
    pub allocation_records_upper_bound: u64,
    /// 写出这一版的那次发布按准入的口径新增几条分配记录：重写的每个角色每盘一条，不抵扣会被复用的已回收记录（增补 2 收口表第 39 行，
    /// 2026-09-18 用户定保留这个上界准入）。第 0 代根记 mkfs 写的两个单元每盘一条。
    pub allocation_records_added_by_its_publish: u64,
    /// 同样口径的每盘已占槽数上界：沿来路每次发布写出的单元的槽数之和，释放与回收一概不扣。
    pub occupied_slots_upper_bound_per_device: u64,
}

impl ModelRoot {
    /// 这条根的用户可见树（inode 树、extent 树）的根指针是谁写的：树表 0 条时 None。
    fn user_visible_trees_written_by(&self) -> Option<ModelRootKey> {
        self.file.as_ref().map(|file| file.written_by)
    }
}

/// 池的几何：盘的身份与每块盘的字节数（与实现的 mkfs 参数由调用方各给一份）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelPoolGeometry {
    pub devices: Vec<ModelDeviceIdentity>,
    pub device_size_in_bytes: u64,
}

/// 根环里的一个槽：第 n 次发布写区域 `n mod R` 的槽 `(n div R) mod S`（D22（单元原子性怎么合成） 已定项 2 / 已定项 16）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ModelRingPosition {
    region: u64,
    slot_in_region: u64,
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
    /// 分配记录树第一版只有一个节点（容量墙，收口表第 39 行：模型答允许拒绝的区间）。
    AllocationRecordNodeWall,
    /// 记账树第一版只有一个节点。
    AccountingNodeWall,
    /// 单元区装不下（D28（挂载期承诺量） 已定项 1 的准入；模型答允许拒绝的区间）。
    UnitAreaWall,
    /// 实例表这次之后要多于一片（D18（块里携带什么信息） 已定项 11：一片 370 条含链指针）：第二片在 bump 次序里怎么排、
    /// 行怎么分片没有条款（D3（空间分配） 已定项 10 ⑤ 只写「实例表单元最前」），第一版不写第二片。
    InstanceTableChainLongerThanOnePageUndecided,
    /// 回退目标不在根环里（D23（journal 的角色与格式） 已定项 14：候选集是根环里的根）。
    RollbackTargetNotInRing,
    /// 回退目标的 txg 低于 F_生效（D16（发布语义） 已定项 1「回退候选集」）。
    RollbackTargetBelowEffectiveFloor,
    /// 回退目标在被抛弃的时间线上（D23（journal 的角色与格式） 已定项 14：有行 (i, Ti, Wi) 且 T > Ti）。
    RollbackTargetOnAbandonedTimeline,
    /// 要建立新实例的那一版树表 0 条、而它的实例表已经不是 mkfs 那一片（上一次挂载在这一版上写过行）：
    /// 被换下的那一片记在哪没有条款——树表 0 条 ⇒ 这一版没有分配记录树，那次释放只住内存，重开之后账取不回来，
    /// 而根环里更旧的候选根还指着它。第一版不支持（设计空白，交主 agent）。
    VersionWithoutFileNotWrittenByMakeFilesystem,
    /// 要抬的 F 超过上限（D16（发布语义） 已定项 1「抬 F 的上限」）。
    FloorAboveCeiling,
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
            ModelRefusalReason::AllocationRecordNodeWall => "分配记录树一个节点装不下",
            ModelRefusalReason::AccountingNodeWall => "记账树一个节点装不下",
            ModelRefusalReason::UnitAreaWall => "单元区装不下",
            ModelRefusalReason::InstanceTableChainLongerThanOnePageUndecided => {
                "实例表要多于一片（第二片怎么写没有条款）"
            }
            ModelRefusalReason::RollbackTargetNotInRing => "回退目标不在根环里",
            ModelRefusalReason::RollbackTargetBelowEffectiveFloor => "回退目标低于 F_生效",
            ModelRefusalReason::RollbackTargetOnAbandonedTimeline => "回退目标在被抛弃的时间线上",
            ModelRefusalReason::VersionWithoutFileNotWrittenByMakeFilesystem => {
                "树表 0 条、而实例表已经不是 mkfs 那一片（条款没写被换下的那一片记在哪）"
            }
            ModelRefusalReason::FloorAboveCeiling => "要抬的 F 超过上限",
        }
    }

    /// 容量墙：模型只答允许拒绝的区间（按这一步计划的每次发布之后的上界判），不答必须拒。
    fn is_capacity_wall_with_an_interval(self) -> bool {
        match self {
            ModelRefusalReason::AllocationRecordNodeWall | ModelRefusalReason::UnitAreaWall => true,
            ModelRefusalReason::ContentExceedsDataUnitPayload
            | ModelRefusalReason::FirstFileVersionDoesNotFollowTheVersionItBuildsOn
            | ModelRefusalReason::FirstFileVersionOnAVersionThatAlreadyHasAFile
            | ModelRefusalReason::AccountingNodeWall
            | ModelRefusalReason::InstanceTableChainLongerThanOnePageUndecided
            | ModelRefusalReason::RollbackTargetNotInRing
            | ModelRefusalReason::RollbackTargetBelowEffectiveFloor
            | ModelRefusalReason::RollbackTargetOnAbandonedTimeline
            | ModelRefusalReason::VersionWithoutFileNotWrittenByMakeFilesystem
            | ModelRefusalReason::FloorAboveCeiling => false,
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
    PublishWithoutUnits,
    MountWritable,
    MountRollback,
    RaiseRollbackFloor,
    ColdStartRecover,
}

/// 一步里计划的一次发布之后的两个上界（容量墙区间的上端按它判），与这次发布按准入口径新增的分配记录条数（分配记录墙区间的下端：
/// 从镜像上现数的基数加上它）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PlannedPublishUpperBounds {
    allocation_records: u64,
    allocation_records_added_by_the_admission: u64,
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
    planned_publish_upper_bounds: Vec<PlannedPublishUpperBounds>,
    /// 容量墙在第一次写之前一串判完（可写挂载、回退）还是逐次发布判（发布、抬 F）。
    walls_are_judged_before_the_first_write: bool,
    /// 这一步接在哪一版后面：发布与抬 F 是会话的现行版本，可写挂载是所选根，回退是目标根；冷启动与候选集外的回退没有。
    starting_version: Option<ModelRootKey>,
    session_after_success: ModelSessionAfterSuccess,
}

impl ModelAnswer {
    /// 分配记录墙拒在做完 `publishes_completed` 次发布之后时，执行器在镜像上数哪一条根下的分配记录：逐次判的（发布、抬 F）数拒之前
    /// 最后写出的那一版（一次都没做完就是这一步的起点），一串判完的（可写挂载、回退）数这一步的起点——实现的准入基数就是这两处的
    /// 分配器条数（`transaction::publish_admission`、`transaction::publish_sequence_admission`）。
    #[must_use]
    pub fn root_whose_allocation_records_the_wall_counts(
        &self,
        publishes_completed: usize,
    ) -> Option<ModelRootKey> {
        if self.walls_are_judged_before_the_first_write || publishes_completed == 0 {
            return self.starting_version;
        }
        self.expected_roots
            .get(publishes_completed - 1)
            .map(|root| root.key)
    }
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

/// 实现写出的一条根，胶水从实现交回的东西里取。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservedRoot {
    pub key: ModelRootKey,
    pub journal_counter: ModelJournalCounter,
    pub rollback_floor: ModelCheckpointTxg,
    pub has_file: bool,
    /// 这一版每个单元（实现交回的那几个角色）的分配记录，每块盘各一条。树表 0 条的一版为空。
    pub unit_allocation_records: Vec<(ModelUnitRole, Vec<ObservedAllocationRecord>)>,
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
        /// 理由是分配记录墙时，执行器按 checker 的解析在镜像上数的准入基数：`ModelAnswer::root_whose_allocation_records_the_wall_counts`
        /// 点名的那条根下有几条分配记录。别的理由不数；数不出（镜像上找不到那条根、树读不出）也是 None，这时模型不放行分配记录墙。
        allocation_records_counted_on_the_image: Option<u64>,
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
}

impl ModelDisagreement {
    fn new(
        aspect: ModelDisagreementAspect,
        model_answer: String,
        implementation_answer: String,
    ) -> Self {
        Self {
            aspect,
            model_answer,
            implementation_answer,
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
    pub cold_start_contents_compared: u64,
    pub ceilings_compared: u64,
    /// 回退做成、目标的 txg 正好等于 F_生效且 F_生效 > 0（B2 那一格跑到了）。
    pub rollbacks_accepted_at_the_effective_floor: u64,
    /// 分配记录墙拒、镜像上数的真条数越过一个节点而放行的次数（墙那一格的下端真的判过）。
    pub allocation_record_wall_refusals_over_one_node: u64,
    /// 单元区墙拒（实现报每块盘上都没有合政策的落点）、在允许拒绝的区间里而放行的次数（小盘那一段的落点拒绝真的走到了、判过）。
    pub unit_area_wall_refusals_in_the_interval: u64,
}

impl ModelJudgementCounts {
    /// 把另一份逐项加进来。
    pub fn add(&mut self, other: &ModelJudgementCounts) {
        self.required_refusals_matched += other.required_refusals_matched;
        self.permitted_refusals_taken += other.permitted_refusals_taken;
        self.successes_matched += other.successes_matched;
        self.roots_compared += other.roots_compared;
        self.allocation_records_compared += other.allocation_records_compared;
        self.cold_start_contents_compared += other.cold_start_contents_compared;
        self.ceilings_compared += other.ceilings_compared;
        self.rollbacks_accepted_at_the_effective_floor +=
            other.rollbacks_accepted_at_the_effective_floor;
        self.allocation_record_wall_refusals_over_one_node +=
            other.allocation_record_wall_refusals_over_one_node;
        self.unit_area_wall_refusals_in_the_interval +=
            other.unit_area_wall_refusals_in_the_interval;
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
    ring: BTreeMap<ModelRingPosition, ModelRoot>,
    /// 取过的最大实例代号（系统配置里的号；取号先于任何带新号的写，D23（journal 的角色与格式） 已定项 16）。
    highest_acquired_instance: ModelInstanceGeneration,
    /// 发布过的最大 txg 与 jsn：不建崩溃，环里全部根与全部记录里的最大值就是它们（D23（journal 的角色与格式） 已定项 14 第 3 条）。
    highest_published_txg: ModelCheckpointTxg,
    highest_journal_counter: ModelJournalCounter,
    session: Option<ModelSession>,
}

impl IdealModel {
    /// mkfs 之后：根环里只有第 0 代根（实例 0、txg 0、F 0、树表 0 条、实例表没有行），mkfs 写了实例表与第 0 版树表两个单元，
    /// 没有可写会话。
    #[must_use]
    pub fn after_make_filesystem(geometry: ModelPoolGeometry) -> Self {
        let device_count = u64::try_from(geometry.devices.len()).expect("盘数");
        let genesis = ModelRoot {
            key: ModelRootKey {
                checkpoint_txg: MAKE_FILESYSTEM_TXG,
                instance: MAKE_FILESYSTEM_INSTANCE,
            },
            journal_counter: ModelJournalCounter(0),
            rollback_floor: ModelCheckpointTxg(0),
            file: None,
            instance_table_rows: Rc::new(Vec::new()),
            role_written_at: BTreeMap::from([
                (ModelUnitRole::InstanceTable, MAKE_FILESYSTEM_TXG),
                (ModelUnitRole::TreeTable, MAKE_FILESYSTEM_TXG),
            ]),
            allocation_records_upper_bound: 2 * device_count,
            allocation_records_added_by_its_publish: 2 * device_count,
            occupied_slots_upper_bound_per_device: ModelUnitRole::InstanceTable.span_in_slots()
                + ModelUnitRole::TreeTable.span_in_slots(),
        };
        let mut ring = BTreeMap::new();
        ring.insert(ring_position_of(MAKE_FILESYSTEM_TXG), genesis);
        Self {
            geometry,
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
                    root.file.as_ref().map(|file| Rc::clone(&file.content)),
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

    /// F_生效 = 各幸存盘所带 F 最大值的最小值；一块盘上一条根都没有就不算它，一条根都没有时 0。
    /// 预想，跟收口表第 ② 行（「生效」那一行 2026-09-17 打回重议，定案之前照字面；被抛弃时间线上的根带的 F 也算，照字面「持久根」）。
    #[must_use]
    pub fn effective_rollback_floor(&self) -> ModelCheckpointTxg {
        let mut highest_floor_per_device: BTreeMap<ModelDeviceIdentity, ModelCheckpointTxg> =
            BTreeMap::new();
        for root in self.ring.values() {
            let highest = highest_floor_per_device
                .entry(device_holding_the_root_of(root.key.checkpoint_txg))
                .or_insert(root.rollback_floor);
            *highest = (*highest).max(root.rollback_floor);
        }
        highest_floor_per_device
            .values()
            .copied()
            .min()
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
    /// 有效 = 按最新根的实例表不被抛弃 ∧ txg ≥ 现行的 F（「非空」从盘上怎么认，2026-09-17 用户定案）；现行的 F 取 F_生效（不建崩溃时与
    /// 现行根带的相等）。非空 = inode 树、extent 树的根指针与前一条有效根（按 (txg, 实例) 排）的不同，最旧的有效根与「没有这两棵树」比。
    /// 一块盘上没有有效根时不算它（条款没写这一格；不建崩溃时抬 F 与暖机都推到每块盘都有，走不到）。预想，跟收口表第 ② 行
    /// （候选集下界用哪个 F、被抛弃根带的 F 算不算都在那一行里重议）。
    #[must_use]
    pub fn rollback_floor_ceiling(&self) -> Option<ModelCheckpointTxg> {
        let current_floor = self.effective_rollback_floor();
        let newest_table = Rc::clone(&self.newest_root().instance_table_rows);
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
        let mut non_empty_txgs: Vec<ModelCheckpointTxg> = Vec::new();
        let mut previous_trees_written_by: Option<ModelRootKey> = None;
        for root in &valid {
            let trees_written_by = root.user_visible_trees_written_by();
            if trees_written_by != previous_trees_written_by {
                non_empty_txgs.push(root.key.checkpoint_txg);
            }
            previous_trees_written_by = trees_written_by;
        }
        non_empty_txgs.sort_unstable_by(|left, right| right.cmp(left));
        let fourth_newest_non_empty = match non_empty_txgs.get(3) {
            Some(fourth) => *fourth,
            None => valid.first()?.key.checkpoint_txg,
        };
        Some(newest_on_every_device.min(fourth_newest_non_empty))
    }

    /// 分配记录树一个节点装几条（D3（空间分配） 已定项 11：key 10、条目 20）。
    fn allocation_record_node_capacity() -> u64 {
        index_node_entry_capacity(ALLOCATION_RECORD_KEY_BYTES, ALLOCATION_RECORD_BYTES)
    }

    /// 一次写记账行的发布要几行、一个节点装几行（D5（快照 / 空间记账机制） 已定项 8）。
    fn accounting_rows_exceed_one_node(&self) -> bool {
        let device_count = u64::try_from(self.geometry.devices.len()).expect("盘数");
        POOL_WIDE_ACCOUNTING_ROWS + ACCOUNTING_ROWS_PER_DEVICE * device_count
            > index_node_entry_capacity(ACCOUNTING_KEY_BYTES, ACCOUNTING_ENTRY_BYTES)
    }

    /// 单元区每块盘的槽数。
    fn unit_area_slots_per_device(&self) -> u64 {
        (self.geometry.device_size_in_bytes / SLOT_BYTES).saturating_sub(UNIT_AREA_START_SLOT)
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
        let key = ModelRootKey {
            checkpoint_txg,
            instance,
        };
        let device_count = u64::try_from(self.geometry.devices.len()).expect("盘数");
        let mut role_written_at = previous.role_written_at.clone();
        let mut slots_written = 0;
        let rewritten = kind.rewritten_roles();
        for role in rewritten {
            role_written_at.insert(*role, checkpoint_txg);
            slots_written += role.span_in_slots();
        }
        let allocation_records_added = kind.allocation_records_added_per_device() * device_count;
        let file = match new_file_content {
            Some(content) => Some(ModelFileVersion {
                written_by: key,
                content: Rc::from(content),
            }),
            None => previous.file.clone(),
        };
        ModelRoot {
            key,
            journal_counter,
            rollback_floor,
            file,
            instance_table_rows,
            role_written_at,
            allocation_records_upper_bound: previous.allocation_records_upper_bound
                + allocation_records_added,
            allocation_records_added_by_its_publish: allocation_records_added,
            occupied_slots_upper_bound_per_device: previous.occupied_slots_upper_bound_per_device
                + slots_written,
        }
    }

    fn upper_bounds_of(roots: &[ModelRoot]) -> Vec<PlannedPublishUpperBounds> {
        roots
            .iter()
            .map(|root| PlannedPublishUpperBounds {
                allocation_records: root.allocation_records_upper_bound,
                allocation_records_added_by_the_admission: root
                    .allocation_records_added_by_its_publish,
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
        // 第一个文件版本接在现行那一版后面：txg 与 jsn 各加一，不取 `FIRST_TRANSACTION_TXG`——那个常量只管 mkfs
        // 同一个进程里那条流（2026-09-23 用户定案）。执行器恒拿现行那一版的根与记录，所以
        // `FirstFileVersionDoesNotFollowTheVersionItBuildsOn` 在随机历史里走不到、模型一次都不要求它。
        // 现行那一版已经有过文件版本：这条路径按「树还没建起来」写，再走一次会重新建树、重新发对象出生代，
        // 与环里那些旧根记的对不上（I-9.14、I-9.10）——再写一版走覆盖写。
        if current.file.is_some() {
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
            current.key,
            required_refusals,
            BTreeSet::new(),
            vec![root],
        ))
    }

    /// 覆盖写（`transaction::publish_overwrite`）：接在现行的带文件的一版后面。
    ///
    /// # Errors
    /// 模型里没有可写会话，或现行版本没有文件（执行器只在带文件时调）：会话状态与实现对不上。
    pub fn answer_publish_overwrite(
        &self,
        content: &[u8],
    ) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        if current.file.is_none() {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::SessionState,
                format!("模型里现行版本 {:?} 没有文件", current.key),
                "实现的现行版本带文件（执行器调了覆盖写）".to_string(),
            ));
        }
        let mut required_refusals = BTreeSet::new();
        if u64::try_from(content.len()).expect("内容长度") > data_unit_payload_capacity_in_bytes()
        {
            required_refusals.insert(ModelRefusalReason::ContentExceedsDataUnitPayload);
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
        Ok(self.answer_for_publishes(
            ModelOperationKind::PublishOverwrite,
            current.key,
            required_refusals,
            BTreeSet::new(),
            vec![root],
        ))
    }

    /// 零单元发布（`transaction::publish_without_units`）：接在现行的树表 0 条的一版后面。
    ///
    /// # Errors
    /// 模型里没有可写会话，或现行版本带文件（执行器只在树表 0 条时调）：会话状态与实现对不上。
    pub fn answer_publish_without_units(&self) -> Result<ModelAnswer, ModelDisagreement> {
        let session = self.open_session()?;
        let current = &session.current;
        if current.file.is_some() {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::SessionState,
                format!("模型里现行版本 {:?} 带文件", current.key),
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
            current.key,
            BTreeSet::new(),
            BTreeSet::new(),
            vec![root],
        ))
    }

    fn answer_for_publishes(
        &self,
        operation: ModelOperationKind,
        starting_version: ModelRootKey,
        mut required_refusals: BTreeSet<ModelRefusalReason>,
        permitted_refusals: BTreeSet<ModelRefusalReason>,
        roots: Vec<ModelRoot>,
    ) -> ModelAnswer {
        // 这一步里有一次发布重写了记账树（零单元发布不写记账行，别的都写，D16（发布语义） 已定项 9）。
        let writes_accounting_rows = roots.iter().any(|root| {
            root.role_written_at.get(&ModelUnitRole::AccountingTree)
                == Some(&root.key.checkpoint_txg)
        });
        if writes_accounting_rows && self.accounting_rows_exceed_one_node() {
            required_refusals.insert(ModelRefusalReason::AccountingNodeWall);
        }
        ModelAnswer {
            operation,
            required_refusals,
            permitted_refusals,
            planned_publish_upper_bounds: Self::upper_bounds_of(&roots),
            expected_roots: roots,
            expected_mount: None,
            expected_read_back: None,
            rollback_floor_ceiling: None,
            walls_are_judged_before_the_first_write: false,
            starting_version: Some(starting_version),
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
            is_rollback: false,
        };
        self.answer_establishing_an_instance(
            ModelOperationKind::MountWritable,
            &chosen,
            Some(previous_row),
        )
    }

    /// 管理员回退（`mount::mount_rollback`，D23（journal 的角色与格式） 已定项 14 的显式例外）。调之前先 `close_session`。
    #[must_use]
    pub fn answer_mount_rollback(&self, target: ModelRootKey) -> ModelAnswer {
        let refused = |reasons: BTreeSet<ModelRefusalReason>| ModelAnswer {
            operation: ModelOperationKind::MountRollback,
            required_refusals: reasons,
            permitted_refusals: BTreeSet::new(),
            expected_roots: Vec::new(),
            expected_mount: None,
            expected_read_back: None,
            rollback_floor_ceiling: None,
            planned_publish_upper_bounds: Vec::new(),
            walls_are_judged_before_the_first_write: true,
            starting_version: None,
            session_after_success: ModelSessionAfterSuccess::Closed,
        };
        let Some(target_root) = self.root_with_key(target) else {
            return refused(BTreeSet::from([
                ModelRefusalReason::RollbackTargetNotInRing,
            ]));
        };
        let mut reasons = BTreeSet::new();
        if target.checkpoint_txg < self.effective_rollback_floor() {
            reasons.insert(ModelRefusalReason::RollbackTargetBelowEffectiveFloor);
        }
        if Self::is_abandoned_by(target_root, &self.newest_root().instance_table_rows) {
            reasons.insert(ModelRefusalReason::RollbackTargetOnAbandonedTimeline);
        }
        // 候选集只有上面那三条（D23（journal 的角色与格式） 已定项 14）：目标那一版树表 0 条不是排除项，
        // 环里还留着带文件版本的根时也照常回退（C511（回退到无文件那一版之后诞生代怎么接） 第 3 步：回退那一版带环里的树 ID 水位 max，
        // 再发第一个文件版本从它往上发号；I-9.14 只比同一条时间线上的根）。
        if !reasons.is_empty() {
            return refused(reasons);
        }
        let previous_row = ModelInstanceRow {
            instance: target.instance,
            selected_root_txg: target.checkpoint_txg,
            applied_transaction_high_water: 0,
            is_rollback: true,
        };
        self.answer_establishing_an_instance(
            ModelOperationKind::MountRollback,
            &target_root.clone(),
            Some(previous_row),
        )
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
        // 行：给 [max(上一个实例, 1), 新实例) 里每个实例一行——上一个实例（或被退回的实例）那一行，中间实例 (i, 0, 0)；实例 0 不写行
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
                            is_rollback: false,
                        }
                    }
                })
                .collect(),
        };
        // 新实例的第一次发布的 txg 与 jsn：环里全部根与全部记录里的最大值 + 1（D23（journal 的角色与格式） 已定项 14 第 3 条）。
        // 回退的 jsn 接在环里最大的 jsn 之后（C340 取 P2）：预想，跟收口表第 ① 行。
        let first_txg = ModelCheckpointTxg(self.highest_published_txg.0 + 1);
        let first_journal_counter = ModelJournalCounter(self.highest_journal_counter.0 + 1);
        // 新实例的根带的 F = 恢复后生效的 F（D16（发布语义） 已定项 1「生效」）：预想，跟收口表第 ② 行。
        let rollback_floor = self.effective_rollback_floor();
        let mut required_refusals = BTreeSet::new();
        let has_file = base.file.is_some();
        // 树表 0 条、而这一版的实例表已经不是 mkfs 那一片（上一次挂载在这一版上写过行）**此前是必拒的一格**：
        // 重建账时只剩根记录那两条指针，被换下的那一片成了空闲槽。C512（树表 0 条的一版上被换下的单元记在哪）
        // 2026-09-23 定案之后，写行那次发布建起这一版自己的分配记录树、根指针住根记录 ⇒ 账取得回来，这一格不再拒。
        // 模型因此**一条都不列**：实现要是还在这一格上拒，对拍当场报「模型说该成、实现拒了」——
        // 那正是这条定案要盯住的回退面。`MountError::VersionWithoutFileNotWrittenByMakeFilesystem` 今天只剩
        // 「树表或实例表不是 mkfs 写的那一版、而这一版又没有自己的分配记录树」那种手造镜像走得到，随机历史里造不出来。
        let rows_after =
            u64::try_from(base.instance_table_rows.len() + rows_to_write.len()).expect("行数");
        // 一片 370 条，链指针记录恒为一片的最后一条（D18（块里携带什么信息） 已定项 11）；这次之后要多于一片时，第二片怎么写没有条款，
        // 第一版不写第二片。两臂都判：树表 0 条的一版上写行同样重写整张实例表。
        if rows_after + 1 > INSTANCE_TABLE_PAGE_RECORDS {
            required_refusals
                .insert(ModelRefusalReason::InstanceTableChainLongerThanOnePageUndecided);
        }
        let table_after: Rc<Vec<ModelInstanceRow>> = if rows_to_write.is_empty() {
            Rc::clone(&base.instance_table_rows)
        } else {
            let mut rows = base.instance_table_rows.as_ref().clone();
            rows.extend_from_slice(&rows_to_write);
            Rc::new(rows)
        };
        let (row_kind, warm_up_kind) = match (has_file, rows_to_write.is_empty()) {
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
        let mut answer = self.answer_for_publishes(
            operation,
            base.key,
            required_refusals,
            BTreeSet::new(),
            roots,
        );
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
        if current.file.is_none() {
            return Err(ModelDisagreement::new(
                ModelDisagreementAspect::SessionState,
                format!("模型里现行版本 {:?} 没有文件", current.key),
                "实现的现行版本带文件（执行器调了抬 F）".to_string(),
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
        let mut roots: Vec<ModelRoot> = Vec::new();
        let mut covered: BTreeSet<ModelDeviceIdentity> = BTreeSet::new();
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
        let mut answer = self.answer_for_publishes(
            ModelOperationKind::RaiseRollbackFloor,
            current.key,
            required_refusals,
            permitted_refusals,
            roots,
        );
        answer.rollback_floor_ceiling = ceiling;
        Ok(answer)
    }

    /// 冷启动读回（`recovery::recover`，看 journal）：所选根 = 环里 (txg, 实例) 最大的那条，读回它那一版的文件。调之前先 `close_session`。
    #[must_use]
    pub fn answer_cold_start_recover(&self) -> ModelAnswer {
        let newest = self.newest_root();
        let read_back = match &newest.file {
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
            planned_publish_upper_bounds: Vec::new(),
            walls_are_judged_before_the_first_write: true,
            starting_version: None,
            session_after_success: ModelSessionAfterSuccess::Closed,
        }
    }

    /// 容量墙的区间（收口表第 39 行那种：条款把答案留给实现取上界，模型答允许拒绝的区间）：
    /// - 分配记录树（预想，跟收口表第 39 行）：允许拒要两头都过。上界这一头：这一步计划的那次发布之后、沿来路每次发布每个角色每盘都新加一条的
    ///   上界超过一个节点——一条记一个单元、释放只改写不删（D3（空间分配） 已定项 7），条目 20 字节、key 10（D3（空间分配） 已定项 11），
    ///   节点 16 KiB 减头（D8（核心索引结构） 已定项 11）⇒ 812 条；模型的上界沿来路累加、不看分配器此刻的条数，只会比真数宽。
    ///   真条数这一头：执行器按 checker 的解析在镜像上现数的准入基数（`ModelAnswer::root_whose_allocation_records_the_wall_counts` 点名的那一版），
    ///   加上计划里到那一次为止每次按准入口径新增的条数（每个重写的角色每盘一条，2026-09-18 用户定保留的上界准入），超过 812 才算装不下；
    ///   真条数 ≤ 812 而实现拒了是对不上，数不出基数也不放行（增补 3 第 2 件代码三方第一轮判决第三节第 1 条：此前只有上界那一头，
    ///   宽到接得住「差一」的误拒——攻方把墙的 `>` 改成 `>=`，长历史里在条款说装得下的格上拒了 44 次，那一刻上界 1376–1778，一次都没判出）。
    ///   必须拒那一头（真条数装不下而实现做成了）不判：装不下还去写会 panic，由第 1 件判。
    /// - 单元区：真条数那一头不判；上界那一头是占槽上界 × 一个聚簇段的槽数超过单元区（每个落点最坏独占一段：已分配 + defer ≤ 2 × 上界，
    ///   保留池 10 + 7 c_max 与切换预留（D16（发布语义） 已定项 1；D28（挂载期承诺量） 已定项 3）在 64 倍里）。预想：D28 已定项 1 的准入式子
    ///   第一版没实现，各项没有现值。
    fn capacity_wall_is_permitted(
        &self,
        reason: ModelRefusalReason,
        answer: &ModelAnswer,
        publishes_completed: usize,
        allocation_records_counted_on_the_image: Option<u64>,
    ) -> bool {
        // 一串判完的（可写挂载、回退）看计划里的每一次；逐次判的（发布、抬 F）只看拒的那一次。
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
            ModelRefusalReason::AllocationRecordNodeWall => {
                let capacity = Self::allocation_record_node_capacity();
                let upper_bound_exceeds = judged_publishes
                    .iter()
                    .any(|planned| planned.allocation_records > capacity);
                let true_count_exceeds =
                    allocation_records_counted_on_the_image.is_some_and(|counted| {
                        judged_publishes
                            .iter()
                            .scan(counted, |records, planned| {
                                *records += planned.allocation_records_added_by_the_admission;
                                Some(*records)
                            })
                            .any(|records_after_the_publish| records_after_the_publish > capacity)
                    });
                upper_bound_exceeds && true_count_exceeds
            }
            ModelRefusalReason::UnitAreaWall => judged_publishes.iter().any(|planned| {
                planned.occupied_slots_per_device * CLUSTER_SEGMENT_SLOTS
                    > self.unit_area_slots_per_device()
            }),
            ModelRefusalReason::ContentExceedsDataUnitPayload
            | ModelRefusalReason::FirstFileVersionDoesNotFollowTheVersionItBuildsOn
            | ModelRefusalReason::FirstFileVersionOnAVersionThatAlreadyHasAFile
            | ModelRefusalReason::AccountingNodeWall
            | ModelRefusalReason::InstanceTableChainLongerThanOnePageUndecided
            | ModelRefusalReason::RollbackTargetNotInRing
            | ModelRefusalReason::RollbackTargetBelowEffectiveFloor
            | ModelRefusalReason::RollbackTargetOnAbandonedTimeline
            | ModelRefusalReason::VersionWithoutFileNotWrittenByMakeFilesystem
            | ModelRefusalReason::FloorAboveCeiling => false,
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
                allocation_records_counted_on_the_image,
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
                                        *allocation_records_counted_on_the_image,
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
                    let counted_on_the_image = match allocation_records_counted_on_the_image {
                        Some(counted) => format!("（镜像上数的准入基数 {counted} 条）"),
                        None => String::new(),
                    };
                    return Err(ModelDisagreement::new(
                        aspect,
                        describe_answer(answer),
                        format!("拒了：{member}{counted_on_the_image}"),
                    ));
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
                if accepted_reason == ModelRefusalReason::AllocationRecordNodeWall {
                    counts.allocation_record_wall_refusals_over_one_node += 1;
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
            Err(ModelDisagreement::new(
                ModelDisagreementAspect::RollbackFloorCeiling,
                format!(
                    "上限 {:?}（D16（发布语义） 已定项 1）",
                    answer.rollback_floor_ceiling.map(|ceiling| ceiling.0)
                ),
                format!("实现报上限 {}", reported.0),
            ))
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
                    return Err(ModelDisagreement::new(
                        ModelDisagreementAspect::InstanceRows,
                        format!("写行 {expected_rows:?}"),
                        format!("写行 {rows_written:?}"),
                    ));
                }
                if answer.operation == ModelOperationKind::MountRollback {
                    let base_txg = expected_rows
                        .iter()
                        .find(|row| row.is_rollback)
                        .map(|row| row.selected_root_txg);
                    let floor = self.effective_rollback_floor();
                    if floor.0 > 0 && base_txg == Some(floor) {
                        counts.rollbacks_accepted_at_the_effective_floor += 1;
                    }
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
            if expected.file.is_some() != observed.has_file {
                return Err(ModelDisagreement::new(
                    ModelDisagreementAspect::FilePresence,
                    format!("{:?} 有文件：{}", expected.key, expected.file.is_some()),
                    format!("有文件：{}", observed.has_file),
                ));
            }
            self.judge_allocation_generations(expected, observed, counts)?;
        }
        Ok(())
    }

    /// 这一版每个单元的分配记录：每块盘各一条、仍分配着、分配代等于写它的那次发布的 txg（D3（空间分配） 已定项 3 / 7）。
    fn judge_allocation_generations(
        &self,
        expected: &ModelRoot,
        observed: &ObservedRoot,
        counts: &mut ModelJudgementCounts,
    ) -> Result<(), ModelDisagreement> {
        for (role, records) in &observed.unit_allocation_records {
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
/// 与层 0 的 `crash::oracle_violation_for_versions` 分开写：那一份的版本表由调用方按固定脚本手写，这一份问的是模型
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

    /// 抬 F 到 6 之后 F_生效 = 6：回退到 txg 6 的根是候选（txg ≥ F_生效，D16（发布语义） 已定项 1「回退候选集」），txg 5 的必须拒；
    /// 回退到 (6, 2) 之后实例 2 的 7–11 被抛弃（写了回退行 (2, 6, 0)），再回退到 (9, 2) 必须拒；冷启动读回回退那一版（txg 6 写的内容）。
    #[test]
    fn rollback_to_the_effective_floor_is_accepted_and_abandons_the_newer_roots_of_that_instance() {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        let raise = model
            .answer_raise_rollback_floor(ModelCheckpointTxg(6))
            .expect("会话开着");
        succeed(&mut model, &raise);
        assert_eq!(model.effective_rollback_floor(), ModelCheckpointTxg(6));
        model.close_session();
        assert_eq!(
            model.answer_mount_rollback(key(5, 2)).required_refusals,
            BTreeSet::from([ModelRefusalReason::RollbackTargetBelowEffectiveFloor])
        );
        let rollback = model.answer_mount_rollback(key(6, 2));
        assert_eq!(
            rollback.expected_mount,
            Some((
                ModelInstanceGeneration(3),
                vec![ModelInstanceRow {
                    instance: ModelInstanceGeneration(2),
                    selected_root_txg: ModelCheckpointTxg(6),
                    applied_transaction_high_water: 0,
                    is_rollback: true,
                }]
            ))
        );
        succeed(&mut model, &rollback);
        model.close_session();
        assert_eq!(
            model.answer_mount_rollback(key(9, 2)).required_refusals,
            BTreeSet::from([ModelRefusalReason::RollbackTargetOnAbandonedTimeline])
        );
        let cold_start = model.answer_cold_start_recover();
        let Some(ModelReadBack::FileRead { root, content }) = cold_start.expected_read_back else {
            panic!("回退那一版带文件：{cold_start:?}");
        };
        assert_eq!(
            root,
            key(13, 3),
            "回退写行 txg 12（盘 0）、暖机 txg 13（盘 1）"
        );
        assert_eq!(content.as_ref(), &[6]);
    }

    /// F_生效 = 各盘所带 F 最大值的最小值：只有一块盘上有带新 F 的根时不生效（D16（发布语义） 已定项 1「生效」）。抬 F 半路停在第一次
    /// （txg 10 落盘 1），盘 0 上最大的 F 仍是 0。
    #[test]
    fn raised_floor_carried_by_one_device_only_does_not_take_effect() {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        let raise = model
            .answer_raise_rollback_floor(ModelCheckpointTxg(6))
            .expect("会话开着");
        let first_publish_only = raise.expected_roots[..1].to_vec();
        for root in first_publish_only {
            model.write_root(root);
        }
        assert_eq!(model.effective_rollback_floor(), ModelCheckpointTxg(0));
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
            has_file: true,
            unit_allocation_records: vec![
                (ModelUnitRole::Data, records(data_generation)),
                (ModelUnitRole::InstanceTable, records(4)),
            ],
        };
        let mut counts = ModelJudgementCounts::default();
        assert_eq!(
            model.judge_allocation_generations(row_publish, &observed(3), &mut counts),
            Ok(())
        );
        assert_eq!(counts.allocation_records_compared, 4);
        let wrong = model
            .judge_allocation_generations(row_publish, &observed(4), &mut counts)
            .expect_err("照抄的数据单元分配代写成了这次的 txg");
        assert_eq!(wrong.aspect, ModelDisagreementAspect::AllocationGeneration);
    }

    fn refused_by_the_allocation_record_wall(
        publishes_completed: usize,
        allocation_records_counted_on_the_image: Option<u64>,
    ) -> ObservedOutcome {
        ObservedOutcome::Refused {
            member: "PublishError::AllocationRecordsExceedOneNode".to_string(),
            reason: ObservedRefusalReason::Explained(ModelRefusalReason::AllocationRecordNodeWall),
            publishes_completed,
            wrote_anything: publishes_completed > 0,
            reported_ceiling: None,
            allocation_records_counted_on_the_image,
        }
    }

    /// 在第二个实例里再覆盖写 50 次：沿来路的上界 102 + 50 × 16 = 902，远过 812（区间的上界那一头早就开了）。
    fn model_with_the_upper_bound_far_above_one_allocation_node() -> IdealModel {
        let mut model = model_after_four_overwrites_in_a_second_instance();
        for content_byte in 0_u8..50 {
            let overwrite = model
                .answer_publish_overwrite(&[content_byte])
                .expect("会话开着、现行版本带文件");
            succeed(&mut model, &overwrite);
        }
        model
    }

    /// 分配记录墙（增补 3 第 2 件代码三方第一轮判决第三节第 1 条）：覆盖写每盘加 8 条；镜像上数的基数 796 ⇒ 这次之后正好 812 条、一个节点装得下，
    /// 墙拒是「模型说该成、实现拒了」，哪怕沿来路的上界早过了 812；基数 797 ⇒ 813 条，放行；数不出基数不放行。
    #[test]
    fn the_allocation_record_wall_is_permitted_only_when_the_counted_records_plus_this_publish_exceed_812(
    ) {
        let model = model_with_the_upper_bound_far_above_one_allocation_node();
        let answer = model
            .answer_publish_overwrite(&[1])
            .expect("会话开着、现行版本带文件");
        assert_eq!(IdealModel::allocation_record_node_capacity(), 812);
        assert!(
            answer.planned_publish_upper_bounds[0].allocation_records > 812,
            "上界那一头开着：{:?}",
            answer.planned_publish_upper_bounds
        );
        assert_eq!(
            answer.planned_publish_upper_bounds[0].allocation_records_added_by_the_admission,
            16
        );
        assert_eq!(
            answer.root_whose_allocation_records_the_wall_counts(0),
            Some(model.session.as_ref().expect("会话开着").current.key),
            "覆盖写数现行版本那一版"
        );
        let judged = |counted: Option<u64>| {
            model
                .clone()
                .judge_and_advance(&answer, &refused_by_the_allocation_record_wall(0, counted))
                .map(|counts| counts.allocation_record_wall_refusals_over_one_node)
                .map_err(|disagreement| disagreement.aspect)
        };
        assert_eq!(
            judged(Some(796)),
            Err(ModelDisagreementAspect::RefusedWhenModelRequiresSuccess)
        );
        assert_eq!(judged(Some(797)), Ok(1));
        assert_eq!(
            judged(None),
            Err(ModelDisagreementAspect::RefusedWhenModelRequiresSuccess)
        );
    }

    /// 可写挂载在第一次写之前一串判完：基数是所选根那一版，加上写行（五个角色每盘一条）与每次暖机（四个角色每盘一条）；一串里有一次
    /// 越过 812 才放行。抬 F 逐次判：做完一次之后被拒，基数数拒之前最后写出的那一版。
    #[test]
    fn mount_and_raise_count_the_wall_from_the_version_their_admission_starts_from() {
        let mut model = model_with_the_upper_bound_far_above_one_allocation_node();
        let raise = model
            .answer_raise_rollback_floor(ModelCheckpointTxg(0))
            .expect("会话开着、F 不往下抬");
        assert!(
            raise.expected_roots.len() >= 2,
            "{:?}",
            raise.expected_roots
        );
        assert_eq!(
            raise.root_whose_allocation_records_the_wall_counts(1),
            Some(raise.expected_roots[0].key),
            "做完一次之后被拒：数第一次抬 F 写出的那一版"
        );
        let second_publish_added =
            raise.planned_publish_upper_bounds[1].allocation_records_added_by_the_admission;
        assert_eq!(second_publish_added, 8);
        let judged_raise = |counted: u64| {
            model
                .clone()
                .judge_and_advance(
                    &raise,
                    &refused_by_the_allocation_record_wall(1, Some(counted)),
                )
                .map_err(|disagreement| disagreement.aspect)
                .is_ok()
        };
        assert!(!judged_raise(812 - second_publish_added));
        assert!(judged_raise(813 - second_publish_added));

        let chosen = model.newest_root().key;
        model.close_session();
        let mount = model.answer_mount_writable();
        assert_eq!(
            mount.root_whose_allocation_records_the_wall_counts(0),
            Some(chosen)
        );
        let added_by_the_mount: u64 = mount
            .planned_publish_upper_bounds
            .iter()
            .map(|planned| planned.allocation_records_added_by_the_admission)
            .sum();
        assert_eq!(
            added_by_the_mount,
            10 + 8 * u64::try_from(mount.planned_publish_upper_bounds.len() - 1).expect("次数"),
            "写行每盘 5 条、每次暖机每盘 4 条"
        );
        let judged_mount = |counted: u64| {
            model
                .clone()
                .judge_and_advance(
                    &mount,
                    &refused_by_the_allocation_record_wall(0, Some(counted)),
                )
                .is_ok()
        };
        assert!(!judged_mount(812 - added_by_the_mount));
        assert!(judged_mount(813 - added_by_the_mount));
    }
}
