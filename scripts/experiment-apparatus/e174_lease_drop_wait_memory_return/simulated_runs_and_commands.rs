//! 模拟任务与自检、负对照、各子命令（判定、核对、计划、模拟）。

use super::*;

// ============================== 不起卡的自检：模型卡上跑整套步骤 ==============================

pub(super) fn simulated_job(group: Group, size_mebibytes: u64, liveness: Liveness, method: Method, arm: Arm, round: u32) -> ChildJob {
    ChildJob { cell: CellKey { group, card: Card::Rtx5090, size_mebibytes, liveness, method, arm }, round }
}

/// 在模型卡上跑一个子进程，交回结果和它会打出来的整段文本（借据行加 child_done 行）。
pub(super) fn simulate_job(job: &ChildJob) -> (ChildOutcome, String) {
    let work_microseconds = job.cell.liveness.work_target_microseconds();
    let mut host = SimulatedCardHost::new(job.cell.card, work_microseconds);
    let mut raw_log = RawLog::default();
    let outcome = run_child_procedure(&mut host, job, work_microseconds, true, &mut raw_log);
    let mut text = String::new();
    for lease in &outcome.leases {
        text.push_str(&lease.to_line());
        text.push('\n');
    }
    text.push_str(&child_done_line(job, &outcome, work_microseconds, true, "background_submissions=none adapter=simulated"));
    text.push('\n');
    (outcome, text)
}

/// 模型卡上把某一格的 `rounds` 个轮都跑一遍，拼成一份产物文本。
fn simulated_product_text(cell: CellKey, rounds: u32) -> String {
    (1..=rounds).map(|round| simulate_job(&ChildJob { cell, round }).1).collect()
}

fn leases_have_no_void_reason(outcome: &ChildOutcome) -> bool {
    !outcome.leases.is_empty() && outcome.child_void.is_empty() && outcome.leases.iter().all(|lease| !lease.is_void())
}

/// 每条自检一个名字和是否通过；`checks` 子命令逐条打出来，单测断言一条都不许红。
pub(super) fn self_check_results() -> Vec<(String, bool)> {
    let mut results: Vec<(String, bool)> = Vec::new();
    let expected_piece_plans: [(u64, u64, u64); 5] = [
        (1, 1, MEBIBYTE),
        (64, 1, 64 * MEBIBYTE),
        (512, 1, 512 * MEBIBYTE),
        (2048, 2, GIBIBYTE),
        (8192, 8, GIBIBYTE),
    ];
    for (size_mebibytes, expected_count, expected_each) in expected_piece_plans {
        let (count, each) = pieces_for_size(size_mebibytes * MEBIBYTE);
        results.push((
            format!("b2_piece_plan_{size_mebibytes}_mib"),
            count == expected_count && each == expected_each && count * each == size_mebibytes * MEBIBYTE,
        ));
    }
    for liveness in [Liveness::Done, Liveness::InFlightMedium, Liveness::InFlightHeavy] {
        for arm in [Arm::WaitLatest, Arm::Poll] {
            let job = simulated_job(Group::MainGrid, 2048, liveness, Method::OneBuild, arm, 1);
            results.push((
                format!("simulated_main_grid_{}_{}_has_no_void_reason", liveness.label(), arm.label()),
                leases_have_no_void_reason(&simulate_job(&job).0),
            ));
        }
    }
    let light_job = simulated_job(Group::MainGrid, 64, Liveness::InFlightLight, Method::OneBuild, Arm::WaitLatest, 1);
    let light_outcome = simulate_job(&light_job).0;
    results.push((
        "simulated_light_trips_k9_at_the_registered_numbers".to_string(),
        light_outcome.leases.len() == 1 && light_outcome.leases[0].void_reasons.contains(&VoidReason::WorkNotInFlightAtPoll),
    ));
    let pass_by_pass_job = simulated_job(Group::PassByPassGrid, 64, Liveness::Done, Method::PassByPass, Arm::WaitLatest, 1);
    results.push(("simulated_pass_by_pass_has_no_void_reason".to_string(), leases_have_no_void_reason(&simulate_job(&pass_by_pass_job).0)));
    for arm in [Arm::WaitLatest, Arm::Poll] {
        let hold_job = simulated_job(Group::HoldControl, 2048, Liveness::InFlightMedium, Method::OneBuild, arm, 1);
        results.push((
            format!("simulated_hold_control_{}_still_holds_the_buffer", arm.label()),
            leases_have_no_void_reason(&simulate_job(&hold_job).0),
        ));
    }
    results.extend(negative_control_results());
    results.extend(judge_on_simulated_product_results());
    results.extend(plan_results());
    results
}

