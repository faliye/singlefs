use super::*;

#[test]
fn the_kernel_cpu_counter_line_gives_the_busy_ticks() {
    assert_eq!(
        busy_ticks_of_the_kernel_cpu_counter_line("cpu  10 20 30 1000 40 50 60 70 0 0"),
        Some(10 + 20 + 30 + 50 + 60 + 70)
    );
    assert_eq!(
        busy_ticks_of_the_kernel_cpu_counter_line("cpu0 1 2 3 4 5 6 7 8"),
        None
    );
    assert_eq!(busy_ticks_of_the_kernel_cpu_counter_line("cpu 1 2"), None);
}

#[test]
fn the_process_status_line_gives_user_and_system_ticks_even_with_a_parenthesis_in_the_name() {
    let line =
        "4242 (e202 (gc) x) S 1 4242 4242 0 -1 4194304 100 0 0 0 1234 567 0 0 20 0 9 0 100 0 0";
    assert_eq!(ticks_of_the_process_status_line(line), Some(1234 + 567));
    assert_eq!(ticks_of_the_process_status_line("4242 (short) S 1"), None);
}

#[test]
fn the_schedstat_line_gives_the_nanoseconds_on_the_cpu() {
    assert_eq!(
        cpu_of_the_schedstat_line("123456789 1000 7\n"),
        Some(Duration::from_nanos(123_456_789))
    );
    assert_eq!(cpu_of_the_schedstat_line(""), None);
}

#[test]
fn this_process_lists_the_calling_thread_among_its_tasks() {
    let task = task_identifier_of_this_thread().expect("读得到 /proc/thread-self");
    let cpu_by_task = cpu_of_every_task_of_this_process().expect("读得到 /proc/self/task");
    assert!(cpu_by_task.contains_key(&task));
}

/// 挂钟的起点与走过的都读 `CLOCK_MONOTONIC_RAW`（登记第十二节修订第 10 条）：起点夹在它前后两次直接读的读数之间，
/// 走过的夹在两对直接读数给的上下界之间。起点读的若是别的钟（线程 CPU 钟），起点落不进前一对读数之间；走过的若没减起点，大过上界。
#[test]
fn the_start_and_the_elapsed_of_the_wall_clock_are_read_on_the_monotonic_raw_clock() {
    let monotonic_raw_time_before_the_start = monotonic_raw_time_now();
    let start = StartOnTheMonotonicRawClock::now();
    let monotonic_raw_time_after_the_start = monotonic_raw_time_now();
    assert!(
        monotonic_raw_time_before_the_start <= start.monotonic_raw_time_at_the_start
            && start.monotonic_raw_time_at_the_start <= monotonic_raw_time_after_the_start,
        "起点 {start:?} 不在前后两次直接读数 {monotonic_raw_time_before_the_start:?}、{monotonic_raw_time_after_the_start:?} 之间"
    );
    std::thread::sleep(Duration::from_millis(5));
    let monotonic_raw_time_before_the_elapsed = monotonic_raw_time_now();
    let elapsed = start.elapsed();
    let monotonic_raw_time_after_the_elapsed = monotonic_raw_time_now();
    let elapsed_at_least = monotonic_raw_time_before_the_elapsed
        .checked_sub(monotonic_raw_time_after_the_start)
        .expect("CLOCK_MONOTONIC_RAW 单调不减");
    let elapsed_at_most = monotonic_raw_time_after_the_elapsed
        .checked_sub(monotonic_raw_time_before_the_start)
        .expect("CLOCK_MONOTONIC_RAW 单调不减");
    assert!(
        elapsed_at_least <= elapsed && elapsed <= elapsed_at_most,
        "走过的 {elapsed:?} 不在 {elapsed_at_least:?} 到 {elapsed_at_most:?} 之间"
    );
}

