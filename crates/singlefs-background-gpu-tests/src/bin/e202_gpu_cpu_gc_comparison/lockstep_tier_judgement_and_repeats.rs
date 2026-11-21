//! 一档交替跑的判定与一次重复：六条臂按轮换次序串着跑，前提、作废与停机。

use super::*;

// ---- 一档交替跑的判定（登记第六节量 W、C、S、P；量 T 要三档，在 `judge-trend`）----

/// 判定用的那几个数：这一次跑刚量出的，或续跑时从产物的 `name=run` 行读回的。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct RunNumbers {
    pub(super) arm: LockstepArm,
    pub(super) repeat: u32,
    batches: u64,
    gc_wall_nanoseconds: f64,
    process_cpu_nanoseconds: f64,
    inner_rejudge_cpu_nanoseconds: f64,
}

impl RunNumbers {
    pub(super) fn of_the_summary(summary: &LockstepRunSummary) -> RunNumbers {
        RunNumbers {
            arm: summary.arm,
            repeat: summary.repeat,
            batches: summary.batches,
            gc_wall_nanoseconds: nanoseconds_as_float(summary.gc_wall),
            process_cpu_nanoseconds: nanoseconds_as_float(summary.process_cpu),
            inner_rejudge_cpu_nanoseconds: nanoseconds_as_float(summary.inner_rejudge_cpu),
        }
    }

    pub(super) fn of_the_fields(fields: &BTreeMap<String, String>) -> Option<RunNumbers> {
        let number = |key: &str| fields.get(key).and_then(|value| value.parse::<f64>().ok());
        Some(RunNumbers {
            arm: LockstepArm::of_the_name(fields.get("arm")?)?,
            repeat: fields.get("repeat")?.parse().ok()?,
            batches: fields.get("batches")?.parse().ok()?,
            gc_wall_nanoseconds: number("gc_wall_nanoseconds")?,
            process_cpu_nanoseconds: number("process_cpu_nanoseconds")?,
            inner_rejudge_cpu_nanoseconds: number("inner_rejudge_cpu_nanoseconds")?,
        })
    }
}

/// 一条臂按重复次序排好的那几个数。
fn numbers_of_the_arm(
    numbers: &[RunNumbers],
    arm: LockstepArm,
    value: impl Fn(&RunNumbers) -> f64,
) -> Vec<f64> {
    let mut of_the_arm: Vec<&RunNumbers> = numbers.iter().filter(|run| run.arm == arm).collect();
    of_the_arm.sort_by_key(|run| run.repeat);
    of_the_arm.into_iter().map(value).collect()
}

pub(super) fn joined_values(values: &[f64]) -> String {
    values
        .iter()
        .map(|value| format!("{value:.0}"))
        .collect::<Vec<String>>()
        .join(",")
}

/// 一档交替跑的判定行与阳性对照过没过。
pub(super) struct JudgementOfATier {
    pub(super) lines: Vec<(String, String)>,
    pub(super) positive_control_holds_on_every_arm_run: bool,
}

