//! 行的分类与跨臂对照、到达与方向与非零行的判定、代价与增长与卡盘与过期的判定行。

use super::*;

pub(super) fn line_suffix(line_name: &str) -> &str {
    line_name.get(2..).unwrap_or(line_name)
}

fn judgement_text(judgement: &RowJudgement) -> String {
    format!(
        "fake{}_unrecoverable{}_false_enospc{}",
        u8::from(judgement.has_fake_candidates), u8::from(judgement.has_unrecoverable), u8::from(judgement.has_false_enospc_in_a_judged_row)
    )
}

fn observed_or_na(value: i64) -> String {
    if value == i64::MIN { "na".to_string() } else { value.to_string() }
}

/// 行键里去掉某一个扫过的字段（按前缀认），剩下的拿来在两个取样点之间配行。
pub(super) fn row_key_without_the_field(row_key: &str, field_prefix: &str) -> String {
    row_key.split(' ').filter(|token| !token.starts_with(field_prefix)).collect::<Vec<&str>>().join(" ")
}

/// 第六节第 6 行：变了的行归哪一类（第一节读法第 4 条）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum RowChangeClass {
    /// 甲：只是数的口径变了。
    CountBasisOnly,
    /// 乙：走法也变了。
    WalkChanged,
    /// 丙：这一行下界差从没为正而行变了，装置错（V5）。
    FloorGapNeverPositive,
}

impl RowChangeClass {
    pub(super) fn name(self) -> &'static str {
        match self {
            RowChangeClass::CountBasisOnly => "jia_count_basis_only",
            RowChangeClass::WalkChanged => "yi_walk_changed",
            RowChangeClass::FloorGapNeverPositive => "bing_floor_gap_never_positive",
        }
    }
}

