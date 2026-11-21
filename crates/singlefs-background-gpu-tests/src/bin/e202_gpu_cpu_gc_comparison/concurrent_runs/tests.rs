use super::*;

fn timings_in_milliseconds(
    planned: u64,
    flag_set: u64,
    lock_acquired: u64,
    publish_returned: u64,
) -> UserChangeTimings {
    UserChangeTimings {
        planned: Duration::from_millis(planned),
        flag_set: Duration::from_millis(flag_set),
        lock_acquired: Duration::from_millis(lock_acquired),
        publish_returned: Duration::from_millis(publish_returned),
    }
}

/// 第九节变异「延迟从拿到锁算起」在这里红：前台落后计划 20 毫秒、等锁 10 毫秒、锁里发布 10 毫秒，延迟是 40 毫秒不是 10 毫秒。
#[test]
fn the_latency_counts_from_the_planned_time_and_the_publish_under_the_lock_from_the_lock() {
    let behind_the_plan = timings_in_milliseconds(100, 120, 130, 140);
    assert_eq!(
        latency_of_the_user_change(behind_the_plan),
        Some(Duration::from_millis(40))
    );
    assert_eq!(
        publish_under_the_lock_of_the_user_change(behind_the_plan),
        Some(Duration::from_millis(10))
    );
    assert_eq!(
        wait_from_the_flag_to_the_lock(behind_the_plan),
        Some(Duration::from_millis(10))
    );
    assert_eq!(
        user_changes_failing_the_inclusion_check(&[behind_the_plan]),
        0
    );
    let lock_before_the_flag = timings_in_milliseconds(100, 120, 110, 140);
    assert_eq!(
        user_changes_failing_the_inclusion_check(&[behind_the_plan, lock_before_the_flag]),
        1
    );
}

/// 第 99 百分位取最近秩：1..=100 取 99，1280 个数取第 1268 个，一个数取它自己。
#[test]
fn the_ninety_ninth_percentile_takes_the_nearest_rank() {
    let hundred: Vec<f64> = (1..=100).map(f64::from).collect();
    let distribution = latency_distribution_of(&hundred).expect("有数");
    assert_eq!(distribution.count, 100);
    assert_eq!(distribution.high_percentile, 99.0);
    assert_eq!(distribution.median, 50.5);
    assert_eq!(distribution.maximum, 100.0);
    let pooled: Vec<f64> = (1..=1280).rev().map(f64::from).collect();
    assert_eq!(
        latency_distribution_of(&pooled)
            .expect("有数")
            .high_percentile,
        1268.0
    );
    assert_eq!(
        latency_distribution_of(&[7.0])
            .expect("有数")
            .high_percentile,
        7.0
    );
    assert_eq!(latency_distribution_of(&[]), None);
}

fn concurrent_call_with(
    stop_name: &str,
    started_at_milliseconds: u64,
    queries: &[(u64, AnswerOfTheCallerBeforeTheNextBatch)],
    returned_at_milliseconds: u64,
) -> ConcurrentCallRecord {
    let mut call = call_record_with(1, Vec::new());
    call.stop_name = stop_name.to_string();
    ConcurrentCallRecord {
        call,
        started_at: Duration::from_millis(started_at_milliseconds),
        returned_at: Duration::from_millis(returned_at_milliseconds),
        fsync_queries: queries
            .iter()
            .map(|(at, answer)| (Duration::from_millis(*at), *answer))
            .collect(),
    }
}

