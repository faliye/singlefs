//! 代码审阅第 27、36、37 条与实审 A2a 交来的「journal 环短于三条记录」（实审 A2b）的会红用例：
//! 树表里同一种树出现两条时，读树表的五个读者一律判损坏（不取首条也不取末条）；
//! 所选根的 checkpoint_txg 加一溢出时恢复报损坏、不 panic；
//! 全池槽 0 都自证不过时，在格式允许的每一档槽距上找槽 1，只收自己记的槽距等于所在偏移的那一槽，几档都收得到时取世代号最大的
//! （D22（单元原子性怎么合成） 已定项 16、D2（RAID 条带策略） 已定项 19）；
//! journal 环装不下 F 条记录时 mkfs 在任何写之前拒掉（I-8.1（环几何够大）、D23（journal 的角色与格式） 已定项 18 / 已定项 19）。
//! 盘都是内存稀疏盘（`singlefs_harness::memory_pool::SparseBlockDevice`），恢复、只读挂载与各个读者读的是 `MemoryPool`。

use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
};
use singlefs_core::allocator::{
    PoolAllocator, UnitAreaStartOffTheClusterSegmentBoundaryUnsupported,
};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::checksum::{crc32_castagnoli, wide_checksum_with_field_zeroed};
use singlefs_core::journal::JournalRecord;
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, MakeFilesystemError, MakeFilesystemOutput,
    MakeFilesystemParameters,
};
use singlefs_core::mounted_read::{
    mount_read_only, open_pool_for_read, MountReadOnlyFailure, OpenPoolForReadFailure,
};
use singlefs_core::pointer::NodePointer;
use singlefs_core::records::{TreeTableEntry, TREE_KIND_ALLOCATION, TREE_KIND_INODE};
use singlefs_core::recovery::{
    allocation_records_under_root, choose_system_configuration, rebuild_version, recover,
    user_visible_tree_root_pointers, walk_to_file, JournalPolicy, PoolReader,
    RebuildVersionFailure, RecoveryFailure, RecoveryOutcome,
    CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR, TREE_TABLE_CARRIES_ONE_KIND_OF_TREE_TWICE,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, RootRingSlot, RootRingSlotsPerRegion};
use singlefs_core::system_configuration::{
    SystemConfiguration, SystemImmutableSizes,
    INCOMPAT_RETIRED_FIRST_SSD_LINE_WITH_ROLLBACK_WITNESS_BIT,
    SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, warm_up, FirstFile, PoolWriter, TransactionOutput,
};
use singlefs_core::unit::{build_index_node, parse_index_node};
use singlefs_format::{
    JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES, JOURNAL_RING_START_SLOT,
    JOURNAL_SAFETY_FACTOR, NODE_BYTES, SLOT_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES,
    UNIT_AREA_START_SLOT,
};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

const DEVICE_BYTES: u64 = 4 << 30;
const MEBIBYTE: u64 = 1 << 20;
const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"singlefs-rev-a2b";
/// 另一个池的 fsid：更早一次 mkfs 留下的槽用它。
const ANOTHER_FILESYSTEM_IDENTIFIER: [u8; 16] = *b"singlefs-earlier";
const WRITE_TIME_SECONDS: u64 = 1_788_100_000;
/// 内存稀疏盘按 512 字节扇区存：物理块宽 512。
const SECTOR_PHYSICAL_BLOCK_SIZE: PhysicalBlockSizeInBytes = PhysicalBlockSizeInBytes(512);
/// 系统配置槽里一个只有整槽校验和罩着的保留字节（与 `system_configuration_per_device_redundancy.rs` 同一处）：
/// 翻掉它，这一槽读得出、自证不过。
const BYTE_IN_THE_RESERVED_AREA_OF_A_SYSTEM_CONFIGURATION_SLOT: u64 = 2048;
/// 系统配置槽里 incompat 位图第一个字节的偏移：magic 4 + 格式版本 2（与 `system_configuration_rollback_floor_and_layout_identity.rs` 同一处）。
const FIRST_INCOMPAT_BYTE_OFFSET: usize = 6;
/// 系统配置槽里每区槽数 S 那一字节的偏移（与 `system_configuration_slots_per_region.rs` 同一处）。
const SLOTS_PER_REGION_OFFSET_IN_THE_SLOT: usize = 362;
/// 根环基址 1 MiB（系统配置「根环起点」字段 64 × 16 KiB，D22（单元原子性怎么合成） 已定项 16 第 1 句）：
/// 槽 1 整槽落在它之前，槽距最大是它减一个系统配置槽宽。
const ROOT_RING_BASE_IN_BYTES: u64 = MEBIBYTE;
/// 拒绝之前先在每个固定结构的起点写的花样：稀疏盘上「清零」是删扇区，全零的盘上看不出 mkfs 清没清过。
const PATTERN_BYTE: u8 = 0xa5;
const PATTERN_BYTES: usize = 4096;

