//! 第 3 次重跑（`research/prompts/e139-r3-prereg.md` 第七节）：CG 三处的锚、K-pc、K-eq 与翻面判法。

use super::*;

const CORE_GATE_SKIPPING_ARM: Arm = Arm::WholeRing(WholeRingArm::CoreGateCeilingSkipsRootlessDisks);
const CORE_GATE_ZERO_COUNTING_ARM: Arm = Arm::WholeRing(WholeRingArm::CoreGateCeilingCountsRootlessDisksAsZero);
const WHOLE_RING_SKIPPING_ARM: Arm = Arm::WholeRing(WholeRingArm::SkipsRootlessDisks);
const WHOLE_RING_ZERO_COUNTING_ARM: Arm = Arm::WholeRing(WholeRingArm::CountsRootlessDisksAsZero);
/// 两条 CG 被测臂各与同读法的 RGall 臂成对（锚对两条臂各做一遍）。
const TESTED_PAIRS: [(Arm, Arm); 2] = [(CORE_GATE_SKIPPING_ARM, WHOLE_RING_SKIPPING_ARM), (CORE_GATE_ZERO_COUNTING_ARM, WHOLE_RING_ZERO_COUNTING_ARM)];

fn fresh_simulation(arm: Arm, slots_per_region: u64) -> Simulation {
    let mut simulation = Simulation::new(arm, slots_per_region, 1, 5, 1, true, None);
    simulation.prefill(STEADY_FILL);
    simulation
}

/// 做 `windows` 个删建窗口（`run_user_window(true, false)`，不盘紧），每个都要发布成功。
fn run_publishing_windows(simulation: &mut Simulation, windows: u64) {
    for _ in 0..windows {
        assert_eq!(simulation.run_user_window(true, false), WindowOutcome::Created, "txg {} 的删建窗口要发布成功", simulation.next_txg);
    }
}

/// 下一个窗口的根写之前崩溃（`run_user_window(true, true)`），然后恢复。
fn crash_before_the_next_root_and_recover(simulation: &mut Simulation) {
    assert_eq!(simulation.run_user_window(true, true), WindowOutcome::AllocatedWithoutPublish, "崩溃那个窗口分配了、没发布");
    simulation.crash_and_recover();
    assert!(!simulation.halted, "恢复要择得到根");
}

/// Kcg1：S = 4，txg 1..10；`raise_floor_to(6)`；txg 11 的根写之前崩溃、恢复。
fn kcg1_state(arm: Arm) -> Simulation {
    let mut simulation = fresh_simulation(arm, 4);
    run_publishing_windows(&mut simulation, 10);
    simulation.raise_floor_to(6);
    crash_before_the_next_root_and_recover(&mut simulation);
    simulation
}

/// Kcg2（`clear_system_configuration_floor` 为真时是 Kcg2n）：同 Kcg1，抬 F 之后再发 txg 11（带 6，落在盘 0），txg 12 的根写之前崩溃、恢复。
fn kcg2_state(arm: Arm, clear_system_configuration_floor: bool) -> Simulation {
    let mut simulation = fresh_simulation(arm, 4);
    run_publishing_windows(&mut simulation, 10);
    simulation.raise_floor_to(6);
    run_publishing_windows(&mut simulation, 1);
    if clear_system_configuration_floor {
        simulation.system_configuration_floor = 0;
    }
    crash_before_the_next_root_and_recover(&mut simulation);
    simulation
}

/// Kcg3 / Kcg5 / Kcg5f：做 `windows` 个窗口（可先抬 F），下一次发布撕裂、恢复。
fn torn_state(arm: Arm, slots_per_region: u64, windows: u64, raise_to: Option<u64>) -> Simulation {
    let mut simulation = fresh_simulation(arm, slots_per_region);
    run_publishing_windows(&mut simulation, windows);
    if let Some(target) = raise_to {
        simulation.raise_floor_to(target);
    }
    simulation.tear_next_publication = true;
    let instance_before = simulation.instance;
    simulation.run_user_window(true, false);
    assert_eq!(simulation.instance, instance_before + 1, "撕裂之后恢复过一次");
    simulation
}

