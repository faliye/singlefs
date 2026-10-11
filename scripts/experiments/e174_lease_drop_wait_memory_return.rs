//! E174（借据丢掉时等待与显存回还）装置。跑前登记：research/prompts/e174-preregistration.md。
//!
//! 独立手写，不与 `crates/` 共用代码（`.claude/rules/implementation-first.md` 第 4 条）：后端只用 Vulkan，
//! `required_limits` 取 `adapter.limits()`，`memory_hints` 取 `Performance`，读数用 `nvidia-smi -q -d MEMORY,PIDS`
//! 按进程号把 `Used GPU Memory` 加起来。这些设置抄自实现的取法，开跑前由执行员用登记第十一节那 7 条 grep 回比。
//!
//! 结构：借据的整套步骤（登记第五节「一笔借据在装置里怎么做」）写在 `run_one_lease`，对着 `CardHost` 这个 trait 写，
//! 有两份实现：`WgpuCardHost`（真卡）与 `SimulatedCardHost`（模型，单测与 `checks` 子命令用，不起卡）。
//! 登记第九节的变异都改在对着 trait 的那一份代码上，所以不起卡也能由单测抓到。
//!
//! 子命令：`run`（父进程：按清单起子进程、续跑；`--segment first|second --output 产物 [--skip-light] [--limit-jobs N] [--simulated]`）、
//! `child`（一个装置进程，由 `run` 起）、`judge`（读产物判，确定性：`--input 产物 [--input …] [--scope 名]`）、
//! `checks`（不起卡的自检，确定性）、`plan`（清单各阶段的格数与子进程数，确定性）、`simulate`（在模型卡上跑某一格几轮，试 `judge` 用）。
//! `run --simulated` 让每个子进程都跑在模型卡上，整条父子流水线（续跑、补轮、原文压缩、F1 停机）不起卡也走得通。

use std::collections::BTreeMap;

// ============================== 常量（登记里的数，各只一处） ==============================

const MEBIBYTE: u64 = 1 << 20;
const GIBIBYTE: u64 = 1 << 30;
/// 登记第五节：S 大于 1 GiB 时拆成每张 1 GiB 的几张（NVIDIA Vulkan 存储缓冲绑定上限是 2 GiB − 1）。
const MAXIMUM_PIECE_BYTES: u64 = GIBIBYTE;
/// 登记第七节 B7。
const EXPECTED_DRIVER_VERSION: &str = "595.84";
const PROCESS_REPORT_ARGUMENTS: [&str; 3] = ["-q", "-d", "MEMORY,PIDS"];
const COMPUTE_APPLICATIONS_ARGUMENTS: [&str; 2] =
    ["--query-compute-apps=gpu_bus_id,pid,used_memory", "--format=csv,noheader,nounits"];
const LEASE_SIZES_MEBIBYTES: [u64; 5] = [1, 64, 512, 2048, 8192];
const WAIT_TIMEOUT_SECONDS: u64 = 30;
const SHORT_WINDOW_MICROSECONDS: u64 = 2_000_000;
const LONG_WINDOW_MICROSECONDS: u64 = 30_000_000;
const PASSES_PER_PASS_BY_PASS_LEASE: u32 = 8;
const LEASES_PER_PROCESS: u32 = 3;
const READS_PER_BASELINE: usize = 3;
/// 登记第六节：U_pre 或 U_dev 那三份的极差超过它，那一笔借据（U_dev 的话那一个子进程）作废。
const MAXIMUM_BASELINE_SPREAD_MEBIBYTES: u64 = 16;
/// 登记第六节：τ = 三份极差 + 2 MiB，2 MiB 是两份各自取整到 MiB 的读数相减最多差出不到 2 MiB。
const READING_ROUNDING_ALLOWANCE_MEBIBYTES: u64 = 2;
const ROUNDS_REQUIRED_PER_CELL: u32 = 5;
const MAXIMUM_REFILL_ROUNDS_PER_CELL: u32 = 3;
const COEXISTING_BUFFER_MEBIBYTES: u64 = 64;
const SHUFFLE_SEED: u64 = 174;
const CALIBRATION_BAND_LOWER_PERCENT: u64 = 80;
const CALIBRATION_BAND_UPPER_PERCENT: u64 = 125;
const CALIBRATION_CONSECUTIVE_HITS: usize = 3;
const CALIBRATION_ATTEMPT_LIMIT: usize = 40;
/// 登记第九节 K7、K9 的 0.8。
const COVERED_WORK_PERCENT: u64 = 80;
/// 登记第六节 Q1、Q1b、Q2、Q3 的门槛（微秒）。
const DONE_WAIT_ZERO_OVERHEAD_LIMIT_MICROSECONDS: u64 = 5_000;
const DONE_WAIT_EXPENSIVE_LIMIT_MICROSECONDS: u64 = 50_000;
const IN_FLIGHT_EXCESS_LIMIT_MICROSECONDS: u64 = 20_000;
const POLL_NO_WAIT_LIMIT_MICROSECONDS: u64 = 1_000;
const BACKGROUND_WORK_TARGET_MICROSECONDS: u64 = 400_000;
const SPIN_KERNEL_BINDING_BYTES: u64 = 256;
const HOLD_CONTROL_SIZES_MEBIBYTES: [u64; 2] = [512, 2048];
const COEXISTING_SIZES_MEBIBYTES: [u64; 2] = [1, 64];
const PASS_BY_PASS_SIZES_MEBIBYTES: [u64; 2] = [1, 64];
const LONG_WINDOW_SIZES_MEBIBYTES: [u64; 2] = [64, 2048];
const SHARED_DEVICE_SIZES_MEBIBYTES: [u64; 2] = [64, 2048];
/// 子进程非 0 退出时，把它 stderr 的末尾这么多个字符原样记进 `child_failed` 行（登记 V8：交回原文）。
const STDERR_TAIL_CHARACTERS: usize = 1500;

// ============================== 取值范围里的枚举 ==============================

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Card {
    Rtx5090,
    Rtx5060Ti,
}

impl Card {
    const EVERY_CARD: [Card; 2] = [Card::Rtx5090, Card::Rtx5060Ti];

    fn label(self) -> &'static str {
        match self {
            Card::Rtx5090 => "rtx5090",
            Card::Rtx5060Ti => "rtx5060ti",
        }
    }

    fn from_label(label: &str) -> Option<Card> {
        Card::EVERY_CARD.into_iter().find(|card| card.label() == label)
    }

    /// 登记第七节 B1：`nvidia-smi` 报的 PCI 总线号（去掉域）。
    fn bus_without_domain(self) -> &'static str {
        match self {
            Card::Rtx5090 => "01:00.0",
            Card::Rtx5060Ti => "10:00.0",
        }
    }

    /// 登记第七节 B1：FB 那一小节报的 Total（MiB）。
    fn total_mebibytes(self) -> u64 {
        match self {
            Card::Rtx5090 => 32607,
            Card::Rtx5060Ti => 16311,
        }
    }
}

/// 活的档。`InFlightMedium` 是跑前登记「十二、修订」补的一条臂：`InFlightLight` 的 K9 在登记自己的数下必红。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Liveness {
    Done,
    InFlightLight,
    InFlightMedium,
    InFlightHeavy,
}

impl Liveness {
    const EVERY_LIVENESS: [Liveness; 4] =
        [Liveness::Done, Liveness::InFlightLight, Liveness::InFlightMedium, Liveness::InFlightHeavy];

    fn label(self) -> &'static str {
        match self {
            Liveness::Done => "done",
            Liveness::InFlightLight => "light",
            Liveness::InFlightMedium => "medium",
            Liveness::InFlightHeavy => "heavy",
        }
    }

    fn from_label(label: &str) -> Option<Liveness> {
        Liveness::EVERY_LIVENESS.into_iter().find(|liveness| liveness.label() == label)
    }

    fn is_in_flight(self) -> bool {
        match self {
            Liveness::Done => false,
            Liveness::InFlightLight | Liveness::InFlightMedium | Liveness::InFlightHeavy => true,
        }
    }

    /// 空转目标时长（微秒）。登记：已做完与在飞·轻 50 ms，在飞·重 400 ms；在飞·中 250 ms 见修订。
    fn work_target_microseconds(self) -> u64 {
        match self {
            Liveness::Done | Liveness::InFlightLight => 50_000,
            Liveness::InFlightMedium => 250_000,
            Liveness::InFlightHeavy => 400_000,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Method {
    OneBuild,
    PassByPass,
}

impl Method {
    fn label(self) -> &'static str {
        match self {
            Method::OneBuild => "one_build",
            Method::PassByPass => "pass_by_pass",
        }
    }

    fn from_label(label: &str) -> Option<Method> {
        [Method::OneBuild, Method::PassByPass].into_iter().find(|method| method.label() == label)
    }
}

/// 借据丢掉时那一次 `device.poll` 的样子（登记第五节「臂」；第二段多一条「等本借据最后一次提交」）。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Arm {
    /// `Wait { submission_index: None, timeout: Some(30 秒) }`
    WaitLatest,
    /// `Wait { submission_index: Some(本借据最后一次提交), timeout: Some(30 秒) }`，只在第二段。
    WaitOwnSubmission,
    /// `Poll`，今天 `crates/` 的做法。
    Poll,
}

impl Arm {
    fn label(self) -> &'static str {
        match self {
            Arm::WaitLatest => "wait",
            Arm::WaitOwnSubmission => "wait_own",
            Arm::Poll => "poll",
        }
    }

    fn from_label(label: &str) -> Option<Arm> {
        [Arm::WaitLatest, Arm::WaitOwnSubmission, Arm::Poll].into_iter().find(|arm| arm.label() == label)
    }

    fn waits(self) -> bool {
        match self {
            Arm::WaitLatest | Arm::WaitOwnSubmission => true,
            Arm::Poll => false,
        }
    }
}

/// 登记第五节「格子」那张表的组。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Group {
    HoldControl,
    MainGrid,
    PassByPassGrid,
    CoexistingLongLease,
    LongWindow,
    SharedDevice,
}

impl Group {
    fn label(self) -> &'static str {
        match self {
            Group::HoldControl => "hold_control",
            Group::MainGrid => "main",
            Group::PassByPassGrid => "pass_by_pass",
            Group::CoexistingLongLease => "coexisting_long_lease",
            Group::LongWindow => "long_window",
            Group::SharedDevice => "shared_device",
        }
    }

    fn from_label(label: &str) -> Option<Group> {
        [
            Group::HoldControl,
            Group::MainGrid,
            Group::PassByPassGrid,
            Group::CoexistingLongLease,
            Group::LongWindow,
            Group::SharedDevice,
        ]
        .into_iter()
        .find(|group| group.label() == label)
    }
}

/// 一格：卡 × S × 活 × 借法 × 臂（再加所属的组）。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct CellKey {
    group: Group,
    card: Card,
    size_mebibytes: u64,
    liveness: Liveness,
    method: Method,
    arm: Arm,
}

impl CellKey {
    fn fields_text(&self) -> String {
        format!(
            "group={} card={} size_mib={} liveness={} method={} arm={}",
            self.group.label(),
            self.card.label(),
            self.size_mebibytes,
            self.liveness.label(),
            self.method.label(),
            self.arm.label()
        )
    }

    fn from_fields(fields: &BTreeMap<String, String>) -> Result<CellKey, String> {
        let field = |name: &str| fields.get(name).ok_or_else(|| format!("缺字段 {name}"));
        let group = Group::from_label(field("group")?).ok_or("group 认不出")?;
        let card = Card::from_label(field("card")?).ok_or("card 认不出")?;
        let size_mebibytes: u64 = field("size_mib")?.parse().map_err(|_| "size_mib 不是整数".to_string())?;
        let liveness = Liveness::from_label(field("liveness")?).ok_or("liveness 认不出")?;
        let method = Method::from_label(field("method")?).ok_or("method 认不出")?;
        let arm = Arm::from_label(field("arm")?).ok_or("arm 认不出")?;
        Ok(CellKey { group, card, size_mebibytes, liveness, method, arm })
    }

    fn size_bytes(&self) -> u64 {
        self.size_mebibytes * MEBIBYTE
    }

    fn is_hold_control(&self) -> bool {
        self.group == Group::HoldControl
    }

    fn coexisting_mebibytes(&self) -> u64 {
        match self.group {
            Group::CoexistingLongLease => COEXISTING_BUFFER_MEBIBYTES,
            Group::HoldControl | Group::MainGrid | Group::PassByPassGrid | Group::LongWindow | Group::SharedDevice => 0,
        }
    }

    fn has_background_load(&self) -> bool {
        self.group == Group::SharedDevice
    }

    fn lease_count(&self) -> u32 {
        match self.group {
            Group::LongWindow => 1,
            Group::HoldControl | Group::MainGrid | Group::PassByPassGrid | Group::CoexistingLongLease | Group::SharedDevice => {
                LEASES_PER_PROCESS
            }
        }
    }

    fn window_microseconds(&self) -> u64 {
        match self.group {
            Group::LongWindow => LONG_WINDOW_MICROSECONDS,
            Group::HoldControl | Group::MainGrid | Group::PassByPassGrid | Group::CoexistingLongLease | Group::SharedDevice => {
                SHORT_WINDOW_MICROSECONDS
            }
        }
    }
}

/// 一轮 = 一个装置进程 = 一格的一个轮号。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct ChildJob {
    cell: CellKey,
    round: u32,
}

impl ChildJob {
    fn child_arguments(&self) -> Vec<String> {
        let cell = &self.cell;
        vec![
            "child".to_string(),
            "--group".to_string(),
            cell.group.label().to_string(),
            "--card".to_string(),
            cell.card.label().to_string(),
            "--size-mib".to_string(),
            cell.size_mebibytes.to_string(),
            "--liveness".to_string(),
            cell.liveness.label().to_string(),
            "--method".to_string(),
            cell.method.label().to_string(),
            "--arm".to_string(),
            cell.arm.label().to_string(),
            "--round".to_string(),
            self.round.to_string(),
        ]
    }

    fn from_child_arguments(arguments: &[String]) -> Result<ChildJob, String> {
        let mut fields: BTreeMap<String, String> = BTreeMap::new();
        let mut round: Option<u32> = None;
        let mut index = 0;
        while index < arguments.len() {
            let flag = arguments[index].as_str();
            let value = arguments.get(index + 1).ok_or_else(|| format!("{flag} 后面没有值"))?;
            match flag {
                "--group" => fields.insert("group".to_string(), value.clone()),
                "--card" => fields.insert("card".to_string(), value.clone()),
                "--size-mib" => fields.insert("size_mib".to_string(), value.clone()),
                "--liveness" => fields.insert("liveness".to_string(), value.clone()),
                "--method" => fields.insert("method".to_string(), value.clone()),
                "--arm" => fields.insert("arm".to_string(), value.clone()),
                "--round" => {
                    round = Some(value.parse().map_err(|_| "--round 不是整数".to_string())?);
                    None
                }
                other => return Err(format!("认不出的参数 {other}")),
            };
            index += 2;
        }
        let cell = CellKey::from_fields(&fields)?;
        Ok(ChildJob { cell, round: round.ok_or("缺 --round")? })
    }

    fn key_text(&self) -> String {
        format!("{} round={}", self.cell.fields_text(), self.round)
    }
}

/// 一行 `E7RESULT name=… k=v k=v` 拆成字段表；值里没有空格。
fn parse_fields(line: &str) -> BTreeMap<String, String> {
    line.split_whitespace()
        .filter_map(|token| token.split_once('='))
        .map(|(name, value)| (name.to_string(), value.to_string()))
        .collect()
}

// ============================== 读数：解析 nvidia-smi 的两种出口 ==============================

/// `00000000:01:00.0`、`0000:01:00.0` 与 `01:00.0` 认成同一张卡：只留最后两段、小写。
fn bus_without_domain(text: &str) -> String {
    let lowered = text.trim().to_ascii_lowercase();
    let parts: Vec<&str> = lowered.split(':').collect();
    parts[parts.len().saturating_sub(2)..].join(":")
}

fn mebibytes_of_value(value: &str) -> Option<u64> {
    value.strip_suffix("MiB")?.trim().parse().ok()
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ReportSection {
    FrameBufferMemory,
    BarOneMemory,
    Processes,
    Other,
}

/// 一份 `nvidia-smi -q -d MEMORY,PIDS` 报告里一张卡那一段读到的。
#[derive(Debug, Default, PartialEq, Eq)]
struct CardReading {
    total_mebibytes: Option<u64>,
    reserved_mebibytes: Option<u64>,
    free_mebibytes: Option<u64>,
    /// 进程号等于装置进程号的各行 `Used GPU Memory` 之和；没列这个进程是 `None`（读不到，不当 0）。
    process_used_mebibytes: Option<u64>,
    /// 列了这个进程而有一行读不出。
    process_used_is_unreadable: bool,
    listed_process_identifiers: Vec<u32>,
}

fn parse_card_reading(report: &str, bus: &str, process_identifier: u32) -> Result<CardReading, String> {
    let mut reading = CardReading::default();
    let mut card_found = false;
    let mut inside_card = false;
    let mut section = ReportSection::Other;
    let mut listed_process_is_this_process = false;
    for line in report.lines() {
        if let Some(bus_text) = line.strip_prefix("GPU ") {
            inside_card = bus_without_domain(bus_text) == bus;
            card_found |= inside_card;
            section = ReportSection::Other;
            listed_process_is_this_process = false;
            continue;
        }
        if !inside_card {
            continue;
        }
        let indentation = line.len() - line.trim_start().len();
        let (key, value) = match line.split_once(':') {
            Some((key, value)) => (key.trim(), value.trim()),
            None => (line.trim(), ""),
        };
        if indentation <= 4 {
            section = match key {
                "FB Memory Usage" => ReportSection::FrameBufferMemory,
                "BAR1 Memory Usage" => ReportSection::BarOneMemory,
                "Processes" => ReportSection::Processes,
                _ => ReportSection::Other,
            };
            continue;
        }
        match (section, key) {
            (ReportSection::FrameBufferMemory, "Total") => reading.total_mebibytes = mebibytes_of_value(value),
            (ReportSection::FrameBufferMemory, "Reserved") => reading.reserved_mebibytes = mebibytes_of_value(value),
            (ReportSection::FrameBufferMemory, "Free") => reading.free_mebibytes = mebibytes_of_value(value),
            (ReportSection::Processes, "Process ID") => {
                let listed: u32 = value.parse().map_err(|_| format!("进程号认不出：{line:?}"))?;
                reading.listed_process_identifiers.push(listed);
                listed_process_is_this_process = listed == process_identifier;
            }
            (ReportSection::Processes, "Used GPU Memory") if listed_process_is_this_process => {
                match mebibytes_of_value(value) {
                    Some(used) => reading.process_used_mebibytes = Some(reading.process_used_mebibytes.unwrap_or(0) + used),
                    None => reading.process_used_is_unreadable = true,
                }
            }
            (
                ReportSection::FrameBufferMemory
                | ReportSection::BarOneMemory
                | ReportSection::Processes
                | ReportSection::Other,
                _,
            ) => {}
        }
    }
    if !card_found {
        return Err(format!("报告里没有总线 {bus} 那张卡"));
    }
    Ok(reading)
}

/// `--query-compute-apps=gpu_bus_id,pid,used_memory --format=csv,noheader,nounits` 的输出里，
/// 这张卡、这个进程号各行的 `used_memory` 之和；没有这个进程的行是 `None`。
fn parse_compute_applications(text: &str, bus: &str, process_identifier: u32) -> Result<Option<u64>, String> {
    let mut total: Option<u64> = None;
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let columns: Vec<&str> = line.split(',').map(str::trim).collect();
        if columns.len() != 3 {
            return Err(format!("compute-apps 一行不是三列：{line:?}"));
        }
        if bus_without_domain(columns[0]) != bus {
            continue;
        }
        let listed: u32 = columns[1].parse().map_err(|_| format!("compute-apps 进程号认不出：{line:?}"))?;
        if listed != process_identifier {
            continue;
        }
        let used: u64 = columns[2].parse().map_err(|_| format!("compute-apps 占用认不出：{line:?}"))?;
        total = Some(total.unwrap_or(0) + used);
    }
    Ok(total)
}

fn median_of(values: &[u64]) -> u64 {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted[sorted.len() / 2]
}

