//! 本地常量与实现的回比（登记第三节、停机条款 H2）与装置自检：手造记录与表上的各条自检、各档的自检表。

use super::*;

// ── 常量回比（登记第三节、停机条款 H2）与装置自检

/// 本地常量逐个对着实现侧的同名常量回比；在飞上限与记录落点在装置取的每个环长上对着实现的函数回比。
fn cross_check_local_constants_against_the_implementation() {
    assert_eq!(
        LOCAL_JOURNAL_RECORD_BYTES,
        singlefs_format::JOURNAL_RECORD_BYTES,
        "H2：记录字节数"
    );
    assert_eq!(
        LOCAL_JOURNAL_SAFETY_FACTOR,
        singlefs_format::JOURNAL_SAFETY_FACTOR,
        "H2：安全系数 F"
    );
    assert_eq!(
        LOCAL_JOURNAL_RING_START_SLOT,
        singlefs_format::JOURNAL_RING_START_SLOT,
        "H2：环起点槽号"
    );
    assert_eq!(
        LOCAL_SLOT_BYTES,
        singlefs_format::SLOT_BYTES,
        "H2：槽字节数"
    );
    assert_eq!(
        LOCAL_FIRST_JOURNAL_COUNTER, FIRST_JOURNAL_COUNTER,
        "H2：第一个计数器"
    );
    assert_eq!(
        LOCAL_INDEX_NODE_UNIT_BYTES,
        singlefs_format::NODE_BYTES,
        "H2：索引节点单元字节数"
    );
    assert_eq!(
        LOCAL_UNIT_CLASS_INDEX_NODE,
        singlefs_core::unit::UNIT_CLASS_INDEX_NODE,
        "H2：索引节点的类标签"
    );
    assert_eq!(
        SlotNumber(NAMED_UNIT_FIRST_SLOT).to_device_offset().0,
        NAMED_UNIT_FIRST_SLOT * LOCAL_SLOT_BYTES,
        "H2：槽号到设备内偏移的换算"
    );
    // 甲类锚点（登记第七节）：64 槽 ⇒ 21，9 槽 ⇒ 3，6 槽 ⇒ 2；不符走 F11。
    for (ring, pinned_limit) in [
        (RingGeometry::BIG, 21usize),
        (RingGeometry::SMALL_NINE, 3),
        (RingGeometry::SMALL_SIX, 2),
    ] {
        assert_eq!(
            ring.local_in_flight_limit(),
            pinned_limit,
            "F11：装置自己算的在飞上限（环 {} 槽）",
            ring.ring_slots
        );
        assert_eq!(
            u64::try_from(ring.local_in_flight_limit()).expect("上限不超过 21"),
            journal_in_flight_record_limit(ring.ring_bytes()),
            "F11 / H2：装置自己算的在飞上限与 journal_in_flight_record_limit 不等（环 {} 槽）",
            ring.ring_slots
        );
        for counter in LOCAL_FIRST_JOURNAL_COUNTER..=200 {
            assert_eq!(
                ring.local_record_offset_in_bytes(counter),
                record_offset(counter, ring.ring_bytes()).0,
                "H2：记录落点（计数器 {counter}，环 {} 槽）",
                ring.ring_slots
            );
        }
    }
}

pub(super) fn hand_made_record(
    counter: u64,
    checkpoint_txg: u64,
    ordinal_within_publish: u32,
    carries_the_last_record_flag: bool,
) -> JournalRecord {
    let segment = NewRootSegment::off_timeline(counter, checkpoint_txg);
    JournalRecord {
        instance: InstanceGeneration(CHOSEN_INSTANCE),
        counter,
        checkpoint_txg: CheckpointTxg(checkpoint_txg),
        transaction: 1,
        is_commit: true,
        ordinal_within_publish: JournalRecordOrdinalWithinPublish(ordinal_within_publish),
        place_in_publish: if carries_the_last_record_flag {
            JournalRecordPlaceInPublish::LastRecordOfThePublish
        } else {
            JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow
        },
        back_chain: 0,
        filesystem_identifier: FILESYSTEM_IDENTIFIER_LOW_EIGHT_BYTES,
        new_tree_table: segment.tree_table,
        new_mapping_root: segment.mapping_root,
        new_tree_identifier_watermark: segment.tree_identifier_watermark,
        new_rollback_floor: CheckpointTxg(segment.rollback_floor),
        new_instance_table: fields_beyond_the_projection_carried_by_every_generated_record()
            .instance_table,
        new_allocation_record_tree_root:
            fields_beyond_the_projection_carried_by_every_generated_record()
                .allocation_record_tree_root,
        new_unmount_marker: fields_beyond_the_projection_carried_by_every_generated_record()
            .unmount_marker,
        named: Vec::new(),
        new_reclaim_threshold: singlefs_core::address::CheckpointTxg(0),
        new_pinned_ring_slot: singlefs_core::root_record::PinnedRingSlot::NO_SLOT_PINNED,
    }
}

pub(super) fn table_of(records: Vec<JournalRecord>) -> RecordTable {
    records
        .into_iter()
        .map(|record| ((record.instance, record.counter), record))
        .collect()
}

/// S 在一张手写的表上、三种读法各自的期望（所选根取对抗域那一条，上限取大环的，表里的前一条一律当不在盘上）。
fn expectations_on_a_hand_made_table(records: &RecordTable) -> [SpecificationExpectation; 3] {
    let chosen_root = adversarial_chosen_root(RunProfile::CONSTANT_FIELDS);
    let view = SpecificationView {
        chosen_root: &chosen_root,
        records,
        in_flight_limit: RingGeometry::BIG.local_in_flight_limit(),
        verify_named_units: false,
        truths: &[],
    };
    FollowReading::REGISTERED.map(|reading| specification_expectation(&view, reading))
}

fn check_the_back_chain_of_every_generated_record() {
    for ring_domain in [Domain::LegalBigRing, Domain::LegalSmallRingSix] {
        for unit in legal_work_units(ring_domain, 4, RunProfile::CONSTANT_FIELDS) {
            let history = generate_history(&unit.plan);
            for (record_index, slot) in history.slots.iter().enumerate() {
                let expected_back_chain = match record_index.checked_sub(1) {
                    Some(index_before) => back_chain_of(&history.slots[index_before].bytes),
                    None => BACK_CHAIN_WHEN_THE_PREVIOUS_SLOT_HOLDS_NOTHING,
                };
                let parsed = slot.parsed.as_ref().expect("合法历史的每条记录都解得出");
                assert_eq!(
                    parsed.back_chain, expected_back_chain,
                    "生成的每条记录的反向链等于 back_chain_of(前一条的字节)（历史 {}，第 {record_index} 条）",
                    history.shape_description
                );
            }
        }
    }
}

