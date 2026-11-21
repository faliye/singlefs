use super::*;

// ───────────────────────────── 本线程的 CPU 时间（登记第一节「CPU 时间」，第十二节 12.2 修订） ─────────────────────────────

// 系统调用、`struct timespec` 与钟号都取 libc crate 的声明（与 singlefs-core 依赖的同一份），不在这份文件里手写外部函数声明块：
// 判法单元闭包（scripts/gate/lib_function_units.py）遇到 extern 块拒算，点名这份二进制里测试的变异行就没有复用键（D13（验证路线） 已定项 17）。

/// Linux 的 `CLOCK_THREAD_CPUTIME_ID`（`<linux/time.h>` 里是 3）：本线程在 CPU 上跑过的时间，读的那一刻现算到纳秒（换钟的依据在登记第十二节 12.2）。
pub(super) const CLOCK_IDENTIFIER_OF_THE_THREAD_CPU_TIME: libc::clockid_t =
    libc::CLOCK_THREAD_CPUTIME_ID;

pub(super) const NANOSECONDS_PER_SECOND: u64 = 1_000_000_000;

/// Linux 的 `CLOCK_PROCESS_CPUTIME_ID`（`<linux/time.h>` 里是 2）：本进程全部线程在 CPU 上跑过的时间（含驱动与 wgpu 在本进程里起的线程；
/// 第 3 次重跑登记第一节「c_rt 的进程钟版」，只给第十条 J5 用）。
#[cfg_attr(
    not(feature = "gpu"),
    allow(dead_code, reason = "第 3 次重跑第一段（gpu 特性）才读进程钟")
)]
const CLOCK_IDENTIFIER_OF_THE_PROCESS_CPU_TIME: libc::clockid_t = libc::CLOCK_PROCESS_CPUTIME_ID;

/// 本线程到此刻在 CPU 上跑过的纳秒；系统调用报错、读出负数的交回 `None`，调用方把那一轮作废（登记第十一节「读不到」）。
pub(super) fn thread_cpu_nanoseconds() -> Option<u64> {
    nanoseconds_of_the_cpu_clock(CLOCK_IDENTIFIER_OF_THE_THREAD_CPU_TIME)
}

/// 本进程到此刻在 CPU 上跑过的纳秒（全部线程合计）；读不到同 [`thread_cpu_nanoseconds`]。
#[cfg_attr(
    not(feature = "gpu"),
    allow(dead_code, reason = "第 3 次重跑第一段（gpu 特性）才读进程钟")
)]
pub(super) fn process_cpu_nanoseconds() -> Option<u64> {
    nanoseconds_of_the_cpu_clock(CLOCK_IDENTIFIER_OF_THE_PROCESS_CPU_TIME)
}

/// 读一个 CPU 时钟（线程钟或进程钟）的纳秒；系统调用报错、读出负数的交回 `None`。
#[allow(
    unsafe_code,
    reason = "std 没有线程 CPU 钟与进程 CPU 钟，只能经 libc 调 clock_gettime(2)；已有依赖里只有 libc，没有带安全封装的 rustix 或 nix（锁文件里都没有）"
)]
fn nanoseconds_of_the_cpu_clock(clock_identifier: libc::clockid_t) -> Option<u64> {
    let mut reading = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: `reading` 是这个栈帧里活着的 `libc::timespec`（排布由 libc crate 按目标平台给）；
    // 指针只在这一次调用里用，内核只往这一个结构里写；clock id 不合法时内核交回 −1、不写内存。
    let status = unsafe { libc::clock_gettime(clock_identifier, std::ptr::addr_of_mut!(reading)) };
    if status != 0 {
        return None;
    }
    let whole_seconds = u64::try_from(reading.tv_sec).ok()?;
    let nanoseconds_within_the_second = u64::try_from(reading.tv_nsec).ok()?;
    whole_seconds
        .checked_mul(NANOSECONDS_PER_SECOND)?
        .checked_add(nanoseconds_within_the_second)
}

/// Linux 的 `CLOCK_MONOTONIC_RAW`（`<linux/time.h>` 里是 4）：与 CPU 读数配对的挂钟。不受 NTP 调速，`CLOCK_MONOTONIC`（`Instant`）
/// 比它慢约 7 ppm，长批上线程 CPU 读数会超过 `Instant` 量的挂钟、误触停机条款第 6 条（登记第十二节 12.4）。
const CLOCK_IDENTIFIER_OF_THE_RAW_MONOTONIC_WALL_CLOCK: libc::clockid_t = libc::CLOCK_MONOTONIC_RAW;

/// 与线程 CPU 读数配对的一次挂钟读数（`CLOCK_MONOTONIC_RAW` 的纳秒）。
#[derive(Clone, Copy)]
pub(super) struct RawWallClockReading(pub(super) u64);

