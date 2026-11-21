use super::*;

/// 核对线程数写成 0：panic，消息带变量名与原值，不悄悄退回默认（与仓里别的线程数变量同一口径）。
#[test]
#[should_panic(expected = "SINGLEFS_CRASH_AMPLIFICATION_THREADS=\"0\" 认不出")]
fn zero_checking_threads_panics_naming_the_variable_and_the_value() {
    let _ = checking_threads_from_the_variable_value(Some("0"), None);
}

/// 核对线程数写坏（不是数）：panic，消息带变量名与原值，不悄悄退回默认。
#[test]
#[should_panic(expected = "SINGLEFS_CRASH_AMPLIFICATION_THREADS=\"eight\" 认不出")]
fn an_unreadable_checking_threads_value_panics_naming_the_variable_and_the_value() {
    let _ = checking_threads_from_the_variable_value(Some("eight"), None);
}

/// 核对线程数设了照它（两头的空白不算），没设读 `SINGLEFS_THREAD_CAP`，都没设取 `.claude/heavy-thread-default` 的登记值（不取本机核数）。
#[test]
fn checking_threads_are_the_value_written_or_the_heavy_thread_default_when_unset() {
    assert_eq!(
        checking_threads_from_the_variable_value(Some(" 3\n"), Some("5")),
        3
    );
    assert_eq!(checking_threads_from_the_variable_value(None, Some("5")), 5);
    assert_eq!(
        checking_threads_from_the_variable_value(None, None),
        crate::heavy_threads::registered_heavy_threads().get()
    );
}

/// C611：一块判得比进度间隔久（假时钟每读一次走过一个间隔多一秒，那一段等到时钟读过两次才交来），等着收的那条线程打一行按挂钟的进度，
/// 行里是这一块的点名、块起点、已收到几段（共几段）、这一趟判完几块与几个状态、开跑多久；那一段照样收进来。
#[test]
fn a_block_judged_for_longer_than_the_progress_interval_reports_a_wall_clock_line_naming_its_point()
{
    use crate::crash::PROGRESS_LINE_INTERVAL_IN_SECONDS;
    use std::sync::atomic::{AtomicU64, Ordering};
    let one_interval_and_a_second =
        std::time::Duration::from_secs(PROGRESS_LINE_INTERVAL_IN_SECONDS + 1);
    let (piece_sender, piece_receiver) = std::sync::mpsc::channel::<CpuJudgedBlock>();
    let (release_sender, release_receiver) = std::sync::mpsc::channel::<()>();
    let clock_reads = AtomicU64::new(0);
    // 第 0 次读是开跑那一刻；第 2 次读（已经过了一个间隔、按挂钟那一行到期）时才放那一段进来
    let clock = || {
        let reads = clock_reads.fetch_add(1, Ordering::SeqCst);
        if reads == 2 {
            // 放行的那一头只收一次，收过之后发不出去也无妨
            let _ = release_sender.send(());
        }
        one_interval_and_a_second * u32::try_from(reads).expect("一条用例里读不了几十亿次")
    };
    let mut wall_clock_lines = WallClockProgressLines::every_progress_line_interval(clock());
    let mut printed: Vec<String> = Vec::new();
    let arrived = std::thread::scope(|scope| {
        scope.spawn(move || {
            release_receiver.recv().expect("时钟读到第 2 次就放行");
            piece_sender
                .send(CpuJudgedBlock {
                    local: 7..9,
                    items_of_each_state: vec![Vec::new(), Vec::new()],
                    recovery_words_of_each_state: Vec::new(),
                    state_counts: StateCountsOfABlock::default(),
                    red_texts: Vec::new(),
                    self_verified_system_configuration_slots_of_the_states:
                        SelfVerifiedSystemConfigurationSlotsOfTheStates::default(),
                })
                .expect("收的那一头还在等");
        });
        receive_the_pieces_of_one_block(
            &piece_receiver,
            &mut wall_clock_lines,
            &clock,
            &BlockInProgress {
                point_name: "publish_first_file.journal_record.1",
                block_start: 7,
                piece_count: 1,
                blocks_finished: 3,
                pending_blocks: 5,
                states_finished: 40,
            },
            &mut |line| printed.push(line),
        )
    });
    assert_eq!(arrived.len(), 1, "那一段照样收进来");
    assert_eq!(
            printed.first().map(String::as_str),
            Some(
                "CRASH_AMPLIFICATION_BLOCK by=wall_clock point=publish_first_file.journal_record.1 block_start=7 pieces_finished=0/1 blocks_finished=3/5 states_finished=40 elapsed_seconds=602.0"
            ),
            "过了一个间隔还没收到那一段，按挂钟打一行：{printed:?}"
        );
}