pub(super) fn judgement_of_a_lockstep_tier(
    tier: UnitAreaTier,
    numbers: &[RunNumbers],
    arms_run: &[LockstepArm],
) -> JudgementOfATier {
    let mut lines: Vec<(String, String)> = Vec::new();
    for arm in arms_run {
        let walls = numbers_of_the_arm(numbers, *arm, |run| run.gc_wall_nanoseconds);
        let processes = numbers_of_the_arm(numbers, *arm, |run| run.process_cpu_nanoseconds);
        lines.push((
            "arm_spread".to_string(),
            format!(
                "tier={} arm={} repeats={} {} {}",
                tier.name(),
                arm.name(),
                walls.len(),
                spread_fields("gc_wall_nanoseconds", &walls),
                spread_fields("process_cpu_nanoseconds", &processes)
            ),
        ));
    }
    let cpu_arm = LOCKSTEP_ARMS[0];
    let cpu_walls = numbers_of_the_arm(numbers, cpu_arm, |run| run.gc_wall_nanoseconds);
    let cpu_processes = numbers_of_the_arm(numbers, cpu_arm, |run| run.process_cpu_nanoseconds);
    let cpu_batches = numbers_of_the_arm(numbers, cpu_arm, |run| count_as_float(run.batches));
    let mut positive_control_holds_on_every_arm_run = true;
    for compared in arms_run
        .iter()
        .filter(|arm| arm.compute.uses_the_gpu() && arm.rejudging == RejudgingOfTheArm::NotRejudged)
    {
        let wall_differences = paired_differences(
            &cpu_walls,
            &numbers_of_the_arm(numbers, *compared, |run| run.gc_wall_nanoseconds),
        );
        lines.push((
            "wall_clock_comparison".to_string(),
            format!(
                "tier={} against={} cpu_minus_gpu_nanoseconds={} sign={}",
                tier.name(),
                compared.name(),
                joined_values(&wall_differences),
                sign_of_the_paired_differences(&wall_differences, 0.0).name()
            ),
        ));
        let savings = paired_differences(
            &cpu_processes,
            &numbers_of_the_arm(numbers, *compared, |run| run.process_cpu_nanoseconds),
        );
        let savings_per_batch: Vec<f64> = savings
            .iter()
            .zip(&cpu_batches)
            .map(|(saving, batches)| saving / batches.max(1.0))
            .collect();
        lines.push((
            "saving".to_string(),
            format!(
                "tier={} against={} cpu_minus_gpu_nanoseconds={} median={:.0} minimum={:.0} maximum={:.0} sign={}",
                tier.name(),
                compared.name(),
                joined_values(&savings),
                median_of(&savings),
                minimum_of(&savings),
                maximum_of(&savings),
                sign_of_the_paired_differences(&savings, 0.0).name()
            ),
        ));
        lines.push((
            "saving_per_batch".to_string(),
            format!(
                "tier={} against={} values={}",
                tier.name(),
                compared.name(),
                joined_values(&savings_per_batch)
            ),
        ));
    }
    for rejudged in arms_run
        .iter()
        .filter(|arm| arm.rejudging == RejudgingOfTheArm::EveryConfirmedEntryRejudgedOnTheCpu)
    {
        let plain = LockstepArm {
            compute: rejudged.compute,
            rejudging: RejudgingOfTheArm::NotRejudged,
        };
        let control = positive_control_of_an_arm(
            &numbers_of_the_arm(numbers, plain, |run| run.process_cpu_nanoseconds),
            &numbers_of_the_arm(numbers, *rejudged, |run| run.process_cpu_nanoseconds),
            &numbers_of_the_arm(numbers, *rejudged, |run| run.inner_rejudge_cpu_nanoseconds),
            rejudged
                .compute
                .uses_the_gpu()
                .then_some(cpu_processes.as_slice()),
        );
        positive_control_holds_on_every_arm_run &= control.holds();
        lines.push((
            "positive_control".to_string(),
            format!(
                "tier={} arm={} every_delta_positive={} median_delta_nanoseconds={:.0} median_inner_rejudge_cpu_nanoseconds={:.0} \
                 median_delta_within_the_band_of_the_inner_timing={} saving_with_rejudging_below_the_saving_in_every_pair={} holds={}",
                tier.name(),
                plain.name(),
                control.every_delta_positive,
                control.median_delta,
                control.median_inner_rejudge_cpu,
                control.median_delta_within_the_band_of_the_inner_timing,
                control
                    .saving_with_rejudging_below_the_saving_in_every_pair
                    .map_or_else(|| "not_a_gpu_arm".to_string(), |below| below.to_string()),
                control.holds()
            ),
        ));
    }
    JudgementOfATier {
        lines,
        positive_control_holds_on_every_arm_run,
    }
}

// ---- 一次重复（六条臂按轮换次序串着跑）与它的前提、作废、停机 ----

/// 一次重复的结局。封闭集合。
pub(super) enum RepeatOutcome {
    /// 前提都过、锚点都成立：这几次跑计数。
    Done(Vec<LockstepRunSummary>),
    /// 作废（作废第 1–4、7 条）：同一档重排一次；`gpu_error` 是作废第 4 条那一种。
    Voided { reason: String, gpu_error: bool },
    /// 停机条款触发（或交主 agent 的那几种）：产物照打，退 3。
    Stopped { reason: String },
}

/// 同一次重复里各臂的录制摘要与读到的内容摘要逐个相同、读到的等于期望、多判对不上的条数都是 0（前提 I）。
fn the_runs_of_the_repeat_are_identical(runs: &[LockstepRunSummary]) -> bool {
    let Some(first_run) = runs.first() else {
        return false;
    };
    runs.iter().all(|run| {
        run.recording_digest == first_run.recording_digest
            && run.content_digest == first_run.content_digest
            && run.content_matches_the_expectation
            && run.rejudge.mismatches == 0
    })
}

