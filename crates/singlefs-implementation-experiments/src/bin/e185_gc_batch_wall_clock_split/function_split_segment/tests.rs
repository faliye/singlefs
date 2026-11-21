use super::*;

#[test]
fn function_split_remainder_subtracts_every_replayed_function() {
    let reading = FunctionSplitReading {
        call_cpu_nanoseconds: 1_000.0,
        clone_before_cpu_nanoseconds: 100.0,
        clone_after_cpu_nanoseconds: 50.0,
        read_the_version_mapping_cpu_nanoseconds: 200.0,
        read_the_version_pinned_cpu_nanoseconds: 20.0,
        region_scan_cpu_nanoseconds: Some(300.0),
        choose_cpu_nanoseconds: None,
    };
    assert_eq!(reading.clone_cpu_nanoseconds(), 150.0);
    assert_eq!(reading.read_the_version_cpu_nanoseconds(), 220.0);
    assert_eq!(reading.remainder_cpu_nanoseconds(), 330.0);
    assert!(
        reading
            .fields_text()
            .contains("remainder_cpu_nanoseconds=330.0 clone_share=0.150000"),
        "{}",
        reading.fields_text()
    );
    let totals = function_split_round_totals(&[reading, reading]).expect("有两次");
    assert_eq!(totals, [2_000.0, 300.0, 440.0, 600.0, 0.0, 660.0]);
    let summary = function_split_summary_text(&[totals]);
    assert!(summary.starts_with("call_cpu_nanoseconds_median=2000.000000 call_share_median=1.000000 clone_cpu_nanoseconds_median=300.000000 clone_share_median=0.150000"), "{summary}");
    assert_eq!(function_split_round_totals(&[]), None);
}

/// 一条合成的映射条目（只给 J-S6 的比法用）。
fn synthetic_mapping_entry(key_byte: u8, slot: u64) -> CentralMappingEntry {
    let location = singlefs_core::pointer::LocationEntry {
        device: DeviceIdentity(0),
        slot: SlotNumber(slot),
        unit_checksum: 0,
    };
    CentralMappingEntry {
        key: vec![key_byte; 27],
        locations: [location, location],
    }
}

#[test]
fn mapping_entries_finding_ignores_the_order_but_not_a_missing_or_moved_entry() {
    let entries = vec![
        synthetic_mapping_entry(1, 100),
        synthetic_mapping_entry(2, 200),
    ];
    let mut reversed = entries.clone();
    reversed.reverse();
    assert_eq!(mapping_entries_finding(0, &entries, &reversed), None);
    assert!(mapping_entries_finding(0, &entries, &entries[..1]).is_some());
    let moved = vec![
        synthetic_mapping_entry(1, 100),
        synthetic_mapping_entry(2, 202),
    ];
    assert!(mapping_entries_finding(0, &entries, &moved).is_some());
}

/// 停机 J-S6 在一个真池上不触发：crates 的 `central_mapping_as_the_compaction_reads_it` 与装置那一份取出的映射条目相同。
#[test]
fn crates_and_apparatus_read_the_same_mapping_entries_of_a_fresh_pool() {
    let (_devices, session) = formatted_pool_with_the_first_file_and_a_session();
    let version = version_of_trees(&session);
    assert_eq!(
        mapping_entries_finding(
            0,
            &central_mapping_as_the_compaction_reads_it(version),
            &version_as_the_replay_reads_it(version).mapping_entries,
        ),
        None
    );
    assert_eq!(
        pinned_slots_of_the_version(version),
        version_as_the_replay_reads_it(version).pinned_slots
    );
}
