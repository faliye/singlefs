//! 第 4 次跑的对照格与阳性对照（有更新的根、只读、丢写、二次读、没有更新的根）。

use super::*;

// ---------------------------------------------------------------------------
// 12.6 第一段的阳性对照（登记 5.4，每一臂都跑）：PC-N-环、PC-N-盖、PC-O、PC-丢写、PC-只读、PC-N0（L0 那 8 格）。
//      「与今天逐项相同」「PC-多读」这几句要跨臂，在 `r4-compare` 里判；这里把它们要的字段打进结果行。
// ---------------------------------------------------------------------------

/// 一格被测挂载跑完整段：A → 撤故障 → `mount_writable`（不注入）→ 冷启动 `recover` 读回，交回 A、去向、回滚与三样。
struct FourthRunControlCell {
    tested: FourthRunTestedMount,
    destination: HiddenRootDestination,
    rollback: bool,
    loss: CellLoss,
    remount_line: String,
    write_fingerprint: u64,
}

fn run_fourth_run_control_cell(
    arm: FourthRunArm,
    history: &ThirdRunHistory,
    parameters: &MakeFilesystemParameters,
    faults: &[ThirdRunFault],
    label: &str,
    scratch: &mut ThirdRunFamilySummary,
) -> Result<FourthRunControlCell, String> {
    let base = history.pool.clone();
    let tested = run_fourth_run_tested_mount(arm, &base, parameters, faults, label, scratch)?;
    let protected = history
        .confirmations
        .last()
        .cloned()
        .ok_or("前缀没有确认过")?;
    let hidden = history.tip();
    let tested_ok = tested.call.call.outcome.is_ok();
    let mounted = tested.call.call.outcome.as_ref().ok();
    let rollback = tested_ok
        && tested
            .summary
            .effective
            .and_then(|effective| history.content_number_of(effective))
            .is_some_and(|number| number < protected.content_number);
    let destination = hidden_root_position(&base, hidden).map_or_else(
        || HiddenRootDestination::Other("hidden_root_not_in_base".to_string()),
        |position| {
            hidden_root_destination(
                &position,
                &content_numbers_with_the_publishes_of(history, mounted),
                protected.content_number,
                &tested.call.call.pool_after,
                tested_ok.then_some(tested.summary.effective).flatten(),
            )
        },
    );
    let remount =
        run_third_run_mount_writable(&tested.call.call.pool_after, parameters, &[], None, None);
    let loss = evaluate_cell_loss(
        history,
        &protected,
        &base,
        &remount.pool_after,
        Some(hidden.0 + 1),
    );
    let remount_line = format!(
        "remount_class={} remount_error={}",
        third_run_outcome_class(&remount.outcome, remount.write_calls),
        mount_error_member_or_none(&remount.outcome)
    );
    let (write_fingerprint, _) = write_fingerprint_of(&tested.call.call.applied, 2);
    Ok(FourthRunControlCell {
        tested,
        destination,
        rollback,
        loss,
        remount_line,
        write_fingerprint,
    })
}

/// 两格被测挂载在 7.3 第三行（V5）意义下逐项相同：K 类、错误成员、写次数、读次数、所选根、E、去向、读回、覆盖格。
fn cells_agree_item_by_item(first: &FourthRunControlCell, second: &FourthRunControlCell) -> bool {
    first.tested.summary.class == second.tested.summary.class
        && first.tested.summary.member == second.tested.summary.member
        && first.tested.summary.writes == second.tested.summary.writes
        && first.tested.summary.reads == second.tested.summary.reads
        && first.tested.summary.chosen == second.tested.summary.chosen
        && first.tested.summary.effective == second.tested.summary.effective
        && first.destination == second.destination
        && first.loss.read_back == second.loss.read_back
        && first.loss.overwrite_cell() == second.loss.overwrite_cell()
}

fn refused_before_any_write(cell: &FourthRunControlCell, arm: FourthRunArm) -> bool {
    cell.tested.summary.member == arm.refusal_member() && cell.tested.summary.writes == 0
}

