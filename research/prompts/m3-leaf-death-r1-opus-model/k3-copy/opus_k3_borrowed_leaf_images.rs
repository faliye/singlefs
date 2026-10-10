//! 攻方副本（m3-leaf-death-r1 opus K3，只在草稿目录的仓副本里）：左从右借之后的镜像，与换了 I-9.4 第三句之后会被放过的
//! 「右半容器号给小了」镜像。每份镜像打一行 `OPUSK3 ...`，不断言（同一个二进制在设与不设 `OPUS_K3_I94_N1` 下各跑一遍比）。
mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use common_deletion_in_progress::{
    change_the_unit_and_propagate_its_checksum, pool_with_new_inodes, violated_invariants,
};
use common_pointer_head_corruption::{
    data_unit_bytes, node_bytes, the_slot_of, tree_table_entry_of_kind, unit_bytes_at,
};
use singlefs_core::address::SlotNumber;
use singlefs_core::bytes::ByteReader;
use singlefs_core::pointer::NodePointer;
use singlefs_core::records::TREE_KIND_INODE;
use singlefs_core::unit::parse_index_node;
use singlefs_harness::memory_pool::MemoryPool;

const RECORD_COUNT_OFFSET: usize = 69;
const DECLARED_LENGTH_OFFSET: usize = 8;
const RECORDS_START: usize = 136;
const RECORD_BYTES: usize = 140;
/// 码 3 头里容器号（`checker_known_bad_images.rs` 的 I-9.4 坏镜像改的就是偏移 53）。
const CONTAINER_NUMBER_OFFSET: usize = 53;
/// inode 树根（码 2，key 宽 8）条目区从 131 起，一条 120：分隔 key 0、身份引用里的容器号 18、子指针 34。
const ROOT_ENTRIES_START: usize = 131;
const ENTRY_BYTES: usize = 120;
const ENTRY_IDENTITY_CONTAINER: usize = 18;
const ENTRY_CHILD_POINTER: usize = 34;

fn get_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().expect("2"))
}
fn get_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8"))
}
fn set_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}
fn set_u64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

fn inode_root_slot(image: &MemoryPool) -> SlotNumber {
    the_slot_of(&tree_table_entry_of_kind(image, TREE_KIND_INODE).root)
}

fn leaf_slots(image: &MemoryPool) -> Vec<SlotNumber> {
    let root = parse_index_node(&unit_bytes_at(image, inode_root_slot(image), node_bytes()))
        .expect("inode 树根解得开");
    assert_eq!(root.level, 1, "根直接管叶容器");
    root.entries
        .iter()
        .map(|entry| {
            let child = NodePointer::read_from(&mut ByteReader::at(entry, ENTRY_CHILD_POINTER));
            child.locations[0].slot
        })
        .collect()
}

fn records_of(image: &MemoryPool, slot: SlotNumber) -> Vec<Vec<u8>> {
    let leaf = unit_bytes_at(image, slot, data_unit_bytes());
    let count = usize::from(get_u16(&leaf, RECORD_COUNT_OFFSET));
    (0..count)
        .map(|index| leaf[RECORDS_START + index * RECORD_BYTES..RECORDS_START + (index + 1) * RECORD_BYTES].to_vec())
        .collect()
}

fn rewrite_leaf(image: &mut MemoryPool, slot: SlotNumber, records: &[Vec<u8>]) {
    change_the_unit_and_propagate_its_checksum(image, slot, data_unit_bytes(), |leaf| {
        leaf[RECORDS_START..].fill(0);
        for (index, record) in records.iter().enumerate() {
            let start = RECORDS_START + index * RECORD_BYTES;
            leaf[start..start + RECORD_BYTES].copy_from_slice(record);
        }
        set_u16(leaf, RECORD_COUNT_OFFSET, u16::try_from(records.len()).expect("≤ 233"));
        set_u16(leaf, DECLARED_LENGTH_OFFSET, u16::try_from(records.len() * RECORD_BYTES).expect("≤ 32620"));
    });
}

fn set_root_entry(image: &mut MemoryPool, index: usize, separator: Option<u64>, container: Option<u64>) {
    let root_slot = inode_root_slot(image);
    change_the_unit_and_propagate_its_checksum(image, root_slot, node_bytes(), |node| {
        let start = ROOT_ENTRIES_START + index * ENTRY_BYTES;
        if let Some(separator) = separator {
            set_u64(node, start, separator);
        }
        if let Some(container) = container {
            set_u64(node, start + ENTRY_IDENTITY_CONTAINER, container);
        }
    });
}

