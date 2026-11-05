//! 世界（d137-ledger-defs-r3 攻方腿）：判段入口表与源码同步（judge-entries-in-sync）。只放认法要看的几样，不是能编的源码。
//! 与门禁样本 judge-entries-in-sync-green 的 crash_amplification.rs 同形。

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
