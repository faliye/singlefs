//! E139：按盘回退下界的收严形态 —— D16（发布语义） 已定项 1 判决的重跑。
//!
//! 预注册 `research/prompts/e139-preregistration.md`。装置从 E138（按盘回退下界与推空的空间要求） 的源码拷出来改：
//! ① 两条被测臂 G2R / G2S：准入按最坏残留收费、界与保留池给 k_tol = 2 次根槽写失败留余量、
//!    保留池与残留按声明的最坏空发布开销 c_max 算、G2S 的上限按「4」数不同用户可见状态的口径取第 4 新的非空根；
//! ② 新增五个世界：活化区间内崩溃不丢盘、先跳号再抬 F、推空期间根槽写失败、坏槽、空发布开销在挂载之后变大。

use e7_index_bench::Emitter;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
use std::rc::Rc;

mod arms_and_arm_readings;
mod core_gate_and_second_rerun_observations;
mod core_gate_comparison;
mod metrics_and_simulation;
mod row_comparison_and_verdicts;
mod second_rerun_arms;
mod stuck_growth_rows;
mod window_row_families_and_emission;
mod window_rows_per_world;
mod world_runs_and_probes;
mod worlds_and_pool_state;

use self::arms_and_arm_readings::*;
use self::core_gate_and_second_rerun_observations::*;
use self::core_gate_comparison::*;
use self::metrics_and_simulation::*;
use self::row_comparison_and_verdicts::*;
use self::second_rerun_arms::*;
use self::stuck_growth_rows::*;
use self::window_row_families_and_emission::*;
use self::window_rows_per_world::*;
use self::world_runs_and_probes::*;
use self::worlds_and_pool_state::*;

