//! 第 4 次跑的前缀（平的、重挂、取号崩溃）、抛弃故障与家族、切点格与尾部格。

use super::*;

// ---------------------------------------------------------------------------
// 12.4 起始镜像（被测那次挂载之前）与抛弃步 A（第 4 次跑）：P(n1, c)（第 3 次跑 5.3）、H1f 的「中间夹一次重挂载」、
//      H1g 的「a 次挂载崩在取号之后」（这 a 次在这一臂自己的副本上做，登记 5.3 H1g 行）；A 藏最新 k 条根（k = 2 是 8.2 的敏感性点）。
// ---------------------------------------------------------------------------

/// 本地常量（登记 5.6；开跑时回比 `crates/`，`fourth_run_local_constants_checks`）：根环区域数；稀疏盘的扇区宽。
pub(super) const ROOT_RING_REGIONS_LOCAL: u64 = 3;
pub(super) const SECTOR_BYTES_LOCAL: u64 = 512;
/// H1f（登记 5.3）：n1a ∈ {0, 1}，c ∈ {C, U}，n1b ∈ {1, 2}。
pub(super) const FOURTH_RUN_REMOUNT_PREFIX_FIRST_OVERWRITES: [u64; 2] = [0, 1];
pub(super) const FOURTH_RUN_REMOUNT_PREFIX_SECOND_OVERWRITES: [u64; 2] = [1, 2];
/// H1g（登记 5.3）：n1 ∈ {1, 2}，a ∈ {1, 2}，m ∈ {1, 2}。
pub(super) const FOURTH_RUN_ACQUISITION_CRASH_PREFIX_OVERWRITES: [u64; 2] = [1, 2];
pub(super) const FOURTH_RUN_ACQUISITION_CRASHES: [u64; 2] = [1, 2];
pub(super) const FOURTH_RUN_ACQUISITION_CRASH_TAIL_OVERWRITES: [u64; 2] = [1, 2];
/// 一块盘上取号写的槽数（D23（journal 的角色与格式） 已定项 16：取号逐盘写一个系统配置槽；G0 两块盘 ⇒ 取号写 2 个）。
const ACQUISITION_WRITES_PER_DEVICE_LOCAL: usize = 1;

/// 一份起始镜像：坐标（进结果行）与装置的账（镜像在 `history.pool`，会话已关）。
pub(super) struct FourthRunPrefix {
    pub(super) coordinates: String,
    pub(super) history: ThirdRunHistory,
}

/// P(n1, c)（第 3 次跑 5.3）：mkfs → `mount_writable` → 首个文件 → n1 次覆盖写 → 关闭 c。关闭 U 那几格的 F14 判定行另打。
pub(super) fn fourth_run_plain_prefix(
    geometry: &Geometry,
    overwrites: u64,
    closing: Closing,
    arm: &str,
) -> Result<FourthRunPrefix, String> {
    let parameters = parameters_for(geometry);
    let mut history = third_run_history_start(geometry, overwrites)?;
    let coordinates = format!("n1={overwrites} c={}", closing.name());
    if let Some((holds, detail)) = third_run_close(&mut history, &parameters, closing)? {
        emit_result(&format!(
            "name=r4_section_seven_one_f14 arm={arm} geometry={} {coordinates} verdict={} {detail}",
            geometry.label,
            if holds { "pass" } else { "fail_f14" }
        ));
    }
    Ok(FourthRunPrefix {
        coordinates,
        history,
    })
}

/// 在装置的账上做一次可写挂载（不注入），把它写下的根记进时间线（沿用 E 的内容号），会话开着。
pub(super) fn fourth_run_mount_writable_into_the_history(
    history: &mut ThirdRunHistory,
    parameters: &MakeFilesystemParameters,
) -> Result<(), String> {
    let mut devices = devices_from_pool(&history.pool, IMAGE_BYTES);
    let mounted = mount_writable(parameters, &mut devices)
        .map_err(|error| format!("前缀里的可写挂载: {error:?}"))?;
    history.pool = memory_pool_of(&devices, IMAGE_BYTES);
    let carried = history
        .content_number_of(root_pair(&mounted.output.effective_root))
        .unwrap_or(0);
    push_the_publishes_of_a_mount(history, &mounted, carried);
    history.session = Some(Session {
        allocator: mounted.allocator,
        current: mounted.current,
        instance: mounted.output.instance,
    });
    Ok(())
}

