use super::*;

// ───── 已发布谓词的两份（C410（checker 两个方向共享什么没有检查））─────

/// 扫描式方向另写的那一份已发布谓词与遍历式那一份在同一组输入上判得一样：挂载根（实例 5、txg 20），实例表行照扫描重建用例的形状
/// （较早的几个实例各一行 (i, T, W)，实例 2 没有行，实例 3 有两行时取先出现的那一行），码 1 与码 3 两类、实例 1..=6、诞生代号与事务号
/// 各取行里 T、W 与挂载根 txg 的两侧，逐个比。
#[test]
fn the_published_predicate_of_the_scan_direction_agrees_with_the_one_of_the_traversal_direction() {
    let instance_table_rows = vec![
        InstanceTableRow {
            instance: 1,
            published_checkpoint_txg: 7,
            applied_transaction_high_water_mark: 40,
        },
        InstanceTableRow {
            instance: 3,
            published_checkpoint_txg: 12,
            applied_transaction_high_water_mark: 90,
        },
        InstanceTableRow {
            instance: 3,
            published_checkpoint_txg: 2,
            applied_transaction_high_water_mark: 1,
        },
        InstanceTableRow {
            instance: 4,
            published_checkpoint_txg: 0,
            applied_transaction_high_water_mark: 0,
        },
    ];
    let traversal_direction = PublishedPredicate {
        mount_root_instance: 5,
        mount_root_checkpoint_txg: 20,
        instance_table_rows: &instance_table_rows,
    };
    let scan_direction = PublishedPredicateOfTheScanDirection {
        mount_root_instance: 5,
        mount_root_checkpoint_txg: 20,
        instance_table_rows: &instance_table_rows,
    };
    let mut compared = 0u32;
    let mut admitted = 0u32;
    for unit_class in [ContentUnitClass::Data, ContentUnitClass::PackedRecord] {
        for instance in 1..=6u32 {
            for birth_txg in [0u64, 1, 2, 3, 6, 7, 8, 11, 12, 13, 19, 20, 21] {
                for transaction in [0u64, 1, 2, 39, 40, 41, 89, 90, 91] {
                    let write_order = match unit_class {
                        ContentUnitClass::Data => WriteOrderUnderThePublishedPredicate::DataUnit {
                            birth_txg,
                            instance,
                            transaction,
                        },
                        ContentUnitClass::PackedRecord => {
                            WriteOrderUnderThePublishedPredicate::IndexOrPackedRecordUnit {
                                birth_txg,
                                instance,
                            }
                        }
                    };
                    let by_the_traversal_direction = traversal_direction.holds_for(write_order);
                    assert_eq!(
                            scan_direction.admits_the_header(
                                unit_class,
                                birth_txg,
                                instance,
                                transaction
                            ),
                            by_the_traversal_direction,
                            "{} 单元（诞生代号 {birth_txg}、实例 {instance}、事务号 {transaction}）两份谓词判得不一样",
                            unit_class.name()
                        );
                    compared += 1;
                    admitted += u32::from(by_the_traversal_direction);
                }
            }
        }
    }
    assert_eq!(compared, 2 * 6 * 13 * 9, "每一组输入都比到了");
    assert!(
        admitted > 0 && admitted < compared,
        "两种结论都出现过（{admitted} / {compared} 判成已发布），对拍不是在一边上空比"
    );
}
