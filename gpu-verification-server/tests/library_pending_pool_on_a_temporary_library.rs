//! 判定库那一份待判池（`src/library_pending_pool.rs`）与取活循环（`src/serving_from_the_library.rs`），对着测试进程里起的临时判定库服务
//! （`prune_server::server::start_in_this_process`，与判定库服务自证同一种临时服务；不写真服务），不上卡：待判行照验证包的写法读写
//! （`VerdictStore` 的表 `1D`、`1E`），验证包那一侧另外三样换成替身（结论字原样存一行，判到了哪一步看那一行），卡与判器换成替身。
//! 每条的弄坏开关（`GPU_VERIFICATION_SERVER_BREAK=<项>`，表在 `src/break_switches.rs` 的 `BreakSwitch::red_test`）打开时那一条必须红。

mod common;

use std::collections::BTreeMap;
use std::ops::Range;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use common::{
    is_stand_in_conclusion_red, matching_expected_keys, pending_block_name, pending_names,
    pending_row, pending_row_in_the_library, put_pending_rows, stand_in_verdicts,
    store_stand_in_verdicts, version_keys_of_this_build, ExpectedVersionKeys,
    StandInVerificationPackageSide, TemporaryLibraryService, STAND_IN_RED_WORD,
};
use gpu_verification_server::break_switches::BreakSwitches;
use gpu_verification_server::card_memory_ledger::{CardIndex, CardOfTheService};
use gpu_verification_server::card_worker::JudgesOnACard;
use gpu_verification_server::cards_of_the_service::CardsOfTheService;
use gpu_verification_server::library_pending_pool::{
    claim_a_pending_block, ClaimAnswer, LibraryPendingPool, PendingBlockName,
    VERSION_MISMATCH_LINE_PREFIX,
};
use gpu_verification_server::pending_pool::StateVerdict;
use gpu_verification_server::service_configuration::ServiceConfiguration;
use gpu_verification_server::serving_from_the_library::serve_from_the_library;
use gpu_verification_server::stream_identity::{StreamFacts, StreamKey};
use singlefs_verification::crash_amplification::CardThatJudgedAPendingBlock;
use singlefs_verification::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER;
use singlefs_verification::crash_judge_gpu::VERDICT_WORDS;
use singlefs_verification::hexadecimal::hexadecimal_text;
use singlefs_verification::sha256::sha256_digest;
use singlefs_verification::verdict_store::{ForcedRejudging, GpuPendingRow, GpuPendingRowPut};

/// 认领的租期：别的用例不等它过期。
const LONG_LEASE_MILLISECONDS: u32 = 60_000;
/// 「写结论那一帧被拒」那一条的租期：等它过期、让别的持有者认领走。
const SHORT_LEASE_MILLISECONDS: u32 = 300;
/// 每条流的态数。
const STATES_OF_A_STREAM: u64 = 8;
/// 等取活循环做完一步的上限（替身判器一块几毫秒，扫库间隔 20 毫秒；10 秒是宽的）。
const STEP_WAIT: Duration = Duration::from_secs(10);
const STEP_POLL: Duration = Duration::from_millis(20);
/// 取活循环的扫库间隔。
const SCAN_INTERVAL: Duration = Duration::from_millis(20);
/// 「池空着卡也持着」那一条池空着等多久再核：十来个扫库间隔。
const IDLE_SPAN: Duration = Duration::from_millis(300);

/// 替身判器装一条流要的：这条流的标记。
struct StandInMaterials {
    tag: u32,
}

/// 替身判出的结论：流的标记、序号的低 32 位，其余字是 0（不红）。
fn stand_in_verdict(tag: u32, ordinal: u64) -> StateVerdict {
    let mut verdict = [0u32; VERDICT_WORDS];
    verdict[0] = tag;
    verdict[1] = u32::try_from(ordinal & 0xffff_ffff).expect("低 32 位");
    verdict
}

fn stand_in_verdicts_of(tag: u32, ordinals: Range<u64>) -> Vec<StateVerdict> {
    ordinals
        .map(|ordinal| stand_in_verdict(tag, ordinal))
        .collect()
}

/// 同 [`stand_in_verdicts_of`]，头一个态记红。
fn red_stand_in_verdicts_of(tag: u32, ordinals: Range<u64>) -> Vec<StateVerdict> {
    let mut verdicts = stand_in_verdicts_of(tag, ordinals);
    verdicts[0][STAND_IN_RED_WORD] = 1;
    verdicts
}

/// 第 `tag` 条流：判表摘要每个字节都是 `tag`。
fn tables_digest_of(tag: u8) -> [u8; 32] {
    [tag; 32]
}

fn stream_key_of(tag: u8) -> StreamKey {
    StreamKey {
        tables_digest: tables_digest_of(tag),
        kernel_sources_digest: [0; 32],
    }
}

