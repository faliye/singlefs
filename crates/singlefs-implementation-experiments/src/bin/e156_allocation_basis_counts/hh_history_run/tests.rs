//! 岔路 1 的族 Hh：一趟历史的状态、逐观测点的行与整趟的驱动的单测。

use super::{
    anchor_cell_for, anchor_cells_of, hh_request, judge_hole_history_outcome, run_hh_history,
    AnchorCellsOfOneSlotsPerRegion, Arm, Emitter, HhCellShape, HhRunOutcome, HolePosition,
    ReclaimTiming, WorkloadDensity,
};
use super::{hole_family_anchor_cells_of, E156_FAMILY_SAME_RING_SLOT};
use std::collections::{BTreeMap, BTreeSet};

fn run_the_hole_history_of_one_shape(
    anchors: &BTreeMap<u64, AnchorCellsOfOneSlotsPerRegion>,
    hole_count: u64,
    position: Option<HolePosition>,
) -> HhRunOutcome {
    let shape = HhCellShape {
        slots_per_region: 4,
        density: WorkloadDensity::EveryPublishOverwrites,
        hole_count,
        position,
    };
    let mut emitter = Emitter { emitted: 0 };
    run_hh_history(
        &mut emitter,
        &hh_request(
            "Hh",
            shape,
            anchor_cell_for(anchors, 4, hole_count, position),
        ),
    )
}

fn anchors_for_four_slots_per_region() -> BTreeMap<u64, AnchorCellsOfOneSlotsPerRegion> {
    BTreeMap::from([(4, anchor_cells_of(4))])
}

fn observation_at(outcome: &HhRunOutcome, txg: u64) -> &super::HhObservation {
    outcome
        .observations
        .iter()
        .find(|observation| observation.txg == txg)
        .unwrap_or_else(|| panic!("txg {txg} 有一个观测点"))
}

/// U14（第 4 次重跑登记第九节）：Hh(0, —, S = 4, ρ = 1) 跑到 txg 48：实与每两个时点 A10 的滞后全是 12、A12 每个观测点
/// Q1a = Q1b = 0、每个观测点分配器装着根环表。M35（第 5 次重跑登记第九节改写：装着根环表而记根时不回收）、M42（「每」的门槛晚一次）在这里红。
#[test]
fn hh_without_holes_at_four_slots_per_region_reclaims_each_tree_table_one_ring_length_after_release(
) {
    let outcome = run_the_hole_history_of_one_shape(&anchors_for_four_slots_per_region(), 0, None);
    assert_eq!(outcome.actual_end, 48, "Hh(0, —, S = 4) 跑到 txg 48");
    assert!(outcome.truncation.is_none(), "没有截断");
    assert_eq!(
        outcome.tree_table_lags_real,
        BTreeSet::from([12]),
        "A10（实）：滞后全是 3S = 12"
    );
    assert_eq!(
        outcome.tree_table_lags_every_publish,
        BTreeSet::from([12]),
        "A10（每）：滞后全是 3S = 12"
    );
    for observation in &outcome.observations {
        for timing in [ReclaimTiming::Real, ReclaimTiming::EveryPublish] {
            assert_eq!(
                observation.over_withheld(Arm::FloorRule, timing),
                0,
                "A12：Q1a = 0，txg {}",
                observation.txg
            );
            assert_eq!(
                observation.over_withheld(Arm::LifetimeIntervalRule, timing),
                0,
                "A12：Q1b = 0，txg {}",
                observation.txg
            );
        }
        assert!(
            observation.reading.root_ring_installed,
            "W7：每个观测点装着根环表，txg {}",
            observation.txg
        );
    }
}

