//! 第 4 次跑的取号崩溃家族、合法状态与故障状态家族。

use super::*;

/// H1g（G0）：P(n1, C) → a 次挂载崩在取号之后（这一臂自己的副本上）→ A（不断，W）→ m 尾巴。
pub(super) fn run_fourth_run_acquisition_crash_family(arm: FourthRunArm) {
    let family = FourthRunFamily {
        name: "h1g",
        geometry: &GEOMETRY_PRIMARY,
        hidden_count: 1,
        continuation: FourthRunContinuation::Tail,
        tail_overwrites: &FOURTH_RUN_ACQUISITION_CRASH_TAIL_OVERWRITES,
        durations: FourthRunDurationPlan::WholeCallOnly,
        checks_the_clauses_on_the_hidden_root:
            ClauseChecksOnTheHiddenRoot::NotRegisteredForThisFamily,
    };
    let mut prefixes = Vec::new();
    for overwrites in FOURTH_RUN_ACQUISITION_CRASH_PREFIX_OVERWRITES {
        for acquisition_crashes in FOURTH_RUN_ACQUISITION_CRASHES {
            prefixes.push(
                fourth_run_acquisition_crash_prefix(
                    &GEOMETRY_PRIMARY,
                    overwrites,
                    acquisition_crashes,
                )
                .map(|(prefix, checks)| {
                    emit_result(&format!(
                        "name=r4_h1g_acquisition_crash arm={} {} checks={}",
                        arm.name,
                        prefix.coordinates,
                        checks.join(",")
                    ));
                    prefix
                }),
            );
        }
    }
    run_fourth_run_family(&family, arm, prefixes);
}

// ---------------------------------------------------------------------------
// 12.9 第四段：L0–L6（第 3 次跑 5.3 那几行，合法状态与第 22 条那一格）；PC-多拒、PC-22、PC-554 的格（跨臂那几句在 compare 里判）。
//      每一格打一行 `r4_l_cell`：被测挂载的结局、N_真、读数、起始镜像指纹（Q4、Q5-同成 的输入），L5 另打隔离总数与每一片的读数（Q6）。
// ---------------------------------------------------------------------------

/// L 族与第四、五段那几格的账：第 3 次跑那一份汇总（7.3 第一行、F13 的点名），加每族每个几何点实际跑到被测挂载的格数（7.2）。
#[derive(Default)]
pub(super) struct FourthRunLegalAndFaultStateTally {
    pub(super) counts: ThirdRunFamilySummary,
    pub(super) cells: BTreeMap<String, u64>,
}

/// 一格 L 族被测挂载打一行。
#[allow(
    clippy::too_many_arguments,
    reason = "一格的族、坐标、起始镜像、被测挂载与故障原样打进结果行，拆成结构体只多一层搬运"
)]
fn emit_fourth_run_legal_or_fault_state_cell(
    family: &str,
    arm: FourthRunArm,
    geometry: &Geometry,
    coordinates: &str,
    base: &MemoryPool,
    tested: &FourthRunTestedMount,
    faults: &[ThirdRunFault],
    extra: &str,
) {
    let (write_fingerprint, _) = write_fingerprint_of(&tested.call.call.applied, 2);
    emit_result(&format!(
        "name=r4_l_cell family={family} arm={} geometry={} {coordinates} base_fingerprint={:016x} write_fingerprint={write_fingerprint:016x} faults={} unintercepted={} refused={} truth_newer_on_the_twin={} {} {extra}",
        arm.name,
        geometry.label,
        image_fingerprint(base),
        faults.len(),
        tested.unintercepted.join(","),
        tested.call.call.outcome.is_err(),
        tested.judgement.truth_newer,
        render_fourth_run_tested_mount(tested)
    ));
}

