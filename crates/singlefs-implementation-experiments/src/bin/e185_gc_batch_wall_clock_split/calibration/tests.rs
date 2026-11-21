use super::*;

/// R2-1：前或后的校准比有一个超过 1.10 就作废；比 = 这一次 ÷ 基线。
#[test]
fn calibration_ratio_above_one_point_one_before_or_after_voids_the_intent() {
    assert!(
        is_voided_by_the_calibration_ratios(
            calibration_ratio(1_200_000.0, 1_000_000.0),
            calibration_ratio(1_000_000.0, 1_000_000.0)
        ),
        "基线 1 ms、这一次 1.2 ms：比 1.2，作废"
    );
    assert!(
        is_voided_by_the_calibration_ratios(1.00, 1.30),
        "前 1.00、后 1.30：作废"
    );
    assert!(!is_voided_by_the_calibration_ratios(1.05, 0.98));
}

/// R2-1 的边界：正好 1.10 不作废（门槛不含等号）。
#[test]
fn calibration_ratio_of_exactly_one_point_one_does_not_void() {
    assert!(!is_voided_by_the_calibration_ratios(1.10, 1.10));
}

fn calibration_reading_with(
    crc_median_nanoseconds: Option<f64>,
    are_the_anchors_holding: bool,
) -> CalibrationReading {
    CalibrationReading {
        crc_median_nanoseconds,
        crc_runs_off_the_anchor: usize::from(!are_the_anchors_holding),
        chase_nanoseconds: Some(100_000_000.0),
        is_the_chase_end_the_anchor: true,
    }
}

/// 停机 10 按遍累计：一次校准里 CRC 有两遍不是锚点、追指针终点也不符，算三遍。
#[test]
fn calibration_anchor_mismatches_count_every_run_off_the_anchor() {
    let reading = CalibrationReading {
        crc_median_nanoseconds: Some(1_000_000.0),
        crc_runs_off_the_anchor: 2,
        chase_nanoseconds: Some(100_000_000.0),
        is_the_chase_end_the_anchor: false,
    };
    assert_eq!(reading.anchor_mismatching_runs(), 3);
    assert!(!reading.are_the_anchors_holding());
    assert_eq!(
        calibration_reading_with(Some(1_000_000.0), true).anchor_mismatching_runs(),
        0
    );
}

/// 停机 10 与读不到：锚点不符先判（比再小也是锚点不符），CRC 那份读不到判读不到，都写进 `invalid_reason`。
#[test]
fn calibration_verdict_puts_an_anchor_mismatch_before_the_ratio_and_an_unreadable_clock() {
    let baseline = CalibrationBaseline {
        crc_median_nanoseconds: 1_000_000.0,
        chase_median_nanoseconds: Some(100_000_000.0),
        are_the_anchors_holding: true,
    };
    let holding = calibration_reading_with(Some(1_000_000.0), true);
    let mismatching = calibration_reading_with(Some(1_000_000.0), false);
    let slow = calibration_reading_with(Some(1_300_000.0), true);
    let unreadable = calibration_reading_with(None, true);
    assert_eq!(
        calibration_verdict(&holding, &holding, &baseline),
        CalibrationVerdict::Holds
    );
    assert_eq!(
        calibration_verdict(&holding, &mismatching, &baseline),
        CalibrationVerdict::AnchorMismatch
    );
    assert_eq!(
        calibration_verdict(&holding, &slow, &baseline),
        CalibrationVerdict::RatioAboveTheEdge
    );
    assert_eq!(
        calibration_verdict(&unreadable, &holding, &baseline),
        CalibrationVerdict::Unreadable
    );
    assert_eq!(
        CalibrationVerdict::RatioAboveTheEdge.invalid_reason(),
        Some("calibration_ratio_above_the_registered_edge")
    );
    assert_eq!(CalibrationVerdict::Holds.invalid_reason(), None);
}

/// 第七节乙类：16 MiB 校准缓冲的 CRC-32C 是 0x3430be3b，32 MiB 是 0x84b65aa5，追指针从 0 走 2^20 步落在 9437184。
#[test]
fn calibration_work_gives_the_registered_crc_and_chase_anchors() {
    let buffer = calibration_buffer(16 << 20);
    assert_eq!(crc32_castagnoli(&buffer), 0x3430_be3b);
    assert_eq!(crc32_castagnoli(&calibration_buffer(32 << 20)), 0x84b6_5aa5);
    let table = chase_table();
    assert_eq!(table.len(), 16_777_216);
    let (_nanoseconds, end_index) = one_chase_run(&table).expect("线程 CPU 时钟读得到");
    assert_eq!(end_index, 9_437_184);
    let reading = calibration_reading(&buffer, &table);
    assert!(reading.are_the_anchors_holding());
    assert!(reading
        .crc_median_nanoseconds
        .is_some_and(|nanoseconds| nanoseconds > 0.0));
}