/// 这一臂在这一格上 N 判假（臂的观测：判了 N、第一遍为假）；-槽 臂在读回全零上要这一句（登记 5.4）。
fn witness_judged_false(cell: &FourthRunControlCell) -> bool {
    cell.tested
        .summary
        .observation
        .as_ref()
        .is_some_and(|observation| {
            observation.witness_judged && !observation.newer_root_on_the_first_pass
        })
}

/// PC-N-环（b = `unit_first_copy`）与 PC-N-盖（b = `records`）：P(1, C) → A（`read_fails`、`reads_zeros` × W、T(δ)）→ 撤故障 →
/// `mount_writable`（不注入）→ 冷启动 `recover` 读回。每臂 4 格；W 格上拒了的做 PC-只读；PC-N-环 的 `read_fails`、W 格做 PC-丢写。
pub(super) fn run_fourth_run_positive_control_newer(
    geometry: &Geometry,
    arm: FourthRunArm,
    replay_break: ReplayBreak,
) {
    let control = match replay_break {
        ReplayBreak::FirstCopyOfTheFirstNamedUnit => "PC-N-ring",
        ReplayBreak::EveryRecord | ReplayBreak::FirstNamedUnit => "PC-N-cover",
    };
    let parameters = parameters_for(geometry);
    let history = match fourth_run_plain_prefix(geometry, 1, Closing::ProcessExit, arm.name) {
        Ok(prefix) => prefix.history,
        Err(error) => {
            emit_fourth_run_positive_control(
                arm.name,
                control,
                "form=none duration=none",
                &[sentence(
                    SentenceTag::Measured,
                    "prefix",
                    SentenceVerdict::NotConstructible,
                    "V1",
                )],
                &format!("reason={error:?}"),
            );
            return;
        }
    };
    let Ok(pool_geometry) = independent_geometry(&history.pool) else {
        return;
    };
    let hidden = history.tip();
    let before_hidden = history.timeline.iter().rev().nth(1).copied();
    let hidden_position = hidden_root_position(&history.pool, hidden);
    let mut cells_constructed = 0u64;
    for form in [UnreadableForm::ReadFails, UnreadableForm::ReadsZeros] {
        let mut by_duration: Vec<(FaultDuration, FourthRunControlCell, Vec<ThirdRunFault>)> =
            Vec::new();
        for duration in [
            FaultDuration::WholeCall,
            FaultDuration::UntilVirtualTime(THIRD_RUN_VIRTUAL_INTERVAL),
        ] {
            let coordinates = format!("form={} duration={}", form.name(), duration.name());
            let faults = match fourth_run_abandonment_faults(
                &history,
                &pool_geometry,
                form,
                replay_break,
                duration,
                1,
            ) {
                Ok(Some(faults)) => faults,
                Ok(None) | Err(_) => {
                    emit_fourth_run_positive_control(
                        arm.name,
                        control,
                        &coordinates,
                        &[sentence(
                            SentenceTag::Measured,
                            "faults",
                            SentenceVerdict::NotConstructible,
                            "V1",
                        )],
                        "reason=\"落点造不出\"",
                    );
                    continue;
                }
            };
            let mut scratch = ThirdRunFamilySummary::default();
            match run_fourth_run_control_cell(
                arm,
                &history,
                &parameters,
                &faults,
                control,
                &mut scratch,
            ) {
                Ok(cell) => {
                    emit_fourth_run_scratch_findings(control, arm.name, &scratch);
                    by_duration.push((duration, cell, faults));
                    cells_constructed += 1;
                }
                Err(error) => emit_fourth_run_positive_control(
                    arm.name,
                    control,
                    &coordinates,
                    &[sentence(
                        SentenceTag::Measured,
                        "cell",
                        SentenceVerdict::NotConstructible,
                        "V1",
                    )],
                    &format!("reason={error:?}"),
                ),
            }
        }
        let whole_call_cell = by_duration
            .iter()
            .find(|(duration, _, _)| *duration == FaultDuration::WholeCall)
            .map(|(_, cell, _)| cell);
        for (duration, cell, faults) in &by_duration {
            let coordinates = format!("form={} duration={}", form.name(), duration.name());
            let sentences = sentences_of_the_newer_root_controls(
                arm,
                replay_break,
                form,
                *duration,
                cell,
                whole_call_cell,
                hidden,
                before_hidden,
                hidden_position,
                geometry,
                device_effective_content_is_older_than_the_hidden_one(&history, cell),
            );
            let row_publish = cell
                .tested
                .call
                .call
                .outcome
                .as_ref()
                .ok()
                .map(|mounted| mounted.output.row_publish.root().checkpoint_txg.0);
            emit_fourth_run_positive_control(
                arm.name,
                control,
                &coordinates,
                &sentences,
                &format!(
                    "faults={} hidden={} hidden_slot={} row_publish_txg={} row_publish_slot={} q0_destination={} q2_rollback={} write_fingerprint={:016x} {} {} {} unintercepted={}",
                    faults.len(),
                    key_text(Some(hidden)),
                    hidden_position.map_or("none".to_string(), |position| format!("{}:{}", position.region, position.slot)),
                    row_publish.map_or("none".to_string(), |txg| txg.to_string()),
                    row_publish.map_or("none".to_string(), |txg| {
                        let (region, slot) = predicted_root_ring_slot_of_a_publish(txg, geometry.slots_per_region);
                        format!("{region}:{slot}")
                    }),
                    cell.destination.name(),
                    cell.rollback,
                    cell.write_fingerprint,
                    render_fourth_run_tested_mount(&cell.tested),
                    cell.remount_line,
                    cell.loss.render(),
                    cell.tested.unintercepted.join(",")
                ),
            );
            if *duration != FaultDuration::WholeCall {
                continue;
            }
            run_fourth_run_read_only_control(arm, control, form, &history, cell, faults);
            if replay_break == ReplayBreak::FirstCopyOfTheFirstNamedUnit
                && form == UnreadableForm::ReadFails
            {
                run_fourth_run_lost_write_control(arm, &history, &parameters, cell);
            }
        }
    }
    let registered = 2 * 2;
    emit_result(&format!(
        "name=r4_section_seven_two_anchor arm={} item=\"{control} 每臂 2 种造法 × 2 种时长\" computed={cells_constructed} registered={registered} verdict={}",
        arm.name,
        if cells_constructed == registered { "pass" } else { "fail_v4" }
    ));
}

