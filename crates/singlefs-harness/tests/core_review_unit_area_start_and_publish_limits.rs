//! 代码审阅第 15 条后一半、第 34 条（实审 A2c）的会红用例：
//! 分配器的单元区起点随 journal 环长走（D23（journal 的角色与格式） 已定项 19 ③「单元区起始槽号随环长走，默认环下是 784 MiB（槽 50176）」、
//! D3（空间分配） 已定项 10 ④「第一版 = journal 环末尾的下一个槽」），默认环下与编译期常量逐槽相同；
//! 第一个文件版本释放 mkfs 那片树表与树表 0 条那一版的分配记录树节点时，照映射那一路逐盘核（不只看第一块盘）；
//! 一次发布切出来的 journal 记录多于在飞上限在任何写之前拒（第 26 条）；txg、实例代号加一越过顶在任何写之前报错、不 panic（第 36 条）；
//! 可写挂载写行与暖机之后推抬 F、抬 F 报错时交回已落盘那几次的写账（实审 A1b Q5）；管理员回退照可写挂载同一套核调用方的参数与盘表。
//! 盘都是内存稀疏盘（`singlefs_harness::crash::SparseBlockDevice`）。

mod common_admission;

use std::collections::BTreeSet;

use common_admission::PoolUnderTest;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::admission::INSTANCE_ROWS_PER_INSTANCE_TABLE_PAGE;
use singlefs_core::allocator::{
    DeviceFreeMap, Placement, PoolAllocator, UnitAreaStart,
    UnitAreaStartOffTheClusterSegmentBoundaryUnsupported, UnitFootprint,
};
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes};
use singlefs_core::instance_table::InstanceRow;
use singlefs_core::journal::{record_offset, slot_after_the_journal_ring};
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, root_ring_occupancy_after_make_filesystem,
    MakeFilesystemParameters, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{
    mount_writable, push_one_floor_raise_within_the_admission_budget, raise_rollback_floor,
    roll_back_by_a_forward_publish, CallerInputsDisagreeingWithTheDisk,
    MakeFilesystemParameterField, MountError, MountSpaceAdmission, RepeatedDeviceIdentity,
    RollbackError, RollbackTarget, SequenceNumberAtTheTopOfItsRange, ShadowLedger,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::{SystemConfiguration, SystemImmutableSizes};
use singlefs_core::transaction::{
    acquire_instance, journal_record_limit_of_one_publish, publish_first_file,
    publish_instance_table_on_version_without_file, publish_new_inodes, publish_overwrite,
    publish_sequential_write, role_of_allocation_record_tree_node, warm_up, FirstFile,
    InstanceTableOnlyPublishPlan, InstanceTableRewrite, JournalRecordNamedEntryCapacity,
    PoolVersion, PoolWriter, PublishError, TransactionOutput, TransactionUnit,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::{
    CLUSTER_SEGMENT_SLOTS, JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES, JOURNAL_SAFETY_FACTOR,
    SLOT_BYTES, SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, UNIT_AREA_START_SLOT,
};
use singlefs_harness::crash::{SparseBlockDevice, SparseDevice};
use singlefs_harness::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
    FaultSchedule, InjectedFault, SharedFaultPlan,
};
use singlefs_harness::history::HistoryDeviceWidth;

const DEVICE_BYTES: u64 = 4 << 30;
const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"singlefs-rev-a2c";
const WRITE_TIME_SECONDS: u64 = 1_788_000_000;
/// 内存稀疏盘按 512 字节扇区存：物理块宽 512。
const SECTOR_PHYSICAL_BLOCK_SIZE: PhysicalBlockSizeInBytes = PhysicalBlockSizeInBytes(512);
const FIRST_FILE_BYTES: usize = 3000;

fn geometry_with_journal_ring_bytes(journal_ring_bytes: u64) -> SystemImmutableSizes {
    SystemImmutableSizes {
        physical_block_size: 512,
        minimum_input_output_bytes: 512,
        fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(512),
        journal_ring_bytes,
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

fn sparse_devices(device_bytes: u64) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
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

// ── 第 34 条：第一个文件版本释放 mkfs 那片树表与树表 0 条那一版的分配记录树节点，逐盘核 ──

/// 两块内存盘上 mkfs → 取号 → 暖机，分配器是 mkfs 同一个进程那一条：第一个文件版本就接在暖机的末条根后面。
struct PoolAfterTheWarmUp {
    parameters: MakeFilesystemParameters,
    devices: Vec<(DeviceIdentity, SparseBlockDevice)>,
    allocator: PoolAllocator,
    instance: InstanceGeneration,
    version_to_build_on: RootRecord,
    previous_record_bytes: Vec<u8>,
}

fn pool_after_the_warm_up() -> PoolAfterTheWarmUp {
    pool_after_the_warm_up_on_a_journal_ring_of(JOURNAL_RING_DEFAULT_BYTES)
}

fn pool_after_the_warm_up_on_a_journal_ring_of(journal_ring_bytes: u64) -> PoolAfterTheWarmUp {
    let parameters = parameters_with(geometry_with_journal_ring_bytes(journal_ring_bytes));
    let mut devices = sparse_devices(DEVICE_BYTES);
    let genesis = make_filesystem(&parameters, &mut devices).expect("默认几何上 mkfs");
    let allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    let instance = acquire_instance(&mut writer).expect("取号 1");
    let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
    let version_to_build_on = *warmed.roots.last().expect("暖机写了两条根");
    PoolAfterTheWarmUp {
        parameters,
        devices,
        allocator,
        instance,
        version_to_build_on,
        previous_record_bytes: warmed.last_record_bytes,
    }
}

/// mkfs 那片树表单元的落点（第 0 版，一槽）。
const GENESIS_TREE_TABLE: Placement = Placement {
    slot: TREE_TABLE_GENESIS_SLOT,
    span: 1,
};

/// 在那一版上发第一个文件版本。
fn publish_the_first_file_on(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, SparseBlockDevice)],
    allocator: &mut PoolAllocator,
    version_to_build_on: &RootRecord,
    instance: InstanceGeneration,
    previous_record_bytes: &[u8],
) -> Result<TransactionOutput, PublishError> {
    let content = content_of(FIRST_FILE_BYTES, 0);
    let mut writer = PoolWriter::new(parameters, devices);
    publish_first_file(
        &mut writer,
        allocator,
        version_to_build_on,
        FirstFile {
            content: &content,
            write_time_seconds: WRITE_TIME_SECONDS,
        },
        instance,
        previous_record_bytes,
    )
}

/// 断言第一个文件版本在任何写之前被拒成「这块盘上那一片已释放」，两块盘逐字节不变。
fn assert_the_first_file_is_refused_before_any_write_for_a_copy_already_released_on(
    pool: &mut PoolAfterTheWarmUp,
    unit: TransactionUnit,
    slot: SlotNumber,
    device_whose_record_is_released: DeviceIdentity,
) {
    let images_before = images_of(&pool.devices);
    let refused = publish_the_first_file_on(
        &pool.parameters,
        pool.devices.as_mut_slice(),
        &mut pool.allocator,
        &pool.version_to_build_on,
        pool.instance,
        &pool.previous_record_bytes,
    )
    .expect_err("两块盘的账对不上：第一个文件版本在任何写之前拒");
    assert!(
        matches!(
            refused,
            PublishError::ReleaseTargetAlreadyReleased { unit: refused_unit, device, slot: refused_slot }
                if refused_unit == unit && device == device_whose_record_is_released && refused_slot == slot
        ),
        "报的是 {unit:?} 在盘 {device_whose_record_is_released:?} 上那一份已释放（槽 {slot:?}），实际 {refused:?}"
    );
    assert_eq!(
        images_of(&pool.devices),
        images_before,
        "在任何写之前拒：两块盘逐字节不变"
    );
}

/// mkfs 那片树表在盘 1 的账里已释放、盘 0 的账里还在册（两块盘的分配记录树对不对称是盘上读来的）：
/// 第一个文件版本换下它时逐盘核，在任何写之前报「盘 1 上那一份已释放」。改之前只看第一块盘（盘 0 在册）就交给释放，
/// 在 `PoolAllocator::release` 的断言上 panic。
#[test]
fn the_first_file_version_refuses_a_genesis_tree_table_that_device_one_already_released_before_any_write(
) {
    let mut pool = pool_after_the_warm_up();
    pool.allocator.release_leaving_the_record_allocated_on(
        GENESIS_TREE_TABLE,
        CheckpointTxg(2),
        &[DeviceIdentity(0)],
    );
    assert_the_first_file_is_refused_before_any_write_for_a_copy_already_released_on(
        &mut pool,
        TransactionUnit::TreeTable,
        TREE_TABLE_GENESIS_SLOT,
        DeviceIdentity(1),
    );
}

/// 反过来：盘 0 的账里已释放、盘 1 的账里还在册。改之前只看第一块盘（盘 0 已释放）就不释放它，第一个文件版本照发，
/// 盘 1 上那条记录从此占着；改后同样在任何写之前报「盘 0 上那一份已释放」。
#[test]
fn the_first_file_version_refuses_a_genesis_tree_table_that_device_zero_already_released_before_any_write(
) {
    let mut pool = pool_after_the_warm_up();
    pool.allocator.release_leaving_the_record_allocated_on(
        GENESIS_TREE_TABLE,
        CheckpointTxg(2),
        &[DeviceIdentity(1)],
    );
    assert_the_first_file_is_refused_before_any_write_for_a_copy_already_released_on(
        &mut pool,
        TransactionUnit::TreeTable,
        TREE_TABLE_GENESIS_SLOT,
        DeviceIdentity(0),
    );
}

/// 对照：两块盘的账里 mkfs 那片树表都已释放（两块盘一致）——哪块盘上都不在册、未释放，照改之前的样子不释放它，第一个文件版本照发。
#[test]
fn the_first_file_version_publishes_without_releasing_a_genesis_tree_table_released_on_every_device(
) {
    let mut pool = pool_after_the_warm_up();
    pool.allocator.release(GENESIS_TREE_TABLE, CheckpointTxg(2));
    let published = publish_the_first_file_on(
        &pool.parameters,
        pool.devices.as_mut_slice(),
        &mut pool.allocator,
        &pool.version_to_build_on,
        pool.instance,
        &pool.previous_record_bytes,
    )
    .expect("两块盘上都已释放：不再释放它，第一个文件版本照发");
    assert_eq!(
        published.root.checkpoint_txg,
        CheckpointTxg(3),
        "第一个文件版本接在暖机那两次后面"
    );
}

/// mkfs → 可写挂载（实例 1，零单元发布）→ 可写挂载（实例 2，写行：树表 0 条那一版写下自己的分配记录树）：第一个文件版本要换下
/// 那棵树的每个节点。把它的根节点在盘 1 的账里改成已释放：第一个文件版本逐盘核，在任何写之前报「盘 1 上那一份已释放」。
/// 改之前只看第一块盘，在 `PoolAllocator::release` 的断言上 panic。
#[test]
fn the_first_file_version_refuses_an_allocation_record_tree_node_of_the_version_without_file_that_device_one_already_released(
) {
    let parameters = parameters_with(geometry_with_journal_ring_bytes(JOURNAL_RING_DEFAULT_BYTES));
    let mut devices = sparse_devices(DEVICE_BYTES);
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    mount_writable(&parameters, &mut devices).expect("mkfs 之后第一次可写挂载：零单元发布");
    let mounted = mount_writable(&parameters, &mut devices).expect("第二次可写挂载：写行");
    let PoolVersion::WithoutFile(version_without_file) = &mounted.current else {
        panic!("还没写过文件：现行那一版树表 0 条");
    };
    let mut allocator = mounted.allocator;
    let (node, pointer) = allocator
        .allocation_record_tree_of_the_version_without_file()
        .expect("写行那次发布写下了这一版的分配记录树")
        .version
        .nodes
        .first()
        .map(|(node, pointer)| (*node, *pointer))
        .expect("那棵树至少一个节点");
    let unit = role_of_allocation_record_tree_node(node);
    let slot = pointer.locations[0].slot;
    allocator.release_leaving_the_record_allocated_on(
        Placement {
            slot,
            span: unit.span_slots(),
        },
        version_without_file.root.checkpoint_txg,
        &[DeviceIdentity(0)],
    );
    let mut pool = PoolAfterTheWarmUp {
        parameters,
        devices,
        allocator,
        instance: mounted.output.instance,
        version_to_build_on: version_without_file.root,
        previous_record_bytes: version_without_file.record_bytes.clone(),
    };
    assert_the_first_file_is_refused_before_any_write_for_a_copy_already_released_on(
        &mut pool,
        unit,
        slot,
        DeviceIdentity(1),
    );
}

// ── 第 15 条后一半：分配器的单元区起点随 journal 环长走 ──

const MEBIBYTE: u64 = 1 << 20;
/// `history.rs` 小盘那一档的环长：末尾的下一个槽是 1024 + 8192 = 9216 = 144 × 64，落在聚簇段边界上。
const JOURNAL_RING_OF_128_MEBIBYTES: u64 = 128 * MEBIBYTE;
const SLOT_AFTER_THE_128_MEBIBYTE_RING: SlotNumber = SlotNumber(9216);

/// 默认环（768 MiB）下单元区起点就是格式常量那一槽（50176 = 784 MiB，D23（journal 的角色与格式） 已定项 19 ③ 的逐字值），
/// `DeviceFreeMap::new` 建的空闲图从它起；mkfs 同一个进程里按「环长算出的起点」建的分配器写出的第一个文件，与按 `DeviceFreeMap::new`
/// 建的逐字节相同（两块盘整盘比）。
#[test]
fn the_default_journal_ring_puts_the_unit_area_at_the_compiled_slot_and_the_first_file_lands_byte_for_byte_as_before(
) {
    assert_eq!(
        slot_after_the_journal_ring(JOURNAL_RING_DEFAULT_BYTES),
        SlotNumber(UNIT_AREA_START_SLOT),
        "默认环 768 MiB 末尾的下一个槽是 784 MiB"
    );
    let default_start = UnitAreaStart::following_the_journal_ring(JOURNAL_RING_DEFAULT_BYTES)
        .expect("默认环的起点落在聚簇段边界上");
    assert_eq!(default_start, UnitAreaStart::of_the_default_journal_ring());
    assert_eq!(default_start.slot(), SlotNumber(UNIT_AREA_START_SLOT));
    let map = DeviceFreeMap::new(DeviceIdentity(0), DEVICE_BYTES);
    assert_eq!(map.unit_area_start(), default_start);
    assert_eq!(
        map.unit_area_slots(),
        DEVICE_BYTES / SLOT_BYTES - UNIT_AREA_START_SLOT,
        "4 GiB 的盘：单元区 211968 槽"
    );

    let with_the_compiled_start =
        first_file_images_with(UnitAreaStartOfTheAllocator::AsMakeFilesystemBuildsIt);
    let with_the_start_following_the_ring =
        first_file_images_with(UnitAreaStartOfTheAllocator::FollowingTheJournalRing);
    assert!(
        with_the_start_following_the_ring == with_the_compiled_start,
        "默认环下按环长算起点的分配器写出的第一个文件与今天逐字节相同"
    );
}

/// mkfs 同一个进程那条会话的分配器从哪个起点建空闲图。
#[derive(Clone, Copy)]
enum UnitAreaStartOfTheAllocator {
    /// `allocator_after_make_filesystem` 那样建（`DeviceFreeMap::new`）。
    AsMakeFilesystemBuildsIt,
    /// 按环长现算的起点建（`UnitAreaStart::following_the_journal_ring` + `DeviceFreeMap::with_unit_area_start`），
    /// mkfs 占的两个单元与根环表照 `allocator_after_make_filesystem` 标上。
    FollowingTheJournalRing,
}

/// 默认几何上 mkfs → 取号 → 暖机 → 第一个文件，分配器按 `start_of_the_allocator` 建；交回两块盘的镜像。
fn first_file_images_with(
    start_of_the_allocator: UnitAreaStartOfTheAllocator,
) -> Vec<(DeviceIdentity, SparseDevice)> {
    let parameters = parameters_with(geometry_with_journal_ring_bytes(JOURNAL_RING_DEFAULT_BYTES));
    let mut devices = sparse_devices(DEVICE_BYTES);
    let genesis = make_filesystem(&parameters, &mut devices).expect("默认几何上 mkfs");
    let mut allocator = match start_of_the_allocator {
        UnitAreaStartOfTheAllocator::AsMakeFilesystemBuildsIt => {
            allocator_after_make_filesystem(&parameters, &devices, &genesis)
        }
        UnitAreaStartOfTheAllocator::FollowingTheJournalRing => {
            let start =
                UnitAreaStart::following_the_journal_ring(parameters.geometry.journal_ring_bytes)
                    .expect("默认环的起点落在聚簇段边界上");
            let mut allocator = PoolAllocator::new(
                devices
                    .iter()
                    .map(|(identity, device)| {
                        DeviceFreeMap::with_unit_area_start(
                            *identity,
                            device.size_in_bytes(),
                            start,
                        )
                    })
                    .collect(),
            );
            allocator.mark_format_time_units(
                Placement {
                    slot: INSTANCE_TABLE_SLOT,
                    span: 2,
                },
                GENESIS_TREE_TABLE,
            );
            allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(
                &parameters,
                &genesis,
            ));
            allocator
        }
    };
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    let instance = acquire_instance(&mut writer).expect("取号 1");
    let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
    let content = content_of(FIRST_FILE_BYTES, 0);
    publish_first_file(
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
    images_of(&devices)
}

/// 环长 128 MiB 时单元区从环末尾的下一个槽（9216）起：用户数据、开段、提交内生块的回落三种落点都从那里起扫，
/// 起点之前一槽不算单元区，末尾照盘的字节数算；标占两槽之后用户数据落到下一对、开段落到下一段。
#[test]
fn a_free_map_whose_unit_area_follows_a_128_mebibyte_journal_ring_hands_out_slots_right_after_the_ring(
) {
    let start = UnitAreaStart::following_the_journal_ring(JOURNAL_RING_OF_128_MEBIBYTES)
        .expect("环长是 1 MiB 的整数倍：起点落在聚簇段边界上");
    assert_eq!(start.slot(), SLOT_AFTER_THE_128_MEBIBYTE_RING);
    let mut map = DeviceFreeMap::with_unit_area_start(DeviceIdentity(0), DEVICE_BYTES, start);
    let unit_area_end = DEVICE_BYTES / SLOT_BYTES;
    assert_eq!(map.unit_area_start(), start);
    assert_eq!(
        map.unit_area_slots(),
        unit_area_end - SLOT_AFTER_THE_128_MEBIBYTE_RING.0
    );
    assert_eq!(
        map.empty_segments(),
        (unit_area_end - SLOT_AFTER_THE_128_MEBIBYTE_RING.0) / CLUSTER_SEGMENT_SLOTS
    );
    assert!(!map.is_free(SlotNumber(SLOT_AFTER_THE_128_MEBIBYTE_RING.0 - 1)));
    assert!(map.is_free(SLOT_AFTER_THE_128_MEBIBYTE_RING));
    assert!(map.is_free(SlotNumber(unit_area_end - 1)));
    assert!(!map.is_free(SlotNumber(unit_area_end)));
    let no_cluster_segment = BTreeSet::new();
    assert_eq!(
        map.lowest_user_data_slot(&no_cluster_segment),
        Some(SLOT_AFTER_THE_128_MEBIBYTE_RING),
        "用户数据从环末尾的下一个槽起"
    );
    assert_eq!(
        map.lowest_empty_segment(),
        Some(SLOT_AFTER_THE_128_MEBIBYTE_RING),
        "开段从环末尾的下一个槽起"
    );
    assert_eq!(
        map.lowest_commit_generated_fallback_slot(UnitFootprint::OneSlot),
        Some(SLOT_AFTER_THE_128_MEBIBYTE_RING),
        "回落从环末尾的下一个槽起"
    );
    map.mark_allocated(SLOT_AFTER_THE_128_MEBIBYTE_RING, 2);
    assert_eq!(
        map.lowest_user_data_slot(&no_cluster_segment),
        Some(SlotNumber(SLOT_AFTER_THE_128_MEBIBYTE_RING.0 + 2))
    );
    assert_eq!(
        map.lowest_empty_segment(),
        Some(SlotNumber(
            SLOT_AFTER_THE_128_MEBIBYTE_RING.0 + CLUSTER_SEGMENT_SLOTS
        ))
    );
    assert_eq!(
        map.lowest_commit_generated_fallback_slot(UnitFootprint::TwoSlotsAligned),
        Some(SlotNumber(SLOT_AFTER_THE_128_MEBIBYTE_RING.0 + 2))
    );
}

/// 环末尾不落在槽边界上时取下一个整槽（128 MiB − 4 KiB 的环：末尾在槽 9215 当中，下一个整槽是 9216）；
/// 取出来的槽不落在聚簇段边界上时（128 MiB + 4 KiB 的环：9217）按第一版不支持拒，带着环长与那个槽。
#[test]
fn a_journal_ring_ending_inside_a_slot_rounds_up_and_a_start_off_the_cluster_segment_boundary_is_refused(
) {
    let ring_ending_inside_the_last_slot = JOURNAL_RING_OF_128_MEBIBYTES - 4096;
    assert_eq!(
        slot_after_the_journal_ring(ring_ending_inside_the_last_slot),
        SLOT_AFTER_THE_128_MEBIBYTE_RING
    );
    assert_eq!(
        UnitAreaStart::following_the_journal_ring(ring_ending_inside_the_last_slot)
            .map(UnitAreaStart::slot),
        Ok(SLOT_AFTER_THE_128_MEBIBYTE_RING),
        "环末尾在一个槽当中：下一个整槽"
    );
    let ring_one_record_past_the_boundary = JOURNAL_RING_OF_128_MEBIBYTES + 4096;
    assert_eq!(
        UnitAreaStart::following_the_journal_ring(ring_one_record_past_the_boundary),
        Err(UnitAreaStartOffTheClusterSegmentBoundaryUnsupported {
            journal_ring_bytes: ring_one_record_past_the_boundary,
            slot_after_the_journal_ring: SlotNumber(SLOT_AFTER_THE_128_MEBIBYTE_RING.0 + 1),
        }),
        "9217 不是 64 的整数倍：聚簇段怎么对齐条款没写，第一版不支持"
    );
}

// ── 第 26、36 条要的池 ──

/// 两块内存盘上 mkfs → 取号 → 暖机 → 第一个文件（txg 3），分配器是 mkfs 同一个进程那一条。
struct PoolAfterTheFirstFile {
    parameters: MakeFilesystemParameters,
    devices: Vec<(DeviceIdentity, SparseBlockDevice)>,
    allocator: PoolAllocator,
    instance: InstanceGeneration,
    first: TransactionOutput,
}

fn pool_after_the_first_file_on_a_journal_ring_of(
    journal_ring_bytes: u64,
) -> PoolAfterTheFirstFile {
    let mut pool = pool_after_the_warm_up_on_a_journal_ring_of(journal_ring_bytes);
    let first = publish_the_first_file_on(
        &pool.parameters,
        pool.devices.as_mut_slice(),
        &mut pool.allocator,
        &pool.version_to_build_on,
        pool.instance,
        &pool.previous_record_bytes,
    )
    .expect("第一个文件");
    PoolAfterTheFirstFile {
        parameters: pool.parameters,
        devices: pool.devices,
        allocator: pool.allocator,
        instance: pool.instance,
        first,
    }
}

// ── 第 26 条：一次发布切出来的 journal 记录数设上限 ──

/// 1 MiB 的环：64 槽，环末尾的下一个槽 1088 = 17 × 64 落在聚簇段边界上（实审 A3b 之后 mkfs 只收这样的环，主 agent 定：不为测试开口子）。
/// 256 条记录，在飞上限 = 256 ÷ 3 = 85 条。环长是 1 MiB 整数倍的环里它的上限最小。
const JOURNAL_RING_OF_ONE_MEBIBYTE: u64 = 1 << 20;

/// [`JOURNAL_RING_OF_ONE_MEBIBYTE`] 上一次发布至多切成几条记录：256 ÷ 3 = 85（用例这一侧照格式常量算）。
const RECORD_LIMIT_ON_THE_ONE_MEBIBYTE_RING: u64 =
    JOURNAL_RING_OF_ONE_MEBIBYTE / JOURNAL_RECORD_BYTES / JOURNAL_SAFETY_FACTOR;

/// 从文件偏移 0 顺序写这么多字节切成 `data_units` 个一单元事务（`write_request_split`：每个数据单元装 32634 字节净荷）。
fn content_bytes_of_a_sequential_write_into(data_units: u64) -> usize {
    let payload_bytes =
        u64::try_from(data_unit_payload_capacity()).expect("一个数据单元的净荷装得进 u64");
    usize::try_from((data_units - 1) * payload_bytes + 1).expect("几兆字节装得进 usize")
}

/// 1 MiB 的环上（在飞上限 85），第一个文件之后顺序写 86 个数据单元：切成 86 条记录（前 85 条各点名自己那个数据单元、末条点名其余），
/// 多于上限，在任何写之前拒成 `JournalRecordsOfThePublishExceedTheLimit { 86, 85 }`，两块盘逐字节不变、分配器的记录不动。
/// 改之前照写：86 条记录落进环里，恢复施加前缀时最多取 85 条、那次发布永远施加不全。
/// 原先用六条记录长的环（在飞上限 2）测同一件事；A3b 之后那条环的起点落在槽 1026、不在段边界上，mkfs 拒它。
#[test]
fn a_sequential_write_cut_into_more_journal_records_than_the_in_flight_limit_is_refused_before_any_write(
) {
    let mut pool = pool_after_the_first_file_on_a_journal_ring_of(JOURNAL_RING_OF_ONE_MEBIBYTE);
    assert_eq!(
        journal_record_limit_of_one_publish(JOURNAL_RING_OF_ONE_MEBIBYTE),
        RECORD_LIMIT_ON_THE_ONE_MEBIBYTE_RING
    );
    assert_eq!(RECORD_LIMIT_ON_THE_ONE_MEBIBYTE_RING, 85, "256 ÷ 3");
    let images_before = images_of(&pool.devices);
    let records_before = pool.allocator.records().to_vec();
    let content = content_of(
        content_bytes_of_a_sequential_write_into(RECORD_LIMIT_ON_THE_ONE_MEBIBYTE_RING + 1),
        1,
    );
    let mut writer = PoolWriter::new(&pool.parameters, pool.devices.as_mut_slice());
    let refused = publish_sequential_write(
        &mut writer,
        &mut pool.allocator,
        &pool.first,
        FirstFile {
            content: &content,
            write_time_seconds: WRITE_TIME_SECONDS,
        },
        pool.instance,
    )
    .expect_err("86 条记录多于在飞上限 85 条");
    assert!(
        matches!(
            refused,
            PublishError::JournalRecordsOfThePublishExceedTheLimit {
                records_of_the_publish: 86,
                record_limit: 85,
            }
        ),
        "报的是记录条数超上限：{refused:?}"
    );
    assert_eq!(images_of(&pool.devices), images_before, "两块盘逐字节不变");
    assert_eq!(
        pool.allocator.records(),
        records_before.as_slice(),
        "分配器的记录不动"
    );
}

/// 对照：同一个环上顺序写 85 个数据单元切成 85 条记录，正好等于上限，照发（上限是「多于才拒」）。
#[test]
fn a_sequential_write_cut_into_as_many_journal_records_as_the_in_flight_limit_is_published() {
    let mut pool = pool_after_the_first_file_on_a_journal_ring_of(JOURNAL_RING_OF_ONE_MEBIBYTE);
    let content = content_of(
        content_bytes_of_a_sequential_write_into(RECORD_LIMIT_ON_THE_ONE_MEBIBYTE_RING),
        2,
    );
    let mut writer = PoolWriter::new(&pool.parameters, pool.devices.as_mut_slice());
    let published = publish_sequential_write(
        &mut writer,
        &mut pool.allocator,
        &pool.first,
        FirstFile {
            content: &content,
            write_time_seconds: WRITE_TIME_SECONDS,
        },
        pool.instance,
    )
    .expect("85 条记录不多于上限 85 条");
    assert_eq!(published.root.checkpoint_txg, CheckpointTxg(4));
    assert_eq!(
        u64::try_from(published.earlier_records_of_this_publish.len()).expect("条数") + 1,
        RECORD_LIMIT_ON_THE_ONE_MEBIBYTE_RING,
        "这次发布切成 85 条记录"
    );
}

/// 1 MiB 的环上（在飞上限 85 条），mkfs 同一个进程里取号之后直接在 mkfs 那一版上写行，一条记录只许点名一项（只供测试的开关）：
/// 行写满 85 片实例表（85 × 369 行），实例表 85 片加分配记录树的节点至少 86 项、切成至少 86 条记录，多于上限，
/// 在动分配器与任何写之前拒，两块盘逐字节不变。
/// 原先用三条记录长的环（在飞上限 1）、实例表一片加分配记录树的节点就超；A3b 之后那条环的起点落在槽 1025、不在段边界上，mkfs 拒它。
#[test]
fn a_row_publish_on_a_version_without_file_cut_into_more_journal_records_than_the_limit_is_refused_before_any_write(
) {
    let parameters = parameters_with(geometry_with_journal_ring_bytes(
        JOURNAL_RING_OF_ONE_MEBIBYTE,
    ));
    let mut devices = sparse_devices(DEVICE_BYTES);
    let genesis = make_filesystem(&parameters, &mut devices).expect("1 MiB 的环上 mkfs");
    let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
    let instance = acquire_instance(&mut PoolWriter::new(&parameters, devices.as_mut_slice()))
        .expect("取号 1");
    let images_before = images_of(&devices);
    let records_before = allocator.records().to_vec();
    let rows_filling_as_many_pages_as_the_limit = u32::try_from(
        RECORD_LIMIT_ON_THE_ONE_MEBIBYTE_RING * INSTANCE_ROWS_PER_INSTANCE_TABLE_PAGE,
    )
    .expect("85 × 369 行装得进实例代号的 32 位");
    let instance_table = InstanceTableRewrite {
        rows: (1..=rows_filling_as_many_pages_as_the_limit)
            .map(|row_number| InstanceRow {
                instance: InstanceGeneration(row_number),
                selected_root_txg: CheckpointTxg(0),
                applied_transaction_high_water: 0,
            })
            .collect(),
        replaced_chain: vec![genesis.root.instance_table],
    };
    assert_eq!(
        u64::try_from(instance_table.pages_after_this_publish()).expect("片数装得进 u64"),
        RECORD_LIMIT_ON_THE_ONE_MEBIBYTE_RING,
        "85 × 369 行正好写满 85 片"
    );
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    writer.set_journal_record_named_entry_capacity(
        JournalRecordNamedEntryCapacity::CappedForTests {
            named_entries_per_record: 1,
        },
    );
    let refused = publish_instance_table_on_version_without_file(
        &mut writer,
        &mut allocator,
        &genesis.root,
        InstanceTableOnlyPublishPlan {
            txg: CheckpointTxg(1),
            counter: 1,
            instance,
            back_chain: 0,
            rollback_floor: CheckpointTxg(0),
            instance_table: &instance_table,
            tree_identifier_watermark: genesis.root.tree_identifier_watermark,
        },
    )
    .expect_err("点名项至少 86 项、一条记录一项：至少 86 条，多于上限 85 条");
    assert!(
        matches!(
            refused,
            PublishError::JournalRecordsOfThePublishExceedTheLimit {
                records_of_the_publish,
                record_limit: 85,
            } if records_of_the_publish >= 86
        ),
        "报的是记录条数超上限：{refused:?}"
    );
    drop(writer);
    assert_eq!(images_of(&devices), images_before, "两块盘逐字节不变");
    assert_eq!(
        allocator.records(),
        records_before.as_slice(),
        "分配器的记录不动"
    );
}

// ── 第 36 条：txg、实例代号加一越过顶在任何写之前报错、不 panic ──

/// 现行那一版的 txg 改成 `u64::MAX` 的那一份（盘上读来的 8 字节，坏镜像才有；这里改的是内存里交给发布路径的那一份）。
fn at_the_top_txg(version: &TransactionOutput) -> TransactionOutput {
    let mut at_the_top = version.clone();
    at_the_top.root.checkpoint_txg = CheckpointTxg(u64::MAX);
    at_the_top
}

/// 覆盖写、顺序写、建 inode、第一个文件版本四条发布路径，接在 txg 已是 `u64::MAX` 的那一版后面：下一个 txg 装不下，
/// 各在任何写之前报 `NextCheckpointTxgPastTheTopOfItsRange`，两块盘逐字节不变。改之前加一溢出 panic。
#[test]
fn publishes_after_a_version_whose_txg_is_at_the_top_of_its_range_are_refused_before_any_write() {
    let mut pool = pool_after_the_first_file_on_a_journal_ring_of(JOURNAL_RING_DEFAULT_BYTES);
    let at_the_top = at_the_top_txg(&pool.first);
    let images_before = images_of(&pool.devices);
    let content = content_of(FIRST_FILE_BYTES, 3);
    let file = FirstFile {
        content: &content,
        write_time_seconds: WRITE_TIME_SECONDS,
    };
    let is_refused_at_the_top = |outcome: Result<TransactionOutput, PublishError>| {
        matches!(
            outcome,
            Err(PublishError::NextCheckpointTxgPastTheTopOfItsRange {
                version_to_build_on: CheckpointTxg(u64::MAX),
            })
        )
    };
    let mut writer = PoolWriter::new(&pool.parameters, pool.devices.as_mut_slice());
    assert!(
        is_refused_at_the_top(publish_overwrite(
            &mut writer,
            &mut pool.allocator,
            &at_the_top,
            file,
            pool.instance
        )),
        "覆盖写"
    );
    assert!(
        is_refused_at_the_top(publish_sequential_write(
            &mut writer,
            &mut pool.allocator,
            &at_the_top,
            file,
            pool.instance
        )),
        "顺序写"
    );
    assert!(
        is_refused_at_the_top(publish_new_inodes(
            &mut writer,
            &mut pool.allocator,
            &at_the_top,
            2,
            WRITE_TIME_SECONDS,
            pool.instance
        )),
        "建 inode"
    );
    assert!(
        is_refused_at_the_top(publish_first_file(
            &mut writer,
            &mut pool.allocator,
            &at_the_top.root,
            file,
            pool.instance,
            &at_the_top.record_bytes,
        )),
        "第一个文件版本"
    );
    drop(writer);
    assert_eq!(images_of(&pool.devices), images_before, "两块盘逐字节不变");
}

/// 在两块盘 journal 环里没用过的一槽写一条自证得过的记录：别的实例（7）、计数器 100、txg 取 `checkpoint_txg`
/// （盘上读来的 8 字节，坏镜像才有这么大的数）。恢复只施加所选根那个实例的记录，它不进前缀；可写挂载算新实例第一次发布的 txg
/// 取环里全部自证通过的记录的 txg 的最大值（D23（journal 的角色与格式） 已定项 14 第 3 条），会读到它。
fn plant_a_foreign_record_at_txg(pool: &mut PoolAfterTheFirstFile, checkpoint_txg: CheckpointTxg) {
    let mut planted = pool.first.record.clone();
    planted.instance = InstanceGeneration(7);
    planted.counter = 100;
    planted.checkpoint_txg = checkpoint_txg;
    planted.transaction = 0;
    planted.back_chain = 0;
    planted.named = Vec::new();
    let bytes = planted.to_bytes();
    let offset = record_offset(planted.counter, pool.parameters.geometry.journal_ring_bytes);
    for (_, device) in &mut pool.devices {
        device.image.write(offset, &bytes);
    }
}

/// 可写挂载：环里有一条 txg 是 `u64::MAX` 的记录，新实例第一次发布的 txg（它加一）装不下，在取号之前报
/// `SequenceNumberPastTheTopOfItsRange(CheckpointTxg(u64::MAX))`，两块盘逐字节不变。改之前加一溢出 panic。
#[test]
fn a_writable_mount_over_a_journal_record_at_the_top_txg_is_refused_before_any_write() {
    let mut pool = pool_after_the_first_file_on_a_journal_ring_of(JOURNAL_RING_DEFAULT_BYTES);
    plant_a_foreign_record_at_txg(&mut pool, CheckpointTxg(u64::MAX));
    let images_before = images_of(&pool.devices);
    let refused = mount_writable(&pool.parameters, &mut pool.devices)
        .expect_err("新实例第一次发布的 txg 装不下");
    assert!(
        matches!(
            refused,
            MountError::SequenceNumberPastTheTopOfItsRange(
                SequenceNumberAtTheTopOfItsRange::CheckpointTxg(CheckpointTxg(u64::MAX))
            )
        ),
        "报的是 txg 到顶：{refused:?}"
    );
    assert_eq!(images_of(&pool.devices), images_before, "两块盘逐字节不变");
}

/// 可写挂载：环里有一条 txg 是 `u64::MAX − 1` 的记录，写行那次发布的 txg 是 `u64::MAX`、装得下，暖机要推的下一次装不下：
/// 取号之前算暖机计划时报 `SequenceNumberPastTheTopOfItsRange(CheckpointTxg(u64::MAX))`，两块盘逐字节不变。改之前加一溢出 panic。
#[test]
fn a_writable_mount_whose_warm_up_would_pass_the_top_txg_is_refused_before_any_write() {
    let mut pool = pool_after_the_first_file_on_a_journal_ring_of(JOURNAL_RING_DEFAULT_BYTES);
    plant_a_foreign_record_at_txg(&mut pool, CheckpointTxg(u64::MAX - 1));
    let images_before = images_of(&pool.devices);
    let refused =
        mount_writable(&pool.parameters, &mut pool.devices).expect_err("暖机那一次的 txg 装不下");
    assert!(
        matches!(
            refused,
            MountError::SequenceNumberPastTheTopOfItsRange(
                SequenceNumberAtTheTopOfItsRange::CheckpointTxg(CheckpointTxg(u64::MAX))
            )
        ),
        "报的是 txg 到顶：{refused:?}"
    );
    assert_eq!(images_of(&pool.devices), images_before, "两块盘逐字节不变");
}

/// 可写挂载：盘 0 较旧的那个系统配置槽（世代号小、不被择中）里的实例代号改成 `u32::MAX`、整槽校验和重算（自证得过）。取号取
/// 每块盘两槽里全部自证过的槽的实例代号的最大值加一，装不下：在取号之前报 `SequenceNumberPastTheTopOfItsRange(InstanceGeneration(u32::MAX))`，
/// 两块盘逐字节不变（除了这里自己改的那一槽）。改之前加一溢出 panic。
#[test]
fn a_writable_mount_over_a_system_configuration_slot_at_the_top_instance_is_refused_before_any_write(
) {
    let mut pool = pool_after_the_first_file_on_a_journal_ring_of(JOURNAL_RING_DEFAULT_BYTES);
    let slot_spacing = u64::from(pool.parameters.geometry.fixed_structure_slot_spacing);
    let device_zero = &mut pool.devices[0].1;
    let older_slot = (0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE)
        .map(|slot_index| {
            let offset = singlefs_core::address::DeviceOffsetInBytes(slot_index * slot_spacing);
            let parsed = SystemConfiguration::parse_slot(&device_zero.image.read(offset, 4096))
                .expect("mkfs 流之后两槽都自证得过");
            (offset, parsed)
        })
        .min_by_key(|(_, parsed)| parsed.quantities.slot_generation)
        .expect("两槽");
    let (offset, mut at_the_top) = older_slot;
    at_the_top.quantities.journal_instance = InstanceGeneration(u32::MAX);
    device_zero.image.write(offset, &at_the_top.to_slot());
    let images_before = images_of(&pool.devices);
    let refused =
        mount_writable(&pool.parameters, &mut pool.devices).expect_err("要取的实例代号装不下");
    assert!(
        matches!(
            refused,
            MountError::SequenceNumberPastTheTopOfItsRange(
                SequenceNumberAtTheTopOfItsRange::InstanceGeneration(InstanceGeneration(u32::MAX))
            )
        ),
        "报的是实例代号到顶：{refused:?}"
    );
    assert_eq!(images_of(&pool.devices), images_before, "两块盘逐字节不变");
}

/// 抬 F（直接给新 F 的入口）与一次准入里再推一串，接在 txg 已是 `u64::MAX` 的现行那一版后面：这一串的 txg 装不下，
/// 在回收、影子账与任何写之前报 `SequenceNumberPastTheTopOfItsRange(CheckpointTxg(u64::MAX))`，两块盘逐字节不变、分配器的记录不动。
/// 改之前加一溢出 panic。
#[test]
fn raising_the_floor_after_a_version_at_the_top_txg_is_refused_before_the_allocator_moves() {
    let mut pool = pool_after_the_first_file_on_a_journal_ring_of(JOURNAL_RING_DEFAULT_BYTES);
    let mut at_the_top = at_the_top_txg(&pool.first);
    let images_before = images_of(&pool.devices);
    let records_before = pool.allocator.records().to_vec();
    let is_refused_at_the_top = |refused: &MountError| {
        matches!(
            refused,
            MountError::SequenceNumberPastTheTopOfItsRange(
                SequenceNumberAtTheTopOfItsRange::CheckpointTxg(CheckpointTxg(u64::MAX))
            )
        )
    };
    let raised = raise_rollback_floor(
        &pool.parameters,
        &mut pool.devices,
        &mut pool.allocator,
        &mut at_the_top,
        CheckpointTxg(0),
        ShadowLedger::On,
    );
    assert!(
        raised.as_ref().is_err_and(is_refused_at_the_top),
        "抬 F：{:?}",
        raised.as_ref().map(|_| "做成了")
    );
    let pushed = push_one_floor_raise_within_the_admission_budget(
        &pool.parameters,
        &mut pool.devices,
        &mut pool.allocator,
        &mut at_the_top,
        ShadowLedger::On,
        0,
    );
    assert!(
        pushed.as_ref().is_err_and(is_refused_at_the_top),
        "一次准入里再推一串：{:?}",
        pushed.as_ref().map(|_| "推了或停了")
    );
    assert_eq!(images_of(&pool.devices), images_before, "两块盘逐字节不变");
    assert_eq!(
        pool.allocator.records(),
        records_before.as_slice(),
        "分配器的记录不动"
    );
}

/// 管理员回退接在 txg 已是 `u64::MAX` 的现行那一版后面：回退那次发布的 txg 装不下，在任何写与动分配器之前报
/// `NextCheckpointTxgPastTheTopOfItsRange`，两块盘逐字节不变。改之前加一溢出 panic。
#[test]
fn rolling_back_after_a_version_at_the_top_txg_is_refused_before_any_write() {
    let mut pool = pool_after_the_first_file_on_a_journal_ring_of(JOURNAL_RING_DEFAULT_BYTES);
    let mut at_the_top = at_the_top_txg(&pool.first);
    let images_before = images_of(&pool.devices);
    let refused = roll_back_by_a_forward_publish(
        &pool.parameters,
        &mut pool.devices,
        &mut pool.allocator,
        &mut at_the_top,
        RollbackTarget {
            instance: pool.instance,
            checkpoint_txg: CheckpointTxg(3),
        },
    )
    .expect_err("回退那次发布的 txg 装不下");
    assert!(
        matches!(
            refused,
            RollbackError::NextCheckpointTxgPastTheTopOfItsRange {
                current: CheckpointTxg(u64::MAX)
            }
        ),
        "报的是 txg 到顶：{refused:?}"
    );
    assert_eq!(images_of(&pool.devices), images_before, "两块盘逐字节不变");
}

// ── 管理员回退照可写挂载同一套核调用方的参数与盘表（实审 A1b 留下的那一处） ──

/// 第一个文件（txg 3）之后覆盖写一次（txg 4）：回退到 txg 3 那一版有候选。
fn pool_after_an_overwrite() -> PoolAfterTheFirstFile {
    let mut pool = pool_after_the_first_file_on_a_journal_ring_of(JOURNAL_RING_DEFAULT_BYTES);
    let content = content_of(FIRST_FILE_BYTES, 4);
    let mut writer = PoolWriter::new(&pool.parameters, pool.devices.as_mut_slice());
    let second = publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &pool.first,
        FirstFile {
            content: &content,
            write_time_seconds: WRITE_TIME_SECONDS,
        },
        pool.instance,
    )
    .expect("覆盖写");
    drop(writer);
    pool.first = second;
    pool
}

