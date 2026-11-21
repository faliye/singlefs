//! 核心门那一臂的状态与读数、被观测的量与轨迹、卡住的标记，以及第二次重跑的观测汇总。

use super::*;

/// 第 3 次重跑第五节「CG」：CG 臂在 `Simulation` 上多记的状态。回收门槛的 F 那一项仍住 `floor_active`、新根带的 F 仍住 `floor_carried`，
/// 两者在 CG 臂上照 CG 的取法写；这里记 CG-F 的扣住那一截、CG-P 的钉槽，与同一状态上旧式门槛的影子（第六节第 4 行「旧式门槛」，只读不写回）。
#[derive(Clone)]
pub(super) struct CoreGateState {
    /// K-eq ② 的开关：关着时 CG 臂照 RGall 原样（恢复与回退照旧式取 F、不钉槽），影子与真值逐次相同。产物里恒开。
    pub(super) is_switched_on: bool,
    /// CG-F：最近一次恢复算出的 F_生效。
    pub(super) effective_floor: u64,
    /// CG-F：F 那一项还压在 min(F_生效, m) 上，在等这个实例号起（含之后回退开的实例）的根落满每块幸存盘；不在等时 `None`。
    pub(super) held_until_rooted_from_instance: Option<u64>,
    /// CG-P：钉着的槽 → 钉值（落到这个槽、在 [0, min(3S, 新实例第一次发布的 txg)) 里的最小 txg）。
    pub(super) pinned_slots: BTreeMap<usize, u64>,
    /// 同一状态上旧式（第 2 次 RGall）的 F：恢复与回退照 `floors_after_recovery` 重算，抬 F 与生效照原样跟。
    pub(super) shadow_floor_carried: u64,
    pub(super) shadow_floor_active: u64,
    pub(super) shadow_pending_floor_value: Option<u64>,
}

impl CoreGateState {
    pub(super) fn fresh() -> Self {
        CoreGateState {
            is_switched_on: true,
            effective_floor: 0,
            held_until_rooted_from_instance: None,
            pinned_slots: BTreeMap::new(),
            shadow_floor_carried: 0,
            shadow_floor_active: 0,
            shadow_pending_floor_value: None,
        }
    }
}

/// CG-F 与 CG-P 在择根之前读到的值（第一节读法第 1 条）。
pub(super) struct CoreGateReadingAtRecovery {
    pub(super) effective_floor: u64,
    /// m：每块幸存盘上最新有效根 txg 的最小；有一块幸存盘一条有效根都没有时 `None`。
    pub(super) newest_valid_txg_on_every_surviving_disk: Option<u64>,
    pub(super) pinned_slots: BTreeMap<usize, u64>,
}

/// 第 3 次重跑第六节第 4 行的新量：每次 `allocate` 读一次（只在 CG 臂上采，几次运行合计时相加）。
#[derive(Clone, Copy, Debug)]
pub(super) struct CoreGateObservations {
    /// d = CG 门槛 − 同一状态上旧式门槛。
    pub(super) gate_difference: ObservedQuantity,
    pub(super) gate_difference_nonzero_samples: u64,
    pub(super) gate_difference_negative_samples: u64,
    /// dF = CG 的 F 那一项 − 旧式 F 那一项 ≠ 0 的采样数。
    pub(super) floor_term_difference_nonzero_samples: u64,
    /// 钉值的最小 < `oldest_persisted_valid_txg()` 的采样数。
    pub(super) pin_effective_samples: u64,
    /// F 那一项 < F_生效 的采样数。
    pub(super) held_waiting_samples: u64,
}

impl CoreGateObservations {
    pub(super) fn fresh() -> Self {
        CoreGateObservations {
            gate_difference: ObservedQuantity::fresh(),
            gate_difference_nonzero_samples: 0,
            gate_difference_negative_samples: 0,
            floor_term_difference_nonzero_samples: 0,
            pin_effective_samples: 0,
            held_waiting_samples: 0,
        }
    }
    pub(super) fn add(&mut self, other: &CoreGateObservations) {
        self.gate_difference.add(&other.gate_difference);
        self.gate_difference_nonzero_samples += other.gate_difference_nonzero_samples;
        self.gate_difference_negative_samples += other.gate_difference_negative_samples;
        self.floor_term_difference_nonzero_samples += other.floor_term_difference_nonzero_samples;
        self.pin_effective_samples += other.pin_effective_samples;
        self.held_waiting_samples += other.held_waiting_samples;
    }
    /// `core_gate_reach` 行的字段。
    pub(super) fn render(&self) -> String {
        format!(
            "{} gate_difference_nonzero={} gate_difference_negative={} floor_term_difference_nonzero={} pin_effective={} held_waiting={}",
            self.gate_difference.render("gate_difference"), self.gate_difference_nonzero_samples, self.gate_difference_negative_samples,
            self.floor_term_difference_nonzero_samples, self.pin_effective_samples, self.held_waiting_samples
        )
    }
}

