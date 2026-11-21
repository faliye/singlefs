//! 逐点结果的汇总、重放保真度、批量核对、写启动的判定与判定行。

use super::*;

/// 验盘开机里每个「断电点 × 取法」的结果（从来宾的 s4_point 行读回来）。
#[derive(Debug, Clone, PartialEq, Eq)]
struct PointResult {
    item: VerifyItem,
    mount_ok: bool,
    check_label: String,
    pub(super) lost_confirmed: u64,
    pub(super) silently_corrupted: u64,
    pub(super) confirmed_read_errors: u64,
    pub(super) unconfirmed_read_errors: u64,
    phantom: u64,
    pub(super) open_failed: bool,
    pub(super) open_microseconds: u64,
    silently_corrupted_blocks: String,
}

impl PointResult {
    fn from_line(item: &VerifyItem, line: &str) -> PointResult {
        let number = |key: &str| field_number(line, key).unwrap_or(0);
        PointResult {
            item: item.clone(),
            mount_ok: field_value(line, "mount") == Some("ok"),
            check_label: field_value(line, "check").unwrap_or("missing").to_string(),
            lost_confirmed: number("q4a"),
            silently_corrupted: number("q4b"),
            confirmed_read_errors: number("q4c"),
            unconfirmed_read_errors: number("q4d"),
            phantom: number("q4e"),
            open_failed: number("q4f") == 1,
            open_microseconds: number("open_microseconds"),
            silently_corrupted_blocks: field_value(line, "q4b_blocks").unwrap_or("-").to_string(),
        }
    }

    fn lost_signal(&self) -> bool {
        self.lost_confirmed + self.confirmed_read_errors > 0 || self.open_failed
    }

