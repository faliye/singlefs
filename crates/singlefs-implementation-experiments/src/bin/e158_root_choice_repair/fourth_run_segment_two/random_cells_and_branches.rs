//! 随机历史的每一格（尾部类、丢失判定、被藏的根落到哪里）、与 harness 的对拍，以及主支、故障支、崩溃支怎么跑。

use super::*;

/// 尾巴那一截读到的东西。
pub(super) struct TailReading {
    pub(super) read_back: RandomReadBack,
    pub(super) tail_class: String,
    pub(super) end: MemoryPool,
}

/// ③c（登记 5.3 H-随 与 r3 登记第 373–391 行 ③c 那一行）：历史末尾镜像上实例代号 ≥ 第一次崩溃恢复抛弃根那一步被藏那条根的实例代号 + 1
/// 的全部自证根的根槽读坏（W、`read_fails`），冷启动读回：读回它落到的那条根自己那一版 / 别的 / 读失败（只有读失败按丢写记）。
pub(super) fn third_tail_class(
    end: &MemoryPool,
    ledger: &RandomLedger,
    first_hidden_instance: Option<u32>,
) -> String {
    let Some(hidden_instance) = first_hidden_instance else {
        return "na".to_string();
    };
    let Ok(geometry) = independent_geometry(end) else {
        return "failed:geometry".to_string();
    };
    let slots = checker_image::root_slot_positions(&geometry);
    let faults: Vec<ThirdRunFault> = checker_image::valid_roots(end, &geometry)
        .into_iter()
        .filter(|(_, _, view)| view.instance > hidden_instance)
        .filter_map(|(region, slot, view)| {
            slots
                .iter()
                .find(|(candidate_region, candidate_slot, _, _)| {
                    *candidate_region == region && *candidate_slot == slot
                })
                .map(|(_, _, device, offset)| ThirdRunFault {
                    device: DeviceIdentity(*device),
                    offset: *offset,
                    length: u64::from(geometry.physical_block_size),
                    form: UnreadableForm::ReadFails,
                    duration: FaultDuration::WholeCall,
                    label: format!(
                        "tail_root{}",
                        key_text(Some((view.instance, view.checkpoint_txg)))
                    ),
                })
        })
        .collect();
    let tail = run_third_run_recover(end, &faults);
    match &tail.outcome.outcome {
        RecoveryOutcome::Failed { failure, .. } => {
            format!("failed:{}", error_member_of_debug(&format!("{failure:?}")))
        }
        RecoveryOutcome::NoFile { .. } | RecoveryOutcome::FileRead { .. } => {
            let landing = tail
                .outcome
                .effective_root
                .map(|(instance, txg)| (instance.0, txg.0))
                .or_else(|| chosen_root_of(&tail.outcome.outcome));
            let expected = landing.and_then(|root| ledger.content_number_of(root));
            let file: Option<Vec<u8>> = match &tail.outcome.outcome {
                RecoveryOutcome::FileRead { content, .. } => Some(content.clone()),
                RecoveryOutcome::NoFile { .. } | RecoveryOutcome::Failed { .. } => None,
            };
            let own = expected.is_some_and(|number| {
                ledger
                    .contents
                    .get(usize::try_from(number).expect("内容号"))
                    == Some(&file)
            });
            if own {
                format!("own_version:{}", key_text(landing))
            } else {
                format!("other:{}:expected={expected:?}", key_text(landing))
            }
        }
    }
}

fn tail_class_is_lost(tail_class: &str) -> bool {
    tail_class.starts_with("failed")
}

/// 这份镜像上 `target` 那条根的根记录（fsid 取这份镜像自己的系统配置：harness 的池用 E142 那一个 fsid，不是本装置的）。
fn root_record_on_the_image(pool: &MemoryPool, target: TimelineRoot) -> Option<RootRecord> {
    let geometry = independent_geometry(pool).ok()?;
    let slot_bytes = usize::try_from(geometry.physical_block_size).ok()?;
    for (_region, _slot, device_number, offset) in checker_image::root_slot_positions(&geometry) {
        let Some(bytes) = pool.read(device_number, offset, slot_bytes) else {
            continue;
        };
        if let Some(record) = RootRecord::parse_slot(&bytes, &geometry.filesystem_identifier) {
            if (record.instance.0, record.checkpoint_txg.0) == target {
                return Some(record);
            }
        }
    }
    None
}

