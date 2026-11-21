//! 第 2 次跑的家族：恰好读失败一次、抛弃那一步、实七两段复现、账本阳性对照与第 2 次跑的模式入口。

use super::*;

// ---------------------------------------------------------------------------
// 10.4 H1d：崩溃恢复把最新的根当成读不出、抛弃它、取号，这次可写挂载在写行根落盘之前断；故障撤掉、再挂载。
//      每一格报三样：① 恢复落到的根引用的单元有没有被覆盖；② 再挂载那一次 K0–K4；③ 读回的是不是最后一次确认的那一版。
// ---------------------------------------------------------------------------

#[derive(Default)]
struct HeldOnceFamilySummary {
    cases: u64,
    void_cases: u64,
    overwritten_cases: u64,
    newest_root_units_overwritten_cases: u64,
    remount_classes: BTreeMap<&'static str, u64>,
    read_back_after_remount: BTreeMap<String, u64>,
    read_back_before_remount: BTreeMap<String, u64>,
    trigger_positive_cases: u64,
    last_confirmed_and_not_overwritten: u64,
    /// 按断在哪一类（none / crash / write_fails / barrier_fails）分。
    by_cut: BTreeMap<&'static str, HeldOnceCutSummary>,
}

#[derive(Default)]
struct HeldOnceCutSummary {
    cases: u64,
    overwritten_cases: u64,
    remount_classes: BTreeMap<&'static str, u64>,
    read_back_after_remount: BTreeMap<String, u64>,
}

/// 一格 H1d：`post` 是那次挂载断掉之后盘上的样子（故障已撤）。`newest_root_units` 是被藏的那条（最新的）根在 `base` 上引用的单元。
/// ① 按「恢复落到的根」取：它在 `base` 上就是一条自证根时，比它引用的单元在 `base` 与 `post` 上的字节；它是这次挂载自己写出的根
/// （挂载做完了的那一格）时，它的单元就是这次写的，记「这次挂载里写的，没被覆盖」。被藏那条根的单元另报一栏。
#[allow(
    clippy::too_many_arguments,
    reason = "一格的全部坐标原样打进结果行，拆成结构体只多一层搬运"
)]
fn evaluate_held_once_cell(
    arm: &str,
    coordinates: &str,
    cut: &'static str,
    base: &MemoryPool,
    post: &MemoryPool,
    parameters: &MakeFilesystemParameters,
    last_confirmed: &Option<Vec<u8>>,
    newest_root_units: &BTreeSet<(u32, u64, u64)>,
    abandon_mount_outcome: &str,
    abandon_mount_trigger: Option<u64>,
    summary: &mut HeldOnceFamilySummary,
) {
    summary.cases += 1;
    let (landing, landing_record) = match landing_root_on(post) {
        Ok(found) => found,
        Err(error) => {
            emit_result(&format!(
                "name=r2_h1d_cell arm={arm} {coordinates} verdict=stop reason={error:?}"
            ));
            summary.void_cases += 1;
            return;
        }
    };
    let landing_in_base = independent_geometry(base)
        .map(|geometry| readable_roots_independent(base, &geometry).contains(&landing))
        .unwrap_or(false);
    let (overwritten, referenced_count) = if landing_in_base {
        match placements_referenced_by_root_on(base, &landing_record) {
            Ok(referenced) => (
                placements_overwritten_between(base, post, &referenced).to_string(),
                referenced.len().to_string(),
            ),
            Err(error) => (format!("unreadable_in_base:{error}"), "none".to_string()),
        }
    } else {
        ("written_in_this_mount".to_string(), "none".to_string())
    };
    let overwritten_now = overwritten.parse::<u64>().is_ok_and(|count| count > 0);
    if overwritten_now {
        summary.overwritten_cases += 1;
    }
    let newest_root_units_overwritten =
        placements_overwritten_between(base, post, newest_root_units);
    if newest_root_units_overwritten > 0 {
        summary.newest_root_units_overwritten_cases += 1;
    }
    let (read_back_before, read_back_before_root) = read_back_against(post, last_confirmed);
    let remount = run_instrumented_mount_writable(post, parameters, &[], None, None);
    let class = mount_outcome_class(&remount.result, remount.write_calls);
    let member = mount_error_member_or_none(&remount.result);
    let remount_trigger = remount
        .result
        .as_ref()
        .ok()
        .map(|mounted| mounted.output.abandoned_roots_unreadable);
    let remount_chosen = remount
        .result
        .as_ref()
        .ok()
        .map(|mounted| root_pair(&mounted.output.chosen_root));
    let (read_back_after, read_back_after_root) =
        read_back_against(&remount.pool_after, last_confirmed);
    if abandon_mount_trigger.is_some_and(|count| count > 0)
        || remount_trigger.is_some_and(|count| count > 0)
    {
        summary.trigger_positive_cases += 1;
    }
    if read_back_after == "last_confirmed" && !overwritten_now {
        summary.last_confirmed_and_not_overwritten += 1;
    }
    let by_cut = summary.by_cut.entry(cut).or_default();
    by_cut.cases += 1;
    if overwritten_now {
        by_cut.overwritten_cases += 1;
    }
    *by_cut.remount_classes.entry(class).or_insert(0) += 1;
    *by_cut
        .read_back_after_remount
        .entry(read_back_after.clone())
        .or_insert(0) += 1;
    *summary.remount_classes.entry(class).or_insert(0) += 1;
    *summary
        .read_back_after_remount
        .entry(read_back_after.clone())
        .or_insert(0) += 1;
    *summary
        .read_back_before_remount
        .entry(read_back_before.clone())
        .or_insert(0) += 1;
    emit_result(&format!(
        "name=r2_h1d_cell arm={arm} {coordinates} abandon_mount={abandon_mount_outcome} abandon_mount_abandoned_roots_unreadable={} landing_root={landing:?} landing_root_in_base={landing_in_base} referenced_placements={referenced_count} cell1_overwritten_placements={overwritten} cell1_overwritten={overwritten_now} newest_root_units={} newest_root_units_overwritten={newest_root_units_overwritten} cell2_remount_class={class} remount_error={member} remount_writes={} remount_chosen_root={} remount_abandoned_roots_unreadable={} read_back_before_remount={read_back_before} read_back_before_remount_root={} cell3_read_back_after_remount={read_back_after} read_back_after_remount_root={} cell3_is_last_confirmed={}",
        abandon_mount_trigger.map_or("none".to_string(), |count| count.to_string()),
        newest_root_units.len(),
        remount.write_calls,
        remount_chosen.map_or("none".to_string(), |root| format!("{root:?}")),
        remount_trigger.map_or("none".to_string(), |count| count.to_string()),
        read_back_before_root.map_or("none".to_string(), |root| format!("{root:?}")),
        read_back_after_root.map_or("none".to_string(), |root| format!("{root:?}")),
        read_back_after == "last_confirmed"
    ));
}