/// H1f 的前缀（登记 5.3 H1f 行）：mkfs → `mount_writable` → 首个文件 → n1a 次覆盖写 → 关闭 c → `mount_writable` → n1b 次覆盖写 → 关闭 C。
/// 前缀在这一臂自己的进程里造：N 为假时各臂与今天同一串写（PC-N0、F12），-配置续 第二次取号写的 tail 按臂的定义不同；
/// 各臂前缀的镜像指纹打进结果行，`r4-compare` 跨臂核（执行员修订第 7 条）。
pub(super) fn fourth_run_remount_prefix(
    geometry: &Geometry,
    first_overwrites: u64,
    closing: Closing,
    second_overwrites: u64,
    arm: &str,
) -> Result<FourthRunPrefix, String> {
    let parameters = parameters_for(geometry);
    let mut history = third_run_history_start(geometry, first_overwrites)?;
    let coordinates = format!(
        "n1a={first_overwrites} c={} n1b={second_overwrites}",
        closing.name()
    );
    if let Some((holds, detail)) = third_run_close(&mut history, &parameters, closing)? {
        emit_result(&format!(
            "name=r4_section_seven_one_f14 arm={arm} geometry={} {coordinates} verdict={} {detail}",
            geometry.label,
            if holds { "pass" } else { "fail_f14" }
        ));
    }
    fourth_run_mount_writable_into_the_history(&mut history, &parameters)?;
    for _ in 0..second_overwrites {
        third_run_change_content(&mut history, &parameters)?;
    }
    third_run_close(&mut history, &parameters, Closing::ProcessExit)?;
    Ok(FourthRunPrefix {
        coordinates,
        history,
    })
}

/// H1g 的前缀（登记 5.3 H1g 行）：P(n1, C) → a 次 `mount_writable` 各自在取号写全部落盘之后、下一个写之前崩溃（在这一臂的副本上做）。
/// 交回前缀与每一次崩溃之后按装置解码核「每块盘世代号最新的系统配置槽的实例代号 = 这一次取的号」的结果（核不上 ⇒ 那一次不是崩在取号之后）。
pub(super) fn fourth_run_acquisition_crash_prefix(
    geometry: &Geometry,
    overwrites: u64,
    acquisition_crashes: u64,
) -> Result<(FourthRunPrefix, Vec<String>), String> {
    let parameters = parameters_for(geometry);
    let mut history = third_run_history_start(geometry, overwrites)?;
    third_run_close(&mut history, &parameters, Closing::ProcessExit)?;
    let mut acquisition_checks = Vec::new();
    for crash in 0..acquisition_crashes {
        let call = run_fourth_run_mount_writable(&history.pool, &parameters, &[], None, None);
        let acquired = call
            .call
            .outcome
            .as_ref()
            .map_err(|error| format!("第 {crash} 次取号崩溃那次挂载被拒: {error:?}"))?
            .output
            .instance
            .0;
        let acquisition_writes = ACQUISITION_WRITES_PER_DEVICE_LOCAL * history.pool.devices.len();
        let crashed =
            image_with_the_first_writes(&history.pool, &call.call.applied, acquisition_writes);
        let geometry_of_the_pool = independent_geometry(&crashed)?;
        let decoded = device_decode(&crashed, &geometry_of_the_pool);
        let newest_instances: Vec<Option<u32>> = [0u32, 1]
            .iter()
            .map(|device| {
                decoded
                    .system_configuration_slots
                    .iter()
                    .filter(|(slot_device, _, _, _)| slot_device == device)
                    .max_by_key(|(_, generation, _, _)| *generation)
                    .map(|(_, _, _, instance)| *instance)
            })
            .collect();
        let holds = newest_instances
            .iter()
            .all(|instance| *instance == Some(acquired));
        acquisition_checks.push(format!(
            "crash{crash}:acquired={acquired}:newest_slot_instances={newest_instances:?}:holds={holds}"
        ));
        history.pool = crashed;
    }
    Ok((
        FourthRunPrefix {
            coordinates: format!("n1={overwrites} a={acquisition_crashes}"),
            history,
        },
        acquisition_checks,
    ))
}

/// 抛弃步 A 的落点（第 3 次跑 `third_run_abandonment_faults` 的推广）：当前时间线上最新 `hidden_count` 条根的根槽，
/// 加第 `hidden_count` 新的那条根那次发布按 b 取的落点（k = 2：断链落在两条里较旧那一次发布上，登记 8.2）。
/// `hidden_count` = 1 时与第 3 次跑那一份逐项相同（单测 `fourth_run_abandonment_faults_equal_the_third_run_ones_for_one_hidden_root`）。
pub(super) fn fourth_run_abandonment_faults(
    history: &ThirdRunHistory,
    geometry: &PoolGeometry,
    form: UnreadableForm,
    replay_break: ReplayBreak,
    duration: FaultDuration,
    hidden_count: usize,
) -> Result<Option<Vec<ThirdRunFault>>, String> {
    if history.timeline.len() < hidden_count + 1 {
        return Ok(None);
    }
    let hidden_roots: Vec<TimelineRoot> = history.timeline[history.timeline.len() - hidden_count..]
        .iter()
        .rev()
        .copied()
        .collect();
    let mut faults = Vec::new();
    for hidden in &hidden_roots {
        let (root_slot_device, root_slot_offset) =
            root_slot_of(&history.pool, geometry, *hidden)
                .ok_or_else(|| format!("被藏的根 {hidden:?} 在根环里读不出"))?;
        faults.push(ThirdRunFault {
            device: root_slot_device,
            offset: root_slot_offset,
            length: u64::from(geometry.physical_block_size),
            form,
            duration,
            label: format!("root{}", key_text(Some(*hidden))),
        });
    }
    let broken = *hidden_roots.last().expect("hidden_count ≥ 1");
    let records = records_of_publish(&history.pool, geometry, broken);
    if records.is_empty() {
        return Ok(None);
    }
    match replay_break {
        ReplayBreak::EveryRecord => {
            for (counter, record_offset_on_the_device, _) in &records {
                for record_device in [DeviceIdentity(0), DeviceIdentity(1)] {
                    faults.push(ThirdRunFault {
                        device: record_device,
                        offset: *record_offset_on_the_device,
                        length: JOURNAL_RECORD_BYTES,
                        form,
                        duration,
                        label: format!("record(counter={counter},device={})", record_device.0),
                    });
                }
            }
        }
        ReplayBreak::FirstCopyOfTheFirstNamedUnit | ReplayBreak::FirstNamedUnit => {
            let Some(named) = records
                .iter()
                .find_map(|(_, _, record)| record.named.first())
            else {
                return Ok(None);
            };
            let copies = match replay_break {
                ReplayBreak::FirstNamedUnit => named.locations.len(),
                ReplayBreak::EveryRecord | ReplayBreak::FirstCopyOfTheFirstNamedUnit => 1,
            };
            for location in named.locations.iter().take(copies) {
                faults.push(ThirdRunFault {
                    device: location.device,
                    offset: location.slot.0 * SLOT_BYTES_LOCAL,
                    length: SLOT_BYTES_LOCAL,
                    form,
                    duration,
                    label: format!(
                        "unit(device={},slot={})",
                        location.device.0, location.slot.0
                    ),
                });
            }
        }
    }
    Ok(Some(faults))
}

