//! 攻方原型（m3-unlink-code-r1，opus 腿）：E3 挂载续做推满仍不够、交回 `MountError::Publish` 时，错里交出的写账与设备一层数到的写比。
//! 盘面照仓里 `admission_raises_the_floor_before_refusing.rs` 那条「续做推满仍不够」用例的造法（窄盘、准入关着造盘面、意图与 1 批之后崩），
//! 这里在原型里重写那几步，挂载时每块盘包一层录制。只在草稿副本上跑。

mod common_admission;

use common_admission::{
    start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off,
    PoolUnderTest, OVERWRITE_BYTES,
};
use singlefs_core::address::{DeviceIdentity, InodeNumber};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::container_tree::ContainerTreeNodeEntries;
use singlefs_core::lineage_tree::LineageContainerCapacity;
use singlefs_core::mount::{mount_writable_with_test_only_switches_and_deletion_switches, MountError, ShadowLedger};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
use singlefs_core::transaction::{publish_unlink_intent, publish_unlink_step, PoolWriter, FIRST_INODE_NUMBER};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::SparseBlockDevice;
use singlefs_harness::{RecordedOperationKind, RecordingBlockDevice, SharedStream};

fn switches_of_one() -> DeletionSwitches {
    DeletionSwitches {
        batch_size: UnlinkBatchSize::CappedForTests(UnlinkBatchDataUnitsCappedForTests::new(1).unwrap()),
        tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
        container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    }
}

fn crashed_pool(batches: usize) -> PoolUnderTest<SparseBlockDevice> {
    let mut pool =
        start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
            HistoryDeviceWidth::UnitAreaOf384Slots,
            256,
        );
    for _ in 0..2 {
        pool.overwrite(OVERWRITE_BYTES).unwrap();
    }
    pool.sequential_write(13 * data_unit_payload_capacity() + 100).unwrap();
    {
        let PoolUnderTest { session, devices, parameters, .. } = &mut pool;
        let session = session.as_mut().unwrap();
        let mut current = session.current.version_of_trees().unwrap().clone();
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        current = publish_unlink_intent(&mut writer, &mut session.allocator, &current, InodeNumber(FIRST_INODE_NUMBER), session.instance).unwrap();
        for _ in 0..batches {
            current = publish_unlink_step(&mut writer, &mut session.allocator, &current, InodeNumber(FIRST_INODE_NUMBER), switches_of_one().batch_size, session.instance).unwrap();
        }
    }
    pool.session = None;
    pool
}

#[test]
fn e3_writes_handed_back_by_a_mount_whose_continuation_stays_short() {
    let mut pool = crashed_pool(1);
    let txg_before = mount_read_only(&pool.image()).unwrap().effective_root.checkpoint_txg.0;
    let stream = SharedStream::new();
    let plain = std::mem::take(&mut pool.devices);
    let mut recorded: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = plain
        .into_iter()
        .map(|(identity, device)| (identity, RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())))
        .collect();
    let result = mount_writable_with_test_only_switches_and_deletion_switches(
        &pool.parameters,
        &mut recorded,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        switches_of_one(),
    );
    pool.devices = recorded.into_iter().map(|(identity, device)| (identity, device.into_inner_and_operations().0)).collect();
    let device_writes = stream
        .operations()
        .iter()
        .filter(|operation| operation.kind != RecordedOperationKind::Barrier)
        .count();
    let txg_after = mount_read_only(&pool.image()).unwrap().effective_root.checkpoint_txg.0;
    match result {
        Err(MountError::Publish(failed)) => {
            let persisted: u64 = failed.writes_of_persisted_publishes.iter().map(|w| w.total().write_calls).sum();
            let failed_writes: u64 = failed.writes_of_failed_publishes.iter().map(|w| w.total().write_calls).sum();
            let before = failed.writes_before_the_first_publish.total().write_calls;
            println!(
                "E3 cause={:?}\nE3 publishes_landed_by_the_failed_mount={} writes_of_persisted_publishes.len()={} accounted_write_calls(before+persisted+failed)={}+{}+{} device_level_write_calls_excluding_barriers={}",
                failed.cause,
                txg_after - txg_before,
                failed.writes_of_persisted_publishes.len(),
                before, persisted, failed_writes,
                device_writes
            );
        }
        Err(other) => println!("E3 other error {other:?}"),
        Ok(_) => println!("E3 mounted"),
    }
}

