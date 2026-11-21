use super::*;

#[test]
fn writing_factor_lines_mark_the_voided_writings_and_the_unread_rounds() {
    let calls = vec![
        (CallKind::Issue, synthetic_cpu_writing_pieces()),
        (CallKind::Batch, synthetic_cpu_writing_pieces()),
    ];
    let point = SubmissionPoint {
        fraction: LiveSlotFraction::OneQuarter,
        pool_data_units: 1024,
        batch_units: 8,
    };
    let mut rounds = vec![Some(calls); ROUNDS];
    rounds[1] = None;
    let record = CpuWritingsSegmentRecord {
        pieces_by_point: BTreeMap::from([(point, rounds)]),
        failed_pieces: BTreeSet::from([TimedPiece::IdentityIndexBuild]),
        stage_lines: 2,
        cpu_writing_stage_lines: 1,
    };
    let lines = writing_factor_lines(&record);
    assert_eq!(lines.len(), CpuWriting::ALL.len() * WritingStage::ALL.len());
    assert!(lines.contains(&"name=writing_factor fraction=0.25 pool_data_units=1024 batch_units=8 writing=hardware_crc_single stage=crc factor_per_round=0.250000,none,0.250000,0.250000,0.250000 factor=0.250000 status=measured".to_string()));
    assert!(lines.iter().any(
        |line| line.contains("writing=both_single_with_build stage=crc ")
            && line.ends_with("status=void_positive_control")
    ));
    assert!(lines.iter().any(|line| line
        .contains("writing=identity_index_without_build stage=scan_increment ")
        && line.ends_with("status=measured")));
}

#[test]
fn cpu_writing_stage_line_carries_the_keys_of_the_stage_line_and_the_nine_pieces() {
    let line = cpu_writing_stage_line(
        "fraction=0.75 pool_data_units=8192 batch_units=16 round=2",
        4,
        CallKind::Batch,
        &synthetic_cpu_writings_of_a_call(),
    );
    assert!(line.starts_with("name=cpu_writing_stage fraction=0.75 pool_data_units=8192 batch_units=16 round=2 call_index=4 kind=batch crc_today_cpu_nanoseconds=4.000000 "));
    assert!(line.contains(" scan_identity_index_lookup_cpu_nanoseconds=0.125000 identity_index_build_cpu_nanoseconds=1.000000 identity_index_entries=1030 "));
    assert!(
        line.ends_with(" hardware_crc_matches=true identity_index_scan_matches=true readable=true")
    );
}

#[test]
fn cpu_writing_stage_line_count_anchor_holds_only_when_the_counts_agree() {
    let mut record = CpuWritingsSegmentRecord {
        pieces_by_point: BTreeMap::new(),
        failed_pieces: BTreeSet::new(),
        stage_lines: 7,
        cpu_writing_stage_lines: 7,
    };
    assert_eq!(
        cpu_writing_stage_line_count_anchor(&record),
        (
            "name=anchor item=cpu_writing_stage_lines value=7 expected=7 holds=true".to_string(),
            true
        )
    );
    record.cpu_writing_stage_lines = 6;
    assert!(!cpu_writing_stage_line_count_anchor(&record).1);
}