fn target_of_the_first_file(pool: &PoolAfterTheFirstFile) -> RollbackTarget {
    RollbackTarget {
        instance: pool.instance,
        checkpoint_txg: CheckpointTxg(3),
    }
}

/// 盘表里盘 0 交了两次（第三项是盘 0 的一份拷贝）：回退在任何读写之前拒成 `CallerInputsDisagreeWithTheDisk(DeviceIdentitiesHandedInMoreThanOnce)`，
/// 三块盘逐字节不变。改之前照发：多交的那一份也收到本池的单元写与系统配置轮换。
#[test]
fn rolling_back_with_a_device_identity_handed_in_twice_is_refused_before_any_write() {
    let mut pool = pool_after_an_overwrite();
    let mut copy_of_device_zero = SparseBlockDevice::new(DEVICE_BYTES, SECTOR_PHYSICAL_BLOCK_SIZE);
    copy_of_device_zero.image = pool.devices[0].1.image.clone();
    pool.devices.push((DeviceIdentity(0), copy_of_device_zero));
    let images_before = images_of(&pool.devices);
    let target = target_of_the_first_file(&pool);
    let refused = roll_back_by_a_forward_publish(
        &pool.parameters,
        &mut pool.devices,
        &mut pool.allocator,
        &mut pool.first,
        target,
    )
    .expect_err("盘 0 交了两次");
    let RollbackError::CallerInputsDisagreeWithTheDisk(
        CallerInputsDisagreeingWithTheDisk::DeviceIdentitiesHandedInMoreThanOnce { repeated },
    ) = &refused
    else {
        panic!("该拒成重复身份，实际 {refused:?}");
    };
    assert_eq!(
        repeated,
        &vec![RepeatedDeviceIdentity {
            device: DeviceIdentity(0),
            times_handed_in: 2,
        }]
    );
    assert_eq!(images_of(&pool.devices), images_before, "三块盘逐字节不变");
}

