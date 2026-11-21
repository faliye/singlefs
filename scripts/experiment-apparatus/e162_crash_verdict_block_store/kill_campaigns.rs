//! 杀进程战役的状态、逐轮杀点与整次战役。

use super::*;

// ============================== 杀进程驱动（S1 主格、S1-large、PC-L、PC-T 共用） ==============================

/// 一串杀进程的趟：同一个库，写入进程每趟从「最后一个已确认块的下一块」写起。
pub(super) struct KillCampaign {
    label: String,
    pub(super) candidate: Candidate,
    pub(super) write_path: WritePath,
    pub(super) library: PathBuf,
    pub(super) seed: u64,
    /// 每块提交时延中位 m（校准趟）。
    commit_median_microseconds: u64,
    /// 前缀：[0, confirmed_end) 已确认，[confirmed_end, started_end) 在途。
    pub(super) confirmed_end: u64,
    pub(super) started_end: u64,
    /// 这条臂各次写入子进程自报的打开用时（给打开中途的杀点）。
    open_times_microseconds: Vec<u64>,
    /// S1-large：每次杀后怎么挑要核的块（None 是核 [0, started_end + 64) 全部）。
    pub(super) large_library_sampling: Option<u64>,
    pub(super) cumulative_lost: BTreeSet<u64>,
    pub(super) cumulative_corrupted: BTreeSet<u64>,
    pub(super) cumulative_confirmed_read_errors: BTreeSet<u64>,
    pub(super) cumulative_unconfirmed_read_errors: BTreeSet<u64>,
    pub(super) cumulative_phantoms: BTreeSet<String>,
    pub(super) open_failures: u64,
    verification_timeouts: u64,
    pub(super) repaired_openings: u64,
    pub(super) kills_inside_commit: u64,
    pub(super) write_phase_kills: u64,
    pub(super) confirmations_before_write_kills: u64,
    reopen_microseconds: Vec<u64>,
    /// 每次核里 Q1a 或 Q1c 或 Q1d 或 Q1e 有没有（按次数算的，给 PC-L、PC-T）。
    pub(super) write_kills_with_loss: u64,
    pub(super) write_kills_with_new_loss: u64,
    pub(super) write_kills_with_read_error: u64,
    /// Q1a–Q1f 各自在第几次杀后第一次涨、涨过几次核（轨迹）。
    first_rise: BTreeMap<&'static str, usize>,
    rises: BTreeMap<&'static str, u64>,
    pub(super) assertion_failures: Vec<String>,
    stopped_after_unopenable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LastProgressLine {
    None,
    Open,
    Begin,
    Commit,
}

impl KillCampaign {
    pub(super) fn new(label: &str, candidate: Candidate, write_path: WritePath, library: PathBuf, seed: u64, commit_median_microseconds: u64) -> Self {
        KillCampaign {
            label: label.to_string(),
            candidate,
            write_path,
            library,
            seed,
            commit_median_microseconds,
            confirmed_end: 0,
            started_end: 0,
            open_times_microseconds: Vec::new(),
            large_library_sampling: None,
            cumulative_lost: BTreeSet::new(),
            cumulative_corrupted: BTreeSet::new(),
            cumulative_confirmed_read_errors: BTreeSet::new(),
            cumulative_unconfirmed_read_errors: BTreeSet::new(),
            cumulative_phantoms: BTreeSet::new(),
            open_failures: 0,
            verification_timeouts: 0,
            repaired_openings: 0,
            kills_inside_commit: 0,
            write_phase_kills: 0,
            confirmations_before_write_kills: 0,
            reopen_microseconds: Vec::new(),
            write_kills_with_loss: 0,
            write_kills_with_new_loss: 0,
            write_kills_with_read_error: 0,
            first_rise: BTreeMap::new(),
            rises: BTreeMap::new(),
            assertion_failures: Vec::new(),
            stopped_after_unopenable: false,
        }
    }

    pub(super) fn writer_arguments(&self) -> Vec<String> {
        vec![
            "writer".to_string(),
            format!("candidate={}", self.candidate.label()),
            format!("write_path={}", self.write_path.label()),
            format!("library={}", self.library.display()),
            format!("seed={}", self.seed),
            "pattern=random".to_string(),
            format!("states={PRIMARY_STATES_PER_BLOCK}"),
            "order=interleaved".to_string(),
            "batch=1".to_string(),
            format!("first={}", self.confirmed_end),
            "end=0".to_string(),
            "sleep=0".to_string(),
            "timed=0".to_string(),
            "probe=0".to_string(),
            "progress=every-block".to_string(),
        ]
    }

