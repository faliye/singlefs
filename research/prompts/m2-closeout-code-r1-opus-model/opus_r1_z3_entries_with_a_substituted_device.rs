//! 代码轮第一轮云端攻方腿（m2-closeout-code-r1）Z3 那一格：挂着之后收盘表的入口（正常卸载、抬 F、管理员回退）
//! 只经 `caller_inputs_agreeing_with_the_disk` 核盘表。它比「盘数 = devs」与「每块盘自证过的系统配置槽里记的本盘设备号 = 盘表身份」，
//! 而一块盘上一个本池 fsid 的自证槽都没有时（空盘、别的池的盘）那一块一条不一致都报不出来——可写挂载那一处另有
//! `devices_without_the_selected_version` 把「没有自证过的系统配置」记成不带所选版本、拒掉，这几个入口没有这一判。
//! 用例钉的是快照今天的行为（这几个入口接受、并把本池的写发到换上去的那块盘上），不是该有的行为。
mod common;
mod common_admission;

use common_admission::{content_of, start_plain, OVERWRITE_BYTES};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor, roll_back_by_a_forward_publish, unmount, RollbackTarget,
    ShadowLedger, Unmounted,
};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::transaction::PoolVersion;
use singlefs_harness::crash::{MemoryPool, SparseBlockDevice};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

type Recorded = RecordingBlockDevice<SparseBlockDevice>;

fn recorded(identity: DeviceIdentity, device: SparseBlockDevice, stream: &SharedStream) -> (DeviceIdentity, Recorded) {
    (identity, RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()))
}

fn copy_of(image: &MemoryPool, identity: DeviceIdentity) -> SparseBlockDevice {
    let mut device = SparseBlockDevice::new(image.device_size_in_bytes, PhysicalBlockSizeInBytes(512));
    device.image = image.devices.get(&identity).cloned().expect("镜像里有这块盘");
    device
}

fn blank(image: &MemoryPool) -> SparseBlockDevice {
    SparseBlockDevice::new(image.device_size_in_bytes, PhysicalBlockSizeInBytes(512))
}

/// 另一个池（fsid 首字节翻转）mkfs 之后的两块盘。
fn foreign_pool(width: HistoryDeviceWidth) -> MemoryPool {
    let mut parameters = width.parameters();
    parameters.filesystem_identifier[0] ^= 0xff;
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| (identity, SparseBlockDevice::new(width.device_bytes(), PhysicalBlockSizeInBytes(512))))
        .collect();
    make_filesystem(&parameters, &mut devices).expect("另一个池 mkfs");
    MemoryPool {
        devices: devices.into_iter().map(|(identity, device)| (identity, device.image)).collect(),
        device_size_in_bytes: width.device_bytes(),
    }
}

fn writes_on(stream: &SharedStream, identity: DeviceIdentity) -> usize {
    stream
        .operations()
        .iter()
        .filter(|operation| operation.device == identity)
        .count()
}

/// 会话里第一个文件之后覆盖写两次（txg 4、5），交回池与此刻的镜像。
fn pool_after_two_overwrites() -> (common_admission::PoolUnderTest<SparseBlockDevice>, MemoryPool) {
    let mut pool = start_plain(HistoryDeviceWidth::FourGibibytes);
    pool.overwrite(OVERWRITE_BYTES).expect("覆盖写 1");
    pool.overwrite(OVERWRITE_BYTES).expect("覆盖写 2");
    let image = pool.image();
    (pool, image)
}

