//! 逐格行的输出、窗口读数、回环问题、断线路径、一致性判定与阳性对照。

use super::*;

fn prefixed_fields(line: &str, skip_first_word: bool) -> String {
    let body = if skip_first_word { line.split_once(' ').map_or("", |(_, rest)| rest) } else { line };
    if body.is_empty() {
        "missing=true".to_string()
    } else {
        body.to_string()
    }
}

/// 分窗：第 k 窗 = 第 1 000(k−1) 到第 1 000k 个确认，状态/秒 = 1 000 × 每块状态数 ÷ 窗长（第 1 窗从第一帧发出算起）。
fn window_states_per_second(end_offsets_microseconds: &[u64], states_per_block: u64) -> Vec<f64> {
    let mut previous = 0u64;
    end_offsets_microseconds
        .iter()
        .map(|&offset| {
            let window_seconds = (offset - previous) as f64 / 1_000_000.0;
            previous = offset;
            (ACKNOWLEDGEMENTS_PER_WINDOW * states_per_block) as f64 / window_seconds
        })
        .collect()
}

fn parse_window_offsets(windows_line: &str) -> Vec<u64> {
    field_value(windows_line, "end_offsets_microseconds")
        .filter(|text| *text != "none")
        .map(|text| text.split(',').filter_map(|number| number.parse().ok()).collect())
        .unwrap_or_default()
}

pub(super) fn emit_cell_lines(output: &mut ProductLines, candidate: Candidate, cell: &LoopbackCell, attempt: usize, run: &NetworkCellRun) {
    let arm = candidate.label();
    let label = cell.label;
    output.line(format!("name=sender arm={arm} cell={label} attempt={attempt} exit_code={} {}", run.sender_exit_code, prefixed_fields(&run.sender_summary, true)));
    output.line(format!("name=sender_windows arm={arm} cell={label} attempt={attempt} {}", prefixed_fields(&run.sender_windows, true)));
    for reconnection in &run.sender_reconnections {
        output.line(format!("name=sender_reconnection arm={arm} cell={label} attempt={attempt} {}", prefixed_fields(reconnection, true)));
    }
    for interruption in &run.interruptions {
        output.line(format!(
            "name=interruption arm={arm} cell={label} attempt={attempt} index={} threshold={} acknowledged_seen={} pause_microseconds={} forwarder_restarted={}",
            interruption.index, interruption.threshold, interruption.acknowledged_seen, interruption.pause_microseconds, interruption.forwarder_restarted
        ));
    }
    let direct_fields: Vec<String> = fields_of(&run.library_direct).into_iter().map(|(key, value)| format!("direct_{key}={value}")).collect();
    output.line(format!(
        "name=library arm={arm} cell={label} attempt={attempt} exit_code={} {} {}",
        run.library_exit_code,
        if direct_fields.is_empty() { "direct_missing=true".to_string() } else { direct_fields.join(" ") },
        prefixed_fields(&run.library_network, true)
    ));
    let expected_keys = cell.direct_blocks + cell.network_blocks;
    let expected_bytes = expected_keys * cell.states_per_block;
    let verification = &run.verification;
    output.line(format!(
        "name=verification arm={arm} cell={label} attempt={attempt} verification_missing={} {} q3d_lost_confirmed={} q3e_silently_corrupted={} q3f_read_errors={} q3g_phantom_keys={} a_count_expected_keys={expected_keys} a_count_expected_value_bytes={expected_bytes} a_count_matches={}",
        verification.verification_missing,
        prefixed_fields(&verification.encode(), true),
        verification.lost_confirmed.len(),
        verification.silently_corrupted.len(),
        run.read_errors(),
        verification.phantom_keys.len(),
        verification.enumerated_key_count == expected_keys && verification.enumerated_value_bytes == expected_bytes,
    ));
    let network_blocks_per_second = run.sender_blocks_per_second();
    let direct_blocks_per_second = run.direct_blocks_per_second();
    let states = cell.states_per_block as f64;
    let window_rates = window_states_per_second(&parse_window_offsets(&run.sender_windows), cell.states_per_block);
    let lowest_window = window_rates.iter().copied().fold(f64::INFINITY, f64::min);
    output.line(format!(
        "name=rates arm={arm} cell={label} attempt={attempt} q3a_blocks_per_second={network_blocks_per_second:.3} q3a_states_per_second={:.1} q3b_blocks_per_second={direct_blocks_per_second:.3} q3b_states_per_second={:.1} q3c_states_per_second={:.1} q3k_windows={} q3k_lowest_window_states_per_second={} q3k_last_window_states_per_second={} q3k_window_states_per_second={}",
        network_blocks_per_second * states,
        direct_blocks_per_second * states,
        (network_blocks_per_second + direct_blocks_per_second) * states,
        window_rates.len(),
        if window_rates.is_empty() { "none".to_string() } else { format!("{lowest_window:.1}") },
        window_rates.last().map_or("none".to_string(), |rate| format!("{rate:.1}")),
        if window_rates.is_empty() { "none".to_string() } else { window_rates.iter().map(|rate| format!("{rate:.0}")).collect::<Vec<_>>().join(",") },
    ));
}

