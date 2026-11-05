
/// m4-close-code-r2 攻方腿的探针（只在攻方草稿副本里，不进仓）：`TransactionOutput::inode_record_found_by_its_leaf` 按叶序折半找，
/// 前提是叶容器之间不交叠；写者 `inode_tree::write_records_into_leaf_containers` 与删者 `remove_record_from_leaf_containers` 逐片找。
/// 恢复（`recovery::inode_leaf_container_judged_against_its_entry`）只判记录号不小于分隔 key 与容器号，不判左片最大号小于右片：两片交叠的镜像挂得上。
/// 交叠时两种找法交回的不同，维护出来的 n_live / n_记录 与逐片现数分开、最后一批删那条记录时 `expect` 落空。对照组是两片按 key 严格升序的同一形。
#[cfg(test)]
mod inode_record_lookup_by_leaf_order_over_overlapping_leaves_probe {
    use super::pool_in_memory_after_the_first_file::pool_after_the_first_file;
    use crate::address::{CheckpointTxg, InodeNumber};
    use crate::admission::InodeRecordCounts;
    use crate::inode_tree::{
        remove_record_from_leaf_containers, write_records_into_leaf_containers, InodeLeafContainer,
    };
    use crate::records::InodeRecord;
    use crate::transaction::{InodeLeafContainerVersion, TransactionOutput};

    /// 第一个文件之后那一版，把 inode 树换成两片：左片装 `left_offsets`、右片装 `right_offsets`（都是相对第一个文件 inode 号的偏移），
    /// 右片的容器号与分隔 key 取它装的最小号。两片的记录都照第一个文件那条记录改号；两个数照逐片现数重算（与冷恢复同一种起始值）。
    fn version_with_two_leaves(left_offsets: &[u64], right_offsets: &[u64]) -> (TransactionOutput, InodeRecord) {
        let pool = pool_after_the_first_file();
        let mut version = pool.current.clone();
        let leaf = version.inode_leaf_containers[0].clone();
        let first_record = leaf.contents.records[0];
        let record_numbered = |offset: u64| InodeRecord {
            inode: first_record.inode + offset,
            ..first_record
        };
        let right_smallest = first_record.inode + right_offsets[0];
        let left = InodeLeafContainerVersion {
            contents: InodeLeafContainer {
                identity: leaf.contents.identity,
                separator_key_in_the_parent_entry: leaf.contents.separator_key_in_the_parent_entry,
                records: left_offsets.iter().map(|offset| record_numbered(*offset)).collect(),
            },
            pointer: leaf.pointer,
        };
        let mut right_identity = leaf.contents.identity;
        right_identity.container = right_smallest;
        let right = InodeLeafContainerVersion {
            contents: InodeLeafContainer {
                identity: right_identity,
                separator_key_in_the_parent_entry: right_smallest,
                records: right_offsets.iter().map(|offset| record_numbered(*offset)).collect(),
            },
            pointer: leaf.pointer,
        };
        version.inode_leaf_containers = vec![left, right];
        version.inode_record_counts = InodeRecordCounts::counted_over_the_inode_tree(
            version.inode_leaf_containers.iter().map(|container| &container.contents),
        );
        (version, first_record)
    }

    fn lookups_agree(left_offsets: &[u64], right_offsets: &[u64], asked_offset: u64) {
        let (version, first_record) = version_with_two_leaves(left_offsets, right_offsets);
        let asked = InodeNumber(first_record.inode + asked_offset);
        let linear = version.inode_record_of(asked);
        let by_leaf_order = version.inode_record_found_by_its_leaf(asked);
        println!(
            "probe=lookup left={left_offsets:?} right={right_offsets:?} asked_offset={asked_offset} linear_found={} leaf_order_found={}",
            linear.is_some(),
            by_leaf_order.is_some()
        );
        assert_eq!(
            by_leaf_order, linear,
            "按叶序折半找与逐条找交回同一条（transaction.rs inode_record_found_by_its_leaf 的文档注释）"
        );
    }

    fn maintained_counts_equal_the_recount(left_offsets: &[u64], right_offsets: &[u64], rewritten_offset: u64) {
        let (version, first_record) = version_with_two_leaves(left_offsets, right_offsets);
        let rewritten = InodeRecord {
            inode: first_record.inode + rewritten_offset,
            size: first_record.size + 1,
            ..first_record
        };
        let containers_before = version.inode_leaf_container_contents();
        let after = write_records_into_leaf_containers(
            &containers_before,
            &[rewritten],
            CheckpointTxg(version.root.checkpoint_txg.0 + 1),
            version.tree_identifiers.inode,
        )
        .expect("改一条已在树里的记录，写者逐片找到它、原地换掉");
        let maintained = version
            .inode_record_counts
            .after_writing(&[rewritten], |inode| version.inode_record_found_by_its_leaf(inode));
        let recounted = InodeRecordCounts::counted_over_the_inode_tree(after.containers.iter());
        println!(
            "probe=after_writing left={left_offsets:?} right={right_offsets:?} rewritten_offset={rewritten_offset} before={:?} maintained={maintained:?} recounted={recounted:?}",
            version.inode_record_counts
        );
        assert_eq!(maintained, recounted, "维护出来的两个数等于写完之后逐片现数");
    }

    fn removed_record_is_found(left_offsets: &[u64], right_offsets: &[u64], removed_offset: u64) {
        let (version, first_record) = version_with_two_leaves(left_offsets, right_offsets);
        let removed = InodeNumber(first_record.inode + removed_offset);
        let containers_before = version.inode_leaf_container_contents();
        let removal = remove_record_from_leaf_containers(&containers_before, removed);
        let found = version.inode_record_found_by_its_leaf(removed);
        println!(
            "probe=final_batch left={left_offsets:?} right={right_offsets:?} removed_offset={removed_offset} remove_ok={} leaf_order_found={}",
            removal.is_ok(),
            found.is_some()
        );
        assert!(removal.is_ok(), "删者逐片找到这条记录");
        assert!(
            found.is_some(),
            "最后一批删的那条记录按叶序找得到（transaction.rs 最后一批那一支的 expect）"
        );
    }

    #[test]
    fn leaves_in_strict_key_order_agree_on_every_lookup_count_and_removal() {
        for asked in [0, 2, 4, 8] {
            lookups_agree(&[0, 2], &[4, 8], asked);
            maintained_counts_equal_the_recount(&[0, 2], &[4, 8], asked);
            removed_record_is_found(&[0, 2], &[4, 8], asked);
        }
    }

    #[test]
    fn overlapping_leaves_agree_on_the_lookup() {
        lookups_agree(&[0, 8], &[4], 4);
    }

    #[test]
    fn overlapping_leaves_keep_the_maintained_counts_equal_to_the_recount() {
        maintained_counts_equal_the_recount(&[0, 8], &[4], 4);
    }

    #[test]
    fn overlapping_leaves_find_the_record_the_final_batch_removes() {
        removed_record_is_found(&[0, 8], &[4], 4);
    }
}
