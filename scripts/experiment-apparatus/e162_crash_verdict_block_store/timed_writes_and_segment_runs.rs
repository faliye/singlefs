//! 计时写入、重跑与小库对照，以及崩溃、吞吐、几何与分辨各问的运行。

use e7_index_bench::truncate_f64_to_u64;
use super::*;

// ============================== 计时的写入趟（校准、PC-rate、S2、几何取样点） ==============================

#[derive(Debug, Clone)]
pub(super) struct TimedWriteSettings {
    pub(super) candidate: Candidate,
    pub(super) write_path: WritePath,
    pub(super) library: PathBuf,
    pub(super) seed: u64,
    pub(super) pattern: ValuePattern,
    pub(super) states_per_block: u64,
    order: WriteOrder,
    batch: usize,
    pub(super) block_count: u64,
    pub(super) sleep_microseconds: u64,
    pub(super) raw_probe: bool,
    measure_occupancy: bool,
    quiet_wait_maximum_seconds: u64,
    allow_interference_reruns: bool,
}

#[derive(Debug, Default)]
pub(super) struct TimedWriteOutcome {
    exit_code: i32,
    raw_median_microseconds: Option<u64>,
    pub(super) open_microseconds: u64,
    pub(super) timing_line: String,
    backlog_lines: Vec<String>,
    window_lines: Vec<String>,
    /// （走之前已写块数，走完已写块数，占用字节）。
    occupancy_samples: Vec<(u64, u64, u64)>,
    stopped_for_occupancy: bool,
    error_line: String,
    interference_at_start: Vec<String>,
    interference_at_end: Vec<String>,
    load_before: String,
    load_after: String,
    nvme_sectors_before: u64,
    nvme_sectors_after: u64,
    quiet_wait_seconds: u64,
}

impl TimedWriteSettings {
    pub(super) fn writer_arguments(&self) -> Vec<String> {
        vec![
            "writer".to_string(),
            format!("candidate={}", self.candidate.label()),
            format!("write_path={}", self.write_path.label()),
            format!("library={}", self.library.display()),
            format!("seed={}", self.seed),
            format!("pattern={}", self.pattern.label()),
            format!("states={}", self.states_per_block),
            format!("order={}", match self.order {
                WriteOrder::Interleaved => "interleaved",
                WriteOrder::RandomPermutation => "random",
            }),
            format!("batch={}", self.batch),
            "first=0".to_string(),
            format!("end={}", self.block_count),
            format!("sleep={}", self.sleep_microseconds),
            "timed=1".to_string(),
            format!("probe={}", u8::from(self.raw_probe)),
            "progress=every-window".to_string(),
        ]
    }
}

fn remove_library(library: &Path) {
    if library.exists() {
        fs::remove_dir_all(library).expect("删得了库目录");
    }
}

/// V15：开跑前等干扰进程散去（每 30 秒看一次，最多等 maximum_wait_seconds 秒），返回等了几秒。
/// 可行性档传 0（`RunTier::quiet_wait_maximum_seconds`）：不等，直接往下走。
fn wait_for_quiet_machine(maximum_wait_seconds: u64) -> u64 {
    if cfg!(test) {
        // 单测跑在 cargo test 底下，cargo 自己就算干扰进程；单测不判 V15。
        return 0;
    }
    let started = Instant::now();
    while !interfering_processes().is_empty() && started.elapsed().as_secs() < maximum_wait_seconds {
        std::thread::sleep(Duration::from_secs(INTERFERENCE_POLL_SECONDS));
    }
    started.elapsed().as_secs()
}