fn geometry_with_minimum_input_output_bytes(
    minimum_input_output_bytes: u32,
) -> SystemImmutableSizes {
    SystemImmutableSizes {
        physical_block_size: 512,
        minimum_input_output_bytes,
        fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(
            minimum_input_output_bytes,
        ),
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
            u8::try_from((index * 137 + seed * 11 + 3) % 251).expect("小于 256")
        })
        .collect()
}

fn two_sparse_devices() -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| {
            (
                identity,
                SparseBlockDevice::new(DEVICE_BYTES, SECTOR_PHYSICAL_BLOCK_SIZE),
            )
        })
        .collect()
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

/// 两块内存盘上 mkfs → 取号 → 暖机 → 第一个文件（txg 3），分配器是 mkfs 同一个进程那一条。
struct PoolAfterTheFirstFile {
    image: MemoryPool,
    instance: InstanceGeneration,
    first_content: Vec<u8>,
    first: TransactionOutput,
    geometry: SystemImmutableSizes,
}

fn pool_after_the_first_file() -> PoolAfterTheFirstFile {
    let parameters = parameters_with(geometry_with_minimum_input_output_bytes(512));
    let mut devices = two_sparse_devices();
    let genesis = make_filesystem(&parameters, &mut devices).expect("默认几何上 mkfs");
    let mut allocator: PoolAllocator =
        allocator_after_make_filesystem(&parameters, &devices, &genesis);
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
        image: image_of(&devices),
        instance,
        first_content,
        first,
        geometry: parameters.geometry,
    }
}

// ── 第 27 条：树表里同一种树出现两条 ──

fn node_bytes() -> usize {
    usize::try_from(NODE_BYTES).expect("16384")
}

/// 一条根指着的树表单元，按它第一条位置条目读回来。
fn tree_table_unit_of(image: &MemoryPool, root: &RootRecord) -> Vec<u8> {
    let location = root.tree_table.locations[0];
    PoolReader::read(
        image,
        location.device,
        location.slot.to_device_offset(),
        node_bytes(),
    )
    .expect("树表单元读得到")
}

/// 用 `entries` 照原树表单元的头重建一个树表单元，写回它两条位置条目指的槽（两块盘各一份），
/// 交回一条除树表指针两条校验和之外与 `root` 相同的根：各个读者拿这条根去读，读到的就是改过的树表。
fn root_whose_tree_table_carries(
    image: &mut MemoryPool,
    root: &RootRecord,
    entries: &[Vec<u8>],
) -> RootRecord {
    let header = parse_index_node(&tree_table_unit_of(image, root)).expect("树表单元解得开");
    let rebuilt = build_index_node(
        header.tree,
        header.level,
        header.key_width,
        &header.smallest_key,
        &header.largest_key,
        header.birth_txg,
        &FILESYSTEM_IDENTIFIER,
        header.instance,
        header.birth_sequence,
        u16::try_from(header.entry_width).expect("条目宽 200"),
        entries,
    );
    let checksum = crc32_castagnoli(&rebuilt);
    let mut rewritten_root = *root;
    for location in &mut rewritten_root.tree_table.locations {
        image
            .devices
            .get_mut(&location.device)
            .expect("池里有这块盘")
            .write(location.slot.to_device_offset(), &rebuilt);
        location.unit_checksum = checksum;
    }
    rewritten_root
}

/// 读树表的五个读者各自读一条根的结局（读下去了是 `Ok(())`）。
#[derive(Debug, PartialEq, Eq)]
struct TreeTableReaderOutcomes {
    rebuild_version: Result<(), RebuildVersionFailure>,
    allocation_records_under_root: Result<(), RecoveryFailure>,
    walk_to_file: Result<(), RecoveryFailure>,
    user_visible_tree_root_pointers: Result<(), RecoveryFailure>,
    open_pool_for_read: Result<(), OpenPoolForReadFailure>,
}

fn outcomes_of_every_tree_table_reader(
    image: &MemoryPool,
    root: &RootRecord,
    record_standing_for_root: &JournalRecord,
) -> TreeTableReaderOutcomes {
    let mut mapping_fallbacks = 0;
    TreeTableReaderOutcomes {
        rebuild_version: rebuild_version(image, root, Some(record_standing_for_root.clone()))
            .map(|_rebuilt| ()),
        allocation_records_under_root: allocation_records_under_root(image, root)
            .map(|_records| ()),
        walk_to_file: walk_to_file(image, root, &mut mapping_fallbacks).map(|_content| ()),
        user_visible_tree_root_pointers: user_visible_tree_root_pointers(image, root)
            .map(|_pointers| ()),
        open_pool_for_read: open_pool_for_read(image, root).map(|_mounted| ()),
    }
}