/// 故障有没有全拦到读（V2）：点了名而一次都没拦到的落点列出来。
pub(super) fn unintercepted_ranges(ranges: &[UnreadableRange], intercepted: &[u64]) -> Vec<String> {
    ranges
        .iter()
        .zip(intercepted)
        .filter(|(_, count)| **count == 0)
        .map(|(range, _)| range.label.clone())
        .collect()
}

/// H1d 的 n1：首个文件之后几次覆盖写。取 0 起（第 2 次跑修订：跑前修订没给 H1d 的参数；n1 = 0 时最新那条根就是首个文件，
/// 它前一条是写行之后的暖机根、树表 0 条，站在那条根看首个文件那次发布开的段是全空的——实七那一形的前提；n1 ≥ 1 时最新那条根的单元
/// 与前一条根的单元在同一个开放段里，写行那次发布开的是更高的新段，碰不到它们）。
const HELD_ONCE_OVERWRITES: [u64; 4] = [0, 1, 2, 3];

fn run_held_once_family(geometry: &Geometry, arm: &str) {
    let parameters = parameters_for(geometry);
    let mut summary = HeldOnceFamilySummary::default();
    for overwrites in HELD_ONCE_OVERWRITES {
        let mut history = match second_run_start(geometry, overwrites) {
            Ok(history) => history,
            Err(error) => {
                emit_result(&format!(
                    "name=r2_h1d_history arm={arm} n1={overwrites} verdict=stop reason={error:?}"
                ));
                continue;
            }
        };
        if let Err(error) =
            second_run_close(&mut history, &parameters, Closing::ProcessExit, arm, "h1d")
        {
            emit_result(&format!(
                "name=r2_h1d_history arm={arm} n1={overwrites} verdict=stop reason={error:?}"
            ));
            continue;
        }
        let base = history.pool.clone();
        let Ok(pool_geometry) = independent_geometry(&base) else {
            emit_result(&format!(
                "name=r2_h1d_history arm={arm} n1={overwrites} verdict=stop reason=\"S5：几何解不出\""
            ));
            continue;
        };
        let root_slots = root_slot_device_offsets(&pool_geometry);
        let last_confirmed = history.tip_content();
        let newest_root_units = match landing_root_on(&base)
            .and_then(|(_, record)| placements_referenced_by_root_on(&base, &record))
        {
            Ok(units) => units,
            Err(error) => {
                emit_result(&format!(
                    "name=r2_h1d_history arm={arm} n1={overwrites} verdict=stop reason={error:?}"
                ));
                continue;
            }
        };
        for form in [UnreadableForm::ReadFails, UnreadableForm::ReadsZeros] {
            for replay_break in [
                ReplayBreak::EveryRecord,
                ReplayBreak::FirstNamedUnit,
                ReplayBreak::FirstCopyOfTheFirstNamedUnit,
            ] {
                let head = format!(
                    "n1={overwrites} tip={:?} form={} break={}",
                    history.tip(),
                    form.name(),
                    replay_break.name()
                );
                let ranges = match unreadable_ranges_of_the_abandonment(
                    &history,
                    &pool_geometry,
                    1,
                    form,
                    replay_break,
                ) {
                    Ok(Some(ranges)) => ranges,
                    Ok(None) => {
                        emit_result(&format!(
                            "name=r2_h1d_reference arm={arm} {head} verdict=not_applicable"
                        ));
                        continue;
                    }
                    Err(error) => {
                        emit_result(&format!(
                            "name=r2_h1d_reference arm={arm} {head} verdict=stop reason={error:?}"
                        ));
                        continue;
                    }
                };
                let reference =
                    run_instrumented_mount_writable(&base, &parameters, &ranges, None, None);
                let missed = unintercepted_ranges(&ranges, &reference.intercepted_reads);
                let first_root_write = index_of_the_first_root_slot_write(
                    &reference.applied,
                    &root_slots,
                    u64::from(pool_geometry.physical_block_size),
                );
                let reference_class = mount_outcome_class(&reference.result, reference.write_calls);
                let reference_first_txg = reference
                    .result
                    .as_ref()
                    .ok()
                    .map(|mounted| mounted.output.row_publish.root().checkpoint_txg.0);
                let rule_of_today = first_txg_by_the_rule_of_today(&base, &pool_geometry, &ranges);
                emit_result(&format!(
                    "name=r2_h1d_reference arm={arm} {head} faults={} unintercepted={missed:?} class={reference_class} error={} writes={} chosen_root={} row_publish_txg={} first_txg_by_the_rule_of_today={rule_of_today} abandoned_roots_unreadable={} writes_before_the_row_root={}",
                    ranges.len(),
                    mount_error_member_or_none(&reference.result),
                    reference.write_calls,
                    reference
                        .result
                        .as_ref()
                        .ok()
                        .map_or("none".to_string(), |mounted| format!("{:?}", root_pair(&mounted.output.chosen_root))),
                    reference_first_txg.map_or("none".to_string(), |txg| txg.to_string()),
                    reference
                        .result
                        .as_ref()
                        .ok()
                        .map_or("none".to_string(), |mounted| mounted.output.abandoned_roots_unreadable.to_string()),
                    first_root_write.map_or("none".to_string(), |index| index.to_string()),
                ));
                if !missed.is_empty() {
                    summary.void_cases += 1;
                    continue;
                }
                let reference_trigger = reference
                    .result
                    .as_ref()
                    .ok()
                    .map(|mounted| mounted.output.abandoned_roots_unreadable);
                evaluate_held_once_cell(
                    arm,
                    &format!("{head} cut=none point=none"),
                    "none",
                    &base,
                    &reference.pool_after,
                    &parameters,
                    &last_confirmed,
                    &newest_root_units,
                    reference_class,
                    reference_trigger,
                    &mut summary,
                );
                let Some(first_root_write) = first_root_write else {
                    continue;
                };
                for write_count in 0..=first_root_write {
                    let post = image_with_the_first_writes(&base, &reference.applied, write_count);
                    evaluate_held_once_cell(
                        arm,
                        &format!("{head} cut=crash point={write_count}"),
                        "crash",
                        &base,
                        &post,
                        &parameters,
                        &last_confirmed,
                        &newest_root_units,
                        "crashed",
                        None,
                        &mut summary,
                    );
                }
                for failing_write in 0..=first_root_write {
                    let failed = run_instrumented_mount_writable(
                        &base,
                        &parameters,
                        &ranges,
                        Some(u64::try_from(failing_write).expect("写序号")),
                        None,
                    );
                    let failed_class = mount_outcome_class(&failed.result, failed.write_calls);
                    let outcome = format!(
                        "{failed_class}:{}:fired={}",
                        mount_error_member_or_none(&failed.result),
                        failed.injected_failure_fired
                    );
                    let trigger = failed
                        .result
                        .as_ref()
                        .ok()
                        .map(|mounted| mounted.output.abandoned_roots_unreadable);
                    evaluate_held_once_cell(
                        arm,
                        &format!("{head} cut=write_fails point={failing_write}"),
                        "write_fails",
                        &base,
                        &failed.pool_after,
                        &parameters,
                        &last_confirmed,
                        &newest_root_units,
                        &outcome,
                        trigger,
                        &mut summary,
                    );
                }
                let barriers = barriers_before_write(&reference.applied, first_root_write + 1);
                for failing_barrier in 0..barriers {
                    let failed = run_instrumented_mount_writable(
                        &base,
                        &parameters,
                        &ranges,
                        None,
                        Some(failing_barrier),
                    );
                    let failed_class = mount_outcome_class(&failed.result, failed.write_calls);
                    let outcome = format!(
                        "{failed_class}:{}:fired={}",
                        mount_error_member_or_none(&failed.result),
                        failed.injected_failure_fired
                    );
                    let trigger = failed
                        .result
                        .as_ref()
                        .ok()
                        .map(|mounted| mounted.output.abandoned_roots_unreadable);
                    evaluate_held_once_cell(
                        arm,
                        &format!("{head} cut=barrier_fails point={failing_barrier}"),
                        "barrier_fails",
                        &base,
                        &failed.pool_after,
                        &parameters,
                        &last_confirmed,
                        &newest_root_units,
                        &outcome,
                        trigger,
                        &mut summary,
                    );
                }
            }
        }
    }
    emit_result(&format!(
        "name=r2_h1d_summary arm={arm} geometry={} cases={} void_cases={} cell1_overwritten_cases={} newest_root_units_overwritten_cases={} cell2_remount_classes={:?} cell3_read_back_after_remount={:?} read_back_before_remount={:?} trigger_positive_cases={} last_confirmed_and_not_overwritten={}",
        geometry.label,
        summary.cases,
        summary.void_cases,
        summary.overwritten_cases,
        summary.newest_root_units_overwritten_cases,
        summary.remount_classes,
        summary.read_back_after_remount,
        summary.read_back_before_remount,
        summary.trigger_positive_cases,
        summary.last_confirmed_and_not_overwritten
    ));
    // 岔路单第 1 行 (b)、(c) 两个指称：这两条臂不在 `crates/` 里实现（跑前登记第三节末句），它们与今天的差别只在
    // 「取号之前或别处的计数 > 0」的那几格（(c) 另在盘上记一笔，落点未定）；计数全 0 时它们的三格就是今天那一臂的三格。
    for (cut, by_cut) in &summary.by_cut {
        emit_result(&format!(
            "name=r2_h1d_summary_by_cut arm={arm} cut={cut} cases={} cell1_overwritten_cases={} cell2_remount_classes={:?} cell3_read_back_after_remount={:?}",
            by_cut.cases,
            by_cut.overwritten_cases,
            by_cut.remount_classes,
            by_cut.read_back_after_remount
        ));
    }
    if arm != "today" {
        return;
    }
    for (fork_one_arm, referent) in [
        ("(b)原形", "tree_table"),
        ("(b)原形", "allocation_record_tree"),
        ("(b)含链", "tree_table"),
        ("(b)含链", "allocation_record_tree"),
        ("(c)", "tree_table"),
        ("(c)", "allocation_record_tree"),
    ] {
        emit_result(&format!(
            "name=r2_h1d_fork_one_derived arm={arm} fork_one_arm={fork_one_arm} referent={referent} cases_where_it_differs_from_this_copy={} cells=same_as_this_copy_except_those",
            summary.trigger_positive_cases
        ));
    }
}