fn core_gate_state(simulation: &Simulation) -> &CoreGateState {
    simulation.core_gate.as_ref().expect("CG 臂有 CG 状态")
}

#[test]
fn kc1_core_gate_arms_keep_the_tightened_states_bound_reserve_and_residue() {
    for whole_ring_arm in CORE_GATE_ARMS.iter().chain(CORE_GATE_SHORT_ARMS.iter()) {
        let arm = Arm::WholeRing(*whole_ring_arm);
        assert_eq!(arm.publication_bound(12), 4 + 2 * 2, "{} B = 4 + 2 × k_tol = 8", arm.name());
        assert_eq!((arm.reserve_blocks(12, 5), arm.push_residue_blocks(12, 5)), (10 + 7 * 5, 5 + 6 * 5), "{} c_max = 5：保留池 45、残留 35", arm.name());
        assert_eq!((arm.reserve_blocks(12, 10), arm.push_residue_blocks(12, 10)), (10 + 7 * 10, 5 + 6 * 10), "{} c_max = 10：保留池 80、残留 65", arm.name());
        assert!(arm.uses_the_core_gate() && arm.uses_the_clause_candidate_floor(), "{} 带 CG 与 CF", arm.name());
    }
}

/// Kcg1（锚 `Kcg1 … core_f_eff=6 core_m=9 … core_gate_now=6`）；MG1、MG10、MG12 的取样点。
#[test]
fn kcg1_the_core_gate_takes_the_system_configuration_floor_six_where_the_old_gate_stays_at_zero() {
    for (core_gate_arm, old_arm) in TESTED_PAIRS {
        let mut core_gate = kcg1_state(core_gate_arm);
        let old = kcg1_state(old_arm);
        assert_eq!(core_gate.system_configuration_floor, 6, "{} 系统配置 F 6", core_gate_arm.name());
        assert_eq!((core_gate.floor_carried, core_gate.floor_active, core_gate.reuse_bound()), (6, 6, 6), "{} floor_carried 6、F 那一项 min(6, m = 9) = 6、门槛 6", core_gate_arm.name());
        assert_eq!(core_gate_state(&core_gate).effective_floor, 6, "{} F_生效 6", core_gate_arm.name());
        assert_eq!((old.floor_carried, old.floor_active, old.reuse_bound()), (0, 0, 0), "{} 旧式 floor_carried 0、门槛 0", old_arm.name());
        assert_eq!(core_gate.allocate(8, 11).map(|blocks| blocks.len()), Some(8), "{} 分配一个对象", core_gate_arm.name());
        let observations = core_gate.metrics.core_gate;
        assert_eq!((observations.gate_difference.maximum, observations.gate_difference_nonzero_samples), (6, 1), "{} d = 6 − 0，读一次", core_gate_arm.name());
        assert_eq!(observations.floor_term_difference_nonzero_samples, 1, "{} dF = 6 − 0 ≠ 0", core_gate_arm.name());
    }
}

