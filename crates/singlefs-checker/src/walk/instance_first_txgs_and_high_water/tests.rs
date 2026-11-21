use super::*;

/// I-8.14、I-3.19 用例的根环：环长 12（R = 3、S = 4），`occupied` 列出的槽坐着那个 txg 的根，别的槽全零。
fn root_ring_slot_readings_holding(
    occupied: &[((u64, u64), RootRingSlotReading)],
) -> BTreeMap<(u64, u64), RootRingSlotReading> {
    let mut readings: BTreeMap<(u64, u64), RootRingSlotReading> = (0..3u64)
        .flat_map(|region| {
            (0..4u64).map(move |slot| ((region, slot), RootRingSlotReading::AllZero))
        })
        .collect();
    for (position, reading) in occupied {
        readings.insert(*position, *reading);
    }
    readings
}

/// 一块盘上的一条记录：实例、计数器、txg、事务号，坐在环长 `ring_slots` 下它该落的槽里，带末条标志。
fn journal_record_in_its_slot(
    instance: u32,
    counter: u64,
    checkpoint_txg: u64,
    transaction: u64,
    ring_slots: u64,
) -> (JournalRecordKeyOnADevice, ScannedJournalRecord) {
    let mut record = scanned_record_of(instance, 0, 0);
    record.checkpoint_txg = checkpoint_txg;
    record.transaction = transaction;
    record.slot = (counter - 1) % ring_slots;
    (JournalRecordKeyOnADevice { counter, instance }, record)
}

/// I-8.14 在一个世界上的判定：根 `roots`、恢复眼里的记录 `records`、根环 `root_ring`、盘 0 读不出的 journal 槽 `journal_slots_that_do_not_read_back`。
fn first_txg_verdict_of(
    roots: &[(u64, u64, crate::RootView)],
    records: &[(JournalRecordKeyOnADevice, ScannedJournalRecord)],
    root_ring: &BTreeMap<(u64, u64), RootRingSlotReading>,
    journal_slots_that_do_not_read_back: Vec<u64>,
) -> InvariantVerdict {
    let records_as_the_recovery_reads_them: BTreeMap<(u32, u64), &ScannedJournalRecord> = records
        .iter()
        .map(|(key, record)| ((key.instance, key.counter), record))
        .collect();
    let mut judgements = Judgements::default();
    judge_the_first_txg_of_each_instance_above_the_older_instances(
        roots,
        &records_as_the_recovery_reads_them,
        root_ring,
        &[(0, journal_slots_that_do_not_read_back)],
        &mut judgements,
    );
    judgements
        .into_report()
        .into_iter()
        .find(|(invariant, _)| {
            *invariant
                == FIRST_TXG_OF_EACH_INSTANCE_IS_ABOVE_EVERY_ROOT_AND_RECORD_OF_THE_OLDER_INSTANCES
        })
        .map(|(_, verdict)| verdict)
        .expect("I-8.14 在清单里")
}

