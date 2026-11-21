//! 第 3 次跑的恰好读失败一次家族、尾部家族与阳性对照。

use super::*;

pub(super) fn run_third_run_held_once_family(geometry: &Geometry, arm: &str) {
    let parameters = parameters_for(geometry);
    let mut summary = ThirdRunFamilySummary::default();
    for overwrites in THIRD_RUN_OVERWRITES {
        let mut history = match third_run_history_start(geometry, overwrites) {
            Ok(history) => history,
            Err(error) => {
                emit_result(&format!(
                    "name=r3_h1d_history arm={arm} n1={overwrites} verdict=stop reason={error:?}"
                ));
                continue;
            }
        };
        let _ = third_run_close(&mut history, &parameters, Closing::ProcessExit);
        let base = history.pool.clone();
        let Ok(pool_geometry) = independent_geometry(&base) else {
            emit_result(&format!(
                "name=r3_h1d_history arm={arm} n1={overwrites} verdict=stop reason=\"几何解不出\""
            ));
            continue;
        };
        let root_slots = root_slot_device_offsets(&pool_geometry);
        let protected = history
            .confirmations
            .last()
            .cloned()
            .expect("P(n1, C) 至少确认过首个文件");
        let last_confirmed = protected.content_number;
        let hidden = history.tip();
        emit_result(&format!(
            "name=r3_h1d_history arm={arm} n1={overwrites} tip={} last_confirmed={last_confirmed} tail_checks={} tail_check_failures={}",
            key_text(Some(hidden)),
            history.tail_checks,
            history.tail_check_failures.len()
        ));
        for failure in &history.tail_check_failures {
            emit_result(&format!(
                "name=r3_f3_tail_check_failure arm={arm} n1={overwrites} detail={failure:?}"
            ));
        }
        for form in [UnreadableForm::ReadFails, UnreadableForm::ReadsZeros] {
            for replay_break in [
                ReplayBreak::EveryRecord,
                ReplayBreak::FirstCopyOfTheFirstNamedUnit,
            ] {
                for (duration, only_the_uncut_cell) in held_once_durations_of(arm) {
                    let head = format!(
                        "n1={overwrites} b={} form={} duration={}",
                        replay_break.name(),
                        form.name(),
                        duration.name()
                    );
                    let faults = match third_run_abandonment_faults(
                        &history,
                        &pool_geometry,
                        form,
                        replay_break,
                        duration,
                    ) {
                        Ok(Some(faults)) => faults,
                        Ok(None) => {
                            emit_result(&format!(
                                "name=r3_h1d_group arm={arm} {head} verdict=not_applicable"
                            ));
                            continue;
                        }
                        Err(error) => {
                            emit_result(&format!(
                                "name=r3_h1d_group arm={arm} {head} verdict=stop reason={error:?}"
                            ));
                            continue;
                        }
                    };
                    let tested = match run_tested_mount(
                        arm,
                        &base,
                        &parameters,
                        &faults,
                        &head,
                        &mut summary,
                    ) {
                        Ok(tested) => tested,
                        Err(error) => {
                            emit_result(&format!(
                                "name=r3_h1d_group arm={arm} {head} verdict=stop reason={error:?}"
                            ));
                            continue;
                        }
                    };
                    let first_root_write = index_of_the_first_root_slot_write(
                        &tested.call.applied,
                        &root_slots,
                        u64::from(pool_geometry.physical_block_size),
                    );
                    let barriers = first_root_write.map_or(0, |index| {
                        barriers_before_write(&tested.call.applied, index + 1)
                    });
                    let cut_cells_expected = if only_the_uncut_cell {
                        1
                    } else {
                        first_root_write.map_or(1, |index| 1 + 2 * (index as u64 + 1) + barriers)
                    };
                    let tested_ok = tested.call.outcome.is_ok();
                    let rollback = tested_ok
                        && tested
                            .summary
                            .effective
                            .and_then(|effective| history.content_number_of(effective))
                            .is_some_and(|number| number < last_confirmed);
                    *summary
                        .tested_classes
                        .entry(format!(
                            "{}:{}",
                            tested.summary.class, tested.summary.member
                        ))
                        .or_insert(0) += 1;
                    let (hidden_in_ring_after, hidden_abandoned_after) =
                        hidden_root_after_the_tested_mount(&tested.call, hidden);
                    if arm == "today" && hidden_abandoned_after == Some(true) {
                        summary.abandoned_hidden_root_cells += 1;
                    }
                    if arm == "today"
                        && tested_ok
                        && replay_break == ReplayBreak::FirstCopyOfTheFirstNamedUnit
                        && duration == FaultDuration::WholeCall
                        && tested.summary.effective != history.timeline.iter().rev().nth(1).copied()
                    {
                        summary.failure_clause_four_findings.push(format!(
                            "{head}: E {}，被藏那条根前一条 {}",
                            key_text(tested.summary.effective),
                            key_text(history.timeline.iter().rev().nth(1).copied())
                        ));
                    }
                    let not_reached = !tested.unintercepted.is_empty();
                    if not_reached {
                        if arm == "today" {
                            summary.void_cells += 1;
                        } else {
                            summary.fault_not_reached_cells += 1;
                        }
                    }
                    let tested_line = format!(
                        "{} {} hidden={} hidden_in_ring_after_a={} q0_hidden_abandoned_after_a={} faults={} unintercepted={} first_root_write={} barriers_before_the_row_root={}",
                        tested.summary.render("a"),
                        render_device_judgement(&tested.judgement),
                        key_text(Some(hidden)),
                        optional_flag_text(hidden_in_ring_after),
                        optional_flag_text(hidden_abandoned_after),
                        faults.len(),
                        tested.unintercepted.join(","),
                        first_root_write.map_or("none".to_string(), |index| index.to_string()),
                        barriers
                    );
                    if arm == "today" && not_reached {
                        emit_result(&format!(
                            "name=r3_h1d_cell arm={arm} {head} cut=none point=none verdict=void_v2 {tested_line}"
                        ));
                        continue;
                    }
                    let mut enumerated = 0u64;
                    evaluate_held_once_cell_of_the_third_run(
                        arm,
                        &format!("{head} cut=none point=none"),
                        &history,
                        &protected,
                        &base,
                        &tested.call.pool_after,
                        &parameters,
                        &tested_line,
                        rollback,
                        tested_ok,
                        true,
                        &mut summary,
                    );
                    enumerated += 1;
                    if !only_the_uncut_cell {
                        if let Some(first_root_write) = first_root_write {
                            for write_count in 0..=first_root_write {
                                let post = image_with_the_first_writes(
                                    &base,
                                    &tested.call.applied,
                                    write_count,
                                );
                                evaluate_held_once_cell_of_the_third_run(
                                    arm,
                                    &format!("{head} cut=crash point={write_count}"),
                                    &history,
                                    &protected,
                                    &base,
                                    &post,
                                    &parameters,
                                    "a_outcome=crashed",
                                    false,
                                    false,
                                    false,
                                    &mut summary,
                                );
                                enumerated += 1;
                            }
                            for failing_write in 0..=first_root_write {
                                let failed = run_third_run_mount_writable(
                                    &base,
                                    &parameters,
                                    &faults,
                                    Some(failing_write as u64),
                                    None,
                                );
                                let failed_summary = TestedMountSummary::of(&failed);
                                let failed_ok = failed.outcome.is_ok();
                                let failed_rollback = failed_ok
                                    && failed_summary
                                        .effective
                                        .and_then(|effective| history.content_number_of(effective))
                                        .is_some_and(|number| number < last_confirmed);
                                evaluate_held_once_cell_of_the_third_run(
                                    arm,
                                    &format!("{head} cut=write_fails point={failing_write}"),
                                    &history,
                                    &protected,
                                    &base,
                                    &failed.pool_after,
                                    &parameters,
                                    &format!(
                                        "{} fired={}",
                                        failed_summary.render("a"),
                                        failed.injected_failure_fired
                                    ),
                                    failed_rollback,
                                    failed_ok,
                                    false,
                                    &mut summary,
                                );
                                enumerated += 1;
                            }
                            for failing_barrier in 0..barriers {
                                let failed = run_third_run_mount_writable(
                                    &base,
                                    &parameters,
                                    &faults,
                                    None,
                                    Some(failing_barrier),
                                );
                                let failed_summary = TestedMountSummary::of(&failed);
                                let failed_ok = failed.outcome.is_ok();
                                let failed_rollback = failed_ok
                                    && failed_summary
                                        .effective
                                        .and_then(|effective| history.content_number_of(effective))
                                        .is_some_and(|number| number < last_confirmed);
                                evaluate_held_once_cell_of_the_third_run(
                                    arm,
                                    &format!("{head} cut=barrier_fails point={failing_barrier}"),
                                    &history,
                                    &protected,
                                    &base,
                                    &failed.pool_after,
                                    &parameters,
                                    &format!(
                                        "{} fired={}",
                                        failed_summary.render("a"),
                                        failed.injected_failure_fired
                                    ),
                                    failed_rollback,
                                    failed_ok,
                                    false,
                                    &mut summary,
                                );
                                enumerated += 1;
                            }
                        }
                    }
                    if enumerated != cut_cells_expected {
                        summary.formula_mismatches.push(format!(
                            "{head}: 枚举 {enumerated} 格，公式 1 + 2(J + 1) + 屏障数 = {cut_cells_expected}"
                        ));
                    }
                    emit_result(&format!(
                        "name=r3_h1d_group arm={arm} {head} cells={enumerated} formula_cells={cut_cells_expected} verdict={}",
                        if enumerated == cut_cells_expected { "pass" } else { "fail_v4" }
                    ));
                }
            }
        }
    }
    summary.emit("h1d", arm);
}

