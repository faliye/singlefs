use super::*;

/// A4：checkpoint 保留池 = c（D28（挂载期承诺量） 已定项 4）。真跑 `gate_snapshot`，
/// 锚点是与公式本身无关的字面量（不是再抄一遍 `fixpoint_cost` 变量），覆盖登记「五、5.5」
/// 两个取样点 c = 4 与 c = 9。
#[test]
fn checkpoint_reserve_equals_fixpoint_cost() {
    for (fixpoint_cost, expected_checkpoint_reserve) in [(4u64, 4u64), (9u64, 9u64)] {
        let geometry = Geometry {
            capacity_slots_per_disk: 600,
            object_units: 1,
            fixpoint_cost,
            slots_per_region: 4,
            padding_deletes: 0,
        };
        let pool = Pool::seed(geometry);
        let snapshot = gate_snapshot(&pool, 0);
        assert_eq!(
            snapshot.checkpoint_reserve, expected_checkpoint_reserve,
            "checkpoint 保留池应等于 c，c = {fixpoint_cost}"
        );
    }
}

/// I2 的锚点公式（`3S − 1`）本身：环里最旧有效根只靠环转一圈往前挪，不靠抬 F、不靠推空——
/// 直接用空发布把 txg 一路推到某个参照点 D 之后，核「再持久多少条空发布，环里最旧有效根
/// 才追上 D」。这与登记「五、5.4」对串族 K 的推导是同一个环形几何事实，只是不经过
/// 准入闸（准入闸会不会额外触发推空是 5.3 的题，这里只核环本身）。
#[test]
fn ring_rotation_only_reaches_reference_txg_after_three_slots_per_region_minus_one_publishes() {
    let geometry = Geometry { capacity_slots_per_disk: 4000, object_units: 1, fixpoint_cost: 4, slots_per_region: 4, padding_deletes: 0 };
    let arm_reclaim = ReclaimTiming::AfterEffectiveFloor;
    let mut pool = Pool::seed(geometry);
    for _ in 0..80 {
        push_one_empty_publish(&mut pool, arm_reclaim, &mut |_last_txg| false);
    }
    let reference_txg = pool.readable_roots().map(|root| root.txg).max().expect("至少一条根");
    let mut steps = 0u64;
    loop {
        if pool.oldest_valid_root_txg() >= reference_txg {
            break;
        }
        push_one_empty_publish(&mut pool, arm_reclaim, &mut |_last_txg| false);
        steps += 1;
    }
    assert_eq!(steps, 3 * geometry.slots_per_region - 1, "环转一圈追上参照点 D 应恰好要 3S − 1 条额外发布");
}

/// 阳性对照 PC2 真的会让 Q2a 计数：`df` 报成 U_all+Rel 之后，串族的 df 会比真实的可用_串
/// 更宽（不扣 M_sw／checkpoint 保留池／占着），至少能在填到快满时出现「df 报的比闸判的宽」。
#[test]
fn positive_control_pc2_produces_q2a_hits() {
    let geometry = Geometry {
        capacity_slots_per_disk: 4000,
        object_units: 16,
        fixpoint_cost: 9,
        slots_per_region: 8,
        padding_deletes: 2,
    };
    let arm = real_baseline_arm();
    let damaged = run_delete_then_rewrite_history_with_damage(arm, geometry, Damage { raw_df: true, ..Damage::default() });
    assert!(damaged.df_reported_enough_but_write_failed_count >= 1, "PC2 改坏 df 之后，填满前后至少要有一次 df≥s 而写失败");
}

/// 阳性对照 PC5：保留量全 0 时，近满盘上固定点分配会失败，卡死。
#[test]
fn positive_control_pc5_wedges_when_reserves_zero() {
    let geometry = Geometry {
        capacity_slots_per_disk: 600,
        object_units: 1,
        fixpoint_cost: 9,
        slots_per_region: 8,
        padding_deletes: 2,
    };
    let mut wedged_any = false;
    for arm in all_arms() {
        let damaged = run_delete_then_rewrite_history_with_damage(arm, geometry, Damage { zero_reserves: true, ..Damage::default() });
        if damaged.wedged {
            wedged_any = true;
        }
    }
    assert!(wedged_any, "PC5：保留量全 0 时，至少要有一条臂在这一格上卡死");
}

/// 阳性对照 PC5 对 H5 也要跑（阳性对照对新历史也要跑）：k=1、位=首
/// 在 `pc5_geometry` 上对每一条臂都退化（实测：`zero_reserves` 把 `checkpoint_reserve`／
/// `switch_reserve` 都清零，gate1 的减项跟着少两项，3 次触动之后 Y 常常不用推空就批准——
/// `FailureInjector` 的窗口一次都没被问到，`injected=0`，H4a／H5 的退化条款按字面触发，这不是 bug，
/// 是「注入的故障没有机会发生」）；k=3、位=覆需要 pushing 推得更久，能撞见真卡死——同一份探针
/// 换成 k=3／覆 之后，24 条臂里多数（除 `串` 族与 `串重判-G7-止`）都在这一格真卡死。
#[test]
fn positive_control_pc5_wedges_root_write_failure_when_reserves_zero() {
    let geometry = Geometry { capacity_slots_per_disk: 600, object_units: 1, fixpoint_cost: 9, slots_per_region: 8, padding_deletes: 2 };
    let mut wedged_any = false;
    for arm in all_arms() {
        let (summary, ..) = run_root_write_failure_history(arm, geometry, 3, FailurePosition::Cover, Damage { zero_reserves: true, ..Damage::default() });
        if summary.wedged {
            wedged_any = true;
        }
    }
    assert!(wedged_any, "PC5 对 H5（k=3、位=覆）：保留量全 0 时，至少要有一条臂在这一格上真卡死，不是退化");
}

/// F5／PC4 的反面：未改坏时，真实基线在这个几何点上不该发出候选根还引用的槽（Q6 = 0）。
#[test]
fn real_baseline_never_reuses_candidate_referenced_slots() {
    let geometry = Geometry {
        capacity_slots_per_disk: 4000,
        object_units: 16,
        fixpoint_cost: 4,
        slots_per_region: 8,
        padding_deletes: 2,
    };
    let arm = real_baseline_arm();
    let summary = run_delete_then_rewrite_history(arm, geometry);
    assert_eq!(summary.unsafe_candidate_reuse_count_field, 0, "真实基线（串-重判 × G7）不该发出候选根还引用的槽");
}
