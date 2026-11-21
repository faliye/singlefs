use super::*;
use singlefs_verification::gpu_unit_checks::{
    bytes_crc32c_of_ranges_needs, gpu_cards_by_priority, GpuCard,
};

/// 段数 n 的取值（登记第五节）。
const RANGE_COUNTS: [u64; 8] = [1, 4, 16, 64, 256, 1024, 4096, 16_384];
/// 每段的字节：只录那一段算 CPU 参照用的同一份（[`SUBMISSION_RANGE_LENGTHS`]）。
const RANGE_LENGTHS: [u64; 2] = SUBMISSION_RANGE_LENGTHS;
/// 每档计时几次取中位（先丢一次编管线、建上下文）。
const TIMED_SUBMISSIONS: usize = 5;

/// 造池时写的一个真数据单元的整单元字节（32768）：mkfs、第一个文件、建一个 inode、顺序写一个数据单元，从盘上读回。
pub(super) fn one_data_unit_of_the_workload() -> Vec<u8> {
    let (mut devices, mut session) = formatted_pool_with_the_first_file_and_a_session();
    let inode = version_of_trees(&session).inode_number_watermark();
    let mut floor_raises = 0_u64;
    publish_while_constructing(
        &mut session,
        &mut devices,
        UserChange::NewInodes {
            count: 1,
            write_time_seconds: experiment_pool::FIXED_WRITE_TIME_SECONDS
                + SECONDS_FROM_THE_FIRST_FILE_TO_THE_WORKLOAD,
        },
        &mut floor_raises,
    );
    let mut workload = WorkloadUnits {
        live_data_inode_by_slot: BTreeMap::new(),
        slot_of_inode: BTreeMap::new(),
        floor_raises,
    };
    workload.write_one_unit(
        &mut session,
        &mut devices,
        inode,
        0,
        experiment_pool::FIXED_WRITE_TIME_SECONDS + SECONDS_FROM_THE_FIRST_FILE_TO_THE_WORKLOAD + 1,
    );
    let slot = workload.slot_of_inode[&inode];
    PoolReader::read(
        devices.as_slice(),
        DeviceIdentity(0),
        SlotNumber(slot).to_device_offset(),
        usize::try_from(LOCAL_DATA_UNIT_BYTES).expect("32768"),
    )
    .expect("刚写的数据单元读得回")
}

/// `nvidia-smi` 报的每张卡的 PCIe 链路代数与宽度（序号 → (代数, 宽度)）；读不到交回空表。
pub(super) fn pcie_links_reported_by_nvidia_smi() -> BTreeMap<u32, (u32, u32)> {
    let Ok(output) = std::process::Command::new("nvidia-smi")
        .args([
            "--query-gpu=index,pcie.link.gen.max,pcie.link.width.max",
            "--format=csv,noheader,nounits",
        ])
        .output()
    else {
        return BTreeMap::new();
    };
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let fields: Vec<u32> = line
                .split(',')
                .filter_map(|field| field.trim().parse().ok())
                .collect();
            match fields.as_slice() {
                [index, generation, width] => Some((*index, (*generation, *width))),
                _ => None,
            }
        })
        .collect()
}

/// 一张卡一组（段长）一个 n 的读数：5 次计时的中位、每次是否没快过物理下限、CRC 与 CPU 是否逐段相同。
struct SubmissionReading {
    range_count: u64,
    median_seconds: f64,
    respects_the_floor: bool,
    crcs_agree: bool,
}

/// 一张卡一组（段长）一个 n：`segment` 与它的 CPU 参照 CRC `cpu_crc` 都是判段从库里读回的（只录那一段算的参照，持卡时不算）。
fn time_the_submissions(
    card: &GpuCard,
    segment: &[u8],
    cpu_crc: u32,
    range_count: u64,
    link_peak: f64,
) -> SubmissionReading {
    let count = usize::try_from(range_count).expect("段数装得进 usize");
    let ranges: Vec<&[u8]> = vec![segment; count];
    let expected = vec![cpu_crc; count];
    let _warm_up = card.crc32c_of_ranges(&ranges);
    let mut seconds = Vec::new();
    let mut respects_the_floor = true;
    let mut crcs_agree = true;
    for _submission in 0..TIMED_SUBMISSIONS {
        let started = Instant::now();
        let results = card.crc32c_of_ranges(&ranges);
        let elapsed = started.elapsed().as_secs_f64();
        respects_the_floor &=
            respects_the_physical_floor(elapsed, range_count * LOCAL_GPU_SLOT_BYTES, link_peak);
        crcs_agree &= gpu_crcs_agree_with_the_cpu(&results, &expected);
        seconds.push(elapsed);
    }
    SubmissionReading {
        range_count,
        median_seconds: median_of(&seconds).expect("计了 5 次"),
        respects_the_floor,
        crcs_agree,
    }
}

