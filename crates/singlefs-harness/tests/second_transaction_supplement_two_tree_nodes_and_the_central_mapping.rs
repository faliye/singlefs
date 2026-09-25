//! 里程碑「第二个事务」增补 2 收口表第 49 行：C483（挂载态读映射与树节点回退都没有条款） 两条条款各自的会红用例。
//!
//! ② **树节点的位置提示读不出时经中央映射回退**（D19（块指针的结构与宽度预算） 已定项 8 末句：豁免三类之外的单元——
//!    码 1 数据单元与 extent 树根、inode 树根、inode 叶容器——位置提示读不出时一律经映射回退）。
//!    镜像：第一个事务那一版上把一个树节点整个搬到一个空槽（两块盘同搬、字节不变），原槽写零，中央映射里它那一条改指新落点，
//!    映射树根与根记录按新字节重封；指向它的父指针（树表条目或 inode 树内部条目）一个字节都不动——它的位置提示指着一个空槽，
//!    映射里是真落点（D19 已定项 5「搬迁只改这里一条条目」）。挂载态（`mount_read_only`）与冷走读（`recover`）都要多跳一次
//!    读回来、整池挂得上、文件字节不变。
//! ① **挂载态把整棵映射树读进来**（D19 已定项 5「挂载态怎么读映射」：映射多层时整棵读进挂载态，解引用照旧只查内存）。
//!    镜像：同一版上把映射长成两层——原来那片根兼叶原样搬去当叶，原槽换成一个层级 1 的内部节点、一条条目指那片叶
//!    （内部条目 = 分隔 key 27 + 子指针 86，D8（核心索引结构） 已定项 11）。根头里的 key 区间写成子树覆盖区间时整池照常挂上、
//!    文件读得回来；写成首末条目（一条分隔 key）那一档读者按「它管的区间不是孩子覆盖的那一段」拒绝（I-1.1）。
//!
//! 两种镜像都不是这一版的发布路径按产品容量写得出来的（今天没有搬迁；映射树要装满 294 条才分裂），由这里按字节表改出来；
//! 读路径只认盘上的字节。
//!
//! **extent 树下段两层时打开文件怎么读**（D8（核心索引结构） 已定项 14「挂载怎么读」：extent 树按需，打开文件时按位置走下去；
//! 代码三方 `research/prompts/m2-final-code-r3-main-verification.md` 第三节 Z15 那一格报的两个测试缺口）：
//! 发布路径写一个 145 个数据单元的文件（下段两片叶上面长出第 1 层的根），① 打开文件时实现自报读了几个节点，
//! 在要多跳的取样点上（下段两层；下段节点的提示过期、经映射回退）与块层数到的读次数对得上；② 下段节点的提示过期时
//! 经映射回退读得回全部 145 个数据单元。搬节点照上面那一套（映射仍是一片根兼叶）。

mod common;

