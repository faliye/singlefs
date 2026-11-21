//! 根写失败、填充期注入、回滚、崩溃重开这几类历史的整趟。

use super::*;

/// H5-k-位（登记「五、5.4」）：同 H4a 的前缀——填满、垫片、删 X、连续 3 次触动（中间不尝试，
/// 与 H1 的「尝试再触动」交替不同）；这次准入里（尝试写 Y）连续 k 次根槽写失败，位=首／覆
/// 决定失败窗口从第几次根槽写算起；失败按 R8 走实例切换；之后正常重试循环（窗口过去之后
/// `FailureInjector` 恒返回假，不再注入）。
///
/// 返回 `(RunSummary, injected, switches_triggered, recovery_publishes_until_covered)`：
/// `injected` 是 `FailureInjector` 真正问到的失败次数（< k 时这一格按登记记「退化」，
/// `delete_then_rewrite_outcome_label` 已经是 "退化"，调用方不用另判）；`switches_triggered` 是
/// `pool.switches_so_far`（诊断量，含级联失败触发的中间几次）；`recovery_publishes_until_covered`
/// 是 Q4 的粗口径——从这次准入开始到两块盘都覆盖新 F 为止，一共发生了几次根槽写尝试（含失败的）。
/// ⚠️ **这不是登记「六」Q4 要的 `D_acct`／`D_alloc` 两个分量**：那两个数要逐次尝试（ROW 级）才
/// 分得开（哪一次持久之后记账先认、哪一次之后分配器才真能发），本文件没有实现 ROW 明细
/// （见文件头「输出体量说明」），所以只报"覆盖总共花了几次尝试"这一个合并数，在实验页与报告里
/// 注明是粗口径。
pub(super) fn run_root_write_failure_history(
    arm: Arm,
    geometry: Geometry,
    consecutive_root_write_failure_count: u64,
    position: FailurePosition,
    damage: Damage,
) -> (RunSummary, u64, u64, u64) {
    let history_label = format!("H5-{consecutive_root_write_failure_count}-{}", position.label());
    let mut pool = Pool::seed(geometry);
    pool.damage = damage;
    let (deleted_reference_object_identifier, mut fillers) = fill_pool(&mut pool, arm, geometry, &mut |_last_txg| false);
    for _ in 0..geometry.padding_deletes {
        if let Some(filler_object_identifier) = fillers.pop() {
            publish_delete(&mut pool, arm.reclaim, filler_object_identifier);
        }
    }
    publish_delete(&mut pool, arm.reclaim, deleted_reference_object_identifier);

    // 连续 3 次触动，中间不尝试。
    let mut prefix_touches = 0u64;
    let mut degenerate = false;
    for _ in 0..3 {
        match fillers.pop() {
            Some(filler_object_identifier) => {
                if publish_delete(&mut pool, arm.reclaim, filler_object_identifier).is_none() {
                    degenerate = true;
                    break;
                }
                prefix_touches += 1;
            }
            None => {
                degenerate = true;
                break;
            }
        }
    }
    if degenerate {
        let summary = degenerate_or_wedged_summary(arm, geometry, history_label, &pool);
        return (summary, 0, pool.switches_so_far, 0);
    }

    let window_start = match position {
        FailurePosition::First => 1,
        FailurePosition::Cover => cover_push_index(&pool),
    };
    let switches_before = pool.switches_so_far;
    let pushes_before = pool.pushes_so_far;
    let mut injector = FailureInjector::new(window_start, consecutive_root_write_failure_count);
    let states_before_retry = distinct_valid_states(&pool);
    let outcome = {
        let mut failures = |last_txg| injector.next(last_txg);
        retry_loop(&mut pool, arm, geometry, &mut fillers, geometry.object_units, prefix_touches, prefix_touches, &mut failures)
    };
    let states_after_retry = distinct_valid_states(&pool);
    let retained_states_dropped_below_minimum = u64::from(states_before_retry >= MINIMUM_RETAINED_STATES && states_after_retry < MINIMUM_RETAINED_STATES);

    if injector.injected < consecutive_root_write_failure_count {
        // 这次准入里根槽写次数不够，k 次失败没有全部问到：登记 H4a／H4b／H5 的退化条款
        // （除非是真卡死——`degenerate_or_wedged_summary` 先问 `pool.allocation_failure_hit`）。
        let summary = degenerate_or_wedged_summary(arm, geometry, history_label, &pool);
        return (summary, injector.injected, pool.switches_so_far - switches_before, 0);
    }

    let (outcome_label, delete_then_rewrite_publish_count) = retry_outcome_label(outcome);
    let recovery_publishes_until_covered = pool.pushes_so_far - pushes_before;
    let summary = build_run_summary(
        arm,
        geometry,
        history_label,
        &pool,
        outcome_label,
        delete_then_rewrite_publish_count,
        matches!(outcome, RetryOutcome::Wedged),
        retained_states_dropped_below_minimum,
    );
    (summary, injector.injected, pool.switches_so_far - switches_before, recovery_publishes_until_covered)
}

