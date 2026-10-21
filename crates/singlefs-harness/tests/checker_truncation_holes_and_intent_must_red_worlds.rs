//! 池级 checker 为截断与洞新判、改判的几条（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K7 的 T-1 到 T-4，第六节 S1）
//! 各配合法世界与必红世界：
//! I-4.4（意图只描述目标态）收截断条目（T-1）、I-4.10（截断中间态有账，T-2）、I-4.6（中间态其余不变量成立）每条记录判单元数对 size 与
//! 上段叶条目的标签（T-3，合并 C711）、I-9.38（数据单元声明长度对 size，T-4）。
//!
//! 合法世界用真写路径造（截断的意图、各批与最后一批，变长留洞，有洞文件的 unlink 意图）；必红世界在真镜像上按字节改一处、重封，
//! 新的整单元校验和沿引用链补到根槽（`common_pointer_head_corruption::replace_the_unit_and_propagate_its_checksum`，经
//! `common_deletion_in_progress::change_the_unit_and_propagate_its_checksum`）。
mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;
mod common_truncation;

use common::build_pool;
use common_deletion_in_progress::{
    change_the_inode_record, change_the_unit_and_propagate_its_checksum,
    pool_with_a_three_unit_file, publish_the_intent_of,
};
use common_pointer_head_corruption::{
    first_upper_leaf_entry, node_bytes, the_slot_of, tree_table_entry_of_kind, unit_bytes_at,
};
use common_truncation::{
    batches_of_one, change_the_size_of_the_first_file_and_advance, payload_capacity,
    publish_the_next_truncation_step_and_advance, truncation_entry_of,
    upper_leaf_tag_of_the_first_file,
};
use singlefs_checker::image::{
    InvariantVerdict, DATA_UNITS_BEYOND_THE_SIZE_ARE_NAMED_BY_A_TRUNCATION_INTENT,
    DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
    DECLARED_LENGTH_OF_EACH_DATA_UNIT_WITHIN_THE_SIZE_MATCHES_THE_SIZE,
    INTENT_DESCRIBES_ONLY_THE_TARGET_STATE, SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::extent_tree::ExtentUpperLeafTarget;
use singlefs_core::records::{TREE_KIND_EXTENT, TREE_KIND_LOGGED_OPERATIONS};
use singlefs_core::transaction::FIRST_INODE_NUMBER;
use singlefs_core::unit::parse_index_node;
use singlefs_harness::memory_pool::MemoryPool;

/// logged_ops 条目 44（判决 K1）：key 是种类 2 | 签发 txg 8；截断意图的 value 是 inode 号 8 | 对象出生代 8 | 目标 size 8 | 补零 10。
const LOGGED_OPERATION_KEY_BYTES: usize = 10;
const LOGGED_OPERATION_ISSUED_TXG_OFFSET: usize = 2;
const TRUNCATION_INTENT_OBJECT_BIRTH_OFFSET: usize = 18;
const TRUNCATION_INTENT_TARGET_SIZE_OFFSET: usize = 26;
const TRUNCATION_INTENT_LAST_PADDING_BYTE_OFFSET: usize = 43;
/// logged_ops 条目的种类码 3：登记表里没有的码（1 = 整理、2 = 截断）。
const UNREGISTERED_LOGGED_OPERATION_KIND: u16 = 3;
/// inode 记录 140 里 size（偏移 40）与 blocks（偏移 48，⌈size ÷ 512⌉，I-9.15）。
const INODE_RECORD_SIZE_OFFSET: usize = 40;
const INODE_RECORD_BLOCKS_OFFSET: usize = 48;
const INODE_RECORD_BLOCKS_FIELD_UNIT_BYTES: u64 = 512;
/// 码 2 节点（key 宽 k）：声明长度在偏移 8、key 区间从 52 起两个 key、条目数在 82 + 2k、条目从 115 + 2k 起（D18（块里携带什么信息）
/// 已定项 18 的偏移表）。
const INDEX_NODE_DECLARED_LENGTH_OFFSET: usize = 8;
const INDEX_NODE_KEY_RANGE_OFFSET: usize = 52;
/// extent 上段叶条目 113：key 24（locality 8、inode 8、offset 8）之后一字节标签；下段叶记录 112 的 offset 段在 key 的 16..24。
const EXTENT_UPPER_LEAF_ENTRY_INODE_OFFSET: usize = 8;
const EXTENT_UPPER_LEAF_ENTRY_TAG_OFFSET: usize = 24;
const EXTENT_LOWER_LEAF_RECORD_OFFSET_IN_BYTES_OFFSET: usize = 16;
/// 上段叶条目的标签 3：下段根，下段最后一个单元之后到 size 是洞（判决 K5）。
const UPPER_LEAF_TAG_LOWER_SEGMENT_ROOT_FOLLOWED_BY_A_HOLE: u8 = 3;
/// 截断与改 size 的目标落在单元 0 中间。
const SIZE_INSIDE_UNIT_ZERO: u64 = 100;

fn index_node_entry_count_offset(key_width: usize) -> usize {
    82 + 2 * key_width
}

fn index_node_entries_start(key_width: usize) -> usize {
    115 + 2 * key_width
}

fn read_u64_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8 字节"))
}