    fn verification_plan(&self, cycle_index: usize, final_full_check: bool) -> VerificationPlan {
        let checked_end = self.started_end + NOT_STARTED_BLOCKS_READ_AFTER_STARTED;
        let checked_blocks = match self.large_library_sampling {
            Some(old_block_count) if !final_full_check => {
                let mut generator = SplitMix64::new(SEED_LARGE ^ u64::try_from(cycle_index).expect("装得进 u64"));
                let mut listed: BTreeSet<u64> = (old_block_count..checked_end).collect();
                while listed.len() < usize::try_from(checked_end - old_block_count).expect("装得进 usize") + CRASH_LARGE_SAMPLED_OLD_BLOCKS {
                    listed.insert(generator.next_output() % old_block_count);
                }
                CheckedBlocks::Listed(listed.into_iter().collect())
            }
            Some(_) | None => CheckedBlocks::Range { start: 0, end: checked_end },
        };
        VerificationPlan {
            seed: self.seed,
            pattern: ValuePattern::Random,
            states_per_block: PRIMARY_STATES_PER_BLOCK,
            confirmed_end: self.confirmed_end,
            started_end: self.started_end,
            checked_blocks,
        }
    }

    fn note_rise(&mut self, quantity: &'static str, cycle_index: usize) {
        self.first_rise.entry(quantity).or_insert(cycle_index);
        *self.rises.entry(quantity).or_insert(0) += 1;
    }

