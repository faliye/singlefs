//! 切片执行器（登记第十二节修订第 10 条：只改调度、不改判据；输出按片的次序，与线程数无关）：线程数、内存上限与按片并行的单测。

use super::*;

fn describe_a_fake_slice(slice: &u64) -> SliceDescription {
    SliceDescription {
        label: format!("fake-{slice}"),
        is_large: false,
        start_rank: 0,
    }
}

/// 六片假片按 `scheduling` 跑：第 k 片睡 60 × (5 − k) 毫秒（前面的片后跑完）、打两行、交回 10k；`stopping_slice` 那一片打完两行停机。
fn fake_slices_on(
    scheduling: SliceScheduling,
    stopping_slice: Option<u64>,
) -> (Vec<String>, Option<String>, Vec<u64>) {
    let slices: Vec<u64> = (0..6).collect();
    let mut values = Vec::new();
    let (lines, stop_line) = lines_and_stop_line_of(|| {
        values = run_slices_in_order(
            &slices,
            &describe_a_fake_slice,
            scheduling,
            &|slice: &u64| {
                std::thread::sleep(Duration::from_millis(60 * (5 - *slice)));
                emit_product_line!("E name=fake slice={slice} line=first");
                emit_product_line!("E name=fake slice={slice} line=second");
                if stopping_slice == Some(*slice) {
                    stop_with("fake_stop", &format!("slice={slice}"));
                }
                *slice * 10
            },
            &|_progress_line: &str| {},
            PROGRESS_INTERVAL,
        );
    });
    (lines, stop_line, values)
}

#[test]
fn slices_on_four_threads_print_the_lines_of_one_thread_in_slice_order_and_stop_at_the_first_stopping_slice(
) {
    let one_thread = SliceScheduling {
        thread_count: 1,
        large_slice_thread_count: 1,
    };
    let four_threads = SliceScheduling {
        thread_count: 4,
        large_slice_thread_count: 1,
    };
    let expected_lines = |slice_count: u64| -> Vec<String> {
        (0..slice_count)
            .flat_map(|slice| {
                [
                    format!("E name=fake slice={slice} line=first"),
                    format!("E name=fake slice={slice} line=second"),
                ]
            })
            .collect()
    };
    for scheduling in [one_thread, four_threads] {
        assert_eq!(
            fake_slices_on(scheduling, None),
            (expected_lines(6), None, vec![0, 10, 20, 30, 40, 50]),
            "{scheduling:?}：六片的行按片的次序整片打出（后开跑的片先跑完也排在后面），值按片的次序"
        );
        assert_eq!(
            fake_slices_on(scheduling, Some(4)),
            (
                expected_lines(5),
                Some("E name=stop reason=fake_stop slice=4".to_owned()),
                Vec::new()
            ),
            "{scheduling:?}：第 4 片停机，打完第 0–4 片的行再交停机行，第 5 片的行不打（单线程跑到同一处打出的就是这些）"
        );
    }
}

#[test]
fn thread_count_reads_the_apparatus_threads_variable_first_then_the_shared_cap_then_the_registered_heavy_thread_default(
) {
    assert_eq!(
        thread_count_from(Some("3"), Some("8")),
        3,
        "E168_THREADS 先读"
    );
    assert_eq!(
        thread_count_from(None, Some("8")),
        8,
        "没设 E168_THREADS 读 SINGLEFS_THREAD_CAP"
    );
    assert_eq!(
        thread_count_from(None, None),
        singlefs_verification::heavy_threads::registered_heavy_threads().get(),
        "都没设取 .claude/heavy-thread-default 的登记值，不取本机核数"
    );
}

