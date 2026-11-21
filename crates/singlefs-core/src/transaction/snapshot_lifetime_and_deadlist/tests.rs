use super::*;

/// D5（快照 / 空间记账机制） 已定项 14 的左边界：birth == S.txg 时快照引用它（进 deadlist），birth == S.txg + 1 时不引用（释放）；
/// 映射 key 不是 27 字节（birth 取不到）默认进 deadlist（「判据的输入取不到时默认进 deadlist」）。没打过快照（p = 0）时一律释放。
#[test]
fn unit_born_at_the_snapshot_txg_is_kept_and_one_born_after_it_is_released() {
    let key_born_at = |birth: u64| {
        crate::records::mapping_key_for_node(
            UNIT_CLASS_INDEX_NODE,
            NodePointer {
                head: PointerHead {
                    birth_tree: TreeIdentifier(65_536),
                    birth_txg: CheckpointTxg(birth),
                },
                locations: NodePointer::empty_root().locations,
                instance: InstanceGeneration(1),
                birth_sequence: BirthSequence(0),
            },
        )
    };
    assert_eq!(
        snapshot_lifetime_verdict(&key_born_at(9), CheckpointTxg(9)),
        SnapshotLifetimeVerdict::KeptOnTheDeadlist,
        "birth == S.txg：快照引用它"
    );
    assert_eq!(
        snapshot_lifetime_verdict(&key_born_at(10), CheckpointTxg(9)),
        SnapshotLifetimeVerdict::ReleasedIntoTheDeferQueue,
        "birth == S.txg + 1：比快照新，释放"
    );
    assert_eq!(
        snapshot_lifetime_verdict(&key_born_at(3), CheckpointTxg(0)),
        SnapshotLifetimeVerdict::ReleasedIntoTheDeferQueue,
        "没打过快照：一律释放"
    );
    assert_eq!(
        snapshot_lifetime_verdict(&key_born_at(10)[..26], CheckpointTxg(9)),
        SnapshotLifetimeVerdict::KeptOnTheDeadlist,
        "birth 取不到：默认进 deadlist"
    );
}

/// 判决 S3：只有六类用户可见子树的角色问生死判定，池级元数据的 14 类（连 deadlist 树自己）一律直接进 defer。
#[test]
fn only_the_six_user_visible_roles_ask_the_snapshot_lifetime_judgement() {
    let position = CodeTwoTreeNodePosition {
        level: 0,
        index_in_level: 0,
    };
    for asking in [
        TransactionUnit::Data(DataUnitIndexInFile::FIRST),
        TransactionUnit::ExtentRoot,
        TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::LEFTMOST),
        TransactionUnit::InodeRoot,
    ] {
        assert!(
            asks_the_snapshot_lifetime_judgement(asking),
            "{asking:?}：用户可见，问判定"
        );
    }
    for not_asking in [
        TransactionUnit::AllocationTree,
        TransactionUnit::AccountingTree,
        TransactionUnit::AccountingTreeNodeBelowTheRoot(position),
        TransactionUnit::DeadlistTree,
        TransactionUnit::DeadlistTreeNodeBelowTheRoot(position),
        TransactionUnit::DeletedInodesRoot,
        TransactionUnit::TombstoneRoot,
        TransactionUnit::LineageRoot,
        TransactionUnit::RetirementRoot,
        TransactionUnit::MappingTree,
        TransactionUnit::MappingTreeNodeBelowTheRoot(position),
        TransactionUnit::TreeTable,
        TransactionUnit::InstanceTable,
    ] {
        assert!(
            !asks_the_snapshot_lifetime_judgement(not_asking),
            "{not_asking:?}：池级元数据，直接进 defer"
        );
    }
}
