//! 钟与机器负载：单调钟、线程与进程的 CPU 钟、`/proc` 里的负载与开卡线程。

use super::*;

// ---- 钟（登记第五节「计时与计数怎么做」）：读不到不等于读到 0 ----
// 装置住 `src/bin/`，它自己是一个 crate 根，门禁 code-source-discipline 的 unsafe-allowances 格不许那里写 unsafe：
// 这条线程的 CPU 钟用产品 GPU 包的 `host_cpu_time_of_this_thread`（`CLOCK_THREAD_CPUTIME_ID`）；别的线程用掉的 CPU 读
// `/proc/self/task/<线程号>/schedstat` 的第一个字段（纳秒，内核记的 `sum_exec_runtime`）；本进程的滴答读 `/proc/self/stat`。
// 每次 GC 调用的挂钟（量 W 与包含自检）与并发跑的时刻读 `StartOnTheMonotonicRawClock`（登记第十二节修订第 10 条）；
// 造池、点名、丙段代价、机器负载与开跑自检那几处只报不判包含关系，照旧读 `Instant`。

/// 挂钟的起点：读产品 GPU 包的 `monotonic_raw_time_now`（`CLOCK_MONOTONIC_RAW`，不受 NTP 调频）。`Instant` 读的 `CLOCK_MONOTONIC`
/// 跟着 NTP 调频走、线程 CPU 钟不跟，长调用上挂钟比 GC 线程 CPU 少走几 ppm，包含自检「挂钟 ≥ GC 线程 CPU」就判不过（登记第十二节修订第 10 条）。
#[derive(Clone, Copy, Debug)]
pub(super) struct StartOnTheMonotonicRawClock {
    monotonic_raw_time_at_the_start: Duration,
}

impl StartOnTheMonotonicRawClock {
    pub(super) fn now() -> StartOnTheMonotonicRawClock {
        StartOnTheMonotonicRawClock {
            monotonic_raw_time_at_the_start: monotonic_raw_time_now(),
        }
    }

    pub(super) fn elapsed(self) -> Duration {
        monotonic_raw_time_now()
            .checked_sub(self.monotonic_raw_time_at_the_start)
            .expect("CLOCK_MONOTONIC_RAW 单调不减：此刻不早于起点")
    }
}

/// 读一样钟或 `/proc` 里的一样计数读不到（登记第十一节作废第 1 条）：哪一样。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ClockReadFailed {
    pub(super) reading: &'static str,
}

impl ClockReadFailed {
    pub(super) fn description(self) -> String {
        format!("{}读不到", self.reading)
    }
}

/// 这条线程在 `work` 里用掉的 CPU；钟往回走交 `None`。
pub(super) fn thread_cpu_spent_in<Outcome>(
    work: impl FnOnce() -> Outcome,
) -> (Outcome, Option<Duration>) {
    let before = host_cpu_time_of_this_thread();
    let outcome = work();
    let after = host_cpu_time_of_this_thread();
    (outcome, after.checked_sub(before))
}

/// 这条线程的线程号（`/proc/thread-self` 指向 `<进程号>/task/<线程号>`）：交给 GC 线程读这条线程用掉的 CPU。
pub(super) fn task_identifier_of_this_thread() -> Result<u64, ClockReadFailed> {
    let failed = ClockReadFailed {
        reading: "/proc/thread-self",
    };
    let target = std::fs::read_link("/proc/thread-self").map_err(|_error| failed)?;
    target
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.parse::<u64>().ok())
        .ok_or(failed)
}

/// `schedstat` 一行的第一个字段：这条线程到此刻在 CPU 上的纳秒数。
fn cpu_of_the_schedstat_line(line: &str) -> Option<Duration> {
    let nanoseconds = line.split_whitespace().next()?.parse::<u64>().ok()?;
    Some(Duration::from_nanos(nanoseconds))
}

/// 本进程此刻每条线程用掉的 CPU（线程号 → CPU）。列出之后、读之前退出的线程不在里面。
pub(super) fn cpu_of_every_task_of_this_process() -> Result<BTreeMap<u64, Duration>, ClockReadFailed>
{
    let failed = ClockReadFailed {
        reading: "/proc/self/task",
    };
    let mut cpu_by_task: BTreeMap<u64, Duration> = BTreeMap::new();
    for task in std::fs::read_dir("/proc/self/task").map_err(|_error| failed)? {
        let task = task.map_err(|_error| failed)?;
        let Some(task_identifier) = task
            .file_name()
            .to_str()
            .and_then(|name| name.parse::<u64>().ok())
        else {
            return Err(failed);
        };
        match std::fs::read_to_string(task.path().join("schedstat")) {
            Ok(line) => {
                cpu_by_task.insert(
                    task_identifier,
                    cpu_of_the_schedstat_line(&line).ok_or(failed)?,
                );
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_error) => return Err(failed),
        }
    }
    Ok(cpu_by_task)
}

/// 开跑自检（登记第十一节、第四节第 3a 步第 2 项）：本线程的 CPU 钟 1 ms 挂钟里读出的不同值有几个。
pub(super) fn advances_of_the_thread_cpu_clock_in_a_millisecond() -> u64 {
    let started = Instant::now();
    let mut last_reading = host_cpu_time_of_this_thread();
    let mut advances = 0_u64;
    while started.elapsed() < Duration::from_millis(1) {
        let reading = host_cpu_time_of_this_thread();
        if reading != last_reading {
            advances += 1;
            last_reading = reading;
        }
    }
    advances
}

