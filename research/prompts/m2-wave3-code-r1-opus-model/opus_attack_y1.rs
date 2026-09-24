//! m2-wave3-code-r1 云端攻方腿（Opus）Y1 的驱动：在第一个文件版本那次发布里崩溃（录制流的每一个前缀），
//! 重开可写挂载，跑池级 checker；再把挂载之后用户能做的几步放开扫。只打印，不断言；副本上跑，不进仓。

mod common;

use common::{parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::journal::back_chain_of;
use singlefs_core::make_filesystem::{make_filesystem, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT};
use singlefs_core::mount::{mount_rollback, mount_writable, Mounted, RollbackTarget, ShadowLedger};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_without_units, warm_up, FirstFile, PoolVersion,
    PoolWriter, ZeroUnitPublishPlan,
};
use singlefs_harness::crash::{MemoryPool, SparseBlockDevice};
use singlefs_harness::{RecordedOperationKind, RecordingBlockDevice, SharedStream};

pub type Dev = RecordingBlockDevice<SparseBlockDevice>;

pub fn devices_of(image: &MemoryPool, stream: &SharedStream) -> Vec<(DeviceIdentity, Dev)> {
    image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device = SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
            device.image = sparse.clone();
            (*identity, RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()))
        })
        .collect()
}

pub fn image_of(devices: &[(DeviceIdentity, Dev)]) -> MemoryPool {
    MemoryPool {
        devices: devices.iter().map(|(i, d)| (*i, d.inner().image.clone())).collect(),
        device_size_in_bytes: IMAGE_BYTES,
    }
}

pub fn violations(image: &MemoryPool) -> Vec<String> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(name, verdict)| match verdict {
            InvariantVerdict::Violated(text) => Some(format!("{name}: {text:?}")),
            _ => None,
        })
        .collect()
}

pub fn memory_of(stream: &SharedStream, upto: usize) -> MemoryPool {
    let mut pool = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
    pool.apply(&stream.retained_operations()[..upto]);
    pool
}

pub fn kind_name(stream: &SharedStream, index: usize) -> String {
    let op = &stream.retained_operations()[index].operation;
    let kind = match op.kind {
        RecordedOperationKind::Barrier => "barrier",
        RecordedOperationKind::WriteZeroes => "zeroes",
        RecordedOperationKind::Write => "write",
        RecordedOperationKind::WriteForceUnitAccess => "fua",
    };
    format!("{kind}@dev{}+{}", op.device.0, op.offset.0)
}

pub fn content(length: usize, seed: usize) -> Vec<u8> {
    (0..length).map(|i| u8::try_from((i * 7 + seed) % 253).unwrap()).collect()
}

/// mkfs → 取号 → 暖机 → 第一个文件版本（同一个进程）；交回流、第一个文件版本那次发布在流里的起止。
pub fn mkfs_stream_first_file() -> (SharedStream, usize, usize) {
    let stream = SharedStream::retaining_contents();
    let mut devices = devices_of(&MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES), &stream);
    let params = parameters();
    let genesis = make_filesystem(&params, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
    ]);
    allocator.mark_format_time_units(
        Placement { slot: INSTANCE_TABLE_SLOT, span: 2 },
        Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 },
    );
    let mut pool = PoolWriter::new(&params, &mut devices);
    let instance = acquire_instance(&mut pool).expect("取号");
    let warm = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
    let start = stream.operation_count();
    publish_first_file(
        &mut pool,
        &mut allocator,
        warm.roots.last().unwrap(),
        FirstFile { content: &content(3000, 1), write_time_seconds: FIXED_WRITE_TIME_SECONDS },
        instance,
        &warm.last_record_bytes,
    )
    .expect("第一个文件版本");
    let end = stream.operation_count();
    (stream, start, end)
}

/// 在上面那条流跑完之后（(1,3) 带文件、水位 19）关掉重开、回退到暖机根 (1,2)，在那个会话里再发第一个文件版本。
/// 交回流、回退之后第一个文件版本那次发布在流里的起止。
pub fn rollback_stream_first_file() -> (SharedStream, usize, usize) {
    let (stream, _, _) = mkfs_stream_first_file();
    let image = memory_of(&stream, stream.operation_count());
    let mut devices = devices_of(&image, &stream);
    let rolled = mount_rollback(
        &parameters(),
        &mut devices,
        RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(2) },
        ShadowLedger::On,
    )
    .expect("回退到暖机根");
    let PoolVersion::WithoutFile(version) = &rolled.current else { panic!("树表 0 条") };
    let mut allocator = rolled.allocator.clone();
    let params = parameters();
    let mut writer = PoolWriter::new(&params, &mut devices);
    let start = stream.operation_count();
    publish_first_file(
        &mut writer,
        &mut allocator,
        &version.root,
        FirstFile { content: &content(3300, 17), write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120 },
        rolled.output.instance,
        &version.record_bytes,
    )
    .expect("回退之后第一个文件版本");
    let end = stream.operation_count();
    (stream, start, end)
}

