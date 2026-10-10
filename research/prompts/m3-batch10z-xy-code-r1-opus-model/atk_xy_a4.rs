
// ---------- K1-1：上一轮那几族形在新接口上逐格重扫，连同被拒之后、要出路之前用户做的事 ----------

/// 单元区 `unit_area_slots` 槽（384 槽那一档的参数与环，只把单元区放大或放窄）的每块盘字节数。
fn atk_device_bytes_with_unit_area(unit_area_slots: u64) -> u64 {
    u64::try_from(
        i128::from(device_bytes_of_the_384_slot_small_devices())
            + (i128::from(unit_area_slots) - 384) * i128::from(singlefs_format::SLOT_BYTES),
    )
    .expect("盘宽为正")
}

/// 攻方上一轮 K1 那一形：前缀覆盖写 `pre` 次，文件每次长 `step` 个单元、长到 `fill` 个，再录一次长 3 个单元的顺序写 R，
/// 崩在 R 第一条记录之后。交回（R 之前的盘面、崩了的盘面、每条根的内容、之后的写入时刻）。
fn atk_k1_record_on(
    unit_area_slots: u64,
    pre: u64,
    fill: usize,
    step: usize,
) -> Option<(MemoryPool, MemoryPool, BTreeMap<(InstanceGeneration, CheckpointTxg), Vec<u8>>, u64)> {
    let unit_bytes = singlefs_core::unit::data_unit_payload_capacity();
    let geometry = fixed_geometry_of_the_384_slot_small_devices();
    let stream = SharedStream::retaining_contents();
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
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
    let (writes, _segments, _indexes) = writes_and_segments_with_stream_indexes(&operations[operations_before_r..], &geometry);
    let first_record_end = writes.iter().position(|write| write.kind == StepKind::JournalRecord)? + 1;
    let mut crashed = base.clone();
    crashed.apply_writes(&writes[..first_record_end]);
    Some((base, crashed, pool.content_of_each_root.clone(), pool.write_time_seconds + 100))
}

/// 推环入口，盘表照 `table` 摆（`swapped`：两块盘的身份对调；`reversed`：次序倒过来、身份与盘不拆）。
fn atk_push_with_table<Device: DeviceOverASparseImage>(pool: &mut PoolUnderTest<Device>, table: &str) -> String {
    let mut session = pool.session.take().expect("挂着的会话");
    match table {
        "swapped" => {
            let first = pool.devices[0].0;
            let second = pool.devices[1].0;
            pool.devices[0].0 = second;
            pool.devices[1].0 = first;
        }
        "reversed" => pool.devices.reverse(),
        _ => {}
    }
    let pushed = session.push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states(&mut pool.devices);
    match table {
        "swapped" => {
            let first = pool.devices[0].0;
            let second = pool.devices[1].0;
            pool.devices[0].0 = second;
            pool.devices[1].0 = first;
        }
        "reversed" => pool.devices.reverse(),
        _ => {}
    }
    pool.session = Some(session);
    match pushed {
        Ok(done) => {
            for version in &done.versions_published {
                pool.note_root_of_the_current_content(&version.root);
            }
            format!("pushed({})", done.versions_published.len())
        }
        Err(refusal) => format!("push_refused[{}]", atk_head(format!("{refusal:?}"))),
    }
}

/// 被拒之后、要出路之前用户做的一件事（`action`），做完照那一份旧报要出路、再照出路二真走（推环、再发同一个改动）逐格比。
/// 能做的：none、same（原样再发一次）、bigger（再长 5 个单元的写）、inode、overwrite、push-swapped、push-reversed、
/// unmount-remount、frozen-overwrite（一次覆盖写第一个改盘调用报块设备错、冻结）、push-fail-1、push-fail-12（推环真发时第 1 / 12 个改盘调用报错）、
/// ask-then-same（先要一次出路、原样再发、拿第二份报要出路，比两份报）。
/// 第二十四块：在被拒那一刻的一份数调用的拷贝上，盘表照 `table` 换（reversed：次序倒过来、身份与盘不拆；swapped：两块盘的身份对调；
/// one：只交第一块；stale：第二块换成 `older` 那一刻的盘面）再要出路，数这一次探物理块大小、读、改盘调用各几次，比盘面、分配器、现行那一版动没动。
fn atk_ask_with_table(after_report: &AtkSnap, older: Option<&MemoryPool>, table: &str, stuck: &StuckBehindTheWideProtection) -> Option<AtkAsk> {
    let (mut counting, calls) = pool_failing_one_changing_call_copied_from(&after_report.pool(), None);
    match table {
        "reversed" => counting.devices.reverse(),
        "swapped" => {
            let first = counting.devices[0].0;
            counting.devices[0].0 = counting.devices[1].0;
            counting.devices[1].0 = first;
        }
        "one" => counting.devices.truncate(1),
        "stale" => {
            let stale = plain_devices_on(older?).remove(1);
            counting.devices[1].1.inner = stale.1;
        }
        other => panic!("没有这种盘表：{other}"),
    }
    let image_before = counting.image();
    let allocator_before = format!("{:?}", counting.session().allocator);
    let current_before = format!("{:?}", counting.session().current);
    let result = counting.session().ways_out_of_the_wide_protection(&counting.devices, stuck);
    let untouched = counting.image() == image_before
        && format!("{:?}", counting.session().allocator) == allocator_before
        && format!("{:?}", counting.session().current) == current_before;
    Some(AtkAsk {
        result,
        probes: calls.probes_of_the_physical_block_size.get(),
        reads: calls.reads.get(),
        changing_calls: calls.calls.get(),
        untouched,
    })
}

fn atk_after_action(after_report: &AtkSnap, stuck: &StuckBehindTheWideProtection, change: &AtkChange, action: &str) -> (String, Option<bool>) {
    atk_after_action_with_older(after_report, None, stuck, change, action)
}

/// 同上；`older` 是被拒之前某一刻的盘面（`table-stale`：要出路与照出路走都交第二块盘的那一份旧盘面）。
/// 另几件改的是要出路与照出路走交进来的盘表（两边交同一份）：table-reversed（次序倒过来、身份与盘不拆）、table-swapped（两块盘的身份对调）、
/// table-one（只交第一块盘）、table-stale。
