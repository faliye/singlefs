//! 实审 A3a：盘上内容可控时读者走得到的 panic（代码审阅第 29、32、33、35、38 条，C476 落在 core 读者这一侧的那几处）。
//! 规矩（`/tmp` 规格与 `code-discipline.md`「错误：能恢复的写进类型，破坏不变量的就断言」）：盘上读到的坏内容是可恢复的失败，
//! 每一处改成带名字的错误成员往上交；这里每条用例造一份让改之前那一版 panic、静默收下或报别的成员的坏镜像（各条文档写明改之前是哪一种），钉住改之后的结局。
//!
//! 盘都是内存稀疏盘（`singlefs_harness::memory_pool::SparseBlockDevice`），池在进程里 mkfs、取号、暖机、写第一个文件（txg 3）造出来；
//! 改盘上字节之后，指着它的每一道校验和（父指针里的位置条目、映射条目、树表、根记录的自证校验和）都重算，读者走到被改的那一格才判得到。
//! 「在任何写之前拒」用两块盘此刻的整份镜像比：拒之前、拒之后逐字节相同。

use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
    TreeIdentifier,
};
use singlefs_core::allocator::{AllocationRecord, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::bytes::{ByteReader, ByteWriter};
use singlefs_core::checksum::{crc32_castagnoli, wide_checksum_with_field_zeroed};
use singlefs_core::code_two_tree::{
    build_internal_entry, parse_internal_entry, InternalEntryRefusal,
};
use singlefs_core::journal::{
    JournalRecord, JournalRecordOrdinalWithinPublish, JournalRecordPlaceInPublish,
    JOURNAL_HEADER_CHECKSUM_OFFSET,
};
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
};
use singlefs_core::mount::{mount_writable, MountError, StillUnreadableAfterOneReread};
use singlefs_core::pointer::{
    BirthSequence, DataPointer, LocationEntry, NodePointer, PointerHead,
    PointerHeadFieldOutsideTheFirstVersion,
};
use singlefs_core::records::{
    mapping_key_for_node, parse_mapping_entry, AccountingEntry, TreeTableEntry,
    STATISTIC_INODE_WATERMARK, TREE_KIND_ACCOUNTING, TREE_KIND_ALLOCATION, TREE_KIND_INODE,
    TREE_KIND_RETIREMENT,
};
use singlefs_core::recovery::{
    choose_system_configuration, every_root_ring_slot, read_root_ring_slot, recover, JournalPolicy,
    PoolReader, RecoveryFailure, RecoveryOutcome, RootRingSlotReading,
    SystemConfigurationValueOutsideWhatThisReaderAccepts,
};
use singlefs_core::root_record::{RootRecord, UnmountMarker};
use singlefs_core::root_ring::{slot_offset, RootRingSlot, RootRingSlotsPerRegion};
use singlefs_core::system_configuration::{
    SystemConfiguration, SystemImmutableSizes, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_new_inodes, warm_up, FirstFile, PoolWriter,
    TransactionOutput,
};
use singlefs_core::unit::{
    build_data_unit, build_index_node, build_packed_unit, parse_data_unit, parse_index_node,
    parse_packed_unit, seal_header_checksum, DataUnitIdentity, PackedIdentity, UnitError,
    WriteOrder, ENTRY_WIDTH_ZERO_WITH_ENTRIES, PACKED_TYPE_INODE, RECORD_WIDTH_ZERO_WITH_RECORDS,
    UNIT_CLASS_INDEX_NODE,
};
use singlefs_format::{
    ACCOUNTING_ENTRY_BYTES, ACCOUNTING_KEY_BYTES, ALLOCATION_RECORD_KEY_BYTES, DATA_UNIT_BYTES,
    JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES, NODE_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES,
    UNIT_AREA_START_SLOT,
};
use singlefs_harness::memory_pool::{SparseBlockDevice, SparseDevice};

const DEVICE_BYTES: u64 = 4 << 30;
const MEBIBYTE: u64 = 1 << 20;
const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"singlefs-rev-a3a";
const WRITE_TIME_SECONDS: u64 = 1_788_200_000;
/// 内存稀疏盘按 512 字节扇区存：物理块宽 512。
const SECTOR_PHYSICAL_BLOCK_SIZE: PhysicalBlockSizeInBytes = PhysicalBlockSizeInBytes(512);
/// 单元区起点之下、根环区域 0（1 MiB 起、8 槽 × 4096）与区域 1（4 MiB 起）之间没人用的一个 16 KiB 槽：
/// 把一个单元的一份拷贝放在这里，读者按位置条目读得到、校验和对得上，槽号却不在单元区里。
const SLOT_BELOW_THE_UNIT_AREA_NOBODY_USES: SlotNumber = SlotNumber(100);

type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

fn parameters() -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
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

fn node_bytes() -> usize {
    usize::try_from(NODE_BYTES).expect("16384")
}

fn data_unit_bytes() -> usize {
    usize::try_from(DATA_UNIT_BYTES).expect("32768")
}

fn two_sparse_devices() -> Devices {
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

/// 两块内存盘上只做过 mkfs 的池（树表 0 条、没取过号）。
fn formatted_pool() -> Devices {
    let mut devices = two_sparse_devices();
    make_filesystem(&parameters(), &mut devices).expect("默认几何上 mkfs");
    devices
}

/// 两块内存盘上 mkfs → 取号 1 → 暖机 → 第一个文件（txg 3）：记账树、中央映射树都是单个叶根，inode 树根下一片叶容器。
fn pool_after_the_first_file() -> (Devices, TransactionOutput) {
    let (devices, _allocator, first) = pool_and_its_allocator_after_the_first_file();
    (devices, first)
}

/// [`pool_after_the_first_file`] 连同接着发布要用的分配器。
fn pool_and_its_allocator_after_the_first_file() -> (Devices, PoolAllocator, TransactionOutput) {
    let parameters = parameters();
    let mut devices = two_sparse_devices();
    let genesis = make_filesystem(&parameters, &mut devices).expect("默认几何上 mkfs");
    let mut allocator: PoolAllocator =
        allocator_after_make_filesystem(&parameters, &devices, &genesis);
    let content: Vec<u8> = (0..3000u32)
        .map(|index| u8::try_from(index % 251).expect("小于 256"))
        .collect();
    let first = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let instance = acquire_instance(&mut writer).expect("取号 1");
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
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
        .expect("第一个文件")
    };
    (devices, allocator, first)
}

/// 两块盘此刻的整份镜像：拒之前、拒之后逐字节比。
fn images_of(devices: &Devices) -> Vec<(DeviceIdentity, SparseDevice)> {
    devices
        .iter()
        .map(|(identity, device)| (*identity, device.image.clone()))
        .collect()
}

fn device_mut(devices: &mut Devices, identity: DeviceIdentity) -> &mut SparseBlockDevice {
    &mut devices
        .iter_mut()
        .find(|(candidate, _)| *candidate == identity)
        .expect("池里有这块盘")
        .1
}

fn read_bytes(
    devices: &Devices,
    device: DeviceIdentity,
    offset: DeviceOffsetInBytes,
    length: usize,
) -> Vec<u8> {
    PoolReader::read(devices, device, offset, length).expect("读得到")
}

/// 按位置条目把一个单元的字节写到它的两份落点上（两块盘各一份），交回整单元 CRC-32C（位置条目里记的那个校验和）。
fn write_unit_at_both_locations(
    devices: &mut Devices,
    locations: &[LocationEntry; 2],
    unit: &[u8],
) -> u32 {
    for location in locations {
        device_mut(devices, location.device)
            .image
            .write(location.slot.to_device_offset(), unit);
    }
    crc32_castagnoli(unit)
}

/// 码 2 节点改过条目区或头之后重封：载荷 CRC（罩 [86 + 2k, 16384)，住 76 + 2k）与头校验和（罩 [0, 86 + 2k)）。
fn reseal_index_node(node: &mut [u8]) {
    let key_width = usize::from(node[51]);
    let header_end = 86 + 2 * key_width;
    let payload_checksum = crc32_castagnoli(&node[header_end..]);
    node[header_end - 10..header_end - 6].copy_from_slice(&payload_checksum.to_le_bytes());
    seal_header_checksum(node, header_end);
}

/// 码 3 单元改过之后重封：载荷 CRC（罩 [107, 32768)，住 89）与头校验和（罩 [0, 107)）。
fn reseal_packed_unit(unit: &mut [u8]) {
    let payload_checksum = crc32_castagnoli(&unit[107..]);
    unit[89..93].copy_from_slice(&payload_checksum.to_le_bytes());
    seal_header_checksum(unit, 107);
}

/// 码 1 单元改过之后重封：载荷 CRC（罩 [105, 32768)，住 101）与头校验和（罩 [0, 105)）。
fn reseal_data_unit(unit: &mut [u8]) {
    let payload_checksum = crc32_castagnoli(&unit[105..]);
    unit[101..105].copy_from_slice(&payload_checksum.to_le_bytes());
    seal_header_checksum(unit, 105);
}

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

/// 一条节点指针里两条位置条目的整单元校验和换成 `unit_checksum`（指针在字节里从 `pointer_offset` 起：头部 50，位置条目 14 × 2，
/// 每条的校验和住条目偏移 4 + 6）。
fn repoint_the_checksums_of_a_pointer(bytes: &mut [u8], pointer_offset: usize, unit_checksum: u32) {
    for location_index in 0..2 {
        let checksum_offset = pointer_offset + 50 + location_index * 14 + 10;
        bytes[checksum_offset..checksum_offset + 4].copy_from_slice(&unit_checksum.to_le_bytes());
    }
}

/// 系统配置里的根环几何（读根槽要它）。
fn system_configuration_of(devices: &Devices) -> SystemConfiguration {
    choose_system_configuration(devices).expect("系统配置择得到")
}

/// 根环里自证过的每一条根，连同它住的槽。
fn roots_in_the_ring(devices: &Devices) -> Vec<(RootRingSlot, RootRecord)> {
    let system_configuration = system_configuration_of(devices);
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
        .collect()
}

/// 根环里 (txg, 实例代号) 最大的那条根与它住的槽（择根择到的就是它）。
fn newest_root_in_the_ring(devices: &Devices) -> (RootRingSlot, RootRecord) {
    roots_in_the_ring(devices)
        .into_iter()
        .max_by_key(|(_, root)| (root.checkpoint_txg, root.instance))
        .expect("根环里有自证过的根")
}

/// 把一条根按它的几何写进根环的一个槽（整槽自证校验和由 `RootRecord::to_slot` 算）。
fn write_root_into_the_ring_slot(
    devices: &mut Devices,
    ring_slot: RootRingSlot,
    root: &RootRecord,
) {
    let parameters = parameters();
    let device = parameters.region_devices[usize::try_from(ring_slot.region).expect("区域号")];
    let offset = slot_offset(ring_slot, parameters.geometry.fixed_structure_slot_spacing);
    device_mut(devices, device).image.write(
        offset,
        &root.to_slot(usize::try_from(parameters.geometry.physical_block_size).expect("根槽宽")),
    );
}

/// 中央映射树根（单个叶根，这里改的几版都是）里码 2 节点 `node_pointer` 那一条映射条目的两条位置条目，整单元校验和换成 `node_checksum`。
/// 改完重封映射树根自己的两道校验和（下一次在同一个根上改另一条时，解它要先过这两道）。
fn repoint_the_mapping_entry_of_an_index_node(
    mapping_root: &mut [u8],
    node_pointer: NodePointer,
    node_checksum: u32,
) {
    let mapping_key = mapping_key_for_node(UNIT_CLASS_INDEX_NODE, node_pointer);
    let mapping_header = parse_index_node(mapping_root).expect("映射树根解得开");
    assert_eq!(
        mapping_header.level, 0,
        "第一个文件那一版与之后建空 inode 那一版的映射树都是单个叶根"
    );
    let mapping_entry_index = mapping_header
        .entries
        .iter()
        .position(|entry| parse_mapping_entry(entry).is_some_and(|(key, _)| key == mapping_key))
        .expect("映射里有这个节点");
    let mapping_entry_range = entry_range_in_node(mapping_root, mapping_entry_index);
    for location_index in 0..2 {
        // 映射条目：key 27 + 位置条目 14 × 2，每条的校验和住条目偏移 4 + 6。
        let checksum_offset = mapping_entry_range.start + 27 + location_index * 14 + 10;
        mapping_root[checksum_offset..checksum_offset + 4]
            .copy_from_slice(&node_checksum.to_le_bytes());
    }
    reseal_index_node(mapping_root);
}

/// 把最新那条根的树表里种类为 `kind` 的那棵树的根节点改一下（`change` 改字节，之后这里重封它），再把指着它的每一道校验和都重算：
/// 树表条目里那条根指针、中央映射里它那一条（单个叶根的映射树）、树表单元、映射树根、根记录。读者沿根记录读下来，走到被改的节点才判得到。
fn rewrite_the_root_node_of_a_tree_of_the_newest_root(
    devices: &mut Devices,
    kind: u16,
    change: impl FnOnce(&mut Vec<u8>),
) {
    let (ring_slot, mut root) = newest_root_in_the_ring(devices);
    let mut tree_table = read_bytes(
        devices,
        root.tree_table.locations[0].device,
        root.tree_table.locations[0].slot.to_device_offset(),
        node_bytes(),
    );
    let tree_table_header = parse_index_node(&tree_table).expect("树表单元解得开");
    let entry_index = tree_table_header
        .entries
        .iter()
        .position(|entry| TreeTableEntry::parse(entry).is_some_and(|parsed| parsed.kind == kind))
        .expect("树表里有这种树");
    let child = TreeTableEntry::parse(&tree_table_header.entries[entry_index])
        .expect("树表条目解得开")
        .root;
    let mut node = read_bytes(
        devices,
        child.locations[0].device,
        child.locations[0].slot.to_device_offset(),
        node_bytes(),
    );
    change(&mut node);
    reseal_index_node(&mut node);
    let node_checksum = write_unit_at_both_locations(devices, &child.locations, &node);

    let entry_range = entry_range_in_node(&tree_table, entry_index);
    // 树表条目里根指针从条目偏移 14 起（树 ID 8 + 条目长度 2 + 种类 2 + flags 2）。
    repoint_the_checksums_of_a_pointer(&mut tree_table[entry_range], 14, node_checksum);
    reseal_index_node(&mut tree_table);
    let tree_table_checksum =
        write_unit_at_both_locations(devices, &root.tree_table.locations, &tree_table);

    let mut mapping_root = read_bytes(
        devices,
        root.mapping_root.locations[0].device,
        root.mapping_root.locations[0].slot.to_device_offset(),
        node_bytes(),
    );
    repoint_the_mapping_entry_of_an_index_node(&mut mapping_root, child, node_checksum);
    let mapping_root_checksum =
        write_unit_at_both_locations(devices, &root.mapping_root.locations, &mapping_root);

    for location in &mut root.tree_table.locations {
        location.unit_checksum = tree_table_checksum;
    }
    for location in &mut root.mapping_root.locations {
        location.unit_checksum = mapping_root_checksum;
    }
    write_root_into_the_ring_slot(devices, ring_slot, &root);
}

