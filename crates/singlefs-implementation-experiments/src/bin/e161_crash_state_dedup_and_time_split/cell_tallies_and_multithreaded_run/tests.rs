//! 一格的累计与多线程跑的单测。

use super::*;

/// 线程数：`E161_THREADS` 先读；没设读 `SINGLEFS_THREAD_CAP`；都没设取登记的每台默认重活线程数，不取本机核数。
#[test]
fn threads_read_the_apparatus_threads_variable_first_then_the_heavy_thread_default() {
    assert_eq!(threads_of_the_variable_values(Some("3"), Some("5")), 3);
    assert_eq!(threads_of_the_variable_values(None, Some("5")), 5);
    assert_eq!(
        threads_of_the_variable_values(None, None),
        singlefs_verification::heavy_threads::registered_heavy_threads().get(),
        "都没设取 .claude/heavy-thread-default 的登记值"
    );
}

#[test]
fn content_first_appearance_does_not_depend_on_the_merge_order() {
    let stream = prepare_first_stream();
    let table = TornWriteTable::of(&stream);
    let context = EvaluationContext::new(&stream, &table);
    let plan = LocalPlan::new(
        &stream,
        &table,
        domain_expansions(&stream, LocalExpansion::EveryCombination),
    );
    let stride: Vec<u64> = stride_domain_ordinals(&stream, &plan)
        .into_iter()
        .take(STATES_PER_SLICE * 3)
        .collect();
    let intervals = TrajectoryIntervals {
        starts: stride.iter().step_by(16).copied().collect(),
    };
    let lines_with = |threads: usize| {
        let mut totals = evaluate_stream_domain(&StreamDomainRun {
            label: format!("test-merge-order-{threads}"),
            context: &context,
            plan: &plan,
            ordinals: &stride,
            control_ordinals: &BTreeSet::new(),
            timing: TimingControl::WALL_CLOCK_WITHOUT_BUSY_WAIT,
            threads,
            depth: EvaluationDepth::FourSteps,
        });
        let stores = std::mem::take(&mut totals.arm_stores);
        let mut output = ResultLines::new();
        report_cell(
            &mut output,
            "test-merge-order",
            &totals,
            stores,
            &intervals,
            None,
        )
        .lines_without_timing
    };
    assert_eq!(
        lines_with(1),
        lines_with(4),
        "V6：线程数 1 与 4 的结果行（去掉计时）逐字相同"
    );
}
