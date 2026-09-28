//! 代码轮第二轮云端攻方腿（m2-closeout-code-r2）Y2：Z3-A 乙之后，挂着之后的入口与会话只核「可见」（至少一份本池自证的系统配置槽）。
//! 这里换三种「可见」的盘进去：同池旧快照的盘 1、两块盘的盘体对调（盘表身份不变）、只剩一槽自证的盘 1。
//! 用例只打印结局、钉快照今天的行为（打中的样子），不是该有的样子。
mod common_admission;

use common_admission::{checker_violations_on, content_of, start_plain, OVERWRITE_BYTES};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::mount::{mount_writable, unmount, ShadowLedger, Unmounted};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::mounted_session::UserChange;
use singlefs_core::transaction::FirstFile;
use singlefs_harness::crash::{MemoryPool, SparseBlockDevice};
use singlefs_harness::history::HistoryDeviceWidth;

const WIDTH: HistoryDeviceWidth = HistoryDeviceWidth::FourGibibytes;

fn copy_of(image: &MemoryPool, identity: DeviceIdentity) -> SparseBlockDevice {
    let mut device = SparseBlockDevice::new(image.device_size_in_bytes, PhysicalBlockSizeInBytes(512));
    device.image = image.devices.get(&identity).cloned().expect("镜像里有这块盘");
    device
}

fn duplicate(device: &SparseBlockDevice, bytes: u64) -> SparseBlockDevice {
    let mut copy = SparseBlockDevice::new(bytes, PhysicalBlockSizeInBytes(512));
    copy.image = device.image.clone();
    copy
}

fn pool_of(devices: &[(DeviceIdentity, SparseBlockDevice)], bytes: u64) -> MemoryPool {
    MemoryPool {
        devices: devices.iter().map(|(identity, device)| (*identity, device.image.clone())).collect(),
        device_size_in_bytes: bytes,
    }
}

fn report_after(tag: &str, parameters: &singlefs_core::make_filesystem::MakeFilesystemParameters, image: &MemoryPool) {
    match mount_read_only(image) {
        Ok(mounted) => println!(
            "Y2 {tag} read_only_after chosen=({},{}) effective=({},{})",
            mounted.chosen_root.instance.0,
            mounted.chosen_root.checkpoint_txg.0,
            mounted.effective_root.instance.0,
            mounted.effective_root.checkpoint_txg.0
        ),
        Err(error) => println!("Y2 {tag} read_only_after err={error:?}"),
    }
    let violations = checker_violations_on(image);
    println!("Y2 {tag} checker_violations_after={} first={:?}", violations.len(), violations.first());
    let mut remount: Vec<(DeviceIdentity, SparseBlockDevice)> =
        vec![(DeviceIdentity(0), copy_of(image, DeviceIdentity(0))), (DeviceIdentity(1), copy_of(image, DeviceIdentity(1)))];
    match mount_writable(parameters, &mut remount) {
        Ok(mounted) => {
            println!("Y2 {tag} writable_after ok instance={}", mounted.output.instance.0);
            let after = pool_of(&remount, image.device_size_in_bytes);
            let violations = checker_violations_on(&after);
            println!("Y2 {tag} checker_violations_after_writable_mount={} first={:?}", violations.len(), violations.first());
        }
        Err(error) => println!("Y2 {tag} writable_after err={}", format!("{error:?}").chars().take(400).collect::<String>()),
    }
}

/// 会话里盘 1 换成同池盘 1 的旧快照（第一个文件那一刻的样子；之后会话覆盖写了两次，盘 1 真身已前进）。
#[test]
fn session_publishes_onto_a_stale_snapshot_of_device_one() {
    let mut pool = start_plain(WIDTH);
    let early = pool.image();
    pool.overwrite(OVERWRITE_BYTES).expect("覆盖写 1");
    pool.overwrite(OVERWRITE_BYTES).expect("覆盖写 2");
    let now = pool.image();
    let write_time_seconds = pool.write_time_seconds + 1;
    let content = content_of(OVERWRITE_BYTES, write_time_seconds);
    let change = UserChange::Overwrite(FirstFile { content: &content, write_time_seconds });
    let session = pool.session.as_mut().expect("会话");
    let mut devices = vec![(DeviceIdentity(0), copy_of(&now, DeviceIdentity(0))), (DeviceIdentity(1), copy_of(&early, DeviceIdentity(1)))];
    let stale_before = devices[1].1.image.clone();
    let outcome = session.publish_user_change(&mut devices, change);
    println!(
        "Y2 session_stale ok={} err={:?} stale_disk_written={}",
        outcome.is_ok(),
        outcome.as_ref().err().map(|error| format!("{error:?}").chars().take(200).collect::<String>()),
        devices[1].1.image != stale_before
    );
    let after = pool_of(&devices, now.device_size_in_bytes);
    report_after("session_stale", &pool.parameters, &after);
}