pub(super) fn run_timed_write(settings: &TimedWriteSettings) -> TimedWriteOutcome {
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::sync::Arc;
    let mut outcome = TimedWriteOutcome {
        quiet_wait_seconds: wait_for_quiet_machine(settings.quiet_wait_maximum_seconds),
        interference_at_start: interfering_processes(),
        load_before: load_average_text(),
        nvme_sectors_before: nvme_written_sectors(),
        ..TimedWriteOutcome::default()
    };
    let written = Arc::new(AtomicU64::new(0));
    let stop = Arc::new(AtomicBool::new(false));
    let (request_sender, request_receiver) = mpsc::channel::<()>();
    let measurer = {
        let written = Arc::clone(&written);
        let stop = Arc::clone(&stop);
        let library = settings.library.clone();
        std::thread::spawn(move || {
            let mut samples = Vec::new();
            for () in request_receiver.iter() {
                let before = written.load(Ordering::SeqCst);
                let occupancy = library_occupancy_bytes(&library);
                let after = written.load(Ordering::SeqCst);
                if occupancy > OCCUPANCY_STOP_BYTES {
                    stop.store(true, Ordering::SeqCst);
                }
                samples.push((before, after, occupancy));
            }
            samples
        })
    };
    let (mut child, receiver) = spawn_child(&settings.writer_arguments());
    loop {
        match receiver.recv_timeout(Duration::from_millis(500)) {
            Ok(line) => {
                if let Some(count) = line.strip_prefix("C ").and_then(|count| count.parse::<u64>().ok()) {
                    written.store(count, Ordering::SeqCst);
                    if settings.measure_occupancy {
                        request_sender.send(()).expect("占用测量线程还在");
                    }
                } else if let Some(open) = line.strip_prefix("O ") {
                    outcome.open_microseconds = open.parse().unwrap_or(0);
                } else if line.starts_with("P ") {
                    outcome.raw_median_microseconds = field_value(&line, "raw_median_microseconds").and_then(|value| value.parse().ok());
                } else if line.starts_with("T ") {
                    outcome.timing_line = line;
                } else if line.starts_with("Q ") {
                    outcome.backlog_lines.push(line);
                } else if line.starts_with("W ") {
                    outcome.window_lines.push(line);
                } else if line.starts_with("E ") {
                    outcome.error_line = line.replace(' ', "_");
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
        if stop.load(Ordering::SeqCst) && !outcome.stopped_for_occupancy {
            outcome.stopped_for_occupancy = true;
            kill_and_reap(&mut child);
        }
    }
    let status = child.wait().expect("收得了写入子进程");
    outcome.exit_code = status.code().unwrap_or(-1);
    drop(request_sender);
    outcome.occupancy_samples = measurer.join().expect("占用测量线程没有 panic");
    outcome.interference_at_end = interfering_processes();
    outcome.load_after = load_average_text();
    outcome.nvme_sectors_after = nvme_written_sectors();
    outcome
}

fn list_or_dash(items: &[String]) -> String {
    if items.is_empty() {
        "-".to_string()
    } else {
        items.join(",")
    }
}

/// V15：一趟开始或结束时有干扰进程就重跑（新库），最多重跑三次；还有就带 interfered=true 照报最后一次。
fn run_timed_write_with_interference_reruns(output: &mut ProductLines, cell: &str, settings: &TimedWriteSettings) -> (TimedWriteOutcome, bool) {
    let mut attempt = 0usize;
    loop {
        remove_library(&settings.library);
        let outcome = run_timed_write(settings);
        let interfered = !outcome.interference_at_start.is_empty() || !outcome.interference_at_end.is_empty();
        output.line(format!(
            "name=timed_attempt cell={cell} arm={} attempt={attempt} quiet_wait_seconds={} interference_at_start={} interference_at_end={} load_before={} load_after={} nvme0n1_written_sectors_before={} nvme0n1_written_sectors_after={} exit_code={}",
            settings.candidate.label(),
            outcome.quiet_wait_seconds,
            list_or_dash(&outcome.interference_at_start),
            list_or_dash(&outcome.interference_at_end),
            outcome.load_before,
            outcome.load_after,
            outcome.nvme_sectors_before,
            outcome.nvme_sectors_after,
            outcome.exit_code,
        ));
        if !interfered || attempt >= INTERFERENCE_MAXIMUM_RERUNS || !settings.allow_interference_reruns {
            return (outcome, interfered);
        }
        attempt += 1;
    }
}

pub(super) fn timing_field(outcome: &TimedWriteOutcome, key: &str) -> f64 {
    field_value(&outcome.timing_line, key).and_then(|value| value.parse().ok()).unwrap_or(0.0)
}

// ============================== 阳性对照 PC-S、PC-P、PC-O（登记 5.3，钉绝对值） ==============================

fn small_library_plan() -> VerificationPlan {
    VerificationPlan {
        seed: SEED_CRASH_BLOCKS,
        pattern: ValuePattern::Random,
        states_per_block: PRIMARY_STATES_PER_BLOCK,
        confirmed_end: SMALL_LIBRARY_BLOCK_COUNT,
        started_end: SMALL_LIBRARY_BLOCK_COUNT,
        checked_blocks: CheckedBlocks::Range { start: 0, end: SMALL_LIBRARY_BLOCK_COUNT + NOT_STARTED_BLOCKS_READ_AFTER_STARTED },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SmallLibraryDamage {
    /// PC-S：第 7 块写进去之前第 40 000 字节异或 0x5A。
    SilentCorruption,
    /// PC-P：另写一个计划外的键。
    PhantomKey,
    /// PC-O：正常关库之后毁掉打开要用的那一处。
    OpenBreaking,
}

/// 新库写第 0–63 块（seed_S1），按 damage 动一处，正常关库，再照 5.1 逐块核。
pub(super) fn positive_control_small_library(candidate: Candidate, library: &Path, damage: SmallLibraryDamage) -> VerificationReport {
    remove_library(library);
    let fingerprint = input_fingerprint();
    {
        let mut store = open_store_for_writing(candidate, WritePath::Registered, library).expect("阳性对照的新库打得开");
        for block_index in 0..SMALL_LIBRARY_BLOCK_COUNT {
            let mut value = block_value(ValuePattern::Random, SEED_CRASH_BLOCKS, block_index, PRIMARY_STATES_PER_BLOCK);
            if damage == SmallLibraryDamage::SilentCorruption && block_index == SILENT_CORRUPTION_BLOCK {
                value[SILENT_CORRUPTION_BYTE_OFFSET] ^= SILENT_CORRUPTION_MASK;
            }
            let key = interleaved_block_key(block_index, PRIMARY_STATES_PER_BLOCK, &fingerprint);
            store.put_blocks(&[(key, value)]).expect("阳性对照写得进");
        }
        if damage == SmallLibraryDamage::PhantomKey {
            let phantom_key = compose_block_key(PHANTOM_KEY_NODE, 0, 0, &fingerprint);
            let value = block_value(ValuePattern::Random, SEED_CRASH_BLOCKS, 0, PRIMARY_STATES_PER_BLOCK);
            store.put_blocks(&[(phantom_key, value)]).expect("阳性对照写得进计划外的键");
        }
    }
    if damage == SmallLibraryDamage::OpenBreaking {
        break_library_opening(candidate, library);
    }
    let report = verify_library(candidate, library, &small_library_plan());
    remove_library(library);
    report
}

/// R1、R0 把库文件前 4096 字节写成 0xFF；K 把 CURRENT 换成 `MANIFEST-999999\n`；F 把 FORMAT 的 16 字节写成 0xFF。
fn break_library_opening(candidate: Candidate, library: &Path) {
    let (path, bytes): (PathBuf, Vec<u8>) = match candidate {
        Candidate::RedbQuickRepair | Candidate::RedbDefault => (library.join(REDB_FILE_NAME), vec![0xFF; 4096]),
        Candidate::RocksDatabase => (library.join("CURRENT"), b"MANIFEST-999999\n".to_vec()),
        Candidate::HardenedFile => (library.join(FORMAT_FILE_NAME), vec![0xFF; FORMAT_FILE_BYTES]),
    };
    let truncate = candidate == Candidate::RocksDatabase;
    let mut handle = OpenOptions::new().write(true).truncate(truncate).open(&path).expect("打得开要毁的文件");
    handle.write_all(&bytes).expect("写得进要毁的文件");
    handle.sync_all().expect("fsync 得了要毁的文件");
}

fn report_counts(report: &VerificationReport) -> String {
    format!(
        "open_failed={} opened_after_repair={} q1a={} q1b={} q1b_blocks={} q1c={} q1d={} q1e={} enumerated_keys={}",
        report.open_failed,
        match report.opened_after_repair {
            None => "not_applicable",
            Some(true) => "yes",
            Some(false) => "no",
        },
        report.lost_confirmed.len(),
        report.silently_corrupted.len(),
        join_numbers(&report.silently_corrupted),
        report.confirmed_read_errors.len(),
        report.unconfirmed_read_errors.len(),
        report.phantom_keys.len(),
        report.enumerated_key_count,
    )
}

pub(super) fn all_zero_except(report: &VerificationReport, allowed: &str) -> bool {
    (allowed == "q1a" || report.lost_confirmed.is_empty())
        && (allowed == "q1b" || report.silently_corrupted.is_empty())
        && report.confirmed_read_errors.is_empty()
        && report.unconfirmed_read_errors.is_empty()
        && (allowed == "q1e" || report.phantom_keys.is_empty())
        && (allowed == "q1f" || !report.open_failed)
}

// ============================== 第一段：S1（每条臂一份产物） ==============================

fn parse_arm(remaining: &[String]) -> Option<Candidate> {
    let candidate = remaining.first().and_then(|label| Candidate::parse(label));
    if candidate.is_none() {
        eprintln!("  ✗ 少了臂或臂写错：{remaining:?}");
        eprintln!("  → 怎么办：臂取 R1 R0 K F 之一");
    }
    candidate
}

/// V9 与 H4：库所在的文件系统不许是 tmpfs、ramfs、overlay；可用 < 40 GiB 不开跑。
fn environment_ready(output: &mut ProductLines, scratch: &ScratchDirectory) -> (bool, u64) {
    let (filesystem_type, available) = filesystem_type_and_available_bytes(&scratch.path);
    let filesystem_voids_everything = matches!(filesystem_type.as_str(), "tmpfs" | "ramfs" | "overlayfs" | "overlay");
    let too_little_space_to_start = available < MINIMUM_AVAILABLE_BYTES;
    output.line(format!(
        "name=environment filesystem_type={filesystem_type} available_bytes={available} filesystem_voids_everything={filesystem_voids_everything} too_little_space_to_start={too_little_space_to_start} logical_processors={} redb=4.3.0 rocksdb=0.25.0",
        std::thread::available_parallelism().map(usize::from).unwrap_or(0),
    ));
    (!filesystem_voids_everything && !too_little_space_to_start, available)
}

pub(super) fn calibration_settings(candidate: Candidate, library: PathBuf, seed: u64, tier: RunTier) -> TimedWriteSettings {
    TimedWriteSettings {
        candidate,
        write_path: WritePath::Registered,
        library,
        seed,
        pattern: ValuePattern::Random,
        states_per_block: PRIMARY_STATES_PER_BLOCK,
        order: WriteOrder::Interleaved,
        batch: 1,
        block_count: CALIBRATION_BLOCK_COUNT,
        sleep_microseconds: 0,
        raw_probe: true,
        measure_occupancy: false,
        quiet_wait_maximum_seconds: tier.quiet_wait_maximum_seconds(),
        allow_interference_reruns: tier.allow_interference_reruns(),
    }
}

fn parse_tier(remaining: &[String]) -> Option<RunTier> {
    let tier = RunTier::from_argument(remaining.get(1));
    if tier.is_none() {
        eprintln!("  ✗ 臂后面那一个参数认不出：{remaining:?}");
        eprintln!("  → 怎么办：不写就是登记的次数；写 feasibility 是可行性档（S1 杀 20 次、S2 写 10⁴ 块）");
    }
    tier
}

pub(super) fn run_segment_one_crash_question(output: &mut ProductLines, remaining: &[String]) -> i32 {
    let Some(candidate) = parse_arm(remaining) else { return 2 };
    let Some(tier) = parse_tier(remaining) else { return 2 };
    let main_cycles = match tier {
        RunTier::Registered => CRASH_MAIN_KILL_CYCLES,
        RunTier::Feasibility => FEASIBILITY_KILL_CYCLES,
    };
    let scratch = ScratchDirectory::create();
    output.line(format!(
        "name=config part=s1 arm={} tier={} kill_cycles={main_cycles} positive_control_cycles={POSITIVE_CONTROL_KILL_CYCLES} states_per_block={PRIMARY_STATES_PER_BLOCK} seed_s1={SEED_CRASH_BLOCKS:#x} seed_kill={SEED_KILL:#x}",
        candidate.label(),
        tier.label(),
    ));
    let (ready, _available) = environment_ready(output, &scratch);
    if !ready {
        output.line(format!("name=verdict part=s1 arm={} run=not_run reason=v9_or_h4", candidate.label()));
        return 3;
    }
    // S1 校准：A-raw 与 1 000 块，给 m。
    let calibration_library = scratch.path.join("calibration");
    let (calibration, calibration_interfered) =
        run_timed_write_with_interference_reruns(output, "s1-calibration", &calibration_settings(candidate, calibration_library.clone(), SEED_CRASH_BLOCKS, tier));
    remove_library(&calibration_library);
    let commit_median = timing_field(&calibration, "commit_median_microseconds");
    let raw_median = calibration.raw_median_microseconds.unwrap_or(0);
    let commit_waits_for_device = commit_median * 2.0 >= raw_median as f64;
    output.line(format!(
        "name=s1_calibration arm={} raw_median_microseconds={raw_median} commit_median_microseconds={commit_median:.0} commit_percentile_99_microseconds={:.0} blocks_per_second={:.3} interfered={calibration_interfered} h2_commit_median_at_least_half_raw={commit_waits_for_device} exit_code={} writer_error={}",
        candidate.label(),
        timing_field(&calibration, "commit_percentile_99_microseconds"),
        timing_field(&calibration, "blocks_per_second"),
        calibration.exit_code,
        if calibration.error_line.is_empty() { "-" } else { &calibration.error_line },
    ));
    if calibration.exit_code != 0 || !commit_waits_for_device {
        output.line(format!(
            "name=verdict part=s1 arm={} run=stopped reason={} commit_waits_for_device={commit_waits_for_device}",
            candidate.label(),
            if calibration.exit_code != 0 { "calibration_writer_failed" } else { "h2_stop" }
        ));
        return 4;
    }
    let commit_median_microseconds = truncate_f64_to_u64(commit_median.round());
    // PC-S、PC-P、PC-O：钉绝对值。
    let silent = positive_control_small_library(candidate, &scratch.path.join("pc-s"), SmallLibraryDamage::SilentCorruption);
    let silent_ok = silent.silently_corrupted == vec![SILENT_CORRUPTION_BLOCK] && all_zero_except(&silent, "q1b");
    output.line(format!("name=positive_control control=PC-S arm={} {} ok={silent_ok}", candidate.label(), report_counts(&silent)));
    let phantom = positive_control_small_library(candidate, &scratch.path.join("pc-p"), SmallLibraryDamage::PhantomKey);
    let phantom_ok = phantom.phantom_keys.len() == 1 && all_zero_except(&phantom, "q1e");
    output.line(format!("name=positive_control control=PC-P arm={} {} ok={phantom_ok}", candidate.label(), report_counts(&phantom)));
    let open_breaking = positive_control_small_library(candidate, &scratch.path.join("pc-o"), SmallLibraryDamage::OpenBreaking);
    let open_ok = open_breaking.open_failed && all_zero_except(&open_breaking, "q1f");
    output.line(format!(
        "name=positive_control control=PC-O arm={} {} ok={open_ok} open_error={}",
        candidate.label(),
        report_counts(&open_breaking),
        open_breaking.open_error.replace(char::is_whitespace, "_"),
    ));
    let full_plan = kill_plan(SEED_KILL, CRASH_MAIN_KILL_CYCLES);
    // PC-L：坏形态，杀点计划前 20 行。
    let lost_write_path = candidate.lost_block_control_write_path();
    let lost_library = scratch.path.join("pc-l");
    remove_library(&lost_library);
    let mut lost_campaign = KillCampaign::new("PC-L", candidate, lost_write_path, lost_library.clone(), SEED_CRASH_BLOCKS, commit_median_microseconds);
    run_kill_campaign(output, &mut lost_campaign, &full_plan[..POSITIVE_CONTROL_KILL_CYCLES]);
    remove_library(&lost_library);
    let lost_ok = lost_campaign.write_kills_with_loss >= u64::try_from(POSITIVE_CONTROL_LOST_MINIMUM_HITS).expect("10");
    output.line(format!(
        "name=positive_control control=PC-L arm={} write_path={} write_kills={} write_kills_with_q1a={} write_kills_with_new_q1a={} ok={lost_ok}",
        candidate.label(),
        lost_write_path.label(),
        lost_campaign.write_phase_kills,
        lost_campaign.write_kills_with_loss,
        lost_campaign.write_kills_with_new_loss,
    ));
    // PC-T：只有 F（F-direct）；前 20 行一次都没看到读报错就把 200 行跑完。
    let torn_ok = match candidate {
        Candidate::HardenedFile => {
            let torn_library = scratch.path.join("pc-t");
            remove_library(&torn_library);
            let mut torn_campaign =
                KillCampaign::new("PC-T", candidate, WritePath::FileDirectInPlace, torn_library.clone(), SEED_CRASH_BLOCKS, commit_median_microseconds);
            run_kill_campaign(output, &mut torn_campaign, &full_plan[..POSITIVE_CONTROL_KILL_CYCLES]);
            let mut extended = false;
            if torn_campaign.write_kills_with_read_error == 0 && tier == RunTier::Registered {
                extended = true;
                run_kill_campaign(output, &mut torn_campaign, &full_plan[POSITIVE_CONTROL_KILL_CYCLES..]);
            }
            remove_library(&torn_library);
            let ok = torn_campaign.write_kills_with_read_error >= 1 && torn_campaign.cumulative_corrupted.is_empty();
            output.line(format!(
                "name=positive_control control=PC-T arm=F write_kills={} write_kills_with_read_error={} q1b={} extended_to_200={extended} ok={ok}",
                torn_campaign.write_phase_kills,
                torn_campaign.write_kills_with_read_error,
                torn_campaign.cumulative_corrupted.len(),
            ));
            if ok {
                "true"
            } else {
                "false"
            }
        }
        Candidate::RedbQuickRepair | Candidate::RedbDefault | Candidate::RocksDatabase => "not_applicable",
    };
    // S1 主格：200 次杀。
    let main_library = scratch.path.join("s1-main");
    remove_library(&main_library);
    let mut main_campaign = KillCampaign::new("S1-main", candidate, WritePath::Registered, main_library.clone(), SEED_CRASH_BLOCKS, commit_median_microseconds);
    let main_plan = &full_plan[..main_cycles];
    run_kill_campaign(output, &mut main_campaign, main_plan);
    remove_library(&main_library);
    // V6 的门槛 100 是对 180 次写中途的杀写的；可行性档只报数，不判。
    let kills_inside_commit_ok = match tier {
        RunTier::Registered => {
            if main_campaign.kills_inside_commit >= u64::try_from(KILL_INSIDE_COMMIT_MINIMUM).expect("100") {
                "true"
            } else {
                "false"
            }
        }
        RunTier::Feasibility => "not_applicable",
    };
    // A-S1：确认数合计 ≥ 用到的那几行 n_i 之和（200 行时是 6 178）。
    let planned_confirmations: u64 = main_plan.iter().map(|row| confirmations_of(row.phase)).sum();
    let confirmations_ok =
        main_campaign.confirmations_before_write_kills >= planned_confirmations && main_campaign.assertion_failures.is_empty();
    let flipped_quantities: Vec<&str> = [
        ("q1a", main_campaign.cumulative_lost.len()),
        ("q1b", main_campaign.cumulative_corrupted.len()),
        ("q1c", main_campaign.cumulative_confirmed_read_errors.len()),
        ("q1d", main_campaign.cumulative_unconfirmed_read_errors.len()),
        ("q1e", main_campaign.cumulative_phantoms.len()),
        ("q1f", usize::try_from(main_campaign.open_failures).expect("装得进 usize")),
    ]
    .into_iter()
    .filter(|(_, count)| *count > 0)
    .map(|(quantity, _)| quantity)
    .collect();
    output.line(format!(
        "name=verdict part=s1 arm={} tier={} judgement={} kill_cycles={main_cycles} kills_inside_commit={} pc_l_ok={lost_ok} pc_s_ok={silent_ok} pc_p_ok={phantom_ok} pc_o_ok={open_ok} pc_t_ok={torn_ok} commit_waits_for_device={commit_waits_for_device} calibration_interfered={calibration_interfered} v6_kills_inside_commit_ok={kills_inside_commit_ok} planned_confirmations={planned_confirmations} a_s1_ok={confirmations_ok} q1a={} q1b={} q1c={} q1d={} q1e={} q1f={} repaired_openings={} s1_positive_quantities={}",
        candidate.label(),
        tier.label(),
        tier.judgement(),
        main_campaign.kills_inside_commit,
        main_campaign.cumulative_lost.len(),
        main_campaign.cumulative_corrupted.len(),
        main_campaign.cumulative_confirmed_read_errors.len(),
        main_campaign.cumulative_unconfirmed_read_errors.len(),
        main_campaign.cumulative_phantoms.len(),
        main_campaign.open_failures,
        main_campaign.repaired_openings,
        if flipped_quantities.is_empty() { "none".to_string() } else { flipped_quantities.join(",") },
    ));
    0
}

// ============================== 第一段：S2 与几何敏感性 ==============================

/// P_B：`ssh -o BatchMode=yes <第二台的地址> nproc`。第二台的地址取多机配置的主机数组（scripts/crash/crash-point-replay-shard-configuration-check.sh 文件头「主机数组」）：
/// 照那份判法 `--emit-hosts`（不连任何一台）打的 `PEER_HOST_ADDRESS=` 那一行，与 scripts/hosts/multi-host-run.sh 同一个取法。
/// 交回（产物里写的第二台，地址是私有的、只写 `peer`；nproc 的回答）。
fn peer_logical_processors() -> (String, String) {
    let emitted = Command::new("bash")
        .arg(repository_root().join("scripts/crash/crash-point-replay-shard-configuration-check.sh"))
        .arg("--emit-hosts")
        .arg(repository_root())
        .stdin(Stdio::null())
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).to_string())
        .unwrap_or_default();
    let host = emitted
        .lines()
        .find_map(|line| line.trim().strip_prefix("PEER_HOST_ADDRESS="))
        .map(|value| value.trim_matches('\'').to_string())
        .unwrap_or_default();
    if host.is_empty() {
        return ("-".to_string(), "no_peer_host_configured".to_string());
    }
    let answer = Command::new("ssh")
        .args(["-o", "BatchMode=yes", "-o", "ConnectTimeout=10", &host, "nproc"])
        .stdin(Stdio::null())
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or_default();
    ("peer".to_string(), if answer.is_empty() { "no_answer".to_string() } else { answer })
}

/// 冷读之前：对库目录每个普通文件 posix_fadvise(DONTNEED)（python3 做，不给装置加依赖）。
fn drop_library_page_cache(library: &Path) -> bool {
    let script = "import os, sys\nfor root, _directories, files in os.walk(sys.argv[1]):\n    for name in files:\n        handle = os.open(os.path.join(root, name), os.O_RDONLY)\n        os.posix_fadvise(handle, 0, 0, os.POSIX_FADV_DONTNEED)\n        os.close(handle)\n";
    Command::new("python3").arg("-c").arg(script).arg(library).status().is_ok_and(|status| status.success())
}

#[allow(clippy::too_many_arguments, reason = "读子进程的参数一个个对着登记列，不收成结构体")]
fn run_read_child(candidate: Candidate, library: &Path, seed: u64, pattern: ValuePattern, states_per_block: u64, block_count: u64, read_count: usize, sleep_microseconds: u64, passes: u64) -> Vec<String> {
    let arguments = vec![
        "read".to_string(),
        format!("candidate={}", candidate.label()),
        format!("library={}", library.display()),
        format!("seed={seed}"),
        format!("pattern={}", pattern.label()),
        format!("states={states_per_block}"),
        format!("blocks={block_count}"),
        format!("count={read_count}"),
        format!("sleep={sleep_microseconds}"),
        format!("passes={passes}"),
    ];
    let (lines, _exit_code) = run_child_to_completion(&arguments);
    lines.into_iter().filter(|line| line.starts_with("R ") || line.starts_with("E ")).collect()
}

fn read_line_number(line: &str, key: &str) -> u64 {
    field_value(line, key).and_then(|value| value.parse().ok()).unwrap_or(0)
}

/// Q2a 的判定：跟不上（翻）就是 true。判别力自证用它（登记 8.2）。
pub(super) fn write_rate_flips(measured_states_per_second: f64, required_states_per_second: f64) -> bool {
    measured_states_per_second < required_states_per_second
}

struct ThroughputContext {
    available_bytes: u64,
    total_logical_processors: u64,
}

/// 一趟 10⁵ 块：计时写入（带 A-raw、占用测量、V15 重跑）→ A-count → 冷热随机读。库留给调用方删。
fn run_throughput_cell(output: &mut ProductLines, cell: &str, settings: &TimedWriteSettings, context: &ThroughputContext) -> f64 {
    let arm = settings.candidate.label();
    let (outcome, interfered) = run_timed_write_with_interference_reruns(output, cell, settings);
    let states_per_second = timing_field(&outcome, "states_per_second");
    let commit_median = timing_field(&outcome, "commit_median_microseconds");
    let raw_median = outcome.raw_median_microseconds.unwrap_or(0);
    let commit_waits_for_device = commit_median * 2.0 >= raw_median as f64;
    let break_even_thread_seconds_per_state = if states_per_second > 0.0 { context.total_logical_processors as f64 / states_per_second } else { 0.0 };
    output.line(format!(
        "name=throughput cell={cell} arm={arm} blocks={} elapsed_microseconds={:.0} blocks_per_second={:.3} write_states_per_second={states_per_second:.1} commit_median_microseconds={commit_median:.0} commit_percentile_99_microseconds={:.0} commit_maximum_microseconds={:.0} raw_median_microseconds={raw_median} commit_waits_for_device={commit_waits_for_device} interfered={interfered} writer_exit={} writer_error={} stopped_h3={} open_microseconds={} break_even_thread_seconds_per_state_thread_seconds={break_even_thread_seconds_per_state:.6e} q2a_verdict=pending_e161",
        timing_field(&outcome, "blocks"),
        timing_field(&outcome, "elapsed_microseconds"),
        timing_field(&outcome, "blocks_per_second"),
        timing_field(&outcome, "commit_percentile_99_microseconds"),
        timing_field(&outcome, "commit_maximum_microseconds"),
        outcome.exit_code,
        if outcome.error_line.is_empty() { "-" } else { &outcome.error_line },
        outcome.stopped_for_occupancy,
        outcome.open_microseconds,
    ));
    let mut minimum_window = f64::MAX;
    for line in &outcome.window_lines {
        let window_rate: f64 = field_value(line, "states_per_second").and_then(|value| value.parse().ok()).unwrap_or(0.0);
        minimum_window = minimum_window.min(window_rate);
        output.line(format!(
            "name=window cell={cell} arm={arm} window={} end_microseconds={} states_per_second={window_rate:.1}",
            field_value(line, "window").unwrap_or("-"),
            field_value(line, "end_microseconds").unwrap_or("-"),
        ));
    }
    let last_window = outcome.window_lines.last().and_then(|line| field_value(line, "states_per_second")).unwrap_or("-");
    output.line(format!(
        "name=window_summary cell={cell} arm={arm} windows={} minimum_states_per_second={:.1} last_states_per_second={last_window} break_even_thread_seconds_per_state_minimum_window_thread_seconds={:.6e} q2b_verdict=pending_e161",
        outcome.window_lines.len(),
        if outcome.window_lines.is_empty() { 0.0 } else { minimum_window },
        if outcome.window_lines.is_empty() || minimum_window <= 0.0 { 0.0 } else { context.total_logical_processors as f64 / minimum_window },
    ));
    for line in &outcome.backlog_lines {
        output.line(format!("name=backlog cell={cell} arm={arm} {}", line.trim_start_matches("Q ")));
    }
    let value_bytes_per_block = settings.states_per_block as f64;
    let (mut peak_high, mut peak_low) = (0f64, 0f64);
    for &(written_before, written_after, occupancy) in &outcome.occupancy_samples {
        let ratio_high = if written_before > 0 { occupancy as f64 / (written_before as f64 * value_bytes_per_block) } else { 0.0 };
        let ratio_low = if written_after > 0 { occupancy as f64 / (written_after as f64 * value_bytes_per_block) } else { 0.0 };
        peak_high = peak_high.max(ratio_high);
        peak_low = peak_low.max(ratio_low);
        output.line(format!(
            "name=occupancy cell={cell} arm={arm} written_before={written_before} written_after={written_after} occupancy_bytes={occupancy} ratio_high={ratio_high:.4} ratio_low={ratio_low:.4}"
        ));
    }
    let after_close = library_occupancy_bytes(&settings.library);
    let after_close_ratio = after_close as f64 / (settings.block_count as f64 * value_bytes_per_block);
    output.line(format!(
        "name=occupancy_summary cell={cell} arm={arm} samples={} peak_ratio_high={peak_high:.4} peak_ratio_low={peak_low:.4} after_close_bytes={after_close} after_close_ratio={after_close_ratio:.4} milestone_bytes_at_peak_high={:.0} clause_bytes_at_peak_high={:.0} available_bytes={} q2e_flips_at_1_9e10={} q2e_flips_at_1_5e10={} q2e_flips_at_1_9e10_using_low={}",
        outcome.occupancy_samples.len(),
        peak_high * MILESTONE_TWO_STATES,
        peak_high * CLAUSE_TEXT_STATES,
        context.available_bytes,
        peak_high * MILESTONE_TWO_STATES > context.available_bytes as f64,
        peak_high * CLAUSE_TEXT_STATES > context.available_bytes as f64,
        peak_low * MILESTONE_TWO_STATES > context.available_bytes as f64,
    ));
    // A-count：恰好 N 个键，块值长度之和恰好 N × V。
    let (key_count, value_bytes) = match open_existing_store(settings.candidate, &settings.library)
        .and_then(|store| store.enumerate_keys_with_value_lengths())
    {
        Ok(entries) => (
            u64::try_from(entries.len()).expect("装得进 u64"),
            entries.iter().map(|(_, length)| length).sum::<u64>(),
        ),
        Err(_) => (0, 0),
    };
    let expected_value_bytes = settings.block_count * settings.states_per_block;
    output.line(format!(
        "name=a_count cell={cell} arm={arm} keys={key_count} value_bytes={value_bytes} expected_keys={} expected_value_bytes={expected_value_bytes} a_count_ok={}",
        settings.block_count,
        key_count == settings.block_count && value_bytes == expected_value_bytes,
    ));
    // Q2d：冷（fadvise 之后重开）一遍、同一个打开的库上热一遍。
    let cache_dropped = drop_library_page_cache(&settings.library);
    let read_interference_at_start = list_or_dash(&interfering_processes());
    let read_lines = run_read_child(settings.candidate, &settings.library, settings.seed, settings.pattern, settings.states_per_block, settings.block_count, RANDOM_READ_COUNT, 0, 2);
    let read_interference_at_end = list_or_dash(&interfering_processes());
    for line in &read_lines {
        let pass = field_value(line, "pass").unwrap_or("-");
        let temperature = if pass == "first" { "cold" } else { "warm" };
        let distinct = read_line_number(line, "distinct_blocks");
        let read_bytes = read_line_number(line, "read_bytes_delta");
        let percentile_99 = read_line_number(line, "percentile_99_microseconds");
        let cold_cache_confirmed = temperature == "warm" || read_bytes * 2 >= distinct * settings.states_per_block;
        output.line(format!(
            "name=random_read cell={cell} arm={arm} pass={temperature} cache_dropped={cache_dropped} reads={} distinct_blocks={distinct} median_microseconds={} percentile_99_microseconds={percentile_99} maximum_microseconds={} mismatches={} read_bytes_delta={read_bytes} v8_cold_cache_ok={cold_cache_confirmed} open_microseconds={} t_state_flip_below_thread_seconds={:.6e} q2d_verdict={} interference_at_start={read_interference_at_start} interference_at_end={read_interference_at_end}",
            read_line_number(line, "reads"),
            read_line_number(line, "median_microseconds"),
            read_line_number(line, "maximum_microseconds"),
            read_line_number(line, "mismatches"),
            read_line_number(line, "open_microseconds"),
            percentile_99 as f64 / 1e6 / settings.states_per_block as f64,
            if temperature == "cold" { "pending_e161" } else { "report_only" },
        ));
    }
    states_per_second
}

fn throughput_settings(candidate: Candidate, library: PathBuf, point: &str, block_count: u64, tier: RunTier) -> TimedWriteSettings {
    let mut settings = TimedWriteSettings {
        candidate,
        write_path: WritePath::Registered,
        library,
        seed: SEED_THROUGHPUT_BLOCKS,
        pattern: ValuePattern::Random,
        states_per_block: PRIMARY_STATES_PER_BLOCK,
        order: WriteOrder::Interleaved,
        batch: 1,
        block_count,
        sleep_microseconds: 0,
        raw_probe: true,
        measure_occupancy: true,
        quiet_wait_maximum_seconds: tier.quiet_wait_maximum_seconds(),
        allow_interference_reruns: tier.allow_interference_reruns(),
    };
    match point {
        "S2-main" => {}
        "G-bs10" => settings.states_per_block = SMALL_STATES_PER_BLOCK,
        "G-batch16" => settings.batch = BATCH_BLOCKS,
        "G-sparse" => settings.pattern = ValuePattern::Sparse,
        "G-random" => settings.order = WriteOrder::RandomPermutation,
        other => panic!("认不出的取样点 {other}"),
    }
    settings
}

fn throughput_context(output: &mut ProductLines, scratch: &ScratchDirectory, part: &str, candidate: Candidate) -> Option<ThroughputContext> {
    let (ready, available_bytes) = environment_ready(output, scratch);
    let local = u64::try_from(std::thread::available_parallelism().map(usize::from).unwrap_or(0)).expect("装得进 u64");
    let (peer_host, peer_answer) = peer_logical_processors();
    let peer = peer_answer.parse::<u64>().ok();
    output.line(format!(
        "name=processors part={part} arm={} p_a={local} p_b_command=ssh_-o_BatchMode=yes_-o_ConnectTimeout=10_{peer_host}_nproc p_b_answer={peer_answer} p_a_plus_p_b={} t_state=pending_e161",
        candidate.label(),
        peer.map_or("pending_p_b".to_string(), |peer| (local + peer).to_string()),
    ));
    if !ready {
        output.line(format!("name=verdict part={part} arm={} run=not_run reason=v9_or_h4", candidate.label()));
        return None;
    }
    Some(ThroughputContext { available_bytes, total_logical_processors: local + peer.unwrap_or(0) })
}

pub(super) fn run_segment_one_throughput_question(output: &mut ProductLines, remaining: &[String]) -> i32 {
    let Some(candidate) = parse_arm(remaining) else { return 2 };
    let Some(tier) = parse_tier(remaining) else { return 2 };
    let main_block_count = match tier {
        RunTier::Registered => THROUGHPUT_BLOCK_COUNT,
        RunTier::Feasibility => FEASIBILITY_THROUGHPUT_BLOCK_COUNT,
    };
    let arm = candidate.label();
    let scratch = ScratchDirectory::create();
    output.line(format!(
        "name=config part=s2 arm={arm} tier={} blocks={main_block_count} states_per_block={PRIMARY_STATES_PER_BLOCK} seed_s2={SEED_THROUGHPUT_BLOCKS:#x} seed_read={SEED_READ:#x} seed_large={SEED_LARGE:#x}",
        tier.label()
    ));
    let Some(context) = throughput_context(output, &scratch, "s2", candidate) else { return 3 };
    // PC-rate：1 000 块两遍，一遍在计时区间里、写入调用之前睡 5 ms。
    let plain_library = scratch.path.join("pc-rate-plain");
    let sleep_library = scratch.path.join("pc-rate-sleep");
    let mut plain_settings = calibration_settings(candidate, plain_library.clone(), SEED_THROUGHPUT_BLOCKS, tier);
    plain_settings.raw_probe = false;
    let mut sleep_settings = plain_settings.clone();
    sleep_settings.library = sleep_library.clone();
    sleep_settings.sleep_microseconds = POSITIVE_CONTROL_SLEEP_MICROSECONDS;
    let (plain, _plain_interfered) = run_timed_write_with_interference_reruns(output, "pc-rate-plain", &plain_settings);
    let (slept, _slept_interfered) = run_timed_write_with_interference_reruns(output, "pc-rate-sleep", &sleep_settings);
    remove_library(&sleep_library);
    let rate_difference = timing_field(&slept, "commit_median_microseconds") - timing_field(&plain, "commit_median_microseconds");
    let rate_ok = rate_difference >= POSITIVE_CONTROL_MINIMUM_DIFFERENCE_MICROSECONDS as f64;
    output.line(format!(
        "name=positive_control control=PC-rate arm={arm} plain_commit_median_microseconds={:.0} sleep_commit_median_microseconds={:.0} difference_microseconds={rate_difference:.0} ok={rate_ok}",
        timing_field(&plain, "commit_median_microseconds"),
        timing_field(&slept, "commit_median_microseconds"),
    ));
    // PC-read：在 PC-rate 那个库上各读 1 000 次，一遍在计时区间里读调用之前睡 5 ms（修订五：照 V15 等、照 V15 重跑）。
    let mut attempt = 0usize;
    let (read_plain, read_slept) = loop {
        let quiet_wait_seconds = wait_for_quiet_machine(tier.quiet_wait_maximum_seconds());
        let interference_at_start = interfering_processes();
        let plain_lines = run_read_child(candidate, &plain_library, SEED_THROUGHPUT_BLOCKS, ValuePattern::Random, PRIMARY_STATES_PER_BLOCK, CALIBRATION_BLOCK_COUNT, POSITIVE_CONTROL_READ_COUNT, 0, 1);
        let slept_lines = run_read_child(candidate, &plain_library, SEED_THROUGHPUT_BLOCKS, ValuePattern::Random, PRIMARY_STATES_PER_BLOCK, CALIBRATION_BLOCK_COUNT, POSITIVE_CONTROL_READ_COUNT, POSITIVE_CONTROL_SLEEP_MICROSECONDS, 1);
        let interference_at_end = interfering_processes();
        output.line(format!(
            "name=timed_attempt cell=pc-read arm={arm} attempt={attempt} quiet_wait_seconds={quiet_wait_seconds} interference_at_start={} interference_at_end={}",
            list_or_dash(&interference_at_start),
            list_or_dash(&interference_at_end),
        ));
        let interfered = !interference_at_start.is_empty() || !interference_at_end.is_empty();
        if !interfered || attempt >= INTERFERENCE_MAXIMUM_RERUNS || !tier.allow_interference_reruns() {
            break (plain_lines, slept_lines);
        }
        attempt += 1;
    };
    let read_median = |lines: &[String]| lines.first().map_or(0, |line| read_line_number(line, "median_microseconds"));
    let read_difference = read_median(&read_slept).saturating_sub(read_median(&read_plain));
    let read_ok = read_difference >= POSITIVE_CONTROL_MINIMUM_DIFFERENCE_MICROSECONDS;
    output.line(format!(
        "name=positive_control control=PC-read arm={arm} plain_median_microseconds={} sleep_median_microseconds={} difference_microseconds={read_difference} ok={read_ok}",
        read_median(&read_plain),
        read_median(&read_slept),
    ));
    // PC-space：库关好之后量一次；往库目录里写一个 2³⁰ 字节的随机文件（sync_all）再量。
    let before = library_occupancy_bytes(&plain_library);
    let space_file = plain_library.join("e162-positive-control-space.bin");
    {
        let mut handle = File::create(&space_file).expect("建得了 PC-space 文件");
        let mut generator = SplitMix64::new(SEED_READ);
        let chunk_words = 1 << 17;
        for _chunk in 0..POSITIVE_CONTROL_SPACE_FILE_BYTES / (chunk_words * 8) {
            let chunk: Vec<u8> = (0..chunk_words).flat_map(|_| generator.next_output().to_le_bytes()).collect();
            handle.write_all(&chunk).expect("写得进 PC-space 文件");
        }
        handle.sync_all().expect("fsync 得了 PC-space 文件");
    }
    let after = library_occupancy_bytes(&plain_library);
    let space_difference = after.saturating_sub(before);
    let space_ok = (POSITIVE_CONTROL_SPACE_FILE_BYTES..=POSITIVE_CONTROL_SPACE_FILE_BYTES + POSITIVE_CONTROL_SPACE_SLACK_BYTES).contains(&space_difference);
    output.line(format!("name=positive_control control=PC-space arm={arm} before_bytes={before} after_bytes={after} difference_bytes={space_difference} ok={space_ok}"));
    remove_library(&plain_library);
    // S2 主格，然后在同一个库上接着跑 S1-large。
    let main_library = scratch.path.join("s2-main");
    let main_settings = throughput_settings(candidate, main_library.clone(), "S2-main", main_block_count, tier);
    let main_rate = run_throughput_cell(output, "S2-main", &main_settings, &context);
    let large_summary = match tier {
        RunTier::Feasibility => "s1_large=not_run_feasibility_tier".to_string(),
        RunTier::Registered => {
            let large_commit_median = calibration_commit_median_in_this_process(&main_library, candidate);
            let mut large_campaign =
                KillCampaign::new("S1-large", candidate, WritePath::Registered, main_library.clone(), SEED_THROUGHPUT_BLOCKS, large_commit_median);
            large_campaign.confirmed_end = THROUGHPUT_BLOCK_COUNT;
            large_campaign.started_end = THROUGHPUT_BLOCK_COUNT;
            large_campaign.large_library_sampling = Some(THROUGHPUT_BLOCK_COUNT);
            run_kill_campaign(output, &mut large_campaign, &kill_plan(SEED_LARGE, CRASH_LARGE_KILL_CYCLES));
            format!(
                "s1_large_q1a={} s1_large_q1b={} s1_large_q1c={} s1_large_q1d={} s1_large_q1e={} s1_large_q1f={} s1_large_assertion_failures={}",
                large_campaign.cumulative_lost.len(),
                large_campaign.cumulative_corrupted.len(),
                large_campaign.cumulative_confirmed_read_errors.len(),
                large_campaign.cumulative_unconfirmed_read_errors.len(),
                large_campaign.cumulative_phantoms.len(),
                large_campaign.open_failures,
                large_campaign.assertion_failures.len(),
            )
        }
    };
    remove_library(&main_library);
    output.line(format!(
        "name=verdict part=s2 arm={arm} tier={} judgement={} blocks={main_block_count} pc_rate_ok={rate_ok} pc_read_ok={read_ok} pc_space_ok={space_ok} s2_main_write_states_per_second={main_rate:.1} q2a_verdict=pending_e161 {large_summary}",
        tier.label(),
        tier.judgement(),
    ));
    0
}

/// S1-large 的 m：照 5.5 的校准趟（新库、1 000 块、主取样点形态、不杀）在这个进程里再量一遍（S1 那一份在另一个进程里）。
fn calibration_commit_median_in_this_process(main_library: &Path, candidate: Candidate) -> u64 {
    let probe_library = main_library.with_file_name("s1-large-calibration");
    let settings = TimedWriteSettings { raw_probe: false, ..calibration_settings(candidate, probe_library.clone(), SEED_CRASH_BLOCKS, RunTier::Registered) };
    remove_library(&probe_library);
    let outcome = run_timed_write(&settings);
    remove_library(&probe_library);
    truncate_f64_to_u64(timing_field(&outcome, "commit_median_microseconds").round())
}

pub(super) fn run_segment_one_geometry(output: &mut ProductLines, remaining: &[String]) -> i32 {
    let Some(candidate) = parse_arm(remaining) else { return 2 };
    let Some(point) = remaining.get(1).map(String::as_str).filter(|point| ["G-bs10", "G-batch16", "G-sparse", "G-random"].contains(point)) else {
        eprintln!("  ✗ 少了取样点或取样点写错：{remaining:?}");
        eprintln!("  → 怎么办：取样点取 G-bs10 G-batch16 G-sparse G-random 之一");
        return 2;
    };
    let scratch = ScratchDirectory::create();
    output.line(format!("name=config part=geometry point={point} arm={} blocks={THROUGHPUT_BLOCK_COUNT} seed_s2={SEED_THROUGHPUT_BLOCKS:#x}", candidate.label()));
    let Some(context) = throughput_context(output, &scratch, "geometry", candidate) else { return 3 };
    let library = scratch.path.join(point);
    let settings = throughput_settings(candidate, library.clone(), point, THROUGHPUT_BLOCK_COUNT, RunTier::Registered);
    let rate = run_throughput_cell(output, point, &settings, &context);
    remove_library(&library);
    output.line(format!("name=verdict part=geometry point={point} arm={} write_states_per_second={rate:.1} q2a_verdict=pending_e161", candidate.label()));
    0
}

/// 登记 8.2 的判别力自证：门槛设在两个 Q2a 值的中点，两点必须一翻一不翻；设在两点之外，两点判得相同。
pub(super) fn run_discrimination(output: &mut ProductLines, remaining: &[String]) -> i32 {
    let values: Vec<f64> = remaining.iter().filter_map(|value| value.parse().ok()).collect();
    let [first, second] = values[..] else {
        eprintln!("  ✗ 要两个 Q2a 值（状态/秒）：{remaining:?}");
        eprintln!("  → 怎么办：从产物里取同一条臂主取样点与 G-bs10 的 write_states_per_second 两个数传进来");
        return 2;
    };
    let midpoint = (first + second) / 2.0;
    let below = first.min(second) / 2.0;
    let above = first.max(second) * 2.0;
    let differs_at_midpoint = write_rate_flips(first, midpoint) != write_rate_flips(second, midpoint);
    let same_below = write_rate_flips(first, below) == write_rate_flips(second, below);
    let same_above = write_rate_flips(first, above) == write_rate_flips(second, above);
    output.line(format!(
        "name=discrimination first={first} second={second} midpoint={midpoint} differs_at_midpoint={differs_at_midpoint} same_below_both={same_below} same_above_both={same_above}"
    ));
    if differs_at_midpoint && same_below && same_above {
        0
    } else {
        1
    }
}
