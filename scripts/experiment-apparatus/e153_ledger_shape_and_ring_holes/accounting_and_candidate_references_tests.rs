use super::*;

/// Q10（登记第五节 G12「支持它的人认的样子」）：候选根 r 引用槽 x ⟺ 当前账里 x 的记录分配代 ≤
/// txg(r) ∧（未释放 ∨ txg(r) < 释放代）——区间推法与实际走读（M0.6）在「记录还在但候选根自己的账
/// 已经不再指着这个槽」时会分歧，这正是第四段手工诊断在 H4d2 上找到的那类假阳性（登记第十二节
/// 第四段：槽 50317–50320，`actually_referenced_by=[]`）。
#[test]
fn interval_inference_diverges_from_walk_when_the_candidate_root_no_longer_references_the_slot() {
    let mut arm = ArmExecutor::new(Arm::IntervalNarrow, PlacementPolicy::Low);
    // 槽 100：分配代 5、释放代 9（已释放）——区间推法认为候选根 txg=6 落在 [5,9) 里，判「引用」。
    arm.ledger.insert(100, LedgerEntry { span_in_slots: 1, allocation_generation: 5, released: true, release_generation: 9 });
    // 但候选根 txg=6 自己存的账（走读得到的那份）已经不再引用槽 100（它在更晚的覆盖写里换到了别处）。
    let mut refs: HashMap<Role, u64> = HashMap::new();
    refs.insert(Role::ExtentRoot, 200);
    arm.root_accounts.insert(6, account_from_refs(&refs));

    let inferred = interval_inferred_candidate_references(&arm, &[6]);
    let walked = candidate_reference_union(&arm, &[6]);
    assert!(inferred.contains(&100), "区间推法应该认为槽 100 被候选根 6 引用（5 ≤ 6 < 9）");
    assert!(!walked.contains(&100), "候选根 6 自己的账里已经不再引用槽 100");
    assert_ne!(inferred, walked, "这就是 Q10 要抓的那类不一致");
}

/// Q10 的反面：账本与候选根自己的账一致时，两条路径给出同一个并集（不是恒等式，是「这份构造下
/// 恰好一致」）。
#[test]
fn interval_inference_matches_walk_when_the_ledger_and_the_candidate_account_agree() {
    let mut arm = ArmExecutor::new(Arm::IntervalNarrow, PlacementPolicy::Low);
    arm.ledger.insert(100, LedgerEntry { span_in_slots: 1, allocation_generation: 5, released: false, release_generation: 0 });
    let mut refs: HashMap<Role, u64> = HashMap::new();
    refs.insert(Role::ExtentRoot, 100);
    arm.root_accounts.insert(6, account_from_refs(&refs));

    let inferred = interval_inferred_candidate_references(&arm, &[6]);
    let walked = candidate_reference_union(&arm, &[6]);
    assert_eq!(inferred, walked);
    assert!(inferred.contains(&100));
}

/// Q6b（登记第六节 Q6b-甲/乙）：`abandoned_root_item1_bounds` 逐臂返回 (txg 最大的被抛弃根的引用
/// 集合大小, 全部被抛弃根里的最大值)——两个读法在「txg 最大的那条不是引用最多的那条」时会给出
/// 不同的数，这正是读法甲、乙分开报的理由（登记第一节 Q6b-甲/乙「环里有几条被抛弃根时指哪一条
/// 没写，两种读法各报一行」）。
#[test]
fn abandoned_root_item1_bounds_distinguishes_newest_from_maximum() {
    let placement = PlacementPolicy::Low;
    let mut arms: [ArmExecutor; 5] = ARM_ORDER.map(|arm| ArmExecutor::new(arm, placement));
    for arm in &mut arms {
        // txg 5（较旧）：引用 3 个槽——三个不同角色各占一个不同的起点槽。
        let mut older: HashMap<Role, u64> = HashMap::new();
        older.insert(Role::ExtentRoot, 100);
        older.insert(Role::InodeRoot, 102);
        older.insert(Role::AllocationTree, 104);
        arm.root_accounts.insert(5, account_from_refs(&older));
        // txg 8（最新，txg 最大）：只引用 1 个槽。
        let mut newer: HashMap<Role, u64> = HashMap::new();
        newer.insert(Role::ExtentRoot, 200);
        arm.root_accounts.insert(8, account_from_refs(&newer));
    }
    let bounds = abandoned_root_item1_bounds(&arms, &[5, 8]);
    for (newest, maximum_value) in bounds {
        assert_eq!(newest, Some(1), "txg 最大的那条（8）只引用 1 个槽");
        assert_eq!(maximum_value, Some(3), "全部被抛弃根里最大的是 txg 5 的 3 个槽");
    }
    let empty = abandoned_root_item1_bounds(&arms, &[]);
    for (newest, maximum_value) in empty {
        assert_eq!(newest, None, "环里没有被抛弃根时两项都该是 None");
        assert_eq!(maximum_value, None);
    }
}

