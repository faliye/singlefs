use super::*;

// ───────────────────────────── 第一段 wall-clock：汇总（Q1–Q4、Q14–Q16、对照、轨迹、几何敏感性、判别力自证） ─────────────────────────────

/// 某一格（臂、取样点、K、注入）的全部有效意图，按轮排。
fn intents_of(
    stored: &[StoredIntent],
    arm: DeviceArm,
    point: SamplingPoint,
    batch_units: u64,
    injection: WriteInjection,
) -> Vec<&StoredIntent> {
    let mut intents: Vec<&StoredIntent> = stored
        .iter()
        .filter(|intent| {
            intent.arm == arm
                && intent.point == point
                && intent.batch_units == batch_units
                && intent.injection == injection
        })
        .collect();
    intents.sort_by_key(|intent| intent.round);
    intents
}

/// 同一轮号的两份读数配对（内存臂与真盘臂、K 与 2K）。
fn paired_by_round<'stored>(
    left: &[&'stored StoredIntent],
    right: &[&'stored StoredIntent],
) -> Vec<(&'stored StoredIntent, &'stored StoredIntent)> {
    left.iter()
        .filter_map(|left_intent| {
            right
                .iter()
                .find(|right_intent| right_intent.round == left_intent.round)
                .map(|right_intent| (*left_intent, *right_intent))
        })
        .collect()
}

/// Q1 每轮的 ρ：真盘臂一批 CPU ÷ 一批等盘（批行之和）。
fn rho_per_round(real: &[&StoredIntent]) -> Vec<f64> {
    real.iter()
        .map(|intent| intent.batch_cpu_nanoseconds() / intent.batch_wait_nanoseconds())
        .collect()
}

/// Q2 每轮的挂钟比：同一轮号真盘臂 ÷ 内存臂（批行挂钟之和）。
fn intent_wall_ratio_per_round(pairs: &[(&StoredIntent, &StoredIntent)]) -> Vec<f64> {
    pairs
        .iter()
        .map(|(memory_intent, real_intent)| {
            real_intent.batch_wall_nanoseconds() / memory_intent.batch_wall_nanoseconds()
        })
        .collect()
}

/// Q3 每轮的对账：真盘臂一批 CPU 对内存臂一批挂钟（[`cpu_reconciliation`]）。
fn cpu_reconciliation_per_round(pairs: &[(&StoredIntent, &StoredIntent)]) -> Vec<f64> {
    pairs
        .iter()
        .map(|(memory_intent, real_intent)| {
            cpu_reconciliation(
                real_intent.batch_cpu_nanoseconds(),
                memory_intent.batch_wall_nanoseconds(),
            )
        })
        .collect()
}

/// 一格（f、P、K）Q1、Q2、Q3 的五轮判定（原登记的判定函数与门槛），给 N14 两种作废规则各算一遍。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct WallClockCellJudgements {
    pub(super) rounds_on_the_real_disk: usize,
    pub(super) paired_rounds: usize,
    pub(super) rho: &'static str,
    pub(super) wall_ratio: &'static str,
    pub(super) reconciliation: &'static str,
}

pub(super) fn wall_clock_cell_judgements(
    stored: &[StoredIntent],
    point: SamplingPoint,
    batch_units: u64,
) -> WallClockCellJudgements {
    let none = WriteInjection::Nothing;
    let memory = intents_of(stored, DeviceArm::Memory, point, batch_units, none);
    let real = intents_of(stored, DeviceArm::RealDisk, point, batch_units, none);
    let pairs = paired_by_round(&memory, &real);
    let judgement_of = |values: Vec<f64>, judge: fn(f64) -> &'static str| {
        let per_round: Vec<&'static str> = values.into_iter().map(judge).collect();
        judgement_of_the_rounds(&per_round)
    };
    WallClockCellJudgements {
        rounds_on_the_real_disk: real.len(),
        paired_rounds: pairs.len(),
        rho: judgement_of(
            rho_per_round(&real),
            judgement_of_rho_at_the_registered_edges,
        ),
        wall_ratio: judgement_of(
            intent_wall_ratio_per_round(&pairs),
            judgement_of_wall_ratio_at_the_registered_edges,
        ),
        reconciliation: judgement_of(
            cpu_reconciliation_per_round(&pairs),
            judgement_of_the_cpu_reconciliation,
        ),
    }
}