// ---------------------------------------------------------------------------
// 11.6 H1e（登记 5.3 第二行）：P(n1, C) → A（不断）→ 在 A 交回的会话里再做 m 次改内容 → 关闭 C → 撤故障 →
//      `mount_writable`（不注入）→ 冷启动 `recover` 读回。A 被拒的臂，那 m 次没有会话可做，记「因拒没做」。
// ---------------------------------------------------------------------------

fn tail_durations_of(arm: &str) -> Vec<FaultDuration> {
    match ThirdRunCandidate::of(arm) {
        Some(ThirdRunCandidate::Yi) => vec![
            FaultDuration::WholeCall,
            FaultDuration::UntilVirtualTime(THIRD_RUN_VIRTUAL_INTERVAL),
        ],
        Some(ThirdRunCandidate::Today | ThirdRunCandidate::Jia | ThirdRunCandidate::Bing)
        | None => {
            vec![FaultDuration::WholeCall]
        }
    }
}

pub(super) fn run_third_run_tail_family(geometry: &Geometry, arm: &str) {
    let parameters = parameters_for(geometry);
    let mut summary = ThirdRunFamilySummary::default();
    let mut cells_by_duration: BTreeMap<String, u64> = BTreeMap::new();
    for overwrites in THIRD_RUN_OVERWRITES {
        let mut prefix = match third_run_history_start(geometry, overwrites) {
            Ok(history) => history,
            Err(error) => {
                emit_result(&format!(
                    "name=r3_h1e_history arm={arm} n1={overwrites} verdict=stop reason={error:?}"
                ));
                continue;
            }
        };
        let _ = third_run_close(&mut prefix, &parameters, Closing::ProcessExit);
        let base = prefix.pool.clone();
        let Ok(pool_geometry) = independent_geometry(&base) else {
            continue;
        };
        let protected = prefix
            .confirmations
            .last()
            .cloned()
            .expect("P(n1, C) 至少确认过首个文件");
        let hidden = prefix.tip();
        for form in [UnreadableForm::ReadFails, UnreadableForm::ReadsZeros] {
            for replay_break in [
                ReplayBreak::EveryRecord,
                ReplayBreak::FirstCopyOfTheFirstNamedUnit,
            ] {
                for duration in tail_durations_of(arm) {
                    let head = format!(
                        "n1={overwrites} b={} form={} duration={}",
                        replay_break.name(),
                        form.name(),
                        duration.name()
                    );
                    let Ok(Some(faults)) = third_run_abandonment_faults(
                        &prefix,
                        &pool_geometry,
                        form,
                        replay_break,
                        duration,
                    ) else {
                        emit_result(&format!(
                            "name=r3_h1e_group arm={arm} {head} verdict=not_applicable"
                        ));
                        continue;
                    };
                    let tested = match run_tested_mount(
                        arm,
                        &base,
                        &parameters,
                        &faults,
                        &head,
                        &mut summary,
                    ) {
                        Ok(tested) => tested,
                        Err(error) => {
                            emit_result(&format!(
                                "name=r3_h1e_group arm={arm} {head} verdict=stop reason={error:?}"
                            ));
                            continue;
                        }
                    };
                    let not_reached = !tested.unintercepted.is_empty();
                    let tested_ok = tested.call.outcome.is_ok();
                    let last_confirmed_before = protected.content_number;
                    let rollback = tested_ok
                        && tested
                            .summary
                            .effective
                            .and_then(|effective| prefix.content_number_of(effective))
                            .is_some_and(|number| number < last_confirmed_before);
                    let (hidden_in_ring_after, hidden_abandoned_after) =
                        hidden_root_after_the_tested_mount(&tested.call, hidden);
                    if arm == "today" && hidden_abandoned_after == Some(true) {
                        summary.abandoned_hidden_root_cells += 1;
                    }
                    let tested_line = format!(
                        "{} {} hidden={} hidden_in_ring_after_a={} q0_hidden_abandoned_after_a={} faults={} unintercepted={}",
                        tested.summary.render("a"),
                        render_device_judgement(&tested.judgement),
                        key_text(Some(hidden)),
                        optional_flag_text(hidden_in_ring_after),
                        optional_flag_text(hidden_abandoned_after),
                        faults.len(),
                        tested.unintercepted.join(",")
                    );
                    for tail_overwrites in THIRD_RUN_TAIL_OVERWRITES {
                        let coordinates = format!("{head} m={tail_overwrites}");
                        *cells_by_duration.entry(duration.name()).or_insert(0) += 1;
                        *summary
                            .tested_classes
                            .entry(format!(
                                "{}:{}",
                                tested.summary.class, tested.summary.member
                            ))
                            .or_insert(0) += 1;
                        if not_reached {
                            if arm == "today" {
                                summary.void_cells += 1;
                                emit_result(&format!(
                                    "name=r3_h1e_cell arm={arm} {coordinates} verdict=void_v2 {tested_line}"
                                ));
                                continue;
                            }
                            summary.fault_not_reached_cells += 1;
                        }
                        let mut history = prefix.clone();
                        history.pool = tested.call.pool_after.clone();
                        let tail_writes = match &tested.call.outcome {
                            Ok(mounted) => {
                                let effective_content = history
                                    .content_number_of(root_pair(&mounted.output.effective_root))
                                    .unwrap_or(0);
                                push_the_publishes_of_a_mount(
                                    &mut history,
                                    mounted,
                                    effective_content,
                                );
                                history.session = Some(Session {
                                    allocator: mounted.allocator.clone(),
                                    current: mounted.current.clone(),
                                    instance: mounted.output.instance,
                                });
                                let mut done = 0u64;
                                let mut failure = None;
                                for _ in 0..tail_overwrites {
                                    match third_run_change_content(&mut history, &parameters) {
                                        Ok(()) => done += 1,
                                        Err(error) => {
                                            failure = Some(error);
                                            break;
                                        }
                                    }
                                }
                                match failure {
                                    None => format!("done:{done}"),
                                    Some(error) => format!(
                                        "failed_after:{done}:{}",
                                        error_member_of_debug(&error)
                                    ),
                                }
                            }
                            Err(_) => "refused_not_done".to_string(),
                        };
                        let _ = third_run_close(&mut history, &parameters, Closing::ProcessExit);
                        let remount = run_third_run_mount_writable(
                            &history.pool,
                            &parameters,
                            &[],
                            None,
                            None,
                        );
                        let remount_class =
                            third_run_outcome_class(&remount.outcome, remount.write_calls);
                        let first_new_instance = hidden.0 + 1;
                        let loss = evaluate_cell_loss(
                            &history,
                            &protected,
                            &base,
                            &remount.pool_after,
                            Some(first_new_instance),
                        );
                        summary.count(&loss, rollback, tested_ok);
                        *summary
                            .remount_classes
                            .entry(remount_class.to_string())
                            .or_insert(0) += 1;
                        emit_result(&format!(
                            "name=r3_h1e_cell arm={arm} {coordinates} {tested_line} q2_rollback={rollback} tail_writes={tail_writes} last_confirmed={} remount_class={remount_class} remount_error={} {}",
                            history.last_confirmed_content_number(),
                            mount_error_member_or_none(&remount.outcome),
                            loss.render()
                        ));
                    }
                }
            }
        }
    }
    for (duration, cells) in &cells_by_duration {
        let registered = 4 * 2 * 2 * 4;
        let passed = *cells == registered;
        if !passed {
            summary.formula_mismatches.push(format!(
                "H1e 时长 {duration}: {cells} 格，7.2 钉 {registered}"
            ));
        }
        emit_result(&format!(
            "name=r3_section_seven_two_anchor arm={arm} item=\"H1e 每种时长的格数 4×2×2×4\" duration={duration} computed={cells} registered={registered} verdict={}",
            if passed { "pass" } else { "fail_v4" }
        ));
    }
    summary.emit("h1e", arm);
}

