//! 镜像判定那一族在判定库那一份待判池上的取活与写回：别人持着不取、任意结论版本键照行写回、强制现判、写结论被拒、判完而可复用的块、守卫那一行被改过。

use super::*;

/// 别的持有者持着的不取；没人持着的照取（这一条不是因为什么都不取才绿）；别人持着的打一行可数的、再扫一轮不再打。
/// 弄坏开关：claim-held-by-others-ignored（别人持着也当认领到）。
#[test]
fn a_block_claimed_by_another_holder_is_left_to_it() {
    let library = TemporaryLibraryService::start("held-by-other");
    let conclusion_version = conclusion_version_of_the_crash_image_judgement();
    let held = block_of(
        1,
        0,
        0..4,
        conclusion_version,
        ForcedRejudging::NotForced,
        1,
    );
    let free = block_of(
        1,
        1,
        4..8,
        conclusion_version,
        ForcedRejudging::NotForced,
        2,
    );
    put_pending_rows(&library.open_store(), &[held.clone(), free.clone()]);
    let mut other = library.open_client();
    assert_eq!(
        claim_a_pending_block(&mut other, &held.0, LONG_LEASE_MILLISECONDS).expect("认领得了"),
        ClaimAnswer::Claimed
    );
    let mut pool = pool_on(
        &library,
        stand_in_side_with(&library, &[1]),
        LONG_LEASE_MILLISECONDS,
    );
    pool.keep_the_service_lines();
    let candidates = pool.candidates().expect("扫得了库");
    assert_eq!(candidates.len(), 2, "两块都还没判完，都挑出来");
    let round = pool.claim_the_candidates(candidates).expect("认领得了");
    let taken: Vec<PendingBlockName> = round.taken.iter().map(|block| block.pending.name).collect();
    assert_eq!(taken, vec![free.0], "只取没人持着的那一块");
    assert_eq!(
        round.held_by_others,
        vec![(held.0, other.client_identity().to_vec())],
        "别的持有者持着的那一块记下持有者、不取"
    );
    assert_eq!(pool.claimed_count(), 1);
    let rescanned = pool.candidates().expect("再扫一次");
    pool.claim_the_candidates(rescanned).expect("再认领一轮");
    let held_lines: Vec<&String> = pool
        .kept_service_lines()
        .iter()
        .filter(|line| line.starts_with(CLAIM_HELD_BY_OTHER_LINE_PREFIX))
        .collect();
    assert_eq!(
        held_lines.len(),
        1,
        "别人持着打一行、连着两轮持着只打头一轮：{held_lines:?}"
    );
    assert!(
        held_lines[0].contains(&held.0.to_string())
            && held_lines[0].contains(&format!(
                "holder={}",
                hexadecimal_text(other.client_identity())
            )),
        "那一行写块与持有者：{}",
        held_lines[0]
    );
    let answer_to_the_other =
        claim_a_pending_block(&mut other, &free.0, LONG_LEASE_MILLISECONDS).expect("认领得了");
    assert!(
        matches!(&answer_to_the_other, ClaimAnswer::HeldByOther { holder, .. } if *holder == pool.holder()),
        "取走的那一块认领在这个池手里，别人认领回别人持有：{answer_to_the_other:?}"
    );
}