/// N14（附带，只报不判）：每格一行，照原作废规则（不加 R2-1）收下的那几轮算的三个判定，并列这一趟正式的判定。
pub(super) fn emit_the_wall_clock_judgements_under_the_original_voiding(
    emitter: &mut Emitter,
    stored: &StoredIntents,
    points: &[SamplingPoint],
) {
    for point in points {
        for batch_units in BATCH_SIZES {
            let original =
                wall_clock_cell_judgements(&stored.under_the_original_voiding, *point, batch_units);
            let registered = wall_clock_cell_judgements(
                &stored.under_the_registered_voiding,
                *point,
                batch_units,
            );
            emitter.line(&format!(
                "name=judgements_under_the_original_voiding segment=wall_clock {} batch_units={batch_units} rounds_on_the_real_disk={} paired_rounds={} rho_judgement={} wall_ratio_judgement={} reconciliation_judgement={} registered_rounds_on_the_real_disk={} registered_paired_rounds={} registered_rho_judgement={} registered_wall_ratio_judgement={} registered_reconciliation_judgement={} differs_from_the_registered={}",
                point.label(),
                original.rounds_on_the_real_disk,
                original.paired_rounds,
                original.rho,
                original.wall_ratio,
                original.reconciliation,
                registered.rounds_on_the_real_disk,
                registered.paired_rounds,
                registered.rho,
                registered.wall_ratio,
                registered.reconciliation,
                (original.rho, original.wall_ratio, original.reconciliation)
                    != (registered.rho, registered.wall_ratio, registered.reconciliation),
            ));
        }
    }
}

fn list_of(values: &[f64]) -> String {
    if values.is_empty() {
        return "none".to_string();
    }
    values
        .iter()
        .map(|value| format!("{value:.6}"))
        .collect::<Vec<_>>()
        .join(",")
}

pub(super) fn optional_number(value: Option<f64>) -> String {
    value.map_or_else(|| "none".to_string(), |number| format!("{number:.6}"))
}

/// 取样点的键：（活槽占比、P、K）。
type SamplingPointKey = (LiveSlotFraction, u64, u64);

/// 一个旋钮取另一端时的那个取样点；这一点本来就在另一端、或没有对应点时交回 `None`。
type SamplingPointAtTheOtherEndOfTheKnob = fn(&SamplingPointKey) -> Option<SamplingPointKey>;

/// 一个量在一个旋钮两端的翻面（登记第八节「几何敏感性」）：键是（活槽占比、P、K），旋钮是三者之一；只比另两样相同、这一样取两端的那几对。
fn emit_the_sensitivity(
    emitter: &mut Emitter,
    quantity: &str,
    judgements: &BTreeMap<SamplingPointKey, &'static str>,
) {
    let knobs: [(&str, SamplingPointAtTheOtherEndOfTheKnob); 3] = [
        ("fraction", |key| {
            (key.0 == LiveSlotFraction::OneQuarter).then_some((
                LiveSlotFraction::ThreeQuarters,
                key.1,
                key.2,
            ))
        }),
        ("pool_data_units", |key| {
            (key.1 == POOL_DATA_UNITS[0]).then_some((key.0, POOL_DATA_UNITS[1], key.2))
        }),
        ("batch_units", |key| {
            (key.2 == BATCH_SIZES[0]).then_some((key.0, key.1, BATCH_SIZES[2]))
        }),
    ];
    for (knob, partner_of) in knobs {
        let mut pairs = 0_usize;
        let mut flips = Vec::new();
        for (key, judgement) in judgements {
            let Some(partner) = partner_of(key) else {
                continue;
            };
            let Some(partner_judgement) = judgements.get(&partner) else {
                continue;
            };
            pairs += 1;
            if judgement != partner_judgement {
                flips.push(format!(
                    "{}:{}:{}:{judgement}->{}:{}:{}:{partner_judgement}",
                    key.0.label(),
                    key.1,
                    key.2,
                    partner.0.label(),
                    partner.1,
                    partner.2
                ));
            }
        }
        emitter.line(&format!(
            "name=sensitivity quantity={quantity} knob={knob} pairs={pairs} flips={} statement={} flipped_pairs={}",
            flips.len(),
            if flips.is_empty() { "does_not_flip_at_these_points" } else { "flips_with_this_knob" },
            if flips.is_empty() { "none".to_string() } else { flips.join(",") }
        ));
    }
}

