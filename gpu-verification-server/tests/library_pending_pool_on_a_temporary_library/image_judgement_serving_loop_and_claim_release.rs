//! 镜像判定那一族在取活的主循环里的样子：开卡与持卡、扫库与认领之间被别人写回的块、上一个进程留下的认领、停的时候放认领、服务日志的汇总行。

use super::*;

/// 取活循环起来就开卡（库里还没有活）；来了活判完写回，池空一段时间卡照持着；再来活不再开卡、照判；停的时候才放掉卡。
/// 弄坏开关：cards-opened-only-with-work（起来不开、有活才开）、cards-let-go-when-the-pool-idles（池空就放掉、再来活再开）。
#[test]
fn the_cards_are_opened_at_start_and_held_while_the_pool_idles() {
    let library = TemporaryLibraryService::start("cards-held");
    let conclusion_version = conclusion_version_of_the_crash_image_judgement();
    let configuration = ServiceConfiguration {
        oldest_pending_wait_limit: Duration::ZERO,
        library_scan_interval: SCAN_INTERVAL,
        claim_lease: Duration::from_millis(u64::from(LONG_LEASE_MILLISECONDS)),
    };
    let break_switches = BreakSwitches::of_the_environment();
    let mut pool = pool_on(
        &library,
        stand_in_side_with(&library, &[8]),
        LONG_LEASE_MILLISECONDS,
    );
    let openings = Arc::new(AtomicU64::new(0));
    let let_go = Arc::new(AtomicU64::new(0));
    let stop = Arc::new(AtomicBool::new(false));
    let first = block_of(
        8,
        0,
        0..4,
        conclusion_version,
        ForcedRejudging::NotForced,
        1,
    );
    let second = block_of(
        8,
        1,
        4..8,
        conclusion_version,
        ForcedRejudging::NotForced,
        2,
    );
    let store = library.open_store();
    let mut raw = library.open_client();
    let summary = std::thread::scope(|scope| {
        let stop_for_the_loop = Arc::clone(&stop);
        let openings_in_the_loop = Arc::clone(&openings);
        let let_go_in_the_loop = Arc::clone(&let_go);
        let serving = scope.spawn(move || {
            let mut open_the_cards = || -> Result<StandInCards, String> {
                openings_in_the_loop.fetch_add(1, Ordering::SeqCst);
                Ok(StandInCards {
                    let_go: Arc::clone(&let_go_in_the_loop),
                    dispatch_lines: Arc::new(Mutex::new(Vec::new())),
                    faults: StandInCardFaults::default(),
                })
            };
            serve_from_the_library(
                &configuration,
                &break_switches,
                &mut pool,
                &mut open_the_cards,
                &|| stop_for_the_loop.load(Ordering::SeqCst),
            )
        });
        let stop_on_drop = StopOnDrop(Arc::clone(&stop));
        wait_until(
            "库里还没有活，取活循环起来就开卡",
            &mut || openings.load(Ordering::SeqCst) == 1,
        );
        for (name, row) in [&first, &second] {
            put_pending_rows(&store, &[(*name, row.clone())]);
            wait_until(
                &format!("{name} 判完写回、待判行删掉"),
                &mut || !pending_names(&store).contains(name),
            );
            assert_eq!(
                stand_in_verdicts(&mut raw, row),
                Some(stand_in_verdicts_of(8, row.state_ordinals.clone())),
                "{name} 写回的是替身判的结论"
            );
            std::thread::sleep(IDLE_SPAN);
            assert_eq!(
                (
                    openings.load(Ordering::SeqCst),
                    let_go.load(Ordering::SeqCst)
                ),
                (1, 0),
                "池空着 {} 毫秒：卡照持着，没放、没再开",
                IDLE_SPAN.as_millis()
            );
        }
        drop(stop_on_drop);
        serving.join().expect("取活循环没 panic")
    })
    .unwrap_or_else(|failure| panic!("取活循环没跑下去：{failure:?}"));
    assert_eq!(summary.card_openings, 1, "起来开一回，之后不再开");
    assert_eq!(summary.cards_let_go_before_stopping, 0, "停之前不放卡");
    assert_eq!(summary.blocks_written_back, 2);
    assert_eq!(let_go.load(Ordering::SeqCst), 1, "停的时候放掉卡");
}

