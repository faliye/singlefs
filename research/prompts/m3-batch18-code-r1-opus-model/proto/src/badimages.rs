//! 坏镜像换一种造法再造一次，看是不是仍只红目标那一条；再造几份「该红而判成立或不适用」的镜像（漏判）。
//! 基线都在内存稀疏盘上由发布路径写出（`SparsePool`），改法复用 harness 档 `common_deletion_in_progress` 的重封与补校验和。
use singlefs_core::address::SlotNumber;
use singlefs_core::records::{TREE_KIND_DELETED_INODES, TREE_KIND_EXTENT, TREE_KIND_INODE};
use singlefs_core::transaction::{TransactionOutput, TransactionUnit};
use singlefs_core::inode_tree::InodeLeafContainerIndexInTree;
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_harness::memory_pool::MemoryPool;

use crate::common::{file_content, FIXED_WRITE_TIME_SECONDS};
use crate::common_deletion_in_progress as deletion;
use crate::common_pointer_head_corruption::{newest_root, node_bytes, the_slot_of, unit_bytes_at};
use crate::judge::{judge, WATCHED};
use crate::pool::{content_of, SparsePool};

fn u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().expect("2"))
}
fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("4"))
}
fn u64_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8"))
}

/// 码 2 节点的 (key 宽, 条目数, 条目宽, 条目区起点)。
fn index_node_layout(node: &[u8]) -> (usize, usize, usize, usize) {
    let key_width = usize::from(node[51]);
    let count = usize::from(u16_at(node, 82 + 2 * key_width));
    let width = usize::from(u16_at(node, 84 + 2 * key_width));
    (key_width, count, width, 115 + 2 * key_width)
}

fn three_unit_pool() -> SparsePool {
    let payload = data_unit_payload_capacity();
    let mut pool = SparsePool::build(4 << 30, &file_content());
    pool.sequential_write(&content_of(2 * payload + payload / 2), FIXED_WRITE_TIME_SECONDS + 60);
    pool
}

fn record_time(pool: &SparsePool) -> u64 {
    pool.output.inode_record.map_or(FIXED_WRITE_TIME_SECONDS, |record| record.write_time_seconds)
}

fn slot_of(version: &TransactionOutput, unit: TransactionUnit) -> SlotNumber {
    version.unit(unit).slot
}

/// 最新那条根的树表单元的槽。
fn newest_tree_table_slot(image: &MemoryPool) -> SlotNumber {
    the_slot_of(&newest_root(image).tree_table)
}

/// 树表单元（`slot`）里种类为 `kind` 的那条条目改一处（`change` 拿到 200 字节的条目）。
fn change_tree_table_entry(image: &mut MemoryPool, slot: SlotNumber, kind: u16, change: impl Fn(&mut [u8])) {
    deletion::change_the_unit_and_propagate_its_checksum(image, slot, node_bytes(), |node| {
        let (_, count, width, start) = index_node_layout(node);
        let index = (0..count).find(|index| u16_at(node, start + index * width + 10) == kind).expect("树表里有这种树");
        change(&mut node[start + index * width..start + (index + 1) * width]);
    });
}

/// 树表单元（`slot`）里种类为 `kind` 的那条条目整条拿掉（后面的条目往前挪、条目数与声明长度各减一条）。
fn drop_tree_table_entry(image: &mut MemoryPool, slot: SlotNumber, kind: u16) {
    deletion::change_the_unit_and_propagate_its_checksum(image, slot, node_bytes(), |node| {
        let (key_width, count, width, start) = index_node_layout(node);
        let index = (0..count).find(|index| u16_at(node, start + index * width + 10) == kind).expect("树表里有这种树");
        assert!(index > 0 && index + 1 < count, "不在首末：头里的 key 区间不变");
        let end = start + count * width;
        node.copy_within(start + (index + 1) * width..end, start + index * width);
        node[end - width..end].fill(0);
        node[82 + 2 * key_width..84 + 2 * key_width].copy_from_slice(&u16::try_from(count - 1).expect("u16").to_le_bytes());
        let declared = u16_at(node, 8);
        node[8..10].copy_from_slice(&(declared - u16::try_from(width).expect("u16")).to_le_bytes());
    });
}

