//! 锚点 A2–A7 的值（第七节 7.2）与流锚点的报告。

use super::*;

// ───────────────────────────── 锚点 A2–A7 的值（第七节 7.2，登记第十三节 anchors.py / anchors2.py 的原样输出） ─────────────────────────────

// 第 3 次重跑登记（`research/prompts/e161-r3-prereg.md`）第七节 A2、A3、A4 与第十三节 `anchors_e161_r3.py` 的原样输出（取号写两轮之后的段形）。
pub(super) const ANCHOR_FIRST_STREAM_SEGMENT_STATES: [u64; 12] =
    [8, 8, 3, 1, 8, 3, 1, 8, 16_777_215, 3, 1, 8];
const ANCHOR_FIRST_STREAM_STATES: u64 = 16_777_268;
/// txg1、txg2、txg3、最后一条根之后。
const ANCHOR_FIRST_STREAM_BY_PUBLISH: [u64; 4] = [20, 12, 16_777_227, 8];
/// 覆盖写固定脚本流取样段 12–28 逐段。
pub(super) const ANCHOR_SECOND_STREAM_SEGMENT_STATES: [u64; 17] = [
    16_777_215, 3, 1, 8, 8, 8, 262_143, 3, 1, 8, 65_535, 3, 1, 8, 65_535, 3, 1,
];
const ANCHOR_SECOND_STREAM_STATES: u64 = 17_170_484;
/// txg4（不含不取的段 11）、txg5、txg6、txg7。
const ANCHOR_SECOND_STREAM_BY_PUBLISH: [u64; 4] = [16_777_219, 262_171, 65_547, 65_547];
const ANCHOR_FIRST_STREAM_SMALL: u64 = 53;
const ANCHOR_SECOND_STREAM_SMALL: u64 = 393_269;
/// 每个 24 写段 ⌈(2²⁴ − 1) / 2311⌉；两条流取样段里各只有一个 24 写段。
const ANCHOR_STRIDE_PER_SEGMENT: u64 = 7_260;
const ANCHOR_FIRST_STREAM_QUICK: u64 = 54;
const ANCHOR_SECOND_STREAM_QUICK: u64 = 60;
/// 段头：每条流的大段（24 写段）各取段内序号 0–4095（重跑登记 I-4 新加的一格）。
const ANCHOR_SEGMENT_HEADS_PER_STREAM: u64 = 4_096;
/// A5：取三态的写数（新池新建文件流 10 次系统配置槽写，覆盖写固定脚本流 50 次；第 3 次重跑登记 A5，取号写两轮各多 2 次）。
const ANCHOR_FIRST_STREAM_IN_PLACE_OVERWRITES: usize = 10;
pub(super) const ANCHOR_SECOND_STREAM_IN_PLACE_OVERWRITES: usize = 50;
/// A6：容量的字节数。
const ANCHOR_LARGEST_CARD_BYTES: u128 = 34_190_917_632;
const ANCHOR_SIXTEEN_GIGABYTE_CARD_BYTES: u128 = 17_094_934_528;
const ANCHOR_FIVE_CARDS_BYTES: u128 = 102_595_821_568;

/// 按发布分段：每一段归它自己或它之后最近的那次根槽写所在的发布；最后一条根之后的段单列一格。
fn states_by_publish(stream: &PreparedStream, counts_by_segment: &[u64]) -> Vec<u64> {
    let mut groups = Vec::new();
    let mut running = 0u64;
    for (segment, count) in stream.segments.iter().zip(counts_by_segment) {
        running += count;
        if segment
            .iter()
            .any(|write_index| stream.writes[*write_index].kind == StepKind::RootRecordFua)
        {
            groups.push(running);
            running = 0;
        }
    }
    groups.push(running);
    groups
}

pub(super) fn bool_text(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}

