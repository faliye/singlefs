//! N1：树表两个方向都配上（每条 deleted_inodes、每棵 inode 树、每棵 extent 树都恰好配一条）而真有数据单元的那棵树没被当成 extent 树读的几形。
//! 底子与第 21 批 S1 用例同：新池新建文件之后建一个空 inode，再拿掉 1 号记录、留着它的单元（I-4.7 那一形坏镜像）。
use singlefs_core::records::{TREE_KIND_EXTENT, TREE_KIND_INODE, TREE_KIND_LIVELIST};
use singlefs_harness::memory_pool::MemoryPool;

use crate::common_deletion_in_progress::{
    change_the_unit_and_propagate_its_checksum, drop_the_record_of_the_first_file_and_keep_its_data_units,
    let_the_empty_livelist_entry_stand_in_for_the_extent_tree_in_the_tree_table_at, tree_table_slot_of_the_newest_root,
};
use crate::judge::judge;
use crate::pool::{content_of, SparsePool};

const ENTRIES_START: usize = 115 + 2 * 8;
const ENTRY_BYTES: usize = 200;
const KIND_OFFSET: usize = 10;
const HEAD_OFFSET: usize = 116;
const ENTRY_COUNT_OFFSET: usize = 82 + 2 * 8;

fn u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}

fn index_of_kind(table: &[u8], kind: u16) -> usize {
    let count = usize::from(u16_at(table, ENTRY_COUNT_OFFSET));
    let found: Vec<usize> = (0..count).filter(|index| u16_at(table, ENTRIES_START + index * ENTRY_BYTES + KIND_OFFSET) == kind).collect();
    assert_eq!(found.len(), 1, "种类 {kind} 恰好一条：{found:?}");
    found[0]
}

/// 真 extent 树那一条种类改成 `hide_as`、头 ID 改 0；根指针全零的 livelist 那一条种类改成 1、头 ID 指 inode 树。树 ID、根指针不动。
fn hide_the_extent_tree_as(image: &mut MemoryPool, hide_as: u16) {
    let slot = tree_table_slot_of_the_newest_root(image);
    change_the_unit_and_propagate_its_checksum(image, slot, 16384, |table| {
        let inode = index_of_kind(table, TREE_KIND_INODE);
        let inode_tree = u64::from_le_bytes(table[ENTRIES_START + inode * ENTRY_BYTES..][..8].try_into().expect("8"));
        let extent = index_of_kind(table, TREE_KIND_EXTENT);
        let livelist = index_of_kind(table, TREE_KIND_LIVELIST);
        let extent_at = ENTRIES_START + extent * ENTRY_BYTES;
        let livelist_at = ENTRIES_START + livelist * ENTRY_BYTES;
        assert!(table[livelist_at + 14..livelist_at + 100].iter().all(|byte| *byte == 0), "livelist 根指针全零");
        table[extent_at + KIND_OFFSET..][..2].copy_from_slice(&hide_as.to_le_bytes());
        table[extent_at + HEAD_OFFSET..][..8].copy_from_slice(&0u64.to_le_bytes());
        table[livelist_at + KIND_OFFSET..][..2].copy_from_slice(&TREE_KIND_EXTENT.to_le_bytes());
        table[livelist_at + HEAD_OFFSET..][..8].copy_from_slice(&inode_tree.to_le_bytes());
    });
}

fn report(name: &str, image: &MemoryPool) {
    let judged = judge(image);
    let watched: Vec<String> = judged.watched.iter().map(|(invariant, kind, text)| format!("{invariant}={kind}({})", text.chars().take(60).collect::<String>())).collect();
    let violated: Vec<&str> = judged.violated.iter().map(|(invariant, _)| *invariant).collect();
    println!("SHAPE name={name} violated={violated:?} watched={}", watched.join(" ").replace(' ', "_"));
}

pub fn run() {
    let mut pool = SparsePool::build(4 << 30, &content_of(3000, 0));
    pool.new_inodes(1);
    let clean = pool.published.last().expect("建 inode 那一版").image.clone();
    report("clean", &clean);
    let mut dropped = clean.clone();
    drop_the_record_of_the_first_file_and_keep_its_data_units(&mut dropped);
    report("drop-record", &dropped);
    let mut s1 = dropped.clone();
    let slot = tree_table_slot_of_the_newest_root(&s1);
    let_the_empty_livelist_entry_stand_in_for_the_extent_tree_in_the_tree_table_at(&mut s1, slot);
    report("drop-record+S1", &s1);
    for (name, hide_as) in [("drop-record+swap-extent-and-livelist-kinds", TREE_KIND_LIVELIST), ("drop-record+extent-as-unregistered-13", 13u16)] {
        let mut image = dropped.clone();
        hide_the_extent_tree_as(&mut image, hide_as);
        report(name, &image);
    }
    let mut clean_swap = clean.clone();
    hide_the_extent_tree_as(&mut clean_swap, TREE_KIND_LIVELIST);
    report("clean+swap-extent-and-livelist-kinds", &clean_swap);
}
