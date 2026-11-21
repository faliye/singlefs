//! 按世界、按活化、按近满、按排空失败与坏槽各跑一遍模拟的入口，以及探针与候选账本。

use super::*;

/// 故障落在哪个窗口、盘紧提前几个窗口（E135 跑前补登第 1 条）。
fn schedule(seed: u64, ring_depth: u64, world: World) -> (u64, u64) {
    let mut state = seed.wrapping_mul(0xD1B5_4A32_D192_ED03) | 1;
    state ^= state >> 29;
    let event_window = 2 * ring_depth + 20 + state % 150;
    let lead = match world {
        World::AdminRollback => TXG_ARITHMETIC_GENERATIONS + 2,
        World::TornNewest | World::DiskLoss | World::SlotWriteFail => if seed.is_multiple_of(2) { 0 } else { TXG_ARITHMETIC_GENERATIONS },
        World::Steady | World::RollbackFirstPublish => 0,
    };
    (event_window, lead)
}

pub(super) fn run_world(arm: Arm, world: World, slots_per_region: u64, seed: u64, lost_disk: usize) -> Metrics {
    run_world_with_the_core_gate_switch(arm, world, slots_per_region, seed, lost_disk, true)
}

/// `run_world` 的全形：CG 臂的开关可以关（第 3 次重跑 K-eq ② 用，产物恒开；别的臂没有这个开关）。
pub(super) fn run_world_with_the_core_gate_switch(arm: Arm, world: World, slots_per_region: u64, seed: u64, lost_disk: usize, is_core_gate_switched_on: bool) -> Metrics {
    let keep_snapshots = match world {
        World::TornNewest | World::DiskLoss | World::AdminRollback | World::RollbackFirstPublish => true,
        World::Steady | World::SlotWriteFail => false,
    };
    let (mount_cost, declared_maximum_cost) = COST_PAIRS[0];
    let mut simulation = Simulation::new(arm, slots_per_region, mount_cost, declared_maximum_cost, seed, keep_snapshots, None);
    if let Some(state) = simulation.core_gate.as_mut() {
        state.is_switched_on = is_core_gate_switched_on;
    }
    simulation.prefill(STEADY_FILL);
    let (event_window, lead) = schedule(seed, simulation.ring_depth, world);
    for window in 0..WINDOWS_PER_WORLD {
        if simulation.halted {
            break;
        }
        if world.forces_pressure_before_event() && arm.uses_floor() && window + lead == event_window {
            simulation.forced_pressure();
        }
        if window == event_window {
            match world {
                World::TornNewest => simulation.tear_next_publication = true,
                World::SlotWriteFail => simulation.fail_next_slot_write = true,
                World::AdminRollback => {
                    simulation.admin_rollback();
                }
                World::RollbackFirstPublish => simulation.admin_rollback_then_crash_before_root(),
                World::Steady | World::DiskLoss => {}
            }
        }
        simulation.run_user_window(true, false);
        if world == World::DiskLoss && window == event_window && !simulation.halted {
            simulation.disk_lost[lost_disk] = true;
            simulation.crash_and_recover();
            simulation.halted = true;
        }
    }
    simulation.metrics
}

/// 写失败组合：空集、单个 k、两个 k，k ∈ 0..5，共 16 种。
pub(super) fn failure_patterns() -> Vec<Vec<u64>> {
    let mut patterns = vec![Vec::new()];
    for first in 0..ACTIVATION_FAULT_ATTEMPTS {
        patterns.push(vec![first]);
    }
    for first in 0..ACTIVATION_FAULT_ATTEMPTS {
        for second in (first + 1)..ACTIVATION_FAULT_ATTEMPTS {
            patterns.push(vec![first, second]);
        }
    }
    patterns
}

/// 活化一族世界的前缀：跑到第 E 个窗口；skip_then_raise 先让区域 1 的根槽连续写失败 r 次；然后 F 臂只抬一次 F（不推空）。
pub(super) fn activation_prefix(arm: Arm, slots_per_region: u64, seed: u64, skip_repeats: u64) -> Option<Simulation> {
    let (mount_cost, declared_maximum_cost) = COST_PAIRS[0];
    let mut simulation = Simulation::new(arm, slots_per_region, mount_cost, declared_maximum_cost, seed, true, None);
    simulation.prefill(STEADY_FILL);
    let (event_window, _) = schedule(seed, simulation.ring_depth, World::Steady);
    for _ in 0..event_window {
        simulation.run_user_window(true, false);
    }
    let mut remaining = skip_repeats;
    while remaining > 0 && !simulation.halted {
        if Simulation::region_of_txg(simulation.next_txg) == 1 {
            simulation.fail_next_slot_write = true;
            remaining -= 1;
        }
        simulation.run_user_window(true, false);
    }
    if simulation.halted {
        return None;
    }
    if arm.uses_floor() {
        simulation.raise_floor_to(simulation.floor_upper_limit());
    }
    Some(simulation)
}

