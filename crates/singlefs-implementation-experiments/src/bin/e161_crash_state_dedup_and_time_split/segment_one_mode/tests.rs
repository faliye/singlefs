//! 模式 segment-one：一条流上的 D1 与 D1-quick的单测。

use super::*;

#[test]
fn refeeding_the_every_write_persisted_state_keeps_distinct_unit_contents() {
    let (stream, totals, _small) =
        evaluate_the_first_stream_small_domain(2, TimingControl::WALL_CLOCK_WITHOUT_BUSY_WAIT);
    let table = TornWriteTable::of(&stream);
    let context = EvaluationContext::new(&stream, &table);
    let plan = LocalPlan::new(
        &stream,
        &table,
        domain_expansions(&stream, LocalExpansion::EveryCombination),
    );
    let (distinct_unchanged, checks_grew_exactly, positions) =
        refeed_every_write_persisted(&context, &plan, &totals.tables, totals.whole_unit_positions);
    assert!(distinct_unchanged, "PC1：去重臂的不同内容数不变");
    assert!(
        checks_grew_exactly,
        "PC1：逐状态臂恰好加上它的不同单元位置数"
    );
    assert!(positions > 0, "全部持久那个状态读到了单元");
}