/// Kc2 / Kcg2（锚 `Kcg2 … old_candidacy=6 old_settled=0 old_gate=0 core_f_eff=6 core_m=10 … core_gate_now=6`）。
#[test]
fn kcg2_the_effective_floor_is_six_from_the_newest_root_on_disk_zero_and_the_old_gate_waits_for_six() {
    for (core_gate_arm, old_arm) in TESTED_PAIRS {
        let core_gate = kcg2_state(core_gate_arm, false);
        let old = kcg2_state(old_arm, false);
        assert_eq!(core_gate_state(&core_gate).effective_floor, 6, "{} Kc2：F_生效 = max(盘 0 的 6, 盘 1 的 0, 系统配置 6) = 6", core_gate_arm.name());
        assert_eq!((core_gate.floor_active, core_gate.reuse_bound()), (6, 6), "{} 当场 min(6, m = 10) = 6、门槛 6", core_gate_arm.name());
        assert_eq!((old.floor_carried, old.floor_active, old.reuse_bound()), (6, 0, 0), "{} 旧式候选下界 6、生效值 0、门槛 0", old_arm.name());
        assert_eq!(old.pending_floor.map(|pending| pending.value), Some(6), "{} 旧式等生效的 F 是 6", old_arm.name());
        let shadow = core_gate_state(&core_gate);
        assert_eq!((shadow.shadow_floor_carried, shadow.shadow_floor_active, shadow.shadow_pending_floor_value), (6, 0, Some(6)), "{} 影子与旧式逐值相同", core_gate_arm.name());
    }
}

/// Kcg2n（锚 `Kcg2n … core_f_eff=6 … core_gate_now=6`）：系统配置 F 清零之后 F_生效 仍取盘 0 最新根带的 6（MG2 的取样点）。
#[test]
fn kcg2n_without_the_system_configuration_floor_the_newest_root_on_disk_zero_still_gives_six() {
    for (core_gate_arm, _) in TESTED_PAIRS {
        let core_gate = kcg2_state(core_gate_arm, true);
        assert_eq!(core_gate_state(&core_gate).effective_floor, 6, "{} F_生效 = max(6, 0, 0) = 6", core_gate_arm.name());
        assert_eq!(core_gate.reuse_bound(), 6, "{} 门槛 6", core_gate_arm.name());
    }
}

/// Kcg3（锚 `Kcg3 … old_gate=15 … pins={8: 2} oldest_valid=15 core_gate_now=2`、`torn_slot = 8 … slot_of_first_txg = 8`）：MG6、MG7、MG8 的取样点。
#[test]
fn kcg3_the_torn_slot_eight_is_pinned_at_txg_two_until_the_first_publication_after_recovery_writes_it() {
    for (core_gate_arm, old_arm) in TESTED_PAIRS {
        let mut core_gate = torn_state(core_gate_arm, 4, 25, None);
        let old = torn_state(old_arm, 4, 25, None);
        assert_eq!(core_gate.slot_index(26), 8, "txg 26 落在槽 8");
        assert_eq!(core_gate.next_txg, 26, "{} 恢复之后第一次发布的 txg 26（撕裂的那次不算发布过）", core_gate_arm.name());
        assert_eq!(core_gate_state(&core_gate).pinned_slots, BTreeMap::from([(8usize, 2u64)]), "{} 钉值 {{槽 8: 2}}", core_gate_arm.name());
        assert_eq!(core_gate.oldest_persisted_valid_txg(), 15, "{} 环里最旧有效根 15", core_gate_arm.name());
        assert_eq!(core_gate.reuse_bound(), 2, "{} CG 门槛 = max(min(0, 24), min(15, 2)) = 2", core_gate_arm.name());
        assert_eq!(old.reuse_bound(), 15, "{} 旧式门槛 15", old_arm.name());
        run_publishing_windows(&mut core_gate, 1);
        assert!(core_gate_state(&core_gate).pinned_slots.is_empty(), "{} txg 26 写进槽 8 之后解钉", core_gate_arm.name());
        assert_eq!(core_gate.reuse_bound(), 15, "{} 解钉之后门槛回到 15", core_gate_arm.name());
    }
}