use common::{build_pool, file_content, parameters, FILE_BYTES, FIXED_WRITE_TIME_SECONDS};
use singlefs_core::address::{
    CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, FileOffsetInBytes, InodeNumber,
    InstanceGeneration, SlotNumber,
};
use singlefs_core::bytes::{ByteReader, ByteWriter};
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::extent_tree::{
    ExtentLowerNodePosition, ExtentUpperLeafEntry, ExtentUpperLeafTarget,
};
use singlefs_core::inode_tree::InodeLeafContainerIndexInTree;
use singlefs_core::journal::JournalRecord;
use singlefs_core::mounted_read::{
    mount_read_only, open_pool_for_read, CentralMappingTreeReadsAtOpen, ExtentTreeReadsAtOpen,
    MountReadOnlyFailure, MountedReadOnly, OpenFileFailure, OpenPoolForReadFailure,
};
use singlefs_core::pointer::{LocationEntry, NodePointer};
use singlefs_core::records::{
    build_mapping_entry, mapping_key_for_data, mapping_key_for_node, parse_inode_internal_entry,
    parse_mapping_entry, TreeTableEntry, TREE_KIND_ACCOUNTING, TREE_KIND_ALLOCATION,
    TREE_KIND_EXTENT, TREE_KIND_INODE,
};
use singlefs_core::recovery::{
    allocation_records_under_root, rebuild_version, recover, JournalPolicy, RebuildVersionFailure,
    RebuiltVersion, RecoveryFailure, RecoveryOutcome,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{
    publish_sequential_write, FirstFile, PoolWriter, TransactionUnit, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::{
    build_index_node, data_unit_payload_capacity, parse_index_node, UNIT_CLASS_INDEX_NODE,
    UNIT_CLASS_PACKED,
};
use singlefs_format::{
    DATA_UNIT_BYTES, EXTENT_KEY_BYTES, JOURNAL_RING_DEFAULT_BYTES, MAPPING_KEY_BYTES, NODE_BYTES,
    NODE_POINTER_BYTES, UNIT_AREA_START_SLOT,
};
use singlefs_harness::crash::MemoryPool;
use singlefs_harness::read_tally::{JournalRingRegion, ReadCountingPoolReader};

/// 搬过去的落点：单元区起点之后 16384 个槽，第一个事务从单元区起点往上取最低空槽，离这里很远。
const RELOCATION_TARGET_SLOT: SlotNumber = SlotNumber(UNIT_AREA_START_SLOT + 16384);
/// 根槽宽 = physical_block_size（`common::parameters` 取 512）。
const ROOT_SLOT_BYTES: usize = 512;
const BOTH_DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

/// 一版的内存镜像与它最新那条根，外加写进去的文件内容与顶着这一版的那条 journal 记录
/// （从盘上重建上一版要它：`recovery::rebuild_version` 的 `record_standing_for_root`）。
/// 多数用例取第一个事务那一版（[`first_transaction_image`]，txg 3）；下段两层那几条取 145 个单元那一版
/// （[`image_of_a_file_whose_lower_extent_segment_has_two_levels`]）。
struct ImageOfOneVersion {
    image: MemoryPool,
    root: RootRecord,
    content: Vec<u8>,
    record: JournalRecord,
}

fn first_transaction_image(tag: &str) -> ImageOfOneVersion {
    let pool = build_pool(tag);
    ImageOfOneVersion {
        image: pool.memory_pool(),
        root: pool.output.root,
        content: file_content(),
        record: pool.output.record.clone(),
    }
}

/// 下段两层那个文件有几个数据单元：比一片下段叶装得下的 144 个多一个（D8（核心索引结构） 已定项 14「实现取值」）。
const DATA_UNITS_OF_THE_FILE_WITH_A_TWO_LEVEL_LOWER_SEGMENT: usize = 145;

/// 下段根（第 1 层第 0 个）与它下面第二片叶（第 0 层第 1 个，罩单元 144）：要多跳的取样点各搬一个。
const LOWER_SEGMENT_ROOT: ExtentLowerNodePosition = ExtentLowerNodePosition { level: 1, index: 0 };
const SECOND_LOWER_SEGMENT_LEAF: ExtentLowerNodePosition =
    ExtentLowerNodePosition { level: 0, index: 1 };

/// 第一个事务之后同一个进程里把文件顺序写成 145 个数据单元（最后一个装一半）：下段两片叶上面长出第 1 层的根。
/// 中央映射照旧是一片根兼叶（145 个数据单元加十来个树节点，不到分裂的 294 条），搬节点那一套照用。
fn image_of_a_file_whose_lower_extent_segment_has_two_levels(tag: &str) -> ImageOfOneVersion {
    let mut pool = build_pool(tag);
    let payload_capacity = data_unit_payload_capacity();
    let content: Vec<u8> = (0..(DATA_UNITS_OF_THE_FILE_WITH_A_TWO_LEVEL_LOWER_SEGMENT - 1)
        * payload_capacity
        + payload_capacity / 2)
        .map(|index| u8::try_from((index * 31 + 7) % 251).expect("小于 256"))
        .collect();
    let output = {
        let publish_parameters = parameters();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        let previous = pool.output.clone();
        publish_sequential_write(
            &mut writer,
            &mut pool.allocator,
            &previous,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            },
            InstanceGeneration(1),
        )
        .expect("顺序写 145 个单元")
    };
    assert_eq!(
        output
            .extent_tree
            .lower_nodes
            .iter()
            .map(|(position, _)| *position)
            .collect::<Vec<_>>(),
        vec![
            ExtentLowerNodePosition { level: 0, index: 0 },
            SECOND_LOWER_SEGMENT_LEAF,
            LOWER_SEGMENT_ROOT,
        ],
        "145 个单元：下段两片叶上面一个第 1 层的根"
    );
    ImageOfOneVersion {
        image: pool.memory_pool(),
        root: output.root,
        content,
        record: output.record.clone(),
    }
}

/// inode 1 的 extent 树下段里的一个节点，按盘上字节找：上段根兼叶里 inode 1 那一条带着下段根的指针（标签 1），
/// 叶的指针在下段根的内部条目里（key 24 + 子指针 86，按孩子的位置排、没有洞）。找到的指针就是它父节点里那一条，搬了之后是提示。
fn lower_extent_segment_node(
    built: &ImageOfOneVersion,
    position: ExtentLowerNodePosition,
) -> MappedUnit {
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    let parse_the_node_at = |pointer: &NodePointer| {
        parse_index_node(&read_unit(
            &built.image,
            DeviceIdentity(0),
            the_slot_both_locations_share(pointer),
            node_bytes,
        ))
        .expect("extent 树节点解得开")
    };
    let upper_root = parse_the_node_at(&tree_table_entry_of_kind(built, TREE_KIND_EXTENT).root);
    assert_eq!(
        (upper_root.level, upper_root.entries.len()),
        (0, 1),
        "上段只有 inode 1 所在那片根兼叶、一条叶条目"
    );
    let ExtentUpperLeafTarget::LowerSegmentRoot(lower_root_pointer) =
        ExtentUpperLeafEntry::parse(&upper_root.entries[0])
            .expect("上段叶条目解得开")
            .target
    else {
        panic!("145 个单元的文件有下段，上段叶条目带下段根的指针（标签 1）");
    };
    let pointer = if position == LOWER_SEGMENT_ROOT {
        lower_root_pointer
    } else {
        let lower_root = parse_the_node_at(&lower_root_pointer);
        assert_eq!(
            (lower_root.level, position.level),
            (1, 0),
            "下段根在第 1 层，要找的是它下面的一片叶"
        );
        let entry = &lower_root.entries[usize::try_from(position.index).expect("叶序号")];
        NodePointer::read_from(&mut ByteReader::at(
            entry,
            usize::try_from(EXTENT_KEY_BYTES).expect("24"),
        ))
    };
    mapped_tree_node(pointer, UNIT_CLASS_INDEX_NODE, node_bytes)
}

fn read_unit(
    image: &MemoryPool,
    device: DeviceIdentity,
    slot: SlotNumber,
    unit_bytes: usize,
) -> Vec<u8> {
    image
        .devices
        .get(&device)
        .expect("池里有这块盘")
        .read(slot.to_device_offset(), unit_bytes)
}

fn write_unit_to_both_devices(image: &mut MemoryPool, slot: SlotNumber, bytes: &[u8]) {
    for device in BOTH_DEVICES {
        image
            .devices
            .get_mut(&device)
            .expect("池里有这块盘")
            .write(slot.to_device_offset(), bytes);
    }
}

/// 一条码 2 / 码 3 指针的两条位置条目落在同一个槽（第一版两盘同槽）：交回那个槽。
fn the_slot_both_locations_share(pointer: &NodePointer) -> SlotNumber {
    the_slot_both_location_entries_share(&pointer.locations)
}

/// 一条指针（码 1 / 码 2 / 码 3 都一样）的两条位置条目落在同一个槽：交回那个槽。
fn the_slot_both_location_entries_share(locations: &[LocationEntry; 2]) -> SlotNumber {
    assert_eq!(
        locations[0].slot, locations[1].slot,
        "第一版两条位置条目同槽"
    );
    locations[0].slot
}

/// 这一版树表里某个种类那棵树的条目（按盘上字节解，不取进程内的发布产物）。
fn tree_table_entry_of_kind(built: &ImageOfOneVersion, kind: u16) -> TreeTableEntry {
    let tree_table = parse_index_node(&read_unit(
        &built.image,
        DeviceIdentity(0),
        the_slot_both_locations_share(&built.root.tree_table),
        usize::try_from(NODE_BYTES).expect("16384"),
    ))
    .expect("树表解得开");
    tree_table
        .entries
        .iter()
        .map(|entry_bytes| TreeTableEntry::parse(entry_bytes).expect("树表条目解得开"))
        .find(|entry| entry.kind == kind)
        .expect("第一个事务那一版的树表里有这棵树")
}

/// 要搬的那个进映射的单元（D19（块指针的结构与宽度预算） 已定项 8 末句：豁免三类之外的码 1 数据单元与码 2 / 码 3 树节点——
/// 前三样是那一句点名的，分配记录树与记账树的根同为进映射的码 2 节点，从盘上重建上一版与读分配记录时要读它们）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MappedUnitToRelocate {
    ExtentTreeRoot,
    InodeTreeRoot,
    InodeLeafContainer,
    AllocationTreeRoot,
    AccountingTreeRoot,
    DataUnit,
}