/// 装置自己重放出的 E_故障 的内容号小于被藏那一版的（PC-N-环、PC-N-盖 里「若今天交回 Ok 且 E 的内容号 < 被藏那一版的」的前件，
/// 取装置的 E、不取 `crates/` 交回的，免得前件与后件（Q2，按 `crates/` 的 E 算）是同一个式子）。
fn device_effective_content_is_older_than_the_hidden_one(
    history: &ThirdRunHistory,
    cell: &FourthRunControlCell,
) -> bool {
    let hidden_content = history.last_confirmed_content_number();
    cell.tested
        .judgement
        .effective
        .and_then(|effective| history.content_number_of(effective))
        .is_some_and(|number| number < hidden_content)
}

/// PC-N-环、PC-N-盖 每一格每一臂要出的句子（登记 5.4 那两行，逐句带标签）。
#[allow(
    clippy::too_many_arguments,
    reason = "一格的坐标、这一格与同一造法的 W 格、被藏那条根的三样原样传进来，拆成结构体只多一层搬运"
)]
fn sentences_of_the_newer_root_controls(
    arm: FourthRunArm,
    replay_break: ReplayBreak,
    form: UnreadableForm,
    duration: FaultDuration,
    cell: &FourthRunControlCell,
    whole_call_cell: Option<&FourthRunControlCell>,
    hidden: TimelineRoot,
    before_hidden: Option<TimelineRoot>,
    hidden_position: Option<HiddenRootPosition>,
    geometry: &Geometry,
    device_effective_is_older: bool,
) -> Vec<ControlSentence> {
    let summary = &cell.tested.summary;
    let ok = cell.tested.call.call.outcome.is_ok();
    let is_whole_call = duration == FaultDuration::WholeCall;
    let cover = replay_break == ReplayBreak::EveryRecord;
    let mut sentences = vec![sentence(
        SentenceTag::Measured,
        "truth_newer",
        SentenceVerdict::of(cell.tested.judgement.truth_newer),
        "V1",
    )];
    let timed_equals_whole_call = || {
        sentence(
            SentenceTag::Measured,
            "timed_equals_whole_call",
            whole_call_cell.map_or(SentenceVerdict::NotConstructible, |whole| {
                SentenceVerdict::of(cells_agree_item_by_item(cell, whole))
            }),
            "V5",
        )
    };
    let refused_before_writing_then_left_as_it_was = || {
        sentence(
            SentenceTag::Measured,
            "if_k3_then_no_rollback_and_left_as_it_was",
            SentenceVerdict::conditional(
                cell.tested.summary.class == "K3",
                !cell.rollback && cell.destination == HiddenRootDestination::LeftAsItWas,
            ),
            "V1",
        )
    };
    let slot_arm_on_zeros = arm.witness == FourthRunWitness::UnreadableRootRingSlot
        && form == UnreadableForm::ReadsZeros;
    match arm.candidate {
        FourthRunCandidate::Today => {
            sentences.push(sentence(
                SentenceTag::TodayDescription,
                "ok_k0_effective_before_hidden",
                SentenceVerdict::of(summary.class == "K0" && summary.effective == before_hidden),
                if cover { "F15" } else { "F4" },
            ));
            if cover {
                let row_txg = cell
                    .tested
                    .call
                    .call
                    .outcome
                    .as_ref()
                    .ok()
                    .map(|mounted| mounted.output.row_publish.root().checkpoint_txg.0);
                sentences.push(sentence(
                    SentenceTag::TodayDescription,
                    "row_publish_txg_equals_hidden_txg",
                    SentenceVerdict::of(row_txg == Some(hidden.1)),
                    "F15",
                ));
                sentences.push(sentence(
                    SentenceTag::TodayDescription,
                    "row_publish_lands_on_the_hidden_slot",
                    SentenceVerdict::of(row_txg.is_some_and(|txg| {
                        hidden_position.is_some_and(|position| {
                            predicted_root_ring_slot_of_a_publish(txg, geometry.slots_per_region)
                                == (position.region, position.slot)
                        })
                    })),
                    "F15",
                ));
                sentences.push(sentence(
                    SentenceTag::TodayDescription,
                    "destination_root_slot_overwritten",
                    SentenceVerdict::of(
                        cell.destination == HiddenRootDestination::RootSlotOverwritten,
                    ),
                    "F15",
                ));
                sentences.push(sentence(
                    SentenceTag::Measured,
                    "if_overwritten_then_reads_back_older_and_counts_as_lost",
                    SentenceVerdict::conditional(
                        cell.destination == HiddenRootDestination::RootSlotOverwritten,
                        cell.loss.read_back.starts_with("older") && cell.loss.lost_write_cell(),
                    ),
                    "V1",
                ));
            } else {
                sentences.push(sentence(
                    SentenceTag::TodayDescription,
                    "hidden_in_ring_and_abandoned",
                    SentenceVerdict::of(
                        cell.destination == HiddenRootDestination::InRingAndAbandoned,
                    ),
                    "F17",
                ));
            }
            sentences.push(sentence(
                SentenceTag::Measured,
                "if_ok_and_older_then_rollback_one",
                SentenceVerdict::conditional(ok && device_effective_is_older, cell.rollback),
                "V1",
            ));
            if !is_whole_call {
                sentences.push(timed_equals_whole_call());
            }
        }
        FourthRunCandidate::Jia => {
            if slot_arm_on_zeros {
                sentences.push(sentence(
                    SentenceTag::ArmDefinition,
                    "slot_witness_false_on_zeros",
                    SentenceVerdict::of(witness_judged_false(cell)),
                    "F12",
                ));
            } else {
                sentences.push(sentence(
                    SentenceTag::ArmDefinition,
                    "refused_k3",
                    SentenceVerdict::of(refused_before_any_write(cell, arm)),
                    "F9",
                ));
                sentences.push(refused_before_writing_then_left_as_it_was());
            }
            if !is_whole_call {
                sentences.push(timed_equals_whole_call());
            }
        }
        FourthRunCandidate::Yi | FourthRunCandidate::YiNarrow => {
            if slot_arm_on_zeros {
                sentences.push(sentence(
                    SentenceTag::ArmDefinition,
                    "slot_witness_false_on_zeros",
                    SentenceVerdict::of(witness_judged_false(cell)),
                    "F12",
                ));
            } else if is_whole_call {
                sentences.push(sentence(
                    SentenceTag::ArmDefinition,
                    "refused_after_r_rounds_k3",
                    SentenceVerdict::of(
                        refused_before_any_write(cell, arm) && summary.waits == arm.reread_rounds,
                    ),
                    "F9",
                ));
            } else {
                sentences.push(sentence(
                    SentenceTag::ArmDefinition,
                    "ok_after_one_round_chosen_hidden_no_rollback",
                    SentenceVerdict::of(
                        ok && summary.chosen == Some(hidden)
                            && !cell.rollback
                            && summary.waits == 1,
                    ),
                    "F16",
                ));
                if cover {
                    let row_txg = cell
                        .tested
                        .call
                        .call
                        .outcome
                        .as_ref()
                        .ok()
                        .map(|mounted| mounted.output.row_publish.root().checkpoint_txg.0);
                    sentences.push(sentence(
                        SentenceTag::ArmDefinition,
                        "row_publish_txg_is_hidden_txg_plus_one",
                        SentenceVerdict::of(row_txg == Some(hidden.1 + 1)),
                        "F16",
                    ));
                    sentences.push(sentence(
                        SentenceTag::ArmDefinition,
                        "destination_chosen_or_applied",
                        SentenceVerdict::of(
                            cell.destination == HiddenRootDestination::ChosenOrApplied,
                        ),
                        "F16",
                    ));
                }
            }
        }
        FourthRunCandidate::Bing => {
            if is_whole_call {
                sentences.push(sentence(
                    SentenceTag::ArmDefinition,
                    "refused_k3",
                    SentenceVerdict::of(refused_before_any_write(cell, arm)),
                    "F9",
                ));
            } else {
                sentences.push(timed_equals_whole_call());
            }
        }
    }
    sentences
}

