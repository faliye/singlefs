//! 放行判定、用户写与删除的发布、重试循环与它的结局。

use super::*;

// ===== 准入（5.3：一次准入的发布预算 B = 8，按闸读法分派） =====

/// 第二道闸不过时不会单独终止：`admit_request_inner` 里的 gate2 循环只会继续推空，
/// 直到批准、停法叫停、预算做满或卡死——所以没有一条独立的 `EnospcGate2` 出口。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum AdmitOutcome {
    Approved,
    EnospcGate1,
    EnospcStop,
    EnospcBudget,
    Wedged,
}

/// 一次准入判定：`budget_used` 是这次调用之前已经用掉的发布尝试数（同一窗口内多个请求共用）；
/// `other_in_flight` 是窗口内其它已批准、还没封窗的请求占用的槽数。
/// 返回批准与否、这次判定额外消耗的发布尝试数、以及经过的失败注入队列（用于 H5）。
pub(super) struct AdmitResult {
    pub(super) outcome: AdmitOutcome,
    attempts_used: u64,
}

/// 外壳：判之前按臂取 `df`（字节），判完按 Q2a／Q3 的口径记账，再转发给内层判定逻辑。
pub(super) fn admit_request(
    pool: &mut Pool,
    arm: Arm,
    need_slots: u64,
    other_in_flight: u64,
    budget_used: u64,
    failures: impl FnMut(u64) -> bool,
) -> AdmitResult {
    let (gate1_before, snapshot_before) = compute_gate1(pool, arm, other_in_flight);
    let initial_df_bytes = df_bytes(pool, arm, &snapshot_before, gate1_before);
    let need_bytes = need_slots * 16384;
    let result = admit_request_inner(pool, arm, need_slots, other_in_flight, budget_used, failures);
    if result.outcome != AdmitOutcome::Approved && initial_df_bytes >= need_bytes {
        pool.df_reported_enough_but_write_failed_count += 1;
    }
    if result.outcome == AdmitOutcome::EnospcBudget {
        pool.budget_exhausted_hit = true;
    }
    pool.maximum_publish_attempts_seen = pool.maximum_publish_attempts_seen.max(budget_used + result.attempts_used);
    result
}

fn admit_request_inner(
    pool: &mut Pool,
    arm: Arm,
    need_slots: u64,
    other_in_flight: u64,
    budget_used: u64,
    mut failures: impl FnMut(u64) -> bool,
) -> AdmitResult {
    let mut attempts = 0u64;
    let budget = pool.damage.budget_override.unwrap_or(RETRY_BUDGET_PUBLICATIONS);
    let (gate1, _) = compute_gate1(pool, arm, other_in_flight);
    let mut gate1_value = gate1;
    if gate1_value < need_slots {
        if arm.gate.rejudges_first_gate() {
            loop {
                if budget_used + attempts >= budget {
                    return AdmitResult { outcome: AdmitOutcome::EnospcBudget, attempts_used: attempts };
                }
                if stop_policy_says_stop(pool, arm.stop) {
                    return AdmitResult { outcome: AdmitOutcome::EnospcStop, attempts_used: attempts };
                }
                attempts += push_one_empty_publish(pool, arm.reclaim, &mut failures);
                if pool.allocation_failure_hit {
                    return AdmitResult { outcome: AdmitOutcome::Wedged, attempts_used: attempts };
                }
                let (recomputed, _) = compute_gate1(pool, arm, other_in_flight);
                gate1_value = recomputed;
                if gate1_value >= need_slots {
                    break;
                }
            }
        } else {
            return AdmitResult { outcome: AdmitOutcome::EnospcGate1, attempts_used: attempts };
        }
    }
    loop {
        let (_, snapshot) = compute_gate1(pool, arm, other_in_flight);
        let gate2_value = compute_gate2(&snapshot);
        if gate2_value >= need_slots {
            return AdmitResult { outcome: AdmitOutcome::Approved, attempts_used: attempts };
        }
        if budget_used + attempts >= budget {
            return AdmitResult { outcome: AdmitOutcome::EnospcBudget, attempts_used: attempts };
        }
        if stop_policy_says_stop(pool, arm.stop) {
            return AdmitResult { outcome: AdmitOutcome::EnospcStop, attempts_used: attempts };
        }
        attempts += push_one_empty_publish(pool, arm.reclaim, &mut failures);
        if pool.allocation_failure_hit {
            return AdmitResult { outcome: AdmitOutcome::Wedged, attempts_used: attempts };
        }
    }
}

