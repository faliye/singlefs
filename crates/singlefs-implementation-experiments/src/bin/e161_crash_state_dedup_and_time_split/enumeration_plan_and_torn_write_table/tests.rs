//! 装置自己的枚举计划与撕裂镜像表（停机条款 S3、锚点 A1）的单测。

use super::*;

#[test]
fn first_stream_plan_counts_match_the_formula_the_crates_and_the_anchor() {
    let stream = prepare_first_stream();
    let table = TornWriteTable::of(&stream);
    let checks = segment_count_checks_up_to(&stream, &table, 1 << 20);
    // 先 A1 两半、后 A2（重跑登记 I-8）：装置计划改坏时头一个断的是「装置对 crates」，crates 那一列因此被比到。
    for check in &checks {
        assert_eq!(
            check.counted_by_the_plan, check.counted_by_the_crates,
            "A1：段 {} 装置对 crates",
            check.segment_index
        );
        assert_eq!(
            check.counted_by_the_crates, check.counted_by_the_formula,
            "A1：段 {} crates 对公式",
            check.segment_index
        );
    }
    let counted: Vec<u64> = checks
        .iter()
        .map(|check| check.counted_by_the_plan)
        .collect();
    assert_eq!(
        counted, ANCHOR_FIRST_STREAM_SEGMENT_STATES,
        "A2：装置计划逐段数"
    );
    assert_eq!(
        table.mask_width(),
        53,
        "A5：43 次写 + 10 份撕裂镜像 + 0 次重放"
    );
    assert_eq!(
        formula_segment_state_count(26, 2),
        150_994_943,
        "3² · 2²⁴ − 1"
    );
    assert_eq!(
        formula_segment_state_count(24, 0),
        16_777_215,
        "2²⁴ − 1：取号写两轮之后新池新建文件流段 8 的 24 个单元写"
    );
}