fn check_the_new_root_segment_of_every_generated_record() {
    for unit in legal_work_units(Domain::LegalBigRing, 4, RunProfile::CONSTANT_FIELDS) {
        let history = generate_history(&unit.plan);
        for publish_index in 0..history.publishes.len() {
            for record_index in history.record_indexes_of_publish(publish_index) {
                let parsed = history.slots[record_index]
                    .parsed
                    .as_ref()
                    .expect("合法历史的每条记录都解得出");
                assert_eq!(
                    VersionFields::of_record(parsed),
                    history.version_fields(publish_index),
                    "一次发布的每条记录带同一份新根段（历史 {}，第 {record_index} 条）",
                    history.shape_description
                );
            }
        }
    }
}

fn check_the_state_counts_of_the_smallest_sizes() {
    assert_eq!(
        every_subset_mask(3).collect::<Vec<u32>>(),
        vec![0, 1, 2, 3, 4, 5, 6, 7],
        "三位的子集掩码从空集起共 8 个"
    );
    for domain in Domain::ALL {
        // 多盘域只在 r2 档有，它们的最小几档在 [`check_the_multi_disk_state_counts_of_the_smallest_sizes`] 里照 r2 档核。
        if domain.family() == Family::MultiDisk {
            continue;
        }
        let largest_checked_size = (domain.smallest_size() + 1).min(2);
        for size in domain.smallest_size()..=largest_checked_size {
            let tally = tally_of_a_domain_size(
                *domain,
                size,
                Implementation::TodayReplayJournal,
                RunProfile::CONSTANT_FIELDS,
                1,
            );
            assert_the_pinned_absolute_values(*domain, size, &tally);
        }
    }
}

fn check_the_three_readings_on_hand_made_tables() {
    // 带标志的一条（txg 101）、txg 越过的一条（102）、同 txg 的一条（101）：全表看为真，连着看两种都在 txg 越过处停、为假。
    let flagged_then_later_txg_then_same_txg = table_of(vec![
        hand_made_record(8, 101, 1, true),
        hand_made_record(9, 102, 1, true),
        hand_made_record(10, 101, 2, false),
    ]);
    let flagged = &flagged_then_later_txg_then_same_txg[&(InstanceGeneration(CHOSEN_INSTANCE), 8)];
    assert_eq!(
        FollowReading::REGISTERED.map(|reading| same_publish_record_follows(
            reading,
            flagged,
            &flagged_then_later_txg_then_same_txg
        )),
        [true, false, false],
        "带标志一条、txg 越过一条、同 txg 一条：全表看 / 连着看·缺号不停 / 连着看·缺号即停"
    );
    let nothing_applied = SpecificationExpectation::ReturnsRoot(VersionFields::of_root(
        &adversarial_chosen_root(RunProfile::CONSTANT_FIELDS),
    ));
    let lands_on_counter_nine = SpecificationExpectation::ReturnsRoot(VersionFields::of_record(
        &hand_made_record(9, 102, 1, true),
    ));
    assert_eq!(
        expectations_on_a_hand_made_table(&flagged_then_later_txg_then_same_txg),
        [
            nothing_applied,
            lands_on_counter_nine,
            lands_on_counter_nine
        ],
        "同一张表上 S 三种读法的期望：全表看一条都不施加，连着看两种都施加到计数器 9 那一条"
    );
    // 带标志的一条、缺一号、同 txg 的一条：全表看与缺号不停为真，缺号即停为假。
    let flagged_then_gap_then_same_txg = table_of(vec![
        hand_made_record(8, 101, 1, true),
        hand_made_record(10, 101, 2, false),
    ]);
    let flagged_before_the_gap =
        &flagged_then_gap_then_same_txg[&(InstanceGeneration(CHOSEN_INSTANCE), 8)];
    assert_eq!(
        FollowReading::REGISTERED.map(|reading| same_publish_record_follows(
            reading,
            flagged_before_the_gap,
            &flagged_then_gap_then_same_txg
        )),
        [true, true, false],
        "带标志一条、缺一号、同 txg 一条：全表看 / 连着看·缺号不停 / 连着看·缺号即停"
    );
    let lands_on_counter_eight = SpecificationExpectation::ReturnsRoot(VersionFields::of_record(
        &hand_made_record(8, 101, 1, true),
    ));
    assert_eq!(
        expectations_on_a_hand_made_table(&flagged_then_gap_then_same_txg),
        [nothing_applied, nothing_applied, lands_on_counter_eight],
        "同一张表上 S 三种读法的期望：只有缺号即停施加计数器 8 那一条"
    );
}

/// U 域、两次发布各一条记录：第二条在盘上、它点名的单元 0 号盘那份校验和不对。
fn state_with_one_bad_unit_copy_judged_under(implementation: Implementation) -> Tally {
    let one_record_publish = || PublishShape {
        record_count: 1,
        commit_marker_bits: 0,
    };
    let plan = history_plan(
        Domain::NamedUnitsVerified,
        ORDINARY_FIRST_TXG,
        vec![one_record_publish(), one_record_publish()],
        Vec::new(),
        RunProfile::CONSTANT_FIELDS,
    );
    let history = generate_history(&plan);
    let presence = [
        RecordPresence::OnDisk(UnitState::BothCopiesGood),
        RecordPresence::OnDisk(UnitState::FirstDeviceCopyChecksumWrong),
    ];
    let state = LegalState {
        history: &history,
        presence: &presence,
        mirroring: JournalMirroring::FirstDeviceOnly,
        residue: None,
        chosen_version: 0,
        verify_named_units: true,
    };
    let mut tally = Tally::new();
    judge_a_legal_state(
        &state,
        implementation,
        SpecificationSelfCheck::Judged,
        false,
        &mut tally,
    );
    tally
}

fn check_root_plus_disk_on_a_bad_unit_copy() {
    let tally = state_with_one_bad_unit_copy_judged_under(Implementation::Control(
        ControlKind::LandsOnePublishTooFarOverAnIncompletePublish,
    ));
    assert_eq!(
        (
            tally.count_of(Counter::ReturnedRootViolations),
            tally.count_of(Counter::RootPlusDiskViolations),
            tally.count_of(Counter::FarthestAtLeastOne)
        ),
        (0, 1, 0),
        "记录在、点名单元一份坏：多落一版的假结局过 J1、不过 J2，j* 停在所选根"
    );
}

fn check_the_in_flight_limit_control_on_both_rings() {
    let readings = control_readings_of(
        Criterion::WithinTheInFlightLimit,
        RunProfile::CONSTANT_FIELDS,
        1,
    );
    assert_eq!(readings.len(), 2, "J3 的对照两个读数：小环、大环");
    assert!(
        readings[0].value > 0 && readings[0].is_red_as_required,
        "不看上限的假结局在小环上 J3 违例数大于 0：{readings:?}"
    );
    assert!(
        readings[1].value == 0 && readings[1].is_red_as_required,
        "不看上限的假结局在大环上 J3 违例数是 0：{readings:?}"
    );
}

