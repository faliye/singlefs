use super::*;

/// 线程数：`E165_THREADS` 先读；没设读 `SINGLEFS_THREAD_CAP`；都没设取登记的每台默认重活线程数，不取本机核数。
#[test]
fn thread_count_reads_the_apparatus_threads_variable_first_then_the_heavy_thread_default() {
    assert_eq!(thread_count_of_the_variable_values(Some("3"), Some("5")), 3);
    assert_eq!(thread_count_of_the_variable_values(None, Some("5")), 5);
    assert_eq!(
        thread_count_of_the_variable_values(None, None),
        singlefs_verification::heavy_threads::registered_heavy_threads().get(),
        "都没设取 .claude/heavy-thread-default 的登记值"
    );
}
