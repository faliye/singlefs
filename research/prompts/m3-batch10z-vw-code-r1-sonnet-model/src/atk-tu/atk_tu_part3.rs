
/// 单元区 `unit_area_slots` 槽（384 槽那一档的参数与环，只把单元区放大）的每块盘字节数与按落点分写要的几何。
fn atk_device_bytes_with_unit_area(unit_area_slots: u64) -> u64 {
    u64::try_from(
        i128::from(device_bytes_of_the_384_slot_small_devices())
            + (i128::from(unit_area_slots) - 384) * i128::from(singlefs_format::SLOT_BYTES),
    )
    .expect("盘宽为正")
}

/// 攻方上一轮 K1 那一形（`atk_k1_record`），单元区放开：前缀覆盖写 `pre` 次，文件每次长 `step` 个单元、长到 `fill` 个，
/// 再录一次长 3 个单元的顺序写 R，崩在 R 第一条记录之后。交回（R 之前的盘面、崩了的盘面、每条根的内容、之后的写入时刻）。
fn atk_k1_record_on(
    unit_area_slots: u64,
    pre: u64,
    fill: usize,
    step: usize,
) -> Option<(MemoryPool, MemoryPool, BTreeMap<(InstanceGeneration, CheckpointTxg), Vec<u8>>, u64)> {
    let unit_bytes = singlefs_core::unit::data_unit_payload_capacity();
    let parameters = parameters_of_the_384_slot_small_devices();
    let geometry = fixed_geometry_of_the_384_slot_small_devices();
    let stream = SharedStream::retaining_contents();
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters,
        atk_device_bytes_with_unit_area(unit_area_slots),
        |identity, device| RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
    );
    for _ in 0..pre {
        pool.overwrite(OVERWRITE_BYTES).expect("前缀覆盖写");
    }
    let mut units = 1usize;
    while units + step <= fill {
        match pool.sequential_write((units + step) * unit_bytes) {
            Ok(_) => units += step,
            Err(_) => break,
        }
    }
    let base = pool.image();
    let operations_before_r = stream.operation_count();
    match pool.sequential_write((units + 3) * unit_bytes) {
        Ok(published) if published.floor_raises.is_empty() => {}
        _ => return None,
    }
    let operations = stream.retained_operations();
    let (writes, _segments, _indexes) =
        writes_and_segments_with_stream_indexes(&operations[operations_before_r..], &geometry);
    let first_record_end = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::JournalRecord)
        .map(|(index, _)| index + 1)
        .next()?;
    let mut crashed = base.clone();
    crashed.apply_writes(&writes[..first_record_end]);
    Some((base, crashed, pool.content_of_each_root.clone(), pool.write_time_seconds + 100))
}

fn atk_env_list<T: std::str::FromStr>(name: &str, default: &str) -> Vec<T>
where
    T::Err: std::fmt::Debug,
{
    std::env::var(name)
        .unwrap_or_else(|_| default.to_string())
        .split(',')
        .map(|text| text.parse().expect("数"))
        .collect()
}

