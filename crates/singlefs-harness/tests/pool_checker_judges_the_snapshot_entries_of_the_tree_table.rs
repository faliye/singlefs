//! 池级 checker 对树表里只读快照条目（码 14 快照 inode 根、码 15 快照 extent 根）的三格（判决
//! `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S1「I-9.16 ③、I-9.27、I-9.25 各加 14、15 一格」）：
//! I-9.16 ③ 码 14 / 15 不按「每种至多一条」判、按头 ID 一一配对判；I-9.27 码 14 的头 ID 是同表活头 inode 树的树 ID、码 15 的是同表一条码 14 的树 ID；
//! I-9.25 码 14 按（诞生 txg, 树 ID）排成前驱链，第一条前驱 0、之后每条是前一条的诞生 txg，活头 inode 条目的前驱是最后一条的诞生 txg。
//!
//! 合法镜像从写路径来：第一个文件之后经 `transaction::publish_create_snapshot` 连建两个快照。必红世界只改最新那条根的树表单元（两份）、重封、
//! 把新的整单元校验和补进根记录再写回根槽、journal 记录的新根段照抄（同 `tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker.rs`
//! 的改法）：盘上每一道校验和都过，只剩点名的那一处不对。
mod common;

use std::collections::BTreeSet;

use common::{
    build_pool, copy_each_root_into_the_new_root_segment_of_its_journal_records, parameters,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::InstanceGeneration;
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::recovery::{
    choose_system_configuration, every_root_ring_slot, read_root_ring_slot, RootRingSlotReading,
};
use singlefs_core::root_ring::slot_offset;
use singlefs_core::transaction::{publish_create_snapshot, PoolWriter};
use singlefs_core::unit::seal_header_checksum;
use singlefs_format::NODE_BYTES;
use singlefs_harness::memory_pool::MemoryPool;

/// 树表条目里 `previous_snapshot_txg` 的偏移（树 ID 8 + 条目长度 2 + 种类 2 + flags 2 + 根指针 86，D8（核心索引结构） 已定项 8）。
const TREE_TABLE_ENTRY_PREVIOUS_SNAPSHOT_TXG_OFFSET: usize = 100;
/// 诞生 txg 那 8 字节的偏移（`previous_snapshot_txg` 之后）。
const TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET: usize = 108;
/// 头 ID 那 8 字节的偏移（诞生 txg 之后）。
const TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET: usize = 116;
/// 第一个文件那一版树表里 inode 树是第 1 条（extent 树第 0 条），十二条按发号次序排。
const LIVE_INODE_TREE_ENTRY_INDEX: usize = 1;
/// 两个快照之后十二条接着按树号升序排的四条：第一个快照的码 14、码 15，第二个快照的码 14、码 15。
const FIRST_SNAPSHOT_INODE_ROOT_ENTRY_INDEX: usize = 12;
const SECOND_SNAPSHOT_EXTENT_ROOT_ENTRY_INDEX: usize = 15;

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

fn field_of_entry(node: &[u8], index: usize, offset: usize) -> u64 {
    let entry = &node[entry_range_in_node(node, index)];
    u64::from_le_bytes(entry[offset..offset + 8].try_into().expect("8 字节"))
}

fn set_field_of_entry(node: &mut [u8], index: usize, offset: usize, value: u64) {
    let range = entry_range_in_node(node, index);
    node[range.start + offset..range.start + offset + 8].copy_from_slice(&value.to_le_bytes());
}

/// 码 2 节点改过之后重封：载荷 CRC（罩 [86 + 2k, 16384)，住 76 + 2k）与头校验和（罩 [0, 86 + 2k)）。
fn reseal_index_node(node: &mut [u8]) {
    let key_width = usize::from(node[51]);
    let header_end = 86 + 2 * key_width;
    let payload_checksum = crc32_castagnoli(&node[header_end..]);
    node[header_end - 10..header_end - 6].copy_from_slice(&payload_checksum.to_le_bytes());
    seal_header_checksum(node, header_end);
}

/// 第一个文件之后连建两个快照的池，录制流施加到内存盘上的镜像。
fn image_with_two_snapshots(tag: &str) -> MemoryPool {
    let mut pool = build_pool(tag);
    for _snapshot in 0..2 {
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
    pool.memory_pool()
}

/// 改最新那条根的树表单元（两份）、重封、把新的整单元校验和补进根记录里树表指针的两条位置条目、根写回它的槽，
/// 同 (实例, txg) 的 journal 记录的新根段照抄改过的根（I-8.10）。
fn rewrite_the_tree_table_of_the_newest_root(image: &mut MemoryPool, change: impl Fn(&mut [u8])) {
    let system_configuration = choose_system_configuration(&*image).expect("系统配置择得到");
    let sizes = system_configuration.immutable.sizes;
    let region_devices = system_configuration.immutable.region_devices;
    let (ring_slot, mut root) = every_root_ring_slot(&sizes)
        .into_iter()
        .filter_map(|ring_slot| {
            match read_root_ring_slot(
                &*image,
                &region_devices,
                &sizes,
                &system_configuration.immutable.filesystem_identifier,
                ring_slot,
            ) {
                RootRingSlotReading::SelfVerified(root) => Some((ring_slot, root)),
                RootRingSlotReading::Bad(_) => None,
            }
        })
        .max_by_key(|(_, root)| (root.checkpoint_txg, root.instance))
        .expect("根环里有自证过的根");
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    let first_location = root.tree_table.locations[0];
    let mut tree_table = image
        .devices
        .get(&first_location.device)
        .expect("池里有这块盘")
        .read(first_location.slot.to_device_offset(), node_bytes);
    change(&mut tree_table);
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

/// 判红的不变量（去重、按名排）。
fn violated_invariants(image: &MemoryPool) -> BTreeSet<&'static str> {
    check_pool_image(image)
        .into_iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(invariant, _)| invariant)
        .collect()
}

/// 写路径建出来的两个快照：`check_pool_image` 一条违例都没有（码 14 / 15 各两条不判「同一种两条」，配对、头 ID、前驱链都对得上）。
#[test]
fn the_pool_checker_finds_no_violation_on_a_pool_with_two_snapshots() {
    let image = image_with_two_snapshots("pool-checker-snapshots-legal");
    assert_eq!(
        violated_invariants(&image),
        BTreeSet::new(),
        "两个快照的合法镜像：一条违例都没有"
    );
}

/// 必红世界：第二个快照的码 15 头 ID 改成第一个快照的码 14 的树号——两条码 15 配同一条码 14。头 ID 仍是同表一条码 14 的树号（I-9.27 成立），
/// 只红 I-9.16（③ 快照条目一一配对）。
#[test]
fn two_snapshot_extent_roots_claiming_one_snapshot_inode_root_redden_only_the_tree_table_ordering()
{
    let mut image = image_with_two_snapshots("pool-checker-snapshots-claim-one");
    rewrite_the_tree_table_of_the_newest_root(&mut image, |tree_table| {
        let first_snapshot_inode_root_tree =
            field_of_entry(tree_table, FIRST_SNAPSHOT_INODE_ROOT_ENTRY_INDEX, 0);
        set_field_of_entry(
            tree_table,
            SECOND_SNAPSHOT_EXTENT_ROOT_ENTRY_INDEX,
            TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET,
            first_snapshot_inode_root_tree,
        );
    });
    assert_eq!(
        violated_invariants(&image),
        BTreeSet::from(["I-9.16"]),
        "两条码 15 配同一条码 14：只红 I-9.16"
    );
}

/// 必红世界：第二个快照的码 15 头 ID 改成活头 inode 树的树号（不是任何一条码 14）：红 I-9.27（码 15 的头 ID 不是同表一条码 14 的树号），
/// 配对随之也不成立、连带红 I-9.16 ③；别的不红。
#[test]
fn a_snapshot_extent_root_whose_head_is_the_live_inode_tree_reddens_the_head_identifiers() {
    let mut image = image_with_two_snapshots("pool-checker-snapshots-head-live");
    rewrite_the_tree_table_of_the_newest_root(&mut image, |tree_table| {
        let live_inode_tree = field_of_entry(tree_table, LIVE_INODE_TREE_ENTRY_INDEX, 0);
        set_field_of_entry(
            tree_table,
            SECOND_SNAPSHOT_EXTENT_ROOT_ENTRY_INDEX,
            TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET,
            live_inode_tree,
        );
    });
    assert_eq!(
        violated_invariants(&image),
        BTreeSet::from(["I-9.16", "I-9.27"]),
        "码 15 的头 ID 指活头 inode 树：红 I-9.27，配对连带红 I-9.16"
    );
}

/// 必红世界：活头 inode 条目的 `previous_snapshot_txg` 改成第一个快照的诞生 txg（不是最后一个的）：仍不大于根 txg、仍不是 0，
/// 只红 I-9.25（树表里有快照时活头的前驱等于最后一个快照的诞生 txg）。
#[test]
fn a_live_head_whose_previous_snapshot_txg_is_not_the_last_snapshot_reddens_only_the_previous_snapshot_txg(
) {
    let mut image = image_with_two_snapshots("pool-checker-snapshots-live-previous");
    rewrite_the_tree_table_of_the_newest_root(&mut image, |tree_table| {
        let first_snapshot_txg = field_of_entry(
            tree_table,
            FIRST_SNAPSHOT_INODE_ROOT_ENTRY_INDEX,
            TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET,
        );
        assert_ne!(
            field_of_entry(
                tree_table,
                LIVE_INODE_TREE_ENTRY_INDEX,
                TREE_TABLE_ENTRY_PREVIOUS_SNAPSHOT_TXG_OFFSET
            ),
            first_snapshot_txg,
            "改之前活头的前驱是第二个快照的诞生 txg，不是第一个的"
        );
        set_field_of_entry(
            tree_table,
            LIVE_INODE_TREE_ENTRY_INDEX,
            TREE_TABLE_ENTRY_PREVIOUS_SNAPSHOT_TXG_OFFSET,
            first_snapshot_txg,
        );
    });
    assert_eq!(
        violated_invariants(&image),
        BTreeSet::from(["I-9.25"]),
        "活头的前驱不是最后一个快照的诞生 txg：只红 I-9.25"
    );
}
