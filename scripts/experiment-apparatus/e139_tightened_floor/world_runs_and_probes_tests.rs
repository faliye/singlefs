use super::*;

#[test]
fn regions_zero_and_two_live_on_disk_zero_region_one_on_disk_one() {
    let simulation = Simulation::new(Arm::RingPersisted, 4, 1, 5, 1, false, None);
    let disks: Vec<usize> = (0..12).map(|slot| simulation.disk_of_slot(slot)).collect();
    assert_eq!(disks, vec![0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0], "区域 0、2 在盘 0，区域 1 在盘 1");
}

#[test]
fn criterion_one_pinned_blocks_match_e123_both_readings() {
    for (slots_per_region, before, after) in [(1u64, 39u64, 26u64), (4, 156, 143), (16, 624, 611)] {
        let (pinned_before, pinned_after, _, _) = steady_probe(Arm::RingPersisted, slots_per_region);
        assert_eq!(pinned_before, before, "S={slots_per_region} 发布之前应扣 N × 13");
        assert_eq!(pinned_after, after, "S={slots_per_region} 发布刚完成应扣 (N − 1) × 13");
    }
}

#[test]
fn criterion_one_fake_candidates_match_e92_both_readings() {
    for (slots_per_region, before, after) in [(1u64, 0u64, 0u64), (4, 9, 8), (16, 45, 44)] {
        let (_, _, fake_before, fake_after) = steady_probe(Arm::ProtectedGenerationsAtMinimum, slots_per_region);
        assert_eq!(fake_before, before, "S={slots_per_region} 发布之前假候选应为 N − K");
        assert_eq!(fake_after, after, "S={slots_per_region} 发布之后假候选应为 N − K − 1");
    }
}

#[test]
fn ring_push_uses_ring_depth_publications_and_leaves_preregistered_residue() {
    for (slots_per_region, cost, publications, pinned) in
        [(1u64, 1u64, 3u64, 6u64), (1, 5, 3, 10), (4, 1, 12, 15), (4, 5, 12, 55), (16, 1, 48, 51), (16, 5, 48, 235)]
    {
        let (publication_set, maximum_publications, maximum_pinned, maximum_lag, is_all_released) = push_probe_over_landings(Arm::RingPersisted, slots_per_region, cost);
        assert!(is_all_released, "S={slots_per_region} c={cost} 处置之后这次删的块必须放回");
        assert_eq!(publication_set, vec![publications], "S={slots_per_region} c={cost} 臂 A 每种落点都是 N 次发布");
        assert_eq!(maximum_publications, publications);
        assert_eq!(maximum_pinned, pinned, "S={slots_per_region} c={cost} 臂 A 扣住 5 + (N − 2) × c");
        assert_eq!(maximum_lag, 0, "臂 A 没有滞后量");
        if slots_per_region > 1 {
            assert_eq!(maximum_pinned, Arm::RingPersisted.push_residue_blocks(REGION_COUNT * slots_per_region, cost), "残留式子与实测一致");
        }
    }
}

#[test]
fn roots_arms_push_use_six_or_seven_publications_and_leave_preregistered_residue() {
    for arm in [Arm::FloorPerDisk, Arm::TightenedRoots] {
        for (slots_per_region, cost, publications, maximum, pinned) in [
            (1u64, 1u64, vec![3u64], 3u64, 6u64),
            (1, 5, vec![3], 3, 10),
            (4, 1, vec![6, 7], 7, 10),
            (4, 5, vec![6, 7], 7, 30),
            (16, 1, vec![6, 7], 7, 10),
            (16, 5, vec![6, 7], 7, 30),
        ] {
            let (publication_set, maximum_publications, maximum_pinned, maximum_lag, is_all_released) = push_probe_over_landings(arm, slots_per_region, cost);
            assert!(is_all_released, "{} S={slots_per_region} c={cost} 处置之后这次删的块必须放回", arm.name());
            assert_eq!(publication_set, publications, "{} S={slots_per_region} c={cost} 三种落点的发布数", arm.name());
            assert_eq!(maximum_publications, maximum, "{} S={slots_per_region} c={cost} 最多 7 次", arm.name());
            assert_eq!(maximum_pinned, pinned, "{} S={slots_per_region} c={cost} 扣住最多 5 + 5c（S = 1 时 5 + c）", arm.name());
            assert_eq!(maximum_lag, 0, "按根数读没有滞后量");
            if slots_per_region > 1 && arm == Arm::FloorPerDisk {
                assert_eq!(maximum_pinned, Arm::FloorPerDisk.push_residue_blocks(REGION_COUNT * slots_per_region, cost), "E138 臂 G 的残留式子与实测一致（G2R 的式子按 B_R = 11 留了余量，比实测大）");
            }
        }
    }
}

