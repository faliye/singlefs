//! 代码审阅第 15、16、21、25、37 条（实审 A2a）的会红用例：
//! mkfs 在任何写之前拒掉固定结构互相盖住、根槽装不下根记录、journal 环短于一条记录或越过单元区起点的几何
//! （D23（journal 的角色与格式） 已定项 19 ③、D22（单元原子性怎么合成） 已定项 16、D2（RAID 条带策略） 已定项 19）；
//! 只读挂载打得开新建的空 inode；恢复照 I-8.6（反向链算法）把链值不等的记录挡在重放前缀之外；
//! 槽 0 自证不过的盘按池里别的盘槽 0 记的槽距找它的槽 1。
//! 盘都是内存稀疏盘（`singlefs_harness::crash::SparseBlockDevice`），恢复与只读挂载读的是 `MemoryPool`。

use singlefs_core::address::{
    DeviceIdentity, DeviceOffsetInBytes, InodeNumber, InstanceGeneration,
};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::journal::{record_offset, JournalRecord};
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, FixedStructure, MakeFilesystemError,
    MakeFilesystemOutput, MakeFilesystemParameters,
};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::recovery::{
    choose_system_configuration, recover, JournalPolicy, RecoveryOutcome,
};
use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlotsPerRegion};
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_new_inodes, publish_overwrite, warm_up,
    FirstFile, PoolWriter, TransactionOutput, FIRST_INODE_NUMBER,
};
use singlefs_format::{
    JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES, JOURNAL_RING_START_SLOT, ROOT_RECORD_BYTES,
    SLOT_BYTES, UNIT_AREA_START_SLOT,
};
use singlefs_harness::crash::{MemoryPool, SparseBlockDevice, SparseDevice};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

const DEVICE_BYTES: u64 = 4 << 30;
const MEBIBYTE: u64 = 1 << 20;
const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"singlefs-rev-a2a";
const WRITE_TIME_SECONDS: u64 = 1_788_000_000;
/// 内存稀疏盘按 512 字节扇区存：物理块宽 512。
const SECTOR_PHYSICAL_BLOCK_SIZE: PhysicalBlockSizeInBytes = PhysicalBlockSizeInBytes(512);
/// 拒绝之前先在每个固定结构的起点写的花样：稀疏盘上「清零」是删扇区，全零的盘上看不出 mkfs 清没清过。
const PATTERN_BYTE: u8 = 0xa5;
const PATTERN_BYTES: usize = 4096;
/// 系统配置槽里一个只有整槽校验和罩着的保留字节（与 `system_configuration_per_device_redundancy.rs` 同一处）：
/// 翻掉它，这一槽读得出、自证不过。
const BYTE_IN_THE_RESERVED_AREA_OF_A_SYSTEM_CONFIGURATION_SLOT: u64 = 2048;
/// journal 记录头里一个整条校验和罩着的字节（新根段里）：翻掉它，这条记录自证不过、扫环时不算在。
const BYTE_IN_THE_HEADER_OF_A_JOURNAL_RECORD: u64 = 300;

fn default_geometry() -> SystemImmutableSizes {
    SystemImmutableSizes {
        physical_block_size: 512,
        minimum_input_output_bytes: 512,
        fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(512),
        journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
        root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
    }
}

fn parameters_with(geometry: SystemImmutableSizes) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry,
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

/// 花样写在哪几处：系统配置槽 0、默认槽距与两倍默认槽距处的槽 1、三个根环区域起点、journal 环起点、单元区起点（装得下的才写）。
fn offsets_carrying_the_pattern(device_bytes: u64) -> Vec<u64> {
    [
        0,
        4096,
        8192,
        MEBIBYTE,
        4 * MEBIBYTE,
        7 * MEBIBYTE,
        JOURNAL_RING_START_SLOT * SLOT_BYTES,
        UNIT_AREA_START_SLOT * SLOT_BYTES,
    ]
    .into_iter()
    .filter(|offset| offset + u64::try_from(PATTERN_BYTES).expect("4096") <= device_bytes)
    .collect()
}

