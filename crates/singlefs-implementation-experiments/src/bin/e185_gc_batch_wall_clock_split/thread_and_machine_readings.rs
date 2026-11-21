use super::*;

// ───────────────────────────── 线程与机器的读数（第 2 次重跑登记第六节 N3–N10；只报不判） ─────────────────────────────

/// `/proc/<…>/stat` 进程名之后头一栏是第 3 栏（状态）。
const COUNTER_RECORD_FIELD_NUMBER_RIGHT_AFTER_THE_COMMAND_NAME: usize = 3;
/// `proc(5)` 里 `stat` 的栏号（从 1 数）：minflt、majflt、utime、stime。
const COUNTER_RECORD_FIELD_MINOR_FAULTS: usize = 10;
const COUNTER_RECORD_FIELD_MAJOR_FAULTS: usize = 12;
const COUNTER_RECORD_FIELD_USER_TICKS: usize = 14;
const COUNTER_RECORD_FIELD_SYSTEM_TICKS: usize = 15;
/// `proc(5)` 里 `stat` 的第 39 栏 processor：这条线程最近一次在哪个核上跑；读的是自己这条正在跑的线程，就是读的那一刻所在的核。
const COUNTER_RECORD_FIELD_PROCESSOR: usize = 39;
/// `/proc/self/auxv` 每一项是两个 `unsigned long`（类型在前、值在后），本机字节序；这里按 u64 读，靠下面那条编译期断言。
const AUXILIARY_VECTOR_WORD_BYTES: usize = std::mem::size_of::<u64>();
// 辅助向量的一个字是 `unsigned long`；它与 u64 不一样宽，按 u64 切就错位，这里编不过。
const _: () = assert!(std::mem::size_of::<libc::c_ulong>() == AUXILIARY_VECTOR_WORD_BYTES);
/// `/proc/meminfo` 的 kB 是 1024 字节。
const BYTES_PER_KIBIBYTE: u64 = 1024;
/// 机器状态（N10）两次读 `/proc/stat` 之间隔几秒。
const SECONDS_BETWEEN_THE_MACHINE_STATE_READS: u64 = 1;

/// 读一个内核文件再照 `parse` 拆出一个数；文件不在、读不了交回 `None`（读不到不当 0）。
pub(super) fn parsed_value_of_the_file<Value>(
    path: &Path,
    parse: fn(&str) -> Option<Value>,
) -> Option<Value> {
    let text = std::fs::read_to_string(path).ok()?;
    parse(&text)
}

/// `/proc/thread-self/schedstat` 第二个字段：本线程在运行队列里等 CPU 的纳秒（N3）。
fn run_queue_wait_nanoseconds_of_the_scheduler_counters(text: &str) -> Option<u64> {
    text.split_whitespace().nth(1)?.parse().ok()
}

/// `/proc/thread-self/status` 的 `nonvoluntary_ctxt_switches`（N4）。
fn involuntary_switches_of_the_status(text: &str) -> Option<u64> {
    text.lines()
        .find_map(|line| line.strip_prefix("nonvoluntary_ctxt_switches:"))?
        .trim()
        .parse()
        .ok()
}

/// `stat` 进程名之后的各栏：从最后一个 `)` 之后数（进程名里有空格与括号也不错位）；交回的头一项是第 3 栏。
fn counter_record_fields_after_the_command_name(text: &str) -> Option<Vec<&str>> {
    let (_command_name, after_the_command_name) = text.rsplit_once(')')?;
    Some(after_the_command_name.split_whitespace().collect())
}

/// `stat` 第 `field_number` 栏（从 1 数，至少是 3）。
fn counter_record_field(
    fields_after_the_command_name: &[&str],
    field_number: usize,
) -> Option<u64> {
    fields_after_the_command_name
        .get(field_number.checked_sub(COUNTER_RECORD_FIELD_NUMBER_RIGHT_AFTER_THE_COMMAND_NAME)?)?
        .parse()
        .ok()
}

/// 本线程 `stat` 里的四个计数（N5、N6）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ThreadCounters {
    pub(super) minor_faults: u64,
    pub(super) major_faults: u64,
    pub(super) user_ticks: u64,
    pub(super) system_ticks: u64,
}

