#!/usr/bin/env bash
# 红样本（新公开入口与表里的入口互相调，写在它后面）：d137-ledger-defs-r3 攻方模型 research/prompts/d137-ledger-defs-r3-opus-model/ 的 sync-cycle-new-entry-after 世界原样做成。
# 同 judge-entries-in-sync-green，表里的 enumerate_crash_states 改成先调新入口 enumerate_crash_states_one_more_round(0)、再调 enumerate_crash_states_with_every_kind_of_observer()；
# 新入口写在文件末尾（第 49 行），调回 enumerate_crash_states()。它经 enumerate_crash_states 走得到判段去路的记号，JUDGE_ENTRIES 里没有：判红点名它
# （扫了 21 个函数单元，公开 18 个，带记号 4 个）。改前递归走图时先算 enumerate_crash_states，在环上把新入口截成「走不到」并记住，轮到新入口直接取这个假，判绿；
# 新入口挪到 enumerate_crash_states 前面就判红（攻方的对照世界 sync-cycle-new-entry-before-control）：结局随先后翻。
# 这一格不看改动范围，不建 git 仓；源文件只放认法要看的几样，不是能编的源码。
set -euo pipefail
mkdir -p crates/singlefs-verification/src
cat > crates/singlefs-verification/src/crash_amplification.rs <<'EOF'
//! 样本：判段入口表与源码同步（judge-entries-in-sync）。只放认法要看的几样，不是能编的源码。

/// 判段不跑的原因：只收两个布尔，不调记号、不算入口。
pub fn judge_not_run_without_the_gpu_feature(gpu_is_switched_on: bool, gpu_feature_is_compiled_in: bool) -> Option<String> {
    (gpu_is_switched_on && !gpu_feature_is_compiled_in).then(|| "没编进 gpu 特性：{判段不跑}".to_string())
}

pub fn judge_not_run_in_this_binary() -> Option<String> {
    judge_not_run_without_the_gpu_feature(true, cfg!(feature = "gpu"))
}

pub fn judging_test_not_run_without_the_gpu_feature(test_name: &str) -> bool {
    judge_not_run_in_this_binary().is_some() && !test_name.is_empty()
}

fn run_one_slice_round(observer_count: usize) -> usize {
    if cfg!(feature = "gpu") { observer_count } else { 0 }
}

pub fn run_from_environment() -> usize {
    run_from_environment_with_observers(&[])
}

pub fn run_from_environment_with_observers(observers: &[usize]) -> usize {
    run_one_slice_round(observers.len())
}

pub struct GivenStatesEngine {
    pub item_name: &'static str,
}

impl GivenStatesEngine {
    pub fn start_from_environment() -> Self {
        let _reason = judge_not_run_in_this_binary();
        Self { item_name: "given-states" }
    }

    pub fn item_name(&self) -> &'static str {
        self.item_name
    }
}

/// 只在注释里写着 cfg!(feature = "gpu") 的公开函数不算入口。
pub fn plan_crash_points() -> usize {
    // cfg!(feature = "gpu") 只在注释里
    let _text = "cfg!(feature = \"cpu\") }";
    7
}

#[cfg(test)]
mod tests {
    pub fn helper_reaching_the_route() -> bool {
        super::judge_not_run_in_this_binary().is_some()
    }
}
EOF
cat > crates/singlefs-verification/src/crash_state_enumeration.rs <<'EOF'
//! 样本：判段入口表与源码同步（judge-entries-in-sync）。只放认法要看的几样，不是能编的源码。

fn crash_point_replay_judge_route() -> bool {
    cfg!(feature = "gpu")
}

pub fn crash_point_replay_judge_not_run_line(item_name: &str) -> String {
    format!("NOT_RUN test={item_name} {:?}", crate::crash_amplification::judge_not_run_without_the_gpu_feature(true, false))
}

fn enumerate_crash_states_through_the_library() -> bool {
    crash_point_replay_judge_route()
}

pub fn enumerate_crash_states_with_every_kind_of_observer() -> bool {
    enumerate_crash_states_through_the_library()
}

pub fn enumerate_crash_states() -> bool {
    enumerate_crash_states_one_more_round(0) || enumerate_crash_states_with_every_kind_of_observer()
}

pub fn enumerate_crash_states_selecting() -> bool {
    enumerate_crash_states_with_every_kind_of_observer()
}

pub fn enumerate_crash_states_versions() -> bool {
    enumerate_crash_states_with_every_kind_of_observer()
}

pub fn enumerate_crash_states_selecting_versions() -> bool {
    enumerate_crash_states_with_every_kind_of_observer()
}

pub fn enumerate_crash_states_quick_tier_versions() -> bool {
    enumerate_crash_states_with_every_kind_of_observer()
}

pub fn enumerate_crash_states_with_expansion() -> bool {
    enumerate_crash_states_with_every_kind_of_observer()
}

/// 只在 CPU 上取样，不走判段。
pub fn sample_crash_states_on_the_cpu() -> usize {
    3
}

/// 新加的公开入口：再走一轮枚举，经 enumerate_crash_states 走到判段（两者互相调），表里还没有。
pub fn enumerate_crash_states_one_more_round(round: usize) -> bool {
    round > 0 && enumerate_crash_states()
}
EOF
