//! 取样域（登记 5.5）的单测。

use super::*;

#[test]
fn stride_and_quick_domains_have_the_registered_sizes() {
    let stream = prepare_first_stream();
    let table = TornWriteTable::of(&stream);
    let full = LocalPlan::new(
        &stream,
        &table,
        domain_expansions(&stream, LocalExpansion::EveryCombination),
    );
    let quick = LocalPlan::new(
        &stream,
        &table,
        domain_expansions(
            &stream,
            LocalExpansion::InPlaceCombinationsWithUnitWritesNoneOrAll,
        ),
    );
    assert_eq!(
        small_domain_ordinals(&stream, &full).len(),
        53,
        "A4：D1-small 新池新建文件流"
    );
    assert_eq!(
        stride_domain_ordinals(&stream, &full).len(),
        7_260,
        "A4：D1-stride 新池新建文件流唯一的 24 写段（段 8）"
    );
    assert_eq!(
        every_ordinal(&stream, &quick).len(),
        54,
        "A4：D1-quick 新池新建文件流"
    );
    assert_eq!(
        strided_segment_heads(&stream, &full).len(),
        4_096,
        "A4：段头，新池新建文件流段 8 序号最小的 4096 个"
    );
}

/// 重跑登记 I-10：只搭覆盖写固定脚本流、建计划，一个状态都不评。
#[test]
fn second_stream_shape_and_sampled_domains_have_the_registered_sizes() {
    let first = prepare_first_stream();
    let stream = prepare_second_stream();
    assert_eq!(stream.segments.len(), 80, "A8：覆盖写固定脚本流 80 段");
    assert_eq!(stream.writes.len(), 485, "A8：覆盖写固定脚本流 485 次写");
    assert_eq!(stream.root_write_count, 19, "A8：覆盖写固定脚本流 19 条根");
    let shape = check_stream_shape(&stream);
    assert!(shape.segment_lengths_match, "S2：段长表与本地常量逐段相同");
    assert!(shape.writes_match, "S2：写数与本地常量相同");
    assert!(shape.roots_match, "S2：根数与本地常量相同");
    let (bases_identical, prefix_identical) = shared_prefix_is_identical(&first, &stream);
    assert!(bases_identical, "S2：两条流的基镜像逐字节相同");
    assert!(
        prefix_identical,
        "S2：覆盖写固定脚本流前 12 段的写与新池新建文件流 12 段逐字节相同"
    );
    let table = TornWriteTable::of(&stream);
    let full = LocalPlan::new(
        &stream,
        &table,
        domain_expansions(&stream, LocalExpansion::EveryCombination),
    );
    let quick = LocalPlan::new(
        &stream,
        &table,
        domain_expansions(
            &stream,
            LocalExpansion::InPlaceCombinationsWithUnitWritesNoneOrAll,
        ),
    );
    let sampled_counts: Vec<u64> = full
        .count_by_segment
        .iter()
        .enumerate()
        .filter(|(segment_index, _count)| is_sampled_segment(stream.name, *segment_index))
        .map(|(_segment_index, count)| *count)
        .collect();
    assert_eq!(
        sampled_counts, ANCHOR_SECOND_STREAM_SEGMENT_STATES,
        "A3：取样段 12–28 逐段状态数"
    );
    assert_eq!(
        small_domain_ordinals(&stream, &full).len(),
        393_269,
        "A4：D1-small 覆盖写固定脚本流"
    );
    assert_eq!(
        stride_domain_ordinals(&stream, &full).len(),
        7_260,
        "A4：D1-stride 覆盖写固定脚本流唯一的 24 写段（段 12）"
    );
    assert_eq!(
        every_ordinal(&stream, &quick).len(),
        60,
        "A4：D1-quick 覆盖写固定脚本流"
    );
    assert_eq!(
        strided_segment_heads(&stream, &full).len(),
        4_096,
        "A4：段头，覆盖写固定脚本流段 12 序号最小的 4096 个"
    );
    let timing_control = TimingControlSegmentShape::of(&stream, &full);
    assert_eq!(
        timing_control.writes, 18,
        "A9：PC3 那一段是写行发布的 18 个单元写"
    );
    assert_eq!(
        timing_control.states, 262_143,
        "A9：PC3 那一段 2¹⁸ − 1 个状态"
    );
    assert!(
        timing_control.holds(),
        "I-6：PC3 取的 1000 个序号都落在那一段"
    );
    let (whole_stream_states, whole_stream_largest) =
        whole_stream_states_by_the_formula(&stream, &table);
    assert_eq!(
        whole_stream_states, 1_678_377_220,
        "A7：覆盖写固定脚本流整条流全量（公式，1 + Σ 各段）"
    );
    assert_eq!(
        whole_stream_largest, 268_435_455,
        "A7：覆盖写固定脚本流整条流最大一段 2²⁸ − 1"
    );
}
