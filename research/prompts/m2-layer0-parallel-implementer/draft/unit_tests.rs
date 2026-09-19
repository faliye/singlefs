
    /// 单线程逐段逐个子集走（并行之前 `enumerate_layer0_selecting_versions_observing_each_state` 的次序）给出的持久集合，按次序排好。
    fn persisted_sets_walking_segment_by_segment(
        segments: &[Vec<usize>],
        write_count: usize,
        expand: &dyn Fn(usize, &[usize]) -> bool,
    ) -> Vec<Vec<bool>> {
        let mut persisted_sets = Vec::new();
        let mut persisted_before = vec![false; write_count];
        for (segment_index, segment) in segments.iter().enumerate() {
            if expand(segment_index, segment) {
                for subset_mask in 0..(1u64 << segment.len()) - 1 {
                    let mut persisted = persisted_before.clone();
                    for (bit, write_index) in segment.iter().enumerate() {
                        if subset_mask & (1 << bit) != 0 {
                            persisted[*write_index] = true;
                        }
                    }
                    persisted_sets.push(persisted);
                }
            }
            for write_index in segment {
                persisted_before[*write_index] = true;
            }
        }
        persisted_sets.push(persisted_before);
        persisted_sets
    }

    /// 按序号取状态（并行切片靠它）与逐段逐个子集走，给出同一串持久集合：展开的段夹着不展开的段、不展开的段在头上和尾上都算。
    #[test]
    fn the_state_plan_hands_out_the_same_persisted_sets_in_the_same_order_as_walking_segment_by_segment(
    ) {
        let segments = vec![
            vec![0, 1],
            vec![2],
            vec![3, 4, 5],
            vec![6, 7],
            vec![8, 9, 10, 11],
            vec![12],
        ];
        let write_count = 13;
        let every_segment = |_segment_index: usize, _segment: &[usize]| true;
        let skip_three_write_segment_and_the_ends =
            |segment_index: usize, segment: &[usize]| {
                segment.len() != 3 && segment_index != 0 && segment_index != 5
            };
        let only_the_four_write_segment = |_segment_index: usize, segment: &[usize]| segment.len() == 4;
        let expansions: [&dyn Fn(usize, &[usize]) -> bool; 3] = [
            &every_segment,
            &skip_three_write_segment_and_the_ends,
            &only_the_four_write_segment,
        ];
        for expand in expansions {
            let walked = persisted_sets_walking_segment_by_segment(&segments, write_count, expand);
            let plan = Layer0StatePlan::new(&segments, expand);
            assert_eq!(
                plan.state_count,
                u64::try_from(walked.len()).expect("状态数"),
                "状态数与逐段走的相同"
            );
            let by_ordinal: Vec<Vec<bool>> = (0..plan.state_count)
                .map(|ordinal| plan.persisted_writes_of_state(ordinal, write_count))
                .collect();
            assert_eq!(by_ordinal, walked, "第 k 个状态就是逐段走到的第 k 个");
        }
    }

    /// 切片首尾相接、从 0 起、到状态数止、一片都不空：丢一片或两片重叠，这里与用例里「状态数等于闭式」的断言都红。
    /// 默认切法下全量两条流切出来的片数不少于线程数（每个线程都领得到片）。
    #[test]
    fn state_slices_cover_every_state_exactly_once_in_ordinal_order() {
        let thread_counts = [1usize, 3, 32, 200];
        let fixed_lengths = [1u64, 7, 16, u64::MAX];
        for state_count in [1u64, 2, 15, 16, 17, 22, 108, 1000, 262_165, 2_104_413] {
            let mut parallelisms: Vec<Layer0Parallelism> = thread_counts
                .iter()
                .map(|worker_threads| Layer0Parallelism {
                    worker_threads: NonZeroUsize::new(*worker_threads).expect("非 0"),
                    worker_threads_source: Layer0WorkerThreadsSource::GivenByCaller,
                    slice_length: Layer0SliceLength::ScaledToWorkerThreads,
                })
                .collect();
            parallelisms.extend(fixed_lengths.iter().map(|states_per_slice| Layer0Parallelism {
                worker_threads: NonZeroUsize::MIN,
                worker_threads_source: Layer0WorkerThreadsSource::GivenByCaller,
                slice_length: Layer0SliceLength::StatesPerSlice(
                    NonZeroU64::new(*states_per_slice).expect("非 0"),
                ),
            }));
            for parallelism in parallelisms {
                let slices = state_slices(state_count, &parallelism);
                let mut next_ordinal = 0u64;
                for slice in &slices {
                    assert_eq!(
                        slice.start, next_ordinal,
                        "{state_count} 个状态、{parallelism:?}：片首接上一片的尾"
                    );
                    assert!(slice.end > slice.start, "{parallelism:?}：没有空片");
                    next_ordinal = slice.end;
                }
                assert_eq!(
                    next_ordinal, state_count,
                    "{parallelism:?}：最后一片止于状态数"
                );
                if state_count >= 262_165 {
                    assert!(
                        slices.len() >= parallelism.worker_threads.get()
                            || matches!(
                                parallelism.slice_length,
                                Layer0SliceLength::StatesPerSlice(_)
                            ),
                        "{state_count} 个状态、{parallelism:?}：片数 {} 不少于线程数",
                        slices.len()
                    );
                }
            }
        }
    }

    /// 并片：计数相加，「第一处」取前面那一片的；前面那一片没有才取后面的。
    #[test]
    fn absorbing_a_following_slice_adds_counts_and_keeps_the_earlier_first_violation() {
        let mut earlier = Layer0Tally {
            states: 3,
            violations: 1,
            first_violation: Some("前面那一片的".to_string()),
            ..Layer0Tally::default()
        };
        earlier.checker_evaluated_states.insert("I-1.1", 3);
        earlier
            .checker_first_violation
            .insert("I-1.1", "前面那一片的 I-1.1".to_string());
        let mut following = Layer0Tally {
            states: 5,
            violations: 2,
            first_violation: Some("后面那一片的".to_string()),
            ignored_violations: 1,
            first_ignored_violation: Some("后面那一片的 Ignore".to_string()),
            ..Layer0Tally::default()
        };
        following.checker_evaluated_states.insert("I-1.1", 5);
        following.checker_evaluated_states.insert("I-3.1", 2);
        following
            .checker_first_violation
            .insert("I-1.1", "后面那一片的 I-1.1".to_string());
        following
            .checker_first_violation
            .insert("I-3.1", "后面那一片的 I-3.1".to_string());
        earlier.absorb_following_slice(following);
        assert_eq!((earlier.states, earlier.violations, earlier.ignored_violations), (8, 3, 1));
        assert_eq!(earlier.first_violation.as_deref(), Some("前面那一片的"));
        assert_eq!(
            earlier.first_ignored_violation.as_deref(),
            Some("后面那一片的 Ignore"),
            "前面那一片没有，取后面的"
        );
        assert_eq!(earlier.checker_evaluated_states.get("I-1.1"), Some(&8));
        assert_eq!(earlier.checker_evaluated_states.get("I-3.1"), Some(&2));
        assert_eq!(
            earlier.checker_first_violation.get("I-1.1").map(String::as_str),
            Some("前面那一片的 I-1.1")
        );
        assert_eq!(
            earlier.checker_first_violation.get("I-3.1").map(String::as_str),
            Some("后面那一片的 I-3.1")
        );
    }
