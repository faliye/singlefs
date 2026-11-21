use super::*;

// ============================================================================
// 十三、判定：每个量一行（登记第六节），阳性对照、锚点、几何敏感性、失败与作废条款逐条，两行岔路各一行（不合成一个真假）。
// ============================================================================

/// 一个量在一组上的读数：对数、张数、洞对总数、判了的张数。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct QuestionReading {
    pub(super) pairs: u64,
    pub(super) images: u64,
    pub(super) hole_pairs: u64,
    pub(super) judged_images: u64,
    pub(super) unreadable_images: u64,
}

fn question_reading(
    results: &[HistoryResult],
    group: Group,
    point: SamplingPoint,
    only_swept: bool,
    arm_error: ArmError,
) -> QuestionReading {
    let totals = totals_of(&results_of(results, group, point), only_swept);
    QuestionReading {
        pairs: arm_error.of(&totals.pairs),
        images: arm_error.of(&totals.images_with_a_positive_count),
        hole_pairs: totals.hole_pairs,
        judged_images: totals.judged_images,
        unreadable_images: totals.unreadable_images,
    }
}

/// 错分类量的判定词：洞对一对都没有记「没有对象」（不记分对，登记 Q18）；0 记分对；≥ 1 记乙。
fn misclassification_judgement(reading: QuestionReading) -> &'static str {
    if reading.judged_images == 0 {
        "not_run"
    } else if reading.hole_pairs == 0 {
        "no_object"
    } else if reading.pairs == 0 {
        "classified_right"
    } else {
        "branch_two"
    }
}

fn question_line(
    question: &str,
    group: Group,
    point: SamplingPoint,
    scope: &str,
    arm_error: ArmError,
    reading: QuestionReading,
) -> String {
    format!(
        "name=verdict question={question} group={} point={} scope={scope} arm={} error={} value_pairs={} value_images={} hole_pairs={} judged_images={} unreadable_images={} judgement={}",
        group.name(),
        point.name(),
        arm_error.arm(),
        arm_error.error(),
        reading.pairs,
        reading.images,
        reading.hole_pairs,
        reading.judged_images,
        reading.unreadable_images,
        misclassification_judgement(reading)
    )
}

/// 几何敏感性一行（登记第八节、`.claude/rules/mutation-sampling.md` 第六类）：判定同向记两点、反向记不稳定；判别力自证把门槛挪到两值之间。
fn sensitivity_line(
    point: SamplingPoint,
    question: &str,
    main_value: u64,
    sensitive_value: u64,
) -> String {
    let record = if (main_value == 0) == (sensitive_value == 0) {
        "two_points"
    } else {
        "unstable"
    };
    let discriminating = if main_value != sensitive_value {
        "proven_threshold_between_the_two_values_splits_them"
    } else if main_value == 0 {
        "both_zero_threshold_cannot_move_between"
    } else {
        "equal_values_threshold_at_one_half_judges_both_the_same_not_proven"
    };
    format!(
        "name=sensitivity point={} question={question} main={main_value} sensitive={sensitive_value} record={record} discriminating={discriminating}",
        point.name()
    )
}

pub(super) fn anchor_line(
    anchor: &str,
    kind: &str,
    subject: &str,
    expected: &str,
    observed: &str,
) -> String {
    format!(
        "name=anchor anchor={anchor} kind={kind} subject={subject} expected={expected} observed={observed} holds={}",
        expected == observed
    )
}

/// C1、C2 这类钉一张的洞对表：交回 (洞对数, 第一对的 R⁻.txg, R.txg, R 的出生 txg, 真值, 两条臂认不认, 两条臂的 E1、E2)。
fn pinned_image_text(result: Option<&HistoryResult>, step: u64) -> String {
    let Some(image) =
        result.and_then(|result| result.images.iter().find(|image| image.step == step))
    else {
        return "image_missing".to_string();
    };
    let Ok(reading) = &image.reading else {
        return "image_unreadable".to_string();
    };
    let pairs: Vec<String> = reading
        .hole_pairs
        .iter()
        .zip(&image.truths)
        .map(|(pair, truth)| {
            format!(
                "({},{}):birth{}:{}:today{}:with_pointer_birth{}",
                pair.previous_txg,
                pair.version.key.txg,
                pointer_birth_text(pair.version.pointer_birth),
                truth.name(),
                u8::from(pair.bridged_today),
                u8::from(pair.bridged_with_the_pointer_birth)
            )
        })
        .collect();
    format!(
        "pairs{}:{}:today_e1_{}:today_e2_{}:with_pointer_birth_e1_{}:with_pointer_birth_e2_{}",
        reading.hole_pairs.len(),
        pairs.join(";"),
        image.counts.today_e1,
        image.counts.today_e2,
        image.counts.with_pointer_birth_e1,
        image.counts.with_pointer_birth_e2
    )
}

