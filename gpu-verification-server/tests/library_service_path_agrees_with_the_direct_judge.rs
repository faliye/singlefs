//! 对拍（要卡，经卡池借）：新池新建文件那条流切成几块，照判段的写法（验证包 `put_the_gpu_pending_row_unless_present_or_raise_it_to_forced`，表 `1D`）
//! 写成待判行，放进测试进程里起的临时判定库服务（不写真服务）；取活循环（`serving_from_the_library`）起来就开卡（借来的那一张）、取走、判、
//! 带守卫写回、删掉待判行，读回的结论字逐块接起来，与直接用 `GpuCrashJudge` 判的逐态相同。验证包那一侧编结论帧换成替身（结论字原样存一行）：
//! 这一条核的是取活、认领、调度、上卡、写回整条路不改结论字；结论帧怎么编由验证包那一侧的测试管
//! （`crash_amplification::tests::a_conclusion_frame_of_a_pending_block_written_as_one_batch_is_the_conclusion_the_judge_stage_stores`）。
//! 这台机器没有 NVIDIA 的卡、借不到卡时打一行「本次未跑」返回（`gpu_unit_checks::gpu_test_start`）；判表只录那一趟打
//! `JUDGE_TABLES_RECORDED_ONLY` 返回。判表先经对拍那一类的库入库、再读库（项名是用例名）。

mod common;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use common::{
    conclusion_version_of_the_crash_image_judgement, new_pool_file_creation_judge_tables,
    pending_block_name, pending_names, pending_row, put_pending_rows, stand_in_verdicts,
    StandInVerificationPackageSide, TemporaryLibraryService,
};
use gpu_verification_server::break_switches::BreakSwitches;
use gpu_verification_server::gpu_judges_on_a_card::{
    GpuStreamMaterials, JudgeMaterialsOnACard, OpenedGpuCards,
};
use gpu_verification_server::library_pending_pool::{LibraryPendingPool, PendingBlockName};
use gpu_verification_server::pending_pool::StateVerdict;
use gpu_verification_server::service_configuration::ServiceConfiguration;
use gpu_verification_server::serving_from_the_library::serve_from_the_library;
use singlefs_verification::crash_judge_dispatch::{
    mebibytes_a_judge_needs, GpuCrashJudge, PackedJudgeTables,
};
use singlefs_verification::crash_judge_gpu::JudgeKernelSources;
use singlefs_verification::crash_library::{
    judge_tables_through_the_pairing_library, StageOutcome,
};
use singlefs_verification::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
use singlefs_verification::verdict_store::{ForcedRejudging, GpuPendingRow};

/// 展开上限：与 `gpu_service_path_agrees_with_the_direct_judge` 同一条小流（段长只有 1、2 与一段 24 次写，取 3 展开的是短的那几段）。
const EXPAND_UP_TO: usize = 3;
/// 这条流切成几块待判行。
const PENDING_BLOCKS: u64 = 3;
/// 待判行的流键每个字节都是它。
const STREAM_TAG: u8 = 0x51;
/// 等取活循环把全部待判行判完写回的上限（编管线、判一条小流；推的，宽着给）。
const ALL_WRITTEN_BACK_WAIT: Duration = Duration::from_secs(1200);
const POLL: Duration = Duration::from_millis(100);
/// 认领租期：比这一趟长，不在判着的时候过期。
const CLAIM_LEASE_MILLISECONDS: u32 = 1_200_000;

/// 测试那一侧结束（含 panic）时叫取活循环停。
struct StopOnDrop(Arc<AtomicBool>);