fn set_leaf_container_number(image: &mut MemoryPool, slot: SlotNumber, number: u64) {
    change_the_unit_and_propagate_its_checksum(image, slot, data_unit_bytes(), |leaf| {
        set_u64(leaf, CONTAINER_NUMBER_OFFSET, number);
    });
}

fn inode_numbers(records: &[Vec<u8>]) -> (u64, u64, usize) {
    (get_u64(&records[0], 0), get_u64(records.last().expect("非空"), 0), records.len())
}

fn report(name: &str, image: &MemoryPool) {
    let slots = leaf_slots(image);
    let shapes: Vec<String> = slots
        .iter()
        .map(|slot| {
            let leaf = unit_bytes_at(image, *slot, data_unit_bytes());
            let (smallest, largest, count) = inode_numbers(&records_of(image, *slot));
            format!("[no={} {smallest}..{largest} n={count}]", get_u64(&leaf, CONTAINER_NUMBER_OFFSET))
        })
        .collect();
    let root = unit_bytes_at(image, inode_root_slot(image), node_bytes());
    let separators: Vec<u64> = (0..slots.len()).map(|index| get_u64(&root, ROOT_ENTRIES_START + index * ENTRY_BYTES)).collect();
    println!(
        "OPUSK3 image={name} n1_rule={} leaves={} separators={:?} violated={:?}",
        std::env::var_os("OPUS_K3_I94_N1").is_some(),
        shapes.join(""),
        separators,
        violated_invariants(image)
    );
}

/// 1 号（新池新建文件）+ 466 个空 inode：三片叶 [1..233]、[234..466]、[467]。
fn three_leaves() -> MemoryPool {
    pool_with_new_inodes("opus-k3-borrowed-leaf", 466).memory_pool()
}

/// 左片删到只剩 [1..100]（M1 删的结果，101..233 当作已删完），右片不动。
fn after_deleting_101_to_233(clean: &MemoryPool) -> MemoryPool {
    let mut image = clean.clone();
    let slots = leaf_slots(&image);
    let left = records_of(&image, slots[0]);
    rewrite_leaf(&mut image, slots[0], &left[..100]);
    image
}

/// 在 `deleted` 上照 M3「左从右借」：右片最小的 `moved` 条接到左片末尾，两片都保留身份；`update_separator` 时右片的分隔 key
/// 改成借完之后的最小 key（否则照 `inode_tree.rs` 的 `separator_key` = 容器号）。
fn after_borrowing(deleted: &MemoryPool, moved: usize, update_separator: bool) -> MemoryPool {
    let mut image = deleted.clone();
    let slots = leaf_slots(&image);
    let mut left = records_of(&image, slots[0]);
    let right = records_of(&image, slots[1]);
    left.extend(right[..moved].iter().cloned());
    rewrite_leaf(&mut image, slots[0], &left);
    let slots = leaf_slots(&image);
    rewrite_leaf(&mut image, slots[1], &right[moved..]);
    if update_separator {
        let new_smallest = get_u64(&right[moved], 0);
        set_root_entry(&mut image, 1, Some(new_smallest), None);
    }
    image
}

/// 第三片（[467]）的容器号改成 `number`（落在中间那片的 key 区间里，大于中间那片的容器号），根条目身份引用跟着改；
/// `separator_follows` 时分隔 key 也改成它（`crates/` 的 `separator_key` 取容器号），否则分隔 key 留 467。
fn third_leaf_numbered(clean: &MemoryPool, number: u64, separator_follows: bool) -> MemoryPool {
    let mut image = clean.clone();
    let slots = leaf_slots(&image);
    set_leaf_container_number(&mut image, slots[2], number);
    set_root_entry(&mut image, 2, separator_follows.then_some(number), Some(number));
    image
}

#[test]
fn opus_k3_borrowed_and_misnumbered_leaf_images() {
    let clean = three_leaves();
    report("clean_three_leaves", &clean);
    let deleted = after_deleting_101_to_233(&clean);
    report("left_deleted_to_100", &deleted);
    for (label, moved) in [("half", 66_usize), ("just_enough", 17)] {
        report(&format!("borrow_{label}_separator_is_container_number"), &after_borrowing(&deleted, moved, false));
        report(&format!("borrow_{label}_separator_is_new_smallest"), &after_borrowing(&deleted, moved, true));
    }
    report("third_leaf_numbered_300_separator_467", &third_leaf_numbered(&clean, 300, false));
    report("third_leaf_numbered_300_separator_300", &third_leaf_numbered(&clean, 300, true));
}
