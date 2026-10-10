//! N1 的树表形态（两个方向都配上而违例照旧漏判）与 N3 坏镜像 E–M 那几组的换造法。
use singlefs_core::address::SlotNumber;
use singlefs_core::bytes::ByteReader;
use singlefs_core::inode_tree::InodeLeafContainerIndexInTree;
use singlefs_core::pointer::NodePointer;
use singlefs_core::records::{TREE_KIND_DELETED_INODES, TREE_KIND_EXTENT, TREE_KIND_INODE, TREE_KIND_TOMBSTONE};
use singlefs_core::transaction::{TransactionOutput, TransactionUnit};
use singlefs_core::unit::parse_index_node;
use singlefs_harness::memory_pool::MemoryPool;

use crate::badimages::{flip_a_byte_of_both_copies, intent, report, set_kind_of_entry, three_unit_pool, zero_the_copy_on_device_zero, Expect, DEV};
use crate::common::{file_content, FIXED_WRITE_TIME_SECONDS};
use crate::common_deletion_in_progress as del;
use crate::common_pointer_head_corruption::{data_unit_bytes, node_bytes, the_slot_of, tree_table_entry_of_kind, unit_bytes_at};
use crate::judge::{judge, recovered};
use crate::pool::SparsePool;

const Q: [&str; 3] = ["I-9.5", "I-4.7", "I-4.6"];
const Q4: [&str; 4] = ["I-9.5", "I-4.7", "I-4.6", "I-3.4"];

fn entry_kind(table: &[u8], index: usize) -> u16 {
    u16::from_le_bytes([table[131 + index * 200 + 10], table[131 + index * 200 + 11]])
}
fn entry_count(table: &[u8]) -> usize {
    usize::from(u16::from_le_bytes([table[98], table[99]]))
}
fn entry_root(table: &[u8], index: usize) -> NodePointer {
    NodePointer::read_from(&mut ByteReader::at(table, 131 + index * 200 + 14))
}
fn entry_tree(table: &[u8], index: usize) -> u64 {
    u64::from_le_bytes(table[131 + index * 200..131 + index * 200 + 8].try_into().expect("8"))
}
fn set_head(table: &mut [u8], index: usize, head: u64) {
    table[131 + index * 200 + 116..131 + index * 200 + 124].copy_from_slice(&head.to_le_bytes());
}
fn set_kind(table: &mut [u8], index: usize, kind: u16) {
    table[131 + index * 200 + 10..131 + index * 200 + 12].copy_from_slice(&kind.to_le_bytes());
}
fn index_of_kind(table: &[u8], kind: u16) -> usize {
    (0..entry_count(table)).find(|index| entry_kind(table, *index) == kind).expect("有这一种")
}
/// 树表里根指针是空根、种类不是 extent / inode / deleted_inodes 的第一条（拿来改种类）。
fn first_empty_rooted_spare(table: &[u8]) -> (usize, u16) {
    (0..entry_count(table))
        .map(|index| (index, entry_kind(table, index)))
        .find(|(index, kind)| entry_root(table, *index) == NodePointer::empty_root() && ![TREE_KIND_EXTENT, TREE_KIND_INODE, TREE_KIND_DELETED_INODES].contains(kind))
        .expect("有空根的备用条目")
}

fn upper_leaf_lower_root_slot(image: &MemoryPool) -> SlotNumber {
    let root = del::root_slot_of_the_tree_of_kind_of_the_newest_root(image, TREE_KIND_EXTENT);
    let node = parse_index_node(&unit_bytes_at(image, root, node_bytes())).expect("extent 根");
    let entry = node.entries.iter().find(|entry| entry[24] == 1).expect("有下段的条目");
    NodePointer::read_from(&mut ByteReader::at(entry, 25)).locations[0].slot
}

