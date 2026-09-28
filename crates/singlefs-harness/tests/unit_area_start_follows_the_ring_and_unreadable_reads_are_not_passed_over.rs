//! 实审 A3b（规格 `/tmp/claude-1000/impl-rev-a3b/spec.md`，写于 2026-09-27）：
//!
//! 一、代码审阅第 15 条（C475（非默认环长下单元区起点取编译期常量））：单元区起点按环长现算、全链路一处来源——mkfs 按环长算起点
//! （journal 环末尾的下一个槽，D3（空间分配） 已定项 10 ④）、写进系统配置偏移 417 的 8 字节、实例表与树表写在那里；读者择系统配置时
//! 判那 8 字节就是环长现算的那个；挂载按它建空闲图、判分配记录落点；checker 按那 8 字节读。默认环 768 MiB 下起点仍是 50176，
//! 新池新建文件写出的字节逐字节不变。起点不在 64 槽段边界上 mkfs 在任何写之前拒（实审 A2c 的「第一版不支持」）。
//!
//! 二、实审 C11b 顺带看到的第 5 条：环长 ≥ 2³² × 12288 字节时在飞上限装不进系统配置里那 4 字节，mkfs 在任何写之前拒，不在写系统配置那一步 panic；
//! 读者择系统配置时同样拒这样的环长（不拒的话挂载之后轮换系统配置槽时 panic）。
//!
//! 三、实审 A3c Q-A（主 agent 定走甲）：重建上一版时判树表条目的排序契约（D8（核心索引结构） 已定项 8「条目按树 ID 升序排」），
//! 走得到发布路径那条「树表条目按树 ID 升序」断言的镜像，可写挂载在任何写之前拒。探针一（两条条目互换种类）、探针二（两条同号）各一条。
//!
//! 四、C554 乙报告 Q6（用户 2026-09-26「不能接受 要改」）：两处「读不出就无声放过」照乙的形态收——先重读一次，仍读不出就在任何写之前拒：
//! 算生效 F 时最新那条根的实例表读不出（改之前「不按表滤」）；挂着时抬 F 重算影子账时根槽读不出（改之前当那一槽没有根）。
//! 合入后验证一（规格 `/tmp/claude-1000/impl-merge-verify-1/spec.md`，2026-09-27）改了三样：
//! - 读根环照 D16（发布语义） 已定项 1「根槽这一次读坏」那一行分两类（善后一报告 P1）：挂载那一刻就读坏、这个进程之后没写过的槽当没有根，
//!   不重读、不拒；这个进程知道住着根的槽读坏重读一次，仍坏才拒；
//! - 两个拒的成员从 `RecoveryFailure` 挪进乙那一族 `StillUnreadableAfterOneReread`（A3b 报告 Q4）；
//! - 另两处读不出就无声放过（A3b 报告 Q7）：管理员回退判候选照同一读法拒成 `RollbackError` 的新成员；写系统配置槽之前算 F 生效值
//!   读不出的重读一次，仍读不出照原来的退路（写入口手里没有根环表，不拒）。
//!
//! 代码三方第二轮（`research/prompts/m2-closeout-code-r2-main-verification.md`，规格 `/tmp/claude-1000/impl-r2-fixes-b/spec.md`）改了两样：
//! - 读根环照 D16（发布语义） 已定项 1「根槽这一次读坏」那一行全句（用户 2026-09-26 定）：知道住着根的槽「读不出**或自证不过**」都重读一次、
//!   仍坏就拒（第四节末尾两条；包装盘另加「读得出、字节被改」那一种坏法）。管理员回退算水位读根环那一遍仍是 `readable_roots`
//!   （不重读），判候选那两遍读得出、这一遍瞬时读坏时水位取内存里的现行那一版（`the_rollback_takes_the_inode_number_watermark_…`）。
//! - 五、core 读系统配置的 journal 环起点（偏移 325）与根环起点（偏移 371），与第一版常量不等整池拒（Y4-a，用户 2026-09-27 定）。
//!
//! 盘：两块内存稀疏盘（`SparseBlockDevice`），改盘上字节之后重封每一道校验和；读故障用这个文件自己的包装盘按「这一段第几次读」坏。
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;

use singlefs_checker::image::{ImageReader, InvariantVerdict};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
};
use singlefs_core::allocator::{
    PoolAllocator, UnitAreaStart, UnitAreaStartOffTheClusterSegmentBoundaryUnsupported,
};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::checksum::{crc32_castagnoli, wide_checksum_with_field_zeroed};
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, MakeFilesystemError,
    MakeFilesystemParameters, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor, ring_slots_known_to_hold_a_root_by,
    roll_back_by_a_forward_publish, MountError, RollbackError, RollbackTarget, ShadowLedger,
    StillUnreadableAfterOneReread,
};
use singlefs_core::mounted_read::{mount_read_only, MountReadOnlyFailure};
use singlefs_core::records::{TREE_KIND_LIVELIST, TREE_KIND_SPARSE_SIDE_TABLE};
use singlefs_core::recovery::{
    choose_system_configuration, effective_rollback_floor_rereading_the_newest_instance_table_once,
    effective_rollback_floor_rereading_unreadable_reads_once, every_root_ring_slot,
    read_root_ring_slot, readable_roots,
    readable_roots_rereading_ring_slots_known_to_hold_a_root_once, readable_roots_with_ring_slots,
    recover, verified_system_configuration_slots, BadRootRingSlotReading,
    InstanceTableOfTheNewestRootStillUnreadableAfterOneReread, JournalPolicy, RecoveryFailure,
    RecoveryOutcome, RootRingSlotKnownToHoldARootStillBadAfterOneReread, RootRingSlotReading,
    SystemConfigurationValueOutsideWhatThisReaderAccepts,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, RootRingSlot, RootRingSlotsPerRegion};
use singlefs_core::system_configuration::{
    unit_area_start_slot_recorded_in_the_slot, SystemImmutableSizes,
    SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_new_inodes, publish_overwrite, warm_up,
    FirstFile, PoolWriter, TransactionOutput,
};
use singlefs_core::unit::seal_header_checksum;
use singlefs_format::{
    JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES, JOURNAL_RING_START_SLOT,
    JOURNAL_SAFETY_FACTOR, ROOT_RING_BASE_SLOT, SLOT_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES,
    UNIT_AREA_START_SLOT,
};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice, SECTOR_BYTES};

const FOUR_GIBIBYTES: u64 = 4 << 30;
const MEBIBYTE: u64 = 1 << 20;
const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"singlefs-rev-a3b";
const WRITE_TIME_SECONDS: u64 = 1_788_000_000;
/// 内存稀疏盘按 512 字节扇区存：物理块宽 512。
const SECTOR_PHYSICAL_BLOCK_SIZE: PhysicalBlockSizeInBytes = PhysicalBlockSizeInBytes(512);
const FIRST_FILE_BYTES: usize = 3000;
/// 系统配置槽里 journal 环长那 8 字节（`SystemConfiguration::to_slot` 的写法；`parse_slot` 按它读回）。
const JOURNAL_RING_BYTES_OFFSET: usize = 333;
/// 系统配置槽里单元区起始槽号那 8 字节（字段表 `layout/01-first-txn.md` 一）。
const UNIT_AREA_START_SLOT_OFFSET: usize = 417;

type SparseDevices = Vec<(DeviceIdentity, SparseBlockDevice)>;

fn geometry_with_journal_ring_bytes(journal_ring_bytes: u64) -> SystemImmutableSizes {
    SystemImmutableSizes {
        physical_block_size: 512,
        minimum_input_output_bytes: 512,
        fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(512),
        journal_ring_bytes,
        root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
    }
}

fn parameters_with_journal_ring_bytes(journal_ring_bytes: u64) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: geometry_with_journal_ring_bytes(journal_ring_bytes),
    }
}

fn content_of(length_in_bytes: usize, seed: u64) -> Vec<u8> {
    (0..length_in_bytes)
        .map(|index| {
            let index = u64::try_from(index).expect("长度装得进 u64");
            u8::try_from((index * 131 + seed * 7 + 1) % 251).expect("小于 256")
        })
        .collect()
}

fn sparse_devices(device_bytes: u64) -> SparseDevices {
    [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| {
            (
                identity,
                SparseBlockDevice::new(device_bytes, SECTOR_PHYSICAL_BLOCK_SIZE),
            )
        })
        .collect()
}

fn images_of(
    devices: &[(DeviceIdentity, SparseBlockDevice)],
) -> Vec<(DeviceIdentity, SparseDevice)> {
    devices
        .iter()
        .map(|(identity, device)| (*identity, device.image.clone()))
        .collect()
}

fn device_mut(devices: &mut SparseDevices, device: DeviceIdentity) -> &mut SparseBlockDevice {
    &mut devices
        .iter_mut()
        .find(|(identity, _)| *identity == device)
        .expect("池里有这块盘")
        .1
}

fn memory_pool_of(
    devices: &[(DeviceIdentity, SparseBlockDevice)],
    device_bytes: u64,
) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: device_bytes,
    }
}

/// 池级 checker 判红的不变量名（`check_pool_image` 交回的判定里 `Violated` 的那几条）。
fn checker_violations(image: &MemoryPool) -> Vec<(&'static str, String)> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some((invariant, detail)),
            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
        })
        .collect()
}

/// 两块内存盘上 mkfs → 取号 1 → 暖机 → 第一个文件（txg 3），分配器是 mkfs 同一个进程那一条（`allocator_after_make_filesystem`）。
struct PoolAfterTheFirstFile {
    parameters: MakeFilesystemParameters,
    device_bytes: u64,
    devices: SparseDevices,
    unit_area_start: UnitAreaStart,
    allocator: PoolAllocator,
    instance: InstanceGeneration,
    first: TransactionOutput,
}

