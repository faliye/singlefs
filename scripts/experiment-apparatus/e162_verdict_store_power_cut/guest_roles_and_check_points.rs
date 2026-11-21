//! 来宾里跑的角色：写来宾、核对来宾、逐检查点核对、核对报告字段。

use super::*;

// ============================== 来宾：公共 ==============================

pub(super) fn run_tool(program: &str, arguments: &[&str], standard_input: Option<&str>) -> Result<String, String> {
    let mut command = Command::new(program);
    command.args(arguments).stdout(Stdio::piped()).stderr(Stdio::piped());
    command.stdin(if standard_input.is_some() { Stdio::piped() } else { Stdio::null() });
    let mut child = command.spawn().map_err(|error| format!("起不了 {program}：{error}"))?;
    if let Some(text) = standard_input {
        let mut child_input = child.stdin.take().expect("标准输入接在管道上");
        child_input.write_all(text.as_bytes()).map_err(|error| format!("写不进 {program} 的标准输入：{error}"))?;
    }
    let output = child.wait_with_output().map_err(|error| format!("等不到 {program}：{error}"))?;
    let combined = format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    if output.status.success() {
        Ok(combined)
    } else {
        Err(format!("{program} {} 退出 {:?}：{combined}", arguments.join(" "), output.status.code()))
    }
}

fn read_trimmed(path: &str) -> String {
    sanitized(fs::read_to_string(path).unwrap_or_else(|_error| "unreadable".to_string()).trim())
}

fn guest_environment_line(role: &str, arm: PowerCutArm) -> String {
    let dirty_fields: Vec<String> = [
        "dirty_background_bytes",
        "dirty_background_ratio",
        "dirty_bytes",
        "dirty_ratio",
        "dirty_expire_centisecs",
        "dirty_writeback_centisecs",
    ]
    .iter()
    .map(|name| format!("{name}={}", read_trimmed(&format!("/proc/sys/vm/{name}"))))
    .collect();
    format!(
        "name=guest_environment role={role} arm={} kernel_release={} write_cache={} fua={} {}",
        arm.label(),
        read_trimmed("/proc/sys/kernel/osrelease"),
        read_trimmed("/sys/block/vda/queue/write_cache"),
        read_trimmed("/sys/block/vda/queue/fua"),
        dirty_fields.join(" ")
    )
}

/// 来宾开头的锚点自检：锚点照跑一遍、行不打出去，只报对不上的个数一行。
fn emit_guest_anchor_check(output: &mut ProductLines, role: &str) -> bool {
    let mut discarded = ProductLines::discarding();
    let mismatches = run_anchors(&mut discarded);
    output.line(format!("name=anchor_self_check role={role} anchor_mismatches={mismatches}"));
    mismatches == 0
}

pub(super) fn halt_line(output: &mut ProductLines, clause: &str, step: &str, detail: &str) -> i32 {
    output.line(format!("name=halt clause={clause} step={step} detail={}", sanitized(detail)));
    HALT_EXIT_CODE
}

// ============================== 来宾：写盘开机 ==============================

pub(super) fn run_write_guest_role(arguments: &RoleArguments) -> i32 {
    let mut output = ProductLines::to_standard_output();
    let exit_code = write_guest_body(&mut output, arguments);
    output.finish();
    exit_code
}

fn write_guest_body(output: &mut ProductLines, arguments: &RoleArguments) -> i32 {
    let Some(arm) = arguments.number("arm").and_then(|index| u32::try_from(index).ok()).and_then(PowerCutArm::from_index) else {
        return halt_line(output, "usage", "arguments", "少了 --arm 或臂号认不出");
    };
    if !emit_guest_anchor_check(output, "s4-write-guest") {
        return halt_line(output, "V10", "anchors", "来宾里锚点对不上");
    }
    output.line(guest_environment_line("write", arm));
    match arm.candidate() {
        None => write_cache_model_guest(output),
        Some(candidate) => {
            let block_count = arguments.number("blocks").unwrap_or_else(|| arm.registered_block_count());
            store_write_guest(output, arm, candidate, block_count)
        }
    }
}

