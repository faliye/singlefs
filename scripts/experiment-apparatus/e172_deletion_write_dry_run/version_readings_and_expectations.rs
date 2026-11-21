//! 一个版本里读出的待删记录、数据单元、墓碑与退役记录数、预期保留量、删除发布的分类与格行。

use super::*;

/// L03-7 的两支（跑前登记 e172-r2 第一节 1.2）：按「上一版里装着被删记录的那片 inode 叶有几条记录」选，
/// 从窗口里上一次写出的那片叶解出来（条件是条款的输入，不是答案）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DeletedRecordLeaf {
    /// 只有被删那一条：那一片删空整片不落盘（layout/03 第 13 行）。
    OnlyTheDeletedRecord,
    /// 还有别的记录：那一片照写（layout/03 第 55 行「叶里还有别的记录时那一片照写」）。
    BesideOtherRecords,
    /// 上一版里找不到装着被删记录的叶（窗口里没写过）。
    NotFound,
}

impl DeletedRecordLeaf {
    pub(super) fn name(self) -> &'static str {
        match self {
            DeletedRecordLeaf::OnlyTheDeletedRecord => "only_record",
            DeletedRecordLeaf::BesideOtherRecords => "other_records",
            DeletedRecordLeaf::NotFound => "not_found",
        }
    }
}

/// 这一版（`state`）inode 树根条目指着的叶里装着被删记录的那一片（L03-7 选支与 L03-16 的身份期望都从它取）。
pub(super) fn inode_leaf_holding_the_deleted_record(state: &VersionState) -> Option<&PackedContainer> {
    let live_containers = container_numbers_of_root(state.latest_node(INODE_TREE, true));
    state
        .latest_containers(INODE_TREE, 2)
        .into_iter()
        .filter(|container| live_containers.contains(&container.container_number))
        .find(|container| container.records.iter().any(|record| read_u64(record, 0) == DELETED_INODE))
}

/// 上一版（`state`）里装着被删记录的那片 inode 叶：哪一支，与叶里别的记录的 inode 号（升序）。
pub(super) fn deleted_record_leaf_of_version(state: &VersionState) -> (DeletedRecordLeaf, Vec<u64>) {
    let Some(leaf) = inode_leaf_holding_the_deleted_record(state) else { return (DeletedRecordLeaf::NotFound, Vec::new()) };
    let mut other_inodes: Vec<u64> = leaf.records.iter().map(|record| read_u64(record, 0)).filter(|inode| *inode != DELETED_INODE).collect();
    other_inodes.sort_unstable();
    let branch = if other_inodes.is_empty() { DeletedRecordLeaf::OnlyTheDeletedRecord } else { DeletedRecordLeaf::BesideOtherRecords };
    (branch, other_inodes)
}

/// layout/03 第 51–55 行：每种删除类发布每盘单元数与角色名（按剩下的数据单元数分）；最后一批在 L03-7 第二支时
/// 照写 inode 叶容器 t3（layout/03 第 55 行「叶里还有别的记录时那一片照写」，跑前登记 e172-r2 第一节 1.2）。
pub(super) fn registered_roles(publish_kind: &str, data_units_after: u64, deleted_record_leaf: DeletedRecordLeaf) -> Option<Vec<&'static str>> {
    let allocation = ["t5@0.0.61", "t5@0.1.61", "t5@1.0.0", "t5@1.1.0", "t5"];
    let mut roles: Vec<&'static str> = Vec::new();
    match (publish_kind == "unlink_intent", data_units_after) {
        (true, _) => {
            roles.extend(["t3", "t4"]);
            roles.extend(allocation);
            roles.extend(["t6", "td", "tt@0", "tt", "t7", "t8"]);
        }
        (false, 2) => {
            roles.extend(["t2下@0.0", "t2", "t3", "t4"]);
            roles.extend(allocation);
            roles.extend(["t6", "t7", "t8"]);
        }
        (false, 1) => {
            roles.extend(["t2", "t3", "t4"]);
            roles.extend(allocation);
            roles.extend(["t6", "t7", "t8"]);
        }
        (false, 0) => {
            roles.push("t2");
            if deleted_record_leaf == DeletedRecordLeaf::BesideOtherRecords {
                roles.push("t3");
            }
            roles.push("t4");
            roles.extend(allocation);
            roles.push("t6");
            // layout/03 第 60 行（最后一批那一行）：那片叶删空、身份消失 ⇒ 退役树开第一片容器、写根，排在 t6 之后、t7 之前；
            // 叶里还有别的记录时那一片照写（t3）、不写退役单元（跑前登记 e172-r3 第五节 5.8 改18）。
            match deleted_record_leaf {
                DeletedRecordLeaf::OnlyTheDeletedRecord | DeletedRecordLeaf::NotFound => roles.extend(["tr@0", "tr"]),
                DeletedRecordLeaf::BesideOtherRecords => {}
            }
            roles.extend(["t7", "t8"]);
        }
        (false, _) => return None,
    }
    Some(roles)
}

