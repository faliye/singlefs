//! 正推腿探针（T5）：
//! ① 被快照钉住的单元搬走、快照照读之后，再删快照：放掉的是留存的映射条目现在指着的目的地，不是源落点；池级 checker 全程没有「违例」。
//! ② 整理的一批多重写的映射节点有没有越过 ckpt_cost：中央映射树压成叶 3 条、内部节点 3 个孩子（只供测试的容量），第一个文件版本的映射树
//!    至少三片叶；一批搬四个落在不同叶里的单元，数这一批写出的固定点单元（保留池那一类）与同一版上一次空发布的 ckpt_cost 比。
mod common;
mod common_tree_split;

use common::{build_pool, file_content, parameters, FIXED_WRITE_TIME_SECONDS};
use common_tree_split::{capacities, TreeSplitPool};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DataUnitIndexInFile, InstanceGeneration, TreeIdentifier};
use singlefs_core::admission::{
    checkpoint_cost_of_the_version_to_build_on_with_node_capacities, space_budget_of_role,
    SpaceBudgetOfARole,
};
use singlefs_core::allocator::Placement;
use singlefs_core::compaction_plan::PlannedRelocation;
use singlefs_core::records::TREE_KIND_SNAPSHOT_INODE_ROOT;
use singlefs_core::recovery::{read_the_first_file_of_a_snapshot, SnapshotFileReading};
use singlefs_core::transaction::{
    central_mapping_locations_in_the_version, publish_create_snapshot, publish_overwrite,
    publish_relocation_batch, publish_snapshot_destruction_intent,
    publish_snapshot_destruction_step, FirstFile, MappingLookup, PoolWriter, TransactionOutput,
    TransactionUnit,
};
use singlefs_core::unlink::UnlinkBatchSize;

const INSTANCE: InstanceGeneration = InstanceGeneration(1);

fn planned_relocation_of(version: &TransactionOutput, identity: TransactionUnit) -> PlannedRelocation {
    let (_, mapping_key) = version
        .mapped_units
        .iter()
        .find(|(role, _)| *role == identity)
        .expect("这个角色进映射");
    let MappingLookup::Found(locations) = central_mapping_locations_in_the_version(version, mapping_key) else {
        panic!("{identity:?} 在映射里查得到");
    };
    PlannedRelocation {
        mapping_key: mapping_key.clone(),
        source: Placement {
            slot: locations[0].slot,
            span: identity.span_slots(),
        },
    }
}

fn violated_invariants(image: &singlefs_harness::memory_pool::MemoryPool) -> (Vec<&'static str>, usize, usize, usize) {
    let mut violated = Vec::new();
    let (mut holds, mut not_applicable, mut partial) = (0, 0, 0);
    for (invariant, verdict) in check_pool_image(image) {
        match verdict {
            InvariantVerdict::Violated(_) => violated.push(invariant),
            InvariantVerdict::Holds => holds += 1,
            InvariantVerdict::NotApplicable(_) => not_applicable += 1,
            InvariantVerdict::PartiallyJudged(_) => partial += 1,
        }
    }
    (violated, holds, not_applicable, partial)
}

