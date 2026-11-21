use super::*;

/// f = 0.25（m = 8）、K = 8 照锚点走的一条意图：签发一次、最后一批搬 8 个单元。
fn intent_following_the_anchors() -> Vec<CallReading> {
    vec![
        call_reading(CallKind::Issue, 262_144, Some(262_144), 0, 0),
        call_reading(CallKind::FinalBatch, 262_144, Some(262_144), 524_288, 1),
    ]
}

#[test]
fn stop_findings_stay_empty_on_an_intent_following_the_anchors() {
    assert_eq!(
        stop_findings_of(&intent_of(intent_following_the_anchors()), Some(8), 8),
        Vec::<String>::new()
    );
}

#[test]
fn stop_findings_flag_scan_reads_that_differ_from_the_records() {
    let mut calls = intent_following_the_anchors();
    calls[1].scan_read_bytes_in_the_region = 229_376;
    let findings = stop_findings_of(&intent_of(calls), Some(8), 8);
    assert!(
        findings
            .iter()
            .any(|finding| finding.starts_with("clause_2")),
        "{findings:?}"
    );
}

#[test]
fn stop_findings_flag_batches_that_differ_from_the_session_or_the_ceiling() {
    let mut calls = intent_following_the_anchors();
    calls[1].batches_observed = 2;
    let findings = stop_findings_of(&intent_of(calls), Some(8), 8);
    assert!(
        findings
            .iter()
            .any(|finding| finding.starts_with("clause_3_batches_counted")),
        "{findings:?}"
    );
    let findings_at_the_ceiling_of_two_batches =
        stop_findings_of(&intent_of(intent_following_the_anchors()), Some(8), 4);
    assert!(
        findings_at_the_ceiling_of_two_batches
            .iter()
            .any(|finding| finding.contains("ceiling_of_m_over_k")),
        "K = 4 时该有两批：{findings_at_the_ceiling_of_two_batches:?}"
    );
}

#[test]
fn stop_findings_flag_a_first_batch_that_writes_one_device_only() {
    let mut calls = intent_following_the_anchors();
    calls[1].relocation_copy_bytes = 262_144;
    calls[1].is_every_destination_written_whole = false;
    let findings = stop_findings_of(&intent_of(calls), Some(8), 8);
    assert!(
        findings
            .iter()
            .any(|finding| finding.starts_with("clause_5")),
        "{findings:?}"
    );
}

#[test]
fn stop_findings_flag_a_reversed_inclusion() {
    let mut calls = intent_following_the_anchors();
    calls[0].inclusion = inclusion_verdict(1_000_000, 800_000, 500_000);
    let findings = stop_findings_of(&intent_of(calls), Some(8), 8);
    assert!(
        findings
            .iter()
            .any(|finding| finding.starts_with("clause_6")),
        "{findings:?}"
    );
}
