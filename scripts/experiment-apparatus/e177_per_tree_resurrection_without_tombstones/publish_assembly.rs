//! 把逻辑规划拼成一次发布：角色、映射键、单元的落点，与发布的结算。

use super::*;

/// 这次发布重写的一个角色（M7 的次序就是 `assemble_publish` 排出来的次序）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum Role {
    Data(usize),
    InstanceTable,
    ExtentLower((u8, u64)),
    ExtentRoot,
    InodeContainer(usize),
    InodeRoot,
    AllocationNode(AllocationNode),
    AccountingNode(CodeTwoNodePosition),
    DeletedInodesRoot,
    TombstoneContainer(usize),
    TombstoneRoot,
    MappingNode(CodeTwoNodePosition),
    TreeTable,
}

impl Role {
    /// 用户数据按政策函数；码 3 容器两槽对齐；码 2 节点一槽（`transaction.rs` `TransactionUnit::placement`）。
    pub(super) const fn placement_rule(self) -> PlacementRule {
        match self {
            Role::Data(_) => PlacementRule::UserData,
            Role::InstanceTable | Role::InodeContainer(_) | Role::TombstoneContainer(_) => {
                PlacementRule::CommitGenerated(UnitFootprint::TwoSlotsAligned)
            }
            Role::ExtentLower(_)
            | Role::ExtentRoot
            | Role::InodeRoot
            | Role::AllocationNode(_)
            | Role::AccountingNode(_)
            | Role::DeletedInodesRoot
            | Role::TombstoneRoot
            | Role::MappingNode(_)
            | Role::TreeTable => PlacementRule::CommitGenerated(UnitFootprint::OneSlot),
        }
    }
}

/// 一次发布排好的样子：角色按 bump 次序、三棵靠落点的树（分配记录、记账、映射）这次之后的样子、要释放的上一版单元。
#[derive(Clone, Debug)]
pub(super) struct AssembledPublish {
    pub(super) roles: Vec<Role>,
    pub(super) allocation_nodes: Vec<(AllocationNode, PlannedUnit)>,
    pub(super) accounting_plan: CodeTwoPlan,
    pub(super) mapping_plan: CodeTwoPlan,
    pub(super) mapping_keys_of_rewritten: HashMap<Role, KeyFields>,
    pub(super) releases: Vec<UnitIdentifier>,
}

fn node_mapping_key(plan: &LogicalPlan, unit_class: u64, tree: u64, birth_sequence: u64) -> KeyFields {
    [unit_class, tree, plan.txg, plan.instance, birth_sequence]
}

fn mapping_key_of_carried(disk: &Disk, unit: UnitIdentifier) -> KeyFields {
    disk.units[unit]
        .mapping_key
        .expect("照抄进来的单元在上一版里进映射、有一把 key")
}

/// 一个单元进映射：重写的按树内 bump 次序发出生序号（M9），照抄的沿用上一版的 key。
fn note_mapped_unit(
    role: Role,
    planned: PlannedUnit,
    rewritten_key: KeyFields,
    disk: &Disk,
    mapping_keys: &mut BTreeSet<KeyFields>,
    mapping_keys_of_rewritten: &mut HashMap<Role, KeyFields>,
) {
    let key = match planned {
        PlannedUnit::Rewritten => {
            mapping_keys_of_rewritten.insert(role, rewritten_key);
            rewritten_key
        }
        PlannedUnit::Carried(unit) => mapping_key_of_carried(disk, unit),
    };
    assert!(mapping_keys.insert(key), "进映射的单元各一把 key，互不相撞");
}

fn code_two_origin_as_planned(origin: CodeTwoOrigin, previous: &CodeTwoTreeVersion) -> PlannedUnit {
    match origin {
        CodeTwoOrigin::CarriedFrom(previous_position) => PlannedUnit::Carried(previous.unit_at(previous_position)),
        CodeTwoOrigin::Rewritten => PlannedUnit::Rewritten,
    }
}