/// 被搬的单元：指向它的那条指针里的两条位置条目（父指针原样，搬完之后就是指着空槽的提示）、它在中央映射里的 key 与单元宽。
struct MappedUnit {
    hint_locations: [LocationEntry; 2],
    mapping_key: Vec<u8>,
    unit_bytes: usize,
}

/// 一个进映射的码 2 / 码 3 树节点：key 按类标签与节点指针算（D19 已定项 6）。
fn mapped_tree_node(pointer: NodePointer, unit_class: u8, unit_bytes: usize) -> MappedUnit {
    MappedUnit {
        hint_locations: pointer.locations,
        mapping_key: mapping_key_for_node(unit_class, pointer),
        unit_bytes,
    }
}

fn mapped_unit(built: &ImageOfOneVersion, which: MappedUnitToRelocate) -> MappedUnit {
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    let index_node_of_tree = |kind: u16| {
        mapped_tree_node(
            tree_table_entry_of_kind(built, kind).root,
            UNIT_CLASS_INDEX_NODE,
            node_bytes,
        )
    };
    match which {
        MappedUnitToRelocate::ExtentTreeRoot => index_node_of_tree(TREE_KIND_EXTENT),
        MappedUnitToRelocate::InodeTreeRoot => index_node_of_tree(TREE_KIND_INODE),
        MappedUnitToRelocate::AllocationTreeRoot => index_node_of_tree(TREE_KIND_ALLOCATION),
        MappedUnitToRelocate::AccountingTreeRoot => index_node_of_tree(TREE_KIND_ACCOUNTING),
        MappedUnitToRelocate::DataUnit => {
            let extent_root_pointer = tree_table_entry_of_kind(built, TREE_KIND_EXTENT).root;
            let extent_root = parse_index_node(&read_unit(
                &built.image,
                DeviceIdentity(0),
                the_slot_both_locations_share(&extent_root_pointer),
                node_bytes,
            ))
            .expect("extent 树根解得开");
            assert_eq!(
                extent_root.entries.len(),
                1,
                "第一个事务那一版只有 inode 1 一条上段叶条目"
            );
            // 一个数据单元的文件内联在上段叶条目里（D8（核心索引结构） 已定项 14，标签 2）。
            let ExtentUpperLeafTarget::InlineDataUnit(data_pointer) =
                ExtentUpperLeafEntry::parse(&extent_root.entries[0])
                    .expect("上段叶条目解得开")
                    .target
            else {
                panic!("第一个事务那一版的文件只有一个数据单元，内联在上段叶条目里");
            };
            MappedUnit {
                hint_locations: data_pointer.locations,
                mapping_key: mapping_key_for_data(data_pointer.head, data_pointer.write_order),
                unit_bytes: usize::try_from(DATA_UNIT_BYTES).expect("32768"),
            }
        }
        MappedUnitToRelocate::InodeLeafContainer => {
            let inode_root_pointer = tree_table_entry_of_kind(built, TREE_KIND_INODE).root;
            let inode_root = parse_index_node(&read_unit(
                &built.image,
                DeviceIdentity(0),
                the_slot_both_locations_share(&inode_root_pointer),
                node_bytes,
            ))
            .expect("inode 树根解得开");
            assert_eq!(inode_root.entries.len(), 1, "第一个事务那一版只有一片叶");
            let (_separator_key, _identity, leaf_pointer) =
                parse_inode_internal_entry(&inode_root.entries[0]).expect("内部条目解得开");
            mapped_tree_node(
                leaf_pointer,
                UNIT_CLASS_PACKED,
                usize::try_from(DATA_UNIT_BYTES).expect("32768"),
            )
        }
    }
}

/// 中央映射树根的盘上字节（根记录里那条指针是权威：映射树根自举豁免，D19 已定项 8）。
fn central_mapping_root_bytes(built: &ImageOfOneVersion) -> Vec<u8> {
    read_unit(
        &built.image,
        DeviceIdentity(0),
        the_slot_both_locations_share(&built.root.mapping_root),
        usize::try_from(NODE_BYTES).expect("16384"),
    )
}

/// 按原映射树根的头字段、换一组条目重装一片根兼叶：头里的树 ID、层级、key 区间、出生身份全照抄。
fn central_mapping_root_rebuilt_with(built: &ImageOfOneVersion, entries: &[Vec<u8>]) -> Vec<u8> {
    let original = parse_index_node(&central_mapping_root_bytes(built)).expect("映射树根解得开");
    build_index_node(
        original.tree,
        original.level,
        original.key_width,
        &original.smallest_key,
        &original.largest_key,
        original.birth_txg,
        &built.root.filesystem_identifier,
        original.instance,
        original.birth_sequence,
        u16::try_from(original.entry_width).expect("条目宽"),
        entries,
    )
}

/// 新的映射树根写回原槽（两块盘），根记录里映射根那条指针的整单元校验和换成新字节的，根槽按新根记录重封写回。
/// 根槽落在哪块盘、哪个偏移由根环几何按这条根的 txg 算（`root_ring::target_for_publish`）。
fn write_back_the_central_mapping_root_and_reseal_the_root(
    built: &mut ImageOfOneVersion,
    new_central_mapping_root: &[u8],
) {
    write_unit_to_both_devices(
        &mut built.image,
        the_slot_both_locations_share(&built.root.mapping_root),
        new_central_mapping_root,
    );
    let checksum = crc32_castagnoli(new_central_mapping_root);
    for location in &mut built.root.mapping_root.locations {
        location.unit_checksum = checksum;
    }
    let geometry = parameters().geometry;
    let target = target_for_publish(
        built.root.checkpoint_txg,
        geometry.root_ring_slots_per_region,
    );
    let root_device = parameters().region_devices[usize::try_from(target.region).expect("区域号")];
    built
        .image
        .devices
        .get_mut(&root_device)
        .expect("根槽所在的盘")
        .write(
            slot_offset(target, geometry.fixed_structure_slot_spacing),
            &built.root.to_slot(ROOT_SLOT_BYTES),
        );
}

