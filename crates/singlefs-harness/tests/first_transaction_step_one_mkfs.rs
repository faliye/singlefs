//! 里程碑「第一个事务」步 1 的验收：mkfs 写出的字节由 checker（独立实现）判，录制流的段序列与登记表逐字相同。

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::{
    BlockDevice, FileBackedBlockDevice, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::make_filesystem::{
    make_filesystem, MakeFilesystemError, MakeFilesystemOutput, MakeFilesystemParameters,
};
use singlefs_core::root_ring::{slot_offset, RootRingSlot};
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_format::{JOURNAL_RING_DEFAULT_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES};
use singlefs_harness::segments::{
    segment_kinds_text, segment_sizes_text, split_into_segments, FixedGeometry,
};
use singlefs_harness::{
    check_stream_integrity, RecordingBlockDevice, SharedStream, StreamIntegrity,
};

static IMAGE_COUNTER: AtomicU64 = AtomicU64::new(0);
/// 4 GiB 稀疏镜像：环 768 MiB ≤ 容量 / 4（D23 已定项 19）。
const IMAGE_BYTES: u64 = 4 << 30;

fn image_path(tag: &str, device: u32) -> PathBuf {
    let sequence = IMAGE_COUNTER.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir().join(format!(
        "singlefs-step1-{tag}-{}-{sequence}-dev{device}.img",
        std::process::id()
    ))
}

fn parameters() -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: *b"singlefs-step1-x",
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(512),
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
        },
    }
}

struct Pool {
    paths: Vec<PathBuf>,
    devices: Vec<(DeviceIdentity, RecordingBlockDevice<FileBackedBlockDevice>)>,
    stream: SharedStream,
    output: MakeFilesystemOutput,
}

fn run_mkfs(tag: &str) -> Pool {
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
    let output = make_filesystem(&parameters(), &mut devices).expect("mkfs");
    Pool {
        paths,
        devices,
        stream,
        output,
    }
}