/// 第一段产物里每批交给 GPU 的字节 B_批（内存臂、不注入、第 1 轮的批），键（活槽占比、P、K、调用序号）。
fn batch_bytes_of_the_wall_clock_product(
    lines: &[BTreeMap<String, String>],
) -> BTreeMap<(String, String, String, String), u64> {
    lines
        .iter()
        .filter(|fields| {
            fields.get("name").map(String::as_str) == Some("call")
                && fields.get("arm").map(String::as_str) == Some("memory")
                && fields.get("injection").map(String::as_str) == Some("none")
                && fields.get("round").map(String::as_str) == Some("1")
                && matches!(
                    fields.get("kind").map(String::as_str),
                    Some("batch" | "final_batch")
                )
        })
        .filter_map(|fields| {
            Some((
                (
                    fields.get("fraction")?.clone(),
                    fields.get("pool_data_units")?.clone(),
                    fields.get("batch_units")?.clone(),
                    fields.get("call_index")?.clone(),
                ),
                fields.get("scan_read_bytes_in_the_region")?.parse().ok()?,
            ))
        })
        .collect()
}

/// 第二段产物里的饱和吞吐，键（活槽占比、P、K、阶段、线程数）→ 字节 / 秒；另交回第二段的阶段字节（键（活槽占比、P、K、阶段）→ 字节）。
#[allow(clippy::type_complexity, reason = "两张表各是一种量：吞吐与字节")]
fn readings_of_the_stage_split_product(
    lines: &[BTreeMap<String, String>],
) -> (
    BTreeMap<(String, String, String, String, String), f64>,
    BTreeMap<(String, String, String, String), f64>,
) {
    let mut throughput = BTreeMap::new();
    let mut stage_bytes = BTreeMap::new();
    for fields in lines {
        let key_of = |stage: &str| -> Option<(String, String, String, String)> {
            Some((
                fields.get("fraction")?.clone(),
                fields.get("pool_data_units")?.clone(),
                fields.get("batch_units")?.clone(),
                stage.to_string(),
            ))
        };
        match fields.get("name").map(String::as_str) {
            Some("saturation") if fields.get("round").map(String::as_str) == Some("1") => {
                let (Some(stage), Some(threads), Some(bytes_per_second)) = (
                    fields.get("stage"),
                    fields.get("threads"),
                    fields
                        .get("bytes_per_second")
                        .and_then(|value| value.parse::<f64>().ok()),
                ) else {
                    continue;
                };
                if let Some((fraction, pool, batch, stage_name)) = key_of(stage) {
                    throughput.insert(
                        (fraction, pool, batch, stage_name, threads.clone()),
                        bytes_per_second,
                    );
                }
            }
            Some("stage") if fields.get("round").map(String::as_str) == Some("1") => {
                let kind = fields.get("kind").map(String::as_str);
                if matches!(kind, Some("batch" | "final_batch"))
                    && !stage_bytes.contains_key(&key_of("crc").unwrap_or_default())
                {
                    if let (Some(key), Some(bytes)) = (
                        key_of("crc"),
                        fields
                            .get("crc_bytes")
                            .and_then(|value| value.parse::<f64>().ok()),
                    ) {
                        stage_bytes.insert(key, bytes);
                    }
                }
                if kind == Some("issue") {
                    let records = fields
                        .get("records_read_per_choice")
                        .and_then(|value| value.parse::<f64>().ok());
                    let calls = fields
                        .get("choose_calls")
                        .and_then(|value| value.parse::<f64>().ok());
                    if let (Some(key), Some(records), Some(calls)) =
                        (key_of("choose"), records, calls)
                    {
                        stage_bytes
                            .insert(key, records * LOCAL_ALLOCATION_RECORD_BYTES as f64 * calls);
                    }
                }
            }
            Some(_) | None => {}
        }
    }
    (throughput, stage_bytes)
}