/// PC-只读（登记 5.4）：W 格上拒了的每一臂，同一组故障下调冷启动 `recover`，交得出结局，读回 E_故障 那一版。
fn run_fourth_run_read_only_control(
    arm: FourthRunArm,
    control: &str,
    form: UnreadableForm,
    history: &ThirdRunHistory,
    cell: &FourthRunControlCell,
    faults: &[ThirdRunFault],
) {
    let coordinates = format!("of={control} form={}", form.name());
    if cell.tested.call.call.outcome.is_ok() {
        emit_result(&format!(
            "name=r4_positive_control arm={} control=PC-read-only {coordinates} verdict=not_applicable reason=\"这一臂在这一格上没拒\"",
            arm.name
        ));
        return;
    }
    let read_only = run_third_run_recover(&history.pool, faults);
    let expected = cell
        .tested
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
    emit_fourth_run_positive_control(
        arm.name,
        "PC-read-only",
        &coordinates,
        &[sentence(
            SentenceTag::ArmDefinition,
            "recover_reads_back_the_version_under_the_faults",
            SentenceVerdict::of(expected.is_some() && got == expected),
            "F9",
        )],
        &format!(
            "expected_content={expected:?} got_content={got:?} landing={} e_under_faults={}",
            key_text(chosen_root_of(&read_only.outcome.outcome)),
            key_text(cell.tested.judgement.effective)
        ),
    );
}

