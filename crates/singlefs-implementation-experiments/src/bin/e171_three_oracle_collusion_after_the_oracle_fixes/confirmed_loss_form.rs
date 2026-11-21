//! 确认丢失那一形（confirmed loss form）：形状、操作、备好的流、回比 crates、跑与判各形状、合计的报告。

use super::*;

// ───────────────────────────── Q5「已确认的丢了」那一形（读法 7） ─────────────────────────────

/// 照读法 7 改过的流二写流：第 2 次调用里第一条 journal 记录写之后、返回点之前的每一道屏障删掉，这次调用写根槽的那次 FUA 写改成普通写，
/// 截在 `truncation`（主取样点：第 3 次调用发出的第一道屏障；方向相反的取样点：第 2 次调用的返回点）。写的字节一个不改。
/// 下标都是 mkfs 之后录制流里的（没改过的那一份的）下标。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ConfirmedLossShape {
    removed_barriers: Vec<usize>,
    root_writes_made_plain: Vec<usize>,
    truncation: usize,
    reverse_truncation: usize,
}

/// 在驱动到第 3 次调用的流二上算读法 7 的造法下标（不改任何东西）。
fn confirmed_loss_shape_of(
    operations: &[RetainedOperation],
    calls: &[RecordedCall],
) -> ConfirmedLossShape {
    let geometry = experiment_pool::geometry();
    let confirmed = &calls[CONFIRMED_LOSS_CONFIRMED_CALL];
    let in_flight = &calls[CONFIRMED_LOSS_IN_FLIGHT_CALL];
    let inside_the_confirmed_call = confirmed.first_operation..confirmed.return_point;
    let first_journal_record = inside_the_confirmed_call
        .clone()
        .find(|index| geometry.classify(&operations[*index].operation) == StepKind::JournalRecord)
        .expect("第一个文件那次发布写 journal 记录");
    let removed_barriers = (first_journal_record + 1..confirmed.return_point)
        .filter(|index| operations[*index].operation.kind == RecordedOperationKind::Barrier)
        .collect();
    let root_writes_made_plain = inside_the_confirmed_call
        .filter(|index| {
            operations[*index].operation.kind == RecordedOperationKind::WriteForceUnitAccess
                && geometry.classify(&operations[*index].operation) == StepKind::RootRecordFua
        })
        .collect();
    let truncation = (in_flight.first_operation..operations.len())
        .find(|index| operations[*index].operation.kind == RecordedOperationKind::Barrier)
        .unwrap_or(operations.len());
    ConfirmedLossShape {
        removed_barriers,
        root_writes_made_plain,
        truncation,
        reverse_truncation: confirmed.return_point,
    }
}

/// 改过、截过的写流；`at_the_return_point` 选方向相反的取样点。交回 (写流, 原下标 → 新下标；删掉的屏障没有)。
fn confirmed_loss_operations(
    operations: &[RetainedOperation],
    shape: &ConfirmedLossShape,
    at_the_return_point: bool,
) -> (Vec<RetainedOperation>, Vec<Option<usize>>) {
    let end = if at_the_return_point {
        shape.reverse_truncation
    } else {
        shape.truncation
    };
    let mut kept = Vec::new();
    let mut new_index_of = Vec::new();
    for (index, retained) in operations.iter().enumerate().take(end) {
        if shape.removed_barriers.contains(&index) {
            new_index_of.push(None);
            continue;
        }
        let mut changed = retained.clone();
        if shape.root_writes_made_plain.contains(&index) {
            changed.operation.kind = RecordedOperationKind::Write;
        }
        new_index_of.push(Some(kept.len()));
        kept.push(changed);
    }
    (kept, new_index_of)
}

/// 驱动流二到第 3 次调用（模型跟到第 2 次调用），交回驱动完的流（[`LiveStream::run_the_second_stream_through_the_first_overwrite`]）。
pub(super) fn second_stream_through_the_first_overwrite(switches: RunSwitches) -> LiveStream {
    let mut live = LiveStream::start(
        StreamName::FixedScriptReuseAfterRaisingFloorThenUnmount,
        "e171r1-confirmed-loss-form",
        switches,
    );
    live.run_the_second_stream_through_the_first_overwrite();
    live
}

/// Q5 那一形截在哪里。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ConfirmedLossTruncation {
    /// 主取样点：第 3 次调用发出的第一道屏障之前（读法 7）。
    BeforeTheInFlightBarrier,
    /// 方向相反的取样点：第 2 次调用的返回点（读法 7、第八节）。
    AtTheConfirmedReturnPoint,
    /// 「等价类对拍」那一形：第 3 次调用发出的前这么多个写之后（登记修订 2 第 2.1 节候选二最后一条，即候选三的造法）。
    AfterTheFirstInFlightWrites(usize),
}

/// 第 3 次调用发出的第 `writes_kept` 个写之后的下标（按发出次序数写，屏障不数）；主取样点截断之前第 3 次调用发出的写不够这么多个，
/// 截在主取样点的截断处（那里第 3 次调用的写已经全留着）。
fn truncation_after_the_first_writes_of_the_in_flight_call(
    operations: &[RetainedOperation],
    calls: &[RecordedCall],
    shape: &ConfirmedLossShape,
    writes_kept: usize,
) -> usize {
    let in_flight = &calls[CONFIRMED_LOSS_IN_FLIGHT_CALL];
    let mut writes_seen = 0usize;
    for (index, retained) in operations
        .iter()
        .enumerate()
        .take(shape.truncation)
        .skip(in_flight.first_operation)
    {
        match retained.operation.kind {
            RecordedOperationKind::Write
            | RecordedOperationKind::WriteForceUnitAccess
            | RecordedOperationKind::WriteZeroes
            | RecordedOperationKind::Discard => writes_seen += 1,
            RecordedOperationKind::Barrier => {}
        }
        if writes_seen == writes_kept {
            return index + 1;
        }
    }
    shape.truncation
}

