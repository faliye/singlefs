//! 攻方腿探针（GPU 服务接住 OOM 与卡坏上报的治理文件轮第一轮，攻方腿的模型）：替身判器上量服务主循环接住 Out of Memory 与卡坏之后的几种历史，不上卡。
//! 每条打一行 `OOM_ATTACK_PROBE name=…`（读数），再断言攻方腿推的那一种结局：断言红就是攻方腿推错了，读数行照打。
//! 拷进仓副本的 `gpu-verification-server/tests/` 下跑（`mod common` 用那里的替身材料与卡、流的取法）。

mod common;

use std::collections::BTreeMap;
use std::ops::Range;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex};

use common::{
    card_leaving, configuration_with_the_limit, mebibytes_of_states, stand_in_facts_bits,
    stream_tagged, StandInFactsVerifierMaterial,
};
use gpu_verification_server::batch_scheduling::WavePlan;
use gpu_verification_server::break_switches::BreakSwitches;
use gpu_verification_server::card_memory_ledger::{CardIndex, CardOfTheService};
use gpu_verification_server::card_worker::{CardAfterAPanic, JudgedOrdinals, JudgesOnACard};
use gpu_verification_server::facts_verifier_family::FactsVerifierMaterial;
use gpu_verification_server::pending_pool::{InMemoryPendingPool, StateVerdict};
use gpu_verification_server::service_loop::{
    serve_the_cards, service_channel_telling_the_notices, CardWithItsJudges, FactsVerifierJob,
    ServiceClient, ServiceNotice, ServiceRunSummary,
};
use gpu_verification_server::stream_identity::{StreamFacts, StreamKey};
use singlefs_verification::crash_judge_gpu::VERDICT_WORDS;

/// 替身判器装一条流要的：这条流的标记（结论里带着它）。
struct ProbeMaterials {
    tag: u32,
}

/// 替身判出的结论：流的标记、序号的低 32 位与高 32 位。
fn probe_verdict(tag: u32, ordinal: u64) -> StateVerdict {
    let mut verdict = [0u32; VERDICT_WORDS];
    verdict[0] = tag;
    verdict[1] = u32::try_from(ordinal & 0xffff_ffff).expect("低 32 位");
    verdict[2] = u32::try_from(ordinal >> 32).expect("高 32 位");
    verdict
}

/// 替身在卡上做过的事（panic 的那一次不记）。
#[derive(Clone, Debug)]
enum ProbeEvent {
    Loaded { card: CardIndex, stream: StreamKey },
    Evicted { card: CardIndex, stream: StreamKey },
    Judged { card: CardIndex, stream: StreamKey, ordinals: Range<u64> },
    Verified { card: CardIndex, loaded: Vec<StreamKey>, blocks: Vec<Range<u64>> },
}

type ProbeLog = Arc<Mutex<Vec<ProbeEvent>>>;

/// 显存不够（之后丢掉卡上全部判器、答 RanOutOfMemory），还是判法自己的断言（设备没丢、没记过 Out of Memory：答 PanickedOtherwise，
/// 同真卡上 `GpuCrashJudge` 装表时「表超过绑定上限」那一句断言）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProbeFaultKind {
    OutOfMemory,
    JudgeAssertion,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProbeFaultTarget {
    Loading(StreamKey),
    Judging(StreamKey),
    Verifying,
}

#[derive(Clone, Debug)]
struct ProbeFault {
    card: CardIndex,
    target: ProbeFaultTarget,
    kind: ProbeFaultKind,
    times: usize,
}

type ProbeFaults = Arc<Mutex<Vec<ProbeFault>>>;

struct ProbeJudges {
    card: CardIndex,
    loaded: BTreeMap<StreamKey, (u32, u64)>,
    log: ProbeLog,
    faults: ProbeFaults,
    fault_of_the_panic: Option<ProbeFaultKind>,
}

impl ProbeJudges {
    fn fail_as_set(&mut self, target: ProbeFaultTarget) {
        let kind = {
            let mut faults = self.faults.lock().expect("没有线程拿着锁 panic");
            faults
                .iter_mut()
                .find(|fault| fault.card == self.card && fault.target == target && fault.times > 0)
                .map(|fault| {
                    fault.times -= 1;
                    fault.kind
                })
        };
        if let Some(kind) = kind {
            self.fault_of_the_panic = Some(kind);
            panic!("探针替身照设定在卡 {:?} 上出毛病：{kind:?}", self.card);
        }
    }