pub(super) fn measure_gpu_submission_segment(
    wall_clock_product: &Path,
    stage_split_product: &Path,
    card_indices: &[u32],
) -> i32 {
    let mut emitter = Emitter::new();
    emitter.line(&format!(
        "name=configuration segment=gpu_submission cards={} range_counts={} range_lengths={} timed_submissions={TIMED_SUBMISSIONS}",
        card_indices.iter().map(u32::to_string).collect::<Vec<_>>().join(","),
        RANGE_COUNTS.iter().map(u64::to_string).collect::<Vec<_>>().join(","),
        RANGE_LENGTHS.iter().map(u64::to_string).collect::<Vec<_>>().join(","),
    ));
    let mismatches = check_the_constants_and_the_anchors(&mut emitter);
    let (Ok(wall_clock_text), Ok(stage_split_text)) = (
        std::fs::read_to_string(wall_clock_product),
        std::fs::read_to_string(stage_split_product),
    ) else {
        emitter
            .line("name=verdict segment=gpu_submission status=not_run reason=products_unreadable");
        emitter.finish();
        return EXIT_CODE_USAGE;
    };
    let batch_bytes = batch_bytes_of_the_wall_clock_product(&result_lines_of(&wall_clock_text));
    let (throughput, stage_bytes) =
        readings_of_the_stage_split_product(&result_lines_of(&stage_split_text));
    // 先录后判：只录那一段造数据单元、算两种段长的 CPU 参照、写进库，到此收尾、不取卡；判段读回之后才取卡（计时窗口只包 GPU 提交，参照本来就在窗口外）
    let Some((read_back, cards_handed_over)) =
        cards_and_pairing_of_the_gpu_segments::submission_unit_read_back_before_the_cards(
            &one_data_unit_of_the_workload(),
            cards_and_pairing_of_the_gpu_segments::input_and_reference_through_the_pairing_library,
            || {
                cards_and_pairing_of_the_gpu_segments::card_indices_handed_over_to_the_segment(
                    card_indices,
                )
            },
        )
    else {
        emitter.line(&format!(
            "name=recorded_only segment=gpu_submission item={} constant_or_anchor_mismatches={mismatches}",
            cards_and_pairing_of_the_gpu_segments::SUBMISSION_PAIRING_ITEM,
        ));
        emitter.finish();
        return if mismatches > 0 {
            EXIT_CODE_STOP_CLAUSE
        } else {
            EXIT_CODE_OK
        };
    };
    let picked_card_indices = match cards_handed_over {
        Ok(picked) => picked,
        Err(refusal) => {
            return cards_and_pairing_of_the_gpu_segments::stop_without_the_cards_handed_over(
                "gpu_submission",
                &refusal,
                &mut emitter,
            )
        }
    };
    let unit = read_back.unit;
    let links = pcie_links_reported_by_nvidia_smi();
    let mut card_failures: Vec<String> = Vec::new();
    let mut cards_measured = 0_usize;
    let mut fixed_share_values = Vec::new();
    for card_index in &picked_card_indices {
        let Some(card) = gpu_cards_by_priority(&[(*card_index, CARD_QUOTA_MEBIBYTES)])
            .into_iter()
            .next()
        else {
            emitter.line(&format!(
                "name=gpu_card index={card_index} status=not_run reason=card_not_usable"
            ));
            continue;
        };
        let Some(link_peak) = links
            .get(card_index)
            .and_then(|(generation, width)| pcie_link_peak_bytes_per_second(*generation, *width))
        else {
            emitter.line(&format!(
                "name=gpu_card index={card_index} status=invalid reason=pcie_link_unreadable"
            ));
            card_failures.push(format!("card_{card_index}_link_unreadable"));
            continue;
        };
        cards_measured += 1;
        let binding_limit = card.device().limits().max_storage_buffer_binding_size;
        let check_string_result = card.crc32c_of_ranges(&[CHECK_STRING]);
        let check_string_holds = check_string_result == [CHECK_STRING_CRC32_CASTAGNOLI];
        emitter.line(&format!(
            "name=gpu_card index={card_index} card_name={} driver={} free_memory_mebibytes={} link_peak_bytes_per_second={link_peak:.0} max_storage_buffer_binding_size={binding_limit} check_string_crc32c={} check_string_holds={check_string_holds}", // naming-lint:external max_storage_buffer_binding_size 是 wgpu 的 Limits 字段名，照原名打，读产物的人拿它直接对 wgpu 文档
            field_token(&card.name),
            field_token(&card.driver_version),
            card.free_memory_mebibytes,
            check_string_result.first().map_or_else(|| "none".to_string(), |crc| format!("{crc:#010x}")),
        ));
        let mut is_the_card_valid = check_string_holds;
        let mut fit_points = Vec::new();
        let mut median_of_one_range = None;
        for (range_length, cpu_crc) in RANGE_LENGTHS
            .into_iter()
            .zip(read_back.cpu_crcs_by_range_length.iter().copied())
        {
            let segment = &unit[..usize::try_from(range_length).expect("段长")];
            for range_count in RANGE_COUNTS
                .iter()
                .copied()
                .filter(|count| count * LOCAL_GPU_SLOT_BYTES <= binding_limit)
            {
                let reading = time_the_submissions(&card, segment, cpu_crc, range_count, link_peak);
                is_the_card_valid &= reading.respects_the_floor && reading.crcs_agree;
                emitter.line(&format!(
                    "name=gpu_submission index={card_index} range_length={range_length} range_count={} input_bytes={} buffer_bytes={} median_seconds={:.9} respects_the_physical_floor={} crcs_agree={}",
                    reading.range_count,
                    reading.range_count * LOCAL_GPU_SLOT_BYTES,
                    bytes_crc32c_of_ranges_needs(usize::try_from(reading.range_count).expect("段数")),
                    reading.median_seconds,
                    reading.respects_the_floor,
                    reading.crcs_agree,
                ));
                if range_length == LOCAL_GPU_SLOT_BYTES {
                    fit_points.push((
                        (reading.range_count * LOCAL_GPU_SLOT_BYTES) as f64,
                        reading.median_seconds,
                    ));
                    if reading.range_count == 1 {
                        median_of_one_range = Some(reading.median_seconds);
                    }
                }
            }
        }
        if !is_the_card_valid {
            card_failures.push(format!("card_{card_index}_result_or_floor"));
        }
        let fit = least_squares_intercept_and_slope(&fit_points);
        let largest_two_slope = match fit_points.as_slice() {
            [.., (smaller_bytes, smaller_seconds), (larger_bytes, larger_seconds)] => {
                Some((larger_seconds - smaller_seconds) / (larger_bytes - smaller_bytes))
            }
            _ => None,
        };
        emitter.line(&format!(
            "name=q11_q12 index={card_index} card_valid={is_the_card_valid} intercept_seconds={} slope_seconds_per_byte={} one_range_median_seconds={} largest_two_points_slope_seconds_per_byte={}",
            optional_number(fit.map(|(intercept, _slope)| intercept)),
            fit.map_or_else(|| "none".to_string(), |(_intercept, slope)| format!("{slope:.6e}")),
            optional_number(median_of_one_range),
            largest_two_slope.map_or_else(|| "none".to_string(), |slope| format!("{slope:.6e}")),
        ));
        let Some((intercept, slope)) = fit.filter(|_fit| is_the_card_valid) else {
            continue;
        };
        for ((fraction, pool, batch, call_index), bytes) in &batch_bytes {
            let share = fixed_overhead_share(intercept, slope, *bytes);
            fixed_share_values.push(share);
            let merge = (share > FIXED_SHARE_AT_MOST_FOR_ONE_BATCH)
                .then(|| batches_to_merge(intercept, slope, *bytes));
            let merged_buffer_bytes = merge.map(|batches| {
                let units_per_batch = bytes / LOCAL_GPU_SLOT_BYTES;
                let merged_units =
                    format!("{batches:.0}").parse::<u64>().unwrap_or(0) * units_per_batch;
                bytes_crc32c_of_ranges_needs(usize::try_from(merged_units).expect("合并后的段数"))
            });
            emitter.line(&format!(
                "name=q13 index={card_index} fraction={fraction} pool_data_units={pool} batch_units={batch} call_index={call_index} batch_bytes={bytes} fixed_overhead_share={share:.6} judgement={} batches_to_merge={} merged_buffer_bytes={} card_free_bytes={}",
                judgement_of_fixed_share_at_the_registered_edges(share),
                merge.map_or_else(|| "none".to_string(), |batches| format!("{batches:.0}")),
                merged_buffer_bytes.map_or_else(|| "none".to_string(), |buffer| buffer.to_string()),
                card.free_memory_mebibytes * 1024 * 1024,
            ));
        }
        for ((fraction, pool, batch, stage), bytes) in &stage_bytes {
            let throughput_of = |stage_name: &str, threads: &str| {
                throughput
                    .get(&(
                        fraction.clone(),
                        pool.clone(),
                        batch.clone(),
                        stage_name.to_string(),
                        threads.to_string(),
                    ))
                    .copied()
            };
            let cpu_seconds_at = |threads: &str| -> Option<f64> {
                match stage.as_str() {
                    "crc" => Some(bytes / throughput_of("crc", threads)?),
                    "choose" => Some(bytes / throughput_of("choose", threads)?),
                    _ => None,
                }
            };
            let gpu_seconds = intercept + slope * bytes;
            let scan_increment_at = |threads: &str| -> Option<f64> {
                Some(
                    bytes / throughput_of("scan", threads)?
                        - bytes / throughput_of("crc", threads)?,
                )
            };
            for (stage_label, cpu_sixteen, cpu_one) in [
                (stage.clone(), cpu_seconds_at("16"), cpu_seconds_at("1")),
                (
                    "scan_increment".to_string(),
                    scan_increment_at("16"),
                    scan_increment_at("1"),
                ),
            ] {
                if stage_label == "scan_increment" && stage != "crc" {
                    continue;
                }
                let ratio_sixteen = cpu_sixteen.map(|cpu| cpu / gpu_seconds);
                let ratio_one = cpu_one.map(|cpu| cpu / gpu_seconds);
                let is_the_sixteen_thread_form_clause_triggered =
                    match (cpu_sixteen, cpu_one, ratio_sixteen, ratio_one) {
                        (Some(sixteen), Some(one), Some(ratio_at_sixteen), Some(ratio_at_one)) => {
                            one / sixteen < 2.0
                                || judgement_of_the_sixteen_thread_to_gpu_ratio(ratio_at_sixteen)
                                    != judgement_of_the_sixteen_thread_to_gpu_ratio(ratio_at_one)
                        }
                        (None, _, _, _) | (_, None, _, _) | (_, _, None, _) | (_, _, _, None) => {
                            false
                        }
                    };
                emitter.line(&format!(
                    "name=q9 index={card_index} fraction={fraction} pool_data_units={pool} batch_units={batch} stage={stage_label} stage_bytes={bytes:.0} cpu_sixteen_threads_seconds={} gpu_seconds={gpu_seconds:.9} ratio={} judgement={} cpu_one_thread_seconds={} ratio_one_thread={} judgement_one_thread={} clause_sixteen_thread_form_misfits_the_gc_triggered={is_the_sixteen_thread_form_clause_triggered}",
                    optional_number(cpu_sixteen),
                    optional_number(ratio_sixteen),
                    ratio_sixteen.map_or("not_run", judgement_of_the_sixteen_thread_to_gpu_ratio),
                    optional_number(cpu_one),
                    optional_number(ratio_one),
                    ratio_one.map_or("not_run", judgement_of_the_sixteen_thread_to_gpu_ratio),
                ));
            }
        }
    }
    let discrimination_holds = emit_the_discrimination(
        &mut emitter,
        "Q13",
        &fixed_share_values,
        |value, threshold| judgement_of_fixed_share(value, threshold, f64::INFINITY),
    );
    emitter.line(&format!(
        "name=verdict segment=gpu_submission constant_or_anchor_mismatches={mismatches} cards_measured={cards_measured} card_failures={} card_failure_items={} discrimination_holds={discrimination_holds}",
        card_failures.len(),
        if card_failures.is_empty() { "none".to_string() } else { card_failures.join(",") },
    ));
    emitter.finish();
    if mismatches > 0 {
        EXIT_CODE_STOP_CLAUSE
    } else if cards_measured == 0 {
        EXIT_CODE_NOT_RUN
    } else if !card_failures.is_empty() || !discrimination_holds {
        EXIT_CODE_POSITIVE_CONTROL_FAILED
    } else {
        EXIT_CODE_OK
    }
}
