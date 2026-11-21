use super::*;

#[test]
fn judgements_land_on_the_registered_side_of_every_threshold() {
    assert_eq!(judgement_of_rho_at_the_registered_edges(0.4), "disk");
    assert_eq!(judgement_of_rho_at_the_registered_edges(1.0), "both");
    assert_eq!(judgement_of_rho_at_the_registered_edges(3.0), "cpu");
    assert_eq!(judgement_of_wall_ratio_at_the_registered_edges(3.5), "disk");
    assert_eq!(judgement_of_wall_ratio_at_the_registered_edges(2.0), "both");
    assert_eq!(judgement_of_wall_ratio_at_the_registered_edges(1.2), "cpu");
    assert_eq!(
        judgement_of_fixed_share_at_the_registered_edges(0.05),
        "one_batch_per_submission"
    );
    assert_eq!(
        judgement_of_fixed_share_at_the_registered_edges(0.3),
        "merging_optional"
    );
    assert_eq!(
        judgement_of_fixed_share_at_the_registered_edges(0.7),
        "merge_batches"
    );
    assert_eq!(
        judgement_of_bandwidth_ratio_at_the_registered_edge(0.05),
        "single_run_enough"
    );
    assert_eq!(
        judgement_of_bandwidth_ratio_at_the_registered_edge(0.2),
        "needs_the_concurrent_measurement"
    );
    assert_eq!(
        judgement_of_the_sixteen_thread_to_gpu_ratio(0.78),
        "gpu_loses"
    );
    assert_eq!(
        judgement_of_the_sixteen_thread_to_gpu_ratio(1.5),
        "gpu_has_a_chance"
    );
    assert_eq!(
        judgement_of_the_foreground_fsync_latency_ratio(2.5),
        "gc_drags_the_foreground"
    );
    assert_eq!(
        judgement_of_share(0.09, STAGE_SHARE_AT_LEAST_FOR_WORTH),
        "not_worth"
    );
    assert_eq!(
        judgement_of_share(0.1, STAGE_SHARE_AT_LEAST_FOR_WORTH),
        "worth"
    );
    assert_eq!(judgement_of_the_cpu_reconciliation(0.25), "paths_disagree");
    assert_eq!(
        judgement_of_the_batch_doubling(&[1.2, 1.1, 1.3, 1.25, 1.05]),
        "rises"
    );
    assert_eq!(
        judgement_of_the_batch_doubling(&[1.2, 0.9, 1.3, 1.25, 1.05]),
        "does_not_rise",
        "有一轮没涨"
    );
}

#[test]
fn rho_wall_ratio_fixed_share_and_bandwidth_ratio_judgements_flip_when_the_threshold_moves_between_two_points(
) {
    let (_, lower_rho_judgement, higher_rho_judgement) =
        judgements_with_the_threshold_moved_between(0.8, 1.6, |value, threshold| {
            judgement_of_rho(value, threshold, f64::INFINITY)
        });
    assert_ne!(
        lower_rho_judgement, higher_rho_judgement,
        "Q1 的门槛挪到 0.8 与 1.6 之间"
    );
    assert_eq!(
        (
            judgement_of_rho_at_the_registered_edges(0.8),
            judgement_of_rho_at_the_registered_edges(1.6)
        ),
        ("both", "both"),
        "不挪门槛两点同判"
    );
    let (_, lower_wall_ratio_judgement, higher_wall_ratio_judgement) =
        judgements_with_the_threshold_moved_between(1.8, 2.6, |value, threshold| {
            judgement_of_wall_ratio(value, threshold, f64::NEG_INFINITY)
        });
    assert_ne!(
        lower_wall_ratio_judgement, higher_wall_ratio_judgement,
        "Q2"
    );
    let (_, lower_fixed_share_judgement, higher_fixed_share_judgement) =
        judgements_with_the_threshold_moved_between(0.2, 0.4, |value, threshold| {
            judgement_of_fixed_share(value, threshold, f64::INFINITY)
        });
    assert_ne!(
        lower_fixed_share_judgement, higher_fixed_share_judgement,
        "Q13"
    );
    let (_, lower_bandwidth_ratio_judgement, higher_bandwidth_ratio_judgement) =
        judgements_with_the_threshold_moved_between(0.02, 0.05, judgement_of_bandwidth_ratio);
    assert_ne!(
        lower_bandwidth_ratio_judgement, higher_bandwidth_ratio_judgement,
        "Q16"
    );
}