/// 判别力自证（产物里那一份）：量到的最低、最高两点之间挪门槛，两点判定必须不同。
pub(super) fn emit_the_discrimination(
    emitter: &mut Emitter,
    quantity: &str,
    values: &[f64],
    judge: impl Fn(f64, f64) -> &'static str,
) -> bool {
    let lowest = values.iter().copied().reduce(f64::min);
    let highest = values.iter().copied().reduce(f64::max);
    match (lowest, highest) {
        (Some(lower_value), Some(higher_value)) if lower_value < higher_value => {
            let (moved, lower_judgement, higher_judgement) =
                judgements_with_the_threshold_moved_between(lower_value, higher_value, judge);
            let holds = lower_judgement != higher_judgement;
            emitter.line(&format!(
                "name=discrimination quantity={quantity} lower_value={lower_value:.6} higher_value={higher_value:.6} moved_threshold={moved:.6} lower_judgement={lower_judgement} higher_judgement={higher_judgement} holds={holds}"
            ));
            holds
        }
        (Some(_), Some(_)) | (None, _) | (Some(_), None) => {
            emitter.line(&format!(
                "name=discrimination quantity={quantity} holds=not_run reason=fewer_than_two_distinct_values"
            ));
            true
        }
    }
}

/// 第一段汇总交回给判决行的几样。
pub(super) struct WallClockSummary {
    pub(super) positive_control_failures: Vec<String>,
    pub(super) gc_to_sequential_write_bandwidth: Option<(f64, &'static str)>,
    pub(super) discrimination_failures: usize,
    pub(super) clause_wall_clock_not_rising_with_the_batch_triggered: usize,
    pub(super) clause_two_cpu_paths_disagree_points: usize,
}

#[allow(
    clippy::too_many_lines,
    reason = "一段汇总按登记第六节的量逐个报（Q1–Q4、Q14–Q16、两行对照、轨迹、敏感性、判别力自证），拆开之后每一段都要同一组取样点与读数"
)]
pub(super) fn summarize_the_wall_clock_segment(
    emitter: &mut Emitter,
    stored: &[StoredIntent],
    points: &[SamplingPoint],
    sequential_write: Option<(f64, f64)>,
) -> WallClockSummary {
    let none = WriteInjection::Nothing;
    let mut summary = WallClockSummary {
        positive_control_failures: Vec::new(),
        gc_to_sequential_write_bandwidth: None,
        discrimination_failures: 0,
        clause_wall_clock_not_rising_with_the_batch_triggered: 0,
        clause_two_cpu_paths_disagree_points: 0,
    };
    let mut rho_judgements_by_point = BTreeMap::new();
    let mut wall_ratio_judgements_by_point = BTreeMap::new();
    let mut bandwidth_ratio_judgements_by_point = BTreeMap::new();
    let mut rho_medians = Vec::new();
    let mut wall_ratio_medians = Vec::new();
    let mut gc_write_bandwidth_medians: Vec<(SamplingPoint, u64, f64)> = Vec::new();
    for point in points {
        for batch_units in BATCH_SIZES {
            let memory = intents_of(stored, DeviceArm::Memory, *point, batch_units, none);
            let real = intents_of(stored, DeviceArm::RealDisk, *point, batch_units, none);
            let formed = real
                .first()
                .or(memory.first())
                .is_some_and(|intent| intent.is_the_geometry_formed);
            let rhos = rho_per_round(&real);
            let rho_judgements: Vec<&'static str> = rhos
                .iter()
                .map(|rho| judgement_of_rho_at_the_registered_edges(*rho))
                .collect();
            let rho_judgement = judgement_of_the_rounds(&rho_judgements);
            emitter.line(&format!(
                "name=q1 {} batch_units={batch_units} geometry_formed={formed} rounds={} rho_per_round={} rho_median={} judgement_per_round={} judgement={rho_judgement}",
                point.label(),
                rhos.len(),
                list_of(&rhos),
                optional_number(median_of(&rhos)),
                rho_judgements.join(","),
            ));
            rho_judgements_by_point.insert(
                (point.fraction, point.pool_data_units, batch_units),
                rho_judgement,
            );
            if let Some(median) = median_of(&rhos) {
                rho_medians.push(median);
            }
            let pairs = paired_by_round(&memory, &real);
            let wall_ratios = intent_wall_ratio_per_round(&pairs);
            let batch_count = pairs
                .iter()
                .map(|(memory_intent, real_intent)| {
                    memory_intent
                        .batch_calls()
                        .count()
                        .min(real_intent.batch_calls().count())
                })
                .min()
                .unwrap_or(0);
            let per_batch_median_ratios: Vec<f64> = (0..batch_count)
                .filter_map(|batch_index| {
                    let ratios: Vec<f64> = pairs
                        .iter()
                        .filter_map(|(memory_intent, real_intent)| {
                            let memory_wall = memory_intent
                                .batch_calls()
                                .nth(batch_index)?
                                .wall_nanoseconds
                                as f64;
                            let real_wall =
                                real_intent.batch_calls().nth(batch_index)?.wall_nanoseconds as f64;
                            Some(real_wall / memory_wall)
                        })
                        .collect();
                    median_of(&ratios)
                })
                .collect();
            let ratio_judgements: Vec<&'static str> = wall_ratios
                .iter()
                .map(|ratio| judgement_of_wall_ratio_at_the_registered_edges(*ratio))
                .collect();
            let wall_ratio_judgement = judgement_of_the_rounds(&ratio_judgements);
            emitter.line(&format!(
                "name=q2 {} batch_units={batch_units} geometry_formed={formed} rounds={} intent_wall_ratio_per_round={} intent_wall_ratio_median={} per_batch_median_ratio={} judgement_per_round={} judgement={wall_ratio_judgement}",
                point.label(),
                wall_ratios.len(),
                list_of(&wall_ratios),
                optional_number(median_of(&wall_ratios)),
                list_of(&per_batch_median_ratios),
                ratio_judgements.join(","),
            ));
            wall_ratio_judgements_by_point.insert(
                (point.fraction, point.pool_data_units, batch_units),
                wall_ratio_judgement,
            );
            if let Some(median) = median_of(&wall_ratios) {
                wall_ratio_medians.push(median);
            }
            let reconciliations = cpu_reconciliation_per_round(&pairs);
            let reconciliation_judgements: Vec<&'static str> = reconciliations
                .iter()
                .map(|value| judgement_of_the_cpu_reconciliation(*value))
                .collect();
            let reconciliation_judgement = judgement_of_the_rounds(&reconciliation_judgements);
            if reconciliation_judgement != "compatible" {
                summary.clause_two_cpu_paths_disagree_points += 1;
            }
            emitter.line(&format!(
                "name=q3 {} batch_units={batch_units} rounds={} reconciliation_per_round={} judgement_per_round={} judgement={reconciliation_judgement} row_one_judgeable_here={}",
                point.label(),
                reconciliations.len(),
                list_of(&reconciliations),
                reconciliation_judgements.join(","),
                reconciliation_judgement == "compatible",
            ));
            let bandwidths: Vec<f64> = real
                .iter()
                .map(|intent| {
                    intent.batch_write_bytes() as f64 / (intent.batch_wall_nanoseconds() / 1e9)
                })
                .collect();
            let relocation_shares: Vec<f64> = real
                .iter()
                .map(|intent| {
                    intent.batch_relocation_copy_bytes() as f64 / intent.batch_write_bytes() as f64
                })
                .collect();
            emitter.line(&format!(
                "name=q14 {} batch_units={batch_units} rounds={} write_bytes_per_second_per_round={} write_bytes_per_second_median={} relocation_copy_share_per_round={} write_bytes_per_round={} relocation_copy_bytes_per_round={}",
                point.label(),
                bandwidths.len(),
                list_of(&bandwidths),
                optional_number(median_of(&bandwidths)),
                list_of(&relocation_shares),
                real.iter().map(|intent| intent.batch_write_bytes().to_string()).collect::<Vec<_>>().join(","),
                real.iter().map(|intent| intent.batch_relocation_copy_bytes().to_string()).collect::<Vec<_>>().join(","),
            ));
            if let Some(median) = median_of(&bandwidths) {
                gc_write_bandwidth_medians.push((*point, batch_units, median));
            }
        }
    }
    emit_the_sensitivity(emitter, "Q1", &rho_judgements_by_point);
    emit_the_sensitivity(emitter, "Q2", &wall_ratio_judgements_by_point);
    if !emit_the_discrimination(emitter, "Q1", &rho_medians, |value, threshold| {
        judgement_of_rho(value, threshold, f64::INFINITY)
    }) {
        summary.discrimination_failures += 1;
    }
    if !emit_the_discrimination(emitter, "Q2", &wall_ratio_medians, |value, threshold| {
        judgement_of_wall_ratio(value, threshold, f64::NEG_INFINITY)
    }) {
        summary.discrimination_failures += 1;
    }
    summarize_the_batch_doubling(emitter, stored, points, &mut summary);
    match sequential_write {
        Some((direct, page_cache)) => {
            let control_holds = page_cache_control_holds(page_cache, direct);
            emitter.line(&format!(
                "name=q15 direct_with_fdatasync_bytes_per_second_median={direct:.1} page_cache_without_sync_bytes_per_second_median={page_cache:.1} positive_control_row_four_holds={control_holds}"
            ));
            if !control_holds {
                summary
                    .positive_control_failures
                    .push("row_four_page_cache".to_string());
            }
            let ratios: Vec<f64> = gc_write_bandwidth_medians
                .iter()
                .map(|(_point, _batch_units, bandwidth)| bandwidth / direct)
                .collect();
            for (point, batch_units, bandwidth) in &gc_write_bandwidth_medians {
                let ratio = bandwidth / direct;
                bandwidth_ratio_judgements_by_point.insert(
                    (point.fraction, point.pool_data_units, *batch_units),
                    judgement_of_bandwidth_ratio_at_the_registered_edge(ratio),
                );
                emitter.line(&format!(
                    "name=q16_point {} batch_units={batch_units} ratio={ratio:.6} judgement={}",
                    point.label(),
                    judgement_of_bandwidth_ratio_at_the_registered_edge(ratio)
                ));
            }
            if let Some(maximum) = ratios.iter().copied().reduce(f64::max) {
                summary.gc_to_sequential_write_bandwidth = Some((
                    maximum,
                    judgement_of_bandwidth_ratio_at_the_registered_edge(maximum),
                ));
                emitter.line(&format!(
                    "name=q16 maximum_ratio={maximum:.6} judgement={} fourth_segment={}",
                    judgement_of_bandwidth_ratio_at_the_registered_edge(maximum),
                    if judgement_of_bandwidth_ratio_at_the_registered_edge(maximum)
                        == "single_run_enough"
                    {
                        "not_run_after_enough_to_judge"
                    } else {
                        "needed"
                    }
                ));
            }
            emit_the_sensitivity(emitter, "Q16", &bandwidth_ratio_judgements_by_point);
            if !emit_the_discrimination(emitter, "Q16", &ratios, judgement_of_bandwidth_ratio) {
                summary.discrimination_failures += 1;
            }
        }
        None => emitter.line("name=q15 valid=false reason=sequential_write_not_measured"),
    }
    summarize_the_row_one_controls(emitter, stored, &mut summary);
    emit_the_trajectories(emitter, stored);
    summary
}