fn pool_after_the_first_file(journal_ring_bytes: u64, device_bytes: u64) -> PoolAfterTheFirstFile {
    let parameters = parameters_with_journal_ring_bytes(journal_ring_bytes);
    let mut devices = sparse_devices(device_bytes);
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
    let content = content_of(FIRST_FILE_BYTES, 0);
    let (instance, first) = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let instance = acquire_instance(&mut writer).expect("取号 1");
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        let first = publish_first_file(
            &mut writer,
            &mut allocator,
            warmed.roots.last().expect("暖机写了两条根"),
            FirstFile {
                content: &content,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("第一个文件");
        (instance, first)
    };
    PoolAfterTheFirstFile {
        parameters,
        device_bytes,
        devices,
        unit_area_start: genesis.unit_area_start,
        allocator,
        instance,
        first,
    }
}

/// 一块盘系统配置槽 `slot_index`（0 或 1）的原样字节。
fn system_configuration_slot_bytes(
    devices: &[(DeviceIdentity, SparseBlockDevice)],
    device: DeviceIdentity,
    slot_index: u64,
    parameters: &MakeFilesystemParameters,
) -> Vec<u8> {
    let offset = slot_index * u64::from(parameters.geometry.fixed_structure_slot_spacing);
    devices
        .iter()
        .find(|(identity, _)| *identity == device)
        .expect("池里有这块盘")
        .1
        .image
        .read(
            DeviceOffsetInBytes(offset),
            usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096"),
        )
}

/// 两块盘各两个系统配置槽逐个改一下、整槽校验和重封：池级字段两盘四槽同值，改就四槽一起改。
fn rewrite_every_system_configuration_slot(
    devices: &mut SparseDevices,
    parameters: &MakeFilesystemParameters,
    change: impl Fn(&mut [u8]),
) {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        for slot_index in 0..2 {
            let mut slot = system_configuration_slot_bytes(devices, device, slot_index, parameters);
            change(&mut slot);
            let digest = wide_checksum_with_field_zeroed(
                &slot,
                slot_bytes,
                SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
            );
            slot[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32]
                .copy_from_slice(&digest);
            let offset = slot_index * u64::from(parameters.geometry.fixed_structure_slot_spacing);
            device_mut(devices, device)
                .image
                .write(DeviceOffsetInBytes(offset), &slot);
        }
    }
}

/// 两块盘整盘镜像的 CRC-32C：按盘、按写过的扇区号升序，把「盘号、扇区号、扇区的 512 字节」依次喂进去。
/// 没写过的扇区读出全 0、不进摘要（mkfs 清零是删扇区，`SparseDevice::zero_fill`）。
fn crc32_of_the_images(devices: &[(DeviceIdentity, SparseBlockDevice)], device_bytes: u64) -> u32 {
    let mut fed = Vec::new();
    for (identity, device) in devices {
        for sector in device
            .image
            .written_sectors_in(DeviceOffsetInBytes(0), device_bytes)
        {
            fed.extend_from_slice(&identity.0.to_le_bytes());
            fed.extend_from_slice(&sector.to_le_bytes());
            fed.extend_from_slice(&device.image.read(
                DeviceOffsetInBytes(sector * SECTOR_BYTES),
                usize::try_from(SECTOR_BYTES).expect("512"),
            ));
        }
    }
    crc32_castagnoli(&fed)
}

// ─── 一、单元区起点随环长走 ───

/// 默认环 768 MiB 上 mkfs → 取号 → 暖机 → 第一个文件之后，两块盘整盘镜像的 CRC-32C。钉的是改之前那一份代码写出的字节
/// （实审 A3b 开工时主工作区的副本上同一段流程现算，`name=a3b_default_ring_new_pool_file_creation_images_crc32 value=1783297687`）：
/// 默认环下单元区起点仍是 50176，新池新建文件写出的字节逐字节不变。
const DEFAULT_RING_NEW_POOL_FILE_CREATION_IMAGES_CRC32: u32 = 1_783_297_687;

/// 默认环 768 MiB：mkfs 按环长现算的单元区起点就是 50176（`INSTANCE_TABLE_SLOT`），树表第 0 版在 50178（`TREE_TABLE_GENESIS_SLOT`），
/// 系统配置偏移 417 的 8 字节写 50176；mkfs 到第一个文件写出的两块盘与改之前逐字节相同（整盘 CRC-32C 钉值）。
#[test]
fn the_default_journal_ring_keeps_the_unit_area_at_slot_50176_and_the_new_pool_file_creation_bytes_unchanged(
) {
    let pool = pool_after_the_first_file(JOURNAL_RING_DEFAULT_BYTES, FOUR_GIBIBYTES);
    assert_eq!(
        pool.unit_area_start.slot(),
        INSTANCE_TABLE_SLOT,
        "默认环下起点仍是 50176"
    );
    assert_eq!(INSTANCE_TABLE_SLOT, SlotNumber(UNIT_AREA_START_SLOT));
    assert!(
        pool.first.root.tree_table.locations[0].slot.0 > TREE_TABLE_GENESIS_SLOT.0,
        "第一个文件换下了 mkfs 那片树表（50178），新树表落在开放段里"
    );
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        for slot_index in 0..2 {
            assert_eq!(
                unit_area_start_slot_recorded_in_the_slot(&system_configuration_slot_bytes(
                    &pool.devices,
                    device,
                    slot_index,
                    &pool.parameters,
                )),
                SlotNumber(UNIT_AREA_START_SLOT),
                "盘 {device:?} 槽 {slot_index} 偏移 417 写 50176"
            );
        }
    }
    assert_eq!(
        crc32_of_the_images(&pool.devices, pool.device_bytes),
        DEFAULT_RING_NEW_POOL_FILE_CREATION_IMAGES_CRC32,
        "默认环下 mkfs 到第一个文件写出的两块盘与改之前逐字节相同"
    );
}

/// 一条非默认环长上走完 mkfs → 第一个文件 → 可写挂载 → 冷走读 → 池级 checker，逐处核单元区起点都是环长现算的那个 `expected_start`。
fn assert_the_unit_area_follows_a_journal_ring_of(
    journal_ring_bytes: u64,
    device_bytes: u64,
    expected_start: u64,
) {
    let mut pool = pool_after_the_first_file(journal_ring_bytes, device_bytes);
    assert_eq!(
        pool.unit_area_start.slot(),
        SlotNumber(expected_start),
        "mkfs 按环长现算的单元区起点"
    );
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        assert_eq!(
            unit_area_start_slot_recorded_in_the_slot(&system_configuration_slot_bytes(
                &pool.devices,
                device,
                0,
                &pool.parameters,
            )),
            SlotNumber(expected_start),
            "系统配置偏移 417 的 8 字节写的是现算的起点（盘 {device:?}）"
        );
    }
    let genesis_instance_table_slot = pool.first.root.instance_table.locations[0].slot;
    assert_eq!(
        genesis_instance_table_slot,
        SlotNumber(expected_start),
        "mkfs 的实例表第 0 片就写在单元区起点（第一个文件照抄它的指针）"
    );
    assert!(
        pool.first
            .allocation_records
            .iter()
            .all(|record| record.slot.0 >= expected_start),
        "第一个文件的每一条分配记录都落在单元区起点之后：{:?}",
        pool.first.allocation_records
    );
    let journal_ring_end_slot =
        (JOURNAL_RING_START_SLOT * SLOT_BYTES + journal_ring_bytes).div_ceil(SLOT_BYTES);
    assert!(
        journal_ring_end_slot <= expected_start,
        "单元区起点不在 journal 环里"
    );
    let image_after_the_first_file = memory_pool_of(&pool.devices, pool.device_bytes);
    for device in [0, 1] {
        let candidates = ImageReader::candidate_unit_slots(&image_after_the_first_file, device)
            .expect("内存镜像交得出单元扫描候选");
        assert!(
            candidates.contains(&expected_start)
                && candidates.iter().all(|slot| *slot >= expected_start),
            "崩溃镜像的单元扫描候选从盘上记着的单元区起点起、罩住 mkfs 的实例表（盘 {device}）：{:?}",
            candidates.iter().take(4).collect::<Vec<_>>()
        );
    }
    let violations = checker_violations(&image_after_the_first_file);
    assert!(
        violations.is_empty(),
        "池级 checker 按偏移 417 那 8 字节划单元区，一条不红：{violations:?}"
    );
    let mounted = mount_writable(&pool.parameters, &mut pool.devices).expect("可写挂载");
    for device_map in &mounted.allocator.devices {
        assert_eq!(
            device_map.unit_area_start().slot(),
            SlotNumber(expected_start),
            "挂载按盘上系统配置建的空闲图从现算的起点起"
        );
    }
    let report = recover(&pool.devices, JournalPolicy::Consult);
    assert!(
        matches!(report.outcome, RecoveryOutcome::FileRead { ref content, .. } if *content == content_of(FIRST_FILE_BYTES, 0)),
        "冷走读读回第一个文件：{:?}",
        report.outcome
    );
    let violations_after_the_mount =
        checker_violations(&memory_pool_of(&pool.devices, pool.device_bytes));
    assert!(
        violations_after_the_mount.is_empty(),
        "可写挂载写行、暖机之后池级 checker 一条不红：{violations_after_the_mount:?}"
    );
}

/// 环 1 GiB（改之前 mkfs 按「环末端越过编译期的单元区起点」拒）：单元区从 1024 + 65536 = 66560 起，mkfs、第一个文件、可写挂载、冷走读、checker 都按它。
#[test]
fn a_one_gibibyte_journal_ring_puts_the_unit_area_right_after_the_ring_through_mount_and_checker() {
    assert_the_unit_area_follows_a_journal_ring_of(1 << 30, FOUR_GIBIBYTES, 66_560);
}

/// 环 128 MiB（改之前单元区照旧从 50176 起、白扔 640 MiB）：单元区从 1024 + 8192 = 9216 起。
#[test]
fn a_128_mebibyte_journal_ring_puts_the_unit_area_right_after_the_ring_through_mount_and_checker() {
    assert_the_unit_area_follows_a_journal_ring_of(128 * MEBIBYTE, FOUR_GIBIBYTES, 9216);
}