/// R = 3（D22（单元原子性怎么合成） 已定项 2）。
const REGION_COUNT: u64 = 3;
const DISK_COUNT: usize = 2;
/// 臂 C、臂 D 的 K_min：C222 按轮转算术给的 3。
const TXG_ARITHMETIC_GENERATIONS: u64 = 3;
/// 盘紧时最少保留 4 个；「4」数的是不同的用户可见状态（臂 G2S）。
const MINIMUM_RETAINED_ROOTS: usize = 4;
/// 对照臂 G_txgcap：抬 F 的上限按 txg 数几代。
const TXG_UPPER_LIMIT_GENERATIONS: u64 = 4;
const USER_BLOCKS_PER_OBJECT: usize = 8;
const USER_BLOCKS: u64 = USER_BLOCKS_PER_OBJECT as u64;
const METADATA_BLOCKS_PER_PUBLICATION: u64 = 5;
const BLOCKS_PER_PUBLICATION: u64 = USER_BLOCKS + METADATA_BLOCKS_PER_PUBLICATION;
/// 臂 C、臂 D 沿用 E135（动态回退下界） 的保留池。
const CONTROL_ARM_RESERVE_BLOCKS: u64 = BLOCKS_PER_PUBLICATION;
/// E138 臂 G 的界：发布正在攒的窗口 + 至多 6 次空发布。
const FLOOR_PER_DISK_PUBLICATION_BOUND: u64 = 7;
/// 预注册：一次处置里容忍几次根槽写失败，每次失败最坏让盘 1 的槽轮空一次、界加 2。
const TOLERATED_ROOT_WRITE_FAILURES: u64 = 2;
const PUBLICATIONS_PER_TOLERATED_FAILURE: u64 = 2;
/// 预注册的无故障界：按根数读 7（E138 已钉），按状态数读 4。
const TIGHTENED_ROOTS_BASE_BOUND: u64 = 7;
const TIGHTENED_STATES_BASE_BOUND: u64 = 4;
const TIGHTENED_ROOTS_PUBLICATION_BOUND: u64 = TIGHTENED_ROOTS_BASE_BOUND + PUBLICATIONS_PER_TOLERATED_FAILURE * TOLERATED_ROOT_WRITE_FAILURES;
const TIGHTENED_STATES_PUBLICATION_BOUND: u64 = TIGHTENED_STATES_BASE_BOUND + PUBLICATIONS_PER_TOLERATED_FAILURE * TOLERATED_ROOT_WRITE_FAILURES;
const CAPACITY_BLOCKS: usize = 4000;
const STEADY_FILL: f64 = 0.70;
const WINDOWS_PER_WORLD: u64 = 400;
const SEED_COUNT: u64 = 24;
const FAULT_SEED_COUNT: u64 = 8;
const RESERVE_SWEEP_SEED_COUNT: u64 = 8;
const SLOTS_PER_REGION_SWEEP: [u64; 3] = [1, 4, 16];
const FAULT_SLOTS_SWEEP: [u64; 2] = [4, 16];
/// (挂载时的空发布开销 c₀, 声明的最坏开销 c_max)。
const COST_PAIRS: [(u64, u64); 2] = [(1, 5), (5, 10)];
/// 第 k 次根槽写失败，k ∈ 0..5。
const ACTIVATION_FAULT_ATTEMPTS: u64 = 5;
/// 第 j 个窗口分配完、根槽写之前崩溃，j ∈ 1..=8。
const ACTIVATION_FAULT_CRASH_WINDOWS: u64 = 8;
/// activation_crash_no_loss：第一次崩溃恢复之后再跑几个窗口才第二次崩溃并丢盘。
const WINDOWS_BETWEEN_CRASHES: u64 = 2;
/// skip_then_raise：抬 F 之前先让区域 1 的根槽连续写失败几次。
const SKIP_THEN_RAISE_REPEATS: [u64; 3] = [1, 2, 3];
/// drain_failure：3 次失败的组合只取前几种；臂 A 的失败位置只扫到第几次根槽写。
const DRAIN_FAILURE_TRIPLE_PATTERNS: usize = 20;
const DRAIN_FAILURE_RING_ARM_OFFSET_LIMIT: u64 = 12;
const DRAIN_FAILURE_WINDOWS_BEFORE: u64 = 5;
const DRAIN_FAILURE_WINDOWS_AFTER: u64 = 20;
/// broken_slot：槽 1（区域 0，盘 0）从此每次写都失败。
const BROKEN_SLOT: usize = 1;
const BROKEN_SLOT_WINDOWS: u64 = 1000;
/// cost_growth：近满盘删建循环做到第几个窗口时开销变大；超过声明上界那一格多几块。
const COST_GROWTH_AT_WINDOW: u64 = 100;
const COST_OVER_DECLARED_EXTRA: u64 = 3;
const RESERVE_SWEEP_BELOW: u64 = 6;
const RESERVE_SWEEP_ABOVE: u64 = 8;
const RESERVE_SWEEP_SLOTS_PER_REGION: u64 = 16;
/// 快照留多少份：要大于最大的根环深度 48。
const SNAPSHOTS_KEPT: usize = 64;
const BLOCK_BYTES: u64 = 16384;
const DETAIL_LINES_MAXIMUM: u64 = 24;
/// 第 2 次重跑（`research/prompts/e139-r2-prereg.md` 第五节「`disk_stuck`」）：占用两档、卡住的盘两块（盘 1 是原取样点，盘 0 是 H3）。
const STUCK_FILLS: [f64; 2] = [0.30, 0.70];
const STUCK_DISKS: [usize; 2] = [1, 0];
/// 刻度：从卡住那一刻起第 1、2、4 圈（一圈 = 3S 个 txg）。
const STUCK_MARK_REVOLUTIONS: [u64; 3] = [1, 2, 4];
/// 卡住段最多做几圈个删建窗口（一圈 3S 个窗口）；收尾再做几圈。
const STUCK_WINDOW_REVOLUTION_LIMIT: u64 = 16;
const STUCK_TAIL_REVOLUTIONS: u64 = 2;
/// `disk_stuck` 的快照上限：卡住段最多 16 × 3S = 768 个窗口加预热 106 个，留 1024 份，卡住那块盘上的旧根在恢复时仍读得到它的快照。
const STUCK_SNAPSHOTS_KEPT: usize = 1024;
/// 「有上界」的门槛（登记第一节读法第 10 条 ⓒ）：s(2→4) 小于 1/2。写成 2 × 扣住的增量 < 放掉的块数，不取整。
const STUCK_BOUNDED_SLOPE_DENOMINATOR: i64 = 2;

