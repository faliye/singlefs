//! 一次发布要写什么的逻辑规划：extent 下层节点、记账键、inode 记录写与删、文件重写、删除批与墓碑追加。

use super::*;

/// extent 树下段层级 L 上一个节点罩几个单元：144 × 147^L（`extent_tree.rs:84-90`）。
fn extent_lower_span_in_data_units(level: u8) -> u64 {
    (0..level).fold(EXTENT_TREE_LOWER_LEAF_DATA_UNITS, |span, _| {
        span.saturating_mul(EXTENT_TREE_INTERNAL_FANOUT)
    })
}

/// 下段的全部节点，先叶后根、同层按位置升序；只有一个单元的文件不建下段（`extent_tree.rs:210-226`）。
fn extent_lower_nodes_of_a_file(data_units: u64) -> Vec<(u8, u64)> {
    if data_units <= 1 {
        return Vec::new();
    }
    let root_level = (0..=u8::MAX)
        .find(|level| extent_lower_span_in_data_units(*level) >= data_units)
        .expect("罩得住");
    let mut nodes = Vec::new();
    for level in 0..=root_level {
        let nodes_in_this_level = data_units.div_ceil(extent_lower_span_in_data_units(level));
        for index in 0..nodes_in_this_level {
            nodes.push((level, index));
        }
    }
    nodes
}

fn extent_lower_node_last_data_unit(position: (u8, u64)) -> u64 {
    let span = extent_lower_span_in_data_units(position.0);
    position.1.saturating_mul(span).saturating_add(span - 1)
}

/// 记账 15 行的 key (统计量, 树, 盘, 代)：池级 3 行、每盘 6 行，代 = 这次的 txg（S3；统计量标签模型自编，C6）。
fn accounting_keys_of(txg: u64) -> BTreeSet<KeyFields> {
    let mut keys = BTreeSet::new();
    for row in 0..ACCOUNTING_POOL_WIDE_ROWS {
        keys.insert([1 + row, TREE_IDENTIFIER_INODE, STATISTIC_WITHOUT_DEVICE, txg, 0]);
    }
    for device in 0..u64::from(DEVICE_COUNT) {
        for row in 0..ACCOUNTING_PER_DEVICE_ROWS {
            keys.insert([1 + ACCOUNTING_POOL_WIDE_ROWS + row, 0, device, txg, 0]);
        }
    }
    keys
}

fn release_if_carried(planned: PlannedUnit, released_units: &mut Vec<UnitIdentifier>) {
    match planned {
        PlannedUnit::Carried(unit) => released_units.push(unit),
        PlannedUnit::Rewritten => {}
    }
}

fn mark_inode_root_rewritten(plan: &mut LogicalPlan) {
    release_if_carried(plan.inode_root, &mut plan.released_units);
    plan.inode_root = PlannedUnit::Rewritten;
}

fn mark_inode_container_rewritten(plan: &mut LogicalPlan, position: usize) {
    let origin = plan.inode_containers[position].origin;
    release_if_carried(origin, &mut plan.released_units);
    plan.inode_containers[position].origin = PlannedUnit::Rewritten;
}