    /// 一趟：起写入子进程 → 按计划杀 → 读到 EOF → 起核对子进程逐块核。返回这一趟的产物行。
    pub(super) fn run_cycle(&mut self, row: &KillPlanRow, final_full_check: bool) -> String {
        let (mut child, receiver) = spawn_child(&self.writer_arguments());
        let mut lines: Vec<String> = Vec::new();
        let mut confirmations_this_cycle = 0u64;
        let delay_microseconds = match row.phase {
            KillPhase::DuringWrite { confirmations_before_kill } => {
                while confirmations_this_cycle < confirmations_before_kill {
                    let Ok(line) = receiver.recv() else { break };
                    if line.starts_with("C ") {
                        confirmations_this_cycle += 1;
                    }
                    lines.push(line);
                }
                row.delay_fraction * WRITE_PHASE_DELAY_FACTOR * self.commit_median_microseconds as f64
            }
            KillPhase::DuringOpen => row.delay_fraction * OPEN_PHASE_DELAY_FACTOR * median_of(&self.open_times_microseconds) as f64,
        };
        std::thread::sleep(Duration::from_secs_f64(delay_microseconds / 1e6));
        kill_and_reap(&mut child);
        lines.extend(drain_lines(&receiver));
        let mut last_line = LastProgressLine::None;
        let mut writer_error = String::from("-");
        let mut confirmations_total = 0u64;
        for line in &lines {
            let mut words = line.split(' ');
            match (words.next(), words.next().and_then(|number| number.parse::<u64>().ok())) {
                (Some("O"), Some(open_microseconds)) => {
                    self.open_times_microseconds.push(open_microseconds);
                    last_line = LastProgressLine::Open;
                }
                (Some("B"), Some(block_index)) => {
                    self.started_end = self.started_end.max(block_index + 1);
                    last_line = LastProgressLine::Begin;
                }
                (Some("C"), Some(block_index)) => {
                    self.confirmed_end = self.confirmed_end.max(block_index + 1);
                    confirmations_total += 1;
                    last_line = LastProgressLine::Commit;
                }
                (Some("E"), _) => writer_error = line.replace(' ', "_"),
                _ => {}
            }
        }
        let is_write_phase = matches!(row.phase, KillPhase::DuringWrite { .. });
        if is_write_phase {
            self.write_phase_kills += 1;
            self.confirmations_before_write_kills += confirmations_this_cycle;
            if last_line == LastProgressLine::Begin {
                self.kills_inside_commit += 1;
            }
            if confirmations_this_cycle < confirmations_of(row.phase) {
                self.assertion_failures.push(format!("A-S1 cycle={} confirmations={confirmations_this_cycle}", row.cycle_index));
            }
        }
        let plan = self.verification_plan(row.cycle_index, final_full_check);
        let checked_count = u64::try_from(plan.checked_list().len()).expect("装得进 u64");
        let verification = self.verify_with_timeout(&plan);
        let mut line = format!(
            "name=kill_cycle campaign={} arm={} write_path={} cycle={} phase={} delay_microseconds={:.0} confirmations_this_cycle={confirmations_this_cycle} confirmations_seen={confirmations_total} last_line={last_line:?} writer_error={writer_error} confirmed_end={} started_end={}",
            self.label,
            self.candidate.label(),
            self.write_path.label(),
            row.cycle_index,
            if is_write_phase { "write" } else { "open" },
            delay_microseconds,
            self.confirmed_end,
            self.started_end,
        );
        match verification {
            None => {
                self.open_failures += 1;
                self.verification_timeouts += 1;
                self.stopped_after_unopenable = true;
                self.note_rise("q1f", row.cycle_index);
                line.push_str(&format!(" verification=timeout_{VERIFICATION_TIMEOUT_SECONDS}s"));
            }
            Some(report) => {
                self.reopen_microseconds.push(report.open_microseconds);
                if report.open_failed {
                    self.open_failures += 1;
                    self.note_rise("q1f", row.cycle_index);
                    if report.opened_after_repair == Some(true) {
                        self.repaired_openings += 1;
                    } else {
                        self.stopped_after_unopenable = true;
                    }
                }
                let checked_sum = report.confirmed_checked + report.in_flight_checked + report.not_started_checked;
                if !report.open_failed && checked_sum != checked_count {
                    self.assertion_failures.push(format!("A-S1 cycle={} checked_sum={checked_sum} checked={checked_count}", row.cycle_index));
                }
                let new_loss = report.lost_confirmed.iter().any(|&block| !self.cumulative_lost.contains(&block));
                let before = [
                    self.cumulative_lost.len(),
                    self.cumulative_corrupted.len(),
                    self.cumulative_confirmed_read_errors.len(),
                    self.cumulative_unconfirmed_read_errors.len(),
                    self.cumulative_phantoms.len(),
                ];
                self.cumulative_lost.extend(report.lost_confirmed.iter().copied());
                self.cumulative_corrupted.extend(report.silently_corrupted.iter().copied());
                self.cumulative_confirmed_read_errors.extend(report.confirmed_read_errors.iter().copied());
                self.cumulative_unconfirmed_read_errors.extend(report.unconfirmed_read_errors.iter().copied());
                self.cumulative_phantoms.extend(report.phantom_keys.iter().cloned());
                let after = [
                    self.cumulative_lost.len(),
                    self.cumulative_corrupted.len(),
                    self.cumulative_confirmed_read_errors.len(),
                    self.cumulative_unconfirmed_read_errors.len(),
                    self.cumulative_phantoms.len(),
                ];
                for (quantity, (old, new)) in ["q1a", "q1b", "q1c", "q1d", "q1e"].into_iter().zip(before.into_iter().zip(after)) {
                    if new > old {
                        self.note_rise(quantity, row.cycle_index);
                    }
                }
                if is_write_phase {
                    if !report.lost_confirmed.is_empty() {
                        self.write_kills_with_loss += 1;
                    }
                    if new_loss {
                        self.write_kills_with_new_loss += 1;
                    }
                    if !report.confirmed_read_errors.is_empty() || !report.unconfirmed_read_errors.is_empty() {
                        self.write_kills_with_read_error += 1;
                    }
                }
                line.push_str(&format!(
                    " open_failed={} opened_after_repair={:?} reopen_microseconds={} checked={checked_count} q1a={} q1b={} q1c={} q1d={} q1e={} enumerated_keys={} first_read_error={} open_error={}",
                    report.open_failed,
                    report.opened_after_repair,
                    report.open_microseconds,
                    report.lost_confirmed.len(),
                    report.silently_corrupted.len(),
                    report.confirmed_read_errors.len(),
                    report.unconfirmed_read_errors.len(),
                    report.phantom_keys.len(),
                    report.enumerated_key_count,
                    if report.first_read_error.is_empty() { "-".to_string() } else { report.first_read_error.replace(' ', "_") },
                    if report.open_error.is_empty() { "-".to_string() } else { report.open_error.replace(' ', "_") },
                ));
                if self.large_library_sampling.is_some() {
                    // [10⁵ + 已确认的新块数, 10⁵ + 已开始的新块数]：写入是顺序前缀，所以就是 [confirmed_end, started_end]。
                    let within = report.enumerated_key_count >= self.confirmed_end && report.enumerated_key_count <= self.started_end;
                    line.push_str(&format!(" key_count_within_bounds={within}"));
                    if !within && !report.open_failed {
                        self.assertion_failures.push(format!("S1-large key count cycle={}", row.cycle_index));
                    }
                }
            }
        }
        line
    }