/// 替身验证包一侧，登记了 `tags` 那几条流。
fn stand_in_side_with(
    library: &TemporaryLibraryService,
    tags: &[u8],
) -> StandInVerificationPackageSide<StandInMaterials> {
    let mut side = StandInVerificationPackageSide::on(library);
    for tag in tags {
        side.register(
            tables_digest_of(*tag),
            stream_key_of(*tag),
            StreamFacts {
                table_mebibytes: 1,
                state_count: STATES_OF_A_STREAM,
            },
            Arc::new(StandInMaterials {
                tag: u32::from(*tag),
            }),
        );
    }
    side
}

fn pool_on(
    library: &TemporaryLibraryService,
    side: StandInVerificationPackageSide<StandInMaterials>,
    claim_lease_milliseconds: u32,
) -> LibraryPendingPool<StandInVerificationPackageSide<StandInMaterials>> {
    LibraryPendingPool::new(
        side,
        library.open_client(),
        version_keys_of_this_build(),
        claim_lease_milliseconds,
        BreakSwitches::of_the_environment(),
    )
}

/// 第 `tag` 条流上的一块：名字与一行不强制的待判行。
fn block_of(
    tag: u8,
    block_number: u64,
    ordinals: Range<u64>,
    expected: ExpectedVersionKeys,
    forced: ForcedRejudging,
    pool_entry_number: u64,
) -> (PendingBlockName, GpuPendingRow) {
    (
        pending_block_name(tag, block_number),
        pending_row(
            tables_digest_of(tag),
            ordinals,
            expected,
            forced,
            pool_entry_number,
        ),
    )
}

/// 替身判这一块的那张卡。
fn stand_in_card() -> CardThatJudgedAPendingBlock {
    CardThatJudgedAPendingBlock {
        card_index: 0,
        card_name: "stand-in card".to_string(),
        card_driver_version: "stand-in driver".to_string(),
    }
}

/// 别的持有者持着的不取；没人持着的照取（这一条不是因为什么都不取才绿）。
/// 弄坏开关：claim-held-by-others-ignored（别人持着也当认领到）。
#[test]
fn a_block_claimed_by_another_holder_is_left_to_it() {
    let library = TemporaryLibraryService::start("held-by-other");
    let expected = matching_expected_keys(&version_keys_of_this_build());
    let held = block_of(1, 0, 0..4, expected, ForcedRejudging::NotForced, 1);
    let free = block_of(1, 1, 4..8, expected, ForcedRejudging::NotForced, 2);
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
    let candidates = pool.candidates().expect("扫得了库");
    assert_eq!(candidates.len(), 2, "两块都还没判完、版本对得上，都挑出来");
    let round = pool.claim_the_candidates(candidates).expect("认领得了");
    let taken: Vec<PendingBlockName> = round.taken.iter().map(|block| block.pending.name).collect();
    assert_eq!(taken, vec![free.0], "只取没人持着的那一块");
    assert_eq!(
        round.held_by_others,
        vec![(held.0, other.client_identity().to_vec())],
        "别的持有者持着的那一块记下持有者、不取"
    );
    assert_eq!(pool.claimed_count(), 1);
    let answer_to_the_other =
        claim_a_pending_block(&mut other, &free.0, LONG_LEASE_MILLISECONDS).expect("认领得了");
    assert!(
        matches!(&answer_to_the_other, ClaimAnswer::HeldByOther { holder, .. } if *holder == pool.holder()),
        "取走的那一块认领在这个池手里，别人认领回别人持有：{answer_to_the_other:?}"
    );
}