/// 在装置的账上记下一次改内容的调用写了哪些（第 3 次跑 `third_run_change_content` 同一串调用，经受观测的设备走），交回写序列。
fn fourth_run_change_content_recorded(
    history: &mut ThirdRunHistory,
    parameters: &MakeFilesystemParameters,
) -> Result<Vec<AppliedDeviceStep>, String> {
    let content_number = history.last_confirmed_content_number() + 1;
    let content = third_run_content(content_number);
    let mut session = history.session.take().ok_or("改内容：没有开着的会话")?;
    let base = history.pool.clone();
    let call = fourth_run_call(&base, &[], None, None, |devices| {
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        let file = FirstFile {
            content: &content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 * content_number,
        };
        match &session.current {
            PoolVersion::WithFile(previous) | PoolVersion::WithTreesAndNoFile(previous) => {
                publish_overwrite(
                    &mut writer,
                    &mut session.allocator,
                    previous,
                    file,
                    session.instance,
                )
                .map_err(|error| format!("覆盖写: {error:?}"))
            }
            PoolVersion::WithoutFile(version) => publish_first_file(
                &mut writer,
                &mut session.allocator,
                &version.root,
                file,
                session.instance,
                session.current.record_bytes(),
            )
            .map_err(|error| format!("首个文件: {error:?}")),
        }
    });
    let output = call.call.outcome?;
    history.pool = call.call.pool_after;
    let root_record = output.root;
    session.current = PoolVersion::WithFile(std::sync::Arc::new(output));
    history.session = Some(session);
    record_confirmation(history, &root_record, content_number)?;
    Ok(call.call.applied)
}

/// 前 `count` 个写的崩溃状态里「写」的个数（屏障不数）。
pub(super) fn writes_in(applied: &[AppliedDeviceStep]) -> usize {
    applied
        .iter()
        .filter(|step| !matches!(step, AppliedDeviceStep::Barrier { .. }))
        .count()
}

pub(super) fn legal_or_fault_state_prefix(
    geometry: &Geometry,
    overwrites: u64,
    arm: FourthRunArm,
) -> Result<ThirdRunHistory, String> {
    Ok(fourth_run_plain_prefix(geometry, overwrites, Closing::ProcessExit, arm.name)?.history)
}

/// 一格 L 族被测挂载：在 `base` 上装 `faults` 跑这一臂的可写挂载，打一行。
#[allow(
    clippy::too_many_arguments,
    reason = "一格的族、坐标、起始镜像与故障原样打进结果行，拆成结构体只多一层搬运"
)]
pub(super) fn run_fourth_run_legal_or_fault_state_cell(
    family: &str,
    arm: FourthRunArm,
    geometry: &Geometry,
    coordinates: &str,
    base: &MemoryPool,
    faults: &[ThirdRunFault],
    extra: &str,
    summary: &mut FourthRunLegalAndFaultStateTally,
) -> Option<FourthRunTestedMount> {
    *summary
        .cells
        .entry(format!("{family}:{}", geometry.label))
        .or_insert(0) += 1;
    let parameters = parameters_for(geometry);
    match run_fourth_run_tested_mount(
        arm,
        base,
        &parameters,
        faults,
        coordinates,
        &mut summary.counts,
    ) {
        Ok(tested) => {
            emit_fourth_run_legal_or_fault_state_cell(
                family,
                arm,
                geometry,
                coordinates,
                base,
                &tested,
                faults,
                extra,
            );
            Some(tested)
        }
        Err(error) => {
            emit_result(&format!(
                "name=r4_l_cell family={family} arm={} geometry={} {coordinates} verdict=stop reason={error:?}",
                arm.name, geometry.label
            ));
            None
        }
    }
}

