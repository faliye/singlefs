use super::*;

fn failing_anchors(lines: &[AnchorLine]) -> Vec<String> {
    lines
        .iter()
        .filter(|line| !line.matches())
        .map(|line| format!("{}: observed [{}] expected [{}]", line.identifier, line.observed, line.expected))
        .collect()
}

fn partition_cell(
    arm: Arm,
    variant: ArmVariant,
    stage: ParallelStage,
    shape_kind: ShapeKind,
    initial_has_file: bool,
    transaction_count: usize,
    device_count: usize,
) -> Aggregate {
    enumerate_specification(specification_of(arm, variant, stage, shape_kind, initial_has_file, transaction_count, device_count))
        .aggregate
}

// ── 登记第七节甲 / 乙 / 丙：钉绝对值的断言（一份产物里的 `name=anchor` 行就是这几组） ─────────────

#[test]
fn serial_baseline_and_stage_definitions_match_the_clause_values() {
    let failing = failing_anchors(&anchors_from_the_clauses());
    assert!(failing.is_empty(), "甲组不符：{failing:#?}");
}

#[test]
fn positive_controls_and_partition_cells_match_the_independently_calculated_values() {
    let failing = failing_anchors(&anchors_from_independent_calculations());
    assert!(failing.is_empty(), "乙组不符：{failing:#?}");
}

#[test]
fn schedule_counts_serial_baseline_and_memoization_match_the_structural_values() {
    let failing = failing_anchors(&anchors_from_structural_counts());
    assert!(failing.is_empty(), "丙组不符：{failing:#?}");
}

#[test]
fn local_constants_agree_with_the_crates_values_they_copy() {
    assert_eq!(JOURNAL_RING_DEFAULT_BYTES, u64::from(DEFAULT_JOURNAL_RING_SLOTS) * JOURNAL_RECORD_BYTES);
    assert_eq!(DEFAULT_JOURNAL_RING_SLOTS, 196_608);
    assert_eq!(ROOT_RING_REGIONS, 3);
    assert_eq!(TREE_IDENTIFIER_WATERMARK_AT_MKFS, 65536);
    assert_eq!(TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH, 65548);
    assert_eq!(TREE_IDENTIFIERS_ISSUED_AT_FIRST_FILE_VERSION, 12);
    assert_eq!(TREE_IDENTIFIERS_ISSUED_AT_FIRST_FILE_VERSION + TREE_IDENTIFIER_WATERMARK_AT_MKFS, 65548);
    assert_eq!(INITIAL_INODE_WATERMARK, 1 + 1);
    assert_eq!(USER_DATA_SPAN_SLOTS, 2);
}

// ── 单个函数 ─────────────────────────────────────────────────────────────

#[test]
fn duplicate_count_is_holders_minus_one_for_every_item() {
    assert_eq!(duplicates_among(&mut [5, 5, 5, 7, 7, 9]), 3);
    assert_eq!(duplicates_among(&mut [1, 2, 3]), 0);
    assert_eq!(duplicates_among(&mut [4, 4]), 1);
}

#[test]
fn lowest_free_pair_takes_the_smallest_untaken_pair_inside_the_range() {
    assert_eq!(lowest_free_pair(0b0000, 0, 16), Some(0));
    assert_eq!(lowest_free_pair(0b0111, 0, 16), Some(3));
    assert_eq!(lowest_free_pair(0b0101, 0, 16), Some(1));
    assert_eq!(lowest_free_pair(0b0111, 0, 3), None);
    assert_eq!(lowest_free_pair(0b0001_1111, 5, 10), Some(5));
    assert_eq!(pair_range_mask(0, 16), u16::MAX);
    assert_eq!(pair_range_mask(5, 10), 0b0000_0011_1110_0000);
}

#[test]
fn schedule_count_formula_is_the_multinomial() {
    assert_eq!(schedule_count_without_retries(2, 1), 2);
    assert_eq!(schedule_count_without_retries(3, 1), 6);
    assert_eq!(schedule_count_without_retries(2, 2), 6);
    assert_eq!(schedule_count_without_retries(3, 2), 90);
    assert_eq!(schedule_count_without_retries(2, 3), 20);
    assert_eq!(schedule_count_without_retries(3, 3), 1680);
    assert_eq!(schedule_count_without_retries(3, 6), 17_153_136);
}