/// PC4-C：/dev/vda 第 8 192 扇区起 O_DIRECT 逐个写 100 个 C 标记，fdatasync，再写 100 个；
/// 末尾再补一次 fdatasync，让设备侧日志的系统配置把 200 个写都数进去（登记第十二节修订：不改断电点 k 与判据）。
fn write_cache_model_guest(output: &mut ProductLines) -> i32 {
    let device = match OpenOptions::new().write(true).custom_flags(OPEN_DIRECT_FLAG).open(GUEST_WHOLE_DISK) {
        Ok(device) => device,
        Err(error) => return halt_line(output, "H8", "open-whole-disk", &error.to_string()),
    };
    let mut aligned = AlignedSector::new();
    let arm_index = PowerCutArm::WriteCacheModel.index();
    for position in 0..2 * WRITE_CACHE_MODEL_BATCH_WRITES {
        if position == WRITE_CACHE_MODEL_BATCH_WRITES {
            if let Err(error) = device.sync_data() {
                return halt_line(output, "H8", "fdatasync-between-batches", &error.to_string());
            }
        }
        let marker = Marker { kind: MarkerKind::Confirm, sequence: position, block_index: Some(position), arm_index };
        let offset = (WRITE_CACHE_MODEL_FIRST_SECTOR + position) * sector_bytes();
        if let Err(error) = device.write_all_at(aligned.fill(&encode_marker(&marker)), offset) {
            return halt_line(output, "H8", "write-marker", &error.to_string());
        }
    }
    if let Err(error) = device.sync_data() {
        return halt_line(output, "H8", "fdatasync-at-end", &error.to_string());
    }
    output.line(format!("name=write_cache_model_guest writes={}", 2 * WRITE_CACHE_MODEL_BATCH_WRITES));
    0
}

fn partition_whole_disk() -> Result<(), String> {
    let script = format!(
        "label: dos\nstart={FIRST_PARTITION_START_SECTOR}, size={}, type=83\nstart={}, size={}, type=83\n",
        first_partition_sectors(),
        marker_partition_start_sector(),
        marker_partition_sectors()
    );
    run_tool(GUEST_PARTITIONER, &[GUEST_WHOLE_DISK], Some(&script))?;
    let _reread_by_sfdisk_already = run_tool(GUEST_BUSYBOX, &["blockdev", "--rereadpt", GUEST_WHOLE_DISK], None);
    run_tool(GUEST_BUSYBOX, &["mdev", "-s"], None)?;
    for partition in [GUEST_FIRST_PARTITION, GUEST_MARKER_PARTITION] {
        if !Path::new(partition).exists() {
            return Err(format!("分区之后没有 {partition}"));
        }
    }
    Ok(())
}

fn mounted_line_of(device: &str) -> String {
    fs::read_to_string("/proc/mounts")
        .unwrap_or_default()
        .lines()
        .find(|line| line.starts_with(device))
        .map(sanitized)
        .unwrap_or_else(|| "absent".to_string())
}