/// 一次重复里各臂的批数都相等且至少 64（前提 N）。
fn the_runs_of_the_repeat_have_enough_batches(runs: &[LockstepRunSummary]) -> bool {
    let batches_of_each_run: Vec<u64> = runs.iter().map(|run| run.batches).collect();
    batches_are_equal_and_at_least_the_minimum(&batches_of_each_run)
}

/// 前提 N 的纯函数：每次跑的批数都等于头一次跑的、且不少于 [`MINIMUM_BATCHES_PER_RUN`]；一次跑都没有不算够。
fn batches_are_equal_and_at_least_the_minimum(batches_of_each_run: &[u64]) -> bool {
    batches_of_each_run.first().is_some_and(|first_batches| {
        batches_of_each_run
            .iter()
            .all(|batches| batches == first_batches && *batches >= MINIMUM_BATCHES_PER_RUN)
    })
}

/// 一档里作废的重复多于 2 次（作废第 3 条），或 GPU 臂出错放弃一批累计 3 次（作废第 4 条）：这一档停下交主 agent。
/// 出错放弃一批的那一次重复也算一次作废。
pub(super) fn the_tier_stops_after_the_voided_repeats(
    voided_repeats: u32,
    gpu_errors: u32,
) -> bool {
    gpu_errors >= GPU_ERRORS_PER_TIER_TO_STOP || voided_repeats > VOIDED_REPEATS_PER_TIER_AT_MOST
}

/// 一次重复的各臂都跑完之后那几样核的读数。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ChecksOfARepeat {
    /// 这次重复里各次跑期间本进程以外平均占的核数，取最大的那次（量 H）。
    pub(super) busiest_host_other_cores: f64,
    /// 前提 I（交替跑：录制流与内容摘要各臂相同、读到的等于期望、多判对不上 0 条；并发跑：读到的等于期望）。
    pub(super) is_identical: bool,
    /// 前提 N（并发跑不看批数，恒为真）。
    pub(super) has_enough_batches: bool,
    pub(super) class_one_anchors_hold: bool,
    pub(super) class_two_anchors_hold: bool,
}