#[test]
fn root_ring_and_journal_slot_formulas_match_the_clause_examples() {
    assert_eq!(slot_text(root_ring_slot_key(6, 4)), "(0,2)");
    assert_eq!(slot_text(root_ring_slot_key(7, 4)), "(1,2)");
    assert_eq!(slot_text(root_ring_slot_key(9, 4)), "(0,3)");
    assert_eq!(slot_text(root_ring_slot_key(12, 4)), "(0,0)");
    assert_eq!(journal_ring_slot(6, DEFAULT_JOURNAL_RING_SLOTS), 5);
    assert_eq!(journal_ring_slot(1, 2), 0);
    assert_eq!(journal_ring_slot(3, 2), 0);
}

// ── 各臂在默认几何上的判定（不是阳性对照，是被测臂自己） ─────────────────────────

#[test]
fn whole_lock_hands_out_no_duplicate_in_any_stage_shape_or_initial_state() {
    for stage in ALL_STAGES {
        for shape_kind in [ShapeKind::SameShape, ShapeKind::Disjoint] {
            for initial_has_file in [true, false] {
                for transaction_count in [2usize, 3] {
                    let aggregate = partition_cell(
                        Arm::WholeLock,
                        ArmVariant::Normal,
                        stage,
                        shape_kind,
                        initial_has_file,
                        transaction_count,
                        DEFAULT_DEVICE_COUNT,
                    );
                    assert_eq!(
                        aggregate.duplicate_total,
                        [0; RESOURCE_COUNT],
                        "{} {shape_kind:?} {initial_has_file} n={transaction_count}",
                        stage.label()
                    );
                    let factorial_of_count: u128 = (1..=transaction_count as u128).product();
                    assert_eq!(aggregate.schedules, factorial_of_count);
                    assert_eq!(aggregate.retries_total, 0);
                }
            }
        }
    }
}

