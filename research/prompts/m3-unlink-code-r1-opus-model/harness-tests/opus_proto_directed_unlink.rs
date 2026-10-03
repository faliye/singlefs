//! 攻方原型（m3-unlink-code-r1，opus 腿）：E5 定向几段历史——回退到删除中的一版之后会话先走空（F1）那几条路、
//! 删非第一个 inode 走到删空、走空之后被拒时交回的账。每一步之后跑池级 checker。只在草稿副本上跑。

mod common_admission;

use common_admission::{checker_violations_on, start_plain, PoolUnderTest};
use singlefs_core::address::InodeNumber;
use singlefs_core::container_tree::ContainerTreeNodeEntries;
use singlefs_core::lineage_tree::LineageContainerCapacity;
use singlefs_core::mount::{roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::mounted_session::{UserChange, UserChangeRefused};
use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
use singlefs_core::transaction::{FirstFile, PoolVersion, FIRST_INODE_NUMBER};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::SparseBlockDevice;

fn switches(batch: u64) -> DeletionSwitches {
    DeletionSwitches {
        batch_size: UnlinkBatchSize::CappedForTests(UnlinkBatchDataUnitsCappedForTests::new(batch).unwrap()),
        tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
        container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    }
}

fn member(v: &PoolVersion) -> &'static str {
    match v {
        PoolVersion::WithoutFile(_) => "WithoutFile",
        PoolVersion::WithFile(_) => "WithFile",
        PoolVersion::WithTreesAndNoFile(_) => "WithTreesAndNoFile",
    }
}

fn txg_on_disk(pool: &PoolUnderTest<SparseBlockDevice>) -> u64 {
    mount_read_only(&pool.image()).unwrap().effective_root.checkpoint_txg.0
}

fn rollback_to(pool: &mut PoolUnderTest<SparseBlockDevice>, txg: u64) {
    let PoolUnderTest { session, devices, parameters, .. } = pool;
    let session = session.as_mut().unwrap();
    let mut current = session.current.version_of_trees().unwrap().clone();
    let target = RollbackTarget { instance: session.instance, checkpoint_txg: singlefs_core::address::CheckpointTxg(txg) };
    roll_back_by_a_forward_publish(parameters, devices, &mut session.allocator, &mut current, target)
        .unwrap_or_else(|e| panic!("rollback to {txg}: {e:?}"));
    session.current = PoolVersion::of_a_version_of_trees(current);
}

fn pool_with_units(units: usize) -> PoolUnderTest<SparseBlockDevice> {
    let mut pool = start_plain(HistoryDeviceWidth::FourGibibytes);
    pool.session.as_mut().unwrap().allocator.set_deletion_switches(switches(1));
    pool.sequential_write((units - 1) * data_unit_payload_capacity() + 100).unwrap();
    pool
}

/// 回退到意图那一版（队列一条）之后各种用户改动：会话先走空，再做它自己。打印交回的成员、交回里带的版数、盘上 txg 前后差。
#[test]
fn e5a_user_changes_after_rolling_back_to_the_intent() {
    for change_kind in ["overwrite", "create", "unlink", "seqwrite"] {
        let mut pool = pool_with_units(4);
        let before_unlink = pool.session().current.root().checkpoint_txg.0;
        let published = pool.publish(UserChange::Unlink { inode: InodeNumber(FIRST_INODE_NUMBER) }, None).unwrap();
        let intent_txg = published.versions_published[0].root.checkpoint_txg.0;
        assert_eq!(intent_txg, before_unlink + 1);
        rollback_to(&mut pool, intent_txg);
        let queue = pool.session().current.version_of_trees().unwrap().deleted_inodes.clone();
        let txg_before = txg_on_disk(&pool);
        let content = vec![7u8; 100];
        let result = match change_kind {
            "overwrite" => pool.publish(UserChange::Overwrite(FirstFile { content: &content, write_time_seconds: 1 }), None),
            "seqwrite" => pool.publish(UserChange::SequentialWrite(FirstFile { content: &content, write_time_seconds: 1 }), None),
            "create" => pool.create_one_inode(),
            _ => pool.publish(UserChange::Unlink { inode: InodeNumber(FIRST_INODE_NUMBER) }, None),
        };
        let txg_after = txg_on_disk(&pool);
        let handed_back = match &result {
            Ok(p) => format!("Ok(versions={}, raises={})", p.versions_published.len(), p.floor_raises.len()),
            Err(UserChangeRefused::NoFileVersionToChange) => "Err(NoFileVersionToChange: carries no versions)".to_string(),
            Err(UserChangeRefused::Publish { cause, versions_published, .. }) => format!("Err(Publish {cause:?}, versions={})", versions_published.len()),
            Err(other) => format!("Err({other:?})").chars().take(160).collect(),
        };
        let violations = checker_violations_on(&pool.image());
        println!(
            "E5a after-rollback-to-intent queue={queue:?} change={change_kind} -> {handed_back}; publishes_landed_on_disk={} current_member={} checker={}",
            txg_after - txg_before,
            member(&pool.session().current),
            violations.len()
        );
        if !violations.is_empty() { println!("   {:?}", violations); }
    }
}

