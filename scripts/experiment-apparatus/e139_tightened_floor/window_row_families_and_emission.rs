//! 行名表与行的家族、窗口行的草稿与记录、打印与按世界或活化取出的字段。

use super::*;

/// 第六节第 6 行里「注入根槽写失败」的六类世界。
pub(super) const WORLD_CLASSES_INJECTING_ROOT_WRITE_FAILURES: [&str; 6] =
    ["slot_write_fail", "activation_faults", "activation_crash_no_loss", "skip_then_raise", "drain_failure", "broken_slot"];
pub(super) const NEAR_FULL_CLASS: &str = "near_full";
pub(super) const DRAIN_FAILURE_CLASS: &str = "drain_failure";
pub(super) const BROKEN_SLOT_CLASS: &str = "broken_slot";
pub(super) const COST_GROWTH_CLASS: &str = "cost_growth";
pub(super) const DISK_STUCK_CLASS: &str = "disk_stuck";
pub(super) const DISK_STUCK_REVERSED_CLASS: &str = "disk_stuck_h3";
/// 一族产物行的行名：第 1 次重跑的 `w_` 行与第 2 次重跑的 `f_` 行同形，只换名字（登记第五节「CF」：行名把 `w_` 换成 `f_`）。
/// 同一类世界里的臂各行逐字比时按这些行名认行（不含几何点后缀）。
pub(super) struct LineNames {
    pub(super) world: &'static str,
    pub(super) activation: &'static str,
    pub(super) near_full: &'static str,
    pub(super) drain_failure: &'static str,
    pub(super) broken_slot: &'static str,
    pub(super) cost_growth: &'static str,
    pub(super) activation_reversed: &'static str,
    pub(super) broken_slot_reversed: &'static str,
    pub(super) crit1_push: &'static str,
    pub(super) trajectory: &'static str,
    pub(super) detail_activation: &'static str,
    pub(super) detail_drain: &'static str,
    pub(super) readings_diff_row: &'static str,
    pub(super) verdict_window: &'static str,
    /// 只有第 2 次重跑有的行；第 1 次的 `w_` 一族不出这几类，留空。
    pub(super) disk_stuck: &'static str,
    pub(super) disk_stuck_reversed: &'static str,
    pub(super) stuck_growth: &'static str,
    pub(super) broken_slot_growth: &'static str,
    pub(super) broken_slot_reversed_growth: &'static str,
    /// 第 2 次重跑起有的行（第 1 次的 `w_` 一族不出，留空）：每行一行归因、卡住段每个种子一行刻度。
    attribution: &'static str,
    pub(super) stuck_marks: &'static str,
    /// 只有第 3 次重跑的 CG 一族有：每行一行 CG 走到（第六节第 4 行）；别的族留空、不打。
    pub(super) core_gate_reach: &'static str,
}

const FIRST_RERUN_LINES: LineNames = LineNames {
    world: "w_cell",
    activation: "w_activation",
    near_full: "w_near_full",
    drain_failure: "w_drain_failure",
    broken_slot: "w_broken_slot",
    cost_growth: "w_cost_growth",
    activation_reversed: "w_activation_h2",
    broken_slot_reversed: "w_broken_slot_h1",
    crit1_push: "w_crit1_push",
    trajectory: "w_trajectory",
    detail_activation: "w_detail_activation",
    detail_drain: "w_detail_drain",
    readings_diff_row: "w_readings_diff_row",
    verdict_window: "verdict_window",
    disk_stuck: "",
    disk_stuck_reversed: "",
    stuck_growth: "",
    broken_slot_growth: "",
    broken_slot_reversed_growth: "",
    attribution: "",
    stuck_marks: "",
    core_gate_reach: "",
};

pub(super) const SECOND_RERUN_LINES: LineNames = LineNames {
    world: "f_cell",
    activation: "f_activation",
    near_full: "f_near_full",
    drain_failure: "f_drain_failure",
    broken_slot: "f_broken_slot",
    cost_growth: "f_cost_growth",
    activation_reversed: "f_activation_h2",
    broken_slot_reversed: "f_broken_slot_h1",
    crit1_push: "f_crit1_push",
    trajectory: "f_trajectory",
    detail_activation: "f_detail_activation",
    detail_drain: "f_detail_drain",
    readings_diff_row: "f_readings_diff_row",
    verdict_window: "verdict_f_window",
    disk_stuck: "f_disk_stuck",
    disk_stuck_reversed: "f_disk_stuck_h3",
    stuck_growth: "f_stuck_growth",
    broken_slot_growth: "f_broken_slot_growth",
    broken_slot_reversed_growth: "f_broken_slot_h1_growth",
    attribution: "f_attribution",
    stuck_marks: "f_stuck_marks",
    core_gate_reach: "",
};

