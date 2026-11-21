use super::*;

// ───────────────────────────── 包含自检与各量的判定（登记第六节） ─────────────────────────────

/// 包含自检（登记第五节）：一批的挂钟要包住 CPU + 等盘。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum InclusionVerdict {
    Holds,
    /// 挂钟 − CPU − 等盘超过挂钟 10%：那一轮作废重跑（机器被别的活挤）。
    UnaccountedAboveTenPercent,
    /// CPU + 等盘大过挂钟：计时口径有错（停机条款第 6 条）。
    CpuPlusWaitAboveTheWall,
}

impl InclusionVerdict {
    pub(super) const fn label(self) -> &'static str {
        match self {
            InclusionVerdict::Holds => "holds",
            InclusionVerdict::UnaccountedAboveTenPercent => "unaccounted_above_ten_percent",
            InclusionVerdict::CpuPlusWaitAboveTheWall => "cpu_plus_wait_above_the_wall",
        }
    }
}

pub(super) fn inclusion_verdict(
    wall_nanoseconds: u64,
    cpu_nanoseconds: u64,
    wait_nanoseconds: i64,
) -> InclusionVerdict {
    let accounted = i128::from(cpu_nanoseconds) + i128::from(wait_nanoseconds);
    let wall = i128::from(wall_nanoseconds);
    if accounted > wall {
        return InclusionVerdict::CpuPlusWaitAboveTheWall;
    }
    let unaccounted = (wall - accounted) as f64;
    if unaccounted > UNACCOUNTED_SHARE_ABOVE_FOR_INVALID * wall as f64 {
        InclusionVerdict::UnaccountedAboveTenPercent
    } else {
        InclusionVerdict::Holds
    }
}

/// Q1 的判法（门槛由调用方给：判别力自证把门槛挪到两点之间再判一次）。
pub(super) fn judgement_of_rho(
    rho: f64,
    at_most_for_disk: f64,
    at_least_for_cpu: f64,
) -> &'static str {
    if rho <= at_most_for_disk {
        "disk"
    } else if rho >= at_least_for_cpu {
        "cpu"
    } else {
        "both"
    }
}

/// Q1：一批（或一条意图）的 CPU 时间 ÷ 等盘时间，按登记的两条边判。
pub(super) fn judgement_of_rho_at_the_registered_edges(rho: f64) -> &'static str {
    judgement_of_rho(rho, RHO_AT_MOST_FOR_DISK, RHO_AT_LEAST_FOR_CPU)
}

/// Q2 的判法：真盘 ÷ 内存的挂钟比。
pub(super) fn judgement_of_wall_ratio(
    ratio: f64,
    at_least_for_disk: f64,
    at_most_for_cpu: f64,
) -> &'static str {
    if ratio >= at_least_for_disk {
        "disk"
    } else if ratio <= at_most_for_cpu {
        "cpu"
    } else {
        "both"
    }
}

pub(super) fn judgement_of_wall_ratio_at_the_registered_edges(ratio: f64) -> &'static str {
    judgement_of_wall_ratio(
        ratio,
        WALL_RATIO_AT_LEAST_FOR_DISK,
        WALL_RATIO_AT_MOST_FOR_CPU,
    )
}

/// Q3：|CPU_真盘 − 挂钟_内存| ÷ 挂钟_内存。
pub(super) fn cpu_reconciliation(cpu_on_the_real_disk: f64, wall_on_the_memory_device: f64) -> f64 {
    (cpu_on_the_real_disk - wall_on_the_memory_device).abs() / wall_on_the_memory_device
}

pub(super) fn judgement_of_the_cpu_reconciliation(reconciliation: f64) -> &'static str {
    if reconciliation <= CPU_RECONCILIATION_AT_MOST {
        "compatible"
    } else {
        "paths_disagree"
    }
}

/// Q4：每轮 `挂钟(2K) ÷ 挂钟(K)`；5 轮中位 > 1 且每轮都 > 1 判「跟着涨」。
pub(super) fn judgement_of_the_batch_doubling(ratios_per_round: &[f64]) -> &'static str {
    let every_round_rises =
        !ratios_per_round.is_empty() && ratios_per_round.iter().all(|ratio| *ratio > 1.0);
    match median_of(ratios_per_round) {
        Some(median) if median > 1.0 && every_round_rises => "rises",
        Some(_) => "does_not_rise",
        None => "not_run",
    }
}