/// 甲类第 5 行：答了「在等」就得是这次调用问的最后一次、只答一次、停在让路上；没答过「在等」的调用不核。
#[test]
fn the_yield_anchor_holds_only_when_the_waiting_answer_ends_the_call_with_the_yield_stop() {
    let yield_stop = CompactionRunStop::YieldedToAForegroundFsync.name();
    let budget_stop = "compaction_stopped_publish_budget_spent";
    let yielded = concurrent_call_with(
        yield_stop,
        0,
        &[
            (
                1,
                AnswerOfTheCallerBeforeTheNextBatch::NoForegroundFsyncWaiting,
            ),
            (
                2,
                AnswerOfTheCallerBeforeTheNextBatch::ForegroundFsyncIsWaiting,
            ),
        ],
        3,
    );
    let never_asked_to_wait = concurrent_call_with(
        budget_stop,
        0,
        &[(
            1,
            AnswerOfTheCallerBeforeTheNextBatch::NoForegroundFsyncWaiting,
        )],
        2,
    );
    assert_eq!(
        calls_not_yielding_after_answering_waiting(&[yielded, never_asked_to_wait]),
        0
    );
    let went_on_after_waiting = concurrent_call_with(
        budget_stop,
        0,
        &[
            (
                1,
                AnswerOfTheCallerBeforeTheNextBatch::ForegroundFsyncIsWaiting,
            ),
            (
                2,
                AnswerOfTheCallerBeforeTheNextBatch::NoForegroundFsyncWaiting,
            ),
        ],
        3,
    );
    let waiting_but_another_stop = concurrent_call_with(
        budget_stop,
        0,
        &[(
            1,
            AnswerOfTheCallerBeforeTheNextBatch::ForegroundFsyncIsWaiting,
        )],
        2,
    );
    let waiting_twice = concurrent_call_with(
        yield_stop,
        0,
        &[
            (
                1,
                AnswerOfTheCallerBeforeTheNextBatch::ForegroundFsyncIsWaiting,
            ),
            (
                2,
                AnswerOfTheCallerBeforeTheNextBatch::ForegroundFsyncIsWaiting,
            ),
        ],
        3,
    );
    let trajectory = waiting_answers_trajectory(&[
        went_on_after_waiting,
        waiting_but_another_stop,
        waiting_twice,
    ]);
    assert_eq!(
        trajectory,
        WaitingAnswersTrajectory {
            peak_in_one_call: 2,
            calls_answered_waiting: 3,
            in_the_last_call: 2,
        }
    );
    for broken in [
        concurrent_call_with(
            yield_stop,
            0,
            &[
                (
                    1,
                    AnswerOfTheCallerBeforeTheNextBatch::ForegroundFsyncIsWaiting,
                ),
                (
                    2,
                    AnswerOfTheCallerBeforeTheNextBatch::NoForegroundFsyncWaiting,
                ),
            ],
            3,
        ),
        concurrent_call_with(
            budget_stop,
            0,
            &[(
                1,
                AnswerOfTheCallerBeforeTheNextBatch::ForegroundFsyncIsWaiting,
            )],
            2,
        ),
        concurrent_call_with(
            yield_stop,
            0,
            &[
                (
                    1,
                    AnswerOfTheCallerBeforeTheNextBatch::ForegroundFsyncIsWaiting,
                ),
                (
                    2,
                    AnswerOfTheCallerBeforeTheNextBatch::ForegroundFsyncIsWaiting,
                ),
            ],
            3,
        ),
    ] {
        assert_eq!(calls_not_yielding_after_answering_waiting(&[broken]), 1);
    }
}

/// 第十条第 4 行：一批的上界取相邻两次问前台之间（连同调用开始到头一次问）最长的一段，收尾取最后一次问到返回；前台等锁超过两者之和的两倍才算。
#[test]
fn the_longest_batch_and_tail_come_from_the_gaps_between_the_foreground_queries() {
    let calls = [
        concurrent_call_with(
            "compaction_stopped_publish_budget_spent",
            0,
            &[
                (
                    1,
                    AnswerOfTheCallerBeforeTheNextBatch::NoForegroundFsyncWaiting,
                ),
                (
                    4,
                    AnswerOfTheCallerBeforeTheNextBatch::NoForegroundFsyncWaiting,
                ),
                (
                    10,
                    AnswerOfTheCallerBeforeTheNextBatch::NoForegroundFsyncWaiting,
                ),
            ],
            15,
        ),
        concurrent_call_with("not_started_NothingToDo", 20, &[], 22),
    ];
    let (longest_batch, longest_tail) = longest_batch_and_longest_tail_of_the_calls(&calls);
    assert_eq!(longest_batch, Duration::from_millis(6));
    assert_eq!(longest_tail, Duration::from_millis(5));
    assert!(!the_wait_exceeds_twice_the_longest_batch_and_tail(
        Duration::from_millis(22),
        longest_batch,
        longest_tail
    ));
    assert!(the_wait_exceeds_twice_the_longest_batch_and_tail(
        Duration::from_millis(23),
        longest_batch,
        longest_tail
    ));
}

