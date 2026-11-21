use super::*;

#[test]
fn every_registered_anchor_matches_the_independent_reference() {
    let readings = anchor_readings();
    let registered = readings.iter().filter(|reading| reading.expected.is_some()).count();
    assert_eq!(registered, 26, "第七节登记的锚点格：A1 四臂、A2 三臂、A3 五格、A4 三臂、A5 四格、A6 三臂、C1、crates 单测三格");
    assert_eq!(readings.len(), 26 + 5, "另有 f = 59 那个变体在 A2 … A6 上只报不判的五格");
    for reading in &readings {
        assert!(reading.matches(), "锚点 {} 臂 {}：读到 {}，登记 {:?}", reading.cell, reading.arm, reading.observed, reading.expected);
    }
}

#[test]
fn insert_until_refused_anchor_refuses_the_insert_that_would_open_the_136th_leaf() {
    let observed = anchor_insert_until_refused(ARM_DROP_EMPTY_ONLY);
    assert_eq!(observed, format!("refused_insert={} live={} containers={}", 31_455 + 1, 233 * 135, 135));
}

#[test]
fn segment_two_anchors_judge_insert_until_refused_for_the_three_variants_and_only_report_the_rest() {
    let readings = segment_two_anchor_readings();
    let registered = readings.iter().filter(|reading| reading.expected.is_some()).count();
    assert_eq!(registered, 26 + 3, "第一段登记的 26 格，加三个变体的 A1");
    assert_eq!(readings.len(), 31 + 3 + 3 + 2 + 3 + 2 + 3, "第一段 31 格，加三个变体 A1、A2、A3（两格）、A4、A5（两格）、A6");
    for reading in &readings {
        assert!(reading.matches(), "锚点 {} 臂 {}：读到 {}，登记 {:?}", reading.cell, reading.arm, reading.observed, reading.expected);
    }
}
