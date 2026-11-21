use super::*;

/// 造池停下时打的行：被拒的先打 `name=construction_refused`（登记第十二节修订第 14 条点名的字段逐个钉值，读得回成键值），再打 `name=stopped`；
/// 不是推满也放不下那一形的两样写 `none`；形状对不上的只打一行 `name=stopped`，带对不上的是哪一样与两个数。
#[test]
fn a_stopped_construction_prints_the_refusal_line_and_the_stop_line() {
    let moment = MomentOfTheConstruction {
        phase: ConstructionPhase::InterleavedHalf,
        step: 7,
        publishes: 1234,
        cluster_segments: ClusterSegmentsOfTheMount {
            opened_by_this_mount: 1614,
            unit_area_segments: 2288,
        },
    };
    let refused = ConstructionStopped::PublishRefused {
        moment,
        refusal: Box::new(RefusalReadWhileConstructing {
            name_of_the_refusal: "NoSpaceEvenWithEverythingAFloorRaiseCouldRelease",
            judged_short_by: vec![
                "SlotPairsOutsideTheClusterSegmentsForDataUnits".to_string(),
                "FormulaAvailableAgainstDemand".to_string(),
            ],
            most_a_floor_raise_could_make_available: vec![-5, 7],
        }),
    };
    let lines = construction_stop_lines(
        UnitAreaTier::TwoThousandTwoHundredEightyEightMebibytes,
        16384,
        &refused,
    );
    assert_eq!(
        lines
            .iter()
            .map(|(name, _fields)| *name)
            .collect::<Vec<_>>(),
        ["construction_refused", "stopped"]
    );
    assert_eq!(
        lines[0].1,
        "tier=2288MiB live_data_units=16384 phase=interleaved_half step=7 publishes=1234 cluster_segments=1614 \
             unit_area_segments=2288 refusal=NoSpaceEvenWithEverythingAFloorRaiseCouldRelease \
             judged_short_by=SlotPairsOutsideTheClusterSegmentsForDataUnits,FormulaAvailableAgainstDemand \
             most_a_floor_raise_could_make_available=-5,7"
    );
    let (kind, fields) = fields_of_a_result_line(&format!(
        "{RESULT_LINE_PREFIX} name={} {}",
        lines[0].0, lines[0].1
    ))
    .expect("拒绝行读得回成键值");
    assert_eq!(kind, "construction_refused");
    for key in [
        "tier",
        "phase",
        "step",
        "publishes",
        "cluster_segments",
        "unit_area_segments",
        "judged_short_by",
        "most_a_floor_raise_could_make_available",
    ] {
        assert!(fields.contains_key(key), "拒绝行没有 {key}：{fields:?}");
    }
    assert!(
        lines[1]
            .1
            .starts_with("clause=construction_refused tier=2288MiB "),
        "{}",
        lines[1].1
    );
    let refused_without_devices_short = ConstructionStopped::PublishRefused {
        moment,
        refusal: Box::new(RefusalReadWhileConstructing {
            name_of_the_refusal: "Publish",
            judged_short_by: Vec::new(),
            most_a_floor_raise_could_make_available: Vec::new(),
        }),
    };
    assert!(construction_stop_lines(
        UnitAreaTier::FourGibibytes,
        8192,
        &refused_without_devices_short
    )[0]
    .1
    .ends_with(
        " refusal=Publish judged_short_by=none most_a_floor_raise_could_make_available=none"
    ));
    let shape = ConstructionStopped::PublishShapeDiffersFromTheRegistration {
        moment,
        disagreement: PublishShapeDisagreement::PublishesOfThePhase {
            publishes: 32,
            registered_publishes: 2,
        },
    };
    let shape_lines = construction_stop_lines(UnitAreaTier::EightGibibytes, 8192, &shape);
    assert_eq!(shape_lines.len(), 1, "{shape_lines:?}");
    assert_eq!(shape_lines[0].0, "stopped");
    assert!(
        shape_lines[0].1.starts_with(
            "clause=construction_publish_shape_differs_from_the_registration tier=8GiB live_data_units=8192 \
                 phase=interleaved_half step=7 "
        ) && shape_lines[0]
            .1
            .contains(" disagreement=publishes_of_the_phase phase_publishes=32 registered_publishes=2 "),
        "{}",
        shape_lines[0].1
    );
}
