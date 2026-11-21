use super::*;

/// 一个观察者的几把版本（成批读录入段那一条单测用）：几把都由 `name` 现算，判段的池级 checker 取 CPU 判器。
fn observer_versions_named(name: &str) -> ObserverVersions {
    let version_of = |part: &str| {
        ConclusionVersion(sha256_digest(
            format!("{part} of the observer {name}\n").as_bytes(),
        ))
    };
    ObserverVersions {
        observer: version_of("recording version"),
        observer_code: version_of("code version"),
        pool_checker: JudgedBy::CpuJudge,
        pool_checker_version: version_of("pool checker version"),
        conclusion: version_of("conclusion version"),
    }
}

/// 场景：那条两个扇区的小流，每一块挂七个观察者，各落在一种情形上：观察者结论存着不红；存着红；没存、录入块那一行在；没存、那一行也不在；
/// 存着不红而在强制现判表里、那一行不在；在强制现判表里、那一行在；存着红、那一行也在。
/// 预期：成批读回的每一项（[`observer_blocks_recorded_or_concluded`]）与逐块问 [`observer_block_is_recorded_or_concluded`] 答的相同、
/// 读的那一刻在不在强制现判表里也对得上；答案照情形钉死：在、不在、在、不在、不在、在、不在。录入段照这个答案定哪几块不再录，
/// 答案相同，写进库的行就相同。
/// 判别力（`crates/mutations.tsv`）：成批读把存着红的观察者结论当录过，这条红。
#[test]
fn observer_blocks_read_together_answer_as_each_block_asked_alone() {
    let (plan, block_count) = plan_of_the_two_sector_flow_with_its_block_count();
    assert!(block_count >= 1, "前提：计划里至少一块");
    let (_directory, mut store) = an_empty_library_for("observer-blocks-read-together");
    let recorded_or_concluded_of_each_case = [true, false, true, false, false, true, false];
    let observers: Vec<ObserverVersions> = (0..recorded_or_concluded_of_each_case.len())
        .map(|case| observer_versions_named(&format!("case {case}")))
        .collect();
    let mut blocks_and_observers: Vec<(VerdictBlockKey, ObserverVersions)> = Vec::new();
    let mut expected: Vec<bool> = Vec::new();
    for (point_index, point) in plan.points.iter().enumerate() {
        for local in point.blocks() {
            let key = point.block_key(&local);
            for (case, observer_versions) in observers.iter().enumerate() {
                let as_stored = ObserverForTheReuseSentinels {
                    holds_each_point: vec![true; plan.points.len()],
                    conclusion: observer_versions.conclusion,
                    input_row: (ConclusionKind::Observer, observer_versions.observer),
                };
                let red_item_of_the_conclusion = match case {
                    0 | 4 => Some(None),
                    1 | 6 => Some(Some("unit_test_red_item")),
                    _ => None,
                };
                if let Some(red_item) = red_item_of_the_conclusion {
                    store_an_observer_conclusion_of_the_block(
                        &mut store,
                        &plan,
                        &as_stored,
                        point_index,
                        &local,
                        red_item,
                    );
                }
                if matches!(case, 2 | 5 | 6) {
                    store_the_observer_input_row_of_the_block(
                        &mut store,
                        &plan,
                        &as_stored,
                        point_index,
                        &local,
                    );
                }
                if matches!(case, 4 | 5) {
                    store.also_force_these_conclusion_blocks_to_be_judged_again([(
                        key.clone(),
                        ConclusionKind::Observer,
                        observer_versions.conclusion,
                    )]);
                }
                blocks_and_observers.push((key.clone(), *observer_versions));
                expected.push(recorded_or_concluded_of_each_case[case]);
            }
        }
    }
    let read_together =
        observer_blocks_recorded_or_concluded(&store, &blocks_and_observers).expect("读得了库");
    let asked_alone: Vec<ObserverBlockRecordedOrConcludedAtTheRead> = blocks_and_observers
        .iter()
        .map(
            |(key, observer_versions)| ObserverBlockRecordedOrConcludedAtTheRead {
                was_forced_at_the_read: store.conclusion_block_is_forced_to_be_judged_again(
                    key,
                    &ConclusionKind::Observer,
                    &observer_versions.conclusion,
                ),
                is_recorded_or_concluded: observer_block_is_recorded_or_concluded(
                    &store,
                    key,
                    observer_versions,
                )
                .expect("读得了库"),
            },
        )
        .collect();
    assert_eq!(read_together, asked_alone, "成批读回的每一项与逐块问的相同");
    assert_eq!(
        read_together
            .iter()
            .map(|at_the_read| at_the_read.is_recorded_or_concluded)
            .collect::<Vec<bool>>(),
        expected,
        "答案照情形钉死：在、不在、在、不在、不在、在、不在"
    );
}

/// 观察者录入段的枚举表一趟只建一份、各核对线程借同一份（`tests/crash_enumeration_make_filesystem_over_a_used_pool_stream.rs` 按甲二展开的那一条：
/// 逐线程各建一份时一份逾 1.5 GiB，八条线程同时建就过 16G）。一块切成几段、各一条线程看，每个状态交给观察者的镜像都叠在同一份
/// 枚举写表上（`CrashImage::writes` 指着同一处）。每条线程看它头一个状态时，等到至少两条线程都到了才往下走：逐线程各建一份时，
/// 那几份在同一刻都活着，地址必不相同，所以「各建一份」一定判红，不靠线程碰巧错开。
/// 核对线程数在用例里定成 2、不读环境：用例调录入段里收核对线程数的那一层（`record_the_observers_of_the_flow_with_threads`）交 2；
/// 单测进程里拿到的是它与里层份额（`crate::crash_library::InnerThreadsTaken`）里小的那个，里层份额至少
/// `crate::crash_library::FEWEST_INNER_THREADS_OF_THE_SHARE`（2）条，拿到的就是 2，一块三个状态切成两段。
#[test]
fn the_observer_recording_threads_of_one_block_share_one_enumeration_table_of_the_flow() {
    let (_directory, mut store) =
        an_empty_library_for("observer-threads-share-one-enumeration-table");
    // 两条核对线程：录入段把一块三个状态切成两段、各一条线程看
    let threads = 2;
    let seen = std::sync::Mutex::new(ThreadsAndEnumeratedWritesSeenByTheObserver::default());
    let a_second_thread_arrived = std::sync::Condvar::new();
    let observe = |_ordinal: u64,
                   image: &CrashImage<'_>,
                   _report: &RecoveryReport,
                   counts: &mut CrashPointReplayObserverCounts| {
        counts.add("images_seen", 1);
        let mut seen_so_far = seen
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        seen_so_far
            .enumerated_writes_address_of_each_state
            .push(image.writes.as_ptr().addr());
        let is_the_first_state_of_this_thread =
            seen_so_far.threads.insert(std::thread::current().id());
        if !is_the_first_state_of_this_thread {
            return;
        }
        // 上界一分钟：线程数不到两条时等不来第二条，到点照样往下走，由下面的断言报
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
        while seen_so_far.threads.len() < 2 {
            let now = std::time::Instant::now();
            if now >= deadline {
                break;
            }
            seen_so_far = a_second_thread_arrived
                .wait_timeout(seen_so_far, deadline - now)
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .0;
        }
        a_second_thread_arrived.notify_all();
    };
    let versions = [versions_of_the_unit_test_observer([0; DIGEST_BYTES])];
    let recorded = a_flow_of_two_writes_to_two_sectors();
    let tally = recorded.with_the_flow(|flow| {
        let plan = plan_crash_points(flow);
        let observers = [ObserverOnTheNodes {
            observer_name: ObserverName::new("unit_test_thread_sharing_observer")
                .expect("名字合规矩"),
            observer_code_file: OBSERVER_UNIT_TEST_SETTING_FILE.to_string(),
            extra_inputs_digest: [0; DIGEST_BYTES],
            nodes: vec![NodeOfAnObserver {
                setting_file: OBSERVER_UNIT_TEST_SETTING_FILE.to_string(),
                entry_name: OBSERVER_UNIT_TEST_POINT_NAME.to_string(),
            }],
            observe: ObserverCode::CountsOnly(&observe),
        }];
        record_the_observers_of_the_flow_with_threads(
            &LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut store),
            flow,
            &plan,
            &observers,
            &versions,
            &|_| true,
            &|| false,
            threads,
        )
        .expect("录入段写得进库")
    });
    assert!(!tally.stopped_early, "没叫停，录完了");
    let seen = seen
        .into_inner()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    assert_eq!(
        seen.enumerated_writes_address_of_each_state.len(),
        3,
        "段里两个写的任意真子集三个状态，观察者每个都看了一次"
    );
    assert!(
        seen.threads.len() >= 2,
        "一块三个状态切成了至少两段、各一条线程看（核对线程数 {threads}），读到 {} 条",
        seen.threads.len()
    );
    let distinct_enumerated_writes: std::collections::BTreeSet<usize> = seen
        .enumerated_writes_address_of_each_state
        .iter()
        .copied()
        .collect();
    assert_eq!(
            distinct_enumerated_writes.len(),
            1,
            "各线程交给观察者的镜像叠在同一份枚举写表上（一趟只建一份、各线程借用），读到 {} 份：{distinct_enumerated_writes:?}",
            distinct_enumerated_writes.len()
        );
}