/// 整条流每段按 A1 的公式算的状态数（只用公式，不枚举；重跑登记 I-5）：交回（全量 = 1 + Σ 各段，最大一段）。
pub(super) fn whole_stream_states_by_the_formula(
    stream: &PreparedStream,
    table: &TornWriteTable,
) -> (u128, u128) {
    let per_segment: Vec<u128> = stream
        .segments
        .iter()
        .map(|segment| {
            let in_place_overwrites = segment
                .iter()
                .filter(|write_index| table.is_in_place_overwrite[**write_index])
                .count();
            u128::from(formula_segment_state_count(
                u64::try_from(segment.len()).expect("写数"),
                u64::try_from(in_place_overwrites).expect("个数"),
            ))
        })
        .collect();
    let largest = per_segment
        .iter()
        .copied()
        .max()
        .expect("录制流至少有一段（S2 核过段长表）");
    (1 + per_segment.iter().sum::<u128>(), largest)
}

/// 一条流的锚点核下来的两样。
pub(super) struct StreamAnchorOutcome {
    /// A1、A2 / A3、A4、A5 与（覆盖写固定脚本流）A7 全部对上。
    pub(super) every_anchor_holds: bool,
    /// 可行性档要评的三个域（D1-small、D1-quick、段头）的大小都等于 A4（重跑登记 I-7 的域守卫只看这一样）。
    pub(super) evaluated_domain_sizes_hold: bool,
}