/// Q5–Q7：份额 ≥ 10% 判「值得」。
pub(super) fn judgement_of_share(share: f64, at_least_for_worth: f64) -> &'static str {
    if share >= at_least_for_worth {
        "worth"
    } else {
        "not_worth"
    }
}

/// Q9：D24 形式的比 < 1 判「这一段在 GPU 上输」。
#[cfg_attr(
    not(feature = "gpu"),
    allow(
        dead_code,
        reason = "第三段（gpu 特性）才用；没编 gpu 特性时只在单测里用"
    )
)]
pub(super) fn judgement_of_the_sixteen_thread_to_gpu_ratio(ratio: f64) -> &'static str {
    if ratio < SIXTEEN_THREAD_TO_GPU_RATIO_BELOW_FOR_GPU_LOSES {
        "gpu_loses"
    } else {
        "gpu_has_a_chance"
    }
}

/// Q13 的判法：φ。
pub(super) fn judgement_of_fixed_share(
    share: f64,
    at_most_for_one_batch: f64,
    at_least_for_merging: f64,
) -> &'static str {
    if share <= at_most_for_one_batch {
        "one_batch_per_submission"
    } else if share >= at_least_for_merging {
        "merge_batches"
    } else {
        "merging_optional"
    }
}

#[cfg_attr(
    not(feature = "gpu"),
    allow(
        dead_code,
        reason = "第三段（gpu 特性）才用；没编 gpu 特性时只在单测里用"
    )
)]
pub(super) fn judgement_of_fixed_share_at_the_registered_edges(share: f64) -> &'static str {
    judgement_of_fixed_share(
        share,
        FIXED_SHARE_AT_MOST_FOR_ONE_BATCH,
        FIXED_SHARE_AT_LEAST_FOR_MERGING,
    )
}

/// Q16 的判法：GC 单跑写带宽 ÷ 设备顺序写带宽。
pub(super) fn judgement_of_bandwidth_ratio(
    ratio: f64,
    at_most_for_single_run: f64,
) -> &'static str {
    if ratio <= at_most_for_single_run {
        "single_run_enough"
    } else {
        "needs_the_concurrent_measurement"
    }
}

pub(super) fn judgement_of_bandwidth_ratio_at_the_registered_edge(ratio: f64) -> &'static str {
    judgement_of_bandwidth_ratio(ratio, BANDWIDTH_RATIO_AT_MOST_FOR_SINGLE_RUN)
}

/// Q17：并发时前台 p99 ÷ 不跑 GC 时的 p99。
pub(super) fn judgement_of_the_foreground_fsync_latency_ratio(ratio: f64) -> &'static str {
    if ratio >= FOREGROUND_PERCENTILE_99_RATIO_AT_LEAST_FOR_DRAG {
        "gc_drags_the_foreground"
    } else {
        "gc_drags_little"
    }
}

/// 判别力自证（登记第八节）：门槛挪到量到的两点之间，两点的判定必须不同。交回（挪到的门槛、低点判定、高点判定）。
pub(super) fn judgements_with_the_threshold_moved_between(
    lower_value: f64,
    higher_value: f64,
    judge: impl Fn(f64, f64) -> &'static str,
) -> (f64, &'static str, &'static str) {
    let moved = f64::midpoint(lower_value, higher_value);
    (moved, judge(lower_value, moved), judge(higher_value, moved))
}

/// 相对变化 (对照 − 基线) ÷ 基线。
fn relative_change(control: f64, baseline: f64) -> f64 {
    (control - baseline) / baseline
}

/// 第一行盘臂对照：等盘增量 ÷（写调用次数 × 2 ms）落在 [1.0, 1.5]，CPU 变化在 ±10% 以内。
fn disk_arm_control_holds(
    wait_increment_nanoseconds: f64,
    write_calls: f64,
    cpu_change: f64,
) -> bool {
    let injected = write_calls * (INJECTED_MILLISECONDS_PER_WRITE * 1_000_000) as f64;
    let ratio = wait_increment_nanoseconds / injected;
    (DISK_CONTROL_RATIO_LOWER..=DISK_CONTROL_RATIO_UPPER).contains(&ratio)
        && cpu_change.abs() <= CONTROL_TOLERANCE
}

