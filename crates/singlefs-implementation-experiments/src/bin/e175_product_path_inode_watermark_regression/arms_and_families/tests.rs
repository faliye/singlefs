//! `arms_and_families` 的单测。
use super::*;

// ───────────── F1 四组与第八节几何四点的规模、比重与每族一段的取样 ─────────────

/// F1 四组与几何四点的段数与步数（登记第五节 F1 那一行、第八节那张表）：
/// `python3 -c "print(4*128*30, 4*32*30, 64*150+64*60+64*60+32*150, 32*150)"` 输出 `15360 3840 22080 4800`。
#[test]
fn existing_random_and_geometry_scales_match_the_registered_counts() {
    let product = Arm::Product;
    let rollback_variant = Arm::RollbackTakesTheTargetsWatermark;
    let publish_variant = Arm::PublishDoesNotAddTheNewInodes;
    let steps_on = |families: &[Family], arm: Arm| -> u64 {
        families
            .iter()
            .map(|family| {
                let per_history = family.operations_per_random_history().expect("随机族");
                family.registered_history_count(arm) * u64::try_from(per_history).expect("几步")
            })
            .sum()
    };
    let existing: Vec<Family> = ExistingWeightGroup::ALL
        .into_iter()
        .map(Family::ExistingRandom)
        .collect();
    let geometry: Vec<Family> = GeometryPoint::ALL
        .into_iter()
        .map(Family::Geometry)
        .collect();
    for family in &existing {
        assert_eq!(
            family.registered_history_count(product),
            128,
            "{}",
            family.label()
        );
        assert_eq!(
            family.registered_history_count(rollback_variant),
            32,
            "{}",
            family.label()
        );
        assert_eq!(
            family.registered_history_count(publish_variant),
            32,
            "{}",
            family.label()
        );
    }
    let counts = |point: GeometryPoint| {
        let family = Family::Geometry(point);
        (
            family.registered_history_count(product),
            family.registered_history_count(rollback_variant),
            family.registered_history_count(publish_variant),
        )
    };
    assert_eq!(
        counts(GeometryPoint::LongerRandomWithRollbacks),
        (64, 32, 0),
        "G-long"
    );
    assert_eq!(
        counts(GeometryPoint::RandomWithRollbacksOnSmallDevices),
        (64, 0, 0),
        "G-small"
    );
    assert_eq!(
        counts(GeometryPoint::RandomWithRollbacksDrawingRecentUniformOrBeyond),
        (64, 0, 0),
        "G-draw"
    );
    assert_eq!(counts(GeometryPoint::LongerBroad), (32, 0, 0), "G-long-F1");
    assert_eq!(steps_on(&existing, product), 15_360);
    assert_eq!(steps_on(&existing, rollback_variant), 3_840);
    assert_eq!(steps_on(&existing, publish_variant), 3_840);
    assert_eq!(steps_on(&geometry, product), 22_080);
    assert_eq!(steps_on(&geometry, rollback_variant), 4_800);
    for family in existing.iter().chain(geometry.iter()) {
        let item = history_item(*family, 0);
        assert_eq!(
            Some(item.history.operations.len()),
            family.operations_per_random_history(),
            "{} 序号 0 生成的步数",
            family.label()
        );
    }
}

/// F1 四组就是 `crates/` 的四组命名比重，都在 unlink 之后抽 `CreateInode`；几何点只改登记第八节点名的那一个旋钮；族名各不相同、反查得回。
#[test]
fn existing_random_and_geometry_families_change_only_the_registered_knob() {
    let named = [
        NamedGenerationWeights::Broad,
        NamedGenerationWeights::ReuseAfterRaisingTheFloor,
        NamedGenerationWeights::DeletingFilesOfSeveralLowerLeaves,
        NamedGenerationWeights::NewestSystemConfigurationSlotReadingZeros,
    ];
    for (group, named_weights_of_the_group) in ExistingWeightGroup::ALL.into_iter().zip(named) {
        assert_eq!(
            group.weights(),
            named_weights_of_the_group.weights(),
            "{}",
            group.label()
        );
        let after_deletion = group
            .weights()
            .with_session_open_after_the_file_was_deleted
            .expect("F1 四组都有「删完之后」那张表");
        assert!(
            after_deletion
                .iter()
                .any(|(kind, weight)| *kind == HistoryOperationKind::CreateInode && *weight > 0),
            "{} 要抽 CreateInode",
            group.label()
        );
    }
    let drawing = RANDOM_WITH_ROLLBACKS_WEIGHTS_DRAWING_RECENT_UNIFORM_OR_BEYOND;
    assert_eq!(
        drawing.rollback_targets,
        RollbackTargetDraw::RecentUniformOrBeyond
    );
    assert_ne!(
        RANDOM_WITH_ROLLBACKS_WEIGHTS.rollback_targets,
        drawing.rollback_targets
    );
    assert_eq!(
        drawing.starting_points,
        RANDOM_WITH_ROLLBACKS_WEIGHTS.starting_points
    );
    assert_eq!(
        drawing.with_session_closed,
        RANDOM_WITH_ROLLBACKS_WEIGHTS.with_session_closed
    );
    assert_eq!(
        drawing.with_session_open_without_file,
        RANDOM_WITH_ROLLBACKS_WEIGHTS.with_session_open_without_file
    );
    assert_eq!(
        drawing.with_session_open_with_file,
        RANDOM_WITH_ROLLBACKS_WEIGHTS.with_session_open_with_file
    );
    assert_eq!(
        drawing.with_session_open_after_the_file_was_deleted,
        RANDOM_WITH_ROLLBACKS_WEIGHTS.with_session_open_after_the_file_was_deleted
    );
    assert_eq!(
        drawing.overwrite_contents,
        RANDOM_WITH_ROLLBACKS_WEIGHTS.overwrite_contents
    );
    for family in Family::every_family() {
        let expected_width =
            if family == Family::Geometry(GeometryPoint::RandomWithRollbacksOnSmallDevices) {
                HistoryDeviceWidth::UnitAreaOf384Slots
            } else {
                HistoryDeviceWidth::FourGibibytes
            };
        assert_eq!(
            family.execution().device_width,
            expected_width,
            "{}",
            family.label()
        );
        assert_eq!(family.execution().per_step_checker, PerStepChecker::Skipped);
        assert_eq!(
            family.execution().space_admission,
            SpaceAdmission::JudgedByTheFormula
        );
        assert_eq!(Family::from_label(family.label()), Some(family));
    }
    let labels: BTreeSet<&str> = Family::every_family()
        .into_iter()
        .map(Family::label)
        .collect();
    assert_eq!(labels.len(), 11);
    assert_eq!(Family::from_label("F1"), None);
}
