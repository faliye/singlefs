//! 里程碑「第一个事务」步 2 的验收：分配一个数据单元、写一个小文件的数据，两盘各一份，读回自检；分配记录与记账增量在内存里。

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, TreeIdentifier,
};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::{
    BlockDevice, FileBackedBlockDevice, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::make_filesystem::{
    make_filesystem, MakeFilesystemParameters, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::pointer::{DataPointer, LocationEntry, PointerHead};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::unit::{build_data_unit, declared_length, DataUnitIdentity, WriteOrder};
use singlefs_format::{JOURNAL_RING_DEFAULT_BYTES, TREE_IDENTIFIER_EXTENT};
use singlefs_harness::{RecordedOperationKind, RecordingBlockDevice, SharedStream};

static IMAGE_COUNTER: AtomicU64 = AtomicU64::new(0);
const IMAGE_BYTES: u64 = 4 << 30;
/// E142 的第一个文件 3000 字节（`name=config file_bytes=3000`）。
const FILE_BYTES: usize = 3000;

fn image_path(tag: &str, device: u32) -> PathBuf {
    let sequence = IMAGE_COUNTER.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir().join(format!(
        "singlefs-step2-{tag}-{}-{sequence}-dev{device}.img",
        std::process::id()
    ))
}

fn parameters() -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: *b"singlefs-step2-x",
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    }
}

fn file_content() -> Vec<u8> {
    (0..FILE_BYTES)
        .map(|index| u8::try_from(index % 251).expect("小于 256"))
        .collect()
}

struct Pool {
    paths: Vec<PathBuf>,
    devices: Vec<(DeviceIdentity, RecordingBlockDevice<FileBackedBlockDevice>)>,
    stream: SharedStream,
    allocator: PoolAllocator,
}

fn mkfs_pool(tag: &str) -> Pool {
    let stream = SharedStream::new();
    let mut paths = Vec::new();
    let mut devices = Vec::new();
    for device_number in 0..2u32 {
        let path = image_path(tag, device_number);
        let file = FileBackedBlockDevice::create_image_file_exclusively(
            &path,
            IMAGE_BYTES,
            PhysicalBlockSizeInBytes(512),
        )
        .expect("建镜像");
        devices.push((
            DeviceIdentity(device_number),
            RecordingBlockDevice::with_shared_stream(
                DeviceIdentity(device_number),
                file,
                stream.clone(),
            ),
        ));
        paths.push(path);
    }
    make_filesystem(&parameters(), &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
    ]);
    allocator.mark_format_time_units(
        Placement {
            slot: INSTANCE_TABLE_SLOT,
            span: 2,
        },
        Placement {
            slot: TREE_TABLE_GENESIS_SLOT,
            span: 1,
        },
    );
    Pool {
        paths,
        devices,
        stream,
        allocator,
    }
}

fn read(
    device: &RecordingBlockDevice<FileBackedBlockDevice>,
    offset: u64,
    length: usize,
) -> Vec<u8> {
    let mut buffer = vec![0u8; length];
    device
        .read_at(DeviceOffsetInBytes(offset), &mut buffer)
        .expect("读");
    buffer
}

/// 步 2 的写：分配落点、装单元、两盘各写一份；返回落点、单元字节与指向它的指针。
fn write_first_file(pool: &mut Pool) -> (Placement, Vec<u8>, DataPointer) {
    let generation = CheckpointTxg(3);
    let placement = pool
        .allocator
        .allocate_user_data(generation)
        .expect("有空槽");
    let identity = DataUnitIdentity {
        tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
        object: 1,
        object_birth: generation,
        anchor_offset: 0,
    };
    let write_order = WriteOrder {
        instance: InstanceGeneration(1),
        transaction: 1,
    };
    let unit = build_data_unit(
        identity,
        generation,
        &parameters().filesystem_identifier,
        write_order,
        &file_content(),
    );
    for (_, device) in pool.devices.iter_mut() {
        device
            .write_at(
                placement.slot.to_device_offset(),
                &unit,
                WriteDurability::Plain,
            )
            .expect("写单元");
    }
    let checksum = crc32_castagnoli(&unit);
    let pointer = DataPointer {
        head: PointerHead {
            birth_tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
            birth_txg: generation,
        },
        locations: [
            LocationEntry {
                device: DeviceIdentity(0),
                slot: placement.slot,
                unit_checksum: checksum,
            },
            LocationEntry {
                device: DeviceIdentity(1),
                slot: placement.slot,
                unit_checksum: checksum,
            },
        ],
        write_order,
    };
    (placement, unit, pointer)
}

fn remove_images(pool: &Pool) {
    for path in &pool.paths {
        std::fs::remove_file(path).expect("清理镜像");
    }
}