/// 场景：这个进程里开着的那一份库（一次性库）上，观察者的录入段在那条两个扇区的小流上录一个只记计数的观察者，观察者头一次看之前先睡
/// 200 毫秒；它睡着的时候另一条线程取同一条连接的锁。
/// 预期：等锁不超过 50 毫秒（录入段只在问块录过没有、写基底与录好的块那几下取锁，看的时候手里不握锁）；录入段照常录完。
/// 判别力（`crates/mutations.tsv`）：「判定库锁按次取放 观察者录入段改回整段握锁」这一行让这一条红。
#[test]
fn another_thread_takes_the_library_lock_while_the_observer_recording_stage_is_observing() {
    // 一次性库留着：要一把这个进程里开着的那一份的真锁
    let disposable =
        crate::crash_library::DisposableLibrary::new("lock-free-while-the-observer-records")
            .unwrap_or_else(|refusal| panic!("{refusal}"));
    let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
    let library = LibraryOpenInThisProcess::open_or_share().expect("开得了库");
    let the_observer_has_started_sleeping = std::sync::atomic::AtomicBool::new(false);
    let observe = |_ordinal: u64,
                   _image: &CrashImage<'_>,
                   _report: &RecoveryReport,
                   counts: &mut CrashPointReplayObserverCounts| {
        if !the_observer_has_started_sleeping.swap(true, std::sync::atomic::Ordering::SeqCst) {
            std::thread::sleep(TIME_THE_SLOW_JUDGE_OR_OBSERVER_SLEEPS);
        }
        counts.add("states_seen", 1);
    };
    let versions = [versions_of_the_unit_test_observer([0; DIGEST_BYTES])];
    let recorded = a_flow_of_two_writes_to_two_sectors();
    let (waited, tally) = recorded.with_the_flow(|flow| {
        let plan = plan_crash_points(flow);
        let observers = [ObserverOnTheNodes {
            observer_name: ObserverName::new("unit_test_slow_observer").expect("名字合规矩"),
            observer_code_file: OBSERVER_UNIT_TEST_SETTING_FILE.to_string(),
            extra_inputs_digest: [0; DIGEST_BYTES],
            nodes: vec![NodeOfAnObserver {
                setting_file: OBSERVER_UNIT_TEST_SETTING_FILE.to_string(),
                entry_name: OBSERVER_UNIT_TEST_POINT_NAME.to_string(),
            }],
            observe: ObserverCode::CountsOnly(&observe),
        }];
        wait_for_the_library_lock_while_the_judge_or_the_observer_sleeps(
            &library,
            &the_observer_has_started_sleeping,
            || {
                record_the_observers_of_the_flow(
                    &LibraryTakenAtEachReadOrWrite::SharedInThisProcess(&library),
                    flow,
                    &plan,
                    &observers,
                    &versions,
                    &|_| true,
                    &|| false,
                )
            },
        )
    });
    let tally = tally.expect("录入段写得进库");
    assert!(
        waited <= LONGEST_WAIT_FOR_THE_LIBRARY_LOCK_MEANWHILE,
        "观察者睡着的时候另一条线程等了 {waited:?} 才取到同一条连接的锁：录入段整段握着锁"
    );
    assert_eq!(
        (tally.blocks_recorded_here, tally.stopped_early),
        (1, false),
        "录入段照常录完观察者挂着的那一块"
    );
}

/// 观察者录入段叫停那条单测的小流里，管前一段与管后一段的两个点的名字。
const OBSERVER_STOP_UNIT_TEST_EARLIER_POINT_NAME: &str = "earlier_pair_of_writes";
const OBSERVER_STOP_UNIT_TEST_LATER_POINT_NAME: &str = "later_pair_of_writes";

/// 一条小流：一块盘，四次写各写一个扇区（字节两两不同），切成两段、一段两个写（各展开任意真子集），两段各归一个点。
/// 两个点各三个状态，加末节点一个状态：三个点各一块，共七个状态。
fn a_flow_of_two_segments_of_two_writes_to_four_sectors() -> RecordedFlow {
    let device = singlefs_core::address::DeviceIdentity(1);
    let write_to = |sector: u64, byte: u8| RetainedWrite {
        device,
        kind: StepKind::UnitWrite,
        is_force_unit_access: false,
        offset: singlefs_core::address::DeviceOffsetInBytes(
            sector * crate::memory_pool::SECTOR_BYTES,
        ),
        contents: WrittenContents::Bytes(vec![byte; 512]),
    };
    let span_of = |name: &str, writes: Range<usize>| CrashPointSpan {
        name: name.to_string(),
        code_file_in_repository: OBSERVER_UNIT_TEST_SETTING_FILE.to_string(),
        writes,
        ignored: false,
    };
    RecordedFlow {
        base: MemoryPool::with_devices(&[device], 1 << 20),
        writes: vec![
            write_to(16, 5),
            write_to(20, 7),
            write_to(24, 9),
            write_to(28, 11),
        ],
        segments: vec![vec![0, 1], vec![2, 3]],
        judged_root_index: 0,
        versions: Vec::new(),
        versions_without_a_file: Vec::new(),
        segment_expansions: vec![
            CrashPointReplaySegmentExpansion::EveryProperSubset,
            CrashPointReplaySegmentExpansion::EveryProperSubset,
        ],
        crash_points: vec![
            span_of(OBSERVER_STOP_UNIT_TEST_EARLIER_POINT_NAME, 0..2),
            span_of(OBSERVER_STOP_UNIT_TEST_LATER_POINT_NAME, 2..4),
        ],
        segment_slices: vec![None, None],
    }
}

/// 观察者录入段在 [`a_flow_of_two_segments_of_two_writes_to_four_sectors`] 上的一趟：录入段的账、叫停被问了几次、观察者看过的状态序号，
/// 连同这条流的计划里每个点的名字与序号区间、每个点有几块、录完之后计划里的每一块（按计划的次序）库里有没有这个观察者录的那一份。
struct ObserverRecordingOfTheFourSectorFlow {
    tally: ObserverRecordingTally,
    stop_asks: u64,
    ordinals_observed: BTreeSet<u64>,
    name_and_ordinals_of_each_point: Vec<(String, Range<u64>)>,
    blocks_of_each_point: Vec<usize>,
    recorded_of_each_block: Vec<bool>,
}

