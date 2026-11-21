//! 表与盘集与判定的描述、历史空间行、读数行、对照行、类一致行、轨迹行、字段差异与不变性检查、手算推导与判定行。

use e7_index_bench::i64_from_usize;
use super::*;

pub(super) fn describe_table(table: Table) -> String {
    format!("{},{}", table[0], table[1])
}

fn describe_disk_set(disks: DiskSet) -> String {
    let named: Vec<String> = (0..DISK_COUNT)
        .filter(|disk| disks[*disk])
        .map(|disk| disk.to_string())
        .collect();
    format!("{{{}}}", named.join(","))
}

fn describe_decision(decision: Decision) -> String {
    let writable = if decision.writable_mount_made {
        "made"
    } else {
        "refused"
    };
    let read_only = if decision.read_only_mount_made {
        "made"
    } else {
        "refused"
    };
    format!(
        "writable={writable} read_only={read_only} named={}",
        describe_disk_set(decision.named_by_read_only)
    )
}

/// 登记第四节第 7 条的元组数与状态数（按每条臂的写序）：（原始状态数，去重后盘面数，可写元组数，族 D 可写元组数，族 S 可写元组数）。
fn expected_history_counts(arm: Arm) -> (usize, usize, usize, usize, usize) {
    match arm {
        Arm::OldTableWhileIntentMarkerIsOnTheDisk => (27, 25, 2207, 1815, 392),
        Arm::ChainThenLowest
        | Arm::HighestTable
        | Arm::LowestToReadHighestToProbeWithAbandon
        | Arm::OnlyIfEveryTableIsEqual => (18, 17, 1487, 1221, 266),
    }
}

pub(super) fn grown_family_history_count(set: &HistorySet) -> usize {
    set.observations
        .iter()
        .filter(|observation| observation.family == Family::GrownBefore)
        .count()
}
fn history_space_lines(analyses: &[ArmAnalysis]) -> Vec<String> {
    analyses
        .iter()
        .map(|analysis| {
            let (writes, barriers) = count_writes_and_pool_barriers(&write_plan_of(analysis.arm));
            let writable_tuples = analysis.set.observations.len();
            let grown_family = grown_family_history_count(&analysis.set);
            format!(
                "name=history_space arm={} writes={writes} pool_barriers={barriers} raw_states={} distinct_boards={} crash_points={} \
                 writable_tuples={writable_tuples} read_only_tuples={writable_tuples} grown_family_tuples={grown_family} \
                 never_grown_family_tuples={} out_of_model_histories={}",
                arm_name(analysis.arm),
                analysis.set.raw_state_count,
                analysis.set.distinct_board_count,
                analysis.set.crash_point_count,
                writable_tuples - grown_family,
                zero_or_count(analysis.set.out_of_model_histories),
            )
        })
        .collect()
}

fn reading_lines(analyses: &[ArmAnalysis]) -> Vec<String> {
    let mut lines = Vec::new();
    for analysis in analyses {
        let denominator = analysis.set.observations.len();
        for reading in ALL_READINGS {
            let count = analysis.arm_readings.counts[reading as usize];
            lines.push(format!(
                "name=reading arm={} reading={} count={count} denominator={denominator} exists={}",
                arm_name(analysis.arm),
                reading_name(reading),
                count >= 1
            ));
        }
    }
    lines
}

fn control_lines(analyses: &[ArmAnalysis]) -> Vec<String> {
    let mut lines = Vec::new();
    for analysis in analyses {
        let denominator = analysis.set.observations.len();
        for control in CONTROL_READERS {
            let counts = control_readings_of(analysis, control).counts;
            lines.push(format!(
                "name=control arm={} reader={} leak_writable={} leak_read_only={} misrefusal_literal={} misrefusal_avoidable={} \
                 writable_denominator={denominator} read_only_denominator={denominator}",
                arm_name(analysis.arm),
                control_reader_name(control),
                counts[ReadingKind::LeakThroughWritableMount as usize],
                counts[ReadingKind::LeakThroughReadOnlyMount as usize],
                counts[ReadingKind::LiteralMisrefusal as usize],
                counts[ReadingKind::AvoidableMisrefusal as usize],
            ));
        }
    }
    lines
}