/// Q6b：`abandoned_root_bound_check` 的四种边界——超过 / 不超过、上界为负 / 非负，逐个钉住。
#[test]
fn abandoned_root_bound_check_matches_hand_computed_cases() {
    // 隔离 5，读法甲上界 = 8 − 3 = 5（不超过，>= 不算超过，只有严格大于才算）；
    // 读法乙上界 = 10 − 3 = 7（更不超过）；两个上界都非负。
    let not_exceeded = abandoned_root_bound_check(5, 8, 10, 3);
    assert_eq!(
        not_exceeded,
        AbandonedRootBoundCheck { isolated_exceeds_newest_bound: false, newest_bound_negative: false, isolated_exceeds_maximum_bound: false, maximum_bound_negative: false }
    );
    // 隔离 6：读法甲上界 5，6 > 5 超过；读法乙上界 7，6 未超过。
    let exceeds_newest_only = abandoned_root_bound_check(6, 8, 10, 3);
    assert!(exceeds_newest_only.isolated_exceeds_newest_bound);
    assert!(!exceeds_newest_only.isolated_exceeds_maximum_bound);
    // R_old 比被抛弃根的代理值还大：上界为负，隔离只要 > 0 就一定超过。
    let negative_bound = abandoned_root_bound_check(1, 2, 2, 5);
    assert!(negative_bound.newest_bound_negative);
    assert!(negative_bound.maximum_bound_negative);
    assert!(negative_bound.isolated_exceeds_newest_bound);
    assert!(negative_bound.isolated_exceeds_maximum_bound);
    // 隔离 0、上界 0：0 > 0 为假，不算超过。
    let zero_bound_zero_isolated = abandoned_root_bound_check(0, 4, 4, 4);
    assert!(!zero_bound_zero_isolated.isolated_exceeds_newest_bound);
    assert!(!zero_bound_zero_isolated.newest_bound_negative);
}

/// 一份记账三项与引用集合都自洽（四个检查都判绿）的合成基底，供坏镜像单测用——不跑完整历史，
/// 只要数值关系对得上：item1 + item2 == 容量，item1 == 两种候选并集，item1 − item5 == 最新引用。
fn green_base_state() -> BaseImageState {
    let item1 = 20u64;
    let item5 = 2u64;
    BaseImageState {
        accounting: AccountingRow { item1_occupied_slots: item1, item2_free_slots: UNIT_AREA_CAPACITY_SLOTS - item1, item5_deferred_slots: item5 },
        candidate_by_effective_floor: item1,
        candidate_by_today_reading: item1,
        newest_reference_size: item1 - item5,
    }
}

/// Q8 起手式：合成的绿色基底四个检查都不判红（对应登记「先证明 G27 在全部合法状态上 0 误红，
/// 再报坏镜像」那句话的最小形态）。
#[test]
fn accounting_checks_of_a_well_formed_base_state_are_all_green() {
    assert_eq!(accounting_checks(green_base_state()), AccountingChecks::default());
}

/// B1a（登记第五节坏镜像）：只翻记录标志，四个检查都判不出——它们都不读分配记录本身。
#[test]
fn bad_image_b1a_is_undetectable_by_any_of_the_four_checks() {
    let checks = accounting_checks(bad_image_b1a(green_base_state()));
    assert_eq!(checks, AccountingChecks::default(), "B1a 只翻记录标志，四个检查都不该判红");
}