/// 批准之后的写通常不会再失败，但阳性对照 PC5（保留量清零）故意破坏了这份保证——
/// 那种时候这里要老老实实返回 `None`，不许 `.expect()`。`attempt_publish` 失败时已经把
/// `pool.allocation_failure_hit` 置位（Q5a／Q5b 的口径），调用方按这个标志走卡死分支。
pub(super) fn publish_user_write(pool: &mut Pool, reclaim: ReclaimTiming, units: u64) -> Option<(u64, u64)> {
    let floor = pool.newest_persisted_floor();
    match attempt_publish(pool, reclaim, true, floor, None, Some(units), false, None, false) {
        Ok(txg) => Some((txg, pool.next_object_identifier - 1)),
        Err(PublishAttemptFailed) => None,
    }
}

pub(super) fn publish_delete(pool: &mut Pool, reclaim: ReclaimTiming, object_identifier: u64) -> Option<u64> {
    let floor = pool.newest_persisted_floor();
    attempt_publish(pool, reclaim, true, floor, Some(object_identifier), None, false, None, false).ok()
}

// ===== Q1 与轨迹：一次「删了再写」的重试循环 =====

#[derive(Clone, Copy, Debug)]
pub(super) enum RetryOutcome {
    Success { non_empty_publish_count: u64 },
    OverBound { non_empty_publish_count: u64 },
    NotCompleted,
    Wedged,
    Degenerate,
}

/// 重试循环（登记「五、5.4」）：尝试写 `unit_count` 个单元；不成就触动一次（删一个填充对象），
/// 再尝试，最多 `retry_touch_budget(geometry.slots_per_region)` 次触动。
/// `failures` 在每次推空前被问一次，返回真表示这次要注入根槽写失败（R8／H5 用）。
/// `initial_non_empty_publish_count` 与 `initial_touches`：H4a／H5 的前缀在进这个循环之前已经先做了
/// 若干次「触动（中间不尝试）」（登记「五、5.4」：H4a／H5 的前缀是「连续 3 次触动，中间不尝试」，
/// 与 H1 从第一次就「尝试再触动」交替着走不同）——这两个参数把那几次已经发生的非空发布计进来，
/// H1 传 0、0 与不带前缀完全等价。
#[allow(clippy::too_many_arguments, reason = "入参对应登记「五、5.4」重试循环的各个输入（池、臂、几何、填充队列、单元数、前缀里已有的发布数与触动数、失败注入），拆结构体不会少传哪一项")]
pub(super) fn retry_loop(
    pool: &mut Pool,
    arm: Arm,
    geometry: Geometry,
    filler_queue: &mut Vec<u64>,
    unit_count: u64,
    initial_non_empty_publish_count: u64,
    initial_touches: u64,
    mut failures: impl FnMut(u64) -> bool,
) -> RetryOutcome {
    let need = unit_count * 2;
    let budget = retry_touch_budget(geometry.slots_per_region);
    let mut non_empty_publish_count = initial_non_empty_publish_count;
    let mut touches = initial_touches;
    loop {
        let result = admit_request(pool, arm, need, 0, 0, &mut failures);
        match result.outcome {
            AdmitOutcome::Approved => {
                return match publish_user_write(pool, arm.reclaim, unit_count) {
                    Some(_) => {
                        if non_empty_publish_count <= 3 {
                            RetryOutcome::Success { non_empty_publish_count }
                        } else {
                            RetryOutcome::OverBound { non_empty_publish_count }
                        }
                    }
                    None => RetryOutcome::Wedged,
                };
            }
            AdmitOutcome::Wedged => return RetryOutcome::Wedged,
            AdmitOutcome::EnospcGate1 | AdmitOutcome::EnospcStop | AdmitOutcome::EnospcBudget => {
                if touches >= budget {
                    return RetryOutcome::NotCompleted;
                }
                match filler_queue.pop() {
                    Some(object_identifier) => {
                        if publish_delete(pool, arm.reclaim, object_identifier).is_none() {
                            return RetryOutcome::Wedged;
                        }
                        non_empty_publish_count += 1;
                        touches += 1;
                    }
                    None => return RetryOutcome::Degenerate,
                }
            }
        }
    }
}