/// 判据 1 里 G2S 那一行：目标是第 4 新的非空根，3 或 4 次发布放回；滞后量 = 3 个非空窗口各放掉的 13 块。
#[test]
fn states_arm_push_uses_three_or_four_publications_and_lags_three_windows() {
    for (slots_per_region, cost, publications, maximum, pinned, lag) in [
        (1u64, 1u64, vec![3u64], 3u64, 6u64, 0u64),
        (1, 5, vec![3], 3, 10, 0),
        (4, 1, vec![3, 4], 4, 7, 39),
        (4, 5, vec![3, 4], 4, 15, 39),
        (16, 1, vec![3, 4], 4, 7, 39),
        (16, 5, vec![3, 4], 4, 15, 39),
    ] {
        let (publication_set, maximum_publications, maximum_pinned, maximum_lag, is_all_released) = push_probe_over_landings(Arm::TightenedStates, slots_per_region, cost);
        assert!(is_all_released, "S={slots_per_region} c={cost} 处置之后目标必须放回");
        assert_eq!(publication_set, publications, "S={slots_per_region} c={cost} 三种落点的发布数");
        assert_eq!(maximum_publications, maximum, "S={slots_per_region} c={cost} 最多 4 次");
        assert_eq!(maximum_pinned, pinned, "S={slots_per_region} c={cost} 滞后量之外扣住最多 5 + 2c");
        assert_eq!(maximum_lag, lag, "S={slots_per_region} c={cost} 滞后量是 3 × 13");
    }
}

fn activation_totals(arm: Arm, slots_per_region: u64, seeds: u64, variant: ActivationVariant, failure_count: Option<usize>) -> Metrics {
    let skip_repeats = match variant {
        ActivationVariant::SkipThenRaise { repeats } => repeats,
        ActivationVariant::Plain | ActivationVariant::CrashWithoutLossFirst => 0,
    };
    let mut total = Metrics::fresh();
    for seed in 0..seeds {
        let prefix = activation_prefix(arm, slots_per_region, seed, skip_repeats).expect("steady 前缀不该卡死");
        for pattern in failure_patterns().iter().filter(|pattern| failure_count.is_none_or(|count| pattern.len() == count)) {
            for crash_window in 1..=ACTIVATION_FAULT_CRASH_WINDOWS {
                for lost_disk in 0..DISK_COUNT {
                    total.add(&run_activation_case(&prefix, pattern, crash_window, lost_disk, variant));
                }
            }
        }
    }
    total
}

#[test]
fn floor_txg_produces_fake_candidates_when_faults_hit_activation_interval() {
    let total = activation_totals(Arm::FloorTxg, 4, 2, ActivationVariant::Plain, None);
    assert!(total.fake_candidates > 0, "按 txg 算活化，活化区间里写失败再丢盘必须冒出假候选");
}

#[test]
fn protected_generations_runtime_is_unrecoverable_when_a_jump_is_followed_by_disk_loss() {
    let total = activation_totals(Arm::ProtectedGenerationsAtMinimum, 4, 2, ActivationVariant::Plain, None);
    assert!(total.unrecoverable > 0, "K = 3 在跳号下不够：写失败 + 丢盘必须出不可恢复");
}

