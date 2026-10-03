//! 攻方原型（m3-unlink-code-r1，opus 腿）：E1 量每一次删除类发布每块盘新占的槽数 W'，与实现算的 W 比；
//! 同时量落点那一判按 ckpt_cost 要的槽与 W' 的比。只在草稿副本上跑。

mod common_admission;

use common_admission::{start_plain, PoolUnderTest};
use singlefs_core::address::{DeviceIdentity, InodeNumber};
use singlefs_core::admission::{checkpoint_cost_of_the_version_to_build_on, ring_window_on_one_device};
use singlefs_core::container_tree::ContainerTreeNodeEntries;
use singlefs_core::lineage_tree::LineageContainerCapacity;
use singlefs_core::mounted_session::UserChange;
use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
use singlefs_core::transaction::{TransactionOutput, FIRST_INODE_NUMBER};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::SparseBlockDevice;

fn switches(batch: u64, node_entries: ContainerTreeNodeEntries) -> DeletionSwitches {
    DeletionSwitches {
        batch_size: if batch == 144 {
            UnlinkBatchSize::OneExtentLowerLeaf
        } else {
            UnlinkBatchSize::CappedForTests(UnlinkBatchDataUnitsCappedForTests::new(batch).unwrap())
        },
        tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
        container_tree_node_entries: node_entries,
    }
}

fn new_slots_per_device(version: &TransactionOutput) -> Vec<(DeviceIdentity, u64)> {
    let txg = version.root.checkpoint_txg;
    let mut per: std::collections::BTreeMap<DeviceIdentity, u64> = Default::default();
    for record in &version.allocation_records {
        if !record.is_released && record.generation == txg {
            *per.entry(record.device).or_default() += u64::from(record.span_slots);
        }
    }
    per.into_iter().collect()
}

fn run_one(width: HistoryDeviceWidth, batch: u64, units: usize, overwrites: usize, label: &str) {
    let mut pool: PoolUnderTest<SparseBlockDevice> = start_plain(width);
    {
        let session = pool.session.as_mut().unwrap();
        session.allocator.set_deletion_switches(switches(batch, ContainerTreeNodeEntries::FromTheNodeFormat));
    }
    for _ in 0..overwrites {
        if let Err(e) = pool.overwrite(2999) {
            println!("E1 {label} overwrite refused: {e:?}");
            return;
        }
    }
    if units > 1 {
        let length = (units - 1) * data_unit_payload_capacity() + 100;
        if let Err(e) = pool.sequential_write(length) {
            println!("E1 {label} seqwrite refused: {:?}", std::mem::discriminant(&e));
            return;
        }
    }
    let session = pool.session.as_ref().unwrap();
    let slots_per_region = session.allocator.root_ring_occupancy().unwrap().slots_per_region();
    let window = ring_window_on_one_device(&session.allocator, slots_per_region, batch).0 / 16384;
    let roots_plus_one = 3 * slots_per_region.count() + 1;
    let w = window / roots_plus_one;
    let ckpt = checkpoint_cost_of_the_version_to_build_on(session.current.version_of_trees(), &session.allocator).0;
    let result = pool.publish(UserChange::Unlink { inode: InodeNumber(FIRST_INODE_NUMBER) }, None);
    let (versions, raises) = match &result {
        Ok(p) => (p.versions_published.clone(), p.floor_raises.len()),
        Err(e) => {
            println!("E1 {label} unlink refused: {e:?}");
            return;
        }
    };
    let mut max_new = 0;
    let mut per_step = Vec::new();
    for v in &versions {
        let per = new_slots_per_device(v);
        let m = per.iter().map(|(_, n)| *n).max().unwrap_or(0);
        max_new = max_new.max(m);
        per_step.push(m);
    }
    println!(
        "E1 {label} width={width:?} K={batch} U={units} P={overwrites} steps={} raises={raises} W={w} ring={window} ckpt={ckpt} maxW'={max_new} exceed={} per_step={:?}",
        versions.len(),
        max_new > w,
        per_step
    );
}

#[test]
fn e1_deletion_publish_new_slots_against_w() {
    for (width, sizes) in [
        (HistoryDeviceWidth::UnitAreaOf384Slots, vec![1usize, 3, 14, 40, 80]),
        (HistoryDeviceWidth::FourGibibytes, vec![1usize, 3, 145, 300, 600]),
    ] {
        for batch in [1u64, 3, 144] {
            for &units in &sizes {
                if batch == 1 && units > 300 { continue; }
                for overwrites in [0usize, 10] {
                    run_one(width, batch, units, overwrites, "base");
                }
            }
        }
    }
}