// ===== H8：注入点改成填满阶段自推的空发布（登记「十二、修订」第三段第 7 条第 2 类诊断，
// 第四段；不是登记「五」原文的一种历史，是 H5 的一个变体） =====

/// H8：与 H5 相同的 k ∈ {1,2,3} × 位 ∈ {首,覆}，区别只在注入点——不等「删后写回」那次准入里
/// 的推空，而是打在**填满阶段为抬 F 自己推的那几次空发布**上（第三段诊断：P-止 在「填满」阶段
/// 自身推了 2 次空发布把 F 从 0 抬到 1949 那一类，串重判-G7-止 在 H5 上因此有 73% 的格从没被
/// H5 的注入点碰到，退化）。用同一个 `LazyFailureInjector`，`window_start` 在第一次真的发生
/// 自推空发布时现算——若填满阶段全程一次都没自推，注入器在填满阶段问零次，`call_index` 保持
/// 0，之后把同一个注入器原样带进「删后写回」那次准入的 3 次触动之后的 `retry_loop`，
/// 那里第一次自推就会把 `window_start` 现算出来——等价于原样落回 H5 的注入点，满足
/// 「填满阶段没有自推空发布的格，照 H5 原样注入」。
///
/// 返回 `(RunSummary, injected, switches_so_far, recovery_publishes_until_covered, fill_phase_self_pushes)`：
/// 最后一个是诊断量——填满阶段里真的发生过几次自推空发布（> 0 就说明这一格的注入确实打在了
/// 填满阶段，= 0 就说明这一格退回了 H5 的注入点，两者都不算错，但报告要分清是哪一种，见登记
/// 「十二、修订」第四段）。`recovery_publishes_until_covered` 从 `fill_pool` 开始算（不是从
/// 3 次触动之后算）：注入可能发生在填满阶段，覆盖两块盘所花的总发布尝试数要从注入可能发生的
/// 最早点起算，否则会漏掉填满阶段里那部分。
pub(super) fn run_fill_phase_injected_history(
    arm: Arm,
    geometry: Geometry,
    consecutive_root_write_failure_count: u64,
    position: FailurePosition,
    damage: Damage,
) -> (RunSummary, u64, u64, u64, u64) {
    let history_label = format!("H8-{consecutive_root_write_failure_count}-{}", position.label());
    let mut pool = Pool::seed(geometry);
    pool.damage = damage;
    let mut injector = LazyFailureInjector::new(position, consecutive_root_write_failure_count);
    let pushes_before_fill = pool.pushes_so_far;
    let (deleted_reference_object_identifier, mut fillers) = {
        let mut failures = |last_txg| injector.next(last_txg);
        fill_pool(&mut pool, arm, geometry, &mut failures)
    };
    let fill_phase_self_pushes = pool.pushes_so_far - pushes_before_fill;

    for _ in 0..geometry.padding_deletes {
        if let Some(filler_object_identifier) = fillers.pop() {
            publish_delete(&mut pool, arm.reclaim, filler_object_identifier);
        }
    }
    publish_delete(&mut pool, arm.reclaim, deleted_reference_object_identifier);

    // 连续 3 次触动，中间不尝试（与 H5 同一前缀）。
    let mut prefix_touches = 0u64;
    let mut degenerate = false;
    for _ in 0..3 {
        match fillers.pop() {
            Some(filler_object_identifier) => {
                if publish_delete(&mut pool, arm.reclaim, filler_object_identifier).is_none() {
                    degenerate = true;
                    break;
                }
                prefix_touches += 1;
            }
            None => {
                degenerate = true;
                break;
            }
        }
    }
    if degenerate {
        let summary = degenerate_or_wedged_summary(arm, geometry, history_label, &pool);
        return (summary, injector.injected, pool.switches_so_far, 0, fill_phase_self_pushes);
    }

    let switches_before = pool.switches_so_far;
    let states_before_retry = distinct_valid_states(&pool);
    let outcome = {
        let mut failures = |last_txg| injector.next(last_txg);
        retry_loop(&mut pool, arm, geometry, &mut fillers, geometry.object_units, prefix_touches, prefix_touches, &mut failures)
    };
    let states_after_retry = distinct_valid_states(&pool);
    let retained_states_dropped_below_minimum = u64::from(states_before_retry >= MINIMUM_RETAINED_STATES && states_after_retry < MINIMUM_RETAINED_STATES);

    if injector.injected < consecutive_root_write_failure_count {
        // 窗口没走完——既没在填满阶段问够 k 次，也没在「删后写回」那次准入里问够（可能连
        // window_start 都没算出来，说明这次运行里一次自推空发布都没发生）：登记「退化」，
        // 与 H4a／H4b／H5 同一形态。
        let summary = degenerate_or_wedged_summary(arm, geometry, history_label, &pool);
        return (summary, injector.injected, pool.switches_so_far - switches_before, 0, fill_phase_self_pushes);
    }

    let (outcome_label, delete_then_rewrite_publish_count) = retry_outcome_label(outcome);
    let recovery_publishes_until_covered = pool.pushes_so_far - pushes_before_fill;
    let summary = build_run_summary(
        arm,
        geometry,
        history_label,
        &pool,
        outcome_label,
        delete_then_rewrite_publish_count,
        matches!(outcome, RetryOutcome::Wedged),
        retained_states_dropped_below_minimum,
    );
    (summary, injector.injected, pool.switches_so_far - switches_before, recovery_publishes_until_covered, fill_phase_self_pushes)
}

