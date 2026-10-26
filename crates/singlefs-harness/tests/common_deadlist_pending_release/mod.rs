//! I-3.6（deadlist 紧界） 新并进的三格（待放集的上界、每个纪元恰好落进一份区间、deadlist key 在同一版中央映射里查得到）的盘上世界的搭建：
//! 写路径写出 deadlist（第一个文件之后建快照、快照下覆盖写、删快照），再只改点名的那一个单元（deadlist 叶、树表、中央映射叶），新的整单元校验和
//! 沿引用链补到根槽（`common_pointer_head_corruption::change_the_unit_and_propagate_its_checksum_on`）。harness 档的池级 checker 那一遍
//! （`deadlist_pending_release_bound_and_mapping_lookup_must_red_worlds.rs`）与 checker 档的 GPU 对拍
//! （`crash_judge_gpu_agrees_with_the_cpu_judge_on_the_pending_release_bound.rs`，经 `#[path]` 引）用同一份。
//! 用这一份的测试文件同时声明 `mod common;` 与 `mod common_pointer_head_corruption;`。
#![allow(dead_code, reason = "每个测试文件各自只用到其中一部分")]

use singlefs_core::address::{CheckpointTxg, InstanceGeneration, SlotNumber, TreeIdentifier};
use singlefs_core::records::{TREE_KIND_DEADLIST, TREE_KIND_SNAPSHOT_INODE_ROOT};
use singlefs_core::transaction::{
    publish_create_snapshot, publish_snapshot_destruction_intent,
    walk_the_deletion_queue_until_it_is_empty, PoolWriter, TransactionOutput,
};
use singlefs_core::unit::{build_index_node, parse_index_node};
use singlefs_core::unlink::UnlinkBatchSize;
use singlefs_format::MAPPING_KEY_BYTES;
use singlefs_harness::memory_pool::MemoryPool;

use crate::common::{
    build_pool, parameters, publish_overwrite_in_process, BuiltPool, PoolImageGeometry,
    FIXED_WRITE_TIME_SECONDS,
};
use crate::common_pointer_head_corruption::{
    change_the_unit_and_propagate_its_checksum_on, newest_root, node_bytes, the_slot_of,
    tree_table_entry_of_kind, unit_bytes_at,
};

/// deadlist 条目 43 字节：头的树 ID 8 + 纪元 8 + 中央映射 key 27（D5（快照 / 空间记账机制） 已定项 12）；纪元在偏移 8。
pub const DEADLIST_ENTRY_BYTES: usize = 8 + 8 + 27;
const DEADLIST_ENTRY_EPOCH_OFFSET: usize = 8;
/// deadlist 条目里映射 key 那 27 字节（头 8、纪元 8 之后）。
const MAPPING_KEY_BYTES_IN_A_DEADLIST_ENTRY: usize = 27;
/// 中央映射 key 的类标签：码 1 数据单元是 1（D18（块里携带什么信息） 已定项 11 的登记表）。
pub const MAPPING_KEY_UNIT_CLASS_OF_A_DATA_UNIT: u8 = 1;
/// 树表条目字段表（D8（核心索引结构） 已定项 8）：树 ID 在偏移 0、`previous_snapshot_txg` 在偏移 100。
const TREE_TABLE_ENTRY_TREE_IDENTIFIER_BYTES: usize = 8;
const TREE_TABLE_ENTRY_PREVIOUS_SNAPSHOT_TXG_OFFSET: usize = 100;
/// 第一个文件在 txg 3；之后头一个快照建在 txg 4、第二个建在 txg 5。
pub const TXG_OF_THE_FIRST_SNAPSHOT: u64 = 4;
pub const TXG_OF_THE_SECOND_SNAPSHOT: u64 = 5;

fn geometry() -> PoolImageGeometry {
    PoolImageGeometry::of_four_gibibyte_devices_under_the_common_parameters()
}

/// 接着 `pool.output` 建一个快照，池接到那一版上。
pub fn create_a_snapshot_and_advance(pool: &mut BuiltPool) {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    pool.output = publish_create_snapshot(
        &mut writer,
        &mut pool.allocator,
        &previous,
        InstanceGeneration(1),
    )
    .expect("建快照");
}