fn every_reader_reads_on() -> TreeTableReaderOutcomes {
    TreeTableReaderOutcomes {
        rebuild_version: Ok(()),
        allocation_records_under_root: Ok(()),
        walk_to_file: Ok(()),
        user_visible_tree_root_pointers: Ok(()),
        open_pool_for_read: Ok(()),
    }
}

fn every_reader_refuses_the_tree_table_as_damaged() -> TreeTableReaderOutcomes {
    let damaged = RecoveryFailure::UnitMalformed {
        what: TREE_TABLE_CARRIES_ONE_KIND_OF_TREE_TWICE,
    };
    TreeTableReaderOutcomes {
        rebuild_version: Err(RebuildVersionFailure::Walk(damaged.clone())),
        allocation_records_under_root: Err(damaged.clone()),
        walk_to_file: Err(damaged.clone()),
        user_visible_tree_root_pointers: Err(damaged.clone()),
        open_pool_for_read: Err(OpenPoolForReadFailure::Walk(damaged)),
    }
}

/// 对照：原样重建的树表与写者写下的那一份逐字节相同，五个读者都读得下去——改过的那几份判红只因为多出来的那一条。
/// 分配记录树的条目原样多抄一条：改前重建取首条、分配记录与走读取末条、另两处不看这一种，五处都读下去了；
/// inode 树多一条根指针全零的条目、排在真的那一条前面：改前挂载态读取首条非空的、走读跳过全零的，也都读下去了。
/// 改后五处一律报同一个损坏。
#[test]
fn a_tree_table_carrying_one_kind_of_tree_twice_is_refused_as_damaged_by_every_reader() {
    let mut pool = pool_after_the_first_file();
    let root = pool.first.root;
    let record = pool.first.record.clone();
    let original_unit = tree_table_unit_of(&pool.image, &root);
    let original_entries = parse_index_node(&original_unit)
        .expect("树表单元解得开")
        .entries;
    let unchanged_root = root_whose_tree_table_carries(&mut pool.image, &root, &original_entries);
    assert_eq!(
        tree_table_unit_of(&pool.image, &unchanged_root),
        original_unit,
        "原样重建的树表单元与写者写下的逐字节相同：重建这一步不改别的"
    );
    assert_eq!(unchanged_root, root, "条目不变，根也不变");
    assert_eq!(
        outcomes_of_every_tree_table_reader(&pool.image, &unchanged_root, &record),
        every_reader_reads_on(),
        "对照：原样的树表五个读者都读得下去"
    );

    let kind_of = |entry_bytes: &Vec<u8>| {
        TreeTableEntry::parse(entry_bytes)
            .expect("写者写的条目解得开")
            .kind
    };
    let mut allocation_twice = Vec::new();
    for entry_bytes in &original_entries {
        allocation_twice.push(entry_bytes.clone());
        if kind_of(entry_bytes) == TREE_KIND_ALLOCATION {
            allocation_twice.push(entry_bytes.clone());
        }
    }
    assert_eq!(
        allocation_twice.len(),
        original_entries.len() + 1,
        "分配记录树恰好一条，多抄了一条"
    );
    let root_with_allocation_twice =
        root_whose_tree_table_carries(&mut pool.image, &root, &allocation_twice);
    assert_eq!(
        outcomes_of_every_tree_table_reader(&pool.image, &root_with_allocation_twice, &record),
        every_reader_refuses_the_tree_table_as_damaged(),
        "分配记录树两条（逐字节相同）：五个读者都判树表损坏"
    );

    let mut inode_twice_first_without_root = Vec::new();
    for entry_bytes in &original_entries {
        let entry = TreeTableEntry::parse(entry_bytes).expect("写者写的条目解得开");
        if entry.kind == TREE_KIND_INODE {
            let mut without_root = entry;
            without_root.root = NodePointer::empty_root();
            inode_twice_first_without_root.push(without_root.to_bytes());
        }
        inode_twice_first_without_root.push(entry_bytes.clone());
    }
    let root_with_inode_twice =
        root_whose_tree_table_carries(&mut pool.image, &root, &inode_twice_first_without_root);
    assert_eq!(
        outcomes_of_every_tree_table_reader(&pool.image, &root_with_inode_twice, &record),
        every_reader_refuses_the_tree_table_as_damaged(),
        "inode 树两条（前一条根指针全零）：五个读者都判树表损坏"
    );
}

// ── 第 36 条：所选根的 checkpoint_txg 加一溢出 ──

