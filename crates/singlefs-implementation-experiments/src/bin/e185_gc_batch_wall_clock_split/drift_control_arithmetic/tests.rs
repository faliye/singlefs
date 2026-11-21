use super::*;

// ───────────── 第 4 次重跑（登记 research/prompts/e185-r4-prereg.md）：控漂移段的算术（不用卡，假钟） ─────────────

/// 一串窗中位喂暖到稳：第 i 窗交回 `medians[i]`（用完了重复最后一个），每窗之后过了 `seconds_per_window` 秒。交回（经过、发了几窗）。
fn warm_up_over(medians: &[f64], seconds_per_window: f64) -> WarmUpToStable {
    let mut window = 0_usize;
    let mut seconds = 0.0_f64;
    warm_up_to_stable(
        &mut || {
            let median = medians[window.min(medians.len() - 1)];
            window += 1;
            Some(median)
        },
        &mut || {
            seconds += seconds_per_window;
            seconds
        },
    )
}

#[test]
fn warm_up_to_stable_stops_at_the_fifth_window_when_the_last_three_agree_with_their_mean() {
    let warm_up = warm_up_over(&[10.0, 9.0, 8.0, 8.0, 8.0], 60.0);
    assert_eq!(warm_up.window_medians_nanoseconds.len(), 5);
    assert!(!warm_up.reached_the_cap);
    assert_eq!(warm_up.trajectory(), (Some(10.0), 2, Some(8.0)));
}

#[test]
fn warm_up_to_stable_waits_sixty_seconds_and_stops_at_six_hundred_without_agreement() {
    let early = warm_up_over(&[8.0, 8.0, 8.0], 10.0);
    assert_eq!(
        early.window_medians_nanoseconds.len(),
        6,
        "60 秒之前稳了也接着暖"
    );
    let never = warm_up_over(&[10.0, 20.0, 10.0, 20.0, 10.0, 20.0, 10.0], 100.0);
    assert!(never.reached_the_cap);
    assert_eq!(never.window_medians_nanoseconds.len(), 6);
    assert!(!are_the_last_windows_stable(&[Some(8.0), None, Some(8.0)]));
}

/// 每格热身的假提交：第 i 次（从 0 起）的线程钟是 100 × 0.98^min(i, 30) 纳秒；一组交回这一组各次的中位。
fn cell_warm_up_with_a_cost_that_settles_after_thirty(policy: CellWarmUp) -> usize {
    let mut sent = 0_usize;
    warm_up_the_cell(policy, &mut |count| {
        let costs: Vec<f64> = (sent..sent + count)
            .map(|index| 100.0 * 0.98_f64.powi(i32::try_from(index.min(30)).expect("30 以内")))
            .collect();
        sent += count;
        median_of(&costs)
    })
}

#[test]
fn cell_warm_up_until_two_groups_agree_stops_at_fifty_and_the_reproduction_arm_at_ten() {
    assert_eq!(
        cell_warm_up_with_a_cost_that_settles_after_thirty(CellWarmUp::UntilTwoGroupsAgree),
        50
    );
    assert_eq!(
        cell_warm_up_with_a_cost_that_settles_after_thirty(CellWarmUp::FixedTenSubmissions),
        WARM_UP_SUBMISSIONS
    );
    let never_settles = warm_up_the_cell(CellWarmUp::UntilTwoGroupsAgree, &mut |_count| None);
    assert_eq!(never_settles, CELL_WARM_UP_SUBMISSIONS_AT_MOST);
}

#[test]
fn controlled_arm_alternates_the_card_order_by_round_and_the_reproduction_arm_keeps_it() {
    let cards = [0_u32, 1];
    assert_eq!(
        cards_in_the_order_of_the_round(DriftArm::Controlled, 1, &cards),
        vec![0, 1]
    );
    assert_eq!(
        cards_in_the_order_of_the_round(DriftArm::Controlled, 2, &cards),
        vec![1, 0]
    );
    assert_eq!(
        cards_in_the_order_of_the_round(DriftArm::Controlled, 5, &cards),
        vec![0, 1]
    );
    assert_eq!(
        cards_in_the_order_of_the_round(DriftArm::Reproduction, 2, &[0]),
        vec![0]
    );
    assert!(is_the_card_measured_first_in_the_round(
        DriftArm::Controlled,
        4,
        &cards,
        1
    ));
}