fn violations_of(image: &MemoryPool) -> Vec<(&'static str, String)> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some((invariant, detail)),
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect()
}

fn violated_names(violations: &[(&'static str, String)]) -> Vec<&'static str> {
    violations.iter().map(|(invariant, _)| *invariant).collect()
}

fn verdict_of(image: &MemoryPool, invariant: &str) -> InvariantVerdict {
    check_pool_image(image)
        .into_iter()
        .find(|(candidate, _)| *candidate == invariant)
        .map(|(_, verdict)| verdict)
        .expect("清单里有这一条")
}

fn detail_of<'violations>(
    violations: &'violations [(&'static str, String)],
    invariant: &str,
) -> &'violations str {
    violations
        .iter()
        .find(|(candidate, _)| *candidate == invariant)
        .map(|(_, detail)| detail.as_str())
        .unwrap_or_else(|| panic!("{invariant} 要红：{violations:?}"))
}

fn assert_holds_every_invariant(image: &MemoryPool, what: &str) {
    let violations = violations_of(image);
    assert!(
        violations.is_empty(),
        "{what}：每一条都不该红，实际 {violations:?}"
    );
}

fn assert_judged_and_holding(image: &MemoryPool, invariants: &[&str], what: &str) {
    for invariant in invariants {
        assert_eq!(
            verdict_of(image, invariant),
            InvariantVerdict::Holds,
            "{what}：{invariant} 真判过且成立"
        );
    }
}

/// 码 2 节点 `slot`（两盘同槽）的条目整份换掉：`change` 拿到条目列表；条目数与声明长度跟着改，旧条目区之后补零；
/// `tighten_the_key_range` 时 key 区间贴紧新的首末条目（logged_ops 这种不按位置寻址的树，空节点区间不动），否则不动
/// （按位置寻址的 extent 树下段，区间是位置规定的那一段）。重封、补校验和。
fn rewrite_the_entries_of_the_index_node_at(
    image: &mut MemoryPool,
    slot: singlefs_core::address::SlotNumber,
    tighten_the_key_range: bool,
    change: impl FnOnce(&mut Vec<Vec<u8>>),
) {
    let parsed = parse_index_node(&unit_bytes_at(image, slot, node_bytes())).expect("节点解得开");
    let key_width = parsed.key_width;
    let entry_width = parsed.entry_width;
    let mut entries = parsed.entries;
    change(&mut entries);
    change_the_unit_and_propagate_its_checksum(image, slot, node_bytes(), |node| {
        let entries_start = index_node_entries_start(key_width);
        node[entries_start..].fill(0);
        for (position, entry) in entries.iter().enumerate() {
            assert_eq!(entry.len(), entry_width, "条目宽不变");
            let start = entries_start + position * entry_width;
            node[start..start + entry_width].copy_from_slice(entry);
        }
        let count_offset = index_node_entry_count_offset(key_width);
        node[count_offset..count_offset + 2].copy_from_slice(
            &u16::try_from(entries.len())
                .expect("条目数装得进 2 字节")
                .to_le_bytes(),
        );
        node[INDEX_NODE_DECLARED_LENGTH_OFFSET..INDEX_NODE_DECLARED_LENGTH_OFFSET + 2]
            .copy_from_slice(
                &u16::try_from(entries.len() * entry_width)
                    .expect("声明长度装得进 2 字节")
                    .to_le_bytes(),
            );
        if tighten_the_key_range {
            if let (Some(first), Some(last)) = (entries.first(), entries.last()) {
                let smallest = INDEX_NODE_KEY_RANGE_OFFSET;
                let largest = INDEX_NODE_KEY_RANGE_OFFSET + key_width;
                node[smallest..smallest + key_width].copy_from_slice(&first[..key_width]);
                node[largest..largest + key_width].copy_from_slice(&last[..key_width]);
            }
        }
    });
}