// ------------------------------------------------------------------------
// 13.5 一格（登记第六节 Q0–Q3 与丢写格；5.3「V_LC、Q1、Q2 按每一次崩溃恢复抛弃根那一步各算一份」）：
//      主路径与注入点分支上每一次崩溃恢复抛弃根那一步、注入点那一步（不是崩溃恢复那一步时另立一格）、每一个崩溃状态各是一格。
//      V_LC = 那一步开始之前最后确认那一版（字面读法；回退读法另一栏）。Q1 = V_LC 引用的单元在这个分支末尾被改的份数，按那条根在末尾
//      还在 / 不在根环里分两栏，另报第三栏（尾巴读回落到的根）与时点栏（第一次被改在哪一步、那一刻之后已有几笔确认、那一刻根在不在环里）。
//      Q2 = 那一格的被测挂载交回 `Ok` 而 E 的内容号更旧。Q3 = 这一格之后的每一次读回（分支末尾那一次、注入点分支的重开、崩溃状态的两次）。
// ------------------------------------------------------------------------

pub(super) struct RandomCell {
    pub(super) name: String,
    pub(super) kind: &'static str,
    pub(super) order: usize,
    pub(super) protected: Option<RandomConfirmation>,
    pub(super) protected_rollback_aware: u64,
    pub(super) mount: Option<usize>,
    pub(super) read_backs: Vec<(String, RandomReadBack)>,
    pub(super) tail_class: String,
    pub(super) destination: Option<String>,
}

/// 一格三样的结果。
pub(super) struct RandomCellLoss {
    pub(super) in_ring_overwritten: u64,
    pub(super) out_of_ring_overwritten: u64,
    pub(super) landing_overwritten: String,
    pub(super) first_change: Option<FirstChange>,
    pub(super) older_version_returned: bool,
    pub(super) older_version_returned_rollback_aware: bool,
    pub(super) read_back_lost: bool,
    pub(super) read_back_lost_rollback_aware: bool,
    pub(super) read_back_last_confirmed: bool,
    pub(super) broken_root_slots_read_back_lost: bool,
}

impl RandomCellLoss {
    pub(super) fn overwrite_cell(&self) -> bool {
        self.in_ring_overwritten + self.out_of_ring_overwritten > 0
    }

    pub(super) fn lost_write_cell(&self) -> bool {
        self.overwrite_cell()
            || self.older_version_returned
            || self.read_back_lost
            || self.broken_root_slots_read_back_lost
    }

    pub(super) fn lost_write_cell_rollback_aware(&self) -> bool {
        self.overwrite_cell()
            || self.older_version_returned_rollback_aware
            || self.read_back_lost_rollback_aware
            || self.broken_root_slots_read_back_lost
    }
}

fn evaluate_a_random_cell(
    cell: &RandomCell,
    mounts: &[RandomMount],
    first_changes: &BTreeMap<u64, FirstChange>,
    base: &MemoryPool,
    end: &MemoryPool,
    final_landing: Option<TimelineRoot>,
) -> RandomCellLoss {
    let end_roots = independent_geometry(end)
        .map(|geometry| readable_roots_independent(end, &geometry))
        .unwrap_or_default();
    let overwritten_in = |version: &UnitBytesOfAVersion| -> u64 {
        version
            .iter()
            .filter(|((device, slot, span), bytes)| {
                let length = usize::try_from(span * SLOT_BYTES_LOCAL).expect("单元长");
                end.read(*device, slot * SLOT_BYTES_LOCAL, length)
                    .unwrap_or_default()
                    != **bytes
            })
            .count() as u64
    };
    let (in_ring_overwritten, out_of_ring_overwritten) = match &cell.protected {
        Some(protected) => {
            let overwritten = overwritten_in(&protected.unit_bytes);
            if end_roots.contains(&protected.root) {
                (overwritten, 0)
            } else {
                (0, overwritten)
            }
        }
        None => (0, 0),
    };
    let landing_overwritten = match final_landing {
        Some(landing) => match root_record_on_the_image(base, landing) {
            Some(record) => match unit_bytes_referenced_by(base, &record) {
                Ok(version) => overwritten_in(&version).to_string(),
                Err(error) => format!("unreadable_in_base:{}", error.replace(' ', "_")),
            },
            None => "written_after_the_cell".to_string(),
        },
        None => "none".to_string(),
    };
    let mount = cell.mount.and_then(|index| mounts.get(index));
    RandomCellLoss {
        in_ring_overwritten,
        out_of_ring_overwritten,
        landing_overwritten,
        first_change: cell
            .protected
            .as_ref()
            .and_then(|protected| first_changes.get(&protected.content_number).copied()),
        older_version_returned: mount.is_some_and(|record| record.older_version_returned),
        older_version_returned_rollback_aware: mount
            .is_some_and(|record| record.older_version_returned_rollback_aware),
        read_back_lost: cell
            .read_backs
            .iter()
            .any(|(_, read_back)| read_back.literal_lost()),
        read_back_lost_rollback_aware: cell
            .read_backs
            .iter()
            .any(|(_, read_back)| read_back.rollback_aware_lost()),
        read_back_last_confirmed: cell
            .read_backs
            .iter()
            .all(|(_, read_back)| read_back.literal.starts_with("last_confirmed")),
        broken_root_slots_read_back_lost: tail_class_is_lost(&cell.tail_class),
    }
}

