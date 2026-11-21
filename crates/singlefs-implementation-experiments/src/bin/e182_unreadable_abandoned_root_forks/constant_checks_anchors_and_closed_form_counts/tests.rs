use super::*;
use crate::helpers_shared_by_the_tests::{small_history, specification_of_the_shape};
use std::sync::OnceLock;

#[test]
fn constants_and_anchors_all_pass() {
    assert_eq!(
        run_constants_and_anchors(),
        0,
        "常量回比、单元区槽数钉值、枚举状态数等于闭式计数、已确认档缺较新系统配置时写之前被拒，都要成立"
    );
}

#[test]
fn enumerated_state_counts_are_pinned_to_independently_computed_numbers() {
    for ((in_place, unit), pinned) in [
        ((0usize, 0usize), 0u64),
        ((3, 0), 7),
        ((16, 0), 65_535),
        ((0, 16), 65_535),
        ((0, 17), 1),
        ((6, 11), 127),
        ((17, 0), 131_071),
    ] {
        let (seen, closed_form) = enumerated_and_closed_form_state_counts(in_place, unit);
        assert_eq!(
            seen, pinned,
            "在原地写 {in_place}、单元写 {unit} 的段里枚举出的状态数"
        );
        assert_eq!(closed_form, pinned, "同一段的闭式");
    }
}

#[test]
fn the_unit_area_slot_counts_are_the_pinned_ones() {
    assert_eq!(unit_area_slots_of(DiskGeometry::Default), 211_968);
    assert_eq!(unit_area_slots_of(DiskGeometry::Narrow), 146_465);
}

#[test]
fn the_small_history_follows_the_registered_anchors() {
    let history = small_history();
    assert!(
        history.consistency_problems.is_empty(),
        "{:?}",
        history.consistency_problems
    );
    assert_eq!(history.mount.refused, None, "未确认档的可写挂载做得成");
    assert_eq!(history.mount.stopped_after_the_mount, None);
    assert_eq!(history.mount.instance, Some(2), "实例 2");
    assert!(history.hidden_root.is_some());
    for reading in [
        anchor_root_slot_writes_land_by_the_ring_rule(history),
        anchor_warm_up_publishes_are_at_most_three(history),
        anchor_first_publish_txg_follows_the_newest_readable(history),
    ] {
        assert!(reading.applicable && reading.holds, "{reading:?}");
    }
    assert!(
        history
            .mount
            .ring_slots_recorded_as_holding_an_abandoned_root
            .is_empty(),
        "被藏根读回全零，核心层的表里没有它的根槽（停机条款第二条，原因是读回全零）"
    );
}

fn small_units_kept_history() -> &'static RecordedHistory {
    static HISTORY: OnceLock<RecordedHistory> = OnceLock::new();
    HISTORY.get_or_init(|| {
        record_the_history(&specification_of_the_shape(
            OverwriteShape::KeepingTheFirstFileUnits,
        ))
        .expect("补臂的小历史建得起来")
    })
}

/// 补臂的小历史跟登记的锚点：挂载做成、可写、被藏根在、每段的状态数展得开（甲二里原地写不超过 40 条）、检测对照每次都判得出。
#[test]
fn the_small_units_kept_history_follows_the_registered_anchors_and_every_segment_can_be_enumerated()
{
    let history = small_units_kept_history();
    assert!(
        history.consistency_problems.is_empty(),
        "{:?}",
        history.consistency_problems
    );
    assert_eq!(history.mount.refused, None, "未确认档的可写挂载做得成");
    assert_eq!(
        history.mount.stopped_after_the_mount, None,
        "挂载之后的 3 次覆盖写做得完"
    );
    assert_eq!(history.mount.instance, Some(2), "实例 2");
    assert!(history.hidden_root.is_some());
    for reading in [
        anchor_root_slot_writes_land_by_the_ring_rule(history),
        anchor_warm_up_publishes_are_at_most_three(history),
        anchor_first_publish_txg_follows_the_newest_readable(history),
    ] {
        assert!(reading.applicable && reading.holds, "{reading:?}");
    }
    for (segment_number, segment) in history.segments.iter().enumerate() {
        assert!(
            state_count_of_segment(&history.writes, segment).is_some(),
            "第 {segment_number} 段（{} 次写）的状态数展得开",
            segment.len()
        );
    }
    let hidden = history.hidden_root.expect("带藏根的历史");
    assert!(
        history
            .roots
            .get(&hidden)
            .expect("被藏根在全知日志里")
            .units
            .len()
            >= 64,
        "被藏根至少引用 64 个落点"
    );
    let (trials, detected, _without_a_unit) = detection_control(history);
    assert!(trials > 0, "至少试了一次");
    assert_eq!(detected, trials, "每次改了单元的字节「被盖」都判得出");
}