/// 待判行里写什么结论版本键都照挑、照认领、照判，结论照行里写的那一把结论版本键写回：一块写验证包给判崩溃镜像那一种结论的那一把，一块写别的一把。
/// 两块都取到，待判行都删掉，不点名任何一行；服务不核行里的键。
/// 服务没有按键挑行与换键编帧的判法，也就没有对应的弄坏开关：任何一处按行里的键挑行、认领，或写回时换一把键编帧，这一条的断言就红。
#[test]
fn a_row_with_any_conclusion_version_is_claimed_and_written_under_the_row_key() {
    let library = TemporaryLibraryService::start("conclusion-version-of-the-row");
    let of_the_kind = conclusion_version_of_the_crash_image_judgement();
    let mut foreign = of_the_kind;
    foreign[31] ^= 0xff;
    let under_the_kind_version = block_of(6, 0, 0..4, of_the_kind, ForcedRejudging::NotForced, 1);
    let under_a_foreign_key = block_of(6, 1, 4..8, foreign, ForcedRejudging::NotForced, 2);
    let store = library.open_store();
    put_pending_rows(
        &store,
        &[under_the_kind_version.clone(), under_a_foreign_key.clone()],
    );
    let mut pool = pool_on(
        &library,
        stand_in_side_with(&library, &[6]),
        LONG_LEASE_MILLISECONDS,
    );
    let candidates = pool.candidates().expect("扫得了库");
    let candidate_names: Vec<PendingBlockName> =
        candidates.iter().map(|candidate| candidate.name).collect();
    assert_eq!(
        candidate_names,
        vec![under_the_kind_version.0, under_a_foreign_key.0],
        "行里写什么结论版本键都挑，不看它"
    );
    let round = pool.claim_the_candidates(candidates).expect("认领得了");
    assert_eq!(round.taken.len(), 2, "两块都认领到");
    let mut raw = library.open_client();
    for (block, ordinals) in [
        (&under_the_kind_version, 0..4),
        (&under_a_foreign_key, 4..8),
    ] {
        let verdicts = stand_in_verdicts_of(6, ordinals);
        let written = pool.write_back(&block.0, verdicts.clone(), &stand_in_card());
        assert!(written.is_ok(), "写回那一帧写成了：{written:?}");
        assert_eq!(
            stand_in_verdicts(&mut raw, &block.1),
            Some(verdicts),
            "结论落在待判行写的那一把结论版本键下"
        );
        assert_eq!(
            pending_row_in_the_library(&store, &block.0),
            None,
            "写回那一帧删掉了待判行"
        );
    }
    let other_key_of_the_foreign_block = pending_row(
        tables_digest_of(6),
        4..8,
        of_the_kind,
        ForcedRejudging::NotForced,
        2,
    );
    assert_eq!(
        stand_in_verdicts(&mut raw, &other_key_of_the_foreign_block),
        None,
        "写别的结论版本键的那一块，判崩溃镜像那一种结论的那一把键下没有它的结论"
    );
    assert!(
        pool.named_lines().is_empty(),
        "结论版本键怎么写都不点名：{:?}",
        pool.named_lines()
    );
}

/// 强制现判的块不问库里判到了哪一步（库里已有它不红的结论也照取），写回那一帧换上新结论、删待判行、写凭据行（持有者、结论摘要）；
/// 不是强制现判、库里已有不红结论的块问过库、不取。
/// 弄坏开关：forced-block-reads-the-library（强制现判也问库）、rejudged-row-not-written（不写凭据行）。
#[test]
fn a_forced_block_is_judged_without_reading_the_library_and_leaves_a_rejudged_row() {
    const POOL_ENTRY_NUMBER_OF_THE_FORCED: u64 = 7;
    let library = TemporaryLibraryService::start("forced");
    let conclusion_version = conclusion_version_of_the_crash_image_judgement();
    let forced = block_of(
        3,
        0,
        0..4,
        conclusion_version,
        ForcedRejudging::Forced,
        POOL_ENTRY_NUMBER_OF_THE_FORCED,
    );
    let judged_before = block_of(
        3,
        1,
        4..8,
        conclusion_version,
        ForcedRejudging::NotForced,
        8,
    );
    let store = library.open_store();
    put_pending_rows(&store, &[forced.clone(), judged_before.clone()]);
    let mut raw = library.open_client();
    store_stand_in_verdicts(&mut raw, &forced.1, &stand_in_verdicts_of(99, 0..4));
    store_stand_in_verdicts(&mut raw, &judged_before.1, &stand_in_verdicts_of(99, 4..8));
    let side = stand_in_side_with(&library, &[3]);
    let judging_questions = Arc::clone(&side.judging_questions);
    let mut pool = pool_on(&library, side, LONG_LEASE_MILLISECONDS);
    let candidates = pool.candidates().expect("扫得了库");
    let candidate_names: Vec<PendingBlockName> =
        candidates.iter().map(|candidate| candidate.name).collect();
    assert_eq!(
        candidate_names,
        vec![forced.0],
        "库里已有不红结论的：强制现判的照取，不是的不取"
    );
    assert_eq!(
        judging_questions.load(Ordering::SeqCst),
        1,
        "只问了不是强制现判的那一块判到了哪一步"
    );
    let round = pool.claim_the_candidates(candidates).expect("认领得了");
    assert_eq!(round.taken.len(), 1);
    let judged_now = red_stand_in_verdicts_of(3, 0..4);
    let written = pool
        .write_back(&forced.0, judged_now.clone(), &stand_in_card())
        .unwrap_or_else(|refused| panic!("写回没成：{}", refused.reason));
    assert_eq!(
        written.conclusion_digest,
        sha256_digest(&common::bytes_of_the_verdicts(&judged_now))
    );
    assert_eq!(
        stand_in_verdicts(&mut raw, &forced.1),
        Some(judged_now),
        "库里换上了现判的结论（现判的红，库里那一份不红：一份红整份换上）"
    );
    assert!(
        !pending_names(&store).contains(&forced.0),
        "写回那一帧删了强制现判那一行"
    );
    let rejudged = store
        .read_gpu_rejudged_row(
            POOL_ENTRY_NUMBER_OF_THE_FORCED,
            &forced.0.stream,
            forced.0.block_number,
        )
        .expect("读得了")
        .expect("写了凭据行（表 1E：入池序号、流键、块号）");
    assert_eq!(
        (rejudged.holder_identity, rejudged.conclusion_digest),
        (pool.holder(), written.conclusion_digest),
        "凭据行写判它的持有者与结论摘要"
    );
}

