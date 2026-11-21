use super::*;

#[test]
fn every_transaction_unit_names_its_class_tree_and_placement_rule() {
    assert_eq!(
        TransactionUnit::IN_BUMP_ORDER
            .iter()
            .map(|unit| unit.unit_class())
            .collect::<Vec<_>>(),
        [
            UNIT_CLASS_DATA,
            UNIT_CLASS_INDEX_NODE,
            UNIT_CLASS_PACKED,
            UNIT_CLASS_INDEX_NODE,
            UNIT_CLASS_INDEX_NODE,
            UNIT_CLASS_INDEX_NODE,
            UNIT_CLASS_INDEX_NODE,
            UNIT_CLASS_INDEX_NODE,
        ],
        "数据单元码 1、inode 叶容器码 3，extent 根、inode 根、分配记录树、记账树、映射树、树表码 2"
    );
    assert_eq!(
        TransactionUnit::InstanceTable.unit_class(),
        UNIT_CLASS_PACKED,
        "实例表单元是码 3 打包容器"
    );
    assert_eq!(
        TransactionUnit::InstanceTablePageAfterTheFirst(InstanceTablePageIndex(1)).unit_class(),
        UNIT_CLASS_PACKED,
        "实例表链第一片之后的每一片同样是码 3"
    );
    assert_eq!(
        TransactionUnit::AllocationTreeNodeBelowTheRoot(AllocationRecordTreeNodePosition {
            level: 0,
            device: DeviceIdentity(1),
            index_in_device: 61,
        })
        .tag(),
        "t5@0.1.61",
        "分配记录树根之下的节点按位置写步号"
    );
    assert_eq!(
        TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::LEFTMOST).placement(),
        PlacementRule::CommitGenerated(UnitFootprint::TwoSlotsAligned),
        "码 3 容器按数据单元那一档"
    );
    assert_eq!(
        TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::of_position(1)).tag(),
        "t3+1",
        "第二片叶容器的步号：字节表七只登记了一片那一档的 t3"
    );
    assert_eq!(
        TransactionUnit::Data(DataUnitIndexInFile::FIRST).placement(),
        PlacementRule::UserData
    );
    let (trees_issued_from_the_make_filesystem_watermark, _) =
        FileVersionTreeIdentifiers::issued_from_watermark(TREE_IDENTIFIER_WATERMARK_AT_MKFS)
            .expect("mkfs 水位 65536 加十二个号装得下");
    assert_eq!(
        TransactionUnit::TreeTable.tree(&trees_issued_from_the_make_filesystem_watermark),
        TreeIdentifier(0),
        "树表单元不属于任何一棵树"
    );
    assert_eq!(
        TransactionUnit::IN_BUMP_ORDER
            .iter()
            .map(|unit| unit.tag())
            .collect::<Vec<_>>(),
        ["t1", "t2", "t3", "t4", "t5", "t6", "t7", "t8"]
    );
}