/// 调用方给的参数里环长与盘上择到的系统配置不同：回退在任何写之前拒成
/// `CallerInputsDisagreeWithTheDisk(ParametersOrDeviceTableDisagreeWithTheSelectedSystemConfiguration { [JournalRingBytes], [] })`，
/// 两块盘逐字节不变；改之前照调用方的参数发，末尾的系统配置轮换把盘上不可变段里的环长改写掉。换回盘上那一份参数，同一次回退做成（txg 5）。
#[test]
fn rolling_back_with_other_parameters_is_refused_before_any_write_and_with_the_pool_parameters_is_made(
) {
    let mut pool = pool_after_an_overwrite();
    let mut other_parameters = pool.parameters.clone();
    other_parameters.geometry.journal_ring_bytes = JOURNAL_RING_DEFAULT_BYTES / 2;
    let images_before = images_of(&pool.devices);
    let target = target_of_the_first_file(&pool);
    let refused = roll_back_by_a_forward_publish(
        &other_parameters,
        &mut pool.devices,
        &mut pool.allocator,
        &mut pool.first,
        target,
    )
    .expect_err("环长与盘上的不同");
    let RollbackError::CallerInputsDisagreeWithTheDisk(
        CallerInputsDisagreeingWithTheDisk::ParametersOrDeviceTableDisagreeWithTheSelectedSystemConfiguration {
            disagreeing_fields,
            disagreeing_device_table,
        },
    ) = &refused
    else {
        panic!("该拒成参数与盘上不一致，实际 {refused:?}");
    };
    assert_eq!(
        disagreeing_fields,
        &vec![MakeFilesystemParameterField::JournalRingBytes]
    );
    assert!(disagreeing_device_table.is_empty());
    assert_eq!(images_of(&pool.devices), images_before, "两块盘逐字节不变");
    roll_back_by_a_forward_publish(
        &pool.parameters,
        &mut pool.devices,
        &mut pool.allocator,
        &mut pool.first,
        target,
    )
    .expect("盘上那一份参数：回退做成");
    assert_eq!(pool.first.root.checkpoint_txg, CheckpointTxg(5));
}