/// 环 128 MiB 加一槽：末尾的下一个槽 9217 不在 64 槽段边界上，mkfs 在任何写之前拒成 `UnitAreaStartOffTheClusterSegmentBoundaryUnsupported`，
/// 两块盘一个扇区都没写。改之前照收（环末端不越过编译期的起点 50176）。
#[test]
fn a_journal_ring_whose_next_slot_is_off_the_cluster_segment_boundary_is_refused_by_mkfs_before_any_write(
) {
    let ring_bytes = 128 * MEBIBYTE + SLOT_BYTES;
    let parameters = parameters_with_journal_ring_bytes(ring_bytes);
    let mut devices = sparse_devices(FOUR_GIBIBYTES);
    let refused = make_filesystem(&parameters, &mut devices);
    assert!(
        matches!(
            refused,
            Err(MakeFilesystemError::UnitAreaStartOffTheClusterSegmentBoundaryUnsupported(
                UnitAreaStartOffTheClusterSegmentBoundaryUnsupported {
                    journal_ring_bytes,
                    slot_after_the_journal_ring: SlotNumber(9217),
                }
            )) if journal_ring_bytes == ring_bytes
        ),
        "起点 9217 不在段边界上：{:?}",
        refused.as_ref().err()
    );
    assert_eq!(
        images_of(&devices),
        images_of(&sparse_devices(FOUR_GIBIBYTES)),
        "两块盘一个扇区都没写"
    );
}

/// 读者：四个系统配置槽偏移 417 的 8 字节改成 50240（不是默认环长现算的 50176），整槽校验和重封。
/// 可写挂载在任何写之前拒成 `UnitAreaStartNotTheSlotAfterTheJournalRing`，两块盘逐字节不变。改之前读者不读这 8 字节、照常挂上。
#[test]
fn a_system_configuration_recording_a_unit_area_start_other_than_the_slot_after_the_ring_is_refused_before_any_write(
) {
    let mut pool = pool_after_the_first_file(JOURNAL_RING_DEFAULT_BYTES, FOUR_GIBIBYTES);
    let parameters = pool.parameters.clone();
    rewrite_every_system_configuration_slot(&mut pool.devices, &parameters, |slot| {
        slot[UNIT_AREA_START_SLOT_OFFSET..UNIT_AREA_START_SLOT_OFFSET + 8]
            .copy_from_slice(&50_240u64.to_le_bytes());
    });
    let images_before = images_of(&pool.devices);
    let refused = mount_writable(&parameters, &mut pool.devices);
    assert!(
        matches!(
            refused,
            Err(MountError::Recovery(RecoveryFailure::SystemConfigurationValueRefused {
                value: SystemConfigurationValueOutsideWhatThisReaderAccepts::UnitAreaStartNotTheSlotAfterTheJournalRing {
                    recorded_unit_area_start_slot: SlotNumber(50_240),
                    slot_after_the_journal_ring: SlotNumber(50_176),
                },
                ..
            }))
        ),
        "{:?}",
        refused.as_ref().err()
    );
    assert_eq!(images_of(&pool.devices), images_before, "两块盘逐字节不变");
}

/// 在飞上限装不进 4 字节的最短环长：2³² 条 × 3（F）× 4096 字节 = 48 TiB（在飞上限恰好 2³²）。它的槽数是 64 的整数倍，单元区起点在段边界上。
const JOURNAL_RING_WHOSE_IN_FLIGHT_LIMIT_OVERFLOWS_FOUR_BYTES: u64 =
    (1 << 32) * JOURNAL_SAFETY_FACTOR * JOURNAL_RECORD_BYTES;

/// 读者：四个系统配置槽的环长写成 48 TiB、偏移 417 写成它现算的起点（两者自洽），整槽校验和重封。择系统配置拒成
/// `JournalRingBytesOutsideTheSupportedRange`：在飞上限装不进 4 字节，挂载之后轮换系统配置槽时写那 4 字节那一句会 panic。
/// （改之前这里由「环末端越过编译期的单元区起点」挡着；单元区起点改成随环长走之后靠的是这一判。）
#[test]
fn a_system_configuration_whose_in_flight_limit_overflows_its_four_bytes_is_refused_by_the_reader()
{
    let mut pool = pool_after_the_first_file(JOURNAL_RING_DEFAULT_BYTES, FOUR_GIBIBYTES);
    let parameters = pool.parameters.clone();
    let ring_bytes = JOURNAL_RING_WHOSE_IN_FLIGHT_LIMIT_OVERFLOWS_FOUR_BYTES;
    let start_after_the_ring = JOURNAL_RING_START_SLOT + ring_bytes / SLOT_BYTES;
    rewrite_every_system_configuration_slot(&mut pool.devices, &parameters, |slot| {
        slot[JOURNAL_RING_BYTES_OFFSET..JOURNAL_RING_BYTES_OFFSET + 8]
            .copy_from_slice(&ring_bytes.to_le_bytes());
        slot[UNIT_AREA_START_SLOT_OFFSET..UNIT_AREA_START_SLOT_OFFSET + 8]
            .copy_from_slice(&start_after_the_ring.to_le_bytes());
    });
    assert_eq!(
        choose_system_configuration(&pool.devices),
        Err(RecoveryFailure::SystemConfigurationValueRefused {
            device: DeviceIdentity(0),
            value: SystemConfigurationValueOutsideWhatThisReaderAccepts::JournalRingBytesOutsideTheSupportedRange {
                journal_ring_bytes: ring_bytes,
            },
        })
    );
}

// ─── 二、在飞上限装不进 4 字节：mkfs 在任何写之前拒 ───

/// 两块 200 TiB 的稀疏盘（环长上界「≤ 容量 ÷ 4」放行 48 TiB 的环）上 mkfs：在任何写之前拒成
/// `JournalInFlightRecordLimitWiderThanItsFourByteField { 48 TiB, 2³² }`，两块盘一个扇区都没写。改之前清完根环与 journal 环、写完两个单元与三条根，
/// 在写系统配置那一步 panic（`to_slot` 里「在飞上限 4 字节」那句 expect）。
#[test]
fn a_journal_ring_whose_in_flight_limit_overflows_four_bytes_is_refused_by_mkfs_before_any_write() {
    let device_bytes: u64 = 200 << 40;
    let ring_bytes = JOURNAL_RING_WHOSE_IN_FLIGHT_LIMIT_OVERFLOWS_FOUR_BYTES;
    assert!(ring_bytes <= device_bytes / 4, "环长上界那一关放行");
    let parameters = parameters_with_journal_ring_bytes(ring_bytes);
    let mut devices = sparse_devices(device_bytes);
    let refused = make_filesystem(&parameters, &mut devices);
    assert!(
        matches!(
            refused,
            Err(MakeFilesystemError::JournalInFlightRecordLimitWiderThanItsFourByteField {
                ring_bytes: refused_ring_bytes,
                in_flight_record_limit,
            }) if refused_ring_bytes == ring_bytes && in_flight_record_limit == 1 << 32
        ),
        "{:?}",
        refused.as_ref().err()
    );
    assert_eq!(
        images_of(&devices),
        images_of(&sparse_devices(device_bytes)),
        "两块盘一个扇区都没写"
    );
}

// ─── 三、树表条目的排序契约（实审 A3c Q-A 甲） ───

/// 码 2 节点第 `index` 条条目在节点里的字节范围（条目区从 86 + 2k + 29 起，条目宽住 84 + 2k）。
fn entry_range_in_node(node: &[u8], index: usize) -> std::ops::Range<usize> {
    let key_width = usize::from(node[51]);
    let entry_width = usize::from(u16::from_le_bytes([
        node[84 + 2 * key_width],
        node[85 + 2 * key_width],
    ]));
    let entries_start = 86 + 2 * key_width + 29;
    entries_start + index * entry_width..entries_start + (index + 1) * entry_width
}

/// 码 2 节点改过之后重封：载荷 CRC（罩 [86 + 2k, 16384)，住 76 + 2k）与头校验和（罩 [0, 86 + 2k)）。
fn reseal_index_node(node: &mut [u8]) {
    let key_width = usize::from(node[51]);
    let header_end = 86 + 2 * key_width;
    let payload_checksum = crc32_castagnoli(&node[header_end..]);
    node[header_end - 10..header_end - 6].copy_from_slice(&payload_checksum.to_le_bytes());
    seal_header_checksum(node, header_end);
}

/// 树表条目里种类那 2 字节的偏移（树 ID 8 + 条目长度 2 之后，D8（核心索引结构） 已定项 8）。
const TREE_TABLE_ENTRY_KIND_OFFSET: usize = 10;
/// 第一个文件那一版树表里的条目数（七棵进树表的树）。
const TREE_TABLE_ENTRIES_OF_A_FILE_VERSION: usize = 7;

/// 树表里种类是 `kind` 的那一条的下标。
fn index_of_the_entry_of_kind(node: &[u8], kind: u16) -> usize {
    (0..TREE_TABLE_ENTRIES_OF_A_FILE_VERSION)
        .find(|index| {
            let entry = &node[entry_range_in_node(node, *index)];
            u16::from_le_bytes([
                entry[TREE_TABLE_ENTRY_KIND_OFFSET],
                entry[TREE_TABLE_ENTRY_KIND_OFFSET + 1],
            ]) == kind
        })
        .expect("树表里有这种树")
}

/// 根环里 (txg, 实例代号) 最大的那条根与它住的槽。
fn newest_root_in_the_ring(devices: &SparseDevices) -> (RootRingSlot, RootRecord) {
    let system_configuration = choose_system_configuration(devices).expect("系统配置择得到");
    every_root_ring_slot(&system_configuration.immutable.sizes)
        .into_iter()
        .filter_map(|ring_slot| {
            match read_root_ring_slot(
                devices,
                &system_configuration.immutable.region_devices,
                &system_configuration.immutable.sizes,
                &system_configuration.immutable.filesystem_identifier,
                ring_slot,
            ) {
                RootRingSlotReading::SelfVerified(root) => Some((ring_slot, root)),
                RootRingSlotReading::Bad(_) => None,
            }
        })
        .max_by_key(|(_, root)| (root.checkpoint_txg, root.instance))
        .expect("根环里有自证过的根")
}

