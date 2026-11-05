//! 压测器的进度行与卡住诊断：一个单独的线程（主线程重起服务时会堵上几秒到两分钟，进度照打）。
//!
//! - 进度行：每隔 `--progress-seconds`（缺省 5）秒、以及阶段一变当场，每一路打一行
//!   `PRUNE_SERVER_BENCH_PROGRESS elapsed_seconds=… phase=… route=… completed_blocks=… frames_sent=… frames_answered=… in_flight=…
//!   reconnections=… journal_compactions=… client_activity=… client_activity_seconds=…`；带故障的负载另带代理的计数与此刻服务的进程号。
//!   `client_activity` 是那一路的容错客户端此刻停在哪一步、停了多久：`connecting`、`backing-off`、`writing-frame`、
//!   `waiting-for-responses`、`writing-journal`（追加日志的追加与整理）、`sleeping`、`outside-client`（在压测器自己的代码里）。
//!   阶段（`phase=`）：别的负载是 `steady`；带故障的依次是 `before-disconnect`、`disconnect-mid-frame`（注入了、等恢复）、
//!   `before-server-kill`、`server-restarting`（`kill -9` 了、等新服务就绪）、`server-kill-recovering`、`before-duplicates`、
//!   `duplicate-frames`、`faults-done`；持续秒数到了而且故障调度走完的打 `draining`（等在途的块做完）。
//! - 卡住：`--stall-seconds`（缺省 300：比一帧超时 120 秒加一轮重连上限 120 秒还长，正常的恢复不会被判卡住）秒里
//!   每一路的帧数、块数、重连数，阶段、服务进程号、代理转的帧数与接的连接数都没变，判卡住：
//!   打 `PRUNE_SERVER_BENCH_STALLED …` 一行（阶段、每一路停在哪一步多久），再按线程打
//!   `PRUNE_SERVER_BENCH_THREAD process=bench|server thread=<线程号> name=<线程名> state=<R/S/D…> wchan=<内核里等在哪> syscall=<系统调用与 fd 指向>`
//!   （读 `/proc/<进程号>/task/*`，服务是自己起的子进程，读得到）；然后置停标记、关掉代理此刻的全部连接，让堵在读写里的客户端出来，
//!   这一趟判失败（`result=failed`），不等 task-server 十分钟后判卡住。

use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicU8, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

use crate::fault_proxy::ProxyControl;
use crate::reliable_client::ActivityProbe;

pub const PROGRESS_LINE_PREFIX: &str = "PRUNE_SERVER_BENCH_PROGRESS";
pub const STALLED_LINE_PREFIX: &str = "PRUNE_SERVER_BENCH_STALLED";
pub const THREAD_LINE_PREFIX: &str = "PRUNE_SERVER_BENCH_THREAD";
/// 看进度的线程隔这么久醒一次（看阶段变没变、该不该打进度、卡没卡）。
const WATCH_TICK: Duration = Duration::from_millis(200);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BenchPhase {
    Steady,
    BeforeDisconnect,
    DisconnectMidFrame,
    BeforeServerKill,
    ServerRestarting,
    ServerKillRecovering,
    BeforeDuplicates,
    DuplicateFrames,
    FaultsDone,
}

impl BenchPhase {
    const ALL: [BenchPhase; 9] = [
        BenchPhase::Steady,
        BenchPhase::BeforeDisconnect,
        BenchPhase::DisconnectMidFrame,
        BenchPhase::BeforeServerKill,
        BenchPhase::ServerRestarting,
        BenchPhase::ServerKillRecovering,
        BenchPhase::BeforeDuplicates,
        BenchPhase::DuplicateFrames,
        BenchPhase::FaultsDone,
    ];

    pub fn name(self) -> &'static str {
        match self {
            BenchPhase::Steady => "steady",
            BenchPhase::BeforeDisconnect => "before-disconnect",
            BenchPhase::DisconnectMidFrame => "disconnect-mid-frame",
            BenchPhase::BeforeServerKill => "before-server-kill",
            BenchPhase::ServerRestarting => "server-restarting",
            BenchPhase::ServerKillRecovering => "server-kill-recovering",
            BenchPhase::BeforeDuplicates => "before-duplicates",
            BenchPhase::DuplicateFrames => "duplicate-frames",
            BenchPhase::FaultsDone => "faults-done",
        }
    }

    fn code(self) -> u8 {
        match self {
            BenchPhase::Steady => 0,
            BenchPhase::BeforeDisconnect => 1,
            BenchPhase::DisconnectMidFrame => 2,
            BenchPhase::BeforeServerKill => 3,
            BenchPhase::ServerRestarting => 4,
            BenchPhase::ServerKillRecovering => 5,
            BenchPhase::BeforeDuplicates => 6,
            BenchPhase::DuplicateFrames => 7,
            BenchPhase::FaultsDone => 8,
        }
    }

    /// 这个阶段之后不再注入故障（持续秒数到了就是收尾）。
    fn is_quiet(self) -> bool {
        matches!(self, BenchPhase::Steady | BenchPhase::FaultsDone)
    }
}