/// （扫源码的用例被领块之前那一问写成 `if false && stop_is_requested` 骗过，
/// 攻方 crash-3）：四块要判，叫停在第三次被问时交真：领每一块之前各问一次、问到真就不再领，叫停之前领的两块判完写了，之后的不领，
/// 叫停一共被问三次。不要卡、不碰库：判一段与写一块都用假的。
#[test]
fn the_stop_request_is_asked_once_before_each_block_and_a_stop_leaves_the_later_blocks_untaken() {
    const ASK_THAT_FINDS_THE_STOP: usize = 3;
    let pending: Vec<(usize, Range<u64>)> = vec![(0, 0..2), (0, 2..4), (0, 4..6), (0, 6..8)];
    let points = vec![NamesOfACrashPoint {
        name: "一个点".to_string(),
        path_number: "0".to_string(),
    }];
    let asks = std::sync::atomic::AtomicUsize::new(0);
    let mut stored_block_starts = Vec::new();
    let tally = check_pending_blocks_in_pieces(
        &pending,
        &points,
        2,
        &|| asks.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1 >= ASK_THAT_FINDS_THE_STOP,
        &|_point_index, local| CpuJudgedBlock {
            items_of_each_state: vec![
                Vec::new();
                usize::try_from(local.end - local.start).expect("一段装得进")
            ],
            local,
            recovery_words_of_each_state: Vec::new(),
            state_counts: StateCountsOfABlock::default(),
            red_texts: Vec::new(),
            self_verified_system_configuration_slots_of_the_states:
                SelfVerifiedSystemConfigurationSlotsOfTheStates::default(),
        },
        &mut |_point_index, local, _checked| {
            stored_block_starts.push(local.start);
            Ok(())
        },
    )
    .expect("假的写库不失败");
    assert_eq!(
        asks.load(std::sync::atomic::Ordering::SeqCst),
        ASK_THAT_FINDS_THE_STOP,
        "领第一、二块之前各问一次（不停），领第三块之前问到停、不再问"
    );
    assert_eq!(
        stored_block_starts,
        vec![0, 2],
        "叫停之前领的两块判完写了，之后的不领"
    );
    assert_eq!(tally.blocks_checked, 2);
}

/// 场景：这个进程里开着的那一份库（一次性库）上，CPU 判段判那条两个扇区的小流，判一段的判器头一次判之前先睡 200 毫秒；它睡着的时候
/// 另一条线程取同一条连接的锁。
/// 预期：等锁不超过 50 毫秒（判段只在问库里哪几块要判、每判完一块写进库那几下取锁，判的时候手里不握锁）；判段照常判完每一块。
/// 判别力（`crates/mutations.tsv`）：「判定库锁按次取放 CPU 判段改回整段握锁」这一行让这一条红。
#[test]
fn another_thread_takes_the_library_lock_while_the_cpu_judge_stage_is_judging() {
    // 一次性库留着：要一把这个进程里开着的那一份的真锁
    let disposable =
        crate::crash_library::DisposableLibrary::new("lock-free-while-the-cpu-judge-judges")
            .unwrap_or_else(|refusal| panic!("{refusal}"));
    let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
    let library = LibraryOpenInThisProcess::open_or_share().expect("开得了库");
    let read_back = a_flow_of_two_writes_to_two_sectors().with_the_flow(|recorded_flow| {
        record_the_flow_and_read_it_back(&mut library.lock(), recorded_flow)
            .expect("流写得进库、读得回")
    });
    let flow = read_back.flow();
    let plan = plan_crash_points(&flow);
    let block_count: usize = plan.points.iter().map(|point| point.blocks().len()).sum();
    let versions = vec![
        ConclusionVersion(sha256_digest(
            b"version of the lock-free cpu judge stage unit test\n"
        ));
        plan.points.len()
    ];
    let conclusions = ConclusionsStoredUnder::TheConclusionKind {
        kind: ConclusionKind::ImageJudgement,
        version_of_each_point: &versions,
    };
    let the_judge_has_started_sleeping = std::sync::atomic::AtomicBool::new(false);
    let (waited, checking) = wait_for_the_library_lock_while_the_judge_or_the_observer_sleeps(
        &library,
        &the_judge_has_started_sleeping,
        || {
            check_blocks_where_until_judging_each_piece_with(
                &LibraryTakenAtEachReadOrWrite::SharedInThisProcess(&library),
                &flow,
                &plan,
                &|_| true,
                &conclusions,
                &|| false,
                1,
                &|crash_states_of_the_flow, point_index, local| {
                    if !the_judge_has_started_sleeping
                        .swap(true, std::sync::atomic::Ordering::SeqCst)
                    {
                        std::thread::sleep(TIME_THE_SLOW_JUDGE_OR_OBSERVER_SLEEPS);
                    }
                    judge_a_piece_of_a_block_on_the_cpu(
                        crash_states_of_the_flow,
                        flow.judged_root_index,
                        flow.version_table(),
                        plan.points[point_index].ordinals.clone(),
                        local,
                    )
                },
            )
        },
    );
    let checking = checking.expect("判段判得完");
    assert!(
        waited <= LONGEST_WAIT_FOR_THE_LIBRARY_LOCK_MEANWHILE,
        "判器睡着的时候另一条线程等了 {waited:?} 才取到同一条连接的锁：判段整段握着锁"
    );
    assert_eq!(
        usize::try_from(checking.blocks_checked).expect("装得进 usize"),
        block_count,
        "判段照常判完每一块"
    );
}