// ===== H6：管理员回退之后隔离了一批槽再写（登记「五、5.4」） =====

/// 管理员回退（D23（journal 的角色与格式） 已定项 14）：取新实例；`pool.slots`／`current_fixpoint_slots`／
/// `current_instance_table_slots` 从 `rollback_target` 的快照载入；影子账按窄读法（
/// D23（journal 的角色与格式） 已定项 14 正文「按主语『被抛弃时间线的根』读，仍被有效根引用的槽不在其内」）
/// 隔离「只被被抛弃根引用、不被候选根引用、且在 `rollback_target` 自己的快照里不是 Allocated」的槽；
/// 回退行 `(rollback_target.instance, rollback_target.txg, 0, is_rollback=true)` 写进实例表；回退那次发布非空（R9：
/// 它改了用户可见状态），带 F_生效（回退候选集已要求 `rollback_target.txg ≥ F_生效`，回退本身不改变 F）。
/// 返回这次回退 + 暖机一共消耗的发布尝试数；回退那次发布自己真的分配不到固定点 + 实例表槽
/// （近满盘，Q5a／Q5b 那类真卡死，不是 H6 故意注入的故障）时返回 `None`——回退不保证总能成功，
/// 这不是登记要它测的故障历史，但模型不能假装它不会发生（冒烟跑撞到过，见登记
/// 「十二、修订」）。
///
/// ⚠️ **简化，不是登记的全部**：`pool.object_slots`（对象号 → 槽号）不按 `rollback_target` 重新对齐——
/// 本模型只追踪"当前还有哪些填充对象未删"这一件事（`fillers` 队列），回退之后可能删除的对象
/// 已经从队列与 `object_slots` 里一起弹出，之后的 `retry_loop` 只会碰未删过的对象，不会再引用
/// 已经不存在的对象号，所以这处简化不影响 Q9／Q6／Q7 的计算；但它意味着本函数不模拟"对象在
/// 回退之后重新出现"这件事（H6 的问法也没有问这个）。
fn perform_rollback(pool: &mut Pool, reclaim: ReclaimTiming, rollback_target: &RootRecord) -> Option<u64> {
    let effective_floor = pool.effective_rollback_floor();
    let abandoned_txgs: Vec<u64> = pool
        .readable_roots()
        .filter(|root| root.txg > rollback_target.txg)
        .map(|root| root.txg)
        .collect();
    let candidate_txgs: Vec<u64> = pool
        .readable_roots()
        .filter(|root| root.txg <= rollback_target.txg && root.txg >= effective_floor)
        .map(|root| root.txg)
        .collect();
    // 先只读、把要隔离的槽号收集成一份独立的列表，再统一写 `pool.isolated`——`snapshot_of` 借用
    // 的是 `pool.readable_roots()`，与写 `pool.isolated` 不能同时借用同一个 `pool`。
    let snapshot_of = |txg: u64| -> Option<Rc<Vec<SlotState>>> {
        pool.readable_roots().find(|root| root.txg == txg).map(|root| root.slots_snapshot.clone())
    };
    let capacity = pool.slots.len();
    let mut slots_to_isolate = Vec::new();
    for index in 0..capacity {
        let referenced_by_abandoned = abandoned_txgs.iter().any(|&txg| {
            snapshot_of(txg).is_some_and(|snapshot| matches!(snapshot.get(index), Some(SlotState::Allocated)))
        });
        if !referenced_by_abandoned {
            continue;
        }
        let referenced_by_candidate = candidate_txgs.iter().any(|&txg| {
            snapshot_of(txg).is_some_and(|snapshot| matches!(snapshot.get(index), Some(SlotState::Allocated)))
        });
        if referenced_by_candidate {
            continue;
        }
        let allocated_in_rollback_target = matches!(rollback_target.slots_snapshot.get(index), Some(SlotState::Allocated));
        if allocated_in_rollback_target {
            continue;
        }
        slots_to_isolate.push(index);
    }
    for index in slots_to_isolate {
        pool.isolated[index] = true;
    }

    pool.slots = (*rollback_target.slots_snapshot).clone();
    pool.current_fixpoint_slots = (*rollback_target.fixpoint_slots_snapshot).clone();
    pool.current_instance_table_slots = (*rollback_target.instance_table_slots_snapshot).clone();

    pool.instance_table.push(InstanceRow {
        instance: rollback_target.instance,
        selected_root_txg: rollback_target.txg,
        applied_transaction_high_water: 0,
        is_rollback: true,
    });
    pool.instance += 1;

    // 回退那次发布不在故障历史范围内（H6 不注入根槽写失败），但它和任何发布一样要真的分配得到
    // 固定点 + 2 个实例表槽——近满盘时这个分配可以真的失败（`attempt_publish` 已经把
    // `pool.allocation_failure_hit` 置位、按 Q5a／Q5b 的口径处理），不能假定它总是成功。
    let rollback_floor = pool.effective_rollback_floor();
    let rows = pool.instance_table.clone();
    let rollback_txg = match attempt_publish(pool, reclaim, true, rollback_floor, None, None, true, Some(rows), false) {
        Ok(txg) => txg,
        Err(PublishAttemptFailed) => return None,
    };

    let mut attempts = 1u64;
    let mut covered = vec![device_of_txg(rollback_txg)];
    let all_devices = [0u64, 1u64];
    while all_devices.iter().any(|device| !covered.contains(device)) && attempts - 1 < RING_REGIONS {
        let current_floor = pool.newest_persisted_floor();
        let warm_result = attempt_publish(pool, reclaim, false, current_floor, None, None, false, None, false);
        attempts += 1;
        let Ok(warm_txg) = warm_result else { break };
        let device = device_of_txg(warm_txg);
        if !covered.contains(&device) {
            covered.push(device);
        }
    }
    Some(attempts)
}

