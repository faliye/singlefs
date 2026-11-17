//! 对拍（要卡，经卡池借）：新池新建文件那条流同一段态，服务路径（池 → 调度 → 工作线程）判出的结论字与直接用 `GpuCrashJudge`
//! 判的逐态相同；两条流（同一条录制、同一个展开上限，窄的那条少展开一段）在同一张卡上交替派，与各自独占一个判器判的逐态相同。
//! 这台机器没有 NVIDIA 的卡、借不到卡时打一行「本次未跑」返回（`gpu_unit_checks::gpu_test_start`）；判表只录那一趟打
//! `JUDGE_TABLES_RECORDED_ONLY` 返回。判表先经对拍那一类的库入库、再读库，GPU 只拿读回的那一份算（项名是用例名）。

mod common;

use std::collections::BTreeMap;
use std::ops::Range;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::sync::Arc;

use common::{
    mebibytes_of_states, new_pool_file_creation_judge_tables,
    new_pool_file_creation_judge_tables_leaving_one_segment_whole,
};
use gpu_verification_server::batch_scheduling::{BatchSchedulingConfiguration, WavePlan};
use gpu_verification_server::break_switches::BreakSwitches;
use gpu_verification_server::card_memory_ledger::{CardIndex, CardOfTheService};
use gpu_verification_server::gpu_judges_on_a_card::{
    GpuJudgesOnACard, GpuStreamMaterials, JudgeMaterialsOnACard,
};
use gpu_verification_server::pending_pool::{InMemoryPendingPool, StateVerdict};
use gpu_verification_server::service_configuration::ServiceConfiguration;
use gpu_verification_server::service_loop::{serve_the_cards, service_channel, CardWithItsJudges};
use gpu_verification_server::stream_identity::StreamKey;
use singlefs_verification::crash_judge_dispatch::{
    mebibytes_a_judge_needs, GpuCrashJudge, PackedJudgeTables, MEBIBYTES_LEFT_FOR_THE_DRIVER,
};
use singlefs_verification::crash_judge_gpu::JudgeKernelSources;
use singlefs_verification::crash_judge_tables::JudgeTables;
use singlefs_verification::crash_library::{
    judge_tables_through_the_pairing_library, StageOutcome,
};
use singlefs_verification::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
use singlefs_verification::gpu_unit_checks::GpuCard;

/// 展开上限：段长不超过它的段逐个真子集展开。这条录制的段长只有 1、2 与一段 24 次写（验证包
/// `on_the_new_pool_file_creation_stream_the_hint_finds_what_the_full_ring_scan_finds_in_every_compared_state` 钉的段长），
/// 上限取 2 到 23 展开的都是同一批段、同一份表；取 0 只剩全部持久那一个态，切不成几批。
const EXPAND_UP_TO: usize = 3;
/// 窄的那条流不展开的那一段（段长 2，宽的那条展开它）：别的段照展开，态数仍够切几批，表与宽的那条不同。
const SEGMENT_LEFT_WHOLE_IN_THE_NARROWER_STREAM: usize = 0;

/// 判表经对拍那一类的库入库、读回，配上判定入库的那三份内核源码；只录那一趟交 `None`。
fn materials_through_the_pairing_library(
    item_name: &str,
    judge_tables: &JudgeTables,
) -> Option<GpuStreamMaterials> {
    let packed = PackedJudgeTables::of(judge_tables);
    match judge_tables_through_the_pairing_library(item_name, &packed) {
        StageOutcome::Judged(read_back) => Some(GpuStreamMaterials {
            tables: read_back,
            kernel_sources: JudgeKernelSources::of_the_judge(),
        }),
        StageOutcome::RecordedOnly | StageOutcome::JudgedMyShareOnly { .. } => {
            println!("JUDGE_TABLES_RECORDED_ONLY test={item_name}");
            None
        }
    }
}

/// 直接判：一个判器一次派活开到整条流，判全部态。
fn judged_directly(card: &GpuCard, materials: &GpuStreamMaterials) -> Vec<StateVerdict> {
    let state_count = materials.tables.state_count();
    let judge = GpuCrashJudge::on_card_compiling_these_kernel_sources(
        card,
        &materials.tables,
        usize::try_from(state_count).expect("装得进 usize"),
        &materials.kernel_sources,
    );
    judge.judge_states(0..state_count)
}

