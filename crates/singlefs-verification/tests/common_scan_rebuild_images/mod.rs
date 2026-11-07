//! 扫描重建那几份用例（`scan_rebuild_agrees_with_the_walked_index.rs`、`scan_rebuild_reads_lineage_and_retirement_records.rs`）与 GPU 扫描重建对拍
//! （`crash_judge_gpu_scan_rebuild_agrees_with_the_cpu_checker.rs`）共用的搭法与造坏法：第一个文件删一半、inode 2 活着的那张镜像；
//! 中央映射叶（这几张镜像上中央映射树只有一层，根兼叶）里挪一条条目第一条位置条目的槽号、按 key 序插进一条条目。
//! 改过的映射叶载荷 CRC 与头校验和重封，新的整单元校验和沿引用链补到根槽。
#![allow(dead_code, reason = "引它的测试目标各只调其中一部分")]

use crate::common::{parameters, BuiltPool};
use crate::common_deletion_in_progress::{
    pool_with_a_three_unit_file, publish_new_inodes_and_advance_the_pool, publish_the_intent,
    publish_the_next_step_in_batches_of_one, INSTANCE_OF_THE_PUBLISHES,
    WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
};
use crate::common_pointer_head_corruption::{
    newest_root, node_bytes, replace_the_unit_and_propagate_its_checksum, the_slot_of,
    unit_bytes_at,
};
use singlefs_core::address::InodeNumber;
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::records::mapping_key_sort_key;
use singlefs_core::transaction::{
    publish_sequential_write_of_an_inode, PoolWriter, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::{data_unit_payload_capacity, seal_header_checksum};
use singlefs_format::{
    CENTRAL_MAPPING_TREE_INDEX_NODE_HEADER_BYTES, MAPPING_ENTRY_BYTES, MAPPING_KEY_BYTES,
};
use singlefs_verification::memory_pool::MemoryPool;

/// 中央映射树节点（码 2，key 宽 27）头里随条目变的几处：声明长度 8（码 2 与码 3 同在共同前缀那一处）、层级 50、最小 key 52、最大 key 79
/// （各 27 字节）、载荷 CRC 130、条目数 136（2 字节），明文头到 140（D18（块里携带什么信息） 已定项 18 的偏移表：52 起 key 区间 2k，
/// 之后诞生代号 8、fsid 8、写序 4、出生序号 4、载荷 CRC 4、预留 2、条目数 2、条目宽 2）。
const MAPPING_NODE_DECLARED_LENGTH_OFFSET: usize = 8;
const MAPPING_NODE_LEVEL_OFFSET: usize = 50;
const MAPPING_NODE_SMALLEST_KEY_OFFSET: usize = 52;
const MAPPING_NODE_LARGEST_KEY_OFFSET: usize = 79;
const MAPPING_NODE_PAYLOAD_CHECKSUM_OFFSET: usize = 130;
const MAPPING_NODE_ENTRY_COUNT_OFFSET: usize = 136;
const MAPPING_NODE_PLAIN_HEADER_END: usize = 140;
/// 中央映射叶条目里第一条位置条目的槽号（6 字节）在 key 之后、设备身份 4 字节之后（D19（块指针的结构与宽度预算） 已定项 6 / 10）。
pub const FIRST_LOCATION_SLOT_OFFSET_IN_A_MAPPING_ENTRY: usize = 27 + 4;
/// 第一个文件删一半的那张镜像上，映射条目的落点挪多远：单元区里那几次发布都没写到的一格（挪两槽会落在 inode 2 的下一个单元上，
/// 先红 I-5.1、I-1.13 报不适用）。
pub const SLOTS_A_MAPPING_LOCATION_OF_ANOTHER_FILE_IS_MOVED_BY: u64 = 4096;

/// 攻方 F 段那一形（`research/prompts/m3-multi-file-unlink-r1-opus-output.md` U4-甲）：第一个文件顺序写成三个单元，
/// 建 inode 2、按 inode 号写它两个单元，再删第一个文件：发完意图与第一批（批大小 1）。这一版里 inode 1 删除中（墓碑压着、记录还在、
/// 剩两个单元），inode 2 活着（nlink 1、两个数据单元）。
pub fn pool_with_the_first_file_half_deleted_beside_inode_two(tag: &str) -> BuiltPool {
    let mut pool = pool_with_a_three_unit_file(tag);
    publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    let content_of_inode_two: Vec<u8> = (0..2 * data_unit_payload_capacity() - 5)
        .map(|index| u8::try_from((index * 7 + 3) % 251).expect("小于 256"))
        .collect();
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let written = {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_sequential_write_of_an_inode(
            &mut writer,
            &mut pool.allocator,
            &previous,
            InodeNumber(2),
            &content_of_inode_two,
            WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
            INSTANCE_OF_THE_PUBLISHES,
        )
        .expect("按 inode 号顺序写 inode 2 两个单元")
    };
    pool.output = written;
    publish_the_intent(&mut pool);
    let after_the_first_batch = publish_the_next_step_in_batches_of_one(&mut pool);
    assert_eq!(
        (
            after_the_first_batch.data_pointers.len(),
            after_the_first_batch.deleted_inodes.clone()
        ),
        (2, vec![InodeNumber(FIRST_INODE_NUMBER)]),
        "第一批之后第一个文件剩两个单元、还在删除队列里"
    );
    pool
}

/// 删除中的对象之外那一形（判决 m3-multi-file-unlink-r1 第三节 M5 的必红世界）：最新根的中央映射树（根兼叶）里 key 是 `mapping_key`
/// 的那条条目，第一条位置条目的槽号挪 [`SLOTS_A_MAPPING_LOCATION_OF_ANOTHER_FILE_IS_MOVED_BY`] 槽；映射叶重封、新的整单元校验和沿引用链补到根槽。
/// 交回原来那个槽。
pub fn move_the_first_location_of_the_mapping_entry_keyed(
    image: &mut MemoryPool,
    mapping_key: &[u8],
) -> u64 {
    let key_width = usize::try_from(MAPPING_KEY_BYTES).expect("27");
    let entry_width = usize::try_from(MAPPING_ENTRY_BYTES).expect("55");
    let entries_start = usize::try_from(CENTRAL_MAPPING_TREE_INDEX_NODE_HEADER_BYTES).expect("169");
    assert_eq!(
        FIRST_LOCATION_SLOT_OFFSET_IN_A_MAPPING_ENTRY,
        key_width + 4,
        "位置条目的槽号在 key 与设备身份之后"
    );
    let leaf_slot = the_slot_of(&newest_root(image).mapping_root);
    let mut leaf = unit_bytes_at(image, leaf_slot, node_bytes());
    assert_eq!(
        leaf[MAPPING_NODE_LEVEL_OFFSET], 0,
        "这张镜像上中央映射树只有一层：根就是叶"
    );
    let entry_count = usize::from(u16::from_le_bytes(
        leaf[MAPPING_NODE_ENTRY_COUNT_OFFSET..MAPPING_NODE_ENTRY_COUNT_OFFSET + 2]
            .try_into()
            .expect("2 字节"),
    ));
    let entry_start = (0..entry_count)
        .map(|index| entries_start + index * entry_width)
        .find(|start| leaf[*start..*start + key_width] == *mapping_key)
        .expect("中央映射里有这把 key 的条目");
    let slot_field_start = entry_start + FIRST_LOCATION_SLOT_OFFSET_IN_A_MAPPING_ENTRY;
    let mut slot_bytes = [0u8; 8];
    slot_bytes[..6].copy_from_slice(&leaf[slot_field_start..slot_field_start + 6]);
    let original_slot = u64::from_le_bytes(slot_bytes);
    let moved_slot = original_slot + SLOTS_A_MAPPING_LOCATION_OF_ANOTHER_FILE_IS_MOVED_BY;
    leaf[slot_field_start..slot_field_start + 6].copy_from_slice(&moved_slot.to_le_bytes()[..6]);
    let payload_checksum = crc32_castagnoli(&leaf[MAPPING_NODE_PLAIN_HEADER_END..]);
    leaf[MAPPING_NODE_PAYLOAD_CHECKSUM_OFFSET..MAPPING_NODE_PAYLOAD_CHECKSUM_OFFSET + 4]
        .copy_from_slice(&payload_checksum.to_le_bytes());
    seal_header_checksum(&mut leaf, MAPPING_NODE_PLAIN_HEADER_END);
    replace_the_unit_and_propagate_its_checksum(image, leaf_slot, &leaf);
    original_slot
}

/// 最新根的中央映射树（这张镜像上是根兼叶）里按 key 序插进 `entry` 那一条：条目数、声明长度、最小与最大 key 跟着改，
/// 载荷 CRC 与头校验和重封，新的整单元校验和沿引用链补到根槽。扫描重建不读码 2，只有遍历方向读到这一条。
pub fn insert_the_entry_into_the_newest_mapping_leaf(image: &mut MemoryPool, entry: &[u8]) {
    let key_width = usize::try_from(MAPPING_KEY_BYTES).expect("27");
    let entry_width = usize::try_from(MAPPING_ENTRY_BYTES).expect("55");
    let entries_start = usize::try_from(CENTRAL_MAPPING_TREE_INDEX_NODE_HEADER_BYTES).expect("169");
    assert_eq!(entry.len(), entry_width, "要插的是一条整的映射叶条目");
    let leaf_slot = the_slot_of(&newest_root(image).mapping_root);
    let mut leaf = unit_bytes_at(image, leaf_slot, node_bytes());
    assert_eq!(
        leaf[MAPPING_NODE_LEVEL_OFFSET], 0,
        "这张镜像上中央映射树只有一层：根就是叶"
    );
    let entry_count = usize::from(u16::from_le_bytes(
        leaf[MAPPING_NODE_ENTRY_COUNT_OFFSET..MAPPING_NODE_ENTRY_COUNT_OFFSET + 2]
            .try_into()
            .expect("2 字节"),
    ));
    let mut entries: Vec<Vec<u8>> = (0..entry_count)
        .map(|index| {
            let start = entries_start + index * entry_width;
            leaf[start..start + entry_width].to_vec()
        })
        .collect();
    assert!(
        entries.windows(2).all(|neighbours| {
            mapping_key_sort_key(&neighbours[0][..key_width])
                < mapping_key_sort_key(&neighbours[1][..key_width])
        }),
        "映射叶的条目按 key 的字段序严格升序"
    );
    let sort_key_of_the_entry = mapping_key_sort_key(&entry[..key_width]);
    let position = entries.partition_point(|existing| {
        mapping_key_sort_key(&existing[..key_width]) < sort_key_of_the_entry
    });
    assert!(
        entries
            .get(position)
            .is_none_or(|next| next[..key_width] != entry[..key_width]),
        "插之前映射叶里没有这把 key"
    );
    entries.insert(position, entry.to_vec());
    assert!(
        entries_start + entries.len() * entry_width <= leaf.len(),
        "映射叶装得下多一条"
    );
    for (index, each_entry) in entries.iter().enumerate() {
        let start = entries_start + index * entry_width;
        leaf[start..start + entry_width].copy_from_slice(each_entry);
    }
    leaf[MAPPING_NODE_ENTRY_COUNT_OFFSET..MAPPING_NODE_ENTRY_COUNT_OFFSET + 2].copy_from_slice(
        &u16::try_from(entries.len())
            .expect("条目数 2 字节")
            .to_le_bytes(),
    );
    leaf[MAPPING_NODE_DECLARED_LENGTH_OFFSET..MAPPING_NODE_DECLARED_LENGTH_OFFSET + 2]
        .copy_from_slice(
            &u16::try_from(entries.len() * entry_width)
                .expect("声明长度 2 字节")
                .to_le_bytes(),
        );
    leaf[MAPPING_NODE_SMALLEST_KEY_OFFSET..MAPPING_NODE_SMALLEST_KEY_OFFSET + key_width]
        .copy_from_slice(&entries.first().expect("刚插进一条")[..key_width]);
    leaf[MAPPING_NODE_LARGEST_KEY_OFFSET..MAPPING_NODE_LARGEST_KEY_OFFSET + key_width]
        .copy_from_slice(&entries.last().expect("刚插进一条")[..key_width]);
    let payload_checksum = crc32_castagnoli(&leaf[MAPPING_NODE_PLAIN_HEADER_END..]);
    leaf[MAPPING_NODE_PAYLOAD_CHECKSUM_OFFSET..MAPPING_NODE_PAYLOAD_CHECKSUM_OFFSET + 4]
        .copy_from_slice(&payload_checksum.to_le_bytes());
    seal_header_checksum(&mut leaf, MAPPING_NODE_PLAIN_HEADER_END);
    replace_the_unit_and_propagate_its_checksum(image, leaf_slot, &leaf);
}