/// 挂载之后的用户动作：0 = 什么都不做；1..=3 = 连推 n 次零单元发布；4 = 再关掉重开一次可写挂载；
/// 5 = 发第一个文件版本（用挂载交回的那一版）。
pub fn after_mount(devices: &mut Vec<(DeviceIdentity, Dev)>, mounted: &Mounted, action: u32) -> String {
    let params = parameters();
    match action {
        0 => "none".into(),
        1..=3 => {
            let PoolVersion::WithoutFile(mut version) = mounted.current.clone() else { return "n/a(with file)".into() };
            for _ in 0..action {
                let mut writer = PoolWriter::new(&params, devices.as_mut_slice());
                version = publish_without_units(
                    &mut writer,
                    &version.root,
                    ZeroUnitPublishPlan {
                        txg: CheckpointTxg(version.root.checkpoint_txg.0 + 1),
                        counter: version.record.counter + 1,
                        instance: mounted.output.instance,
                        back_chain: back_chain_of(&version.record_bytes),
                        rollback_floor: version.root.rollback_floor,
                        tree_identifier_watermark: version.root.tree_identifier_watermark,
                    },
                )
                .expect("零单元发布");
            }
            format!("{action} zero-unit publishes")
        }
        4 => {
            match mount_writable(&params, devices) {
                Ok(_) => "remount ok".into(),
                Err(e) => format!("remount err {e:?}"),
            }
        }
        _ => {
            let PoolVersion::WithoutFile(version) = &mounted.current else { return "n/a(with file)".into() };
            let mut allocator = mounted.allocator.clone();
            let mut writer = PoolWriter::new(&params, devices.as_mut_slice());
            match publish_first_file(
                &mut writer,
                &mut allocator,
                &version.root,
                FirstFile { content: &content(2000, 5), write_time_seconds: FIXED_WRITE_TIME_SECONDS + 999 },
                mounted.output.instance,
                &version.record_bytes,
            ) {
                Ok(out) => format!("first file ok trees {:?} wm {}", out.tree_identifiers.in_issue_order().map(|t| t.0), out.root.tree_identifier_watermark),
                Err(e) => format!("first file err {e:?}"),
            }
        }
    }
}

fn sweep(name: &str, stream: &SharedStream, start: usize, end: usize) {
    println!("=== {name}: publish ops [{start}, {end}) ===");
    for k in start..=end {
        for action in 0..=5u32 {
            let crash = memory_of(stream, k);
            let side = SharedStream::retaining_contents();
            let mut devices = devices_of(&crash, &side);
            let mounted = match mount_writable(&parameters(), &mut devices) {
                Ok(m) => m,
                Err(e) => {
                    if action == 0 {
                        println!("k={k} last={} mount err {e:?}", if k > start { kind_name(stream, k - 1) } else { "-".into() });
                    }
                    continue;
                }
            };
            let root = mounted.current.root().clone();
            let what = after_mount(&mut devices, &mounted, action);
            let image = image_of(&devices);
            let v = violations(&image);
            println!(
                "k={k} last={} action={action}({what}) mounted root=({},{}) wm={} tree_table_empty={} violations={}: {:?}",
                if k > start { kind_name(stream, k - 1) } else { "-".into() },
                root.instance.0,
                root.checkpoint_txg.0,
                root.tree_identifier_watermark,
                matches!(mounted.current, PoolVersion::WithoutFile(_)),
                v.len(),
                v
            );
        }
    }
}

#[test]
fn y1_crash_inside_first_file_publish_then_mount_writable_mkfs_stream() {
    let (stream, start, end) = mkfs_stream_first_file();
    sweep("mkfs stream", &stream, start, end);
}

#[test]
fn y1_crash_inside_first_file_publish_then_mount_writable_after_rollback() {
    let (stream, start, end) = rollback_stream_first_file();
    sweep("after rollback to warm-up root", &stream, start, end);
}