/// 每个读者（臂自己与三条对照读者）在每个观测类里的决定是否全相同：不同的类数必须是 0。
fn class_consistency_lines(analyses: &[ArmAnalysis]) -> Vec<String> {
    let mut lines = Vec::new();
    for analysis in analyses {
        let mut readers: Vec<(String, &ReaderReadings)> =
            vec![("arm".to_string(), &analysis.arm_readings)];
        for control in CONTROL_READERS {
            readers.push((
                control_reader_name(control).to_string(),
                control_readings_of(analysis, control),
            ));
        }
        for (reader_name, readings) in readers {
            lines.push(format!(
                "name=class_consistency arm={} reader={reader_name} classes={} classes_with_differing_decisions={}",
                arm_name(analysis.arm),
                readings.classes,
                zero_or_count(readings.classes_with_differing_decisions),
            ));
        }
    }
    lines
}

fn literal_minus_avoidable_lines(analyses: &[ArmAnalysis]) -> Vec<String> {
    analyses
        .iter()
        .map(|analysis| {
            let literal = analysis.arm_readings.counts[ReadingKind::LiteralMisrefusal as usize];
            let avoidable = analysis.arm_readings.counts[ReadingKind::AvoidableMisrefusal as usize];
            format!(
                "name=literal_minus_avoidable arm={} literal={literal} avoidable={avoidable} difference={}",
                arm_name(analysis.arm),
                i64_from_usize(literal) - i64_from_usize(avoidable)
            )
        })
        .collect()
}

/// 轨迹：每条臂每个读数按断电点序号各报一个计数，连同峰值、为正的断电点个数与两个期末值（登记第八节）。
fn trajectory_lines(analyses: &[ArmAnalysis]) -> Vec<String> {
    let mut lines = Vec::new();
    for analysis in analyses {
        for reading in ALL_READINGS {
            let per_point: Vec<usize> = analysis
                .arm_readings
                .trajectory
                .iter()
                .map(|counts| counts[reading as usize])
                .collect();
            let peak = *per_point
                .iter()
                .max()
                .expect("the trajectory has at least one crash point");
            let peak_at = per_point
                .iter()
                .position(|count| *count == peak)
                .expect("the peak is one of the counts");
            let positive_points = per_point.iter().filter(|count| **count > 0).count();
            let counts_text = per_point
                .iter()
                .map(|count| count.to_string())
                .collect::<Vec<String>>()
                .join(",");
            lines.push(format!(
                "name=trajectory arm={} reading={} counts={counts_text} peak={peak} peak_at={peak_at} positive_points={positive_points} of={} \
                 end_of_use={} last_segment_fully_persisted={}",
                arm_name(analysis.arm),
                reading_name(reading),
                per_point.len(),
                per_point[analysis.set.crash_point_count - 1],
                per_point[analysis.set.last_segment_fully_persisted_index],
            ));
        }
    }
    lines
}
/// 一个读者的某个字段在主格与变体上的两个值。
pub(super) struct FieldDifference {
    pub(super) reader: String,
    pub(super) field: String,
    main_value: String,
    variant_value: String,
}

