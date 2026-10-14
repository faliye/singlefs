//! harness 测试文件 `crates/singlefs-harness/tests/tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::{
    build_pool, copy_each_root_into_the_new_root_segment_of_its_journal_records, file_content,
};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_core::allocator::UnitAreaStart;
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::journal::JournalRecord;
use singlefs_core::records::{TREE_KIND_INODE, TREE_KIND_LINEAGE, TREE_KIND_TOMBSTONE};
use singlefs_core::recovery::{
    choose_system_configuration, every_root_ring_slot, read_root_ring_slot,
    walk_to_file_in_the_unit_area_starting_at, RootRingSlotReading,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::slot_offset;
use singlefs_core::unit::seal_header_checksum;
use singlefs_format::NODE_BYTES;
use singlefs_harness::memory_pool::MemoryPool;

/// 树表条目里种类那 2 字节的偏移（树 ID 8 + 条目长度 2 之后，D8（核心索引结构） 已定项 8）。
pub const TREE_TABLE_ENTRY_KIND_OFFSET: usize = 10;

/// 树表条目里头 ID 那 8 字节的偏移（树 ID 8 + 条目长度 2 + 种类 2 + flags 2 + 根指针 86 + previous_snapshot_txg 8 + 诞生 txg 8 之后，
/// D8（核心索引结构） 已定项 8「头 ID 8 字节的取值」，判它的是 I-9.27（树表条目的头 ID 取值））。
pub const TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET: usize = 116;

/// 种类码 2（inode 树）的条目在第一个文件那一版树表里是第 1 条（extent 树第 0 条）。
pub const INODE_ENTRY_INDEX: usize = 1;

/// 第一个文件那一版树表里十一条按发号次序排：livelist 是第 4 条（从 0 数）、稀疏旁表第 5 条，号 65541、65542（mkfs 水位 65536 起连号发，
/// 中央映射树占 65540、不进树表）。
pub const LIVELIST_ENTRY_INDEX: usize = 4;

pub const SPARSE_SIDE_TABLE_ENTRY_INDEX: usize = 5;

/// 第一个文件版本那次零根注册的四棵（D18（块里携带什么信息） 已定项 24 第 6 条）接在 deadlist（第 6 条）之后：
/// `deleted_inodes` 第 7 条、墓碑树第 8 条、谱系树第 9 条、logged_ops 第 10 条，号 65544..65547。
pub const TOMBSTONE_ENTRY_INDEX: usize = 8;

pub const LINEAGE_ENTRY_INDEX: usize = 9;

/// 码 2 节点第 `index` 条条目在节点里的字节范围（条目区从 86 + 2k + 29 起，条目宽住 84 + 2k）。
pub fn entry_range_in_node(node: &[u8], index: usize) -> std::ops::Range<usize> {
    let key_width = usize::from(node[51]);
    let entry_width = usize::from(u16::from_le_bytes([
        node[84 + 2 * key_width],
        node[85 + 2 * key_width],
    ]));
    let entries_start = 86 + 2 * key_width + 29;
    entries_start + index * entry_width..entries_start + (index + 1) * entry_width
}

pub fn kind_of_entry(node: &[u8], index: usize) -> u16 {
    let entry = &node[entry_range_in_node(node, index)];
    u16::from_le_bytes([
        entry[TREE_TABLE_ENTRY_KIND_OFFSET],
        entry[TREE_TABLE_ENTRY_KIND_OFFSET + 1],
    ])
}

pub fn head_identifier_of_entry(node: &[u8], index: usize) -> u64 {
    let entry = &node[entry_range_in_node(node, index)];
    u64::from_le_bytes(
        entry[TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET..TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET + 8]
            .try_into()
            .expect("8 字节"),
    )
}

pub fn tree_identifier_of_entry(node: &[u8], index: usize) -> u64 {
    let entry = &node[entry_range_in_node(node, index)];
    u64::from_le_bytes(entry[0..8].try_into().expect("8 字节"))
}

/// 码 2 节点改过之后重封：载荷 CRC（罩 [86 + 2k, 16384)，住 76 + 2k）与头校验和（罩 [0, 86 + 2k)）。
pub fn reseal_index_node(node: &mut [u8]) {
    let key_width = usize::from(node[51]);
    let header_end = 86 + 2 * key_width;
    let payload_checksum = crc32_castagnoli(&node[header_end..]);
    node[header_end - 10..header_end - 6].copy_from_slice(&payload_checksum.to_le_bytes());
    seal_header_checksum(node, header_end);
}

/// 探针丙：墓碑树与谱系树两条互换种类，头 ID 跟着新种类一起换（I-9.27：墓碑树的头 ID 等于 inode 树的树 ID、谱系树写 0；
/// 只换种类不换头 ID，两条的头 ID 都不合新种类，I-9.27 跟着红）。盘上仍按树 ID 升序，按发号次序墓碑树（种类 10）取到的号 65546
/// 大过谱系树（种类 11）取到的号 65545：只违反 ②（头 ID 合种类，I-9.27 不红），而且只在发号次序里认得这两棵新树的判法上红
/// （读法照抄树表之前七棵那一份的判法看不见它）。
pub fn swap_the_kinds_of_the_tombstone_and_lineage_entries(tree_table: &mut [u8]) {
    assert_eq!(
        kind_of_entry(tree_table, INODE_ENTRY_INDEX),
        TREE_KIND_INODE
    );
    assert_eq!(
        kind_of_entry(tree_table, TOMBSTONE_ENTRY_INDEX),
        TREE_KIND_TOMBSTONE
    );
    assert_eq!(
        kind_of_entry(tree_table, LINEAGE_ENTRY_INDEX),
        TREE_KIND_LINEAGE
    );
    let inode_tree_identifier = tree_identifier_of_entry(tree_table, INODE_ENTRY_INDEX);
    assert_eq!(
        head_identifier_of_entry(tree_table, TOMBSTONE_ENTRY_INDEX),
        inode_tree_identifier,
        "互换之前墓碑树的头 ID 等于 inode 树的树 ID"
    );
    assert_eq!(
        head_identifier_of_entry(tree_table, LINEAGE_ENTRY_INDEX),
        0,
        "互换之前谱系树的头 ID 是 0"
    );
    // 每条条目的头 ID 跟着它的新种类：种类 11（谱系树）写 0，种类 10（墓碑树）写 inode 树的树 ID。
    for (index, kind, head_identifier) in [
        (TOMBSTONE_ENTRY_INDEX, TREE_KIND_LINEAGE, 0),
        (
            LINEAGE_ENTRY_INDEX,
            TREE_KIND_TOMBSTONE,
            inode_tree_identifier,
        ),
    ] {
        let entry_start = entry_range_in_node(tree_table, index).start;
        let kind_start = entry_start + TREE_TABLE_ENTRY_KIND_OFFSET;
        tree_table[kind_start..kind_start + 2].copy_from_slice(&kind.to_le_bytes());
        let head_identifier_start = entry_start + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET;
        tree_table[head_identifier_start..head_identifier_start + 8]
            .copy_from_slice(&head_identifier.to_le_bytes());
    }
    assert_eq!(
        head_identifier_of_entry(tree_table, TOMBSTONE_ENTRY_INDEX),
        0,
        "互换之后第 8 条是种类 11（谱系树），头 ID 合它的新种类：0"
    );
    assert_eq!(
        head_identifier_of_entry(tree_table, LINEAGE_ENTRY_INDEX),
        inode_tree_identifier,
        "互换之后第 9 条是种类 10（墓碑树），头 ID 合它的新种类：inode 树的树 ID"
    );
}

/// 探针乙：livelist 与稀疏旁表两条整条互换位置（200 字节连同树 ID、种类、根指针一起搬）。种类仍跟着自己的号，
/// 按发号次序 65541 < 65542 照旧成立；盘上次序变成 …、65542、65541、…。首末两条没动，树表头里的 key 区间照旧贴紧。
pub fn swap_the_places_of_the_livelist_and_sparse_side_table_entries(tree_table: &mut [u8]) {
    let livelist = entry_range_in_node(tree_table, LIVELIST_ENTRY_INDEX);
    let sparse_side_table = entry_range_in_node(tree_table, SPARSE_SIDE_TABLE_ENTRY_INDEX);
    let livelist_bytes = tree_table[livelist.clone()].to_vec();
    let sparse_side_table_bytes = tree_table[sparse_side_table.clone()].to_vec();
    tree_table[livelist].copy_from_slice(&sparse_side_table_bytes);
    tree_table[sparse_side_table].copy_from_slice(&livelist_bytes);
}

/// 写完第一个文件的干净镜像，以及那一版的记录（重建上一版要它顶着）。
pub fn clean_image_after_the_first_file(tag: &str) -> (MemoryPool, JournalRecord) {
    let pool = build_pool(tag);
    (pool.memory_pool(), pool.output.record.clone())
}

/// 改最新那条根的树表单元（两份），重封、把新的整单元校验和补进根记录里树表指针的两条位置条目，再把根写回它的槽，
/// 同 (实例, txg) 的 journal 记录的新根段照抄改过的根（I-8.10（记录新根段与同代根相同）：写者发布时两处带同一份树表指针）：
/// 盘上每一道校验和都过，只剩树表条目的排序不对。交回改过之后的根。
pub fn rewrite_the_tree_table_of_the_newest_root(
    image: &mut MemoryPool,
    change: impl Fn(&mut [u8]),
) -> RootRecord {
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
    root
}

/// 干净镜像上冷走读读得出第一个文件、checker 判 I-9.16 成立：两个探针的红不是镜像本来就坏。
pub fn the_clean_image_after_the_first_file_holds_the_tree_table_ordering_on_the_cold_walk_and_the_checker(
    images: &mut PoolCheckerImages,
) {
    let (image, _) = clean_image_after_the_first_file("tree-table-ordering-clean");
    let mut rewritten_without_change = image.clone();
    let root =
        rewrite_the_tree_table_of_the_newest_root(&mut rewritten_without_change, |_tree_table| {});
    assert_eq!(
        rewritten_without_change, image,
        "什么都不改时重封与写回逐字节不变：探针改的只有它们改的那几个字节"
    );
    let mut mapping_fallbacks = 0usize;
    assert_eq!(
        walk_to_file_in_the_unit_area_starting_at(
            &image,
            UnitAreaStart::of_the_default_journal_ring(),
            &root,
            &mut mapping_fallbacks,
        ),
        Ok(Some(file_content())),
        "干净镜像上冷走读读得出第一个文件"
    );
    images.hand_out(
        "第一个文件之后的干净镜像",
        || image,
        PoolCheckerExpectation::only_the_named_invariants().and_holds("I-9.16"),
    );
}

/// 探针丙在 checker 上：只红 I-9.16，第一处违例写明最新那条根（实例 1、txg 3）与两条的种类、下标、号。
/// checker 的发号次序里没有这两棵时一条都不红。
pub fn the_checker_reddens_only_the_tree_table_ordering_when_the_tombstone_and_lineage_entries_swapped_their_kinds(
    images: &mut PoolCheckerImages,
) {
    let (mut image, _) =
        clean_image_after_the_first_file("tree-table-ordering-checker-new-tree-kinds");
    rewrite_the_tree_table_of_the_newest_root(
        &mut image,
        swap_the_kinds_of_the_tombstone_and_lineage_entries,
    );
    // 墓碑树与谱系树互换种类：判红的该只有 I-9.16，红在最新那条根的树表墓碑树、谱系树两条
    images.hand_out(
        "墓碑树与谱系树两条树表条目互换种类",
        || image,
        PoolCheckerExpectation::exactly_these_violated(&["I-9.16"]).and_violated_with_a_detail_containing(
            "I-9.16",
            "根（实例 1、txg 3）的树表：种类 10 那条（第 9 条，树 ID 65546）与发号次序上的下一棵、种类 11 那条（第 8 条，树 ID 65545）不按树 ID 严格升序（② 发号次序）",
        ),
    );
}

/// 探针乙在 checker 上：红的恰是 I-1.1（树表的 key 不严格递增）与 I-9.16（① 盘上次序）两条，I-9.16 的第一处违例写明最新那条根
/// （实例 1、txg 3）与第 4、5 两条的号。只违反 ② 的探针甲「只红 I-9.16」钉在 `checker_known_bad_images.rs`。
pub fn the_checker_reddens_the_tree_table_ordering_and_the_key_order_when_two_entries_swapped_their_places(
    images: &mut PoolCheckerImages,
) {
    let (mut image, _) = clean_image_after_the_first_file("tree-table-ordering-checker-places");
    rewrite_the_tree_table_of_the_newest_root(
        &mut image,
        swap_the_places_of_the_livelist_and_sparse_side_table_entries,
    );
    // 两条整条互换位置：判红的该只有 I-1.1 与 I-9.16，I-9.16 红在最新那条根的树表第 4、5 两条
    images.hand_out(
        "livelist 与稀疏旁表两条树表条目互换位置",
        || image,
        PoolCheckerExpectation::exactly_these_violated(&["I-1.1", "I-9.16"]).and_violated_with_a_detail_containing(
            "I-9.16",
            "根（实例 1、txg 3）的树表：第 4 条（树 ID 65542）与第 5 条（树 ID 65541）不按树 ID 严格升序（① 盘上次序）",
        ),
    );
}