/// 登记第七节 C1：nrb（last 10、k 3、bad 1）那一张。期望的写法与 `pinned_image_text` 同形，数抄自登记第七节与第十三节 `anchors.py` 的输出。
const NEWEST_ROW_BAD_PIN_EXPECTED: &str = "pairs1:(10,12):birth11:existed_versions_in_between:today1:with_pointer_birth0:today_e1_1:today_e2_0:with_pointer_birth_e1_0:with_pointer_birth_e2_0";
/// 登记第七节 C2：skr（last 6、k 0）那一张。
const SKIPPED_ROW_PIN_EXPECTED: &str = "pairs1:(6,8):birth8:directly_bridged:today1:with_pointer_birth1:today_e1_0:today_e2_0:with_pointer_birth_e1_0:with_pointer_birth_e2_0";

fn find_history<'results>(
    results: &'results [HistoryResult],
    specification: &HistorySpecification,
) -> Option<&'results HistoryResult> {
    results
        .iter()
        .find(|result| &result.specification == specification)
}

pub(super) fn newest_row_bad_pin_specification() -> HistorySpecification {
    HistorySpecification {
        group: Group::NewestRowBad,
        point: SamplingPoint::Main,
        slots_per_region: 4,
        journal_ring_bytes: SHORT_JOURNAL_RING_BYTES,
        shape: HistoryShape::NewestRowBad {
            last: 10,
            steps_after_the_mount: 3,
            bad_slots: 1,
            user_write: UserWrite::Overwrite,
        },
    }
}

pub(super) fn skipped_row_pin_specification() -> HistorySpecification {
    HistorySpecification {
        group: Group::SkippedRow,
        point: SamplingPoint::Main,
        slots_per_region: 4,
        journal_ring_bytes: SHORT_JOURNAL_RING_BYTES,
        shape: HistoryShape::SkippedRow { last: 6 },
    }
}

fn joined_or_none(items: &[String]) -> String {
    if items.is_empty() {
        "none".to_string()
    } else {
        items.join(",")
    }
}

pub(super) fn yes_or_no(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}

fn clause_line(clause: &str, triggered: bool, observation: &str) -> String {
    format!(
        "name=clause clause={clause} triggered={} observation={}",
        yes_or_no(triggered),
        observation.replace(' ', "_")
    )
}

/// 今天的池级 checker 在这条实验里取的两格。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CheckedInvariant {
    ReleaseGenerationInterval,
    ReferenceIntervalPrecision,
}

/// 一组镜像上那一格判违例的张数与各判定的张数。
fn checker_violations(
    results: &[HistoryResult],
    group: Group,
    point: SamplingPoint,
    invariant: CheckedInvariant,
) -> (u64, String) {
    let totals = totals_of(&results_of(results, group, point), false);
    let counts = match invariant {
        CheckedInvariant::ReleaseGenerationInterval => &totals.release_generation_verdicts,
        CheckedInvariant::ReferenceIntervalPrecision => &totals.reference_interval_verdicts,
    };
    (
        counts.get("violated").copied().unwrap_or(0),
        count_text(counts),
    )
}

/// 张数锚点：(A 或 B 的行, 张数对不上的锚名)。
fn image_count_anchor_lines(
    specifications: &[HistorySpecification],
    results: &[HistoryResult],
    segment_groups: &[Group],
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut lines = Vec::new();
    let mut independent_failures = Vec::new();
    let mut clause_failures = Vec::new();
    for (anchor, group, point, expected) in ATTEMPTED_IMAGE_ANCHORS {
        if !segment_groups.contains(&group)
            || !specifications
                .iter()
                .any(|specification| specification.group == group && specification.point == point)
        {
            continue;
        }
        let observed = attempted_images_of(specifications, group, point)
            .map_or("unknown".to_string(), |count| count.to_string());
        let line = anchor_line(
            anchor,
            "independent",
            &format!("attempted_images.{}.{}", group.name(), point.name()),
            &expected.to_string(),
            &observed,
        );
        if observed != expected.to_string() {
            independent_failures.push(format!("{anchor}.{}.{}", group.name(), point.name()));
        }
        lines.push(line);
    }
    for (anchor, group, expected) in CLAUSE_IMAGE_ANCHORS {
        if !segment_groups.contains(&group) {
            continue;
        }
        let produced =
            totals_of(&results_of(results, group, SamplingPoint::Main), false).produced_images;
        if produced != expected {
            clause_failures.push(format!("{anchor}.{}", group.name()));
        }
        lines.push(anchor_line(
            anchor,
            "clause",
            &format!("produced_images.{}.main", group.name()),
            &expected.to_string(),
            &produced.to_string(),
        ));
    }
    (lines, independent_failures, clause_failures)
}