// ---------------------------------------------------------------------------
// 10.5 Q1-0：H1c 的抛弃步 A(k, b) 在今天的代码上造出几条被抛弃的根（前提 1 在装置上的读数）。故障一律读错（读法写死表「故障」）。
// ---------------------------------------------------------------------------

fn run_abandonment_step_family(geometry: &Geometry, arm: &str) {
    let parameters = parameters_for(geometry);
    let mut attempted = 0u64;
    let mut not_applicable = 0u64;
    let mut void_attempts = 0u64;
    let mut mounted_count = 0u64;
    let mut producing = 0u64;
    let mut abandoned_total = 0u64;
    let mut fewest_faults: Option<usize> = None;
    let mut first_txg_rule_mismatches = 0u64;
    let mut below_floor_rows = 0u64;
    for overwrites in [1u64, 2, 3] {
        for closing in [Closing::ProcessExit, Closing::NormalUnmount] {
            let mut history = match second_run_start(geometry, overwrites) {
                Ok(history) => history,
                Err(error) => {
                    emit_result(&format!(
                        "name=r2_q1_0_history arm={arm} n1={overwrites} c1={} verdict=stop reason={error:?}",
                        closing.name()
                    ));
                    continue;
                }
            };
            if let Err(error) = second_run_close(
                &mut history,
                &parameters,
                closing,
                arm,
                &format!("q1-0 n1={overwrites}"),
            ) {
                emit_result(&format!(
                    "name=r2_q1_0_history arm={arm} n1={overwrites} c1={} verdict=stop reason={error:?}",
                    closing.name()
                ));
                continue;
            }
            let Ok(pool_geometry) = independent_geometry(&history.pool) else {
                continue;
            };
            let slot_spacing = u64::from(parameters.geometry.fixed_structure_slot_spacing);
            let floor_before =
                highest_floor_in_the_system_configurations(&history.pool, slot_spacing);
            let ring_slot_count = 3 * geometry.slots_per_region;
            let timeline_length = history.timeline.len();
            emit_result(&format!(
                "name=r2_q1_0_history arm={arm} n1={overwrites} c1={} timeline={:?} timeline_roots={timeline_length} root_ring_slots={ring_slot_count} highest_floor_in_the_system_configurations={floor_before}",
                closing.name(),
                history.timeline
            ));
            for hidden in 1..timeline_length {
                for replay_break in [
                    ReplayBreak::EveryRecord,
                    ReplayBreak::FirstNamedUnit,
                    ReplayBreak::FirstCopyOfTheFirstNamedUnit,
                ] {
                    attempted += 1;
                    let head = format!(
                        "n1={overwrites} c1={} k={hidden} b={}",
                        closing.name(),
                        replay_break.name()
                    );
                    let ranges = match unreadable_ranges_of_the_abandonment(
                        &history,
                        &pool_geometry,
                        hidden,
                        UnreadableForm::ReadFails,
                        replay_break,
                    ) {
                        Ok(Some(ranges)) => ranges,
                        Ok(None) => {
                            not_applicable += 1;
                            emit_result(&format!(
                                "name=r2_q1_0 arm={arm} {head} verdict=not_applicable"
                            ));
                            continue;
                        }
                        Err(error) => {
                            emit_result(&format!(
                                "name=r2_q1_0 arm={arm} {head} verdict=stop reason={error:?}"
                            ));
                            continue;
                        }
                    };
                    let run = run_instrumented_mount_writable(
                        &history.pool,
                        &parameters,
                        &ranges,
                        None,
                        None,
                    );
                    let missed = unintercepted_ranges(&ranges, &run.intercepted_reads);
                    let class = mount_outcome_class(&run.result, run.write_calls);
                    if !missed.is_empty() {
                        void_attempts += 1;
                    }
                    let rule_of_today =
                        first_txg_by_the_rule_of_today(&history.pool, &pool_geometry, &ranges);
                    let (chosen, row_txg, abandoned_text, abandoned_count) = match &run.result {
                        Ok(mounted) => {
                            mounted_count += 1;
                            let row_txg = mounted.output.row_publish.root().checkpoint_txg.0;
                            if row_txg != rule_of_today {
                                first_txg_rule_mismatches += 1;
                            }
                            if row_txg <= floor_before {
                                below_floor_rows += 1;
                            }
                            match abandoned_roots_on(&run.pool_after) {
                                Ok(abandoned) => {
                                    let count = abandoned.len() as u64;
                                    if count > 0 && missed.is_empty() {
                                        producing += 1;
                                        abandoned_total += count;
                                        fewest_faults =
                                            Some(fewest_faults.map_or(ranges.len(), |fewest| {
                                                fewest.min(ranges.len())
                                            }));
                                    }
                                    (
                                        format!("{:?}", root_pair(&mounted.output.chosen_root)),
                                        row_txg.to_string(),
                                        format!("{abandoned:?}"),
                                        count,
                                    )
                                }
                                Err(error) => (
                                    format!("{:?}", root_pair(&mounted.output.chosen_root)),
                                    row_txg.to_string(),
                                    format!("stop:{error}"),
                                    0,
                                ),
                            }
                        }
                        Err(_) => (
                            "none".to_string(),
                            "none".to_string(),
                            "none".to_string(),
                            0,
                        ),
                    };
                    emit_result(&format!(
                        "name=r2_q1_0 arm={arm} {head} faults={} unintercepted={missed:?} class={class} error={} writes={} chosen_root={chosen} expected_chosen_root={:?} row_publish_txg={row_txg} first_txg_by_the_rule_of_today={rule_of_today} highest_floor_in_the_system_configurations={floor_before} abandoned_count={abandoned_count} abandoned={abandoned_text:?}",
                        ranges.len(),
                        mount_error_member_or_none(&run.result),
                        run.write_calls,
                        history.timeline[timeline_length - 1 - hidden],
                    ));
                }
            }
        }
    }
    emit_result(&format!(
        "name=r2_q1_0_summary arm={arm} geometry={} attempted={attempted} not_applicable={not_applicable} void_attempts={void_attempts} mounted={mounted_count} producing_at_least_one_abandoned_root={producing} abandoned_roots_total={abandoned_total} fewest_faults_that_produced_one={} first_txg_rule_of_today_mismatches={first_txg_rule_mismatches} row_publish_txg_at_or_below_the_floor={below_floor_rows}",
        geometry.label,
        fewest_faults.map_or("none".to_string(), |fewest| fewest.to_string())
    ));
}