/// 把最新那条根的树表里种类为 `kind` 的那条条目改一下（`change` 改条目的 200 字节），重封树表单元、重算根记录里树表指针的校验和。
fn rewrite_a_tree_table_entry_of_the_newest_root(
    devices: &mut Devices,
    kind: u16,
    change: impl FnOnce(&mut [u8]),
) {
    let (ring_slot, mut root) = newest_root_in_the_ring(devices);
    let mut tree_table = read_bytes(
        devices,
        root.tree_table.locations[0].device,
        root.tree_table.locations[0].slot.to_device_offset(),
        node_bytes(),
    );
    let tree_table_header = parse_index_node(&tree_table).expect("树表单元解得开");
    let entry_index = tree_table_header
        .entries
        .iter()
        .position(|entry| TreeTableEntry::parse(entry).is_some_and(|parsed| parsed.kind == kind))
        .expect("树表里有这种树");
    let entry_range = entry_range_in_node(&tree_table, entry_index);
    change(&mut tree_table[entry_range]);
    reseal_index_node(&mut tree_table);
    let tree_table_checksum =
        write_unit_at_both_locations(devices, &root.tree_table.locations, &tree_table);
    for location in &mut root.tree_table.locations {
        location.unit_checksum = tree_table_checksum;
    }
    write_root_into_the_ring_slot(devices, ring_slot, &root);
}

/// 可写挂载被拒、盘上逐字节不变：交回拒的那个错。
fn writable_mount_refused_before_any_write(devices: &mut Devices) -> MountError {
    let before = images_of(devices);
    let refusal = match mount_writable(&parameters(), devices) {
        Ok(mounted) => panic!(
            "坏镜像被可写挂载收下了：这一版是实例 {:?}、txg {:?}",
            mounted.output.effective_root.instance, mounted.output.effective_root.checkpoint_txg
        ),
        Err(refusal) => refusal,
    };
    assert_eq!(
        images_of(devices),
        before,
        "拒在任何写之前：两块盘逐字节不变（{refusal:?}）"
    );
    refusal
}

// ─── 代码审阅第 33 条（与第 24 条同一件）：从盘上重建上一版照冷走读判全 ───

/// 场景：第一个文件那一版的记账树（单个叶根，15 条）里第二条条目整条抄成第一条，两条的 key 相同；叶的两道校验和、树表条目里的根指针、
/// 映射条目、树表、映射树根与根记录的自证校验和都重算。
/// 预期：可写挂载在任何写之前报「叶里的条目不按 key 严格递增」（I-1.1）。改之前重建只按「拼得成一棵树」读记账树、不判 key 次序，
/// 重复 key 的叶原样交给写行那次发布，规划记账树时在 `transaction` 的断言上 panic。
#[test]
fn an_accounting_leaf_with_a_repeated_key_is_refused_by_the_writable_mount_before_any_write() {
    let (mut devices, _first) = pool_after_the_first_file();
    rewrite_the_root_node_of_a_tree_of_the_newest_root(
        &mut devices,
        TREE_KIND_ACCOUNTING,
        |node| {
            let first_entry = node[entry_range_in_node(node, 0)].to_vec();
            let second_entry_range = entry_range_in_node(node, 1);
            node[second_entry_range].copy_from_slice(&first_entry);
        },
    );
    let refusal = writable_mount_refused_before_any_write(&mut devices);
    assert!(
        matches!(
            refusal,
            MountError::Recovery(RecoveryFailure::InvariantViolated {
                invariant: "I-1.1",
                detail: "叶里的条目不按 key 严格递增",
            })
        ),
        "重复 key 的记账叶判 key 次序：{refusal:?}"
    );
}

/// 场景：最新那条根的树表里退役树那条条目的树 ID 改成根记录的树 ID 水位（第一个文件那一版是 65549），树表与根记录重封。
/// 退役树在发号次序里排最后、号最大（65548），改成 65549 之后树表条目仍按树 ID 严格升序、合发号次序，只违反水位那一条
/// （第 50 批之前排最后的是 logged_ops，改的是它）。
/// （树表 7 → 11 条之前排最后的是 deadlist（那时号 18，改成水位 19），D18（块里携带什么信息） 已定项 24 第 6 条之后 deadlist 后面还有四棵，改它就先违反 I-9.16。）
/// 预期：可写挂载在任何写之前报 I-7.8（根记录树 ID 水位不低于全池最大树 ID）「树 ID 不低于水位」。改之前重建不判它，写行那次发布照抄这张树表。
/// 合入后验证一（2026-09-27）把被改的条目从 livelist 换成 deadlist：livelist 改成 19 同时违反 I-9.16（树表条目按树 ID 严格升序且合发号次序），
/// 而 I-9.16 那一判排在水位之前（树表排序报告第八节第 3 条），报的是 I-9.16，这条用例就判不到 I-7.8 了。
#[test]
fn a_tree_table_entry_at_or_above_the_tree_identifier_watermark_is_refused_by_the_writable_mount() {
    let (mut devices, first) = pool_after_the_first_file();
    let watermark = first.root.tree_identifier_watermark;
    rewrite_a_tree_table_entry_of_the_newest_root(&mut devices, TREE_KIND_RETIREMENT, |entry| {
        entry[..8].copy_from_slice(&watermark.to_le_bytes());
    });
    let refusal = writable_mount_refused_before_any_write(&mut devices);
    assert!(
        matches!(
            refusal,
            MountError::Recovery(RecoveryFailure::InvariantViolated {
                invariant: "I-7.8",
                detail: "树 ID 不低于水位",
            })
        ),
        "树表条目的树 ID 判 I-7.8：{refusal:?}"
    );
}

/// 场景：inode 树根第一条内部条目里身份引用的容器出生代加一（条目 = 分隔 key 8 + 身份引用 26 + 子指针 86，容器出生代住条目偏移 26），
/// 与它指着的叶容器头里的四元组不再逐字相等；inode 树根与指着它的每一道校验和重算。
/// 预期：可写挂载在任何写之前报 I-9.2（条目身份与子头相符）「inode 叶头与条目身份引用不符」。改之前重建取容器头那一份身份、不与条目比。
#[test]
fn an_inode_internal_entry_whose_identity_differs_from_its_leaf_container_is_refused_by_the_writable_mount(
) {
    let (mut devices, _first) = pool_after_the_first_file();
    rewrite_the_root_node_of_a_tree_of_the_newest_root(&mut devices, TREE_KIND_INODE, |node| {
        let entry_range = entry_range_in_node(node, 0);
        let container_birth_offset = entry_range.start + 8 + 8 + 2 + 8;
        let container_birth = u64::from_le_bytes(
            node[container_birth_offset..container_birth_offset + 8]
                .try_into()
                .expect("8 字节"),
        );
        node[container_birth_offset..container_birth_offset + 8]
            .copy_from_slice(&(container_birth + 1).to_le_bytes());
    });
    let refusal = writable_mount_refused_before_any_write(&mut devices);
    assert!(
        matches!(
            refusal,
            MountError::Recovery(RecoveryFailure::InvariantViolated {
                invariant: "I-9.2",
                detail: "inode 叶头与条目身份引用不符",
            })
        ),
        "inode 内部条目的身份引用判 I-9.2：{refusal:?}"
    );
}

/// 场景：inode 树根节点头里的出生序号加一（住 72 + 2k，k = 8），与树表条目里指着它的那条指针不再相同；节点与指着它的每一道校验和重算。
/// 预期：可写挂载在任何写之前报 I-1.2「树根出生序号与指针不符」。改之前重建读 inode 树根只解节点、不核它的出生身份。
#[test]
fn an_inode_tree_root_whose_birth_sequence_differs_from_its_pointer_is_refused_by_the_writable_mount(
) {
    let (mut devices, _first) = pool_after_the_first_file();
    rewrite_the_root_node_of_a_tree_of_the_newest_root(&mut devices, TREE_KIND_INODE, |node| {
        let birth_sequence_offset = 72 + 2 * usize::from(node[51]);
        let birth_sequence = u32::from_le_bytes(
            node[birth_sequence_offset..birth_sequence_offset + 4]
                .try_into()
                .expect("4 字节"),
        );
        node[birth_sequence_offset..birth_sequence_offset + 4]
            .copy_from_slice(&(birth_sequence + 1).to_le_bytes());
    });
    let refusal = writable_mount_refused_before_any_write(&mut devices);
    assert!(
        matches!(
            refusal,
            MountError::Recovery(RecoveryFailure::InvariantViolated {
                invariant: "I-1.2",
                detail: "树根出生序号与指针不符",
            })
        ),
        "inode 树根的出生序号判 I-1.2：{refusal:?}"
    );
}

// ─── 代码审阅第 32 条：盘上读来的指针槽号进分配器之前判在单元区里 ───

/// 场景：只做过 mkfs 的池，三个根环区域里第 0 代根的实例表指针两条位置条目都改指槽 100（单元区起点之下、没人用的空当），
/// 实例表单元原样拷一份到两块盘的槽 100：读者按位置条目读得到、校验和对得上、身份 (0, 4, 0, 0) 照样对。
/// 预期：可写挂载在动分配器与任何写之前报「分配记录的槽号落在单元区起点之下」（与盘上分配记录同一道几何判）。
/// 改之前 mkfs 那一版的两个落点直接进 `mark_format_time_units`，在 `DeviceFreeMap::index` 的 expect 上 panic。
#[test]
fn a_format_time_instance_table_pointer_below_the_unit_area_is_refused_before_the_allocator_is_touched(
) {
    let mut devices = formatted_pool();
    let genesis_roots = roots_in_the_ring(&devices);
    assert_eq!(genesis_roots.len(), 3, "mkfs 把第 0 代根种进三个区域各一份");
    let instance_table_pointer = genesis_roots[0].1.instance_table;
    let instance_table = read_bytes(
        &devices,
        instance_table_pointer.locations[0].device,
        instance_table_pointer.locations[0].slot.to_device_offset(),
        data_unit_bytes(),
    );
    let mut moved_locations = instance_table_pointer.locations;
    for location in &mut moved_locations {
        location.slot = SLOT_BELOW_THE_UNIT_AREA_NOBODY_USES;
    }
    write_unit_at_both_locations(&mut devices, &moved_locations, &instance_table);
    for (ring_slot, mut root) in genesis_roots {
        root.instance_table.locations = moved_locations;
        write_root_into_the_ring_slot(&mut devices, ring_slot, &root);
    }
    let refusal = writable_mount_refused_before_any_write(&mut devices);
    assert!(
        matches!(
            refusal,
            MountError::Recovery(RecoveryFailure::AllocationRecordOutsideThePoolGeometry {
                what: "分配记录的槽号落在单元区起点之下",
            })
        ),
        "mkfs 那一版的落点先过几何判：{refusal:?}"
    );
}

/// 场景：第一个文件那一版之后重开一次（实例 2 写行 (1, 3, W)、暖机），再往根环一个空槽里放一条实例 1、txg 4 的根——照抄实例 1 暖机那条
/// 树表 0 条的根、只改 txg 与实例表指针：实例表指针两条位置条目都指槽 100（单元区起点之下）。按最新那条根的实例表，它是被抛弃的根。
/// 预期：这条根的账算「解不开」，它是树表 0 条的一版、从中央映射也现算不成，第二次重开在取号之前拒可写、报出这条根（C393（被抛弃根的账读不出时修复没有条款） 取 (b)-从映射现算，用户 2026-09-28 定），不 panic。
/// 改之前影子账只判了两条位置条目同槽，槽 100 进 `isolate_abandoned`，在 `DeviceFreeMap::index` 的 expect 上 panic。
#[test]
fn an_abandoned_root_whose_instance_table_pointer_sits_below_the_unit_area_refuses_the_writable_mount_instead_of_panicking(
) {
    let (mut devices, first) = pool_after_the_first_file();
    let first_remount =
        mount_writable(&parameters(), &mut devices).expect("第一次重开：实例 2 写行、暖机");
    assert_eq!(first_remount.output.instance, InstanceGeneration(2));
    let roots = roots_in_the_ring(&devices);
    let (_, warm_up_root_of_instance_one) = roots
        .iter()
        .find(|(_, root)| {
            root.instance == first.root.instance && root.checkpoint_txg == CheckpointTxg(1)
        })
        .copied()
        .expect("实例 1 暖机那条 txg 1 的根还在环里");
    let occupied: Vec<RootRingSlot> = roots.iter().map(|(ring_slot, _)| *ring_slot).collect();
    let system_configuration = system_configuration_of(&devices);
    let empty_ring_slot = every_root_ring_slot(&system_configuration.immutable.sizes)
        .into_iter()
        .rev()
        .find(|ring_slot| !occupied.contains(ring_slot))
        .expect("根环里还有空槽");
    let mut abandoned = warm_up_root_of_instance_one;
    abandoned.checkpoint_txg = CheckpointTxg(first.root.checkpoint_txg.0 + 1);
    for location in &mut abandoned.instance_table.locations {
        location.slot = SLOT_BELOW_THE_UNIT_AREA_NOBODY_USES;
    }
    write_root_into_the_ring_slot(&mut devices, empty_ring_slot, &abandoned);
    // 指针槽号不在单元区里的那条被抛弃根算「账读不出」；它是暖机那一版（树表 0 条、映射根是空指针），从中央映射也现算不成，
    // 重读一次仍不成就在取号之前拒可写（C393（被抛弃根的账读不出时修复没有条款） 取 (b)-从映射现算那一条的最后一支，用户 2026-09-28 定）——不 panic。
    let refusal = mount_writable(&parameters(), &mut devices)
        .expect_err("被抛弃根的指针不在单元区里、映射也现算不成：拒可写，不 panic");
    let MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable) = &refusal else {
        panic!("拒因是「重读一次仍读不出」：{refusal:?}");
    };
    assert!(
        matches!(
            still_unreadable.as_ref(),
            StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot { abandoned_root }
                if abandoned_root.instance == first.root.instance
                    && abandoned_root.checkpoint_txg == CheckpointTxg(first.root.checkpoint_txg.0 + 1)
        ),
        "拒因点名那条被抛弃根：{still_unreadable:?}"
    );
}