fn thread_counters_of_the_counter_record(text: &str) -> Option<ThreadCounters> {
    let fields = counter_record_fields_after_the_command_name(text)?;
    Some(ThreadCounters {
        minor_faults: counter_record_field(&fields, COUNTER_RECORD_FIELD_MINOR_FAULTS)?,
        major_faults: counter_record_field(&fields, COUNTER_RECORD_FIELD_MAJOR_FAULTS)?,
        user_ticks: counter_record_field(&fields, COUNTER_RECORD_FIELD_USER_TICKS)?,
        system_ticks: counter_record_field(&fields, COUNTER_RECORD_FIELD_SYSTEM_TICKS)?,
    })
}

/// 本进程 `stat` 的 utime + stime（N10）。
fn process_ticks_of_the_counter_record(text: &str) -> Option<u64> {
    let fields = counter_record_fields_after_the_command_name(text)?;
    counter_record_field(&fields, COUNTER_RECORD_FIELD_USER_TICKS)?.checked_add(
        counter_record_field(&fields, COUNTER_RECORD_FIELD_SYSTEM_TICKS)?,
    )
}

/// `/proc/pressure/{cpu,memory}` 的 `some avg10`（N7）。
fn pressure_some_ten_seconds_of(text: &str) -> Option<f64> {
    text.lines()
        .find_map(|line| line.strip_prefix("some "))?
        .split_whitespace()
        .find_map(|field| field.strip_prefix("avg10="))?
        .parse()
        .ok()
}

/// `/proc/loadavg` 第一个数（N7）。
fn load_average_one_minute_of(text: &str) -> Option<f64> {
    text.split_whitespace().next()?.parse().ok()
}

/// `/proc/meminfo` 的 `MemAvailable`（kB）折成字节（N10）。
fn memory_available_bytes_of(text: &str) -> Option<u64> {
    let kibibytes: u64 = text
        .lines()
        .find_map(|line| line.strip_prefix("MemAvailable:"))?
        .split_whitespace()
        .next()?
        .parse()
        .ok()?;
    kibibytes.checked_mul(BYTES_PER_KIBIBYTE)
}

/// `stat` 第 39 栏（processor）：这条线程最近一次在哪个核上跑。
pub(super) fn cpu_number_of_the_counter_record(text: &str) -> Option<u32> {
    let fields = counter_record_fields_after_the_command_name(text)?;
    let cpu_number = counter_record_field(&fields, COUNTER_RECORD_FIELD_PROCESSOR)?;
    u32::try_from(cpu_number).ok()
}

/// 辅助向量（`/proc/self/auxv` 的原样字节）里 `AT_CLKTCK`（类型 17）那一项的值：内核在 exec 时交给进程的节拍数，
/// `/proc` 下 `stat` 的 utime、stime 按它计。到 `AT_NULL`（类型 0，向量收尾）之前没有这一项、或值是 0，交回 `None`；
/// 末尾凑不满一项的零头不读。
fn clock_ticks_of_the_auxiliary_vector(auxiliary_vector: &[u8]) -> Option<u64> {
    auxiliary_vector
        .as_chunks::<{ 2 * AUXILIARY_VECTOR_WORD_BYTES }>()
        .0
        .iter()
        .map(|entry| {
            let (type_bytes, value_bytes) = entry.split_at(AUXILIARY_VECTOR_WORD_BYTES);
            (
                u64::from_ne_bytes(
                    type_bytes
                        .try_into()
                        .expect("as_chunks 切出的一项是两个字，前一半正好一个字"),
                ),
                u64::from_ne_bytes(
                    value_bytes
                        .try_into()
                        .expect("as_chunks 切出的一项是两个字，后一半正好一个字"),
                ),
            )
        })
        .take_while(|(entry_type, _value)| *entry_type != libc::AT_NULL)
        .find_map(|(entry_type, value)| (entry_type == libc::AT_CLKTCK).then_some(value))
        .filter(|ticks| *ticks > 0)
}