/// Kcg4（锚 `Kcg4 … core_f_eff=10 core_m=9 … core_gate_now=9 core_gate_after_held=10`、`held_released_after_txg = 13`）：MG3、MG4 的取样点。
#[test]
fn kcg4_the_floor_term_waits_at_nine_until_txg_thirteen_roots_the_new_instance_on_both_disks_then_takes_ten() {
    for (core_gate_arm, old_arm) in TESTED_PAIRS {
        let build = |arm: Arm| {
            let mut simulation = fresh_simulation(arm, 4);
            run_publishing_windows(&mut simulation, 10);
            simulation.raise_floor_to(10);
            crash_before_the_next_root_and_recover(&mut simulation);
            simulation
        };
        let mut core_gate = build(core_gate_arm);
        let old = build(old_arm);
        assert_eq!(core_gate_state(&core_gate).effective_floor, 10, "{} F_生效 10", core_gate_arm.name());
        assert_eq!((core_gate.floor_active, core_gate.reuse_bound()), (9, 9), "{} 当场 min(10, m = 9) = 9", core_gate_arm.name());
        assert_eq!(old.reuse_bound(), 0, "{} 旧式门槛 0", old_arm.name());
        for (published_txg, expected_floor_term) in [(11u64, 9u64), (12, 9), (13, 10)] {
            run_publishing_windows(&mut core_gate, 1);
            assert_eq!(core_gate.newest_durable_txg, published_txg, "{} 发到 txg {published_txg}", core_gate_arm.name());
            assert_eq!(core_gate.floor_active, expected_floor_term, "{} txg {published_txg} 之后 F 那一项 {expected_floor_term}", core_gate_arm.name());
        }
        assert_eq!(core_gate.reuse_bound(), 10, "{} 放开之后门槛 = max(10, 环里最旧有效根 2) = 10", core_gate_arm.name());
        assert_eq!(core_gate_state(&core_gate).held_until_rooted_from_instance, None, "{} 扣住那一截放开了", core_gate_arm.name());
    }
}

/// Kcg5（锚 `Kcg5 S=1 … old_gate=29 core_f_eff=0 core_m=None pins={1: 1} oldest_valid=29 core_gate_now=1`）：MG6 的第二个取样点。
#[test]
fn kcg5_with_one_slot_per_region_the_torn_slot_is_the_only_slot_of_disk_one_and_pins_the_gate_at_one() {
    for (core_gate_arm, old_arm) in TESTED_PAIRS {
        let core_gate = torn_state(core_gate_arm, 1, 30, None);
        let old = torn_state(old_arm, 1, 30, None);
        assert_eq!((core_gate.slot_index(31), core_gate.disk_of_slot(1)), (1, 1), "撕裂的是盘 1 唯一的槽 1");
        assert_eq!(core_gate.newest_valid_txg_on_disk(1), None, "{} 盘 1 没有有效根：m 取不到", core_gate_arm.name());
        assert_eq!(core_gate_state(&core_gate).pinned_slots, BTreeMap::from([(1usize, 1u64)]), "{} 钉值 {{槽 1: 1}}", core_gate_arm.name());
        assert_eq!((core_gate.floor_active, core_gate.reuse_bound()), (0, 1), "{} F 那一项 0、门槛 1", core_gate_arm.name());
        assert_eq!(old.reuse_bound(), 29, "{} 旧式门槛 29", old_arm.name());
    }
}

/// Kcg5f（锚 `Kcg5f S=1 … core_f_eff=20 core_m=None … core_gate_now=1 core_gate_after_held=20`）：MG5 的取样点。
#[test]
fn kcg5f_without_a_root_on_every_disk_the_floor_term_is_zero_even_when_the_system_configuration_floor_is_twenty() {
    for (core_gate_arm, old_arm) in TESTED_PAIRS {
        let core_gate = torn_state(core_gate_arm, 1, 30, Some(20));
        let old = torn_state(old_arm, 1, 30, Some(20));
        assert_eq!(core_gate_state(&core_gate).effective_floor, 20, "{} F_生效 = 系统配置 20", core_gate_arm.name());
        assert_eq!((core_gate.floor_active, core_gate.reuse_bound()), (0, 1), "{} m 取不到：F 那一项 0、门槛 1", core_gate_arm.name());
        assert_eq!(core_gate_state(&core_gate).held_until_rooted_from_instance, Some(core_gate.instance), "{} 在等本实例落满两块盘再升到 20", core_gate_arm.name());
        assert_eq!(old.reuse_bound(), 29, "{} 旧式门槛 29", old_arm.name());
    }
}

