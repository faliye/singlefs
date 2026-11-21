//! 分配记录树的节点、选择组与单元归属、单元内容与出现、盘与版本。

use super::*;

/// 分配记录树的一个节点：根之下按 (层级, 盘, 同盘同层序号)，根排最末（`allocation_record_tree.rs:44-56`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum AllocationNode {
    BelowTheRoot { level: u8, device: u8, index_in_device: u64 },
    Root,
}

fn allocation_span_in_slots_at_level(level: u8) -> u64 {
    (0..level).fold(ALLOCATION_RECORD_TREE_LEAF_SLOTS, |span, _| {
        span.saturating_mul(ALLOCATION_RECORD_TREE_INTERNAL_FANOUT)
    })
}

/// 根的层级：最小的 R ≥ 1 使 Σ_盘 ⌈盘上绝对槽数 ÷ S_{R−1}⌉ ≤ F（`allocation_record_tree.rs:94-116`）。
fn allocation_root_level(device_slots: u64, devices: u64) -> u8 {
    (1..=u8::MAX)
        .find(|candidate| {
            device_slots.div_ceil(allocation_span_in_slots_at_level(candidate - 1)) * devices
                <= ALLOCATION_RECORD_TREE_INTERNAL_FANOUT
        })
        .expect("两块盘时层级至多几层就装得下")
}

/// 三个几何都按 4 GiB 两盘的几何分叶与祖先（执行员修订 2 C8）。
pub(super) fn allocation_tree_root_level() -> u8 {
    allocation_root_level(DEVICE_SLOTS, u64::from(DEVICE_COUNT))
}

fn allocation_parent_of(node: AllocationNode, root_level: u8) -> Option<AllocationNode> {
    match node {
        AllocationNode::Root => None,
        AllocationNode::BelowTheRoot {
            level,
            device,
            index_in_device,
        } => {
            if level + 1 == root_level {
                Some(AllocationNode::Root)
            } else {
                Some(AllocationNode::BelowTheRoot {
                    level: level + 1,
                    device,
                    index_in_device: index_in_device / ALLOCATION_RECORD_TREE_INTERNAL_FANOUT,
                })
            }
        }
    }
}

pub(super) fn allocation_nodes_of_leaves(leaves: impl Iterator<Item = u64>) -> BTreeSet<AllocationNode> {
    let root_level = allocation_tree_root_level();
    let mut nodes = BTreeSet::new();
    nodes.insert(AllocationNode::Root);
    for leaf in leaves {
        for device in 0..DEVICE_COUNT {
            let mut current = Some(AllocationNode::BelowTheRoot {
                level: 0,
                device,
                index_in_device: leaf,
            });
            while let Some(node) = current {
                if !nodes.insert(node) && node != AllocationNode::Root {
                    break;
                }
                current = allocation_parent_of(node, root_level);
            }
        }
    }
    nodes
}

/// 从 `before` 到 `after`，记录逐槽变了的叶连祖先与根（`nodes_whose_contents_changed` 的口径；比的是 `after` 上动过的槽）。
pub(super) fn allocation_nodes_whose_contents_changed(before: &Allocator, after: &Allocator) -> BTreeSet<AllocationNode> {
    let changed_leaves: BTreeSet<u64> = after
        .dirty_slots
        .iter()
        .filter(|slot| before.records.get(*slot) != after.records.get(*slot))
        .map(|slot| Allocator::leaf_of_slot(*slot))
        .collect();
    if changed_leaves.is_empty() {
        return BTreeSet::new();
    }
    allocation_nodes_of_leaves(changed_leaves.into_iter())
}

// ---------------------------------------------------------------------------------------------
// 单元与盘：槽阵列只模一份（两盘同槽同内容，登记 5.1）；一个单元的任一槽被之后的写盖过就读不出（W8）
// ---------------------------------------------------------------------------------------------

/// 择现行的分组（登记 5.4）：数据单元按 (出生树, 对象, 对象出生代, 锚点)，码 3 容器按 (出生树, 类型, 容器号, 容器出生代)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(super) enum SelectionGroup {
    DataUnit {
        birth_tree: u64,
        object: u64,
        object_birth: u64,
        anchor_unit: u64,
    },
    PackedContainer {
        birth_tree: u64,
        record_type: u64,
        container_number: u64,
        container_birth: u64,
    },
}