fn main() {
    let mut emitter = Emitter::new();
    println!("{}", emitter.emit_raw(&format!(
        "name=config r={REGION_COUNT} disks={DISK_COUNT} k_min_txg={TXG_ARITHMETIC_GENERATIONS} retained={MINIMUM_RETAINED_ROOTS} \
         tolerated_failures={TOLERATED_ROOT_WRITE_FAILURES} bound_g={FLOOR_PER_DISK_PUBLICATION_BOUND} bound_g2r={TIGHTENED_ROOTS_PUBLICATION_BOUND} \
         bound_g2s={TIGHTENED_STATES_PUBLICATION_BOUND} user_blocks={USER_BLOCKS} metadata_blocks={METADATA_BLOCKS_PER_PUBLICATION} \
         capacity={CAPACITY_BLOCKS} windows={WINDOWS_PER_WORLD} seeds={SEED_COUNT} fault_seeds={FAULT_SEED_COUNT} sweep_seeds={RESERVE_SWEEP_SEED_COUNT} \
         cost_pairs={} block_bytes={BLOCK_BYTES}",
        COST_PAIRS.iter().map(|(mount, declared)| format!("{mount}:{declared}")).collect::<Vec<String>>().join("/")
    )));

    // 判据 1：跨装置闸与跑前式子
    let mut criterion_one_holds = true;
    for slots_per_region in SLOTS_PER_REGION_SWEEP {
        let ring_depth = REGION_COUNT * slots_per_region;
        let (pinned_before, pinned_after, _, _) = steady_probe(Arm::RingPersisted, slots_per_region);
        let expect_before = ring_depth * BLOCKS_PER_PUBLICATION;
        let expect_after = (ring_depth - 1) * BLOCKS_PER_PUBLICATION;
        criterion_one_holds &= pinned_before == expect_before && pinned_after == expect_after;
        println!("{}", emitter.emit_raw(&format!(
            "name=crit1_pinned arm=A s={slots_per_region} pinned_before={pinned_before} expect_before={expect_before} \
             pinned_after={pinned_after} expect_after={expect_after}"
        )));
        let (_, _, fake_before, fake_after) = steady_probe(Arm::ProtectedGenerationsAtMinimum, slots_per_region);
        let expect_fake_before = ring_depth - TXG_ARITHMETIC_GENERATIONS;
        let expect_fake_after = ring_depth.saturating_sub(TXG_ARITHMETIC_GENERATIONS + 1);
        criterion_one_holds &= fake_before == expect_fake_before && fake_after == expect_fake_after;
        println!("{}", emitter.emit_raw(&format!(
            "name=crit1_fake arm=C s={slots_per_region} fake_before={fake_before} expect_before={expect_fake_before} \
             fake_after={fake_after} expect_after={expect_fake_after}"
        )));
    }
    for arm in [Arm::RingPersisted, Arm::FloorPerDisk, Arm::TightenedRoots, Arm::TightenedStates] {
        for slots_per_region in SLOTS_PER_REGION_SWEEP {
            for (cost, _) in COST_PAIRS {
                let (publication_set, maximum_publications, maximum_pinned, maximum_lag, is_all_released) = push_probe_over_landings(arm, slots_per_region, cost);
                let (expect_set, expect_maximum, expect_pinned, expect_lag) = expected_push(arm, slots_per_region, cost);
                let holds = is_all_released && maximum_publications == expect_maximum && maximum_pinned == expect_pinned && maximum_lag == expect_lag
                    && publication_set.iter().all(|publications| expect_set.contains(publications));
                criterion_one_holds &= holds;
                println!("{}", emitter.emit_raw(&format!(
                    "name=crit1_push arm={} s={slots_per_region} empty_cost={cost} publications={} max_publications={maximum_publications} \
                     expect_publications={} expect_max={expect_maximum} max_pinned={maximum_pinned} expect_pinned={expect_pinned} \
                     lag={maximum_lag} expect_lag={expect_lag} released={} holds={}",
                    arm.name(), format_set(&publication_set), format_set(&expect_set), u8::from(is_all_released), u8::from(holds)
                )));
            }
        }
    }

    let mut ledgers = [CandidateLedger::fresh(); 2];

    // 故障世界（E138 原样）
    let mut first_publish_unrecoverable = [0u64; 8];
    for world in WORLDS {
        for slots_per_region in SLOTS_PER_REGION_SWEEP {
            for (arm_index, arm) in ARMS.iter().enumerate() {
                let lost_disks: &[usize] = if world == World::DiskLoss { &[0, 1] } else { &[0] };
                for &lost_disk in lost_disks {
                    let mut total = Metrics::fresh();
                    for seed in 0..SEED_COUNT {
                        total.add(&run_world(*arm, world, slots_per_region, seed, lost_disk));
                    }
                    println!("{}", emitter.emit_raw(&format!(
                        "name=cell arm={} world={} s={slots_per_region} lost_disk={lost_disk} fakes={} unrecoverable={} retention_violations={} \
                         checks={} min_candidates={} avg_candidates_x100={} avg_nonempty_x100={} min_states={} avg_states_x100={} stalls={}",
                        arm.name(), world.name(), total.fake_candidates, total.unrecoverable, total.retention_violations, total.checks,
                        total.minimum_candidates_or_zero(), total.average_candidates_in_hundredths(),
                        total.average_nonempty_candidates_in_hundredths(), total.minimum_distinct_states_or_zero(),
                        total.average_distinct_states_in_hundredths(), total.stalls
                    )));
                    if world == World::RollbackFirstPublish {
                        first_publish_unrecoverable[arm_index] += total.unrecoverable;
                        continue;
                    }
                    if let Some(index) = candidate_index(*arm) {
                        ledgers[index].add_correctness(&total);
                    }
                }
            }
        }
    }

    // 活化一族：activation_faults（E138 原样）、activation_crash_no_loss、skip_then_raise
    let patterns = failure_patterns();
    let mut floor_txg_activation_fakes = 0u64;
    let mut txg_upper_limit_skip_retention = 0u64;
    let mut detail_lines = 0u64;
    let mut variants = vec![ActivationVariant::Plain, ActivationVariant::CrashWithoutLossFirst];
    for repeats in SKIP_THEN_RAISE_REPEATS {
        variants.push(ActivationVariant::SkipThenRaise { repeats });
    }
    for variant in variants {
        let skip_repeats = match variant {
            ActivationVariant::SkipThenRaise { repeats } => repeats,
            ActivationVariant::Plain | ActivationVariant::CrashWithoutLossFirst => 0,
        };
        for slots_per_region in FAULT_SLOTS_SWEEP {
            for arm in ARMS {
                let mut by_class = [[Metrics::fresh(); 3]; DISK_COUNT];
                for seed in 0..FAULT_SEED_COUNT {
                    let prefix = match activation_prefix(arm, slots_per_region, seed, skip_repeats) {
                        None => continue,
                        Some(simulation) => simulation,
                    };
                    for pattern in &patterns {
                        for crash_window in 1..=ACTIVATION_FAULT_CRASH_WINDOWS {
                            for (lost_disk, class_row) in by_class.iter_mut().enumerate().take(DISK_COUNT) {
                                let case = run_activation_case(&prefix, pattern, crash_window, lost_disk, variant);
                                class_row[pattern.len()].add(&case);
                                if arm.is_candidate() && (case.fake_candidates > 0 || case.unrecoverable > 0 || case.retention_violations > 0) && detail_lines < DETAIL_LINES_MAXIMUM {
                                    detail_lines += 1;
                                    println!("{}", emitter.emit_raw(&format!(
                                        "name=detail_activation variant={} arm={} s={slots_per_region} seed={seed} failures={} crash_window={crash_window} \
                                         lost_disk={lost_disk} fakes={} unrecoverable={} retention_violations={}",
                                        variant.name(), arm.name(), format_set(pattern), case.fake_candidates, case.unrecoverable, case.retention_violations
                                    )));
                                }
                            }
                        }
                    }
                }
                for (lost_disk, classes) in by_class.iter().enumerate() {
                    for (failure_count, total) in classes.iter().enumerate() {
                        println!("{}", emitter.emit_raw(&format!(
                            "name=activation variant={} arm={} s={slots_per_region} lost_disk={lost_disk} failures={failure_count} fakes={} unrecoverable={} \
                             retention_violations={} checks={}",
                            variant.name(), arm.name(), total.fake_candidates, total.unrecoverable, total.retention_violations, total.checks
                        )));
                        if let Some(index) = candidate_index(arm) {
                            ledgers[index].add_correctness(total);
                        }
                        match (arm, variant) {
                            (Arm::FloorTxg, ActivationVariant::Plain) => floor_txg_activation_fakes += total.fake_candidates,
                            (Arm::FloorPerDiskTxgUpperLimit, ActivationVariant::SkipThenRaise { .. }) => txg_upper_limit_skip_retention += total.retention_violations,
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    // near_full（E138 原样，两档开销对）
    let mut floor_per_disk_near_full_write_after_delete_cost_five = 0u64;
    let mut ring_raw_false_enospc = 0u64;
    for slots_per_region in SLOTS_PER_REGION_SWEEP {
        for (mount_cost, declared_maximum_cost) in COST_PAIRS {
            for arm in ARMS {
                let ring_depth = REGION_COUNT * slots_per_region;
                let mut total = Metrics::fresh();
                let mut failure_windows: Vec<(u64, u64, u64)> = Vec::new();
                for seed in 0..SEED_COUNT {
                    let one = run_near_full(arm, slots_per_region, mount_cost, declared_maximum_cost, seed, None);
                    if arm.is_candidate() && one.write_after_delete_failed > 0 {
                        failure_windows.push((seed, one.first_failure_window_or_zero(), one.last_write_after_delete_failure_window));
                    }
                    total.add(&one);
                }
                let formula_cost = arm.formula_cost(mount_cost, declared_maximum_cost);
                let reserve = arm.reserve_blocks(ring_depth, formula_cost);
                let residue = arm.push_residue_blocks(ring_depth, formula_cost);
                let bound = arm.publication_bound(ring_depth);
                println!("{}", emitter.emit_raw(&format!(
                    "name=near_full arm={} s={slots_per_region} c0={mount_cost} c_max={declared_maximum_cost} reserve={reserve} residue={residue} \
                     hidden_blocks={} hidden_bytes={} avg_lag_x100={} max_lag={} false_enospc_raw={} false_enospc_arm={} true_enospc={} \
                     write_after_delete_failed={} first_failure_window={} last_failure_window={} max_publications={} bound={bound} \
                     admissions_under_pressure={} forced_per_window_x100={} stalls={} halted_before_counting={} fakes={} unrecoverable={} \
                     retention_violations={} min_candidates={} avg_candidates_x100={} avg_nonempty_x100={} min_states={} avg_states_x100={}",
                    arm.name(), reserve + residue, (reserve + residue) * BLOCK_BYTES, total.average_lag_in_hundredths(), total.lag_maximum,
                    total.false_enospc_raw, total.false_enospc_arm, total.true_enospc, total.write_after_delete_failed,
                    total.first_failure_window_or_zero(), total.last_write_after_delete_failure_window, total.maximum_publications_per_admission,
                    total.admissions_under_pressure, total.forced_publications_per_window_in_hundredths(), total.stalls, total.halted_before_counting,
                    total.fake_candidates, total.unrecoverable, total.retention_violations, total.minimum_candidates_or_zero(),
                    total.average_candidates_in_hundredths(), total.average_nonempty_candidates_in_hundredths(),
                    total.minimum_distinct_states_or_zero(), total.average_distinct_states_in_hundredths()
                )));
                if !failure_windows.is_empty() {
                    println!("{}", emitter.emit_raw(&format!(
                        "name=near_full_failure_windows arm={} s={slots_per_region} c0={mount_cost} seeds_with_failure={} first_windows={} last_windows={}",
                        arm.name(), failure_windows.len(),
                        format_set(&failure_windows.iter().map(|(_, first, _)| *first).collect::<Vec<u64>>()),
                        format_set(&failure_windows.iter().map(|(_, _, last)| *last).collect::<Vec<u64>>())
                    )));
                }
                if arm == Arm::FloorPerDisk && mount_cost == 5 {
                    floor_per_disk_near_full_write_after_delete_cost_five += total.write_after_delete_failed;
                }
                if arm == Arm::RingPersisted {
                    ring_raw_false_enospc += total.false_enospc_raw;
                }
                if let Some(index) = candidate_index(arm) {
                    ledgers[index].add_space_cell(&total, bound);
                }
            }
        }
    }

    // drain_failure
    let mut ring_drain_false_enospc = 0u64;
    for slots_per_region in FAULT_SLOTS_SWEEP {
        for (mount_cost, declared_maximum_cost) in COST_PAIRS {
            for arm in NEAR_FULL_FAULT_ARMS {
                let ring_depth = REGION_COUNT * slots_per_region;
                let bound = arm.publication_bound(ring_depth);
                let drain_patterns = drain_failure_patterns(arm.drain_failure_offset_limit(ring_depth));
                let mut by_count = [Metrics::fresh(); 4];
                let mut cases_by_count = [0u64; 4];
                let mut runs_with_false = [0u64; 4];
                let mut runs_with_write_after_delete = [0u64; 4];
                for seed in 0..FAULT_SEED_COUNT {
                    let prefix = match drain_failure_prefix(arm, slots_per_region, mount_cost, declared_maximum_cost, seed) {
                        None => continue,
                        Some(simulation) => simulation,
                    };
                    for pattern in &drain_patterns {
                        let case = run_drain_failure_case(&prefix, pattern);
                        by_count[pattern.len()].add(&case);
                        cases_by_count[pattern.len()] += 1;
                        runs_with_false[pattern.len()] += u64::from(case.false_enospc_arm > 0);
                        runs_with_write_after_delete[pattern.len()] += u64::from(case.write_after_delete_failed > 0);
                        if arm.is_candidate() && (pattern.len() as u64) <= TOLERATED_ROOT_WRITE_FAILURES
                            && (case.false_enospc_arm > 0 || case.write_after_delete_failed > 0 || case.maximum_publications_per_admission > bound || case.stalls > 0)
                            && detail_lines < 2 * DETAIL_LINES_MAXIMUM
                        {
                            detail_lines += 1;
                            println!("{}", emitter.emit_raw(&format!(
                                "name=detail_drain arm={} s={slots_per_region} c0={mount_cost} seed={seed} failures={} false_enospc_arm={} \
                                 write_after_delete_failed={} max_publications={} stalls={}",
                                arm.name(), format_set(pattern), case.false_enospc_arm, case.write_after_delete_failed,
                                case.maximum_publications_per_admission, case.stalls
                            )));
                        }
                    }
                }
                for failure_count in 1..=3usize {
                    let total = by_count[failure_count];
                    println!("{}", emitter.emit_raw(&format!(
                        "name=drain_failure arm={} s={slots_per_region} c0={mount_cost} c_max={declared_maximum_cost} failures={failure_count} cases={} \
                         runs_with_false_enospc={} runs_with_write_after_delete_failed={} false_enospc_arm={} false_enospc_raw={} write_after_delete_failed={} \
                         max_publications={} bound={bound} stalls={} fakes={} unrecoverable={} retention_violations={} injected={}",
                        arm.name(), cases_by_count[failure_count], runs_with_false[failure_count], runs_with_write_after_delete[failure_count],
                        total.false_enospc_arm, total.false_enospc_raw, total.write_after_delete_failed, total.maximum_publications_per_admission,
                        total.stalls, total.fake_candidates, total.unrecoverable, total.retention_violations, total.root_write_failures_injected
                    )));
                    if arm == Arm::RingPersisted {
                        ring_drain_false_enospc += total.false_enospc_arm;
                    }
                    if let Some(index) = candidate_index(arm) {
                        if failure_count as u64 <= TOLERATED_ROOT_WRITE_FAILURES {
                            ledgers[index].add_space_cell(&total, bound);
                            ledgers[index].maximum_publications_within_tolerance =
                                ledgers[index].maximum_publications_within_tolerance.max(total.maximum_publications_per_admission);
                        } else {
                            ledgers[index].add_correctness(&total);
                            ledgers[index].maximum_publications_beyond_tolerance =
                                ledgers[index].maximum_publications_beyond_tolerance.max(total.maximum_publications_per_admission);
                        }
                    }
                }
            }
        }
    }

    // broken_slot
    let mut ring_broken_slot_false_or_stall = 0u64;
    for slots_per_region in FAULT_SLOTS_SWEEP {
        for (mount_cost, declared_maximum_cost) in COST_PAIRS {
            for arm in NEAR_FULL_FAULT_ARMS {
                let ring_depth = REGION_COUNT * slots_per_region;
                let bound = arm.publication_bound(ring_depth);
                let mut total = Metrics::fresh();
                let mut seeds_with_no_space = 0u64;
                let mut seeds_halted = 0u64;
                for seed in 0..FAULT_SEED_COUNT {
                    let one = run_broken_slot(arm, slots_per_region, mount_cost, declared_maximum_cost, seed);
                    seeds_with_no_space += u64::from(one.write_after_delete_failed > 0);
                    seeds_halted += u64::from(one.stalls > 0);
                    total.add(&one);
                }
                println!("{}", emitter.emit_raw(&format!(
                    "name=broken_slot arm={} s={slots_per_region} c0={mount_cost} c_max={declared_maximum_cost} seeds={FAULT_SEED_COUNT} seeds_with_no_space={seeds_with_no_space} \
                     seeds_halted={seeds_halted} first_failure_window={} false_enospc_arm={} false_enospc_raw={} write_after_delete_failed={} max_publications={} bound={bound} \
                     stalls={} fakes={} unrecoverable={} retention_violations={} injected={} forced_per_window_x100={} max_lag={}",
                    arm.name(), total.first_failure_window_or_zero(), total.false_enospc_arm, total.false_enospc_raw, total.write_after_delete_failed,
                    total.maximum_publications_per_admission, total.stalls, total.fake_candidates, total.unrecoverable, total.retention_violations,
                    total.root_write_failures_injected, total.forced_publications_per_window_in_hundredths(), total.lag_maximum
                )));
                if arm == Arm::RingPersisted {
                    ring_broken_slot_false_or_stall += total.false_enospc_arm + total.stalls;
                }
                if let Some(index) = candidate_index(arm) {
                    ledgers[index].add_space_cell(&total, bound);
                }
            }
        }
    }

    // cost_growth
    let mut mount_formula_growth_false_or_stall = 0u64;
    for slots_per_region in FAULT_SLOTS_SWEEP {
        for (mount_cost, declared_maximum_cost) in COST_PAIRS {
            for arm in COST_GROWTH_ARMS {
                let ring_depth = REGION_COUNT * slots_per_region;
                let bound = arm.publication_bound(ring_depth);
                for grown_cost in [declared_maximum_cost, declared_maximum_cost + COST_OVER_DECLARED_EXTRA] {
                    let is_within_declared = grown_cost <= declared_maximum_cost;
                    let mut total = Metrics::fresh();
                    for seed in 0..FAULT_SEED_COUNT {
                        total.add(&run_cost_growth(arm, slots_per_region, mount_cost, declared_maximum_cost, seed, grown_cost));
                    }
                    println!("{}", emitter.emit_raw(&format!(
                        "name=cost_growth arm={} s={slots_per_region} c0={mount_cost} c_max={declared_maximum_cost} grown_to={grown_cost} within_declared={} \
                         false_enospc_arm={} false_enospc_raw={} true_enospc={} write_after_delete_failed={} first_failure_window={} max_publications={} bound={bound} \
                         stalls={} fakes={} unrecoverable={} retention_violations={} forced_per_window_x100={}",
                        arm.name(), u8::from(is_within_declared), total.false_enospc_arm, total.false_enospc_raw, total.true_enospc,
                        total.write_after_delete_failed, total.first_failure_window_or_zero(), total.maximum_publications_per_admission, total.stalls,
                        total.fake_candidates, total.unrecoverable, total.retention_violations, total.forced_publications_per_window_in_hundredths()
                    )));
                    if arm == Arm::TightenedRootsAtMount && is_within_declared {
                        mount_formula_growth_false_or_stall += total.false_enospc_arm + total.stalls;
                    }
                    if let Some(index) = candidate_index(arm) {
                        if is_within_declared {
                            ledgers[index].add_space_cell(&total, bound);
                        } else {
                            ledgers[index].add_correctness(&total);
                        }
                    }
                }
            }
        }
    }

    // 保留池扫描
    let mut minimum_no_stall_reserve = [[u64::MAX; 2]; 2];
    for (index, arm) in CANDIDATE_ARMS.iter().enumerate() {
        for (cost_index, (mount_cost, declared_maximum_cost)) in COST_PAIRS.iter().enumerate() {
            let ring_depth = REGION_COUNT * RESERVE_SWEEP_SLOTS_PER_REGION;
            let bound = arm.publication_bound(ring_depth);
            let formula = arm.reserve_blocks(ring_depth, arm.formula_cost(*mount_cost, *declared_maximum_cost));
            for reserve in (formula - RESERVE_SWEEP_BELOW)..=(formula + RESERVE_SWEEP_ABOVE) {
                let mut total = Metrics::fresh();
                for seed in 0..RESERVE_SWEEP_SEED_COUNT {
                    total.add(&run_near_full(*arm, RESERVE_SWEEP_SLOTS_PER_REGION, *mount_cost, *declared_maximum_cost, seed, Some(reserve)));
                }
                println!("{}", emitter.emit_raw(&format!(
                    "name=reserve_sweep arm={} s={RESERVE_SWEEP_SLOTS_PER_REGION} c0={mount_cost} c_max={declared_maximum_cost} reserve={reserve} formula={formula} \
                     stalls={} false_enospc_arm={} write_after_delete_failed={} max_publications={} fakes={}",
                    arm.name(), total.stalls, total.false_enospc_arm, total.write_after_delete_failed,
                    total.maximum_publications_per_admission, total.fake_candidates
                )));
                if total.stalls == 0 {
                    minimum_no_stall_reserve[index][cost_index] = minimum_no_stall_reserve[index][cost_index].min(reserve);
                }
                if reserve >= formula {
                    ledgers[index].add_space_cell(&total, bound);
                }
            }
        }
    }

    println!("{}", emitter.emit_raw(&format!(
        "name=verdict crit1_holds={} crit2_a_drain_false_enospc={ring_drain_false_enospc} crit2_a_broken_false_or_stall={ring_broken_slot_false_or_stall} \
         crit2_g_near_full_wad_cost5={floor_per_disk_near_full_write_after_delete_cost_five} crit2_g2r_mount_growth_false_or_stall={mount_formula_growth_false_or_stall} \
         crit2_d_activation_fakes={floor_txg_activation_fakes} crit2_gtxgcap_skip_retention={txg_upper_limit_skip_retention} crit2_a_raw_false_enospc={ring_raw_false_enospc}",
        u8::from(criterion_one_holds)
    )));
    for (index, arm) in CANDIDATE_ARMS.iter().enumerate() {
        let ledger = ledgers[index];
        println!("{}", emitter.emit_raw(&format!(
            "name=verdict_candidate arm={} crit3_fakes={} crit3_unrecoverable={} crit4_false_enospc_cells={} crit5_write_after_delete_cells={} \
             crit5_bound_exceeded_cells={} crit6_stall_cells={} crit7_retention={} crit8_max_publications_within_tolerance={} \
             beyond_tolerance_max_publications={} bound={}",
            arm.name(), ledger.fakes, ledger.unrecoverable, ledger.false_enospc_cells, ledger.write_after_delete_cells, ledger.bound_exceeded_cells,
            ledger.stall_cells, ledger.retention_violations, ledger.maximum_publications_within_tolerance, ledger.maximum_publications_beyond_tolerance,
            arm.publication_bound(REGION_COUNT * 16)
        )));
    }
    println!("{}", emitter.emit_raw(&format!(
        "name=crit6_min_no_stall_reserve g2r_pair0={} g2r_pair1={} g2s_pair0={} g2s_pair1={}",
        minimum_no_stall_reserve[0][0], minimum_no_stall_reserve[0][1], minimum_no_stall_reserve[1][0], minimum_no_stall_reserve[1][1]
    )));
    println!("{}", emitter.emit_raw(&format!(
        "name=crit9_c281 {}",
        ARMS.iter().enumerate().map(|(index, arm)| format!("{}={}", arm.name(), first_publish_unrecoverable[index])).collect::<Vec<String>>().join(" ")
    )));
    let (first_rerun_rows, first_rerun_crit1_bodies) = run_window_arms(&mut emitter);
    start_recording_lines();
    run_second_rerun_arms(&mut emitter, &first_rerun_rows, &first_rerun_crit1_bodies);
    let second_rerun_lines = stop_recording_lines();
    run_core_gate_arms(&mut emitter, &second_rerun_lines);
    println!("{}", emitter.finish());
}
