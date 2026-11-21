use super::*;

// ───────────────────────────── 阶段计时与饱和吞吐（第二段） ─────────────────────────────

/// 一个纯阶段在一次重放里的读数：每次重复的 CPU 纳秒与挂钟纳秒、重复了几次。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct StageTiming {
    pub(super) cpu_nanoseconds_per_repetition: f64,
    wall_nanoseconds_per_repetition: f64,
    pub(super) repetitions: u64,
}

/// 一次重放：先不计时跑一遍热身（被测那一批里扫描读到的字节刚过缓存，热身让重放的缓存状态接近那一次），再计时跑一遍，交回这一遍的
/// CPU 与挂钟（`repetitions` 是 1）；线程 CPU 时钟读不到交回 `None`。只计时一遍（登记第五节「每个阶段重放 5 次取中位」、阳性对照表
/// 第 2 行「各跑一次」）：线程 CPU 时钟读的那一刻现算到纳秒，单遍的阶段读得出数，不用为了钟的分辨率把同一份输入重复跑很多遍。
/// 挂钟读数包着 CPU 读数，与一次 `run_compaction` 调用那一层同一个嵌套。
fn time_a_pure_stage(mut stage: impl FnMut()) -> Option<StageTiming> {
    stage();
    let started = RawWallClockReading::now();
    let cpu_before = thread_cpu_nanoseconds()?;
    stage();
    let cpu_after = thread_cpu_nanoseconds()?;
    let wall = started.elapsed();
    Some(StageTiming {
        cpu_nanoseconds_per_repetition: cpu_after.saturating_sub(cpu_before) as f64,
        wall_nanoseconds_per_repetition: wall.as_nanos() as f64,
        repetitions: 1,
    })
}

/// 甲段：对每个被区域扫描读到的单元的整单元字节算 CRC-32C；交回算过的字节数与各段 CRC 的异或（不让优化器删掉这段活）。
pub(super) fn crc_stage_over_the_units(units: &[Vec<u8>]) -> (u64, u32) {
    let mut covered_bytes = 0_u64;
    let mut folded = 0_u32;
    for unit in units {
        let covered: &[u8] = unit.as_slice();
        covered_bytes += length_in_bytes(covered);
        folded ^= crc32_castagnoli(std::hint::black_box(covered));
    }
    (covered_bytes, folded)
}

/// 饱和吞吐的一档：合计字节 ÷ 挂钟、起了几条不同的线程、合计重复次数。
#[derive(Clone, Copy, Debug, PartialEq)]
struct SaturationReading {
    bytes_per_second: f64,
    distinct_threads: usize,
    repetitions: u64,
}

/// `thread_count` 条线程各自重复跑同一阶段函数 `duration`（登记第五节「饱和吞吐」）；装置核起了几条不同的线程。
fn saturated_throughput(
    thread_count: usize,
    duration: Duration,
    bytes_per_repetition: u64,
    stage: &(dyn Fn() + Sync),
) -> SaturationReading {
    let repetitions = AtomicU64::new(0);
    let started = Instant::now();
    let thread_identities: Vec<std::thread::ThreadId> = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..thread_count)
            .map(|_worker_index| {
                scope.spawn(|| {
                    // 迭代上界：挂钟到 `duration` 就停；每次重复是一次完整的阶段函数。
                    loop {
                        stage();
                        repetitions.fetch_add(1, Ordering::Relaxed);
                        if started.elapsed() >= duration {
                            break;
                        }
                    }
                    std::thread::current().id()
                })
            })
            .collect();
        workers
            .into_iter()
            .map(|worker| worker.join().expect("饱和吞吐的线程跑完、没 panic"))
            .collect()
    });
    let wall_seconds = started.elapsed().as_secs_f64();
    let total_repetitions = repetitions.load(Ordering::Relaxed);
    let distinct: HashSet<std::thread::ThreadId> = thread_identities.into_iter().collect();
    SaturationReading {
        bytes_per_second: (total_repetitions * bytes_per_repetition) as f64 / wall_seconds,
        distinct_threads: distinct.len(),
        repetitions: total_repetitions,
    }
}

// ───────────────────────────── 第二段 stage-split：阶段重放（登记第五节「阶段拆分」「饱和吞吐」） ─────────────────────────────

/// 每个阶段重放几次取中位（纯函数、内存里：几次只说明没有隐藏状态）。
pub(super) const STAGE_REPLAYS: usize = 5;
/// 饱和吞吐每一档跑多久。
const SATURATION_SECONDS: u64 = 2;
/// D24 已定项 3 的「CPU 16 线程」。
const CPU_THREADS_OF_THE_GPU_OFFLOAD_COMPARISON: usize = 16;
/// 丙段盘大小敏感性：把单元区的分配记录平铺这么多份（4 GiB → 64 GiB）。
const DISK_SIZE_MULTIPLIER: u64 = 16;

/// 一次调用里扫描读到 R 的那几个单元的字节（按记下的读，从重放材料里取）。
pub(super) fn units_read_by_the_scan(material: &CallReplayMaterial) -> Vec<Vec<u8>> {
    material
        .scan_reads_in_the_region
        .iter()
        .filter_map(|read| {
            material
                .unit_bytes_by_slot
                .get(&(read.offset / LOCAL_SLOT_BYTES))
                .cloned()
        })
        .collect()
}

/// 重放时读单元字节：先取材料里记下的，没有的从这一轮的池（调用之前的样子：签发与批不动别处的数据单元）读。
pub(super) fn replay_reader<'material>(
    material: &'material CallReplayMaterial,
    pool_devices: &'material [(DeviceIdentity, SparseBlockDevice)],
) -> impl FnMut(DeviceIdentity, SlotNumber, usize) -> Option<Vec<u8>> + 'material {
    move |device, slot, length| {
        material
            .unit_bytes_by_slot
            .get(&slot.0)
            .filter(|bytes| bytes.len() == length && device == DeviceIdentity(0))
            .cloned()
            .or_else(|| PoolReader::read(pool_devices, device, slot.to_device_offset(), length))
    }
}