/// 故障调度线程写、看进度的线程读的当前阶段。
pub struct PhaseCell {
    code: AtomicU8,
}

impl PhaseCell {
    pub fn new(phase: BenchPhase) -> PhaseCell {
        PhaseCell {
            code: AtomicU8::new(phase.code()),
        }
    }

    pub fn set(&self, phase: BenchPhase) {
        self.code.store(phase.code(), Ordering::SeqCst);
    }

    pub fn get(&self) -> BenchPhase {
        let code = self.code.load(Ordering::SeqCst);
        BenchPhase::ALL
            .into_iter()
            .find(|phase| phase.code() == code)
            .unwrap_or(BenchPhase::Steady)
    }
}

/// 一路驱动线程每转一圈发布一次的计数，与它那个客户端的动作探针。
pub struct RouteProgress {
    pub label: &'static str,
    completed_blocks: AtomicU64,
    sent_frames: AtomicU64,
    answered_frames: AtomicU64,
    in_flight: AtomicU64,
    reconnections: AtomicU64,
    journal_compactions: AtomicU64,
    activity: OnceLock<ActivityProbe>,
}

/// 一路驱动线程一次发布的计数。
#[derive(Clone, Copy, Debug)]
pub struct RouteCounters {
    pub completed_blocks: u64,
    pub sent_frames: u64,
    pub answered_frames: u64,
    pub in_flight: u64,
    pub reconnections: u64,
    pub journal_compactions: u64,
}

impl RouteProgress {
    pub fn new(label: &'static str) -> RouteProgress {
        RouteProgress {
            label,
            completed_blocks: AtomicU64::new(0),
            sent_frames: AtomicU64::new(0),
            answered_frames: AtomicU64::new(0),
            in_flight: AtomicU64::new(0),
            reconnections: AtomicU64::new(0),
            journal_compactions: AtomicU64::new(0),
            activity: OnceLock::new(),
        }
    }

    /// 客户端连上之后挂上它的动作探针（只挂一次）。
    pub fn attach(&self, probe: ActivityProbe) {
        let _ = self.activity.set(probe);
    }

    pub fn publish(&self, counters: RouteCounters) {
        self.completed_blocks
            .store(counters.completed_blocks, Ordering::SeqCst);
        self.sent_frames
            .store(counters.sent_frames, Ordering::SeqCst);
        self.answered_frames
            .store(counters.answered_frames, Ordering::SeqCst);
        self.in_flight.store(counters.in_flight, Ordering::SeqCst);
        self.reconnections
            .store(counters.reconnections, Ordering::SeqCst);
        self.journal_compactions
            .store(counters.journal_compactions, Ordering::SeqCst);
    }

    /// 判卡住比的那几样计数。客户端停在哪一步不比：空等回应时它每几毫秒在「等回应」与「两步之间」换一次，比了就永远判不出卡住。
    fn signature(&self) -> [u64; 4] {
        [
            self.completed_blocks.load(Ordering::SeqCst),
            self.sent_frames.load(Ordering::SeqCst),
            self.answered_frames.load(Ordering::SeqCst),
            self.reconnections.load(Ordering::SeqCst),
        ]
    }

    fn activity_fields(&self) -> String {
        self.activity.get().map_or_else(
            || "client_activity=not-connected client_activity_seconds=0.0".to_string(),
            |probe| {
                let (activity, stayed) = probe.snapshot();
                format!(
                    "client_activity={} client_activity_seconds={:.1}",
                    activity.name(),
                    stayed.as_secs_f64()
                )
            },
        )
    }

    fn fields(&self) -> String {
        format!(
            "route={} completed_blocks={} frames_sent={} frames_answered={} in_flight={} reconnections={} journal_compactions={} {}",
            self.label,
            self.completed_blocks.load(Ordering::SeqCst),
            self.sent_frames.load(Ordering::SeqCst),
            self.answered_frames.load(Ordering::SeqCst),
            self.in_flight.load(Ordering::SeqCst),
            self.reconnections.load(Ordering::SeqCst),
            self.journal_compactions.load(Ordering::SeqCst),
            self.activity_fields()
        )
    }
}

