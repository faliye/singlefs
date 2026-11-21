//! 核对待判行那一族（表 `1F`）：取活循环认领、核、写回核对行、删核对待判行、放认领，已核过的、两个守卫各被改一回。

use super::*;

/// 核对待判行那一族的池：替身验证包一侧照 `interference` 在编好核对行那一帧之后插进来写；另交回读材料的次数与插进来的设定（测试中途可改）。
fn facts_verifier_pool_on(
    library: &TemporaryLibraryService,
    interference: FactsVerifierFrameInterference,
) -> (
    LibraryPendingPool<StandInVerificationPackageSide<StandInMaterials>>,
    Arc<AtomicU64>,
    Arc<Mutex<FactsVerifierFrameInterference>>,
) {
    let side = StandInVerificationPackageSide::on(library);
    *side
        .facts_verifier_frame_interference
        .lock()
        .expect("没有线程拿着锁 panic") = interference;
    let material_questions = Arc::clone(&side.facts_material_questions);
    let interference_setting = Arc::clone(&side.facts_verifier_frame_interference);
    (
        pool_on(library, side, LONG_LEASE_MILLISECONDS),
        material_questions,
        interference_setting,
    )
}

/// 扫一次核对待判行、认领挑出来的，交回取到的各块（名字与重读到的行，按组、组里按键排）。
fn claim_the_facts_verifier_blocks(
    pool: &mut LibraryPendingPool<StandInVerificationPackageSide<StandInMaterials>>,
) -> Vec<(FactsVerifierBlockName, FactsVerifierPendingRow)> {
    let candidates = pool.facts_verifier_candidates().expect("扫得了库");
    pool.claim_the_facts_verifier_candidates(candidates)
        .expect("认领得了")
        .into_iter()
        .flat_map(|job| job.blocks)
        .map(|pending| (pending.name, pending.row))
        .collect()
}

/// 取活循环取核对待判行那一族：同一组的两块认领、交主循环在（替身）卡上核、带守卫写回核对行、删掉核对待判行；读回的核对行是这一块的事实表摘要与
/// 替身核对器的结论位，写回那一帧放掉了这一族的认领（别的持有者随即认领得到）；认领到、核完、写成各打一行，核完那一行的字段与判完一块那一行同名。
/// 弄坏开关：facts-verifier-rows-not-scanned（不扫这一张表）、facts-verifier-frame-under-the-image-judgement-claim-key（那一帧的认领键写成镜像判定那一族的形，
/// 这一族的认领没随那一帧放掉）。
#[test]
fn a_facts_verifier_pending_row_is_claimed_verified_and_written_back_by_the_serving_loop() {
    let library = TemporaryLibraryService::start("facts-verifier-served");
    let blocks = [
        facts_verifier_block(21, 0, 0..4, 1),
        facts_verifier_block(21, 1, 4..9, 2),
    ];
    let configuration = ServiceConfiguration {
        oldest_pending_wait_limit: Duration::ZERO,
        library_scan_interval: SCAN_INTERVAL,
        claim_lease: Duration::from_millis(u64::from(LONG_LEASE_MILLISECONDS)),
    };
    let break_switches = BreakSwitches::of_the_environment();
    let (mut pool, _, _) =
        facts_verifier_pool_on(&library, FactsVerifierFrameInterference::NoInterference);
    pool.keep_the_service_lines();
    let store = library.open_store();
    put_facts_verifier_pending_rows(&store, &blocks);
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
        wait_until("两块核对待判行核完写回、删掉", &mut || {
            facts_verifier_pending_names(&store).is_empty()
        });
        drop(stop_on_drop);
        serving.join().expect("取活循环没 panic")
    })
    .unwrap_or_else(|failure| panic!("取活循环没跑下去：{failure:?}"));
    assert_eq!(summary.facts_blocks_written_back, 2);
    assert_eq!(summary.facts_blocks_refused_at_the_write, 0);
    let mut other = library.open_client();
    for (name, row) in &blocks {
        let verifier_row = verifier_row_in_the_library(&store, row)
            .unwrap_or_else(|| panic!("{name} 核完没有核对行"));
        assert_eq!(verifier_row.facts_digest, row.facts_digest);
        assert_eq!(
            verifier_row.bits,
            stand_in_facts_bits(row.state_ordinals.clone()),
            "{name} 写回的是替身核对器的结论位"
        );
        assert_eq!(
            claim_a_facts_verifier_pending_block(&mut other, name, LONG_LEASE_MILLISECONDS)
                .expect("认领得了"),
            ClaimAnswer::Claimed,
            "{name}：写回那一帧放掉了这一族的认领，别的持有者随即认领得到"
        );
    }
    for prefix in [
        FACTS_VERIFIER_BLOCK_CLAIMED_LINE_PREFIX,
        FACTS_BLOCK_VERIFIED_LINE_PREFIX,
        FACTS_VERIFIER_WRITTEN_LINE_PREFIX,
    ] {
        let lines: Vec<&String> = pool
            .kept_service_lines()
            .iter()
            .filter(|line| line.starts_with(prefix))
            .collect();
        assert_eq!(lines.len(), 2, "{prefix} 每块一行：{lines:?}");
    }
    let verified_lines: Vec<&String> = pool
        .kept_service_lines()
        .iter()
        .filter(|line| line.starts_with(FACTS_BLOCK_VERIFIED_LINE_PREFIX))
        .collect();
    for line in verified_lines {
        assert_eq!(
            field_names_of(line),
            vec![
                "stream",
                "block",
                "states",
                "cards",
                "dispatches",
                "card_milliseconds"
            ],
            "核完一块那一行的字段与 {BLOCK_JUDGED_LINE_PREFIX} 那一行同名：{line}"
        );
    }
}

