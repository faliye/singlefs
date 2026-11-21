use super::*;

/// 假钟：每读一次，真时间走 100 ns；交回的读数按 `step_nanoseconds` 向下取整（`step_nanoseconds` = 1 ms 时照一个调度节拍才前进一次的钟）。
fn clock_stepping_every(step_nanoseconds: u64) -> impl FnMut() -> Option<u64> {
    let mut true_nanoseconds = 0_u64;
    move || {
        true_nanoseconds += 100;
        Some(true_nanoseconds / step_nanoseconds * step_nanoseconds)
    }
}

#[test]
fn the_clock_resolution_check_refuses_a_clock_that_advances_once_a_millisecond() {
    let reading =
        clock_advances_while_spinning(clock_stepping_every(1_000_000)).expect("假钟读得到");
    assert_eq!(
        reading,
        ClockResolutionReading {
            reads: 10_000,
            advances: 1,
            spanned_nanoseconds: 1_000_000
        },
        "真时间走到 1 ms 那一读才前进，1 ms 里只前进一次"
    );
    assert!(!is_the_clock_resolution_enough(reading), "{reading:?}");
}

#[test]
fn the_clock_resolution_check_passes_a_clock_that_advances_on_every_read() {
    let reading = clock_advances_while_spinning(clock_stepping_every(1)).expect("假钟读得到");
    assert_eq!(
        reading,
        ClockResolutionReading {
            reads: 10_001,
            advances: 10_000,
            spanned_nanoseconds: 1_000_000
        }
    );
    assert!(is_the_clock_resolution_enough(reading), "{reading:?}");
}

/// 门槛两边各一个假钟：每 10000 ns 前进一次的钟在 1 ms 里前进 100 次（过），每 10102 ns 前进一次的前进 99 次（不过）。
#[test]
fn the_clock_resolution_check_draws_the_line_at_one_hundred_advances() {
    let at_the_line =
        clock_advances_while_spinning(clock_stepping_every(10_000)).expect("假钟读得到");
    assert_eq!(at_the_line.advances, 100, "{at_the_line:?}");
    assert!(
        is_the_clock_resolution_enough(at_the_line),
        "{at_the_line:?}"
    );
    let below_the_line =
        clock_advances_while_spinning(clock_stepping_every(10_102)).expect("假钟读得到");
    assert_eq!(below_the_line.advances, 99, "{below_the_line:?}");
    assert!(
        !is_the_clock_resolution_enough(below_the_line),
        "{below_the_line:?}"
    );
}

#[test]
fn the_clock_resolution_check_refuses_a_stuck_clock_and_an_unreadable_one() {
    let stuck = clock_advances_while_spinning(|| Some(42)).expect("假钟读得到");
    assert_eq!(
        stuck,
        ClockResolutionReading {
            reads: CLOCK_RESOLUTION_CHECK_READS_AT_MOST,
            advances: 0,
            spanned_nanoseconds: 0
        },
        "停着不走的钟读满上限就停"
    );
    assert!(!is_the_clock_resolution_enough(stuck));
    assert_eq!(clock_advances_while_spinning(|| None), None, "读不到不当 0");
}

#[test]
fn the_clock_resolution_check_passes_the_thread_cpu_clock() {
    let reading =
        clock_advances_while_spinning(thread_cpu_nanoseconds).expect("线程 CPU 时钟读得到");
    assert!(is_the_clock_resolution_enough(reading), "{reading:?}");
    let mut emitter = Emitter::new();
    assert!(
        check_the_thread_cpu_clock_resolution(&mut emitter),
        "开跑自检在本线程上过"
    );
}