#[test]
fn inclusion_check_flags_cpu_plus_wait_above_the_wall() {
    assert_eq!(
        inclusion_verdict(1_000_000, 800_000, 500_000),
        InclusionVerdict::CpuPlusWaitAboveTheWall,
        "挂钟 1 ms、CPU 0.8 ms、等盘 0.5 ms"
    );
}

#[test]
fn inclusion_check_flags_unaccounted_time_above_ten_percent() {
    assert_eq!(
        inclusion_verdict(1_000_000, 500_000, 300_000),
        InclusionVerdict::UnaccountedAboveTenPercent
    );
    assert_eq!(
        inclusion_verdict(1_000_000, 600_000, 350_000),
        InclusionVerdict::Holds
    );
}

#[test]
fn positive_control_judges_hold_inside_their_bands_only() {
    assert!(
        disk_arm_control_holds(24_000_000.0, 10.0, 0.05),
        "10 次写 × 2 ms，等盘多 24 ms（1.2 倍）"
    );
    assert!(
        !disk_arm_control_holds(16_000_000.0, 10.0, 0.0),
        "只多 0.8 倍"
    );
    assert!(
        !disk_arm_control_holds(24_000_000.0, 10.0, 0.2),
        "CPU 变了 20%"
    );
    assert!(
        cpu_arm_cpu_clause_holds(20_500_000.0, 20_000_000.0),
        "10 次写 × 2 ms，CPU 多 20.5 ms"
    );
    assert!(!cpu_arm_cpu_clause_holds(25_000_000.0, 20_000_000.0));
    assert!(
        cpu_arm_write_wait_clause_holds(1_900_000.0, 20_000_000.0),
        "写调用等盘多 1.9 ms，是注入量的 9.5%"
    );
    assert!(
        !cpu_arm_write_wait_clause_holds(-2_100_000.0, 20_000_000.0),
        "少 2.1 ms 也超出：判的是绝对值"
    );
    assert!(!cpu_arm_write_wait_clause_holds(2_100_000.0, 20_000_000.0));
    assert!(cpu_arm_original_wait_clause_holds(-0.05));
    assert!(!cpu_arm_original_wait_clause_holds(0.2));
    assert!(stage_control_holds(2_100_000.0));
    assert!(!stage_control_holds(2_500_000.0));
    assert!(double_crc_control_holds(2.0, 1.0));
    assert!(!double_crc_control_holds(1.5, 1.0));
    assert!(page_cache_control_holds(3.0e9, 2.0e9));
    assert!(!page_cache_control_holds(2.1e9, 2.0e9));
}

/// 基线先跑完、对照后跑的那一趟产物（`e185-gc-batch-wall-clock-split-wall-clock.out`）对照点 f = 0.75、P = 1024、K = 8 每轮整条意图的读数，由那一轮四次调用的
/// `name=call` 行按栏加出来：（轮号、调用 CPU、等盘、写调用次数、写调用等盘、写调用挂钟、设备调用挂钟）。
/// 最后两栏是 V1（等盘不减设备调用里的 CPU）会读到的写调用等盘与整条意图等盘。
type SeparatedScheduleRound = (usize, f64, f64, f64, f64, f64, f64);