impl RawWallClockReading {
    #[allow(
        unsafe_code,
        reason = "std 的 Instant 读 CLOCK_MONOTONIC，受 NTP 调速、比 CLOCK_MONOTONIC_RAW 慢约 7 ppm，长批上会误触停机条款第 6 条（登记第十二节 12.4）；读 CLOCK_MONOTONIC_RAW 只能经 libc 调 clock_gettime(2)，已有依赖没有安全封装"
    )]
    pub(super) fn now() -> Self {
        let mut reading = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        // SAFETY: `reading` 是这个栈帧里活着的 `libc::timespec`（排布由 libc crate 按目标平台给）；
        // 指针只在这一次调用里用，内核只往这一个结构里写；clock id 是 Linux 认得的 CLOCK_MONOTONIC_RAW，读不到时内核交回 −1、不写内存。
        let status = unsafe {
            libc::clock_gettime(
                CLOCK_IDENTIFIER_OF_THE_RAW_MONOTONIC_WALL_CLOCK,
                std::ptr::addr_of_mut!(reading),
            )
        };
        assert_eq!(
            status, 0,
            "CLOCK_MONOTONIC_RAW 在 Linux 上恒可读；读不到说明平台不对"
        );
        let whole_seconds = u64::try_from(reading.tv_sec).expect("单调钟的整秒不是负数");
        let nanoseconds_within_the_second =
            u64::try_from(reading.tv_nsec).expect("单调钟的秒内纳秒不是负数");
        RawWallClockReading(whole_seconds * NANOSECONDS_PER_SECOND + nanoseconds_within_the_second)
    }

    pub(super) fn elapsed(self) -> Duration {
        Duration::from_nanos(Self::now().0.saturating_sub(self.0))
    }
}

/// 在本线程上自旋到线程 CPU 时钟多出 `nanoseconds`（第一行 CPU 臂的阳性对照）；读不到交回 `false`。
pub(super) fn spin_until_the_thread_cpu_advances(nanoseconds: u64) -> bool {
    let Some(start) = thread_cpu_nanoseconds() else {
        return false;
    };
    // 迭代上界：自旋一直占着 CPU，线程 CPU 时钟每读一次都现算到那一刻，跑满 `nanoseconds` 的 CPU 就越过；读不到就停。
    loop {
        match thread_cpu_nanoseconds() {
            Some(now) if now >= start + nanoseconds => return true,
            Some(_) => std::hint::spin_loop(),
            None => return false,
        }
    }
}

/// 按线程 CPU 钟自旋到前进了多少（[`spin_by_the_cpu_clock_until_it_advances_or_the_wall_runs_out`]）：线程钟前进的纳秒与包着它的挂钟走了多少。
#[cfg_attr(not(test), allow(dead_code, reason = "只有单测按线程钟转、看钟走没走"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SpinByTheCpuClock {
    cpu_advanced_nanoseconds: u64,
    wall_elapsed_nanoseconds: u64,
}

/// 在本线程上按 CPU 钟（`read_the_cpu_clock`）自旋，直到它前进 `cpu_nanoseconds`，或挂钟（`read_the_wall_clock`）走满 `wall_limit_nanoseconds`
/// 先到为止；任何一次读不到交回 `None`。满载时线程被换下来，挂钟走了 CPU 钟不走，按 CPU 钟转才转得够（按挂钟转的话换下来那一段不算 CPU）；
/// 挂钟上限只防钟停着不走时转不出来。两个钟都从参数进来，单测换成假钟（CPU 钟只按挂钟一半的速度走）。
#[cfg_attr(not(test), allow(dead_code, reason = "只有单测按线程钟转、看钟走没走"))]
fn spin_by_the_cpu_clock_until_it_advances_or_the_wall_runs_out(
    cpu_nanoseconds: u64,
    wall_limit_nanoseconds: u64,
    mut read_the_cpu_clock: impl FnMut() -> Option<u64>,
    mut read_the_wall_clock: impl FnMut() -> u64,
) -> Option<SpinByTheCpuClock> {
    let wall_start = read_the_wall_clock();
    let cpu_start = read_the_cpu_clock()?;
    // 迭代上界：挂钟单调前进，走满 wall_limit_nanoseconds 必出去；跨圈的状态只有两个起点，不改。
    loop {
        let cpu_now = read_the_cpu_clock()?;
        let wall_now = read_the_wall_clock();
        let spin = SpinByTheCpuClock {
            cpu_advanced_nanoseconds: cpu_now.saturating_sub(cpu_start),
            wall_elapsed_nanoseconds: wall_now.saturating_sub(wall_start),
        };
        if spin.cpu_advanced_nanoseconds >= cpu_nanoseconds
            || spin.wall_elapsed_nanoseconds >= wall_limit_nanoseconds
        {
            return Some(spin);
        }
        std::hint::spin_loop();
    }
}