/// 把逻辑计划与这一轮的分配记录树节点排成一次发布（`transaction.rs:5235-5344`、`:5390-5649`）。
fn assemble_publish(
    plan: &LogicalPlan,
    previous: &Version,
    disk: &Disk,
    allocation_nodes_after: &BTreeSet<AllocationNode>,
    allocation_rewritten: &BTreeSet<AllocationNode>,
) -> AssembledPublish {
    let allocation_nodes: Vec<(AllocationNode, PlannedUnit)> = allocation_nodes_after
        .iter()
        .map(|node| {
            let origin = if allocation_rewritten.contains(node) {
                PlannedUnit::Rewritten
            } else {
                match previous.allocation_nodes.get(node) {
                    Some(unit) => PlannedUnit::Carried(*unit),
                    None => PlannedUnit::Rewritten,
                }
            };
            (*node, origin)
        })
        .collect();
    let mut releases = plan.released_units.clone();
    for (node, unit) in &previous.allocation_nodes {
        if allocation_rewritten.contains(node) || !allocation_nodes_after.contains(node) {
            releases.push(*unit);
        }
    }
    let accounting_plan = plan_code_two_tree(&previous.accounting.shape, &plan.accounting_keys, accounting_tree_capacity());
    for position in &accounting_plan.replaced_previous {
        releases.push(previous.accounting.unit_at(*position));
    }

    let mut mapping_keys = BTreeSet::new();
    let mut mapping_keys_of_rewritten = HashMap::new();
    for (position, planned) in plan.data_units.iter().enumerate() {
        let rewritten_key = [
            UNIT_CLASS_DATA,
            TREE_IDENTIFIER_EXTENT,
            plan.txg,
            plan.instance,
            plan.transaction_base + length_as_u64(position),
        ];
        note_mapped_unit(Role::Data(position), *planned, rewritten_key, disk, &mut mapping_keys, &mut mapping_keys_of_rewritten);
    }
    let mut extent_sequence = 0;
    let extent_nodes = plan
        .extent_lower
        .iter()
        .map(|(position, planned)| (Role::ExtentLower(*position), *planned))
        .chain(std::iter::once((Role::ExtentRoot, plan.extent_root)));
    for (role, planned) in extent_nodes {
        let rewritten_key = node_mapping_key(plan, UNIT_CLASS_INDEX_NODE, TREE_IDENTIFIER_EXTENT, extent_sequence);
        if planned == PlannedUnit::Rewritten {
            extent_sequence += 1;
        }
        note_mapped_unit(role, planned, rewritten_key, disk, &mut mapping_keys, &mut mapping_keys_of_rewritten);
    }
    let mut inode_sequence = 0;
    for (position, container) in plan.inode_containers.iter().enumerate() {
        let rewritten_key = node_mapping_key(plan, UNIT_CLASS_PACKED, TREE_IDENTIFIER_INODE, inode_sequence);
        if container.origin == PlannedUnit::Rewritten {
            inode_sequence += 1;
        }
        note_mapped_unit(Role::InodeContainer(position), container.origin, rewritten_key, disk, &mut mapping_keys, &mut mapping_keys_of_rewritten);
    }
    let inode_root_key = node_mapping_key(plan, UNIT_CLASS_INDEX_NODE, TREE_IDENTIFIER_INODE, inode_sequence);
    note_mapped_unit(Role::InodeRoot, plan.inode_root, inode_root_key, disk, &mut mapping_keys, &mut mapping_keys_of_rewritten);
    let mut allocation_sequence = 0;
    for (node, planned) in &allocation_nodes {
        let rewritten_key = node_mapping_key(plan, UNIT_CLASS_INDEX_NODE, TREE_IDENTIFIER_ALLOCATION_RECORDS, allocation_sequence);
        if *planned == PlannedUnit::Rewritten {
            allocation_sequence += 1;
        }
        note_mapped_unit(Role::AllocationNode(*node), *planned, rewritten_key, disk, &mut mapping_keys, &mut mapping_keys_of_rewritten);
    }
    let mut accounting_sequence = 0;
    for ((position, _), origin) in accounting_plan.shape.nodes.iter().zip(&accounting_plan.origins) {
        let planned = code_two_origin_as_planned(*origin, &previous.accounting);
        let rewritten_key = node_mapping_key(plan, UNIT_CLASS_INDEX_NODE, TREE_IDENTIFIER_ACCOUNTING, accounting_sequence);
        if planned == PlannedUnit::Rewritten {
            accounting_sequence += 1;
        }
        note_mapped_unit(Role::AccountingNode(*position), planned, rewritten_key, disk, &mut mapping_keys, &mut mapping_keys_of_rewritten);
    }
    if let Some(planned) = plan.deleted_root {
        let rewritten_key = node_mapping_key(plan, UNIT_CLASS_INDEX_NODE, TREE_IDENTIFIER_DELETED_INODES, 0);
        note_mapped_unit(Role::DeletedInodesRoot, planned, rewritten_key, disk, &mut mapping_keys, &mut mapping_keys_of_rewritten);
    }
    let mut tombstone_sequence = 0;
    for (position, container) in plan.tombstone_containers.iter().enumerate() {
        let rewritten_key = node_mapping_key(plan, UNIT_CLASS_PACKED, TREE_IDENTIFIER_TOMBSTONE, tombstone_sequence);
        if container.origin == PlannedUnit::Rewritten {
            tombstone_sequence += 1;
        }
        note_mapped_unit(Role::TombstoneContainer(position), container.origin, rewritten_key, disk, &mut mapping_keys, &mut mapping_keys_of_rewritten);
    }
    if let Some(planned) = plan.tombstone_root {
        let rewritten_key = node_mapping_key(plan, UNIT_CLASS_INDEX_NODE, TREE_IDENTIFIER_TOMBSTONE, tombstone_sequence);
        note_mapped_unit(Role::TombstoneRoot, planned, rewritten_key, disk, &mut mapping_keys, &mut mapping_keys_of_rewritten);
    }
    let mapping_plan = plan_code_two_tree(&previous.mapping.shape, &mapping_keys, mapping_tree_capacity());
    for position in &mapping_plan.replaced_previous {
        releases.push(previous.mapping.unit_at(*position));
    }
    releases.push(previous.tree_table);

    let mut roles = Vec::new();
    for (position, planned) in plan.data_units.iter().enumerate() {
        if *planned == PlannedUnit::Rewritten {
            roles.push(Role::Data(position));
        }
    }
    if plan.instance_table_rewritten {
        roles.push(Role::InstanceTable);
    }
    for (position, planned) in &plan.extent_lower {
        if *planned == PlannedUnit::Rewritten {
            roles.push(Role::ExtentLower(*position));
        }
    }
    if plan.extent_root == PlannedUnit::Rewritten {
        roles.push(Role::ExtentRoot);
    }
    for (position, container) in plan.inode_containers.iter().enumerate() {
        if container.origin == PlannedUnit::Rewritten {
            roles.push(Role::InodeContainer(position));
        }
    }
    if plan.inode_root == PlannedUnit::Rewritten {
        roles.push(Role::InodeRoot);
    }
    for (node, planned) in &allocation_nodes {
        if *planned == PlannedUnit::Rewritten {
            roles.push(Role::AllocationNode(*node));
        }
    }
    for ((position, _), origin) in accounting_plan.shape.nodes.iter().zip(&accounting_plan.origins) {
        if *origin == CodeTwoOrigin::Rewritten {
            roles.push(Role::AccountingNode(*position));
        }
    }
    if plan.deleted_root == Some(PlannedUnit::Rewritten) {
        roles.push(Role::DeletedInodesRoot);
    }
    for (position, container) in plan.tombstone_containers.iter().enumerate() {
        if container.origin == PlannedUnit::Rewritten {
            roles.push(Role::TombstoneContainer(position));
        }
    }
    if plan.tombstone_root == Some(PlannedUnit::Rewritten) {
        roles.push(Role::TombstoneRoot);
    }
    for ((position, _), origin) in mapping_plan.shape.nodes.iter().zip(&mapping_plan.origins) {
        if *origin == CodeTwoOrigin::Rewritten {
            roles.push(Role::MappingNode(*position));
        }
    }
    roles.push(Role::TreeTable);
    AssembledPublish {
        roles,
        allocation_nodes,
        accounting_plan,
        mapping_plan,
        mapping_keys_of_rewritten,
        releases,
    }
}