fn spread_of(values: &[u64]) -> u64 {
    values.iter().max().copied().unwrap_or(0) - values.iter().min().copied().unwrap_or(0)
}

fn pieces_for_size(size_bytes: u64) -> (u64, u64) {
    let piece_count = size_bytes.div_ceil(MAXIMUM_PIECE_BYTES);
    let piece_bytes_each = size_bytes.min(MAXIMUM_PIECE_BYTES);
    (piece_count, piece_bytes_each)
}

// ============================== 作废理由与一笔借据的记录 ==============================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum VoidScope {
    /// 只作废这一笔借据（登记第十一节 V2、V3 的 U_pre、V6、V9）。
    Lease,
    /// 作废整个子进程（V1、V3 的 U_dev、V4、V5、V7、V8）。
    Child,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum VoidReason {
    ReadFailure,
    PreSpreadTooWide,
    DeviceSpreadTooWide,
    CardTotalMismatch,
    PiecePlanMismatch,
    AllocatorBelowPlan,
    PeakBelowSize,
    HoldControlDidNotHold,
    ProcessReportMismatch,
    CalibrationOutOfBand,
    ForeignProcessListed,
    WaitDidNotCoverWork,
    OuterTimerShorterThanWait,
    WorkNotInFlightAtPoll,
    ChildFailure,
}

impl VoidReason {
    const EVERY_REASON: [VoidReason; 15] = [
        VoidReason::ReadFailure,
        VoidReason::PreSpreadTooWide,
        VoidReason::DeviceSpreadTooWide,
        VoidReason::CardTotalMismatch,
        VoidReason::PiecePlanMismatch,
        VoidReason::AllocatorBelowPlan,
        VoidReason::PeakBelowSize,
        VoidReason::HoldControlDidNotHold,
        VoidReason::ProcessReportMismatch,
        VoidReason::CalibrationOutOfBand,
        VoidReason::ForeignProcessListed,
        VoidReason::WaitDidNotCoverWork,
        VoidReason::OuterTimerShorterThanWait,
        VoidReason::WorkNotInFlightAtPoll,
        VoidReason::ChildFailure,
    ];

    /// 标签里带登记里的编号：V 是第十一节作废条款，K 是第九节装置自检，F 是第十节失败条款。
    fn label(self) -> &'static str {
        match self {
            VoidReason::ReadFailure => "v2_read_failure",
            VoidReason::PreSpreadTooWide => "v3_pre_spread_too_wide",
            VoidReason::DeviceSpreadTooWide => "v3_device_spread_too_wide",
            VoidReason::CardTotalMismatch => "v4_card_total_mismatch",
            VoidReason::PiecePlanMismatch => "v5_k2_piece_plan_mismatch",
            VoidReason::AllocatorBelowPlan => "v6_k3_allocator_below_plan",
            VoidReason::PeakBelowSize => "v6_k4_peak_below_size",
            VoidReason::HoldControlDidNotHold => "f1_k4_hold_control_did_not_hold",
            VoidReason::ProcessReportMismatch => "v7_k5_process_report_mismatch",
            VoidReason::CalibrationOutOfBand => "v7_k6_calibration_out_of_band",
            VoidReason::ForeignProcessListed => "v1_foreign_process_listed",
            VoidReason::WaitDidNotCoverWork => "k7_wait_did_not_cover_work",
            VoidReason::OuterTimerShorterThanWait => "v9_k8_outer_timer_shorter_than_wait",
            VoidReason::WorkNotInFlightAtPoll => "f6_k9_work_not_in_flight_at_poll",
            VoidReason::ChildFailure => "v8_child_failure",
        }
    }

    fn from_label(label: &str) -> Option<VoidReason> {
        VoidReason::EVERY_REASON.into_iter().find(|reason| reason.label() == label)
    }

    fn scope(self) -> VoidScope {
        match self {
            VoidReason::ReadFailure
            | VoidReason::PreSpreadTooWide
            | VoidReason::AllocatorBelowPlan
            | VoidReason::PeakBelowSize
            | VoidReason::HoldControlDidNotHold
            | VoidReason::WaitDidNotCoverWork
            | VoidReason::OuterTimerShorterThanWait
            | VoidReason::WorkNotInFlightAtPoll => VoidScope::Lease,
            VoidReason::DeviceSpreadTooWide
            | VoidReason::CardTotalMismatch
            | VoidReason::PiecePlanMismatch
            | VoidReason::ProcessReportMismatch
            | VoidReason::CalibrationOutOfBand
            | VoidReason::ForeignProcessListed
            | VoidReason::ChildFailure => VoidScope::Child,
        }
    }
}

fn void_reasons_text(reasons: &[VoidReason]) -> String {
    if reasons.is_empty() {
        return "none".to_string();
    }
    reasons.iter().map(|reason| reason.label()).collect::<Vec<_>>().join("+")
}

fn void_reasons_from_text(text: &str) -> Result<Vec<VoidReason>, String> {
    if text == "none" {
        return Ok(Vec::new());
    }
    text.split('+').map(|label| VoidReason::from_label(label).ok_or_else(|| format!("作废理由认不出：{label}"))).collect()
}

/// `Device::generate_allocator_report()` 的两项合计（字节）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct AllocatorFigures {
    reserved_bytes: u64,
    allocated_bytes: u64,
}

fn allocator_figures_text(figures: Option<AllocatorFigures>) -> String {
    match figures {
        Some(figures) => format!("{}/{}", figures.reserved_bytes, figures.allocated_bytes),
        None => "none".to_string(),
    }
}

fn allocator_figures_from_text(text: &str) -> Result<Option<AllocatorFigures>, String> {
    if text == "none" {
        return Ok(None);
    }
    let (reserved, allocated) = text.split_once('/').ok_or_else(|| format!("分配器两项认不出：{text}"))?;
    let reserved_bytes = reserved.parse().map_err(|_| format!("分配器 reserved 认不出：{text}"))?;
    let allocated_bytes = allocated.parse().map_err(|_| format!("分配器 allocated 认不出：{text}"))?;
    Ok(Some(AllocatorFigures { reserved_bytes, allocated_bytes }))
}

/// t0 之后背靠背的一份报告：起读、读完相对 t0 的微秒，与这个进程占着的（MiB；读不到是 `None`）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct WindowSample {
    started_microseconds: u64,
    finished_microseconds: u64,
    used_mebibytes: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct LeaseRecord {
    job: ChildJob,
    lease_index: u32,
    piece_count: u64,
    piece_bytes: u64,
    work_microseconds: u64,
    device_mebibytes: u64,
    device_spread_mebibytes: u64,
    pre_mebibytes: u64,
    pre_spread_mebibytes: u64,
    peak_mebibytes: Option<u64>,
    allocator_device: Option<AllocatorFigures>,
    allocator_pre: Option<AllocatorFigures>,
    allocator_peak: Option<AllocatorFigures>,
    allocator_next: Option<AllocatorFigures>,
    allocator_end: Option<AllocatorFigures>,
    submit_to_poll_microseconds: u64,
    wait_microseconds: u64,
    outer_microseconds: u64,
    /// 臂是等待而 `Wait` 返回超时或错时的原样文本（空格换成下划线）。
    wait_failure: Option<String>,
    samples: Vec<WindowSample>,
    void_reasons: Vec<VoidReason>,
}

fn optional_mebibytes_text(value: Option<u64>) -> String {
    value.map_or_else(|| "none".to_string(), |mebibytes| mebibytes.to_string())
}

fn optional_mebibytes_from_text(text: &str) -> Result<Option<u64>, String> {
    if text == "none" {
        return Ok(None);
    }
    text.parse().map(Some).map_err(|_| format!("整数认不出：{text}"))
}

impl LeaseRecord {
    fn new(job: ChildJob, lease_index: u32) -> LeaseRecord {
        LeaseRecord {
            job,
            lease_index,
            piece_count: 0,
            piece_bytes: 0,
            work_microseconds: 0,
            device_mebibytes: 0,
            device_spread_mebibytes: 0,
            pre_mebibytes: 0,
            pre_spread_mebibytes: 0,
            peak_mebibytes: None,
            allocator_device: None,
            allocator_pre: None,
            allocator_peak: None,
            allocator_next: None,
            allocator_end: None,
            submit_to_poll_microseconds: 0,
            wait_microseconds: 0,
            outer_microseconds: 0,
            wait_failure: None,
            samples: Vec::new(),
            void_reasons: Vec::new(),
        }
    }

    fn is_void(&self) -> bool {
        !self.void_reasons.is_empty()
    }

    fn has_child_scope_reason(&self) -> bool {
        self.void_reasons.iter().any(|reason| reason.scope() == VoidScope::Child)
    }

    /// 这笔借据的 τ（MiB）：U_pre 那三份的极差加取整余量。
    fn tau_mebibytes(&self) -> u64 {
        self.pre_spread_mebibytes + READING_ROUNDING_ALLOWANCE_MEBIBYTES
    }

    fn tau_device_mebibytes(&self) -> u64 {
        self.device_spread_mebibytes + READING_ROUNDING_ALLOWANCE_MEBIBYTES
    }

    fn to_line(&self) -> String {
        let samples = if self.samples.is_empty() {
            "none".to_string()
        } else {
            self.samples
                .iter()
                .map(|sample| {
                    format!(
                        "{}:{}:{}",
                        sample.started_microseconds,
                        sample.finished_microseconds,
                        optional_mebibytes_text(sample.used_mebibytes)
                    )
                })
                .collect::<Vec<_>>()
                .join(";")
        };
        format!(
            "E7RESULT name=lease {} round={} lease={} piece_count={} piece_bytes={} work_microseconds={} device_mebibytes={} \
             device_spread_mebibytes={} pre_mebibytes={} pre_spread_mebibytes={} peak_mebibytes={} allocator_device={} \
             allocator_pre={} allocator_peak={} allocator_next={} allocator_end={} submit_to_poll_microseconds={} \
             wait_microseconds={} outer_microseconds={} wait_failure={} void={} samples={}",
            self.job.cell.fields_text(),
            self.job.round,
            self.lease_index,
            self.piece_count,
            self.piece_bytes,
            self.work_microseconds,
            self.device_mebibytes,
            self.device_spread_mebibytes,
            self.pre_mebibytes,
            self.pre_spread_mebibytes,
            optional_mebibytes_text(self.peak_mebibytes),
            allocator_figures_text(self.allocator_device),
            allocator_figures_text(self.allocator_pre),
            allocator_figures_text(self.allocator_peak),
            allocator_figures_text(self.allocator_next),
            allocator_figures_text(self.allocator_end),
            self.submit_to_poll_microseconds,
            self.wait_microseconds,
            self.outer_microseconds,
            self.wait_failure.clone().unwrap_or_else(|| "none".to_string()),
            void_reasons_text(&self.void_reasons),
            samples,
        )
    }

    fn from_line(line: &str) -> Result<LeaseRecord, String> {
        let fields = parse_fields(line);
        let field = |name: &str| fields.get(name).map(String::as_str).ok_or_else(|| format!("缺字段 {name}"));
        let number = |name: &str| -> Result<u64, String> { field(name)?.parse().map_err(|_| format!("{name} 不是整数")) };
        let round: u32 = number("round")?.try_into().map_err(|_| "round 太大".to_string())?;
        let lease_index: u32 = number("lease")?.try_into().map_err(|_| "lease 太大".to_string())?;
        let job = ChildJob { cell: CellKey::from_fields(&fields)?, round };
        let mut samples = Vec::new();
        if field("samples")? != "none" {
            for text in field("samples")?.split(';') {
                let parts: Vec<&str> = text.split(':').collect();
                if parts.len() != 3 {
                    return Err(format!("窗口里一份读数不是三段：{text}"));
                }
                samples.push(WindowSample {
                    started_microseconds: parts[0].parse().map_err(|_| format!("起读认不出：{text}"))?,
                    finished_microseconds: parts[1].parse().map_err(|_| format!("读完认不出：{text}"))?,
                    used_mebibytes: optional_mebibytes_from_text(parts[2])?,
                });
            }
        }
        Ok(LeaseRecord {
            job,
            lease_index,
            piece_count: number("piece_count")?,
            piece_bytes: number("piece_bytes")?,
            work_microseconds: number("work_microseconds")?,
            device_mebibytes: number("device_mebibytes")?,
            device_spread_mebibytes: number("device_spread_mebibytes")?,
            pre_mebibytes: number("pre_mebibytes")?,
            pre_spread_mebibytes: number("pre_spread_mebibytes")?,
            peak_mebibytes: optional_mebibytes_from_text(field("peak_mebibytes")?)?,
            allocator_device: allocator_figures_from_text(field("allocator_device")?)?,
            allocator_pre: allocator_figures_from_text(field("allocator_pre")?)?,
            allocator_peak: allocator_figures_from_text(field("allocator_peak")?)?,
            allocator_next: allocator_figures_from_text(field("allocator_next")?)?,
            allocator_end: allocator_figures_from_text(field("allocator_end")?)?,
            submit_to_poll_microseconds: number("submit_to_poll_microseconds")?,
            wait_microseconds: number("wait_microseconds")?,
            outer_microseconds: number("outer_microseconds")?,
            wait_failure: match field("wait_failure")? {
                "none" => None,
                text => Some(text.to_string()),
            },
            samples,
            void_reasons: void_reasons_from_text(field("void")?)?,
        })
    }
}

// ============================== 卡的接口：真卡与模型各一份实现 ==============================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct SubmissionToken(u64);

/// 借据丢掉时（或别处）那一次 `device.poll` 的样子；等待那两种的超时都是 30 秒，由实现取。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PollMode {
    Poll,
    WaitLatest,
    WaitOwnSubmission(SubmissionToken),
}

/// 一次提交里第一张缓冲上派的内核：空转（时长由标定定）或只写一个字。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum WorkKind {
    Spin,
    SmallKernel,
}

trait CardHost {
    fn card(&self) -> Card;
    fn process_identifier(&self) -> u32;
    /// 单调时钟，自进程起点算的微秒；全部分段计时都在被测进程里用它计。
    fn now_microseconds(&self) -> u64;
    fn read_report_text(&mut self) -> Result<String, String>;
    fn read_compute_applications_text(&mut self) -> Result<String, String>;
    fn read_allocator_figures(&mut self) -> Option<AllocatorFigures>;
    /// 追加建缓冲：用途 `STORAGE | COPY_DST`，`mapped_at_creation: false`。
    fn create_lease_pieces(&mut self, piece_byte_lengths: &[u64]);
    /// 先对每张借据缓冲 `clear_buffer`，再在第一张上派一次内核，一次提交。
    fn submit_lease_work(&mut self, kind: WorkKind) -> SubmissionToken;
    fn poll_device(&mut self, mode: PollMode) -> Result<(), String>;
    /// 丢掉借据缓冲对象（还没做完的活由 wgpu 自己拖到做完再释放）。
    fn drop_lease_pieces(&mut self);
    /// 登记第八节「并存的长借据」：建一张缓冲、清一次、等完，活到子进程末尾。
    fn create_coexisting_buffer(&mut self, bytes: u64);
}

/// 原文只存 U_dev、U_pre、U_peak 与 t0 之后第一份（登记第五节「产物大小」）。
#[derive(Default)]
struct RawLog {
    entries: Vec<(String, String)>,
}

struct TakenReading {
    used_mebibytes: u64,
}

fn push_reason_once(reasons: &mut Vec<VoidReason>, reason: VoidReason) {
    if !reasons.contains(&reason) {
        reasons.push(reason);
    }
}

/// 读一份报告并解析：读不到这个进程占着的、报告里没有这张卡，记 V2 并交回 `None`（不当 0）；
/// 卡上列了装置以外的进程号记 V1；Total 与登记 B1 不符记 V4。
fn take_reading<Host: CardHost>(
    host: &mut Host,
    raw_log: &mut RawLog,
    raw_label: Option<&str>,
    reasons: &mut Vec<VoidReason>,
) -> Option<TakenReading> {
    let text = match host.read_report_text() {
        Ok(text) => text,
        Err(_) => {
            push_reason_once(reasons, VoidReason::ReadFailure);
            return None;
        }
    };
    if let Some(label) = raw_label {
        raw_log.entries.push((label.to_string(), text.clone()));
    }
    let card = host.card();
    let reading = match parse_card_reading(&text, card.bus_without_domain(), host.process_identifier()) {
        Ok(reading) => reading,
        Err(_) => {
            push_reason_once(reasons, VoidReason::ReadFailure);
            return None;
        }
    };
    if reading.total_mebibytes != Some(card.total_mebibytes()) {
        push_reason_once(reasons, VoidReason::CardTotalMismatch);
    }
    let process_identifier = host.process_identifier();
    if reading.listed_process_identifiers.iter().any(|listed| *listed != process_identifier) {
        push_reason_once(reasons, VoidReason::ForeignProcessListed);
    }
    match reading.process_used_mebibytes {
        Some(used_mebibytes) if !reading.process_used_is_unreadable => Some(TakenReading { used_mebibytes }),
        Some(_) | None => {
            push_reason_once(reasons, VoidReason::ReadFailure);
            None
        }
    }
}

/// 登记第七节 B5：同一时刻 `-q` 读出的这个进程占用与 `--query-compute-apps` 里这张卡、这个进程号那一行逐 MiB 相等。
fn check_process_report_against_compute_applications<Host: CardHost>(
    host: &mut Host,
    used_mebibytes: u64,
    reasons: &mut Vec<VoidReason>,
) {
    let card = host.card();
    let process_identifier = host.process_identifier();
    let compute_applications = host
        .read_compute_applications_text()
        .and_then(|text| parse_compute_applications(&text, card.bus_without_domain(), process_identifier));
    match compute_applications {
        Err(_) => push_reason_once(reasons, VoidReason::ReadFailure),
        Ok(Some(listed_mebibytes)) if listed_mebibytes == used_mebibytes => {}
        Ok(Some(_) | None) => push_reason_once(reasons, VoidReason::ProcessReportMismatch),
    }
}

fn poll_mode_for(arm: Arm, own_submission: SubmissionToken) -> PollMode {
    match arm {
        Arm::WaitLatest => PollMode::WaitLatest,
        Arm::WaitOwnSubmission => PollMode::WaitOwnSubmission(own_submission),
        Arm::Poll => PollMode::Poll,
    }
}

fn without_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join("_")
}

// ============================== 一笔借据（登记第五节「一笔借据在装置里怎么做」） ==============================

#[derive(Clone, Copy)]
struct DeviceBaselines {
    device_mebibytes: u64,
    device_spread_mebibytes: u64,
    allocator_device: Option<AllocatorFigures>,
}

struct LeasePlan {
    job: ChildJob,
    lease_index: u32,
    work_microseconds: u64,
    baselines: DeviceBaselines,
}

/// 丢缓冲、等设备到空闲，不进任何统计；只用在借据中途失败或「扣住不丢」组窗口之后。
fn release_everything_and_wait<Host: CardHost>(host: &mut Host) {
    host.drop_lease_pieces();
    let _ = host.poll_device(PollMode::WaitLatest);
}

/// 一次建、一次提交（登记借法 P1）。返回最后一次提交的号与提交返回的时刻。
fn submit_one_build_lease<Host: CardHost>(
    host: &mut Host,
    piece_byte_lengths: &[u64],
    liveness: Liveness,
) -> Result<(SubmissionToken, u64), String> {
    host.create_lease_pieces(piece_byte_lengths);
    let submission = host.submit_lease_work(WorkKind::Spin);
    let submitted_at = host.now_microseconds();
    if matches!(liveness, Liveness::Done) {
        host.poll_device(PollMode::WaitOwnSubmission(submission))?;
    }
    Ok((submission, submitted_at))
}

/// 一趟一趟（登记借法 P2，第 ② 段核对的样子）：每趟建、清 + 小内核、等完、丢；最后一趟的缓冲留着。
fn submit_pass_by_pass_lease<Host: CardHost>(
    host: &mut Host,
    piece_byte_lengths: &[u64],
) -> Result<(SubmissionToken, u64), String> {
    let mut last_pass = None;
    for pass_index in 0..PASSES_PER_PASS_BY_PASS_LEASE {
        host.create_lease_pieces(piece_byte_lengths);
        let submission = host.submit_lease_work(WorkKind::SmallKernel);
        last_pass = Some((submission, host.now_microseconds()));
        host.poll_device(PollMode::WaitLatest)?;
        if pass_index + 1 < PASSES_PER_PASS_BY_PASS_LEASE {
            host.drop_lease_pieces();
        }
    }
    Ok(last_pass.expect("至少一趟"))
}