/// 一个阶段 5 次重放的中位：（每次重复的 CPU 纳秒中位、挂钟纳秒中位、重复次数中位）；有一次线程 CPU 时钟读不到就交回 `None`（那一次作废）。
pub(super) fn median_stage_timing(mut stage: impl FnMut()) -> Option<StageTiming> {
    let mut timings = Vec::new();
    for _replay in 0..STAGE_REPLAYS {
        timings.push(time_a_pure_stage(&mut stage)?);
    }
    let cpu: Vec<f64> = timings
        .iter()
        .map(|timing| timing.cpu_nanoseconds_per_repetition)
        .collect();
    let wall: Vec<f64> = timings
        .iter()
        .map(|timing| timing.wall_nanoseconds_per_repetition)
        .collect();
    let repetitions: Vec<f64> = timings
        .iter()
        .map(|timing| timing.repetitions as f64)
        .collect();
    let median_repetitions = median_of(&repetitions)?;
    Some(StageTiming {
        cpu_nanoseconds_per_repetition: median_of(&cpu)?,
        wall_nanoseconds_per_repetition: median_of(&wall)?,
        repetitions: timings
            .iter()
            .map(|timing| timing.repetitions)
            .find(|count| *count as f64 >= median_repetitions)
            .unwrap_or(0),
    })
}

/// 一次调用的阶段重放读数。甲段两个子项：`crc` 与 `crc_bytes` 是区域扫描读到的单元（登记第一节「甲段的 CPU 时间与字节」那一项），
/// `release_check_crc` 与 `release_check_crc_bytes` 是释放前读盘核读的那几份副本（第十二节 12.5 补的子项）。
pub(super) struct CallStageReplay {
    pub(super) crc: Option<StageTiming>,
    pub(super) release_check_crc: Option<StageTiming>,
    pub(super) scan: Option<StageTiming>,
    pub(super) choose: Option<StageTiming>,
    choose_calls: usize,
    pub(super) crc_bytes: u64,
    pub(super) release_check_crc_bytes: u64,
    records_read_per_choice: u64,
    full_segments: u64,
    pub(super) stop_finding: Option<String>,
}

/// 甲段两个子项的字节锚点（登记第七节「甲段的字节 = 这一批计时包装记的扫描读字节」，第十二节 12.5 补的「甲段释放前读盘核那一项的字节
/// = 这一批 `release_check_read_bytes`」）：有一条对不上就交回一句（同停机条款第 4 条那一路报），原来那一项先判；两条都对上交 `None`。
fn crc_byte_anchor_finding(
    call_index: usize,
    crc_bytes: u64,
    scan_read_bytes: u64,
    release_check_crc_bytes: u64,
    release_check_read_bytes: u64,
) -> Option<String> {
    if crc_bytes != scan_read_bytes {
        return Some(format!(
            "clause_4_crc_bytes_{crc_bytes}_differ_from_the_scan_read_bytes_{scan_read_bytes}_at_call_{call_index}"
        ));
    }
    if release_check_crc_bytes != release_check_read_bytes {
        return Some(format!(
            "clause_4_release_check_crc_bytes_{release_check_crc_bytes}_differ_from_the_release_check_read_bytes_{release_check_read_bytes}_at_call_{call_index}"
        ));
    }
    None
}

/// 照会话重放一次调用：甲（每个被读单元整单元 CRC；另一个子项是释放前读盘核那几份副本的整单元 CRC）、乙（整个 `scan_region`）、
/// 丙（签发那一步的选段那一圈）；另核停机条款第 4 条与甲段两个子项的字节锚点。
pub(super) fn replay_the_stages_of_a_call(
    material: &CallReplayMaterial,
    region: Option<CompactionRegion>,
    choice: ChoiceOfTheRegion,
    batch_units: u64,
    pool_devices: &[(DeviceIdentity, SparseBlockDevice)],
) -> CallStageReplay {
    let units = units_read_by_the_scan(material);
    let (crc_bytes, _folded) = crc_stage_over_the_units(&units);
    let scan_read_bytes: u64 = material
        .scan_reads_in_the_region
        .iter()
        .map(|read| read.length)
        .sum();
    let (release_check_crc_bytes, _release_check_folded) =
        crc_stage_over_the_units(&material.release_check_copies);
    let release_check_read_bytes: u64 = material
        .release_check_reads
        .iter()
        .map(|read| read.length)
        .sum();
    let mut stop_finding = crc_byte_anchor_finding(
        material.call_index,
        crc_bytes,
        scan_read_bytes,
        release_check_crc_bytes,
        release_check_read_bytes,
    );
    let crc = median_stage_timing(|| {
        std::hint::black_box(crc_stage_over_the_units(&units));
    });
    let release_check_crc = median_stage_timing(|| {
        std::hint::black_box(crc_stage_over_the_units(&material.release_check_copies));
    });
    let scan = region.and_then(|scanned_region| {
        let version = &material.version;
        let state = &material.state;
        let replayed = scan_region(
            scanned_region,
            &version.allocation_records,
            &version.mapping_entries,
            &version.pinned_slots,
            &state.quarantined_slots,
            replay_reader(material, pool_devices),
        );
        if counts_as_a_batch(material.kind) {
            let movable_keys: Vec<Vec<u8>> = match &replayed {
                Ok(scan) => scan
                    .movable_units()
                    .take(usize::try_from(batch_units).expect("批大小"))
                    .map(|(_placement, unit)| unit.mapping_key.clone())
                    .collect(),
                Err(_refusal) => Vec::new(),
            };
            let inconsistent = replayed.as_ref().map_or(0, |scan| {
                scan.allocations
                    .iter()
                    .filter(|allocation| matches!(allocation.class, AllocationInRegion::Inconsistent(_)))
                    .count()
            });
            if (movable_keys != material.relocated_keys || inconsistent > 0) && stop_finding.is_none() {
                stop_finding = Some(format!(
                    "clause_4_replayed_movable_keys_differ_from_the_relocated_keys_at_call_{}:replayed={}:relocated={}:inconsistent={inconsistent}",
                    material.call_index,
                    movable_keys.len(),
                    material.relocated_keys.len()
                ));
            }
        }
        median_stage_timing(|| {
            std::hint::black_box(
                scan_region(
                    scanned_region,
                    &version.allocation_records,
                    &version.mapping_entries,
                    &version.pinned_slots,
                    &state.quarantined_slots,
                    replay_reader(material, pool_devices),
                )
                .is_ok(),
            );
        })
    });
    let mut choose = None;
    let mut choose_calls = 0;
    if material.kind == CallKind::Issue {
        let replay = replay_the_greedy_choice(
            &material.version,
            &material.state,
            replay_reader(material, pool_devices),
        );
        choose_calls = replay.inputs_of_each_call.len();
        if choice == ChoiceOfTheRegion::Greedy && replay.chosen != region && stop_finding.is_none()
        {
            stop_finding = Some(format!(
                "clause_4_replayed_greedy_choice_{}_differs_from_the_issued_region_{}",
                replay.chosen.map_or_else(
                    || "none".to_string(),
                    |chosen| chosen.start_slot().0.to_string()
                ),
                region.map_or_else(
                    || "none".to_string(),
                    |issued| issued.start_slot().0.to_string()
                )
            ));
        }
        let slots_that_disqualify_a_segment: BTreeSet<SlotNumber> = material
            .version
            .pinned_slots
            .union(&material.state.quarantined_slots)
            .copied()
            .collect();
        choose = median_stage_timing(|| {
            for (region_scans, refused) in &replay.inputs_of_each_call {
                std::hint::black_box(choose_region(
                    &material.version.allocation_records,
                    material.state.unit_area,
                    &material.state.cluster_segments,
                    None,
                    &slots_that_disqualify_a_segment,
                    region_scans,
                    refused,
                    RegionChoiceCriterion::Greedy,
                ));
            }
        });
    }
    CallStageReplay {
        crc,
        release_check_crc,
        scan,
        choose,
        choose_calls,
        crc_bytes,
        release_check_crc_bytes,
        records_read_per_choice: u64::try_from(
            material
                .version
                .allocation_records
                .iter()
                .filter(|record| {
                    record.device == material.state.unit_area.device && !record.is_released
                })
                .count(),
        )
        .expect("记录条数"),
        full_segments: material.state.unit_area.slots / LOCAL_CLUSTER_SEGMENT_SLOTS,
        stop_finding,
    }
}