/// 检查本身也要分得出差别：模型卡被注入已知的错，对应的作废理由必须出现。
fn negative_control_results() -> Vec<(String, bool)> {
    let job = simulated_job(Group::MainGrid, 64, Liveness::InFlightMedium, Method::OneBuild, Arm::WaitLatest, 1);
    let work_microseconds = job.cell.liveness.work_target_microseconds();
    let mut with_foreign_process = SimulatedCardHost::new(job.cell.card, work_microseconds);
    with_foreign_process.foreign_process_identifier = Some(777);
    let foreign = run_child_procedure(&mut with_foreign_process, &job, work_microseconds, true, &mut RawLog::default());
    let mut with_report_difference = SimulatedCardHost::new(job.cell.card, work_microseconds);
    with_report_difference.compute_applications_extra_mebibytes = 3;
    let different = run_child_procedure(&mut with_report_difference, &job, work_microseconds, true, &mut RawLog::default());
    let out_of_band = run_child_procedure(&mut SimulatedCardHost::new(job.cell.card, work_microseconds), &job, work_microseconds, false, &mut RawLog::default());
    vec![
        ("negative_control_foreign_process_is_v1".to_string(), foreign.child_void.contains(&VoidReason::ForeignProcessListed)),
        ("negative_control_report_difference_is_k5".to_string(), different.child_void.contains(&VoidReason::ProcessReportMismatch)),
        ("negative_control_calibration_out_of_band_is_k6".to_string(), out_of_band.child_void == vec![VoidReason::CalibrationOutOfBand]),
    ]
}

/// 模型卡产物上跑判：等待臂读不出含着、不等待臂读得出含着、小借据按块多拿，判别力自证至少翻一栏。
fn judge_on_simulated_product_results() -> Vec<(String, bool)> {
    let wait_cell = CellKey {
        group: Group::MainGrid,
        card: Card::Rtx5090,
        size_mebibytes: 2048,
        liveness: Liveness::InFlightMedium,
        method: Method::OneBuild,
        arm: Arm::WaitLatest,
    };
    let poll_cell = CellKey { arm: Arm::Poll, ..wait_cell };
    let small_wait_cell = CellKey { size_mebibytes: 64, ..wait_cell };
    let text: String = [wait_cell, poll_cell, small_wait_cell].iter().map(|cell| simulated_product_text(*cell, 5)).collect();
    let rounds = parse_product(&text).expect("模型卡自己打的产物读得回来");
    let judged = judge_product(&rounds, JudgeSettings { tau_override_mebibytes: None });
    let column_of = |cell: CellKey, column: Column| judged.columns.iter().find(|verdict| verdict.cell == cell && verdict.column == column);
    let wait_columns_are_clean = [Column::First, Column::Later].iter().all(|column| {
        column_of(wait_cell, *column).is_some_and(|verdict| {
            verdict.next_presence == Presence::Absent
                && verdict.clear_token == "next_reading_already_absent"
                && verdict.end_presence == Presence::Absent
                && verdict.wait_token == "only_the_work_itself"
                && verdict.rounds_used == 5
        })
    });
    // 模型里不等待臂的第 2、3 笔会在自己那次 poll 顺带收掉上一笔做完了的缓冲，所以第二栏的 R_next 不含、窗口末还含着。
    let poll_first_column_holds = column_of(poll_cell, Column::First).is_some_and(|verdict| {
        verdict.next_presence == Presence::Present && verdict.wait_token == "does_not_wait" && verdict.end_presence == Presence::Present
    });
    let poll_later_column_ends_holding = column_of(poll_cell, Column::Later)
        .is_some_and(|verdict| verdict.wait_token == "does_not_wait" && verdict.end_presence == Presence::Present);
    let poll_columns_hold = poll_first_column_holds && poll_later_column_ends_holding;
    let small_first_column_keeps_the_block = column_of(small_wait_cell, Column::First)
        .is_some_and(|verdict| verdict.next_presence == Presence::Present && verdict.end_presence == Presence::Present && verdict.overtake_presence == Presence::Present);
    let line_round_trip = rounds.values().flat_map(|data| data.leases.iter()).all(|lease| LeaseRecord::from_line(&lease.to_line()).as_ref() == Ok(lease));
    vec![
        ("judge_simulated_wait_arm_2048_is_clean_in_both_columns".to_string(), wait_columns_are_clean),
        ("judge_simulated_poll_arm_2048_holds_the_memory".to_string(), poll_columns_hold),
        ("judge_simulated_small_lease_first_column_keeps_the_block".to_string(), small_first_column_keeps_the_block),
        ("lease_line_round_trips".to_string(), line_round_trip),
        (
            "judge_moving_tau_flips_at_least_one_column".to_string(),
            count_columns_flipped_by_moving_tau(&rounds, &judged) >= 1,
        ),
    ]
}