/// L0（P(n1, c)，不注入）与 L1（崩溃状态上不注入）。
pub(super) fn run_fourth_run_legal_state_families(
    geometry: &Geometry,
    arm: FourthRunArm,
    summary: &mut FourthRunLegalAndFaultStateTally,
) {
    let parameters = parameters_for(geometry);
    for overwrites in THIRD_RUN_OVERWRITES {
        for closing in [Closing::ProcessExit, Closing::NormalUnmount] {
            let coordinates = format!("n1={overwrites} c={}", closing.name());
            match fourth_run_plain_prefix(geometry, overwrites, closing, arm.name) {
                Ok(prefix) => {
                    run_fourth_run_legal_or_fault_state_cell(
                        "l0",
                        arm,
                        geometry,
                        &coordinates,
                        &prefix.history.pool,
                        &[],
                        "",
                        summary,
                    );
                }
                Err(error) => emit_result(&format!(
                    "name=r4_l_cell family=l0 arm={} {coordinates} verdict=stop reason={error:?}",
                    arm.name
                )),
            }
        }
        // L1(a)：P(n1, C) 最后一次改内容的发布的写序列上，前 j 个写落盘的崩溃状态（j 取全部真前缀）。
        let history_before_the_last_change = if overwrites == 0 {
            third_run_history_start_without_the_first_file(geometry)
        } else {
            third_run_history_start(geometry, overwrites - 1)
        };
        match history_before_the_last_change.and_then(|mut history| {
            let before = history.pool.clone();
            let applied = fourth_run_change_content_recorded(&mut history, &parameters)?;
            Ok((before, applied))
        }) {
            Ok((before, applied)) => {
                for writes in 0..writes_in(&applied) {
                    let crashed = image_with_the_first_writes(&before, &applied, writes);
                    run_fourth_run_legal_or_fault_state_cell(
                        "l1a",
                        arm,
                        geometry,
                        &format!("n1={overwrites} point={writes} of={}", writes_in(&applied)),
                        &crashed,
                        &[],
                        "",
                        summary,
                    );
                }
            }
            Err(error) => emit_result(&format!(
                "name=r4_l_cell family=l1a arm={} n1={overwrites} verdict=stop reason={error:?}",
                arm.name
            )),
        }
        // L1(b)：P(n1, C) 之后一次可写挂载的写序列（今天那一臂上录的，r4-bases 存了每个崩溃状态）。
        let mut point = 0usize;
        loop {
            let key = format!(
                "l1b-s{}-ring{}-n1{overwrites}-point{point}",
                geometry.slots_per_region, geometry.journal_ring_bytes
            );
            let Ok(base) = load_today_base(&key) else {
                if point == 0 {
                    emit_result(&format!(
                        "name=r4_l_cell family=l1b arm={} n1={overwrites} verdict=stop reason=\"今天那一臂的起始镜像读不出（先跑 r4-bases）\"",
                        arm.name
                    ));
                }
                break;
            };
            run_fourth_run_legal_or_fault_state_cell(
                "l1b",
                arm,
                geometry,
                &format!("n1={overwrites} point={point}"),
                &base.image,
                &[],
                "",
                summary,
            );
            point += 1;
        }
        // L1(c)：P(n1, U) 的卸载写序列上前 j 个写落盘的崩溃状态。
        match third_run_history_start(geometry, overwrites).and_then(|mut history| {
            let before = history.pool.clone();
            let applied = fourth_run_unmount_recorded(&mut history, &parameters)?;
            Ok((before, applied))
        }) {
            Ok((before, applied)) => {
                for writes in 0..writes_in(&applied) {
                    let crashed = image_with_the_first_writes(&before, &applied, writes);
                    run_fourth_run_legal_or_fault_state_cell(
                        "l1c",
                        arm,
                        geometry,
                        &format!("n1={overwrites} point={writes} of={}", writes_in(&applied)),
                        &crashed,
                        &[],
                        "",
                        summary,
                    );
                }
            }
            Err(error) => emit_result(&format!(
                "name=r4_l_cell family=l1c arm={} n1={overwrites} verdict=stop reason={error:?}",
                arm.name
            )),
        }
    }
}

/// P(n1, ·) 走到「mkfs → 可写挂载」为止、首个文件之前（L1(a) 在 n1 = 0 上要首个文件那一次发布自己的写序列）。
fn third_run_history_start_without_the_first_file(
    geometry: &Geometry,
) -> Result<ThirdRunHistory, String> {
    let parameters = parameters_for(geometry);
    let mut devices = new_devices(IMAGE_BYTES);
    let genesis =
        make_filesystem(&parameters, &mut devices).map_err(|error| format!("mkfs: {error:?}"))?;
    let mounted = mount_writable(&parameters, &mut devices)
        .map_err(|error| format!("第一次可写挂载: {error:?}"))?;
    let mut history = ThirdRunHistory {
        pool: memory_pool_of(&devices, IMAGE_BYTES),
        session: None,
        timeline: vec![root_pair(&genesis.root)],
        content_number_by_root: BTreeMap::from([(root_pair(&genesis.root), 0)]),
        confirmations: Vec::new(),
        tail_checks: 0,
        tail_check_failures: Vec::new(),
    };
    push_the_publishes_of_a_mount(&mut history, &mounted, 0);
    history.session = Some(Session {
        allocator: mounted.allocator,
        current: mounted.current,
        instance: mounted.output.instance,
    });
    Ok(history)
}