/// H6（登记「五、5.4」）：H1 跑到 Y 成功（没成功记「退化」）；再触动 2 次；管理员回退到
/// `R_old` = Y 被封进的那次发布之前最新的持久有效根（要求 `R_old` 在候选集里，不在记「回退够不着」，
/// 这一格同样退化）；回退与暖机之后尝试写 Y2，进重试循环——这里的 Q1（`RunSummary` 里那一栏）
/// 就是登记「六」的 Q9（回退之后再写要几次非空发布）。
///
/// 返回 `(RunSummary, isolated_slots_after_rollback)`：后者是回退之后 `pool.isolated` 里
/// 置位的槽数，对应登记「H6：回退之后隔离了一批槽再写」那句话本身——它是不是 > 0，是这一格
/// 最直接能核的事实。
pub(super) fn run_rollback_history(arm: Arm, geometry: Geometry, damage: Damage) -> (RunSummary, u64) {
    let mut pool = Pool::seed(geometry);
    pool.damage = damage;
    let (deleted_reference_object_identifier, mut fillers) = fill_pool(&mut pool, arm, geometry, &mut |_last_txg| false);
    for _ in 0..geometry.padding_deletes {
        if let Some(filler_object_identifier) = fillers.pop() {
            publish_delete(&mut pool, arm.reclaim, filler_object_identifier);
        }
    }
    publish_delete(&mut pool, arm.reclaim, deleted_reference_object_identifier);
    let last_txg_before_rewrite_attempt = pool.readable_roots().map(|root| root.txg).max().unwrap_or(0);
    let outcome = retry_loop(&mut pool, arm, geometry, &mut fillers, geometry.object_units, 0, 0, |_last_txg| false);
    let degenerate_summary =
        |pool_to_summarize: &Pool| degenerate_or_wedged_summary(arm, geometry, "H6".to_string(), pool_to_summarize);
    if !matches!(outcome, RetryOutcome::Success { .. } | RetryOutcome::OverBound { .. }) {
        // H1 到 Y 那一步没成功：outcome 本身已经知道是卡死还是未成／退化，直接用它的标签
        // （不是「猜」`allocation_failure_hit`——这里已经有真实的 `RetryOutcome` 可读）。
        let (label, count) = retry_outcome_label(outcome);
        let summary = build_run_summary(arm, geometry, "H6".to_string(), &pool, label, count, matches!(outcome, RetryOutcome::Wedged), 0);
        return (summary, 0);
    }

    // R_old = Y 被封进的那次发布之前最新的持久有效根：环里 txg 严格小于「删 X 之后到 Y 成功为止
    // 新产生的第一条根」的、这个前缀阶段本来就有的最新根。因为 Y 的重试循环从 `last_txg_before_rewrite_attempt` 之后
    // 才开始产生新根，`last_txg_before_rewrite_attempt` 本身就是「Y 被封进的那次发布之前最新的持久根」。
    let rollback_target_root = pool.readable_roots().find(|root| root.txg == last_txg_before_rewrite_attempt).cloned();
    let Some(rollback_target_root) = rollback_target_root else {
        return (degenerate_summary(&pool), 0);
    };

    for _ in 0..2 {
        match fillers.pop() {
            Some(filler_object_identifier) => {
                if publish_delete(&mut pool, arm.reclaim, filler_object_identifier).is_none() {
                    return (degenerate_summary(&pool), 0);
                }
            }
            None => return (degenerate_summary(&pool), 0),
        }
    }

    let effective_floor_now = pool.effective_rollback_floor();
    if rollback_target_root.txg < effective_floor_now || pool.is_abandoned(&rollback_target_root) {
        // 回退够不着（不在候选集里）：登记明令这一格记「回退够不着」，退化。
        return (degenerate_summary(&pool), 0);
    }

    if perform_rollback(&mut pool, arm.reclaim, &rollback_target_root).is_none() {
        // 回退那次发布自己真的分配不到（近满盘，Q5a／Q5b）：记卡死，不是「退化」——它是这一格
        // 真实发生的结果，不是「这次准入根本没走到该走的路径」。
        let summary = build_run_summary(arm, geometry, "H6".to_string(), &pool, "卡死".to_string(), -2, true, 0);
        return (summary, pool.isolated_count());
    }
    let isolated_slots_after_rollback = pool.isolated_count();

    let states_before_retry = distinct_valid_states(&pool);
    let outcome2 = retry_loop(&mut pool, arm, geometry, &mut fillers, geometry.object_units, 0, 0, |_last_txg| false);
    let states_after_retry = distinct_valid_states(&pool);
    let retained_states_dropped_below_minimum = u64::from(states_before_retry >= MINIMUM_RETAINED_STATES && states_after_retry < MINIMUM_RETAINED_STATES);
    let (outcome_label, delete_then_rewrite_publish_count) = retry_outcome_label(outcome2);
    let summary = build_run_summary(
        arm,
        geometry,
        "H6".to_string(),
        &pool,
        outcome_label,
        delete_then_rewrite_publish_count,
        matches!(outcome2, RetryOutcome::Wedged),
        retained_states_dropped_below_minimum,
    );
    (summary, isolated_slots_after_rollback)
}

