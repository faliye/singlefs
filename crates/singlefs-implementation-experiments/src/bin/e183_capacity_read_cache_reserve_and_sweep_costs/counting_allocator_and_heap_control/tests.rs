use super::*;

/// 全进程的那一份记账计的是整个测试进程：同进程里别的用例在窗里释放内存，「峰值减基线」就少掉一截，下沿断言会随用例数与调度时红时绿。
/// 量窗读的是发起它的那条线程自己的一份（不受并行跑的别的用例影响，直接在测试进程里量）；只读全进程那一份的用例
/// （全进程峰值的取大、PC-堆 要两份都落在区间里）各在一个只跑它自己的子进程里量（开关设了、单线程）：
/// 在测试进程里被调到时起那个子进程并要求它跑过；在子进程里被调到时才真量。
const QUIET_WINDOW_CHILD_SWITCH: &str = "E183_ALLOCATOR_WINDOW_IN_THIS_PROCESS";

/// 当前进程就是只跑一条量窗用例的子进程吗。
fn is_the_quiet_window_child() -> bool {
    std::env::var_os(QUIET_WINDOW_CHILD_SWITCH).is_some()
}

/// 起子进程只跑 `test_name` 这一条（`--exact`、单线程），要求它退 0 且报「1 passed」（名字写错时 `--exact` 什么都不跑也退 0，靠这一句拦住）。
fn assert_the_quiet_window_child_passes(test_name: &str) {
    let output = std::process::Command::new(std::env::current_exe().expect("测试程序的路径"))
        .args(["--exact", test_name, "--nocapture", "--test-threads=1"])
        .env(QUIET_WINDOW_CHILD_SWITCH, "1")
        .output()
        .expect("子进程起得来");
    let printed = String::from_utf8_lossy(&output.stdout);
    let failed_text = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "{test_name} 的子进程退 {:?}：\n{printed}\n{failed_text}",
        output.status
    );
    assert!(
        printed.contains("test result: ok. 1 passed;"),
        "{test_name} 的子进程没有恰好跑过 1 条：\n{printed}"
    );
}

const COUNTING_ALLOCATOR_PEAK_TEST: &str =
    "tests::counting_allocator_peak_stays_at_the_maximum_after_the_memory_is_freed";
const HEAP_POSITIVE_CONTROL_TEST: &str =
    "tests::heap_positive_control_sees_a_sixty_four_mebibyte_buffer_inside_the_registered_window";

#[test]
fn counting_allocator_growth_past_the_top_of_u64_saturates_instead_of_panicking() {
    let allocator = CountingAllocator::new();
    allocator.record_growth(1);
    allocator.record_growth(u64::MAX);
    assert_eq!(
        allocator.peak(),
        u64::MAX,
        "在用 1 字节之后再长 u64::MAX 字节，峰值饱和在 u64::MAX、计数路径不 panic"
    );
}

#[test]
fn counting_allocator_peak_stays_at_the_maximum_after_the_memory_is_freed() {
    if !is_the_quiet_window_child() {
        assert_the_quiet_window_child_passes(COUNTING_ALLOCATOR_PEAK_TEST);
        return;
    }
    let baseline_bytes = COUNTING_ALLOCATOR.in_use();
    COUNTING_ALLOCATOR.reset_peak_to_in_use();
    let big = vec![1u8; 64 << 20];
    std::hint::black_box(&big);
    drop(big);
    let small = vec![2u8; 4096];
    std::hint::black_box(&small);
    let peak_over_baseline_bytes = COUNTING_ALLOCATOR.peak().saturating_sub(baseline_bytes);
    assert!(
        peak_over_baseline_bytes >= 64 << 20,
        "先占 64 MiB 再放掉、再小分配一次之后，峰值仍是出现过的最大值：{peak_over_baseline_bytes}"
    );
}

#[test]
fn heap_positive_control_sees_a_sixty_four_mebibyte_buffer_inside_the_registered_window() {
    if !is_the_quiet_window_child() {
        assert_the_quiet_window_child_passes(HEAP_POSITIVE_CONTROL_TEST);
        return;
    }
    let control = heap_positive_control(&COUNTING_ALLOCATOR, HEAP_POSITIVE_CONTROL_SLACK_BYTES);
    assert!(control.holds, "{control:?}");
    assert!(control.peak_over_baseline_bytes >= HEAP_POSITIVE_CONTROL_BYTES);
}

#[test]
fn heap_positive_control_judges_both_edges_of_the_registered_window() {
    let slack = HEAP_POSITIVE_CONTROL_SLACK_BYTES;
    assert!(!heap_control_holds(HEAP_POSITIVE_CONTROL_BYTES - 1, slack));
    assert!(heap_control_holds(HEAP_POSITIVE_CONTROL_BYTES, slack));
    assert!(heap_control_holds(
        HEAP_POSITIVE_CONTROL_BYTES + slack,
        slack
    ));
    assert!(!heap_control_holds(
        HEAP_POSITIVE_CONTROL_BYTES + slack + 1,
        slack
    ));
}

#[test]
fn heap_positive_control_needs_the_thread_and_the_process_reading_both_inside_the_window() {
    let slack = HEAP_POSITIVE_CONTROL_SLACK_BYTES;
    let inside = HEAP_POSITIVE_CONTROL_BYTES;
    let beyond = HEAP_POSITIVE_CONTROL_BYTES + slack + 1;
    assert!(heap_control_holds_on_both_readings(inside, inside, slack));
    assert!(!heap_control_holds_on_both_readings(inside, beyond, slack));
    assert!(!heap_control_holds_on_both_readings(beyond, inside, slack));
    assert!(!heap_control_holds_on_both_readings(
        inside - 1,
        inside,
        slack
    ));
}