/// Q5 那一形的流：改过、截过的写流重新切段，调用的下标照改之后的写流换，第 3 次调用记「在途」、返回点取截断处。
pub(super) fn confirmed_loss_prepared_stream(
    live: LiveStream,
    truncation: ConfirmedLossTruncation,
) -> (PreparedStream, ConfirmedLossShape) {
    let mkfs_operation_count = live.formatted.mkfs_operation_count;
    let all_operations = live.formatted.retained_operations();
    let operations = &all_operations[mkfs_operation_count..];
    let mut shape = confirmed_loss_shape_of(operations, &live.driver.calls);
    let at_the_return_point = match truncation {
        ConfirmedLossTruncation::BeforeTheInFlightBarrier => false,
        ConfirmedLossTruncation::AtTheConfirmedReturnPoint => true,
        ConfirmedLossTruncation::AfterTheFirstInFlightWrites(writes_kept) => {
            shape.truncation = truncation_after_the_first_writes_of_the_in_flight_call(
                operations,
                &live.driver.calls,
                &shape,
                writes_kept,
            );
            false
        }
    };
    let (kept, new_index_of) = confirmed_loss_operations(operations, &shape, at_the_return_point);
    let remap = |old: usize| -> usize {
        new_index_of
            .iter()
            .take(old)
            .filter(|new_index| new_index.is_some())
            .count()
    };
    let mut calls: Vec<RecordedCall> = live
        .driver
        .calls
        .iter()
        .take(if at_the_return_point {
            CONFIRMED_LOSS_IN_FLIGHT_CALL
        } else {
            CONFIRMED_LOSS_IN_FLIGHT_CALL + 1
        })
        .cloned()
        .collect();
    for call in &mut calls {
        call.first_operation = remap(call.first_operation);
        call.return_point = remap(call.return_point);
    }
    if let Some(in_flight) = calls.get_mut(CONFIRMED_LOSS_IN_FLIGHT_CALL) {
        in_flight.outcome = CallOutcome::InFlightAtTheTruncation;
        in_flight.return_point = kept.len();
        in_flight.return_value = None;
    }
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes_and_entries(
        &kept,
        &[],
        &experiment_pool::geometry(),
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .unwrap_or(usize::MAX);
    let prepared = PreparedStream {
        name: StreamName::FixedScriptReuseAfterRaisingFloorThenUnmount,
        base: live.formatted.memory_pool_after_mkfs(),
        operations: kept,
        writes,
        segments,
        stream_indexes,
        calls,
        judged_root_index,
        versions: live.versions.clone(),
        committed_versions: live.driver.committed_versions.clone(),
        model_stopped_at_call: live.driver.model_stopped_at_call,
        checks: Vec::new(),
        ending: ending_of_the_driver(&live.driver),
    };
    (prepared, shape)
}

/// 末段每个写的取法数之积（整写 2 种，可撕裂的 3 种；第七节 A10），与末段写数、可撕裂写数。
pub(super) fn last_segment_product(
    stream: &PreparedStream,
    tables: &CrashStateEnumerationTables,
) -> (usize, usize, u64) {
    let last_segment = stream.segments.last().cloned().unwrap_or_default();
    let tearable = last_segment
        .iter()
        .filter(|write_index| tables.is_tearable_by_write[**write_index])
        .count();
    let product = last_segment.iter().fold(1u64, |product, write_index| {
        product
            * if tables.is_tearable_by_write[*write_index] {
                3
            } else {
                2
            }
    });
    (last_segment.len(), tearable, product)
}

/// S1 里 Q5 造法在基线上的几样（5.3）：驱动一遍流二前四次调用，算造法下标、末段写数、可撕裂写数与全枚举态数，对本地常量。
pub(super) fn confirmed_loss_form_checks_against_the_crates() -> Vec<StopCheck> {
    let live = second_stream_through_the_first_overwrite(RunSwitches::default());
    let (prepared, shape) =
        confirmed_loss_prepared_stream(live, ConfirmedLossTruncation::BeforeTheInFlightBarrier);
    let tables = crash_state_enumeration_tables(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        &full_expansion,
    );
    let (writes, tearable, product) = last_segment_product(&prepared, &tables);
    // 等价类的起点分法（候选二）：只看末段每个写属于哪次调用与各自的进制，不跑恢复（细分要跑恢复，归 `confirmed-loss-form` 那一趟）。
    let digits = LastSegmentDigits::of(&prepared, &tables);
    let partition = ConfirmedLossPartition::before_any_refinement(
        &call_index_of_each_write(&prepared),
        &digits,
        CONFIRMED_LOSS_IN_FLIGHT_CALL,
    );
    let key_choices = partition.keyed_assignments(&digits);
    let states_of_a_landed_class = partition.states_with_the_in_flight_writes_landed(&digits);
    // 「等价类对拍」那一形的三样（修订 5 第 5.2 节 S1 新加三条）。
    let (pairing_stream, _) = confirmed_loss_prepared_stream(
        second_stream_through_the_first_overwrite(RunSwitches::default()),
        ConfirmedLossTruncation::AfterTheFirstInFlightWrites(
            EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES_KEPT,
        ),
    );
    let pairing_tables = crash_state_enumeration_tables(
        &pairing_stream.base,
        &pairing_stream.writes,
        &pairing_stream.segments,
        &full_expansion,
    );
    let pairing_facts = equivalence_pairing_shape_facts(&pairing_stream, &pairing_tables);
    vec![
        StopCheck::new(
            "equivalence_pairing_in_flight_writes",
            LOCAL_EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES,
            pairing_facts.in_flight_writes.len(),
        ),
        StopCheck::new(
            "equivalence_pairing_states",
            LOCAL_EQUIVALENCE_PAIRING_STATES,
            pairing_facts.states,
        ),
        StopCheck::new(
            "equivalence_pairing_mirrored_pairs",
            LOCAL_EQUIVALENCE_PAIRING_MIRRORED_PAIRS,
            pairing_facts.mirrored_pairs,
        ),
        StopCheck::new(
            "confirmed_loss_key_choices",
            LOCAL_CONFIRMED_LOSS_KEY_CHOICES,
            key_choices,
        ),
        StopCheck::new(
            "confirmed_loss_in_flight_writes",
            LOCAL_CONFIRMED_LOSS_IN_FLIGHT_WRITES,
            partition.class_positions.len(),
        ),
        StopCheck::new(
            "confirmed_loss_classes",
            LOCAL_CONFIRMED_LOSS_CLASSES,
            key_choices * 2,
        ),
        StopCheck::new(
            "confirmed_loss_states_of_a_class_with_in_flight_writes_landed",
            LOCAL_CONFIRMED_LOSS_STATES_OF_A_CLASS_WITH_IN_FLIGHT_WRITES_LANDED,
            states_of_a_landed_class,
        ),
        StopCheck::new(
            "confirmed_loss_judged_states",
            LOCAL_CONFIRMED_LOSS_JUDGED_STATES,
            partition.judged_state_count(&digits),
        ),
        // 各类态数之和（「C 一个没落」各 1 态加「C 至少落一个」各那么多态）等于全枚举态数：写成加法（第七节 A10）。
        StopCheck::new(
            "confirmed_loss_class_states_sum_to_the_states",
            LOCAL_CONFIRMED_LOSS_STATES,
            key_choices * (1 + states_of_a_landed_class),
        ),
        StopCheck::new(
            "confirmed_loss_removed_barriers",
            LOCAL_CONFIRMED_LOSS_REMOVED_BARRIERS.to_vec(),
            shape.removed_barriers.clone(),
        ),
        StopCheck::new(
            "confirmed_loss_root_write_made_plain",
            vec![LOCAL_CONFIRMED_LOSS_ROOT_WRITE],
            shape.root_writes_made_plain.clone(),
        ),
        StopCheck::new(
            "confirmed_loss_truncation",
            LOCAL_CONFIRMED_LOSS_TRUNCATION,
            shape.truncation,
        ),
        StopCheck::new(
            "confirmed_loss_reverse_truncation",
            LOCAL_CONFIRMED_LOSS_REVERSE_TRUNCATION,
            shape.reverse_truncation,
        ),
        StopCheck::new(
            "confirmed_loss_last_segment_writes",
            LOCAL_CONFIRMED_LOSS_LAST_SEGMENT_WRITES,
            writes,
        ),
        StopCheck::new(
            "confirmed_loss_last_segment_tearable_writes",
            LOCAL_CONFIRMED_LOSS_LAST_SEGMENT_TEARABLE_WRITES,
            tearable,
        ),
        StopCheck::new(
            "confirmed_loss_states",
            LOCAL_CONFIRMED_LOSS_STATES,
            product,
        ),
        StopCheck::new(
            "confirmed_loss_states_equal_the_last_segment_and_the_fully_persisted_state",
            product,
            tables
                .state_ranges_by_segment
                .last()
                .map_or(0, |range| range.end - range.start)
                + 1,
        ),
    ]
}

/// `confirmed-loss-form` 模式：造那一形、每态算 O1 三个子列与附带的 O2，导出中性格式（`<输出>/confirmed_loss_form/<流二目录名>/`），
/// 打 Q5 的计数（读法 7、第六节 Q5-有效 / Q5-判 / Q5-阴性、第八节轨迹）。方向相反的取样点末段全枚举；主取样点按等价类判
/// （登记修订 2 第 2.1 节候选二、修订 3）：先过「等价类对拍」与阳性对照「判态路 panic 单列」，再按类判主取样点，类核对 ①–④ 或对拍不过走停机 S13。
/// 次序照修订 5 第 5.3 节：S1 → 造主取样点（或反向取样点）那一形 → 造对拍那一形 → [`judge_the_confirmed_loss_shapes`]（先看两形的根写）。
pub(super) fn run_confirmed_loss_form(command_line: &ConfirmedLossCommandLine) -> i32 {
    let mut lines = ResultLines {
        emitted: 0,
        collected: None,
    };
    let truncation = if command_line.truncate_at_the_return_point {
        ConfirmedLossTruncation::AtTheConfirmedReturnPoint
    } else {
        ConfirmedLossTruncation::BeforeTheInFlightBarrier
    };
    let sampling_point = if command_line.truncate_at_the_return_point {
        "reverse_at_the_return_point_of_call_2"
    } else {
        "main_before_the_first_barrier_of_call_3"
    };
    lines.emit_result_line(&format!(
        "name=config segment=s1 mode=confirmed-loss-form sampling_point={sampling_point}"
    ));
    let mut failed_checks = 0usize;
    for check in confirmed_loss_form_checks_against_the_crates() {
        failed_checks += usize::from(!check.holds());
        lines.emit_result_line(&format!(
            "name=s1 stream=confirmed_loss_form check={} local={} crates={} holds={}",
            check.name,
            field_text(&check.local),
            field_text(&check.crates),
            check.holds()
        ));
    }
    let started = Instant::now();
    let live = second_stream_through_the_first_overwrite(RunSwitches::default());
    let (prepared, shape) = confirmed_loss_prepared_stream(live, truncation);
    let drive_nanoseconds = started.elapsed().as_nanos();
    // 方向相反的取样点没有对拍那一形（修订 5 第 5.3 节：反向取样点只在导出之前先判它自己那一形）。
    let pairing = (!command_line.truncate_at_the_return_point).then(|| {
        confirmed_loss_prepared_stream(
            second_stream_through_the_first_overwrite(RunSwitches::default()),
            ConfirmedLossTruncation::AfterTheFirstInFlightWrites(
                EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES_KEPT,
            ),
        )
        .0
    });
    judge_the_confirmed_loss_shapes(
        &mut lines,
        command_line,
        ConfirmedLossShapes {
            sampling_point,
            main: prepared,
            shape,
            drive_nanoseconds,
            pairing,
            // 跑法里三格一律空、命令行不开放（登记修订 6 第 6.2 节第 3 条；修订 5 第 5.1 节「注入只经装置内部的开关进」）。
            piece_panic_injections: ConfirmedLossPiecePanicInjections {
                pairing: None,
                piece_panic_control: None,
                sampling_point_export: None,
            },
        },
        failed_checks,
    )
}

/// Q5 那一次判态要的两形：主取样点（或反向取样点）那一形与它的造法下标、驱动它的耗时，与对拍那一形（反向取样点没有）；
/// 另带判态路 panic 的三格注入（跑法里都空）。
struct ConfirmedLossShapes {
    pub(super) sampling_point: &'static str,
    pub(super) main: PreparedStream,
    shape: ConfirmedLossShape,
    pub(super) drive_nanoseconds: u128,
    pub(super) pairing: Option<PreparedStream>,
    piece_panic_injections: ConfirmedLossPiecePanicInjections,
}

/// Q5 判决路的注入入口（登记修订 6 第 6.2 节第 3 条；判决 b29 第二节 L1 末行、攻方改法 A1、A2）：
/// [`judge_the_confirmed_loss_shapes`] 里收 unattributed 那四个调用点前面的三处判态各一格，可空。
/// 只有内联用例 U13–U16 交非空的，跑法（[`run_confirmed_loss_form`]）一律交三格空。
struct ConfirmedLossPiecePanicInjections {
    /// 对拍那一遍：空时与 [`equivalence_pairing_of`] 同义。
    pub(super) pairing: Option<PiecePanicInjection>,
    /// 阳性对照「判态路 panic 单列」那一遍：空时照 [`piece_panic_control_injection`] 造；不空时**替换**内置的那三条规则。
    piece_panic_control: Option<PiecePanicInjection>,
    /// 主取样点或反向取样点那一次导出（[`export_prepared_stream`] 交给判态的 [`ObservationContext`]）。
    sampling_point_export: Option<PiecePanicInjection>,
}

/// 两形里头一个没有可判的根槽写的（`judged_root_index` 是 `usize::MAX`）：`main` 或 `pairing`（修订 5 第 5.3 节）；都有交 `None`。
fn shape_without_a_root_write(shapes: &ConfirmedLossShapes) -> Option<&'static str> {
    if shapes.main.judged_root_index == usize::MAX {
        return Some("main");
    }
    if shapes
        .pairing
        .as_ref()
        .is_some_and(|pairing| pairing.judged_root_index == usize::MAX)
    {
        return Some("pairing");
    }
    None
}

