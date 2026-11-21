//! 回环格的设置与中断记录、一格的跑法、转发进程、带干扰重跑的一格。

use super::*;

// ============================== 回环格：一格怎么跑 ==============================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct LoopbackCell {
    pub(super) label: &'static str,
    pub(super) direct_blocks: u64,
    pub(super) network_blocks: u64,
    pub(super) states_per_block: u64,
    pub(super) window_frames: usize,
    pub(super) receive_mode: ReceiveMode,
    pub(super) interruption_count: usize,
    /// V15 管的计时格（开始与结束各看一次干扰进程）。
    pub(super) is_timed: bool,
}

impl LoopbackCell {
    pub(super) fn throughput_shape(label: &'static str, window_frames: usize, states_per_block: u64, receive_mode: ReceiveMode) -> LoopbackCell {
        LoopbackCell {
            label,
            direct_blocks: PATH_BLOCK_COUNT,
            network_blocks: PATH_BLOCK_COUNT,
            states_per_block,
            window_frames,
            receive_mode,
            interruption_count: 0,
            is_timed: true,
        }
    }

    pub(super) fn consistency_shape(label: &'static str, receive_mode: ReceiveMode, interruption_count: usize) -> LoopbackCell {
        LoopbackCell {
            label,
            direct_blocks: PATH_BLOCK_COUNT,
            network_blocks: PATH_BLOCK_COUNT,
            states_per_block: PRIMARY_STATES_PER_BLOCK,
            window_frames: PRIMARY_WINDOW_FRAMES,
            receive_mode,
            interruption_count,
            is_timed: false,
        }
    }

