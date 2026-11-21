use super::*;

fn fake_steps_and_clocks(
    preparation_nanoseconds: u64,
    wait_nanoseconds: u64,
) -> (FakeSteps, FakeClocks, Rc<RefCell<FakeClockState>>) {
    let clock = Rc::new(RefCell::new(FakeClockState::default()));
    (
        FakeSteps {
            clock: Rc::clone(&clock),
            preparation_nanoseconds,
            wait_nanoseconds,
            other_nanoseconds: 100_000,
        },
        FakeClocks(Rc::clone(&clock)),
        clock,
    )
}

#[test]
fn submission_interval_includes_a_wait_of_three_milliseconds() {
    let (mut steps, mut clocks, _clock) = fake_steps_and_clocks(100_000, 3_000_000);
    let timing = time_one_submission(&mut steps, &mut clocks, &mut || {}).expect("假钟读得到");
    assert_eq!(timing.thread_cpu_nanoseconds, 3_000_000 + 5 * 100_000);
    assert_eq!(timing.wait_thread_cpu_nanoseconds, Some(3_000_000));
    assert!(timing.wait_thread_cpu_nanoseconds <= Some(timing.thread_cpu_nanoseconds));
    assert_eq!(timing.floor_thread_cpu_nanoseconds(), Some(5 * 100_000));
    assert_eq!(timing.input_bytes_written, 8 * LOCAL_GPU_SLOT_BYTES);
}

#[test]
fn submission_interval_includes_a_preparation_of_one_millisecond_and_the_insertion_before_it() {
    let (mut steps, mut clocks, clock) = fake_steps_and_clocks(1_000_000, 100_000);
    let mut insertion = || {
        let mut state = clock.borrow_mut();
        state.thread_cpu += 2_000_000;
        state.wall += 2_000_000;
    };
    let timing = time_one_submission(&mut steps, &mut clocks, &mut insertion).expect("假钟读得到");
    assert_eq!(
        timing.thread_cpu_nanoseconds,
        2_000_000 + 1_000_000 + 5 * 100_000
    );
    assert_eq!(timing.wall_nanoseconds, 2_000_000 + 1_000_000 + 5 * 100_000);
}

#[test]
fn gap_between_submissions_stays_outside_the_interval() {
    let (mut steps, mut clocks, clock) = fake_steps_and_clocks(100_000, 100_000);
    let mut sleep = |duration: Duration| {
        clock.borrow_mut().wall += u64::try_from(duration.as_nanos()).expect("纳秒");
    };
    let timings = time_the_submissions_of_a_cell(
        &mut steps,
        &mut clocks,
        5,
        Duration::from_millis(GAP_BETWEEN_TWO_BATCHES_MILLISECONDS),
        &mut sleep,
        &mut || {},
    );
    let reading = cell_round_reading(&timings, 8).expect("5 次都读得到");
    assert_eq!(reading.wall_median_nanoseconds, 600_000.0);
    assert!(reading.wall_median_nanoseconds < 50_000_000.0);
    assert_eq!(reading.thread_cpu_in_order.len(), 5);
}

#[test]
fn sleeping_wait_sleeps_once_for_every_poll_that_found_nothing() {
    let polls_seen = std::cell::Cell::new(0_u64);
    let sleeps = std::cell::Cell::new(0_u64);
    let polls = wait_by_sleeping(
        || {
            polls_seen.set(polls_seen.get() + 1);
            polls_seen.get() >= 6
        },
        |_interval| sleeps.set(sleeps.get() + 1),
        Duration::from_nanos(SHORT_SLEEP_INTERVAL_NANOSECONDS),
    );
    assert_eq!(polls, 6);
    assert_eq!(sleeps.get(), 5);
    assert_eq!(WaitMethod::Blocking.sleep_interval(), None);
    assert_eq!(
        WaitMethod::SleepingLonger.sleep_interval(),
        Some(Duration::from_millis(1))
    );
}

#[test]
fn sleep_intervals_of_the_configuration_are_one_hundred_microseconds_and_one_millisecond() {
    assert_eq!(
        sleep_interval_configuration_text(),
        "sleep_interval_nanoseconds=100000,1000000"
    );
}

#[test]
fn split_input_writes_the_unit_bytes_into_both_kernel_buffers() {
    let unit_bytes = vec![7_u8; usize::try_from(8 * LOCAL_GPU_SLOT_BYTES).expect("字节")];
    for (form, expected_buffers) in [
        (InputForm::Shared, vec![InputBufferOfTheKernel::KernelOne]),
        (
            InputForm::Split,
            vec![
                InputBufferOfTheKernel::KernelOne,
                InputBufferOfTheKernel::KernelTwo,
            ],
        ),
    ] {
        let mut written: Vec<(InputBufferOfTheKernel, usize)> = Vec::new();
        let bytes = write_the_unit_bytes(form, &unit_bytes, |buffer, bytes| {
            written.push((buffer, bytes.len()))
        });
        assert_eq!(bytes, expected_input_bytes(form, 8), "{form:?}");
        assert_eq!(
            written
                .iter()
                .map(|(buffer, _)| *buffer)
                .collect::<Vec<_>>(),
            expected_buffers,
            "{form:?}"
        );
        assert!(written
            .iter()
            .all(|(_, length)| *length == unit_bytes.len()));
    }
    assert_eq!(expected_input_bytes(InputForm::Split, 512), 33_554_432);
    assert_eq!(expected_input_bytes(InputForm::Shared, 512), 16_777_216);
}

/// 回读缓冲：前 n 条核一、后 n 条核二，每条（CRC、序号）小端。
fn readback_of(kernel_one: &[(u32, u32)], kernel_two: &[(u32, u32)]) -> Vec<u8> {
    kernel_one
        .iter()
        .chain(kernel_two)
        .flat_map(|(crc, sequence)| crc.to_le_bytes().into_iter().chain(sequence.to_le_bytes()))
        .collect()
}

#[test]
fn comparison_reports_exactly_the_flipped_entry_as_uncertain() {
    let expected: Vec<u32> = (0..8).map(|entry| 0x1000 + entry).collect();
    let entries: Vec<(u32, u32)> = expected
        .iter()
        .zip(0_u32..)
        .map(|(crc, entry)| (*crc, entry))
        .collect();
    let readback = readback_of(&entries, &entries);
    let clean = compare_the_readback(&readback, &expected, None);
    assert_eq!(clean.confirmed, 8);
    assert!(clean.uncertain_entries.is_empty());
    let flipped = compare_the_readback(
        &readback,
        &expected,
        Some(READBACK_ENTRY_FLIPPED_BY_THE_COMPARISON_CONTROL),
    );
    assert_eq!(flipped.confirmed, 7);
    assert_eq!(flipped.uncertain_entries, vec![2]);
    assert_eq!(flipped.kernel_one_differs_from_the_expected, 0);
    assert_eq!(flipped.kernel_two_differs_from_the_expected, 1);
    assert!(comparison_control_holds(&flipped.uncertain_entries));
    assert!(!comparison_control_holds(&clean.uncertain_entries));
    assert!(!comparison_control_holds(&[2, 3]));
}