// ---------------------------------------------------------------------------
// 10.6 实七报告「交主 agent 的」第 1 条点名的两段历史（H1d 的取样要罩住它们，跑前修订原话）：照 `crates/singlefs-verification`
//      自己的注入器原样重跑，报每个新发现的签名与重开走到哪、有没有「映射落点读不出」。
// ---------------------------------------------------------------------------

fn run_seventh_batch_reproductions(arm: &str) {
    use singlefs_verification::history::{
        generate_history_with_weights, GenerationWeights, HistoryDeviceWidth, HistoryExecution,
        HistorySeed, PerStepChecker,
    };
    let fault_seed =
        SEED_BASE_OF_THIS_TEST_CYCLE_LOCAL + SEVENTH_BATCH_FAULT_REPRODUCTION_SEED_OFFSET;
    let fault_history = generate_history_with_weights(
        HistorySeed(fault_seed),
        SEVENTH_BATCH_FAULT_REPRODUCTION_OPERATIONS,
        &GenerationWeights::BROAD,
    );
    // 池级 checker 那一遍在给定状态引擎两段判，照判红就停那一档的口径并进 harness 的判定
    let fault_execution = HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: HistoryDeviceWidth::FourGibibytes,
        space_admission: singlefs_core::admission::SpaceAdmission::JudgedByTheFormula,
    };
    let fault_engine = given_states_engine_of_this_experiment("e158_fault_reproduction");
    let mut images_judged = 0usize;
    let fault_injection = singlefs_verification::fault_injection::inject_faults_into_history_merging_the_pool_checker_verdicts_of_the_given_states_engine(
        &fault_history,
        fault_execution,
        SEVENTH_BATCH_FAULT_REPRODUCTION_FAULTS,
        &mut |images| {
            images_judged += 1;
            pool_checker_verdicts_of_the_images_after_a_fault_through_the_engine(
                &fault_engine,
                &format!(
                    "seventh-batch-fault-reproduction/seed={fault_seed}/hand-out={images_judged}"
                ),
                fault_execution,
                images,
            )
        },
    );
    for drawn in &fault_injection.drawn_faults {
        emit_result(&format!(
            "name=r2_seventh_batch_fault_drawn arm={arm} seed={fault_seed} drawn={:?}",
            drawn.render()
        ));
    }
    let mut mapping_unreadable = 0u64;
    for finding in &fault_injection.new_findings {
        let violations: Vec<&str> = finding
            .observation
            .violations
            .iter()
            .map(|(invariant, _)| *invariant)
            .collect();
        let names_mapping = finding.reopen.contains("MappingStillUnreadable");
        if names_mapping {
            mapping_unreadable += 1;
        }
        emit_result(&format!(
            "name=r2_seventh_batch_fault_finding arm={arm} seed={fault_seed} drawn={:?} signature={:?} reopen={:?} violations={violations:?} reopen_names_mapping_still_unreadable={names_mapping}",
            finding.drawn.render(),
            finding.signature,
            finding.reopen
        ));
    }
    emit_result(&format!(
        "name=r2_seventh_batch_fault_summary arm={arm} seed={fault_seed} drawn={} new_findings={} findings_reopening_to_mapping_still_unreadable={mapping_unreadable} known_red_hits={}",
        fault_injection.drawn_faults.len(),
        fault_injection.new_findings.len(),
        fault_injection.known_red_hits.len()
    ));
    let crash_seed =
        SEED_BASE_OF_THIS_TEST_CYCLE_LOCAL + SEVENTH_BATCH_CRASH_REPRODUCTION_SEED_OFFSET;
    let crash_history = generate_history_with_weights(
        HistorySeed(crash_seed),
        SEVENTH_BATCH_CRASH_REPRODUCTION_OPERATIONS,
        &GenerationWeights::BROAD,
    );
    let crash_injection = singlefs_verification::crash_injection::inject_crashes_into_history(
        &crash_history,
        HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width: HistoryDeviceWidth::FourGibibytes,
            space_admission: singlefs_core::admission::SpaceAdmission::JudgedByTheFormula,
        },
        singlefs_verification::crash_injection::CrashPointDraw::Sampled {
            crash_points_per_history: SEVENTH_BATCH_CRASH_REPRODUCTION_CRASH_POINTS,
        },
        None,
        &given_states_engine_of_this_experiment("e158_crash_reproduction"),
    );
    for crash_point in &crash_injection.crash_points {
        emit_result(&format!(
            "name=r2_seventh_batch_crash_point arm={arm} seed={crash_seed} point={:?} step={:?} operation={:?}",
            crash_point.render(),
            crash_point.step,
            crash_point.operation_kind
        ));
    }
    let mut crash_mapping_unreadable = 0u64;
    for finding in &crash_injection.new_findings {
        let violations: Vec<&str> = finding
            .observation
            .violations
            .iter()
            .map(|(invariant, _)| *invariant)
            .collect();
        let names_mapping = finding.read_back.contains("MappingStillUnreadable");
        if names_mapping {
            crash_mapping_unreadable += 1;
        }
        emit_result(&format!(
            "name=r2_seventh_batch_crash_finding arm={arm} seed={crash_seed} point={:?} operation={:?} signature={:?} read_back={:?} violations={violations:?} read_back_names_mapping_still_unreadable={names_mapping}",
            finding.crash_point.render(),
            finding.crash_point.operation_kind,
            finding.signature,
            finding.read_back
        ));
    }
    emit_result(&format!(
        "name=r2_seventh_batch_crash_summary arm={arm} seed={crash_seed} crash_points={} new_findings={} findings_reading_back_mapping_still_unreadable={crash_mapping_unreadable} known_red_hits={}",
        crash_injection.crash_points.len(),
        crash_injection.new_findings.len(),
        crash_injection.known_red_hits.len()
    ));
}

