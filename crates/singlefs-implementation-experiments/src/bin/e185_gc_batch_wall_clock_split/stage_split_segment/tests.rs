use super::*;

#[test]
fn crc_stage_covers_every_byte_of_every_unit() {
    let units = vec![vec![7_u8; 32_768], vec![9_u8; 32_768]];
    let (covered_bytes, folded) = crc_stage_over_the_units(&units);
    assert_eq!(covered_bytes, 65_536, "甲段字节等于扫描读到的字节");
    assert_eq!(
        folded,
        crc32_castagnoli(&units[0]) ^ crc32_castagnoli(&units[1]),
        "整单元算 CRC"
    );
}

#[test]
fn saturation_workers_run_on_distinct_threads() {
    let stage = || {
        std::hint::black_box(crc32_castagnoli(&[5_u8; 4096]));
    };
    let reading = saturated_throughput(4, Duration::from_millis(30), 4096, &stage);
    assert_eq!(reading.distinct_threads, 4, "起了 4 条不同的线程");
    assert!(reading.repetitions >= 4 && reading.bytes_per_second > 0.0);
}

#[test]
fn crc_byte_anchors_hold_only_when_both_items_match_their_reads() {
    assert_eq!(
        crc_byte_anchor_finding(1, 262_144, 262_144, 524_288, 524_288),
        None
    );
    let release_check_short = crc_byte_anchor_finding(1, 262_144, 262_144, 262_144, 524_288);
    assert!(
        release_check_short
            .as_deref()
            .is_some_and(|finding| finding.starts_with("clause_4_release_check_crc_bytes_262144")),
        "{release_check_short:?}"
    );
    let release_check_long = crc_byte_anchor_finding(1, 262_144, 262_144, 786_432, 524_288);
    assert!(release_check_long.is_some(), "多算了也不对");
    let scan_short = crc_byte_anchor_finding(1, 229_376, 262_144, 524_288, 524_288);
    assert!(
        scan_short
            .as_deref()
            .is_some_and(|finding| finding.starts_with("clause_4_crc_bytes_229376")),
        "{scan_short:?}"
    );
}

fn stage_share_row_of(
    kind: CallKind,
    choose_share_per_batch: Option<f64>,
    choose_share_of_the_issue: Option<f64>,
) -> StageShareRow {
    StageShareRow {
        kind,
        crc_cpu_nanoseconds: None,
        scan_cpu_nanoseconds: None,
        scan_increment_cpu_nanoseconds: None,
        choose_cpu_nanoseconds: None,
        crc_share: None,
        scan_increment_share: None,
        choose_share_per_batch,
        choose_share_of_the_issue,
    }
}

/// N13：丙段的判定从签发那一行收（签发行 0.15、批行读不到 ⇒ 值得），不从批行收。
#[test]
fn choose_judgement_reads_the_issue_row_not_the_batch_rows() {
    let rows = [
        stage_share_row_of(CallKind::Issue, Some(0.15), None),
        stage_share_row_of(CallKind::Batch, None, None),
    ];
    let (per_round, judgement) =
        choose_judgement_of_the_rounds(&[choose_share_of_the_issue_row(&rows)]);
    assert_eq!(per_round, vec!["worth"]);
    assert_eq!(judgement, "worth");
}

/// N13：取的是摊到每批的份额（0.05 ⇒ 不值），不是签发那一次的原值（0.15）。
#[test]
fn choose_judgement_reads_the_share_per_batch_not_the_share_of_the_issue() {
    let rows = [stage_share_row_of(CallKind::Issue, Some(0.05), Some(0.15))];
    let (_per_round, judgement) =
        choose_judgement_of_the_rounds(&[choose_share_of_the_issue_row(&rows)]);
    assert_eq!(judgement, "not_worth");
}

/// N13：读不到的那一轮写 `not_run`（不当 0），五轮不全相同写 `unstable`。
#[test]
fn choose_judgement_of_the_rounds_writes_not_run_for_an_unreadable_round() {
    let (per_round, judgement) = choose_judgement_of_the_rounds(&[Some(0.15), None, Some(0.02)]);
    assert_eq!(per_round, vec!["worth", "not_run", "not_worth"]);
    assert_eq!(judgement, "unstable");
    let (_per_round, judgement_of_five_equal_rounds) =
        choose_judgement_of_the_rounds(&[Some(0.2); 5]);
    assert_eq!(judgement_of_five_equal_rounds, "worth");
}

/// N14（第二段）：两轮的份额行，甲、乙段按批序号五轮收成一个判定（签发那一行不算批），丙段从签发那一行收。
#[test]
fn stage_cell_judgements_collect_the_batch_rows_by_index_and_the_choice_from_the_issue_row() {
    let row_with =
        |kind: CallKind, crc_share: Option<f64>, scan_increment_share: Option<f64>| StageShareRow {
            kind,
            crc_cpu_nanoseconds: None,
            scan_cpu_nanoseconds: None,
            scan_increment_cpu_nanoseconds: None,
            choose_cpu_nanoseconds: None,
            crc_share,
            scan_increment_share,
            choose_share_per_batch: None,
            choose_share_of_the_issue: None,
        };
    let round_of = |final_batch_crc_share: f64| {
        vec![
            stage_share_row_of(CallKind::Issue, Some(0.15), Some(0.40)),
            row_with(CallKind::Batch, Some(0.2), Some(0.05)),
            row_with(
                CallKind::FinalBatch,
                Some(final_batch_crc_share),
                Some(0.01),
            ),
        ]
    };
    assert_eq!(
        stage_cell_judgements_text(&[round_of(0.3), round_of(0.05)]),
        "rounds=2 crc_share_judgement_per_batch=worth,unstable scan_increment_share_judgement_per_batch=not_worth,not_worth choose_share_judgement=worth"
    );
    assert_eq!(
        stage_cell_judgements_text(&[]),
        "rounds=0 crc_share_judgement_per_batch= scan_increment_share_judgement_per_batch= choose_share_judgement=not_run"
    );
}