/// 正常卸载时盘 1 换成同池盘 1 的旧快照。
#[test]
fn unmount_onto_a_stale_snapshot_of_device_one() {
    let mut pool = start_plain(WIDTH);
    let early = pool.image();
    pool.overwrite(OVERWRITE_BYTES).expect("覆盖写 1");
    pool.overwrite(OVERWRITE_BYTES).expect("覆盖写 2");
    let now = pool.image();
    let mut session = pool.session.take().expect("会话");
    let mut devices = vec![(DeviceIdentity(0), copy_of(&now, DeviceIdentity(0))), (DeviceIdentity(1), copy_of(&early, DeviceIdentity(1)))];
    let stale_before = devices[1].1.image.clone();
    let outcome = unmount(&pool.parameters, &mut devices, &mut session.allocator, &mut session.current, ShadowLedger::On);
    println!(
        "Y2 unmount_stale accepted={} err={:?} stale_disk_written={}",
        matches!(outcome, Ok(Unmounted::FloorRaisedToTheCurrentVersion(_))),
        outcome.as_ref().err().map(|error| format!("{error:?}").chars().take(200).collect::<String>()),
        devices[1].1.image != stale_before
    );
    let after = pool_of(&devices, now.device_size_in_bytes);
    report_after("unmount_stale", &pool.parameters, &after);
}

/// 会话里两块盘的盘体对调：盘表身份照旧是 [0, 1]，盘 0 的位置交盘 1 的盘体、盘 1 的位置交盘 0 的盘体。
#[test]
fn session_publishes_with_the_two_device_bodies_swapped() {
    let mut pool = start_plain(WIDTH);
    pool.overwrite(OVERWRITE_BYTES).expect("覆盖写 1");
    pool.overwrite(OVERWRITE_BYTES).expect("覆盖写 2");
    let now = pool.image();
    let write_time_seconds = pool.write_time_seconds + 1;
    let content = content_of(OVERWRITE_BYTES, write_time_seconds);
    let change = UserChange::Overwrite(FirstFile { content: &content, write_time_seconds });
    let session = pool.session.as_mut().expect("会话");
    let mut devices = vec![(DeviceIdentity(0), copy_of(&now, DeviceIdentity(1))), (DeviceIdentity(1), copy_of(&now, DeviceIdentity(0)))];
    let outcome = session.publish_user_change(&mut devices, change);
    println!(
        "Y2 session_swapped ok={} err={:?}",
        outcome.is_ok(),
        outcome.as_ref().err().map(|error| format!("{error:?}").chars().take(300).collect::<String>())
    );
    // 盘体放回各自的身份：盘 0 的盘体是交在身份 1 那一项里的那块。
    let after = MemoryPool {
        devices: [(DeviceIdentity(0), devices[1].1.image.clone()), (DeviceIdentity(1), devices[0].1.image.clone())].into_iter().collect(),
        device_size_in_bytes: now.device_size_in_bytes,
    };
    report_after("session_swapped", &pool.parameters, &after);
}

/// 对照：正常卸载时两块盘的盘体对调（挂着之后的入口比本盘设备号）。
#[test]
fn unmount_with_the_two_device_bodies_swapped() {
    let mut pool = start_plain(WIDTH);
    pool.overwrite(OVERWRITE_BYTES).expect("覆盖写 1");
    let now = pool.image();
    let mut session = pool.session.take().expect("会话");
    let mut devices = vec![(DeviceIdentity(0), copy_of(&now, DeviceIdentity(1))), (DeviceIdentity(1), copy_of(&now, DeviceIdentity(0)))];
    let outcome = unmount(&pool.parameters, &mut devices, &mut session.allocator, &mut session.current, ShadowLedger::On);
    println!(
        "Y2 unmount_swapped accepted={} err={:?}",
        outcome.is_ok(),
        outcome.as_ref().err().map(|error| format!("{error:?}").chars().take(300).collect::<String>())
    );
}

/// 会话里盘 1 只剩一槽自证（较新那一槽被写坏），照常发布之后的样子。
#[test]
fn session_publishes_with_the_newer_system_configuration_slot_of_device_one_corrupted() {
    let mut pool = start_plain(WIDTH);
    pool.overwrite(OVERWRITE_BYTES).expect("覆盖写 1");
    let now = pool.image();
    let spacing = u64::from(pool.parameters.geometry.fixed_structure_slot_spacing);
    let write_time_seconds = pool.write_time_seconds + 1;
    let content = content_of(OVERWRITE_BYTES, write_time_seconds);
    let change = UserChange::Overwrite(FirstFile { content: &content, write_time_seconds });
    let session = pool.session.as_mut().expect("会话");
    let mut devices = vec![(DeviceIdentity(0), copy_of(&now, DeviceIdentity(0))), (DeviceIdentity(1), copy_of(&now, DeviceIdentity(1)))];
    for slot in 0..2u64 {
        let mut bytes = vec![0u8; 512];
        devices[1].1.read_at(DeviceOffsetInBytes(slot * spacing), &mut bytes).expect("读");
        println!("Y2 one_slot slot{slot}_head={:02x?}", &bytes[..16]);
    }
    let mut slot_one = vec![0u8; 512];
    devices[1].1.read_at(DeviceOffsetInBytes(spacing), &mut slot_one).expect("读槽 1");
    slot_one[100] ^= 0xff;
    devices[1].1.write_at(DeviceOffsetInBytes(spacing), &slot_one, WriteDurability::Plain).expect("写坏槽 1");
    let outcome = session.publish_user_change(&mut devices, change);
    println!("Y2 session_one_slot ok={} err={:?}", outcome.is_ok(), outcome.as_ref().err().map(|error| format!("{error:?}").chars().take(200).collect::<String>()));
    let after = pool_of(&devices, now.device_size_in_bytes);
    report_after("session_one_slot", &pool.parameters, &after);
}