/// 只记计数的观察者挂在那条小流的三个点上（两个点与末节点），照键复用录一趟。
/// 叫停：`stop_at_the_ask` 是 `Some(k)` 时第 k 次被问起交真（之前交假），`None` 时从不交真。
fn record_the_observer_of_the_four_sector_flow(
    store: &mut VerdictStore,
    stop_at_the_ask: Option<u64>,
) -> ObserverRecordingOfTheFourSectorFlow {
    let stop_asks = std::sync::atomic::AtomicU64::new(0);
    let stop_is_requested = || {
        let ask = stop_asks.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
        stop_at_the_ask.is_some_and(|stopping_ask| ask >= stopping_ask)
    };
    let ordinals_observed = std::sync::Mutex::new(BTreeSet::new());
    let observe = |ordinal: u64,
                   _image: &CrashImage<'_>,
                   _report: &RecoveryReport,
                   counts: &mut CrashPointReplayObserverCounts| {
        counts.add("images_seen", 1);
        ordinals_observed
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(ordinal);
    };
    let versions = [versions_of_the_unit_test_observer([0; DIGEST_BYTES])];
    let node_of = |entry_name: &str| NodeOfAnObserver {
        setting_file: OBSERVER_UNIT_TEST_SETTING_FILE.to_string(),
        entry_name: entry_name.to_string(),
    };
    let recorded = a_flow_of_two_segments_of_two_writes_to_four_sectors();
    let (tally, name_and_ordinals_of_each_point, blocks_of_each_point, recorded_of_each_block) =
        recorded.with_the_flow(|flow| {
            let plan = plan_crash_points(flow);
            let observers = [ObserverOnTheNodes {
                observer_name: ObserverName::new("unit_test_stopped_recording_observer")
                    .expect("名字合规矩"),
                observer_code_file: OBSERVER_UNIT_TEST_SETTING_FILE.to_string(),
                extra_inputs_digest: [0; DIGEST_BYTES],
                nodes: vec![
                    node_of(OBSERVER_STOP_UNIT_TEST_EARLIER_POINT_NAME),
                    node_of(OBSERVER_STOP_UNIT_TEST_LATER_POINT_NAME),
                    node_of(END_OF_FLOW_NODE_NAME),
                ],
                observe: ObserverCode::CountsOnly(&observe),
            }];
            let tally = record_the_observers_of_the_flow(
                &LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(store),
                flow,
                &plan,
                &observers,
                &versions,
                &|_| true,
                &stop_is_requested,
            )
            .expect("录入段读写得了库");
            let kind = ConclusionKind::Observer;
            let mut recorded_of_each_block = Vec::new();
            for point in &plan.points {
                for local in point.blocks() {
                    recorded_of_each_block.push(
                        store
                            .read_observer_block(
                                &point.block_key(&local),
                                &kind,
                                &versions[0].observer,
                            )
                            .expect("库读得了")
                            .is_some(),
                    );
                }
            }
            (
                tally,
                plan.points
                    .iter()
                    .map(|point| (point.name.clone(), point.ordinals.clone()))
                    .collect(),
                plan.points
                    .iter()
                    .map(|point| point.blocks().len())
                    .collect(),
                recorded_of_each_block,
            )
        });
    ObserverRecordingOfTheFourSectorFlow {
        tally,
        stop_asks: stop_asks.into_inner(),
        ordinals_observed: ordinals_observed
            .into_inner()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
        name_and_ordinals_of_each_point,
        blocks_of_each_point,
        recorded_of_each_block,
    }
}

/// 场景：只记计数的观察者挂在两段四写那条小流的三个点上（三块，每点一块），录入段每录一块之前问一次叫停，叫停在第二次被问时交真；
/// 同一份库、同一份输入再录一趟，不叫停。
/// 预期：头一趟录完第一块、停在第二块前面：叫停只被问了两次，库里只有第一块，后两块没录，账标着提早停了，观察者只看过第一块的状态；
/// 再录那一趟（被打断后重跑同一条命令就是续跑）第一块照库里的复用（算录过、观察者不再看它的状态），只补录后两块，观察者只看过后两块的状态。
/// 判别力：`crates/mutations.tsv` 里「观察者录入段内层每录一块之前不问叫停」那一行（删掉 `record_the_observers_of_the_flow`
/// 里每块之前那一次问）让这一条红在头一趟的账上（三块照录到底、没标提早停）。
#[test]
fn a_stop_asked_before_each_observer_block_leaves_the_later_blocks_unrecorded_and_the_rerun_records_only_them(
) {
    let (_directory, mut store) = an_empty_library_for("observer-recording-stopped-per-block");
    let stopped = record_the_observer_of_the_four_sector_flow(&mut store, Some(2));
    let point_names: Vec<&str> = stopped
        .name_and_ordinals_of_each_point
        .iter()
        .map(|(name, _ordinals)| name.as_str())
        .collect();
    assert_eq!(
        (
            point_names,
            stopped.blocks_of_each_point.clone(),
            stopped
                .name_and_ordinals_of_each_point
                .last()
                .map(|(_name, ordinals)| ordinals.end),
        ),
        (
            vec![
                OBSERVER_STOP_UNIT_TEST_EARLIER_POINT_NAME,
                OBSERVER_STOP_UNIT_TEST_LATER_POINT_NAME,
                END_OF_FLOW_NODE_NAME,
            ],
            vec![1, 1, 1],
            Some(7),
        ),
        "前提：这条小流两个点加末节点，每点一块，共七个状态"
    );
    let ordinals_of_the_points = |points: Range<usize>| -> BTreeSet<u64> {
        stopped.name_and_ordinals_of_each_point[points]
            .iter()
            .flat_map(|(_name, ordinals)| ordinals.clone())
            .collect()
    };
    assert_eq!(
        (
            stopped.tally.stopped_early,
            stopped.tally.blocks_recorded_here,
            stopped.tally.blocks_already_recorded,
            stopped.recorded_of_each_block.clone(),
        ),
        (true, 1, 0, vec![true, false, false]),
        "叫停在第二次被问时交真：录完第一块、停在第二块前面，账标着提早停了，库里只有第一块：{:?}",
        stopped.tally
    );
    assert_eq!(
        stopped.stop_asks, 2,
        "每录一块之前问一次：第一块前、第二块前各一次，交真之后不再问"
    );
    assert_eq!(
        stopped.ordinals_observed,
        ordinals_of_the_points(0..1),
        "观察者只看过第一块的状态，叫停之后的块一个状态都没看"
    );
    let resumed = record_the_observer_of_the_four_sector_flow(&mut store, None);
    assert_eq!(
        (
            resumed.tally.stopped_early,
            resumed.tally.blocks_already_recorded,
            resumed.tally.blocks_recorded_here,
            resumed.recorded_of_each_block.clone(),
        ),
        (false, 1, 2, vec![true, true, true]),
        "续跑：第一块照库里的复用，只补录后两块，录完三块都在库里：{:?}",
        resumed.tally
    );
    assert_eq!(
        resumed.ordinals_observed,
        ordinals_of_the_points(1..3),
        "续跑时观察者只看后两块的状态，录过的第一块不再看"
    );
}

