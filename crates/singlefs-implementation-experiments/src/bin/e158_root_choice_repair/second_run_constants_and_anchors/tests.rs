use super::*;

#[test]
fn unique_offset_of_finds_a_single_occurrence_and_refuses_two() {
    assert_eq!(unique_offset_of(&[0, 1, 2, 3, 4], &[2, 3]), Some(2));
    assert_eq!(unique_offset_of(&[2, 3, 0, 2, 3], &[2, 3]), None);
    assert_eq!(unique_offset_of(&[0, 1], &[7]), None);
}

/// 跑前登记 5.6 表「journal tail 偏移、F 偏移」：装置自己解码找到 469 与 481（实例代号在 477）。
#[test]
fn decoded_offsets_put_the_tail_at_469_the_instance_at_477_and_the_floor_at_481() {
    let (tail, instance, floor) =
        decoded_tail_instance_and_floor_offsets().expect("三项都恰好一处");
    assert_eq!(tail, 469);
    assert_eq!(instance, 469 + 8);
    assert_eq!(floor, 481);
}

/// 7.2 的锚点在今天的字段表（489）与乙 / 丁的字段表（497）上都过；字段表写错一字节（488）就有不过的。
#[test]
fn second_run_arithmetic_anchors_hold_for_489_and_for_497_and_not_for_488() {
    for layout in [489u64, 497] {
        for (item, computed, registered) in second_run_arithmetic_anchors(layout) {
            assert_eq!(computed, registered, "{layout}：{item}");
        }
    }
    assert!(second_run_arithmetic_anchors(488)
        .iter()
        .any(|(_, computed, registered)| computed != registered));
    assert_eq!(SYSTEM_CONFIGURATION_BYTES_TODAY_LOCAL, 489);
}
