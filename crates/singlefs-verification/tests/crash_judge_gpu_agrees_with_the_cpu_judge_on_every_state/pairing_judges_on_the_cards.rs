//! 对拍判器在卡上：判表建不出拒判、叫停之后下一趟只判剩下的块、暂存宽度与派发耗时、卡与主机的上限、吞吐。

use super::*;

/// 卡领块的时候被叫停：不再领新的块，已经领的判完写进库（块序连着）；下一趟只判剩下的，对账为空。
/// 配置开了 GPU、判器却建不出这条流的表（这里把流里第一次单元写往后挪一个扇区：单元区的落点按 16 KiB 槽起，挪过的写起点不在
/// 任何落点上，判器的表不接，`JudgeTablesRefusal::WriteNotAlignedToALocation`；清零写判表已收（批 13），拿它造不出建不出）：报错，
/// 不退回 CPU 判器，库里一块判定都不写（配了 GPU 就一定要 GPU 判，做不到就明确报错）。建表在上卡之前，不用卡。
/// 判别力：`crates/mutations.tsv` 里「B16 GPU 判表把起点不在落点上的单元写当成不占落点」与「崩溃放量 建不出表时悄悄交回空结果」两行各让这一条红。
#[test]
fn a_flow_whose_judge_tables_cannot_be_built_is_refused_and_not_judged_on_the_cpu() {
    let recorded = record_fixed_script_through_rollback("crash-judge-gpu-refused");
    let mut writes = recorded.writes.clone();
    let moved = writes
        .iter()
        .position(|write| write.kind == StepKind::UnitWrite)
        .expect("流里至少一次单元写");
    writes[moved].offset.0 += 512;
    let expansion = |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= 8 {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    let flow = CrashFlow {
        base: &recorded.base,
        writes: &writes,
        segments: &recorded.segments,
        judged_root_index: recorded.judged_root_index,
        versions: &recorded.versions,
        versions_without_a_file: &[],
        expansion: &expansion,
        crash_points: vec![CrashPointSpan {
            name: "every_write".to_string(),
            code_file_in_repository: PATH_OF_THIS_TEST_FILE.to_string(),
            writes: 0..writes.len(),
            ignored: false,
        }],
    };
    let plan = plan_crash_points(&flow);
    let version = JudgingUnitsFingerprint([9u8; 32]);
    // 一份空库：临时 prune-server 里的一段一次性名字空间（临时 prune-server 由它自己在这个进程里起，起不来当场失败）
    // 一次性库留着：假的判法单元指纹下钉叫停之后下一趟只判剩下的，要从空库数
    let directory =
        singlefs_verification::crash_library::DisposableLibrary::new("crash-judge-gpu-refused")
            .unwrap_or_else(|refusal| panic!("{refusal}"));
    let mut store = directory
        .open_store(
            singlefs_verification::verdict_store::PruneServerFailureHandling::ReturnTheError,
        )
        .expect("开得了一次性库");
    record_crash_points(&mut store, &plan, &version, &mut CoverageReport::default())
        .expect("录得进");
    let before = unjudged_blocks(&store, &version).expect("对得了账").len();
    assert!(before > 0, "这个点至少一块要判");
    // 判法只吃库里读回的那一份：流先写进库再读回。
    let read_back_of_line_3177 =
        record_the_flow_and_read_it_back(&mut store, &flow).expect("流写得进库、读得回");
    let refused = judge_my_blocks_on_gpu_cards(
        &mut store,
        &read_back_of_line_3177.flow(),
        &plan,
        &|_block| true,
        &version,
        &[],
        &|| false,
    );
    let error = refused.expect_err("建不出表要报错，不退回 CPU 判器");
    assert!(
        error.contains("建不出这条流的表"),
        "报错说清是表建不出：{error}"
    );
    assert!(
        error.contains("WriteNotAlignedToALocation"),
        "建不出表是因为挪过的单元写起点不在落点上：{error}"
    );
    assert_eq!(
        unjudged_blocks(&store, &version).expect("对得了账").len(),
        before,
        "一块判定都没写进库"
    );
    drop(store);
    drop(directory);
}

/// 流取固定脚本到回退 D、展开 16，每段一个崩溃点（几十块）；只用一张卡，叫停之前恰好领五块。
#[test]
fn the_card_stops_taking_blocks_when_asked_and_the_next_run_judges_only_the_rest() {
    const EXPAND_UP_TO: usize = 16;
    const BLOCKS_BEFORE_THE_STOP: usize = 5;
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_card_stops_taking_blocks_when_asked_and_the_next_run_judges_only_the_rest",
    ) else {
        return;
    };
    let recorded = record_fixed_script_through_rollback("crash-judge-gpu-stop");
    let expansion = move |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= EXPAND_UP_TO {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    let flow = CrashFlow {
        base: &recorded.base,
        writes: &recorded.writes,
        segments: &recorded.segments,
        judged_root_index: recorded.judged_root_index,
        versions: &recorded.versions,
        versions_without_a_file: &[],
        expansion: &expansion,
        crash_points: recorded
            .segments
            .iter()
            .enumerate()
            .map(|(index, segment)| CrashPointSpan {
                name: format!("segment_{index}"),
                code_file_in_repository: PATH_OF_THIS_TEST_FILE.to_string(),
                writes: *segment.first().expect("段里至少一次写")
                    ..*segment.last().expect("段里至少一次写") + 1,
                ignored: false,
            })
            .collect(),
    };
    // 借一个定量：一条草稿缓冲那么宽的判器加表（判别力在叫停与续判，不在宽度）；放量路径照借来的卡的额度自己定一次派活带几个。
    // 表先在这里建一次知道多大（放量路径里面照同一条流再建一次，这里没有点被忽略，两次建的是同一份）
    let tables_of_the_flow = PackedJudgeTables::of(
        &build_judge_tables(&JudgeFlow {
            base: flow.base,
            writes: flow.writes,
            segments: flow.segments,
            judged_root_index: flow.judged_root_index,
            versions: flow.versions,
            expansion: &expansion,
        })
        .expect("建得出表"),
    );
    let Some(lease) = cards.lease_card_or_report_not_run(
        "the_card_stops_taking_blocks_when_asked_and_the_next_run_judges_only_the_rest",
        mebibytes_a_judge_needs(
            &tables_of_the_flow,
            states_of_one_scratch_lane_on_every_card_of(cards),
        ),
    ) else {
        return;
    };
    let card = std::sync::Arc::new(lease.card().clone());
    let plan = plan_crash_points(&flow);
    let version = JudgingUnitsFingerprint([7u8; 32]);
    // 一份空库：临时 prune-server 里的一段一次性名字空间（临时 prune-server 由它自己在这个进程里起，起不来当场失败）
    let directory =
        // 一次性库留着：造拒算的版本（假的判法单元指纹）钉照判不入库，要数库里的行
        singlefs_verification::crash_library::DisposableLibrary::new("crash-judge-gpu-stop")
            .unwrap_or_else(|refusal| panic!("{refusal}"));
    let mut store = directory
        .open_store(
            singlefs_verification::verdict_store::PruneServerFailureHandling::ReturnTheError,
        )
        .expect("开得了一次性库");
    record_crash_points(&mut store, &plan, &version, &mut CoverageReport::default())
        .expect("录得进");
    let total = unjudged_blocks(&store, &version).expect("对得了账").len();
    assert!(
        total > BLOCKS_BEFORE_THE_STOP + 1,
        "这条流只有 {total} 块，不够在中间叫停"
    );
    let asked = std::sync::atomic::AtomicUsize::new(0);
    let stop_after_five_blocks =
        || asked.fetch_add(1, std::sync::atomic::Ordering::SeqCst) >= BLOCKS_BEFORE_THE_STOP;
    // 判法只吃库里读回的那一份：流先写进库再读回。
    let read_back_of_line_3283 =
        record_the_flow_and_read_it_back(&mut store, &flow).expect("流写得进库、读得回");
    let Some((stopped_tally, stopped)) = judged_on_the_cards_or_report_not_run_when_out_of_memory(
        "the_card_stops_taking_blocks_when_asked_and_the_next_run_judges_only_the_rest",
        judge_my_blocks_on_gpu_cards(
            &mut store,
            &read_back_of_line_3283.flow(),
            &plan,
            &|_block| true,
            &version,
            std::slice::from_ref(&card),
            &stop_after_five_blocks,
        ),
    ) else {
        drop(store);
        drop(directory);
        return;
    };
    println!(
        "CRASH_JUDGE_STOP total_blocks={total} blocks_before_the_stop={} states_before_the_stop={}",
        stopped.blocks_judged, stopped.states_judged
    );
    assert_eq!(
        stopped.blocks_judged,
        u64::try_from(BLOCKS_BEFORE_THE_STOP).expect("装得进 u64"),
        "叫停之前恰好领了五块"
    );
    assert_eq!(stopped_tally.blocks_checked, stopped.blocks_judged);
    assert_eq!(
        unjudged_blocks(&store, &version).expect("对得了账").len(),
        total - BLOCKS_BEFORE_THE_STOP,
        "领过的五块都在库里"
    );
    // 判法只吃库里读回的那一份：流先写进库再读回。
    let read_back_of_line_3314 =
        record_the_flow_and_read_it_back(&mut store, &flow).expect("流写得进库、读得回");
    let Some((resumed_tally, resumed)) = judged_on_the_cards_or_report_not_run_when_out_of_memory(
        "the_card_stops_taking_blocks_when_asked_and_the_next_run_judges_only_the_rest",
        judge_my_blocks_on_gpu_cards(
            &mut store,
            &read_back_of_line_3314.flow(),
            &plan,
            &|_block| true,
            &version,
            std::slice::from_ref(&card),
            &|| false,
        ),
    ) else {
        drop(store);
        drop(directory);
        return;
    };
    assert_eq!(
        resumed.blocks_judged,
        u64::try_from(total - BLOCKS_BEFORE_THE_STOP).expect("装得进 u64"),
        "接着判的那一趟只判剩下的块"
    );
    assert_eq!(
        stopped.states_judged + resumed.states_judged,
        plan.points.last().expect("有点").ordinals.end,
        "两趟合起来每个状态恰好判一次"
    );
    assert_eq!(stopped_tally.red_states + resumed_tally.red_states, 0);
    assert!(unjudged_blocks(&store, &version)
        .expect("对得了账")
        .is_empty());
    drop(store);
    drop(directory);
}

/// 这个进程此刻在卡上占的显存（MiB，`nvidia-smi` 按进程报的；几张卡加起来）。读不到报 0。
fn memory_used_on_the_card_by_this_process() -> u64 {
    let Ok(output) = std::process::Command::new("nvidia-smi")
        .args([
            "--query-compute-apps=pid,used_memory",
            "--format=csv,noheader,nounits",
        ])
        .output()
    else {
        return 0;
    };
    let mine = std::process::id().to_string();
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let (process, used) = line.split_once(',')?;
            (process.trim() == mine)
                .then(|| used.trim().parse::<u64>().ok())
                .flatten()
        })
        .sum()
}