// ---------------------------------------------------------------------------
// 11.7 阳性对照（登记 5.4，每一臂都跑）：PC-N、PC-O、PC-N0、PC-丢写、PC-只读；PC-多读在 `r3-compare` 里判。
//      判定词 pass / fail / not_constructible（登记 5.5 第 7 条）。
// ---------------------------------------------------------------------------

fn emit_third_run_positive_control(arm: &str, control: &str, verdict: &str, detail: &str) {
    emit_result(&format!(
        "name=r3_positive_control arm={arm} control={control} verdict={verdict} {detail}"
    ));
}

/// 阳性对照那几格顺带做的 7.3 第一行对拍与 F13（今天那一臂）打成结果行。
fn emit_scratch_findings(family: &str, arm: &str, scratch: &ThirdRunFamilySummary) {
    emit_result(&format!(
        "name=r3_positive_control_cross_checks family={family} arm={arm} s4_mismatches={} f13_checked={} f13_failures={}",
        scratch.stop_condition_four_mismatches.len(),
        scratch.failure_clause_thirteen_checks,
        scratch.failure_clause_thirteen_findings.len()
    ));
    for (kind, list) in [
        ("s4", &scratch.stop_condition_four_mismatches),
        ("f13", &scratch.failure_clause_thirteen_findings),
    ] {
        for detail in list {
            emit_result(&format!(
                "name=r3_family_finding family={family} arm={arm} kind={kind} detail={detail:?}"
            ));
        }
    }
}

