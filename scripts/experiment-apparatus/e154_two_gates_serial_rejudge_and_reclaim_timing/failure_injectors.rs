//! 注入失败的位置与两种注入器、退化或卡死的汇总。

use super::*;

// ===== H5-k-位：根槽写失败按 R8 走实例切换（登记「五、5.4」） =====

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum FailurePosition {
    /// 从第一条带新 F 的根起（这次准入内部第 1 次根槽写）。
    First,
    /// 从让 F_生效 变成新值的那条根起（覆盖两块盘所需的最后一次根槽写；「四」的 I1 锚点
    /// 给出的 2/3 次公式在这里复用，因为它就是同一个 txg mod 3 轮转事实）。
    Cover,
}

impl FailurePosition {
    pub(super) fn label(self) -> &'static str {
        match self {
            FailurePosition::First => "首",
            FailurePosition::Cover => "覆",
        }
    }
}

/// 覆盖两块盘所需的根槽写次数（与「四」I1 锚点同一个公式：`last_txg mod 3 = 1` 时 3 次，否则 2 次），
/// 只是这里要的是"从哪次开始失败"的窗口起点，不是覆盖次数本身——两者数值相同（覆盖需要几次，
/// 让 F_生效 真正变成新值的正是最后那一次，也就是第几次）。拆成一个只吃 `last_txg` 标量的版本
/// （`cover_push_index_from_last_txg`），是因为 H8 要在 `push_one_empty_publish`
/// 内部、`pool` 已经借给函数参数的那一刻现算这个值——传一个 `&Pool` 进闭包会撞借用检查，传一个
/// 提前读好的 `u64` 不会。
fn cover_push_index_from_last_txg(last_txg: u64) -> u64 {
    if last_txg % RING_REGIONS == 1 { 3 } else { 2 }
}

pub(super) fn cover_push_index(pool: &Pool) -> u64 {
    cover_push_index_from_last_txg(pool.readable_roots().map(|root| root.txg).max().unwrap_or(0))
}

/// 这次推空即将发生之前，环里最新一条持久根的 txg——`push_one_empty_publish`／`perform_instance_switch`
/// 在问 `failures` 之前用它现算「这是这次运行里第几次自推空发布」所需要知道的那一点点 pool 状态，
/// 不需要把整个 `&Pool` 传进闭包。
pub(super) fn last_persisted_txg(pool: &Pool) -> u64 {
    pool.readable_roots().map(|root| root.txg).max().unwrap_or(0)
}

/// 连续 k 次根槽写失败的窗口：从 `window_start` 起（1 起数，数的是这次准入内部**每一次**根槽写
/// 尝试——推空自己的、切换写行的、暖机的，登记「五、5.4」H5-k-位「切换的写行发布的根也算在
/// 连续失败里」），连续 k 次返回真。`injected` 记真正问到几次失败——若这次准入的根槽写次数不够
/// （比如闸提前批准、或提前卡死），`injected < k`，调用方据此判定「退化」（登记 H4a／H4b／H5
/// 那句「在这次准入里根本没有推空（或没有第二次推空）的臂上……记『退化』」）。
pub(super) struct FailureInjector {
    call_index: u64,
    window_start: u64,
    remaining: u64,
    pub(super) injected: u64,
}

impl FailureInjector {
    pub(super) fn new(window_start: u64, consecutive_root_write_failure_count: u64) -> Self {
        FailureInjector { call_index: 0, window_start, remaining: consecutive_root_write_failure_count, injected: 0 }
    }
    /// `_last_txg`：H5 的 `window_start` 已经在构造时算好（在进入这次准入的 episode 之前，
    /// 用 `cover_push_index(&pool)` 现算一次），这里不需要再看 pool 状态——参数只是为了跟
    /// `LazyFailureInjector::next` 共用同一个 `FnMut(u64) -> bool` 闭包签名。
    pub(super) fn next(&mut self, _last_txg: u64) -> bool {
        self.call_index += 1;
        if self.call_index >= self.window_start && self.remaining > 0 {
            self.remaining -= 1;
            self.injected += 1;
            true
        } else {
            false
        }
    }
}

/// H8 专用（依据「十二、修订」第三段第 7 条第 2 类诊断：
/// P-止 在「填满」阶段自己把 F 抬到位，H5 的失败注入窗口从没落在这几次自推空发布上）：与
/// `FailureInjector` 同一套「连续 k 次」语义，区别是 `window_start` 不在构造时算好，而是在
/// 第一次被问到（也就是这次运行里第一次真的发生自推空发布，不管是在填满阶段还是在「删后写回」
/// 那次准入里）那一刻，用当时的 `last_txg` 现算——这样同一个注入器既能落在填满阶段第一次为
/// 抬 F 自推的那个 episode 上，也能在填满阶段全程不自推时，原样落到「删后写回」那次准入的推空
/// episode 上（`run_fill_phase_injected_history` 把它从 `fill_pool` 一路带进后续 3 次触动之后的 `retry_loop`，
/// 两处都不清空 `call_index`）。
pub(super) struct LazyFailureInjector {
    position: FailurePosition,
    call_index: u64,
    window_start: Option<u64>,
    remaining: u64,
    pub(super) injected: u64,
}

impl LazyFailureInjector {
    pub(super) fn new(position: FailurePosition, consecutive_root_write_failure_count: u64) -> Self {
        LazyFailureInjector { position, call_index: 0, window_start: None, remaining: consecutive_root_write_failure_count, injected: 0 }
    }
    pub(super) fn next(&mut self, last_txg: u64) -> bool {
        self.call_index += 1;
        let window_start = *self.window_start.get_or_insert_with(|| match self.position {
            FailurePosition::First => 1,
            FailurePosition::Cover => cover_push_index_from_last_txg(last_txg),
        });
        if self.call_index >= window_start && self.remaining > 0 {
            self.remaining -= 1;
            self.injected += 1;
            true
        } else {
            false
        }
    }
}

/// 「这一格该记『退化』还是『卡死』」：`.claude/rules/mutation-sampling.md`「同一条纪律的反面」——
/// 探针跑撞到过（PC5 在 H5 上 `pc5_root_write_failure_hit_any=false`）：`Damage::zero_reserves` 时
/// `fill_pool` 自己就会先卡死（近满盘固定点分配失败，`pool.allocation_failure_hit` 已置位），
/// 之后连续 3 次触动里第一次 `publish_delete` 撞上同一个卡死、`is_none()` 为真，H5 的退化分支
/// 把它记成「退化」——这掩盖了 PC5 真正要证明的那件事：它不是「这次准入没走到该走的路径」，
/// 是真的卡死了。所以两处退化分支都要先问 `pool.allocation_failure_hit`，真卡死记「卡死」，
/// 不记「退化」。
pub(super) fn degenerate_or_wedged_summary(arm: Arm, geometry: Geometry, history: String, pool: &Pool) -> RunSummary {
    if pool.allocation_failure_hit {
        build_run_summary(arm, geometry, history, pool, "卡死".to_string(), -2, true, 0)
    } else {
        build_run_summary(arm, geometry, history, pool, "退化".to_string(), -3, false, 0)
    }
}

#[cfg(test)]
#[path = "failure_injectors_tests.rs"]
mod tests;