pub(super) fn run_activation_case(prefix: &Simulation, pattern: &[u64], crash_window: u64, lost_disk: usize, variant: ActivationVariant) -> Metrics {
    let mut simulation = prefix.clone();
    simulation.metrics = Metrics::fresh();
    let base = simulation.root_write_attempts;
    for offset in pattern {
        simulation.planned_root_write_failures.insert(base + offset);
    }
    for _ in 1..crash_window {
        simulation.run_user_window(true, false);
    }
    let _ = simulation.run_user_window(true, true);
    match variant {
        ActivationVariant::Plain | ActivationVariant::SkipThenRaise { .. } => {}
        ActivationVariant::CrashWithoutLossFirst => {
            if !simulation.halted {
                simulation.crash_and_recover();
            }
            for _ in 0..WINDOWS_BETWEEN_CRASHES {
                simulation.run_user_window(true, false);
            }
            let _ = simulation.run_user_window(true, true);
        }
    }
    if !simulation.halted {
        simulation.disk_lost[lost_disk] = true;
        simulation.crash_and_recover();
    }
    simulation.metrics
}

/// near_full：预热 2N + 10 个窗口，只建不删直到第一次 ENOSPC，再做 400 个「删一个、建一个」。
pub(super) fn run_near_full(arm: Arm, slots_per_region: u64, mount_cost: u64, declared_maximum_cost: u64, seed: u64, reserve_override: Option<u64>) -> Metrics {
    let mut simulation = fill_to_enospc(arm, slots_per_region, mount_cost, declared_maximum_cost, seed, reserve_override);
    for _ in 0..WINDOWS_PER_WORLD {
        if simulation.halted {
            break;
        }
        simulation.run_counted_delete_create_window();
    }
    simulation.metrics
}

/// 近满盘一族的共同前缀：预热、填满到第一次 ENOSPC；计数从填满之后开始。
fn fill_to_enospc(arm: Arm, slots_per_region: u64, mount_cost: u64, declared_maximum_cost: u64, seed: u64, reserve_override: Option<u64>) -> Simulation {
    let mut simulation = Simulation::new(arm, slots_per_region, mount_cost, declared_maximum_cost, seed, false, reserve_override);
    simulation.prefill(STEADY_FILL);
    let warm_up = 2 * simulation.ring_depth + 10;
    for _ in 0..warm_up {
        simulation.run_user_window(true, false);
    }
    let stalls_while_warming = simulation.metrics.stalls;
    simulation.metrics = Metrics::fresh();
    simulation.metrics.stalls = stalls_while_warming;
    for _ in 0..CAPACITY_BLOCKS {
        match simulation.run_user_window(false, false) {
            WindowOutcome::Created => {}
            WindowOutcome::NoSpace | WindowOutcome::Halted | WindowOutcome::AllocatedWithoutPublish => break,
        }
    }
    simulation.metrics.halted_before_counting = u64::from(simulation.halted);
    simulation.counting_window = 0;
    simulation
}

/// cost_growth：near_full 的删建循环做到第 100 个窗口时空发布开销升到 grown_cost，再做 300 个窗口。
pub(super) fn run_cost_growth(arm: Arm, slots_per_region: u64, mount_cost: u64, declared_maximum_cost: u64, seed: u64, grown_cost: u64) -> Metrics {
    let mut simulation = fill_to_enospc(arm, slots_per_region, mount_cost, declared_maximum_cost, seed, None);
    for window in 0..WINDOWS_PER_WORLD {
        if simulation.halted {
            break;
        }
        if window == COST_GROWTH_AT_WINDOW {
            simulation.empty_publication_cost = grown_cost;
        }
        simulation.run_counted_delete_create_window();
    }
    simulation.metrics
}

/// drain_failure 的前缀：填满、再做 5 个删建窗口，计数清零。
pub(super) fn drain_failure_prefix(arm: Arm, slots_per_region: u64, mount_cost: u64, declared_maximum_cost: u64, seed: u64) -> Option<Simulation> {
    let mut simulation = fill_to_enospc(arm, slots_per_region, mount_cost, declared_maximum_cost, seed, None);
    for _ in 0..DRAIN_FAILURE_WINDOWS_BEFORE {
        simulation.run_user_window(true, false);
    }
    if simulation.halted {
        return None;
    }
    simulation.metrics = Metrics::fresh();
    simulation.counting_window = 0;
    Some(simulation)
}