/// 在两块带花样的盘上做一次 mkfs：交回结果、录制流里记了几步、两块盘是不是与 mkfs 之前逐字节相同。
struct MakeFilesystemObservation {
    result: Result<MakeFilesystemOutput, MakeFilesystemError>,
    recorded_operations: usize,
    images_unchanged: bool,
}

fn make_filesystem_on_patterned_devices(
    parameters: &MakeFilesystemParameters,
    device_bytes: u64,
) -> MakeFilesystemObservation {
    let stream = SharedStream::new();
    let mut devices = Vec::new();
    let mut images_before: Vec<SparseDevice> = Vec::new();
    for identity in [DeviceIdentity(0), DeviceIdentity(1)] {
        let mut device = SparseBlockDevice::new(device_bytes, SECTOR_PHYSICAL_BLOCK_SIZE);
        for offset in offsets_carrying_the_pattern(device_bytes) {
            device
                .image
                .write(DeviceOffsetInBytes(offset), &[PATTERN_BYTE; PATTERN_BYTES]);
        }
        images_before.push(device.image.clone());
        devices.push((
            identity,
            RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
        ));
    }
    let result = make_filesystem(parameters, &mut devices);
    let images_unchanged = devices
        .iter()
        .zip(&images_before)
        .all(|((_, device), before)| device.inner().image == *before);
    MakeFilesystemObservation {
        result,
        recorded_operations: stream.operations().len(),
        images_unchanged,
    }
}

fn assert_refused_before_any_write(observation: &MakeFilesystemObservation, geometry_named: &str) {
    assert_eq!(
        observation.recorded_operations, 0,
        "{geometry_named}：拒绝之前录制流里一步都没有"
    );
    assert!(
        observation.images_unchanged,
        "{geometry_named}：两块盘与 mkfs 之前逐字节相同（花样一个扇区都没被清、没被盖）"
    );
}

// ── 第 15 条：journal 环长是 mkfs 参数，单元区起点仍是编译期常量 ──

/// 环 1 GiB（≤ 4 GiB ÷ 4，环长上界那一关放行）：环从 16 MiB 盖到 1040 MiB，越过单元区起点 784 MiB（实例表单元就写在那里）。
/// 单元区起点还不随环长走（分配器与恢复都按 `UNIT_AREA_START_SLOT` 算，C475），第一版在任何写之前拒掉。
#[test]
fn a_journal_ring_reaching_past_the_compiled_unit_area_start_is_refused_before_any_write() {
    let ring_bytes = 1 << 30;
    let observation = make_filesystem_on_patterned_devices(
        &parameters_with(SystemImmutableSizes {
            journal_ring_bytes: ring_bytes,
            ..default_geometry()
        }),
        DEVICE_BYTES,
    );
    assert!(
        matches!(
            observation.result,
            Err(MakeFilesystemError::JournalRingPastTheCompiledUnitAreaStartUnsupported {
                journal_ring_end_in_bytes,
                unit_area_start_in_bytes,
            }) if journal_ring_end_in_bytes == JOURNAL_RING_START_SLOT * SLOT_BYTES + ring_bytes
                && unit_area_start_in_bytes == UNIT_AREA_START_SLOT * SLOT_BYTES
        ),
        "1 GiB 的环盖住实例表单元：{:?}",
        observation.result.as_ref().err()
    );
    assert_refused_before_any_write(&observation, "1 GiB 的环");
}

/// 边界：环比默认值多一条记录（4096 字节，不是槽宽的整数倍）也越过单元区起点；恰好默认 768 MiB 时环末端正好是单元区起点，放行。
#[test]
fn a_journal_ring_one_record_longer_than_the_default_is_refused_and_the_default_ring_is_not() {
    let one_record_longer = make_filesystem_on_patterned_devices(
        &parameters_with(SystemImmutableSizes {
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES + JOURNAL_RECORD_BYTES,
            ..default_geometry()
        }),
        DEVICE_BYTES,
    );
    assert!(
        matches!(
            one_record_longer.result,
            Err(MakeFilesystemError::JournalRingPastTheCompiledUnitAreaStartUnsupported { .. })
        ),
        "默认环长加一条记录：{:?}",
        one_record_longer.result.as_ref().err()
    );
    assert_refused_before_any_write(&one_record_longer, "默认环长加一条记录");
    let default_ring =
        make_filesystem_on_patterned_devices(&parameters_with(default_geometry()), DEVICE_BYTES);
    assert!(
        default_ring.result.is_ok(),
        "默认 768 MiB 的环末端正好是单元区起点，不重叠：{:?}",
        default_ring.result.as_ref().err()
    );
}

