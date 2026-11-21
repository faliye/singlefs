//! 判取样态的线程数（按内存上限收）与几条线程的结局按序收齐。

use super::*;

/// 服务交给这一趟的内存上限（字节）所在的环境变量；没设（不经服务起）就不按内存收线程。
const MEMORY_CAP_ENVIRONMENT_VARIABLE: &str = "TASK_SERVER_MEMORY_CAP_BYTES";

/// 内存上限里留给取样线程之外的（两条流的驱动、枚举表、事实表、写文件、引擎与库的连接）：只开一条线程的 E171 取样段，
/// 整个进程的峰值不到 1.5 GiB（`REPLAY_ROW` 的在跑行），这里按它留出余量。
const MEBIBYTES_OUTSIDE_THE_SAMPLING_THREADS: u64 = 2048;

/// 一条取样线程要留的内存：录入段的流字节、读回的流与它的计划、撕裂表、物化的盘、表的字节与读回的那一份、判表与打包判表各一份，
/// 固定脚本流的写流载荷约 7.2 MiB、撕裂载荷约 0.2 MiB（`ops-payload.bin`、`torn-payload.bin`），每样几份，这个数按它们估。
/// 判已知坏镜像与 E170 坏状态的线程同一份数：手里是一张盘的两三份拷贝（已知坏镜像只在新池新建文件流上，载荷约 0.5 MiB；E170 坏状态重建的盘不大于固定脚本流的载荷）、
/// 算摘要的消息、给定状态表的录入字节与读回的那一份、判表与打包判表，没有比取样线程多出来的一样。
const MEBIBYTES_OF_A_SAMPLING_THREAD: u64 = 256;

/// 取样线程的栈：与判定在主线程上跑时同样大（`std::thread::spawn` 缺省的栈比主线程小，恢复与 checker 不该因为换了线程换一种栈）。
const STACK_BYTES_OF_A_SAMPLING_THREAD: usize = 8 * 1024 * 1024;

/// 这一趟判取样态、判坏镜像开几条线程：`--sampling-threads` 给了照它，没给取每台默认重活线程数；再按内存上限与阳性对照收。
///
/// # Panics
/// 同 [`singlefs_verification::heavy_threads::default_heavy_threads`]；服务交的内存上限不是数。
pub(super) fn sampling_threads_of_this_run(switches: RunSwitches) -> NonZeroUsize {
    let requested = switches
        .sampling_threads
        .unwrap_or_else(singlefs_verification::heavy_threads::default_heavy_threads);
    let memory_cap_bytes =
        singlefs_verification::numeric_environment_variable::number_from_the_variable_value_panicking_when_unreadable::<u64>(
            MEMORY_CAP_ENVIRONMENT_VARIABLE,
            singlefs_verification::numeric_environment_variable::process_environment_variable(
                MEMORY_CAP_ENVIRONMENT_VARIABLE,
            )
            .as_deref(),
        );
    sampling_threads_given_the_positive_control(
        sampling_threads_within_the_memory_cap(requested, memory_cap_bytes),
        switches,
    )
}

/// 按内存上限收线程数：上限扣掉 [`MEBIBYTES_OUTSIDE_THE_SAMPLING_THREADS`]，余下的放得下几条 [`MEBIBYTES_OF_A_SAMPLING_THREAD`] 就开几条，
/// 不多于要的、至少一条。线程数只定快慢，不进产物。
fn sampling_threads_within_the_memory_cap(
    requested: NonZeroUsize,
    memory_cap_bytes: Option<u64>,
) -> NonZeroUsize {
    let Some(memory_cap_bytes) = memory_cap_bytes else {
        return requested;
    };
    let mebibytes_for_the_threads =
        (memory_cap_bytes / (1024 * 1024)).saturating_sub(MEBIBYTES_OUTSIDE_THE_SAMPLING_THREADS);
    let threads_that_fit =
        usize::try_from(mebibytes_for_the_threads / MEBIBYTES_OF_A_SAMPLING_THREAD)
            .unwrap_or(usize::MAX);
    NonZeroUsize::new(requested.get().min(threads_that_fit)).unwrap_or(NonZeroUsize::MIN)
}

/// O1-崩溃判官的阳性对照要翻「批 1 第一个读回了文件的态」，先后由序号定：这一趟只开一条线程，别的趟照给的线程数。
fn sampling_threads_given_the_positive_control(
    threads: NonZeroUsize,
    switches: RunSwitches,
) -> NonZeroUsize {
    if switches.positive_control == Some(PositiveControl::CrashJudge) {
        NonZeroUsize::MIN
    } else {
        threads
    }
}