// ─── 代码审阅第 35 条：盘比单元区起点还短 ───

/// 场景：只做过 mkfs 的池，盘 1 换成一块 700 MiB 的盘（前 700 MiB 与原来逐字节相同）：系统配置、根环都读得到，单元读盘 0 那一份。
/// 预期：可写挂载在读根环之前报盘 1 末尾在单元区起点（784 MiB）之前。改之前走到重建分配器，`DeviceFreeMap::new` 里单元区槽数的减法下溢 panic。
#[test]
fn a_device_ending_before_the_unit_area_start_is_refused_by_the_writable_mount_before_any_write() {
    let mut devices = formatted_pool();
    let short_device_bytes = 700 * MEBIBYTE;
    let mut short_device = SparseBlockDevice::new(short_device_bytes, SECTOR_PHYSICAL_BLOCK_SIZE);
    short_device.image = device_mut(&mut devices, DeviceIdentity(1)).image.clone();
    *device_mut(&mut devices, DeviceIdentity(1)) = short_device;
    assert!(
        short_device_bytes < UNIT_AREA_START_SLOT * 16384,
        "700 MiB 在单元区起点 784 MiB 之前"
    );
    let refusal = writable_mount_refused_before_any_write(&mut devices);
    assert!(
        matches!(
            refusal,
            MountError::Recovery(RecoveryFailure::DeviceEndsBeforeTheUnitAreaStart {
                device: DeviceIdentity(1),
                device_bytes,
            }) if device_bytes == short_device_bytes
        ),
        "盘容量下界：{refusal:?}"
    );
}

// ─── 代码审阅第 29、38 条：系统配置里的池级不可变字段读出来先判 ───

/// 系统配置槽里几样字段的偏移（`SystemConfiguration::to_slot` 的写法）：格式版本紧跟 magic；加密类型在整槽校验和 32、MAC 16、nonce 水位 12、
/// KDF 标识 4 之后，紧跟着的是 MAC 长度声明 16；`physical_block_size`、journal 环长、固定结构槽距三个用例改完之后按 `parse_slot` 读回核。
const FORMAT_VERSION_OFFSET: usize = 4;
const ENCRYPTION_TYPE_OFFSET: usize = SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32 + 16 + 12 + 4;
const PHYSICAL_BLOCK_SIZE_OFFSET: usize = 317;
const JOURNAL_RING_BYTES_OFFSET: usize = 333;
const FIXED_STRUCTURE_SLOT_SPACING_OFFSET: usize = 429;

/// 两块盘各两个系统配置槽（偏移 0 与槽距 4096）逐个改一下、整槽校验和重封：池级字段两盘四槽同值，改就四槽一起改。
fn rewrite_every_system_configuration_slot(devices: &mut Devices, change: impl Fn(&mut [u8])) {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        for offset in [0, spacing] {
            let mut slot = read_bytes(devices, device, DeviceOffsetInBytes(offset), slot_bytes);
            assert_eq!(
                (
                    &slot[FORMAT_VERSION_OFFSET..FORMAT_VERSION_OFFSET + 2],
                    slot[ENCRYPTION_TYPE_OFFSET],
                    slot[ENCRYPTION_TYPE_OFFSET + 1]
                ),
                (&[1u8, 0][..], 0, 16),
                "偏移对得上字段表：格式版本 1、加密类型 0、其后 MAC 长度声明 16"
            );
            change(&mut slot);
            let digest = wide_checksum_with_field_zeroed(
                &slot,
                slot_bytes,
                SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
            );
            slot[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32]
                .copy_from_slice(&digest);
            device_mut(devices, device)
                .image
                .write(DeviceOffsetInBytes(offset), &slot);
        }
    }
}

/// 同一份镜像先给只读恢复（不带调用方参数，只看盘上择到的那份系统配置），再给可写挂载：两处都在读系统配置那一步拒、报同一个成员，
/// 可写挂载在任何写之前拒。交回被拒的那个值。
fn system_configuration_value_refused(
    devices: &mut Devices,
) -> SystemConfigurationValueOutsideWhatThisReaderAccepts {
    let report = recover(&*devices, JournalPolicy::Consult);
    let RecoveryOutcome::Failed {
        failure:
            RecoveryFailure::SystemConfigurationValueRefused {
                device: DeviceIdentity(0),
                value,
            },
        ..
    } = report.outcome
    else {
        panic!(
            "只读恢复要在读系统配置那一步拒（盘 0 先读到的那一槽）：{:?}",
            report.outcome
        );
    };
    let refusal = writable_mount_refused_before_any_write(devices);
    assert!(
        matches!(
            refusal,
            MountError::Recovery(RecoveryFailure::SystemConfigurationValueRefused {
                device: DeviceIdentity(0),
                value: refused_by_the_mount,
            }) if refused_by_the_mount == value
        ),
        "可写挂载与只读恢复报同一个成员：{refusal:?}"
    );
    value
}