/// 另一个池扫完库、认领之前，先认领的池写回了这一块（那一帧删掉待判行、放掉认领键）：后一个池认领回「认领到」，
/// 重读待判行不在了：不取这一块，并放掉刚认领到的那一把（别的持有者随即认领得到），打一行可数的。
/// 一个场景按次序几步：扫库、写回、认领要在同一份库上依次发生。
/// 弄坏开关：pending-row-not-reread-after-the-claim（认领到之后不重读）。
#[test]
fn a_block_written_back_between_the_scan_and_the_claim_is_not_taken_again() {
    let library = TemporaryLibraryService::start("written-back-between-scan-and-claim");
    let conclusion_version = conclusion_version_of_the_crash_image_judgement();
    let block = block_of(
        9,
        0,
        0..4,
        conclusion_version,
        ForcedRejudging::NotForced,
        1,
    );
    put_pending_rows(&library.open_store(), std::slice::from_ref(&block));
    let mut judging = pool_on(
        &library,
        stand_in_side_with(&library, &[9]),
        LONG_LEASE_MILLISECONDS,
    );
    let mut scanning = pool_on(
        &library,
        stand_in_side_with(&library, &[9]),
        LONG_LEASE_MILLISECONDS,
    );
    scanning.keep_the_service_lines();
    let candidates = judging.candidates().expect("扫得了库");
    assert_eq!(
        judging
            .claim_the_candidates(candidates)
            .expect("认领得了")
            .taken
            .len(),
        1
    );
    let stale = scanning.candidates().expect("扫得了库");
    assert_eq!(stale.len(), 1, "写回之前扫，这一行还在、没判完");
    judging
        .write_back(&block.0, stand_in_verdicts_of(9, 0..4), &stand_in_card())
        .expect("先认领的池写回成了");
    let round = scanning.claim_the_candidates(stale).expect("认领得了");
    let taken: Vec<PendingBlockName> = round.taken.iter().map(|taken| taken.pending.name).collect();
    assert!(taken.is_empty(), "已写回的块又被取走：{taken:?}");
    assert_eq!(scanning.claimed_count(), 0, "没取的块不当在判");
    let mut other = library.open_client();
    assert_eq!(
        claim_a_pending_block(&mut other, &block.0, LONG_LEASE_MILLISECONDS).expect("认领得了"),
        ClaimAnswer::Claimed,
        "刚认领到的那一把放掉了：别的持有者随即认领得到，不等租期"
    );
    let released_lines = scanning
        .kept_service_lines()
        .iter()
        .filter(|line| line.starts_with(CLAIM_RELEASED_AFTER_THE_REREAD_LINE_PREFIX))
        .count();
    assert_eq!(released_lines, 1, "放掉刚认领到的那一把打一行");
}

/// 认领了块、没写回就没了的服务进程：它「在跑」那一行记的进程号此刻在、起来的时刻对不上（进程号让别的进程用了，就是它没了）；
/// 下一个进程起来时以它的持有者放掉它留下的认领，别的持有者随即认领得到，不等租期。
/// 一个场景按次序几步：上一个进程认领、没了，下一个进程起来放。
/// 弄坏开关：previous-process-claims-kept（起来时不放上一个进程的认领）。
#[test]
fn blocks_claimed_by_a_service_process_that_is_gone_are_released_at_the_next_start() {
    let library = TemporaryLibraryService::start("claims-of-a-process-that-is-gone");
    let conclusion_version = conclusion_version_of_the_crash_image_judgement();
    let block = block_of(
        10,
        0,
        0..4,
        conclusion_version,
        ForcedRejudging::NotForced,
        1,
    );
    put_pending_rows(&library.open_store(), std::slice::from_ref(&block));
    let mut gone = pool_on(
        &library,
        stand_in_side_with(&library, &[10]),
        LONG_LEASE_MILLISECONDS,
    );
    let candidates = gone.candidates().expect("扫得了库");
    assert_eq!(
        gone.claim_the_candidates(candidates)
            .expect("认领得了")
            .taken
            .len(),
        1
    );
    let this_process = ServiceProcessIdentity::of_this_process().expect("读得到这个进程的身份");
    let process_that_is_gone = ServiceProcessIdentity {
        process_number: this_process.process_number,
        start_ticks: this_process.start_ticks + 1,
        boot: this_process.boot,
    };
    let previous_serving_line = serving_line(&process_that_is_gone, 0, &gone.holder());
    drop(gone);
    let mut next = pool_on(
        &library,
        stand_in_side_with(&library, &[10]),
        LONG_LEASE_MILLISECONDS,
    );
    assert_eq!(
        next.release_the_claims_left_by_the_previous_process(Some(&previous_serving_line))
            .expect("读写得了判定库"),
        PreviousProcessClaims::Released(ClaimsReleased {
            released: 1,
            not_claimed: 0,
            held_by_others: 0,
        }),
        "上一个进程没了：以它的持有者放掉它认领的那一块"
    );
    let mut other = library.open_client();
    assert_eq!(
        claim_a_pending_block(&mut other, &block.0, LONG_LEASE_MILLISECONDS).expect("认领得了"),
        ClaimAnswer::Claimed,
        "别的持有者随即认领得到，不等租期"
    );
}

