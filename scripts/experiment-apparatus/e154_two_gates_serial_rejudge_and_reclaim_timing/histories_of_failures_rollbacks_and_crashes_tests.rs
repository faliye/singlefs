use super::*;

/// R8「切换的写行发布的根也算在连续失败里」：k=1、位=首在一个真的需要推空的几何点上必须真正
/// 触发至少一次实例切换（不是「退化」——这里手选了一个容量小、S 小、临近满盘的格，确保 3 次
/// 触动之后写 Y 一定要推空）。钉绝对值：`injected` 必须恰好等于 1（不多不少，多了说明窗口
/// 没有在命中之后关掉，少了说明这次准入里没有走到该走的推空）。
#[test]
fn run_root_write_failure_history_with_one_failure_and_first_position_injects_exactly_one_failure_and_switches() {
    let geometry = Geometry { capacity_slots_per_disk: 600, object_units: 16, fixpoint_cost: 4, slots_per_region: 4, padding_deletes: 0 };
    let arm = real_baseline_arm();
    let (summary, injected, switches_triggered, recovery_publishes_until_covered) =
        run_root_write_failure_history(arm, geometry, 1, FailurePosition::First, Damage::default());
    assert_eq!(injected, 1, "k=1／首：窗口只要 1 次失败，多问或少问都不对");
    assert!(switches_triggered >= 1, "根槽写失败必须按 R8 触发至少一次实例切换");
    assert!(recovery_publishes_until_covered >= 1, "覆盖两块盘至少要 1 次根槽写尝试");
    assert_ne!(summary.delete_then_rewrite_outcome_label, "退化", "这一格本该真的走到推空，不该退化");
}

/// H6「回退之后隔离了一批槽再写」：真实基线在一个能回退成功的几何点上，隔离槽数必须 > 0——
/// 这正是登记 H6 那句话本身要核的事实（不是摆设：`perform_rollback` 若漏掉「只被被抛弃根引用」
/// 这个条件，隔离数会变成 0 或过大，这条断言都能看出来）。
#[test]
fn run_rollback_history_isolates_slots_after_rollback() {
    let geometry = Geometry { capacity_slots_per_disk: 4000, object_units: 16, fixpoint_cost: 4, slots_per_region: 8, padding_deletes: 2 };
    let arm = real_baseline_arm();
    let (summary, isolated_slots_after_rollback) = run_rollback_history(arm, geometry, Damage::default());
    assert_ne!(summary.delete_then_rewrite_outcome_label, "退化", "这一格本该回退成功（不是回退够不着）");
    assert!(isolated_slots_after_rollback > 0, "回退之后应当隔离了一批只被被抛弃根引用的槽");
}

// ===== H4a／H4b（第四段）：推空发布之中崩溃再重开 =====

/// H4a 与 H4b 在本模型里产生同一个 `RunSummary`（除 `history` 标签外）——见
/// `run_crash_mid_push_history` 头部 ⚠️：`attempt_publish` 是一次性完成分配 + 写根，
/// 崩在「根持久之后」与崩在「下一次已分配、根还没持久」在本模型能表达的粒度上是同一个状态。
/// 这条断言把「先证明等价，再当等价留档」的等价性钉死，不靠肉眼比对产物。
#[test]
fn h4a_and_h4b_crash_points_are_observationally_equivalent_in_this_model() {
    let geometry = Geometry { capacity_slots_per_disk: 600, object_units: 1, fixpoint_cost: 4, slots_per_region: 4, padding_deletes: 0 };
    let arm = Arm { gate: GateReading::SerialRejudge, reclaim: ReclaimTiming::AfterEffectiveFloor, stop: StopPolicy::FillBudget };
    let (after_first_push_summary, after_first_push_self_pushes, after_first_push_recovery_publishes) = run_crash_mid_push_history(arm, geometry, CrashPoint::AfterFirstPush, Damage::default());
    let (before_second_push_summary, before_second_push_self_pushes, before_second_push_recovery_publishes) = run_crash_mid_push_history(arm, geometry, CrashPoint::BeforeSecondPush, Damage::default());
    assert_eq!(after_first_push_summary.delete_then_rewrite_outcome_label, "达标", "这一格本该真的崩到、重开、写成——不是退化或卡死");
    assert_eq!(after_first_push_summary.delete_then_rewrite_outcome_label, before_second_push_summary.delete_then_rewrite_outcome_label);
    assert_eq!(after_first_push_summary.delete_then_rewrite_publish_count, before_second_push_summary.delete_then_rewrite_publish_count);
    assert_eq!(after_first_push_summary.wedged, before_second_push_summary.wedged);
    assert_eq!(after_first_push_summary.unsafe_candidate_reuse_count_field, before_second_push_summary.unsafe_candidate_reuse_count_field);
    assert_eq!(after_first_push_summary.filler_objects_written_in_fill_phase, before_second_push_summary.filler_objects_written_in_fill_phase);
    assert_eq!(after_first_push_self_pushes, before_second_push_self_pushes);
    assert_eq!(after_first_push_self_pushes, 1, "两个崩溃点在本模型里都恰好在 1 次自推空发布持久之后崩");
    assert_eq!(after_first_push_recovery_publishes, before_second_push_recovery_publishes);
}

