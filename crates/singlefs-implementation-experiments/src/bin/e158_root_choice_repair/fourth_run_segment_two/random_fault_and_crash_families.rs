//! 第二段的随机历史家族、实七的故障家族与崩溃家族。

use super::*;

/// H-随、H-随全：每段历史的主路径（逐步的池级 checker 交 `checker_engine` 判）、执行器比对与 harness 那一侧第一次判红。
pub(super) fn run_the_random_families(
    arm: FourthRunArm,
    family: SegmentTwoFamily,
    histories: Vec<SegmentTwoHistory>,
    checker_engine: &GivenStatesEngine,
) -> FamilyRun {
    let mut lines = Vec::new();
    let mut tally = FamilyTally::default();
    let mut compared = 0usize;
    let mut mismatches: Vec<String> = Vec::new();
    let mut segments_without_an_abandoned_red = Vec::new();
    for segment in &histories {
        let (executor, branch) = run_the_main_branch(
            arm,
            segment,
            PoolCheckerOnEveryStep::of_the_family(family, checker_engine),
        );
        tally.segments += 1;
        if executor.panicked.is_some() {
            tally.segments_panicked += 1;
        }
        tally.absorb_branch(&branch);
        if arm.is_today() {
            let trace = harness_trace(&segment.history);
            let (steps, segment_mismatches) = executor_agreement(&executor, &trace);
            compared += steps;
            mismatches.extend(
                segment_mismatches
                    .into_iter()
                    .map(|mismatch| format!("{}:{mismatch}", segment.label)),
            );
            if family == SegmentTwoFamily::RandomRed {
                let harness_red = harness_first_red(checker_engine, &segment.history);
                lines.push(format!(
                    "name=r4_harness_first_red family={} seed={} arm={} harness_first_red_order={} harness_abandoned_named={:?} harness_invariants={:?} device_first_abandoned_red={}",
                    family.name(),
                    segment.label,
                    arm.name,
                    harness_red.as_ref().map_or("none".to_string(), |(order, _, _)| order.to_string()),
                    harness_red.as_ref().map(|(_, named, _)| named.clone()).unwrap_or_default(),
                    harness_red.as_ref().map(|(_, _, invariants)| invariants.clone()).unwrap_or_default(),
                    first_abandoned_red(&executor).map_or("none".to_string(), |(order, named)| format!("order{order}:{named:?}"))
                ));
                if segment.label.starts_with("base+") && first_abandoned_red(&executor).is_none() {
                    segments_without_an_abandoned_red.push(segment.label.clone());
                }
            }
        }
        lines.push(render_random_segment(
            family,
            &segment.label,
            arm,
            &executor,
            &branch,
            segment.history.operations.len(),
        ));
        if family == SegmentTwoFamily::RandomRed {
            lines.extend(
                executor
                    .steps
                    .iter()
                    .map(|step| render_random_step(family, &segment.label, arm, step)),
            );
            if let Some((of, _)) = POSITIVE_CONTROL_RANDOM_SEGMENTS
                .iter()
                .find(|(_, label)| *label == segment.label)
            {
                let (sentences, detail) =
                    positive_control_random_sentences(arm, of, &executor, &branch);
                let fields: Vec<String> = sentences
                    .iter()
                    .map(|one| {
                        format!(
                            "sentence_{}_{}={}",
                            one.tag.name(),
                            one.name,
                            one.verdict.name()
                        )
                    })
                    .collect();
                let routes: Vec<&str> = sentences
                    .iter()
                    .filter(|one| {
                        matches!(
                            one.verdict,
                            SentenceVerdict::Fail | SentenceVerdict::NotConstructible
                        )
                    })
                    .map(|one| one.clause)
                    .collect();
                lines.push(format!(
                    "name=r4_positive_control arm={} control=PC-random of={of} seed={} verdict={} sentences={} {} missed_routes={} {detail}",
                    arm.name,
                    segment.label,
                    overall_verdict(&sentences),
                    sentences.len(),
                    fields.join(" "),
                    if routes.is_empty() { "none".to_string() } else { routes.join(",") }
                ));
            }
        }
        if family.measures_lost_writes() {
            lines.extend(lines_of_a_branch(family, &segment.label, arm, &branch));
        } else {
            lines.extend(
                branch
                    .mounts
                    .iter()
                    .map(|mount| render_random_mount(family, &segment.label, "main", arm, mount)),
            );
        }
    }
    if family == SegmentTwoFamily::RandomRed && arm.is_today() {
        lines.push(format!(
            "name=r4_failure_clause clause=F19 family={} arm={} segments_without_an_abandoned_red={} labels={:?} verdict={}",
            family.name(),
            arm.name,
            segments_without_an_abandoned_red.len(),
            segments_without_an_abandoned_red,
            if segments_without_an_abandoned_red.is_empty() { "not_triggered" } else { "triggered" }
        ));
    }
    let segments_registered_holds = family != SegmentTwoFamily::RandomRed
        || tally.segments == u64::try_from(RANDOM_HISTORY_SEGMENTS_REGISTERED).expect("段数");
    lines.push(format!(
        "name=r4_section_seven_two_anchor arm={} item=\"H-随 实际跑的段数（每臂）\" family={} computed={} registered={} verdict={}",
        arm.name,
        family.name(),
        tally.segments,
        if family == SegmentTwoFamily::RandomRed { RANDOM_HISTORY_SEGMENTS_REGISTERED.to_string() } else { "na".to_string() },
        if segments_registered_holds { "pass" } else { "fail" }
    ));
    if family.measures_lost_writes() {
        lines.push(tally.render(family, arm, ""));
    } else {
        lines.push(format!(
            "name=r4_random_all_summary family={} geometry={HARNESS_GEOMETRY_NAME} arm={} segments={} segments_panicked={} tested_mounts={} tested_classes={:?} s4_mismatches={} f6_checked={} f6_failures={}",
            family.name(),
            arm.name,
            tally.segments,
            tally.segments_panicked,
            tally.mounts,
            tally.classes,
            tally.stop_clause_four_mismatches,
            tally.failure_clause_six_checked,
            tally.failure_clause_six_failures
        ));
    }
    let mut stop = None;
    if arm.is_today() {
        lines.insert(0, agreement_line(family, compared, &mismatches));
        if compared == 0 || !mismatches.is_empty() {
            stop = Some(format!(
                "clause=S9 reason=\"装置执行器与 harness 执行器在今天那一臂上对不上\" compared_steps={compared} mismatches={} first_mismatches={:?}",
                mismatches.len(),
                mismatches.iter().take(3).map(|mismatch| mismatch.replace(' ', "")).collect::<Vec<_>>()
            ));
        }
    }
    if tally.stop_clause_four_mismatches > 0 && stop.is_none() {
        stop = Some(format!(
            "clause=S4 reason=\"7.3 第一二行在这一族的被测挂载上对不上（{} 处）\"",
            tally.stop_clause_four_mismatches
        ));
    }
    FamilyRun { lines, stop }
}