/// PC-丢写（登记 5.4）：PC-N-环 那段历史（`read_fails`、W）走完之后、最后那次 `mount_writable` 之前，
/// 把被藏那一版的数据单元两份都改写成别的字节；每一臂：Q1 覆盖 > 0，Q3 读回不是最后确认那一版。
fn run_fourth_run_lost_write_control(
    arm: FourthRunArm,
    history: &ThirdRunHistory,
    parameters: &MakeFilesystemParameters,
    cell: &FourthRunControlCell,
) {
    let protected = history.confirmations.last().cloned().expect("确认过");
    let (corrupted, loss) = positive_control_lost_write(
        history,
        &protected,
        &history.pool,
        &cell.tested.call.call.pool_after,
        parameters,
    );
    let verdict = if corrupted == 0 {
        SentenceVerdict::NotConstructible
    } else {
        SentenceVerdict::of(loss.overwrite_cell() && loss.read_back != "last_confirmed")
    };
    emit_fourth_run_positive_control(
        arm.name,
        "PC-lost-write",
        "form=read_fails duration=W",
        &[sentence(
            SentenceTag::Measured,
            "overwrite_counted_and_not_last_confirmed",
            verdict,
            "V1",
        )],
        &format!("corrupted_copies={corrupted} {}", loss.render()),
    );
}