// ---------------------------------------------------------------------------
// 10.7 PC1-a / PC1-b / PC1-c（跑前登记 5.2 阳性对照；第一段只跑今天那一臂与 (c)、A1 副本上 PC1-a 的 (a) 那一句，
//      (b) 两个指称的那几句归第二段）。被抛弃根取自 H1c 的抛弃步（固定次序 n1、c1、k、b，m = 0、c2 = C），op1 = O1、瞬时。
// ---------------------------------------------------------------------------

/// H1c 抛弃步造出的一个节点：抛弃步交 `Ok`、点了名的落点全拦到（V2）、按新实例表判出至少一条被抛弃的根。
struct AbandonmentStepNode {
    label: String,
    pool: MemoryPool,
    abandoned: BTreeSet<TimelineRoot>,
}

/// H1c 抛弃步造出的节点，按固定次序（n1、c1、k、b）；故障一律读错。
fn abandonment_step_node_list(geometry: &Geometry) -> Vec<AbandonmentStepNode> {
    let parameters = parameters_for(geometry);
    let mut nodes = Vec::new();
    for overwrites in [1u64, 2, 3] {
        for closing in [Closing::ProcessExit, Closing::NormalUnmount] {
            let Ok(mut history) = second_run_start(geometry, overwrites) else {
                continue;
            };
            if second_run_close(&mut history, &parameters, closing, "node-list", "node-list")
                .is_err()
            {
                continue;
            }
            let Ok(pool_geometry) = independent_geometry(&history.pool) else {
                continue;
            };
            for hidden in 1..history.timeline.len() {
                for replay_break in [
                    ReplayBreak::EveryRecord,
                    ReplayBreak::FirstNamedUnit,
                    ReplayBreak::FirstCopyOfTheFirstNamedUnit,
                ] {
                    let Ok(Some(ranges)) = unreadable_ranges_of_the_abandonment(
                        &history,
                        &pool_geometry,
                        hidden,
                        UnreadableForm::ReadFails,
                        replay_break,
                    ) else {
                        continue;
                    };
                    let run = run_instrumented_mount_writable(
                        &history.pool,
                        &parameters,
                        &ranges,
                        None,
                        None,
                    );
                    if run.result.is_err()
                        || !unintercepted_ranges(&ranges, &run.intercepted_reads).is_empty()
                    {
                        continue;
                    }
                    let Ok(abandoned) = abandoned_roots_on(&run.pool_after) else {
                        continue;
                    };
                    if abandoned.is_empty() {
                        continue;
                    }
                    nodes.push(AbandonmentStepNode {
                        label: format!(
                            "n1={overwrites} c1={} k={hidden} b={}",
                            closing.name(),
                            replay_break.name()
                        ),
                        pool: run.pool_after,
                        abandoned,
                    });
                }
            }
        }
    }
    nodes
}