/// 在根环区域 0 的槽 7（txg 3 之前的发布都没用到它）写一条 `root` 的根记录、checkpoint_txg 换成 `checkpoint_txg`：
/// 它的 txg 最大，择根择中它。
fn image_whose_chosen_root_has_the_checkpoint_txg(
    pool: &PoolAfterTheFirstFile,
    checkpoint_txg: CheckpointTxg,
) -> MemoryPool {
    let mut image = pool.image.clone();
    let mut planted = pool.first.root;
    planted.checkpoint_txg = checkpoint_txg;
    let ring_slot = RootRingSlot { region: 0, slot: 7 };
    image
        .devices
        .get_mut(&DeviceIdentity(0))
        .expect("区域 0 在盘 0")
        .write(
            slot_offset(ring_slot, pool.geometry.fixed_structure_slot_spacing),
            &planted.to_slot(usize::try_from(pool.geometry.physical_block_size).expect("根槽宽")),
        );
    image
}

/// 所选根的 checkpoint_txg 是 u64 最大值（自证校验和对得上、别的字段都是第一个文件那一版的）：锚点读不出时链首那条的 txg 要取它加一，
/// 改前在这一步溢出、只读挂载与恢复 panic；改后在施加任何一条记录之前报损坏。
/// 边界：最大值减一加一不溢出，恢复照常读回第一个文件。
#[test]
fn a_chosen_root_whose_checkpoint_txg_is_the_largest_value_is_refused_as_damaged_instead_of_panicking(
) {
    let pool = pool_after_the_first_file();
    let largest = CheckpointTxg(u64::MAX);
    let image = image_whose_chosen_root_has_the_checkpoint_txg(&pool, largest);
    let damaged = RecoveryFailure::UnitMalformed {
        what: CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR,
    };
    let recovered = recover(&image, JournalPolicy::Consult);
    assert_eq!(
        recovered.outcome,
        RecoveryOutcome::Failed {
            root: Some((pool.instance, largest)),
            failure: damaged.clone(),
        },
        "txg 最大值的根：恢复报损坏"
    );
    assert_eq!(
        recovered.effective_root, None,
        "一条记录都没施加，没有「实际走的那条根」"
    );
    assert_eq!(
        mount_read_only(&image).err(),
        Some(MountReadOnlyFailure::Recovery(damaged)),
        "txg 最大值的根：只读挂载报同一个损坏"
    );

    let one_below = CheckpointTxg(u64::MAX - 1);
    let image_one_below = image_whose_chosen_root_has_the_checkpoint_txg(&pool, one_below);
    let recovered_one_below = recover(&image_one_below, JournalPolicy::Consult);
    assert_eq!(
        recovered_one_below.outcome,
        RecoveryOutcome::FileRead {
            root: (pool.instance, one_below),
            content: pool.first_content.clone(),
        },
        "最大值减一：加一不溢出，恢复读回第一个文件"
    );
    assert!(
        mount_read_only(&image_one_below).is_ok(),
        "最大值减一：只读挂载挂得上"
    );
}

// ── 第 37 条：全池槽 0 都自证不过时逐档找槽 1 ──

fn slot_bytes() -> usize {
    usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096")
}

/// 按 `minimum_input_output_bytes` 算的槽距做一次 mkfs（没有文件），交回内存池。
fn formatted_image_with_minimum_input_output_bytes(minimum_input_output_bytes: u32) -> MemoryPool {
    let parameters = parameters_with(geometry_with_minimum_input_output_bytes(
        minimum_input_output_bytes,
    ));
    let mut devices = two_sparse_devices();
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    image_of(&devices)
}

fn flip_slot_zero_on_every_device(image: &mut MemoryPool) {
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        image.flip_byte(
            device,
            DeviceOffsetInBytes(0),
            BYTE_IN_THE_RESERVED_AREA_OF_A_SYSTEM_CONFIGURATION_SLOT,
        );
    }
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let slot_zero = PoolReader::read(image, device, DeviceOffsetInBytes(0), slot_bytes())
            .expect("槽 0 读得到");
        assert!(
            SystemConfiguration::parse_slot(&slot_zero).is_err(),
            "盘 {device:?} 的槽 0 自证不过"
        );
    }
}

fn slot_at(image: &MemoryPool, device: DeviceIdentity, offset: u64) -> SystemConfiguration {
    SystemConfiguration::parse_slot(
        &PoolReader::read(image, device, DeviceOffsetInBytes(offset), slot_bytes())
            .expect("槽读得到"),
    )
    .expect("这一槽自证得过")
}

fn plant_slot(
    image: &mut MemoryPool,
    device: DeviceIdentity,
    offset: u64,
    system_configuration: &SystemConfiguration,
) {
    image
        .devices
        .get_mut(&device)
        .expect("池里有这块盘")
        .write(DeviceOffsetInBytes(offset), &system_configuration.to_slot());
}

