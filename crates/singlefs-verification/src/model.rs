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
//! 模型只拿实现交回的每个单元那几条记录比「每块盘一条、仍分配、分配代等于写它的那次发布」（增补 3 第 2 件代码三方 那一格：
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

mod allocation_generation_judging;
mod answers_and_observed_outcomes;
mod answers_to_mounts_and_rollbacks;
mod answers_to_publishes_and_inode_changes;
mod answers_to_snapshots_and_compaction;
mod answers_to_truncations;
mod crash_recovery_judging;
mod disagreements_and_judgement_counts;
mod judging_of_observed_outcomes;
#[cfg(test)]
mod model_scenarios_for_tests;
mod next_root_of_a_publish;
mod pending_changes;
mod publish_changes;
mod ring_positions_and_tree_node_bounds;
mod rollback_floor_and_ceiling;
mod unit_roles_and_publish_kinds;
mod versions_and_roots;

pub use self::answers_and_observed_outcomes::*;
pub use self::crash_recovery_judging::*;
pub use self::disagreements_and_judgement_counts::*;
use self::judging_of_observed_outcomes::*;
#[cfg(test)]
use self::model_scenarios_for_tests::*;
pub use self::pending_changes::*;
use self::publish_changes::*;
pub use self::ring_positions_and_tree_node_bounds::*;
pub use self::rollback_floor_and_ceiling::*;
pub use self::unit_roles_and_publish_kinds::*;
pub use self::versions_and_roots::*;

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

/// 一条根落在哪块盘：区域的设备归属第一版写死 0 / 1 / 0（D2（RAID 条带策略） 已定项 7；D16（发布语义） 已定项 8）。
fn device_holding_the_root_of(checkpoint_txg: ModelCheckpointTxg) -> ModelDeviceIdentity {
    let region = usize::try_from(checkpoint_txg.0 % ROOT_RING_REGIONS).expect("区域号小于 3");
    ModelDeviceIdentity(ROOT_RING_REGION_DEVICES[region])
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
    /// 掉电留在系统配置里、根环里还没有根带着的那一份 F（[`Self::settle_floor_left_in_the_system_configurations_by_a_lost_power`]）；
    /// 没有这样的掉电时 0。只进 F_生效（[`Self::effective_rollback_floor`]）。
    floor_left_in_the_system_configurations_by_a_lost_power: ModelCheckpointTxg,
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

/// 第一版同时活着的只读快照至多几个：「10」（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第七节岔路 1）。
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
            floor_left_in_the_system_configurations_by_a_lost_power: ModelCheckpointTxg(0),
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
}
