#!/usr/bin/env bash
# 绿样本（判段入口表与源码同步）：两份判段源文件里认出的公开判段入口恰好是 JUDGE_ENTRIES 那 13 个名字（GivenStatesEngine 经它的 start_from_environment 认出）；
# 另放几样不算入口的：不调记号的公开纯函数 judge_not_run_without_the_gpu_feature、只在注释里写着 cfg!(feature = "gpu") 的 plan_crash_points、
# #[cfg(test)] 模块里调判段的公开函数、只在 CPU 上取样的 sample_crash_states_on_the_cpu、GivenStatesEngine 里不走判段的 item_name。判绿：20 个函数单元，公开 17 个，带记号 4 个。
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
    enumerate_crash_states_with_every_kind_of_observer()
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

pub fn enumerate_crash_states_with_attached_observers() -> bool {
    enumerate_crash_states_with_every_kind_of_observer()
}

pub fn enumerate_crash_states_with_expansion() -> bool {
    enumerate_crash_states_with_every_kind_of_observer()
}

/// 只在 CPU 上取样，不走判段。
pub fn sample_crash_states_on_the_cpu() -> usize {
    3
}
EOF