/// 整槽校验和重封：改过字节的槽 magic 与校验和两关照样过，拒不拒只看被改的那一格。
fn reseal_system_configuration_slot(slot: &mut [u8]) {
    let digest =
        wide_checksum_with_field_zeroed(slot, slot_bytes(), SYSTEM_CONFIGURATION_CHECKSUM_OFFSET);
    slot[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32]
        .copy_from_slice(&digest);
}

fn rewrite_slot_bytes(
    image: &mut MemoryPool,
    device: DeviceIdentity,
    offset: u64,
    change: impl Fn(&mut Vec<u8>),
) {
    let mut slot = PoolReader::read(image, device, DeviceOffsetInBytes(offset), slot_bytes())
        .expect("槽读得到");
    change(&mut slot);
    reseal_system_configuration_slot(&mut slot);
    image
        .devices
        .get_mut(&device)
        .expect("池里有这块盘")
        .write(DeviceOffsetInBytes(offset), &slot);
}

/// 审阅报的场景：槽距 8192（io_min 8192）的池，两块盘的槽 0 同时自证不过（同一步轮换写槽 0 时掉电）。
/// 改前按最小槽距 4096 找槽 1，两块盘都找不到、整池报没有有效的系统配置；改后逐档试到 8192，择到盘 0 的槽 1，恢复照常走到第 0 代根。
#[test]
fn slot_one_is_found_by_trying_every_spacing_when_no_slot_zero_in_the_pool_verifies() {
    let spacing = 8192;
    let mut image = formatted_image_with_minimum_input_output_bytes(spacing);
    flip_slot_zero_on_every_device(&mut image);
    let chosen = choose_system_configuration(&image).expect("两块盘的槽 1 都在槽距 8192 处");
    assert_eq!(
        (
            chosen.immutable.this_device,
            chosen.immutable.sizes.fixed_structure_slot_spacing,
            chosen.quantities.slot_generation,
        ),
        (DeviceIdentity(0), spacing, 1),
        "择到盘 0 在 8192 处的槽 1（mkfs 种的世代号 1）"
    );
    assert_eq!(
        recover(&image, JournalPolicy::Consult).outcome,
        RecoveryOutcome::NoFile {
            root: (InstanceGeneration(0), CheckpointTxg(0)),
        },
        "系统配置择到了，恢复走到 mkfs 的第 0 代根"
    );
}

/// 槽距只是 512 的整数倍、不是 1024 的（io_min 1536 ⇒ 槽距 4608，D2（RAID 条带策略） 已定项 19 的式子）：逐档按 512 试才试得到。
#[test]
fn a_slot_spacing_that_is_a_multiple_of_512_bytes_only_is_found_too() {
    let mut image = formatted_image_with_minimum_input_output_bytes(1536);
    let spacing = SystemImmutableSizes::slot_spacing_for(1536);
    assert_eq!(spacing, 4608, "io_min 1536 时槽距 4608");
    flip_slot_zero_on_every_device(&mut image);
    let chosen = choose_system_configuration(&image).expect("两块盘的槽 1 都在 4608 处");
    assert_eq!(
        chosen.immutable.sizes.fixed_structure_slot_spacing, spacing,
        "择到 4608 处的槽 1"
    );
}

/// 槽距 8192 的池，4096 处放一份自证得过、世代号 9、记着槽距 8192 的槽（它不在自己记的槽距上）：
/// 改前按 4096 读到它就收；改后只收记的槽距等于所在偏移的那一槽，择到 8192 处世代号 1 的那一份。
#[test]
fn a_self_verifying_slot_whose_recorded_spacing_is_not_its_own_offset_is_not_taken() {
    let spacing = 8192;
    let mut image = formatted_image_with_minimum_input_output_bytes(spacing);
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let mut misplaced = slot_at(&image, device, u64::from(spacing));
        misplaced.quantities.slot_generation = 9;
        plant_slot(&mut image, device, 4096, &misplaced);
    }
    flip_slot_zero_on_every_device(&mut image);
    let chosen = choose_system_configuration(&image).expect("8192 处的槽 1 自证得过");
    assert_eq!(
        (
            chosen.immutable.sizes.fixed_structure_slot_spacing,
            chosen.quantities.slot_generation,
        ),
        (spacing, 1),
        "4096 处那一份记的槽距是 8192，不收；择到 8192 处世代号 1 的"
    );
}