pub(super) fn verdict_word(passed: bool) -> &'static str {
    if passed {
        "pass"
    } else {
        "fail"
    }
}

/// PC-N（W；乙 另跑 T(δ)）、PC-丢写、PC-只读：P(1, C) → A（b = records、read_fails）。
pub(super) fn run_third_run_positive_control_newer(geometry: &Geometry, arm: &str) {
    let parameters = parameters_for(geometry);
    let candidate = ThirdRunCandidate::of(arm);
    let mut history = match third_run_history_start(geometry, 1) {
        Ok(history) => history,
        Err(error) => {
            emit_third_run_positive_control(
                arm,
                "PC-N",
                "not_constructible",
                &format!("reason={error:?}"),
            );
            return;
        }
    };
    let _ = third_run_close(&mut history, &parameters, Closing::ProcessExit);
    let base = history.pool.clone();
    let Ok(pool_geometry) = independent_geometry(&base) else {
        emit_third_run_positive_control(arm, "PC-N", "not_constructible", "reason=\"几何解不出\"");
        return;
    };
    let hidden = history.tip();
    let before_hidden = history.timeline.iter().rev().nth(1).copied();
    let protected = history.confirmations.last().cloned().expect("确认过");
    // 乙 三臂另跑 T(δ)（登记 5.4 PC-N 行）；今天、甲、丙 也跑 T(δ)，核 7.3 第三行「T(τ) 对它们与 W 逐格相同」（在 PC-N 上抽跑）。
    for duration in [
        FaultDuration::WholeCall,
        FaultDuration::UntilVirtualTime(THIRD_RUN_VIRTUAL_INTERVAL),
    ] {
        let Ok(Some(faults)) = third_run_abandonment_faults(
            &history,
            &pool_geometry,
            UnreadableForm::ReadFails,
            ReplayBreak::EveryRecord,
            duration,
        ) else {
            emit_third_run_positive_control(
                arm,
                "PC-N",
                "not_constructible",
                "reason=\"落点造不出\"",
            );
            continue;
        };
        let mut scratch = ThirdRunFamilySummary::default();
        let Ok(tested) = run_tested_mount(arm, &base, &parameters, &faults, "PC-N", &mut scratch)
        else {
            emit_third_run_positive_control(
                arm,
                "PC-N",
                "not_constructible",
                "reason=\"起始镜像几何解不出\"",
            );
            continue;
        };
        emit_scratch_findings("pc_n", arm, &scratch);
        let rollback = tested.call.outcome.is_ok()
            && tested
                .summary
                .effective
                .and_then(|effective| history.content_number_of(effective))
                .is_some_and(|number| number < protected.content_number);
        let abandoned_after_reading = tested
            .call
            .outcome
            .is_ok()
            .then(|| abandoned_roots_on(&tested.call.pool_after));
        let abandoned_after = abandoned_after_reading.as_ref().is_some_and(|reading| {
            reading
                .as_ref()
                .is_ok_and(|abandoned| abandoned.contains(&hidden))
        });
        let hidden_in_ring_after =
            independent_geometry(&tested.call.pool_after).is_ok_and(|geometry_after| {
                readable_roots_independent(&tested.call.pool_after, &geometry_after)
                    .contains(&hidden)
            });
        let abandoned_after_detail = match &abandoned_after_reading {
            None => "not_mounted".to_string(),
            Some(Ok(abandoned)) => abandoned
                .iter()
                .map(|root| key_text(Some(*root)))
                .collect::<Vec<_>>()
                .join(","),
            Some(Err(error)) => format!("error:{}", error.replace(' ', "_")),
        };
        let refused_with_zero_writes =
            tested.summary.member == third_run_refusal_member(arm) && tested.summary.writes == 0;
        let (expectation, passed) = match (candidate, duration) {
            (Some(ThirdRunCandidate::Today), _) => (
                "ok_k0_effective_before_hidden_abandoned_after_rollback_one",
                tested.summary.class == "K0"
                    && tested.summary.effective == before_hidden
                    && abandoned_after
                    && rollback,
            ),
            (Some(ThirdRunCandidate::Yi), FaultDuration::UntilVirtualTime(_)) => (
                "ok_after_one_round_chosen_hidden_no_rollback",
                tested.call.outcome.is_ok()
                    && tested.summary.chosen == Some(hidden)
                    && !rollback
                    && tested.summary.waits == THIRD_RUN_REREAD_ROUNDS,
            ),
            (Some(ThirdRunCandidate::Yi), _) => (
                "refused_after_r_rounds_k3",
                refused_with_zero_writes && tested.summary.waits == THIRD_RUN_REREAD_ROUNDS,
            ),
            (Some(ThirdRunCandidate::Jia | ThirdRunCandidate::Bing), _) => {
                ("refused_k3", refused_with_zero_writes)
            }
            (None, _) => ("unknown_arm", false),
        };
        let passed = passed && tested.judgement.truth_newer && tested.unintercepted.is_empty();
        emit_third_run_positive_control(
            arm,
            "PC-N",
            verdict_word(passed),
            &format!(
                "duration={} faults={} expectation={expectation} truth_newer={} rollback={rollback} hidden={} hidden_in_ring_after={hidden_in_ring_after} hidden_abandoned_after={abandoned_after} abandoned_after={abandoned_after_detail} clause_ok={} clause_effective_before_hidden={} clause_rollback_one={} {} {} unintercepted={}",
                duration.name(),
                faults.len(),
                tested.judgement.truth_newer,
                key_text(Some(hidden)),
                tested.call.outcome.is_ok(),
                tested.summary.effective == before_hidden,
                rollback,
                tested.summary.render("a"),
                render_device_judgement(&tested.judgement),
                tested.unintercepted.join(",")
            ),
        );
        if duration != FaultDuration::WholeCall {
            continue;
        }
        // PC-只读：拒了的那一格，同一组故障下冷启动 `recover` 交得出结局、读回 E_故障 那一版。
        if tested.call.outcome.is_err() {
            let read_only = run_third_run_recover(&base, &faults);
            let expected = tested
                .judgement
                .effective
                .and_then(|effective| history.content_number_of(effective));
            let got = match &read_only.outcome.outcome {
                RecoveryOutcome::FileRead { content, .. } => {
                    content_number_of_bytes(content, history.last_confirmed_content_number())
                }
                RecoveryOutcome::NoFile { .. } => Some(0),
                RecoveryOutcome::Failed { .. } => None,
            };
            let read_only_passed = expected.is_some() && got == expected;
            emit_third_run_positive_control(
                arm,
                "PC-read-only",
                verdict_word(read_only_passed),
                &format!(
                    "expected_content={expected:?} got_content={got:?} landing={} e_under_faults={}",
                    key_text(chosen_root_of(&read_only.outcome.outcome)),
                    key_text(tested.judgement.effective)
                ),
            );
        } else {
            emit_third_run_positive_control(
                arm,
                "PC-read-only",
                "not_applicable",
                "reason=\"这一臂在 PC-N 上没拒\"",
            );
        }
        // PC-丢写：PC-N 那段历史走完之后、最后那次 `mount_writable` 之前，把最后确认那一版的数据单元两份都改写成别的字节。
        let (corrupted, loss) = positive_control_lost_write(
            &history,
            &protected,
            &base,
            &tested.call.pool_after,
            &parameters,
        );
        let lost_write_seen =
            corrupted > 0 && loss.overwrite_cell() && loss.read_back != "last_confirmed";
        emit_third_run_positive_control(
            arm,
            "PC-lost-write",
            if corrupted == 0 {
                "not_constructible"
            } else {
                verdict_word(lost_write_seen)
            },
            &format!("corrupted_copies={corrupted} {}", loss.render()),
        );
    }
}

