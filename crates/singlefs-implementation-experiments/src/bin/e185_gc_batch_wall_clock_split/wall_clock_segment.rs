use super::*;

// ───────────────────────────── 第一段 wall-clock：测量 ─────────────────────────────

/// 存下来的一条有效意图（汇总用）。
pub(super) struct StoredIntent {
    pub(super) arm: DeviceArm,
    pub(super) point: SamplingPoint,
    pub(super) batch_units: u64,
    pub(super) round: usize,
    pub(super) injection: WriteInjection,
    pub(super) is_the_geometry_formed: bool,
    pub(super) calls: Vec<CallReading>,
}

impl StoredIntent {
    pub(super) fn batch_calls(&self) -> impl Iterator<Item = &CallReading> {
        self.calls
            .iter()
            .filter(|call| counts_as_a_batch(call.kind))
    }

    pub(super) fn batch_cpu_nanoseconds(&self) -> f64 {
        self.batch_calls()
            .map(|call| call.cpu_nanoseconds as f64)
            .sum()
    }

    pub(super) fn batch_wait_nanoseconds(&self) -> f64 {
        self.batch_calls()
            .map(|call| call.wait_nanoseconds() as f64)
            .sum()
    }

    pub(super) fn batch_wall_nanoseconds(&self) -> f64 {
        self.batch_calls()
            .map(|call| call.wall_nanoseconds as f64)
            .sum()
    }

    pub(super) fn batch_write_bytes(&self) -> u64 {
        self.batch_calls().map(|call| call.write_bytes).sum()
    }

    pub(super) fn batch_relocation_copy_bytes(&self) -> u64 {
        self.batch_calls()
            .map(|call| call.relocation_copy_bytes)
            .sum()
    }

    pub(super) fn first_batch_wall_nanoseconds(&self) -> Option<f64> {
        self.batch_calls()
            .next()
            .map(|call| call.wall_nanoseconds as f64)
    }

    /// 整条意图（签发与各批）的 CPU、等盘与写调用次数：阳性对照的注入在每次 `write_at` 里，按整条意图比。
    fn intent_cpu_nanoseconds(&self) -> f64 {
        self.calls
            .iter()
            .map(|call| call.cpu_nanoseconds as f64)
            .sum()
    }

    fn intent_wait_nanoseconds(&self) -> f64 {
        self.calls
            .iter()
            .map(|call| call.wait_nanoseconds() as f64)
            .sum()
    }

    fn intent_write_calls(&self) -> f64 {
        self.calls
            .iter()
            .map(|call| {
                call.per_method
                    .get(&DeviceMethod::Write)
                    .map_or(0, |totals| totals.calls) as f64
            })
            .sum()
    }

    /// 整条意图里写调用（`write_at`，阳性对照注入所在的那一种调用）的等盘之和（登记第十二节 12.9）。
    fn intent_write_wait_nanoseconds(&self) -> f64 {
        self.calls
            .iter()
            .map(|call| {
                call.per_method
                    .get(&DeviceMethod::Write)
                    .map_or(0, |totals| totals.wait_nanoseconds) as f64
            })
            .sum()
    }

    pub(super) fn row_one_control_round(&self) -> RowOneControlRound {
        RowOneControlRound {
            round: self.round,
            cpu_nanoseconds: self.intent_cpu_nanoseconds(),
            wait_nanoseconds: self.intent_wait_nanoseconds(),
            write_calls: self.intent_write_calls(),
            write_wait_nanoseconds: self.intent_write_wait_nanoseconds(),
        }
    }
}

/// 一次尝试在两种作废规则下的样子（第 2 次重跑登记第六节 N14）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct AttemptVoiding {
    /// 照原登记第十一节（不加 R2-1）这一次有效。
    pub(super) is_valid_under_the_original_voiding: bool,
    /// 这一次被校准（R2-1 的比、校准读不到、校准锚点不符）作废。
    pub(super) is_voided_by_the_calibration: bool,
}

/// 照原登记第十一节（不加 R2-1）会被收下的那一次（N14）：头一个照原规则有效的——只因校准作废的那一次照原规则算有效。
fn attempt_kept_under_the_original_voiding(attempts: &[AttemptVoiding]) -> Option<usize> {
    attempts
        .iter()
        .position(|attempt| attempt.is_valid_under_the_original_voiding)
}