/// 期待的 GPU 判器组键与编进来的不同：不认领、点名一次（流键、块号、两边的 GPU 判器组键）；对得上的照取。
/// 弄坏开关：version-mismatch-ignored（不对 GPU 判器组键）。
#[test]
fn a_row_expecting_another_gpu_judge_is_not_claimed_and_is_named() {
    let library = TemporaryLibraryService::start("version-mismatch");
    let version_keys = version_keys_of_this_build();
    let matching = matching_expected_keys(&version_keys);
    let mut other_gpu_judge = matching.gpu_judge;
    other_gpu_judge[0] ^= 0xff;
    let judge_differs = block_of(
        2,
        0,
        0..2,
        ExpectedVersionKeys {
            gpu_judge: other_gpu_judge,
            conclusion_version: matching.conclusion_version,
        },
        ForcedRejudging::NotForced,
        1,
    );
    let matches = block_of(2, 1, 2..4, matching, ForcedRejudging::NotForced, 2);
    put_pending_rows(
        &library.open_store(),
        &[judge_differs.clone(), matches.clone()],
    );
    let mut pool = pool_on(
        &library,
        stand_in_side_with(&library, &[2]),
        LONG_LEASE_MILLISECONDS,
    );
    let candidates = pool.candidates().expect("扫得了库");
    let candidate_names: Vec<PendingBlockName> =
        candidates.iter().map(|candidate| candidate.name).collect();
    assert_eq!(
        candidate_names,
        vec![matches.0],
        "GPU 判器组键对不上的那一块不挑"
    );
    let round = pool.claim_the_candidates(candidates).expect("认领得了");
    assert_eq!(round.taken.len(), 1);
    let mut other = library.open_client();
    assert_eq!(
        claim_a_pending_block(&mut other, &judge_differs.0, LONG_LEASE_MILLISECONDS)
            .expect("认领得了"),
        ClaimAnswer::Claimed,
        "GPU 判器组键对不上的 {} 没被这个池认领，留给编出对的键的那一份",
        judge_differs.0
    );
    pool.candidates().expect("再扫一次");
    let mismatch_lines: Vec<&String> = pool
        .named_lines()
        .iter()
        .filter(|line| line.starts_with(VERSION_MISMATCH_LINE_PREFIX))
        .collect();
    assert_eq!(
        mismatch_lines.len(),
        1,
        "点名一次，扫第二次不再打：{mismatch_lines:?}"
    );
    assert!(
        mismatch_lines[0].contains(&judge_differs.0.to_string())
            && mismatch_lines[0].contains(&format!(
                "expected_gpu_judge={}",
                hexadecimal_text(&other_gpu_judge)
            ))
            && mismatch_lines[0].contains(&format!(
                "this_gpu_judge={}",
                hexadecimal_text(&version_keys.gpu_judge)
            )),
        "点名写流键、块号与两边的 GPU 判器组键：{}",
        mismatch_lines[0]
    );
}

