//! 验证抽象仲裁模型到真实 HashMap、回调和批处理接口的衔接；不调用 GPU。
use singlefs_background_gpu::two_kernel_reconciliation::{
    ConsecutiveUncertaintyCounts, EntryDecision, SameEntryKeyTwiceInOneBatch,
    TwoKernelOutcomesOfAnEntry,
};

/// 历史模型里注入的那一批失败（带历史号与轮号，原样交回才算没动），或比对报的重复键。
#[derive(Debug, PartialEq, Eq)]
enum BatchFailureOfTheHistory {
    Injected { history: usize, round: usize },
    SameEntryKeyTwiceInOneBatch(SameEntryKeyTwiceInOneBatch),
}

impl From<SameEntryKeyTwiceInOneBatch> for BatchFailureOfTheHistory {
    fn from(repeated: SameEntryKeyTwiceInOneBatch) -> Self {
        Self::SameEntryKeyTwiceInOneBatch(repeated)
    }
}

fn entry_number(key: usize) -> i32 {
    i32::try_from(key).expect("用例里的条目号都小于 i32::MAX")
}

fn entry(key: usize, agrees: bool) -> TwoKernelOutcomesOfAnEntry<usize, i32> {
    TwoKernelOutcomesOfAnEntry {
        entry_key: key,
        outcome_by_one_kernel: 10 + entry_number(key),
        outcome_by_the_independently_written_kernel: if agrees {
            10 + entry_number(key)
        } else {
            -1
        },
    }
}

#[test]
fn interleaved_entries_failures_empty_batches_and_remounts_match_the_history_model() {
    // 七种事件：两条目的“一致/分歧”、GPU 批失败、空批、重挂。
    // 枚举全部 7^6 条历史；参照只记录每条目是否有一次尚未解决的分歧。
    for history in 0..7usize.pow(6) {
        let mut events = history;
        let mut counts = ConsecutiveUncertaintyCounts::with_no_entry_uncertain();
        let mut pending = [false; 2];
        let mut actual_cpu_calls = 0;
        let mut expected_cpu_calls = 0;
        for round in 0..6 {
            let event = events % 7;
            events /= 7;
            match event {
                0..=3 => {
                    let key = event / 2;
                    let agrees = event % 2 == 0;
                    let cpu_expected = !agrees && pending[key];
                    let batch = counts
                        .reconcile_unless_the_batch_failed::<_, SameEntryKeyTwiceInOneBatch>(
                            Ok(vec![entry(key, agrees)]),
                            |position| {
                                assert_eq!(position, 0);
                                actual_cpu_calls += 1;
                                900 + entry_number(key)
                            },
                        )
                        .unwrap();
                    let expected = if agrees {
                        EntryDecision::ConfirmedByBothKernels(10 + entry_number(key))
                    } else if cpu_expected {
                        expected_cpu_calls += 1;
                        EntryDecision::JudgedByTheCpuReference(900 + entry_number(key))
                    } else {
                        EntryDecision::UncertainThisRound
                    };
                    pending[key] = !agrees && !cpu_expected;
                    assert_eq!(
                        batch.decisions,
                        vec![expected],
                        "history={history}, round={round}"
                    );
                    assert_eq!(batch.confirmed_entries, usize::from(agrees));
                    assert_eq!(batch.uncertain_entries, usize::from(pending[key]));
                    assert_eq!(
                        batch.entries_judged_by_the_cpu_reference,
                        usize::from(cpu_expected)
                    );
                }
                4 => {
                    let failure = counts.reconcile_unless_the_batch_failed::<i32, _>(
                        Err(BatchFailureOfTheHistory::Injected { history, round }),
                        |_| panic!("GPU 批失败不能调用 CPU"),
                    );
                    assert_eq!(
                        failure,
                        Err(BatchFailureOfTheHistory::Injected { history, round })
                    );
                }
                5 => {
                    let empty = counts
                        .reconcile_unless_the_batch_failed::<i32, SameEntryKeyTwiceInOneBatch>(
                            Ok(vec![]),
                            |_| panic!("空批不能调用 CPU"),
                        )
                        .unwrap();
                    assert!(empty.decisions.is_empty());
                    assert_eq!(
                        (
                            empty.confirmed_entries,
                            empty.uncertain_entries,
                            empty.entries_judged_by_the_cpu_reference
                        ),
                        (0, 0, 0)
                    );
                }
                6 => {
                    counts = ConsecutiveUncertaintyCounts::with_no_entry_uncertain();
                    pending = [false; 2];
                }
                _ => unreachable!(),
            }
            for (key, is_pending) in pending.iter().enumerate() {
                assert_eq!(
                    counts.consecutive_uncertain_rounds_of(&key),
                    u32::from(*is_pending),
                    "history={history}, round={round}, key={key}"
                );
            }
            assert_eq!(
                actual_cpu_calls, expected_cpu_calls,
                "history={history}, round={round}"
            );
        }
    }
}

#[test]
fn permuting_distinct_entries_preserves_keyed_results_and_cpu_calls() {
    for initial in 0..4 {
        for agreements in 0..4 {
            for order in [[0usize, 1], [1, 0]] {
                let mut counts = ConsecutiveUncertaintyCounts::with_no_entry_uncertain();
                for key in 0..2 {
                    if initial & (1 << key) != 0 {
                        counts
                            .reconcile_the_two_kernel_outcomes(vec![entry(key, false)], |_| {
                                panic!("首轮分歧不能交 CPU")
                            })
                            .expect("一批一条，键不重复");
                    }
                }
                let mut callbacks = Vec::new();
                let batch = counts
                    .reconcile_the_two_kernel_outcomes(
                        order
                            .iter()
                            .map(|key| entry(*key, agreements & (1 << key) != 0))
                            .collect(),
                        |position| {
                            callbacks.push(position);
                            900 + entry_number(order[position])
                        },
                    )
                    .expect("两条键不同");
                let expected_callbacks: Vec<_> = order
                    .iter()
                    .enumerate()
                    .filter(|(_, key)| {
                        initial & (1 << **key) != 0 && agreements & (1 << **key) == 0
                    })
                    .map(|(position, _)| position)
                    .collect();
                assert_eq!(callbacks, expected_callbacks);
                for (position, key) in order.into_iter().enumerate() {
                    let agrees = agreements & (1 << key) != 0;
                    let was_pending = initial & (1 << key) != 0;
                    let expected = if agrees {
                        EntryDecision::ConfirmedByBothKernels(10 + entry_number(key))
                    } else if was_pending {
                        EntryDecision::JudgedByTheCpuReference(900 + entry_number(key))
                    } else {
                        EntryDecision::UncertainThisRound
                    };
                    assert_eq!(batch.decisions[position], expected);
                    assert_eq!(
                        counts.consecutive_uncertain_rounds_of(&key),
                        u32::from(!agrees && !was_pending)
                    );
                }
                assert_eq!(
                    batch.confirmed_entries
                        + batch.uncertain_entries
                        + batch.entries_judged_by_the_cpu_reference,
                    2
                );
                assert_eq!(batch.entries_judged_by_the_cpu_reference, callbacks.len());
            }
        }
    }
}
