//! 服务主循环（池 → 调度 → 工作线程 → 池 → 客户端）在替身判器上的整条路，不上卡：替身按（流、序号）造结论，
//! 核每一块交回的结论逐态是它自己的、每个态只判一次、判器没装就判或装两次替身当场 panic；一张卡判的时候坏了，在判的块报拒、
//! 服务照样停得下来；登记与入池的几种拒。

mod common;

use std::collections::BTreeMap;
use std::ops::Range;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex};

use common::{card_leaving, configuration_with_the_limit, mebibytes_of_states, stream_tagged};
use gpu_verification_server::batch_scheduling::WavePlan;
use gpu_verification_server::break_switches::BreakSwitches;
use gpu_verification_server::card_memory_ledger::{CardIndex, CardOfTheService};
use gpu_verification_server::card_worker::JudgesOnACard;
use gpu_verification_server::pending_pool::{InMemoryPendingPool, StateVerdict};
use gpu_verification_server::service_loop::{
    serve_the_cards, service_channel, CardWithItsJudges, ServiceClient, ServiceRefusal,
    ServiceRunSummary,
};
use gpu_verification_server::stream_identity::{StreamFacts, StreamKey};
use singlefs_verification::crash_judge_gpu::VERDICT_WORDS;

/// 替身判器装一条流要的：这条流的标记（结论里带着它）。
struct StandInMaterials {
    tag: u32,
}

/// 替身判出的结论：流的标记、序号的低 32 位与高 32 位，其余字是 0。
fn stand_in_verdict(tag: u32, ordinal: u64) -> StateVerdict {
    let mut verdict = [0u32; VERDICT_WORDS];
    verdict[0] = tag;
    verdict[1] = u32::try_from(ordinal & 0xffff_ffff).expect("低 32 位");
    verdict[2] = u32::try_from(ordinal >> 32).expect("高 32 位");
    verdict
}

/// 替身判器判过的每一段：哪张卡、哪条流、哪几个态。
type JudgedRuns = Arc<Mutex<Vec<(CardIndex, StreamKey, Range<u64>)>>>;

struct StandInJudges {
    card: CardIndex,
    loaded: BTreeMap<StreamKey, (u32, u64)>,
    judged_runs: JudgedRuns,
    panics_on: Option<StreamKey>,
}

impl JudgesOnACard for StandInJudges {
    type StreamMaterials = StandInMaterials;

    fn load_the_judge(
        &mut self,
        stream: StreamKey,
        materials: &StandInMaterials,
        states_per_dispatch: u64,
    ) {
        assert!(
            self.loaded
                .insert(stream, (materials.tag, states_per_dispatch))
                .is_none(),
            "{stream:?} 在卡 {:?} 上装了两次",
            self.card
        );
    }

    fn evict_the_judge(&mut self, stream: StreamKey) {
        assert!(
            self.loaded.remove(&stream).is_some(),
            "要换出的 {stream:?} 不在卡 {:?} 上",
            self.card
        );
    }

    fn judge_ordinals(&mut self, stream: StreamKey, ordinals: Range<u64>) -> Vec<StateVerdict> {
        let (tag, states_per_dispatch) = *self
            .loaded
            .get(&stream)
            .unwrap_or_else(|| panic!("要判的 {stream:?} 不在卡 {:?} 上", self.card));
        assert!(
            ordinals.end - ordinals.start <= states_per_dispatch,
            "一段 {ordinals:?} 多于判器一次派活的 {states_per_dispatch} 个态"
        );
        assert!(self.panics_on != Some(stream), "替身在这条流上照设定坏掉");
        self.judged_runs
            .lock()
            .expect("没有线程拿着锁 panic")
            .push((self.card, stream, ordinals.clone()));
        ordinals
            .map(|ordinal| stand_in_verdict(tag, ordinal))
            .collect()
    }
}