    fn note(&self, event: ProbeEvent) {
        self.log.lock().expect("没有线程拿着锁 panic").push(event);
    }
}

impl JudgesOnACard for ProbeJudges {
    type StreamMaterials = ProbeMaterials;

    fn load_the_judge(&mut self, stream: StreamKey, materials: &ProbeMaterials, states_per_dispatch: u64) {
        self.fail_as_set(ProbeFaultTarget::Loading(stream));
        assert!(
            self.loaded.insert(stream, (materials.tag, states_per_dispatch)).is_none(),
            "{stream:?} 在卡 {:?} 上装了两次",
            self.card
        );
        self.note(ProbeEvent::Loaded { card: self.card, stream });
    }

    fn evict_the_judge(&mut self, stream: StreamKey) {
        assert!(self.loaded.remove(&stream).is_some(), "要换出的 {stream:?} 不在卡 {:?} 上", self.card);
        self.note(ProbeEvent::Evicted { card: self.card, stream });
    }

    fn judge_ordinals(&mut self, stream: StreamKey, ordinals: Range<u64>) -> JudgedOrdinals {
        let (tag, states_per_dispatch) = *self
            .loaded
            .get(&stream)
            .unwrap_or_else(|| panic!("要判的 {stream:?} 不在卡 {:?} 上", self.card));
        assert!(ordinals.end - ordinals.start <= states_per_dispatch, "一段多于一次派活");
        self.fail_as_set(ProbeFaultTarget::Judging(stream));
        self.note(ProbeEvent::Judged { card: self.card, stream, ordinals: ordinals.clone() });
        JudgedOrdinals {
            verdicts: ordinals.map(|ordinal| probe_verdict(tag, ordinal)).collect(),
            dispatches: 1,
        }
    }

    fn verify_the_facts(&mut self, _material: &dyn FactsVerifierMaterial, blocks: &[Range<u64>]) -> Vec<Vec<u32>> {
        self.fail_as_set(ProbeFaultTarget::Verifying);
        self.note(ProbeEvent::Verified {
            card: self.card,
            loaded: self.loaded.keys().copied().collect(),
            blocks: blocks.to_vec(),
        });
        blocks.iter().map(|ordinals| stand_in_facts_bits(ordinals.clone())).collect()
    }

    fn after_a_panic(&mut self) -> CardAfterAPanic {
        match self.fault_of_the_panic.take() {
            Some(ProbeFaultKind::OutOfMemory) => {
                let judges_dropped = self.loaded.len();
                self.loaded.clear();
                CardAfterAPanic::RanOutOfMemory { judges_dropped }
            }
            Some(ProbeFaultKind::JudgeAssertion) | None => CardAfterAPanic::PanickedOtherwise,
        }
    }
}

/// 一次服务跑下来的：服务交回的账、每一波的计划、主循环交出的几件事、替身在卡上做过的事。
type Served = (ServiceRunSummary, Vec<WavePlan>, Vec<ServiceNotice>, Vec<ProbeEvent>);

/// 起服务（每张卡一个探针替身，照 `faults` 出毛病），客户端那一侧在另一条线程里做 `client_side`，做完叫服务判完就停。
fn serve_with_probe_judges(
    cards: Vec<CardOfTheService>,
    faults: &[ProbeFault],
    client_side: impl FnOnce(&ServiceClient<ProbeMaterials>) + Send,
) -> Served {
    let log = ProbeLog::default();
    let faults: ProbeFaults = Arc::new(Mutex::new(faults.to_vec()));
    let (client, inbox, notices) = service_channel_telling_the_notices::<ProbeMaterials>();
    let configuration = configuration_with_the_limit(20);
    let (summary, waves) = std::thread::scope(|scope| {
        let client_thread = scope.spawn(move || {
            let outcome = catch_unwind(AssertUnwindSafe(|| client_side(&client)));
            client.drain_and_stop();
            if let Err(panic) = outcome {
                resume_unwind(panic);
            }
        });
        let mut waves = Vec::new();
        let summary = serve_the_cards(
            &configuration,
            cards
                .into_iter()
                .map(|card| {
                    let index = card.card;
                    let log = Arc::clone(&log);
                    let faults = Arc::clone(&faults);
                    CardWithItsJudges {
                        card,
                        make_the_judges: move || ProbeJudges {
                            card: index,
                            loaded: BTreeMap::new(),
                            log,
                            faults,
                            fault_of_the_panic: None,
                        },
                    }
                })
                .collect(),
            InMemoryPendingPool::empty(BreakSwitches::of_the_environment()),
            inbox,
            &mut |plan: &WavePlan| waves.push(plan.clone()),
        );
        if let Err(panic) = client_thread.join() {
            resume_unwind(panic);
        }
        (summary, waves)
    });
    let events = log.lock().expect("服务停了，没有线程拿着锁").clone();
    (summary, waves, notices.try_iter().collect(), events)
}

