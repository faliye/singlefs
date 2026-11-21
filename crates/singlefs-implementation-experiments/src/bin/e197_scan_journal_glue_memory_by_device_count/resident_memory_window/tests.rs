//! `resident_memory_window` 的单测。
use super::super::tests::small_cell;
use super::*;

/// 单测里写 `/proc/self/status` 的样子与 `clear_refs` 的替身，放在系统临时目录里，用完删。
fn sample_status_file(label: &str, text: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("e197-{label}-{}.status", std::process::id()));
    std::fs::write(&path, text).expect("写得了临时目录");
    path
}

const SAMPLE_STATUS_TEXT: &str = "Name:\te197\nVmPeak:\t  999999 kB\nVmHWM:\t    1300 kB\nVmRSS:\t     900 kB\nRssAnon:\t     100 kB\n";

#[test]
fn the_status_fields_are_read_in_kibibytes_and_a_missing_field_or_a_foreign_unit_is_unreadable() {
    assert_eq!(
        kibibytes_of_the_status_field(SAMPLE_STATUS_TEXT, VM_HIGH_WATER_MARK_FIELD),
        Some(1300)
    );
    assert_eq!(
        kibibytes_of_the_status_field(SAMPLE_STATUS_TEXT, VM_RESIDENT_FIELD),
        Some(900)
    );
    assert_eq!(
        kibibytes_of_the_status_field(SAMPLE_STATUS_TEXT, "VmSwap:"),
        None
    );
    assert_eq!(
        kibibytes_of_the_status_field("VmRSS:\t12 MB\n", VM_RESIDENT_FIELD),
        None
    );
    assert_eq!(
        kibibytes_of_the_status_field("VmRSS:\tabc kB\n", VM_RESIDENT_FIELD),
        None
    );
    assert_eq!(
        kibibytes_of_the_status_field("VmRSS:\t12\n", VM_RESIDENT_FIELD),
        None
    );
}

#[test]
fn the_window_turns_kibibytes_into_bytes_for_the_peak_and_the_end_values() {
    let window = ResidentWindow {
        rss_start_kib: 1000,
    };
    let end = ResidentWindowEnd {
        high_water_mark_kib: 1300,
        resident_kib: 900,
    };
    assert_eq!(window.peak_over_the_start_bytes(&end), 300 * 1024);
    assert_eq!(
        window.resident_over_the_start_bytes(end.resident_kib),
        -100 * 1024
    );
}

#[test]
fn the_end_reading_takes_the_high_water_mark_field_and_not_the_resident_field() {
    let path = sample_status_file("end", SAMPLE_STATUS_TEXT);
    let end = ResidentWindowEnd::read_from(path.to_str().expect("路径是 UTF-8"));
    std::fs::remove_file(&path).expect("删得掉临时文件");
    assert_eq!(
        end,
        Some(ResidentWindowEnd {
            high_water_mark_kib: 1300,
            resident_kib: 900
        })
    );
    let without_the_field = sample_status_file("end-missing", "Name:\te197\nVmRSS:\t 900 kB\n");
    let missing = ResidentWindowEnd::read_from(without_the_field.to_str().expect("路径是 UTF-8"));
    std::fs::remove_file(&without_the_field).expect("删得掉临时文件");
    assert_eq!(missing, None);
    assert_eq!(
        ResidentWindowEnd::read_from("/nonexistent-e197/status"),
        None
    );
}

#[test]
fn window_that_cannot_reset_or_read_is_unreadable_and_the_result_line_says_so() {
    let status = sample_status_file("open", SAMPLE_STATUS_TEXT);
    let status_path = status.to_str().expect("路径是 UTF-8");
    let clear_refs = sample_status_file("open-clear-refs", "");
    let clear_refs_path = clear_refs.to_str().expect("路径是 UTF-8");
    assert_eq!(
        ResidentWindow::open_with(clear_refs_path, status_path),
        Some(ResidentWindow { rss_start_kib: 900 })
    );
    assert_eq!(std::fs::read_to_string(&clear_refs).expect("读得回"), "5");
    assert_eq!(
        ResidentWindow::open_with("/nonexistent-e197/clear_refs", status_path),
        None,
        "写不进 clear_refs：不当 0"
    );
    assert_eq!(
        ResidentWindow::open_with(clear_refs_path, "/nonexistent-e197/status"),
        None,
        "读不出 VmRSS：不当 0"
    );
    std::fs::remove_file(&status).expect("删得掉临时文件");
    std::fs::remove_file(&clear_refs).expect("删得掉临时文件");
    let mut outcome = small_cell(Arm::Baseline, 1, Fill::Full, 1, 256);
    let readable_line = outcome.result_line(structure_sizes());
    assert!(
        readable_line.contains(" high_water_mark_reset=ok "),
        "{readable_line}"
    );
    outcome.resident = None;
    let line = outcome.result_line(structure_sizes());
    for field in [
        "resident_peak_over_start_bytes",
        "resident_at_each_device_end_bytes",
        "resident_after_return_bytes",
        "vm_rss_at_start_kib",
        "vm_hwm_kib",
        "high_water_mark_reset",
    ] {
        assert!(
            line.contains(&format!(" {field}=unreadable ")),
            "{field}：{line}"
        );
    }
    let pool = GeneratedPool::new(
        1,
        TEST_IMAGE_DEFAULT_BYTES,
        16,
        production_slot_source(8, 1),
    );
    assert_eq!(
        resident_reading_of(None, ResidentWindowEnd::read_now(), &pool),
        None
    );
    assert_eq!(
        resident_reading_of(Some(ResidentWindow { rss_start_kib: 1 }), None, &pool),
        None
    );
    pool.resident_kib_at_each_device_end.borrow_mut().push(None);
    assert_eq!(
        resident_reading_of(
            Some(ResidentWindow { rss_start_kib: 1 }),
            Some(ResidentWindowEnd {
                high_water_mark_kib: 2,
                resident_kib: 2
            }),
            &pool
        ),
        None,
        "轨迹里有一项读不出：整格 unreadable"
    );
}

#[test]
fn this_environment_lets_the_process_reset_and_read_its_own_high_water_mark() {
    let window = ResidentWindow::open()
        .expect("写得进 /proc/self/clear_refs、读得出 VmRSS（读不出时每一格都按 V11 作废）");
    let end = ResidentWindowEnd::read_now().expect("读得出 VmHWM 与 VmRSS");
    assert!(window.rss_start_kib > 0);
    assert!(end.high_water_mark_kib >= end.resident_kib);
}
