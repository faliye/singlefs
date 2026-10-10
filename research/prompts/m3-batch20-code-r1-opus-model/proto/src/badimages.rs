//! N3：第 20 批新登记的每份坏镜像换一种造法再造（换历史、换版本、换改法），看是不是仍只红目标、判不了的理由是不是仍是那一句。
//! 池都在内存稀疏盘上造（`pool::SparsePool`），改法用 `common_deletion_in_progress` 里那几个对 `MemoryPool` 动手的函数，或这里另写的。
use singlefs_core::address::SlotNumber;
use singlefs_core::inode_tree::InodeLeafContainerIndexInTree;
use singlefs_core::records::{TREE_KIND_DELETED_INODES, TREE_KIND_EXTENT, TREE_KIND_INODE, TREE_KIND_TOMBSTONE};
use singlefs_core::transaction::{TransactionOutput, TransactionUnit};
use singlefs_harness::memory_pool::MemoryPool;

use crate::common::{file_content, FIXED_WRITE_TIME_SECONDS};
use crate::common_deletion_in_progress as del;
use crate::common_pointer_head_corruption::{data_unit_bytes, node_bytes, the_slot_of, tree_table_entry_of_kind, unit_bytes_at};
use crate::judge::judge;
use crate::pool::SparsePool;

pub const DEV: u64 = 4 << 30;

pub fn three_unit_pool() -> SparsePool {
    let mut pool = SparsePool::build(DEV, &file_content());
    pool.sequential_write(&del::content_of_three_data_units(), FIXED_WRITE_TIME_SECONDS + 60);
    pool
}

pub fn record_time(pool: &SparsePool, inode: u64) -> u64 {
    pool.output
        .inode_leaf_containers
        .iter()
        .flat_map(|container| container.contents.records.iter())
        .find(|record| record.inode == inode)
        .map_or(FIXED_WRITE_TIME_SECONDS, |record| record.write_time_seconds)
}

pub fn intent(pool: &mut SparsePool, inode: u64) -> TransactionOutput {
    let time = record_time(pool, inode);
    pool.intent(inode, time).expect("意图")
}

/// 一个单元两份各翻一个字节（不重封、不补引用链）：两份都对不上整单元校验和。
pub fn flip_a_byte_of_both_copies(image: &mut MemoryPool, slot: SlotNumber, offset: usize) {
    for device in image.devices.values_mut() {
        let sector_start = offset / 512 * 512;
        let at = singlefs_core::address::DeviceOffsetInBytes(slot.to_device_offset().0 + sector_start as u64);
        let mut sector = device.read(at, 512);
        sector[offset - sector_start] ^= 0x5A;
        device.write(at, &sector);
    }
}

/// 只把盘 0 那一份清零：盘 1 那一份还读得出。
pub fn zero_the_copy_on_device_zero(image: &mut MemoryPool, slot: SlotNumber, length: usize) {
    let device = image.devices.get_mut(&singlefs_core::address::DeviceIdentity(0)).expect("盘 0");
    device.write(slot.to_device_offset(), &vec![0u8; length]);
}