#[test]
fn comparison_checks_the_sequence_numbers_of_both_kernels() {
    let expected = vec![5_u32; 4];
    let good: Vec<(u32, u32)> = (0..4).map(|entry| (5, entry)).collect();
    let mut kernel_two_wrong = good.clone();
    kernel_two_wrong[1].1 = 9;
    assert!(
        compare_the_readback(&readback_of(&good, &good), &expected, None)
            .are_the_sequence_numbers_holding
    );
    assert!(
        !compare_the_readback(&readback_of(&good, &kernel_two_wrong), &expected, None)
            .are_the_sequence_numbers_holding
    );
    assert!(
        !compare_the_readback(&readback_of(&kernel_two_wrong, &good), &expected, None)
            .are_the_sequence_numbers_holding
    );
}

#[test]
fn submission_sequence_at_a_quarter_and_four_issues_once_and_scans_what_is_left() {
    let sequence =
        submission_units_of_the_intent(GpuCandidate::EveryStage, LiveSlotFraction::OneQuarter, 4);
    assert_eq!(sequence, vec![8, 8, 4]);
    assert_eq!(sequence.len(), 3);
}

#[test]
fn submission_sequence_scans_what_is_left_in_the_region_not_what_the_batch_moves() {
    let sequence = submission_units_of_the_intent(
        GpuCandidate::ScanChecksumOnly,
        LiveSlotFraction::ThreeQuarters,
        8,
    );
    assert_eq!(sequence, vec![24, 24, 16, 8]);
    assert_eq!(mean_submission_tenths_of_bytes(&sequence), 5_898_240);
}

#[test]
fn release_check_sequence_has_no_issue_submission() {
    assert_eq!(
        submission_units_of_the_intent(
            GpuCandidate::ReleaseCheckOnly,
            LiveSlotFraction::ThreeQuarters,
            16
        ),
        vec![32, 16]
    );
    assert_eq!(
        submission_units_of_the_intent(
            GpuCandidate::ReleaseCheckOnly,
            LiveSlotFraction::ThreeQuarters,
            4
        ),
        vec![8; 6]
    );
}

#[test]
fn sequence_anchors_hold_for_every_fraction_and_batch_size() {
    for (fraction, batch_units, submissions, mean_tenths, release_submissions) in
        SUBMISSION_SEQUENCE_ANCHORS
    {
        let every_stage =
            submission_units_of_the_intent(GpuCandidate::EveryStage, fraction, batch_units);
        let release =
            submission_units_of_the_intent(GpuCandidate::ReleaseCheckOnly, fraction, batch_units);
        assert_eq!(
            every_stage.len(),
            usize::try_from(submissions).expect("次数"),
            "{fraction:?} {batch_units}"
        );
        assert_eq!(
            mean_submission_tenths_of_bytes(&every_stage),
            mean_tenths,
            "{fraction:?} {batch_units}"
        );
        assert_eq!(
            release.len(),
            usize::try_from(release_submissions).expect("次数"),
            "{fraction:?} {batch_units}"
        );
    }
    assert_eq!(every_submission_point().len(), 12);
    assert_eq!(
        units_of_one_submission_per_intent(&submission_units_of_the_intent(
            GpuCandidate::EveryStage,
            LiveSlotFraction::ThreeQuarters,
            8
        )),
        24
    );
}

#[test]
fn point_milliseconds_average_the_submissions_rather_than_take_the_median() {
    let sequence = submission_units_of_the_intent(
        GpuCandidate::EveryStage,
        LiveSlotFraction::ThreeQuarters,
        4,
    );
    let by_units: BTreeMap<u64, f64> = sequence
        .iter()
        .map(|units| (*units, 0.01 * *units as f64))
        .collect();
    let point = point_milliseconds_of_the_sequence(&sequence, &by_units).expect("每档都有");
    assert!((point - 0.01 * 108.0 / 7.0).abs() < 1e-12, "{point}");
    assert_eq!(
        point_milliseconds_of_the_sequence(&sequence, &BTreeMap::new()),
        None
    );
}

#[test]
fn host_cpu_equal_to_the_threshold_is_losing_and_moving_the_threshold_between_two_values_flips_it()
{
    assert_eq!(
        judgement_against_the_threshold(0.5, 0.5, GpuCandidate::EveryStage),
        "losing"
    );
    assert_eq!(
        judgement_against_the_threshold(0.4999, 0.5, GpuCandidate::EveryStage),
        "not_losing"
    );
    assert_eq!(
        judgement_against_the_threshold(0.3, 0.35, GpuCandidate::ReleaseCheckOnly),
        "not_losing"
    );
    assert_eq!(
        judgement_against_the_threshold(0.4, 0.35, GpuCandidate::ReleaseCheckOnly),
        "losing"
    );
}

#[test]
fn choice_candidate_not_losing_is_written_as_a_lower_bound() {
    assert_eq!(
        judgement_against_the_threshold(0.1, 0.5, GpuCandidate::ScanChecksumAndChoice),
        "not_losing_lower_bound"
    );
    assert_eq!(
        judgement_against_the_threshold(0.1, 0.5, GpuCandidate::ScanChecksumOnly),
        "not_losing"
    );
    assert_eq!(
        judgement_against_the_threshold(0.9, 0.5, GpuCandidate::ScanChecksumAndChoice),
        "losing"
    );
}

#[test]
fn judgement_of_the_valid_rounds_writes_rounds_insufficient_for_a_voided_round() {
    assert_eq!(
        judgement_of_the_valid_rounds(&[Some("losing"); ROUNDS]),
        "losing"
    );
    let mut mixed = [Some("losing"); ROUNDS];
    mixed[2] = Some("not_losing");
    assert_eq!(judgement_of_the_valid_rounds(&mixed), "unstable");
    let mut voided = [Some("losing"); ROUNDS];
    voided[4] = None;
    assert_eq!(
        judgement_of_the_valid_rounds(&voided),
        "rounds_insufficient"
    );
    assert_eq!(
        judgement_of_the_valid_rounds(&[Some("losing"); 4]),
        "rounds_insufficient"
    );
}

/// 合成的 r2 第二段 `name=stage` 行：（轮、签发不、crc、乙增量、丙、释放前核）。
fn ledger_row(
    round: &str,
    is_the_issue: bool,
    numbers: (f64, f64, Option<f64>, f64),
) -> StageLedgerRow {
    StageLedgerRow {
        intent_key: (
            "0.25".to_string(),
            "1024".to_string(),
            "4".to_string(),
            round.to_string(),
        ),
        is_the_issue,
        call_cpu_nanoseconds: 10_000_000.0,
        crc_cpu_nanoseconds: numbers.0,
        scan_increment_cpu_nanoseconds: numbers.1,
        choose_cpu_nanoseconds: numbers.2,
        release_check_crc_cpu_nanoseconds: numbers.3,
    }
}

fn point_key() -> (String, String, String) {
    ("0.25".to_string(), "1024".to_string(), "4".to_string())
}

#[test]
fn ledger_groups_by_round_and_takes_the_median_of_the_rounds() {
    let rows = vec![
        ledger_row("1", true, (1_000_000.0, 0.0, Some(0.0), 80.0)),
        ledger_row("1", false, (100_000.0, 0.0, None, 0.0)),
        ledger_row("1", false, (100_000.0, 0.0, None, 0.0)),
        ledger_row("2", true, (3_000_000.0, 0.0, Some(0.0), 80.0)),
        ledger_row("2", false, (300_000.0, 0.0, None, 0.0)),
        ledger_row("2", false, (300_000.0, 0.0, None, 0.0)),
    ];
    let thresholds = candidate_thresholds_of_the_ledger(&rows).expect("读得成");
    let scan_checksum = thresholds[&point_key()][&GpuCandidate::ScanChecksumOnly];
    assert!(
        (scan_checksum.saved_per_intent_milliseconds - 2.4).abs() < 1e-12,
        "{scan_checksum:?}"
    );
    assert_eq!(scan_checksum.submissions_per_intent, 3);
    assert!(
        (scan_checksum.threshold_milliseconds - 0.8).abs() < 1e-12,
        "{scan_checksum:?}"
    );
}