/// Q4（批大小翻倍，C412 检查列）：同一 f、P、设备，第一批挂钟的比 `挂钟(2K) ÷ 挂钟(K)`，只取 2K ≤ m 的点对；不跟着涨而搬迁副本字节锚点
/// 确实翻倍的，记第十条 F1 触发（条款可能错，不作废）。
fn summarize_the_batch_doubling(
    emitter: &mut Emitter,
    stored: &[StoredIntent],
    points: &[SamplingPoint],
    summary: &mut WallClockSummary,
) {
    for arm in DeviceArm::ALL {
        for point in points {
            let data_units = point.fraction.data_units_per_segment();
            for (smaller, larger) in doubling_pairs_within(data_units) {
                let small = intents_of(stored, arm, *point, smaller, WriteInjection::Nothing);
                let large = intents_of(stored, arm, *point, larger, WriteInjection::Nothing);
                let ratios: Vec<f64> = paired_by_round(&small, &large)
                    .iter()
                    .filter_map(|(small_intent, large_intent)| {
                        Some(
                            large_intent.first_batch_wall_nanoseconds()?
                                / small_intent.first_batch_wall_nanoseconds()?,
                        )
                    })
                    .collect();
                let judgement = judgement_of_the_batch_doubling(&ratios);
                let copy_bytes_double = first_batch_copy_bytes(data_units, larger)
                    == 2 * first_batch_copy_bytes(data_units, smaller);
                let is_the_wall_clock_not_rising_clause_triggered =
                    judgement == "does_not_rise" && copy_bytes_double;
                if is_the_wall_clock_not_rising_clause_triggered {
                    summary.clause_wall_clock_not_rising_with_the_batch_triggered += 1;
                }
                emitter.line(&format!(
                    "name=q4 arm={} {} batch_units_pair={smaller}->{larger} rounds={} first_batch_wall_ratio_per_round={} median={} judgement={judgement} copy_bytes_double={copy_bytes_double} clause_wall_clock_not_rising_with_the_batch_triggered={is_the_wall_clock_not_rising_clause_triggered}",
                    arm.label(),
                    point.label(),
                    ratios.len(),
                    list_of(&ratios),
                    optional_number(median_of(&ratios)),
                ));
            }
        }
    }
}

