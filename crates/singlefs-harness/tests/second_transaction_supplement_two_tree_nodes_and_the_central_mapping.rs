//! 里程碑「第二个事务」增补 2 收口表第 49 行：C483（挂载态读映射与树节点回退都没有条款） 两条条款各自的会红用例。
//!
//! ② **树节点的位置提示读不出时经中央映射回退**（D19（块指针的结构与宽度预算） 已定项 8 末句：豁免三类之外的单元——
//!    码 1 数据单元与 extent 树根、inode 树根、inode 叶容器——位置提示读不出时一律经映射回退）。
//!    镜像：第一个事务那一版上把一个树节点整个搬到一个空槽（两块盘同搬、字节不变），原槽写零，中央映射里它那一条改指新落点，
//!    映射树根与根记录按新字节重封；指向它的父指针（树表条目或 inode 树内部条目）一个字节都不动——它的位置提示指着一个空槽，
//!    映射里是真落点（D19 已定项 5「搬迁只改这里一条条目」）。挂载态（`mount_read_only`）与冷走读（`recover`）都要多跳一次
//!    读回来、整池挂得上、文件字节不变。
//! ① **挂载态整片读映射，映射根不是根兼叶就拒绝打开**（D19 已定项 5「挂载态怎么读映射」，第一版的限制）。
//!    镜像：同一版上把映射长成两层——原来那片根兼叶原样搬去当叶，原槽换成一个层级 1 的内部节点、一条条目指那片叶。
//!
//! 两种镜像都不是发布路径写得出来的（今天没有搬迁、映射也不分裂），由这里按字节表改出来；读路径只认盘上的字节。

mod common;

use common::{build_pool, file_content, parameters, FILE_BYTES};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, FileOffsetInBytes, InodeNumber, InstanceGeneration, SlotNumber,
};
use singlefs_core::bytes::ByteWriter;
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::mounted_read::{
    mount_read_only, open_pool_for_read, MountReadOnlyFailure, OpenPoolForReadFailure,
};
use singlefs_core::pointer::{LocationEntry, NodePointer};
use singlefs_core::records::{
    build_mapping_entry, mapping_key_for_node, parse_inode_internal_entry, parse_mapping_entry,
    TreeTableEntry, TREE_KIND_EXTENT, TREE_KIND_INODE,
};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryFailure, RecoveryOutcome};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::FIRST_INODE_NUMBER;
use singlefs_core::unit::{
    build_index_node, parse_index_node, UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED,
};
use singlefs_format::{
    DATA_UNIT_BYTES, JOURNAL_RING_DEFAULT_BYTES, MAPPING_KEY_BYTES, NODE_BYTES, NODE_POINTER_BYTES,
    UNIT_AREA_START_SLOT,
};
use singlefs_harness::crash::MemoryPool;
use singlefs_harness::read_tally::{JournalRingRegion, ReadCountingPoolReader};

/// 搬过去的落点：单元区起点之后 16384 个槽，第一个事务从单元区起点往上取最低空槽，离这里很远。
const RELOCATION_TARGET_SLOT: SlotNumber = SlotNumber(UNIT_AREA_START_SLOT + 16384);
/// 根槽宽 = physical_block_size（`common::parameters` 取 512）。
const ROOT_SLOT_BYTES: usize = 512;
const BOTH_DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

/// 第一个事务那一版的内存镜像与它最新那条根（txg 3），外加写进去的文件内容。
struct FirstTransactionImage {
    image: MemoryPool,
    root: RootRecord,
    content: Vec<u8>,
}

fn first_transaction_image(tag: &str) -> FirstTransactionImage {
    let pool = build_pool(tag);
    FirstTransactionImage {
        image: pool.memory_pool(),
        root: pool.output.root,
        content: file_content(),
    }
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
    assert_eq!(
        pointer.locations[0].slot, pointer.locations[1].slot,
        "第一版两条位置条目同槽"
    );
    pointer.locations[0].slot
}

