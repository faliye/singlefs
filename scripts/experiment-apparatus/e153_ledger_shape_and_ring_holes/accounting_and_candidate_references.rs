//! 根账户、记账读数与字节数、候选引用的并集与区间推断、废弃根的界、记账核对与五种坏镜像。

use e7_index_bench::i64_from_u64;
use super::*;

// ---------------------------------------------------------------------
// 一条根的账（供影子账、回退恢复、违例检查用）：role → (slot, span)。
// ---------------------------------------------------------------------

#[derive(Clone, Debug, Default)]
pub(super) struct RootAccount {
    slots: BTreeMap<u64, u64>, // slot -> span
}

impl RootAccount {
    /// 展开成账里覆盖的每一个物理槽号（跨度 2 的角色要展开成两个槽，不能只取起点）。
    pub(super) fn expanded_slots(&self) -> impl Iterator<Item = u64> + '_ {
        self.slots.iter().flat_map(|(&start, &span)| (0..span).map(move |offset| start + offset))
    }
}

pub(super) fn account_from_refs(refs: &HashMap<Role, u64>) -> RootAccount {
    let mut slots = BTreeMap::new();
    for (role, slot) in refs {
        slots.insert(*slot, role.span_in_slots());
    }
    RootAccount { slots }
}

/// Q1/Q2 用：一批 txg 的账在这条臂上引用的物理槽并集大小（去重，第一节「I-3.1（臂内读法）」的
/// 「按 (设备, 起点槽, 跨度) 去重，跨度求和」——本模型单设备视角下按物理槽本身去重即可，
/// 两盘同槽（M0.1）意味着两盘各自算一遍、槽号相同，不需要另带设备维）。
/// Q9 轨迹用：五条臂当前的分配记录条目数（Q9a）与两个读数计数器（Q9b，`reads_last_reclaim_step`
/// 是每次发布都会更新的「发布前步」读数；`reads_last_shadow_recompute` 只在重建 / 抬 F 时更新，
/// 调用方按 `is_rebuild_step` 决定这一格是不是「重建步」的采样点——见 `PublishOutcome` 字段注释）。
pub(super) fn snapshot_allocation_record_read_counters(arms: &[ArmExecutor; 5]) -> ([u64; 5], [u64; 5], [u64; 5]) {
    let mut entries = [0u64; 5];
    let mut reclaim = [0u64; 5];
    let mut shadow = [0u64; 5];
    for (index, arm) in arms.iter().enumerate() {
        entries[index] = arm.ledger.len() as u64;
        reclaim[index] = arm.reads_last_reclaim_step;
        shadow[index] = arm.reads_last_shadow_recompute;
    }
    (entries, reclaim, shadow)
}

/// Q9a：一条臂某一时刻的条目数，推出条目字节与叶节点数（登记第六节 Q9a「条目字节 = 条目数 ×
/// 条目宽；叶节点数 = ⌈条目数 ÷ 每节点条目数⌉」）。
pub(super) fn allocation_record_bytes_and_leaf_nodes(entries: u64, record_width_bytes: u64, records_per_node: u64) -> (u64, u64) {
    (entries * record_width_bytes, entries.div_ceil(records_per_node))
}

/// Q9 轨迹的峰值 / 均值 / 末值三元组；`values` 为空时三项都是 0（调用方另行判断样本数是不是 0，
/// 均值为 0 不代表「量到了 0」，见 Q9b「重建步」样本数可能为 0 的情形）。
pub(super) fn peak_mean_end(values: &[u64]) -> (u64, f64, u64) {
    if values.is_empty() {
        return (0, 0.0, 0);
    }
    let peak = *values.iter().max().expect("非空");
    let mean = values.iter().sum::<u64>() as f64 / values.len() as f64;
    let end = *values.last().expect("非空");
    (peak, mean, end)
}

fn candidate_reference_union(arm: &ArmExecutor, txgs: &[u64]) -> BTreeSet<u64> {
    let mut union: BTreeSet<u64> = BTreeSet::new();
    for txg in txgs {
        if let Some(account) = arm.root_accounts.get(txg) {
            union.extend(account.expanded_slots());
        }
    }
    union
}

pub(super) fn candidate_reference_union_size(arm: &ArmExecutor, txgs: &[u64]) -> u64 {
    candidate_reference_union(arm, txgs).len() as u64
}

