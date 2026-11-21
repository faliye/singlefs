use super::*;

/// 今天的 `replay_journal` 在 r5 档 L 域 N ≤ 2 上 J8 每格 0、施加过的态有判到、「无出处」0（调真实现，同 r2 登记修订 4 挪进单测）。
/// 9.2「J8a 拿所选根当期望（回到旧规则）」点名这一条：那一变异在 N = 2、P_1 落了那一态上 J8a 两格各 1。
#[test]
fn todays_replay_journal_breaks_no_new_root_segment_fields_cell_on_the_big_ring_up_to_two_records()
{
    let mut judged_rebuilt_states = 0;
    for size in 1..=2 {
        let tally = tally_of_a_domain_size(
            Domain::LegalBigRing,
            size,
            Implementation::TodayReplayJournal,
            RunProfile::root_fields_in_records(
                MarkerParity::EvenVersionsWrittenByTheUnmountSequence,
            ),
            1,
        );
        for counter in [
            Counter::J8aInstanceTableViolations,
            Counter::J8aAllocationRecordTreeRootViolations,
            Counter::J8aUnmountMarkerViolations,
            Counter::J8bViolations,
            Counter::J8cViolations,
            Counter::NewRootSegmentFieldsWithoutASource,
        ] {
            assert_eq!(
                tally.count_of(counter),
                0,
                "L 域 N = {size} 上 {} 是 0：{:?}",
                counter.label(),
                tally.witness(counter)
            );
        }
        judged_rebuilt_states += tally.count_of(Counter::J8aJudged);
    }
    assert!(
        judged_rebuilt_states > 0,
        "L 域 N ≤ 2 里有施加过、判到 J8a 的态"
    );
}
