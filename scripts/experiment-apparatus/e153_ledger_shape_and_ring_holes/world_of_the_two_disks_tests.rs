use super::*;

/// K1：P-impl 的落点几何（三档 S 共 6 格，S 不影响单元区落点）。
#[test]
fn new_pool_file_creation_placements_match_registered_layout() {
    for slots_per_region in [4u64, 8, 16] {
        for workload_period in [1u64, 4] {
            let geometry = GeometryCell { slots_per_region, placement: PlacementPolicy::Impl, workload_period };
            let world = new_pool_file_creation_world(geometry);
            let arm = &world.arms[World::arm_index(Arm::ThresholdConservative)];
            assert_eq!(arm.refs[&Role::DataUnit], 50180, "S={slots_per_region} rho_period={workload_period}");
            assert_eq!(arm.refs[&Role::ExtentRoot], 50240);
            assert_eq!(arm.refs[&Role::InodeLeaf], 50242);
            assert_eq!(arm.refs[&Role::InodeRoot], 50244);
            assert_eq!(arm.refs[&Role::AllocationTree], 50245);
            assert_eq!(arm.refs[&Role::AcctTree], 50246);
            assert_eq!(arm.refs[&Role::CentralMapTree], 50247);
            assert_eq!(arm.refs[&Role::TableUnit], 50248);
            assert!(arm.is_free(50241), "槽 50241 在 txg 3 之后应当空着");
        }
    }
}

/// Q7/G27（第五段）：新池新建文件这一格，第 1 项 13 − 第 5 项 1 == 最新根走读引用 12（A6 已经钉过
/// 13/1，这里补上 G27 自己的算式，五条臂、全部 12 格都要成立——U11 的锚点正是这一行）。
#[test]
fn allocated_minus_deferred_matches_reference_on_new_pool_file_creation_for_every_arm_and_cell() {
    for cell in all_geometry_cells() {
        let world = new_pool_file_creation_world(cell);
        let snapshot = world.invariant_snapshot();
        for (index, arm) in world.arms.iter().enumerate() {
            let row = arm.accounting_row();
            assert_eq!(row.item1_occupied_slots - row.item5_deferred_slots, 12, "{:?} cell={:?}", arm.arm, cell);
            assert!(!snapshot.allocated_minus_deferred_reference_mismatch[index], "G27 不该在新池新建文件这一格误红：{:?} cell={:?}", arm.arm, cell);
        }
    }
}

/// U11（登记第九节 M19）：G27 不减第 5 项时，txg=3 这一行会误报——手写谓词核对变异表要抓的形状，
/// 不依赖 `invariant_snapshot` 里已经写好的算式（那条算式本身就是被测对象）。
#[test]
fn without_subtracting_deferred_the_check_would_misreport_at_new_pool_file_creation() {
    let world = new_pool_file_creation_world(BASE_GEOMETRY);
    let arm = &world.arms[World::arm_index(Arm::ThresholdConservative)];
    let row = arm.accounting_row();
    let newest_reference_size = account_from_refs(&arm.refs).expanded_slots().count() as u64;
    assert_eq!(row.item1_occupied_slots, newest_reference_size + 1, "不减第 5 项时应该比引用多 1（M19 的变异形状）");
    assert_eq!(row.item1_occupied_slots - row.item5_deferred_slots, newest_reference_size, "减了第 5 项才对上");
}

/// Q8 基底：β1（H1 末态）、β2（H4dNm2 回退后第 S 次工作负载之后）两个真实历史算出来的基底状态，
/// 与第四段已经查明的两条事实对得上——甲-T1 在 H1 连续覆盖写下不该判红（每次发布前都回收），
/// 基线「今」该判红（P4：一次挂载之内不回收）；G12（IntervalNarrow）在含回退的族上该判红（第四段
/// 手工诊断的根因，见跑前登记第十二节第四段）。
#[test]
fn beta_one_and_beta_two_bases_match_known_shape() {
    let cell = BASE_GEOMETRY;
    let beta_one = beta_one_world(cell);
    let beta_one_states = base_image_states(&beta_one);
    let threshold_conservative = World::arm_index(Arm::ThresholdConservative);
    let baseline = World::arm_index(Arm::Baseline);
    assert!(
        !accounting_checks(beta_one_states[threshold_conservative]).allocated_statistic_mismatch_by_arm_reading,
        "甲-T1 每次发布前都回收，H1 连续覆盖写下不该在臂内读法上判红"
    );
    assert!(accounting_checks(beta_one_states[baseline]).allocated_statistic_mismatch_by_arm_reading, "基线『今』一次挂载之内不回收，H1 转过一圈之后该判红（P4）");

    let beta_two = beta_two_world(cell).expect("H4dNm2 在基准格上必有候选（第四段产物已验证过）");
    let beta_two_states = base_image_states(&beta_two);
    let interval_narrow = World::arm_index(Arm::IntervalNarrow);
    assert!(accounting_checks(beta_two_states[interval_narrow]).allocated_statistic_mismatch_by_arm_reading, "G12 在含回退的族上该判红（第四段手工诊断已查清根因）");
}