/// 最新那一版 logged_ops 树（这几份镜像上是根兼叶）的条目整份换掉。
fn rewrite_the_logged_operations_entries(
    image: &mut MemoryPool,
    change: impl FnOnce(&mut Vec<Vec<u8>>),
) {
    let root = tree_table_entry_of_kind(image, TREE_KIND_LOGGED_OPERATIONS).root;
    let slot = the_slot_of(&root);
    assert_eq!(
        unit_bytes_at(image, slot, node_bytes())[50],
        0,
        "logged_ops 树只有根兼叶"
    );
    rewrite_the_entries_of_the_index_node_at(image, slot, true, change);
}

/// 最新那一版那一条截断条目改一处（`change` 拿到那 44 字节）。
fn change_the_truncation_entry(image: &mut MemoryPool, change: impl FnOnce(&mut Vec<u8>)) {
    rewrite_the_logged_operations_entries(image, |entries| {
        assert_eq!(entries.len(), 1, "logged_ops 里只有那一条截断条目");
        change(&mut entries[0]);
    });
}

/// 第一个文件的记录 size 改成 `size_in_bytes`（blocks 跟着按 ⌈size ÷ 512⌉ 改，I-9.15 照旧成立），数据单元一个不动。
fn set_the_size_of_the_first_file(image: &mut MemoryPool, size_in_bytes: u64) {
    change_the_inode_record(image, FIRST_INODE_NUMBER, |record| {
        record[INODE_RECORD_SIZE_OFFSET..INODE_RECORD_SIZE_OFFSET + 8]
            .copy_from_slice(&size_in_bytes.to_le_bytes());
        record[INODE_RECORD_BLOCKS_OFFSET..INODE_RECORD_BLOCKS_OFFSET + 8].copy_from_slice(
            &size_in_bytes
                .div_ceil(INODE_RECORD_BLOCKS_FIELD_UNIT_BYTES)
                .to_le_bytes(),
        );
    });
}

/// 第一个文件下段（标签 1，下段根兼叶）里第 `unit` 个单元那条 extent 叶记录拿掉；单元本身、它的分配记录与映射条目都不动。
fn drop_the_extent_record_of_the_first_file_at_unit(image: &mut MemoryPool, unit: u64) {
    let ExtentUpperLeafTarget::LowerSegmentRoot(lower_root) = first_upper_leaf_entry(image).target
    else {
        panic!("三个单元的第一个文件上段叶条目是标签 1（下段根）")
    };
    let offset_in_bytes = unit * payload_capacity();
    rewrite_the_entries_of_the_index_node_at(image, the_slot_of(&lower_root), false, |entries| {
        let before = entries.len();
        entries.retain(|record| {
            read_u64_at(record, EXTENT_LOWER_LEAF_RECORD_OFFSET_IN_BYTES_OFFSET) != offset_in_bytes
        });
        assert_eq!(
            entries.len() + 1,
            before,
            "下段叶里恰好一条记录在单元 {unit}"
        );
    });
}