/// 第一行 CPU 臂对照 CPU 那一句：CPU 增量落在写调用次数 × 2 ms 的 ±10% 里（登记第五节原文，没改）。
fn cpu_arm_cpu_clause_holds(cpu_increment_nanoseconds: f64, injected_nanoseconds: f64) -> bool {
    relative_change(cpu_increment_nanoseconds, injected_nanoseconds).abs() <= CONTROL_TOLERANCE
}

/// 第一行 CPU 臂对照等盘那一句（登记第十二节 12.9 的新判法）：写调用的等盘增量的绝对值 ≤ 写调用次数 × 2 ms × 10%。
/// 只算注入所在的写调用，分母是注入量：内存盘上的基线等盘是读钟嵌套留下的负的小数，拿它当分母分不开装置正确与 V1（等盘不减 CPU）。
fn cpu_arm_write_wait_clause_holds(
    write_wait_increment_nanoseconds: f64,
    injected_nanoseconds: f64,
) -> bool {
    write_wait_increment_nanoseconds.abs() <= injected_nanoseconds * CONTROL_TOLERANCE
}

/// 第一行 CPU 臂对照等盘那一句的原判据（登记第五节原文「等盘时间的变化在 ±10% 以内」）：交错跑的那一趟起照算照报，不作判定用（12.9）。
fn cpu_arm_original_wait_clause_holds(wait_change: f64) -> bool {
    wait_change.abs() <= CONTROL_TOLERANCE
}

/// 第一行阳性对照一格里一轮整条意图的读数（从 [`StoredIntent`] 取；单测直接喂基线先跑完、对照后跑的那一趟产物里 `name=call` 行按轮加出来的数）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct RowOneControlRound {
    pub(super) round: usize,
    pub(super) cpu_nanoseconds: f64,
    pub(super) wait_nanoseconds: f64,
    pub(super) write_calls: f64,
    pub(super) write_wait_nanoseconds: f64,
}

/// 第一行阳性对照一格的判定。盘臂那一行没改（登记第五节原文，一个合取）；CPU 臂两句各自报，原判据那一句只报不判（12.9）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RowOneControlJudgement {
    DiskArm {
        holds: bool,
    },
    CpuArm {
        cpu_clause_holds: bool,
        write_wait_clause_holds: bool,
        wait_clause_under_the_original_criterion_holds: bool,
    },
}

impl RowOneControlJudgement {
    pub(super) const fn holds(self) -> bool {
        match self {
            RowOneControlJudgement::DiskArm { holds } => holds,
            RowOneControlJudgement::CpuArm {
                cpu_clause_holds,
                write_wait_clause_holds,
                wait_clause_under_the_original_criterion_holds: _,
            } => cpu_clause_holds && write_wait_clause_holds,
        }
    }
}

/// 第一行阳性对照一格（设备 × 注入）的读数与判定。增量一律取「对照各轮的中位 − 基线各轮的中位」（与基线先跑完、对照后跑的那一趟取法相同，没改）；
/// 按轮号配对的增量中位另报，不作判定用（12.9）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct RowOneControlReading {
    pub(super) write_calls: f64,
    pub(super) injected_nanoseconds: f64,
    pub(super) cpu_increment_nanoseconds: f64,
    pub(super) wait_increment_nanoseconds: f64,
    pub(super) write_wait_increment_nanoseconds: f64,
    pub(super) cpu_change: f64,
    pub(super) wait_change: f64,
    pub(super) paired_rounds: usize,
    pub(super) cpu_increment_paired_median_nanoseconds: Option<f64>,
    pub(super) write_wait_increment_paired_median_nanoseconds: Option<f64>,
    pub(super) judgement: RowOneControlJudgement,
}

/// 同一轮号的「对照 − 基线」之差（只取两边都有的轮号，按对照的次序）。
fn paired_round_differences(
    baseline: &[RowOneControlRound],
    control: &[RowOneControlRound],
    value_of: fn(&RowOneControlRound) -> f64,
) -> Vec<f64> {
    control
        .iter()
        .filter_map(|control_round| {
            baseline
                .iter()
                .find(|baseline_round| baseline_round.round == control_round.round)
                .map(|baseline_round| value_of(control_round) - value_of(baseline_round))
        })
        .collect()
}