/// Q10：G12×G28 的候选引用推法（登记第五节「G12」定义：「候选根 r 引用槽 x ⟺ 当前账里 x 的记录
/// 分配代 ≤ txg(r) ∧（未释放 ∨ txg(r) < 释放代）」）——从这条臂自己的账本直接推，不读候选根自己的
/// 账（`root_accounts`）。与 `candidate_reference_union`（M0.6 走读）比较得到 Q10。
pub(super) fn interval_inferred_candidate_references(arm: &ArmExecutor, candidate_txgs: &[u64]) -> BTreeSet<u64> {
    let mut inferred: BTreeSet<u64> = BTreeSet::new();
    for (&slot, entry) in &arm.ledger {
        let referenced = candidate_txgs.iter().any(|&txg| entry.allocation_generation <= txg && (!entry.released || txg < entry.release_generation));
        if referenced {
            for offset in 0..entry.span_in_slots {
                inferred.insert(slot + offset);
            }
        }
    }
    inferred
}

/// Q6b：环里被抛弃根各自的「记账第 1 项」代理值——本模型没有为每条历史根单独存一份「它持久那一刻
/// 记的 item1」，只存了它的引用集合（`root_accounts`）；对一条仍在候选集 / 环里活着、按定义没有被
/// 破坏的根，它持久那一刻的 item1 与它自己的引用集合大小相等（这正是 I-3.1 要成立的那件事），所以
/// 用引用集合大小当代理值——登记第十二节修订记这一条简化。逐臂返回 (环里 txg 最大的被抛弃根的值,
/// 全部被抛弃根里的最大值)；环里没有被抛弃根时两项都是 `None`。
pub(super) fn abandoned_root_item1_bounds(arms: &[ArmExecutor; 5], abandoned_txgs: &[u64]) -> [(Option<u64>, Option<u64>); 5] {
    let mut result: [(Option<u64>, Option<u64>); 5] = [(None, None); 5];
    if abandoned_txgs.is_empty() {
        return result;
    }
    let newest_txg = *abandoned_txgs.iter().max().expect("非空");
    for (index, arm) in arms.iter().enumerate() {
        let mut newest_value: Option<u64> = None;
        let mut maximum_value: Option<u64> = None;
        for &txg in abandoned_txgs {
            if let Some(account) = arm.root_accounts.get(&txg) {
                let size = account.expanded_slots().count() as u64;
                if txg == newest_txg {
                    newest_value = Some(size);
                }
                maximum_value = Some(maximum_value.map_or(size, |current: u64| current.max(size)));
            }
        }
        result[index] = (newest_value, maximum_value);
    }
    result
}

/// Q8（坏镜像矩阵）：一个 (臂, 格) 组合在某个合法基底状态上的记账三项与三种引用集合大小——
/// 臂内候选并集（I-3.1 臂内读法）、今天读法候选并集（I-3.1 今天 checker 读法）、最新根自己的
/// 引用集合（G27）。坏镜像只改 `accounting`，三种引用集合大小保持基底不变（登记：B1a/B1b/B2/Bk1/Bk2
/// 只改 D0 的记账，不改盘上单元本身，所以走读得到的引用集合不受影响）。
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct BaseImageState {
    pub(super) accounting: AccountingRow,
    candidate_by_effective_floor: u64,
    candidate_by_today_reading: u64,
    newest_reference_size: u64,
}

/// 四个候选检查在一份（可能被坏镜像改过的）状态上是否判红。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(super) struct AccountingChecks {
    pub(super) allocated_statistic_mismatch_by_arm_reading: bool,
    pub(super) allocated_statistic_mismatch_by_today_reading: bool,
    pub(super) free_statistic_mismatch: bool,
    pub(super) allocated_minus_deferred_red: bool,
}

pub(super) fn accounting_checks(state: BaseImageState) -> AccountingChecks {
    let row = state.accounting;
    AccountingChecks {
        allocated_statistic_mismatch_by_arm_reading: row.item1_occupied_slots != state.candidate_by_effective_floor,
        allocated_statistic_mismatch_by_today_reading: row.item1_occupied_slots != state.candidate_by_today_reading,
        free_statistic_mismatch: row.item1_occupied_slots + row.item2_free_slots != UNIT_AREA_CAPACITY_SLOTS,
        allocated_minus_deferred_red: row.item1_occupied_slots.saturating_sub(row.item5_deferred_slots) != state.newest_reference_size,
    }
}

/// 五条臂在这一刻的 `BaseImageState`（登记第五节坏镜像的「基底」：β1 = H1 主历史最后一次发布之后，
/// β2 = H4dNm2 主历史回退之后第 S 次工作负载发布之后）。
pub(super) fn base_image_states(world: &World) -> [BaseImageState; 5] {
    let candidate_by_effective_floor: Vec<u64> = world.pool.candidate_roots().iter().map(|entry| entry.txg).collect();
    let floor_of_newest_root = world.pool.current_rollback_floor;
    let candidate_by_today_reading: Vec<u64> = world
        .pool
        .live_roots()
        .iter()
        .filter(|entry| world.pool.is_valid(entry.instance, entry.txg) && entry.txg >= floor_of_newest_root)
        .map(|entry| entry.txg)
        .collect();
    let mut states = [BaseImageState::default(); 5];
    for (index, arm) in world.arms.iter().enumerate() {
        states[index] = BaseImageState {
            accounting: arm.accounting_row(),
            candidate_by_effective_floor: candidate_reference_union_size(arm, &candidate_by_effective_floor),
            candidate_by_today_reading: candidate_reference_union_size(arm, &candidate_by_today_reading),
            newest_reference_size: account_from_refs(&arm.refs).expanded_slots().count() as u64,
        };
    }
    states
}