/// 场景：那条一个扇区两次写同一份字节的小流，挂一个只记计数的观察者（崩溃点与末节点），空库里录、判一趟（两块的观察者结论存进库）；
/// 再换一把观察者版本（录入段那一行按它存）而结论那一把不变，像结论在库里、录入段那一行不在库里的块；复用哨兵抽中崩溃点那一块强制现判。
/// 预期：回录入段只给强制现判那一块重录（交回「录了」）：它在新的那一把观察者版本下录进库；末节点那一块不重录
/// （「抽中库里没有录入块的块，回录入段重录」）。
/// 判别力（`crates/mutations.tsv`）：「复用哨兵：强制现判那一块缺录入段那一行也不回录入段重录」「复用哨兵：强制现判的观察者结论在库里就算录过」
/// 两行各让这一条红。
#[test]
fn the_forced_block_missing_its_observer_input_row_is_recorded_again_and_only_it() {
    let (_directory, mut store) = an_empty_library_for("forced-block-recorded-again");
    let count_the_state = |_ordinal: u64,
                           _image: &CrashImage<'_>,
                           _report: &RecoveryReport,
                           counts: &mut CrashPointReplayObserverCounts| {
        counts.add("states_seen", 1);
    };
    a_flow_of_two_writes_of_the_same_bytes_to_one_sector().with_the_flow(|flow| {
        let plan = plan_crash_points(flow);
        let observers = [ObserverOnTheNodes {
            observer_name: ObserverName::new("unit_test_forced_block_observer")
                .expect("名字合规矩"),
            observer_code_file: OBSERVER_UNIT_TEST_SETTING_FILE.to_string(),
            extra_inputs_digest: [4; DIGEST_BYTES],
            nodes: vec![
                NodeOfAnObserver {
                    setting_file: OBSERVER_UNIT_TEST_SETTING_FILE.to_string(),
                    entry_name: OBSERVER_UNIT_TEST_POINT_NAME.to_string(),
                },
                NodeOfAnObserver {
                    setting_file: OBSERVER_UNIT_TEST_SETTING_FILE.to_string(),
                    entry_name: END_OF_FLOW_NODE_NAME.to_string(),
                },
            ],
            observe: ObserverCode::CountsOnly(&count_the_state),
        }];
        let first_versions = versions_of_the_unit_test_observer([4; DIGEST_BYTES]);
        record_the_observers_of_the_flow(
            &LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut store),
            flow,
            &plan,
            &observers,
            &[first_versions],
            &|_| true,
            &|| false,
        )
        .expect("录入段写得进库");
        judge_the_observers_of_the_flow(
            &LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut store),
            &plan,
            &observers,
            &[first_versions],
            &PoolCheckerOfTheImagesAfterTheObservers::OnTheCpu,
            &|_| true,
        )
        .expect("判段判得了");
        let keys: Vec<VerdictBlockKey> = plan
            .points
            .iter()
            .flat_map(|point| {
                point
                    .blocks()
                    .into_iter()
                    .map(|local| point.block_key(&local))
                    .collect::<Vec<_>>()
            })
            .collect();
        assert_eq!(keys.len(), 2, "前提：小流两块（崩溃点一块、末节点一块）");
        for key in &keys {
            assert!(
                observer_conclusion_is_reused(&store, key, &first_versions.conclusion)
                    .expect("读得了库"),
                "前提：头一趟两块的观察者结论都存进库、不红"
            );
        }
        let versions_of_another_input_row = ObserverVersions {
            observer: ConclusionVersion(sha256_digest(
                b"another observer version of the forced block unit test\n",
            )),
            observer_code: first_versions.observer_code,
            pool_checker: first_versions.pool_checker,
            pool_checker_version: first_versions.pool_checker_version,
            conclusion: first_versions.conclusion,
        };
        let forced_local = plan.points[0].blocks()[0].clone();
        let sentinels = ReuseSentinelsOfTheRun {
            seed: 0,
            candidate_count: 1,
            sentinels: Vec::new(),
            forced: Some((
                ReuseSentinelBlock {
                    block_sequence: 0,
                    point_index: 0,
                    local: forced_local,
                    key: (
                        keys[0].clone(),
                        ConclusionKind::ImageJudgement,
                        ConclusionVersion([0; DIGEST_BYTES]),
                    ),
                    observer_keys: vec![(
                        keys[0].clone(),
                        ConclusionKind::Observer,
                        first_versions.conclusion,
                    )],
                },
                ConclusionSpelledOutInVerdictVectors {
                    verdict_vector_of_each_state: Vec::new(),
                    violations: Vec::new(),
                    count_vector: BlockCountVector::default(),
                    recovery_words: None,
                    abandoned_roots_named_by_the_pool_checker: None,
                },
            )),
            forced_observer_conclusions: Vec::new(),
            offered_before_judging: 0,
        };
        let forced_observer_conclusion = (
            keys[0].clone(),
            ConclusionKind::Observer,
            first_versions.conclusion,
        );
        store.also_force_these_conclusion_blocks_to_be_judged_again([
            forced_observer_conclusion.clone()
        ]);
        let recorded_again = record_the_observers_of_the_forced_block_again(
            &LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut store),
            flow,
            &plan,
            &observers,
            &[versions_of_another_input_row],
            &sentinels,
            &|| false,
        )
        .expect("录入段写得进库");
        store.stop_forcing_these_conclusion_blocks_to_be_judged_again([forced_observer_conclusion]);
        assert!(recorded_again, "强制现判那一块缺录入段那一行：回录入段重录");
        let input_row_of = |key: &VerdictBlockKey| {
            store
                .read_observer_block(
                    key,
                    &ConclusionKind::Observer,
                    &versions_of_another_input_row.observer,
                )
                .expect("读得了库")
                .is_some()
        };
        assert_eq!(
            (input_row_of(&keys[0]), input_row_of(&keys[1])),
            (true, false),
            "只有强制现判那一块在新的那一把观察者版本下录进库，末节点那一块不重录"
        );
    });
}

/// 这条小流每一块的块键（崩溃点一块、末节点一块）。
fn block_keys_of_the_plan(plan: &CrashPlan) -> Vec<VerdictBlockKey> {
    plan.points
        .iter()
        .flat_map(|point| {
            point
                .blocks()
                .into_iter()
                .map(|local| point.block_key(&local))
                .collect::<Vec<_>>()
        })
        .collect()
}