/// Q3a 对 R_B 的判定（第八节末段的判别力自证用同一个函数）：低于门槛就翻。
fn network_throughput_flips(states_per_second: f64, threshold_states_per_second: f64) -> bool {
    states_per_second < threshold_states_per_second
}

/// PC3-rate：慢的那遍 ≤ 200 块/秒、快的那遍 > 200 块/秒（登记 5.3）。
fn rate_control_seen(fast_blocks_per_second: f64, slow_blocks_per_second: f64) -> bool {
    slow_blocks_per_second <= RATE_CONTROL_THRESHOLD_BLOCKS_PER_SECOND && fast_blocks_per_second > RATE_CONTROL_THRESHOLD_BLOCKS_PER_SECOND
}

const LOOPBACK_CELL_ORDER: [&str; 9] = ["PC3-drop", "PC3-rate", "PC3-torn", "S3-T", "S3-C", "S3-W1", "S3-W256", "S3-bs10", "S3-raw"];

pub(super) fn run_loopback_question(output: &mut ProductLines, arguments: &[String]) -> i32 {
    let Some(candidate) = arguments.first().and_then(|text| Candidate::parse(text)) else {
        eprintln!("  ✗ s3 要一个臂：R1 R0 K F");
        return 2;
    };
    let requested: Vec<&str> = if arguments.len() > 1 { arguments[1..].iter().map(String::as_str).collect() } else { LOOPBACK_CELL_ORDER.to_vec() };
    if let Some(unknown) = requested.iter().find(|label| !LOOPBACK_CELL_ORDER.contains(label)) {
        eprintln!("  ✗ 认不出的格 {unknown}；格取 {}", LOOPBACK_CELL_ORDER.join(" "));
        return 2;
    }
    if !sender_binary_path().exists() {
        eprintln!("  ✗ 找不到送块进程 {}", sender_binary_path().display());
        eprintln!("  → 怎么办：先 cargo build --release -p e7-index-bench --bin {SENDER_BINARY_NAME}（与这个 bin 编进同一个目录）");
        return 2;
    }
    if run_anchors(output) != 0 {
        return 1;
    }
    let scratch = ScratchDirectory::create();
    let (filesystem_type, available_bytes) = filesystem_type_and_available_bytes(&scratch.path);
    output.line(format!(
        "name=environment arm={} scratch={} filesystem_type={filesystem_type} available_bytes={available_bytes} logical_cpus={} kernel={}",
        candidate.label(),
        scratch.path.display(),
        std::thread::available_parallelism().map(|count| count.get()).unwrap_or(0),
        command_output_text("uname", &["-r"]),
    ));
    output.line(format!(
        "name=threshold arm={} t_state=pending_e161_segment_one peer_logical_cpus=pending_not_queried_this_segment r_b=pending r_ab=pending q3i_peer_cores_break_even=pending q3j_cross_link_bytes_per_second_needed=pending formula_q3j=r_b_times_65580_over_65536",
        candidate.label()
    ));
    if matches!(filesystem_type.as_str(), "tmpfs" | "ramfs" | "overlayfs") {
        output.line(format!("name=verdict part=environment arm={} v9_filesystem_allowed=false filesystem_type={filesystem_type}", candidate.label()));
        return 1;
    }
    if available_bytes < MINIMUM_AVAILABLE_BYTES {
        output.line(format!("name=verdict part=environment arm={} h4_available_bytes_enough=false available_bytes={available_bytes}", candidate.label()));
        return 1;
    }
    let arm = candidate.label();
    for label in requested {
        match label {
            "PC3-drop" => {
                let cell = LoopbackCell::consistency_shape("PC3-drop", ReceiveMode::DropOnePerThousand, 0);
                let attempt = run_cell_with_interference_reruns(output, candidate, &cell, &scratch.path);
                let lost = u64::try_from(attempt.run.verification.lost_confirmed.len()).expect("装得进 u64");
                let recorded = number_field(&attempt.run.library_network, "controlled_drops");
                output.line(format!(
                    "name=verdict part=pc3_drop arm={arm} lost_confirmed={lost} library_controlled_drops={recorded} positive_control_passed={}",
                    lost == recorded && recorded >= 1
                ));
            }
            "PC3-rate" => {
                let fast = run_cell_with_interference_reruns(output, candidate, &LoopbackCell::rate_control_shape("PC3-rate-fast", ReceiveMode::Registered), &scratch.path);
                let slow = run_cell_with_interference_reruns(output, candidate, &LoopbackCell::rate_control_shape("PC3-rate-slow", ReceiveMode::SleepBeforeCommit), &scratch.path);
                let fast_rate = fast.run.sender_blocks_per_second();
                let slow_rate = slow.run.sender_blocks_per_second();
                output.line(format!(
                    "name=verdict part=pc3_rate arm={arm} fast_blocks_per_second={fast_rate:.3} slow_blocks_per_second={slow_rate:.3} positive_control_passed={} timing_undisturbed={}",
                    rate_control_seen(fast_rate, slow_rate),
                    !fast.is_interfered && !slow.is_interfered
                ));
            }
            "PC3-torn" => {
                let mut cell = LoopbackCell::consistency_shape("PC3-torn", ReceiveMode::TornAccepting, REGISTERED_INTERRUPTIONS);
                let mut attempt = run_cell_with_interference_reruns(output, candidate, &cell, &scratch.path);
                let mut escalated = false;
                if attempt.run.verification.silently_corrupted.is_empty() {
                    escalated = true;
                    cell = LoopbackCell::consistency_shape("PC3-torn-200", ReceiveMode::TornAccepting, ESCALATED_INTERRUPTIONS);
                    attempt = run_cell_with_interference_reruns(output, candidate, &cell, &scratch.path);
                }
                output.line(format!(
                    "name=verdict part=pc3_torn arm={arm} interruptions={} escalated_to_200={escalated} silently_corrupted={} positive_control_passed={}",
                    cell.interruption_count,
                    attempt.run.verification.silently_corrupted.len(),
                    !attempt.run.verification.silently_corrupted.is_empty()
                ));
            }
            "S3-C" => {
                let mut cell = LoopbackCell::consistency_shape("S3-C", ReceiveMode::Registered, REGISTERED_INTERRUPTIONS);
                let mut attempt = run_cell_with_interference_reruns(output, candidate, &cell, &scratch.path);
                let paths_reached = disconnect_paths_reached;
                let mut escalated = false;
                if !paths_reached(&attempt.run) {
                    escalated = true;
                    cell = LoopbackCell::consistency_shape("S3-C-200", ReceiveMode::Registered, ESCALATED_INTERRUPTIONS);
                    attempt = run_cell_with_interference_reruns(output, candidate, &cell, &scratch.path);
                }
                emit_consistency_verdict(output, arm, &cell, &attempt, escalated, paths_reached(&attempt.run));
            }
            "S3-T" | "S3-W1" | "S3-W256" | "S3-bs10" | "S3-raw" => {
                let cell = match label {
                    "S3-T" => LoopbackCell::throughput_shape("S3-T", PRIMARY_WINDOW_FRAMES, PRIMARY_STATES_PER_BLOCK, ReceiveMode::Registered),
                    "S3-W1" => LoopbackCell::throughput_shape("S3-W1", STOP_AND_WAIT_WINDOW_FRAMES, PRIMARY_STATES_PER_BLOCK, ReceiveMode::Registered),
                    "S3-W256" => LoopbackCell::throughput_shape("S3-W256", WIDE_WINDOW_FRAMES, PRIMARY_STATES_PER_BLOCK, ReceiveMode::Registered),
                    "S3-bs10" => LoopbackCell::throughput_shape("S3-bs10", PRIMARY_WINDOW_FRAMES, SMALL_STATES_PER_BLOCK, ReceiveMode::Registered),
                    _ => LoopbackCell::throughput_shape("S3-raw", PRIMARY_WINDOW_FRAMES, PRIMARY_STATES_PER_BLOCK, ReceiveMode::AcknowledgeWithoutWriting),
                };
                let attempt = run_cell_with_interference_reruns(output, candidate, &cell, &scratch.path);
                let states = cell.states_per_block as f64;
                let network_states_per_second = attempt.run.sender_blocks_per_second() * states;
                let direct_states_per_second = attempt.run.direct_blocks_per_second() * states;
                output.line(format!(
                    "name=verdict part=s3_throughput arm={arm} cell={} network_states_per_second={network_states_per_second:.1} direct_states_per_second={direct_states_per_second:.1} both_paths_states_per_second={:.1} judgement_against_r_b=pending_t_state_and_peer_cores timing_undisturbed={} attempts={} sender_complete={}",
                    cell.label,
                    network_states_per_second + direct_states_per_second,
                    !attempt.is_interfered,
                    attempt.attempt,
                    field_value(&attempt.run.sender_summary, "ending") == Some("complete"),
                ));
                if label != "S3-raw" {
                    emit_consistency_verdict(output, arm, &cell, &attempt, false, true);
                }
            }
            _ => unreachable!("格名在上面已经核过"),
        }
    }
    0
}

