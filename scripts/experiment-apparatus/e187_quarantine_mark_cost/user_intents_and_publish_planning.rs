//! 用户意图的规划、发布计划、放置与释放、发布的规划与施加。

use super::*;

/// 这次发布要放下的一个新单元。
#[derive(Clone, Copy, Debug)]
struct PlannedUnit {
    pub(super) unit: UnitIdentifier,
    pub(super) structure: WrittenStructure,
    footprint: Footprint,
    pub(super) mapping_key: Option<MappingKey>,
    forced_slot: Option<u64>,
}

fn planned(unit: UnitIdentifier, structure: WrittenStructure, footprint: Footprint, mapping_key: Option<MappingKey>) -> PlannedUnit {
    PlannedUnit { unit, structure, footprint, mapping_key, forced_slot: None }
}

/// 与分配记录树重写集无关的那一半：用户看得见的那一版怎么变、换下谁、映射删插哪些 key、记录先改哪几条、乙的隔离树怎么变。
struct UserIntent {
    pub(super) txg: u64,
    fresh: FreshUnits,
    pub(super) data_units: Vec<PlannedUnit>,
    /// extent 叶、extent 根、inode 叶容器、inode 根（bump 次序里排在分配记录树之前的那几样）。
    nodes_before_the_allocation_record_tree: Vec<PlannedUnit>,
    accounting: PlannedUnit,
    quarantine_tree_after: Option<CodeTwoNode<QuarantineEntryKey>>,
    quarantine_tree_nodes: Vec<PlannedUnit>,
    tree_table: PlannedUnit,
    version_after: UserVisibleVersion,
    released_data_units: Vec<UnitIdentifier>,
    released_nodes: Vec<UnitIdentifier>,
    mapping_deleted: BTreeSet<MappingKey>,
    mapping_inserted: BTreeSet<MappingKey>,
    record_changes: BTreeMap<u64, Option<AllocationRecord>>,
    resurrected_deferred: Vec<(u64, u64)>,
    quarantine_releases: Vec<(u64, u64)>,
    newly_quarantined: Option<UnitIdentifier>,
    pub(super) revived: Vec<UnitIdentifier>,
    released_quarantined: Option<UnitIdentifier>,
    quarantine_tree_registered_by_this_publish: bool,
}

fn current_record(state: &PoolState, record_changes: &BTreeMap<u64, Option<AllocationRecord>>, slot: u64) -> Option<AllocationRecord> {
    match record_changes.get(&slot) {
        Some(changed) => *changed,
        None => state.records.get(&slot).copied(),
    }
}

pub(super) fn quarantine_entries_of(slot: u64) -> [QuarantineEntryKey; 2] {
    [(0, slot), (1, slot)]
}

/// 新写一组 extent 叶与 extent 根（`rewritten_leaves` 升序），映射 key 先叶后根逐树从 0 起数出生序号。
fn plan_extent_nodes(
    txg: u64,
    rewritten_leaves: &BTreeSet<usize>,
    has_root: bool,
    version_after: &mut UserVisibleVersion,
    fresh: &mut FreshUnits,
) -> Vec<PlannedUnit> {
    let mut nodes = Vec::new();
    let mut sequence = 0u64;
    for leaf in rewritten_leaves {
        let unit = fresh.issue();
        version_after.extent_leaves[*leaf] = unit;
        nodes.push(planned(unit, WrittenStructure::ExtentNode, Footprint::OneSlot, Some((UNIT_CLASS_INDEX_NODE, EXTENT_TREE, txg, WRITE_ORDER_INSTANCE, sequence))));
        sequence += 1;
    }
    if has_root {
        let unit = fresh.issue();
        version_after.extent_root = Some(unit);
        nodes.push(planned(unit, WrittenStructure::ExtentNode, Footprint::OneSlot, Some((UNIT_CLASS_INDEX_NODE, EXTENT_TREE, txg, WRITE_ORDER_INSTANCE, sequence))));
    }
    nodes
}