#[test]
fn scan_checksum_candidate_saves_only_the_checksum_cpu() {
    let rows = vec![
        ledger_row("1", true, (1_000_000.0, 500_000.0, Some(2_000_000.0), 80.0)),
        ledger_row("1", false, (100_000.0, 50_000.0, None, 70_000.0)),
        ledger_row("1", false, (100_000.0, 50_000.0, None, 70_000.0)),
    ];
    let thresholds = candidate_thresholds_of_the_ledger(&rows).expect("读得成");
    let of = |candidate| thresholds[&point_key()][&candidate];
    assert!((of(GpuCandidate::ScanChecksumOnly).threshold_milliseconds - 0.4).abs() < 1e-12);
    assert!(
        (of(GpuCandidate::ScanChecksumAndChoice).threshold_milliseconds - 3.2 / 3.0).abs() < 1e-12
    );
    assert!((of(GpuCandidate::EveryStage).threshold_milliseconds - 3.8 / 3.0).abs() < 1e-12);
    assert!((of(GpuCandidate::ReleaseCheckOnly).threshold_milliseconds - 0.07).abs() < 1e-12);
    assert_eq!(of(GpuCandidate::ReleaseCheckOnly).submissions_per_intent, 2);
    assert_eq!(of(GpuCandidate::EveryStage).submissions_per_intent, 3);
}

#[test]
fn stage_ledger_rows_keep_rows_of_every_validity() {
    let lines = result_lines_of(
        "E7RESULT name=stage fraction=0.25 pool_data_units=1024 batch_units=4 round=1 call_index=0 kind=issue valid=false call_cpu_nanoseconds=1 crc_cpu_nanoseconds=2.0 scan_increment_cpu_nanoseconds=3.0 choose_cpu_nanoseconds=4.0 release_check_crc_cpu_nanoseconds=5.0\n\
             E7RESULT name=stage fraction=0.25 pool_data_units=1024 batch_units=4 round=1 call_index=1 kind=batch call_cpu_nanoseconds=1 crc_cpu_nanoseconds=2.0 scan_increment_cpu_nanoseconds=3.0 choose_cpu_nanoseconds=none release_check_crc_cpu_nanoseconds=5.0\n\
             E7RESULT name=calibration_baseline segment=stage_split position=start runs=21 median_nanoseconds=5224477.0",
    );
    let rows = stage_ledger_rows_of(&lines).expect("读得成");
    assert_eq!(rows.len(), 2);
    assert!(rows[0].is_the_issue);
    assert_eq!(rows[1].choose_cpu_nanoseconds, None);
    assert_eq!(peer_calibration_baseline_of(&lines), Some(5_224_477.0));
    assert_eq!(peer_calibration_baseline_of(&lines[..2]), None);
}