/// 第一个文件之后，改最新那条根的树表单元（两份），重封树表、改根记录里树表指针两条位置条目的整单元校验和、把根写回它的槽：
/// 盘上每一道校验和都过，只剩排序契约那一格不对。
fn rewrite_the_tree_table_of_the_newest_root(
    pool: &mut PoolAfterTheFirstFile,
    change: impl Fn(&mut [u8]),
) {
    let (ring_slot, mut root) = newest_root_in_the_ring(&pool.devices);
    let node_bytes = usize::try_from(SLOT_BYTES).expect("16384");
    let first_location = root.tree_table.locations[0];
    let mut tree_table = device_mut(&mut pool.devices, first_location.device)
        .image
        .read(first_location.slot.to_device_offset(), node_bytes);
    change(&mut tree_table);
    reseal_index_node(&mut tree_table);
    let unit_checksum = crc32_castagnoli(&tree_table);
    for location in &mut root.tree_table.locations {
        device_mut(&mut pool.devices, location.device)
            .image
            .write(location.slot.to_device_offset(), &tree_table);
        location.unit_checksum = unit_checksum;
    }
    let root_device =
        pool.parameters.region_devices[usize::try_from(ring_slot.region).expect("区域号")];
    let offset = slot_offset(
        ring_slot,
        pool.parameters.geometry.fixed_structure_slot_spacing,
    );
    let root_slot_bytes =
        usize::try_from(pool.parameters.geometry.physical_block_size).expect("根槽宽");
    device_mut(&mut pool.devices, root_device)
        .image
        .write(offset, &root.to_slot(root_slot_bytes));
}

/// 可写挂载在任何写之前拒成 `InvariantViolated { "I-9.16", detail }`（树表条目按树 ID 严格升序且合发号次序），两块盘逐字节不变。
fn assert_the_writable_mount_refuses_the_ordering_before_any_write(
    pool: &mut PoolAfterTheFirstFile,
    expected_detail: &str,
) {
    let images_before = images_of(&pool.devices);
    let parameters = pool.parameters.clone();
    let refused = mount_writable(&parameters, &mut pool.devices);
    assert!(
        matches!(
            refused,
            Err(MountError::Recovery(RecoveryFailure::InvariantViolated {
                invariant: "I-9.16",
                detail,
            })) if detail == expected_detail
        ),
        "{:?}",
        refused.as_ref().err()
    );
    assert_eq!(images_of(&pool.devices), images_before, "两块盘逐字节不变");
}

/// 探针一：livelist 与稀疏旁表两条条目互换种类（条目偏移 10 的 2 字节）。盘上条目仍按树 ID 升序，而按发号次序
/// （extent、inode、分配记录、记账、livelist、稀疏旁表、deadlist）livelist 的号大过稀疏旁表的号：写者照抄这两个号、按种类装树表时降序，
/// 改之前可写挂载在发布路径那条断言上 panic（实审 A3c 实测，`transaction.rs` 装树表那一步）。
#[test]
fn a_tree_table_whose_two_entries_swapped_their_kinds_is_refused_by_the_writable_mount_before_any_write(
) {
    let mut pool = pool_after_the_first_file(JOURNAL_RING_DEFAULT_BYTES, FOUR_GIBIBYTES);
    rewrite_the_tree_table_of_the_newest_root(&mut pool, |tree_table| {
        let livelist = entry_range_in_node(
            tree_table,
            index_of_the_entry_of_kind(tree_table, TREE_KIND_LIVELIST),
        );
        let sparse_side_table = entry_range_in_node(
            tree_table,
            index_of_the_entry_of_kind(tree_table, TREE_KIND_SPARSE_SIDE_TABLE),
        );
        tree_table[livelist.start + TREE_TABLE_ENTRY_KIND_OFFSET
            ..livelist.start + TREE_TABLE_ENTRY_KIND_OFFSET + 2]
            .copy_from_slice(&TREE_KIND_SPARSE_SIDE_TABLE.to_le_bytes());
        tree_table[sparse_side_table.start + TREE_TABLE_ENTRY_KIND_OFFSET
            ..sparse_side_table.start + TREE_TABLE_ENTRY_KIND_OFFSET + 2]
            .copy_from_slice(&TREE_KIND_LIVELIST.to_le_bytes());
    });
    assert_the_writable_mount_refuses_the_ordering_before_any_write(
        &mut pool,
        "树表里七棵树的树 ID 不按发号次序（extent、inode、分配记录、记账、livelist、稀疏旁表、deadlist）严格升序",
    );
}

/// 探针二：livelist 条目的树 ID（条目打头 8 字节）改成稀疏旁表的号，两条同号。改之前可写挂载在同一条断言上 panic。
#[test]
fn a_tree_table_whose_two_entries_carry_the_same_tree_identifier_is_refused_by_the_writable_mount_before_any_write(
) {
    let mut pool = pool_after_the_first_file(JOURNAL_RING_DEFAULT_BYTES, FOUR_GIBIBYTES);
    rewrite_the_tree_table_of_the_newest_root(&mut pool, |tree_table| {
        let livelist = entry_range_in_node(
            tree_table,
            index_of_the_entry_of_kind(tree_table, TREE_KIND_LIVELIST),
        );
        let sparse_side_table = entry_range_in_node(
            tree_table,
            index_of_the_entry_of_kind(tree_table, TREE_KIND_SPARSE_SIDE_TABLE),
        );
        let sparse_side_table_tree: [u8; 8] = tree_table
            [sparse_side_table.start..sparse_side_table.start + 8]
            .try_into()
            .expect("切了 8 字节");
        tree_table[livelist.start..livelist.start + 8].copy_from_slice(&sparse_side_table_tree);
    });
    assert_the_writable_mount_refuses_the_ordering_before_any_write(
        &mut pool,
        "树表条目不按树 ID 严格升序",
    );
}

// ─── 四、读不出就不无声放过（C554 乙报告 Q6） ───

/// 这个文件自己的读故障：几段落点，每一段按它自己被读的次数（从 1 数，一次读碰到这一段就算一次）点名哪几次读坏。
/// 两块盘共用一份。
struct ChosenReadFaults {
    ranges: Vec<ChosenReadFaultsOfARange>,
}

/// 点名的那几次读怎么坏。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ChosenReadFault {
    /// 块设备报 `InputOutput`（读不出）。
    Fails,
    /// 读得出，这一段落在这次读里的第一个字节取反（根槽里是 magic 那一字节：自证不过）。
    ReturnsTheFirstByteOfTheRangeFlipped,
}

struct ChosenReadFaultsOfARange {
    device: DeviceIdentity,
    offset_in_bytes: u64,
    length_in_bytes: u64,
    failing_read_numbers: BTreeSet<u64>,
    fault: ChosenReadFault,
    reads_so_far: u64,
}

/// 这一次读里点名要坏的那一段：它怎么坏，和它在这次读里从第几个字节起。
struct ChosenReadGoingBad {
    fault: ChosenReadFault,
    first_byte_of_the_range_in_the_read: usize,
}

impl ChosenReadFaults {
    /// 记下这一次读碰到的每一段被读了一次；这一次读里有一段点名这一次坏，就交回第一段点名的那一个。
    fn note_a_read_and_tell_how_it_goes_bad(
        &mut self,
        device: DeviceIdentity,
        offset_in_bytes: u64,
        length_in_bytes: u64,
    ) -> Option<ChosenReadGoingBad> {
        let mut going_bad = None;
        for range in &mut self.ranges {
            let overlaps = range.device == device
                && offset_in_bytes < range.offset_in_bytes + range.length_in_bytes
                && range.offset_in_bytes < offset_in_bytes + length_in_bytes;
            if !overlaps {
                continue;
            }
            range.reads_so_far += 1;
            if going_bad.is_none() && range.failing_read_numbers.contains(&range.reads_so_far) {
                going_bad = Some(ChosenReadGoingBad {
                    fault: range.fault,
                    first_byte_of_the_range_in_the_read: usize::try_from(
                        range.offset_in_bytes.saturating_sub(offset_in_bytes),
                    )
                    .expect("一次读里的偏移装得进 usize"),
                });
            }
        }
        going_bad
    }
}

/// 按 [`ChosenReadFaults`] 读坏的内存盘；写、写零、屏障原样交给里面那块盘。
struct DeviceFailingChosenReads {
    identity: DeviceIdentity,
    inner: SparseBlockDevice,
    faults: Rc<RefCell<ChosenReadFaults>>,
}