/// Kcg6（第 2 次 K10 的构造，之后管理员回退）：CG-R 一样都不动（MG9 的取样点）。
#[test]
fn kcg6_admin_rollback_leaves_every_core_gate_floor_and_pin_as_it_was() {
    for (core_gate_arm, _) in TESTED_PAIRS {
        let mut simulation = fresh_simulation(core_gate_arm, 4);
        run_publishing_windows(&mut simulation, 10);
        simulation.raise_floor_to(6);
        simulation.tear_next_publication = true;
        simulation.run_user_window(true, false);
        simulation.damaged_txgs = HashSet::from([2u64, 3]);
        let fields_of = |simulation_to_read: &Simulation| {
            let state = core_gate_state(simulation_to_read);
            (simulation_to_read.floor_carried, simulation_to_read.floor_active, simulation_to_read.pending_floor.map(|pending| pending.value), state.effective_floor,
             state.held_until_rooted_from_instance, state.pinned_slots.clone())
        };
        let before = fields_of(&simulation);
        assert_eq!((before.0, before.1, before.3), (6, 6, 6), "{} 恢复之后 F_生效 6、当场 min(6, 9) = 6", core_gate_arm.name());
        assert!(simulation.admin_rollback(), "{} 回退要成功", core_gate_arm.name());
        assert_eq!(simulation.newest_durable_txg, 6, "{} 回到条款形下界选的目标 6", core_gate_arm.name());
        assert_eq!(fields_of(&simulation), before, "{} 回退前后 floor_carried、F 那一项、等生效、扣住、钉槽逐字段相同", core_gate_arm.name());
        let shadow = core_gate_state(&simulation);
        assert_eq!((shadow.shadow_floor_carried, shadow.shadow_floor_active), (0, 0), "{} 影子照旧式在回退处重算：根上带的 F 都是 0", core_gate_arm.name());
    }
}

/// K-pc：Kcg1 的状态上分配一个对象，旧候选下界 0 数得出假候选、CF 下界 6 数不出（CG-F 把门槛抬到 6 的检测器）。
#[test]
fn positive_control_after_the_core_gate_reuses_up_to_six_the_old_candidate_floor_sees_fake_candidates_and_the_clause_floor_does_not() {
    for (core_gate_arm, _) in TESTED_PAIRS {
        let mut simulation = kcg1_state(core_gate_arm);
        assert!(simulation.allocate(8, 11).is_some(), "{} 分配一个对象", core_gate_arm.name());
        let (old_floor_fakes, _, _, _) = simulation.count_fake_candidates_at_floor(simulation.candidacy_floor());
        let (clause_floor_fakes, _, _, _) = simulation.count_fake_candidates_at_floor(simulation.candidate_floor());
        assert_eq!((simulation.candidacy_floor(), simulation.candidate_floor()), (0, 6), "{} 旧下界 0、CF 6", core_gate_arm.name());
        assert!(old_floor_fakes > 0, "{} 旧下界下假候选 > 0（复用的块被 txg < 6 的根引用）", core_gate_arm.name());
        assert_eq!(clause_floor_fakes, 0, "{} CF 下界下假候选 0", core_gate_arm.name());
    }
}

/// `Metrics` 除第 3 次新加的观测之外的全部字段（含轨迹与第 2 次的新量），K-eq 逐个比。
fn metrics_signature_without_the_core_gate_observations(metrics: &Metrics) -> String {
    let mut without = *metrics;
    without.core_gate = CoreGateObservations::fresh();
    format!("{without:?}")
}