/// inode 树叶容器（码 3，两槽 32768 对齐）与 inode 根（码 2），每次普通发布都重写。
fn plan_inode_nodes(txg: u64, version_after: &mut UserVisibleVersion, fresh: &mut FreshUnits) -> Vec<PlannedUnit> {
    let container = fresh.issue();
    let root = fresh.issue();
    version_after.inode_leaf_container = container;
    version_after.inode_root = root;
    vec![
        planned(container, WrittenStructure::InodeNode, Footprint::TwoSlotsAligned, Some((UNIT_CLASS_PACKED, INODE_TREE, txg, WRITE_ORDER_INSTANCE, 0))),
        planned(root, WrittenStructure::InodeNode, Footprint::OneSlot, Some((UNIT_CLASS_INDEX_NODE, INODE_TREE, txg, WRITE_ORDER_INSTANCE, 1))),
    ]
}

fn extent_leaf_of(data_index: u32) -> usize {
    usize_of(u64::from(data_index) / EXTENT_LEAF_ENTRIES)
}

/// 用户看得见的那一版这次怎么变（新建文件、覆盖写、回退三种），与分配记录树的重写集无关。
fn user_intent(state: &PoolState, request: &PublishRequest, arm: Arm) -> UserIntent {
    let txg = state.txg + 1;
    let mut fresh = FreshUnits::starting_at(state.next_unit());
    let mut data_units = Vec::new();
    let mut nodes_before_the_allocation_record_tree = Vec::new();
    let mut released_data_units = Vec::new();
    let mut released_nodes = Vec::new();
    let mut mapping_deleted = BTreeSet::new();
    let mut record_changes = BTreeMap::new();
    let mut resurrected_deferred = Vec::new();
    let mut quarantine_releases = Vec::new();
    let mut newly_quarantined = None;
    let mut revived = Vec::new();
    let mut released_quarantined = None;
    let mut quarantine_tree_inserted: BTreeSet<QuarantineEntryKey> = BTreeSet::new();
    let mut quarantine_tree_deleted: BTreeSet<QuarantineEntryKey> = BTreeSet::new();
    let mut extra_mapping_inserted: BTreeSet<MappingKey> = BTreeSet::new();
    let version_after = match &request.kind {
        PublishKind::CreateFile { data_unit_count } => {
            assert!(state.version.is_none(), "新建文件只做一次");
            let leaf_count = usize_of(u64::from(*data_unit_count).div_ceil(EXTENT_LEAF_ENTRIES));
            let placeholder = UnitIdentifier(u64::MAX);
            let mut version_after = UserVisibleVersion {
                data_units: Vec::new(),
                extent_leaves: vec![placeholder; leaf_count],
                extent_root: None,
                inode_leaf_container: placeholder,
                inode_root: placeholder,
            };
            for index in 0..*data_unit_count {
                let unit = fresh.issue();
                version_after.data_units.push(unit);
                let mut data_unit = planned(unit, WrittenStructure::DataUnit, Footprint::TwoSlotsAligned, Some((UNIT_CLASS_DATA, EXTENT_TREE, txg, WRITE_ORDER_INSTANCE, u64::from(index))));
                data_unit.forced_slot = request.forced_data_slot(index);
                data_units.push(data_unit);
            }
            let all_leaves: BTreeSet<usize> = (0..leaf_count).collect();
            nodes_before_the_allocation_record_tree.extend(plan_extent_nodes(txg, &all_leaves, leaf_count > 1, &mut version_after, &mut fresh));
            nodes_before_the_allocation_record_tree.extend(plan_inode_nodes(txg, &mut version_after, &mut fresh));
            version_after
        }
        PublishKind::Overwrite { data_indices, quarantine_victim, release_quarantined } => {
            let before = state.version.as_ref().expect("覆盖写之前文件已经建好");
            let mut version_after = before.clone();
            let sorted_indices: BTreeSet<u32> = data_indices.iter().copied().collect();
            assert_eq!(sorted_indices.len(), data_indices.len(), "一次发布里同一个数据单元只覆盖写一次");
            for (sequence, index) in sorted_indices.iter().enumerate() {
                let old_unit = before.data_units[usize_of(u64::from(*index))];
                mapping_deleted.insert(state.unit(old_unit).mapping_key.expect("数据单元进映射"));
                let unit = fresh.issue();
                version_after.data_units[usize_of(u64::from(*index))] = unit;
                let mut data_unit = planned(unit, WrittenStructure::DataUnit, Footprint::TwoSlotsAligned, Some((UNIT_CLASS_DATA, EXTENT_TREE, txg, WRITE_ORDER_INSTANCE, u64_of(sequence))));
                data_unit.forced_slot = request.forced_data_slot(*index);
                data_units.push(data_unit);
                let is_the_victim = *quarantine_victim == Some(*index);
                match (is_the_victim, arm) {
                    (true, Arm::NoMark | Arm::SpanBit | Arm::QuarantineTree) => {
                        // 释放之前读盘核没对上：两块盘上的记录都留在已分配，映射条目照常去掉（D19 已定项 5 硬规则 1）。
                        let old_information = state.unit(old_unit);
                        newly_quarantined = Some(old_unit);
                        let mut quarantined_record = state.records.get(&old_information.slot).copied().expect("被隔离单元有记录");
                        match arm {
                            Arm::SpanBit => quarantined_record.carries_the_quarantine_mark = true,
                            Arm::QuarantineTree => quarantine_tree_inserted.extend(quarantine_entries_of(old_information.slot)),
                            Arm::NoMark | Arm::PlainRelease => {}
                        }
                        record_changes.insert(old_information.slot, Some(quarantined_record));
                    }
                    (true, Arm::PlainRelease) | (false, Arm::NoMark | Arm::SpanBit | Arm::QuarantineTree | Arm::PlainRelease) => {
                        released_data_units.push(old_unit);
                    }
                }
            }
            let rewritten_leaves: BTreeSet<usize> = sorted_indices.iter().map(|index| extent_leaf_of(*index)).collect();
            for leaf in &rewritten_leaves {
                released_nodes.push(before.extent_leaves[*leaf]);
            }
            released_nodes.extend(before.extent_root);
            released_nodes.push(before.inode_leaf_container);
            released_nodes.push(before.inode_root);
            for replaced_node in &released_nodes {
                mapping_deleted.insert(state.unit(*replaced_node).mapping_key.expect("extent 与 inode 的节点进映射"));
            }
            nodes_before_the_allocation_record_tree.extend(plan_extent_nodes(txg, &rewritten_leaves, before.extent_root.is_some(), &mut version_after, &mut fresh));
            nodes_before_the_allocation_record_tree.extend(plan_inode_nodes(txg, &mut version_after, &mut fresh));
            if let Some(unit) = release_quarantined {
                // 释放一个隔离的记录：两块盘上那条改成已释放 + 释放代，落点进 defer，记号随之清掉（甲）或从隔离树里删掉（乙）。
                let quarantined = state.quarantined.get(unit).copied().expect("释放的是一个隔离着的单元");
                let mut released_record = state.records.get(&quarantined.slot).copied().expect("隔离的记录在");
                released_record.is_released = true;
                released_record.generation = txg;
                released_record.carries_the_quarantine_mark = false;
                record_changes.insert(quarantined.slot, Some(released_record));
                quarantine_releases.push((quarantined.slot, quarantined.span_slots));
                released_quarantined = Some(*unit);
                if arm == Arm::QuarantineTree {
                    quarantine_tree_deleted.extend(quarantine_entries_of(quarantined.slot));
                }
            }
            version_after
        }
        PublishKind::RollBack { target_txg } => {
            let before = state.version.as_ref().expect("回退之前文件已经建好");
            let target = state.kept_versions.get(target_txg).expect("回退目标在候选窗口里、那一版留着").clone();
            let current_units = before.units();
            let target_units = target.units();
            // 现行那一版引用、目标不引用的释放；目标引用、还在 defer 里的复活；目标引用、隔离着的那几个重新被引用（记号清掉）。
            for unit in current_units.difference(&target_units) {
                mapping_deleted.insert(state.unit(*unit).mapping_key.expect("用户看得见的单元都进映射"));
                match state.unit(*unit).structure {
                    WrittenStructure::DataUnit => released_data_units.push(*unit),
                    WrittenStructure::ExtentNode
                    | WrittenStructure::InodeNode
                    | WrittenStructure::AllocationRecordTreeNode
                    | WrittenStructure::AccountingNode
                    | WrittenStructure::QuarantineTreeNode
                    | WrittenStructure::CentralMappingNode
                    | WrittenStructure::TreeTableUnit => released_nodes.push(*unit),
                }
            }
            for unit in target_units.difference(&current_units) {
                let information = state.unit(*unit);
                extra_mapping_inserted.insert(information.mapping_key.expect("用户看得见的单元都进映射"));
                let record = state.records.get(&information.slot).copied().expect("目标那一版引用的单元还有记录");
                if let Some(quarantined) = state.quarantined.get(unit) {
                    let mut revived_record = record;
                    revived_record.carries_the_quarantine_mark = false;
                    record_changes.insert(quarantined.slot, Some(revived_record));
                    if arm == Arm::QuarantineTree {
                        quarantine_tree_deleted.extend(quarantine_entries_of(quarantined.slot));
                    }
                    revived.push(*unit);
                } else {
                    assert!(record.is_released, "目标那一版引用、现行那一版不引用的单元在现行的账里是已释放的");
                    resurrected_deferred.push((record.generation, information.slot));
                    record_changes.insert(
                        information.slot,
                        Some(AllocationRecord { span_slots: record.span_slots, is_released: false, generation: information.birth_txg, carries_the_quarantine_mark: false }),
                    );
                }
            }
            target
        }
    };
    // 记账树根兼叶每次发布都重写。
    if let Some(old_accounting) = state.accounting {
        released_nodes.push(old_accounting);
        mapping_deleted.insert(state.unit(old_accounting).mapping_key.expect("记账树进映射"));
    }
    let accounting = planned(fresh.issue(), WrittenStructure::AccountingNode, Footprint::OneSlot, Some((UNIT_CLASS_INDEX_NODE, ACCOUNTING_TREE, txg, WRITE_ORDER_INSTANCE, 0)));
    // 乙的隔离树：照码 2 规则长；节点进中央映射；删空之后零根、树表条目留着。
    let quarantine_tree_after = if quarantine_tree_inserted.is_empty() && quarantine_tree_deleted.is_empty() {
        state.quarantine_tree.clone()
    } else {
        plan_the_tree_after_this_publish(&state.quarantine_tree, &quarantine_tree_deleted, &quarantine_tree_inserted, quarantine_tree_capacity(), &mut fresh)
    };
    let quarantine_units_after: BTreeSet<UnitIdentifier> = units_of_the_tree_in_bump_order(&quarantine_tree_after).into_iter().collect();
    for old_node in units_of_the_tree_in_bump_order(&state.quarantine_tree) {
        if !quarantine_units_after.contains(&old_node) {
            released_nodes.push(old_node);
            mapping_deleted.insert(state.unit(old_node).mapping_key.expect("隔离树节点进映射"));
        }
    }
    let mut quarantine_tree_nodes = Vec::new();
    for unit in units_of_the_tree_in_bump_order(&quarantine_tree_after) {
        if fresh.is_fresh(unit) {
            let sequence = u64_of(quarantine_tree_nodes.len());
            let mut node = planned(unit, WrittenStructure::QuarantineTreeNode, Footprint::OneSlot, Some((UNIT_CLASS_INDEX_NODE, QUARANTINE_TREE, txg, WRITE_ORDER_INSTANCE, sequence)));
            if quarantine_tree_nodes.is_empty() {
                node.forced_slot = request.forced_quarantine_tree_node_slot();
            }
            quarantine_tree_nodes.push(node);
        }
    }
    let quarantine_tree_registered_by_this_publish = !state.quarantine_tree_registered && quarantine_tree_after.is_some();
    released_nodes.extend(state.tree_table);
    let tree_table = planned(fresh.issue(), WrittenStructure::TreeTableUnit, Footprint::OneSlot, None);
    let mut mapping_inserted: BTreeSet<MappingKey> = data_units
        .iter()
        .chain(nodes_before_the_allocation_record_tree.iter())
        .chain(std::iter::once(&accounting))
        .chain(quarantine_tree_nodes.iter())
        .filter_map(|planned_unit| planned_unit.mapping_key)
        .collect();
    mapping_inserted.extend(extra_mapping_inserted);
    UserIntent {
        txg,
        fresh,
        data_units,
        nodes_before_the_allocation_record_tree,
        accounting,
        quarantine_tree_after,
        quarantine_tree_nodes,
        tree_table,
        version_after,
        released_data_units,
        released_nodes,
        mapping_deleted,
        mapping_inserted,
        record_changes,
        resurrected_deferred,
        quarantine_releases,
        newly_quarantined,
        revived,
        released_quarantined,
        quarantine_tree_registered_by_this_publish,
    }
}