/// Q0 的去向（登记「读法写死」表那一行，判的次序见执行员修订第 8 条）：崩溃恢复抛弃根那一步交回（或崩在中途）之后的镜像上判。
fn destination_of_the_hidden_root(
    event: &CrashRecoveryEvent,
    mounts: &[RandomMount],
    ledger: &RandomLedger,
) -> String {
    let Some(position) = hidden_root_position(&event.base, event.hidden_root) else {
        return "other:hidden_root_not_in_the_base".to_string();
    };
    let Some(hidden_content_number) = event.hidden_content_number else {
        return "other:hidden_root_not_in_the_ledger".to_string();
    };
    let tested_ok_effective = mounts
        .get(event.mount_index)
        .filter(|record| class_is_a_successful_mount(record.summary.class))
        .and_then(|record| record.summary.effective);
    hidden_root_destination(
        &position,
        &ledger.content_number_by_root,
        hidden_content_number,
        &event.after,
        tested_ok_effective,
    )
    .name()
}

pub(super) fn render_first_change(first_change: Option<FirstChange>) -> String {
    first_change.map_or("none".to_string(), |change| {
        format!(
            "order{}:later_confirmations{}:in_ring{}",
            change.order, change.later_confirmations, change.root_in_ring_then
        )
    })
}

// ------------------------------------------------------------------------
// 13.6 今天那一臂上与 harness 自己的执行器比（S9 末句的收严、S7、S5a / S5b）：harness 执行器（不跑 checker，只在模型、执行器判定、
//      panic 上停）每个观察到的位置的镜像指纹与整池调用数，与装置执行器同一位置的逐项比；harness 判红的那一步（跑 checker 那一档）。
// ------------------------------------------------------------------------

/// harness 执行器在今天那一份副本上跑同一段历史：每个观察到的位置（步）→（镜像指纹，那一步做完时整池的调用数）。
pub(super) fn harness_trace(history: &GeneratedHistory) -> Vec<(usize, u64, FaultCallCounts)> {
    let plan = SharedFaultPlan::unarmed(HistoryDeviceWidth::FourGibibytes.fixed_geometry());
    let mut positions = Vec::new();
    let _run = execute_history_with_faults(
        history,
        harness_execution(PerStepChecker::Skipped),
        &SharedStream::new(),
        &plan,
        &mut |observation| {
            let order = match observation.position {
                StepPosition::StartingPoint => 0,
                StepPosition::Operation(index) => index + 1,
            };
            positions.push((order, image_fingerprint(observation.image), plan.calls()));
        },
    );
    positions
}

/// harness 执行器（随机历史默认批那一种跑法：两块 4 GiB 的盘、准入按公式判）上「每一步跑 checker、判红就停」时第一次判红在哪一步
/// （步序：起点 0，第 i 步 i + 1）、I-7.4 被抛弃根那一半点名了哪几条根、红了哪几条。跑完没判红交回 None。
/// harness 不再逐步跑池级 checker（`PerStepChecker::Skipped`，判红就停那一档删了）：这里照不逐步跑 checker 的执行器跑一遍，
/// 每一步写过盘的镜像经钩子交出来，按历史的次序判到第一张红的为止（之后的不判）。判红就停那一档的结局与这里一一对上：
/// 第一张红的镜像在执行器自己停下的那一步之前或就是那一步，判红就停那一档就停在它上面、红的就是它的违例；没有，就停在执行器停下的那一步、
/// 一条违例都不带（那一步的镜像不红）；执行器跑完、一张都不红是 None。每张镜像交 `engine` 判（[`checker_reading_of`]）。
pub(super) fn harness_first_red(
    engine: &GivenStatesEngine,
    history: &GeneratedHistory,
) -> Option<(usize, Vec<TimelineRoot>, Vec<&'static str>)> {
    let order_of = |position: StepPosition| match position {
        StepPosition::StartingPoint => 0,
        StepPosition::Operation(index) => index + 1,
    };
    let execution = harness_execution(PerStepChecker::Skipped);
    let mut first_red_step_image: Option<(usize, CheckerReading)> = None;
    let run = execute_history_handing_out_every_step_image(
        history,
        execution,
        &SharedStream::new(),
        &SharedFaultPlan::unarmed(execution.device_width.fixed_geometry()),
        &mut |_| {},
        &mut |step_image| {
            if first_red_step_image.is_none() {
                let reading = checker_reading_of(engine, step_image.image);
                if !reading.red_invariants.is_empty() {
                    first_red_step_image = Some((order_of(step_image.position), reading));
                }
            }
        },
    );
    let order_where_the_executor_stopped = match &run.ending {
        HistoryEnding::Completed => None,
        HistoryEnding::KnownRed { observation, .. }
        | HistoryEnding::NewFinding { observation, .. } => Some(order_of(observation.position)),
    };
    match (first_red_step_image, order_where_the_executor_stopped) {
        (Some((order, reading)), stopped)
            if stopped.is_none_or(|stopped_order| order <= stopped_order) =>
        {
            Some((
                order,
                reading.abandoned_roots_named_in_the_order_of_the_detail,
                reading.red_invariants,
            ))
        }
        (Some(_) | None, Some(stopped_order)) => Some((stopped_order, Vec::new(), Vec::new())),
        (Some(_) | None, None) => None,
    }
}