/// 检查本身分得出差别：Kcg3 的构造上（撕裂之后钉值 2 压过环里最旧有效根 15）再发一个窗口，CG 臂与 RGall 的签名不同。
/// （`run_world` 的 torn_newest、S = 4 不行：撕裂之前盘紧把 F 抬过了钉值，F 那一项盖住了钉，门槛不变，签名逐字段相同。）
#[test]
fn core_gate_equality_the_signature_tells_the_core_gate_apart_where_it_takes_a_value() {
    let signature_after_one_more_window = |arm: Arm| {
        let mut simulation = torn_state(arm, 4, 25, None);
        run_publishing_windows(&mut simulation, 1);
        metrics_signature_without_the_core_gate_observations(&simulation.metrics)
    };
    assert_ne!(signature_after_one_more_window(CORE_GATE_SKIPPING_ARM), signature_after_one_more_window(WHOLE_RING_SKIPPING_ARM), "CG-P 把门槛压到 2，签名要看得出来");
}

/// K-eq ①：没有恢复、没有回退的世界里 CG 三处一处都不取值，CG 臂与同读法的 RGall 臂逐字段相同。
#[test]
fn core_gate_equality_without_recovery_or_rollback_the_core_gate_arms_agree_with_the_rgall_arms_in_every_field() {
    for (core_gate_arm, old_arm) in TESTED_PAIRS {
        for world in [World::Steady, World::SlotWriteFail] {
            for slots_per_region in SLOTS_PER_REGION_SWEEP {
                for seed in 0..2u64 {
                    assert_eq!(
                        metrics_signature_without_the_core_gate_observations(&run_world(core_gate_arm, world, slots_per_region, seed, 0)),
                        metrics_signature_without_the_core_gate_observations(&run_world(old_arm, world, slots_per_region, seed, 0)),
                        "K-eq {} {} S={slots_per_region} 种子 {seed}", core_gate_arm.name(), world.name()
                    );
                }
            }
        }
        for seed in 0..2u64 {
            let same = |core_gate: Metrics, old: Metrics, what: &str| assert_eq!(
                metrics_signature_without_the_core_gate_observations(&core_gate), metrics_signature_without_the_core_gate_observations(&old),
                "K-eq {} {what} 种子 {seed}", core_gate_arm.name()
            );
            same(run_near_full(core_gate_arm, 4, 1, 5, seed, None), run_near_full(old_arm, 4, 1, 5, seed, None), "near_full");
            same(run_broken_slot(core_gate_arm, 4, 1, 5, seed), run_broken_slot(old_arm, 4, 1, 5, seed), "broken_slot");
            same(run_cost_growth(core_gate_arm, 4, 1, 5, seed, 5), run_cost_growth(old_arm, 4, 1, 5, seed, 5), "cost_growth");
            let core_gate_prefix = drain_failure_prefix(core_gate_arm, 4, 1, 5, seed).expect("填满不该卡死");
            let old_prefix = drain_failure_prefix(old_arm, 4, 1, 5, seed).expect("填满不该卡死");
            for pattern in drain_failure_patterns(old_arm.drain_failure_offset_limit(12)).iter().take(3) {
                same(run_drain_failure_case(&core_gate_prefix, pattern), run_drain_failure_case(&old_prefix, pattern), "drain_failure");
            }
        }
    }
}

