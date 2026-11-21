use super::*;

/// T-表与 T-行两条对照的读数（登记 r2 5.5、第七节）：T-表 `tables_differ` = `direct_table_holds_a_record` = 2 989、`rows_differ` > 0；
/// T-行 `rows_differ` = 状态数 = 3 110、`states` = 3 110、`tables_differ` = 0。
#[test]
fn the_table_comparison_controls_read_2989_on_the_tables_and_3110_on_the_rows() {
    let run_profile =
        RunProfile::varying_fields(MarkerParity::EvenVersionsWrittenByTheUnmountSequence)
            .with_table_source(RecordTableSource::Both);
    for (criterion, expected_values) in [
        (
            Criterion::TableComparisonSeesADifferentTable,
            [Some(2_989), Some(2_989), None],
        ),
        (
            Criterion::TableComparisonSeesADifferentRow,
            [Some(3_110), Some(3_110), Some(0)],
        ),
    ] {
        let readings = control_readings_of(criterion, run_profile, 1);
        assert_eq!(readings.len(), expected_values.len(), "{readings:?}");
        for (reading, expected) in readings.iter().zip(expected_values) {
            assert!(reading.is_red_as_required, "对照没红：{reading:?}");
            match expected {
                Some(value) => assert_eq!(reading.value, value, "{reading:?}"),
                None => assert!(reading.value > 0, "{reading:?}"),
            }
        }
    }
}