/// 装置执行器（今天那一臂）与 harness 执行器逐步比：交回比了几步、对不上的那几步。
pub(super) fn executor_agreement(
    executor: &RandomExecutor,
    trace: &[(usize, u64, FaultCallCounts)],
) -> (usize, Vec<String>) {
    let mut mismatches = Vec::new();
    for (order, fingerprint, calls) in trace {
        let mine = executor.steps.iter().find(|step| step.order == *order);
        let my_calls = executor.calls_after_step.get(*order);
        // 读次数不要求相等：harness 那一侧的稀疏盘答日志环记录槽提示、不读没写过的槽；装置这一侧不答提示、照真盘全环扫
        // （实验量的是真盘的读开销），所以装置的读不少于 harness 的。写与屏障两侧没有这层差别，逐字相等。
        let agrees = mine.is_some_and(|step| step.fingerprint_after == *fingerprint)
            && my_calls.is_some_and(|device_counts| {
                device_counts.writes == calls.writes
                    && device_counts.barriers == calls.barriers
                    && device_counts.reads >= calls.reads
            });
        if !agrees {
            mismatches.push(format!(
                "order{order}:harness_fingerprint={fingerprint:016x}:device_fingerprint={}:harness_calls={}/{}/{}:device_calls={}",
                mine.map_or("none".to_string(), |step| format!("{:016x}", step.fingerprint_after)),
                calls.reads,
                calls.writes,
                calls.barriers,
                my_calls.map_or("none".to_string(), |counts| format!(
                    "{}/{}/{}",
                    counts.reads, counts.writes, counts.barriers
                ))
            ));
        }
    }
    (trace.len(), mismatches)
}

// ------------------------------------------------------------------------
// 13.7 harness 的注入点与崩溃状态怎么映到各臂（第十二节第二段修订）：今天那一臂上，harness 摆的每个注入点换成（步，步内第几次那一种调用），
//      每个崩溃状态换成（它那一段第一个写落在哪一步，那一步里的第几段，段内持久了哪几个写）；打进今天那一臂的产物，别的臂从那份产物读回来。
// ------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FaultPointKey {
    pub(super) index: usize,
    pub(super) fault: InjectedFault,
    pub(super) order: usize,
    pub(super) ordinal_within_the_step: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CrashPointKey {
    pub(super) index: usize,
    pub(super) order_of_the_first_write: usize,
    pub(super) segment_within_the_step: usize,
    pub(super) persisted_within_the_segment: Vec<bool>,
}

pub(super) fn render_injected_fault(fault: InjectedFault) -> String {
    match fault {
        InjectedFault::ReadReturnsCorruptedBytes { flipped_bit } => format!(
            "{}:{}:{}",
            fault.name(),
            flipped_bit.byte_index,
            flipped_bit.bit_index
        ),
        InjectedFault::WriteFails
        | InjectedFault::WriteIsSwallowed
        | InjectedFault::ReadFails
        | InjectedFault::BarrierFails
        | InjectedFault::BarrierIsSwallowed => fault.name().to_string(),
    }
}

pub(super) fn parse_injected_fault(text: &str) -> Option<InjectedFault> {
    let mut parts = text.split(':');
    let name = parts.next()?;
    [
        InjectedFault::WriteFails,
        InjectedFault::WriteIsSwallowed,
        InjectedFault::ReadFails,
        InjectedFault::BarrierFails,
        InjectedFault::BarrierIsSwallowed,
    ]
    .into_iter()
    .find(|fault| fault.name() == name)
    .or_else(|| {
        if name != "read_returns_corrupted_bytes" {
            return None;
        }
        let byte_index = parts.next()?.parse().ok()?;
        let bit_index = parts.next()?.parse().ok()?;
        Some(InjectedFault::ReadReturnsCorruptedBytes {
            flipped_bit: singlefs_verification::fault_injection::FlippedBit {
                byte_index,
                bit_index,
            },
        })
    })
}

pub(super) fn render_persisted(persisted: &[bool]) -> String {
    persisted
        .iter()
        .map(|is_persisted| if *is_persisted { '1' } else { '0' })
        .collect()
}

pub(super) fn parse_persisted(text: &str) -> Vec<bool> {
    text.chars().map(|bit| bit == '1').collect()
}

/// 录制流的下标落在哪一步（起点 0）：那一步开始时录制流的长度不超过它的最后一步。
fn order_of_a_stream_index(stream_length_at_step_start: &[usize], stream_index: usize) -> usize {
    stream_length_at_step_start
        .iter()
        .rposition(|start| *start <= stream_index)
        .unwrap_or(0)
}

