//! 判表待判行那一族（表 `20`、`21`）：取活循环认领、判、写回结论字、删判表待判行、放认领，结论字已在库里的、两个守卫各被改一回。

use super::*;

use singlefs_verification::verdict_store::JudgingDevice;

/// 判表待判行那一族的池：替身验证包一侧登记了 `tags` 那几条流，照 `interference` 在编好结论字那一帧之后插进来写；
/// 另交回配流的次数与插进来的设定（测试中途可改）。
fn packed_tables_pool_on(
    library: &TemporaryLibraryService,
    tags: &[u8],
    interference: PackedTablesFrameInterference,
) -> (
    LibraryPendingPool<StandInVerificationPackageSide<StandInMaterials>>,
    Arc<AtomicU64>,
    Arc<Mutex<PackedTablesFrameInterference>>,
) {
    let side = stand_in_side_with(library, tags);
    *side
        .packed_tables_frame_interference
        .lock()
        .expect("没有线程拿着锁 panic") = interference;
    let stream_questions = Arc::clone(&side.packed_tables_stream_questions);
    let interference_setting = Arc::clone(&side.packed_tables_frame_interference);
    (
        pool_on(library, side, LONG_LEASE_MILLISECONDS),
        stream_questions,
        interference_setting,
    )
}

/// 第 `tag` 条流的那一张判表（态数是替身流的态数、不强制现判）。
fn packed_tables_of(tag: u8, pool_entry_number: u64) -> (PackedTablesName, PackedTablesPendingRow) {
    packed_tables_pending(
        tag,
        STATES_OF_A_STREAM,
        ForcedRejudging::NotForced,
        pool_entry_number,
    )
}

/// 扫一次判表待判行、认领挑出来的，交回取到的各张（名字与重读到的行，按键排）。
fn claim_the_packed_tables(
    pool: &mut LibraryPendingPool<StandInVerificationPackageSide<StandInMaterials>>,
) -> Vec<(PackedTablesName, PackedTablesPendingRow)> {
    let candidates = pool.packed_tables_candidates().expect("扫得了库");
    pool.claim_the_packed_tables_candidates(candidates)
        .expect("认领得了")
        .into_iter()
        .map(|taken| (taken.pending.name, taken.pending.row))
        .collect()
}