/// 记账树根兼叶（`slot`）里统计量 `statistic`、设备段 0xFFFF_FFFF 那一行的值改成 `value`。
fn set_accounting_row(image: &mut MemoryPool, slot: SlotNumber, statistic: u16, value: u64) {
    deletion::change_the_unit_and_propagate_its_checksum(image, slot, node_bytes(), |node| {
        assert_eq!(node[50], 0, "记账树只有根兼叶");
        let (_, count, width, start) = index_node_layout(node);
        let row = (0..count)
            .map(|index| start + index * width)
            .find(|row| u16_at(node, *row) == statistic && u32_at(node, row + 10) == 0xFFFF_FFFF)
            .expect("有这一行");
        node[row + 22..row + 30].copy_from_slice(&value.to_le_bytes());
    });
}

/// `deleted_inodes` 根（最新那条根那一版）里加一个号（升序插在末尾，号要比现有的都大）。
fn add_to_the_queue(image: &mut MemoryPool, inode: u64) {
    let root = deletion_root_slot(image);
    deletion::change_the_unit_and_propagate_its_checksum(image, root, node_bytes(), |node| {
        let (key_width, count, width, start) = index_node_layout(node);
        assert!(u64_at(node, start + (count - 1) * width) < inode, "号比队列里现有的大");
        node[start + count * width..start + count * width + 8].copy_from_slice(&inode.to_le_bytes());
        node[82 + 2 * key_width..84 + 2 * key_width].copy_from_slice(&u16::try_from(count + 1).expect("u16").to_le_bytes());
        let declared = u16_at(node, 8);
        node[8..10].copy_from_slice(&(declared + u16::try_from(width).expect("u16")).to_le_bytes());
        node[52 + key_width..52 + 2 * key_width].copy_from_slice(&inode.to_le_bytes());
    });
}

fn deletion_root_slot(image: &MemoryPool) -> SlotNumber {
    the_slot_of(&crate::common_pointer_head_corruption::tree_table_entry_of_kind(image, TREE_KIND_DELETED_INODES).root)
}

fn set_record_size(record: &mut [u8], size: u64) {
    record[40..48].copy_from_slice(&size.to_le_bytes());
    record[48..56].copy_from_slice(&size.div_ceil(512).to_le_bytes());
}

struct Case {
    name: &'static str,
    /// 改法打中的那一条（「只红自己」要求违例集合恰好是它）；None 是漏判那一组：按 invariants.md 字面该红的一条写在 `literal_red`。
    expected_only: Option<&'static str>,
    literal_red: &'static str,
    image: MemoryPool,
}