/// 删完第一个文件之后建两个 inode，再 unlink 它们（非第一个 inode：意图 + 最后一批）。看删到最后一个时那一版归不归「有树而文件集合为空」。
#[test]
fn e5b_unlink_the_inodes_created_after_the_first_file() {
    let mut pool = pool_with_units(2);
    pool.publish(UserChange::Unlink { inode: InodeNumber(FIRST_INODE_NUMBER) }, None).unwrap();
    println!("E5b after unlink 1: {}", member(&pool.session().current));
    pool.create_one_inode().unwrap();
    pool.create_one_inode().unwrap();
    let inodes: Vec<u64> = pool.session().current.version_of_trees().unwrap().inode_leaf_containers.iter().flat_map(|c| c.contents.records.iter().map(|r| r.inode)).collect();
    println!("E5b after two creates: {} inodes={inodes:?}", member(&pool.session().current));
    for inode in inodes {
        let r = pool.publish(UserChange::Unlink { inode: InodeNumber(inode) }, None);
        let v = pool.session().current.version_of_trees().unwrap();
        println!(
            "E5b unlink {inode}: {} -> {} containers={} data_pointers={} queue={:?} tombstones={} checker={}",
            if r.is_ok() { "ok" } else { "refused" },
            member(&pool.session().current),
            v.inode_leaf_containers.len(),
            v.data_pointers.len(),
            v.deleted_inodes,
            v.tombstone_containers.iter().map(|c| c.contents.records.len()).sum::<usize>(),
            checker_violations_on(&pool.image()).len()
        );
        if let Err(e) = r { println!("   {e:?}"); }
    }
}

/// 第一个文件活着时先建一个 inode，再 unlink 第一个文件（多批），再 unlink 那个 inode：两次删除的中间态与终态。
#[test]
fn e5c_unlink_first_file_while_another_inode_lives_then_that_inode() {
    let mut pool = pool_with_units(3);
    pool.create_one_inode().unwrap();
    let r1 = pool.publish(UserChange::Unlink { inode: InodeNumber(FIRST_INODE_NUMBER) }, None);
    println!("E5c unlink 1: {:?} -> {} checker={}", r1.as_ref().map(|p| p.versions_published.len()), member(&pool.session().current), checker_violations_on(&pool.image()).len());
    let other = pool.session().current.version_of_trees().unwrap().inode_leaf_containers.iter().flat_map(|c| c.contents.records.iter().map(|r| r.inode)).next();
    if let Some(other) = other {
        let r2 = pool.publish(UserChange::Unlink { inode: InodeNumber(other) }, None);
        let v = pool.session().current.version_of_trees().unwrap();
        println!("E5c unlink {other}: {:?} -> {} containers={} checker={}", r2.as_ref().map(|p| p.versions_published.len()), member(&pool.session().current), v.inode_leaf_containers.len(), checker_violations_on(&pool.image()).len());
    }
}