/// N14：照原作废规则收下的那一次（[`attempt_kept_under_the_original_voiding`]）是被校准作废的那一次时打一行
/// `name=kept_under_the_original_voiding`，点名这一轮 R2-1 换掉了哪一次（只报）；交回收下的那一次的下标。
pub(super) fn kept_attempt_under_the_original_voiding(
    emitter: &mut Emitter,
    label: &MeasurementLabel,
    voidings: &[AttemptVoiding],
) -> Option<usize> {
    let kept_index = attempt_kept_under_the_original_voiding(voidings)?;
    if voidings[kept_index].is_voided_by_the_calibration {
        emitter.line(&format!(
            "name=kept_under_the_original_voiding {} kept_attempt={} voided_by_the_calibration=true",
            label.text(),
            kept_index + 1
        ));
    }
    Some(kept_index)
}

/// 存下的有效意图两份：照这一趟的作废规则（加了 R2-1，Q1–Q17 的判定只读这一份）与照原作废规则（N14 并列报，只报不判）。
pub(super) struct StoredIntents {
    pub(super) under_the_registered_voiding: Vec<StoredIntent>,
    pub(super) under_the_original_voiding: Vec<StoredIntent>,
}

fn stored_intent_of(
    label: &MeasurementLabel,
    is_the_geometry_formed: bool,
    calls: Vec<CallReading>,
) -> StoredIntent {
    StoredIntent {
        arm: label.arm,
        point: label.point,
        batch_units: label.batch_units,
        round: label.round,
        injection: label.injection,
        is_the_geometry_formed,
        calls,
    }
}

/// 一个取样点、一个设备、一个批大小、一轮（或对照的一轮）：照登记重跑至多三次，作废的照打 `valid=false`，第一份有效的存下；
/// 交回停机条款触发了的那几条。每一次前后做校准与读数（[`SegmentCalibration::around_the_intent`]），校准作废的照 R2-1 作废；
/// 作废之后要重量的，先照 R2-2 等常态（[`SegmentCalibration::wait_for_the_machine_before_the_retry`]）。照原作废规则会收下的那一次另存一份（N14）。
fn measure_with_retries(
    emitter: &mut Emitter,
    stored: &mut StoredIntents,
    images: Option<&mut RealDiskImages>,
    label: MeasurementLabel,
    is_the_geometry_formed: bool,
    pool_of_the_round: Option<&ConstructedPool>,
    calibration: &mut SegmentCalibration,
) -> Vec<String> {
    let switches = measured_switches(label.batch_units, label.choice.forced_region());
    let data_units =
        is_the_geometry_formed.then_some(label.point.fraction.data_units_per_segment());
    let mut images = images;
    let mut voidings: Vec<AttemptVoiding> = Vec::new();
    let mut calls_of_each_attempt: Vec<Option<Vec<CallReading>>> = Vec::new();
    let mut is_stored = false;
    for attempt in 1..=ATTEMPTS_PER_ROUND_AT_MOST {
        enter_the_timing_window(TimingWindow::WallClockCompactionRun);
        let mut attempt_label = label;
        attempt_label.attempt = attempt;
        let (reading, intent_calibration) = calibration.around_the_intent(|| {
            match (label.arm, pool_of_the_round, images.as_deref_mut()) {
                (DeviceArm::Memory, Some(pool), _) => Ok(measure_on_the_memory_device(
                    pool,
                    switches,
                    label.injection,
                    ReplayCapture::Off,
                )),
                (DeviceArm::RealDisk, _, Some(real_disk_images)) => {
                    measure_on_the_real_disk(real_disk_images, switches, label.injection)
                }
                (DeviceArm::Memory, None, _) | (DeviceArm::RealDisk, _, None) => {
                    Err("no_device_for_this_arm".to_string())
                }
            }
        });
        calibration.note_the_intent(emitter, &attempt_label, &intent_calibration);
        let calibration_invalid_reason = intent_calibration.verdict.invalid_reason();
        let reading = match reading {
            Ok(reading) => reading,
            Err(reason) => {
                emitter.line(&format!(
                    "name=intent {} valid=false invalid_reason={reason}",
                    attempt_label.text()
                ));
                voidings.push(AttemptVoiding {
                    is_valid_under_the_original_voiding: false,
                    is_voided_by_the_calibration: calibration_invalid_reason.is_some(),
                });
                calls_of_each_attempt.push(None);
                continue;
            }
        };
        let original_invalid_reason = invalid_reason_of(&reading);
        let invalid_reason = original_invalid_reason.or(calibration_invalid_reason);
        emit_the_intent(emitter, &attempt_label, &reading, invalid_reason);
        let findings = stop_findings_of(&reading, data_units, label.batch_units);
        for finding in &findings {
            emitter.line(&format!(
                "name=stop_clause {} finding={finding}",
                attempt_label.text()
            ));
        }
        if !findings.is_empty() {
            return findings;
        }
        voidings.push(AttemptVoiding {
            is_valid_under_the_original_voiding: original_invalid_reason.is_none(),
            is_voided_by_the_calibration: calibration_invalid_reason.is_some(),
        });
        match invalid_reason {
            None => {
                stored.under_the_registered_voiding.push(stored_intent_of(
                    &label,
                    is_the_geometry_formed,
                    reading.calls.clone(),
                ));
                calls_of_each_attempt.push(Some(reading.calls));
                is_stored = true;
                break;
            }
            Some(reason) => {
                calls_of_each_attempt.push(Some(reading.calls));
                if attempt < ATTEMPTS_PER_ROUND_AT_MOST {
                    calibration.wait_for_the_machine_before_the_retry(
                        emitter,
                        &attempt_label,
                        reason,
                    );
                }
            }
        }
    }
    if !is_stored {
        emitter.line(&format!(
            "name=round_invalid {} attempts={ATTEMPTS_PER_ROUND_AT_MOST}",
            label.text()
        ));
    }
    if let Some(Some(calls)) = kept_attempt_under_the_original_voiding(emitter, &label, &voidings)
        .and_then(|kept_index| calls_of_each_attempt.get_mut(kept_index))
        .map(Option::take)
    {
        stored.under_the_original_voiding.push(stored_intent_of(
            &label,
            is_the_geometry_formed,
            calls,
        ));
    }
    Vec::new()
}

