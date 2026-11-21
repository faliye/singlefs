//! 梯子组（同一 (盘大小点, S, K, 臂) 下扫 N）与阳性对照的单测。

use super::*;

#[test]
fn positive_control_reads_free_0_on_the_disk_it_fills_with_the_converged_file_size() {
    let geometry_for = |data_units: u64| {
        Some(smallest_usable_candidate(
            ROOT_RING_SLOTS_PER_REGION_MAXIMUM,
            SMALL_JOURNAL_RING_BYTES,
            64 + 2 * (data_units / 10),
        ))
    };
    let free_after_the_fill =
        |geometry: Geometry, _data_units: u64| Some(geometry.unit_area_slots() / 4 + 30);
    let PositiveControlFileSize {
        geometry,
        data_units,
        free_after_the_fill_slots,
    } = positive_control_file_size_on_the_filled_disk(&geometry_for, &free_after_the_fill)
        .expect("盘随 N 涨得慢，迭代收敛");
    assert_eq!(
        (geometry.unit_area_slots(), data_units, free_after_the_fill_slots),
        (74, 50, 48),
        "N = 1 的盘（64 槽）读到 46 ⇒ N = 48 的盘（72 槽）读到 48 ⇒ N = 50 的盘（74 槽）读到 48 = N − 2，停；照 N = 1 那块盘的 46 取 N = 48 就是改之前的读法"
    );
    assert_eq!(
        free_after_the_fill(geometry_for(data_units).expect("有盘"), data_units),
        Some(data_units - 2),
        "free_0 读的是写进 N 个单元、压满之后那块盘本身（登记 5.5）"
    );
    let never_settles = |_geometry: Geometry, file_data_units: u64| Some(file_data_units);
    assert_eq!(
        positive_control_file_size_on_the_filled_disk(&geometry_for, &never_settles)
            .expect_err("N 每次多 2，不收敛")
            .len(),
        POSITIVE_CONTROL_FILE_SIZE_ITERATIONS_AT_MOST,
        "不收敛时迭代满上限交回迭代过的每一次"
    );
}

#[test]
fn positive_control_refusal_after_raising_the_floor_holds_for_a_session_refusal_whose_floor_raise_was_refused_in_the_rehearsal_and_not_for_a_landing_refusal(
) {
    let like_the_seventh_rerun_session_cell_with_no_chain_landed =
        PositiveControlCellVerdict::of_the_cell(
            RefusalHandling::ThroughTheSession,
            Some(DeleteRefusal::NoSpaceAfterRaisingTheFloor),
        );
    assert_eq!(
        like_the_seventh_rerun_session_cell_with_no_chain_landed,
        PositiveControlCellVerdict {
            is_refused: true,
            is_refused_with_no_space_after_raising_the_floor: Some(true),
        },
        "r7 冒烟 P1-S16 N-B 那一格：会话推了抬 F、那一串在预演第 2 次被拒、零落盘（floor_raise_publishes=0），拒绝种类 no_space_after_raising_the_floor；读法乙下 U12 成立"
    );
    assert!(like_the_seventh_rerun_session_cell_with_no_chain_landed.holds());
    assert_eq!(
        like_the_seventh_rerun_session_cell_with_no_chain_landed
            .refused_after_raising_the_floor_label(),
        "true"
    );
    for refusal_without_the_session in [DeleteRefusal::Landing, DeleteRefusal::Placement] {
        let session_arm_refused_without_the_session = PositiveControlCellVerdict::of_the_cell(
            RefusalHandling::ThroughTheSession,
            Some(refusal_without_the_session),
        );
        assert_eq!(
            session_arm_refused_without_the_session,
            PositiveControlCellVerdict {
                is_refused: true,
                is_refused_with_no_space_after_raising_the_floor: Some(false),
            },
            "B 臂不经会话（变异 U12）：拒绝种类是 {}，U12 不成立",
            refusal_without_the_session.label()
        );
        assert!(!session_arm_refused_without_the_session.holds());
        assert_eq!(
            session_arm_refused_without_the_session.refused_after_raising_the_floor_label(),
            "false"
        );
    }
    let session_arm_not_refused =
        PositiveControlCellVerdict::of_the_cell(RefusalHandling::ThroughTheSession, None);
    assert_eq!(
        session_arm_not_refused,
        PositiveControlCellVerdict {
            is_refused: false,
            is_refused_with_no_space_after_raising_the_floor: Some(false),
        }
    );
    assert!(!session_arm_not_refused.holds());
    let direct_refused = PositiveControlCellVerdict::of_the_cell(
        RefusalHandling::Direct,
        Some(DeleteRefusal::Landing),
    );
    assert_eq!(
        direct_refused,
        PositiveControlCellVerdict {
            is_refused: true,
            is_refused_with_no_space_after_raising_the_floor: None,
        }
    );
    assert!(direct_refused.holds());
    assert_eq!(
        direct_refused.refused_after_raising_the_floor_label(),
        "not_applicable"
    );
    let direct_not_refused = PositiveControlCellVerdict::of_the_cell(RefusalHandling::Direct, None);
    assert!(!direct_not_refused.holds());
    assert_eq!(
        direct_not_refused.refused_after_raising_the_floor_label(),
        "not_applicable"
    );
}