    pub(super) fn rate_control_shape(label: &'static str, receive_mode: ReceiveMode) -> LoopbackCell {
        LoopbackCell {
            label,
            direct_blocks: 0,
            network_blocks: RATE_CONTROL_BLOCKS,
            states_per_block: PRIMARY_STATES_PER_BLOCK,
            window_frames: PRIMARY_WINDOW_FRAMES,
            receive_mode,
            interruption_count: 0,
            is_timed: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct InterruptionRecord {
    pub(super) index: usize,
    pub(super) threshold: u64,
    pub(super) acknowledged_seen: u64,
    pub(super) pause_microseconds: u64,
    pub(super) forwarder_restarted: bool,
}

#[derive(Debug, Clone, Default)]
pub(super) struct NetworkCellRun {
    pub(super) sender_summary: String,
    pub(super) sender_windows: String,
    pub(super) sender_reconnections: Vec<String>,
    pub(super) sender_exit_code: i32,
    pub(super) library_direct: String,
    pub(super) library_network: String,
    pub(super) library_exit_code: i32,
    pub(super) interruptions: Vec<InterruptionRecord>,
    pub(super) verification: VerificationReport,
    pub(super) setup_problem: String,
}

impl NetworkCellRun {
    pub(super) fn sender_blocks_per_second(&self) -> f64 {
        let acknowledged = number_field(&self.sender_summary, "acknowledged") as f64;
        let elapsed_seconds = number_field(&self.sender_summary, "elapsed_microseconds") as f64 / 1_000_000.0;
        if elapsed_seconds > 0.0 {
            acknowledged / elapsed_seconds
        } else {
            0.0
        }
    }

    pub(super) fn direct_blocks_per_second(&self) -> f64 {
        let confirmed = number_field(&self.library_direct, "confirmed") as f64;
        let elapsed_seconds = number_field(&self.library_direct, "elapsed_microseconds") as f64 / 1_000_000.0;
        if elapsed_seconds > 0.0 {
            confirmed / elapsed_seconds
        } else {
            0.0
        }
    }

    pub(super) fn read_errors(&self) -> u64 {
        u64::try_from(self.verification.confirmed_read_errors.len() + self.verification.unconfirmed_read_errors.len()).expect("装得进 u64")
    }
}

fn spawn_forwarder(listen_port: u16, target_port: u16) -> Option<(RunningChild, u16)> {
    let arguments = vec![
        "forward".to_string(),
        "--listen-port".to_string(),
        listen_port.to_string(),
        "--target-port".to_string(),
        target_port.to_string(),
    ];
    let mut running = spawn_running_child(child_command(&arguments), false);
    let mut seen = Vec::new();
    match wait_for_line(&running, "F port=", &mut seen) {
        Some(line) => {
            let port = u16::try_from(number_field(&line, "port")).expect("端口装得进 u16");
            Some((running, port))
        }
        None => {
            kill_and_reap(&mut running.child);
            None
        }
    }
}

/// 在子进程里核，3 600 秒没交回就杀（登记 5.5）。
fn verify_in_child_with_timeout(candidate: Candidate, library: &Path, plan: &PathVerificationPlan) -> VerificationReport {
    let mut running = spawn_running_child(child_command(&plan.to_arguments(candidate, library)), false);
    let deadline = Instant::now() + Duration::from_secs(VERIFICATION_TIMEOUT_SECONDS);
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        match running.lines.recv_timeout(remaining) {
            Ok(line) if line.starts_with("V ") => {
                let _exit_status = running.child.wait();
                return decode_verification_line(&line);
            }
            Ok(_other_line) => continue,
            Err(_ended_or_timed_out) => {
                kill_and_reap(&mut running.child);
                return VerificationReport { verification_missing: true, ..VerificationReport::default() };
            }
        }
    }
}

/// 核对子进程的结果行只带计数与前 10 个块号；父进程按计数还原成同样长的列表（块号只在产物行里看前 10 个）。
pub(super) fn decode_verification_line(line: &str) -> VerificationReport {
    let first_list = |key: &str| -> Vec<u64> {
        field_value(line, key)
            .filter(|text| *text != "none")
            .map(|text| text.split(',').filter_map(|number| number.parse().ok()).collect())
            .unwrap_or_default()
    };
    let padded = |count: u64, first: Vec<u64>| -> Vec<u64> {
        let mut numbers = first;
        numbers.resize(usize::try_from(count).expect("装得进 usize"), u64::MAX);
        numbers
    };
    let read_error_first = first_list("read_errors_first");
    let confirmed_read_errors = number_field(line, "confirmed_read_errors");
    let unconfirmed_read_errors = number_field(line, "unconfirmed_read_errors");
    let phantom_count = number_field(line, "phantom_keys");
    VerificationReport {
        open_failed: field_value(line, "open_failed") == Some("true"),
        open_error: field_value(line, "open_error").unwrap_or("none").to_string(),
        opened_after_repair: match field_value(line, "opened_after_repair") {
            Some("true") => Some(true),
            Some("false") => Some(false),
            _ => None,
        },
        open_microseconds: number_field(line, "open_microseconds"),
        confirmed_checked: number_field(line, "confirmed_checked"),
        in_flight_checked: number_field(line, "in_flight_checked"),
        not_started_checked: number_field(line, "not_started_checked"),
        lost_confirmed: padded(number_field(line, "lost_confirmed"), first_list("lost_confirmed_first")),
        silently_corrupted: padded(number_field(line, "silently_corrupted"), first_list("silently_corrupted_first")),
        confirmed_read_errors: padded(confirmed_read_errors, read_error_first.iter().take(usize::try_from(confirmed_read_errors).expect("usize")).copied().collect()),
        unconfirmed_read_errors: padded(unconfirmed_read_errors, Vec::new()),
        phantom_keys: (0..phantom_count).map(|phantom_index| format!("phantom-{phantom_index}")).collect(),
        enumerated_key_count: number_field(line, "enumerated_keys"),
        enumerated_value_bytes: number_field(line, "enumerated_value_bytes"),
        enumeration_error: field_value(line, "enumeration_error").unwrap_or("none").to_string(),
        first_read_error: field_value(line, "first_read_error").unwrap_or("none").to_string(),
        verification_missing: false,
    }
}

/// 一格回环：起库进程（直写线程 + 收块线程）、转发进程、送块进程；按送块进程报的确认数中断转发进程；
/// 送块进程退出之后停转发、关库进程的标准输入让它收尾；再在核对子进程里逐块核两路。
pub(super) fn run_loopback_cell(candidate: Candidate, cell: &LoopbackCell, scratch: &Path) -> NetworkCellRun {
    let mut run = NetworkCellRun::default();
    let library = scratch.join(format!("library-{}-{}", candidate.label(), cell.label));
    if library.exists() {
        fs::remove_dir_all(&library).expect("删得了上一次的库");
    }
    let library_settings = LibrarySettings {
        candidate,
        library: library.clone(),
        receive_mode: cell.receive_mode,
        direct_blocks: cell.direct_blocks,
        network_blocks: cell.network_blocks,
        states_per_block: cell.states_per_block,
        channel: ReceiveChannel::TcpListen,
    };
    let mut library_process = spawn_running_child(child_command(&library_settings.to_arguments()), true);
    let mut library_lines = Vec::new();
    let Some(port_line) = wait_for_line(&library_process, "L port=", &mut library_lines) else {
        kill_and_reap(&mut library_process.child);
        run.setup_problem = format!("library_did_not_start:{}", text_field(&library_lines.join("|")));
        return run;
    };
    let library_port = u16::try_from(number_field(&port_line, "port")).expect("端口装得进 u16");
    let Some((mut forwarder, forward_port)) = spawn_forwarder(0, library_port) else {
        kill_and_reap(&mut library_process.child);
        run.setup_problem = "forwarder_did_not_start".to_string();
        return run;
    };
    let sender_settings = SenderSettings {
        endpoint: SenderEndpoint::ConnectTcp(format!("127.0.0.1:{forward_port}")),
        block_count: cell.network_blocks,
        states_per_block: cell.states_per_block,
        window_frames: cell.window_frames,
        pace_between_first_transmissions: None,
        acknowledgement_stall_limit: Duration::from_secs(DEFAULT_ACKNOWLEDGEMENT_STALL_SECONDS),
        spinning_thread_count: 0,
    };
    let mut sender = spawn_running_child(sender_command(&sender_arguments(&sender_settings)), false);
    let plan = interruption_plan(cell.interruption_count, cell.network_blocks.max(2));
    let mut next_interruption = 0usize;
    for line in sender.lines.iter() {
        if let Some(count_text) = line.strip_prefix("A ") {
            let acknowledged: u64 = count_text.trim().parse().unwrap_or(0);
            while next_interruption < plan.len() && acknowledged >= plan[next_interruption].acknowledgement_threshold {
                let interruption = plan[next_interruption];
                kill_and_reap(&mut forwarder.child);
                let paused_at = Instant::now();
                std::thread::sleep(pause_of(&interruption));
                let pause_microseconds = u64::try_from(paused_at.elapsed().as_micros()).expect("微秒装得进 u64");
                let restarted = spawn_forwarder(forward_port, library_port);
                let forwarder_restarted = restarted.is_some();
                if let Some((replacement, _same_port)) = restarted {
                    forwarder = replacement;
                }
                run.interruptions.push(InterruptionRecord {
                    index: next_interruption,
                    threshold: interruption.acknowledgement_threshold,
                    acknowledged_seen: acknowledged,
                    pause_microseconds,
                    forwarder_restarted,
                });
                next_interruption += 1;
            }
        } else if line.starts_with("R ") {
            run.sender_reconnections.push(line);
        } else if line.starts_with("W ") {
            run.sender_windows = line;
        } else if line.starts_with("S ") {
            run.sender_summary = line;
        }
    }
    run.sender_exit_code = sender.child.wait().map(|status| status.code().unwrap_or(-1)).unwrap_or(-1);
    kill_and_reap(&mut forwarder.child);
    drop(library_process.input.take());
    for line in library_process.lines.iter() {
        if line.starts_with("D ") {
            run.library_direct = line;
        } else if line.starts_with("N ") {
            run.library_network = line;
        }
    }
    run.library_exit_code = library_process.child.wait().map(|status| status.code().unwrap_or(-1)).unwrap_or(-1);
    let network_acknowledged = expand_true_positions(field_value(&run.sender_summary, "acknowledged_ranges").unwrap_or("none"), cell.network_blocks)
        .unwrap_or_else(|| vec![false; usize::try_from(cell.network_blocks).expect("块数装得进 usize")]);
    let verification_plan = PathVerificationPlan {
        states_per_block: cell.states_per_block,
        direct_blocks: cell.direct_blocks,
        direct_confirmed: number_field(&run.library_direct, "confirmed"),
        direct_started: number_field(&run.library_direct, "started"),
        network_blocks: cell.network_blocks,
        network_acknowledged,
        network_sent: number_field(&run.sender_summary, "sent_positions"),
    };
    run.verification = verify_in_child_with_timeout(candidate, &library, &verification_plan);
    let _library_removed = fs::remove_dir_all(&library);
    run
}

// ============================== 回环格：驱动、产物行与判定 ==============================

/// 一格的一次跑（计时格 V15 干扰就重跑，最多重跑三次）。
pub(super) struct CellAttempt {
    pub(super) run: NetworkCellRun,
    pub(super) attempt: usize,
    pub(super) is_interfered: bool,
}

pub(super) fn run_cell_with_interference_reruns(output: &mut ProductLines, candidate: Candidate, cell: &LoopbackCell, scratch: &Path) -> CellAttempt {
    let mut attempt = 0usize;
    loop {
        attempt += 1;
        let waited_seconds = if cell.is_timed { wait_for_quiet_machine(INTERFERENCE_MAXIMUM_WAIT_SECONDS) } else { 0 };
        let interference_at_start = interfering_processes();
        let load_at_start = load_average_text();
        let sectors_at_start = nvme_written_sectors();
        let run = run_loopback_cell(candidate, cell, scratch);
        let interference_at_end = interfering_processes();
        let is_interfered = cell.is_timed && !(interference_at_start.is_empty() && interference_at_end.is_empty());
        output.line(format!(
            "name=cell arm={} cell={} attempt={attempt} direct_blocks={} network_blocks={} states_per_block={} window_frames={} receive_mode={} interruptions_planned={} is_timed={} waited_for_quiet_seconds={waited_seconds} interference_at_start={} interference_at_end={} load_at_start={load_at_start} load_at_end={} nvme_written_sectors_at_start={sectors_at_start} nvme_written_sectors_at_end={} setup_problem={}",
            candidate.label(),
            cell.label,
            cell.direct_blocks,
            cell.network_blocks,
            cell.states_per_block,
            cell.window_frames,
            cell.receive_mode.label(),
            cell.interruption_count,
            cell.is_timed,
            list_or_dash(&interference_at_start),
            list_or_dash(&interference_at_end),
            load_average_text(),
            nvme_written_sectors(),
            text_field(&run.setup_problem),
        ));
        emit_cell_lines(output, candidate, cell, attempt, &run);
        if !is_interfered || attempt > INTERFERENCE_MAXIMUM_RERUNS {
            return CellAttempt { run, attempt, is_interfered };
        }
    }
}

#[cfg(test)]
#[path = "loopback_cells_tests.rs"]
mod tests;