pub enum Expect {
    /// 判红的恰好是这几条。
    OnlyRed(Vec<&'static str>),
    /// 判红的恰好是 `red` 这几条，`not_judged` 里每条报不适用、理由含 `reason`；`holds` 里每条成立。
    NotJudged { red: Vec<&'static str>, not_judged: Vec<&'static str>, reason: &'static str, holds: Vec<&'static str> },
    /// 这一条成立（不管别的）。
    Holds(&'static str),
}

pub fn report(case: &str, image: &MemoryPool, expect: Expect) -> bool {
    let judged = judge(image);
    let violated = judged.violated_names();
    for (invariant, text) in &judged.violated {
        println!("BAD_VIOLATION case=「{case}」 {invariant}：{}", text.chars().take(200).collect::<String>());
    }
    let ok = match &expect {
        Expect::OnlyRed(red) => &violated == red,
        Expect::NotJudged { red, not_judged, reason, holds } => {
            &violated == red
                && not_judged.iter().all(|invariant| matches!(judged.verdict_of(invariant), 'G' | 'N') && judged.detail_of(invariant).contains(reason))
                && holds.iter().all(|invariant| judged.verdict_of(invariant) == 'H')
        }
        Expect::Holds(invariant) => judged.verdict_of(invariant) == 'H',
    };
    let expect_text = match &expect {
        Expect::OnlyRed(red) => format!("only_red={red:?}"),
        Expect::NotJudged { red, not_judged, reason, holds } => format!("red={red:?} not_judged={not_judged:?} reason∋「{reason}」 holds={holds:?}"),
        Expect::Holds(invariant) => format!("holds={invariant}"),
    };
    let reasons: Vec<String> = judged
        .watched
        .iter()
        .filter(|(_, kind, _)| matches!(kind, 'G' | 'P' | 'V'))
        .map(|(invariant, kind, detail)| format!("{invariant}={kind}:{}", detail.chars().take(60).collect::<String>()))
        .collect();
    println!(
        "BAD case=「{case}」 expect={expect_text} as_expected={ok} violated={violated:?} watched={} details={reasons:?}",
        judged.watched_string()
    );
    ok
}

fn tree_table_slot_of(output: &TransactionOutput) -> SlotNumber {
    output.unit(TransactionUnit::TreeTable).slot
}

fn leftmost_leaf_slot_of(output: &TransactionOutput) -> SlotNumber {
    output.unit(TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::LEFTMOST)).slot
}

const Q: [&str; 3] = ["I-9.5", "I-4.7", "I-4.6"];
const Q4: [&str; 4] = ["I-9.5", "I-4.7", "I-4.6", "I-3.4"];
const R_INODE_TREE_NAMED: &str = "头 ID 指着它的 deleted_inodes 条目不是恰好一条";
const R_DELETED_HEAD: &str = "deleted_inodes 条目偏移 116 的头 ID 配不上同一张树表里恰好一条 inode 树条目";
const R_EXTENT_NOT_SINGLE: &str = "头 ID 指着这棵 inode 树的 extent 树条目不是恰好一条";
const R_EXTENT_UNREADABLE: &str = "extent 树读不全";
const R_DELETED_ROOT: &str = "deleted_inodes 根读不出";
const R_TREE_TABLE: &str = "树表读不出";
const R_NARROW: &str = "树表条目宽不到字段表的 200 字节";
const R_TOMBSTONE: &str = "所选根的树表或墓碑树读不全";

pub fn run() {
    let mut not_as_expected = 0;
    let mut tally = |ok: bool| {
        if !ok {
            not_as_expected += 1;
        }
    };
    // ── A：C708 那一形（队列里的号在树里 nlink ≠ 0），用例原样只在三单元文件意图之后那一版造。
    {
        let mut pool = three_unit_pool();
        intent(&mut pool, 1);
        let mut image = pool.image();
        del::set_the_nlink_of_the_queued_first_file_back_to_one(&mut image);
        tally(report("A0 原造法：三单元意图之后 1 号 nlink 0→1", &image, Expect::OnlyRed(vec!["I-9.5"])));
        pool.step(1, 1).expect("第一批");
        let mut image = pool.image();
        del::set_the_nlink_of_the_queued_first_file_back_to_one(&mut image);
        tally(report("A1 第一批之后（剩两个单元）1 号 nlink 0→1", &image, Expect::OnlyRed(vec!["I-9.5"])));
        let mut image = pool.image();
        del::change_the_inode_record(&mut image, 1, |record| record[36..40].copy_from_slice(&7u32.to_le_bytes()));
        tally(report("A5 第一批之后 1 号 nlink 0→7", &image, Expect::OnlyRed(vec!["I-9.5"])));
    }
    {
        let mut pool = SparsePool::build(DEV, &file_content());
        intent(&mut pool, 1);
        let mut image = pool.image();
        del::set_the_nlink_of_the_queued_first_file_back_to_one(&mut image);
        tally(report("A2 一个单元（3000 字节）意图之后 1 号 nlink 0→1", &image, Expect::OnlyRed(vec!["I-9.5"])));
    }
    {
        let mut pool = SparsePool::build(DEV, &file_content());
        pool.new_inodes(1, FIXED_WRITE_TIME_SECONDS);
        pool.sequential_write(&del::content_of_three_data_units(), FIXED_WRITE_TIME_SECONDS + 60);
        intent(&mut pool, 2);
        let mut image = pool.image();
        del::change_the_inode_record(&mut image, 2, |record| record[36..40].copy_from_slice(&1u32.to_le_bytes()));
        tally(report("A3 H3 那一形（先建 size 0 的 2 号）删 2 号意图之后 2 号 nlink 0→1", &image, Expect::OnlyRed(vec!["I-9.5"])));
        pool.step(2, 1).ok();
    }
    {
        let mut pool = SparsePool::build(DEV, &file_content());
        pool.new_inodes(1, FIXED_WRITE_TIME_SECONDS);
        pool.sequential_write(&del::content_of_three_data_units(), FIXED_WRITE_TIME_SECONDS + 60);
        intent(&mut pool, 1);
        let mut image = pool.image();
        del::set_the_nlink_of_the_queued_first_file_back_to_one(&mut image);
        tally(report("A4 H3 那一形删 1 号意图之后 1 号 nlink 0→1", &image, Expect::OnlyRed(vec!["I-9.5"])));
    }
    // ── B：L1 那一族（反方向不配）。
    {
        let mut pool = three_unit_pool();
        let intent_version = intent(&mut pool, 1);
        let image = pool.image();
        let mut b0 = image.clone();
        del::give_the_deleted_inodes_entry_an_unregistered_kind_in_the_tree_table_at(&mut b0, del::tree_table_slot_of_the_newest_root(&image));
        tally(report("B0 原造法 L1a：意图之后最新树表 deleted_inodes 种类 9→13", &b0, Expect::NotJudged { red: vec!["I-9.16"], not_judged: Q.to_vec(), reason: R_INODE_TREE_NAMED, holds: vec![] }));
        let mut b4 = image.clone();
        del::change_the_unit_and_propagate_its_checksum(&mut b4, del::tree_table_slot_of_the_newest_root(&image), node_bytes(), |table| set_kind_of_entry(table, TREE_KIND_DELETED_INODES, 0));
        tally(report("B4 意图之后最新树表 deleted_inodes 种类 9→0", &b4, Expect::NotJudged { red: vec!["I-9.16"], not_judged: Q.to_vec(), reason: R_INODE_TREE_NAMED, holds: vec![] }));
        let first_batch = pool.step(1, 1).expect("第一批");
        let after = pool.image();
        let mut b1 = after.clone();
        del::give_the_deleted_inodes_entry_an_unregistered_kind_in_the_tree_table_at(&mut b1, tree_table_slot_of(&first_batch));
        tally(report("B1 第一批之后最新树表 deleted_inodes 种类 9→13", &b1, Expect::NotJudged { red: vec!["I-9.16"], not_judged: Q.to_vec(), reason: R_INODE_TREE_NAMED, holds: vec![] }));
        pool.step(1, 1).expect("第二批");
        let later = pool.image();
        let mut b3 = later.clone();
        del::remove_the_deleted_inodes_entry_from_the_tree_table_at(&mut b3, tree_table_slot_of(&intent_version));
        tally(report("B3 只在意图那一版（中间版）整条拿掉 deleted_inodes 条目", &b3, Expect::NotJudged { red: vec!["I-9.16"], not_judged: Q.to_vec(), reason: R_INODE_TREE_NAMED, holds: vec![] }));
    }
    {
        let mut pool = SparsePool::build(DEV, &file_content());
        pool.new_inodes(2, FIXED_WRITE_TIME_SECONDS + 120);
        let image = pool.image();
        let mut b2 = image.clone();
        del::give_the_deleted_inodes_entry_an_unregistered_kind_in_the_tree_table_at(&mut b2, del::tree_table_slot_of_the_newest_root(&image));
        tally(report("B2 从没删过的池（建了两个空 inode）最新树表 deleted_inodes 种类 9→13", &b2, Expect::NotJudged { red: vec!["I-9.16"], not_judged: Q.to_vec(), reason: R_INODE_TREE_NAMED, holds: vec![] }));
    }
    // ── C：deleted_inodes 头 ID 配不上。
    {
        let mut pool = three_unit_pool();
        let intent_version = intent(&mut pool, 1);
        let image = pool.image();
        let extent_tree = tree_table_entry_of_kind(&image, TREE_KIND_EXTENT).tree.0;
        let deleted_tree = tree_table_entry_of_kind(&image, TREE_KIND_DELETED_INODES).tree.0;
        for (name, head) in [("C0 原造法：头 ID→0", 0u64), ("C1 头 ID→extent 树的树 ID", extent_tree), ("C2 头 ID→deleted_inodes 自己的树 ID", deleted_tree)] {
            let mut bad = image.clone();
            del::set_the_head_identifier_of_the_tree_table_entry_of_kind_at(&mut bad, del::tree_table_slot_of_the_newest_root(&image), TREE_KIND_DELETED_INODES, head);
            tally(report(name, &bad, Expect::NotJudged { red: vec!["I-9.27"], not_judged: Q.to_vec(), reason: R_DELETED_HEAD, holds: vec![] }));
        }
        pool.step(1, 1).expect("第一批");
        let later = pool.image();
        let mut c3 = later.clone();
        del::set_the_head_identifier_of_the_tree_table_entry_of_kind_at(&mut c3, tree_table_slot_of(&intent_version), TREE_KIND_DELETED_INODES, 0);
        tally(report("C3 只在意图那一版（中间版）deleted_inodes 头 ID→0", &c3, Expect::NotJudged { red: vec!["I-9.27"], not_judged: Q.to_vec(), reason: R_DELETED_HEAD, holds: vec![] }));
    }
    // ── D：extent 树头 ID 配不上。
    {
        let mut pool = three_unit_pool();
        let intent_version = intent(&mut pool, 1);
        let image = pool.image();
        let deleted_tree = tree_table_entry_of_kind(&image, TREE_KIND_DELETED_INODES).tree.0;
        for (name, head) in [("D0 原造法：extent 头 ID→0", 0u64), ("D1 extent 头 ID→deleted_inodes 的树 ID", deleted_tree)] {
            let mut bad = image.clone();
            del::set_the_head_identifier_of_the_tree_table_entry_of_kind_at(&mut bad, del::tree_table_slot_of_the_newest_root(&image), TREE_KIND_EXTENT, head);
            tally(report(name, &bad, Expect::NotJudged { red: vec!["I-9.27"], not_judged: vec!["I-4.7", "I-4.6"], reason: R_EXTENT_NOT_SINGLE, holds: vec!["I-9.5"] }));
        }
        pool.step(1, 1).expect("第一批");
        let later = pool.image();
        let mut d2 = later.clone();
        del::set_the_head_identifier_of_the_tree_table_entry_of_kind_at(&mut d2, tree_table_slot_of(&intent_version), TREE_KIND_EXTENT, 0);
        tally(report("D2 只在意图那一版（中间版）extent 头 ID→0", &d2, Expect::NotJudged { red: vec!["I-9.27"], not_judged: vec!["I-4.7", "I-4.6"], reason: R_EXTENT_NOT_SINGLE, holds: vec!["I-9.5"] }));
    }
    crate::shapes::more_bad_images(&mut tally);
    println!("BAD_SUMMARY not_as_expected={not_as_expected}");
}

/// 树表里种类是 `kind` 的那一条（恰好一条）的种类码改成 `new_kind`。
pub fn set_kind_of_entry(table: &mut Vec<u8>, kind: u16, new_kind: u16) {
    let count = usize::from(u16::from_le_bytes([table[98], table[99]]));
    let index = (0..count).find(|index| u16::from_le_bytes([table[131 + index * 200 + 10], table[131 + index * 200 + 11]]) == kind).expect("有这一种");
    table[131 + index * 200 + 10..131 + index * 200 + 12].copy_from_slice(&new_kind.to_le_bytes());
}
