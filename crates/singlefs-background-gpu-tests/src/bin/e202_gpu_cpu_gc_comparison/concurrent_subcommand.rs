//! 并发跑的一档判定、一次重复与子命令 concurrent。

use super::*;

/// 一档并发跑的判定行（登记第六节量 F、量 F 底）：每条臂 5 次跑合起来的延迟分布与锁里发布用时的分布、每次跑各自的中位与第 99 百分位；
/// GPU 点名一张卡臂减 CPU 臂的第 99 百分位与最大值（同一次重复成对），5 个同号记方向，有正有负记「不稳定」。只报分布，不判谁好。
fn judgement_of_a_concurrent_tier(
    tier: UnitAreaTier,
    foreground_interval: Duration,
    numbers: &[ConcurrentRunNumbers],
    arms_run: &[ConcurrentArm],
) -> Vec<(String, String)> {
    let mut lines: Vec<(String, String)> = Vec::new();
    let per_run =
        |runs: &[&ConcurrentRunNumbers], value: fn(LatencyDistribution) -> f64| -> Vec<f64> {
            runs.iter()
                .filter_map(|run| latency_distribution_of(&run.latencies).map(value))
                .collect()
        };
    for arm in arms_run {
        let runs = concurrent_runs_of_the_arm(numbers, *arm);
        let pooled_latencies: Vec<f64> = runs
            .iter()
            .flat_map(|run| run.latencies.iter().copied())
            .collect();
        let pooled_publishes: Vec<f64> = runs
            .iter()
            .flat_map(|run| run.publishes_under_the_lock.iter().copied())
            .collect();
        lines.push((
            "latency_distribution".to_string(),
            format!(
                "tier={} arm={} foreground_interval_milliseconds={} runs={} {} {} median_of_each_run={} ninety_ninth_percentile_of_each_run={}",
                tier.name(),
                arm.name(),
                foreground_interval.as_millis(),
                runs.len(),
                distribution_fields("latency_nanoseconds", latency_distribution_of(&pooled_latencies)),
                distribution_fields(
                    "publish_under_the_lock_nanoseconds",
                    latency_distribution_of(&pooled_publishes)
                ),
                joined_values(&per_run(&runs, |distribution| distribution.median)),
                joined_values(&per_run(&runs, |distribution| distribution.high_percentile)),
            ),
        ));
    }
    let cpu_runs = concurrent_runs_of_the_arm(numbers, ConcurrentArm::Cpu);
    let gpu_runs =
        concurrent_runs_of_the_arm(numbers, ConcurrentArm::GpuOnTheCardWithTheLowestPciAddress);
    if arms_run.contains(&ConcurrentArm::Cpu)
        && arms_run.contains(&ConcurrentArm::GpuOnTheCardWithTheLowestPciAddress)
    {
        let statistics: [(&str, StatisticOfALatencyDistribution); 2] = [
            ("ninety_ninth_percentile", |distribution| {
                distribution.high_percentile
            }),
            ("maximum", |distribution| distribution.maximum),
        ];
        for (statistic, value) in statistics {
            let gpu_minus_cpu =
                paired_differences(&per_run(&gpu_runs, value), &per_run(&cpu_runs, value));
            lines.push((
                "latency_comparison".to_string(),
                format!(
                    "tier={} foreground_interval_milliseconds={} statistic={statistic} gpu_one_card_minus_cpu_nanoseconds={} sign={}",
                    tier.name(),
                    foreground_interval.as_millis(),
                    joined_values(&gpu_minus_cpu),
                    sign_of_the_paired_differences(&gpu_minus_cpu, 0.0).name()
                ),
            ));
        }
    }
    lines
}

/// 并发跑一次重复的结局。封闭集合。
enum ConcurrentRepeatOutcome {
    Done(Vec<ConcurrentRunSummary>),
    Voided { reason: String, gpu_error: bool },
    Stopped { reason: String },
}