impl Drop for StopOnDrop {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

#[test]
fn the_library_service_path_gives_the_verdicts_of_the_judge_called_directly() {
    const TEST_NAME: &str =
        "the_library_service_path_gives_the_verdicts_of_the_judge_called_directly";
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let packed = PackedJudgeTables::of(&new_pool_file_creation_judge_tables(EXPAND_UP_TO));
    let tables = match judge_tables_through_the_pairing_library(TEST_NAME, &packed) {
        StageOutcome::Judged(read_back) => read_back,
        StageOutcome::RecordedOnly | StageOutcome::JudgedMyShareOnly { .. } => {
            println!("JUDGE_TABLES_RECORDED_ONLY test={TEST_NAME}");
            return;
        }
    };
    let materials = Arc::new(GpuStreamMaterials {
        tables,
        kernel_sources: JudgeKernelSources::of_the_judge(),
    });
    let state_count = materials.tables.state_count();
    assert!(
        state_count >= PENDING_BLOCKS,
        "这条流至少 {PENDING_BLOCKS} 个态，才切得成几块"
    );
    let whole_stream = usize::try_from(state_count).expect("装得进 usize");
    let Some(lease) = cards.lease_card_or_report_not_run(
        TEST_NAME,
        mebibytes_a_judge_needs(&materials.tables, whole_stream),
    ) else {
        return;
    };
    let direct = GpuCrashJudge::on_card_compiling_these_kernel_sources(
        lease.card(),
        &materials.tables,
        whole_stream,
        &materials.kernel_sources,
    )
    .judge_states(0..state_count);

    let library = TemporaryLibraryService::start("library-service-path");
    let stream = materials.stream_key();
    let mut side = StandInVerificationPackageSide::on(&library);
    side.register(
        stream.tables_digest,
        stream,
        materials.stream_facts(),
        Arc::clone(&materials),
    );
    let conclusion_version = conclusion_version_of_the_crash_image_judgement();
    let block_states = state_count.div_ceil(PENDING_BLOCKS);
    let rows: Vec<(PendingBlockName, GpuPendingRow)> = (0..PENDING_BLOCKS)
        .map(|block| {
            let start = block * block_states;
            let end = (start + block_states).min(state_count);
            (
                pending_block_name(STREAM_TAG, block),
                pending_row(
                    stream.tables_digest,
                    start..end,
                    conclusion_version,
                    ForcedRejudging::NotForced,
                    block,
                ),
            )
        })
        .filter(|(_, row)| !row.state_ordinals.is_empty())
        .collect();
    let store = library.open_store();
    put_pending_rows(&store, &rows);

    let configuration = ServiceConfiguration {
        library_scan_interval: Duration::from_millis(20),
        claim_lease: Duration::from_millis(u64::from(CLAIM_LEASE_MILLISECONDS)),
        oldest_pending_wait_limit: ServiceConfiguration::of_the_repository()
            .oldest_pending_wait_limit,
    };
    let mut pool = LibraryPendingPool::new(
        side,
        library.open_client(),
        CLAIM_LEASE_MILLISECONDS,
        BreakSwitches::none(),
    );
    let lent_card = lease.card().clone();
    let stop = Arc::new(AtomicBool::new(false));
    let summary = std::thread::scope(|scope| {
        let stop_for_the_loop = Arc::clone(&stop);
        let serving = scope.spawn(move || {
            // 借来的那一张交给循环开一回：取活循环起来就开、一直持着，不再开第二回
            let mut lent = Some(OpenedGpuCards::<GpuStreamMaterials>::of_the_cards(vec![
                lent_card,
            ]));
            let mut open_the_cards = || {
                lent.take()
                    .ok_or_else(|| "借来的那张卡已经交给循环了".to_string())
            };
            serve_from_the_library(
                &configuration,
                &BreakSwitches::none(),
                &mut pool,
                &mut open_the_cards,
                &|| stop_for_the_loop.load(Ordering::SeqCst),
            )
        });
        let stop_on_drop = StopOnDrop(Arc::clone(&stop));
        let deadline = Instant::now() + ALL_WRITTEN_BACK_WAIT;
        while !pending_names(&store).is_empty() {
            assert!(
                Instant::now() < deadline && !serving.is_finished(),
                "{} 秒里待判行没判完写回（或取活循环先退了）",
                ALL_WRITTEN_BACK_WAIT.as_secs()
            );
            std::thread::sleep(POLL);
        }
        drop(stop_on_drop);
        serving.join().expect("取活循环没 panic")
    })
    .unwrap_or_else(|failure| panic!("取活循环没跑下去：{failure:?}"));
    assert_eq!(summary.card_openings, 1, "起来开一回卡，一直持着");
    assert_eq!(
        summary.blocks_written_back,
        u64::try_from(rows.len()).expect("装得进 u64")
    );
    assert_eq!(summary.blocks_refused_at_the_write, 0);
    let mut raw = library.open_client();
    let mut served: Vec<StateVerdict> = Vec::new();
    for (name, row) in &rows {
        served.extend(
            stand_in_verdicts(&mut raw, row).unwrap_or_else(|| panic!("{name} 没有写回的结论")),
        );
    }
    assert_eq!(served.len(), direct.len(), "两边判的态数相同");
    let differing: Vec<usize> = served
        .iter()
        .zip(&direct)
        .enumerate()
        .filter(|(_, (served_verdict, direct_verdict))| served_verdict != direct_verdict)
        .map(|(ordinal, _)| ordinal)
        .collect();
    assert!(
        differing.is_empty(),
        "{} 个态经判定库取活写回的与直接判的结论字不同（前几个序号 {:?}）",
        differing.len(),
        differing.iter().take(8).collect::<Vec<_>>()
    );
}