fn classify_changed_row(first_row: &WindowRow, row: &WindowRow) -> RowChangeClass {
    let old = &first_row.metrics;
    let new = &row.metrics;
    let observations = &row.metrics.second_rerun;
    if observations.candidate_floor_gap.positive_samples == 0 {
        return RowChangeClass::FloorGapNeverPositive;
    }
    let is_same_walk = observations.rollback_target_changed == 0
        && observations.fakes_with_the_old_floor == old.fake_candidates
        && new.unrecoverable == old.unrecoverable
        && (!row.has_false_enospc_field || new.false_enospc_arm == old.false_enospc_arm);
    if is_same_walk { RowChangeClass::CountBasisOnly } else { RowChangeClass::WalkChanged }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct CompareTally {
    rows_compared: u64,
    pub(super) rows_with_a_count_changed: u64,
    rows_with_a_judgement_changed: u64,
    count_basis_only: u64,
    walk_changed: u64,
    pub(super) floor_gap_never_positive: u64,
}

impl CompareTally {
    pub(super) fn fresh() -> Self {
        CompareTally { rows_compared: 0, rows_with_a_count_changed: 0, rows_with_a_judgement_changed: 0, count_basis_only: 0, walk_changed: 0, floor_gap_never_positive: 0 }
    }
    fn add_class(&mut self, class: RowChangeClass) {
        match class {
            RowChangeClass::CountBasisOnly => self.count_basis_only += 1,
            RowChangeClass::WalkChanged => self.walk_changed += 1,
            RowChangeClass::FloorGapNeverPositive => self.floor_gap_never_positive += 1,
        }
    }
    pub(super) fn add(&mut self, other: &CompareTally) {
        self.rows_compared += other.rows_compared;
        self.rows_with_a_count_changed += other.rows_with_a_count_changed;
        self.rows_with_a_judgement_changed += other.rows_with_a_judgement_changed;
        self.count_basis_only += other.count_basis_only;
        self.walk_changed += other.walk_changed;
        self.floor_gap_never_positive += other.floor_gap_never_positive;
    }
    pub(super) fn render(&self) -> String {
        format!(
            "rows_compared={} rows_with_a_count_changed={} rows_with_a_judgement_changed={} jia_count_basis_only={} yi_walk_changed={} bing_floor_gap_never_positive={}",
            self.rows_compared, self.rows_with_a_count_changed, self.rows_with_a_judgement_changed, self.count_basis_only, self.walk_changed, self.floor_gap_never_positive
        )
    }
}

fn comparison_key(row: &WindowRow) -> String {
    format!("{}|{}|{}", line_suffix(row.line_name), row.arm.name(), row.row_key)
}

/// 第六节第 5、6、7 行：第 2 次重跑的每一行与第 1 次同 (行名去掉前缀, 臂, 行键) 的行比假候选、不可恢复、假性 ENOSPC 三个计数；变了的逐行打、逐行归因。
/// 没有第 1 次对应行的（RGall 一族、`disk_stuck`）不进比；返回比的合计与没有对应行的行数。
pub(super) fn emit_row_comparison(emitter: &mut Emitter, first_rows: &[WindowRow], rows: &[WindowRow]) -> (CompareTally, u64) {
    let first_by_key: HashMap<String, &WindowRow> = first_rows.iter().map(|row| (comparison_key(row), row)).collect();
    let mut tallies: BTreeMap<(String, &'static str), CompareTally> = BTreeMap::new();
    let mut rows_without_counterpart = 0u64;
    for row in rows {
        let Some(first_row) = first_by_key.get(&comparison_key(row)) else {
            rows_without_counterpart += 1;
            continue;
        };
        let old = &first_row.metrics;
        let new = &row.metrics;
        let is_count_changed = old.fake_candidates != new.fake_candidates
            || old.unrecoverable != new.unrecoverable
            || (row.has_false_enospc_field && old.false_enospc_arm != new.false_enospc_arm);
        let is_judgement_changed = first_row.judgement != row.judgement;
        let tally = tallies.entry((row.world_class.clone(), row.arm.name())).or_insert_with(CompareTally::fresh);
        tally.rows_compared += 1;
        tally.rows_with_a_judgement_changed += u64::from(is_judgement_changed);
        if is_count_changed {
            tally.rows_with_a_count_changed += 1;
            let class = classify_changed_row(first_row, row);
            tally.add_class(class);
            let enospc_pair = if row.has_false_enospc_field { format!("{}/{}", old.false_enospc_arm, new.false_enospc_arm) } else { "na".to_string() };
            println!("{}", emitter.emit_raw(&format!(
                "name=f_changed_row line={} arm={} {} fakes_old_new={}/{} unrecoverable_old_new={}/{} false_enospc_arm_old_new={enospc_pair} \
                 fakes_with_the_old_floor={} rollback_target_changed={} floor_gap_positive_samples={} judgement_old={} judgement_new={} class={}",
                row.line_name, row.arm.name(), row.row_key, old.fake_candidates, new.fake_candidates, old.unrecoverable, new.unrecoverable,
                new.second_rerun.fakes_with_the_old_floor, new.second_rerun.rollback_target_changed, new.second_rerun.candidate_floor_gap.positive_samples,
                judgement_text(&first_row.judgement), judgement_text(&row.judgement), class.name()
            )));
        }
    }
    let mut totals = CompareTally::fresh();
    for ((class, arm_name), tally) in &tallies {
        println!("{}", emitter.emit_raw(&format!("name=verdict_f_compare class={class} arm={arm_name} {}", tally.render())));
        totals.add(tally);
    }
    println!("{}", emitter.emit_raw(&format!("name=verdict_f_compare_total {} rows_without_a_first_rerun_counterpart={rows_without_counterpart}", totals.render())));
    (totals, rows_without_counterpart)
}

/// 第六节第 4 行：下界差 g 为正的采样，每类世界与合计；返回 g 为正的采样总数。
pub(super) fn emit_reach_verdicts(emitter: &mut Emitter, rows: &[WindowRow]) -> u64 {
    let mut classes: Vec<&str> = Vec::new();
    for row in rows {
        if !classes.contains(&row.world_class.as_str()) {
            classes.push(row.world_class.as_str());
        }
    }
    let mut positive_samples_total = 0u64;
    for class in classes {
        let class_rows: Vec<&WindowRow> = rows.iter().filter(|row| row.world_class == class).collect();
        let positive_samples: u64 = class_rows.iter().map(|row| row.metrics.second_rerun.candidate_floor_gap.positive_samples).sum();
        let peak = class_rows.iter().map(|row| row.metrics.second_rerun.candidate_floor_gap.maximum).max().unwrap_or(i64::MIN);
        positive_samples_total += positive_samples;
        println!("{}", emitter.emit_raw(&format!(
            "name=verdict_f_reach class={class} rows={} rows_with_positive_gap={} positive_samples_total={positive_samples} peak_gap={}",
            class_rows.len(), class_rows.iter().filter(|row| row.metrics.second_rerun.candidate_floor_gap.positive_samples > 0).count(), observed_or_na(peak)
        )));
    }
    println!("{}", emitter.emit_raw(&format!(
        "name=verdict_f_reach_summary positive_samples_total={positive_samples_total} cf_never_reached={}", u8::from(positive_samples_total == 0)
    )));
    positive_samples_total
}

/// 第六节第 8 行与 V6、V7：每条臂与合计的「旧候选集严格大于条款形」「条款形严格大于旧」次数；系统配置 F 低于根上 F 的次数。返回合计。
pub(super) fn emit_direction_verdicts(emitter: &mut Emitter, rows: &[WindowRow], family: &RowFamily) -> (u64, u64, u64) {
    let mut totals = (0u64, 0u64, 0u64);
    for arm in family.arms.iter().chain(family.short_arms.iter()).copied() {
        let arm_rows: Vec<&WindowRow> = rows.iter().filter(|row| row.arm == arm).collect();
        let old_larger: u64 = arm_rows.iter().map(|row| row.metrics.second_rerun.checks_old_set_strictly_larger).sum();
        let new_larger: u64 = arm_rows.iter().map(|row| row.metrics.second_rerun.checks_new_set_strictly_larger).sum();
        let below: u64 = arm_rows.iter().map(|row| row.metrics.second_rerun.system_configuration_below_root_floor_events).sum();
        let target_changed: u64 = arm_rows.iter().map(|row| row.metrics.second_rerun.rollback_target_changed).sum();
        println!("{}", emitter.emit_raw(&format!(
            "name=verdict_f_direction arm={} rows={} checks_old_set_strictly_larger={old_larger} checks_new_set_strictly_larger={new_larger} \
             rollback_target_changed={target_changed} system_configuration_below_root_floor_events={below}",
            arm.name(), arm_rows.len()
        )));
        totals = (totals.0 + old_larger, totals.1 + new_larger, totals.2 + below);
    }
    println!("{}", emitter.emit_raw(&format!(
        "name=verdict_f_direction_total checks_old_set_strictly_larger={} checks_new_set_strictly_larger={} system_configuration_below_root_floor_events={}",
        totals.0, totals.1, totals.2
    )));
    totals
}

/// 第六节第 11–13 行：条款形下不为零的行（假候选、不可恢复、有这一字段的行里的假性 ENOSPC），被测臂与对照臂逐行点名、不截断。
pub(super) fn emit_nonzero_rows(emitter: &mut Emitter, rows: &[WindowRow], family: &RowFamily) {
    for row in rows.iter().filter(|row| family.arms.contains(&row.arm)) {
        let is_nonzero = row.metrics.fake_candidates > 0 || row.metrics.unrecoverable > 0 || (row.has_false_enospc_field && row.metrics.false_enospc_arm > 0);
        if !is_nonzero {
            continue;
        }
        let false_enospc_text = if row.has_false_enospc_field { row.metrics.false_enospc_arm.to_string() } else { "na".to_string() };
        println!("{}", emitter.emit_raw(&format!(
            "name=f_nonzero_row line={} arm={} {} fakes={} unrecoverable={} false_enospc_arm={false_enospc_text} judged_false_enospc={} report_only_world={}",
            row.line_name, row.arm.name(), row.row_key, row.metrics.fake_candidates, row.metrics.unrecoverable,
            u8::from(row.judgement.has_false_enospc_in_a_judged_row), u8::from(row.is_report_only_world)
        )));
    }
}

/// 第六节第 14 行：RGall（读法 P）与 RGall_Q 逐行的三位判定比，另报某块幸存盘整环里一条有效根都没有的次数。返回判定不同的行数。
pub(super) fn emit_rgall_reading_verdicts(emitter: &mut Emitter, rows: &[WindowRow]) -> u64 {
    let rootless_skipping_arm = Arm::WholeRing(WholeRingArm::SkipsRootlessDisks);
    let rootless_zero_counting_arm = Arm::WholeRing(WholeRingArm::CountsRootlessDisksAsZero);
    let rootless_skipping_judgement_by_key: HashMap<(&str, &str), RowJudgement> = rows.iter()
        .filter(|row| row.arm == rootless_skipping_arm)
        .map(|row| ((row.line_name, row.row_key.as_str()), row.judgement)).collect();
    let mut compared = 0u64;
    let mut differing = 0u64;
    for row in rows.iter().filter(|row| row.arm == rootless_zero_counting_arm) {
        let Some(rootless_skipping_judgement) = rootless_skipping_judgement_by_key.get(&(row.line_name, row.row_key.as_str())) else {
            continue;
        };
        compared += 1;
        if *rootless_skipping_judgement != row.judgement {
            differing += 1;
            println!("{}", emitter.emit_raw(&format!(
                "name=f_rgall_readings_diff_row line={} {} reading_p={} reading_q={}", row.line_name, row.row_key, judgement_text(rootless_skipping_judgement), judgement_text(&row.judgement)
            )));
        }
    }
    let events_of = |arm: Arm| -> u64 { rows.iter().filter(|row| row.arm == arm).map(|row| row.metrics.second_rerun.disk_without_valid_root_events).sum() };
    println!("{}", emitter.emit_raw(&format!(
        "name=verdict_f_rgall_readings rows_compared={compared} rows_with_a_judgement_different_between_p_and_q={differing} \
         disk_without_valid_root_events_p={} disk_without_valid_root_events_q={}",
        events_of(rootless_skipping_arm), events_of(rootless_zero_counting_arm)
    )));
    differing
}

/// 第六节第 15、16 行：代价三样按（类, 臂）合并之后的轨迹（峰值、为正的采样数、期末值，全程与吃光供给之后各报）。
pub(super) fn emit_cost_verdicts(emitter: &mut Emitter, rows: &[WindowRow], family: &RowFamily) {
    let mut merged: BTreeMap<(String, &'static str), SecondRerunObservations> = BTreeMap::new();
    for row in rows.iter().filter(|row| family.arms.contains(&row.arm)) {
        merged.entry((row.world_class.clone(), row.arm.name())).or_insert_with(SecondRerunObservations::fresh).add(&row.metrics.second_rerun);
    }
    for ((class, arm_name), observations) in &merged {
        println!("{}", emitter.emit_raw(&format!("name=verdict_f_cost class={class} arm={arm_name} {}", observations.render_costs())));
    }
}

/// 第六节第 17 行：每个 (行名, 臂) 上有上界、没有上界、不判的行数；返回 (`disk_stuck` 里 G2SW 没有上界的行数, `broken_slot` 与 H1 里 RGall 没有上界的行数)。
pub(super) fn emit_growth_verdicts(emitter: &mut Emitter, growth: &[StuckGrowthSummary]) -> (u64, u64) {
    let mut line_and_arm_pairs: Vec<(&'static str, Arm)> = Vec::new();
    for summary in growth {
        if !line_and_arm_pairs.contains(&(summary.line_name, summary.arm)) {
            line_and_arm_pairs.push((summary.line_name, summary.arm));
        }
    }
    for (line_name, arm) in line_and_arm_pairs {
        let selected: Vec<&StuckGrowthSummary> = growth.iter().filter(|summary| summary.line_name == line_name && summary.arm == arm).collect();
        let count_of = |bound: StuckBound| selected.iter().filter(|summary| summary.bound == bound).count();
        println!("{}", emitter.emit_raw(&format!(
            "name=verdict_f_growth line={line_name} arm={} rows={} bounded={} unbounded={} not_judged={}",
            arm.name(), selected.len(), count_of(StuckBound::Bounded), count_of(StuckBound::Unbounded), count_of(StuckBound::NotJudged)
        )));
    }
    let window_arm = Arm::ClauseFloorWindowed(WindowArm::Window);
    let is_whole_ring = |arm: Arm| matches!(arm, Arm::WholeRing(_));
    let window_unbounded_in_disk_stuck = growth.iter()
        .filter(|summary| summary.line_name == SECOND_RERUN_LINES.stuck_growth && summary.arm == window_arm && summary.bound == StuckBound::Unbounded).count() as u64;
    let whole_ring_unbounded_in_broken_slot = growth.iter()
        .filter(|summary| (summary.line_name == SECOND_RERUN_LINES.broken_slot_growth || summary.line_name == SECOND_RERUN_LINES.broken_slot_reversed_growth)
            && is_whole_ring(summary.arm) && summary.bound == StuckBound::Unbounded).count() as u64;
    (window_unbounded_in_disk_stuck, whole_ring_unbounded_in_broken_slot)
}

/// 第八节几何敏感性（刻度与判）：同一臂、同一组参数上两个取样点的「有上界 / 没有上界」逐行比；返回判不同的行数。
pub(super) fn emit_growth_pairing(
    emitter: &mut Emitter,
    kind: &str,
    growth: &[StuckGrowthSummary],
    is_base: impl Fn(&StuckGrowthSummary) -> bool,
    is_other: impl Fn(&StuckGrowthSummary) -> bool,
    pairing_key: impl Fn(&StuckGrowthSummary) -> String,
) -> u64 {
    let mut arms: Vec<Arm> = Vec::new();
    for summary in growth.iter().filter(|summary| is_base(summary)) {
        if !arms.contains(&summary.arm) {
            arms.push(summary.arm);
        }
    }
    let mut total_differing = 0u64;
    for arm in arms {
        let base_bound_by_key: HashMap<String, StuckBound> = growth.iter().filter(|summary| summary.arm == arm && is_base(summary)).map(|summary| (pairing_key(summary), summary.bound)).collect();
        let mut compared = 0u64;
        let mut differing = 0u64;
        for summary in growth.iter().filter(|summary| summary.arm == arm && is_other(summary)) {
            let Some(base_bound) = base_bound_by_key.get(&pairing_key(summary)) else {
                continue;
            };
            compared += 1;
            differing += u64::from(*base_bound != summary.bound);
        }
        total_differing += differing;
        println!("{}", emitter.emit_raw(&format!(
            "name=verdict_f_growth_geometry point={kind} arm={} rows_compared={compared} rows_with_a_different_bound={differing}", arm.name()
        )));
    }
    total_differing
}

/// 占用两档（0.30 与 0.70）上 `disk_stuck` 行的三位判定逐行比；返回判定不同的行数。
pub(super) fn emit_stuck_fill_verdicts(emitter: &mut Emitter, rows: &[WindowRow]) -> u64 {
    let is_stuck_row = |row: &&WindowRow| row.world_class == DISK_STUCK_CLASS || row.world_class == DISK_STUCK_REVERSED_CLASS;
    let mut total_differing = 0u64;
    let mut arms: Vec<Arm> = Vec::new();
    for row in rows.iter().filter(is_stuck_row) {
        if !arms.contains(&row.arm) {
            arms.push(row.arm);
        }
    }
    for arm in arms {
        let low_fill_judgement_by_key: HashMap<(&str, String), RowJudgement> = rows.iter()
            .filter(is_stuck_row)
            .filter(|row| row.arm == arm && row.row_key.contains("fill=0.30"))
            .map(|row| ((row.line_name, row_key_without_the_field(&row.row_key, "fill=")), row.judgement)).collect();
        let mut compared = 0u64;
        let mut differing = 0u64;
        for row in rows.iter().filter(is_stuck_row).filter(|row| row.arm == arm && row.row_key.contains("fill=0.70")) {
            let Some(low_fill_judgement) = low_fill_judgement_by_key.get(&(row.line_name, row_key_without_the_field(&row.row_key, "fill="))) else {
                continue;
            };
            compared += 1;
            differing += u64::from(*low_fill_judgement != row.judgement);
        }
        total_differing += differing;
        println!("{}", emitter.emit_raw(&format!(
            "name=verdict_f_fill_geometry arm={} rows_compared={compared} rows_with_a_judgement_different_between_fills={differing}", arm.name()
        )));
    }
    total_differing
}

/// 登记第五节「别的世界不量斜率」：洞里的旧根至多活两圈。非 `broken_slot`、非 `disk_stuck` 的行里采样时幸存盘上仍有 txg + 2 × 3S ≤ N 的有效根的行逐行点名；返回这样的行数。
pub(super) fn emit_stale_verdicts(emitter: &mut Emitter, rows: &[WindowRow], family: &RowFamily) -> u64 {
    let is_stuck_world = |class: &str| class == BROKEN_SLOT_CLASS || class == DISK_STUCK_CLASS || class == DISK_STUCK_REVERSED_CLASS;
    let mut rows_outside_the_stuck_worlds = 0u64;
    let mut rows_in_the_stuck_worlds = 0u64;
    for row in rows.iter().filter(|row| family.arms.contains(&row.arm) || family.short_arms.contains(&row.arm)) {
        let samples = row.metrics.second_rerun.stale_beyond_two_revolutions_samples;
        if samples == 0 {
            continue;
        }
        if is_stuck_world(&row.world_class) {
            rows_in_the_stuck_worlds += 1;
        } else {
            rows_outside_the_stuck_worlds += 1;
            println!("{}", emitter.emit_raw(&format!("name=f_stale_row line={} arm={} {} stale_beyond_two_revolutions={samples}", row.line_name, row.arm.name(), row.row_key)));
        }
    }
    println!("{}", emitter.emit_raw(&format!(
        "name=verdict_f_stale rows_outside_the_stuck_worlds_with_a_root_beyond_two_revolutions={rows_outside_the_stuck_worlds} \
         rows_in_the_stuck_worlds_with_a_root_beyond_two_revolutions={rows_in_the_stuck_worlds}"
    )));
    rows_outside_the_stuck_worlds
}

/// V8：条款形下的 G2SW、G2SW_Q、G2SW_L 的 x 按定义恒 0（回收门槛就是窗口门槛）；有一次不为 0 的行数。
pub(super) fn count_window_arm_rows_with_a_nonzero_extra_pinned(rows: &[WindowRow]) -> u64 {
    let window_arms = [Arm::ClauseFloorWindowed(WindowArm::Window), Arm::ClauseFloorWindowed(WindowArm::DiskHold), Arm::ClauseFloorWindowed(WindowArm::RingLag)];
    rows.iter().filter(|row| window_arms.contains(&row.arm)).filter(|row| {
        let quantity = &row.metrics.second_rerun.extra_pinned_against_the_window_gate;
        quantity.samples > 0 && (quantity.maximum != 0 || quantity.minimum != 0)
    }).count() as u64
}