fn cases() -> Vec<Case> {
    let payload = u64::try_from(data_unit_payload_capacity()).expect("u64");
    let mut cases = Vec::new();

    // V1：I-9.5 正方向，改在第一批之后那一版（最新）而不是意图那一版。
    let mut pool = three_unit_pool();
    let time = record_time(&pool);
    pool.intent(1, time).expect("意图");
    pool.step(1, 1).expect("第一批");
    let mut image = pool.image();
    deletion::put_an_inode_number_without_a_record_into_the_deletion_queue(&mut image);
    cases.push(Case { name: "V1 第一批之后队列 {1} 改 {1,7}（7 没有记录）", expected_only: Some("I-9.5"), literal_red: "I-9.5", image });

    // V2：I-9.5 正方向另一形：队列里的号在树里有记录而 nlink = 1（C708 还差的那一形）。
    let mut pool = three_unit_pool();
    pool.new_inodes(2, FIXED_WRITE_TIME_SECONDS + 120);
    let time = record_time(&pool);
    pool.intent(1, time).expect("意图");
    let mut image = pool.image();
    add_to_the_queue(&mut image, 2);
    cases.push(Case { name: "V2 意图之后队列 {1} 改 {1,2}（2 号活着、nlink 1）", expected_only: Some("I-9.5"), literal_red: "I-9.5", image });

    // V3：I-9.5 反方向，僵尸记录只在中间版（意图那一版），最新根写对。
    let mut pool = three_unit_pool();
    pool.new_inodes(1, FIXED_WRITE_TIME_SECONDS + 120);
    let time = record_time(&pool);
    let intent = pool.intent(1, time).expect("意图");
    pool.step(1, 1).expect("第一批");
    pool.step(1, 1).expect("第二批");
    let mut image = pool.image();
    let leaf = slot_of(&intent, TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::LEFTMOST));
    deletion::change_the_inode_record_in_the_leaf_at(&mut image, leaf, 2, |record| record[36..40].copy_from_slice(&0u32.to_le_bytes()));
    cases.push(Case { name: "V3 只在意图那一版把空 inode 2 的 nlink 改 0（最新根写对）", expected_only: Some("I-9.5"), literal_red: "I-9.5", image });

    // V4：I-4.7 孤儿单元，三个单元的文件（有下段）而不是内联的一个单元。
    let mut pool = three_unit_pool();
    pool.new_inodes(1, FIXED_WRITE_TIME_SECONDS + 120);
    let mut image = pool.image();
    deletion::drop_the_record_of_the_first_file_and_keep_its_data_units(&mut image);
    cases.push(Case { name: "V4 三个单元（有下段）的文件拿掉记录、单元留着", expected_only: Some("I-4.7"), literal_red: "I-4.7", image });

    // V5、V6：I-4.6 改小 size（意图那一版，最新）。
    for (name, size) in [("V5 意图那一版 size 2.5 净荷改 1.5 净荷（要 2 个、有 3 个）", payload + payload / 2), ("V6 意图那一版 size 改 0（要 0 或 1 个、有 3 个）", 0)] {
        let mut pool = three_unit_pool();
        let time = record_time(&pool);
        pool.intent(1, time).expect("意图");
        let mut image = pool.image();
        deletion::change_the_inode_record(&mut image, 1, |record| set_record_size(record, size));
        cases.push(Case { name, expected_only: Some("I-4.6"), literal_red: "I-4.6", image });
    }

    // V7：I-3.4 标签 4 非 0，改在意图那一版（最新）。
    let mut pool = three_unit_pool();
    let time = record_time(&pool);
    pool.intent(1, time).expect("意图");
    let mut image = pool.image();
    deletion::set_the_pending_deletion_occupancy_row_to_one(&mut image);
    cases.push(Case { name: "V7 意图那一版记账标签 4 那一行 0 改 1", expected_only: Some("I-3.4"), literal_red: "I-3.4", image });

    // V8：I-3.4 缺行，改在中间那一版（第一批），最新根写对。
    let mut pool = three_unit_pool();
    let time = record_time(&pool);
    pool.intent(1, time).expect("意图");
    let first_batch = pool.step(1, 1).expect("第一批");
    pool.step(1, 1).expect("第二批");
    let mut image = pool.image();
    deletion::change_the_pending_deletion_occupancy_row_of_the_accounting_tree_at(&mut image, slot_of(&first_batch, TransactionUnit::AccountingTree), None);
    cases.push(Case { name: "V8 只在第一批那一版拿掉标签 4 那一行（最新根写对）", expected_only: Some("I-3.4"), literal_red: "I-3.4", image });

    // V9：I-9.6 墓碑那一半：删完之后（inode 树空）记账水位 2 改 1，墓碑对象 ID 1 不小于它。
    let mut pool = three_unit_pool();
    let time = record_time(&pool);
    pool.intent(1, time).expect("意图");
    pool.steps_until_empty(1);
    let mut image = pool.image();
    let accounting = deletion::accounting_tree_slot_of_the_newest_root(&image);
    set_accounting_row(&mut image, accounting, 12, 1);
    cases.push(Case { name: "V9 删完之后记账 inode 号水位 2 改 1（墓碑对象 ID 1）", expected_only: Some("I-9.6"), literal_red: "I-9.6", image });

    // 漏判组 L1：意图那一版（最新）树表里 deleted_inodes 那条条目的种类码 9 改成没登记的 13：
    // 1 号 nlink = 0、号不在这棵 inode 树的任何队列里。
    let mut pool = three_unit_pool();
    let time = record_time(&pool);
    pool.intent(1, time).expect("意图");
    let mut image = pool.image();
    let slot = newest_tree_table_slot(&image);
    change_tree_table_entry(&mut image, slot, TREE_KIND_DELETED_INODES, |entry| entry[10..12].copy_from_slice(&13u16.to_le_bytes()));
    cases.push(Case { name: "L1a 最新那一版树表 deleted_inodes 条目种类码 9 改 13", expected_only: None, literal_red: "I-9.5", image });

    // L1b：同上，整条拿掉。
    let mut pool = three_unit_pool();
    let time = record_time(&pool);
    pool.intent(1, time).expect("意图");
    let mut image = pool.image();
    let slot = newest_tree_table_slot(&image);
    drop_tree_table_entry(&mut image, slot, TREE_KIND_DELETED_INODES);
    cases.push(Case { name: "L1b 最新那一版树表 deleted_inodes 条目整条拿掉", expected_only: None, literal_red: "I-9.5", image });

    // L1c：同 L1a，只改中间那一版（第一批），最新根写对。
    let mut pool = three_unit_pool();
    let time = record_time(&pool);
    pool.intent(1, time).expect("意图");
    let first_batch = pool.step(1, 1).expect("第一批");
    pool.step(1, 1).expect("第二批");
    let mut image = pool.image();
    change_tree_table_entry(&mut image, slot_of(&first_batch, TransactionUnit::TreeTable), TREE_KIND_DELETED_INODES, |entry| entry[10..12].copy_from_slice(&13u16.to_le_bytes()));
    cases.push(Case { name: "L1c 只在第一批那一版树表把 deleted_inodes 种类码改 13", expected_only: None, literal_red: "I-9.5", image });

    // L1d：对照：同一处换成头 ID 配不上（deleted_inodes 条目偏移 116 改成 extent 树的树 ID），共用判法说「配不上的版记判不了」。
    let mut pool = three_unit_pool();
    let time = record_time(&pool);
    pool.intent(1, time).expect("意图");
    let mut image = pool.image();
    let slot = newest_tree_table_slot(&image);
    let extent_tree = crate::common_pointer_head_corruption::tree_table_entry_of_kind(&image, TREE_KIND_EXTENT);
    let extent_tree_identifier = extent_tree.tree.0;
    change_tree_table_entry(&mut image, slot, TREE_KIND_DELETED_INODES, |entry| entry[116..124].copy_from_slice(&extent_tree_identifier.to_le_bytes()));
    cases.push(Case { name: "L1d 对照：最新那一版 deleted_inodes 头 ID 改成 extent 树的树 ID", expected_only: None, literal_red: "I-9.5", image });

    // L2：I-4.7 / I-4.6 也只在 deleted_inodes 那条条目下判：L1a 那一形上 1 号还有 3 个单元、size 改大一个净荷——I-4.6 字面该红。
    let mut pool = three_unit_pool();
    let time = record_time(&pool);
    pool.intent(1, time).expect("意图");
    let mut image = pool.image();
    deletion::change_the_inode_record(&mut image, 1, |record| set_record_size(record, 3 * payload + payload / 2));
    let slot = newest_tree_table_slot(&image);
    change_tree_table_entry(&mut image, slot, TREE_KIND_DELETED_INODES, |entry| entry[10..12].copy_from_slice(&13u16.to_le_bytes()));
    cases.push(Case { name: "L2 L1a 之上再把 1 号 size 加一个净荷（要 4 个、有 3 个）", expected_only: None, literal_red: "I-4.6", image });

    let _ = TREE_KIND_INODE;
    cases
}

pub fn run() {
    let mut mismatches = 0;
    for case in cases() {
        let judged = judge(&case.image);
        let violated = judged.violated_names();
        let verdict = match case.expected_only {
            Some(target) => {
                let only = violated == vec![target];
                if !only {
                    mismatches += 1;
                }
                format!("only_red_itself={only}")
            }
            None => {
                let literal = judged.verdict_of(case.literal_red);
                let missed = literal != 'V';
                if missed {
                    mismatches += 1;
                }
                format!("literal_red={} checker_verdict={literal} missed={missed} reason={}", case.literal_red, judged.detail_of(case.literal_red))
            }
        };
        println!("BAD_IMAGE case=「{}」 {verdict} violated={violated:?} watched={}", case.name, judged.watched_string());
        for (invariant, text) in &judged.violated {
            if WATCHED.contains(invariant) {
                println!("BAD_IMAGE_DETAIL case=「{}」 {invariant}：{text}", case.name);
            }
        }
    }
    println!("BAD_IMAGE_SUMMARY cases_not_as_expected={mismatches}");
}
