//! 世界（d137-ledger-defs-r3 攻方腿）：判段入口表与源码同步（judge-entries-in-sync）。只放认法要看的几样，不是能编的源码。

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