/// 最新那一版 extent 树上段（根兼叶）里第一个文件那一条的标签改成 `tag`，载荷不动（上段按位置寻址，key 区间不动）。
fn set_the_upper_leaf_tag_of_the_first_file(image: &mut MemoryPool, tag: u8) {
    let extent_root = tree_table_entry_of_kind(image, TREE_KIND_EXTENT).root;
    let slot = the_slot_of(&extent_root);
    rewrite_the_entries_of_the_index_node_at(image, slot, false, |entries| {
        let first_file_entry = entries
            .iter_mut()
            .find(|entry| {
                read_u64_at(entry, EXTENT_UPPER_LEAF_ENTRY_INODE_OFFSET) == FIRST_INODE_NUMBER
            })
            .expect("上段叶里有第一个文件那一条");
        first_file_entry[EXTENT_UPPER_LEAF_ENTRY_TAG_OFFSET] = tag;
    });
}

/// 只有一个单元（3000 字节）的第一个文件截到 100 的意图那一版：截断条目挂着，size 之外一个单元都没有（下一步是放 0 个单元的最后一批）。
fn image_after_the_intent_of_truncating_the_only_data_unit(tag: &str) -> MemoryPool {
    let mut pool = build_pool(tag);
    let intent = change_the_size_of_the_first_file_and_advance(&mut pool, SIZE_INSIDE_UNIT_ZERO);
    assert!(
        truncation_entry_of(&intent).is_some(),
        "意图那一版挂着截断条目"
    );
    assert_eq!(intent.data_pointers.len(), 1, "size 之外没有单元");
    pool.memory_pool()
}

/// 三个单元（2.5 个净荷）的第一个文件截到 100 的意图那一版（批大小 1）：截断条目挂着，size 之外还挂着两个单元。
fn image_after_the_intent_of_truncating_a_three_unit_file(tag: &str) -> MemoryPool {
    let mut pool = pool_with_a_three_unit_file(tag);
    let intent = change_the_size_of_the_first_file_and_advance(&mut pool, SIZE_INSIDE_UNIT_ZERO);
    assert!(
        truncation_entry_of(&intent).is_some(),
        "意图那一版挂着截断条目"
    );
    assert_eq!(intent.data_pointers.len(), 3, "size 之外还挂着两个单元");
    pool.memory_pool()
}

// ─── 合法世界：真写路径 ───

/// 三个单元截到 100、批大小 1：意图（size 之外挂着两个）、一批（挂着一个）、最后一批（删条目）三版各判，每一条都不红；
/// I-4.4 判截断条目、I-4.10、I-4.6、I-9.38、I-1.13 在意图那一版与那一批之后都真判过且成立。
#[test]
fn every_version_of_a_truncation_in_batches_of_one_holds_every_invariant() {
    let mut pool = pool_with_a_three_unit_file("checker-truncation-batches-of-one");
    change_the_size_of_the_first_file_and_advance(&mut pool, SIZE_INSIDE_UNIT_ZERO);
    let judged_on_the_intermediate_versions = [
        INTENT_DESCRIBES_ONLY_THE_TARGET_STATE,
        DATA_UNITS_BEYOND_THE_SIZE_ARE_NAMED_BY_A_TRUNCATION_INTENT,
        DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
        DECLARED_LENGTH_OF_EACH_DATA_UNIT_WITHIN_THE_SIZE_MATCHES_THE_SIZE,
        SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
    ];
    let intent_image = pool.memory_pool();
    assert_holds_every_invariant(&intent_image, "意图那一版");
    assert_judged_and_holding(
        &intent_image,
        &judged_on_the_intermediate_versions,
        "意图那一版",
    );
    let batch = publish_the_next_truncation_step_and_advance(&mut pool, batches_of_one());
    assert!(
        truncation_entry_of(&batch).is_some(),
        "那一批之后条目还挂着"
    );
    let batch_image = pool.memory_pool();
    assert_holds_every_invariant(&batch_image, "一批之后");
    assert_judged_and_holding(
        &batch_image,
        &judged_on_the_intermediate_versions,
        "一批之后",
    );
    let final_batch = publish_the_next_truncation_step_and_advance(&mut pool, batches_of_one());
    assert!(
        truncation_entry_of(&final_batch).is_none(),
        "最后一批删条目"
    );
    assert_eq!(final_batch.data_pointers.len(), 1, "只剩单元 0");
    assert_holds_every_invariant(&pool.memory_pool(), "最后一批之后");
}

