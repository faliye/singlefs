//! 对拍（要卡，经卡池借）：新池新建文件那条流的事实表（第 ② 段核对读的那一份；经对拍那一类的库入库、再读库，项名是用例名）切成几块，
//! 照判段的写法（验证包 `put_the_facts_verifier_pending_row_unless_present`，表 `1F`）写成核对待判行，放进测试进程里起的临时判定库服务
//! （不写真服务）；取活循环（`serving_from_the_library`）起来就开卡（借来的那一张）、取走、照显存账借给核对器、在卡上核、带守卫写回核对行、
//! 删掉核对待判行，读回的结论位逐块接起来，与直接用 `GpuFactsVerifier` 核的逐态相同，也与 CPU 参照
//! （`crash_facts::verify_states_from_facts_by_the_segment_expansions`）逐态相同。验证包那一侧的核对材料换成替身（交的就是这一份读回的事实表与拆法，
//! 不经录下的流程读回；扫、核过没有、编那一帧照验证包那一份）：这一条核的是取活、认领、借显存、上卡、写回整条路不改结论位；
//! 材料怎么读回由验证包那一侧的测试管。这台机器没有 NVIDIA 的卡、借不到卡时打一行「本次未跑」返回（`gpu_unit_checks::gpu_test_start`）；
//! 事实表只录那一趟打 `FACTS_RECORDED_ONLY` 返回。

mod common;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use common::{
    facts_verifier_block, facts_verifier_pending_names, new_pool_file_creation_facts,
    put_facts_verifier_pending_rows, verifier_row_in_the_library, StandInVerificationPackageSide,
    TemporaryLibraryService,
};
use gpu_verification_server::break_switches::BreakSwitches;
use gpu_verification_server::card_memory_ledger::mebibytes_of_a_facts_verifier;
use gpu_verification_server::facts_verifier_family::{
    FactsVerifierBlockName, FactsVerifierMaterial,
};
use gpu_verification_server::gpu_judges_on_a_card::{GpuStreamMaterials, OpenedGpuCards};
use gpu_verification_server::library_pending_pool::LibraryPendingPool;
use gpu_verification_server::service_configuration::ServiceConfiguration;
use gpu_verification_server::serving_from_the_library::serve_from_the_library;
use singlefs_verification::crash_facts::{
    verify_states_from_facts_by_the_segment_expansions, FlowFactsReadBack, SegmentExpansionFact,
};
use singlefs_verification::crash_library::{facts_through_the_pairing_library, StageOutcome};
use singlefs_verification::crash_verify_gpu::GpuFactsVerifier;
use singlefs_verification::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
use singlefs_verification::verdict_store::FactsVerifierPendingRow;

/// 展开上限：与 `library_service_path_agrees_with_the_direct_judge` 同一条小流（段长只有 1、2 与一段 24 次写，取 3 展开的是短的那几段）。
const EXPAND_UP_TO: usize = 3;
/// 这条流切成几块核对待判行。
const PENDING_BLOCKS: u64 = 3;
/// 核对待判行的事实表摘要与块键每个字节都是它（替身材料不照摘要读回）。
const FACTS_TAG: u8 = 0x52;
/// 等取活循环把全部核对待判行核完写回的上限（编核对管线、核一条小流；推的，宽着给）。
const ALL_WRITTEN_BACK_WAIT: Duration = Duration::from_secs(1200);
const POLL: Duration = Duration::from_millis(100);
/// 认领租期：比这一趟长，不在核着的时候过期。
const CLAIM_LEASE_MILLISECONDS: u32 = 1_200_000;

/// 读回的事实表与每段的拆法：服务照它在卡上建核对器。
struct ReadBackFactsMaterial {
    facts: FlowFactsReadBack,
    segment_expansions: Vec<SegmentExpansionFact>,
}

impl FactsVerifierMaterial for ReadBackFactsMaterial {
    fn facts(&self) -> &FlowFactsReadBack {
        &self.facts
    }

