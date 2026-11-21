//! 判段走哪一路（GPU、CPU、不跑）照多机配置与特性定，与崩溃点重放的工作线程数。

use super::*;

/// 崩溃点重放判段的工作线程数从这个环境变量取（十进制正整数）；没设取崩溃放量的 `SINGLEFS_CRASH_AMPLIFICATION_THREADS`，再没设取
/// 每台默认重活线程数（`SINGLEFS_THREAD_CAP`，再没设取 `.claude/heavy-thread-default` 的登记值）。门禁 crash-point-replay 显式传进来。
pub const CRASH_POINT_REPLAY_WORKER_THREADS_ENVIRONMENT_VARIABLE: &str =
    "SINGLEFS_CRASH_POINT_REPLAY_THREADS";

/// 照本机多机配置选路（[`crash_point_replay_judge_route_of_the_switches`]）：`configuration` 是判段入口读的那一份
/// （`crate::crash_amplification::multi_host_configuration_of_this_process`，按这个进程从哪起找，不收仓根）。
pub(super) fn crash_point_replay_judge_route(
    configuration: &crate::crash_amplification::MultiHostConfigurationOfThisProcess,
) -> CrashPointReplayJudgeRoute {
    crash_point_replay_judge_route_of_the_switches(
        configuration.mode.uses_gpu(),
        cfg!(feature = "gpu"),
    )
}

/// 配置开了 GPU（`ENABLE_GPU=1`）而这份二进制编进了 `gpu` 特性就上卡；配了而没编进来，判段不跑、报本次未跑（不退回 CPU 判
/// 「配 GPU 必用 GPU、做不到报错不降级」）；没配由 CPU 判。
pub(super) fn crash_point_replay_judge_route_of_the_switches(
    gpu_is_switched_on: bool,
    gpu_feature_is_compiled_in: bool,
) -> CrashPointReplayJudgeRoute {
    match (gpu_is_switched_on, gpu_feature_is_compiled_in) {
        (true, true) => CrashPointReplayJudgeRoute::Gpu,
        (true, false) => CrashPointReplayJudgeRoute::JudgeNotRunWithoutTheGpuFeature,
        (false, _) => CrashPointReplayJudgeRoute::Cpu {
            why_not_the_gpu: "gpu_not_enabled_in_the_configuration",
        },
    }
}

/// 判段不跑那一趟打的本次未跑那一行：门禁认的 GPU 用例那一族记号（`NOT_RUN_<原因> test=<用例> reason=<一句>`，
/// `singlefs_verification::gpu_memory_ledger::gpu_not_run_line`），用例是这条崩溃枚举用例的项名；原因同崩溃放量那一处
/// （`crate::crash_amplification::judge_not_run_without_the_gpu_feature`）。不打 `CRASH_POINT_REPLAY_JUDGE route=cpu`、不打带 `judge=` 的 PARALLEL 行。
#[must_use]
pub fn crash_point_replay_judge_not_run_line(item_name: &str) -> String {
    crate::gpu_memory_ledger::gpu_not_run_line(
        &format!("crash_state_enumeration::{item_name}"),
        &crate::crash_amplification::judge_not_run_without_the_gpu_feature(true, false)
            .expect("配置开了 GPU、没编进特性的那一种原因总在"),
    )
}

/// 判段的工作线程数与它从哪来（`environment_variable`、`crash_amplification_threads_variable`、`heavy_thread_default`）。
///
/// # Panics
/// 变量设了却不是正整数。
#[must_use]
pub fn crash_point_replay_worker_threads() -> (usize, &'static str) {
    worker_threads_of_the_variable_values(
        process_environment_variable(CRASH_POINT_REPLAY_WORKER_THREADS_ENVIRONMENT_VARIABLE)
            .as_deref(),
        process_environment_variable(
            crate::crash_amplification::CHECKING_THREADS_ENVIRONMENT_VARIABLE,
        )
        .as_deref(),
        process_environment_variable(THREAD_CAP_ENVIRONMENT_VARIABLE).as_deref(),
    )
}

