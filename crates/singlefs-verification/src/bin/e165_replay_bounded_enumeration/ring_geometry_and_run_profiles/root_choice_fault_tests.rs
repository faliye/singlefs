use super::*;

// ── r5 档（登记 `research/prompts/e165-r5-prereg.md` 5.3、5.4、第七节乙类、第八节、9.2）

/// 登记 r5 5.3 第 1 条：第 j 次发布的每条记录带时间线第 j 版 R_j 的后三项，生成器真相里那条记录的投影对到同一份；
/// r1 档那种生成器上记录的后三项仍是 r1 常量。9.2「r5 档时间线记录的后三项写回 r1 常量」点名这一条。
#[test]
fn the_root_fields_in_records_generator_puts_the_three_fields_of_each_publishs_root_into_every_record_of_that_publish(
) {
    let shapes = || {
        vec![
            PublishShape {
                record_count: 1,
                commit_marker_bits: 0,
            },
            PublishShape {
                record_count: 2,
                commit_marker_bits: 0,
            },
            PublishShape {
                record_count: 1,
                commit_marker_bits: 0,
            },
        ]
    };
    for marker_parity in [
        MarkerParity::EvenVersionsWrittenByTheUnmountSequence,
        MarkerParity::OddVersionsWrittenByTheUnmountSequence,
    ] {
        let root_fields_in_records_profile = RunProfile::root_fields_in_records(marker_parity);
        let history = generate_history(&history_plan(
            Domain::LegalBigRing,
            ORDINARY_FIRST_TXG,
            shapes(),
            Vec::new(),
            root_fields_in_records_profile,
        ));
        for publish_index in 0..history.publishes.len() {
            let fields_of_the_version =
                RecordFieldsBeyondTheProjection::of_root(&history.timeline_root(publish_index));
            for record_index in history.record_indexes_of_publish(publish_index) {
                let parsed = history.slots[record_index]
                    .parsed
                    .as_ref()
                    .expect("合法历史的每条记录都解得出");
                assert_eq!(
                    RecordFieldsBeyondTheProjection::of_record(parsed),
                    fields_of_the_version,
                    "r5 档（{}）第 {publish_index} 次发布的第 {record_index} 条记录带 R_{publish_index} 的后三项",
                    marker_parity.label()
                );
                assert_eq!(
                    history
                        .record_truth
                        .fields_of(VersionFields::of_record(parsed)),
                    Some(fields_of_the_version),
                    "生成器真相里这条记录的投影对到同一份后三项"
                );
            }
        }
        let first = RecordFieldsBeyondTheProjection::of_root(&history.timeline_root(0));
        let second = RecordFieldsBeyondTheProjection::of_root(&history.timeline_root(1));
        assert!(
            first.instance_table != second.instance_table
                && first.allocation_record_tree_root != second.allocation_record_tree_root
                && first.unmount_marker != second.unmount_marker,
            "R_0 与 R_1 的后三项逐项不同（C8a 的钉值靠这一条）"
        );
    }
    let constant_record_fields = RecordFieldsBeyondTheProjection::of_root_fields(
        fields_beyond_the_projection_carried_by_every_generated_record(),
    );
    let constant_fields_generator_history = generate_history(&history_plan(
        Domain::LegalBigRing,
        ORDINARY_FIRST_TXG,
        shapes(),
        Vec::new(),
        RunProfile::CONSTANT_FIELDS_GENERATOR_WITH_ROOT_FIELDS_IN_RECORDS_CRITERIA,
    ));
    for slot in &constant_fields_generator_history.slots {
        let parsed = slot.parsed.as_ref().expect("合法历史的每条记录都解得出");
        assert_eq!(
            RecordFieldsBeyondTheProjection::of_record(parsed),
            constant_record_fields,
            "r1 档那种生成器上记录的后三项恒为 r1 常量"
        );
    }
}

