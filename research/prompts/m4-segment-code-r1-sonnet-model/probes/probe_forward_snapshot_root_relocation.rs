//! 正推腿探针（T11、T5）：整理搬的是某个快照抄着的那个 inode 树根单元（建快照之后活头换了根，旧根只被快照引用、映射条目留存）时，
//! 树表里码 14 条目的根指针字节变不变、「非空」那一判（`mount::user_visible_trees_changed`，照字节比）算不算非空；
//! 模型的 `user_visible_root_units` 把快照抄的根单元也算进搬根的名单里（搬了就是新状态）。
mod common;

use common::{build_pool, parameters, FIXED_WRITE_TIME_SECONDS};
use singlefs_core::address::InstanceGeneration;
use singlefs_core::allocator::Placement;
use singlefs_core::compaction_plan::PlannedRelocation;
use singlefs_core::mount::user_visible_trees_changed;
use singlefs_core::records::{mapping_key_for_node, TREE_KIND_SNAPSHOT_EXTENT_ROOT, TREE_KIND_SNAPSHOT_INODE_ROOT};
use singlefs_core::recovery::user_visible_tree_root_pointers;
use singlefs_core::transaction::{publish_create_snapshot, publish_overwrite, publish_relocation_batch, FirstFile, PoolWriter};
use singlefs_core::unit::UNIT_CLASS_INDEX_NODE;

const INSTANCE: InstanceGeneration = InstanceGeneration(1);

#[test]
fn probe_forward_relocating_a_root_unit_only_a_snapshot_references() {
    let mut pool = build_pool("probe-forward-snapshot-root-relocation");
    let publish_parameters = parameters();
    let created = {
        let previous = pool.output.clone();
        let devices = pool.devices.as_mut().unwrap();
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_create_snapshot(&mut writer, &mut pool.allocator, &previous, INSTANCE).unwrap()
    };
    pool.output = created.clone();
    let overwritten = {
        let previous = pool.output.clone();
        let devices = pool.devices.as_mut().unwrap();
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut pool.allocator,
            &previous,
            FirstFile { content: &[9u8; 1500], write_time_seconds: FIXED_WRITE_TIME_SECONDS + 5 },
            INSTANCE,
        )
        .unwrap()
    };
    pool.output = overwritten.clone();
    for (label, kind) in [("snapshot_inode_root", TREE_KIND_SNAPSHOT_INODE_ROOT), ("snapshot_extent_root", TREE_KIND_SNAPSHOT_EXTENT_ROOT)] {
        let entry = *overwritten.tree_table_entries.iter().find(|entry| entry.kind == kind).expect("快照条目");
        let key = mapping_key_for_node(UNIT_CLASS_INDEX_NODE, entry.root);
        let kept = overwritten
            .mapping_entries_kept_for_the_snapshots
            .iter()
            .find(|(kept_key, _)| *kept_key == key)
            .cloned();
        let Some((kept_key, kept_locations)) = kept else {
            println!("name=probe_forward_snapshot_root_relocation unit={label} kept_mapping_entry=none");
            continue;
        };
        let planned = PlannedRelocation {
            mapping_key: kept_key,
            source: Placement { slot: kept_locations[0].slot, span: 1 },
        };
        let before_pointers = user_visible_tree_root_pointers(&pool.memory_pool(), &pool.output.root).unwrap();
        let batch = {
            let previous = pool.output.clone();
            let devices = pool.devices.as_mut().unwrap();
            let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
            publish_relocation_batch(&mut writer, &mut pool.allocator, &previous, std::slice::from_ref(&planned), INSTANCE).unwrap()
        };
        let after_pointers = user_visible_tree_root_pointers(&pool.memory_pool(), &batch.root).unwrap();
        let entry_after = *batch.tree_table_entries.iter().find(|candidate| candidate.kind == kind).unwrap();
        println!(
            "name=probe_forward_snapshot_root_relocation unit={label} source_slot={} kept_entry_destination_slot={} tree_table_entry_root_hint_slot_before={} after={} entry_root_pointer_changed={} user_visible_trees_changed={}",
            kept_locations[0].slot.0,
            batch.mapping_entries_kept_for_the_snapshots.iter().find(|(k, _)| *k == planned.mapping_key).map(|(_, l)| l[0].slot.0).unwrap_or(0),
            entry.root.locations[0].slot.0,
            entry_after.root.locations[0].slot.0,
            entry.root != entry_after.root,
            user_visible_trees_changed(&after_pointers, &before_pointers)
        );
        pool.output = batch;
    }
}
