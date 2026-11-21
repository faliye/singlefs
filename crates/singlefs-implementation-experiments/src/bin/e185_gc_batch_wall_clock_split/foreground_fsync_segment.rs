use super::*;

// ───────────────────────────── 第四段 foreground-fsync（附带，登记第五节分段表第四段；只在 Q16 > 10% 时跑） ─────────────────────────────

/// 前台一次写的字节。
const FOREGROUND_WRITE_BYTES: u64 = 4096;
/// 前台写的镜像文件多大（写在里面轮着走）。
const FOREGROUND_IMAGE_BYTES: u64 = 64 << 20;
/// 每轮默认几秒（登记第五节：各 5 轮、每轮 60 秒）。
pub(super) const FOREGROUND_SECONDS_DEFAULT: u64 = 60;

/// 前台循环「4 KiB `O_DIRECT` 写 + `fdatasync`」到 `deadline`，交回每次的延迟（纳秒）。
fn foreground_write_latencies(path: &Path, deadline: Instant) -> Result<Vec<f64>, String> {
    let mut device = DirectInputOutputBlockDevice::open_existing(
        path,
        PageCachePolicy::BypassWithDirectInputOutput,
        PhysicalBlockSizeSource::Declared(PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES)),
    )
    .map_err(|error| field_token(&format!("open_failed:{error:?}")))?;
    let block: Vec<u8> = (0..FOREGROUND_WRITE_BYTES)
        .map(|position| u8::try_from(position % 241).expect("小于 241"))
        .collect();
    let mut latencies = Vec::new();
    let mut write_index = 0_u64;
    // 迭代上界：挂钟到 `deadline` 就停。
    while Instant::now() < deadline {
        let offset = (write_index % (FOREGROUND_IMAGE_BYTES / FOREGROUND_WRITE_BYTES))
            * FOREGROUND_WRITE_BYTES;
        let started = Instant::now();
        device
            .write_at(DeviceOffsetInBytes(offset), &block, WriteDurability::Plain)
            .map_err(|error| field_token(&format!("write_failed:{error:?}")))?;
        device
            .barrier()
            .map_err(|error| field_token(&format!("barrier_failed:{error:?}")))?;
        latencies.push(started.elapsed().as_nanos() as f64);
        write_index += 1;
    }
    Ok(latencies)
}

/// GC 那条线程一轮做了什么：调了几次、发了几次、搬了几批、在挂钟到之前停没停（停了之后这一轮余下的时间 GC 不在跑）。
struct BackgroundCompaction {
    calls: u64,
    publishes: u64,
    batches: u64,
    stopped_before_the_deadline_after_seconds: Option<f64>,
}

/// 在真盘工作件上可写挂载、等前台就位，然后一次一次调 `run_compaction` 到 `seconds` 秒。
fn background_compaction(
    images: &mut RealDiskImages,
    switches: CompactionSwitches,
    start_together: &std::sync::Barrier,
    seconds: u64,
) -> Result<BackgroundCompaction, String> {
    let restored = images.restore_and_check();
    let ledger = Rc::new(RefCell::new(CallLedger::new(WriteInjection::Nothing)));
    let opened = restored.and_then(|_chunks| images.open_timed(&ledger));
    let mounted = opened.and_then(|mut devices| {
        mount_writable_with_compaction_switches(
            &experiment_pool::parameters(),
            &mut devices,
            switches,
        )
        .map(|mounted| (devices, mounted))
        .map_err(|error| field_token(&format!("mount_failed:{error:?}")))
    });
    start_together.wait();
    let (mut devices, mounted) = mounted?;
    let (_mount_output, mut session) = MountedSession::of_the_mount(mounted);
    let started = Instant::now();
    let deadline = started + Duration::from_secs(seconds);
    let mut background = BackgroundCompaction {
        calls: 0,
        publishes: 0,
        batches: 0,
        stopped_before_the_deadline_after_seconds: None,
    };
    // 迭代上界：挂钟到 `deadline` 就停；整理停了（没有可选的段、没有产出）也停。
    while Instant::now() < deadline {
        session.compaction.publishes_since_the_last_user_change =
            PUBLISHES_ALREADY_PASSED_LEAVING_A_BUDGET_OF_ONE;
        let Ok(run) = session.run_compaction(&mut devices) else {
            background.stopped_before_the_deadline_after_seconds =
                Some(started.elapsed().as_secs_f64());
            break;
        };
        background.calls += 1;
        background.publishes += u64::try_from(run.versions_published.len()).expect("发布次数");
        background.batches += run.batches_published;
        if run.versions_published.is_empty() {
            background.stopped_before_the_deadline_after_seconds =
                Some(started.elapsed().as_secs_f64());
            break;
        }
    }
    drop(devices);
    images.note_the_changed_chunks(&ledger);
    Ok(background)
}