/// 走完固定点的一次发布：要改哪些记录、取了哪些槽、换下哪些落点、新写了哪些单元、之后的各棵树。
pub(super) struct PublishPlan {
    pub(super) txg: u64,
    fresh: FreshUnits,
    new_units: BTreeMap<UnitIdentifier, UnitInformation>,
    record_changes: BTreeMap<u64, Option<AllocationRecord>>,
    released_placements: Vec<(u64, u64)>,
    resurrected_deferred: Vec<(u64, u64)>,
    pub(super) taken: BTreeSet<u64>,
    pub(super) open_segment_start: Option<u64>,
    pub(super) bump_cursor: u64,
    pub(super) opened_segments: BTreeSet<usize>,
    pub(super) placements_that_fell_back: u64,
    version_after: UserVisibleVersion,
    allocation_nodes_after: BTreeMap<AllocationNodePosition, UnitIdentifier>,
    pub(super) rewritten_allocation_positions: BTreeSet<AllocationNodePosition>,
    accounting_after: UnitIdentifier,
    tree_table_after: UnitIdentifier,
    mapping_after: Option<CodeTwoNode<MappingKey>>,
    quarantine_tree_after: Option<CodeTwoNode<QuarantineEntryKey>>,
    quarantine_tree_registered_by_this_publish: bool,
    newly_quarantined: Option<UnitIdentifier>,
    pub(super) revived: Vec<UnitIdentifier>,
    released_quarantined: Option<UnitIdentifier>,
}

