//! 攻方副本（m3-leaf-death-r3 opus，只在草稿目录的仓副本里跑）：K2 区间墓碑最小改法放过的坏镜像、K3 删 I-9.4 第三句之后放过的坏镜像。
//! 每份镜像打一行 `OPUSR3 ...`，不断言（判定交给 checker；checker 的改法由 `patch_checker.py` 加的环境变量开关选，见那份文件头）。
mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use common_deletion_in_progress::{
    change_the_unit_and_propagate_its_checksum, first_tombstone_container_slot_of_the_newest_root, object_birth_of,
    pool_with_a_three_unit_file, pool_with_inode_three_unlinked_and_the_root_ring_turned_over, pool_with_new_inodes,
    publish_new_inodes_and_advance_the_pool, publish_the_intent, turn_the_root_ring_over_by_creating_inodes, unlink_to_the_end,
    violated_invariants,
};
use common_pointer_head_corruption::{data_unit_bytes, node_bytes, the_slot_of, tree_table_entry_of_kind, unit_bytes_at};
use singlefs_core::address::SlotNumber;
use singlefs_core::bytes::ByteReader;
use singlefs_core::pointer::NodePointer;
use singlefs_core::records::TREE_KIND_INODE;
use singlefs_core::unit::{data_unit_payload_capacity, parse_index_node};
use singlefs_harness::memory_pool::MemoryPool;

const RECORD_COUNT_OFFSET: usize = 69;
const DECLARED_LENGTH_OFFSET: usize = 8;
const RECORDS_START: usize = 136;
const TOMBSTONE_BYTES: usize = 40;
const INODE_RECORD_BYTES: usize = 140;
const INODE_RECORD_SIZE_OFFSET: usize = 40;
const CONTAINER_NUMBER_OFFSET: usize = 53;
const ROOT_ENTRIES_START: usize = 131;
const ENTRY_BYTES: usize = 120;
const ENTRY_IDENTITY_CONTAINER: usize = 18;
const ENTRY_CHILD_POINTER: usize = 34;

fn get_u16(b: &[u8], o: usize) -> u16 { u16::from_le_bytes(b[o..o + 2].try_into().unwrap()) }
fn get_u64(b: &[u8], o: usize) -> u64 { u64::from_le_bytes(b[o..o + 8].try_into().unwrap()) }
fn set_u16(b: &mut [u8], o: usize, v: u16) { b[o..o + 2].copy_from_slice(&v.to_le_bytes()); }
fn set_u64(b: &mut [u8], o: usize, v: u64) { b[o..o + 8].copy_from_slice(&v.to_le_bytes()); }

fn mode() -> String {
    format!(
        "range={} drop_i94_third={}",
        std::env::var("OPUS_R3_RANGE").unwrap_or_else(|_| "today".into()),
        std::env::var_os("OPUS_R3_DROP_I94_THIRD").is_some()
    )
}

// ── K2：区间墓碑。墓碑记录 (对象, 对象出生代, 区间起, 区间止, 死亡 txg)，区间按单元序号读（取字节偏移时同形）。
type Tombstone = (u64, u64, u64, u64, u64);

fn tombstones(image: &MemoryPool) -> Vec<Tombstone> {
    let c = unit_bytes_at(image, first_tombstone_container_slot_of_the_newest_root(image), data_unit_bytes());
    (0..usize::from(get_u16(&c, RECORD_COUNT_OFFSET))).map(|i| {
        let o = RECORDS_START + i * TOMBSTONE_BYTES;
        (get_u64(&c, o), get_u64(&c, o + 8), get_u64(&c, o + 16), get_u64(&c, o + 24), get_u64(&c, o + 32))
    }).collect()
}

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

