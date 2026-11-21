//! 一趟的汇总与结局标签、填充池、可读根的不同状态数、汇总行的拼装、删后重写历史。

use super::*;

// ===== 六：一次「跑」的汇总（Q1–Q8） =====

#[derive(Clone, Debug)]
pub(super) struct RunSummary {
    pub(super) arm: String,
    pub(super) is_real_baseline: bool,
    pub(super) reclaim_label: String,
    pub(super) stop_label: String,
    pub(super) history: String,
    pub(super) capacity: u64,
    pub(super) object_units: u64,
    pub(super) fixpoint_cost: u64,
    pub(super) slots_per_region: u64,
    pub(super) padding_deletes: u64,
    pub(super) delete_then_rewrite_outcome_label: String,
    pub(super) delete_then_rewrite_publish_count: i64,
    pub(super) df_reported_enough_but_write_failed_count: u64,
    pub(super) admission_passed_but_allocation_failed_count: u64,
    pub(super) budget_was_binding: bool,
    pub(super) maximum_publish_attempts_seen_field: u64,
    pub(super) allocation_failure_hit_field: bool,
    pub(super) wedged: bool,
    pub(super) unsafe_candidate_reuse_count_field: u64,
    pub(super) retained_states_dropped_below_minimum: u64,
    pub(super) filler_objects_written_in_fill_phase: u64,
}

pub(super) fn retry_outcome_label(outcome: RetryOutcome) -> (String, i64) {
    match outcome {
        RetryOutcome::Success { non_empty_publish_count } => {
            ("达标".to_string(), i64::try_from(non_empty_publish_count).unwrap_or(i64::MAX))
        }
        RetryOutcome::OverBound { non_empty_publish_count } => {
            ("越界".to_string(), i64::try_from(non_empty_publish_count).unwrap_or(i64::MAX))
        }
        RetryOutcome::NotCompleted => ("未成".to_string(), -1),
        RetryOutcome::Wedged => ("卡死".to_string(), -2),
        RetryOutcome::Degenerate => ("退化".to_string(), -3),
    }
}

/// 填满阶段：写对象 X（`geometry.object_units` 个单元），再一个接一个写 1 单元填充对象，
/// 直到某次填充写报 ENOSPC。返回 X 的对象号与填充对象队列（先进先出即最先写、槽号最低）。
/// `failures`（H8 用）：填满阶段每一次自推空发布之前被问一次，参数是
/// `last_persisted_txg`——H1／H5／H6 一律传 `&mut |_last_txg| false`（不注入），
/// 只有 `run_fill_phase_injected_history` 传一个真的 `LazyFailureInjector`。内部两处
/// `admit_request` 调用都要 `&mut *failures` 重新借用——`failures` 是个 `&mut dyn FnMut(u64) -> bool`，
/// 不能被 `admit_request` 按值吃掉之后还留着给下一轮循环用。
pub(super) fn fill_pool(pool: &mut Pool, arm: Arm, geometry: Geometry, failures: &mut dyn FnMut(u64) -> bool) -> (u64, Vec<u64>) {
    let deleted_reference_object_admission_result = admit_request(pool, arm, geometry.object_units * 2, 0, 0, &mut *failures);
    assert_eq!(
        deleted_reference_object_admission_result.outcome,
        AdmitOutcome::Approved,
        "起点状态之后立刻写 X 应当总是过闸（池几乎全空）"
    );
    let (_, deleted_reference_object_identifier) = publish_user_write(pool, arm.reclaim, geometry.object_units)
        .expect("起点状态之后立刻写 X：池几乎全空，唯一会失败的原因是分配记账有 bug");
    pool.reset_filler_objects_written_counter();
    let mut fillers = Vec::new();
    let mut filler_writes = 0u64;
    loop {
        // 填充对象每个占 2 槽，物理上最多写 capacity/2 个；超过这个数还没停就是分配记账有 bug
        // （比如新分配的槽没被标记 Allocated），不是「池子恰好很大」。
        assert!(
            filler_writes <= pool.capacity_slots / 2 + 1,
            "fill_pool：写了 {filler_writes} 个填充对象还没被拒绝，容量只有 {} 槽——分配记账像是有 bug",
            pool.capacity_slots
        );
        let result = admit_request(pool, arm, 2, 0, 0, &mut *failures);
        match result.outcome {
            AdmitOutcome::Approved => match publish_user_write(pool, arm.reclaim, 1) {
                Some((_, filler_object_identifier)) => {
                    fillers.push(filler_object_identifier);
                    pool.filler_objects_written_so_far += 1;
                    filler_writes += 1;
                }
                // PC5（保留量清零）会让「闸批准了、实际分配却失败」真的发生；填满阶段把它
                // 当成这一格已经写满处理，Q5a／Q5b 已经在 attempt_publish 里记过。
                None => break,
            },
            AdmitOutcome::EnospcGate1 | AdmitOutcome::EnospcStop | AdmitOutcome::EnospcBudget | AdmitOutcome::Wedged => break,
        }
    }
    (deleted_reference_object_identifier, fillers)
}