/// 环短于一条记录：`journal::record_offset` 按环槽数取模，环槽数是 0。mkfs 在任何写之前拒掉；恰好一条记录那么长放行。
#[test]
fn a_journal_ring_shorter_than_one_record_is_refused_before_any_write() {
    let half_a_record = JOURNAL_RECORD_BYTES / 2;
    let observation = make_filesystem_on_patterned_devices(
        &parameters_with(SystemImmutableSizes {
            journal_ring_bytes: half_a_record,
            ..default_geometry()
        }),
        DEVICE_BYTES,
    );
    assert!(
        matches!(
            observation.result,
            Err(MakeFilesystemError::JournalRingShorterThanOneRecord {
                ring_bytes,
                record_bytes,
            }) if ring_bytes == half_a_record && record_bytes == JOURNAL_RECORD_BYTES
        ),
        "半条记录长的环：{:?}",
        observation.result.as_ref().err()
    );
    assert_refused_before_any_write(&observation, "半条记录长的环");
    let one_record = make_filesystem_on_patterned_devices(
        &parameters_with(SystemImmutableSizes {
            journal_ring_bytes: JOURNAL_RECORD_BYTES,
            ..default_geometry()
        }),
        DEVICE_BYTES,
    );
    assert!(
        one_record.result.is_ok(),
        "一条记录长的环放行：{:?}",
        one_record.result.as_ref().err()
    );
}

/// 100 MiB 的盘、16 MiB 的环：环长上界放行，而 mkfs 真正写实例表的单元区起点（784 MiB）在盘外。
/// 按 mkfs 实际写单元的那个起点判越界，在任何写之前拒掉，不在清完根环与 journal 环之后才撞上块设备的越界错。
#[test]
fn a_unit_area_starting_past_the_device_end_is_refused_before_any_write() {
    let device_bytes = 100 * MEBIBYTE;
    let observation = make_filesystem_on_patterned_devices(
        &parameters_with(SystemImmutableSizes {
            journal_ring_bytes: 16 * MEBIBYTE,
            ..default_geometry()
        }),
        device_bytes,
    );
    assert!(
        matches!(
            observation.result,
            Err(MakeFilesystemError::UnitAreaBeyondDevice {
                unit_area_start,
                device_bytes: refused_device_bytes,
            }) if unit_area_start == UNIT_AREA_START_SLOT * SLOT_BYTES
                && refused_device_bytes == device_bytes
        ),
        "单元区起点在盘外：{:?}",
        observation.result.as_ref().err()
    );
    assert_refused_before_any_write(&observation, "100 MiB 的盘");
}

// ── 第 16 条：固定结构几何互不重叠 ──

fn geometry_with_slot_spacing(fixed_structure_slot_spacing: u32) -> SystemImmutableSizes {
    SystemImmutableSizes {
        minimum_input_output_bytes: fixed_structure_slot_spacing,
        fixed_structure_slot_spacing,
        ..default_geometry()
    }
}

fn assert_overlap_refused(
    observation: &MakeFilesystemObservation,
    expected_first: FixedStructure,
    expected_second: FixedStructure,
    geometry_named: &str,
) {
    assert!(
        matches!(
            &observation.result,
            Err(MakeFilesystemError::FixedStructuresOverlap {
                first_in_layout_order,
                second_in_layout_order,
            }) if first_in_layout_order.structure == expected_first
                && second_in_layout_order.structure == expected_second
        ),
        "{geometry_named}：{:?}",
        observation.result.as_ref().err()
    );
    assert_refused_before_any_write(observation, geometry_named);
}

/// 槽距 1 MiB（io_min 1 MiB）：系统配置槽 1 落在 [1 MiB, 1 MiB + 4 KiB)，正是根环区域 0 的槽 0。
#[test]
fn a_system_configuration_slot_one_landing_on_root_ring_region_zero_is_refused_before_any_write() {
    let observation = make_filesystem_on_patterned_devices(
        &parameters_with(geometry_with_slot_spacing(1 << 20)),
        DEVICE_BYTES,
    );
    assert_overlap_refused(
        &observation,
        FixedStructure::SystemConfigurationSlot { slot_index: 1 },
        FixedStructure::RootRingRegion { region: 0 },
        "槽距 1 MiB",
    );
}