/// 一个分支的录制流切成写表与段（harness 崩溃注入同一切法），外加每一段第一个写落在哪一步。
pub(super) struct SegmentedStream {
    pub(super) writes: Vec<RetainedWrite>,
    pub(super) segments: Vec<Vec<usize>>,
    pub(super) order_of_the_first_write: Vec<usize>,
    order_of_each_write: Vec<usize>,
}

pub(super) fn segmented_stream_of(executor: &RandomExecutor) -> SegmentedStream {
    let operations = executor.stream.retained_operations();
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations,
        &HistoryDeviceWidth::FourGibibytes.fixed_geometry(),
    );
    let order_of_each_write: Vec<usize> = stream_indexes
        .iter()
        .map(|stream_index| {
            order_of_a_stream_index(&executor.stream_length_at_step_start, *stream_index)
        })
        .collect();
    let order_of_the_first_write = segments
        .iter()
        .map(|segment| {
            segment
                .first()
                .map_or(0, |first| order_of_each_write[*first])
        })
        .collect();
    SegmentedStream {
        writes,
        segments,
        order_of_the_first_write,
        order_of_each_write,
    }
}

impl SegmentedStream {
    /// 第 `segment_index` 段在它第一个写那一步里是第几段。
    pub(super) fn segment_within_the_step(&self, segment_index: usize) -> usize {
        let order = self.order_of_the_first_write[segment_index];
        self.order_of_the_first_write[..segment_index]
            .iter()
            .filter(|other| **other == order)
            .count()
    }

    pub(super) fn find(&self, order: usize, segment_within_the_step: usize) -> Option<usize> {
        self.order_of_the_first_write
            .iter()
            .enumerate()
            .filter(|(_, first)| **first == order)
            .nth(segment_within_the_step)
            .map(|(index, _)| index)
    }

    /// 崩溃状态的镜像：更早的段整段持久，这一段按 `persisted` 挑，更晚的一个都没有（harness 崩溃注入同一摆法）。
    fn crash_image(&self, segment_index: usize, persisted: &[bool]) -> MemoryPool {
        let segment = &self.segments[segment_index];
        let first = segment[0];
        let mut image = MemoryPool::with_devices(
            &[DeviceIdentity(0), DeviceIdentity(1)],
            history_device_bytes(),
        );
        image.apply_writes(&self.writes[..first]);
        for (within, is_persisted) in persisted.iter().enumerate() {
            if *is_persisted {
                image.apply_writes(&self.writes[first + within..=first + within]);
            }
        }
        image
    }

    /// 崩溃在哪一步：这一段第一个没持久的写落在的那一步（harness `CrashPoint::step` 同一取法）。
    pub(super) fn crash_order(&self, segment_index: usize, persisted: &[bool]) -> usize {
        let segment = &self.segments[segment_index];
        let withheld = persisted
            .iter()
            .position(|is_persisted| !*is_persisted)
            .unwrap_or(0);
        self.order_of_each_write[segment[withheld]]
    }
}

// ------------------------------------------------------------------------
// 13.8 一段历史在一臂上的一个分支的结果（主路径、注入点、崩溃状态），与结果行。
// ------------------------------------------------------------------------

pub(super) struct BranchResult {
    pub(super) attempt: String,
    pub(super) mounts: Vec<RandomMount>,
    pub(super) cells: Vec<(RandomCell, RandomCellLoss)>,
    pub(super) detail: String,
    /// 注入点分支：注入真的发出的那一次调用是整池第几次那一种调用（没发出 None）。
    pub(super) fired_call_ordinal: Option<u64>,
}