fn write_u32_at(slot: &mut [u8], offset: usize, value: u32) {
    slot[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

/// 场景：四个系统配置槽的格式版本写成 2，整槽校验和重封。预期：可写挂载在读系统配置那一步整池拒（格式版本不认识），盘上逐字节不变。
/// 改之前读者不看这 2 字节：只读恢复按第 1 版的字段表读下去，可写挂载照常挂上。
#[test]
fn a_system_configuration_of_an_unrecognized_format_version_is_refused() {
    let mut devices = formatted_pool();
    rewrite_every_system_configuration_slot(&mut devices, |slot| {
        slot[FORMAT_VERSION_OFFSET..FORMAT_VERSION_OFFSET + 2].copy_from_slice(&2u16.to_le_bytes());
    });
    assert_eq!(
        system_configuration_value_refused(&mut devices),
        SystemConfigurationValueOutsideWhatThisReaderAccepts::FormatVersionNotRecognized {
            format_version: 2
        }
    );
}

/// 场景：四个系统配置槽的加密类型写成 1（登记表里的 ChaCha20-Poly1305），整槽校验和重封。
/// 预期：整池拒（加密不进第一个可运行版本，D9（加密） 已定项 10），盘上逐字节不变。改之前读者不看这 1 字节：只读恢复按明文读下去，可写挂载照常挂上。
#[test]
fn a_system_configuration_whose_encryption_type_is_not_off_is_refused() {
    let mut devices = formatted_pool();
    rewrite_every_system_configuration_slot(&mut devices, |slot| {
        slot[ENCRYPTION_TYPE_OFFSET] = 1;
    });
    assert_eq!(
        system_configuration_value_refused(&mut devices),
        SystemConfigurationValueOutsideWhatThisReaderAccepts::EncryptionTypeNotOff {
            encryption_type: 1
        }
    );
}

/// 场景：四个系统配置槽的 `physical_block_size` 写成 256（装不下 457 字节的根记录），整槽校验和重封。
/// 预期：整池拒（根槽宽下界），盘上逐字节不变。改之前：只读恢复按 256 字节读根槽、一条根都自证不过，报「根环里一条根都没有」；
/// 可写挂载因为调用方给的 512 与盘上不一致报 `CallerParametersDisagreeWithTheSelectedSystemConfiguration`——只读那条路没有调用方参数挡着。
#[test]
fn a_physical_block_size_narrower_than_the_root_record_is_refused() {
    let mut devices = formatted_pool();
    rewrite_every_system_configuration_slot(&mut devices, |slot| {
        write_u32_at(slot, PHYSICAL_BLOCK_SIZE_OFFSET, 256)
    });
    assert_eq!(
        system_configuration_value_refused(&mut devices),
        SystemConfigurationValueOutsideWhatThisReaderAccepts::PhysicalBlockSizeOutsideTheRootSlotBounds { physical_block_size: 256 }
    );
}

/// 场景：四个系统配置槽的 `physical_block_size` 写成 8192（大过固定结构槽距 4096，一个根槽盖到下一个槽上），整槽校验和重封。
/// 预期：整池拒（根槽宽上界是槽距），盘上逐字节不变。改之前：只读恢复按 8192 字节开缓冲读根槽（缓冲随这 4 字节开）、一条根都自证不过；可写挂载报调用方参数不一致。
#[test]
fn a_physical_block_size_wider_than_the_fixed_structure_slot_spacing_is_refused() {
    let mut devices = formatted_pool();
    rewrite_every_system_configuration_slot(&mut devices, |slot| {
        write_u32_at(slot, PHYSICAL_BLOCK_SIZE_OFFSET, 8192)
    });
    let refused_slot = read_bytes(&devices, DeviceIdentity(0), DeviceOffsetInBytes(0), 4096);
    assert_eq!(
        SystemConfiguration::parse_slot(&refused_slot)
            .expect("整槽校验和重封过")
            .immutable
            .sizes
            .physical_block_size,
        8192,
        "偏移 317 就是 physical_block_size"
    );
    assert_eq!(
        system_configuration_value_refused(&mut devices),
        SystemConfigurationValueOutsideWhatThisReaderAccepts::PhysicalBlockSizeOutsideTheRootSlotBounds { physical_block_size: 8192 }
    );
}

/// 场景：四个系统配置槽的固定结构槽距写成 2 MiB（槽 1 越过根环基址 1 MiB），整槽校验和重封。
/// 预期：整池拒（槽距在格式允许的区间之外），盘上逐字节不变。改之前：只读恢复按 2 MiB 的槽距找槽 1 与第 1 号起的根槽（落到根环区域里别的地方）；可写挂载报调用方参数不一致。
#[test]
fn a_fixed_structure_slot_spacing_outside_the_format_range_is_refused() {
    let mut devices = formatted_pool();
    let spacing = u32::try_from(2 * MEBIBYTE).expect("2 MiB 装得进 u32");
    rewrite_every_system_configuration_slot(&mut devices, |slot| {
        write_u32_at(slot, FIXED_STRUCTURE_SLOT_SPACING_OFFSET, spacing)
    });
    let refused_slot = read_bytes(&devices, DeviceIdentity(0), DeviceOffsetInBytes(0), 4096);
    assert_eq!(
        SystemConfiguration::parse_slot(&refused_slot)
            .expect("整槽校验和重封过")
            .immutable
            .sizes
            .fixed_structure_slot_spacing,
        spacing,
        "偏移 429 就是固定结构槽距"
    );
    assert_eq!(
        system_configuration_value_refused(&mut devices),
        SystemConfigurationValueOutsideWhatThisReaderAccepts::FixedStructureSlotSpacingOutsideTheFormatRange {
            fixed_structure_slot_spacing: spacing,
        }
    );
}

/// 场景：四个系统配置槽的 journal 环长写成 1000 字节（一条 4096 的记录都装不下），整槽校验和重封。
/// 预期：整池拒（环装不下 F 条记录），盘上逐字节不变。改之前：只读恢复扫 0 个环槽、照常走到第 0 代根；可写挂载报调用方参数不一致。
/// 环槽数 0 时 `journal::record_offset` 取模的那句 expect，读者一侧靠的就是这一道。
#[test]
fn a_journal_ring_too_short_for_the_safety_factor_is_refused_before_any_write() {
    let mut devices = formatted_pool();
    rewrite_every_system_configuration_slot(&mut devices, |slot| {
        slot[JOURNAL_RING_BYTES_OFFSET..JOURNAL_RING_BYTES_OFFSET + 8]
            .copy_from_slice(&1000u64.to_le_bytes());
    });
    let refused_slot = read_bytes(&devices, DeviceIdentity(0), DeviceOffsetInBytes(0), 4096);
    assert_eq!(
        SystemConfiguration::parse_slot(&refused_slot)
            .expect("整槽校验和重封过")
            .immutable
            .sizes
            .journal_ring_bytes,
        1000,
        "偏移 333 就是 journal 环长"
    );
    assert_eq!(
        system_configuration_value_refused(&mut devices),
        SystemConfigurationValueOutsideWhatThisReaderAccepts::JournalRingBytesOutsideTheSupportedRange { journal_ring_bytes: 1000 }
    );
}

/// 场景：四个系统配置槽的 journal 环长写成 2 GiB（末端越过编译期的单元区起点 784 MiB，C475），整槽校验和重封。
/// 预期：整池拒，盘上逐字节不变。改之前：只读恢复把单元区当环扫（按环长一次列出 2 GiB ÷ 4096 个偏移逐个读）；可写挂载报调用方参数不一致。
#[test]
fn a_journal_ring_reaching_past_the_unit_area_start_is_refused() {
    let mut devices = formatted_pool();
    let ring_bytes: u64 = 2 << 30;
    rewrite_every_system_configuration_slot(&mut devices, |slot| {
        slot[JOURNAL_RING_BYTES_OFFSET..JOURNAL_RING_BYTES_OFFSET + 8]
            .copy_from_slice(&ring_bytes.to_le_bytes());
    });
    assert_eq!(
        system_configuration_value_refused(&mut devices),
        SystemConfigurationValueOutsideWhatThisReaderAccepts::JournalRingBytesOutsideTheSupportedRange {
            journal_ring_bytes: ring_bytes,
        }
    );
}

// ─── 代码审阅第 29 条：单元头那 29 字节预留位、指针头部的加密与压缩那几段 ───

/// 29 字节预留位（nonce 12 + MAC 16 + 算法类型 1）里挑的那一字节：MAC 的第一个字节（预留位起点 + 12）。
const MAC_BYTE_IN_THE_RESERVED_BYTES: usize = 12;

fn sample_index_node() -> Vec<u8> {
    build_index_node(
        TreeIdentifier(singlefs_format::TREE_IDENTIFIER_ACCOUNTING),
        0,
        22,
        &[1u8; 22],
        &[3u8; 22],
        CheckpointTxg(3),
        &FILESYSTEM_IDENTIFIER,
        InstanceGeneration(1),
        BirthSequence(0),
        34,
        &[vec![1u8; 34], vec![2u8; 34], vec![3u8; 34]],
    )
}

/// 场景：一个码 2 节点（key 宽 22）预留位里 MAC 的第一个字节写成 1，载荷 CRC 与头校验和重算（预留位在载荷 CRC 覆盖内、不在头校验和覆盖内）。
/// 预期：`parse_index_node` 报预留位非 0（I-2.4「恒 0、读者遇到非 0 一律判该结构损坏」）。改之前读者跳过这 29 字节、照常解开。
#[test]
fn an_index_node_with_a_non_zero_encryption_reserved_byte_is_refused() {
    let mut node = sample_index_node();
    assert!(parse_index_node(&node).is_ok(), "改之前这个节点解得开");
    node[86 + 2 * 22 + MAC_BYTE_IN_THE_RESERVED_BYTES] = 1;
    reseal_index_node(&mut node);
    assert_eq!(
        parse_index_node(&node),
        Err(UnitError::EncryptionReservedBytesNotZero)
    );
}

/// 场景：一个码 3 inode 叶容器（记录宽 140、一条记录）预留位里 MAC 的第一个字节写成 1，两道校验和重算。
/// 预期：`parse_packed_unit` 报预留位非 0。改之前读者跳过这 29 字节。
#[test]
fn a_packed_unit_with_a_non_zero_encryption_reserved_byte_is_refused() {
    let mut unit = build_packed_unit(
        PackedIdentity {
            birth_tree: TreeIdentifier(singlefs_format::TREE_IDENTIFIER_INODE),
            record_type: PACKED_TYPE_INODE,
            container: 1,
            container_birth: CheckpointTxg(3),
        },
        140,
        &[vec![7u8; 140]],
        CheckpointTxg(3),
        &FILESYSTEM_IDENTIFIER,
        WriteOrder {
            instance: InstanceGeneration(1),
            transaction: 1,
        },
        BirthSequence(0),
    );
    assert!(parse_packed_unit(&unit).is_ok(), "改之前这个单元解得开");
    unit[107 + MAC_BYTE_IN_THE_RESERVED_BYTES] = 1;
    reseal_packed_unit(&mut unit);
    assert_eq!(
        parse_packed_unit(&unit),
        Err(UnitError::EncryptionReservedBytesNotZero)
    );
}

/// 场景：一个码 1 数据单元（3000 字节载荷）预留位里 MAC 的第一个字节写成 1，两道校验和重算。
/// 预期：`parse_data_unit` 报预留位非 0。改之前读者跳过这 29 字节。
#[test]
fn a_data_unit_with_a_non_zero_encryption_reserved_byte_is_refused() {
    let mut unit = build_data_unit(
        DataUnitIdentity {
            tree: TreeIdentifier(singlefs_format::TREE_IDENTIFIER_EXTENT),
            object: 1,
            object_birth: CheckpointTxg(3),
            anchor_offset: 0,
        },
        CheckpointTxg(3),
        &FILESYSTEM_IDENTIFIER,
        WriteOrder {
            instance: InstanceGeneration(1),
            transaction: 1,
        },
        &[5u8; 3000],
    );
    assert!(parse_data_unit(&unit).is_ok(), "改之前这个单元解得开");
    unit[105 + MAC_BYTE_IN_THE_RESERVED_BYTES] = 1;
    reseal_data_unit(&mut unit);
    assert_eq!(
        parse_data_unit(&unit),
        Err(UnitError::EncryptionReservedBytesNotZero)
    );
}

/// 场景：第一个文件那一版的记账树根预留位里 MAC 的第一个字节写成 1，节点与指着它的每一道校验和重算。
/// 预期：可写挂载在任何写之前报这个节点解不开（`UnitMalformed`「树根节点」）。改之前读者跳过这 29 字节、照常挂上。
#[test]
fn an_accounting_root_with_a_non_zero_encryption_reserved_byte_is_refused_by_the_writable_mount() {
    let (mut devices, _first) = pool_after_the_first_file();
    rewrite_the_root_node_of_a_tree_of_the_newest_root(
        &mut devices,
        TREE_KIND_ACCOUNTING,
        |node| {
            let reserved_start = 86 + 2 * usize::from(node[51]);
            node[reserved_start + MAC_BYTE_IN_THE_RESERVED_BYTES] = 1;
        },
    );
    let refusal = writable_mount_refused_before_any_write(&mut devices);
    assert!(
        matches!(
            refusal,
            MountError::Recovery(RecoveryFailure::UnitMalformed {
                what: "树根节点"
            })
        ),
        "预留位非 0 的记账树根：{refusal:?}"
    );
}

fn sample_node_pointer() -> NodePointer {
    NodePointer {
        head: PointerHead {
            birth_tree: TreeIdentifier(singlefs_format::TREE_IDENTIFIER_ACCOUNTING),
            birth_txg: CheckpointTxg(3),
        },
        locations: [
            LocationEntry {
                device: DeviceIdentity(0),
                slot: SlotNumber(UNIT_AREA_START_SLOT + 8),
                unit_checksum: 0x1234,
            },
            LocationEntry {
                device: DeviceIdentity(1),
                slot: SlotNumber(UNIT_AREA_START_SLOT + 8),
                unit_checksum: 0x1234,
            },
        ],
        instance: InstanceGeneration(1),
        birth_sequence: BirthSequence(2),
    }
}

fn node_pointer_bytes(pointer: &NodePointer) -> Vec<u8> {
    let mut writer = ByteWriter::new(86);
    pointer.write_to(&mut writer);
    writer.into_bytes()
}

/// 场景：一条节点指针头部（MAC 16、nonce 12、算法类型 1、压缩算法码 1、压后长度 2 依次在偏移 0、16、28、29、30）逐段改一个非 0 值。
/// 预期：判的读法逐段报它是哪一段（D19（块指针的结构与宽度预算） 已定项 3 / 已定项 7，D9（加密） 已定项 10 / 已定项 14）；
/// 五段都是 0 时照常读出同一条指针。改之前读者跳过这 36 字节，拿这条指针照常解引用。
#[test]
fn a_node_pointer_head_carrying_encryption_or_compression_is_refused_field_by_field() {
    let pointer = sample_node_pointer();
    let clean = node_pointer_bytes(&pointer);
    assert_eq!(
        NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
            &mut ByteReader::at(&clean, 0)
        ),
        Ok(pointer),
        "第一版的取值照常读出"
    );
    let cases: [(usize, u8, PointerHeadFieldOutsideTheFirstVersion); 5] = [
        (
            0,
            1,
            PointerHeadFieldOutsideTheFirstVersion::MacOrNonceNotZero,
        ),
        (
            16,
            1,
            PointerHeadFieldOutsideTheFirstVersion::MacOrNonceNotZero,
        ),
        (
            28,
            2,
            PointerHeadFieldOutsideTheFirstVersion::AlgorithmTypeNotUnencrypted {
                algorithm_type: 2,
            },
        ),
        (
            29,
            1,
            PointerHeadFieldOutsideTheFirstVersion::CompressionAlgorithmCodeNotRecognized {
                compression_algorithm_code: 1,
            },
        ),
        (
            30,
            5,
            PointerHeadFieldOutsideTheFirstVersion::CompressedLengthNotZeroWithoutCompression {
                compressed_length: 5,
            },
        ),
    ];
    for (offset, value, expected) in cases {
        let mut damaged = clean.clone();
        damaged[offset] = value;
        assert_eq!(
            NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
                &mut ByteReader::at(&damaged, 0)
            ),
            Err(expected),
            "指针头部偏移 {offset} 写成 {value}"
        );
    }
}

/// 场景：一条数据指针（88 字节）头部的算法类型写成 1。预期：判的读法报算法类型不是「未加密」。
#[test]
fn a_data_pointer_head_carrying_an_encryption_algorithm_is_refused() {
    let node_pointer = sample_node_pointer();
    let data_pointer = DataPointer {
        head: node_pointer.head,
        locations: node_pointer.locations,
        write_order: WriteOrder {
            instance: InstanceGeneration(1),
            transaction: 1,
        },
    };
    let mut writer = ByteWriter::new(88);
    data_pointer.write_to(&mut writer);
    let mut bytes = writer.into_bytes();
    assert_eq!(
        DataPointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
            &mut ByteReader::at(&bytes, 0)
        ),
        Ok(data_pointer)
    );
    bytes[28] = 1;
    assert_eq!(
        DataPointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
            &mut ByteReader::at(&bytes, 0)
        ),
        Err(
            PointerHeadFieldOutsideTheFirstVersion::AlgorithmTypeNotUnencrypted {
                algorithm_type: 1
            }
        )
    );
}

/// 场景：多层记账树的一条内部条目（key 22 + 子指针 86）里子指针头部的 nonce 第一个字节写成 1。
/// 预期：`parse_internal_entry` 报子指针头部不是第一版的取值（读多层码 2 树时报 `UnitMalformed`）。改之前照常解出子指针。
#[test]
fn a_code_two_internal_entry_whose_child_pointer_head_is_not_the_first_version_is_refused() {
    let key = [9u8; 22];
    let mut entry = build_internal_entry(&key, sample_node_pointer());
    assert_eq!(
        parse_internal_entry(&entry, 22),
        Ok((key.to_vec(), sample_node_pointer()))
    );
    entry[22 + 16] = 1;
    assert_eq!(
        parse_internal_entry(&entry, 22),
        Err(
            InternalEntryRefusal::ChildPointerHeadOutsideTheFirstVersion(
                PointerHeadFieldOutsideTheFirstVersion::MacOrNonceNotZero
            )
        )
    );
}

/// 场景：一条 journal 记录新根段里树表指针头部的压缩算法码写成 1（新根段从 99 起，指针头部偏移 29），整条 `header_csum` 重算。
/// 预期：`JournalRecord::parse` 当这条记录损坏（交回 `None`，与校验和不过同一个结局）。改之前照常解出、重放时拿这条指针当新根的树表。
#[test]
fn a_journal_record_whose_new_tree_table_pointer_carries_compression_is_not_parsed() {
    let record = JournalRecord {
        instance: InstanceGeneration(1),
        counter: 3,
        checkpoint_txg: CheckpointTxg(3),
        transaction: 1,
        is_commit: true,
        ordinal_within_publish: JournalRecordOrdinalWithinPublish::FIRST,
        place_in_publish: JournalRecordPlaceInPublish::LastRecordOfThePublish,
        back_chain: 0,
        filesystem_identifier: 7,
        new_tree_table: sample_node_pointer(),
        new_mapping_root: sample_node_pointer(),
        new_tree_identifier_watermark:
            singlefs_format::TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
        new_rollback_floor: CheckpointTxg(0),
        new_instance_table: NodePointer::empty_root(),
        new_allocation_record_tree_root: NodePointer::empty_root(),
        new_unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
        named: Vec::new(),
    };
    let mut bytes = record.to_bytes();
    assert_eq!(
        JournalRecord::parse(&bytes, 7),
        Some(record),
        "改之前解得开"
    );
    bytes[99 + 29] = 1;
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    let digest =
        wide_checksum_with_field_zeroed(&bytes, record_bytes, JOURNAL_HEADER_CHECKSUM_OFFSET);
    bytes[JOURNAL_HEADER_CHECKSUM_OFFSET..JOURNAL_HEADER_CHECKSUM_OFFSET + 32]
        .copy_from_slice(&digest);
    assert_eq!(JournalRecord::parse(&bytes, 7), None);
}

// ─── 代码审阅第 38 条：条目宽 0 而条目数非 0 ───

/// 场景：一个 key 宽 0、没有条目的码 2 节点，头里的条目数改成 3、条目宽改成 0（声明长度 0 = 3 × 0 照样对得上），头校验和重算。
/// 预期：`parse_index_node` 报结构错「条目宽为 0 而条目数不为 0」。改之前切出 0 条条目、头里却说 3 条，照常交出去。
#[test]
fn an_index_node_declaring_entries_of_width_zero_is_refused() {
    let mut node = build_index_node(
        TreeIdentifier(singlefs_format::TREE_IDENTIFIER_ACCOUNTING),
        0,
        0,
        &[],
        &[],
        CheckpointTxg(3),
        &FILESYSTEM_IDENTIFIER,
        InstanceGeneration(1),
        BirthSequence(0),
        1,
        &[],
    );
    // key 宽 0 时条目数住 82、条目宽住 84。
    node[82..84].copy_from_slice(&3u16.to_le_bytes());
    node[84..86].copy_from_slice(&0u16.to_le_bytes());
    reseal_index_node(&mut node);
    assert_eq!(
        parse_index_node(&node),
        Err(UnitError::Structure(ENTRY_WIDTH_ZERO_WITH_ENTRIES))
    );
}