fn ranges_of_both_copies(locations: &[LocationEntry; 2], label: &str) -> Vec<UnreadableRange> {
    locations
        .iter()
        .map(|location| UnreadableRange {
            device: location.device,
            offset: location.slot.0 * SLOT_BYTES_LOCAL,
            length: SLOT_BYTES_LOCAL,
            form: UnreadableForm::ReadFails,
            label: format!(
                "{label}(device={},slot={})",
                location.device.0, location.slot.0
            ),
        })
        .collect()
}

/// 一条根某个单元的位置键（(盘, 槽)）；取不到时 None。
type UnitKeysOfARoot<'reader> = dyn Fn(&RootRecord) -> Option<BTreeSet<(u32, u64)>> + 'reader;

pub(super) fn location_keys(locations: &[LocationEntry; 2]) -> BTreeSet<(u32, u64)> {
    locations.iter().map(location_key).collect()
}

/// 一次 op1（O1、瞬时）的结局行，外加这一臂按跑前登记 5.2 该出的结局判没判对：今天那一臂与 (c) 要 `Ok` 且计数 = 装置数的；
/// A1 副本（PC1-a、PC1-c 两句）按那一句写的；别的臂没有登记结局，只报。
#[allow(clippy::too_many_arguments, reason = "一行结果的全部栏目原样打出")]
fn emit_positive_control(
    arm: &str,
    control: &str,
    node_label: &str,
    abandoned_root: TimelineRoot,
    ranges: &[UnreadableRange],
    run: &InstrumentedMount,
    expected_count: u64,
    extra: &str,
) -> bool {
    let missed = unintercepted_ranges(ranges, &run.intercepted_reads);
    let class = mount_outcome_class(&run.result, run.write_calls);
    let member = mount_error_member_or_none(&run.result);
    let count = run
        .result
        .as_ref()
        .ok()
        .map(|mounted| mounted.output.abandoned_roots_unreadable);
    let isolated: Option<u64> = run.result.as_ref().ok().map(|mounted| {
        mounted
            .output
            .isolated_slots_per_device
            .iter()
            .map(|(_, slots)| *slots)
            .sum()
    });
    let (expectation, passed) = match (arm, control) {
        ("A1", "PC1-a") => (
            "AbandonedRootLedgerUnreadable_and_zero_writes",
            member == "AbandonedRootLedgerUnreadable" && run.write_calls == 0,
        ),
        ("A1", "PC1-c") => ("ok", run.result.is_ok()),
        ("today", "PC1-c") => (
            "ok_count_zero_isolated_zero",
            count == Some(0) && isolated == Some(0),
        ),
        ("today", _) => (
            "ok_count_equals_the_device_count",
            count == Some(expected_count),
        ),
        _ => ("not_registered_for_this_arm", true),
    };
    let passed = passed && missed.is_empty();
    emit_result(&format!(
        "name=r2_positive_control arm={arm} control={control} node={node_label:?} abandoned_root={abandoned_root:?} faults={} unintercepted={missed:?} class={class} error={member} writes={} abandoned_roots_unreadable={} expected_count_by_the_device={expected_count} isolated_slots={} {extra} expectation={expectation} verdict={}",
        ranges.len(),
        run.write_calls,
        count.map_or("none".to_string(), |value| value.to_string()),
        isolated.map_or("none".to_string(), |value| value.to_string()),
        if passed { "pass" } else { "fail" }
    ));
    passed
}

