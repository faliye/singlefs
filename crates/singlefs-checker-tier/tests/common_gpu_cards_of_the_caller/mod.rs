//! 用例自己写开 GPU 的多机配置、在环境守卫里起引擎时交给引擎的显卡先后：调用方给这个进程的那一份
//! （`gpu_card_configuration::GpuConfigurationOfTheCaller`，清过场取清场之后那一份），写成环境变量 `GPU_CARDS_BY_PRIORITY` 的值。
//! 为什么经环境变量、不抄进用例的配置：调用方的显卡先后只写在仓根 `multi-host.env` 里、变异复跑的源码副本里又没有那一份时，抄进配置的是空串，
//! 配置只剩 `ENABLE_GPU=1`；清场之下开了 GPU 而没写清场之后那一份显卡先后，引擎照「不退回、不照空闲显存挑卡」拒
//! （`gpu_card_configuration::cards_by_priority_from`）。环境变量设了（空串也算：本机能用的卡全用）引擎就照它，与卡池照调用方那一份挑的卡对得上。
//! 经 `run_from_environment` 跑的那一路在 `common_crash_points` 里照同一个值设（`card_priority_of_the_caller_for_a_test_configuration_naming_no_card`）。

use singlefs_checker_tier::gpu_card_configuration::GpuConfigurationOfTheCaller;

/// 调用方给这个进程的显卡先后写成 `GPU_CARDS_BY_PRIORITY` 的值：`<机器>:<序号>:<额度>M` 空白隔开，一张卡都没点名是空串。
///
/// # Panics
/// 调用方的多机配置读不了、显卡先后写坏了，或清场之下开了 GPU 而没写清场之后那一份（调用方自己的配置就跑不了 GPU 用例）。
pub fn card_priority_value_of_the_caller() -> String {
    let caller = GpuConfigurationOfTheCaller::of_this_process();
    // 多机配置是这台机器的，按这个进程的当前目录找（cargo 把测试进程的当前目录设成包目录；仓根不是键的输入，定案 1）
    let caller_configuration = caller
        .multi_host_configuration_file(
            &std::env::current_dir().expect("取得到这个进程的当前目录（多机配置从这里起找）"),
        )
        .expect("调用方的多机配置在");
    caller
        .cards_by_priority(caller_configuration.as_deref())
        .expect("调用方的显卡先后写法对、清场之下写了清场之后那一份")
        .map_or_else(String::new, |cards| {
            cards
                .iter()
                .map(|card| {
                    format!(
                        "{}:{}:{}M",
                        card.host.name(),
                        card.index,
                        card.memory_quota_mebibytes
                    )
                })
                .collect::<Vec<_>>()
                .join(" ")
        })
}