// ---------------------------------------------------------------------------
// 12.5 一族的汇总（第 4 次跑）：第 3 次跑那一份的各栏，加 Q0 去向（每一臂每一格五类）、丢写格按去向分列、F15、F17 的点名。
// ---------------------------------------------------------------------------

#[derive(Default)]
struct FourthRunFamilySummary {
    counts: ThirdRunFamilySummary,
    destinations: BTreeMap<String, u64>,
    lost_write_cells_by_destination: BTreeMap<String, u64>,
    /// 失败条款 F15（7.1 新加第一行）：今天那一臂 b = records、W 的不断格上，写行那次发布没落在被藏那条根的槽上或去向不是「根槽被盖」。
    failure_clause_fifteen_findings: Vec<String>,
    failure_clause_fifteen_checks: u64,
    /// 失败条款 F6（第四节推的 ①、③）：今天那一臂被测挂载上装置算的 N-配置 为假的格。
    failure_clause_six_findings: Vec<String>,
    failure_clause_six_checks: u64,
}

impl FourthRunFamilySummary {
    pub(super) fn count(
        &mut self,
        loss: &CellLoss,
        rollback: bool,
        tested_ok: bool,
        destination: &HiddenRootDestination,
    ) {
        self.counts.count(loss, rollback, tested_ok);
        let destination_class = destination
            .name()
            .split(':')
            .next()
            .unwrap_or("")
            .to_string();
        *self
            .destinations
            .entry(destination_class.clone())
            .or_insert(0) += 1;
        if loss.lost_write_cell() || rollback {
            *self
                .lost_write_cells_by_destination
                .entry(destination_class)
                .or_insert(0) += 1;
        }
    }

    pub(super) fn emit(&self, family: &str, arm: &str, geometry: &Geometry) {
        let counts = &self.counts;
        emit_result(&format!(
            "name=r4_family_summary family={family} arm={arm} geometry={} cells={} void_cells_v2={} fault_not_reached_cells={} q1_overwrite_cells={} q1_in_ring_overwrite_cells={} q1_out_of_ring_overwrite_cells={} q1_landing_overwrite_cells={} q2_rollback_cells={} tested_ok_cells={} q3_read_back={:?} q3_last_confirmed_cells={} q3_lost_cells={} q3c_tail={:?} q3c_lost_cells={} lost_write_cells_q1_q2_q3={} q0_destinations={:?} lost_write_cells_by_destination={:?} tested_classes={:?} remount_classes={:?} s4_mismatches={} f4_failures={} f6_checked={} f6_findings={} f13_checked={} f13_failures={} f15_checked={} f15_findings={} formula_mismatches={}",
            geometry.label,
            counts.cells,
            counts.void_cells,
            counts.fault_not_reached_cells,
            counts.overwrite_cells,
            counts.in_ring_overwrite_cells,
            counts.out_of_ring_overwrite_cells,
            counts.landing_overwrite_cells,
            counts.rollback_cells,
            counts.tested_ok_cells,
            counts.read_back,
            counts.read_back_last_confirmed_cells,
            counts.read_back_lost_cells,
            counts.tail,
            counts.tail_lost_cells,
            counts.lost_write_cells,
            self.destinations,
            self.lost_write_cells_by_destination,
            counts.tested_classes,
            counts.remount_classes,
            counts.stop_condition_four_mismatches.len(),
            counts.failure_clause_four_findings.len(),
            self.failure_clause_six_checks,
            self.failure_clause_six_findings.len(),
            counts.failure_clause_thirteen_checks,
            counts.failure_clause_thirteen_findings.len(),
            self.failure_clause_fifteen_checks,
            self.failure_clause_fifteen_findings.len(),
            counts.formula_mismatches.len()
        ));
        for (kind, list) in [
            ("s4", &counts.stop_condition_four_mismatches),
            ("f4", &counts.failure_clause_four_findings),
            ("f6", &self.failure_clause_six_findings),
            ("f13", &counts.failure_clause_thirteen_findings),
            ("f15", &self.failure_clause_fifteen_findings),
            ("formula", &counts.formula_mismatches),
        ] {
            for detail in list {
                emit_result(&format!(
                    "name=r4_family_finding family={family} arm={arm} geometry={} kind={kind} detail={detail:?}",
                    geometry.label
                ));
            }
        }
    }
}