/// PC-丢写（登记 5.4）：被测那次挂载之后（`after_the_tested_mount`）、最后那次 `mount_writable` 之前，
/// 把「被测那次挂载之前最后确认那一版」的数据单元（类标签 数据 / 打包）每一份都改写成 0xA5，再可写挂载、在历史末尾算三样。
/// 交回 (改写了几份, 那一格的三样)。
pub(super) fn positive_control_lost_write(
    history: &ThirdRunHistory,
    protected: &ThirdRunConfirmation,
    base: &MemoryPool,
    after_the_tested_mount: &MemoryPool,
    parameters: &MakeFilesystemParameters,
) -> (u64, CellLoss) {
    let mut devices = devices_from_pool(after_the_tested_mount, IMAGE_BYTES);
    let mut corrupted = 0u64;
    for ((device, slot, span), bytes) in &protected.unit_bytes {
        let is_data_unit = singlefs_checker::check_unit(bytes)
            .is_ok_and(|class| class == UNIT_CLASS_DATA_LOCAL || class == UNIT_CLASS_PACKED_LOCAL);
        if !is_data_unit {
            continue;
        }
        let length = usize::try_from(span * SLOT_BYTES_LOCAL).expect("单元长");
        if let Some((_, target)) = devices
            .iter_mut()
            .find(|(identity, _)| identity.0 == *device)
        {
            target
                .write_at(
                    DeviceOffsetInBytes(slot * SLOT_BYTES_LOCAL),
                    &vec![0xA5; length],
                    WriteDurability::Plain,
                )
                .expect("内存盘写不报错");
            corrupted += 1;
        }
    }
    let post = memory_pool_of(&devices, IMAGE_BYTES);
    let remount = run_third_run_mount_writable(&post, parameters, &[], None, None);
    let loss = evaluate_cell_loss(history, protected, base, &remount.pool_after, None);
    (corrupted, loss)
}

