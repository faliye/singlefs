
// ───── 攻方腿（Opus）m2-supp3-item1-code-r2 · Y5：并行版对单线程原写法的差分（只追加在仓副本的这份用例文件末尾） ─────
// 单线程原写法照 00c9d4f 的 `enumerate_layer0_selecting_versions_observing_each_state` 逐行抄（只用公开的 `evaluate_state_for_versions`）。
// 每一趟随机取：段的切法（段长 0–5、可以漏写、可以同一个写进两段、可以乱序）、每段展不展开、版本表（原样或把 B 的内容换掉造违例）、
// 线程数（1、2、3、5、8、32）、片长（按线程数定、或每片 1–7 个状态、或整条流一片）、有没有观察者；比计数（整个结构逐项相等，含每一处「第一处」）
// 与观察者看到的（持久集合、看 journal 那一遍恢复的报告）逐个相等。

fn opus_reference_walk(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: &[PublishedVersion],
    expand: &dyn Fn(usize, &[usize]) -> bool,
    observe_state: &mut dyn FnMut(&CrashImage<'_>, &RecoveryReport),
) -> Layer0Tally {
    let mut tally = Layer0Tally::default();
    let mut persisted_before = vec![false; writes.len()];
    for (segment_index, segment) in segments.iter().enumerate() {
        if expand(segment_index, segment) {
            let full_mask = (1u64 << segment.len()) - 1;
            for mask in 0..full_mask {
                let mut persisted = persisted_before.clone();
                for (bit, write_index) in segment.iter().enumerate() {
                    if mask & (1 << bit) != 0 {
                        persisted[*write_index] = true;
                    }
                }
                let consulted_report = evaluate_state_for_versions(
                    base, writes, persisted.clone(), judged_root_index, versions, &mut tally,
                );
                observe_state(&CrashImage { base, writes, persisted }, &consulted_report);
            }
        }
        for write_index in segment {
            persisted_before[*write_index] = true;
        }
    }
    let consulted_report = evaluate_state_for_versions(
        base, writes, persisted_before.clone(), judged_root_index, versions, &mut tally,
    );
    observe_state(&CrashImage { base, writes, persisted: persisted_before }, &consulted_report);
    tally
}

struct OpusMix(u64);
impl OpusMix {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound
    }
}

#[test]
#[ignore = "攻方探针，副本上手动跑"]
fn opus_r2_parallel_matches_the_single_threaded_walk() {
    let trials: u64 = std::env::var("OPUS_TRIALS").map_or(40, |text| text.parse().expect("趟数"));
    let first_trial: u64 = std::env::var("OPUS_FIRST_TRIAL").map_or(0, |text| text.parse().expect("趟号"));
    let prepared = prepare("opus-y5", Script::ReuseAfterRaisingFloor);
    let mut wrong_versions = prepared.versions.clone();
    wrong_versions
        .iter_mut()
        .find(|version| version.instance == InstanceGeneration(1) && version.checkpoint_txg == CheckpointTxg(4))
        .expect("B")
        .content = third_content();
    let write_count = prepared.writes.len();
    let mut total_states = 0u64;
    let mut trials_with_violations = 0u64;
    let mut trials_with_checker_violations = 0u64;
    for trial in first_trial..first_trial + trials {
        let mut mix = OpusMix(trial.wrapping_mul(0x1234_5678_9ABC_DEF1) ^ 0xA5A5);
        // 切段：按写的次序走，段长 0–5；每个写以 1/12 的概率被漏掉、1/12 的概率同时进下一段；每段以 1/6 的概率段内倒序。
        let mut segments: Vec<Vec<usize>> = Vec::new();
        let mut index = 0usize;
        while index < write_count {
            let length = usize::try_from(mix.below(6)).unwrap();
            let mut segment = Vec::new();
            for _ in 0..length {
                if index >= write_count {
                    break;
                }
                match mix.below(12) {
                    0 => {}
                    1 => {
                        segment.push(index);
                        if index + 1 < write_count {
                            segment.push(index + 1);
                        }
                    }
                    _ => segment.push(index),
                }
                index += 1;
            }
            if mix.below(6) == 0 {
                segment.reverse();
            }
            segment.truncate(6);
            segments.push(segment);
        }
        let expand_mask: Vec<bool> = segments.iter().map(|_| mix.below(4) == 0).collect();
        let expand = |segment_index: usize, _segment: &[usize]| expand_mask[segment_index];
        let versions = if mix.below(2) == 0 { &prepared.versions } else { &wrong_versions };
        let threads = [1usize, 2, 3, 5, 8, 32][usize::try_from(mix.below(6)).unwrap()];
        let slice_length = match mix.below(4) {
            0 => Layer0SliceLength::ScaledToWorkerThreads,
            1 => Layer0SliceLength::StatesPerSlice(NonZeroU64::MAX),
            _ => Layer0SliceLength::StatesPerSlice(NonZeroU64::new(1 + mix.below(7)).unwrap()),
        };
        let with_observer = mix.below(2) == 0;
        let judged = prepared.judged_root_index;
        let mut reference_seen: Vec<(Vec<bool>, String)> = Vec::new();
        let reference = opus_reference_walk(
            &prepared.base, &prepared.writes, &segments, judged, versions, &expand,
            &mut |image, report| reference_seen.push((image.persisted.clone(), format!("{report:?}"))),
        );
        let mut parallel_seen: Vec<(Vec<bool>, String)> = Vec::new();
        let parallelism = Layer0Parallelism {
            worker_threads: NonZeroUsize::new(threads).unwrap(),
            worker_threads_source: Layer0WorkerThreadsSource::GivenByCaller,
            slice_length,
        };
        let parallel = if with_observer {
            enumerate_layer0_in_state_slices(
                &prepared.base, &prepared.writes, &segments, judged, versions, &expand, parallelism,
                Some(&mut |image: &CrashImage<'_>, report: &RecoveryReport| {
                    parallel_seen.push((image.persisted.clone(), format!("{report:?}")));
                }),
            )
        } else {
            enumerate_layer0_in_state_slices(
                &prepared.base, &prepared.writes, &segments, judged, versions, &expand, parallelism, None,
            )
        };
        total_states += reference.states;
        if reference.violations + reference.ignored_violations > 0 {
            trials_with_violations += 1;
        }
        if reference.checker_violated_states.values().sum::<u64>() > 0 {
            trials_with_checker_violations += 1;
        }
        println!(
            "OPUS_Y5 trial={trial} segments={} expanded={} states={} threads={threads} slice={slice_length:?} observer={with_observer} violations={} ignored={} checker_violated={} equal_tally={} equal_observed={}",
            segments.len(),
            expand_mask.iter().filter(|e| **e).count(),
            reference.states,
            reference.violations,
            reference.ignored_violations,
            reference.checker_violated_states.values().sum::<u64>(),
            parallel == reference,
            !with_observer || parallel_seen == reference_seen
        );
        assert_eq!(parallel, reference, "趟 {trial}：计数与每一处「第一处」");
        if with_observer {
            assert!(parallel_seen == reference_seen, "趟 {trial}：观察者看到的次序与内容");
        }
    }
    println!("OPUS_Y5_SUMMARY trials={trials} first_trial={first_trial} states={total_states} trials_with_violations={trials_with_violations} trials_with_checker_violations={trials_with_checker_violations}");
}