/// `/proc` 计数用的时钟节拍数（USER_HZ）：读 `/proc/self/auxv` 的 `AT_CLKTCK`；读不到交回 `None`（不写死 100，那是替平台作假设）。
pub(super) fn clock_ticks_per_second() -> Option<u64> {
    let auxiliary_vector = std::fs::read("/proc/self/auxv").ok()?;
    clock_ticks_of_the_auxiliary_vector(&auxiliary_vector)
}

/// 一条意图前或后的一组线程与机器读数（N3–N9）；读不到的是 `None`（不当 0）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ThreadAndMachineSnapshot {
    pub(super) run_queue_wait_nanoseconds: Option<u64>,
    pub(super) involuntary_switches: Option<u64>,
    pub(super) thread_counters: Option<ThreadCounters>,
    pub(super) cpu_pressure_some_ten_seconds: Option<f64>,
    pub(super) memory_pressure_some_ten_seconds: Option<f64>,
    pub(super) load_average_one_minute: Option<f64>,
    pub(super) cpu_number: Option<u32>,
    pub(super) seconds_since_the_segment_start: f64,
}

pub(super) fn thread_and_machine_snapshot(segment_started: Instant) -> ThreadAndMachineSnapshot {
    let run_queue_wait_nanoseconds = parsed_value_of_the_file(
        Path::new("/proc/thread-self/schedstat"),
        run_queue_wait_nanoseconds_of_the_scheduler_counters,
    );
    let involuntary_switches = parsed_value_of_the_file(
        Path::new("/proc/thread-self/status"),
        involuntary_switches_of_the_status,
    );
    // 四个计数与核号出自同一次读的 `stat`：一次快照只开一次这个文件。
    let counter_record_of_this_thread = std::fs::read_to_string("/proc/thread-self/stat").ok();
    ThreadAndMachineSnapshot {
        run_queue_wait_nanoseconds,
        involuntary_switches,
        thread_counters: counter_record_of_this_thread
            .as_deref()
            .and_then(thread_counters_of_the_counter_record),
        cpu_pressure_some_ten_seconds: parsed_value_of_the_file(
            Path::new("/proc/pressure/cpu"),
            pressure_some_ten_seconds_of,
        ),
        memory_pressure_some_ten_seconds: parsed_value_of_the_file(
            Path::new("/proc/pressure/memory"),
            pressure_some_ten_seconds_of,
        ),
        load_average_one_minute: parsed_value_of_the_file(
            Path::new("/proc/loadavg"),
            load_average_one_minute_of,
        ),
        cpu_number: counter_record_of_this_thread
            .as_deref()
            .and_then(cpu_number_of_the_counter_record),
        seconds_since_the_segment_start: segment_started.elapsed().as_secs_f64(),
    }
}

/// 后减前；两头有一头读不到、或后比前小（计数回绕）交回 `None`。
pub(super) fn increase_of(before: Option<u64>, after: Option<u64>) -> Option<u64> {
    after?.checked_sub(before?)
}

pub(super) fn optional_count(value: Option<u64>) -> String {
    value.map_or_else(|| "none".to_string(), |count| count.to_string())
}

pub(super) fn optional_with_two_decimals(value: Option<f64>) -> String {
    value.map_or_else(|| "none".to_string(), |number| format!("{number:.2}"))
}

/// `/proc/stat` 首行前八栏（user、nice、system、idle、iowait、irq、softirq、steal）之和是全部，idle 与 iowait 两栏之外是忙。
const CPU_JIFFY_FIELDS_SUMMED: usize = 8;
const CPU_JIFFY_IDLE_FIELD_INDEX: usize = 3;
const CPU_JIFFY_INPUT_OUTPUT_WAIT_FIELD_INDEX: usize = 4;

/// 全机的 CPU 节拍（N10）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CpuJiffies {
    busy: u64,
    total: u64,
}

fn cpu_jiffies_of_the_system_counters(text: &str) -> Option<CpuJiffies> {
    let values: Vec<u64> = text
        .lines()
        .next()?
        .strip_prefix("cpu ")?
        .split_whitespace()
        .take(CPU_JIFFY_FIELDS_SUMMED)
        .map(|field| field.parse().ok())
        .collect::<Option<Vec<u64>>>()?;
    if values.len() < CPU_JIFFY_FIELDS_SUMMED {
        return None;
    }
    let total: u64 = values.iter().sum();
    let idle = values[CPU_JIFFY_IDLE_FIELD_INDEX] + values[CPU_JIFFY_INPUT_OUTPUT_WAIT_FIELD_INDEX];
    Some(CpuJiffies {
        busy: total.checked_sub(idle)?,
        total,
    })
}