/// `run_confirmed_loss_form` 判完 S1 之后的那一段：先看两形的根写（修订 5 第 5.3 节），再对拍、阳性对照「判态路 panic 单列」、
/// 主取样点的分类与判态（反向取样点全枚举）；结果行打进 `lines`，交回退出码（0 或 3）。
fn judge_the_confirmed_loss_shapes(
    lines: &mut ResultLines,
    command_line: &ConfirmedLossCommandLine,
    shapes: ConfirmedLossShapes,
    mut failed_checks: usize,
) -> i32 {
    let sampling_point = shapes.sampling_point;
    if let Some(shape_name) = shape_without_a_root_write(&shapes) {
        // 那一形截出来的写流里没有可判的根槽写：对拍、阳性对照与类核对都不跑，不是 S13；Q5 记「不够判」、不进判定。
        lines.emit_result_line(&format!(
            "name=q5 sampling_point={sampling_point} not_run=no_root_write shape={shape_name}"
        ));
        lines.emit_result_line(&format!(
            "name=verdict s1_failed_checks={failed_checks} s13=false q5_not_run=no_root_write piece_panic_states=- unattributed_panic_states=0"
        ));
        lines.finish();
        return 3;
    }
    if let Some(pairing_stream) = &shapes.pairing {
        // 装置先过「等价类对拍」再跑主取样点（修订 2 第 2.1 节「定之前与定之后」末句）。
        let pairing = equivalence_pairing_with_injected_piece_panics(
            pairing_stream,
            shapes.piece_panic_injections.pairing.as_ref(),
        );
        let in_flight_writes_kept = call_index_of_each_write(pairing_stream)
            .iter()
            .filter(|call| **call == Some(CONFIRMED_LOSS_IN_FLIGHT_CALL))
            .count();
        lines.emit_result_line(&format!(
            "name=q5_equivalence_pairing in_flight_writes_kept={in_flight_writes_kept} states={} product_of_the_choices={} class_states={} classes={} mismatching_states={} crash_judge_not_applicable_states={} {} s13={}",
            pairing.states,
            pairing.product_of_the_choices,
            pairing.class_states,
            pairing.classes,
            pairing.mismatching_states,
            pairing
                .observations
                .iter()
                .filter(|observation| observation.crash_judge == CrashJudgeVerdict::NotApplicable)
                .count(),
            equivalence_check_fields(&pairing.classification, &pairing.checks),
            pairing.stops_at_the_class_check()
        ));
        lines.emit_result_line(&format!(
            "name=q5_partition sampling_point=pairing {}",
            partition_fields(&pairing.classification, &pairing.checks)
        ));
        lines.emit_result_line(&confirmed_loss_form_piece_panics_line(
            "pairing",
            &pairing.observations,
        ));
        // S1（修订 5 第 5.2 节）：对拍那一行打的实际截的 k 与分法里的 C 写数相等。
        let kept_check = StopCheck::new(
            "equivalence_pairing_in_flight_writes_kept_equal_the_partition",
            in_flight_writes_kept,
            pairing.classification.in_flight_writes,
        );
        failed_checks += usize::from(!kept_check.holds());
        lines.emit_result_line(&format!(
            "name=s1 stream=confirmed_loss_form check={} local={} crates={} holds={}",
            kept_check.name,
            field_text(&kept_check.local),
            field_text(&kept_check.crates),
            kept_check.holds()
        ));
        if let Some(exit_code) =
            stop_the_confirmed_loss_form_and_write_the_verdict_when_a_panic_is_unattributed(
                lines,
                failed_checks,
                &pairing.observations,
            )
        {
            return exit_code;
        }
        if pairing.stops_at_the_class_check() {
            lines.emit_result_line(&format!(
                "name=verdict s1_failed_checks={failed_checks} s13=equivalence_pairing q5_not_run=equivalence_pairing_failed piece_panic_states=- unattributed_panic_states=0"
            ));
            lines.finish();
            return 3;
        }
        let control = piece_panic_control_of(
            pairing_stream,
            &pairing,
            shapes.piece_panic_injections.piece_panic_control.as_ref(),
        );
        lines.emit_result_line(&format!(
            "name=q5_piece_panic_control states={} pool_checker={} ignored_recovery={} record_checker={} other_pieces={} consulted_recovery={} consulted_recovery_of_the_plain_pairing={} o1_columns_differing_from_the_plain_pairing={} mismatching_states={} s13={} holds={}",
            control.states,
            control.piece_panic_states[JudgingPiece::PoolChecker.position()],
            control.piece_panic_states[JudgingPiece::IgnoredRecovery.position()],
            control.piece_panic_states[JudgingPiece::RecordChecker.position()],
            control.other_piece_panic_states(),
            control.piece_panic_states[JudgingPiece::ConsultedRecovery.position()],
            control.consulted_recovery_panics_of_the_plain_pairing,
            control.crash_judge_columns_differing_from_the_plain_pairing,
            control.mismatching_states,
            control.stops_at_the_class_check,
            control.holds()
        ));
        lines.emit_result_line(&confirmed_loss_form_piece_panics_line(
            "piece_panic_control",
            &control.observations,
        ));
        if let Some(exit_code) =
            stop_the_confirmed_loss_form_and_write_the_verdict_when_a_panic_is_unattributed(
                lines,
                failed_checks,
                &control.observations,
            )
        {
            return exit_code;
        }
        if !control.holds() {
            // 失败 F9（修订 5 第 5.7 节）：甲那条管子接错了，主取样点不跑、不记成「O1 盲」。
            lines.emit_result_line(&format!(
                "name=verdict s1_failed_checks={failed_checks} s13=false q5_not_run=piece_panic_control_failed piece_panic_states=- unattributed_panic_states=0"
            ));
            lines.finish();
            return 3;
        }
    }
    let started = Instant::now();
    let prepared = shapes.main;
    let shape = shapes.shape;
    let drive_nanoseconds = shapes.drive_nanoseconds;
    let confirmed_root = prepared
        .writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .map(|index| {
            singlefs_verification::memory_pool::root_identity_of_write(&prepared.writes[index])
        });
    let in_flight_returned = prepared
        .calls
        .get(CONFIRMED_LOSS_IN_FLIGHT_CALL)
        .is_some_and(|call| call.outcome != CallOutcome::InFlightAtTheTruncation);
    lines.emit_result_line(&format!(
        "name=q5_shape sampling_point={sampling_point} removed_barriers={} root_writes_made_plain={} truncation={} operations_kept={} confirmed_root={} confirmed_call_returned_before_the_truncation={} in_flight_call_returned={in_flight_returned}",
        field_text(&format!("{:?}", shape.removed_barriers)),
        field_text(&format!("{:?}", shape.root_writes_made_plain)),
        if command_line.truncate_at_the_return_point { shape.reverse_truncation } else { shape.truncation },
        prepared.operations.len(),
        root_text(confirmed_root),
        prepared.calls[CONFIRMED_LOSS_CONFIRMED_CALL].outcome == CallOutcome::Succeeded
            && prepared.calls[CONFIRMED_LOSS_CONFIRMED_CALL].return_point <= prepared.operations.len(),
    ));
    let directory = command_line
        .output_directory
        .join("confirmed_loss_form")
        .join(StreamName::FixedScriptReuseAfterRaisingFloorThenUnmount.directory_name());
    let observation_tables = ObservationTables::of(&prepared);
    let (last_segment_writes, tearable, product) =
        last_segment_product(&prepared, &observation_tables.tables);
    let last_segment: Vec<usize> = prepared.segments.last().cloned().unwrap_or_default();
    let (export, totals, stops_at_the_class_check) = if command_line.truncate_at_the_return_point {
        // 方向相反的取样点照读法 7 全枚举（修订 2 的五个候选都不改它）；态数超上限不跑。
        if product > CONFIRMED_LOSS_STATE_LIMIT {
            lines.emit_result_line(&format!(
                "name=q5 sampling_point={sampling_point} not_run=too_many_states states={product} limit={CONFIRMED_LOSS_STATE_LIMIT}"
            ));
            lines.emit_result_line(&format!(
                "name=verdict s1_failed_checks={failed_checks} q5_not_run=too_many_states piece_panic_states=- unattributed_panic_states=0"
            ));
            lines.finish();
            return 3;
        }
        let mut export = export_prepared_stream(
            &prepared,
            &directory,
            false,
            RunSwitches::default(),
            StateSelection::WholeLastSegmentAndTheFullyPersistedState,
            started,
            drive_nanoseconds,
            shapes.piece_panic_injections.sampling_point_export.as_ref(),
        );
        let reference_filesystem_checks = reference_filesystem_checks_of_a_stream(&export);
        export.checks.extend(reference_filesystem_checks);
        let totals = report_stream(lines, &export);
        lines.emit_result_line(&confirmed_loss_form_piece_panics_line(
            "reverse",
            &export.observations,
        ));
        if let Some(exit_code) =
            stop_the_confirmed_loss_form_and_write_the_verdict_when_a_panic_is_unattributed(
                lines,
                failed_checks + totals.failed_checks,
                &export.observations,
            )
        {
            return exit_code;
        }
        let rows: Vec<ConfirmedLossTallyRow<'_>> = export
            .observations
            .iter()
            .map(|observation| ConfirmedLossTallyRow::of_a_judged_state(observation, &last_segment))
            .collect();
        report_the_confirmed_loss(
            lines,
            sampling_point,
            &rows,
            confirmed_root,
            (last_segment_writes, tearable, product),
            "",
        );
        (export, totals, false)
    } else {
        let context = observation_tables.context(&prepared);
        let digits = LastSegmentDigits::of(&prepared, &observation_tables.tables);
        let classification = classify_the_last_segment(
            &context,
            &digits,
            CONFIRMED_LOSS_IN_FLIGHT_CALL,
            CONFIRMED_LOSS_STATE_LIMIT,
        );
        if classification.stopped_at_the_state_limit {
            lines.emit_result_line(&format!(
                "name=q5_equivalence sampling_point={sampling_point} {} s13={}",
                equivalence_check_fields(&classification, &EquivalenceCheckCounts::default()),
                equivalence_stops_at_the_class_check(
                    &classification,
                    &EquivalenceCheckCounts::default()
                )
            ));
            lines.emit_result_line(&format!(
                "name=q5_partition sampling_point={sampling_point} {}",
                partition_fields(&classification, &EquivalenceCheckCounts::default())
            ));
            lines.emit_result_line(&format!(
                "name=verdict s1_failed_checks={failed_checks} s13=state_limit q5_not_run=too_many_states_after_refinement piece_panic_states=- unattributed_panic_states=0"
            ));
            lines.finish();
            return 3;
        }
        let judged = classification.judged_ordinals(&digits);
        let judged_states = judged.len();
        let mut export = export_prepared_stream(
            &prepared,
            &directory,
            false,
            RunSwitches::default(),
            StateSelection::ListedStatesKeepingTheConsultedReports(judged),
            started,
            drive_nanoseconds,
            shapes.piece_panic_injections.sampling_point_export.as_ref(),
        );
        let reference_filesystem_checks = reference_filesystem_checks_of_a_stream(&export);
        export.checks.extend(reference_filesystem_checks);
        let totals = report_stream(lines, &export);
        lines.emit_result_line(&confirmed_loss_form_piece_panics_line(
            "main",
            &export.observations,
        ));
        if let Some(exit_code) =
            stop_the_confirmed_loss_form_and_write_the_verdict_when_a_panic_is_unattributed(
                lines,
                failed_checks + totals.failed_checks,
                &export.observations,
            )
        {
            return exit_code;
        }
        if let Some(why) = &export.not_exported {
            // 第二道（第一道是修订 5 第 5.3 节的 [`shape_without_a_root_write`]）：这一形截出来的写流里没有可判的根槽写
            // （`export_prepared_stream` 记 `no_root_write_to_judge`）：类核对没走到，不是 S13；记 Q5 没跑、不进判定
            // （登记「主 agent 认定：25-3a 装置返工交回」第 6 条）。
            lines.emit_result_line(&format!(
                "name=q5 sampling_point={sampling_point} not_run=no_root_write not_exported_why={}",
                field_text(why)
            ));
            lines.emit_result_line(&format!(
                "name=verdict s1_failed_checks={} s4_digest_mismatches={} s3_o2_judgement_mismatches={} states=0 s13=false q5_not_run=no_root_write piece_panic_states={} unattributed_panic_states={}",
                failed_checks + totals.failed_checks,
                totals.rebuilt_digest_mismatches,
                totals.pool_checker_direct_mismatches,
                piece_panic_states_text(&totals.piece_panic_states),
                totals.unattributed_panic_states
            ));
            lines.finish();
            return 3;
        } else {
            let evaluation = evaluate_the_classes(
                &context,
                &digits,
                &classification,
                &export.observations,
                CONFIRMED_LOSS_IN_FLIGHT_CALL,
            );
            lines.emit_result_line(&format!(
                "name=q5_equivalence sampling_point={sampling_point} {} s13={}",
                equivalence_check_fields(&classification, &evaluation.checks),
                equivalence_stops_at_the_class_check(&classification, &evaluation.checks)
            ));
            lines.emit_result_line(&format!(
                "name=q5_partition sampling_point={sampling_point} {}",
                partition_fields(&classification, &evaluation.checks)
            ));
            for class in evaluation.classes.values() {
                let lost = confirmed_root
                    .and_then(|confirmed| {
                        lost_the_confirmed_version(class.consulted_root, confirmed)
                    })
                    .map_or_else(|| "none".to_string(), |lost| lost.to_string());
                lines.emit_result_line(&format!(
                    "name=q5_class sampling_point={sampling_point} keyed_landing={} in_flight_writes={} states={} consulted_root={} lost_the_confirmed_version={lost} crash_judge={} reference_filesystem={}",
                    class.keyed_landing,
                    class.key.in_flight_writes_landed.word(),
                    class.states,
                    root_text(class.consulted_root),
                    class.columns.crash_judge.word(),
                    class.columns.reference_filesystem.word()
                ));
            }
            let rows: Vec<ConfirmedLossTallyRow<'_>> = evaluation
                .classes
                .values()
                .map(ConfirmedLossTallyRow::of_a_class)
                .collect();
            report_the_confirmed_loss(
                lines,
                sampling_point,
                &rows,
                confirmed_root,
                (last_segment_writes, tearable, product),
                &format!(" derived_by=equivalence_classes judged_states={judged_states}"),
            );
            let stops = equivalence_stops_at_the_class_check(&classification, &evaluation.checks);
            (export, totals, stops)
        }
    };
    failed_checks += totals.failed_checks;
    lines.emit_result_line(&format!(
        "name=verdict s1_failed_checks={failed_checks} s4_digest_mismatches={} s3_o2_judgement_mismatches={} states={} s13={stops_at_the_class_check} piece_panic_states={} unattributed_panic_states={}",
        totals.rebuilt_digest_mismatches,
        totals.pool_checker_direct_mismatches,
        export.observations.len(),
        piece_panic_states_text(&totals.piece_panic_states),
        totals.unattributed_panic_states
    ));
    lines.finish();
    if failed_checks > 0
        || totals.rebuilt_digest_mismatches > 0
        || totals.pool_checker_direct_mismatches > 0
        || stops_at_the_class_check
    {
        3
    } else {
        0
    }
}