fn register_probe_stream(client: &ServiceClient<ProbeMaterials>, tag: u8, table_mebibytes: u64, state_count: u64) -> StreamKey {
    let stream = stream_tagged(tag);
    client
        .register_stream(
            stream,
            StreamFacts { table_mebibytes, state_count },
            Arc::new(ProbeMaterials { tag: u32::from(tag) }),
        )
        .expect("登记得上");
    stream
}

fn blocks_put_back(notices: &[ServiceNotice]) -> usize {
    notices
        .iter()
        .map(|notice| match notice {
            ServiceNotice::OutOfMemoryCaught { blocks_put_back, .. } => blocks_put_back.len(),
            _ => 0,
        })
        .sum()
}

/// 第 6 问后半：一波里 40 块各 1 态，卡 0 判它们那一批时显存不够。推的：40 块全撤回、各占卡 0 单独一波（每波换出再装同一条流的判器），
/// 卡 1 一直空着、一块不帮判；每一块最后都判对、每态只判一次。
#[test]
fn probe_put_back_blocks_wait_one_by_one_for_the_card_that_ran_out_of_memory() {
    const BLOCKS: u64 = 40;
    let stream = stream_tagged(1);
    let capacity = 4 + mebibytes_of_states(64);
    let faults = [ProbeFault {
        card: CardIndex(0),
        target: ProbeFaultTarget::Judging(stream),
        kind: ProbeFaultKind::OutOfMemory,
        times: 1,
    }];
    let (summary, waves, notices, events) = serve_with_probe_judges(
        vec![card_leaving(0, capacity), card_leaving(1, capacity)],
        &faults,
        move |client| {
            register_probe_stream(client, 1, 2, 64);
            let answers: Vec<_> = (0..BLOCKS)
                .map(|ordinal| (ordinal, client.submit_block(stream, ordinal..ordinal + 1)))
                .collect();
            for (ordinal, answer) in answers {
                let judged = answer
                    .recv()
                    .expect("服务交回这一块")
                    .unwrap_or_else(|refusal| panic!("块 {ordinal} 被拒：{refusal:?}"));
                assert_eq!(judged.verdicts, vec![probe_verdict(1, ordinal)], "块 {ordinal} 的结论是它自己的");
            }
        },
    );
    let put_back = blocks_put_back(&notices);
    let lone_waves_after_the_first = waves
        .iter()
        .skip(1)
        .filter(|plan| {
            let busy: Vec<_> = plan.cards.iter().filter(|card| !card.batches.is_empty()).collect();
            busy.len() == 1
                && busy[0].card == CardIndex(0)
                && busy[0].loads.iter().map(|load| load.stream).collect::<Vec<_>>() == vec![stream]
        })
        .count();
    let loads_on_card0 = events
        .iter()
        .filter(|event| matches!(event, ProbeEvent::Loaded { card: CardIndex(0), .. }))
        .count();
    let evictions_on_card0 = events
        .iter()
        .filter(|event| matches!(event, ProbeEvent::Evicted { card: CardIndex(0), .. }))
        .count();
    let judged_runs_on_card1 = events
        .iter()
        .filter(|event| matches!(event, ProbeEvent::Judged { card: CardIndex(1), .. }))
        .count();
    let judged_states: usize = events
        .iter()
        .map(|event| match event {
            ProbeEvent::Judged { ordinals, .. } => usize::try_from(ordinals.end - ordinals.start).expect("装得进"),
            _ => 0,
        })
        .sum();
    println!(
        "OOM_ATTACK_PROBE name=put_back_blocks_serialized blocks={BLOCKS} out_of_memory_caught={} put_back={put_back} waves={} \
         lone_waves_after_the_first={lone_waves_after_the_first} loads_on_card0={loads_on_card0} evictions_on_card0={evictions_on_card0} \
         judged_runs_on_card1={judged_runs_on_card1} judged_states={judged_states} blocks_judged={}",
        summary.out_of_memory_caught,
        waves.len(),
        summary.blocks_judged
    );
    let put_back_ordinals: Vec<Range<u64>> = notices
        .iter()
        .flat_map(|notice| match notice {
            ServiceNotice::OutOfMemoryCaught { blocks_put_back, .. } => {
                blocks_put_back.iter().map(|(_, ordinals)| ordinals.clone()).collect::<Vec<_>>()
            }
            _ => Vec::new(),
        })
        .collect();
    let put_back_judged_on_card1 = events
        .iter()
        .filter(|event| match event {
            ProbeEvent::Judged { card: CardIndex(1), ordinals, .. } => {
                put_back_ordinals.iter().any(|put| put.start < ordinals.end && ordinals.start < put.end)
            }
            _ => false,
        })
        .count();
    println!("OOM_ATTACK_PROBE name=put_back_blocks_on_card1 put_back_judged_on_card1={put_back_judged_on_card1}");
    assert_eq!(summary.out_of_memory_caught, 1);
    assert!(put_back > 1, "取样：头一波卡 0 那一批要带不止一块（机器忙、头一波发早了的重跑）");
    assert_eq!(lone_waves_after_the_first, put_back, "推的：撤回的每一块各占卡 0 单独一波");
    assert_eq!(put_back_judged_on_card1, 0, "推的：撤回的块卡 1 一块都不帮判");
    assert_eq!(judged_states, usize::try_from(BLOCKS).expect("装得进"), "每态只判一次");
}