/// 一族怎么跑：族名（进结果行）、几何、A 藏几条根、一个前缀之后怎么接（断点全枚举 / 只不断 / m 尾巴）。
#[derive(Clone, Copy, Debug)]
pub(super) enum FourthRunContinuation {
    /// H1d 的全部断点（不断、crash、write_fails、barrier_fails）。
    EveryCut,
    /// 只跑不断那一格（8.2 的敏感性点「H1d 的 `none`」）。
    UncutOnly,
    /// H1e 的 m 尾巴（A 不断 → m 次改内容 → 关闭 C）。
    Tail,
}

pub(super) struct FourthRunFamily<'plan> {
    pub(super) name: &'plan str,
    pub(super) geometry: &'plan Geometry,
    pub(super) hidden_count: usize,
    pub(super) continuation: FourthRunContinuation,
    pub(super) tail_overwrites: &'plan [u64],
    pub(super) durations: FourthRunDurationPlan,
    /// 今天那一臂顺带核 F6（第四节推的 ①）与 F15（7.1 新加第一行）：只在登记点名的 G0 上的 H1d、H1e 开。
    pub(super) checks_the_clauses_on_the_hidden_root: ClauseChecksOnTheHiddenRoot,
}

/// A 取哪几种时长（登记 5.3：H1d 与接 H1d 断点的族按这一臂的 H1d 时长，接 H1e 尾巴的按这一臂的 H1e 时长；H1g 只 W）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FourthRunDurationPlan {
    OfTheArm,
    WholeCallOnly,
}

/// 这一族顺带核不核 F6、F15（登记第十节：F6 管 H1d、H1e；F15 管 H1d 的 `none` 格与 H1e 里 b = records、W 的格）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ClauseChecksOnTheHiddenRoot {
    Checked,
    NotRegisteredForThisFamily,
}

/// 被测挂载之后、断点那一刻（或 m 尾巴之后）的镜像 `post` → 撤故障 → 再可写挂载（这一臂的代码，不注入）→ 历史末尾算三样。
struct FourthRunCellContext<'cell> {
    arm: FourthRunArm,
    family: &'cell FourthRunFamily<'cell>,
    history: &'cell ThirdRunHistory,
    protected: &'cell ThirdRunConfirmation,
    base: &'cell MemoryPool,
    parameters: &'cell MakeFilesystemParameters,
    first_new_instance: u32,
}

#[allow(
    clippy::too_many_arguments,
    reason = "一格的坐标、被测挂载那一行与它的判定原样打进结果行，拆成结构体只多一层搬运"
)]
fn evaluate_fourth_run_cell(
    context: &FourthRunCellContext<'_>,
    coordinates: &str,
    post: &MemoryPool,
    tested_line: &str,
    rollback: bool,
    tested_ok: bool,
    do_the_tail: bool,
    destination: &HiddenRootDestination,
    summary: &mut FourthRunFamilySummary,
) {
    let remount = run_third_run_mount_writable(post, context.parameters, &[], None, None);
    let remount_class = third_run_outcome_class(&remount.outcome, remount.write_calls);
    let loss = evaluate_cell_loss(
        context.history,
        context.protected,
        context.base,
        &remount.pool_after,
        do_the_tail.then_some(context.first_new_instance),
    );
    summary.count(&loss, rollback, tested_ok, destination);
    *summary
        .counts
        .remount_classes
        .entry(remount_class.to_string())
        .or_insert(0) += 1;
    emit_result(&format!(
        "name=r4_cell family={} arm={} geometry={} {coordinates} {tested_line} q0_destination={} q2_rollback={rollback} remount_class={remount_class} remount_error={} remount_chosen={} {}",
        context.family.name,
        context.arm.name,
        context.family.geometry.label,
        destination.name(),
        mount_error_member_or_none(&remount.outcome),
        key_text(
            remount
                .outcome
                .as_ref()
                .ok()
                .map(|mounted| root_pair(&mounted.output.chosen_root))
        ),
        loss.render()
    ));
}