#[test]
fn the_memory_cap_reads_the_e168_variable_first_then_the_control_group_limit_then_forty_gibibytes()
{
    assert_eq!(
        memory_cap_given(Some("16G"), Some("8589934592\n")),
        MemoryCap {
            bytes: 16 << 30,
            source: MemoryCapSource::Environment,
        },
        "E168_MEMORY_CAP 先读，写成 16G"
    );
    assert_eq!(
        memory_cap_given(None, Some("8589934592\n")),
        MemoryCap {
            bytes: 8 << 30,
            source: MemoryCapSource::ControlGroup,
        },
        "没设 E168_MEMORY_CAP 读控制组的 memory.max（字节数带换行）"
    );
    let default_cap = MemoryCap {
        bytes: DEFAULT_MEMORY_CAP_BYTES,
        source: MemoryCapSource::Default,
    };
    assert_eq!(
        memory_cap_given(None, Some("max\n")),
        default_cap,
        "控制组写着 max（不限）取 40 GiB"
    );
    assert_eq!(
        memory_cap_given(None, None),
        default_cap,
        "控制组读不到取 40 GiB"
    );
    assert_eq!(DEFAULT_MEMORY_CAP_BYTES, 40 << 30, "用户定的 40 GB");
}

#[test]
#[should_panic(expected = "E168_MEMORY_CAP")]
fn an_unreadable_e168_memory_cap_panics_naming_the_variable() {
    let _cap = memory_cap_given(Some("lots"), None);
}

#[test]
fn large_slices_running_at_once_are_the_memory_cap_less_the_resident_rest_over_the_peak_of_one_large_slice(
) {
    assert_eq!(
        large_slice_threads_within_the_memory_cap(
            RESIDENT_BYTES_BESIDES_THE_LARGE_SLICES + 3 * PEAK_BYTES_OF_ONE_LARGE_SLICE
        ),
        3,
        "扣掉常驻那一份之外恰好放得下三片：同时跑三片"
    );
    assert_eq!(
        large_slice_threads_within_the_memory_cap(
            RESIDENT_BYTES_BESIDES_THE_LARGE_SLICES
                + PEAK_BYTES_OF_ONE_LARGE_SLICE
                + PEAK_BYTES_OF_ONE_LARGE_SLICE / 2
        ),
        1,
        "放得下一片半：往下取整、一片（常驻那一份不扣就成了两片）"
    );
    assert_eq!(
        large_slice_threads_within_the_memory_cap(0),
        1,
        "一片都放不下也开一片（照撞上限办，不悄悄不跑 P4）"
    );
    assert!(
        large_slice_threads_within_the_memory_cap(DEFAULT_MEMORY_CAP_BYTES) > 1,
        "用户定的 40 GB 下大片不再一次一片"
    );
}

#[test]
fn large_slices_run_at_most_their_own_thread_cap_and_leave_a_thread_for_the_rest() {
    let description = |is_large: bool, start_rank: u64| SliceDescription {
        label: String::new(),
        is_large,
        start_rank,
    };
    let descriptions = [
        description(false, 5),
        description(true, 10),
        description(false, 9),
        description(true, 30),
        description(true, 10),
        description(false, 1),
    ];
    let pools_of = |thread_count: usize, large_slice_thread_count: usize| {
        worker_pools(
            &descriptions,
            SliceScheduling {
                thread_count,
                large_slice_thread_count,
            },
        )
    };
    let pool = |positions_in_start_order: Vec<usize>, thread_count: usize| WorkerPool {
        positions_in_start_order,
        thread_count,
    };
    assert_eq!(
        pools_of(8, 1),
        vec![pool(vec![3, 1, 4], 1), pool(vec![2, 0, 5], 3)],
        "大片（P4）上限 1 片时同时只跑 1 片，按删除发布数从大到小开跑，同值按片号"
    );
    assert_eq!(
        pools_of(3, 8),
        vec![pool(vec![3, 1, 4], 2), pool(vec![2, 0, 5], 1)],
        "大片上限比线程数大时，给其余片留一条线程"
    );
    assert_eq!(
        pools_of(1, 1),
        vec![pool(vec![0, 1, 2, 3, 4, 5], 1)],
        "一条线程：一个池、按片的次序（与改并行之前同一个次序）"
    );
}