/// 写结论之前认领已被别的持有者持着（租期过了被认领走）：整帧不写，结论留在服务里（不丢）、报红，待判行还在库里。
/// 弄坏开关：refused-conclusions-dropped（结论不留，只打红行）。
#[test]
fn conclusions_refused_at_the_write_are_kept_and_reported_red() {
    let library = TemporaryLibraryService::start("refused-write");
    let conclusion_version = conclusion_version_of_the_crash_image_judgement();
    let block = block_of(
        4,
        0,
        0..4,
        conclusion_version,
        ForcedRejudging::NotForced,
        1,
    );
    let store = library.open_store();
    put_pending_rows(&store, std::slice::from_ref(&block));
    let mut pool = pool_on(
        &library,
        stand_in_side_with(&library, &[4]),
        SHORT_LEASE_MILLISECONDS,
    );
    let candidates = pool.candidates().expect("扫得了库");
    let round = pool.claim_the_candidates(candidates).expect("认领得了");
    assert_eq!(round.taken.len(), 1);
    std::thread::sleep(Duration::from_millis(
        u64::from(SHORT_LEASE_MILLISECONDS) * 2,
    ));
    let mut other = library.open_client();
    assert_eq!(
        claim_a_pending_block(&mut other, &block.0, LONG_LEASE_MILLISECONDS).expect("认领得了"),
        ClaimAnswer::Claimed,
        "租期过了，别的持有者认领走"
    );
    let verdicts = stand_in_verdicts_of(4, 0..4);
    let refused = pool
        .write_back(&block.0, verdicts.clone(), &stand_in_card())
        .expect_err("认领被别人持着，那一帧不写");
    assert!(
        refused
            .reason
            .contains(&hexadecimal_text(other.client_identity())),
        "红行点名持着的那一位：{}",
        refused.reason
    );
    assert_eq!(
        pool.refused_writes()
            .iter()
            .map(|kept| (kept.name, kept.verdicts.clone()))
            .collect::<Vec<_>>(),
        vec![(block.0, verdicts)],
        "没写成的结论留在服务里，不丢"
    );
    let mut raw = library.open_client();
    assert_eq!(
        stand_in_verdicts(&mut raw, &block.1),
        None,
        "整帧没写：库里没有结论行"
    );
    assert_eq!(pending_names(&store), vec![block.0], "待判行还在库里");
}