/// 每块盘的分块摘要（内存盘）。
pub(super) fn digests_of_the_pool(pool: &ConstructedPool) -> Vec<ChunkDigest> {
    pool.devices
        .iter()
        .map(|(_identity, device)| digest_of_the_memory_device(device))
        .collect()
}

/// 一轮里第一行阳性对照三种注入的次序（登记第十二节 12.9）：基线、盘臂对照、CPU 臂对照每轮各一次，次序按轮号轮换。
/// 基线 5 轮先跑完、对照后跑的那一趟（产物 `e185-gc-batch-wall-clock-split-wall-clock.out`），时段的差别整个算进了「对照 − 基线」；交错之后每一轮的三种注入挨着量。
fn row_one_control_injections_of_the_round(round: usize) -> [WriteInjection; 3] {
    let in_registration_order = [
        WriteInjection::Nothing,
        WriteInjection::SleepTwoMilliseconds,
        WriteInjection::SpinTwoMillisecondsOfThreadCpu,
    ];
    let shift = round.checked_sub(1).expect("轮号从 1 起") % in_registration_order.len();
    let mut order = in_registration_order;
    order.rotate_left(shift);
    order
}

/// 一个取样点一轮里要量的（批大小、设备臂、注入），按量的次序。对照点 K = 8 那一格，每个设备臂挨着量三种注入
/// （[`row_one_control_injections_of_the_round`] 的次序）；别的格只量基线。
fn measurements_of_the_round(
    point: SamplingPoint,
    round: usize,
) -> Vec<(u64, DeviceArm, WriteInjection)> {
    let mut plan = Vec::new();
    for batch_units in BATCH_SIZES {
        let is_the_control_cell = point == CONTROL_POINT && batch_units == CONTROL_BATCH_UNITS;
        for arm in DeviceArm::ALL {
            if is_the_control_cell {
                for injection in row_one_control_injections_of_the_round(round) {
                    plan.push((batch_units, arm, injection));
                }
            } else {
                plan.push((batch_units, arm, WriteInjection::Nothing));
            }
        }
    }
    plan
}