/// 场景：那条一个扇区两次写同一份字节的小流挂一个没在 `OBSERVER_ENTRIES` 登记的只记计数的观察者（判法单元拒算、照判不入库）。
/// 录入在一个进程里（一个库连接，进程标记甲）跑完、连接关掉；判段在另一个进程里（同一份库另开一个连接，进程标记乙）经两处判段共用的那一处
/// （[`judge_the_observers_then_the_reuse_sentinels`]）判。对照：同一份库再开一个连接（进程标记丙），同一个进程里先录后判。
/// 预期：判段进程把两块补录一遍（补录的账两块、四个状态）、判完，观察者的结果与同一个进程录判的逐项相同；同一个进程录判的那一趟一块都不补；
/// 三个标记下这个观察者的录入块与结论一行都不在库里（另开一个不登记照判不入库的连接读）。
/// 判别力（`crates/mutations.tsv`）：「判段不补录照判不入库的观察者」那一行让这一条红（判段报录入段那一块不在库里）。
#[test]
fn an_observer_refused_its_judging_units_recorded_in_another_process_is_recorded_again_and_judged_as_in_one_process(
) {
    let (directory, mut recording_store) =
        an_empty_library_for("observer-refused-recorded-in-another-process");
    let count_the_state = |_ordinal: u64,
                           _image: &CrashImage<'_>,
                           _report: &RecoveryReport,
                           counts: &mut CrashPointReplayObserverCounts| {
        counts.add("states_seen", 1);
    };
    let observers = [the_counting_observer_without_an_observer_entry(
        &count_the_state,
    )];
    let (recording_versions, recording_kept_out) =
        versions_of_the_observer_without_an_observer_entry_under_the_process_marker(
            &observers[0],
            JudgedBy::CpuJudge,
            1,
        );
    let (judging_versions, judging_kept_out) =
        versions_of_the_observer_without_an_observer_entry_under_the_process_marker(
            &observers[0],
            JudgedBy::CpuJudge,
            2,
        );
    let (one_process_versions, one_process_kept_out) =
        versions_of_the_observer_without_an_observer_entry_under_the_process_marker(
            &observers[0],
            JudgedBy::CpuJudge,
            3,
        );
    assert_ne!(
        recording_versions.observer, judging_versions.observer,
        "前提：两个进程的占位不同，判段照自己的占位读不到录入进程的那一份"
    );
    a_flow_of_two_writes_of_the_same_bytes_to_one_sector().with_the_flow(|flow| {
        let plan = plan_crash_points(flow);
        let keys = block_keys_of_the_plan(&plan);
        assert_eq!(keys.len(), 2, "前提：小流两块（崩溃点一块、末节点一块）");
        recording_store.keep_the_rows_of_these_versions_out_of_the_library(recording_kept_out);
        let recording = record_the_observers_of_the_flow(
            &LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut recording_store),
            flow,
            &plan,
            &observers,
            &[recording_versions],
            &|_| true,
            &|| false,
        )
        .expect("录入段读写得了库");
        assert_eq!(recording.blocks_recorded_here, 2, "前提：录入进程录了两块");
        drop(recording_store);
        let mut judging_store = directory
            .open_store(crate::verdict_store::PruneServerFailureHandling::ReturnTheError)
            .expect("开得了一次性库");
        judging_store.keep_the_rows_of_these_versions_out_of_the_library(judging_kept_out);
        let (observing_across_two_processes, problems_across_two_processes, recorded_again) =
            judge_the_observers_then_the_reuse_sentinels(
                &LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut judging_store),
                flow,
                &plan,
                Some((
                    observers.as_slice(),
                    std::slice::from_ref(&judging_versions),
                    &PoolCheckerOfTheImagesAfterTheObservers::OnTheCpu,
                )),
                &|_| true,
                None,
                &|| false,
            )
            .expect("判段补录之后判得完");
        assert_eq!(
            (
                recorded_again.blocks_recorded_here,
                recorded_again.states_recorded_here
            ),
            (2, 4),
            "判段进程照自己的占位把两块（四个状态）补录一遍：{recorded_again:?}"
        );
        let mut one_process_store = directory
            .open_store(crate::verdict_store::PruneServerFailureHandling::ReturnTheError)
            .expect("开得了一次性库");
        one_process_store.keep_the_rows_of_these_versions_out_of_the_library(one_process_kept_out);
        record_the_observers_of_the_flow(
            &LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut one_process_store),
            flow,
            &plan,
            &observers,
            &[one_process_versions],
            &|_| true,
            &|| false,
        )
        .expect("录入段读写得了库");
        let (observing_in_one_process, problems_in_one_process, recorded_again_in_one_process) =
            judge_the_observers_then_the_reuse_sentinels(
                &LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut one_process_store),
                flow,
                &plan,
                Some((
                    observers.as_slice(),
                    std::slice::from_ref(&one_process_versions),
                    &PoolCheckerOfTheImagesAfterTheObservers::OnTheCpu,
                )),
                &|_| true,
                None,
                &|| false,
            )
            .expect("同一个进程录判判得完");
        assert_eq!(
            recorded_again_in_one_process,
            ObserverRecordingTally::default(),
            "同一个进程录判的那一趟块都在，一块都不补"
        );
        assert_eq!(
            (
                observing_across_two_processes,
                problems_across_two_processes
            ),
            (observing_in_one_process, problems_in_one_process),
            "录入与判段各在一个进程里，判出的与同一个进程录判的逐项相同"
        );
        drop(judging_store);
        drop(one_process_store);
        let reading_store = directory
            .open_store(crate::verdict_store::PruneServerFailureHandling::ReturnTheError)
            .expect("开得了一次性库");
        for versions in [recording_versions, judging_versions, one_process_versions] {
            for key in &keys {
                assert_eq!(
                    (
                        reading_store
                            .read_observer_block(key, &ConclusionKind::Observer, &versions.observer)
                            .expect("读得了库")
                            .is_some(),
                        reading_store
                            .read_conclusion_block(
                                key,
                                &ConclusionKind::Observer,
                                &versions.conclusion
                            )
                            .expect("读得了库")
                            .is_some()
                    ),
                    (false, false),
                    "照判不入库：这个观察者的录入块与结论一行都不进库"
                );
            }
        }
    });
}

/// 场景：那条小流挂一个只记计数的观察者，版本是单测另造的、没登记照判不入库（登记了、不拒算的那一种）。录入段没跑就经两处判段共用的那一处判；
/// 再录一遍、判一遍。
/// 预期：没录就判，判段不补录，照旧报录入段那一块不在库里、出路是先跑录入段（不点名拒算），库里这一把录入块一块都没有；
/// 录过再判，补录的账全零（一块都不补），判得完。
/// 判别力（`crates/mutations.tsv`）：「判段补录不挑照判不入库的观察者」那一行让这一条红（补录了不拒算的观察者）。
#[test]
fn an_observer_keyed_into_the_library_is_never_recorded_again_by_the_judging_stage() {
    let (_directory, mut store) =
        an_empty_library_for("observer-keyed-into-the-library-not-recorded-again");
    let count_the_state = |_ordinal: u64,
                           _image: &CrashImage<'_>,
                           _report: &RecoveryReport,
                           counts: &mut CrashPointReplayObserverCounts| {
        counts.add("states_seen", 1);
    };
    let observers = [the_counting_observer_without_an_observer_entry(
        &count_the_state,
    )];
    let versions = versions_of_the_unit_test_observer([6; DIGEST_BYTES]);
    a_flow_of_two_writes_of_the_same_bytes_to_one_sector().with_the_flow(|flow| {
        let plan = plan_crash_points(flow);
        let judge_once = |library_of_this_case: &mut VerdictStore| {
            judge_the_observers_then_the_reuse_sentinels(
                &LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(library_of_this_case),
                flow,
                &plan,
                Some((
                    observers.as_slice(),
                    std::slice::from_ref(&versions),
                    &PoolCheckerOfTheImagesAfterTheObservers::OnTheCpu,
                )),
                &|_| true,
                None,
                &|| false,
            )
        };
        let refusal = judge_once(&mut store).expect_err("录入段没跑：判段读不到录入块");
        assert!(
            refusal.contains("先跑录入段") && !refusal.contains("拒算"),
            "不拒算的观察者照旧报先跑录入段、不点名拒算：{refusal}"
        );
        for key in block_keys_of_the_plan(&plan) {
            assert!(
                store
                    .read_observer_block(&key, &ConclusionKind::Observer, &versions.observer)
                    .expect("读得了库")
                    .is_none(),
                "判段不补录不拒算的观察者：库里这一把录入块一块都没有"
            );
        }
        record_the_observers_of_the_flow(
            &LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut store),
            flow,
            &plan,
            &observers,
            &[versions],
            &|_| true,
            &|| false,
        )
        .expect("录入段读写得了库");
        let (_observing, _problems, recorded_again) =
            judge_once(&mut store).expect("录过之后判得完");
        assert_eq!(
            recorded_again,
            ObserverRecordingTally::default(),
            "不拒算的观察者判段一块都不补"
        );
    });
}