fn run_one_lease<Host: CardHost>(host: &mut Host, plan: &LeasePlan, raw_log: &mut RawLog) -> LeaseRecord {
    let cell = plan.job.cell;
    let mut record = LeaseRecord::new(plan.job, plan.lease_index);
    record.work_microseconds = plan.work_microseconds;
    record.device_mebibytes = plan.baselines.device_mebibytes;
    record.device_spread_mebibytes = plan.baselines.device_spread_mebibytes;
    record.allocator_device = plan.baselines.allocator_device;
    let size_bytes = cell.size_bytes();

    // 第 1 步：建缓冲之前读三份，中位数是 U_pre，极差定 τ。
    let mut pre_readings = Vec::new();
    for read_index in 0..READS_PER_BASELINE {
        let raw_label = format!("lease{}_pre{}", plan.lease_index, read_index);
        if let Some(taken) = take_reading(host, raw_log, Some(&raw_label), &mut record.void_reasons) {
            pre_readings.push(taken.used_mebibytes);
        }
    }
    if pre_readings.len() != READS_PER_BASELINE {
        return record;
    }
    record.pre_mebibytes = median_of(&pre_readings);
    record.pre_spread_mebibytes = spread_of(&pre_readings);
    if record.pre_spread_mebibytes > MAXIMUM_BASELINE_SPREAD_MEBIBYTES {
        push_reason_once(&mut record.void_reasons, VoidReason::PreSpreadTooWide);
        return record;
    }
    record.allocator_pre = host.read_allocator_figures();

    // 第 2 步：张数 × 每张 == S（登记 B2，加法形）。
    let (piece_count, piece_bytes_each) = pieces_for_size(size_bytes);
    record.piece_count = piece_count;
    record.piece_bytes = piece_bytes_each;
    if piece_count * piece_bytes_each != size_bytes {
        push_reason_once(&mut record.void_reasons, VoidReason::PiecePlanMismatch);
        return record;
    }
    let piece_byte_lengths = vec![piece_bytes_each; usize::try_from(piece_count).expect("张数装得进 usize")];

    // 第 3、4 步：建缓冲、提交一次；「已做完」先等这次提交，「在飞」不等。
    let submission = match cell.method {
        Method::OneBuild => submit_one_build_lease(host, &piece_byte_lengths, cell.liveness),
        Method::PassByPass => submit_pass_by_pass_lease(host, &piece_byte_lengths),
    };
    let Ok((last_submission, submitted_at)) = submission else {
        push_reason_once(&mut record.void_reasons, VoidReason::ChildFailure);
        release_everything_and_wait(host);
        return record;
    };

    // 第 5 步：缓冲还活着时读一份（U_peak）与分配器报告，B5 比 compute-apps。
    let peak_label = format!("lease{}_peak", plan.lease_index);
    let Some(peak) = take_reading(host, raw_log, Some(&peak_label), &mut record.void_reasons) else {
        release_everything_and_wait(host);
        return record;
    };
    record.peak_mebibytes = Some(peak.used_mebibytes);
    record.allocator_peak = host.read_allocator_figures();
    check_process_report_against_compute_applications(host, peak.used_mebibytes, &mut record.void_reasons);
    check_peak_against_plan(&mut record, size_bytes);

    // 第 6、7 步：丢缓冲对象（「扣住不丢」组除外），然后借据丢掉：按臂做一次 poll，前后各记一次单调时钟。
    let drop_started_at = host.now_microseconds();
    if !cell.is_hold_control() {
        host.drop_lease_pieces();
    }
    let poll_started_at = host.now_microseconds();
    let poll_result = host.poll_device(poll_mode_for(cell.arm, last_submission));
    let lease_drop_instant = host.now_microseconds();
    record.submit_to_poll_microseconds = poll_started_at.saturating_sub(submitted_at);
    record.wait_microseconds = lease_drop_instant - poll_started_at;
    record.outer_microseconds = lease_drop_instant - drop_started_at;
    if let Err(text) = poll_result {
        record.wait_failure = Some(without_whitespace(&text));
    }
    check_timing_against_work(&mut record, cell);
    // 已经作废的借据读窗口没有用处（第 1 笔作废这一轮就不进，见 `run_child_procedure`）：不等 2 秒或 30 秒，直接收场。
    if record.is_void() {
        release_everything_and_wait(host);
        return record;
    }

    // 第 8 步：从 t0 起背靠背读报告，窗口 2 秒（长窗 30 秒）；t0 之后第一份、窗口末那一份另读分配器报告。
    let window_end = lease_drop_instant + cell.window_microseconds();
    let mut raw_samples: Vec<(u64, u64, Result<String, String>)> = Vec::new();
    loop {
        let started_at = host.now_microseconds();
        let text = host.read_report_text();
        let finished_at = host.now_microseconds();
        raw_samples.push((started_at, finished_at, text));
        if raw_samples.len() == 1 {
            record.allocator_next = host.read_allocator_figures();
        }
        if finished_at >= window_end {
            break;
        }
    }
    record.allocator_end = host.read_allocator_figures();
    record_window_samples(host, raw_log, &mut record, lease_drop_instant, raw_samples);
    if cell.is_hold_control() {
        check_hold_control_still_holds(&mut record, size_bytes);
        release_everything_and_wait(host);
    }
    record
}

/// 登记 B3：缓冲建好之后分配器分出的字节至少多出 S；登记 B4 后一半：S ≥ 512 MiB 时缓冲活着读到的占用至少多出 S − τ。
fn check_peak_against_plan(record: &mut LeaseRecord, size_bytes: u64) {
    if let (Some(before), Some(at_peak)) = (record.allocator_pre, record.allocator_peak) {
        if at_peak.allocated_bytes < before.allocated_bytes + size_bytes {
            push_reason_once(&mut record.void_reasons, VoidReason::AllocatorBelowPlan);
        }
    }
    let size_mebibytes = size_bytes / MEBIBYTE;
    if size_mebibytes >= 512 {
        if let Some(peak_mebibytes) = record.peak_mebibytes {
            if peak_mebibytes + record.tau_mebibytes() < record.pre_mebibytes + size_mebibytes {
                push_reason_once(&mut record.void_reasons, VoidReason::PeakBelowSize);
            }
        }
    }
}

/// 登记第九节 K7（等待真的盖住了剩下的活）、K8（外层计时包住 T_wait）、K9（poll 开始时活还没做完）。
fn check_timing_against_work(record: &mut LeaseRecord, cell: CellKey) {
    let covered_microseconds = record.work_microseconds * COVERED_WORK_PERCENT / 100;
    if cell.liveness.is_in_flight() {
        let wait_covers_the_rest =
            record.wait_microseconds + record.submit_to_poll_microseconds >= covered_microseconds;
        if cell.arm.waits() && record.wait_failure.is_none() && !wait_covers_the_rest {
            push_reason_once(&mut record.void_reasons, VoidReason::WaitDidNotCoverWork);
        }
        if record.submit_to_poll_microseconds >= covered_microseconds {
            push_reason_once(&mut record.void_reasons, VoidReason::WorkNotInFlightAtPoll);
        }
    }
    if record.outer_microseconds < record.wait_microseconds {
        push_reason_once(&mut record.void_reasons, VoidReason::OuterTimerShorterThanWait);
    }
}

/// 窗口里每份报告按相对 t0 的时刻记下；第一份或最后一份读不到，那一笔借据作废（登记 V2）。
fn record_window_samples<Host: CardHost>(
    host: &Host,
    raw_log: &mut RawLog,
    record: &mut LeaseRecord,
    lease_drop_instant: u64,
    raw_samples: Vec<(u64, u64, Result<String, String>)>,
) {
    let card = host.card();
    let process_identifier = host.process_identifier();
    for (sample_index, (started_at, finished_at, text)) in raw_samples.into_iter().enumerate() {
        let mut used_mebibytes = None;
        if let Ok(text) = text {
            if sample_index == 0 {
                raw_log.entries.push((format!("lease{}_after_first", record.lease_index), text.clone()));
            }
            if let Ok(reading) = parse_card_reading(&text, card.bus_without_domain(), process_identifier) {
                if reading.listed_process_identifiers.iter().any(|listed| *listed != process_identifier) {
                    push_reason_once(&mut record.void_reasons, VoidReason::ForeignProcessListed);
                }
                if !reading.process_used_is_unreadable {
                    used_mebibytes = reading.process_used_mebibytes;
                }
            }
        }
        record.samples.push(WindowSample {
            started_microseconds: started_at - lease_drop_instant,
            finished_microseconds: finished_at - lease_drop_instant,
            used_mebibytes,
        });
    }
    let first_is_unreadable = record.samples.first().is_none_or(|sample| sample.used_mebibytes.is_none());
    let last_is_unreadable = record.samples.last().is_none_or(|sample| sample.used_mebibytes.is_none());
    if first_is_unreadable || last_is_unreadable {
        push_reason_once(&mut record.void_reasons, VoidReason::ReadFailure);
    }
}

/// 登记 B4 前一半：「扣住不丢」组 t0 之后第一份读数必须含着缓冲，R_next ≥ S − τ。
fn check_hold_control_still_holds(record: &mut LeaseRecord, size_bytes: u64) {
    let size_mebibytes = size_bytes / MEBIBYTE;
    if let Some(next_mebibytes) = record.samples.first().and_then(|sample| sample.used_mebibytes) {
        if next_mebibytes + record.tau_mebibytes() < record.pre_mebibytes + size_mebibytes {
            push_reason_once(&mut record.void_reasons, VoidReason::HoldControlDidNotHold);
        }
    }
}

// ============================== 一个子进程 ==============================

struct ChildOutcome {
    leases: Vec<LeaseRecord>,
    child_void: Vec<VoidReason>,
}

/// 设备起好、管线编好、标定做完之后：读三份记 U_dev 与分配器报告，B5 比一次。极差太大交回 `None`。
fn measure_device_baselines<Host: CardHost>(
    host: &mut Host,
    raw_log: &mut RawLog,
    child_void: &mut Vec<VoidReason>,
) -> Option<DeviceBaselines> {
    let mut device_readings = Vec::new();
    for read_index in 0..READS_PER_BASELINE {
        let raw_label = format!("device{read_index}");
        if let Some(taken) = take_reading(host, raw_log, Some(&raw_label), child_void) {
            device_readings.push(taken.used_mebibytes);
        }
    }
    if device_readings.len() != READS_PER_BASELINE {
        return None;
    }
    let device_spread_mebibytes = spread_of(&device_readings);
    if device_spread_mebibytes > MAXIMUM_BASELINE_SPREAD_MEBIBYTES {
        push_reason_once(child_void, VoidReason::DeviceSpreadTooWide);
        return None;
    }
    let allocator_device = host.read_allocator_figures();
    let device_mebibytes = median_of(&device_readings);
    check_process_report_against_compute_applications(host, device_mebibytes, child_void);
    Some(DeviceBaselines { device_mebibytes, device_spread_mebibytes, allocator_device })
}

fn run_child_procedure<Host: CardHost>(
    host: &mut Host,
    job: &ChildJob,
    work_microseconds: u64,
    calibration_is_in_band: bool,
    raw_log: &mut RawLog,
) -> ChildOutcome {
    let mut child_void = Vec::new();
    let mut leases = Vec::new();
    if !calibration_is_in_band {
        push_reason_once(&mut child_void, VoidReason::CalibrationOutOfBand);
        return ChildOutcome { leases, child_void };
    }
    let Some(baselines) = measure_device_baselines(host, raw_log, &mut child_void) else {
        return ChildOutcome { leases, child_void };
    };
    if job.cell.coexisting_mebibytes() > 0 {
        host.create_coexisting_buffer(job.cell.coexisting_mebibytes() * MEBIBYTE);
    }
    for lease_index in 1..=job.cell.lease_count() {
        let plan = LeasePlan { job: *job, lease_index, work_microseconds, baselines };
        let record = run_one_lease(host, &plan, raw_log);
        for reason in &record.void_reasons {
            if reason.scope() == VoidScope::Child {
                push_reason_once(&mut child_void, *reason);
            }
        }
        let round_cannot_complete = record.is_void() && (lease_index == 1 || record.has_child_scope_reason());
        leases.push(record);
        if round_cannot_complete {
            break;
        }
    }
    ChildOutcome { leases, child_void }
}

// ============================== 模型卡：不起卡的那一份 CardHost ==============================
//
// 模型只回答「这段步骤写对没有」，不回答「真卡怎么样」：时间是虚拟时钟，读一份报告记 50 ms（登记第四节量到的 0.05 秒），
// 内存账是一条写死的规则（比 256 MiB 小的缓冲共用一块 256 MiB 的块、块用过就留着；更大的单独拿、释放时才还；
// 释放要等缓冲被丢掉、提交做完、之后有一次 poll）。它存在的理由是让登记第九节的变异在不起卡时也有东西可红。

const SIMULATED_PROCESS_IDENTIFIER: u32 = 4242;
const SIMULATED_RUNTIME_MEBIBYTES: u64 = 300;
const SIMULATED_REPORT_COST_MICROSECONDS: u64 = 50_000;
const SIMULATED_SMALL_KERNEL_MICROSECONDS: u64 = 2_000;
const SIMULATED_BLOCK_MEBIBYTES: u64 = 256;
const SIMULATED_DEDICATED_ALLOCATION_THRESHOLD_BYTES: u64 = SIMULATED_BLOCK_MEBIBYTES * MEBIBYTE;
const SIMULATED_WAIT_OVERHEAD_MICROSECONDS: u64 = 100;
const SIMULATED_POLL_OVERHEAD_MICROSECONDS: u64 = 50;

struct SimulatedPiece {
    bytes: u64,
    is_dropped: bool,
    is_released: bool,
    completes_at_microseconds: u64,
}

struct SimulatedCardHost {
    card: Card,
    clock_microseconds: std::cell::Cell<u64>,
    spin_microseconds: u64,
    block_is_retained: bool,
    pieces: Vec<SimulatedPiece>,
    submission_completions: Vec<u64>,
    coexisting_bytes: u64,
    /// 单测用：报告里多列一个别的进程号（登记 V1）。
    foreign_process_identifier: Option<u32>,
    /// 单测用：compute-apps 那一份比 `-q` 那一份多出的 MiB（登记 B5）。
    compute_applications_extra_mebibytes: u64,
}

impl SimulatedCardHost {
    fn new(card: Card, spin_microseconds: u64) -> SimulatedCardHost {
        SimulatedCardHost {
            card,
            clock_microseconds: std::cell::Cell::new(0),
            spin_microseconds,
            block_is_retained: false,
            pieces: Vec::new(),
            submission_completions: Vec::new(),
            coexisting_bytes: 0,
            foreign_process_identifier: None,
            compute_applications_extra_mebibytes: 0,
        }
    }

    fn advance(&self, microseconds: u64) {
        self.clock_microseconds.set(self.clock_microseconds.get() + microseconds);
    }

    fn dedicated_live_bytes(&self) -> u64 {
        self.pieces
            .iter()
            .filter(|piece| !piece.is_released && piece.bytes >= SIMULATED_DEDICATED_ALLOCATION_THRESHOLD_BYTES)
            .map(|piece| piece.bytes)
            .sum()
    }

    fn retained_block_bytes(&self) -> u64 {
        if self.block_is_retained { SIMULATED_BLOCK_MEBIBYTES * MEBIBYTE } else { 0 }
    }

    fn process_used_mebibytes(&self) -> u64 {
        SIMULATED_RUNTIME_MEBIBYTES + (self.retained_block_bytes() + self.dedicated_live_bytes()) / MEBIBYTE
    }

    /// 被丢掉、做完了的缓冲，在一次 poll 之后才还。
    fn release_dropped_and_finished_pieces(&mut self) {
        let now = self.clock_microseconds.get();
        for piece in &mut self.pieces {
            if piece.is_dropped && piece.completes_at_microseconds <= now {
                piece.is_released = true;
            }
        }
    }

    fn report_text(&self) -> String {
        let card = self.card;
        let used_by_process = self.process_used_mebibytes();
        let free = card.total_mebibytes().saturating_sub(497 + 2 + used_by_process);
        let mut processes = format!(
            "        GPU instance ID                                : N/A\n        Compute instance ID                            : N/A\n\
             \x20       Process ID                                     : {}\n            Type                                       : C+G\n\
             \x20           Name                                       : /simulated/e174\n            Used GPU Memory                            : {} MiB\n",
            SIMULATED_PROCESS_IDENTIFIER, used_by_process
        );
        if let Some(foreign) = self.foreign_process_identifier {
            processes.push_str(&format!(
                "        Process ID                                     : {foreign}\n            Used GPU Memory                            : 100 MiB\n"
            ));
        }
        format!(
            "\n==============NVSMI LOG==============\n\nDriver Version                                         : {EXPECTED_DRIVER_VERSION}\n\
             Attached GPUs                                          : 1\nGPU 00000000:{}\n    FB Memory Usage\n\
             \x20       Total                                          : {} MiB\n        Reserved                                       : 497 MiB\n\
             \x20       Used                                           : 2 MiB\n        Free                                           : {} MiB\n\
             \x20   BAR1 Memory Usage\n        Total                                          : 32768 MiB\n\
             \x20       Used                                           : 1 MiB\n        Free                                           : 32767 MiB\n\
             \x20   Processes\n{}\n",
            card.bus_without_domain(),
            card.total_mebibytes(),
            free,
            processes
        )
    }
}

impl CardHost for SimulatedCardHost {
    fn card(&self) -> Card {
        self.card
    }

    fn process_identifier(&self) -> u32 {
        SIMULATED_PROCESS_IDENTIFIER
    }

    fn now_microseconds(&self) -> u64 {
        self.clock_microseconds.get()
    }

    fn read_report_text(&mut self) -> Result<String, String> {
        self.advance(SIMULATED_REPORT_COST_MICROSECONDS);
        Ok(self.report_text())
    }

    fn read_compute_applications_text(&mut self) -> Result<String, String> {
        self.advance(SIMULATED_REPORT_COST_MICROSECONDS);
        Ok(format!(
            "00000000:{}, {}, {}\n",
            self.card.bus_without_domain(),
            SIMULATED_PROCESS_IDENTIFIER,
            self.process_used_mebibytes() + self.compute_applications_extra_mebibytes
        ))
    }

    fn read_allocator_figures(&mut self) -> Option<AllocatorFigures> {
        let allocated_bytes = self
            .pieces
            .iter()
            .filter(|piece| !piece.is_released)
            .map(|piece| piece.bytes)
            .sum::<u64>()
            + self.coexisting_bytes;
        Some(AllocatorFigures {
            reserved_bytes: self.retained_block_bytes() + self.dedicated_live_bytes(),
            allocated_bytes,
        })
    }

    fn create_lease_pieces(&mut self, piece_byte_lengths: &[u64]) {
        for bytes in piece_byte_lengths {
            if *bytes < SIMULATED_DEDICATED_ALLOCATION_THRESHOLD_BYTES {
                self.block_is_retained = true;
            }
            self.pieces.push(SimulatedPiece {
                bytes: *bytes,
                is_dropped: false,
                is_released: false,
                completes_at_microseconds: 0,
            });
        }
    }

    fn submit_lease_work(&mut self, kind: WorkKind) -> SubmissionToken {
        self.advance(300);
        let duration = match kind {
            WorkKind::Spin => self.spin_microseconds,
            WorkKind::SmallKernel => SIMULATED_SMALL_KERNEL_MICROSECONDS,
        };
        let completes_at = self.clock_microseconds.get() + duration;
        for piece in self.pieces.iter_mut().filter(|piece| !piece.is_dropped && piece.completes_at_microseconds == 0) {
            piece.completes_at_microseconds = completes_at;
        }
        self.submission_completions.push(completes_at);
        SubmissionToken(u64::try_from(self.submission_completions.len() - 1).expect("提交号装得进 u64"))
    }

