use super::*;

#[test]
fn control_passes_keep_the_page_cache_that_fio_would_otherwise_drop() {
    for job in [FioJob::ControlBufferedReadFirstPass, FioJob::ControlBufferedReadSecondPass] {
        let arguments = job.arguments("/mnt/large");
        assert!(arguments.contains(&"--invalidate=0".to_string()), "{} 不关 invalidate，第二遍读不到缓存", job.label());
        assert_eq!(arguments.len(), 6);
    }
    assert!(!FioJob::ReadRandomFourKibibytes.arguments("/mnt/large").contains(&"--invalidate=0".to_string()));
}

#[test]
fn nearest_rank_percentile_of_one_hundred_and_of_five_samples() {
    let hundred: Vec<u128> = (1..=100).collect();
    assert_eq!(nearest_rank_percentile(&hundred, 50), 50);
    assert_eq!(nearest_rank_percentile(&hundred, 99), 99);
    assert_eq!(nearest_rank_percentile(&hundred, 100), 100);
    let five = [10, 20, 30, 40, 50];
    assert_eq!(nearest_rank_percentile(&five, 50), 30, "秩 ⌈2.5⌉ = 3");
    assert_eq!(nearest_rank_percentile(&five, 99), 50, "秩 ⌈4.95⌉ = 5");
}

#[test]
fn persist_summary_reports_median_percentile_means_and_the_first_operation() {
    let delta = |write_requests, write_bytes, flush_requests| BlockLayerDelta { read_requests: 0, read_bytes: 0, write_requests, write_bytes, flush_requests };
    let measurements = [
        PersistMeasurement { latency_nanoseconds: 30, delta: delta(3, 12288, 2) },
        PersistMeasurement { latency_nanoseconds: 10, delta: delta(1, 4096, 1) },
        PersistMeasurement { latency_nanoseconds: 20, delta: delta(2, 8192, 1) },
    ];
    let summary = summarize_persist(&measurements);
    assert_eq!(summary.operations, 3);
    assert_eq!(summary.median_latency_nanoseconds, 20);
    assert_eq!(summary.percentile_99_latency_nanoseconds, 30);
    assert_eq!(summary.mean_write_requests_thousandths, 2000);
    assert_eq!(summary.mean_write_bytes, 8192);
    assert_eq!(summary.mean_flush_requests_thousandths, 1333);
    assert_eq!(summary.first_latency_nanoseconds, 30);
    assert_eq!(summary.first_write_requests, 3);
    assert_eq!(summary.first_write_bytes, 12288);
    assert_eq!(summary.first_flush_requests, 2);
}

#[test]
fn capacity_comes_from_busybox_file_system_status_output() {
    assert_eq!(parse_file_system_capacity("4128768 3907213 4096\n"), Some((16_911_433_728, 16_003_944_448)));
    assert_eq!(parse_file_system_capacity("4128768 3907213"), None);
}
