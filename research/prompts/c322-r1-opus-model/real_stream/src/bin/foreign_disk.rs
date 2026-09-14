//! C322（取号那一步的屏障怎么放没有条款） 第一轮反推腿，模型 A 附带的一格（J3）：
//! 把池 A 的盘 1 换成另一个池（fsid 不同、同样 mkfs → 第一次可写挂载取实例 1 → 暖机 → 第一个事务）的盘 1，
//! 看 checker 的 I-7.7 与恢复各判什么。每个池都是第一次挂载取号得 1，所以两盘实例代号相等。只读仓里的 crate。

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::DeviceIdentity;
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{
    make_filesystem, MakeFilesystemParameters, INSTANCE_TABLE_SLOT, MKFS_INSTANCE_GENERATION,
    TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::superblock::FormatTimeGeometry;
use singlefs_core::transaction::{acquire_instance, publish_first_file, warm_up, FirstFile, PoolWriter};
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
use singlefs_harness::crash::{MemoryPool, SparseBlockDevice};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

const IMAGE_BYTES: u64 = 4 << 30;
const FIXED_WRITE_TIME_SECONDS: u64 = 1_788_000_000;
const HOME_FILESYSTEM_IDENTIFIER: [u8; 16] = [
    0x5f, 0x53, 0x46, 0x53, 0x2d, 0x45, 0x31, 0x34, 0x32, 0x2d, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x00,
];
const FOREIGN_FILESYSTEM_IDENTIFIER: [u8; 16] = [
    0x5f, 0x53, 0x46, 0x53, 0x2d, 0x46, 0x4f, 0x52, 0x45, 0x49, 0x47, 0x4e, 0x2d, 0x30, 0x32, 0x00,
];

/// mkfs → 取号 → 暖机 → 第一个事务，全部施加到内存镜像上（与 tests/common 的 build_pool 同一串调用）。
fn full_image(filesystem_identifier: [u8; 16]) -> MemoryPool {
    let parameters = MakeFilesystemParameters {
        filesystem_identifier,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: FormatTimeGeometry {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
        },
    };
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = (0..2u32)
        .map(|device_number| {
            (
                DeviceIdentity(device_number),
                RecordingBlockDevice::with_shared_stream(
                    DeviceIdentity(device_number),
                    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                    stream.clone(),
                ),
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
    ]);
    allocator.mark_format_time_units(&[
        Placement { slot: INSTANCE_TABLE_SLOT, span: 2 },
        Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 },
    ]);
    let content: Vec<u8> = (0..3000usize)
        .map(|byte_position| u8::try_from(byte_position % 251).expect("小于 256"))
        .collect();
    {
        let mut pool = PoolWriter { parameters: &parameters, devices: &mut devices };
        let instance = acquire_instance(&mut pool, MKFS_INSTANCE_GENERATION).expect("取号");
        let warm_up_output = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
        publish_first_file(
            &mut pool,
            &mut allocator,
            &genesis.root,
            FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS },
            instance,
            &warm_up_output.last_record_bytes,
        )
        .expect("第一个事务");
    }
    let mut image = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
    image.apply(&stream.retained_operations());
    image
}

fn describe_recovery(outcome: &RecoveryOutcome) -> String {
    match outcome {
        RecoveryOutcome::NoFile { root } => format!("NoFile root={root:?}"),
        RecoveryOutcome::FileRead { root, content } => format!("FileRead root={root:?} bytes={}", content.len()),
        RecoveryOutcome::Failed { root, failure } => format!("Failed root={root:?} failure={failure:?}"),
    }
}

fn main() {
    let home = full_image(HOME_FILESYSTEM_IDENTIFIER);
    let foreign = full_image(FOREIGN_FILESYSTEM_IDENTIFIER);
    let mut spliced = home.clone();
    spliced
        .devices
        .insert(DeviceIdentity(1), foreign.devices[&DeviceIdentity(1)].clone());
    for (case, pool) in [("home_pool_untouched", &home), ("disk1_replaced_by_foreign_pool_disk1", &spliced)] {
        let verdicts = check_pool_image(pool);
        let instance_verdict = verdicts
            .iter()
            .find(|(invariant, _)| *invariant == "I-7.7")
            .map(|(_, verdict)| verdict.clone());
        let violated: Vec<String> = verdicts
            .iter()
            .filter_map(|(invariant, verdict)| match verdict {
                InvariantVerdict::Violated(detail) => Some(format!("{invariant}: {detail}")),
                InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
            })
            .collect();
        let recovery = recover(pool, JournalPolicy::Consult).outcome;
        println!(
            "FOREIGN case={case} I-7.7={instance_verdict:?} violated=[{}] recovery={}",
            violated.join(" | "),
            describe_recovery(&recovery)
        );
    }
}