fn store_write_guest(output: &mut ProductLines, arm: PowerCutArm, candidate: Candidate, block_count: u64) -> i32 {
    let settings = arm.write_settings().expect("带候选的臂都有写入设定");
    if let Err(detail) = partition_whole_disk() {
        return halt_line(output, "H8", "partition", &detail);
    }
    if let Err(detail) = run_tool(
        GUEST_FILESYSTEM_FORMATTER,
        &["-F", "-q", "-E", "nodiscard,lazy_itable_init=0,lazy_journal_init=0", GUEST_FIRST_PARTITION],
        None,
    ) {
        return halt_line(output, "H8", "mkfs", &detail);
    }
    if let Err(error) = fs::create_dir_all(GUEST_MOUNT_POINT) {
        return halt_line(output, "H8", "mount-point", &error.to_string());
    }
    let mount_outcome = match settings.filesystem_barrier {
        FilesystemBarrier::Enabled => run_tool(GUEST_BUSYBOX, &["mount", "-t", "ext4", GUEST_FIRST_PARTITION, GUEST_MOUNT_POINT], None),
        FilesystemBarrier::Disabled => {
            run_tool(GUEST_BUSYBOX, &["mount", "-t", "ext4", "-o", "barrier=0", GUEST_FIRST_PARTITION, GUEST_MOUNT_POINT], None)
        }
    };
    if let Err(detail) = mount_outcome {
        return halt_line(output, "H8", "mount", &detail);
    }
    output.line(format!("name=guest_mount role=write arm={} mounts_line={}", arm.label(), mounted_line_of(GUEST_FIRST_PARTITION)));
    let mut sink = match MarkerSink::open_marker_partition(GUEST_MARKER_PARTITION, arm.index()) {
        Ok(sink) => sink,
        Err(detail) => return halt_line(output, "H8", "open-marker-partition", &detail),
    };
    let library = Path::new(GUEST_MOUNT_POINT).join(GUEST_LIBRARY_NAME);
    let mut store = match open_store_for_writing(candidate, settings, &library) {
        Ok(store) => store,
        Err(error) => return halt_line(output, "store-create", "open-for-writing", &error.to_string()),
    };
    let write_outcome = write_blocks_with_markers(&mut store, &mut sink, block_count, arm.states_per_block());
    drop(store);
    let (flushes, compactions) = match candidate {
        Candidate::RocksDatabase => rocksdb_background_counts(&library),
        Candidate::RedbQuickRepair | Candidate::RedbDefault | Candidate::HardenedFile => (0, 0),
    };
    if candidate == Candidate::RocksDatabase {
        output.line(format!("name=rocksdb_background arm={} flushes={flushes} compactions={compactions}", arm.label()));
    }
    let unmount_outcome = run_tool(GUEST_BUSYBOX, &["umount", GUEST_MOUNT_POINT], None);
    let final_flush = OpenOptions::new().write(true).open(GUEST_WHOLE_DISK).and_then(|whole_disk| whole_disk.sync_all());
    let confirmed = match write_outcome {
        Ok(confirmed) => confirmed,
        Err(detail) => return halt_line(output, "write-failed", "write-blocks", &detail),
    };
    output.line(format!(
        "name=write_guest arm={} candidate={} blocks={block_count} states_per_block={} confirmed={confirmed} markers={} unmounted={} final_flush={}",
        arm.label(),
        candidate.label(),
        arm.states_per_block(),
        sink.emitted_marker_count(),
        unmount_outcome.is_ok(),
        final_flush.is_ok()
    ));
    0
}

// ============================== 来宾：验盘开机 ==============================

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PlanHeader {
    pub(super) arm: PowerCutArm,
    pub(super) block_count: u64,
    pub(super) states_per_block: u64,
    pub(super) disk_bytes: u64,
    pub(super) start_entry: usize,
    pub(super) end_entry: usize,
}

pub(super) fn plan_header_line(header: &PlanHeader) -> String {
    format!(
        "header arm={} blocks={} states_per_block={} disk_bytes={} start_entry={} end_entry={}",
        header.arm.index(),
        header.block_count,
        header.states_per_block,
        header.disk_bytes,
        header.start_entry,
        header.end_entry
    )
}

fn parse_plan_header(line: &str) -> Option<PlanHeader> {
    Some(PlanHeader {
        arm: PowerCutArm::from_index(u32::try_from(field_number(line, "arm")?).ok()?)?,
        block_count: field_number(line, "blocks")?,
        states_per_block: field_number(line, "states_per_block")?,
        disk_bytes: field_number(line, "disk_bytes")?,
        start_entry: usize::try_from(field_number(line, "start_entry")?).ok()?,
        end_entry: usize::try_from(field_number(line, "end_entry")?).ok()?,
    })
}

pub(super) fn parse_plan_item(line: &str) -> Option<VerifyItem> {
    let group = match (field_value(line, "group")?, field_value(line, "bucket")?) {
        ("C-strat", "A") => PointGroup::StratifiedNormalCommits,
        ("C-strat", "B") => PointGroup::StratifiedLongCommits,
        ("C-strat", "C") => PointGroup::StratifiedBetweenCommits,
        ("C-commit", _bucket) => PointGroup::CommitSweep {
            picked: field_number(line, "picked")?,
            boundary: field_number(line, "boundary")?,
            block_index: field_number(line, "block")?,
        },
        ("control", _bucket) => PointGroup::ControlUniform,
        ("full-replay", _bucket) => PointGroup::FullReplayDamage,
        (_group, _bucket) => return None,
    };
    Some(VerifyItem {
        item_number: usize::try_from(field_number(line, "number")?).ok()?,
        group,
        point: usize::try_from(field_number(line, "point")?).ok()?,
        variant: ReplayVariant::parse(field_value(line, "variant")?)?,
        subset_index: field_number(line, "subset")?,
        confirmed_end: field_number(line, "confirmed_end")?,
        started_end: field_number(line, "started_end")?,
        damage: LibraryDamage::parse(field_value(line, "damage")?)?,
    })
}

