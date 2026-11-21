//! 卡、存活、方法、臂、组与格键，任务单，读数的解析，作废原因，分配器读数与一次租借的记录。

use super::*;

// ============================== 取值范围里的枚举 ==============================

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(super) enum Card {
    Rtx5090,
    Rtx5060Ti,
}

impl Card {
    pub(super) const EVERY_CARD: [Card; 2] = [Card::Rtx5090, Card::Rtx5060Ti];

    pub(super) fn label(self) -> &'static str {
        match self {
            Card::Rtx5090 => "rtx5090",
            Card::Rtx5060Ti => "rtx5060ti",
        }
    }

    pub(super) fn from_label(label: &str) -> Option<Card> {
        Card::EVERY_CARD.into_iter().find(|card| card.label() == label)
    }

    /// 登记第七节 B1：FB 那一小节报的 Total（MiB）；服务给的卡照它认型号（`bus_for_the_cell_among`、`device_lines_by_card`）。
    pub(super) fn total_mebibytes(self) -> u64 {
        match self {
            Card::Rtx5090 => 32607,
            Card::Rtx5060Ti => 16311,
        }
    }
}

/// 活的档。`InFlightMedium` 是跑前登记「十二、修订」补的一条臂：`InFlightLight` 的 K9 在登记自己的数下必红。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(super) enum Liveness {
    Done,
    InFlightLight,
    InFlightMedium,
    InFlightHeavy,
}

impl Liveness {
    pub(super) const EVERY_LIVENESS: [Liveness; 4] =
        [Liveness::Done, Liveness::InFlightLight, Liveness::InFlightMedium, Liveness::InFlightHeavy];

    pub(super) fn label(self) -> &'static str {
        match self {
            Liveness::Done => "done",
            Liveness::InFlightLight => "light",
            Liveness::InFlightMedium => "medium",
            Liveness::InFlightHeavy => "heavy",
        }
    }

    pub(super) fn from_label(label: &str) -> Option<Liveness> {
        Liveness::EVERY_LIVENESS.into_iter().find(|liveness| liveness.label() == label)
    }

    pub(super) fn is_in_flight(self) -> bool {
        match self {
            Liveness::Done => false,
            Liveness::InFlightLight | Liveness::InFlightMedium | Liveness::InFlightHeavy => true,
        }
    }

    /// 空转目标时长（微秒）。登记：已做完与在飞·轻 50 ms，在飞·重 400 ms；在飞·中 250 ms 见修订。
    pub(super) fn work_target_microseconds(self) -> u64 {
        match self {
            Liveness::Done | Liveness::InFlightLight => 50_000,
            Liveness::InFlightMedium => 250_000,
            Liveness::InFlightHeavy => 400_000,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(super) enum Method {
    OneBuild,
    PassByPass,
}

impl Method {
    pub(super) fn label(self) -> &'static str {
        match self {
            Method::OneBuild => "one_build",
            Method::PassByPass => "pass_by_pass",
        }
    }

    pub(super) fn from_label(label: &str) -> Option<Method> {
        [Method::OneBuild, Method::PassByPass].into_iter().find(|method| method.label() == label)
    }
}

/// 借据丢掉时那一次 `device.poll` 的样子（登记第五节「臂」；第二段多一条「等本借据最后一次提交」）。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(super) enum Arm {
    /// `Wait { submission_index: None, timeout: Some(30 秒) }`
    WaitLatest,
    /// `Wait { submission_index: Some(本借据最后一次提交), timeout: Some(30 秒) }`，只在第二段。
    WaitOwnSubmission,
    /// `Poll`，今天 `crates/` 的做法。
    Poll,
}

impl Arm {
    pub(super) fn label(self) -> &'static str {
        match self {
            Arm::WaitLatest => "wait",
            Arm::WaitOwnSubmission => "wait_own",
            Arm::Poll => "poll",
        }
    }

    pub(super) fn from_label(label: &str) -> Option<Arm> {
        [Arm::WaitLatest, Arm::WaitOwnSubmission, Arm::Poll].into_iter().find(|arm| arm.label() == label)
    }

    pub(super) fn waits(self) -> bool {
        match self {
            Arm::WaitLatest | Arm::WaitOwnSubmission => true,
            Arm::Poll => false,
        }
    }
}

/// 登记第五节「格子」那张表的组。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(super) enum Group {
    HoldControl,
    MainGrid,
    PassByPassGrid,
    CoexistingLongLease,
    LongWindow,
    SharedDevice,
}