#[test]
fn decreasing_probe_spins_hit_the_registered_ratios_from_point_zero_five_to_thirty_milliseconds() {
    let spins: Vec<u64> = (1..=ROUNDS)
        .map(|round| probe_spin_nanoseconds(DriftProbe::Decreasing, round))
        .collect();
    assert_eq!(
        spins,
        vec![8_000_000, 6_100_000, 4_200_000, 2_300_000, 400_000]
    );
    assert_eq!(probe_spin_nanoseconds(DriftProbe::Constant, 4), 2_000_000);
    for (crt_nanoseconds, expected_ten_thousandths) in [
        (50_000_u64, 559_u64),
        (1_000_000, 1556),
        (3_000_000, 3091),
        (6_000_000, 4571),
        (14_000_000, 6545),
        (30_000_000, 8000),
    ] {
        let medians: Vec<Option<f64>> = spins
            .iter()
            .map(|spin| Some((crt_nanoseconds + spin) as f64))
            .collect();
        let ratio = round_five_over_round_one(&medians).expect("五轮都读得到");
        assert_eq!(
            (ratio * 10_000.0).round(),
            expected_ten_thousandths as f64,
            "c_rt {crt_nanoseconds} ns 时第 5 轮 ÷ 第 1 轮是 {ratio}"
        );
        assert!(probe_drift_holds(ratio), "c_rt ≤ 30 ms 都看得出漂移");
    }
    assert!(!probe_drift_holds(0.93));
    assert_eq!(
        round_five_over_round_one(&[Some(1.0), None, None, None, None]),
        None
    );
}

#[test]
fn controlled_round_ratio_judgement_puts_the_edges_point_nine_and_its_inverse_inside_drift_gone() {
    assert_eq!(controlled_round_ratio_judgement(0.90), "drift_gone");
    assert_eq!(controlled_round_ratio_judgement(1.0 / 0.90), "drift_gone");
    assert_eq!(controlled_round_ratio_judgement(0.85), "between");
    assert_eq!(controlled_round_ratio_judgement(0.80), "between");
    assert_eq!(controlled_round_ratio_judgement(0.79), "drift_remains");
    assert_eq!(controlled_round_ratio_judgement(1.26), "drift_remains");
    assert_eq!(controlled_round_ratio_judgement(1.2), "between");
}

#[test]
fn reproduction_round_ratio_judgement_reproduces_at_or_below_point_eight() {
    assert_eq!(
        reproduction_round_ratio_judgement(0.528),
        "drift_reproduced"
    );
    assert_eq!(reproduction_round_ratio_judgement(0.80), "drift_reproduced");
    assert_eq!(reproduction_round_ratio_judgement(0.85), "between");
    assert_eq!(
        reproduction_round_ratio_judgement(0.958),
        "drift_not_reproduced"
    );
    assert_eq!(
        reproduction_round_ratio_judgement(1.2),
        "above_the_registered_bands"
    );
}

#[test]
fn unstable_ratio_judgement_handles_both_zero_and_the_two_edges() {
    assert_eq!(unstable_ratio_judgement(0, 0), (None, "both_zero"));
    assert_eq!(
        unstable_ratio_judgement(3, 2),
        (Some(1.5), "card_zero_not_less_stable")
    );
    assert_eq!(
        unstable_ratio_judgement(4, 2),
        (Some(2.0), "card_zero_less_stable")
    );
    assert_eq!(unstable_ratio_judgement(7, 4), (Some(1.75), "between"));
    assert_eq!(unstable_ratio_judgement(3, 0).1, "card_zero_less_stable");
}

/// 一行合成的控漂移段点行（候选 A、换到本机的门槛）。
fn drift_point_line(
    arm: DriftArm,
    card: u32,
    clock: &str,
    per_round: &str,
    judgement: &str,
) -> BTreeMap<String, String> {
    fields_of_a_line(&format!(
        "name=drift_control_point arm={} index={card} candidate=scan_checksum_only threshold_kind=machine_scaled clock={clock} host_cpu_per_submission_milliseconds_per_round={per_round} judgement={judgement}",
        arm.label()
    ))
}

#[test]
fn round_ratios_take_the_fifth_round_over_the_first() {
    let lines = vec![drift_point_line(
        DriftArm::Controlled,
        0,
        "thread",
        "10,9,8,7,5",
        "losing",
    )];
    let ratios = round_ratios_of_the_point_lines(&lines, DriftArm::Controlled, 0);
    assert_eq!(ratios.lines, 1);
    assert_eq!(ratios.round_five(), Some(0.5));
    assert_eq!(
        ratios.median_over_round_one,
        [Some(0.9), Some(0.8), Some(0.7), Some(0.5)]
    );
    assert_eq!(ratios.trajectory(), (Some(1.0), 3, Some(0.5)));
}