pub(super) fn run_verify_guest_role(arguments: &RoleArguments) -> i32 {
    let mut output = ProductLines::to_standard_output();
    let exit_code = verify_guest_body(&mut output, arguments);
    output.finish();
    exit_code
}

fn verify_guest_body(output: &mut ProductLines, arguments: &RoleArguments) -> i32 {
    if !emit_guest_anchor_check(output, "s4-verify-guest") {
        return halt_line(output, "V10", "anchors", "来宾里锚点对不上");
    }
    let plan_text = match fs::read_to_string(arguments.text("plan").unwrap_or(GUEST_PLAN_PATH)) {
        Ok(text) => text,
        Err(error) => return halt_line(output, "H10", "read-plan", &error.to_string()),
    };
    let Some(header) = plan_text.lines().find(|line| line.starts_with("header ")).and_then(parse_plan_header) else {
        return halt_line(output, "H10", "plan-header", "计划里没有能读的 header 行");
    };
    let items: Vec<VerifyItem> = plan_text.lines().filter(|line| line.starts_with("item ")).filter_map(parse_plan_item).collect();
    output.line(guest_environment_line("verify", header.arm));
    output.line(format!("name=verify_guest_plan arm={} items={}", header.arm.label(), items.len()));
    let log = match LogBytes::open(Path::new(arguments.text("log").unwrap_or(GUEST_LOG_PATH))) {
        Ok(log) => log,
        Err(detail) => return halt_line(output, "H10", "open-log", &detail),
    };
    let device_log = match parse_device_log(&log, DeclaredCountHandling::StopAtDeclared) {
        Ok(device_log) => device_log,
        Err(error) => return halt_line(output, "H9", "parse-log", &error.to_string()),
    };
    if let Err(detail) = fs::create_dir_all(GUEST_IMAGE_DIRECTORY)
        .map_err(|error| error.to_string())
        .and_then(|()| run_tool(GUEST_BUSYBOX, &["mount", "-t", "tmpfs", "-o", GUEST_IMAGE_FILESYSTEM_OPTIONS, "tmpfs", GUEST_IMAGE_DIRECTORY], None).map(|_text| ()))
        .and_then(|()| fs::create_dir_all(GUEST_MOUNT_POINT).map_err(|error| error.to_string()))
    {
        return halt_line(output, "H8", "image-tmpfs", &detail);
    }
    let mut mount_failures = 0u64;
    for item in &items {
        let point_line = verify_one_item(&header, &log, &device_log, item);
        if field_value(&point_line, "mount") == Some("failed") {
            mount_failures += 1;
        }
        output.line(point_line);
    }
    output.line(format!("name=verify_guest_summary arm={} items={} mount_failures={mount_failures}", header.arm.label(), items.len()));
    0
}

enum CheckChildOutcome {
    Finished { report_line: Option<String>, exit_code: Option<i32> },
    TimedOut,
    SpawnFailed(String),
}

/// 子进程的标准输出另起一个线程读到 EOF；这里不计时、不转打。
fn collect_output_in_background(standard_output: std::process::ChildStdout) -> std::thread::JoinHandle<String> {
    std::thread::spawn(move || {
        let mut text = String::new();
        let _read_outcome = BufReader::new(standard_output).read_to_string(&mut text);
        text
    })
}