/// 分配记录树重写哪几个节点走固定点（M8，`transaction.rs:6627-6704`）：每一轮在分配器拷贝上先释放、再按这一轮的角色取落点，
/// 内容变了的叶与装着记录的节点集对得上才定下。
pub(super) fn settle_publish(
    plan: &LogicalPlan,
    previous: &Version,
    disk: &Disk,
    allocator: &Allocator,
) -> Result<AssembledPublish, PlacementRefused> {
    let previous_nodes: BTreeSet<AllocationNode> = previous.allocation_nodes.keys().copied().collect();
    let mut rewritten: BTreeSet<AllocationNode> = BTreeSet::new();
    let mut nodes_after = previous_nodes.clone();
    for _round in 0..64 {
        let rewritten_among_the_nodes_after: BTreeSet<AllocationNode> =
            rewritten.intersection(&nodes_after).copied().collect();
        let assembled = assemble_publish(plan, previous, disk, &nodes_after, &rewritten_among_the_nodes_after);
        let rewritten_by_the_plan: BTreeSet<AllocationNode> = assembled
            .allocation_nodes
            .iter()
            .filter(|(_, planned)| *planned == PlannedUnit::Rewritten)
            .map(|(node, _)| *node)
            .collect();
        let mut rehearsal = allocator.rehearsal_copy();
        for unit in &assembled.releases {
            let released = &disk.units[*unit];
            rehearsal.release(released.slot, released.span_slots, plan.txg);
        }
        let changed_by_the_release = allocation_nodes_whose_contents_changed(allocator, &rehearsal);
        if !changed_by_the_release.is_subset(&rewritten_by_the_plan) {
            rewritten.extend(changed_by_the_release);
            nodes_after.extend(rehearsal.nodes_holding_records());
            continue;
        }
        for role in &assembled.roles {
            rehearsal.allocate(role.placement_rule(), plan.txg, PLANTED_UNIT)?;
        }
        let changed = allocation_nodes_whose_contents_changed(allocator, &rehearsal);
        let holding_records = rehearsal.nodes_holding_records();
        if holding_records == nodes_after && changed.is_subset(&rewritten_by_the_plan) {
            return Ok(assembled);
        }
        rewritten.extend(changed);
        nodes_after = holding_records;
    }
    panic!("分配记录树的重写集 64 轮没定下来：产品路径上三四轮就定（M8）");
}