impl BlockDevice for DeviceFailingChosenReads {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let length_in_bytes = u64::try_from(buffer.len()).expect("一次读的长度装得进 u64");
        let going_bad = self
            .faults
            .borrow_mut()
            .note_a_read_and_tell_how_it_goes_bad(self.identity, offset.0, length_in_bytes);
        match going_bad {
            None => self.inner.read_at(offset, buffer),
            Some(ChosenReadGoingBad {
                fault: ChosenReadFault::Fails,
                ..
            }) => Err(BlockDeviceError::InputOutput(std::io::Error::other(
                "用例点名的这一次读坏",
            ))),
            Some(ChosenReadGoingBad {
                fault: ChosenReadFault::ReturnsTheFirstByteOfTheRangeFlipped,
                first_byte_of_the_range_in_the_read,
            }) => {
                self.inner.read_at(offset, buffer)?;
                buffer[first_byte_of_the_range_in_the_read] ^= 0xff;
                Ok(())
            }
        }
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

/// 一段读故障：哪块盘、从哪个字节起、多长，这一段第几次读坏（读不出）。
fn chosen_read_faults_of(
    device: DeviceIdentity,
    offset_in_bytes: u64,
    length_in_bytes: u64,
    failing_read_numbers: &[u64],
) -> ChosenReadFaultsOfARange {
    ChosenReadFaultsOfARange {
        device,
        offset_in_bytes,
        length_in_bytes,
        failing_read_numbers: failing_read_numbers.iter().copied().collect(),
        fault: ChosenReadFault::Fails,
        reads_so_far: 0,
    }
}

/// 同 [`chosen_read_faults_of`]，点名的那几次读得出、这一段第一个字节取反（自证不过）。
fn chosen_corrupted_reads_of(
    device: DeviceIdentity,
    offset_in_bytes: u64,
    length_in_bytes: u64,
    corrupted_read_numbers: &[u64],
) -> ChosenReadFaultsOfARange {
    ChosenReadFaultsOfARange {
        device,
        offset_in_bytes,
        length_in_bytes,
        failing_read_numbers: corrupted_read_numbers.iter().copied().collect(),
        fault: ChosenReadFault::ReturnsTheFirstByteOfTheRangeFlipped,
        reads_so_far: 0,
    }
}

fn with_chosen_read_faults(
    devices: SparseDevices,
    ranges: Vec<ChosenReadFaultsOfARange>,
) -> Vec<(DeviceIdentity, DeviceFailingChosenReads)> {
    let faults = Rc::new(RefCell::new(ChosenReadFaults { ranges }));
    devices
        .into_iter()
        .map(|(identity, inner)| {
            (
                identity,
                DeviceFailingChosenReads {
                    identity,
                    inner,
                    faults: faults.clone(),
                },
            )
        })
        .collect()
}

fn without_chosen_read_faults(
    devices: Vec<(DeviceIdentity, DeviceFailingChosenReads)>,
) -> SparseDevices {
    devices
        .into_iter()
        .map(|(identity, device)| (identity, device.inner))
        .collect()
}

/// 第一个文件之后在同一个进程里覆盖写三次（txg 4、5、6），现行版本跟着往前推：抬 F 到 3 在上限之内。
fn pool_after_three_overwrites() -> PoolAfterTheFirstFile {
    let mut pool = pool_after_the_first_file(JOURNAL_RING_DEFAULT_BYTES, FOUR_GIBIBYTES);
    for seed in 1..=3 {
        let content = content_of(2000 + 97 * usize::try_from(seed).expect("小"), seed);
        let mut writer = PoolWriter::new(&pool.parameters, pool.devices.as_mut_slice());
        pool.first = publish_overwrite(
            &mut writer,
            &mut pool.allocator,
            &pool.first,
            FirstFile {
                content: &content,
                write_time_seconds: WRITE_TIME_SECONDS + 60,
            },
            pool.instance,
        )
        .expect("覆盖写");
    }
    pool
}

/// 根环里最新那条根指着的实例表第 0 片（两块盘各一份、各两槽）：这个进程里没写过行，是 mkfs 那一片。
fn instance_table_ranges_of_the_newest_root(
    pool: &PoolAfterTheFirstFile,
) -> [(DeviceIdentity, u64); 2] {
    let (_, newest) = newest_root_in_the_ring(&pool.devices);
    newest
        .instance_table
        .locations
        .map(|location| (location.device, location.slot.to_device_offset().0))
}

/// 抬 F 到 3：交回结局，池换回里面那几块盘。
fn raise_the_floor_to_three_through(
    pool: &mut PoolAfterTheFirstFile,
    ranges: Vec<ChosenReadFaultsOfARange>,
) -> Result<singlefs_core::mount::RaisedFloor, MountError> {
    let parameters = pool.parameters.clone();
    let mut faulty = with_chosen_read_faults(std::mem::take(&mut pool.devices), ranges);
    let mut current = pool.first.clone();
    let raised = raise_rollback_floor(
        &parameters,
        &mut faulty,
        &mut pool.allocator,
        &mut current,
        CheckpointTxg(3),
        ShadowLedger::On,
    );
    pool.devices = without_chosen_read_faults(faulty);
    raised
}

/// 实例表那一段每块盘各两槽（32 KiB）。
const INSTANCE_TABLE_BYTES: u64 = 2 * SLOT_BYTES;

/// 抬 F 那一串先读现行那一版的实例表（这一段每块盘的第 1 次读，读盘 0 那一份就够），再算生效 F 时读根环里最新那条根的实例表
/// （同一片：盘 0 第 2 次、盘 1 第 1 次），重读一次（盘 0 第 3 次、盘 1 第 2 次）。点名这两遍两份都坏：拒成
/// `StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheEffectiveFloor`（经 `MountError::NewerStateStillUnreadableAfterOneReread`，
/// 与乙那一族同级），在动分配器与任何写之前——两块盘逐字节不变、分配器与抬 F 之前逐项相同。
/// 改之前「不按表滤」往下走，这一次抬照做。
#[test]
fn raising_the_floor_while_the_newest_roots_instance_table_stays_unreadable_is_refused_before_any_write(
) {
    let mut pool = pool_after_three_overwrites();
    let [(device_zero, offset_zero), (device_one, offset_one)] =
        instance_table_ranges_of_the_newest_root(&pool);
    let images_before = images_of(&pool.devices);
    let allocator_before = format!("{:?}", pool.allocator);
    let refused = raise_the_floor_to_three_through(
        &mut pool,
        vec![
            chosen_read_faults_of(device_zero, offset_zero, INSTANCE_TABLE_BYTES, &[2, 3]),
            chosen_read_faults_of(device_one, offset_one, INSTANCE_TABLE_BYTES, &[1, 2]),
        ],
    );
    assert!(
        matches!(
            &refused,
            Err(MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable))
                if **still_unreadable
                    == StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheEffectiveFloor {
                        newest_root_on_the_reread: Some(RollbackTarget {
                            instance: InstanceGeneration(1),
                            checkpoint_txg: CheckpointTxg(6),
                        }),
                    }
        ),
        "{:?}",
        refused.as_ref().err()
    );
    assert_eq!(images_of(&pool.devices), images_before, "两块盘逐字节不变");
    assert_eq!(
        format!("{:?}", pool.allocator),
        allocator_before,
        "分配器与抬 F 之前逐项相同"
    );
}

/// 同一片只在算生效 F 的第一遍读坏（盘 0 第 2 次、盘 1 第 1 次），重读那一遍读得出：按读出来的表算，这一次抬照做。
#[test]
fn the_newest_roots_instance_table_read_on_the_one_reread_lets_the_floor_be_raised() {
    let mut pool = pool_after_three_overwrites();
    let [(device_zero, offset_zero), (device_one, offset_one)] =
        instance_table_ranges_of_the_newest_root(&pool);
    let raised = raise_the_floor_to_three_through(
        &mut pool,
        vec![
            chosen_read_faults_of(device_zero, offset_zero, INSTANCE_TABLE_BYTES, &[2]),
            chosen_read_faults_of(device_one, offset_one, INSTANCE_TABLE_BYTES, &[1]),
        ],
    );
    assert!(raised.is_ok(), "重读读得出，抬 F 做成：{:?}", raised.err());
}

/// 根环里最新那条根住的槽：挂载（这里是 mkfs 同一个进程）之后这个进程写过、FUA 返回过，在这个进程的根环表里。抬 F 到 3 那一串读它：
/// 第 1 次是算 F 生效值读根环，第 2 次是算上限读根环，都读得出；第 3 次是重算影子账与回收门槛读根环，读坏，重读（第 4 次）仍坏 ⇒
/// 拒成 `StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot`（经 `MountError::NewerStateStillUnreadableAfterOneReread`），
/// 在动分配器与任何写之前。改之前（A3b 之前）当那一槽没有根、这一次抬照做。
/// 合入后验证一改了造法：原来坏的是从没写过的最后一槽，D16（发布语义） 已定项 1「根槽这一次读坏」那一行要它当没有根、不拒
/// （[`raising_the_floor_while_a_root_ring_slot_never_written_stays_unreadable_counts_it_as_no_root`]）。
#[test]
fn raising_the_floor_while_a_root_ring_slot_known_to_hold_a_root_stays_unreadable_is_refused_before_any_write(
) {
    let mut pool = pool_after_three_overwrites();
    let sizes = pool.parameters.geometry;
    let (newest_slot, _) = newest_root_in_the_ring(&pool.devices);
    assert!(
        ring_slots_known_to_hold_a_root_by(&pool.allocator).contains(&newest_slot),
        "最新那条根住的槽在这个进程的根环表里"
    );
    let slot_device =
        pool.parameters.region_devices[usize::try_from(newest_slot.region).expect("区域号")];
    let slot_start = slot_offset(newest_slot, sizes.fixed_structure_slot_spacing).0;
    let images_before = images_of(&pool.devices);
    let allocator_before = format!("{:?}", pool.allocator);
    let refused = raise_the_floor_to_three_through(
        &mut pool,
        vec![chosen_read_faults_of(
            slot_device,
            slot_start,
            u64::from(sizes.physical_block_size),
            &[3, 4],
        )],
    );
    assert!(
        matches!(
            &refused,
            Err(MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable))
                if **still_unreadable
                    == StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot {
                        ring_slot: newest_slot,
                        first_reading: BadRootRingSlotReading::Unreadable,
                        reread: BadRootRingSlotReading::Unreadable,
                    }
        ),
        "{:?}",
        refused.as_ref().err()
    );
    assert_eq!(images_of(&pool.devices), images_before, "两块盘逐字节不变");
    assert_eq!(
        format!("{:?}", pool.allocator),
        allocator_before,
        "分配器与抬 F 之前逐项相同"
    );
}