/// 正常卸载（经受观测的设备走），交回写序列（L1(c)）。
fn fourth_run_unmount_recorded(
    history: &mut ThirdRunHistory,
    parameters: &MakeFilesystemParameters,
) -> Result<Vec<AppliedDeviceStep>, String> {
    let Session {
        mut allocator,
        mut current,
        ..
    } = history.session.take().ok_or("卸载：没有开着的会话")?;
    let call = fourth_run_call(&history.pool, &[], None, None, |devices| {
        singlefs_core::mount::unmount(
            parameters,
            devices,
            &mut allocator,
            &mut current,
            ShadowLedger::On,
        )
        .map(|_| ())
        .map_err(|error| format!("正常卸载: {error:?}"))
    });
    call.call.outcome?;
    history.pool = call.call.pool_after;
    Ok(call.call.applied)
}

/// 第 k 新的根（按时间线）根槽的落点，W 读坏（L2、PC-多拒 的 -槽 那一格）。
fn older_root_slot_faults(
    history: &ThirdRunHistory,
    geometry: &PoolGeometry,
    newest_rank: usize,
    form: UnreadableForm,
) -> Option<Vec<ThirdRunFault>> {
    let root = *history.timeline.iter().rev().nth(newest_rank - 1)?;
    let (device, offset) = root_slot_of(&history.pool, geometry, root)?;
    Some(vec![ThirdRunFault {
        device,
        offset,
        length: u64::from(geometry.physical_block_size),
        form,
        duration: FaultDuration::WholeCall,
        label: format!("root{}", key_text(Some(root))),
    }])
}

/// L3 的三种：(a) 最新那次发布的末条记录两份；(b) 全部记录两份；(c) 末条记录盘 0 那一份。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RecordFaultChoice {
    LastRecordBothCopies,
    EveryRecordBothCopies,
    LastRecordDeviceZeroCopy,
}

impl RecordFaultChoice {
    pub(super) fn name(self) -> &'static str {
        match self {
            RecordFaultChoice::LastRecordBothCopies => "a",
            RecordFaultChoice::EveryRecordBothCopies => "b",
            RecordFaultChoice::LastRecordDeviceZeroCopy => "c",
        }
    }
}

fn record_faults(
    history: &ThirdRunHistory,
    geometry: &PoolGeometry,
    choice: RecordFaultChoice,
    form: UnreadableForm,
) -> Option<Vec<ThirdRunFault>> {
    let records = records_of_publish(&history.pool, geometry, history.tip());
    let last_counter = records.iter().map(|(counter, _, _)| *counter).max()?;
    let chosen: Vec<&(u64, u64, JournalRecord)> = records
        .iter()
        .filter(|(counter, _, _)| match choice {
            RecordFaultChoice::LastRecordBothCopies
            | RecordFaultChoice::LastRecordDeviceZeroCopy => *counter == last_counter,
            RecordFaultChoice::EveryRecordBothCopies => true,
        })
        .collect();
    let devices: &[u32] = match choice {
        RecordFaultChoice::LastRecordDeviceZeroCopy => &[0],
        RecordFaultChoice::LastRecordBothCopies | RecordFaultChoice::EveryRecordBothCopies => {
            &[0, 1]
        }
    };
    let mut faults = Vec::new();
    for (counter, offset, _) in chosen {
        for device in devices {
            faults.push(ThirdRunFault {
                device: DeviceIdentity(*device),
                offset: *offset,
                length: JOURNAL_RECORD_BYTES,
                form,
                duration: FaultDuration::WholeCall,
                label: format!("record(counter={counter},device={device})"),
            });
        }
    }
    Some(faults)
}

/// L4：世代号最新的系统配置槽：(a) 盘 0 那一槽；(b) 两块盘各自那一槽。
fn system_configuration_faults(
    pool: &MemoryPool,
    geometry: &PoolGeometry,
    both_devices: bool,
    form: UnreadableForm,
) -> Vec<ThirdRunFault> {
    let slot_spacing = geometry.slot_spacing;
    let devices: &[u32] = if both_devices { &[0, 1] } else { &[0] };
    let mut faults = Vec::new();
    for device in devices {
        let newest = [0u64, slot_spacing]
            .into_iter()
            .filter_map(|offset| {
                let bytes = pool.read(*device, offset, 4096)?;
                singlefs_core::system_configuration::SystemConfiguration::parse_slot(&bytes)
                    .ok()
                    .map(|parsed| (parsed.quantities.slot_generation, offset))
            })
            .max_by_key(|(generation, _)| *generation);
        if let Some((generation, offset)) = newest {
            faults.push(ThirdRunFault {
                device: DeviceIdentity(*device),
                offset,
                length: SYSTEM_CONFIGURATION_SLOT_BYTES_LOCAL,
                form,
                duration: FaultDuration::WholeCall,
                label: format!("system_configuration(device={device},generation={generation})"),
            });
        }
    }
    faults
}