/// 槽距 1024（没走 io_min 的取整）：系统配置两槽 [0, 4096) 与 [1024, 5120) 重叠，写槽 1 撕掉槽 0。
#[test]
fn system_configuration_slots_closer_than_a_slot_width_are_refused_before_any_write() {
    let observation = make_filesystem_on_patterned_devices(
        &parameters_with(geometry_with_slot_spacing(1024)),
        DEVICE_BYTES,
    );
    assert_overlap_refused(
        &observation,
        FixedStructure::SystemConfigurationSlot { slot_index: 0 },
        FixedStructure::SystemConfigurationSlot { slot_index: 1 },
        "槽距 1024",
    );
}

/// 槽距的上界：区域之间隔 P × chunk = 3 MiB，S = 8 时槽距到 384 KiB 三个区域首尾相接、放行；再多 512 字节，区域 0 伸进区域 1。
#[test]
fn root_ring_regions_longer_than_the_region_step_are_refused_and_regions_that_just_touch_are_not() {
    let regions_that_just_touch = make_filesystem_on_patterned_devices(
        &parameters_with(geometry_with_slot_spacing(384 * 1024)),
        DEVICE_BYTES,
    );
    assert!(
        regions_that_just_touch.result.is_ok(),
        "S = 8、槽距 384 KiB：区域 [1, 4)、[4, 7)、[7, 10) MiB 首尾相接：{:?}",
        regions_that_just_touch.result.as_ref().err()
    );
    let regions_overlapping = make_filesystem_on_patterned_devices(
        &parameters_with(geometry_with_slot_spacing(384 * 1024 + 512)),
        DEVICE_BYTES,
    );
    assert_overlap_refused(
        &regions_overlapping,
        FixedStructure::RootRingRegion { region: 0 },
        FixedStructure::RootRingRegion { region: 1 },
        "S = 8、槽距 384 KiB + 512",
    );
}

/// physical_block_size 256：根槽装不下 457 字节的根记录。在 mkfs 开头拒，不走到 `RootRecord::to_slot` 的断言
/// （那时根环、journal 环已经清过、两个单元已经写了）。
#[test]
fn a_root_slot_narrower_than_the_root_record_is_refused_before_any_write() {
    let observation = make_filesystem_on_patterned_devices(
        &parameters_with(SystemImmutableSizes {
            physical_block_size: 256,
            ..default_geometry()
        }),
        DEVICE_BYTES,
    );
    assert!(
        matches!(
            observation.result,
            Err(MakeFilesystemError::RootSlotNarrowerThanTheRootRecord {
                physical_block_size: 256,
                root_record_bytes,
            }) if root_record_bytes == ROOT_RECORD_BYTES
        ),
        "根槽 256 字节：{:?}",
        observation.result.as_ref().err()
    );
    assert_refused_before_any_write(&observation, "根槽 256 字节");
}

/// physical_block_size 8192、槽距 4096：每条根写 8 KiB，盖到同一区域的下一个槽。根槽与槽距一样宽时放行。
#[test]
fn a_root_slot_wider_than_the_slot_spacing_is_refused_and_one_as_wide_is_not() {
    let wider = make_filesystem_on_patterned_devices(
        &parameters_with(SystemImmutableSizes {
            physical_block_size: 8192,
            ..default_geometry()
        }),
        DEVICE_BYTES,
    );
    assert!(
        matches!(
            wider.result,
            Err(
                MakeFilesystemError::RootSlotWiderThanTheFixedStructureSlotSpacing {
                    physical_block_size: 8192,
                    fixed_structure_slot_spacing: 4096,
                }
            )
        ),
        "根槽 8 KiB、槽距 4 KiB：{:?}",
        wider.result.as_ref().err()
    );
    assert_refused_before_any_write(&wider, "根槽 8 KiB、槽距 4 KiB");
    let as_wide = make_filesystem_on_patterned_devices(
        &parameters_with(SystemImmutableSizes {
            physical_block_size: 4096,
            ..default_geometry()
        }),
        DEVICE_BYTES,
    );
    assert!(
        as_wide.result.is_ok(),
        "根槽与槽距都是 4 KiB：{:?}",
        as_wide.result.as_ref().err()
    );
}

