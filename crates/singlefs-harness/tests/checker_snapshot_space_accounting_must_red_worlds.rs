//! 池级 checker 对只读快照与 deadlist 那几条的必红世界（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S1、S4、S5）：
//! I-3.7（活快照可枚举）——树表里不带销毁中记号的码 14 条目就是全部活快照，回退候选集里 txg 相接的两版之间只经建快照多出来、
//! 只经写销毁意图与最后一批少下去；I-3.5（引用区间的精确性） ④ 与逆向——活头引用的块不许在 deadlist 里，deadlist 里每一条都被一个活快照钉着
//! （或在未完成销毁意图的待放集里）；I-3.6（deadlist 紧界）。
//!
//! 合法镜像从写路径来：第一个文件之后经 `transaction::publish_create_snapshot` 连建三个快照（txg 4、5、6）。必红世界只改最新那条根
//! （txg 6）的树表单元（两份）、重封、把新的整单元校验和补进根记录再写回根槽、journal 记录的新根段照抄（同
//! `pool_checker_judges_the_snapshot_entries_of_the_tree_table.rs` 的改法）。deadlist 那两个世界另在单元区远处（槽 60000）两块盘各写一片
//! deadlist 叶、让 deadlist 树条目的根指针指它：那一片不在分配记录里，I-3.1 一类跟着红，这几个用例只断言点名的那一条红、红在哪一格。
//! deadlist 写者（里程碑四丙 2）落地之前，「活快照钉着、活头不引用」的块在写路径上造不出来，I-3.5 ③、I-3.6 与 I-3.9 从 deadlist 放掉的那一格的
//! 必红世界在池级 checker 的单测里（`crates/singlefs-checker/src/walk.rs` 的测试模块）。
mod common;
mod common_pointer_head_corruption;

