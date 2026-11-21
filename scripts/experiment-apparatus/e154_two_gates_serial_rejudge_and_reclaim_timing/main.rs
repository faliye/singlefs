//! E154 两道闸串-重判与回收时点的代价。
//!
//! 跑前登记：`research/prompts/e154-preregistration.md`。纯计数模型：没有 I/O、没有并发、
//! 没有随机源，同一份输入跑 N 遍必然一样；证据强度来自本文件的绝对值单测与变异表
//! （`scripts/mutation/tables/e154_two_gates_serial_rejudge_and_reclaim_timing.tsv`），不靠重跑轮数。
//!
//! 输出体量说明（技术实现细节，不改登记「六、七、九」的判据）：六节要求「臂 × 历史 × 格」
//! 逐格报判定，本文件对全部 48 格 × 24 臂 × 12 历史都算出并报出 `RUN` 汇总行（Q1–Q8 与
//! 几何一起报）；`ROW`（逐次尝试）明细只对真实基线（串-重判 × G7，两种停法）、全部阳性对照、
//! 以及任何一次触发失败条款的跑打印——全仓现存最大产物 371 KB
//! （`research/results/e47-ring-loss-2026-08-30.out`），13824 次跑的全量 ROW 明细会把产物撑到
//! 数十 MB，不是这个仓的产物体量。RUN 行本身覆盖了六节「分辨格」要求的全部判定，没有被这项
//! 调整削弱；I1–I10、A1–A6 由单测钉死，不依赖 ROW 明细是否打印。

use e7_index_bench::Emitter;
use std::collections::BTreeMap;
use std::rc::Rc;

mod admission_and_retry_loop;
mod arms_geometries_and_root_records;
mod failure_injectors;
mod gates_and_reserves;
mod histories_of_failures_rollbacks_and_crashes;
mod pool_state_and_damage;
mod publishing_and_instance_switches;
mod registered_constants_and_slot_states;
mod run_summaries_and_fill;
mod slot_allocation_and_reclaim;
#[cfg(test)]
mod test_support_for_the_baseline_runs;

use self::admission_and_retry_loop::*;
use self::arms_geometries_and_root_records::*;
use self::failure_injectors::*;
use self::gates_and_reserves::*;
use self::histories_of_failures_rollbacks_and_crashes::*;
use self::pool_state_and_damage::*;
use self::publishing_and_instance_switches::*;
use self::registered_constants_and_slot_states::*;
use self::run_summaries_and_fill::*;
use self::slot_allocation_and_reclaim::*;
#[cfg(test)]
use self::test_support_for_the_baseline_runs::*;

fn run_summary_line(summary: &RunSummary) -> String {
    format!(
        "RUN gate={} baseline={} reclaim={} stop={} history={} c={} n={} c_max={} s={} pad={} delete_then_rewrite_outcome_label={} delete_then_rewrite_publish_count={} df_reported_enough_but_write_failed_count={} admission_passed_but_allocation_failed_count={} budget_was_binding={} maximum_publish_attempts_seen_field={} allocation_failure_hit_field={} wedged={} unsafe_candidate_reuse_count_field={} retained_states_dropped_below_minimum={} filler_objects_written_so_far={}",
        summary.arm,
        summary.is_real_baseline,
        summary.reclaim_label,
        summary.stop_label,
        summary.history,
        summary.capacity,
        summary.object_units,
        summary.fixpoint_cost,
        summary.slots_per_region,
        summary.padding_deletes,
        summary.delete_then_rewrite_outcome_label,
        summary.delete_then_rewrite_publish_count,
        summary.df_reported_enough_but_write_failed_count,
        summary.admission_passed_but_allocation_failed_count,
        summary.budget_was_binding,
        summary.maximum_publish_attempts_seen_field,
        summary.allocation_failure_hit_field,
        summary.wedged,
        summary.unsafe_candidate_reuse_count_field,
        summary.retained_states_dropped_below_minimum,
        summary.filler_objects_written_in_fill_phase,
    )
}