/// 同一槽只在重算影子账那一遍的第一次读坏（第 3 次），重读（第 4 次）读得出：照读出来的根算，这一次抬照做。
#[test]
fn a_root_ring_slot_known_to_hold_a_root_read_on_the_one_reread_lets_the_floor_be_raised() {
    let mut pool = pool_after_three_overwrites();
    let sizes = pool.parameters.geometry;
    let (newest_slot, _) = newest_root_in_the_ring(&pool.devices);
    let slot_device =
        pool.parameters.region_devices[usize::try_from(newest_slot.region).expect("区域号")];
    let slot_start = slot_offset(newest_slot, sizes.fixed_structure_slot_spacing).0;
    let raised = raise_the_floor_to_three_through(
        &mut pool,
        vec![chosen_read_faults_of(
            slot_device,
            slot_start,
            u64::from(sizes.physical_block_size),
            &[3],
        )],
    );
    assert!(raised.is_ok(), "重读读得出，抬 F 做成：{:?}", raised.err());
}

/// 根环里一个从没写过的槽（最后一个区域的最后一槽）：挂载那一刻就自证不过、这个进程之后也没写过，不在这个进程的根环表里。
/// 抬 F 到 3 那一串里它的每一次读都坏：D16（发布语义） 已定项 1「根槽这一次读坏」那一行要它当没有根——不重读、不拒，这一次抬照做
/// （善后一报告 P1；A3b 那一版对每个读不出的槽都重读、仍读不出就拒）。
#[test]
fn raising_the_floor_while_a_root_ring_slot_never_written_stays_unreadable_counts_it_as_no_root() {
    let mut pool = pool_after_three_overwrites();
    let sizes = pool.parameters.geometry;
    let last_slot = *every_root_ring_slot(&sizes).last().expect("根环至少一槽");
    assert!(
        !ring_slots_known_to_hold_a_root_by(&pool.allocator).contains(&last_slot),
        "最后一槽没写过，不在这个进程的根环表里"
    );
    let slot_device =
        pool.parameters.region_devices[usize::try_from(last_slot.region).expect("区域号")];
    let slot_start = slot_offset(last_slot, sizes.fixed_structure_slot_spacing).0;
    let every_read_number: Vec<u64> = (1..=64).collect();
    let raised = raise_the_floor_to_three_through(
        &mut pool,
        vec![chosen_read_faults_of(
            slot_device,
            slot_start,
            u64::from(sizes.physical_block_size),
            &every_read_number,
        )],
    );
    assert!(
        raised.is_ok(),
        "从没写过的槽读不出当没有根，抬 F 做成：{:?}",
        raised.err()
    );
}

/// 读根环逐槽重读那一段本身：这个进程知道住着根的槽（最新那条根住的）第一次读坏、重读读得出，交回的根里有它；两次都坏，报那一槽；
/// 不在这个进程根环表里的槽（从没写过的最后一槽）每一次都坏，当没有根、不报错，交回的根与没有读故障时相同。
#[test]
fn a_root_ring_slot_unreadable_once_is_read_on_the_reread_and_one_unreadable_twice_is_named() {
    let pool = pool_after_three_overwrites();
    let sizes = pool.parameters.geometry;
    let known = ring_slots_known_to_hold_a_root_by(&pool.allocator);
    let (newest_slot, newest) = newest_root_in_the_ring(&pool.devices);
    let slot_device =
        pool.parameters.region_devices[usize::try_from(newest_slot.region).expect("区域号")];
    let slot_start = slot_offset(newest_slot, sizes.fixed_structure_slot_spacing).0;
    let slot_length = u64::from(sizes.physical_block_size);
    let read_once_badly = with_chosen_read_faults(
        copy_of_the_devices(&pool.devices),
        vec![chosen_read_faults_of(
            slot_device,
            slot_start,
            slot_length,
            &[1],
        )],
    );
    let roots = readable_roots_rereading_ring_slots_known_to_hold_a_root_once(
        &read_once_badly,
        &pool.parameters.region_devices,
        &sizes,
        &pool.parameters.filesystem_identifier,
        &known,
    )
    .expect("重读读得出");
    assert!(roots.contains(&newest), "重读那一遍认出了最新那条根");
    let read_twice_badly = with_chosen_read_faults(
        copy_of_the_devices(&pool.devices),
        vec![chosen_read_faults_of(
            slot_device,
            slot_start,
            slot_length,
            &[1, 2],
        )],
    );
    assert_eq!(
        readable_roots_rereading_ring_slots_known_to_hold_a_root_once(
            &read_twice_badly,
            &pool.parameters.region_devices,
            &sizes,
            &pool.parameters.filesystem_identifier,
            &known,
        ),
        Err(RootRingSlotKnownToHoldARootStillBadAfterOneReread {
            ring_slot: newest_slot,
            first_reading: BadRootRingSlotReading::Unreadable,
            reread: BadRootRingSlotReading::Unreadable,
        })
    );
    let last_slot = *every_root_ring_slot(&sizes).last().expect("根环至少一槽");
    assert!(!known.contains(&last_slot), "最后一槽没写过");
    let last_slot_device =
        pool.parameters.region_devices[usize::try_from(last_slot.region).expect("区域号")];
    let never_written_slot_always_bad = with_chosen_read_faults(
        copy_of_the_devices(&pool.devices),
        vec![chosen_read_faults_of(
            last_slot_device,
            slot_offset(last_slot, sizes.fixed_structure_slot_spacing).0,
            slot_length,
            &[1, 2, 3],
        )],
    );
    assert_eq!(
        readable_roots_rereading_ring_slots_known_to_hold_a_root_once(
            &never_written_slot_always_bad,
            &pool.parameters.region_devices,
            &sizes,
            &pool.parameters.filesystem_identifier,
            &known,
        ),
        Ok(readable_roots(
            &pool.devices,
            &pool.parameters.region_devices,
            &sizes,
            &pool.parameters.filesystem_identifier,
        )),
        "不在根环表里的槽读不出当没有根"
    );
}

/// 算生效 F 那一段本身：最新那条根的实例表第一遍两份都读坏、重读读得出，按表算出来的 F 与没有读故障时相同；两遍都坏报错。
#[test]
fn the_effective_floor_rereads_the_newest_roots_instance_table_once() {
    let pool = pool_after_three_overwrites();
    let [(device_zero, offset_zero), (device_one, offset_one)] =
        instance_table_ranges_of_the_newest_root(&pool);
    let floor_of = |devices: &Vec<(DeviceIdentity, DeviceFailingChosenReads)>| {
        effective_rollback_floor_rereading_the_newest_instance_table_once(
            devices,
            &pool.parameters.region_devices,
            &pool.parameters.geometry,
            &pool.parameters.filesystem_identifier,
        )
    };
    let without_faults = with_chosen_read_faults(copy_of_the_devices(&pool.devices), Vec::new());
    let expected = floor_of(&without_faults).expect("没有读故障");
    let first_read_bad = with_chosen_read_faults(
        copy_of_the_devices(&pool.devices),
        vec![
            chosen_read_faults_of(device_zero, offset_zero, INSTANCE_TABLE_BYTES, &[1]),
            chosen_read_faults_of(device_one, offset_one, INSTANCE_TABLE_BYTES, &[1]),
        ],
    );
    assert_eq!(floor_of(&first_read_bad), Ok(expected), "重读读得出");
    let both_reads_bad = with_chosen_read_faults(
        copy_of_the_devices(&pool.devices),
        vec![
            chosen_read_faults_of(device_zero, offset_zero, INSTANCE_TABLE_BYTES, &[1, 2]),
            chosen_read_faults_of(device_one, offset_one, INSTANCE_TABLE_BYTES, &[1, 2]),
        ],
    );
    assert_eq!(
        floor_of(&both_reads_bad),
        Err(InstanceTableOfTheNewestRootStillUnreadableAfterOneReread {
            newest_root_on_the_reread: Some((InstanceGeneration(1), CheckpointTxg(6))),
        })
    );
}

/// 管理员回退 txg 5 那一版（实例 1）的结局，盘换回里面那几块。目标那条根住的槽在这个进程的根环表里。
fn roll_back_to_txg_five_through(
    pool: &mut PoolAfterTheFirstFile,
    ranges_of_the_target_slot: impl Fn(DeviceIdentity, u64, u64) -> Vec<ChosenReadFaultsOfARange>,
) -> Result<singlefs_core::mount::RolledBack, RollbackError> {
    let sizes = pool.parameters.geometry;
    let target = RollbackTarget {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(5),
    };
    let (target_slot, _) = readable_roots_with_ring_slots(
        &pool.devices,
        &pool.parameters.region_devices,
        &sizes,
        &pool.parameters.filesystem_identifier,
    )
    .into_iter()
    .find(|(_, root)| {
        root.instance == target.instance && root.checkpoint_txg == target.checkpoint_txg
    })
    .expect("txg 5 那条根在环里");
    assert!(
        ring_slots_known_to_hold_a_root_by(&pool.allocator).contains(&target_slot),
        "目标住的槽在这个进程的根环表里"
    );
    let slot_device =
        pool.parameters.region_devices[usize::try_from(target_slot.region).expect("区域号")];
    let slot_start = slot_offset(target_slot, sizes.fixed_structure_slot_spacing).0;
    let parameters = pool.parameters.clone();
    let mut faulty = with_chosen_read_faults(
        std::mem::take(&mut pool.devices),
        ranges_of_the_target_slot(
            slot_device,
            slot_start,
            u64::from(sizes.physical_block_size),
        ),
    );
    let mut current = pool.first.clone();
    let rolled_back = roll_back_by_a_forward_publish(
        &parameters,
        &mut faulty,
        &mut pool.allocator,
        &mut current,
        target,
    );
    pool.devices = without_chosen_read_faults(faulty);
    rolled_back
}