/// drain_failure 的写失败组合：单个、两个（位置 ≤ limit），三个只取前几种。
pub(super) fn drain_failure_patterns(offset_limit: u64) -> Vec<Vec<u64>> {
    let mut patterns = Vec::new();
    for first in 0..=offset_limit {
        patterns.push(vec![first]);
    }
    for first in 0..=offset_limit {
        for second in (first + 1)..=offset_limit {
            patterns.push(vec![first, second]);
        }
    }
    let mut triples = 0usize;
    'outer: for first in 0..=offset_limit {
        for second in (first + 1)..=offset_limit {
            for third in (second + 1)..=offset_limit {
                if triples >= DRAIN_FAILURE_TRIPLE_PATTERNS {
                    break 'outer;
                }
                patterns.push(vec![first, second, third]);
                triples += 1;
            }
        }
    }
    patterns
}

/// drain_failure 的一格：下一次准入处置里第 k 次根槽写失败（k 按组合），之后再做 20 个删建窗口。
pub(super) fn run_drain_failure_case(prefix: &Simulation, pattern: &[u64]) -> Metrics {
    let mut simulation = prefix.clone();
    let base = simulation.root_write_attempts;
    for offset in pattern {
        simulation.planned_root_write_failures.insert(base + offset);
    }
    for _ in 0..DRAIN_FAILURE_WINDOWS_AFTER {
        if simulation.halted {
            break;
        }
        simulation.run_counted_delete_create_window();
    }
    simulation.metrics
}

/// broken_slot：占用 70% 预热 2N + 10 个窗口之后，槽 1 从此每次写都失败；再做 1000 个删建窗口。
pub(super) fn run_broken_slot(arm: Arm, slots_per_region: u64, mount_cost: u64, declared_maximum_cost: u64, seed: u64) -> Metrics {
    let mut simulation = Simulation::new(arm, slots_per_region, mount_cost, declared_maximum_cost, seed, false, None);
    simulation.prefill(STEADY_FILL);
    let warm_up = 2 * simulation.ring_depth + 10;
    for _ in 0..warm_up {
        simulation.run_user_window(true, false);
    }
    simulation.metrics = Metrics::fresh();
    simulation.counting_window = 0;
    simulation.broken_slot = Some(BROKEN_SLOT);
    simulation.stuck_marks_origin_txg = Some(simulation.next_txg);
    for _ in 0..BROKEN_SLOT_WINDOWS {
        if simulation.halted {
            break;
        }
        simulation.run_counted_delete_create_window();
    }
    simulation.metrics
}

/// 判据 1（跨装置闸）：steady 下最后一个窗口，在发布之前与发布刚完成各量一次。
pub(super) fn steady_probe(arm: Arm, slots_per_region: u64) -> (u64, u64, u64, u64) {
    let (mount_cost, declared_maximum_cost) = COST_PAIRS[0];
    let mut simulation = Simulation::new(arm, slots_per_region, mount_cost, declared_maximum_cost, 1, false, None);
    simulation.prefill(STEADY_FILL);
    for _ in 0..(WINDOWS_PER_WORLD - 1) {
        simulation.run_user_window(true, false);
    }
    let outcome = simulation.run_user_window(true, true);
    assert_eq!(outcome, WindowOutcome::AllocatedWithoutPublish, "steady 下不该走盘紧处置");
    let txg = simulation.next_txg;
    let pinned_before = simulation.pinned_count();
    let fake_before = simulation.count_fake_candidates().0;
    simulation.publish(txg);
    let pinned_after = simulation.pinned_count();
    let fake_after = simulation.count_fake_candidates().0;
    (pinned_before, pinned_after, fake_before, fake_after)
}

/// 判据 1（跑前式子）：从 steady 触发一次完整的有界处置，做到目标放回为止。
/// 返回（用了几次发布，含发布正在攒的窗口；之后还扣着几块，不含滞后量；滞后量；目标放回了没有）。
fn push_probe(arm: Arm, slots_per_region: u64, cost: u64, extra_windows: u64) -> (u64, u64, u64, bool) {
    let mut simulation = Simulation::new(arm, slots_per_region, cost, cost, 1, false, None);
    simulation.prefill(STEADY_FILL);
    for _ in 0..(200 + extra_windows) {
        simulation.run_user_window(true, false);
    }
    let open_txg = simulation.next_txg;
    let victim = simulation.pool.objects.swap_remove(0);
    simulation.free_blocks(&victim, open_txg);
    let old_metadata = std::mem::take(&mut simulation.pool.metadata);
    simulation.free_blocks(&old_metadata, open_txg);
    // 这个窗口删了一个对象，发布出来的根改了用户可见状态
    simulation.current_window_changes_user_state = true;
    assert!(simulation.commit_open_window(open_txg), "steady 下发布正在攒的窗口不该卡死");
    let goal_txg = simulation.newest_durable_txg;
    let target = simulation.disposal_target(goal_txg);
    let publications = 1 + simulation.push_after_commit(goal_txg, false);
    let lag = simulation.lag_blocks();
    (publications, simulation.pinned_count() - lag, lag, simulation.is_goal_released(target))
}