impl PublishPlan {
    fn new_unit_list(&self) -> Vec<UnitInformation> {
        self.new_units.values().copied().collect()
    }

    pub(super) fn writes(&self, padding_named_roles: u64) -> PublishWrites {
        tally_publish_writes(&self.new_unit_list(), padding_named_roles, self.quarantine_tree_registered_by_this_publish)
    }
}

/// 新落点：罩住的已回收记录随这次分配删掉，起点上那条（已回收的）改写成仍分配（`allocator.rs` 的 `AllocationRecord` 文档）。
fn record_a_new_placement(state: &PoolState, record_changes: &mut BTreeMap<u64, Option<AllocationRecord>>, slot: u64, span_slots: u64, txg: u64) {
    for start in slot.saturating_sub(1)..slot + span_slots {
        if start == slot {
            continue;
        }
        if let Some(record) = current_record(state, record_changes, start) {
            if start + u64::from(record.span_slots) > slot {
                assert!(record.is_released, "新落点罩住的记录是已回收的");
                record_changes.insert(start, None);
            }
        }
    }
    if let Some(existing) = current_record(state, record_changes, slot) {
        assert!(existing.is_released, "新落点起点上的记录是已回收的");
    }
    let span_slots = u16::try_from(span_slots).expect("一个单元两槽以内");
    record_changes.insert(slot, Some(AllocationRecord { span_slots, is_released: false, generation: txg, carries_the_quarantine_mark: false }));
}