/// 第一行两条臂的阳性对照（登记第五节阳性对照表第 1 行，CPU 臂等盘那一句照第十二节 12.9）：对照点上 K = 8，按整条意图比注入前后的中位；
/// CPU 臂另报写调用的等盘增量、两句各自的判定与原判据那一句（只报不判），每格另报按轮号配对的增量中位（不判）。
fn summarize_the_row_one_controls(
    emitter: &mut Emitter,
    stored: &[StoredIntent],
    summary: &mut WallClockSummary,
) {
    for arm in DeviceArm::ALL {
        let rounds_of = |injection: WriteInjection| -> Vec<RowOneControlRound> {
            intents_of(stored, arm, CONTROL_POINT, CONTROL_BATCH_UNITS, injection)
                .iter()
                .map(|intent| intent.row_one_control_round())
                .collect()
        };
        let baseline = rounds_of(WriteInjection::Nothing);
        for injection in [
            WriteInjection::SleepTwoMilliseconds,
            WriteInjection::SpinTwoMillisecondsOfThreadCpu,
        ] {
            let control = rounds_of(injection);
            let Some(reading) = row_one_control_reading(injection, &baseline, &control) else {
                emitter.line(&format!(
                    "name=positive_control row=1 device={} injection={} holds=not_run reason=missing_rounds baseline_rounds={} control_rounds={}",
                    arm.label(),
                    injection.label(),
                    baseline.len(),
                    control.len()
                ));
                summary.positive_control_failures.push(format!(
                    "row_one_{}_{}_not_run",
                    arm.label(),
                    injection.label()
                ));
                continue;
            };
            let (arm_of_the_question, clause_text) = match reading.judgement {
                RowOneControlJudgement::DiskArm { holds: _ } => ("disk", String::new()),
                RowOneControlJudgement::CpuArm {
                    cpu_clause_holds,
                    write_wait_clause_holds,
                    wait_clause_under_the_original_criterion_holds,
                } => (
                    "cpu",
                    format!(
                        " cpu_increment_over_injected={:.6} cpu_clause_holds={cpu_clause_holds} write_wait_increment_over_injected={:.6} write_wait_clause_holds={write_wait_clause_holds} wait_clause_under_the_original_criterion_holds={wait_clause_under_the_original_criterion_holds} original_criterion_is_reported_only=true",
                        reading.cpu_increment_nanoseconds / reading.injected_nanoseconds,
                        reading.write_wait_increment_nanoseconds / reading.injected_nanoseconds,
                    ),
                ),
            };
            let holds = reading.judgement.holds();
            emitter.line(&format!(
                "name=positive_control row=1 arm_of_the_question={arm_of_the_question} device={} injection={} baseline_rounds={} control_rounds={} write_calls_median={:.1} injected_nanoseconds={:.1} cpu_increment_nanoseconds={:.1} wait_increment_nanoseconds={:.1} write_wait_increment_nanoseconds={:.1} cpu_change={:.6} wait_change={:.6}{clause_text} paired_rounds={} cpu_increment_paired_median_nanoseconds={} write_wait_increment_paired_median_nanoseconds={} holds={holds}",
                arm.label(),
                injection.label(),
                baseline.len(),
                control.len(),
                reading.write_calls,
                reading.injected_nanoseconds,
                reading.cpu_increment_nanoseconds,
                reading.wait_increment_nanoseconds,
                reading.write_wait_increment_nanoseconds,
                reading.cpu_change,
                reading.wait_change,
                reading.paired_rounds,
                optional_number(reading.cpu_increment_paired_median_nanoseconds),
                optional_number(reading.write_wait_increment_paired_median_nanoseconds),
            ));
            if !holds {
                summary.positive_control_failures.push(format!(
                    "row_one_{arm_of_the_question}_arm_on_{}",
                    arm.label()
                ));
            }
        }
    }
}