#[test]
fn round_ratios_count_only_the_thread_clock_lines_of_the_arm_and_the_card() {
    let lines = vec![
        drift_point_line(DriftArm::Controlled, 0, "thread", "4,4,4,4,4", "losing"),
        drift_point_line(DriftArm::Controlled, 0, "process", "4,4,4,4,2", "losing"),
        drift_point_line(DriftArm::Reproduction, 0, "thread", "4,4,4,4,1", "losing"),
        drift_point_line(DriftArm::Controlled, 1, "thread", "4,4,4,4,1", "losing"),
        drift_point_line(DriftArm::Controlled, 0, "thread", "none,4,4,4,4", "losing"),
    ];
    let ratios = round_ratios_of_the_point_lines(&lines, DriftArm::Controlled, 0);
    assert_eq!(ratios.lines, 2, "进程钟、别的臂、别的卡都不收");
    assert_eq!(
        ratios.round_five(),
        Some(1.0),
        "头一轮读不到的那一行不算进比"
    );
}

#[test]
fn unstable_count_takes_only_the_unstable_judgements_of_the_arm_and_the_card() {
    let lines = vec![
        drift_point_line(DriftArm::Controlled, 0, "thread", "1,1,1,1,1", "unstable"),
        drift_point_line(DriftArm::Controlled, 0, "process", "1,1,1,1,1", "unstable"),
        drift_point_line(DriftArm::Controlled, 0, "thread", "1,1,1,1,1", "not_run"),
        drift_point_line(
            DriftArm::Controlled,
            0,
            "thread",
            "1,1,1,1,1",
            "rounds_insufficient",
        ),
        drift_point_line(DriftArm::Controlled, 0, "thread", "1,1,1,1,1", "losing"),
        drift_point_line(DriftArm::Controlled, 1, "thread", "1,1,1,1,1", "unstable"),
    ];
    assert_eq!(unstable_count(&lines, DriftArm::Controlled, 0), (5, 2));
    assert_eq!(unstable_count(&lines, DriftArm::Reproduction, 0), (0, 0));
}

#[test]
fn card_order_effect_compares_the_rounds_measured_second_with_those_measured_first() {
    let lines = vec![
        drift_point_line(
            DriftArm::Controlled,
            0,
            "thread",
            "10,20,10,20,10",
            "losing",
        ),
        drift_point_line(
            DriftArm::Controlled,
            1,
            "thread",
            "10,20,10,20,10",
            "losing",
        ),
    ];
    assert_eq!(
        card_order_effect(&lines, &[0, 1], 0),
        (Some(10.0), Some(20.0), Some(2.0))
    );
    assert_eq!(
        card_order_effect(&lines, &[0, 1], 1),
        (Some(20.0), Some(10.0), Some(0.5))
    );
}

#[test]
fn rank_correlation_is_one_for_the_same_order_and_minus_one_for_the_reverse() {
    let rising = [(1.0, 10.0), (2.0, 20.0), (3.0, 35.0), (4.0, 36.0)];
    assert_eq!(rank_correlation(&rising), Some(1.0));
    let falling = [(1.0, 36.0), (2.0, 35.0), (3.0, 20.0), (4.0, 10.0)];
    assert_eq!(rank_correlation(&falling), Some(-1.0));
    assert_eq!(rank_correlation(&rising[..2]), None);
    assert_eq!(
        rank_correlation(&[(1.0, 5.0), (2.0, 5.0), (3.0, 5.0)]),
        None
    );
}

#[test]
fn card_clock_readings_parse_the_eight_query_columns_and_mark_unreadable_ones() {
    let readings = card_clock_readings_of(
        "0, 00000000:01:00.0, 2805, 14001, P1, 45, 120.50, 0x0000000000000000\n1, 00000000:02:00.0, [N/A], 9001, P2, 40, 80, 0x0000000000000001\n2, 00000000:03:00.0, 1, 2, P8, 3\n",
    );
    assert_eq!(readings.len(), 2, "栏数不对的那一行不收");
    let card_zero = &readings[&0];
    assert_eq!(card_zero.bus_identifier, "00000000:01:00.0");
    assert_eq!(card_zero.sm_clock_megahertz, Some(2805.0));
    assert_eq!(card_zero.memory_clock_megahertz, Some(14001.0));
    assert_eq!(card_zero.performance_state.as_deref(), Some("P1"));
    assert_eq!(card_zero.temperature_celsius, Some(45.0));
    assert_eq!(card_zero.power_watts, Some(120.5));
    assert_eq!(
        card_zero.clocks_event_reasons.as_deref(),
        Some("0x0000000000000000")
    );
    assert_eq!(readings[&1].sm_clock_megahertz, None);
    assert_eq!(CARD_QUERY_FIELDS.split(',').count(), 8);
}