    /// 起核对子进程，3 600 秒还没交回就对它的 pid 发 SIGKILL（记 Q1f 超时一次，返回 None）。
    fn verify_with_timeout(&self, plan: &VerificationPlan) -> Option<VerificationReport> {
        let checked = match &plan.checked_blocks {
            CheckedBlocks::Range { start, end } => format!("range:{start}:{end}"),
            CheckedBlocks::Listed(blocks) => format!("list:{}", join_numbers(blocks)),
        };
        let arguments = vec![
            "verify".to_string(),
            format!("candidate={}", self.candidate.label()),
            format!("library={}", self.library.display()),
            format!("seed={}", plan.seed),
            format!("pattern={}", plan.pattern.label()),
            format!("states={}", plan.states_per_block),
            format!("confirmed_end={}", plan.confirmed_end),
            format!("started_end={}", plan.started_end),
            format!("checked={checked}"),
        ];
        let (mut child, receiver) = spawn_child(&arguments);
        let deadline = Instant::now() + Duration::from_secs(VERIFICATION_TIMEOUT_SECONDS);
        loop {
            match receiver.recv_timeout(Duration::from_millis(200)) {
                Ok(line) => {
                    if let Some(report) = decode_verification_report(&line) {
                        child.wait().expect("收得了核对子进程");
                        return Some(report);
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    child.wait().expect("收得了核对子进程");
                    panic!("核对子进程没交回 VERIFY 行就退出了（{}）", self.label);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if Instant::now() >= deadline {
                        kill_and_reap(&mut child);
                        return None;
                    }
                }
            }
        }
    }

    fn summary_line(&self) -> String {
        let mut sorted = self.reopen_microseconds.clone();
        sorted.sort_unstable();
        let over_sixty_seconds = sorted.iter().filter(|&&microseconds| microseconds > 60_000_000).count();
        let rises: Vec<String> = self.rises.iter().map(|(quantity, count)| format!("{quantity}:{count}")).collect();
        let first: Vec<String> = self.first_rise.iter().map(|(quantity, cycle)| format!("{quantity}:{cycle}")).collect();
        format!(
            "name=kill_campaign_summary campaign={} arm={} write_path={} cycles_verified={} q1a_lost={} q1b_corrupted={} q1c_confirmed_read_errors={} q1d_unconfirmed_read_errors={} q1e_phantoms={} q1f_open_failures={} q1f_timeouts={} repaired_openings={} q1g_kills_inside_commit={} write_phase_kills={} confirmations_before_write_kills={} write_kills_with_loss={} write_kills_with_new_loss={} write_kills_with_read_error={} reopen_peak_microseconds={} reopen_median_microseconds={} reopen_last_microseconds={} reopen_over_60s={over_sixty_seconds} rises={} first_rise_cycle={} stopped_after_unopenable={} assertion_failures={}",
            self.label,
            self.candidate.label(),
            self.write_path.label(),
            self.reopen_microseconds.len(),
            self.cumulative_lost.len(),
            self.cumulative_corrupted.len(),
            self.cumulative_confirmed_read_errors.len(),
            self.cumulative_unconfirmed_read_errors.len(),
            self.cumulative_phantoms.len(),
            self.open_failures,
            self.verification_timeouts,
            self.repaired_openings,
            self.kills_inside_commit,
            self.write_phase_kills,
            self.confirmations_before_write_kills,
            self.write_kills_with_loss,
            self.write_kills_with_new_loss,
            self.write_kills_with_read_error,
            sorted.last().copied().unwrap_or(0),
            sorted_percentile(&sorted, 1, 2),
            self.reopen_microseconds.last().copied().unwrap_or(0),
            if rises.is_empty() { "-".to_string() } else { rises.join(",") },
            if first.is_empty() { "-".to_string() } else { first.join(",") },
            self.stopped_after_unopenable,
            if self.assertion_failures.is_empty() { "-".to_string() } else { self.assertion_failures.join(",") },
        )
    }
}

pub(super) fn run_kill_campaign(output: &mut ProductLines, campaign: &mut KillCampaign, plan: &[KillPlanRow]) {
    let last_cycle = plan.len().saturating_sub(1);
    for row in plan {
        let final_full_check = campaign.large_library_sampling.is_some() && row.cycle_index == last_cycle;
        let line = campaign.run_cycle(row, final_full_check);
        output.line(line);
        if campaign.stopped_after_unopenable {
            output.line(format!(
                "name=kill_campaign_stopped campaign={} arm={} after_cycle={} reason=library_unopenable remaining_cycles_not_run={}",
                campaign.label,
                campaign.candidate.label(),
                row.cycle_index,
                last_cycle - row.cycle_index,
            ));
            break;
        }
    }
    output.line(campaign.summary_line());
}