use common::{
    build_pool, copy_each_root_into_the_new_root_segment_of_its_journal_records, parameters,
    publish_overwrite_in_process, BuiltPool, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_checker::image::{
    InvariantVerdict, DEADLIST_ENTRIES_ARE_BORN_NO_LATER_THAN_THE_PREVIOUS_SNAPSHOT_OF_THEIR_SIDE,
    LIVE_SNAPSHOTS_ARE_ENUMERABLE, REFERENCE_INTERVALS_ARE_EXACT,
    SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
    TreeIdentifier,
};
use singlefs_core::allocator::ALLOCATION_RECORD_RELEASED_FLAG;
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::pointer::BirthSequence;
use singlefs_core::records::{
    TREE_KIND_ALLOCATION, TREE_KIND_DEADLIST, TREE_KIND_INODE, TREE_KIND_SNAPSHOT_EXTENT_ROOT,
    TREE_KIND_SNAPSHOT_INODE_ROOT,
};
use singlefs_core::recovery::{
    choose_system_configuration, every_root_ring_slot, read_root_ring_slot, RootRingSlotReading,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::slot_offset;
use singlefs_core::transaction::{
    publish_create_snapshot, publish_snapshot_destruction_intent,
    walk_the_deletion_queue_until_it_is_empty, PoolWriter, TransactionOutput, TransactionUnit,
};
use singlefs_core::unit::{build_index_node, parse_index_node, seal_header_checksum};
use singlefs_core::unlink::UnlinkBatchSize;
use singlefs_format::{MAPPING_KEY_BYTES, NODE_BYTES, SLOT_BYTES};
use singlefs_harness::memory_pool::MemoryPool;

/// 树表条目字段表（D8（核心索引结构） 已定项 8）：种类在偏移 10、根指针 14..100、`previous_snapshot_txg` 100、诞生 txg 108、头 ID 116。
const TREE_TABLE_ENTRY_KIND_OFFSET: usize = 10;
const TREE_TABLE_ENTRY_ROOT_POINTER: std::ops::Range<usize> = 14..100;
const TREE_TABLE_ENTRY_PREVIOUS_SNAPSHOT_TXG_OFFSET: usize = 100;
const TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET: usize = 108;
const TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET: usize = 116;
/// 三个快照各建在第一个文件（txg 3）之后的一次独立发布里：S.txg 依次 4、5、6，最新那条根就是 txg 6。
const SNAPSHOTS_CREATED: u64 = 3;
const NEWEST_TXG: u64 = 6;
/// 手写那一片 deadlist 叶的落点：单元区远处一个这几个池都不会分到的槽，两块盘同槽各一份。
const DEADLIST_LEAF_SLOT: u64 = 60_000;
/// 那一片叶的出生序号：这次发布里没发过的一个号（只要不与同一 (实例, txg) 的别的码 2 / 码 3 单元重号）。
const DEADLIST_LEAF_BIRTH_SEQUENCE: u32 = 4000;
/// deadlist 条目 43 字节：头的树 ID 8 + 纪元 8 + 中央映射 key 27（D5（快照 / 空间记账机制） 已定项 12）。
const DEADLIST_ENTRY_BYTES: usize = 8 + 8 + 27;
/// 中央映射叶条目 55 字节（key 27 + 位置条目 14 × 2，D19（块指针的结构与宽度预算） 已定项 10）里 key 的类标签：码 1 数据单元是 1。
const MAPPING_KEY_UNIT_CLASS_OF_A_DATA_UNIT: u8 = 1;

fn node_bytes() -> usize {
    usize::try_from(NODE_BYTES).expect("16384")
}

fn read_u64_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8 字节"))
}

fn kind_of_entry(entry: &[u8]) -> u16 {
    u16::from_le_bytes(
        entry[TREE_TABLE_ENTRY_KIND_OFFSET..TREE_TABLE_ENTRY_KIND_OFFSET + 2]
            .try_into()
            .expect("2 字节"),
    )
}

/// 第一个文件之后连建三个快照的池，录制流施加到内存盘上的镜像。
fn image_with_three_snapshots(tag: &str) -> MemoryPool {
    let mut pool = build_pool(tag);
    for _snapshot in 0..SNAPSHOTS_CREATED {
        let publish_parameters = parameters();
        let previous = pool.output.clone();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        pool.output = publish_create_snapshot(
            &mut writer,
            &mut pool.allocator,
            &previous,
            InstanceGeneration(1),
        )
        .expect("建快照");
    }
    assert_eq!(
        pool.output.root.checkpoint_txg,
        CheckpointTxg(NEWEST_TXG),
        "第一个文件在 txg 3，三次建快照各一次独立发布"
    );
    pool.memory_pool()
}

/// 根环里最新那条根与它的槽。
fn newest_root(image: &MemoryPool) -> (singlefs_core::root_ring::RootRingSlot, RootRecord) {
    let system_configuration = choose_system_configuration(image).expect("系统配置择得到");
    let sizes = system_configuration.immutable.sizes;
    every_root_ring_slot(&sizes)
        .into_iter()
        .filter_map(|ring_slot| {
            match read_root_ring_slot(
                image,
                &system_configuration.immutable.region_devices,
                &sizes,
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

/// 最新那条根的树表条目（盘上原样、按盘上次序）。
fn entries_of_the_newest_tree_table(image: &MemoryPool) -> Vec<Vec<u8>> {
    let (_, root) = newest_root(image);
    let location = root.tree_table.locations[0];
    let tree_table = image
        .devices
        .get(&location.device)
        .expect("池里有这块盘")
        .read(location.slot.to_device_offset(), node_bytes());
    parse_index_node(&tree_table)
        .expect("树表单元解得开")
        .entries
}

/// 码 2 节点改过之后重封：载荷 CRC（罩 [86 + 2k, 16384)，住 76 + 2k）与头校验和（罩 [0, 86 + 2k)）。
fn reseal_index_node(node: &mut [u8]) {
    let key_width = usize::from(node[51]);
    let header_end = 86 + 2 * key_width;
    let payload_checksum = crc32_castagnoli(&node[header_end..]);
    node[header_end - 10..header_end - 6].copy_from_slice(&payload_checksum.to_le_bytes());
    seal_header_checksum(node, header_end);
}

/// 最新那条根的树表单元换成按 `entries` 照原头重建的一片（key 区间取首末两条的树 ID），两份都写回原槽、重封，
/// 新的整单元校验和补进根记录里树表指针的两条位置条目、根写回它的槽，同 (实例, txg) 的 journal 记录的新根段照抄改过的根（I-8.10）。
fn rewrite_the_entries_of_the_newest_tree_table(image: &mut MemoryPool, entries: &[Vec<u8>]) {
    let system_configuration = choose_system_configuration(&*image).expect("系统配置择得到");
    let sizes = system_configuration.immutable.sizes;
    let region_devices = system_configuration.immutable.region_devices;
    let (ring_slot, mut root) = newest_root(image);
    let first_location = root.tree_table.locations[0];
    let header = parse_index_node(
        &image
            .devices
            .get(&first_location.device)
            .expect("池里有这块盘")
            .read(first_location.slot.to_device_offset(), node_bytes()),
    )
    .expect("树表单元解得开");
    let key_of = |entry: &Vec<u8>| entry[..8].to_vec();
    let mut tree_table = build_index_node(
        header.tree,
        header.level,
        header.key_width,
        &key_of(entries.first().expect("树表有条目")),
        &key_of(entries.last().expect("树表有条目")),
        header.birth_txg,
        &parameters().filesystem_identifier,
        header.instance,
        header.birth_sequence,
        u16::try_from(header.entry_width).expect("条目宽 200"),
        entries,
    );
    reseal_index_node(&mut tree_table);
    let unit_checksum = crc32_castagnoli(&tree_table);
    for location in &mut root.tree_table.locations {
        image
            .devices
            .get_mut(&location.device)
            .expect("池里有这块盘")
            .write(location.slot.to_device_offset(), &tree_table);
        location.unit_checksum = unit_checksum;
    }
    let root_device = region_devices[usize::try_from(ring_slot.region).expect("区域号")];
    let root_slot_bytes = usize::try_from(sizes.physical_block_size).expect("根槽宽");
    image
        .devices
        .get_mut(&root_device)
        .expect("池里有这块盘")
        .write(
            slot_offset(ring_slot, sizes.fixed_structure_slot_spacing),
            &root.to_slot(root_slot_bytes),
        );
    copy_each_root_into_the_new_root_segment_of_its_journal_records(image);
}

/// 一条判定。
fn verdict(image: &MemoryPool, invariant: &str) -> InvariantVerdict {
    check_pool_image(image)
        .into_iter()
        .find(|(candidate, _)| *candidate == invariant)
        .expect("清单里有这一条")
        .1
}

fn assert_violated_mentioning(image: &MemoryPool, invariant: &str, fragment: &str, what: &str) {
    match verdict(image, invariant) {
        InvariantVerdict::Violated(detail) => assert!(
            detail.contains(fragment),
            "{what}：{invariant} 的违例说明里要有「{fragment}」，实际 {detail}"
        ),
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::PartiallyJudged(_)) => {
            panic!("{what}：{invariant} 要判红，实际 {other:?}")
        }
    }
}

/// 写路径建出来的三个快照：一条违例都没有，I-3.7 与 I-3.6 都判到且成立（每一版的活快照都是那一版之前的捕获、相接各版只经建快照多一条；
/// deadlist 一条都没有）；三个快照都只捕获活头还引用着的块，I-3.5 ③ ④ 没有对象。
#[test]
fn the_pool_checker_holds_the_snapshot_and_deadlist_invariants_on_a_pool_with_three_snapshots() {
    let image = image_with_three_snapshots("snapshot-space-accounting-legal");
    let verdicts = check_pool_image(&image);
    let violated: Vec<&(&str, InvariantVerdict)> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .collect();
    assert_eq!(
        violated,
        Vec::<&(&str, InvariantVerdict)>::new(),
        "一条违例都没有"
    );
    for invariant in [
        LIVE_SNAPSHOTS_ARE_ENUMERABLE,
        DEADLIST_ENTRIES_ARE_BORN_NO_LATER_THAN_THE_PREVIOUS_SNAPSHOT_OF_THEIR_SIDE,
    ] {
        assert_eq!(
            verdict(&image, invariant),
            InvariantVerdict::Holds,
            "{invariant} 判到且成立"
        );
    }
}

/// 必红世界（I-3.7 少一条）：最新那一版（txg 6）的树表去掉第二个快照（S.txg 5）那一对：txg 5 那一版它还活着、没带销毁中记号，
/// txg 6 那一版它就没了。
#[test]
fn removing_the_second_snapshot_from_the_newest_tree_table_reddens_the_enumerable_invariant() {
    let mut image = image_with_three_snapshots("snapshot-space-accounting-one-fewer");
    let entries = entries_of_the_newest_tree_table(&image);
    let second_snapshot_inode_root_tree = entries
        .iter()
        .filter(|entry| kind_of_entry(entry) == TREE_KIND_SNAPSHOT_INODE_ROOT)
        .find(|entry| read_u64_at(entry, TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET) == NEWEST_TXG - 1)
        .map(|entry| read_u64_at(entry, 0))
        .expect("txg 5 建的那个快照在最新那一版的树表里");
    let entries_without_the_second_snapshot: Vec<Vec<u8>> = entries
        .into_iter()
        .filter(|entry| {
            let is_its_inode_root = kind_of_entry(entry) == TREE_KIND_SNAPSHOT_INODE_ROOT
                && read_u64_at(entry, 0) == second_snapshot_inode_root_tree;
            let is_its_extent_root = kind_of_entry(entry) == TREE_KIND_SNAPSHOT_EXTENT_ROOT
                && read_u64_at(entry, TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET)
                    == second_snapshot_inode_root_tree;
            !(is_its_inode_root || is_its_extent_root)
        })
        .collect();
    rewrite_the_entries_of_the_newest_tree_table(&mut image, &entries_without_the_second_snapshot);
    assert_violated_mentioning(
        &image,
        LIVE_SNAPSHOTS_ARE_ENUMERABLE,
        "少一条",
        "第二个快照没经销毁意图就从最新那一版的树表里没了",
    );
}

/// 必红世界（I-3.7 多一条）：最新那一版（txg 6）的树表末尾加一对快照条目，抄第一个快照那一对、树 ID 换成比表里都大的两个号、
/// 诞生 txg 写成第一个文件那一版的 3：txg 5 那一版没有它，txg 6 那一版多出来、诞生 txg 又不是 6。
#[test]
fn adding_a_snapshot_pair_with_an_older_txg_to_the_newest_tree_table_reddens_the_enumerable_invariant(
) {
    let mut image = image_with_three_snapshots("snapshot-space-accounting-one-more");
    let mut entries = entries_of_the_newest_tree_table(&image);
    let first_snapshot_inode_root = entries
        .iter()
        .filter(|entry| kind_of_entry(entry) == TREE_KIND_SNAPSHOT_INODE_ROOT)
        .min_by_key(|entry| read_u64_at(entry, TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET))
        .cloned()
        .expect("最新那一版的树表里有快照");
    let first_snapshot_tree = read_u64_at(&first_snapshot_inode_root, 0);
    let first_snapshot_extent_root = entries
        .iter()
        .find(|entry| {
            kind_of_entry(entry) == TREE_KIND_SNAPSHOT_EXTENT_ROOT
                && read_u64_at(entry, TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET)
                    == first_snapshot_tree
        })
        .cloned()
        .expect("第一个快照的码 15 配着它");
    let largest_tree_in_the_table = entries
        .iter()
        .map(|entry| read_u64_at(entry, 0))
        .max()
        .expect("树表有条目");
    let fabricated_tree = largest_tree_in_the_table + 1;
    let fabricated_txg: u64 = 3;
    let mut fabricated_inode_root = first_snapshot_inode_root;
    fabricated_inode_root[..8].copy_from_slice(&fabricated_tree.to_le_bytes());
    fabricated_inode_root[TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET..TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET + 8]
        .copy_from_slice(&fabricated_txg.to_le_bytes());
    let mut fabricated_extent_root = first_snapshot_extent_root;
    fabricated_extent_root[..8].copy_from_slice(&(fabricated_tree + 1).to_le_bytes());
    fabricated_extent_root
        [TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET..TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET + 8]
        .copy_from_slice(&fabricated_txg.to_le_bytes());
    fabricated_extent_root
        [TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET..TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET + 8]
        .copy_from_slice(&fabricated_tree.to_le_bytes());
    entries.push(fabricated_inode_root);
    entries.push(fabricated_extent_root);
    rewrite_the_entries_of_the_newest_tree_table(&mut image, &entries);
    assert_violated_mentioning(
        &image,
        LIVE_SNAPSHOTS_ARE_ENUMERABLE,
        "多一条",
        "最新那一版多出一个诞生 txg 是 3 的快照",
    );
}

/// 一条码 2 指针（86 字节，D19（块指针的结构与宽度预算） 已定项 11 的偏移表）：头部加密与压缩那几段与 extent 偏移恒 0、出生树 34、出生 txg 42，
/// 两条位置条目（盘 0、盘 1 同槽，设备 4 + 槽 6 + 整单元校验和 4）在 50、64，实例代号 78、出生序号 82。
fn node_pointer_bytes(
    birth_tree: u64,
    birth_txg: u64,
    slot: u64,
    unit_checksum: u32,
    instance: u32,
    birth_sequence: u32,
) -> Vec<u8> {
    let mut pointer = vec![0u8; 86];
    pointer[34..42].copy_from_slice(&birth_tree.to_le_bytes());
    pointer[42..50].copy_from_slice(&birth_txg.to_le_bytes());
    for (device, location_offset) in [(0u32, 50usize), (1, 64)] {
        pointer[location_offset..location_offset + 4].copy_from_slice(&device.to_le_bytes());
        pointer[location_offset + 4..location_offset + 10]
            .copy_from_slice(&slot.to_le_bytes()[..6]);
        pointer[location_offset + 10..location_offset + 14]
            .copy_from_slice(&unit_checksum.to_le_bytes());
    }
    pointer[78..82].copy_from_slice(&instance.to_le_bytes());
    pointer[82..86].copy_from_slice(&birth_sequence.to_le_bytes());
    pointer
}

/// 最新那一版的 deadlist 换成只装 `deadlist_entries` 的一片叶（key 宽、条目宽都是 43，按 key 升序给）：叶写在槽 60000（两块盘各一份），
/// 出生 (deadlist 树, txg 6)、实例 1；树表里 deadlist 树那一条的根指针指它，树表与根照 [`rewrite_the_entries_of_the_newest_tree_table`] 补齐。
fn image_whose_newest_deadlist_holds(
    mut image: MemoryPool,
    deadlist_entries: &[Vec<u8>],
) -> MemoryPool {
    let mut entries = entries_of_the_newest_tree_table(&image);
    let deadlist_entry_index = entries
        .iter()
        .position(|entry| kind_of_entry(entry) == TREE_KIND_DEADLIST)
        .expect("树表里有 deadlist 树那一条");
    let deadlist_tree = read_u64_at(&entries[deadlist_entry_index], 0);
    let mut leaf = build_index_node(
        TreeIdentifier(deadlist_tree),
        0,
        DEADLIST_ENTRY_BYTES,
        &deadlist_entries.first().expect("至少一条")[..DEADLIST_ENTRY_BYTES],
        &deadlist_entries.last().expect("至少一条")[..DEADLIST_ENTRY_BYTES],
        CheckpointTxg(NEWEST_TXG),
        &parameters().filesystem_identifier,
        InstanceGeneration(1),
        BirthSequence(DEADLIST_LEAF_BIRTH_SEQUENCE),
        u16::try_from(DEADLIST_ENTRY_BYTES).expect("43"),
        deadlist_entries,
    );
    reseal_index_node(&mut leaf);
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        image
            .devices
            .get_mut(&device)
            .expect("池里有这块盘")
            .write(DeviceOffsetInBytes(DEADLIST_LEAF_SLOT * SLOT_BYTES), &leaf);
    }
    entries[deadlist_entry_index][TREE_TABLE_ENTRY_ROOT_POINTER].copy_from_slice(
        &node_pointer_bytes(
            deadlist_tree,
            NEWEST_TXG,
            DEADLIST_LEAF_SLOT,
            crc32_castagnoli(&leaf),
            1,
            DEADLIST_LEAF_BIRTH_SEQUENCE,
        ),
    );
    rewrite_the_entries_of_the_newest_tree_table(&mut image, &entries);
    image
}

/// 最新那一版的活头：inode 树的树 ID 与它的 `previous_snapshot_txg`（最后一个快照的 S.txg，也就是这时进 deadlist 的条目的纪元）。
fn head_inode_tree_and_its_previous_snapshot_txg(image: &MemoryPool) -> (u64, u64) {
    let entries = entries_of_the_newest_tree_table(image);
    let head = entries
        .iter()
        .find(|entry| kind_of_entry(entry) == TREE_KIND_INODE)
        .expect("树表里有活头 inode 树那一条");
    (
        read_u64_at(head, 0),
        read_u64_at(head, TREE_TABLE_ENTRY_PREVIOUS_SNAPSHOT_TXG_OFFSET),
    )
}

fn deadlist_entry(head: u64, epoch: u64, mapping_key: &[u8]) -> Vec<u8> {
    [
        head.to_le_bytes().to_vec(),
        epoch.to_le_bytes().to_vec(),
        mapping_key.to_vec(),
    ]
    .concat()
}

/// 必红世界（I-3.5 ④）：把一个活头还引用的块放进 deadlist——最新那一版中央映射树里一条码 1 数据单元的 key，挂在活头那一份（纪元 = 活头的前驱）。
#[test]
fn a_deadlist_pinning_a_data_unit_the_head_still_references_reddens_the_reference_intervals() {
    let image = image_with_three_snapshots("snapshot-space-accounting-head-referenced-in-deadlist");
    let (_, root) = newest_root(&image);
    let mapping_location = root.mapping_root.locations[0];
    let mapping_root = parse_index_node(
        &image
            .devices
            .get(&mapping_location.device)
            .expect("池里有这块盘")
            .read(mapping_location.slot.to_device_offset(), node_bytes()),
    )
    .expect("中央映射树的根解得开");
    assert_eq!(mapping_root.level, 0, "这几个池的映射树只有一层");
    let mapping_key_bytes = usize::try_from(MAPPING_KEY_BYTES).expect("27");
    let data_unit_mapping_key = mapping_root
        .entries
        .iter()
        .find(|entry| entry[0] == MAPPING_KEY_UNIT_CLASS_OF_A_DATA_UNIT)
        .map(|entry| entry[..mapping_key_bytes].to_vec())
        .expect("第一个文件的数据单元在映射里");
    let (head, epoch) = head_inode_tree_and_its_previous_snapshot_txg(&image);
    let image = image_whose_newest_deadlist_holds(
        image,
        &[deadlist_entry(head, epoch, &data_unit_mapping_key)],
    );
    assert_violated_mentioning(
        &image,
        REFERENCE_INTERVALS_ARE_EXACT,
        "活头还引用着",
        "活头引用的数据单元进了 deadlist",
    );
}

/// 必红世界（I-3.5 逆向）：deadlist 里一条钉着谁都不引用的单元——一把合成的码 1 key（出生 txg 2、事务号 999，池里没有这个单元），
/// 挂在活头那一份；没有销毁中的快照，它也不在待放集里。birth 2 不大于活头的前驱，I-3.6 不因它红。
#[test]
fn a_deadlist_pinning_a_unit_no_live_snapshot_references_reddens_the_reverse_direction() {
    let image = image_with_three_snapshots("snapshot-space-accounting-unpinned-in-deadlist");
    let (head, epoch) = head_inode_tree_and_its_previous_snapshot_txg(&image);
    let mapping_key_bytes = usize::try_from(MAPPING_KEY_BYTES).expect("27");
    let mut unit_nobody_references = vec![0u8; mapping_key_bytes];
    unit_nobody_references[0] = MAPPING_KEY_UNIT_CLASS_OF_A_DATA_UNIT;
    unit_nobody_references[1..9].copy_from_slice(&65536u64.to_le_bytes());
    unit_nobody_references[9..17].copy_from_slice(&2u64.to_le_bytes());
    unit_nobody_references[17..21].copy_from_slice(&1u32.to_le_bytes());
    unit_nobody_references[21..27].copy_from_slice(&999u64.to_le_bytes()[..6]);
    let image = image_whose_newest_deadlist_holds(
        image,
        &[deadlist_entry(head, epoch, &unit_nobody_references)],
    );
    assert_violated_mentioning(
        &image,
        REFERENCE_INTERVALS_ARE_EXACT,
        "逆向",
        "没有活快照钉着的单元进了 deadlist",
    );
    assert!(
        !matches!(
            verdict(
                &image,
                DEADLIST_ENTRIES_ARE_BORN_NO_LATER_THAN_THE_PREVIOUS_SNAPSHOT_OF_THEIR_SIDE
            ),
            InvariantVerdict::Violated(_)
        ),
        "birth 2 不大于活头的前驱：I-3.6 不红"
    );
}

// ───── 写路径写出的 deadlist（丙 2 主体）上的合法镜像与必红世界：快照下覆盖写、删快照，快照两棵树的走读 ─────

/// 建一个快照之后覆盖写一次：活头换下的数据单元、extent 树根、inode 叶容器与根都生在建快照之前，进 deadlist（头 = 活头 inode 树、
/// 纪元 = S.txg）、映射条目留存，只被快照的两棵树与 deadlist 钉着。交回池（接在覆盖写那一版上）与第一个文件那个数据单元的槽。
fn pool_with_a_snapshot_and_an_overwrite_under_it(tag: &str) -> (BuiltPool, SlotNumber) {
    let mut pool = build_pool(tag);
    let data_unit_of_the_first_file = pool
        .output
        .units
        .iter()
        .find(|unit| matches!(unit.identity, TransactionUnit::Data(_)))
        .expect("第一个文件有一个数据单元")
        .slot;
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        pool.output = publish_create_snapshot(
            &mut writer,
            &mut pool.allocator,
            &previous,
            InstanceGeneration(1),
        )
        .expect("建快照");
    }
    overwrite_and_advance(&mut pool, 7);
    (pool, data_unit_of_the_first_file)
}

/// 同一个进程里接着 `pool.output` 覆盖写一次（与第一个文件不同的一份 2700 字节），池接到那一版上，交回那一版。
fn overwrite_and_advance(pool: &mut BuiltPool, seed: u32) -> TransactionOutput {
    let content: Vec<u8> = (0..2700_u32)
        .map(|index| u8::try_from((index * 13 + seed) % 239).expect("小于 256"))
        .collect();
    let previous = pool.output.clone();
    let overwritten = publish_overwrite_in_process(
        pool,
        &previous,
        &content,
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("覆盖写");
    pool.output = overwritten.clone();
    overwritten
}

/// 删最新那一版里唯一那个快照的意图那次发布，池接到那一版上。
fn publish_the_destruction_intent_and_advance(pool: &mut BuiltPool) {
    let snapshot = pool
        .output
        .tree_table_entries
        .iter()
        .find(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
        .expect("树表里有一个快照")
        .tree;
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    pool.output = publish_snapshot_destruction_intent(
        &mut writer,
        &mut pool.allocator,
        &previous,
        snapshot,
        InstanceGeneration(1),
    )
    .expect("删快照的意图");
}

/// 走空（销毁中的快照）到底：产品批大小一批放得完四条，就是最后一批，同一次发布删掉那一对条目。池接到最后一版上。
fn walk_the_destruction_to_the_end_and_advance(pool: &mut BuiltPool) {
    let publish_parameters = parameters();
    let mut current = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let walk = walk_the_deletion_queue_until_it_is_empty(
        &mut writer,
        &mut pool.allocator,
        &mut current,
        UnlinkBatchSize::OneExtentLowerLeaf,
        InstanceGeneration(1),
    );
    walk.stopped_by.expect("走空走到底");
    pool.output = current;
}

fn violations(image: &MemoryPool) -> Vec<(&'static str, String)> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, found)| match found {
            InvariantVerdict::Violated(detail) => Some((invariant, detail)),
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect()
}

/// 一个码 2 节点换成同一个头（树、层级、key 宽、出生身份、条目宽）、给定的 key 区间与条目，重封。
fn rebuilt_node(
    node: &[u8],
    smallest_key: &[u8],
    largest_key: &[u8],
    entries: &[Vec<u8>],
) -> Vec<u8> {
    let header = parse_index_node(node).expect("节点解得开");
    let mut rebuilt = build_index_node(
        header.tree,
        header.level,
        header.key_width,
        smallest_key,
        largest_key,
        header.birth_txg,
        &parameters().filesystem_identifier,
        header.instance,
        header.birth_sequence,
        u16::try_from(header.entry_width).expect("条目宽装得进 u16"),
        entries,
    );
    reseal_index_node(&mut rebuilt);
    rebuilt
}

/// deadlist key 43 字节按字段 [8, 8, 1, 8, 8, 4, 6] 逐段读成小端无符号数（盘上的序就是逐段比，`KEY_SCHEMA_DEADLIST`）。
fn deadlist_key_fields(entry: &[u8]) -> Vec<u64> {
    let mut fields = Vec::new();
    let mut offset = 0;
    for width in [8usize, 8, 1, 8, 8, 4, 6] {
        let mut wide = [0u8; 8];
        wide[..width].copy_from_slice(&entry[offset..offset + width]);
        fields.push(u64::from_le_bytes(wide));
        offset += width;
    }
    fields
}

/// 最新那一版的 deadlist 叶（单层）换成 `change` 改过的条目（按 key 升序重排、key 区间贴紧首末两条），重封、新校验和沿引用链补到根槽
/// （deadlist 树的节点进映射：映射条目里的那一条一起补）。
fn change_the_newest_deadlist_leaf(image: &mut MemoryPool, change: impl FnOnce(&mut Vec<Vec<u8>>)) {
    let leaf_slot = common_pointer_head_corruption::the_slot_of(
        &common_pointer_head_corruption::tree_table_entry_of_kind(image, TREE_KIND_DEADLIST).root,
    );
    let leaf = common_pointer_head_corruption::unit_bytes_at(image, leaf_slot, node_bytes());
    let header = parse_index_node(&leaf).expect("deadlist 叶解得开");
    assert_eq!(header.level, 0, "四条条目，deadlist 树只有一层");
    let mut entries = header.entries;
    change(&mut entries);
    entries.sort_by_key(|entry| deadlist_key_fields(entry));
    let rebuilt = rebuilt_node(
        &leaf,
        &entries.first().expect("至少一条")[..DEADLIST_ENTRY_BYTES],
        &entries.last().expect("至少一条")[..DEADLIST_ENTRY_BYTES],
        &entries,
    );
    common_pointer_head_corruption::replace_the_unit_and_propagate_its_checksum(
        image, leaf_slot, &rebuilt,
    );
}

/// 快照下覆盖写之后的合法镜像（写路径写出的 deadlist 与留存的映射条目）：一条违例都没有。I-1.13 判到且成立——遍历方向那一张中央映射
/// 不收 deadlist 点名的、留存的那四条（它们不是当前版本，扫描重建只重建当前版本）；快照两棵树里只被快照引用的那几个单元照判结构、
/// 与活头共享的不重报（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3）。
#[test]
fn the_pool_checker_holds_every_invariant_after_an_overwrite_under_a_snapshot() {
    let (pool, _) = pool_with_a_snapshot_and_an_overwrite_under_it("snapshot-overwrite-legal");
    let image = pool.memory_pool();
    assert_eq!(violations(&image), [], "快照下覆盖写之后一条违例都没有");
    assert_eq!(
        verdict(&image, SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX),
        InvariantVerdict::Holds,
        "I-1.13 判到且成立：留存的映射条目不进比对"
    );
}

/// 必红世界（走读罩上快照的两棵树）：快照下覆盖写之后，旧的 inode 树根只剩快照的码 14 条目与 deadlist 钉着（活头已换新根）；把它头里的
/// 最小 key 改掉（区间不再贴紧首末条目）、重封、新校验和沿引用链补上。最新那一版先走，它的活头树里没有这个节点，I-1.1 第一处违例就是
/// 快照那棵树（种类 14 那条条目）走出来的。主走读不走快照的树时，这一处要等更早那几版把它当活头的根走到才红、点名的是种类 2。
#[test]
fn a_node_only_the_snapshot_tree_references_with_a_broken_key_range_reddens_the_self_description_invariant(
) {
    let (pool, _) = pool_with_a_snapshot_and_an_overwrite_under_it("snapshot-tree-broken-node");
    let mut image = pool.memory_pool();
    let snapshot_root = common_pointer_head_corruption::tree_table_entry_of_kind(
        &image,
        TREE_KIND_SNAPSHOT_INODE_ROOT,
    )
    .root;
    let head_root =
        common_pointer_head_corruption::tree_table_entry_of_kind(&image, TREE_KIND_INODE).root;
    let snapshot_root_slot = common_pointer_head_corruption::the_slot_of(&snapshot_root);
    assert_ne!(
        snapshot_root_slot,
        common_pointer_head_corruption::the_slot_of(&head_root),
        "覆盖写之后活头换了新的 inode 树根，旧的只剩快照引用"
    );
    let node =
        common_pointer_head_corruption::unit_bytes_at(&image, snapshot_root_slot, node_bytes());
    let header = parse_index_node(&node).expect("旧的 inode 树根解得开");
    let wrong_smallest_key = vec![0xffu8; header.key_width];
    let rebuilt = rebuilt_node(
        &node,
        &wrong_smallest_key,
        &header.largest_key,
        &header.entries,
    );
    common_pointer_head_corruption::replace_the_unit_and_propagate_its_checksum(
        &mut image,
        snapshot_root_slot,
        &rebuilt,
    );
    assert_violated_mentioning(
        &image,
        "I-1.1",
        &format!("（种类 {TREE_KIND_SNAPSHOT_INODE_ROOT}）的根"),
        "快照那棵树里只被快照引用的节点结构坏了",
    );
}

/// I-9.25 在删快照途中（判决 S4、S5）：写意图那一版，被删的快照（码 14 带销毁中记号）自己的前驱不动，活头的前驱已换成它的前驱 P；
/// 前驱链跳过带记号的那一条，这一版成立。最后一批删掉那一对之后，环里还留着意图那条根，同样成立。两版上一条违例都没有。
#[test]
fn the_previous_snapshot_txg_holds_after_the_destruction_intent_and_after_the_last_batch() {
    let (mut pool, _) =
        pool_with_a_snapshot_and_an_overwrite_under_it("snapshot-destroy-previous-txg");
    publish_the_destruction_intent_and_advance(&mut pool);
    let after_the_intent = pool.memory_pool();
    assert_eq!(
        verdict(&after_the_intent, "I-9.25"),
        InvariantVerdict::Holds,
        "写意图那一版：前驱链跳过带销毁中记号的那一条"
    );
    assert_eq!(
        violations(&after_the_intent),
        [],
        "写意图那一版一条违例都没有"
    );
    walk_the_destruction_to_the_end_and_advance(&mut pool);
    let after_the_last_batch = pool.memory_pool();
    assert_eq!(
        verdict(&after_the_last_batch, "I-9.25"),
        InvariantVerdict::Holds,
        "最后一批之后：环里意图那条根照样成立"
    );
    assert_eq!(
        violations(&after_the_last_batch),
        [],
        "最后一批之后一条违例都没有"
    );
}

/// 必红世界（I-3.5 ③ 缺一条）：快照下覆盖写之后，deadlist 里钉着第一个文件那个数据单元的那一条删掉（叶重写、重封、补到根槽）。
/// 那个单元只剩快照引用、活头不引用，deadlist 里没有钉它的那一条。
#[test]
fn a_deadlist_missing_the_entry_of_a_unit_only_the_snapshot_references_reddens_the_reference_intervals(
) {
    let (pool, _) =
        pool_with_a_snapshot_and_an_overwrite_under_it("snapshot-deadlist-missing-entry");
    let mut image = pool.memory_pool();
    change_the_newest_deadlist_leaf(&mut image, |entries| {
        let before = entries.len();
        entries.retain(|entry| {
            entry[DEADLIST_ENTRY_BYTES - MAPPING_KEY_BYTES_IN_A_DEADLIST_ENTRY]
                != MAPPING_KEY_UNIT_CLASS_OF_A_DATA_UNIT
        });
        assert_eq!(
            entries.len() + 1,
            before,
            "deadlist 里恰好一条钉的是数据单元"
        );
    });
    assert_violated_mentioning(
        &image,
        REFERENCE_INTERVALS_ARE_EXACT,
        "（③）",
        "只被快照引用的数据单元在 deadlist 里没有那一条",
    );
}

/// deadlist 条目 43 字节里映射 key 那 27 字节（条目末尾那一段：头 8、纪元 8 之后）。
const MAPPING_KEY_BYTES_IN_A_DEADLIST_ENTRY: usize = 27;

/// 必红世界（I-3.6 该直接释放的块进了 deadlist）：建快照、覆盖写（txg 5 写出一个新数据单元，生在快照之后）、再覆盖写（那个单元生在活头的前驱之后，
/// 直接释放、不进 deadlist）；再手工把它挂进活头那一份（纪元 = 活头的前驱）。它的 birth 大于那一侧的前驱。
#[test]
fn a_unit_born_after_the_snapshot_put_on_the_deadlist_reddens_the_tight_bound() {
    let (mut pool, _) =
        pool_with_a_snapshot_and_an_overwrite_under_it("snapshot-deadlist-born-after");
    let data_unit_born_after_the_snapshot = pool
        .output
        .mapped_units
        .iter()
        .find(|(identity, _)| matches!(identity, TransactionUnit::Data(_)))
        .map(|(_, key)| key.clone())
        .expect("覆盖写那一版的数据单元在映射里");
    overwrite_and_advance(&mut pool, 11);
    let mut image = pool.memory_pool();
    let (head, epoch) = head_inode_tree_and_its_previous_snapshot_txg(&image);
    change_the_newest_deadlist_leaf(&mut image, |entries| {
        entries.push(deadlist_entry(
            head,
            epoch,
            &data_unit_born_after_the_snapshot,
        ));
    });
    assert_violated_mentioning(
        &image,
        DEADLIST_ENTRIES_ARE_BORN_NO_LATER_THAN_THE_PREVIOUS_SNAPSHOT_OF_THEIR_SIDE,
        "",
        "生在快照之后、该直接释放的数据单元进了 deadlist",
    );
}

/// 必红世界（I-3.9 从 deadlist 放掉的块）：快照下覆盖写、删快照（意图一次、最后一批一次）；最后一批放掉的第一个文件那个数据单元，
/// 最新那一版账里那条记录的释放代改成写意图那次的 txg。写意图那条根还经留存的映射条目引用着它，释放代该落在（意图 txg, 最后一批 txg]。
#[test]
fn a_block_released_from_the_deadlist_with_the_txg_of_the_destruction_intent_reddens_the_release_generation_invariant(
) {
    let (mut pool, data_unit_of_the_first_file) =
        pool_with_a_snapshot_and_an_overwrite_under_it("snapshot-deadlist-release-generation");
    publish_the_destruction_intent_and_advance(&mut pool);
    let intent_txg = pool.output.root.checkpoint_txg.0;
    walk_the_destruction_to_the_end_and_advance(&mut pool);
    let last_batch_txg = pool.output.root.checkpoint_txg.0;
    assert_eq!(
        last_batch_txg,
        intent_txg + 1,
        "四条一批放完：最后一批紧接着意图"
    );
    let mut image = pool.memory_pool();
    let allocation_root =
        common_pointer_head_corruption::tree_table_entry_of_kind(&image, TREE_KIND_ALLOCATION).root;
    let (leaf_slot, leaf) =
        common_pointer_head_corruption::allocation_record_tree_nodes_as_far_as_readable(
            &image,
            &allocation_root,
        )
        .into_iter()
        .map(|pointer| {
            let slot = common_pointer_head_corruption::the_slot_of(&pointer);
            (
                slot,
                common_pointer_head_corruption::unit_bytes_at(&image, slot, node_bytes()),
            )
        })
        .find(|(_, node)| {
            let header = parse_index_node(node).expect("分配记录树节点解得开");
            header.level == 0
                && header.entries.iter().any(|record| {
                    record[..4] == 0u32.to_le_bytes()
                        && record[4..10] == data_unit_of_the_first_file.0.to_le_bytes()[..6]
                })
        })
        .expect("最新那一版的账里有第一个文件那个数据单元在盘 0 上的记录");
    let header = parse_index_node(&leaf).expect("分配记录叶解得开");
    let mut records = header.entries.clone();
    let record = records
        .iter_mut()
        .find(|record| {
            record[..4] == 0u32.to_le_bytes()
                && record[4..10] == data_unit_of_the_first_file.0.to_le_bytes()[..6]
        })
        .expect("盘 0 上那条记录");
    let span_field = u16::from_le_bytes([record[10], record[11]]);
    assert_ne!(
        span_field & ALLOCATION_RECORD_RELEASED_FLAG,
        0,
        "最后一批放掉了它"
    );
    assert_eq!(
        read_u64_at(record, 12),
        last_batch_txg,
        "释放代是最后一批的 txg"
    );
    record[12..20].copy_from_slice(&intent_txg.to_le_bytes());
    let rebuilt = rebuilt_node(&leaf, &header.smallest_key, &header.largest_key, &records);
    common_pointer_head_corruption::replace_the_unit_and_propagate_its_checksum(
        &mut image, leaf_slot, &rebuilt,
    );
    assert_violated_mentioning(
        &image,
        "I-3.9",
        "",
        "从 deadlist 放掉的块的释放代落在写意图那一格",
    );
}
