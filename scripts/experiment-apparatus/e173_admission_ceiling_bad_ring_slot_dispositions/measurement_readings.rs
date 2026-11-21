use super::*;

fn request(accepted: bool, rejection: Option<RejectionReason>, was_judged_short: bool, floor_after: u64, candidates: usize) -> RequestRecord {
    RequestRecord {
        accepted,
        rejection,
        was_judged_short,
        empty_publishes: 0,
        empty_publish_blocks: 0,
        publishes_in_the_admission: usize::from(accepted),
        crossed_non_empty_roots: 0,
        states_lost: 0,
        floor_after,
        free_above_the_reserve_after: 7,
        candidate_states_after: candidates,
    }
}

fn run_of(requests: Vec<RequestRecord>, raises: Vec<RaiseRecord>) -> MountRun {
    MountRun {
        requests,
        raises,
        candidate_states_at_the_start: 4,
        cap_active_passes: 0,
        cap_blocked_passes: 0,
        empty_publish_refusals: 0,
        raise_refused_after_an_empty_publish: 0,
        empty_publish_bound_exhausted: 0,
        lowest_free_blocks_in_the_first_raise: None,
        trace: Vec::new(),
    }
}

/// 场景：基线 5 次请求全发成；注了读故障的那一次第 2 次起全被拒（第 2 次基线发成、这一次拒）。
/// 预期：多拒 4、第一次多拒落在第 1 步（从 0 数）、之后发成 0 次、一次挂载全 ENOSPC 成立；轨迹的峰值取第一次「不够」之后的最大值、为正的步数数全程、期末值取最后一步。
#[test]
fn extra_rejections_the_first_extra_step_and_the_trajectory_are_read_from_the_two_runs() {
    let accepted = |floor| request(true, None, false, floor, 4);
    let rejected = |floor| request(false, Some(RejectionReason::FloorAlreadyAtTheCeiling), true, floor, 3);
    let baseline = run_of(vec![accepted(0), accepted(0), accepted(0), accepted(0), accepted(0)], Vec::new());
    let with_fault = run_of(vec![accepted(0), rejected(0), rejected(2), rejected(5), rejected(5)], Vec::new());
    let metrics = metrics_of(&with_fault, &baseline);
    assert_eq!((metrics.rejected, metrics.extra_rejected, metrics.first_extra_step, metrics.accepted_after_the_first_extra, metrics.all_enospc), (4, 4, 1, 0, 1));
    assert_eq!(metrics.rejected_by_reason[RejectionReason::FloorAlreadyAtTheCeiling.index()], 4);
    assert_eq!(metrics.trajectories[0], Trajectory { peak_after_the_first_short: Some(1), positive_steps: 4, final_value: 1 });
    assert_eq!(metrics.trajectories[4], Trajectory { peak_after_the_first_short: Some(5), positive_steps: 3, final_value: 5 });
    assert_eq!((metrics.candidates_lowest, metrics.candidates_final), (3, 3));
    let one_accepted_afterwards = run_of(vec![accepted(0), rejected(0), accepted(0), rejected(0), rejected(0)], Vec::new());
    let later = metrics_of(&one_accepted_afterwards, &baseline);
    assert_eq!((later.extra_rejected, later.first_extra_step, later.accepted_after_the_first_extra, later.all_enospc), (3, 1, 1, 0));
}

/// 场景：两次抬 F：一次 F 18 越过 1 个非空根、少掉 2 个状态；一次 F 50 越过 0 个、没越过真值上限。
/// 预期：越过总数 1、一次抬 F 里越过的峰值 1、F 高于真值上限的步数 1、少掉的状态总数 2、一次抬 F 里少掉的最大 2。
#[test]
fn raise_records_fold_into_crossed_over_truth_and_lost_counts() {
    let raises = vec![
        RaiseRecord { old_floor: 0, new_floor: 18, truth_ceiling: 17, crossed_non_empty_roots: 1, states_lost: 2, publishes: 2 },
        RaiseRecord { old_floor: 18, new_floor: 50, truth_ceiling: 50, crossed_non_empty_roots: 0, states_lost: 0, publishes: 3 },
    ];
    let run = run_of(vec![request(true, None, false, 50, 4)], raises);
    let metrics = metrics_of(&run, &run_of(vec![request(true, None, false, 50, 4)], Vec::new()));
    assert_eq!((metrics.crossed, metrics.crossed_peak_in_one_raise, metrics.over_truth_steps), (1, 1, 1));
    assert_eq!((metrics.lost_total, metrics.most_states_lost_in_one_raise, metrics.raise_sequences), (2, 2, 2));
}