/// 一族在一条臂上跑完：每个前缀 × 造法 × b × 时长 → A →（断点 / m 尾巴）→ 撤故障 → 再挂载 → 三样。
pub(super) fn run_fourth_run_family(
    family: &FourthRunFamily<'_>,
    arm: FourthRunArm,
    prefixes: Vec<Result<FourthRunPrefix, String>>,
) {
    let parameters = parameters_for(family.geometry);
    let mut summary = FourthRunFamilySummary::default();
    let durations: Vec<(FaultDuration, bool)> = match (family.durations, family.continuation) {
        (FourthRunDurationPlan::WholeCallOnly, _) => vec![(FaultDuration::WholeCall, true)],
        (FourthRunDurationPlan::OfTheArm, FourthRunContinuation::EveryCut) => {
            fourth_run_held_once_durations(arm)
        }
        (FourthRunDurationPlan::OfTheArm, FourthRunContinuation::UncutOnly) => {
            fourth_run_held_once_durations(arm)
                .into_iter()
                .map(|(duration, _)| (duration, true))
                .collect()
        }
        (FourthRunDurationPlan::OfTheArm, FourthRunContinuation::Tail) => {
            fourth_run_tail_durations(arm)
                .into_iter()
                .map(|duration| (duration, true))
                .collect()
        }
    };
    let mut cells_by_duration: BTreeMap<String, u64> = BTreeMap::new();
    for prefix in prefixes {
        let prefix = match prefix {
            Ok(prefix) => prefix,
            Err(error) => {
                emit_result(&format!(
                    "name=r4_prefix family={} arm={} geometry={} verdict=stop reason={error:?}",
                    family.name, arm.name, family.geometry.label
                ));
                continue;
            }
        };
        let history = &prefix.history;
        let base = history.pool.clone();
        let Ok(pool_geometry) = independent_geometry(&base) else {
            emit_result(&format!(
                "name=r4_prefix family={} arm={} geometry={} {} verdict=stop reason=\"几何解不出\"",
                family.name, arm.name, family.geometry.label, prefix.coordinates
            ));
            continue;
        };
        let root_slots = root_slot_device_offsets(&pool_geometry);
        let protected = history
            .confirmations
            .last()
            .cloned()
            .expect("前缀至少确认过首个文件");
        let last_confirmed = protected.content_number;
        let hidden = history.tip();
        let hidden_position = hidden_root_position(&base, hidden);
        let base_fingerprint = image_fingerprint(&base);
        // 「不绕环」（第 3 次跑登记 5.3 G0 那一句）：最大计数器 < 环槽数。
        let highest_counter = device_decode(&base, &pool_geometry)
            .records
            .keys()
            .map(|(_, counter)| *counter)
            .max()
            .unwrap_or(0);
        let ring_not_wrapped =
            highest_counter < pool_geometry.journal_ring_bytes / JOURNAL_RECORD_BYTES;
        emit_result(&format!(
            "name=r4_prefix family={} arm={} geometry={} {} tip={} hidden_slot={} last_confirmed={last_confirmed} base_fingerprint={base_fingerprint:016x} highest_counter={highest_counter} ring_not_wrapped={ring_not_wrapped} tail_checks={} tail_check_failures={}",
            family.name,
            arm.name,
            family.geometry.label,
            prefix.coordinates,
            key_text(Some(hidden)),
            hidden_position.map_or("none".to_string(), |position| format!("{}:{}", position.region, position.slot)),
            history.tail_checks,
            history.tail_check_failures.len()
        ));
        for failure in &history.tail_check_failures {
            emit_result(&format!(
                "name=r4_f3_tail_check_failure family={} arm={} {} detail={failure:?}",
                family.name, arm.name, prefix.coordinates
            ));
        }
        let context = FourthRunCellContext {
            arm,
            family,
            history,
            protected: &protected,
            base: &base,
            parameters: &parameters,
            first_new_instance: hidden.0 + 1,
        };
        for form in [UnreadableForm::ReadFails, UnreadableForm::ReadsZeros] {
            for replay_break in [
                ReplayBreak::EveryRecord,
                ReplayBreak::FirstCopyOfTheFirstNamedUnit,
            ] {
                for (duration, only_the_uncut_cell) in &durations {
                    let head = format!(
                        "{} b={} form={} duration={}",
                        prefix.coordinates,
                        replay_break.name(),
                        form.name(),
                        duration.name()
                    );
                    let faults = match fourth_run_abandonment_faults(
                        history,
                        &pool_geometry,
                        form,
                        replay_break,
                        *duration,
                        family.hidden_count,
                    ) {
                        Ok(Some(faults)) => faults,
                        Ok(None) => {
                            emit_result(&format!(
                                "name=r4_group family={} arm={} geometry={} {head} verdict=not_applicable",
                                family.name, arm.name, family.geometry.label
                            ));
                            continue;
                        }
                        Err(error) => {
                            emit_result(&format!(
                                "name=r4_group family={} arm={} geometry={} {head} verdict=stop reason={error:?}",
                                family.name, arm.name, family.geometry.label
                            ));
                            continue;
                        }
                    };
                    let Ok(tested) = run_fourth_run_tested_mount(
                        arm,
                        &base,
                        &parameters,
                        &faults,
                        &head,
                        &mut summary.counts,
                    ) else {
                        emit_result(&format!(
                            "name=r4_group family={} arm={} geometry={} {head} verdict=stop reason=\"几何解不出\"",
                            family.name, arm.name, family.geometry.label
                        ));
                        continue;
                    };
                    let tested_ok = tested.call.call.outcome.is_ok();
                    let mounted = tested.call.call.outcome.as_ref().ok();
                    let rollback = tested_ok
                        && tested
                            .summary
                            .effective
                            .and_then(|effective| history.content_number_of(effective))
                            .is_some_and(|number| number < last_confirmed);
                    let content_numbers = content_numbers_with_the_publishes_of(history, mounted);
                    let destination_after_the_tested_mount = hidden_position.map_or_else(
                        || HiddenRootDestination::Other("hidden_root_not_in_base".to_string()),
                        |position| {
                            hidden_root_destination(
                                &position,
                                &content_numbers,
                                last_confirmed,
                                &tested.call.call.pool_after,
                                tested_ok.then_some(tested.summary.effective).flatten(),
                            )
                        },
                    );
                    if family.checks_the_clauses_on_the_hidden_root
                        == ClauseChecksOnTheHiddenRoot::Checked
                    {
                        fourth_run_family_cross_checks(
                            arm,
                            &head,
                            replay_break,
                            *duration,
                            history,
                            &tested,
                            hidden_position,
                            family.geometry,
                            &destination_after_the_tested_mount,
                            &mut summary,
                        );
                    }
                    let not_reached = !tested.unintercepted.is_empty();
                    if not_reached {
                        if arm.is_today() {
                            summary.counts.void_cells += 1;
                        } else {
                            summary.counts.fault_not_reached_cells += 1;
                        }
                    }
                    *summary
                        .counts
                        .tested_classes
                        .entry(format!(
                            "{}:{}",
                            tested.summary.class, tested.summary.member
                        ))
                        .or_insert(0) += 1;
                    let tested_line = format!(
                        "{} hidden={} base_fingerprint={base_fingerprint:016x} faults={} unintercepted={}",
                        render_fourth_run_tested_mount(&tested),
                        key_text(Some(hidden)),
                        faults.len(),
                        tested.unintercepted.join(",")
                    );
                    if arm.is_today() && not_reached {
                        emit_result(&format!(
                            "name=r4_cell family={} arm={} geometry={} {head} cut=none point=none verdict=void_v2 {tested_line}",
                            family.name, arm.name, family.geometry.label
                        ));
                        continue;
                    }
                    *cells_by_duration.entry(duration.name()).or_insert(0) += 1;
                    match family.continuation {
                        FourthRunContinuation::Tail => run_fourth_run_tail_cells(
                            &context,
                            &head,
                            &tested,
                            &tested_line,
                            rollback,
                            &destination_after_the_tested_mount,
                            &mut summary,
                        ),
                        FourthRunContinuation::EveryCut | FourthRunContinuation::UncutOnly => {
                            run_fourth_run_cut_cells(
                                &context,
                                &head,
                                &faults,
                                &tested,
                                &tested_line,
                                rollback,
                                *only_the_uncut_cell,
                                &root_slots,
                                &pool_geometry,
                                hidden_position,
                                &destination_after_the_tested_mount,
                                &mut summary,
                            );
                        }
                    }
                }
            }
        }
    }
    for (duration, groups) in &cells_by_duration {
        emit_result(&format!(
            "name=r4_family_groups family={} arm={} geometry={} duration={duration} groups={groups}",
            family.name, arm.name, family.geometry.label
        ));
        // 7.2：H1e 每种时长 4 × 2 × 2 × 4 格；H1g 每臂 n1 × a × m × b × 造法 = 32 格（只 W）。
        let registered = match (family.continuation, family.name) {
            (FourthRunContinuation::Tail, "h1e") if family.hidden_count == 1 => Some(4 * 2 * 2 * 4),
            (FourthRunContinuation::Tail, "h1g") => Some(2 * 2 * 2 * 2 * 2),
            (
                FourthRunContinuation::Tail
                | FourthRunContinuation::EveryCut
                | FourthRunContinuation::UncutOnly,
                _,
            ) => None,
        };
        if let Some(registered) = registered {
            let cells = groups * family.tail_overwrites.len() as u64;
            emit_result(&format!(
                "name=r4_section_seven_two_anchor arm={} geometry={} item=\"{} 每种时长的格数\" duration={duration} computed={cells} registered={registered} verdict={}",
                arm.name,
                family.geometry.label,
                family.name,
                if cells == registered { "pass" } else { "fail_v4" }
            ));
        }
    }
    summary.emit(family.name, arm.name, family.geometry);
}