/// 实七-甲（故障注入）。今天那一臂：harness 在这一份副本上抽 6 个注入点（`inject_faults_into_history_merging_the_pool_checker_verdicts_of_the_given_states_engine`），用 harness 测量跑的调用数
/// 换成「步、步内第几次」；S5 按 harness 那一遍与装置这一遍各自复现不复现判。别的臂：从今天那一臂的产物读注入点。
/// 主路径逐步的池级 checker 交 `checker_engine` 判。
pub(super) fn run_the_seventh_fault_family(
    arm: FourthRunArm,
    today_points: &TodayPoints,
    checker_engine: &GivenStatesEngine,
) -> FamilyRun {
    let family = SegmentTwoFamily::SeventhFault;
    let segment = seventh_fault_history();
    let (main, main_branch) = run_the_main_branch(
        arm,
        &segment,
        PoolCheckerOnEveryStep::of_the_family(family, checker_engine),
    );
    let mut lines = Vec::new();
    let mut stop = None;
    let mut harness_reproduces = false;
    let points: Vec<FaultPointKey> = if arm.is_today() {
        let trace = harness_trace(&segment.history);
        let (compared, mismatches) = executor_agreement(&main, &trace);
        lines.push(agreement_line(family, compared, &mismatches));
        if compared == 0 || !mismatches.is_empty() {
            stop = Some(
                "clause=S7 reason=\"装置执行器与 harness 执行器对不上，注入点映不过去\""
                    .to_string(),
            );
        }
        // 池级 checker 那一遍在给定状态引擎两段判，照判红就停那一档的口径并进 harness 的判定
        let fault_engine = given_states_engine_of_this_experiment("e158_seventh_fault_family");
        let mut images_judged = 0usize;
        let injection =
            inject_faults_into_history_merging_the_pool_checker_verdicts_of_the_given_states_engine(
                &segment.history,
                harness_execution(PerStepChecker::Skipped),
                SEVENTH_BATCH_FAULT_REPRODUCTION_FAULTS,
                &mut |images| {
                    images_judged += 1;
                    pool_checker_verdicts_of_the_images_after_a_fault_through_the_engine(
                        &fault_engine,
                        &format!(
                            "seventh-fault-family/{}/hand-out={images_judged}",
                            segment.label
                        ),
                        harness_execution(PerStepChecker::Skipped),
                        images,
                    )
                },
            );
        harness_reproduces = injection.new_findings.iter().any(|finding| {
            finding.reopen.contains(&format!(
                "MappingStillUnreadable {{ slot: SlotNumber({SEVENTH_BATCH_SIGNATURE_SLOT}) }}"
            )) && finding.reopen.contains(&format!(
                "ModelCheckpointTxg({})",
                SEVENTH_BATCH_SIGNATURE_ROOT.1
            ))
        });
        lines.push(format!(
            "name=r4_harness_signature family={} arm={} harness_reproduces={harness_reproduces} drawn={} new_findings={} findings={:?}",
            family.name(),
            arm.name,
            injection.drawn_faults.len(),
            injection.new_findings.len(),
            injection
                .new_findings
                .iter()
                .map(|finding| format!("{}|{}", finding.drawn.render(), finding.reopen))
                .collect::<Vec<_>>()
        ));
        let mut keys = Vec::new();
        for (index, drawn) in injection.drawn_faults.iter().enumerate() {
            let order = match drawn.segment.position() {
                StepPosition::StartingPoint => 0,
                StepPosition::Operation(step_index) => step_index + 1,
            };
            let kind = drawn.fault.call_kind();
            let calls_before = if order == 0 {
                0
            } else {
                trace
                    .iter()
                    .find(|(traced, _, _)| *traced == order - 1)
                    .map_or(0, |(_, _, calls)| calls.of(kind))
            };
            let key = FaultPointKey {
                index,
                fault: drawn.fault,
                order,
                ordinal_within_the_step: drawn.call_ordinal - calls_before,
            };
            lines.push(format!(
                "name=r4_segment_two_fault_point family={} index={index} fault={} order={order} ordinal_within_the_step={} harness_call_ordinal={} harness_drawn={:?}",
                family.name(),
                render_injected_fault(drawn.fault),
                key.ordinal_within_the_step,
                drawn.call_ordinal,
                drawn.render()
            ));
            keys.push(key);
        }
        keys
    } else {
        today_points.fault_points.clone()
    };
    lines.push(render_random_segment(
        family,
        &segment.label,
        arm,
        &main,
        &main_branch,
        segment.history.operations.len(),
    ));
    lines.extend(
        main.steps
            .iter()
            .map(|step| render_random_step(family, &segment.label, arm, step)),
    );
    let mut tally = FamilyTally {
        segments: 1,
        ..FamilyTally::default()
    };
    tally.absorb_branch(&main_branch);
    lines.extend(lines_of_a_branch(family, &segment.label, arm, &main_branch));
    let mut device_reproduces = false;
    for point in &points {
        let branch = with_panic_capture(|| run_a_fault_branch(arm, &segment, &main, point))
            .unwrap_or_else(|panic| BranchResult {
                attempt: format!("fault{}", point.index),
                mounts: Vec::new(),
                cells: Vec::new(),
                detail: format!(
                    "panic={:?}",
                    format!("{}: {}", panic.location, panic.message)
                ),
                fired_call_ordinal: None,
            });
        device_reproduces = device_reproduces
            || branch.cells.iter().any(|(cell, _)| {
                cell.read_backs.iter().any(|(name, read_back)| {
                    name == "reopen_read_back"
                        && read_back.root == Some(SEVENTH_BATCH_SIGNATURE_ROOT)
                        && read_back.failure_debug.as_ref().is_some_and(|debug| {
                            debug.contains("MappingStillUnreadable")
                                && debug.contains(&format!(
                                    "SlotNumber({SEVENTH_BATCH_SIGNATURE_SLOT})"
                                ))
                        })
                })
            });
        lines.push(format!(
            "name=r4_random_branch family={} seed={} arm={} attempt={} mounts={} cells={} fired_call_ordinal={} {}",
            family.name(),
            segment.label,
            arm.name,
            branch.attempt,
            branch.mounts.len(),
            branch.cells.len(),
            optional_number_text(branch.fired_call_ordinal),
            branch.detail
        ));
        tally.absorb_branch(&branch);
        lines.extend(lines_of_a_branch(family, &segment.label, arm, &branch));
    }
    lines.push(tally.render(
        family,
        arm,
        &format!("device_reproduces={device_reproduces}"),
    ));
    if arm.is_today() {
        let verdict = stop_clause_five(harness_reproduces, device_reproduces);
        lines.push(format!(
            "name=r4_stop_check clause=S5 family={} harness_reproduces={harness_reproduces} device_reproduces={device_reproduces} verdict={verdict}",
            family.name()
        ));
        if verdict == "S5a_stop" && stop.is_none() {
            stop = Some(
                "clause=S5a reason=\"harness 复现得出、装置在今天那一臂上复现不出实七-甲的签名\""
                    .to_string(),
            );
        }
    }
    FamilyRun { lines, stop }
}