/// B1a（登记第五节坏镜像）：活单元被记成已释放，只翻记录标志——记账三项与引用集合都不受影响，
/// 走的是「记录本身没人读」这条口径（第三节「checker 不读记账第 5 项、不读分配记录的已释放标志」）。
pub(super) fn bad_image_b1a(base: BaseImageState) -> BaseImageState {
    base
}

/// B1b：B1a 再把第 5 项加 2 槽（登记：「第 1 项、第 2 项不动」）。
pub(super) fn bad_image_b1b(base: BaseImageState) -> BaseImageState {
    let mut modified = base;
    modified.accounting.item5_deferred_slots += 2;
    modified
}

/// B2：第 5 项单独多报一槽，别的不动。
pub(super) fn bad_image_deferred_overcount_by_one(base: BaseImageState) -> BaseImageState {
    let mut modified = base;
    modified.accounting.item5_deferred_slots += 1;
    modified
}

/// PC-F 的 Bk1（阳性对照）：D0 第 1 项多记 1 槽。
pub(super) fn bad_image_bk1(base: BaseImageState) -> BaseImageState {
    let mut modified = base;
    modified.accounting.item1_occupied_slots += 1;
    modified
}

/// PC-F 的 Bk2（阳性对照）：D0 第 2 项少记 1 槽。
pub(super) fn bad_image_bk2(base: BaseImageState) -> BaseImageState {
    let mut modified = base;
    modified.accounting.item2_free_slots = modified.accounting.item2_free_slots.saturating_sub(1);
    modified
}

/// Q6b 一次采样的判定结果（登记第六节 Q6b-甲/乙）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct AbandonedRootBoundCheck {
    pub(super) isolated_exceeds_newest_bound: bool,
    pub(super) newest_bound_negative: bool,
    pub(super) isolated_exceeds_maximum_bound: bool,
    pub(super) maximum_bound_negative: bool,
}

/// Q6b：把「这一刻的隔离槽数」与两种读法的上界比较（读法甲用环里 txg 最大的被抛弃根，读法乙用
/// 全部被抛弃根里的最大值；两者都减去 R_old 的代理值）。纯函数，main() 的采样循环与单测共用。
pub(super) fn abandoned_root_bound_check(isolated_slots: u64, newest_abandoned_item1: u64, maximum_abandoned_item1: u64, rollback_target_reference_count: u64) -> AbandonedRootBoundCheck {
    let isolated = i64_from_u64(isolated_slots);
    let newest_bound = i64_from_u64(newest_abandoned_item1) - i64_from_u64(rollback_target_reference_count);
    let maximum_bound = i64_from_u64(maximum_abandoned_item1) - i64_from_u64(rollback_target_reference_count);
    AbandonedRootBoundCheck {
        isolated_exceeds_newest_bound: isolated > newest_bound,
        newest_bound_negative: newest_bound < 0,
        isolated_exceeds_maximum_bound: isolated > maximum_bound,
        maximum_bound_negative: maximum_bound < 0,
    }
}

/// Q10 的一次核对：只在 IntervalNarrow（G12×G28）这条臂上有意义（登记第五节 G12 的「支持它的人认的
/// 样子」只对这条组合许诺「不另读候选根的树」）；别的臂调用这个函数不做任何事。核对不一致时累计
/// `interval_candidate_inference_mismatch_count`，不改变任何实际状态——它是一次旁路核对，不参与隔离集计算
/// （隔离集仍按 `recompute_shadow` / `isolate_additional_for_raised_floor` 里已经写好的走读路径算，
/// 本函数只是多算一遍、拿去比对，避免「一改就影响真实结果」这类风险）。
pub(super) fn check_interval_candidate_inference_against_walk(arm: &mut ArmExecutor, candidate_txgs: &[u64]) {
    if arm.arm != Arm::IntervalNarrow {
        return;
    }
    let inferred = interval_inferred_candidate_references(arm, candidate_txgs);
    let walked = candidate_reference_union(arm, candidate_txgs);
    if inferred != walked {
        arm.interval_candidate_inference_mismatch_count += 1;
    }
}

#[cfg(test)]
#[path = "accounting_and_candidate_references_tests.rs"]
mod tests;