/// 一组（一个前缀 × 造法 × b × 时长）在 H1d 那样的断点上全枚举：不断、crash j（前 j 个写落盘）、write_fails j、barrier_fails j。
#[allow(
    clippy::too_many_arguments,
    reason = "一组的上下文、被测挂载与它的判定原样传给每一格，拆成结构体只多一层搬运"
)]
fn run_fourth_run_cut_cells(
    context: &FourthRunCellContext<'_>,
    head: &str,
    faults: &[ThirdRunFault],
    tested: &FourthRunTestedMount,
    tested_line: &str,
    rollback: bool,
    only_the_uncut_cell: bool,
    root_slots: &[(DeviceIdentity, u64)],
    pool_geometry: &PoolGeometry,
    hidden_position: Option<HiddenRootPosition>,
    destination_after_the_tested_mount: &HiddenRootDestination,
    summary: &mut FourthRunFamilySummary,
) {
    let arm = context.arm;
    let history = context.history;
    let last_confirmed = context.protected.content_number;
    let tested_ok = tested.call.call.outcome.is_ok();
    let first_root_write = index_of_the_first_root_slot_write(
        &tested.call.call.applied,
        root_slots,
        u64::from(pool_geometry.physical_block_size),
    );
    let barriers = first_root_write.map_or(0, |index| {
        barriers_before_write(&tested.call.call.applied, index + 1)
    });
    let cut_cells_expected = if only_the_uncut_cell {
        1
    } else {
        first_root_write.map_or(1, |index| 1 + 2 * (index as u64 + 1) + barriers)
    };
    let destination_of =
        |pool: &MemoryPool, ok_effective: Option<TimelineRoot>, mounted: Option<&Mounted>| {
            hidden_position.map_or_else(
                || HiddenRootDestination::Other("hidden_root_not_in_base".to_string()),
                |position| {
                    hidden_root_destination(
                        &position,
                        &content_numbers_with_the_publishes_of(history, mounted),
                        last_confirmed,
                        pool,
                        ok_effective,
                    )
                },
            )
        };
    let mut enumerated = 0u64;
    evaluate_fourth_run_cell(
        context,
        &format!("{head} cut=none point=none"),
        &tested.call.call.pool_after,
        &format!(
            "{tested_line} first_root_write={} barriers_before_the_row_root={barriers}",
            first_root_write.map_or("none".to_string(), |index| index.to_string())
        ),
        rollback,
        tested_ok,
        true,
        destination_after_the_tested_mount,
        summary,
    );
    enumerated += 1;
    if !only_the_uncut_cell {
        if let Some(first_root_write) = first_root_write {
            let tested_mounted = tested.call.call.outcome.as_ref().ok();
            for write_count in 0..=first_root_write {
                let post = image_with_the_first_writes(
                    context.base,
                    &tested.call.call.applied,
                    write_count,
                );
                let destination = destination_of(&post, None, tested_mounted);
                evaluate_fourth_run_cell(
                    context,
                    &format!("{head} cut=crash point={write_count}"),
                    &post,
                    "a_outcome=crashed",
                    false,
                    false,
                    false,
                    &destination,
                    summary,
                );
                enumerated += 1;
            }
            for (cut, failing_write, failing_barrier) in (0..=first_root_write)
                .map(|index| ("write_fails", Some(index as u64), None))
                .chain((0..barriers).map(|index| ("barrier_fails", None, Some(index))))
            {
                let failed = run_fourth_run_mount_writable(
                    context.base,
                    context.parameters,
                    faults,
                    failing_write,
                    failing_barrier,
                );
                let failed_summary = TestedMountSummary::of(&failed.call);
                let failed_ok = failed.call.outcome.is_ok();
                let failed_rollback = failed_ok
                    && failed_summary
                        .effective
                        .and_then(|effective| history.content_number_of(effective))
                        .is_some_and(|number| number < last_confirmed);
                let failed_mounted = failed.call.outcome.as_ref().ok();
                let destination = destination_of(
                    &failed.call.pool_after,
                    failed_ok.then_some(failed_summary.effective).flatten(),
                    failed_mounted,
                );
                evaluate_fourth_run_cell(
                    context,
                    &format!(
                        "{head} cut={cut} point={}",
                        failing_write.or(failing_barrier).unwrap_or(0)
                    ),
                    &failed.call.pool_after,
                    &format!(
                        "{} {} fired={}",
                        failed_summary.render("a"),
                        failed.render_marker("a"),
                        failed.call.injected_failure_fired
                    ),
                    failed_rollback,
                    failed_ok,
                    false,
                    &destination,
                    summary,
                );
                enumerated += 1;
            }
        }
    }
    if enumerated != cut_cells_expected {
        summary.counts.formula_mismatches.push(format!(
            "{head}: 枚举 {enumerated} 格，公式 1 + 2(J + 1) + 屏障数 = {cut_cells_expected}"
        ));
    }
    emit_result(&format!(
        "name=r4_group family={} arm={} geometry={} {head} cells={enumerated} formula_cells={cut_cells_expected} verdict={}",
        context.family.name,
        arm.name,
        context.family.geometry.label,
        if enumerated == cut_cells_expected { "pass" } else { "fail_v4" }
    ));
}