#[test]
fn ledger_finding_stops_when_the_threshold_differs_at_the_fourth_decimal() {
    // `out/g1.out` 的一行照原样拼（键名是攻方模型脚本定的，照它的写法；拼起来免得字面量里出现它的缩写键名）
    let fields = [
        ["f", "0.25"],
        ["P", "1024"],
        ["K", "4"],
        ["saved_upper_ms", "0.648"],
        ["submissions_per_intent", "3"],
        ["break_even_host_cpu_per_submission_ms", "0.2159"],
    ];
    let line = fields
        .iter()
        .map(|[key, value]| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join(" ");
    let attack_ledger = attack_ledger_points_of(&format!(
        "name=g1_ledger_header columns=none\nname=g1_ledger {line}\n"
    ))
    .expect("读得成");
    let point = SubmissionPoint {
        fraction: LiveSlotFraction::OneQuarter,
        pool_data_units: 1024,
        batch_units: 4,
    };
    let attack_point = &attack_ledger[&point_key()];
    assert_eq!(attack_point.saved_upper_milliseconds, 0.648);
    assert_eq!(attack_point.break_even_milliseconds, 0.2159);
    let threshold = |milliseconds: f64, submissions: u64| CandidateThreshold {
        saved_per_intent_milliseconds: milliseconds * 3.0,
        submissions_per_intent: submissions,
        threshold_milliseconds: milliseconds,
    };
    assert_eq!(
        ledger_finding_of_the_point(point, &threshold(0.21594, 3), attack_point),
        None
    );
    assert!(ledger_finding_of_the_point(point, &threshold(0.21596, 3), attack_point).is_some());
    assert!(ledger_finding_of_the_point(point, &threshold(0.2159, 2), attack_point).is_some());
}

#[test]
fn machine_ratio_is_this_machine_over_the_peer() {
    assert_eq!(machine_ratio(1_000_000.0, 2_000_000.0), 0.5);
    assert!(!is_the_machine_ratio_far_from_one(0.5));
    assert!(is_the_machine_ratio_far_from_one(0.49));
    assert!(is_the_machine_ratio_far_from_one(2.01));
    assert!(!is_the_cross_check_off_by_more_than_double(0.9, 0.5));
    assert!(is_the_cross_check_off_by_more_than_double(1.1, 0.5));
    assert_eq!(
        ThresholdKind::MachineScaled.scaled(0.4, Some(0.5)),
        Some(0.2)
    );
    assert_eq!(ThresholdKind::MachineScaled.scaled(0.4, None), None);
    assert_eq!(ThresholdKind::AttackLedger.scaled(0.4, None), Some(0.4));
}

/// 八档 n 上照 `milliseconds_of(MiB)` 合成的 c_rt。
fn synthetic_milliseconds(milliseconds_of: impl Fn(f64) -> f64) -> BTreeMap<u64, f64> {
    SUBMISSION_UNIT_COUNTS
        .iter()
        .map(|units| {
            (
                *units,
                milliseconds_of((units * LOCAL_GPU_SLOT_BYTES) as f64 / 1_048_576.0),
            )
        })
        .collect()
}

#[test]
fn fit_recovers_the_fixed_and_per_mebibyte_parts_within_one_percent() {
    let fit = fixed_and_per_byte_fit(&synthetic_milliseconds(|mebibytes| 0.1 + 0.2 * mebibytes))
        .expect("八档都有");
    assert!((fit.fixed_milliseconds - 0.1).abs() <= 0.001, "{fit:?}");
    assert!(
        (fit.per_mebibyte_milliseconds - 0.2).abs() <= 0.002,
        "{fit:?}"
    );
    assert!(fit.maximum_relative_residual < 1e-9, "{fit:?}");
    assert!(fit.is_linear());
}

/// 合成的线性拟合：固定 `fixed_milliseconds`，n = 24 那一次的每字节那一部分是 `per_byte_part_at_twenty_four_units`。
fn linear_fit(
    fixed_milliseconds: f64,
    per_byte_part_at_twenty_four_units: f64,
) -> FixedAndPerByteFit {
    let per_byte_milliseconds =
        per_byte_part_at_twenty_four_units / (24 * LOCAL_GPU_SLOT_BYTES) as f64;
    FixedAndPerByteFit {
        fixed_milliseconds,
        per_byte_milliseconds,
        per_mebibyte_milliseconds: per_byte_milliseconds * 1_048_576.0,
        maximum_relative_residual: 0.0,
    }
}

#[test]
fn merging_takes_at_least_the_ceiling_of_the_fixed_part_over_the_margin() {
    let shape = merging_shape(5.0, 1.0, 5.0, 2.0, 24, Some(linear_fit(1.0, 1.7)), true);
    assert_eq!(
        shape,
        Some(MergingShape::MergeAcrossIntents {
            intents_to_merge_at_least: 4.0
        })
    );
}

#[test]
fn merging_shape_gives_each_of_the_four_categories() {
    let fit = Some(linear_fit(1.0, 1.7));
    assert_eq!(
        merging_shape(0.5, 1.0, 5.0, 2.0, 24, fit, true),
        Some(MergingShape::OneSubmissionPerBatch)
    );
    assert_eq!(
        merging_shape(2.0, 1.0, 1.0, 2.0, 24, fit, true),
        Some(MergingShape::OneSubmissionPerIntent)
    );
    assert!(matches!(
        merging_shape(2.0, 1.0, 5.0, 2.0, 24, fit, true),
        Some(MergingShape::MergeAcrossIntents { .. })
    ));
    assert_eq!(
        merging_shape(2.0, 1.0, 5.0, 2.0, 24, Some(linear_fit(1.0, 3.0)), true),
        Some(MergingShape::LosesHoweverMerged)
    );
    assert_eq!(merging_shape(2.0, 1.0, 5.0, 2.0, 24, None, true), None);
}

#[test]
fn merging_shape_refuses_the_last_two_categories_when_the_fit_is_not_linear() {
    let quadratic = fixed_and_per_byte_fit(&synthetic_milliseconds(|mebibytes| {
        0.1 + mebibytes * mebibytes
    }))
    .expect("八档都有");
    assert!(!quadratic.is_linear(), "{quadratic:?}");
    assert_eq!(
        merging_shape(
            5.0,
            1.0,
            5.0,
            2.0,
            24,
            Some(quadratic),
            quadratic.is_linear()
        ),
        Some(MergingShape::FitNotLinear)
    );
    assert_eq!(
        merging_shape(0.5, 1.0, 5.0, 2.0, 24, Some(quadratic), false),
        Some(MergingShape::OneSubmissionPerBatch),
        "前两类不经拟合，照判"
    );
}

#[test]
fn trajectory_counts_the_submissions_at_or_above_each_threshold_edge() {
    let in_order = [300_000.0, 216_000.0, 800_000.0, 100_000.0, 722_000.0];
    let trajectory = submission_trajectory(&in_order, 0.2159, 0.7214).expect("有读数");
    assert_eq!(trajectory.maximum_nanoseconds, 800_000.0);
    assert_eq!(trajectory.at_or_above_the_smallest_threshold, 4);
    assert_eq!(trajectory.at_or_above_the_largest_threshold, 2);
    assert_eq!(trajectory.first_nanoseconds, 300_000.0);
    assert_eq!(trajectory.last_nanoseconds, 722_000.0);
    assert_eq!(trajectory.percentile_90_nanoseconds, 800_000.0);
}

#[test]
fn thread_clock_control_judges_the_spin_and_the_sleep_at_the_registered_edges() {
    assert!(spin_control_holds(1_800_000.0));
    assert!(spin_control_holds(2_200_000.0));
    assert!(!spin_control_holds(1_799_999.0));
    assert!(!spin_control_holds(2_200_001.0));
    assert!(sleep_control_holds(200_000.0, 1_800_000.0));
    assert!(!sleep_control_holds(200_001.0, 1_800_000.0));
    assert!(!sleep_control_holds(100_000.0, 1_799_999.0));
    let baseline = [Some(1.0), Some(2.0), Some(3.0), Some(4.0), Some(5.0)];
    let inserted = [Some(3.0), Some(4.5), Some(5.0), Some(6.0), Some(7.0)];
    assert_eq!(
        median_increment_of_the_rounds(&baseline, &inserted),
        Some(2.0)
    );
    assert_eq!(
        median_increment_of_the_rounds(
            &baseline,
            &[None, Some(1.0), Some(1.0), Some(1.0), Some(1.0)]
        ),
        None
    );
}

#[test]
fn thread_clock_underestimate_triggers_above_ten_percent_and_thread_above_wall_is_outside() {
    assert!(!is_the_thread_clock_underestimating(110.0, 100.0));
    assert!(is_the_thread_clock_underestimating(110.1, 100.0));
    assert!(is_the_thread_cpu_inside_the_wall(100, 100));
    assert!(!is_the_thread_cpu_inside_the_wall(101, 100));
}

#[test]
fn kernel_one_finding_stops_when_one_result_differs_from_the_crates() {
    assert_eq!(
        kernel_one_finding_against_the_crates(3, &[1, 2, 3], &[1, 2, 3]),
        None
    );
    assert!(kernel_one_finding_against_the_crates(3, &[1, 9, 3], &[1, 2, 3]).is_some());
    assert!(kernel_one_finding_against_the_crates(3, &[1, 2], &[1, 2, 3]).is_some());
}

#[test]
fn other_compute_processes_on_the_card_leave_out_this_process_and_other_cards() {
    let buses = bus_identifiers_by_index_of("0, 00000000:01:00.0\n1, 00000000:02:00.0\n");
    assert_eq!(buses.get(&1).map(String::as_str), Some("00000000:02:00.0"));
    let apps = "100, 00000000:01:00.0\n200, 00000000:02:00.0\n300, 00000000:02:00.0\n";
    assert_eq!(
        other_compute_processes_on_the_card(apps, &buses[&1], 300),
        vec![200]
    );
    assert_eq!(
        other_compute_processes_on_the_card(apps, &buses[&0], 100),
        Vec::<u32>::new()
    );
}

#[test]
fn physical_floor_of_split_input_counts_both_copies() {
    let peak = 63_015_384_615.0;
    let one_copy_seconds = 512.0 * LOCAL_GPU_SLOT_BYTES as f64 / peak;
    assert!(respects_the_floor_of_the_form(
        1.5 * one_copy_seconds,
        InputForm::Shared,
        512,
        peak
    ));
    assert!(!respects_the_floor_of_the_form(
        1.5 * one_copy_seconds,
        InputForm::Split,
        512,
        peak
    ));
    assert!(respects_the_floor_of_the_form(
        2.0 * one_copy_seconds,
        InputForm::Split,
        512,
        peak
    ));
}

// ───────────── 第 3 次重跑第二段：注入、重算、计数 ─────────────

#[test]
fn disagreement_trajectory_reports_the_peak_the_batches_with_some_and_the_last_batch() {
    assert_eq!(
        disagreement_trajectory_text(&[0, 3, 0, 1]),
        "first_round_disagreements_peak_of_a_batch=3 batches_with_first_round_disagreements=2 first_round_disagreements_of_the_last_batch=1"
    );
    assert!(disagreement_trajectory_text(&[]).contains("peak_of_a_batch=none"));
}

#[test]
fn natural_unit_times_leave_out_the_sixteen_injected_units() {
    assert_eq!(injected_units_per_combination(), 16);
    assert_eq!(
        natural_unit_times(
            DISAGREEMENT_BATCHES,
            UNITS_PER_DISAGREEMENT_BATCH,
            injected_units_per_combination()
        ),
        1_048_560
    );
    assert!(are_the_natural_unit_times_sufficient(1_048_560));
    assert!(!are_the_natural_unit_times_sufficient(1_048_559));
    assert!(is_an_injected_unit(1, 7) && is_an_injected_unit(8, 11));
    assert!(!is_an_injected_unit(9, 7) && !is_an_injected_unit(1, 8));
}

/// 假卡：单元 `unit` 的真 CRC 是 1000 + 单元号；核一照真值，核二在分开输入的 P5 那个单元上读的是改过的输入（交回不同的值），
/// 只派核一时核二交 `None`。`kernels_both_wrong` 里的单元两核都交同一个错值。
fn fake_kernel_results(
    form: InputForm,
    batch: u64,
    unit: u64,
    dispatched: KernelsToDispatch,
    kernels_both_wrong: &[u64],
) -> KernelResultsOfAUnit {
    let truth = 1000 + u32::try_from(unit).expect("单元号");
    if kernels_both_wrong.contains(&unit) {
        return (Some(truth ^ 0xF0), Some(truth ^ 0xF0));
    }
    let kernel_two = match dispatched {
        KernelsToDispatch::Both => Some(if kernel_two_input_flip(form, batch, unit).is_some() {
            truth ^ 0xABCD
        } else {
            truth
        }),
        KernelsToDispatch::KernelOneOnly => None,
    };
    (Some(truth), kernel_two)
}

fn fake_batch(form: InputForm, batch: u64, kernels_both_wrong: &[u64]) -> DisagreementBatchOutcome {
    let units = 12_u64;
    let cpu_crcs: Vec<u32> = (0..units)
        .map(|unit| 1000 + u32::try_from(unit).expect("单元号"))
        .collect();
    let first_results: Vec<KernelResultsOfAUnit> = (0..units)
        .map(|unit| {
            fake_kernel_results(
                form,
                batch,
                unit,
                KernelsToDispatch::Both,
                kernels_both_wrong,
            )
        })
        .collect();
    process_the_disagreement_batch(
        form,
        batch,
        &cpu_crcs,
        &first_results,
        |unit, dispatched| fake_kernel_results(form, batch, unit, dispatched, kernels_both_wrong),
    )
}

#[test]
fn transient_injection_flips_only_the_first_computation() {
    for form in InputForm::ALL {
        let outcome = fake_batch(form, 1, &[]);
        assert_eq!(
            outcome.transient_injection,
            Some(Some(RecomputeOutcome::Transient)),
            "{form:?}"
        );
        assert_eq!(outcome.natural_units, 10, "{form:?}");
        assert_eq!(outcome.first_round_any_difference, 0, "{form:?}");
        assert!(outcome.natural_recomputes.is_empty(), "{form:?}");
    }
    assert!(is_the_kernel_two_readback_flipped(
        InputForm::Split,
        1,
        11,
        ComputationOfTheUnit::FirstInTheBatch
    ));
    assert!(!is_the_kernel_two_readback_flipped(
        InputForm::Split,
        1,
        11,
        ComputationOfTheUnit::Recompute
    ));
    assert!(!is_the_kernel_two_readback_flipped(
        InputForm::Split,
        9,
        11,
        ComputationOfTheUnit::FirstInTheBatch
    ));
}

#[test]
fn deterministic_injection_stays_deterministic_after_three_recomputes_of_both_kernels() {
    for form in InputForm::ALL {
        let outcome = fake_batch(form, 1, &[]);
        assert_eq!(
            outcome.deterministic_injection,
            Some(Some(RecomputeOutcome::Deterministic)),
            "{form:?}"
        );
    }
    assert_eq!(kernel_two_input_flip(InputForm::Split, 8, 7), Some(100));
    assert_eq!(kernel_two_input_flip(InputForm::Shared, 1, 7), None);
    assert!(is_the_kernel_two_readback_flipped(
        InputForm::Shared,
        1,
        7,
        ComputationOfTheUnit::Recompute
    ));
    let outside = fake_batch(InputForm::Split, 9, &[]);
    assert_eq!(outside.natural_units, 12);
    assert_eq!(outside.deterministic_injection, None);
}

#[test]
fn both_kernels_agreeing_against_the_cpu_are_counted() {
    let outcome = fake_batch(InputForm::Shared, 20, &[3]);
    assert_eq!(outcome.both_kernels_agree_and_differ_from_the_cpu, 1);
    assert_eq!(outcome.first_round_kernel_disagreements, 0);
    assert_eq!(outcome.first_round_any_difference, 1);
    assert_eq!(outcome.kernel_one_differs_from_the_cpu, 1);
    assert_eq!(outcome.natural_recomputes.len(), 1);
    assert_eq!(
        outcome.natural_recomputes[0].outcome,
        RecomputeOutcome::Deterministic
    );
    assert_eq!(still_disagreeing_after(&outcome.natural_recomputes, 3), 1);
}

#[test]
fn upper_bound_at_95_percent_when_zero_matches_the_registered_value() {
    assert_eq!(
        format!("{:.6e}", upper_bound_at_95_percent_when_zero(1_048_560)),
        "2.856992e-6"
    );
    assert_eq!(
        format!("{:.6e}", upper_bound_at_95_percent_when_zero(2_097_120)),
        "1.428497e-6"
    );
}

#[test]
fn recompute_category_is_zero_when_the_first_round_has_no_difference() {
    assert_eq!(recompute_category(0, &[]), "zero");
    assert_eq!(
        recompute_category(
            2,
            &[RecomputeOutcome::Transient, RecomputeOutcome::Transient]
        ),
        "transient_only"
    );
    assert_eq!(
        recompute_category(
            2,
            &[RecomputeOutcome::Transient, RecomputeOutcome::Deterministic]
        ),
        "deterministic_present"
    );
    assert_eq!(
        recompute_category(
            2,
            &[RecomputeOutcome::Transient, RecomputeOutcome::AgreedLater]
        ),
        "mixed"
    );
}

#[test]
fn content_anchors_hold_and_the_seed_includes_the_input_form() {
    assert_eq!(content_anchor_finding(), None);
    let mut bytes = vec![0_u8; disagreement_unit_length(16_383)];
    disagreement_unit_content(1, InputForm::Split, 64, 16_383, &mut bytes);
    assert_eq!(bytes.len(), 16_384);
    assert_eq!(crc32_castagnoli(&bytes), 0x261c_3840);
    let mut shared = vec![0_u8; bytes.len()];
    disagreement_unit_content(1, InputForm::Shared, 64, 16_383, &mut shared);
    assert_ne!(shared, bytes);
}

#[test]
fn still_disagreeing_counts_and_injected_units_hold_only_with_all_eight_outcomes() {
    let differing = UnitComparison {
        kernels_differ: true,
        kernel_one_differs_from_the_cpu: false,
        kernel_two_differs_from_the_cpu: true,
    };
    let differ = Some(differing);
    let agree = UnitComparison::of(Some(1), Some(1), 1);
    let unit = |recomputes: Vec<Option<UnitComparison>>, outcome| RecomputedUnit {
        unit: 0,
        first_round: differing,
        recomputes,
        outcome,
    };
    let units = [
        unit(vec![agree], RecomputeOutcome::Transient),
        unit(vec![differ, agree], RecomputeOutcome::AgreedLater),
        unit(
            vec![differ, differ, differ],
            RecomputeOutcome::Deterministic,
        ),
    ];
    assert_eq!(still_disagreeing_after(&units, 1), 2);
    assert_eq!(still_disagreeing_after(&units, 2), 1);
    assert_eq!(still_disagreeing_after(&units, 3), 1);
    assert!(injected_units_hold(
        &[Some(RecomputeOutcome::Transient); 8],
        RecomputeOutcome::Transient
    ));
    assert!(!injected_units_hold(
        &[Some(RecomputeOutcome::Transient); 7],
        RecomputeOutcome::Transient
    ));
    let mut one_missing = [Some(RecomputeOutcome::Transient); 8];
    one_missing[3] = None;
    assert!(!injected_units_hold(
        &one_missing,
        RecomputeOutcome::Transient
    ));
    assert_eq!(
        identical_entries(
            &[(Some(1), Some(1)), (Some(2), None)],
            &[(Some(1), Some(1)), (Some(2), Some(2))]
        ),
        1
    );
}

// ───────────── 第 3 次重跑第一段：格、点行、拟合与攒批行、旋钮翻面 ─────────────

#[test]
fn submission_cells_of_a_card_are_eighty_in_the_registered_order() {
    let cells = submission_cells_of_a_card();
    assert_eq!(cells.len(), 80);
    let count_of = |predicate: &dyn Fn(&SubmissionCell) -> bool| {
        cells.iter().filter(|cell| predicate(cell)).count()
    };
    assert_eq!(
        count_of(
            &|cell| cell.gap_milliseconds == 0 && matches!(cell.form, SubmissionForm::Resident(_))
        ),
        48
    );
    assert_eq!(
        count_of(&|cell| cell.gap_milliseconds == GAP_BETWEEN_TWO_BATCHES_MILLISECONDS),
        24
    );
    assert_eq!(
        count_of(&|cell| cell.form == SubmissionForm::TodaySingleKernel),
        8
    );
    assert_eq!(
        cells[0].label(),
        "form=shared wait=blocking gap_milliseconds=0 units=4"
    );
    assert_eq!(cells[7].units, 512);
    assert_eq!(cells[8].wait, WaitMethod::SleepingShortly);
    let second_round = submission_cells_in_the_order_of_the_round(2);
    assert_eq!(second_round[0], cells[1]);
    assert_eq!(second_round[79], cells[0]);
    assert_eq!(submission_cells_in_the_order_of_the_round(81)[0], cells[0]);
}

fn quarter_point_at_four() -> SubmissionPoint {
    SubmissionPoint {
        fraction: LiveSlotFraction::OneQuarter,
        pool_data_units: 1024,
        batch_units: 4,
    }
}

/// 一张卡（序号 0）一种形态一种等法间隔 0 的合成中位：n = 4 是 0.6 ms、n = 8 是 0.3 ms，五轮相同（`missing_round` 那一轮没有读数）。
fn synthetic_medians(
    form: SubmissionForm,
    missing_round: Option<usize>,
    process_cpu_nanoseconds: f64,
) -> CellMediansOfTheSegment {
    let mut medians = CellMediansOfTheSegment::new();
    for (units, thread_cpu_nanoseconds) in [(4_u64, 600_000.0), (8, 300_000.0)] {
        let cell = SubmissionCell {
            gap_milliseconds: 0,
            form,
            wait: WaitMethod::Blocking,
            units,
        };
        let rounds = (0..ROUNDS)
            .map(|round_index| {
                (Some(round_index) != missing_round).then_some(CellRoundMedians {
                    thread_cpu_nanoseconds,
                    floor_thread_cpu_nanoseconds: Some(thread_cpu_nanoseconds / 2.0),
                    process_cpu_nanoseconds,
                })
            })
            .collect();
        medians.insert((0, cell), rounds);
    }
    medians
}

#[test]
fn host_cpu_point_lines_judge_each_threshold_kind_and_mark_voided_and_short_rounds() {
    let thresholds = BTreeMap::from([(
        quarter_point_at_four(),
        PointThresholds {
            by_candidate: BTreeMap::from([(GpuCandidate::EveryStage, 0.5)]),
            saved_upper_milliseconds: 1.5,
        },
    )]);
    let shared = SubmissionForm::Resident(InputForm::Shared);
    let medians = synthetic_medians(shared, None, 300_000.0);
    let voiding = BTreeMap::new();
    let process_clock_cells = BTreeSet::new();
    let inputs = HostCpuPointInputs {
        thresholds: &thresholds,
        machine_ratio: Some(0.5),
        cards: &[0],
        medians: &medians,
        voiding: &voiding,
        process_clock_cells: &process_clock_cells,
    };
    let (lines, judgements, floor_judgements) = host_cpu_point_lines(&inputs);
    let key = |kind| {
        (
            quarter_point_at_four(),
            GpuCandidate::EveryStage,
            0,
            shared,
            WaitMethod::Blocking,
            0,
            kind,
            CpuClockKind::Thread,
        )
    };
    assert_eq!(judgements[&key(ThresholdKind::AttackLedger)], "not_losing");
    assert_eq!(judgements[&key(ThresholdKind::MachineScaled)], "losing");
    assert_eq!(
        floor_judgements[&key(ThresholdKind::MachineScaled)],
        "not_losing"
    );
    let attack_line = lines
        .iter()
        .find(|line| {
            line.contains("form=shared wait=blocking")
                && line.contains("threshold_kind=attack_ledger")
        })
        .expect("有这一行");
    assert!(attack_line.contains("submission_units=8,8,4 host_cpu_per_submission_milliseconds_per_round=0.400000,0.400000,0.400000,0.400000,0.400000"), "{attack_line}");
    let flips = knob_flip_lines("host_cpu_per_submission", &judgements, &[0]);
    assert!(
        flips.iter().any(|line| line
            .contains("knob=threshold_machine clause=machine_ratio_decides")
            && line.contains("judgement_at_one_end=not_losing judgement_at_the_other_end=losing")),
        "{flips:?}"
    );
    assert!(flips.contains(&"name=knob_flip_summary quantity=host_cpu_per_submission knob=threshold_machine clause=machine_ratio_decides pairs=1 flipped=1".to_string()), "{flips:?}");
    let short = synthetic_medians(shared, Some(2), 400_000.0);
    let voided = BTreeMap::from([((0, shared, WaitMethod::Blocking), "void_positive_control")]);
    let underestimated = BTreeSet::from([(
        0,
        SubmissionCell {
            gap_milliseconds: 0,
            form: shared,
            wait: WaitMethod::Blocking,
            units: 8,
        },
    )]);
    let short_inputs = HostCpuPointInputs {
        thresholds: &thresholds,
        machine_ratio: Some(0.5),
        cards: &[0],
        medians: &short,
        voiding: &voiding,
        process_clock_cells: &underestimated,
    };
    let (_short_lines, short_judgements, _short_floor) = host_cpu_point_lines(&short_inputs);
    assert_eq!(
        short_judgements[&key(ThresholdKind::AttackLedger)],
        "rounds_insufficient"
    );
    let process_key = (
        quarter_point_at_four(),
        GpuCandidate::EveryStage,
        0,
        shared,
        WaitMethod::Blocking,
        0,
        ThresholdKind::AttackLedger,
        CpuClockKind::Process,
    );
    assert_eq!(short_judgements[&process_key], "rounds_insufficient");
    let voided_inputs = HostCpuPointInputs {
        thresholds: &thresholds,
        machine_ratio: Some(0.5),
        cards: &[0],
        medians: &medians,
        voiding: &voided,
        process_clock_cells: &process_clock_cells,
    };
    let (_voided_lines, voided_judgements, _voided_floor) = host_cpu_point_lines(&voided_inputs);
    assert_eq!(
        voided_judgements[&key(ThresholdKind::AttackLedger)],
        "void_positive_control"
    );
}

#[test]
fn knob_flips_pair_each_key_with_its_other_end_once() {
    let judgements: BTreeMap<u32, String> = BTreeMap::from([
        (0, "losing".to_string()),
        (1, "not_losing".to_string()),
        (2, "losing".to_string()),
        (3, "losing".to_string()),
    ]);
    let other_end = |key: &u32| key.is_multiple_of(2).then_some(key + 1);
    assert_eq!(knob_flips(&judgements, other_end), vec![(0, 1)]);
    assert_eq!(knob_pairs(&judgements, other_end), 2);
    let key = (
        quarter_point_at_four(),
        GpuCandidate::EveryStage,
        0,
        SubmissionForm::Resident(InputForm::Shared),
        WaitMethod::Blocking,
        0,
        ThresholdKind::AttackLedger,
        CpuClockKind::Thread,
    );
    assert_eq!(
        JudgementKnob::Card
            .other_end(&key, &[0, 1])
            .map(|other| other.2),
        Some(1)
    );
    assert_eq!(
        JudgementKnob::Gap
            .other_end(&key, &[0, 1])
            .map(|other| other.5),
        Some(50)
    );
    assert_eq!(
        JudgementKnob::ShortAndLongerSleep.other_end(&key, &[0, 1]),
        None
    );
    assert_eq!(
        JudgementKnob::BlockingAndLongerSleep
            .other_end(&key, &[0, 1])
            .map(|other| other.4),
        Some(WaitMethod::SleepingLonger)
    );
    assert_eq!(
        JudgementKnob::Clock
            .other_end(&key, &[0, 1])
            .map(|other| other.7),
        Some(CpuClockKind::Process)
    );
    assert_eq!(JudgementKnob::ALL.len(), 8);
}

#[test]
fn knob_flip_lines_count_only_pairs_with_a_judgement_at_both_ends() {
    let shared = SubmissionForm::Resident(InputForm::Shared);
    let key = |form, wait, gap, kind, clock| {
        (
            quarter_point_at_four(),
            GpuCandidate::EveryStage,
            0_u32,
            form,
            wait,
            gap,
            kind,
            clock,
        )
    };
    let judgements = BTreeMap::from([
        (
            key(
                shared,
                WaitMethod::Blocking,
                0,
                ThresholdKind::AttackLedger,
                CpuClockKind::Thread,
            ),
            "losing".to_string(),
        ),
        (
            key(
                shared,
                WaitMethod::Blocking,
                0,
                ThresholdKind::MachineScaled,
                CpuClockKind::Thread,
            ),
            "unstable".to_string(),
        ),
        (
            key(
                SubmissionForm::Resident(InputForm::Split),
                WaitMethod::Blocking,
                0,
                ThresholdKind::AttackLedger,
                CpuClockKind::Thread,
            ),
            "rounds_insufficient".to_string(),
        ),
        (
            key(
                shared,
                WaitMethod::Blocking,
                GAP_BETWEEN_TWO_BATCHES_MILLISECONDS,
                ThresholdKind::AttackLedger,
                CpuClockKind::Thread,
            ),
            "void_positive_control".to_string(),
        ),
        (
            key(
                shared,
                WaitMethod::SleepingShortly,
                0,
                ThresholdKind::AttackLedger,
                CpuClockKind::Thread,
            ),
            "not_run".to_string(),
        ),
        (
            key(
                shared,
                WaitMethod::SleepingLonger,
                0,
                ThresholdKind::AttackLedger,
                CpuClockKind::Thread,
            ),
            "losing".to_string(),
        ),
        (
            key(
                shared,
                WaitMethod::Blocking,
                0,
                ThresholdKind::AttackLedger,
                CpuClockKind::Process,
            ),
            "not_applicable".to_string(),
        ),
    ]);
    let lines = knob_flip_lines("host_cpu_per_submission", &judgements, &[0]);
    let summary = |knob: &str, clause: &str, pairs: usize, flipped: usize| {
        format!("name=knob_flip_summary quantity=host_cpu_per_submission knob={knob} clause={clause} pairs={pairs} flipped={flipped}")
    };
    for expected in [
        summary("card", "two_cards_judge_differently", 0, 0),
        summary("input_form", "flips_with_a_knob", 0, 0),
        summary(
            "wait_blocking_and_sleep_100_microseconds",
            "the_wait_method_decides",
            0,
            0,
        ),
        summary(
            "wait_blocking_and_sleep_1_millisecond",
            "the_wait_method_decides",
            1,
            0,
        ),
        summary(
            "wait_sleep_100_microseconds_and_sleep_1_millisecond",
            "flips_with_a_knob",
            0,
            0,
        ),
        summary("gap", "flips_with_a_knob", 0, 0),
        summary("threshold_machine", "machine_ratio_decides", 1, 1),
        summary("clock", "thread_clock_underestimates", 0, 0),
    ] {
        assert!(lines.contains(&expected), "缺 {expected}：{lines:?}");
    }
    let flip_lines: Vec<&String> = lines
        .iter()
        .filter(|line| line.starts_with("name=knob_flip "))
        .collect();
    assert_eq!(flip_lines.len(), 1, "{lines:?}");
    assert!(
        flip_lines[0].contains("knob=threshold_machine")
            && flip_lines[0]
                .ends_with("judgement_at_one_end=losing judgement_at_the_other_end=unstable"),
        "{flip_lines:?}"
    );
    for outcome in [
        "rounds_insufficient",
        "not_run",
        "not_applicable",
        "void_positive_control",
        "void_card",
    ] {
        assert!(!is_a_judgement(outcome), "{outcome} 不是判定");
    }
    for outcome in [
        "losing",
        "not_losing",
        "not_losing_lower_bound",
        "unstable",
        "one_submission_per_batch",
    ] {
        assert!(is_a_judgement(outcome), "{outcome} 是判定");
    }
}

/// 一张卡（序号 0）共用输入、阻塞等、间隔 0 的八档合成中位：c_rt = `fixed` + `per_mebibyte` × MiB，五轮相同。
fn linear_medians(
    fixed_milliseconds: f64,
    per_mebibyte_milliseconds: f64,
) -> CellMediansOfTheSegment {
    let shared = SubmissionForm::Resident(InputForm::Shared);
    SUBMISSION_UNIT_COUNTS
        .iter()
        .map(|units| {
            let mebibytes = (units * LOCAL_GPU_SLOT_BYTES) as f64 / 1_048_576.0;
            let nanoseconds = (fixed_milliseconds + per_mebibyte_milliseconds * mebibytes) * 1e6;
            let cell = SubmissionCell {
                gap_milliseconds: 0,
                form: shared,
                wait: WaitMethod::Blocking,
                units: *units,
            };
            let round = CellRoundMedians {
                thread_cpu_nanoseconds: nanoseconds,
                floor_thread_cpu_nanoseconds: None,
                process_cpu_nanoseconds: nanoseconds,
            };
            ((0, cell), vec![Some(round); ROUNDS])
        })
        .collect()
}

#[test]
fn fit_and_merging_lines_report_the_fit_and_the_category_of_every_round() {
    let medians = linear_medians(1.0, 0.25);
    let shared = SubmissionForm::Resident(InputForm::Shared);
    let cell = SubmissionCell {
        gap_milliseconds: 0,
        form: shared,
        wait: WaitMethod::Blocking,
        units: 0,
    };
    let fits = fits_of_the_cell(&medians, 0, cell);
    let overall = fits.overall.expect("八档都有");
    assert!(
        (overall.fixed_milliseconds - 1.0).abs() < 1e-9
            && (overall.per_mebibyte_milliseconds - 0.25).abs() < 1e-9
    );
    assert_eq!(fits.per_round.iter().flatten().count(), ROUNDS);
    let line = fit_line(0, cell, &fits);
    assert!(
        line.contains("fixed_milliseconds=1.000000 per_mebibyte_milliseconds=0.250000")
            && line.contains("fit_not_linear=false"),
        "{line}"
    );
    let thresholds = BTreeMap::from([(
        SubmissionPoint {
            fraction: LiveSlotFraction::ThreeQuarters,
            pool_data_units: 1024,
            batch_units: 8,
        },
        PointThresholds {
            by_candidate: BTreeMap::from([(GpuCandidate::EveryStage, 0.3)]),
            saved_upper_milliseconds: 1.4,
        },
    )]);
    let voiding = BTreeMap::new();
    let process_clock_cells = BTreeSet::new();
    let inputs = HostCpuPointInputs {
        thresholds: &thresholds,
        machine_ratio: None,
        cards: &[0],
        medians: &medians,
        voiding: &voiding,
        process_clock_cells: &process_clock_cells,
    };
    let (lines, categories) = merging_shape_lines(&inputs);
    let attack = lines
        .iter()
        .find(|merging_line| {
            merging_line.contains(
                "form=shared wait=blocking gap_milliseconds=0 threshold_kind=attack_ledger",
            )
        })
        .expect("有这一行");
    // 这一点 c_rt = 1.140625 ≥ 0.3，不是甲；n = 24 那一档 c_rt(m) = 1 + 0.25 × 0.75 = 1.1875 < 1.4，是乙-1
    assert!(
        attack.contains("one_submission_per_intent_units=24")
            && attack.ends_with("category=one_submission_per_intent"),
        "{attack}"
    );
    let machine = lines
        .iter()
        .find(|merging_line| merging_line.contains("threshold_kind=machine_scaled"))
        .expect("有这一行");
    assert!(machine.ends_with("category=not_run"), "{machine}");
    assert_eq!(
        categories.len(),
        2 * 9,
        "两种门槛 × 共用六格（三种等法、两种间隔）加分开三格"
    );
}

/// 合成计时样本里等的那一段占整次的几分之一（线程 CPU 纳秒与墙钟纳秒都照它取）：样本里的时间量，不是线程数，也不起线程。
const WAIT_SHARE_DIVISOR_OF_THE_SAMPLE_TIMING: u64 = 2;

fn timing_with(
    thread_cpu_nanoseconds: u64,
    wall_nanoseconds: u64,
    kernel_two_differs: u64,
) -> Option<SubmissionTiming> {
    Some(SubmissionTiming {
        thread_cpu_nanoseconds,
        process_cpu_nanoseconds: thread_cpu_nanoseconds + 10,
        wall_nanoseconds,
        wait_thread_cpu_nanoseconds: Some(
            thread_cpu_nanoseconds / WAIT_SHARE_DIVISOR_OF_THE_SAMPLE_TIMING,
        ),
        wait_wall_nanoseconds: Some(wall_nanoseconds / WAIT_SHARE_DIVISOR_OF_THE_SAMPLE_TIMING),
        polls: Some(3),
        input_bytes_written: 8 * LOCAL_GPU_SLOT_BYTES,
        comparison: SubmissionComparison {
            confirmed: 8,
            uncertain_entries: Vec::new(),
            kernel_one_differs_from_the_expected: 0,
            kernel_two_differs_from_the_expected: kernel_two_differs,
            are_the_sequence_numbers_holding: true,
        },
    })
}

#[test]
fn cell_round_reading_flags_a_result_differing_from_the_cpu_and_any_unreadable_submission() {
    let clean = cell_round_reading(
        &[
            timing_with(100, 200, 0),
            timing_with(300, 400, 0),
            timing_with(200, 300, 0),
        ],
        8,
    )
    .expect("都读得到");
    assert_eq!(clean.thread_cpu_median_nanoseconds, 200.0);
    assert_eq!(clean.floor_thread_cpu_median_nanoseconds, Some(100.0));
    assert_eq!(clean.process_cpu_median_nanoseconds, 210.0);
    assert_eq!(clean.polls_median, Some(3.0));
    assert!(clean.are_the_results_holding && clean.is_every_thread_cpu_inside_the_wall);
    let wrong = cell_round_reading(&[timing_with(100, 200, 0), timing_with(100, 200, 1)], 8)
        .expect("都读得到");
    assert!(!wrong.are_the_results_holding);
    let outside = cell_round_reading(&[timing_with(300, 200, 0)], 8).expect("读得到");
    assert!(!outside.is_every_thread_cpu_inside_the_wall);
    assert_eq!(
        cell_round_reading(&[timing_with(100, 200, 0), None], 8),
        None
    );
    assert!(clean.is_every_wait_inside_the_submission && clean.are_the_comparison_counts_holding);
    assert!(
        !cell_round_reading(&[timing_with(100, 200, 0)], 9)
            .expect("读得到")
            .are_the_comparison_counts_holding
    );
    let mut waiting_longer = timing_with(100, 200, 0).expect("合成的");
    waiting_longer.wait_thread_cpu_nanoseconds = Some(101);
    assert!(
        !cell_round_reading(&[Some(waiting_longer)], 8)
            .expect("读得到")
            .is_every_wait_inside_the_submission
    );
}

#[test]
fn input_file_checks_hold_only_for_the_registered_digests() {
    let checks = input_file_checks(b"not the attack ledger", b"not the stage split product");
    assert_eq!(checks.len(), 2);
    assert!(checks.iter().all(|(_, _, _, holds)| !holds));
    assert_eq!(checks[0].2, ATTACK_LEDGER_SHA256);
    assert_eq!(checks[1].2, STAGE_SPLIT_PRODUCT_SHA256);
}

#[test]
fn machine_ratio_cross_check_compares_the_local_checksum_cpu_over_the_threshold_with_the_ratio() {
    let thresholds = BTreeMap::from([(
        quarter_point_at_four(),
        PointThresholds {
            by_candidate: BTreeMap::from([(GpuCandidate::ScanChecksumOnly, 0.1)]),
            saved_upper_milliseconds: 1.0,
        },
    )]);
    let cpu: BTreeMap<u64, f64> = BTreeMap::from([(4, 0.04), (8, 0.04)]);
    let near = machine_ratio_cross_check_lines(&thresholds, &cpu, Some(0.5));
    assert!(
        near[0].contains("ratio=0.400000 machine_ratio=0.500000 off_by_more_than_double=false"),
        "{near:?}"
    );
    let far = machine_ratio_cross_check_lines(&thresholds, &cpu, Some(0.1));
    assert!(far[0].ends_with("off_by_more_than_double=true"), "{far:?}");
}