/// 中央映射里要不要跟着改指新落点：改是「搬迁只改映射一条条目」；不改就是提示与映射都指着写零的旧槽。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CentralMappingAfterTheMove {
    PointsAtTheNewPlacement,
    StillPointsAtTheZeroedSlot,
}

/// 把一个进映射的树节点搬到 [`RELOCATION_TARGET_SLOT`]：新落点两块盘写同一份字节，旧槽两块盘写零，
/// 父指针一个字节不动（提示于是指着一个空槽）；按 `mapping` 决定映射里那一条改不改指新落点。交回旧槽。
fn relocate_leaving_the_location_hint_on_an_empty_slot(
    built: &mut ImageOfOneVersion,
    which: MappedUnitToRelocate,
    mapping: CentralMappingAfterTheMove,
) -> SlotNumber {
    let moved = mapped_unit(built, which);
    relocate_the_mapped_unit_leaving_the_location_hint_on_an_empty_slot(built, moved, mapping)
}

/// 同 [`relocate_leaving_the_location_hint_on_an_empty_slot`]，搬哪个单元由调用方找好交进来（extent 树下段的节点
/// 不在 [`MappedUnitToRelocate`] 里：它们只在下段长出来的那几版上有，见 [`lower_extent_segment_node`]）。
fn relocate_the_mapped_unit_leaving_the_location_hint_on_an_empty_slot(
    built: &mut ImageOfOneVersion,
    moved: MappedUnit,
    mapping: CentralMappingAfterTheMove,
) -> SlotNumber {
    let hinted_slot = the_slot_both_location_entries_share(&moved.hint_locations);
    let unit = read_unit(
        &built.image,
        DeviceIdentity(0),
        hinted_slot,
        moved.unit_bytes,
    );
    assert_eq!(
        crc32_castagnoli(&unit),
        moved.hint_locations[0].unit_checksum,
        "搬之前提示是对的：盘上就是它指的那个单元"
    );
    write_unit_to_both_devices(&mut built.image, RELOCATION_TARGET_SLOT, &unit);
    write_unit_to_both_devices(&mut built.image, hinted_slot, &vec![0u8; moved.unit_bytes]);

    let mapping_key = moved.mapping_key;
    let original_entries = parse_index_node(&central_mapping_root_bytes(built))
        .expect("映射树根解得开")
        .entries;
    assert_eq!(
        central_mapping_root_rebuilt_with(built, &original_entries),
        central_mapping_root_bytes(built),
        "原样重装映射树根要逐字节等于盘上那一份：下面改出来的镜像与原镜像只差那一条条目"
    );
    let entries_with_the_key: Vec<usize> = original_entries
        .iter()
        .enumerate()
        .filter(|(_index, entry)| {
            parse_mapping_entry(entry).expect("映射条目解得开").0 == mapping_key
        })
        .map(|(index, _entry)| index)
        .collect();
    assert_eq!(
        entries_with_the_key.len(),
        1,
        "这个单元在映射里恰好一条（D19 已定项 8：码 1、码 2、码 3 进映射）"
    );
    let new_locations = [
        LocationEntry {
            slot: RELOCATION_TARGET_SLOT,
            ..moved.hint_locations[0]
        },
        LocationEntry {
            slot: RELOCATION_TARGET_SLOT,
            ..moved.hint_locations[1]
        },
    ];
    match mapping {
        CentralMappingAfterTheMove::PointsAtTheNewPlacement => {
            let mut moved_entries = original_entries.clone();
            moved_entries[entries_with_the_key[0]] =
                build_mapping_entry(&mapping_key, new_locations);
            let new_central_mapping_root = central_mapping_root_rebuilt_with(built, &moved_entries);
            write_back_the_central_mapping_root_and_reseal_the_root(
                built,
                &new_central_mapping_root,
            );
        }
        CentralMappingAfterTheMove::StillPointsAtTheZeroedSlot => {}
    }
    hinted_slot
}

/// 验收：提示指空槽、映射里是真落点 ⇒ 挂载态与冷走读都多跳一次读回来、整池挂得上、文件字节不变。
fn the_relocated_node_reads_back_through_the_central_mapping(
    which: MappedUnitToRelocate,
    tag: &str,
) {
    let mut built = first_transaction_image(tag);
    let unit_bytes = mapped_unit(&built, which).unit_bytes;
    let hinted_slot = relocate_leaving_the_location_hint_on_an_empty_slot(
        &mut built,
        which,
        CentralMappingAfterTheMove::PointsAtTheNewPlacement,
    );
    for device in BOTH_DEVICES {
        assert!(
            read_unit(&built.image, device, hinted_slot, unit_bytes)
                .iter()
                .all(|byte| *byte == 0),
            "{which:?} 的位置提示指的槽在盘 {} 上已经是空槽",
            device.0
        );
    }

    let mounted = mount_read_only(&built.image).unwrap_or_else(|failure| {
        panic!("{which:?} 提示指空槽、映射里是真落点：整池要挂得上，实际 {failure:?}")
    });
    let file = mounted
        .mounted
        .open_file(&built.image, InodeNumber(FIRST_INODE_NUMBER))
        .expect("打开第一个文件");
    // extent 树按需读（D8（核心索引结构） 已定项 14，K4）：它的节点在打开文件时才读，那一跳记在打开文件那一步；
    // inode 树与其余的在打开挂载态时整棵读，跳记在打开挂载态那一步。
    let expected_hops_at_the_mount_and_at_the_file = match which {
        MappedUnitToRelocate::ExtentTreeRoot => (0, 1),
        MappedUnitToRelocate::InodeTreeRoot
        | MappedUnitToRelocate::InodeLeafContainer
        | MappedUnitToRelocate::AllocationTreeRoot
        | MappedUnitToRelocate::AccountingTreeRoot
        | MappedUnitToRelocate::DataUnit => (1, 0),
    };
    assert_eq!(
        (
            mounted.mounted.tree_node_stale_location_hint_hops_at_open(),
            file.tree_node_stale_location_hint_hops_at_open()
        ),
        expected_hops_at_the_mount_and_at_the_file,
        "{which:?}：树节点多跳一次（D19 已定项 5 硬规则 3 的观测点），跳在读它的那一步"
    );
    let output = file
        .read_at(
            &built.image,
            FileOffsetInBytes(0),
            u64::try_from(FILE_BYTES).expect("3000"),
        )
        .expect("读整个文件");
    assert_eq!(
        output.bytes, built.content,
        "{which:?}：挂载态读回的文件字节不变"
    );

    let report = recover(&built.image, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(1), CheckpointTxg(3)),
            content: built.content.clone(),
        },
        "{which:?}：冷走读同一条回退，文件读得回来"
    );
    assert_eq!(
        report.mapping_fallbacks, 1,
        "{which:?}：冷走读多跳一次（数据单元那一步提示没动，不多跳）"
    );
}