    fn segment_expansions(&self) -> &[SegmentExpansionFact] {
        &self.segment_expansions
    }
}

/// 测试那一侧结束（含 panic）时叫取活循环停。
struct StopOnDrop(Arc<AtomicBool>);

impl Drop for StopOnDrop {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

#[test]
fn the_facts_verifier_service_path_gives_the_bits_of_the_verifier_called_directly() {
    const TEST_NAME: &str =
        "the_facts_verifier_service_path_gives_the_bits_of_the_verifier_called_directly";
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let (facts, segment_expansions) = new_pool_file_creation_facts(EXPAND_UP_TO);
    let facts = match facts_through_the_pairing_library(TEST_NAME, &facts) {
        StageOutcome::Judged(read_back) => read_back,
        StageOutcome::RecordedOnly | StageOutcome::JudgedMyShareOnly { .. } => {
            println!("FACTS_RECORDED_ONLY test={TEST_NAME}");
            return;
        }
    };
    let state_count = facts.state_count;
    assert!(
        state_count >= PENDING_BLOCKS,
        "这条流至少 {PENDING_BLOCKS} 个态，才切得成几块"
    );
    let Some(lease) = cards.lease_card_or_report_not_run(
        TEST_NAME,
        mebibytes_of_a_facts_verifier(&facts, state_count),
    ) else {
        return;
    };
    let direct =
        GpuFactsVerifier::new_by_the_segment_expansions(lease.card(), &facts, &segment_expansions)
            .verify_states(0..state_count);
    let reference = verify_states_from_facts_by_the_segment_expansions(
        &facts,
        &segment_expansions,
        0..state_count,
    );
    assert_eq!(
        direct, reference,
        "直接用 GpuFactsVerifier 核的与 CPU 参照逐态相同（对拍的前提）"
    );

    let library = TemporaryLibraryService::start("facts-verifier-service-path");
    let mut side = StandInVerificationPackageSide::<GpuStreamMaterials>::on(&library);
    let material: Arc<dyn FactsVerifierMaterial> = Arc::new(ReadBackFactsMaterial {
        facts,
        segment_expansions,
    });
    side.register_facts_material(material);
    let block_states = state_count.div_ceil(PENDING_BLOCKS);
    let rows: Vec<(FactsVerifierBlockName, FactsVerifierPendingRow)> = (0..PENDING_BLOCKS)
        .map(|block| {
            let start = block * block_states;
            let end = (start + block_states).min(state_count);
            facts_verifier_block(FACTS_TAG, block, start..end, block)
        })
        .filter(|(_, row)| !row.state_ordinals.is_empty())
        .collect();
    let store = library.open_store();
    put_facts_verifier_pending_rows(&store, &rows);

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
        while !facts_verifier_pending_names(&store).is_empty() {
            assert!(
                Instant::now() < deadline && !serving.is_finished(),
                "{} 秒里核对待判行没核完写回（或取活循环先退了）",
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
        summary.facts_blocks_written_back,
        u64::try_from(rows.len()).expect("装得进 u64")
    );
    assert_eq!(summary.facts_blocks_refused_at_the_write, 0);
    let mut served: Vec<u32> = Vec::new();
    for (name, row) in &rows {
        served.extend(
            verifier_row_in_the_library(&store, row)
                .unwrap_or_else(|| panic!("{name} 没有写回的核对行"))
                .bits,
        );
    }
    assert_eq!(served.len(), direct.len(), "两边核的态数相同");
    let differing: Vec<usize> = served
        .iter()
        .zip(&direct)
        .enumerate()
        .filter(|(_, (served_bits, direct_bits))| served_bits != direct_bits)
        .map(|(ordinal, _)| ordinal)
        .collect();
    assert!(
        differing.is_empty(),
        "{} 个态经判定库取活写回的与直接核的结论位不同（前几个序号 {:?}）",
        differing.len(),
        differing.iter().take(8).collect::<Vec<_>>()
    );
}
