use super::*;

/// 场景：峰值取「第一次不够那一步起」的最大值，最大值恰好落在第一次不够那一步。
/// 预期：序列 0、9、3、2 在第 1 步第一次不够时峰值 9、为正的步数 3、期末 2；从没不够过时峰值没有（none），不当 0 报。
#[test]
fn trajectory_peak_starts_at_the_first_short_step_and_is_none_when_never_short() {
    let series = [0i64, 9, 3, 2];
    assert_eq!(trajectory_of(&series, Some(1)), Trajectory { peak_after_the_first_short: Some(9), positive_steps: 3, final_value: 2 });
    assert_eq!(trajectory_of(&series, None).peak_after_the_first_short, None);
    let mut aggregate = TrajectoryAggregate::empty();
    aggregate.add(trajectory_of(&series, Some(1)));
    aggregate.add(trajectory_of(&series, None));
    assert_eq!((aggregate.peak_maximum, aggregate.peak_minimum, aggregate.positions_never_short), (Some(9), Some(9), 1));
}

/// 场景：注了读故障的那一次比基线恰好多拒 1 次，而且是最后一次请求。
/// 预期：多拒 1 就够「一次挂载全 ENOSPC」成立（门槛是 ≥ 1，不是 > 1）：第一次多拒落在第 4 步、之后发成 0 次。
#[test]
fn exactly_one_extra_rejection_at_the_end_is_already_an_all_enospc_mount() {
    let accepted = RequestRecord {
        accepted: true,
        rejection: None,
        was_judged_short: false,
        empty_publishes: 0,
        empty_publish_blocks: 0,
        publishes_in_the_admission: 1,
        crossed_non_empty_roots: 0,
        states_lost: 0,
        floor_after: 0,
        free_above_the_reserve_after: 3,
        candidate_states_after: 4,
    };
    let rejected = RequestRecord { accepted: false, rejection: Some(RejectionReason::AdmissionBudgetUsedUp), was_judged_short: true, ..accepted.clone() };
    let make_run = |requests: Vec<RequestRecord>| MountRun {
        requests,
        raises: Vec::new(),
        candidate_states_at_the_start: 4,
        cap_active_passes: 0,
        cap_blocked_passes: 0,
        empty_publish_refusals: 0,
        raise_refused_after_an_empty_publish: 0,
        empty_publish_bound_exhausted: 0,
        lowest_free_blocks_in_the_first_raise: None,
        trace: Vec::new(),
    };
    let baseline = make_run(vec![accepted.clone(); 5]);
    let with_fault = make_run(vec![accepted.clone(), accepted.clone(), accepted.clone(), accepted.clone(), rejected]);
    let metrics = metrics_of(&with_fault, &baseline);
    assert_eq!((metrics.extra_rejected, metrics.first_extra_step, metrics.accepted_after_the_first_extra, metrics.all_enospc), (1, 4, 0, 1));
}