/// 并发跑一次重复：三条臂按轮换次序串着跑，作废的那一臂重跑一次；跑完核机器负载、读到的内容、甲类（作废第 1–4、7 条，停机第 4、5 条）。
fn run_one_concurrent_repeat(
    setting: &ConcurrentRunSetting<'_>,
    arms_run: &[ConcurrentArm],
    repeat: u32,
    session_number: u64,
    group: &HardwareGroup,
    product: &mut ProductWriter,
) -> ConcurrentRepeatOutcome {
    let tier = setting.image.tier;
    let start = usize::try_from(repeat).expect("重复次数") % arms_run.len().max(1);
    let rotation: Vec<ConcurrentArm> = arms_run[start..]
        .iter()
        .chain(&arms_run[..start])
        .copied()
        .collect();
    let mut runs: Vec<ConcurrentRunSummary> = Vec::new();
    for arm in rotation {
        let mut kept: Option<ConcurrentRunSummary> = None;
        for attempt in 0..2 {
            eprintln!(
                "E202 并发跑 {} 间隔 {} 毫秒 第 {repeat} 次重复 {} 第 {attempt} 次尝试",
                tier.name(),
                setting.foreground_interval.as_millis(),
                arm.name()
            );
            let summary = match run_concurrent_once(setting, arm, repeat) {
                Ok(summary) => summary,
                Err(reason) => return ConcurrentRepeatOutcome::Stopped { reason },
            };
            product.emit(
                "concurrent_run",
                &summary.fields(tier, session_number, setting.foreground_interval),
            );
            for line in summary.forced_fallback.choice_lines(&format!(
                "tier={} arm={} repeat={repeat} session={session_number} foreground_interval_milliseconds={}",
                tier.name(),
                arm.name(),
                setting.foreground_interval.as_millis()
            )) {
                product.emit("forced_choice", &line);
            }
            if summary
                .gpu_work_anchors
                .batches_abandoned_after_a_compute_error
                > 0
            {
                return ConcurrentRepeatOutcome::Voided {
                    reason: format!("{}_abandoned_a_batch_after_a_compute_error", arm.name()),
                    gpu_error: true,
                };
            }
            let is_void = summary.void_reason.is_some();
            kept = Some(summary);
            if !is_void {
                break;
            }
        }
        match kept {
            Some(summary) if summary.void_reason.is_none() => runs.push(summary),
            Some(summary) => {
                return ConcurrentRepeatOutcome::Voided {
                    reason: format!(
                        "{}_void_twice_{}",
                        arm.name(),
                        summary.void_reason.unwrap_or_default()
                    ),
                    gpu_error: false,
                }
            }
            None => {
                return ConcurrentRepeatOutcome::Stopped {
                    reason: "no_attempt_ran".to_string(),
                }
            }
        }
    }
    let busiest_host = runs
        .iter()
        .map(|run| run.host_other_cores)
        .fold(0.0, f64::max);
    let is_identical =
        !runs.is_empty() && runs.iter().all(|run| run.content_matches_the_expectation);
    let class_one_anchors_hold = runs
        .iter()
        .all(ConcurrentRunSummary::class_one_anchors_hold);
    product.emit(
        "concurrent_premise",
        &format!(
            "tier={} repeat={repeat} session={session_number} foreground_interval_milliseconds={} \
             content_matches_the_expectation={is_identical} class_one_anchors_hold={class_one_anchors_hold} \
             busiest_host_other_cores={busiest_host:.2}",
            tier.name(),
            setting.foreground_interval.as_millis()
        ),
    );
    match verdict_of_the_repeat_checks(ChecksOfARepeat {
        busiest_host_other_cores: busiest_host,
        is_identical,
        has_enough_batches: true,
        class_one_anchors_hold,
        class_two_anchors_hold: true,
    }) {
        VerdictOfTheRepeatChecks::Counted => {}
        VerdictOfTheRepeatChecks::Voided(reason) => {
            return ConcurrentRepeatOutcome::Voided {
                reason: reason.to_string(),
                gpu_error: false,
            }
        }
        VerdictOfTheRepeatChecks::Stopped(reason) => {
            return ConcurrentRepeatOutcome::Stopped {
                reason: reason.to_string(),
            }
        }
    }
    product.emit(
        "concurrent_repeat_done",
        &format!(
            "tier={} repeat={repeat} session={session_number} foreground_interval_milliseconds={} group={}",
            tier.name(),
            setting.foreground_interval.as_millis(),
            group.identifier
        ),
    );
    ConcurrentRepeatOutcome::Done(runs)
}

/// 子命令 concurrent 的设定（几何敏感性那一点也走它）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ConcurrentCommand {
    pub(super) tier: UnitAreaTier,
    pub(super) foreground_interval: Duration,
    pub(super) arms_requested: Vec<ConcurrentArm>,
    pub(super) product: PathBuf,
}

/// 并发跑那条命令打在产物头一行（`name=session_start`）上的设定，连同 GC 线程放锁之后让不让（登记第十二节修订第 7 条）。
fn command_fields_of_a_concurrent_command(command: &ConcurrentCommand) -> String {
    format!(
        "command=concurrent reading=occupancy tier={} foreground_interval_milliseconds={} user_changes={CONCURRENT_USER_CHANGES} \
         batch_units={BATCH_UNITS_AT_THE_MAIN_SAMPLING_POINT} gc_thread_yields_after_unlock={GC_THREAD_YIELDS_AFTER_UNLOCK} arms={}",
        command.tier.name(),
        command.foreground_interval.as_millis(),
        command
            .arms_requested
            .iter()
            .map(|arm| arm.name())
            .collect::<Vec<&str>>()
            .join(",")
    )
}