/// K1-1：攻方上一轮 9 形（前缀覆盖写 × 文件长到几个单元）× 长出去的单元数 g，C 臂上每个 g：先发一次报什么；报「被宽护卡住」的，
/// 照出路二真走一遍（推环、再发逐字节同一个改动一次）与预演逐格比（v0）；不推环再发一次看第二报与盘面（v1）；
/// 设了 ATK_SMALL 的，再放开「先做 k 次小动作、再发、照那一报走」（v2）。
#[test]
#[ignore = "攻方原型，手动跑"]
fn atk_k1_rehearsal_vs_real_sweep() {
    let pres: Vec<u64> = atk_env_list("ATK_PRES", "2,4,8");
    let fills: Vec<usize> = atk_env_list("ATK_FILLS", "6,11,16,21");
    let extra: usize = std::env::var("ATK_EXTRA").map_or(8, |text| text.parse().expect("数"));
    let smalls: Vec<String> = std::env::var("ATK_SMALL").map_or_else(|_| Vec::new(), |text| text.split(',').map(str::to_string).collect());
    let ks: Vec<usize> = atk_env_list("ATK_KS", "1,2,3,4,6");
    let unit_bytes = singlefs_core::unit::data_unit_payload_capacity();
    let parameters = parameters_of_the_384_slot_small_devices();
    let area: u64 = std::env::var("ATK_AREA").map_or(384, |text| text.parse().expect("数"));
    let device_bytes = atk_device_bytes_with_unit_area(area);
    let (mut cells, mut stuck_cells, mut disagree) = (0usize, 0usize, 0usize);
    let mut rehearsal_kinds: BTreeMap<String, usize> = BTreeMap::new();
    let (mut small_cells, mut small_disagree) = (0usize, 0usize);
    for pre in &pres {
        for fill in &fills {
            let label = format!("pre={pre} fill={fill}");
            let Some((base, crashed, contents, write_time_seconds)) = atk_k1_record_on(area, *pre, *fill, 5) else {
                println!("ATK k1 {label} r_not_recorded");
                continue;
            };
            let first_refused_of = |image: &MemoryPool| -> (usize, Option<usize>) {
                let (probe, _) = atk_mounted_pool_from(&parameters, device_bytes, image, &contents, write_time_seconds).expect("挂");
                let size_now = probe.content_of_the_current_version.len().div_ceil(unit_bytes);
                for grow in 1..=90usize {
                    let (mut trial, _) = atk_mounted_pool_from(&parameters, device_bytes, image, &contents, write_time_seconds).expect("挂");
                    if trial.sequential_write((size_now + grow) * unit_bytes).is_err() {
                        return (size_now, Some(grow));
                    }
                }
                (size_now, None)
            };
            let (size_now, g_ctrl) = first_refused_of(&base);
            let (_, g_c) = first_refused_of(&crashed);
            println!("ATK k1 {label} size_now={size_now} g_ctrl={g_ctrl:?} g_c={g_c:?}");
            let (Some(g_ctrl), Some(g_c)) = (g_ctrl, g_c) else { continue };
            for grow in g_c..=(g_ctrl + extra) {
                cells += 1;
                let (mut pool, _) = atk_mounted_pool_from(&parameters, device_bytes, &crashed, &contents, write_time_seconds).expect("挂");
                let write = atk_write_of(&mut pool, size_now + grow);
                let outcome = atk_apply(&mut pool, &write);
                let ctrl = if grow >= g_ctrl { "refused" } else { "admitted" };
                let Some(stuck) = atk_stuck_of(&outcome) else {
                    println!("ATK k1 {label} g={grow} ctrl={ctrl} first={}", atk_kind(&outcome));
                    continue;
                };
                stuck_cells += 1;
                *rehearsal_kinds.entry(atk_rehearsal_head(stuck)).or_insert(0) += 1;
                let after_report = AtkSnap::of(&pool);
                let (line, ok) = atk_follow_way_two(&after_report, stuck, &write);
                if !ok {
                    disagree += 1;
                }
                let repeat = atk_repeat_without_push(&after_report, &write);
                println!("ATK k1 {label} g={grow} ctrl={ctrl} first=stuck {line} {repeat}");
                for action in &smalls {
                    for k in &ks {
                        let (small_line, small_ok) = atk_small_actions_then_report_and_follow(&after_report, size_now + grow, action, *k);
                        if let Some(small_ok) = small_ok {
                            small_cells += 1;
                            if !small_ok {
                                small_disagree += 1;
                            }
                        }
                        println!("ATK k1small {label} g={grow} {small_line}");
                    }
                }
            }
        }
    }
    println!("ATK k1 totals area={area} cells={cells} stuck={stuck_cells} disagree={disagree} small_stuck={small_cells} small_disagree={small_disagree} rehearsal_kinds={rehearsal_kinds:?}");
}