fn run_ledger_positive_controls(geometry: &Geometry, arm: &str) {
    let parameters = parameters_for(geometry);
    let nodes = abandonment_step_node_list(geometry);
    emit_result(&format!(
        "name=r2_positive_control_nodes arm={arm} nodes={}",
        nodes.len()
    ));
    let mut ledger_unit_controls_found = false;
    'search: for node in &nodes {
        let Ok(pool_geometry) = independent_geometry(&node.pool) else {
            continue;
        };
        let readable = readable_roots_independent(&node.pool, &pool_geometry);
        let plain = devices_from_pool(&node.pool, IMAGE_BYTES);
        for &abandoned_root in node.abandoned.iter().filter(|root| readable.contains(root)) {
            let Some(record) = root_record_of(&node.pool, &pool_geometry, abandoned_root) else {
                continue;
            };
            let Ok(allocation_locations) = locate_allocation_record_tree(&plain, &record) else {
                continue;
            };
            let allocation_keys = location_keys(&allocation_locations);
            if is_unit_shared(
                &plain,
                &node.pool,
                &pool_geometry,
                abandoned_root,
                &allocation_keys,
            ) {
                continue;
            }
            ledger_unit_controls_found = true;
            let through = |keys_of: &UnitKeysOfARoot, keys: &BTreeSet<(u32, u64)>| {
                node.abandoned
                    .iter()
                    .filter(|root| readable.contains(root))
                    .filter_map(|root| root_record_of(&node.pool, &pool_geometry, *root))
                    .filter(|other| {
                        keys_of(other).is_some_and(|other_keys| !other_keys.is_disjoint(keys))
                    })
                    .count() as u64
            };
            let allocation_keys_of = |other: &RootRecord| {
                locate_allocation_record_tree(&plain, other)
                    .ok()
                    .map(|locations| location_keys(&locations))
            };
            let tree_table_keys_of =
                |other: &RootRecord| Some(location_keys(&other.tree_table.locations));
            let ranges = ranges_of_both_copies(&allocation_locations, "L");
            let run = run_instrumented_mount_writable(&node.pool, &parameters, &ranges, None, None);
            emit_positive_control(
                arm,
                "PC1-a",
                &node.label,
                abandoned_root,
                &ranges,
                &run,
                through(&allocation_keys_of, &allocation_keys),
                "unit=allocation_record_tree_root shared=false",
            );
            let tree_table_keys = location_keys(&record.tree_table.locations);
            let tree_table_ranges = ranges_of_both_copies(&record.tree_table.locations, "T");
            let tree_table_run = run_instrumented_mount_writable(
                &node.pool,
                &parameters,
                &tree_table_ranges,
                None,
                None,
            );
            emit_positive_control(
                arm,
                "PC1-b",
                &node.label,
                abandoned_root,
                &tree_table_ranges,
                &tree_table_run,
                through(&tree_table_keys_of, &tree_table_keys),
                &format!(
                    "unit=tree_table shared={}",
                    is_unit_shared(
                        &plain,
                        &node.pool,
                        &pool_geometry,
                        abandoned_root,
                        &tree_table_keys
                    )
                ),
            );
            break 'search;
        }
    }
    if !ledger_unit_controls_found {
        emit_result(&format!(
            "name=r2_positive_control arm={arm} control=PC1-a verdict=not_constructible reason=\"H1c 里找不到被抛弃根的分配记录树节点不共享的历史（V1）\""
        ));
    }
    let mut root_slot_control_found = false;
    for node in &nodes {
        let Ok(pool_geometry) = independent_geometry(&node.pool) else {
            continue;
        };
        let readable = readable_roots_independent(&node.pool, &pool_geometry);
        let abandoned_readable: Vec<TimelineRoot> = node
            .abandoned
            .iter()
            .copied()
            .filter(|root| readable.contains(root))
            .collect();
        let [abandoned_root] = abandoned_readable.as_slice() else {
            continue;
        };
        let Some(record) = root_record_of(&node.pool, &pool_geometry, *abandoned_root) else {
            continue;
        };
        let Ok(mut exclusive) = placements_referenced_by_root_on(&node.pool, &record) else {
            continue;
        };
        for other in readable.iter().filter(|root| *root != abandoned_root) {
            if let Some(other_record) = root_record_of(&node.pool, &pool_geometry, *other) {
                if let Ok(referenced) = placements_referenced_by_root_on(&node.pool, &other_record)
                {
                    exclusive.retain(|(device, slot, _)| {
                        !referenced.iter().any(|(other_device, other_slot, _)| {
                            other_device == device && other_slot == slot
                        })
                    });
                }
            }
        }
        if exclusive.is_empty() {
            continue;
        }
        let Some((device, offset)) = root_slot_of(&node.pool, &pool_geometry, *abandoned_root)
        else {
            continue;
        };
        root_slot_control_found = true;
        let twin = run_instrumented_mount_writable(&node.pool, &parameters, &[], None, None);
        let twin_isolated: Option<u64> = twin.result.as_ref().ok().map(|mounted| {
            mounted
                .output
                .isolated_slots_per_device
                .iter()
                .map(|(_, slots)| *slots)
                .sum()
        });
        let ranges = vec![UnreadableRange {
            device,
            offset,
            length: u64::from(pool_geometry.physical_block_size),
            form: UnreadableForm::ReadFails,
            label: format!("R{abandoned_root:?}"),
        }];
        let run = run_instrumented_mount_writable(&node.pool, &parameters, &ranges, None, None);
        emit_positive_control(
            arm,
            "PC1-c",
            &node.label,
            *abandoned_root,
            &ranges,
            &run,
            0,
            &format!(
                "exclusive_placements={} isolated_slots_without_the_fault={}",
                exclusive.len(),
                twin_isolated.map_or("none".to_string(), |value| value.to_string())
            ),
        );
        break;
    }
    if !root_slot_control_found {
        emit_result(&format!(
            "name=r2_positive_control arm={arm} control=PC1-c verdict=not_constructible reason=\"H1c 里找不到只有一条被抛弃根且它独占的槽 > 0 的历史（V1）\""
        ));
    }
}

