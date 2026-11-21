//! 卡盘臂的界、增长汇总、标记与斜率、卡盘段与结局、卡盘行的打印。

use super::*;

/// 一行刻度与斜率的判（登记第六节第 17 行）：8 个种子的 s(2→4) 都小于 1/2 为有上界；有一个不小于就是没有上界；一个种子都没有 s(2→4) 时不判。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum StuckBound {
    Bounded,
    Unbounded,
    NotJudged,
}

impl StuckBound {
    pub(super) fn name(self) -> &'static str {
        match self {
            StuckBound::Bounded => "bounded",
            StuckBound::Unbounded => "unbounded",
            StuckBound::NotJudged => "not_judged",
        }
    }
}

pub(super) struct StuckGrowthSummary {
    pub(super) line_name: &'static str,
    pub(super) arm: Arm,
    pub(super) row_key: String,
    pub(super) bound: StuckBound,
}

/// 每个种子两个刻度之间的（扣住的增量，放掉的块数）；两个刻度没都走到的种子不进。
fn marks_slopes(seed_marks: &[[Option<StuckMark>; 3]], from_index: usize, to_index: usize) -> Vec<(i64, i64)> {
    seed_marks.iter().filter_map(|marks| match (&marks[from_index], &marks[to_index]) {
        (Some(earlier), Some(later)) => stuck_slope_between(earlier, later),
        _ => None,
    }).collect()
}

fn slope_range_in_hundredths(slopes: &[(i64, i64)]) -> (String, String) {
    let hundredths: Vec<i64> = slopes.iter().map(|(increase, freed)| slope_in_hundredths(*increase, *freed)).collect();
    match (hundredths.iter().min(), hundredths.iter().max()) {
        (Some(smallest), Some(largest)) => (smallest.to_string(), largest.to_string()),
        _ => ("na".to_string(), "na".to_string()),
    }
}

fn render_stuck_marks(marks: &[Option<StuckMark>; 3]) -> String {
    STUCK_MARK_REVOLUTIONS.iter().zip(marks.iter()).map(|(revolutions, mark)| match mark {
        Some(mark) => format!("mark{revolutions}_txg={} mark{revolutions}_pinned={} mark{revolutions}_freed={}", mark.published_txg, mark.pinned_blocks, mark.blocks_freed_in_total),
        None => format!("mark{revolutions}_txg=na mark{revolutions}_pinned=na mark{revolutions}_freed=na"),
    }).collect::<Vec<String>>().join(" ")
}

/// 刻度与斜率一行（`f_stuck_growth`、`f_broken_slot_growth`、`f_broken_slot_h1_growth` 同形）：8 个种子合计的刻度、两段斜率的种子最小与最大（乘 100 取整）、没走到的种子数、判。
pub(super) fn emit_stuck_growth_row(emitter: &mut Emitter, line_name: &'static str, arm: Arm, row_key: &str, seed_marks: &[[Option<StuckMark>; 3]]) -> StuckGrowthSummary {
    let mut mark_fields: Vec<String> = Vec::new();
    for (mark_index, revolutions) in STUCK_MARK_REVOLUTIONS.iter().enumerate() {
        let reached: Vec<&StuckMark> = seed_marks.iter().filter_map(|marks| marks[mark_index].as_ref()).collect();
        let pinned_sum: u64 = reached.iter().map(|mark| mark.pinned_blocks).sum();
        mark_fields.push(format!("mark{revolutions}_seeds_reached={} mark{revolutions}_pinned_sum={pinned_sum}", reached.len()));
    }
    let first_to_second = marks_slopes(seed_marks, 0, 1);
    let second_to_fourth = marks_slopes(seed_marks, 1, 2);
    let (first_to_second_smallest, first_to_second_largest) = slope_range_in_hundredths(&first_to_second);
    let (second_to_fourth_smallest, second_to_fourth_largest) = slope_range_in_hundredths(&second_to_fourth);
    let seeds_not_below_one_half = second_to_fourth.iter().filter(|(increase, freed)| !is_stuck_slope_bounded(*increase, *freed)).count();
    let bound = if second_to_fourth.is_empty() {
        StuckBound::NotJudged
    } else if seeds_not_below_one_half == 0 {
        StuckBound::Bounded
    } else {
        StuckBound::Unbounded
    };
    print_and_record(emitter.emit_raw(&format!(
        "name={line_name} arm={} {row_key} seeds={} {} s12_seeds={} s12_min_x100={first_to_second_smallest} s12_max_x100={first_to_second_largest} \
         s24_seeds={} s24_min_x100={second_to_fourth_smallest} s24_max_x100={second_to_fourth_largest} s24_seeds_not_below_one_half={seeds_not_below_one_half} \
         seeds_without_mark4={} bound={}",
        arm.name(), seed_marks.len(), mark_fields.join(" "), first_to_second.len(), second_to_fourth.len(),
        seed_marks.iter().filter(|marks| marks[STUCK_MARK_REVOLUTIONS.len() - 1].is_none()).count(), bound.name()
    )));
    StuckGrowthSummary { line_name, arm, row_key: row_key.to_string(), bound }
}