/// 起服务（每张卡一个替身判器缓存），客户端那一侧在另一条线程里做 `client_side`，做完（panic 了也一样）叫服务判完就停。
fn serve_with_stand_in_judges(
    cards: Vec<CardOfTheService>,
    panics_on: Option<StreamKey>,
    judged_runs: &JudgedRuns,
    client_side: impl FnOnce(&ServiceClient<StandInMaterials>) + Send,
) -> (ServiceRunSummary, Vec<WavePlan>) {
    let (client, inbox) = service_channel::<StandInMaterials>();
    let configuration = configuration_with_the_limit(20);
    std::thread::scope(|scope| {
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
                .map(|card| CardWithItsJudges {
                    card,
                    make_the_judges: {
                        let judged_runs = Arc::clone(judged_runs);
                        move || StandInJudges {
                            card: card.card,
                            loaded: BTreeMap::new(),
                            judged_runs,
                            panics_on,
                        }
                    },
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
    })
}

fn register(
    client: &ServiceClient<StandInMaterials>,
    tag: u8,
    table_mebibytes: u64,
    state_count: u64,
) -> StreamKey {
    let stream = stream_tagged(tag);
    client
        .register_stream(
            stream,
            StreamFacts {
                table_mebibytes,
                state_count,
            },
            Arc::new(StandInMaterials {
                tag: u32::from(tag),
            }),
        )
        .expect("登记得上");
    stream
}

#[test]
fn every_block_comes_back_with_its_own_verdicts_and_every_state_is_judged_once() {
    let judged_runs = JudgedRuns::default();
    let cards = vec![
        card_leaving(0, 8 + mebibytes_of_states(10)),
        card_leaving(1, 8 + mebibytes_of_states(10)),
    ];
    let blocks_of_each_stream: [(u8, u64, Vec<Range<u64>>); 3] = [
        (1, 50, vec![0..10, 10..20, 20..30, 30..40, 40..50]),
        (2, 23, vec![0..13, 13..23]),
        (3, 7, std::iter::once(0..7).collect()),
    ];
    let (summary, waves) = serve_with_stand_in_judges(cards, None, &judged_runs, |client| {
        let mut answers = Vec::new();
        for (tag, state_count, blocks) in blocks_of_each_stream {
            let stream = register(client, tag, 2, state_count);
            for ordinals in blocks {
                answers.push((
                    tag,
                    ordinals.clone(),
                    client.submit_block(stream, ordinals.clone()),
                ));
            }
        }
        for (tag, ordinals, answer) in answers {
            let judged = answer
                .recv()
                .expect("服务交回这一块")
                .unwrap_or_else(|refusal| panic!("流 {tag} 的块 {ordinals:?} 被拒：{refusal:?}"));
            assert_eq!(judged.ordinals, ordinals);
            let expected: Vec<StateVerdict> = ordinals
                .clone()
                .map(|ordinal| stand_in_verdict(u32::from(tag), ordinal))
                .collect();
            assert_eq!(
                judged.verdicts, expected,
                "流 {tag} 的块 {ordinals:?} 交回的结论逐态是它自己的"
            );
        }
    });
    assert_eq!(summary.blocks_judged, 8);
    assert!(summary.cards_failed.is_empty());
    assert!(!waves.is_empty());
    let mut judged_states: BTreeMap<(StreamKey, u64), u32> = BTreeMap::new();
    for (_card, stream, ordinals) in judged_runs.lock().expect("服务停了，没有线程拿着锁").iter()
    {
        for ordinal in ordinals.clone() {
            *judged_states.entry((*stream, ordinal)).or_default() += 1;
        }
    }
    assert_eq!(judged_states.len(), 50 + 23 + 7, "每个态都判过");
    assert!(
        judged_states.values().all(|count| *count == 1),
        "每个态只判一次"
    );
}

#[test]
fn a_card_failing_while_judging_refuses_its_blocks_and_the_service_still_stops() {
    let judged_runs = JudgedRuns::default();
    let breaking = stream_tagged(9);
    let cards = vec![card_leaving(0, 8 + mebibytes_of_states(10))];
    let (summary, _waves) =
        serve_with_stand_in_judges(cards, Some(breaking), &judged_runs, |client| {
            let stream = register(client, 9, 2, 20);
            let answer = client
                .submit_block(stream, 0..20)
                .recv()
                .expect("服务交回这一块");
            assert!(
                matches!(
                    answer,
                    Err(ServiceRefusal::CardFailed {
                        card: CardIndex(0),
                        ..
                    })
                ),
                "卡坏了，在判的块报拒：{answer:?}"
            );
            let after = client
                .submit_block(stream, 0..5)
                .recv()
                .expect("服务交回这一块");
            assert_eq!(
                after,
                Err(ServiceRefusal::NoWorkingCard),
                "卡都坏了，新块当场拒"
            );
        });
    assert_eq!(summary.cards_failed, vec![CardIndex(0)]);
    assert_eq!(summary.blocks_judged, 0);
}

#[test]
fn registration_and_entry_refuse_what_no_card_can_judge() {
    let judged_runs = JudgedRuns::default();
    let cards = vec![card_leaving(0, 8 + mebibytes_of_states(10))];
    let (summary, _waves) = serve_with_stand_in_judges(cards, None, &judged_runs, |client| {
        let too_large = stream_tagged(5);
        let refusal = client.register_stream(
            too_large,
            StreamFacts {
                table_mebibytes: 100_000,
                state_count: 10,
            },
            Arc::new(StandInMaterials { tag: 5 }),
        );
        assert!(
            matches!(refusal, Err(ServiceRefusal::StreamFitsNoCard { .. })),
            "表装不进任何一张卡：{refusal:?}"
        );
        let unregistered = client
            .submit_block(stream_tagged(6), 0..3)
            .recv()
            .expect("服务回话");
        assert!(matches!(
            unregistered,
            Err(ServiceRefusal::StreamNotRegistered { .. })
        ));
        let stream = register(client, 7, 2, 10);
        let past_the_end = client.submit_block(stream, 5..11).recv().expect("服务回话");
        assert!(matches!(
            past_the_end,
            Err(ServiceRefusal::OrdinalsOutsideTheStream { .. })
        ));
        let other_facts = client.register_stream(
            stream,
            StreamFacts {
                table_mebibytes: 2,
                state_count: 11,
            },
            Arc::new(StandInMaterials { tag: 7 }),
        );
        assert!(matches!(
            other_facts,
            Err(ServiceRefusal::StreamRegisteredWithOtherFacts { .. })
        ));
    });
    assert_eq!(summary.blocks_judged, 0);
    assert!(judged_runs.lock().expect("服务停了").is_empty());
}