fn differences_between_readings(
    reader_name: &str,
    main: &ReaderReadings,
    variant: &ReaderReadings,
    differences: &mut Vec<FieldDifference>,
) {
    let mut push = |field: String, main_value: String, variant_value: String| {
        differences.push(FieldDifference {
            reader: reader_name.to_string(),
            field,
            main_value,
            variant_value,
        });
    };
    for reading in ALL_READINGS {
        let index = reading as usize;
        if main.counts[index] != variant.counts[index] {
            push(
                reading_name(reading).to_string(),
                main.counts[index].to_string(),
                variant.counts[index].to_string(),
            );
        }
        if main.counts_on_early_image_histories[index]
            != variant.counts_on_early_image_histories[index]
        {
            push(
                format!("{}_on_early_image_histories", reading_name(reading)),
                main.counts_on_early_image_histories[index].to_string(),
                variant.counts_on_early_image_histories[index].to_string(),
            );
        }
    }
    let crash_points_with_different_counts = main
        .trajectory
        .iter()
        .zip(variant.trajectory.iter())
        .filter(|(main_counts, variant_counts)| main_counts != variant_counts)
        .count()
        + main.trajectory.len().abs_diff(variant.trajectory.len());
    if crash_points_with_different_counts > 0 {
        push(
            "trajectory_crash_points_with_different_counts".to_string(),
            "0".to_string(),
            crash_points_with_different_counts.to_string(),
        );
    }
    if main.classes != variant.classes {
        push(
            "classes".to_string(),
            main.classes.to_string(),
            variant.classes.to_string(),
        );
    }
    if main.classes_with_differing_decisions != variant.classes_with_differing_decisions {
        push(
            "classes_with_differing_decisions".to_string(),
            main.classes_with_differing_decisions.to_string(),
            variant.classes_with_differing_decisions.to_string(),
        );
    }
    if main.whole_pool_refusals != variant.whole_pool_refusals {
        push(
            "whole_pool_refusals".to_string(),
            main.whole_pool_refusals.to_string(),
            variant.whole_pool_refusals.to_string(),
        );
    }
}

/// 两份分析里逐项不同的读数（臂自己与三条对照读者，字段与 `ReaderReadings` 一一对应），相同时为空。
pub(super) fn differing_fields(main: &ArmAnalysis, variant: &ArmAnalysis) -> Vec<FieldDifference> {
    let mut differences = Vec::new();
    differences_between_readings(
        "arm",
        &main.arm_readings,
        &variant.arm_readings,
        &mut differences,
    );
    for (control, main_readings) in &main.control_readings {
        differences_between_readings(
            control_reader_name(*control),
            main_readings,
            control_readings_of(variant, *control),
            &mut differences,
        );
    }
    differences
}

/// 不变性核对里一个变体与主格逐臂比的结果。登记第十节第 4 条的原文触发（`in_registered_clause`）是世代号奇偶的另三个组合加
/// 「再变大每盘各加 100」的另一组整数；另补的核对（`in_order_preserving_check`）是奇偶的另三个组合加把探测值一并换数的另一组整数。
struct InvarianceVariant {
    in_registered_clause: bool,
    in_order_preserving_check: bool,
    mismatching_arms: usize,
}

pub(super) struct InvarianceCheck {
    pub(super) lines: Vec<String>,
    variants: Vec<InvarianceVariant>,
}

impl InvarianceCheck {
    fn compare(
        &mut self,
        variant_name: &str,
        in_registered_clause: bool,
        in_order_preserving_check: bool,
        main_analyses: &[ArmAnalysis],
        variant_analyses: &[ArmAnalysis],
    ) {
        let mut mismatching_arms = 0;
        for main in main_analyses {
            let variant = analysis_of(variant_analyses, main.arm);
            let agrees = analyses_agree(main, variant);
            self.lines.push(format!(
                "name=invariance arm={} variant={variant_name} equal_to_main={agrees}",
                arm_name(main.arm)
            ));
            if !agrees {
                mismatching_arms += 1;
                for difference in differing_fields(main, variant) {
                    self.lines.push(format!(
                        "name=invariance_detail arm={} variant={variant_name} compared_reader={} field={} main={} variant_value={}",
                        arm_name(main.arm),
                        difference.reader,
                        difference.field,
                        difference.main_value,
                        difference.variant_value
                    ));
                }
            }
        }
        self.variants.push(InvarianceVariant {
            in_registered_clause,
            in_order_preserving_check,
            mismatching_arms,
        });
    }

