//! `one_cell_measurement` 的单测。
use super::super::tests::{measure_in_a_process_of_its_own, small_cell, SMALL_RING_SLOTS};
use super::*;

#[test]
fn measurement_positive_control_lands_in_the_registered_window() {
    // 登记修订三第七节 A8：measurement_pc 的 H 落在 [254 MiB, 260 MiB]；H 是整个进程的数，所以在子进程里量。
    let cell =
        measure_in_a_process_of_its_own(Arm::MeasurementPc, 1, Fill::Full, 64, SMALL_RING_SLOTS);
    assert_eq!(cell.fields["high_water_mark_reset"], "ok");
    let peak_over_start = cell.number("resident_peak_over_start_bytes");
    assert!(
        measurement_peak_is_in_the_registered_window(peak_over_start),
        "测量路径对照的 H {peak_over_start}"
    );
    assert!(measurement_peak_is_in_the_registered_window(266_338_304));
    assert!(measurement_peak_is_in_the_registered_window(272_629_760));
    assert!(!measurement_peak_is_in_the_registered_window(266_338_303));
    assert!(!measurement_peak_is_in_the_registered_window(272_629_761));
}

#[test]
fn the_result_line_carries_every_field_the_report_reads() {
    let outcome = small_cell(Arm::Split, 2, Fill::Half, 3, 256);
    let line = outcome.result_line(structure_sizes());
    let body = line.strip_prefix("E7RESULT ").expect("以 E7RESULT 起头");
    let fields: BTreeMap<&str, &str> = body
        .split_whitespace()
        .filter_map(|word| word.split_once('='))
        .collect();
    for key in [
        "name",
        "arm",
        "devices",
        "fill",
        "named",
        "repeat",
        "ring_slots",
        "filled_slots",
        "reads",
        "reads_expected",
        "records_returned",
        "records_expected",
        "anchors_hold",
        "resident_peak_over_start_bytes",
        "resident_at_each_device_end_bytes",
        "resident_after_return_bytes",
        "vm_rss_at_start_kib",
        "vm_hwm_kib",
        "high_water_mark_reset",
        "scan_seconds",
        "table_digest",
        "cache_entries",
        "cache_entries_expected",
        "cache_value_bytes",
        "size_location_entry",
        "size_named_unit",
        "size_node_pointer",
        "size_journal_record",
    ] {
        assert!(fields.contains_key(key), "结果行缺字段 {key}：{line}");
    }
    assert!(
        !fields.keys().any(|key| key.starts_with("allocator_")),
        "登记修订三删掉了计数分配器的三个字段，结果行不再带：{line}"
    );
    assert_eq!(fields["cache_entries"], "none");
    let cached = small_cell(Arm::ReadStageCache, 2, Fill::Half, 3, 256);
    let cached_line = cached.result_line(structure_sizes());
    assert!(
        cached_line
            .contains(" cache_entries=256 cache_entries_expected=256 cache_value_bytes=1048576 "),
        "读缓存臂 2 盘半满（每盘 128 个非 0 槽）：{cached_line}"
    );
    assert_eq!(fields["name"], "e197");
    assert_eq!(fields["arm"], "split");
    assert_eq!(fields["fill"], "half");
    assert_eq!(fields["records_returned"], "128");
    assert_eq!(fields["reads"], "512");
    assert_eq!(fields["anchors_hold"], "true");
    assert_eq!(fields["high_water_mark_reset"], "ok");
    assert_eq!(
        fields["resident_at_each_device_end_bytes"]
            .split(',')
            .count(),
        2
    );
    for name in [
        "resident_peak_over_start_bytes",
        "resident_after_return_bytes",
        "vm_rss_at_start_kib",
        "vm_hwm_kib",
    ] {
        assert!(
            fields[name].parse::<i64>().is_ok(),
            "{name} 该是整数：{}",
            fields[name]
        );
    }
    assert_eq!(fields["table_digest"].len(), 16);
}
