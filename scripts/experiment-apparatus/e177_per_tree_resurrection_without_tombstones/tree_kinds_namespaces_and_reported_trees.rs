//! 逐树报告用的树种、名字空间与带名字空间的键、被报告的树与它们各自的判法。

use super::*;

// ---------------------------------------------------------------------------------------------
// 树、key 的名字空间、读法、臂、几何
// ---------------------------------------------------------------------------------------------

/// 树的种类（`crates/singlefs-core/src/records.rs:373-385` 的十一种，加码 5 中央映射树）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(super) enum TreeKind {
    Extent,
    Inode,
    AllocationRecords,
    Accounting,
    CentralMapping,
    Livelist,
    SparseSideTable,
    Deadlist,
    DeletedInodes,
    Tombstone,
    Lineage,
    LoggedOperations,
}

impl TreeKind {
    pub(super) const fn kind_code(self) -> u16 {
        match self {
            TreeKind::Extent => 1,
            TreeKind::Inode => 2,
            TreeKind::AllocationRecords => 3,
            TreeKind::Accounting => 4,
            TreeKind::CentralMapping => 5,
            TreeKind::Livelist => 6,
            TreeKind::SparseSideTable => 7,
            TreeKind::Deadlist => 8,
            TreeKind::DeletedInodes => 9,
            TreeKind::Tombstone => 10,
            TreeKind::Lineage => 11,
            TreeKind::LoggedOperations => 12,
        }
    }
}

/// 登记第一节「11 种树」：码 5 中央映射树不在里面（A1、K5）。
pub(super) const ELEVEN_TREE_KINDS: [TreeKind; 11] = [
    TreeKind::Extent,
    TreeKind::Inode,
    TreeKind::AllocationRecords,
    TreeKind::Accounting,
    TreeKind::Livelist,
    TreeKind::SparseSideTable,
    TreeKind::Deadlist,
    TreeKind::DeletedInodes,
    TreeKind::Tombstone,
    TreeKind::Lineage,
    TreeKind::LoggedOperations,
];

/// key 的名字空间：每棵有对象的树一个；分配记录树按两个口径各一个；R-S、R-G 比的分配记录与映射真值另各一个（执行员修订 2 C3）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(super) enum Namespace {
    Extent,
    Inode,
    AllocationRecordsPresent,
    AllocationRecordsAllocated,
    AllocationUnitsOfTheVersion,
    Accounting,
    CentralMapping,
    CentralMappingUnitsOfTheVersion,
    DeletedInodes,
    Tombstone,
}

pub(super) const NAMESPACE_COUNT: usize = 10;

pub(super) const NAMESPACES: [Namespace; NAMESPACE_COUNT] = [
    Namespace::Extent,
    Namespace::Inode,
    Namespace::AllocationRecordsPresent,
    Namespace::AllocationRecordsAllocated,
    Namespace::AllocationUnitsOfTheVersion,
    Namespace::Accounting,
    Namespace::CentralMapping,
    Namespace::CentralMappingUnitsOfTheVersion,
    Namespace::DeletedInodes,
    Namespace::Tombstone,
];

impl Namespace {
    pub(super) const fn position(self) -> usize {
        match self {
            Namespace::Extent => 0,
            Namespace::Inode => 1,
            Namespace::AllocationRecordsPresent => 2,
            Namespace::AllocationRecordsAllocated => 3,
            Namespace::AllocationUnitsOfTheVersion => 4,
            Namespace::Accounting => 5,
            Namespace::CentralMapping => 6,
            Namespace::CentralMappingUnitsOfTheVersion => 7,
            Namespace::DeletedInodes => 8,
            Namespace::Tombstone => 9,
        }
    }
}

/// 一条 key 的字段（不用的补 0）：extent (inode, 单元号)；inode (inode)；分配记录 (槽号)；记账 (统计量, 树, 盘, 代)；
/// 映射 (类标签, 出生树, 出生 txg, 实例代号, 尾段)；`deleted_inodes` (inode)；墓碑 (对象, 出生代, 区间起, 区间止, 死亡 txg)。
pub(super) type KeyFields = [u64; 5];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(super) struct NamespacedKey {
    pub(super) namespace: Namespace,
    pub(super) fields: KeyFields,
}

/// 产物里报的十三行树：十一种 + 码 5 中央映射树，分配记录树两个口径各一行。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(super) enum ReportedTree {
    Extent,
    Inode,
    AllocationRecordsPresent,
    AllocationRecordsAllocated,
    Accounting,
    CentralMapping,
    Livelist,
    SparseSideTable,
    Deadlist,
    DeletedInodes,
    Tombstone,
    Lineage,
    LoggedOperations,
}

