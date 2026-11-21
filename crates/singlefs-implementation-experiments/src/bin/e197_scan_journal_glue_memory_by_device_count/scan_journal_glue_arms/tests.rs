//! `scan_journal_glue_arms` 的单测。
use super::super::tests::{
    measure_in_a_process_of_its_own, small_cell, READ_STAGE_CACHE_ARMS, SCANNING_ARMS,
    SMALL_RING_SLOTS,
};
use super::*;

#[test]
fn every_scanning_arm_reads_once_per_ring_slot_per_device_and_returns_one_key_per_filled_slot() {
    let ring_slots = 256;
    for arm in SCANNING_ARMS.into_iter().chain(READ_STAGE_CACHE_ARMS) {
        for devices in 1..=3u32 {
            for (fill, filled) in [(Fill::Full, 256u64), (Fill::Half, 128)] {
                let outcome = small_cell(arm, devices, fill, 2, ring_slots);
                let context = format!("{} {devices} 盘 {}", arm.name(), fill.name());
                assert_eq!(
                    outcome.reads,
                    u64::from(devices) * ring_slots,
                    "{context} 读数"
                );
                assert_eq!(
                    outcome.reads_expected,
                    u64::from(devices) * ring_slots,
                    "{context}"
                );
                assert_eq!(outcome.records_returned, filled, "{context} 键数");
                assert!(outcome.anchors_hold(), "{context}");
                let reading = outcome
                    .resident
                    .as_ref()
                    .unwrap_or_else(|| panic!("{context} 量窗读不出（V11）"));
                assert_eq!(
                    reading.at_each_device_end_bytes.len(),
                    usize::try_from(devices).expect("盘数装得进 usize"),
                    "{context} 轨迹点数"
                );
            }
        }
    }
    for devices in 1..=3u32 {
        let empty = small_cell(Arm::Empty, devices, Fill::Full, 2, ring_slots);
        assert_eq!((empty.reads, empty.reads_expected), (0, 0));
        assert_eq!((empty.records_returned, empty.records_expected), (0, 0));
        assert!(empty.anchors_hold());
        assert_eq!(empty.table_digest, "none");
    }
}

#[test]
fn table_comparison_and_digest_see_a_changed_value_a_missing_key_and_nothing_else() {
    let (devices, source) = differential_case("D1").expect("D1");
    let configuration =
        system_configuration_for(DIFFERENTIAL_RING_SLOTS * JOURNAL_RECORD_BYTES, devices);
    let pool = GeneratedPool::new(
        devices,
        TEST_IMAGE_DEFAULT_BYTES,
        DIFFERENTIAL_RING_SLOTS,
        source,
    );
    let table = scan_journal(&pool, &configuration);
    let same = table.clone();
    assert!(tables_are_equal(&table, &same));
    assert_eq!(table_digest(&table), table_digest(&same));
    let key = (InstanceGeneration(PRODUCTION_INSTANCE), 7);
    let mut changed_value = table.clone();
    changed_value
        .get_mut(&key)
        .expect("计数器 7 在")
        .transaction += 1;
    assert!(!tables_are_equal(&table, &changed_value));
    assert_ne!(table_digest(&table), table_digest(&changed_value));
    let mut changed_named = table.clone();
    changed_named
        .get_mut(&key)
        .expect("计数器 7 在")
        .named
        .pop();
    assert!(!tables_are_equal(&table, &changed_named));
    let mut missing_key = table.clone();
    missing_key.remove(&key);
    assert!(!tables_are_equal(&table, &missing_key));
    assert_ne!(table_digest(&table), table_digest(&missing_key));
    let mut moved_key = table.clone();
    let record = moved_key.remove(&key).expect("计数器 7 在");
    moved_key.insert((InstanceGeneration(PRODUCTION_INSTANCE), 100), record);
    assert_ne!(table_digest(&table), table_digest(&moved_key));
}

#[test]
fn the_digest_follows_the_encoding_checksum_of_each_value_and_not_only_the_keys() {
    let (devices, source) = differential_case("D3").expect("D3");
    let configuration =
        system_configuration_for(DIFFERENTIAL_RING_SLOTS * JOURNAL_RECORD_BYTES, devices);
    let pool = GeneratedPool::new(
        devices,
        TEST_IMAGE_DEFAULT_BYTES,
        DIFFERENTIAL_RING_SLOTS,
        source,
    );
    let table = scan_journal(&pool, &configuration);
    let mut changed_value_only = table.clone();
    changed_value_only
        .get_mut(&(InstanceGeneration(PRODUCTION_INSTANCE), 3))
        .expect("计数器 3 在")
        .back_chain ^= 1;
    assert_eq!(
        table.keys().collect::<Vec<_>>(),
        changed_value_only.keys().collect::<Vec<_>>()
    );
    assert_ne!(table_digest(&table), table_digest(&changed_value_only));
}

#[test]
fn keeping_the_scan_until_return_peaks_above_the_baseline_by_the_named_unit_heap_of_the_first_copies(
) {
    let named = 64;
    let baseline =
        measure_in_a_process_of_its_own(Arm::Baseline, 2, Fill::Full, named, SMALL_RING_SLOTS);
    let control =
        measure_in_a_process_of_its_own(Arm::BaselinePc, 2, Fill::Full, named, SMALL_RING_SLOTS);
    let extra = control.peak() - baseline.peak();
    // 每条首份记录多一份点名项堆：一块 chunk(K × 点名项大小)（常驻页口径，登记修订三第七节 A7）。
    let expected_extra = (SMALL_RING_SLOTS
        * glibc_chunk_bytes(u64::try_from(named).expect("64") * structure_sizes().named_unit))
        as f64;
    assert!(
        control.peak() * 10.0 > baseline.peak() * 11.0,
        "阳性对照要比基线高出 10% 以上：{} 对 {}",
        control.peak(),
        baseline.peak()
    );
    assert!(
        (extra - expected_extra).abs() * 20.0 < expected_extra,
        "多占的应是首份点名项堆 {expected_extra}（±5%），量到 {extra}"
    );
    assert_eq!(
        baseline.fields["table_digest"],
        control.fields["table_digest"]
    );
}

#[test]
fn merging_device_by_device_stops_the_peak_growing_with_the_device_count_and_keeping_the_duplicates_does_not(
) {
    let peak = |arm: Arm, devices: u32| {
        measure_in_a_process_of_its_own(arm, devices, Fill::Full, 64, SMALL_RING_SLOTS).peak()
    };
    let slope = |arm: Arm| (peak(arm, 4) - peak(arm, 2)) / (2.0 * peak(arm, 1));
    let split_slope = slope(Arm::Split);
    let control_slope = slope(Arm::SplitPc);
    let baseline_slope = slope(Arm::Baseline);
    assert!(split_slope <= 0.10, "split 的 slope {split_slope}");
    assert!(control_slope > 0.50, "split_pc 的 slope {control_slope}");
    assert!(baseline_slope > 0.50, "baseline 的 slope {baseline_slope}");
}