    pub(super) fn mismatches_in_registered_clause(&self) -> usize {
        self.variants
            .iter()
            .filter(|variant| variant.in_registered_clause)
            .map(|variant| variant.mismatching_arms)
            .sum()
    }

    pub(super) fn mismatches_in_order_preserving_check(&self) -> usize {
        self.variants
            .iter()
            .filter(|variant| variant.in_order_preserving_check)
            .map(|variant| variant.mismatching_arms)
            .sum()
    }
}

/// 不变性核对（登记第八节与第十节第 4 条）：世代号奇偶的另三个组合与两种另一组整数的写法上，全部读数与主格逐项比，不同的臂逐字段列出。
/// 读者规则只比大小次序，所以奇偶的三个组合与把探测值一并换数的写法预期逐项相同；「再变大」在换过的后尺寸上直接加 100 的写法
/// 把主格里 200 加 100 等于 300 这类跨形状的相等拆开了，观测类的分法随之变，类数与按类算的读数预期会变，逐字段列出。
pub(super) fn invariance_lines(main_analyses: &[ArmAnalysis], threads: usize) -> InvarianceCheck {
    let mut check = InvarianceCheck {
        lines: Vec::new(),
        variants: Vec::new(),
    };
    for combination in NEWEST_SLOT_COMBINATIONS_BESIDES_THE_MAIN_GRID {
        let variant_analyses = analyse_arms(combination, Numbering::Main, threads);
        let variant_name = format!("newest_slots_{}_{}", combination[0], combination[1]);
        check.compare(&variant_name, true, true, main_analyses, &variant_analyses);
    }
    for (numbering, in_registered_clause, in_order_preserving_check) in [
        (Numbering::OrderPreservingOnEveryNumber, false, true),
        (Numbering::OrderPreservingWithFixedGrowthStep, true, false),
    ] {
        let variant_analyses = analyse_arms(NEWEST_SLOTS_OF_THE_MAIN_GRID, numbering, threads);
        check.compare(
            numbering_name(numbering),
            in_registered_clause,
            in_order_preserving_check,
            main_analyses,
            &variant_analyses,
        );
    }
    check
}

/// 登记第四节第 8 至 13 条的手推结局：装置读数与手推对不上时先查装置，再记「手推被推翻」（第十节第 9 条）。
struct HandDerivation {
    claim: &'static str,
    holds: bool,
    reading: String,
}

