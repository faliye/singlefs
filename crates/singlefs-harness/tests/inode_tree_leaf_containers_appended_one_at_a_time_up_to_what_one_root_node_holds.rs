//! inode 树叶容器的切法（实现一侧，`singlefs_core::inode_tree::write_records_into_leaf_containers`，纯函数）在 135 片的边界上：
//! 一次追加一条，第 k 条（从 0 数）重写的恰是左起第 ⌊k ÷ 233⌋ 片，追加到第 233 × 135 条（k = 31 454）为止都收；
//! 再追加一条要第 136 片，一个码 2 根装不下（D8（核心索引结构） 已定项 6：扇出 135；树长高那一步条款没写），实现在任何落盘动作之前拒。
//! 理想模型的落点式子（`crates/singlefs-harness/src/model.rs` 的 `inode_leaf_container_position_of_a_new_record`）与它是同一条规则、
//! 同一个 135（第 25 批代码轮正推 K1 第一行与第四行，判决 `research/prompts/m3-formal-code-b25-r1-main-verification.md`）。

use singlefs_core::address::{CheckpointTxg, TreeIdentifier};
use singlefs_core::inode_tree::{
    leaf_containers_one_root_node_holds, write_records_into_leaf_containers,
    InodeLeafContainerIndexInTree, InodeTreeWriteRefusal,
};
use singlefs_core::records::{InodeRecord, NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES};
use singlefs_format::{INODE_LEAF_RECORDS, TREE_IDENTIFIER_INODE};

/// 一个码 2 根装得下几片叶容器（D8（核心索引结构） 已定项 6：内部条目 120 字节、扇出 135）。
const LEAF_CONTAINERS_ONE_ROOT_NODE_HOLDS: usize = 135;

/// 第一条追加的记录用的 inode 号：inode 1 是第一个文件，删完之后建的从 2 起。
const FIRST_APPENDED_INODE_NUMBER: u64 = 2;

/// 第一次追加那次发布的 checkpoint_txg（之后每次加 1）。
const FIRST_APPEND_CHECKPOINT_TXG: u64 = 4;

fn size_zero_inode_record_appended_as(records_before: u64) -> InodeRecord {
    InodeRecord {
        inode: FIRST_APPENDED_INODE_NUMBER + records_before,
        object_birth: CheckpointTxg(FIRST_APPEND_CHECKPOINT_TXG + records_before),
        nlink: NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES,
        size: 0,
        change_count: 1,
        write_time_seconds: 1,
    }
}

/// 一次追加一条记录，追加到 135 片装满（233 × 135 条），每次只重写新记录落的第 ⌊k ÷ 233⌋ 片；
/// 再追加一条，交回 `MoreLeafContainersThanOneRootNodeHolds { containers: 136, capacity: 135 }`。
/// 判别力：容器数的上限放宽一片（第 31 456 条照收），或分裂点提前（一片不装满 233 条就分裂，第 k 条落的片不是 ⌊k ÷ 233⌋）。
#[test]
fn appending_inode_records_one_at_a_time_rewrites_container_floor_k_over_233_and_refuses_the_136th_container(
) {
    assert_eq!(
        leaf_containers_one_root_node_holds(),
        LEAF_CONTAINERS_ONE_ROOT_NODE_HOLDS,
        "一个码 2 根的扇出是 135（D8 已定项 6）"
    );
    let inode_tree = TreeIdentifier(TREE_IDENTIFIER_INODE);
    let records_one_root_node_holds = INODE_LEAF_RECORDS
        * u64::try_from(LEAF_CONTAINERS_ONE_ROOT_NODE_HOLDS).expect("135 装得进 u64");
    let mut containers = Vec::new();
    for records_before in 0..records_one_root_node_holds {
        let after = write_records_into_leaf_containers(
            &containers,
            &[size_zero_inode_record_appended_as(records_before)],
            CheckpointTxg(FIRST_APPEND_CHECKPOINT_TXG + records_before),
            inode_tree,
        )
        .unwrap_or_else(|refusal| panic!("第 {records_before} 条（从 0 数）还装得下：{refusal:?}"));
        let expected_position = usize::try_from(records_before / INODE_LEAF_RECORDS)
            .expect("不到 135 的片号装得进 usize");
        assert_eq!(
            after.rewritten,
            vec![InodeLeafContainerIndexInTree::of_position(
                expected_position
            )],
            "第 {records_before} 条（从 0 数）只重写左起第 ⌊k ÷ 233⌋ 片"
        );
        containers = after.containers;
    }
    assert_eq!(
        containers.len(),
        LEAF_CONTAINERS_ONE_ROOT_NODE_HOLDS,
        "233 × 135 条装满 135 片"
    );
    let refusal = write_records_into_leaf_containers(
        &containers,
        &[size_zero_inode_record_appended_as(
            records_one_root_node_holds,
        )],
        CheckpointTxg(FIRST_APPEND_CHECKPOINT_TXG + records_one_root_node_holds),
        inode_tree,
    )
    .expect_err("第 233 × 135 + 1 条要第 136 片，一个码 2 根装不下");
    assert_eq!(
        refusal,
        InodeTreeWriteRefusal::MoreLeafContainersThanOneRootNodeHolds {
            containers: LEAF_CONTAINERS_ONE_ROOT_NODE_HOLDS + 1,
            capacity: LEAF_CONTAINERS_ONE_ROOT_NODE_HOLDS,
        },
        "拒的是根装不下的那一格"
    );
}