// ===== H4a／H4b：推空发布之中崩溃，第一条带新 F 的根只落了一块盘，再重开（登记「五、5.4」，
// 第四段） =====

/// 两个崩溃点，登记原文：
/// - H4a「这次准入里第一条带新 F 的推空根持久之后立刻崩溃（这条根只在一块盘上）」；
/// - H4b「崩在这次准入的第二次推空已经分配、单元写完、根还没持久的时候」。
///
/// ⚠️ **两者在本模型里落到同一条代码路径**：`attempt_publish` 是一次性完成「释放 → 分配 →
/// 写根 → 持久」，不暴露「已分配、单元写完、根还没持久」这个中间态——崩在这个中间态，与崩在
/// 「上一次已持久之后、下一次还没开始」，两种说法在本模型能表达的粒度上，恢复选到的根都是
/// 「崩溃前最后一条已持久的根」，完全相同（H4b 那次「已经分配」的槽从没进过 `pool.ring`，
/// 崩溃即丢，等价于那次推空从没发生过）。这不是取样点选窄了，是 `attempt_publish` 原子性
/// 粒度决定的、与 M12「暖机等价变异」同一类（先证明等价，再当等价留档，不算漏测）——
/// `crash_point_equivalence_is_documented_not_assumed` 那条单测把这句话钉成断言。History
/// 标签仍按 `CrashPoint::label()` 分开打两条，供报告按标签统计。真要把两者分开，需要先把
/// `attempt_publish` 拆成「分配」与「写根持久」两个能单独崩溃的步骤，这份登记没有要求这一段
/// 做到（与 H6 `perform_rollback` 头部「简化，不是登记的全部」同一处置）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum CrashPoint {
    AfterFirstPush,
    BeforeSecondPush,
}