impl Pool {
    fn reset_filler_objects_written_counter(&mut self) {
        self.filler_objects_written_so_far = 0;
    }
}

/// Q7 的「状态数」：候选集（持久∧有效∧txg≥F_生效 的根）里按 txg 去重的根数——
/// 登记原话按「最近非空发布」归并连续的空发布进同一个状态；本文件的简化读法直接数候选集里
/// 的持久根条数（每条持久根本身就是一个可退到的落点），比原定义更严格（更容易记违例）。
pub(super) fn distinct_valid_states(pool: &Pool) -> u64 {
    let effective = pool.effective_rollback_floor();
    pool.readable_roots()
        .filter(|root| root.txg >= effective && !pool.is_abandoned(root))
        .count() as u64
}

/// 把一次跑的公共字段（「六」Q1–Q8 里不专属某一种历史的那些）拼成 `RunSummary`。H1／H5／H6
/// 三个历史构造器都走这个函数，字段的取法只写一处（`.claude/singlefs-ai-sop/rules/code-discipline.md`
/// 「重复要生成，不许手抄」）。
#[allow(clippy::too_many_arguments, reason = "RunSummary 本身就是「六」那张表逐列摊开，拆结构体不会减少要传的信息")]
pub(super) fn build_run_summary(
    arm: Arm,
    geometry: Geometry,
    history: String,
    pool: &Pool,
    outcome_label: String,
    delete_then_rewrite_publish_count: i64,
    wedged: bool,
    retained_states_dropped_below_minimum: u64,
) -> RunSummary {
    RunSummary {
        arm: arm.label(),
        is_real_baseline: arm.is_real_baseline(),
        reclaim_label: arm.reclaim.label().to_string(),
        stop_label: arm.stop.label().to_string(),
        history,
        capacity: geometry.capacity_slots_per_disk,
        object_units: geometry.object_units,
        fixpoint_cost: geometry.fixpoint_cost,
        slots_per_region: geometry.slots_per_region,
        padding_deletes: geometry.padding_deletes,
        delete_then_rewrite_outcome_label: outcome_label,
        delete_then_rewrite_publish_count,
        df_reported_enough_but_write_failed_count: pool.df_reported_enough_but_write_failed_count,
        admission_passed_but_allocation_failed_count: 0,
        budget_was_binding: pool.budget_exhausted_hit,
        maximum_publish_attempts_seen_field: pool.maximum_publish_attempts_seen,
        allocation_failure_hit_field: pool.allocation_failure_hit,
        wedged,
        unsafe_candidate_reuse_count_field: pool.unsafe_candidate_reuse_count,
        retained_states_dropped_below_minimum,
        filler_objects_written_in_fill_phase: pool.filler_objects_written_so_far,
    }
}

/// H1：填满、垫片、删 X、重试循环（登记「五、5.4」）。`damage` 非默认时用于阳性对照（5.6）。
pub(super) fn run_delete_then_rewrite_history_with_damage(arm: Arm, geometry: Geometry, damage: Damage) -> RunSummary {
    let mut pool = Pool::seed(geometry);
    pool.damage = damage;
    let (deleted_reference_object_identifier, mut fillers) = fill_pool(&mut pool, arm, geometry, &mut |_last_txg| false);
    for _ in 0..geometry.padding_deletes {
        if let Some(filler_object_identifier) = fillers.pop() {
            publish_delete(&mut pool, arm.reclaim, filler_object_identifier);
        }
    }
    publish_delete(&mut pool, arm.reclaim, deleted_reference_object_identifier);
    let states_before_retry = distinct_valid_states(&pool);
    let outcome = retry_loop(&mut pool, arm, geometry, &mut fillers, geometry.object_units, 0, 0, |_last_txg| false);
    let (outcome_label, delete_then_rewrite_publish_count) = retry_outcome_label(outcome);
    let states_after_retry = distinct_valid_states(&pool);
    let retained_states_dropped_below_minimum = u64::from(states_before_retry >= MINIMUM_RETAINED_STATES && states_after_retry < MINIMUM_RETAINED_STATES);
    build_run_summary(
        arm,
        geometry,
        "H1".to_string(),
        &pool,
        outcome_label,
        delete_then_rewrite_publish_count,
        matches!(outcome, RetryOutcome::Wedged),
        retained_states_dropped_below_minimum,
    )
}

pub(super) fn run_delete_then_rewrite_history(arm: Arm, geometry: Geometry) -> RunSummary {
    run_delete_then_rewrite_history_with_damage(arm, geometry, Damage::default())
}

#[cfg(test)]
#[path = "run_summaries_and_fill_tests.rs"]
mod tests;