/// 预注册判据 1 的期望：(允许的发布数集合, 最大发布数, 最大扣住块数（不含滞后量）, 滞后量)。
pub(super) fn expected_push(arm: Arm, slots_per_region: u64, cost: u64) -> (Vec<u64>, u64, u64, u64) {
    let ring_depth = REGION_COUNT * slots_per_region;
    if slots_per_region == 1 {
        return match arm {
            // 环里只有 3 个根，环一转就把目标放回了；按状态数读时非空有效根不足 4 个，滞后量就是环之外的那些
            Arm::TightenedStates | Arm::TightenedStatesWindowed(_) | Arm::ClauseFloorWindowed(_) | Arm::WholeRing(_) => (vec![3], 3, METADATA_BLOCKS_PER_PUBLICATION + cost, 0),
            Arm::RingPersisted | Arm::FloorPerDisk | Arm::TightenedRoots => (vec![3], 3, METADATA_BLOCKS_PER_PUBLICATION + cost, 0),
            Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRootsAtMount => (Vec::new(), 0, 0, 0),
        };
    }
    match arm {
        Arm::RingPersisted => (vec![ring_depth], ring_depth, METADATA_BLOCKS_PER_PUBLICATION + (ring_depth - 2) * cost, 0),
        Arm::FloorPerDisk | Arm::TightenedRoots => (vec![6, 7], 7, METADATA_BLOCKS_PER_PUBLICATION + 5 * cost, 0),
        // 目标是第 4 新的非空根（3 个窗口之前），发布正在攒的窗口之后立刻抬，带新 F 的第一个根 t + 1，最多再 2 次两块盘都有；
        // 滞后量 = t 与前两个非空窗口各放掉的 8 块对象 + 5 块元数据
        Arm::TightenedStates | Arm::TightenedStatesWindowed(_) | Arm::ClauseFloorWindowed(_) | Arm::WholeRing(_) => {
            (vec![3, 4], 4, METADATA_BLOCKS_PER_PUBLICATION + 2 * cost, 3 * BLOCKS_PER_PUBLICATION)
        }
        Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRootsAtMount => (Vec::new(), 0, 0, 0),
    }
}

/// 三种 t 的落点（t mod 3）各做一次处置探针：(发布数集合, 最大发布数, 最大扣住块数, 最大滞后量, 全部放回)。
pub(super) fn push_probe_over_landings(arm: Arm, slots_per_region: u64, cost: u64) -> (Vec<u64>, u64, u64, u64, bool) {
    let results: Vec<(u64, u64, u64, bool)> = (0..REGION_COUNT).map(|extra| push_probe(arm, slots_per_region, cost, extra)).collect();
    let publication_set: Vec<u64> = results.iter().map(|(publications, _, _, _)| *publications).collect::<BTreeSet<u64>>().into_iter().collect();
    let maximum_publications = results.iter().map(|(publications, _, _, _)| *publications).max().unwrap_or(0);
    let maximum_pinned = results.iter().map(|(_, pinned, _, _)| *pinned).max().unwrap_or(0);
    let maximum_lag = results.iter().map(|(_, _, lag, _)| *lag).max().unwrap_or(0);
    let is_all_released = results.iter().all(|(_, _, _, is_released)| *is_released);
    (publication_set, maximum_publications, maximum_pinned, maximum_lag, is_all_released)
}

pub(super) fn format_set(values: &[u64]) -> String {
    values.iter().map(|value| value.to_string()).collect::<Vec<String>>().join("/")
}

pub(super) fn candidate_index(arm: Arm) -> Option<usize> {
    match arm {
        Arm::TightenedRoots => Some(0),
        Arm::TightenedStates => Some(1),
        Arm::RingPersisted | Arm::ProtectedGenerationsAtMinimum | Arm::FloorTxg | Arm::FloorPerDisk | Arm::FloorPerDiskTxgUpperLimit | Arm::TightenedRootsAtMount | Arm::TightenedStatesWindowed(_)
        | Arm::ClauseFloorWindowed(_) | Arm::WholeRing(_) => None,
    }
}