/// 登记 r5 5.3 第 2 条：时间线之外的记录（对抗一族字母表里的、R 域的残留）后三项 h(d) 的 (i) 实例表指针与分配记录树根各是 d 的单射、
/// 与时间线第 0..12 版的同一字段两两不同；(ii) 卸载记号取与 R_0 相反的那一值（A 域 n = 1 第一个字母那条记录也如此）；(iii) 过 H4 往返
/// （A 域 n = 1、R 域 n = 2 整格生成一遍，生成时断言往返）。9.2「h(d) 的卸载记号取与 R_0 相同的值」点名这一条。
#[test]
fn the_records_off_the_timeline_carry_tails_distinct_from_every_version_and_the_unmount_marker_opposite_to_the_first_version(
) {
    let alphabet = adversarial_record_alphabet();
    let mut distinguishing_numbers: Vec<u64> = (0..12).collect();
    for position in 1..=4usize {
        for alphabet_index in 0..alphabet.len() {
            distinguishing_numbers
                .push(u64::try_from(position * 1000 + alphabet_index).expect("很小"));
        }
    }
    for marker_parity in [
        MarkerParity::EvenVersionsWrittenByTheUnmountSequence,
        MarkerParity::OddVersionsWrittenByTheUnmountSequence,
    ] {
        let root_fields_in_records_profile = RunProfile::root_fields_in_records(marker_parity);
        let first_version_marker = root_fields_in_records_profile
            .fields_beyond_the_projection_of_version(0)
            .unmount_marker;
        let mut instance_table_slots = std::collections::BTreeSet::new();
        let mut allocation_record_tree_root_slots = std::collections::BTreeSet::new();
        for distinguishing_number in &distinguishing_numbers {
            let tail = root_fields_in_records_profile
                .record_fields_off_the_timeline(*distinguishing_number);
            assert_ne!(
                tail.unmount_marker,
                first_version_marker,
                "（{}）d = {distinguishing_number} 的卸载记号与 R_0 相反",
                marker_parity.label()
            );
            for version in 0..=12usize {
                let fields_of_the_version =
                    root_fields_in_records_profile.fields_beyond_the_projection_of_version(version);
                assert!(
                    tail.instance_table != fields_of_the_version.instance_table
                        && tail.allocation_record_tree_root
                            != fields_of_the_version.allocation_record_tree_root,
                    "d = {distinguishing_number} 的两条指针与时间线第 {version} 版的同一字段不同"
                );
            }
            instance_table_slots.insert(tail.instance_table.locations[0].slot.0);
            allocation_record_tree_root_slots
                .insert(tail.allocation_record_tree_root.locations[0].slot.0);
        }
        assert_eq!(
            instance_table_slots.len(),
            distinguishing_numbers.len(),
            "实例表指针对 d 单射"
        );
        assert_eq!(
            allocation_record_tree_root_slots.len(),
            distinguishing_numbers.len(),
            "分配记录树根对 d 单射"
        );
        let chosen_root = adversarial_chosen_root(root_fields_in_records_profile);
        let (_projection, first_letter_tail) = adversarial_record_projection_and_fields(
            &chosen_root,
            alphabet[0],
            1,
            0,
            root_fields_in_records_profile,
        );
        assert_ne!(
            first_letter_tail.unmount_marker, chosen_root.unmount_marker,
            "A 域 n = 1 第一个字母那条记录的卸载记号与所选根 R_0 相反"
        );
        for (domain, size) in [(Domain::AdversarialBigRing, 1), (Domain::Residue, 2)] {
            let tally = tally_of_a_domain_size(
                domain,
                size,
                Implementation::TodayReplayJournal,
                root_fields_in_records_profile,
                1,
            );
            assert!(
                tally.count_of(Counter::States) > 0,
                "{} 域 n = {size} 生成过",
                domain.label()
            );
        }
    }
    let constant_record_fields = RecordFieldsBeyondTheProjection::of_root_fields(
        fields_beyond_the_projection_carried_by_every_generated_record(),
    );
    assert_eq!(
        RunProfile::CONSTANT_FIELDS_GENERATOR_WITH_ROOT_FIELDS_IN_RECORDS_CRITERIA
            .record_fields_off_the_timeline(1000),
        constant_record_fields,
        "r1 档那种生成器上时间线之外的记录也带 r1 常量"
    );
    assert_eq!(
        RunProfile::txg_order_split(MarkerParity::EvenVersionsWrittenByTheUnmountSequence)
            .record_fields_off_the_timeline(1000),
        constant_record_fields,
        "r1–r4 档一个字不改：时间线之外的记录带 r1 常量"
    );
}