/// 这一态恢复落到的版本早于第 2 次调用写的那一版（按 (checkpoint_txg, 实例代号) 比，读法 7「已确认的丢了」）；恢复没落到任何版本的交 `None`。
fn lost_the_confirmed_version(
    consulted_root: Option<(InstanceGeneration, CheckpointTxg)>,
    confirmed_root: (InstanceGeneration, CheckpointTxg),
) -> Option<bool> {
    consulted_root.map(|(instance, checkpoint_txg)| {
        (checkpoint_txg, instance) < (confirmed_root.1, confirmed_root.0)
    })
}

/// Q5 计数的一行：全枚举时是一个判过的态，按等价类判时是一类。恢复落到的版本、两个崩溃子列，与这一行的态按「末段里落了几个写」
/// 排开的态数（键是落了几个写，值是态数；全枚举的一态只有一格、值是 1）。
struct ConfirmedLossTallyRow<'row> {
    /// 这一行罩几态（全枚举的一态是 1，等价类的一类是这一类的态数）。
    pub(super) states: u64,
    pub(super) consulted_root: Option<(InstanceGeneration, CheckpointTxg)>,
    pub(super) crash_judge: &'row CrashJudgeVerdict,
    pub(super) reference_filesystem: &'row ReferenceFileSystemVoteOfAState,
    pub(super) states_by_writes_landed_in_the_last_segment: BTreeMap<usize, u64>,
}

