//! 攻方副本（m3-leaf-death-r2 opus，只在草稿目录的仓副本里跑）：区间墓碑落在活对象上、同一区间两条墓碑、分隔 key 与容器号脱钩、
//! 删除各次发布的重写角色。每份镜像打一行 `OPUSR2 ...`，不断言（判定交给 checker，谓词另由本文件从镜像上现算）。
mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use common_deletion_in_progress::{
    change_the_unit_and_propagate_its_checksum, first_tombstone_container_slot_of_the_newest_root, object_birth_of,
    pool_with_inode_three_unlinked_and_the_root_ring_turned_over, pool_with_new_inodes, unlink_to_the_end, violated_invariants,
};
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
const TOMBSTONE_BYTES: usize = 40;
const INODE_RECORD_BYTES: usize = 140;
const CONTAINER_NUMBER_OFFSET: usize = 53;
const ROOT_ENTRIES_START: usize = 131;
const ENTRY_BYTES: usize = 120;
const ENTRY_CHILD_POINTER: usize = 34;

fn get_u16(b: &[u8], o: usize) -> u16 { u16::from_le_bytes(b[o..o + 2].try_into().unwrap()) }
fn get_u64(b: &[u8], o: usize) -> u64 { u64::from_le_bytes(b[o..o + 8].try_into().unwrap()) }
fn set_u16(b: &mut [u8], o: usize, v: u16) { b[o..o + 2].copy_from_slice(&v.to_le_bytes()); }
fn set_u64(b: &mut [u8], o: usize, v: u64) { b[o..o + 8].copy_from_slice(&v.to_le_bytes()); }

/// (对象, 对象出生代, 区间起, 区间止, 死亡 txg)
type Tombstone = (u64, u64, u64, u64, u64);

fn tombstones(image: &MemoryPool) -> Vec<Tombstone> {
    let c = unit_bytes_at(image, first_tombstone_container_slot_of_the_newest_root(image), data_unit_bytes());
    (0..usize::from(get_u16(&c, RECORD_COUNT_OFFSET))).map(|i| {
        let o = RECORDS_START + i * TOMBSTONE_BYTES;
        (get_u64(&c, o), get_u64(&c, o + 8), get_u64(&c, o + 16), get_u64(&c, o + 24), get_u64(&c, o + 32))
    }).collect()
}

/// 墓碑树第一片容器整份换成 `records`（重封、补校验和）。
fn set_tombstones(image: &mut MemoryPool, records: &[Tombstone]) {
    let slot = first_tombstone_container_slot_of_the_newest_root(image);
    change_the_unit_and_propagate_its_checksum(image, slot, data_unit_bytes(), |c| {
        c[RECORDS_START..].fill(0);
        for (i, t) in records.iter().enumerate() {
            let o = RECORDS_START + i * TOMBSTONE_BYTES;
            for (k, v) in [t.0, t.1, t.2, t.3, t.4].into_iter().enumerate() { set_u64(c, o + 8 * k, v); }
        }
        set_u16(c, RECORD_COUNT_OFFSET, u16::try_from(records.len()).unwrap());
        set_u16(c, DECLARED_LENGTH_OFFSET, u16::try_from(records.len() * TOMBSTONE_BYTES).unwrap());
    });
}

fn report_tombstones(name: &str, image: &MemoryPool) {
    println!("OPUSR2 image={name} tombstones={:?} violated={:?}", tombstones(image), violated_invariants(image));
}

/// K3：区间墓碑。1 号（新池新建文件，活着）、2 号（空 inode，活着）、3 号删完；根环已转过一圈，3 号那条整对象墓碑在谱系上。
#[test]
fn opus_r2_range_tombstones_on_live_objects() {
    let pool = pool_with_inode_three_unlinked_and_the_root_ring_turned_over("opus-r2-range-tombstones");
    let birth_of_one = object_birth_of(&pool.output, 1).expect("1 号活着").0;
    let newest = pool.output.root.checkpoint_txg.0;
    let clean = pool.memory_pool();
    report_tombstones("clean", &clean);
    let whole_of_three = tombstones(&clean)[0];
    let death = whole_of_three.4;
    println!("OPUSR2 facts birth_of_inode_one={birth_of_one} death_txg_of_the_tombstone_of_three={death} newest_txg={newest}");
    // 1 号「变短」那次写一条 [1, 2⁶⁴ − 1]（区间起取单元序号 1；取字节偏移时同形）：1 号活着、nlink 1。
    let mut image = clean.clone();
    set_tombstones(&mut image, &[whole_of_three, (1, birth_of_one, 1, u64::MAX, death)]);
    report_tombstones("one_range_tombstone_on_live_inode_one", &image);
    // 同一个活文件两次变短到同一个末尾（变短 → 顺序写长回来 → 再变短）：两条区间相同、死亡 txg 不同，都在谱系上。
    let mut image = clean.clone();
    set_tombstones(&mut image, &[whole_of_three, (1, birth_of_one, 1, u64::MAX, death - 1), (1, birth_of_one, 1, u64::MAX, death)]);
    report_tombstones("two_equal_range_tombstones_on_live_inode_one", &image);
    // 对照：同两条换成整对象的（同号同出生代两条整对象在谱系上，今天只红 I-9.20 的不重号那一句 + I-9.19）。
    let mut image = clean.clone();
    set_tombstones(&mut image, &[whole_of_three, (1, birth_of_one, 0, u64::MAX, death)]);
    report_tombstones("one_whole_object_tombstone_on_live_inode_one", &image);
}