/// 场景：一个没有记录的码 3 单元，头里的记录数改成 3、记录宽改成 0（住 69、71），头校验和重算。
/// 预期：`parse_packed_unit` 报结构错「记录宽为 0 而记录数不为 0」。改之前切出 0 条记录照常交出去。
#[test]
fn a_packed_unit_declaring_records_of_width_zero_is_refused() {
    let mut unit = build_packed_unit(
        PackedIdentity {
            birth_tree: TreeIdentifier(singlefs_format::TREE_IDENTIFIER_INODE),
            record_type: PACKED_TYPE_INODE,
            container: 1,
            container_birth: CheckpointTxg(3),
        },
        140,
        &[],
        CheckpointTxg(3),
        &FILESYSTEM_IDENTIFIER,
        WriteOrder {
            instance: InstanceGeneration(1),
            transaction: 1,
        },
        BirthSequence(0),
    );
    unit[69..71].copy_from_slice(&3u16.to_le_bytes());
    unit[71..73].copy_from_slice(&0u16.to_le_bytes());
    reseal_packed_unit(&mut unit);
    assert_eq!(
        parse_packed_unit(&unit),
        Err(UnitError::Structure(RECORD_WIDTH_ZERO_WITH_RECORDS))
    );
}

// ─── C476 普查「缺口」那张表 `pointer.rs` 那一行：I-2.5 升序断言，照抄写回盘上读来的指针时走得到 ───

/// 根记录里实例表指针在槽里的偏移：magic 4 + fsid 16 + flags 4 + 实例代号 4 + txg 8 + 树表指针 86 + 树 ID 水位 8 + F 8 + 自证校验和 32。
const INSTANCE_TABLE_POINTER_OFFSET_IN_THE_ROOT_SLOT: usize = 170;
/// 根记录自证校验和的偏移（`root_record::ROOT_CHECKSUM_OFFSET`，D22（单元原子性怎么合成） 已定项 7 的字段表）。
const ROOT_CHECKSUM_OFFSET_IN_THE_ROOT_SLOT: usize = 138;

/// 一条节点指针里两条位置条目（头部 50 之后，各 14 字节）对调。
fn swap_the_two_location_entries_of_a_pointer(bytes: &mut [u8], pointer_offset: usize) {
    let first_start = pointer_offset + 50;
    let first = bytes[first_start..first_start + 14].to_vec();
    let second = bytes[first_start + 14..first_start + 28].to_vec();
    bytes[first_start..first_start + 14].copy_from_slice(&second);
    bytes[first_start + 14..first_start + 28].copy_from_slice(&first);
}

/// 场景：第一个文件那一版的树表里 extent 树那条条目的根指针，两条位置条目对调成（盘 1, 盘 0）——读者按位置条目逐条试，照样读得到；
/// 树表与根记录重封。
/// 预期：可写挂载在任何写之前报 I-2.5（位置条目按设备身份升序）。改之前重建照收，写行那次发布照抄这条指针写新树表，
/// 在 `NodePointer::write_to` 的升序断言上 panic（实审 A3a 实测）。
#[test]
fn a_tree_table_root_pointer_with_descending_location_entries_is_refused_by_the_writable_mount() {
    let (mut devices, _first) = pool_after_the_first_file();
    rewrite_a_tree_table_entry_of_the_newest_root(
        &mut devices,
        singlefs_core::records::TREE_KIND_EXTENT,
        |entry| swap_the_two_location_entries_of_a_pointer(entry, 14),
    );
    let refusal = writable_mount_refused_before_any_write(&mut devices);
    assert!(
        matches!(
            refusal,
            MountError::Recovery(RecoveryFailure::InvariantViolated {
                invariant: "I-2.5",
                detail: "位置条目不按设备身份严格升序",
            })
        ),
        "逆序的树表根指针：{refusal:?}"
    );
}

/// 场景：只做过 mkfs 的池，三个区域里第 0 代根的实例表指针两条位置条目对调、整槽自证校验和重算。
/// 预期：可写挂载在任何写之前报 I-2.5。改之前第一次可写挂载的零单元发布照抄这条指针写新根，在 `NodePointer::write_to` 的升序断言上 panic。
#[test]
fn a_genesis_root_whose_instance_table_pointer_has_descending_location_entries_is_refused_by_the_writable_mount(
) {
    let mut devices = formatted_pool();
    let parameters = parameters();
    let root_slot_bytes = usize::try_from(parameters.geometry.physical_block_size).expect("根槽宽");
    for (ring_slot, root) in roots_in_the_ring(&devices) {
        let mut slot = root.to_slot(root_slot_bytes);
        swap_the_two_location_entries_of_a_pointer(
            &mut slot,
            INSTANCE_TABLE_POINTER_OFFSET_IN_THE_ROOT_SLOT,
        );
        let digest = wide_checksum_with_field_zeroed(
            &slot,
            root_slot_bytes,
            ROOT_CHECKSUM_OFFSET_IN_THE_ROOT_SLOT,
        );
        slot[ROOT_CHECKSUM_OFFSET_IN_THE_ROOT_SLOT..ROOT_CHECKSUM_OFFSET_IN_THE_ROOT_SLOT + 32]
            .copy_from_slice(&digest);
        let device = parameters.region_devices[usize::try_from(ring_slot.region).expect("区域号")];
        let offset = slot_offset(ring_slot, parameters.geometry.fixed_structure_slot_spacing);
        device_mut(&mut devices, device).image.write(offset, &slot);
    }
    let swapped_roots = roots_in_the_ring(&devices);
    assert_eq!(swapped_roots.len(), 3, "对调之后三条根照样自证得过");
    assert_eq!(
        swapped_roots[0].1.instance_table.locations[0].device,
        DeviceIdentity(1),
        "实例表指针第一条位置条目现在是盘 1"
    );
    let refusal = writable_mount_refused_before_any_write(&mut devices);
    assert!(
        matches!(
            refusal,
            MountError::Recovery(RecoveryFailure::InvariantViolated {
                invariant: "I-2.5",
                detail: "位置条目不按设备身份严格升序",
            })
        ),
        "逆序的第 0 代实例表指针：{refusal:?}"
    );
}

// ─── 第 33 条续：重建读另外三棵树也照冷走读判全，一棵一条 ───

/// 码 2 节点头里出生序号那 4 字节住 72 + 2k（D18（块里携带什么信息） 已定项 18 的偏移表），加一。
fn bump_the_birth_sequence_in_the_header(node: &mut [u8]) {
    let birth_sequence_offset = 72 + 2 * usize::from(node[51]);
    let birth_sequence = u32::from_le_bytes(
        node[birth_sequence_offset..birth_sequence_offset + 4]
            .try_into()
            .expect("4 字节"),
    );
    node[birth_sequence_offset..birth_sequence_offset + 4]
        .copy_from_slice(&(birth_sequence + 1).to_le_bytes());
}

fn refused_with_a_tree_root_birth_sequence_differing_from_its_pointer(
    refusal: &MountError,
) -> bool {
    matches!(
        refusal,
        MountError::Recovery(RecoveryFailure::InvariantViolated {
            invariant: "I-1.2",
            detail: "树根出生序号与指针不符",
        })
    )
}

/// 场景：第一个文件那一版的 extent 树根节点头里的出生序号加一，节点与指着它的每一道校验和重算。
/// 预期：可写挂载在任何写之前报 I-1.2「树根出生序号与指针不符」。改之前重建按「位置那几样」读 extent 树、不核头里的出生身份。
#[test]
fn an_extent_tree_root_whose_birth_sequence_differs_from_its_pointer_is_refused_by_the_writable_mount(
) {
    let (mut devices, _first) = pool_after_the_first_file();
    rewrite_the_root_node_of_a_tree_of_the_newest_root(
        &mut devices,
        singlefs_core::records::TREE_KIND_EXTENT,
        |node| bump_the_birth_sequence_in_the_header(node),
    );
    let refusal = writable_mount_refused_before_any_write(&mut devices);
    assert!(
        refused_with_a_tree_root_birth_sequence_differing_from_its_pointer(&refusal),
        "extent 树根的出生序号判 I-1.2：{refusal:?}"
    );
}

/// 场景：第一个文件那一版的分配记录树根节点头里的出生序号加一，节点与指着它的每一道校验和重算。
/// 预期：可写挂载在任何写之前报 I-1.2「树根出生序号与指针不符」。改之前重建按「位置那几样」读分配记录树、不核头里的出生身份。
#[test]
fn an_allocation_record_tree_root_whose_birth_sequence_differs_from_its_pointer_is_refused_by_the_writable_mount(
) {
    let (mut devices, _first) = pool_after_the_first_file();
    rewrite_the_root_node_of_a_tree_of_the_newest_root(
        &mut devices,
        singlefs_core::records::TREE_KIND_ALLOCATION,
        |node| bump_the_birth_sequence_in_the_header(node),
    );
    let refusal = writable_mount_refused_before_any_write(&mut devices);
    assert!(
        refused_with_a_tree_root_birth_sequence_differing_from_its_pointer(&refusal),
        "分配记录树根的出生序号判 I-1.2：{refusal:?}"
    );
}

/// 场景：第一个文件那一版的中央映射树（单个叶根）里第二条条目整条抄成第一条，叶重封、根记录里映射根指针的校验和重算。
/// 预期：可写挂载在任何写之前报「叶里的条目不按 key 严格递增」（I-1.1）。改之前重建只按「拼得成一棵树」读映射树、不判 key 次序。
#[test]
fn a_central_mapping_leaf_with_a_repeated_key_is_refused_by_the_writable_mount_before_any_write() {
    let (mut devices, _first) = pool_after_the_first_file();
    let (ring_slot, mut root) = newest_root_in_the_ring(&devices);
    let mut mapping_root = read_bytes(
        &devices,
        root.mapping_root.locations[0].device,
        root.mapping_root.locations[0].slot.to_device_offset(),
        node_bytes(),
    );
    let first_entry = mapping_root[entry_range_in_node(&mapping_root, 0)].to_vec();
    let second_entry_range = entry_range_in_node(&mapping_root, 1);
    mapping_root[second_entry_range].copy_from_slice(&first_entry);
    reseal_index_node(&mut mapping_root);
    let mapping_root_checksum =
        write_unit_at_both_locations(&mut devices, &root.mapping_root.locations, &mapping_root);
    for location in &mut root.mapping_root.locations {
        location.unit_checksum = mapping_root_checksum;
    }
    write_root_into_the_ring_slot(&mut devices, ring_slot, &root);
    let refusal = writable_mount_refused_before_any_write(&mut devices);
    assert!(
        matches!(
            refusal,
            MountError::Recovery(RecoveryFailure::InvariantViolated {
                invariant: "I-1.1",
                detail: "叶里的条目不按 key 严格递增",
            })
        ),
        "重复 key 的映射叶判 key 次序：{refusal:?}"
    );
}

/// 场景：只做过 mkfs 的池可写挂载两次（第二次写行 (1, …)，树表 0 条的这一版建起自己的分配记录树，C512），再把最新那条根的
/// 分配记录树根节点头里的出生序号加一，节点重封、根记录里那条指针的校验和重算。
/// 预期：第三次可写挂载在任何写之前报 I-1.2「树根出生序号与指针不符」。改之前重开树表 0 条的一版只按「位置那几样」读这棵树。
#[test]
fn the_allocation_record_tree_of_a_version_without_file_is_judged_like_the_cold_walk_on_the_writable_mount(
) {
    let mut devices = formatted_pool();
    mount_writable(&parameters(), &mut devices).expect("第一次可写挂载：实例 1，零单元发布");
    let second = mount_writable(&parameters(), &mut devices).expect("第二次可写挂载：实例 2 写行");
    assert!(
        second.current.file_version().is_none(),
        "第二次挂载之后仍是树表 0 条的一版"
    );
    let (ring_slot, mut root) = newest_root_in_the_ring(&devices);
    assert_ne!(
        root.allocation_record_tree_root,
        NodePointer::empty_root(),
        "写过行的树表 0 条那一版有自己的分配记录树"
    );
    let pointer = root.allocation_record_tree_root;
    let mut node = read_bytes(
        &devices,
        pointer.locations[0].device,
        pointer.locations[0].slot.to_device_offset(),
        node_bytes(),
    );
    bump_the_birth_sequence_in_the_header(&mut node);
    reseal_index_node(&mut node);
    let node_checksum = write_unit_at_both_locations(&mut devices, &pointer.locations, &node);
    for location in &mut root.allocation_record_tree_root.locations {
        location.unit_checksum = node_checksum;
    }
    write_root_into_the_ring_slot(&mut devices, ring_slot, &root);
    let refusal = writable_mount_refused_before_any_write(&mut devices);
    assert!(
        refused_with_a_tree_root_birth_sequence_differing_from_its_pointer(&refusal),
        "树表 0 条那一版的分配记录树根判 I-1.2：{refusal:?}"
    );
}

// ─── 记账条目宽先于记账条目数：台阶行解不开就认不出 ───

/// 两块内存盘上第一个文件（txg 3）之后再建一个空 inode（txg 4）：记账树单个叶根 16 条，池级 3 行、每盘 6 行之外，
/// 多一级第一个文件那次留下的 inode 号水位台阶行（代段 3，早于这一版的 txg；D5（快照 / 空间记账机制） 已定项 2 射程「inode 号水位的台阶行」）。
fn pool_whose_newest_accounting_root_carries_one_inode_number_watermark_step_row() -> Devices {
    let (mut devices, mut allocator, first) = pool_and_its_allocator_after_the_first_file();
    let parameters = parameters();
    let second = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_new_inodes(
            &mut writer,
            &mut allocator,
            &first,
            1,
            WRITE_TIME_SECONDS + 60,
            first.root.instance,
        )
        .expect("建一个空 inode")
    };
    let step_rows_older_than_this_version = second
        .accounting_entries
        .iter()
        .filter(|entry| {
            entry.statistic == STATISTIC_INODE_WATERMARK
                && entry.generation < second.root.checkpoint_txg
        })
        .count();
    assert_eq!(
        (
            second.root.checkpoint_txg,
            second.accounting_entries.len(),
            step_rows_older_than_this_version,
            second.accounting_tree.node_count()
        ),
        (CheckpointTxg(4), 16, 1, 1),
        "txg 4 那一版的记账树：单个叶根 16 条，其中 1 条是代段早于 4 的台阶行"
    );
    devices
}