/// 看进度的线程要的全部东西。
pub struct ProgressWatch {
    pub start: Instant,
    pub duration: Duration,
    pub interval: Duration,
    pub stall_timeout: Duration,
    pub routes: Vec<Arc<RouteProgress>>,
    pub phase: Arc<PhaseCell>,
    pub proxy: Option<Arc<ProxyControl>>,
    /// 此刻那一份服务的进程号（重起之后主线程改它）。
    pub server_process: Arc<AtomicU32>,
    /// 判了卡住置真：各路客户端的停标记读它。
    pub stalled: Arc<AtomicBool>,
    /// 判卡住时的说明，进末行之前的失败说明。
    pub stall_description: Arc<Mutex<Option<String>>>,
}

impl ProgressWatch {
    fn phase_now(&self, elapsed: Duration) -> &'static str {
        let phase = self.phase.get();
        if elapsed >= self.duration && phase.is_quiet() {
            "draining"
        } else {
            phase.name()
        }
    }

    fn proxy_fields(&self) -> String {
        self.proxy.as_ref().map_or_else(String::new, |control| {
            format!(
                " proxy_connections={} proxy_frames_forwarded={} proxy_cuts={} proxy_duplicates={} server_process={}",
                control.connections_accepted.load(Ordering::SeqCst),
                control.frames_forwarded.load(Ordering::SeqCst),
                control.cuts_done.load(Ordering::SeqCst),
                control.duplicates_injected.load(Ordering::SeqCst),
                self.server_process.load(Ordering::SeqCst)
            )
        })
    }

    fn print_progress(&self, prefix: &str, elapsed: Duration) {
        let phase = self.phase_now(elapsed);
        let proxy = self.proxy_fields();
        for route in &self.routes {
            println!(
                "{prefix} elapsed_seconds={:.1} phase={phase} {}{proxy}",
                elapsed.as_secs_f64(),
                route.fields()
            );
        }
    }

    fn signature(&self) -> Vec<u64> {
        let mut signature: Vec<u64> = self
            .routes
            .iter()
            .flat_map(|route| route.signature())
            .collect();
        signature.push(u64::from(self.phase.get().code()));
        signature.push(u64::from(self.server_process.load(Ordering::SeqCst)));
        if let Some(control) = &self.proxy {
            signature.push(control.frames_forwarded.load(Ordering::SeqCst));
            signature.push(control.connections_accepted.load(Ordering::SeqCst));
        }
        signature
    }

    fn report_stall(&self, elapsed: Duration) {
        self.print_progress(STALLED_LINE_PREFIX, elapsed);
        for line in thread_dump_lines("bench", std::process::id()) {
            println!("{line}");
        }
        for line in thread_dump_lines("server", self.server_process.load(Ordering::SeqCst)) {
            println!("{line}");
        }
        let stuck: Vec<String> = self
            .routes
            .iter()
            .map(|route| format!("{} {}", route.label, route.activity_fields()))
            .collect();
        let description = format!(
            "卡住：{} 秒里帧数、块数、重连数、阶段、代理转的帧数与接的连接数都没变（阶段 {}；{}）；各线程等在哪见上面的 {THREAD_LINE_PREFIX} 行",
            self.stall_timeout.as_secs(),
            self.phase_now(elapsed),
            stuck.join("；")
        );
        *self
            .stall_description
            .lock()
            .expect("卡住说明的锁只在判卡住与收尾时短暂持有") = Some(description);
        self.stalled.store(true, Ordering::SeqCst);
        if let Some(control) = &self.proxy {
            control.cut_all_connections();
        }
    }

    /// 看进度，直到 `finished` 置真。
    pub fn run(&self, finished: &AtomicBool) {
        let mut last_phase = self.phase_now(Duration::ZERO);
        let mut last_print = Instant::now();
        let mut last_signature = self.signature();
        let mut last_change = Instant::now();
        let mut is_stall_reported = false;
        while !finished.load(Ordering::SeqCst) {
            thread::sleep(WATCH_TICK);
            let now = Instant::now();
            let elapsed = now.duration_since(self.start);
            let phase = self.phase_now(elapsed);
            if phase != last_phase || now.duration_since(last_print) >= self.interval {
                self.print_progress(PROGRESS_LINE_PREFIX, elapsed);
                last_phase = phase;
                last_print = now;
            }
            let signature = self.signature();
            if signature != last_signature {
                last_signature = signature;
                last_change = now;
            } else if !is_stall_reported && now.duration_since(last_change) >= self.stall_timeout {
                self.report_stall(elapsed);
                is_stall_reported = true;
            }
        }
    }
}