/// 风险那一格（写进 `recovery::choose_system_configuration` 的射程）：槽距 4096 的池，8192 处躺着一份更早一次 mkfs 留下的槽 1
/// （同一个 fsid、记着槽距 8192、世代号 5，mkfs 不清它）。全池槽 0 都自证不过时，两份都自证得过、记的槽距都等于所在偏移、分不出来，
/// 按择槽取世代号最大的（D22（单元原子性怎么合成） 已定项 16，逐盘计）择到的是旧的那一份。改前只试 4096，择到本池的。
#[test]
fn the_search_takes_the_highest_generation_even_when_it_is_a_stale_slot_of_an_earlier_make_filesystem(
) {
    let mut image = formatted_image_with_minimum_input_output_bytes(512);
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let mut stale = slot_at(&image, device, 4096);
        stale.immutable.sizes.fixed_structure_slot_spacing = 8192;
        stale.quantities.slot_generation = 5;
        plant_slot(&mut image, device, 8192, &stale);
    }
    flip_slot_zero_on_every_device(&mut image);
    let chosen = choose_system_configuration(&image).expect("4096 与 8192 两处都自证得过");
    assert_eq!(
        (
            chosen.immutable.sizes.fixed_structure_slot_spacing,
            chosen.quantities.slot_generation,
        ),
        (8192, 5),
        "世代号 5 的那一份（旧 mkfs 留下的）胜过本池世代号 1 的"
    );
}

/// 逐档试只在全池没有一个槽 0 可择时才走：只有盘 0 的槽 0 自证不过、盘 1 的槽 0 可择（记着槽距 4096），盘 0 的槽 1 照旧按
/// 借来的 4096 找，盘 0 上 8192 处那份世代号 5 的旧槽读都不读（改前改后都绿）。盘 0 不借、也逐档试，择到的就是那份旧槽。
#[test]
fn a_device_whose_slot_zero_fails_borrows_the_spacing_another_slot_zero_records_instead_of_searching(
) {
    let mut image = formatted_image_with_minimum_input_output_bytes(512);
    let mut stale = slot_at(&image, DeviceIdentity(0), 4096);
    stale.immutable.sizes.fixed_structure_slot_spacing = 8192;
    stale.quantities.slot_generation = 5;
    plant_slot(&mut image, DeviceIdentity(0), 8192, &stale);
    image.flip_byte(
        DeviceIdentity(0),
        DeviceOffsetInBytes(0),
        BYTE_IN_THE_RESERVED_AREA_OF_A_SYSTEM_CONFIGURATION_SLOT,
    );
    let chosen = choose_system_configuration(&image).expect("盘 0 的槽 1 与盘 1 两槽都自证得过");
    assert_eq!(
        (
            chosen.immutable.this_device,
            chosen.immutable.sizes.fixed_structure_slot_spacing,
            chosen.quantities.slot_generation,
        ),
        (DeviceIdentity(0), 4096, 1),
        "盘 0 按盘 1 槽 0 记的 4096 找到本池的槽 1，不去试 8192"
    );
}

/// 世代号相同时取偏移小的那一档（与槽 0、槽 1 世代号相同取槽 0 同一个次序）：槽距 4096 的池，8192 处躺着一份世代号同为 1、
/// 记着槽距 8192 的旧槽，择到的是 4096 处本池的那一份。
#[test]
fn slots_of_equal_generation_found_by_the_search_resolve_to_the_smaller_offset() {
    let mut image = formatted_image_with_minimum_input_output_bytes(512);
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let mut stale = slot_at(&image, device, 4096);
        stale.immutable.sizes.fixed_structure_slot_spacing = 8192;
        plant_slot(&mut image, device, 8192, &stale);
    }
    flip_slot_zero_on_every_device(&mut image);
    let chosen = choose_system_configuration(&image).expect("4096 与 8192 两处都自证得过");
    assert_eq!(
        (
            chosen.immutable.sizes.fixed_structure_slot_spacing,
            chosen.quantities.slot_generation,
        ),
        (4096, 1),
        "世代号都是 1：取偏移小的 4096 那一份"
    );
}

/// 两块盘各自择到的世代号最大的那一份 fsid 不同（盘 0 的 8192 处躺着另一个池的槽、世代号 5）：各盘 fsid 对不上，整池拒。
/// 改前两块盘都只试 4096、都择到本池的，挂得上。
#[test]
fn devices_whose_highest_found_slots_disagree_on_the_filesystem_identifier_refuse_the_pool() {
    let mut image = formatted_image_with_minimum_input_output_bytes(512);
    let mut another_pool = slot_at(&image, DeviceIdentity(0), 4096);
    another_pool.immutable.filesystem_identifier = ANOTHER_FILESYSTEM_IDENTIFIER;
    another_pool.immutable.sizes.fixed_structure_slot_spacing = 8192;
    another_pool.quantities.slot_generation = 5;
    plant_slot(&mut image, DeviceIdentity(0), 8192, &another_pool);
    flip_slot_zero_on_every_device(&mut image);
    assert_eq!(
        choose_system_configuration(&image).err(),
        Some(RecoveryFailure::SystemConfigurationsDisagree),
        "盘 0 择到另一个池的、盘 1 择到本池的：fsid 对不上"
    );
}