fn hand_derivations(analyses: &[ArmAnalysis]) -> Vec<HandDerivation> {
    let count_of = |arm: Arm, reading: ReadingKind| {
        analysis_of(analyses, arm).arm_readings.counts[reading as usize]
    };
    let early_count_of = |arm: Arm, reading: ReadingKind| {
        analysis_of(analyses, arm)
            .arm_readings
            .counts_on_early_image_histories[reading as usize]
    };
    let literal_per_arm: Vec<String> = ARMS
        .iter()
        .map(|arm| {
            format!(
                "{}={}",
                arm_name(*arm),
                count_of(*arm, ReadingKind::LiteralMisrefusal)
            )
        })
        .collect();
    let chain_early_writable =
        early_count_of(Arm::ChainThenLowest, ReadingKind::LeakThroughWritableMount);
    let chain_early_read_only =
        early_count_of(Arm::ChainThenLowest, ReadingKind::LeakThroughReadOnlyMount);
    let highest_writable = count_of(Arm::HighestTable, ReadingKind::LeakThroughWritableMount);
    let highest_read_only = count_of(Arm::HighestTable, ReadingKind::LeakThroughReadOnlyMount);
    let highest_avoidable = count_of(Arm::HighestTable, ReadingKind::AvoidableMisrefusal);
    let probe_early_writable = early_count_of(
        Arm::LowestToReadHighestToProbeWithAbandon,
        ReadingKind::LeakThroughWritableMount,
    );
    let baseline_avoidable = count_of(
        Arm::OnlyIfEveryTableIsEqual,
        ReadingKind::AvoidableMisrefusal,
    );
    let chain_whole_pool_refusals = analysis_of(analyses, Arm::ChainThenLowest)
        .arm_readings
        .whole_pool_refusals;
    vec![
        HandDerivation {
            claim: "literal_misrefusal_on_every_arm",
            holds: ARMS.iter().all(|arm| count_of(*arm, ReadingKind::LiteralMisrefusal) >= 1),
            reading: literal_per_arm.join(","),
        },
        HandDerivation {
            claim: "chain_then_lowest_leaks_on_early_image",
            holds: chain_early_writable >= 1 && chain_early_read_only >= 1,
            reading: format!("early_image_leak_writable={chain_early_writable},early_image_leak_read_only={chain_early_read_only}"),
        },
        HandDerivation {
            claim: "highest_table_has_no_leak",
            holds: highest_writable == 0 && highest_read_only == 0,
            reading: format!("leak_writable={highest_writable},leak_read_only={highest_read_only}"),
        },
        HandDerivation {
            claim: "highest_table_misrefuses_avoidably",
            holds: highest_avoidable >= 1,
            reading: format!("misrefusal_avoidable={highest_avoidable}"),
        },
        HandDerivation {
            claim: "lowest_to_read_highest_to_probe_leaks_on_early_image",
            holds: probe_early_writable >= 1,
            reading: format!("early_image_leak_writable={probe_early_writable}"),
        },
        HandDerivation {
            claim: "baseline_misrefuses_avoidably",
            holds: baseline_avoidable >= 1,
            reading: format!("misrefusal_avoidable={baseline_avoidable}"),
        },
        HandDerivation {
            claim: "chain_then_lowest_never_refuses_the_whole_pool",
            holds: chain_whole_pool_refusals == 0,
            reading: format!("whole_pool_refusals={chain_whole_pool_refusals}"),
        },
    ]
}

fn describe_board(board: &Board) -> String {
    let mut disks = Vec::new();
    for (disk, slots) in board.iter().enumerate() {
        let described: Vec<String> = slots
            .iter()
            .map(|slot| match slot {
                Slot::Unreadable => "unreadable".to_string(),
                Slot::SelfVerified {
                    table,
                    carries_intent_marker,
                    ..
                } => {
                    format!(
                        "table({}){}",
                        describe_table(*table),
                        if *carries_intent_marker {
                            "+marker"
                        } else {
                            ""
                        }
                    )
                }
            })
            .collect();
        disks.push(format!("disk{disk}:[{}]", described.join(";")));
    }
    disks.join("|")
}

