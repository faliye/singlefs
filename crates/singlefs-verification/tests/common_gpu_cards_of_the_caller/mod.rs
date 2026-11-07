//! 用例自己写开 GPU 的多机配置、在环境守卫里起引擎时交给引擎的显卡先后：调用方给这个进程的那一份
//! （`gpu_card_configuration::GpuConfigurationOfTheCaller`，服务分卡时经环境交的那一份），写成环境变量 `GPU_CARDS_BY_PRIORITY` 的值。
//! 为什么经环境变量、不抄进用例的配置：配置里的卡不拿来挑卡，环境变量没设而配置开了 GPU，清没清场引擎都照「清场与分卡由服务做」拒
//! （`gpu_card_configuration::cards_by_priority_from`）。环境变量设了（空串也算：本机能用的卡全用）引擎就照它，与卡池照调用方那一份挑的卡对得上。
//! 经 `run_from_environment` 跑的那一路在 `common_crash_points` 里照同一个值设（`card_priority_of_the_caller_for_a_test_configuration_naming_no_card`）。

use singlefs_verification::gpu_card_configuration::GpuConfigurationOfTheCaller;

/// 调用方给这个进程的显卡先后写成 `GPU_CARDS_BY_PRIORITY` 的值：`<HOSTS 里的地址>:<序号>:<额度>M` 空白隔开，一张卡都没点名是空串。
///
/// # Panics
/// 调用方的多机配置读不了、显卡先后写坏了（连同地址不在 HOSTS 里、多机配置判法认不出这一台），或没经服务给卡而调用方的多机配置开了 GPU
/// （清没清场都一样；GPU 用例整条经 `task-server/task submit` 交，服务清场、分卡）。
pub fn card_priority_value_of_the_caller() -> String {
    let caller = GpuConfigurationOfTheCaller::of_this_process();
    // 多机配置是这台机器的，按这个进程的当前目录找（cargo 把测试进程的当前目录设成包目录；仓根不是键的输入）
    let caller_configuration = caller
        .multi_host_configuration_file(
            &std::env::current_dir().expect("取得到这个进程的当前目录（多机配置从这里起找）"),
        )
        .expect("调用方的多机配置在");
    caller
        .cards_by_priority(caller_configuration.as_deref())
        .expect("调用方的显卡先后写法对、开着 GPU 时经服务给了卡（清没清场都一样）")
        .map_or_else(String::new, |cards| {
            cards
                .cards()
                .iter()
                .map(|card| {
                    format!(
                        "{}:{}:{}M",
                        card.host.as_str(),
                        card.index,
                        card.memory_quota_mebibytes
                    )
                })
                .collect::<Vec<_>>()
                .join(" ")
        })
}