fn release_a_unit(state: &PoolState, record_changes: &mut BTreeMap<u64, Option<AllocationRecord>>, released_placements: &mut Vec<(u64, u64)>, unit: UnitIdentifier, txg: u64) {
    let information = state.unit(unit);
    let mut record = current_record(state, record_changes, information.slot).expect("换下的单元有记录");
    assert!(!record.is_released, "同一个落点释放了两次");
    record.is_released = true;
    record.generation = txg;
    record_changes.insert(information.slot, Some(record));
    released_placements.push((information.slot, information.span_slots));
}

/// 按给定的分配记录树重写集走一遍：映射树怎么变、释放、取落点、改记录（`transaction.rs` 的 `settle_the_allocation_record_tree` 每一轮做的事）。
fn walk_the_publish(state: &PoolState, intent: &UserIntent, rewrite_set: &BTreeSet<AllocationNodePosition>) -> PublishPlan {
    let txg = intent.txg;
    let mut fresh = intent.fresh;
    let mut released_nodes = intent.released_nodes.clone();
    let mut mapping_deleted = intent.mapping_deleted.clone();
    let mut mapping_inserted = intent.mapping_inserted.clone();
    let mut allocation_nodes_after = state.allocation_nodes.clone();
    let mut allocation_nodes = Vec::new();
    for (sequence, position) in rewrite_set.iter().enumerate() {
        if let Some(old_node) = state.allocation_nodes.get(position) {
            released_nodes.push(*old_node);
            mapping_deleted.insert(state.unit(*old_node).mapping_key.expect("分配记录树节点进映射"));
        }
        let unit = fresh.issue();
        let mapping_key = (UNIT_CLASS_INDEX_NODE, ALLOCATION_RECORD_TREE, txg, WRITE_ORDER_INSTANCE, u64_of(sequence));
        mapping_inserted.insert(mapping_key);
        allocation_nodes_after.insert(*position, unit);
        allocation_nodes.push(planned(unit, WrittenStructure::AllocationRecordTreeNode, Footprint::OneSlot, Some(mapping_key)));
    }
    let mapping_after = plan_the_tree_after_this_publish(&state.mapping, &mapping_deleted, &mapping_inserted, mapping_capacity(), &mut fresh);
    let mapping_units_after = units_of_the_tree_in_bump_order(&mapping_after);
    let mapping_unit_set: BTreeSet<UnitIdentifier> = mapping_units_after.iter().copied().collect();
    released_nodes.extend(units_of_the_tree_in_bump_order(&state.mapping).into_iter().filter(|old_node| !mapping_unit_set.contains(old_node)));
    let mapping_nodes: Vec<PlannedUnit> = mapping_units_after
        .iter()
        .filter(|unit| fresh.is_fresh(**unit))
        .map(|unit| planned(*unit, WrittenStructure::CentralMappingNode, Footprint::OneSlot, None))
        .collect();

    // 释放先于分配：换下的落点进 defer、槽仍占着，这次的落点不会落到它们上面。
    let mut record_changes = intent.record_changes.clone();
    let mut released_placements = Vec::new();
    for unit in &intent.released_data_units {
        release_a_unit(state, &mut record_changes, &mut released_placements, *unit, txg);
    }
    for replaced_node in &released_nodes {
        release_a_unit(state, &mut record_changes, &mut released_placements, *replaced_node, txg);
    }
    released_placements.extend(intent.quarantine_releases.iter().copied());

    // 取落点：数据单元先取（最低的空槽对），提交内生块再按 bump 次序从开放段取（D3（空间分配） 已定项 10 ⑤）。
    let mut walk = PlacementWalk::over(state);
    let mut new_units = BTreeMap::new();
    let commit_generated = intent
        .nodes_before_the_allocation_record_tree
        .iter()
        .chain(allocation_nodes.iter())
        .chain(std::iter::once(&intent.accounting))
        .chain(intent.quarantine_tree_nodes.iter())
        .chain(mapping_nodes.iter())
        .chain(std::iter::once(&intent.tree_table));
    let in_placement_order: Vec<&PlannedUnit> = intent.data_units.iter().chain(commit_generated).collect();
    // 微历史构造里指定了槽的那几个先占上，分配器再从剩下的槽里取。
    let mut forced_slots: BTreeMap<UnitIdentifier, u64> = BTreeMap::new();
    for planned_unit in &in_placement_order {
        if let Some(forced_slot) = planned_unit.forced_slot {
            forced_slots.insert(planned_unit.unit, walk.force(forced_slot, planned_unit.footprint));
        }
    }
    for planned_unit in in_placement_order {
        let slot = match (forced_slots.get(&planned_unit.unit), planned_unit.structure) {
            (Some(forced_slot), _any_structure) => *forced_slot,
            (None, WrittenStructure::DataUnit) => walk.allocate_data_unit(),
            (
                None,
                WrittenStructure::ExtentNode
                | WrittenStructure::InodeNode
                | WrittenStructure::AllocationRecordTreeNode
                | WrittenStructure::AccountingNode
                | WrittenStructure::QuarantineTreeNode
                | WrittenStructure::CentralMappingNode
                | WrittenStructure::TreeTableUnit,
            ) => walk.allocate_commit_generated(planned_unit.footprint),
        };
        record_a_new_placement(state, &mut record_changes, slot, planned_unit.footprint.slots(), txg);
        new_units.insert(
            planned_unit.unit,
            UnitInformation { slot, span_slots: planned_unit.footprint.slots(), mapping_key: planned_unit.mapping_key, structure: planned_unit.structure, birth_txg: txg },
        );
    }
    PublishPlan {
        txg,
        fresh,
        new_units,
        record_changes,
        released_placements,
        resurrected_deferred: intent.resurrected_deferred.clone(),
        taken: walk.taken,
        open_segment_start: walk.open_segment_start,
        bump_cursor: walk.bump_cursor,
        opened_segments: walk.opened_segments,
        placements_that_fell_back: walk.placements_that_fell_back,
        version_after: intent.version_after.clone(),
        allocation_nodes_after,
        rewritten_allocation_positions: rewrite_set.clone(),
        accounting_after: intent.accounting.unit,
        tree_table_after: intent.tree_table.unit,
        mapping_after,
        quarantine_tree_after: intent.quarantine_tree_after.clone(),
        quarantine_tree_registered_by_this_publish: intent.quarantine_tree_registered_by_this_publish,
        newly_quarantined: intent.newly_quarantined,
        revived: intent.revived.clone(),
        released_quarantined: intent.released_quarantined,
    }
}