/// CPU 判段的枚举表同样一趟只建一份、各核对线程借同一份（`check_pending_blocks_sharing_one_enumeration_table_of_the_flow`，
/// 与观察者录入段那一条 `the_observer_recording_threads_of_one_block_share_one_enumeration_table_of_the_flow` 同形）：一块切成几段、
/// 各一条线程判，每条线程拿到的状态镜像都叠在同一份枚举写表上。判一段的那一份换成
/// 记账的假判法（不跑恢复与池级 checker，判器本身不在这条的射程），每条线程看它头一个状态时等到至少两条线程都到了才往下走：
/// 逐段各建一份时那几份在同一刻都活着，地址必不相同，所以「各建一份」一定判红。核对线程数在用例里定成 2、不读环境，交给被测那一段；
/// 单测进程里拿到的是它与里层份额（`crate::crash_library::InnerThreadsTaken`）里小的那个，里层份额至少
/// `crate::crash_library::FEWEST_INNER_THREADS_OF_THE_SHARE`（2）条，拿到的就是 2，一块三个状态切成两段。
#[test]
fn the_cpu_judging_threads_of_one_block_share_one_enumeration_table_of_the_flow() {
    // 两条核对线程：CPU 判段把一块三个状态切成两段、各一条线程判
    let threads = 2;
    let seen = std::sync::Mutex::new(ThreadsAndEnumeratedWritesSeenByTheObserver::default());
    let a_second_thread_arrived = std::sync::Condvar::new();
    let recorded = a_flow_of_two_writes_to_two_sectors();
    let tally = recorded.with_the_flow(|flow| {
        let plan = plan_crash_points(flow);
        let point_index = plan
            .points
            .iter()
            .position(|point| point.name == OBSERVER_UNIT_TEST_POINT_NAME)
            .expect("小流设了那一个点");
        let ordinals_of_the_point = plan.points[point_index].ordinals.clone();
        let judge_a_piece =
            |crash_states_of_the_flow: &crate::crash::CrashStatesOfTheReadBackFlow<'_>,
             judged_point_index: usize,
             local: Range<u64>| {
                let ordinals_start = plan.points[judged_point_index].ordinals.start;
                let _stream_state_count = crash_states_of_the_flow
                    .visit_the_states_in_the_ordinals(
                        ordinals_start + local.start..ordinals_start + local.end,
                        &mut |_ordinal, _publish, image| {
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
                            let deadline =
                                std::time::Instant::now() + std::time::Duration::from_secs(60);
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
                        },
                    );
                CpuJudgedBlock {
                    items_of_each_state: vec![
                        Vec::new();
                        usize::try_from(local.end - local.start)
                            .expect("一块装得进 usize")
                    ],
                    local,
                    recovery_words_of_each_state: Vec::new(),
                    state_counts: StateCountsOfABlock::default(),
                    red_texts: Vec::new(),
                    self_verified_system_configuration_slots_of_the_states:
                        SelfVerifiedSystemConfigurationSlotsOfTheStates::default(),
                }
            };
        check_pending_blocks_sharing_one_enumeration_table_of_the_flow(
            flow,
            &plan,
            &[(
                point_index,
                0..ordinals_of_the_point.end - ordinals_of_the_point.start,
            )],
            threads,
            &|| false,
            &judge_a_piece,
            &mut |_point_index, _local, _checked| Ok(()),
        )
        .expect("假的存块不报错")
    });
    assert_eq!(tally.blocks_checked, 1, "判了那一块");
    let seen = seen
        .into_inner()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    assert_eq!(
        seen.enumerated_writes_address_of_each_state.len(),
        3,
        "段里两个写的任意真子集三个状态，CPU 判段每个都判了一次"
    );
    assert!(
        seen.threads.len() >= 2,
        "一块三个状态切成了至少两段、各一条线程判（核对线程数 {threads}），读到 {} 条",
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
            "各核对线程判的镜像叠在同一份枚举写表上（一趟只建一份、各线程借用），读到 {} 份：{distinct_enumerated_writes:?}",
            distinct_enumerated_writes.len()
        );
}