fn cells_of_an_executed_branch(
    executor: &RandomExecutor,
    tail: &TailReading,
    attempt: &str,
    from_order: usize,
    fault_event: Option<(usize, RandomReadBack)>,
) -> Vec<(RandomCell, RandomCellLoss)> {
    let mut cells = Vec::new();
    let mut fault_event_is_a_crash_recovery = false;
    for event in executor
        .crash_recovery_events
        .iter()
        .filter(|event| event.order >= from_order)
    {
        let mut read_backs = vec![("tail_read_back".to_string(), tail.read_back.clone())];
        if let Some((order, reopen)) = &fault_event {
            if *order == event.order {
                read_backs.push(("reopen_read_back".to_string(), reopen.clone()));
                fault_event_is_a_crash_recovery = true;
            }
        }
        let cell = RandomCell {
            name: format!("{attempt}:crash_recovery@{}", event.order),
            kind: "crash_recovery",
            order: event.order,
            protected: executor
                .ledger
                .last_confirmation_before(event.order)
                .cloned(),
            protected_rollback_aware: executor
                .ledger
                .last_confirmed_number_before_rollback_aware(event.order),
            mount: Some(event.mount_index),
            read_backs,
            tail_class: tail.tail_class.clone(),
            destination: Some(destination_of_the_hidden_root(
                event,
                &executor.mounts,
                &executor.ledger,
            )),
        };
        let loss = evaluate_a_random_cell(
            &cell,
            &executor.mounts,
            &executor.first_changes,
            &event.base,
            &tail.end,
            tail.read_back.root,
        );
        cells.push((cell, loss));
    }
    if let Some((order, reopen)) = fault_event {
        if !fault_event_is_a_crash_recovery {
            let cell = RandomCell {
                name: format!("{attempt}:fault@{order}"),
                kind: "fault",
                order,
                protected: executor.ledger.last_confirmation_before(order).cloned(),
                protected_rollback_aware: executor
                    .ledger
                    .last_confirmed_number_before_rollback_aware(order),
                mount: executor
                    .mounts
                    .iter()
                    .position(|mount| mount.order == order),
                read_backs: vec![
                    ("tail_read_back".to_string(), tail.read_back.clone()),
                    ("reopen_read_back".to_string(), reopen),
                ],
                tail_class: tail.tail_class.clone(),
                destination: None,
            };
            let base = executor
                .captured_image
                .clone()
                .unwrap_or_else(|| tail.end.clone());
            let loss = evaluate_a_random_cell(
                &cell,
                &executor.mounts,
                &executor.first_changes,
                &base,
                &tail.end,
                tail.read_back.root,
            );
            cells.push((cell, loss));
        }
    }
    cells
}

/// 一段历史的主路径（不注入）：跑完操作、接尾巴（这一族要的话）、按每一次崩溃恢复抛弃根那一步立格。
/// 每一步的池级 checker 照 `pool_checker_on_every_step` 判（产品那一趟是 [`PoolCheckerOnEveryStep::of_the_family`]）。
pub(super) fn run_the_main_branch<'engine>(
    arm: FourthRunArm,
    segment: &SegmentTwoHistory,
    pool_checker_on_every_step: PoolCheckerOnEveryStep<'engine>,
) -> (RandomExecutor<'engine>, BranchResult) {
    let family = segment.family;
    let mut executor = RandomExecutor::start(
        arm,
        segment.history.starting_point,
        SharedFaultPlan::unarmed(HistoryDeviceWidth::FourGibibytes.fixed_geometry()),
        pool_checker_on_every_step,
    );
    executor.run_the_operations(&segment.history);
    let finished = executor.panicked.is_none()
        && executor.stopped_at_the_first_red.is_none()
        && executor.starting_point_error.is_none();
    let mut cells = Vec::new();
    let mut detail = String::new();
    if family.runs_the_tail() && finished {
        let order_of_the_tail = segment.history.operations.len() + 1;
        match with_panic_capture(|| executor.run_the_tail(order_of_the_tail)) {
            Ok(tail) => {
                detail = format!(
                    "{} q3c_tail={}",
                    tail.read_back.render("tail_read_back"),
                    tail.tail_class
                );
                cells = cells_of_an_executed_branch(&executor, &tail, "main", 0, None);
            }
            Err(panic) => {
                executor.panicked = Some((
                    order_of_the_tail,
                    format!("{}: {}", panic.location, panic.message),
                ));
            }
        }
    }
    let mounts = std::mem::take(&mut executor.mounts);
    (
        executor,
        BranchResult {
            attempt: "main".to_string(),
            mounts,
            cells,
            detail,
            fired_call_ordinal: None,
        },
    )
}