/// 饱和吞吐三档（1、16、N 线程）各一行；交回 16 线程那一档（给第三段的 Q9）与「起了几条不同的线程」的核过没过。
fn emit_the_saturation(
    emitter: &mut Emitter,
    prefix: &str,
    stage_name: &str,
    bytes_per_repetition: u64,
    stage: &(dyn Fn() + Sync),
) -> bool {
    let mut is_every_thread_count_distinct = true;
    let worker_threads = worker_threads_from_the_environment();
    for thread_count in [1, CPU_THREADS_OF_THE_GPU_OFFLOAD_COMPARISON, worker_threads] {
        let reading = saturated_throughput(
            thread_count,
            Duration::from_secs(SATURATION_SECONDS),
            bytes_per_repetition,
            stage,
        );
        let distinct_holds = reading.distinct_threads == thread_count;
        is_every_thread_count_distinct &= distinct_holds;
        emitter.line(&format!(
            "name=saturation {prefix} stage={stage_name} threads={thread_count} bytes_per_repetition={bytes_per_repetition} repetitions={} bytes_per_second={:.1} distinct_threads={} distinct_threads_holds={distinct_holds}",
            reading.repetitions, reading.bytes_per_second, reading.distinct_threads
        ));
    }
    is_every_thread_count_distinct
}

/// 丙段的盘大小敏感性（登记第八节）：同一份选段输入，分配记录平铺 16 份（单元区 16 倍）喂 `choose_region`，报每整段的 CPU 斜率；TB 级盘照斜率外推、标「推的」。
fn emit_the_disk_size_sensitivity(
    emitter: &mut Emitter,
    prefix: &str,
    material: &CallReplayMaterial,
    replay: &GreedyReplay,
) {
    let Some((final_scans, final_refused)) = replay.inputs_of_each_call.last() else {
        return;
    };
    let state = &material.state;
    let disqualify: BTreeSet<SlotNumber> = material
        .version
        .pinned_slots
        .union(&state.quarantined_slots)
        .copied()
        .collect();
    let time_one_choice =
        |records: &[AllocationRecord], unit_area: UnitAreaOfOneDevice, scans: &[RegionScan]| {
            median_stage_timing(|| {
                std::hint::black_box(choose_region(
                    records,
                    unit_area,
                    &state.cluster_segments,
                    None,
                    &disqualify,
                    scans,
                    final_refused,
                    RegionChoiceCriterion::Greedy,
                ));
            })
        };
    let small = time_one_choice(
        &material.version.allocation_records,
        state.unit_area,
        final_scans,
    );
    let mut tiled_records = Vec::new();
    for copy in 0..DISK_SIZE_MULTIPLIER {
        tiled_records.extend(material.version.allocation_records.iter().map(|record| {
            AllocationRecord {
                device: record.device,
                slot: SlotNumber(record.slot.0 + copy * state.unit_area.slots),
                span_slots: record.span_slots,
                generation: record.generation,
                is_released: record.is_released,
                quarantine_mark: record.quarantine_mark,
            }
        }));
    }
    let large_area = UnitAreaOfOneDevice {
        device: state.unit_area.device,
        start: state.unit_area.start,
        slots: state.unit_area.slots * DISK_SIZE_MULTIPLIER,
    };
    let mut large_scans =
        region_scans_of_the_segments_without_an_unreleased_record(&tiled_records, large_area);
    large_scans.extend(
        final_scans
            .iter()
            .filter(|scan| !scan.allocations.is_empty())
            .cloned(),
    );
    let large = time_one_choice(&tiled_records, large_area, &large_scans);
    let small_segments = state.unit_area.slots / LOCAL_CLUSTER_SEGMENT_SLOTS;
    let large_segments = large_area.slots / LOCAL_CLUSTER_SEGMENT_SLOTS;
    let slope = match (small, large) {
        (Some(small_timing), Some(large_timing)) => Some(
            (large_timing.cpu_nanoseconds_per_repetition
                - small_timing.cpu_nanoseconds_per_repetition)
                / (large_segments - small_segments) as f64,
        ),
        (None, _) | (_, None) => None,
    };
    emitter.line(&format!(
        "name=choose_disk_size {prefix} scanned_segments_before_choosing={} small_full_segments={small_segments} small_records={} small_cpu_nanoseconds_per_call={} large_full_segments={large_segments} large_records={} large_cpu_nanoseconds_per_call={} cpu_nanoseconds_per_full_segment={} one_tebibyte_extrapolated_nanoseconds_per_call={} extrapolation=inferred",
        replay.scanned_regions.len(),
        material.version.allocation_records.len(),
        optional_number(small.map(|timing| timing.cpu_nanoseconds_per_repetition)),
        tiled_records.len(),
        optional_number(large.map(|timing| timing.cpu_nanoseconds_per_repetition)),
        optional_number(slope),
        optional_number(slope.map(|per_segment| per_segment * ((1_u64 << 40) / (LOCAL_CLUSTER_SEGMENT_SLOTS * LOCAL_SLOT_BYTES)) as f64)),
    ));
}