/// U15（第 4 次重跑登记第九节）：PQ1-后（S = 4）：环里没有 txg 16 的根、槽里是 txg 4 的根；txg 19–27 门槛 = 4、txg 28 门槛 = 17；
/// txg 19 Q1a ≥ 4、txg 27 Q1a ≥ 12；每个观测点 Q1b = 0；txg 30 h_缺 = 0。M36、M37、M38、M41 在这里红。
#[test]
fn hh_back_hole_at_four_slots_per_region_pins_the_threshold_to_txg_four_until_txg_twenty_eight() {
    let outcome = run_the_hole_history_of_one_shape(
        &anchors_for_four_slots_per_region(),
        1,
        Some(HolePosition::Back),
    );
    let recovery = outcome.recoveries.first().expect("造了一个洞");
    assert_eq!(recovery.hole_txg, 16, "洞在 txg 16");
    assert!(
        !recovery.ring_has_a_root_at_the_hole_txg,
        "环里没有 txg 16 的根"
    );
    assert_eq!(
        recovery.hole_slot_content, "root_txg_4",
        "txg 16 的槽里仍是 txg 4 的根"
    );
    for txg in 19..=27u64 {
        assert_eq!(
            observation_at(&outcome, txg).reading.ring_threshold,
            4,
            "A14：txg {txg} 门槛 = 4"
        );
    }
    assert_eq!(
        observation_at(&outcome, 28).reading.ring_threshold,
        17,
        "A14：txg 28 门槛 = 17"
    );
    for timing in [ReclaimTiming::Real, ReclaimTiming::EveryPublish] {
        assert!(
            observation_at(&outcome, 19).over_withheld(Arm::FloorRule, timing) >= 4,
            "A16：txg 19 Q1a ≥ 4"
        );
        assert!(
            observation_at(&outcome, 27).over_withheld(Arm::FloorRule, timing) >= 12,
            "A15：txg 27 Q1a ≥ 12"
        );
    }
    for observation in &outcome.observations {
        for timing in [ReclaimTiming::Real, ReclaimTiming::EveryPublish] {
            assert_eq!(
                observation.over_withheld(Arm::LifetimeIntervalRule, timing),
                0,
                "A13：Q1b = 0，txg {}",
                observation.txg
            );
        }
    }
    assert_eq!(
        observation_at(&outcome, 30).reading.missing_txgs,
        0,
        "txg 30 环里只剩 19–30，h_缺 = 0"
    );
}

/// U16（第 4 次重跑登记第九节）：Hh(2, 后, S = 4, ρ = 1)：每个观测点 h_缺、h_洞 等于锚点模型那一格（txg 28–33 的 h_洞 = 1）。M39 在这里红。
#[test]
fn hh_two_back_holes_at_four_slots_per_region_match_the_anchor_hole_counts() {
    let outcome = run_the_hole_history_of_one_shape(
        &anchors_for_four_slots_per_region(),
        2,
        Some(HolePosition::Back),
    );
    assert!(!outcome.observations.is_empty(), "至少一个观测点");
    for observation in &outcome.observations {
        let row = observation.anchor.expect("每个观测点都有锚点那一行");
        assert_eq!(
            observation.reading.missing_txgs, row.missing_txgs,
            "A11：h_缺，txg {}",
            observation.txg
        );
        assert_eq!(
            observation.reading.live_holes, row.live_holes,
            "A11：h_洞，txg {}",
            observation.txg
        );
    }
    for txg in 28..=33u64 {
        assert_eq!(
            observation_at(&outcome, txg).reading.live_holes,
            1,
            "txg {txg} 的 h_洞 = 1"
        );
    }
}

/// U17（第 4 次重跑登记第九节）：PQ1-前（S = 4）：txg 7 那一刻 h_缺 = h_洞 = 1、Q1a ≥ 1、Q1b = 0。M40 在这里红。
#[test]
fn hh_front_hole_at_four_slots_per_region_over_withholds_one_slot_under_the_floor_rule_only() {
    let outcome = run_the_hole_history_of_one_shape(
        &anchors_for_four_slots_per_region(),
        1,
        Some(HolePosition::Front),
    );
    let recovery = outcome.recoveries.first().expect("造了一个洞");
    assert_eq!(recovery.hole_txg, 5, "洞在 txg 5");
    assert_eq!(
        recovery.hole_slot_content, "never_written",
        "txg 5 的槽从没写过"
    );
    let observation = observation_at(&outcome, 7);
    assert_eq!(observation.reading.missing_txgs, 1, "txg 7 h_缺 = 1");
    assert_eq!(observation.reading.live_holes, 1, "txg 7 h_洞 = 1");
    for timing in [ReclaimTiming::Real, ReclaimTiming::EveryPublish] {
        assert!(
            observation.over_withheld(Arm::FloorRule, timing) >= 1,
            "txg 7 Q1a ≥ 1"
        );
        assert_eq!(
            observation.over_withheld(Arm::LifetimeIntervalRule, timing),
            0,
            "txg 7 Q1b = 0"
        );
    }
}