/// 上界：槽 1 整槽落在根环基址之前，槽距最大是 1 MiB − 4096；那一档也试（盘 0 那里放一份记着这个槽距、世代号 2 的槽）。
#[test]
fn the_search_tries_the_largest_spacing_whose_slot_one_ends_at_the_root_ring_base() {
    let largest_spacing = ROOT_RING_BASE_IN_BYTES - SYSTEM_CONFIGURATION_SLOT_BYTES;
    let mut image = formatted_image_with_minimum_input_output_bytes(512);
    let mut at_the_upper_end = slot_at(&image, DeviceIdentity(0), 4096);
    at_the_upper_end
        .immutable
        .sizes
        .fixed_structure_slot_spacing = u32::try_from(largest_spacing).expect("装得进 u32");
    at_the_upper_end.quantities.slot_generation = 2;
    plant_slot(
        &mut image,
        DeviceIdentity(0),
        largest_spacing,
        &at_the_upper_end,
    );
    flip_slot_zero_on_every_device(&mut image);
    let chosen = choose_system_configuration(&image).expect("盘 0 上两处、盘 1 上一处自证得过");
    assert_eq!(
        (
            chosen.immutable.this_device,
            u64::from(chosen.immutable.sizes.fixed_structure_slot_spacing),
            chosen.quantities.slot_generation,
        ),
        (DeviceIdentity(0), largest_spacing, 2),
        "盘 0 择到 1 MiB − 4096 处世代号 2 的那一份"
    );
}

/// 保留下来的分流（改前改后都绿）：两块盘槽 0 自证不过、4096 处的槽 1 自证得过而 incompat 位图不认识 ⇒ 报布局不认识、点名盘 0；
/// 盘 0 的槽 1 自证得过而每区槽数 S 越界 ⇒ 整池拒、点名盘 0。逐档试的那一段与读槽 0、槽 1 走同一个分流。
#[test]
fn the_search_keeps_the_incompat_and_slots_per_region_refusals_of_slot_one() {
    let mut incompat_image = formatted_image_with_minimum_input_output_bytes(512);
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        rewrite_slot_bytes(&mut incompat_image, device, 4096, |slot| {
            slot[FIRST_INCOMPAT_BYTE_OFFSET] =
                INCOMPAT_RETIRED_FIRST_SSD_LINE_WITH_ROLLBACK_WITNESS_BIT;
        });
    }
    flip_slot_zero_on_every_device(&mut incompat_image);
    let Err(RecoveryFailure::SystemConfigurationIncompatBitsNotRecognized {
        first_device_carrying_them,
        incompat_bitmap,
    }) = choose_system_configuration(&incompat_image)
    else {
        panic!(
            "槽 1 布局不认识：该报 incompat 位图不认识，实际 {:?}",
            choose_system_configuration(&incompat_image).err()
        )
    };
    assert_eq!(
        (first_device_carrying_them, incompat_bitmap.0[0]),
        (
            DeviceIdentity(0),
            INCOMPAT_RETIRED_FIRST_SSD_LINE_WITH_ROLLBACK_WITNESS_BIT
        ),
        "点名盘 0、带着它槽 1 里的位图"
    );

    let mut slots_per_region_image = formatted_image_with_minimum_input_output_bytes(512);
    rewrite_slot_bytes(
        &mut slots_per_region_image,
        DeviceIdentity(0),
        4096,
        |slot| {
            slot[SLOTS_PER_REGION_OFFSET_IN_THE_SLOT] = 17;
        },
    );
    flip_slot_zero_on_every_device(&mut slots_per_region_image);
    let Err(RecoveryFailure::RootRingSlotsPerRegionOutOfRange {
        device,
        out_of_range,
    }) = choose_system_configuration(&slots_per_region_image)
    else {
        panic!(
            "槽 1 的 S 越界：该整池拒，实际 {:?}",
            choose_system_configuration(&slots_per_region_image).err()
        )
    };
    assert_eq!(
        (device, out_of_range.declared_slots_per_region),
        (DeviceIdentity(0), 17),
        "点名盘 0、带着 S = 17"
    );
}

// ── journal 环短于三条记录：mkfs 拒 ──

/// 在两块带花样的盘上做一次 mkfs：交回结果、录制流里记了几步、两块盘是不是与 mkfs 之前逐字节相同。
struct MakeFilesystemObservation {
    result: Result<MakeFilesystemOutput, MakeFilesystemError>,
    recorded_operations: usize,
    images_unchanged: bool,
}