/// 码 2 节点自述的条目宽缩到 key 宽、每条只留 key：条目数不变，条目区紧排、空出来的那一段清零，声明长度 = 条目数 × key 宽
/// （条目数住 82 + 2k、条目宽住 84 + 2k，声明长度住偏移 8）。
fn narrow_every_entry_of_an_index_node_to_its_key(node: &mut [u8]) {
    let key_width = usize::from(node[51]);
    let header_end = 86 + 2 * key_width;
    let entry_count = usize::from(u16::from_le_bytes([
        node[header_end - 4],
        node[header_end - 3],
    ]));
    let entry_ranges: Vec<std::ops::Range<usize>> = (0..entry_count)
        .map(|entry_index| entry_range_in_node(node, entry_index))
        .collect();
    let entries_start = header_end + 29;
    let entries_end_before = entry_ranges.last().map_or(entries_start, |range| range.end);
    // 第 i 条的 key 挪到 i × key 宽：key 宽小于原条目宽，写到的那一段止于 (i + 1) × key 宽，盖不到还没挪的那几条。
    for (entry_index, range) in entry_ranges.iter().enumerate() {
        node.copy_within(
            range.start..range.start + key_width,
            entries_start + entry_index * key_width,
        );
    }
    node[entries_start + entry_count * key_width..entries_end_before].fill(0);
    let narrowed_entry_width = u16::try_from(key_width).expect("key 宽是 1 字节的字段");
    node[header_end - 2..header_end].copy_from_slice(&narrowed_entry_width.to_le_bytes());
    let declared_length =
        u16::try_from(entry_count * key_width).expect("缩之后的条目区比缩之前短，装得进 2 字节");
    node[8..10].copy_from_slice(&declared_length.to_le_bytes());
}

/// 最新那条根（txg 4）的记账树根条目宽 34 缩到 key 宽 22、16 条每条只留 key，节点与指着它的每一道校验和重算。
fn pool_whose_accounting_root_carrying_a_step_row_is_narrowed_to_its_keys() -> Devices {
    let mut devices =
        pool_whose_newest_accounting_root_carries_one_inode_number_watermark_step_row();
    rewrite_the_root_node_of_a_tree_of_the_newest_root(
        &mut devices,
        TREE_KIND_ACCOUNTING,
        |node| narrow_every_entry_of_an_index_node_to_its_key(node),
    );
    devices
}

/// 记账条目窄于字段表那个成员：22 对 34。
fn accounting_entry_narrower_than_its_field_table() -> RecoveryFailure {
    RecoveryFailure::EntryNarrowerThanItsFieldTable {
        what: "记账条目",
        entry_bytes: usize::try_from(ACCOUNTING_KEY_BYTES).expect("22"),
        field_table_bytes: usize::try_from(ACCOUNTING_ENTRY_BYTES).expect("34"),
    }
}

/// 场景：第一个文件之后再建一个空 inode（txg 4），记账树根 16 条里有一级代段 3 的 inode 号水位台阶行；根节点自述的条目宽缩到 key 宽、
/// 每条只留 key（条目数 16 不变），节点与指着它的每一道校验和重算。
/// 预期：只读恢复交回 `EntryNarrowerThanItsFieldTable`（记账条目，22 对 34）。台阶行要解开才认得出（统计量标签与代段都在条目里）：
/// 数记账条目那一判排在条目宽之前时，解不开的台阶行被算进 3 + 6 × 盘数，16 ≠ 15，报成「记账条目数（inode 号水位的台阶行不算）
/// 不是 3 + 6 × 盘数」。checker 档坏盘战役 `every_fixed_panic_site_reports_its_own_error_member_instead_of_panicking` 的
/// 「条目宽：记账树根」那一种是同一个坏法，这条在 harness 档钉住次序。
#[test]
fn a_narrowed_accounting_root_carrying_a_step_row_is_refused_for_its_entry_width_by_the_recovery() {
    let devices = pool_whose_accounting_root_carrying_a_step_row_is_narrowed_to_its_keys();
    let report = recover(&devices, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::Failed {
            root: Some((InstanceGeneration(1), CheckpointTxg(4))),
            failure: accounting_entry_narrower_than_its_field_table(),
        },
        "只读恢复在 txg 4 那条根上报条目宽，不报条目数"
    );
}

/// 场景：同上一条的坏镜像，交给可写挂载。
/// 预期：可写挂载在任何写之前交回 `EntryNarrowerThanItsFieldTable`（记账条目，22 对 34）：重建那一版读记账树时同样先核条目宽，
/// 后数条目（判法与上一条同一个理由）。
#[test]
fn a_narrowed_accounting_root_carrying_a_step_row_is_refused_for_its_entry_width_by_the_writable_mount(
) {
    let mut devices = pool_whose_accounting_root_carrying_a_step_row_is_narrowed_to_its_keys();
    let refusal = writable_mount_refused_before_any_write(&mut devices);
    assert!(
        matches!(&refusal, MountError::Recovery(failure) if *failure == accounting_entry_narrower_than_its_field_table()),
        "可写挂载报条目宽，不报条目数：{refusal:?}"
    );
}

// ─── 记账条目逐条判代与 seq 先于记账条目数：代写到根之后的台阶行认不出 ───

/// 台阶行那个池（[`pool_whose_newest_accounting_root_carries_one_inode_number_watermark_step_row`]）最新那条根的 txg。
const TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW: CheckpointTxg = CheckpointTxg(4);
/// 台阶行的代改成的值：晚于根的 txg 4。
const STEP_ROW_GENERATION_AFTER_THE_ROOT: CheckpointTxg = CheckpointTxg(5);

/// 码 2 记账叶根里恰好一条满足 `is_the_entry_to_change` 的条目交给 `change` 改，之后全部条目按 key 重排、节点头里的最小 / 最大 key
/// （从偏移 52 起，各一个 key 宽）换成首末两条的 key：条目宽、条目数不变，key 照旧严格递增，判全那一道（节点头、key 区间与次序）照过。
fn rewrite_one_entry_of_an_accounting_leaf_root_keeping_the_key_order(
    node: &mut [u8],
    is_the_entry_to_change: impl Fn(&AccountingEntry) -> bool,
    change: impl FnOnce(&mut AccountingEntry),
) {
    let key_width = usize::from(node[51]);
    let header_end = 86 + 2 * key_width;
    let entry_count = usize::from(u16::from_le_bytes([
        node[header_end - 4],
        node[header_end - 3],
    ]));
    let entry_ranges: Vec<std::ops::Range<usize>> = (0..entry_count)
        .map(|entry_index| entry_range_in_node(node, entry_index))
        .collect();
    let mut entries: Vec<AccountingEntry> = entry_ranges
        .iter()
        .map(|range| {
            AccountingEntry::parse(&node[range.clone()]).expect("记账叶根的条目宽是 34，每条解得开")
        })
        .collect();
    let indices_to_change: Vec<usize> = (0..entries.len())
        .filter(|entry_index| is_the_entry_to_change(&entries[*entry_index]))
        .collect();
    assert_eq!(indices_to_change.len(), 1, "恰好一条条目要改");
    change(&mut entries[indices_to_change[0]]);
    entries.sort_by_key(AccountingEntry::sort_key);
    for (range, entry) in entry_ranges.iter().zip(&entries) {
        node[range.clone()].copy_from_slice(&entry.to_bytes());
    }
    let smallest_key = entries.first().expect("记账叶根至少一条").key_bytes();
    let largest_key = entries.last().expect("记账叶根至少一条").key_bytes();
    node[52..52 + key_width].copy_from_slice(&smallest_key);
    node[52 + key_width..52 + 2 * key_width].copy_from_slice(&largest_key);
}

/// 最新那条根（txg 4）记账树根里那一级台阶行（inode 号水位、代段 3）的代改成 5（晚于根），按 key 重排到这一版自己那一行（代 4）之后；
/// 节点与指着它的每一道校验和重算。攻方原型 `research/prompts/m3-batch52-54-code-r1-opus-model/probes/corrupt_on_disk_k1_append.rs` 的 ② 那一形。
fn pool_whose_accounting_step_row_carries_a_generation_after_the_root() -> Devices {
    let mut devices =
        pool_whose_newest_accounting_root_carries_one_inode_number_watermark_step_row();
    rewrite_the_root_node_of_a_tree_of_the_newest_root(
        &mut devices,
        TREE_KIND_ACCOUNTING,
        |node| {
            rewrite_one_entry_of_an_accounting_leaf_root_keeping_the_key_order(
                node,
                |entry| {
                    entry.statistic == STATISTIC_INODE_WATERMARK
                        && entry.generation < TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW
                },
                |step_row| step_row.generation = STEP_ROW_GENERATION_AFTER_THE_ROOT,
            );
        },
    );
    devices
}

/// 记账条目逐条判那一个成员：代晚于根或 seq 为 0。
fn accounting_entry_generation_after_the_root_or_sequence_zero() -> RecoveryFailure {
    RecoveryFailure::InvariantViolated {
        invariant: "E142 走读同款",
        detail: "记账条目的代晚于根或 seq 为 0",
    }
}

/// 场景：第一个文件之后再建一个空 inode（txg 4），记账树根 16 条里那一级台阶行（代段 3）的代改成 5，晚于根的 txg 4；条目按 key 重排、
/// 节点与指着它的每一道校验和重算。
/// 预期：只读恢复（冷走读）在 txg 4 那条根上报「记账条目的代晚于根或 seq 为 0」。数条目按「统计量是 inode 号水位且代早于根」认台阶行，
/// 代写到根之后就认不出、被算进 3 + 6 × 盘数（16 ≠ 15）：数条目排在逐条判代之前时，这一坏法报成「记账条目数（inode 号水位的台阶行不算）
/// 不是 3 + 6 × 盘数」（判决 `research/prompts/m3-batch52-54-code-r1-main-verification.md` 第二节 K1-1、第三节 G1）。
#[test]
fn an_accounting_step_row_whose_generation_is_after_the_root_is_refused_for_its_generation_by_the_recovery(
) {
    let devices = pool_whose_accounting_step_row_carries_a_generation_after_the_root();
    let report = recover(&devices, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::Failed {
            root: Some((InstanceGeneration(1), TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW)),
            failure: accounting_entry_generation_after_the_root_or_sequence_zero(),
        },
        "只读恢复在 txg 4 那条根上报记账条目的代晚于根，不报条目数"
    );
}

/// 场景：同上一条的坏镜像，交给可写挂载。
/// 预期：可写挂载在任何写之前交回「记账条目的代晚于根或 seq 为 0」：重建那一版读记账树时同样先逐条判代与 seq、后数条目
/// （判法与上一条同一个理由）。
#[test]
fn an_accounting_step_row_whose_generation_is_after_the_root_is_refused_for_its_generation_by_the_writable_mount(
) {
    let mut devices = pool_whose_accounting_step_row_carries_a_generation_after_the_root();
    let refusal = writable_mount_refused_before_any_write(&mut devices);
    assert!(
        matches!(&refusal, MountError::Recovery(failure) if *failure == accounting_entry_generation_after_the_root_or_sequence_zero()),
        "可写挂载报记账条目的代晚于根，不报条目数：{refusal:?}"
    );
}

// ─── 分配记录逐条判代与跨度先于记账条目宽与逐条判、数记账条目、数映射条目 ───

/// 分配记录的代改成的值：晚于台阶行那个池最新那条根的 txg 4。
const ALLOCATION_RECORD_GENERATION_AFTER_THE_ROOT: CheckpointTxg = CheckpointTxg(5);
/// 这一版自己那一行 inode 号水位（代段 4）的代改成的值：晚于台阶行那个池最新那条根的 txg 4。
const WATERMARK_ROW_GENERATION_AFTER_THE_ROOT: CheckpointTxg = CheckpointTxg(5);

/// 分配记录树里 `pointer` 指着的节点与它下面整棵子树：叶里满足 `is_the_record_to_change` 的记录交给 `change` 改（只改不进 key 的字段），
/// 改过的节点重封、写回两份落点；内部节点里指着改过的孩子的那条内部条目（key 10 + 子指针 86），子指针两条位置条目的整单元校验和
/// 换成孩子的新值。改过的节点连同它的新整单元校验和推进 `rewritten_nodes`，改过的记录数加进 `changed_records`；
/// 交回这个节点改过之后的整单元校验和，没改过交回 `None`。
fn rewrite_allocation_records_in_the_subtree(
    devices: &mut Devices,
    pointer: &NodePointer,
    is_the_record_to_change: &dyn Fn(&AllocationRecord) -> bool,
    change: &dyn Fn(&mut AllocationRecord),
    rewritten_nodes: &mut Vec<(NodePointer, u32)>,
    changed_records: &mut usize,
) -> Option<u32> {
    let mut node = read_bytes(
        devices,
        pointer.locations[0].device,
        pointer.locations[0].slot.to_device_offset(),
        node_bytes(),
    );
    let header = parse_index_node(&node).expect("分配记录树节点解得开");
    let key_width = usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect("10");
    let mut is_this_node_changed = false;
    for entry_index in 0..header.entries.len() {
        let entry_range = entry_range_in_node(&node, entry_index);
        if header.level == 0 {
            let mut record = AllocationRecord::parse(&node[entry_range.clone()])
                .expect("分配记录叶的条目宽是 20");
            if is_the_record_to_change(&record) {
                change(&mut record);
                node[entry_range].copy_from_slice(&record.to_bytes());
                *changed_records += 1;
                is_this_node_changed = true;
            }
            continue;
        }
        let child =
            NodePointer::read_from(&mut ByteReader::at(&node[entry_range.clone()], key_width));
        if let Some(child_checksum) = rewrite_allocation_records_in_the_subtree(
            devices,
            &child,
            is_the_record_to_change,
            change,
            rewritten_nodes,
            changed_records,
        ) {
            repoint_the_checksums_of_a_pointer(&mut node[entry_range], key_width, child_checksum);
            is_this_node_changed = true;
        }
    }
    if !is_this_node_changed {
        return None;
    }
    reseal_index_node(&mut node);
    let node_checksum = write_unit_at_both_locations(devices, &pointer.locations, &node);
    rewritten_nodes.push((*pointer, node_checksum));
    Some(node_checksum)
}