/// U20（第 6 次重跑登记第九节；钉的值出自第七节 K16、L6）：Hh(1, 前, S = 4, ρ = 1)，与 U17 同一段历史。定乙：洞在 txg 5，
/// 那一槽挂载时读回全零、钉在 5；txg 16 环里最旧有效根是 6、门槛照钉住取 5；txg 17 这个进程写掉那一槽，门槛回到 6。
/// M45（门槛不并进全零槽）、M46（锚点模型不并进）、M47（写掉也不解开）在这里红。
#[test]
fn hh_front_hole_at_four_slots_per_region_pins_the_threshold_to_the_hole_txg_until_the_process_rewrites_its_slot(
) {
    let anchors = anchors_for_four_slots_per_region();
    let outcome = run_the_hole_history_of_one_shape(&anchors, 1, Some(HolePosition::Front));
    let recovery = outcome.recoveries.first().expect("造了一个洞");
    assert_eq!(recovery.hole_txg, 5, "洞在 txg 5");
    assert_eq!(
        recovery.hole_slot_content, "never_written",
        "txg 5 的槽从没写过"
    );
    assert_eq!(outcome.all_zero_pin_mounts.len(), 1, "一次挂载");
    let txg_five_slot = super::RootRingSlot { region: 2, slot: 1 };
    assert_eq!(
        outcome.all_zero_pin_mounts[0].pinned,
        BTreeMap::from([(txg_five_slot, 5)]),
        "W16、W17：这次挂载恰好钉 txg 5 那一槽（区域 2、槽 1），钉在 5"
    );
    for (txg, expected_threshold) in [(15u64, 4u64), (16, 5), (17, 6)] {
        assert_eq!(
            observation_at(&outcome, txg).reading.ring_threshold,
            expected_threshold,
            "K16：txg {txg} 门槛 = {expected_threshold}"
        );
    }
    let anchor = anchor_cell_for(&anchors, 4, 1, Some(HolePosition::Front));
    assert_eq!(
        anchor.observed_row(16).expect("锚点有 txg 16").floor,
        5,
        "L6：锚点模型 txg 16 门槛 = 5"
    );
    assert_eq!(
        anchor.observed_row(17).expect("锚点有 txg 17").floor,
        6,
        "锚点模型 txg 17 门槛 = 6"
    );
    for observation in &outcome.observations {
        assert_eq!(
            observation.reading.device_zero.reclaim_rule_violations, 0,
            "K10：txg {}",
            observation.txg
        );
        assert_eq!(
            observation.over_withheld(Arm::FloorRule, ReclaimTiming::Real),
            observation.over_withheld(Arm::FloorRule, ReclaimTiming::EveryPublish),
            "F24：实、每两列 Q1a 相等，txg {}",
            observation.txg
        );
    }
    assert_eq!(
        outcome.all_zero_pins_released_by_write, 1,
        "txg 17 写掉那一槽"
    );
    assert_eq!(
        outcome.all_zero_pins_released_by_reread_only, 0,
        "没有重读解开"
    );
    assert_eq!(
        outcome.all_zero_pin_mounts[0].bad_slots_at_mount, 0,
        "挂载那一刻没有读坏的槽"
    );
}

/// U19（第 5 次重跑登记第九节）：Hh(1, 后, S = 4, ρ = 1)，与 U15 同一段历史。① 实、每两份滞后集合都含 6S − 1 = 23
/// （K15、L1：洞让 txg 4 的旧根压着门槛到 txg 28，释放代 6 的树表到那一次发布才可发，滞后 = 28 + 1 − 6）；
/// ② 交 k = 0 的 A10 判定、期望 {12} 时，实、每两项都判「不符」；③ 期望换成这一格自己量出来的两份集合时，两项各自判「符」
/// （判定不是恒红）。M44（「实」列不读真实分配器的空闲位图、按 3S 的时间表记）在 ① 红。
#[test]
fn hh_back_hole_at_four_slots_per_region_delays_the_tree_table_reclaim_to_six_ring_slots_minus_one_and_the_reclaim_lag_judgement_sees_it(
) {
    let outcome = run_the_hole_history_of_one_shape(
        &anchors_for_four_slots_per_region(),
        1,
        Some(HolePosition::Back),
    );
    assert!(
        outcome.tree_table_lags_real.contains(&23),
        "K15：实列滞后集合含 6S − 1 = 23，量到 {:?}",
        outcome.tree_table_lags_real
    );
    assert!(
        outcome.tree_table_lags_every_publish.contains(&23),
        "K15：每列滞后集合含 6S − 1 = 23，量到 {:?}",
        outcome.tree_table_lags_every_publish
    );
    let three_slots_per_ring_length = BTreeSet::from([12]);
    let against_the_no_hole_expectation =
        judge_hole_history_outcome(&outcome, Some(&three_slots_per_ring_length));
    assert_eq!(
        against_the_no_hole_expectation.tree_table_lag_real_mismatch, 1,
        "期望 {{12}}：实列判不符"
    );
    assert_eq!(
        against_the_no_hole_expectation.tree_table_lag_every_publish_mismatch, 1,
        "期望 {{12}}：每列判不符"
    );
    let real_own = outcome.tree_table_lags_real.clone();
    let every_own = outcome.tree_table_lags_every_publish.clone();
    assert_eq!(
        judge_hole_history_outcome(&outcome, Some(&real_own)).tree_table_lag_real_mismatch,
        0,
        "期望 = 实列自己量出来的集合：实列判符"
    );
    assert_eq!(
        judge_hole_history_outcome(&outcome, Some(&every_own))
            .tree_table_lag_every_publish_mismatch,
        0,
        "期望 = 每列自己量出来的集合：每列判符"
    );
}