/// L2（第 k 新的根槽读坏，k ∈ {2, 3}）、L3（记录读坏三种）、L4（系统配置读坏两种）、L6（最新根槽读坏、链完整），两种造法。
pub(super) fn run_fourth_run_fault_state_families(
    geometry: &Geometry,
    arm: FourthRunArm,
    families: &[&str],
    summary: &mut FourthRunLegalAndFaultStateTally,
) {
    for overwrites in THIRD_RUN_OVERWRITES {
        let history = match legal_or_fault_state_prefix(geometry, overwrites, arm) {
            Ok(history) => history,
            Err(error) => {
                emit_result(&format!(
                    "name=r4_l_cell family=l arm={} n1={overwrites} verdict=stop reason={error:?}",
                    arm.name
                ));
                continue;
            }
        };
        let Ok(pool_geometry) = independent_geometry(&history.pool) else {
            continue;
        };
        let hidden = history.tip();
        for form in [UnreadableForm::ReadFails, UnreadableForm::ReadsZeros] {
            if families.contains(&"l2") {
                for newest_rank in [2usize, 3] {
                    let coordinates =
                        format!("n1={overwrites} k={newest_rank} form={}", form.name());
                    match older_root_slot_faults(&history, &pool_geometry, newest_rank, form) {
                        Some(faults) => {
                            run_fourth_run_legal_or_fault_state_cell("l2", arm, geometry, &coordinates, &history.pool, &faults, "", summary);
                        }
                        None => emit_result(&format!(
                            "name=r4_l_cell family=l2 arm={} geometry={} {coordinates} verdict=not_applicable reason=\"时间线上的根不够 k 条\"",
                            arm.name, geometry.label
                        )),
                    }
                }
            }
            if families.contains(&"l3") {
                for choice in [
                    RecordFaultChoice::LastRecordBothCopies,
                    RecordFaultChoice::EveryRecordBothCopies,
                    RecordFaultChoice::LastRecordDeviceZeroCopy,
                ] {
                    let coordinates = format!(
                        "n1={overwrites} which={} form={}",
                        choice.name(),
                        form.name()
                    );
                    match record_faults(&history, &pool_geometry, choice, form) {
                        Some(faults) => {
                            run_fourth_run_legal_or_fault_state_cell("l3", arm, geometry, &coordinates, &history.pool, &faults, "", summary);
                        }
                        None => emit_result(&format!(
                            "name=r4_l_cell family=l3 arm={} geometry={} {coordinates} verdict=not_applicable reason=\"最新那次发布没有记录\"",
                            arm.name, geometry.label
                        )),
                    }
                }
            }
            if families.contains(&"l4") {
                for (which, both_devices) in [("a", false), ("b", true)] {
                    let coordinates = format!("n1={overwrites} which={which} form={}", form.name());
                    let faults = system_configuration_faults(
                        &history.pool,
                        &pool_geometry,
                        both_devices,
                        form,
                    );
                    run_fourth_run_legal_or_fault_state_cell(
                        "l4",
                        arm,
                        geometry,
                        &coordinates,
                        &history.pool,
                        &faults,
                        "",
                        summary,
                    );
                }
            }
            if families.contains(&"l6") {
                let coordinates = format!("n1={overwrites} form={}", form.name());
                if let Some(faults) = older_root_slot_faults(&history, &pool_geometry, 1, form) {
                    if let Some(tested) = run_fourth_run_legal_or_fault_state_cell(
                        "l6",
                        arm,
                        geometry,
                        &coordinates,
                        &history.pool,
                        &faults,
                        "",
                        summary,
                    ) {
                        // 7.1 第三行（F5，今天那一臂）：根槽读不出而记录与单元都读得出时，恢复由记录重建那次发布的根。
                        if arm.is_today() {
                            let holds = tested.summary.effective == Some(hidden);
                            emit_result(&format!(
                                "name=r4_section_seven_one_f5 arm={} {coordinates} verdict={} effective={} hidden={}",
                                arm.name,
                                if holds { "pass" } else { "fail_f5" },
                                key_text(tested.summary.effective),
                                key_text(Some(hidden))
                            ));
                        }
                    }
                }
            }
        }
    }
}