/// 判据 2 的靶子：先跳号再抬 F，按 txg 数 4 代的上限越过盘 1 上最新的根（E138 判「未验」的那一半）。
#[test]
fn txg_upper_limit_control_violates_retention_when_floor_is_raised_after_the_jump() {
    let total = activation_totals(Arm::FloorPerDiskTxgUpperLimit, 4, 2, ActivationVariant::SkipThenRaise { repeats: 2 }, Some(0));
    assert!(total.retention_violations > 0, "先跳号再抬 F 时按 txg 数 4 代的上限必须越过盘 1 上最新的根");
}

#[test]
fn tightened_arms_are_clean_across_activation_variants() {
    for arm in CANDIDATE_ARMS {
        for variant in [ActivationVariant::Plain, ActivationVariant::CrashWithoutLossFirst, ActivationVariant::SkipThenRaise { repeats: 2 }] {
            for slots_per_region in [4u64, 16] {
                let total = activation_totals(arm, slots_per_region, 2, variant, None);
                assert_eq!(total.fake_candidates, 0, "{} {} S={slots_per_region} 不许有假候选", arm.name(), variant.name());
                assert_eq!(total.unrecoverable, 0, "{} {} S={slots_per_region} 不许不可恢复", arm.name(), variant.name());
                assert_eq!(total.retention_violations, 0, "{} {} S={slots_per_region} 不许越过用户口径", arm.name(), variant.name());
            }
        }
    }
}

#[test]
fn tightened_arms_are_clean_in_fault_worlds() {
    for arm in CANDIDATE_ARMS {
        for world in [World::TornNewest, World::DiskLoss, World::SlotWriteFail, World::AdminRollback] {
            // S = 1 也要跑：非空有效根不足 4 个的那条分支只在 3 槽的环上走到（第一次运行 G2S 在这里 41 次违反最少保留）
            for slots_per_region in [1u64, 4, 16] {
                for lost_disk in 0..DISK_COUNT {
                    let total = (0..6).fold(Metrics::fresh(), |mut sum, seed| {
                        sum.add(&run_world(arm, world, slots_per_region, seed, lost_disk));
                        sum
                    });
                    assert_eq!(total.fake_candidates, 0, "{} {} S={slots_per_region} 丢盘 {lost_disk} 不许有假候选", arm.name(), world.name());
                    assert_eq!(total.unrecoverable, 0, "{} {} S={slots_per_region} 丢盘 {lost_disk} 不许不可恢复", arm.name(), world.name());
                    assert_eq!(total.retention_violations, 0, "{} {} S={slots_per_region} 丢盘 {lost_disk} 不许违反最少保留", arm.name(), world.name());
                }
            }
        }
    }
}

/// 判据 2 的靶子与判据 5：E138 的臂 G 在 c₀ = 5 近满盘删了再写要失败，按最坏残留收费的 G2R 不失败。
#[test]
fn charging_worst_residue_removes_write_after_delete_failure_at_cost_five() {
    let control = run_near_full(Arm::FloorPerDisk, 4, 5, 10, 0, None);
    assert!(control.write_after_delete_failed > 0, "E138 的臂 G 在 S=4 c₀=5 种子 0 删了再写要失败（判据 2 的靶子）");
    assert!(control.true_enospc > 0, "填满阶段必须真的走到 ENOSPC");
    let tightened = run_near_full(Arm::TightenedRoots, 4, 5, 10, 0, None);
    assert_eq!(tightened.write_after_delete_failed, 0, "G2R 按最坏残留收费之后删了再写不许失败");
    assert_eq!(tightened.false_enospc_arm, 0, "G2R 不许有假性 ENOSPC");
    assert!(tightened.maximum_publications_per_admission <= TIGHTENED_ROOTS_PUBLICATION_BOUND, "G2R 一次准入不超过 B_R");
    assert_eq!(tightened.stalls, 0, "跑前保留池下不许卡死");
    assert!(tightened.true_enospc > 0, "填满阶段必须真的走到 ENOSPC，否则上面几条什么都没测");
}