/// 镜像上每个活对象（这一版 inode 树里 nlink ≥ 1 的记录）被它自己那几条区间墓碑（同号同出生代、不是整对象）压住的单元数：
/// 区间起落在 size 折成的单元数之内，[区间起, 单元数) 那几个单元按区间墓碑的读法就是死的（真值活着）。
fn live_units_under_range_tombstones(live: &[(u64, u64, u64)], ts: &[Tombstone]) -> u64 {
    let cap = data_unit_payload_capacity() as u64;
    let mut n = 0;
    for &(ino, birth, size) in live {
        let units = size.div_ceil(cap);
        let start = ts.iter().filter(|t| t.0 == ino && t.1 == birth && !(t.2 == 0 && t.3 == u64::MAX)).map(|t| t.2).min();
        if let Some(s) = start { n += units.saturating_sub(s); }
    }
    n
}

fn report_tombstones(name: &str, image: &MemoryPool, live: &[(u64, u64, u64)]) {
    let ts = tombstones(image);
    println!(
        "OPUSR3 k2 {} image={name} tombstones={ts:?} live_units_under_a_range_tombstone={} violated={:?}",
        mode(), live_units_under_range_tombstones(live, &ts), violated_invariants(image)
    );
}

fn live_objects(output: &singlefs_core::transaction::TransactionOutput) -> Vec<(u64, u64, u64)> {
    output.inode_leaf_container_contents().iter().flat_map(|c| c.records.iter())
        .filter(|r| r.nlink >= 1).map(|r| (r.inode, r.object_birth.0, r.size)).collect()
}

/// 底：1 号三个数据单元（新池新建文件之后顺序写），再建 2、3 号两个空 inode（分两次建），3 号删完，根环转过一圈。
#[test]
fn opus_r3_range_tombstone_minimal_fixes() {
    let mut pool = pool_with_a_three_unit_file("opus-r3-range");
    publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    unlink_to_the_end(&mut pool, 3);
    turn_the_root_ring_over_by_creating_inodes(&mut pool);
    let live = live_objects(&pool.output);
    let birth_of_one = object_birth_of(&pool.output, 1).expect("1 号活着").0;
    let birth_of_two = object_birth_of(&pool.output, 2).expect("2 号活着").0;
    let clean = pool.memory_pool();
    let three = tombstones(&clean)[0];
    let (death, newest) = (three.4, pool.output.root.checkpoint_txg.0);
    println!("OPUSR3 k2 {} facts live={live:?} tombstone_of_three={three:?} newest_txg={newest}", mode());
    let cases: Vec<(&str, Vec<Tombstone>)> = vec![
        ("clean", vec![three]),
        // 今天已有的坏镜像（`pool_checker_judges_the_deletion_state_of_every_walked_version.rs` 那几条的同形，墓碑整份写）。
        ("today_whole_tombstone_retargeted_to_live_two", vec![(2, birth_of_two, 0, u64::MAX, death)]),
        ("today_range_end_five", vec![(3, three.1, 0, 5, death)]),
        ("today_death_zero", vec![(3, three.1, 0, u64::MAX, 0)]),
        ("today_death_after_every_version", vec![(3, three.1, 0, u64::MAX, newest + 1)]),
        ("today_two_whole_copies", vec![three, three]),
        // 新的：区间起被写错。
        ("deleted_three_start_one_no_whole_tombstone_left", vec![(3, three.1, 1, u64::MAX, death)]),
        ("deleted_three_whole_plus_a_start_one_copy", vec![three, (3, three.1, 1, u64::MAX, death)]),
        ("live_one_start_one_covers_two_live_units", vec![three, (1, birth_of_one, 1, u64::MAX, death)]),
        ("live_one_start_two_covers_one_live_unit", vec![three, (1, birth_of_one, 2, u64::MAX, death)]),
        ("live_one_start_three_covers_nothing", vec![three, (1, birth_of_one, 3, u64::MAX, death)]),
        ("live_two_start_one_empty_file", vec![three, (2, birth_of_two, 1, u64::MAX, death)]),
        ("live_one_two_equal_ranges_two_deaths", vec![three, (1, birth_of_one, 3, u64::MAX, death - 1), (1, birth_of_one, 3, u64::MAX, death)]),
        ("live_one_range_end_not_max", vec![three, (1, birth_of_one, 1, 2, death)]),
        ("whole_tombstone_retargeted_to_live_one_start_one", vec![(1, birth_of_one, 1, u64::MAX, death)]),
    ];
    for (name, ts) in cases {
        let mut image = clean.clone();
        set_tombstones(&mut image, &ts);
        report_tombstones(name, &image, &live);
    }
}