/// 并发跑不拆成先录后判：量 F 与让路读数是同一个进程里 CPU 臂、GPU 臂与不跑 GC 臂按轮换次序挨着跑出来的（量 F 底就是不跑 GC 臂，
/// [`run_one_concurrent_repeat`]），把 CPU 臂挪到另一段去跑，三条臂就不再同时同机。只把造起步镜像挪到取卡之前（它不在任何计时窗口里）。
pub(super) fn concurrent_command(command: &ConcurrentCommand) -> i32 {
    let mut product = match ProductWriter::appending_to(&command.product) {
        Ok(product) => product,
        Err(reason) => {
            eprintln!("{reason}");
            return EXIT_RUN_CONDITION_NOT_MET;
        }
    };
    let command_fields = command_fields_of_a_concurrent_command(command);
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
        .map(|arm| (arm.name().to_string(), arm.compute_of_the_arm()))
        .collect();
    let plan = match emit_the_plan_of_the_run_or_the_refusal(
        &mut product,
        &InputsBeforeTheRun {
            lines_already_in_the_product: &opening.lines_already_in_the_product,
            session_number: opening.session_number,
            run: RunResumedByGroup::Concurrent {
                tier: command.tier,
                foreground_interval_milliseconds: command.foreground_interval.as_millis(),
            },
            arms_requested: &arms_requested,
            cards_of_the_every_card_arm: CardsOfTheEveryCardArm::EveryHandedOverCard,
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
    let arms_run: Vec<ConcurrentArm> = plan
        .indices_of_the_arms_run
        .iter()
        .map(|index| command.arms_requested[*index])
        .collect();
    let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
    let setting = ConcurrentRunSetting {
        image: &image,
        discrete_gpus: &cards.discrete_gpus,
        memory_leaked: &memory_leaked,
        vulkan_instance: cards.vulkan_instance.as_ref(),
        user_changes: CONCURRENT_USER_CHANGES,
        foreground_interval: command.foreground_interval,
    };
    let mut numbers: Vec<ConcurrentRunNumbers> = plan
        .run_lines_of_the_group
        .iter()
        .filter_map(ConcurrentRunNumbers::of_the_fields)
        .collect();
    let mut next_repeat = plan.next_repeat;
    let mut repeats_done = u32::try_from(plan.repeats_of_the_group_done.len()).expect("重复次数");
    let mut voided_repeats = 0_u32;
    let mut gpu_errors = 0_u32;
    while repeats_done < REPEATS_PER_ARM {
        let repeat = next_repeat;
        next_repeat += 1;
        match run_one_concurrent_repeat(
            &setting,
            &arms_run,
            repeat,
            opening.session_number,
            &plan.group,
            &mut product,
        ) {
            ConcurrentRepeatOutcome::Done(runs) => {
                numbers.extend(runs.iter().map(ConcurrentRunNumbers::of_the_summary));
                repeats_done += 1;
            }
            ConcurrentRepeatOutcome::Voided { reason, gpu_error } => {
                voided_repeats += 1;
                gpu_errors += u32::from(gpu_error);
                product.emit(
                    "repeat_voided",
                    &format!(
                        "command=concurrent tier={} repeat={repeat} reason={}",
                        command.tier.name(),
                        field_value(&reason)
                    ),
                );
                if the_tier_stops_after_the_voided_repeats(voided_repeats, gpu_errors) {
                    product.emit(
                        "stopped",
                        &format!("clause=too_many_voided_repeats voided={voided_repeats} gpu_errors={gpu_errors}"),
                    );
                    product.finish();
                    return EXIT_STOPPED;
                }
            }
            ConcurrentRepeatOutcome::Stopped { reason } => {
                product.emit(
                    "stopped",
                    &format!(
                        "command=concurrent tier={} repeat={repeat} clause={}",
                        command.tier.name(),
                        field_value(&reason)
                    ),
                );
                product.finish();
                return EXIT_STOPPED;
            }
        }
    }
    for (name, fields) in judgement_of_a_concurrent_tier(
        command.tier,
        command.foreground_interval,
        &numbers,
        &arms_run,
    ) {
        product.emit(&name, &fields_with_the_group(&fields, &plan.group));
    }
    product.emit(
        "verdict",
        &fields_with_the_group(
            &format!(
                "command=concurrent tier={} foreground_interval_milliseconds={} repeats={repeats_done} arms_run={} heavy_threads_registered={} \
                 yield_bound_reading=upper_bound yield_bound_failure_clause={}",
                command.tier.name(),
                command.foreground_interval.as_millis(),
                arms_run.len(),
                opening.registered_heavy_threads,
                yield_bound_failure_clause_reading(false)
            ),
            &plan.group,
        ),
    );
    product.finish();
    EXIT_SUCCESS
}

#[cfg(test)]
mod tests;