/// 第二行阶段对照（登记第五节阳性对照表第 2 行）：三个阶段计时器各加一段 2 ms 忙等，各跑一次；甲另算两遍。交回没过的几条。
pub(super) fn emit_the_stage_controls(
    emitter: &mut Emitter,
    material: &CallReplayMaterial,
    region: CompactionRegion,
    pool_devices: &[(DeviceIdentity, SparseBlockDevice)],
) -> Vec<String> {
    let units = units_read_by_the_scan(material);
    let injected = Duration::from_millis(INJECTED_MILLISECONDS_PER_WRITE);
    let crc_stage = || {
        std::hint::black_box(crc_stage_over_the_units(&units));
    };
    let scan_stage = || {
        std::hint::black_box(
            scan_region(
                region,
                &material.version.allocation_records,
                &material.version.mapping_entries,
                &material.version.pinned_slots,
                &material.state.quarantined_slots,
                replay_reader(material, pool_devices),
            )
            .is_ok(),
        );
    };
    let disqualify: BTreeSet<SlotNumber> = material
        .version
        .pinned_slots
        .union(&material.state.quarantined_slots)
        .copied()
        .collect();
    let choose_stage = || {
        std::hint::black_box(choose_region(
            &material.version.allocation_records,
            material.state.unit_area,
            &material.state.cluster_segments,
            None,
            &disqualify,
            &[],
            &material.state.segments_refused_by_the_region_scan,
            RegionChoiceCriterion::Greedy,
        ));
    };
    let mut failures = Vec::new();
    let stages: [(&str, &dyn Fn()); 3] = [
        ("crc", &crc_stage),
        ("scan", &scan_stage),
        ("choose", &choose_stage),
    ];
    for (stage_name, stage) in stages {
        let plain = median_stage_timing(stage);
        let with_the_wait = median_stage_timing(|| {
            stage();
            busy_wait_for(injected);
        });
        let increment = match (plain, with_the_wait) {
            (Some(plain_timing), Some(waited_timing)) => Some(
                waited_timing.cpu_nanoseconds_per_repetition
                    - plain_timing.cpu_nanoseconds_per_repetition,
            ),
            (None, _) | (_, None) => None,
        };
        let holds = increment.is_some_and(stage_control_holds);
        if !holds {
            failures.push(format!("row_two_{stage_name}"));
        }
        emitter.line(&format!(
            "name=positive_control row=2 stage={stage_name} increment_nanoseconds={} injected_nanoseconds={} holds={holds}",
            optional_number(increment),
            INJECTED_MILLISECONDS_PER_WRITE * 1_000_000
        ));
    }
    let once = median_stage_timing(crc_stage);
    let twice = median_stage_timing(|| {
        crc_stage();
        crc_stage();
    });
    let double_holds = match (once, twice) {
        (Some(once_timing), Some(twice_timing)) => double_crc_control_holds(
            twice_timing.cpu_nanoseconds_per_repetition,
            once_timing.cpu_nanoseconds_per_repetition,
        ),
        (None, _) | (_, None) => false,
    };
    if !double_holds {
        failures.push("row_two_crc_twice".to_string());
    }
    emitter.line(&format!(
        "name=positive_control row=2 stage=crc_twice once_nanoseconds={} twice_nanoseconds={} holds={double_holds}",
        optional_number(once.map(|timing| timing.cpu_nanoseconds_per_repetition)),
        optional_number(twice.map(|timing| timing.cpu_nanoseconds_per_repetition)),
    ));
    failures
}

/// 一次调用三个阶段的 CPU 与份额（Q5、Q6、Q7 的算式照原登记第六节）：`name=stage` 那一行、批行的判定与丙段的判定（N13）都从这里取。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct StageShareRow {
    pub(super) kind: CallKind,
    pub(super) crc_cpu_nanoseconds: Option<f64>,
    scan_cpu_nanoseconds: Option<f64>,
    pub(super) scan_increment_cpu_nanoseconds: Option<f64>,
    pub(super) choose_cpu_nanoseconds: Option<f64>,
    crc_share: Option<f64>,
    scan_increment_share: Option<f64>,
    /// 选段 CPU ÷ 这条意图的批数 ÷ 这条意图批 CPU 的中位（原登记 Q7 的算法；只有签发那一次的重放里有选段 CPU）。
    choose_share_per_batch: Option<f64>,
    /// 签发那一次的原值：选段 CPU ÷ 这一次调用的 CPU（只报）。
    choose_share_of_the_issue: Option<f64>,
}

impl StageShareRow {
    /// 这一行上 Q5、Q6、Q7 的判定（读不到的是 `None`）。
    fn judgements(&self) -> [Option<&'static str>; 3] {
        [
            self.crc_share,
            self.scan_increment_share,
            self.choose_share_per_batch,
        ]
        .map(|share| share.map(|value| judgement_of_share(value, STAGE_SHARE_AT_LEAST_FOR_WORTH)))
    }
}

fn stage_share_row(
    call: &CallReading,
    kind: CallKind,
    replay: &CallStageReplay,
    batches_of_the_intent: u64,
    median_batch_cpu: Option<f64>,
) -> StageShareRow {
    let call_cpu = call.cpu_nanoseconds as f64;
    let crc_cpu = replay
        .crc
        .map(|timing| timing.cpu_nanoseconds_per_repetition);
    let scan_cpu = replay
        .scan
        .map(|timing| timing.cpu_nanoseconds_per_repetition);
    let scan_increment = match (scan_cpu, crc_cpu) {
        (Some(scan), Some(crc)) => Some(scan - crc),
        (None, _) | (_, None) => None,
    };
    let choose_cpu = replay
        .choose
        .map(|timing| timing.cpu_nanoseconds_per_repetition);
    let choose_share_per_batch = match (choose_cpu, median_batch_cpu) {
        (Some(choose), Some(batch_cpu)) if batches_of_the_intent > 0 => {
            Some(choose / batches_of_the_intent as f64 / batch_cpu)
        }
        (Some(_), Some(_)) | (None, _) | (_, None) => None,
    };
    StageShareRow {
        kind,
        crc_cpu_nanoseconds: crc_cpu,
        scan_cpu_nanoseconds: scan_cpu,
        scan_increment_cpu_nanoseconds: scan_increment,
        choose_cpu_nanoseconds: choose_cpu,
        crc_share: crc_cpu.map(|crc| crc / call_cpu),
        scan_increment_share: scan_increment.map(|increment| increment / call_cpu),
        choose_share_per_batch,
        choose_share_of_the_issue: choose_cpu.map(|choose| choose / call_cpu),
    }
}

