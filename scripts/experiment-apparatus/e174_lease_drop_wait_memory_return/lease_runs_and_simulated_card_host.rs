//! 卡主机的接口、读数与一次租借的执行与检查、子进程流程，模拟卡主机，产物的解析。

use super::*;

// ============================== 卡的接口：真卡与模型各一份实现 ==============================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct SubmissionToken(pub(super) u64);

/// 借据丢掉时（或别处）那一次 `device.poll` 的样子；等待那两种的超时都是 30 秒，由实现取。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum PollMode {
    Poll,
    WaitLatest,
    WaitOwnSubmission(SubmissionToken),
}

/// 一次提交里第一张缓冲上派的内核：空转（时长由标定定）或只写一个字。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum WorkKind {
    Spin,
    SmallKernel,
}

pub(super) trait CardHost {
    fn card(&self) -> Card;
    /// 开着的这张卡在 `nvidia-smi` 两种出口里的总线号（去掉域）：读数只认这一张的那一节、那几行。
    fn bus_of_the_card(&self) -> String;
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
pub(super) struct RawLog {
    pub(super) entries: Vec<(String, String)>,
}

struct TakenReading {
    pub(super) used_mebibytes: u64,
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
    let reading = match parse_card_reading(&text, &host.bus_of_the_card(), host.process_identifier()) {
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
    let bus = host.bus_of_the_card();
    let process_identifier = host.process_identifier();
    let compute_applications = host
        .read_compute_applications_text()
        .and_then(|text| parse_compute_applications(&text, &bus, process_identifier));
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

pub(super) fn without_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join("_")
}

// ============================== 一笔借据（登记第五节「一笔借据在装置里怎么做」） ==============================

#[derive(Clone, Copy)]
struct DeviceBaselines {
    pub(super) device_mebibytes: u64,
    pub(super) device_spread_mebibytes: u64,
    pub(super) allocator_device: Option<AllocatorFigures>,
}

struct LeasePlan {
    pub(super) job: ChildJob,
    pub(super) lease_index: u32,
    pub(super) work_microseconds: u64,
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
    let bus = host.bus_of_the_card();
    let process_identifier = host.process_identifier();
    for (sample_index, (started_at, finished_at, text)) in raw_samples.into_iter().enumerate() {
        let mut used_mebibytes = None;
        if let Ok(text) = text {
            if sample_index == 0 {
                raw_log.entries.push((format!("lease{}_after_first", record.lease_index), text.clone()));
            }
            if let Ok(reading) = parse_card_reading(&text, &bus, process_identifier) {
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

pub(super) struct ChildOutcome {
    pub(super) leases: Vec<LeaseRecord>,
    pub(super) child_void: Vec<VoidReason>,
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

pub(super) fn run_child_procedure<Host: CardHost>(
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

pub(super) const SIMULATED_PROCESS_IDENTIFIER: u32 = 4242;
/// 模型卡在它自己造的报告里用的总线号（去掉域）：只在模型里自洽，不拿它开真卡。
const SIMULATED_CARD_BUS: &str = "01:00.0";
pub(super) const SIMULATED_RUNTIME_MEBIBYTES: u64 = 300;
const SIMULATED_REPORT_COST_MICROSECONDS: u64 = 50_000;
const SIMULATED_SMALL_KERNEL_MICROSECONDS: u64 = 2_000;
const SIMULATED_BLOCK_MEBIBYTES: u64 = 256;
const SIMULATED_DEDICATED_ALLOCATION_THRESHOLD_BYTES: u64 = SIMULATED_BLOCK_MEBIBYTES * MEBIBYTE;
const SIMULATED_WAIT_OVERHEAD_MICROSECONDS: u64 = 100;
const SIMULATED_POLL_OVERHEAD_MICROSECONDS: u64 = 50;

pub(super) struct SimulatedPiece {
    bytes: u64,
    is_dropped: bool,
    is_released: bool,
    completes_at_microseconds: u64,
}

pub(super) struct SimulatedCardHost {
    pub(super) card: Card,
    clock_microseconds: std::cell::Cell<u64>,
    spin_microseconds: u64,
    block_is_retained: bool,
    pub(super) pieces: Vec<SimulatedPiece>,
    submission_completions: Vec<u64>,
    coexisting_bytes: u64,
    /// 单测用：报告里多列一个别的进程号（登记 V1）。
    pub(super) foreign_process_identifier: Option<u32>,
    /// 单测用：compute-apps 那一份比 `-q` 那一份多出的 MiB（登记 B5）。
    pub(super) compute_applications_extra_mebibytes: u64,
}

impl SimulatedCardHost {
    pub(super) fn new(card: Card, spin_microseconds: u64) -> SimulatedCardHost {
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

    pub(super) fn process_used_mebibytes(&self) -> u64 {
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

    pub(super) fn report_text(&self) -> String {
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
            SIMULATED_CARD_BUS,
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

    fn bus_of_the_card(&self) -> String {
        SIMULATED_CARD_BUS.to_string()
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
            SIMULATED_CARD_BUS,
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
pub(super) struct ChildDoneRecord {
    pub(super) child_void: Vec<VoidReason>,
}

#[derive(Default, Clone, Debug)]
pub(super) struct RoundData {
    pub(super) done: Option<ChildDoneRecord>,
    pub(super) failure_detail: Option<String>,
    pub(super) leases: Vec<LeaseRecord>,
}

pub(super) type ProductRounds = BTreeMap<ChildJob, RoundData>;

fn job_of_fields(fields: &BTreeMap<String, String>) -> Result<ChildJob, String> {
    let round: u32 = fields.get("round").ok_or("缺字段 round")?.parse().map_err(|_| "round 不是整数".to_string())?;
    Ok(ChildJob { cell: CellKey::from_fields(fields)?, round })
}

/// 产物里 `name=lease`、`name=child_done`、`name=child_failed` 三种行按轮归拢；别的行不管。
pub(super) fn parse_product(text: &str) -> Result<ProductRounds, String> {
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
