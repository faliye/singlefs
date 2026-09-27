//! I-9.16（树表条目按树 ID 严格升序且合发号次序）：实审 A3b Q2 交上来，用户 2026-09-27 JST 17:4x 定「立不变量并同步」。
//! 两道合成这一条，任一道不成立即判红：① 盘上次序树 ID 严格升序（D8（核心索引结构） 已定项 8 排序契约）；
//! ② 按发号次序（extent、inode、分配记录、记账、livelist、稀疏旁表、deadlist）相邻两棵的树 ID 严格升序（D8 已定项 8 ②）。
//!
//! 实审 A3b 只在重建上一版（`rebuild_version`）判它；这里钉住另外两处也判：冷走读 `walk_to_file` 在拿到树表条目之后、
//! 读任何一棵树之前拒，池级 checker 每读一版的树表判一次、违例写明哪一版根与哪两条。
//! 两个探针都从写完第一个文件的干净镜像出发，只改最新那条根的树表单元（两份）、重封、把新的整单元校验和补进根记录再写回根槽：
//! 探针甲（livelist 与稀疏旁表两条互换种类）只违反 ②；探针乙（这两条整条互换位置）只违反 ①。
//! 只违反 ② 的那一份「checker 只红 I-9.16」钉在 `checker_known_bad_images.rs`；探针乙在 checker 上连 I-1.1 一起红
//! （树表的 key 就是树 ID，① 与 I-1.1 在树表上判的「key 严格递增」是同一个谓词），钉在这里。
mod common;

use std::cell::RefCell;
use std::collections::BTreeSet;

use common::{build_pool, file_content};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::allocator::UnitAreaStart;
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::journal::JournalRecord;
use singlefs_core::records::{TREE_KIND_LIVELIST, TREE_KIND_SPARSE_SIDE_TABLE};
use singlefs_core::recovery::{
    choose_system_configuration, every_root_ring_slot, read_root_ring_slot, rebuild_version,
    walk_to_file_in_the_unit_area_starting_at, PoolReader, RebuildVersionFailure, RecoveryFailure,
    RootRingSlotReading,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::slot_offset;
use singlefs_core::unit::seal_header_checksum;
use singlefs_format::NODE_BYTES;
use singlefs_harness::crash::MemoryPool;

/// 树表条目里种类那 2 字节的偏移（树 ID 8 + 条目长度 2 之后，D8（核心索引结构） 已定项 8）。
const TREE_TABLE_ENTRY_KIND_OFFSET: usize = 10;
/// 第一个文件那一版树表里七条按发号次序排：livelist 是第 4 条（从 0 数）、稀疏旁表第 5 条，号 16、17（mkfs 水位 11 起连号发，
/// 中央映射树占 15、不进树表）。
const LIVELIST_ENTRY_INDEX: usize = 4;
const SPARSE_SIDE_TABLE_ENTRY_INDEX: usize = 5;
/// 探针甲的 detail（核心层只违反 ② 时报的那一句）。
const DETAIL_OUT_OF_THE_ISSUING_ORDER: &str =
    "树表里七棵树的树 ID 不按发号次序（extent、inode、分配记录、记账、livelist、稀疏旁表、deadlist）严格升序";
/// 探针乙的 detail（核心层 ① 不成立时报的那一句）。
const DETAIL_OUT_OF_THE_ON_DISK_ORDER: &str = "树表条目不按树 ID 严格升序";

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

fn kind_of_entry(node: &[u8], index: usize) -> u16 {
    let entry = &node[entry_range_in_node(node, index)];
    u16::from_le_bytes([
        entry[TREE_TABLE_ENTRY_KIND_OFFSET],
        entry[TREE_TABLE_ENTRY_KIND_OFFSET + 1],
    ])
}

/// 码 2 节点改过之后重封：载荷 CRC（罩 [86 + 2k, 16384)，住 76 + 2k）与头校验和（罩 [0, 86 + 2k)）。
fn reseal_index_node(node: &mut [u8]) {
    let key_width = usize::from(node[51]);
    let header_end = 86 + 2 * key_width;
    let payload_checksum = crc32_castagnoli(&node[header_end..]);
    node[header_end - 10..header_end - 6].copy_from_slice(&payload_checksum.to_le_bytes());
    seal_header_checksum(node, header_end);
}

/// 探针甲：livelist 与稀疏旁表两条互换种类（条目偏移 10 的 2 字节）。盘上仍按树 ID 升序，按发号次序 livelist 的号 17 大过稀疏旁表的号 16。
fn swap_the_kinds_of_the_livelist_and_sparse_side_table_entries(tree_table: &mut [u8]) {
    assert_eq!(
        kind_of_entry(tree_table, LIVELIST_ENTRY_INDEX),
        TREE_KIND_LIVELIST
    );
    assert_eq!(
        kind_of_entry(tree_table, SPARSE_SIDE_TABLE_ENTRY_INDEX),
        TREE_KIND_SPARSE_SIDE_TABLE
    );
    for (index, kind) in [
        (LIVELIST_ENTRY_INDEX, TREE_KIND_SPARSE_SIDE_TABLE),
        (SPARSE_SIDE_TABLE_ENTRY_INDEX, TREE_KIND_LIVELIST),
    ] {
        let kind_start =
            entry_range_in_node(tree_table, index).start + TREE_TABLE_ENTRY_KIND_OFFSET;
        tree_table[kind_start..kind_start + 2].copy_from_slice(&kind.to_le_bytes());
    }
}

/// 探针乙：livelist 与稀疏旁表两条整条互换位置（200 字节连同树 ID、种类、根指针一起搬）。种类仍跟着自己的号，
/// 按发号次序 16 < 17 照旧成立；盘上次序变成 …、17、16、…。首末两条没动，树表头里的 key 区间照旧贴紧。
fn swap_the_places_of_the_livelist_and_sparse_side_table_entries(tree_table: &mut [u8]) {
    let livelist = entry_range_in_node(tree_table, LIVELIST_ENTRY_INDEX);
    let sparse_side_table = entry_range_in_node(tree_table, SPARSE_SIDE_TABLE_ENTRY_INDEX);
    let livelist_bytes = tree_table[livelist.clone()].to_vec();
    let sparse_side_table_bytes = tree_table[sparse_side_table.clone()].to_vec();
    tree_table[livelist].copy_from_slice(&sparse_side_table_bytes);
    tree_table[sparse_side_table].copy_from_slice(&livelist_bytes);
}

/// 写完第一个文件的干净镜像，以及那一版的记录（重建上一版要它顶着）。
fn clean_image_after_the_first_file(tag: &str) -> (MemoryPool, JournalRecord) {
    let pool = build_pool(tag);
    (pool.memory_pool(), pool.output.record.clone())
}

/// 改最新那条根的树表单元（两份），重封、把新的整单元校验和补进根记录里树表指针的两条位置条目，再把根写回它的槽：
/// 盘上每一道校验和都过，只剩树表条目的排序不对。交回改过之后的根。
fn rewrite_the_tree_table_of_the_newest_root(
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
    root
}

/// 记下每一次读落在哪 (盘, 偏移)：冷走读在读任何一棵树之前拒，读过的就只有实例表与树表那几份。
struct ReadRecordingReader<'image> {
    image: &'image MemoryPool,
    reads: RefCell<Vec<(DeviceIdentity, DeviceOffsetInBytes)>>,
}