/// 场景：一格里 4 个位置上的同一个量取 3、0、7、7。
/// 预期：最大 7（取到最大的位置按先到的 2）、最小 0、量 > 0 的位置 3、位置总数 4；渲染成「7@2/0/3/4」。
#[test]
fn aggregate_keeps_the_maximum_its_first_position_the_minimum_and_the_positive_count() {
    let mut aggregate = Aggregate::empty();
    for (position, value) in [(1usize, 3i64), (2, 0), (3, 7), (4, 7)] {
        aggregate.add(position, value);
    }
    assert_eq!((aggregate.maximum, aggregate.maximum_at_position, aggregate.minimum, aggregate.positive_positions, aggregate.positions), (7, 3, 0, 3, 4));
    assert_eq!(aggregate.render(), "7@3/0/3/4");
    assert_eq!(Aggregate::empty().render(), "none");
}

/// 场景：一条判定在两个点上各有两格：A 点两格的值是 0 和 1，B 点两格的值是 0 和 0。
/// 预期：A 点打中（最大值 1、打中的格 1 / 2）、B 点没打中；门槛挪到最大值 + 1 之后 A 点由真转假。
#[test]
fn judgment_hits_at_one_and_the_shifted_threshold_flips_it() {
    let key = |slots_per_region: u64| CellKey {
        family: HistoryFamily::CrashedAfterOverwrites,
        fault_kind: Some(ReadFaultKind::DeviceError),
        persistence: Some(FaultPersistence::UntilTheProcessWritesTheSlot),
        slots_per_region,
        metadata_blocks: 5,
        reserve_reading: ReserveReading::CodeReadingOneEmptyPublish,
        fullness: PoolFullness::NearlyFullWithNoSpareOverwrite,
        arm: arm_of(StuckCeilingDisposition::AsToday, AllZeroSlotDisposition::NotMerged),
    };
    let cell_with = |slots_per_region: u64, value: i64| {
        let mut cell = CellSummary::new(key(slots_per_region), 1, 0);
        cell.quantities.insert("extra_rejected", {
            let mut aggregate = Aggregate::empty();
            aggregate.add(1, value);
            aggregate
        });
        cell
    };
    let cells = [cell_with(4, 0), cell_with(4, 1), cell_with(8, 0), cell_with(8, 0)];
    let references: Vec<&CellSummary> = cells.iter().collect();
    let settings = JudgmentSettings {
        judgment_label: "test",
        scope: "test",
        value: &|cell| Some(maximum_of(cell, "extra_rejected")),
        path_evidence: &|_| 1,
    };
    let points = collect_the_points(&references, &settings);
    let point_of = |slots_per_region: u64| points.iter().find(|(point, _)| point.slots_per_region == slots_per_region).map(|(_, totals)| *totals).expect("这个点有格");
    assert_eq!((point_of(4).observed_maximum, point_of(4).cells_hit, point_of(4).cells), (Some(1), 1, 2));
    assert_eq!((point_of(8).observed_maximum, point_of(8).cells_hit, point_of(8).cells), (Some(0), 0, 2));
    let mut emitter = Emitter::new();
    let per_arm = emit_a_judgment(&mut emitter, "test", &references, &settings);
    assert_eq!(per_arm.get("today"), Some(&(1, 2, Some(1))), "两个点里 A 点打中");
    let mut tally = RunTally::default();
    emit_the_threshold_shift_check(&mut emitter, &mut tally, "test", "test", &per_arm);
    assert_eq!((tally.self_checks_passed, tally.void_conditions.len()), (1, 0), "门槛挪到 2 之后由真转假");
}