/// 第 1 次重跑第八节的轨迹：被谓词消费的量报「吃光初始供给之后的峰值、为正的次数、期末值」（`test-discipline.md`「端点不是轨迹」）。
#[derive(Clone, Copy, Debug)]
pub(super) struct ObservedQuantity {
    pub(super) samples: u64,
    pub(super) positive_samples: u64,
    pub(super) maximum: i64,
    maximum_after_supply: i64,
    pub(super) minimum: i64,
    /// 一次运行里最后一个读数；几次运行合计时取各次最后读数的最大值。
    pub(super) last: i64,
}

impl ObservedQuantity {
    pub(super) fn fresh() -> Self {
        ObservedQuantity { samples: 0, positive_samples: 0, maximum: i64::MIN, maximum_after_supply: i64::MIN, minimum: i64::MAX, last: i64::MIN }
    }
    pub(super) fn observe(&mut self, value: i64, is_supply_exhausted: bool) {
        self.samples += 1;
        self.positive_samples += u64::from(value > 0);
        self.maximum = self.maximum.max(value);
        if is_supply_exhausted {
            self.maximum_after_supply = self.maximum_after_supply.max(value);
        }
        self.minimum = self.minimum.min(value);
        self.last = value;
    }
    pub(super) fn add(&mut self, other: &ObservedQuantity) {
        self.samples += other.samples;
        self.positive_samples += other.positive_samples;
        self.maximum = self.maximum.max(other.maximum);
        self.maximum_after_supply = self.maximum_after_supply.max(other.maximum_after_supply);
        self.minimum = self.minimum.min(other.minimum);
        self.last = self.last.max(other.last);
    }
    /// 没读数的字段写 na，不写成 0（读不到不等于 0）。
    pub(super) fn render(&self, label: &str) -> String {
        let shown = |value: i64, unobserved: i64| if value == unobserved { "na".to_string() } else { value.to_string() };
        format!(
            "{label}_samples={} {label}_positive={} {label}_max={} {label}_max_after_supply={} {label}_min={} {label}_last_max={}",
            self.samples, self.positive_samples, shown(self.maximum, i64::MIN), shown(self.maximum_after_supply, i64::MIN),
            shown(self.minimum, i64::MAX), shown(self.last, i64::MIN)
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Trajectory {
    /// 第六节第 6 行：这条臂自己的回收门槛减整环门槛，每次 `allocate` 之前读一次。
    pub(super) gate_difference: ObservedQuantity,
    /// 这条臂的抬 F 上限减原 G2S 整环取法的上限，每次决定抬不抬 F 时读一次。
    pub(super) ceiling_difference: ObservedQuantity,
    /// 滞后量与这条臂自己的 `df`，每个用户窗口读一次。
    pub(super) lag: ObservedQuantity,
    pub(super) df: ObservedQuantity,
    pub(super) df_below_user_blocks_windows: u64,
}

impl Trajectory {
    pub(super) fn fresh() -> Self {
        Trajectory {
            gate_difference: ObservedQuantity::fresh(),
            ceiling_difference: ObservedQuantity::fresh(),
            lag: ObservedQuantity::fresh(),
            df: ObservedQuantity::fresh(),
            df_below_user_blocks_windows: 0,
        }
    }
    pub(super) fn add(&mut self, other: &Trajectory) {
        self.gate_difference.add(&other.gate_difference);
        self.ceiling_difference.add(&other.ceiling_difference);
        self.lag.add(&other.lag);
        self.df.add(&other.df);
        self.df_below_user_blocks_windows += other.df_below_user_blocks_windows;
    }
    pub(super) fn render(&self) -> String {
        format!(
            "{} {} {} {} df_below_user_blocks_windows={}",
            self.gate_difference.render("gate_difference"), self.ceiling_difference.render("ceiling_difference"),
            self.lag.render("lag"), self.df.render("df"), self.df_below_user_blocks_windows
        )
    }
}

pub(super) fn signed_difference(left: u64, right: u64) -> i64 {
    i64::try_from(left).expect("块数与 txg 装得进 i64") - i64::try_from(right).expect("块数与 txg 装得进 i64")
}

/// 刻度那一次采样（登记第一节读法第 10 条 ⓒ）：第一个落盘 txg ≥ T₀ + k × 3S 的那次根落盘之后读 `pinned_count()`，
/// 连同这条臂到那时为止放掉的块数（`Simulation::free_blocks` 累计）。
#[derive(Clone, Copy, Debug)]
pub(super) struct StuckMark {
    pub(super) published_txg: u64,
    pub(super) pinned_blocks: u64,
    pub(super) blocks_freed_in_total: u64,
}

/// 两个刻度之间：（扣住的块数增量，这条臂放掉的块数）。放掉的块数为 0 时没有斜率。
pub(super) fn stuck_slope_between(earlier: &StuckMark, later: &StuckMark) -> Option<(i64, i64)> {
    let freed_between = signed_difference(later.blocks_freed_in_total, earlier.blocks_freed_in_total);
    if freed_between <= 0 {
        return None;
    }
    Some((signed_difference(later.pinned_blocks, earlier.pinned_blocks), freed_between))
}

/// 「有上界」：s < 1/2，写成 2 × 增量 < 放掉的块数，不取整（登记第一节读法第 10 条 ⓒ）。
pub(super) fn is_stuck_slope_bounded(pinned_increase: i64, blocks_freed_between: i64) -> bool {
    STUCK_BOUNDED_SLOPE_DENOMINATOR * pinned_increase < blocks_freed_between
}

pub(super) fn slope_in_hundredths(pinned_increase: i64, blocks_freed_between: i64) -> i64 {
    pinned_increase * 100 / blocks_freed_between
}

/// 第 2 次重跑的新量，只在 `Arm::collects_second_rerun_observations` 的臂上采、只在 `f_` 行里打印（`w_` 行不读它们）。
#[derive(Clone, Copy, Debug)]
pub(super) struct SecondRerunObservations {
    /// 下界差 g = 条款形候选下界 − 旧候选下界：每次 `record_check` 与每次 `admin_rollback` 读一次。
    pub(super) candidate_floor_gap: ObservedQuantity,
    pub(super) rollback_target_changed: u64,
    pub(super) fakes_with_the_old_floor: u64,
    pub(super) checks_old_set_strictly_larger: u64,
    pub(super) checks_new_set_strictly_larger: u64,
    pub(super) system_configuration_below_root_floor_events: u64,
    /// ⓐ 每次 `crash_and_recover` 做完（`adopt_floors` 之后）的 `pinned_count()`。
    pub(super) pinned_after_recovery: ObservedQuantity,
    /// ⓑ 每次根落盘之后：这条臂的门槛下扣住的块数减窗口门槛下扣住的块数。
    pub(super) extra_pinned_against_the_window_gate: ObservedQuantity,
    /// ⓑ 幸存盘上住着窗口外有效根的槽数 > 0 的采样上：多扣的块数之和、槽数之和、多扣的块数除以槽数的最大值（乘 100 取整）。
    pub(super) samples_with_a_hole: u64,
    pub(super) extra_pinned_sum_over_samples_with_a_hole: i64,
    pub(super) hole_slots_sum_over_samples_with_a_hole: u64,
    pub(super) extra_pinned_per_hole_maximum_in_hundredths: i64,
    /// ⓒ 每次根落盘之后的 `pinned_count()`。
    pub(super) pinned_after_publication: ObservedQuantity,
    /// 采样时幸存盘上有一条有效根满足 txg + 2 × 3S ≤ N 的采样数（登记第五节「别的世界不量斜率」）。
    pub(super) stale_beyond_two_revolutions_samples: u64,
    /// 每次决定抬不抬 F 时：某块幸存盘整环里一条有效根都没有的次数。
    pub(super) disk_without_valid_root_events: u64,
    /// 刻度 k = 1、2、4（`STUCK_MARK_REVOLUTIONS`）；没走到的是 `None`。几次运行合计时不合并，只留自己的。
    pub(super) marks: [Option<StuckMark>; 3],
}

impl SecondRerunObservations {
    pub(super) fn fresh() -> Self {
        SecondRerunObservations {
            candidate_floor_gap: ObservedQuantity::fresh(),
            rollback_target_changed: 0,
            fakes_with_the_old_floor: 0,
            checks_old_set_strictly_larger: 0,
            checks_new_set_strictly_larger: 0,
            system_configuration_below_root_floor_events: 0,
            pinned_after_recovery: ObservedQuantity::fresh(),
            extra_pinned_against_the_window_gate: ObservedQuantity::fresh(),
            samples_with_a_hole: 0,
            extra_pinned_sum_over_samples_with_a_hole: 0,
            hole_slots_sum_over_samples_with_a_hole: 0,
            extra_pinned_per_hole_maximum_in_hundredths: i64::MIN,
            pinned_after_publication: ObservedQuantity::fresh(),
            stale_beyond_two_revolutions_samples: 0,
            disk_without_valid_root_events: 0,
            marks: [None; 3],
        }
    }
    pub(super) fn add(&mut self, other: &SecondRerunObservations) {
        self.candidate_floor_gap.add(&other.candidate_floor_gap);
        self.rollback_target_changed += other.rollback_target_changed;
        self.fakes_with_the_old_floor += other.fakes_with_the_old_floor;
        self.checks_old_set_strictly_larger += other.checks_old_set_strictly_larger;
        self.checks_new_set_strictly_larger += other.checks_new_set_strictly_larger;
        self.system_configuration_below_root_floor_events += other.system_configuration_below_root_floor_events;
        self.pinned_after_recovery.add(&other.pinned_after_recovery);
        self.extra_pinned_against_the_window_gate.add(&other.extra_pinned_against_the_window_gate);
        self.samples_with_a_hole += other.samples_with_a_hole;
        self.extra_pinned_sum_over_samples_with_a_hole += other.extra_pinned_sum_over_samples_with_a_hole;
        self.hole_slots_sum_over_samples_with_a_hole += other.hole_slots_sum_over_samples_with_a_hole;
        self.extra_pinned_per_hole_maximum_in_hundredths = self.extra_pinned_per_hole_maximum_in_hundredths.max(other.extra_pinned_per_hole_maximum_in_hundredths);
        self.pinned_after_publication.add(&other.pinned_after_publication);
        self.stale_beyond_two_revolutions_samples += other.stale_beyond_two_revolutions_samples;
        self.disk_without_valid_root_events += other.disk_without_valid_root_events;
    }
    /// `f_attribution` 行的字段：第六节第 4、6、7、8 行与 V5–V7 要的量。
    pub(super) fn render_attribution(&self) -> String {
        format!(
            "{} rollback_target_changed={} fakes_with_the_old_floor={} checks_old_set_strictly_larger={} checks_new_set_strictly_larger={} \
             system_configuration_below_root_floor_events={}",
            self.candidate_floor_gap.render("candidate_floor_gap"), self.rollback_target_changed, self.fakes_with_the_old_floor,
            self.checks_old_set_strictly_larger, self.checks_new_set_strictly_larger, self.system_configuration_below_root_floor_events
        )
    }
    /// `f_trajectory` 行接在第 1 次四样轨迹之后的字段：代价三样与两个推的量。
    pub(super) fn render_costs(&self) -> String {
        let per_hole_maximum = if self.extra_pinned_per_hole_maximum_in_hundredths == i64::MIN { "na".to_string() } else { self.extra_pinned_per_hole_maximum_in_hundredths.to_string() };
        format!(
            "{} {} holes_samples={} extra_pinned_sum_at_holes={} hole_slots_sum_at_holes={} extra_pinned_per_hole_max_x100={per_hole_maximum} {} \
             stale_beyond_two_revolutions={} disk_without_valid_root_events={}",
            self.pinned_after_recovery.render("pinned_after_recovery"), self.extra_pinned_against_the_window_gate.render("extra_pinned_vs_window_gate"),
            self.samples_with_a_hole, self.extra_pinned_sum_over_samples_with_a_hole, self.hole_slots_sum_over_samples_with_a_hole,
            self.pinned_after_publication.render("pinned_after_publication"), self.stale_beyond_two_revolutions_samples, self.disk_without_valid_root_events
        )
    }
}
