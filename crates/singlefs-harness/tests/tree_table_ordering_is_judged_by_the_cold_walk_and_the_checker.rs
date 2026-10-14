//! I-9.16（树表条目按树 ID 严格升序且合发号次序）。
//! 两道合成这一条，任一道不成立即判红：① 盘上次序树 ID 严格升序（D8（核心索引结构） 已定项 8 排序契约）；
//! ② 按发号次序（extent、inode、分配记录、记账、livelist、稀疏旁表、deadlist，再接第一个文件版本那次零根注册的 `deleted_inodes`、
//! 墓碑树、谱系树、logged_ops）相邻两棵的树 ID 严格升序（D8 已定项 8 ②；D18（块里携带什么信息） 已定项 24 第 6 条）。
//!
//! 实审 A3b 只在重建上一版（`rebuild_version`）判它；这里钉住另外两处也判：冷走读 `walk_to_file` 在拿到树表条目之后、
//! 读任何一棵树之前拒，池级 checker 每读一版的树表判一次、违例写明哪一版根与哪两条。
//! 两个探针都从写完第一个文件的干净镜像出发，只改最新那条根的树表单元（两份）、重封、把新的整单元校验和补进根记录再写回根槽：
//! 探针甲（livelist 与稀疏旁表两条互换种类）只违反 ②；探针乙（这两条整条互换位置）只违反 ①；
//! 探针丙（墓碑树与谱系树两条互换种类）只违反 ②，钉住两处的发号次序都认得四棵新树。
//! 只违反 ② 的那一份「checker 只红 I-9.16」钉在 `checker_known_bad_images.rs`；探针乙在 checker 上连 I-1.1 一起红
//! （树表的 key 就是树 ID，① 与 I-1.1 在树表上判的「key 严格递增」是同一个谓词），钉在这里。
mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker.rs"]
mod scenarios_of_tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker;

use common_images_for_the_checker_tier::PoolCheckerImages;
use scenarios_of_tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker::{
    clean_image_after_the_first_file, entry_range_in_node, kind_of_entry,
    rewrite_the_tree_table_of_the_newest_root, swap_the_kinds_of_the_tombstone_and_lineage_entries,
    swap_the_places_of_the_livelist_and_sparse_side_table_entries, LIVELIST_ENTRY_INDEX,
    SPARSE_SIDE_TABLE_ENTRY_INDEX, TREE_TABLE_ENTRY_KIND_OFFSET,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::allocator::UnitAreaStart;
use singlefs_core::records::{TREE_KIND_LIVELIST, TREE_KIND_SPARSE_SIDE_TABLE};
use singlefs_core::recovery::{
    rebuild_version, walk_to_file_in_the_unit_area_starting_at, PoolReader, RebuildVersionFailure,
    RecoveryFailure,
};
use singlefs_core::root_record::RootRecord;
use singlefs_harness::memory_pool::MemoryPool;
use std::cell::RefCell;
use std::collections::BTreeSet;

/// 探针甲的 detail（核心层只违反 ② 时报的那一句）。
const DETAIL_OUT_OF_THE_ISSUING_ORDER: &str =
    "树表里各棵树的树 ID 不按发号次序（extent、inode、分配记录、记账、livelist、稀疏旁表、deadlist、deleted_inodes、墓碑树、谱系树、logged_ops）严格升序";

/// 探针乙的 detail（核心层 ① 不成立时报的那一句）。
const DETAIL_OUT_OF_THE_ON_DISK_ORDER: &str = "树表条目不按树 ID 严格升序";

/// 探针甲：livelist 与稀疏旁表两条互换种类（条目偏移 10 的 2 字节）。盘上仍按树 ID 升序，按发号次序 livelist 的号 65542 大过稀疏旁表的号 65541。
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

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker::the_clean_image_after_the_first_file_holds_the_tree_table_ordering_on_the_cold_walk_and_the_checker`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn the_clean_image_after_the_first_file_holds_the_tree_table_ordering_on_the_cold_walk_and_the_checker(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker::the_clean_image_after_the_first_file_holds_the_tree_table_ordering_on_the_cold_walk_and_the_checker,
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

/// 探针丙（只违反 ②，落在第一个文件版本那次零根注册的两棵新树上）：冷走读在读任何一棵树之前拒成 I-9.16。
/// 核心层的发号次序里没有这两棵时它一路走到文件。
#[test]
fn the_cold_walk_refuses_a_tree_table_whose_tombstone_and_lineage_entries_swapped_their_kinds_before_reading_any_tree(
) {
    let (mut image, _) =
        clean_image_after_the_first_file("tree-table-ordering-walk-new-tree-kinds");
    let root = rewrite_the_tree_table_of_the_newest_root(
        &mut image,
        swap_the_kinds_of_the_tombstone_and_lineage_entries,
    );
    assert_the_cold_walk_refuses_before_reading_any_tree(
        &image,
        &root,
        DETAIL_OUT_OF_THE_ISSUING_ORDER,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker::the_checker_reddens_only_the_tree_table_ordering_when_the_tombstone_and_lineage_entries_swapped_their_kinds`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn the_checker_reddens_only_the_tree_table_ordering_when_the_tombstone_and_lineage_entries_swapped_their_kinds(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker::the_checker_reddens_only_the_tree_table_ordering_when_the_tombstone_and_lineage_entries_swapped_their_kinds,
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

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker::the_checker_reddens_the_tree_table_ordering_and_the_key_order_when_two_entries_swapped_their_places`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn the_checker_reddens_the_tree_table_ordering_and_the_key_order_when_two_entries_swapped_their_places(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker::the_checker_reddens_the_tree_table_ordering_and_the_key_order_when_two_entries_swapped_their_places,
    );
}