/// 包含自检用的几样钟走速差多少（这一条只量钟速、不判钟速；判的只是量的前提）。纯自旋的窗口里读四样：挂钟 `Instant`（`CLOCK_MONOTONIC`，
/// 受 NTP 调频）、装置的挂钟 `StartOnTheMonotonicRawClock`（`CLOCK_MONOTONIC_RAW`，不受调频，登记第十二节修订第 10 条）、
/// 这条线程的 CPU 钟（`CLOCK_THREAD_CPUTIME_ID`，走内核的调度钟，不受调频）、这条线程 schedstat 的第一个字段（同一份调度钟，
/// 读 CPU 钟时内核刚把它记到那一刻，紧跟着读就是同一刻的值）。读序同 GC 调用的窗口：挂钟在外（`Instant` 最外、`CLOCK_MONOTONIC_RAW` 次之）、CPU 钟在里。
/// 每窗打一行「E202 钟速」，末了打一行「E202 钟速汇总」：两两之差的最小、中位、最大，两样挂钟各自短于 CPU 钟的窗数（`--nocapture` 看）。
/// 判的前提：每一窗都读得到；至少一半的窗里 CPU 钟走了挂钟的 0.9 以上（不在 CPU 上的窗太多，读数说明不了钟速）。
/// 后一条不成立（机器忙、这条线程常被换下）时打一行「本次未跑」就返回，不判红、不打汇总（[`reading_of_the_spin_windows`]）。
#[test]
fn the_window_clocks_on_a_spin_loop_print_their_pairwise_differences() {
    const SPIN_WINDOWS: usize = 40;
    const SPIN_OF_ONE_WINDOW: Duration = Duration::from_millis(50);
    let schedstat_of_this_thread = || -> Duration {
        let line = std::fs::read_to_string("/proc/thread-self/schedstat")
            .expect("读得到这条线程的 schedstat");
        cpu_of_the_schedstat_line(&line).expect("schedstat 认得出")
    };
    let signed_nanoseconds =
        |duration: Duration| i128::try_from(duration.as_nanos()).expect("纳秒数装得进 i128");
    let mut wall_minus_thread_cpu: Vec<i128> = Vec::with_capacity(SPIN_WINDOWS);
    let mut wall_minus_thread_cpu_in_millionths: Vec<f64> = Vec::with_capacity(SPIN_WINDOWS);
    let mut thread_cpu_minus_schedstat: Vec<i128> = Vec::with_capacity(SPIN_WINDOWS);
    let mut monotonic_raw_minus_thread_cpu: Vec<i128> = Vec::with_capacity(SPIN_WINDOWS);
    let mut monotonic_raw_minus_thread_cpu_in_millionths: Vec<f64> =
        Vec::with_capacity(SPIN_WINDOWS);
    let mut windows_mostly_on_the_cpu = 0_usize;
    for window in 0..SPIN_WINDOWS {
        let wall_at_the_opening = Instant::now();
        let monotonic_raw_at_the_opening = StartOnTheMonotonicRawClock::now();
        let thread_cpu_at_the_opening = host_cpu_time_of_this_thread();
        let schedstat_at_the_opening = schedstat_of_this_thread();
        while wall_at_the_opening.elapsed() < SPIN_OF_ONE_WINDOW {
            std::hint::spin_loop();
        }
        let thread_cpu_at_the_closing = host_cpu_time_of_this_thread();
        let schedstat_at_the_closing = schedstat_of_this_thread();
        let monotonic_raw_wall = monotonic_raw_at_the_opening.elapsed();
        let wall = wall_at_the_opening.elapsed();
        let thread_cpu = thread_cpu_at_the_closing
            .checked_sub(thread_cpu_at_the_opening)
            .expect("线程 CPU 钟不往回走");
        let schedstat = schedstat_at_the_closing
            .checked_sub(schedstat_at_the_opening)
            .expect("schedstat 不往回走");
        let millionths = (wall.as_secs_f64() - thread_cpu.as_secs_f64()) / wall.as_secs_f64() * 1e6;
        let monotonic_raw_millionths = (monotonic_raw_wall.as_secs_f64()
            - thread_cpu.as_secs_f64())
            / monotonic_raw_wall.as_secs_f64()
            * 1e6;
        eprintln!(
            "E202 钟速 window={window} wall_nanoseconds={} monotonic_raw_nanoseconds={} thread_cpu_nanoseconds={} \
                 schedstat_nanoseconds={} wall_minus_thread_cpu_nanoseconds={} wall_minus_thread_cpu_ppm={millionths:.3} \
                 monotonic_raw_minus_thread_cpu_nanoseconds={} monotonic_raw_minus_thread_cpu_ppm={monotonic_raw_millionths:.3}",
            wall.as_nanos(),
            monotonic_raw_wall.as_nanos(),
            thread_cpu.as_nanos(),
            schedstat.as_nanos(),
            signed_nanoseconds(wall) - signed_nanoseconds(thread_cpu),
            signed_nanoseconds(monotonic_raw_wall) - signed_nanoseconds(thread_cpu),
        );
        if thread_cpu.as_secs_f64() >= 0.9 * wall.as_secs_f64() {
            windows_mostly_on_the_cpu += 1;
        }
        wall_minus_thread_cpu.push(signed_nanoseconds(wall) - signed_nanoseconds(thread_cpu));
        wall_minus_thread_cpu_in_millionths.push(millionths);
        thread_cpu_minus_schedstat
            .push(signed_nanoseconds(thread_cpu) - signed_nanoseconds(schedstat));
        monotonic_raw_minus_thread_cpu
            .push(signed_nanoseconds(monotonic_raw_wall) - signed_nanoseconds(thread_cpu));
        monotonic_raw_minus_thread_cpu_in_millionths.push(monotonic_raw_millionths);
    }
    match reading_of_the_spin_windows(windows_mostly_on_the_cpu, SPIN_WINDOWS) {
        ReadingOfTheSpinWindows::NotRun => {
            eprintln!(
                "E202 钟速 本次未跑：{SPIN_WINDOWS} 窗里只有 {windows_mostly_on_the_cpu} 窗 CPU 钟走了挂钟的 0.9 以上，\
                     不到一半，这条线程常被换下，读数说明不了钟速"
            );
            return;
        }
        ReadingOfTheSpinWindows::Measured => {}
    }
    wall_minus_thread_cpu.sort_unstable();
    wall_minus_thread_cpu_in_millionths.sort_by(f64::total_cmp);
    thread_cpu_minus_schedstat.sort_unstable();
    monotonic_raw_minus_thread_cpu.sort_unstable();
    monotonic_raw_minus_thread_cpu_in_millionths.sort_by(f64::total_cmp);
    let middle = SPIN_WINDOWS / 2;
    eprintln!(
        "E202 钟速汇总 windows={SPIN_WINDOWS} windows_mostly_on_the_cpu={windows_mostly_on_the_cpu} \
             windows_wall_below_thread_cpu={} wall_minus_thread_cpu_nanoseconds_minimum={} median={} maximum={} \
             wall_minus_thread_cpu_ppm_minimum={:.3} median={:.3} maximum={:.3} \
             thread_cpu_minus_schedstat_nanoseconds_minimum={} median={} maximum={} \
             windows_monotonic_raw_below_thread_cpu={} monotonic_raw_minus_thread_cpu_nanoseconds_minimum={} median={} maximum={} \
             monotonic_raw_minus_thread_cpu_ppm_minimum={:.3} median={:.3} maximum={:.3}",
        wall_minus_thread_cpu
            .iter()
            .filter(|difference| **difference < 0)
            .count(),
        wall_minus_thread_cpu[0],
        wall_minus_thread_cpu[middle],
        wall_minus_thread_cpu[SPIN_WINDOWS - 1],
        wall_minus_thread_cpu_in_millionths[0],
        wall_minus_thread_cpu_in_millionths[middle],
        wall_minus_thread_cpu_in_millionths[SPIN_WINDOWS - 1],
        thread_cpu_minus_schedstat[0],
        thread_cpu_minus_schedstat[middle],
        thread_cpu_minus_schedstat[SPIN_WINDOWS - 1],
        monotonic_raw_minus_thread_cpu
            .iter()
            .filter(|difference| **difference < 0)
            .count(),
        monotonic_raw_minus_thread_cpu[0],
        monotonic_raw_minus_thread_cpu[middle],
        monotonic_raw_minus_thread_cpu[SPIN_WINDOWS - 1],
        monotonic_raw_minus_thread_cpu_in_millionths[0],
        monotonic_raw_minus_thread_cpu_in_millionths[middle],
        monotonic_raw_minus_thread_cpu_in_millionths[SPIN_WINDOWS - 1],
    );
}

