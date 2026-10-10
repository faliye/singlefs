//! 第 20 批 N3 那几份坏镜像（`common_deletion_in_progress` 的四张改法表）换底子再造：改法原样，底子换成别的合法历史上的同一类版本
//! （单元数 3 / 145、建过 3 个或 470 个空 inode、批大小 1 / 100），看池级 checker 是不是仍只红目标那一条。
use std::panic::{catch_unwind, AssertUnwindSafe};

use singlefs_harness::memory_pool::MemoryPool;

use crate::common_deletion_in_progress::{
    change_the_unit_and_propagate_its_checksum, first_tombstone_container_slot_of_the_newest_root, known_bad_images_after_one_new_inode, known_bad_images_after_the_first_batch_of_a_three_unit_file,
    known_bad_images_after_the_intent_of_a_three_unit_file, known_bad_images_of_the_pending_deletion_occupancy, Mutation,
};
use crate::judge::judge;
use crate::pool::{content_of, SparsePool};
use singlefs_core::unit::data_unit_payload_capacity;

struct Bases {
    name: String,
    after_new_inodes: Option<MemoryPool>,
    intent: MemoryPool,
    first_batch: MemoryPool,
    file: MemoryPool,
}

fn bases(units: usize, inodes: u64, batch: u64) -> Bases {
    let payload = data_unit_payload_capacity();
    let mut pool = SparsePool::build(4 << 30, &content_of(3000, 0));
    if units > 1 {
        pool.sequential_write(&content_of((units - 1) * payload + payload / 2, 1));
    }
    let file = pool.published.last().expect("有文件的一版").image.clone();
    let mut after_new_inodes = None;
    if inodes > 0 {
        pool.new_inodes(inodes);
        after_new_inodes = Some(pool.published.last().expect("建 inode").image.clone());
    }
    pool.intent(1).expect("意图");
    let intent = pool.published.last().expect("意图").image.clone();
    pool.step(batch).expect("第一批");
    let first_batch = pool.published.last().expect("第一批").image.clone();
    Bases { name: format!("u{units}-n{inodes}-b{batch}"), after_new_inodes, intent, first_batch, file }
}

fn apply(base_name: &str, version: &str, base: &MemoryPool, list: Vec<(&'static str, Mutation)>) {
    for (index, (target, mutation)) in list.into_iter().enumerate() {
        let mut image = base.clone();
        let applied = catch_unwind(AssertUnwindSafe(|| mutation(&mut image)));
        if applied.is_err() {
            println!("BADIMAGE base={base_name} version={version} mutation={index} target={target} result=mutation_panicked");
            continue;
        }
        let judged = judge(&image);
        let violated: Vec<&str> = judged.violated.iter().map(|(invariant, _)| *invariant).collect();
        let only_target = violated == vec![target];
        println!("BADIMAGE base={base_name} version={version} mutation={index} target={target} only_target={only_target} violated={violated:?}");
    }
}

const RECORDS_START: usize = 136;
const RECORD_COUNT_OFFSET: usize = 69;
const DECLARED_LENGTH_OFFSET: usize = 8;
const INODE_RECORD_BYTES: usize = 140;

/// 攻方自己的通用改法（不靠底子上的号）：墓碑树第一片容器第一条墓碑的对象 ID 抬到 10^6（高于这几份底子上任何水位），只该红 I-9.6。
fn raise_the_first_tombstone_far_past_the_watermark(image: &mut MemoryPool) {
    let slot = first_tombstone_container_slot_of_the_newest_root(image);
    change_the_unit_and_propagate_its_checksum(image, slot, 32768, |container| {
        container[RECORDS_START..RECORDS_START + 8].copy_from_slice(&1_000_000u64.to_le_bytes());
    });
}

/// 攻方自己的通用改法：1 号所在那片 inode 叶（最新根的 inode 树第一片，1 号是第 0 条）里拿掉 1 号那条、后面的记录前移，单元留着，只该红 I-4.7。
fn drop_the_first_record_of_the_first_leaf(image: &mut MemoryPool) {
    use singlefs_core::bytes::ByteReader;
    use singlefs_core::pointer::NodePointer;
    use singlefs_core::records::TREE_KIND_INODE;
    use singlefs_core::unit::parse_index_node;
    let newest = crate::common_pointer_head_corruption::newest_root(image);
    let table = parse_index_node(&crate::common_pointer_head_corruption::unit_bytes_at(image, crate::common_pointer_head_corruption::the_slot_of(&newest.tree_table), 16384)).expect("树表");
    let entry = table.entries.iter().find(|entry| u16::from_le_bytes([entry[10], entry[11]]) == TREE_KIND_INODE).expect("inode 树").clone();
    let root_pointer = NodePointer::read_from(&mut ByteReader::at(&entry, 14));
    let root = parse_index_node(&crate::common_pointer_head_corruption::unit_bytes_at(image, root_pointer.locations[0].slot, 16384)).expect("inode 根");
    let child = NodePointer::read_from(&mut ByteReader::at(&root.entries[0], 34));
    change_the_unit_and_propagate_its_checksum(image, child.locations[0].slot, 32768, |leaf| {
        let count = usize::from(u16::from_le_bytes([leaf[RECORD_COUNT_OFFSET], leaf[RECORD_COUNT_OFFSET + 1]]));
        assert_eq!(u64::from_le_bytes(leaf[RECORDS_START..RECORDS_START + 8].try_into().expect("8")), 1, "第 0 条是 1 号");
        assert!(count >= 2, "叶里至少两条");
        let end = RECORDS_START + count * INODE_RECORD_BYTES;
        leaf.copy_within(RECORDS_START + INODE_RECORD_BYTES..end, RECORDS_START);
        leaf[end - INODE_RECORD_BYTES..end].fill(0);
        let remaining = u16::try_from(count - 1).expect("条数");
        leaf[RECORD_COUNT_OFFSET..RECORD_COUNT_OFFSET + 2].copy_from_slice(&remaining.to_le_bytes());
        let declared = u16::try_from((count - 1) * INODE_RECORD_BYTES).expect("长度");
        leaf[DECLARED_LENGTH_OFFSET..DECLARED_LENGTH_OFFSET + 2].copy_from_slice(&declared.to_le_bytes());
    });
}

pub fn run() {
    for (units, inodes, batch) in [(3usize, 0u64, 1u64), (145, 3, 100), (3, 470, 1), (1, 1, 144)] {
        let bases = bases(units, inodes, batch);
        apply(&bases.name, "intent", &bases.intent, known_bad_images_after_the_intent_of_a_three_unit_file());
        apply(&bases.name, "first_batch", &bases.first_batch, known_bad_images_after_the_first_batch_of_a_three_unit_file());
        if let Some(after) = &bases.after_new_inodes {
            apply(&bases.name, "after_new_inodes", after, known_bad_images_after_one_new_inode());
        }
        apply(&bases.name, "intent_generic", &bases.intent, vec![("I-9.6", Box::new(raise_the_first_tombstone_far_past_the_watermark) as Mutation)]);
        if let Some(after) = &bases.after_new_inodes {
            apply(&bases.name, "after_new_inodes_generic", after, vec![("I-4.7", Box::new(drop_the_first_record_of_the_first_leaf) as Mutation)]);
        }
        apply(&bases.name, "file", &bases.file, known_bad_images_of_the_pending_deletion_occupancy());
        apply(&bases.name, "intent", &bases.intent, known_bad_images_of_the_pending_deletion_occupancy());
    }
}