/// 照走出来的记录，内容变了的节点（记录清单不同的叶连全部祖先与根）与这一版有记录却还没有节点的叶。
fn allocation_nodes_to_rewrite(state: &PoolState, record_changes: &BTreeMap<u64, Option<AllocationRecord>>) -> BTreeSet<AllocationNodePosition> {
    let mut positions = BTreeSet::new();
    for (slot, record_after) in record_changes {
        if state.records.get(slot) != record_after.as_ref() {
            leaf_and_its_ancestors(state.root_level, leaf_of_slot(*slot), &mut positions);
        }
    }
    for leaf in &state.leaves_with_records {
        if !state.allocation_nodes.contains_key(&AllocationNodePosition { level: 0, device: 0, index_in_device: *leaf }) {
            leaf_and_its_ancestors(state.root_level, *leaf, &mut positions);
        }
    }
    positions
}

/// 固定点：从「重写集空」起，按它走一遍、算出要重写的节点，都在重写集里就定下，否则并进去再走（多罩的节点照样重写，合法的 COW）。
pub(super) fn plan_publish(state: &PoolState, request: &PublishRequest, arm: Arm) -> PublishPlan {
    let intent = user_intent(state, request, arm);
    let mut rewrite_set = BTreeSet::new();
    for _round in 0..SETTLING_ROUNDS_LIMIT {
        let walked = walk_the_publish(state, &intent, &rewrite_set);
        let needed = allocation_nodes_to_rewrite(state, &walked.record_changes);
        if needed.is_subset(&rewrite_set) {
            return walked;
        }
        rewrite_set.extend(needed);
    }
    panic!("分配记录树重写集 {SETTLING_ROUNDS_LIMIT} 轮没定下：模型里重写集只增不减，以池里的节点数为上界");
}