/// GPU 判器组键与编进来的相同、期待的结论版本键是这个二进制算不出的一把（判段的测试二进制与服务编的时候依赖包开的特性不同，结论版本键就不同）：
/// 照挑、照认领、照判，结论照待判行里写的那一把结论版本键写回（这个二进制算的那一把下没有），待判行删掉，不点名版本对不上。
/// 弄坏开关：conclusion-version-checked（认领时另要结论版本键是这个二进制算得出的）、
/// conclusion-written-under-this-build-key（写回时结论帧换成这个二进制算的那一把）。
#[test]
fn a_row_whose_conclusion_version_this_build_does_not_compute_is_claimed_and_written_under_the_row_key(
) {
    let library = TemporaryLibraryService::start("conclusion-version-of-the-row");
    let version_keys = version_keys_of_this_build();
    let matching = matching_expected_keys(&version_keys);
    let mut foreign_conclusion_version = matching.conclusion_version;
    foreign_conclusion_version[31] ^= 0xff;
    assert!(
        !version_keys
            .conclusion_versions
            .contains(&foreign_conclusion_version),
        "造的那一把结论版本键不是这个二进制算得出的"
    );
    let foreign = ExpectedVersionKeys {
        gpu_judge: matching.gpu_judge,
        conclusion_version: foreign_conclusion_version,
    };
    let block = block_of(6, 0, 0..4, foreign, ForcedRejudging::NotForced, 1);
    let store = library.open_store();
    put_pending_rows(&store, std::slice::from_ref(&block));
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
        vec![block.0],
        "GPU 判器组键对得上就挑，不看结论版本键"
    );
    let round = pool.claim_the_candidates(candidates).expect("认领得了");
    assert_eq!(round.taken.len(), 1, "认领到");
    let verdicts = stand_in_verdicts_of(6, 0..4);
    let written = pool.write_back(&block.0, verdicts.clone(), &stand_in_card());
    assert!(written.is_ok(), "写回那一帧写成了：{written:?}");
    let mut raw = library.open_client();
    assert_eq!(
        stand_in_verdicts(&mut raw, &block.1),
        Some(verdicts),
        "结论落在待判行写的那一把结论版本键下"
    );
    let under_this_build_key = pending_row(
        tables_digest_of(6),
        0..4,
        matching,
        ForcedRejudging::NotForced,
        1,
    );
    assert_eq!(
        stand_in_verdicts(&mut raw, &under_this_build_key),
        None,
        "这个二进制算的那一把结论版本键下没有结论"
    );
    assert_eq!(
        pending_row_in_the_library(&store, &block.0),
        None,
        "写回那一帧删掉了待判行"
    );
    assert!(
        pool.named_lines()
            .iter()
            .all(|line| !line.starts_with(VERSION_MISMATCH_LINE_PREFIX)),
        "结论版本键算不出不算版本对不上，不点名：{:?}",
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
    let expected = matching_expected_keys(&version_keys_of_this_build());
    let forced = block_of(
        3,
        0,
        0..4,
        expected,
        ForcedRejudging::Forced,
        POOL_ENTRY_NUMBER_OF_THE_FORCED,
    );
    let judged_before = block_of(3, 1, 4..8, expected, ForcedRejudging::NotForced, 8);
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
    let expected = matching_expected_keys(&version_keys_of_this_build());
    let block = block_of(4, 0, 0..4, expected, ForcedRejudging::NotForced, 1);
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
    let expected = matching_expected_keys(&version_keys_of_this_build());
    let reusable = block_of(5, 0, 0..2, expected, ForcedRejudging::NotForced, 1);
    let red = block_of(5, 1, 2..4, expected, ForcedRejudging::NotForced, 2);
    let not_judged = block_of(5, 2, 4..6, expected, ForcedRejudging::NotForced, 3);
    let reusable_held = block_of(5, 3, 6..8, expected, ForcedRejudging::NotForced, 4);
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
    let expected = matching_expected_keys(&version_keys_of_this_build());
    let (name, not_forced) = block_of(6, 0, 0..4, expected, ForcedRejudging::NotForced, 1);
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
    let expected = matching_expected_keys(&version_keys_of_this_build());
    let red_written_by_another = block_of(7, 0, 0..4, expected, ForcedRejudging::NotForced, 1);
    let green_written_by_another = block_of(7, 1, 4..8, expected, ForcedRejudging::NotForced, 2);
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

/// 替身卡：一张，判器是替身；放掉（丢掉）时记一笔。
struct StandInCards {
    let_go: Arc<AtomicU64>,
}

impl Drop for StandInCards {
    fn drop(&mut self) {
        self.let_go.fetch_add(1, Ordering::SeqCst);
    }
}

struct StandInJudges {
    loaded: BTreeMap<StreamKey, u32>,
}

impl JudgesOnACard for StandInJudges {
    type StreamMaterials = StandInMaterials;

    fn load_the_judge(
        &mut self,
        stream: StreamKey,
        materials: &StandInMaterials,
        _states_per_dispatch: u64,
    ) {
        assert!(
            self.loaded.insert(stream, materials.tag).is_none(),
            "{stream:?} 装了两次"
        );
    }

    fn evict_the_judge(&mut self, stream: StreamKey) {
        assert!(
            self.loaded.remove(&stream).is_some(),
            "要换出的 {stream:?} 不在卡上"
        );
    }

    fn judge_ordinals(&mut self, stream: StreamKey, ordinals: Range<u64>) -> Vec<StateVerdict> {
        let tag = *self
            .loaded
            .get(&stream)
            .unwrap_or_else(|| panic!("要判的 {stream:?} 不在卡上"));
        stand_in_verdicts_of(tag, ordinals)
    }
}

impl CardsOfTheService for StandInCards {
    type Materials = StandInMaterials;
    type Judges<'cards> = StandInJudges;

    fn cards_of_the_service(&self) -> Vec<CardOfTheService> {
        vec![CardOfTheService {
            card: CardIndex(0),
            service_quota_mebibytes: MEBIBYTES_LEFT_FOR_THE_DRIVER + 4096,
        }]
    }

    fn judges_on(&self, _card: CardIndex) -> StandInJudges {
        StandInJudges {
            loaded: BTreeMap::new(),
        }
    }

    fn card_that_judged(&self, _card: CardIndex) -> CardThatJudgedAPendingBlock {
        stand_in_card()
    }
}

fn wait_until(what: &str, condition: &mut dyn FnMut() -> bool) {
    let deadline = Instant::now() + STEP_WAIT;
    while Instant::now() < deadline {
        if condition() {
            return;
        }
        std::thread::sleep(STEP_POLL);
    }
    panic!("{} 秒里没等到：{what}", STEP_WAIT.as_secs());
}

/// 测试那一侧结束（含 panic）时叫取活循环停。
struct StopOnDrop(Arc<AtomicBool>);

impl Drop for StopOnDrop {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

/// 取活循环起来就开卡（库里还没有活）；来了活判完写回，池空一段时间卡照持着；再来活不再开卡、照判；停的时候才放掉卡。
/// 弄坏开关：cards-opened-only-with-work（起来不开、有活才开）、cards-let-go-when-the-pool-idles（池空就放掉、再来活再开）。
#[test]
fn the_cards_are_opened_at_start_and_held_while_the_pool_idles() {
    let library = TemporaryLibraryService::start("cards-held");
    let expected = matching_expected_keys(&version_keys_of_this_build());
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
    let first = block_of(8, 0, 0..4, expected, ForcedRejudging::NotForced, 1);
    let second = block_of(8, 1, 4..8, expected, ForcedRejudging::NotForced, 2);
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