/// 一个取样点造好的池那一行 `name=pool`（第一段、第二段与 `geometry-check` 同一个格式；占比 0.75 的核比的是它的 `digests=`）。
pub(super) fn pool_line(prepared: &PreparedPoint, speculative_rebuilds_voided: usize) -> String {
    format!(
        "name=pool {} choice={} geometry_formed={} speculative_rebuilds_voided={speculative_rebuilds_voided} digests={}",
        prepared.recipe.point.label(),
        prepared.choice.label(),
        prepared.is_the_geometry_formed,
        prepared
            .reference_digests
            .iter()
            .map(hexadecimal_of)
            .collect::<Vec<_>>()
            .join(",")
    )
}

/// 第一段一个取样点：从准备队列取它的真盘镜像（镜像在量这一点之前建，连同造池与几何核；还没好就跑一次准备，[`PreparationQueue`]），
/// 打准备时攥着的行，再串行量 5 轮 × 两臂 × 三个批大小；是对照点的，每一轮在 K = 8 那一格里另跑第一行两条臂的阳性对照
/// （与基线交错，[`measurements_of_the_round`]）。第 2 轮起的重造池从队列里取，还没造好就在那一轮开头跑一次准备（量的时候不造）。
/// 量完放掉这一点的池与镜像。交回停机条款的触发。准备完、量之前打一行机器状态（N10）。
fn measure_one_sampling_point(
    emitter: &mut Emitter,
    stored: &mut StoredIntents,
    queue: &mut PreparationQueue<'_>,
    point_index: usize,
    calibration: &mut SegmentCalibration,
) -> Vec<String> {
    let real_disk_images_or_reason = queue.take_the_real_disk_images(point_index, emitter);
    let prepared = queue.prepared_point(point_index, emitter);
    let point = prepared.recipe.point;
    for line in &prepared.lines {
        emitter.line(line);
    }
    emitter.line(&pool_line(
        &prepared,
        queue.points[point_index].speculative_rebuilds_voided,
    ));
    emit_the_machine_state(
        emitter,
        calibration.segment,
        MachineStatePosition::PointStart(point),
        calibration.started,
    );
    let mut real_disk_images = match real_disk_images_or_reason {
        Ok(images) => Some(images),
        Err(reason) => {
            emitter.line(&format!(
                "name=real_disk_images {} created=false reason={reason}",
                point.label(),
            ));
            None
        }
    };
    let mut findings = Vec::new();
    let mut rebuilt_pool_of_the_round: Option<ConstructedPool> = None;
    for round in 1..=ROUNDS {
        wait_until_no_other_measurement_runs(emitter);
        if round > 1 {
            let rebuilt = queue.take_the_rebuilt_pool(point_index, round, emitter);
            let is_identical = rebuilt.digests == prepared.reference_digests;
            emitter.line(&format!(
                "name=memory_rebuild {} round={round} construction_seconds={:.3} digests_identical={is_identical}",
                point.label(),
                rebuilt.pool.construction_seconds
            ));
            rebuilt_pool_of_the_round = is_identical.then_some(rebuilt.pool);
        }
        let pool_of_the_round = if round == 1 {
            Some(&prepared.pool)
        } else {
            rebuilt_pool_of_the_round.as_ref()
        };
        for (batch_units, arm, injection) in measurements_of_the_round(point, round) {
            let label = MeasurementLabel {
                arm,
                point,
                batch_units,
                round,
                attempt: 0,
                injection,
                choice: prepared.choice,
            };
            findings.extend(measure_with_retries(
                emitter,
                stored,
                real_disk_images.as_mut(),
                label,
                prepared.is_the_geometry_formed,
                pool_of_the_round,
                calibration,
            ));
            if !findings.is_empty() {
                return findings;
            }
        }
    }
    if let Some(images) = real_disk_images {
        if let Err(error) = images.remove() {
            emitter.line(&format!(
                "name=real_disk_images {} removed=false reason={}",
                point.label(),
                field_token(&error.to_string())
            ));
        }
    }
    queue.finish_the_point(point_index);
    findings
}