/// 内核经 `/proc/stat` 与 `/proc/<进程>/stat` 报给用户空间的滴答，每秒这么多（Linux 的 `USER_HZ`，x86-64 与 aarch64 上固定）。
const KERNEL_TICKS_PER_SECOND_REPORTED_TO_USER_SPACE: u64 = 100;

/// 全机忙时与本进程 CPU 的一次读数（登记第五节「计时与计数」最后一条，量 H），都按滴答。
#[derive(Clone, Copy, Debug)]
pub(super) struct HostLoadReading {
    busy_ticks_of_every_core: u64,
    ticks_of_this_process: u64,
    at: Instant,
}

/// `/proc/stat` 头一行（`cpu  user nice system idle iowait irq softirq steal …`）里的忙时：user + nice + system + irq + softirq + steal。
fn busy_ticks_of_the_kernel_cpu_counter_line(line: &str) -> Option<u64> {
    let mut fields = line.split_whitespace();
    if fields.next() != Some("cpu") {
        return None;
    }
    let counters: Vec<u64> = fields
        .take(8)
        .map(|field| field.parse::<u64>().ok())
        .collect::<Option<Vec<u64>>>()?;
    let [user, nice, system, _idle, _waiting_for_input_output, interrupts, soft_interrupts, stolen] =
        counters.as_slice().try_into().ok()?;
    Some(user + nice + system + interrupts + soft_interrupts + stolen)
}

/// `/proc/self/stat` 那一行里本进程的 utime + stime（第 14、15 个字段；第 2 个字段是括号里的线程名，可以带空格，从最后一个右括号之后数）。
fn ticks_of_the_process_status_line(line: &str) -> Option<u64> {
    let (_before, after_the_name) = line.rsplit_once(')')?;
    let fields: Vec<&str> = after_the_name.split_whitespace().collect();
    let user_ticks = fields.get(11)?.parse::<u64>().ok()?;
    let system_ticks = fields.get(12)?.parse::<u64>().ok()?;
    Some(user_ticks + system_ticks)
}

pub(super) fn host_load_reading() -> Result<HostLoadReading, String> {
    let kernel_cpu_counters =
        std::fs::read_to_string("/proc/stat").map_err(|error| format!("读 /proc/stat：{error}"))?;
    let busy_ticks_of_every_core = kernel_cpu_counters
        .lines()
        .next()
        .and_then(busy_ticks_of_the_kernel_cpu_counter_line)
        .ok_or_else(|| "/proc/stat 头一行认不出".to_string())?;
    let process_status = std::fs::read_to_string("/proc/self/stat")
        .map_err(|error| format!("读 /proc/self/stat：{error}"))?;
    let ticks_of_this_process = ticks_of_the_process_status_line(&process_status)
        .ok_or_else(|| "/proc/self/stat 认不出".to_string())?;
    Ok(HostLoadReading {
        busy_ticks_of_every_core,
        ticks_of_this_process,
        at: Instant::now(),
    })
}

/// 两次读数之间本进程以外平均占了几个核（量 H）：全机忙时减本进程，除以挂钟。
pub(super) fn cores_used_outside_this_process(start: HostLoadReading, end: HostLoadReading) -> f64 {
    let busy_ticks = end
        .busy_ticks_of_every_core
        .saturating_sub(start.busy_ticks_of_every_core);
    let process_ticks = end
        .ticks_of_this_process
        .saturating_sub(start.ticks_of_this_process);
    let outside_seconds = count_as_float(busy_ticks.saturating_sub(process_ticks))
        / count_as_float(KERNEL_TICKS_PER_SECOND_REPORTED_TO_USER_SPACE);
    outside_seconds
        / end
            .at
            .duration_since(start.at)
            .as_secs_f64()
            .max(f64::MIN_POSITIVE)
}

/// 本进程此刻有没有挂载开卡另起的那条线程（登记第十条第 6 行）。线程名在 `/proc/self/task/*/comm` 里截到 15 个字节。
pub(super) fn the_gpu_opening_thread_is_still_running() -> Result<bool, String> {
    let tasks = std::fs::read_dir("/proc/self/task")
        .map_err(|error| format!("读 /proc/self/task：{error}"))?;
    for task in tasks {
        let task = task.map_err(|error| format!("读 /proc/self/task 的一项：{error}"))?;
        // 线程在读的这一刻退出了：读不到它的名字就是它已经不在。
        if let Ok(comm_line) = std::fs::read_to_string(task.path().join("comm")) {
            if is_the_comm_line_of_the_gpu_opening_thread(&comm_line) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// 内核记的线程名截到 15 个字节（`/proc/self/task/*/comm` 一行，末尾带换行）：比的是开卡线程名的前 15 个字节（作废第 7 条）。
fn is_the_comm_line_of_the_gpu_opening_thread(comm_line: &str) -> bool {
    comm_line.trim_end() == &OPENING_GPU_THREAD_NAME[..LINUX_THREAD_NAME_BYTES_AT_MOST]
}

#[cfg(test)]
mod tests;