/// 核对行已在（这一块在行的核对键下核过）：不取、不读材料、不核，带守卫只删核对待判行；库里的核对行原样。
/// 弄坏开关：facts-verifier-verified-block-verified-again（不问核过没有，照取）。
#[test]
fn an_already_verified_facts_block_only_has_its_pending_row_deleted() {
    let library = TemporaryLibraryService::start("facts-verifier-already-verified");
    let block = facts_verifier_block(24, 0, 0..6, 1);
    let store = library.open_store();
    put_facts_verifier_pending_rows(&store, std::slice::from_ref(&block));
    let stored_bits: Vec<u32> = stand_in_facts_bits(0..6)
        .into_iter()
        .map(|bits| bits + 1)
        .collect();
    let frame = store
        .facts_verifier_rows_for_one_frame(
            &block.0.stream,
            block.0.block_number,
            &block.1,
            &facts_verifier_row_key_of(&block.1),
            &stored_bits,
        )
        .expect("编得出那一帧");
    let (key, value) = frame.verifier_row_to_put.expect("库里还没有核对行");
    write_a_raw_row(&mut library.open_client(), key, value);
    let (mut pool, material_questions, _) =
        facts_verifier_pool_on(&library, FactsVerifierFrameInterference::NoInterference);
    let candidates = pool.facts_verifier_candidates().expect("扫得了库");
    assert!(candidates.is_empty(), "已核过的不取：{candidates:?}");
    assert!(
        facts_verifier_pending_names(&store).is_empty(),
        "核对待判行带守卫删掉了"
    );
    assert_eq!(pool.verified_facts_rows_deleted(), 1);
    assert_eq!(
        material_questions.load(Ordering::SeqCst),
        0,
        "没读材料，也就没核"
    );
    assert_eq!(
        verifier_row_in_the_library(&store, &block.1).map(|row| row.bits),
        Some(stored_bits),
        "库里的核对行原样"
    );
}