/// V14：S3-C 的中断走没走到半帧与重发两条路。原读法是半帧、重发都 > 0；加修订里收严的一条：库进程至少收过两条连接
/// （至少一次重连真把帧送进了库；连上正在退出的转发进程的那一次到不了库）。
fn disconnect_paths_reached(run: &NetworkCellRun) -> bool {
    number_field(&run.library_network, "half_frames_dropped") > 0
        && number_field(&run.sender_summary, "resent_frames") > 0
        && number_field(&run.library_network, "connections") >= 2
}

fn emit_consistency_verdict(output: &mut ProductLines, arm: &str, cell: &LoopbackCell, attempt: &CellAttempt, escalated: bool, paths_reached: bool) {
    let run = &attempt.run;
    let expected_keys = cell.direct_blocks + cell.network_blocks;
    let expected_bytes = expected_keys * cell.states_per_block;
    output.line(format!(
        "name=verdict part=s3_consistency arm={arm} cell={} q3d_lost_confirmed={} q3e_silently_corrupted={} q3f_read_errors={} q3g_phantom_keys={} open_failed={} verification_missing={} a_count_matches={} half_frames_dropped={} resent_frames={} library_connections={} disconnect_paths_reached={} escalated_to_200={escalated} sender_complete={}",
        cell.label,
        run.verification.lost_confirmed.len(),
        run.verification.silently_corrupted.len(),
        run.read_errors(),
        run.verification.phantom_keys.len(),
        run.verification.open_failed,
        run.verification.verification_missing,
        run.verification.enumerated_key_count == expected_keys && run.verification.enumerated_value_bytes == expected_bytes,
        number_field(&run.library_network, "half_frames_dropped"),
        number_field(&run.sender_summary, "resent_frames"),
        number_field(&run.library_network, "connections"),
        if cell.interruption_count == 0 { "not_applicable".to_string() } else { paths_reached.to_string() },
        field_value(&run.sender_summary, "ending") == Some("complete"),
    ));
}