#[test]
fn data_unit_reads_back_from_both_devices_with_content_then_zeroes() {
    let mut pool = mkfs_pool("readback");
    let (placement, unit, pointer) = write_first_file(&mut pool);
    assert_eq!(placement.slot.0, 50180, "t1@50180");
    for (_, device) in &pool.devices {
        let read_back = read(device, placement.slot.to_device_offset().0, 32768);
        assert_eq!(read_back, unit);
        assert_eq!(
            singlefs_checker::check_unit(&read_back).expect("头自检过、载荷 CRC 对"),
            1
        );
        assert_eq!(usize::from(declared_length(&read_back)), FILE_BYTES);
        assert_eq!(
            &read_back[134..134 + FILE_BYTES],
            &file_content()[..],
            "前 N 字节等于文件内容"
        );
        assert!(
            read_back[134 + FILE_BYTES..].iter().all(|byte| *byte == 0),
            "之后全 0"
        );
        assert_eq!(
            read_back[43..51],
            TREE_IDENTIFIER_EXTENT.to_le_bytes(),
            "五元组树 ID = extent 树 11"
        );
        assert_eq!(&read_back[91..95], &1u32.to_le_bytes(), "写序实例代号 1");
    }
    assert_eq!(
        singlefs_checker::crc32_castagnoli_bitwise(&unit),
        pointer.locations[0].unit_checksum,
        "位置条目里的整单元校验和"
    );
    remove_images(&pool);
}

#[test]
fn corrupting_payload_or_header_is_caught_by_the_matching_checksum() {
    let mut pool = mkfs_pool("corrupt");
    let (_, unit, _) = write_first_file(&mut pool);
    let mut payload_damaged = unit.clone();
    payload_damaged[20000] ^= 0x01;
    assert_eq!(
        singlefs_checker::check_unit(&payload_damaged).unwrap_err(),
        singlefs_checker::Verdict::ChecksumMismatch,
        "载荷 CRC 判红"
    );
    let mut padding_damaged = unit.clone();
    padding_damaged[32767] ^= 0x01;
    assert_eq!(
        singlefs_checker::check_unit(&padding_damaged).unwrap_err(),
        singlefs_checker::Verdict::ChecksumMismatch,
        "补齐区也在载荷 CRC 内（I-2.3）"
    );
    let mut header_damaged = unit.clone();
    header_damaged[60] ^= 0x01;
    assert_eq!(
        singlefs_checker::check_unit(&header_damaged).unwrap_err(),
        singlefs_checker::Verdict::ChecksumMismatch,
        "头校验和判红"
    );
    let mut flags_damaged = unit;
    flags_damaged[7] = 0x01;
    assert_eq!(
        singlefs_checker::check_unit(&flags_damaged).unwrap_err(),
        singlefs_checker::Verdict::NonZeroFlags,
        "flags 非 0 拒收"
    );
    remove_images(&pool);
}

#[test]
fn exactly_two_new_allocation_records_whose_slot_matches_the_recorded_write_offsets() {
    let mut pool = mkfs_pool("records");
    let (placement, _, _) = write_first_file(&mut pool);
    let all_records = pool.allocator.records();
    assert_eq!(
        all_records.len(),
        6,
        "mkfs 两个单元各两条（分配代 0）+ 这一步两条"
    );
    let records: Vec<_> = all_records
        .iter()
        .filter(|record| record.generation == CheckpointTxg(3))
        .collect();
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].device, DeviceIdentity(0));
    assert_eq!(records[1].device, DeviceIdentity(1));
    for record in records {
        assert_eq!(record.slot, placement.slot);
        assert_eq!(record.span_slots, 2);
        assert_eq!(record.generation, CheckpointTxg(3));
        assert_eq!(record.to_bytes().len(), 20);
    }
    let unit_writes: Vec<u64> = pool
        .stream
        .operations()
        .iter()
        .filter(|operation| {
            operation.kind == RecordedOperationKind::Write && operation.offset.0 >= 50180 * 16384
        })
        .map(|operation| operation.offset.0)
        .collect();
    assert_eq!(
        unit_writes,
        vec![
            placement.slot.to_device_offset().0,
            placement.slot.to_device_offset().0
        ],
        "两盘各一次写，偏移 = 槽号 × 16384"
    );
    assert_eq!(
        pool.allocator.policy_mismatches, 0,
        "C146 ② 的运行时计数第一个事务恒 0"
    );
    assert_eq!(pool.allocator.devices[0].allocated_slots(), 5);
    assert_eq!(pool.allocator.devices[0].free_runs(), 2);
    remove_images(&pool);
}