/// 核对行那一帧编好之后、写之前，别的写方每一回都在这一块的核对键下写一份不同的核对行：核对行守卫每一回都不过，重编几回都落不下，
/// 那一帧一回都没落（核对待判行还在、库里的核对行是别的写方那一份），结论位留在服务里报红。
/// 弄坏开关：facts-verifier-row-guard-taken-at-the-write（核对行守卫取写之前此刻那一份）。
#[test]
fn a_facts_verifier_frame_whose_verifier_row_changed_after_it_was_read_does_not_land() {
    let library = TemporaryLibraryService::start("facts-verifier-row-guard");
    let block = facts_verifier_block(25, 0, 0..5, 1);
    let store = library.open_store();
    put_facts_verifier_pending_rows(&store, std::slice::from_ref(&block));
    let (mut pool, _, _) = facts_verifier_pool_on(
        &library,
        FactsVerifierFrameInterference::AnotherVerifierRowEveryTime,
    );
    assert_eq!(
        claim_the_facts_verifier_blocks(&mut pool),
        vec![block.clone()]
    );
    let refused = pool
        .write_back_the_facts_verifier_block(&block.0, stand_in_facts_bits(0..5))
        .expect_err("核对行每编一帧就被别的写方改掉：那一帧落不下");
    assert_eq!(refused.name, block.0);
    assert_eq!(
        pool.refused_facts_verifier_writes().len(),
        1,
        "结论位留在服务里"
    );
    assert_eq!(
        facts_verifier_pending_row_in_the_library(&store, &block.0),
        Some(block.1.clone()),
        "核对待判行没删"
    );
    let stored = verifier_row_in_the_library(&store, &block.1).expect("别的写方写的核对行在");
    assert_ne!(
        stored.bits,
        stand_in_facts_bits(0..5),
        "库里的是别的写方那一份：这一帧没落"
    );
}

/// 核对行那一帧编好之后、写之前，判段每一回都照同一块补写核对待判行（入池序号换一个）：核对待判行守卫每一回都不过，重读、重编几回都落不下，
/// 那一帧一回都没落（核对待判行还在、没有核对行），结论位留在服务里报红。
/// 弄坏开关：facts-verifier-pending-row-guard-taken-at-the-write（删核对待判行的期待取写之前此刻那一份）。
#[test]
fn a_facts_verifier_frame_whose_pending_row_changed_after_it_was_read_does_not_land() {
    let library = TemporaryLibraryService::start("facts-verifier-pending-row-guard");
    let block = facts_verifier_block(27, 0, 0..5, 1);
    let store = library.open_store();
    put_facts_verifier_pending_rows(&store, std::slice::from_ref(&block));
    let (mut pool, _, _) = facts_verifier_pool_on(
        &library,
        FactsVerifierFrameInterference::PendingRowRewrittenEveryTime,
    );
    assert_eq!(
        claim_the_facts_verifier_blocks(&mut pool),
        vec![block.clone()]
    );
    pool.write_back_the_facts_verifier_block(&block.0, stand_in_facts_bits(0..5))
        .expect_err("核对待判行每编一帧就被补写：那一帧落不下");
    assert_eq!(
        pool.refused_facts_verifier_writes().len(),
        1,
        "结论位留在服务里"
    );
    let left = facts_verifier_pending_row_in_the_library(&store, &block.0)
        .expect("核对待判行没删（补写的那一份在）");
    assert_ne!(
        left.pool_entry_number, block.1.pool_entry_number,
        "库里的是补写的那一份"
    );
    assert!(
        verifier_row_in_the_library(&store, &block.1).is_none(),
        "没有核对行：这一帧没落"
    );
}