#[test]
fn an_extent_tree_root_whose_location_hint_points_at_an_empty_slot_reads_back_through_the_central_mapping(
) {
    the_relocated_node_reads_back_through_the_central_mapping(
        MappedUnitToRelocate::ExtentTreeRoot,
        "c483-extent-root",
    );
}

#[test]
fn an_inode_tree_root_whose_location_hint_points_at_an_empty_slot_reads_back_through_the_central_mapping(
) {
    the_relocated_node_reads_back_through_the_central_mapping(
        MappedUnitToRelocate::InodeTreeRoot,
        "c483-inode-root",
    );
}

#[test]
fn an_inode_leaf_container_whose_location_hint_points_at_an_empty_slot_reads_back_through_the_central_mapping(
) {
    the_relocated_node_reads_back_through_the_central_mapping(
        MappedUnitToRelocate::InodeLeafContainer,
        "c483-inode-leaf",
    );
}

/// 对照：同样搬走 extent 树根、映射却没跟着改（映射里也是那个写零的旧槽）⇒ 回退走了一趟仍读不出，
/// 两条路都报 `MappingStillUnreadable`（与冷走读里数据单元那一条回退同一个成员），不是读成别的东西。
/// 这一条钉住上面那几条的「读得回来」确实是映射里那一条新落点给的。
#[test]
fn an_extent_tree_root_moved_without_updating_the_central_mapping_is_still_unreadable_after_the_hop(
) {
    let mut built = first_transaction_image("c483-extent-root-mapping-stale");
    let hinted_slot = relocate_leaving_the_location_hint_on_an_empty_slot(
        &mut built,
        MappedUnitToRelocate::ExtentTreeRoot,
        CentralMappingAfterTheMove::StillPointsAtTheZeroedSlot,
    );
    let still_unreadable = RecoveryFailure::MappingStillUnreadable { slot: hinted_slot };
    // extent 树按需读（D8（核心索引结构） 已定项 14，K4）：打开挂载态不碰它，读不出报在打开文件那一步。
    let mounted = mount_read_only(&built.image).expect("打开挂载态不读 extent 树");
    assert_eq!(
        mounted
            .mounted
            .open_file(&built.image, InodeNumber(FIRST_INODE_NUMBER))
            .err(),
        Some(OpenFileFailure::ExtentTreeWalk(still_unreadable.clone())),
        "挂载态打开文件：提示与映射都指着空槽"
    );
    let report = recover(&built.image, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::Failed {
            root: Some((InstanceGeneration(1), CheckpointTxg(3))),
            failure: still_unreadable,
        },
        "冷走读：提示与映射都指着空槽"
    );
    assert_eq!(report.mapping_fallbacks, 1, "回退走过一趟");
}

/// 这个单元在从盘上重建出来的上一版里是哪个角色（`TransactionOutput::unit` 的键）。
fn role_in_the_rebuilt_version(which: MappedUnitToRelocate) -> TransactionUnit {
    match which {
        MappedUnitToRelocate::ExtentTreeRoot => TransactionUnit::ExtentRoot,
        MappedUnitToRelocate::InodeTreeRoot => TransactionUnit::InodeRoot,
        MappedUnitToRelocate::InodeLeafContainer => {
            TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::LEFTMOST)
        }
        MappedUnitToRelocate::AllocationTreeRoot => TransactionUnit::AllocationTree,
        MappedUnitToRelocate::AccountingTreeRoot => TransactionUnit::AccountingTree,
        MappedUnitToRelocate::DataUnit => TransactionUnit::Data(DataUnitIndexInFile::FIRST),
    }
}

/// C483 ② 跟着要做的那一件（实六报告第七节 Q5）：从盘上重建上一版（可写挂载与回退走的 `recovery::rebuild_version`）
/// 与冷走读、挂载态同一条回退——提示指空槽、映射里是真落点 ⇒ 重建做成，那个角色的字节就是搬之前那一份，位置项照抄父指针里的
/// （提示，不替写者改指针）；重建出来的分配记录与搬之前从盘上读的逐条相等。改之前 `rebuild_version` 只按提示读，报 `UnitUnreadable`。
/// 数据单元那一格同时分开「经映射读回来」与 N2 的「读不出就照抄位置项、不读内容」：读回来的字节不是空的。
#[test]
fn rebuilding_the_previous_version_reads_every_relocated_mapped_unit_through_the_central_mapping() {
    for which in [
        MappedUnitToRelocate::ExtentTreeRoot,
        MappedUnitToRelocate::InodeTreeRoot,
        MappedUnitToRelocate::InodeLeafContainer,
        MappedUnitToRelocate::AllocationTreeRoot,
        MappedUnitToRelocate::AccountingTreeRoot,
        MappedUnitToRelocate::DataUnit,
    ] {
        let mut built = first_transaction_image(&format!("rebuild-through-mapping-{which:?}"));
        let RebuiltVersion::WithFile(before_the_move) =
            rebuild_version(&built.image, &built.root, Some(built.record.clone()))
                .expect("搬之前重建得出来")
        else {
            panic!("第一个事务那一版带文件");
        };
        let moved = mapped_unit(&built, which);
        let original_bytes = read_unit(
            &built.image,
            DeviceIdentity(0),
            the_slot_both_location_entries_share(&moved.hint_locations),
            moved.unit_bytes,
        );
        let hinted_slot = relocate_leaving_the_location_hint_on_an_empty_slot(
            &mut built,
            which,
            CentralMappingAfterTheMove::PointsAtTheNewPlacement,
        );
        let rebuilt = rebuild_version(&built.image, &built.root, Some(built.record.clone()))
            .unwrap_or_else(|failure| {
                panic!("{which:?} 提示指空槽、映射里是真落点：重建要做成，实际 {failure:?}")
            });
        let RebuiltVersion::WithFile(after_the_move) = rebuilt else {
            panic!("{which:?}：第一个事务那一版带文件");
        };
        let carried = after_the_move.unit(role_in_the_rebuilt_version(which));
        assert_eq!(
            carried.bytes, original_bytes,
            "{which:?}：重建出来的那个角色的字节是经映射读回来的那一份"
        );
        assert_eq!(
            carried.slot, hinted_slot,
            "{which:?}：位置项照抄父指针里的提示（重建不替写者改指针）"
        );
        assert_eq!(
            after_the_move.allocation_records, before_the_move.allocation_records,
            "{which:?}：重建出来的分配记录与搬之前逐条相等"
        );
        assert_eq!(
            after_the_move.data_pointers, before_the_move.data_pointers,
            "{which:?}：数据指针照抄"
        );
    }
}