fn check_the_farthest_version_on_the_small_ring() {
    // 小环 6（上限 2）：P_0 一条、P_1 两条、P_2 一条全在盘上 ⇒ j* = 1（再加 P_2 是 3 条，过上限）；不看上限是 2。
    let shape = |record_count: usize| PublishShape {
        record_count,
        commit_marker_bits: 0,
    };
    let plan = history_plan(
        Domain::LegalSmallRingSix,
        ORDINARY_FIRST_TXG,
        vec![shape(1), shape(2), shape(1)],
        Vec::new(),
        RunProfile::CONSTANT_FIELDS,
    );
    let history = generate_history(&plan);
    let presence = [RecordPresence::OnDisk(UnitState::BothCopiesGood); 4];
    let state = LegalState {
        history: &history,
        presence: &presence,
        mirroring: JournalMirroring::FirstDeviceOnly,
        residue: None,
        chosen_version: 0,
        verify_named_units: true,
    };
    let in_flight_limit = history.ring.local_in_flight_limit();
    assert_eq!(
        (
            farthest_version_of(&state, Some(in_flight_limit)),
            farthest_version_of(&state, None)
        ),
        (1, 2),
        "小环 6 上 1 + 2 + 1 全在盘上：j* 看上限是 1，不看上限是 2"
    );
}

fn check_the_fixed_point_control() {
    let tally = control_tally(
        Domain::LegalBigRing,
        3,
        ControlKind::SecondCallMovesOn,
        RunProfile::CONSTANT_FIELDS,
        1,
    );
    assert!(
        tally.count_of(Counter::FixedPointPremiseHoldsViolations) > 0,
        "第二次调用换成「再多落一版」的假结局，J6 违例数大于 0"
    );
}

fn check_the_two_monotone_readings_on_one_later_record() {
    let tally = tally_of_a_domain_size(
        Domain::MonotoneBigRing,
        1,
        Implementation::TodayReplayJournal,
        RunProfile::CONSTANT_FIELDS,
        1,
    );
    assert_eq!(
        (
            tally.pair(PairSet::Wide, PairColumn::Pairs),
            tally.pair(PairSet::Wide, PairColumn::IdealAppliedSetViolations),
            tally.pair(PairSet::Wide, PairColumn::IdealLandedVersionViolations)
        ),
        (9, 1, 0),
        "C 域 Nc = 1 的「宽」集合 9 对：理想恢复在施加集读法上恰 1 对违例（记录落了、根槽落没落那一对），落到的版本读法 0 对"
    );
}

fn check_the_structure_control() {
    let tally = control_tally(
        Domain::AdversarialBigRing,
        1,
        ControlKind::RootFromARecordWithoutTheLastRecordFlag,
        RunProfile::CONSTANT_FIELDS,
        1,
    );
    assert!(
        tally.count_of(Counter::StructureViolations) > 0,
        "返回根取自不带末条标志的记录的假结局，A-结构 违例数大于 0"
    );
}

fn check_a_residue_does_not_complete_a_publish() {
    // R 域 N = 2：P_1 那一条 0 号盘没落，1 号盘上有残留 ⇒ 它进了表，但 P_1 不算完整在盘上，j* 是 0。
    let units = legal_work_units(Domain::Residue, 2, RunProfile::CONSTANT_FIELDS);
    assert_eq!(units.len(), 1, "R 域 N = 2 只有一段历史、残留只有一个位置");
    let history = generate_history(&units[0].plan);
    let residue = generate_residue(&history, 1);
    let presence = [
        RecordPresence::OnDisk(UnitState::BothCopiesGood),
        RecordPresence::NotOnDisk,
    ];
    let state = LegalState {
        history: &history,
        presence: &presence,
        mirroring: JournalMirroring::FirstDeviceOnly,
        residue: Some(&residue),
        chosen_version: 0,
        verify_named_units: true,
    };
    let (records, _disk, _truths) = record_table_and_disk_of(&state);
    assert_eq!(records.len(), 2, "0 号盘那一份没落时残留进表");
    assert_eq!(
        farthest_version_of(&state, Some(history.ring.local_in_flight_limit())),
        0,
        "残留不算时间线上的记录：P_1 不完整，j* 停在所选根"
    );
}

fn check_parse_rejects_ordinal_zero_whose_checksums_hold() {
    let ordinal_zero = hand_made_record(8, 101, 0, true);
    let ordinal_one = hand_made_record(8, 101, 1, true);
    let slot = generate_slot(
        0,
        RingGeometry::BIG,
        &ordinal_zero,
        ParseExpectation::OrdinalZeroMustBeRejected,
    );
    assert!(
        slot.parsed.is_none(),
        "序号 0 的记录 JournalRecord::parse 拒收（读到 0 当损坏）"
    );
    // 只把序号那 4 字节改回 1、重算自证校验和，就解得出：被拒的原因只是序号。
    let readable = generate_slot(
        0,
        RingGeometry::BIG,
        &ordinal_one,
        ParseExpectation::MustRoundTrip,
    );
    assert_eq!(readable.parsed.as_ref(), Some(&ordinal_one));
    assert_ne!(slot.bytes, readable.bytes, "两条记录只差序号，字节不同");
}

/// 装置开跑先做的自检（不含阳性对照，那一段在 [`main`] 里单独跑、逐行进产物）。每一项都有一条同名的单测。
pub(super) const SELF_CHECKS: &[(&str, fn())] = &[
    (
        "local_constants_equal_the_implementation",
        cross_check_local_constants_against_the_implementation,
    ),
    (
        "back_chain_of_every_generated_record",
        check_the_back_chain_of_every_generated_record,
    ),
    (
        "new_root_segment_of_every_generated_record",
        check_the_new_root_segment_of_every_generated_record,
    ),
    (
        "state_counts_of_the_smallest_sizes",
        check_the_state_counts_of_the_smallest_sizes,
    ),
    (
        "three_readings_on_hand_made_tables",
        check_the_three_readings_on_hand_made_tables,
    ),
    (
        "root_plus_disk_on_a_bad_unit_copy",
        check_root_plus_disk_on_a_bad_unit_copy,
    ),
    (
        "farthest_version_on_the_small_ring",
        check_the_farthest_version_on_the_small_ring,
    ),
    (
        "two_monotone_readings_on_one_later_record",
        check_the_two_monotone_readings_on_one_later_record,
    ),
    (
        "residue_does_not_complete_a_publish",
        check_a_residue_does_not_complete_a_publish,
    ),
    (
        "parse_rejects_ordinal_zero_whose_checksums_hold",
        check_parse_rejects_ordinal_zero_whose_checksums_hold,
    ),
    (
        "in_flight_limit_control_on_both_rings",
        check_the_in_flight_limit_control_on_both_rings,
    ),
    ("fixed_point_control", check_the_fixed_point_control),
    ("structure_control", check_the_structure_control),
];