pub fn more_bad_images(tally: &mut dyn FnMut(bool)) {
    // ── E：extent 树读不全。
    {
        let mut pool = three_unit_pool();
        intent(&mut pool, 1);
        let image = pool.image();
        let extent_root = del::root_slot_of_the_tree_of_kind_of_the_newest_root(&image, TREE_KIND_EXTENT);
        let expect = || Expect::NotJudged { red: vec![], not_judged: vec!["I-4.7", "I-4.6"], reason: "extent 树读不全", holds: vec!["I-9.5"] };
        let mut e0 = image.clone();
        del::zero_both_copies_of_the_unit_at(&mut e0, extent_root, node_bytes());
        println!("BAD_NOTE case=E0 用例只断言 I-9.5 成立与两条判不了，不断言判红集合");
        report("E0 原造法：extent 根两份清零（判红集合照实打）", &e0, Expect::Holds("I-9.5"));
        let lower = upper_leaf_lower_root_slot(&image);
        let mut e1 = image.clone();
        del::zero_both_copies_of_the_unit_at(&mut e1, lower, node_bytes());
        report("E1 extent 下段根两份清零（判红集合照实打）", &e1, Expect::Holds("I-9.5"));
        tally(judge(&e1).detail_of("I-4.7").contains("extent 树读不全"));
        let mut e2 = image.clone();
        flip_a_byte_of_both_copies(&mut e2, extent_root, 9000);
        report("E2 extent 根两份各翻一个载荷字节（判红集合照实打）", &e2, Expect::Holds("I-9.5"));
        tally(judge(&e2).detail_of("I-4.6").contains("extent 树读不全"));
        let mut e3 = image.clone();
        zero_the_copy_on_device_zero(&mut e3, extent_root, node_bytes());
        tally(report("E3 extent 根只清盘 0 那一份（盘 1 读得出）", &e3, Expect::NotJudged { red: vec![], not_judged: vec![], reason: "", holds: Q.to_vec() }));
        let _ = expect;
    }
    // ── F：deleted_inodes 根读不出。
    {
        let mut pool = three_unit_pool();
        intent(&mut pool, 1);
        let image = pool.image();
        let root = del::root_slot_of_the_tree_of_kind_of_the_newest_root(&image, TREE_KIND_DELETED_INODES);
        let mut f0 = image.clone();
        del::zero_both_copies_of_the_unit_at(&mut f0, root, node_bytes());
        report("F0 原造法：deleted_inodes 根两份清零（判红集合照实打）", &f0, Expect::Holds("I-3.4"));
        tally(Q.iter().all(|invariant| judge(&f0).detail_of(invariant).contains("deleted_inodes 根读不出")));
        let mut f1 = image.clone();
        flip_a_byte_of_both_copies(&mut f1, root, 200);
        report("F1 deleted_inodes 根两份各翻一个载荷字节（判红集合照实打）", &f1, Expect::Holds("I-3.4"));
        tally(Q.iter().all(|invariant| judge(&f1).detail_of(invariant).contains("deleted_inodes 根读不出")));
        let mut f2 = image.clone();
        del::change_the_unit_and_propagate_its_checksum(&mut f2, root, node_bytes(), |node| node[50] = 1);
        report("F2 deleted_inodes 根层级字节 0→1、重封补链（判红集合照实打）", &f2, Expect::Holds("I-3.4"));
        tally(Q.iter().all(|invariant| judge(&f2).detail_of(invariant).contains("deleted_inodes 根读不出")));
    }
    // ── G、H：中间版树表读不出 / 条目窄。
    {
        let mut pool = three_unit_pool();
        intent(&mut pool, 1);
        let intermediate = pool.step(1, 1).expect("第一批");
        pool.step(1, 1).expect("第二批");
        let image = pool.image();
        let slot = intermediate.unit(TransactionUnit::TreeTable).slot;
        let mut g1 = image.clone();
        flip_a_byte_of_both_copies(&mut g1, slot, 5000);
        report("G1 中间版树表两份各翻一个载荷字节（判红集合照实打）", &g1, Expect::Holds("I-9.6"));
        tally(Q4.iter().all(|invariant| judge(&g1).detail_of(invariant).contains("树表读不出")));
        let mut g2 = image.clone();
        zero_the_copy_on_device_zero(&mut g2, slot, node_bytes());
        tally(report("G2 中间版树表只清盘 0 那一份", &g2, Expect::NotJudged { red: vec![], not_judged: vec![], reason: "", holds: Q4.to_vec() }));
        let mut h1 = image.clone();
        del::narrow_the_entries_of_the_tree_table_at_to_the_key_width(&mut h1, del::tree_table_slot_of_the_newest_root(&image));
        report("H1 最新那一版树表条目收窄到 8（判红集合照实打）", &h1, Expect::Holds("I-9.6"));
        tally(Q4.iter().all(|invariant| judge(&h1).detail_of(invariant).contains("树表条目宽不到字段表的 200 字节")));
    }
    // ── I、J：墓碑那一半。
    {
        let mut pool = three_unit_pool();
        intent(&mut pool, 1);
        let image = pool.image();
        let tombstone_root = del::root_slot_of_the_tree_of_kind_of_the_newest_root(&image, TREE_KIND_TOMBSTONE);
        let container = del::first_tombstone_container_slot_of_the_newest_root(&image);
        let mut i1 = image.clone();
        del::zero_both_copies_of_the_unit_at(&mut i1, tombstone_root, node_bytes());
        report("I1 墓碑树根两份清零（判红集合照实打）", &i1, Expect::Holds("I-9.5"));
        tally(judge(&i1).detail_of("I-9.6").contains("墓碑树读不全"));
        let mut i2 = image.clone();
        flip_a_byte_of_both_copies(&mut i2, container, 20000);
        report("I2 墓碑容器两份各翻一个载荷字节（判红集合照实打）", &i2, Expect::Holds("I-9.5"));
        tally(judge(&i2).detail_of("I-9.6").contains("墓碑树读不全"));
        let mut j0 = image.clone();
        del::move_the_tombstone_container_to_an_unpublished_instance_and_raise_its_object_identifier(&mut j0);
        tally(report("J0 原造法：墓碑容器实例 1→2、对象 ID 抬到水位", &j0, Expect::Holds("I-9.6")));
        let mut j1 = image.clone();
        del::change_the_unit_and_propagate_its_checksum(&mut j1, container, data_unit_bytes(), |unit| {
            unit[93..97].copy_from_slice(&6u32.to_le_bytes());
            unit[136..144].copy_from_slice(&2u64.to_le_bytes());
        });
        tally(report("J1 墓碑容器实例 1→6、对象 ID 抬到水位", &j1, Expect::Holds("I-9.6")));
        let mut j2 = image.clone();
        del::change_the_unit_and_propagate_its_checksum(&mut j2, container, data_unit_bytes(), |unit| {
            unit[136..144].copy_from_slice(&2u64.to_le_bytes());
        });
        tally(report("J2 对照：实例不动、只抬对象 ID", &j2, Expect::OnlyRed(vec!["I-9.6"])));
    }
    // ── L、M：只改中间版。
    {
        let mut pool = SparsePool::build(DEV, &file_content());
        let first = pool.new_inodes(1, FIXED_WRITE_TIME_SECONDS + 100);
        let second = pool.new_inodes(1, FIXED_WRITE_TIME_SECONDS + 100);
        pool.new_inodes(1, FIXED_WRITE_TIME_SECONDS + 100);
        let image = pool.image();
        let leaf = |output: &TransactionOutput| output.unit(TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::LEFTMOST)).slot;
        let mut l1 = image.clone();
        del::set_the_nlink_of_an_inode_record_in_the_leaf_at_to_zero(&mut l1, leaf(&first), 2);
        tally(report("L1 三次建 inode，只在第一次那一版把 2 号 nlink 改 0", &l1, Expect::OnlyRed(vec!["I-9.5"])));
        let mut l2 = image.clone();
        del::set_the_nlink_of_an_inode_record_in_the_leaf_at_to_zero(&mut l2, leaf(&second), 3);
        tally(report("L2 三次建 inode，只在第二次那一版把 3 号 nlink 改 0", &l2, Expect::OnlyRed(vec!["I-9.5"])));
    }
    {
        let mut pool = three_unit_pool();
        let first = pool.new_inodes(1, FIXED_WRITE_TIME_SECONDS + 100);
        pool.new_inodes(1, FIXED_WRITE_TIME_SECONDS + 100);
        let image = pool.image();
        let mut m1 = image.clone();
        del::drop_the_record_of_the_first_file_in_the_leaf_at_and_keep_its_data_units(&mut m1, first.unit(TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::LEFTMOST)).slot);
        tally(report("M1 三单元文件（有下段）建两次 inode，只在中间版拿掉 1 号记录", &m1, Expect::OnlyRed(vec!["I-4.7"])));
    }
}