#[test]
fn preferred_core_takes_the_highest_ranking_then_the_lowest_number() {
    let rankings = [(0, Some(200)), (3, Some(231)), (5, Some(231)), (7, None)];
    assert_eq!(preferred_core(&rankings, &[0, 3, 5, 7]), Some(3));
    assert_eq!(preferred_core(&rankings, &[0, 5, 7]), Some(5));
    assert_eq!(preferred_core(&[(4, None), (2, None)], &[0, 4, 2]), Some(2));
    assert_eq!(preferred_core(&[], &[0]), Some(0));
    assert_eq!(preferred_core(&[], &[]), None);
}

/// 区间列表 `0-3,8,10-11`（行尾带换行，与内核文件一样）拆成三段闭区间：单个核是只含它的区间，「起点-终点」两头都算。
#[test]
fn range_list_splits_into_single_cores_and_closed_ranges() {
    assert_eq!(
        core_ranges_of_the_range_list("0-3,8,10-11\n"),
        Some(vec![0..=3, 8..=8, 10..=11])
    );
    let only_core_five = vec![5..=5_usize; 1];
    assert_eq!(core_ranges_of_the_range_list("5"), Some(only_core_five));
}

/// 认不得的区间列表交回 `None`：起点比终点大、不是数、空串。
#[test]
fn range_list_with_a_part_that_is_not_a_core_or_a_range_reads_as_none() {
    assert_eq!(core_ranges_of_the_range_list("3-1"), None, "起点比终点大");
    assert_eq!(core_ranges_of_the_range_list("0-x"), None, "终点不是数");
    assert_eq!(
        core_ranges_of_the_range_list("0,,2"),
        None,
        "两个逗号之间是空的"
    );
    assert_eq!(core_ranges_of_the_range_list(""), None, "空串");
}

/// 允许的核是 `status` 里 `Cpus_allowed_list` 那一行（不是十六进制的 `Cpus_allowed` 那一行）与在线核求交：
/// 允许 `0-3,8,10-11`、在线 `0-2,8-31`，下线的 3 号核不算，交出来从小到大。
#[test]
fn allowed_cores_are_the_allowed_list_of_the_status_intersected_with_the_online_cores() {
    let status_text =
        "Name:\te185\nCpus_allowed:\td0f\nCpus_allowed_list:\t0-3,8,10-11\nMems_allowed_list:\t0\n";
    let allowed_core_list =
        allowed_core_list_of_the_status(status_text).expect("status 里有 Cpus_allowed_list 一行");
    assert_eq!(allowed_core_list.trim(), "0-3,8,10-11");
    assert_eq!(
        cores_in_both_range_lists(allowed_core_list, "0-2,8-31\n"),
        Some(vec![0, 1, 2, 8, 10, 11])
    );
    assert_eq!(
        cores_in_both_range_lists("10-11,0-1", "0-31"),
        Some(vec![0, 1, 10, 11]),
        "原文次序乱的也交从小到大"
    );
    assert_eq!(
        allowed_core_list_of_the_status("Name:\te185\nCpus_allowed:\tff\n"),
        None,
        "只有十六进制那一行的不当读到"
    );
}

#[test]
fn controlled_arm_pins_its_thread_to_a_single_core() {
    let (placement, allowed_after) = std::thread::spawn(|| {
        let placement = place_the_thread_of_the_arm(DriftArm::Controlled);
        (placement, allowed_cores_of_this_thread())
    })
    .join()
    .expect("绑核的那条线程跑完");
    let pinned = placement.pinned_core.expect("允许集合读得到、绑得上");
    assert_eq!(allowed_after, Some(vec![pinned]), "绑完只剩那一个核");
    assert_eq!(placement.allowed_cores, Some(vec![pinned]));
}