/// 取活循环取判表待判行那一族：两张判表认领、登记流、`0..态数` 一整块交主循环在（替身）卡上判、带守卫写回表 `21`、删掉判表待判行；
/// 读回的结论字是替身判器判的那一份，写回那一帧放掉了这一族的认领（别的持有者随即认领得到）；认领到、判完、写成各打一行，
/// 判完那一行除名字那一项外与判完一块那一行同名。
/// 弄坏开关：packed-tables-rows-not-scanned（不扫这一张表）。
#[test]
fn a_packed_tables_pending_row_is_claimed_judged_and_written_back_by_the_serving_loop() {
    let library = TemporaryLibraryService::start("packed-tables-served");
    let tables = [packed_tables_of(41, 1), packed_tables_of(42, 2)];
    let configuration = ServiceConfiguration {
        oldest_pending_wait_limit: Duration::ZERO,
        library_scan_interval: SCAN_INTERVAL,
        claim_lease: Duration::from_millis(u64::from(LONG_LEASE_MILLISECONDS)),
    };
    let break_switches = BreakSwitches::of_the_environment();
    let (mut pool, _, _) = packed_tables_pool_on(
        &library,
        &[41, 42],
        PackedTablesFrameInterference::NoInterference,
    );
    pool.keep_the_service_lines();
    let store = library.open_store();
    put_packed_tables_pending_rows(&store, &tables);
    let stop = Arc::new(AtomicBool::new(false));
    let summary = std::thread::scope(|scope| {
        let stop_for_the_loop = Arc::clone(&stop);
        let pool_in_the_loop = &mut pool;
        let configuration_of_the_loop = &configuration;
        let break_switches_of_the_loop = &break_switches;
        let serving = scope.spawn(move || {
            let mut open_the_cards = || -> Result<StandInCards, String> {
                Ok(StandInCards {
                    let_go: Arc::new(AtomicU64::new(0)),
                    dispatch_lines: Arc::new(Mutex::new(Vec::new())),
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
            "两张判表判完写回、判表待判行删掉",
            &mut || packed_tables_pending_names(&store).is_empty(),
        );
        drop(stop_on_drop);
        serving.join().expect("取活循环没 panic")
    })
    .unwrap_or_else(|failure| panic!("取活循环没跑下去：{failure:?}"));
    assert_eq!(summary.packed_tables_written_back, 2);
    assert_eq!(summary.packed_tables_refused_at_the_write, 0);
    let mut other = library.open_client();
    for (name, _) in &tables {
        let tag = u32::from(name.packed_judge_tables_digest[0]);
        assert_eq!(
            packed_tables_verdict_words_in_the_library(&store, name),
            Some(stand_in_verdicts_of(tag, 0..STATES_OF_A_STREAM)),
            "{name} 写回的是替身判器判的这张表全部状态的结论字"
        );
        assert_eq!(
            claim_a_packed_tables_pending_row(&mut other, name, LONG_LEASE_MILLISECONDS)
                .expect("认领得了"),
            ClaimAnswer::Claimed,
            "{name}：写回那一帧放掉了这一族的认领，别的持有者随即认领得到"
        );
    }
    for prefix in [
        PACKED_TABLES_CLAIMED_LINE_PREFIX,
        PACKED_TABLES_JUDGED_LINE_PREFIX,
        PACKED_TABLES_VERDICT_WORDS_WRITTEN_LINE_PREFIX,
    ] {
        let lines: Vec<&String> = pool
            .kept_service_lines()
            .iter()
            .filter(|line| line.starts_with(prefix))
            .collect();
        assert_eq!(lines.len(), 2, "{prefix} 每张一行：{lines:?}");
    }
    let judged_lines: Vec<&String> = pool
        .kept_service_lines()
        .iter()
        .filter(|line| line.starts_with(PACKED_TABLES_JUDGED_LINE_PREFIX))
        .collect();
    for line in judged_lines {
        assert_eq!(
            field_names_of(line),
            vec![
                "tables",
                "states",
                "cards",
                "dispatches",
                "card_milliseconds"
            ],
            "判完一张判表那一行除名字那一项外与 {BLOCK_JUDGED_LINE_PREFIX} 那一行同名：{line}"
        );
    }
}

/// 表 `21` 已有这张判表的结论字、判表待判行不强制现判：不取、不配流、不判，带守卫只删判表待判行；库里表 `21` 那一行原样。
/// 弄坏开关：packed-tables-verdict-words-judged-again（不问结论字在不在，照取）。
#[test]
fn packed_tables_whose_verdict_words_are_in_the_library_only_have_their_pending_row_deleted() {
    let library = TemporaryLibraryService::start("packed-tables-already-judged");
    let table = packed_tables_of(45, 1);
    let store = library.open_store();
    put_packed_tables_pending_rows(&store, std::slice::from_ref(&table));
    let stored_verdicts = stand_in_verdicts_of(99, 0..STATES_OF_A_STREAM);
    store_packed_tables_verdict_words(&library, &table.0, &table.1, &stored_verdicts);
    let (mut pool, stream_questions, _) = packed_tables_pool_on(
        &library,
        &[45],
        PackedTablesFrameInterference::NoInterference,
    );
    let candidates = pool.packed_tables_candidates().expect("扫得了库");
    assert!(candidates.is_empty(), "结论字已在的不取：{candidates:?}");
    assert!(
        packed_tables_pending_names(&store).is_empty(),
        "判表待判行带守卫删掉了"
    );
    assert_eq!(pool.packed_tables_rows_deleted(), 1);
    assert_eq!(
        stream_questions.load(Ordering::SeqCst),
        0,
        "没配流，也就没判"
    );
    assert_eq!(
        packed_tables_verdict_words_in_the_library(&store, &table.0),
        Some(stored_verdicts),
        "库里表 21 那一行原样"
    );
}

/// 结论字那一帧编好之后、写之前，别的写方每一回都在这张判表的键下写一份不同的结论字：表 `21` 守卫每一回都不过，重编几回都落不下，
/// 那一帧一回都没落（判表待判行还在、表 21 里是别的写方那一份），结论字留在服务里报红。
/// 弄坏开关：packed-tables-verdict-words-guard-taken-at-the-write（表 21 守卫取写之前此刻那一份）。
#[test]
fn a_packed_tables_frame_whose_verdict_words_row_changed_after_it_was_read_does_not_land() {
    let library = TemporaryLibraryService::start("packed-tables-verdict-words-guard");
    let table = packed_tables_of(46, 1);
    let store = library.open_store();
    put_packed_tables_pending_rows(&store, std::slice::from_ref(&table));
    let (mut pool, _, _) = packed_tables_pool_on(
        &library,
        &[46],
        PackedTablesFrameInterference::AnotherVerdictWordsRowEveryTime,
    );
    assert_eq!(claim_the_packed_tables(&mut pool), vec![table]);
    let served = stand_in_verdicts_of(46, 0..STATES_OF_A_STREAM);
    let refused = pool
        .write_back_the_packed_tables(&table.0, served.clone(), &stand_in_card())
        .expect_err("表 21 每编一帧就被别的写方改掉：那一帧落不下");
    assert_eq!(refused.name, table.0);
    assert_eq!(
        pool.refused_packed_tables_writes().len(),
        1,
        "结论字留在服务里"
    );
    assert_eq!(
        packed_tables_pending_row_in_the_library(&store, &table.0),
        Some(table.1),
        "判表待判行没删"
    );
    let stored = packed_tables_verdict_words_in_the_library(&store, &table.0)
        .expect("别的写方写的表 21 那一行在");
    assert_ne!(stored, served, "库里的是别的写方那一份：这一帧没落");
}

/// 结论字那一帧编好之后、写之前，判段每一回都照同一张判表补写判表待判行（入池序号换一个）：删键的守卫每一回都不过，重读、重编几回都落不下，
/// 那一帧一回都没落（判表待判行是补写的那一份、没有表 21 那一行），结论字留在服务里报红。
/// 弄坏开关：packed-tables-pending-row-guard-taken-at-the-write（删判表待判行的期待取写之前此刻那一份）。
#[test]
fn a_packed_tables_frame_whose_pending_row_changed_after_it_was_read_does_not_land() {
    let library = TemporaryLibraryService::start("packed-tables-pending-row-guard");
    let table = packed_tables_of(47, 1);
    let store = library.open_store();
    put_packed_tables_pending_rows(&store, std::slice::from_ref(&table));
    let (mut pool, _, _) = packed_tables_pool_on(
        &library,
        &[47],
        PackedTablesFrameInterference::PendingRowRewrittenEveryTime,
    );
    assert_eq!(claim_the_packed_tables(&mut pool), vec![table]);
    pool.write_back_the_packed_tables(
        &table.0,
        stand_in_verdicts_of(47, 0..STATES_OF_A_STREAM),
        &stand_in_card(),
    )
    .expect_err("判表待判行每编一帧就被补写：那一帧落不下");
    assert_eq!(
        pool.refused_packed_tables_writes().len(),
        1,
        "结论字留在服务里"
    );
    let left = packed_tables_pending_row_in_the_library(&store, &table.0)
        .expect("判表待判行没删（补写的那一份在）");
    assert_ne!(
        left.pool_entry_number, table.1.pool_entry_number,
        "库里的是补写的那一份"
    );
    assert!(
        packed_tables_verdict_words_in_the_library(&store, &table.0).is_none(),
        "没有表 21 那一行：这一帧没落"
    );
}

/// 结论字那一帧被每个守卫各拒一回之后照种类重来、落下：表 `21` 被别的写方写成逐字节相同的那一份，重编之后照此刻那一份写；
/// 判表待判行被判段补写一回，重读之后照补写的那一份删。两张都写成，判表待判行都删了，表 21 是替身判器判的结论字。
/// 弄坏开关：packed-tables-guard-refusal-not-retried（守卫不过当场报红）。
#[test]
fn a_packed_tables_frame_refused_once_by_each_guard_is_composed_again_and_lands() {
    let library = TemporaryLibraryService::start("packed-tables-guard-retried");
    let meets_the_same_verdict_words_row = packed_tables_of(48, 1);
    let meets_a_rewritten_pending_row = packed_tables_of(49, 2);
    let store = library.open_store();
    put_packed_tables_pending_rows(
        &store,
        &[
            meets_the_same_verdict_words_row,
            meets_a_rewritten_pending_row,
        ],
    );
    let (mut pool, _, interference) = packed_tables_pool_on(
        &library,
        &[48, 49],
        PackedTablesFrameInterference::TheSameVerdictWordsRowOnce,
    );
    assert_eq!(
        claim_the_packed_tables(&mut pool),
        vec![
            meets_the_same_verdict_words_row,
            meets_a_rewritten_pending_row
        ]
    );
    pool.write_back_the_packed_tables(
        &meets_the_same_verdict_words_row.0,
        stand_in_verdicts_of(48, 0..STATES_OF_A_STREAM),
        &stand_in_card(),
    )
    .expect("表 21 被写成相同的那一份之后重编、照此刻那一份写");
    *interference.lock().expect("没有线程拿着锁 panic") =
        PackedTablesFrameInterference::PendingRowRewrittenOnce;
    pool.write_back_the_packed_tables(
        &meets_a_rewritten_pending_row.0,
        stand_in_verdicts_of(49, 0..STATES_OF_A_STREAM),
        &stand_in_card(),
    )
    .expect("判表待判行被补写一回之后重读、照补写的那一份删");
    assert!(pool.refused_packed_tables_writes().is_empty());
    assert!(
        packed_tables_pending_names(&store).is_empty(),
        "两张的判表待判行都删了"
    );
    for (name, _) in [
        &meets_the_same_verdict_words_row,
        &meets_a_rewritten_pending_row,
    ] {
        let tag = u32::from(name.packed_judge_tables_digest[0]);
        assert_eq!(
            packed_tables_verdict_words_in_the_library(&store, name),
            Some(stand_in_verdicts_of(tag, 0..STATES_OF_A_STREAM)),
            "{name} 的表 21 是替身判器判的结论字"
        );
    }
}

/// 判表待判行那一族的认领同三族一起放：认领了一张、没写回就没了的服务进程（起来的时刻对不上），下一个进程起来时以它的持有者放掉，
/// 别的持有者随即认领得到；停的时候以自己的持有者放掉还持着的，别人持着的不动。
/// 一个场景按次序几步：上一个进程认领、没了，下一个进程起来放，再一个进程认领、停的时候放。
/// 弄坏开关：packed-tables-claims-left-at-the-release（放认领不放这一族的）。
#[test]
fn packed_tables_claims_are_released_at_the_next_start_and_at_the_stop() {
    let library = TemporaryLibraryService::start("packed-tables-claims-released");
    let left_by_the_gone = packed_tables_of(50, 1);
    let held_at_the_stop = packed_tables_of(51, 2);
    let store = library.open_store();
    put_packed_tables_pending_rows(&store, &[left_by_the_gone, held_at_the_stop]);
    let (mut gone, _, _) = packed_tables_pool_on(
        &library,
        &[50, 51],
        PackedTablesFrameInterference::NoInterference,
    );
    let mut other = library.open_client();
    assert_eq!(
        claim_a_packed_tables_pending_row(&mut other, &held_at_the_stop.0, LONG_LEASE_MILLISECONDS)
            .expect("认领得了"),
        ClaimAnswer::Claimed,
        "那一张先让别的持有者认领着，没了的那个进程只取到另一张"
    );
    assert_eq!(claim_the_packed_tables(&mut gone), vec![left_by_the_gone]);
    let this_process = ServiceProcessIdentity::of_this_process().expect("读得到这个进程的身份");
    let process_that_is_gone = ServiceProcessIdentity {
        process_number: this_process.process_number,
        start_ticks: this_process.start_ticks + 1,
        boot: this_process.boot,
    };
    let previous_serving_line = serving_line(&process_that_is_gone, 0, &gone.holder());
    drop(gone);
    let (mut next, _, _) = packed_tables_pool_on(
        &library,
        &[50, 51],
        PackedTablesFrameInterference::NoInterference,
    );
    assert_eq!(
        next.release_the_claims_left_by_the_previous_process(Some(&previous_serving_line))
            .expect("读写得了判定库"),
        PreviousProcessClaims::Released(ClaimsReleased {
            released: 1,
            not_claimed: 0,
            held_by_others: 1,
        }),
        "上一个进程没了：以它的持有者放掉它认领的那一张判表，别人持着的那一张不动"
    );
    let mut another = library.open_client();
    assert_eq!(
        claim_a_packed_tables_pending_row(
            &mut another,
            &left_by_the_gone.0,
            LONG_LEASE_MILLISECONDS
        )
        .expect("认领得了"),
        ClaimAnswer::Claimed,
        "别的持有者随即认领得到，不等租期"
    );
    let (mut stopping, _, _) = packed_tables_pool_on(
        &library,
        &[50, 51],
        PackedTablesFrameInterference::NoInterference,
    );
    let holder_of_the_other = other.client_identity().to_vec();
    assert_eq!(
        other
            .release_claims_as(
                &holder_of_the_other,
                &[(
                    prune_server::protocol::Table::Conclusions,
                    held_at_the_stop.0.claim_key(),
                )],
            )
            .expect("放得了")
            .len(),
        1,
        "先认领着那一张的持有者放掉它，停的那个池取得到"
    );
    assert_eq!(
        claim_the_packed_tables(&mut stopping),
        vec![held_at_the_stop],
        "没了的那个进程留下的那一张此刻别的持有者持着，停的那个池只取到另一张"
    );
    assert_eq!(
        stopping
            .release_the_claims_of_this_process()
            .expect("读写得了判定库"),
        ClaimsReleased {
            released: 1,
            not_claimed: 0,
            held_by_others: 1,
        },
        "停的时候放掉自己持着的那一张判表，别人持着的不动"
    );
    assert_eq!(
        claim_a_packed_tables_pending_row(&mut other, &held_at_the_stop.0, LONG_LEASE_MILLISECONDS)
            .expect("认领得了"),
        ClaimAnswer::Claimed,
        "停的时候放掉了：别的持有者随即认领得到"
    );
}

/// 另一个池扫完判表待判行、认领之前，先认领的池写回了这张判表（那一帧删了判表待判行、放了认领）：后一个池认领回「认领到」，
/// 重读判表待判行不在了：不取、不配流，并放掉刚认领到的那一把，打一行可数的。
/// 一个场景按次序几步：扫库、写回、认领要在同一份库上依次发生。
/// 弄坏开关：packed-tables-row-not-reread-after-the-claim（认领到之后不重读）。
#[test]
fn a_packed_tables_row_written_back_between_the_scan_and_the_claim_is_not_taken_again() {
    let library =
        TemporaryLibraryService::start("packed-tables-written-back-between-scan-and-claim");
    let table = packed_tables_of(52, 1);
    let store = library.open_store();
    put_packed_tables_pending_rows(&store, std::slice::from_ref(&table));
    let (mut judging, _, _) = packed_tables_pool_on(
        &library,
        &[52],
        PackedTablesFrameInterference::NoInterference,
    );
    let (mut scanning, scanning_stream_questions, _) = packed_tables_pool_on(
        &library,
        &[52],
        PackedTablesFrameInterference::NoInterference,
    );
    scanning.keep_the_service_lines();
    assert_eq!(claim_the_packed_tables(&mut judging), vec![table]);
    let stale = scanning.packed_tables_candidates().expect("扫得了库");
    assert_eq!(stale.len(), 1, "写回之前扫，这一行还在、结论字不在");
    judging
        .write_back_the_packed_tables(
            &table.0,
            stand_in_verdicts_of(52, 0..STATES_OF_A_STREAM),
            &stand_in_card(),
        )
        .expect("先认领的池写回成了");
    let taken = scanning
        .claim_the_packed_tables_candidates(stale)
        .expect("认领得了");
    assert!(taken.is_empty(), "已写回的判表又被取走");
    assert_eq!(scanning.packed_tables_claimed_count(), 0, "没取的不当在判");
    assert_eq!(
        scanning_stream_questions.load(Ordering::SeqCst),
        0,
        "没取，也就没配流"
    );
    let mut other = library.open_client();
    assert_eq!(
        claim_a_packed_tables_pending_row(&mut other, &table.0, LONG_LEASE_MILLISECONDS)
            .expect("认领得了"),
        ClaimAnswer::Claimed,
        "刚认领到的那一把放掉了：别的持有者随即认领得到，不等租期"
    );
    let released_lines = scanning
        .kept_service_lines()
        .iter()
        .filter(|line| line.starts_with(PACKED_TABLES_CLAIM_RELEASED_AFTER_THE_REREAD_LINE_PREFIX))
        .count();
    assert_eq!(released_lines, 1, "放掉刚认领到的那一把打一行");
}

/// 写回的结论字那一行（表 `21`）的设备属性只有卡型与驱动版本，不记 GPU 判器是哪一版：读回的设备是判它的那张卡的卡型与驱动版本，
/// 整行的字节数恰好是态数、逐态的结论字与设备属性（一个头字节、卡型与驱动版本各带 4 字节长度的文字）那么多，设备属性后面没有别的栏。
#[test]
fn the_written_back_verdict_words_row_records_the_card_and_the_driver_and_no_judge_version() {
    let library = TemporaryLibraryService::start("packed-tables-device-attribute");
    let table = packed_tables_of(47, 1);
    let store = library.open_store();
    put_packed_tables_pending_rows(&store, std::slice::from_ref(&table));
    let (mut pool, _, _) = packed_tables_pool_on(
        &library,
        &[47],
        PackedTablesFrameInterference::NoInterference,
    );
    assert_eq!(claim_the_packed_tables(&mut pool), vec![table]);
    let card = stand_in_card();
    let written = pool.write_back_the_packed_tables(
        &table.0,
        stand_in_verdicts_of(47, 0..STATES_OF_A_STREAM),
        &card,
    );
    assert!(written.is_ok(), "写回那一帧写成了：{written:?}");
    let stored = store
        .read_packed_tables_verdict_words(&table.0.library_key())
        .expect("读得了表 21")
        .expect("写回那一帧写下了表 21 那一行");
    assert_eq!(
        stored.judged_on,
        JudgingDevice::Gpu {
            card_type: card.card_name.clone(),
            driver_version: card.card_driver_version.clone(),
        },
        "设备属性是判它的那张卡的卡型与驱动版本"
    );
    let words_bytes =
        8 + usize::try_from(STATES_OF_A_STREAM).expect("态数装得进 usize") * VERDICT_WORDS * 4;
    let device_bytes = 1 + 4 + card.card_name.len() + 4 + card.card_driver_version.len();
    assert_eq!(
        stored.to_value_bytes().len(),
        words_bytes + device_bytes,
        "表 21 那一行的值里，结论字之后只有设备属性的头字节、卡型与驱动版本，没有记判器版本的栏"
    );
}