/// [`crash_point_replay_worker_threads`] 的判法，三个变量的值由调用方给（`None` 是没设）：崩溃点重放的变量设了用它，没设用崩溃放量核对线程数的变量，
/// 都没设取每台默认重活线程数（`thread_cap_value` 是 `SINGLEFS_THREAD_CAP` 的值，设了照它，再没设取 `.claude/heavy-thread-default`
/// 的登记值，判法在 `singlefs_verification::heavy_threads`）。设了而不是正整数（0 也不是）就 panic、消息带变量名与原值（经共用解析
/// [`number_from_the_variable_value_panicking_when_unreadable`]），不悄悄退回 1 个线程。
///
/// # Panics
/// 设了的值不是正整数。
#[must_use]
pub fn worker_threads_of_the_variable_values(
    crash_point_replay_threads_value: Option<&str>,
    checking_threads_value: Option<&str>,
    thread_cap_value: Option<&str>,
) -> (usize, &'static str) {
    if let Some(threads) = number_from_the_variable_value_panicking_when_unreadable::<NonZeroUsize>(
        CRASH_POINT_REPLAY_WORKER_THREADS_ENVIRONMENT_VARIABLE,
        crash_point_replay_threads_value,
    ) {
        return (threads.get(), "environment_variable");
    }
    if let Some(threads) = number_from_the_variable_value_panicking_when_unreadable::<NonZeroUsize>(
        crate::crash_amplification::CHECKING_THREADS_ENVIRONMENT_VARIABLE,
        checking_threads_value,
    ) {
        return (threads.get(), "crash_amplification_threads_variable");
    }
    (
        default_heavy_threads_of_the_cap_value(thread_cap_value).get(),
        "heavy_thread_default",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_judge_route_is_named_and_mapped_to_the_conclusion_kind() {
        assert_eq!(CrashPointReplayJudgeRoute::Gpu.name(), "gpu");
        assert_eq!(
            CrashPointReplayJudgeRoute::Gpu.judged_by(),
            JudgedBy::GpuJudge
        );
        let cpu = CrashPointReplayJudgeRoute::Cpu {
            why_not_the_gpu: "gpu_not_enabled_in_the_configuration",
        };
        assert_eq!(cpu.name(), "cpu");
        assert_eq!(cpu.judged_by(), JudgedBy::CpuJudge);
    }

    #[test]
    fn worker_threads_come_from_the_crash_point_replay_variable_then_the_checking_threads_variable_then_the_heavy_thread_default(
    ) {
        assert_eq!(
            worker_threads_of_the_variable_values(Some("3"), Some("5"), Some("7")),
            (3, "environment_variable")
        );
        assert_eq!(
            worker_threads_of_the_variable_values(None, Some("5"), Some("7")),
            (5, "crash_amplification_threads_variable")
        );
        assert_eq!(
            worker_threads_of_the_variable_values(None, None, Some("7")),
            (7, "heavy_thread_default"),
            "两个自己的变量都没设读 SINGLEFS_THREAD_CAP"
        );
        assert_eq!(
            worker_threads_of_the_variable_values(None, None, None),
            (
                crate::heavy_threads::registered_heavy_threads().get(),
                "heavy_thread_default"
            ),
            "都没设取 .claude/heavy-thread-default 的登记值，不取本机核数"
        );
    }

    #[test]
    fn zero_or_unreadable_worker_threads_stop_instead_of_falling_back() {
        for value in ["0", "x", "-1"] {
            let caught_with_the_crash_point_replay_threads_value = std::panic::catch_unwind(|| {
                worker_threads_of_the_variable_values(Some(value), None, None)
            });
            let payload = caught_with_the_crash_point_replay_threads_value
                .err()
                .unwrap_or_else(|| panic!("{value:?} 要停下，不退回 1 个线程"));
            let message = payload
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| {
                    payload
                        .downcast_ref::<&str>()
                        .map(|text| (*text).to_string())
                })
                .unwrap_or_default();
            assert!(
                message.contains("认不出")
                    && message.contains(CRASH_POINT_REPLAY_WORKER_THREADS_ENVIRONMENT_VARIABLE),
                "消息经共用解析，带「变量=原值 认不出」：{message}"
            );
            let caught_with_the_checking_threads_value = std::panic::catch_unwind(|| {
                worker_threads_of_the_variable_values(None, Some(value), None)
            });
            assert!(
                caught_with_the_checking_threads_value.is_err(),
                "崩溃放量那个变量 {value:?} 同样要停下"
            );
        }
    }
}