/// r2 档生成器的四个字段（登记 r2 第一节、第八节）：两种奇偶各取第 0–3 版逐字段比；fsid 低 8 字节不随版本变、本地取法与
/// `unit_filesystem_identifier` 回比（登记 r2 第三节）；由记录重建的根取 S1 的形态；r1 档生成器上「重建」与时间线根逐字段相同。
fn check_the_fields_beyond_the_projection_of_the_varying_fields_generator() {
    for version in 0..4usize {
        assert_eq!(
            RunProfile::CONSTANT_FIELDS.fields_beyond_the_projection_of_version(version),
            FieldsBeyondTheProjection {
                filesystem_identifier: ROOT_FILESYSTEM_IDENTIFIER,
                unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
                instance_table: node_pointer_at_slot(69_000, 1),
                allocation_record_tree_root: NodePointer::empty_root(),
            },
            "r1 档：第 {version} 版投影之外的四个字段与 r1 时的装置相同"
        );
    }
    for (marker_parity, versions_written_by_the_unmount_sequence) in [
        (
            MarkerParity::EvenVersionsWrittenByTheUnmountSequence,
            [0usize, 2],
        ),
        (MarkerParity::OddVersionsWrittenByTheUnmountSequence, [1, 3]),
    ] {
        for version in 0..4usize {
            let fields = RunProfile::varying_fields(marker_parity)
                .fields_beyond_the_projection_of_version(version);
            let version_number = u64::try_from(version).expect("版本数很小");
            let mut expected_filesystem_identifier = ROOT_FILESYSTEM_IDENTIFIER;
            expected_filesystem_identifier[15] = u8::try_from(version).expect("版本数很小");
            assert_eq!(
                (
                    fields.instance_table,
                    fields.allocation_record_tree_root,
                    fields.filesystem_identifier
                ),
                (
                    node_pointer_at_slot(69_000 + version_number, 1),
                    node_pointer_at_slot(68_000 + version_number, 1),
                    expected_filesystem_identifier
                ),
                "r2 档（{}）第 {version} 版：实例表槽 69 000 + j、分配记录树根槽 68 000 + j、fsid 末字节 j",
                marker_parity.label()
            );
            assert_eq!(
                fields.unmount_marker,
                if versions_written_by_the_unmount_sequence.contains(&version) {
                    UnmountMarker::WrittenByTheUnmountSequence
                } else {
                    UnmountMarker::NotWrittenByTheUnmountSequence
                },
                "r2 档（{}）第 {version} 版的卸载记号",
                marker_parity.label()
            );
            assert_eq!(
                local_filesystem_identifier_low_eight_bytes(&fields.filesystem_identifier),
                FILESYSTEM_IDENTIFIER_LOW_EIGHT_BYTES,
                "fsid 低 8 字节不随版本变，等于记录头那 8 字节"
            );
            assert_eq!(
                local_filesystem_identifier_low_eight_bytes(&fields.filesystem_identifier),
                singlefs_core::unit::unit_filesystem_identifier(&fields.filesystem_identifier),
                "登记 r2 第三节：fsid 低 8 字节的本地取法与 unit_filesystem_identifier 不等"
            );
        }
    }
    let one_record_publish = || PublishShape {
        record_count: 1,
        commit_marker_bits: 0,
    };
    let history_under = |run_profile: RunProfile| {
        generate_history(&history_plan(
            Domain::LegalBigRing,
            ORDINARY_FIRST_TXG,
            vec![one_record_publish(), one_record_publish()],
            Vec::new(),
            run_profile,
        ))
    };
    let varying_fields_profile =
        RunProfile::varying_fields(MarkerParity::EvenVersionsWrittenByTheUnmountSequence);
    let varying_fields_history = history_under(varying_fields_profile);
    let first_root = varying_fields_history.timeline_root(0);
    let second_root = varying_fields_history.timeline_root(1);
    assert_eq!(
        VersionFields::of_root(&second_root),
        varying_fields_history.version_fields(1)
    );
    assert_eq!(
        FieldsBeyondTheProjection::of_root(&second_root),
        varying_fields_profile.fields_beyond_the_projection_of_version(1),
        "时间线根 R_1 的四个字段照 r2 档第 1 版"
    );
    assert_eq!(
        adversarial_chosen_root(varying_fields_profile),
        first_root,
        "对抗一族的所选根就是时间线 R_0"
    );
    let rebuilt = varying_fields_history
        .version_fields(1)
        .to_root_rebuilt_over(&first_root);
    assert_eq!(
        (
            VersionFields::of_root(&rebuilt),
            rebuilt.instance_table,
            rebuilt.allocation_record_tree_root,
            rebuilt.filesystem_identifier,
            rebuilt.unmount_marker
        ),
        (
            varying_fields_history.version_fields(1),
            first_root.instance_table,
            first_root.allocation_record_tree_root,
            first_root.filesystem_identifier,
            UnmountMarker::NotWrittenByTheUnmountSequence
        ),
        "由记录重建的根：投影取记录的，实例表指针、分配记录树根、fsid 抄所选根，卸载记号 NotWritten"
    );
    let constant_fields_history = history_under(RunProfile::CONSTANT_FIELDS);
    assert_eq!(
        constant_fields_history
            .version_fields(1)
            .to_root_rebuilt_over(&constant_fields_history.timeline_root(0)),
        constant_fields_history.timeline_root(1),
        "r1 档生成器上由记录重建的根与时间线根逐字段相同（r1 档输出不变的前提）"
    );
}

