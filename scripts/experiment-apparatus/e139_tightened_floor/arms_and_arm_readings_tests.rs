//! 第 1 次重跑的单测（`research/prompts/e139-r1-prereg.md` 第七节的锚点 K1–K9 与第九节要补的几条）。
//! 独立核算脚本 `anchors.py`（登记第十三节末尾）按槽公式摆根、写失败的槽留旧根，这里的期望值照它的输出抄，不读装置。

use super::*;

fn window_simulation(window_arm: WindowArm, slots_per_region: u64, keep_snapshots: bool) -> Simulation {
    let mut simulation = Simulation::new(Arm::TightenedStatesWindowed(window_arm), slots_per_region, 1, 5, 1, keep_snapshots, None);
    simulation.prefill(STEADY_FILL);
    simulation
}

/// 只建不删的窗口一个接一个发布；`failed_txg` 那个 txg 的根槽写失败（推进一格再发，那个 txg 从没写成）。
fn publish_creating_windows_up_to(simulation: &mut Simulation, newest_txg: u64, failed_txg: Option<u64>) {
    while simulation.next_txg <= newest_txg {
        if Some(simulation.next_txg) == failed_txg {
            simulation.fail_next_slot_write = true;
        }
        assert_eq!(simulation.run_user_window(false, false), WindowOutcome::Created, "txg {} 的窗口要发布成功", simulation.next_txg);
    }
}

fn sorted_ring_txgs(simulation: &Simulation) -> Vec<u64> {
    let mut txgs: Vec<u64> = simulation.ring.iter().flatten().map(|root| root.txg).collect();
    txgs.sort_unstable();
    txgs
}

/// 锚 2 / 4 / 7 / 9 共用的状态：S = 4，txg 1..22 依次发布，21 的根槽写失败（22 重发）。
fn anchor_two_state(window_arm: WindowArm, keep_snapshots: bool) -> Simulation {
    let mut simulation = window_simulation(window_arm, 4, keep_snapshots);
    publish_creating_windows_up_to(&mut simulation, 22, Some(21));
    simulation
}

/// 锚 3 / 5 共用的状态：S = 1，txg 1..9 依次发布，7（区域 1、盘 1）写失败（8 重发）。
fn anchor_three_state(window_arm: WindowArm, keep_snapshots: bool) -> Simulation {
    let mut simulation = window_simulation(window_arm, 1, keep_snapshots);
    publish_creating_windows_up_to(&mut simulation, 9, Some(7));
    simulation
}

#[test]
fn window_is_txg_greater_than_anchor_minus_ring_length() {
    let lowest_inside = (0..=35u64).find(|txg| txg_is_inside_window(*txg, 35, REGION_COUNT * 8)).expect("窗口里有 txg");
    let highest_outside = (0..=35u64).filter(|txg| !txg_is_inside_window(*txg, 35, REGION_COUNT * 8)).max().expect("窗口外有 txg");
    assert_eq!((lowest_inside, highest_outside), (12, 11), "anchor1 window lowest in = 12 highest out = 11");
    let mut simulation = window_simulation(WindowArm::Window, 8, false);
    simulation.newest_durable_txg = 35;
    assert_eq!(simulation.ring_depth, 24, "S = 8 时环长 3S = 24");
    assert!(simulation.is_inside_candidate_window(12) && !simulation.is_inside_candidate_window(11), "候选集的窗口判定走同一个谓词");
}

#[test]
fn window_arms_keep_the_tightened_states_bound_reserve_and_residue() {
    for window_arm in WINDOW_ARMS.iter().chain(WINDOW_SHORT_ARMS.iter()) {
        let arm = Arm::TightenedStatesWindowed(*window_arm);
        assert_eq!(arm.publication_bound(12), 4 + 4, "{} B = 4 + 2 × k_tol = 8", window_arm.name());
        assert_eq!((arm.reserve_blocks(12, 5), arm.push_residue_blocks(12, 5)), (35 + 10, 30 + 5), "{} c_max = 5：保留池 45、残留 35", window_arm.name());
        assert_eq!((arm.reserve_blocks(12, 10), arm.push_residue_blocks(12, 10)), (70 + 10, 60 + 5), "{} c_max = 10：保留池 80、残留 65", window_arm.name());
    }
}