/// 设备顺序写带宽的一轮读数（字节 / 秒）：`O_DIRECT` 每次 4 MiB 写满 1 GiB、末尾一次 `fdatasync`；走页缓存同样写、不同步（第四行对照）。
const SEQUENTIAL_WRITE_BYTES: u64 = 1 << 30;

fn sequential_write_bytes_per_second(path: &Path, policy: PageCachePolicy) -> Result<f64, String> {
    let mut device = DirectInputOutputBlockDevice::open_existing(
        path,
        policy,
        PhysicalBlockSizeSource::Declared(PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES)),
    )
    .map_err(|error| field_token(&format!("open_failed:{error:?}")))?;
    let pattern: Vec<u8> = (0..IMAGE_CHUNK_BYTES)
        .map(|position| u8::try_from(position % 251).expect("小于 251"))
        .collect();
    let started = Instant::now();
    for chunk in 0..SEQUENTIAL_WRITE_BYTES / IMAGE_CHUNK_BYTES {
        device
            .write_at(
                DeviceOffsetInBytes(chunk * IMAGE_CHUNK_BYTES),
                &pattern,
                WriteDurability::Plain,
            )
            .map_err(|error| field_token(&format!("write_failed:{error:?}")))?;
    }
    let is_synced = match policy {
        PageCachePolicy::BypassWithDirectInputOutput => {
            device
                .barrier()
                .map_err(|error| field_token(&format!("barrier_failed:{error:?}")))?;
            true
        }
        PageCachePolicy::GoThroughPageCache => false,
    };
    let seconds = started.elapsed().as_secs_f64();
    if !is_synced {
        // 走页缓存那一臂的脏页在计时之外落盘，免得拖到下一轮的 `O_DIRECT` 计时里
        device
            .barrier()
            .map_err(|error| field_token(&format!("barrier_failed:{error:?}")))?;
    }
    Ok(SEQUENTIAL_WRITE_BYTES as f64 / seconds)
}

/// Q15 与第四行对照：各 5 轮，交回（`O_DIRECT` 带同步的中位、走页缓存不同步的中位）。
fn measure_the_device_sequential_write(
    emitter: &mut Emitter,
    image_directory: &Path,
) -> Option<(f64, f64)> {
    let path = image_directory.join("e185-sequential-write.img");
    if path.exists() {
        std::fs::remove_file(&path).ok()?;
    }
    if let Err(error) = create_a_zero_filled_image(&path, SEQUENTIAL_WRITE_BYTES) {
        emitter.line(&format!(
            "name=q15 created=false reason={}",
            field_token(&error.to_string())
        ));
        return None;
    }
    let mut direct = Vec::new();
    let mut page_cache = Vec::new();
    for round in 1..=ROUNDS {
        wait_until_no_other_measurement_runs(emitter);
        enter_the_timing_window(TimingWindow::DeviceSequentialWrite);
        for (policy, label, readings) in [
            (
                PageCachePolicy::BypassWithDirectInputOutput,
                "direct_with_fdatasync",
                &mut direct,
            ),
            (
                PageCachePolicy::GoThroughPageCache,
                "page_cache_without_sync",
                &mut page_cache,
            ),
        ] {
            match sequential_write_bytes_per_second(&path, policy) {
                Ok(bytes_per_second) => {
                    emitter.line(&format!("name=q15_round round={round} write_path={label} bytes_per_second={bytes_per_second:.1}"));
                    readings.push(bytes_per_second);
                }
                Err(reason) => emitter.line(&format!(
                    "name=q15_round round={round} write_path={label} valid=false reason={reason}"
                )),
            }
        }
    }
    std::fs::remove_file(&path).ok();
    Some((median_of(&direct)?, median_of(&page_cache)?))
}

// ───────────────────────────── 第一段 wall-clock：入口 ─────────────────────────────

/// 开机会清空的目录：Ubuntu 的 systemd-tmpfiles（`tmp.conf` 里 `D /tmp`）开机清 `/tmp`，不论它挂在什么文件系统上。
const DIRECTORIES_CLEARED_AT_BOOT: [&str; 1] = ["/tmp"];
/// 开机会清空的文件系统种类：内容只在内存里。
const FILESYSTEM_TYPES_CLEARED_AT_BOOT: [&str; 2] = ["tmpfs", "ramfs"];

