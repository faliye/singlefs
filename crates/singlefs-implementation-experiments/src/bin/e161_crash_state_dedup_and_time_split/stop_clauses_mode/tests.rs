//! 模式 stop-clauses：S1–S6 与 A1–A7（跑产物之前）的单测。

use super::*;

#[test]
fn device_tally_equals_the_crates_tally_on_the_first_stream_small_domain() {
    let stream = prepare_first_stream();
    let table = TornWriteTable::of(&stream);
    let context = EvaluationContext::new(&stream, &table);
    let plan = LocalPlan::new(
        &stream,
        &table,
        domain_expansions(&stream, LocalExpansion::EveryCombination),
    );
    let small = small_domain_ordinals(&stream, &plan);
    let small_segments: BTreeSet<usize> = small
        .iter()
        .map(|ordinal| plan.segment_of(*ordinal))
        .collect();
    let mut output = ResultLines::new();
    let clauses = report_crates_comparison(
        &mut output,
        "test-first-small",
        &context,
        &plan,
        &small,
        &|segment_index| {
            if small_segments.contains(&segment_index) {
                CrashPointReplaySegmentExpansion::EveryProperSubset
            } else {
                CrashPointReplaySegmentExpansion::NotExpanded
            }
        },
        2,
    );
    let CratesComparisonClauses::Judged {
        enumeration_clause_holds,
        tally_clause_holds,
    } = clauses
    else {
        // 判段这一趟没跑（配置开了 GPU 而没编进 gpu 特性）：上面已打本次未跑那一行，不判
        return;
    };
    assert!(enumeration_clause_holds, "S3：持久集合与写表逐个相同");
    assert!(tally_clause_holds, "S4：装置照跑的计数与 crates 逐项相同");
}