// ---------------------------------------------------------------------------------------------
// 一版的派生量：剩下的数据单元数、队列里的号、记账行、n_live、墓碑树内部层数。
// ---------------------------------------------------------------------------------------------

/// 映射条目的类标签是码 1 的条数（layout/01 第 228 行：码 1 一条一个数据单元）。
pub(super) fn data_units_of_version(state: &VersionState) -> Option<u64> {
    state.latest_node(MAPPING_TREE, true).map(|node| u64_of(node.entries.iter().filter(|entry| entry[0] == 1).count()))
}

fn tree_table_of_version(state: &VersionState) -> Vec<TreeTableEntry> {
    state.latest_node(0, true).map(tree_table_entries_of).unwrap_or_default()
}

pub(super) fn tree_table_root_is_zero(state: &VersionState, tree: u64) -> Option<bool> {
    tree_table_of_version(state).iter().find(|entry| entry.tree == tree).map(|entry| entry.root_pointer.is_all_zero)
}

/// 队列里的号：树表里 `deleted_inodes` 根指针置零时为空，否则取现行那一版 td 的条目（layout/03 第 10–11 行）。
pub(super) fn deleted_inodes_of_version(state: &VersionState) -> Vec<u64> {
    match tree_table_root_is_zero(state, DELETED_INODES_TREE) {
        Some(false) => state
            .latest_node(DELETED_INODES_TREE, true)
            .map(|node| node.entries.iter().map(|entry| read_u64(entry, 0)).collect())
            .unwrap_or_default(),
        Some(true) | None => Vec::new(),
    }
}

pub(super) fn accounting_value(state: &VersionState, statistic_tag: u64) -> Option<u64> {
    state.latest_node(ACCOUNTING_TREE, true).and_then(|node| {
        node.entries.iter().find(|entry| read_u16(entry, 0) == statistic_tag).map(|entry| read_u64(entry, 22))
    })
}

/// 现行那一版 inode 树里活着的记录：inode 树根条目指着的容器里的记录（根条目数 0 ⇒ 没有记录）。
pub(super) fn inode_records_of_version(state: &VersionState) -> Vec<Vec<u8>> {
    let live_containers = container_numbers_of_root(state.latest_node(INODE_TREE, true));
    state
        .latest_containers(INODE_TREE, 2)
        .into_iter()
        .filter(|container| live_containers.contains(&container.container_number))
        .flat_map(|container| container.records.iter().cloned())
        .collect()
}

pub(super) fn record_of_inode(records: &[Vec<u8>], inode: u64) -> Option<Vec<u8>> {
    records.iter().find(|record| read_u64(record, 0) == inode).cloned()
}

/// layout/03 第 20 行：n_live = inode 树里 nlink ≥ 1 的记录数；H = 墓碑树内部层数（根直接管容器时层级 1 ⇒ H = 0）。
pub(super) fn live_inode_count(state: &VersionState) -> u64 {
    u64_of(inode_records_of_version(state).iter().filter(|record| read_u32(record, 36) >= 1).count())
}

pub(super) fn tombstone_internal_levels(state: &VersionState) -> u64 {
    match tree_table_root_is_zero(state, TOMBSTONE_TREE) {
        Some(false) => state.latest_node(TOMBSTONE_TREE, true).map(|node| u64::from(node.level).saturating_sub(1)).unwrap_or(0),
        Some(true) | None => 0,
    }
}

/// layout/03 第 24 行：n_记录 = 这一版 inode 树里全部记录数（含 nlink = 0 的）。
pub(super) fn inode_record_count(state: &VersionState) -> u64 {
    u64_of(inode_records_of_version(state).len())
}

/// layout/03 第 24 行：H_退役 = 现行那一版退役树的内部层数（零根与只有根时 0；根直接管容器时层级 1 ⇒ 0）。
pub(super) fn retirement_internal_levels(state: &VersionState) -> u64 {
    match tree_table_root_is_zero(state, RETIREMENT_TREE) {
        Some(false) => state.latest_node(RETIREMENT_TREE, true).map(|retirement_root| u64::from(retirement_root.level).saturating_sub(1)).unwrap_or(0),
        Some(true) | None => 0,
    }
}