/// 经服务判：一张卡，给服务的额度是 `service_quota_mebibytes`；几条流先都登记，再按 `block_states` 一块切开、各流的块交错着入池，
/// 判完就停。交回各流按序号接起来的结论与发出的每一波。
fn judged_through_the_service(
    card: &GpuCard,
    service_quota_mebibytes: u64,
    streams: &[Arc<GpuStreamMaterials>],
    block_states: u64,
) -> (BTreeMap<StreamKey, Vec<StateVerdict>>, Vec<WavePlan>) {
    let configuration = BatchSchedulingConfiguration {
        oldest_pending_wait_limit: ServiceConfiguration::of_the_repository()
            .oldest_pending_wait_limit,
        break_switches: BreakSwitches::none(),
    };
    let (client, inbox) = service_channel::<GpuStreamMaterials>();
    let submit_and_collect = || {
        let mut keys = Vec::new();
        for materials in streams {
            let stream = materials.stream_key();
            client
                .register_stream(stream, materials.stream_facts(), Arc::clone(materials))
                .expect("登记得上");
            keys.push((stream, materials.tables.state_count()));
        }
        let longest = keys
            .iter()
            .map(|(_, state_count)| *state_count)
            .max()
            .unwrap_or(0);
        let mut answers: Vec<(StreamKey, Range<u64>, _)> = Vec::new();
        let mut start = 0;
        while start < longest {
            for (stream, state_count) in &keys {
                if start < *state_count {
                    let ordinals = start..(start + block_states).min(*state_count);
                    answers.push((
                        *stream,
                        ordinals.clone(),
                        client.submit_block(*stream, ordinals),
                    ));
                }
            }
            start += block_states;
        }
        let mut verdicts: BTreeMap<StreamKey, Vec<StateVerdict>> = BTreeMap::new();
        for (stream, ordinals, answer) in answers {
            let judged = answer
                .recv()
                .expect("服务交回这一块")
                .unwrap_or_else(|refusal| panic!("{stream:?} 的块 {ordinals:?} 被拒：{refusal:?}"));
            assert_eq!(judged.ordinals, ordinals);
            let collected = verdicts.entry(stream).or_default();
            assert_eq!(
                u64::try_from(collected.len()).expect("装得进 u64"),
                ordinals.start,
                "同一条流的块按序号先后收"
            );
            collected.extend(judged.verdicts);
        }
        verdicts
    };
    std::thread::scope(|scope| {
        // 客户端那一侧 panic 了也叫服务判完就停：不然主循环一直等，用例挂住
        let client_thread = scope.spawn(|| {
            let outcome = catch_unwind(AssertUnwindSafe(submit_and_collect));
            client.drain_and_stop();
            match outcome {
                Ok(verdicts) => verdicts,
                Err(panic) => resume_unwind(panic),
            }
        });
        let mut waves = Vec::new();
        let summary = serve_the_cards(
            &configuration,
            vec![CardWithItsJudges {
                card: CardOfTheService {
                    card: CardIndex(0),
                    service_quota_mebibytes,
                },
                make_the_judges: move || GpuJudgesOnACard::on_the_card(card),
            }],
            InMemoryPendingPool::empty(BreakSwitches::none()),
            inbox,
            &mut |plan: &WavePlan| waves.push(plan.clone()),
        );
        assert!(
            summary.cards_failed.is_empty(),
            "服务路径上卡坏了：{summary:?}"
        );
        match client_thread.join() {
            Ok(verdicts) => (verdicts, waves),
            Err(panic) => resume_unwind(panic),
        }
    })
}

fn assert_the_same_verdicts_state_by_state(
    label: &str,
    served: &[StateVerdict],
    direct: &[StateVerdict],
) {
    assert_eq!(served.len(), direct.len(), "{label}：两边判的态数相同");
    let differing: Vec<usize> = served
        .iter()
        .zip(direct)
        .enumerate()
        .filter(|(_, (served_verdict, direct_verdict))| served_verdict != direct_verdict)
        .map(|(ordinal, _)| ordinal)
        .collect();
    assert!(
        differing.is_empty(),
        "{label}：{} 个态服务路径与直接判的结论字不同（前几个序号 {:?}）",
        differing.len(),
        differing.iter().take(8).collect::<Vec<_>>()
    );
}