/// 判完而可复用的块不取，它的待判行带守卫删掉；判完而红的、没判完的照取；可复用而别的持有者持着认领的不删（归持有者）。
/// 弄坏开关：reusable-block-pending-row-kept（可复用的只不取、待判行不删）。
#[test]
fn a_reusable_block_is_not_taken_and_its_pending_row_is_deleted_under_the_guard() {
    let library = TemporaryLibraryService::start("reusable");
    let conclusion_version = conclusion_version_of_the_crash_image_judgement();
    let reusable = block_of(
        5,
        0,
        0..2,
        conclusion_version,
        ForcedRejudging::NotForced,
        1,
    );
    let red = block_of(
        5,
        1,
        2..4,
        conclusion_version,
        ForcedRejudging::NotForced,
        2,
    );
    let not_judged = block_of(
        5,
        2,
        4..6,
        conclusion_version,
        ForcedRejudging::NotForced,
        3,
    );
    let reusable_held = block_of(
        5,
        3,
        6..8,
        conclusion_version,
        ForcedRejudging::NotForced,
        4,
    );
    let store = library.open_store();
    put_pending_rows(
        &store,
        &[
            reusable.clone(),
            red.clone(),
            not_judged.clone(),
            reusable_held.clone(),
        ],
    );
    let mut raw = library.open_client();
    store_stand_in_verdicts(&mut raw, &reusable.1, &stand_in_verdicts_of(5, 0..2));
    store_stand_in_verdicts(&mut raw, &red.1, &red_stand_in_verdicts_of(5, 2..4));
    store_stand_in_verdicts(&mut raw, &reusable_held.1, &stand_in_verdicts_of(5, 6..8));
    let mut other = library.open_client();
    assert_eq!(
        claim_a_pending_block(&mut other, &reusable_held.0, LONG_LEASE_MILLISECONDS)
            .expect("认领得了"),
        ClaimAnswer::Claimed
    );
    let mut pool = pool_on(
        &library,
        stand_in_side_with(&library, &[5]),
        LONG_LEASE_MILLISECONDS,
    );
    let candidate_names: Vec<PendingBlockName> = pool
        .candidates()
        .expect("扫得了库")
        .iter()
        .map(|candidate| candidate.name)
        .collect();
    assert_eq!(
        candidate_names,
        vec![red.0, not_judged.0],
        "判完而红的、没判完的照取；可复用的两块不取"
    );
    assert_eq!(
        pending_names(&store),
        vec![red.0, not_judged.0, reusable_held.0],
        "可复用那一块的待判行删了；别的持有者持着认领的那一块不删"
    );
    assert_eq!(pool.reusable_rows_deleted(), 1);
}

/// 一块判着的时候，判段把它的待判行（不强制）升成强制现判：写结论那一帧删键对不上被拒，服务重读那一行、还是同一块，
/// 带重读到的那一份再写：结论落下、待判行删掉、凭据行照升过那一行的入池序号写。
/// 弄坏开关：pending-row-change-not-reread（不重读，当场报红）。
#[test]
fn a_pending_row_raised_to_forced_while_judged_is_reread_and_written_with_its_credential() {
    const POOL_ENTRY_NUMBER_OF_THE_RAISE: u64 = 9;
    let library = TemporaryLibraryService::start("raised-to-forced");
    let conclusion_version = conclusion_version_of_the_crash_image_judgement();
    let (name, not_forced) = block_of(
        6,
        0,
        0..4,
        conclusion_version,
        ForcedRejudging::NotForced,
        1,
    );
    let store = library.open_store();
    put_pending_rows(&store, &[(name, not_forced.clone())]);
    let mut pool = pool_on(
        &library,
        stand_in_side_with(&library, &[6]),
        LONG_LEASE_MILLISECONDS,
    );
    let candidates = pool.candidates().expect("扫得了库");
    let round = pool.claim_the_candidates(candidates).expect("认领得了");
    assert_eq!(round.taken.len(), 1);
    let mut raised = not_forced.clone();
    raised.forced = ForcedRejudging::Forced;
    raised.pool_entry_number = POOL_ENTRY_NUMBER_OF_THE_RAISE;
    assert_eq!(
        put_pending_rows(&store, &[(name, raised.clone())]),
        vec![GpuPendingRowPut::RaisedToForced {
            replaced: not_forced
        }],
        "判段照它的写法把不强制的那一行升成强制现判"
    );
    let verdicts = stand_in_verdicts_of(6, 0..4);
    let written = pool
        .write_back(&name, verdicts.clone(), &stand_in_card())
        .unwrap_or_else(|refused| panic!("写回没成：{}", refused.reason));
    assert!(
        written.wrote_the_rejudged_row,
        "重读到的是强制现判的那一份：带上凭据行"
    );
    let mut raw = library.open_client();
    assert_eq!(stand_in_verdicts(&mut raw, &raised), Some(verdicts));
    assert_eq!(
        pending_row_in_the_library(&store, &name),
        None,
        "升过的那一行照重读到的那一份删掉"
    );
    let rejudged = store
        .read_gpu_rejudged_row(
            POOL_ENTRY_NUMBER_OF_THE_RAISE,
            &name.stream,
            name.block_number,
        )
        .expect("读得了")
        .expect("凭据行照升过那一行的入池序号写");
    assert_eq!(rejudged.conclusion_digest, written.conclusion_digest);
}