pub(super) fn run_the_foreground_fsync_segment(
    image_directory: &Path,
    point: SamplingPoint,
    batch_units: u64,
    seconds: u64,
) -> i32 {
    let mut emitter = Emitter::new();
    emitter.line(&format!(
        "name=configuration segment=answer_before_the_next_batch {} batch_units={batch_units} seconds_per_round={seconds} rounds={ROUNDS} foreground_write_bytes={FOREGROUND_WRITE_BYTES}",
        point.label()
    ));
    if !check_the_thread_cpu_clock_resolution(&mut emitter) {
        return stop_for_the_clock_resolution(&mut emitter, "answer_before_the_next_batch");
    }
    let mismatches = check_the_constants_and_the_anchors(&mut emitter);
    if mismatches > 0 {
        emitter.line(&format!(
            "name=verdict segment=answer_before_the_next_batch constant_or_anchor_mismatches={mismatches}"
        ));
        emitter.finish();
        return EXIT_CODE_STOP_CLAUSE;
    }
    // 第四段只有一个点、一份池，准备照串行做（不起准备线程）。
    let prepared = prepare_the_sampling_point(point);
    for line in &prepared.lines {
        emitter.line(line);
    }
    let switches = measured_switches(batch_units, prepared.choice.forced_region());
    let mut images = match RealDiskImages::create(image_directory, point, &prepared.pool) {
        Ok(images) => images,
        Err(error) => {
            emitter.line(&format!(
                "name=verdict segment=answer_before_the_next_batch status=not_run reason={}",
                field_token(&error.to_string())
            ));
            emitter.finish();
            return EXIT_CODE_NOT_RUN;
        }
    };
    drop(prepared);
    let foreground_path = image_directory.join("e185-foreground.img");
    if foreground_path.exists() {
        std::fs::remove_file(&foreground_path).ok();
    }
    if let Err(error) = create_a_zero_filled_image(&foreground_path, FOREGROUND_IMAGE_BYTES) {
        emitter.line(&format!(
            "name=verdict segment=answer_before_the_next_batch status=not_run reason={}",
            field_token(&error.to_string())
        ));
        emitter.finish();
        return EXIT_CODE_NOT_RUN;
    }
    let mut judgements = Vec::new();
    for round in 1..=ROUNDS {
        wait_until_no_other_measurement_runs(&mut emitter);
        let quiet = foreground_write_latencies(
            &foreground_path,
            Instant::now() + Duration::from_secs(seconds),
        );
        let start_together = std::sync::Barrier::new(2);
        let (concurrent, background) = std::thread::scope(|scope| {
            let compaction = scope
                .spawn(|| background_compaction(&mut images, switches, &start_together, seconds));
            start_together.wait();
            let latencies = foreground_write_latencies(
                &foreground_path,
                Instant::now() + Duration::from_secs(seconds),
            );
            (
                latencies,
                compaction.join().expect("GC 那条线程跑完、没 panic"),
            )
        });
        match (quiet, concurrent, background) {
            (Ok(quiet_latencies), Ok(concurrent_latencies), Ok(background_run)) => {
                let quiet_percentile_99 = percentile_of(&quiet_latencies, 99);
                let concurrent_percentile_99 = percentile_of(&concurrent_latencies, 99);
                let ratio = match (quiet_percentile_99, concurrent_percentile_99) {
                    (Some(quiet_value), Some(concurrent_value)) => {
                        Some(concurrent_value / quiet_value)
                    }
                    (None, _) | (_, None) => None,
                };
                let judgement =
                    ratio.map_or("not_run", judgement_of_the_foreground_fsync_latency_ratio);
                judgements.push(judgement);
                emitter.line(&format!(
                    "name=q17_round round={round} quiet_writes={} quiet_percentile_99_nanoseconds={} concurrent_writes={} concurrent_percentile_99_nanoseconds={} ratio={} judgement={judgement} gc_calls={} gc_publishes={} gc_batches={} gc_stopped_after_seconds={}",
                    quiet_latencies.len(),
                    optional_number(quiet_percentile_99),
                    concurrent_latencies.len(),
                    optional_number(concurrent_percentile_99),
                    optional_number(ratio),
                    background_run.calls,
                    background_run.publishes,
                    background_run.batches,
                    optional_number(background_run.stopped_before_the_deadline_after_seconds),
                ));
            }
            (quiet_result, concurrent_result, background_result) => {
                emitter.line(&format!(
                    "name=q17_round round={round} valid=false quiet={} concurrent={} background={}",
                    quiet_result.err().unwrap_or_else(|| "ok".to_string()),
                    concurrent_result.err().unwrap_or_else(|| "ok".to_string()),
                    background_result.err().unwrap_or_else(|| "ok".to_string()),
                ));
            }
        }
    }
    std::fs::remove_file(&foreground_path).ok();
    let removal = images.remove();
    let foreground_fsync_latency_judgement = judgement_of_the_rounds(&judgements);
    emitter.line(&format!(
        "name=verdict segment=answer_before_the_next_batch constant_or_anchor_mismatches=0 valid_rounds={} foreground_fsync_latency_judgement_per_round={} foreground_fsync_latency_judgement={foreground_fsync_latency_judgement} images_removed={}",
        judgements.len(),
        judgements.join(","),
        removal.is_ok()
    ));
    emitter.finish();
    EXIT_CODE_OK
}
