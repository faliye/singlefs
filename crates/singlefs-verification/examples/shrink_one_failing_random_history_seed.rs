//! 收缩一个随机历史种子：一件工具，不是判定，所以不在测试目标里（`cargo test` 跑测试目标时不该跑到它）。
//! 随机历史的默认批与各取样点判红时只报种子，打一行复现命令（`singlefs_verification::history::random_history_reproduction_line`）；
//! 照抄那一行起这个 example，生成的历史与跑法与判红那一段逐项相同，再把它收到最短复现，报告打在标准输出。
//!
//! 起法（复现行原样）：`<复现行里的变量…> task-server/task submit -- cargo run --release -p singlefs-verification --example shrink_one_failing_random_history_seed`，内存额度与线程由服务定。
//! 读的环境变量：`SINGLEFS_RANDOM_HISTORY_SHRINK_SEED` 必给；步数、比重、checker、盘宽、空间准入那几个
//! （`SINGLEFS_RANDOM_HISTORY_OPERATIONS`、`_WEIGHTS`、`_CHECKER`、`_DEVICES`、`_SPACE_ADMISSION`）没设的取默认批的
//! （`RANDOM_HISTORY_DEFAULT_SAMPLING`：30 步、broad、skip、4gib、judged），读法是 `random_history_to_reproduce_from_environment`，
//! 与 `tests/random_histories.rs` 里「复现行喂回入口」那几条自证同一个函数、同一份默认值；
//! `SINGLEFS_RANDOM_HISTORY_THREADS` 默认取每台默认重活线程数（`SINGLEFS_THREAD_CAP`，再没设取 `.claude/heavy-thread-default` 的登记值）。
//!
//! 退出码：0 收缩完、报告已打出；1 那一段跑完没有撞到「已知红」清单外的失败（没有可收缩的）；2 环境变量缺或取值认不得（用法错）。

use std::io::Write as _;
use std::process::ExitCode;

use singlefs_verification::heavy_threads::{
    own_variable_or_default_heavy_threads, THREAD_CAP_ENVIRONMENT_VARIABLE,
};
use singlefs_verification::history::{
    execute_history_with, random_history_to_reproduce_from_environment, shrink_to_reproduction,
    HistoryEnding, NewFindingReport, RANDOM_HISTORY_DEFAULT_SAMPLING,
    RANDOM_HISTORY_WORKER_THREADS_VARIABLE,
};
use singlefs_verification::numeric_environment_variable::process_environment_variable;
use singlefs_verification::SharedStream;

/// 那一段跑完没有撞到新发现：没有可收缩的。
const EXIT_CODE_NO_NEW_FINDING: u8 = 1;
/// 环境变量缺（种子没给）或取值认不得。
const EXIT_CODE_ENVIRONMENT_NOT_USABLE: u8 = 2;

fn main() -> ExitCode {
    let (seed, sampling) = match random_history_to_reproduce_from_environment(
        &process_environment_variable,
        &RANDOM_HISTORY_DEFAULT_SAMPLING,
    ) {
        Ok(seed_and_sampling) => seed_and_sampling,
        Err(error) => {
            eprintln!("{}", error.render());
            return ExitCode::from(EXIT_CODE_ENVIRONMENT_NOT_USABLE);
        }
    };
    let history = sampling.generate(seed);
    let worker_threads = own_variable_or_default_heavy_threads(
        RANDOM_HISTORY_WORKER_THREADS_VARIABLE,
        process_environment_variable(RANDOM_HISTORY_WORKER_THREADS_VARIABLE).as_deref(),
        process_environment_variable(THREAD_CAP_ENVIRONMENT_VARIABLE).as_deref(),
    )
    .get();
    let HistoryEnding::NewFinding {
        signature,
        observation,
    } = execute_history_with(
        &history,
        sampling.execution,
        &SharedStream::new(),
        &mut |_| {},
    )
    .ending
    else {
        eprintln!(
            "种子 {} 的 {} 步（比重 {}；{}）没有撞到新发现",
            history.seed.0,
            sampling.operations_per_history,
            sampling.weights.environment_value(),
            sampling.execution.name()
        );
        return ExitCode::from(EXIT_CODE_NO_NEW_FINDING);
    };
    let shrunk = shrink_to_reproduction(&history, &signature, sampling.execution, worker_threads);
    let report = NewFindingReport {
        signature,
        first_seed: history.seed,
        observation,
        seeds: vec![history.seed],
        shrunk: Some(shrunk),
        sampling,
    }
    .render();
    let mut standard_output = std::io::stdout();
    standard_output
        .write_all(report.as_bytes())
        .and_then(|()| standard_output.flush())
        .expect("报告写进标准输出：标准输出关着就没有地方交出收缩的结果");
    ExitCode::SUCCESS
}