/// 把最新那条根的分配记录树里满足 `is_the_record_to_change` 的记录交给 `change` 改，再把指着改过的节点的每一道校验和都重算：
/// 父节点内部条目里的子指针（逐层到根）、树表条目里那条根指针、树表单元、中央映射树根里每个改过的节点那一条、
/// 根记录里树表、中央映射树根两条指针（带文件的那一版根记录里分配记录树根那一条是空的）。交回改了几条记录。
fn rewrite_allocation_records_of_the_newest_root(
    devices: &mut Devices,
    is_the_record_to_change: impl Fn(&AllocationRecord) -> bool,
    change: impl Fn(&mut AllocationRecord),
) -> usize {
    let (ring_slot, mut root) = newest_root_in_the_ring(devices);
    let mut tree_table = read_bytes(
        devices,
        root.tree_table.locations[0].device,
        root.tree_table.locations[0].slot.to_device_offset(),
        node_bytes(),
    );
    let tree_table_header = parse_index_node(&tree_table).expect("树表单元解得开");
    let entry_index = tree_table_header
        .entries
        .iter()
        .position(|entry| {
            TreeTableEntry::parse(entry).is_some_and(|parsed| parsed.kind == TREE_KIND_ALLOCATION)
        })
        .expect("树表里有分配记录树");
    let allocation_tree_root_pointer =
        TreeTableEntry::parse(&tree_table_header.entries[entry_index])
            .expect("树表条目解得开")
            .root;
    assert_eq!(
        root.allocation_record_tree_root,
        NodePointer::empty_root(),
        "带文件的那一版分配记录树根只在树表条目里，根记录里那一条是空的"
    );
    let mut rewritten_nodes: Vec<(NodePointer, u32)> = Vec::new();
    let mut changed_records = 0;
    let root_node_checksum = rewrite_allocation_records_in_the_subtree(
        devices,
        &allocation_tree_root_pointer,
        &is_the_record_to_change,
        &change,
        &mut rewritten_nodes,
        &mut changed_records,
    )
    .expect("至少改了一条记录：它所在的叶到根这一路都改过");

    let entry_range = entry_range_in_node(&tree_table, entry_index);
    // 树表条目里根指针从条目偏移 14 起（树 ID 8 + 条目长度 2 + 种类 2 + flags 2）。
    repoint_the_checksums_of_a_pointer(&mut tree_table[entry_range], 14, root_node_checksum);
    reseal_index_node(&mut tree_table);
    let tree_table_checksum =
        write_unit_at_both_locations(devices, &root.tree_table.locations, &tree_table);

    let mut mapping_root = read_bytes(
        devices,
        root.mapping_root.locations[0].device,
        root.mapping_root.locations[0].slot.to_device_offset(),
        node_bytes(),
    );
    for (node_pointer, node_checksum) in &rewritten_nodes {
        repoint_the_mapping_entry_of_an_index_node(
            &mut mapping_root,
            *node_pointer,
            *node_checksum,
        );
    }
    let mapping_root_checksum =
        write_unit_at_both_locations(devices, &root.mapping_root.locations, &mapping_root);

    for location in &mut root.tree_table.locations {
        location.unit_checksum = tree_table_checksum;
    }
    for location in &mut root.mapping_root.locations {
        location.unit_checksum = mapping_root_checksum;
    }
    write_root_into_the_ring_slot(devices, ring_slot, &root);
    changed_records
}

/// 数记账条目那一个成员。
fn accounting_entry_count_is_not_three_plus_six_per_device() -> RecoveryFailure {
    RecoveryFailure::InvariantViolated {
        invariant: "E142 走读同款",
        detail: "记账条目数（inode 号水位的台阶行不算）不是 3 + 6 × 盘数",
    }
}

/// 逐条判分配记录那一个成员：跨度为 0 或代晚于根。
fn allocation_record_span_zero_or_generation_after_the_root() -> RecoveryFailure {
    RecoveryFailure::InvariantViolated {
        invariant: "E142 走读同款",
        detail: "分配记录跨度为 0，或分配代 / 释放代晚于根",
    }
}

/// 映射条目数那一个成员。
fn mapping_entry_count_is_not_one_per_mapped_unit() -> RecoveryFailure {
    RecoveryFailure::InvariantViolated {
        invariant: "E142 走读同款",
        detail: "映射条目数不是 1 + extent 树、分配记录树、记账树、deadlist 树的节点数 + deadlist 条目数 + inode 叶容器数 + 数据单元数",
    }
}

/// 码 2 单个叶根里删掉第 `entry_index` 条条目：之后的条目前移一条、空出来的末条清零，条目数减一、声明长度（偏移 8，= 条目数 × 条目宽）跟着减，
/// 节点头里的最小 / 最大 key（从偏移 52 起，各一个 key 宽）换成新的首末两条的 key（条目的前 key 宽字节）。条目宽不变，key 照旧严格递增，
/// 判全那一道（节点头、key 区间与次序）照过。交回删掉的那条条目的字节；重封归调用方。
fn drop_one_entry_of_an_index_leaf_root(node: &mut [u8], entry_index: usize) -> Vec<u8> {
    let key_width = usize::from(node[51]);
    let header_end = 86 + 2 * key_width;
    let entry_count = usize::from(u16::from_le_bytes([
        node[header_end - 4],
        node[header_end - 3],
    ]));
    assert!(
        entry_count >= 2 && entry_index < entry_count,
        "叶根至少两条、删的那条在里面：删完还有首末两条可取 key（{entry_count} 条，删第 {entry_index} 条）"
    );
    let dropped_range = entry_range_in_node(node, entry_index);
    let last_range = entry_range_in_node(node, entry_count - 1);
    let entry_width = dropped_range.len();
    let dropped_entry = node[dropped_range.clone()].to_vec();
    node.copy_within(dropped_range.end..last_range.end, dropped_range.start);
    node[last_range].fill(0);
    let remaining_entry_count = entry_count - 1;
    let remaining_entry_count_field =
        u16::try_from(remaining_entry_count).expect("条目数比删之前少一条，装得进 2 字节");
    node[header_end - 4..header_end - 2]
        .copy_from_slice(&remaining_entry_count_field.to_le_bytes());
    let declared_length = u16::try_from(remaining_entry_count * entry_width)
        .expect("条目区比删之前短，装得进 2 字节");
    node[8..10].copy_from_slice(&declared_length.to_le_bytes());
    let first_range = entry_range_in_node(node, 0);
    let new_last_range = entry_range_in_node(node, remaining_entry_count - 1);
    let smallest_key = node[first_range.start..first_range.start + key_width].to_vec();
    let largest_key = node[new_last_range.start..new_last_range.start + key_width].to_vec();
    node[52..52 + key_width].copy_from_slice(&smallest_key);
    node[52 + key_width..52 + 2 * key_width].copy_from_slice(&largest_key);
    dropped_entry
}

/// 最新那条根的中央映射树根（单个叶根）里 key 最大的那一条删掉，重封、写回两份落点，根记录里映射树根那条指针两条位置条目的校验和换成新值。
fn drop_the_entry_of_the_largest_key_from_the_central_mapping_root_of_the_newest_root(
    devices: &mut Devices,
) {
    let (ring_slot, mut root) = newest_root_in_the_ring(devices);
    let mut mapping_root = read_bytes(
        devices,
        root.mapping_root.locations[0].device,
        root.mapping_root.locations[0].slot.to_device_offset(),
        node_bytes(),
    );
    let mapping_header = parse_index_node(&mapping_root).expect("映射树根解得开");
    assert_eq!(
        mapping_header.level, 0,
        "建空 inode 那一版的映射树是单个叶根"
    );
    let dropped_mapping_entry =
        drop_one_entry_of_an_index_leaf_root(&mut mapping_root, mapping_header.entries.len() - 1);
    assert!(
        parse_mapping_entry(&dropped_mapping_entry).is_some(),
        "删掉的是一条解得开的映射条目"
    );
    reseal_index_node(&mut mapping_root);
    let mapping_root_checksum =
        write_unit_at_both_locations(devices, &root.mapping_root.locations, &mapping_root);
    for location in &mut root.mapping_root.locations {
        location.unit_checksum = mapping_root_checksum;
    }
    write_root_into_the_ring_slot(devices, ring_slot, &root);
}

/// 最新那条根（txg 4）的分配记录树里罩这一版树表单元那个槽的两条记录（两盘同槽各一条，代 4、还着）代改成 5，晚于根；两盘一起改，
/// 「每个落点每盘一条」那一判（比的是各盘的（槽, 跨度, 代, 已释放）集合）照过。指着改过的节点的每一道校验和重算。
fn move_the_allocation_records_of_the_tree_table_slot_after_the_root(devices: &mut Devices) {
    let (_, root) = newest_root_in_the_ring(devices);
    let tree_table_slot = root.tree_table.locations[0].slot;
    let changed_records = rewrite_allocation_records_of_the_newest_root(
        devices,
        |record| record.slot == tree_table_slot,
        |record| {
            assert_eq!(
                (record.generation, record.is_released),
                (TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW, false),
                "罩这一版树表单元的记录是这一版分配的、还着"
            );
            record.generation = ALLOCATION_RECORD_GENERATION_AFTER_THE_ROOT;
        },
    );
    assert_eq!(changed_records, 2, "罩树表单元那个槽的分配记录两盘各一条");
}

/// 台阶行那个池（[`pool_whose_newest_accounting_root_carries_one_inode_number_watermark_step_row`]，txg 4）上两处一起坏，
/// 每一处都把指着它的校验和重算：
/// ① 记账树根里 key 最小的那一行（不是 inode 号水位的行）删掉，之后的条目前移、条目数与声明长度减一；统计量标签与代一处都不碰：
///    逐条判照过，数记账条目 16 − 1 − 1 条台阶行 = 14 ≠ 3 + 6 × 2，与读者认不认哪个标签无关。只坏这一处时只读恢复报记账条目数，这里先核一遍；
/// ② [`move_the_allocation_records_of_the_tree_table_slot_after_the_root`]。
fn pool_whose_allocation_records_carry_a_generation_after_the_root_and_whose_accounting_entry_count_is_off(
) -> Devices {
    let mut devices =
        pool_whose_newest_accounting_root_carries_one_inode_number_watermark_step_row();
    rewrite_the_root_node_of_a_tree_of_the_newest_root(
        &mut devices,
        TREE_KIND_ACCOUNTING,
        |node| {
            let dropped_row =
                AccountingEntry::parse(&drop_one_entry_of_an_index_leaf_root(node, 0))
                    .expect("记账叶根的条目宽是 34，删掉的那条解得开");
            assert_ne!(
                dropped_row.statistic, STATISTIC_INODE_WATERMARK,
                "删的不是 inode 号水位的行：条目数少的这一条与认不认台阶行无关"
            );
        },
    );
    assert_eq!(
        recover(&devices, JournalPolicy::Consult).outcome,
        RecoveryOutcome::Failed {
            root: Some((InstanceGeneration(1), TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW)),
            failure: accounting_entry_count_is_not_three_plus_six_per_device(),
        },
        "前提：只坏记账那一处时，只读恢复在 txg 4 那条根上报记账条目数"
    );
    move_the_allocation_records_of_the_tree_table_slot_after_the_root(&mut devices);
    devices
}

/// 台阶行那个池（txg 4）上两处一起坏，每一处都把指着它的校验和重算：
/// ① 中央映射树根（单个叶根）里 key 最大的那一条删掉（[`drop_the_entry_of_the_largest_key_from_the_central_mapping_root_of_the_newest_root`]）：
///    各树节点数、inode 叶容器数、数据单元数一样都没动，映射条目数比它们算出的少一条；记账树一处不碰，数记账条目照过。
///    只坏这一处时只读恢复报映射条目数，这里先核一遍；
/// ② [`move_the_allocation_records_of_the_tree_table_slot_after_the_root`]。
fn pool_whose_allocation_records_carry_a_generation_after_the_root_and_whose_mapping_entry_count_is_off(
) -> Devices {
    let mut devices =
        pool_whose_newest_accounting_root_carries_one_inode_number_watermark_step_row();
    drop_the_entry_of_the_largest_key_from_the_central_mapping_root_of_the_newest_root(
        &mut devices,
    );
    assert_eq!(
        recover(&devices, JournalPolicy::Consult).outcome,
        RecoveryOutcome::Failed {
            root: Some((InstanceGeneration(1), TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW)),
            failure: mapping_entry_count_is_not_one_per_mapped_unit(),
        },
        "前提：只坏映射那一处时，只读恢复在 txg 4 那条根上报映射条目数"
    );
    move_the_allocation_records_of_the_tree_table_slot_after_the_root(&mut devices);
    devices
}