impl CrashPoint {
    pub(super) fn label(self) -> &'static str {
        match self {
            CrashPoint::AfterFirstPush => "H4a",
            CrashPoint::BeforeSecondPush => "H4b",
        }
    }
}

/// 崩溃与重开（5.1「崩溃与重开」行）：进程没了——扣住、在飞、影子账都丢；恢复选 txg 最大的
/// 持久有效根，从它的快照载入槽状态；取新实例、写行发布、暖机到本实例的根覆盖两块盘、M_sw
/// 回到 4 份再扣一份——这后半段（新实例、写行、暖机、扣一份切换预留）与「实例切换」（R8）
/// 逐字相同，直接复用 `perform_instance_switch`；这个函数只做前半段：选根、载入槽状态、
/// 清扣住／影子账、把切换预留重置回 4 份。写行那次发布之前不再推空发布——崩溃重开走的是
/// 恢复，不是准入，`perform_instance_switch` 本来就不推空，只写行 + 暖机。
fn crash_and_reopen(pool: &mut Pool, reclaim: ReclaimTiming) {
    let recovery_root = pool
        .readable_roots()
        .filter(|root| !pool.is_abandoned(root))
        .max_by_key(|root| root.txg)
        .cloned()
        .expect("起点状态至少有 mkfs 那条根，readable_roots 不会空");
    pool.slots = (*recovery_root.slots_snapshot).clone();
    pool.current_fixpoint_slots = (*recovery_root.fixpoint_slots_snapshot).clone();
    pool.current_instance_table_slots = (*recovery_root.instance_table_slots_snapshot).clone();
    pool.isolated.iter_mut().for_each(|flag| *flag = false);
    pool.pending_hold_floor = None;
    pool.switch_reserve_shares_per_disk = INSTANCE_SWITCH_LIMIT;
    let mut no_further_failures = |_last_txg: u64| false;
    perform_instance_switch(pool, reclaim, &mut no_further_failures);
}

