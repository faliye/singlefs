//! PROBE (forward leg, not in repo): crash prefixes of an unlink publish chain under knob 1 in all three container trees,
//! then writable mount (continuation) and the pool checker with the knobs: no red allowed.
mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use common::{crash_state_devices, geometry, memory_pool_of_sparse_devices, parameters};
use common_deletion_in_progress::{
    pool_with_new_inodes, INSTANCE_OF_THE_PUBLISHES, WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
};
use singlefs_core::address::InodeNumber;
use singlefs_core::container_tree::ContainerTreeCapacities;
use singlefs_core::transaction::{
    publish_unlink_intent, publish_unlink_steps_until_the_queue_is_empty, PoolWriter,
};
use common::BuiltPool;

fn capacities() -> ContainerTreeCapacities {
    let s = switches();
    ContainerTreeCapacities {
        tombstone_container: s.tombstone_container_capacity,
        lineage_container: s.lineage_container_capacity,
        retirement_container: s.retirement_container_capacity,
        node_entries: s.container_tree_node_entries,
    }
}

fn unlink_to_the_end(pool: &mut BuiltPool, inode: u64) {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("open");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    writer.set_container_tree_capacities(capacities());
    let intent = publish_unlink_intent(
        &mut writer,
        &mut pool.allocator,
        &previous,
        InodeNumber(inode),
        WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
        INSTANCE_OF_THE_PUBLISHES,
    )
    .unwrap();
    let mut current = intent.clone();
    publish_unlink_steps_until_the_queue_is_empty(
        &mut writer,
        &mut pool.allocator,
        &mut current,
        UnlinkBatchSize::OneExtentLowerLeaf,
        INSTANCE_OF_THE_PUBLISHES,
    )
    .unwrap();
    pool.output = current;
}
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::{
    check_pool_image_given_the_container_capacity_knobs, ContainerCapacityKnobs,
    LineageContainerCapacityKnob, RetirementContainerCapacityKnob, TombstoneContainerCapacityKnob,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::container_tree::{
    ContainerTreeNodeEntries, ContainerTreeNodeEntriesCappedForTests,
};
use singlefs_core::lineage_tree::{
    LineageContainerCapacity, LineageRecordsPerContainerCappedForTests,
};
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_deletion_switches, ShadowLedger,
};
use singlefs_core::tombstone_tree::{
    TombstoneContainerCapacity, TombstoneRecordsPerContainerCappedForTests,
};
use singlefs_core::unlink::{DeletionSwitches, UnlinkBatchSize};
use singlefs_harness::memory_pool::writes_and_segments_with_stream_indexes;
use singlefs_harness::SharedStream;

fn switches() -> DeletionSwitches {
    DeletionSwitches {
        batch_size: UnlinkBatchSize::OneExtentLowerLeaf,
        tombstone_container_capacity: TombstoneContainerCapacity::CappedForTests(
            TombstoneRecordsPerContainerCappedForTests::new(1).unwrap(),
        ),
        lineage_container_capacity: LineageContainerCapacity::CappedForTests(
            LineageRecordsPerContainerCappedForTests::new(1).unwrap(),
        ),
        retirement_container_capacity:
            singlefs_core::retirement_tree::RetirementContainerCapacity::CappedForTests(
                singlefs_core::retirement_tree::RetirementRecordsPerContainerCappedForTests::new(1)
                    .unwrap(),
            ),
        container_tree_node_entries: ContainerTreeNodeEntries::CappedForTests(
            ContainerTreeNodeEntriesCappedForTests::new(2).unwrap(),
        ),
    }
}

fn knobs() -> ContainerCapacityKnobs {
    ContainerCapacityKnobs {
        tombstone: Some(TombstoneContainerCapacityKnob::new(1).unwrap()),
        lineage: Some(LineageContainerCapacityKnob::new(1).unwrap()),
        retirement: Some(RetirementContainerCapacityKnob::new(1).unwrap()),
    }
}

fn violated(image: &singlefs_harness::memory_pool::MemoryPool) -> Vec<String> {
    check_pool_image_given_the_container_capacity_knobs(image, knobs())
        .iter()
        .filter_map(|(i, v)| match v {
            InvariantVerdict::Violated(d) => Some(format!("{i}: {d}")),
            _ => None,
        })
        .collect()
}

#[test]
fn zz_probe_crash_prefixes_of_unlink_chain() {
    let stride: usize = std::env::var("PROBE_STRIDE").ok().and_then(|s| s.parse().ok()).unwrap_or(1);
    let start: usize = std::env::var("PROBE_START").ok().and_then(|s| s.parse().ok()).unwrap_or(0);
    let end_cap: usize = std::env::var("PROBE_END").ok().and_then(|s| s.parse().ok()).unwrap_or(usize::MAX);
    let mut pool = pool_with_new_inodes("probe-abandon", 8);
    pool.allocator.set_deletion_switches(switches());
    unlink_to_the_end(&mut pool, 9);
    let v9 = pool.output.clone();
    unlink_to_the_end(&mut pool, 8);
    let kind = std::env::var("PROBE_KIND").unwrap_or_else(|_| "unlink".to_string());
    if kind == "rollback" {
        common_deletion_in_progress::roll_back_to(&mut pool, &v9);
    }
    let base = pool.memory_pool();
    assert!(violated(&base).is_empty(), "base image reds: {:?}", violated(&base));
    let ops_before = pool.retained_operations().len();
    if kind == "rollback" {
        common_deletion_in_progress::roll_back_to(&mut pool, &v9);
    } else {
        unlink_to_the_end(&mut pool, 7);
    }
    let ops = pool.retained_operations();
    let (writes, _segments, _idx) =
        writes_and_segments_with_stream_indexes(&ops[ops_before..], &geometry());
    eprintln!("PROBE writes={}", writes.len());
    let (mut mounted_ok, mut mount_err, mut reds) = (0usize, 0usize, Vec::<String>::new());
    let mut k = start;
    while k <= writes.len().min(end_cap) {
        let persisted: Vec<bool> = (0..writes.len()).map(|i| i < k).collect();
        let stream = SharedStream::new();
        let mut devices = crash_state_devices(&base, &writes, &persisted, &stream);
        match mount_writable_with_test_only_switches_and_deletion_switches(
            &parameters(),
            &mut devices,
            SpaceAdmission::JudgedByTheFormula,
            ShadowLedger::On,
            switches(),
        ) {
            Ok(_m) => {
                mounted_ok += 1;
                let image = memory_pool_of_sparse_devices(&devices);
                let v = violated(&image);
                if !v.is_empty() {
                    reds.push(format!("prefix {k}: {v:?}"));
                }
            }
            Err(e) => {
                mount_err += 1;
                if mount_err <= 3 {
                    eprintln!("PROBE mount err at prefix {k}: {e:?}");
                }
            }
        }
        k += stride;
    }
    eprintln!("PROBE mounted_ok={mounted_ok} mount_err={mount_err} reds={}", reds.len());
    for r in reds.iter().take(10) {
        eprintln!("PROBE RED {r}");
    }
    assert!(reds.is_empty());
}