/// 对照：不经挂着之后的入口，直接拿（盘 0 此刻, 盘 1 旧快照）可写挂载：取号之前逐盘核的「落后且缺单元」那一支拒不拒。
#[test]
fn control_writable_mount_directly_on_the_stale_snapshot_of_device_one() {
    let mut pool = start_plain(WIDTH);
    let early = pool.image();
    pool.overwrite(OVERWRITE_BYTES).expect("覆盖写 1");
    pool.overwrite(OVERWRITE_BYTES).expect("覆盖写 2");
    let now = pool.image();
    let mut devices = vec![(DeviceIdentity(0), copy_of(&now, DeviceIdentity(0))), (DeviceIdentity(1), copy_of(&early, DeviceIdentity(1)))];
    match mount_writable(&pool.parameters, &mut devices) {
        Ok(mounted) => println!("Y2 control_stale_direct writable ok instance={}", mounted.output.instance.0),
        Err(error) => println!("Y2 control_stale_direct writable err={}", format!("{error:?}").chars().take(300).collect::<String>()),
    }
    let image = pool_of(&devices, now.device_size_in_bytes);
    let violations = checker_violations_on(&image);
    println!("Y2 control_stale_direct checker_violations={} first={:?}", violations.len(), violations.first());
}

/// 把旧快照那一格的用户动作放开扫：旧快照取自哪一刻（第一个文件之后 / 覆盖写 1 之后）、换的是哪块盘、之后经会话发几次（1..=3）或做一次正常卸载，
/// 看每一格之后可写挂载做不做得成、池级 checker 红几条；对照是不经挂着的入口、直接拿同样的盘可写挂载。
#[test]
fn stale_snapshot_substitution_swept_over_the_user_steps() {
    for stale_after_overwrites in [0usize, 1] {
        for stale_device in [DeviceIdentity(0), DeviceIdentity(1)] {
            for steps in ["session1", "session2", "session3", "unmount"] {
                let mut pool = start_plain(WIDTH);
                let mut early = pool.image();
                for index in 0..2 {
                    if index == stale_after_overwrites {
                        early = pool.image();
                    }
                    pool.overwrite(OVERWRITE_BYTES).expect("覆盖写");
                }
                let now = pool.image();
                let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = [DeviceIdentity(0), DeviceIdentity(1)]
                    .into_iter()
                    .map(|identity| (identity, if identity == stale_device { copy_of(&early, identity) } else { copy_of(&now, identity) }))
                    .collect();
                let mut control_devices = devices.iter().map(|(identity, device)| (*identity, duplicate(device, now.device_size_in_bytes))).collect::<Vec<_>>();
                let control = mount_writable(&pool.parameters, &mut control_devices).is_ok();
                let mut accepted = Vec::new();
                match steps {
                    "unmount" => {
                        let mut session = pool.session.take().expect("会话");
                        accepted.push(unmount(&pool.parameters, &mut devices, &mut session.allocator, &mut session.current, ShadowLedger::On).is_ok());
                    }
                    _ => {
                        let count: usize = steps.trim_start_matches("session").parse().expect("次数");
                        let session = pool.session.as_mut().expect("会话");
                        for step in 0..count {
                            let write_time_seconds = pool.write_time_seconds + 1 + step as u64;
                            let content = content_of(OVERWRITE_BYTES, write_time_seconds);
                            accepted.push(session.publish_user_change(&mut devices, UserChange::Overwrite(FirstFile { content: &content, write_time_seconds })).is_ok());
                        }
                    }
                }
                let after = pool_of(&devices, now.device_size_in_bytes);
                let violations = checker_violations_on(&after);
                let mut remount = devices.iter().map(|(identity, device)| (*identity, duplicate(device, now.device_size_in_bytes))).collect::<Vec<_>>();
                let writable = mount_writable(&pool.parameters, &mut remount);
                println!(
                    "Y2 sweep stale_after_overwrites={stale_after_overwrites} stale_device={} steps={steps} control_direct_writable_ok={control} entries_accepted={accepted:?} checker_after={} writable_after_ok={} writable_err={:?}",
                    stale_device.0,
                    violations.len(),
                    writable.is_ok(),
                    writable.as_ref().err().map(|error| format!("{error:?}").chars().take(120).collect::<String>())
                );
            }
        }
    }
}
