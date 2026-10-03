//! 攻方原型（m3-unlink-code-r1，opus 腿）：E2 准入开着把小盘写到被式子拒，再 unlink（会话一路做完，或发意图与 j 批后崩、重挂续做），
//! 看删除类发布会不会被空间拒到推满仍不够、挂载会不会交回错、池级 checker 绿不绿。只在草稿副本上跑。

mod common_admission;

use common_admission::{checker_violations_on, start_plain, PoolUnderTest};
use singlefs_core::address::InodeNumber;
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::container_tree::ContainerTreeNodeEntries;
use singlefs_core::lineage_tree::LineageContainerCapacity;
use singlefs_core::mount::{mount_writable_with_test_only_switches_and_deletion_switches, MountError, ShadowLedger};
use singlefs_core::mounted_session::{MountedSession, UserChange, UserChangeRefused};
use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
use singlefs_core::transaction::{publish_unlink_intent, publish_unlink_step, PoolWriter, FIRST_INODE_NUMBER};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::SparseBlockDevice;

fn switches(batch: u64) -> DeletionSwitches {
    DeletionSwitches {
        batch_size: if batch == 144 {
            UnlinkBatchSize::OneExtentLowerLeaf
        } else {
            UnlinkBatchSize::CappedForTests(UnlinkBatchDataUnitsCappedForTests::new(batch).unwrap())
        },
        tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
        container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    }
}

fn refusal_name(e: &UserChangeRefused) -> String {
    match e {
        UserChangeRefused::NoSpaceAfterRaisingTheFloor(n) => format!("NoSpace(stop={:?}, raises={}, published={})", n.stop, n.floor_raises.len(), n.versions_published.len()),
        UserChangeRefused::Publish { cause, versions_published, .. } => format!("Publish({cause:?}, published={})", versions_published.len()),
        other => format!("{other:?}").chars().take(200).collect(),
    }
}

/// 把文件长到被准入拒为止（每次多一个单元），交回最后做成的单元数。
fn grow_until_refused(pool: &mut PoolUnderTest<SparseBlockDevice>, step_units: usize) -> usize {
    let mut units = 1;
    loop {
        let next = units + step_units;
        let length = (next - 1) * data_unit_payload_capacity() + 100;
        match pool.sequential_write(length) {
            Ok(_) => units = next,
            Err(_) => return units,
        }
    }
}

fn mount_with(pool: &mut PoolUnderTest<SparseBlockDevice>, batch: u64) -> Result<(), MountError> {
    pool.session = None;
    let mounted = mount_writable_with_test_only_switches_and_deletion_switches(
        &pool.parameters,
        &mut pool.devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        switches(batch),
    )?;
    let (_output, session) = MountedSession::of_the_mount(mounted);
    pool.session = Some(session);
    Ok(())
}

fn case_session(width: HistoryDeviceWidth, batch: u64, overwrites: usize, remount_first: bool, step_units: usize) {
    let mut pool = start_plain(width);
    if remount_first {
        mount_with(&mut pool, batch).expect("first remount");
    } else {
        pool.session.as_mut().unwrap().allocator.set_deletion_switches(switches(batch));
    }
    for _ in 0..overwrites {
        if pool.overwrite(2999).is_err() { break; }
    }
    let units = grow_until_refused(&mut pool, step_units);
    let result = pool.publish(UserChange::Unlink { inode: InodeNumber(FIRST_INODE_NUMBER) }, None);
    let outcome = match &result {
        Ok(p) => format!("OK steps={} raises={}", p.versions_published.len(), p.floor_raises.len()),
        Err(e) => format!("REFUSED {}", refusal_name(e)),
    };
    let violations = checker_violations_on(&pool.image());
    println!("E2S width={width:?} K={batch} P={overwrites} remount_first={remount_first} grow_step={step_units} units={units} unlink={outcome} checker={}", violations.len());
    if !violations.is_empty() { println!("   violations: {:?}", &violations[..violations.len().min(3)]); }
}

fn case_crash(width: HistoryDeviceWidth, batch: u64, overwrites: usize, steps_before_crash: usize) {
    let mut pool = start_plain(width);
    mount_with(&mut pool, batch).expect("first remount");
    for _ in 0..overwrites {
        if pool.overwrite(2999).is_err() { break; }
    }
    let units = grow_until_refused(&mut pool, 1);
    // 进程里发意图与 j 批（与会话同一条发布路径，落点那一判照判），再崩。
    let mut published = 0;
    {
        let PoolUnderTest { session, devices, parameters, .. } = &mut pool;
        let session = session.as_mut().unwrap();
        let mut current = session.current.version_of_trees().unwrap().clone();
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        match publish_unlink_intent(&mut writer, &mut session.allocator, &current, InodeNumber(FIRST_INODE_NUMBER), session.instance) {
            Ok(v) => { current = v; published += 1; }
            Err(e) => { println!("E2C width={width:?} K={batch} P={overwrites} j={steps_before_crash} units={units} intent refused {e:?}"); return; }
        }
        for _ in 0..steps_before_crash {
            if current.deleted_inodes.is_empty() { break; }
            match publish_unlink_step(&mut writer, &mut session.allocator, &current, InodeNumber(FIRST_INODE_NUMBER), switches(batch).batch_size, session.instance) {
                Ok(v) => { current = v; published += 1; }
                Err(e) => { println!("   in-process step refused after {published}: {:?}", std::mem::discriminant(&e)); break; }
            }
        }
    }
    let mount = mount_with(&mut pool, batch);
    let outcome = match &mount {
        Ok(()) => {
            let cur = pool.session.as_ref().unwrap().current.version_of_trees().unwrap();
            format!("MOUNTED queue={}", cur.deleted_inodes.len())
        }
        Err(MountError::Publish(f)) => format!("MOUNT-ERR Publish({:?})", f.cause),
        Err(e) => format!("MOUNT-ERR {:?}", e).chars().take(200).collect(),
    };
    let violations = checker_violations_on(&pool.image());
    println!("E2C width={width:?} K={batch} P={overwrites} j={steps_before_crash} units={units} published_before_crash={published} mount={outcome} checker={}", violations.len());
    if !violations.is_empty() { println!("   violations: {:?}", &violations[..violations.len().min(3)]); }
}

#[test]
fn e2_session() {
    for width in [HistoryDeviceWidth::UnitAreaOf384Slots, HistoryDeviceWidth::UnitAreaOf256Slots, HistoryDeviceWidth::UnitAreaOf240Slots] {
        for batch in [1u64, 2, 144] {
            for overwrites in [0usize, 5, 20] {
                for remount_first in [false, true] {
                    case_session(width, batch, overwrites, remount_first, 1);
                }
            }
        }
    }
}

#[test]
fn e2_crash() {
    for width in [HistoryDeviceWidth::UnitAreaOf384Slots, HistoryDeviceWidth::UnitAreaOf256Slots, HistoryDeviceWidth::UnitAreaOf240Slots] {
        for batch in [1u64, 2] {
            for overwrites in [0usize, 5] {
                for j in [0usize, 1, 3, 10, 30, 60] {
                    case_crash(width, batch, overwrites, j);
                }
            }
        }
    }
}