/// 第 6 问前半（好卡的驻留判器）：卡 0、卡 1 各驻留一条流的判器、都空着；一组核对块派到卡 0（旁边放得下、不换出），核的时候显存不够。
/// 推的：卡 0 的「空出来了」还没收到时主循环先给放回的那一组找卡，只剩卡 1 空着、又要腾空的卡，就把卡 1 驻留的判器全换出再核，
/// 虽然卡 1 不换出也借得到。
#[test]
fn probe_a_facts_job_put_back_after_running_out_of_memory_empties_the_other_card() {
    const TABLE: u64 = 40;
    const STATES: u64 = 10;
    let capacity = 2 * TABLE + mebibytes_of_states(STATES) + 3;
    let first = stream_tagged(1);
    let second = stream_tagged(2);
    let faults = [ProbeFault {
        card: CardIndex(0),
        target: ProbeFaultTarget::Verifying,
        kind: ProbeFaultKind::OutOfMemory,
        times: 1,
    }];
    let retried_on = Arc::new(Mutex::new(Vec::new()));
    let retried_on_for_the_client = Arc::clone(&retried_on);
    let (summary, _waves, _notices, events) = serve_with_probe_judges(
        vec![card_leaving(0, capacity), card_leaving(1, capacity)],
        &faults,
        move |client| {
            register_probe_stream(client, 1, TABLE, STATES);
            register_probe_stream(client, 2, TABLE, STATES);
            let answers = [client.submit_block(first, 0..STATES), client.submit_block(second, 0..STATES)];
            for answer in answers {
                answer.recv().expect("服务交回这一块").unwrap_or_else(|refusal| panic!("块被拒：{refusal:?}"));
            }
            let material: Arc<dyn FactsVerifierMaterial> = Arc::new(StandInFactsVerifierMaterial::empty());
            let verified = client
                .submit_facts_verifier_job(FactsVerifierJob { material, blocks: vec![0..3], verifier_mebibytes: 2 })
                .recv()
                .expect("服务交回这一组")
                .unwrap_or_else(|refusal| panic!("核对组被拒：{refusal:?}"));
            retried_on_for_the_client.lock().expect("没有线程拿着锁 panic").push(verified.card);
        },
    );
    let loaded: Vec<(u32, StreamKey)> = events
        .iter()
        .filter_map(|event| match event {
            ProbeEvent::Loaded { card, stream } => Some((card.0, *stream)),
            _ => None,
        })
        .collect();
    let verified: Vec<(u32, usize)> = events
        .iter()
        .filter_map(|event| match event {
            ProbeEvent::Verified { card, loaded, .. } => Some((card.0, loaded.len())),
            _ => None,
        })
        .collect();
    let evicted_on_card1 = events
        .iter()
        .filter(|event| matches!(event, ProbeEvent::Evicted { card: CardIndex(1), .. }))
        .count();
    let retried = retried_on.lock().expect("服务停了").clone();
    println!(
        "OOM_ATTACK_PROBE name=facts_job_retry_empties_the_other_card first_wave_loads={loaded:?} verified_card_and_judges_on_it={verified:?} \
         evicted_on_card1={evicted_on_card1} answered_card={retried:?} out_of_memory_caught={} facts_jobs_put_back={}",
        summary.out_of_memory_caught, summary.facts_jobs_put_back_after_running_out_of_memory
    );
    let mut loaded_cards: Vec<u32> = loaded.iter().map(|(card, _)| *card).collect();
    loaded_cards.sort_unstable();
    assert_eq!(loaded_cards, vec![0, 1], "取样：两条流头一波各装在一张卡上（机器忙、第二块晚到时这一条红，重跑，不算推翻）");
    assert_eq!(verified, vec![(1, 0)], "推的：卡 0 上那一回 panic 了不记；重核派到卡 1、核的时候卡 1 上一个判器都不剩");
    assert_eq!(evicted_on_card1, 1, "推的：卡 1 驻留的那一个判器被换出");
}