/// PC-O（第 3 次跑登记 5.4 PC-O 行，标签照第 4 次跑 5.4）：P(1, C) → A（b = `unit_first_copy`、`read_fails`、O(1 只)）。
pub(super) fn run_fourth_run_positive_control_second_read(geometry: &Geometry, arm: FourthRunArm) {
    let parameters = parameters_for(geometry);
    let history = match fourth_run_plain_prefix(geometry, 1, Closing::ProcessExit, arm.name) {
        Ok(prefix) => prefix.history,
        Err(error) => {
            emit_fourth_run_positive_control(
                arm.name,
                "PC-O",
                "duration=O1only",
                &[sentence(
                    SentenceTag::Measured,
                    "prefix",
                    SentenceVerdict::NotConstructible,
                    "V1",
                )],
                &format!("reason={error:?}"),
            );
            return;
        }
    };
    let Ok(pool_geometry) = independent_geometry(&history.pool) else {
        return;
    };
    let hidden = history.tip();
    let before_hidden = history.timeline.iter().rev().nth(1).copied();
    let Ok(Some(faults)) = fourth_run_abandonment_faults(
        &history,
        &pool_geometry,
        UnreadableForm::ReadFails,
        ReplayBreak::FirstCopyOfTheFirstNamedUnit,
        FaultDuration::OnlyTheNthRead(1),
        1,
    ) else {
        emit_fourth_run_positive_control(
            arm.name,
            "PC-O",
            "duration=O1only",
            &[sentence(
                SentenceTag::Measured,
                "faults",
                SentenceVerdict::NotConstructible,
                "V1",
            )],
            "reason=\"落点造不出\"",
        );
        return;
    };
    let mut scratch = ThirdRunFamilySummary::default();
    let Ok(cell) =
        run_fourth_run_control_cell(arm, &history, &parameters, &faults, "PC-O", &mut scratch)
    else {
        return;
    };
    emit_fourth_run_scratch_findings("PC-O", arm.name, &scratch);
    let summary = &cell.tested.summary;
    let ok = cell.tested.call.call.outcome.is_ok();
    let mut sentences = vec![sentence(
        SentenceTag::Measured,
        "truth_newer",
        SentenceVerdict::of(cell.tested.judgement.truth_newer),
        "V1",
    )];
    if arm.is_today() {
        sentences.push(sentence(
            SentenceTag::Measured,
            "faults_reached",
            SentenceVerdict::of(cell.tested.unintercepted.is_empty()),
            "V2",
        ));
    }
    sentences.push(match arm.candidate {
        FourthRunCandidate::Today => sentence(
            SentenceTag::TodayDescription,
            "ok_effective_before_hidden",
            SentenceVerdict::of(ok && summary.effective == before_hidden),
            "F4",
        ),
        FourthRunCandidate::Jia => sentence(
            SentenceTag::ArmDefinition,
            "refused_k3",
            SentenceVerdict::of(refused_before_any_write(&cell, arm)),
            "F9",
        ),
        FourthRunCandidate::Yi | FourthRunCandidate::YiNarrow => sentence(
            SentenceTag::ArmDefinition,
            "ok_after_one_round_chosen_hidden",
            SentenceVerdict::of(ok && summary.chosen == Some(hidden) && summary.waits == 1),
            "F16",
        ),
        FourthRunCandidate::Bing => sentence(
            SentenceTag::ArmDefinition,
            "ok_effective_hidden",
            SentenceVerdict::of(ok && summary.effective == Some(hidden)),
            "F9",
        ),
    });
    emit_fourth_run_positive_control(
        arm.name,
        "PC-O",
        "duration=O1only",
        &sentences,
        &format!(
            "faults={} q0_destination={} {} {} {} unintercepted={}",
            faults.len(),
            cell.destination.name(),
            render_fourth_run_tested_mount(&cell.tested),
            cell.remount_line,
            cell.loss.render(),
            cell.tested.unintercepted.join(",")
        ),
    );
}