/// C764 的镜像：实例 1 的根 txg 1、2，记录 txg 3（根槽没落，下一次挂载要施加它）。
/// 新实例 2 的第一个 txg 照写者取 max(2, 3) + 1 = 4 ⇒ 成立；改小那一句（只看根环）取 3 ⇒ 等于实例 1 记录的 txg，红。
/// 读不出的根槽让成立降成部分没判，全零的槽不算读不出；违例照报违例。
#[test]
fn the_first_txg_of_a_new_instance_must_be_above_the_records_of_the_older_instance() {
    let ring_slots = 64;
    let older_records = [journal_record_in_its_slot(1, 3, 3, 2, ring_slots)];
    let roots_with_the_first_txg = |first_txg: u64| {
        vec![
            (1u64, 0u64, ring_root_of(1, 1, 0, 0, 255)),
            (2, 0, ring_root_of(1, 2, 0, 0, 255)),
            (
                first_txg % 3,
                (first_txg / 3) % 4,
                ring_root_of(2, first_txg, 0, 0, 255),
            ),
        ]
    };
    let ring_without_unreadable_slots = root_ring_slot_readings_holding(&[]);
    assert_eq!(
        first_txg_verdict_of(
            &roots_with_the_first_txg(4),
            &older_records,
            &ring_without_unreadable_slots,
            Vec::new()
        ),
        InvariantVerdict::Holds,
        "实例 2 第一个 txg 4 高于实例 1 的根 2 与记录 3"
    );
    assert!(
        matches!(
            first_txg_verdict_of(
                &roots_with_the_first_txg(3),
                &older_records,
                &ring_without_unreadable_slots,
                Vec::new()
            ),
            InvariantVerdict::Violated(_)
        ),
        "实例 2 第一个 txg 3 不高于实例 1 那条 txg 3 的记录：I-8.14 红"
    );
    let ring_with_an_unreadable_slot =
        root_ring_slot_readings_holding(&[((2, 3), RootRingSlotReading::DoesNotReadBack)]);
    assert!(
        matches!(
            first_txg_verdict_of(
                &roots_with_the_first_txg(4),
                &older_records,
                &ring_with_an_unreadable_slot,
                Vec::new()
            ),
            InvariantVerdict::PartiallyJudged(_)
        ),
        "根环有一槽读不出：那一槽可能坐着更早实例 txg 更大的根，成立降成部分没判"
    );
    assert!(
        matches!(
            first_txg_verdict_of(
                &roots_with_the_first_txg(4),
                &older_records,
                &ring_without_unreadable_slots,
                vec![9]
            ),
            InvariantVerdict::PartiallyJudged(_)
        ),
        "journal 环有一槽读不出：同上"
    );
    assert!(
        matches!(
            first_txg_verdict_of(
                &roots_with_the_first_txg(3),
                &older_records,
                &ring_with_an_unreadable_slot,
                Vec::new()
            ),
            InvariantVerdict::Violated(_)
        ),
        "读不出的槽不压违例"
    );
    assert_eq!(
        first_txg_verdict_of(
            &roots_with_the_first_txg(4)[..2],
            &older_records,
            &ring_without_unreadable_slots,
            Vec::new()
        ),
        InvariantVerdict::NotApplicable(NO_INSTANCE_WITH_AN_OLDER_INSTANCE_TO_COMPARE),
        "只有实例 1：没有更小的实例可比"
    );
}

/// I-3.19 在一行上的判定：根环 `root_ring`、根 `roots`、实例 1 的记录 `records`。
fn high_water_verdict_of(
    row: InstanceTableRow,
    roots: &[(u64, u64, crate::RootView)],
    root_ring: &BTreeMap<(u64, u64), RootRingSlotReading>,
    records: &[(JournalRecordKeyOnADevice, ScannedJournalRecord)],
) -> InvariantVerdict {
    let records_as_the_recovery_reads_them: BTreeMap<(u32, u64), &ScannedJournalRecord> = records
        .iter()
        .map(|(key, record)| ((key.instance, key.counter), record))
        .collect();
    let mut judgements = Judgements::default();
    judge_the_applied_transaction_high_water_of_the_instance_table_rows(
        &[row],
        AbandonedRootsFilteringOfTheCandidateSet::FilteredByTheNewestRootsInstanceTable,
        roots,
        root_ring,
        &geometry_of_a_ring_of_twelve(),
        &records_as_the_recovery_reads_them,
        &mut judgements,
    );
    judgements
        .into_report()
        .into_iter()
        .find(|(invariant, _)| {
            *invariant == INSTANCE_TABLE_ROW_HIGH_WATER_IS_ONE_A_RECOVERY_COULD_HAVE_APPLIED
        })
        .map(|(_, verdict)| verdict)
        .expect("I-3.19 在清单里")
}