/// S5a / S5b（登记 11.2）：harness 复现得出而装置复现不出 ⇒ 停（S5a）；harness 也复现不出 ⇒ 不停，标「底座上复现不出」（S5b）。
fn stop_clause_five(harness_reproduces: bool, device_reproduces: bool) -> &'static str {
    match (harness_reproduces, device_reproduces) {
        (true, true) => "reproduced",
        (true, false) => "S5a_stop",
        (false, true) => "S5b_harness_does_not_reproduce_device_does",
        (false, false) => "S5b_not_reproduced_on_the_base",
    }
}

/// 实七-乙、实八（崩溃注入）。今天那一臂：harness 在这一份副本上抽 4 个崩溃状态（`inject_crashes_into_history`），按装置这一遍的录制流
/// 换成「第一个写落在哪一步、那一步里第几段、段内持久哪几个」（段号与写数与 harness 那一遍逐个核，对不上走 S7）。
/// 主路径逐步的池级 checker 与 harness 那一侧第一次判红都交 `checker_engine` 判。
pub(super) fn run_a_crash_family(
    arm: FourthRunArm,
    family: SegmentTwoFamily,
    segment: &SegmentTwoHistory,
    today_points: &TodayPoints,
    checker_engine: &GivenStatesEngine,
) -> FamilyRun {
    let (main, main_branch) = run_the_main_branch(
        arm,
        segment,
        PoolCheckerOnEveryStep::of_the_family(family, checker_engine),
    );
    let stream = segmented_stream_of(&main);
    let mut lines = Vec::new();
    let mut stop = None;
    let mut harness_reproduces = false;
    let points: Vec<CrashPointKey> = if arm.is_today() {
        let trace = harness_trace(&segment.history);
        let (compared, mismatches) = executor_agreement(&main, &trace);
        lines.push(agreement_line(family, compared, &mismatches));
        let injection = inject_crashes_into_history(
            &segment.history,
            harness_execution(PerStepChecker::Skipped),
            CrashPointDraw::Sampled {
                crash_points_per_history: match family {
                    SegmentTwoFamily::SeventhCrash => SEVENTH_BATCH_CRASH_REPRODUCTION_CRASH_POINTS,
                    SegmentTwoFamily::EighthCrash
                    | SegmentTwoFamily::RandomRed
                    | SegmentTwoFamily::RandomAll
                    | SegmentTwoFamily::SeventhFault => {
                        EIGHTH_BATCH_CRASH_REPRODUCTION_CRASH_POINTS
                    }
                },
            },
            None,
            &given_states_engine_of_this_experiment("e158_crash_reproduction"),
        );
        harness_reproduces = match family {
            SegmentTwoFamily::SeventhCrash => injection.new_findings.iter().any(|finding| {
                finding.read_back.contains(&format!(
                    "MappingStillUnreadable {{ slot: SlotNumber({SEVENTH_BATCH_SIGNATURE_SLOT}) }}"
                ))
            }),
            SegmentTwoFamily::EighthCrash
            | SegmentTwoFamily::RandomRed
            | SegmentTwoFamily::RandomAll
            | SegmentTwoFamily::SeventhFault => harness_first_red(checker_engine, &segment.history)
                .is_some_and(|(_, named, _)| named.contains(&EIGHTH_BATCH_SIGNATURE_ROOT)),
        };
        lines.push(format!(
            "name=r4_harness_signature family={} arm={} harness_reproduces={harness_reproduces} crash_points={} new_findings={} findings={:?}",
            family.name(),
            arm.name,
            injection.crash_points.len(),
            injection.new_findings.len(),
            injection
                .new_findings
                .iter()
                .map(|finding| format!("{}|{}|{}", finding.crash_point.render(), finding.stage.name(), finding.read_back.chars().take(200).collect::<String>()))
                .collect::<Vec<_>>()
        ));
        let mut keys = Vec::new();
        let mut mapping_mismatches = Vec::new();
        for (index, crash_point) in injection.crash_points.iter().enumerate() {
            let segment_index = crash_point.segment_index;
            let harness_order = match crash_point.step {
                StepPosition::StartingPoint => 0,
                StepPosition::Operation(step_index) => step_index + 1,
            };
            let agrees = stream.segments.get(segment_index).is_some_and(|writes| {
                writes.len() == crash_point.persisted_within_the_segment.len()
            }) && stream
                .crash_order(segment_index, &crash_point.persisted_within_the_segment)
                == harness_order;
            if !agrees {
                mapping_mismatches.push(format!("crash{index}:{}", crash_point.render()));
                continue;
            }
            let key = CrashPointKey {
                index,
                order_of_the_first_write: stream.order_of_the_first_write[segment_index],
                segment_within_the_step: stream.segment_within_the_step(segment_index),
                persisted_within_the_segment: crash_point.persisted_within_the_segment.clone(),
            };
            lines.push(format!(
                "name=r4_segment_two_crash_point family={} index={index} order_of_the_first_write={} segment_within_the_step={} persisted={} harness_segment_index={segment_index} harness_crash_order={harness_order} harness_point={:?}",
                family.name(),
                key.order_of_the_first_write,
                key.segment_within_the_step,
                render_persisted(&key.persisted_within_the_segment),
                crash_point.render()
            ));
            keys.push(key);
        }
        if compared == 0 || !mismatches.is_empty() || !mapping_mismatches.is_empty() {
            stop = Some(format!(
                "clause=S7 reason=\"装置执行器与 harness 执行器对不上，或崩溃状态映不过去\" executor_mismatches={} mapping_mismatches={:?}",
                mismatches.len(),
                mapping_mismatches
            ));
        }
        keys
    } else {
        today_points
            .crash_points
            .get(family.name())
            .cloned()
            .unwrap_or_default()
    };
    lines.push(render_random_segment(
        family,
        &segment.label,
        arm,
        &main,
        &main_branch,
        segment.history.operations.len(),
    ));
    lines.extend(
        main.steps
            .iter()
            .map(|step| render_random_step(family, &segment.label, arm, step)),
    );
    let mut tally = FamilyTally {
        segments: 1,
        ..FamilyTally::default()
    };
    tally.absorb_branch(&main_branch);
    lines.extend(lines_of_a_branch(family, &segment.label, arm, &main_branch));
    let mut device_reproduces = match family {
        SegmentTwoFamily::EighthCrash => checker_ever_named(&main, EIGHTH_BATCH_SIGNATURE_ROOT),
        SegmentTwoFamily::SeventhCrash
        | SegmentTwoFamily::RandomRed
        | SegmentTwoFamily::RandomAll
        | SegmentTwoFamily::SeventhFault => false,
    };
    for point in &points {
        let branch = with_panic_capture(|| run_a_crash_branch(arm, &main, &stream, point))
            .unwrap_or_else(|panic| BranchResult {
                attempt: format!("crash{}", point.index),
                mounts: Vec::new(),
                cells: Vec::new(),
                detail: format!(
                    "panic={:?}",
                    format!("{}: {}", panic.location, panic.message)
                ),
                fired_call_ordinal: None,
            });
        if family == SegmentTwoFamily::SeventhCrash {
            device_reproduces = device_reproduces
                || branch.cells.iter().any(|(cell, _)| {
                    cell.read_backs.iter().any(|(_, read_back)| {
                        read_back.failure_debug.as_ref().is_some_and(|debug| {
                            debug.contains("MappingStillUnreadable")
                                && debug.contains(&format!(
                                    "SlotNumber({SEVENTH_BATCH_SIGNATURE_SLOT})"
                                ))
                        })
                    })
                });
        }
        lines.push(format!(
            "name=r4_random_branch family={} seed={} arm={} attempt={} mounts={} cells={} {}",
            family.name(),
            segment.label,
            arm.name,
            branch.attempt,
            branch.mounts.len(),
            branch.cells.len(),
            branch.detail
        ));
        tally.absorb_branch(&branch);
        lines.extend(lines_of_a_branch(family, &segment.label, arm, &branch));
    }
    lines.push(tally.render(
        family,
        arm,
        &format!("device_reproduces={device_reproduces}"),
    ));
    if arm.is_today() {
        let verdict = stop_clause_five(harness_reproduces, device_reproduces);
        lines.push(format!(
            "name=r4_stop_check clause=S5 family={} harness_reproduces={harness_reproduces} device_reproduces={device_reproduces} verdict={verdict}",
            family.name()
        ));
        if verdict == "S5a_stop" && stop.is_none() {
            stop = Some(format!(
                "clause=S5a reason=\"harness 复现得出、装置在今天那一臂上复现不出{}的签名\"",
                family.name()
            ));
        }
    }
    FamilyRun { lines, stop }
}