fn premise_question_lines(results: &[HistoryResult], scope: &str) -> Vec<String> {
    let all: Vec<&HistoryResult> = results.iter().collect();
    let totals = totals_of(&all, false);
    let judge = |value: u64| {
        if value == 0 {
            "premise_holds_on_these_groups"
        } else {
            "branch_two"
        }
    };
    vec![
        format!("name=verdict question=Q1 scope={scope} value={} sweep_pointer_changes={} judgement={}", totals.adjacent_version_violations, totals.sweep_pointer_changes, judge(totals.adjacent_version_violations)),
        format!("name=verdict question=Q2 scope={scope} value={} instance_one_reported_separately={} judgement={}", totals.first_version_violations, totals.instance_one_first_version_mismatches, judge(totals.first_version_violations)),
        format!("name=verdict question=Q3 scope={scope} value={} judgement={}", totals.birth_after_the_txg_violations, judge(totals.birth_after_the_txg_violations)),
    ]
}

/// PC-P1：每段历史实例 1 的第一版都报一处不成立（指针出生 txg = 0）。交回 (行, 没中的段数)。
fn instance_one_control_line(results: &[HistoryResult]) -> (String, usize) {
    let missed = results
        .iter()
        .filter(|result| result.premise.instance_one_first_version_mismatches != 1)
        .count();
    (
        format!("name=control control=PC-P1 histories={} histories_not_reporting_exactly_one={missed} hit={}", results.len(), yes_or_no(missed == 0)),
        missed,
    )
}