/// 上一个服务进程还活着（「在跑」那一行记的就是这个测试进程，进程号、起来的时刻、开机号都对得上）：它的认领不放，别的持有者认领回它持着。
/// 弄坏开关：previous-process-taken-for-gone（还活着也当没了，放掉它的认领）。
#[test]
fn blocks_claimed_by_a_service_process_still_running_are_not_released() {
    let library = TemporaryLibraryService::start("claims-of-a-process-still-running");
    let conclusion_version = conclusion_version_of_the_crash_image_judgement();
    let block = block_of(
        12,
        0,
        0..4,
        conclusion_version,
        ForcedRejudging::NotForced,
        1,
    );
    put_pending_rows(&library.open_store(), std::slice::from_ref(&block));
    let mut running = pool_on(
        &library,
        stand_in_side_with(&library, &[12]),
        LONG_LEASE_MILLISECONDS,
    );
    let candidates = running.candidates().expect("扫得了库");
    assert_eq!(
        running
            .claim_the_candidates(candidates)
            .expect("认领得了")
            .taken
            .len(),
        1
    );
    let this_process = ServiceProcessIdentity::of_this_process().expect("读得到这个进程的身份");
    let running_serving_line = serving_line(&this_process, 0, &running.holder());
    let mut next = pool_on(
        &library,
        stand_in_side_with(&library, &[12]),
        LONG_LEASE_MILLISECONDS,
    );
    assert_eq!(
        next.release_the_claims_left_by_the_previous_process(Some(&running_serving_line))
            .expect("读写得了判定库"),
        PreviousProcessClaims::KeptWhileItRuns
    );
    let mut other = library.open_client();
    let answer =
        claim_a_pending_block(&mut other, &block.0, LONG_LEASE_MILLISECONDS).expect("认领得了");
    assert!(
        matches!(&answer, ClaimAnswer::HeldByOther { holder, .. } if *holder == running.holder()),
        "还活着的那个进程的认领照旧挡着：{answer:?}"
    );
}

/// 服务停的时候（取活循环收尾之后）以自己的持有者放掉还持着的认领，别的持有者随即认领得到。
/// 弄坏开关：claims-kept-at-the-stop（停的时候不放）。
#[test]
fn claims_still_held_when_the_service_stops_are_released() {
    let library = TemporaryLibraryService::start("claims-released-at-the-stop");
    let conclusion_version = conclusion_version_of_the_crash_image_judgement();
    let block = block_of(
        13,
        0,
        0..4,
        conclusion_version,
        ForcedRejudging::NotForced,
        1,
    );
    put_pending_rows(&library.open_store(), std::slice::from_ref(&block));
    let mut stopping = pool_on(
        &library,
        stand_in_side_with(&library, &[13]),
        LONG_LEASE_MILLISECONDS,
    );
    let candidates = stopping.candidates().expect("扫得了库");
    assert_eq!(
        stopping
            .claim_the_candidates(candidates)
            .expect("认领得了")
            .taken
            .len(),
        1
    );
    assert_eq!(
        stopping
            .release_the_claims_of_this_process()
            .expect("读写得了判定库"),
        ClaimsReleased {
            released: 1,
            not_claimed: 0,
            held_by_others: 0,
        }
    );
    let mut other = library.open_client();
    assert_eq!(
        claim_a_pending_block(&mut other, &block.0, LONG_LEASE_MILLISECONDS).expect("认领得了"),
        ClaimAnswer::Claimed,
        "停的时候放掉了：别的持有者随即认领得到"
    );
}