/// 并发跑的甲类合判：有一次答了「在等」而没让路、前台等锁过了界、某一项记账走错，各判不成立。
#[test]
fn the_class_one_anchors_of_a_concurrent_run_break_on_a_call_not_yielding_or_a_wait_past_the_bound()
{
    let holding_path = PathAnchor {
        compute_records: 3,
        compute_records_on_the_path_of_the_settings: 3,
        batches: 1,
    };
    let anchors = GpuWorkAnchors::default();
    assert!(class_one_anchors_of_a_concurrent_run_hold(
        holding_path,
        ComputeOfTheArm::Cpu,
        anchors,
        0,
        false
    ));
    assert!(!class_one_anchors_of_a_concurrent_run_hold(
        holding_path,
        ComputeOfTheArm::Cpu,
        anchors,
        1,
        false
    ));
    assert!(!class_one_anchors_of_a_concurrent_run_hold(
        holding_path,
        ComputeOfTheArm::Cpu,
        anchors,
        0,
        true
    ));
    assert!(!class_one_anchors_of_a_concurrent_run_hold(
        PathAnchor {
            compute_records: 0,
            compute_records_on_the_path_of_the_settings: 0,
            batches: 1,
        },
        ComputeOfTheArm::Cpu,
        anchors,
        0,
        false
    ));
}

/// 续跑：`name=concurrent_run` 行里的两串纳秒数读回来与写的相同，空串读成一个都没有。
#[test]
fn the_numbers_of_a_concurrent_run_line_read_back_as_written() {
    let (name, fields) = fields_of_a_result_line(
        "E202 name=concurrent_run arm=gpu_one_card repeat=3 latencies_nanoseconds=12,7 publishes_under_the_lock_nanoseconds= void=no",
    )
    .expect("是本装置的结果行");
    assert_eq!(name, "concurrent_run");
    assert_eq!(
        ConcurrentRunNumbers::of_the_fields(&fields),
        Some(ConcurrentRunNumbers {
            arm: ConcurrentArm::GpuOnTheCardWithTheLowestPciAddress,
            repeat: 3,
            latencies: vec![12.0, 7.0],
            publishes_under_the_lock: Vec::new(),
        })
    );
}

/// 并发跑整条路在小池上走一遍（同交替跑那条小池用例的池；8 次用户改动、间隔 5 毫秒，真实的 256 次与 50 毫秒进不了同包测试）：
/// CPU 臂与不跑 GC 臂各一次，读到的等于前台写的，每次改动的延迟不短于锁里发布用的时间，答了「在等」的调用都让了路，每一项记账照设置走。
#[test]
fn the_small_pool_concurrent_run_reads_back_what_the_foreground_wrote_and_every_waiting_answer_yields(
) {
    let image = small_pool_image();
    let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
    let setting = ConcurrentRunSetting {
        image: &image,
        discrete_gpus: &[],
        memory_leaked: &memory_leaked,
        vulkan_instance: None,
        user_changes: 8,
        foreground_interval: Duration::from_millis(5),
    };
    for arm in [ConcurrentArm::Cpu, ConcurrentArm::WithoutCompaction] {
        let run = run_concurrent_once(&setting, arm, 0).expect("CPU 上的两条臂跑得成");
        assert!(run.content_matches_the_expectation, "{}", arm.name());
        assert_eq!(run.latencies.len(), 8);
        assert_eq!(run.publishes_under_the_lock.len(), 8);
        assert_eq!(run.user_changes_failing_the_inclusion_check, 0);
        assert_eq!(run.calls_not_yielding_after_answering_waiting, 0);
        assert!(run.path_anchor.holds(), "{:?}", run.path_anchor);
        assert!(!run.the_foreground_waited_past_the_yield_bound);
        assert_eq!(run.gc_calls > 0, arm.runs_the_compaction());
    }
}

/// 第十条第 4 行照上界取法判（登记第十二节修订第 6 条）：打中记 `hit`，没打中只记「上界口径下没打中」；产物行写明是上界口径。
#[test]
fn the_yield_bound_clause_not_hit_is_reported_under_the_upper_bound_reading() {
    assert_eq!(yield_bound_failure_clause_reading(true), "hit");
    assert_eq!(
        yield_bound_failure_clause_reading(false),
        "not_hit_under_the_upper_bound_reading"
    );
}