    fn poll_device(&mut self, mode: PollMode) -> Result<(), String> {
        match mode {
            PollMode::Poll => self.advance(SIMULATED_POLL_OVERHEAD_MICROSECONDS),
            PollMode::WaitLatest => {
                let latest = self.submission_completions.iter().copied().max().unwrap_or(0);
                self.clock_microseconds.set(self.clock_microseconds.get().max(latest) + SIMULATED_WAIT_OVERHEAD_MICROSECONDS);
            }
            PollMode::WaitOwnSubmission(token) => {
                let own = self.submission_completions[usize::try_from(token.0).expect("提交号装得进 usize")];
                self.clock_microseconds.set(self.clock_microseconds.get().max(own) + SIMULATED_WAIT_OVERHEAD_MICROSECONDS);
            }
        }
        self.release_dropped_and_finished_pieces();
        Ok(())
    }

    fn drop_lease_pieces(&mut self) {
        self.advance(50);
        for piece in &mut self.pieces {
            piece.is_dropped = true;
        }
    }

    fn create_coexisting_buffer(&mut self, bytes: u64) {
        self.block_is_retained = true;
        self.coexisting_bytes += bytes;
        self.advance(300);
    }
}

// ============================== 读产物：按轮、按格归拢 ==============================

#[derive(Clone, Debug, PartialEq, Eq)]
struct ChildDoneRecord {
    child_void: Vec<VoidReason>,
}

#[derive(Default, Clone, Debug)]
struct RoundData {
    done: Option<ChildDoneRecord>,
    failure_detail: Option<String>,
    leases: Vec<LeaseRecord>,
}

type ProductRounds = BTreeMap<ChildJob, RoundData>;

fn job_of_fields(fields: &BTreeMap<String, String>) -> Result<ChildJob, String> {
    let round: u32 = fields.get("round").ok_or("缺字段 round")?.parse().map_err(|_| "round 不是整数".to_string())?;
    Ok(ChildJob { cell: CellKey::from_fields(fields)?, round })
}

/// 产物里 `name=lease`、`name=child_done`、`name=child_failed` 三种行按轮归拢；别的行不管。
fn parse_product(text: &str) -> Result<ProductRounds, String> {
    let mut rounds: ProductRounds = BTreeMap::new();
    for (line_index, line) in text.lines().enumerate() {
        let located = |message: String| format!("第 {} 行：{message}", line_index + 1);
        let Some(body) = line.strip_prefix("E7RESULT ") else { continue };
        let fields = parse_fields(body);
        match fields.get("name").map(String::as_str) {
            Some("lease") => {
                let record = LeaseRecord::from_line(line).map_err(located)?;
                rounds.entry(record.job).or_default().leases.push(record);
            }
            Some("child_done") => {
                let job = job_of_fields(&fields).map_err(located)?;
                let child_void = void_reasons_from_text(fields.get("child_void").ok_or_else(|| located("缺 child_void".to_string()))?)
                    .map_err(located)?;
                let round = rounds.entry(job).or_default();
                if round.done.is_some() {
                    return Err(located("同一轮出现两条 child_done".to_string()));
                }
                round.done = Some(ChildDoneRecord { child_void });
            }
            Some("child_failed") => {
                let job = job_of_fields(&fields).map_err(located)?;
                let detail = fields.get("detail").cloned().unwrap_or_default();
                rounds.entry(job).or_default().failure_detail = Some(detail);
            }
            Some(_) | None => {}
        }
    }
    Ok(rounds)
}

impl RoundData {
    fn lease(&self, lease_index: u32) -> Option<&LeaseRecord> {
        self.leases.iter().find(|lease| lease.lease_index == lease_index)
    }

    fn lease_is_valid(&self, lease_index: u32) -> bool {
        self.lease(lease_index).is_some_and(|lease| !lease.is_void())
    }

    /// 一轮有效：子进程没作废，第 1 笔有效，第 2、3 笔里至少一笔有效（长窗那几格只有一笔）。
    fn is_complete(&self, lease_count: u32) -> bool {
        let Some(done) = &self.done else { return false };
        if !done.child_void.is_empty() || self.failure_detail.is_some() {
            return false;
        }
        if lease_count == 1 {
            return self.lease_is_valid(1);
        }
        self.lease_is_valid(1) && (self.lease_is_valid(2) || self.lease_is_valid(3))
    }

    /// 这一轮在第一栏（第 1 笔）或第二栏（第 2、3 笔）里的有效借据。
    fn column_leases(&self, column: Column) -> Vec<&LeaseRecord> {
        let wanted: &[u32] = match column {
            Column::First => &[1],
            Column::Later => &[2, 3],
        };
        wanted.iter().filter_map(|index| self.lease(*index)).filter(|lease| !lease.is_void()).collect()
    }

    /// 这一轮没能有效的原因，原样列给产物（登记第十一节：作废的轮照样留在产物里）。
    fn void_explanation(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if let Some(detail) = &self.failure_detail {
            parts.push(format!("child_failed:{detail}"));
        }
        if let Some(done) = &self.done {
            if !done.child_void.is_empty() {
                parts.push(format!("child_void:{}", void_reasons_text(&done.child_void)));
            }
        } else if self.failure_detail.is_none() {
            parts.push("no_child_done_line".to_string());
        }
        for lease in self.leases.iter().filter(|lease| lease.is_void()) {
            parts.push(format!("lease{}:{}", lease.lease_index, void_reasons_text(&lease.void_reasons)));
        }
        if parts.is_empty() { "none".to_string() } else { parts.join(",") }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Column {
    First,
    Later,
}

impl Column {
    fn label(self) -> &'static str {
        match self {
            Column::First => "first",
            Column::Later => "later",
        }
    }
}

/// 按格归拢；每格取轮号最小的 5 个有效轮（登记第六节：5 轮各自的值都列出来）。
struct CellView<'rounds> {
    cell: CellKey,
    all_rounds: Vec<(u32, &'rounds RoundData)>,
    complete_rounds: Vec<(u32, &'rounds RoundData)>,
}

fn group_into_cells(rounds: &ProductRounds) -> Vec<CellView<'_>> {
    let mut by_cell: BTreeMap<CellKey, Vec<(u32, &RoundData)>> = BTreeMap::new();
    for (job, data) in rounds {
        by_cell.entry(job.cell).or_default().push((job.round, data));
    }
    by_cell
        .into_iter()
        .map(|(cell, mut all_rounds)| {
            all_rounds.sort_by_key(|(round, _)| *round);
            let complete_rounds: Vec<(u32, &RoundData)> = all_rounds
                .iter()
                .filter(|(_, data)| data.is_complete(cell.lease_count()))
                .take(usize::try_from(ROUNDS_REQUIRED_PER_CELL).expect("轮数装得进 usize"))
                .copied()
                .collect();
            CellView { cell, all_rounds, complete_rounds }
        })
        .collect()
}

// ============================== 判：每个量的算法（登记第六节） ==============================

fn signed(value: u64) -> i64 {
    i64::try_from(value).expect("读数与时长装得进 i64")
}

/// 判定的可调旋钮，只有 τ：判别力自证（登记第八节）把它换成别的值，看判定翻不翻。
#[derive(Clone, Copy, Debug)]
struct JudgeSettings {
    tau_override_mebibytes: Option<u64>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum TimeToClear {
    /// t0 之后第一份就不含：登记 Q2c 记 0。
    Next,
    /// 第一份含，从这一份（相对 t0 的起读微秒）起之后每一份都不含。
    After(u64),
    /// 窗口末那一份还含，或读不到：写「> 窗口长（窗口上沿，没量到）」。
    BeyondWindow,
}

impl TimeToClear {
    fn text(self) -> String {
        match self {
            TimeToClear::Next => "0".to_string(),
            TimeToClear::After(microseconds) => microseconds.to_string(),
            TimeToClear::BeyondWindow => "beyond_window".to_string(),
        }
    }
}

impl LeaseRecord {
    fn judge_tau_mebibytes(&self, settings: JudgeSettings) -> u64 {
        settings.tau_override_mebibytes.unwrap_or_else(|| self.tau_mebibytes())
    }

    fn judge_tau_device_mebibytes(&self, settings: JudgeSettings) -> u64 {
        settings.tau_override_mebibytes.unwrap_or_else(|| self.tau_device_mebibytes())
    }

    fn size_mebibytes(&self) -> i64 {
        signed(self.job.cell.size_mebibytes)
    }

    /// R_lease(t) = U(t) − U_pre。
    fn lease_remainder_of(&self, used_mebibytes: u64) -> i64 {
        signed(used_mebibytes) - signed(self.pre_mebibytes)
    }

    /// R_next：t0 之后第一份报告的 R_lease。
    fn next_remainder_mebibytes(&self) -> Option<i64> {
        let used = self.samples.first()?.used_mebibytes?;
        Some(self.lease_remainder_of(used))
    }

    /// 窗口末那一份的 R_dev = U − U_dev，「并存的长借据」那组再减那张缓冲的 64 MiB。
    fn end_device_remainder_mebibytes(&self) -> Option<i64> {
        let used = self.samples.last()?.used_mebibytes?;
        Some(signed(used) - signed(self.device_mebibytes) - signed(self.job.cell.coexisting_mebibytes()))
    }

    /// 缓冲活着时的 ΔU_peak 减去 S：按块拿多出来的那一截。
    fn peak_over_size_mebibytes(&self) -> Option<i64> {
        Some(self.lease_remainder_of(self.peak_mebibytes?) - self.size_mebibytes())
    }

    /// Q1 在飞档：E = T_wait − max(0, W − e)。
    fn excess_wait_microseconds(&self) -> i64 {
        signed(self.wait_microseconds) - (signed(self.work_microseconds) - signed(self.submit_to_poll_microseconds)).max(0)
    }

    /// Q1 判的那个量：在飞档取超出，已做完档取 T_wait 本身。
    fn judged_wait_microseconds(&self) -> i64 {
        if self.job.cell.liveness.is_in_flight() {
            self.excess_wait_microseconds()
        } else {
            signed(self.wait_microseconds)
        }
    }

    fn time_to_clear(&self, settings: JudgeSettings) -> TimeToClear {
        let tau = signed(self.judge_tau_mebibytes(settings));
        let satisfies = |sample: &WindowSample| sample.used_mebibytes.is_some_and(|used| self.lease_remainder_of(used) <= tau);
        let satisfied_suffix_length = self.samples.iter().rev().take_while(|sample| satisfies(sample)).count();
        if satisfied_suffix_length == 0 {
            return TimeToClear::BeyondWindow;
        }
        let first_clear_index = self.samples.len() - satisfied_suffix_length;
        if first_clear_index == 0 {
            TimeToClear::Next
        } else {
            TimeToClear::After(self.samples[first_clear_index].started_microseconds)
        }
    }

    /// 窗口里 R_lease > τ 的份数与读得到的总份数。
    fn positive_reading_counts(&self, settings: JudgeSettings) -> (usize, usize) {
        let tau = signed(self.judge_tau_mebibytes(settings));
        let readable: Vec<i64> =
            self.samples.iter().filter_map(|sample| sample.used_mebibytes).map(|used| self.lease_remainder_of(used)).collect();
        (readable.iter().filter(|remainder| **remainder > tau).count(), readable.len())
    }

    /// 窗口里 R_lease 最大的那一份（t0 之后的峰值）。
    fn peak_remainder_mebibytes(&self) -> Option<i64> {
        self.samples.iter().filter_map(|sample| sample.used_mebibytes).map(|used| self.lease_remainder_of(used)).max()
    }

    /// Q3c 留着的块：窗口末分配器 reserved 减设备起好时的。
    fn reserved_kept_bytes(&self) -> Option<i64> {
        Some(signed(self.allocator_end?.reserved_bytes) - signed(self.allocator_device?.reserved_bytes))
    }

    /// Q3c 没还的分配：窗口末 allocated 减设备起好时的，并存的长借据那张缓冲也减掉。
    fn unreturned_allocation_bytes(&self) -> Option<i64> {
        let coexisting_bytes = signed(self.job.cell.coexisting_mebibytes() * MEBIBYTE);
        Some(signed(self.allocator_end?.allocated_bytes) - signed(self.allocator_device?.allocated_bytes) - coexisting_bytes)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Presence {
    Absent,
    Present,
    Unstable,
}

impl Presence {
    fn label(self) -> &'static str {
        match self {
            Presence::Absent => "absent",
            Presence::Present => "present",
            Presence::Unstable => "unstable",
        }
    }
}

/// 每轮一个 (值, τ)：5 轮都不大于各自的 τ 记不含，都大于记含，否则不稳定。空表也是不稳定（没有轮，不能说不含）。
fn classify_presence(measurements: &[(i64, u64)]) -> Presence {
    if measurements.is_empty() {
        return Presence::Unstable;
    }
    let exceeding = measurements.iter().filter(|(value, tau)| *value > signed(*tau)).count();
    if exceeding == 0 {
        Presence::Absent
    } else if exceeding == measurements.len() {
        Presence::Present
    } else {
        Presence::Unstable
    }
}

/// 同一轮同一栏里有两笔（第 2、3 笔）时取大的那一笔；平手取靠前的。
fn pick_larger<'leases, Key: Ord>(
    leases: &[&'leases LeaseRecord],
    key: impl Fn(&LeaseRecord) -> Option<Key>,
) -> Option<&'leases LeaseRecord> {
    leases
        .iter()
        .copied()
        .filter_map(|lease| key(lease).map(|measured| (measured, lease)))
        .reduce(|best, candidate| if candidate.0 > best.0 { candidate } else { best })
        .map(|(_, lease)| lease)
}

fn join_numbers<Number: ToString>(values: &[Number]) -> String {
    if values.is_empty() {
        return "none".to_string();
    }
    values.iter().map(ToString::to_string).collect::<Vec<_>>().join(";")
}

/// 每轮在这一栏里取大的那一笔借据（`rank` 定大小），交出 `output` 要的那几项。
fn per_round_larger<Key: Ord, Output>(
    rounds: &[(u32, &RoundData)],
    column: Column,
    rank: impl Fn(&LeaseRecord) -> Option<Key>,
    output: impl Fn(&LeaseRecord) -> Output,
) -> Vec<Output> {
    rounds
        .iter()
        .filter_map(|(_, data)| {
            let leases = data.column_leases(column);
            pick_larger(&leases, &rank).map(&output)
        })
        .collect()
}

struct ColumnVerdict {
    cell: CellKey,
    column: Column,
    rounds_used: usize,
    wait_token: &'static str,
    judged_wait_microseconds: Vec<i64>,
    wait_microseconds: Vec<u64>,
    submit_to_poll_microseconds: Vec<u64>,
    work_microseconds: Vec<u64>,
    wait_never_returned: bool,
    next_presence: Presence,
    next_measurements: Vec<(i64, u64)>,
    clear_token: &'static str,
    clear_values: Vec<TimeToClear>,
    positive_counts: Vec<(usize, usize)>,
    peak_remainders: Vec<i64>,
    end_presence: Presence,
    end_measurements: Vec<(i64, u64)>,
    overtake_presence: Presence,
    overtake_measurements: Vec<(i64, u64)>,
    blocks_token: &'static str,
    reserved_kept_bytes: Vec<i64>,
    unreturned_bytes: Vec<i64>,
}

fn wait_cost_token(cell: CellKey, judged: &[i64], never_returned: bool) -> &'static str {
    let all_at_most = |limit: u64| judged.iter().all(|value| *value <= signed(limit));
    let all_above = |limit: u64| judged.iter().all(|value| *value > signed(limit));
    if cell.arm.waits() && never_returned {
        "wait_never_returned"
    } else if !cell.arm.waits() {
        if all_at_most(POLL_NO_WAIT_LIMIT_MICROSECONDS) { "does_not_wait" } else { "reports_numbers" }
    } else if cell.liveness.is_in_flight() {
        if all_at_most(IN_FLIGHT_EXCESS_LIMIT_MICROSECONDS) {
            "only_the_work_itself"
        } else if all_above(IN_FLIGHT_EXCESS_LIMIT_MICROSECONDS) {
            "longer_than_the_work"
        } else {
            "unstable"
        }
    } else if all_at_most(DONE_WAIT_ZERO_OVERHEAD_LIMIT_MICROSECONDS) {
        "zero_overhead"
    } else if all_above(DONE_WAIT_ZERO_OVERHEAD_LIMIT_MICROSECONDS) {
        "fixed_overhead"
    } else {
        "unstable"
    }
}

fn judge_column(view: &CellView<'_>, column: Column, settings: JudgeSettings) -> ColumnVerdict {
    let rounds = &view.complete_rounds;
    let cell = view.cell;
    let judged_wait = |lease: &LeaseRecord| {
        if cell.arm.waits() { lease.judged_wait_microseconds() } else { signed(lease.wait_microseconds) }
    };
    let waits = per_round_larger(rounds, column, |lease| Some(judged_wait(lease)), |lease| {
        (judged_wait(lease), lease.wait_microseconds, lease.submit_to_poll_microseconds, lease.work_microseconds)
    });
    let judged_wait_microseconds: Vec<i64> = waits.iter().map(|entry| entry.0).collect();
    let wait_never_returned =
        view.all_rounds.iter().any(|(_, data)| data.leases.iter().any(|lease| lease.wait_failure.is_some()));
    let next_measurements = per_round_larger(
        rounds,
        column,
        |lease| lease.next_remainder_mebibytes().map(|value| value - signed(lease.judge_tau_mebibytes(settings))),
        |lease| (lease.next_remainder_mebibytes().unwrap_or(0), lease.judge_tau_mebibytes(settings)),
    );
    let clears = per_round_larger(
        rounds,
        column,
        |lease| Some(lease.time_to_clear(settings)),
        |lease| {
            (lease.time_to_clear(settings), lease.positive_reading_counts(settings), lease.peak_remainder_mebibytes().unwrap_or(0))
        },
    );
    let clear_values: Vec<TimeToClear> = clears.iter().map(|entry| entry.0).collect();
    let clear_token = if clear_values.iter().all(|value| *value == TimeToClear::Next) {
        "next_reading_already_absent"
    } else if clear_values.iter().any(|value| *value == TimeToClear::BeyondWindow) {
        "not_absent_within_window"
    } else {
        "absent_after_wait"
    };
    let end_measurements = per_round_larger(
        rounds,
        column,
        |lease| lease.end_device_remainder_mebibytes().map(|value| value - signed(lease.judge_tau_device_mebibytes(settings))),
        |lease| (lease.end_device_remainder_mebibytes().unwrap_or(0), lease.judge_tau_device_mebibytes(settings)),
    );
    let overtake_measurements = per_round_larger(
        rounds,
        column,
        |lease| lease.peak_over_size_mebibytes().map(|value| value - signed(lease.judge_tau_mebibytes(settings))),
        |lease| (lease.peak_over_size_mebibytes().unwrap_or(0), lease.judge_tau_mebibytes(settings)),
    );
    let blocks = per_round_larger(
        rounds,
        column,
        LeaseRecord::reserved_kept_bytes,
        |lease| (lease.reserved_kept_bytes().unwrap_or(0), lease.unreturned_allocation_bytes().unwrap_or(0)),
    );
    let reserved_kept_bytes: Vec<i64> = blocks.iter().map(|entry| entry.0).collect();
    let blocks_token = if blocks.is_empty() {
        "unreadable"
    } else if reserved_kept_bytes.iter().all(|value| *value > 0) {
        "wgpu_keeps_blocks"
    } else if reserved_kept_bytes.iter().all(|value| *value == 0) {
        "wgpu_does_not_keep_blocks"
    } else {
        "unstable"
    };
    ColumnVerdict {
        cell,
        column,
        rounds_used: rounds.len(),
        wait_token: wait_cost_token(cell, &judged_wait_microseconds, wait_never_returned),
        judged_wait_microseconds,
        wait_microseconds: waits.iter().map(|entry| entry.1).collect(),
        submit_to_poll_microseconds: waits.iter().map(|entry| entry.2).collect(),
        work_microseconds: waits.iter().map(|entry| entry.3).collect(),
        wait_never_returned,
        next_presence: classify_presence(&next_measurements),
        next_measurements,
        clear_token,
        clear_values,
        positive_counts: clears.iter().map(|entry| entry.1).collect(),
        peak_remainders: clears.iter().map(|entry| entry.2).collect(),
        end_presence: classify_presence(&end_measurements),
        end_measurements,
        overtake_presence: classify_presence(&overtake_measurements),
        overtake_measurements,
        blocks_token,
        reserved_kept_bytes,
        unreturned_bytes: blocks.iter().map(|entry| entry.1).collect(),
    }
}

fn measurement_values_text(measurements: &[(i64, u64)]) -> (String, String) {
    let values: Vec<i64> = measurements.iter().map(|measurement| measurement.0).collect();
    let taus: Vec<u64> = measurements.iter().map(|measurement| measurement.1).collect();
    (join_numbers(&values), join_numbers(&taus))
}

impl ColumnVerdict {
    /// 登记 Q1：「已做完」档等待臂的中位数比读一次显存（50 ms）还贵，记「乙的代价翻面候选」交用户。
    fn is_expensive_candidate(&self) -> bool {
        self.cell.arm.waits()
            && !self.cell.liveness.is_in_flight()
            && !self.wait_microseconds.is_empty()
            && median_of(&self.wait_microseconds) > DONE_WAIT_EXPENSIVE_LIMIT_MICROSECONDS
    }