fn make_filesystem_on_patterned_devices(
    parameters: &MakeFilesystemParameters,
) -> MakeFilesystemObservation {
    let stream = SharedStream::new();
    let mut devices = Vec::new();
    let mut images_before: Vec<SparseDevice> = Vec::new();
    for identity in [DeviceIdentity(0), DeviceIdentity(1)] {
        let mut device = SparseBlockDevice::new(DEVICE_BYTES, SECTOR_PHYSICAL_BLOCK_SIZE);
        for offset in [
            0,
            4096,
            MEBIBYTE,
            4 * MEBIBYTE,
            7 * MEBIBYTE,
            JOURNAL_RING_START_SLOT * SLOT_BYTES,
            UNIT_AREA_START_SLOT * SLOT_BYTES,
        ] {
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
        .all(|((_, device), before)| device.wrapped_device().image == *before);
    MakeFilesystemObservation {
        result,
        recorded_operations: stream.operations().len(),
        images_unchanged,
    }
}

fn ring_of(ring_bytes: u64) -> MakeFilesystemParameters {
    let mut geometry = geometry_with_minimum_input_output_bytes(512);
    geometry.journal_ring_bytes = ring_bytes;
    parameters_with(geometry)
}

/// 在飞上限 = 环槽数 ÷ F（F = 3，D23（journal 的角色与格式） 已定项 18）：环装不下 3 条记录时上限是 0，恢复一条记录都不施加。
/// 一条记录长（改前放行）与差一个字节够三条的环都在任何写之前拒掉。恰好三条的环过了这一判，由紧跟着的段边界那一判在任何写之前拒
/// （环末尾的下一个槽 1025 不在 64 槽聚簇段边界上，实审 A3b；主 agent 定不为测试开口子）；段边界上最短的 1 MiB 环（在飞上限 85）做成。
/// 改之前这一格钉「恰好三条放行」：A3b 让单元区起点随环长走之后，环长不是 1 MiB 整数倍的环都被段边界那一判拒。
#[test]
fn a_journal_ring_holding_fewer_records_than_the_safety_factor_is_refused_before_any_write() {
    let minimum_ring_bytes = JOURNAL_SAFETY_FACTOR * JOURNAL_RECORD_BYTES;
    assert_eq!(minimum_ring_bytes, 12288, "三条 4 KiB 记录");
    for ring_bytes in [
        JOURNAL_RECORD_BYTES,
        2 * JOURNAL_RECORD_BYTES,
        minimum_ring_bytes - 1,
    ] {
        let observation = make_filesystem_on_patterned_devices(&ring_of(ring_bytes));
        assert!(
            matches!(
                observation.result,
                Err(MakeFilesystemError::JournalRingHoldsFewerRecordsThanTheSafetyFactor {
                    ring_bytes: refused_ring_bytes,
                    minimum_ring_bytes: refused_minimum,
                }) if refused_ring_bytes == ring_bytes && refused_minimum == minimum_ring_bytes
            ),
            "环 {ring_bytes} 字节装不下三条记录：{:?}",
            observation.result.as_ref().err()
        );
        assert_eq!(
            observation.recorded_operations, 0,
            "环 {ring_bytes} 字节：拒绝之前录制流里一步都没有"
        );
        assert!(
            observation.images_unchanged,
            "环 {ring_bytes} 字节：两块盘与 mkfs 之前逐字节相同"
        );
    }
    let exactly_three = make_filesystem_on_patterned_devices(&ring_of(minimum_ring_bytes));
    assert!(
        matches!(
            exactly_three.result,
            Err(MakeFilesystemError::UnitAreaStartOffTheClusterSegmentBoundaryUnsupported(
                UnitAreaStartOffTheClusterSegmentBoundaryUnsupported {
                    journal_ring_bytes,
                    slot_after_the_journal_ring,
                }
            )) if journal_ring_bytes == minimum_ring_bytes
                && slot_after_the_journal_ring == SlotNumber(JOURNAL_RING_START_SLOT + 1)
        ),
        "恰好三条记录长的环过了在飞上限那一判，被段边界那一判拒：{:?}",
        exactly_three.result.as_ref().err()
    );
    assert_eq!(
        exactly_three.recorded_operations, 0,
        "恰好三条记录长的环：拒绝之前录制流里一步都没有"
    );
    assert!(
        exactly_three.images_unchanged,
        "恰好三条记录长的环：两块盘与 mkfs 之前逐字节相同"
    );
    let one_mebibyte = make_filesystem_on_patterned_devices(&ring_of(MEBIBYTE));
    assert!(
        one_mebibyte.result.is_ok(),
        "1 MiB 的环（末尾的下一个槽 1088 = 17 × 64）做成：{:?}",
        one_mebibyte.result.as_ref().err()
    );
}
