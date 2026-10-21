//! 上段叶条目标签 0（一个数据单元都没有）配 size > 0 是整个文件都是洞（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K5；
//! 截甲停下的第 ⑤ 处，主 agent 定三处读者统一放开）：挂载态读者、冷走读、从盘上重建上一版都收下它，读出 size 个 0、不读盘。
//! 今天的写路径写不出这一形：镜像手造——新池新建文件（3000 字节、标签 2 内联一个单元）那一版，把第一个文件那一条上段叶条目改成标签 0、
//! 载荷清零，中央映射树里那个数据单元的映射条目拿掉（映射条目数对得上单元数），新的整单元校验和沿引用链补到根槽。
mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use common::build_pool;
use common_deletion_in_progress::change_the_unit_and_propagate_its_checksum;
use common_pointer_head_corruption::{
    first_upper_leaf_entry, newest_root, node_bytes, the_slot_of, tree_table_entry_of_kind,
    unit_bytes_at,
};
use singlefs_core::address::{FileOffsetInBytes, InodeNumber};
use singlefs_core::extent_tree::ExtentUpperLeafTarget;
use singlefs_core::journal::JournalRecord;
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::records::{parse_mapping_entry, TREE_KIND_EXTENT};
use singlefs_core::recovery::{
    rebuild_version, recover, walk_to_file, JournalPolicy, RebuiltVersion, RecoveryOutcome,
};
use singlefs_core::transaction::FIRST_INODE_NUMBER;
use singlefs_core::unit::parse_index_node;
use singlefs_harness::memory_pool::MemoryPool;

/// 新池新建文件那一版第一个文件的长度。
const SIZE_OF_THE_FIRST_FILE: usize = 3000;
/// extent 上段叶条目 113：key 24（locality 8、inode 8、offset 8）之后一字节标签、载荷 88。
const EXTENT_UPPER_LEAF_ENTRY_INODE_OFFSET: usize = 8;
const EXTENT_UPPER_LEAF_ENTRY_TAG_OFFSET: usize = 24;
/// 上段叶条目标签 0：没有数据单元。
const UPPER_LEAF_TAG_NO_DATA_UNIT: u8 = 0;
/// 码 2 节点（key 宽 k）：声明长度在偏移 8、层级在 50、key 区间从 52 起两个 key、条目数在 82 + 2k、条目从 115 + 2k 起（D18（块里携带什么信息）
/// 已定项 18 的偏移表）。
const INDEX_NODE_DECLARED_LENGTH_OFFSET: usize = 8;
const INDEX_NODE_LEVEL_OFFSET: usize = 50;
const INDEX_NODE_KEY_RANGE_OFFSET: usize = 52;

fn read_u64_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8 字节"))
}

/// 节点头里的 key 区间跟不跟着条目改：按 key 寻址的树（中央映射树）跟着改；按位置寻址的 extent 上段由位置定，不动。
#[derive(Clone, Copy)]
enum KeyRangeOfTheNode {
    FollowsTheEntries,
    FixedByThePosition,
}

/// `slot` 那个码 2 节点的条目整份交给 `change` 改，条目数、声明长度跟着改（key 区间照 `key_range`），重封、校验和补到根槽。
fn rewrite_the_entries_of_the_index_node_at(
    image: &mut MemoryPool,
    slot: singlefs_core::address::SlotNumber,
    key_range: KeyRangeOfTheNode,
    change: impl FnOnce(&mut Vec<Vec<u8>>),
) {
    let parsed = parse_index_node(&unit_bytes_at(image, slot, node_bytes())).expect("节点解得开");
    let key_width = parsed.key_width;
    let entry_width = parsed.entry_width;
    let mut entries = parsed.entries;
    change(&mut entries);
    change_the_unit_and_propagate_its_checksum(image, slot, node_bytes(), |node| {
        let entries_start = 115 + 2 * key_width;
        node[entries_start..].fill(0);
        for (position, entry) in entries.iter().enumerate() {
            assert_eq!(entry.len(), entry_width, "条目宽不变");
            let start = entries_start + position * entry_width;
            node[start..start + entry_width].copy_from_slice(entry);
        }
        let count_offset = 82 + 2 * key_width;
        node[count_offset..count_offset + 2].copy_from_slice(
            &u16::try_from(entries.len())
                .expect("条目数装得进 2 字节")
                .to_le_bytes(),
        );
        node[INDEX_NODE_DECLARED_LENGTH_OFFSET..INDEX_NODE_DECLARED_LENGTH_OFFSET + 2]
            .copy_from_slice(
                &u16::try_from(entries.len() * entry_width)
                    .expect("声明长度装得进 2 字节")
                    .to_le_bytes(),
            );
        if let (KeyRangeOfTheNode::FollowsTheEntries, Some(first), Some(last)) =
            (key_range, entries.first(), entries.last())
        {
            let smallest = INDEX_NODE_KEY_RANGE_OFFSET;
            let largest = INDEX_NODE_KEY_RANGE_OFFSET + key_width;
            node[smallest..smallest + key_width].copy_from_slice(&first[..key_width]);
            node[largest..largest + key_width].copy_from_slice(&last[..key_width]);
        }
    });
}