#[test]
fn the_service_path_gives_the_verdicts_of_the_judge_called_directly() {
    const TEST_NAME: &str = "the_service_path_gives_the_verdicts_of_the_judge_called_directly";
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some(materials) = materials_through_the_pairing_library(
        TEST_NAME,
        &new_pool_file_creation_judge_tables(EXPAND_UP_TO),
    ) else {
        return;
    };
    let state_count = materials.tables.state_count();
    assert!(state_count >= 3, "这条流至少三个态，服务路径才切得成几批");
    let whole_stream = usize::try_from(state_count).expect("装得进 usize");
    let Some(lease) = cards.lease_card_or_report_not_run(
        TEST_NAME,
        mebibytes_a_judge_needs(&materials.tables, whole_stream),
    ) else {
        return;
    };
    let direct = judged_directly(lease.card(), &materials);
    // 服务那一侧一次派活只给三分之一条流的显存：同一段态要切成几批、几波判
    let third = (state_count / 3).max(1);
    let service_quota =
        materials.tables.mebibytes() + mebibytes_of_states(third) + MEBIBYTES_LEFT_FOR_THE_DRIVER;
    let materials = Arc::new(materials);
    let (served, waves) = judged_through_the_service(
        lease.card(),
        service_quota,
        std::slice::from_ref(&materials),
        (third / 2).max(1),
    );
    let batches: usize = waves.iter().map(WavePlan::batch_count).sum();
    assert!(
        batches >= 3,
        "服务路径把 {state_count} 个态切成了 {batches} 批"
    );
    assert_the_same_verdicts_state_by_state("一条流", &served[&materials.stream_key()], &direct);
}

#[test]
fn two_streams_alternating_on_one_card_give_the_verdicts_of_each_stream_judged_alone() {
    const TEST_NAME: &str =
        "two_streams_alternating_on_one_card_give_the_verdicts_of_each_stream_judged_alone";
    let narrower_item = format!("{TEST_NAME}.narrower");
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some(wider) = materials_through_the_pairing_library(
        TEST_NAME,
        &new_pool_file_creation_judge_tables(EXPAND_UP_TO),
    ) else {
        return;
    };
    let Some(narrower) = materials_through_the_pairing_library(
        &narrower_item,
        &new_pool_file_creation_judge_tables_leaving_one_segment_whole(
            EXPAND_UP_TO,
            SEGMENT_LEFT_WHOLE_IN_THE_NARROWER_STREAM,
        ),
    ) else {
        return;
    };
    assert_ne!(
        wider.stream_key(),
        narrower.stream_key(),
        "少展开一段是另一条流"
    );
    let fewer_states = wider
        .tables
        .state_count()
        .min(narrower.tables.state_count());
    assert!(fewer_states >= 3, "两条流都至少三个态，才各切得成几批");
    let per_dispatch = (fewer_states / 3).max(1);
    let shared_card = wider.tables.mebibytes()
        + narrower.tables.mebibytes()
        + 2 * mebibytes_of_states(per_dispatch);
    let alone = |materials: &GpuStreamMaterials| {
        mebibytes_a_judge_needs(
            &materials.tables,
            usize::try_from(materials.tables.state_count()).expect("装得进 usize"),
        )
    };
    let need = shared_card.max(alone(&wider)).max(alone(&narrower));
    let Some(lease) = cards.lease_card_or_report_not_run(TEST_NAME, need) else {
        return;
    };
    let wider_alone = judged_directly(lease.card(), &wider);
    let narrower_alone = judged_directly(lease.card(), &narrower);
    let wider = Arc::new(wider);
    let narrower = Arc::new(narrower);
    let (served, waves) = judged_through_the_service(
        lease.card(),
        shared_card + MEBIBYTES_LEFT_FOR_THE_DRIVER,
        &[Arc::clone(&wider), Arc::clone(&narrower)],
        per_dispatch,
    );
    let sequence: Vec<StreamKey> = waves
        .iter()
        .flat_map(|wave| wave.cards.iter())
        .flat_map(|card| card.batches.iter().map(|batch| batch.stream))
        .collect();
    let last_wider = sequence
        .iter()
        .rposition(|stream| *stream == wider.stream_key())
        .expect("宽的那条流派过批");
    let first_narrower = sequence
        .iter()
        .position(|stream| *stream == narrower.stream_key())
        .expect("窄的那条流派过批");
    assert!(
        first_narrower < last_wider,
        "两条流交替派：窄的那条的头一批排在宽的那条最后一批之前（派的次序 {sequence:?}）"
    );
    assert!(
        waves.iter().any(|wave| {
            wave.cards.iter().any(|card| {
                card.batches
                    .iter()
                    .any(|batch| batch.stream == wider.stream_key())
                    && card
                        .batches
                        .iter()
                        .any(|batch| batch.stream == narrower.stream_key())
            })
        }),
        "至少有一波同一张卡上两条流各一批"
    );
    assert_the_same_verdicts_state_by_state(
        "宽的那条流",
        &served[&wider.stream_key()],
        &wider_alone,
    );
    assert_the_same_verdicts_state_by_state(
        "窄的那条流",
        &served[&narrower.stream_key()],
        &narrower_alone,
    );
}