/// 一条流的锚点 A1、A2 / A3、A5 与 A4 的域大小（覆盖写固定脚本流另核 A7 的整条流全量与最大一段）。
pub(super) fn report_stream_anchors(
    output: &mut ResultLines,
    stream: &PreparedStream,
    table: &TornWriteTable,
) -> StreamAnchorOutcome {
    let checks = segment_count_checks(stream, table);
    let mut all_three_agree = true;
    for check in &checks {
        let agree = check.counted_by_the_plan == check.counted_by_the_crates
            && check.counted_by_the_crates == check.counted_by_the_formula;
        all_three_agree &= agree;
        output.line(format!(
            "name=anchor_segment stream={} segment={} in_domain_counted_one_by_one={} writes={} in_place_overwrites={} counted_by_the_plan={} counted_by_the_crates={} counted_by_the_formula={} three_agree={}",
            stream.name.label(),
            check.segment_index,
            bool_text(is_sampled_segment(stream.name, check.segment_index)),
            check.writes,
            check.in_place_overwrites,
            check.counted_by_the_plan,
            check.counted_by_the_crates,
            check.counted_by_the_formula,
            bool_text(agree),
        ));
    }
    let full = LocalPlan::new(
        stream,
        table,
        domain_expansions(stream, LocalExpansion::EveryCombination),
    );
    let quick = LocalPlan::new(
        stream,
        table,
        domain_expansions(
            stream,
            LocalExpansion::InPlaceCombinationsWithUnitWritesNoneOrAll,
        ),
    );
    let sampled_counts: Vec<u64> = full
        .count_by_segment
        .iter()
        .enumerate()
        .filter(|(segment_index, _)| is_sampled_segment(stream.name, *segment_index))
        .map(|(_, count)| *count)
        .collect();
    let domain_states =
        full.expanded_state_count + u64::from(includes_every_write_persisted(stream.name));
    let by_publish = states_by_publish(stream, &full.count_by_segment);
    let in_place = table
        .is_in_place_overwrite
        .iter()
        .filter(|is_in_place| **is_in_place)
        .count();
    let small = u64::try_from(small_domain_ordinals(stream, &full).len()).expect("个数");
    let stride = u64::try_from(stride_domain_ordinals(stream, &full).len()).expect("个数");
    let quick_states = u64::try_from(every_ordinal(stream, &quick).len()).expect("个数");
    let heads = u64::try_from(strided_segment_heads(stream, &full).len()).expect("个数");
    let (segments_ok, states_ok, publish_ok, small_ok, quick_ok, in_place_ok) = match stream.name {
        StreamName::First => (
            sampled_counts == ANCHOR_FIRST_STREAM_SEGMENT_STATES,
            domain_states == ANCHOR_FIRST_STREAM_STATES,
            by_publish == ANCHOR_FIRST_STREAM_BY_PUBLISH,
            small == ANCHOR_FIRST_STREAM_SMALL,
            quick_states == ANCHOR_FIRST_STREAM_QUICK,
            in_place == ANCHOR_FIRST_STREAM_IN_PLACE_OVERWRITES,
        ),
        StreamName::Second => (
            sampled_counts == ANCHOR_SECOND_STREAM_SEGMENT_STATES,
            domain_states == ANCHOR_SECOND_STREAM_STATES,
            by_publish
                .iter()
                .copied()
                .filter(|count| *count > 0)
                .collect::<Vec<u64>>()
                == ANCHOR_SECOND_STREAM_BY_PUBLISH,
            small == ANCHOR_SECOND_STREAM_SMALL,
            quick_states == ANCHOR_SECOND_STREAM_QUICK,
            in_place == ANCHOR_SECOND_STREAM_IN_PLACE_OVERWRITES,
        ),
    };
    let stride_ok = stride == ANCHOR_STRIDE_PER_SEGMENT;
    let heads_ok = heads == ANCHOR_SEGMENT_HEADS_PER_STREAM;
    let mask_width_formula_holds =
        table.mask_width() == stream.writes.len() + in_place + table.replay_count;
    let (whole_stream_fields, whole_stream_ok) = match stream.name {
        StreamName::First => (String::new(), true),
        StreamName::Second => {
            let (whole_stream_states, whole_stream_largest) =
                whole_stream_states_by_the_formula(stream, table);
            let states_match = whole_stream_states == SECOND_STREAM_FULL_STATES;
            let largest_match = whole_stream_largest == SECOND_STREAM_LARGEST_SEGMENT_STATES;
            (
                format!(
                    " whole_stream_states={whole_stream_states} whole_stream_largest_segment_states={whole_stream_largest} whole_stream_states_match={} whole_stream_largest_segment_match={}",
                    bool_text(states_match),
                    bool_text(largest_match),
                ),
                states_match && largest_match,
            )
        }
    };
    output.line(format!(
        "name=anchor_stream stream={} sampled_segment_states={sampled_counts:?} domain_states={domain_states} by_publish={by_publish:?} d1_small={small} d1_stride={stride} d1_quick={quick_states} d1_heads={heads} recorded_writes={} in_place_overwrites={in_place} replays={} w={} segments_match={} domain_states_match={} by_publish_match={} d1_small_match={} d1_stride_match={} d1_quick_match={} d1_heads_match={} in_place_overwrites_match={} w_formula_holds={}{whole_stream_fields}",
        stream.name.label(),
        stream.writes.len(),
        table.replay_count,
        table.mask_width(),
        bool_text(segments_ok),
        bool_text(states_ok),
        bool_text(publish_ok),
        bool_text(small_ok),
        bool_text(stride_ok),
        bool_text(quick_ok),
        bool_text(heads_ok),
        bool_text(in_place_ok),
        bool_text(mask_width_formula_holds),
    ));
    StreamAnchorOutcome {
        every_anchor_holds: all_three_agree
            && segments_ok
            && states_ok
            && publish_ok
            && small_ok
            && stride_ok
            && quick_ok
            && heads_ok
            && in_place_ok
            && mask_width_formula_holds
            && whole_stream_ok,
        evaluated_domain_sizes_hold: small_ok && quick_ok && heads_ok,
    }
}

/// A6、A7：容量与最大一段的算术。
pub(super) fn capacity_anchors_hold() -> bool {
    LARGEST_CARD_MEBIBYTES * MEBIBYTE == ANCHOR_LARGEST_CARD_BYTES
        && SMALLEST_SIXTEEN_GIGABYTE_CARD_MEBIBYTES * MEBIBYTE == ANCHOR_SIXTEEN_GIGABYTE_CARD_BYTES
        && FIVE_CARDS_MEBIBYTES * MEBIBYTE == ANCHOR_FIVE_CARDS_BYTES
        && 2u128.pow(28) == SECOND_STREAM_LARGEST_SEGMENT_STATES + 1
}
