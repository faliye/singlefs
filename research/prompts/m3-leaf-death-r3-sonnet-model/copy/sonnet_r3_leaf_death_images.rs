//! 正推腿副本（m3-leaf-death-r3 sonnet，只在草稿目录的仓副本里跑）：真 checker 对一族手改镜像判什么。每份镜像按两种 checker 各打一行 `SONR3 ...`：
//! today = 冻结树原样；third_clause_deleted = 只把 I-9.4 第三句（左容器最大 key < 右容器号）判成恒真（环境变量 R3_I94_THIRD_OFF，副本里的补丁开关）。
//! 不断言（判定交给 checker）。读写镜像的函数照第二轮攻方 `copy/opus_r2_leaf_death_images.rs` 与第一轮攻方 `k3-copy/opus_k3_borrowed_leaf_images.rs` 重写。
mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use common_deletion_in_progress::{change_the_unit_and_propagate_its_checksum, pool_with_new_inodes, violated_invariants};
use common_pointer_head_corruption::{data_unit_bytes, node_bytes, the_slot_of, tree_table_entry_of_kind, unit_bytes_at};
use singlefs_core::address::SlotNumber;
use singlefs_core::bytes::ByteReader;
use singlefs_core::pointer::NodePointer;
use singlefs_core::records::TREE_KIND_INODE;
use singlefs_core::unit::parse_index_node;
use singlefs_harness::memory_pool::MemoryPool;

const RECORD_COUNT_OFFSET: usize = 69;
const DECLARED_LENGTH_OFFSET: usize = 8;
const RECORDS_START: usize = 136;
const INODE_RECORD_BYTES: usize = 140;
const CONTAINER_NUMBER_OFFSET: usize = 53;
const ROOT_ENTRIES_START: usize = 131;
const ENTRY_BYTES: usize = 120;
const ENTRY_CHILD_POINTER: usize = 34;
const ENTRY_IDENTITY_CONTAINER: usize = 18;

fn get_u16(b: &[u8], o: usize) -> u16 { u16::from_le_bytes(b[o..o + 2].try_into().unwrap()) }
fn get_u64(b: &[u8], o: usize) -> u64 { u64::from_le_bytes(b[o..o + 8].try_into().unwrap()) }
fn set_u16(b: &mut [u8], o: usize, v: u16) { b[o..o + 2].copy_from_slice(&v.to_le_bytes()); }
fn set_u64(b: &mut [u8], o: usize, v: u64) { b[o..o + 8].copy_from_slice(&v.to_le_bytes()); }

fn inode_root_slot(image: &MemoryPool) -> SlotNumber { the_slot_of(&tree_table_entry_of_kind(image, TREE_KIND_INODE).root) }

fn leaf_slots(image: &MemoryPool) -> Vec<SlotNumber> {
    let root = parse_index_node(&unit_bytes_at(image, inode_root_slot(image), node_bytes())).expect("inode 树根");
    root.entries.iter().map(|e| NodePointer::read_from(&mut ByteReader::at(e, ENTRY_CHILD_POINTER)).locations[0].slot).collect()
}

fn records_of(image: &MemoryPool, slot: SlotNumber) -> Vec<Vec<u8>> {
    let leaf = unit_bytes_at(image, slot, data_unit_bytes());
    (0..usize::from(get_u16(&leaf, RECORD_COUNT_OFFSET)))
        .map(|i| leaf[RECORDS_START + i * INODE_RECORD_BYTES..RECORDS_START + (i + 1) * INODE_RECORD_BYTES].to_vec()).collect()
}

fn rewrite_leaf(image: &mut MemoryPool, slot: SlotNumber, records: &[Vec<u8>]) {
    change_the_unit_and_propagate_its_checksum(image, slot, data_unit_bytes(), |leaf| {
        leaf[RECORDS_START..].fill(0);
        for (i, r) in records.iter().enumerate() {
            leaf[RECORDS_START + i * INODE_RECORD_BYTES..RECORDS_START + (i + 1) * INODE_RECORD_BYTES].copy_from_slice(r);
        }
        set_u16(leaf, RECORD_COUNT_OFFSET, u16::try_from(records.len()).unwrap());
        set_u16(leaf, DECLARED_LENGTH_OFFSET, u16::try_from(records.len() * INODE_RECORD_BYTES).unwrap());
    });
}

fn set_separator(image: &mut MemoryPool, index: usize, separator: u64) {
    let slot = inode_root_slot(image);
    change_the_unit_and_propagate_its_checksum(image, slot, node_bytes(), |node| set_u64(node, ROOT_ENTRIES_START + index * ENTRY_BYTES, separator));
}

/// 容器号：叶头里那份（偏移 53）与父条目身份引用里那份（条目偏移 18）一起改，I-9.2 才不先红。
fn set_container_number(image: &mut MemoryPool, index: usize, number: u64) {
    let slot = leaf_slots(image)[index];
    change_the_unit_and_propagate_its_checksum(image, slot, data_unit_bytes(), |leaf| set_u64(leaf, CONTAINER_NUMBER_OFFSET, number));
    let root = inode_root_slot(image);
    change_the_unit_and_propagate_its_checksum(image, root, node_bytes(), |node| set_u64(node, ROOT_ENTRIES_START + index * ENTRY_BYTES + ENTRY_IDENTITY_CONTAINER, number));
}