/// 核对行那一帧被每个守卫各拒一回之后照种类重来、落下：核对行被别的写方写成逐字节相同的那一份，重编之后只删核对待判行；
/// 核对待判行被判段补写一回，重读之后照补写的那一份删、写核对行。两块都写成，核对待判行都删了，核对行是替身核对器的结论位。
/// 弄坏开关：facts-verifier-guard-refusal-not-retried（守卫不过当场报红）。
#[test]
fn a_facts_verifier_frame_refused_once_by_each_guard_is_composed_again_and_lands() {
    let library = TemporaryLibraryService::start("facts-verifier-guard-retried");
    let meets_the_same_verifier_row = facts_verifier_block(26, 0, 0..3, 1);
    let meets_a_rewritten_pending_row = facts_verifier_block(26, 1, 3..7, 2);
    let store = library.open_store();
    put_facts_verifier_pending_rows(
        &store,
        &[
            meets_the_same_verifier_row.clone(),
            meets_a_rewritten_pending_row.clone(),
        ],
    );
    let (mut pool, _, interference) = facts_verifier_pool_on(
        &library,
        FactsVerifierFrameInterference::TheSameVerifierRowOnce,
    );
    assert_eq!(
        claim_the_facts_verifier_blocks(&mut pool),
        vec![
            meets_the_same_verifier_row.clone(),
            meets_a_rewritten_pending_row.clone()
        ]
    );
    assert_eq!(
        pool.write_back_the_facts_verifier_block(
            &meets_the_same_verifier_row.0,
            stand_in_facts_bits(0..3)
        )
        .expect("核对行被写成相同的那一份之后重编、只删核对待判行"),
        FactsVerifierWrittenBack {
            verifier_row_written: false
        }
    );
    *interference.lock().expect("没有线程拿着锁 panic") =
        FactsVerifierFrameInterference::PendingRowRewrittenOnce;
    assert_eq!(
        pool.write_back_the_facts_verifier_block(
            &meets_a_rewritten_pending_row.0,
            stand_in_facts_bits(3..7)
        )
        .expect("核对待判行被补写一回之后重读、照补写的那一份删"),
        FactsVerifierWrittenBack {
            verifier_row_written: true
        }
    );
    assert!(pool.refused_facts_verifier_writes().is_empty());
    assert!(
        facts_verifier_pending_names(&store).is_empty(),
        "两块的核对待判行都删了"
    );
    for (name, row) in [&meets_the_same_verifier_row, &meets_a_rewritten_pending_row] {
        assert_eq!(
            verifier_row_in_the_library(&store, row).map(|stored| stored.bits),
            Some(stand_in_facts_bits(row.state_ordinals.clone())),
            "{name} 的核对行是替身核对器的结论位"
        );
    }
}

/// 核对待判行那一族的认领同别的族一起放：认领了一块、没写回就没了的服务进程（起来的时刻对不上），下一个进程起来时以它的持有者放掉，
/// 别的持有者随即认领得到；停的时候以自己的持有者放掉还持着的，别人持着的不动。
/// 弄坏开关：facts-verifier-claims-left-at-the-release（放认领只放待判行那一族的）。
#[test]
fn facts_verifier_claims_are_released_at_the_next_start_and_at_the_stop() {
    let library = TemporaryLibraryService::start("facts-verifier-claims-released");
    let left_by_the_gone = facts_verifier_block(28, 0, 0..4, 1);
    let held_at_the_stop = facts_verifier_block(29, 0, 0..4, 2);
    let store = library.open_store();
    put_facts_verifier_pending_rows(
        &store,
        &[left_by_the_gone.clone(), held_at_the_stop.clone()],
    );
    let (mut gone, _, _) =
        facts_verifier_pool_on(&library, FactsVerifierFrameInterference::NoInterference);
    let mut other = library.open_client();
    assert_eq!(
        claim_a_facts_verifier_pending_block(
            &mut other,
            &held_at_the_stop.0,
            LONG_LEASE_MILLISECONDS
        )
        .expect("认领得了"),
        ClaimAnswer::Claimed,
        "那一块先让别的持有者认领着，没了的那个进程只取到另一块"
    );
    assert_eq!(
        claim_the_facts_verifier_blocks(&mut gone),
        vec![left_by_the_gone.clone()]
    );
    let this_process = ServiceProcessIdentity::of_this_process().expect("读得到这个进程的身份");
    let process_that_is_gone = ServiceProcessIdentity {
        process_number: this_process.process_number,
        start_ticks: this_process.start_ticks + 1,
        boot: this_process.boot,
    };
    let previous_serving_line = serving_line(&process_that_is_gone, 0, &gone.holder());
    drop(gone);
    let (mut next, _, _) =
        facts_verifier_pool_on(&library, FactsVerifierFrameInterference::NoInterference);
    assert_eq!(
        next.release_the_claims_left_by_the_previous_process(Some(&previous_serving_line))
            .expect("读写得了判定库"),
        PreviousProcessClaims::Released(ClaimsReleased {
            released: 1,
            not_claimed: 0,
            held_by_others: 1,
        }),
        "上一个进程没了：以它的持有者放掉它认领的那一块核对待判行，别人持着的那一块不动"
    );
    let mut another = library.open_client();
    assert_eq!(
        claim_a_facts_verifier_pending_block(
            &mut another,
            &left_by_the_gone.0,
            LONG_LEASE_MILLISECONDS
        )
        .expect("认领得了"),
        ClaimAnswer::Claimed,
        "别的持有者随即认领得到，不等租期"
    );
    let (mut stopping, _, _) =
        facts_verifier_pool_on(&library, FactsVerifierFrameInterference::NoInterference);
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
        "先认领着那一块的持有者放掉它，停的那个池取得到"
    );
    assert_eq!(
        claim_the_facts_verifier_blocks(&mut stopping),
        vec![held_at_the_stop.clone()],
        "没了的那个进程留下的那一块此刻别的持有者持着，停的那个池只取到另一块"
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
        "停的时候放掉自己持着的那一块核对待判行，别人持着的不动"
    );
    assert_eq!(
        claim_a_facts_verifier_pending_block(
            &mut other,
            &held_at_the_stop.0,
            LONG_LEASE_MILLISECONDS
        )
        .expect("认领得了"),
        ClaimAnswer::Claimed,
        "停的时候放掉了：别的持有者随即认领得到"
    );
}