/// 开跑自检（登记第十二节 12.2，收严）：自旋这么多纳秒（按被检的钟自己算）里，钟至少要前进 [`CLOCK_ADVANCES_WITHIN_THE_SPIN_AT_LEAST`] 次。
const CLOCK_RESOLUTION_CHECK_SPIN_NANOSECONDS: u64 = 1_000_000;
/// 1 ms 里前进不到 100 次的钟分不出一次设备调用（几微秒到几十微秒）的 CPU，停机退 3。
const CLOCK_ADVANCES_WITHIN_THE_SPIN_AT_LEAST: u64 = 100;
/// 自检至多读几次钟：钟停着不走也要停下（停着不走 = 前进 0 次、没自旋满，判不够）。
const CLOCK_RESOLUTION_CHECK_READS_AT_MOST: u64 = 10_000_000;

/// 自检读数：读了几次、前进了几次（读数大过已读到的最大值算一次）、自旋覆盖了钟上的多少纳秒。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ClockResolutionReading {
    reads: u64,
    advances: u64,
    spanned_nanoseconds: u64,
}

/// 连读 `read_the_clock`，读到比第一读多出 [`CLOCK_RESOLUTION_CHECK_SPIN_NANOSECONDS`] 就停（或读满上限），数它前进了几次；
/// 有一次读不到交回 `None`。钟从参数传进来，单测换成按 1 ms 前进的假钟。
fn clock_advances_while_spinning(
    mut read_the_clock: impl FnMut() -> Option<u64>,
) -> Option<ClockResolutionReading> {
    let start = read_the_clock()?;
    let mut largest_seen = start;
    let mut reading = ClockResolutionReading {
        reads: 1,
        advances: 0,
        spanned_nanoseconds: 0,
    };
    // 迭代上界 CLOCK_RESOLUTION_CHECK_READS_AT_MOST；跨轮携带的 largest_seen 只增不减，spanned_nanoseconds = largest_seen − start。
    while reading.reads < CLOCK_RESOLUTION_CHECK_READS_AT_MOST {
        let now = read_the_clock()?;
        reading.reads += 1;
        if now > largest_seen {
            reading.advances += 1;
            largest_seen = now;
        }
        reading.spanned_nanoseconds = largest_seen - start;
        if reading.spanned_nanoseconds >= CLOCK_RESOLUTION_CHECK_SPIN_NANOSECONDS {
            break;
        }
    }
    Some(reading)
}

/// 自检判定：自旋满 1 ms、其间前进够 100 次才算过；读满上限还没自旋满（钟停着）同样不过。
fn is_the_clock_resolution_enough(reading: ClockResolutionReading) -> bool {
    reading.spanned_nanoseconds >= CLOCK_RESOLUTION_CHECK_SPIN_NANOSECONDS
        && reading.advances >= CLOCK_ADVANCES_WITHIN_THE_SPIN_AT_LEAST
}

/// 开跑自检：在调用它的线程上连读线程 CPU 时钟（`wall-clock`、`stage-split` 里就是之后量 GC 的那条线程；`foreground-fsync` 的 GC
/// 在另起的线程上，自检的是同一种钟），打一行 `name=clock_resolution_check`；
/// 不过再打一行 `name=stop_clause`，交回 `false`，调用方停机退 3。
pub(super) fn check_the_thread_cpu_clock_resolution(emitter: &mut Emitter) -> bool {
    let reading = clock_advances_while_spinning(thread_cpu_nanoseconds);
    let holds = reading.is_some_and(is_the_clock_resolution_enough);
    let field = |value_of: fn(ClockResolutionReading) -> u64| {
        reading.map_or_else(
            || "unreadable".to_string(),
            |measured| value_of(measured).to_string(),
        )
    };
    emitter.line(&format!(
        "name=clock_resolution_check clock=thread_cputime clock_identifier={CLOCK_IDENTIFIER_OF_THE_THREAD_CPU_TIME} reads={} advances={} spanned_nanoseconds={} spin_nanoseconds={CLOCK_RESOLUTION_CHECK_SPIN_NANOSECONDS} advances_at_least={CLOCK_ADVANCES_WITHIN_THE_SPIN_AT_LEAST} holds={holds}",
        field(|measured| measured.reads),
        field(|measured| measured.advances),
        field(|measured| measured.spanned_nanoseconds),
    ));
    if !holds {
        emitter.line("name=stop_clause clause=clock_resolution reason=thread_cpu_clock_advanced_fewer_times_than_required_within_the_spin_or_was_unreadable");
    }
    holds
}

/// 自检没过：这一段一个取样点都不量，打判决行、退 3。
pub(super) fn stop_for_the_clock_resolution(emitter: &mut Emitter, segment: &str) -> i32 {
    emitter.line(&format!(
        "name=verdict segment={segment} clock_resolution_holds=false stop_clause_findings=1"
    ));
    emitter.finish();
    EXIT_CODE_STOP_CLAUSE
}

/// 按挂钟忙等 `duration`（第二行阶段计时器的 2 ms 对照：单线程忙等，挂钟就是 CPU 时间）。
pub(super) fn busy_wait_for(duration: Duration) {
    let started = Instant::now();
    while started.elapsed() < duration {
        std::hint::spin_loop();
    }
}

#[cfg(test)]
mod tests;
