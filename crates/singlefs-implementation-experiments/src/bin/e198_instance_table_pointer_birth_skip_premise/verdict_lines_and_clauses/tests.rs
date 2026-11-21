use super::*;
use crate::helpers_shared_by_the_tests::{
    assert_the_history_is_consistent, key, ring_version, with_pointer_birth_e1,
};

#[test]
fn newest_row_bad_with_the_row_publish_slot_read_bad_pins_the_registered_hole_pair_table() {
    let result = run_the_history(&newest_row_bad_pin_specification());
    assert_the_history_is_consistent(&result, 1);
    assert_eq!(
        pinned_image_text(Some(&result), 0),
        NEWEST_ROW_BAD_PIN_EXPECTED,
        "登记第七节 C1"
    );
}

#[test]
fn skipped_row_first_image_pins_the_registered_hole_pair_table() {
    let result = run_the_history(&skipped_row_pin_specification());
    assert_the_history_is_consistent(&result, 15);
    assert_eq!(
        pinned_image_text(Some(&result), 0),
        SKIPPED_ROW_PIN_EXPECTED,
        "登记第七节 C2"
    );
}

fn synthetic_image(
    step: u64,
    counts: ArmCounts,
    truths: Vec<Truth>,
    checker_verdicts: Option<(&str, &str)>,
) -> ImageResult {
    let hole_pairs = truths
        .iter()
        .map(|_| HolePair {
            previous_txg: 4,
            previous_group: vec![key(1, 4)],
            version: ring_version(2, 6, Some(6)),
            bridged_today: false,
            bridged_with_the_pointer_birth: false,
        })
        .collect();
    ImageResult {
        label: format!("publishes_after_the_mount={step}"),
        step,
        reading: Ok(ImageReadingOfTheJudgements {
            newest: key(2, 6),
            rows: Vec::new(),
            effective_floor: 0,
            candidates: Vec::new(),
            applied_only_by_records: Vec::new(),
            hole_pairs,
        }),
        truths,
        counts,
        pointer_unreadable_pairs: 0,
        ring_slot_mismatches: Vec::new(),
        row_mismatch: None,
        records_applied: RecordsAppliedComparison::default(),
        checker_verdicts: checker_verdicts
            .map(|(release, reference)| (release.to_string(), reference.to_string())),
        corrupted_skipped: Vec::new(),
    }
}

fn unreadable_image(step: u64) -> ImageResult {
    let mut image = synthetic_image(step, ArmCounts::default(), Vec::new(), None);
    image.reading = Err("最新根的实例表读不出".to_string());
    image
}

fn synthetic_history(
    group: Group,
    name: &str,
    swept: bool,
    images: Vec<ImageResult>,
) -> HistoryResult {
    HistoryResult {
        specification: HistorySpecification {
            group,
            point: SamplingPoint::Main,
            slots_per_region: 4,
            journal_ring_bytes: SHORT_JOURNAL_RING_BYTES,
            shape: HistoryShape::SkippedRow { last: 6 },
        },
        name: name.to_string(),
        images,
        stopped: None,
        sweep: Some(SweepReading {
            sweep_instance: 3,
            outcome: "synthetic".to_string(),
            sweep_version: swept.then_some(key(3, 20)),
            last_txg_before_the_sweep_instance: 16,
            predicted_sweep_txg: 20,
            instances_mounted_before_the_sweep: 1,
        }),
        premise: PremiseCounts::default(),
        ledger: Ledger::default(),
    }
}