/// 管理员回退判候选（实审 A3b 报告 Q7，合入后验证一）：两处读根环都照 D16（发布语义） 已定项 1「根槽这一次读坏」那一行的读法。
/// 目标那条根住的槽：判「根环里有它」那一遍读坏（第 1 次）、重读（第 2 次）仍坏；或那一遍读得出、算 F_生效 那一遍读坏（第 2 次）、
/// 重读（第 3 次）仍坏 ⇒ 都拒成 `RollbackError::CandidateJudgementStillUnreadableAfterOneReread(RootRingSlotKnownToHoldARoot)`，
/// 在任何写之前：两块盘逐字节不变、分配器不动。改之前第一格当那一槽没有根、报目标不在环里；第二格 F_生效 少算一条根、回退照做。
/// 只在第一遍读坏（第 1 次）、重读读得出：回退照做。
#[test]
fn rolling_back_while_the_targets_root_ring_slot_stays_unreadable_is_refused_before_any_write() {
    for (failing_read_numbers, what) in [
        (&[1_u64, 2][..], "判根环里有它那一遍"),
        (&[2, 3][..], "算 F_生效 那一遍"),
    ] {
        let mut pool = pool_after_three_overwrites();
        let images_before = images_of(&pool.devices);
        let allocator_before = format!("{:?}", pool.allocator);
        let refused = roll_back_to_txg_five_through(&mut pool, |device, offset, length| {
            vec![chosen_read_faults_of(
                device,
                offset,
                length,
                failing_read_numbers,
            )]
        });
        assert!(
            matches!(
                &refused,
                Err(RollbackError::CandidateJudgementStillUnreadableAfterOneReread(still_unreadable))
                    if matches!(
                        **still_unreadable,
                        StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot { .. }
                    )
            ),
            "{what}：{:?}",
            refused.as_ref().err()
        );
        assert_eq!(
            images_of(&pool.devices),
            images_before,
            "{what}：两块盘逐字节不变"
        );
        assert_eq!(
            format!("{:?}", pool.allocator),
            allocator_before,
            "{what}：分配器不动"
        );
    }
    let mut pool = pool_after_three_overwrites();
    let rolled_back = roll_back_to_txg_five_through(&mut pool, |device, offset, length| {
        vec![chosen_read_faults_of(device, offset, length, &[1])]
    });
    assert!(
        rolled_back.is_ok(),
        "重读读得出，回退做成：{:?}",
        rolled_back.err()
    );
}

/// 系统配置槽里回退下界 F 那 8 字节的偏移（`SystemConfiguration::to_slot` 的写法，小端）。
const ROLLBACK_FLOOR_OFFSET_IN_THE_SYSTEM_CONFIGURATION_SLOT: usize = 481;

/// 写系统配置槽之前算 F 生效值（`transaction::PoolWriter`，实审 A3b 报告 Q7，合入后验证一）：读不出的根槽重读一次。
/// 造法：抬 F 到 3（那一串的根带 F 3、落满两块盘），再把两块盘四槽系统配置里的 F 改回 0、重封——一份违反 I-7.12（系统配置 F 不低于
/// 同盘根上的 F） 的改出来的镜像，只为让根上的 F 成为生效值里唯一带 3 的来源。之后取号：先读一遍根环算要取的号（带 F 3 的每条根住的槽
/// 第 1 次读），再逐盘写一次系统配置槽、每一写之前算一遍生效值读一遍根环。点名那几个槽第 2、4 次读坏：两次都重读（第 3、5 次）读得出，
/// 两块盘新写的那一槽都带 F 3。改之前（不重读）第一块盘那一写读不到带 F 3 的根，写成 0。
#[test]
fn a_system_configuration_write_rereads_an_unreadable_root_ring_slot_once_before_taking_the_effective_floor(
) {
    let mut pool = pool_after_three_overwrites();
    raise_the_floor_to_three_through(&mut pool, Vec::new()).expect("抬 F 到 3");
    let parameters = pool.parameters.clone();
    rewrite_every_system_configuration_slot(&mut pool.devices, &parameters, |slot| {
        slot[ROLLBACK_FLOOR_OFFSET_IN_THE_SYSTEM_CONFIGURATION_SLOT
            ..ROLLBACK_FLOOR_OFFSET_IN_THE_SYSTEM_CONFIGURATION_SLOT + 8]
            .copy_from_slice(&0_u64.to_le_bytes());
    });
    let sizes = parameters.geometry;
    let slots_of_the_roots_carrying_three: Vec<RootRingSlot> = readable_roots_with_ring_slots(
        &pool.devices,
        &parameters.region_devices,
        &sizes,
        &parameters.filesystem_identifier,
    )
    .into_iter()
    .filter(|(_, root)| root.rollback_floor == CheckpointTxg(3))
    .map(|(ring_slot, _)| ring_slot)
    .collect();
    assert!(
        slots_of_the_roots_carrying_three.len() >= 2,
        "抬 F 那一串落满两块盘"
    );
    assert_eq!(
        effective_rollback_floor_rereading_unreadable_reads_once(
            &pool.devices,
            &parameters.region_devices,
            &sizes,
            &parameters.filesystem_identifier,
        ),
        CheckpointTxg(3),
        "没有读故障时生效值只从根上来"
    );
    let ranges = slots_of_the_roots_carrying_three
        .iter()
        .map(|ring_slot| {
            chosen_read_faults_of(
                parameters.region_devices[usize::try_from(ring_slot.region).expect("区域号")],
                slot_offset(*ring_slot, sizes.fixed_structure_slot_spacing).0,
                u64::from(sizes.physical_block_size),
                &[2, 4],
            )
        })
        .collect();
    let mut faulty = with_chosen_read_faults(std::mem::take(&mut pool.devices), ranges);
    let mut writer = PoolWriter::new(&parameters, faulty.as_mut_slice());
    acquire_instance(&mut writer).expect("取号");
    pool.devices = without_chosen_read_faults(faulty);
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let newest_written = verified_system_configuration_slots(
            &pool.devices,
            device,
            u64::from(sizes.fixed_structure_slot_spacing),
            &parameters.filesystem_identifier,
        )
        .into_iter()
        .max_by_key(|system_configuration| system_configuration.quantities.slot_generation)
        .expect("这块盘刚写过一槽");
        assert_eq!(
            newest_written.quantities.rollback_floor,
            CheckpointTxg(3),
            "盘 {} 取号写的那一槽带的 F",
            device.0
        );
    }
}

/// 抬 F 到 3 那一串里最新那条根住的槽（这个进程知道住着根）：第 3 次读（重算影子账那一遍）读得出而自证不过、重读（第 4 次）仍自证不过 ⇒
/// 拒成 `StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot { first_reading: NotSelfVerified, reread: NotSelfVerified }`，
/// 在动分配器与任何写之前（D16（发布语义） 已定项 1「根槽这一次读坏」那一行全句：「读不出或自证不过就重读一次，仍坏就拒」）。
/// 合入后验证一那一版只管「读不出」，自证不过当没有根、这一次抬照做。
#[test]
fn raising_the_floor_while_a_root_ring_slot_known_to_hold_a_root_stays_not_self_verified_is_refused_before_any_write(
) {
    let mut pool = pool_after_three_overwrites();
    let sizes = pool.parameters.geometry;
    let (newest_slot, _) = newest_root_in_the_ring(&pool.devices);
    assert!(
        ring_slots_known_to_hold_a_root_by(&pool.allocator).contains(&newest_slot),
        "最新那条根住的槽在这个进程的根环表里"
    );
    let slot_device =
        pool.parameters.region_devices[usize::try_from(newest_slot.region).expect("区域号")];
    let slot_start = slot_offset(newest_slot, sizes.fixed_structure_slot_spacing).0;
    let images_before = images_of(&pool.devices);
    let allocator_before = format!("{:?}", pool.allocator);
    let refused = raise_the_floor_to_three_through(
        &mut pool,
        vec![chosen_corrupted_reads_of(
            slot_device,
            slot_start,
            u64::from(sizes.physical_block_size),
            &[3, 4],
        )],
    );
    assert!(
        matches!(
            &refused,
            Err(MountError::NewerStateStillUnreadableAfterOneReread(still_bad))
                if **still_bad
                    == StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot {
                        ring_slot: newest_slot,
                        first_reading: BadRootRingSlotReading::NotSelfVerified,
                        reread: BadRootRingSlotReading::NotSelfVerified,
                    }
        ),
        "{:?}",
        refused.as_ref().err()
    );
    assert_eq!(images_of(&pool.devices), images_before, "两块盘逐字节不变");
    assert_eq!(
        format!("{:?}", pool.allocator),
        allocator_before,
        "分配器与抬 F 之前逐项相同"
    );
}

/// 同一槽只在重算影子账那一遍的第一次读（第 3 次）自证不过，重读（第 4 次）自证得过：照读出来的根算，这一次抬照做。
#[test]
fn a_root_ring_slot_known_to_hold_a_root_not_self_verified_once_is_read_on_the_reread_and_the_floor_is_raised(
) {
    let mut pool = pool_after_three_overwrites();
    let sizes = pool.parameters.geometry;
    let (newest_slot, _) = newest_root_in_the_ring(&pool.devices);
    let slot_device =
        pool.parameters.region_devices[usize::try_from(newest_slot.region).expect("区域号")];
    let slot_start = slot_offset(newest_slot, sizes.fixed_structure_slot_spacing).0;
    let raised = raise_the_floor_to_three_through(
        &mut pool,
        vec![chosen_corrupted_reads_of(
            slot_device,
            slot_start,
            u64::from(sizes.physical_block_size),
            &[3],
        )],
    );
    assert!(
        raised.is_ok(),
        "重读自证得过，抬 F 做成：{:?}",
        raised.err()
    );
}