// ── 第 21、25、37 条要的池 ──

/// 两块内存盘上 mkfs → 取号 → 暖机 → 第一个文件（txg 3），分配器是 mkfs 同一个进程那一条。
struct PoolAfterTheFirstFile {
    parameters: MakeFilesystemParameters,
    devices: Vec<(DeviceIdentity, SparseBlockDevice)>,
    allocator: PoolAllocator,
    instance: InstanceGeneration,
    first_content: Vec<u8>,
    first: TransactionOutput,
}

fn pool_after_the_first_file() -> PoolAfterTheFirstFile {
    let parameters = parameters_with(default_geometry());
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> =
        [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| {
                (
                    identity,
                    SparseBlockDevice::new(DEVICE_BYTES, SECTOR_PHYSICAL_BLOCK_SIZE),
                )
            })
            .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("默认几何上 mkfs");
    let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
    let first_content = content_of(3000, 0);
    let (instance, first) = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let instance = acquire_instance(&mut writer).expect("取号 1");
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        let first = publish_first_file(
            &mut writer,
            &mut allocator,
            warmed.roots.last().expect("暖机写了两条根"),
            FirstFile {
                content: &first_content,
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
        devices,
        allocator,
        instance,
        first_content,
        first,
    }
}

fn image_of(devices: &[(DeviceIdentity, SparseBlockDevice)]) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: DEVICE_BYTES,
    }
}

// ── 第 21 条：只读挂载打得开新建的空 inode ──

/// 新建的 inode 长度 0、extent 树里没有它的条目（`transaction::publish_new_inodes`）：只读挂载打开它得到 0 个数据单元，
/// 不报「记录条数与文件大小对不上」。第一个文件照旧一个数据单元。
#[test]
fn a_new_empty_inode_opens_under_the_read_only_mount_with_no_data_unit() {
    let mut pool = pool_after_the_first_file();
    let new_inode_number = pool.first.inode_number_watermark();
    let with_the_new_inode = {
        let mut writer = PoolWriter::new(&pool.parameters, pool.devices.as_mut_slice());
        publish_new_inodes(
            &mut writer,
            &mut pool.allocator,
            &pool.first,
            1,
            WRITE_TIME_SECONDS + 1,
            pool.instance,
        )
        .expect("建一个空 inode")
    };
    let image = image_of(&pool.devices);
    let mounted = mount_read_only(&image).expect("只读挂载");
    assert_eq!(
        mounted.effective_root.checkpoint_txg, with_the_new_inode.root.checkpoint_txg,
        "挂到建 inode 的那一版"
    );
    let new_inode = mounted
        .mounted
        .open_file(&image, InodeNumber(new_inode_number))
        .expect("新建的空 inode 打得开");
    assert_eq!(new_inode.inode_record().size, 0, "新建的 inode 长度 0");
    assert_eq!(new_inode.data_unit_count(), 0, "新建的 inode 没有数据单元");
    let first_file = mounted
        .mounted
        .open_file(&image, InodeNumber(FIRST_INODE_NUMBER))
        .expect("第一个文件照旧打得开");
    assert_eq!(
        first_file.data_unit_count(),
        1,
        "3000 字节的文件一个数据单元"
    );
}

/// 写成空内容的文件长度也是 0，但它有一个声明长度 0 的数据单元（写侧切分的下界）：只读挂载照旧按一个单元打开它。
#[test]
fn a_file_overwritten_with_empty_content_still_opens_with_its_one_data_unit() {
    let mut pool = pool_after_the_first_file();
    let emptied = {
        let mut writer = PoolWriter::new(&pool.parameters, pool.devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut pool.allocator,
            &pool.first,
            FirstFile {
                content: &[],
                write_time_seconds: WRITE_TIME_SECONDS + 1,
            },
            pool.instance,
        )
        .expect("覆盖写成空内容")
    };
    assert_eq!(emptied.data_pointers.len(), 1, "空内容也写一个数据单元");
    let image = image_of(&pool.devices);
    let mounted = mount_read_only(&image).expect("只读挂载");
    let emptied_file = mounted
        .mounted
        .open_file(&image, InodeNumber(FIRST_INODE_NUMBER))
        .expect("写成空内容的文件打得开");
    assert_eq!(emptied_file.inode_record().size, 0, "长度 0");
    assert_eq!(
        emptied_file.data_unit_count(),
        1,
        "一个声明长度 0 的数据单元"
    );
}