/// 把走完固定点的那一次发布落进池状态，再按新的最新 txg 回收（释放代 ≤ 最新 txg − K 的落点回到空闲）。
pub(super) fn apply_publish(state: &mut PoolState, plan: PublishPlan) {
    for slot in &plan.taken {
        state.set_occupied(*slot, true);
    }
    for segment in &plan.opened_segments {
        state.is_cluster_segment[*segment] = true;
    }
    state.open_segment_start = plan.open_segment_start;
    state.bump_cursor = plan.bump_cursor;
    for (slot, record_after) in &plan.record_changes {
        match record_after {
            Some(record) => {
                state.records.insert(*slot, *record);
                state.leaves_with_records.insert(leaf_of_slot(*slot));
            }
            None => {
                state.records.remove(slot);
                let leaf = leaf_of_slot(*slot);
                let leaf_start = leaf * ALLOCATION_RECORD_TREE_LEAF_SLOTS;
                assert!(
                    state.records.range(leaf_start..leaf_start + ALLOCATION_RECORD_TREE_LEAF_SLOTS).next().is_some(),
                    "一片叶的记录不会删到空：被删的只有新落点罩住的已回收记录，新落点那条就在旁边"
                );
            }
        }
    }
    for (slot, span_slots) in &plan.released_placements {
        state.deferred.insert((plan.txg, *slot), *span_slots);
    }
    for key in &plan.resurrected_deferred {
        state.deferred.remove(key).expect("复活的落点还在 defer 里：目标那一版在候选窗口里");
    }
    state.units.resize(usize_of(plan.fresh.next_unit), None);
    for (unit, information) in &plan.new_units {
        state.units[usize_of(unit.0)] = Some(*information);
    }
    state.version = Some(plan.version_after);
    state.allocation_nodes = plan.allocation_nodes_after;
    for leaf in &state.leaves_with_records {
        assert!(state.allocation_nodes.contains_key(&AllocationNodePosition { level: 0, device: 0, index_in_device: *leaf }), "有记录的叶都有节点");
    }
    state.accounting = Some(plan.accounting_after);
    state.tree_table = Some(plan.tree_table_after);
    state.mapping = plan.mapping_after;
    state.quarantine_tree = plan.quarantine_tree_after;
    state.quarantine_tree_registered |= plan.quarantine_tree_registered_by_this_publish;
    if let Some(unit) = plan.newly_quarantined {
        let information = state.unit(unit);
        state.quarantined.insert(unit, QuarantinedUnit { slot: information.slot, span_slots: information.span_slots });
    }
    for unit in plan.revived.iter().chain(plan.released_quarantined.iter()) {
        state.quarantined.remove(unit).expect("复活或释放的是一个隔离着的单元");
    }
    state.txg = plan.txg;
    let floor = oldest_candidate_txg(state.txg, state.candidate_window);
    while let Some((&(release_generation, slot), &span_slots)) = state.deferred.first_key_value() {
        if release_generation > floor {
            break;
        }
        state.deferred.remove(&(release_generation, slot));
        for covered in slot..slot + span_slots {
            state.set_occupied(covered, false);
        }
    }
    // 两个下界提示只往上收紧：此刻往下都是占着的槽、都不是全空段。
    state.lowest_possibly_free_slot = state.first_unoccupied_slot_from(state.lowest_possibly_free_slot);
    while state.lowest_possibly_empty_segment < state.used_slots_per_segment.len() && state.used_slots_per_segment[state.lowest_possibly_empty_segment] > 0 {
        state.lowest_possibly_empty_segment += 1;
    }
    if state.keeps_versions_for_rollback {
        let kept = state.version.clone().expect("刚落了这一版");
        state.kept_versions.insert(state.txg, kept);
        state.kept_versions.retain(|txg, _| *txg >= floor);
    }
}

/// 走一次发布并落进池状态；交回这次的写与取落点回落过几次。
pub(super) fn publish(state: &mut PoolState, request: &PublishRequest, arm: Arm) -> (PublishWrites, u64) {
    let plan = plan_publish(state, request, arm);
    let writes = plan.writes(0);
    let placements_that_fell_back = plan.placements_that_fell_back;
    apply_publish(state, plan);
    (writes, placements_that_fell_back)
}