/// 两种读法的差别落成数：按根数读，推空之后候选集只剩 1 个状态；按状态数读永远不少于 4 个。
#[test]
fn states_arm_keeps_four_distinct_states_where_roots_arm_drops_to_one() {
    let roots = run_near_full(Arm::TightenedRoots, 4, 5, 10, 0, None);
    assert_eq!(roots.minimum_distinct_states_or_zero(), 1, "按根数读，推空之后候选集里只剩这次删除那一个状态");
    assert!(roots.minimum_candidates_or_zero() >= 4, "但按根数数它仍然不少于 4 个根");
    let states = run_near_full(Arm::TightenedStates, 4, 5, 10, 0, None);
    assert!(states.minimum_distinct_states_or_zero() >= 4, "按状态数读，候选集里任何时候都不少于 4 个不同状态");
    assert_eq!(states.retention_violations, 0, "按状态数读不许违反用户口径");
}

/// 判据 2 的靶子与判据 4：坏槽之下臂 A 出假性 ENOSPC 或卡死，两条收严臂都不出。
#[test]
fn broken_slot_breaks_ring_arm_but_not_tightened_arms() {
    let ring = run_broken_slot(Arm::RingPersisted, 4, 1, 5, 0);
    assert!(ring.false_enospc_arm + ring.stalls > 0, "坏槽把臂 A 的界冻住，必须出假性 ENOSPC 或卡死（判据 2 的靶子）");
    for arm in CANDIDATE_ARMS {
        let total = run_broken_slot(arm, 4, 1, 5, 0);
        assert_eq!(total.false_enospc_arm, 0, "{} 坏槽之下不许有假性 ENOSPC", arm.name());
        assert_eq!(total.write_after_delete_failed, 0, "{} 坏槽之下删了再写不许失败", arm.name());
        assert_eq!(total.stalls, 0, "{} 坏槽之下不许卡死", arm.name());
        assert!(total.root_write_failures_injected > 0, "{} 坏槽必须真的被写到过，否则上面什么都没测", arm.name());
    }
}

/// 判据 2 的靶子与判据 4、8：推空期间一次根槽写失败，臂 A 出假性 ENOSPC；G2R 在容忍次数之内不出、发布数不超过 B_R。
#[test]
fn drain_failure_breaks_ring_arm_and_stays_within_tightened_bound() {
    let ring_prefix = drain_failure_prefix(Arm::RingPersisted, 4, 5, 10, 0).expect("填满不该卡死");
    let ring = run_drain_failure_case(&ring_prefix, &[1]);
    assert!(ring.false_enospc_arm > 0, "推空第 1 次空发布的根槽写失败，臂 A 必须出假性 ENOSPC（第三轮反推腿 R1）");
    for arm in CANDIDATE_ARMS {
        let prefix = drain_failure_prefix(arm, 4, 5, 10, 0).expect("填满不该卡死");
        let bound = arm.publication_bound(12);
        for pattern in drain_failure_patterns(arm.drain_failure_offset_limit(12)).iter().filter(|pattern| pattern.len() <= 2) {
            let case = run_drain_failure_case(&prefix, pattern);
            assert_eq!(case.false_enospc_arm, 0, "{} 失败 {:?} 之下不许有假性 ENOSPC", arm.name(), pattern);
            assert!(case.maximum_publications_per_admission <= bound, "{} 失败 {:?} 之下发布数 {} 不许超过 B = {bound}", arm.name(), pattern, case.maximum_publications_per_admission);
            assert_eq!(case.stalls, 0, "{} 失败 {:?} 之下不许卡死", arm.name(), pattern);
            assert_eq!(case.fake_candidates, 0, "{} 失败 {:?} 之下不许有假候选", arm.name(), pattern);
        }
    }
}