/// 规划显卡用：每张能用的卡按整张卡的额度（空闲显存）开草稿区，从一条草稿缓冲开到这张卡装得下的条数，每一档照那一档的宽度派满三次，
/// 打每次派活卡算了多久、每个状态几微秒、这个进程在卡上占多少显存。看草稿区开到多大时每个状态的用时开始猛涨（缓冲被驱动放出显存）。
#[test]
#[ignore = "量显存上限用：要清场之后空着的卡，每张卡开十几档草稿区"]
fn dispatch_time_at_each_scratch_size_with_the_whole_card_given() {
    let Some(usable_cards) = usable_cards_or_report_not_run(
        "dispatch_time_at_each_scratch_size_with_the_whole_card_given",
    ) else {
        return;
    };
    let every_card: Vec<(u32, u64)> = usable_cards
        .iter()
        .map(|card| (card.index, card.free_memory_mebibytes))
        .collect();
    // 量的是这个进程在卡上占多少显存：取卡时起的上下文先放掉，下面按额度重起的那一份才是量的对象
    drop(usable_cards);
    let flow = record_fixed_script_through_rollback("crash-judge-gpu-scratch-sizes");
    let limit = expand_up_to();
    let expansion = move |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= limit {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    let judge_flow = JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &expansion,
    };
    let tables = build_judge_tables(&judge_flow).expect("建得出表");
    // 判表先入库、再读库，GPU 只拿读回的那一份算
    let Some(packed) = packed_tables_through_the_library(
        "dispatch_time_at_each_scratch_size_with_the_whole_card_given",
        &tables,
    ) else {
        return;
    };
    let state_count = tables.state_count;
    for card in singlefs_verification::gpu_unit_checks::gpu_cards_by_priority(&every_card) {
        let limit_of_the_card = card.device().limits().max_storage_buffer_binding_size;
        let lane =
            singlefs_verification::crash_judge_dispatch::states_per_scratch_lane(limit_of_the_card);
        let capacity = singlefs_verification::crash_judge_dispatch::states_in_flight_within(
            limit_of_the_card,
            card.memory_budget_mebibytes(),
            &packed,
        );
        println!(
            "CRASH_JUDGE_SCRATCH_CARD card={:?} index={} free_mebibytes={} capacity_states={capacity} states_per_lane={lane} states={state_count}",
            card.name, card.index, card.free_memory_mebibytes
        );
        let mut lanes = 1usize;
        loop {
            let width = (lanes * lane).min(capacity);
            let measured_this_width = on_the_card_or_report_not_run_when_out_of_memory(
                "dispatch_time_at_each_scratch_size_with_the_whole_card_given",
                &card,
                "按这一档草稿区建判器与派活",
                || {
                    let judge = GpuCrashJudge::on_card_with_packed_tables(&card, &packed, width);
                    let used = memory_used_on_the_card_by_this_process();
                    // 从流的中间取，避开开头那些很快判完的小状态
                    let first =
                        (state_count / 2).min(state_count - u64::try_from(width).expect("装得进"));
                    for round in 0..3 {
                        let before = judge.time_spent().seconds_waiting_for_the_card;
                        let verdicts = judge.judge_states_in_one_dispatch(
                            first..first + u64::try_from(width).expect("装得进"),
                        );
                        let seconds = judge.time_spent().seconds_waiting_for_the_card - before;
                        println!(
                            "CRASH_JUDGE_SCRATCH card_index={} lanes={lanes} states={width} round={round} seconds={seconds:.3} microseconds_per_state={:.2} scratch_mebibytes={} process_memory_on_the_card_mebibytes={used}",
                            card.index,
                            seconds * 1e6 / verdicts.len() as f64,
                            judge.scratch_mebibytes()
                        );
                        if seconds > 5.0 {
                            break;
                        }
                    }
                },
            );
            if measured_this_width.is_none() {
                return;
            }
            if width == capacity {
                break;
            }
            lanes += 1;
        }
    }
}