/// 场景：那条小流挂那个没登记的观察者（判法单元拒算），录入在一个进程里跑完、连接关掉；判段另开一个连接（另一个进程标记），
/// 经两处判段共用的那一处判，叫停一直是真的。
/// 预期：补录停在第一块之前，判段不判观察者、交回叫停那一句；判段这个连接里这个观察者一块录入块都没有。
/// 判别力：两处判段共用的那一处、补录那一层把收到的叫停换成永不叫停，补录就录进两块、往下判完，这一条红
/// （`every_layer_handed_the_stop_request_hands_it_on_or_asks_it` 那两层只进名单，靠这一条）。
#[test]
fn a_stop_while_recording_again_the_observers_kept_out_of_the_library_leaves_them_unjudged() {
    let (directory, mut recording_store) =
        an_empty_library_for("observer-refused-recorded-again-stopped");
    let count_the_state = |_ordinal: u64,
                           _image: &CrashImage<'_>,
                           _report: &RecoveryReport,
                           counts: &mut CrashPointReplayObserverCounts| {
        counts.add("states_seen", 1);
    };
    let observers = [the_counting_observer_without_an_observer_entry(
        &count_the_state,
    )];
    let (recording_versions, recording_kept_out) =
        versions_of_the_observer_without_an_observer_entry_under_the_process_marker(
            &observers[0],
            JudgedBy::CpuJudge,
            1,
        );
    let (judging_versions, judging_kept_out) =
        versions_of_the_observer_without_an_observer_entry_under_the_process_marker(
            &observers[0],
            JudgedBy::CpuJudge,
            2,
        );
    a_flow_of_two_writes_of_the_same_bytes_to_one_sector().with_the_flow(|flow| {
        let plan = plan_crash_points(flow);
        recording_store.keep_the_rows_of_these_versions_out_of_the_library(recording_kept_out);
        record_the_observers_of_the_flow(
            &LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut recording_store),
            flow,
            &plan,
            &observers,
            &[recording_versions],
            &|_| true,
            &|| false,
        )
        .expect("录入段读写得了库");
        drop(recording_store);
        let mut judging_store = directory
            .open_store(crate::verdict_store::PruneServerFailureHandling::ReturnTheError)
            .expect("开得了一次性库");
        judging_store.keep_the_rows_of_these_versions_out_of_the_library(judging_kept_out);
        let refusal = judge_the_observers_then_the_reuse_sentinels(
            &LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut judging_store),
            flow,
            &plan,
            Some((
                observers.as_slice(),
                std::slice::from_ref(&judging_versions),
                &PoolCheckerOfTheImagesAfterTheObservers::OnTheCpu,
            )),
            &|_| true,
            None,
            &|| true,
        )
        .expect_err("补录被叫停：判段不判观察者");
        assert!(refusal.contains("叫停了"), "交回叫停那一句：{refusal}");
        for key in block_keys_of_the_plan(&plan) {
            assert!(
                judging_store
                    .read_observer_block(
                        &key,
                        &ConclusionKind::Observer,
                        &judging_versions.observer
                    )
                    .expect("读得了库")
                    .is_none(),
                "补录停在第一块之前：判段这个连接里一块录入块都没有"
            );
        }
    });
}

/// 场景：那条小流挂那个没登记的观察者（判法单元拒算、照判不入库），录入段没跑，直接判观察者（判段补录之后仍读不到的那一处）；
/// 对照：同一个观察者换成单测另造的、不照判不入库的版本。
/// 预期：拒算的那一趟报错点名「这个观察者的判法单元拒算（没在 OBSERVER_ENTRIES 登记）」、不叫人先跑录入段；不拒算的照旧报先跑录入段。
/// 判别力（`crates/mutations.tsv`）：「判段读不到录入块时不分拒算」那一行让这一条红。
#[test]
fn a_missing_block_of_an_observer_refused_its_judging_units_names_the_missing_observer_entry() {
    let (_directory, mut store) = an_empty_library_for("observer-refused-missing-block-sentence");
    let count_the_state = |_ordinal: u64,
                           _image: &CrashImage<'_>,
                           _report: &RecoveryReport,
                           counts: &mut CrashPointReplayObserverCounts| {
        counts.add("states_seen", 1);
    };
    let observers = [the_counting_observer_without_an_observer_entry(
        &count_the_state,
    )];
    let (refused_versions, versions_kept_out) =
        versions_of_the_observer_without_an_observer_entry_under_the_process_marker(
            &observers[0],
            JudgedBy::CpuJudge,
            1,
        );
    let versions_keyed_into_the_library = versions_of_the_unit_test_observer([6; DIGEST_BYTES]);
    store.keep_the_rows_of_these_versions_out_of_the_library(versions_kept_out);
    a_flow_of_two_writes_of_the_same_bytes_to_one_sector().with_the_flow(|flow| {
        let plan = plan_crash_points(flow);
        let mut judge_under = |versions: ObserverVersions| {
            judge_the_observers_of_the_flow(
                &LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut store),
                &plan,
                &observers,
                &[versions],
                &PoolCheckerOfTheImagesAfterTheObservers::OnTheCpu,
                &|_| true,
            )
            .expect_err("录入段没跑：判段读不到录入块")
        };
        let refused_sentence = judge_under(refused_versions);
        assert!(
            refused_sentence.contains("这个观察者的判法单元拒算（没在 OBSERVER_ENTRIES 登记）")
                && !refused_sentence.contains("先跑录入段"),
            "拒算的观察者点名没在 OBSERVER_ENTRIES 登记、不叫人先跑录入段：{refused_sentence}"
        );
        let keyed_sentence = judge_under(versions_keyed_into_the_library);
        assert!(
            keyed_sentence.contains("先跑录入段") && !keyed_sentence.contains("拒算"),
            "不拒算的照旧报先跑录入段：{keyed_sentence}"
        );
    });
}

/// 只记计数的与交回镜像（只读第一个扇区）的两个观察者里挑 `observers_chosen` 那几个（下标 0 是计数的、1 是交回镜像的；另读的输入各不同，
/// 录入块的键因此各不同），挂在那条两个扇区的小流的点与末节点上，在 `store` 上照键复用录一趟，核对线程一条。
/// 线程取一条，是因为交回镜像的观察者在一个桶里头一个真跑的状态，它的持久集合写进看过的镜像那一行（`ImageToJudgeAfterTheObserver::persisted`）：
/// 点上只落后一个写与一个都没落的两个状态读的扇区字节相同、对上同一条见证，几条线程时哪一个先拿到这个桶的真跑锁随调度变，
/// 写下的那一行跟着变（一个观察者单独录两趟同样会不同）；一条线程时状态照块内次序看，写下的逐字节定下来。
fn record_the_counting_and_the_image_observers(
    store: &mut VerdictStore,
    observers_chosen: &[usize],
) -> ObserverRecordingTally {
    let count_the_state = |_ordinal: u64,
                           _image: &CrashImage<'_>,
                           _report: &RecoveryReport,
                           counts: &mut CrashPointReplayObserverCounts| {
        counts.add("states_counted", 1);
    };
    let hand_back_the_image = |_ordinal: u64,
                               image: &CrashImage<'_>,
                               _report: &RecoveryReport,
                               counts: &mut CrashPointReplayObserverCounts|
     -> ImageAfterTheObserver {
        counts.add("images_seen", 1);
        ImageAfterTheObserver {
            writes_on_the_crash_image: Vec::new(),
            disks_after_the_observer: pool_of_the_crash_image(
                image.base,
                image.writes,
                &image.persisted,
            ),
            ranges_read_on_the_crash_image: image
                .base
                .devices
                .keys()
                .map(|identity| ReadRangeOnTheCrashImage {
                    device: identity.0,
                    first_sector: TWO_SECTOR_FLOW_READ_SECTOR,
                    end_sector: TWO_SECTOR_FLOW_READ_SECTOR + 1,
                })
                .collect(),
            invariants_judged: InvariantsJudgedOnTheImageAfterTheObserver::Only(Vec::new()),
            counter_when_the_pool_checker_is_clean: "judged_clean".to_string(),
        }
    };
    let nodes = vec![
        NodeOfAnObserver {
            setting_file: OBSERVER_UNIT_TEST_SETTING_FILE.to_string(),
            entry_name: OBSERVER_UNIT_TEST_POINT_NAME.to_string(),
        },
        NodeOfAnObserver {
            setting_file: OBSERVER_UNIT_TEST_SETTING_FILE.to_string(),
            entry_name: END_OF_FLOW_NODE_NAME.to_string(),
        },
    ];
    let every_observer = [
        (
            ObserverOnTheNodes {
                observer_name: ObserverName::new(
                    "unit_test_counting_observer_sharing_the_recovery",
                )
                .expect("名字合规矩"),
                observer_code_file: OBSERVER_UNIT_TEST_SETTING_FILE.to_string(),
                extra_inputs_digest: [1; DIGEST_BYTES],
                nodes: nodes.clone(),
                observe: ObserverCode::CountsOnly(&count_the_state),
            },
            versions_of_the_unit_test_observer([1; DIGEST_BYTES]),
        ),
        (
            ObserverOnTheNodes {
                observer_name: ObserverName::new("unit_test_image_observer_sharing_the_recovery")
                    .expect("名字合规矩"),
                observer_code_file: OBSERVER_UNIT_TEST_SETTING_FILE.to_string(),
                extra_inputs_digest: [2; DIGEST_BYTES],
                nodes,
                observe: ObserverCode::HandsBackAnImage {
                    observe: &hand_back_the_image,
                    deduplication: ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
                },
            },
            versions_of_the_unit_test_observer([2; DIGEST_BYTES]),
        ),
    ];
    let observers: Vec<ObserverOnTheNodes<'_>> = observers_chosen
        .iter()
        .map(|position| every_observer[*position].0.clone())
        .collect();
    let versions: Vec<ObserverVersions> = observers_chosen
        .iter()
        .map(|position| every_observer[*position].1)
        .collect();
    a_flow_of_two_writes_to_two_sectors().with_the_flow(|flow| {
        let plan = plan_crash_points(flow);
        record_the_observers_of_the_flow_with_threads(
            &LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(store),
            flow,
            &plan,
            &observers,
            &versions,
            &|_| true,
            &|| false,
            1,
        )
        .expect("录入段读写得了库")
    })
}