// ── 实审 A1b Q5：可写挂载写行与暖机之后推抬 F、抬 F 报错时交回已落盘那几次的写账 ──

const DEVICE_WIDTH_OF_THE_FLOOR_RAISE_SCENARIO: HistoryDeviceWidth =
    HistoryDeviceWidth::UnitAreaOf240Slots;

type FaultInjectingDevice = FaultInjectingBlockDevice<SparseBlockDevice>;

/// 场景照 `a_block_device_error_while_raising_the_floor_for_space_is_handed_up_as_is.rs`：两块单元区 240 槽的小盘，外面包一层故障注入
/// （计划先不上膛），mkfs、取号、暖机、第一个文件，之后崩了再挂 9 次（都做成）；第 10 次取号之前空间准入不够、写行与暖机之后推抬 F。
fn pool_after_nine_crash_remounts() -> (PoolUnderTest<FaultInjectingDevice>, SharedFaultPlan) {
    let plan = SharedFaultPlan::unarmed(DEVICE_WIDTH_OF_THE_FLOOR_RAISE_SCENARIO.fixed_geometry());
    let mut pool = PoolUnderTest::start_after_the_first_file(
        DEVICE_WIDTH_OF_THE_FLOOR_RAISE_SCENARIO,
        |identity, sparse| FaultInjectingBlockDevice::new(identity, sparse, plan.clone()),
    );
    for mount_index in 1..=9 {
        pool.crash_and_mount_writable()
            .unwrap_or_else(|error| panic!("第 {mount_index} 次崩了再挂做成：{error:?}"));
    }
    (pool, plan)
}