/// 一块判下来：替身判器每判一段照真判器的样子打逐次派活的那两种行；与待判池收下的服务行一起过服务的日志过滤之后，
/// 逐次派活的行一行不剩、滤掉的行数就是替身打的行数；判完一块的汇总行恰好一行、态数是这一块的态数；认领到、写回成了各一行。
/// 弄坏开关：dispatch-lines-forwarded（服务日志不滤逐次派活的行）。
#[test]
fn a_judged_block_leaves_one_summary_line_and_no_dispatch_lines_in_the_service_log() {
    const STATES_OF_THE_BLOCK: u64 = 4;
    let library = TemporaryLibraryService::start("service-log-summary");
    let conclusion_version = conclusion_version_of_the_crash_image_judgement();
    let configuration = ServiceConfiguration {
        oldest_pending_wait_limit: Duration::ZERO,
        library_scan_interval: SCAN_INTERVAL,
        claim_lease: Duration::from_millis(u64::from(LONG_LEASE_MILLISECONDS)),
    };
    let break_switches = BreakSwitches::of_the_environment();
    let mut pool = pool_on(
        &library,
        stand_in_side_with(&library, &[11]),
        LONG_LEASE_MILLISECONDS,
    );
    pool.keep_the_service_lines();
    let block = block_of(
        11,
        0,
        0..STATES_OF_THE_BLOCK,
        conclusion_version,
        ForcedRejudging::NotForced,
        1,
    );
    let store = library.open_store();
    put_pending_rows(&store, std::slice::from_ref(&block));
    let dispatch_lines: DispatchLinesOfTheStandInJudges = Arc::new(Mutex::new(Vec::new()));
    let stop = Arc::new(AtomicBool::new(false));
    let summary = std::thread::scope(|scope| {
        let stop_for_the_loop = Arc::clone(&stop);
        let dispatch_lines_for_the_cards = Arc::clone(&dispatch_lines);
        let pool_in_the_loop = &mut pool;
        let configuration_of_the_loop = &configuration;
        let break_switches_of_the_loop = &break_switches;
        let serving = scope.spawn(move || {
            let mut open_the_cards = || -> Result<StandInCards, String> {
                Ok(StandInCards {
                    let_go: Arc::new(AtomicU64::new(0)),
                    dispatch_lines: Arc::clone(&dispatch_lines_for_the_cards),
                    faults: StandInCardFaults::default(),
                })
            };
            serve_from_the_library(
                configuration_of_the_loop,
                break_switches_of_the_loop,
                pool_in_the_loop,
                &mut open_the_cards,
                &|| stop_for_the_loop.load(Ordering::SeqCst),
            )
        });
        let stop_on_drop = StopOnDrop(Arc::clone(&stop));
        wait_until(
            &format!("{} 判完写回、待判行删掉", block.0),
            &mut || !pending_names(&store).contains(&block.0),
        );
        drop(stop_on_drop);
        serving.join().expect("取活循环没 panic")
    })
    .unwrap_or_else(|failure| panic!("取活循环没跑下去：{failure:?}"));
    assert_eq!(summary.blocks_written_back, 1);
    let lines_of_the_judges = dispatch_lines.lock().expect("没有线程拿着锁 panic").clone();
    assert!(
        !lines_of_the_judges.is_empty(),
        "替身判器打了逐次派活的行（不然这一条什么都没核）"
    );
    let mut unfiltered = String::new();
    for line in lines_of_the_judges.iter().chain(pool.kept_service_lines()) {
        unfiltered.push_str(line);
        unfiltered.push('\n');
    }
    let dropped = AtomicU64::new(0);
    let mut filtered = Vec::new();
    filter_the_service_log(
        unfiltered.as_bytes(),
        &mut filtered,
        &dropped,
        &break_switches,
    )
    .expect("内存里读写不会失败");
    let filtered = String::from_utf8(filtered).expect("转出来的是 UTF-8");
    let dispatch_lines_left: Vec<&str> = filtered
        .lines()
        .filter(|line| is_a_dispatch_line(line.as_bytes()))
        .collect();
    assert!(
        dispatch_lines_left.is_empty(),
        "服务日志里还有逐次派活的行：{dispatch_lines_left:?}"
    );
    assert_eq!(
        dropped.load(Ordering::Relaxed),
        u64::try_from(lines_of_the_judges.len()).expect("装得进 u64"),
        "滤掉的行数就是替身判器打的行数"
    );
    let summaries: Vec<&str> = filtered
        .lines()
        .filter(|line| line.starts_with(BLOCK_JUDGED_LINE_PREFIX))
        .collect();
    assert_eq!(summaries.len(), 1, "判完一块恰好一行汇总：{summaries:?}");
    let states_field = format!("states={STATES_OF_THE_BLOCK}");
    assert!(
        summaries[0]
            .split_whitespace()
            .any(|field| field == states_field),
        "汇总行的态数是这一块的态数：{}",
        summaries[0]
    );
    for prefix in [BLOCK_CLAIMED_LINE_PREFIX, CONCLUSION_WRITTEN_LINE_PREFIX] {
        assert_eq!(
            filtered
                .lines()
                .filter(|line| line.starts_with(prefix))
                .count(),
            1,
            "{prefix} 恰好一行：{filtered}"
        );
    }
}