fn plan_results() -> Vec<(String, bool)> {
    let first = first_segment_cells();
    let second = second_segment_cells();
    let empty_product: ProductRounds = BTreeMap::new();
    let pending = pending_jobs(&first, &empty_product, 2);
    let required_rounds = usize::try_from(ROUNDS_REQUIRED_PER_CELL).expect("轮数装得进 usize");
    let light_cell = CellKey {
        group: Group::MainGrid,
        card: Card::Rtx5090,
        size_mebibytes: 64,
        liveness: Liveness::InFlightLight,
        method: Method::OneBuild,
        arm: Arm::WaitLatest,
    };
    let mut light_product = parse_product(&simulated_product_text(light_cell, 5)).expect("读得回来");
    let mut refills: Vec<u32> = pending_jobs(&[light_cell], &light_product, 2).iter().map(|job| job.round).collect();
    refills.sort_unstable();
    light_product = parse_product(&simulated_product_text(light_cell, 8)).expect("读得回来");
    let after_cap = pending_jobs(&[light_cell], &light_product, 2);
    let second_pending = pending_jobs(&second, &empty_product, 3);
    vec![
        ("plan_first_segment_has_128_cells".to_string(), first.len() == 128),
        ("plan_second_segment_has_24_cells".to_string(), second.len() == 24),
        ("plan_pending_on_an_empty_product_is_five_rounds_per_cell".to_string(), pending.len() == first.len() * required_rounds),
        (
            "plan_hold_control_first_and_light_last".to_string(),
            pending.first().is_some_and(|job| job.cell.group == Group::HoldControl && job.cell.liveness != Liveness::InFlightLight)
                && pending.last().is_some_and(|job| job.cell.liveness == Liveness::InFlightLight),
        ),
        ("plan_refills_are_rounds_6_to_8_when_five_rounds_are_all_void".to_string(), refills == vec![6, 7, 8]),
        ("plan_no_refill_after_three".to_string(), after_cap.is_empty()),
        ("plan_second_segment_pending_is_five_rounds_per_cell".to_string(), second_pending.len() == second.len() * required_rounds),
        ("plan_order_is_the_same_every_time".to_string(), pending == pending_jobs(&first, &empty_product, 2)),
    ]
}

// ============================== 子命令：judge、checks、plan ==============================

fn done_line(lines: &[String]) -> String {
    let emitted = lines.iter().filter(|line| line.starts_with("E7RESULT ")).count() + 1;
    format!("E7RESULT name=done emitted={emitted}")
}