#[test]
fn region_partition_and_device_partition_hand_out_no_duplicate_in_default_geometry() {
    for arm in [Arm::RegionPartition, Arm::DevicePartition] {
        for stage in ALL_STAGES {
            for shape_kind in [ShapeKind::SameShape, ShapeKind::Disjoint] {
                for initial_has_file in [true, false] {
                    for transaction_count in [2usize, 3] {
                        for device_count in [SAMPLE_DEVICE_COUNT, DEFAULT_DEVICE_COUNT] {
                            let aggregate = partition_cell(
                                arm,
                                ArmVariant::Normal,
                                stage,
                                shape_kind,
                                initial_has_file,
                                transaction_count,
                                device_count,
                            );
                            assert_eq!(
                                aggregate.duplicate_total,
                                [0; RESOURCE_COUNT],
                                "{arm:?} {} {shape_kind:?} {initial_has_file} n={transaction_count} D={device_count}",
                                stage.label()
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn region_partition_with_one_region_per_transaction_never_falls_back_or_deadlocks() {
    for stage in ALL_STAGES {
        let aggregate = partition_cell(
            Arm::RegionPartition,
            ArmVariant::Normal,
            stage,
            ShapeKind::SameShape,
            false,
            3,
            DEFAULT_DEVICE_COUNT,
        );
        assert_eq!(aggregate.fallbacks_total, 0);
        assert_eq!(aggregate.deadlocks, 0);
        assert_eq!(aggregate.retries_total, 0);
        assert_eq!(aggregate.busy_regions_peak, 3, "三个事务各占一个区，峰值 3");
        assert_eq!(aggregate.busy_regions_final_maximum, 0, "全部提交之后区都释放了");
    }
}

#[test]
fn literal_lock_leaves_transaction_numbers_and_tree_identifiers_outside_the_lock() {
    // 锁内四样（U、G、I、journal 计数器、txg）没有重复；锁外两样（X、T）有（失败 7 的前件）。
    let aggregate = partition_cell(
        Arm::LiteralLock,
        ArmVariant::Normal,
        ParallelStage::ManyPublishesInFlight,
        ShapeKind::SameShape,
        false,
        2,
        DEFAULT_DEVICE_COUNT,
    );
    for resource in [
        Resource::UserDataSlot,
        Resource::CommitGeneratedSlot,
        Resource::InodeNumber,
        Resource::JsnCounter,
        Resource::JournalRingSlot,
        Resource::CheckpointTxg,
        Resource::RootRingSlot,
    ] {
        assert_eq!(aggregate.duplicate_total[resource.index()], 0, "{}", resource.label());
    }
    assert_eq!(aggregate.duplicate_total[Resource::TransactionNumber.index()], 18);
    // 取第 2 次重跑登记第七节 7.2 补 1「无文件 n = 2 的 T 合计 216」（独立 Python `run_literal(2, False, 65536, 12)` 算出）。
    assert_eq!(aggregate.duplicate_total[Resource::TreeIdentifier.index()], 216);
}

#[test]
fn read_set_optimistic_checks_the_user_pair_and_takes_the_transaction_number_at_the_commit_point() {
    // Q1乙、不相交：事务 0 只建 inode、其余只写用户数据单元。
    let three = partition_cell(
        Arm::OptimisticReadSet,
        ArmVariant::Normal,
        ParallelStage::OnePublishCarriesManyTransactions,
        ShapeKind::Disjoint,
        true,
        3,
        DEFAULT_DEVICE_COUNT,
    );
    assert_eq!(three.duplicate_total, [0; RESOURCE_COUNT], "校验里有「用户数据对仍空」，事务 1 与 2 争同一个对时后提交那个重算");
    assert!(three.retries_total > 0);
    let two = partition_cell(
        Arm::OptimisticReadSet,
        ArmVariant::Normal,
        ParallelStage::OnePublishCarriesManyTransactions,
        ShapeKind::Disjoint,
        true,
        2,
        DEFAULT_DEVICE_COUNT,
    );
    assert_eq!(two.duplicate_total[Resource::TransactionNumber.index()], 0, "事务号在提交点按当时的共享值发");
    assert_eq!(two.retries_total, 0, "事务 0 的读集只有 inode 起点、事务 1 只有一个用户数据对，互不相交");
}

#[test]
fn read_set_retries_equal_whole_version_retries_when_base_version_is_in_the_read_set() {
    // 失败 5 的前件：Q1甲 与 Q1丙 下 BASE 版本号在读集里，任何插进来的提交都让 O读 重算，与 O整 逐格相等。
    for stage in [ParallelStage::PrepareParallelCommitSerial, ParallelStage::ManyPublishesInFlight] {
        for shape_kind in [ShapeKind::SameShape, ShapeKind::Disjoint] {
            for transaction_count in [2usize, 3] {
                let whole = partition_cell(
                    Arm::OptimisticWholeVersion,
                    ArmVariant::Normal,
                    stage,
                    shape_kind,
                    true,
                    transaction_count,
                    DEFAULT_DEVICE_COUNT,
                );
                let read_set = partition_cell(
                    Arm::OptimisticReadSet,
                    ArmVariant::Normal,
                    stage,
                    shape_kind,
                    true,
                    transaction_count,
                    DEFAULT_DEVICE_COUNT,
                );
                assert_eq!(whole.retries_total, read_set.retries_total);
                assert_eq!(whole.schedules, read_set.schedules);
            }
        }
    }
}

// ── 分区臂的另一端：区不够、预留不够（第二段的取样点，这里只验模型的这几条路径） ─────────────

#[test]
fn one_region_for_two_transactions_makes_the_second_claim_wait_for_the_first_commit() {
    let mut specification = specification_of(
        Arm::RegionPartition,
        ArmVariant::Normal,
        ParallelStage::PrepareParallelCommitSerial,
        ShapeKind::SameShape,
        true,
        2,
        DEFAULT_DEVICE_COUNT,
    );
    specification.region_count = RegionCountChoice::Fixed(1);
    let model = Model::new(specification);
    let enumeration = enumerate_schedules(&model, false);
    assert_eq!(enumeration.aggregate.schedules, 2, "先认领到区的那个事务先走完，只剩两种次序");
    assert_eq!(enumeration.aggregate.deadlocks, 0);
    assert_eq!(enumeration.aggregate.duplicate_total, [0; RESOURCE_COUNT]);
    let rounds = simulate_rounds(&model);
    assert_eq!(rounds.rounds, Some(2));
    assert_eq!(rounds.commits_per_round, vec![1, 1]);
}

#[test]
fn inode_reservation_smaller_than_the_need_takes_a_fallback_interval_without_duplicates() {
    let mut specification = specification_of(
        Arm::RegionPartition,
        ArmVariant::Normal,
        ParallelStage::OnePublishCarriesManyTransactions,
        ShapeKind::MultiInode,
        true,
        2,
        DEFAULT_DEVICE_COUNT,
    );
    specification.inode_reservation = InodeReservation::Fixed(1);
    let model = Model::new(specification);
    let aggregate = enumerate_schedules(&model, false).aggregate;
    assert_eq!(aggregate.fallbacks_total, aggregate.schedules * 2, "每个事务都要补一段区间");
    assert_eq!(aggregate.duplicate_total, [0; RESOURCE_COUNT]);
    assert_eq!(aggregate.deadlocks, 0);
    let state = run_serially(&model);
    assert_eq!(state.shared.inode_watermark, INITIAL_INODE_WATERMARK + 4, "两个事务各要 2 个号，共发 4 个");
}

#[test]
fn inode_reservation_larger_than_the_need_leaves_numbers_unused() {
    let mut specification = specification_of(
        Arm::RegionPartition,
        ArmVariant::Normal,
        ParallelStage::OnePublishCarriesManyTransactions,
        ShapeKind::SameShape,
        true,
        2,
        DEFAULT_DEVICE_COUNT,
    );
    specification.inode_reservation = InodeReservation::Fixed(3);
    let aggregate = enumerate_schedules(&Model::new(specification), false).aggregate;
    assert_eq!(aggregate.unused_reserved_final_maximum, 4, "每个事务多预留 2 个，两个事务共 4 个号空洞");
    assert_eq!(aggregate.duplicate_total, [0; RESOURCE_COUNT]);
}

// ── 轨迹量 ───────────────────────────────────────────────────────────────

#[test]
fn duplicate_peak_equals_the_largest_final_duplicate_because_holders_only_grow() {
    for (arm, variant, stage, transaction_count) in [
        (Arm::WholeLock, ArmVariant::PositiveControl, ParallelStage::ManyPublishesInFlight, 3),
        (Arm::RegionPartition, ArmVariant::PositiveControl, ParallelStage::ManyPublishesInFlight, 3),
        (Arm::OptimisticWholeVersion, ArmVariant::PositiveControl, ParallelStage::PrepareParallelCommitSerial, 3),
        (Arm::LiteralLock, ArmVariant::Normal, ParallelStage::PrepareParallelCommitSerial, 3),
        (Arm::DevicePartition, ArmVariant::PositiveControl, ParallelStage::PrepareParallelCommitSerial, 2),
    ] {
        let aggregate = partition_cell(arm, variant, stage, ShapeKind::SameShape, false, transaction_count, DEFAULT_DEVICE_COUNT);
        assert_eq!(aggregate.duplicate_peak, aggregate.duplicate_maximum_single_schedule, "{}", arm.label(variant));
        for resource in 0..RESOURCE_COUNT {
            assert!(
                aggregate.duplicate_positive_steps[resource] >= aggregate.duplicate_schedules[resource],
                "有重复的调度至少有一步为正：{}",
                arm.label(variant)
            );
        }
    }
}

#[test]
fn inode_margin_goes_negative_only_when_the_shared_watermark_falls_behind_a_held_number() {
    // 整份写回：事务 1、0 先后按水位 2 拿号并写回水位 3，事务 2 拿 3 写回水位 4，最后事务 1 用旧拷贝把水位写回 3：
    // 已提交的最大号 3，余量 = 3 − 3 − 1 = −1，共享水位退到了已发出的号之下（I-9.6 的反面）。
    let lost_update = partition_cell(
        Arm::WholeLock,
        ArmVariant::PositiveControl,
        ParallelStage::ManyPublishesInFlight,
        ShapeKind::SameShape,
        true,
        3,
        DEFAULT_DEVICE_COUNT,
    );
    assert_eq!(lost_update.inode_margin_minimum, -1);
    assert!(lost_update.inode_margin_negative_steps > 0);
    let correct = partition_cell(
        Arm::WholeLock,
        ArmVariant::Normal,
        ParallelStage::ManyPublishesInFlight,
        ShapeKind::SameShape,
        true,
        3,
        DEFAULT_DEVICE_COUNT,
    );
    assert_eq!(correct.inode_margin_minimum, 0);
    assert_eq!(correct.inode_margin_negative_steps, 0);
}

// ── 同步轮次 ─────────────────────────────────────────────────────────────

fn rounds_of(arm: Arm, stage: ParallelStage, shape_kind: ShapeKind, transaction_count: usize) -> RoundsOutcome {
    simulate_rounds(&Model::new(specification_of(
        arm,
        ArmVariant::Normal,
        stage,
        shape_kind,
        true,
        transaction_count,
        DEFAULT_DEVICE_COUNT,
    )))
}

#[test]
fn synchronous_rounds_match_the_hand_calculation_for_each_arm() {
    let one_publish = ParallelStage::OnePublishCarriesManyTransactions;
    for transaction_count in [2usize, 3] {
        let serial_rounds = u32::try_from(transaction_count).expect("n 很小");
        for stage in ALL_STAGES {
            assert_eq!(rounds_of(Arm::OptimisticWholeVersion, stage, ShapeKind::SameShape, transaction_count).rounds, Some(serial_rounds));
            let all_in_one_round = rounds_of(Arm::RegionPartition, stage, ShapeKind::SameShape, transaction_count);
            assert_eq!(all_in_one_round.rounds, Some(1));
            assert_eq!(all_in_one_round.commits_per_round, vec![serial_rounds]);
            assert_eq!(rounds_of(Arm::DevicePartition, stage, ShapeKind::SameShape, transaction_count).rounds, Some(1));
        }
        assert_eq!(rounds_of(Arm::OptimisticReadSet, one_publish, ShapeKind::SameShape, transaction_count).rounds, Some(serial_rounds));
    }
    let disjoint_three = rounds_of(Arm::OptimisticReadSet, one_publish, ShapeKind::Disjoint, 3);
    assert_eq!(disjoint_three.commits_per_round, vec![2, 1], "事务 1 与 2 争同一个用户数据对，后提交那个下一轮重来");
    assert_eq!(rounds_of(Arm::OptimisticReadSet, one_publish, ShapeKind::Disjoint, 2).commits_per_round, vec![2]);
    assert_eq!(
        rounds_of(Arm::OptimisticReadSet, ParallelStage::ManyPublishesInFlight, ShapeKind::Disjoint, 3).rounds,
        Some(3),
        "Q1丙 下 BASE 版本号在读集里，不相交也一个一个提交"
    );
}

fn descending_order(transaction_count: usize) -> Vec<usize> {
    (0..transaction_count).rev().collect()
}

#[test]
fn commit_order_inside_a_round_changes_the_commits_per_round_for_one_inode_only_one_unit_only_and_one_of_both() {
    // 默认的「不相交」形状（事务 0 只建 inode，其余只写用户数据单元）上，升序与倒序的每轮提交数一样（[2, 1]），
    // 变异 V19 在那里不敏感；这一组形状上升序是 [2, 1]、倒序是 [1, 2]：升序先让事务 0、1 同轮提交、事务 2 与两者都冲突；
    // 倒序先让事务 2 提交，事务 0 与 1 各与它冲突，下一轮才一起提交。总轮数在这两种次序下总是相同（n ≤ 7 的穷举见报告）。
    let mut model = Model::new(specification_of(
        Arm::OptimisticReadSet,
        ArmVariant::Normal,
        ParallelStage::OnePublishCarriesManyTransactions,
        ShapeKind::SameShape,
        true,
        3,
        DEFAULT_DEVICE_COUNT,
    ));
    model.shapes = vec![
        TransactionShape { user_units: 0, new_inodes: 1 },
        TransactionShape { user_units: 1, new_inodes: 0 },
        TransactionShape { user_units: 1, new_inodes: 1 },
    ];
    let ascending = simulate_rounds(&model);
    assert_eq!(ascending.commits_per_round, vec![2, 1]);
    let descending = simulate_rounds_in_order(&model, descending_order);
    assert_eq!(descending.commits_per_round, vec![1, 2]);
    assert_eq!(ascending.rounds, descending.rounds);
}