/// 第一段（补第 2 行）：nrb、rs1 与 G4。
pub(super) fn judgement_lines_of_segment_one(
    specifications: &[HistorySpecification],
    results: &[HistoryResult],
    stops: &StopClauses,
) -> Vec<String> {
    let mut lines = Vec::new();
    let main = SamplingPoint::Main;
    let wider = SamplingPoint::NewestRowAndRollbackWiderRing;
    let (anchor_lines, mut independent_failures, clause_failures) = image_count_anchor_lines(
        specifications,
        results,
        &[Group::NewestRowBad, Group::RollbackSlotBad],
    );
    lines.extend(anchor_lines);
    let newest_row_bad_pin_observed = pinned_image_text(
        find_history(results, &newest_row_bad_pin_specification()),
        0,
    );
    lines.push(anchor_line(
        "C1",
        "independent",
        "nrb.main.S4.last10.k3.bad1",
        NEWEST_ROW_BAD_PIN_EXPECTED,
        &newest_row_bad_pin_observed,
    ));
    if newest_row_bad_pin_observed != NEWEST_ROW_BAD_PIN_EXPECTED {
        independent_failures.push("C1".to_string());
    }
    let nrb_today = question_reading(results, Group::NewestRowBad, main, false, ArmError::TodayE1);
    let rs1_today = question_reading(
        results,
        Group::RollbackSlotBad,
        main,
        false,
        ArmError::TodayE1,
    );
    let nrb_with = question_reading(
        results,
        Group::NewestRowBad,
        main,
        false,
        ArmError::WithPointerBirthE1,
    );
    let rs1_with = question_reading(
        results,
        Group::RollbackSlotBad,
        main,
        false,
        ArmError::WithPointerBirthE1,
    );
    let (rollback_release_generation_violations, rollback_release_generation_verdicts) =
        checker_violations(
            results,
            Group::RollbackSlotBad,
            main,
            CheckedInvariant::ReleaseGenerationInterval,
        );
    let (newest_row_release_generation_violations, newest_row_release_generation_verdicts) =
        checker_violations(
            results,
            Group::NewestRowBad,
            main,
            CheckedInvariant::ReleaseGenerationInterval,
        );
    let clause_number_mismatches: Vec<&str> = [
        ("A5", nrb_today.images != 6),
        ("A6", rs1_today.images != 285),
        ("A7", rollback_release_generation_violations != 150),
    ]
    .into_iter()
    .filter(|(_, differs)| *differs)
    .map(|(anchor, _)| anchor)
    .collect();
    lines.push(anchor_line(
        "A5",
        "clause",
        "today_e1_images.nrb.main",
        "6",
        &nrb_today.images.to_string(),
    ));
    lines.push(anchor_line(
        "A6",
        "clause",
        "today_e1_images.rs1.main",
        "285",
        &rs1_today.images.to_string(),
    ));
    lines.push(anchor_line(
        "A7",
        "clause",
        "i39_violated_images.rs1.main",
        "150",
        &rollback_release_generation_violations.to_string(),
    ));
    let nrb_produced =
        totals_of(&results_of(results, Group::NewestRowBad, main), false).produced_images;
    lines.push(format!(
        "name=control control=PC-H1A nrb_images={nrb_produced} nrb_images_with_today_e1={} c1_today_e1_is_one={} hit={}",
        nrb_today.images,
        yes_or_no(newest_row_bad_pin_observed.contains(":today_e1_1:")),
        yes_or_no(nrb_today.images >= 1 && newest_row_bad_pin_observed.contains(":today_e1_1:"))
    ));
    let (instance_one_control_text, instance_one_control_misses) =
        instance_one_control_line(results);
    lines.push(instance_one_control_text);
    lines.push(question_line(
        "Q13",
        Group::NewestRowBad,
        main,
        "all_histories",
        ArmError::TodayE1,
        nrb_today,
    ));
    lines.push(question_line(
        "Q13",
        Group::RollbackSlotBad,
        main,
        "all_histories",
        ArmError::TodayE1,
        rs1_today,
    ));
    lines.push(question_line(
        "Q14",
        Group::NewestRowBad,
        main,
        "all_histories",
        ArmError::WithPointerBirthE1,
        nrb_with,
    ));
    lines.push(question_line(
        "Q15",
        Group::RollbackSlotBad,
        main,
        "all_histories",
        ArmError::WithPointerBirthE1,
        rs1_with,
    ));
    let release_generation_judgement = |violated: u64| {
        if violated == 0 {
            "release_generation_half_not_red"
        } else {
            "branch_two"
        }
    };
    lines.push(format!("name=verdict question=Q16 group=rs1 point=main value_images={rollback_release_generation_violations} verdicts={rollback_release_generation_verdicts} judgement={}", release_generation_judgement(rollback_release_generation_violations)));
    lines.push(format!("name=verdict question=Q17 group=nrb point=main value_images={newest_row_release_generation_violations} verdicts={newest_row_release_generation_verdicts} judgement={}", release_generation_judgement(newest_row_release_generation_violations)));
    for group in [Group::NewestRowBad, Group::RollbackSlotBad] {
        let totals = totals_of(&results_of(results, group, main), false);
        lines.push(format!(
            "name=verdict question=Q18 group={} point=main hole_pairs={} directly_bridged={} existed_versions_in_between={} previous_not_on_the_chain={} version_missing_from_the_ledger={} pointer_unreadable_pairs={} judgement=coverage_only",
            group.name(),
            totals.hole_pairs,
            truth_count(&totals.truth_counts, Truth::DirectlyBridged),
            truth_count(&totals.truth_counts, Truth::ExistedVersionsInBetween),
            truth_count(&totals.truth_counts, Truth::PreviousNotOnTheChain),
            truth_count(&totals.truth_counts, Truth::VersionMissingFromTheLedger),
            totals.pointer_unreadable_pairs
        ));
        let (_, reference_interval_verdicts_text) = checker_violations(
            results,
            group,
            main,
            CheckedInvariant::ReferenceIntervalPrecision,
        );
        lines.push(format!("name=verdict question=Q19 group={} point=main reference_interval_verdicts={reference_interval_verdicts_text} judgement=attached_not_judged", group.name()));
    }
    lines.extend(premise_question_lines(results, "segment_one_groups"));
    for (question, group, arm_error) in [
        ("Q13.nrb", Group::NewestRowBad, ArmError::TodayE1),
        ("Q13.rs1", Group::RollbackSlotBad, ArmError::TodayE1),
        ("Q14", Group::NewestRowBad, ArmError::WithPointerBirthE1),
        ("Q15", Group::RollbackSlotBad, ArmError::WithPointerBirthE1),
    ] {
        let main_value = question_reading(results, group, main, false, arm_error).pairs;
        let wider_value = question_reading(results, group, wider, false, arm_error).pairs;
        lines.push(sensitivity_line(wider, question, main_value, wider_value));
    }
    for (question, group) in [
        ("Q16", Group::RollbackSlotBad),
        ("Q17", Group::NewestRowBad),
    ] {
        let (main_value, _) = checker_violations(
            results,
            group,
            main,
            CheckedInvariant::ReleaseGenerationInterval,
        );
        let (wider_value, _) = checker_violations(
            results,
            group,
            wider,
            CheckedInvariant::ReleaseGenerationInterval,
        );
        lines.push(sensitivity_line(wider, question, main_value, wider_value));
    }
    let unreadable: u64 = [Group::NewestRowBad, Group::RollbackSlotBad]
        .iter()
        .flat_map(|group| POINT_ORDER.iter().map(move |point| (*group, *point)))
        .map(|(group, point)| {
            totals_of(&results_of(results, group, point), false).unreadable_images
        })
        .sum();
    let positive_control_of_today_missed =
        nrb_today.images == 0 || !newest_row_bad_pin_observed.contains(":today_e1_1:");
    let premise_controls_missed = instance_one_control_misses > 0;
    let independent_anchor_failed = !independent_failures.is_empty();
    lines.push(clause_line(
        "Z1",
        positive_control_of_today_missed,
        &format!(
            "nrb_today_e1_images={} c1={newest_row_bad_pin_observed}",
            nrb_today.images
        ),
    ));
    lines.push(clause_line(
        "Z3",
        premise_controls_missed,
        &format!("instance_one_control_histories_missed={instance_one_control_misses}"),
    ));
    lines.push(clause_line(
        "Z5",
        independent_anchor_failed,
        &format!(
            "failed_independent_anchors={}",
            if independent_failures.is_empty() {
                "none".to_string()
            } else {
                independent_failures.join(",")
            }
        ),
    ));
    lines.push(clause_line(
        "Z6",
        false,
        &format!("unreadable_images_listed_separately_not_counted={unreadable}"),
    ));
    lines.push(clause_line(
        "F3",
        nrb_with.pairs + rs1_with.pairs > 0,
        &format!(
            "nrb_with_pointer_birth_pairs={} rs1_with_pointer_birth_pairs={}",
            nrb_with.pairs, rs1_with.pairs
        ),
    ));
    lines.push(clause_line(
        "F4",
        rollback_release_generation_violations + newest_row_release_generation_violations > 0,
        &format!("rs1_release_generation_violated_images={rollback_release_generation_violations} nrb_release_generation_violated_images={newest_row_release_generation_violations}"),
    ));
    lines.push(clause_line(
        "F5",
        !clause_failures.is_empty(),
        &format!(
            "clause_image_anchors_unequal={}",
            if clause_failures.is_empty() {
                "none".to_string()
            } else {
                clause_failures.join(",")
            }
        ),
    ));
    lines.push(clause_line(
        "F7",
        rs1_today.images == 0,
        &format!("rs1_today_e1_images={}", rs1_today.images),
    ));
    lines.push(clause_line(
        "F8",
        !clause_number_mismatches.is_empty(),
        &format!(
            "clause_numbers_unequal={}",
            if clause_number_mismatches.is_empty() {
                "none".to_string()
            } else {
                clause_number_mismatches.join(",")
            }
        ),
    ));
    let void =
        positive_control_of_today_missed || premise_controls_missed || independent_anchor_failed;
    let all_made = nrb_produced == 6
        && totals_of(&results_of(results, Group::RollbackSlotBad, main), false).produced_images
            == 285;
    let status = row_status(stops, void, all_made);
    lines.push(format!("name=verdict row=2 half=reference_interval_precision value_pairs={} judgement={} status={status}", nrb_with.pairs + rs1_with.pairs, if nrb_with.pairs + rs1_with.pairs == 0 { "branch_one" } else { "branch_two" }));
    lines.push(format!("name=verdict row=2 half=release_generation_interval value_images={} judgement={} status={status}", rollback_release_generation_violations + newest_row_release_generation_violations, if rollback_release_generation_violations + newest_row_release_generation_violations == 0 { "branch_one" } else { "branch_two" }));
    lines
}