/// 这一版树表里某个种类那棵树的条目（按盘上字节解，不取进程内的发布产物）。
fn tree_table_entry_of_kind(built: &FirstTransactionImage, kind: u16) -> TreeTableEntry {
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

/// 要搬的那个进映射的树节点（D19（块指针的结构与宽度预算） 已定项 8 末句点名的三样）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MappedTreeNodeToRelocate {
    ExtentTreeRoot,
    InodeTreeRoot,
    InodeLeafContainer,
}

/// 被搬的节点：指向它的那条指针（父指针原样）、它的类标签与单元宽。
struct MappedTreeNode {
    pointer: NodePointer,
    unit_class: u8,
    unit_bytes: usize,
}

fn mapped_tree_node(
    built: &FirstTransactionImage,
    which: MappedTreeNodeToRelocate,
) -> MappedTreeNode {
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    match which {
        MappedTreeNodeToRelocate::ExtentTreeRoot => MappedTreeNode {
            pointer: tree_table_entry_of_kind(built, TREE_KIND_EXTENT).root,
            unit_class: UNIT_CLASS_INDEX_NODE,
            unit_bytes: node_bytes,
        },
        MappedTreeNodeToRelocate::InodeTreeRoot => MappedTreeNode {
            pointer: tree_table_entry_of_kind(built, TREE_KIND_INODE).root,
            unit_class: UNIT_CLASS_INDEX_NODE,
            unit_bytes: node_bytes,
        },
        MappedTreeNodeToRelocate::InodeLeafContainer => {
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
            MappedTreeNode {
                pointer: leaf_pointer,
                unit_class: UNIT_CLASS_PACKED,
                unit_bytes: usize::try_from(DATA_UNIT_BYTES).expect("32768"),
            }
        }
    }
}

/// 中央映射树根的盘上字节（根记录里那条指针是权威：映射树根自举豁免，D19 已定项 8）。
fn central_mapping_root_bytes(built: &FirstTransactionImage) -> Vec<u8> {
    read_unit(
        &built.image,
        DeviceIdentity(0),
        the_slot_both_locations_share(&built.root.mapping_root),
        usize::try_from(NODE_BYTES).expect("16384"),
    )
}

/// 按原映射树根的头字段、换一组条目重装一片根兼叶：头里的树 ID、层级、key 区间、出生身份全照抄。
fn central_mapping_root_rebuilt_with(
    built: &FirstTransactionImage,
    entries: &[Vec<u8>],
) -> Vec<u8> {
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
    built: &mut FirstTransactionImage,
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
    built: &mut FirstTransactionImage,
    which: MappedTreeNodeToRelocate,
    mapping: CentralMappingAfterTheMove,
) -> SlotNumber {
    let node = mapped_tree_node(built, which);
    let hinted_slot = the_slot_both_locations_share(&node.pointer);
    let unit = read_unit(
        &built.image,
        DeviceIdentity(0),
        hinted_slot,
        node.unit_bytes,
    );
    assert_eq!(
        crc32_castagnoli(&unit),
        node.pointer.locations[0].unit_checksum,
        "搬之前提示是对的：盘上就是它指的那个单元"
    );
    write_unit_to_both_devices(&mut built.image, RELOCATION_TARGET_SLOT, &unit);
    write_unit_to_both_devices(&mut built.image, hinted_slot, &vec![0u8; node.unit_bytes]);

    let mapping_key = mapping_key_for_node(node.unit_class, node.pointer);
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
        "这个节点在映射里恰好一条（D19 已定项 8：码 2 / 码 3 进映射）"
    );
    let new_locations = [
        LocationEntry {
            slot: RELOCATION_TARGET_SLOT,
            ..node.pointer.locations[0]
        },
        LocationEntry {
            slot: RELOCATION_TARGET_SLOT,
            ..node.pointer.locations[1]
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
    which: MappedTreeNodeToRelocate,
    tag: &str,
) {
    let mut built = first_transaction_image(tag);
    let unit_bytes = mapped_tree_node(&built, which).unit_bytes;
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
    assert_eq!(
        mounted.mounted.tree_node_stale_location_hint_hops_at_open(),
        1,
        "{which:?}：打开挂载态时树节点多跳一次（D19 已定项 5 硬规则 3 的观测点）"
    );
    let file = mounted
        .mounted
        .open_file(InodeNumber(FIRST_INODE_NUMBER))
        .expect("打开第一个文件");
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
        MappedTreeNodeToRelocate::ExtentTreeRoot,
        "c483-extent-root",
    );
}