impl<'row> ConfirmedLossTallyRow<'row> {
    fn of_a_judged_state(observation: &'row StateObservation, last_segment: &[usize]) -> Self {
        let landed = last_segment
            .iter()
            .filter(|write_index| {
                observation
                    .landing_vector
                    .as_bytes()
                    .get(**write_index)
                    .is_some_and(|landing| *landing != b'0')
            })
            .count();
        Self {
            states: 1,
            consulted_root: observation.consulted_root,
            crash_judge: &observation.crash_judge,
            reference_filesystem: &observation.reference_filesystem,
            states_by_writes_landed_in_the_last_segment: BTreeMap::from([(landed, 1)]),
        }
    }

    fn of_a_class(class: &'row ConfirmedLossClass) -> Self {
        Self {
            states: class.states,
            consulted_root: class.consulted_root,
            crash_judge: &class.columns.crash_judge,
            reference_filesystem: &class.columns.reference_filesystem,
            states_by_writes_landed_in_the_last_segment: class
                .states_by_writes_landed_in_the_last_segment
                .clone(),
        }
    }
}

/// Q5 的计数行（第六节 Q5-有效、Q5-判、Q5-阴性；第七节 A10；第八节轨迹：按末段里落了几个写排开；修订 5 第 5.1 节第 7 条另加两列各自
/// `judge_panic` 的态数）。每一行按它罩的态数计
/// （全枚举的一态计 1，等价类的一类计这一类的态数）；`derived_by` 接在 `name=q5` 那一行末尾（按等价类判时写 `derived_by=…`）。
fn report_the_confirmed_loss(
    lines: &mut ResultLines,
    sampling_point: &str,
    rows: &[ConfirmedLossTallyRow<'_>],
    confirmed_root: Option<(InstanceGeneration, CheckpointTxg)>,
    (last_segment_writes, tearable, product): (usize, usize, u64),
    derived_by: &str,
) {
    let Some(confirmed_root) = confirmed_root else {
        lines.emit_result_line(&format!(
            "name=q5 sampling_point={sampling_point} confirmed_root=none{derived_by}"
        ));
        return;
    };
    let lost: Vec<&ConfirmedLossTallyRow<'_>> = rows
        .iter()
        .filter(|row| lost_the_confirmed_version(row.consulted_root, confirmed_root) == Some(true))
        .collect();
    let not_lost: Vec<&ConfirmedLossTallyRow<'_>> = rows
        .iter()
        .filter(|row| lost_the_confirmed_version(row.consulted_root, confirmed_root) == Some(false))
        .collect();
    fn states_of(
        selected: &[&ConfirmedLossTallyRow<'_>],
        predicate: &dyn Fn(&ConfirmedLossTallyRow<'_>) -> bool,
    ) -> u64 {
        selected
            .iter()
            .filter(|row| predicate(row))
            .map(|row| row.states)
            .sum()
    }
    fn reference_filesystem_red(row: &ConfirmedLossTallyRow<'_>) -> bool {
        row.reference_filesystem.word() == "red"
    }
    fn model_red(row: &ConfirmedLossTallyRow<'_>) -> bool {
        row.crash_judge.word() == "red"
    }
    fn both_green(row: &ConfirmedLossTallyRow<'_>) -> bool {
        row.reference_filesystem.word() == "green" && row.crash_judge.word() == "green"
    }
    fn every_row(_: &ConfirmedLossTallyRow<'_>) -> bool {
        true
    }
    let states: u64 = rows.iter().map(|row| row.states).sum();
    let without_a_root: u64 = rows
        .iter()
        .filter(|row| row.consulted_root.is_none())
        .map(|row| row.states)
        .sum();
    lines.emit_result_line(&format!(
        "name=q5 sampling_point={sampling_point} states={states} last_segment_writes={last_segment_writes} last_segment_tearable_writes={tearable} product_of_the_choices={product} a10_states_equal_the_product={} lost_the_confirmed_version={} reference_filesystem_red={} model_crash_judge_red={} both_green={} reference_filesystem_abstained={} reference_filesystem_judge_panic={} model_crash_judge_judge_panic={} not_lost_states={} not_lost_o1_red={} states_without_a_root={without_a_root}{derived_by}",
        states == product,
        states_of(&lost, &every_row),
        states_of(&lost, &reference_filesystem_red),
        states_of(&lost, &model_red),
        states_of(&lost, &both_green),
        states_of(&lost, &|row| row.reference_filesystem.word() == "abstained"),
        states_of(&lost, &|row| row.reference_filesystem.word() == "judge_panic"),
        states_of(&lost, &|row| row.crash_judge.word() == "judge_panic"),
        states_of(&not_lost, &every_row),
        states_of(&not_lost, &|row| reference_filesystem_red(row) || model_red(row)),
    ));
    let mut by_landed: BTreeMap<usize, (u64, u64, u64)> = BTreeMap::new();
    for row in &lost {
        for (landed, landed_states) in &row.states_by_writes_landed_in_the_last_segment {
            let entry = by_landed.entry(*landed).or_insert((0, 0, 0));
            entry.0 += landed_states;
            if reference_filesystem_red(row) {
                entry.1 += landed_states;
            }
            if model_red(row) {
                entry.2 += landed_states;
            }
        }
    }
    for (landed, (lost_states, reference_filesystem_red_states, model_red_states)) in by_landed {
        lines.emit_result_line(&format!(
            "name=q5_trajectory sampling_point={sampling_point} writes_landed_in_the_last_segment={landed} lost_states={lost_states} reference_filesystem_red={reference_filesystem_red_states} model_crash_judge_red={model_red_states}"
        ));
    }
}

#[cfg(test)]
mod tests;