#[test]
fn probe_forward_destroying_a_snapshot_after_its_pinned_unit_was_relocated_releases_the_destination() {
    let mut pool = build_pool("probe-forward-relocate-then-destroy");
    let publish_parameters = parameters();
    let created = {
        let previous = pool.output.clone();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_create_snapshot(&mut writer, &mut pool.allocator, &previous, INSTANCE).expect("建快照")
    };
    pool.output = created.clone();
    let snapshot: TreeIdentifier = created
        .tree_table_entries
        .iter()
        .find(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
        .expect("一个快照")
        .tree;
    let content: Vec<u8> = (0..2600_u32).map(|index| u8::try_from((index * 7 + 3) % 241).unwrap()).collect();
    let overwritten = {
        let previous = pool.output.clone();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut pool.allocator,
            &previous,
            FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 },
            INSTANCE,
        )
        .expect("覆盖写")
    };
    pool.output = overwritten.clone();
    let (kept_key, kept_locations) = overwritten
        .mapping_entries_kept_for_the_snapshots
        .iter()
        .find(|(key, _)| key.first() == Some(&singlefs_core::unit::UNIT_CLASS_DATA))
        .cloned()
        .expect("旧数据单元的映射条目留存着");
    let planned = PlannedRelocation {
        mapping_key: kept_key.clone(),
        source: Placement { slot: kept_locations[0].slot, span: 2 },
    };
    let batch = {
        let previous = pool.output.clone();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_relocation_batch(&mut writer, &mut pool.allocator, &previous, std::slice::from_ref(&planned), INSTANCE)
            .expect("搬")
    };
    pool.output = batch.clone();
    let (_, kept_after) = batch
        .mapping_entries_kept_for_the_snapshots
        .iter()
        .find(|(key, _)| *key == kept_key)
        .cloned()
        .expect("留存条目还在");
    {
        let device_map = &pool.allocator.devices[0];
        let unit_area = singlefs_core::compaction_plan::UnitAreaOfOneDevice {
            device: device_map.device,
            start: device_map.unit_area_start(),
            slots: device_map.unit_area_slots(),
        };
        let by_records = singlefs_core::compaction_plan::empty_cluster_segments_by_the_records(&batch.allocation_records, unit_area);
        let row_11 = batch
            .accounting_entries
            .iter()
            .find(|entry| entry.statistic == singlefs_core::records::STATISTIC_EMPTY_CLUSTER_SEGMENTS && entry.device == singlefs_core::address::DeviceIdentity(0) && entry.generation == batch.root.checkpoint_txg)
            .expect("第 11 项")
            .value;
        println!(
            "name=probe_forward_empty_segments_after_a_relocation_batch allocator_incremental={} full_scan_of_the_records={by_records} accounting_row_11={row_11}",
            device_map.empty_segments()
        );
        assert_eq!(device_map.empty_segments(), by_records);
        assert_eq!(row_11, by_records);
    }
    let (violated_mid, holds_mid, na_mid, partial_mid) = violated_invariants(&pool.memory_pool());
    println!("name=probe_forward_after_relocation violated={violated_mid:?} holds={holds_mid} not_applicable={na_mid} partially={partial_mid}");
    assert!(violated_mid.is_empty(), "搬完之后池级 checker 没有违例：{violated_mid:?}");
    // 删快照：意图 + 批到最后一批。
    let mut current = batch;
    let intent = {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_snapshot_destruction_intent(&mut writer, &mut pool.allocator, &current, snapshot, INSTANCE)
            .expect("删快照的意图")
    };
    current = intent;
    let mut released_by_the_destruction = Vec::new();
    for _ in 0..8 {
        if current.snapshots_being_destroyed.is_empty() {
            break;
        }
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        let next = publish_snapshot_destruction_step(
            &mut writer,
            &mut pool.allocator,
            &current,
            snapshot,
            UnlinkBatchSize::OneExtentLowerLeaf,
            INSTANCE,
        )
        .expect("删快照的批");
        released_by_the_destruction.extend(next.released.clone());
        current = next;
    }
    assert!(current.snapshots_being_destroyed.is_empty() && current.snapshot_entries().is_empty(), "快照删光");
    let destination_slot = kept_after[0].slot;
    let still_unreleased_at_destination = current
        .allocation_records
        .iter()
        .filter(|record| record.slot == destination_slot && !record.is_released)
        .count();
    let released_destination_records = current
        .allocation_records
        .iter()
        .filter(|record| record.slot == destination_slot && record.is_released)
        .count();
    let source_records_released = current
        .allocation_records
        .iter()
        .filter(|record| record.slot == kept_locations[0].slot && record.is_released)
        .count();
    println!(
        "name=probe_forward_after_destroy destination_slot={} source_slot={} destination_records_released={} destination_records_unreleased={} source_records_released={} released_placements={:?}",
        destination_slot.0, kept_locations[0].slot.0, released_destination_records, still_unreleased_at_destination, source_records_released,
        released_by_the_destruction.iter().map(|placement| placement.slot.0).collect::<Vec<_>>()
    );
    assert_eq!(still_unreleased_at_destination, 0, "目的地的记录放掉了");
    assert!(
        released_by_the_destruction.iter().any(|placement| placement.slot == destination_slot),
        "删快照放的是目的地"
    );
    assert!(
        !released_by_the_destruction.iter().any(|placement| placement.slot == kept_locations[0].slot),
        "没有再放一次源落点"
    );
    let (violated, holds, not_applicable, partial) = violated_invariants(&pool.memory_pool());
    println!("name=probe_forward_after_destroy_checker violated={violated:?} holds={holds} not_applicable={not_applicable} partially={partial}");
    assert!(violated.is_empty(), "删完快照之后池级 checker 没有违例：{violated:?}");
    let _ = (file_content(), DataUnitIndexInFile::FIRST, read_the_first_file_of_a_snapshot as fn(_, _, _) -> _);
    let _ = SnapshotFileReading::NoFile;
}