impl PoolReader for ReadRecordingReader<'_> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        self.image.device_identities()
    }
    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        self.image.device_size_in_bytes(device)
    }
    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        self.reads.borrow_mut().push((device, offset));
        self.image.read(device, offset, length)
    }
    fn journal_record_offsets_hint(
        &self,
        device: DeviceIdentity,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        self.image
            .journal_record_offsets_hint(device, ring_start, ring_bytes)
    }
}

/// 冷走读拒成 I-9.16、detail 是 `expected_detail`，而且读过的只有这条根的实例表与树表那几份。
fn assert_the_cold_walk_refuses_before_reading_any_tree(
    image: &MemoryPool,
    root: &RootRecord,
    expected_detail: &'static str,
) {
    let reader = ReadRecordingReader {
        image,
        reads: RefCell::new(Vec::new()),
    };
    let mut mapping_fallbacks = 0usize;
    let walked = walk_to_file_in_the_unit_area_starting_at(
        &reader,
        UnitAreaStart::of_the_default_journal_ring(),
        root,
        &mut mapping_fallbacks,
    );
    assert_eq!(
        walked,
        Err(RecoveryFailure::InvariantViolated {
            invariant: "I-9.16",
            detail: expected_detail,
        }),
        "冷走读拒成 I-9.16"
    );
    let units_of_the_root_itself: BTreeSet<(DeviceIdentity, DeviceOffsetInBytes)> = root
        .instance_table
        .locations
        .iter()
        .chain(root.tree_table.locations.iter())
        .map(|location| (location.device, location.slot.to_device_offset()))
        .collect();
    let reads = reader.reads.into_inner();
    assert!(!reads.is_empty(), "冷走读读过实例表与树表");
    for read in &reads {
        assert!(
            units_of_the_root_itself.contains(read),
            "冷走读在拒之前读了实例表与树表之外的 {read:?}：它往树里走了"
        );
    }
}

/// 干净镜像上冷走读读得出第一个文件、checker 判 I-9.16 成立：两个探针的红不是镜像本来就坏。
#[test]
fn the_clean_image_after_the_first_file_holds_the_tree_table_ordering_on_the_cold_walk_and_the_checker(
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
    let verdicts = check_pool_image(&image);
    let tree_table_ordering = verdicts
        .iter()
        .find(|(invariant, _)| *invariant == "I-9.16")
        .map(|(_, verdict)| verdict.clone());
    assert_eq!(
        tree_table_ordering,
        Some(InvariantVerdict::Holds),
        "干净镜像上 I-9.16 真被评估过且成立"
    );
}