/// 判据 2 的靶子与判据 4：保留池按挂载时的开销算，开销变大之后出假性 ENOSPC 或卡死；按声明的最坏值算的 G2R 不出。
#[test]
fn cost_growth_breaks_mount_formula_but_not_declared_maximum_formula() {
    let mount = run_cost_growth(Arm::TightenedRootsAtMount, 4, 1, 5, 0, 5);
    assert!(mount.false_enospc_arm + mount.stalls > 0, "按挂载时的开销算式子，开销升到 c_max 之后必须出假性 ENOSPC 或卡死（第三轮反推腿 R3）");
    let declared = run_cost_growth(Arm::TightenedRoots, 4, 1, 5, 0, 5);
    assert_eq!(declared.false_enospc_arm, 0, "按 c_max 算式子，开销升到 c_max 不许有假性 ENOSPC");
    assert_eq!(declared.write_after_delete_failed, 0, "按 c_max 算式子，开销升到 c_max 删了再写不许失败");
    assert_eq!(declared.stalls, 0, "按 c_max 算式子，开销升到 c_max 不许卡死");
}

/// 变异 M6 的敏感取样点（E138 原样）：新 F 只写在一块盘上时崩溃（两块盘都在），恢复后的生效值必须还是旧的。
#[test]
fn recovery_does_not_activate_floor_carried_on_only_one_disk() {
    let mut simulation = activation_prefix(Arm::TightenedRoots, 4, 0, 0).expect("steady 前缀不该卡死");
    let new_floor = simulation.floor_carried;
    let old_active = simulation.floor_active;
    assert!(new_floor > old_active, "前缀要真的抬了 F");
    assert_eq!(simulation.run_user_window(true, false), WindowOutcome::Created);
    assert!(simulation.pending_floor.is_some(), "只写了一个带新 F 的根，不该已经生效");
    assert!(!simulation.floor_written_on_every_surviving_disk(new_floor), "前提：新 F 只在一块盘上");
    simulation.crash_and_recover();
    assert!(simulation.floor_active < new_floor, "恢复后生效值取各盘所带 F 的最小值，只在一块盘上的新 F 不许生效");
    assert!(simulation.pending_floor.is_some(), "新 F 还要照生效条件继续等");
}

/// 强制盘紧把 F 抬到上限，推空的空根进了环之后再抬一次：按状态数读的上限是第 4 新的非空根，抬完候选集里仍有 4 个不同状态；
/// 按根数读的上限把空根也数进去，抬得更高（对照）。
#[test]
fn forced_pressure_keeps_four_states_for_states_arm() {
    let mut floors = Vec::new();
    for arm in [Arm::TightenedStates, Arm::TightenedRoots] {
        let mut simulation = Simulation::new(arm, 4, 1, 5, 3, false, None);
        simulation.prefill(STEADY_FILL);
        for _ in 0..100 {
            simulation.run_user_window(true, false);
        }
        simulation.forced_pressure();
        assert_eq!(simulation.run_user_window(true, false), WindowOutcome::Created);
        simulation.forced_pressure();
        assert!(simulation.floor_carried > 0, "{} 强制盘紧要真的抬了 F", arm.name());
        if arm.counts_states() {
            assert!(simulation.floor_carried <= simulation.per_disk_states_upper_limit(), "F 不许越过按状态数读的上限");
            assert_eq!(simulation.floor_carried, simulation.fourth_newest_nonempty_valid_txg(), "上限就是第 4 新的非空根");
            let (_, _, _, distinct_states) = simulation.count_fake_candidates();
            assert!(distinct_states >= 4, "抬完候选集里还要有 4 个不同状态，只剩 {distinct_states} 个");
        }
        floors.push(simulation.floor_carried);
    }
    assert!(floors[1] > floors[0], "环里有空根时按根数读抬得比按状态数读高（roots={} states={}）", floors[1], floors[0]);
}

#[test]
fn raw_df_detector_fires_for_ring_arm_near_full() {
    let total = run_near_full(Arm::RingPersisted, 4, 5, 10, 0, None);
    assert!(total.false_enospc_raw > 0, "df_raw 把保留池算成空闲，臂 A 报 ENOSPC 时它必须 ≥ 8（检测器的阳性对照）");
}