/// 读一条根的分配记录（影子账与重建分配器走的 `recovery::allocation_records_under_root`）同一条回退：分配记录树根的提示指空槽、
/// 映射里是真落点 ⇒ 读得出、与搬之前逐条相等；映射没跟着改 ⇒ 走了一趟仍读不出，报 `MappingStillUnreadable`（带提示的那个槽，
/// 映射里也是它）。改之前只按提示读，两种镜像都报 `UnitUnreadable`。
#[test]
fn reading_the_allocation_records_of_a_root_goes_through_the_central_mapping_when_the_hint_is_stale(
) {
    let mut relocated = first_transaction_image("allocation-records-through-mapping");
    let before_the_move =
        allocation_records_under_root(&relocated.image, &relocated.root).expect("搬之前读得出");
    assert!(!before_the_move.is_empty(), "第一个事务那一版有分配记录");
    relocate_leaving_the_location_hint_on_an_empty_slot(
        &mut relocated,
        MappedUnitToRelocate::AllocationTreeRoot,
        CentralMappingAfterTheMove::PointsAtTheNewPlacement,
    );
    assert_eq!(
        allocation_records_under_root(&relocated.image, &relocated.root),
        Ok(before_the_move),
        "提示指空槽、映射里是真落点：经映射读回来，逐条相等"
    );

    let mut mapping_stale = first_transaction_image("allocation-records-mapping-stale");
    let hinted_slot = relocate_leaving_the_location_hint_on_an_empty_slot(
        &mut mapping_stale,
        MappedUnitToRelocate::AllocationTreeRoot,
        CentralMappingAfterTheMove::StillPointsAtTheZeroedSlot,
    );
    assert_eq!(
        allocation_records_under_root(&mapping_stale.image, &mapping_stale.root),
        Err(RecoveryFailure::MappingStillUnreadable { slot: hinted_slot }),
        "提示与映射都指着空槽"
    );
}

/// 对照：extent 树根搬了而映射没跟着改 ⇒ 重建走了一趟仍读不出，报 `MappingStillUnreadable`（树节点不在 N2 的容下之列，
/// 那一条只管数据单元）。
#[test]
fn rebuilding_with_a_relocated_extent_tree_root_that_the_mapping_does_not_follow_fails_after_the_hop(
) {
    let mut built = first_transaction_image("rebuild-extent-root-mapping-stale");
    let hinted_slot = relocate_leaving_the_location_hint_on_an_empty_slot(
        &mut built,
        MappedUnitToRelocate::ExtentTreeRoot,
        CentralMappingAfterTheMove::StillPointsAtTheZeroedSlot,
    );
    assert_eq!(
        rebuild_version(&built.image, &built.root, Some(built.record.clone())),
        Err(RebuildVersionFailure::Walk(
            RecoveryFailure::MappingStillUnreadable { slot: hinted_slot }
        )),
        "提示与映射都指着空槽"
    );
}

/// 两层映射的根头里写的 key 区间：子树覆盖区间（D18（块里携带什么信息） 已定项 2），或只有那一条分隔 key（首末条目的 key）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KeyRangeInTheInternalRootHeader {
    CoveringTheSubtree,
    OnlyTheSeparatorKey,
}

/// 把中央映射长成两层：原来那片根兼叶原样搬到 [`RELOCATION_TARGET_SLOT`] 当叶，原槽换成一个层级 1 的内部节点，
/// 一条条目 = 那片叶的最小 key 27 + 指它的节点指针 86（D8（核心索引结构） 已定项 11 的内部条目）。
fn grow_the_central_mapping_into_two_levels(
    built: &mut ImageOfOneVersion,
    key_range: KeyRangeInTheInternalRootHeader,
) {
    let leaf_bytes = central_mapping_root_bytes(built);
    let leaf = parse_index_node(&leaf_bytes).expect("原映射树根解得开");
    assert_eq!(leaf.level, 0, "第一个事务那一版的映射树根是根兼叶");
    write_unit_to_both_devices(&mut built.image, RELOCATION_TARGET_SLOT, &leaf_bytes);
    let leaf_checksum = crc32_castagnoli(&leaf_bytes);
    let leaf_pointer = NodePointer {
        head: built.root.mapping_root.head,
        locations: [
            LocationEntry {
                slot: RELOCATION_TARGET_SLOT,
                unit_checksum: leaf_checksum,
                ..built.root.mapping_root.locations[0]
            },
            LocationEntry {
                slot: RELOCATION_TARGET_SLOT,
                unit_checksum: leaf_checksum,
                ..built.root.mapping_root.locations[1]
            },
        ],
        instance: leaf.instance,
        birth_sequence: leaf.birth_sequence,
    };
    let key_bytes = usize::try_from(MAPPING_KEY_BYTES).expect("27");
    let internal_entry_bytes = key_bytes + usize::try_from(NODE_POINTER_BYTES).expect("86");
    let mut writer = ByteWriter::new(internal_entry_bytes);
    writer.put(&leaf.smallest_key);
    leaf_pointer.write_to(&mut writer);
    let internal_entry = writer.into_bytes();
    let largest_key_in_the_header = match key_range {
        KeyRangeInTheInternalRootHeader::CoveringTheSubtree => &leaf.largest_key,
        KeyRangeInTheInternalRootHeader::OnlyTheSeparatorKey => &leaf.smallest_key,
    };
    let internal_root = build_index_node(
        leaf.tree,
        1,
        key_bytes,
        &leaf.smallest_key,
        largest_key_in_the_header,
        leaf.birth_txg,
        &built.root.filesystem_identifier,
        leaf.instance,
        leaf.birth_sequence,
        u16::try_from(internal_entry_bytes).expect("113"),
        &[internal_entry],
    );
    write_back_the_central_mapping_root_and_reseal_the_root(built, &internal_root);
}