pub(super) const REPORTED_TREES: [ReportedTree; 13] = [
    ReportedTree::Extent,
    ReportedTree::Inode,
    ReportedTree::AllocationRecordsPresent,
    ReportedTree::AllocationRecordsAllocated,
    ReportedTree::Accounting,
    ReportedTree::CentralMapping,
    ReportedTree::Livelist,
    ReportedTree::SparseSideTable,
    ReportedTree::Deadlist,
    ReportedTree::DeletedInodes,
    ReportedTree::Tombstone,
    ReportedTree::Lineage,
    ReportedTree::LoggedOperations,
];

impl ReportedTree {
    pub(super) const fn name(self) -> &'static str {
        match self {
            ReportedTree::Extent => "extent",
            ReportedTree::Inode => "inode",
            ReportedTree::AllocationRecordsPresent => "allocation_records_a",
            ReportedTree::AllocationRecordsAllocated => "allocation_records_b",
            ReportedTree::Accounting => "accounting",
            ReportedTree::CentralMapping => "central_mapping",
            ReportedTree::Livelist => "livelist",
            ReportedTree::SparseSideTable => "sparse_side_table",
            ReportedTree::Deadlist => "deadlist",
            ReportedTree::DeletedInodes => "deleted_inodes",
            ReportedTree::Tombstone => "tombstone",
            ReportedTree::Lineage => "lineage",
            ReportedTree::LoggedOperations => "logged_operations",
        }
    }

    pub(super) const fn kind(self) -> TreeKind {
        match self {
            ReportedTree::Extent => TreeKind::Extent,
            ReportedTree::Inode => TreeKind::Inode,
            ReportedTree::AllocationRecordsPresent | ReportedTree::AllocationRecordsAllocated => {
                TreeKind::AllocationRecords
            }
            ReportedTree::Accounting => TreeKind::Accounting,
            ReportedTree::CentralMapping => TreeKind::CentralMapping,
            ReportedTree::Livelist => TreeKind::Livelist,
            ReportedTree::SparseSideTable => TreeKind::SparseSideTable,
            ReportedTree::Deadlist => TreeKind::Deadlist,
            ReportedTree::DeletedInodes => TreeKind::DeletedInodes,
            ReportedTree::Tombstone => TreeKind::Tombstone,
            ReportedTree::Lineage => TreeKind::Lineage,
            ReportedTree::LoggedOperations => TreeKind::LoggedOperations,
        }
    }

    /// 这一行在这个读法下比哪一个名字空间：R-S、R-G 下分配记录（两个口径）与映射比「这一版的码 1 / 码 3 单元」那一份（C3）；
    /// 今天没有单元的五棵树没有名字空间。
    pub(super) const fn namespace(self, reader: Reader) -> Option<Namespace> {
        let selects = match reader {
            Reader::UnitScan => false,
            Reader::SelectCurrent | Reader::SelectCurrentGatedByObject => true,
        };
        match self {
            ReportedTree::Extent => Some(Namespace::Extent),
            ReportedTree::Inode => Some(Namespace::Inode),
            ReportedTree::AllocationRecordsPresent => {
                if selects {
                    Some(Namespace::AllocationUnitsOfTheVersion)
                } else {
                    Some(Namespace::AllocationRecordsPresent)
                }
            }
            ReportedTree::AllocationRecordsAllocated => {
                if selects {
                    Some(Namespace::AllocationUnitsOfTheVersion)
                } else {
                    Some(Namespace::AllocationRecordsAllocated)
                }
            }
            ReportedTree::Accounting => Some(Namespace::Accounting),
            ReportedTree::CentralMapping => {
                if selects {
                    Some(Namespace::CentralMappingUnitsOfTheVersion)
                } else {
                    Some(Namespace::CentralMapping)
                }
            }
            ReportedTree::DeletedInodes => Some(Namespace::DeletedInodes),
            ReportedTree::Tombstone => Some(Namespace::Tombstone),
            ReportedTree::Livelist
            | ReportedTree::SparseSideTable
            | ReportedTree::Deadlist
            | ReportedTree::Lineage
            | ReportedTree::LoggedOperations => None,
        }
    }
}

#[cfg(test)]
#[path = "tree_kinds_namespaces_and_reported_trees_tests.rs"]
mod tests;
