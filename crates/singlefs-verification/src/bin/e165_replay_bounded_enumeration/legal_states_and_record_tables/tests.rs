use super::*;

/// 一张表键相同、其中一条记录的新根段不同：必须报「两张表不同」（登记 r2 9.5「T-表只比键集合、不比值」）。
#[test]
fn two_record_tables_with_the_same_keys_and_a_different_new_root_segment_are_reported_as_different()
{
    let record = hand_made_record(7, 101, 1, true);
    let mut other_record = record.clone();
    other_record.new_tree_identifier_watermark += 1;
    let direct_table = table_of(vec![record]);
    let scanned_table = table_of(vec![other_record]);
    assert!(
        direct_table.keys().eq(scanned_table.keys()),
        "两张表的键相同"
    );
    assert!(the_two_record_tables_differ(&direct_table, &scanned_table));
    assert!(!the_two_record_tables_differ(
        &direct_table,
        &direct_table.clone()
    ));
}

/// 两行的计数器标签集合相同、结局一边「报错」一边「返回根」：必须报「两行不同」（登记 r2 9.5「T-行只比计数器标签、不比实现结局」）。
#[test]
fn two_judged_rows_with_the_same_counter_labels_and_a_failure_against_a_returned_root_are_reported_as_different(
) {
    let returned = ReplayOutcome::Returned(ReturnedReplay {
        root: adversarial_chosen_root(RunProfile::CONSTANT_FIELDS),
        prefix_applied: 0,
        verification_passed: 0,
    });
    let failed = ReplayOutcome::Failed(FailureKind::Other);
    let summary = |first_outcome: ReplayOutcome, counter_labels: &str| JudgedRowSummary {
        first_outcome,
        counter_labels: counter_labels.to_string(),
    };
    assert!(the_two_judged_rows_differ(
        &summary(returned, "states,J4"),
        &summary(failed, "states,J4")
    ));
    assert!(the_two_judged_rows_differ(
        &summary(returned, "states,J4"),
        &summary(returned, "states")
    ));
    assert!(!the_two_judged_rows_differ(
        &summary(returned, "states,J4"),
        &summary(returned, "states,J4")
    ));
}

/// 两种建表法在最小的几档上逐态相同（登记 r2 第七节甲类「全部落盘那一态的扫盘表条数等于生成的条数」由装置在每个这样的态上断言，
/// 断言没跑到的域这里也报红）：合法历史一族、多盘一族、对抗一族各取最小的两三档。
#[test]
fn the_two_table_sources_agree_on_the_smallest_sizes_and_the_scan_table_of_every_fully_landed_state_holds_every_record(
) {
    let run_profile =
        RunProfile::varying_fields(MarkerParity::EvenVersionsWrittenByTheUnmountSequence)
            .with_table_source(RecordTableSource::Both);
    for (domain, sizes) in [
        (Domain::LegalBigRing, 1..=3),
        (Domain::LegalSmallRingSix, 1..=3),
        (Domain::NamedUnitsVerified, 1..=2),
        (Domain::MonotoneBigRing, 1..=2),
        (Domain::CrossInstance, 2..=3),
        (Domain::TxgOneBelowTheLargest, 1..=3),
        (Domain::Residue, 2..=3),
        (Domain::MirroredBigRing, 1..=2),
        (Domain::MirroredSmallRingSix, 1..=2),
        (Domain::MirroredMonotoneBigRing, 1..=2),
        (Domain::AdversarialBigRing, 1..=1),
    ] {
        for size in sizes {
            let tally = tally_of_a_domain_size(
                domain,
                size,
                Implementation::TodayReplayJournal,
                run_profile,
                1,
            );
            assert_eq!(
                (
                    tally.count_of(Counter::TablesDiffer),
                    tally.count_of(Counter::RowsDiffer)
                ),
                (0, 0),
                "{} 域 N = {size}：两种建表法逐态相同",
                domain.label()
            );
            if domain.family() != Family::Adversarial {
                assert!(
                    tally.count_of(Counter::FullyLandedScanTableSizeChecked) > 0,
                    "{} 域 N = {size}：全部落盘那一态的扫盘表条数断言要真跑到",
                    domain.label()
                );
                assert!(tally.count_of(Counter::DirectTableHoldsARecord) > 0);
            }
        }
    }
}

/// 盘次序反过来（登记 r2 第八节「判别力自证」后一半）：R 域 N = 2，残留在 1 号盘，扫盘先读到残留；「盘号升序」那一栏的直接表在
/// 0 号盘那一份落了的态上与扫盘表不同，必须大于 0。「照盘报的次序」那一栏是推 s7 的预期（0），不是判别力自证，不在这里断言。
#[test]
fn with_the_disk_order_reversed_the_scan_takes_the_residue_first_so_the_device_number_order_direct_table_differs(
) {
    let run_profile =
        RunProfile::varying_fields(MarkerParity::EvenVersionsWrittenByTheUnmountSequence)
            .with_table_source(RecordTableSource::Both)
            .with_reported_disk_order(ReportedDiskOrder::SecondDeviceBeforeFirst);
    let tally = tally_of_a_domain_size(
        Domain::Residue,
        2,
        Implementation::TodayReplayJournal,
        run_profile,
        1,
    );
    assert!(
        tally.count_of(Counter::TablesDiffer) > 0,
        "R 域 N = 2：盘号升序一栏的直接表在残留所在那条记录 0 号盘那份落了的态上要与扫盘表不同"
    );
}