/// 对拍判器的宽度：放得下时开到一张卡额度的一半（两条缓冲以上，判器才量宽度）；放不下一条缓冲的也开一条；不超过缓冲条数上限；不超过这一段的状态数。不用卡。
/// 判别力（`crates/mutations.tsv`）：额度不平分给同卡的判器，头一种红；不保底一条缓冲，第二种红；不按这一段的状态数压，第四种红。
#[test]
fn a_pairing_judge_opens_half_of_the_smallest_card_and_at_least_one_scratch_lane() {
    const STATES_PER_LANE: usize = 267;
    const TABLES_MEBIBYTES: u64 = 100;
    let mebibytes_for_states =
        |states: usize| TABLES_MEBIBYTES + u64::try_from(states).expect("装得进 u64") * 8;
    let everything = u64::MAX;
    // 空着时借得出 13824 MiB 的卡：一个判器 6912 MiB，(6912 - 100) / 8 = 851 个状态
    let states_on_the_card_with_room = states_per_dispatch_of_a_pairing_judge(
        13_824,
        STATES_PER_LANE,
        everything,
        &mebibytes_for_states,
    );
    assert_eq!(
        states_on_the_card_with_room, 851,
        "一张卡的额度平分给同卡的判器，一个判器放得下几个状态就开几个"
    );
    assert!(
        states_on_the_card_with_room >= 2 * STATES_PER_LANE,
        "两条缓冲以上：判器量宽度的条件进得去"
    );
    // 一半的额度放不下一条缓冲（267 个状态要 2236 MiB）：开一条
    assert_eq!(
        states_per_dispatch_of_a_pairing_judge(
            3000,
            STATES_PER_LANE,
            everything,
            &mebibytes_for_states
        ),
        STATES_PER_LANE,
        "放不下一条缓冲的也开一条"
    );
    assert_eq!(
        states_per_dispatch_of_a_pairing_judge(u64::MAX, STATES_PER_LANE, everything, &|_states| 1),
        STATES_PER_LANE * MAXIMUM_SCRATCH_LANES,
        "显存再多也不超过缓冲条数上限"
    );
    assert_eq!(
        states_per_dispatch_of_a_pairing_judge(13_824, STATES_PER_LANE, 100, &mebibytes_for_states),
        100,
        "这一段只有 100 个状态：草稿区开 100 个"
    );
}