// ── K4：分隔 key 与容器号脱钩（左从右借），扫一族分隔 key，看 checker 的 I-9.4 / I-9.12 与本文件现算的 N1 谓词。
// 叶、根条目的读写照第一轮攻方 `k3-copy/opus_k3_borrowed_leaf_images.rs` 那几个函数重写。

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

fn report_separators(name: &str, image: &MemoryPool) {
    let slots = leaf_slots(image);
    let root = unit_bytes_at(image, inode_root_slot(image), node_bytes());
    let mut shape = String::new();
    let (mut n1a, mut n1b, mut today) = (true, true, true);
    let mut previous: Option<u64> = None;
    for (i, slot) in slots.iter().enumerate() {
        let leaf = unit_bytes_at(image, *slot, data_unit_bytes());
        let recs = records_of(image, *slot);
        let (smallest, largest) = (get_u64(&recs[0], 0), get_u64(recs.last().unwrap(), 0));
        let number = get_u64(&leaf, CONTAINER_NUMBER_OFFSET);
        let separator = get_u64(&root, ROOT_ENTRIES_START + i * ENTRY_BYTES);
        if let Some(left_largest) = previous {
            n1a &= left_largest < smallest;
            n1b &= left_largest < separator;
            today &= left_largest < number;
        }
        previous = Some(largest);
        shape.push_str(&format!("[no={number} sep={separator} {smallest}..{largest}]"));
    }
    println!("OPUSR2 image={name} leaves={shape} third_clause_today={today} n1a_left_max_lt_right_min={n1a} n1b_left_max_lt_right_separator={n1b} violated={:?}",
        violated_invariants(image));
}

#[test]
fn opus_r2_separator_family() {
    let clean = pool_with_new_inodes("opus-r2-separators", 466).memory_pool();
    report_separators("clean", &clean);
    let slots = leaf_slots(&clean);
    let mut deleted = clean.clone();
    let left = records_of(&deleted, slots[0]);
    rewrite_leaf(&mut deleted, slots[0], &left[..100]);
    report_separators("left_deleted_to_100", &deleted);
    for moved in [17usize, 66] {
        let mut borrowed = deleted.clone();
        let s = leaf_slots(&borrowed);
        let mut l = records_of(&borrowed, s[0]);
        let r = records_of(&borrowed, s[1]);
        l.extend(r[..moved].iter().cloned());
        rewrite_leaf(&mut borrowed, s[0], &l);
        rewrite_leaf(&mut borrowed, s[1], &r[moved..]);
        let left_largest = get_u64(l.last().unwrap(), 0);
        let right_smallest = get_u64(&r[moved], 0);
        for (label, base) in [("deleted", &deleted), ("borrowed", &borrowed)] {
            let ll = if label == "deleted" { 100 } else { left_largest };
            let rs = if label == "deleted" { 234 } else { right_smallest };
            for sep in [ll - 1, ll, ll + 1, rs - 1, rs, rs + 1, 234] {
                let mut image = base.clone();
                set_separator(&mut image, 1, sep);
                report_separators(&format!("{label}_moved{moved}_separator{sep}"), &image);
            }
        }
    }
}

// ── K2 代价：删除各次发布重写哪几个角色（每盘各写一次）。删空最右那片叶（234 号一个人一片）与删空中间那片叶（234..466 一片）。
#[test]
fn opus_r2_roles_of_the_deletion_publishes() {
    let mut pool = pool_with_new_inodes("opus-r2-roles-rightmost", 233);
    let intent = unlink_to_the_end(&mut pool, 234);
    for (label, v) in [("intent", &intent), ("last_batch", &pool.output)] {
        println!("OPUSR2 roles history=drop_the_rightmost_leaf_234 publish={label} txg={} count={} rewritten={:?}", v.root.checkpoint_txg.0, v.rewritten.len(), v.rewritten);
    }
    let mut pool = pool_with_new_inodes("opus-r2-roles-middle", 466);
    for inode in 234..466u64 { unlink_to_the_end(&mut pool, inode); }
    let before = pool.output.clone();
    let intent = unlink_to_the_end(&mut pool, 466);
    println!("OPUSR2 roles history=drop_the_middle_leaf_234_to_466 leaves_before={} leaves_after={}", before.inode_leaf_container_contents().len(), pool.output.inode_leaf_container_contents().len());
    for (label, v) in [("intent", &intent), ("last_batch", &pool.output)] {
        println!("OPUSR2 roles history=drop_the_middle_leaf_234_to_466 publish={label} txg={} count={} rewritten={:?}", v.root.checkpoint_txg.0, v.rewritten.len(), v.rewritten);
    }
}