/// 钟速单测这一次读得出钟速没有。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReadingOfTheSpinWindows {
    /// 至少一半的窗里 CPU 钟走了挂钟的 0.9 以上：照打汇总。
    Measured,
    /// 不到一半：本次未跑，不判红。
    NotRun,
}

/// `windows` 窗里有 `windows_mostly_on_the_cpu` 窗 CPU 钟走了挂钟的 0.9 以上，钟速单测这一次读得出钟速没有。
fn reading_of_the_spin_windows(
    windows_mostly_on_the_cpu: usize,
    windows: usize,
) -> ReadingOfTheSpinWindows {
    if windows_mostly_on_the_cpu * 2 >= windows {
        ReadingOfTheSpinWindows::Measured
    } else {
        ReadingOfTheSpinWindows::NotRun
    }
}

/// 钟速单测的前提不成立（机器忙）是本次未跑，不是红：40 窗里 19 窗在 CPU 上交「本次未跑」，20 窗交「量了」。
#[test]
fn spin_windows_mostly_off_the_cpu_are_not_run_rather_than_failed() {
    assert_eq!(
        reading_of_the_spin_windows(19, 40),
        ReadingOfTheSpinWindows::NotRun
    );
    assert_eq!(
        reading_of_the_spin_windows(20, 40),
        ReadingOfTheSpinWindows::Measured
    );
    assert_eq!(
        reading_of_the_spin_windows(0, 40),
        ReadingOfTheSpinWindows::NotRun
    );
}

/// 作废第 7 条：内核把线程名截到 15 个字节，`comm` 一行带换行。
#[test]
fn the_comm_line_of_the_gpu_opening_thread_matches_on_its_first_fifteen_bytes() {
    assert!(is_the_comm_line_of_the_gpu_opening_thread(
        "singlefs-openin\n"
    ));
    assert!(!is_the_comm_line_of_the_gpu_opening_thread(
        "singlefs-opening-a-gpu\n"
    ));
    assert!(!is_the_comm_line_of_the_gpu_opening_thread(
        "e202_gpu_cpu_gc\n"
    ));
}