/// 第 7 次重跑登记 P25 与第七节 K21、A22–A24：Hs(2, 后, S = 4, ρ = 1)，洞在 txg 16、28；锚点脚本 `--dump` 的值：txg 31 与 39 门槛 4（那一槽里 txg 4 的根没被写掉），
/// txg 39 h_缺 = 24，txg 40 这个进程写掉那一槽、门槛 29；每个观测点门槛、h_缺、h_洞、被钉住的槽数与锚点模型相同，Q1a 不低于锚点的树表下界。
#[test]
fn hs_two_back_holes_at_four_slots_per_region_keep_the_threshold_on_txg_four_until_the_slot_is_rewritten(
) {
    let table = hole_family_anchor_cells_of(4);
    let cell = table
        .cells
        .iter()
        .find(|cell| {
            cell.family == E156_FAMILY_SAME_RING_SLOT
                && cell.position == HolePosition::Back
                && cell.hole_count == 2
        })
        .expect("族表里有 Hs(2, 后)");
    let shape = HhCellShape {
        slots_per_region: 4,
        density: WorkloadDensity::EveryPublishOverwrites,
        hole_count: 2,
        position: Some(HolePosition::Back),
    };
    let mut emitter = Emitter { emitted: 0 };
    let outcome = run_hh_history(
        &mut emitter,
        &hh_request(E156_FAMILY_SAME_RING_SLOT, shape, &cell.anchor),
    );
    assert!(outcome.truncation.is_none(), "没有截断");
    let hole_txgs: Vec<u64> = outcome
        .recoveries
        .iter()
        .map(|recovery| recovery.hole_txg)
        .collect();
    assert_eq!(hole_txgs, vec![16, 28], "两个洞钉在同一个环槽");
    for (txg, threshold) in [(31u64, 4u64), (39, 4), (40, 29)] {
        assert_eq!(
            observation_at(&outcome, txg).reading.ring_threshold,
            threshold,
            "锚点脚本 --dump：txg {txg} 门槛 {threshold}"
        );
    }
    assert_eq!(
        observation_at(&outcome, 39).reading.missing_txgs,
        24,
        "txg 39 h_缺"
    );
    for observation in &outcome.observations {
        let row = observation.anchor.expect("每个观测点都有锚点那一行");
        assert_eq!(
            observation.reading.ring_threshold, row.floor,
            "A23：门槛，txg {}",
            observation.txg
        );
        assert_eq!(
            observation.reading.missing_txgs, row.missing_txgs,
            "A24：h_缺，txg {}",
            observation.txg
        );
        assert_eq!(
            observation.reading.live_holes, row.live_holes,
            "A24：h_洞，txg {}",
            observation.txg
        );
        assert_eq!(
            observation.reading.all_zero_slots_still_pinned, row.pinned_slots,
            "A24：被钉住的槽数，txg {}",
            observation.txg
        );
        for timing in [ReclaimTiming::Real, ReclaimTiming::EveryPublish] {
            assert!(
                observation.over_withheld(Arm::FloorRule, timing) >= row.tree_table_lower_bound,
                "A22：Q1a 不低于树表下界，txg {}",
                observation.txg
            );
        }
    }
}