#[rustfmt::skip]
const SEPARATED_SCHEDULE_MEMORY_BASELINE: [SeparatedScheduleRound; 5] = [
    (1, 106_650_857.0, -120_419.0, 176.0, -19_440.0, 799_443.0, 1_271_745.0),
    (2, 106_204_453.0, -118_649.0, 176.0, -19_041.0, 816_543.0, 1_275_655.0),
    (3, 106_781_276.0, -119_583.0, 176.0, -19_380.0, 799_744.0, 1_273_445.0),
    (4, 106_644_964.0, -122_455.0, 176.0, -19_270.0, 804_394.0, 1_259_889.0),
    (5, 106_632_087.0, -119_043.0, 176.0, -19_181.0, 818_224.0, 1_266_807.0),
];
#[rustfmt::skip]
const SEPARATED_SCHEDULE_MEMORY_SLEEP: [SeparatedScheduleRound; 5] = [
    (1, 107_321_255.0, 361_903_916.0, 176.0, 362_011_585.0, 363_937_453.0, 364_415_787.0),
    (2, 108_514_827.0, 361_867_449.0, 176.0, 361_975_619.0, 363_881_382.0, 364_353_044.0),
    (3, 108_618_464.0, 361_879_634.0, 176.0, 361_987_757.0, 363_938_593.0, 364_422_204.0),
    (4, 107_897_310.0, 361_584_541.0, 176.0, 361_691_513.0, 363_453_732.0, 363_925_104.0),
    (5, 108_574_516.0, 361_889_292.0, 176.0, 361_998_301.0, 363_928_584.0, 364_409_338.0),
];
#[rustfmt::skip]
const SEPARATED_SCHEDULE_MEMORY_SPIN: [SeparatedScheduleRound; 5] = [
    (1, 458_878_127.0, -81_616.0, 176.0, 18_418.0, 352_899_546.0, 353_359_687.0),
    (2, 458_955_739.0, -79_847.0, 176.0, 20_514.0, 352_885_629.0, 353_348_721.0),
    (3, 459_127_621.0, 25_568.0, 176.0, 127_010.0, 352_998_957.0, 353_461_187.0),
    (4, 460_322_754.0, 31_784.0, 176.0, 132_072.0, 353_031_469.0, 353_504_572.0),
    (5, 459_040_019.0, -56_116.0, 176.0, 44_294.0, 352_915_809.0, 353_370_413.0),
];
#[rustfmt::skip]
const SEPARATED_SCHEDULE_REAL_DISK_BASELINE: [SeparatedScheduleRound; 5] = [
    (1, 109_092_461.0, 94_772_746.0, 176.0, 22_022_149.0, 23_272_218.0, 99_037_220.0),
    (2, 116_147_957.0, 241_725_690.0, 176.0, 66_655_889.0, 68_048_987.0, 246_513_279.0),
    (3, 116_614_314.0, 341_369_727.0, 176.0, 104_965_004.0, 106_504_352.0, 346_530_879.0),
    (4, 116_404_811.0, 50_426_296.0, 176.0, 5_518_406.0, 6_876_064.0, 54_918_191.0),
    (5, 119_740_390.0, 335_425_318.0, 176.0, 87_655_725.0, 89_148_566.0, 340_540_194.0),
];
#[rustfmt::skip]
const SEPARATED_SCHEDULE_REAL_DISK_SLEEP: [SeparatedScheduleRound; 5] = [
    (1, 119_998_852.0, 618_946_708.0, 176.0, 400_368_017.0, 403_069_041.0, 625_195_352.0),
    (2, 119_374_356.0, 648_815_686.0, 176.0, 431_612_652.0, 434_321_325.0, 654_967_575.0),
    (3, 121_477_226.0, 645_589_647.0, 176.0, 426_420_624.0, 429_068_835.0, 651_841_556.0),
    (4, 119_360_821.0, 615_122_931.0, 176.0, 428_582_878.0, 431_193_597.0, 621_116_652.0),
    (5, 120_276_080.0, 681_350_924.0, 176.0, 425_225_299.0, 427_879_238.0, 687_481_321.0),
];
#[rustfmt::skip]
const SEPARATED_SCHEDULE_REAL_DISK_SPIN: [SeparatedScheduleRound; 5] = [
    (1, 470_584_575.0, 282_688_855.0, 176.0, 69_994_372.0, 424_142_393.0, 640_223_561.0),
    (2, 471_986_271.0, 335_232_227.0, 176.0, 122_034_216.0, 476_227_747.0, 692_923_103.0),
    (3, 469_560_156.0, 54_841_237.0, 176.0, 6_815_812.0, 360_956_439.0, 412_162_281.0),
    (4, 471_007_281.0, 305_723_259.0, 176.0, 68_484_488.0, 422_628_281.0, 663_299_405.0),
    (5, 471_091_665.0, 311_975_618.0, 176.0, 61_572_246.0, 415_830_508.0, 669_657_825.0),
];

/// 等盘怎么读：装置今天的读法，或 V1 的读法（等盘 = 设备调用挂钟）。
#[derive(Clone, Copy)]
enum WaitReadingOfTheFixture {
    AsBuilt,
    DeviceWallWithoutTheCpuSubtracted,
}

