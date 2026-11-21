//! 判取样态的线程数（按内存上限收）与几条线程的结局按序收齐的单测。

use super::*;

/// 让几个连续段按 `finishing_order` 的次序判完：轮到谁谁才交回，还没轮到的等着。要这几个连续段同时在跑（线程数不少于连续段数）。
struct RunsFinishingInAGivenOrder {
    finishing_order: Vec<usize>,
    finished: std::sync::Mutex<usize>,
    one_finished: std::sync::Condvar,
}

impl RunsFinishingInAGivenOrder {
    fn new(finishing_order: &[usize]) -> Self {
        Self {
            finishing_order: finishing_order.to_vec(),
            finished: std::sync::Mutex::new(0),
            one_finished: std::sync::Condvar::new(),
        }
    }

    fn wait_for_the_turn_of(&self, run: usize) {
        let mut finished = self.finished.lock().expect("这把锁里不 panic");
        while self.finishing_order[*finished] != run {
            finished = self.one_finished.wait(finished).expect("这把锁里不 panic");
        }
        *finished += 1;
        self.one_finished.notify_all();
    }
}

#[test]
fn outcomes_come_back_in_run_order_when_the_later_runs_finish_earlier() {
    let turns = RunsFinishingInAGivenOrder::new(&[3, 2, 1, 0]);
    let outcomes = outcomes_of_the_runs_in_their_order(
        4,
        SEVERAL_THREADS,
        &|_outcome: &usize| false,
        &|run| {
            turns.wait_for_the_turn_of(run);
            run
        },
    );
    assert_eq!(outcomes, vec![0, 1, 2, 3], "按连续段的次序，不按判完的次序");
}

#[test]
fn on_one_thread_no_run_after_a_stopping_run_is_taken() {
    let taken = std::sync::Mutex::new(Vec::new());
    let outcomes = outcomes_of_the_runs_in_their_order(
        5,
        NonZeroUsize::MIN,
        &|outcome: &usize| *outcome == 2,
        &|run| {
            taken.lock().expect("这把锁里不 panic").push(run);
            run
        },
    );
    assert_eq!(outcomes, vec![0, 1, 2], "到叫停的连续段为止，含它");
    assert_eq!(
        *taken.lock().expect("这把锁里不 panic"),
        vec![0, 1, 2],
        "叫停之后不再领新的"
    );
}

#[test]
fn the_earliest_stopping_run_ends_the_outcomes_when_a_later_one_stopped_earlier() {
    let turns = RunsFinishingInAGivenOrder::new(&[3, 0, 2, 1]);
    let outcomes = outcomes_of_the_runs_in_their_order(
        4,
        SEVERAL_THREADS,
        &|outcome: &usize| *outcome == 1 || *outcome == 3,
        &|run| {
            turns.wait_for_the_turn_of(run);
            run
        },
    );
    assert_eq!(
        outcomes,
        vec![0, 1],
        "次序上第一个叫停的是连续段 1，判完得早的连续段 3 与排在后面的都丢掉"
    );
}

#[test]
fn the_memory_cap_limits_the_sampling_threads_and_at_least_one_is_left() {
    const MEBIBYTE: u64 = 1024 * 1024;
    let requested = NonZeroUsize::new(31).expect("31 不是 0");
    let threads_under = |memory_cap_mebibytes: u64| {
        sampling_threads_within_the_memory_cap(requested, Some(memory_cap_mebibytes * MEBIBYTE))
            .get()
    };
    assert_eq!(
        sampling_threads_within_the_memory_cap(requested, None).get(),
        31,
        "没有上限照要的开"
    );
    assert_eq!(threads_under(1024 * 1024), 31, "上限放得下时不多于要的");
    assert_eq!(threads_under(9216), 28, "(9216 - 2048) / 256");
    assert_eq!(threads_under(4096), 8, "(4096 - 2048) / 256");
    assert_eq!(threads_under(1024), 1, "连留给线程之外的都不够时仍开一条");
}

#[test]
fn only_the_positive_control_of_the_crash_judge_takes_one_thread() {
    let switches_of = |positive_control| RunSwitches {
        positive_control,
        ..RunSwitches::default()
    };
    assert_eq!(
        sampling_threads_given_the_positive_control(
            SEVERAL_THREADS,
            switches_of(Some(PositiveControl::CrashJudge))
        ),
        NonZeroUsize::MIN,
        "批 1 第一个读回了文件的态要一条线程按序号走才说得清"
    );
    for positive_control in [
        None,
        Some(PositiveControl::ComparisonOfTheCalls),
        Some(PositiveControl::PoolChecker),
    ] {
        assert_eq!(
            sampling_threads_given_the_positive_control(
                SEVERAL_THREADS,
                switches_of(positive_control)
            ),
            SEVERAL_THREADS,
            "{positive_control:?}"
        );
    }
}