/// 手推说某条臂在「换回早镜像」上漏放行时，打印这样的第一条历史的盘面、探测值、落点上界与该臂的决定。
fn first_early_image_leak_witness(claim: &str, analysis: &ArmAnalysis) -> String {
    for observation in &analysis.set.observations {
        if !observation.is_early_image {
            continue;
        }
        let decision = reader_decision(Reader::Arm(analysis.arm), observation, &analysis.set);
        let beyond = beyond_the_end(observation.landing, observation.probe_sizes);
        if is_leak_through_writable_mount(decision, beyond)
            || is_leak_through_read_only_mount(decision, beyond)
        {
            return format!(
                "name=hand_derivation_witness claim={claim} board={} probe_sizes={} landing={} decision={}",
                describe_board(&observation.board),
                describe_table(observation.probe_sizes),
                describe_table(observation.landing),
                describe_decision(decision).replace(' ', ",")
            );
        }
    }
    format!("name=hand_derivation_witness claim={claim} witness=none")
}
/// 判决行：每一项检查各占一行；字段取 `true` 或 `zero` 是过，`false` 或大于 0 的违例计数是没过。
fn verdict_lines(
    analyses: &[ArmAnalysis],
    invariance: &InvarianceCheck,
    derivations: &[HandDerivation],
) -> Vec<String> {
    let allow_everything_leaks_on_every_arm = analyses.iter().all(|analysis| {
        let counts = control_readings_of(analysis, ControlReader::AllowEverything).counts;
        counts[ReadingKind::LeakThroughWritableMount as usize] > 0
            && counts[ReadingKind::LeakThroughReadOnlyMount as usize] > 0
    });
    let reject_every_writable_mount_misrefuses_on_every_arm = analyses.iter().all(|analysis| {
        let counts = control_readings_of(analysis, ControlReader::RejectEveryWritableMount).counts;
        counts[ReadingKind::LiteralMisrefusal as usize] > 0
            && counts[ReadingKind::AvoidableMisrefusal as usize] > 0
    });
    let negative_control_violations: usize = analyses
        .iter()
        .map(|analysis| {
            let counts =
                control_readings_of(analysis, ControlReader::NameWhatTheClassOverruns).counts;
            counts[ReadingKind::LeakThroughWritableMount as usize]
                + counts[ReadingKind::LeakThroughReadOnlyMount as usize]
                + counts[ReadingKind::AvoidableMisrefusal as usize]
        })
        .sum();
    let differing_class_violations: usize = analyses
        .iter()
        .map(|analysis| {
            analysis.arm_readings.classes_with_differing_decisions
                + analysis
                    .control_readings
                    .iter()
                    .map(|(_, readings)| readings.classes_with_differing_decisions)
                    .sum::<usize>()
        })
        .sum();
    let at_least_one_arm_literal_exceeds_avoidable = analyses.iter().any(|analysis| {
        analysis.arm_readings.counts[ReadingKind::LiteralMisrefusal as usize]
            > analysis.arm_readings.counts[ReadingKind::AvoidableMisrefusal as usize]
    });
    let out_of_model_violations: usize = analyses
        .iter()
        .map(|analysis| analysis.set.out_of_model_histories)
        .sum();
    let history_count_mismatches: usize = analyses
        .iter()
        .map(|analysis| {
            let (raw_states, distinct_boards, writable_tuples, grown_family, never_grown_family) =
                expected_history_counts(analysis.arm);
            let grown_family_observed = grown_family_history_count(&analysis.set);
            let observed = [
                analysis.set.raw_state_count,
                analysis.set.distinct_board_count,
                analysis.set.observations.len(),
                grown_family_observed,
                analysis.set.observations.len() - grown_family_observed,
            ];
            let expected = [
                raw_states,
                distinct_boards,
                writable_tuples,
                grown_family,
                never_grown_family,
            ];
            (0..observed.len())
                .filter(|position| observed[*position] != expected[*position])
                .count()
        })
        .sum();
    let hand_derivations_overturned = derivations
        .iter()
        .filter(|derivation| !derivation.holds)
        .count();
    let candidate_readings: Vec<[usize; 4]> = ARMS[..4]
        .iter()
        .map(|arm| {
            let counts = analysis_of(analyses, *arm).arm_readings.counts;
            [
                counts[ReadingKind::LeakThroughWritableMount as usize],
                counts[ReadingKind::LeakThroughReadOnlyMount as usize],
                counts[ReadingKind::LiteralMisrefusal as usize],
                counts[ReadingKind::AvoidableMisrefusal as usize],
            ]
        })
        .collect();
    let any_two_of_the_four_candidate_arms_differ = candidate_readings
        .iter()
        .any(|readings| *readings != candidate_readings[0]);
    vec![
        format!(
            "name=verdict check=positive_controls allow_everything_leaks_on_every_arm={allow_everything_leaks_on_every_arm} \
             reject_every_writable_mount_misrefuses_on_every_arm={reject_every_writable_mount_misrefuses_on_every_arm}"
        ),
        format!("name=verdict check=negative_controls negative_control_violations={}", zero_or_count(negative_control_violations)),
        format!("name=verdict check=decision_is_function_of_observation differing_class_violations={}", zero_or_count(differing_class_violations)),
        format!("name=verdict check=literal_minus_avoidable at_least_one_arm_literal_exceeds_avoidable={at_least_one_arm_literal_exceeds_avoidable}"),
        format!("name=verdict check=invariance invariance_mismatches={}", zero_or_count(invariance.mismatches_in_registered_clause())),
        format!("name=verdict check=invariance_order_preserving order_preserving_mismatches={}", zero_or_count(invariance.mismatches_in_order_preserving_check())),
        format!("name=verdict check=out_of_model out_of_model_violations={}", zero_or_count(out_of_model_violations)),
        format!("name=verdict check=history_counts history_count_mismatches={}", zero_or_count(history_count_mismatches)),
        format!("name=verdict check=hand_derivations hand_derivations_overturned={}", zero_or_count(hand_derivations_overturned)),
        format!("name=verdict check=arms_distinguishable any_two_of_the_four_candidate_arms_differ={any_two_of_the_four_candidate_arms_differ}"),
    ]
}

