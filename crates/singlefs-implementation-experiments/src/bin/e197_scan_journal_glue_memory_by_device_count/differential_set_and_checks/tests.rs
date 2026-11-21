//! `differential_set_and_checks` 的单测。
use super::*;

#[test]
fn local_constants_equal_the_implementation_constants_and_the_default_ring_has_196608_slots() {
    constants_against_the_implementation();
    assert_eq!(RING_SLOTS, 196_608);
    assert_eq!(JOURNAL_RING_DEFAULT_BYTES % JOURNAL_RECORD_BYTES, 0);
    assert_eq!(JOURNAL_RING_DEFAULT_BYTES / JOURNAL_RECORD_BYTES, 196_608);
    assert_eq!(JOURNAL_NAMED_ENTRIES_PER_RECORD, 64);
}

#[test]
fn every_glue_arm_returns_the_table_scan_journal_returns_on_the_registered_differential_cases() {
    let expected_reference_records = [("D1", 64u64), ("D2", 1), ("D3", 64), ("D4", 1)];
    for (case, reference_records) in expected_reference_records {
        for arm in GLUE_ARMS_COMPARED_WITH_SCAN_JOURNAL {
            let (equal, records) = differential_result(case, arm);
            assert!(
                equal,
                "{case} 上 {} 交回的表与 scan_journal 不同",
                arm.name()
            );
            assert_eq!(records, reference_records, "{case} 的键数");
        }
    }
    let (lines, all_equal) = run_the_differential_set();
    assert!(all_equal);
    // 四例 × 五份胶水（原三份 12 行，登记修订一补两条读缓存臂 8 行）。
    assert_eq!(lines.len(), 20);
    assert!(lines
        .iter()
        .all(|line| line.starts_with("E7RESULT name=e197_diff case=D")));
}

#[test]
fn every_glue_arm_returns_the_table_scan_journal_returns_on_the_generated_pool_and_the_comparison_can_say_no(
) {
    for arm in GLUE_ARMS_COMPARED_WITH_SCAN_JOURNAL {
        for devices in 1..=3u32 {
            for fill in [Fill::Full, Fill::Half] {
                for named in [1usize, 3] {
                    let specification = CellSpecification {
                        arm,
                        devices,
                        fill,
                        named,
                        repeat: 1,
                    };
                    let (equal, records) = compare_arm_with_scan_journal(specification, 256);
                    let context = format!("{} {devices} 盘 {} K={named}", arm.name(), fill.name());
                    assert!(equal, "{context}");
                    assert_eq!(records, fill.filled_slots_per_device(256), "{context}");
                }
            }
        }
    }
    // 比法自己要能判不等：空跑臂交回空表，对 256 个键的基准表不等。
    let empty_arm = CellSpecification {
        arm: Arm::Empty,
        devices: 2,
        fill: Fill::Full,
        named: 1,
        repeat: 1,
    };
    assert_eq!(compare_arm_with_scan_journal(empty_arm, 256), (false, 256));
}

#[test]
fn scan_journal_keeps_the_first_device_copy_of_each_key_so_the_conflicting_copy_case_tells_a_per_device_merge_apart(
) {
    let (devices, source) = differential_case("D2").expect("D2");
    let configuration =
        system_configuration_for(DIFFERENTIAL_RING_SLOTS * JOURNAL_RECORD_BYTES, devices);
    let pool = GeneratedPool::new(
        devices,
        TEST_IMAGE_DEFAULT_BYTES,
        DIFFERENTIAL_RING_SLOTS,
        source,
    );
    let table = scan_journal(&pool, &configuration);
    assert_eq!(table.len(), 1);
    let kept = table
        .get(&(InstanceGeneration(PRODUCTION_INSTANCE), 5))
        .expect("计数器 5 的记录在");
    assert_eq!(kept.transaction, 100);
    let (later_devices, later_source) = differential_case("D4").expect("D4");
    let later_configuration = system_configuration_for(
        DIFFERENTIAL_RING_SLOTS * JOURNAL_RECORD_BYTES,
        later_devices,
    );
    let later_pool = GeneratedPool::new(
        later_devices,
        TEST_IMAGE_DEFAULT_BYTES,
        DIFFERENTIAL_RING_SLOTS,
        later_source,
    );
    let later_table = scan_journal(&later_pool, &later_configuration);
    assert_eq!(
        later_table
            .get(&(InstanceGeneration(PRODUCTION_INSTANCE), 3))
            .expect("计数器 3 的记录在")
            .transaction,
        300
    );
    assert!(differential_case("D5").is_none());
}

#[test]
fn the_checks_lines_state_the_constants_the_ring_and_the_sizes() {
    let lines = run_the_checks();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("constants_match_the_implementation=true"));
    assert!(lines[0].contains("ring_slots=196608 ring_slots_remainder=0"));
    assert!(lines[0].contains("device_holds_the_default_ring=true"));
    assert!(lines[0].contains("capacity_after_196608_pushes=262144"));
    assert!(lines[0].contains("capacity_after_786432_pushes=1048576"));
    assert!(
        lines[1].starts_with("E7RESULT name=e197_sizes size_location_entry=16 size_named_unit=64")
    );
    // 登记修订一第七节 A11：读缓存的键 24 字节。
    assert!(lines[1].ends_with(" size_cache_key=24"), "{}", lines[1]);
    assert_eq!(cache_key_bytes(), 24);
}