/// 判一格阳性对照。基线或对照一轮都没有交回 `None`（那一格记 `not_run`）；注入是 `Nothing` 也交回 `None`（基线对基线不是对照）。
pub(super) fn row_one_control_reading(
    injection: WriteInjection,
    baseline: &[RowOneControlRound],
    control: &[RowOneControlRound],
) -> Option<RowOneControlReading> {
    let median_over = |rounds: &[RowOneControlRound], value_of: fn(&RowOneControlRound) -> f64| {
        median_of(&rounds.iter().map(value_of).collect::<Vec<_>>())
    };
    let baseline_cpu = median_over(baseline, |round| round.cpu_nanoseconds)?;
    let control_cpu = median_over(control, |round| round.cpu_nanoseconds)?;
    let baseline_wait = median_over(baseline, |round| round.wait_nanoseconds)?;
    let control_wait = median_over(control, |round| round.wait_nanoseconds)?;
    let baseline_write_wait = median_over(baseline, |round| round.write_wait_nanoseconds)?;
    let control_write_wait = median_over(control, |round| round.write_wait_nanoseconds)?;
    let write_calls = median_over(control, |round| round.write_calls)?;
    let injected_nanoseconds = write_calls * (INJECTED_MILLISECONDS_PER_WRITE * 1_000_000) as f64;
    let cpu_increment_nanoseconds = control_cpu - baseline_cpu;
    let wait_increment_nanoseconds = control_wait - baseline_wait;
    let write_wait_increment_nanoseconds = control_write_wait - baseline_write_wait;
    let cpu_change = relative_change(control_cpu, baseline_cpu);
    let wait_change = relative_change(control_wait, baseline_wait);
    let judgement = match injection {
        WriteInjection::SleepTwoMilliseconds => RowOneControlJudgement::DiskArm {
            holds: disk_arm_control_holds(wait_increment_nanoseconds, write_calls, cpu_change),
        },
        WriteInjection::SpinTwoMillisecondsOfThreadCpu => {
            let cpu_clause_holds =
                cpu_arm_cpu_clause_holds(cpu_increment_nanoseconds, injected_nanoseconds);
            let write_wait_clause_holds = cpu_arm_write_wait_clause_holds(
                write_wait_increment_nanoseconds,
                injected_nanoseconds,
            );
            let wait_clause_under_the_original_criterion_holds =
                cpu_arm_original_wait_clause_holds(wait_change);
            RowOneControlJudgement::CpuArm {
                cpu_clause_holds,
                write_wait_clause_holds,
                wait_clause_under_the_original_criterion_holds,
            }
        }
        WriteInjection::Nothing => return None,
    };
    let cpu_differences =
        paired_round_differences(baseline, control, |round| round.cpu_nanoseconds);
    let write_wait_differences =
        paired_round_differences(baseline, control, |round| round.write_wait_nanoseconds);
    Some(RowOneControlReading {
        write_calls,
        injected_nanoseconds,
        cpu_increment_nanoseconds,
        wait_increment_nanoseconds,
        write_wait_increment_nanoseconds,
        cpu_change,
        wait_change,
        paired_rounds: write_wait_differences.len(),
        cpu_increment_paired_median_nanoseconds: median_of(&cpu_differences),
        write_wait_increment_paired_median_nanoseconds: median_of(&write_wait_differences),
        judgement,
    })
}

/// 第二行阶段对照：加 2 ms 忙等之后的增量在 2 ms ± 10% 里。
pub(super) fn stage_control_holds(increment_nanoseconds: f64) -> bool {
    let injected = (INJECTED_MILLISECONDS_PER_WRITE * 1_000_000) as f64;
    relative_change(increment_nanoseconds, injected).abs() <= CONTROL_TOLERANCE
}

/// 第二行甲的对照：算两遍是一遍的 [1.8, 2.2] 倍。
pub(super) fn double_crc_control_holds(twice: f64, once: f64) -> bool {
    (DOUBLE_CRC_RATIO_LOWER..=DOUBLE_CRC_RATIO_UPPER).contains(&(twice / once))
}

/// 第四行对照：走页缓存不同步的带宽 ≥ `O_DIRECT` 带同步的 1.15 倍。
pub(super) fn page_cache_control_holds(
    page_cache_bytes_per_second: f64,
    direct_bytes_per_second: f64,
) -> bool {
    page_cache_bytes_per_second >= PAGE_CACHE_SPEEDUP_AT_LEAST * direct_bytes_per_second
}

#[cfg(test)]
mod tests;