/// 第 3 次重跑的 CG 一族（登记第五节「臂」）：行名把 `f_` 换成 `core_gate_`，格式与 `f_` 行逐字同形。
pub(super) const CORE_GATE_LINES: LineNames = LineNames {
    world: "core_gate_cell",
    activation: "core_gate_activation",
    near_full: "core_gate_near_full",
    drain_failure: "core_gate_drain_failure",
    broken_slot: "core_gate_broken_slot",
    cost_growth: "core_gate_cost_growth",
    activation_reversed: "core_gate_activation_h2",
    broken_slot_reversed: "core_gate_broken_slot_h1",
    crit1_push: "core_gate_crit1_push",
    trajectory: "core_gate_trajectory",
    detail_activation: "core_gate_detail_activation",
    detail_drain: "core_gate_detail_drain",
    readings_diff_row: "core_gate_readings_diff_row",
    verdict_window: "verdict_core_gate_window",
    disk_stuck: "core_gate_disk_stuck",
    disk_stuck_reversed: "core_gate_disk_stuck_h3",
    stuck_growth: "core_gate_stuck_growth",
    broken_slot_growth: "core_gate_broken_slot_growth",
    broken_slot_reversed_growth: "core_gate_broken_slot_h1_growth",
    attribution: "core_gate_attribution",
    stuck_marks: "core_gate_stuck_marks",
    core_gate_reach: "core_gate_reach",
};

/// 一族臂：进全部格的臂、只进 steady 的阳性对照，与几条对照关系里的角色（自证对照、两条读法臂）。
pub(super) struct RowFamily {
    pub(super) lines: &'static LineNames,
    pub(super) arms: Vec<Arm>,
    pub(super) short_arms: Vec<Arm>,
    pub(super) window_arm: Arm,
    pub(super) ring_gate_arm: Arm,
    pub(super) disk_hold_arm: Arm,
    pub(super) ring_lag_arm: Arm,
    /// 第 2 次重跑一族：多打 `f_attribution` 行与 `disk_stuck`、刻度。
    pub(super) is_second_rerun: bool,
    /// `disk_stuck` 跑哪几条臂（第 1 次一族不跑 `disk_stuck`，空）。
    pub(super) disk_stuck_arms: Vec<Arm>,
    /// 阳性对照臂进哪几个基本世界（S ∈ {4, 16}）；第 1、2 次只进 steady。
    pub(super) short_arm_worlds: Vec<World>,
    /// 阳性对照臂也进原取样点的 activation_faults（第 3 次重跑第五节）。
    pub(super) short_arms_enter_activation_faults: bool,
}

/// 第 1 次重跑的一族：窗口臂四条加三条阳性对照，候选下界取旧式。
pub(super) fn first_rerun_family() -> RowFamily {
    RowFamily {
        lines: &FIRST_RERUN_LINES,
        arms: WINDOW_ARMS.iter().map(|window_arm| Arm::TightenedStatesWindowed(*window_arm)).collect(),
        short_arms: WINDOW_SHORT_ARMS.iter().map(|window_arm| Arm::TightenedStatesWindowed(*window_arm)).collect(),
        window_arm: Arm::TightenedStatesWindowed(WindowArm::Window),
        ring_gate_arm: Arm::TightenedStatesWindowed(WindowArm::RingGate),
        disk_hold_arm: Arm::TightenedStatesWindowed(WindowArm::DiskHold),
        ring_lag_arm: Arm::TightenedStatesWindowed(WindowArm::RingLag),
        is_second_rerun: false,
        disk_stuck_arms: Vec::new(),
        short_arm_worlds: vec![World::Steady],
        short_arms_enter_activation_faults: false,
    }
}

