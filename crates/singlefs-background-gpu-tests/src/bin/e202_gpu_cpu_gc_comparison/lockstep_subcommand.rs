//! 子命令 lockstep（交替跑）。

use super::*;

// ---- 子命令 lockstep（登记第五节「交替跑」，补问题单第 1、2、4、5 行）----

pub(super) struct LockstepCommand {
    pub(super) tier: UnitAreaTier,
    pub(super) user_changes: u64,
    pub(super) batch_units: usize,
    pub(super) arms_requested: Vec<LockstepArm>,
    /// 全部卡臂开几张（`--cards`）。
    pub(super) cards_of_the_every_card_arm: CardsOfTheEveryCardArm,
    pub(super) product: PathBuf,
}

/// 交替跑不拆成先录后判：量 W、C、S、P 是同一个进程、同一次重复里 CPU 臂与 GPU 臂按轮换次序挨着跑的成对差（[`run_one_lockstep_repeat`]、
/// [`judgement_of_a_lockstep_tier`]），H 按一次重复里最忙的那一次判整组作废；把 CPU 臂挪到另一段（另一个进程、另一个时刻、可能另一台）去跑，
/// 成对差的两边就不再同时同机。只把造起步镜像挪到取卡之前（它不在任何计时窗口里）。
pub(super) fn lockstep_command(command: &LockstepCommand) -> i32 {
    let mut product = match ProductWriter::appending_to(&command.product) {
        Ok(product) => product,
        Err(reason) => {
            eprintln!("{reason}");
            return EXIT_RUN_CONDITION_NOT_MET;
        }
    };
    let command_fields = format!(
        "command=lockstep reading=occupancy tier={} user_changes={} batch_units={} predicted_publishes_of_one_intent={} arms={}",
        command.tier.name(),
        command.user_changes,
        command.batch_units,
        publishes_of_one_intent(command.batch_units),
        command.arms_requested.iter().map(|arm| arm.name()).collect::<Vec<String>>().join(",")
    );
    let opening = match opening_of_the_command(
        &mut product,
        &command.product,
        &command_fields,
        CardsOfTheCommand::TakenBeforeOpeningTheDevices,
    ) {
        Ok(opening) => opening,
        Err(exit_code) => {
            product.finish();
            return exit_code;
        }
    };
    if !emit_whether_the_make_filesystem_accepts_the_tier(&mut product, command.tier) {
        product.finish();
        return EXIT_STOPPED;
    }
    let image = match construct_the_starting_image_or_emit_the_stop(&mut product, command.tier) {
        Ok(image) => image,
        Err(exit_code) => {
            product.finish();
            return exit_code;
        }
    };
    if !emit_the_construction(&mut product, &image) {
        product.emit("stopped", GEOMETRY_NOT_FORMED_STOP_FIELDS);
        product.finish();
        return EXIT_STOPPED;
    }
    let cards = match cards_seen_by_the_command(&mut product) {
        Ok(cards) => cards,
        Err(exit_code) => {
            product.finish();
            return exit_code;
        }
    };
    let arms_requested: Vec<(String, ComputeOfTheArm)> = command
        .arms_requested
        .iter()
        .map(|arm| (arm.name(), arm.compute))
        .collect();
    let plan = match emit_the_plan_of_the_run_or_the_refusal(
        &mut product,
        &InputsBeforeTheRun {
            lines_already_in_the_product: &opening.lines_already_in_the_product,
            session_number: opening.session_number,
            run: RunResumedByGroup::Lockstep { tier: command.tier },
            arms_requested: &arms_requested,
            cards_of_the_every_card_arm: command.cards_of_the_every_card_arm,
            cards: &cards,
            host_hardware: &opening.host_hardware,
        },
    ) {
        Ok(plan) => plan,
        Err(exit_code) => {
            product.finish();
            return exit_code;
        }
    };
    let arms_run: Vec<LockstepArm> = plan
        .indices_of_the_arms_run
        .iter()
        .map(|index| command.arms_requested[*index])
        .collect();
    let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
    let setting = LockstepRunSetting {
        image: &image,
        discrete_gpus: &cards.discrete_gpus,
        discrete_gpus_of_the_every_card_arm: &plan.discrete_gpus_of_the_every_card_arm,
        memory_leaked: &memory_leaked,
        vulkan_instance: cards.vulkan_instance.as_ref(),
        user_changes: command.user_changes,
        batch_units: command.batch_units,
    };
    let mut numbers: Vec<RunNumbers> = plan
        .run_lines_of_the_group
        .iter()
        .filter_map(RunNumbers::of_the_fields)
        .collect();
    let mut next_repeat = plan.next_repeat;
    let mut repeats_done = u32::try_from(plan.repeats_of_the_group_done.len()).expect("重复次数");
    let mut voided_repeats = 0_u32;
    let mut gpu_errors = 0_u32;
    while repeats_done < REPEATS_PER_ARM {
        let repeat = next_repeat;
        next_repeat += 1;
        match run_one_lockstep_repeat(
            &setting,
            &arms_run,
            repeat,
            opening.session_number,
            &plan.group,
            &mut product,
        ) {
            RepeatOutcome::Done(runs) => {
                numbers.extend(runs.iter().map(RunNumbers::of_the_summary));
                repeats_done += 1;
            }
            RepeatOutcome::Voided { reason, gpu_error } => {
                voided_repeats += 1;
                gpu_errors += u32::from(gpu_error);
                product.emit(
                    "repeat_voided",
                    &format!(
                        "tier={} repeat={repeat} reason={}",
                        command.tier.name(),
                        field_value(&reason)
                    ),
                );
                if the_tier_stops_after_the_voided_repeats(voided_repeats, gpu_errors) {
                    product.emit("stopped", &format!("clause=too_many_voided_repeats voided={voided_repeats} gpu_errors={gpu_errors}"));
                    product.finish();
                    return EXIT_STOPPED;
                }
            }
            RepeatOutcome::Stopped { reason } => {
                product.emit(
                    "stopped",
                    &format!(
                        "tier={} repeat={repeat} clause={}",
                        command.tier.name(),
                        field_value(&reason)
                    ),
                );
                product.finish();
                return EXIT_STOPPED;
            }
        }
    }
    let judgement = judgement_of_a_lockstep_tier(command.tier, &numbers, &arms_run);
    for (name, fields) in &judgement.lines {
        product.emit(name, &fields_with_the_group(fields, &plan.group));
    }
    product.emit(
        "verdict",
        &fields_with_the_group(
            &format!(
                "tier={} repeats={repeats_done} arms_run={} positive_control_holds_on_every_arm_run={} heavy_threads_registered={}",
                command.tier.name(),
                arms_run.len(),
                judgement.positive_control_holds_on_every_arm_run,
                opening.registered_heavy_threads
            ),
            &plan.group,
        ),
    );
    product.finish();
    if judgement.positive_control_holds_on_every_arm_run {
        EXIT_SUCCESS
    } else {
        EXIT_POSITIVE_CONTROL_FAILED
    }
}