/// 截在唯一那个单元之内（3000 → 100）的意图那一版：条目挂着、size 之外一个单元都没有，I-4.10 只判「size 之外有单元 ⇒ 有条目」
/// 那一个方向，这一版成立；I-4.4 判截断条目成立。
#[test]
fn a_truncation_entry_without_data_units_beyond_the_size_holds_both_intent_invariants() {
    let image =
        image_after_the_intent_of_truncating_the_only_data_unit("checker-truncation-only-unit");
    assert_holds_every_invariant(&image, "截在唯一单元之内的意图那一版");
    assert_judged_and_holding(
        &image,
        &[
            INTENT_DESCRIBES_ONLY_THE_TARGET_STATE,
            DATA_UNITS_BEYOND_THE_SIZE_ARE_NAMED_BY_A_TRUNCATION_INTENT,
            DECLARED_LENGTH_OF_EACH_DATA_UNIT_WITHIN_THE_SIZE_MATCHES_THE_SIZE,
        ],
        "截在唯一单元之内的意图那一版",
    );
}

/// 三个单元变长到 5 个净荷：最后一个单元补零重写到满、之后到 size 是洞，上段叶条目标签 3；每一条都不红（I-1.1 收标签 3，
/// I-4.6 认标签 3 少于 ⌈size ÷ 净荷⌉ 个单元）。
#[test]
fn an_extension_past_a_lower_segment_with_tag_three_holds_every_invariant() {
    let mut pool = pool_with_a_three_unit_file("checker-extension-tag-three");
    let extended = change_the_size_of_the_first_file_and_advance(&mut pool, 5 * payload_capacity());
    assert_eq!(upper_leaf_tag_of_the_first_file(&extended), 3, "标签 3");
    let image = pool.memory_pool();
    assert_holds_every_invariant(&image, "变长留洞（标签 3）之后");
    assert_judged_and_holding(
        &image,
        &[
            DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
            DECLARED_LENGTH_OF_EACH_DATA_UNIT_WITHIN_THE_SIZE_MATCHES_THE_SIZE,
            SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
        ],
        "变长留洞（标签 3）之后",
    );
}

/// 一个单元的文件变长到 3 个净荷加 5 字节（标签 4，之后是洞）再发 unlink 意图：nlink = 0、size 不变、单元少于 size 要的，
/// I-4.6 照标签 4 判成立（判决 K7 T-3 的合法世界），每一条都不红。
#[test]
fn the_unlink_intent_of_a_file_with_a_hole_holds_the_data_unit_count() {
    let mut pool = build_pool("checker-unlink-intent-with-a-hole");
    let extended =
        change_the_size_of_the_first_file_and_advance(&mut pool, 3 * payload_capacity() + 5);
    assert_eq!(upper_leaf_tag_of_the_first_file(&extended), 4, "标签 4");
    publish_the_intent_of(&mut pool, FIRST_INODE_NUMBER);
    let image = pool.memory_pool();
    assert_holds_every_invariant(&image, "有洞文件的 unlink 意图那一版");
    assert_judged_and_holding(
        &image,
        &[DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE],
        "有洞文件的 unlink 意图那一版",
    );
}

// ─── T-1：I-4.4 收截断条目 ───