/// 一个注入点的分支：先按主路径的调用数把「步内第几次」换成这一臂的整池序号，再开着注入重跑这段历史、接尾巴；
/// 注入那一步做完时的镜像上冷启动读回（harness 的「重开」）。
pub(super) fn run_a_fault_branch(
    arm: FourthRunArm,
    segment: &SegmentTwoHistory,
    main: &RandomExecutor,
    point: &FaultPointKey,
) -> BranchResult {
    let attempt = format!("fault{}", point.index);
    let kind = point.fault.call_kind();
    let calls_before = if point.order == 0 {
        FaultCallCounts::default()
    } else {
        main.calls_after_step
            .get(point.order - 1)
            .copied()
            .unwrap_or_default()
    };
    let calls_after = main.calls_after_step.get(point.order).copied();
    let reachable = calls_after.is_some_and(|after| {
        calls_before.of(kind) + point.ordinal_within_the_step <= after.of(kind)
    });
    if !reachable {
        return BranchResult {
            attempt,
            mounts: Vec::new(),
            cells: Vec::new(),
            fired_call_ordinal: None,
            detail: format!(
                "fault={} order={} ordinal_within_the_step={} fault_not_reached=true arm_calls_in_the_step={}",
                render_injected_fault(point.fault),
                point.order,
                point.ordinal_within_the_step,
                calls_after.map_or("none".to_string(), |after| {
                    (after.of(kind) - calls_before.of(kind)).to_string()
                })
            ),
        };
    }
    let arm_ordinal = calls_before.of(kind) + point.ordinal_within_the_step;
    let plan = SharedFaultPlan::armed(
        HistoryDeviceWidth::FourGibibytes.fixed_geometry(),
        FaultSchedule::the_nth_call_across_the_pool(point.fault, arm_ordinal),
    );
    let mut executor = RandomExecutor::start(
        arm,
        segment.history.starting_point,
        plan.clone(),
        PoolCheckerOnEveryStep::Skipped,
    );
    executor.capture_after_order = Some(point.order);
    if point.order == 0 {
        executor.captured_image = Some(executor.image());
    }
    executor.run_the_operations(&segment.history);
    let fired = plan.fired_count();
    let fired_call_ordinal = plan
        .fired()
        .first()
        .map(|fired_fault| fired_fault.call_ordinal_across_the_pool);
    let reopen = executor.captured_image.as_ref().map(|image| {
        let report = recover(image, JournalPolicy::Consult);
        classify_random_read_back(
            &report,
            &executor.ledger,
            executor
                .ledger
                .last_confirmed_number_before(point.order + 1),
            executor
                .ledger
                .last_confirmed_number_before_rollback_aware(point.order + 1),
        )
    });
    let finished = executor.panicked.is_none()
        && executor.stopped_at_the_first_red.is_none()
        && executor.starting_point_error.is_none();
    let mut cells = Vec::new();
    let mut detail = format!(
        "fault={} order={} ordinal_within_the_step={} arm_ordinal={arm_ordinal} fired={fired} {} starting_point_error={} panicked={}",
        render_injected_fault(point.fault),
        point.order,
        point.ordinal_within_the_step,
        reopen
            .as_ref()
            .map_or("reopen_read_back=none".to_string(), |read_back| read_back
                .render("reopen_read_back")),
        executor.starting_point_error.is_some(),
        executor.panicked.is_some()
    );
    if let Some(read_back) = &reopen {
        detail.push_str(&format!(
            " reopen_failure={:?}",
            read_back.failure_debug.clone().unwrap_or_default()
        ));
    }
    if finished {
        let order_of_the_tail = segment.history.operations.len() + 1;
        match with_panic_capture(|| executor.run_the_tail(order_of_the_tail)) {
            Ok(tail) => {
                detail.push_str(&format!(
                    " {} q3c_tail={}",
                    tail.read_back.render("tail_read_back"),
                    tail.tail_class
                ));
                cells = cells_of_an_executed_branch(
                    &executor,
                    &tail,
                    &attempt,
                    point.order,
                    reopen.map(|read_back| (point.order, read_back)),
                );
            }
            Err(panic) => {
                detail.push_str(&format!(
                    " tail_panic={:?}",
                    format!("{}: {}", panic.location, panic.message)
                ));
            }
        }
    }
    let mounts = std::mem::take(&mut executor.mounts)
        .into_iter()
        .filter(|mount| mount.order >= point.order)
        .collect();
    BranchResult {
        attempt,
        mounts,
        cells,
        detail,
        fired_call_ordinal,
    }
}

/// 崩溃分支那次可写挂载之后读回用的账（第 4 次跑 seg2b 修的装置记账撞键，调查报告 `research/prompts/e158-r4-seg2-ledger-investigation-report.md`
/// 第一节）：账按根的键 (实例, txg) 记内容号，键里没有时间线这一维；分支挂载在崩溃镜像上取号、发布，会与主路径后来的某次发布落到同一个键上，
/// 按主路径的账查就查到主路径那一版。分支自己的账 = 主路径的账拷一份（崩溃镜像只含主路径写序列的前缀，镜像里的根与主路径同源），
/// 再把分支那次挂载写出的根——挂载之后读得出、崩溃镜像里没有同一条（键与根记录逐字节）的——改记成这次挂载承接的那一版。
/// 承接的那一版照主路径 `observed_mount` 同一条规则：交回 `Ok` 记 E 的内容号，查不到记字面最后确认那一版；拒了记装置判的 E 的内容号，查不到同上。
fn ledger_after_the_mount_of_a_crash_branch(
    main_ledger: &RandomLedger,
    crash_image: &MemoryPool,
    after_mount: &MemoryPool,
    mount_returned_ok: bool,
    mount: &RandomMount,
) -> RandomLedger {
    let carried_when_ok = mount.effective_content.unwrap_or(mount.literal_last);
    let carried = if mount_returned_ok {
        carried_when_ok
    } else {
        mount.device_effective_content.unwrap_or(carried_when_ok)
    };
    let roots_with_their_records = |pool: &MemoryPool| -> BTreeSet<(TimelineRoot, Vec<u8>)> {
        independent_geometry(pool)
            .map(|geometry| {
                checker_image::valid_roots(pool, &geometry)
                    .into_iter()
                    .map(|(_, _, view)| ((view.instance, view.checkpoint_txg), view.record_bytes))
                    .collect()
            })
            .unwrap_or_default()
    };
    let roots_in_the_crash_image = roots_with_their_records(crash_image);
    let mut ledger = main_ledger.clone();
    for (root, record_bytes) in roots_with_their_records(after_mount) {
        if !roots_in_the_crash_image.contains(&(root, record_bytes)) {
            ledger.carry(root, carried);
        }
    }
    ledger
}