/// 探针甲（只违反 ②）：冷走读在读任何一棵树之前拒成 I-9.16。改之前冷走读不判，livelist 与稀疏旁表的根都是空根，一路走到文件。
#[test]
fn the_cold_walk_refuses_a_tree_table_whose_two_entries_swapped_their_kinds_before_reading_any_tree(
) {
    let (mut image, _) = clean_image_after_the_first_file("tree-table-ordering-walk-kinds");
    let root = rewrite_the_tree_table_of_the_newest_root(
        &mut image,
        swap_the_kinds_of_the_livelist_and_sparse_side_table_entries,
    );
    assert_the_cold_walk_refuses_before_reading_any_tree(
        &image,
        &root,
        DETAIL_OUT_OF_THE_ISSUING_ORDER,
    );
}

/// 探针乙（只违反 ①）：冷走读在读任何一棵树之前拒成 I-9.16。
#[test]
fn the_cold_walk_refuses_a_tree_table_whose_two_entries_swapped_their_places_before_reading_any_tree(
) {
    let (mut image, _) = clean_image_after_the_first_file("tree-table-ordering-walk-places");
    let root = rewrite_the_tree_table_of_the_newest_root(
        &mut image,
        swap_the_places_of_the_livelist_and_sparse_side_table_entries,
    );
    assert_the_cold_walk_refuses_before_reading_any_tree(
        &image,
        &root,
        DETAIL_OUT_OF_THE_ON_DISK_ORDER,
    );
}

/// 重建上一版（可写挂载接着它发布）拒成 `Walk(InvariantViolated { "I-9.16", detail })`。
fn assert_rebuilding_the_previous_version_refuses(
    image_tag: &str,
    change_the_tree_table: impl Fn(&mut [u8]),
    expected_detail: &'static str,
) {
    let (mut image, record) = clean_image_after_the_first_file(image_tag);
    let root = rewrite_the_tree_table_of_the_newest_root(&mut image, change_the_tree_table);
    let rebuilt = rebuild_version(&image, &root, Some(record));
    assert_eq!(
        rebuilt.err(),
        Some(RebuildVersionFailure::Walk(
            RecoveryFailure::InvariantViolated {
                invariant: "I-9.16",
                detail: expected_detail,
            }
        )),
        "重建上一版拒成 I-9.16"
    );
}

/// 探针甲（只违反 ②）：重建上一版拒成 I-9.16，红在发号次序那一句。
#[test]
fn rebuilding_the_previous_version_refuses_a_tree_table_whose_two_entries_swapped_their_kinds() {
    assert_rebuilding_the_previous_version_refuses(
        "tree-table-ordering-rebuild-kinds",
        swap_the_kinds_of_the_livelist_and_sparse_side_table_entries,
        DETAIL_OUT_OF_THE_ISSUING_ORDER,
    );
}

/// 探针乙（只违反 ①）：重建上一版拒成 I-9.16，红在盘上次序那一句。
#[test]
fn rebuilding_the_previous_version_refuses_a_tree_table_whose_two_entries_swapped_their_places() {
    assert_rebuilding_the_previous_version_refuses(
        "tree-table-ordering-rebuild-places",
        swap_the_places_of_the_livelist_and_sparse_side_table_entries,
        DETAIL_OUT_OF_THE_ON_DISK_ORDER,
    );
}

/// 探针乙在 checker 上：红的恰是 I-1.1（树表的 key 不严格递增）与 I-9.16（① 盘上次序）两条，I-9.16 的第一处违例写明最新那条根
/// （实例 1、txg 3）与第 4、5 两条的号。只违反 ② 的探针甲「只红 I-9.16」钉在 `checker_known_bad_images.rs`。
#[test]
fn the_checker_reddens_the_tree_table_ordering_and_the_key_order_when_two_entries_swapped_their_places(
) {
    let (mut image, _) = clean_image_after_the_first_file("tree-table-ordering-checker-places");
    rewrite_the_tree_table_of_the_newest_root(
        &mut image,
        swap_the_places_of_the_livelist_and_sparse_side_table_entries,
    );
    let verdicts = check_pool_image(&image);
    let violated: Vec<&str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(invariant, _)| *invariant)
        .collect();
    assert_eq!(
        violated,
        ["I-1.1", "I-9.16"],
        "两条整条互换位置：判红的该只有 I-1.1 与 I-9.16：{verdicts:?}"
    );
    let tree_table_ordering = verdicts
        .iter()
        .find(|(invariant, _)| *invariant == "I-9.16")
        .map(|(_, verdict)| verdict.clone());
    assert_eq!(
        tree_table_ordering,
        Some(InvariantVerdict::Violated(
            "根（实例 1、txg 3）的树表：第 4 条（树 ID 17）与第 5 条（树 ID 16）不按树 ID 严格升序（① 盘上次序）"
                .to_owned()
        )),
        "红在最新那条根的树表第 4、5 两条"
    );
}