#[test]
fn reproduction_arm_leaves_the_affinity_of_its_thread_as_it_was() {
    let (before, placement, after) = std::thread::spawn(|| {
        let before = allowed_cores_of_this_thread();
        let placement = place_the_thread_of_the_arm(DriftArm::Reproduction);
        (before, placement, allowed_cores_of_this_thread())
    })
    .join()
    .expect("复现臂的那条线程跑完");
    assert_eq!(placement.pinned_core, None);
    assert!(
        before.as_ref().is_some_and(|cores| cores.len() > 1),
        "这台机器允许不止一个核：{before:?}"
    );
    assert_eq!(after, before, "复现臂不绑核");
}

/// 控漂移段一格一轮的假卡：提交步骤与钟是第 3 次重跑单测那一套假的（每步 0.1 ms），校准比恒 1、线程数恒 1。
struct FakeCellEnvironment {
    clock: Rc<RefCell<FakeClockState>>,
    steps: Option<FakeSteps>,
}

impl CellRoundEnvironment for FakeCellEnvironment {
    fn card_index(&self) -> u32 {
        0
    }

    fn link_peak_bytes_per_second(&self) -> f64 {
        1e12
    }

    fn host_core_reading(&mut self) -> HostCoreReading {
        HostCoreReading {
            core: Some(3),
            scaling_current_frequency_kilohertz: Some(5_000_000),
            average_frequency_kilohertz: Some(4_900_000),
        }
    }

    fn calibration_ratio(&mut self) -> Option<f64> {
        Some(1.0)
    }

    fn threads(&mut self) -> Option<usize> {
        Some(1)
    }

    fn open_the_cell(&mut self, _cell: SubmissionCell) {
        self.steps = Some(FakeSteps {
            clock: Rc::clone(&self.clock),
            preparation_nanoseconds: 100_000,
            wait_nanoseconds: 100_000,
            other_nanoseconds: 100_000,
        });
    }

    fn time_the_submissions(
        &mut self,
        count: usize,
        gap: Duration,
        insertion: &mut dyn FnMut(),
    ) -> Vec<Option<SubmissionTiming>> {
        let steps = self.steps.as_mut().expect("开格之后才计时");
        let mut clocks = FakeClocks(Rc::clone(&self.clock));
        let mut no_sleep = |_duration: Duration| {};
        time_the_submissions_of_a_cell(steps, &mut clocks, count, gap, &mut no_sleep, insertion)
    }

    fn close_the_cell(&mut self) {
        self.steps = None;
    }
}

/// 假的读卡：每读一次线程钟与挂钟各走 50 ms（读数要是挪进了计时区间，每次提交都会多出这 50 ms）。
struct FakeCardReader {
    clock: Rc<RefCell<FakeClockState>>,
    reads: usize,
}

impl CardReader for FakeCardReader {
    fn wait_for_the_card_and_read_the_cards(
        &mut self,
        card_index: u32,
        _emitter: &mut Emitter,
    ) -> BTreeMap<u32, CardClockReading> {
        self.reads += 1;
        let mut clock = self.clock.borrow_mut();
        clock.thread_cpu += 50_000_000;
        clock.wall += 50_000_000;
        BTreeMap::from([(
            card_index,
            CardClockReading {
                bus_identifier: "00000000:01:00.0".to_string(),
                sm_clock_megahertz: Some(2805.0),
                memory_clock_megahertz: Some(14001.0),
                performance_state: Some("P1".to_string()),
                temperature_celsius: Some(45.0),
                power_watts: Some(120.5),
                clocks_event_reasons: None,
            },
        )])
    }
}

/// 在假卡上量一格一轮（n = 8 的探针那一格，不带探针的自旋），交回量出来的那一次与读卡的次数。
fn measure_a_fake_cell_round(arm: DriftArm, emitter: &mut Emitter) -> (CellRoundOutcome, usize) {
    let clock = Rc::new(RefCell::new(FakeClockState::default()));
    let mut environment = FakeCellEnvironment {
        clock: Rc::clone(&clock),
        steps: None,
    };
    let mut card_reader = FakeCardReader {
        clock: Rc::clone(&clock),
        reads: 0,
    };
    let mut waiting = WaitingLedger {
        total_seconds: 0,
        longest_seconds: 0,
        waits: 0,
    };
    let outcome = measure_one_cell_round(
        &mut environment,
        &mut card_reader,
        CellRoundRequest {
            arm,
            cell: probe_cell(),
            round: 1,
            probe: None,
            threshold_edges: (0.1, 1.0),
        },
        &mut CardTallies::default(),
        &mut waiting,
        &mut Vec::new(),
        emitter,
    )
    .expect("假钟里线程钟不超过挂钟");
    (outcome, card_reader.reads)
}