#[test]
fn probe_forward_a_batch_moving_units_of_different_mapping_leaves_writes_more_reserve_class_units_than_the_checkpoint_cost_of_its_version() {
    let tiny_mapping = capacities((477, 150), (3, 3));
    let mut pool = TreeSplitPool::with_the_first_file_version_under(tiny_mapping);
    let previous = pool.output.clone();
    let leaves_before = common_tree_split::leaf_count(&previous.central_mapping_tree);
    let ckpt_cost_before = checkpoint_cost_of_the_version_to_build_on_with_node_capacities(Some(&previous), &pool.allocator, tiny_mapping).0;
    let inode_leaf_container = previous
        .mapped_units
        .iter()
        .map(|(role, _)| *role)
        .find(|role| matches!(role, TransactionUnit::InodeLeafContainer(_)))
        .expect("一片 inode 叶容器");
    let identities = [
        TransactionUnit::Data(DataUnitIndexInFile::FIRST),
        TransactionUnit::ExtentRoot,
        inode_leaf_container,
        TransactionUnit::InodeRoot,
    ];
    let planned: Vec<PlannedRelocation> = identities.iter().map(|identity| planned_relocation_of(&previous, *identity)).collect();
    let publish_parameters = parameters();
    let batch = {
        let mut writer = PoolWriter::new(&publish_parameters, pool.devices.as_mut_slice());
        writer.set_code_two_tree_node_capacities(tiny_mapping);
        publish_relocation_batch(&mut writer, &mut pool.allocator, &previous, &planned, previous.root.instance)
            .expect("一批四个")
    };
    let mut reserve_class = 0usize;
    let mut mapping_nodes = 0usize;
    let mut ordinary = 0usize;
    for role in &batch.rewritten {
        match space_budget_of_role(*role) {
            SpaceBudgetOfARole::CheckpointReservePool => reserve_class += 1,
            SpaceBudgetOfARole::OrdinaryAllocation => ordinary += 1,
            SpaceBudgetOfARole::InstanceSwitchReserve => {}
        }
        if matches!(role, TransactionUnit::MappingTree | TransactionUnit::MappingTreeNodeBelowTheRoot(_)) {
            mapping_nodes += 1;
        }
    }
    let leaves_after = common_tree_split::leaf_count(&batch.central_mapping_tree);
    println!(
        "name=probe_forward_batch_vs_ckpt_cost mapping_leaves_before={leaves_before} mapping_leaves_after={leaves_after} units_moved={} ckpt_cost_of_the_version_before_the_batch={ckpt_cost_before} reserve_class_units_written_by_the_batch={reserve_class} of_which_mapping_nodes={mapping_nodes} ordinary_allocation_units_written={ordinary}",
        planned.len()
    );
    // 对照：同一版上一次空发布写出多少保留池一类（实测，不是上界）。
    let mut empty_pool = TreeSplitPool::with_the_first_file_version_under(tiny_mapping);
    empty_pool.empty_publish(tiny_mapping);
    let empty_reserve_class = empty_pool
        .output
        .rewritten
        .iter()
        .filter(|role| space_budget_of_role(**role) == SpaceBudgetOfARole::CheckpointReservePool)
        .count();
    let empty_mapping_nodes = empty_pool
        .output
        .rewritten
        .iter()
        .filter(|role| matches!(role, TransactionUnit::MappingTree | TransactionUnit::MappingTreeNodeBelowTheRoot(_)))
        .count();
    println!("name=probe_forward_empty_publish_for_comparison reserve_class_units_written={empty_reserve_class} of_which_mapping_nodes={empty_mapping_nodes}");
}