/// 第 8 问与第 6 问（判法自己 panic 连坏好卡）：一条流的判器一装就断言失败（同真卡上表超过绑定上限那一句），设备没丢、没记 Out of Memory。
/// 推的：卡 0 报卡坏；同一块再交一回（取活循环被拒的块放回库里、租期过后照认领）落到卡 1，卡 1 也报卡坏；之后服务一张卡都没有，
/// 好流登记不上、好块入不了池。
#[test]
fn probe_a_stream_whose_judge_panics_on_loading_fails_every_card_one_after_another() {
    let bad = stream_tagged(5);
    let capacity = 8 + mebibytes_of_states(10);
    let faults = [
        ProbeFault { card: CardIndex(0), target: ProbeFaultTarget::Loading(bad), kind: ProbeFaultKind::JudgeAssertion, times: usize::MAX },
        ProbeFault { card: CardIndex(1), target: ProbeFaultTarget::Loading(bad), kind: ProbeFaultKind::JudgeAssertion, times: usize::MAX },
    ];
    let answers = Arc::new(Mutex::new(Vec::new()));
    let answers_for_the_client = Arc::clone(&answers);
    let (summary, _waves, notices, _events) = serve_with_probe_judges(
        vec![card_leaving(0, capacity), card_leaving(1, capacity)],
        &faults,
        move |client| {
            register_probe_stream(client, 5, 2, 10);
            let mut said = Vec::new();
            for attempt in ["first", "claimed-again"] {
                let answer = client.submit_block(bad, 0..10).recv().expect("服务交回这一块");
                said.push(format!("{attempt}:{:?}", answer.map(|judged| judged.ordinals)));
            }
            let good = stream_tagged(6);
            let registered = client.register_stream(
                good,
                StreamFacts { table_mebibytes: 2, state_count: 10 },
                Arc::new(ProbeMaterials { tag: 6 }),
            );
            said.push(format!("good-registration:{registered:?}"));
            let good_block = client.submit_block(good, 0..10).recv().expect("服务交回这一块");
            said.push(format!("good-block:{:?}", good_block.map(|judged| judged.ordinals)));
            answers_for_the_client.lock().expect("没有线程拿着锁 panic").extend(said);
        },
    );
    let card_failures: Vec<(u32, bool)> = notices
        .iter()
        .filter_map(|notice| match notice {
            ServiceNotice::CardFailed { card, device_lost, .. } => Some((card.0, *device_lost)),
            _ => None,
        })
        .collect();
    let said = answers.lock().expect("服务停了").clone();
    println!(
        "OOM_ATTACK_PROBE name=judge_panic_fails_every_card cards_failed={:?} card_failed_notices={card_failures:?} answers={said:?}",
        summary.cards_failed
    );
    assert_eq!(card_failures, vec![(0, false), (1, false)], "推的：两张卡先后报卡坏、设备都没丢");
    assert!(said[0].starts_with("first:Err(CardFailed"), "推的：头一回卡 0 坏：{said:?}");
    assert!(said[1].starts_with("claimed-again:Err(CardFailed"), "推的：再交一回卡 1 坏：{said:?}");
}