/// 截断条目的补零里塞「再放 5 个」：只红 I-4.4。
#[test]
fn a_truncation_entry_carrying_an_increment_in_its_padding_reddens_only_the_target_state_invariant()
{
    let mut image =
        image_after_the_intent_of_truncating_the_only_data_unit("checker-truncation-entry-padding");
    change_the_truncation_entry(&mut image, |entry| {
        entry[TRUNCATION_INTENT_LAST_PADDING_BYTE_OFFSET] = 5;
    });
    let violations = violations_of(&image);
    assert_eq!(
        violated_names(&violations),
        [INTENT_DESCRIBES_ONLY_THE_TARGET_STATE]
    );
    assert!(detail_of(&violations, INTENT_DESCRIBES_ONLY_THE_TARGET_STATE).contains("补零"));
}

/// 截断条目的种类码改成 3（登记表里没有）：只红 I-4.4。
#[test]
fn a_logged_operation_of_kind_three_reddens_only_the_target_state_invariant() {
    let mut image =
        image_after_the_intent_of_truncating_the_only_data_unit("checker-truncation-entry-kind");
    change_the_truncation_entry(&mut image, |entry| {
        entry[..2].copy_from_slice(&UNREGISTERED_LOGGED_OPERATION_KIND.to_le_bytes());
    });
    let violations = violations_of(&image);
    assert_eq!(
        violated_names(&violations),
        [INTENT_DESCRIBES_ONLY_THE_TARGET_STATE]
    );
    assert!(detail_of(&violations, INTENT_DESCRIBES_ONLY_THE_TARGET_STATE).contains("种类码 3"));
}

/// logged_ops 里再加一条同样内容、签发 txg 早一的截断条目（key 照样严格递增）：全池两条截断条目，只红 I-4.4。
#[test]
fn two_truncation_entries_redden_only_the_target_state_invariant() {
    let mut image =
        image_after_the_intent_of_truncating_the_only_data_unit("checker-two-truncation-entries");
    rewrite_the_logged_operations_entries(&mut image, |entries| {
        assert_eq!(entries.len(), 1, "logged_ops 里只有那一条截断条目");
        let mut earlier = entries[0].clone();
        let issued_txg = read_u64_at(&earlier, LOGGED_OPERATION_ISSUED_TXG_OFFSET);
        earlier[LOGGED_OPERATION_ISSUED_TXG_OFFSET..LOGGED_OPERATION_KEY_BYTES]
            .copy_from_slice(&(issued_txg - 1).to_le_bytes());
        entries.insert(0, earlier);
    });
    let violations = violations_of(&image);
    assert_eq!(
        violated_names(&violations),
        [INTENT_DESCRIBES_ONLY_THE_TARGET_STATE]
    );
    assert!(
        detail_of(&violations, INTENT_DESCRIBES_ONLY_THE_TARGET_STATE).contains("2 条截断条目")
    );
}

/// 截断条目的目标 size 改成 101（记录的 size 是 100）：只红 I-4.4。
#[test]
fn a_truncation_entry_whose_target_differs_from_the_size_reddens_only_the_target_state_invariant() {
    let mut image =
        image_after_the_intent_of_truncating_the_only_data_unit("checker-truncation-entry-target");
    change_the_truncation_entry(&mut image, |entry| {
        entry[TRUNCATION_INTENT_TARGET_SIZE_OFFSET..TRUNCATION_INTENT_TARGET_SIZE_OFFSET + 8]
            .copy_from_slice(&(SIZE_INSIDE_UNIT_ZERO + 1).to_le_bytes());
    });
    let violations = violations_of(&image);
    assert_eq!(
        violated_names(&violations),
        [INTENT_DESCRIBES_ONLY_THE_TARGET_STATE]
    );
    assert!(
        detail_of(&violations, INTENT_DESCRIBES_ONLY_THE_TARGET_STATE).contains("目标 size 101")
    );
}