/// 收尾的三种形态（登记第五节「`disk_stuck`」收尾 e）：只崩不丢盘，或丢盘 0 / 1 之后恢复。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum StuckEnding {
    CrashOnly,
    LostDisk(usize),
}

pub(super) const STUCK_ENDINGS: [StuckEnding; 3] = [StuckEnding::CrashOnly, StuckEnding::LostDisk(0), StuckEnding::LostDisk(1)];

impl StuckEnding {
    pub(super) fn name(self) -> String {
        match self {
            StuckEnding::CrashOnly => "crash".to_string(),
            StuckEnding::LostDisk(disk) => format!("lost_disk_{disk}"),
        }
    }
}

/// `disk_stuck` 的卡住段（登记第五节）：keep_snapshots 开，占用 `fill`，预热 2 × 3S + 10 个删建窗口，计数清零，记 T₀ = `next_txg`；
/// 之后 `stuck_disk` 上的根槽每次写都失败，做删建窗口直到第一个落盘 txg ≥ T₀ + 4 × 3S（第 4 个刻度走到）、停机，或做满 16 × 3S 个窗口。
pub(super) fn disk_stuck_segment(arm: Arm, slots_per_region: u64, stuck_disk: usize, fill: f64, seed: u64) -> Simulation {
    let (mount_cost, declared_maximum_cost) = COST_PAIRS[0];
    let mut simulation = Simulation::new(arm, slots_per_region, mount_cost, declared_maximum_cost, seed, true, None);
    simulation.snapshots_kept_limit = STUCK_SNAPSHOTS_KEPT;
    simulation.prefill(fill);
    let warm_up = 2 * simulation.ring_depth + 10;
    for _ in 0..warm_up {
        simulation.run_user_window(true, false);
    }
    simulation.metrics = Metrics::fresh();
    simulation.counting_window = 0;
    simulation.stuck_disk = Some(stuck_disk);
    simulation.stuck_marks_origin_txg = Some(simulation.next_txg);
    let window_limit = STUCK_WINDOW_REVOLUTION_LIMIT * simulation.ring_depth;
    for _ in 0..window_limit {
        let is_last_mark_taken = simulation.metrics.second_rerun.marks[STUCK_MARK_REVOLUTIONS.len() - 1].is_some();
        if simulation.halted || is_last_mark_taken {
            break;
        }
        simulation.run_counted_delete_create_window();
    }
    simulation
}