/// skip_then_raise 的前缀真的跳了号：区域 1 连续失败 r 次之后，盘 1 上最新的根比最新根旧 3r 以上。
#[test]
fn skip_then_raise_prefix_really_lags_disk_one() {
    let prefix = activation_prefix(Arm::TightenedRoots, 4, 0, 3).expect("前缀不该卡死");
    assert_eq!(prefix.metrics.root_write_failures_injected, 3, "要恰好注入 3 次写失败");
    let newest = prefix.newest_durable_txg;
    let disk_one = prefix.newest_valid_txg_on_disk(1).expect("盘 1 上要有根");
    assert!(newest - disk_one >= 3, "盘 1 上最新的根要落后 3 个 txg 以上（newest={newest} disk_one={disk_one}）");
    assert!(prefix.floor_carried <= disk_one, "按盘取上限：抬到的 F 不许越过盘 1 上最新的根");
}
/// 跑后补的诊断：`reserve_sweep` 的产物行不打印失败窗口号，
/// 所以「按状态数读的删了再写失败全在填满后的前 3 个窗口」对保留池扫描的 18 格没有产物字段撑着。
/// 这里把 G2S 在 S = 16、保留池 ≥ 跑前式子的每一格按种子重跑一遍，打印首末失败窗口并钉死：
/// c₀ = 5 的 9 格首 1、末 3；c₀ = 1 的 9 格里 reserve = 式子 + 2..5 的四格末窗口是 6，其余末 3。
/// 它只读装置、不改 `main` 的产物；产物留存为 `research/results/e139-tightened-floor-2026-09-12-reserve-sweep-windows.out`。
#[test]
fn states_arm_reserve_sweep_failure_windows_end_by_window_six() {
    let arm = Arm::TightenedStates;
    let ring_depth = REGION_COUNT * RESERVE_SWEEP_SLOTS_PER_REGION;
    for (mount_cost, declared_maximum_cost) in COST_PAIRS {
        let formula = arm.reserve_blocks(ring_depth, arm.formula_cost(mount_cost, declared_maximum_cost));
        for reserve in formula..=(formula + RESERVE_SWEEP_ABOVE) {
            let mut first_window_minimum = u64::MAX;
            let mut last_window_maximum = 0u64;
            let mut write_after_delete_failed = 0u64;
            let mut seeds_failing = 0u64;
            for seed in 0..RESERVE_SWEEP_SEED_COUNT {
                let one = run_near_full(arm, RESERVE_SWEEP_SLOTS_PER_REGION, mount_cost, declared_maximum_cost, seed, Some(reserve));
                write_after_delete_failed += one.write_after_delete_failed;
                if one.write_after_delete_failed > 0 {
                    seeds_failing += 1;
                    first_window_minimum = first_window_minimum.min(one.first_write_after_delete_failure_window);
                    last_window_maximum = last_window_maximum.max(one.last_write_after_delete_failure_window);
                }
            }
            println!("RSWEEP_WINDOWS arm={} s={RESERVE_SWEEP_SLOTS_PER_REGION} c0={mount_cost} c_max={declared_maximum_cost} reserve={reserve} formula={formula} write_after_delete_failed={write_after_delete_failed} seeds_failing={seeds_failing} first_failure_window={first_window_minimum} last_failure_window={last_window_maximum}",
                arm.name());
            let above_formula = reserve - formula;
            let (expected_first, expected_last) = match (mount_cost, above_formula) {
                (5, _) => (1, 3),
                (1, 2) => (2, 6),
                (1, 3..=5) => (1, 6),
                (1, _) => (2, 3),
                (other_cost, _) => panic!("COST_PAIRS 只有 c₀ = 1 与 5，这里是 {other_cost}"),
            };
            assert_eq!(seeds_failing, RESERVE_SWEEP_SEED_COUNT, "c0={mount_cost} reserve={reserve}：每个种子都该在填满后失败过");
            assert_eq!((first_window_minimum, last_window_maximum), (expected_first, expected_last),
                "c0={mount_cost} reserve={reserve}（式子 + {above_formula}）：首末失败窗口要钉在 ({expected_first}, {expected_last})");
        }
    }
}