/// 管理员回退算水位（D23（journal 的角色与格式） 已定项 14「水位」：inode 号水位 = max(现行那一版内存里的, 环里读得出的根的)）：
/// txg 7 那一版新建三个 inode（号 2、3、4，水位 5），环里别的根的 inode 号水位都是 2。回退到 txg 5 时 txg 7 的根槽判候选那两遍
/// （第 1、2 次读）读得出，算水位那一遍（第 3 次，`readable_roots`，不重读）瞬时读不出：回退那次发布的 inode 号水位仍是 5，
/// 之后新建的 inode 不重发 2–4。只取环里读得出的根时是 2——`crates/mutations.tsv` 里那一行（实三「水位」）证这条断言红。
/// 这一格原来钉在 `rollback_by_a_forward_publish.rs` 那一条（C 的根槽在回退之前改坏），D16（发布语义） 已定项 1「根槽这一次读坏」
/// 全句接进判候选之后那一形在判候选时就被拒（那一条改钉拒），水位这一判挪到这里：判候选读得出、算水位那一遍读不出。
#[test]
fn the_rollback_takes_the_inode_number_watermark_from_the_current_version_in_memory_when_the_watermark_read_misses_its_root(
) {
    let mut pool = pool_after_three_overwrites();
    let sizes = pool.parameters.geometry;
    let with_new_inodes = {
        let mut writer = PoolWriter::new(&pool.parameters, pool.devices.as_mut_slice());
        publish_new_inodes(
            &mut writer,
            &mut pool.allocator,
            &pool.first,
            3,
            WRITE_TIME_SECONDS + 120,
            pool.instance,
        )
        .expect("新建三个 inode")
    };
    assert_eq!(with_new_inodes.root.checkpoint_txg, CheckpointTxg(7));
    assert_eq!(with_new_inodes.inode_number_watermark(), 5);
    let (newest_slot, newest) = newest_root_in_the_ring(&pool.devices);
    assert_eq!(
        newest.checkpoint_txg,
        CheckpointTxg(7),
        "最新那条根是新建 inode 那一版的"
    );
    let slot_device =
        pool.parameters.region_devices[usize::try_from(newest_slot.region).expect("区域号")];
    let slot_start = slot_offset(newest_slot, sizes.fixed_structure_slot_spacing).0;
    let parameters = pool.parameters.clone();
    let mut faulty = with_chosen_read_faults(
        std::mem::take(&mut pool.devices),
        vec![chosen_read_faults_of(
            slot_device,
            slot_start,
            u64::from(sizes.physical_block_size),
            &[3],
        )],
    );
    let mut current = with_new_inodes;
    roll_back_by_a_forward_publish(
        &parameters,
        &mut faulty,
        &mut pool.allocator,
        &mut current,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(5),
        },
    )
    .expect("判候选两遍读得出，回退做成");
    pool.devices = without_chosen_read_faults(faulty);
    assert_eq!(
        current.root.checkpoint_txg,
        CheckpointTxg(8),
        "回退那次发布"
    );
    assert_eq!(
        current.inode_number_watermark(),
        5,
        "回退那次发布的 inode 号水位 = max(现行那一版内存里的 5, 环里读得出的根的 2)"
    );
}

// ─── 五、core 读系统配置的 journal 环起点与根环起点（代码三方 m2-closeout-code-r2 Y4-a） ───

/// 系统配置槽里 journal 环起点那 8 字节（16 KiB 槽号，字段表 `layout/01-first-txn.md` 一）。
const JOURNAL_RING_START_SLOT_OFFSET: usize = 325;
/// 系统配置槽里根环起点那 8 字节（16 KiB 槽号，D22（单元原子性怎么合成） 已定项 16 第 1 句）。
const ROOT_RING_BASE_SLOT_OFFSET: usize = 371;

/// 攻方 Y4 那五格（`research/prompts/m2-closeout-code-r2-opus-model/opus_r2_y4_system_configuration_fields_core_and_checker_read_differently.rs`）：
/// 第一个文件之后那份合法镜像，四个系统配置槽里改一个字段、整槽校验和重封。每一格改哪几个字段、core 该拒成哪一个值。
struct SystemConfigurationFieldCell {
    name: &'static str,
    fields_written: Vec<(usize, u64)>,
    refused_as: SystemConfigurationValueOutsideWhatThisReaderAccepts,
}

fn system_configuration_field_cells() -> Vec<SystemConfigurationFieldCell> {
    let ring_off_the_segment_boundary = JOURNAL_RING_DEFAULT_BYTES - SLOT_BYTES;
    let start_after_the_ring_off_the_boundary =
        JOURNAL_RING_START_SLOT + ring_off_the_segment_boundary / SLOT_BYTES;
    vec![
        SystemConfigurationFieldCell {
            name: "A：单元区起点改成现算起点 + 64",
            fields_written: vec![(UNIT_AREA_START_SLOT_OFFSET, 50_240)],
            refused_as: SystemConfigurationValueOutsideWhatThisReaderAccepts::UnitAreaStartNotTheSlotAfterTheJournalRing {
                recorded_unit_area_start_slot: SlotNumber(50_240),
                slot_after_the_journal_ring: SlotNumber(50_176),
            },
        },
        SystemConfigurationFieldCell {
            name: "B：journal 环起点 1024 改成 1023",
            fields_written: vec![(JOURNAL_RING_START_SLOT_OFFSET, 1023)],
            refused_as: SystemConfigurationValueOutsideWhatThisReaderAccepts::JournalRingStartNotTheFirstVersionSlot {
                recorded_journal_ring_start_slot: SlotNumber(1023),
                first_version_journal_ring_start_slot: SlotNumber(JOURNAL_RING_START_SLOT),
            },
        },
        SystemConfigurationFieldCell {
            name: "C：根环起点 64 改成 65",
            fields_written: vec![(ROOT_RING_BASE_SLOT_OFFSET, 65)],
            refused_as: SystemConfigurationValueOutsideWhatThisReaderAccepts::RootRingBaseNotTheFirstVersionSlot {
                recorded_root_ring_base_slot: SlotNumber(65),
                first_version_root_ring_base_slot: SlotNumber(ROOT_RING_BASE_SLOT),
            },
        },
        SystemConfigurationFieldCell {
            name: "D：环长改成 768 MiB − 16 KiB、单元区起点跟着改成它现算的 50175（不在段边界上）",
            fields_written: vec![
                (JOURNAL_RING_BYTES_OFFSET, ring_off_the_segment_boundary),
                (UNIT_AREA_START_SLOT_OFFSET, start_after_the_ring_off_the_boundary),
            ],
            refused_as: SystemConfigurationValueOutsideWhatThisReaderAccepts::UnitAreaStartOffTheClusterSegmentBoundaryUnsupported {
                unit_area_start_slot: SlotNumber(start_after_the_ring_off_the_boundary),
            },
        },
        SystemConfigurationFieldCell {
            name: "E：根环起点 64 改成 0（槽 1 不再整槽落在同一槽自述的根环起点之前）",
            fields_written: vec![(ROOT_RING_BASE_SLOT_OFFSET, 0)],
            refused_as: SystemConfigurationValueOutsideWhatThisReaderAccepts::FixedStructureSlotSpacingOutsideTheFormatRange {
                fixed_structure_slot_spacing: 4096,
            },
        },
    ]
}

/// 五格 core 都整池拒，只读挂载与可写挂载拒成同一个值（`RecoveryFailure::SystemConfigurationValueRefused`），两块盘逐字节不变。
/// 改之前 core 解槽跳过 325 与 371、取编译期常量：B、C、E 三格只读与可写都做成（E 格 checker 报 I-7.13「实现整池拒绝挂载」而实现照挂）。
#[test]
fn a_system_configuration_field_the_reader_does_not_accept_is_refused_by_both_mounts_before_any_write(
) {
    for cell in system_configuration_field_cells() {
        let mut pool = pool_after_the_first_file(JOURNAL_RING_DEFAULT_BYTES, FOUR_GIBIBYTES);
        let parameters = pool.parameters.clone();
        rewrite_every_system_configuration_slot(&mut pool.devices, &parameters, |slot| {
            for (offset, value) in &cell.fields_written {
                slot[*offset..*offset + 8].copy_from_slice(&value.to_le_bytes());
            }
        });
        let images_before = images_of(&pool.devices);
        let read_only = mount_read_only(&pool.devices);
        assert!(
            matches!(
                &read_only,
                Err(MountReadOnlyFailure::Recovery(RecoveryFailure::SystemConfigurationValueRefused {
                    value,
                    ..
                })) if *value == cell.refused_as
            ),
            "{}：只读挂载 {:?}",
            cell.name,
            read_only.as_ref().err()
        );
        let writable = mount_writable(&parameters, &mut pool.devices);
        assert!(
            matches!(
                &writable,
                Err(MountError::Recovery(RecoveryFailure::SystemConfigurationValueRefused {
                    value,
                    ..
                })) if *value == cell.refused_as
            ),
            "{}：可写挂载 {:?}",
            cell.name,
            writable.as_ref().err()
        );
        assert_eq!(
            images_of(&pool.devices),
            images_before,
            "{}：两块盘逐字节不变",
            cell.name
        );
    }
}

/// 解槽读回 325 与 371 两个字段：mkfs 写的是第一版常量，择到的系统配置里两个字段就是它们（没改过的镜像照挂）。
#[test]
fn the_chosen_system_configuration_carries_the_first_version_journal_ring_start_and_root_ring_base()
{
    let pool = pool_after_the_first_file(JOURNAL_RING_DEFAULT_BYTES, FOUR_GIBIBYTES);
    let chosen = choose_system_configuration(&pool.devices).expect("没改过的镜像择得到");
    assert_eq!(
        chosen.immutable.journal_ring_start_slot,
        SlotNumber(JOURNAL_RING_START_SLOT)
    );
    assert_eq!(
        chosen.immutable.root_ring_base_slot,
        SlotNumber(ROOT_RING_BASE_SLOT)
    );
}

/// 一池内存盘的一份拷贝（盘宽与镜像都照抄）：读故障包在拷贝外面，原池不动。
fn copy_of_the_devices(devices: &[(DeviceIdentity, SparseBlockDevice)]) -> SparseDevices {
    devices
        .iter()
        .map(|(identity, device)| {
            let mut copy =
                SparseBlockDevice::new(device.size_in_bytes(), SECTOR_PHYSICAL_BLOCK_SIZE);
            copy.image = device.image.clone();
            (*identity, copy)
        })
        .collect()
}