#[test]
fn positive_control_verdict_counts_the_refusal_and_the_session_floor_raise_refusal_apart_and_stops_on_either(
) {
    let direct_refused = PositiveControlCellVerdict::of_the_cell(
        RefusalHandling::Direct,
        Some(DeleteRefusal::Landing),
    );
    let direct_not_refused = PositiveControlCellVerdict::of_the_cell(RefusalHandling::Direct, None);
    let session_refused_after_raising_the_floor = PositiveControlCellVerdict::of_the_cell(
        RefusalHandling::ThroughTheSession,
        Some(DeleteRefusal::NoSpaceAfterRaisingTheFloor),
    );
    let session_refused_by_the_landing = PositiveControlCellVerdict::of_the_cell(
        RefusalHandling::ThroughTheSession,
        Some(DeleteRefusal::Landing),
    );
    let session_not_refused =
        PositiveControlCellVerdict::of_the_cell(RefusalHandling::ThroughTheSession, None);
    let like_the_seventh_rerun_smoke = PositiveControlTally::of_the_cells(&[
        direct_refused,
        session_refused_after_raising_the_floor,
        direct_refused,
        session_refused_after_raising_the_floor,
    ]);
    assert_eq!(
        like_the_seventh_rerun_smoke,
        PositiveControlTally {
            cells: 4,
            cells_without_refusal: 0,
            session_cells_refused_other_than_no_space_after_raising_the_floor: 0,
            cells_not_holding: 0,
        },
        "r7 冒烟那四格（N-A、N-B、W-A、W-B）：四格都出了拒绝，两条 B 臂的拒绝都是 no_space_after_raising_the_floor（N-B 零落盘），读法乙下都成立"
    );
    assert_eq!(like_the_seventh_rerun_smoke.stop_reason(), None);
    assert_eq!(
        lines_and_stop_line_of(|| stop_when_a_positive_control_does_not_hold(
            like_the_seventh_rerun_smoke
        )),
        (Vec::new(), None),
        "四格都合：不停"
    );
    let session_arm_without_the_session = PositiveControlTally::of_the_cells(&[
        direct_refused,
        session_refused_by_the_landing,
        direct_refused,
        session_refused_after_raising_the_floor,
    ]);
    assert_eq!(
        session_arm_without_the_session,
        PositiveControlTally {
            cells: 4,
            cells_without_refusal: 0,
            session_cells_refused_other_than_no_space_after_raising_the_floor: 1,
            cells_not_holding: 1,
        },
        "一条 B 臂被落点那一判直接拒（不经会话）：四格都出了拒绝，U12 那个数是 1"
    );
    assert_eq!(
        session_arm_without_the_session.verdict_fields(),
        "positive_control_cells=4 positive_control_cells_without_refusal=0 positive_control_session_cells_refused_other_than_no_space_after_raising_the_floor=1 positive_control_cells_not_holding=1",
        "不出拒绝的格数报在 without_refusal 下，U12 不合的格数报在自己的名字下"
    );
    assert_eq!(
        session_arm_without_the_session.stop_reason(),
        Some(
            "u12_session_arm_positive_control_refused_other_than_no_space_after_raising_the_floor"
        )
    );
    let (lines, stop_line) = lines_and_stop_line_of(|| {
        stop_when_a_positive_control_does_not_hold(session_arm_without_the_session)
    });
    assert!(lines.is_empty());
    assert_eq!(
        stop_line.as_deref(),
        Some("E name=stop reason=u12_session_arm_positive_control_refused_other_than_no_space_after_raising_the_floor positive_control_cells=4 positive_control_cells_without_refusal=0 positive_control_session_cells_refused_other_than_no_space_after_raising_the_floor=1 positive_control_cells_not_holding=1"),
        "有一格不合就打停机行（退 4），不当成跑完"
    );
    let none_refused =
        PositiveControlTally::of_the_cells(&[direct_not_refused, session_not_refused]);
    assert_eq!(
        none_refused,
        PositiveControlTally {
            cells: 2,
            cells_without_refusal: 2,
            session_cells_refused_other_than_no_space_after_raising_the_floor: 0,
            cells_not_holding: 2,
        },
        "B 臂没出拒绝只算进 without_refusal（V1），不算进 U12 那个数"
    );
    assert_eq!(
        none_refused.stop_reason(),
        Some("v1_positive_control_cells_without_refusal"),
        "不出拒绝的格先按 V1 停"
    );
    let all_holding = PositiveControlTally::of_the_cells(&[
        direct_refused,
        session_refused_after_raising_the_floor,
        direct_refused,
        session_refused_after_raising_the_floor,
        direct_refused,
        session_refused_after_raising_the_floor,
        direct_refused,
        session_refused_after_raising_the_floor,
    ]);
    assert_eq!(
        all_holding.verdict_fields(),
        "positive_control_cells=8 positive_control_cells_without_refusal=0 positive_control_session_cells_refused_other_than_no_space_after_raising_the_floor=0 positive_control_cells_not_holding=0",
        "八格都合：r8 脚本第六步按这一串判"
    );
    assert_eq!(
        lines_and_stop_line_of(|| stop_when_a_positive_control_does_not_hold(all_holding)),
        (Vec::new(), None),
        "八格都合：不停"
    );
}