#[test]
fn thread_cpu_time_advances_while_the_thread_spins() {
    // 按线程 CPU 钟转够 30 ms（或挂钟走满 20 秒）：满载时线程被换下来，按挂钟忙等 30 ms 转不出 15 ms 的 CPU，按 CPU 钟转不受它影响
    let spin = spin_by_the_cpu_clock_until_it_advances_or_the_wall_runs_out(
        30_000_000,
        20_000_000_000,
        thread_cpu_nanoseconds,
        || RawWallClockReading::now().0,
    )
    .expect("线程 CPU 时钟读得到");
    assert!(
        spin.cpu_advanced_nanoseconds >= 15_000_000,
        "按线程 CPU 钟转 30 ms，线程 CPU 时钟至少前进 15 ms：{spin:?}"
    );
}

/// 假钟：CPU 钟只按挂钟一半的速度走（满载时线程有一半时间被换下来）。按 CPU 钟转的要转到 CPU 钟走满 30 ms（挂钟 60 ms）才停；
/// 按挂钟转的挂钟走满 30 ms 就停，CPU 钟只走了 15 ms。
#[test]
fn spinning_by_the_cpu_clock_runs_until_the_cpu_clock_advances_even_at_half_the_wall_speed() {
    let wall = std::cell::Cell::new(0_u64);
    let read_the_wall = || {
        wall.set(wall.get() + 1_000);
        wall.get()
    };
    let read_the_cpu = || Some(wall.get() / 2);
    let spin = spin_by_the_cpu_clock_until_it_advances_or_the_wall_runs_out(
        30_000_000,
        20_000_000_000,
        read_the_cpu,
        read_the_wall,
    )
    .expect("假钟读得到");
    assert!(
        spin.cpu_advanced_nanoseconds >= 30_000_000,
        "CPU 钟要走满 30 ms：{spin:?}"
    );
    assert!(
        spin.wall_elapsed_nanoseconds >= 60_000_000,
        "挂钟跟着走了两倍：{spin:?}"
    );
    let stopped = spin_by_the_cpu_clock_until_it_advances_or_the_wall_runs_out(
        30_000_000,
        5_000_000,
        || Some(0),
        read_the_wall,
    )
    .expect("假钟读得到");
    assert_eq!(
        stopped.cpu_advanced_nanoseconds, 0,
        "钟停着不走时挂钟走满上限就停"
    );
}

/// 睡着的线程不在 CPU 上：线程 CPU 时钟几乎不动（第一行盘臂阳性对照「写里多睡 2 ms 不算 CPU」靠的就是这一点）。
#[test]
fn thread_cpu_time_never_exceeds_the_raw_monotonic_wall_clock_wrapped_around_it() {
    // 登记第十二节 12.4：挂钟读数包着 CPU 读数，忙等 200 ms，CPU 增量不许大过挂钟增量（`Instant` 慢约 7 ppm 时这里会反过来）。
    let started = RawWallClockReading::now();
    let cpu_before = thread_cpu_nanoseconds().expect("线程 CPU 时钟读得到");
    let busy_until = RawWallClockReading::now().0 + 200_000_000;
    while RawWallClockReading::now().0 < busy_until {
        std::hint::spin_loop();
    }
    let cpu_after = thread_cpu_nanoseconds().expect("线程 CPU 时钟读得到");
    let wall_nanoseconds = u64::try_from(started.elapsed().as_nanos()).expect("200 ms 装得进 u64");
    assert!(
        cpu_after - cpu_before <= wall_nanoseconds,
        "CPU {} 纳秒大过包着它的挂钟 {} 纳秒",
        cpu_after - cpu_before,
        wall_nanoseconds
    );
}

#[test]
fn thread_cpu_time_stands_still_while_the_thread_sleeps() {
    let before = thread_cpu_nanoseconds().expect("线程 CPU 时钟读得到");
    std::thread::sleep(Duration::from_millis(20));
    let after = thread_cpu_nanoseconds().expect("线程 CPU 时钟读得到");
    assert!(
        after < before + 2_000_000,
        "睡 20 ms，线程 CPU 时钟前进不到 2 ms：{before} → {after}"
    );
}