/// 盘 0 上从上膛起数第 `ordinal` 次系统配置槽写报块设备错，只这一次。
fn fail_the_system_configuration_write_on_device_zero(
    pool: &PoolUnderTest<FaultInjectingDevice>,
    ordinal: u64,
) -> FaultSchedule {
    FaultSchedule {
        fault: InjectedFault::WriteFails,
        device: FaultDeviceSelector::OnlyDevice(DeviceIdentity(0)),
        placement: FaultPlacement::OffsetBelow(
            SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE
                * u64::from(pool.parameters.geometry.fixed_structure_slot_spacing),
        ),
        counting: FaultCounting::PerDevice,
        occurrence: FaultOccurrence::TheNthMatchingCall(ordinal),
    }
}

/// 同一个池的盘面拷贝上不注入地挂一遍第 10 次：交回那次挂载写行、暖机各次的写账与推成的几串。
fn the_tenth_mount_on_a_copy(
    pool: &PoolUnderTest<FaultInjectingDevice>,
) -> singlefs_core::mount::Mounted {
    let mut copy_devices = common_admission::plain_devices_on(&pool.image());
    singlefs_core::mount::mount_writable_with_space_admission(
        &pool.parameters,
        &mut copy_devices,
        singlefs_core::admission::SpaceAdmission::JudgedByTheFormula,
    )
    .expect("拷贝上不注入：第 10 次挂载做成")
}