/// `st_dev` 拆成主、次设备号（glibc `gnu_dev_major` / `gnu_dev_minor` 的编码），与 `/proc/self/mountinfo` 第三栏「主:次」同一口径。
fn major_and_minor_of_the_device_number(device_number: u64) -> (u64, u64) {
    let major = ((device_number >> 8) & 0xfff) | ((device_number >> 32) & 0xffff_f000);
    let minor = (device_number & 0xff) | ((device_number >> 12) & 0xffff_ff00);
    (major, minor)
}

/// 在 `/proc/self/mountinfo` 的正文里找「主:次」等于这个设备号的那一行，交回它的文件系统种类（` - ` 之后第一栏）；找不到交回 `None`。
fn filesystem_type_of_the_device(device_number: u64, mount_information: &str) -> Option<String> {
    let (major, minor) = major_and_minor_of_the_device_number(device_number);
    let wanted = format!("{major}:{minor}");
    mount_information.lines().find_map(|line| {
        let (mount_fields, filesystem_fields) = line.split_once(" - ")?;
        if mount_fields.split(' ').nth(2)? != wanted {
            return None;
        }
        filesystem_fields.split(' ').next().map(str::to_string)
    })
}

/// 镜像目录在不在开机清空的地方：文件系统是 tmpfs / ramfs，或真路径落在 `/tmp` 下，任一样成立就是；
/// 两样都读到而都不成立才不是；读不全又没有一样成立交回 `None`。
fn is_cleared_at_boot(
    filesystem_type: Option<&str>,
    canonical_directory: Option<&Path>,
) -> Option<bool> {
    let is_a_type_cleared_at_boot =
        filesystem_type.map(|kind| FILESYSTEM_TYPES_CLEARED_AT_BOOT.contains(&kind));
    let is_under_a_directory_cleared_at_boot = canonical_directory.map(|directory| {
        DIRECTORIES_CLEARED_AT_BOOT
            .iter()
            .any(|cleared| directory.starts_with(cleared))
    });
    match (
        is_a_type_cleared_at_boot,
        is_under_a_directory_cleared_at_boot,
    ) {
        (Some(true), _) | (_, Some(true)) => Some(true),
        (Some(false), Some(false)) => Some(false),
        (Some(false), None) | (None, Some(false)) | (None, None) => None,
    }
}

/// 第一段的配置行。`image_directory` 是相对路径时按当前目录解析（`std::fs` 本来的读法，装置不换当前目录）；
/// 行里不打这个路径（在第二台跑时那是第二台的私有目录名，产物要进仓），只打它所在文件系统的种类、在不在开机清空的地方与设备号。
fn wall_clock_configuration_line(
    image_directory: &Path,
    points: &[SamplingPoint],
    settings: &PreparationSettings,
) -> String {
    let metadata =
        std::fs::create_dir_all(image_directory).and_then(|()| std::fs::metadata(image_directory));
    let filesystem_type = metadata.as_ref().ok().and_then(|directory_metadata| {
        std::fs::read_to_string("/proc/self/mountinfo")
            .ok()
            .and_then(|mount_information| {
                filesystem_type_of_the_device(directory_metadata.dev(), &mount_information)
            })
    });
    let canonical_directory = std::fs::canonicalize(image_directory).ok();
    format!(
        "name=configuration segment=wall_clock points={} batch_sizes={} rounds={ROUNDS} image_directory_filesystem={} image_directory_is_cleared_at_boot={} image_device={} worker_threads={} cpu_clock=thread_cputime cpu_clock_identifier={CLOCK_IDENTIFIER_OF_THE_THREAD_CPU_TIME} {}",
        points.iter().map(|point| format!("{}:{}", point.fraction.label(), point.pool_data_units)).collect::<Vec<_>>().join(","),
        BATCH_SIZES.iter().map(u64::to_string).collect::<Vec<_>>().join(","),
        filesystem_type.as_deref().map_or_else(|| "unreadable".to_string(), field_token),
        is_cleared_at_boot(filesystem_type.as_deref(), canonical_directory.as_deref())
            .map_or_else(|| "unknown".to_string(), |is_cleared| format!("{is_cleared}")),
        metadata.map_or_else(|error| field_token(&error.to_string()), |directory_metadata| directory_metadata.dev().to_string()),
        worker_threads_from_the_environment(),
        preparation_configuration_fields(points, Some(image_directory), settings),
    )
}

