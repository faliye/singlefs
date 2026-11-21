//! 第 2 次重跑的单测（`research/prompts/e139-r2-prereg.md` 第七节的锚点 K2r、K5r、K6r、K9r、K10–K16，与第九节变异要的取样点）。
//! 独立核算脚本 `anchors_r2.py`（登记第十三节末尾）按槽公式摆根、写失败的槽留旧根，这里的期望值照它的输出抄，不读装置。

use super::*;

const CLAUSE_WINDOW_ARM: Arm = Arm::ClauseFloorWindowed(WindowArm::Window);
const CLAUSE_RING_GATE_ARM: Arm = Arm::ClauseFloorWindowed(WindowArm::RingGate);
const WHOLE_RING_ROOTLESS_SKIPPING_ARM: Arm = Arm::WholeRing(WholeRingArm::SkipsRootlessDisks);
const WHOLE_RING_ROOTLESS_ZERO_COUNTING_ARM: Arm = Arm::WholeRing(WholeRingArm::CountsRootlessDisksAsZero);

fn simulation_with(arm: Arm, slots_per_region: u64, fill: f64, keep_snapshots: bool) -> Simulation {
    let mut simulation = Simulation::new(arm, slots_per_region, 1, 5, 1, keep_snapshots, None);
    simulation.prefill(fill);
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

fn run_delete_create_windows(simulation: &mut Simulation, windows: u64) {
    for _ in 0..windows {
        assert_eq!(simulation.run_user_window(true, false), WindowOutcome::Created, "卡住的盘不盘紧时每个窗口都要发布成功");
    }
}

fn sorted_ring_txgs(simulation: &Simulation) -> Vec<u64> {
    let mut txgs: Vec<u64> = simulation.ring.iter().flatten().map(|root| root.txg).collect();
    txgs.sort_unstable();
    txgs
}

fn sorted_valid_txgs(simulation: &Simulation) -> Vec<u64> {
    let mut txgs: Vec<u64> = simulation.valid_roots().iter().map(|root| root.txg).collect();
    txgs.sort_unstable();
    txgs
}

/// 锚 K11 / K9r 共用的状态：S = 4，txg 1..22 依次发布，21 的根槽写失败（22 重发）。
fn anchor_two_state(arm: Arm, keep_snapshots: bool) -> Simulation {
    let mut simulation = simulation_with(arm, 4, STEADY_FILL, keep_snapshots);
    publish_creating_windows_up_to(&mut simulation, 22, Some(21));
    simulation
}

/// 锚 K5r 的状态：S = 1，txg 1..9 依次发布，7（区域 1、盘 1）写失败（8 重发）。
fn anchor_three_state(arm: Arm) -> Simulation {
    let mut simulation = simulation_with(arm, 1, STEADY_FILL, false);
    publish_creating_windows_up_to(&mut simulation, 9, Some(7));
    simulation
}

/// 锚 K10：S = 4，占用 0.70，10 个删建窗口（txg 1..10，不盘紧）；直接抬 F 到 6（系统配置记 6）；第 11 个窗口的根撕裂、崩溃恢复；恢复之后 `damaged_txgs` 设成 {2, 3}。
fn state_with_the_system_configuration_floor_raised_to_six(arm: Arm) -> Simulation {
    let mut simulation = simulation_with(arm, 4, STEADY_FILL, true);
    run_delete_create_windows(&mut simulation, 10);
    simulation.raise_floor_to(6);
    simulation.tear_next_publication = true;
    simulation.run_user_window(true, false);
    simulation.damaged_txgs = HashSet::from([2u64, 3]);
    simulation
}

#[test]
fn k2r_whole_ring_arms_keep_the_tightened_states_bound_reserve_and_residue() {
    for whole_ring_arm in WHOLE_RING_ARMS.iter().chain(WHOLE_RING_SHORT_ARMS.iter()) {
        let arm = Arm::WholeRing(*whole_ring_arm);
        assert_eq!(arm.publication_bound(12), 4 + 4, "{} B = 4 + 2 × k_tol = 8", arm.name());
        assert_eq!((arm.reserve_blocks(12, 5), arm.push_residue_blocks(12, 5)), (35 + 10, 30 + 5), "{} c_max = 5：保留池 45、残留 35", arm.name());
        assert_eq!((arm.reserve_blocks(12, 10), arm.push_residue_blocks(12, 10)), (70 + 10, 60 + 5), "{} c_max = 10：保留池 80、残留 65", arm.name());
    }
}

/// 变异 MF1–MF5、MC3 的取样点：根上带的 F 都是 0、系统配置里是 6（`raise_floor_to` 记下的），撕裂的根在恢复之后不自证。
#[test]
fn system_configuration_floor_six_above_root_floors_zero_lifts_the_candidate_floor_and_the_rollback_target() {
    for arm in [CLAUSE_WINDOW_ARM, WHOLE_RING_ROOTLESS_SKIPPING_ARM, WHOLE_RING_ROOTLESS_ZERO_COUNTING_ARM] {
        let simulation = state_with_the_system_configuration_floor_raised_to_six(arm);
        assert_eq!(simulation.slot_index(11), 11, "{} K10 torn_slot = 11", arm.name());
        assert_eq!(sorted_valid_txgs(&simulation), (0..=10).collect::<Vec<u64>>(), "{} K10 valid = [0..10]", arm.name());
        assert_eq!((simulation.floor_carried, simulation.floor_active), (0, 0), "{} 差异 ③ 不动：恢复之后 floor_carried、floor_active 取各根所带的 F，仍是 0", arm.name());
        assert_eq!(simulation.system_configuration_floor, 6, "{} 系统配置里记的是 raise_floor_to 抬到的 6", arm.name());
        assert_eq!(simulation.candidacy_floor(), 0, "{} 旧下界：根上带的 F 都是 0", arm.name());
        assert_eq!(simulation.candidate_floor(), 6, "{} K10a：条款形下界 = 6", arm.name());
        assert_eq!(simulation.candidate_floor(), simulation.candidacy_floor() + 6, "{} g = 6", arm.name());
        let (old_fakes, old_candidates, _, _) = simulation.count_fake_candidates_at_floor(simulation.candidacy_floor());
        let (new_fakes, new_candidates, _, _) = simulation.count_fake_candidates();
        assert_eq!((old_candidates, new_candidates, old_fakes, new_fakes), (11, 5, 2, 0), "{} K10 候选数 旧 11、CF 5；假候选 旧 2、CF 0", arm.name());
        assert_eq!(simulation.oldest_candidate_root_at_floor(simulation.candidacy_floor()).map(|root| root.txg), Some(0), "{} 回退目标 旧 0", arm.name());
        assert_eq!(simulation.oldest_candidate_root_at_floor(simulation.candidate_floor()).map(|root| root.txg), Some(6), "{} 回退目标 CF 6", arm.name());
        let mut rolled_back = simulation.clone();
        assert!(rolled_back.admin_rollback(), "{} 回退要成功", arm.name());
        assert_eq!(rolled_back.newest_durable_txg, 6, "{} admin_rollback 真的回到条款形下界选的目标 6", arm.name());
        assert_eq!(rolled_back.metrics.second_rerun.rollback_target_changed, 1, "{} 新旧下界选的目标不同记一次", arm.name());
        let gap = simulation.metrics.second_rerun.candidate_floor_gap;
        assert_eq!((gap.maximum, gap.minimum), (6, 0), "{} 恢复那次 record_check 读到 g = 6，撕裂之前的 g 全是 0", arm.name());
        let recovered = simulation.metrics.second_rerun.pinned_after_recovery;
        assert_eq!((recovered.samples, recovered.maximum, recovered.last), (1, 130, 130), "{} K10 pinned_after_recovery = 13 × 10，恢复只采一次", arm.name());
        assert_eq!(simulation.metrics.second_rerun.system_configuration_below_root_floor_events, 0, "{} K16", arm.name());
    }
}

/// 同一状态上旧候选下界的臂（开关关着）逐次相同：候选下界仍是 0。
#[test]
fn old_candidate_floor_arm_does_not_see_the_system_configuration_floor() {
    let simulation = state_with_the_system_configuration_floor_raised_to_six(Arm::TightenedStatesWindowed(WindowArm::Window));
    assert_eq!(simulation.system_configuration_floor, 6, "开关开不开都记系统配置 F");
    assert_eq!(simulation.candidate_floor(), 0, "开关关着：候选下界仍是旧的 0");
    let (fakes, candidates, _, _) = simulation.count_fake_candidates();
    assert_eq!((fakes, candidates), (2, 11), "旧下界：候选 11、假候选 2");
    assert_eq!(simulation.metrics.second_rerun.pinned_after_recovery.samples, 0, "旧臂不采第 2 次重跑的新量");
}

/// 变异 MR1、MR2、MC1 的取样点：第 1 次 K3 的构造上 RGall 的门槛取整环（9）、候选集不滤窗口（12 个）、上限 18；窗口那一族是 11 / 11。
#[test]
fn rgall_gate_is_nine_and_candidates_twelve_where_the_window_arm_has_gate_eleven_and_candidates_eleven() {
    let rgall = anchor_two_state(WHOLE_RING_ROOTLESS_SKIPPING_ARM, false);
    let window = anchor_two_state(CLAUSE_WINDOW_ARM, false);
    let mut expected_ring: Vec<u64> = vec![9];
    expected_ring.extend(11..=20);
    expected_ring.push(22);
    for simulation in [&rgall, &window] {
        assert_eq!(sorted_ring_txgs(simulation), expected_ring, "K11 ring = [9, 11, …, 20, 22]");
        assert_eq!(simulation.newest_durable_txg, 22);
    }
    assert_eq!((rgall.reuse_bound(), window.reuse_bound()), (9, 11), "K11 rgall_gate = 9 window_gate = 11（F_生效 = 0）");
    assert_eq!((rgall.count_fake_candidates().1, window.count_fake_candidates().1), (12, 11), "K11 rgall_candidates = 12 window_candidates = 11");
    assert_eq!(rgall.floor_upper_limit(), 18, "K11 rgall_ceiling = min(盘 0 最新 20, 盘 1 最新 22, 第 4 新的非空 18)");
    assert_eq!(rgall.slots_holding_a_valid_root_outside_the_window(), 1, "K11 holes (slots outside window) = 1");
    for (arm, mut simulation) in [(WHOLE_RING_ROOTLESS_SKIPPING_ARM, rgall), (CLAUSE_WINDOW_ARM, window)] {
        let extra_pinned_before = simulation.extra_pinned_blocks_against_the_window_gate();
        // 释放代 10、11 各放一块已释放块：落在 (9, 11] 里，整环门槛扣住、窗口门槛放得回
        simulation.pool.blocks[0] = BlockState::Freed { allocated_at: 1, freed_at: 10 };
        simulation.pool.blocks[1] = BlockState::Freed { allocated_at: 1, freed_at: 11 };
        let extra_pinned_after = simulation.extra_pinned_blocks_against_the_window_gate();
        if arm == WHOLE_RING_ROOTLESS_SKIPPING_ARM {
            assert_eq!((extra_pinned_before, extra_pinned_after), (10, 10 + 2), "RGall：释放代 10、11 的元数据各 5 块加手放的 2 块，x = 12，h = 1");
        } else {
            assert_eq!((extra_pinned_before, extra_pinned_after), (0, 0), "V8：窗口那一族的回收门槛就是窗口门槛，x 恒 0");
        }
    }
}

/// 变异 MR3、MR4 的取样点：第 1 次 K5 的构造（S = 1，txg 1..9，7 写失败）上 RGall 的上限与滞后量下界取整环（4），窗口读法是 8。
#[test]
fn k5r_rgall_ceiling_and_lag_bound_read_the_whole_ring() {
    for arm in [WHOLE_RING_ROOTLESS_SKIPPING_ARM, WHOLE_RING_ROOTLESS_ZERO_COUNTING_ARM] {
        let simulation = anchor_three_state(arm);
        assert_eq!(simulation.floor_upper_limit(), 4, "{} K5r rgall_ceiling = 4（窗口读法 P 是 8）", arm.name());
        assert_eq!(simulation.lag_bound_txg(), 4, "{} K5r rgall_lag_bound = 4（读法 a 是 8）", arm.name());
    }
    let window = anchor_three_state(CLAUSE_WINDOW_ARM);
    assert_eq!((window.floor_upper_limit(), window.lag_bound_txg()), (8, 8), "对照：条款形下界的窗口臂仍是 8 / 8（只换候选下界，不换窗口取法）");
}

/// 变异 MR5 的取样点：S = 1，txg 1..5 全部非空，管理员回退到候选集里最旧的根 3；盘 1 一条有效根都没有：读法 P 上限 3、读法 Q 上限 0。
#[test]
fn rgall_skipping_reading_skips_the_disk_without_a_valid_root_and_zero_counting_reading_counts_it_as_zero() {
    for (arm, expected_ceiling) in [(WHOLE_RING_ROOTLESS_SKIPPING_ARM, 3u64), (WHOLE_RING_ROOTLESS_ZERO_COUNTING_ARM, 0)] {
        let mut simulation = simulation_with(arm, 1, STEADY_FILL, true);
        publish_creating_windows_up_to(&mut simulation, 5, None);
        let ring_by_slot: Vec<Option<u64>> = simulation.ring.iter().map(|root| root.map(|record| record.txg)).collect();
        assert_eq!(ring_by_slot, vec![Some(3), Some(4), Some(5)], "K12 ring = {{0: 3, 1: 4, 2: 5}}");
        assert!(simulation.admin_rollback(), "候选集里最旧的根 3 是回退目标");
        assert_eq!(sorted_valid_txgs(&simulation), vec![3], "K12 valid_after_rollback = {{0: 3}}：盘 1 上一条有效根都没有");
        assert_eq!(simulation.newest_valid_txg_on_disk(1), None);
        assert_eq!(simulation.floor_upper_limit(), expected_ceiling, "{} K12 ceiling_P = 3 ceiling_Q = 0", arm.name());
        assert_eq!(simulation.metrics.second_rerun.disk_without_valid_root_events, 0, "回退之前没有读过上限");
        simulation.observe_ceiling_difference();
        assert_eq!(simulation.metrics.second_rerun.disk_without_valid_root_events, 1, "{} K12 disk_without_valid_root_events 记一次", arm.name());
    }
}

/// 变异 MR6 的取样点：K11 的状态套 RGall_short 的修饰（txg + 9 > 22 那一段），门槛 14；整环是 9。
#[test]
fn k9r_shortened_window_gate_of_the_rgall_positive_controls_is_fourteen() {
    for whole_ring_arm in WHOLE_RING_SHORT_ARMS {
        let simulation = anchor_two_state(Arm::WholeRing(whole_ring_arm), false);
        assert_eq!(simulation.reuse_bound(), 14, "{} K9r rgall_short_gate = 14", whole_ring_arm.name());
    }
    assert_eq!(anchor_two_state(WHOLE_RING_ROOTLESS_SKIPPING_ARM, false).reuse_bound(), 9, "对照：RGall 整环门槛 9");
}

/// K6r：steady 里从稳态点触发一次完整处置，RGall 两条臂各 3 或 4 次发布、扣住最多 5 + 2c、滞后量正好 39；S = 1 时 3 次。
#[test]
fn k6r_rgall_arms_in_steady_use_three_or_four_publications_and_lag_three_windows() {
    for whole_ring_arm in WHOLE_RING_ARMS {
        for (slots_per_region, cost, publications, maximum, pinned, lag) in [
            (1u64, 1u64, vec![3u64], 3u64, 6u64, 0u64),
            (1, 5, vec![3], 3, 10, 0),
            (4, 1, vec![3, 4], 4, 7, 39),
            (4, 5, vec![3, 4], 4, 15, 39),
            (16, 1, vec![3, 4], 4, 7, 39),
            (16, 5, vec![3, 4], 4, 15, 39),
        ] {
            let arm = Arm::WholeRing(whole_ring_arm);
            let (publication_set, maximum_publications, maximum_pinned, maximum_lag, is_all_released) = push_probe_over_landings(arm, slots_per_region, cost);
            assert!(is_all_released, "{} S={slots_per_region} c={cost} 处置之后目标必须放回", arm.name());
            assert_eq!(publication_set, publications, "{} S={slots_per_region} c={cost} 三种落点的发布数", arm.name());
            assert_eq!(maximum_publications, maximum, "{} S={slots_per_region} c={cost} 最多 4 次", arm.name());
            assert_eq!(maximum_pinned, pinned, "{} S={slots_per_region} c={cost} 滞后量之外扣住最多 5 + 2c", arm.name());
            assert_eq!(maximum_lag, lag, "{} S={slots_per_region} c={cost} 滞后量 lag 3 windows x 13 = 39", arm.name());
        }
    }
}

fn stuck_from_the_first_window(arm: Arm, stuck_disk: usize, fill: f64) -> Simulation {
    let mut simulation = simulation_with(arm, 4, fill, false);
    simulation.stuck_disk = Some(stuck_disk);
    simulation.stuck_marks_origin_txg = Some(simulation.next_txg);
    simulation
}

/// 变异 MS1、MS2、MC1、MC2 的取样点：S = 4，占用 0.30，盘 1 从第一个窗口起卡住（区域 1 的槽写不进去），不盘紧；每个窗口放掉 13 块。
#[test]
fn disk_one_stuck_rgall_pins_thirteen_blocks_per_window_and_the_window_arm_stays_at_ninety_one() {
    for (windows, newest_txg, window_gate, rgall_pinned, window_pinned) in [(8u64, 12u64, 2u64, 104u64, 91u64), (16, 24, 14, 208, 91), (32, 48, 38, 416, 91)] {
        let mut rgall = stuck_from_the_first_window(WHOLE_RING_ROOTLESS_SKIPPING_ARM, 1, 0.30);
        let mut window = stuck_from_the_first_window(CLAUSE_WINDOW_ARM, 1, 0.30);
        let mut ring_gate = stuck_from_the_first_window(CLAUSE_RING_GATE_ARM, 1, 0.30);
        for simulation in [&mut rgall, &mut window, &mut ring_gate] {
            run_delete_create_windows(simulation, windows);
            assert_eq!(simulation.newest_durable_txg, newest_txg, "K13 w = {windows}：N = {newest_txg}");
            assert_eq!(simulation.slots_holding_a_valid_root_outside_the_window(), 4, "K13 w = {windows}：盘 1 的 4 个槽一直住着第 0 代，窗口外");
        }
        assert_eq!((rgall.reuse_bound(), window.reuse_bound(), ring_gate.reuse_bound()), (0, window_gate, 0), "K13 w = {windows}：RGall 门槛 0、窗口门槛 {window_gate}");
        assert_eq!((rgall.pinned_count(), window.pinned_count(), ring_gate.pinned_count()), (rgall_pinned, window_pinned, rgall_pinned), "K13 w = {windows}：扣住的块数");
    }
    let mut rgall = stuck_from_the_first_window(WHOLE_RING_ROOTLESS_SKIPPING_ARM, 1, 0.30);
    let mut window = stuck_from_the_first_window(CLAUSE_WINDOW_ARM, 1, 0.30);
    let mut ring_gate = stuck_from_the_first_window(CLAUSE_RING_GATE_ARM, 1, 0.30);
    for simulation in [&mut rgall, &mut window, &mut ring_gate] {
        run_delete_create_windows(simulation, 16);
    }
    assert_eq!(
        (rgall.extra_pinned_blocks_against_the_window_gate(), ring_gate.extra_pinned_blocks_against_the_window_gate(), window.extra_pinned_blocks_against_the_window_gate()),
        (117, 117, 0), "K13 w = 16 时 x = 117（释放代落在 (0, 14] 里的 9 个窗口 × 13），窗口臂按定义 0"
    );
}

/// 刻度（T₀ = 1）k = 1、2、4 在第 9、17、33 个窗口；RGall 117 / 221 / 429，窗口臂 91 / 91 / 91；s(2→4) RGall = 208 / 208 = 1，窗口臂 = 0。
#[test]
fn marks_at_one_two_and_four_revolutions_give_slope_one_for_rgall_and_zero_for_the_window_arm() {
    for (arm, expected_pinned, expected_increase_two_to_four) in [(WHOLE_RING_ROOTLESS_SKIPPING_ARM, [117u64, 221, 429], 208i64), (CLAUSE_WINDOW_ARM, [91, 91, 91], 0)] {
        let mut simulation = stuck_from_the_first_window(arm, 1, 0.30);
        let mut windows_at_mark: [Option<u64>; 3] = [None; 3];
        for window in 1..=40u64 {
            run_delete_create_windows(&mut simulation, 1);
            for (mark_index, mark) in simulation.metrics.second_rerun.marks.iter().enumerate() {
                if mark.is_some() && windows_at_mark[mark_index].is_none() {
                    windows_at_mark[mark_index] = Some(window);
                }
            }
        }
        assert_eq!(windows_at_mark, [Some(9), Some(17), Some(33)], "{} K13 刻度在第 9、17、33 个窗口", arm.name());
        let marks = simulation.metrics.second_rerun.marks.map(|mark| mark.expect("三个刻度都走到了"));
        assert_eq!(marks.iter().map(|mark| mark.published_txg).collect::<Vec<u64>>(), vec![14, 26, 50], "{} K13 mark N", arm.name());
        assert_eq!(marks.iter().map(|mark| mark.pinned_blocks).collect::<Vec<u64>>(), expected_pinned.to_vec(), "{} K13 刻度上扣住的块数", arm.name());
        let (increase, freed) = stuck_slope_between(&marks[1], &marks[2]).expect("两个刻度之间放掉了块");
        assert_eq!((increase, freed), (expected_increase_two_to_four, 208), "{} s(2→4)：增量 / 两个刻度之间 16 个窗口 × 13 块", arm.name());
    }
}

/// 判别力自证：门槛 1/2 落在两个点（s = 1 与 s = 0）之间；挪到 0 与 1 之外任一侧，两个点由分开变成同判。
#[test]
fn bounded_threshold_one_half_separates_the_two_slopes_and_moving_it_outside_zero_to_one_makes_them_agree() {
    let rgall_slope = (208i64, 208i64);
    let window_slope = (0i64, 208i64);
    let is_below = |numerator: i64, denominator: i64, slope: (i64, i64)| slope.0 * denominator < slope.1 * numerator;
    assert_eq!((is_stuck_slope_bounded(rgall_slope.0, rgall_slope.1), is_stuck_slope_bounded(window_slope.0, window_slope.1)), (false, true), "门槛 1/2：RGall 没有上界、窗口臂有上界");
    assert_eq!((is_below(1, 2, rgall_slope), is_below(1, 2, window_slope)), (false, true), "本测试自己的式子在 1/2 上与装置一致");
    assert_eq!((is_below(0, 1, rgall_slope), is_below(0, 1, window_slope)), (false, false), "门槛挪到 0：两个点同判没有上界");
    assert_eq!((is_below(2, 1, rgall_slope), is_below(2, 1, window_slope)), (true, true), "门槛挪到 2：两个点同判有上界");
    assert_eq!((is_below(1, 1, rgall_slope), is_below(1, 1, window_slope)), (false, true), "门槛恰为 1：仍分开");
    assert!(!is_stuck_slope_bounded(104, 208) && is_stuck_slope_bounded(103, 208), "恰好 1/2 不算有上界，不取整");
}

/// 变异 MS2 的取样点：盘 0 卡住（H3，区域 0 与 2）。
#[test]
fn disk_zero_stuck_rgall_pins_thirteen_blocks_per_window_and_the_window_arm_stays_at_thirty_nine() {
    for (windows, newest_txg, window_gate, rgall_pinned, window_pinned, hole_slots) in [(4u64, 10u64, 0u64, 52u64, 52u64, 0u64), (8, 22, 13, 104, 39, 8), (16, 46, 37, 208, 39, 8)] {
        let mut rgall = stuck_from_the_first_window(WHOLE_RING_ROOTLESS_SKIPPING_ARM, 0, 0.30);
        let mut window = stuck_from_the_first_window(CLAUSE_WINDOW_ARM, 0, 0.30);
        for simulation in [&mut rgall, &mut window] {
            run_delete_create_windows(simulation, windows);
            assert_eq!(simulation.newest_durable_txg, newest_txg, "K14 w = {windows}：N = {newest_txg}");
            assert_eq!(simulation.slots_holding_a_valid_root_outside_the_window(), hole_slots, "K14 w = {windows}：窗口外的槽数");
        }
        assert_eq!((rgall.reuse_bound(), window.reuse_bound()), (0, window_gate), "K14 w = {windows}：门槛");
        assert_eq!((rgall.pinned_count(), window.pinned_count()), (rgall_pinned, window_pinned), "K14 w = {windows}：扣住的块数");
    }
}

/// `Metrics` 里除轨迹与第 2 次重跑的新量之外的全部字段，K15 逐个比。
fn core_metrics_signature(metrics: &Metrics) -> String {
    format!("{:?}", [
        metrics.fake_candidates, metrics.unrecoverable, metrics.retention_violations, metrics.checks, metrics.minimum_candidates, metrics.candidate_sum,
        metrics.nonempty_candidate_sum, metrics.minimum_distinct_states, metrics.distinct_state_sum, metrics.stalls, metrics.forced_publications, metrics.user_windows,
        metrics.admissions_under_pressure, metrics.maximum_publications_per_admission, metrics.false_enospc_raw, metrics.false_enospc_arm, metrics.true_enospc,
        metrics.write_after_delete_failed, metrics.first_write_after_delete_failure_window, metrics.last_write_after_delete_failure_window, metrics.halted_before_counting,
        metrics.root_write_failures_injected, metrics.lag_sum, metrics.lag_maximum, metrics.lag_samples,
    ])
}

/// 检查本身分得出差别：两条确实不同的臂（臂 A 与 G2S）在 steady 里的签名不同。
#[test]
fn signature_tells_two_different_arms_apart() {
    let ring = core_metrics_signature(&run_world(Arm::RingPersisted, World::Steady, 4, 0, 0));
    let states = core_metrics_signature(&run_world(Arm::TightenedStates, World::Steady, 4, 0, 0));
    assert_ne!(ring, states, "臂 A 与 G2S 的 Metrics 字段不同，签名要看得出来");
}

/// K15 等价留档：候选下界取旧式的 RGall_Q 与原 G2S 逐处同式（回收门槛、候选集、上限、处置目标、滞后量都取整环、读法 Q），`Metrics` 除轨迹之外每个字段逐个相等。
#[test]
fn rgall_zero_counting_reading_with_the_old_candidate_floor_agrees_with_the_original_states_arm_in_every_field_except_the_trajectory() {
    let original = Arm::TightenedStates;
    let whole_ring = Arm::WholeRing(WholeRingArm::CountsRootlessDisksAsZeroWithTheOldCandidateFloor);
    for world in WORLDS {
        for slots_per_region in SLOTS_PER_REGION_SWEEP {
            let lost_disks: &[usize] = if world == World::DiskLoss { &[0, 1] } else { &[0] };
            for &lost_disk in lost_disks {
                for seed in 0..8u64 {
                    assert_eq!(
                        core_metrics_signature(&run_world(whole_ring, world, slots_per_region, seed, lost_disk)),
                        core_metrics_signature(&run_world(original, world, slots_per_region, seed, lost_disk)),
                        "K15 {} S={slots_per_region} 丢盘 {lost_disk} 种子 {seed}", world.name()
                    );
                }
            }
        }
    }
    for seed in 0..2u64 {
        assert_eq!(
            core_metrics_signature(&run_near_full(whole_ring, 4, 1, 5, seed, None)), core_metrics_signature(&run_near_full(original, 4, 1, 5, seed, None)),
            "K15 near_full 种子 {seed}"
        );
        assert_eq!(
            core_metrics_signature(&run_broken_slot(whole_ring, 4, 1, 5, seed)), core_metrics_signature(&run_broken_slot(original, 4, 1, 5, seed)),
            "K15 broken_slot 种子 {seed}"
        );
        let whole_ring_prefix = drain_failure_prefix(whole_ring, 4, 1, 5, seed).expect("填满不该卡死");
        let original_prefix = drain_failure_prefix(original, 4, 1, 5, seed).expect("填满不该卡死");
        for pattern in drain_failure_patterns(original.drain_failure_offset_limit(12)) {
            assert_eq!(
                core_metrics_signature(&run_drain_failure_case(&whole_ring_prefix, &pattern)), core_metrics_signature(&run_drain_failure_case(&original_prefix, &pattern)),
                "K15 drain_failure 种子 {seed} 失败 {pattern:?}"
            );
        }
    }
}

/// K16：条款形下界的臂在各个世界里每次 `record_check`，系统配置 F ≥ 根上带的 F（「先写后抬」在装置里的不变量）；条款形候选集从不大于旧候选集的反面（V6）。
#[test]
fn system_configuration_floor_never_falls_below_the_root_floors_and_the_clause_candidate_set_is_never_larger() {
    let clause_arms = [
        CLAUSE_WINDOW_ARM, Arm::ClauseFloorWindowed(WindowArm::DiskHold), Arm::ClauseFloorWindowed(WindowArm::RingLag), CLAUSE_RING_GATE_ARM,
        WHOLE_RING_ROOTLESS_SKIPPING_ARM, WHOLE_RING_ROOTLESS_ZERO_COUNTING_ARM,
    ];
    let mut gap_samples_seen = 0u64;
    for arm in clause_arms {
        for world in WORLDS {
            for slots_per_region in [1u64, 4] {
                for seed in 0..4u64 {
                    let observations = run_world(arm, world, slots_per_region, seed, 0).second_rerun;
                    assert_eq!(observations.system_configuration_below_root_floor_events, 0, "K16 {} {} S={slots_per_region} 种子 {seed}", arm.name(), world.name());
                    assert_eq!(observations.checks_new_set_strictly_larger, 0, "V6 {} {} S={slots_per_region} 种子 {seed}", arm.name(), world.name());
                    gap_samples_seen += observations.candidate_floor_gap.samples;
                }
            }
        }
    }
    assert!(gap_samples_seen > 0, "K16 要真的读到过下界差，否则上面两条什么都没测");
}

/// 第 1 次的 `w_` 行不许被这一次的改动碰：G2SW 在 steady 与 admin_rollback 的 S = 4 两行，照第 1 次产物
/// `research/results/e139-tightened-floor-2026-10-04.out` 第 934、1054 行钉死（旧下界、旧行格式）。
#[test]
fn first_rerun_world_rows_of_the_window_arm_stay_as_in_the_first_product() {
    let arm = Arm::TightenedStatesWindowed(WindowArm::Window);
    for (world, expected_fields) in [
        (World::Steady, "fakes=0 unrecoverable=0 retention_violations=0 checks=9600 min_candidates=12 avg_candidates_x100=1200 avg_nonempty_x100=1200 min_states=2 avg_states_x100=1186 stalls=0 false_enospc_arm=0"),
        (World::AdminRollback, "fakes=0 unrecoverable=0 retention_violations=0 checks=9677 min_candidates=1 avg_candidates_x100=1174 avg_nonempty_x100=1170 min_states=1 avg_states_x100=1156 stalls=0 false_enospc_arm=0"),
    ] {
        let total = fresh_total_over_seeds(SEED_COUNT, |seed| run_world(arm, world, 4, seed, 0));
        assert_eq!(world_outcome_fields(&total), expected_fields, "{} S=4 种子 0..24 与第 1 次产物逐字相同", world.name());
    }
}

/// 路径跑到了（阴性结果要能与「代码没跑到」分开）：`disk_stuck` 的卡住段在低占用上三个刻度都走到、写失败真的注入过；三种收尾都做了一次恢复。
#[test]
fn disk_stuck_segment_reaches_the_three_marks_and_every_ending_runs_one_recovery() {
    let stuck_end = disk_stuck_segment(WHOLE_RING_ROOTLESS_SKIPPING_ARM, 4, 1, 0.30, 0);
    assert!(stuck_end.metrics.second_rerun.marks.iter().all(|mark| mark.is_some()), "占用 0.30、S = 4：三个刻度都走到");
    assert!(stuck_end.metrics.root_write_failures_injected > 0, "卡住的盘上的槽真的被写过、写失败了");
    assert_eq!(stuck_end.stuck_disk, Some(1));
    for ending in STUCK_ENDINGS {
        let metrics = disk_stuck_ending(&stuck_end, ending);
        assert_eq!(metrics.second_rerun.pinned_after_recovery.samples, 1, "{} 收尾恰做一次恢复", ending.name());
    }
}

/// `broken_slot` 与 H1 的刻度起点设在坏槽开始的那一刻：条款形下界的臂走到三个刻度，旧臂不采。
#[test]
fn broken_slot_runs_record_marks_only_for_the_second_rerun_arms() {
    let clause = run_broken_slot(CLAUSE_WINDOW_ARM, 4, 1, 5, 0);
    assert!(clause.second_rerun.marks.iter().all(|mark| mark.is_some()), "坏槽 1000 个窗口里三个刻度都走到");
    let old = run_broken_slot(Arm::TightenedStatesWindowed(WindowArm::Window), 4, 1, 5, 0);
    assert!(old.second_rerun.marks.iter().all(|mark| mark.is_none()), "旧臂不采刻度");
}

/// 判决行的烟测：拿一小撮行簿（steady 与 admin_rollback 两个世界、S = 4、种子 0 与 1，两族每条臂各一行，再加一条臂的两个卡住段刻度）把判决行跑一遍，不 panic。
#[test]
fn the_second_rerun_verdict_lines_run_on_a_handful_of_rows() {
    let first_family = first_rerun_family();
    let second_family = second_rerun_family();
    let mut emitter = Emitter::new();
    let mut first_rows: Vec<WindowRow> = Vec::new();
    let mut rows: Vec<WindowRow> = Vec::new();
    for (family, target_rows) in [(&first_family, &mut first_rows), (&second_family, &mut rows)] {
        for world in [World::Steady, World::AdminRollback] {
            for arm in family.arms.iter().copied() {
                let total = fresh_total_over_seeds(2, |seed| run_world(arm, world, 4, seed, 0));
                emit_window_row(&mut emitter, target_rows, family, WindowRowDraft {
                    line_name: family.lines.world,
                    world_class: world.name(),
                    row_key: format!("world={} s=4 lost_disk=0", world.name()),
                    arm,
                    outcome_fields: world_outcome_fields(&total),
                    judgement: RowJudgement::of(&total, false),
                    is_report_only_world: false,
                    has_false_enospc_field: true,
                    metrics: &total,
                });
            }
        }
    }
    assert_eq!((first_rows.len(), rows.len()), (8, 12), "第 1 次一族 4 条臂 × 2 个世界，第 2 次一族 6 条臂 × 2 个世界");
    let seed_marks: Vec<[Option<StuckMark>; 3]> = (0..2u64).map(|seed| disk_stuck_segment(WHOLE_RING_ROOTLESS_SKIPPING_ARM, 4, 1, 0.30, seed).metrics.second_rerun.marks).collect();
    let growth = vec![emit_stuck_growth_row(&mut emitter, second_family.lines.stuck_growth, WHOLE_RING_ROOTLESS_SKIPPING_ARM, "s=4 stuck_disk=1 fill=0.30", &seed_marks)];
    assert_eq!(growth[0].bound, StuckBound::Unbounded, "RGall 在占用 0.30、盘 1 卡住时 s(2→4) = 1，判没有上界");
    emit_second_rerun_verdict_lines(&mut emitter, &second_family, &first_rows, &[], &rows, &growth, &[]);
}

/// `disk_stuck` 世界全部组合的预演（4 条臂 × S ∈ {4, 16} × 卡住的盘 × 占用两档 × 8 个种子 × 三种收尾）：不 panic、写失败真的注入过。
/// 耗时，在默认那一趟里跑（慢的交统一任务服务，按单元键复用）。
#[test]
fn disk_stuck_world_runs_every_combination_without_panicking() {
    for arm in [WHOLE_RING_ROOTLESS_SKIPPING_ARM, WHOLE_RING_ROOTLESS_ZERO_COUNTING_ARM, CLAUSE_WINDOW_ARM, CLAUSE_RING_GATE_ARM] {
        for slots_per_region in FAULT_SLOTS_SWEEP {
            for stuck_disk in STUCK_DISKS {
                for fill in STUCK_FILLS {
                    for seed in 0..FAULT_SEED_COUNT {
                        let stuck_end = disk_stuck_segment(arm, slots_per_region, stuck_disk, fill, seed);
                        assert!(stuck_end.metrics.root_write_failures_injected > 0, "{} S={slots_per_region} 卡住的盘 {stuck_disk} 占用 {fill:.2} 种子 {seed}：写失败真的注入过", arm.name());
                        for ending in STUCK_ENDINGS {
                            let _ending_metrics = disk_stuck_ending(&stuck_end, ending);
                        }
                    }
                }
            }
        }
    }
}