#[test]
fn unmount_with_device_one_replaced_by_a_blank_disk() {
    let (mut pool, image) = pool_after_two_overwrites();
    let mut session = pool.session.take().expect("会话");
    let stream = SharedStream::new();
    let mut devices = vec![
        recorded(DeviceIdentity(0), copy_of(&image, DeviceIdentity(0)), &stream),
        recorded(DeviceIdentity(1), blank(&image), &stream),
    ];
    let outcome = unmount(&pool.parameters, &mut devices, &mut session.allocator, &mut session.current, ShadowLedger::On);
    let accepted = matches!(outcome, Ok(Unmounted::FloorRaisedToTheCurrentVersion(_)));
    println!(
        "Z3A unmount_blank accepted={accepted} outcome_is_err={} writes_dev0={} writes_blank_dev1={}",
        outcome.is_err(),
        writes_on(&stream, DeviceIdentity(0)),
        writes_on(&stream, DeviceIdentity(1)),
    );
    // 真的盘 1 没收到这一串；拿盘 0（卸载之后）与真的盘 1 再挂。
    let after = MemoryPool {
        devices: [
            (DeviceIdentity(0), devices[0].1.inner().image.clone()),
            (DeviceIdentity(1), image.devices[&DeviceIdentity(1)].clone()),
        ]
        .into_iter()
        .collect(),
        device_size_in_bytes: image.device_size_in_bytes,
    };
    let read_only = mount_read_only(&after);
    match &read_only {
        Ok(mounted) => println!(
            "Z3A unmount_blank read_only_after chosen=({},{}) effective=({},{}) floor={}",
            mounted.chosen_root.instance.0, mounted.chosen_root.checkpoint_txg.0,
            mounted.effective_root.instance.0, mounted.effective_root.checkpoint_txg.0,
            mounted.effective_root.rollback_floor.0
        ),
        Err(error) => println!("Z3A unmount_blank read_only_after err={error:?}"),
    }
    let violations = common_admission::checker_violations_on(&after);
    println!("Z3A unmount_blank checker_violations_after={} first={:?}", violations.len(), violations.first());
    let mut remount: Vec<(DeviceIdentity, SparseBlockDevice)> = vec![
        (DeviceIdentity(0), copy_of(&after, DeviceIdentity(0))),
        (DeviceIdentity(1), copy_of(&after, DeviceIdentity(1))),
    ];
    match mount_writable(&pool.parameters, &mut remount) {
        Ok(mounted) => println!("Z3A unmount_blank writable_after ok instance={}", mounted.output.instance.0),
        Err(error) => println!("Z3A unmount_blank writable_after err={error:?}"),
    }
    assert!(accepted, "快照今天：盘 1 换成空盘，正常卸载照样做成（这一格就是打中的那一处）");
    assert!(writes_on(&stream, DeviceIdentity(1)) > 0, "空盘收到了本池的写");
}

#[test]
fn unmount_with_device_one_replaced_by_device_one_of_another_pool() {
    let (mut pool, image) = pool_after_two_overwrites();
    let foreign = foreign_pool(HistoryDeviceWidth::FourGibibytes);
    let foreign_before = foreign.clone();
    let mut session = pool.session.take().expect("会话");
    let stream = SharedStream::new();
    let mut devices = vec![
        recorded(DeviceIdentity(0), copy_of(&image, DeviceIdentity(0)), &stream),
        recorded(DeviceIdentity(1), copy_of(&foreign, DeviceIdentity(1)), &stream),
    ];
    let outcome = unmount(&pool.parameters, &mut devices, &mut session.allocator, &mut session.current, ShadowLedger::On);
    let accepted = matches!(outcome, Ok(Unmounted::FloorRaisedToTheCurrentVersion(_)));
    if let Err(error) = &outcome {
        println!("Z3A unmount_foreign err={error:?}");
    }
    println!(
        "Z3A unmount_foreign accepted={accepted} writes_dev0={} writes_foreign_dev1={}",
        writes_on(&stream, DeviceIdentity(0)),
        writes_on(&stream, DeviceIdentity(1)),
    );
    let foreign_after = MemoryPool {
        devices: [
            (DeviceIdentity(0), foreign_before.devices[&DeviceIdentity(0)].clone()),
            (DeviceIdentity(1), devices[1].1.inner().image.clone()),
        ]
        .into_iter()
        .collect(),
        device_size_in_bytes: foreign.device_size_in_bytes,
    };
    let mut foreign_parameters = HistoryDeviceWidth::FourGibibytes.parameters();
    foreign_parameters.filesystem_identifier[0] ^= 0xff;
    let mut foreign_devices: Vec<(DeviceIdentity, SparseBlockDevice)> = vec![
        (DeviceIdentity(0), copy_of(&foreign_after, DeviceIdentity(0))),
        (DeviceIdentity(1), copy_of(&foreign_after, DeviceIdentity(1))),
    ];
    match mount_writable(&foreign_parameters, &mut foreign_devices) {
        Ok(mounted) => println!("Z3A unmount_foreign other_pool_writable_after ok instance={}", mounted.output.instance.0),
        Err(error) => println!("Z3A unmount_foreign other_pool_writable_after err={error:?}"),
    }
    let mut foreign_devices_before: Vec<(DeviceIdentity, SparseBlockDevice)> = vec![
        (DeviceIdentity(0), copy_of(&foreign_before, DeviceIdentity(0))),
        (DeviceIdentity(1), copy_of(&foreign_before, DeviceIdentity(1))),
    ];
    println!(
        "Z3A unmount_foreign other_pool_writable_before ok={}",
        mount_writable(&foreign_parameters, &mut foreign_devices_before).is_ok()
    );
    // 没打中：两块盘上各有一份自证过、fsid 不同的系统配置，择系统配置那一步报 SystemConfigurationsDisagree，一个写都没发。
    assert!(!accepted && writes_on(&stream, DeviceIdentity(1)) == 0, "别的池的盘（系统配置读得出）在任何写之前被拒");
}