#[test]
fn drift_cell_round_reads_the_cards_outside_the_timed_submissions() {
    let mut emitter = Emitter::new();
    let (outcome, reads) = measure_a_fake_cell_round(DriftArm::Reproduction, &mut emitter);
    assert_eq!(reads, 1, "开格前读一次卡");
    let medians = outcome.medians.expect("一次就收下");
    assert_eq!(
        medians.thread_cpu_nanoseconds, 600_000.0,
        "一次提交 6 步各 0.1 ms，读卡那 50 ms 不在里面"
    );
    assert_eq!(outcome.warm_up_submissions, WARM_UP_SUBMISSIONS);
    assert_eq!(outcome.cards_before[&0].sm_clock_megahertz, Some(2805.0));
    assert_eq!(outcome.host_before.core, Some(3));
}

#[test]
fn controlled_arm_cell_round_warms_up_until_two_groups_agree() {
    let mut emitter = Emitter::new();
    let (outcome, _reads) = measure_a_fake_cell_round(DriftArm::Controlled, &mut emitter);
    assert_eq!(
        outcome.warm_up_submissions,
        WARM_UP_SUBMISSIONS + CELL_WARM_UP_GROUP_SUBMISSIONS,
        "假提交每次一样快：头 10 次之后再一组就相邻两组相同"
    );
}

#[test]
fn drift_round_has_eighty_cells_and_two_probes() {
    assert_eq!(
        submission_cells_of_a_card().len() + DriftProbe::ALL.len(),
        CELLS_AND_PROBES_PER_ROUND
    );
    assert_eq!(CELLS_AND_PROBES_PER_ROUND, 82);
}

#[test]
fn round_ratio_point_lines_per_card_are_twelve_points_times_nine_resident_cells() {
    let resident_cells_of_one_size = submission_cells_of_a_card()
        .iter()
        .filter(|cell| {
            cell.units == SUBMISSION_UNIT_COUNTS[0]
                && cell.form != SubmissionForm::TodaySingleKernel
        })
        .count();
    assert_eq!(
        every_submission_point().len() * resident_cells_of_one_size,
        POINT_LINES_OF_THE_ROUND_RATIO_PER_CARD
    );
    assert_eq!(POINT_LINES_OF_THE_ROUND_RATIO_PER_CARD, 108);
}

#[test]
fn the_failing_drift_probe_voids_the_round_ratio_and_an_unread_one_is_not_run() {
    assert_eq!(
        drift_round_ratio_judgement(Some(0.95), false, controlled_round_ratio_judgement),
        (Some(0.95), "drift_gone")
    );
    assert_eq!(
        drift_round_ratio_judgement(Some(0.95), true, controlled_round_ratio_judgement),
        (Some(0.95), "void_positive_control")
    );
    assert_eq!(
        drift_round_ratio_judgement(None, false, reproduction_round_ratio_judgement),
        (None, "not_run")
    );
}

#[test]
fn controlled_run_drifting_failure_clause_needs_card_one_drifting_or_both_warm_ups_at_the_cap() {
    assert!(is_the_controlled_run_drifting_failure_clause_triggered(
        Some("drift_remains"),
        &[false, false]
    ));
    assert!(is_the_controlled_run_drifting_failure_clause_triggered(
        Some("drift_gone"),
        &[true, true]
    ));
    assert!(!is_the_controlled_run_drifting_failure_clause_triggered(
        Some("drift_gone"),
        &[true, false]
    ));
    assert!(!is_the_controlled_run_drifting_failure_clause_triggered(
        Some("between"),
        &[true]
    ));
    assert!(!is_the_controlled_run_drifting_failure_clause_triggered(
        None,
        &[false, false]
    ));
}

#[test]
fn drift_not_reproduced_failure_clause_fires_only_when_the_reproduction_arm_does_not_reproduce() {
    assert!(is_the_drift_not_reproduced_failure_clause_triggered(Some(
        "drift_not_reproduced"
    )));
    assert!(!is_the_drift_not_reproduced_failure_clause_triggered(Some(
        "drift_reproduced"
    )));
    assert!(!is_the_drift_not_reproduced_failure_clause_triggered(Some(
        "between"
    )));
    assert!(!is_the_drift_not_reproduced_failure_clause_triggered(None));
}