/// x86_64 上常见的几个会堵住的系统调用号与名字；别的只打号。
const SYSTEM_CALL_NAMES: [(u64, &str); 22] = [
    (0, "read"),
    (1, "write"),
    (7, "poll"),
    (17, "pread64"),
    (18, "pwrite64"),
    (23, "select"),
    (35, "nanosleep"),
    (42, "connect"),
    (43, "accept"),
    (44, "sendto"),
    (45, "recvfrom"),
    (46, "sendmsg"),
    (47, "recvmsg"),
    (61, "wait4"),
    (74, "fsync"),
    (75, "fdatasync"),
    (202, "futex"),
    (230, "clock_nanosleep"),
    (232, "epoll_wait"),
    (271, "ppoll"),
    (281, "epoll_pwait"),
    (288, "accept4"),
];

/// 头一个参数是 fd 的那几个系统调用（读写、收发、同步）。
const FILE_DESCRIPTOR_SYSTEM_CALLS: [u64; 11] = [0, 1, 17, 18, 42, 44, 45, 46, 47, 74, 75];

/// `/proc/<进程号>/task/<线程号>/syscall` 的头两个字段换成「名字(fd → 指向)」；在跑或不在系统调用里的照原样。
fn describe_system_call(process_identifier: u32, raw: &str) -> String {
    let mut fields = raw.split_whitespace();
    let Some(number_text) = fields.next() else {
        return "unknown".to_string();
    };
    let Ok(number) = number_text.parse::<u64>() else {
        return number_text.replace(' ', "_");
    };
    let name = SYSTEM_CALL_NAMES
        .iter()
        .find(|(known, _)| *known == number)
        .map_or_else(
            || format!("syscall-{number}"),
            |(_, name)| (*name).to_string(),
        );
    if !FILE_DESCRIPTOR_SYSTEM_CALLS.contains(&number) {
        return name;
    }
    let Some(descriptor) = fields
        .next()
        .and_then(|text| u64::from_str_radix(text.trim_start_matches("0x"), 16).ok())
    else {
        return name;
    };
    let target = std::fs::read_link(format!("/proc/{process_identifier}/fd/{descriptor}"))
        .map_or_else(|_| "unknown".to_string(), |path| path.display().to_string());
    format!("{name}(fd={descriptor}->{target})")
}

fn read_trimmed(path: &Path) -> String {
    std::fs::read_to_string(path).map_or_else(
        |error| format!("unreadable:{}", error.kind()),
        |text| text.trim().replace(' ', "_"),
    )
}

/// 线程状态：`/proc/…/stat` 里右括号之后的头一个字段。
fn thread_state(status_path: &Path) -> String {
    std::fs::read_to_string(status_path).map_or_else(
        |_| "unknown".to_string(),
        |status_text| {
            status_text
                .rfind(')')
                .and_then(|position| status_text[position + 1..].split_whitespace().next())
                .unwrap_or("unknown")
                .to_string()
        },
    )
}

/// 一个进程每个线程一行：线程号、名字、状态、内核里等在哪、此刻的系统调用。
pub fn thread_dump_lines(process: &str, process_identifier: u32) -> Vec<String> {
    let task_directory = format!("/proc/{process_identifier}/task");
    let Ok(entries) = std::fs::read_dir(&task_directory) else {
        return vec![format!(
            "{THREAD_LINE_PREFIX} process={process} process_identifier={process_identifier} unreadable={task_directory}"
        )];
    };
    let mut threads: Vec<u64> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            entry
                .file_name()
                .to_str()
                .and_then(|name| name.parse().ok())
        })
        .collect();
    threads.sort_unstable();
    threads
        .into_iter()
        .map(|thread_identifier| {
            let directory = Path::new(&task_directory).join(thread_identifier.to_string());
            let system_call = std::fs::read_to_string(directory.join("syscall")).map_or_else(
                |error| format!("unreadable:{}", error.kind()),
                |raw| describe_system_call(process_identifier, &raw),
            );
            format!(
                "{THREAD_LINE_PREFIX} process={process} process_identifier={process_identifier} thread={thread_identifier} name={} state={} wchan={} syscall={system_call}",
                read_trimmed(&directory.join("comm")),
                thread_state(&directory.join("stat")),
                read_trimmed(&directory.join("wchan")),
            )
        })
        .collect()
}
