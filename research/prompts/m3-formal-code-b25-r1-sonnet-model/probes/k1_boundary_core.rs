//! 正推腿探针：实现的叶容器规则（`inode_tree::write_records_into_leaf_containers`，纯函数）在 233 的倍数两侧与 135 片的边界上。
use singlefs_core::address::{CheckpointTxg, InodeNumber, TreeIdentifier};
use singlefs_core::inode_tree::{
    leaf_containers_one_root_node_holds, remove_record_from_leaf_containers,
    write_records_into_leaf_containers, InodeLeafContainerIndexInTree,
};
use singlefs_core::records::InodeRecord;

fn record(inode: u64) -> InodeRecord {
    InodeRecord {
        inode,
        object_birth: CheckpointTxg(3),
        nlink: singlefs_core::records::NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES,
        size: 0,
        change_count: 3,
        write_time_seconds: 1,
    }
}

#[test]
fn core_one_at_a_time_lands_in_floor_k_over_233_and_refuses_at_136_containers() {
    println!("PROBE core capacity = {}", leaf_containers_one_root_node_holds());
    let tree = TreeIdentifier(singlefs_format::TREE_IDENTIFIER_INODE);
    let mut containers = Vec::new();
    let mut checked = 0u64;
    for k in 0u64..(233 * 135 + 1) {
        let result = write_records_into_leaf_containers(&containers, &[record(2 + k)], CheckpointTxg(4 + k), tree);
        match result {
            Ok(after) => {
                let expected = InodeLeafContainerIndexInTree::of_position(usize::try_from(k / 233).unwrap());
                assert_eq!(after.rewritten, vec![expected], "k={k}");
                containers = after.containers;
                checked += 1;
            }
            Err(refusal) => {
                println!("PROBE core refusal at k={k} (records_before) -> {refusal:?}");
                break;
            }
        }
    }
    println!("PROBE core checked={checked} containers={}", containers.len());
    // 删除之后条数变少：删最后一条，最右片不空时只重写它；删到最右片空时那一片不落盘。
    let last = containers.last().unwrap().records.last().unwrap().inode;
    let removed = remove_record_from_leaf_containers(&containers, InodeNumber(last)).unwrap();
    println!("PROBE core remove last: rewritten={:?} dropped={}", removed.rewritten, removed.containers_dropped);
}