fn separated_schedule_rounds(
    rows: &[SeparatedScheduleRound],
    wait_reading: WaitReadingOfTheFixture,
) -> Vec<RowOneControlRound> {
    rows.iter()
        .map(
            |&(round, cpu, wait, write_calls, write_wait, write_wall, device_wall)| {
                let (wait_nanoseconds, write_wait_nanoseconds) = match wait_reading {
                    WaitReadingOfTheFixture::AsBuilt => (wait, write_wait),
                    WaitReadingOfTheFixture::DeviceWallWithoutTheCpuSubtracted => {
                        (device_wall, write_wall)
                    }
                };
                RowOneControlRound {
                    round,
                    cpu_nanoseconds: cpu,
                    wait_nanoseconds,
                    write_calls,
                    write_wait_nanoseconds,
                }
            },
        )
        .collect()
}

fn separated_schedule_reading(
    injection: WriteInjection,
    baseline: &[SeparatedScheduleRound],
    control: &[SeparatedScheduleRound],
    wait_reading: WaitReadingOfTheFixture,
) -> RowOneControlReading {
    row_one_control_reading(
        injection,
        &separated_schedule_rounds(baseline, wait_reading),
        &separated_schedule_rounds(control, wait_reading),
    )
    .expect("基线与对照各 5 轮")
}

/// 登记第十二节 12.9：基线先跑完、对照后跑那一趟的 CPU 臂两格（产物 `name=positive_control row=1 arm_of_the_question=cpu` 那两行）照原判据不过，
/// 照新判法（写调用的等盘增量 ÷ 注入量）过；CPU 那一句两格都过。
#[test]
fn separated_schedule_cpu_arm_cells_hold_by_the_write_wait_over_the_injected_and_fail_by_the_original_change(
) {
    let as_built = WaitReadingOfTheFixture::AsBuilt;
    let memory = separated_schedule_reading(
        WriteInjection::SpinTwoMillisecondsOfThreadCpu,
        &SEPARATED_SCHEDULE_MEMORY_BASELINE,
        &SEPARATED_SCHEDULE_MEMORY_SPIN,
        as_built,
    );
    let real_disk = separated_schedule_reading(
        WriteInjection::SpinTwoMillisecondsOfThreadCpu,
        &SEPARATED_SCHEDULE_REAL_DISK_BASELINE,
        &SEPARATED_SCHEDULE_REAL_DISK_SPIN,
        as_built,
    );
    assert_eq!(
        (memory.write_calls, memory.injected_nanoseconds),
        (176.0, 176.0 * 2_000_000.0)
    );
    assert_eq!(memory.cpu_increment_nanoseconds, 352_395_055.0);
    assert_eq!(memory.wait_increment_nanoseconds, 63_467.0);
    assert_eq!(memory.write_wait_increment_nanoseconds, 63_564.0);
    assert_eq!(format!("{:.6}", memory.wait_change), "-0.530736");
    assert_eq!(real_disk.cpu_increment_nanoseconds, 354_602_470.0);
    assert_eq!(real_disk.wait_increment_nanoseconds, 63_997_569.0);
    assert_eq!(real_disk.write_wait_increment_nanoseconds, 1_828_599.0);
    assert_eq!(format!("{:.6}", real_disk.wait_change), "0.264753");
    for (device, reading) in [("memory", memory), ("real_disk", real_disk)] {
        assert_eq!(
            reading.judgement,
            RowOneControlJudgement::CpuArm {
                cpu_clause_holds: true,
                write_wait_clause_holds: true,
                wait_clause_under_the_original_criterion_holds: false,
            },
            "{device}：新判法过、原判据那一句照算不过"
        );
        assert!(reading.judgement.holds(), "{device}：这一格按新判法算过");
    }
}