/// 场景（一个状态恢复一次、交给这个点上的各个观察者）：那条两个扇区的小流（点里三个状态、末节点一个），挂只记计数的与交回镜像的两个观察者，
/// 一起录一趟；同样两个观察者各在一份空库里单独录一趟。三趟都是一条核对线程（为什么见 [`record_the_counting_and_the_image_observers`]）。
/// 预期：一起录那一趟恢复 4 次（每个状态一次，不是态数乘 2），两个单独录的各 4 次；一起录的那份库里的每一行（两个观察者的录入块、看过的镜像、
/// 见证与读集、基底、布局）与两份单独录的库并起来逐字节相同，账里录的块数、看的状态数也是两趟之和。
/// 判别力：`crates/mutations.tsv` 里「观察者录入段每个观察者各恢复一次同一个状态」那一行让这一条红在恢复次数上。
#[test]
fn two_observers_on_a_point_share_one_recovery_of_each_state_and_store_what_each_stores_alone() {
    let (_together_directory, mut together) =
        an_empty_library_for("observers-sharing-one-recovery-together");
    let (_counting_directory, mut counting_alone) =
        an_empty_library_for("observers-sharing-one-recovery-counting-alone");
    let (_image_directory, mut image_alone) =
        an_empty_library_for("observers-sharing-one-recovery-image-alone");
    let recorded_together = record_the_counting_and_the_image_observers(&mut together, &[0, 1]);
    let recorded_counting_alone =
        record_the_counting_and_the_image_observers(&mut counting_alone, &[0]);
    let recorded_image_alone = record_the_counting_and_the_image_observers(&mut image_alone, &[1]);
    assert_eq!(
        (
            recorded_together.states_recovered_here,
            recorded_counting_alone.states_recovered_here,
            recorded_image_alone.states_recovered_here,
        ),
        (4, 4, 4),
        "一起录时每个状态恢复一次（4 个状态 4 次，不是 8 次）；单独录的各恢复 4 次"
    );
    assert_eq!(
        (
            recorded_together.blocks_recorded_here,
            recorded_together.states_recorded_here,
            recorded_together.states_counted_here,
            recorded_together.images_observed_here,
        ),
        (
            recorded_counting_alone.blocks_recorded_here
                + recorded_image_alone.blocks_recorded_here,
            recorded_counting_alone.states_recorded_here
                + recorded_image_alone.states_recorded_here,
            recorded_counting_alone.states_counted_here + recorded_image_alone.states_counted_here,
            recorded_counting_alone.images_observed_here
                + recorded_image_alone.images_observed_here,
        ),
        "一起录的账是两趟单独录之和"
    );
    let rows_of = |store: &VerdictStore| -> BTreeMap<Box<[u8]>, Box<[u8]>> {
        store.every_stored_entry().into_iter().collect()
    };
    let mut rows_alone = rows_of(&counting_alone);
    rows_alone.extend(rows_of(&image_alone));
    assert_eq!(
        rows_of(&together),
        rows_alone,
        "一起录写进库的每一行与两个观察者单独录写的并起来逐字节相同"
    );
}

/// 等价关系（「按恢复结果去重」；剪枝只剪键相同与等价的）：两个不同崩溃态恢复出逐字节相同的镜像，观察者只看一次、
/// 录入块里只记一份观察结果；四个状态两张不同的镜像，观察者真跑两次，另两个状态取那一次的结果（录入段「现判 2、去重复用 2」），
/// 判段每张不同的镜像只判一次，每个状态照样拼进结论、计数照状态数。
#[test]
fn crash_states_recovering_to_byte_identical_images_are_observed_and_judged_once() {
    let (_directory, mut store) = an_empty_library_for("observer-identical-images");
    let run = run_the_unit_test_observer(
        &mut store,
        [3; DIGEST_BYTES],
        ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
        RunStage::RecordThenJudge,
    );
    let observing = run.observing.expect("判段判得了");
    assert_eq!(
        run.items_of_each_state.len(),
        4,
        "前提：段内三个状态加末节点"
    );
    assert_eq!(
        run.observer_calls, 2,
        "四个状态两张不同的镜像：观察者只看两次"
    );
    let recording = run.recording.expect("录入段录了");
    assert_eq!(
        (
            recording.images_observed_here,
            recording.states_reusing_an_observed_image
        ),
        (2, 2),
        "录入段现判 2、去重复用 2"
    );
    assert_eq!(
        run.observations_of_each_block,
        vec![2, 1],
        "崩溃点那一块三个状态两份观察结果（只落前一个与只落后一个是同一份），末节点那一块一份"
    );
    assert_eq!(
        (
            observing.images_judged_here,
            observing.image_judgements_reused
        ),
        (2, 1),
        "判段每张不同的镜像只判一次：末节点那一块的镜像取前一块判过的"
    );
    let images_seen: u64 = run
        .count_vectors
        .iter()
        .map(|count_vector| {
            count_vector
                .per_observer
                .get("images_seen")
                .copied()
                .unwrap_or(0)
        })
        .sum();
    assert_eq!(images_seen, 4, "去重之后计数照状态数拼：每个状态都记一次");
    assert_eq!(
        run.judged_on_of_each_block,
        vec![JudgingDevice::Cpu, JudgingDevice::Cpu],
        "配置关 GPU：结论记的在哪判的是 CPU（实际判盘的那一路）"
    );
}

/// 另读的输入不同的两个观察者（名字、代码相同）看同一张镜像不合并：后一个照样逐张镜像看一遍，不取前一个的结果；同一份另读的输入再跑，
/// 录入块都在库里，一张都不再看。
#[test]
fn observers_reading_different_extra_inputs_do_not_share_their_observed_images() {
    let (_directory, mut store) = an_empty_library_for("observer-extra-inputs");
    let first = run_the_unit_test_observer(
        &mut store,
        [3; DIGEST_BYTES],
        ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
        RunStage::RecordThenJudge,
    );
    let second = run_the_unit_test_observer(
        &mut store,
        [4; DIGEST_BYTES],
        ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
        RunStage::RecordThenJudge,
    );
    let again = run_the_unit_test_observer(
        &mut store,
        [4; DIGEST_BYTES],
        ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
        RunStage::RecordThenJudge,
    );
    assert_eq!(
        (
            first.observer_calls,
            second.observer_calls,
            again.observer_calls
        ),
        (2, 2, 0),
        "另读的输入不同不合并；相同的再跑全复用"
    );
}

