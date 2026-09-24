//! 里程碑「第一个事务」步 1 的验收：mkfs 写出的字节由 checker（独立实现）判，录制流的段序列与登记表逐字相同。

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::block_device::{
    BlockDevice, FileBackedBlockDevice, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::make_filesystem::{
    make_filesystem, MakeFilesystemError, MakeFilesystemOutput, MakeFilesystemParameters,
};
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::{choose_root, choose_system_configuration, readable_roots};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{
    region_length_in_bytes, region_start, slot_offset, RootRingSlot, RootRingSlotsPerRegion,
};
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_format::{
    JOURNAL_RING_DEFAULT_BYTES, JOURNAL_RING_START_SLOT, ROOT_RING_REGIONS, SLOT_BYTES,
    SYSTEM_CONFIGURATION_SLOT_BYTES,
};
use singlefs_harness::scenario::run_first_transaction;
use singlefs_harness::segments::{
    segment_kinds_text, segment_sizes_text, split_into_segments, FixedGeometry,
};
use singlefs_harness::{
    check_stream_integrity, RecordedOperationKind, RecordingBlockDevice, SharedStream,
    StreamIntegrity,
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
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
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
    let mut pool = run_mkfs("system_configuration");
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
        root_ring_slots_per_region: parameters().geometry.root_ring_slots_per_region,
    };
    let segments = split_into_segments(&operations, &geometry);
    assert_eq!(
        segment_sizes_text(&segments),
        "12+1+1+1+4",
        "layout/01-first-txn.md 八 mkfs 那一行"
    );
    assert_eq!(
        segment_kinds_text(&segments),
        "[zero_fill×8,unit_write×4,barrier]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[system_configuration_slot×4,barrier]"
    );
    assert_eq!(
        operations.len(),
        21,
        "mkfs 21 次操作（每盘 3 段根环 + 1 段环共八次清零、四个单元写、三次根 FUA、四次系统配置槽写、两道池屏障）"
    );
    assert_eq!(
        check_stream_integrity(&pool.stream.render()),
        StreamIntegrity::Consistent { operations: 21 }
    );
    remove_images(&pool);
}