/// 岔路单行 1：四条候选读者规则加基线臂在同一组历史上的读数，各带分母（命令行参数 `reader-rules-on-histories`）。
pub(super) fn reader_rules_on_histories_lines(threads: usize) -> Vec<String> {
    let main_analyses = analyse_arms(NEWEST_SLOTS_OF_THE_MAIN_GRID, Numbering::Main, threads);
    let unit_count = reader_units().len();
    let mut lines = vec![format!(
        "name=config segment=reader-rules-on-histories configured_threads={threads} history_set_phase_threads={} reader_phase_threads={} \
         arms={} reader_units={unit_count} shapes={}",
        threads_used_for(ARMS.len(), threads),
        threads_used_for(unit_count, threads),
        ARMS.len(),
        registered_shapes().len()
    )];
    lines.extend(history_space_lines(&main_analyses));
    lines.extend(reading_lines(&main_analyses));
    lines.extend(control_lines(&main_analyses));
    lines.extend(class_consistency_lines(&main_analyses));
    lines.extend(literal_minus_avoidable_lines(&main_analyses));
    lines.extend(trajectory_lines(&main_analyses));
    let invariance = invariance_lines(&main_analyses, threads);
    lines.extend(invariance.lines.iter().cloned());
    let derivations = hand_derivations(&main_analyses);
    for derivation in &derivations {
        lines.push(format!(
            "name=hand_derivation claim={} holds={} reading={}",
            derivation.claim, derivation.holds, derivation.reading
        ));
    }
    lines.push(first_early_image_leak_witness(
        "chain_then_lowest_leaks_on_early_image",
        analysis_of(&main_analyses, Arm::ChainThenLowest),
    ));
    lines.push(first_early_image_leak_witness(
        "lowest_to_read_highest_to_probe_leaks_on_early_image",
        analysis_of(&main_analyses, Arm::LowestToReadHighestToProbeWithAbandon),
    ));
    lines.push("name=not_run_after_sufficient item=first_five_witnesses_and_distribution_by_action_and_persistence_segment".to_string());
    lines.push("name=not_run_after_sufficient item=read_only_refusal_count_class_count_and_class_size_histogram".to_string());
    lines.push("name=not_run_this_segment item=existence_judgments_on_the_geometry_sensitivity_sampling_points reason=stability_not_dispatched".to_string());
    lines.push("name=not_run_this_segment item=discrimination_self_proof_of_the_geometry_sensitivity reason=stability_not_dispatched".to_string());
    lines.extend(verdict_lines(&main_analyses, &invariance, &derivations));
    lines
}