pub(super) fn run_the_wall_clock_segment(image_directory: &Path, points: &[SamplingPoint]) -> i32 {
    let segment_started = Instant::now();
    let settings = PreparationSettings::from_the_environment();
    let mut emitter = Emitter::new();
    emitter.line(&wall_clock_configuration_line(
        image_directory,
        points,
        &settings,
    ));
    assert_this_process_runs_one_thread("segment_start");
    if !check_the_apparatus_sha256(&mut emitter) {
        return EXIT_CODE_STOP_CLAUSE;
    }
    if !check_the_thread_cpu_clock_resolution(&mut emitter) {
        return stop_for_the_clock_resolution(&mut emitter, "wall_clock");
    }
    emit_the_machine_state(
        &mut emitter,
        "wall_clock",
        MachineStatePosition::SegmentStart,
        segment_started,
    );
    let mut calibration =
        match SegmentCalibration::start("wall_clock", segment_started, &mut emitter) {
            Ok(calibration) => calibration,
            Err(exit_code) => return exit_code,
        };
    let mismatches = check_the_constants_and_the_anchors(&mut emitter);
    let mut stored = StoredIntents {
        under_the_registered_voiding: Vec::new(),
        under_the_original_voiding: Vec::new(),
    };
    let mut stop_findings: Vec<String> = Vec::new();
    if mismatches == 0 {
        let mut queue = PreparationQueue::new(
            "wall_clock",
            points,
            Some(image_directory),
            &settings,
            &run_the_preparation_work,
        );
        for point_index in 0..points.len() {
            stop_findings.extend(measure_one_sampling_point(
                &mut emitter,
                &mut stored,
                &mut queue,
                point_index,
                &mut calibration,
            ));
            if !stop_findings.is_empty() {
                break;
            }
        }
    }
    let sequential_write = if mismatches == 0 && stop_findings.is_empty() {
        measure_the_device_sequential_write(&mut emitter, image_directory)
    } else {
        None
    };
    let summary = summarize_the_wall_clock_segment(
        &mut emitter,
        &stored.under_the_registered_voiding,
        points,
        sequential_write,
    );
    emit_the_wall_clock_judgements_under_the_original_voiding(&mut emitter, &stored, points);
    calibration.finish(&mut emitter);
    emit_the_machine_state(
        &mut emitter,
        "wall_clock",
        MachineStatePosition::SegmentEnd,
        segment_started,
    );
    emitter.line(&format!(
        "name=verdict segment=wall_clock constant_or_anchor_mismatches={mismatches} stop_clause_findings={} positive_control_failures={} positive_control_failure_items={} discrimination_failures={} clause_wall_clock_not_rising_with_the_batch_triggered={} clause_two_cpu_paths_disagree_points={} gc_to_sequential_write_bandwidth_maximum_ratio={} gc_to_sequential_write_bandwidth_judgement={} stored_intents={}",
        stop_findings.len(),
        summary.positive_control_failures.len(),
        if summary.positive_control_failures.is_empty() { "none".to_string() } else { summary.positive_control_failures.join(",") },
        summary.discrimination_failures,
        summary.clause_wall_clock_not_rising_with_the_batch_triggered,
        summary.clause_two_cpu_paths_disagree_points,
        summary.gc_to_sequential_write_bandwidth.map_or_else(|| "none".to_string(), |(ratio, _judgement)| format!("{ratio:.6}")),
        summary.gc_to_sequential_write_bandwidth.map_or("not_run", |(_ratio, judgement)| judgement),
        stored.under_the_registered_voiding.len(),
    ));
    emitter.finish();
    if mismatches > 0 || !stop_findings.is_empty() {
        EXIT_CODE_STOP_CLAUSE
    } else if !summary.positive_control_failures.is_empty() {
        EXIT_CODE_POSITIVE_CONTROL_FAILED
    } else {
        EXIT_CODE_OK
    }
}

#[cfg(test)]
mod tests;
