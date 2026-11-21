//! 对拍（要卡，经卡池借）：新池新建文件那条流的判表照判表待判行那一族的写法（验证包 `put_the_packed_tables_pending_row_unless_present_or_raise_it_to_forced`，
//! 表 `20`）写成一行判表待判行，放进测试进程里起的临时判定库服务（不写真服务）；取活循环（`serving_from_the_library`）起来就开卡（借来的那一张）、
//! 取走、登记流、把 `0..态数` 当一整块判、带守卫写回表 `21`、删掉判表待判行，读回的结论字与同一张判表直接用
//! `GpuCrashJudge::on_card_compiling_these_kernel_sources` 判的逐态、逐字相同。验证包那一侧只有流是替身（交的就是读回的那一份判表与
//! `JudgeKernelSources::of_the_judge()`，行里的旋钮栏是产品那一份），扫、重读、结论字在不在、编结论字那一帧都照验证包那一份。
//! 这台机器没有 NVIDIA 的卡、借不到卡时打一行「本次未跑」返回（`gpu_unit_checks::gpu_test_start`）；判表只录那一趟打
//! `JUDGE_TABLES_RECORDED_ONLY` 返回。判表先经对拍那一类的库入库、再读库（项名是用例名）。

mod common;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use common::{
    new_pool_file_creation_judge_tables, packed_tables_pending_names,
    packed_tables_pending_on_the_stream, packed_tables_verdict_words_in_the_library,
    put_packed_tables_pending_rows, StandInVerificationPackageSide, TemporaryLibraryService,
};
use gpu_verification_server::break_switches::BreakSwitches;
use gpu_verification_server::gpu_judges_on_a_card::{
    GpuStreamMaterials, JudgeMaterialsOnACard, OpenedGpuCards,
};
use gpu_verification_server::library_pending_pool::LibraryPendingPool;
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
use singlefs_verification::verdict_store::ForcedRejudging;

/// 展开上限：与 `library_service_path_agrees_with_the_direct_judge` 同一条小流。
const EXPAND_UP_TO: usize = 3;
/// 等取活循环把那一张判表判完写回的上限（编管线、判一条小流；推的，宽着给）。
const WRITTEN_BACK_WAIT: Duration = Duration::from_secs(1200);
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
fn the_packed_tables_service_path_gives_the_verdict_words_of_the_judge_called_directly() {
    const TEST_NAME: &str =
        "the_packed_tables_service_path_gives_the_verdict_words_of_the_judge_called_directly";
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

    let library = TemporaryLibraryService::start("packed-tables-service-path");
    let stream = materials.stream_key();
    let mut side = StandInVerificationPackageSide::on(&library);
    side.register(
        stream.tables_digest,
        stream,
        materials.stream_facts(),
        Arc::clone(&materials),
    );
    let table =
        packed_tables_pending_on_the_stream(stream, state_count, ForcedRejudging::NotForced, 1);
    let store = library.open_store();
    put_packed_tables_pending_rows(&store, std::slice::from_ref(&table));

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
        let deadline = Instant::now() + WRITTEN_BACK_WAIT;
        while !packed_tables_pending_names(&store).is_empty() {
            assert!(
                Instant::now() < deadline && !serving.is_finished(),
                "{} 秒里判表待判行没判完写回（或取活循环先退了）",
                WRITTEN_BACK_WAIT.as_secs()
            );
            std::thread::sleep(POLL);
        }
        drop(stop_on_drop);
        serving.join().expect("取活循环没 panic")
    })
    .unwrap_or_else(|failure| panic!("取活循环没跑下去：{failure:?}"));
    assert_eq!(summary.card_openings, 1, "起来开一回卡，一直持着");
    assert_eq!(summary.packed_tables_written_back, 1);
    assert_eq!(summary.packed_tables_refused_at_the_write, 0);
    let served = packed_tables_verdict_words_in_the_library(&store, &table.0)
        .unwrap_or_else(|| panic!("{} 没有写回的结论字", table.0));
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
        "{} 个态经判表待判行写回的与直接判的结论字不同（前几个序号 {:?}）",
        differing.len(),
        differing.iter().take(8).collect::<Vec<_>>()
    );
}