    fn to_line(&self) -> String {
        let (next_values, next_taus) = measurement_values_text(&self.next_measurements);
        let (end_values, end_taus) = measurement_values_text(&self.end_measurements);
        let (overtake_values, overtake_taus) = measurement_values_text(&self.overtake_measurements);
        let clear_values = self.clear_values.iter().map(|value| value.text()).collect::<Vec<_>>().join(";");
        let positives: Vec<usize> = self.positive_counts.iter().map(|counts| counts.0).collect();
        let readable: Vec<usize> = self.positive_counts.iter().map(|counts| counts.1).collect();
        format!(
            "E7RESULT name=cell_verdict {} column={} rounds_used={} q1_wait={} q1_judged_microseconds={} q1_wait_microseconds={} \
             q1_submit_to_poll_microseconds={} q1_work_microseconds={} q1_wait_never_returned={} q1_expensive_candidate={} q2_next_reading={} \
             q2_remainders_mebibytes={} q2_taus_mebibytes={} q2c_time_to_clear={} q2c_values_microseconds={} \
             q2c_positive_readings={} q2c_readable_readings={} q2c_peak_remainder_mebibytes={} q3_end_of_window={} \
             q3_remainders_mebibytes={} q3_taus_mebibytes={} q3b_block_overtake={} q3b_values_mebibytes={} \
             q3b_taus_mebibytes={} q3c_wgpu_blocks={} q3c_reserved_kept_bytes={} q3c_unreturned_allocation_bytes={}",
            self.cell.fields_text(),
            self.column.label(),
            self.rounds_used,
            self.wait_token,
            join_numbers(&self.judged_wait_microseconds),
            join_numbers(&self.wait_microseconds),
            join_numbers(&self.submit_to_poll_microseconds),
            join_numbers(&self.work_microseconds),
            self.wait_never_returned,
            self.is_expensive_candidate(),
            self.next_presence.label(),
            next_values,
            next_taus,
            self.clear_token,
            clear_values,
            join_numbers(&positives),
            join_numbers(&readable),
            join_numbers(&self.peak_remainders),
            self.end_presence.label(),
            end_values,
            end_taus,
            self.overtake_presence.label(),
            overtake_values,
            overtake_taus,
            self.blocks_token,
            join_numbers(&self.reserved_kept_bytes),
            join_numbers(&self.unreturned_bytes),
        )
    }
}

/// 一次判的全部输出：逐格逐栏的行，加上汇总用的各格结论。
struct JudgedProduct {
    lines: Vec<String>,
    columns: Vec<ColumnVerdict>,
    void_cells: Vec<CellKey>,
    hold_control_failed: bool,
    /// 登记 F6：因为「在飞」档其实没在飞（K9）而凑不够 5 个有效轮的格数。
    cells_void_for_work_not_in_flight: usize,
}

fn columns_of_cell(cell: CellKey) -> &'static [Column] {
    if cell.lease_count() == 1 { &[Column::First] } else { &[Column::First, Column::Later] }
}

fn judge_product(rounds: &ProductRounds, settings: JudgeSettings) -> JudgedProduct {
    let mut judged = JudgedProduct {
        lines: Vec::new(),
        columns: Vec::new(),
        void_cells: Vec::new(),
        hold_control_failed: false,
        cells_void_for_work_not_in_flight: 0,
    };
    let required = usize::try_from(ROUNDS_REQUIRED_PER_CELL).expect("轮数装得进 usize");
    for view in group_into_cells(rounds) {
        if view.cell.is_hold_control() {
            judged.hold_control_failed |= view.all_rounds.iter().any(|(_, data)| {
                data.leases.iter().any(|lease| lease.void_reasons.contains(&VoidReason::HoldControlDidNotHold))
            });
        }
        if view.complete_rounds.len() < required {
            let reasons: Vec<String> = view
                .all_rounds
                .iter()
                .filter(|(_, data)| !data.is_complete(view.cell.lease_count()))
                .map(|(round, data)| format!("round{round}={}", data.void_explanation()))
                .collect();
            judged.lines.push(format!(
                "E7RESULT name=cell_void {} rounds_complete={} rounds_total={} reasons={}",
                view.cell.fields_text(),
                view.complete_rounds.len(),
                view.all_rounds.len(),
                reasons.join("|")
            ));
            judged.void_cells.push(view.cell);
            let work_not_in_flight = view.all_rounds.iter().any(|(_, data)| {
                data.leases.iter().any(|lease| lease.void_reasons.contains(&VoidReason::WorkNotInFlightAtPoll))
            });
            if work_not_in_flight {
                judged.cells_void_for_work_not_in_flight += 1;
            }
            continue;
        }
        for column in columns_of_cell(view.cell) {
            let verdict = judge_column(&view, *column, settings);
            judged.lines.push(verdict.to_line());
            judged.columns.push(verdict);
        }
    }
    judged
}

// ============================== 汇总：失败条款与几何敏感性（登记第八、十节） ==============================

fn tri_state_text(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "true",
        Some(false) => "false",
        None => "not_run",
    }
}

/// 一组条件全成立记真，有一条不成立记假，一条都没有（没跑到）记 `None`。
fn all_hold_or_not_run(conditions: impl Iterator<Item = bool>) -> Option<bool> {
    let collected: Vec<bool> = conditions.collect();
    if collected.is_empty() { None } else { Some(collected.iter().all(|holds| *holds)) }
}

fn is_wait_arm_outside_the_control(column: &ColumnVerdict) -> bool {
    column.cell.arm.waits() && !column.cell.is_hold_control()
}

fn same_place_other_arm<'columns>(
    columns: &'columns [ColumnVerdict],
    column: &ColumnVerdict,
    arm: Arm,
) -> Option<&'columns ColumnVerdict> {
    columns.iter().find(|other| {
        other.cell == CellKey { arm, ..column.cell } && other.column == column.column
    })
}

/// 同一个 S、同一张卡、同一条臂、同一档活：「并存的长借据」那一格与主格子那一格逐栏对（登记第八节）。
fn geometry_pairs<'columns>(columns: &'columns [ColumnVerdict]) -> Vec<(&'columns ColumnVerdict, &'columns ColumnVerdict)> {
    columns
        .iter()
        .filter(|column| column.cell.group == Group::CoexistingLongLease)
        .filter_map(|coexisting| {
            columns
                .iter()
                .find(|main| {
                    main.cell == CellKey { group: Group::MainGrid, ..coexisting.cell } && main.column == coexisting.column
                })
                .map(|main| (main, coexisting))
        })
        .collect()
}

fn geometry_pair_line(main: &ColumnVerdict, coexisting: &ColumnVerdict) -> String {
    let next_same = main.next_presence == coexisting.next_presence;
    let end_same = main.end_presence == coexisting.end_presence;
    format!(
        "E7RESULT name=geometry_pair {} column={} q2_main_grid={} q2_coexisting={} q3_main_grid={} q3_coexisting={} same={}",
        coexisting.cell.fields_text(),
        coexisting.column.label(),
        main.next_presence.label(),
        coexisting.next_presence.label(),
        main.end_presence.label(),
        coexisting.end_presence.label(),
        next_same && end_same
    )
}

/// 把 τ 换成落在读数之间的一个值，数有几栏的判定翻了；新 τ 取全部有效借据里「含」着的 R_next（大于各自 τ 的）的中位数，
/// 一条含着的都没有就取 0（把为 1、2 的读数也翻成含）。
fn count_columns_flipped_by_moving_tau(rounds: &ProductRounds, original: &JudgedProduct) -> usize {
    let mut positives: Vec<i64> = Vec::new();
    for view in group_into_cells(rounds) {
        for (_, data) in &view.complete_rounds {
            positives.extend(
                data.leases
                    .iter()
                    .filter(|lease| !lease.is_void())
                    .filter_map(|lease| lease.next_remainder_mebibytes().filter(|value| *value > signed(lease.tau_mebibytes()))),
            );
        }
    }
    positives.sort_unstable();
    let moved_tau = positives.get(positives.len() / 2).copied().unwrap_or(0).max(0);
    let moved = judge_product(rounds, JudgeSettings { tau_override_mebibytes: Some(u64::try_from(moved_tau).expect("非负")) });
    original
        .columns
        .iter()
        .zip(moved.columns.iter())
        .filter(|(before, after)| {
            before.next_presence != after.next_presence
                || before.end_presence != after.end_presence
                || before.overtake_presence != after.overtake_presence
                || before.clear_token != after.clear_token
        })
        .count()
}

fn verdict_line(scope: &str, judged: &JudgedProduct, flipped_columns: usize, synthetic_flips: bool) -> String {
    let columns = &judged.columns;
    let wait_columns = || columns.iter().filter(|column| is_wait_arm_outside_the_control(column));
    let hold_columns: Vec<&ColumnVerdict> = columns.iter().filter(|column| column.cell.is_hold_control()).collect();
    let hold_control_holds = if judged.hold_control_failed {
        Some(false)
    } else if hold_columns.is_empty() {
        None
    } else {
        Some(hold_columns.iter().all(|column| column.next_presence == Presence::Present))
    };
    let next_absent = all_hold_or_not_run(
        wait_columns().map(|column| column.next_presence == Presence::Absent && column.clear_token == "next_reading_already_absent"),
    );
    let end_absent = all_hold_or_not_run(wait_columns().map(|column| column.end_presence == Presence::Absent));
    let always_returns = all_hold_or_not_run(wait_columns().map(|column| !column.wait_never_returned));
    let poll_also_absent = all_hold_or_not_run(
        wait_columns()
            .filter(|column| column.next_presence == Presence::Absent)
            .filter_map(|column| same_place_other_arm(columns, column, Arm::Poll))
            .map(|poll_column| poll_column.next_presence == Presence::Absent),
    );
    let long_window_clears = all_hold_or_not_run(
        columns
            .iter()
            .filter(|column| column.cell.group == Group::LongWindow && column.cell.arm.waits())
            .map(|column| column.clear_token != "not_absent_within_window"),
    );
    let geometry_same = all_hold_or_not_run(geometry_pairs(columns).into_iter().map(|(main, coexisting)| {
        main.next_presence == coexisting.next_presence && main.end_presence == coexisting.end_presence
    }));
    format!(
        "E7RESULT name=verdict scope={scope} columns_judged={} cells_void={} hold_control_holds={} \
         wait_arm_next_reading_absent_everywhere={} wait_arm_end_of_window_not_kept_everywhere={} wait_always_returns={} \
         poll_arm_also_absent_where_wait_arm_absent={} in_flight_cells_void_for_work_not_in_flight={} \
         long_window_clear_within_window={} geometry_same_as_main_grid={} done_wait_expensive_candidate_columns={} \
         discrimination_real_data_flipped_columns={flipped_columns} discrimination_synthetic_flips={synthetic_flips}",
        columns.len(),
        judged.void_cells.len(),
        tri_state_text(hold_control_holds),
        tri_state_text(next_absent),
        tri_state_text(end_absent),
        tri_state_text(always_returns),
        tri_state_text(poll_also_absent),
        judged.cells_void_for_work_not_in_flight,
        tri_state_text(long_window_clears),
        tri_state_text(geometry_same),
        columns.iter().filter(|column| column.is_expensive_candidate()).count(),
    )
}

// ===== 以下是真卡的那一份 CardHost（挂 wgpu，单测与 `checks` 不碰） =====

// ============================== 真卡：wgpu 30.0.1，Vulkan ==============================

/// 空转内核：单线程做 `iteration_count` 次相依的整数运算，结果写进第一张缓冲的第一个字，编译器消不掉。
/// `iteration_count` 是管线常量，标定时按目标时长调。另一个入口只写一个字（登记借法 P2 的小内核）。
const WORK_SHADER_SOURCE: &str = r#"
override iteration_count: u32 = 1u;

@group(0) @binding(0) var<storage, read_write> output_words: array<u32>;

@compute @workgroup_size(1)
fn spin() {
    var state: u32 = 12345u;
    for (var step: u32 = 0u; step < iteration_count; step = step + 1u) {
        state = state * 1664525u + 1013904223u;
        state = state ^ (state >> 11u);
    }
    output_words[0] = state;
}

@compute @workgroup_size(1)
fn write_one_word() {
    output_words[0] = 1u;
}
"#;

const NVIDIA_VENDOR_IDENTIFIER: u32 = 0x10DE;
const CALIBRATION_BUFFER_BYTES: u64 = MEBIBYTE;
const FIRST_CALIBRATION_GUESS_ITERATIONS: u32 = 2_000_000;
const LEASE_PIECE_USAGE: wgpu::BufferUsages = wgpu::BufferUsages::STORAGE.union(wgpu::BufferUsages::COPY_DST);

fn wait_timeout() -> std::time::Duration {
    std::time::Duration::from_secs(WAIT_TIMEOUT_SECONDS)
}

fn build_work_pipeline(
    device: &wgpu::Device,
    module: &wgpu::ShaderModule,
    entry_point: &str,
    iteration_count: u32,
) -> wgpu::ComputePipeline {
    let constants = [("iteration_count", f64::from(iteration_count))];
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("e174-work-pipeline"),
        layout: None,
        module,
        entry_point: Some(entry_point),
        compilation_options: wgpu::PipelineCompilationOptions { constants: &constants, zero_initialize_workgroup_memory: true },
        cache: None,
    })
}

/// 清每张缓冲，再在第一张的头 256 字节上派一次内核，一次提交。
fn submit_clear_and_kernel(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pieces: &[wgpu::Buffer],
    pipeline: &wgpu::ComputePipeline,
) -> wgpu::SubmissionIndex {
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("e174-encoder") });
    for piece in pieces {
        encoder.clear_buffer(piece, 0, None);
    }
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("e174-bind-group"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: &pieces[0],
                offset: 0,
                size: std::num::NonZeroU64::new(SPIN_KERNEL_BINDING_BYTES),
            }),
        }],
    });
    {
        let mut compute_pass =
            encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("e174-pass"), timestamp_writes: None });
        compute_pass.set_pipeline(pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);
        compute_pass.dispatch_workgroups(1, 1, 1);
    }
    queue.submit(Some(encoder.finish()))
}

fn create_piece(device: &wgpu::Device, bytes: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("e174-lease-piece"),
        size: bytes,
        usage: LEASE_PIECE_USAGE,
        mapped_at_creation: false,
    })
}

struct CalibrationResult {
    iteration_count: u32,
    work_microseconds: u64,
    is_in_band: bool,
}

/// 登记第五节「活的标定」：用一张 1 MiB 的缓冲派空转内核，按目标时长调循环次数，同一条管线连量 3 次都在
/// 目标的 0.8–1.25 倍之间为止；记这 3 次的中位数为这个进程的 W。试了 `CALIBRATION_ATTEMPT_LIMIT` 次还没调到记不在带内。
fn calibrate_spin_kernel(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    module: &wgpu::ShaderModule,
    target_microseconds: u64,
) -> CalibrationResult {
    let calibration_pieces = vec![create_piece(device, CALIBRATION_BUFFER_BYTES)];
    let lower = target_microseconds * CALIBRATION_BAND_LOWER_PERCENT / 100;
    let upper = target_microseconds * CALIBRATION_BAND_UPPER_PERCENT / 100;
    let mut iteration_count = FIRST_CALIBRATION_GUESS_ITERATIONS;
    let mut result = CalibrationResult { iteration_count, work_microseconds: 0, is_in_band: false };
    for _attempt in 0..CALIBRATION_ATTEMPT_LIMIT {
        let pipeline = build_work_pipeline(device, module, "spin", iteration_count);
        let mut durations = Vec::with_capacity(CALIBRATION_CONSECUTIVE_HITS);
        for _measurement in 0..CALIBRATION_CONSECUTIVE_HITS {
            let started = std::time::Instant::now();
            let submission = submit_clear_and_kernel(device, queue, &calibration_pieces, &pipeline);
            device
                .poll(wgpu::PollType::Wait { submission_index: Some(submission), timeout: Some(wait_timeout()) })
                .expect("标定时 Wait 失败");
            durations.push(u64::try_from(started.elapsed().as_micros()).expect("时长装得进 u64"));
        }
        let median = median_of(&durations);
        result = CalibrationResult {
            iteration_count,
            work_microseconds: median,
            is_in_band: durations.iter().all(|duration| (lower..=upper).contains(duration)),
        };
        if result.is_in_band {
            break;
        }
        let scaled = u128::from(iteration_count) * u128::from(target_microseconds) / u128::from(median.max(1));
        iteration_count = u32::try_from(scaled.clamp(1, u128::from(u32::MAX))).expect("夹在 u32 范围内");
    }
    result
}

struct WgpuCardHost {
    card: Card,
    started: std::time::Instant,
    /// 实例要活到进程结束，设备才不会在用着的时候被收走。
    _instance: wgpu::Instance,
    adapter_description: String,
    device: wgpu::Device,
    queue: wgpu::Queue,
    shader_module: wgpu::ShaderModule,
    spin_pipeline: wgpu::ComputePipeline,
    small_pipeline: wgpu::ComputePipeline,
    pieces: Vec<wgpu::Buffer>,
    coexisting: Vec<wgpu::Buffer>,
    submissions: Vec<wgpu::SubmissionIndex>,
}

/// 登记第五节：后端只用 Vulkan，`required_limits` 取 `adapter.limits()`，`memory_hints` 取 `Performance`。
fn open_wgpu_card(card: Card) -> Result<WgpuCardHost, String> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });
    let adapter = pollster::block_on(instance.enumerate_adapters(wgpu::Backends::VULKAN))
        .into_iter()
        .find(|adapter| {
            let information = adapter.get_info();
            information.vendor == NVIDIA_VENDOR_IDENTIFIER
                && information.device_type == wgpu::DeviceType::DiscreteGpu
                && bus_without_domain(&information.device_pci_bus_id) == card.bus_without_domain()
        })
        .ok_or_else(|| format!("Vulkan 适配器里没有总线 {} 那张 NVIDIA 独显", card.bus_without_domain()))?;
    let information = adapter.get_info();
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("e174-lease-drop-wait-memory-return"),
        required_features: wgpu::Features::empty(),
        required_limits: adapter.limits(),
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
    }))
    .map_err(|error| format!("request_device 失败：{error:?}"))?;
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("e174-work-shader"),
        source: wgpu::ShaderSource::Wgsl(WORK_SHADER_SOURCE.into()),
    });
    let spin_pipeline = build_work_pipeline(&device, &shader_module, "spin", FIRST_CALIBRATION_GUESS_ITERATIONS);
    let small_pipeline = build_work_pipeline(&device, &shader_module, "write_one_word", 1);
    Ok(WgpuCardHost {
        card,
        started: std::time::Instant::now(),
        _instance: instance,
        adapter_description: format!(
            "name={:?} vendor=0x{:X} device=0x{:X} driver={:?} driver_info={:?} pci_bus_id={:?}",
            information.name, information.vendor, information.device, information.driver, information.driver_info, information.device_pci_bus_id
        ),
        device,
        queue,
        shader_module,
        spin_pipeline,
        small_pipeline,
        pieces: Vec::new(),
        coexisting: Vec::new(),
        submissions: Vec::new(),
    })
}