/// PC-O：P(1, C) → A（b = unit_first_copy、read_fails），时长 O(1 只)。
pub(super) fn run_third_run_positive_control_second_read(geometry: &Geometry, arm: &str) {
    let parameters = parameters_for(geometry);
    let candidate = ThirdRunCandidate::of(arm);
    let mut history = match third_run_history_start(geometry, 1) {
        Ok(history) => history,
        Err(error) => {
            emit_third_run_positive_control(
                arm,
                "PC-O",
                "not_constructible",
                &format!("reason={error:?}"),
            );
            return;
        }
    };
    let _ = third_run_close(&mut history, &parameters, Closing::ProcessExit);
    let base = history.pool.clone();
    let Ok(pool_geometry) = independent_geometry(&base) else {
        emit_third_run_positive_control(arm, "PC-O", "not_constructible", "reason=\"几何解不出\"");
        return;
    };
    let hidden = history.tip();
    let before_hidden = history.timeline.iter().rev().nth(1).copied();
    let Ok(Some(faults)) = third_run_abandonment_faults(
        &history,
        &pool_geometry,
        UnreadableForm::ReadFails,
        ReplayBreak::FirstCopyOfTheFirstNamedUnit,
        FaultDuration::OnlyTheNthRead(1),
    ) else {
        emit_third_run_positive_control(arm, "PC-O", "not_constructible", "reason=\"落点造不出\"");
        return;
    };
    let mut scratch = ThirdRunFamilySummary::default();
    let Ok(tested) = run_tested_mount(arm, &base, &parameters, &faults, "PC-O", &mut scratch)
    else {
        emit_third_run_positive_control(
            arm,
            "PC-O",
            "not_constructible",
            "reason=\"起始镜像几何解不出\"",
        );
        return;
    };
    emit_scratch_findings("pc_o", arm, &scratch);
    let ok = tested.call.outcome.is_ok();
    let (expectation, passed) = match candidate {
        Some(ThirdRunCandidate::Today) => (
            "ok_effective_before_hidden",
            ok && tested.summary.effective == before_hidden,
        ),
        Some(ThirdRunCandidate::Jia) => (
            "refused_k3",
            tested.summary.member == third_run_refusal_member(arm) && tested.summary.writes == 0,
        ),
        Some(ThirdRunCandidate::Yi) => (
            "ok_after_one_round_chosen_hidden",
            ok && tested.summary.chosen == Some(hidden) && tested.summary.waits == 1,
        ),
        Some(ThirdRunCandidate::Bing) => (
            "ok_effective_hidden",
            ok && tested.summary.effective == Some(hidden),
        ),
        None => ("unknown_arm", false),
    };
    let passed = passed && tested.unintercepted.is_empty();
    emit_third_run_positive_control(
        arm,
        "PC-O",
        verdict_word(passed),
        &format!(
            "expectation={expectation} faults={} {} {} unintercepted={}",
            faults.len(),
            tested.summary.render("a"),
            render_device_judgement(&tested.judgement),
            tested.unintercepted.join(",")
        ),
    );
}