/// 那几样核的结局。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum VerdictOfTheRepeatChecks {
    Counted,
    /// 作废第 3 条：同一档重排一次。
    Voided(&'static str),
    /// 停机条款第 4、5 条，前提 N 的阶梯走完，作废第 6 条（那一档作废、交主 agent）：产物照打，退 3。
    Stopped(&'static str),
}

/// 先看机器忙不忙（忙的那一次重复整组作废，别的读数不作数），再依次看前提 I、前提 N、甲类、乙类（停机条款第 4 条看的是甲类那一样）。
pub(super) fn verdict_of_the_repeat_checks(checks: ChecksOfARepeat) -> VerdictOfTheRepeatChecks {
    if checks.busiest_host_other_cores > HOST_LOAD_CORES_AT_MOST {
        VerdictOfTheRepeatChecks::Voided("the_host_was_busy")
    } else if !checks.is_identical {
        VerdictOfTheRepeatChecks::Stopped("the_arms_of_the_repeat_did_not_write_and_read_the_same")
    } else if !checks.has_enough_batches {
        VerdictOfTheRepeatChecks::Stopped(
            "fewer_than_64_batches_or_unequal_batches_climb_the_user_change_ladder",
        )
    } else if !checks.class_one_anchors_hold {
        VerdictOfTheRepeatChecks::Stopped("a_class_one_anchor_does_not_hold")
    } else if !checks.class_two_anchors_hold {
        VerdictOfTheRepeatChecks::Stopped("a_class_two_anchor_does_not_hold_the_tier_is_void")
    } else {
        VerdictOfTheRepeatChecks::Counted
    }
}

/// 第 `repeat` 次重复的轮换次序：从第 `repeat` 条臂起（登记第十四节）。
fn arms_in_the_rotation_of_the_repeat(arms_run: &[LockstepArm], repeat: u32) -> Vec<LockstepArm> {
    let start = usize::try_from(repeat).expect("重复次数") % arms_run.len().max(1);
    arms_run[start..]
        .iter()
        .chain(&arms_run[..start])
        .copied()
        .collect()
}

/// `name=run` 行里 GPU 臂的型号串：这条臂取卡的那一份里每张卡的型号，以 `+` 连。
pub(super) fn gpu_models_in_the_run_line(
    discrete_gpus_of_the_arm: &[DiscreteGpuSeenByThisProcess],
) -> String {
    discrete_gpus_of_the_arm
        .iter()
        .map(|gpu| field_value(&gpu.model))
        .collect::<Vec<String>>()
        .join("+")
}

pub(super) fn run_one_lockstep_repeat(
    setting: &LockstepRunSetting<'_>,
    arms_run: &[LockstepArm],
    repeat: u32,
    session_number: u64,
    group: &HardwareGroup,
    product: &mut ProductWriter,
) -> RepeatOutcome {
    let tier = setting.image.tier;
    let mut runs: Vec<LockstepRunSummary> = Vec::new();
    for arm in arms_in_the_rotation_of_the_repeat(arms_run, repeat) {
        let models_of_the_arm = if arm.compute.uses_the_gpu() {
            gpu_models_in_the_run_line(setting.discrete_gpus_of_the_arm(arm.compute))
        } else {
            "none".to_string()
        };
        let mut kept: Option<LockstepRunSummary> = None;
        for attempt in 0..2 {
            eprintln!(
                "E202 交替跑 {} 第 {repeat} 次重复 {} 第 {attempt} 次尝试",
                tier.name(),
                arm.name()
            );
            let summary = match run_lockstep_once(setting, arm, repeat) {
                Ok(summary) => summary,
                Err(reason) => return RepeatOutcome::Stopped { reason },
            };
            product.emit(
                "run",
                &summary.fields(tier, session_number, &models_of_the_arm),
            );
            for window in &summary.windows_failing_the_inclusion_checks {
                eprintln!(
                    "E202 交替跑 {} 第 {repeat} 次重复 {} 第 {attempt} 次尝试判不过包含自检的窗口：{window:?}",
                    tier.name(),
                    arm.name()
                );
            }
            for line in summary.forced_fallback.choice_lines(&format!(
                "tier={} arm={} repeat={repeat} session={session_number}",
                tier.name(),
                arm.name()
            )) {
                product.emit("forced_choice", &line);
            }
            if summary
                .gpu_work_anchors
                .batches_abandoned_after_a_compute_error
                > 0
            {
                return RepeatOutcome::Voided {
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
                return RepeatOutcome::Voided {
                    reason: format!(
                        "{}_void_twice_{}",
                        arm.name(),
                        summary.void_reason.unwrap_or_default()
                    ),
                    gpu_error: false,
                }
            }
            None => {
                return RepeatOutcome::Stopped {
                    reason: "no_attempt_ran".to_string(),
                }
            }
        }
    }
    let busiest_host = runs
        .iter()
        .map(|run| run.host_other_cores)
        .fold(0.0, f64::max);
    let is_identical = the_runs_of_the_repeat_are_identical(&runs);
    let has_enough_batches = the_runs_of_the_repeat_have_enough_batches(&runs);
    let class_one_anchors_hold = runs.iter().all(LockstepRunSummary::class_one_anchors_hold);
    let class_two_anchors_hold = runs.iter().all(LockstepRunSummary::class_two_anchors_hold);
    product.emit(
        "premise",
        &format!(
            "tier={} repeat={repeat} session={session_number} identical={is_identical} enough_batches={has_enough_batches} \
             batches={} class_one_anchors_hold={class_one_anchors_hold} class_two_anchors_hold={class_two_anchors_hold} \
             busiest_host_other_cores={busiest_host:.2}",
            tier.name(),
            runs.iter().map(|run| run.batches.to_string()).collect::<Vec<String>>().join(",")
        ),
    );
    match verdict_of_the_repeat_checks(ChecksOfARepeat {
        busiest_host_other_cores: busiest_host,
        is_identical,
        has_enough_batches,
        class_one_anchors_hold,
        class_two_anchors_hold,
    }) {
        VerdictOfTheRepeatChecks::Counted => {}
        VerdictOfTheRepeatChecks::Voided(reason) => {
            return RepeatOutcome::Voided {
                reason: reason.to_string(),
                gpu_error: false,
            }
        }
        VerdictOfTheRepeatChecks::Stopped(reason) => {
            return RepeatOutcome::Stopped {
                reason: reason.to_string(),
            }
        }
    }
    product.emit(
        "repeat_done",
        &format!(
            "tier={} repeat={repeat} session={session_number} group={}",
            tier.name(),
            group.identifier
        ),
    );
    RepeatOutcome::Done(runs)
}

#[cfg(test)]
mod tests;