/// 并行线四的验收第一条：mkfs 之后抽样读 journal 环全 0，mkfs 交给设备的写字节数
/// = 116 224 + 2 × 768 MiB + 6 × 32 KiB（末一项是 2026-09-22 连根环一起清之后加的：两盘各三个区域）。
///
/// 镜像先在环里撒上非 0（稀疏镜像本来就读回全 0，不先弄脏就分不出「清过」与「从来没写过」，
/// `test-discipline.md`「读不到 ≠ 读到 0」），再跑 mkfs：环里抽样点全 0，说明清零真的发生过；
/// 字节数从录制流数（每条写记的是它自己的长度），清零一条记它自己那一段的长度。
#[test]
fn mkfs_zeroes_the_whole_journal_ring_and_writes_the_registered_number_of_bytes() {
    const RING_START: u64 = JOURNAL_RING_START_SLOT * SLOT_BYTES;
    const MKFS_METADATA_BYTES: u64 = 116_224;

    let stream = SharedStream::new();
    let mut paths = Vec::new();
    let mut devices = Vec::new();
    for device_number in 0..2u32 {
        let path = image_path("ring-zero", device_number);
        let mut file = FileBackedBlockDevice::create_image_file_exclusively(
            &path,
            IMAGE_BYTES,
            PhysicalBlockSizeInBytes(512),
        )
        .expect("建镜像");
        // 环的头、中、尾各撒一扇区非 0：上一次 mkfs 留下的记录就长在这些地方。
        for offset in [
            RING_START,
            RING_START + JOURNAL_RING_DEFAULT_BYTES / 2,
            RING_START + JOURNAL_RING_DEFAULT_BYTES - 512,
        ] {
            file.write_at(
                DeviceOffsetInBytes(offset),
                &[0xB6u8; 512],
                WriteDurability::Plain,
            )
            .expect("撒非 0");
        }
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

    for (_, device) in &devices {
        for offset in [
            RING_START,
            RING_START + JOURNAL_RING_DEFAULT_BYTES / 2,
            RING_START + JOURNAL_RING_DEFAULT_BYTES - 512,
            RING_START + JOURNAL_RING_DEFAULT_BYTES - 1024 * 1024,
        ] {
            let sample = read(device, offset, 512);
            assert!(
                sample.iter().all(|byte| *byte == 0),
                "环里偏移 {offset} 那一扇区没清干净"
            );
        }
    }

    let operations = stream.operations();
    let zero_fills: Vec<&singlefs_harness::RecordedOperation> = operations
        .iter()
        .filter(|operation| operation.kind == RecordedOperationKind::WriteZeroes)
        .collect();
    assert_eq!(
        zero_fills.len(),
        8,
        "每块盘四次清零调用：三个根环区域各一次、journal 环一次"
    );
    let journal_zero_fills: Vec<&&singlefs_harness::RecordedOperation> = zero_fills
        .iter()
        .filter(|zero_fill| zero_fill.offset == DeviceOffsetInBytes(RING_START))
        .collect();
    assert_eq!(
        journal_zero_fills.len(),
        2,
        "两块盘各清一次环，一块盘一次调用"
    );
    for zero_fill in &journal_zero_fills {
        assert_eq!(zero_fill.length, JOURNAL_RING_DEFAULT_BYTES);
    }
    let root_ring_zeroed_bytes: u64 = 2
        * ROOT_RING_REGIONS
        * region_length_in_bytes(
            parameters().geometry.fixed_structure_slot_spacing,
            parameters().geometry.root_ring_slots_per_region,
        );
    assert_eq!(
        root_ring_zeroed_bytes, 196_608,
        "两盘 × 3 个区域 × 8 个槽 × 4096 槽距"
    );
    let written_bytes: u64 = operations
        .iter()
        .map(|operation| match operation.kind {
            RecordedOperationKind::Write
            | RecordedOperationKind::WriteForceUnitAccess
            | RecordedOperationKind::WriteZeroes => operation.length,
            RecordedOperationKind::Barrier => 0,
        })
        .sum();
    assert_eq!(
        written_bytes,
        MKFS_METADATA_BYTES + 2 * JOURNAL_RING_DEFAULT_BYTES + root_ring_zeroed_bytes,
        "并行线四验收（连根环一起清之后重算）：116 224 + 2 × 768 MiB + 6 × 32 KiB = 1 610 925 568"
    );
    assert_eq!(written_bytes, 1_610_925_568, "验收那个数写成一个字面量");
    assert_eq!(
        JOURNAL_RING_DEFAULT_BYTES,
        768 * 1024 * 1024,
        "验收那个数里的 768 MiB 就是这个常量"
    );

    for path in &paths {
        std::fs::remove_file(path).expect("清理镜像");
    }
}

/// C484（mkfs 不清根环，同 fsid 重来旧根还择得中）的正题：在一块**已经有池**的盘上拿**同一个 fsid** 重做 mkfs，
/// 根环里一条旧池的根都不许剩下，择到的是新池的第 0 代根。
///
/// 怎么造出那个漏：先跑完整条第一个事务（mkfs → 取号 → 暖机 × 2 → 第一个事务），根环里因此留着四条根——
/// txg 0 / 1 / 2 落在各自区域的槽 0（重做 mkfs 的三道 FUA 写原地盖掉它们），而 txg 3 落在区域 0 的**槽 1**，
/// 重做 mkfs 一个字节都碰不到那一槽。挡住它被择中的只有 fsid（`RootRecord::parse_slot` 逐条比），
/// 而 fsid 是调用方给的、mkfs 自己不生成 ⇒ 同一个 fsid 重来，不清根环的话 `choose_root`
/// 按 (checkpoint_txg, 实例代号) 取最大，择中的是旧池那条 txg 3，它指着的树表、实例表、分配记录全是旧池的账。
///
/// 重做之前把根环三段**整段**弄脏（两块盘都弄，包括区域不归属那块盘上的同一段），只留那四条真根的 512 字节：
/// 稀疏镜像本来就读回全 0，不先弄脏就分不出「清过」与「从来没写过」（`test-discipline.md`「读不到 ≠ 读到 0」）。
#[test]
fn remaking_the_pool_with_the_same_filesystem_identifier_leaves_no_root_of_the_old_pool() {
    const DIRTY_BYTE: u8 = 0xB6;
    let parameters = parameters();
    let spacing = parameters.geometry.fixed_structure_slot_spacing;
    let root_slot_bytes = usize::try_from(parameters.geometry.physical_block_size).expect("根槽宽");
    let stream = SharedStream::new();
    let mut paths = Vec::new();
    let mut devices = Vec::new();
    for device_number in 0..2u32 {
        let path = image_path("remake", device_number);
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
    let old_pool = run_first_transaction(&parameters, &mut devices, &stream, |_point, _devices| {})
        .expect("旧池：mkfs 加第一个事务");
    assert_eq!(
        old_pool.output.root.checkpoint_txg,
        CheckpointTxg(3),
        "第一个事务发布的是 txg 3"
    );
    let landmine = RootRingSlot { region: 0, slot: 1 };
    let landmine_device = 0usize;
    let old_root = RootRecord::parse_slot(
        &read(
            &devices[landmine_device].1,
            slot_offset(landmine, spacing).0,
            root_slot_bytes,
        ),
        &parameters.filesystem_identifier,
    )
    .expect("旧池 txg 3 的根落在区域 0 槽 1：重做 mkfs 的三道 FUA 写碰不到这一槽");
    assert_eq!(
        (old_root.checkpoint_txg, old_root.instance),
        (CheckpointTxg(3), InstanceGeneration(1)),
        "埋下的就是旧池最新的那条根"
    );

    // 旧池留在根环里的四条根：(盘, 区域, 槽)。除它们之外的字节整段撒上非 0。
    let old_root_positions = [
        (0usize, 0u64, 0u64),
        (1usize, 1u64, 0u64),
        (0usize, 2u64, 0u64),
        (0usize, 0u64, 1u64),
    ];
    for (device_number, (_, device)) in devices.iter_mut().enumerate() {
        for region in 0..ROOT_RING_REGIONS {
            for slot in 0..parameters.geometry.root_ring_slots_per_region.count() {
                let carries_an_old_root =
                    old_root_positions.contains(&(device_number, region, slot));
                let dirty_from = if carries_an_old_root {
                    u64::try_from(root_slot_bytes).expect("根槽宽放得进 u64")
                } else {
                    0
                };
                let dirty = vec![
                    DIRTY_BYTE;
                    usize::try_from(u64::from(spacing) - dirty_from)
                        .expect("槽距放得进 usize")
                ];
                device
                    .write_at(
                        DeviceOffsetInBytes(
                            slot_offset(RootRingSlot { region, slot }, spacing).0 + dirty_from,
                        ),
                        &dirty,
                        WriteDurability::Plain,
                    )
                    .expect("撒非 0");
            }
        }
    }

    let remade =
        make_filesystem(&parameters, &mut devices).expect("同一个 fsid 在用过的盘上重做 mkfs");
    assert_eq!(remade.root.checkpoint_txg, CheckpointTxg(0));

    assert_eq!(
        RootRecord::parse_slot(
            &read(
                &devices[landmine_device].1,
                slot_offset(landmine, spacing).0,
                root_slot_bytes,
            ),
            &parameters.filesystem_identifier,
        ),
        None,
        "旧池 txg 3 的根（区域 0 槽 1）在重做 mkfs 之后还解得开：根环没清"
    );
    let roots = readable_roots(
        &devices,
        &parameters.region_devices,
        &parameters.geometry,
        &parameters.filesystem_identifier,
    );
    assert_eq!(
        roots,
        vec![remade.root, remade.root, remade.root],
        "根环里自证得过的根只剩这次 mkfs 写下的三条"
    );
    let system_configuration = choose_system_configuration(&devices).expect("择系统配置");
    assert_eq!(
        system_configuration.immutable.filesystem_identifier, parameters.filesystem_identifier,
        "新池的 fsid 与旧池逐字相同：挡住旧根的从来不是它"
    );
    assert_eq!(
        choose_root(&devices, &system_configuration),
        Some(remade.root),
        "择到的是新池的第 0 代根"
    );

    // 三段根环在**每块盘**上只剩这次写下的根那 512 字节，别处全 0：清零按几何、不按区域归属。
    for (device_number, (_, device)) in devices.iter().enumerate() {
        for region in 0..ROOT_RING_REGIONS {
            // ⚠️ 读多长由**用例自己算**，不调 `region_length_in_bytes`：那正是清根环用的那个函数，
            // 用它算读长的话，它少算一个槽时这段复核也跟着少读一个槽——最后那个没被清的槽
            // 结构性地看不见，那条变异一次都红不了（`.claude/rules/mutation-sampling.md` 第三类：
            // 取样点不敏感。2026-09-22 变异分诊实测：这一条「按 S − 1 个槽算」的变异没红，就是这么漏的）。
            let region_bytes = read(
                device,
                region_start(region).0,
                usize::try_from(
                    parameters.geometry.root_ring_slots_per_region.count() * u64::from(spacing),
                )
                .expect("区域长度放得进 usize"),
            );
            let carries_the_new_root = parameters.region_devices
                [usize::try_from(region).expect("区域号")]
            .0 == u32::try_from(device_number).expect("设备号");
            let tail_from = if carries_the_new_root {
                assert_eq!(
                    region_bytes[..root_slot_bytes],
                    remade.root.to_slot(root_slot_bytes)[..],
                    "盘 {device_number} 区域 {region} 的槽 0 该是这次写下的第 0 代根"
                );
                root_slot_bytes
            } else {
                0
            };
            let first_leftover = region_bytes[tail_from..]
                .iter()
                .position(|byte| *byte != 0)
                .map(|offset| tail_from + offset);
            assert_eq!(
                first_leftover, None,
                "盘 {device_number} 区域 {region}：mkfs 之后区域内还有非 0 字节，第一处在区域内偏移 {first_leftover:?}"
            );
        }
    }

    let mounted = mount_writable(&parameters, &mut devices).expect("新池可写挂载");
    assert_eq!(
        (
            mounted.output.chosen_root.checkpoint_txg,
            mounted.output.chosen_root.instance
        ),
        (CheckpointTxg(0), InstanceGeneration(0)),
        "挂载择到的是新池刚种下的第 0 代根，不是旧池那条 txg 3"
    );

    drop(devices);
    for path in &paths {
        std::fs::remove_file(path).expect("清理镜像");
    }
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