/// Q9a：txg 3（新池新建文件）之后，五条臂的分配记录条目数（`ledger.len()`）都是 10——
/// mkfs 两个角色（实例表单元、树表单元）各留一条（树表单元那条在 txg 3 被覆盖释放但条目不删，
/// D3 已定项 7），加上 txg 3 这次覆盖写新分配的 8 个角色，1+1+8=10；不随 S / 落点政策 / period 变。
#[test]
fn allocation_record_entries_after_new_pool_file_creation_is_ten_on_every_cell_and_arm() {
    for cell in all_geometry_cells() {
        let mut world = World::mkfs(cell);
        for _ in world.first_mount_warmup() {
        }
        let outcome = world.publish(PublishKind::Overwrite);
        for (index, arm) in world.arms.iter().enumerate() {
            // 两条路径都要对上 10：`arm.ledger.len()` 是权威定义（M0.18），
            // `outcome.allocation_record_entries` 是 Q9a 轨迹实际读取的那个字段
            // （`snapshot_allocation_record_read_counters`）——只查前者会漏过后者算错的变异。
            assert_eq!(arm.ledger.len(), 10, "{:?} cell={:?}", arm.arm, cell);
            assert_eq!(outcome.allocation_record_entries[index], 10, "{:?} cell={:?}", arm.arm, cell);
        }
    }
}

/// Q9b（第四段）：H4d2（单次回退）的历史里，`is_rebuild_step` 只在回退那次写行发布上为真——
/// 回退之前 / 之后的普通发布，以及回退后的暖机空发布，都不是紧跟 `rebuild_allocator` 的那一次。
#[test]
fn h4d2_history_has_exactly_one_rebuild_step_outcome() {
    let cell = BASE_GEOMETRY;
    let mut world = World::mkfs(cell);
    for _ in world.first_mount_warmup() {}
    let (outcomes, _target, write_row_txg) = world.history_single_rollback(2).expect("H4d2 在基准几何上必有候选");
    let rebuild_outcomes: Vec<&PublishOutcome> = outcomes.iter().filter(|outcome| outcome.is_rebuild_step).collect();
    assert_eq!(rebuild_outcomes.len(), 1, "H4d2 只回退一次，重建步只应出现一次");
    assert_eq!(rebuild_outcomes[0].txg, write_row_txg, "重建步的采样点必须是回退那次写行发布");
    assert_eq!(rebuild_outcomes[0].label, "write_row");
}

/// A7：测量臂在 H1 稳态（无被抛弃根、环连续）下，多扣与隔离恒为 0。
#[test]
fn measured_arms_over_held_and_isolated_are_zero_in_connected_ring() {
    let cell = BASE_GEOMETRY;
    let mut world = World::mkfs(cell);
    for _ in world.first_mount_warmup() {
    }
    let ring_capacity = cell.ring_capacity();
    let outcomes = world.run_workload_segment(6 * ring_capacity, 1);
    for outcome in outcomes.iter().filter(|outcome| outcome.txg > 3 + 2 * ring_capacity) {
        for index in ARM_ORDER.iter().enumerate().filter(|(_, arm)| arm.is_measured()).map(|(index, _)| index) {
            assert_eq!(outcome.pre[index].over_held_slots, 0, "txg={} arm_index={index}", outcome.txg);
            assert_eq!(outcome.pre[index].isolated_slots, 0, "txg={}", outcome.txg);
        }
    }
}