/// H4a／H4b 的退化条款：这次准入根本不用推空就过闸（第一道闸不重判）的臂上，崩溃点不存在。
#[test]
fn run_crash_mid_push_history_degenerates_when_gate_never_self_pushes() {
    let geometry = Geometry { capacity_slots_per_disk: 4000, object_units: 16, fixpoint_cost: 4, slots_per_region: 8, padding_deletes: 2 };
    let arm = Arm { gate: GateReading::Serial, reclaim: ReclaimTiming::AfterEffectiveFloor, stop: StopPolicy::StopAtTarget };
    let (summary, self_pushes_before_crash, recovery_publishes_after_reopen) = run_crash_mid_push_history(arm, geometry, CrashPoint::AfterFirstPush, Damage::default());
    assert_eq!(summary.delete_then_rewrite_outcome_label, "退化", "串 这条臂的第一道闸不重判，永远走不到自推空发布，崩溃点不存在");
    assert_eq!(self_pushes_before_crash, 0);
    assert_eq!(recovery_publishes_after_reopen, 0);
}

// ===== H8（第四段）：注入点改到填满阶段 =====

/// 真实基线在这个几何点上，填满阶段自己确实自推了空发布——`run_fill_phase_injected_history` 的注入
/// 打在填满阶段（`fill_phase_self_pushes` > 0），不是退回 H5 的注入点。数字是这个几何 + k=1
/// + 位=首下的实测值（见登记「十二、修订」第四段）。
#[test]
fn run_fill_phase_injected_history_injects_during_fill_phase_for_real_baseline() {
    let geometry = Geometry { capacity_slots_per_disk: 600, object_units: 1, fixpoint_cost: 4, slots_per_region: 4, padding_deletes: 0 };
    let arm = real_baseline_arm();
    let (summary, injected, _switches, _recovery, fill_phase_self_pushes) =
        run_fill_phase_injected_history(arm, geometry, 1, FailurePosition::First, Damage::default());
    assert_eq!(injected, 1, "k=1：窗口只要 1 次失败");
    assert_eq!(fill_phase_self_pushes, 4, "这个几何点上填满阶段自己推了 4 次空发布，注入应打在其中第 1 次");
    assert_ne!(summary.delete_then_rewrite_outcome_label, "退化", "注入已经打在填满阶段，这一格不该退化");
}

/// 头部结论：真实基线（串重判 × G7 × 止）在 H5 上因为「填满阶段自己把 F 抬走」而退化的那个
/// 具体几何点（登记「十二、修订」第三段诊断第 2 类原文实测），H8 上不再退化——这是
/// `run_fill_phase_injected_history` 整个改动要解决的那个问题，钉成一条对照断言（H5 与 H8 用同一个
/// k=1、位=首，唯一变量是注入点）。
#[test]
fn fill_phase_injection_eliminates_degeneration_that_original_injection_point_could_not_reach() {
    let geometry = Geometry { capacity_slots_per_disk: 4000, object_units: 1, fixpoint_cost: 4, slots_per_region: 4, padding_deletes: 0 };
    let arm = real_baseline_arm();
    let (original_injection_point_summary, ..) = run_root_write_failure_history(arm, geometry, 1, FailurePosition::First, Damage::default());
    let (fill_phase_injected_summary, ..) = run_fill_phase_injected_history(arm, geometry, 1, FailurePosition::First, Damage::default());
    assert_eq!(original_injection_point_summary.delete_then_rewrite_outcome_label, "退化", "H5 在这一格上退化（注入点从没被这次准入的推空碰到）");
    assert_eq!(fill_phase_injected_summary.delete_then_rewrite_outcome_label, "达标", "H8 把注入点挪到填满阶段之后，这一格不再退化");
}