/// 第 2 次重跑的一族：同一批窗口臂候选下界取条款形，再加 RGall 两条被测臂与它们的阳性对照。
pub(super) fn second_rerun_family() -> RowFamily {
    let mut arms: Vec<Arm> = WINDOW_ARMS.iter().map(|window_arm| Arm::ClauseFloorWindowed(*window_arm)).collect();
    arms.extend(WHOLE_RING_ARMS.iter().map(|whole_ring_arm| Arm::WholeRing(*whole_ring_arm)));
    let mut short_arms: Vec<Arm> = WINDOW_SHORT_ARMS.iter().map(|window_arm| Arm::ClauseFloorWindowed(*window_arm)).collect();
    short_arms.extend(WHOLE_RING_SHORT_ARMS.iter().map(|whole_ring_arm| Arm::WholeRing(*whole_ring_arm)));
    RowFamily {
        lines: &SECOND_RERUN_LINES,
        arms,
        short_arms,
        window_arm: Arm::ClauseFloorWindowed(WindowArm::Window),
        ring_gate_arm: Arm::ClauseFloorWindowed(WindowArm::RingGate),
        disk_hold_arm: Arm::ClauseFloorWindowed(WindowArm::DiskHold),
        ring_lag_arm: Arm::ClauseFloorWindowed(WindowArm::RingLag),
        is_second_rerun: true,
        disk_stuck_arms: vec![
            Arm::WholeRing(WholeRingArm::SkipsRootlessDisks),
            Arm::WholeRing(WholeRingArm::CountsRootlessDisksAsZero),
            Arm::ClauseFloorWindowed(WindowArm::Window),
            Arm::ClauseFloorWindowed(WindowArm::RingGate),
        ],
        short_arm_worlds: vec![World::Steady],
        short_arms_enter_activation_faults: false,
    }
}

/// 第 3 次重跑的 CG 一族：两条 CG 被测臂进 RGall 一族进的每一格（含 `disk_stuck`），两条阳性对照进 steady、torn_newest、activation_faults。
/// 窗口一族的角色（自证对照、读法臂）这一族不用，填 CG 被测臂占位；这一族不调窗口一族的判决函数。
pub(super) fn core_gate_family() -> RowFamily {
    let tested_arm = Arm::WholeRing(WholeRingArm::CoreGateCeilingSkipsRootlessDisks);
    RowFamily {
        lines: &CORE_GATE_LINES,
        arms: CORE_GATE_ARMS.iter().map(|whole_ring_arm| Arm::WholeRing(*whole_ring_arm)).collect(),
        short_arms: CORE_GATE_SHORT_ARMS.iter().map(|whole_ring_arm| Arm::WholeRing(*whole_ring_arm)).collect(),
        window_arm: tested_arm,
        ring_gate_arm: tested_arm,
        disk_hold_arm: tested_arm,
        ring_lag_arm: tested_arm,
        is_second_rerun: true,
        disk_stuck_arms: CORE_GATE_ARMS.iter().map(|whole_ring_arm| Arm::WholeRing(*whole_ring_arm)).collect(),
        short_arm_worlds: vec![World::Steady, World::TornNewest],
        short_arms_enter_activation_faults: true,
    }
}

pub(super) fn activation_class(variant: ActivationVariant) -> &'static str {
    match variant {
        ActivationVariant::Plain => "activation_faults",
        ActivationVariant::CrashWithoutLossFirst => "activation_crash_no_loss",
        ActivationVariant::SkipThenRaise { .. } => "skip_then_raise",
    }
}

/// 第六节第 8–10 行三个量各自「是否 > 0」；假性 ENOSPC 只在判的格里算进来，不判的格这一位恒 false（数在行里另报）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct RowJudgement {
    pub(super) has_fake_candidates: bool,
    pub(super) has_unrecoverable: bool,
    pub(super) has_false_enospc_in_a_judged_row: bool,
}

impl RowJudgement {
    pub(super) fn of(metrics: &Metrics, is_false_enospc_judged: bool) -> Self {
        RowJudgement {
            has_fake_candidates: metrics.fake_candidates > 0,
            has_unrecoverable: metrics.unrecoverable > 0,
            has_false_enospc_in_a_judged_row: is_false_enospc_judged && metrics.false_enospc_arm > 0,
        }
    }
}

/// 产物里的一行（一个世界、一条臂、一组参数；种子在行内合计）。
pub(super) struct WindowRow {
    pub(super) line_name: &'static str,
    pub(super) world_class: String,
    /// 行键：世界 / 变体、S、开销对、丢盘、失败组合，不含臂。
    pub(super) row_key: String,
    pub(super) arm: Arm,
    /// 整行去掉臂字段之后的文本：第六节第 7 行 G2SW 与 G2SW_ringgate 逐字比它。
    pub(super) text_without_arm: String,
    pub(super) judgement: RowJudgement,
    pub(super) is_report_only_world: bool,
    pub(super) gate_difference_positive_samples: u64,
    /// 第 2 次重跑逐行比、归因与代价汇总用：这一行（种子合计之后）的全部计数。
    pub(super) metrics: Metrics,
    /// 这类行的结果字段里有没有 `false_enospc_arm`（活化一族的行没有，逐行比时不比这一项）。
    pub(super) has_false_enospc_field: bool,
}