/// 删除途中（意图之后、队列里还有 1 号）：意图那次写的整对象墓碑区间起被写成 1。
#[test]
fn opus_r3_range_tombstone_during_the_unlink() {
    let mut pool = pool_with_a_three_unit_file("opus-r3-range-queued");
    publish_the_intent(&mut pool);
    let live = live_objects(&pool.output);
    let clean = pool.memory_pool();
    let one = tombstones(&clean)[0];
    report_tombstones("queued_clean", &clean, &live);
    let mut image = clean.clone();
    set_tombstones(&mut image, &[(one.0, one.1, 1, u64::MAX, one.4)]);
    report_tombstones("queued_start_one", &image, &live);
}

// ── K3：分裂出来的右半容器号给错（C116 那一族）。三片叶 [1..233]、[234..466]、[467]；改第二片的容器号（叶头与根条目身份引用两处一起改），
// 分隔 key 取 N1 新形（孩子这一版的最小 key）或今天的写法（= 容器号）。
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

fn set_root_entry(image: &mut MemoryPool, index: usize, separator: u64, container: u64) {
    let slot = inode_root_slot(image);
    change_the_unit_and_propagate_its_checksum(image, slot, node_bytes(), |node| {
        let start = ROOT_ENTRIES_START + index * ENTRY_BYTES;
        set_u64(node, start, separator);
        set_u64(node, start + ENTRY_IDENTITY_CONTAINER, container);
    });
}

fn set_leaf_container_number(image: &mut MemoryPool, slot: SlotNumber, number: u64) {
    change_the_unit_and_propagate_its_checksum(image, slot, data_unit_bytes(), |leaf| set_u64(leaf, CONTAINER_NUMBER_OFFSET, number));
}

fn report_leaves(name: &str, image: &MemoryPool) {
    let root = unit_bytes_at(image, inode_root_slot(image), node_bytes());
    let mut shape = String::new();
    for (i, slot) in leaf_slots(image).iter().enumerate() {
        let leaf = unit_bytes_at(image, *slot, data_unit_bytes());
        let recs = records_of(image, *slot);
        shape.push_str(&format!("[no={} sep={} {}..{}]", get_u64(&leaf, CONTAINER_NUMBER_OFFSET),
            get_u64(&root, ROOT_ENTRIES_START + i * ENTRY_BYTES), get_u64(&recs[0], 0), get_u64(recs.last().unwrap(), 0)));
    }
    println!("OPUSR3 k3 {} image={name} leaves={shape} violated={:?}", mode(), violated_invariants(image));
}

#[test]
fn opus_r3_right_half_container_number_family() {
    let clean = pool_with_new_inodes("opus-r3-split-numbers", 466).memory_pool();
    report_leaves("clean", &clean);
    let slots = leaf_slots(&clean);
    // 左片删掉 200..=233 的那一份：左最大 key 199，左右之间留一段空号 200..233。
    let mut gap = clean.clone();
    let left = records_of(&gap, slots[0]);
    rewrite_leaf(&mut gap, slots[0], &left[..199]);
    report_leaves("gap_clean", &gap);
    for (base_name, base, numbers) in [
        ("dense", &clean, vec![1u64, 2, 100, 233, 234, 235, 466, 467, 500]),
        ("gap", &gap, vec![1u64, 150, 199, 200, 210, 233, 234, 235]),
    ] {
        for c in numbers {
            for (sep_name, sep) in [("n1_min_key", 234u64), ("today_equals_number", c)] {
                let mut image = base.clone();
                set_leaf_container_number(&mut image, slots[1], c);
                set_root_entry(&mut image, 1, sep, c);
                report_leaves(&format!("{base_name}_right_half_number_{c}_separator_{sep_name}"), &image);
            }
        }
    }
}