/// 第 5 问（OOM 之后服务收到停止信号）：卡 0 判头一批时显存不够，紧接着客户端叫判完就停。推的（没打中的形状）：撤回的块照样单独判完，每块交回、每态只判一次。
#[test]
fn probe_out_of_memory_then_drain_and_stop_still_judges_every_block_once() {
    const BLOCKS: u64 = 6;
    let stream = stream_tagged(1);
    let capacity = 4 + mebibytes_of_states(64);
    let faults = [ProbeFault {
        card: CardIndex(0),
        target: ProbeFaultTarget::Judging(stream),
        kind: ProbeFaultKind::OutOfMemory,
        times: 1,
    }];
    let (summary, _waves, notices, events) = serve_with_probe_judges(
        vec![card_leaving(0, capacity), card_leaving(1, capacity)],
        &faults,
        move |client| {
            register_probe_stream(client, 1, 2, 64);
            let answers: Vec<_> = (0..BLOCKS)
                .map(|ordinal| (ordinal, client.submit_block(stream, ordinal..ordinal + 1)))
                .collect();
            client.drain_and_stop();
            for (ordinal, answer) in answers {
                let judged = answer
                    .recv()
                    .expect("服务交回这一块")
                    .unwrap_or_else(|refusal| panic!("块 {ordinal} 被拒：{refusal:?}"));
                assert_eq!(judged.verdicts, vec![probe_verdict(1, ordinal)]);
            }
        },
    );
    let mut counts: BTreeMap<u64, u32> = BTreeMap::new();
    for event in &events {
        if let ProbeEvent::Judged { ordinals, .. } = event {
            for ordinal in ordinals.clone() {
                *counts.entry(ordinal).or_default() += 1;
            }
        }
    }
    println!(
        "OOM_ATTACK_PROBE name=out_of_memory_then_drain blocks={BLOCKS} out_of_memory_caught={} put_back={} blocks_judged={} judged_counts={counts:?}",
        summary.out_of_memory_caught,
        blocks_put_back(&notices),
        summary.blocks_judged
    );
    assert_eq!(counts.len(), usize::try_from(BLOCKS).expect("装得进"));
    assert!(counts.values().all(|count| *count == 1), "每态只判一次");
}

/// 第 5、6 问（一块跨两张卡、卡 0 显存不够）：一块 30 态，头一波卡 0 判 0..10、卡 1 判 10..20。推的：整块撤回、卡 0 单独重判 0..30，
/// 卡 1 判过的 10..20 白判（结论不要），这一块交回的结论逐态对、没有错卡错流。
#[test]
fn probe_a_block_split_across_two_cards_is_put_back_whole_and_the_good_card_work_is_dropped() {
    let stream = stream_tagged(3);
    let capacity = 4 + mebibytes_of_states(10);
    let faults = [ProbeFault {
        card: CardIndex(0),
        target: ProbeFaultTarget::Judging(stream),
        kind: ProbeFaultKind::OutOfMemory,
        times: 1,
    }];
    let (summary, waves, _notices, events) = serve_with_probe_judges(
        vec![card_leaving(0, capacity), card_leaving(1, capacity)],
        &faults,
        move |client| {
            register_probe_stream(client, 3, 2, 30);
            let judged = client
                .submit_block(stream, 0..30)
                .recv()
                .expect("服务交回这一块")
                .unwrap_or_else(|refusal| panic!("块被拒：{refusal:?}"));
            let expected: Vec<StateVerdict> = (0..30).map(|ordinal| probe_verdict(3, ordinal)).collect();
            assert_eq!(judged.verdicts, expected, "结论逐态是它自己的");
            assert_eq!(judged.work_on_each_card.keys().copied().collect::<Vec<_>>(), vec![CardIndex(0)], "推的：交回的账只记卡 0");
        },
    );
    let runs: Vec<(u32, Range<u64>)> = events
        .iter()
        .filter_map(|event| match event {
            ProbeEvent::Judged { card, ordinals, .. } => Some((card.0, ordinals.clone())),
            _ => None,
        })
        .collect();
    let first_wave: Vec<(u32, u64)> = waves
        .first()
        .map(|plan| plan.cards.iter().map(|card| (card.card.0, card.batches.iter().map(|batch| batch.state_count()).sum())).collect())
        .unwrap_or_default();
    println!(
        "OOM_ATTACK_PROBE name=split_block_put_back first_wave_states_by_card={first_wave:?} judged_runs={runs:?} put_back={} blocks_judged={}",
        summary.blocks_put_back_after_running_out_of_memory, summary.blocks_judged
    );
    assert!(runs.iter().any(|(card, _)| *card == 1), "推的：卡 1 在头一波判过它那一截");
}