/// K-eq ② 与 V6：CG 开关关着的 CG 臂在六个基本世界里与同读法的 RGall 臂逐字段相同，旧式门槛的影子与真值逐次相同（d、dF 恒 0，且真的读过）。
#[test]
fn core_gate_equality_with_the_core_gate_switched_off_the_arm_and_its_shadow_agree_with_the_rgall_arm() {
    let mut shadow_samples = 0u64;
    for (core_gate_arm, old_arm) in TESTED_PAIRS {
        for world in WORLDS {
            for slots_per_region in SLOTS_PER_REGION_SWEEP {
                let lost_disks: &[usize] = if world == World::DiskLoss { &[0, 1] } else { &[0] };
                for &lost_disk in lost_disks {
                    for seed in 0..2u64 {
                        let switched_off = run_world_with_the_core_gate_switch(core_gate_arm, world, slots_per_region, seed, lost_disk, false);
                        assert_eq!(
                            metrics_signature_without_the_core_gate_observations(&switched_off),
                            metrics_signature_without_the_core_gate_observations(&run_world(old_arm, world, slots_per_region, seed, lost_disk)),
                            "K-eq ② {} {} S={slots_per_region} 丢盘 {lost_disk} 种子 {seed}", core_gate_arm.name(), world.name()
                        );
                        let observations = switched_off.core_gate;
                        assert_eq!(
                            (observations.gate_difference_nonzero_samples, observations.floor_term_difference_nonzero_samples, observations.pin_effective_samples),
                            (0, 0, 0), "V6 {} {} S={slots_per_region} 丢盘 {lost_disk} 种子 {seed}：开关关着时影子等于真值", core_gate_arm.name(), world.name()
                        );
                        shadow_samples += observations.gate_difference.samples;
                    }
                }
            }
        }
    }
    assert!(shadow_samples > 0, "V6 要真的读过影子，否则上面那条什么都没测");
}

fn cell_line(line_name: &str, arm_name: &str, fakes: u64) -> String {
    format!("E7RESULT name={line_name} arm={arm_name} world=torn_newest s=4 lost_disk=0 fakes={fakes} unrecoverable=0 retention_violations=0 checks=10 stalls=0 false_enospc_arm=0")
}

/// 第八节判别力自证：只有一对、一位从 0 到 1 的假行簿判乙（翻面 1），把那一位改回 0 判甲（翻面 0）；反过来从 1 到 0 只记反向、不记翻面（MG11 的取样点）。
#[test]
fn flip_verdict_turns_on_a_single_pair() {
    let flipped = compare_core_gate_lines(&[cell_line("f_cell", "RGall_ring_all", 0)], &[cell_line("core_gate_cell", "RGall_core_gate", 1)]);
    let tally = flipped.by_arm.get("RGall_ring_all").copied().unwrap_or_default();
    assert_eq!((tally.pairs, tally.flips, tally.pairs_with_a_flip, tally.reverse, tally.pairs_with_a_differing_field), (1, 1, 1, 0, 1), "一位 0 → 1：判乙");
    assert_eq!((flipped.old_lines_without_partner, flipped.core_gate_lines_without_partner, flipped.duplicate_keys), (0, 0, 0), "同键配上");
    let unchanged = compare_core_gate_lines(&[cell_line("f_cell", "RGall_ring_all", 0)], &[cell_line("core_gate_cell", "RGall_core_gate", 0)]);
    let unchanged_tally = unchanged.by_arm.get("RGall_ring_all").copied().unwrap_or_default();
    assert_eq!((unchanged_tally.pairs, unchanged_tally.flips, unchanged_tally.pairs_with_a_differing_field), (1, 0, 0), "改回 0：判甲");
    let reversed = compare_core_gate_lines(&[cell_line("f_cell", "RGall_ring_all", 1)], &[cell_line("core_gate_cell", "RGall_core_gate", 0)]);
    let reversed_tally = reversed.by_arm.get("RGall_ring_all").copied().unwrap_or_default();
    assert_eq!((reversed_tally.flips, reversed_tally.reverse), (0, 1), "1 → 0 只记反向");
    let report_only = compare_core_gate_lines(
        &[cell_line("f_cell", "RGall_ring_all", 0).replace("torn_newest", "rollback_first_publish")],
        &[cell_line("core_gate_cell", "RGall_core_gate", 1).replace("torn_newest", "rollback_first_publish")],
    );
    assert_eq!(report_only.by_arm.get("RGall_ring_all").copied().unwrap_or_default().flips, 0, "rollback_first_publish 只报、不判");
}