/// 这次调用写的结构种类与字节的指纹（PC-N0 跨臂比）：取号那几个系统配置槽写把 tail 与整槽校验和那两段抹成 0
/// （-配置续 那 8 个字节按臂的定义与今天不同，另核它 = 装置算的 c_见证）。交回 (指纹, 取号写里解出的 tail 列表)。
pub(super) fn write_fingerprint_of(
    applied: &[AppliedDeviceStep],
    acquisition_writes: usize,
) -> (u64, Vec<u64>) {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let mut mix = |bytes: &[u8]| {
        for byte in bytes {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    };
    let mut acquisition_tails = Vec::new();
    let mut write_index = 0usize;
    for step in applied {
        match step {
            AppliedDeviceStep::Write {
                device,
                offset,
                bytes,
            } => {
                mix(b"W");
                mix(&device.0.to_le_bytes());
                mix(&offset.to_le_bytes());
                if write_index < acquisition_writes
                    && bytes.len() >= SYSTEM_CONFIGURATION_TAIL_OFFSET_LOCAL + 8
                {
                    let mut masked = bytes.clone();
                    let mut tail = [0u8; 8];
                    tail.copy_from_slice(
                        &masked[SYSTEM_CONFIGURATION_TAIL_OFFSET_LOCAL
                            ..SYSTEM_CONFIGURATION_TAIL_OFFSET_LOCAL + 8],
                    );
                    acquisition_tails.push(u64::from_le_bytes(tail));
                    masked[SYSTEM_CONFIGURATION_TAIL_OFFSET_LOCAL
                        ..SYSTEM_CONFIGURATION_TAIL_OFFSET_LOCAL + 8]
                        .fill(0);
                    masked[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET_LOCAL
                        ..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET_LOCAL + WIDE_CHECKSUM_BYTES_LOCAL]
                        .fill(0);
                    mix(&masked);
                } else {
                    mix(bytes);
                }
                write_index += 1;
            }
            AppliedDeviceStep::ZeroFill {
                device,
                offset,
                length,
            } => {
                mix(b"Z");
                mix(&device.0.to_le_bytes());
                mix(&offset.to_le_bytes());
                mix(&length.to_le_bytes());
                write_index += 1;
            }
            AppliedDeviceStep::Barrier { device } => {
                mix(b"B");
                mix(&device.0.to_le_bytes());
            }
        }
    }
    (hash, acquisition_tails)
}

/// PC-N0（登记 5.4）：L0 全部 8 格（P(n1, c)，n1 ∈ {0..3}，c ∈ {C, U}）→ 被测挂载，不注入。每臂要 `Ok`；
/// 与今天逐项相同（K 类、所选根、E、写的结构种类与字节）由 `r3-compare` 判，这里报各项与指纹；-配置续 另核取号写的 tail = c_见证。
pub(super) fn run_third_run_positive_control_no_newer(geometry: &Geometry, arm: &str) {
    let parameters = parameters_for(geometry);
    let mut cells = 0u64;
    for overwrites in THIRD_RUN_OVERWRITES {
        for closing in [Closing::ProcessExit, Closing::NormalUnmount] {
            let coordinates = format!("n1={overwrites} c={}", closing.name());
            let mut history = match third_run_history_start(geometry, overwrites) {
                Ok(history) => history,
                Err(error) => {
                    emit_third_run_positive_control(
                        arm,
                        "PC-N0",
                        "not_constructible",
                        &format!("{coordinates} reason={error:?}"),
                    );
                    continue;
                }
            };
            let unmount_check = match third_run_close(&mut history, &parameters, closing) {
                Ok(check) => check,
                Err(error) => {
                    emit_third_run_positive_control(
                        arm,
                        "PC-N0",
                        "not_constructible",
                        &format!("{coordinates} reason={error:?}"),
                    );
                    continue;
                }
            };
            if let Some((holds, detail)) = &unmount_check {
                emit_result(&format!(
                    "name=r3_section_seven_one_f14 arm={arm} {coordinates} verdict={} {detail}",
                    if *holds { "pass" } else { "fail_f14" }
                ));
            }
            let base = history.pool.clone();
            let mut scratch = ThirdRunFamilySummary::default();
            let Ok(tested) =
                run_tested_mount(arm, &base, &parameters, &[], &coordinates, &mut scratch)
            else {
                emit_third_run_positive_control(
                    arm,
                    "PC-N0",
                    "not_constructible",
                    &format!("{coordinates} reason=\"几何解不出\""),
                );
                continue;
            };
            cells += 1;
            let (fingerprint, acquisition_tails) = write_fingerprint_of(&tested.call.applied, 2);
            let tail_expected = if arm.ends_with("-carry") {
                tested.judgement.witnessed_tail
            } else {
                0
            };
            let tails_hold = acquisition_tails.len() == 2
                && acquisition_tails.iter().all(|tail| *tail == tail_expected);
            let passed =
                tested.summary.class == "K0" && tails_hold && !tested.judgement.truth_newer;
            emit_third_run_positive_control(
                arm,
                "PC-N0",
                verdict_word(passed),
                &format!(
                    "{coordinates} write_fingerprint={fingerprint:016x} acquisition_tails={} acquisition_tail_expected={tail_expected} {} {} s4_mismatches={}",
                    acquisition_tails
                        .iter()
                        .map(u64::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                    tested.summary.render("a"),
                    render_device_judgement(&tested.judgement),
                    scratch.stop_condition_four_mismatches.len()
                ),
            );
            emit_scratch_findings("pc_n0", arm, &scratch);
        }
    }
    let registered = 4 * 2;
    emit_result(&format!(
        "name=r3_section_seven_two_anchor arm={arm} item=\"L0 格数 4×2\" computed={cells} registered={registered} verdict={}",
        if cells == registered { "pass" } else { "fail_v4" }
    ));
}

#[cfg(test)]
mod third_run_tests;