/// 第 2 次跑的模式：开跑检查不过就停（V3 / V4 / S3），过了跑点名的那一族。交回 true = 这个模式归第 2 次跑。
pub(super) fn run_second_run_mode(mode: &str) -> bool {
    let known = [
        "r2-all",
        "r2-constants",
        "r2-h1d",
        "r2-q1-0",
        "r2-pc1",
        "r2-seventh-batch",
    ];
    if !known.contains(&mode) {
        return false;
    }
    let arm = second_run_arm_label();
    if !run_second_run_constants_and_anchors(&arm) {
        emit_result(&format!(
            "name=stop arm={arm} reason=V3 detail=\"第 2 次跑的常量回比、第七节锚点或偏移解码有不过的，整轮不开跑\""
        ));
        emit_result(&format!(
            "name=done emitted={}",
            emitted_result_line_count() + 1
        ));
        return true;
    }
    match mode {
        "r2-all" => {
            run_held_once_family(&GEOMETRY_PRIMARY, &arm);
            run_abandonment_step_family(&GEOMETRY_PRIMARY, &arm);
            run_ledger_positive_controls(&GEOMETRY_PRIMARY, &arm);
            run_seventh_batch_reproductions(&arm);
        }
        "r2-constants" => {}
        "r2-h1d" => run_held_once_family(&GEOMETRY_PRIMARY, &arm),
        "r2-q1-0" => run_abandonment_step_family(&GEOMETRY_PRIMARY, &arm),
        "r2-pc1" => run_ledger_positive_controls(&GEOMETRY_PRIMARY, &arm),
        "r2-seventh-batch" => run_seventh_batch_reproductions(&arm),
        other => unreachable!("第 2 次跑的模式表 known 已经挡过：{other}"),
    }
    emit_result(&format!(
        "name=done emitted={}",
        emitted_result_line_count() + 1
    ));
    true
}
