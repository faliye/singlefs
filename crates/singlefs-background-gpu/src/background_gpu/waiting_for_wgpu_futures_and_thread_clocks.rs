//! 等 wgpu 的 future 的最小等法，与这条线程的 CPU 钟、不受 NTP 调频的单调钟。

use super::*;

/// 这条线程到此刻用掉的 CPU 时间（`CLOCK_THREAD_CPUTIME_ID`）：线程睡着、等 GPU 时不走。
///
/// # Panics
/// `clock_gettime` 报错：Linux 上这个钟总在。
#[must_use]
#[allow(unsafe_code, reason = "std 没有线程 CPU 钟，只能调 clock_gettime(2)")]
pub fn host_cpu_time_of_this_thread() -> std::time::Duration {
    let mut thread_cpu_time = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: `thread_cpu_time` 是这个栈帧里一个初始化过的 `timespec`，`clock_gettime` 只往它里面写，调用期间没有别的引用。
    let status =
        unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, &raw mut thread_cpu_time) };
    assert_eq!(
        status, 0,
        "clock_gettime(CLOCK_THREAD_CPUTIME_ID) 在 Linux 上总读得到"
    );
    std::time::Duration::new(
        u64::try_from(thread_cpu_time.tv_sec).expect("CPU 时间的秒数不为负"),
        u32::try_from(thread_cpu_time.tv_nsec).expect("纳秒部分在 0 到 10^9 之间"),
    )
}

/// 本机开机以来的 `CLOCK_MONOTONIC_RAW`：单调不减，不受 NTP 调频。
/// `std::time::Instant` 读的 `CLOCK_MONOTONIC` 跟着 NTP 调频走，线程 CPU 钟（[`host_cpu_time_of_this_thread`]）不跟，
/// 两样成对相减时差出几十 ppm；要拿挂钟与线程 CPU 钟比大小的读这一个。
///
/// # Panics
/// `clock_gettime` 报错：Linux 上这个钟总在。
#[must_use]
#[allow(
    unsafe_code,
    reason = "std 的 Instant 读 CLOCK_MONOTONIC，没有不受 NTP 调频的单调钟，只能调 clock_gettime(2)"
)]
pub fn monotonic_raw_time_now() -> std::time::Duration {
    let mut monotonic_raw_time = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: `monotonic_raw_time` 是这个栈帧里一个初始化过的 `timespec`，`clock_gettime` 只往它里面写，调用期间没有别的引用。
    let status =
        unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC_RAW, &raw mut monotonic_raw_time) };
    assert_eq!(
        status, 0,
        "clock_gettime(CLOCK_MONOTONIC_RAW) 在 Linux 上总读得到"
    );
    std::time::Duration::new(
        u64::try_from(monotonic_raw_time.tv_sec).expect("开机以来的秒数不为负"),
        u32::try_from(monotonic_raw_time.tv_nsec).expect("纳秒部分在 0 到 10^9 之间"),
    )
}

/// 阻塞等一个 wgpu 的 future 交出结果（枚举适配器、起设备），最多等 `wait_limit`（按 `clock` 量）：轮询它，没好就把这条线程挂起至多到期限，
/// 等它的唤醒把线程叫回来再轮询。本机的 wgpu 后端这几个 future 一轮询就好；不依赖 `pollster`。
/// 期限从进这个函数起算：实参求值时做的活（造 future 那一步同步做完的枚举）不在 `wait_limit` 之内。
///
/// # Errors
/// 到期限还没好 ⇒ [`GpuFailureCause::WaitForTheGpuExceededTheLimit`]。
pub fn wait_for_the_wgpu_future<Output>(
    future: impl Future<Output = Output>,
    wait_limit: Duration,
    clock: &impl MonotonicClock,
) -> Result<Output, GpuFailureCause> {
    wait_for_the_wgpu_future_before_the_deadline(
        future,
        &GpuWaitDeadline::starting_now(wait_limit, clock),
        clock,
    )
}

/// 同 [`wait_for_the_wgpu_future`]，期限是交进来的 `deadline`（照 `clock` 量）：期限读调用方在停的标记时，挂起至多一个短等间隔就醒来读一次，
/// 置了就放弃这一等（[`wait_for_completion_before_the_deadline`]）。建一段缓冲时等错误范围那一等走这里、用这张卡等一批的结果的期限。
///
/// # Errors
/// 到期限还没好 ⇒ [`GpuFailureCause::WaitForTheGpuExceededTheLimit`]；期限读标记而调用方在停 ⇒
/// [`GpuFailureCause::GaveUpWaitingForTheCallerIsStopping`]。
pub fn wait_for_the_wgpu_future_before_the_deadline<Output>(
    future: impl Future<Output = Output>,
    deadline: &GpuWaitDeadline,
    clock: &impl MonotonicClock,
) -> Result<Output, GpuFailureCause> {
    let waker = Waker::from(Arc::new(UnparkTheWaitingThread(std::thread::current())));
    let mut context = Context::from_waker(&waker);
    let mut pinned_future = std::pin::pin!(future);
    wait_for_completion_before_the_deadline(deadline, clock, |time_left| {
        match pinned_future.as_mut().poll(&mut context) {
            Poll::Ready(output) => CompletionCheck::Completed(output),
            Poll::Pending => {
                std::thread::park_timeout(time_left);
                CompletionCheck::NotYetComplete
            }
        }
    })
}

struct UnparkTheWaitingThread(std::thread::Thread);

impl Wake for UnparkTheWaitingThread {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
}