/// R2-4：段末 ÷ 段开头不在 [0.90, 1.10] 里记漂移，两端不算。
#[test]
fn calibration_baseline_drift_is_flagged_outside_ninety_to_one_hundred_ten_percent() {
    assert!(calibration_drifted(calibration_ratio(
        1_200_000.0,
        1_000_000.0
    )));
    assert!(calibration_drifted(0.85));
    assert!(!calibration_drifted(1.0));
    assert!(!calibration_drifted(0.90));
    assert!(!calibration_drifted(1.10));
    let end = CalibrationBaseline {
        crc_median_nanoseconds: 1_200_000.0,
        chase_median_nanoseconds: None,
        are_the_anchors_holding: true,
    };
    assert!(
        calibration_baseline_line("wall_clock", "end", &end, Some(1.2))
            .ends_with("ratio_of_the_end_to_the_start=1.200000 drifted=true")
    );
}

/// R2-3：加倍的活 ÷ 基线在 [1.8, 2.2] 里（两端算）才算校准分得出两倍的活。
#[test]
fn calibration_doubling_holds_only_between_one_point_eight_and_two_point_two() {
    assert!(calibration_doubling_holds(calibration_ratio(
        2_000_000.0,
        1_000_000.0
    )));
    assert!(calibration_doubling_holds(1.8));
    assert!(calibration_doubling_holds(2.2));
    assert!(!calibration_doubling_holds(1.5));
    assert!(!calibration_doubling_holds(2.5));
}

/// 第八节「新加的轨迹」：读不到的意图不进峰值与条数（不当 0），三条门槛各数一遍，期末值取最后一条。
#[test]
fn calibration_trajectory_counts_the_intents_above_each_edge() {
    let line = calibration_trajectory_line(
        "wall_clock",
        &[Some(1.02), Some(1.08), None, Some(1.15), Some(1.30)],
        &WaitingLedger {
            total_seconds: 120,
            longest_seconds: 80,
            waits: 2,
        },
    );
    assert!(
        line.contains("intents=5 intents_with_a_readable_ratio=4 larger_ratio_peak=1.300000 intents_above_the_voiding_edge=2 "),
        "{line}"
    );
    assert!(
        line.contains(" intents_above_the_lower_reported_edge=3 "),
        "{line}"
    );
    assert!(
        line.contains(" intents_above_the_upper_reported_edge=1 "),
        "{line}"
    );
    assert!(
        line.ends_with(
            "larger_ratio_final=1.300000 wait_longest_seconds=80 waits=2 wait_total_seconds=120"
        ),
        "{line}"
    );
}

/// N1、N3–N9、N15 那一行：前后两个比、各个增量与前后两个读数照键写（读不到的写 none）。
#[test]
fn intent_machine_readings_text_reports_the_ratios_and_the_increases() {
    let calibration = SegmentCalibration {
        segment: "wall_clock",
        started: Instant::now(),
        buffer: Vec::new(),
        chase_table: Vec::new(),
        baseline: CalibrationBaseline {
            crc_median_nanoseconds: 1_000_000.0,
            chase_median_nanoseconds: Some(100_000_000.0),
            are_the_anchors_holding: true,
        },
        clock_ticks_per_second: Some(100),
        anchor_mismatches: 0,
        waiting: WaitingLedger {
            total_seconds: 0,
            longest_seconds: 0,
            waits: 0,
        },
        larger_ratio_per_intent: Vec::new(),
    };
    let snapshot_at =
        |run_queue_wait_nanoseconds: u64, user_ticks: u64, cpu_number: u32, seconds: f64| {
            ThreadAndMachineSnapshot {
                run_queue_wait_nanoseconds: Some(run_queue_wait_nanoseconds),
                involuntary_switches: Some(user_ticks),
                thread_counters: Some(ThreadCounters {
                    minor_faults: user_ticks + 1,
                    major_faults: 0,
                    user_ticks,
                    system_ticks: 1,
                }),
                cpu_pressure_some_ten_seconds: Some(2.5),
                memory_pressure_some_ten_seconds: None,
                load_average_one_minute: Some(18.47),
                cpu_number: Some(cpu_number),
                seconds_since_the_segment_start: seconds,
            }
        };
    let intent = IntentCalibration {
        before: calibration_reading_with(Some(1_200_000.0), true),
        after: calibration_reading_with(Some(1_000_000.0), true),
        snapshot_before: snapshot_at(1_000, 7, 4, 1.0),
        snapshot_after: snapshot_at(1_500, 9, 5, 2.5),
        verdict: CalibrationVerdict::RatioAboveTheEdge,
    };
    let text =
        calibration.intent_machine_readings_text(&label_of_round(DeviceArm::Memory, 1), &intent);
    assert!(
        text.starts_with("name=intent_machine_readings arm=memory fraction=0.75 pool_data_units=1024 batch_units=8 round=1 attempt=1 injection=none choice=greedy "),
        "{text}"
    );
    assert!(
        text.contains(" calibration_before_ratio=1.200000 calibration_after_ratio=1.000000 "),
        "{text}"
    );
    assert!(
        text.contains(" calibration_verdict=ratio_above_the_edge run_queue_wait_nanoseconds=500 involuntary_switches=2 user_ticks=2 system_ticks=0 clock_ticks_per_second=100 minor_faults=2 major_faults=0 cpu_pressure_some_ten_seconds_before=2.50 "),
        "{text}"
    );
    assert!(
        text.contains(" memory_pressure_some_ten_seconds_before=none "),
        "{text}"
    );
    assert!(
        text.ends_with(" cpu_number_before=4 cpu_number_after=5 seconds_since_the_segment_start_before=1.000 seconds_since_the_segment_start_after=2.500"),
        "{text}"
    );
}