/// 往 inode 树写记录（S5，`inode_tree.rs:181-272`）：已有的就地改，新号追加到最右有空位的那片，满了末尾新开一片。
fn plan_inode_record_writes(plan: &mut LogicalPlan, records_to_write: &[InodeRecordSummary]) {
    let mut rewritten_positions: BTreeSet<usize> = BTreeSet::new();
    for record in records_to_write {
        let holding = plan
            .inode_containers
            .iter()
            .position(|container| container.records.iter().any(|existing| existing.inode == record.inode));
        match holding {
            Some(position) => {
                let container = &mut plan.inode_containers[position];
                let at = container
                    .records
                    .iter()
                    .position(|existing| existing.inode == record.inode)
                    .expect("刚找到这条记录所在的容器");
                container.records[at] = *record;
                rewritten_positions.insert(position);
            }
            None => {
                if let Some(largest) = plan
                    .inode_containers
                    .last()
                    .and_then(|container| container.records.last())
                    .map(|existing| existing.inode)
                {
                    assert!(record.inode > largest, "新号大于树里每一个 key：插入落在最右的叶");
                }
                let rightmost_has_room = plan
                    .inode_containers
                    .last()
                    .is_some_and(|container| length_as_u64(container.records.len()) < INODE_LEAF_RECORDS);
                if rightmost_has_room {
                    let rightmost = plan.inode_containers.len() - 1;
                    plan.inode_containers[rightmost].records.push(*record);
                    rewritten_positions.insert(rightmost);
                } else {
                    plan.inode_containers.push(PlannedInodeContainer {
                        container_number: record.inode,
                        container_birth: plan.txg,
                        records: vec![*record],
                        origin: PlannedUnit::Rewritten,
                    });
                    rewritten_positions.insert(plan.inode_containers.len() - 1);
                }
            }
        }
    }
    assert!(
        length_as_u64(plan.inode_containers.len()) <= ROOT_NODE_CONTAINER_CAPACITY,
        "inode 树的叶容器不多于一个根装得下的 135 片"
    );
    for position in &rewritten_positions {
        mark_inode_container_rewritten(plan, *position);
    }
    if !rewritten_positions.is_empty() {
        mark_inode_root_rewritten(plan);
    }
}

/// 删一条 inode 记录（S5，`inode_tree.rs:286-317`）：删空的那片不落盘、右边各片左移、全部重写。
fn plan_inode_record_removal(plan: &mut LogicalPlan, inode: u64) {
    let position = plan
        .inode_containers
        .iter()
        .position(|container| container.records.iter().any(|existing| existing.inode == inode))
        .expect("删的记录在树里");
    plan.inode_containers[position]
        .records
        .retain(|existing| existing.inode != inode);
    if plan.inode_containers[position].records.is_empty() {
        let removed = plan.inode_containers.remove(position);
        release_if_carried(removed.origin, &mut plan.released_units);
        for later in position..plan.inode_containers.len() {
            mark_inode_container_rewritten(plan, later);
        }
    } else {
        mark_inode_container_rewritten(plan, position);
    }
    mark_inode_root_rewritten(plan);
}

/// 写文件内容（覆盖写、顺序写、新池新建文件）：整份换成 `data_units` 个新单元，上一版的单元一并换下（M11）；extent 下段整段重写、
/// 上段根重写（`transaction.rs:4723-4805`）。
fn plan_file_rewrite(plan: &mut LogicalPlan, previous: &Version, data_units: u64) {
    plan.released_units.extend(previous.file_units.iter().copied());
    plan.data_units = vec![PlannedUnit::Rewritten; u64_as_index(data_units)];
    plan.released_units.extend(previous.extent_lower.values().copied());
    plan.extent_lower = extent_lower_nodes_of_a_file(data_units)
        .into_iter()
        .map(|position| (position, PlannedUnit::Rewritten))
        .collect();
    if let Some(root) = previous.extent_root {
        plan.released_units.push(root);
    }
    plan.extent_root = PlannedUnit::Rewritten;
    plan.highest_transaction_after = plan.transaction_base + data_units - 1;
}

/// 删除的一批（`unlink.rs:166-235`）：留下前 `data_units_kept` 个单元，整落在留下段里的下段节点照抄，别的重写，上段根重写。
fn plan_deletion_batch(plan: &mut LogicalPlan, previous: &Version, data_units_kept: u64) {
    let kept = u64_as_index(data_units_kept);
    assert!(kept >= 1 && kept < previous.file_units.len(), "一批留下至少一个、至少截掉一个");
    plan.released_units.extend(previous.file_units[kept..].iter().copied());
    plan.data_units = previous.file_units[..kept]
        .iter()
        .map(|unit| PlannedUnit::Carried(*unit))
        .collect();
    let mut carried_positions = BTreeSet::new();
    let mut lower = Vec::new();
    for position in extent_lower_nodes_of_a_file(data_units_kept) {
        if extent_lower_node_last_data_unit(position) < data_units_kept {
            let unit = *previous
                .extent_lower
                .get(&position)
                .expect("罩的单元全在留下那一段里的节点，上一版同一个位置上必有");
            lower.push((position, PlannedUnit::Carried(unit)));
            carried_positions.insert(position);
        } else {
            lower.push((position, PlannedUnit::Rewritten));
        }
    }
    for (position, unit) in &previous.extent_lower {
        if !carried_positions.contains(position) {
            plan.released_units.push(*unit);
        }
    }
    plan.extent_lower = lower;
    release_if_carried(plan.extent_root, &mut plan.released_units);
    plan.extent_root = PlannedUnit::Rewritten;
}