/// 截断条目的对象出生代加一（与记录的不符）：只红 I-4.4。
#[test]
fn a_truncation_entry_whose_object_birth_differs_from_the_record_reddens_only_the_target_state_invariant(
) {
    let mut image =
        image_after_the_intent_of_truncating_the_only_data_unit("checker-truncation-entry-birth");
    change_the_truncation_entry(&mut image, |entry| {
        let object_birth = read_u64_at(entry, TRUNCATION_INTENT_OBJECT_BIRTH_OFFSET);
        entry[TRUNCATION_INTENT_OBJECT_BIRTH_OFFSET..TRUNCATION_INTENT_OBJECT_BIRTH_OFFSET + 8]
            .copy_from_slice(&(object_birth + 1).to_le_bytes());
    });
    let violations = violations_of(&image);
    assert_eq!(
        violated_names(&violations),
        [INTENT_DESCRIBES_ONLY_THE_TARGET_STATE]
    );
    assert!(
        detail_of(&violations, INTENT_DESCRIBES_ONLY_THE_TARGET_STATE).contains("对象出生代相等")
    );
}

/// 截断条目的签发 txg 往后挪 5（晚于它所在那一版的 txg，key 照样只有一条）：只红 I-4.4。
#[test]
fn a_truncation_entry_issued_after_its_version_reddens_only_the_target_state_invariant() {
    let mut image =
        image_after_the_intent_of_truncating_the_only_data_unit("checker-truncation-entry-issued");
    change_the_truncation_entry(&mut image, |entry| {
        let issued_txg = read_u64_at(entry, LOGGED_OPERATION_ISSUED_TXG_OFFSET);
        entry[LOGGED_OPERATION_ISSUED_TXG_OFFSET..LOGGED_OPERATION_KEY_BYTES]
            .copy_from_slice(&(issued_txg + 5).to_le_bytes());
    });
    let violations = violations_of(&image);
    assert_eq!(
        violated_names(&violations),
        [INTENT_DESCRIBES_ONLY_THE_TARGET_STATE]
    );
    assert!(detail_of(&violations, INTENT_DESCRIBES_ONLY_THE_TARGET_STATE).contains("晚于这一版"));
}

// ─── T-2：I-4.10 截断中间态有账 ───

/// 三个单元截到 100 的意图那一版上删掉截断条目、留着 size 之外的两个单元（「最后一批与删条目同事务」改坏成「删条目而不放单元」的样子）：
/// 红 I-4.10，单元数对不上标签（没有条目时标签 1 要恰好 1 个）也红 I-4.6。
#[test]
fn deleting_the_truncation_entry_while_units_beyond_the_size_remain_reddens_the_intermediate_state_account(
) {
    let mut image =
        image_after_the_intent_of_truncating_a_three_unit_file("checker-truncation-entry-deleted");
    rewrite_the_logged_operations_entries(&mut image, |entries| {
        assert_eq!(entries.len(), 1, "logged_ops 里只有那一条截断条目");
        entries.clear();
    });
    let violations = violations_of(&image);
    assert_eq!(
        violated_names(&violations),
        [
            DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
            DATA_UNITS_BEYOND_THE_SIZE_ARE_NAMED_BY_A_TRUNCATION_INTENT,
        ]
    );
    assert!(detail_of(
        &violations,
        DATA_UNITS_BEYOND_THE_SIZE_ARE_NAMED_BY_A_TRUNCATION_INTENT
    )
    .contains("size 之外 2 个"));
}

// ─── T-3：I-4.6 单元数对 size 与标签 ───

/// 三个单元的文件拿掉单元 1 那条 extent 叶记录：存在的单元 {0, 2} 不连号，I-4.6 红（连号那一句）。
#[test]
fn a_missing_middle_data_unit_reddens_the_data_unit_count_on_the_sequence() {
    let mut image = pool_with_a_three_unit_file("checker-missing-middle-unit").memory_pool();
    drop_the_extent_record_of_the_first_file_at_unit(&mut image, 1);
    let violations = violations_of(&image);
    assert!(
        detail_of(
            &violations,
            DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE
        )
        .contains("不从 0 起连号"),
        "{violations:?}"
    );
}