fn remove_images(pool: &Pool) {
    for path in &pool.paths {
        std::fs::remove_file(path).expect("清理镜像");
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

#[test]
fn same_parameters_twice_give_byte_identical_images() {
    let first = run_mkfs("twice-a");
    let second = run_mkfs("twice-b");
    for device_number in 0..2 {
        let touched = [
            0u64,
            4096,
            1 << 20,
            4 << 20,
            7 << 20,
            50176 * 16384,
            50178 * 16384,
        ];
        for offset in touched {
            assert_eq!(
                read(&first.devices[device_number].1, offset, 32768),
                read(&second.devices[device_number].1, offset, 32768),
                "偏移 {offset} 两次 mkfs 不同"
            );
        }
    }
    assert_eq!(first.output, second.output);
    assert_eq!(
        first.stream.render(),
        second.stream.render(),
        "录制流也逐字相同"
    );
    remove_images(&first);
    remove_images(&second);
}

#[test]
fn checker_reads_the_same_generation_zero_root_from_all_three_regions_and_isolates_a_torn_slot() {
    let mut pool = run_mkfs("roots");
    let expected_fsid = parameters().filesystem_identifier;
    let spacing = parameters().geometry.fixed_structure_slot_spacing;
    let mut views = Vec::new();
    for region in 0..3u64 {
        let device_number = parameters().region_devices[usize::try_from(region).expect("区域号")].0;
        let offset = slot_offset(RootRingSlot { region, slot: 0 }, spacing).0;
        let slot = read(
            &pool.devices[usize::try_from(device_number).expect("设备号")].1,
            offset,
            512,
        );
        let view = singlefs_checker::check_root_slot(&slot, &expected_fsid)
            .expect("区域槽 0 的根记录合法");
        assert_eq!(view.checkpoint_txg, 0);
        assert_eq!(view.instance, 0);
        assert_eq!(view.tree_identifier_watermark, 11);
        views.push(view);
    }
    assert_eq!(views[0].record_bytes, views[1].record_bytes);
    assert_eq!(views[1].record_bytes, views[2].record_bytes);

    let torn_offset = slot_offset(RootRingSlot { region: 1, slot: 0 }, spacing).0;
    let mut torn = read(&pool.devices[1].1, torn_offset, 512);
    torn[400] ^= 0x01;
    pool.devices[1]
        .1
        .write_at(
            DeviceOffsetInBytes(torn_offset),
            &torn,
            WriteDurability::Plain,
        )
        .expect("写坏一字节");
    assert_eq!(
        singlefs_checker::check_root_slot(&torn, &expected_fsid).unwrap_err(),
        singlefs_checker::Verdict::ChecksumMismatch
    );
    for region in [0u64, 2] {
        let offset = slot_offset(RootRingSlot { region, slot: 0 }, spacing).0;
        let slot = read(&pool.devices[0].1, offset, 512);
        assert!(
            singlefs_checker::check_root_slot(&slot, &expected_fsid).is_ok(),
            "别的区域不受影响"
        );
    }
    remove_images(&pool);
}

#[test]
fn both_system_configuration_slots_verify_with_the_same_generation_and_a_corrupt_slot_loses_the_choice(
) {
    let mut pool = run_mkfs("superblock");
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    for (device_number, (_, device)) in pool.devices.iter().enumerate() {
        let slot_zero = read(device, 0, slot_bytes);
        let slot_one = read(device, 4096, slot_bytes);
        let view_zero =
            singlefs_checker::check_system_configuration_slot(&slot_zero).expect("槽 0");
        let view_one = singlefs_checker::check_system_configuration_slot(&slot_one).expect("槽 1");
        assert_eq!(view_zero.slot_generation, 1);
        assert_eq!(view_one.slot_generation, 1);
        assert_eq!(
            view_zero.this_device,
            u32::try_from(device_number).expect("设备号")
        );
        assert_eq!(view_zero.device_count, 2);
        assert_eq!(view_zero.declared_physical_block_size, 512);
        assert_eq!(view_zero.journal_instance, 0);
        assert_eq!(view_zero.journal_tail, 0);
        assert_eq!(&view_zero.filesystem_identifier, b"singlefs-step1-x");
    }
    let mut bad = read(&pool.devices[0].1, 0, slot_bytes);
    bad[3000] ^= 0x80;
    pool.devices[0]
        .1
        .write_at(DeviceOffsetInBytes(0), &bad, WriteDurability::Plain)
        .expect("写坏槽 0");
    let slot_zero = read(&pool.devices[0].1, 0, slot_bytes);
    let slot_one = read(&pool.devices[0].1, 4096, slot_bytes);
    let (chosen, _) = singlefs_checker::choose_system_configuration(&[&slot_zero, &slot_one])
        .expect("还有一槽可择");
    assert_eq!(chosen, 1, "挂载选没坏的那一槽");
    remove_images(&pool);
}

#[test]
fn units_named_by_the_root_pass_their_own_header_checks() {
    let pool = run_mkfs("units");
    let root = &pool.output.root;
    for (pointer, expected_class) in [(&root.instance_table, 3u8), (&root.tree_table, 2u8)] {
        for location in &pointer.locations {
            let length = if expected_class == 3 { 32768 } else { 16384 };
            let unit = read(
                &pool.devices[usize::try_from(location.device.0).expect("设备号")].1,
                location.slot.0 * 16384,
                length,
            );
            assert_eq!(
                singlefs_checker::check_unit(&unit).expect("头自检过"),
                expected_class
            );
            assert_eq!(
                singlefs_checker::crc32_castagnoli_bitwise(&unit),
                location.unit_checksum,
                "位置条目里的整单元校验和"
            );
        }
    }
    assert_eq!(
        pool.output.instance_table_unit[136], 1,
        "实例表第一条是 kind 1 链指针记录"
    );
    remove_images(&pool);
}

#[test]
fn recorded_stream_matches_the_registered_mkfs_segment_sequence() {
    let pool = run_mkfs("segments");
    let operations = pool.stream.operations();
    let geometry = FixedGeometry {
        fixed_structure_slot_spacing: 4096,
        journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
    };
    let segments = split_into_segments(&operations, &geometry);
    assert_eq!(
        segment_sizes_text(&segments),
        "4+1+1+1+4",
        "layout/01-first-txn.md 八 mkfs 那一行"
    );
    assert_eq!(
        segment_kinds_text(&segments),
        "[unit_write×4,barrier]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[superblock_slot×4,barrier]"
    );
    assert_eq!(operations.len(), 13, "mkfs 13 次操作（含两道池屏障）");
    assert_eq!(
        check_stream_integrity(&pool.stream.render()),
        StreamIntegrity::Consistent { operations: 13 }
    );
    remove_images(&pool);
}

/// 第一版两块盘时根环区域归属写死盘 0 / 盘 1 / 盘 0（D2（RAID 条带策略） 已定项 7：不是 mkfs 参数），参数给别的归属就在任何写之前拒绝
/// （m2-emptypool-nonempty-r1 云端攻方腿 Z3-B）：[0, 1, 1] 与 [1, 0, 0] 各一次 ⇒ 返回 `RegionDevicesNotTheFirstVersionLayout`，
/// 录制流一步没有、两块盘的系统配置槽 0 与三个区域的根槽 0 全是 0。
#[test]
fn region_layout_other_than_zero_one_zero_is_refused_before_any_write() {
    for (tag, layout) in [
        (
            "layout-011",
            [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(1)],
        ),
        (
            "layout-100",
            [DeviceIdentity(1), DeviceIdentity(0), DeviceIdentity(0)],
        ),
    ] {
        let stream = SharedStream::new();
        let mut devices = Vec::new();
        let mut paths = Vec::new();
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
        let mut layout_parameters = parameters();
        layout_parameters.region_devices = layout;
        let result = make_filesystem(&layout_parameters, &mut devices);
        assert!(
            matches!(
                result,
                Err(MakeFilesystemError::RegionDevicesNotTheFirstVersionLayout { region_devices })
                    if region_devices == layout
            ),
            "{layout:?} 不是第一版写死的 0 / 1 / 0：{:?}",
            result.as_ref().err()
        );
        assert!(stream.operations().is_empty(), "拒绝之前一个字节都没写");
        let spacing = SystemImmutableSizes::slot_spacing_for(512);
        for (identity, device) in &devices {
            assert!(
                read(
                    device,
                    0,
                    usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096")
                )
                .iter()
                .all(|byte| *byte == 0),
                "盘 {identity:?} 的系统配置槽 0 全零"
            );
            for region in 0..3 {
                let offset = slot_offset(RootRingSlot { region, slot: 0 }, spacing);
                assert!(
                    read(device, offset.0, 512).iter().all(|byte| *byte == 0),
                    "盘 {identity:?} 上区域 {region} 的根槽 0 全零"
                );
            }
        }
        drop(devices);
        for path in paths {
            std::fs::remove_file(path).expect("清理镜像");
        }
    }
}

#[test]
fn ring_larger_than_a_quarter_of_the_device_is_refused_before_any_write() {
    let stream = SharedStream::new();
    let mut devices = Vec::new();
    let mut paths = Vec::new();
    for device_number in 0..2u32 {
        let path = image_path("small", device_number);
        let file = FileBackedBlockDevice::create_image_file_exclusively(
            &path,
            1 << 30,
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
    let result = make_filesystem(&parameters(), &mut devices);
    assert!(
        matches!(
            result,
            Err(MakeFilesystemError::JournalRingTooLargeForDevice { .. })
        ),
        "1 GiB 镜像装不下 768 MiB 的环"
    );
    assert!(stream.operations().is_empty(), "拒绝之前一个字节都没写");
    for path in paths {
        std::fs::remove_file(path).expect("清理镜像");
    }
}