// ── 第 25 条：恢复判反向链 ──

/// 覆盖写一次（txg 4）之后，把 txg 4 那条根的根槽清零：择根落回 txg 3，txg 4 那条记录在水位之上、由记录施加。
struct PoolWhoseNewestVersionOnlyTheJournalCarries {
    image: MemoryPool,
    instance: InstanceGeneration,
    first_content: Vec<u8>,
    first: TransactionOutput,
    second_content: Vec<u8>,
    second: TransactionOutput,
    journal_ring_bytes: u64,
}

fn pool_whose_newest_version_only_the_journal_carries(
) -> PoolWhoseNewestVersionOnlyTheJournalCarries {
    let mut pool = pool_after_the_first_file();
    let second_content = content_of(2999, 1);
    let second = {
        let mut writer = PoolWriter::new(&pool.parameters, pool.devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut pool.allocator,
            &pool.first,
            FirstFile {
                content: &second_content,
                write_time_seconds: WRITE_TIME_SECONDS + 1,
            },
            pool.instance,
        )
        .expect("覆盖写 txg 4")
    };
    assert!(
        second.earlier_records_of_this_publish.is_empty(),
        "一个数据单元的覆盖写只有一条记录"
    );
    let mut image = image_of(&pool.devices);
    let geometry = pool.parameters.geometry;
    let target = target_for_publish(
        second.root.checkpoint_txg,
        geometry.root_ring_slots_per_region,
    );
    let root_slot_device =
        pool.parameters.region_devices[usize::try_from(target.region).expect("区域号")];
    image
        .devices
        .get_mut(&root_slot_device)
        .expect("池里有这块盘")
        .write(
            slot_offset(target, geometry.fixed_structure_slot_spacing),
            &vec![0u8; usize::try_from(geometry.physical_block_size).expect("根槽宽")],
        );
    PoolWhoseNewestVersionOnlyTheJournalCarries {
        image,
        instance: pool.instance,
        first_content: pool.first_content,
        first: pool.first,
        second_content,
        second,
        journal_ring_bytes: geometry.journal_ring_bytes,
    }
}

/// 把 `record` 原样的字节换成 `replacement` 的，两块盘上各写一份（记录两份镜像）。
fn write_the_record_on_every_device(
    image: &mut MemoryPool,
    counter: u64,
    journal_ring_bytes: u64,
    replacement: &JournalRecord,
) {
    let bytes = replacement.to_bytes();
    for device in image.devices.values_mut() {
        device.write(record_offset(counter, journal_ring_bytes), &bytes);
    }
}

/// 正对照：链没动，txg 4 那一版由它的记录施加回来（施加路径真的走到了）；把它的反向链改掉一位、整条校验和重算
/// （校验和自洽、链不等），它不进重放前缀：恢复停在 txg 3，读回第一个文件。
#[test]
fn a_journal_record_whose_back_chain_disagrees_with_the_previous_record_is_left_out_of_the_replayed_prefix(
) {
    let mut pool = pool_whose_newest_version_only_the_journal_carries();
    let chosen = (pool.instance, pool.first.root.checkpoint_txg);
    assert_eq!(
        pool.second.record.to_bytes(),
        pool.second.record_bytes,
        "按解出来的字段重写的字节就是写者写下的那一份：改掉的只有反向链"
    );
    let untouched = recover(&pool.image, JournalPolicy::Consult);
    assert_eq!(
        untouched.effective_root,
        Some((pool.instance, pool.second.root.checkpoint_txg)),
        "链没动：txg 4 由记录施加"
    );
    assert_eq!(
        untouched.outcome,
        RecoveryOutcome::FileRead {
            root: chosen,
            content: pool.second_content.clone(),
        }
    );

    let mut broken_chain = pool.second.record.clone();
    broken_chain.back_chain ^= 1;
    write_the_record_on_every_device(
        &mut pool.image,
        pool.second.record.counter,
        pool.journal_ring_bytes,
        &broken_chain,
    );
    let refused = recover(&pool.image, JournalPolicy::Consult);
    assert_eq!(
        refused.effective_root,
        Some(chosen),
        "链值不等的记录不进重放前缀（I-8.6）：恢复停在所选根"
    );
    assert_eq!(refused.journal.prefix_applied, 0, "一条都没施加");
    assert_eq!(
        refused.outcome,
        RecoveryOutcome::FileRead {
            root: chosen,
            content: pool.first_content.clone(),
        },
        "读回的是所选根那一版的文件"
    );
}