/// 一组接 H1e 的 m 尾巴：A 交回的会话里再做 m 次改内容 → 关闭 C → 撤故障 → 再挂载 → 三样（A 被拒的臂记「因拒没做」）。
#[allow(
    clippy::too_many_arguments,
    reason = "一组的上下文、被测挂载与它的判定原样传给每一格，拆成结构体只多一层搬运"
)]
fn run_fourth_run_tail_cells(
    context: &FourthRunCellContext<'_>,
    head: &str,
    tested: &FourthRunTestedMount,
    tested_line: &str,
    rollback: bool,
    destination_after_the_tested_mount: &HiddenRootDestination,
    summary: &mut FourthRunFamilySummary,
) {
    let tested_ok = tested.call.call.outcome.is_ok();
    for tail_overwrites in context.family.tail_overwrites {
        let mut history = context.history.clone();
        history.pool = tested.call.call.pool_after.clone();
        let tail_writes = match &tested.call.call.outcome {
            Ok(mounted) => {
                let effective_content = history
                    .content_number_of(root_pair(&mounted.output.effective_root))
                    .unwrap_or(0);
                push_the_publishes_of_a_mount(&mut history, mounted, effective_content);
                history.session = Some(Session {
                    allocator: mounted.allocator.clone(),
                    current: mounted.current.clone(),
                    instance: mounted.output.instance,
                });
                let mut done = 0u64;
                let mut failure = None;
                for _ in 0..*tail_overwrites {
                    match third_run_change_content(&mut history, context.parameters) {
                        Ok(()) => done += 1,
                        Err(error) => {
                            failure = Some(error);
                            break;
                        }
                    }
                }
                match failure {
                    None => format!("done:{done}"),
                    Some(error) => format!("failed_after:{done}:{}", error_member_of_debug(&error)),
                }
            }
            Err(_) => "refused_not_done".to_string(),
        };
        let _ = third_run_close(&mut history, context.parameters, Closing::ProcessExit);
        let tail_context = FourthRunCellContext {
            arm: context.arm,
            family: context.family,
            history: &history,
            protected: context.protected,
            base: context.base,
            parameters: context.parameters,
            first_new_instance: context.first_new_instance,
        };
        evaluate_fourth_run_cell(
            &tail_context,
            &format!("{head} m={tail_overwrites}"),
            &history.pool.clone(),
            &format!(
                "{tested_line} tail_writes={tail_writes} last_confirmed_after_the_tail={}",
                history.last_confirmed_content_number()
            ),
            rollback,
            tested_ok,
            true,
            destination_after_the_tested_mount,
            summary,
        );
    }
}

