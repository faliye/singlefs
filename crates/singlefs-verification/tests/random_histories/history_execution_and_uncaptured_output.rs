//! 几段取样与删文件的历史共用的执行：不带池级 checker 跑一段历史，输出直接打到终端。

use super::*;

/// 报告直接写进进程的标准输出，不经 libtest 的捕获：默认批通过时计数照样出现在 `check.sh` 的输出里
/// （`test-discipline.md`「阴性结果要能和「代码没跑到」分开」）。
pub(super) fn print_uncaptured(text: &str) {
    let mut standard_output = std::io::stdout();
    let _ = standard_output.write_all(text.as_bytes());
    let _ = standard_output.flush();
}

/// 一段历史在两块 4 GiB 的盘上跑，harness 不跑每一步的池级 checker（每一步的镜像经钩子交出去，那一遍归给定状态引擎两段）。
pub(super) fn execute_history_without_the_pool_checker(history: &GeneratedHistory) -> HistoryRun {
    execute_history_with(
        history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |_| {},
    )
}