/// 另一个池扫完核对待判行、认领之前，先认领的池写回了这一块（那一帧删了核对待判行、放了认领）：后一个池认领回「认领到」，
/// 重读核对待判行不在了：不取，并放掉刚认领到的那一把，打一行可数的。
/// 弄坏开关：facts-verifier-row-not-reread-after-the-claim（认领到之后不重读）。
#[test]
fn a_facts_verifier_block_written_back_between_the_scan_and_the_claim_is_not_taken_again() {
    let library =
        TemporaryLibraryService::start("facts-verifier-written-back-between-scan-and-claim");
    let block = facts_verifier_block(30, 0, 0..4, 1);
    let store = library.open_store();
    put_facts_verifier_pending_rows(&store, std::slice::from_ref(&block));
    let (mut verifying, _, _) =
        facts_verifier_pool_on(&library, FactsVerifierFrameInterference::NoInterference);
    let (mut scanning, scanning_material_questions, _) =
        facts_verifier_pool_on(&library, FactsVerifierFrameInterference::NoInterference);
    scanning.keep_the_service_lines();
    assert_eq!(
        claim_the_facts_verifier_blocks(&mut verifying),
        vec![block.clone()]
    );
    let stale = scanning.facts_verifier_candidates().expect("扫得了库");
    assert_eq!(stale.len(), 1, "写回之前扫，这一行还在、没核过");
    verifying
        .write_back_the_facts_verifier_block(&block.0, stand_in_facts_bits(0..4))
        .expect("先认领的池写回成了");
    let jobs = scanning
        .claim_the_facts_verifier_candidates(stale)
        .expect("认领得了");
    assert!(jobs.is_empty(), "已写回的块又被取走");
    assert_eq!(scanning.facts_claimed_count(), 0, "没取的块不当在核");
    assert_eq!(
        scanning_material_questions.load(Ordering::SeqCst),
        0,
        "没取，也就没读材料"
    );
    let mut other = library.open_client();
    assert_eq!(
        claim_a_facts_verifier_pending_block(&mut other, &block.0, LONG_LEASE_MILLISECONDS)
            .expect("认领得了"),
        ClaimAnswer::Claimed,
        "刚认领到的那一把放掉了：别的持有者随即认领得到，不等租期"
    );
    let released_lines = scanning
        .kept_service_lines()
        .iter()
        .filter(|line| line.starts_with(FACTS_VERIFIER_CLAIM_RELEASED_AFTER_THE_REREAD_LINE_PREFIX))
        .count();
    assert_eq!(released_lines, 1, "放掉刚认领到的那一把打一行");
}
