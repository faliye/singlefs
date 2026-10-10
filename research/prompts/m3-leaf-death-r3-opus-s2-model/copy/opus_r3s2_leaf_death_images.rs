//! 攻方副本（m3-leaf-death-r3 opus-s2，只在草稿目录的仓副本里跑；checker 与 core 的副本加了几个环境变量开关，见 `walk.rs` 的
//! `opus_r3s2_mode` 与 `inode_tree.rs` 的 `OPUS_R3S2_*`）。每份镜像打一行 `OPUSR3S2 ...`：今天的 checker 判哪几条红、
//! 打开开关之后判哪几条红；不断言（判定交给 checker）。
//! K2：区间墓碑的最小改法（I-9.19、I-9.20 不重号只判整对象墓碑，形状放宽到 [起, 2⁶⁴ − 1]）放过哪些今天红的镜像。
//! K3：删 I-9.4 第三句（N1）之后，C116「分裂出来的右半容器号给错」那一族哪一形没有判据拦；写路径上把右半号写错、分隔 key 取最小 key 的整池。
mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use common::{overwrite_in_process_as_instance_one_and_advance_the_pool, sequential_write_in_process_as_instance_one_and_advance_the_pool};
use common_deletion_in_progress::{
    change_the_unit_and_propagate_its_checksum, content_of_three_data_units, first_tombstone_container_slot_of_the_newest_root,
    object_birth_of, pool_with_a_three_unit_file, pool_with_new_inodes, publish_new_inodes_and_advance_the_pool,
    turn_the_root_ring_over_by_creating_inodes, unlink_to_the_end, violated_invariants,
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
const ENTRY_IDENTITY_CONTAINER: usize = 18;
const ENTRY_CHILD_POINTER: usize = 34;

fn get_u16(b: &[u8], o: usize) -> u16 { u16::from_le_bytes(b[o..o + 2].try_into().unwrap()) }
fn get_u64(b: &[u8], o: usize) -> u64 { u64::from_le_bytes(b[o..o + 8].try_into().unwrap()) }
fn set_u16(b: &mut [u8], o: usize, v: u16) { b[o..o + 2].copy_from_slice(&v.to_le_bytes()); }
fn set_u64(b: &mut [u8], o: usize, v: u64) { b[o..o + 8].copy_from_slice(&v.to_le_bytes()); }

/// 今天的 checker 与打开 `mode` 那个开关的 checker 各判一遍。
fn both(image: &MemoryPool, mode: &str) -> (Vec<&'static str>, Vec<&'static str>) {
    std::env::remove_var(mode);
    let today = violated_invariants(image);
    std::env::set_var(mode, "1");
    let changed = violated_invariants(image);
    std::env::remove_var(mode);
    (today, changed)
}

// ── K2：区间墓碑

/// (对象, 对象出生代, 区间起, 区间止, 死亡 txg)
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

fn report_tombstones(name: &str, image: &MemoryPool) {
    let (today, minimal) = both(image, "OPUS_R3S2_RANGE_TOMBSTONES_MINIMAL");
    println!("OPUSR3S2 k2 image={name} tombstones={:?} violated_today={today:?} violated_minimal={minimal:?}", tombstones(image));
}

/// 1 号：顺序写三个单元（txg A）→ 建 2、3 号，3 号删完（墓碑树一片容器）→ 覆盖写成一个单元（txg S1，「变短」，区间墓碑该在这次写）
/// → 顺序写回三个单元（G1，「长回来」）→ 再变短（S2）→ 再长回来（G2）→ 根环转过一圈。区间墓碑的死亡 txg 与数据单元的诞生代号谁大谁小，
/// 镜像上只差这一个数。改的墓碑容器被回退候选集里每一版共用，死亡 txg 取不大于候选集里最旧那一版的 txg（G2 + 1），否则 I-9.20 的
/// 「死亡 txg ≤ 这一版的 txg」在旧版上红，那是改镜像的人造出来的，不是要判的东西。
#[test]
fn opus_r3s2_range_tombstones() {
    let mut pool = pool_with_a_three_unit_file("opus-r3s2-range");
    let birth_of_one = object_birth_of(&pool.output, 1).expect("1 号活着").0;
    let txg_three_units = pool.output.root.checkpoint_txg.0;
    publish_new_inodes_and_advance_the_pool(&mut pool, 2);
    unlink_to_the_end(&mut pool, 3);
    let shrink = overwrite_in_process_as_instance_one_and_advance_the_pool(&mut pool, b"one data unit after the shrink");
    let txg_shrink = shrink.root.checkpoint_txg.0;
    assert_eq!(shrink.data_pointers.len(), 1, "变短到一个单元");
    let first_regrow = sequential_write_in_process_as_instance_one_and_advance_the_pool(&mut pool, &content_of_three_data_units());
    assert_eq!(first_regrow.data_pointers.len(), 3, "长回三个单元");
    let second_shrink = overwrite_in_process_as_instance_one_and_advance_the_pool(&mut pool, b"one data unit after the second shrink");
    let txg_second_shrink = second_shrink.root.checkpoint_txg.0;
    let regrow = sequential_write_in_process_as_instance_one_and_advance_the_pool(&mut pool, &content_of_three_data_units());
    let txg_regrow = regrow.root.checkpoint_txg.0;
    assert_eq!(regrow.data_pointers.len(), 3, "再长回三个单元");
    let after_the_regrow = txg_regrow + 1;
    turn_the_root_ring_over_by_creating_inodes(&mut pool);
    let newest = pool.output.root.checkpoint_txg.0;
    let clean = pool.memory_pool();
    let whole_of_three = tombstones(&clean)[0];
    println!("OPUSR3S2 k2 facts birth_of_inode_one={birth_of_one} txg_three_units={txg_three_units} txg_shrink={txg_shrink} txg_first_regrow={} txg_second_shrink={txg_second_shrink} txg_regrow={txg_regrow} death_after_the_regrow={after_the_regrow} newest_txg={newest} live_units_of_inode_one={} tombstone_of_three={whole_of_three:?}", first_regrow.root.checkpoint_txg.0, regrow.data_pointers.len());
    report_tombstones("clean", &clean);
    let max = u64::MAX;
    let cases: Vec<(&str, Vec<Tombstone>)> = vec![
        // 合法形状：变短那次写的 [1, 2⁶⁴ − 1]，死亡 txg = 变短那次；之后长回来的单元都比它新。
        ("legit_tail_tombstone_of_the_shrink", vec![whole_of_three, (1, birth_of_one, 1, max, txg_shrink)]),
        // 合法形状：变短 → 长回来 → 再变短到同一个末尾（两条只差死亡 txg）。
        ("legit_two_tail_tombstones_shrink_regrow_shrink", vec![whole_of_three, (1, birth_of_one, 1, max, txg_shrink), (1, birth_of_one, 1, max, txg_second_shrink)]),
        // 坏：同一个形状，死亡 txg 晚于长回来那次（压掉活着的单元 1、2）。
        ("bad_tail_tombstone_after_the_regrow_kills_live_units", vec![whole_of_three, (1, birth_of_one, 1, max, after_the_regrow)]),
        ("bad_two_tail_tombstones_the_second_after_the_regrow", vec![whole_of_three, (1, birth_of_one, 1, max, txg_shrink), (1, birth_of_one, 1, max, after_the_regrow)]),
        ("bad_tail_tombstone_at_the_regrow_txg", vec![whole_of_three, (1, birth_of_one, 1, max, txg_regrow)]),
        // 坏：同一次变短写了两条一模一样的（写者重复写）。
        ("bad_two_identical_tail_tombstones", vec![whole_of_three, (1, birth_of_one, 1, max, txg_shrink), (1, birth_of_one, 1, max, txg_shrink)]),
        // 坏：区间墓碑点名一个从没有过的对象。
        ("bad_tail_tombstone_of_an_object_that_never_existed", vec![whole_of_three, (999, txg_shrink, 1, max, txg_shrink)]),
        // 坏：区间墓碑点名 1 号的号、出生代给错（I-9.29 那一形）。
        ("bad_tail_tombstone_with_a_wrong_object_birth", vec![whole_of_three, (1, birth_of_one + 1, 1, max, txg_shrink)]),
        // 坏：不收尾的区间（打洞那一形），最小改法照样只许尾区间。
        ("bad_inner_range_tombstone", vec![whole_of_three, (1, birth_of_one, 0, 5, txg_shrink)]),
        // 坏：区间起 = 0 的尾区间就是整对象：活对象上的整对象墓碑（对照：最小改法照判 I-9.19）。
        ("bad_whole_object_tombstone_on_live_inode_one", vec![whole_of_three, (1, birth_of_one, 0, max, txg_shrink)]),
        // 坏：死亡 txg 早于对象出生代（I-9.20 的死亡 txg 区间那一句照判）。
        ("bad_tail_tombstone_dying_before_the_object_was_born", vec![whole_of_three, (1, birth_of_one, 1, max, birth_of_one - 1)]),
        // 坏：3 号整对象墓碑换成尾区间（unlink 写成了区间墓碑；I-9.18 照判）。
        ("bad_unlink_wrote_a_tail_tombstone_instead_of_the_whole_object", vec![(whole_of_three.0, whole_of_three.1, 1, max, whole_of_three.4)]),
        // 坏：3 号整对象墓碑两条（今天的 I-9.20 不重号那一句；最小改法照判整对象那一半）。
        ("bad_two_whole_object_tombstones_of_three", vec![whole_of_three, whole_of_three]),
    ];
    for (name, records) in cases {
        let mut image = clean.clone();
        set_tombstones(&mut image, &records);
        report_tombstones(name, &image);
    }
}

// ── K3：右半容器号给错（I-9.4 第三句删掉之后）

fn inode_root_slot(image: &MemoryPool) -> SlotNumber { the_slot_of(&tree_table_entry_of_kind(image, TREE_KIND_INODE).root) }

fn leaf_slots(image: &MemoryPool) -> Vec<SlotNumber> {
    let root = parse_index_node(&unit_bytes_at(image, inode_root_slot(image), node_bytes())).expect("inode 树根");
    assert_eq!(root.level, 1, "根直接管叶容器");
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

/// 第 `index` 片叶的容器号改成 `number`（叶头与根里那条条目的身份引用一起改，I-9.2 照样成立），分隔 key 改成 `separator`。
fn renumber(image: &mut MemoryPool, index: usize, number: u64, separator: u64) {
    let slot = leaf_slots(image)[index];
    change_the_unit_and_propagate_its_checksum(image, slot, data_unit_bytes(), |leaf| set_u64(leaf, CONTAINER_NUMBER_OFFSET, number));
    let root = inode_root_slot(image);
    change_the_unit_and_propagate_its_checksum(image, root, node_bytes(), |node| {
        let start = ROOT_ENTRIES_START + index * ENTRY_BYTES;
        set_u64(node, start, separator);
        set_u64(node, start + ENTRY_IDENTITY_CONTAINER, number);
    });
}

fn set_separators_to_the_smallest_keys(image: &mut MemoryPool) {
    let slots = leaf_slots(image);
    for (i, slot) in slots.iter().enumerate() {
        let smallest = get_u64(&records_of(image, *slot)[0], 0);
        let root = inode_root_slot(image);
        change_the_unit_and_propagate_its_checksum(image, root, node_bytes(), |node| set_u64(node, ROOT_ENTRIES_START + i * ENTRY_BYTES, smallest));
    }
}

fn shape(image: &MemoryPool) -> String {
    let root = unit_bytes_at(image, inode_root_slot(image), node_bytes());
    leaf_slots(image).iter().enumerate().map(|(i, slot)| {
        let leaf = unit_bytes_at(image, *slot, data_unit_bytes());
        let recs = records_of(image, *slot);
        format!("[no={} sep={} {}..{}]", get_u64(&leaf, CONTAINER_NUMBER_OFFSET), get_u64(&root, ROOT_ENTRIES_START + i * ENTRY_BYTES),
            get_u64(&recs[0], 0), get_u64(recs.last().unwrap(), 0))
    }).collect()
}

fn report_numbers(name: &str, image: &MemoryPool) {
    let (today, n1) = both(image, "OPUS_R3S2_DROP_I94_THIRD");
    println!("OPUSR3S2 k3 image={name} leaves={} violated_today={today:?} violated_without_the_third_clause={n1:?}", shape(image));
}

/// 叶 [1..233] [234..466] [467..699] [700..700]；左片先删到 1..150（左最大 key 150 < 右片容器号 234，留出 (150, 234) 那一段），
/// 第二、三片的容器号扫一族取值；分隔 key 两种：跟着容器号走（今天的代码，右半号写错时分隔 key 一起错）与「这一版的最小 key」（N1 新形）。
#[test]
fn opus_r3s2_container_number_family() {
    let clean = pool_with_new_inodes("opus-r3s2-numbers", 699).memory_pool();
    report_numbers("clean", &clean);
    let mut base = clean.clone();
    let slots = leaf_slots(&base);
    let left = records_of(&base, slots[0]);
    rewrite_leaf(&mut base, slots[0], &left[..150]);
    report_numbers("left_deleted_to_150", &base);
    let mut base_n1 = base.clone();
    set_separators_to_the_smallest_keys(&mut base_n1);
    report_numbers("left_deleted_to_150_separators_smallest", &base_n1);
    // 第二片（左邻容器号 1、左最大 key 150、本片最小 234、右邻容器号 467）与第三片（左邻 234、左最大 466、本片最小 467、右邻 700）。
    let families: [(usize, &[u64]); 2] = [
        (1, &[0, 1, 2, 100, 150, 151, 200, 233, 234, 235, 466, 467, 468]),
        (2, &[0, 233, 234, 235, 300, 466, 467, 468, 699, 700, 701]),
    ];
    for (index, numbers) in families {
        for &number in numbers {
            let mut image = base.clone();
            renumber(&mut image, index, number, number);
            report_numbers(&format!("leaf{index}_number{number}_separator_follows_the_number"), &image);
            let mut image = base_n1.clone();
            let smallest = get_u64(&records_of(&image, leaf_slots(&image)[index])[0], 0);
            renumber(&mut image, index, number, smallest);
            report_numbers(&format!("leaf{index}_number{number}_separator_smallest_key"), &image);
        }
    }
}

/// 写路径上把右半容器号写错（新记录号 − k）、分隔 key 取这一版的最小 key（N1 新形）：建 699 个 inode 的整池。
#[test]
fn opus_r3s2_wrong_right_half_number_written_by_the_writer() {
    for (offset, n1) in [(0u64, false), (0, true), (1, true), (40, true), (1, false)] {
        std::env::set_var("OPUS_R3S2_RIGHT_HALF_NUMBER_OFFSET", offset.to_string());
        if n1 { std::env::set_var("OPUS_R3S2_N1_SEPARATOR", "1"); } else { std::env::remove_var("OPUS_R3S2_N1_SEPARATOR"); }
        let image = pool_with_new_inodes(&format!("opus-r3s2-writer-{offset}-{n1}"), 699).memory_pool();
        std::env::remove_var("OPUS_R3S2_RIGHT_HALF_NUMBER_OFFSET");
        std::env::remove_var("OPUS_R3S2_N1_SEPARATOR");
        report_numbers(&format!("writer_right_half_number_minus_{offset}_n1_separator_{n1}"), &image);
    }
}