/// 剪枝自证（`.claude/rules/test-script-discipline.md`「判定可复用：按输入存，按前缀剪」）：小输入上关掉去重跑一遍，
/// 逐状态的红项与每块的计数向量与开着时逐项相同；关掉时观察者每个状态都看一遍。
#[test]
fn turning_the_observer_deduplication_off_judges_every_state_as_with_it_on() {
    let (_on_directory, mut store_with_it_on) = an_empty_library_for("observer-dedup-on");
    let (_off_directory, mut store_with_it_off) = an_empty_library_for("observer-dedup-off");
    let with_it_on = run_the_unit_test_observer(
        &mut store_with_it_on,
        [3; DIGEST_BYTES],
        ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
        RunStage::RecordThenJudge,
    );
    let with_it_off = run_the_unit_test_observer(
        &mut store_with_it_off,
        [3; DIGEST_BYTES],
        ObserverDeduplication::OffForTheSelfProof,
        RunStage::RecordThenJudge,
    );
    assert_eq!(
        (with_it_on.observer_calls, with_it_off.observer_calls),
        (2, 4),
        "开着去重看两张镜像，关掉每个状态都看"
    );
    assert_eq!(
        with_it_on.items_of_each_state,
        with_it_off.items_of_each_state
    );
    assert_eq!(with_it_on.count_vectors, with_it_off.count_vectors);
}

/// 读集见证（「恢复报告分桶 + 读集见证」）：两次写落在两个扇区、四个状态的镜像字节两两不同，恢复报告相同（池没格式化，
/// 四个状态都择不到系统配置）落进一桶；只读第一个扇区的观察者看两张（那个扇区没写、写了），另两个状态按见证复用；
/// 读整个池的观察者四张都看、一个都不复用。
#[test]
fn crash_states_differing_only_outside_the_read_set_of_the_observer_are_observed_once() {
    let (_directory, mut store) = an_empty_library_for("observer-read-set-witness");
    let reading_one_sector = run_the_unit_test_observer_on(
        &mut store,
        a_flow_of_two_writes_to_two_sectors(),
        DeclaredReads::OnlySector(TWO_SECTOR_FLOW_READ_SECTOR),
        [3; DIGEST_BYTES],
        ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
        RunStage::RecordThenJudge,
    );
    assert_eq!(
        reading_one_sector.items_of_each_state.len(),
        4,
        "前提：段内三个状态加末节点"
    );
    let recording = reading_one_sector.recording.expect("录入段录了");
    assert_eq!(
            (
                reading_one_sector.observer_calls,
                recording.images_observed_here,
                recording.states_reusing_an_observed_image,
                recording.buckets,
                recording.witness_misses
            ),
            (2, 2, 2, 1, 1),
            "一桶；第一个扇区没写与写了各一个见证；两个只在读集外不同的状态复用；第二个见证是见证一个都对不上才真跑的那一次"
        );
    let (_whole_directory, mut whole_store) = an_empty_library_for("observer-read-set-whole");
    let reading_the_whole_pool = run_the_unit_test_observer_on(
        &mut whole_store,
        a_flow_of_two_writes_to_two_sectors(),
        DeclaredReads::WholePool,
        [3; DIGEST_BYTES],
        ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
        RunStage::RecordThenJudge,
    );
    let whole_recording = reading_the_whole_pool.recording.expect("录入段录了");
    assert_eq!(
        (
            reading_the_whole_pool.observer_calls,
            whole_recording.images_observed_here,
            whole_recording.states_reusing_an_observed_image
        ),
        (4, 4, 0),
        "读整个池：四个状态的镜像字节两两不同，一个都合不掉"
    );
}

/// 场景（恢复报告的摘要相邻状态共用）：一份恢复报告与只差转查映射次数的另一份，照「甲、甲、乙、甲」的次序共用一份留着的报告取摘要。
/// 预期：每一次取到的与对那一份现算的（[`recovery_report_digest`]）逐字节相同，两份的摘要不同。
/// 判别力：`crates/mutations.tsv` 里「观察者录入段恢复报告不同也取留着的摘要」那一行让这一条红在乙那一次上。
#[test]
fn a_recovery_report_digest_kept_from_the_last_state_is_reused_only_for_an_equal_report() {
    let device = singlefs_core::address::DeviceIdentity(1);
    let pool = MemoryPool::with_devices(&[device], 1 << 20);
    let report_of_an_unformatted_pool = recover(&pool, JournalPolicy::Consult);
    let mut report_with_one_more_fallback = report_of_an_unformatted_pool.clone();
    report_with_one_more_fallback.mapping_fallbacks += 1;
    assert_ne!(
        recovery_report_digest(&report_of_an_unformatted_pool),
        recovery_report_digest(&report_with_one_more_fallback),
        "前提：只差转查映射次数的两份报告摘要不同"
    );
    let kept = std::cell::RefCell::new(None);
    for report in [
        &report_of_an_unformatted_pool,
        &report_of_an_unformatted_pool,
        &report_with_one_more_fallback,
        &report_of_an_unformatted_pool,
    ] {
        assert_eq!(
            recovery_report_digest_reusing_the_one_kept(report, &kept),
            recovery_report_digest(report),
            "共用留着的那一份取到的摘要与现算的逐字节相同"
        );
    }
}

/// 观察者录入段一块录得比两个进度间隔还久（假时钟每读一次走过一个间隔多一秒，那一段等到时钟读到第 5 次才交来）：等着收的那条线程
/// 至少打两行按挂钟的，行里是观察者、点、块起点、收齐几段（共几段）、这一块已看过几个状态、桶数、见证条数、读集组数、开跑多久；那一段照样收进来。
#[test]
fn an_observer_block_recorded_for_longer_than_two_progress_intervals_reports_two_wall_clock_lines()
{
    use crate::crash::PROGRESS_LINE_INTERVAL_IN_SECONDS;
    use std::sync::atomic::{AtomicU64, Ordering};
    let one_interval_and_a_second =
        std::time::Duration::from_secs(PROGRESS_LINE_INTERVAL_IN_SECONDS + 1);
    let (piece_sender, piece_receiver) =
        std::sync::mpsc::channel::<Result<RecordedObserverPiece, String>>();
    let (release_sender, release_receiver) = std::sync::mpsc::channel::<()>();
    let clock_reads = AtomicU64::new(0);
    // 第 0 次读是开跑那一刻；第 5 次读（按挂钟那一行已经到期过两回）时才放那一段进来
    let clock = || {
        let reads = clock_reads.fetch_add(1, Ordering::SeqCst);
        if reads == 5 {
            // 放行的那一头只收一次，收过之后发不出去也无妨
            let _ = release_sender.send(());
        }
        one_interval_and_a_second * u32::try_from(reads).expect("一条用例里读不了几十亿次")
    };
    let mut wall_clock_lines = WallClockProgressLines::every_progress_line_interval(clock());
    let mut printed: Vec<String> = Vec::new();
    let arrived = std::thread::scope(|scope| {
        scope.spawn(move || {
            release_receiver.recv().expect("时钟读到第 5 次就放行");
            piece_sender
                .send(Ok(RecordedObserverPiece {
                    local: 0..3,
                    observation_of_each_state: Vec::new(),
                    state_counts: StateCountsOfABlock::default(),
                    tally: ObserverRecordingTally::default(),
                }))
                .expect("收的那一头还在等");
        });
        receive_the_observer_pieces_of_one_block(
            &piece_receiver,
            &mut wall_clock_lines,
            &clock,
            &ObserverBlockInProgress {
                observer_name: "remount_after_the_sweep_phases",
                point_name: "sweep.unit_writes",
                block_start: 0,
                piece_count: 1,
            },
            &|| ObserverRecordingProgress {
                states_looked_at_in_the_block: 2,
                buckets: 1,
                witnesses: 9,
                read_set_groups: 4,
            },
            &mut |line| printed.push(line),
        )
    });
    assert_eq!(arrived.len(), 1, "那一段照样收进来");
    assert!(
        printed.len() >= 2,
        "录了两个间隔多，至少两行按挂钟的：{printed:?}"
    );
    assert_eq!(
            printed.first().map(String::as_str),
            Some(
                "CRASH_AMPLIFICATION_OBSERVER_RECORDING by=wall_clock observer=remount_after_the_sweep_phases point=sweep.unit_writes block_start=0 pieces_finished=0/1 states_looked_at_in_the_block=2 buckets=1 witnesses=9 read_set_groups=4 elapsed_seconds=602.0"
            ),
            "过了一个间隔还没收齐，按挂钟打一行：{printed:?}"
        );
}
