//! 攻方探针（副本，不入库）：退役容器、谱系容器、墓碑容器「提早关」（非最右容器没装满）的镜像，池级 checker 红不红。
#![allow(dead_code, clippy::all)]
mod common_admission;

use common_admission::*;
use singlefs_core::address::InodeNumber;
use singlefs_core::container_tree::ContainerTreeNodeEntries;
use singlefs_core::lineage_tree::{LineageContainerCapacity, LineageRecordsPerContainerCappedForTests};
use singlefs_core::mount::{roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::mounted_session::UserChange;
use singlefs_core::retirement_tree::{RetirementContainerCapacity, RetirementRecordsPerContainerCappedForTests};
use singlefs_core::tombstone_tree::{TombstoneContainerCapacity, TombstoneRecordsPerContainerCappedForTests};
use singlefs_core::transaction::PoolVersion;
use singlefs_core::unlink::{DeletionSwitches, UnlinkBatchSize};
use singlefs_harness::history::HistoryDeviceWidth;

fn run(name: &str, retire: Option<u64>, lineage: Option<u64>, tomb: Option<u64>) {
    let mut pool = start_plain(HistoryDeviceWidth::UnitAreaOf384Slots);
    pool.session.as_mut().unwrap().allocator.set_deletion_switches(DeletionSwitches {
        batch_size: UnlinkBatchSize::OneExtentLowerLeaf,
        tombstone_container_capacity: tomb.map_or(TombstoneContainerCapacity::FromTheContainerFormat, |n| TombstoneContainerCapacity::CappedForTests(TombstoneRecordsPerContainerCappedForTests::new(n).unwrap())),
        lineage_container_capacity: lineage.map_or(LineageContainerCapacity::FromTheContainerFormat, |n| LineageContainerCapacity::CappedForTests(LineageRecordsPerContainerCappedForTests::new(n).unwrap())),
        retirement_container_capacity: retire.map_or(RetirementContainerCapacity::FromTheContainerFormat, |n| RetirementContainerCapacity::CappedForTests(RetirementRecordsPerContainerCappedForTests::new(n).unwrap())),
        container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    });
    let mut wt = pool.write_time_seconds;
    let mut publish = |pool: &mut PoolUnderTest<_>, c: UserChange<'static>| { pool.publish(c, None).expect("发布") };
    wt += 1;
    publish(&mut pool, UserChange::NewInodes { count: 233, write_time_seconds: wt });
    // 叶 [1..=233]、[234]；删 234（最右那片删空，退役一条）
    wt += 1;
    publish(&mut pool, UserChange::Unlink { inode: InodeNumber(234), write_time_seconds: wt });
    let after_first = pool.current_file_version().clone();
    wt += 1;
    publish(&mut pool, UserChange::NewInodes { count: 1, write_time_seconds: wt });
    wt += 1;
    publish(&mut pool, UserChange::Unlink { inode: InodeNumber(235), write_time_seconds: wt });
    // 两次回退（谱系两条）
    for _ in 0..2 {
        let target = RollbackTarget { instance: after_first.root.instance, checkpoint_txg: after_first.root.checkpoint_txg };
        let mut session = pool.session.take().unwrap();
        let mut cur = session.current.file_version().unwrap().clone();
        roll_back_by_a_forward_publish(&pool.parameters, &mut pool.devices, &mut session.allocator, &mut cur, target).expect("回退");
        session.current = PoolVersion::of_a_version_of_trees(cur);
        pool.session = Some(session);
    }
    let v = pool.current_file_version();
    let shape = |n: &str, counts: Vec<usize>, numbers: Vec<u64>| format!("{n} containers={} records={counts:?} numbers={numbers:?}", counts.len());
    println!("CASE {name}");
    println!("  {}", shape("retirement", v.retirement_containers.iter().map(|c| c.contents.records.len()).collect(), v.retirement_containers.iter().map(|c| c.contents.identity.container).collect()));
    println!("  {}", shape("lineage", v.lineage_containers.iter().map(|c| c.contents.records.len()).collect(), v.lineage_containers.iter().map(|c| c.contents.identity.container).collect()));
    println!("  {}", shape("tombstone", v.tombstone_containers.iter().map(|c| c.contents.records.len()).collect(), v.tombstone_containers.iter().map(|c| c.contents.identity.container).collect()));
    let violations = checker_violations_on(&pool.image());
    println!("  checker violations: {}", violations.len());
    for x in violations {
        println!("  VIOLATION {x:.200}");
    }
}

#[test]
fn product_capacities() { run("product", None, None, None); }
#[test]
fn retirement_and_lineage_closed_after_one_record() { run("retire1-lineage1", Some(1), Some(1), None); }
#[test]
fn tombstone_closed_after_one_record() { run("tomb1", None, None, Some(1)); }