/// 新池新建文件那一版手造成「标签 0 配 size 3000」：第一个文件那一条上段叶条目改成标签 0、载荷清零，那个数据单元的映射条目拿掉。
/// 连同那一版顶着根的 journal 记录（重建要它）交回。
fn image_whose_first_file_is_a_whole_file_hole(tag: &str) -> (MemoryPool, JournalRecord) {
    let pool = build_pool(tag);
    let record = pool.output.record.clone();
    let mut image = pool.memory_pool();
    let ExtentUpperLeafTarget::InlineDataUnit(data_unit) = first_upper_leaf_entry(&image).target
    else {
        panic!("前提：新池新建文件那一版第一个文件是标签 2、内联一个单元")
    };
    let mapping_root_slot = the_slot_of(&newest_root(&image).mapping_root);
    assert_eq!(
        unit_bytes_at(&image, mapping_root_slot, node_bytes())[INDEX_NODE_LEVEL_OFFSET],
        0,
        "前提：中央映射树只有根兼叶"
    );
    rewrite_the_entries_of_the_index_node_at(
        &mut image,
        mapping_root_slot,
        KeyRangeOfTheNode::FollowsTheEntries,
        |entries| {
            let before = entries.len();
            entries.retain(|entry| {
                let (_key, locations) = parse_mapping_entry(entry).expect("映射条目解得开");
                locations[0].slot != data_unit.locations[0].slot
            });
            assert_eq!(
                entries.len() + 1,
                before,
                "恰好一条映射条目落在那个数据单元上"
            );
        },
    );
    let extent_root_slot = the_slot_of(&tree_table_entry_of_kind(&image, TREE_KIND_EXTENT).root);
    rewrite_the_entries_of_the_index_node_at(
        &mut image,
        extent_root_slot,
        KeyRangeOfTheNode::FixedByThePosition,
        |entries| {
            let first_file_entry = entries
                .iter_mut()
                .find(|entry| {
                    read_u64_at(entry, EXTENT_UPPER_LEAF_ENTRY_INODE_OFFSET) == FIRST_INODE_NUMBER
                })
                .expect("上段叶里有第一个文件那一条");
            first_file_entry[EXTENT_UPPER_LEAF_ENTRY_TAG_OFFSET] = UPPER_LEAF_TAG_NO_DATA_UNIT;
            first_file_entry[EXTENT_UPPER_LEAF_ENTRY_TAG_OFFSET + 1..].fill(0);
        },
    );
    assert_eq!(
        first_upper_leaf_entry(&image).target,
        ExtentUpperLeafTarget::NoDataUnit,
        "前提：手造之后第一个文件那一条是标签 0"
    );
    (image, record)
}

/// 冷走读（`recovery::walk_to_file`）读出 3000 个 0；冷启动恢复（`recover`）同样读回 3000 个 0。今天之前这两条拒成
/// 「extent 树里第一个文件一个数据单元都没有」。
#[test]
fn the_cold_walk_and_the_recovery_read_a_whole_file_hole_of_tag_zero_with_a_positive_size_as_zeros()
{
    let (image, _record) = image_whose_first_file_is_a_whole_file_hole("tag-zero-cold-walk");
    let root = newest_root(&image);
    let mut mapping_fallbacks = 0;
    assert_eq!(
        walk_to_file(&image, &root, &mut mapping_fallbacks).expect("冷走读收下标签 0"),
        Some(vec![0u8; SIZE_OF_THE_FIRST_FILE]),
        "冷走读读出 size 个 0"
    );
    match recover(&image, JournalPolicy::Consult).outcome {
        RecoveryOutcome::FileRead { content, .. } => {
            assert!(
                content == vec![0u8; SIZE_OF_THE_FIRST_FILE],
                "恢复读回 size 个 0（读回 {} 字节）",
                content.len()
            );
        }
        other @ (RecoveryOutcome::NoFile { .. } | RecoveryOutcome::Failed { .. }) => {
            panic!("恢复收下标签 0 配 size > 0，实际 {other:?}")
        }
    }
}

/// 从盘上重建上一版（可写挂载建上一版那一步）收下它：第一个文件那条记录 size 3000、一个数据指针都没有。
#[test]
fn rebuilding_the_version_accepts_tag_zero_with_a_positive_size_and_no_data_unit() {
    let (image, record) = image_whose_first_file_is_a_whole_file_hole("tag-zero-rebuild");
    let root = newest_root(&image);
    match rebuild_version(&image, &root, Some(record)) {
        Ok(RebuiltVersion::WithFile(version)) => {
            assert_eq!(
                (
                    version.inode_record.map(|inode_record| inode_record.size),
                    version.data_pointers.len()
                ),
                (Some(3000), 0),
                "重建出 size 3000、没有数据单元的第一个文件"
            );
        }
        other @ (Ok(RebuiltVersion::WithoutFile | RebuiltVersion::WithTreesAndNoFile(_))
        | Err(_)) => {
            panic!("重建收下标签 0 配 size > 0，实际 {other:?}")
        }
    }
}

/// 挂载态读者（只读挂载、打开第一个文件、从 0 读到 size）同样读出 3000 个 0。
#[test]
fn the_mounted_reader_reads_a_whole_file_hole_of_tag_zero_with_a_positive_size_as_zeros() {
    let (image, _record) = image_whose_first_file_is_a_whole_file_hole("tag-zero-mounted");
    let mounted = mount_read_only(&image).expect("只读挂载");
    let file = mounted
        .mounted
        .open_file(&image, InodeNumber(FIRST_INODE_NUMBER))
        .expect("打开第一个文件");
    assert_eq!(file.data_unit_count(), 0, "标签 0：一个数据单元都没有");
    let read = file
        .read_at(&image, FileOffsetInBytes(0), 3000)
        .expect("从 0 读到 size");
    assert!(
        read.bytes == vec![0u8; SIZE_OF_THE_FIRST_FILE],
        "读出 size 个 0"
    );
}