/// 第 10 次挂载推的第一串抬 F，先写系统配置那一步里盘 0 的写报块设备错（盘 0 这次挂载里的系统配置槽写依次是取号 1 次、写行那次的轮换 1 次、
/// 暖机每次 1 次，第一串先写系统配置是再下一次）。挂载交回 `FloorRaiseFailedAfterTheMountsPublishes`：`cause` 是抬 F 的错原样
/// （`RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot`，一块盘都还没带上新 F），`writes_of_persisted_publishes` 逐项等于
/// 拷贝上那一遍写行与暖机各次的写账，之前没有推成的串。改之前挂载原样交回抬 F 的错，写行与暖机的账随写入口丢掉。
#[test]
fn a_block_device_error_in_the_first_floor_raise_after_the_row_publish_hands_back_the_writes_of_the_mounts_publishes(
) {
    let (mut pool, plan) = pool_after_nine_crash_remounts();
    let rehearsal = the_tenth_mount_on_a_copy(&pool);
    assert!(
        matches!(
            &rehearsal.output.space_admission,
            MountSpaceAdmission::StillShortAfterTheFloorRaises { floor_raises, .. }
                if !floor_raises.is_empty()
        ),
        "拷贝上不注入：第 10 次推了抬 F：{:?}",
        rehearsal.output.space_admission
    );
    let writes_of_the_mounts_publishes: Vec<_> = std::iter::once(&rehearsal.output.row_publish)
        .chain(&rehearsal.output.warm_up_publishes)
        .map(|version| version.writes().clone())
        .collect();
    let system_configuration_writes_on_device_zero_before_the_first_raise =
        1 + 1 + u64::try_from(rehearsal.output.warm_up_publishes.len()).expect("暖机次数");
    plan.arm(fail_the_system_configuration_write_on_device_zero(
        &pool,
        system_configuration_writes_on_device_zero_before_the_first_raise + 1,
    ));
    let refusal = pool
        .crash_and_mount_writable()
        .expect_err("抬 F 先写系统配置那一步报块设备错");
    plan.disarm();
    let MountError::FloorRaiseFailedAfterTheMountsPublishes(failed) = refusal else {
        panic!("该交回 FloorRaiseFailedAfterTheMountsPublishes，实际 {refusal:?}");
    };
    let MountError::RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(cause) = &failed.cause
    else {
        panic!("cause 是抬 F 的错原样，实际 {:?}", failed.cause);
    };
    assert!(
        cause.devices_carrying_the_raised_floor.is_empty(),
        "盘 0 是这一步的第一个写：一块盘都还没带上新 F"
    );
    assert_eq!(
        failed.writes_of_persisted_publishes, writes_of_the_mounts_publishes,
        "写行那次与暖机各次的写账都交出来了"
    );
    assert!(
        failed.floor_raises.is_empty(),
        "第一串就报错：之前没有推成的串"
    );
}