fn main() {
    let mut emitter = Emitter::new();
    let mut run_count = 0u64;
    for geometry in all_geometries() {
        for arm in all_arms() {
            let summary = run_delete_then_rewrite_history(arm, geometry);
            println!("{}", emitter.emit_raw(&run_summary_line(&summary)));
            run_count += 1;
        }
    }

    // H5-k-位（登记「五、5.4」，第二段）：k ∈ {1,2,3} × 位 ∈ {首,覆}，24 臂 × 48 格。
    for consecutive_root_write_failure_count in [1u64, 2, 3] {
        for position in [FailurePosition::First, FailurePosition::Cover] {
            for geometry in all_geometries() {
                for arm in all_arms() {
                    let (summary, injected, switches_triggered, recovery_publishes_until_covered) =
                        run_root_write_failure_history(arm, geometry, consecutive_root_write_failure_count, position, Damage::default());
                    println!("{}", emitter.emit_raw(&run_summary_line(&summary)));
                    println!(
                        "{}",
                        emitter.emit_raw(&format!(
                            "H5DETAIL gate={} reclaim={} stop={} k={} position={} c={} n={} c_max={} s={} pad={} injected_failures={} switches_triggered={} recovery_publishes_until_covered={}",
                            summary.arm,
                            summary.reclaim_label,
                            summary.stop_label,
                            consecutive_root_write_failure_count,
                            position.label(),
                            geometry.capacity_slots_per_disk,
                            geometry.object_units,
                            geometry.fixpoint_cost,
                            geometry.slots_per_region,
                            geometry.padding_deletes,
                            injected,
                            switches_triggered,
                            recovery_publishes_until_covered,
                        ))
                    );
                    run_count += 1;
                }
            }
        }
    }

    // H8（第四段）：同一套 k × 位，注入点改成填满阶段的自推空发布，
    // 填满阶段不自推时原样落回 H5 的注入点；24 臂 × 48 格。
    for consecutive_root_write_failure_count in [1u64, 2, 3] {
        for position in [FailurePosition::First, FailurePosition::Cover] {
            for geometry in all_geometries() {
                for arm in all_arms() {
                    let (summary, injected, switches_triggered, recovery_publishes_until_covered, fill_phase_self_pushes) =
                        run_fill_phase_injected_history(arm, geometry, consecutive_root_write_failure_count, position, Damage::default());
                    println!("{}", emitter.emit_raw(&run_summary_line(&summary)));
                    println!(
                        "{}",
                        emitter.emit_raw(&format!(
                            "H8DETAIL gate={} reclaim={} stop={} k={} position={} c={} n={} c_max={} s={} pad={} injected_failures={} switches_triggered={} recovery_publishes_until_covered={} fill_phase_self_pushes={}",
                            summary.arm,
                            summary.reclaim_label,
                            summary.stop_label,
                            consecutive_root_write_failure_count,
                            position.label(),
                            geometry.capacity_slots_per_disk,
                            geometry.object_units,
                            geometry.fixpoint_cost,
                            geometry.slots_per_region,
                            geometry.padding_deletes,
                            injected,
                            switches_triggered,
                            recovery_publishes_until_covered,
                            fill_phase_self_pushes,
                        ))
                    );
                    run_count += 1;
                }
            }
        }
    }

    // H6（登记「五、5.4」，第二段）：24 臂 × 48 格。
    for geometry in all_geometries() {
        for arm in all_arms() {
            let (summary, isolated_slots_after_rollback) = run_rollback_history(arm, geometry, Damage::default());
            println!("{}", emitter.emit_raw(&run_summary_line(&summary)));
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "H6DETAIL gate={} reclaim={} stop={} c={} n={} c_max={} s={} pad={} isolated_slots_after_rollback={}",
                    summary.arm,
                    summary.reclaim_label,
                    summary.stop_label,
                    geometry.capacity_slots_per_disk,
                    geometry.object_units,
                    geometry.fixpoint_cost,
                    geometry.slots_per_region,
                    geometry.padding_deletes,
                    isolated_slots_after_rollback,
                ))
            );
            run_count += 1;
        }
    }

    // H4a／H4b（登记「五、5.4」，第四段）：推空发布之中崩溃再重开；24 臂 × 48 格。
    for crash_point in [CrashPoint::AfterFirstPush, CrashPoint::BeforeSecondPush] {
        for geometry in all_geometries() {
            for arm in all_arms() {
                let (summary, self_pushes_before_crash, recovery_publishes_after_reopen) =
                    run_crash_mid_push_history(arm, geometry, crash_point, Damage::default());
                println!("{}", emitter.emit_raw(&run_summary_line(&summary)));
                println!(
                    "{}",
                    emitter.emit_raw(&format!(
                        "H4DETAIL gate={} reclaim={} stop={} crash_point={} c={} n={} c_max={} s={} pad={} self_pushes_before_crash={} recovery_publishes_after_reopen={}",
                        summary.arm,
                        summary.reclaim_label,
                        summary.stop_label,
                        crash_point.label(),
                        geometry.capacity_slots_per_disk,
                        geometry.object_units,
                        geometry.fixpoint_cost,
                        geometry.slots_per_region,
                        geometry.padding_deletes,
                        self_pushes_before_crash,
                        recovery_publishes_after_reopen,
                    ))
                );
                run_count += 1;
            }
        }
    }

    // Q3：B=∞ 反事实（登记「五、5.5」「不设上限；P-满 改成推到闸过或本次准入推满 10×3S 次为止」），
    // 同臂同格与 B=8 的基线比——H1、全部 24 臂 × 48 格。
    let mut unbounded_budget_diverged_count = 0u64;
    for geometry in all_geometries() {
        let unbounded_budget = 10 * 3 * geometry.slots_per_region;
        for arm in all_arms() {
            let baseline = run_delete_then_rewrite_history(arm, geometry);
            let unbounded = run_delete_then_rewrite_history_with_damage(
                arm,
                geometry,
                Damage { budget_override: Some(unbounded_budget), ..Damage::default() },
            );
            let diverged = baseline.delete_then_rewrite_outcome_label != unbounded.delete_then_rewrite_outcome_label
                || baseline.delete_then_rewrite_publish_count != unbounded.delete_then_rewrite_publish_count
                || baseline.wedged != unbounded.wedged;
            if diverged {
                unbounded_budget_diverged_count += 1;
            }
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "Q3 gate={} reclaim={} stop={} history=H1 c={} n={} c_max={} s={} pad={} baseline_q1={}/{} unbounded_q1={}/{} baseline_wedged={} unbounded_wedged={} diverged={}",
                    baseline.arm,
                    baseline.reclaim_label,
                    baseline.stop_label,
                    geometry.capacity_slots_per_disk,
                    geometry.object_units,
                    geometry.fixpoint_cost,
                    geometry.slots_per_region,
                    geometry.padding_deletes,
                    baseline.delete_then_rewrite_outcome_label,
                    baseline.delete_then_rewrite_publish_count,
                    unbounded.delete_then_rewrite_outcome_label,
                    unbounded.delete_then_rewrite_publish_count,
                    baseline.wedged,
                    unbounded.wedged,
                    diverged,
                ))
            );
            run_count += 1;
        }
    }

    // 阳性对照（5.6）：每条改坏在指定的格上，遍历全部 24 条臂。
    let pc_geometry = Geometry {
        capacity_slots_per_disk: 4000,
        object_units: 16,
        fixpoint_cost: 4,
        slots_per_region: 8,
        padding_deletes: 2,
    };
    let mut pc1_hit_any = false;
    let mut pc2_hit_any = false;
    let mut pc4_hit_any = false;
    let mut pc5_hit_any = false;
    for arm in all_arms() {
        let baseline = run_delete_then_rewrite_history(arm, pc_geometry);
        let pc1 = run_delete_then_rewrite_history_with_damage(
            arm,
            pc_geometry,
            Damage { disable_floor_raise: true, ..Damage::default() },
        );
        if pc1.delete_then_rewrite_outcome_label != baseline.delete_then_rewrite_outcome_label || pc1.delete_then_rewrite_publish_count != baseline.delete_then_rewrite_publish_count {
            pc1_hit_any = true;
        }
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "PC name=pc1 arm={} baseline_q1={}/{} damaged_q1={}/{}",
                arm.label(),
                baseline.delete_then_rewrite_outcome_label,
                baseline.delete_then_rewrite_publish_count,
                pc1.delete_then_rewrite_outcome_label,
                pc1.delete_then_rewrite_publish_count,
            ))
        );

        let pc2 = run_delete_then_rewrite_history_with_damage(arm, pc_geometry, Damage { raw_df: true, ..Damage::default() });
        if pc2.df_reported_enough_but_write_failed_count >= 1 {
            pc2_hit_any = true;
        }
        println!("{}", emitter.emit_raw(&format!("PC name=pc2 arm={} df_reported_enough_but_write_failed_count={}", arm.label(), pc2.df_reported_enough_but_write_failed_count)));

        let pc4_damage = if arm.reclaim == ReclaimTiming::AfterEffectiveFloor {
            Damage { reclaim_on_first_new_floor_root: true, ..Damage::default() }
        } else {
            Damage { skip_hold: true, ..Damage::default() }
        };
        let pc4 = run_delete_then_rewrite_history_with_damage(arm, pc_geometry, pc4_damage);
        if pc4.unsafe_candidate_reuse_count_field >= 1 {
            pc4_hit_any = true;
        }
        println!("{}", emitter.emit_raw(&format!("PC name=pc4 arm={} unsafe_candidate_reuse_count_field={}", arm.label(), pc4.unsafe_candidate_reuse_count_field)));
        run_count += 3;
    }
    let pc5_geometry = Geometry {
        capacity_slots_per_disk: 600,
        object_units: 1,
        fixpoint_cost: 9,
        slots_per_region: 8,
        padding_deletes: 2,
    };
    for arm in all_arms() {
        let pc5 = run_delete_then_rewrite_history_with_damage(arm, pc5_geometry, Damage { zero_reserves: true, ..Damage::default() });
        if pc5.wedged {
            pc5_hit_any = true;
        }
        println!("{}", emitter.emit_raw(&format!("PC name=pc5 arm={} wedged={}", arm.label(), pc5.wedged)));
        run_count += 1;
    }

    // 阳性对照对新历史也要跑：H5 用 k=1、位=首（最简单、最容易命中
    // 的组合，与 H1 用同一组 PC 几何格）；H6 与 H1 完全共用 fill_pool／retry_loop／publish_delete，
    // 按同样的道理重跑一遍。
    let mut pc1_root_write_failure_hit_any = false;
    let mut pc2_root_write_failure_hit_any = false;
    let mut pc4_root_write_failure_hit_any = false;
    let mut pc5_root_write_failure_hit_any = false;
    let mut pc1_rollback_hit_any = false;
    let mut pc2_rollback_hit_any = false;
    let mut pc4_rollback_hit_any = false;
    let mut pc5_rollback_hit_any = false;
    // H4（`CrashPoint::AfterFirstPush`）与 H8（k=1、位=首，与其它历史的阳性对照同一个理由：
    // 最简单、最容易命中的组合）也要跑（阳性对照对 H4、H8
    // 也跑）。H8 的 PC5 复用 H5 探针探出来的 k=3／位=覆——`pc5_geometry` 不落在诊断报的
    // 「n=16×S=4」自推区间里，`LazyFailureInjector` 大概率在填满阶段问零次、原样落回 H5 的
    // 注入点，理由与判定同 H5 的 PC5。
    let mut pc1_crash_mid_push_hit_any = false;
    let mut pc2_crash_mid_push_hit_any = false;
    let mut pc4_crash_mid_push_hit_any = false;
    let mut pc5_crash_mid_push_hit_any = false;
    let mut pc1_fill_phase_injected_hit_any = false;
    let mut pc2_fill_phase_injected_hit_any = false;
    let mut pc4_fill_phase_injected_hit_any = false;
    let mut pc5_fill_phase_injected_hit_any = false;
    for arm in all_arms() {
        let (root_write_failure_baseline, ..) = run_root_write_failure_history(arm, pc_geometry, 1, FailurePosition::First, Damage::default());
        let (root_write_failure_pc1, ..) = run_root_write_failure_history(arm, pc_geometry, 1, FailurePosition::First, Damage { disable_floor_raise: true, ..Damage::default() });
        if root_write_failure_pc1.delete_then_rewrite_outcome_label != root_write_failure_baseline.delete_then_rewrite_outcome_label
            || root_write_failure_pc1.delete_then_rewrite_publish_count != root_write_failure_baseline.delete_then_rewrite_publish_count
        {
            pc1_root_write_failure_hit_any = true;
        }
        println!("{}", emitter.emit_raw(&format!("PC name=pc1-root_write_failure arm={} baseline_q1={}/{} damaged_q1={}/{}", arm.label(), root_write_failure_baseline.delete_then_rewrite_outcome_label, root_write_failure_baseline.delete_then_rewrite_publish_count, root_write_failure_pc1.delete_then_rewrite_outcome_label, root_write_failure_pc1.delete_then_rewrite_publish_count)));

        let (root_write_failure_pc2, ..) = run_root_write_failure_history(arm, pc_geometry, 1, FailurePosition::First, Damage { raw_df: true, ..Damage::default() });
        if root_write_failure_pc2.df_reported_enough_but_write_failed_count >= 1 {
            pc2_root_write_failure_hit_any = true;
        }
        println!("{}", emitter.emit_raw(&format!("PC name=pc2-root_write_failure arm={} df_reported_enough_but_write_failed_count={}", arm.label(), root_write_failure_pc2.df_reported_enough_but_write_failed_count)));

        let pc4_root_write_failure_damage = if arm.reclaim == ReclaimTiming::AfterEffectiveFloor {
            Damage { reclaim_on_first_new_floor_root: true, ..Damage::default() }
        } else {
            Damage { skip_hold: true, ..Damage::default() }
        };
        let (root_write_failure_pc4, ..) = run_root_write_failure_history(arm, pc_geometry, 1, FailurePosition::First, pc4_root_write_failure_damage);
        if root_write_failure_pc4.unsafe_candidate_reuse_count_field >= 1 {
            pc4_root_write_failure_hit_any = true;
        }
        println!("{}", emitter.emit_raw(&format!("PC name=pc4-root_write_failure arm={} unsafe_candidate_reuse_count_field={}", arm.label(), root_write_failure_pc4.unsafe_candidate_reuse_count_field)));

        // k=1／首 在这个几何上对每条臂都会退化（zero_reserves 让 gate1 少两项减项，3 次触动后常常
        // 不用推空就批准）；k=3／位=覆需要推得更久，才撞得到真卡死（见 `positive_control_pc5_wedges_root_write_failure_when_reserves_zero`）。
        let (root_write_failure_pc5, ..) = run_root_write_failure_history(arm, pc5_geometry, 3, FailurePosition::Cover, Damage { zero_reserves: true, ..Damage::default() });
        if root_write_failure_pc5.wedged {
            pc5_root_write_failure_hit_any = true;
        }
        println!("{}", emitter.emit_raw(&format!("PC name=pc5-root_write_failure arm={} wedged={}", arm.label(), root_write_failure_pc5.wedged)));

        let (rollback_baseline, _) = run_rollback_history(arm, pc_geometry, Damage::default());
        let (rollback_pc1, _) = run_rollback_history(arm, pc_geometry, Damage { disable_floor_raise: true, ..Damage::default() });
        if rollback_pc1.delete_then_rewrite_outcome_label != rollback_baseline.delete_then_rewrite_outcome_label
            || rollback_pc1.delete_then_rewrite_publish_count != rollback_baseline.delete_then_rewrite_publish_count
        {
            pc1_rollback_hit_any = true;
        }
        println!("{}", emitter.emit_raw(&format!("PC name=pc1-rollback arm={} baseline_q1={}/{} damaged_q1={}/{}", arm.label(), rollback_baseline.delete_then_rewrite_outcome_label, rollback_baseline.delete_then_rewrite_publish_count, rollback_pc1.delete_then_rewrite_outcome_label, rollback_pc1.delete_then_rewrite_publish_count)));

        let (rollback_pc2, _) = run_rollback_history(arm, pc_geometry, Damage { raw_df: true, ..Damage::default() });
        if rollback_pc2.df_reported_enough_but_write_failed_count >= 1 {
            pc2_rollback_hit_any = true;
        }
        println!("{}", emitter.emit_raw(&format!("PC name=pc2-rollback arm={} df_reported_enough_but_write_failed_count={}", arm.label(), rollback_pc2.df_reported_enough_but_write_failed_count)));

        let pc4_rollback_damage = if arm.reclaim == ReclaimTiming::AfterEffectiveFloor {
            Damage { reclaim_on_first_new_floor_root: true, ..Damage::default() }
        } else {
            Damage { skip_hold: true, ..Damage::default() }
        };
        let (rollback_pc4, _) = run_rollback_history(arm, pc_geometry, pc4_rollback_damage);
        if rollback_pc4.unsafe_candidate_reuse_count_field >= 1 {
            pc4_rollback_hit_any = true;
        }
        println!("{}", emitter.emit_raw(&format!("PC name=pc4-rollback arm={} unsafe_candidate_reuse_count_field={}", arm.label(), rollback_pc4.unsafe_candidate_reuse_count_field)));

        let (rollback_pc5, _) = run_rollback_history(arm, pc5_geometry, Damage { zero_reserves: true, ..Damage::default() });
        if rollback_pc5.wedged {
            pc5_rollback_hit_any = true;
        }
        println!("{}", emitter.emit_raw(&format!("PC name=pc5-rollback arm={} wedged={}", arm.label(), rollback_pc5.wedged)));

        let (crash_mid_push_baseline, ..) = run_crash_mid_push_history(arm, pc_geometry, CrashPoint::AfterFirstPush, Damage::default());
        let (crash_mid_push_pc1, ..) = run_crash_mid_push_history(arm, pc_geometry, CrashPoint::AfterFirstPush, Damage { disable_floor_raise: true, ..Damage::default() });
        if crash_mid_push_pc1.delete_then_rewrite_outcome_label != crash_mid_push_baseline.delete_then_rewrite_outcome_label
            || crash_mid_push_pc1.delete_then_rewrite_publish_count != crash_mid_push_baseline.delete_then_rewrite_publish_count
        {
            pc1_crash_mid_push_hit_any = true;
        }
        println!("{}", emitter.emit_raw(&format!("PC name=pc1-crash_mid_push arm={} baseline_q1={}/{} damaged_q1={}/{}", arm.label(), crash_mid_push_baseline.delete_then_rewrite_outcome_label, crash_mid_push_baseline.delete_then_rewrite_publish_count, crash_mid_push_pc1.delete_then_rewrite_outcome_label, crash_mid_push_pc1.delete_then_rewrite_publish_count)));

        let (crash_mid_push_pc2, ..) = run_crash_mid_push_history(arm, pc_geometry, CrashPoint::AfterFirstPush, Damage { raw_df: true, ..Damage::default() });
        if crash_mid_push_pc2.df_reported_enough_but_write_failed_count >= 1 {
            pc2_crash_mid_push_hit_any = true;
        }
        println!("{}", emitter.emit_raw(&format!("PC name=pc2-crash_mid_push arm={} df_reported_enough_but_write_failed_count={}", arm.label(), crash_mid_push_pc2.df_reported_enough_but_write_failed_count)));

        let pc4_crash_mid_push_damage = if arm.reclaim == ReclaimTiming::AfterEffectiveFloor {
            Damage { reclaim_on_first_new_floor_root: true, ..Damage::default() }
        } else {
            Damage { skip_hold: true, ..Damage::default() }
        };
        let (crash_mid_push_pc4, ..) = run_crash_mid_push_history(arm, pc_geometry, CrashPoint::AfterFirstPush, pc4_crash_mid_push_damage);
        if crash_mid_push_pc4.unsafe_candidate_reuse_count_field >= 1 {
            pc4_crash_mid_push_hit_any = true;
        }
        println!("{}", emitter.emit_raw(&format!("PC name=pc4-crash_mid_push arm={} unsafe_candidate_reuse_count_field={}", arm.label(), crash_mid_push_pc4.unsafe_candidate_reuse_count_field)));

        let (crash_mid_push_pc5, ..) = run_crash_mid_push_history(arm, pc5_geometry, CrashPoint::AfterFirstPush, Damage { zero_reserves: true, ..Damage::default() });
        if crash_mid_push_pc5.wedged {
            pc5_crash_mid_push_hit_any = true;
        }
        println!("{}", emitter.emit_raw(&format!("PC name=pc5-crash_mid_push arm={} wedged={}", arm.label(), crash_mid_push_pc5.wedged)));

        let (fill_phase_injected_baseline, ..) = run_fill_phase_injected_history(arm, pc_geometry, 1, FailurePosition::First, Damage::default());
        let (fill_phase_injected_pc1, ..) = run_fill_phase_injected_history(arm, pc_geometry, 1, FailurePosition::First, Damage { disable_floor_raise: true, ..Damage::default() });
        if fill_phase_injected_pc1.delete_then_rewrite_outcome_label != fill_phase_injected_baseline.delete_then_rewrite_outcome_label
            || fill_phase_injected_pc1.delete_then_rewrite_publish_count != fill_phase_injected_baseline.delete_then_rewrite_publish_count
        {
            pc1_fill_phase_injected_hit_any = true;
        }
        println!("{}", emitter.emit_raw(&format!("PC name=pc1-h8 arm={} baseline_q1={}/{} damaged_q1={}/{}", arm.label(), fill_phase_injected_baseline.delete_then_rewrite_outcome_label, fill_phase_injected_baseline.delete_then_rewrite_publish_count, fill_phase_injected_pc1.delete_then_rewrite_outcome_label, fill_phase_injected_pc1.delete_then_rewrite_publish_count)));

        let (fill_phase_injected_pc2, ..) = run_fill_phase_injected_history(arm, pc_geometry, 1, FailurePosition::First, Damage { raw_df: true, ..Damage::default() });
        if fill_phase_injected_pc2.df_reported_enough_but_write_failed_count >= 1 {
            pc2_fill_phase_injected_hit_any = true;
        }
        println!("{}", emitter.emit_raw(&format!("PC name=pc2-h8 arm={} df_reported_enough_but_write_failed_count={}", arm.label(), fill_phase_injected_pc2.df_reported_enough_but_write_failed_count)));

        let pc4_fill_phase_injected_damage = if arm.reclaim == ReclaimTiming::AfterEffectiveFloor {
            Damage { reclaim_on_first_new_floor_root: true, ..Damage::default() }
        } else {
            Damage { skip_hold: true, ..Damage::default() }
        };
        let (fill_phase_injected_pc4, ..) = run_fill_phase_injected_history(arm, pc_geometry, 1, FailurePosition::First, pc4_fill_phase_injected_damage);
        if fill_phase_injected_pc4.unsafe_candidate_reuse_count_field >= 1 {
            pc4_fill_phase_injected_hit_any = true;
        }
        println!("{}", emitter.emit_raw(&format!("PC name=pc4-h8 arm={} unsafe_candidate_reuse_count_field={}", arm.label(), fill_phase_injected_pc4.unsafe_candidate_reuse_count_field)));

        let (fill_phase_injected_pc5, ..) = run_fill_phase_injected_history(arm, pc5_geometry, 3, FailurePosition::Cover, Damage { zero_reserves: true, ..Damage::default() });
        if fill_phase_injected_pc5.wedged {
            pc5_fill_phase_injected_hit_any = true;
        }
        println!("{}", emitter.emit_raw(&format!("PC name=pc5-h8 arm={} wedged={}", arm.label(), fill_phase_injected_pc5.wedged)));

        run_count += 16;
    }

    println!(
        "{}",
        emitter.emit_raw(&format!(
            "DONE runs={run_count} pc1_hit_any={pc1_hit_any} pc2_hit_any={pc2_hit_any} pc4_hit_any={pc4_hit_any} pc5_hit_any={pc5_hit_any} pc1_root_write_failure_hit_any={pc1_root_write_failure_hit_any} pc2_root_write_failure_hit_any={pc2_root_write_failure_hit_any} pc4_root_write_failure_hit_any={pc4_root_write_failure_hit_any} pc5_root_write_failure_hit_any={pc5_root_write_failure_hit_any} pc1_rollback_hit_any={pc1_rollback_hit_any} pc2_rollback_hit_any={pc2_rollback_hit_any} pc4_rollback_hit_any={pc4_rollback_hit_any} pc5_rollback_hit_any={pc5_rollback_hit_any} pc1_crash_mid_push_hit_any={pc1_crash_mid_push_hit_any} pc2_crash_mid_push_hit_any={pc2_crash_mid_push_hit_any} pc4_crash_mid_push_hit_any={pc4_crash_mid_push_hit_any} pc5_crash_mid_push_hit_any={pc5_crash_mid_push_hit_any} pc1_fill_phase_injected_hit_any={pc1_fill_phase_injected_hit_any} pc2_fill_phase_injected_hit_any={pc2_fill_phase_injected_hit_any} pc4_fill_phase_injected_hit_any={pc4_fill_phase_injected_hit_any} pc5_fill_phase_injected_hit_any={pc5_fill_phase_injected_hit_any} unbounded_budget_diverged_count={unbounded_budget_diverged_count}"
        ))
    );
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests;
