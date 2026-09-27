//! 实审 A3a：盘上内容可控时读者走得到的 panic（代码审阅第 29、32、33、35、38 条，C476 落在 core 读者这一侧的那几处）。
//! 规矩（`/tmp` 规格与 `code-discipline.md`「错误：能恢复的写进类型，破坏不变量的就断言」）：盘上读到的坏内容是可恢复的失败，
//! 每一处改成带名字的错误成员往上交；这里每条用例造一份让改之前那一版 panic、静默收下或报别的成员的坏镜像（各条文档写明改之前是哪一种），钉住改之后的结局。
//!
//! 盘都是内存稀疏盘（`singlefs_harness::crash::SparseBlockDevice`），池在进程里 mkfs、取号、暖机、写第一个文件（txg 3）造出来；
//! 改盘上字节之后，指着它的每一道校验和（父指针里的位置条目、映射条目、树表、根记录的自证校验和）都重算，读者走到被改的那一格才判得到。
//! 「在任何写之前拒」用两块盘此刻的整份镜像比：拒之前、拒之后逐字节相同。

use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
    TreeIdentifier,
};
use singlefs_core::allocator::PoolAllocator;
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
use singlefs_core::mount::{mount_writable, MountError};
use singlefs_core::pointer::{
    BirthSequence, DataPointer, LocationEntry, NodePointer, PointerHead,
    PointerHeadFieldOutsideTheFirstVersion,
};
use singlefs_core::records::{
    mapping_key_for_node, parse_mapping_entry, TreeTableEntry, TREE_KIND_ACCOUNTING,
    TREE_KIND_INODE, TREE_KIND_LIVELIST,
};
use singlefs_core::recovery::{
    choose_system_configuration, every_root_ring_slot, read_root_ring_slot, recover, JournalPolicy,
    PoolReader, RecoveryFailure, RecoveryOutcome, RootRingSlotReading,
    SystemConfigurationValueOutsideWhatThisReaderAccepts,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, RootRingSlot, RootRingSlotsPerRegion};
use singlefs_core::system_configuration::{
    SystemConfiguration, SystemImmutableSizes, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, warm_up, FirstFile, PoolWriter, TransactionOutput,
};
use singlefs_core::unit::{
    build_data_unit, build_index_node, build_packed_unit, parse_data_unit, parse_index_node,
    parse_packed_unit, seal_header_checksum, DataUnitIdentity, PackedIdentity, UnitError,
    WriteOrder, ENTRY_WIDTH_ZERO_WITH_ENTRIES, PACKED_TYPE_INODE, RECORD_WIDTH_ZERO_WITH_RECORDS,
    UNIT_CLASS_INDEX_NODE,
};
use singlefs_format::{
    DATA_UNIT_BYTES, JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES, NODE_BYTES,
    SYSTEM_CONFIGURATION_SLOT_BYTES, UNIT_AREA_START_SLOT,
};
use singlefs_harness::crash::{SparseBlockDevice, SparseDevice};

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
    (devices, first)
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
    let mapping_key = mapping_key_for_node(UNIT_CLASS_INDEX_NODE, child);
    let mapping_header = parse_index_node(&mapping_root).expect("映射树根解得开");
    assert_eq!(
        mapping_header.level, 0,
        "第一个文件那一版的映射树是单个叶根"
    );
    let mapping_entry_index = mapping_header
        .entries
        .iter()
        .position(|entry| parse_mapping_entry(entry).is_some_and(|(key, _)| key == mapping_key))
        .expect("映射里有这个节点");
    let mapping_entry_range = entry_range_in_node(&mapping_root, mapping_entry_index);
    for location_index in 0..2 {
        // 映射条目：key 27 + 位置条目 14 × 2，每条的校验和住条目偏移 4 + 6。
        let checksum_offset = mapping_entry_range.start + 27 + location_index * 14 + 10;
        mapping_root[checksum_offset..checksum_offset + 4]
            .copy_from_slice(&node_checksum.to_le_bytes());
    }
    reseal_index_node(&mut mapping_root);
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

/// 场景：最新那条根的树表里 livelist 那条条目的树 ID 改成根记录的树 ID 水位（第一个文件那一版是 19），树表与根记录重封。
/// 预期：可写挂载在任何写之前报 I-7.8（根记录树 ID 水位不低于全池最大树 ID）「树 ID 不低于水位」。改之前重建不判它，
/// 写行那次发布照抄这张树表，在 `transaction` 的「树表条目按树 ID 升序」断言上 panic（livelist 的号 19 排到了 deadlist 18 前面）。
#[test]
fn a_tree_table_entry_at_or_above_the_tree_identifier_watermark_is_refused_by_the_writable_mount() {
    let (mut devices, first) = pool_after_the_first_file();
    let watermark = first.root.tree_identifier_watermark;
    rewrite_a_tree_table_entry_of_the_newest_root(&mut devices, TREE_KIND_LIVELIST, |entry| {
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
/// 预期：第二次重开照常做成，这条根的账算「解不开」（`abandoned_roots_unreadable` 是 1），一个槽都不因它隔离。
/// 改之前影子账只判了两条位置条目同槽，槽 100 进 `isolate_abandoned`，在 `DeviceFreeMap::index` 的 expect 上 panic。
#[test]
fn an_abandoned_root_whose_instance_table_pointer_sits_below_the_unit_area_counts_as_unreadable_instead_of_panicking(
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
    let second_remount = mount_writable(&parameters(), &mut devices)
        .expect("被抛弃根的指针不在单元区里：这条根的账算读不出，挂载照常");
    assert_eq!(second_remount.output.instance, InstanceGeneration(3));
    assert_eq!(
        second_remount.output.abandoned_roots_unreadable, 1,
        "指针槽号不在单元区里的那条被抛弃根算「账读不出」"
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
        TreeIdentifier(14),
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
            birth_tree: TreeIdentifier(12),
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
            tree: TreeIdentifier(11),
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
            birth_tree: TreeIdentifier(14),
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
        NodePointer::read_judging_the_encryption_and_compression_fields_from(&mut ByteReader::at(
            &clean, 0
        )),
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
            NodePointer::read_judging_the_encryption_and_compression_fields_from(
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
        DataPointer::read_judging_the_encryption_and_compression_fields_from(&mut ByteReader::at(
            &bytes, 0
        )),
        Ok(data_pointer)
    );
    bytes[28] = 1;
    assert_eq!(
        DataPointer::read_judging_the_encryption_and_compression_fields_from(&mut ByteReader::at(
            &bytes, 0
        )),
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
        new_tree_identifier_watermark: 19,
        new_rollback_floor: CheckpointTxg(0),
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
        TreeIdentifier(14),
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
            birth_tree: TreeIdentifier(12),
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