/// 编帧之后、写之前别人往同一块写了一份结论：写结论那一帧守卫不过，服务重编（替身照存块的规矩重读、重比）再写：
/// 别人写的那一份红的，整份换上这一份；别人写的那一份不红而不同的，两份都绿而不同、报红，结论留在服务里，库里还是别人那一份。
/// 弄坏开关：guard-row-change-not-reread（守卫那一行变了不重编，当场报红）。
#[test]
fn a_conclusion_changed_under_the_guard_is_compared_again_before_writing() {
    let library = TemporaryLibraryService::start("guard-row-changed");
    let conclusion_version = conclusion_version_of_the_crash_image_judgement();
    let red_written_by_another = block_of(
        7,
        0,
        0..4,
        conclusion_version,
        ForcedRejudging::NotForced,
        1,
    );
    let green_written_by_another = block_of(
        7,
        1,
        4..8,
        conclusion_version,
        ForcedRejudging::NotForced,
        2,
    );
    let store = library.open_store();
    put_pending_rows(
        &store,
        &[
            red_written_by_another.clone(),
            green_written_by_another.clone(),
        ],
    );
    let side = stand_in_side_with(&library, &[7]);
    let another_writes = Arc::clone(&side.written_by_another_between_the_frame_and_the_write);
    let mut pool = pool_on(&library, side, LONG_LEASE_MILLISECONDS);
    let candidates = pool.candidates().expect("扫得了库");
    let round = pool.claim_the_candidates(candidates).expect("认领得了");
    assert_eq!(round.taken.len(), 2);
    let mut raw = library.open_client();

    let ours = stand_in_verdicts_of(7, 0..4);
    let red_of_another = red_stand_in_verdicts_of(70, 0..4);
    *another_writes.lock().expect("没有别的线程 panic 着持锁") = Some(red_of_another);
    let written = pool.write_back(&red_written_by_another.0, ours.clone(), &stand_in_card());
    assert!(
        written.is_ok(),
        "别人写的红：重编之后整份换上这一份，回的是 {:?}",
        written.err().map(|refused| refused.reason)
    );
    assert_eq!(
        stand_in_verdicts(&mut raw, &red_written_by_another.1),
        Some(ours)
    );

    let ours_green = stand_in_verdicts_of(7, 4..8);
    let green_of_another = stand_in_verdicts_of(71, 4..8);
    assert!(!is_stand_in_conclusion_red(&green_of_another));
    *another_writes.lock().expect("没有别的线程 panic 着持锁") = Some(green_of_another.clone());
    let refused = pool
        .write_back(&green_written_by_another.0, ours_green, &stand_in_card())
        .expect_err("两份都绿而不同：报红");
    assert!(
        refused.reason.contains("两份都不红"),
        "红行写两份都绿而不同：{}",
        refused.reason
    );
    assert_eq!(
        stand_in_verdicts(&mut raw, &green_written_by_another.1),
        Some(green_of_another),
        "库里还是别人那一份"
    );
}
