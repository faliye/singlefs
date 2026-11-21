//! 等 wgpu 的 future 的最小等法与两样钟（这条线程的 CPU 钟、不受 NTP 调频的单调钟）。

use super::*;

/// 没被叫醒过时交 Pending 并当场叫醒自己，再被轮询时交出结果。
struct ReadyAfterWakingItselfOnce {
    polled_before: bool,
}

impl Future for ReadyAfterWakingItselfOnce {
    type Output = u32;

    fn poll(mut self: std::pin::Pin<&mut Self>, context: &mut Context<'_>) -> Poll<u32> {
        if self.polled_before {
            return Poll::Ready(42);
        }
        self.polled_before = true;
        context.waker().wake_by_ref();
        Poll::Pending
    }
}

/// 永远交 Pending、每次都当场叫醒自己（挂起的线程马上回来）。
struct NeverReadyButWakingItself;

impl Future for NeverReadyButWakingItself {
    type Output = u32;

    fn poll(self: std::pin::Pin<&mut Self>, context: &mut Context<'_>) -> Poll<u32> {
        context.waker().wake_by_ref();
        Poll::Pending
    }
}

#[test]
fn waiting_for_a_future_that_is_pending_once_returns_its_output() {
    assert_eq!(
        wait_for_the_wgpu_future(
            ReadyAfterWakingItselfOnce {
                polled_before: false
            },
            Duration::from_secs(60),
            &SteadyClockOfThisProcess::starting_now(),
        ),
        Ok(42)
    );
}

#[test]
fn the_wgpu_future_still_pending_at_the_wait_limit_is_a_timeout_failure() {
    let clock = ClockMovingOneSecondPerReading {
        readings: Cell::new(0),
    };
    assert_eq!(
        wait_for_the_wgpu_future(NeverReadyButWakingItself, Duration::from_secs(3), &clock),
        Err(GpuFailureCause::WaitForTheGpuExceededTheLimit {
            wait_limit: Duration::from_secs(3)
        }),
        "到期限还没好：交回超时出错，不一直等"
    );
}

#[test]
fn the_thread_cpu_clock_does_not_advance_while_the_thread_sleeps() {
    let sleep = std::time::Duration::from_millis(300);
    let cpu_time_before = host_cpu_time_of_this_thread();
    std::thread::sleep(sleep);
    let cpu_time_spent = host_cpu_time_of_this_thread().saturating_sub(cpu_time_before);
    assert!(
        cpu_time_spent < sleep / 3,
        "睡 {sleep:?} 只用掉 {cpu_time_spent:?} 的 CPU：量的是线程 CPU 时间，不是挂钟"
    );
}

#[test]
fn the_thread_cpu_clock_advances_while_the_thread_computes() {
    let cpu_time_before = host_cpu_time_of_this_thread();
    let mut accumulator: u64 = 0;
    let mut rounds: u64 = 0;
    while host_cpu_time_of_this_thread().saturating_sub(cpu_time_before)
        < std::time::Duration::from_millis(20)
    {
        for step in 0..10_000_u64 {
            accumulator = accumulator
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(step);
        }
        rounds += 1;
        assert!(
            rounds < 1_000_000,
            "算了这么多轮线程 CPU 钟还没走 20 ms：钟没在走"
        );
    }
    std::hint::black_box(accumulator);
}

#[test]
fn the_monotonic_raw_clock_never_goes_backward_over_successive_readings() {
    const SUCCESSIVE_READINGS: usize = 100_000;
    let mut previous_reading = monotonic_raw_time_now();
    for reading_number in 0..SUCCESSIVE_READINGS {
        let reading = monotonic_raw_time_now();
        assert!(
                reading >= previous_reading,
                "读数号 {reading_number} 读到 {reading:?}，比它前一个读数 {previous_reading:?} 小：钟往回走了"
            );
        previous_reading = reading;
    }
}

#[test]
fn the_monotonic_raw_clock_keeps_running_while_the_thread_sleeps() {
    let sleep = Duration::from_millis(100);
    let monotonic_raw_time_before = monotonic_raw_time_now();
    std::thread::sleep(sleep);
    let elapsed = monotonic_raw_time_now()
        .checked_sub(monotonic_raw_time_before)
        .expect("CLOCK_MONOTONIC_RAW 单调不减");
    assert!(
        elapsed >= sleep / 2,
        "睡 {sleep:?} 这个钟只走了 {elapsed:?}：读的是挂钟，不是线程 CPU 时间"
    );
}

/// 读序同包含自检：挂钟在外、线程 CPU 钟在里，读钟那几次的开销只落在挂钟里。一窗只自旋 1.5 ms 的线程 CPU：
/// 两样钟走速差几十 ppm 在这么短的窗里只差几十纳秒，小于读钟的开销；按滴答走的粗钟在这么短的窗里常常一格都不走，挂钟就短于 CPU 钟。
#[test]
fn the_monotonic_raw_clock_runs_at_least_as_far_as_the_thread_cpu_clock_over_a_spin() {
    const SPIN_WINDOWS: usize = 40;
    const THREAD_CPU_OF_ONE_WINDOW: Duration = Duration::from_micros(1500);
    let mut windows_with_the_wall_behind_the_thread_cpu: Vec<(Duration, Duration)> = Vec::new();
    for _spin_window in 0..SPIN_WINDOWS {
        let wall_at_the_opening = monotonic_raw_time_now();
        let thread_cpu_at_the_opening = host_cpu_time_of_this_thread();
        let mut thread_cpu_at_the_closing = thread_cpu_at_the_opening;
        while thread_cpu_at_the_closing.saturating_sub(thread_cpu_at_the_opening)
            < THREAD_CPU_OF_ONE_WINDOW
        {
            std::hint::spin_loop();
            thread_cpu_at_the_closing = host_cpu_time_of_this_thread();
        }
        let wall = monotonic_raw_time_now()
            .checked_sub(wall_at_the_opening)
            .expect("CLOCK_MONOTONIC_RAW 单调不减");
        let thread_cpu = thread_cpu_at_the_closing.saturating_sub(thread_cpu_at_the_opening);
        if wall < thread_cpu {
            windows_with_the_wall_behind_the_thread_cpu.push((wall, thread_cpu));
        }
    }
    assert!(
            windows_with_the_wall_behind_the_thread_cpu.is_empty(),
            "这几窗（挂钟，线程 CPU）里挂钟走得比线程 CPU 钟少：{windows_with_the_wall_behind_the_thread_cpu:?}"
        );
}
