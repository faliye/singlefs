use super::*;

/// 一次发布的重写角色：`data_units` 个数据单元、extent 根、`leaf_containers` 片 inode 叶容器、inode 根与四个固定点角色的根（分配记录树、记账树、映射树、树表；根之下的节点不在这里），按 bump 次序。
fn rewritten_roles(data_units: u64, leaf_containers: usize) -> Vec<TransactionUnit> {
    (0..data_units)
        .map(|index| TransactionUnit::Data(DataUnitIndexInFile(index)))
        .chain([TransactionUnit::ExtentRoot])
        .chain((0..leaf_containers).map(|position| {
            TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::of_position(
                position,
            ))
        }))
        .chain([
            TransactionUnit::InodeRoot,
            TransactionUnit::AllocationTree,
            TransactionUnit::AccountingTree,
            TransactionUnit::MappingTree,
            TransactionUnit::TreeTable,
        ])
        .collect()
}

/// 末条再跨记录（D23（journal 的角色与格式） 已定项 17）：最后一个事务要点名的项多于一条记录装得下的 64 项时，按 bump 次序
/// 装满一条再开下一条，这几条都属于最后一个事务；前 N − 1 个数据单元各一条、各是自己那个事务。
/// 两个数据单元 + extent 根 + 130 片叶容器 + 5 个角色 = 138 个：第一条只点名数据单元 0（事务 0），
/// 最后一个事务要点名 137 项 ⇒ 64 + 64 + 9 三条（事务都是 1）。不写数据单元的 65 个 ⇒ 64 + 1 两条（事务都是 0）。
/// 装得下的照旧：三个数据单元 ⇒ 三条三个事务，末条点名数据单元 2 与 6 个共享角色。
#[test]
fn the_last_transaction_spills_over_as_many_records_as_its_named_units_need_and_they_all_share_its_transaction(
) {
    let spilling_with_data = rewritten_roles(2, 130);
    let records_with_data = roles_named_by_each_record_of_the_publish(
        &spilling_with_data,
        JournalRecordNamedEntryCapacity::FromTheRecordFormat,
    );
    assert_eq!(
        records_with_data.iter().map(Vec::len).collect::<Vec<_>>(),
        vec![1, 64, 64, 9],
        "数据单元 0 一条；最后一个事务 137 项装满两条再开第三条"
    );
    assert_eq!(
        records_with_data[0],
        vec![TransactionUnit::Data(DataUnitIndexInFile(0))]
    );
    assert_eq!(
        records_with_data.concat(),
        spilling_with_data,
        "几条合起来按 bump 次序恰好点名每个重写角色一次"
    );
    assert_eq!(
        transaction_offset_of_each_record_of_the_publish(
            &spilling_with_data,
            JournalRecordNamedEntryCapacity::FromTheRecordFormat
        ),
        vec![0, 1, 1, 1],
        "跨出去的两条属于最后一个事务"
    );

    let spilling_without_data = rewritten_roles(0, 59);
    assert_eq!(spilling_without_data.len(), 65);
    let records_without_data = roles_named_by_each_record_of_the_publish(
        &spilling_without_data,
        JournalRecordNamedEntryCapacity::FromTheRecordFormat,
    );
    assert_eq!(
        records_without_data
            .iter()
            .map(Vec::len)
            .collect::<Vec<_>>(),
        vec![64, 1]
    );
    assert_eq!(
        transaction_offset_of_each_record_of_the_publish(
            &spilling_without_data,
            JournalRecordNamedEntryCapacity::FromTheRecordFormat
        ),
        vec![0, 0],
        "一个事务跨两条"
    );

    let fitting = rewritten_roles(3, 1);
    assert_eq!(
        roles_named_by_each_record_of_the_publish(
            &fitting,
            JournalRecordNamedEntryCapacity::FromTheRecordFormat
        )
        .iter()
        .map(Vec::len)
        .collect::<Vec<_>>(),
        vec![1, 1, 8]
    );
    assert_eq!(
        transaction_offset_of_each_record_of_the_publish(
            &fitting,
            JournalRecordNamedEntryCapacity::FromTheRecordFormat
        ),
        vec![0, 1, 2]
    );

    assert_eq!(
        roles_named_by_each_record_of_the_publish(
            &[],
            JournalRecordNamedEntryCapacity::FromTheRecordFormat
        ),
        vec![Vec::<TransactionUnit>::new()],
        "一个角色都不重写的发布照样一条记录"
    );
    assert_eq!(
        transaction_offset_of_each_record_of_the_publish(
            &[],
            JournalRecordNamedEntryCapacity::FromTheRecordFormat
        ),
        vec![0]
    );
}