/// 判据 3–8 按格累计的账。
#[derive(Clone, Copy, Debug)]
pub(super) struct CandidateLedger {
    pub(super) fakes: u64,
    pub(super) unrecoverable: u64,
    pub(super) retention_violations: u64,
    pub(super) false_enospc_cells: u64,
    pub(super) write_after_delete_cells: u64,
    pub(super) bound_exceeded_cells: u64,
    pub(super) stall_cells: u64,
    pub(super) maximum_publications_within_tolerance: u64,
    pub(super) maximum_publications_beyond_tolerance: u64,
}

impl CandidateLedger {
    pub(super) fn fresh() -> Self {
        CandidateLedger {
            fakes: 0,
            unrecoverable: 0,
            retention_violations: 0,
            false_enospc_cells: 0,
            write_after_delete_cells: 0,
            bound_exceeded_cells: 0,
            stall_cells: 0,
            maximum_publications_within_tolerance: 0,
            maximum_publications_beyond_tolerance: 0,
        }
    }
    pub(super) fn add_correctness(&mut self, total: &Metrics) {
        self.fakes += total.fake_candidates;
        self.unrecoverable += total.unrecoverable;
        self.retention_violations += total.retention_violations;
    }
    /// 近满盘一族的一格：判据 4、5、6 按格记。
    pub(super) fn add_space_cell(&mut self, total: &Metrics, bound: u64) {
        self.add_correctness(total);
        self.false_enospc_cells += u64::from(total.false_enospc_arm > 0);
        self.write_after_delete_cells += u64::from(total.write_after_delete_failed > 0);
        self.bound_exceeded_cells += u64::from(total.maximum_publications_per_admission > bound);
        self.stall_cells += u64::from(total.stalls > 0);
    }
}

/// 几何敏感性（跑前登记第八节）：原取样点与方向相反的取样点。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum GeometryPoint {
    /// 原取样点：跳号在区域 1（盘 1，独占一块盘），坏槽在槽 1（区域 0，盘 0）。
    Original,
    /// H1 / H2：跳号换成区域 0（盘 0），坏槽换成区域 1 的第一个槽（槽号 S，盘 1）。
    Reversed,
}

/// H2 的前缀：同 `activation_prefix`，只是跳号落在 `jump_region` 而不是区域 1。
pub(super) fn activation_prefix_with_jump_region(arm: Arm, slots_per_region: u64, seed: u64, skip_repeats: u64, jump_region: u64) -> Option<Simulation> {
    let (mount_cost, declared_maximum_cost) = COST_PAIRS[0];
    let mut simulation = Simulation::new(arm, slots_per_region, mount_cost, declared_maximum_cost, seed, true, None);
    simulation.prefill(STEADY_FILL);
    let (event_window, _) = schedule(seed, simulation.ring_depth, World::Steady);
    for _ in 0..event_window {
        simulation.run_user_window(true, false);
    }
    let mut remaining = skip_repeats;
    while remaining > 0 && !simulation.halted {
        if Simulation::region_of_txg(simulation.next_txg) == jump_region {
            simulation.fail_next_slot_write = true;
            remaining -= 1;
        }
        simulation.run_user_window(true, false);
    }
    if simulation.halted {
        return None;
    }
    if arm.uses_floor() {
        simulation.raise_floor_to(simulation.floor_upper_limit());
    }
    Some(simulation)
}

/// H1：同 `run_broken_slot`，坏槽换成 `broken_slot` 号槽。
pub(super) fn run_broken_slot_at(arm: Arm, slots_per_region: u64, mount_cost: u64, declared_maximum_cost: u64, seed: u64, broken_slot: usize) -> Metrics {
    let mut simulation = Simulation::new(arm, slots_per_region, mount_cost, declared_maximum_cost, seed, false, None);
    simulation.prefill(STEADY_FILL);
    let warm_up = 2 * simulation.ring_depth + 10;
    for _ in 0..warm_up {
        simulation.run_user_window(true, false);
    }
    simulation.metrics = Metrics::fresh();
    simulation.counting_window = 0;
    simulation.broken_slot = Some(broken_slot);
    simulation.stuck_marks_origin_txg = Some(simulation.next_txg);
    for _ in 0..BROKEN_SLOT_WINDOWS {
        if simulation.halted {
            break;
        }
        simulation.run_counted_delete_create_window();
    }
    simulation.metrics
}

#[cfg(test)]
#[path = "world_runs_and_probes_tests.rs"]
mod tests;