/// 一条意图里批行的 Q5/Q6/Q7 判定：（批序号、三个判定），按批序号。
type BatchShareJudgements = Vec<(usize, [Option<&'static str>; 3])>;

/// 一条意图逐次调用的份额行里，批行的 Q5/Q6/Q7 判定（批序号 → 判定），次序同 `name=stage_judgement` 一直以来的收法。
fn batch_judgements_of(rows: &[StageShareRow]) -> BatchShareJudgements {
    rows.iter()
        .filter(|row| counts_as_a_batch(row.kind))
        .enumerate()
        .map(|(batch_index, row)| (batch_index, row.judgements()))
        .collect()
}

/// 丙段（N13）一轮的读数：那一轮有效意图签发那一行的 `choose_share_per_batch`（不从批行收：选段只在签发那一次发生）。
fn choose_share_of_the_issue_row(rows: &[StageShareRow]) -> Option<f64> {
    rows.iter()
        .find(|row| row.kind == CallKind::Issue)
        .and_then(|row| row.choose_share_per_batch)
}

/// 丙段五轮收成一个判定（原登记 Q7 的门槛 10%）：每轮的判定，读不到的那一轮写 `not_run`（不当 0）；五轮相同写那个判定，不全相同写 `unstable`。
fn choose_judgement_of_the_rounds(
    choose_share_per_round: &[Option<f64>],
) -> (Vec<&'static str>, &'static str) {
    let per_round: Vec<&'static str> = choose_share_per_round
        .iter()
        .map(|share| {
            share.map_or("not_run", |value| {
                judgement_of_share(value, STAGE_SHARE_AT_LEAST_FOR_WORTH)
            })
        })
        .collect();
    let judgement = judgement_of_the_rounds(&per_round);
    (per_round, judgement)
}

/// 第二段一次调用的份额与吞吐（Q5–Q8）：份额的分母是这一次被测调用的 CPU（线程 CPU 时钟），分子是阶段重放每次重复的 CPU。
/// Q5 与 Q8 只用甲段原来那一项（区域扫描读到的单元）；释放前读盘核那一项与两项之和另列在同一行末尾，不进判定（登记第十二节 12.5）。
/// 返回这一次调用的份额那一行（[`stage_share_row`]，N13、N14 从它取）。
pub(super) fn emit_the_stage_shares(
    emitter: &mut Emitter,
    prefix: &str,
    call: &CallReading,
    material: &CallReplayMaterial,
    replay: &CallStageReplay,
    batches_of_the_intent: u64,
    median_batch_cpu: Option<f64>,
) -> StageShareRow {
    let row = stage_share_row(
        call,
        material.kind,
        replay,
        batches_of_the_intent,
        median_batch_cpu,
    );
    let crc_cpu = row.crc_cpu_nanoseconds;
    let scan_increment = row.scan_increment_cpu_nanoseconds;
    let choose_cpu = row.choose_cpu_nanoseconds;
    let release_check_crc_cpu = replay
        .release_check_crc
        .map(|timing| timing.cpu_nanoseconds_per_repetition);
    let crc_with_the_release_check_cpu = match (crc_cpu, release_check_crc_cpu) {
        (Some(crc), Some(release_check_crc)) => Some(crc + release_check_crc),
        (None, _) | (_, None) => None,
    };
    let crc_bytes = replay.crc_bytes as f64;
    let choose_bytes = (replay.records_read_per_choice * LOCAL_ALLOCATION_RECORD_BYTES) as f64
        * replay.choose_calls as f64;
    emitter.line(&format!(
        "name=stage {prefix} call_index={} kind={} call_cpu_nanoseconds={} crc_cpu_nanoseconds={} crc_wall_nanoseconds={} crc_repetitions={} scan_cpu_nanoseconds={} scan_increment_cpu_nanoseconds={} choose_cpu_nanoseconds={} choose_calls={} crc_bytes={} records_read_per_choice={} full_segments={} crc_share={} scan_increment_share={} choose_share_per_batch={} choose_share_of_the_issue={} crc_bytes_per_second={} scan_increment_bytes_per_second={} choose_record_bytes_per_second={} choose_records_per_second={} choose_full_segments_per_second={} snapshot_branch_bytes=0 snapshot_branch_note=the_pools_have_no_snapshot release_check_crc_cpu_nanoseconds={} release_check_crc_wall_nanoseconds={} release_check_crc_repetitions={} release_check_crc_bytes={} crc_with_the_release_check_cpu_nanoseconds={} crc_with_the_release_check_bytes={}",
        material.call_index,
        material.kind.label(),
        call.cpu_nanoseconds,
        optional_number(crc_cpu),
        optional_number(replay.crc.map(|timing| timing.wall_nanoseconds_per_repetition)),
        replay.crc.map_or(0, |timing| timing.repetitions),
        optional_number(row.scan_cpu_nanoseconds),
        optional_number(scan_increment),
        optional_number(choose_cpu),
        replay.choose_calls,
        replay.crc_bytes,
        replay.records_read_per_choice,
        replay.full_segments,
        optional_number(row.crc_share),
        optional_number(row.scan_increment_share),
        optional_number(row.choose_share_per_batch),
        optional_number(row.choose_share_of_the_issue),
        optional_number(crc_cpu.map(|crc| crc_bytes / (crc / 1e9))),
        optional_number(scan_increment.map(|increment| crc_bytes / (increment / 1e9))),
        optional_number(choose_cpu.map(|choose| choose_bytes / (choose / 1e9))),
        optional_number(choose_cpu.map(|choose| (replay.records_read_per_choice as f64 * replay.choose_calls as f64) / (choose / 1e9))),
        optional_number(choose_cpu.map(|choose| (replay.full_segments as f64 * replay.choose_calls as f64) / (choose / 1e9))),
        optional_number(release_check_crc_cpu),
        optional_number(replay.release_check_crc.map(|timing| timing.wall_nanoseconds_per_repetition)),
        replay.release_check_crc.map_or(0, |timing| timing.repetitions),
        replay.release_check_crc_bytes,
        optional_number(crc_with_the_release_check_cpu),
        replay.crc_bytes + replay.release_check_crc_bytes,
    ));
    row
}

/// 第二段一格一轮交回的东西。
pub(super) struct StageSplitIntentOutcome {
    pub(super) stop_findings: Vec<String>,
    pub(super) control_failures: Vec<String>,
    /// 这一趟作废规则下收下的那一次逐次调用的份额行（三次都作废是 `None`）。
    registered: Option<Vec<StageShareRow>>,
    /// 照原作废规则（不加 R2-1）收下的那一次逐次调用的份额行（N14，只报不判）。
    pub(super) under_the_original_voiding: Option<Vec<StageShareRow>>,
}

/// 这条意图的批数与批 CPU 的中位（Q7 的分母）。
pub(super) fn batches_and_the_median_batch_cpu_of(reading: &IntentReading) -> (u64, Option<f64>) {
    let batch_cpus: Vec<f64> = reading
        .calls
        .iter()
        .filter(|call| counts_as_a_batch(call.kind))
        .map(|call| call.cpu_nanoseconds as f64)
        .collect();
    (
        u64::try_from(batch_cpus.len()).expect("批数"),
        median_of(&batch_cpus),
    )
}

/// 收下的那一条意图：逐次调用重放三个阶段、打 `name=stage` 行、判停机条款第 4 条；最先一轮另量饱和吞吐、盘大小敏感性，`with_the_controls` 时
/// 另跑第二行对照。停机触发与对照失败记进 `outcome`；交回逐次调用的份额行。
fn replay_and_emit_the_stages(
    emitter: &mut Emitter,
    reading: &IntentReading,
    label: &MeasurementLabel,
    pool: &ConstructedPool,
    with_the_controls: bool,
    outcome: &mut StageSplitIntentOutcome,
) -> Vec<StageShareRow> {
    let (prepared_choice, batch_units, round) = (label.choice, label.batch_units, label.round);
    let prefix = format!(
        "{} batch_units={batch_units} round={round}",
        label.point.label()
    );
    let (batches_of_the_intent, median_batch_cpu) = batches_and_the_median_batch_cpu_of(reading);
    let mut rows = Vec::new();
    let mut batch_index = 0_usize;
    for material in &reading.replay_material {
        let call = &reading.calls[material.call_index];
        enter_the_timing_window(TimingWindow::StageReplay);
        let replay = replay_the_stages_of_a_call(
            material,
            reading.region,
            prepared_choice,
            batch_units,
            &pool.devices,
        );
        if let Some(finding) = &replay.stop_finding {
            emitter.line(&format!(
                "name=stop_clause clause=4 {prefix} finding={finding}"
            ));
            outcome.stop_findings.push(finding.clone());
        }
        rows.push(emit_the_stage_shares(
            emitter,
            &prefix,
            call,
            material,
            &replay,
            batches_of_the_intent,
            median_batch_cpu,
        ));
        if counts_as_a_batch(material.kind) {
            batch_index += 1;
        }
        if round == 1 {
            let units = units_read_by_the_scan(material);
            if counts_as_a_batch(material.kind) && batch_index == 1 {
                if let Some(region) = reading.region {
                    let crc_stage = || {
                        std::hint::black_box(crc_stage_over_the_units(&units));
                    };
                    let scan_stage = || {
                        std::hint::black_box(
                            scan_region(
                                region,
                                &material.version.allocation_records,
                                &material.version.mapping_entries,
                                &material.version.pinned_slots,
                                &material.state.quarantined_slots,
                                |_device, slot, length| {
                                    material
                                        .unit_bytes_by_slot
                                        .get(&slot.0)
                                        .filter(|bytes| bytes.len() == length)
                                        .cloned()
                                },
                            )
                            .is_ok(),
                        );
                    };
                    let bytes = replay.crc_bytes;
                    if !emit_the_saturation(emitter, &prefix, "crc", bytes, &crc_stage)
                        || !emit_the_saturation(emitter, &prefix, "scan", bytes, &scan_stage)
                    {
                        outcome
                            .control_failures
                            .push("saturation_threads_not_distinct".to_string());
                    }
                    if with_the_controls {
                        enter_the_timing_window(TimingWindow::StageControls);
                        outcome.control_failures.extend(emit_the_stage_controls(
                            emitter,
                            material,
                            region,
                            &pool.devices,
                        ));
                    }
                }
            }
            if material.kind == CallKind::Issue {
                let greedy = replay_the_greedy_choice(
                    &material.version,
                    &material.state,
                    replay_reader(material, &pool.devices),
                );
                let disqualify: BTreeSet<SlotNumber> = material
                    .version
                    .pinned_slots
                    .union(&material.state.quarantined_slots)
                    .copied()
                    .collect();
                let choose_stage = || {
                    for (region_scans, refused) in &greedy.inputs_of_each_call {
                        std::hint::black_box(choose_region(
                            &material.version.allocation_records,
                            material.state.unit_area,
                            &material.state.cluster_segments,
                            None,
                            &disqualify,
                            region_scans,
                            refused,
                            RegionChoiceCriterion::Greedy,
                        ));
                    }
                };
                let choose_bytes = replay.records_read_per_choice
                    * LOCAL_ALLOCATION_RECORD_BYTES
                    * u64::try_from(greedy.inputs_of_each_call.len()).expect("调用次数");
                if !emit_the_saturation(emitter, &prefix, "choose", choose_bytes, &choose_stage) {
                    outcome
                        .control_failures
                        .push("saturation_threads_not_distinct".to_string());
                }
                enter_the_timing_window(TimingWindow::DiskSizeSensitivity);
                emit_the_disk_size_sensitivity(emitter, &prefix, material, &greedy);
            }
        }
    }
    rows
}

/// 只因校准作废、照原作废规则算有效的那一次（N14）：逐次调用重放三个阶段只为算份额，不打 `name=stage` 行、不量饱和吞吐与对照。
fn stage_share_rows_without_emitting(
    reading: &IntentReading,
    label: &MeasurementLabel,
    pool: &ConstructedPool,
) -> Vec<StageShareRow> {
    let (batches_of_the_intent, median_batch_cpu) = batches_and_the_median_batch_cpu_of(reading);
    reading
        .replay_material
        .iter()
        .map(|material| {
            enter_the_timing_window(TimingWindow::StageReplay);
            let replay = replay_the_stages_of_a_call(
                material,
                reading.region,
                label.choice,
                label.batch_units,
                &pool.devices,
            );
            stage_share_row(
                &reading.calls[material.call_index],
                material.kind,
                &replay,
                batches_of_the_intent,
                median_batch_cpu,
            )
        })
        .collect()
}

/// 收下的那一条意图交给谁重放：第二段是 [`replay_and_emit_the_stages`]，第 4 次重跑写法与拆分段另加九种写法的计时件。
pub(super) type ReplayOfTheKeptIntent<'replay> = dyn FnMut(
        &mut Emitter,
        &IntentReading,
        &MeasurementLabel,
        &ConstructedPool,
        &mut StageSplitIntentOutcome,
    ) -> Vec<StageShareRow>
    + 'replay;

/// 第二段一个取样点、一个 K、一轮（`label` 的 `attempt` 不读）：量一条意图（收重放材料），收下的那一次交 `replay_the_kept_intent`
/// 逐次调用重放（第二段是 [`replay_and_emit_the_stages`]）。每一次前后做校准与读数，校准作废的照 R2-1 作废、作废之后照 R2-2 等常态再重量；
/// 照原作废规则会收下的那一次另算一份份额（N14）。
pub(super) fn stage_split_one_intent(
    emitter: &mut Emitter,
    label: MeasurementLabel,
    pool: &ConstructedPool,
    calibration: &mut SegmentCalibration,
    replay_the_kept_intent: &mut ReplayOfTheKeptIntent<'_>,
) -> StageSplitIntentOutcome {
    let switches = measured_switches(label.batch_units, label.choice.forced_region());
    let mut outcome = StageSplitIntentOutcome {
        stop_findings: Vec::new(),
        control_failures: Vec::new(),
        registered: None,
        under_the_original_voiding: None,
    };
    let mut voidings: Vec<AttemptVoiding> = Vec::new();
    let mut rows_of_each_attempt: Vec<Option<Vec<StageShareRow>>> = Vec::new();
    for attempt in 1..=ATTEMPTS_PER_ROUND_AT_MOST {
        enter_the_timing_window(TimingWindow::StageSplitCompactionRun);
        let mut attempt_label = label;
        attempt_label.attempt = attempt;
        let (reading, intent_calibration) = calibration.around_the_intent(|| {
            measure_on_the_memory_device(pool, switches, WriteInjection::Nothing, ReplayCapture::On)
        });
        calibration.note_the_intent(emitter, &attempt_label, &intent_calibration);
        let calibration_invalid_reason = intent_calibration.verdict.invalid_reason();
        let original_invalid_reason = invalid_reason_of(&reading);
        let invalid_reason = original_invalid_reason.or(calibration_invalid_reason);
        emit_the_intent(emitter, &attempt_label, &reading, invalid_reason);
        voidings.push(AttemptVoiding {
            is_valid_under_the_original_voiding: original_invalid_reason.is_none(),
            is_voided_by_the_calibration: calibration_invalid_reason.is_some(),
        });
        match invalid_reason {
            None => {
                let rows =
                    replay_the_kept_intent(emitter, &reading, &attempt_label, pool, &mut outcome);
                rows_of_each_attempt.push(Some(rows.clone()));
                outcome.registered = Some(rows);
                break;
            }
            Some(reason) => {
                rows_of_each_attempt.push(
                    original_invalid_reason
                        .is_none()
                        .then(|| stage_share_rows_without_emitting(&reading, &attempt_label, pool)),
                );
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
    if outcome.registered.is_none() {
        emitter.line(&format!(
            "name=round_invalid arm=memory {} batch_units={} round={} attempts={ATTEMPTS_PER_ROUND_AT_MOST}",
            label.point.label(),
            label.batch_units,
            label.round,
        ));
    }
    outcome.under_the_original_voiding =
        kept_attempt_under_the_original_voiding(emitter, &label, &voidings)
            .and_then(|kept_index| rows_of_each_attempt.get_mut(kept_index))
            .and_then(Option::take);
    outcome
}

/// N14（第二段，只报不判）：一格（f、P、K）里两种作废规则下各自收下的那几轮，甲、乙段按批序号五轮收成的判定与丙段的判定。
fn stage_cell_judgements_text(rows_per_round: &[Vec<StageShareRow>]) -> String {
    let batch_judgements_per_round: Vec<BatchShareJudgements> = rows_per_round
        .iter()
        .map(|rows| batch_judgements_of(rows))
        .collect();
    let batch_indices: BTreeSet<usize> = batch_judgements_per_round
        .iter()
        .flatten()
        .map(|(batch_index, _judgements)| *batch_index)
        .collect();
    let judgement_per_batch = |position: usize| {
        batch_indices
            .iter()
            .map(|batch_index| {
                let per_round: Vec<&'static str> = batch_judgements_per_round
                    .iter()
                    .map(|judgements| {
                        judgements
                            .iter()
                            .find(|(index, _judgements)| index == batch_index)
                            .and_then(|(_index, cell)| cell[position])
                            .unwrap_or("not_run")
                    })
                    .collect();
                judgement_of_the_rounds(&per_round)
            })
            .collect::<Vec<_>>()
            .join(",")
    };
    let choose_share_per_round: Vec<Option<f64>> = rows_per_round
        .iter()
        .map(|rows| choose_share_of_the_issue_row(rows))
        .collect();
    let (_choose_per_round, choose_judgement) =
        choose_judgement_of_the_rounds(&choose_share_per_round);
    format!(
        "rounds={} crc_share_judgement_per_batch={} scan_increment_share_judgement_per_batch={} choose_share_judgement={choose_judgement}",
        rows_per_round.len(),
        judgement_per_batch(0),
        judgement_per_batch(1),
    )
}

pub(super) fn run_the_stage_split_segment(points: &[SamplingPoint]) -> i32 {
    let segment_started = Instant::now();
    let settings = PreparationSettings::from_the_environment();
    let mut emitter = Emitter::new();
    emitter.line(&format!(
        "name=configuration segment=stage_split points={} batch_sizes={} rounds={ROUNDS} stage_replays={STAGE_REPLAYS} stage_timed_runs_per_replay=1 saturation_seconds={SATURATION_SECONDS} worker_threads={} {}",
        points.iter().map(|point| format!("{}:{}", point.fraction.label(), point.pool_data_units)).collect::<Vec<_>>().join(","),
        BATCH_SIZES.iter().map(u64::to_string).collect::<Vec<_>>().join(","),
        worker_threads_from_the_environment(),
        preparation_configuration_fields(points, None, &settings),
    ));
    assert_this_process_runs_one_thread("segment_start");
    if !check_the_apparatus_sha256(&mut emitter) {
        return EXIT_CODE_STOP_CLAUSE;
    }
    if !check_the_thread_cpu_clock_resolution(&mut emitter) {
        return stop_for_the_clock_resolution(&mut emitter, "stage_split");
    }
    emit_the_machine_state(
        &mut emitter,
        "stage_split",
        MachineStatePosition::SegmentStart,
        segment_started,
    );
    let mut calibration =
        match SegmentCalibration::start("stage_split", segment_started, &mut emitter) {
            Ok(calibration) => calibration,
            Err(exit_code) => return exit_code,
        };
    let mismatches = check_the_constants_and_the_anchors(&mut emitter);
    let mut stop_findings: Vec<String> = Vec::new();
    let mut control_failures: Vec<String> = Vec::new();
    let mut is_the_first_intent = true;
    if mismatches == 0 {
        let mut queue = PreparationQueue::new(
            "stage_split",
            points,
            None,
            &settings,
            &run_the_preparation_work,
        );
        'points: for point_index in 0..points.len() {
            // 从准备队列取这一点造好的池（还没好就跑一次准备），量完放掉（`finish_the_point`）
            let prepared = queue.prepared_point(point_index, &mut emitter);
            let point = &prepared.recipe.point;
            for line in &prepared.lines {
                emitter.line(line);
            }
            emit_the_machine_state(
                &mut emitter,
                "stage_split",
                MachineStatePosition::PointStart(*point),
                segment_started,
            );
            let choice = prepared.choice;
            let mut rebuilt_pool_of_the_round: Option<ConstructedPool> = None;
            let mut judgements_by_cell: BTreeMap<(u64, usize), Vec<[Option<&'static str>; 3]>> =
                BTreeMap::new();
            let mut choose_share_by_batch_units: BTreeMap<u64, Vec<Option<f64>>> = BTreeMap::new();
            let mut registered_rows_by_batch_units: BTreeMap<u64, Vec<Vec<StageShareRow>>> =
                BTreeMap::new();
            let mut original_rows_by_batch_units: BTreeMap<u64, Vec<Vec<StageShareRow>>> =
                BTreeMap::new();
            for round in 1..=ROUNDS {
                wait_until_no_other_measurement_runs(&mut emitter);
                if round > 1 {
                    let rebuilt = queue.take_the_rebuilt_pool(point_index, round, &mut emitter);
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
                let Some(pool) = pool_of_the_round else {
                    emitter.line(&format!("name=round_invalid arm=memory {} round={round} reason=start_image_digest_differs", point.label()));
                    continue;
                };
                for batch_units in BATCH_SIZES {
                    let label = MeasurementLabel {
                        arm: DeviceArm::Memory,
                        point: *point,
                        batch_units,
                        round,
                        attempt: 0,
                        injection: WriteInjection::Nothing,
                        choice,
                    };
                    let with_the_controls = is_the_first_intent;
                    let outcome = stage_split_one_intent(
                        &mut emitter,
                        label,
                        pool,
                        &mut calibration,
                        &mut |replay_emitter, kept_reading, kept_label, kept_pool, kept_outcome| {
                            replay_and_emit_the_stages(
                                replay_emitter,
                                kept_reading,
                                kept_label,
                                kept_pool,
                                with_the_controls,
                                kept_outcome,
                            )
                        },
                    );
                    is_the_first_intent = false;
                    control_failures.extend(outcome.control_failures);
                    if let Some(rows) = outcome.registered {
                        for (batch_index, cell) in batch_judgements_of(&rows) {
                            judgements_by_cell
                                .entry((batch_units, batch_index))
                                .or_default()
                                .push(cell);
                        }
                        choose_share_by_batch_units
                            .entry(batch_units)
                            .or_default()
                            .push(choose_share_of_the_issue_row(&rows));
                        registered_rows_by_batch_units
                            .entry(batch_units)
                            .or_default()
                            .push(rows);
                    }
                    if let Some(rows) = outcome.under_the_original_voiding {
                        original_rows_by_batch_units
                            .entry(batch_units)
                            .or_default()
                            .push(rows);
                    }
                    if !outcome.stop_findings.is_empty() {
                        stop_findings.extend(outcome.stop_findings);
                        break 'points;
                    }
                }
            }
            for ((batch_units, batch_index), cells) in &judgements_by_cell {
                let stable = |position: usize| {
                    let per_round: Vec<&'static str> = cells
                        .iter()
                        .map(|cell| cell[position].unwrap_or("not_run"))
                        .collect();
                    (per_round.join(","), judgement_of_the_rounds(&per_round))
                };
                let (crc_share_judgement_per_round, crc_share_judgement) = stable(0);
                let (scan_increment_share_judgement_per_round, scan_increment_share_judgement) =
                    stable(1);
                let (choose_share_judgement_per_round, choose_share_judgement) = stable(2);
                emitter.line(&format!(
                    "name=stage_judgement {} batch_units={batch_units} batch_index={batch_index} rounds={} crc_share_judgement_per_round={crc_share_judgement_per_round} crc_share_judgement={crc_share_judgement} scan_increment_share_judgement_per_round={scan_increment_share_judgement_per_round} scan_increment_share_judgement={scan_increment_share_judgement} choose_share_judgement_per_round={choose_share_judgement_per_round} choose_share_judgement={choose_share_judgement}",
                    point.label(),
                    cells.len()
                ));
            }
            for (batch_units, choose_share_per_round) in &choose_share_by_batch_units {
                let (choose_share_judgement_per_round, choose_share_judgement) =
                    choose_judgement_of_the_rounds(choose_share_per_round);
                emitter.line(&format!(
                    "name=choose_judgement {} batch_units={batch_units} rounds={} choose_share_per_batch_per_round={} choose_share_judgement_per_round={} choose_share_judgement={choose_share_judgement}",
                    point.label(),
                    choose_share_per_round.len(),
                    choose_share_per_round.iter().map(|share| optional_number(*share)).collect::<Vec<_>>().join(","),
                    choose_share_judgement_per_round.join(","),
                ));
            }
            for batch_units in BATCH_SIZES {
                let empty: Vec<Vec<StageShareRow>> = Vec::new();
                let original = original_rows_by_batch_units
                    .get(&batch_units)
                    .unwrap_or(&empty);
                let registered = registered_rows_by_batch_units
                    .get(&batch_units)
                    .unwrap_or(&empty);
                let original_text = stage_cell_judgements_text(original);
                let registered_text = stage_cell_judgements_text(registered);
                emitter.line(&format!(
                    "name=judgements_under_the_original_voiding segment=stage_split {} batch_units={batch_units} {original_text} registered_{} differs_from_the_registered={}",
                    point.label(),
                    registered_text.replace(' ', " registered_"),
                    original_text != registered_text,
                ));
            }
            queue.finish_the_point(point_index);
        }
    }
    calibration.finish(&mut emitter);
    emit_the_machine_state(
        &mut emitter,
        "stage_split",
        MachineStatePosition::SegmentEnd,
        segment_started,
    );
    emitter.line(&format!(
        "name=verdict segment=stage_split constant_or_anchor_mismatches={mismatches} stop_clause_findings={} positive_control_failures={} positive_control_failure_items={}",
        stop_findings.len(),
        control_failures.len(),
        if control_failures.is_empty() { "none".to_string() } else { control_failures.join(",") },
    ));
    emitter.finish();
    if mismatches > 0 || !stop_findings.is_empty() {
        EXIT_CODE_STOP_CLAUSE
    } else if !control_failures.is_empty() {
        EXIT_CODE_POSITIVE_CONTROL_FAILED
    } else {
        EXIT_CODE_OK
    }
}

#[cfg(test)]
mod tests;