/// C483 ① 换成 D19（块指针的结构与宽度预算） 已定项 5 重定之后的样子：映射长成两层 ⇒ 挂载态把整棵树读进来（根与叶各一次），
/// 整池只读挂载、冷走读都照常读回文件；打开之后的读查映射不发设备读（那一格的读数在并行线二的两层映射用例里重量）。
/// 判别力：挂载态只读根不读叶，映射里就只剩一条内部条目，这份镜像上它照样打得开，但 `node_reads` 是 1 不是 2。
#[test]
fn central_mapping_grown_into_two_levels_is_read_whole_into_the_mount_state_and_the_file_reads_back(
) {
    let mut built = first_transaction_image("c483-two-level-mapping");
    grow_the_central_mapping_into_two_levels(
        &mut built,
        KeyRangeInTheInternalRootHeader::CoveringTheSubtree,
    );
    let counting = ReadCountingPoolReader::new(
        &built.image,
        JournalRingRegion::starting_at_the_standard_slot(JOURNAL_RING_DEFAULT_BYTES),
    );
    let mounted = open_pool_for_read(&counting, &built.root).expect("两层映射照常打开");
    assert_eq!(
        mounted.central_mapping_tree_reads_at_open(),
        CentralMappingTreeReadsAtOpen {
            node_reads: 2,
            height: 2,
        },
        "根（层级 1）与它下面那片叶各读一次"
    );
    let file = mounted
        .open_file(&counting, InodeNumber(FIRST_INODE_NUMBER))
        .expect("打开第一个文件");
    let output = file
        .read_at(
            &counting,
            FileOffsetInBytes(0),
            u64::try_from(FILE_BYTES).expect("3000"),
        )
        .expect("读整个文件");
    assert_eq!(output.bytes, built.content, "挂载态读回的文件字节不变");

    let mounted_read_only = mount_read_only(&built.image).expect("整池只读挂载");
    assert_eq!(
        mounted_read_only
            .mounted
            .central_mapping_tree_reads_at_open(),
        CentralMappingTreeReadsAtOpen {
            node_reads: 2,
            height: 2,
        }
    );
    let report = recover(&built.image, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(1), CheckpointTxg(3)),
            content: built.content.clone(),
        },
        "冷走读沿两层映射照样读回文件"
    );
}

/// 同一份两层映射，根头里的 key 区间只写那一条分隔 key（首末条目的 key），不是子树覆盖区间：读者按块头自带的区间与孩子的区间对，
/// 对不上就拒（I-1.1：索引节点的身份是树 ID + 层级 + key 区间，D18（块里携带什么信息） 已定项 2），挂载态与冷走读同一个成员。
/// 判别力：`code_two_tree` 读树时不核内部节点的覆盖区间，这份镜像就照常打开——由红转绿。
#[test]
fn a_two_level_central_mapping_whose_root_header_does_not_cover_its_leaf_is_refused_by_both_readers(
) {
    let mut built = first_transaction_image("c483-two-level-mapping-not-covering");
    grow_the_central_mapping_into_two_levels(
        &mut built,
        KeyRangeInTheInternalRootHeader::OnlyTheSeparatorKey,
    );
    let refusal = RecoveryFailure::InvariantViolated {
        invariant: "I-1.1",
        detail:
            "内部节点的 key 区间不是子树覆盖区间（第一个孩子的最小 key 到最后一个孩子的最大 key）",
    };
    assert_eq!(
        open_pool_for_read(&built.image, &built.root).err(),
        Some(OpenPoolForReadFailure::Walk(refusal.clone())),
        "挂载态"
    );
    assert_eq!(
        mount_read_only(&built.image).err(),
        Some(MountReadOnlyFailure::Open(OpenPoolForReadFailure::Walk(
            refusal.clone()
        ))),
        "整池只读挂载"
    );
    let report = recover(&built.image, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::Failed {
            root: Some((InstanceGeneration(1), CheckpointTxg(3))),
            failure: refusal,
        },
        "冷走读"
    );
}

/// 提示过期的那一个节点多付的块层读：两条位置提示各试一次（读回全 0、校验和不对），再按映射的第一条就中
/// （`recovery::read_mapped_tree_node_via_hint_then_central_mapping`；数据单元那一条回退是同一个数，并行线二的用例钉着）。
/// 实现自报的节点数每个节点只算一次（`ExtentTreeReadsAtOpen::node_reads`，「不按位置条目数」），所以提示过期的那几跳
/// 各多出两次白试的块层读。
const WASTED_HINT_READS_PER_STALE_HOP: u64 = 2;

/// 打开下段两层那个文件那一步：实现自报的（extent 树读了几个节点、树高、叶数，树节点多跳几次）与块层数到的读次数。
struct ReadsWhileOpeningTheFile {
    reported_by_the_implementation: ExtentTreeReadsAtOpen,
    stale_location_hint_hops: u64,
    counted_by_the_block_layer: u64,
}

/// 只读挂载（打开挂载态不碰 extent 树），块层计数清零之后打开 inode 1：交回挂载态与打开那一步的读数。
fn open_the_file_counting_block_layer_reads(
    built: &ImageOfOneVersion,
) -> (MountedReadOnly, ReadsWhileOpeningTheFile) {
    let counting = ReadCountingPoolReader::new(
        &built.image,
        JournalRingRegion::starting_at_the_standard_slot(JOURNAL_RING_DEFAULT_BYTES),
    );
    let mounted = mount_read_only(&counting).expect("整池只读挂载");
    assert_eq!(
        mounted.mounted.tree_node_stale_location_hint_hops_at_open(),
        0,
        "打开挂载态不读 extent 树（按需，D8（核心索引结构） 已定项 14），它的节点一跳都不记在这一步"
    );
    counting.reset_tally();
    let reads = {
        let file = mounted
            .mounted
            .open_file(&counting, InodeNumber(FIRST_INODE_NUMBER))
            .expect("打开下段两层的那个文件");
        ReadsWhileOpeningTheFile {
            reported_by_the_implementation: file.extent_tree_reads_at_open(),
            stale_location_hint_hops: file.tree_node_stale_location_hint_hops_at_open(),
            counted_by_the_block_layer: counting.tally().reads,
        }
    };
    (mounted, reads)
}