/// N1：树表形态两个方向都配上，违例照旧漏判。
pub fn run() {
    // S1：extent 树反方向不配。基底：1 号（一个单元）记录拿掉、单元留着、号不在队列（已登记的 I-4.7 坏镜像，只红 I-4.7）。
    let mut pool = SparsePool::build(DEV, &file_content());
    pool.new_inodes(1, FIXED_WRITE_TIME_SECONDS + 100);
    let clean = pool.image();
    let mut base = clean.clone();
    del::drop_the_record_of_the_first_file_and_keep_its_data_units(&mut base);
    report("S1-base 已登记的 I-4.7 坏镜像（对照）", &base, Expect::OnlyRed(vec!["I-4.7"]));
    let slot = del::tree_table_slot_of_the_newest_root(&base);
    let table = unit_bytes_at(&base, slot, node_bytes());
    let inode_tree = entry_tree(&table, index_of_kind(&table, TREE_KIND_INODE));
    let (spare, spare_kind) = first_empty_rooted_spare(&table);
    println!("SHAPE_NOTE spare_entry_index={spare} spare_kind={spare_kind} spare_tree={} inode_tree={inode_tree}", entry_tree(&table, spare));
    let mut s1 = base.clone();
    del::change_the_unit_and_propagate_its_checksum(&mut s1, slot, node_bytes(), |table| {
        let extent = index_of_kind(table, TREE_KIND_EXTENT);
        set_head(table, extent, 0);
        set_kind(table, spare, TREE_KIND_EXTENT);
        set_head(table, spare, inode_tree);
    });
    let judged = judge(&s1);
    println!(
        "SHAPE case=「S1 extent 树反方向不配：真 extent 树头 ID→0，一条空根备用条目改成种类 1、头 ID→inode 树」 literal=I-4.7（真 extent 树里 1 号还有 1 个单元、inode 树里没有 1 号记录、队列空） checker_I-4.7={} detail={} violated={:?} watched={}",
        judged.verdict_of("I-4.7"),
        judged.detail_of("I-4.7"),
        judged.violated_names(),
        judged.watched_string()
    );
    // S1b：同一手法，基底换成删除中间态（三单元文件意图之后再把 1 号的记录拿掉、单元留着、队列 {1} 改空不可行时用 2 号）：
    // 2 号建出来、意图删 1 号之后，把 1 号的记录拿掉（单元留着）、队列里只剩它——I-4.7 本该成立（号在队列）；对照组不改。
    // S2：deleted_inodes 那一侧同一手法：真 deleted_inodes 头 ID→0，备用条目改成种类 9、头 ID→inode 树。
    let mut s2_base = SparsePool::build(DEV, &file_content());
    s2_base.sequential_write(&del::content_of_three_data_units(), FIXED_WRITE_TIME_SECONDS + 60);
    intent(&mut s2_base, 1);
    let image = s2_base.image();
    let slot = del::tree_table_slot_of_the_newest_root(&image);
    let table = unit_bytes_at(&image, slot, node_bytes());
    let inode_tree = entry_tree(&table, index_of_kind(&table, TREE_KIND_INODE));
    let (spare, _) = first_empty_rooted_spare(&table);
    let mut s2 = image.clone();
    del::change_the_unit_and_propagate_its_checksum(&mut s2, slot, node_bytes(), |table| {
        let deleted = index_of_kind(table, TREE_KIND_DELETED_INODES);
        set_head(table, deleted, 0);
        set_kind(table, spare, TREE_KIND_DELETED_INODES);
        set_head(table, spare, inode_tree);
    });
    let judged = judge(&s2);
    println!(
        "SHAPE case=「S2 deleted_inodes 那一侧同一手法（意图之后）：真 deleted_inodes 头 ID→0，备用条目改种类 9、头 ID→inode 树」 checker={} violated={:?} I-9.5_detail={}",
        judged.watched_string(),
        judged.violated_names(),
        judged.detail_of("I-9.5").chars().take(80).collect::<String>()
    );
    // S3：第二棵 inode 树（备用条目改种类 2，树 ID 是它自己的）：反方向那一遍配不上它。
    let mut s3 = image.clone();
    del::change_the_unit_and_propagate_its_checksum(&mut s3, slot, node_bytes(), |table| {
        set_kind(table, spare, TREE_KIND_INODE);
        let own = entry_tree(table, spare);
        set_head(table, spare, own);
    });
    let judged = judge(&s3);
    println!(
        "SHAPE case=「S3 第二棵 inode 树（备用条目改种类 2、空根）」 checker={} violated={:?} I-9.5_detail={}",
        judged.watched_string(),
        judged.violated_names(),
        judged.detail_of("I-9.5").chars().take(80).collect::<String>()
    );
    // S4：同号两条记录（不是树表形态；I-9.5 两个方向都配上）。1 号（一个单元）、2 号空：叶里 [1 nlink 0, 1 nlink 1]，队列空。
    for (name, first_nlink, second_nlink) in [("S4a 叶里 [1 nlink 0, 1 nlink 1]、队列空", 0u32, 1u32), ("S4b 叶里 [1 nlink 1, 1 nlink 0]、队列空", 1, 0)] {
        let mut s4 = clean.clone();
        let (leaf_slot, record0) = {
            let root = del::root_slot_of_the_tree_of_kind_of_the_newest_root(&clean, TREE_KIND_INODE);
            let node = parse_index_node(&unit_bytes_at(&clean, root, node_bytes())).expect("inode 根");
            let pointer = NodePointer::read_from(&mut ByteReader::at(&node.entries[0], 34));
            let leaf = unit_bytes_at(&clean, pointer.locations[0].slot, data_unit_bytes());
            (pointer.locations[0].slot, leaf[136..276].to_vec())
        };
        del::change_the_unit_and_propagate_its_checksum(&mut s4, leaf_slot, data_unit_bytes(), |leaf| {
            leaf[136..276].copy_from_slice(&record0);
            leaf[276..416].copy_from_slice(&record0);
            leaf[136 + 36..136 + 40].copy_from_slice(&first_nlink.to_le_bytes());
            leaf[276 + 36..276 + 40].copy_from_slice(&second_nlink.to_le_bytes());
        });
        let judged = judge(&s4);
        let recovered = recovered(&s4);
        println!(
            "SHAPE case=「{name}」 literal=I-9.5②（有一条 nlink 0 的 1 号记录、号不在队列） checker_I-9.5={} violated={:?} cpu_recovery_consulted={} ignored={}",
            judged.verdict_of("I-9.5"),
            judged.violated_names(),
            recovered.consulted,
            recovered.ignored
        );
    }
}