fn with_inode(record: &[u8], inode: u64) -> Vec<u8> { let mut r = record.to_vec(); set_u64(&mut r, 0, inode); r }

fn shape_of(image: &MemoryPool) -> String {
    let slots = leaf_slots(image);
    let root = unit_bytes_at(image, inode_root_slot(image), node_bytes());
    let mut s = String::new();
    for (i, slot) in slots.iter().enumerate() {
        let leaf = unit_bytes_at(image, *slot, data_unit_bytes());
        let recs = records_of(image, *slot);
        let range = if recs.is_empty() { "empty".to_string() } else { format!("{}..{}", get_u64(&recs[0], 0), get_u64(recs.last().unwrap(), 0)) };
        s.push_str(&format!("[no={} sep={} n={} {}]", get_u64(&leaf, CONTAINER_NUMBER_OFFSET), get_u64(&root, ROOT_ENTRIES_START + i * ENTRY_BYTES), recs.len(), range));
    }
    s
}

fn judged(label: &str, extra: &str, image: &MemoryPool) {
    for variant in ["today", "third_clause_deleted"] {
        if variant == "today" { std::env::remove_var("R3_I94_THIRD_OFF"); } else { std::env::set_var("R3_I94_THIRD_OFF", "1"); }
        println!("SONR3 image={label} {extra} leaves={} checker={variant} violated={:?}", shape_of(image), violated_invariants(image));
    }
    std::env::remove_var("R3_I94_THIRD_OFF");
}

#[test]
fn sonnet_r3_shape_of_the_clean_pool() {
    let clean = pool_with_new_inodes("sonnet-r3-shape", 466).memory_pool();
    judged("clean", "", &clean);
}

/// K3：C116 那一族「分裂出来的右半容器号给错」。第二片（真值：容器号 234 = 触发分裂的那条新记录的 inode 号，记录 234..466）的容器号取一族值，
/// 分隔 key 两种约定——T：分隔 key = 容器号（今天）；N：分隔 key = 孩子这一版的最小 key（N1 新形，234）。
#[test]
fn sonnet_r3_right_half_container_number_family() {
    let clean = pool_with_new_inodes("sonnet-r3-right-half", 466).memory_pool();
    for convention in ["T_separator_is_the_container_number", "N_separator_is_the_childs_smallest_key"] {
        for number in [0u64, 1, 2, 17, 100, 233, 234, 235, 300, 466, 467] {
            let mut image = clean.clone();
            set_container_number(&mut image, 1, number);
            set_separator(&mut image, 1, if convention.starts_with('T') { number } else { 234 });
            judged(&format!("right_half_number_{number}"), &format!("separator_convention={convention}"), &image);
        }
    }
}

/// 候选二（零条目叶一直留）的结构：删空 / 吸收之后留一片零条目叶，分隔 key 要放得下。
/// 前缀：左片删到 100 条（1..100）、中片删到 17 条（234..250）、右片 467。
#[test]
fn sonnet_r3_zero_entry_leaf_family() {
    let clean = pool_with_new_inodes("sonnet-r3-stub", 466).memory_pool();
    let s = leaf_slots(&clean);
    let (a, b, c) = (records_of(&clean, s[0]), records_of(&clean, s[1]), records_of(&clean, s[2]));
    let mut base = clean.clone();
    rewrite_leaf(&mut base, s[0], &a[..100]);
    rewrite_leaf(&mut base, s[1], &b[..17]);
    judged("prefix_left_100_middle_17", "", &base);
    // 删空（零条目叶留在叶序里，容器号、分隔 key 照旧）
    let mut image = base.clone();
    rewrite_leaf(&mut image, s[1], &[]);
    judged("drop_leaves_a_zero_entry_leaf", "", &image);
    // 左吸收右：左片 1..100 + 234..250，中片留零条目；分隔 key 三种取法
    for (label, sep) in [("separator_kept_234", 234u64), ("separator_raised_to_251_with_room", 251)] {
        let mut image = base.clone();
        let mut merged = a[..100].to_vec();
        merged.extend(b[..17].iter().cloned());
        rewrite_leaf(&mut image, s[0], &merged);
        rewrite_leaf(&mut image, s[1], &[]);
        set_separator(&mut image, 1, sep);
        judged(&format!("absorb_leaves_a_zero_entry_leaf_{label}"), "", &image);
    }
    // 没有空隙：右片从 251 起（容器号 251、分隔 key 251、记录号 251），零条目叶的分隔 key 取左片最大 key + 1 = 251 与右片相等
    let mut image = base.clone();
    let mut merged = a[..100].to_vec();
    merged.extend(b[..17].iter().cloned());
    rewrite_leaf(&mut image, s[0], &merged);
    rewrite_leaf(&mut image, s[1], &[]);
    rewrite_leaf(&mut image, s[2], &[with_inode(&c[0], 251)]);
    set_container_number(&mut image, 2, 251);
    set_separator(&mut image, 2, 251);
    set_separator(&mut image, 1, 251);
    judged("absorb_leaves_a_zero_entry_leaf_no_room_for_a_separator", "", &image);
}
