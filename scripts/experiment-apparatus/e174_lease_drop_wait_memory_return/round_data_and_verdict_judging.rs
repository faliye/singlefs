//! 每轮数据、列与格视图、按列与按产物的判定，以及判定行。

use super::*;

impl RoundData {
    fn lease(&self, lease_index: u32) -> Option<&LeaseRecord> {
        self.leases.iter().find(|lease| lease.lease_index == lease_index)
    }

    fn lease_is_valid(&self, lease_index: u32) -> bool {
        self.lease(lease_index).is_some_and(|lease| !lease.is_void())
    }

    /// 一轮有效：子进程没作废，第 1 笔有效，第 2、3 笔里至少一笔有效（长窗那几格只有一笔）。
    pub(super) fn is_complete(&self, lease_count: u32) -> bool {
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
pub(super) enum Column {
    First,
    Later,
}

impl Column {
    pub(super) fn label(self) -> &'static str {
        match self {
            Column::First => "first",
            Column::Later => "later",
        }
    }
}

/// 按格归拢；每格取轮号最小的 5 个有效轮（登记第六节：5 轮各自的值都列出来）。
struct CellView<'rounds> {
    pub(super) cell: CellKey,
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

pub(super) fn signed(value: u64) -> i64 {
    i64::try_from(value).expect("读数与时长装得进 i64")
}

/// 判定的可调旋钮，只有 τ：判别力自证（登记第八节）把它换成别的值，看判定翻不翻。
#[derive(Clone, Copy, Debug)]
pub(super) struct JudgeSettings {
    pub(super) tau_override_mebibytes: Option<u64>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(super) enum TimeToClear {
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

    pub(super) fn size_mebibytes(&self) -> i64 {
        signed(self.job.cell.size_mebibytes)
    }

    /// R_lease(t) = U(t) − U_pre。
    fn lease_remainder_of(&self, used_mebibytes: u64) -> i64 {
        signed(used_mebibytes) - signed(self.pre_mebibytes)
    }

    /// R_next：t0 之后第一份报告的 R_lease。
    pub(super) fn next_remainder_mebibytes(&self) -> Option<i64> {
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

    pub(super) fn time_to_clear(&self, settings: JudgeSettings) -> TimeToClear {
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
    pub(super) fn positive_reading_counts(&self, settings: JudgeSettings) -> (usize, usize) {
        let tau = signed(self.judge_tau_mebibytes(settings));
        let readable: Vec<i64> =
            self.samples.iter().filter_map(|sample| sample.used_mebibytes).map(|used| self.lease_remainder_of(used)).collect();
        (readable.iter().filter(|remainder| **remainder > tau).count(), readable.len())
    }

    /// 窗口里 R_lease 最大的那一份（t0 之后的峰值）。
    pub(super) fn peak_remainder_mebibytes(&self) -> Option<i64> {
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
pub(super) enum Presence {
    Absent,
    Present,
    Unstable,
}

impl Presence {
    pub(super) fn label(self) -> &'static str {
        match self {
            Presence::Absent => "absent",
            Presence::Present => "present",
            Presence::Unstable => "unstable",
        }
    }
}

/// 每轮一个 (值, τ)：5 轮都不大于各自的 τ 记不含，都大于记含，否则不稳定。空表也是不稳定（没有轮，不能说不含）。
pub(super) fn classify_presence(measurements: &[(i64, u64)]) -> Presence {
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

pub(super) struct ColumnVerdict {
    pub(super) cell: CellKey,
    pub(super) column: Column,
    pub(super) rounds_used: usize,
    pub(super) wait_token: &'static str,
    judged_wait_microseconds: Vec<i64>,
    pub(super) wait_microseconds: Vec<u64>,
    pub(super) submit_to_poll_microseconds: Vec<u64>,
    pub(super) work_microseconds: Vec<u64>,
    wait_never_returned: bool,
    pub(super) next_presence: Presence,
    next_measurements: Vec<(i64, u64)>,
    pub(super) clear_token: &'static str,
    clear_values: Vec<TimeToClear>,
    positive_counts: Vec<(usize, usize)>,
    peak_remainders: Vec<i64>,
    pub(super) end_presence: Presence,
    end_measurements: Vec<(i64, u64)>,
    pub(super) overtake_presence: Presence,
    overtake_measurements: Vec<(i64, u64)>,
    blocks_token: &'static str,
    reserved_kept_bytes: Vec<i64>,
    unreturned_bytes: Vec<i64>,
}

pub(super) fn wait_cost_token(cell: CellKey, judged: &[i64], never_returned: bool) -> &'static str {
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

    pub(super) fn to_line(&self) -> String {
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
pub(super) struct JudgedProduct {
    pub(super) lines: Vec<String>,
    pub(super) columns: Vec<ColumnVerdict>,
    void_cells: Vec<CellKey>,
    hold_control_failed: bool,
    /// 登记 F6：因为「在飞」档其实没在飞（K9）而凑不够 5 个有效轮的格数。
    cells_void_for_work_not_in_flight: usize,
}

fn columns_of_cell(cell: CellKey) -> &'static [Column] {
    if cell.lease_count() == 1 { &[Column::First] } else { &[Column::First, Column::Later] }
}

pub(super) fn judge_product(rounds: &ProductRounds, settings: JudgeSettings) -> JudgedProduct {
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
pub(super) fn geometry_pairs<'columns>(columns: &'columns [ColumnVerdict]) -> Vec<(&'columns ColumnVerdict, &'columns ColumnVerdict)> {
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

pub(super) fn geometry_pair_line(main: &ColumnVerdict, coexisting: &ColumnVerdict) -> String {
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
pub(super) fn count_columns_flipped_by_moving_tau(rounds: &ProductRounds, original: &JudgedProduct) -> usize {
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

pub(super) fn verdict_line(scope: &str, judged: &JudgedProduct, flipped_columns: usize, synthetic_flips: bool) -> String {
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