/// 第八节末段的判别力自证：拿两个 Q3a（状态/秒），门槛设成中点时一翻一不翻，设在两者之外时两点判得相同。
pub(super) fn run_discrimination(output: &mut ProductLines, arguments: &[String]) -> i32 {
    let values: Vec<f64> = arguments.iter().filter_map(|text| text.parse().ok()).collect();
    let [first, second] = values[..] else {
        eprintln!("  ✗ discrimination 要两个 Q3a（状态/秒）");
        return 2;
    };
    let midpoint = (first + second) / 2.0;
    let below_both = first.min(second) / 2.0;
    let above_both = first.max(second) * 2.0;
    let midpoint_separates = network_throughput_flips(first, midpoint) != network_throughput_flips(second, midpoint);
    let outside_agree = network_throughput_flips(first, below_both) == network_throughput_flips(second, below_both)
        && network_throughput_flips(first, above_both) == network_throughput_flips(second, above_both);
    output.line(format!(
        "name=verdict part=discrimination first={first} second={second} midpoint={midpoint} midpoint_separates={midpoint_separates} outside_thresholds_agree={outside_agree}"
    ));
    i32::from(!(midpoint_separates && outside_agree))
}

#[cfg(test)]
#[path = "cell_reports_and_questions_tests.rs"]
mod tests;