/// 第 10 次挂载推的第二串抬 F 先写系统配置那一步里盘 0 的写报块设备错：挂载交回 `FloorRaiseFailedAfterTheMountsPublishes`，
/// `floor_raises` 里是已经推成的第一串（它的几次空发布的 txg 与拷贝上那一遍的第一串逐次相同），写行与暖机的账照样在。
#[test]
fn a_block_device_error_in_the_second_floor_raise_after_the_row_publish_hands_back_the_first_raise_too(
) {
    let (mut pool, plan) = pool_after_nine_crash_remounts();
    let rehearsal = the_tenth_mount_on_a_copy(&pool);
    let MountSpaceAdmission::StillShortAfterTheFloorRaises {
        floor_raises: rehearsed_floor_raises,
        ..
    } = &rehearsal.output.space_admission
    else {
        panic!(
            "拷贝上不注入：第 10 次推满仍不够，实际 {:?}",
            rehearsal.output.space_admission
        );
    };
    assert!(
        rehearsed_floor_raises.len() >= 2,
        "拷贝上推了至少两串：{}",
        rehearsed_floor_raises.len()
    );
    let first_raise_txgs: Vec<CheckpointTxg> = rehearsed_floor_raises[0]
        .publishes
        .iter()
        .map(|published| published.root.checkpoint_txg)
        .collect();
    let system_configuration_writes_on_device_zero_before_the_second_raise = 1
        + 1
        + u64::try_from(rehearsal.output.warm_up_publishes.len()).expect("暖机次数")
        + 1
        + u64::try_from(first_raise_txgs.len()).expect("第一串的空发布次数");
    plan.arm(fail_the_system_configuration_write_on_device_zero(
        &pool,
        system_configuration_writes_on_device_zero_before_the_second_raise + 1,
    ));
    let refusal = pool
        .crash_and_mount_writable()
        .expect_err("第二串先写系统配置那一步报块设备错");
    plan.disarm();
    let MountError::FloorRaiseFailedAfterTheMountsPublishes(failed) = refusal else {
        panic!("该交回 FloorRaiseFailedAfterTheMountsPublishes，实际 {refusal:?}");
    };
    assert!(
        matches!(
            failed.cause,
            MountError::RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(_)
        ),
        "cause 是第二串抬 F 的错原样：{:?}",
        failed.cause
    );
    assert_eq!(
        failed.writes_of_persisted_publishes.len(),
        1 + rehearsal.output.warm_up_publishes.len(),
        "写行那次与暖机各次的写账"
    );
    assert_eq!(failed.floor_raises.len(), 1, "已经推成的第一串");
    let handed_back_txgs: Vec<CheckpointTxg> = failed.floor_raises[0]
        .publishes
        .iter()
        .map(|published| published.root.checkpoint_txg)
        .collect();
    assert_eq!(
        handed_back_txgs, first_raise_txgs,
        "第一串的几次空发布与拷贝上的逐次相同"
    );
}