/// 台阶行那个池（txg 4）上两处一起坏，每一处都把指着它的校验和重算：
/// ① 记账树根里这一版自己那一行 inode 号水位（代段 4，等于根的 txg；台阶行是代段 3 的那一行，不碰）的代改成 5，晚于根，条目按 key 重排：
///    条目宽照旧 34；台阶行照旧认得出，数记账条目照旧 3 + 6 × 2，只有逐条判代与 seq 那一判报它。只坏这一处时只读恢复报记账条目的代，这里先核一遍；
/// ② [`move_the_allocation_records_of_the_tree_table_slot_after_the_root`]。
fn pool_whose_allocation_records_and_one_accounting_entry_carry_a_generation_after_the_root(
) -> Devices {
    let mut devices =
        pool_whose_newest_accounting_root_carries_one_inode_number_watermark_step_row();
    rewrite_the_root_node_of_a_tree_of_the_newest_root(
        &mut devices,
        TREE_KIND_ACCOUNTING,
        |node| {
            rewrite_one_entry_of_an_accounting_leaf_root_keeping_the_key_order(
                node,
                |entry| {
                    entry.statistic == STATISTIC_INODE_WATERMARK
                        && entry.generation == TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW
                },
                |watermark_row_of_this_version| {
                    watermark_row_of_this_version.generation =
                        WATERMARK_ROW_GENERATION_AFTER_THE_ROOT;
                },
            );
        },
    );
    assert_eq!(
        recover(&devices, JournalPolicy::Consult).outcome,
        RecoveryOutcome::Failed {
            root: Some((InstanceGeneration(1), TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW)),
            failure: accounting_entry_generation_after_the_root_or_sequence_zero(),
        },
        "前提：只坏记账那一处时，只读恢复在 txg 4 那条根上报记账条目的代晚于根"
    );
    move_the_allocation_records_of_the_tree_table_slot_after_the_root(&mut devices);
    devices
}

/// 台阶行那个池（txg 4）上两处一起坏，每一处都把指着它的校验和重算：
/// ① 记账树根的条目宽 34 缩到 key 宽 22、16 条每条只留 key（[`pool_whose_accounting_root_carrying_a_step_row_is_narrowed_to_its_keys`]）：
///    条目宽那一判报它，条目解不开、逐条判与数条目都轮不到。只坏这一处时只读恢复报记账条目窄于字段表，这里先核一遍；
/// ② [`move_the_allocation_records_of_the_tree_table_slot_after_the_root`]。
fn pool_whose_allocation_records_carry_a_generation_after_the_root_and_whose_accounting_root_is_narrowed_to_its_keys(
) -> Devices {
    let mut devices = pool_whose_accounting_root_carrying_a_step_row_is_narrowed_to_its_keys();
    assert_eq!(
        recover(&devices, JournalPolicy::Consult).outcome,
        RecoveryOutcome::Failed {
            root: Some((InstanceGeneration(1), TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW)),
            failure: accounting_entry_narrower_than_its_field_table(),
        },
        "前提：只坏记账那一处时，只读恢复在 txg 4 那条根上报记账条目窄于字段表"
    );
    move_the_allocation_records_of_the_tree_table_slot_after_the_root(&mut devices);
    devices
}

/// 场景：第一个文件之后再建一个空 inode（txg 4）；记账树根里 key 最小的那一行删掉（标签与代都不碰，记账条目数因此对不上），
/// 罩这一版树表单元那个槽的两条分配记录代改成 5，晚于根；指着改过的节点的每一道校验和重算。
/// 预期：只读恢复（冷走读）在 txg 4 那条根上报「分配记录跨度为 0，或分配代 / 释放代晚于根」：逐条判分配记录的代与跨度排在数记账条目之前
/// （先判得了自己那一条的，再数；判决 `research/prompts/m3-batch56-code-r1-main-verification.md` 第三节 H4）。
/// 那一判挪到数记账条目之后，这份镜像报成记账条目数。排在数映射条目、记账条目逐条判、记账条目宽之前那三个次序由下面三张镜像各钉一个。
#[test]
fn allocation_records_whose_generation_is_after_the_root_are_refused_before_the_accounting_entry_count_by_the_recovery(
) {
    let devices =
        pool_whose_allocation_records_carry_a_generation_after_the_root_and_whose_accounting_entry_count_is_off();
    let report = recover(&devices, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::Failed {
            root: Some((InstanceGeneration(1), TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW)),
            failure: allocation_record_span_zero_or_generation_after_the_root(),
        },
        "只读恢复在 txg 4 那条根上报分配记录的代晚于根，不报记账条目数"
    );
}

/// 场景：同上一条的坏镜像，交给可写挂载。
/// 预期：可写挂载在任何写之前交回「分配记录跨度为 0，或分配代 / 释放代晚于根」：重建那一版在读记账树之前就逐条判分配记录的代与跨度
/// （与冷走读同一个次序），那一判挪到数记账条目之后，这份镜像报成记账条目数。
#[test]
fn allocation_records_whose_generation_is_after_the_root_are_refused_before_the_accounting_entry_count_by_the_writable_mount(
) {
    let mut devices =
        pool_whose_allocation_records_carry_a_generation_after_the_root_and_whose_accounting_entry_count_is_off();
    let refusal = writable_mount_refused_before_any_write(&mut devices);
    assert!(
        matches!(&refusal, MountError::Recovery(failure) if *failure == allocation_record_span_zero_or_generation_after_the_root()),
        "可写挂载报分配记录的代晚于根，不报记账条目数：{refusal:?}"
    );
}

/// 场景：第一个文件之后再建一个空 inode（txg 4）；中央映射树根里 key 最大的那一条删掉（映射条目数因此对不上，记账树不碰），
/// 罩这一版树表单元那个槽的两条分配记录代改成 5，晚于根；指着改过的节点的每一道校验和重算。
/// 预期：只读恢复（冷走读）在 txg 4 那条根上报「分配记录跨度为 0，或分配代 / 释放代晚于根」：逐条判分配记录的代与跨度也排在数映射条目之前
/// （判决 `research/prompts/m3-batch56-code-r1-main-verification.md` 第三节 H4「排在两个数之前」的另一个数；
/// 攻方原型 `research/prompts/m3-batch57-code-r1-opus-model/probes/corrupt_on_disk_k3_append.rs` 的 ③ 那一形）。
/// 数映射条目挪到那一判之前，这份镜像报成映射条目数，而可写挂载照报分配记录：两条路对同一张盘报不同的成员。
#[test]
fn allocation_records_whose_generation_is_after_the_root_are_refused_before_the_mapping_entry_count_by_the_recovery(
) {
    let devices =
        pool_whose_allocation_records_carry_a_generation_after_the_root_and_whose_mapping_entry_count_is_off();
    let report = recover(&devices, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::Failed {
            root: Some((InstanceGeneration(1), TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW)),
            failure: allocation_record_span_zero_or_generation_after_the_root(),
        },
        "只读恢复在 txg 4 那条根上报分配记录的代晚于根，不报映射条目数"
    );
}

/// 场景：同上一条的坏镜像，交给可写挂载。
/// 预期：可写挂载在任何写之前交回「分配记录跨度为 0，或分配代 / 释放代晚于根」：重建那一版逐条判分配记录在读记账树之前，
/// 数映射条目在读完记账树之后，与冷走读报同一个成员。
#[test]
fn allocation_records_whose_generation_is_after_the_root_are_refused_before_the_mapping_entry_count_by_the_writable_mount(
) {
    let mut devices =
        pool_whose_allocation_records_carry_a_generation_after_the_root_and_whose_mapping_entry_count_is_off();
    let refusal = writable_mount_refused_before_any_write(&mut devices);
    assert!(
        matches!(&refusal, MountError::Recovery(failure) if *failure == allocation_record_span_zero_or_generation_after_the_root()),
        "可写挂载报分配记录的代晚于根，不报映射条目数：{refusal:?}"
    );
}

/// 场景：第一个文件之后再建一个空 inode（txg 4）；记账树根里这一版自己那一行 inode 号水位的代改成 5（晚于根；台阶行不碰，记账条目数照旧对得上），
/// 罩这一版树表单元那个槽的两条分配记录代改成 5，晚于根；指着改过的节点的每一道校验和重算。
/// 预期：只读恢复（冷走读）在 txg 4 那条根上报「分配记录跨度为 0，或分配代 / 释放代晚于根」：逐条判分配记录的代与跨度排在
/// 记账条目逐条判代与 seq 之前（冷走读那一段的次序注释与重建相同：分配记录、条目宽、逐条判、数）。
/// 那一判挪到记账条目逐条判之后（仍在两个数之前），这份镜像报成「记账条目的代晚于根或 seq 为 0」。这张盘每条记账条目宽都是 34，
/// 条目宽那一判不坏，「分配记录排在条目宽之前」这半句它钉不到，由最后两条（记账树根缩到 key 宽的那张盘）钉。可写挂载那一条在下面。
#[test]
fn allocation_records_whose_generation_is_after_the_root_are_refused_before_each_accounting_entry_is_judged_by_the_recovery(
) {
    let devices =
        pool_whose_allocation_records_and_one_accounting_entry_carry_a_generation_after_the_root();
    let report = recover(&devices, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::Failed {
            root: Some((InstanceGeneration(1), TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW)),
            failure: allocation_record_span_zero_or_generation_after_the_root(),
        },
        "只读恢复在 txg 4 那条根上报分配记录的代晚于根，不报记账条目的代"
    );
}

/// 场景：同上一条的坏镜像，交给可写挂载。
/// 预期：可写挂载在任何写之前交回「分配记录跨度为 0，或分配代 / 释放代晚于根」：重建那一版逐条判分配记录在读记账树之前，
/// 排在记账条目逐条判代与 seq 之前，与冷走读报同一个成员。重建把那一判挪到记账条目逐条判之后（仍在数记账条目之前），
/// 这份镜像报成「记账条目的代晚于根或 seq 为 0」，而上面两张盘的可写挂载照报分配记录：那两张盘坏的是两个数，挪到两个数之前它们分不出
/// （攻方原型 `research/prompts/m3-batch58-code-r1-opus-output.md`「K3-3」一节的 `r-rebuild-alloc-after-each-entry`）。
#[test]
fn allocation_records_whose_generation_is_after_the_root_are_refused_before_each_accounting_entry_is_judged_by_the_writable_mount(
) {
    let mut devices =
        pool_whose_allocation_records_and_one_accounting_entry_carry_a_generation_after_the_root();
    let refusal = writable_mount_refused_before_any_write(&mut devices);
    assert!(
        matches!(&refusal, MountError::Recovery(failure) if *failure == allocation_record_span_zero_or_generation_after_the_root()),
        "可写挂载报分配记录的代晚于根，不报记账条目的代：{refusal:?}"
    );
}

/// 场景：第一个文件之后再建一个空 inode（txg 4）；记账树根的条目宽缩到 key 宽、每条只留 key（条目数 16 不变），
/// 罩这一版树表单元那个槽的两条分配记录代改成 5，晚于根；指着改过的节点的每一道校验和重算。
/// 预期：只读恢复（冷走读）在 txg 4 那条根上报「分配记录跨度为 0，或分配代 / 释放代晚于根」：逐条判分配记录的代与跨度排在
/// 记账条目宽那一判之前（冷走读那一段的次序：分配记录、条目宽、逐条判、数）。那一判挪到条目宽之后（逐条判与两个数之前），
/// 这份镜像报成记账条目窄于字段表（22 对 34）；上面那张逐条判坏的盘条目宽都对，分不出这一形。可写挂载那一条在下面。
#[test]
fn allocation_records_whose_generation_is_after_the_root_are_refused_before_the_accounting_entry_width_by_the_recovery(
) {
    let devices =
        pool_whose_allocation_records_carry_a_generation_after_the_root_and_whose_accounting_root_is_narrowed_to_its_keys();
    let report = recover(&devices, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::Failed {
            root: Some((InstanceGeneration(1), TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW)),
            failure: allocation_record_span_zero_or_generation_after_the_root(),
        },
        "只读恢复在 txg 4 那条根上报分配记录的代晚于根，不报记账条目窄于字段表"
    );
}

/// 场景：同上一条的坏镜像，交给可写挂载。
/// 预期：可写挂载在任何写之前交回「分配记录跨度为 0，或分配代 / 释放代晚于根」：重建那一版逐条判分配记录在读记账树之前，
/// 排在记账条目宽那一判之前，与冷走读报同一个成员。重建把那一判挪到条目宽之后（逐条判与数记账条目之前），这份镜像报成记账条目窄于字段表。
#[test]
fn allocation_records_whose_generation_is_after_the_root_are_refused_before_the_accounting_entry_width_by_the_writable_mount(
) {
    let mut devices =
        pool_whose_allocation_records_carry_a_generation_after_the_root_and_whose_accounting_root_is_narrowed_to_its_keys();
    let refusal = writable_mount_refused_before_any_write(&mut devices);
    assert!(
        matches!(&refusal, MountError::Recovery(failure) if *failure == allocation_record_span_zero_or_generation_after_the_root()),
        "可写挂载报分配记录的代晚于根，不报记账条目窄于字段表：{refusal:?}"
    );
}

// ─── 第 35 条续：单元区槽数那一处减法 ───

/// 场景：700 MiB 的盘（单元区起点在 784 MiB）。预期：单元区槽数交回「盘末尾在单元区起点之前」，带盘的字节数与起点槽号；
/// 784 MiB 整的盘单元区是 0 槽。改之前是一句直接的减法，`overflow-checks = true` 下下溢 panic。
#[test]
fn the_unit_area_slot_count_of_a_device_ending_before_the_unit_area_start_is_an_error() {
    use singlefs_core::allocator::{
        unit_area_slots_of_device_starting_at, DeviceEndsBeforeTheUnitAreaStart, UnitAreaStart,
    };
    let start = UnitAreaStart::of_the_default_journal_ring();
    assert_eq!(
        unit_area_slots_of_device_starting_at(700 * MEBIBYTE, start),
        Err(DeviceEndsBeforeTheUnitAreaStart {
            device_bytes: 700 * MEBIBYTE,
            unit_area_start: SlotNumber(UNIT_AREA_START_SLOT),
        })
    );
    assert_eq!(
        unit_area_slots_of_device_starting_at(784 * MEBIBYTE, start),
        Ok(0),
        "盘末尾正好在单元区起点：0 槽"
    );
}