/// 删 inode 1 的最后一批（`unlink.rs:243-264`）：剩下的数据单元全换下，下段整段换下，上段根重写成零条目。
fn plan_final_batch_of_the_first_file(plan: &mut LogicalPlan, previous: &Version) {
    plan.released_units.extend(previous.file_units.iter().copied());
    plan.data_units = Vec::new();
    plan.released_units.extend(previous.extent_lower.values().copied());
    plan.extent_lower = Vec::new();
    release_if_carried(plan.extent_root, &mut plan.released_units);
    plan.extent_root = PlannedUnit::Rewritten;
}

/// 意图那次往墓碑树最右的容器追加一条（S7，`container_tree.rs:423-473`）：满 815 开新容器，号 = 最大号 + 1；根重写。
fn plan_tombstone_append(plan: &mut LogicalPlan, record: TombstoneRecordSummary) {
    let rightmost_has_room = plan
        .tombstone_containers
        .last()
        .is_some_and(|container| length_as_u64(container.records.len()) < TOMBSTONE_CONTAINER_RECORDS);
    if rightmost_has_room {
        let rightmost = plan.tombstone_containers.len() - 1;
        let origin = plan.tombstone_containers[rightmost].origin;
        release_if_carried(origin, &mut plan.released_units);
        plan.tombstone_containers[rightmost].records.push(record);
        plan.tombstone_containers[rightmost].origin = PlannedUnit::Rewritten;
    } else {
        let container_number = plan
            .tombstone_containers
            .last()
            .map_or(1, |rightmost| rightmost.container_number + 1);
        plan.tombstone_containers.push(PlannedTombstoneContainer {
            container_number,
            container_birth: plan.txg,
            records: vec![record],
            origin: PlannedUnit::Rewritten,
        });
    }
    assert!(
        length_as_u64(plan.tombstone_containers.len()) <= ROOT_NODE_CONTAINER_CAPACITY,
        "墓碑树的容器不多于一个根装得下的：第一段的历史里根不长高"
    );
    if let Some(root) = plan.tombstone_root {
        release_if_carried(root, &mut plan.released_units);
    }
    plan.tombstone_root = Some(PlannedUnit::Rewritten);
    plan.tombstones_written.push(record);
}