/// 同一个进程里接着 `pool.output` 覆盖写一次（与第一个文件不同的一份 2700 字节），池接到那一版上。
pub fn overwrite_and_advance(pool: &mut BuiltPool, seed: u32) {
    let content: Vec<u8> = (0..2700_u32)
        .map(|index| u8::try_from((index * 13 + seed) % 239).expect("小于 256"))
        .collect();
    let previous = pool.output.clone();
    pool.output = publish_overwrite_in_process(
        pool,
        &previous,
        &content,
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("覆盖写");
}

/// 最新那一版里诞生 txg 是 `snapshot_txg` 的那个快照（码 14）的树号。
pub fn snapshot_tree_born_at(output: &TransactionOutput, snapshot_txg: u64) -> TreeIdentifier {
    output
        .tree_table_entries
        .iter()
        .find(|entry| {
            entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT
                && entry.birth_txg == CheckpointTxg(snapshot_txg)
        })
        .expect("树表里有这一 txg 建的快照")
        .tree
}

/// 删 `snapshot` 的意图那次发布，池接到那一版上。
pub fn publish_the_destruction_intent_of_and_advance(
    pool: &mut BuiltPool,
    snapshot: TreeIdentifier,
) {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    pool.output = publish_snapshot_destruction_intent(
        &mut writer,
        &mut pool.allocator,
        &previous,
        snapshot,
        InstanceGeneration(1),
    )
    .expect("删快照的意图");
}

/// 走空（销毁中的快照）到底，池接到最后一版上。
pub fn walk_the_destruction_to_the_end_and_advance(pool: &mut BuiltPool) {
    let publish_parameters = parameters();
    let mut current = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let walk = walk_the_deletion_queue_until_it_is_empty(
        &mut writer,
        &mut pool.allocator,
        &mut current,
        UnlinkBatchSize::OneExtentLowerLeaf,
        InstanceGeneration(1),
    );
    walk.stopped_by.expect("走空走到底");
    pool.output = current;
}

/// 第一个文件（txg 3）之后建一个快照（txg 4）、快照下覆盖写一次（txg 5）：活头换下的四个单元（数据单元、extent 树根、inode 叶容器与根）
/// 都生在 txg 3，进 deadlist（头 = 活头 inode 树、纪元 = 4），映射条目留存。
pub fn pool_with_a_snapshot_and_an_overwrite_under_it(tag: &str) -> BuiltPool {
    let mut pool = build_pool(tag);
    create_a_snapshot_and_advance(&mut pool);
    overwrite_and_advance(&mut pool, 7);
    pool
}

/// [`pool_with_a_snapshot_and_an_overwrite_under_it`] 再写删那个快照的意图：意图与最后一批之间，四条条目都在待放集里、还没放
/// （lo = 4、P = 0，下一侧是活头）。
pub fn pool_after_the_destruction_intent_under_an_overwrite(tag: &str) -> BuiltPool {
    let mut pool = pool_with_a_snapshot_and_an_overwrite_under_it(tag);
    let snapshot = snapshot_tree_born_at(&pool.output, TXG_OF_THE_FIRST_SNAPSHOT);
    publish_the_destruction_intent_of_and_advance(&mut pool, snapshot);
    pool
}

/// 建快照 X（txg 4）、Y（txg 5），快照下覆盖写（四个单元生在 txg 3、进 deadlist、纪元 5）；删 Y（意图与最后一批：待放 [5, ∞) 里 birth > 4 的
/// 一条都没有，四条都留着、纪元仍是 5，归活头那一侧 [4, ∞)）；再写删 X 的意图（lo = 4、P = 0）：四条纪元 5 都大于 lo、小于这一版的 txg、
/// birth 3 > 0，都在待放集里。合法态。
pub fn pool_whose_pending_entries_lie_above_the_lower_bound_of_the_intent(tag: &str) -> BuiltPool {
    let mut pool = build_pool(tag);
    create_a_snapshot_and_advance(&mut pool);
    create_a_snapshot_and_advance(&mut pool);
    overwrite_and_advance(&mut pool, 7);
    let earlier_snapshot = snapshot_tree_born_at(&pool.output, TXG_OF_THE_FIRST_SNAPSHOT);
    let later_snapshot = snapshot_tree_born_at(&pool.output, TXG_OF_THE_SECOND_SNAPSHOT);
    publish_the_destruction_intent_of_and_advance(&mut pool, later_snapshot);
    walk_the_destruction_to_the_end_and_advance(&mut pool);
    publish_the_destruction_intent_of_and_advance(&mut pool, earlier_snapshot);
    pool
}

/// 槽 `slot` 上那个码 2 节点（叶，两盘同槽）换成同一个头（树、层级、key 宽、出生身份、条目宽）、`change` 改过的条目，key 区间贴紧首末两条的
/// 前 `key_bytes` 字节，重封、写回，新的整单元校验和沿引用链补到根槽。
pub fn change_the_entries_of_the_leaf_at(
    image: &mut MemoryPool,
    slot: SlotNumber,
    key_bytes: usize,
    change: impl FnOnce(&mut Vec<Vec<u8>>),
) {
    let leaf = unit_bytes_at(image, slot, node_bytes());
    let header = parse_index_node(&leaf).expect("节点解得开");
    assert_eq!(header.level, 0, "这几棵树在这几个池上都只有一层：根兼叶");
    let mut entries = header.entries.clone();
    change(&mut entries);
    let rebuilt = build_index_node(
        header.tree,
        header.level,
        header.key_width,
        &entries.first().expect("改完至少一条")[..key_bytes],
        &entries.last().expect("改完至少一条")[..key_bytes],
        header.birth_txg,
        &parameters().filesystem_identifier,
        header.instance,
        header.birth_sequence,
        u16::try_from(header.entry_width).expect("条目宽装得进 u16"),
        &entries,
    );
    change_the_unit_and_propagate_its_checksum_on(image, &geometry(), slot, node_bytes(), |unit| {
        *unit = rebuilt;
    });
}

/// deadlist key 43 字节按字段 [8, 8, 1, 8, 8, 4, 6] 逐段读成小端无符号数（盘上的序就是逐段比，`KEY_SCHEMA_DEADLIST`）。
fn deadlist_key_fields(entry: &[u8]) -> Vec<u64> {
    let mut fields = Vec::new();
    let mut offset = 0;
    for width in [8usize, 8, 1, 8, 8, 4, 6] {
        let mut wide = [0u8; 8];
        wide[..width].copy_from_slice(&entry[offset..offset + width]);
        fields.push(u64::from_le_bytes(wide));
        offset += width;
    }
    fields
}

/// 最新那一版 deadlist 树（单层）的叶换成 `change` 改过的条目，按盘上的 key 序重排。
pub fn change_the_newest_deadlist_leaf(
    image: &mut MemoryPool,
    change: impl FnOnce(&mut Vec<Vec<u8>>),
) {
    let leaf_slot = the_slot_of(&tree_table_entry_of_kind(image, TREE_KIND_DEADLIST).root);
    change_the_entries_of_the_leaf_at(image, leaf_slot, DEADLIST_ENTRY_BYTES, |entries| {
        change(entries);
        entries.sort_by_key(|entry| deadlist_key_fields(entry));
    });
}

pub fn epoch_of_a_deadlist_entry(entry: &[u8]) -> u64 {
    u64::from_le_bytes(
        entry[DEADLIST_ENTRY_EPOCH_OFFSET..DEADLIST_ENTRY_EPOCH_OFFSET + 8]
            .try_into()
            .expect("8 字节"),
    )
}

/// deadlist 条目里钉的是码 1 数据单元的那一条（条目末尾 27 字节是映射 key，首字节是类标签）。
pub fn pins_a_data_unit(entry: &[u8]) -> bool {
    entry[DEADLIST_ENTRY_BYTES - MAPPING_KEY_BYTES_IN_A_DEADLIST_ENTRY]
        == MAPPING_KEY_UNIT_CLASS_OF_A_DATA_UNIT
}

/// 最新那一版 deadlist 里钉数据单元的那一条纪元由 `expected_epoch` 改成 `new_epoch`。
pub fn change_the_epoch_of_the_data_unit_entry(
    image: &mut MemoryPool,
    expected_epoch: u64,
    new_epoch: u64,
) {
    change_the_newest_deadlist_leaf(image, |entries| {
        let entry = entries
            .iter_mut()
            .find(|entry| pins_a_data_unit(entry))
            .expect("deadlist 里有一条钉数据单元");
        assert_eq!(
            epoch_of_a_deadlist_entry(entry),
            expected_epoch,
            "改之前的纪元"
        );
        entry[DEADLIST_ENTRY_EPOCH_OFFSET..DEADLIST_ENTRY_EPOCH_OFFSET + 8]
            .copy_from_slice(&new_epoch.to_le_bytes());
    });
}

/// 必红世界（判决 `research/prompts/m4-c145-destruction-bound-r1-main-verification.md` 第二节 B2「纪元抬到本版 txg 以上」）：
/// [`pool_after_the_destruction_intent_under_an_overwrite`] 那一版上，钉数据单元的那一条纪元由 S.txg 4 抬到意图那一版的 txg + 1。
pub fn image_whose_pending_entry_epoch_is_raised_above_the_txg_of_the_version(
    tag: &str,
) -> MemoryPool {
    let pool = pool_after_the_destruction_intent_under_an_overwrite(tag);
    let intent_txg = pool.output.root.checkpoint_txg.0;
    let mut image = pool.memory_pool();
    change_the_epoch_of_the_data_unit_entry(&mut image, TXG_OF_THE_FIRST_SNAPSHOT, intent_txg + 1);
    image
}

/// 必红世界（C145 第一条：每个纪元恰好落进一份区间）：[`pool_with_a_snapshot_and_an_overwrite_under_it`] 那一版上，钉数据单元的那一条
/// deadlist 条目纪元由 4 改成 1；树表里快照那条码 14 的前驱由 0 改成 2（前驱链照样首尾相接：活头的前驱仍是 S.txg 4）。纪元 1 比最早那一侧的
/// 前驱 2 小，落不进任何一侧。
pub fn image_whose_deadlist_epoch_falls_below_every_side_of_a_contiguous_chain(
    tag: &str,
) -> MemoryPool {
    let pool = pool_with_a_snapshot_and_an_overwrite_under_it(tag);
    let snapshot = snapshot_tree_born_at(&pool.output, TXG_OF_THE_FIRST_SNAPSHOT);
    let mut image = pool.memory_pool();
    change_the_epoch_of_the_data_unit_entry(&mut image, TXG_OF_THE_FIRST_SNAPSHOT, 1);
    let tree_table_slot = the_slot_of(&newest_root(&image).tree_table);
    change_the_entries_of_the_leaf_at(
        &mut image,
        tree_table_slot,
        TREE_TABLE_ENTRY_TREE_IDENTIFIER_BYTES,
        |entries| {
            let snapshot_entry = entries
                .iter_mut()
                .find(|entry| {
                    entry[..TREE_TABLE_ENTRY_TREE_IDENTIFIER_BYTES] == snapshot.0.to_le_bytes()
                })
                .expect("树表里有快照那条码 14");
            snapshot_entry[TREE_TABLE_ENTRY_PREVIOUS_SNAPSHOT_TXG_OFFSET
                ..TREE_TABLE_ENTRY_PREVIOUS_SNAPSHOT_TXG_OFFSET + 8]
                .copy_from_slice(&2u64.to_le_bytes());
        },
    );
    image
}

/// 必红世界（C300：deadlist key 在同一版中央映射里查得到）：[`pool_with_a_snapshot_and_an_overwrite_under_it`] 那一版上，钉数据单元的那一条
/// deadlist 条目的留存映射条目从中央映射叶里删掉、重封到根。
pub fn image_whose_kept_mapping_entry_of_a_deadlist_key_is_deleted(tag: &str) -> MemoryPool {
    let pool = pool_with_a_snapshot_and_an_overwrite_under_it(tag);
    let kept_key_of_the_data_unit = pool
        .output
        .mapping_entries_kept_for_the_snapshots
        .iter()
        .map(|(key, _)| key.clone())
        .find(|key| key[0] == MAPPING_KEY_UNIT_CLASS_OF_A_DATA_UNIT)
        .expect("快照下覆盖写换下的数据单元留着一条映射条目");
    assert!(
        pool.output
            .deadlist_entries
            .iter()
            .any(|entry| entry.mapping_key == kept_key_of_the_data_unit),
        "这把 key 被 deadlist 记着"
    );
    let mut image = pool.memory_pool();
    let mapping_root_slot = the_slot_of(&newest_root(&image).mapping_root);
    let mapping_key_bytes = usize::try_from(MAPPING_KEY_BYTES).expect("27");
    change_the_entries_of_the_leaf_at(
        &mut image,
        mapping_root_slot,
        mapping_key_bytes,
        |entries| {
            let before = entries.len();
            entries.retain(|entry| entry[..mapping_key_bytes] != kept_key_of_the_data_unit[..]);
            assert_eq!(entries.len() + 1, before, "映射叶里恰好一条是这把 key");
        },
    );
    image
}