/// 现行那一版退役树里的退役记录数：退役树根条目指着的退役容器里的记录（树表里根指针置零 ⇒ 0；R-traj 那一列）。
pub(super) fn retirement_record_count_of_version(state: &VersionState) -> u64 {
    if tree_table_root_is_zero(state, RETIREMENT_TREE) != Some(false) {
        return 0;
    }
    let live_containers = container_numbers_of_root(state.latest_node(RETIREMENT_TREE, true));
    state
        .latest_containers(RETIREMENT_TREE, RETIREMENT_PACKED_RECORD_TYPE)
        .into_iter()
        .filter(|container| live_containers.contains(&container.container_number))
        .map(|container| u64_of(container.records.len()))
        .sum()
}

/// 标签 6 式子里退役那一份算不算（跑前登记 e172-r3 第一节 1.2 判别臂：去掉退役项那一份把三槽与 35 × n_记录 置零）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RetirementShare {
    Included,
    Removed,
}

/// R-ret 判别臂「去掉退役项」那一份用的是哪一种（第九节 M31 的锚点：改成 `Included` 判别臂就退化成被测臂）。
const DISCRIMINATING_ARM_RETIREMENT_SHARE: RetirementShare = RetirementShare::Removed;

/// layout/03 第 24 行：副本数 × ((2 + H_墓碑 + 1 + 2 + H_退役 + 1) × 16384 + 41 × n_live + 35 × n_记录)。
pub(super) fn expected_committed_reservation(tombstone_levels: u64, retirement_levels: u64, live_inodes: u64, inode_records: u64, retirement_share: RetirementShare) -> u64 {
    let retirement_slots = match retirement_share {
        RetirementShare::Included => 2 + retirement_levels + 1,
        RetirementShare::Removed => 0,
    };
    let retirement_record_bytes = match retirement_share {
        RetirementShare::Included => PER_INODE_RECORD_RETIREMENT_BYTES * inode_records,
        RetirementShare::Removed => 0,
    };
    ACCOUNTING_COPIES * ((2 + tombstone_levels + 1 + retirement_slots) * SLOT_BYTES + PER_LIVE_OBJECT_TOMBSTONE_BYTES * live_inodes + retirement_record_bytes)
}

/// R-ret 的两份期望（带退役项的、判别臂那一份），由同一个式子函数在同一份解码读数上算。
pub(super) fn retirement_arm_expectations(tombstone_levels: u64, retirement_levels: u64, live_inodes: u64, inode_records: u64) -> (u64, u64) {
    (
        expected_committed_reservation(tombstone_levels, retirement_levels, live_inodes, inode_records, RetirementShare::Included),
        expected_committed_reservation(tombstone_levels, retirement_levels, live_inodes, inode_records, DISCRIMINATING_ARM_RETIREMENT_SHARE),
    )
}

/// 删除类发布按「这一版之后剩下几个数据单元」分的四种（3 单元文件、批大小 1；跑前登记 e172-r3 第一节 1.2 那张表的四行）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DeletionPublish {
    Intent,
    BatchLeavingTwoUnits,
    BatchLeavingOneUnit,
    FinalBatch,
}

impl DeletionPublish {
    pub(super) fn name(self) -> &'static str {
        match self {
            DeletionPublish::Intent => "intent",
            DeletionPublish::BatchLeavingTwoUnits => "three_to_two",
            DeletionPublish::BatchLeavingOneUnit => "two_to_one",
            DeletionPublish::FinalBatch => "final",
        }
    }
}

/// 一次删除类发布是四种里的哪一种：意图看导出的发布种类，批看解码出的这一版剩下的数据单元数。
pub(super) fn deletion_publish_of(segment: &SegmentAnalysis, after: &VersionState) -> Option<DeletionPublish> {
    if !segment.is_deletion_publish {
        return None;
    }
    if segment.label_kind() == "unlink_intent" {
        return Some(DeletionPublish::Intent);
    }
    match data_units_of_version(after) {
        Some(2) => Some(DeletionPublish::BatchLeavingTwoUnits),
        Some(1) => Some(DeletionPublish::BatchLeavingOneUnit),
        Some(0) => Some(DeletionPublish::FinalBatch),
        Some(_) | None => None,
    }
}

// ---------------------------------------------------------------------------------------------
// 判定行。
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub(super) struct CellRow {
    pub(super) cell: &'static str,
    pub(super) field: String,
    pub(super) history: String,
    pub(super) txg: u64,
    pub(super) publish_kind: String,
    pub(super) expected: String,
    pub(super) actual: String,
    pub(super) discriminable: bool,
}

impl CellRow {
    pub(super) fn is_equal(&self) -> bool {
        self.expected == self.actual
    }
    pub(super) fn line(&self) -> String {
        format!(
            "name=l03_cell cell={} field={} history={} txg={} publish={} expected={} actual={} equal={} discriminable={}",
            self.cell,
            self.field,
            self.history,
            self.txg,
            self.publish_kind,
            self.expected,
            self.actual,
            self.is_equal(),
            self.discriminable
        )
    }
}