/// 起核对子进程；3 600 秒没交回就 SIGKILL（`Child::kill`）。等的循环只看它退没退，不读它的输出。
fn run_check_child_with_timeout(role_arguments: &[String], timeout: Duration) -> CheckChildOutcome {
    let executable = match std::env::current_exe() {
        Ok(executable) => executable,
        Err(error) => return CheckChildOutcome::SpawnFailed(error.to_string()),
    };
    let mut child = match Command::new(executable).args(role_arguments).stdout(Stdio::piped()).stderr(Stdio::inherit()).spawn() {
        Ok(child) => child,
        Err(error) => return CheckChildOutcome::SpawnFailed(error.to_string()),
    };
    let reader = collect_output_in_background(child.stdout.take().expect("标准输出接在管道上"));
    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _already_exited = child.kill();
                    let _reaped = child.wait();
                    let _discarded = reader.join();
                    return CheckChildOutcome::TimedOut;
                }
                std::thread::sleep(Duration::from_millis(CHECK_POLL_MILLISECONDS));
            }
            Err(_error) => break None,
        }
    };
    let text = reader.join().unwrap_or_default();
    let report_line = text.lines().find(|line| line.starts_with("S4CHECK ")).map(str::to_string);
    CheckChildOutcome::Finished { report_line, exit_code: status.and_then(|finished| finished.code()) }
}

fn dmesg_tail() -> String {
    let text = run_tool(GUEST_BUSYBOX, &["dmesg"], None).unwrap_or_else(|error| error);
    let lines: Vec<&str> = text.lines().collect();
    let start = lines.len().saturating_sub(12);
    sanitized(&lines[start..].join(" | "))
}

fn verify_one_item(header: &PlanHeader, log: &LogBytes, device_log: &DeviceLog, item: &VerifyItem) -> String {
    let (group_label, bucket_label) = item.group.labels();
    let span = header.end_entry.saturating_sub(header.start_entry).max(1);
    let fraction = (item.point.saturating_sub(header.start_entry)) as f64 / span as f64;
    let selection = select_replayed_writes(&device_log.entries, item.point, item.variant, u64::from(header.arm.index()), item.subset_index);
    let block_field = match item.group {
        PointGroup::CommitSweep { picked, boundary, block_index } => format!("picked={picked} boundary={boundary} block={block_index}"),
        PointGroup::StratifiedNormalCommits
        | PointGroup::StratifiedLongCommits
        | PointGroup::StratifiedBetweenCommits
        | PointGroup::ControlUniform
        | PointGroup::FullReplayDamage => "picked=- boundary=- block=-".to_string(),
    };
    let prefix = format!(
        "name=s4_point arm={} item={} group={group_label} bucket={bucket_label} {block_field} point={} fraction={fraction:.6} variant={} subset={} unflushed_writes={} confirmed_end={} started_end={} damage={}",
        header.arm.label(),
        item.item_number,
        item.point,
        item.variant.label(),
        item.subset_index,
        selection.unflushed_entries.len(),
        item.confirmed_end,
        item.started_end,
        item.damage.label()
    );
    let image_path = Path::new(GUEST_IMAGE_DIRECTORY).join("disk.img");
    let image_built = File::create(&image_path)
        .and_then(|file| file.set_len(header.disk_bytes).map(|()| file))
        .map_err(|error| error.to_string())
        .and_then(|file| {
            let mut image = ReplayImage::OnDisk(file);
            apply_replayed_writes(log, &selection.writes, &mut image)
        });
    if let Err(detail) = image_built {
        let _cleanup = fs::remove_file(&image_path);
        return format!("{prefix} mount=not_attempted check=image_failed detail={}", sanitized(&detail));
    }
    let offset_text = (FIRST_PARTITION_START_SECTOR * sector_bytes()).to_string();
    let image_text = image_path.to_string_lossy().to_string();
    if let Err(detail) = run_tool(GUEST_BUSYBOX, &["losetup", "-o", &offset_text, GUEST_LOOP_DEVICE, &image_text], None) {
        let _cleanup = fs::remove_file(&image_path);
        return format!("{prefix} mount=not_attempted check=loop_failed detail={}", sanitized(&detail));
    }
    let line = match run_tool(GUEST_BUSYBOX, &["mount", "-t", "ext4", GUEST_LOOP_DEVICE, GUEST_MOUNT_POINT], None) {
        Err(detail) => format!("{prefix} mount=failed check=not_run detail={} dmesg_tail={}", sanitized(&detail), dmesg_tail()),
        Ok(_text) => {
            let library = Path::new(GUEST_MOUNT_POINT).join(GUEST_LIBRARY_NAME);
            let child_arguments: Vec<String> = [
                "--role",
                "s4-check-point",
                "--arm",
                &header.arm.index().to_string(),
                "--library",
                &library.to_string_lossy(),
                "--blocks",
                &header.block_count.to_string(),
                "--states-per-block",
                &header.states_per_block.to_string(),
                "--confirmed-end",
                &item.confirmed_end.to_string(),
                "--started-end",
                &item.started_end.to_string(),
                "--damage",
                item.damage.label(),
            ]
            .iter()
            .map(|text| text.to_string())
            .collect();
            let check_fields = match run_check_child_with_timeout(&child_arguments, Duration::from_secs(CHECK_TIMEOUT_SECONDS)) {
                CheckChildOutcome::Finished { report_line: Some(report_line), .. } => {
                    format!("check=done {}", report_line.trim_start_matches("S4CHECK "))
                }
                CheckChildOutcome::Finished { report_line: None, exit_code } => format!("check=crashed exit_code={exit_code:?}"),
                CheckChildOutcome::TimedOut => "check=timeout q4f=1".to_string(),
                CheckChildOutcome::SpawnFailed(detail) => format!("check=spawn_failed detail={}", sanitized(&detail)),
            };
            let unmounted = run_tool(GUEST_BUSYBOX, &["umount", GUEST_MOUNT_POINT], None).is_ok();
            format!("{prefix} mount=ok {check_fields} unmounted={unmounted}")
        }
    };
    let _detached = run_tool(GUEST_BUSYBOX, &["losetup", "-d", GUEST_LOOP_DEVICE], None);
    let _removed = fs::remove_file(&image_path);
    line
}