/// 宿主内存那一本账：几十条线程抢，同时记着的状态数不超过上限，每一段都判到；账上空着时比上限大的一段也放行；票丢掉（连 panic 展开）都还。不用卡。
/// 判别力（`crates/mutations.tsv`）：放得下不放得下都放行，同时记着的超过上限，这条红；票丢掉不还，后面的等不到票、这条卡到超时，主线程判红。
#[test]
fn spans_queue_for_room_on_the_host_and_never_hold_more_states_than_the_room() {
    const THREADS: usize = 32;
    const STATES_OF_A_SPAN: u64 = 3;
    const MOST_STATES: u64 = 6;
    let queue = Arc::new(PairingStatesHeldOnTheHost::with_room_for_these_many_states(
        MOST_STATES,
    ));
    let states_judged = Arc::new(AtomicU64::new(0));
    let everyone_ready = Arc::new(std::sync::Barrier::new(THREADS));
    let (all_finished, all_finished_received) = mpsc::channel::<()>();
    // 不用线程作用域：票不还时后面的线程永远等着，这条要判红、不等它们
    for thread_index in 0..THREADS {
        let queue = Arc::clone(&queue);
        let states_judged = Arc::clone(&states_judged);
        let everyone_ready = Arc::clone(&everyone_ready);
        let all_finished = all_finished.clone();
        std::thread::spawn(move || {
            everyone_ready.wait();
            // 有几条线程判的时候 panic：票在展开时也还
            let judged = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _ticket = queue.wait_for_room_and_hold_the_states_of_a_span(STATES_OF_A_SPAN);
                std::thread::sleep(Duration::from_millis(5));
                assert!(thread_index % 8 != 0, "这条线程判的那一段判红");
            }));
            drop(judged);
            states_judged.fetch_add(STATES_OF_A_SPAN, Ordering::SeqCst);
            drop(all_finished);
        });
    }
    drop(all_finished);
    assert_eq!(
        all_finished_received.recv_timeout(Duration::from_secs(60)),
        Err(mpsc::RecvTimeoutError::Disconnected),
        "每条线程都排到了（一分钟之内全部收尾）：票丢掉都还"
    );
    let counts = queue.pairing_states_counts_locked();
    assert_eq!(
        states_judged.load(Ordering::SeqCst),
        u64::try_from(THREADS).expect("装得进 u64") * STATES_OF_A_SPAN,
        "每一段都判到"
    );
    assert_eq!(counts.states_held_now, 0, "全部还了");
    assert!(
        (STATES_OF_A_SPAN..=MOST_STATES).contains(&counts.most_states_held_so_far),
        "同时记着的到上限为止：实际最多 {}",
        counts.most_states_held_so_far
    );
    drop(counts);
    let oversized = queue.wait_for_room_and_hold_the_states_of_a_span(MOST_STATES + 1);
    assert_eq!(
        queue.pairing_states_counts_locked().states_held_now,
        MOST_STATES + 1,
        "账上空着时比上限大的一段也放行"
    );
    drop(oversized);
}