/// C740 的镜像：环长 12，实例 1 的根 txg 1（区域 1 槽 0）、txg 2（区域 2 槽 0），txg 3 那次发布带事务号 4、
/// 记录已落（计数器 3）。行 (1, T = 3, W)。
/// T 的根槽（区域 0 槽 1）从没写过（全零）：那次挂载从 txg 2 起重放、施加 txg 3 ⇒ 集合 {4}，W 4 成立，W 写成 0 红（C740 的判别力自证）。
/// T 的根槽坐着 txg 3 的根：挂载时它读得出（落在 T 上，W 0）或一时读坏（从 txg 2 起重放，W 4）都合法 ⇒ 集合 {0, 4}，两种都成立
/// （「T 在环里 ⇒ W 必为 0」在后一种上误红）。T 的根槽读不出：认不出 0 在不在集合里，W 0 报部分没判。
/// txg 2 那条根已被盖掉（区域 1 槽 0 坐着环转过之后的 txg 13）而 T 的根槽从没写过：集合只剩认得出的那几格，W 4 报部分没判、不报违例。
#[test]
fn the_applied_transaction_high_water_of_a_row_must_be_one_some_starting_root_could_have_applied() {
    let ring_slots = 64;
    let roots = vec![
        (1u64, 0u64, ring_root_of(1, 1, 0, 0, 255)),
        (2, 0, ring_root_of(1, 2, 0, 0, 255)),
    ];
    let records = [
        journal_record_in_its_slot(1, 1, 1, 1, ring_slots),
        journal_record_in_its_slot(1, 2, 2, 2, ring_slots),
        journal_record_in_its_slot(1, 3, 3, 4, ring_slots),
    ];
    let row_with_the_high_water = |applied_transaction_high_water_mark: u64| InstanceTableRow {
        instance: 1,
        published_checkpoint_txg: 3,
        applied_transaction_high_water_mark,
    };
    let genesis_and_the_first_two = [
        (
            (0, 0),
            RootRingSlotReading::SelfVerified { checkpoint_txg: 0 },
        ),
        (
            (1, 0),
            RootRingSlotReading::SelfVerified { checkpoint_txg: 1 },
        ),
        (
            (2, 0),
            RootRingSlotReading::SelfVerified { checkpoint_txg: 2 },
        ),
    ];
    let ring_where_the_row_published_root_never_landed =
        root_ring_slot_readings_holding(&genesis_and_the_first_two);
    assert_eq!(
        high_water_verdict_of(
            row_with_the_high_water(4),
            &roots,
            &ring_where_the_row_published_root_never_landed,
            &records
        ),
        InvariantVerdict::Holds,
        "T 没落过盘、从 txg 2 起施加 txg 3：W 4"
    );
    assert!(
        matches!(
            high_water_verdict_of(
                row_with_the_high_water(0),
                &roots,
                &ring_where_the_row_published_root_never_landed,
                &records
            ),
            InvariantVerdict::Violated(_)
        ),
        "T 没落过盘而 W 写成 0：I-3.19 红"
    );
    let mut ring_where_the_row_published_root_landed =
        ring_where_the_row_published_root_never_landed.clone();
    ring_where_the_row_published_root_landed.insert(
        (0, 1),
        RootRingSlotReading::SelfVerified { checkpoint_txg: 3 },
    );
    for high_water in [0, 4] {
        assert_eq!(
                high_water_verdict_of(row_with_the_high_water(high_water), &roots, &ring_where_the_row_published_root_landed, &records),
                InvariantVerdict::Holds,
                "T 的根在环里：落在 T 上（W 0）与 T 那一槽一时读坏、从 txg 2 起重放（W 4）都合法，W {high_water} 成立"
            );
    }
    let mut ring_where_the_row_published_root_slot_does_not_read_back =
        ring_where_the_row_published_root_never_landed.clone();
    ring_where_the_row_published_root_slot_does_not_read_back
        .insert((0, 1), RootRingSlotReading::DoesNotReadBack);
    assert!(
        matches!(
            high_water_verdict_of(
                row_with_the_high_water(0),
                &roots,
                &ring_where_the_row_published_root_slot_does_not_read_back,
                &records
            ),
            InvariantVerdict::PartiallyJudged(_)
        ),
        "T 那一槽读不出：认不出 0 在不在集合里，W 0 报部分没判"
    );
    let mut ring_where_the_starting_root_was_overwritten =
        ring_where_the_row_published_root_never_landed.clone();
    ring_where_the_starting_root_was_overwritten.insert(
        (1, 0),
        RootRingSlotReading::SelfVerified { checkpoint_txg: 13 },
    );
    assert!(
        matches!(
            high_water_verdict_of(
                row_with_the_high_water(4),
                &roots[..0],
                &ring_where_the_starting_root_was_overwritten,
                &records
            ),
            InvariantVerdict::PartiallyJudged(_)
        ),
        "起点那几条根都被环盖掉了：集合里认得出的一格都没有，W 4 报部分没判、不报违例"
    );
}