#[test]
fn an_inode_tree_root_whose_location_hint_points_at_an_empty_slot_reads_back_through_the_central_mapping(
) {
    the_relocated_node_reads_back_through_the_central_mapping(
        MappedTreeNodeToRelocate::InodeTreeRoot,
        "c483-inode-root",
    );
}

#[test]
fn an_inode_leaf_container_whose_location_hint_points_at_an_empty_slot_reads_back_through_the_central_mapping(
) {
    the_relocated_node_reads_back_through_the_central_mapping(
        MappedTreeNodeToRelocate::InodeLeafContainer,
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
        MappedTreeNodeToRelocate::ExtentTreeRoot,
        CentralMappingAfterTheMove::StillPointsAtTheZeroedSlot,
    );
    let still_unreadable = RecoveryFailure::MappingStillUnreadable { slot: hinted_slot };
    assert_eq!(
        mount_read_only(&built.image).err(),
        Some(MountReadOnlyFailure::Open(OpenPoolForReadFailure::Walk(
            still_unreadable.clone()
        ))),
        "挂载态：提示与映射都指着空槽"
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

/// 把中央映射长成两层：原来那片根兼叶原样搬到 [`RELOCATION_TARGET_SLOT`] 当叶，原槽换成一个层级 1 的内部节点，
/// 一条条目 = 那片叶的最小 key 27 + 指它的节点指针 86（映射树内部条目的字段表今天没有条款，这里只要它是
/// 「一片真叶之上一个真内部节点」；拒绝按层级判，不看条目）。
fn grow_the_central_mapping_into_two_levels(built: &mut FirstTransactionImage) {
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
    let internal_root = build_index_node(
        leaf.tree,
        1,
        key_bytes,
        &leaf.smallest_key,
        &leaf.smallest_key,
        leaf.birth_txg,
        &built.root.filesystem_identifier,
        leaf.instance,
        leaf.birth_sequence,
        u16::try_from(internal_entry_bytes).expect("113"),
        &[internal_entry],
    );
    write_back_the_central_mapping_root_and_reseal_the_root(built, &internal_root);
}

/// C483 ①：映射长成两层 ⇒ 打开挂载态被拒，错误成员说清是多层映射、第一版不支持；读完映射树根就停，
/// 树表、映射那片叶、任何 extent / inode 单元都不读（块层只数到一次 16 KiB 读）。整池只读挂载同样挂不上。
/// 判别力：去掉那道层级判，内部条目被当 55 字节映射条目解下去，挂载态照样打开——这里由绿转红。
#[test]
fn central_mapping_grown_into_two_levels_is_refused_at_open_without_reading_past_its_root() {
    let mut built = first_transaction_image("c483-two-level-mapping");
    grow_the_central_mapping_into_two_levels(&mut built);
    let refusal =
        OpenPoolForReadFailure::CentralMappingWithMoreThanOneLevelIsNotSupportedInTheFirstVersion {
            mapping_tree: built.root.mapping_root.head.birth_tree,
            mapping_root_level: 1,
        };

    let counting = ReadCountingPoolReader::new(
        &built.image,
        JournalRingRegion::starting_at_the_standard_slot(JOURNAL_RING_DEFAULT_BYTES),
    );
    let failure = open_pool_for_read(&counting, &built.root)
        .err()
        .expect("映射根不是根兼叶，打开挂载态要被拒");
    assert_eq!(failure, refusal, "错误成员说清是多层映射、第一版不支持");
    let tally = counting.tally();
    assert_eq!(
        (
            tally.reads,
            tally.reads_of_a_whole_index_node,
            tally.reads_of_a_whole_unit
        ),
        (1, 1, 0),
        "只读了映射树根这一个 16 KiB 节点（第一条位置条目就中）：映射之外一次读都不发"
    );

    assert_eq!(
        mount_read_only(&built.image).err(),
        Some(MountReadOnlyFailure::Open(refusal)),
        "整池只读挂载：择根、扫环之后沿根打开挂载态，同一个拒绝"
    );
}