/// 两个上界照卡数算：一条流至多几段一起判、宿主内存里至多压着几个状态，都随卡数涨、与并行的用例数无关。不用卡。
/// 判别力（`crates/mutations.tsv`）：宿主内存那个上界不随卡数涨，两张卡那一格红；一条流几段一起判不随卡数涨，同一格红。
#[test]
fn the_bounds_of_the_pairing_grow_with_the_cards_and_nothing_else() {
    assert_eq!(
        (
            stretches_of_a_flow_judged_at_once_on_these_cards(1).get(),
            stretches_of_a_flow_judged_at_once_on_these_cards(2).get()
        ),
        (2, 4),
        "每张卡两个判器，一条流至多那么多段一起判"
    );
    let states_of_a_full_span = u64::try_from(PAIRING_BLOCKS_JUDGED_TOGETHER).expect("装得进 u64")
        * STATES_PER_PAIRING_BLOCK;
    assert_eq!(
        (
            most_pairing_states_held_on_the_host_of(1),
            most_pairing_states_held_on_the_host_of(2)
        ),
        (4 * states_of_a_full_span, 8 * states_of_a_full_span),
        "每张卡两个判器、每个判器的位子备两段满段"
    );
}

/// 对拍判器照它开的宽度建在卡上：至少两条草稿缓冲（判器量宽度的条件进得去；日志里 `CRASH_JUDGE_DISPATCH_CALIBRATED` 由真跑核），判出的逐状态结论与一条缓冲的判器相同。
/// 卡小到一半额度放不下两条缓冲的报本次未跑。流与展开同 `verdicts_are_the_same_whether_one_dispatch_spans_one_scratch_lane_or_several`。
/// 判别力（`crates/mutations.tsv`）：对拍判器的宽度一律一条缓冲，这条红。
#[test]
fn a_pairing_judge_is_wide_enough_to_calibrate_and_judges_like_one_scratch_lane() {
    const TEST_NAME: &str =
        "a_pairing_judge_is_wide_enough_to_calibrate_and_judges_like_one_scratch_lane";
    const EXPAND_UP_TO: usize = 16;
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let flow = record_fixed_script_through_rollback("crash-judge-gpu-pairing-width");
    let expansion = expansion_up_to(EXPAND_UP_TO);
    let judge_flow = JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &expansion,
    };
    let tables = build_judge_tables(&judge_flow).expect("建得出表");
    let state_count = tables.state_count;
    // 判表先入库、再读库，GPU 只拿读回的那一份算
    let Some(packed) = packed_tables_through_the_library(TEST_NAME, &tables) else {
        return;
    };
    let states_per_lane = states_of_one_scratch_lane_on_every_card_of(cards);
    assert!(
        state_count >= 2 * u64::try_from(states_per_lane).expect("装得进 u64"),
        "这条流只有 {state_count} 个状态，跨不过两条草稿缓冲"
    );
    let half_of_the_smallest_card = smallest_card_ceiling_of_the_pool_mebibytes(cards)
        / u64::try_from(PAIRING_JUDGES_SHARING_A_CARD).expect("装得进 u64");
    if mebibytes_a_judge_needs(&packed, 2 * states_per_lane) > half_of_the_smallest_card {
        println!(
            "{}",
            not_run_line(
                TEST_NAME,
                &format!(
                    "最小的一张卡一个判器只分得到 {half_of_the_smallest_card} MiB，放不下两条草稿缓冲"
                )
            )
        );
        return;
    }
    let requested =
        states_per_dispatch_of_a_pairing_judge_on_these_cards(cards, &packed, state_count);
    assert!(
        requested >= 2 * states_per_lane,
        "放得下两条缓冲的卡上，对拍判器开的宽度 {requested} 至少两条缓冲（一条 {states_per_lane} 个状态）：判器才量宽度"
    );
    let Some(lease) = cards.lease_card_or_report_not_run(
        TEST_NAME,
        mebibytes_a_judge_needs(&packed, requested)
            + mebibytes_a_judge_needs(&packed, states_per_lane),
    ) else {
        return;
    };
    let card = lease.card();
    let Some((wide, narrow)) = on_the_card_or_report_not_run_when_out_of_memory(
        TEST_NAME,
        card,
        "建两个判器与判这条流",
        || {
            let wide_judge = GpuCrashJudge::on_card_with_packed_tables(card, &packed, requested);
            let narrow_judge =
                GpuCrashJudge::on_card_with_packed_tables(card, &packed, states_per_lane);
            assert_eq!(
                wide_judge.states_per_dispatch(),
                requested,
                "借来的卡开得出借的那么宽"
            );
            (
                wide_judge.judge_states(0..state_count),
                narrow_judge.judge_states(0..state_count),
            )
        },
    ) else {
        return;
    };
    drop(lease);
    assert!(wide == narrow, "开宽的判器判出的结论与一条缓冲的逐状态相同");
}