/// E8：同一份盘面（续做推满仍不够、挂载交回错之后），再挂几次：续做的文档说「下一次挂载接着走」。
#[test]
fn e8_mounting_again_after_the_continuation_stayed_short() {
    use common_admission::plain_devices_on;
    let mut pool = crashed_pool(1);
    for attempt in 1..=4 {
        let image = pool.image();
        let mut devices = plain_devices_on(&image);
        let result = mount_writable_with_test_only_switches_and_deletion_switches(
            &pool.parameters, &mut devices, SpaceAdmission::JudgedByTheFormula, ShadowLedger::On, switches_of_one());
        let txg_before = mount_read_only(&image).unwrap().effective_root.checkpoint_txg.0;
        pool.devices = devices;
        let txg_after = mount_read_only(&pool.image()).unwrap().effective_root.checkpoint_txg.0;
        let text = match &result {
            Ok(m) => format!("Mounted(queue={}, continuation_steps={})", m.current.version_of_trees().map_or(0, |v| v.deleted_inodes.len()), m.output.deletion_queue_continuation.len()),
            Err(e) => format!("{e:?}").chars().take(200).collect(),
        };
        println!("E8 attempt {attempt}: {text}; publishes_landed={}", txg_after - txg_before);
        if result.is_ok() { break; }
    }
}

/// E8b：原型臂（推满仍不够时挂载照样做成可写）：挂上之后正常卸载（抬 F 到现行那一版），再挂一次看续做走不走得完。
#[test]
fn e8b_arm_mount_writable_then_unmount_then_mount_again() {
    use common_admission::plain_devices_on;
    use singlefs_core::mount::unmount;
    if std::env::var("OPUS_ARM_MOUNT_WRITABLE_ON_CONTINUATION_SHORT").is_err() {
        println!("E8b skipped: arm not switched on");
        return;
    }
    let mut pool = crashed_pool(1);
    let image = pool.image();
    let mut devices = plain_devices_on(&image);
    let mounted = mount_writable_with_test_only_switches_and_deletion_switches(
        &pool.parameters, &mut devices, SpaceAdmission::JudgedByTheFormula, ShadowLedger::On, switches_of_one()).expect("arm: mount made writable");
    let queue = mounted.current.version_of_trees().map_or(0, |v| v.deleted_inodes.len());
    println!("E8b first mount (arm): queue={queue} continuation_steps={}", mounted.output.deletion_queue_continuation.len());
    let singlefs_core::mount::Mounted { mut allocator, mut current, .. } = mounted;
    let unmounted = unmount(&pool.parameters, &mut devices, &mut allocator, &mut current, ShadowLedger::On);
    println!("E8b unmount: {}", match &unmounted { Ok(_) => "ok".to_string(), Err(e) => format!("{e:?}").chars().take(200).collect::<String>() });
    pool.devices = devices;
    let after_unmount = pool.image();
    let mut devices = plain_devices_on(&after_unmount);
    let second = mount_writable_with_test_only_switches_and_deletion_switches(
        &pool.parameters, &mut devices, SpaceAdmission::JudgedByTheFormula, ShadowLedger::On, switches_of_one());
    match &second {
        Ok(m) => println!("E8b second mount: queue={} continuation_steps={} raises={}", m.current.version_of_trees().map_or(0, |v| v.deleted_inodes.len()), m.output.deletion_queue_continuation.len(), m.output.floor_raises_pushed_by_the_continuation.len()),
        Err(e) => println!("E8b second mount: {}", format!("{e:?}").chars().take(200).collect::<String>()),
    }
    pool.devices = devices;
    println!("E8b checker after second mount: {:?}", common_admission::checker_violations_on(&pool.image()));
}

/// E8c：取号之前预演取不到落点那一拒（`PlacementRefusedBeforeAcquisitionMountAdmissionUndecided`）在这份盘面上照旧「一个写都不发」吗：
/// 录制流数写与屏障，镜像前后逐字节比。
#[test]
fn e8c_the_refusal_before_acquisition_writes_nothing() {
    use common_admission::plain_devices_on;
    let mut pool = crashed_pool(1);
    // 第一次挂载：续做推满仍不够、交回错（已落盘 13 次）。
    {
        let image = pool.image();
        let mut devices = plain_devices_on(&image);
        let _ = mount_writable_with_test_only_switches_and_deletion_switches(
            &pool.parameters, &mut devices, SpaceAdmission::JudgedByTheFormula, ShadowLedger::On, switches_of_one());
        pool.devices = devices;
    }
    let before = pool.image();
    let stream = SharedStream::new();
    let mut recorded: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = plain_devices_on(&before)
        .into_iter()
        .map(|(identity, device)| (identity, RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())))
        .collect();
    let result = mount_writable_with_test_only_switches_and_deletion_switches(
        &pool.parameters, &mut recorded, SpaceAdmission::JudgedByTheFormula, ShadowLedger::On, switches_of_one());
    let after_devices: Vec<(DeviceIdentity, SparseBlockDevice)> = recorded.into_iter().map(|(i, d)| (i, d.into_inner_and_operations().0)).collect();
    let identical = before.devices.iter().all(|(identity, image)| after_devices.iter().find(|(i, _)| i == identity).map_or(false, |(_, d)| d.image == *image));
    println!("E8c second mount: {}; recorded_operations={} images_identical={identical}", format!("{:?}", result.err()).chars().take(160).collect::<String>(), stream.operations().len());
}
