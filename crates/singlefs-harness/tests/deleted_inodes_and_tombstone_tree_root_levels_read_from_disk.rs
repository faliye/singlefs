//! C713（删除线两个树根的层级没有条款）：删除那两棵树的根节点头里的层级字节，从盘上读回来按写者现行的值比——
//! `deleted_inodes` 根是层级 0 的单节点（`transaction` 里写 `deleted_inodes` 根那一处，池级 checker 的删除态扫描也按层级 0 读它，
//! `walk.rs` 的 `deleted_inodes_queue_without_judging`）；墓碑树根层级取容器层数，根直接管容器时 1（`container_tree::ContainerTreeShape::root_level`，
//! 池级 checker 读墓碑树按根层级 ≥ 1，`largest_published_tombstone_object_identifier`）。条款（D8（核心索引结构） 已定项 16、D18（块里携带什么信息）
//! 已定项 24 各一句）由 kb 那一半写定；这一条钉的是盘上两份副本的层级字节。
//!
//! 盘面：三个数据单元的文件发完意图（`tests/common_deletion_in_progress/mod.rs` 的 `publish_the_intent`）：队列 {1}，墓碑树一条墓碑、一个容器。
mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use common_deletion_in_progress::{pool_with_a_three_unit_file, publish_the_intent};
use singlefs_core::address::DeviceIdentity;
use singlefs_core::transaction::TransactionOutput;
use singlefs_core::unit::{parse_index_node, UNIT_CLASS_INDEX_NODE};
use singlefs_format::{
    NODE_BYTES, SLOT_BYTES, TREE_IDENTIFIER_DELETED_INODES, TREE_IDENTIFIER_TOMBSTONE,
};
use singlefs_harness::memory_pool::MemoryPool;

/// 码 2 头里类标签与层级那两个字节的偏移（`singlefs_core::unit::build_index_node` 的写法：magic 4 + 版本 2 之后是类标签 6；
/// 公共前缀到 42 为止，之后树 ID 42..50、层级 50、key 宽 51）。
const UNIT_CLASS_TAG_OFFSET: usize = 6;
const INDEX_NODE_LEVEL_OFFSET: usize = 50;

/// `deleted_inodes` 根的层级：单节点、条目就是号。
const LEVEL_OF_THE_DELETED_INODES_ROOT: u8 = 0;

/// 墓碑树只有一个容器、根直接管它时根的层级。
const LEVEL_OF_A_TOMBSTONE_ROOT_OVER_ONE_LAYER_OF_CONTAINERS: u8 = 1;

/// `output` 那一版树表里 `tree` 那棵树的根，在盘上两份副本各自的码 2 头：(盘, 类标签字节, 层级字节, 解出来的层级与树号)。
fn root_headers_on_disk(
    image: &MemoryPool,
    output: &TransactionOutput,
    tree: u64,
) -> Vec<(DeviceIdentity, u8, u8, u8, u64)> {
    let pointer = output.tree_root_pointer(tree);
    pointer
        .locations
        .iter()
        .map(|location| {
            let bytes = image
                .devices
                .get(&location.device)
                .expect("根的副本落在池里的盘上")
                .read(
                    singlefs_core::address::DeviceOffsetInBytes(location.slot.0 * SLOT_BYTES),
                    usize::try_from(NODE_BYTES).expect("16384"),
                );
            let parsed = parse_index_node(&bytes).expect("根节点是自证过的码 2 节点");
            (
                location.device,
                bytes[UNIT_CLASS_TAG_OFFSET],
                bytes[INDEX_NODE_LEVEL_OFFSET],
                parsed.level,
                parsed.tree.0,
            )
        })
        .collect()
}

/// 发完意图之后，`deleted_inodes` 根两份副本的层级字节都是 0，墓碑树根两份副本的层级字节都是 1（根直接管唯一那个容器）。
/// 判别力：`deleted_inodes` 根写成层级 1、墓碑树根写成层级 0（`crates/mutations.tsv` 里「C713」那两行）时这一条红。
#[test]
fn the_deleted_inodes_root_is_written_at_level_zero_and_the_tombstone_root_over_one_container_at_level_one(
) {
    let mut pool = pool_with_a_three_unit_file("c713-deletion-tree-root-levels");
    let after_the_intent = publish_the_intent(&mut pool);
    let image = pool.memory_pool();
    for (tree, expected_level) in [
        (
            TREE_IDENTIFIER_DELETED_INODES,
            LEVEL_OF_THE_DELETED_INODES_ROOT,
        ),
        (
            TREE_IDENTIFIER_TOMBSTONE,
            LEVEL_OF_A_TOMBSTONE_ROOT_OVER_ONE_LAYER_OF_CONTAINERS,
        ),
    ] {
        let headers = root_headers_on_disk(&image, &after_the_intent, tree);
        assert_eq!(headers.len(), 2, "树 {tree} 的根两份副本");
        for (device, class_tag, level_byte, parsed_level, parsed_tree) in headers {
            assert_eq!(
                (class_tag, parsed_tree),
                (UNIT_CLASS_INDEX_NODE, tree),
                "盘 {device:?} 上那一份是树 {tree} 的码 2 节点"
            );
            assert_eq!(
                level_byte, parsed_level,
                "取样点：盘 {device:?} 上偏移 {INDEX_NODE_LEVEL_OFFSET} 那一字节就是解析器读的层级"
            );
            assert_eq!(
                level_byte, expected_level,
                "盘 {device:?} 上树 {tree} 的根层级字节"
            );
        }
    }
}