#[test]
fn ring_has_nine_outside_the_window_and_the_two_gates_differ_by_two() {
    for (window_arm, expected_gate) in [(WindowArm::Window, 11u64), (WindowArm::DiskHold, 11), (WindowArm::RingLag, 11), (WindowArm::RingGate, 9)] {
        let mut simulation = anchor_two_state(window_arm, false);
        assert_eq!(sorted_ring_txgs(&simulation), vec![9, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 22], "{} anchor2 ring", window_arm.name());
        assert_eq!(simulation.newest_durable_txg, 22);
        assert_eq!(simulation.oldest_persisted_valid_txg(), 9, "anchor2 ring_oldest = 9");
        assert_eq!(simulation.oldest_valid_txg_in_the_reclaim_window(simulation.ring_depth), Some(11), "anchor2 window_oldest = 11");
        assert_eq!(simulation.reuse_bound(), expected_gate, "{} 回收门槛（F_生效 = 0）", window_arm.name());
        // 释放代 10、11 的已释放块：窗口门槛放得回，整环门槛放不回
        let reusable_before = simulation.reusable_count();
        simulation.pool.blocks[0] = BlockState::Freed { allocated_at: 1, freed_at: 10 };
        simulation.pool.blocks[1] = BlockState::Freed { allocated_at: 1, freed_at: 11 };
        let reusable_gain = simulation.reusable_count() - reusable_before;
        assert_eq!(reusable_gain, if window_arm == WindowArm::RingGate { 0 } else { 2 }, "{} 释放代 10、11 的块可再分配的个数", window_arm.name());
    }
}

#[test]
fn root_nine_outside_the_window_is_not_a_fake_candidate_but_would_be_without_the_window() {
    for window_arm in WINDOW_ARMS {
        let mut simulation = anchor_two_state(window_arm, false);
        simulation.damaged_txgs.clear();
        simulation.damaged_txgs.insert(9);
        let (fake, candidates, _, _) = simulation.count_fake_candidates();
        assert_eq!((fake, candidates), (0, 11), "{} 根 9 在窗口外：不进候选集、不记假候选，候选 11 个", window_arm.name());
        let without_window = simulation.valid_roots().iter().filter(|root| simulation.damaged_txgs.contains(&root.txg)).count();
        assert_eq!(without_window, 1, "没有窗口这一项时根 9 记成 1 个假候选");
    }
}

#[test]
fn ceiling_readings_skipping_and_zero_counting_and_the_two_lag_bound_readings_differ_on_the_skipped_disk() {
    let simulation = anchor_three_state(WindowArm::Window, false);
    let ring_by_slot: Vec<Option<u64>> = simulation.ring.iter().map(|root| root.map(|record| record.txg)).collect();
    assert_eq!(ring_by_slot, vec![Some(9), Some(4), Some(8)], "anchor3 ring = {{0: 9, 1: 4, 2: 8}}");
    assert_eq!(simulation.windowed_states_upper_limit(WindowArm::Window), 8, "读法 P：盘 1 窗口里没有根，不进取小，上限 8");
    assert_eq!(simulation.windowed_states_upper_limit(WindowArm::DiskHold), 0, "读法 Q：盘 1 记 0，上限 0");
    assert_eq!(simulation.per_disk_states_upper_limit(), 4, "原 G2S 整环取法：上限 4");
    assert_eq!(simulation.windowed_lag_bound_txg(WindowArm::Window), 8, "读法 a：滞后量下界 8");
    assert_eq!(simulation.windowed_lag_bound_txg(WindowArm::RingLag), 4, "读法 b：滞后量下界 4");
}

#[test]
fn window_arms_in_steady_use_three_or_four_publications_and_lag_three_windows() {
    for window_arm in WINDOW_ARMS {
        for (slots_per_region, cost, publications, maximum, pinned, lag) in [
            (1u64, 1u64, vec![3u64], 3u64, 6u64, 0u64),
            (1, 5, vec![3], 3, 10, 0),
            (4, 1, vec![3, 4], 4, 7, 39),
            (4, 5, vec![3, 4], 4, 15, 39),
            (16, 1, vec![3, 4], 4, 7, 39),
            (16, 5, vec![3, 4], 4, 15, 39),
        ] {
            let (publication_set, maximum_publications, maximum_pinned, maximum_lag, is_all_released) =
                push_probe_over_landings(Arm::TightenedStatesWindowed(window_arm), slots_per_region, cost);
            assert!(is_all_released, "{} S={slots_per_region} c={cost} 处置之后目标必须放回", window_arm.name());
            assert_eq!(publication_set, publications, "{} S={slots_per_region} c={cost} 三种落点的发布数", window_arm.name());
            assert_eq!(maximum_publications, maximum, "{} S={slots_per_region} c={cost} 最多 4 次", window_arm.name());
            assert_eq!(maximum_pinned, pinned, "{} S={slots_per_region} c={cost} 滞后量之外扣住最多 5 + 2c", window_arm.name());
            assert_eq!(maximum_lag, lag, "{} S={slots_per_region} c={cost} 滞后量 lag 3 windows x 13 = 39", window_arm.name());
        }
    }
}

