//! C322（取号那一步的屏障怎么放没有条款） 第二轮攻方腿（Opus）的模型 R：在仓里今天的实现（crates/，只读、不改）上做第二次挂载的取号。
//! 流：mkfs → 取号（实例 1）→ 暖机 → 第一个事务（与层 0 同一串调用，内存盘），然后当成第二次可写挂载：
//!   ① 取实例 2：今天的 acquire_instance（世代号写死成 2，transaction.rs 第 49 行），或 G1 / G2 都给的世代号 6；
//!   ② 补一道屏障（甲″）；
//!   ③ 借 publish_first_file 以实例 2 发 8 个单元，只留到它第一道屏障之前——恢复写行那次发布还没实现，这 8 个单元是它的替身：
//!      甲″ 下本实例的单元写排在那道屏障之后，这里取的是「那一段全落了、下一道屏障还没过」这个崩溃状态。
//! 然后读：每盘两槽、恢复择到的超级块（recovery::choose_superblock）、checker 择到的（image::chosen_superblocks）、根环、
//! 那 8 个单元头里的写序实例代号；算三种读法下的下一次取号；跑 checker 的 I-7.7。

use std::collections::BTreeSet;

use singlefs_checker::image::{chosen_superblocks, valid_roots};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{
    make_filesystem, MakeFilesystemParameters, INSTANCE_TABLE_SLOT, MKFS_INSTANCE_GENERATION,
    TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::recovery::{choose_superblock, PoolReader};
use singlefs_core::superblock::{FormatTimeGeometry, Superblock};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, warm_up, CommitStep, FirstFile, PoolWriter,
};
use singlefs_core::unit::{parse_data_unit, parse_index_node, parse_packed_unit};
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
use singlefs_harness::crash::{writes_and_segments, MemoryPool, RetainedWrite, SparseBlockDevice};
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::{RecordedOperationKind, RecordingBlockDevice, SharedStream};

/// 与 crates/singlefs-harness/tests/common/mod.rs 同参数（第一轮模型 A 同）。
const IMAGE_BYTES: u64 = 4 << 30;
const FILE_BYTES: usize = 3000;
const FILESYSTEM_IDENTIFIER: [u8; 16] = [
    0x5f, 0x53, 0x46, 0x53, 0x2d, 0x45, 0x31, 0x34, 0x32, 0x2d, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x00,
];
const FIXED_WRITE_TIME_SECONDS: u64 = 1_788_000_000;

fn parameters() -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: FormatTimeGeometry {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
        },
    }
}

fn fixed_geometry() -> FixedGeometry {
    FixedGeometry {
        fixed_structure_slot_spacing: 4096,
        journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
    }
}

fn file_content() -> Vec<u8> {
    (0..FILE_BYTES)
        .map(|byte_position| u8::try_from(byte_position % 251).expect("小于 256"))
        .collect()
}

/// 第二次挂载的取号怎么写超级块。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SecondAcquisition {
    /// 今天的 acquire_instance：世代号写死成 2。
    TodayAcquireInstance,
    /// 两盘择到的世代号都是 5：G1（全池最大 + 1）与 G2（每盘 + 1）在这一格都给 6。
    GenerationSixFromEitherRule,
}

/// 跑完整条流，返回施加到「替身发布第一道屏障之前」的镜像，与第二次挂载那一段（取号起）的写表。
fn build(second: SecondAcquisition) -> (MemoryPool, Vec<RetainedWrite>) {
    let parameters = parameters();
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
        Placement {
            slot: INSTANCE_TABLE_SLOT,
            span: 2,
        },
        Placement {
            slot: TREE_TABLE_GENESIS_SLOT,
            span: 1,
        },
    ]);
    let content = file_content();
    let second_mount_start;
    {
        let mut pool = PoolWriter {
            parameters: &parameters,
            devices: &mut devices,
        };
        let first_instance = acquire_instance(&mut pool, MKFS_INSTANCE_GENERATION).expect("取号");
        let warm = warm_up(&mut pool, &genesis.root, first_instance).expect("暖机");
        let first_file = FirstFile {
            content: &content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS,
        };
        publish_first_file(&mut pool, &mut allocator, &genesis.root, first_file, first_instance, &warm.last_record_bytes)
            .expect("第一个事务");
        second_mount_start = stream.operations().len();
        let second_instance = InstanceGeneration(first_instance.0 + 1);
        match second {
            SecondAcquisition::TodayAcquireInstance => {
                let acquired = acquire_instance(&mut pool, first_instance).expect("第二次取号");
                assert_eq!(acquired.0, second_instance.0, "今天的取号 = 上一个 + 1");
            }
            SecondAcquisition::GenerationSixFromEitherRule => pool
                .perform(CommitStep::RotateSuperblockSlots {
                    slot_generation: 6,
                    journal_tail: 3,
                    journal_instance: second_instance,
                })
                .expect("第二次取号"),
        }
        pool.perform(CommitStep::Barrier).expect("甲″ 的那道屏障");
        let stand_in = FirstFile {
            content: &content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS,
        };
        publish_first_file(&mut pool, &mut allocator, &genesis.root, stand_in, second_instance, &warm.last_record_bytes)
            .expect("第二次挂载第一次发布的替身");
    }
    let operations = stream.retained_operations();
    let is_barrier = |position: &usize| operations[*position].operation.kind == RecordedOperationKind::Barrier;
    let acquisition_barrier = (second_mount_start..operations.len()).find(is_barrier).expect("甲″ 的屏障");
    let cut = (acquisition_barrier + 1..operations.len()).find(is_barrier).expect("替身的第一道屏障");
    let mut image = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
    image.apply(&operations[..cut]);
    let (second_mount_writes, _) = writes_and_segments(&operations[second_mount_start..cut], &fixed_geometry());
    (image, second_mount_writes)
}