    fn any_positive(&self) -> bool {
        self.lost_signal() || self.silently_corrupted + self.unconfirmed_read_errors + self.phantom > 0
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct PointSummary {
    points: usize,
    judged_points: usize,
    lost_confirmed_total: u64,
    corrupted_total: u64,
    unopenable_points: usize,
    positive_points: usize,
    lost_signal_points: usize,
    lost_confirmed_points: usize,
    first_positive_point: Option<usize>,
    peak_single_point: u64,
    last_point_value: u64,
    open_peak_microseconds: u64,
    open_median_microseconds: u64,
    open_last_microseconds: u64,
    slow_opens: usize,
    unjudged: usize,
    mount_failures: usize,
}

fn summarize_points(results: &[&PointResult]) -> PointSummary {
    let mut sorted: Vec<&PointResult> = results.to_vec();
    sorted.sort_by_key(|result| (result.item.point, result.item.item_number));
    let mut summary = PointSummary { points: sorted.len(), ..PointSummary::default() };
    let mut open_times = Vec::new();
    for result in &sorted {
        if !result.mount_ok {
            summary.mount_failures += 1;
        }
        if result.check_label != "done" && result.check_label != "timeout" {
            summary.unjudged += 1;
            continue;
        }
        summary.judged_points += 1;
        summary.lost_confirmed_total += result.lost_confirmed + result.confirmed_read_errors;
        summary.corrupted_total += result.silently_corrupted + result.unconfirmed_read_errors + result.phantom;
        if result.open_failed {
            summary.unopenable_points += 1;
        }
        if result.lost_signal() {
            summary.lost_signal_points += 1;
        }
        if result.lost_confirmed > 0 {
            summary.lost_confirmed_points += 1;
        }
        let point_value = result.lost_confirmed
            + result.confirmed_read_errors
            + result.silently_corrupted
            + result.unconfirmed_read_errors
            + result.phantom
            + u64::from(result.open_failed);
        if point_value > 0 {
            summary.positive_points += 1;
            summary.first_positive_point.get_or_insert(result.item.point);
        }
        summary.peak_single_point = summary.peak_single_point.max(point_value);
        summary.last_point_value = point_value;
        open_times.push(result.open_microseconds);
        summary.open_last_microseconds = result.open_microseconds;
        if result.open_microseconds > SLOW_OPEN_MICROSECONDS {
            summary.slow_opens += 1;
        }
    }
    summary.open_peak_microseconds = open_times.iter().copied().max().unwrap_or(0);
    summary.open_median_microseconds = median_of(&open_times);
    summary
}

fn summary_fields(summary: &PointSummary) -> String {
    format!(
        "points={} judged_points={} unjudged_points={} mount_failures={} lost_confirmed={} corrupted={} unopenable={} positive_points={} first_positive_point={} peak_single_point={} last_point_value={} open_peak_microseconds={} open_median_microseconds={} open_last_microseconds={} opens_over_60_seconds={}",
        summary.points,
        summary.judged_points,
        summary.unjudged,
        summary.mount_failures,
        summary.lost_confirmed_total,
        summary.corrupted_total,
        summary.unopenable_points,
        summary.positive_points,
        summary.first_positive_point.map_or_else(|| "none".to_string(), |point| point.to_string()),
        summary.peak_single_point,
        summary.last_point_value,
        summary.open_peak_microseconds,
        summary.open_median_microseconds,
        summary.open_last_microseconds,
        summary.slow_opens
    )
}

/// 一次写盘开机连同它的验盘之后留下的、判定要用的东西。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct BootOutcome {
    replay_fidelity_holds: Option<bool>,
    markers_hold: Option<bool>,
    write_cache_is_write_back: Option<bool>,
    barrier_flushes_between_markers: Option<usize>,
    unflushed_stratified_points: Option<usize>,
    rocksdb_background_holds: Option<bool>,
    points: Vec<PointResult>,
}

/// 驱动的停机：条款与说明。
struct DriveHalt {
    clause: String,
    detail: String,
}

struct DriveContext {
    kernel: String,
    launcher: PathBuf,
    write_root: PathBuf,
    scratch: PathBuf,
}

fn sha256_of_file(path: &Path) -> Result<String, String> {
    let text = run_tool("sha256sum", &[&path.to_string_lossy()], None)?;
    text.split_whitespace().next().map(str::to_string).ok_or_else(|| "sha256sum 没有输出".to_string())
}

/// PC4-R：把这次开机的全部条目重放到全 0 的镜像上，与它的 disk0.img 整盘 sha256 比。
fn replay_fidelity(output: &mut ProductLines, boot_label: &str, boot_directory: &Path, log: &LogBytes, device_log: &DeviceLog) -> Result<bool, String> {
    let disk_path = boot_directory.join("disk0.img");
    let disk_length = fs::metadata(&disk_path).map_err(|error| format!("读不到 disk0.img：{error}"))?.len();
    let replay_path = boot_directory.join("replay-all.img");
    let replay_file = OpenOptions::new().write(true).create_new(true).open(&replay_path).map_err(|error| error.to_string())?;
    replay_file.set_len(disk_length).map_err(|error| error.to_string())?;
    let selection = select_replayed_writes(&device_log.entries, device_log.entries.len(), ReplayVariant::KeepUnflushed, 0, 0);
    let mut image = ReplayImage::OnDisk(replay_file);
    apply_replayed_writes(log, &selection.writes, &mut image)?;
    drop(image);
    let replay_digest = sha256_of_file(&replay_path)?;
    let disk_digest = sha256_of_file(&disk_path)?;
    let _removed = fs::remove_file(&replay_path);
    let identical = replay_digest == disk_digest;
    output.line(format!(
        "name=pc4_replay_fidelity boot={boot_label} entries={} replay_sha256={replay_digest} disk_sha256={disk_digest} identical={identical}",
        device_log.entries.len()
    ));
    Ok(identical)
}

fn log_summary_line(boot_label: &str, device_log: &DeviceLog, entries_until_zero_header: usize) -> String {
    let mut writes = 0usize;
    let mut forced_writes = 0usize;
    let mut flushes = 0usize;
    let mut discards = 0usize;
    for entry in &device_log.entries {
        match entry {
            LogEntry::Write { is_forced_unit_access, .. } => {
                writes += 1;
                if *is_forced_unit_access {
                    forced_writes += 1;
                }
            }
            LogEntry::Flush => flushes += 1,
            LogEntry::Discard { .. } => discards += 1,
        }
    }
    format!(
        "name=log_summary boot={boot_label} declared_entries={} parsed_entries={} entries_until_zero_header={entries_until_zero_header} writes={writes} fua_writes={forced_writes} flushes={flushes} discards={discards} end_offset_in_log={}",
        device_log.declared_entry_count.map_or_else(|| "none".to_string(), |declared| declared.to_string()),
        device_log.entries.len(),
        device_log.end_offset_in_log
    )
}

/// 一批验盘开机：搭验盘附加根、起虚机、收 s4_point 行；缺行的「断电点 × 取法」重做一次（V30），还缺就停机 H10。
fn run_verify_batches(
    output: &mut ProductLines,
    context: &DriveContext,
    boot: &PlannedWriteBoot,
    boot_directory: &Path,
    device_log: &DeviceLog,
    header: &PlanHeader,
    items: &[VerifyItem],
) -> Result<Vec<PointResult>, DriveHalt> {
    let mut results = Vec::new();
    for (batch_number, batch) in items.chunks(VERIFY_BATCH_ITEM_LIMIT).enumerate() {
        let mut attempt = 0usize;
        loop {
            let verify_root = context.scratch.join(format!("verify-root-{}-{batch_number}-{attempt}", boot.order));
            let staged = link_tree(&context.write_root, &verify_root)
                .and_then(|()| fs::create_dir_all(verify_root.join("s4")).map_err(|error| error.to_string()))
                .and_then(|()| copy_log_prefix(&boot_directory.join("log0.img"), device_log.end_offset_in_log, &verify_root.join(GUEST_LOG_PATH.trim_start_matches('/'))))
                .and_then(|()| {
                    let mut plan_text = plan_header_line(header);
                    plan_text.push('\n');
                    for item in batch {
                        plan_text.push_str(&item.plan_line());
                        plan_text.push('\n');
                    }
                    fs::write(verify_root.join(GUEST_PLAN_PATH.trim_start_matches('/')), plan_text).map_err(|error| error.to_string())
                });
            if let Err(detail) = staged {
                let _cleanup = fs::remove_dir_all(&verify_root);
                return Err(DriveHalt { clause: "H10".to_string(), detail: format!("搭不出验盘附加根：{detail}") });
            }
            let run = verify_boot_run(&verify_root, &context.kernel);
            let boot_label = format!("{}-{}-verify{batch_number}-attempt{attempt}", boot.order, boot.arm.label());
            output.line(format!("name=boot_started boot={boot_label} kind=verify items={} {}", batch.len(), run_description(&run, &context.launcher)));
            let (guest_lines, exit_code) = execute_virtual_machine(&run, &context.launcher);
            let _cleanup = fs::remove_dir_all(&verify_root);
            let mut lines_by_item: BTreeMap<usize, String> = BTreeMap::new();
            for guest_line in &guest_lines {
                output.line(tagged_guest_line(&boot_label, guest_line));
                if guest_line.starts_with("name=s4_point ") {
                    if let Some(item_number) = field_number(guest_line, "item").and_then(|number| usize::try_from(number).ok()) {
                        lines_by_item.insert(item_number, guest_line.clone());
                    }
                }
            }
            let has_done = guest_lines.iter().any(|line| line.starts_with("name=done "));
            let missing: Vec<usize> = batch.iter().map(|item| item.item_number).filter(|number| !lines_by_item.contains_key(number)).collect();
            output.line(format!(
                "name=boot_finished boot={boot_label} exit_code={exit_code:?} guest_lines={} done_line={has_done} missing_items={}",
                guest_lines.len(),
                missing.len()
            ));
            if exit_code == Some(0) && has_done && missing.is_empty() {
                results.extend(batch.iter().map(|item| PointResult::from_line(item, &lines_by_item[&item.item_number])));
                if results.iter().any(|result| result.check_label == "loop_failed" || result.check_label == "image_failed") {
                    return Err(DriveHalt { clause: "H8".to_string(), detail: format!("{boot_label} 里有断电点 losetup 失败或 tmpfs 上造不出镜像") });
                }
                if results.iter().any(|result| !result.mount_ok && result.check_label == "not_run") {
                    return Err(DriveHalt { clause: "H7".to_string(), detail: format!("{} 的某个断电点重放出的盘面挂不上 ext4（Q4m > 0）", boot_label) });
                }
                break;
            }
            if attempt >= 1 {
                return Err(DriveHalt { clause: "H10".to_string(), detail: format!("{boot_label} 重做一次仍缺收尾行或缺 {} 个断电点（V30）", missing.len()) });
            }
            output.line(format!("name=v30_redo boot={boot_label} missing_items={}", missing.len()));
            attempt += 1;
        }
    }
    Ok(results)
}

pub(super) fn guest_confirmed_blocks(guest_lines: &[String]) -> Option<u64> {
    guest_lines.iter().find(|line| line.starts_with("name=write_guest ")).and_then(|line| field_number(line, "confirmed"))
}

/// 写盘开机一次，连同 PC4-R、标记核对、分桶与它的全部验盘开机；K 的主格照 V25 翻倍重做。
fn run_one_write_boot(output: &mut ProductLines, context: &DriveContext, planned: &PlannedWriteBoot) -> Result<BootOutcome, DriveHalt> {
    let mut boot = *planned;
    let mut redo_count = 0usize;
    loop {
        let boot_label = format!("{}-{}-write{redo_count}", boot.order, boot.arm.label());
        let boot_directory = context.scratch.join(format!("boot-{boot_label}"));
        fs::create_dir(&boot_directory).map_err(|error| DriveHalt { clause: "H10".to_string(), detail: error.to_string() })?;
        let run = write_boot_run(&boot, &boot_directory, &context.write_root, &context.kernel);
        output.line(format!("name=boot_started boot={boot_label} kind=write arm={} arm_index={} blocks={} {}", boot.arm.label(), boot.arm.index(), boot.block_count, run_description(&run, &context.launcher)));
        let (guest_lines, exit_code) = execute_virtual_machine(&run, &context.launcher);
        for guest_line in &guest_lines {
            output.line(tagged_guest_line(&boot_label, guest_line));
        }
        output.line(format!("name=boot_finished boot={boot_label} exit_code={exit_code:?} guest_lines={}", guest_lines.len()));
        if exit_code != Some(0) {
            let clause = guest_lines.iter().find(|line| line.starts_with("name=halt ")).and_then(|line| field_value(line, "clause")).unwrap_or("H10").to_string();
            let _cleanup = fs::remove_dir_all(&boot_directory);
            return Err(DriveHalt { clause, detail: format!("写盘开机 {boot_label} 退出码 {exit_code:?}") });
        }
        let outcome = judge_write_boot(output, context, &boot, &boot_label, &boot_directory, &guest_lines);
        let redo_for_background = match &outcome {
            Ok(result) => {
                result.rocksdb_background_holds == Some(false) && boot.arm == PowerCutArm::RocksDatabase && boot.block_count * 2 <= ROCKSDB_REDO_BLOCK_LIMIT
            }
            Err(_halt) => false,
        };
        let _cleanup = fs::remove_dir_all(&boot_directory);
        if !redo_for_background {
            return outcome;
        }
        output.line(format!("name=v25_redo boot={boot_label} blocks_before={} blocks_after={}", boot.block_count, boot.block_count * 2));
        boot.block_count *= 2;
        redo_count += 1;
    }
}

fn judge_write_boot(
    output: &mut ProductLines,
    context: &DriveContext,
    boot: &PlannedWriteBoot,
    boot_label: &str,
    boot_directory: &Path,
    guest_lines: &[String],
) -> Result<BootOutcome, DriveHalt> {
    let halt = |clause: &str, detail: String| DriveHalt { clause: clause.to_string(), detail };
    let log = LogBytes::open(&boot_directory.join("log0.img")).map_err(|detail| halt("H9", detail))?;
    let device_log = parse_device_log(&log, DeclaredCountHandling::StopAtDeclared).map_err(|error| halt("H9", error.to_string()))?;
    let entries_until_zero_header = parse_device_log(&log, DeclaredCountHandling::ReadUntilZeroHeader).map_or(0, |until_zero| until_zero.entries.len());
    output.line(log_summary_line(boot_label, &device_log, entries_until_zero_header));
    let mut outcome = BootOutcome::default();
    let fidelity = replay_fidelity(output, boot_label, boot_directory, &log, &device_log).map_err(|detail| halt("H10", detail))?;
    outcome.replay_fidelity_holds = Some(fidelity);
    if !fidelity && entries_until_zero_header != device_log.entries.len() {
        return Err(halt("H9", format!("{boot_label}：声明条目数 {} 与读到全 0 为止的 {entries_until_zero_header} 不同，PC4-R 又过不了", device_log.entries.len())));
    }
    let environment_line = guest_lines.iter().find(|line| line.starts_with("name=guest_environment "));
    outcome.write_cache_is_write_back = environment_line.map(|line| field_value(line, "write_cache") == Some("write_back"));
    if boot.arm == PowerCutArm::WriteCacheModel {
        let judgement = judge_write_cache_model(&log, &device_log).map_err(|detail| halt("H10", detail))?;
        let lost_fields: Vec<String> = judgement
            .lost_by_variant
            .iter()
            .map(|(variant, first_lost, second_lost)| format!("{}_first_lost={first_lost} {}_second_lost={second_lost}", variant.label(), variant.label()))
            .collect();
        output.line(format!(
            "name=verdict part=pc4_c boot={boot_label} point={} entries_between_batches={} flushes_between_batches={} {} registered_counts_hold={}",
            judgement.point,
            judgement.entries_between_batches,
            judgement.flushes_between_batches,
            lost_fields.join(" "),
            judgement.registered_counts_hold
        ));
        outcome.markers_hold = Some(judgement.registered_counts_hold);
        return Ok(outcome);
    }
    let scan = scan_markers(&log, &device_log).map_err(|detail| halt("H10", detail))?;
    let guest_confirmed = guest_confirmed_blocks(guest_lines).unwrap_or(u64::MAX);
    let audit = audit_markers(&scan, boot.arm.index(), guest_confirmed);
    output.line(format!(
        "name=marker_check boot={boot_label} markers={} confirm_markers={} guest_confirmed={guest_confirmed} problems={} markers_hold={}",
        scan.markers.len(),
        audit.confirm_count,
        if audit.problems.is_empty() { "none".to_string() } else { audit.problems.join(",") },
        audit.windows.is_some()
    ));
    outcome.markers_hold = Some(audit.windows.is_some());
    let Some(windows) = audit.windows else {
        return Ok(outcome);
    };
    let discards_between = device_log.entries[windows.start_entry..windows.end_entry].iter().filter(|entry| matches!(entry, LogEntry::Discard { .. })).count();
    let flushes_between = device_log.entries[windows.start_entry..windows.end_entry].iter().filter(|entry| **entry == LogEntry::Flush).count();
    outcome.barrier_flushes_between_markers = Some(flushes_between);
    if discards_between > 0 {
        return Err(halt("H14", format!("{boot_label}：S 与 E 之间有 {discards_between} 个 DISCARD 条目")));
    }
    if boot.arm == PowerCutArm::RocksDatabase {
        let background = guest_lines.iter().find(|line| line.starts_with("name=rocksdb_background "));
        let flushes = background.and_then(|line| field_number(line, "flushes")).unwrap_or(0);
        let compactions = background.and_then(|line| field_number(line, "compactions")).unwrap_or(0);
        let holds = flushes >= ROCKSDB_FLUSH_MINIMUM && compactions >= ROCKSDB_COMPACTION_MINIMUM;
        outcome.rocksdb_background_holds = Some(holds);
        output.line(format!("name=q4q boot={boot_label} flushes={flushes} compactions={compactions} background_writes_in_sampled_range={holds}"));
        if !holds && boot.block_count * 2 <= ROCKSDB_REDO_BLOCK_LIMIT {
            return Ok(outcome);
        }
    }
    let pools = bucket_pools(&windows);
    let items = verification_items(boot.arm, &windows, &pools, device_log.entries.len());
    let stratified_points: Vec<usize> = items
        .iter()
        .filter(|item| matches!(item.group, PointGroup::StratifiedNormalCommits | PointGroup::StratifiedLongCommits | PointGroup::StratifiedBetweenCommits))
        .map(|item| item.point)
        .collect::<std::collections::BTreeSet<usize>>()
        .into_iter()
        .collect();
    let unflushed_points = stratified_points.iter().filter(|&&point| unflushed_write_count(&device_log.entries, point) > 0).count();
    let inside_commit_points = items
        .iter()
        .filter(|item| matches!(item.group, PointGroup::StratifiedNormalCommits | PointGroup::StratifiedLongCommits) && item.variant == ReplayVariant::DropUnflushed)
        .count();
    outcome.unflushed_stratified_points = Some(unflushed_points);
    output.line(format!(
        "name=cut_plan boot={boot_label} start_entry={} end_entry={} median_window_entries={} normal_windows={} long_windows={} pool_normal={} pool_long={} pool_between={} stratified_points={} q4u_unflushed_points={unflushed_points} q4g_inside_commit_points={inside_commit_points} items={}",
        windows.start_entry,
        windows.end_entry,
        pools.median_window_entries,
        pools.normal_blocks.len(),
        pools.long_blocks.len(),
        pools.normal_commit_points.len(),
        pools.long_commit_points.len(),
        pools.between_commit_points.len(),
        stratified_points.len(),
        items.len()
    ));
    let disk_bytes = fs::metadata(boot_directory.join("disk0.img")).map_err(|error| halt("H10", error.to_string()))?.len();
    let header = PlanHeader {
        arm: boot.arm,
        block_count: boot.block_count,
        states_per_block: boot.arm.states_per_block(),
        disk_bytes,
        start_entry: windows.start_entry,
        end_entry: windows.end_entry,
    };
    outcome.points = run_verify_batches(output, context, boot, boot_directory, &device_log, &header, &items)?;
    Ok(outcome)
}

fn damage_verdict_line(arm: PowerCutArm, damage: LibraryDamage, result: Option<&PointResult>) -> String {
    let matches = result.is_some_and(|result| match damage {
        LibraryDamage::SilentCorruption => {
            result.silently_corrupted == 1
                && result.silently_corrupted_blocks == SILENT_CORRUPTION_BLOCK.to_string()
                && result.lost_confirmed + result.confirmed_read_errors + result.unconfirmed_read_errors + result.phantom == 0
                && !result.open_failed
        }
        LibraryDamage::PhantomKey => {
            result.phantom == 1 && result.lost_confirmed + result.silently_corrupted + result.confirmed_read_errors + result.unconfirmed_read_errors == 0 && !result.open_failed
        }
        LibraryDamage::OpenBreaking => result.open_failed,
        LibraryDamage::Untouched => false,
    });
    let part = match damage {
        LibraryDamage::SilentCorruption => "pc4_s",
        LibraryDamage::PhantomKey => "pc4_p",
        LibraryDamage::OpenBreaking => "pc4_o",
        LibraryDamage::Untouched => "pc4_untouched",
    };
    format!(
        "name=verdict part={part} arm={} q4a={} q4b={} q4b_blocks={} q4c={} q4d={} q4e={} q4f={} matches_registration={matches}",
        arm.label(),
        result.map_or(0, |result| result.lost_confirmed),
        result.map_or(0, |result| result.silently_corrupted),
        result.map_or("-", |result| result.silently_corrupted_blocks.as_str()),
        result.map_or(0, |result| result.confirmed_read_errors),
        result.map_or(0, |result| result.unconfirmed_read_errors),
        result.map_or(0, |result| result.phantom),
        result.map_or(0, |result| u8::from(result.open_failed))
    )
}

/// 补 6、补 10、补 11：每条主臂的判定行与各阳性对照、作废条款的判定行。
fn emit_verdicts(output: &mut ProductLines, main_arms: &[PowerCutArm], outcomes: &BTreeMap<u32, BootOutcome>) {
    let model_holds = outcomes.get(&PowerCutArm::WriteCacheModel.index()).and_then(|outcome| outcome.markers_hold).unwrap_or(false);
    output.line(format!("name=verdict part=v17_write_cache_model write_cache_model_holds={model_holds}"));
    let mut decision_flips = Vec::new();
    for &arm in main_arms {
        let empty = BootOutcome::default();
        let main = outcomes.get(&arm.index()).unwrap_or(&empty);
        let page_cache_control = outcomes.get(&arm.lost_page_cache_control().index()).unwrap_or(&empty);
        let write_cache_control = outcomes.get(&arm.lost_write_cache_control().index()).unwrap_or(&empty);
        let page_cache_signal = page_cache_control.points.iter().filter(|result| result.lost_signal()).count();
        let write_cache_signal = write_cache_control.points.iter().filter(|result| result.lost_signal()).count();
        let write_cache_control_holds = write_cache_control.barrier_flushes_between_markers == Some(0);
        output.line(format!(
            "name=verdict part=pc4_l arm={} control={} points={} signal_points={page_cache_signal} required={CONTROL_SIGNAL_POINT_MINIMUM} sees_lost_page_cache={}",
            arm.label(),
            arm.lost_page_cache_control().label(),
            page_cache_control.points.len(),
            page_cache_signal >= CONTROL_SIGNAL_POINT_MINIMUM
        ));
        output.line(format!(
            "name=verdict part=pc4_nb arm={} control={} points={} signal_points={write_cache_signal} required={CONTROL_SIGNAL_POINT_MINIMUM} sees_lost_write_cache={} barrier_flushes_between_markers={} barrier_off_holds={write_cache_control_holds}",
            arm.label(),
            arm.lost_write_cache_control().label(),
            write_cache_control.points.len(),
            write_cache_signal >= CONTROL_SIGNAL_POINT_MINIMUM,
            write_cache_control.barrier_flushes_between_markers.map_or_else(|| "unknown".to_string(), |count| count.to_string())
        ));
        if arm == PowerCutArm::HardenedFile {
            let directory_control = outcomes.get(&PowerCutArm::HardenedFileWithoutDirectorySync.index()).unwrap_or(&empty);
            let directory_signal = directory_control.points.iter().filter(|result| result.lost_confirmed > 0).count();
            output.line(format!(
                "name=verdict part=pc4_fd arm=F control=F-nodirsync points={} q4a_points={directory_signal} required={CONTROL_SIGNAL_POINT_MINIMUM} sees_missing_directory_sync={}",
                directory_control.points.len(),
                directory_signal >= CONTROL_SIGNAL_POINT_MINIMUM
            ));
        }
        for damage in [LibraryDamage::SilentCorruption, LibraryDamage::PhantomKey, LibraryDamage::OpenBreaking] {
            let result = main.points.iter().find(|result| result.item.damage == damage);
            output.line(damage_verdict_line(arm, damage, result));
        }
        let validity = format!(
            "replay_fidelity_holds={} markers_hold={} write_cache_is_write_back={} q4u_unflushed_points={} unflushed_points_enough={}{}",
            main.replay_fidelity_holds.unwrap_or(false),
            main.markers_hold.unwrap_or(false),
            main.write_cache_is_write_back.unwrap_or(false),
            main.unflushed_stratified_points.map_or_else(|| "unknown".to_string(), |count| count.to_string()),
            main.unflushed_stratified_points.is_some_and(|count| count >= UNFLUSHED_POINT_MINIMUM),
            if arm == PowerCutArm::RocksDatabase {
                format!(" background_writes_in_sampled_range={}", main.rocksdb_background_holds.unwrap_or(false))
            } else {
                String::new()
            }
        );
        let decision_points: Vec<&PointResult> = main
            .points
            .iter()
            .filter(|result| result.item.damage == LibraryDamage::Untouched && result.item.variant == ReplayVariant::DropUnflushed)
            .collect();
        let decision = summarize_points(&decision_points);
        let flips = decision_points.iter().any(|result| result.any_positive());
        decision_flips.push((arm, flips));
        output.line(format!("name=verdict part=s4_decision arm={} reading=U-drop groups=C-strat,C-commit {} {validity} flips={flips}", arm.label(), summary_fields(&decision)));
        for variant in [ReplayVariant::KeepUnflushed, ReplayVariant::RandomSubset, ReplayVariant::RandomSubsetWithTear] {
            let sampled: Vec<&PointResult> = main
                .points
                .iter()
                .filter(|result| result.item.damage == LibraryDamage::Untouched && result.item.variant == variant)
                .collect();
            let sampled_flips = sampled.iter().any(|result| result.any_positive());
            output.line(format!("name=verdict part=s4_sampling arm={} reading={} {} flips={sampled_flips}", arm.label(), variant.label(), summary_fields(&summarize_points(&sampled))));
        }
        let small = outcomes.get(&arm.small_block_sampling_point().index()).unwrap_or(&empty);
        for variant in [ReplayVariant::DropUnflushed, ReplayVariant::RandomSubset] {
            let sampled: Vec<&PointResult> = small.points.iter().filter(|result| result.item.variant == variant).collect();
            let sampled_flips = sampled.iter().any(|result| result.any_positive());
            output.line(format!(
                "name=verdict part=s4_sampling arm={} reading=bs10-{} {} replay_fidelity_holds={} markers_hold={} flips={sampled_flips}",
                arm.label(),
                variant.label(),
                summary_fields(&summarize_points(&sampled)),
                small.replay_fidelity_holds.unwrap_or(false),
                small.markers_hold.unwrap_or(false)
            ));
        }
    }
    let every_arm_flips = !decision_flips.is_empty() && decision_flips.iter().all(|(_arm, flips)| *flips);
    output.line(format!("name=verdict part=f13_all_arms_flip reading=U-drop arms={} every_arm_flips={every_arm_flips}", decision_flips.len()));
}

fn parse_main_arms(text: &str) -> Option<Vec<PowerCutArm>> {
    let arms: Option<Vec<PowerCutArm>> = text.split(',').map(PowerCutArm::main_arm_from_label).collect();
    arms.filter(|parsed| !parsed.is_empty())
}

/// `--arms` 的值定哪几条主臂：不给就是全部四臂，给了照写的取（写错是 `None`）。
pub(super) fn selected_main_arms(arms_text: Option<&str>) -> Option<Vec<PowerCutArm>> {
    match arms_text {
        None => Some(PowerCutArm::EVERY_MAIN_ARM.to_vec()),
        Some(text) => parse_main_arms(text),
    }
}

fn write_launcher(scratch: &Path) -> Result<PathBuf, String> {
    let launcher = scratch.join("launcher.sh");
    fs::write(&launcher, format!("#!/bin/sh\nexec {STAGED_BINARY_GUEST_PATH} \"$@\"\n")).map_err(|error| error.to_string())?;
    fs::set_permissions(&launcher, fs::Permissions::from_mode(0o755)).map_err(|error| error.to_string())?;
    Ok(launcher)
}

/// 验盘开机的「断电点 × 取法」数的上界（干跑时打印用，真数要等写盘开机的日志）。
fn verify_items_upper_bound(arm: PowerCutArm) -> usize {
    match arm.purpose() {
        ArmPurpose::Main => STRATIFIED_POINT_TOTAL * ALL_REPLAY_VARIANTS.len() + PICKED_COMMIT_TOTAL * COMMIT_BOUNDARY_LIMIT * 2 + 3,
        ArmPurpose::SmallBlockSamplingPoint => SMALL_BLOCK_POINT_TOTAL * 2,
        ArmPurpose::LostPageCacheControl | ArmPurpose::LostWriteCacheControl | ArmPurpose::MissingDirectorySyncControl => CONTROL_POINT_TOTAL,
        ArmPurpose::WriteCacheModelControl => 0,
    }
}

pub(super) fn run_drive_role(arguments: &RoleArguments, forced_summary: Option<&str>) -> i32 {
    let is_dry_run = arguments.flag("dry-run");
    let Some(main_arms) = selected_main_arms(arguments.text("arms")) else {
        eprintln!("  ✗ --arms 要写成 R1,R0,K,F 的子集（逗号分隔），不给就是全部四臂");
        return USAGE_EXIT_CODE;
    };
    let kernel = arguments.text("kernel").map(str::to_string);
    if !is_dry_run && kernel.as_deref().is_none_or(|path| !Path::new(path).is_file()) {
        eprintln!("  ✗ --kernel 要给一个可读的内核镜像（补 5.2「内核」）");
        return USAGE_EXIT_CODE;
    }
    let kernel_text = kernel.unwrap_or_else(|| "<kernel>".to_string());
    let mut output = match arguments.text("out") {
        Some(path) => match ProductLines::with_file_copy(Path::new(path)) {
            Ok(output) => output,
            Err(detail) => {
                eprintln!("  ✗ {detail}");
                return USAGE_EXIT_CODE;
            }
        },
        None => ProductLines::to_standard_output(),
    };
    if let Some(summary) = forced_summary {
        output.line(format!("name=preflight forced={summary}"));
    }
    let exit_code = drive_body(&mut output, &main_arms, &kernel_text, is_dry_run);
    output.finish();
    exit_code
}

/// 只跑了四条主臂的一部分就是缩域：`s4_drive_config` 行接这一段，产物不当正式产物；四臂全跑不接。
pub(super) fn reduced_scale_note(main_arms: &[PowerCutArm]) -> String {
    if PowerCutArm::EVERY_MAIN_ARM.iter().all(|arm| main_arms.contains(arm)) {
        String::new()
    } else {
        " reduced=yes registered_arms=R1,R0,K,F not_official_product=yes".to_string()
    }
}

fn drive_body(output: &mut ProductLines, main_arms: &[PowerCutArm], kernel: &str, is_dry_run: bool) -> i32 {
    let arm_labels: Vec<&str> = main_arms.iter().map(|arm| arm.label()).collect();
    output.line(format!(
        "name=s4_drive_config dry_run={is_dry_run} arms={} kernel={kernel} main_blocks={MAIN_BLOCK_COUNT} control_blocks={CONTROL_BLOCK_COUNT} states_per_block={PRIMARY_STATES_PER_BLOCK} small_states_per_block={SMALL_STATES_PER_BLOCK}{}",
        arm_labels.join(","),
        reduced_scale_note(main_arms)
    ));
    let mismatches = run_anchors(output);
    output.line(format!("name=verdict part=anchors anchor_mismatches={mismatches}"));
    if mismatches != 0 {
        return 1;
    }
    let scratch = match ScratchDirectory::create() {
        Ok(scratch) => scratch,
        Err(detail) => return halt_line(output, "H10", "scratch", &detail),
    };
    let memory = available_memory_bytes();
    let disk = available_disk_bytes(&scratch.path);
    let interfering = interfering_processes();
    let resources_ok = memory >= MINIMUM_AVAILABLE_MEMORY_BYTES && disk >= MINIMUM_AVAILABLE_DISK_BYTES && interfering.is_empty();
    output.line(format!(
        "name=host_resources scratch={} available_memory_bytes={memory} available_disk_bytes={disk} interfering_processes={} resources_ok={resources_ok}",
        scratch.path.display(),
        if interfering.is_empty() { "none".to_string() } else { interfering.join(",") }
    ));
    let write_root = scratch.path.join("write-root");
    match stage_write_root(&write_root) {
        Ok(summary) => {
            output.line(stage_summary_line(&write_root, &summary));
            if !summary.staged_selftest_passed {
                return halt_line(output, "V10", "staged-selftest", "搭好的加载器跑搭好的二进制 --role selftest 没过");
            }
        }
        Err(detail) => return halt_line(output, "H10", "stage", &detail),
    }
    let launcher = match write_launcher(&scratch.path) {
        Ok(launcher) => launcher,
        Err(detail) => return halt_line(output, "H10", "launcher", &detail),
    };
    let context = DriveContext { kernel: kernel.to_string(), launcher, write_root, scratch: scratch.path.clone() };
    let boots = planned_write_boots(main_arms);
    if is_dry_run {
        for boot in &boots {
            let boot_label = format!("{}-{}-write0", boot.order, boot.arm.label());
            let run = write_boot_run(boot, &context.scratch.join(format!("boot-{boot_label}")), &context.write_root, kernel);
            output.line(format!(
                "name=planned_boot order={} kind=write arm={} arm_index={} blocks={} states_per_block={} {}",
                boot.order,
                boot.arm.label(),
                boot.arm.index(),
                boot.block_count,
                boot.arm.states_per_block(),
                run_description(&run, &context.launcher)
            ));
            let items_upper_bound = verify_items_upper_bound(boot.arm);
            if items_upper_bound > 0 {
                let batches = items_upper_bound.div_ceil(VERIFY_BATCH_ITEM_LIMIT);
                let run = verify_boot_run(&context.scratch.join(format!("verify-root-{}-<batch>-<attempt>", boot.order)), kernel);
                output.line(format!(
                    "name=planned_boot order={} kind=verify after_write_boot={boot_label} items_at_most={items_upper_bound} batches_at_most={batches} {}",
                    boot.order,
                    run_description(&run, &context.launcher)
                ));
            }
        }
        output.line(format!("name=dry_run_summary write_boots={} note=not_started", boots.len()));
        return 0;
    }
    if !resources_ok {
        return halt_line(output, "precondition", "host-resources", "可用内存 < 16 GiB、可用盘 < 40 GiB，或机器上有别的虚机与性能测量（补 5.3）");
    }
    let mut outcomes: BTreeMap<u32, BootOutcome> = BTreeMap::new();
    for boot in &boots {
        match run_one_write_boot(output, &context, boot) {
            Ok(outcome) => {
                outcomes.insert(boot.arm.index(), outcome);
            }
            Err(halt) => {
                emit_verdicts(output, main_arms, &outcomes);
                return halt_line(output, &halt.clause, &format!("boot-{}-{}", boot.order, boot.arm.label()), &halt.detail);
            }
        }
    }
    emit_verdicts(output, main_arms, &outcomes);
    0
}