/// 一行岔路的状态：停机（装置与 `crates/` 对不上）> 作废（阳性对照或独立锚点不中）> 造全了够判 / 还差。
fn row_status(stops: &StopClauses, void: bool, all_made: bool) -> String {
    if !stops.triggered.is_empty() {
        format!("stopped_by_{}", stops.triggered.join("_"))
    } else if void {
        "void".to_string()
    } else if all_made {
        "enough_to_judge".to_string()
    } else {
        "enough_to_judge_on_the_images_made_short_of_the_clause_count".to_string()
    }
}

/// 第二段（补第 1 行）：ss、skr、sw-crash、sw-read 与 G1–G3。
pub(super) fn judgement_lines_of_segment_two(
    specifications: &[HistorySpecification],
    results: &[HistoryResult],
    stops: &StopClauses,
) -> Vec<String> {
    let mut lines = Vec::new();
    let main = SamplingPoint::Main;
    let (anchor_lines, mut independent_failures, clause_failures) = image_count_anchor_lines(
        specifications,
        results,
        &[
            Group::ShortRing,
            Group::SkippedRow,
            Group::SweepCrash,
            Group::SweepRead,
        ],
    );
    lines.extend(anchor_lines);
    let skipped_row_pin_observed =
        pinned_image_text(find_history(results, &skipped_row_pin_specification()), 0);
    lines.push(anchor_line(
        "C2",
        "independent",
        "skr.main.S4.last6.k0",
        SKIPPED_ROW_PIN_EXPECTED,
        &skipped_row_pin_observed,
    ));
    if skipped_row_pin_observed != SKIPPED_ROW_PIN_EXPECTED {
        independent_failures.push("C2".to_string());
    }
    let questions: [(&str, Group, bool, ArmError); 8] = [
        ("Q4", Group::ShortRing, false, ArmError::WithPointerBirthE1),
        ("Q5", Group::SkippedRow, false, ArmError::WithPointerBirthE1),
        ("Q6", Group::SweepCrash, true, ArmError::WithPointerBirthE1),
        ("Q7", Group::ShortRing, false, ArmError::WithPointerBirthE2),
        ("Q8", Group::SkippedRow, false, ArmError::WithPointerBirthE2),
        ("Q9", Group::SweepCrash, true, ArmError::WithPointerBirthE2),
        ("Q10", Group::SweepRead, true, ArmError::WithPointerBirthE1),
        ("Q11", Group::SweepRead, true, ArmError::WithPointerBirthE2),
    ];
    let mut misclassified_questions = Vec::new();
    for (question, group, only_swept, arm_error) in questions {
        let reading = question_reading(results, group, main, only_swept, arm_error);
        if reading.pairs > 0 {
            misclassified_questions.push(question);
        }
        let scope = if only_swept {
            "swept_histories"
        } else {
            "all_histories"
        };
        lines.push(question_line(
            question, group, main, scope, arm_error, reading,
        ));
    }
    for group in [
        Group::ShortRing,
        Group::SkippedRow,
        Group::SweepCrash,
        Group::SweepRead,
    ] {
        for arm_error in [ArmError::TodayE1, ArmError::TodayE2] {
            let reading = question_reading(results, group, main, false, arm_error);
            lines.push(question_line(
                "Q19",
                group,
                main,
                "all_histories_attached_not_judged",
                arm_error,
                reading,
            ));
        }
    }
    let mut swept_counts = Vec::new();
    for group in [Group::SweepCrash, Group::SweepRead] {
        let totals = totals_of(&results_of(results, group, main), false);
        swept_counts.push((group, totals.histories, totals.swept_histories));
        lines.push(format!(
            "name=verdict question=Q12 group={} point=main histories={} swept_histories={} sweep_txg_matches_the_prediction={} judgement=validity_gate_{}",
            group.name(),
            totals.histories,
            totals.swept_histories,
            totals.sweep_txg_matches_the_prediction,
            if totals.swept_histories == totals.histories { "every_history_swept" } else { "some_history_did_not_sweep_see_F6" }
        ));
    }
    for group in [
        Group::ShortRing,
        Group::SkippedRow,
        Group::SweepCrash,
        Group::SweepRead,
    ] {
        let totals = totals_of(&results_of(results, group, main), false);
        lines.push(format!(
            "name=verdict question=Q18 group={} point=main hole_pairs={} directly_bridged={} existed_versions_in_between={} previous_not_on_the_chain={} version_missing_from_the_ledger={} pointer_unreadable_pairs={} judgement={}",
            group.name(),
            totals.hole_pairs,
            truth_count(&totals.truth_counts, Truth::DirectlyBridged),
            truth_count(&totals.truth_counts, Truth::ExistedVersionsInBetween),
            truth_count(&totals.truth_counts, Truth::PreviousNotOnTheChain),
            truth_count(&totals.truth_counts, Truth::VersionMissingFromTheLedger),
            totals.pointer_unreadable_pairs,
            if totals.hole_pairs == 0 { "no_object_in_this_group" } else { "coverage_only" }
        ));
    }
    let premise_lines = premise_question_lines(results, "segment_two_groups");
    let all: Vec<&HistoryResult> = results.iter().collect();
    let premise_totals = totals_of(&all, false);
    lines.extend(premise_lines);
    let skr = totals_of(&results_of(results, Group::SkippedRow, main), false);
    let ss = totals_of(&results_of(results, Group::ShortRing, main), false);
    let skr_direct = truth_count(&skr.truth_counts, Truth::DirectlyBridged);
    let ss_in_between = truth_count(&ss.truth_counts, Truth::ExistedVersionsInBetween);
    lines.push(format!("name=control control=PC-truth skr_directly_bridged={skr_direct} ss_existed_versions_in_between={ss_in_between} hit={}", yes_or_no(skr_direct >= 1 && ss_in_between >= 1)));
    let sweep_read_swept = totals_of(&results_of(results, Group::SweepRead, main), true);
    lines.push(format!(
        "name=control control=PC-with-pointer-birth swept_histories={} with_pointer_birth_e1_pairs={} hit={}",
        sweep_read_swept.histories,
        sweep_read_swept.pairs.with_pointer_birth_e1,
        yes_or_no(sweep_read_swept.histories > 0 && sweep_read_swept.pairs.with_pointer_birth_e1 >= 1)
    ));
    let sweep_swept: Vec<&HistoryResult> = results
        .iter()
        .filter(|result| history_swept(result))
        .collect();
    let sweep_changes: u64 = sweep_swept
        .iter()
        .map(|result| result.premise.sweep_pointer_changes)
        .sum();
    lines.push(format!("name=control control=PC-P2 swept_histories={} sweep_pointer_changes={sweep_changes} hit={}", sweep_swept.len(), yes_or_no(!sweep_swept.is_empty() && sweep_changes >= 1)));
    let (instance_one_control_text, instance_one_control_misses) =
        instance_one_control_line(results);
    lines.push(instance_one_control_text);
    let sensitivity_cases: [(SamplingPoint, &str, Group, bool, ArmError); 8] = [
        (
            SamplingPoint::SweepThresholdProductDefault,
            "Q6",
            Group::SweepCrash,
            true,
            ArmError::WithPointerBirthE1,
        ),
        (
            SamplingPoint::SweepThresholdProductDefault,
            "Q9",
            Group::SweepCrash,
            true,
            ArmError::WithPointerBirthE2,
        ),
        (
            SamplingPoint::LongJournalRing,
            "Q4",
            Group::ShortRing,
            false,
            ArmError::WithPointerBirthE1,
        ),
        (
            SamplingPoint::LongJournalRing,
            "Q7",
            Group::ShortRing,
            false,
            ArmError::WithPointerBirthE2,
        ),
        (
            SamplingPoint::LongJournalRing,
            "Q19.today_E1",
            Group::ShortRing,
            false,
            ArmError::TodayE1,
        ),
        (
            SamplingPoint::LongJournalRing,
            "Q19.today_E2",
            Group::ShortRing,
            false,
            ArmError::TodayE2,
        ),
        (
            SamplingPoint::SkippedRowWiderRing,
            "Q5",
            Group::SkippedRow,
            false,
            ArmError::WithPointerBirthE1,
        ),
        (
            SamplingPoint::SkippedRowWiderRing,
            "Q8",
            Group::SkippedRow,
            false,
            ArmError::WithPointerBirthE2,
        ),
    ];
    for (point, question, group, only_swept, arm_error) in sensitivity_cases {
        let main_value = question_reading(results, group, main, only_swept, arm_error).pairs;
        let sensitive_value = question_reading(results, group, point, only_swept, arm_error).pairs;
        lines.push(sensitivity_line(
            point,
            question,
            main_value,
            sensitive_value,
        ));
    }
    let sweep_threshold_point = SamplingPoint::SweepThresholdProductDefault;
    let main_sweep = totals_of(&results_of(results, Group::SweepCrash, main), false);
    let sweep_threshold_point_totals = totals_of(
        &results_of(results, Group::SweepCrash, sweep_threshold_point),
        false,
    );
    lines.push(sensitivity_line(
        sweep_threshold_point,
        "Q1",
        main_sweep.adjacent_version_violations,
        sweep_threshold_point_totals.adjacent_version_violations,
    ));
    lines.push(sensitivity_line(
        sweep_threshold_point,
        "Q12.histories_without_a_sweep",
        main_sweep.histories - main_sweep.swept_histories,
        sweep_threshold_point_totals.histories - sweep_threshold_point_totals.swept_histories,
    ));
    let positive_control_of_the_pointer_birth_missed =
        sweep_read_swept.histories > 0 && sweep_read_swept.pairs.with_pointer_birth_e1 == 0;
    let premise_controls_missed =
        instance_one_control_misses > 0 || (!sweep_swept.is_empty() && sweep_changes == 0);
    let truth_control_missed = skr_direct == 0 || ss_in_between == 0;
    let independent_anchor_failed = !independent_failures.is_empty();
    let void = positive_control_of_the_pointer_birth_missed
        || premise_controls_missed
        || truth_control_missed
        || independent_anchor_failed;
    lines.push(clause_line(
        "Z2",
        positive_control_of_the_pointer_birth_missed,
        &format!(
            "swept_sweep_read_histories={} with_pointer_birth_e1_pairs={}",
            sweep_read_swept.histories, sweep_read_swept.pairs.with_pointer_birth_e1
        ),
    ));
    lines.push(clause_line("Z3", premise_controls_missed, &format!("instance_one_control_histories_missed={instance_one_control_misses} swept_histories={} sweep_pointer_changes={sweep_changes}", sweep_swept.len())));
    lines.push(clause_line(
        "Z4",
        truth_control_missed,
        &format!(
            "skr_directly_bridged={skr_direct} ss_existed_versions_in_between={ss_in_between}"
        ),
    ));
    lines.push(clause_line(
        "Z5",
        independent_anchor_failed,
        &format!(
            "failed_independent_anchors={}",
            joined_or_none(&independent_failures)
        ),
    ));
    let unreadable: u64 = [
        Group::ShortRing,
        Group::SkippedRow,
        Group::SweepCrash,
        Group::SweepRead,
    ]
    .iter()
    .flat_map(|group| POINT_ORDER.iter().map(move |point| (*group, *point)))
    .map(|(group, point)| totals_of(&results_of(results, group, point), false).unreadable_images)
    .sum();
    lines.push(clause_line(
        "Z6",
        false,
        &format!("unreadable_images_listed_separately_not_counted={unreadable}"),
    ));
    let misclassified: Vec<String> = misclassified_questions
        .iter()
        .map(|question| (*question).to_string())
        .collect();
    lines.push(clause_line(
        "F1",
        !misclassified.is_empty(),
        &format!(
            "questions_with_misclassified_pairs={}",
            joined_or_none(&misclassified)
        ),
    ));
    let premise_broken = premise_totals.first_version_violations
        + premise_totals.adjacent_version_violations
        + premise_totals.birth_after_the_txg_violations
        > 0;
    lines.push(clause_line(
        "F2",
        premise_broken,
        &format!(
            "first_version_violations={} adjacent_version_violations={} birth_after_the_txg_violations={}",
            premise_totals.first_version_violations,
            premise_totals.adjacent_version_violations,
            premise_totals.birth_after_the_txg_violations
        ),
    ));
    lines.push(clause_line(
        "F5",
        !clause_failures.is_empty(),
        &format!(
            "clause_image_anchors_unequal={}",
            joined_or_none(&clause_failures)
        ),
    ));
    let unswept: Vec<String> = swept_counts
        .iter()
        .filter(|(_, histories, swept)| swept < histories)
        .map(|(group, histories, swept)| format!("{}:{swept}/{histories}", group.name()))
        .collect();
    lines.push(clause_line(
        "F6",
        !unswept.is_empty(),
        &format!("groups_with_unswept_histories={}", joined_or_none(&unswept)),
    ));
    let made = [
        Group::ShortRing,
        Group::SkippedRow,
        Group::SweepCrash,
        Group::SweepRead,
    ]
    .iter()
    .all(|group| totals_of(&results_of(results, *group, main), false).judged_images > 0)
        && [
            SamplingPoint::SweepThresholdProductDefault,
            SamplingPoint::LongJournalRing,
            SamplingPoint::SkippedRowWiderRing,
        ]
        .iter()
        .all(|point| {
            results
                .iter()
                .any(|result| result.specification.point == *point && !result.images.is_empty())
        });
    let status = row_status(stops, void, made && clause_failures.is_empty());
    lines.push(format!(
        "name=verdict row=1 questions_with_misclassified_pairs={} premise_violations={} judgement={} status={status}",
        joined_or_none(&misclassified),
        premise_totals.first_version_violations + premise_totals.adjacent_version_violations + premise_totals.birth_after_the_txg_violations,
        if misclassified.is_empty() && !premise_broken { "branch_one" } else { "branch_two" }
    ));
    lines
}

/// 第三段（附带，不判岔路）：sw-none 只报合计与轨迹。
pub(super) fn judgement_lines_of_segment_three(
    results: &[HistoryResult],
    stops: &StopClauses,
) -> Vec<String> {
    let mut lines = Vec::new();
    for arm_error in ARM_ERRORS {
        let reading = question_reading(
            results,
            Group::SweepNone,
            SamplingPoint::Main,
            false,
            arm_error,
        );
        lines.push(question_line(
            "Q19",
            Group::SweepNone,
            SamplingPoint::Main,
            "all_histories_attached_not_judged",
            arm_error,
            reading,
        ));
    }
    lines.push(format!(
        "name=verdict row=attached status={}",
        row_status(stops, false, true)
    ));
    lines
}

#[cfg(test)]
mod tests;