fn run_nvidia_smi(arguments: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new("nvidia-smi")
        .args(arguments)
        .output()
        .map_err(|error| format!("起不了 nvidia-smi：{error}"))?;
    if !output.status.success() {
        return Err(format!("nvidia-smi 退 {:?}：{}", output.status.code(), String::from_utf8_lossy(&output.stderr)));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

impl CardHost for WgpuCardHost {
    fn card(&self) -> Card {
        self.card
    }

    fn process_identifier(&self) -> u32 {
        std::process::id()
    }

    fn now_microseconds(&self) -> u64 {
        u64::try_from(self.started.elapsed().as_micros()).expect("时长装得进 u64")
    }

    fn read_report_text(&mut self) -> Result<String, String> {
        run_nvidia_smi(&PROCESS_REPORT_ARGUMENTS)
    }

    fn read_compute_applications_text(&mut self) -> Result<String, String> {
        run_nvidia_smi(&COMPUTE_APPLICATIONS_ARGUMENTS)
    }

    fn read_allocator_figures(&mut self) -> Option<AllocatorFigures> {
        self.device.generate_allocator_report().map(|report| AllocatorFigures {
            reserved_bytes: report.total_reserved_bytes,
            allocated_bytes: report.total_allocated_bytes,
        })
    }

    fn create_lease_pieces(&mut self, piece_byte_lengths: &[u64]) {
        for bytes in piece_byte_lengths {
            self.pieces.push(create_piece(&self.device, *bytes));
        }
    }

    fn submit_lease_work(&mut self, kind: WorkKind) -> SubmissionToken {
        let pipeline = match kind {
            WorkKind::Spin => &self.spin_pipeline,
            WorkKind::SmallKernel => &self.small_pipeline,
        };
        let submission = submit_clear_and_kernel(&self.device, &self.queue, &self.pieces, pipeline);
        self.submissions.push(submission);
        SubmissionToken(u64::try_from(self.submissions.len() - 1).expect("提交号装得进 u64"))
    }

    fn poll_device(&mut self, mode: PollMode) -> Result<(), String> {
        let poll_type = match mode {
            PollMode::Poll => wgpu::PollType::Poll,
            PollMode::WaitLatest => wgpu::PollType::Wait { submission_index: None, timeout: Some(wait_timeout()) },
            PollMode::WaitOwnSubmission(token) => wgpu::PollType::Wait {
                submission_index: Some(self.submissions[usize::try_from(token.0).expect("提交号装得进 usize")].clone()),
                timeout: Some(wait_timeout()),
            },
        };
        self.device.poll(poll_type).map(|_status| ()).map_err(|error| format!("{error:?}"))
    }

    fn drop_lease_pieces(&mut self) {
        self.pieces.clear();
    }

    fn create_coexisting_buffer(&mut self, bytes: u64) {
        let buffer = create_piece(&self.device, bytes);
        let coexisting_pieces = [buffer];
        let submission = submit_clear_and_kernel(&self.device, &self.queue, &coexisting_pieces, &self.small_pipeline);
        self.device
            .poll(wgpu::PollType::Wait { submission_index: Some(submission), timeout: Some(wait_timeout()) })
            .expect("并存的长借据那张缓冲清完时 Wait 失败");
        self.coexisting.extend(coexisting_pieces);
    }
}

// ============================== 第二段：同一设备上另一条线程接着提交 ==============================

/// 登记第五节第二段：线程 B 在同一个设备上一次接一次提交空转 400 ms 的活；起好之后才放行读数，B 不再建缓冲。
struct BackgroundLoad {
    should_stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    handle: Option<std::thread::JoinHandle<u64>>,
}

fn start_background_load(host: &WgpuCardHost, iteration_count: u32) -> BackgroundLoad {
    let device = host.device.clone();
    let queue = host.queue.clone();
    let module = host.shader_module.clone();
    let should_stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let thread_should_stop = std::sync::Arc::clone(&should_stop);
    let (ready_sender, ready_receiver) = std::sync::mpsc::channel::<()>();
    let handle = std::thread::spawn(move || {
        let pipeline = build_work_pipeline(&device, &module, "spin", iteration_count);
        let background_pieces = vec![create_piece(&device, CALIBRATION_BUFFER_BYTES)];
        ready_sender.send(()).expect("主线程在等 B 起好");
        let mut submissions_done: u64 = 0;
        while !thread_should_stop.load(std::sync::atomic::Ordering::Relaxed) {
            let submission = submit_clear_and_kernel(&device, &queue, &background_pieces, &pipeline);
            device
                .poll(wgpu::PollType::Wait { submission_index: Some(submission), timeout: Some(wait_timeout()) })
                .expect("线程 B 的 Wait 失败");
            submissions_done += 1;
        }
        submissions_done
    });
    ready_receiver.recv().expect("线程 B 起好之前就退了");
    BackgroundLoad { should_stop, handle: Some(handle) }
}

impl BackgroundLoad {
    fn stop_and_count_submissions(&mut self) -> u64 {
        self.should_stop.store(true, std::sync::atomic::Ordering::Relaxed);
        self.handle.take().map_or(0, |handle| handle.join().expect("线程 B 没有 panic"))
    }
}

// ============================== 子进程入口 ==============================

/// 真卡上的一个装置进程。
fn run_wgpu_child(job: &ChildJob, raw_output_path: Option<&str>) -> Result<(), String> {
    let mut host = open_wgpu_card(job.cell.card)?;
    let calibration = calibrate_spin_kernel(&host.device, &host.queue, &host.shader_module, job.cell.liveness.work_target_microseconds());
    host.spin_pipeline = build_work_pipeline(&host.device, &host.shader_module, "spin", calibration.iteration_count);
    let mut background = if job.cell.has_background_load() {
        let background_calibration =
            calibrate_spin_kernel(&host.device, &host.queue, &host.shader_module, BACKGROUND_WORK_TARGET_MICROSECONDS);
        Some(start_background_load(&host, background_calibration.iteration_count))
    } else {
        None
    };
    let mut raw_log = RawLog::default();
    let outcome = run_child_procedure(&mut host, job, calibration.work_microseconds, calibration.is_in_band, &mut raw_log);
    let background_submissions = background.as_mut().map(BackgroundLoad::stop_and_count_submissions);
    let extra = format!(
        "background_submissions={} adapter={}",
        background_submissions.map_or_else(|| "none".to_string(), |count| count.to_string()),
        without_whitespace(&host.adapter_description)
    );
    write_child_result(job, &outcome, (calibration.work_microseconds, calibration.is_in_band), &extra, &raw_log, raw_output_path)
}

// ===== 以上是真卡的那一份 CardHost =====

/// 子进程的结果行写 stdout（父进程等它退出后整份读），原文写 `--raw-output` 给的文件。
fn write_child_result(
    job: &ChildJob,
    outcome: &ChildOutcome,
    calibration: (u64, bool),
    extra: &str,
    raw_log: &RawLog,
    raw_output_path: Option<&str>,
) -> Result<(), String> {
    let (work_microseconds, calibration_is_in_band) = calibration;
    for lease in &outcome.leases {
        println!("{}", lease.to_line());
    }
    println!("{}", child_done_line(job, outcome, work_microseconds, calibration_is_in_band, extra));
    if let Some(path) = raw_output_path {
        std::fs::write(path, raw_log_text(job, raw_log)).map_err(|error| format!("写不了原文 {path}：{error}"))?;
    }
    Ok(())
}

/// 模型卡上的一个子进程：给 `run --simulated` 用，整条父子流水线（续跑、补轮、原文压缩、F1 停机）不起卡也走得通。
fn run_simulated_child(job: &ChildJob, raw_output_path: Option<&str>) -> Result<(), String> {
    let work_microseconds = job.cell.liveness.work_target_microseconds();
    let mut host = SimulatedCardHost::new(job.cell.card, work_microseconds);
    let mut raw_log = RawLog::default();
    let outcome = run_child_procedure(&mut host, job, work_microseconds, true, &mut raw_log);
    let extra = "background_submissions=none adapter=simulated";
    write_child_result(job, &outcome, (work_microseconds, true), extra, &raw_log, raw_output_path)
}

/// `child` 子命令：一个装置进程。参数是 `ChildJob::child_arguments` 那几个，再加 `--raw-output 文件`，可选 `--simulated`。
fn run_child(arguments: &[String]) -> Result<(), String> {
    let mut job_arguments: Vec<String> = Vec::new();
    let mut raw_output_path: Option<String> = None;
    let mut simulated = false;
    let mut index = 0;
    while index < arguments.len() {
        if arguments[index] == "--simulated" {
            simulated = true;
            index += 1;
            continue;
        }
        let value = arguments.get(index + 1).ok_or_else(|| format!("{} 后面没有值", arguments[index]))?;
        if arguments[index] == "--raw-output" {
            raw_output_path = Some(value.clone());
        } else {
            job_arguments.push(arguments[index].clone());
            job_arguments.push(value.clone());
        }
        index += 2;
    }
    let job = ChildJob::from_child_arguments(&job_arguments)?;
    if simulated {
        run_simulated_child(&job, raw_output_path.as_deref())
    } else {
        run_wgpu_child(&job, raw_output_path.as_deref())
    }
}

fn raw_log_text(job: &ChildJob, raw_log: &RawLog) -> String {
    let mut text = format!("######## {}\n", job.key_text());
    for (label, report) in &raw_log.entries {
        text.push_str(&format!("==== {label}\n{report}\n"));
    }
    text
}

fn child_done_line(job: &ChildJob, outcome: &ChildOutcome, work_microseconds: u64, calibration_is_in_band: bool, extra: &str) -> String {
    format!(
        "E7RESULT name=child_done {} leases_run={} work_microseconds={work_microseconds} calibration_in_band={calibration_is_in_band} \
         child_void={} {extra}",
        job.key_text(),
        outcome.leases.len(),
        void_reasons_text(&outcome.child_void),
    )
}

// ============================== 清单：格、次序、续跑（登记第五节「次序」「续跑、分段」） ==============================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Segment {
    First,
    Second,
}

impl Segment {
    fn label(self) -> &'static str {
        match self {
            Segment::First => "first",
            Segment::Second => "second",
        }
    }

    fn from_label(label: &str) -> Option<Segment> {
        [Segment::First, Segment::Second].into_iter().find(|segment| segment.label() == label)
    }
}

/// 第一段的全部格。在飞·中是跑前登记「十二、修订」补的一条臂，每个登记里用在飞·轻的组都另有一份在飞·中。
fn first_segment_cells() -> Vec<CellKey> {
    let mut cells = Vec::new();
    for card in Card::EVERY_CARD {
        for arm in [Arm::WaitLatest, Arm::Poll] {
            let mut push = |group: Group, size_mebibytes: u64, liveness: Liveness, method: Method| {
                cells.push(CellKey { group, card, size_mebibytes, liveness, method, arm });
            };
            for size in HOLD_CONTROL_SIZES_MEBIBYTES {
                for liveness in [Liveness::InFlightMedium, Liveness::InFlightLight] {
                    push(Group::HoldControl, size, liveness, Method::OneBuild);
                }
            }
            for size in LEASE_SIZES_MEBIBYTES {
                for liveness in Liveness::EVERY_LIVENESS {
                    push(Group::MainGrid, size, liveness, Method::OneBuild);
                }
            }
            for size in PASS_BY_PASS_SIZES_MEBIBYTES {
                push(Group::PassByPassGrid, size, Liveness::Done, Method::PassByPass);
            }
            for size in COEXISTING_SIZES_MEBIBYTES {
                for liveness in [Liveness::InFlightMedium, Liveness::InFlightLight] {
                    push(Group::CoexistingLongLease, size, liveness, Method::OneBuild);
                }
            }
        }
        for size in LONG_WINDOW_SIZES_MEBIBYTES {
            for liveness in [Liveness::InFlightMedium, Liveness::InFlightLight] {
                cells.push(CellKey {
                    group: Group::LongWindow,
                    card,
                    size_mebibytes: size,
                    liveness,
                    method: Method::OneBuild,
                    arm: Arm::WaitLatest,
                });
            }
        }
    }
    cells
}

/// 第二段的全部格：同一设备几条线程共用，三条臂。
fn second_segment_cells() -> Vec<CellKey> {
    let mut cells = Vec::new();
    for card in Card::EVERY_CARD {
        for size_mebibytes in SHARED_DEVICE_SIZES_MEBIBYTES {
            for liveness in [Liveness::InFlightMedium, Liveness::InFlightLight] {
                for arm in [Arm::WaitLatest, Arm::WaitOwnSubmission, Arm::Poll] {
                    cells.push(CellKey {
                        group: Group::SharedDevice,
                        card,
                        size_mebibytes,
                        liveness,
                        method: Method::OneBuild,
                        arm,
                    });
                }
            }
        }
    }
    cells
}

fn cells_of_segment(segment: Segment, skip_light: bool) -> Vec<CellKey> {
    let cells = match segment {
        Segment::First => first_segment_cells(),
        Segment::Second => second_segment_cells(),
    };
    cells.into_iter().filter(|cell| !(skip_light && cell.liveness == Liveness::InFlightLight)).collect()
}

/// 阶段：不用在飞·轻的先跑，「扣住不丢」排最前（它不过整轮作废）；在飞·轻最后，它的 K9 在登记的数下必红，主 agent 可以不等它。
fn stage_of(cell: CellKey) -> u32 {
    let group_rank = match cell.group {
        Group::HoldControl => 0,
        Group::MainGrid => 1,
        Group::PassByPassGrid => 2,
        Group::CoexistingLongLease => 3,
        Group::LongWindow => 4,
        Group::SharedDevice => 5,
    };
    if cell.liveness == Liveness::InFlightLight { 10 + group_rank } else { group_rank }
}

/// 种子 174 的确定性乱序：同一个键永远排同一个位置，续跑时不受已经跑了哪些的影响。
fn shuffle_rank(slot_text: &str) -> u64 {
    let mut state = SHUFFLE_SEED;
    for byte in slot_text.bytes() {
        state = state.wrapping_mul(0x100_0000_01B3) ^ u64::from(byte);
    }
    state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    state = (state ^ (state >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    state = (state ^ (state >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    state ^ (state >> 31)
}

/// 次序键：阶段 → 乱序后的位置 → 两张卡交替（先后由位置的一位定）→ 两条臂（三条臂）按轮号轮换。
fn job_order_key(job: &ChildJob, arm_count: usize) -> (u32, u64, u32, usize) {
    let cell = &job.cell;
    let slot_text = format!(
        "{} {} {} {} round={}",
        cell.group.label(),
        cell.size_mebibytes,
        cell.liveness.label(),
        cell.method.label(),
        job.round
    );
    let rank = shuffle_rank(&slot_text);
    let card_index = Card::EVERY_CARD.iter().position(|card| *card == cell.card).expect("卡在表里");
    let card_first = usize::try_from((rank >> 1) & 1).expect("一位");
    let arms: &[Arm] = if arm_count == 3 {
        &[Arm::WaitLatest, Arm::WaitOwnSubmission, Arm::Poll]
    } else {
        &[Arm::WaitLatest, Arm::Poll]
    };
    let arm_index = arms.iter().position(|arm| *arm == cell.arm).expect("臂在表里");
    let round_shift = usize::try_from(job.round).expect("轮号装得进 usize");
    (stage_of(*cell), rank, u32::try_from((card_index + card_first) % 2).expect("0 或 1"), (arm_index + round_shift) % arms.len())
}

/// 产物里已有的轮之外还要跑哪些：第 1..=5 轮没出现的；5 轮都出现了而有效的不足 5 个，补轮（轮号 6 起，一格最多补 3 次）。
fn pending_jobs(cells: &[CellKey], product: &ProductRounds, arm_count: usize) -> Vec<ChildJob> {
    let required = ROUNDS_REQUIRED_PER_CELL;
    let mut pending = Vec::new();
    for cell in cells {
        let present: Vec<u32> = product.keys().filter(|job| job.cell == *cell).map(|job| job.round).collect();
        let complete = product
            .iter()
            .filter(|(job, data)| job.cell == *cell && data.is_complete(cell.lease_count()))
            .count();
        let first_pass_missing: Vec<u32> = (1..=required).filter(|round| !present.contains(round)).collect();
        if !first_pass_missing.is_empty() {
            pending.extend(first_pass_missing.into_iter().map(|round| ChildJob { cell: *cell, round }));
            continue;
        }
        let refills_done = u32::try_from(present.iter().filter(|round| **round > required).count()).expect("轮数装得进 u32");
        let missing = required.saturating_sub(u32::try_from(complete).expect("轮数装得进 u32"));
        let refills_now = missing.min(MAXIMUM_REFILL_ROUNDS_PER_CELL.saturating_sub(refills_done));
        let last_round = present.iter().copied().max().unwrap_or(required);
        pending.extend((1..=refills_now).map(|offset| ChildJob { cell: *cell, round: last_round + offset }));
    }
    pending.sort_by_key(|job| job_order_key(job, arm_count));
    pending
}

// ============================== 父进程：按清单起子进程，续跑 ==============================

struct ParentOptions {
    segment: Segment,
    output_path: String,
    skip_light: bool,
    /// 子进程都跑在模型卡上，不起卡、不读 `nvidia-smi`：只用来试父子流水线。
    simulated: bool,
    /// 最多起这么多个子进程就停（不写 verdict 与 done）：冒烟用，续跑照常。
    limit_jobs: Option<usize>,
}

fn parse_parent_options(arguments: &[String]) -> Result<ParentOptions, String> {
    let mut segment: Option<Segment> = None;
    let mut output_path: Option<String> = None;
    let mut skip_light = false;
    let mut simulated = false;
    let mut limit_jobs: Option<usize> = None;
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--limit-jobs" => {
                let text = arguments.get(index + 1).ok_or("--limit-jobs 后面没有值")?;
                limit_jobs = Some(text.parse().map_err(|_| format!("--limit-jobs 不是整数：{text}"))?);
                index += 2;
            }
            "--skip-light" => {
                skip_light = true;
                index += 1;
            }
            "--simulated" => {
                simulated = true;
                index += 1;
            }
            "--segment" => {
                let label = arguments.get(index + 1).ok_or("--segment 后面没有值")?;
                segment = Some(Segment::from_label(label).ok_or_else(|| format!("--segment 认不出：{label}"))?);
                index += 2;
            }
            "--output" => {
                output_path = Some(arguments.get(index + 1).ok_or("--output 后面没有值")?.clone());
                index += 2;
            }
            other => return Err(format!("认不出的参数 {other}")),
        }
    }
    Ok(ParentOptions {
        segment: segment.ok_or("缺 --segment")?,
        output_path: output_path.ok_or("缺 --output")?,
        skip_light,
        simulated,
        limit_jobs,
    })
}

/// 登记第七节 B7、第十一节停机：驱动版本不是 595.84，停下，不作废也不当结果。
fn check_driver_version() -> Result<(), String> {
    let text = run_nvidia_smi(&["--query-gpu=driver_version", "--format=csv,noheader"])?;
    let versions: Vec<&str> = text.lines().map(str::trim).filter(|line| !line.is_empty()).collect();
    if versions.is_empty() || versions.iter().any(|version| *version != EXPECTED_DRIVER_VERSION) {
        return Err(format!("驱动版本不是 {EXPECTED_DRIVER_VERSION}：{versions:?}（登记第十一节停机）"));
    }
    Ok(())
}

fn append_to_file(path: &str, text: &str) -> Result<(), String> {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| format!("打不开 {path}：{error}"))?;
    file.write_all(text.as_bytes()).map_err(|error| format!("写不了 {path}：{error}"))
}

fn raw_archive_path(output_path: &str) -> String {
    format!("{output_path}.raw.gz")
}

/// 把子进程的原文压成一个 gzip 成员追加到 `.raw.gz`（gzip 成员可以首尾相连，`zcat` 读得整份）。
fn append_gzip_member(raw_file: &str, archive_path: &str) -> Result<(), String> {
    let archive = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(archive_path)
        .map_err(|error| format!("打不开 {archive_path}：{error}"))?;
    let status = std::process::Command::new("gzip")
        .args(["-9", "-n", "-c", raw_file])
        .stdout(archive)
        .status()
        .map_err(|error| format!("起不了 gzip：{error}"))?;
    if status.success() { Ok(()) } else { Err(format!("gzip 退 {:?}", status.code())) }
}