/// `--profile` 与 `--marker-parity`（登记 r2 5.1、第八节）：不给 `--profile` 时 `replay.sh` 三份 r1 产物的那三条命令仍是 r1 档、
/// 参数原样留下；两个参数写在哪都行；`--marker-parity` 只在 r2 档有。
fn check_the_profile_options() {
    let split = |words: &[&str]| {
        split_off_the_profile_options(
            &words
                .iter()
                .map(|word| (*word).to_string())
                .collect::<Vec<String>>(),
        )
    };
    let owned = |words: &[&str]| {
        words
            .iter()
            .map(|word| (*word).to_string())
            .collect::<Vec<String>>()
    };
    for constant_fields_product_command in [
        ["--bounds", "segment1", "--family", "legal"],
        ["--bounds", "segment2", "--family", "legal"],
        ["--bounds", "segment1", "--family", "adversarial"],
    ] {
        assert_eq!(
            split(&constant_fields_product_command),
            Ok((
                RunProfile::CONSTANT_FIELDS,
                owned(&constant_fields_product_command)
            )),
            "不给 --profile 时是 r1 档、参数原样留下：{constant_fields_product_command:?}"
        );
    }
    assert_eq!(
        split(&["--profile", "r2", "--bounds", "segment1"]),
        Ok((
            RunProfile::varying_fields(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
            owned(&["--bounds", "segment1"])
        ))
    );
    assert_eq!(
        split(&[
            "--bounds",
            "segment1",
            "--profile",
            "r2",
            "--marker-parity",
            "odd"
        ]),
        Ok((
            RunProfile::varying_fields(MarkerParity::OddVersionsWrittenByTheUnmountSequence),
            owned(&["--bounds", "segment1"])
        ))
    );
    assert_eq!(
        split(&["--profile", "r2", "--list-states", "L:2"]),
        Ok((
            RunProfile::varying_fields(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
            owned(&["--list-states", "L:2"])
        )),
        "逐态判定行同样认 --profile"
    );
    assert!(split(&["--marker-parity", "odd"]).is_err(), "r1 档不认奇偶");
    assert_eq!(
        split(&["--profile", "r3", "--bounds", "segment1"]),
        Ok((
            RunProfile::txg_order_split(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
            owned(&["--bounds", "segment1"])
        )),
        "r3 档认得，参数原样留下"
    );
    assert_ne!(
        RunProfile::txg_order_split(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
        RunProfile::varying_fields(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
        "r3 档与 r2 档是两个档"
    );
    assert_eq!(
        split(&[
            "--profile",
            "r5",
            "--marker-parity",
            "odd",
            "--bounds",
            "segment1"
        ]),
        Ok((
            RunProfile::root_fields_in_records(
                MarkerParity::OddVersionsWrittenByTheUnmountSequence
            ),
            owned(&["--bounds", "segment1"])
        )),
        "r5 档认得（登记 r5 5.1），奇偶照 r2 档的意思认，参数原样留下"
    );
    let (root_fields_in_records_profile, _remaining_arguments) =
        split(&["--profile", "r5"]).expect("r5 档认得（登记 r5 5.1）");
    assert!(
        root_fields_in_records_profile.judges_new_root_segment_fields()
            && !root_fields_in_records_profile.judges_copied_fields()
            && root_fields_in_records_profile.judges_j6qn(),
        "r5 档判 J8、J6Qn，不判 J7（登记 r5 5.1、5.3）"
    );
    assert!(
        !root_fields_in_records_profile.admits_the_root_choice_family(),
        "r5 档不认根环择根一族（登记 r5 5.1）"
    );
    assert_ne!(
        RunProfile::root_fields_in_records(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
        RunProfile::txg_order_split(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
        "r5 档与 r3 档是两个档"
    );
    assert!(split(&["--profile", "r6"]).is_err());
    assert!(split(&["--profile"]).is_err());
    assert!(split(&["--profile", "r2", "--profile", "r1"]).is_err());
}

/// J6Qn 三条对照的读数（登记 r2 第七节「5.5 的对照钉成等式」）：L 域 N ≤ 5 两种第二次报错各 3 110（违例与 judged），报
/// `CHOSEN_ROOT_…` 那一条原 J6 同值；E1 全界第一次取理想恢复时原 J6 70、J6Qn 0。
#[cfg(test)]
fn check_the_j6qn_control_readings() {
    let readings = control_readings_of(
        Criterion::QnFixedPoint,
        RunProfile::varying_fields(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
        1,
    );
    assert_eq!(
        readings
            .iter()
            .map(|reading| reading.value)
            .collect::<Vec<u64>>(),
        vec![3_110, 3_110, 3_110, 3_110, 3_110, 70, 0],
        "J6Qn 三条对照的读数：{readings:?}"
    );
    assert!(
        readings.iter().all(|reading| reading.is_red_as_required),
        "J6Qn 的对照都照登记红：{readings:?}"
    );
}

/// I-8.12 在手写的表上（登记 r3 第一节「Q3 的拆法」、9.2 甲）：同一实例里 txg 随计数器不减才成立；相等算成立（「严格增」那一读法判不成立）；
/// 换实例的两条不比；计数器断号不影响。
#[cfg(test)]
fn check_the_txg_order_judgement_on_hand_made_tables() {
    let in_another_instance = |mut record: JournalRecord| {
        record.instance = InstanceGeneration(CHOSEN_INSTANCE + 1);
        record
    };
    assert!(
        record_table_keeps_the_txg_order(&table_of(Vec::new())),
        "空表：成立"
    );
    assert!(
        record_table_keeps_the_txg_order(&table_of(vec![hand_made_record(7, 100, 1, true)])),
        "一条：成立"
    );
    assert!(
        record_table_keeps_the_txg_order(&table_of(vec![
            hand_made_record(7, 100, 1, true),
            hand_made_record(8, 100, 1, true)
        ])),
        "同一实例里相邻两条 txg 相等：不减，成立"
    );
    assert!(
        !record_table_keeps_the_txg_order(&table_of(vec![
            hand_made_record(7, 101, 1, true),
            hand_made_record(8, 100, 1, true)
        ])),
        "同一实例里后一条的 txg 比前一条小：不成立"
    );
    assert!(
        record_table_keeps_the_txg_order(&table_of(vec![
            hand_made_record(7, 100, 1, true),
            hand_made_record(9, 102, 1, true)
        ])),
        "计数器 7、9 之间断号、txg 上升：成立"
    );
    assert!(
        !record_table_keeps_the_txg_order(&table_of(vec![
            hand_made_record(7, 102, 1, true),
            hand_made_record(9, 100, 1, true)
        ])),
        "计数器 7、9 之间断号、txg 下降：不成立（相邻不减与任两条不减等价，断号照比）"
    );
    assert!(
        record_table_keeps_the_txg_order(&table_of(vec![
            hand_made_record(7, 101, 1, true),
            in_another_instance(hand_made_record(8, 100, 1, true))
        ])),
        "两条属于不同实例：I-8.12 只管同一实例里，换实例那一半不判"
    );
}

/// 对抗一族最小几档上，表成立 / 不成立 I-8.12 的态数等于登记的分类计数（登记 r3 第七节乙类；9.2 甲前两条变异靠它红）。
#[cfg(test)]
fn check_the_txg_order_state_counts_of_the_smallest_sizes() {
    let run_profile =
        RunProfile::txg_order_split(MarkerParity::EvenVersionsWrittenByTheUnmountSequence);
    for (domain, largest_size) in [
        (Domain::AdversarialBigRing, 3),
        (Domain::AdversarialSmallRingSix, 3),
        (Domain::AdversarialBackChain, 2),
    ] {
        for size in domain.smallest_size()..=largest_size {
            let tally = tally_of_a_domain_size(
                domain,
                size,
                Implementation::TodayReplayJournal,
                run_profile,
                1,
            );
            assert_the_pinned_txg_order_state_counts(domain, size, &tally);
            assert_eq!(
                the_split_rows_differ_from_the_unsplit_rows(&tally),
                None,
                "{} 域 n = {size}：拆开的四行对得回未拆的两行",
                domain.label()
            );
        }
    }
}

/// KF3 的四格读数（登记 r3 5.7、第七节乙类）：对照界里七个 (域, n)，每个 n 上四格的违例数与 judged、成立 / 不成立的态数共十个读数，都红。
#[cfg(test)]
fn check_the_txg_order_split_control_readings() {
    let readings = control_readings_of(
        Criterion::QnFixedPointSplitByTxgOrder,
        RunProfile::txg_order_split(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
        1,
    );
    assert_eq!(
        readings.len(),
        PINNED_KF3_CELLS.len() * 10,
        "每个 (域, n) 十个读数：四格各违例数与 judged 八个、成立与不成立的态数两个：{readings:?}"
    );
    assert!(
        readings.iter().all(|reading| reading.is_red_as_required),
        "KF3 的读数都等于钉的数：{readings:?}"
    );
}

/// 拆出来的行（登记 r3 5.6、9.2 甲）：r3 档对抗一族每个 (域, n) 五行（四格各一行、`i812_states` 一行）；r2 档与合法历史一族一行都没有。
#[cfg(test)]
fn check_the_rows_of_the_txg_order_split() {
    let txg_order_split_profile =
        RunProfile::txg_order_split(MarkerParity::EvenVersionsWrittenByTheUnmountSequence);
    let varying_fields_profile =
        RunProfile::varying_fields(MarkerParity::EvenVersionsWrittenByTheUnmountSequence);
    let tally = tally_of_a_domain_size(
        Domain::AdversarialBigRing,
        2,
        Implementation::TodayReplayJournal,
        txg_order_split_profile,
        1,
    );
    let place = "domain=A ring_slots=64 n=2";
    let rows = rows_of_the_txg_order_split(
        txg_order_split_profile,
        Domain::AdversarialBigRing,
        place,
        &tally,
    );
    assert_eq!(rows.len(), 5, "四格各一行、i812_states 一行：{rows:?}");
    for criterion in [
        "J6Qn_premise_holds_i812_holds",
        "J6Qn_premise_holds_i812_fails",
        "J6Qn_premise_fails_i812_holds",
        "J6Qn_premise_fails_i812_fails",
    ] {
        assert!(
            rows.iter()
                .any(|row| row.contains(&format!(" criterion={criterion} violations="))),
            "缺 {criterion} 那一行：{rows:?}"
        );
    }
    assert_eq!(
        rows[4],
        format!("name=i812_states {place} i812_holds=9122 i812_fails=1536 states=10658"),
        "A n = 2 的分类计数（登记 r3 第七节乙类）"
    );
    assert!(
        rows_of_the_txg_order_split(
            varying_fields_profile,
            Domain::AdversarialBigRing,
            place,
            &tally
        )
        .is_empty(),
        "r2 档不出这几行"
    );
    assert!(
        rows_of_the_txg_order_split(txg_order_split_profile, Domain::LegalBigRing, place, &tally)
            .is_empty(),
        "合法历史一族不出这几行"
    );
}

/// J7 五行对照的读数（登记 r2 第七节）：r2 档 J7a 各自那一格 762、另两格 0，J7b 762，J7c 2 348；r1 档生成器上都是 0。
#[cfg(test)]
fn check_the_copied_fields_control_readings() {
    let run_profile =
        RunProfile::varying_fields(MarkerParity::EvenVersionsWrittenByTheUnmountSequence);
    for (criterion, expected_values) in [
        (Criterion::J7aInstanceTable, vec![762, 0, 0, 0]),
        (Criterion::J7aAllocationRecordTreeRoot, vec![0, 762, 0, 0]),
        (Criterion::J7aFilesystemIdentifier, vec![0, 0, 762, 0]),
        (Criterion::J7bUnmountMarkerOfARebuiltRoot, vec![762, 0]),
        (
            Criterion::J7cUnmountMarkerWhenNothingIsApplied,
            vec![2_348, 0],
        ),
    ] {
        let readings = control_readings_of(criterion, run_profile, 1);
        assert_eq!(
            readings
                .iter()
                .map(|reading| reading.value)
                .collect::<Vec<u64>>(),
            expected_values,
            "{} 的对照读数：{readings:?}",
            criterion.label()
        );
        assert!(
            readings.iter().all(|reading| reading.is_red_as_required),
            "{} 的对照都照登记红：{readings:?}",
            criterion.label()
        );
    }
}

/// 第八节奇偶的判别力自证：`--marker-parity odd` 解出来的档上，J7b、J7c 两条对照在 L 域 N ≤ 5 都是 0（R_0 是 `NotWritten…`，
/// 假结局与期望同值），在 C 域 Nc ≤ 3 都大于 0。
#[cfg(test)]
fn check_the_odd_marker_parity_turns_the_unmount_marker_controls() {
    let arguments: Vec<String> = [
        "--profile",
        "r2",
        "--marker-parity",
        "odd",
        "--bounds",
        "segment1",
    ]
    .iter()
    .map(|word| (*word).to_string())
    .collect();
    let (run_profile, _remaining_arguments) =
        split_off_the_profile_options(&arguments).expect("这几个参数都认得");
    for (control, counter) in [
        (
            ControlKind::IdealCopyingTheChosenUnmountMarkerWhenApplied,
            Counter::J7bViolations,
        ),
        (
            ControlKind::IdealWritingNotWrittenWhenNothingIsApplied,
            Counter::J7cViolations,
        ),
    ] {
        let big_ring = control_tally(
            Domain::LegalBigRing,
            CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN,
            control,
            run_profile,
            1,
        );
        let monotone = control_tally(
            Domain::MonotoneBigRing,
            CONTROL_LARGEST_SIZE_OF_THE_MONOTONE_BIG_RING_DOMAIN,
            control,
            run_profile,
            1,
        );
        assert_eq!(
            big_ring.count_of(counter),
            0,
            "反奇偶：L 域 {} 对照是 0",
            counter.label()
        );
        assert!(
            monotone.count_of(counter) > 0,
            "反奇偶：C 域 {} 对照大于 0",
            counter.label()
        );
    }
}

/// r2 档另跑的启动自检（r1 档不跑、不进产物，r1 档的输出因此与 r1 时的装置逐字节相同）。每一项都有一条同名的单测。
///
/// 启动自检只放不读实现结局的：第二段把装置交给被变异的实现跑，实现一变，读它结局的断言在启动时红、一个域都没跑
/// （segment2b：登记行 406、414、448、518、1003、1004 五个变体在 [`check_the_j6qn_control_readings`] 上 panic）。
/// 逐条判（登记 r2 修订 4）：
/// - `fields_beyond_the_projection_of_the_r2_generator`：只读生成器、`VersionFields` 与 `unit_filesystem_identifier`，不调 `replay_journal`，留下。
/// - `profile_options`：纯参数解析，留下。
/// - `j6qn_control_readings`、`j7_control_readings`、`odd_marker_parity_turns_the_unmount_marker_controls`：三条都经 [`control_readings_of`] /
///   [`control_tally`] 跑假结局，假结局里「第一次照真结局」「第二次照真调用」都调 `replay_journal`（J6Qn 的读数直接取决于它返回什么，
///   J7 与奇偶两条的读数由理想恢复定、但同样调它）；挪成单测（`tests` 模块里同名的三条，断言一个字没动，三个检查函数只在 `cfg(test)` 下编）。产物里它们的读数
///   仍由 [`positive_control_readings`] 逐行打成 `name=control`，不红记 V2。
pub(super) const VARYING_FIELDS_SELF_CHECKS: &[(&str, fn())] = &[
    (
        "fields_beyond_the_projection_of_the_r2_generator",
        check_the_fields_beyond_the_projection_of_the_varying_fields_generator,
    ),
    ("profile_options", check_the_profile_options),
];

/// 多盘三个域最小的两档（登记 r2 第七节乙类）：状态数、M 与 M6 的落后类别与见证、MC 的宽对与见证，照 r2 档逐格比钉的值。
fn check_the_multi_disk_state_counts_of_the_smallest_sizes() {
    for domain in [
        Domain::MirroredBigRing,
        Domain::MirroredSmallRingSix,
        Domain::MirroredMonotoneBigRing,
    ] {
        for size in 1..=2 {
            let tally = tally_of_a_domain_size(
                domain,
                size,
                Implementation::TodayReplayJournal,
                RunProfile::varying_fields(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
                1,
            );
            assert_the_pinned_absolute_values(domain, size, &tally);
        }
    }
}

/// 落后的方向（登记 r2 SQ17、9.5「落后方向两行对调」）：0 号盘什么都没落、1 号盘落了第一条 ⇒ 0 号盘落后；反过来 1 号盘落后。
fn check_the_lagging_direction_on_hand_made_masks() {
    assert_eq!(
        lagging_class_of(0b0, 0b1),
        LaggingClass::FirstDeviceBehind,
        "0 号盘 {{}}、1 号盘 {{r_1}}：0 号盘的落盘集合真包含于 1 号盘的，是 0 号盘落后"
    );
    assert_eq!(
        lagging_class_of(0b1, 0b0),
        LaggingClass::SecondDeviceBehind,
        "0 号盘 {{r_1}}、1 号盘 {{}}：1 号盘落后"
    );
    assert_eq!(lagging_class_of(0b11, 0b11), LaggingClass::Equal);
    assert_eq!(lagging_class_of(0b01, 0b10), LaggingClass::Crossed);
}

/// 多盘四条判据的对照读数（登记 r2 5.5、第七节）：只数 0 号盘的 j\* 在 M、M6、L 上 100、996、0；原登记 J1、J4 两条对照在 M、M6、MC 上
/// 228、2 052、45（J4 那条的 J1、J2、J3 是 0）；MC 宽对上 J5 那条对照两列都大于 0。
fn check_the_multi_disk_control_readings() {
    let run_profile =
        RunProfile::varying_fields(MarkerParity::EvenVersionsWrittenByTheUnmountSequence);
    for (criterion, expected_values) in [
        (Criterion::MultiDiskAnyCopyIsOnDisk, vec![100, 996, 0]),
        (
            Criterion::MultiDiskReturnedRoot,
            vec![228, 228, 2_052, 2_052, 45, 45],
        ),
        (
            Criterion::MultiDiskFarthestVersion,
            vec![228, 0, 0, 0, 2_052, 0, 0, 0, 45, 0, 0, 0],
        ),
    ] {
        let readings = control_readings_of(criterion, run_profile, 1);
        assert_eq!(
            readings
                .iter()
                .map(|reading| reading.value)
                .collect::<Vec<u64>>(),
            expected_values,
            "{} 的对照读数：{readings:?}",
            criterion.label()
        );
    }
    let readings = control_readings_of(
        Criterion::MultiDiskAppliedSetAndLandedVersion,
        run_profile,
        1,
    );
    assert_eq!(readings.len(), 2, "J5 那条对照两个读数：J5a、J5b");
    assert!(
        readings
            .iter()
            .all(|reading| reading.value > 0 && reading.is_red_as_required),
        "MC 宽对上 J5 那条对照两列都大于 0：{readings:?}"
    );
}

/// 直接建表的盘次序（登记 5.1、r2 5.3、9.5「多盘直接建表先读 1 号盘」）：R 域 N = 2，P_1 那条在 0 号盘落了时表里取 0 号盘那一份、
/// 残留不进表；0 号盘那一份没落时残留进表。M 域两条记录、第二条只有 1 号盘那一份：它进表，P_1 完整在盘上。
fn check_the_direct_table_reads_the_first_device_before_the_second() {
    let units = legal_work_units(Domain::Residue, 2, RunProfile::CONSTANT_FIELDS);
    let history = generate_history(&units[0].plan);
    let residue = generate_residue(&history, 1);
    let second_record = history.slots[1]
        .parsed
        .clone()
        .expect("合法历史的每条记录都解得出");
    let residue_record = residue.slot.parsed.clone().expect("残留解得出");
    let key = (second_record.instance, second_record.counter);
    for (second_record_presence, expected_record) in [
        (
            RecordPresence::OnDisk(UnitState::BothCopiesGood),
            &second_record,
        ),
        (RecordPresence::NotOnDisk, &residue_record),
    ] {
        let presence = [
            RecordPresence::OnDisk(UnitState::BothCopiesGood),
            second_record_presence,
        ];
        let state = LegalState {
            history: &history,
            presence: &presence,
            mirroring: JournalMirroring::FirstDeviceOnly,
            residue: Some(&residue),
            chosen_version: 0,
            verify_named_units: true,
        };
        let (records, _disk, _truths) = record_table_and_disk_of(&state);
        assert_eq!(
            records.get(&key),
            Some(expected_record),
            "残留只在 0 号盘那一份没落时进表（P_1 那条 0 号盘：{second_record_presence:?}）"
        );
    }
    let one_record_publish = || PublishShape {
        record_count: 1,
        commit_marker_bits: 0,
    };
    let mirrored_history = generate_history(&history_plan(
        Domain::MirroredBigRing,
        ORDINARY_FIRST_TXG,
        vec![one_record_publish(), one_record_publish()],
        Vec::new(),
        RunProfile::varying_fields(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
    ));
    let presence = [
        RecordPresence::OnDisk(UnitState::BothCopiesGood),
        RecordPresence::NotOnDisk,
    ];
    let second_device_copy_is_on_disk = [false, true];
    let state = LegalState {
        history: &mirrored_history,
        presence: &presence,
        mirroring: JournalMirroring::BothDevices {
            second_device_copy_is_on_disk: &second_device_copy_is_on_disk,
        },
        residue: None,
        chosen_version: 0,
        verify_named_units: true,
    };
    let (records, _disk, _truths) = record_table_and_disk_of(&state);
    assert_eq!(records.len(), 2, "只有 1 号盘那一份的记录也进表");
    assert_eq!(
        farthest_version_of(&state, Some(mirrored_history.ring.local_in_flight_limit())),
        1,
        "多盘：记录任一份在即在，P_1 完整在盘上，j* = 1"
    );
}

/// 多盘一族另跑的自检（只在 `--family multi_disk` 时跑、进那一份产物）。每一项都有一条同名的单测。
pub(super) const MULTI_DISK_SELF_CHECKS: &[(&str, fn())] = &[
    (
        "multi_disk_state_counts_of_the_smallest_sizes",
        check_the_multi_disk_state_counts_of_the_smallest_sizes,
    ),
    (
        "lagging_direction_on_hand_made_masks",
        check_the_lagging_direction_on_hand_made_masks,
    ),
    (
        "multi_disk_control_readings",
        check_the_multi_disk_control_readings,
    ),
    (
        "direct_table_reads_the_first_device_before_the_second",
        check_the_direct_table_reads_the_first_device_before_the_second,
    ),
];

/// 扫盘建表喂的系统配置与生成器对得上（登记 r2 第三节回比表的 fsid 一行、第七节甲类）：fsid 低 8 字节的本地取法等于记录头里那 8 字节、
/// 等于 `unit_filesystem_identifier`，journal 环长等于这个环几何的字节数。不调 `scan_journal`（它是被测对象），可以放进启动自检。
fn check_the_scan_system_configuration_matches_the_generator() {
    for ring in [
        RingGeometry::BIG,
        RingGeometry::SMALL_SIX,
        RingGeometry::SMALL_NINE,
    ] {
        let configuration = system_configuration_for_the_scan(ring);
        let filesystem_identifier = configuration.immutable.filesystem_identifier;
        assert_eq!(
            local_filesystem_identifier_low_eight_bytes(&filesystem_identifier),
            FILESYSTEM_IDENTIFIER_LOW_EIGHT_BYTES,
            "扫盘建表的系统配置 fsid 低 8 字节要等于记录头里那 8 字节（环 {} 槽）",
            ring.ring_slots
        );
        assert_eq!(
            local_filesystem_identifier_low_eight_bytes(&filesystem_identifier),
            singlefs_core::unit::unit_filesystem_identifier(&filesystem_identifier),
            "登记 r2 第三节：fsid 低 8 字节的本地取法与 unit_filesystem_identifier 不等（环 {} 槽）",
            ring.ring_slots
        );
        assert_eq!(
            configuration.immutable.sizes.journal_ring_bytes,
            ring.ring_bytes(),
            "扫盘建表的系统配置里 journal 环长要等于这个环几何的字节数（环 {} 槽）",
            ring.ring_slots
        );
    }
}

/// `--table` 与 `--disk-order`（登记 r2 5.4、第八节）：不给取 `direct` 与 `normal`；写在哪都行；r1 档不认。
fn check_the_table_and_disk_order_options() {
    let owned = |words: &[&str]| {
        words
            .iter()
            .map(|word| (*word).to_string())
            .collect::<Vec<String>>()
    };
    let split = |words: &[&str]| split_off_the_profile_options(&owned(words));
    let even_varying_fields =
        RunProfile::varying_fields(MarkerParity::EvenVersionsWrittenByTheUnmountSequence);
    assert_eq!(
        split(&["--profile", "r2", "--bounds", "segment1"]),
        Ok((even_varying_fields, owned(&["--bounds", "segment1"]))),
        "不给 --table 与 --disk-order 时取 direct 与 normal"
    );
    assert_eq!(
        split(&[
            "--bounds",
            "segment1",
            "--table",
            "both",
            "--profile",
            "r2",
            "--disk-order",
            "reversed"
        ]),
        Ok((
            even_varying_fields
                .with_table_source(RecordTableSource::Both)
                .with_reported_disk_order(ReportedDiskOrder::SecondDeviceBeforeFirst),
            owned(&["--bounds", "segment1"])
        ))
    );
    assert_eq!(
        split(&["--profile", "r2", "--table", "scan"]),
        Ok((
            even_varying_fields.with_table_source(RecordTableSource::Scanned),
            Vec::new()
        ))
    );
    assert_eq!(
        split(&[
            "--profile",
            "r2",
            "--table",
            "direct",
            "--disk-order",
            "normal"
        ]),
        Ok((even_varying_fields, Vec::new()))
    );
    assert!(split(&["--table", "both"]).is_err(), "r1 档不认 --table");
    assert!(
        split(&["--disk-order", "reversed"]).is_err(),
        "r1 档不认 --disk-order"
    );
    assert!(split(&["--profile", "r2", "--table", "scanned"]).is_err());
    assert!(split(&["--profile", "r2", "--disk-order", "backwards"]).is_err());
    assert!(split(&["--profile", "r2", "--table"]).is_err());
    assert!(
        split(&["--profile", "r2", "--table", "scan", "--table", "both"]).is_err(),
        "同一个参数给了不止一次"
    );
}

/// 盘次序（登记 r2 第八节「判别力自证」前一半）：`--disk-order reversed` 的内存盘 `device_identities()` 报 1 号盘在前，默认的报 0 号盘在前。
fn check_a_reversed_memory_disk_reports_the_second_device_first() {
    let device_numbers_reported_by = |reported_disk_order: ReportedDiskOrder| -> Vec<u32> {
        MemoryDisk {
            ring: RingGeometry::BIG,
            reported_disk_order,
            contents: Vec::new(),
        }
        .device_identities()
        .iter()
        .map(|device| device.0)
        .collect()
    };
    assert_eq!(
        device_numbers_reported_by(ReportedDiskOrder::SecondDeviceBeforeFirst),
        vec![1, 0],
        "--disk-order reversed：device_identities() 要报 1 号盘在前"
    );
    assert_eq!(
        device_numbers_reported_by(ReportedDiskOrder::FirstDeviceBeforeSecond),
        vec![0, 1],
        "默认的盘次序：0 号盘在前"
    );
}

/// 记录表怎么建、盘次序有一个不是默认值时另跑的启动自检（只在 r2 档能设、进那一份产物）。每一项都有一条同名的单测。
/// 都不调 `scan_journal` 与 `replay_journal`：它们是被测对象，读它们结局的断言在单测里。
pub(super) const RECORD_TABLE_SELF_CHECKS: &[(&str, fn())] = &[
    (
        "scan_system_configuration_matches_the_generator",
        check_the_scan_system_configuration_matches_the_generator,
    ),
    (
        "table_and_disk_order_options",
        check_the_table_and_disk_order_options,
    ),
    (
        "reversed_memory_disk_reports_the_second_device_first",
        check_a_reversed_memory_disk_reports_the_second_device_first,
    ),
];

#[cfg(test)]
mod tests;