/// 每盘两槽：(槽世代号, 实例代号)；槽不自证记 (0, 0)。
fn slot_contents(image: &MemoryPool) -> Vec<[(u64, u32); 2]> {
    (0..2u32)
        .map(|device| {
            let read_slot = |offset: u64| {
                PoolReader::read(image, DeviceIdentity(device), DeviceOffsetInBytes(offset), 4096)
                    .and_then(|bytes| Superblock::parse_slot(&bytes))
                    .map_or((0, 0), |superblock| (superblock.slot_generation, superblock.journal_instance.0))
            };
            [read_slot(0), read_slot(4096)]
        })
        .collect()
}

/// 一次单元写落盘之后头里带着的写序实例代号（不经恢复代码，直接解头）。
fn unit_instance(write: &RetainedWrite) -> Option<u32> {
    match write.kind {
        StepKind::UnitWrite => Some(match write.bytes[6] {
            1 => parse_data_unit(&write.bytes).expect("码 1").write_order.instance.0,
            2 => parse_index_node(&write.bytes).expect("码 2").instance.0,
            3 => parse_packed_unit(&write.bytes).expect("码 3").write_order.instance.0,
            unit_class => panic!("没登记的单元类 {unit_class}"),
        }),
        StepKind::SuperblockSlot | StepKind::RootRecordFua | StepKind::JournalRecord | StepKind::Barrier => None,
    }
}

fn report(second: SecondAcquisition) {
    let (image, second_mount_writes) = build(second);
    let slots = slot_contents(&image);
    let recovery_choice = choose_superblock(&image).expect("择超级块");
    let checker_chosen: Vec<(u64, u32)> = chosen_superblocks(&image)
        .iter()
        .map(|(_, choice)| {
            let (view, _) = choice.as_ref().expect("两盘都有自证过的槽");
            (view.slot_generation, view.journal_instance)
        })
        .collect();
    let geometry = chosen_superblocks(&image)[0].1.as_ref().expect("盘 0 有自证过的槽").1;
    let root_instances: BTreeSet<u32> = valid_roots(&image, &geometry)
        .iter()
        .map(|(_, _, root)| u32::from(root.instance))
        .collect();
    let unit_instances: BTreeSet<u32> = second_mount_writes.iter().filter_map(unit_instance).collect();
    let unit_writes = second_mount_writes.iter().filter(|write| unit_instance(write).is_some()).count();
    let superblock_writes = second_mount_writes
        .iter()
        .filter(|write| matches!(write.kind, StepKind::SuperblockSlot))
        .count();
    let highest_root = root_instances.iter().copied().max().unwrap_or(0);
    let by_recovery_choice = recovery_choice.journal_instance.0.max(highest_root) + 1;
    let by_checker_chosen = checker_chosen
        .iter()
        .map(|(_, instance)| *instance)
        .max()
        .unwrap_or(0)
        .max(highest_root)
        + 1;
    let by_every_slot = slots
        .iter()
        .flat_map(|disk| disk.iter())
        .map(|(_, instance)| *instance)
        .max()
        .unwrap_or(0)
        .max(highest_root)
        + 1;
    let collides = |next: u32| unit_instances.contains(&next) || root_instances.contains(&next);
    let invariant_seven_seven: Vec<String> = check_pool_image(&image)
        .into_iter()
        .filter(|(name, _)| *name == "I-7.7")
        .map(|(_, verdict)| format!("{verdict:?}"))
        .collect();
    println!(
        "REAL second_acquisition={second:?} second_mount_superblock_writes={superblock_writes} second_mount_unit_writes={unit_writes} slots(generation,instance)={slots:?} recovery_choose_superblock=({}, {}) checker_chosen={checker_chosen:?} root_instances={root_instances:?} second_mount_unit_instances={unit_instances:?}",
        recovery_choice.slot_generation, recovery_choice.journal_instance.0
    );
    println!(
        "REAL second_acquisition={second:?} next_acquisition by_recovery_choice={by_recovery_choice} by_checker_chosen={by_checker_chosen} by_every_slot={by_every_slot} collides_with_units_or_roots by_recovery_choice={} by_checker_chosen={} by_every_slot={} checker_I-7.7={invariant_seven_seven:?}",
        collides(by_recovery_choice),
        collides(by_checker_chosen),
        collides(by_every_slot)
    );
}

fn main() {
    report(SecondAcquisition::TodayAcquireInstance);
    report(SecondAcquisition::GenerationSixFromEitherRule);
}