#[test]
fn group_totals_and_question_readings_count_only_what_was_judged() {
    let swept = synthetic_history(
        Group::SweepCrash,
        "swept",
        true,
        vec![
            synthetic_image(
                0,
                with_pointer_birth_e1(1),
                vec![Truth::ExistedVersionsInBetween],
                None,
            ),
            synthetic_image(1, ArmCounts::default(), vec![Truth::DirectlyBridged], None),
        ],
    );
    let unswept = synthetic_history(
        Group::SweepCrash,
        "unswept",
        false,
        vec![
            synthetic_image(
                0,
                with_pointer_birth_e1(2),
                vec![Truth::ExistedVersionsInBetween; 2],
                None,
            ),
            unreadable_image(1),
        ],
    );
    let results = vec![swept, unswept];
    let every: Vec<&HistoryResult> = results.iter().collect();
    let totals = totals_of(&every, false);
    assert_eq!(
        (
            totals.histories,
            totals.produced_images,
            totals.judged_images,
            totals.unreadable_images,
            totals.hole_pairs
        ),
        (2, 4, 3, 1, 4),
        "读不出的那一张单列、不进对数"
    );
    assert_eq!(
        (
            totals.pairs.with_pointer_birth_e1,
            totals.images_with_a_positive_count.with_pointer_birth_e1
        ),
        (3, 2)
    );
    assert_eq!(
        truth_count(&totals.truth_counts, Truth::ExistedVersionsInBetween),
        3
    );
    let swept_only = totals_of(&every, true);
    assert_eq!(
        (
            swept_only.histories,
            swept_only.swept_histories,
            swept_only.sweep_txg_matches_the_prediction
        ),
        (1, 1, 1)
    );
    let reading = question_reading(
        &results,
        Group::SweepCrash,
        SamplingPoint::Main,
        true,
        ArmError::WithPointerBirthE1,
    );
    assert_eq!(
        reading,
        QuestionReading {
            pairs: 1,
            images: 1,
            hole_pairs: 2,
            judged_images: 2,
            unreadable_images: 0
        }
    );
    assert_eq!(misclassification_judgement(reading), "branch_two");
    assert_eq!(
        misclassification_judgement(QuestionReading::default()),
        "not_run"
    );
    assert_eq!(
        misclassification_judgement(QuestionReading {
            pairs: 0,
            images: 0,
            hole_pairs: 0,
            judged_images: 1,
            unreadable_images: 0,
        }),
        "no_object"
    );
    assert_eq!(
        misclassification_judgement(QuestionReading {
            pairs: 0,
            images: 0,
            hole_pairs: 2,
            judged_images: 1,
            unreadable_images: 0,
        }),
        "classified_right"
    );
    assert_eq!(
        trajectory_of(&results[0], ArmError::WithPointerBirthE1),
        (1, 1, 0),
        "峰值 1、为正 1 步、期末 0"
    );
    assert!(sensitivity_line(SamplingPoint::LongJournalRing, "Q4", 0, 0)
        .contains("record=two_points discriminating=both_zero"));
    assert!(sensitivity_line(SamplingPoint::LongJournalRing, "Q4", 0, 2)
        .contains("record=unstable discriminating=proven"));
    assert!(sensitivity_line(SamplingPoint::LongJournalRing, "Q4", 3, 3)
        .contains("record=two_points discriminating=equal_values"));
    let checked = synthetic_history(
        Group::NewestRowBad,
        "checked",
        false,
        vec![
            synthetic_image(
                0,
                ArmCounts::default(),
                Vec::new(),
                Some(("violated", "holds")),
            ),
            synthetic_image(
                1,
                ArmCounts::default(),
                Vec::new(),
                Some(("holds", "holds")),
            ),
        ],
    );
    assert_eq!(
        checker_violations(
            &[checked],
            Group::NewestRowBad,
            SamplingPoint::Main,
            CheckedInvariant::ReleaseGenerationInterval
        ),
        (1, "holds:1,violated:1".to_string())
    );
    let stopped = StopClauses {
        lines: Vec::new(),
        triggered: vec!["S5"],
    };
    let clear = StopClauses {
        lines: Vec::new(),
        triggered: Vec::new(),
    };
    assert_eq!(row_status(&stopped, false, true), "stopped_by_S5");
    assert_eq!(row_status(&clear, true, true), "void");
    assert_eq!(row_status(&clear, false, true), "enough_to_judge");
}