/// 这次发布不依赖落点的那一半（M5、S5–S13）。
pub(super) fn plan_logical(
    previous: &Version,
    operation: PublishOperation,
    txg: u64,
    write_path: WritePath,
    file_object_birth: u64,
) -> LogicalPlan {
    let instance = if txg == NEW_POOL_FILE_CREATION_TXG {
        MAKE_FILESYSTEM_PROCESS_INSTANCE
    } else {
        MOUNT_INSTANCE
    };
    assert!(previous.instance <= instance, "实例代号只增：mkfs 那个进程取 1，可写挂载取 2");
    let transaction_base = if operation == PublishOperation::RowPublish {
        1
    } else {
        previous.highest_transaction + 1
    };
    let mut plan = LogicalPlan {
        txg,
        instance,
        transaction_base,
        highest_transaction_after: transaction_base,
        next_inode_after: previous.next_inode,
        file_object_birth,
        data_units: previous.file_units.iter().map(|unit| PlannedUnit::Carried(*unit)).collect(),
        instance_table_rewritten: false,
        extent_lower: previous
            .extent_lower
            .iter()
            .map(|(position, unit)| (*position, PlannedUnit::Carried(*unit)))
            .collect(),
        extent_root: previous.extent_root.map_or(PlannedUnit::Rewritten, PlannedUnit::Carried),
        inode_containers: previous
            .inode_containers
            .iter()
            .map(|container| PlannedInodeContainer {
                container_number: container.container_number,
                container_birth: container.container_birth,
                records: container.records.clone(),
                origin: PlannedUnit::Carried(container.unit),
            })
            .collect(),
        inode_root: previous.inode_root.map_or(PlannedUnit::Rewritten, PlannedUnit::Carried),
        deleted_queue: previous.deleted_queue.clone(),
        deleted_root: previous.deleted_root.map(PlannedUnit::Carried),
        tombstone_containers: previous
            .tombstone_containers
            .iter()
            .map(|container| PlannedTombstoneContainer {
                container_number: container.container_number,
                container_birth: container.container_birth,
                records: container.records.clone(),
                origin: PlannedUnit::Carried(container.unit),
            })
            .collect(),
        tombstone_root: previous.tombstone_root.map(PlannedUnit::Carried),
        tombstones_written: Vec::new(),
        accounting_keys: accounting_keys_of(txg),
        released_units: Vec::new(),
    };
    let first_file_record = || {
        previous
            .live_record_of(FIRST_FILE_INODE)
            .expect("写文件内容的发布接在 inode 1 活着的一版之后")
    };
    match operation {
        PublishOperation::FirstFile => {
            plan_file_rewrite(&mut plan, previous, 1);
            plan_inode_record_writes(
                &mut plan,
                &[InodeRecordSummary {
                    inode: FIRST_FILE_INODE,
                    object_birth: txg,
                    nlink: 1,
                }],
            );
            plan.next_inode_after = FIRST_FILE_INODE + 1;
        }
        PublishOperation::RowPublish => {
            plan.instance_table_rewritten = true;
            plan.released_units.push(previous.instance_table);
        }
        PublishOperation::WarmUp => {}
        PublishOperation::Overwrite => {
            let record = first_file_record();
            plan_file_rewrite(&mut plan, previous, 1);
            plan_inode_record_writes(&mut plan, &[record]);
        }
        PublishOperation::SequentialWrite { data_units } => {
            let record = first_file_record();
            plan_file_rewrite(&mut plan, previous, data_units);
            plan_inode_record_writes(&mut plan, &[record]);
        }
        PublishOperation::NewInodes { count } => {
            let records: Vec<InodeRecordSummary> = (0..count)
                .map(|offset| InodeRecordSummary {
                    inode: previous.next_inode + offset,
                    object_birth: txg,
                    nlink: 1,
                })
                .collect();
            plan_inode_record_writes(&mut plan, &records);
            plan.next_inode_after = previous.next_inode + count;
        }
        PublishOperation::UnlinkIntent { inode } => {
            let record = previous.live_record_of(inode).expect("意图删的 inode 是一条活记录");
            assert!(record.nlink > 0, "意图删的 inode 还活着");
            plan_inode_record_writes(
                &mut plan,
                &[InodeRecordSummary {
                    inode,
                    object_birth: record.object_birth,
                    nlink: 0,
                }],
            );
            assert!(plan.deleted_queue.is_empty(), "发意图之前队列已走空");
            plan.deleted_queue.push(inode);
            if let Some(root) = plan.deleted_root {
                release_if_carried(root, &mut plan.released_units);
            }
            plan.deleted_root = Some(PlannedUnit::Rewritten);
            if write_path == WritePath::WithTombstones {
                plan_tombstone_append(
                    &mut plan,
                    TombstoneRecordSummary {
                        object: inode,
                        object_birth: record.object_birth,
                        death_txg: txg,
                    },
                );
            }
        }
        PublishOperation::UnlinkBatch { data_units_kept } => {
            let record = previous.live_record_of(FIRST_FILE_INODE).expect("分批删的是 inode 1");
            plan_deletion_batch(&mut plan, previous, data_units_kept);
            plan_inode_record_writes(&mut plan, &[record]);
        }
        PublishOperation::UnlinkFinal { inode } => {
            plan_inode_record_removal(&mut plan, inode);
            if inode == FIRST_FILE_INODE {
                plan_final_batch_of_the_first_file(&mut plan, previous);
            }
            plan.deleted_queue.retain(|queued| *queued != inode);
            assert!(plan.deleted_queue.is_empty(), "任何一版至多一条 key");
            if let Some(root) = plan.deleted_root {
                release_if_carried(root, &mut plan.released_units);
            }
            plan.deleted_root = None;
        }
    }
    plan
}

#[cfg(test)]
#[path = "logical_planning_tests.rs"]
mod tests;