impl Group {
    pub(super) fn label(self) -> &'static str {
        match self {
            Group::HoldControl => "hold_control",
            Group::MainGrid => "main",
            Group::PassByPassGrid => "pass_by_pass",
            Group::CoexistingLongLease => "coexisting_long_lease",
            Group::LongWindow => "long_window",
            Group::SharedDevice => "shared_device",
        }
    }

    pub(super) fn from_label(label: &str) -> Option<Group> {
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
pub(super) struct CellKey {
    pub(super) group: Group,
    pub(super) card: Card,
    pub(super) size_mebibytes: u64,
    pub(super) liveness: Liveness,
    pub(super) method: Method,
    pub(super) arm: Arm,
}

impl CellKey {
    pub(super) fn fields_text(&self) -> String {
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

    pub(super) fn from_fields(fields: &BTreeMap<String, String>) -> Result<CellKey, String> {
        let field = |name: &str| fields.get(name).ok_or_else(|| format!("缺字段 {name}"));
        let group = Group::from_label(field("group")?).ok_or("group 认不出")?;
        let card = Card::from_label(field("card")?).ok_or("card 认不出")?;
        let size_mebibytes: u64 = field("size_mib")?.parse().map_err(|_| "size_mib 不是整数".to_string())?;
        let liveness = Liveness::from_label(field("liveness")?).ok_or("liveness 认不出")?;
        let method = Method::from_label(field("method")?).ok_or("method 认不出")?;
        let arm = Arm::from_label(field("arm")?).ok_or("arm 认不出")?;
        Ok(CellKey { group, card, size_mebibytes, liveness, method, arm })
    }

    pub(super) fn size_bytes(&self) -> u64 {
        self.size_mebibytes * MEBIBYTE
    }

    pub(super) fn is_hold_control(&self) -> bool {
        self.group == Group::HoldControl
    }

    pub(super) fn coexisting_mebibytes(&self) -> u64 {
        match self.group {
            Group::CoexistingLongLease => COEXISTING_BUFFER_MEBIBYTES,
            Group::HoldControl | Group::MainGrid | Group::PassByPassGrid | Group::LongWindow | Group::SharedDevice => 0,
        }
    }

    pub(super) fn has_background_load(&self) -> bool {
        self.group == Group::SharedDevice
    }

    pub(super) fn lease_count(&self) -> u32 {
        match self.group {
            Group::LongWindow => 1,
            Group::HoldControl | Group::MainGrid | Group::PassByPassGrid | Group::CoexistingLongLease | Group::SharedDevice => {
                LEASES_PER_PROCESS
            }
        }
    }

    pub(super) fn window_microseconds(&self) -> u64 {
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
pub(super) struct ChildJob {
    pub(super) cell: CellKey,
    pub(super) round: u32,
}

impl ChildJob {
    pub(super) fn child_arguments(&self) -> Vec<String> {
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

    pub(super) fn from_child_arguments(arguments: &[String]) -> Result<ChildJob, String> {
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

    pub(super) fn key_text(&self) -> String {
        format!("{} round={}", self.cell.fields_text(), self.round)
    }
}

/// 一行 `E7RESULT name=… k=v k=v` 拆成字段表；值里没有空格。
pub(super) fn parse_fields(line: &str) -> BTreeMap<String, String> {
    line.split_whitespace()
        .filter_map(|token| token.split_once('='))
        .map(|(name, value)| (name.to_string(), value.to_string()))
        .collect()
}

// ============================== 读数：解析 nvidia-smi 的两种出口 ==============================

/// `00000000:01:00.0`、`0000:01:00.0` 与 `01:00.0` 认成同一张卡：只留最后两段、小写。
pub(super) fn bus_without_domain(text: &str) -> String {
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
pub(super) struct CardReading {
    pub(super) total_mebibytes: Option<u64>,
    reserved_mebibytes: Option<u64>,
    free_mebibytes: Option<u64>,
    /// 进程号等于装置进程号的各行 `Used GPU Memory` 之和；没列这个进程是 `None`（读不到，不当 0）。
    pub(super) process_used_mebibytes: Option<u64>,
    /// 列了这个进程而有一行读不出。
    pub(super) process_used_is_unreadable: bool,
    pub(super) listed_process_identifiers: Vec<u32>,
}

pub(super) fn parse_card_reading(report: &str, bus: &str, process_identifier: u32) -> Result<CardReading, String> {
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
pub(super) fn parse_compute_applications(text: &str, bus: &str, process_identifier: u32) -> Result<Option<u64>, String> {
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

pub(super) fn median_of(values: &[u64]) -> u64 {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted[sorted.len() / 2]
}

pub(super) fn spread_of(values: &[u64]) -> u64 {
    values.iter().max().copied().unwrap_or(0) - values.iter().min().copied().unwrap_or(0)
}

pub(super) fn pieces_for_size(size_bytes: u64) -> (u64, u64) {
    let piece_count = size_bytes.div_ceil(MAXIMUM_PIECE_BYTES);
    let piece_bytes_each = size_bytes.min(MAXIMUM_PIECE_BYTES);
    (piece_count, piece_bytes_each)
}

// ============================== 作废理由与一笔借据的记录 ==============================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum VoidScope {
    /// 只作废这一笔借据（登记第十一节 V2、V3 的 U_pre、V6、V9）。
    Lease,
    /// 作废整个子进程（V1、V3 的 U_dev、V4、V5、V7、V8）。
    Child,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum VoidReason {
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
    pub(super) fn label(self) -> &'static str {
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

    pub(super) fn from_label(label: &str) -> Option<VoidReason> {
        VoidReason::EVERY_REASON.into_iter().find(|reason| reason.label() == label)
    }

    pub(super) fn scope(self) -> VoidScope {
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

pub(super) fn void_reasons_text(reasons: &[VoidReason]) -> String {
    if reasons.is_empty() {
        return "none".to_string();
    }
    reasons.iter().map(|reason| reason.label()).collect::<Vec<_>>().join("+")
}

pub(super) fn void_reasons_from_text(text: &str) -> Result<Vec<VoidReason>, String> {
    if text == "none" {
        return Ok(Vec::new());
    }
    text.split('+').map(|label| VoidReason::from_label(label).ok_or_else(|| format!("作废理由认不出：{label}"))).collect()
}

/// `Device::generate_allocator_report()` 的两项合计（字节）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct AllocatorFigures {
    pub(super) reserved_bytes: u64,
    pub(super) allocated_bytes: u64,
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
pub(super) struct WindowSample {
    pub(super) started_microseconds: u64,
    pub(super) finished_microseconds: u64,
    pub(super) used_mebibytes: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct LeaseRecord {
    pub(super) job: ChildJob,
    pub(super) lease_index: u32,
    pub(super) piece_count: u64,
    pub(super) piece_bytes: u64,
    pub(super) work_microseconds: u64,
    pub(super) device_mebibytes: u64,
    pub(super) device_spread_mebibytes: u64,
    pub(super) pre_mebibytes: u64,
    pub(super) pre_spread_mebibytes: u64,
    pub(super) peak_mebibytes: Option<u64>,
    pub(super) allocator_device: Option<AllocatorFigures>,
    pub(super) allocator_pre: Option<AllocatorFigures>,
    pub(super) allocator_peak: Option<AllocatorFigures>,
    pub(super) allocator_next: Option<AllocatorFigures>,
    pub(super) allocator_end: Option<AllocatorFigures>,
    pub(super) submit_to_poll_microseconds: u64,
    pub(super) wait_microseconds: u64,
    pub(super) outer_microseconds: u64,
    /// 臂是等待而 `Wait` 返回超时或错时的原样文本（空格换成下划线）。
    pub(super) wait_failure: Option<String>,
    pub(super) samples: Vec<WindowSample>,
    pub(super) void_reasons: Vec<VoidReason>,
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
    pub(super) fn new(job: ChildJob, lease_index: u32) -> LeaseRecord {
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

    pub(super) fn is_void(&self) -> bool {
        !self.void_reasons.is_empty()
    }

    pub(super) fn has_child_scope_reason(&self) -> bool {
        self.void_reasons.iter().any(|reason| reason.scope() == VoidScope::Child)
    }

    /// 这笔借据的 τ（MiB）：U_pre 那三份的极差加取整余量。
    pub(super) fn tau_mebibytes(&self) -> u64 {
        self.pre_spread_mebibytes + READING_ROUNDING_ALLOWANCE_MEBIBYTES
    }

    pub(super) fn tau_device_mebibytes(&self) -> u64 {
        self.device_spread_mebibytes + READING_ROUNDING_ALLOWANCE_MEBIBYTES
    }

    pub(super) fn to_line(&self) -> String {
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

    pub(super) fn from_line(line: &str) -> Result<LeaseRecord, String> {
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