/// 本实例内逻辑前一条（txg 3 那条记录）两份都自证不过、不在盘上：这一条的链判不了，不拿它断前缀
/// （与池级 checker 的 I-8.6 同一口径）——链改过的 txg 4 那条照样按「锚点读不出」那一支施加。
#[test]
fn a_journal_record_whose_previous_record_is_not_on_disk_is_replayed_without_judging_its_back_chain(
) {
    let mut pool = pool_whose_newest_version_only_the_journal_carries();
    let mut broken_chain = pool.second.record.clone();
    broken_chain.back_chain ^= 1;
    write_the_record_on_every_device(
        &mut pool.image,
        pool.second.record.counter,
        pool.journal_ring_bytes,
        &broken_chain,
    );
    let previous_record_offset = record_offset(pool.first.record.counter, pool.journal_ring_bytes);
    let identities: Vec<DeviceIdentity> = pool.image.devices.keys().copied().collect();
    for identity in identities {
        pool.image.flip_byte(
            identity,
            previous_record_offset,
            BYTE_IN_THE_HEADER_OF_A_JOURNAL_RECORD,
        );
    }
    let replayed = recover(&pool.image, JournalPolicy::Consult);
    assert_eq!(
        replayed.effective_root,
        Some((pool.instance, pool.second.root.checkpoint_txg)),
        "前一条不在盘上：链判不了，txg 4 照样施加"
    );
    assert_eq!(
        replayed.outcome,
        RecoveryOutcome::FileRead {
            root: (pool.instance, pool.first.root.checkpoint_txg),
            content: pool.second_content.clone(),
        }
    );
}

// ── 第 37 条：槽 0 自证不过时按池里记的槽距找槽 1 ──

/// 槽距 8192（io_min 8192）的池，盘 0 的槽 0 自证不过：盘 1 的槽 0 自证过、记着槽距 8192，盘 0 的槽 1 就在 8192。
/// 按它找到盘 0 的槽 1，择到的是盘 0 那一份（池里第一块交得出系统配置的盘）；按最小槽距 4096 找，盘 0 整块被跳过、择到盘 1 那一份。
#[test]
fn slot_one_is_looked_for_at_the_slot_spacing_another_devices_slot_zero_records() {
    let spacing = 8192;
    let parameters = parameters_with(geometry_with_slot_spacing(spacing));
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> =
        [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| {
                (
                    identity,
                    SparseBlockDevice::new(DEVICE_BYTES, SECTOR_PHYSICAL_BLOCK_SIZE),
                )
            })
            .collect();
    make_filesystem(&parameters, &mut devices).expect("槽距 8192 的池");
    let mut image = image_of(&devices);
    image.flip_byte(
        DeviceIdentity(0),
        DeviceOffsetInBytes(0),
        BYTE_IN_THE_RESERVED_AREA_OF_A_SYSTEM_CONFIGURATION_SLOT,
    );
    let chosen = choose_system_configuration(&image).expect("盘 0 的槽 1 与盘 1 两槽都自证得过");
    assert_eq!(
        chosen.immutable.this_device,
        DeviceIdentity(0),
        "盘 0 的槽 1 在槽距 8192 处找到"
    );
    assert_eq!(
        chosen.immutable.sizes.fixed_structure_slot_spacing, spacing,
        "择到的那一份记着槽距 8192"
    );
    assert_eq!(
        chosen.quantities.slot_generation, 1,
        "mkfs 两槽都种世代号 1"
    );
}