/// 三个单元的文件拿掉尾部单元 2 那条 extent 叶记录（尾单元被错放）：标签 1 却只有 2 个、size 要 3 个，I-4.6 红。
#[test]
fn tag_one_with_its_tail_data_unit_missing_reddens_the_data_unit_count() {
    let mut image = pool_with_a_three_unit_file("checker-tag-one-tail-missing").memory_pool();
    drop_the_extent_record_of_the_first_file_at_unit(&mut image, 2);
    let violations = violations_of(&image);
    assert!(
        detail_of(
            &violations,
            DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE
        )
        .contains("恰好 3 个"),
        "{violations:?}"
    );
}

/// 三个单元的文件（单元数恰是 size 要的）上段叶条目标签 1 改成 3：声明之后是洞、却一个洞单元都没有，只红 I-4.6。
#[test]
fn tag_three_without_a_hole_reddens_only_the_data_unit_count() {
    let mut image = pool_with_a_three_unit_file("checker-tag-three-without-a-hole").memory_pool();
    set_the_upper_leaf_tag_of_the_first_file(
        &mut image,
        UPPER_LEAF_TAG_LOWER_SEGMENT_ROOT_FOLLOWED_BY_A_HOLE,
    );
    let violations = violations_of(&image);
    assert_eq!(
        violated_names(&violations),
        [DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE]
    );
    assert!(detail_of(
        &violations,
        DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE
    )
    .contains("少于 3 个"));
}

/// nlink ≥ 1 的三个单元的文件 size 改成 2 个净荷（blocks 跟着改）、没有截断条目：多出一个单元，I-4.6（标签 1 要恰好 2 个）与 I-4.10 红，
/// 单元 1 的声明长度照样是满的、I-9.38 不红。
#[test]
fn a_live_file_with_one_data_unit_beyond_its_size_and_no_entry_reddens_the_count_and_the_account() {
    let mut image = pool_with_a_three_unit_file("checker-live-file-unit-beyond-size").memory_pool();
    set_the_size_of_the_first_file(&mut image, 2 * payload_capacity());
    let violations = violations_of(&image);
    assert_eq!(
        violated_names(&violations),
        [
            DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
            DATA_UNITS_BEYOND_THE_SIZE_ARE_NAMED_BY_A_TRUNCATION_INTENT,
        ]
    );
    assert!(detail_of(
        &violations,
        DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE
    )
    .contains("恰好 2 个"));
}

// ─── T-4：I-9.38 声明长度对 size ───

/// 一个单元（3000 字节）的文件只把记录的 size 改成 100、不重写尾单元（截到单元中间的错写法）：只红 I-9.38。
#[test]
fn shrinking_the_size_into_the_unit_without_rewriting_it_reddens_only_the_declared_length() {
    let mut image = build_pool("checker-size-shrunk-without-rewrite").memory_pool();
    set_the_size_of_the_first_file(&mut image, SIZE_INSIDE_UNIT_ZERO);
    let violations = violations_of(&image);
    assert_eq!(
        violated_names(&violations),
        [DECLARED_LENGTH_OF_EACH_DATA_UNIT_WITHIN_THE_SIZE_MATCHES_THE_SIZE]
    );
    assert!(detail_of(
        &violations,
        DECLARED_LENGTH_OF_EACH_DATA_UNIT_WITHIN_THE_SIZE_MATCHES_THE_SIZE
    )
    .contains("声明长度 3000"));
}

/// 一个单元（3000 字节）的文件只把记录的 size 改成 5000、不重写尾单元（单元内变长的错写法）：只红 I-9.38。
#[test]
fn extending_the_size_within_the_unit_without_rewriting_it_reddens_only_the_declared_length() {
    let mut image = build_pool("checker-size-extended-without-rewrite").memory_pool();
    set_the_size_of_the_first_file(&mut image, 5000);
    let violations = violations_of(&image);
    assert_eq!(
        violated_names(&violations),
        [DECLARED_LENGTH_OF_EACH_DATA_UNIT_WITHIN_THE_SIZE_MATCHES_THE_SIZE]
    );
    assert!(detail_of(
        &violations,
        DECLARED_LENGTH_OF_EACH_DATA_UNIT_WITHIN_THE_SIZE_MATCHES_THE_SIZE
    )
    .contains("= 5000"));
}