// ============================== 来宾：核对子进程 ==============================

fn check_report_fields(report: &VerificationReport) -> String {
    let first_blocks = |blocks: &[u64]| -> String {
        if blocks.is_empty() {
            "-".to_string()
        } else {
            blocks.iter().take(8).map(u64::to_string).collect::<Vec<String>>().join(",")
        }
    };
    format!(
        "q4a={} q4b={} q4c={} q4d={} q4e={} q4f={} opened_after_repair={} open_microseconds={} q4a_blocks={} q4b_blocks={} q4c_blocks={} q4d_blocks={} enumerated_keys={} enumeration_error={} first_read_error={} open_error={}",
        report.lost_confirmed.len(),
        report.silently_corrupted.len(),
        report.confirmed_read_errors.len(),
        report.unconfirmed_read_errors.len(),
        report.phantom_keys.len(),
        u8::from(report.open_failed),
        match report.opened_after_repair {
            None => "not_applicable",
            Some(true) => "yes",
            Some(false) => "no",
        },
        report.open_microseconds,
        first_blocks(&report.lost_confirmed),
        first_blocks(&report.silently_corrupted),
        first_blocks(&report.confirmed_read_errors),
        first_blocks(&report.unconfirmed_read_errors),
        report.enumerated_key_count,
        sanitized(&report.enumeration_error),
        sanitized(&report.first_read_error),
        sanitized(&report.open_error)
    )
}

pub(super) fn run_check_point_role(arguments: &RoleArguments) -> i32 {
    let parsed = (|| {
        let arm = PowerCutArm::from_index(u32::try_from(arguments.number("arm")?).ok()?)?;
        let candidate = arm.candidate()?;
        let plan = VerificationPlan {
            states_per_block: arguments.number("states-per-block")?,
            block_count: arguments.number("blocks")?,
            confirmed_end: arguments.number("confirmed-end")?,
            started_end: arguments.number("started-end")?,
        };
        Some((candidate, PathBuf::from(arguments.text("library")?), plan, LibraryDamage::parse(arguments.text("damage")?)?))
    })();
    let Some((candidate, library, plan, damage)) = parsed else {
        eprintln!("  ✗ s4-check-point 的参数不齐");
        return USAGE_EXIT_CODE;
    };
    let damage_outcome = apply_library_damage(candidate, &library, damage, plan.states_per_block);
    let report = verify_library(candidate, &library, &plan);
    println!(
        "S4CHECK {} damage_applied={} damage_error={}",
        check_report_fields(&report),
        damage_outcome.is_ok(),
        sanitized(&damage_outcome.err().unwrap_or_default())
    );
    0
}