#[test]
fn gate_difference_counter_reads_two_on_the_anchor_two_state_and_zero_in_steady() {
    for (window_arm, expected_difference) in [(WindowArm::Window, 2i64), (WindowArm::RingGate, 0)] {
        let mut simulation = anchor_two_state(window_arm, false);
        simulation.metrics = Metrics::fresh();
        simulation.allocate(1, 23).expect("Unused 块还很多");
        let gate_difference = simulation.metrics.trajectory.gate_difference;
        assert_eq!((gate_difference.samples, gate_difference.maximum), (1, expected_difference), "{} 门槛差 d = 11 − 9", window_arm.name());
        assert_eq!(gate_difference.positive_samples, u64::from(expected_difference > 0), "{} d > 0 记一次", window_arm.name());
    }
    let mut steady = window_simulation(WindowArm::Window, 4, false);
    for _ in 0..60 {
        steady.run_user_window(true, false);
    }
    let steady_difference = steady.metrics.trajectory.gate_difference;
    assert!(steady_difference.samples > 0, "steady 里 allocate 要真的被读到，否则下一条什么都没测");
    assert_eq!((steady_difference.positive_samples, steady_difference.maximum), (0, 0), "没有写失败的 steady 里 d 恒为 0");
}

#[test]
fn window_boundary_is_strict_the_hole_left_by_a_failed_write_keeps_generation_zero_outside() {
    let mut simulation = window_simulation(WindowArm::Window, 4, false);
    publish_creating_windows_up_to(&mut simulation, 12, Some(5));
    assert_eq!(sorted_ring_txgs(&simulation), vec![0, 1, 2, 3, 4, 6, 7, 8, 9, 10, 11, 12], "anchor8 ring");
    assert_eq!(simulation.newest_durable_txg, 12);
    assert_eq!(simulation.oldest_valid_txg_in_the_reclaim_window(simulation.ring_depth), Some(1), "anchor8 window_oldest '>' = 1");
    let with_greater_or_equal = sorted_ring_txgs(&simulation).into_iter().filter(|txg| txg + simulation.ring_depth >= 12).min();
    assert_eq!(with_greater_or_equal, Some(0), "anchor8 window_oldest '>=' = 0");
}

#[test]
fn shortened_window_gate_is_fourteen_on_the_anchor_two_state() {
    let simulation = anchor_two_state(WindowArm::WindowShort, false);
    assert_eq!(simulation.oldest_valid_txg_in_the_reclaim_window(simulation.ring_depth - REGION_COUNT), Some(14), "anchor9 short-window oldest = 14");
    assert_eq!(simulation.reuse_bound(), 14, "缩短一行的窗口：回收门槛 14");
    assert_eq!(anchor_two_state(WindowArm::Window, false).reuse_bound(), 11, "对照：整窗口门槛 11");
}

/// 变异 MW2 / MW8 的敏感取样点：管理员回退之后、下一次发布之前，锚是回退目标 11，不是回退前的最大 txg 22。
#[test]
fn after_admin_rollback_the_window_is_anchored_at_the_rollback_target() {
    let mut simulation = anchor_two_state(WindowArm::Window, true);
    assert!(simulation.admin_rollback(), "窗口里最旧的候选根 11 是回退目标");
    assert_eq!(simulation.newest_durable_txg, 11, "回退之后锚停在回退目标（装置的取法，登记第十一节差异 ⑦）");
    assert_eq!(simulation.maximum_published_txg, 22, "回退前的最大 txg 仍是 22（变异会拿它当锚）");
    assert_eq!(simulation.reuse_bound(), 9, "窗口锚在 11：根 9 在窗口里，门槛 9（锚取 22 时是 11）");
    let (_, candidates, _, _) = simulation.count_fake_candidates();
    assert_eq!(candidates, 2, "候选集 {{9, 11}}（锚取 22 时只剩 {{11}}）");
}