#[test]
fn raising_the_floor_and_rolling_back_with_device_one_replaced_by_a_blank_disk() {
    let (mut pool, image) = pool_after_two_overwrites();
    let mut session = pool.session.take().expect("会话");
    let stream = SharedStream::new();
    let mut devices = vec![
        recorded(DeviceIdentity(0), copy_of(&image, DeviceIdentity(0)), &stream),
        recorded(DeviceIdentity(1), blank(&image), &stream),
    ];
    let PoolVersion::WithFile(current) = &mut session.current else { panic!("带文件") };
    let mut allocator = session.allocator.clone();
    let mut current_for_raise = current.clone();
    let raise = raise_rollback_floor(&pool.parameters, &mut devices, &mut allocator, &mut current_for_raise, CheckpointTxg(3), ShadowLedger::On);
    println!("Z3A raise_blank ok={} err={:?} writes_blank_dev1={}", raise.is_ok(), raise.as_ref().err(), writes_on(&stream, DeviceIdentity(1)));
    let raised_ok = raise.is_ok();
    let stream2 = SharedStream::new();
    let mut devices2 = vec![
        recorded(DeviceIdentity(0), copy_of(&image, DeviceIdentity(0)), &stream2),
        recorded(DeviceIdentity(1), blank(&image), &stream2),
    ];
    let mut allocator_for_old = session.allocator.clone();
    let mut current_for_old = current.clone();
    let rollback_to_old = roll_back_by_a_forward_publish(
        &pool.parameters,
        &mut devices2,
        &mut allocator_for_old,
        &mut current_for_old,
        RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(3) },
    );
    println!("Z3A rollback_blank_to_txg3 ok={} err={:?} writes_blank_dev1={}", rollback_to_old.is_ok(), rollback_to_old.as_ref().err(), writes_on(&stream2, DeviceIdentity(1)));
    let current_txg = current.root.checkpoint_txg;
    let rollback = roll_back_by_a_forward_publish(
        &pool.parameters,
        &mut devices2,
        &mut session.allocator,
        current,
        RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: current_txg },
    );
    println!("Z3A rollback_blank_to_current ok={} err={:?} writes_blank_dev1={}", rollback.is_ok(), rollback.as_ref().err(), writes_on(&stream2, DeviceIdentity(1)));
    // 抬 F：算上限读根环时盘 1 上「知道住着根」的槽重读仍坏，在任何写之前拒（侧面挡住，不是盘表那一判挡的）。
    // 回退到 txg 3：复活集逐盘验读到空盘，拒。回退到现行那一版：没有复活集、没有读根环的那一判，做成、空盘收到写。
    assert!(!raised_ok, "抬 F 被读根环那一判侧面拒");
    assert!(rollback_to_old.is_err(), "回退到 txg 3 被复活集逐盘验拒");
    assert!(rollback.is_ok() && writes_on(&stream2, DeviceIdentity(1)) > 0, "快照今天：回退到现行那一版在盘 1 换成空盘时照样做成、空盘收到写");
    let _ = content_of;
}

/// 挂着的会话只比盘表的身份（实审 A1b 报告「会话只比盘表的设备身份，不读盘」那一条，实现员自报、规格没写）：盘 1 换成空盘照样发布。
#[test]
fn the_session_publishes_with_device_one_replaced_by_a_blank_disk() {
    let (mut pool, image) = pool_after_two_overwrites();
    let real_device_one = pool.devices[1].1.image.clone();
    pool.devices[1].1 = blank(&image);
    let outcome = pool.overwrite(OVERWRITE_BYTES);
    let blank_written = pool.devices[1].1.image != blank(&image).image;
    println!("Z3A session_blank ok={} err={:?} blank_written={blank_written}", outcome.is_ok(), outcome.as_ref().err());
    assert!(outcome.is_ok() && blank_written, "快照今天：会话在盘 1 换成空盘时照样发布、空盘收到写");
    let _ = real_device_one;
}