/// 起一个子进程，等它退出后整份读（不在转发输出的循环里打时间戳），交回要追加进产物的文本。
fn run_one_child(job: &ChildJob, raw_archive: &str, scratch_directory: &std::path::Path, simulated: bool) -> Result<String, String> {
    let raw_file = scratch_directory.join(format!("e174-raw-{}.txt", std::process::id()));
    let executable = std::env::current_exe().map_err(|error| format!("找不到自己的路径：{error}"))?;
    let mut command = std::process::Command::new(executable);
    command.args(job.child_arguments()).arg("--raw-output").arg(&raw_file);
    if simulated {
        command.arg("--simulated");
    }
    let output = command.output().map_err(|error| format!("起不了子进程：{error}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let has_done_line = stdout.lines().any(|line| line.starts_with("E7RESULT name=child_done "));
    if output.status.success() && has_done_line {
        if raw_file.exists() {
            append_gzip_member(&raw_file.to_string_lossy(), raw_archive)?;
            let _ = std::fs::remove_file(&raw_file);
        }
        return Ok(stdout);
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let tail: String = stderr.chars().rev().take(STDERR_TAIL_CHARACTERS).collect::<Vec<_>>().into_iter().rev().collect();
    let _ = std::fs::remove_file(&raw_file);
    Ok(format!(
        "E7RESULT name=child_failed {} exit={:?} has_done_line={has_done_line} detail={}\n",
        job.key_text(),
        output.status.code(),
        without_whitespace(&tail)
    ))
}

fn load_product(path: &str) -> Result<(String, ProductRounds), String> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(format!("读不了 {path}：{error}")),
    };
    let rounds = parse_product(&text)?;
    Ok((text, rounds))
}

fn run_verdict_line(segment: Segment, cells: &[CellKey], product: &ProductRounds, stopped_by: &str) -> String {
    let required = usize::try_from(ROUNDS_REQUIRED_PER_CELL).expect("轮数装得进 usize");
    let cells_incomplete = cells
        .iter()
        .filter(|cell| {
            product.iter().filter(|(job, data)| job.cell == **cell && data.is_complete(cell.lease_count())).count() < required
        })
        .count();
    let children_complete = product.iter().filter(|(job, data)| data.is_complete(job.cell.lease_count())).count();
    let children_failed = product.values().filter(|data| data.failure_detail.is_some()).count();
    format!(
        "E7RESULT name=verdict scope=run segment={} cells_total={} cells_incomplete={cells_incomplete} children_total={} \
         children_complete={children_complete} children_failed={children_failed} children_void={} stopped_by={stopped_by}",
        segment.label(),
        cells.len(),
        product.len(),
        product.len() - children_complete - children_failed,
    )
}

/// 「扣住不丢」组的一个子进程里出现了 F1：整轮停（登记第十一节够判停机）。
fn child_output_shows_hold_control_failure(job: &ChildJob, child_text: &str) -> bool {
    job.cell.is_hold_control() && child_text.contains(VoidReason::HoldControlDidNotHold.label())
}

fn run_parent(arguments: &[String]) -> Result<i32, String> {
    let options = parse_parent_options(arguments)?;
    if !options.simulated {
        check_driver_version()?;
    }
    let cells = cells_of_segment(options.segment, options.skip_light);
    let arm_count = match options.segment {
        Segment::First => 2,
        Segment::Second => 3,
    };
    let (initial_text, initial_product) = load_product(&options.output_path)?;
    let already_has_done_line = initial_text.lines().any(|line| line.starts_with("E7RESULT name=done "));
    if already_has_done_line && pending_jobs(&cells, &initial_product, arm_count).is_empty() {
        println!("{} 已经有 name=done 行，这一组格也没有要补的轮，不再追加", options.output_path);
        return Ok(0);
    }
    if initial_text.is_empty() || already_has_done_line {
        append_to_file(
            &options.output_path,
            &format!(
                "E7RESULT name=config segment={} seed={SHUFFLE_SEED} driver_version={EXPECTED_DRIVER_VERSION} skip_light={} cells={} \
                 rounds_required={ROUNDS_REQUIRED_PER_CELL} refills_at_most={MAXIMUM_REFILL_ROUNDS_PER_CELL} simulated={}\n",
                options.segment.label(),
                options.skip_light,
                cells.len(),
                options.simulated
            ),
        )?;
    }
    let scratch_directory = std::env::temp_dir();
    let raw_archive = raw_archive_path(&options.output_path);
    let mut stopped_by = "none";
    let mut jobs_run: usize = 0;
    'passes: loop {
        let (_, product) = load_product(&options.output_path)?;
        let pending = pending_jobs(&cells, &product, arm_count);
        if pending.is_empty() {
            break;
        }
        for job in pending {
            if options.limit_jobs.is_some_and(|limit| jobs_run >= limit) {
                println!("跑满 --limit-jobs {jobs_run} 个子进程，停在这里：没写 verdict 与 done，同一条命令去掉 --limit-jobs 就接着往下跑");
                return Ok(0);
            }
            let child_text = run_one_child(&job, &raw_archive, &scratch_directory, options.simulated)?;
            jobs_run += 1;
            append_to_file(&options.output_path, &child_text)?;
            if child_output_shows_hold_control_failure(&job, &child_text) {
                stopped_by = "f1_hold_control";
                break 'passes;
            }
        }
    }
    let (text, product) = load_product(&options.output_path)?;
    let verdict = run_verdict_line(options.segment, &cells, &product, stopped_by);
    let emitted = text.lines().filter(|line| line.starts_with("E7RESULT ")).count() + 2;
    append_to_file(&options.output_path, &format!("{verdict}\nE7RESULT name=done emitted={emitted}\n"))?;
    Ok(if stopped_by == "none" { 0 } else { 4 })
}

// ============================== 不起卡的自检：模型卡上跑整套步骤 ==============================

fn simulated_job(group: Group, size_mebibytes: u64, liveness: Liveness, method: Method, arm: Arm, round: u32) -> ChildJob {
    ChildJob { cell: CellKey { group, card: Card::Rtx5090, size_mebibytes, liveness, method, arm }, round }
}

/// 在模型卡上跑一个子进程，交回结果和它会打出来的整段文本（借据行加 child_done 行）。
fn simulate_job(job: &ChildJob) -> (ChildOutcome, String) {
    let work_microseconds = job.cell.liveness.work_target_microseconds();
    let mut host = SimulatedCardHost::new(job.cell.card, work_microseconds);
    let mut raw_log = RawLog::default();
    let outcome = run_child_procedure(&mut host, job, work_microseconds, true, &mut raw_log);
    let mut text = String::new();
    for lease in &outcome.leases {
        text.push_str(&lease.to_line());
        text.push('\n');
    }
    text.push_str(&child_done_line(job, &outcome, work_microseconds, true, "background_submissions=none adapter=simulated"));
    text.push('\n');
    (outcome, text)
}

/// 模型卡上把某一格的 `rounds` 个轮都跑一遍，拼成一份产物文本。
fn simulated_product_text(cell: CellKey, rounds: u32) -> String {
    (1..=rounds).map(|round| simulate_job(&ChildJob { cell, round }).1).collect()
}

fn leases_have_no_void_reason(outcome: &ChildOutcome) -> bool {
    !outcome.leases.is_empty() && outcome.child_void.is_empty() && outcome.leases.iter().all(|lease| !lease.is_void())
}

/// 每条自检一个名字和是否通过；`checks` 子命令逐条打出来，单测断言一条都不许红。
fn self_check_results() -> Vec<(String, bool)> {
    let mut results: Vec<(String, bool)> = Vec::new();
    let expected_piece_plans: [(u64, u64, u64); 5] = [
        (1, 1, MEBIBYTE),
        (64, 1, 64 * MEBIBYTE),
        (512, 1, 512 * MEBIBYTE),
        (2048, 2, GIBIBYTE),
        (8192, 8, GIBIBYTE),
    ];
    for (size_mebibytes, expected_count, expected_each) in expected_piece_plans {
        let (count, each) = pieces_for_size(size_mebibytes * MEBIBYTE);
        results.push((
            format!("b2_piece_plan_{size_mebibytes}_mib"),
            count == expected_count && each == expected_each && count * each == size_mebibytes * MEBIBYTE,
        ));
    }
    for liveness in [Liveness::Done, Liveness::InFlightMedium, Liveness::InFlightHeavy] {
        for arm in [Arm::WaitLatest, Arm::Poll] {
            let job = simulated_job(Group::MainGrid, 2048, liveness, Method::OneBuild, arm, 1);
            results.push((
                format!("simulated_main_grid_{}_{}_has_no_void_reason", liveness.label(), arm.label()),
                leases_have_no_void_reason(&simulate_job(&job).0),
            ));
        }
    }
    let light_job = simulated_job(Group::MainGrid, 64, Liveness::InFlightLight, Method::OneBuild, Arm::WaitLatest, 1);
    let light_outcome = simulate_job(&light_job).0;
    results.push((
        "simulated_light_trips_k9_at_the_registered_numbers".to_string(),
        light_outcome.leases.len() == 1 && light_outcome.leases[0].void_reasons.contains(&VoidReason::WorkNotInFlightAtPoll),
    ));
    let pass_by_pass_job = simulated_job(Group::PassByPassGrid, 64, Liveness::Done, Method::PassByPass, Arm::WaitLatest, 1);
    results.push(("simulated_pass_by_pass_has_no_void_reason".to_string(), leases_have_no_void_reason(&simulate_job(&pass_by_pass_job).0)));
    for arm in [Arm::WaitLatest, Arm::Poll] {
        let hold_job = simulated_job(Group::HoldControl, 2048, Liveness::InFlightMedium, Method::OneBuild, arm, 1);
        results.push((
            format!("simulated_hold_control_{}_still_holds_the_buffer", arm.label()),
            leases_have_no_void_reason(&simulate_job(&hold_job).0),
        ));
    }
    results.extend(negative_control_results());
    results.extend(judge_on_simulated_product_results());
    results.extend(plan_results());
    results
}

/// 检查本身也要分得出差别：模型卡被注入已知的错，对应的作废理由必须出现。
fn negative_control_results() -> Vec<(String, bool)> {
    let job = simulated_job(Group::MainGrid, 64, Liveness::InFlightMedium, Method::OneBuild, Arm::WaitLatest, 1);
    let work_microseconds = job.cell.liveness.work_target_microseconds();
    let mut with_foreign_process = SimulatedCardHost::new(job.cell.card, work_microseconds);
    with_foreign_process.foreign_process_identifier = Some(777);
    let foreign = run_child_procedure(&mut with_foreign_process, &job, work_microseconds, true, &mut RawLog::default());
    let mut with_report_difference = SimulatedCardHost::new(job.cell.card, work_microseconds);
    with_report_difference.compute_applications_extra_mebibytes = 3;
    let different = run_child_procedure(&mut with_report_difference, &job, work_microseconds, true, &mut RawLog::default());
    let out_of_band = run_child_procedure(&mut SimulatedCardHost::new(job.cell.card, work_microseconds), &job, work_microseconds, false, &mut RawLog::default());
    vec![
        ("negative_control_foreign_process_is_v1".to_string(), foreign.child_void.contains(&VoidReason::ForeignProcessListed)),
        ("negative_control_report_difference_is_k5".to_string(), different.child_void.contains(&VoidReason::ProcessReportMismatch)),
        ("negative_control_calibration_out_of_band_is_k6".to_string(), out_of_band.child_void == vec![VoidReason::CalibrationOutOfBand]),
    ]
}

/// 模型卡产物上跑判：等待臂读不出含着、不等待臂读得出含着、小借据按块多拿，判别力自证至少翻一栏。
fn judge_on_simulated_product_results() -> Vec<(String, bool)> {
    let wait_cell = CellKey {
        group: Group::MainGrid,
        card: Card::Rtx5090,
        size_mebibytes: 2048,
        liveness: Liveness::InFlightMedium,
        method: Method::OneBuild,
        arm: Arm::WaitLatest,
    };
    let poll_cell = CellKey { arm: Arm::Poll, ..wait_cell };
    let small_wait_cell = CellKey { size_mebibytes: 64, ..wait_cell };
    let text: String = [wait_cell, poll_cell, small_wait_cell].iter().map(|cell| simulated_product_text(*cell, 5)).collect();
    let rounds = parse_product(&text).expect("模型卡自己打的产物读得回来");
    let judged = judge_product(&rounds, JudgeSettings { tau_override_mebibytes: None });
    let column_of = |cell: CellKey, column: Column| judged.columns.iter().find(|verdict| verdict.cell == cell && verdict.column == column);
    let wait_columns_are_clean = [Column::First, Column::Later].iter().all(|column| {
        column_of(wait_cell, *column).is_some_and(|verdict| {
            verdict.next_presence == Presence::Absent
                && verdict.clear_token == "next_reading_already_absent"
                && verdict.end_presence == Presence::Absent
                && verdict.wait_token == "only_the_work_itself"
                && verdict.rounds_used == 5
        })
    });
    // 模型里不等待臂的第 2、3 笔会在自己那次 poll 顺带收掉上一笔做完了的缓冲，所以第二栏的 R_next 不含、窗口末还含着。
    let poll_first_column_holds = column_of(poll_cell, Column::First).is_some_and(|verdict| {
        verdict.next_presence == Presence::Present && verdict.wait_token == "does_not_wait" && verdict.end_presence == Presence::Present
    });
    let poll_later_column_ends_holding = column_of(poll_cell, Column::Later)
        .is_some_and(|verdict| verdict.wait_token == "does_not_wait" && verdict.end_presence == Presence::Present);
    let poll_columns_hold = poll_first_column_holds && poll_later_column_ends_holding;
    let small_first_column_keeps_the_block = column_of(small_wait_cell, Column::First)
        .is_some_and(|verdict| verdict.next_presence == Presence::Present && verdict.end_presence == Presence::Present && verdict.overtake_presence == Presence::Present);
    let line_round_trip = rounds.values().flat_map(|data| data.leases.iter()).all(|lease| LeaseRecord::from_line(&lease.to_line()).as_ref() == Ok(lease));
    vec![
        ("judge_simulated_wait_arm_2048_is_clean_in_both_columns".to_string(), wait_columns_are_clean),
        ("judge_simulated_poll_arm_2048_holds_the_memory".to_string(), poll_columns_hold),
        ("judge_simulated_small_lease_first_column_keeps_the_block".to_string(), small_first_column_keeps_the_block),
        ("lease_line_round_trips".to_string(), line_round_trip),
        (
            "judge_moving_tau_flips_at_least_one_column".to_string(),
            count_columns_flipped_by_moving_tau(&rounds, &judged) >= 1,
        ),
    ]
}

fn plan_results() -> Vec<(String, bool)> {
    let first = first_segment_cells();
    let second = second_segment_cells();
    let empty_product: ProductRounds = BTreeMap::new();
    let pending = pending_jobs(&first, &empty_product, 2);
    let required_rounds = usize::try_from(ROUNDS_REQUIRED_PER_CELL).expect("轮数装得进 usize");
    let light_cell = CellKey {
        group: Group::MainGrid,
        card: Card::Rtx5090,
        size_mebibytes: 64,
        liveness: Liveness::InFlightLight,
        method: Method::OneBuild,
        arm: Arm::WaitLatest,
    };
    let mut light_product = parse_product(&simulated_product_text(light_cell, 5)).expect("读得回来");
    let mut refills: Vec<u32> = pending_jobs(&[light_cell], &light_product, 2).iter().map(|job| job.round).collect();
    refills.sort_unstable();
    light_product = parse_product(&simulated_product_text(light_cell, 8)).expect("读得回来");
    let after_cap = pending_jobs(&[light_cell], &light_product, 2);
    let second_pending = pending_jobs(&second, &empty_product, 3);
    vec![
        ("plan_first_segment_has_128_cells".to_string(), first.len() == 128),
        ("plan_second_segment_has_24_cells".to_string(), second.len() == 24),
        ("plan_pending_on_an_empty_product_is_five_rounds_per_cell".to_string(), pending.len() == first.len() * required_rounds),
        (
            "plan_hold_control_first_and_light_last".to_string(),
            pending.first().is_some_and(|job| job.cell.group == Group::HoldControl && job.cell.liveness != Liveness::InFlightLight)
                && pending.last().is_some_and(|job| job.cell.liveness == Liveness::InFlightLight),
        ),
        ("plan_refills_are_rounds_6_to_8_when_five_rounds_are_all_void".to_string(), refills == vec![6, 7, 8]),
        ("plan_no_refill_after_three".to_string(), after_cap.is_empty()),
        ("plan_second_segment_pending_is_five_rounds_per_cell".to_string(), second_pending.len() == second.len() * required_rounds),
        ("plan_order_is_the_same_every_time".to_string(), pending == pending_jobs(&first, &empty_product, 2)),
    ]
}

// ============================== 子命令：judge、checks、plan ==============================

fn done_line(lines: &[String]) -> String {
    let emitted = lines.iter().filter(|line| line.starts_with("E7RESULT ")).count() + 1;
    format!("E7RESULT name=done emitted={emitted}")
}

fn run_judge_command(arguments: &[String]) -> Result<(), String> {
    let mut input_paths: Vec<&String> = Vec::new();
    let mut scope = "first".to_string();
    let mut index = 0;
    while index < arguments.len() {
        let value = arguments.get(index + 1).ok_or_else(|| format!("{} 后面没有值", arguments[index]))?;
        match arguments[index].as_str() {
            "--input" => input_paths.push(value),
            "--scope" => scope = value.clone(),
            other => return Err(format!("认不出的参数 {other}")),
        }
        index += 2;
    }
    if input_paths.is_empty() {
        return Err("缺 --input".to_string());
    }
    let mut text = String::new();
    for path in input_paths {
        text.push_str(&std::fs::read_to_string(path).map_err(|error| format!("读不了 {path}：{error}"))?);
    }
    let rounds = parse_product(&text)?;
    let judged = judge_product(&rounds, JudgeSettings { tau_override_mebibytes: None });
    let flipped = count_columns_flipped_by_moving_tau(&rounds, &judged);
    let synthetic_flips = judge_on_simulated_product_results()
        .iter()
        .any(|(name, passed)| name == "judge_moving_tau_flips_at_least_one_column" && *passed);
    let mut lines = judged.lines.clone();
    for (main, coexisting) in geometry_pairs(&judged.columns) {
        lines.push(geometry_pair_line(main, coexisting));
    }
    lines.push(verdict_line(&scope, &judged, flipped, synthetic_flips));
    lines.push(done_line(&lines));
    println!("{}", lines.join("\n"));
    Ok(())
}

fn run_checks_command() -> i32 {
    let results = self_check_results();
    let mut lines: Vec<String> =
        results.iter().map(|(name, passed)| format!("E7RESULT name=self_check id={name} passed={passed}")).collect();
    let failed = results.iter().filter(|(_, passed)| !*passed).count();
    lines.push(format!(
        "E7RESULT name=verdict scope=checks self_checks_total={} self_checks_failed={failed} void_conditions=0",
        results.len()
    ));
    lines.push(done_line(&lines));
    println!("{}", lines.join("\n"));
    if failed == 0 { 0 } else { 1 }
}