/// PC-N0（L0 全部 8 格）：P(n1, c) → 被测挂载，不注入。每臂 `Ok`（K0）、N_真 为假、取号写的 tail（-配置续 = c_见证，别的 = 0）；
/// 与今天逐项相同、PC-多读 (ii) 的点值在 `r4-compare` 里判（这里打出读数、指纹与 p）。L0 族（第四段）用同一批格。
pub(super) fn run_fourth_run_positive_control_no_newer(geometry: &Geometry, arm: FourthRunArm) {
    let parameters = parameters_for(geometry);
    let mut cells = 0u64;
    for overwrites in THIRD_RUN_OVERWRITES {
        for closing in [Closing::ProcessExit, Closing::NormalUnmount] {
            let coordinates = format!("n1={overwrites} c={}", closing.name());
            let history = match fourth_run_plain_prefix(geometry, overwrites, closing, arm.name) {
                Ok(prefix) => prefix.history,
                Err(error) => {
                    emit_fourth_run_positive_control(
                        arm.name,
                        "PC-N0",
                        &coordinates,
                        &[sentence(
                            SentenceTag::Measured,
                            "prefix",
                            SentenceVerdict::NotConstructible,
                            "V1",
                        )],
                        &format!("reason={error:?}"),
                    );
                    continue;
                }
            };
            let base = history.pool.clone();
            let mut scratch = ThirdRunFamilySummary::default();
            let Ok(tested) = run_fourth_run_tested_mount(
                arm,
                &base,
                &parameters,
                &[],
                &coordinates,
                &mut scratch,
            ) else {
                continue;
            };
            cells += 1;
            let (fingerprint, acquisition_tails) =
                write_fingerprint_of(&tested.call.call.applied, 2);
            let tail_expected = if arm.witness == FourthRunWitness::SystemConfigurationCarried {
                tested.judgement.witnessed_tail
            } else {
                0
            };
            let tails_hold = acquisition_tails.len() == 2
                && acquisition_tails.iter().all(|tail| *tail == tail_expected);
            let instance_table_pages = tested.call.call.outcome.as_ref().ok().and_then(|mounted| {
                let devices = devices_from_pool(&base, IMAGE_BYTES);
                singlefs_core::recovery::instance_table_chain_of_root(
                    &devices,
                    &mounted.output.effective_root,
                )
                .ok()
                .map(|chain| chain.page_pointers.len())
            });
            let sentences = vec![
                sentence(
                    SentenceTag::Measured,
                    "truth_not_newer",
                    SentenceVerdict::of(!tested.judgement.truth_newer),
                    "V1",
                ),
                sentence(
                    SentenceTag::ArmDefinition,
                    "ok_k0",
                    SentenceVerdict::of(tested.summary.class == "K0"),
                    "F12",
                ),
                sentence(
                    SentenceTag::ArmDefinition,
                    "acquisition_tails",
                    SentenceVerdict::of(tails_hold),
                    "F12",
                ),
            ];
            emit_fourth_run_positive_control(
                arm.name,
                "PC-N0",
                &coordinates,
                &sentences,
                &format!(
                    "write_fingerprint={fingerprint:016x} base_fingerprint={:016x} acquisition_tails={} acquisition_tail_expected={tail_expected} effective_instance_table_pages={} {} s4_mismatches={}",
                    image_fingerprint(&base),
                    acquisition_tails
                        .iter()
                        .map(u64::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                    instance_table_pages.map_or("none".to_string(), |pages| pages.to_string()),
                    render_fourth_run_tested_mount(&tested),
                    scratch.stop_condition_four_mismatches.len()
                ),
            );
            emit_fourth_run_scratch_findings("PC-N0", arm.name, &scratch);
        }
    }
    let registered = 4 * 2;
    emit_result(&format!(
        "name=r4_section_seven_two_anchor arm={} item=\"L0 格数 4×2\" computed={cells} registered={registered} verdict={}",
        arm.name,
        if cells == registered { "pass" } else { "fail_v4" }
    ));
}