pub(super) fn run_judge_command(arguments: &[String]) -> Result<(), String> {
    let mut input_paths: Vec<&String> = Vec::new();
    let mut scope = "first".to_string();
    let mut index = 0;
    while index < arguments.len() {
        let value = arguments.get(index + 1).ok_or_else(|| format!("{} 后面没有值", arguments[index]))?;
        match arguments[index].as_str() {
            "--input" => input_paths.push(value),
            "--scope" => scope = value.clone(),
            other => return Err(format!("认不出的参数 {other}")),
        }
        index += 2;
    }
    if input_paths.is_empty() {
        return Err("缺 --input".to_string());
    }
    let mut text = String::new();
    for path in input_paths {
        text.push_str(&std::fs::read_to_string(path).map_err(|error| format!("读不了 {path}：{error}"))?);
    }
    let rounds = parse_product(&text)?;
    let judged = judge_product(&rounds, JudgeSettings { tau_override_mebibytes: None });
    let flipped = count_columns_flipped_by_moving_tau(&rounds, &judged);
    let synthetic_flips = judge_on_simulated_product_results()
        .iter()
        .any(|(name, passed)| name == "judge_moving_tau_flips_at_least_one_column" && *passed);
    let mut lines = judged.lines.clone();
    for (main, coexisting) in geometry_pairs(&judged.columns) {
        lines.push(geometry_pair_line(main, coexisting));
    }
    lines.push(verdict_line(&scope, &judged, flipped, synthetic_flips));
    lines.push(done_line(&lines));
    println!("{}", lines.join("\n"));
    Ok(())
}

pub(super) fn run_checks_command() -> i32 {
    let results = self_check_results();
    let mut lines: Vec<String> =
        results.iter().map(|(name, passed)| format!("E7RESULT name=self_check id={name} passed={passed}")).collect();
    let failed = results.iter().filter(|(_, passed)| !*passed).count();
    lines.push(format!(
        "E7RESULT name=verdict scope=checks self_checks_total={} self_checks_failed={failed} void_conditions=0",
        results.len()
    ));
    lines.push(done_line(&lines));
    println!("{}", lines.join("\n"));
    if failed == 0 { 0 } else { 1 }
}

pub(super) fn run_plan_command(arguments: &[String]) -> Result<(), String> {
    let options = parse_parent_options(&[arguments, &["--output".to_string(), "unused".to_string()]].concat())?;
    let cells = cells_of_segment(options.segment, options.skip_light);
    let mut per_stage: BTreeMap<(u32, &'static str), usize> = BTreeMap::new();
    for cell in &cells {
        *per_stage.entry((stage_of(*cell), cell.group.label())).or_default() += 1;
    }
    let mut lines: Vec<String> = per_stage
        .iter()
        .map(|((stage, group), count)| {
            format!(
                "E7RESULT name=plan segment={} stage={stage} group={group} cells={count} first_pass_children={}",
                options.segment.label(),
                count * usize::try_from(ROUNDS_REQUIRED_PER_CELL).expect("轮数装得进 usize")
            )
        })
        .collect();
    lines.push(format!(
        "E7RESULT name=verdict scope=plan segment={} cells_total={} first_pass_children_total={} void_conditions=0",
        options.segment.label(),
        cells.len(),
        cells.len() * usize::try_from(ROUNDS_REQUIRED_PER_CELL).expect("轮数装得进 usize")
    ));
    lines.push(done_line(&lines));
    println!("{}", lines.join("\n"));
    Ok(())
}

/// `simulate` 子命令：在模型卡上跑某一格的几轮，打出产物行；拿来试 `judge`，不起卡。参数同 `child`，再加 `--rounds N`。
pub(super) fn run_simulate_command(arguments: &[String]) -> Result<(), String> {
    let mut job_arguments: Vec<String> = Vec::new();
    let mut rounds: u32 = ROUNDS_REQUIRED_PER_CELL;
    let mut index = 0;
    while index < arguments.len() {
        let value = arguments.get(index + 1).ok_or_else(|| format!("{} 后面没有值", arguments[index]))?;
        if arguments[index] == "--rounds" {
            rounds = value.parse().map_err(|_| "--rounds 不是整数".to_string())?;
        } else {
            job_arguments.push(arguments[index].clone());
            job_arguments.push(value.clone());
        }
        index += 2;
    }
    let job = ChildJob::from_child_arguments(&job_arguments)?;
    print!("{}", simulated_product_text(job.cell, rounds));
    Ok(())
}

pub(super) fn exit_code_of(result: Result<i32, String>) -> i32 {
    match result {
        Ok(code) => code,
        Err(message) => {
            eprintln!("e174：{message}");
            3
        }
    }
}