/// 择新键 (诞生代号, 实例代号, 事务号)。
pub(super) type WriteOrder = (u64, u64, u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum UnitOwner {
    FileData,
    ExtentLowerNode,
    ExtentRoot,
    InodeContainer,
    InodeRoot,
    AllocationNode,
    AccountingNode,
    MappingNode,
    DeletedInodesRoot,
    TombstoneContainer,
    TombstoneRoot,
    TreeTable,
    InstanceTable,
}

impl UnitOwner {
    /// 这个单元算哪棵树的单元（判「没有对象」用）；数据单元、树表、实例表不算哪棵树的节点。
    pub(super) const fn tree_kind(self) -> Option<TreeKind> {
        match self {
            UnitOwner::ExtentLowerNode | UnitOwner::ExtentRoot => Some(TreeKind::Extent),
            UnitOwner::InodeContainer | UnitOwner::InodeRoot => Some(TreeKind::Inode),
            UnitOwner::AllocationNode => Some(TreeKind::AllocationRecords),
            UnitOwner::AccountingNode => Some(TreeKind::Accounting),
            UnitOwner::MappingNode => Some(TreeKind::CentralMapping),
            UnitOwner::DeletedInodesRoot => Some(TreeKind::DeletedInodes),
            UnitOwner::TombstoneContainer | UnitOwner::TombstoneRoot => Some(TreeKind::Tombstone),
            UnitOwner::FileData | UnitOwner::TreeTable | UnitOwner::InstanceTable => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct InodeRecordSummary {
    pub(super) inode: u64,
    pub(super) object_birth: u64,
    pub(super) nlink: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct TombstoneRecordSummary {
    pub(super) object: u64,
    pub(super) object_birth: u64,
    pub(super) death_txg: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ExtentRecordSummary {
    pub(super) inode: u64,
    pub(super) data_unit: u64,
    /// 它指着的那个数据单元的出生 txg（C9）。
    pub(super) pointer_birth: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum UnitContents {
    DataUnit {
        object: u64,
        object_birth: u64,
        anchor_unit: u64,
    },
    InodeRecords(Vec<InodeRecordSummary>),
    TombstoneRecords(Vec<TombstoneRecordSummary>),
    ExtentRecords(Vec<ExtentRecordSummary>),
    /// (槽号, 是否已释放)。
    AllocationRecords(Vec<(u64, bool)>),
    AccountingKeys(Vec<KeyFields>),
    MappingKeys(Vec<KeyFields>),
    DeletedInodes(Vec<u64>),
    WithoutKeys,
}

/// 一次出现带的身份（臂 B 的墓碑效力按它判，登记 5.3）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum OccurrenceTag {
    InodeRecord { inode: u64, object_birth: u64 },
    ExtentRecord { inode: u64, pointer_birth: u64 },
    DeletedInode { inode: u64 },
    Untagged,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Occurrence {
    pub(super) key: NamespacedKey,
    pub(super) tag: OccurrenceTag,
}

pub(super) fn single_field_key(value: u64) -> KeyFields {
    [value, 0, 0, 0, 0]
}

pub(super) fn extent_key(inode: u64, data_unit: u64) -> KeyFields {
    [inode, data_unit, 0, 0, 0]
}

pub(super) fn tombstone_key(record: TombstoneRecordSummary) -> KeyFields {
    [record.object, record.object_birth, 0, WHOLE_OBJECT_RANGE_END, record.death_txg]
}

pub(super) fn namespaced(namespace: Namespace, fields: KeyFields) -> NamespacedKey {
    NamespacedKey { namespace, fields }
}

/// R-U 读这个单元收到的每一次 key 出现（登记 5.4 R-U 那一列）。
pub(super) fn occurrences_of(contents: &UnitContents) -> Vec<Occurrence> {
    let untagged = |namespace: Namespace, fields: KeyFields| Occurrence {
        key: namespaced(namespace, fields),
        tag: OccurrenceTag::Untagged,
    };
    match contents {
        UnitContents::InodeRecords(records) => records
            .iter()
            .map(|record| Occurrence {
                key: namespaced(Namespace::Inode, single_field_key(record.inode)),
                tag: OccurrenceTag::InodeRecord {
                    inode: record.inode,
                    object_birth: record.object_birth,
                },
            })
            .collect(),
        UnitContents::TombstoneRecords(records) => records
            .iter()
            .map(|record| untagged(Namespace::Tombstone, tombstone_key(*record)))
            .collect(),
        UnitContents::ExtentRecords(records) => records
            .iter()
            .map(|record| Occurrence {
                key: namespaced(Namespace::Extent, extent_key(record.inode, record.data_unit)),
                tag: OccurrenceTag::ExtentRecord {
                    inode: record.inode,
                    pointer_birth: record.pointer_birth,
                },
            })
            .collect(),
        UnitContents::AllocationRecords(records) => {
            let mut occurrences = Vec::new();
            for (slot, is_released) in records {
                occurrences.push(untagged(Namespace::AllocationRecordsPresent, single_field_key(*slot)));
                if !is_released {
                    occurrences.push(untagged(Namespace::AllocationRecordsAllocated, single_field_key(*slot)));
                }
            }
            occurrences
        }
        UnitContents::AccountingKeys(keys) => keys
            .iter()
            .map(|fields| untagged(Namespace::Accounting, *fields))
            .collect(),
        UnitContents::MappingKeys(keys) => keys
            .iter()
            .map(|fields| untagged(Namespace::CentralMapping, *fields))
            .collect(),
        UnitContents::DeletedInodes(inodes) => inodes
            .iter()
            .map(|inode| Occurrence {
                key: namespaced(Namespace::DeletedInodes, single_field_key(*inode)),
                tag: OccurrenceTag::DeletedInode { inode: *inode },
            })
            .collect(),
        UnitContents::DataUnit { .. } | UnitContents::WithoutKeys => Vec::new(),
    }
}

#[derive(Clone, Debug)]
pub(super) struct Unit {
    pub(super) owner: UnitOwner,
    pub(super) class: u64,
    pub(super) group: Option<SelectionGroup>,
    pub(super) write_order: WriteOrder,
    pub(super) slot: u64,
    pub(super) span_slots: u64,
    pub(super) release_txg: Option<u64>,
    pub(super) is_readable: bool,
    pub(super) mapping_key: Option<KeyFields>,
    pub(super) contents: UnitContents,
}

impl Unit {
    pub(super) fn birth_txg(&self) -> u64 {
        self.write_order.0
    }
}

#[derive(Clone, Debug, Default)]
pub(super) struct Disk {
    pub(super) units: Vec<Unit>,
    pub(super) occupant_of_slot: HashMap<u64, UnitIdentifier>,
    pub(super) units_written: u64,
    pub(super) units_made_unreadable: u64,
}

impl Disk {
    /// 写一个单元：它占的每一槽上原来那个单元读不出了（整个或部分盖住一样，W8）。交回这次变得读不出的单元。
    pub(super) fn write(&mut self, unit: Unit) -> Vec<UnitIdentifier> {
        let unit_identifier = self.units.len();
        let mut made_unreadable = Vec::new();
        for slot in unit.slot..unit.slot + unit.span_slots {
            if let Some(previous) = self.occupant_of_slot.insert(slot, unit_identifier) {
                if self.units[previous].is_readable {
                    self.units[previous].is_readable = false;
                    self.units_made_unreadable += 1;
                    made_unreadable.push(previous);
                }
            }
        }
        self.units.push(unit);
        self.units_written += 1;
        made_unreadable
    }

    pub(super) fn readable_units_by_scan(&self) -> u64 {
        length_as_u64(self.units.iter().filter(|unit| unit.is_readable).count())
    }
}

// ---------------------------------------------------------------------------------------------
// 一版的样子与一次发布的计划（S3–S13、M5–M11）
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub(super) struct InodeContainerVersion {
    pub(super) container_number: u64,
    pub(super) container_birth: u64,
    pub(super) records: Vec<InodeRecordSummary>,
    pub(super) unit: UnitIdentifier,
}

#[derive(Clone, Debug)]
pub(super) struct TombstoneContainerVersion {
    pub(super) container_number: u64,
    pub(super) container_birth: u64,
    pub(super) records: Vec<TombstoneRecordSummary>,
    pub(super) unit: UnitIdentifier,
}

#[derive(Clone, Debug)]
pub(super) struct Version {
    pub(super) txg: u64,
    pub(super) instance: u64,
    pub(super) highest_transaction: u64,
    pub(super) next_inode: u64,
    /// inode 1 的数据单元，按单元序号。
    pub(super) file_units: Vec<UnitIdentifier>,
    pub(super) extent_root: Option<UnitIdentifier>,
    pub(super) extent_lower: BTreeMap<(u8, u64), UnitIdentifier>,
    pub(super) inode_containers: Vec<InodeContainerVersion>,
    pub(super) inode_root: Option<UnitIdentifier>,
    pub(super) deleted_queue: Vec<u64>,
    pub(super) deleted_root: Option<UnitIdentifier>,
    pub(super) tombstone_containers: Vec<TombstoneContainerVersion>,
    pub(super) tombstone_root: Option<UnitIdentifier>,
    pub(super) allocation_nodes: BTreeMap<AllocationNode, UnitIdentifier>,
    pub(super) accounting: CodeTwoTreeVersion,
    pub(super) mapping: CodeTwoTreeVersion,
    pub(super) tree_table: UnitIdentifier,
    pub(super) instance_table: UnitIdentifier,
}

impl Version {
    pub(super) fn live_record_of(&self, inode: u64) -> Option<InodeRecordSummary> {
        self.inode_containers
            .iter()
            .flat_map(|container| container.records.iter())
            .find(|record| record.inode == inode)
            .copied()
    }

    pub(super) fn has_live_inode(&self, inode: u64) -> bool {
        self.live_record_of(inode).is_some_and(|record| record.nlink > 0)
    }

    /// 「有树、文件集合为空」（`transaction.rs:2968-2977`）：没有 inode 叶容器、没有数据单元、`deleted_inodes` 无根。
    pub(super) fn is_with_trees_and_no_file(&self) -> bool {
        self.inode_containers.is_empty() && self.file_units.is_empty() && self.deleted_root.is_none()
    }
}

/// 一次发布做什么（M5、S12、S13）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PublishOperation {
    FirstFile,
    RowPublish,
    WarmUp,
    Overwrite,
    SequentialWrite { data_units: u64 },
    NewInodes { count: u64 },
    UnlinkIntent { inode: u64 },
    UnlinkBatch { data_units_kept: u64 },
    UnlinkFinal { inode: u64 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PlannedUnit {
    Rewritten,
    Carried(UnitIdentifier),
}

#[derive(Clone, Debug)]
pub(super) struct PlannedInodeContainer {
    pub(super) container_number: u64,
    pub(super) container_birth: u64,
    pub(super) records: Vec<InodeRecordSummary>,
    pub(super) origin: PlannedUnit,
}

#[derive(Clone, Debug)]
pub(super) struct PlannedTombstoneContainer {
    pub(super) container_number: u64,
    pub(super) container_birth: u64,
    pub(super) records: Vec<TombstoneRecordSummary>,
    pub(super) origin: PlannedUnit,
}

/// 一次发布里不依赖落点的那一半：每棵树这次之后的内容、哪些单元重写、哪些照抄，与换下的上一版单元（分配记录树、记账树、
/// 映射树、树表那几样由 [`assemble_publish`] 补）。
#[derive(Clone, Debug)]
pub(super) struct LogicalPlan {
    pub(super) txg: u64,
    pub(super) instance: u64,
    pub(super) transaction_base: u64,
    pub(super) highest_transaction_after: u64,
    pub(super) next_inode_after: u64,
    pub(super) file_object_birth: u64,
    pub(super) data_units: Vec<PlannedUnit>,
    pub(super) instance_table_rewritten: bool,
    pub(super) extent_lower: Vec<((u8, u64), PlannedUnit)>,
    pub(super) extent_root: PlannedUnit,
    pub(super) inode_containers: Vec<PlannedInodeContainer>,
    pub(super) inode_root: PlannedUnit,
    pub(super) deleted_queue: Vec<u64>,
    pub(super) deleted_root: Option<PlannedUnit>,
    pub(super) tombstone_containers: Vec<PlannedTombstoneContainer>,
    pub(super) tombstone_root: Option<PlannedUnit>,
    pub(super) tombstones_written: Vec<TombstoneRecordSummary>,
    pub(super) accounting_keys: BTreeSet<KeyFields>,
    pub(super) released_units: Vec<UnitIdentifier>,
}