/// 规划显卡用：同一条流（固定脚本到回退 D）在一张卡上按几种「一次派活的状态数」各判一遍，打每秒判多少态、草稿区占多少显存；
/// 换派活的粒度不许改判定：每一档的结论逐状态与第一档相同。到这张卡的上限（绑定上限、空闲显存）那一档为止。
#[test]
#[ignore = "量速度用：要一张空闲显存够的独显，几种派活粒度各把整条流判一遍"]
fn gpu_judge_throughput_by_states_per_dispatch() {
    let Some(card) =
        the_whole_emptiest_card_or_report_not_run("gpu_judge_throughput_by_states_per_dispatch")
    else {
        return;
    };
    let flow = record_fixed_script_through_rollback("crash-judge-gpu-throughput");
    let limit = expand_up_to();
    let expansion = move |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= limit {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    let judge_flow = JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &expansion,
    };
    let tables = build_judge_tables(&judge_flow).expect("建得出表");
    let state_count = tables.state_count;
    println!(
        "CRASH_JUDGE_THROUGHPUT_CARD card={:?} free_mebibytes={} max_storage_buffer_binding_size={} max_buffer_size={} states={state_count}",
        card.name,
        card.free_memory_mebibytes,
        card.device().limits().max_storage_buffer_binding_size,
        card.device().limits().max_buffer_size
    );
    let mut first_verdicts: Option<Vec<[u32; VERDICT_WORDS]>> = None;
    // 判表先入库、再读库，GPU 只拿读回的那一份算
    let Some(packed) =
        packed_tables_through_the_library("gpu_judge_throughput_by_states_per_dispatch", &tables)
    else {
        return;
    };
    let mut reached_the_limit_of_the_card = false;
    for requested in [
        256usize, 534, 1068, 2170, 4340, 8680, 13_020, 17_360, 26_040, 34_720,
    ] {
        if reached_the_limit_of_the_card {
            break;
        }
        let Some((judge, verdicts, build_seconds, seconds)) =
            on_the_card_or_report_not_run_when_out_of_memory(
                "gpu_judge_throughput_by_states_per_dispatch",
                &card,
                "按这一档派活粒度建判器与判这条流",
                || {
                    let build_started = Instant::now();
                    let judge =
                        GpuCrashJudge::on_card_with_packed_tables(&card, &packed, requested);
                    let build_seconds = build_started.elapsed().as_secs_f64();
                    let started = Instant::now();
                    let verdicts = judge.judge_states(0..state_count);
                    let seconds = started.elapsed().as_secs_f64();
                    (judge, verdicts, build_seconds, seconds)
                },
            )
        else {
            return;
        };
        let spent = judge.time_spent();
        println!(
            "CRASH_JUDGE_THROUGHPUT requested={requested} states_per_dispatch={} scratch_mebibytes={} judge_seconds={seconds:.2} states_per_second={:.0} build_seconds={build_seconds:.1} dispatches={} seconds_preparing={:.2} seconds_waiting_for_the_card={:.2} seconds_reading_back={:.2} card_memory_used_mebibytes={}",
            judge.states_per_dispatch(),
            judge.scratch_mebibytes(),
            state_count as f64 / seconds,
            spent.dispatches,
            spent.seconds_preparing,
            spent.seconds_waiting_for_the_card,
            spent.seconds_reading_back,
            memory_used_on_the_card_by_this_process()
        );
        match &first_verdicts {
            None => first_verdicts = Some(verdicts),
            Some(first) => assert!(
                first == &verdicts,
                "一次派活 {requested} 个状态判出来的结论与第一档不同"
            ),
        }
        if judge.states_per_dispatch() < requested {
            reached_the_limit_of_the_card = true;
        }
    }
}