/// `disk_stuck` 的收尾：从卡住段的末态拷一份，卡住解除，丢盘（若有）之后崩溃恢复，再做 2 × 3S 个删建窗口。卡住段已经停机的不收尾。
pub(super) fn disk_stuck_ending(stuck_end: &Simulation, ending: StuckEnding) -> Metrics {
    let mut simulation = stuck_end.clone();
    simulation.stuck_disk = None;
    simulation.stuck_marks_origin_txg = None;
    if !simulation.halted {
        if let StuckEnding::LostDisk(disk) = ending {
            simulation.disk_lost[disk] = true;
        }
        simulation.crash_and_recover();
        let tail_windows = STUCK_TAIL_REVOLUTIONS * simulation.ring_depth;
        for _ in 0..tail_windows {
            if simulation.halted {
                break;
            }
            simulation.run_counted_delete_create_window();
        }
    }
    simulation.metrics
}

fn stuck_outcome_fields(total: &Metrics) -> String {
    format!(
        "seeds={FAULT_SEED_COUNT} fakes={} unrecoverable={} false_enospc_arm={} true_enospc={} write_after_delete_failed={} stalls={} injected={}",
        total.fake_candidates, total.unrecoverable, total.false_enospc_arm, total.true_enospc, total.write_after_delete_failed, total.stalls, total.root_write_failures_injected
    )
}

/// `disk_stuck`（登记第五节，第 2 行第三样代价的目标形态）：被测 RGall、RGall_Q 与对照 G2SW、G2SW_ringgate，全部开条款形候选下界。
pub(super) fn emit_disk_stuck_rows(emitter: &mut Emitter, rows: &mut Vec<WindowRow>, growth: &mut Vec<StuckGrowthSummary>, family: &RowFamily) {
    let stuck_arms = family.disk_stuck_arms.clone();
    for stuck_disk in STUCK_DISKS {
        let (line_name, world_class) = if stuck_disk == 1 { (family.lines.disk_stuck, DISK_STUCK_CLASS) } else { (family.lines.disk_stuck_reversed, DISK_STUCK_REVERSED_CLASS) };
        for slots_per_region in FAULT_SLOTS_SWEEP {
            for arm in stuck_arms.iter().copied() {
                for fill in STUCK_FILLS {
                    let growth_key = format!("s={slots_per_region} stuck_disk={stuck_disk} fill={fill:.2}");
                    let mut totals_by_ending = [Metrics::fresh(); 3];
                    let mut seed_marks: Vec<[Option<StuckMark>; 3]> = Vec::new();
                    for seed in 0..FAULT_SEED_COUNT {
                        let stuck_end = disk_stuck_segment(arm, slots_per_region, stuck_disk, fill, seed);
                        let marks = stuck_end.metrics.second_rerun.marks;
                        seed_marks.push(marks);
                        let slope_text = |from_index: usize, to_index: usize| match marks_slopes(&[marks], from_index, to_index).first() {
                            Some((increase, freed)) => slope_in_hundredths(*increase, *freed).to_string(),
                            None => "na".to_string(),
                        };
                        print_and_record(emitter.emit_raw(&format!(
                            "name={} line={line_name} arm={} {growth_key} seed={seed} {} s12_x100={} s24_x100={}",
                            family.lines.stuck_marks, arm.name(), render_stuck_marks(&marks), slope_text(0, 1), slope_text(1, 2)
                        )));
                        for (ending_index, ending) in STUCK_ENDINGS.iter().enumerate() {
                            totals_by_ending[ending_index].add(&disk_stuck_ending(&stuck_end, *ending));
                        }
                    }
                    growth.push(emit_stuck_growth_row(emitter, family.lines.stuck_growth, arm, &growth_key, &seed_marks));
                    for (ending_index, ending) in STUCK_ENDINGS.iter().enumerate() {
                        let total = &totals_by_ending[ending_index];
                        emit_window_row(emitter, rows, family, WindowRowDraft {
                            line_name,
                            world_class,
                            row_key: format!("s={slots_per_region} fill={fill:.2} ending={}", ending.name()),
                            arm,
                            outcome_fields: stuck_outcome_fields(total),
                            judgement: RowJudgement::of(total, false),
                            is_report_only_world: false,
                            has_false_enospc_field: true,
                            metrics: total,
                        });
                    }
                }
            }
        }
    }
}