/// 把 `run_count` 个连续段（坏镜像模式里一条算一段）分给至多 `threads` 条线程判，交回的结局按连续段的次序排，与哪条线程先判完无关：线程按次序领下一个连续段，
/// 判完放回它自己的位置。某个连续段的结局叫停（`stops_the_later_runs`）之后不再领新的，交回的到次序上第一个叫停的连续段为止（含它）：
/// 领的次序就是连续段的次序，比它靠前的都领过、都判完了，所以这一份与一条线程按次序走到那里停下的那一份相同；领了而排在它后面的丢掉。
///
/// # Panics
/// `outcome_of_the_run` panic（它自己要接住连续段里的 panic）；起不了线程。
pub(super) fn outcomes_of_the_runs_in_their_order<Outcome: Send>(
    run_count: usize,
    threads: NonZeroUsize,
    stops_the_later_runs: &(dyn Fn(&Outcome) -> bool + Sync),
    outcome_of_the_run: &(dyn Fn(usize) -> Outcome + Sync),
) -> Vec<Outcome> {
    let next_run = AtomicUsize::new(0);
    let later_runs_are_stopped = AtomicBool::new(false);
    let outcomes: Vec<Mutex<Option<Outcome>>> =
        (0..run_count).map(|_run| Mutex::new(None)).collect();
    std::thread::scope(|scope| {
        let workers: Vec<_> = (0..threads.get().min(run_count))
            .map(|_worker| {
                let next_run = &next_run;
                let later_runs_are_stopped = &later_runs_are_stopped;
                let outcomes = &outcomes;
                std::thread::Builder::new()
                    .name("e171-judging".to_string())
                    .stack_size(STACK_BYTES_OF_A_SAMPLING_THREAD)
                    .spawn_scoped(scope, move || loop {
                        if later_runs_are_stopped.load(Ordering::SeqCst) {
                            break;
                        }
                        let run = next_run.fetch_add(1, Ordering::SeqCst);
                        if run >= run_count {
                            break;
                        }
                        let outcome = outcome_of_the_run(run);
                        if stops_the_later_runs(&outcome) {
                            later_runs_are_stopped.store(true, Ordering::SeqCst);
                        }
                        *outcomes[run].lock().unwrap_or_else(PoisonError::into_inner) =
                            Some(outcome);
                    })
                    .expect("起得了判取样态的线程")
            })
            .collect();
        for worker in workers {
            worker
                .join()
                .expect("连续段里的 panic 都在 outcome_of_the_run 里接住了");
        }
    });
    let mut outcomes_in_their_order = Vec::with_capacity(run_count);
    for outcome_of_a_run in outcomes {
        let outcome = outcome_of_a_run
            .into_inner()
            .unwrap_or_else(PoisonError::into_inner)
            .expect("次序上第一个叫停的连续段之前的都领过、判完了：领的次序就是连续段的次序");
        let stops = stops_the_later_runs(&outcome);
        outcomes_in_their_order.push(outcome);
        if stops {
            break;
        }
    }
    outcomes_in_their_order
}

/// 一件里 panic 时接住的载荷：交回主线程，排到它的次序上再原样抛出。
pub(super) type CaughtPanicPayload = Box<dyn std::any::Any + Send>;

/// [`outcomes_of_the_runs_in_their_order`] 按一件一件判、接住每件里的 panic：次序上第一个 panic 的件之后不再领新的，它之前的都判完了。
/// 交回的 `Err` 只可能是最后一个，主线程把它之前的结局照次序收完再 [`std::panic::resume_unwind`]：打出的行与一条线程走到那里 panic 时相同。
///
/// # Panics
/// 起不了线程。
pub(super) fn outcomes_up_to_the_earliest_panic<Outcome: Send>(
    item_count: usize,
    threads: NonZeroUsize,
    outcome_of_the_item: &(dyn Fn(usize) -> Outcome + Sync),
) -> Vec<Result<Outcome, CaughtPanicPayload>> {
    outcomes_of_the_runs_in_their_order(
        item_count,
        threads,
        &|outcome: &Result<Outcome, CaughtPanicPayload>| outcome.is_err(),
        &|item| {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| outcome_of_the_item(item)))
        },
    )
}

#[cfg(test)]
mod tests;