/// 隔一秒的两次读数之间：整机忙的比例、本进程占的比例与其余（「其余不归 singlefs」只记这个总量，不打进程名）。
#[derive(Clone, Copy, Debug, PartialEq)]
struct MachineShares {
    machine_busy_share: f64,
    this_process_share: f64,
    others_share: f64,
}

fn machine_shares(
    cpu_before: Option<CpuJiffies>,
    cpu_after: Option<CpuJiffies>,
    process_ticks_before: Option<u64>,
    process_ticks_after: Option<u64>,
) -> Option<MachineShares> {
    let (cpu_before, cpu_after) = (cpu_before?, cpu_after?);
    let total = cpu_after.total.checked_sub(cpu_before.total)?;
    if total == 0 {
        return None;
    }
    let machine_busy_share = cpu_after.busy.checked_sub(cpu_before.busy)? as f64 / total as f64;
    let this_process_share =
        increase_of(process_ticks_before, process_ticks_after)? as f64 / total as f64;
    Some(MachineShares {
        machine_busy_share,
        this_process_share,
        others_share: machine_busy_share - this_process_share,
    })
}

/// 机器状态那一行打在哪一刻（N10）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MachineStatePosition {
    SegmentStart,
    /// 一个取样点准备完、量之前。
    PointStart(SamplingPoint),
    SegmentEnd,
}

impl MachineStatePosition {
    pub(super) fn text(self) -> String {
        match self {
            MachineStatePosition::SegmentStart => "position=segment_start".to_string(),
            MachineStatePosition::PointStart(point) => {
                format!("position=point_start {}", point.label())
            }
            MachineStatePosition::SegmentEnd => "position=segment_end".to_string(),
        }
    }
}

/// N10：读 `/proc/stat` 首行与本进程 `/proc/self/stat`，隔 1 秒再读一次，打一行 `name=machine_state`（连同可用内存与 N7 那三样）。
pub(super) fn emit_the_machine_state(
    emitter: &mut Emitter,
    segment: &str,
    position: MachineStatePosition,
    segment_started: Instant,
) {
    let cpu_before =
        parsed_value_of_the_file(Path::new("/proc/stat"), cpu_jiffies_of_the_system_counters);
    let process_before = parsed_value_of_the_file(
        Path::new("/proc/self/stat"),
        process_ticks_of_the_counter_record,
    );
    std::thread::sleep(Duration::from_secs(SECONDS_BETWEEN_THE_MACHINE_STATE_READS));
    let cpu_after =
        parsed_value_of_the_file(Path::new("/proc/stat"), cpu_jiffies_of_the_system_counters);
    let process_after = parsed_value_of_the_file(
        Path::new("/proc/self/stat"),
        process_ticks_of_the_counter_record,
    );
    let shares = machine_shares(cpu_before, cpu_after, process_before, process_after);
    let snapshot = thread_and_machine_snapshot(segment_started);
    emitter.line(&format!(
        "name=machine_state segment={segment} {} machine_busy_share={} this_process_share={} others_share={} others_note=not_attributed_to_singlefs memory_available_bytes={} cpu_pressure_some_ten_seconds={} memory_pressure_some_ten_seconds={} load_average_one_minute={} seconds_since_the_segment_start={:.3}",
        position.text(),
        optional_number(shares.map(|reading| reading.machine_busy_share)),
        optional_number(shares.map(|reading| reading.this_process_share)),
        optional_number(shares.map(|reading| reading.others_share)),
        optional_count(parsed_value_of_the_file(Path::new("/proc/meminfo"), memory_available_bytes_of)),
        optional_with_two_decimals(snapshot.cpu_pressure_some_ten_seconds),
        optional_with_two_decimals(snapshot.memory_pressure_some_ten_seconds),
        optional_with_two_decimals(snapshot.load_average_one_minute),
        snapshot.seconds_since_the_segment_start,
    ));
}

#[cfg(test)]
mod tests;