/// 峰值、为正的个数、期末值（登记第八节「轨迹」：被谓词消费的量报这三样）。
fn trajectory_text(name: &str, values: &[f64]) -> String {
    let peak = values.iter().copied().reduce(f64::max);
    let positive = values.iter().filter(|value| **value > 0.0).count();
    format!(
        "{name}_peak={} {name}_positive_batches={positive} {name}_final={} {name}_per_batch={}",
        optional_number(peak),
        optional_number(values.last().copied()),
        list_of(values)
    )
}

/// 每条有效意图逐批的轨迹：B_批（扫描读到 R 的字节）、每批写带宽、每批 ρ、每批等盘时间。
fn emit_the_trajectories(emitter: &mut Emitter, stored: &[StoredIntent]) {
    for intent in stored
        .iter()
        .filter(|intent| intent.injection == WriteInjection::Nothing)
    {
        let batch_bytes: Vec<f64> = intent
            .batch_calls()
            .map(|call| call.scan_read_bytes_in_the_region as f64)
            .collect();
        let write_bandwidths: Vec<f64> = intent
            .batch_calls()
            .map(|call| call.write_bytes as f64 / (call.wall_nanoseconds as f64 / 1e9))
            .collect();
        let rhos: Vec<f64> = intent
            .batch_calls()
            .map(|call| call.cpu_nanoseconds as f64 / call.wait_nanoseconds() as f64)
            .collect();
        let waits: Vec<f64> = intent
            .batch_calls()
            .map(|call| call.wait_nanoseconds() as f64)
            .collect();
        let rho_at_least_for_cpu = rhos
            .iter()
            .filter(|rho| **rho >= RHO_AT_LEAST_FOR_CPU)
            .count();
        let rho_at_most_for_disk = rhos
            .iter()
            .filter(|rho| **rho <= RHO_AT_MOST_FOR_DISK)
            .count();
        emitter.line(&format!(
            "name=trajectory arm={} {} batch_units={} round={} {} {} {} rho_batches_at_least_two={rho_at_least_for_cpu} rho_batches_at_most_half={rho_at_most_for_disk} {}",
            intent.arm.label(),
            intent.point.label(),
            intent.batch_units,
            intent.round,
            trajectory_text("batch_scan_bytes", &batch_bytes),
            trajectory_text("write_bytes_per_second", &write_bandwidths),
            trajectory_text("rho", &rhos),
            trajectory_text("wait_nanoseconds", &waits),
        ));
    }
}