/// 一个崩溃状态的分支（r3 登记第 373–391 行实七-乙那一行）：主路径的写序列上按映射摆出崩溃镜像，冷启动读回，再可写挂载一次、读回一次。
pub(super) fn run_a_crash_branch(
    arm: FourthRunArm,
    main: &RandomExecutor,
    stream: &SegmentedStream,
    point: &CrashPointKey,
) -> BranchResult {
    let attempt = format!("crash{}", point.index);
    let Some(segment_index) = stream.find(
        point.order_of_the_first_write,
        point.segment_within_the_step,
    ) else {
        return BranchResult {
            attempt,
            mounts: Vec::new(),
            cells: Vec::new(),
            fired_call_ordinal: None,
            detail: format!(
                "not_constructible=segment_missing order_of_the_first_write={} segment_within_the_step={}",
                point.order_of_the_first_write, point.segment_within_the_step
            ),
        };
    };
    let writes_in_the_segment = stream.segments[segment_index].len();
    if writes_in_the_segment != point.persisted_within_the_segment.len() {
        return BranchResult {
            attempt,
            mounts: Vec::new(),
            cells: Vec::new(),
            fired_call_ordinal: None,
            detail: format!(
                "not_constructible=segment_length arm_writes_in_the_segment={writes_in_the_segment} registered_writes={}",
                point.persisted_within_the_segment.len()
            ),
        };
    }
    let crash_order = stream.crash_order(segment_index, &point.persisted_within_the_segment);
    let image = stream.crash_image(segment_index, &point.persisted_within_the_segment);
    let literal_last = main.ledger.last_confirmed_number_before(crash_order);
    let rollback_aware_last = main
        .ledger
        .last_confirmed_number_before_rollback_aware(crash_order);
    let crash_read_back = classify_random_read_back(
        &recover(&image, JournalPolicy::Consult),
        &main.ledger,
        literal_last,
        rollback_aware_last,
    );
    let mut devices = plain_devices_of(&image);
    let parameters = HistoryDeviceWidth::FourGibibytes.parameters();
    let (mounted, reading) = observed_call(&mut devices, &[], true, |observed| {
        mount_writable_with_space_admission(
            &parameters,
            observed,
            SpaceAdmission::JudgedByTheFormula,
        )
    });
    let phase = if main
        .crash_recovery_events
        .iter()
        .any(|event| event.order < crash_order)
    {
        "after_crash_recovery"
    } else {
        "before_first_crash_recovery"
    };
    let mount = evaluate_a_random_mount(
        arm,
        &main.ledger,
        &image,
        crash_order,
        &attempt,
        "after_crash",
        phase,
        &[],
        &mounted,
        &reading,
    );
    let after_mount = memory_pool_of(&devices, history_device_bytes());
    let ledger_of_the_branch = ledger_after_the_mount_of_a_crash_branch(
        &main.ledger,
        &image,
        &after_mount,
        mounted.is_ok(),
        &mount,
    );
    let after_mount_read_back = classify_random_read_back(
        &recover(&after_mount, JournalPolicy::Consult),
        &ledger_of_the_branch,
        literal_last,
        rollback_aware_last,
    );
    let mounts = vec![mount];
    let cell = RandomCell {
        name: format!("{attempt}:crash@{crash_order}"),
        kind: "crash",
        order: crash_order,
        protected: main.ledger.last_confirmation_before(crash_order).cloned(),
        protected_rollback_aware: rollback_aware_last,
        mount: Some(0),
        read_backs: vec![
            ("crash_read_back".to_string(), crash_read_back.clone()),
            (
                "after_mount_read_back".to_string(),
                after_mount_read_back.clone(),
            ),
        ],
        tail_class: "na".to_string(),
        destination: None,
    };
    let loss = evaluate_a_random_cell(
        &cell,
        &mounts,
        &BTreeMap::new(),
        &image,
        &after_mount,
        after_mount_read_back.root,
    );
    BranchResult {
        attempt,
        mounts,
        cells: vec![(cell, loss)],
        fired_call_ordinal: None,
        detail: format!(
            "segment_index={segment_index} crash_order={crash_order} persisted={} {} crash_failure={:?} {}",
            render_persisted(&point.persisted_within_the_segment),
            crash_read_back.render("crash_read_back"),
            crash_read_back.failure_debug.clone().unwrap_or_default(),
            after_mount_read_back.render("after_mount_read_back")
        ),
    }
}

#[cfg(test)]
mod tests;