/// 判据表每一行都有一条对照且都红（登记 5.4、作废条款 V2；r2 档加的 J6Qn、J7 五行同样，登记 r2 5.5；r5 档加的 J8 五行，登记 r5 5.4；
/// T_table、T_row，登记 r2 5.5）。逐个核正式跑的单元：`scripts/experiment-apparatus/replay.sh` 起这个装置的那几条驱动，加上第 5 次重跑登记
/// 比对对象表里没有复跑行的四条（r2 直接建表，两族、两种标记奇偶），每条一族、一档、
/// 一种建表法、一种盘次序、一种标记奇偶；每个单元跑出对照的判据恰好是那条驱动产物里 `name=control` 的那一份（写死成字面，
/// 不从 `is_judged_under`、`Criterion::family` 现算，那是被核的东西），且读数全红；全部单元并起来罩全表。不把几档、几族并起来判：
/// 并起来时一个单元少跑的对照会被别的单元的读数补上（判决 `research/prompts/m3-formal-code-b37-r1-main-verification.md` B1-甲、
/// `research/prompts/m3-formal-code-b37-r2-main-verification.md` C3-甲）。对抗一族那几行要扫到 n = 3，线程数照装置的取法（[`thread_count_from_the_environment`]）。
#[test]
fn every_criterion_has_a_positive_control_and_every_control_is_red() {
    let thread_count = thread_count_from_the_environment();
    let even = MarkerParity::EvenVersionsWrittenByTheUnmountSequence;
    let odd = MarkerParity::OddVersionsWrittenByTheUnmountSequence;
    let labels: std::collections::BTreeSet<&str> = Criterion::ALL
        .iter()
        .map(|criterion| criterion.label())
        .collect();
    assert_eq!(labels.len(), Criterion::ALL.len(), "判据的标签互不相同");
    let formal_run_units: [(&str, Family, RunProfile, &[Criterion]); 19] = [
        // `scripts/experiment-apparatus/replay.sh` 的 `driver_e165`、`driver_e165_light`；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r1-legal-family.out`，11 条
        (
            "r1 合法",
            Family::LegalHistory,
            RunProfile::CONSTANT_FIELDS,
            &[
                Criterion::LegalDefined,
                Criterion::ReturnedRoot,
                Criterion::RootPlusDisk,
                Criterion::WithinTheInFlightLimit,
                Criterion::FarthestVersion,
                Criterion::J5aAppliedSet,
                Criterion::J5bLandedVersion,
                Criterion::J5aAndJ5bAreSeparable,
                Criterion::FixedPoint,
                Criterion::Ia1LargestTxgFails,
                Criterion::Ia2NoVerificationCounted,
            ],
        ),
        // `scripts/experiment-apparatus/replay.sh` 的 `driver_e165_adversarial`；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r1-adversarial-family.out`，7 条
        (
            "r1 对抗",
            Family::Adversarial,
            RunProfile::CONSTANT_FIELDS,
            &[
                Criterion::AdversarialDefined,
                Criterion::AsWholeTable,
                Criterion::AsGapContinues,
                Criterion::AsGapStops,
                Criterion::AMutual,
                Criterion::AStructure,
                Criterion::ALimit,
            ],
        ),
        // `scripts/experiment-apparatus/replay.sh` 的 `driver_e165_r2_segment3_multi_disk`；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-multi-disk.out`，4 条
        (
            "r2 多盘",
            Family::MultiDisk,
            RunProfile::varying_fields(even),
            &[
                Criterion::MultiDiskAnyCopyIsOnDisk,
                Criterion::MultiDiskReturnedRoot,
                Criterion::MultiDiskFarthestVersion,
                Criterion::MultiDiskAppliedSetAndLandedVersion,
            ],
        ),
        // `research/prompts/e165-r5-prereg.md` 比对对象表第 4 行（`replay.sh` 里没有复跑行）；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-legal-family.out`，17 条
        (
            "r2 合法",
            Family::LegalHistory,
            RunProfile::varying_fields(even),
            &[
                Criterion::LegalDefined,
                Criterion::ReturnedRoot,
                Criterion::RootPlusDisk,
                Criterion::WithinTheInFlightLimit,
                Criterion::FarthestVersion,
                Criterion::J5aAppliedSet,
                Criterion::J5bLandedVersion,
                Criterion::J5aAndJ5bAreSeparable,
                Criterion::FixedPoint,
                Criterion::Ia1LargestTxgFails,
                Criterion::Ia2NoVerificationCounted,
                Criterion::QnFixedPoint,
                Criterion::J7aInstanceTable,
                Criterion::J7aAllocationRecordTreeRoot,
                Criterion::J7aFilesystemIdentifier,
                Criterion::J7bUnmountMarkerOfARebuiltRoot,
                Criterion::J7cUnmountMarkerWhenNothingIsApplied,
            ],
        ),
        // `research/prompts/e165-r5-prereg.md` 比对对象表第 5 行（`replay.sh` 里没有复跑行）；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-adversarial-family.out`，7 条
        (
            "r2 对抗",
            Family::Adversarial,
            RunProfile::varying_fields(even),
            &[
                Criterion::AdversarialDefined,
                Criterion::AsWholeTable,
                Criterion::AsGapContinues,
                Criterion::AsGapStops,
                Criterion::AMutual,
                Criterion::AStructure,
                Criterion::ALimit,
            ],
        ),
        // `research/prompts/e165-r5-prereg.md` 比对对象表第 6 行（`replay.sh` 里没有复跑行）；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-odd-marker-legal-family.out`，17 条
        (
            "奇数版本的 r2 合法",
            Family::LegalHistory,
            RunProfile::varying_fields(odd),
            &[
                Criterion::LegalDefined,
                Criterion::ReturnedRoot,
                Criterion::RootPlusDisk,
                Criterion::WithinTheInFlightLimit,
                Criterion::FarthestVersion,
                Criterion::J5aAppliedSet,
                Criterion::J5bLandedVersion,
                Criterion::J5aAndJ5bAreSeparable,
                Criterion::FixedPoint,
                Criterion::Ia1LargestTxgFails,
                Criterion::Ia2NoVerificationCounted,
                Criterion::QnFixedPoint,
                Criterion::J7aInstanceTable,
                Criterion::J7aAllocationRecordTreeRoot,
                Criterion::J7aFilesystemIdentifier,
                Criterion::J7bUnmountMarkerOfARebuiltRoot,
                Criterion::J7cUnmountMarkerWhenNothingIsApplied,
            ],
        ),
        // `research/prompts/e165-r5-prereg.md` 比对对象表第 7 行（`replay.sh` 里没有复跑行）；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-odd-marker-adversarial-family.out`，7 条
        (
            "奇数版本的 r2 对抗",
            Family::Adversarial,
            RunProfile::varying_fields(odd),
            &[
                Criterion::AdversarialDefined,
                Criterion::AsWholeTable,
                Criterion::AsGapContinues,
                Criterion::AsGapStops,
                Criterion::AMutual,
                Criterion::AStructure,
                Criterion::ALimit,
            ],
        ),
        // `scripts/experiment-apparatus/replay.sh` 的 `driver_e165_r2_segment4_table_both_legal`；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-table-both-legal-family.out`，19 条
        (
            "两份表都取的 r2 合法",
            Family::LegalHistory,
            RunProfile::varying_fields(even).with_table_source(RecordTableSource::Both),
            &[
                Criterion::LegalDefined,
                Criterion::ReturnedRoot,
                Criterion::RootPlusDisk,
                Criterion::WithinTheInFlightLimit,
                Criterion::FarthestVersion,
                Criterion::J5aAppliedSet,
                Criterion::J5bLandedVersion,
                Criterion::J5aAndJ5bAreSeparable,
                Criterion::FixedPoint,
                Criterion::Ia1LargestTxgFails,
                Criterion::Ia2NoVerificationCounted,
                Criterion::QnFixedPoint,
                Criterion::J7aInstanceTable,
                Criterion::J7aAllocationRecordTreeRoot,
                Criterion::J7aFilesystemIdentifier,
                Criterion::J7bUnmountMarkerOfARebuiltRoot,
                Criterion::J7cUnmountMarkerWhenNothingIsApplied,
                Criterion::TableComparisonSeesADifferentTable,
                Criterion::TableComparisonSeesADifferentRow,
            ],
        ),
        // `scripts/experiment-apparatus/replay.sh` 的 `driver_e165_r2_segment4_table_both_adversarial`；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-table-both-adversarial-family.out`，9 条
        (
            "两份表都取的 r2 对抗",
            Family::Adversarial,
            RunProfile::varying_fields(even).with_table_source(RecordTableSource::Both),
            &[
                Criterion::AdversarialDefined,
                Criterion::AsWholeTable,
                Criterion::AsGapContinues,
                Criterion::AsGapStops,
                Criterion::AMutual,
                Criterion::AStructure,
                Criterion::ALimit,
                Criterion::TableComparisonSeesADifferentTable,
                Criterion::TableComparisonSeesADifferentRow,
            ],
        ),
        // `scripts/experiment-apparatus/replay.sh` 的 `driver_e165_r2_segment4_table_both_multi_disk`；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-table-both-multi-disk.out`，6 条
        (
            "两份表都取的 r2 多盘",
            Family::MultiDisk,
            RunProfile::varying_fields(even).with_table_source(RecordTableSource::Both),
            &[
                Criterion::MultiDiskAnyCopyIsOnDisk,
                Criterion::MultiDiskReturnedRoot,
                Criterion::MultiDiskFarthestVersion,
                Criterion::MultiDiskAppliedSetAndLandedVersion,
                Criterion::TableComparisonSeesADifferentTable,
                Criterion::TableComparisonSeesADifferentRow,
            ],
        ),
        // `scripts/experiment-apparatus/replay.sh` 的 `driver_e165_r2_segment4_reversed_disk_order_legal`；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-reversed-disk-order-legal-family.out`，19 条
        (
            "两份表都取、盘次序反报的 r2 合法",
            Family::LegalHistory,
            RunProfile::varying_fields(even)
                .with_table_source(RecordTableSource::Both)
                .with_reported_disk_order(ReportedDiskOrder::SecondDeviceBeforeFirst),
            &[
                Criterion::LegalDefined,
                Criterion::ReturnedRoot,
                Criterion::RootPlusDisk,
                Criterion::WithinTheInFlightLimit,
                Criterion::FarthestVersion,
                Criterion::J5aAppliedSet,
                Criterion::J5bLandedVersion,
                Criterion::J5aAndJ5bAreSeparable,
                Criterion::FixedPoint,
                Criterion::Ia1LargestTxgFails,
                Criterion::Ia2NoVerificationCounted,
                Criterion::QnFixedPoint,
                Criterion::J7aInstanceTable,
                Criterion::J7aAllocationRecordTreeRoot,
                Criterion::J7aFilesystemIdentifier,
                Criterion::J7bUnmountMarkerOfARebuiltRoot,
                Criterion::J7cUnmountMarkerWhenNothingIsApplied,
                Criterion::TableComparisonSeesADifferentTable,
                Criterion::TableComparisonSeesADifferentRow,
            ],
        ),
        // `scripts/experiment-apparatus/replay.sh` 的 `driver_e165_r2_segment4_reversed_disk_order_multi_disk`；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-reversed-disk-order-multi-disk.out`，6 条
        (
            "两份表都取、盘次序反报的 r2 多盘",
            Family::MultiDisk,
            RunProfile::varying_fields(even)
                .with_table_source(RecordTableSource::Both)
                .with_reported_disk_order(ReportedDiskOrder::SecondDeviceBeforeFirst),
            &[
                Criterion::MultiDiskAnyCopyIsOnDisk,
                Criterion::MultiDiskReturnedRoot,
                Criterion::MultiDiskFarthestVersion,
                Criterion::MultiDiskAppliedSetAndLandedVersion,
                Criterion::TableComparisonSeesADifferentTable,
                Criterion::TableComparisonSeesADifferentRow,
            ],
        ),
        // `scripts/experiment-apparatus/replay.sh` 的 `driver_e165_r3_segment1_adversarial`；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r3-adversarial-family.out`，8 条
        (
            "r3 对抗",
            Family::Adversarial,
            RunProfile::txg_order_split(even),
            &[
                Criterion::AdversarialDefined,
                Criterion::AsWholeTable,
                Criterion::AsGapContinues,
                Criterion::AsGapStops,
                Criterion::AMutual,
                Criterion::AStructure,
                Criterion::ALimit,
                Criterion::QnFixedPointSplitByTxgOrder,
            ],
        ),
        // `scripts/experiment-apparatus/replay.sh` 的 `driver_e165_r6_r3_legal`；产物 `e165-replay-bounded-enumeration-2026-10-09-r6-r3-legal-family.out`，17 条
        (
            "r3 合法",
            Family::LegalHistory,
            RunProfile::txg_order_split(even),
            &[
                Criterion::LegalDefined,
                Criterion::ReturnedRoot,
                Criterion::RootPlusDisk,
                Criterion::WithinTheInFlightLimit,
                Criterion::FarthestVersion,
                Criterion::J5aAppliedSet,
                Criterion::J5bLandedVersion,
                Criterion::J5aAndJ5bAreSeparable,
                Criterion::FixedPoint,
                Criterion::Ia1LargestTxgFails,
                Criterion::Ia2NoVerificationCounted,
                Criterion::QnFixedPoint,
                Criterion::J7aInstanceTable,
                Criterion::J7aAllocationRecordTreeRoot,
                Criterion::J7aFilesystemIdentifier,
                Criterion::J7bUnmountMarkerOfARebuiltRoot,
                Criterion::J7cUnmountMarkerWhenNothingIsApplied,
            ],
        ),
        // `scripts/experiment-apparatus/replay.sh` 的 `driver_e165_r5_segment3_legal`；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-legal-family.out`，16 条
        (
            "r5 合法",
            Family::LegalHistory,
            RunProfile::root_fields_in_records(even),
            &[
                Criterion::LegalDefined,
                Criterion::ReturnedRoot,
                Criterion::RootPlusDisk,
                Criterion::WithinTheInFlightLimit,
                Criterion::FarthestVersion,
                Criterion::J5aAppliedSet,
                Criterion::J5bLandedVersion,
                Criterion::J5aAndJ5bAreSeparable,
                Criterion::FixedPoint,
                Criterion::Ia1LargestTxgFails,
                Criterion::Ia2NoVerificationCounted,
                Criterion::QnFixedPoint,
                Criterion::J8aFieldsFromTheNewRootSegment,
                Criterion::J8aUnmountMarkerFromTheNewRootSegment,
                Criterion::J8bFilesystemIdentifierOfTheChosenRoot,
                Criterion::J8cNothingAppliedReturnsTheChosenRoot,
            ],
        ),
        // `scripts/experiment-apparatus/replay.sh` 的 `driver_e165_r5_segment3_adversarial`；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-adversarial-family.out`，8 条
        (
            "r5 对抗",
            Family::Adversarial,
            RunProfile::root_fields_in_records(even),
            &[
                Criterion::AdversarialDefined,
                Criterion::AsWholeTable,
                Criterion::AsGapContinues,
                Criterion::AsGapStops,
                Criterion::AMutual,
                Criterion::AStructure,
                Criterion::ALimit,
                Criterion::QnFixedPointSplitByTxgOrder,
            ],
        ),
        // `scripts/experiment-apparatus/replay.sh` 的 `driver_e165_r5_segment3_multi_disk`；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-multi-disk.out`，4 条
        (
            "r5 多盘",
            Family::MultiDisk,
            RunProfile::root_fields_in_records(even),
            &[
                Criterion::MultiDiskAnyCopyIsOnDisk,
                Criterion::MultiDiskReturnedRoot,
                Criterion::MultiDiskFarthestVersion,
                Criterion::MultiDiskAppliedSetAndLandedVersion,
            ],
        ),
        // `scripts/experiment-apparatus/replay.sh` 的 `driver_e165_r5_segment3_odd_marker_legal`；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-odd-marker-legal-family.out`，16 条
        (
            "奇数版本的 r5 合法",
            Family::LegalHistory,
            RunProfile::root_fields_in_records(odd),
            &[
                Criterion::LegalDefined,
                Criterion::ReturnedRoot,
                Criterion::RootPlusDisk,
                Criterion::WithinTheInFlightLimit,
                Criterion::FarthestVersion,
                Criterion::J5aAppliedSet,
                Criterion::J5bLandedVersion,
                Criterion::J5aAndJ5bAreSeparable,
                Criterion::FixedPoint,
                Criterion::Ia1LargestTxgFails,
                Criterion::Ia2NoVerificationCounted,
                Criterion::QnFixedPoint,
                Criterion::J8aFieldsFromTheNewRootSegment,
                Criterion::J8aUnmountMarkerFromTheNewRootSegment,
                Criterion::J8bFilesystemIdentifierOfTheChosenRoot,
                Criterion::J8cNothingAppliedReturnsTheChosenRoot,
            ],
        ),
        // `scripts/experiment-apparatus/replay.sh` 的 `driver_e165_r5_segment3_odd_marker_adversarial`；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-odd-marker-adversarial-family.out`，8 条
        (
            "奇数版本的 r5 对抗",
            Family::Adversarial,
            RunProfile::root_fields_in_records(odd),
            &[
                Criterion::AdversarialDefined,
                Criterion::AsWholeTable,
                Criterion::AsGapContinues,
                Criterion::AsGapStops,
                Criterion::AMutual,
                Criterion::AStructure,
                Criterion::ALimit,
                Criterion::QnFixedPointSplitByTxgOrder,
            ],
        ),
    ];
    let mut controlled_in_some_unit = std::collections::BTreeSet::new();
    for (unit_name, family, run_profile, registered_criteria) in formal_run_units {
        let readings = positive_control_readings(&[family], run_profile, thread_count);
        let controlled: std::collections::BTreeSet<Criterion> =
            readings.iter().map(|reading| reading.criterion).collect();
        let registered: std::collections::BTreeSet<Criterion> =
            registered_criteria.iter().copied().collect();
        assert_eq!(
            controlled, registered,
            "{unit_name} 那个单元跑出对照的判据与那条正式跑产物里的那一份不同"
        );
        for reading in &readings {
            assert!(
                reading.is_red_as_required,
                "{unit_name} 那个单元对照没红：{reading:?}"
            );
        }
        controlled_in_some_unit.extend(controlled);
    }
    for criterion in Criterion::ALL.iter() {
        assert!(
            controlled_in_some_unit.contains(criterion),
            "判据 {} 在正式跑的单元里都没有对照的读数",
            criterion.label()
        );
    }
}