/// H4a／H4b：同 H3a 的前缀到 3 次触动；尝试写 Y；这次准入里让恰好 1 次自推空发布持久
/// （见上方 ⚠️，H4a／H4b 因此共用这一步），然后崩溃、重开；重试循环接着走（触动计数接着数）。
///
/// 这次准入里自推次数一次都没发生（闸不用推空就过，或这条臂第一道闸不重判）：崩溃点不存在，
/// 记「退化」，与 H4a／H4b／H5 那句「这次准入里根本没有推空的臂上……记『退化』」同一形态。
/// 那一次自推自己就真的分配不到固定点（近满盘，PC5 那一类）：记「卡死」，不是「退化」——
/// 与 H4a／H4b／H5 要测的「崩溃再重开」是两件事。
///
/// 返回 `(RunSummary, self_pushes_before_crash, recovery_publishes_after_reopen)`：前者应恒为
/// 0（退化／卡死）或 1（真的崩到了）；后者是重开之后到 Y 最终批准为止又发生了几次发布尝试
/// （写行 + 暖机 + 之后可能还要的推空），诊断量，口径与 H5 的 `recovery_publishes_until_covered`
/// 相同（不是登记「六」Q4 的精确分量）。
pub(super) fn run_crash_mid_push_history(arm: Arm, geometry: Geometry, crash_point: CrashPoint, damage: Damage) -> (RunSummary, u64, u64) {
    let history_label = crash_point.label().to_string();
    let mut pool = Pool::seed(geometry);
    pool.damage = damage;
    let (deleted_reference_object_identifier, mut fillers) = fill_pool(&mut pool, arm, geometry, &mut |_last_txg| false);
    for _ in 0..geometry.padding_deletes {
        if let Some(filler_object_identifier) = fillers.pop() {
            publish_delete(&mut pool, arm.reclaim, filler_object_identifier);
        }
    }
    publish_delete(&mut pool, arm.reclaim, deleted_reference_object_identifier);

    let mut prefix_touches = 0u64;
    let mut degenerate = false;
    for _ in 0..3 {
        match fillers.pop() {
            Some(filler_object_identifier) => {
                if publish_delete(&mut pool, arm.reclaim, filler_object_identifier).is_none() {
                    degenerate = true;
                    break;
                }
                prefix_touches += 1;
            }
            None => {
                degenerate = true;
                break;
            }
        }
    }
    if degenerate {
        let summary = degenerate_or_wedged_summary(arm, geometry, history_label, &pool);
        return (summary, 0, 0);
    }

    let need = geometry.object_units * 2;
    let (gate1, _) = compute_gate1(&pool, arm, 0);
    if gate1 >= need || !arm.gate.rejudges_first_gate() {
        // 闸不用推空就过，或这条臂第一道闸不重判：这次准入根本走不到「自推空发布」，
        // 崩溃点不存在——退化。
        let summary = degenerate_or_wedged_summary(arm, geometry, history_label, &pool);
        return (summary, 0, 0);
    }
    let mut no_injected_failures = |_last_txg: u64| false;
    push_one_empty_publish(&mut pool, arm.reclaim, &mut no_injected_failures);
    if pool.allocation_failure_hit {
        // 这一次自推自己就真的卡死了（近满盘）：与 H4a／H4b 要测的「崩溃再重开」是两件事。
        let summary = degenerate_or_wedged_summary(arm, geometry, history_label, &pool);
        return (summary, 0, 0);
    }
    let self_pushes_before_crash = 1u64;

    crash_and_reopen(&mut pool, arm.reclaim);
    if pool.allocation_failure_hit {
        let summary = build_run_summary(arm, geometry, history_label, &pool, "卡死".to_string(), -2, true, 0);
        return (summary, self_pushes_before_crash, 0);
    }

    let pushes_before_retry = pool.pushes_so_far;
    let states_before_retry = distinct_valid_states(&pool);
    let outcome = retry_loop(&mut pool, arm, geometry, &mut fillers, geometry.object_units, prefix_touches, prefix_touches, |_last_txg| false);
    let states_after_retry = distinct_valid_states(&pool);
    let retained_states_dropped_below_minimum = u64::from(states_before_retry >= MINIMUM_RETAINED_STATES && states_after_retry < MINIMUM_RETAINED_STATES);
    let (outcome_label, delete_then_rewrite_publish_count) = retry_outcome_label(outcome);
    let recovery_publishes_after_reopen = pool.pushes_so_far - pushes_before_retry;
    let summary = build_run_summary(
        arm,
        geometry,
        history_label,
        &pool,
        outcome_label,
        delete_then_rewrite_publish_count,
        matches!(outcome, RetryOutcome::Wedged),
        retained_states_dropped_below_minimum,
    );
    (summary, self_pushes_before_crash, recovery_publishes_after_reopen)
}

#[cfg(test)]
#[path = "histories_of_failures_rollbacks_and_crashes_tests.rs"]
mod tests;