/// 一组的被测挂载上顺带核的条款（今天那一臂）：F4（7.1）、F6（第四节推的 ①）、F15（7.1 新加第一行）；Q0「被抛弃」那一栏。
#[allow(
    clippy::too_many_arguments,
    reason = "一组的坐标、被测挂载与装置的判定原样传进来，拆成结构体只多一层搬运"
)]
fn fourth_run_family_cross_checks(
    arm: FourthRunArm,
    head: &str,
    replay_break: ReplayBreak,
    duration: FaultDuration,
    history: &ThirdRunHistory,
    tested: &FourthRunTestedMount,
    hidden_position: Option<HiddenRootPosition>,
    geometry: &Geometry,
    destination: &HiddenRootDestination,
    summary: &mut FourthRunFamilySummary,
) {
    if !arm.is_today() {
        return;
    }
    let tested_ok = tested.call.call.outcome.is_ok();
    if *destination == HiddenRootDestination::InRingAndAbandoned {
        summary.counts.abandoned_hidden_root_cells += 1;
    }
    let before_hidden = history.timeline.iter().rev().nth(1).copied();
    if tested_ok
        && replay_break == ReplayBreak::FirstCopyOfTheFirstNamedUnit
        && duration == FaultDuration::WholeCall
        && tested.summary.effective != before_hidden
    {
        summary.counts.failure_clause_four_findings.push(format!(
            "{head}: E {}，被藏那条根前一条 {}",
            key_text(tested.summary.effective),
            key_text(before_hidden)
        ));
    }
    summary.failure_clause_six_checks += 1;
    if !tested.judgement.configuration_newer {
        summary
            .failure_clause_six_findings
            .push(format!("{head}: 装置在今天那一臂上算的 N-配置 为假"));
    }
    if let (true, ReplayBreak::EveryRecord, FaultDuration::WholeCall, Ok(mounted), Some(position)) = (
        tested_ok,
        replay_break,
        duration,
        &tested.call.call.outcome,
        hidden_position,
    ) {
        summary.failure_clause_fifteen_checks += 1;
        let row_txg = mounted.output.row_publish.root().checkpoint_txg.0;
        let predicted = predicted_root_ring_slot_of_a_publish(row_txg, geometry.slots_per_region);
        let holds = row_txg == position.key.1
            && predicted == (position.region, position.slot)
            && *destination == HiddenRootDestination::RootSlotOverwritten;
        if !holds {
            summary.failure_clause_fifteen_findings.push(format!(
                "{head}: 写行那次发布 txg {row_txg}（落在 {predicted:?}），被藏那条根 txg {}、s_h {:?}，去向 {}",
                position.key.1,
                (position.region, position.slot),
                destination.name()
            ));
        }
    }
}

#[cfg(test)]
mod fourth_run_tests;