/// V1（等盘 = 设备调用挂钟，不减设备调用里的 CPU）在那一趟两格上读到的写调用等盘增量约等于注入量，新判法两格都判红。
#[test]
fn device_wait_without_the_cpu_subtracted_fails_the_write_wait_clause_on_both_separated_schedule_cells(
) {
    let without_the_cpu = WaitReadingOfTheFixture::DeviceWallWithoutTheCpuSubtracted;
    let memory = separated_schedule_reading(
        WriteInjection::SpinTwoMillisecondsOfThreadCpu,
        &SEPARATED_SCHEDULE_MEMORY_BASELINE,
        &SEPARATED_SCHEDULE_MEMORY_SPIN,
        without_the_cpu,
    );
    let real_disk = separated_schedule_reading(
        WriteInjection::SpinTwoMillisecondsOfThreadCpu,
        &SEPARATED_SCHEDULE_REAL_DISK_BASELINE,
        &SEPARATED_SCHEDULE_REAL_DISK_SPIN,
        without_the_cpu,
    );
    assert_eq!(memory.write_wait_increment_nanoseconds, 352_111_415.0);
    assert_eq!(real_disk.write_wait_increment_nanoseconds, 354_579_294.0);
    for (device, reading) in [("memory", memory), ("real_disk", real_disk)] {
        assert_eq!(
            reading.judgement,
            RowOneControlJudgement::CpuArm {
                cpu_clause_holds: true,
                write_wait_clause_holds: false,
                wait_clause_under_the_original_criterion_holds: false,
            },
            "{device}：漏减 CPU 时写调用等盘多出一整份注入量"
        );
        assert!(!reading.judgement.holds(), "{device}：V1 下这一格判红");
    }
}

/// 盘臂那一行没改：那一趟两格（产物 `arm_of_the_question=disk` 那两行）照旧判过，增量与产物相同。
#[test]
fn separated_schedule_disk_arm_cells_keep_their_judgement() {
    let memory = separated_schedule_reading(
        WriteInjection::SleepTwoMilliseconds,
        &SEPARATED_SCHEDULE_MEMORY_BASELINE,
        &SEPARATED_SCHEDULE_MEMORY_SLEEP,
        WaitReadingOfTheFixture::AsBuilt,
    );
    let real_disk = separated_schedule_reading(
        WriteInjection::SleepTwoMilliseconds,
        &SEPARATED_SCHEDULE_REAL_DISK_BASELINE,
        &SEPARATED_SCHEDULE_REAL_DISK_SLEEP,
        WaitReadingOfTheFixture::AsBuilt,
    );
    assert_eq!(
        (
            memory.cpu_increment_nanoseconds,
            memory.wait_increment_nanoseconds
        ),
        (1_869_863.0, 361_999_217.0)
    );
    assert_eq!(
        (
            real_disk.cpu_increment_nanoseconds,
            real_disk.wait_increment_nanoseconds
        ),
        (3_594_041.0, 403_863_957.0)
    );
    assert_eq!(
        memory.judgement,
        RowOneControlJudgement::DiskArm { holds: true }
    );
    assert_eq!(
        real_disk.judgement,
        RowOneControlJudgement::DiskArm { holds: true }
    );
}

/// 按轮号配对的增量中位只报不判：取同一轮号的「对照 − 基线」，缺了一边的轮号不进。
#[test]
fn paired_round_increments_take_the_same_round_number_from_both_sides() {
    let reading = separated_schedule_reading(
        WriteInjection::SpinTwoMillisecondsOfThreadCpu,
        &SEPARATED_SCHEDULE_REAL_DISK_BASELINE,
        &SEPARATED_SCHEDULE_REAL_DISK_SPIN,
        WaitReadingOfTheFixture::AsBuilt,
    );
    assert_eq!(reading.paired_rounds, 5);
    assert_eq!(
        reading.cpu_increment_paired_median_nanoseconds,
        Some(354_602_470.0)
    );
    assert_eq!(
        reading.write_wait_increment_paired_median_nanoseconds,
        Some(47_972_223.0)
    );
    let baseline_missing_one_round: Vec<SeparatedScheduleRound> =
        SEPARATED_SCHEDULE_MEMORY_BASELINE
            .into_iter()
            .filter(|row| row.0 != 3)
            .collect();
    let reading_with_one_baseline_round_missing = separated_schedule_reading(
        WriteInjection::SpinTwoMillisecondsOfThreadCpu,
        &baseline_missing_one_round,
        &SEPARATED_SCHEDULE_MEMORY_SPIN,
        WaitReadingOfTheFixture::AsBuilt,
    );
    assert_eq!(reading_with_one_baseline_round_missing.paired_rounds, 4);
    assert_eq!(
        reading_with_one_baseline_round_missing.write_wait_increment_paired_median_nanoseconds,
        Some(f64::midpoint(39_555.0, 63_475.0))
    );
}