#[test]
fn rollback_target_is_the_oldest_root_inside_the_window_not_the_oldest_in_the_ring() {
    let mut simulation = anchor_two_state(WindowArm::Window, true);
    simulation.admin_rollback();
    assert_eq!(simulation.newest_durable_txg, 11, "W2：回退目标取窗口里最旧的候选根 11，不取窗口外的 9");
    let mut ring_gate_simulation = anchor_two_state(WindowArm::RingGate, true);
    ring_gate_simulation.admin_rollback();
    assert_eq!(ring_gate_simulation.newest_durable_txg, 11, "G2SW_ringgate 的 W2 照旧（只有 W1 换回旧式）");
}

/// 登记第五节 W2 的「等价」一条：这台装置里丢盘只在崩溃时发生，恢复后锚 = 择中的根 = 幸存盘上最新的有效根，它总在窗口里，
/// 所以「每块幸存盘上最新的有效根在候选集里」那一查加不加窗口同值。
#[test]
fn after_disk_loss_the_newest_valid_root_on_every_surviving_disk_is_inside_the_candidate_window() {
    for lost_disk in 0..DISK_COUNT {
        for mut simulation in [anchor_two_state(WindowArm::Window, true), anchor_three_state(WindowArm::Window, true)] {
            simulation.disk_lost[lost_disk] = true;
            simulation.crash_and_recover();
            for disk in simulation.surviving_disks() {
                let newest = simulation.newest_valid_txg_on_disk(disk).expect("幸存盘上有有效根");
                assert!(simulation.is_inside_candidate_window(newest), "丢盘 {lost_disk} 之后幸存盘 {disk} 上最新的有效根 {newest} 要在窗口里");
            }
        }
    }
}

#[test]
fn ceiling_difference_counter_reads_the_two_readings_against_the_whole_ring_ceiling() {
    for (window_arm, expected_difference) in [(WindowArm::Window, 4i64), (WindowArm::DiskHold, -4)] {
        let mut simulation = anchor_three_state(window_arm, false);
        simulation.metrics = Metrics::fresh();
        simulation.observe_ceiling_difference();
        let ceiling_difference = simulation.metrics.trajectory.ceiling_difference;
        assert_eq!((ceiling_difference.samples, ceiling_difference.maximum, ceiling_difference.minimum), (1, expected_difference, expected_difference),
            "{} 上限差 = 窗口上限 − 整环上限（读法 P 8 − 4，读法 Q 0 − 4）", window_arm.name());
    }
}

#[test]
fn jump_region_prefix_with_region_one_is_the_original_prefix() {
    for slots_per_region in [4u64, 16] {
        for seed in 0..3 {
            let original = activation_prefix(Arm::TightenedStatesWindowed(WindowArm::Window), slots_per_region, seed, 2).expect("前缀不该卡死");
            let copy = activation_prefix_with_jump_region(Arm::TightenedStatesWindowed(WindowArm::Window), slots_per_region, seed, 2, 1).expect("前缀不该卡死");
            assert_eq!((sorted_ring_txgs(&original), original.newest_durable_txg, original.floor_carried, original.root_write_attempts),
                (sorted_ring_txgs(&copy), copy.newest_durable_txg, copy.floor_carried, copy.root_write_attempts), "S={slots_per_region} 种子 {seed}");
        }
    }
}

#[test]
fn window_arms_agree_with_the_original_states_arm_in_steady_where_the_ring_has_no_hole() {
    for slots_per_region in [4u64, 16] {
        for seed in 0..3 {
            let original = run_world(Arm::TightenedStates, World::Steady, slots_per_region, seed, 0);
            for window_arm in WINDOW_ARMS {
                let windowed = run_world(Arm::TightenedStatesWindowed(window_arm), World::Steady, slots_per_region, seed, 0);
                assert_eq!(world_outcome_fields(&windowed), world_outcome_fields(&original), "{} S={slots_per_region} 种子 {seed}：环里没有洞时与原 G2S 逐字相同", window_arm.name());
            }
        }
    }
}