/// 顺序读回全部数据单元，逐字节等于写进去的内容。
fn assert_every_data_unit_reads_back(
    mounted: &MountedReadOnly,
    reader: &MemoryPool,
    content: &[u8],
    what: &str,
) {
    let file = mounted
        .mounted
        .open_file(reader, InodeNumber(FIRST_INODE_NUMBER))
        .expect("打开下段两层的那个文件");
    let data_units =
        u64::try_from(DATA_UNITS_OF_THE_FILE_WITH_A_TWO_LEVEL_LOWER_SEGMENT).expect("单元数");
    assert_eq!(file.data_unit_count(), data_units, "{what}：单元数");
    let payload_capacity = u64::try_from(data_unit_payload_capacity()).expect("32634");
    let file_size = u64::try_from(content.len()).expect("文件大小");
    let mut read_back = Vec::with_capacity(content.len());
    for unit_number in 0..data_units {
        let first_byte = DataUnitIndexInFile(unit_number).first_file_byte(payload_capacity);
        let length = payload_capacity.min(file_size - first_byte.0);
        let output = file
            .read_at(reader, FileOffsetInBytes(first_byte.0), length)
            .unwrap_or_else(|failure| panic!("{what}：读第 {unit_number} 个单元：{failure:?}"));
        read_back.extend_from_slice(&output.bytes);
    }
    assert!(
        read_back == content,
        "{what}：顺序读回的 145 个单元与写入逐字节相同"
    );
}

/// Z15 缺口 ① 的第一个取样点：下段两层（比下段一片叶多一跳）。打开文件读 4 个节点——上段根兼叶、下段根、两片下段叶——
/// 实现自报 4、块层数到 4，树高 3（上段 1 + 下段 2）、叶 3。此前只在零多跳的取样点上对过（单单元 (1, 1)、四单元 (2, 2)）。
#[test]
fn opening_a_file_whose_lower_extent_segment_has_two_levels_reports_as_many_node_reads_as_the_block_layer_counts(
) {
    let built =
        image_of_a_file_whose_lower_extent_segment_has_two_levels("z15-two-level-node-reads");
    let (mounted, reads) = open_the_file_counting_block_layer_reads(&built);
    let reported = reads.reported_by_the_implementation;
    assert_eq!(
        (reported.node_reads, reported.height, reported.leaves),
        (4, 3, 3),
        "上段根兼叶、下段根、两片下段叶各读一次；树高 3、叶 3"
    );
    assert_eq!(reads.stale_location_hint_hops, 0);
    assert_eq!(
        reads.counted_by_the_block_layer, reported.node_reads,
        "打开文件读下段两层：块层数到的与实现自报的节点数相等"
    );
    assert_every_data_unit_reads_back(&mounted, &built.image, &built.content, "下段两层");
}

/// Z15 缺口 ② 与 ① 的第二个取样点：下段的一个节点提示过期（整个搬到空槽、原槽写零、父指针不动，映射里是真落点）。
/// 挂载态打开文件经映射多跳一次读回那个节点，读得回全部 145 个单元；冷走读同一条回退，读回同一份内容。
/// 自报的节点数照旧 4；块层数到 4 + 2 × 1 = 6——自报数每个节点算一次，多跳那一个节点另有两次白试的提示读
/// （[`WASTED_HINT_READS_PER_STALE_HOP`]）。
fn a_lower_extent_segment_node_whose_location_hint_is_stale_reads_back_through_the_central_mapping(
    position: ExtentLowerNodePosition,
    tag: &str,
) {
    let mut built = image_of_a_file_whose_lower_extent_segment_has_two_levels(tag);
    let moved = lower_extent_segment_node(&built, position);
    let unit_bytes = moved.unit_bytes;
    let hinted_slot = relocate_the_mapped_unit_leaving_the_location_hint_on_an_empty_slot(
        &mut built,
        moved,
        CentralMappingAfterTheMove::PointsAtTheNewPlacement,
    );
    for device in BOTH_DEVICES {
        assert!(
            read_unit(&built.image, device, hinted_slot, unit_bytes)
                .iter()
                .all(|byte| *byte == 0),
            "{position:?} 的位置提示指的槽在盘 {} 上已经是空槽",
            device.0
        );
    }
    let (mounted, reads) = open_the_file_counting_block_layer_reads(&built);
    let reported = reads.reported_by_the_implementation;
    assert_eq!(
        reads.stale_location_hint_hops, 1,
        "{position:?}：打开文件多跳一次（D19（块指针的结构与宽度预算） 已定项 5 硬规则 3 的观测点）"
    );
    assert_eq!(
        (reported.node_reads, reported.height, reported.leaves),
        (4, 3, 3),
        "{position:?}：自报的节点数照旧每个节点算一次"
    );
    assert_eq!(
        reads.counted_by_the_block_layer,
        reported.node_reads + WASTED_HINT_READS_PER_STALE_HOP * reads.stale_location_hint_hops,
        "{position:?}：块层数到的 = 自报的节点数 + 每一跳两条提示各白试一次"
    );
    assert_every_data_unit_reads_back(
        &mounted,
        &built.image,
        &built.content,
        &format!("{position:?} 提示过期"),
    );
    let report = recover(&built.image, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (built.root.instance, built.root.checkpoint_txg),
            content: built.content.clone(),
        },
        "{position:?}：冷走读同一条回退，读回全部 145 个单元"
    );
    assert_eq!(
        report.mapping_fallbacks, 1,
        "{position:?}：冷走读多跳一次（数据单元的提示都没动）"
    );
}

#[test]
fn the_root_of_a_two_level_lower_extent_segment_with_a_stale_location_hint_reads_back_every_data_unit_through_the_central_mapping(
) {
    a_lower_extent_segment_node_whose_location_hint_is_stale_reads_back_through_the_central_mapping(
        LOWER_SEGMENT_ROOT,
        "z15-stale-lower-root",
    );
}

#[test]
fn a_leaf_of_a_two_level_lower_extent_segment_with_a_stale_location_hint_reads_back_every_data_unit_through_the_central_mapping(
) {
    a_lower_extent_segment_node_whose_location_hint_is_stale_reads_back_through_the_central_mapping(
        SECOND_LOWER_SEGMENT_LEAF,
        "z15-stale-lower-leaf",
    );
}