#[test]
fn segment_two_clause_lines_follow_the_readings() {
    let results = vec![
        synthetic_history(
            Group::ShortRing,
            "ss",
            false,
            vec![synthetic_image(
                0,
                with_pointer_birth_e1(1),
                vec![Truth::ExistedVersionsInBetween],
                None,
            )],
        ),
        synthetic_history(
            Group::SweepCrash,
            "unswept",
            false,
            vec![synthetic_image(0, ArmCounts::default(), Vec::new(), None)],
        ),
    ];
    let clear = StopClauses {
        lines: Vec::new(),
        triggered: Vec::new(),
    };
    let lines = judgement_lines_of_segment_two(&[], &results, &clear);
    let has = |text: &str| lines.iter().any(|line| line.contains(text));
    assert!(has("name=verdict question=Q4 group=ss point=main scope=all_histories arm=with_pointer_birth error=E1 value_pairs=1"), "{lines:#?}");
    assert!(has("name=clause clause=F1 triggered=yes"), "Q4 有错分的对");
    assert!(
        has("name=clause clause=F6 triggered=yes"),
        "sw-crash 有一段没扫"
    );
    assert!(
        has("name=clause clause=Z4 triggered=yes"),
        "skr 一对直接接上的都没有"
    );
    assert!(has("name=clause clause=Z5 triggered=yes"), "C2 那一张不在");
    assert!(
        has("name=verdict row=1") && has("status=void"),
        "{lines:#?}"
    );
}

#[test]
fn segment_one_clause_lines_follow_the_readings() {
    let counts = ArmCounts {
        today_e1: 1,
        today_e2: 0,
        with_pointer_birth_e1: 0,
        with_pointer_birth_e2: 0,
    };
    let results = vec![synthetic_history(
        Group::NewestRowBad,
        "nrb",
        false,
        vec![synthetic_image(
            0,
            counts,
            vec![Truth::ExistedVersionsInBetween],
            Some(("violated", "holds")),
        )],
    )];
    let clear = StopClauses {
        lines: Vec::new(),
        triggered: Vec::new(),
    };
    let lines = judgement_lines_of_segment_one(&[], &results, &clear);
    let has = |text: &str| lines.iter().any(|line| line.contains(text));
    assert!(has("name=verdict question=Q13 group=nrb point=main scope=all_histories arm=today error=E1 value_pairs=1 value_images=1"), "{lines:#?}");
    assert!(
        has("name=verdict question=Q17 group=nrb point=main value_images=1"),
        "{lines:#?}"
    );
    assert!(
        has("name=clause clause=F4 triggered=yes"),
        "I-3.9 判违例一张"
    );
    assert!(has("name=clause clause=F3 triggered=no"), "甲没有认错");
    assert!(
        has("name=clause clause=Z1 triggered=yes"),
        "C1 那一张不在，PC-H1A 不算中"
    );
    assert!(has("name=verdict row=2 half=release_generation_interval value_images=1 judgement=branch_two status=void"), "{lines:#?}");
    // 登记修订 5：读数里的键名是全词，不是题号缩写。
    assert!(
        has("clause=F3 triggered=no observation=nrb_with_pointer_birth_pairs=0_rs1_with_pointer_birth_pairs=0"),
        "{lines:#?}"
    );
    assert!(
        has("clause=F4 triggered=yes observation=rs1_release_generation_violated_images=")
            && has("_nrb_release_generation_violated_images=1"),
        "{lines:#?}"
    );
    assert!(
        has("clause=F7 triggered=yes observation=rs1_today_e1_images="),
        "{lines:#?}"
    );
    assert!(
        has("observation=instance_one_control_histories_missed="),
        "{lines:#?}"
    );
    assert!(
        has("name=verdict question=Q19 group=nrb point=main reference_interval_verdicts="),
        "{lines:#?}"
    );
}