pub(super) struct WindowRowDraft<'metrics> {
    pub(super) line_name: &'static str,
    pub(super) world_class: &'metrics str,
    pub(super) row_key: String,
    pub(super) arm: Arm,
    pub(super) outcome_fields: String,
    pub(super) judgement: RowJudgement,
    pub(super) is_report_only_world: bool,
    pub(super) has_false_enospc_field: bool,
    pub(super) metrics: &'metrics Metrics,
}

thread_local! {
    /// 第 3 次重跑逐行比用：开着时 `print_and_record` 打的每一行另记一份（第 2 次一族与 CG 一族各记一段）。只在 `main` 与单测里开关。
    static RECORDED_LINES: std::cell::RefCell<Option<Vec<String>>> = const { std::cell::RefCell::new(None) };
}

pub(super) fn start_recording_lines() {
    RECORDED_LINES.with(|recorded| *recorded.borrow_mut() = Some(Vec::new()));
}

pub(super) fn stop_recording_lines() -> Vec<String> {
    RECORDED_LINES.with(|recorded| recorded.borrow_mut().take().expect("先 start_recording_lines 再 stop"))
}

/// 照原样打印一行；记录开着时另记一份。只改不改输出：打出去的字节与直接 `println!` 相同。
pub(super) fn print_and_record(line: String) {
    println!("{line}");
    RECORDED_LINES.with(|recorded| {
        if let Some(lines) = recorded.borrow_mut().as_mut() {
            lines.push(line);
        }
    });
}

/// 打印一行结果与它的轨迹行（第 2 次重跑起的一族再打一行归因，CG 一族再打一行 CG 走到），并记进行簿。
pub(super) fn emit_window_row(emitter: &mut Emitter, rows: &mut Vec<WindowRow>, family: &RowFamily, draft: WindowRowDraft) {
    print_and_record(emitter.emit_raw(&format!("name={} arm={} {} {}", draft.line_name, draft.arm.name(), draft.row_key, draft.outcome_fields)));
    let new_costs = if family.is_second_rerun { format!(" {}", draft.metrics.second_rerun.render_costs()) } else { String::new() };
    print_and_record(emitter.emit_raw(&format!(
        "name={} line={} arm={} {} {}{new_costs}", family.lines.trajectory, draft.line_name, draft.arm.name(), draft.row_key, draft.metrics.trajectory.render()
    )));
    if family.is_second_rerun {
        print_and_record(emitter.emit_raw(&format!(
            "name={} line={} arm={} {} {}", family.lines.attribution, draft.line_name, draft.arm.name(), draft.row_key, draft.metrics.second_rerun.render_attribution()
        )));
    }
    if !family.lines.core_gate_reach.is_empty() {
        print_and_record(emitter.emit_raw(&format!(
            "name={} line={} arm={} {} {}", family.lines.core_gate_reach, draft.line_name, draft.arm.name(), draft.row_key, draft.metrics.core_gate.render()
        )));
    }
    rows.push(WindowRow {
        line_name: draft.line_name,
        world_class: draft.world_class.to_string(),
        text_without_arm: format!("name={} {} {}", draft.line_name, draft.row_key, draft.outcome_fields),
        row_key: draft.row_key,
        arm: draft.arm,
        judgement: draft.judgement,
        is_report_only_world: draft.is_report_only_world,
        gate_difference_positive_samples: draft.metrics.trajectory.gate_difference.positive_samples,
        metrics: *draft.metrics,
        has_false_enospc_field: draft.has_false_enospc_field,
    });
}

pub(super) fn world_outcome_fields(total: &Metrics) -> String {
    format!(
        "fakes={} unrecoverable={} retention_violations={} checks={} min_candidates={} avg_candidates_x100={} avg_nonempty_x100={} min_states={} \
         avg_states_x100={} stalls={} false_enospc_arm={}",
        total.fake_candidates, total.unrecoverable, total.retention_violations, total.checks, total.minimum_candidates_or_zero(),
        total.average_candidates_in_hundredths(), total.average_nonempty_candidates_in_hundredths(), total.minimum_distinct_states_or_zero(),
        total.average_distinct_states_in_hundredths(), total.stalls, total.false_enospc_arm
    )
}

pub(super) fn activation_outcome_fields(total: &Metrics) -> String {
    format!("fakes={} unrecoverable={} retention_violations={} checks={}", total.fake_candidates, total.unrecoverable, total.retention_violations, total.checks)
}

pub(super) fn fresh_total_over_seeds(seed_count: u64, mut run_seed: impl FnMut(u64) -> Metrics) -> Metrics {
    let mut total = Metrics::fresh();
    for seed in 0..seed_count {
        total.add(&run_seed(seed));
    }
    total
}