fn run_plan_command(arguments: &[String]) -> Result<(), String> {
    let options = parse_parent_options(&[arguments, &["--output".to_string(), "unused".to_string()]].concat())?;
    let cells = cells_of_segment(options.segment, options.skip_light);
    let mut per_stage: BTreeMap<(u32, &'static str), usize> = BTreeMap::new();
    for cell in &cells {
        *per_stage.entry((stage_of(*cell), cell.group.label())).or_default() += 1;
    }
    let mut lines: Vec<String> = per_stage
        .iter()
        .map(|((stage, group), count)| {
            format!(
                "E7RESULT name=plan segment={} stage={stage} group={group} cells={count} first_pass_children={}",
                options.segment.label(),
                count * usize::try_from(ROUNDS_REQUIRED_PER_CELL).expect("轮数装得进 usize")
            )
        })
        .collect();
    lines.push(format!(
        "E7RESULT name=verdict scope=plan segment={} cells_total={} first_pass_children_total={} void_conditions=0",
        options.segment.label(),
        cells.len(),
        cells.len() * usize::try_from(ROUNDS_REQUIRED_PER_CELL).expect("轮数装得进 usize")
    ));
    lines.push(done_line(&lines));
    println!("{}", lines.join("\n"));
    Ok(())
}

/// `simulate` 子命令：在模型卡上跑某一格的几轮，打出产物行；拿来试 `judge`，不起卡。参数同 `child`，再加 `--rounds N`。
fn run_simulate_command(arguments: &[String]) -> Result<(), String> {
    let mut job_arguments: Vec<String> = Vec::new();
    let mut rounds: u32 = 1;
    let mut index = 0;
    while index < arguments.len() {
        let value = arguments.get(index + 1).ok_or_else(|| format!("{} 后面没有值", arguments[index]))?;
        if arguments[index] == "--rounds" {
            rounds = value.parse().map_err(|_| "--rounds 不是整数".to_string())?;
        } else {
            job_arguments.push(arguments[index].clone());
            job_arguments.push(value.clone());
        }
        index += 2;
    }
    let job = ChildJob::from_child_arguments(&job_arguments)?;
    print!("{}", simulated_product_text(job.cell, rounds));
    Ok(())
}

fn exit_code_of(result: Result<i32, String>) -> i32 {
    match result {
        Ok(code) => code,
        Err(message) => {
            eprintln!("e174：{message}");
            3
        }
    }
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let rest = arguments.get(1..).unwrap_or(&[]);
    let exit_code = match arguments.first().map(String::as_str) {
        Some("run") => exit_code_of(run_parent(rest)),
        Some("child") => exit_code_of(run_child(rest).map(|()| 0)),
        Some("judge") => exit_code_of(run_judge_command(rest).map(|()| 0)),
        Some("checks") => run_checks_command(),
        Some("plan") => exit_code_of(run_plan_command(rest).map(|()| 0)),
        Some("simulate") => exit_code_of(run_simulate_command(rest).map(|()| 0)),
        Some(_) | None => {
            eprintln!("用法：e174-lease-drop-wait-memory-return run|child|judge|checks|plan|simulate …（见文件头）");
            2
        }
    };
    std::process::exit(exit_code);
}

// ============================== 单测（不起卡；变异表 scripts/mutation/tables/e174_lease_drop_wait_memory_return.tsv 的变异由它们抓） ==============================

#[cfg(test)]
mod tests {
    use super::*;

    fn first_lease_of(job: &ChildJob) -> LeaseRecord {
        let outcome = simulate_job(job).0;
        assert!(outcome.child_void.is_empty(), "模型卡上子进程不该作废：{:?}", outcome.child_void);
        outcome.leases.into_iter().next().expect("至少一笔借据")
    }

    fn lease_with_samples(pre_mebibytes: u64, pre_spread_mebibytes: u64, used: &[Option<u64>]) -> LeaseRecord {
        let job = simulated_job(Group::MainGrid, 64, Liveness::InFlightMedium, Method::OneBuild, Arm::WaitLatest, 1);
        let mut record = LeaseRecord::new(job, 1);
        record.pre_mebibytes = pre_mebibytes;
        record.pre_spread_mebibytes = pre_spread_mebibytes;
        record.samples = used
            .iter()
            .enumerate()
            .map(|(index, used_mebibytes)| {
                let started = u64::try_from(index).expect("下标装得进 u64") * 1_000;
                WindowSample { started_microseconds: started, finished_microseconds: started + 500, used_mebibytes: *used_mebibytes }
            })
            .collect();
        record
    }

    #[test]
    fn every_self_check_passes() {
        let failed: Vec<String> = self_check_results().into_iter().filter(|(_, passed)| !*passed).map(|(name, _)| name).collect();
        assert!(failed.is_empty(), "红的自检：{failed:?}");
        assert!(self_check_results().len() >= 30, "自检条数不该变少");
    }

    #[test]
    fn report_parser_reads_the_process_line_and_not_the_bar_one_line() {
        let report = SimulatedCardHost::new(Card::Rtx5090, 50_000).report_text();
        let reading = parse_card_reading(&report, "01:00.0", SIMULATED_PROCESS_IDENTIFIER).expect("读得出");
        assert_eq!(reading.total_mebibytes, Some(32607), "Total 是 FB 那一小节的，不是 BAR1 的 32768");
        assert_eq!(reading.process_used_mebibytes, Some(SIMULATED_RUNTIME_MEBIBYTES), "进程占着的是 Processes 下的，不是 BAR1 的 Used 1 MiB");
        assert_eq!(reading.listed_process_identifiers, vec![SIMULATED_PROCESS_IDENTIFIER]);
        assert!(parse_card_reading(&report, "10:00.0", SIMULATED_PROCESS_IDENTIFIER).is_err(), "报告里没有的卡是读不出，不是 0");
    }

    #[test]
    fn report_parser_adds_up_every_line_of_this_process_and_skips_others() {
        let report = "GPU 00000000:01:00.0\n    FB Memory Usage\n        Total : 32607 MiB\n    Processes\n        Process ID : 10\n            Used GPU Memory : 100 MiB\n        Process ID : 11\n            Used GPU Memory : 7 MiB\n        Process ID : 10\n            Used GPU Memory : 23 MiB\n";
        let reading = parse_card_reading(report, "01:00.0", 10).expect("读得出");
        assert_eq!(reading.process_used_mebibytes, Some(123));
        assert_eq!(reading.listed_process_identifiers, vec![10, 11, 10]);
        let unreadable = report.replace("23 MiB", "N/A");
        assert!(parse_card_reading(&unreadable, "01:00.0", 10).expect("读得出").process_used_is_unreadable);
        let not_listed = parse_card_reading(report, "01:00.0", 99).expect("读得出");
        assert_eq!(not_listed.process_used_mebibytes, None, "没列这个进程是读不到，不当 0");
    }

    #[test]
    fn compute_applications_parser_sums_rows_of_this_card_and_process() {
        let text = "00000000:01:00.0, 10, 100\n00000000:01:00.0, 11, 7\n00000000:10:00.0, 10, 5\n00000000:01:00.0, 10, 20\n";
        assert_eq!(parse_compute_applications(text, "01:00.0", 10), Ok(Some(120)));
        assert_eq!(parse_compute_applications(text, "01:00.0", 99), Ok(None));
        assert!(parse_compute_applications("00000000:01:00.0, 10\n", "01:00.0", 10).is_err());
    }

    #[test]
    fn piece_plan_covers_every_registered_size() {
        let sizes_and_counts = [(1_u64, 1_u64), (64, 1), (512, 1), (2048, 2), (8192, 8)];
        for (size_mebibytes, expected_count) in sizes_and_counts {
            let job = simulated_job(Group::MainGrid, size_mebibytes, Liveness::Done, Method::OneBuild, Arm::WaitLatest, 1);
            let lease = first_lease_of(&job);
            assert_eq!(lease.piece_count, expected_count, "S={size_mebibytes} MiB 的张数");
            assert_eq!(lease.piece_count * lease.piece_bytes, size_mebibytes * MEBIBYTE, "张数 × 每张 == S（加法形）");
            assert!(!lease.is_void(), "S={size_mebibytes} MiB：{:?}", lease.void_reasons);
        }
    }

    #[test]
    fn allocator_peak_grows_by_at_least_the_whole_planned_size() {
        let job = simulated_job(Group::MainGrid, 512, Liveness::Done, Method::OneBuild, Arm::WaitLatest, 1);
        let lease = first_lease_of(&job);
        let before = lease.allocator_pre.expect("模型卡给分配器两项");
        let at_peak = lease.allocator_peak.expect("模型卡给分配器两项");
        assert!(at_peak.allocated_bytes >= before.allocated_bytes + 512 * MEBIBYTE, "登记 B3");
        assert!(!lease.void_reasons.contains(&VoidReason::AllocatorBelowPlan));
        assert!(lease.peak_mebibytes.expect("读到了") + lease.tau_mebibytes() >= lease.pre_mebibytes + 512, "登记 B4 后一半");
    }

    #[test]
    fn heavy_wait_arm_waits_for_the_rest_of_the_work_and_the_outer_timer_covers_it() {
        let job = simulated_job(Group::MainGrid, 64, Liveness::InFlightHeavy, Method::OneBuild, Arm::WaitLatest, 1);
        let lease = first_lease_of(&job);
        assert!(lease.void_reasons.is_empty(), "K7、K8、K9 都该过：{:?}", lease.void_reasons);
        let work = lease.work_microseconds;
        assert_eq!(work, 400_000);
        assert!(lease.wait_microseconds + lease.submit_to_poll_microseconds >= work * 4 / 5, "K7：T_wait + e ≥ 0.8 W（0.8 写成 4/5，不引常量）");
        assert!(lease.wait_microseconds < work * 4 / 5, "e 含两次读数，所以 T_wait 本身小于 0.8 W：K7 的左边必须带上 e");
        assert!(lease.wait_microseconds >= 200_000, "等待臂在在飞·重上真的等了：{}", lease.wait_microseconds);
        assert!(lease.outer_microseconds >= lease.wait_microseconds, "K8：外层包住 T_wait");
        assert!(lease.submit_to_poll_microseconds * 100 < work * COVERED_WORK_PERCENT, "K9：poll 开始时活还没做完");
    }

    #[test]
    fn a_lease_that_is_already_void_does_not_sit_through_the_window() {
        let light_job = simulated_job(Group::MainGrid, 64, Liveness::InFlightLight, Method::OneBuild, Arm::WaitLatest, 1);
        let outcome = simulate_job(&light_job).0;
        assert_eq!(outcome.leases.len(), 1, "第 1 笔作废，这一轮不再起第 2、3 笔");
        assert!(outcome.leases[0].void_reasons.contains(&VoidReason::WorkNotInFlightAtPoll));
        assert!(outcome.leases[0].samples.is_empty(), "作废的借据不读窗口");
        let healthy_job = simulated_job(Group::MainGrid, 64, Liveness::InFlightMedium, Method::OneBuild, Arm::WaitLatest, 1);
        assert!(!first_lease_of(&healthy_job).samples.is_empty(), "没作废的借据读窗口");
    }

    #[test]
    fn in_flight_work_is_not_waited_for_before_the_lease_drop() {
        for liveness in [Liveness::InFlightMedium, Liveness::InFlightHeavy] {
            let job = simulated_job(Group::MainGrid, 64, liveness, Method::OneBuild, Arm::Poll, 1);
            let lease = first_lease_of(&job);
            assert!(!lease.void_reasons.contains(&VoidReason::WorkNotInFlightAtPoll), "{liveness:?} 的活在 poll 之前就被等完了");
            assert!(lease.wait_microseconds <= POLL_NO_WAIT_LIMIT_MICROSECONDS, "不等待臂不等：{}", lease.wait_microseconds);
        }
    }

    #[test]
    fn hold_control_still_holds_the_buffer_at_the_first_reading_after_the_drop() {
        for arm in [Arm::WaitLatest, Arm::Poll] {
            let job = simulated_job(Group::HoldControl, 2048, Liveness::InFlightMedium, Method::OneBuild, arm, 1);
            let lease = first_lease_of(&job);
            assert!(lease.void_reasons.is_empty(), "{arm:?}：{:?}", lease.void_reasons);
            let next = lease.next_remainder_mebibytes().expect("第一份读到了");
            assert!(next + signed(lease.tau_mebibytes()) >= 2048, "B4 前一半：R_next={next}");
        }
    }

    #[test]
    fn time_to_clear_is_the_start_of_the_first_reading_after_which_every_reading_is_clean() {
        let settings = JudgeSettings { tau_override_mebibytes: None };
        let clean_from_the_start = lease_with_samples(100, 1, &[Some(100), Some(101), Some(103)]);
        assert_eq!(clean_from_the_start.time_to_clear(settings), TimeToClear::Next, "τ=3，103−100 不大于 τ");
        let clean_after_two = lease_with_samples(100, 1, &[Some(300), Some(200), Some(101), Some(100), Some(100)]);
        assert_eq!(clean_after_two.time_to_clear(settings), TimeToClear::After(2_000));
        let dirty_at_the_end = lease_with_samples(100, 1, &[Some(100), Some(100), Some(250)]);
        assert_eq!(dirty_at_the_end.time_to_clear(settings), TimeToClear::BeyondWindow);
        let unreadable_at_the_end = lease_with_samples(100, 1, &[Some(100), Some(100), None]);
        assert_eq!(unreadable_at_the_end.time_to_clear(settings), TimeToClear::BeyondWindow, "读不到不当 0");
        let dirty_in_the_middle = lease_with_samples(100, 1, &[Some(100), Some(300), Some(100), Some(100)]);
        assert_eq!(dirty_in_the_middle.time_to_clear(settings), TimeToClear::After(2_000), "中间含过一次，从之后算");
        assert_eq!(
            dirty_in_the_middle.positive_reading_counts(settings),
            (1, 4),
            "为正的份数与读得到的总份数并列"
        );
        assert_eq!(dirty_in_the_middle.peak_remainder_mebibytes(), Some(200));
        let moved = JudgeSettings { tau_override_mebibytes: Some(500) };
        assert_eq!(dirty_in_the_middle.time_to_clear(moved), TimeToClear::Next, "判别力自证：τ 挪到读数之上，判定翻面");
    }

    #[test]
    fn presence_needs_all_rounds_on_one_side_of_their_own_tau() {
        assert_eq!(classify_presence(&[(2, 2), (0, 2), (-5, 2), (1, 3), (3, 3)]), Presence::Absent, "等于 τ 算不含");
        assert_eq!(classify_presence(&[(3, 2), (300, 2), (4, 3), (9, 3), (5, 3)]), Presence::Present);
        assert_eq!(classify_presence(&[(3, 2), (300, 2), (4, 3), (9, 3), (2, 3)]), Presence::Unstable);
        assert_eq!(classify_presence(&[]), Presence::Unstable, "没有轮不能说不含");
    }

    #[test]
    fn wait_cost_tokens_follow_the_registered_limits() {
        let done_wait = CellKey {
            group: Group::MainGrid,
            card: Card::Rtx5090,
            size_mebibytes: 64,
            liveness: Liveness::Done,
            method: Method::OneBuild,
            arm: Arm::WaitLatest,
        };
        assert_eq!(wait_cost_token(done_wait, &[5_000, 0, 10, 3, 4_999], false), "zero_overhead");
        assert_eq!(wait_cost_token(done_wait, &[5_001, 9_000, 6_000, 7_000, 8_000], false), "fixed_overhead");
        assert_eq!(wait_cost_token(done_wait, &[5_001, 100, 6_000, 7_000, 8_000], false), "unstable");
        assert_eq!(wait_cost_token(done_wait, &[1, 1, 1, 1, 1], true), "wait_never_returned");
        let in_flight_wait = CellKey { liveness: Liveness::InFlightHeavy, ..done_wait };
        assert_eq!(wait_cost_token(in_flight_wait, &[20_000, -50, 0, 0, 0], false), "only_the_work_itself");
        assert_eq!(wait_cost_token(in_flight_wait, &[20_001, 30_000, 25_000, 90_000, 21_000], false), "longer_than_the_work");
        let poll = CellKey { arm: Arm::Poll, ..done_wait };
        assert_eq!(wait_cost_token(poll, &[1_000, 3, 0, 1, 2], false), "does_not_wait");
        assert_eq!(wait_cost_token(poll, &[1_001, 3, 0, 1, 2], false), "reports_numbers");
    }

    #[test]
    fn round_is_complete_when_the_child_is_clean_and_the_needed_leases_are_valid() {
        let job = simulated_job(Group::MainGrid, 64, Liveness::InFlightMedium, Method::OneBuild, Arm::WaitLatest, 1);
        let valid = |lease_index: u32| LeaseRecord::new(job, lease_index);
        let void = |lease_index: u32| {
            let mut record = LeaseRecord::new(job, lease_index);
            record.void_reasons.push(VoidReason::ReadFailure);
            record
        };
        let round = |leases: Vec<LeaseRecord>, child_void: Vec<VoidReason>| RoundData {
            done: Some(ChildDoneRecord { child_void }),
            failure_detail: None,
            leases,
        };
        assert!(round(vec![valid(1), valid(2), valid(3)], vec![]).is_complete(3));
        assert!(round(vec![valid(1), void(2), valid(3)], vec![]).is_complete(3), "第 2、3 笔有一笔有效就够");
        assert!(!round(vec![valid(1), void(2), void(3)], vec![]).is_complete(3));
        assert!(!round(vec![void(1), valid(2), valid(3)], vec![]).is_complete(3), "第 1 笔作废，这一轮不进");
        assert!(!round(vec![valid(1), valid(2), valid(3)], vec![VoidReason::ForeignProcessListed]).is_complete(3));
        assert!(round(vec![valid(1)], vec![]).is_complete(1), "长窗只有一笔");
        let failed = RoundData { done: None, failure_detail: Some("exit".to_string()), leases: vec![] };
        assert!(!failed.is_complete(3));
    }

    #[test]
    fn job_arguments_round_trip_and_unknown_flags_are_refused() {
        for cell in first_segment_cells().into_iter().chain(second_segment_cells()) {
            let job = ChildJob { cell, round: 4 };
            let arguments = job.child_arguments();
            assert_eq!(arguments[0], "child");
            assert_eq!(ChildJob::from_child_arguments(&arguments[1..]), Ok(job));
        }
        assert!(ChildJob::from_child_arguments(&["--bogus".to_string(), "1".to_string()]).is_err());
    }

    #[test]
    fn shuffle_rank_is_seeded_and_does_not_depend_on_what_was_already_run() {
        assert_eq!(
            shuffle_rank("main 64 medium one_build round=1"),
            2_015_186_589_042_433_523,
            "种子 174 下这个键的位置钉死（独立算出）：种子或混合式变了这里先红"
        );
        assert_eq!(shuffle_rank("main 64 medium one_build round=1"), shuffle_rank("main 64 medium one_build round=1"));
        assert_ne!(shuffle_rank("main 64 medium one_build round=1"), shuffle_rank("main 64 medium one_build round=2"));
        let first = first_segment_cells();
        let full = pending_jobs(&first, &BTreeMap::new(), 2);
        let positions: BTreeMap<ChildJob, usize> = full.iter().enumerate().map(|(index, job)| (*job, index)).collect();
        let first_cell_rounds_one_to_three: Vec<ChildJob> =
            (1..=3).map(|round| ChildJob { cell: first[0], round }).collect();
        let done_so_far: String = first_cell_rounds_one_to_three.iter().map(|job| simulate_job(job).1).collect();
        let resumed = pending_jobs(&first, &parse_product(&done_so_far).expect("读得回来"), 2);
        assert_eq!(resumed.len(), full.len() - 3, "续跑只补没做完的");
        let resumed_positions: Vec<usize> = resumed.iter().map(|job| positions[job]).collect();
        assert!(resumed_positions.windows(2).all(|pair| pair[0] < pair[1]), "续跑的次序与整份清单的次序一致");
    }

    #[test]
    fn two_arms_alternate_with_the_round_number_on_the_same_slot() {
        let cells = first_segment_cells();
        let mut leading_arm_by_round: BTreeMap<u32, Vec<Arm>> = BTreeMap::new();
        for round in 1..=4 {
            let key_of = |arm: Arm| {
                let cell = cells
                    .iter()
                    .find(|cell| {
                        cell.group == Group::MainGrid && cell.card == Card::Rtx5090 && cell.size_mebibytes == 64 && cell.liveness == Liveness::InFlightMedium && cell.arm == arm
                    })
                    .expect("格在清单里");
                job_order_key(&ChildJob { cell: *cell, round }, 2)
            };
            let leading = if key_of(Arm::WaitLatest) < key_of(Arm::Poll) { Arm::WaitLatest } else { Arm::Poll };
            leading_arm_by_round.entry(round).or_default().push(leading);
        }
        assert_ne!(leading_arm_by_round[&1], leading_arm_by_round[&2]);
        assert_eq!(leading_arm_by_round[&1], leading_arm_by_round[&3]);
    }

    #[test]
    fn only_a_hold_control_child_that_lost_the_buffer_stops_the_whole_run() {
        let hold_job = simulated_job(Group::HoldControl, 2048, Liveness::InFlightMedium, Method::OneBuild, Arm::WaitLatest, 1);
        let main_job = ChildJob { cell: CellKey { group: Group::MainGrid, ..hold_job.cell }, round: 1 };
        let lost = format!("E7RESULT name=lease void={}", VoidReason::HoldControlDidNotHold.label());
        assert!(child_output_shows_hold_control_failure(&hold_job, &lost));
        assert!(!child_output_shows_hold_control_failure(&main_job, &lost), "别的组的子进程不触发 F1");
        assert!(!child_output_shows_hold_control_failure(&hold_job, "E7RESULT name=lease void=none"));
    }

    #[test]
    fn tau_is_the_pre_reading_spread_plus_the_rounding_allowance() {
        let lease = lease_with_samples(100, 5, &[Some(100)]);
        assert_eq!(lease.tau_mebibytes(), 5 + 2);
        let mut device_lease = lease;
        device_lease.device_spread_mebibytes = 9;
        assert_eq!(device_lease.tau_device_mebibytes(), 9 + 2);
    }
}