/// B1b：B1a 再把第 5 项加 2——只有 G27（读第 5 项）判得出，I-3.1/I-5.2 都不读第 5 项。
#[test]
fn bad_image_b1b_is_caught_only_by_the_allocated_minus_deferred_check() {
    let checks = accounting_checks(bad_image_b1b(green_base_state()));
    assert!(checks.allocated_minus_deferred_red, "第 5 项多 2，G27 该判红");
    assert!(!checks.allocated_statistic_mismatch_by_arm_reading, "I-3.1（臂内读法）不读第 5 项");
    assert!(!checks.allocated_statistic_mismatch_by_today_reading, "I-3.1（今天读法）不读第 5 项");
    assert!(!checks.free_statistic_mismatch, "I-5.2 不读第 5 项");
}

/// B2：第 5 项单独多报一槽——形状与 B1b 相同（只是幅度不同），同样只有 G27 判得出。
#[test]
fn bad_image_deferred_overcount_by_one_is_caught_only_by_the_allocated_minus_deferred_check() {
    let checks = accounting_checks(bad_image_deferred_overcount_by_one(green_base_state()));
    assert!(checks.allocated_minus_deferred_red);
    assert!(!checks.allocated_statistic_mismatch_by_arm_reading && !checks.allocated_statistic_mismatch_by_today_reading && !checks.free_statistic_mismatch);
}

/// PC-F 的 Bk1（阳性对照）：D0 第 1 项多 1 槽——三个基于「第 1 项 vs 引用集合大小」比较的检查
/// 都该判红（I-3.1 两种读法与 G27）；它同时也会破坏 I-5.2（第 1+2 项之和），这是数值上的必然
/// 结果，不是登记「必须看到」那句要否定的东西。
#[test]
fn bad_image_bk1_is_caught_by_the_reference_comparison_checks() {
    let checks = accounting_checks(bad_image_bk1(green_base_state()));
    assert!(checks.allocated_statistic_mismatch_by_arm_reading && checks.allocated_statistic_mismatch_by_today_reading && checks.allocated_minus_deferred_red);
}

/// PC-F 的 Bk2（阳性对照）：D0 第 2 项少 1 槽——只有 I-5.2 判得出，其余三个检查只看第 1/5 项。
#[test]
fn bad_image_bk2_is_caught_only_by_the_free_statistic_check() {
    let checks = accounting_checks(bad_image_bk2(green_base_state()));
    assert!(checks.free_statistic_mismatch, "第 2 项少 1，I-5.2 该判红");
    assert!(!checks.allocated_statistic_mismatch_by_arm_reading && !checks.allocated_statistic_mismatch_by_today_reading, "候选并集比较用的是第 1 项，没变");
    assert!(!checks.allocated_minus_deferred_red, "G27 用第 1 项减第 5 项，都没变");
}

/// Q9a（第四段，见跑前登记第十二节）：`peak_mean_end` 的基本行为——空序列三项都是 0；
/// 非空序列峰值取最大、末值取最后一个、均值按算术平均（不是中位数或别的口径）。
#[test]
fn peak_mean_end_matches_arithmetic_definition() {
    assert_eq!(peak_mean_end(&[]), (0, 0.0, 0));
    let (peak, mean, end) = peak_mean_end(&[3, 7, 2]);
    assert_eq!(peak, 7, "峰值取最大值，不是首值或末值");
    assert!((mean - 4.0).abs() < 1e-9, "均值 = (3+7+2)/3 = 4，实得 {mean}");
    assert_eq!(end, 2, "末值取序列最后一个元素，不是最大或最小");
}

/// Q9a：条目字节 = 条目数 × 条目宽；叶节点数 = ⌈条目数 ÷ 每节点条目数⌉（向上取整，不是向下）。
#[test]
fn allocation_record_bytes_and_leaf_nodes_match_registered_formula() {
    assert_eq!(allocation_record_bytes_and_leaf_nodes(10, 20, 812), (200, 1));
    assert_eq!(allocation_record_bytes_and_leaf_nodes(812, 20, 812), (16240, 1), "整除时叶节点数不多算一个");
    assert_eq!(allocation_record_bytes_and_leaf_nodes(813, 20, 812), (16260, 2), "多 1 条也要多开一个叶节点，不能向下取整");
    assert_eq!(allocation_record_bytes_and_leaf_nodes(1160, 28, 580), (32480, 2));
}
