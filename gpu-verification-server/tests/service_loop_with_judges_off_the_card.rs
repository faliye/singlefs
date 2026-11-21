//! 服务主循环（池 → 调度 → 工作线程 → 池 → 客户端）在替身判器上的整条路，不上卡：替身按（流、序号）造结论，
//! 核每一块交回的结论逐态是它自己的、每个态只判一次、判器没装就判或装两次替身当场 panic；一张卡判的时候坏了，在判的块报拒、
//! 服务照样停得下来；登记与入池的几种拒；一组核对块照卡上的显存账借（放得下不换出、放不下先换出驻留的判器、哪张卡都放不下当场拒）。
//! 接住 Out of Memory 与判法自己 panic（`src/service_loop.rs` 文件头）：替身照设定（[`StandInFault`]：哪张卡、装哪条流、判哪条流或核的时候、几回、
//! 显存不够、设备丢了、设备因显存不够丢了还是判法断言失败）在那一步 panic，之后照设定答设备怎样（丢了设备之外的先丢掉卡上全部判器）；
//! 替身在卡上做过的几样（装上、等那一次提交做完）记进 [`CardEvents`]；主循环交出的几件事（[`ServiceNotice`]）收下来逐件核，
//! 打成的行照 `serving_from_the_library::service_notice_line` 核行首与字段。真卡那一份判器缓存认设备怎样的那一步
//! （`gpu_judges_on_a_card::device_after_a_panic`）在假的设备情况（验证包 `DeviceConditionOfACard::default`）上核。

mod common;

use std::collections::BTreeMap;
use std::ops::Range;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use common::{
    card_leaving, configuration_with_the_limit, mebibytes_of_states, stand_in_facts_bits,
    stream_tagged, StandInFactsVerifierMaterial,
};
use gpu_verification_server::batch_scheduling::WavePlan;
use gpu_verification_server::break_switches::BreakSwitches;
use gpu_verification_server::card_memory_ledger::{
    mebibytes_of_a_judge, CardIndex, CardOfTheService,
};
use gpu_verification_server::card_worker::{CardAfterAPanic, JudgedOrdinals, JudgesOnACard};
use gpu_verification_server::facts_verifier_family::FactsVerifierMaterial;
use gpu_verification_server::gpu_judges_on_a_card::{device_after_a_panic, DeviceAfterAPanic};
use gpu_verification_server::pending_pool::{InMemoryPendingPool, StateVerdict};
use gpu_verification_server::service_log::is_a_dispatch_line;
use gpu_verification_server::service_loop::{
    serve_the_cards, service_channel_telling_the_notices, CardWithItsJudges, FactsVerifierJob,
    ServiceClient, ServiceNotice, ServiceRefusal, ServiceRunSummary,
};
use gpu_verification_server::serving_from_the_library::{
    service_notice_line, CARD_FAILED_LINE_PREFIX, JUDGE_LOADED_LINE_PREFIX,
    OUT_OF_MEMORY_CAUGHT_LINE_PREFIX,
};
use gpu_verification_server::stream_identity::{StreamFacts, StreamKey};
use singlefs_verification::crash_judge_gpu::VERDICT_WORDS;
use singlefs_verification::gpu_memory_ledger::{CauseOfTheDeviceLoss, DeviceConditionOfACard};

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

/// 替身核对器每核一组记一笔：哪张卡、核的那一刻卡上装着的判器（流、一次派活的态数）、核的几块。
type VerifiedFactsJobs = Arc<Mutex<Vec<(CardIndex, Vec<(StreamKey, u64)>, Vec<Range<u64>>)>>>;

/// 替身照设定出的毛病是哪一种。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StandInFaultKind {
    /// 显存不够、设备没丢（之后丢掉卡上全部判器，答 `RanOutOfMemory`）。
    OutOfMemory,
    /// 设备为别的原因丢了（答 `DeviceLost`）。
    DeviceLost,
    /// 设备因显存不够丢了（之后丢掉卡上全部判器、当重开了设备，答 `DeviceReopenedAfterRunningOutOfMemory`）。
    DeviceLostForWantOfMemory,
    /// 判法自己的断言失败（同真卡上装表时「表超过绑定上限」那一句；之后丢掉卡上全部判器，答 `PanickedOtherwise`）。
    JudgeAssertion,
}

/// 毛病出在装哪条流的判器、判哪条流的时候，还是核的时候。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StandInFaultTarget {
    Loading(StreamKey),
    Judging(StreamKey),
    Verifying,
}

/// 替身在一张卡上做过的一样（panic 的那一步不记）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CardEvent {
    /// 装上了这条流的判器。
    Loaded(StreamKey),
    /// 照设定出了毛病（panic 之前记）。
    FaultRaised,
    /// 接住 Out of Memory 之后做了那一次提交、等它做完（`settle_after_running_out_of_memory`）。
    Settled,
}

/// 各卡替身共用的一份事件记录：（卡、事件），按发生的先后。
type CardEvents = Arc<Mutex<Vec<(CardIndex, CardEvent)>>>;

/// 替身照设定出的一种毛病：哪张卡、出在哪、还出几回。
#[derive(Clone, Debug)]
struct StandInFault {
    card: CardIndex,
    target: StandInFaultTarget,
    kind: StandInFaultKind,
    times: usize,
}

/// 各卡替身共用的一份毛病设定。
type StandInFaults = Arc<Mutex<Vec<StandInFault>>>;

fn faults_of(faults: &[StandInFault]) -> StandInFaults {
    Arc::new(Mutex::new(faults.to_vec()))
}

/// 替身卡的 PCI 总线号。
fn stand_in_pci_bus_identifier(card: CardIndex) -> String {
    format!("0000:{:02x}:00.0", card.0)
}

struct StandInJudges {
    card: CardIndex,
    loaded: BTreeMap<StreamKey, (u32, u64)>,
    judged_runs: JudgedRuns,
    verified_facts_jobs: VerifiedFactsJobs,
    card_events: CardEvents,
    faults: StandInFaults,
    /// 上一回 panic 是照设定出的哪一种毛病（`after_a_panic` 照它答）。
    fault_of_the_panic: Option<StandInFaultKind>,
}

impl StandInJudges {
    /// 照设定出毛病：这张卡在 `target` 上还有没出完的，记下、panic。
    fn fail_as_set(&mut self, target: StandInFaultTarget) {
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
            self.note(CardEvent::FaultRaised);
            panic!("替身照设定在卡 {:?} 上出毛病：{kind:?}", self.card);
        }
    }

    fn note(&self, event: CardEvent) {
        self.card_events
            .lock()
            .expect("没有线程拿着锁 panic")
            .push((self.card, event));
    }
}

impl JudgesOnACard for StandInJudges {
    type StreamMaterials = StandInMaterials;

    fn load_the_judge(
        &mut self,
        stream: StreamKey,
        materials: &StandInMaterials,
        states_per_dispatch: u64,
    ) {
        self.fail_as_set(StandInFaultTarget::Loading(stream));
        assert!(
            self.loaded
                .insert(stream, (materials.tag, states_per_dispatch))
                .is_none(),
            "{stream:?} 在卡 {:?} 上装了两次",
            self.card
        );
        self.note(CardEvent::Loaded(stream));
    }

    fn evict_the_judge(&mut self, stream: StreamKey) {
        assert!(
            self.loaded.remove(&stream).is_some(),
            "要换出的 {stream:?} 不在卡 {:?} 上",
            self.card
        );
    }

    fn judge_ordinals(&mut self, stream: StreamKey, ordinals: Range<u64>) -> JudgedOrdinals {
        let (tag, states_per_dispatch) = *self
            .loaded
            .get(&stream)
            .unwrap_or_else(|| panic!("要判的 {stream:?} 不在卡 {:?} 上", self.card));
        assert!(
            ordinals.end - ordinals.start <= states_per_dispatch,
            "一段 {ordinals:?} 多于判器一次派活的 {states_per_dispatch} 个态"
        );
        self.fail_as_set(StandInFaultTarget::Judging(stream));
        self.judged_runs
            .lock()
            .expect("没有线程拿着锁 panic")
            .push((self.card, stream, ordinals.clone()));
        JudgedOrdinals {
            verdicts: ordinals
                .map(|ordinal| stand_in_verdict(tag, ordinal))
                .collect(),
            dispatches: 1,
        }
    }

    /// 记下这一刻卡上装着的判器与核的几块，结论位照序号出（`common::stand_in_facts_bits`）。
    fn verify_the_facts(
        &mut self,
        _material: &dyn FactsVerifierMaterial,
        blocks: &[Range<u64>],
    ) -> Vec<Vec<u32>> {
        self.fail_as_set(StandInFaultTarget::Verifying);
        self.verified_facts_jobs
            .lock()
            .expect("没有线程拿着锁 panic")
            .push((
                self.card,
                self.loaded
                    .iter()
                    .map(|(stream, (_, states_per_dispatch))| (*stream, *states_per_dispatch))
                    .collect(),
                blocks.to_vec(),
            ));
        blocks
            .iter()
            .map(|ordinals| stand_in_facts_bits(ordinals.clone()))
            .collect()
    }

    /// 照设定出的毛病答：设备为别的原因丢了的照实答，别的先丢掉全部判器；没照设定出毛病的 panic（判器没装就判、装两次这一类）当判法自己的 panic。
    fn after_a_panic(&mut self) -> CardAfterAPanic {
        let judges_dropped = self.loaded.len();
        let fault = self.fault_of_the_panic.take();
        if fault == Some(StandInFaultKind::DeviceLost) {
            return CardAfterAPanic::DeviceLost {
                reason: "替身照设定丢了设备".to_string(),
            };
        }
        self.loaded.clear();
        match fault {
            Some(StandInFaultKind::OutOfMemory) => {
                CardAfterAPanic::RanOutOfMemory { judges_dropped }
            }
            Some(StandInFaultKind::DeviceLostForWantOfMemory) => {
                CardAfterAPanic::DeviceReopenedAfterRunningOutOfMemory {
                    judges_dropped,
                    reason: STAND_IN_REOPENED_REASON.to_string(),
                }
            }
            Some(StandInFaultKind::JudgeAssertion | StandInFaultKind::DeviceLost) | None => {
                CardAfterAPanic::PanickedOtherwise { judges_dropped }
            }
        }
    }

    /// 记一笔「等那一次提交做完了」。
    fn settle_after_running_out_of_memory(&mut self) -> Result<(), String> {
        self.note(CardEvent::Settled);
        Ok(())
    }

    fn pci_bus_identifier(&self) -> Option<String> {
        Some(stand_in_pci_bus_identifier(self.card))
    }
}

/// 替身答「设备因显存不够丢了、已重开」时带的那一句。
const STAND_IN_REOPENED_REASON: &str = "替身照设定：设备因显存不够丢了，已重开这张卡的设备";

/// 一次服务跑下来的：服务交回的账、每一波的计划、主循环交出的几件事（按交出的先后）。
type ServedWithStandIns = (ServiceRunSummary, Vec<WavePlan>, Vec<ServiceNotice>);

/// 起服务（每张卡一个替身判器缓存，照 `faults` 出毛病），客户端那一侧在另一条线程里做 `client_side`，做完（panic 了也一样）叫服务判完就停。
fn serve_with_stand_in_judges(
    cards: Vec<CardOfTheService>,
    card_events: &CardEvents,
    faults: &StandInFaults,
    judged_runs: &JudgedRuns,
    verified_facts_jobs: &VerifiedFactsJobs,
    client_side: impl FnOnce(&ServiceClient<StandInMaterials>) + Send,
) -> ServedWithStandIns {
    let (client, inbox, notices) = service_channel_telling_the_notices::<StandInMaterials>();
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
                        let verified_facts_jobs = Arc::clone(verified_facts_jobs);
                        let faults = Arc::clone(faults);
                        let card_events = Arc::clone(card_events);
                        move || StandInJudges {
                            card: card.card,
                            loaded: BTreeMap::new(),
                            judged_runs,
                            verified_facts_jobs,
                            card_events,
                            faults,
                            fault_of_the_panic: None,
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
        (summary, waves, notices.try_iter().collect())
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
    let (summary, waves, _notices) = serve_with_stand_in_judges(
        cards,
        &CardEvents::default(),
        &StandInFaults::default(),
        &judged_runs,
        &VerifiedFactsJobs::default(),
        |client| {
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
                    .unwrap_or_else(|refusal| {
                        panic!("流 {tag} 的块 {ordinals:?} 被拒：{refusal:?}")
                    });
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
        },
    );
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
    let faults = faults_of(&[StandInFault {
        card: CardIndex(0),
        target: StandInFaultTarget::Judging(breaking),
        kind: StandInFaultKind::DeviceLost,
        times: usize::MAX,
    }]);
    let (summary, _waves, _notices) = serve_with_stand_in_judges(
        cards,
        &CardEvents::default(),
        &faults,
        &judged_runs,
        &VerifiedFactsJobs::default(),
        |client| {
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
        },
    );
    assert_eq!(summary.cards_failed, vec![CardIndex(0)]);
    assert_eq!(summary.blocks_judged, 0);
}

#[test]
fn registration_and_entry_refuse_what_no_card_can_judge() {
    let judged_runs = JudgedRuns::default();
    let cards = vec![card_leaving(0, 8 + mebibytes_of_states(10))];
    let (summary, _waves, _notices) = serve_with_stand_in_judges(
        cards,
        &CardEvents::default(),
        &StandInFaults::default(),
        &judged_runs,
        &VerifiedFactsJobs::default(),
        |client| {
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
        },
    );
    assert_eq!(summary.blocks_judged, 0);
    assert!(judged_runs.lock().expect("服务停了").is_empty());
}

/// 一组核对块照卡上的显存账借（`src/service_loop.rs` 文件头「核对」）：一张卡上驻留着判过一块的判器，判器之外放得下的那一组不换出、照派；
/// 放不下的那一组先换出驻留的判器再派；核的那一刻卡上装着的判器占的加这一组借的不超过这张卡给判器的额度；比哪一张卡给判器的都大的当场拒。
/// 每组交回的结论位逐块是替身核对器照序号出的。
/// 弄坏开关：facts-verifier-memory-unchecked（不照账腾地方，放不下也照派）。
#[test]
fn a_facts_verifier_job_borrows_card_memory_and_evicts_resident_judges_only_as_needed() {
    const TABLE_MEBIBYTES: u64 = 2;
    const STATES: u64 = 10;
    /// 判器装满这条流的态之后这张卡还空着的。
    const FREE_BESIDE_THE_JUDGE: u64 = 3;
    let capacity = TABLE_MEBIBYTES + mebibytes_of_states(STATES) + FREE_BESIDE_THE_JUDGE;
    let fits_beside_the_judge = FREE_BESIDE_THE_JUDGE - 1;
    let needs_the_judge_evicted = capacity - 1;
    let judged_runs = JudgedRuns::default();
    let verified_facts_jobs = VerifiedFactsJobs::default();
    let jobs: [(u64, Vec<Range<u64>>); 2] = [
        (fits_beside_the_judge, vec![0..3, 3..5]),
        (needs_the_judge_evicted, std::iter::once(5..9).collect()),
    ];
    let jobs_for_the_client = jobs.clone();
    let (summary, _waves, _notices) = serve_with_stand_in_judges(
        vec![card_leaving(0, capacity)],
        &CardEvents::default(),
        &StandInFaults::default(),
        &judged_runs,
        &verified_facts_jobs,
        move |client| {
            let stream = register(client, 4, TABLE_MEBIBYTES, STATES);
            client
                .submit_block(stream, 0..STATES)
                .recv()
                .expect("服务交回这一块")
                .unwrap_or_else(|refusal| panic!("判器那一块被拒：{refusal:?}"));
            for (verifier_mebibytes, blocks) in jobs_for_the_client {
                let material: Arc<dyn FactsVerifierMaterial> =
                    Arc::new(StandInFactsVerifierMaterial::empty());
                let verified = client
                    .submit_facts_verifier_job(FactsVerifierJob {
                        material,
                        blocks: blocks.clone(),
                        verifier_mebibytes,
                    })
                    .recv()
                    .expect("服务交回这一组")
                    .unwrap_or_else(|refusal| {
                        panic!("借 {verifier_mebibytes} MiB 的那一组被拒：{refusal:?}")
                    });
                assert_eq!(verified.card, CardIndex(0));
                let expected: Vec<Vec<u32>> = blocks
                    .iter()
                    .map(|ordinals| stand_in_facts_bits(ordinals.clone()))
                    .collect();
                assert_eq!(
                    verified.bits_of_each_block, expected,
                    "交回的结论位逐块是替身核对器照序号出的"
                );
                assert_eq!(verified.card_nanoseconds_of_each_block.len(), blocks.len());
            }
            let material: Arc<dyn FactsVerifierMaterial> =
                Arc::new(StandInFactsVerifierMaterial::empty());
            let too_large = client
                .submit_facts_verifier_job(FactsVerifierJob {
                    material,
                    blocks: std::iter::once(0..1).collect(),
                    verifier_mebibytes: capacity + 1,
                })
                .recv()
                .expect("服务回话");
            assert!(
                matches!(
                    too_large,
                    Err(ServiceRefusal::FactsVerifierFitsNoCard { .. })
                ),
                "比这张卡给判器的还大，当场拒：{too_large:?}"
            );
        },
    );
    assert_eq!(summary.facts_verifier_jobs_verified, 2);
    let verified = verified_facts_jobs
        .lock()
        .expect("服务停了，没有线程拿着锁")
        .clone();
    assert_eq!(verified.len(), 2, "两组各核一回：{verified:?}");
    let facts = StreamFacts {
        table_mebibytes: TABLE_MEBIBYTES,
        state_count: STATES,
    };
    for ((_card, loaded, blocks), (verifier_mebibytes, expected_blocks)) in
        verified.iter().zip(&jobs)
    {
        assert_eq!(blocks, expected_blocks);
        let loaded_mebibytes: u64 = loaded
            .iter()
            .map(|(_, states_per_dispatch)| mebibytes_of_a_judge(&facts, *states_per_dispatch))
            .sum();
        assert!(
            loaded_mebibytes + verifier_mebibytes <= capacity,
            "核的那一刻卡上判器占 {loaded_mebibytes} MiB、核对器借 {verifier_mebibytes} MiB，超过这张卡给判器的 {capacity} MiB"
        );
    }
    assert!(
        !verified[0].1.is_empty(),
        "判器之外放得下的那一组不换出：核的那一刻判器还装着"
    );
    assert!(
        verified[1].1.is_empty(),
        "放不下的那一组先换出驻留的判器：{:?}",
        verified[1].1
    );
}

/// 交了这些块，等每一块的回话：都要判完、结论逐态是替身照它自己的流与序号出的。
fn every_block_judged(
    client: &ServiceClient<StandInMaterials>,
    blocks: Vec<(u8, StreamKey, Range<u64>)>,
) {
    let answers: Vec<_> = blocks
        .into_iter()
        .map(|(tag, stream, ordinals)| {
            let answer = client.submit_block(stream, ordinals.clone());
            (tag, ordinals, answer)
        })
        .collect();
    for (tag, ordinals, answer) in answers {
        let judged = answer
            .recv()
            .expect("服务交回这一块")
            .unwrap_or_else(|refusal| panic!("流 {tag} 的块 {ordinals:?} 被拒：{refusal:?}"));
        let expected: Vec<StateVerdict> = ordinals
            .clone()
            .map(|ordinal| stand_in_verdict(u32::from(tag), ordinal))
            .collect();
        assert_eq!(
            judged.verdicts, expected,
            "流 {tag} 的块 {ordinals:?} 交回的结论逐态是它自己的"
        );
    }
}

/// 替身判过的每个态各判了几次（panic 的那一段不记）。
fn judged_state_counts(judged_runs: &JudgedRuns) -> BTreeMap<(StreamKey, u64), u32> {
    let mut counts = BTreeMap::new();
    for (_card, stream, ordinals) in judged_runs.lock().expect("服务停了，没有线程拿着锁").iter()
    {
        for ordinal in ordinals.clone() {
            *counts.entry((*stream, ordinal)).or_default() += 1;
        }
    }
    counts
}

fn out_of_memory_notices(notices: &[ServiceNotice]) -> Vec<&ServiceNotice> {
    notices
        .iter()
        .filter(|notice| matches!(notice, ServiceNotice::OutOfMemoryCaught { .. }))
        .collect()
}

/// 撤回、报拒的块在行里的名字：判表摘要头的第一个字节与序号。
fn stand_in_block_name(stream: StreamKey, ordinals: &Range<u64>) -> String {
    format!(
        "{:02x}:{}..{}",
        stream.tables_digest[0], ordinals.start, ordinals.end
    )
}

/// 一张卡判的时候显存不够、设备没丢（`src/service_loop.rs` 文件头「接住 Out of Memory」）：这张卡不坏，它那一波在判的块撤回、单独在腾空的
/// 这张卡上重判（另一张卡装不下流 1），下一波照常收活；另一张卡照常判自己的块；每一块都判完交回、结论逐态是它自己的，每个态只判一次
/// （显存不够的那一段不算）；没有判法 panic；恰好交出一件「接住 Out of Memory」，打成的那一行行首与字段齐、过服务日志的过滤。
/// 一个场景：两张卡（卡 0 装得下流 1 的表两份，卡 1 装不下）、两条流的块按次序入池，卡 0 头一回判流 1 时显存不够。
/// 弄坏开关：out-of-memory-taken-for-a-card-failure（当卡坏）、residents-kept-after-running-out-of-memory（账上不清驻留的判器：
/// 重判那一波换出卡上已没有的判器、替身 panic）、blocks-of-the-out-of-memory-wave-refused（在判的块报拒、不撤回）、
/// out-of-memory-caught-not-told（不交那一件）。
#[test]
fn a_card_that_ran_out_of_memory_takes_the_next_wave_and_every_block_comes_back_judged_once() {
    const LARGE_TABLE_MEBIBYTES: u64 = 40;
    let judged_runs = JudgedRuns::default();
    let cards = vec![
        card_leaving(0, 2 * LARGE_TABLE_MEBIBYTES + mebibytes_of_states(10)),
        card_leaving(1, 8 + mebibytes_of_states(10)),
    ];
    let faults = faults_of(&[StandInFault {
        card: CardIndex(0),
        target: StandInFaultTarget::Judging(stream_tagged(1)),
        kind: StandInFaultKind::OutOfMemory,
        times: 1,
    }]);
    let (summary, _waves, notices) = serve_with_stand_in_judges(
        cards,
        &CardEvents::default(),
        &faults,
        &judged_runs,
        &VerifiedFactsJobs::default(),
        |client| {
            let first = register(client, 1, LARGE_TABLE_MEBIBYTES, 40);
            let second = register(client, 2, 2, 20);
            every_block_judged(
                client,
                vec![
                    (1, first, 0..10),
                    (1, first, 10..20),
                    (1, first, 20..30),
                    (1, first, 30..40),
                    (2, second, 0..10),
                    (2, second, 10..20),
                ],
            );
        },
    );
    assert!(
        summary.cards_failed.is_empty(),
        "显存不够不是卡坏：{summary:?}"
    );
    assert_eq!(summary.out_of_memory_caught, 1);
    assert_eq!(
        summary.judge_panics_caught, 0,
        "没有判法 panic（重判没撞上卡上已丢的判器）：{summary:?}"
    );
    assert!(summary.blocks_put_back_after_running_out_of_memory >= 1);
    assert_eq!(summary.blocks_refused_on_an_emptied_card, 0);
    let counts = judged_state_counts(&judged_runs);
    assert_eq!(counts.len(), 40 + 20, "每个态都判过");
    assert!(counts.values().all(|count| *count == 1), "每个态只判一次");
    assert!(
        judged_runs
            .lock()
            .expect("服务停了")
            .iter()
            .any(|(card, _, _)| *card == CardIndex(0)),
        "卡 0 显存不够之后照常收活、判完（显存不够的那一段 panic 了、不记）"
    );
    let caught = out_of_memory_notices(&notices);
    assert_eq!(
        caught.len(),
        1,
        "恰好交出一件接住 Out of Memory：{notices:?}"
    );
    let ServiceNotice::OutOfMemoryCaught {
        card,
        judges_dropped,
        blocks_put_back,
        blocks_refused,
        ..
    } = caught[0]
    else {
        unreachable!("上面只挑了这一种");
    };
    assert_eq!(*card, CardIndex(0));
    assert!(*judges_dropped >= 1, "卡 0 上至少装着流 1 的判器");
    assert!(!blocks_put_back.is_empty() && blocks_refused.is_empty());
    let line = service_notice_line(caught[0], None, &stand_in_block_name);
    assert!(
        line.starts_with(&format!("{OUT_OF_MEMORY_CAUGHT_LINE_PREFIX} card=0 "))
            && line.contains(&format!("judges_dropped={judges_dropped} "))
            && line.contains(&format!("blocks_put_back={} ", blocks_put_back.len()))
            && line.contains("blocks_refused=0 ")
            && line.contains("put_back=01:")
            && line.contains("reason=\""),
        "接住 Out of Memory 那一行：{line}"
    );
    assert!(
        !is_a_dispatch_line(line.as_bytes()),
        "那一行不被服务日志滤掉"
    );
    assert!(
        notices
            .iter()
            .all(|notice| !matches!(notice, ServiceNotice::CardFailed { .. })),
        "没有卡坏那一件：{notices:?}"
    );
}

/// 撤回的块单独在一张腾空的卡上又显存不够（文件头「接住 Out of Memory」）：那一块报拒（在腾空的卡上也显存不够），卡不坏、照常收下一块；
/// 第二回判它的那一波只装它那一条流的判器；两回各交一件「接住 Out of Memory」，头一回撤回、第二回报拒。
/// 一个场景：一张卡，流 3 的那一块显存不够两回，之后交流 4 的一块。
/// 弄坏开关：retracted-block-shares-the-next-wave（撤回的块照常交调度）、out-of-memory-on-an-emptied-card-retried（单独判又不够也撤回）。
#[test]
fn a_block_running_out_of_memory_again_alone_on_an_emptied_card_is_refused_and_the_card_takes_the_next_block(
) {
    let judged_runs = JudgedRuns::default();
    let alone = stream_tagged(3);
    let faults = faults_of(&[StandInFault {
        card: CardIndex(0),
        target: StandInFaultTarget::Judging(alone),
        kind: StandInFaultKind::OutOfMemory,
        times: 2,
    }]);
    let (summary, waves, notices) = serve_with_stand_in_judges(
        vec![card_leaving(0, 8 + mebibytes_of_states(10))],
        &CardEvents::default(),
        &faults,
        &judged_runs,
        &VerifiedFactsJobs::default(),
        |client| {
            let stream = register(client, 3, 2, 10);
            let answer = client
                .submit_block(stream, 0..10)
                .recv()
                .expect("服务交回这一块");
            assert_eq!(
                answer.map(|judged| judged.ordinals),
                Err(ServiceRefusal::OutOfMemoryOnAnEmptiedCard {
                    card: CardIndex(0),
                    reason: "替身照设定在卡 CardIndex(0) 上出毛病：OutOfMemory".to_string(),
                }),
                "单独在腾空的卡上又显存不够：报拒"
            );
            let next = register(client, 4, 2, 5);
            every_block_judged(client, vec![(4, next, 0..5)]);
        },
    );
    assert!(summary.cards_failed.is_empty(), "卡照常用：{summary:?}");
    assert_eq!(summary.out_of_memory_caught, 2);
    assert_eq!(summary.blocks_refused_on_an_emptied_card, 1);
    let caught = out_of_memory_notices(&notices);
    let put_back_and_refused: Vec<(usize, usize)> = caught
        .iter()
        .map(|notice| match notice {
            ServiceNotice::OutOfMemoryCaught {
                blocks_put_back,
                blocks_refused,
                ..
            } => (blocks_put_back.len(), blocks_refused.len()),
            ServiceNotice::JudgeLoaded { .. } | ServiceNotice::CardFailed { .. } => (0, 0),
        })
        .collect();
    assert_eq!(
        put_back_and_refused,
        vec![(1, 0), (0, 1)],
        "头一回撤回、第二回报拒：{notices:?}"
    );
    let waves_of_the_block: Vec<&WavePlan> = waves
        .iter()
        .filter(|plan| {
            plan.cards
                .iter()
                .any(|card| card.batches.iter().any(|batch| batch.stream == alone))
        })
        .collect();
    assert_eq!(waves_of_the_block.len(), 2, "那一块派了两波：{waves:?}");
    let lone = &waves_of_the_block[1].cards[0];
    assert_eq!(
        lone.resident_after_the_wave()
            .judges()
            .iter()
            .map(|judge| judge.stream)
            .collect::<Vec<_>>(),
        vec![alone],
        "第二回那一波腾空了卡、只装它那一条流的判器"
    );
    let states: u64 = lone.batches.iter().map(|batch| batch.state_count()).sum();
    assert_eq!(states, 10, "第二回那一波把那一块的态都带上");
}

/// 判的时候设备丢了：这张卡照旧坏掉、在判的块报拒（卡坏），交出恰好一件「卡坏」（设备丢了、PCI 总线号、原因），打成的那一行行首与字段齐、
/// 过服务日志的过滤；没有「接住 Out of Memory」那一件。
/// 弄坏开关：card-failure-not-told（不交那一件）。
#[test]
fn a_card_whose_device_is_lost_while_judging_still_fails_and_tells_one_card_failure() {
    let judged_runs = JudgedRuns::default();
    let lost_on = stream_tagged(9);
    let faults = faults_of(&[StandInFault {
        card: CardIndex(0),
        target: StandInFaultTarget::Judging(lost_on),
        kind: StandInFaultKind::DeviceLost,
        times: 1,
    }]);
    let (summary, _waves, notices) = serve_with_stand_in_judges(
        vec![card_leaving(0, 8 + mebibytes_of_states(10))],
        &CardEvents::default(),
        &faults,
        &judged_runs,
        &VerifiedFactsJobs::default(),
        |client| {
            let stream = register(client, 9, 2, 10);
            let answer = client
                .submit_block(stream, 0..10)
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
                "设备丢了，在判的块报拒：{answer:?}"
            );
        },
    );
    assert_eq!(summary.cards_failed, vec![CardIndex(0)]);
    assert_eq!(summary.out_of_memory_caught, 0);
    let failed: Vec<&ServiceNotice> = notices
        .iter()
        .filter(|notice| matches!(notice, ServiceNotice::CardFailed { .. }))
        .collect();
    assert_eq!(failed.len(), 1, "恰好交出一件卡坏：{notices:?}");
    assert!(out_of_memory_notices(&notices).is_empty());
    let line = service_notice_line(failed[0], None, &stand_in_block_name);
    assert!(
        line.starts_with(&format!("{CARD_FAILED_LINE_PREFIX} card=0 "))
            && line.contains(&format!(
                "pci={} ",
                stand_in_pci_bus_identifier(CardIndex(0))
            ))
            && line.contains("device_lost=yes ")
            && line.contains("nvidia_smi_index=unknown ")
            && line.contains("reason=\"")
            && line.contains("替身照设定丢了设备"),
        "卡坏那一行：{line}"
    );
    assert!(
        !is_a_dispatch_line(line.as_bytes()),
        "那一行不被服务日志滤掉"
    );
}

/// 核的时候显存不够、设备没丢（文件头「接住 Out of Memory」）：那一组核对块放回排头，下一回先腾空那张卡再核，核完交回；卡不坏。
/// 一个场景：一张卡上驻留着判过一块的判器，一组放得下、不换出的核对块头一回核时显存不够。
/// 弄坏开关：facts-job-of-the-out-of-memory-round-refused（那一组报拒）。
#[test]
fn a_facts_verifier_job_that_ran_out_of_memory_is_verified_again_on_an_emptied_card() {
    const TABLE_MEBIBYTES: u64 = 2;
    const STATES: u64 = 10;
    const FREE_BESIDE_THE_JUDGE: u64 = 3;
    let capacity = TABLE_MEBIBYTES + mebibytes_of_states(STATES) + FREE_BESIDE_THE_JUDGE;
    let judged_runs = JudgedRuns::default();
    let verified_facts_jobs = VerifiedFactsJobs::default();
    let faults = faults_of(&[StandInFault {
        card: CardIndex(0),
        target: StandInFaultTarget::Verifying,
        kind: StandInFaultKind::OutOfMemory,
        times: 1,
    }]);
    let blocks = vec![0..3, 3..5];
    let blocks_for_the_client = blocks.clone();
    let (summary, _waves, notices) = serve_with_stand_in_judges(
        vec![card_leaving(0, capacity)],
        &CardEvents::default(),
        &faults,
        &judged_runs,
        &verified_facts_jobs,
        move |client| {
            let stream = register(client, 4, TABLE_MEBIBYTES, STATES);
            every_block_judged(client, vec![(4, stream, 0..STATES)]);
            let material: Arc<dyn FactsVerifierMaterial> =
                Arc::new(StandInFactsVerifierMaterial::empty());
            let verified = client
                .submit_facts_verifier_job(FactsVerifierJob {
                    material,
                    blocks: blocks_for_the_client.clone(),
                    verifier_mebibytes: FREE_BESIDE_THE_JUDGE - 1,
                })
                .recv()
                .expect("服务交回这一组")
                .unwrap_or_else(|refusal| panic!("显存不够之后那一组被拒：{refusal:?}"));
            let expected: Vec<Vec<u32>> = blocks_for_the_client
                .iter()
                .map(|ordinals| stand_in_facts_bits(ordinals.clone()))
                .collect();
            assert_eq!(verified.bits_of_each_block, expected);
        },
    );
    assert!(summary.cards_failed.is_empty(), "卡照常用：{summary:?}");
    assert_eq!(summary.facts_jobs_put_back_after_running_out_of_memory, 1);
    let verified = verified_facts_jobs.lock().expect("服务停了").clone();
    assert_eq!(
        verified.len(),
        1,
        "显存不够的那一回 panic 了、不记，再核一回：{verified:?}"
    );
    assert!(
        verified[0].1.is_empty(),
        "再核的那一回卡上不留判器：{:?}",
        verified[0].1
    );
    assert_eq!(verified[0].2, blocks);
    let caught = out_of_memory_notices(&notices);
    assert!(
        matches!(
            caught.as_slice(),
            [ServiceNotice::OutOfMemoryCaught {
                facts_jobs_put_back: 1,
                facts_jobs_refused: 0,
                ..
            }]
        ),
        "恰好一件接住、放回那一组：{notices:?}"
    );
}

/// 每装上一个判器交一件「装上」（`src/service_loop.rs` 文件头）：表、草稿区、一次派活的态数、此刻驻留几个、账上还剩多少，与计划与显存账逐个相同；
/// 打成的行恰好一行、字段齐、过服务日志的过滤。
/// 一个场景：一张卡、一条流的三块，判器装一回、之后驻留。
/// 弄坏开关：judge-loaded-not-told（不交那一件）。
#[test]
fn every_judge_loaded_tells_one_line_with_its_table_scratch_and_what_the_card_has_left() {
    let judged_runs = JudgedRuns::default();
    let capacity = 8 + mebibytes_of_states(10);
    let (_summary, waves, notices) = serve_with_stand_in_judges(
        vec![card_leaving(0, capacity)],
        &CardEvents::default(),
        &StandInFaults::default(),
        &judged_runs,
        &VerifiedFactsJobs::default(),
        |client| {
            let stream = register(client, 5, 2, 30);
            every_block_judged(
                client,
                vec![(5, stream, 0..10), (5, stream, 10..20), (5, stream, 20..30)],
            );
        },
    );
    let loads: Vec<_> = waves
        .iter()
        .flat_map(|plan| plan.cards.iter())
        .flat_map(|card| card.loads.iter())
        .collect();
    assert_eq!(loads.len(), 1, "判器装一回、之后驻留：{waves:?}");
    let lines: Vec<String> = notices
        .iter()
        .filter(|notice| matches!(notice, ServiceNotice::JudgeLoaded { .. }))
        .map(|notice| service_notice_line(notice, None, &stand_in_block_name))
        .collect();
    assert_eq!(lines.len(), 1, "恰好一行：{notices:?}");
    let facts = StreamFacts {
        table_mebibytes: 2,
        state_count: 30,
    };
    let judge = mebibytes_of_a_judge(&facts, loads[0].states_per_dispatch);
    let expected = format!(
        "{JUDGE_LOADED_LINE_PREFIX} card=0 stream=0505050505050505 table_mebibytes=2 scratch_mebibytes={} \
         states_per_dispatch={} judges_on_the_card=1 free_mebibytes={}",
        judge - 2,
        loads[0].states_per_dispatch,
        capacity - judge
    );
    assert_eq!(lines[0], expected);
    assert!(
        !is_a_dispatch_line(lines[0].as_bytes()),
        "那一行不被服务日志滤掉"
    );
}

/// 这张卡上替身记下的事件（按发生的先后）。
fn events_on(card_events: &CardEvents, card: CardIndex) -> Vec<CardEvent> {
    card_events
        .lock()
        .expect("服务停了，没有线程拿着锁")
        .iter()
        .filter(|(on, _)| *on == card)
        .map(|(_, event)| *event)
        .collect()
}

/// 接住设备没丢的 Out of Memory 之后（`src/card_worker.rs` 文件头「接住 Out of Memory」）：工作线程先在这张卡上做一次提交、等它做完，
/// 才报接住、收下一波，所以单独重判那一波装判器之前恰好等过一次；那一块照样判完交回。
/// 一个场景：一张卡，流 3 的一块头一回判时显存不够。
/// 弄坏开关：card-not-settled-after-running-out-of-memory（不做那一次提交、不等）。
#[test]
fn a_card_that_ran_out_of_memory_waits_for_one_submission_to_finish_before_loading_again() {
    let judged_runs = JudgedRuns::default();
    let card_events = CardEvents::default();
    let stream = stream_tagged(3);
    let faults = faults_of(&[StandInFault {
        card: CardIndex(0),
        target: StandInFaultTarget::Judging(stream),
        kind: StandInFaultKind::OutOfMemory,
        times: 1,
    }]);
    let (summary, _waves, _notices) = serve_with_stand_in_judges(
        vec![card_leaving(0, 8 + mebibytes_of_states(10))],
        &card_events,
        &faults,
        &judged_runs,
        &VerifiedFactsJobs::default(),
        |client| {
            let registered = register(client, 3, 2, 10);
            every_block_judged(client, vec![(3, registered, 0..10)]);
        },
    );
    assert_eq!(summary.out_of_memory_caught, 1, "{summary:?}");
    let events = events_on(&card_events, CardIndex(0));
    let raised = events
        .iter()
        .position(|event| *event == CardEvent::FaultRaised)
        .unwrap_or_else(|| panic!("替身照设定出过一回毛病：{events:?}"));
    let after_the_fault = &events[raised + 1..];
    assert_eq!(
        after_the_fault.first(),
        Some(&CardEvent::Settled),
        "接住之后、再装判器之前先等一次提交做完：{events:?}"
    );
    assert_eq!(
        after_the_fault.get(1),
        Some(&CardEvent::Loaded(stream)),
        "等完了才单独重判那一块（装它那条流的判器）：{events:?}"
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| **event == CardEvent::Settled)
            .count(),
        1,
        "只在接住那一回等一次：{events:?}"
    );
}

/// 设备因显存不够丢了（`src/card_worker.rs` 文件头「接住 Out of Memory」）：判器缓存重开了设备，这张卡不进坏卡、打接住 Out of Memory 那一行
/// （原因里写重开了设备）、照常收下一波，那一块撤回重判、判完交回；设备为别的原因丢了的照旧进坏卡（同
/// `a_card_whose_device_is_lost_while_judging_still_fails_and_tells_one_card_failure`，这里另跑一趟对照）。
/// 两个场景各一张卡：流 3 的那一块头一回判时设备因显存不够丢了，之后交流 4 的一块；流 9 的那一块判时设备为别的原因丢了。
/// 弄坏开关：device-lost-for-want-of-memory-taken-for-a-card-failure（因显存不够丢了、已重开的也报卡坏）。
#[test]
fn a_device_lost_for_want_of_memory_is_reopened_and_the_card_takes_the_next_block() {
    let reopened_on = stream_tagged(3);
    let (summary, _waves, notices) = serve_with_stand_in_judges(
        vec![card_leaving(0, 8 + mebibytes_of_states(10))],
        &CardEvents::default(),
        &faults_of(&[StandInFault {
            card: CardIndex(0),
            target: StandInFaultTarget::Judging(reopened_on),
            kind: StandInFaultKind::DeviceLostForWantOfMemory,
            times: 1,
        }]),
        &JudgedRuns::default(),
        &VerifiedFactsJobs::default(),
        |client| {
            let stream = register(client, 3, 2, 10);
            every_block_judged(client, vec![(3, stream, 0..10)]);
            let next = register(client, 4, 2, 5);
            every_block_judged(client, vec![(4, next, 0..5)]);
        },
    );
    assert!(
        summary.cards_failed.is_empty(),
        "因显存不够丢了设备、已重开：卡不坏：{summary:?}"
    );
    assert_eq!(summary.out_of_memory_caught, 1, "{summary:?}");
    let caught = out_of_memory_notices(&notices);
    assert_eq!(caught.len(), 1, "恰好一件接住：{notices:?}");
    let line = service_notice_line(caught[0], None, &stand_in_block_name);
    assert!(
        line.starts_with(&format!("{OUT_OF_MEMORY_CAUGHT_LINE_PREFIX} card=0 "))
            && line.contains(STAND_IN_REOPENED_REASON),
        "接住那一行写着重开了设备：{line}"
    );
    let lost_on = stream_tagged(9);
    let (lost_summary, _lost_waves, lost_notices) = serve_with_stand_in_judges(
        vec![card_leaving(0, 8 + mebibytes_of_states(10))],
        &CardEvents::default(),
        &faults_of(&[StandInFault {
            card: CardIndex(0),
            target: StandInFaultTarget::Judging(lost_on),
            kind: StandInFaultKind::DeviceLost,
            times: 1,
        }]),
        &JudgedRuns::default(),
        &VerifiedFactsJobs::default(),
        |client| {
            let stream = register(client, 9, 2, 10);
            let answer = client
                .submit_block(stream, 0..10)
                .recv()
                .expect("服务交回这一块");
            assert!(
                matches!(answer, Err(ServiceRefusal::CardFailed { .. })),
                "设备为别的原因丢了：在判的块报拒（卡坏）：{answer:?}"
            );
        },
    );
    assert_eq!(lost_summary.cards_failed, vec![CardIndex(0)]);
    assert!(
        out_of_memory_notices(&lost_notices).is_empty(),
        "为别的原因丢了不算接住：{lost_notices:?}"
    );
}

/// 真卡那一份判器缓存认 panic 之后设备怎样（`src/gpu_judges_on_a_card.rs` 文件头）：照验证包的设备情况与它认显存不够的判法，在假的设备情况上逐种核：
/// 没丢、没多记 Out of Memory 是判法自己的 panic；没丢而多记了是显存不够；丢了、缘由是显存不够的是因显存不够丢的；丢了、缘由是别的、上一回之后没记过
/// Out of Memory 的是为别的原因丢的；缘由是别的而上一回之后记过 Out of Memory 的，验证包认成显存不够。不上卡。
#[test]
fn the_device_after_a_panic_is_read_with_the_verification_package_judgement() {
    let untouched = DeviceConditionOfACard::default();
    assert_eq!(
        device_after_a_panic(&untouched, 0),
        DeviceAfterAPanic::PanickedOtherwise
    );
    let ran_out = DeviceConditionOfACard::default();
    ran_out.record_an_out_of_memory("替身：没接住的 Out of Memory".to_string());
    assert_eq!(
        device_after_a_panic(&ran_out, 0),
        DeviceAfterAPanic::RanOutOfMemory
    );
    assert_eq!(
        device_after_a_panic(&ran_out, 1),
        DeviceAfterAPanic::PanickedOtherwise,
        "上一回已经数过的那一次不算这一回的"
    );
    let lost_for_memory = DeviceConditionOfACard::default();
    lost_for_memory.record_the_device_lost(
        "Unknown：替身因显存不够丢了设备".to_string(),
        CauseOfTheDeviceLoss::GpuMemoryShort,
    );
    assert!(
        matches!(
            device_after_a_panic(&lost_for_memory, 0),
            DeviceAfterAPanic::LostForWantOfMemory { .. }
        ),
        "缘由是显存不够：因显存不够丢的"
    );
    let lost_otherwise = DeviceConditionOfACard::default();
    lost_otherwise.record_the_device_lost(
        "Unknown：替身驱动重置".to_string(),
        CauseOfTheDeviceLoss::Other,
    );
    assert_eq!(
        device_after_a_panic(&lost_otherwise, 0),
        DeviceAfterAPanic::LostOtherwise {
            reason: "Unknown：替身驱动重置".to_string()
        }
    );
    lost_otherwise.record_an_out_of_memory("替身：丢之后的 Out of Memory".to_string());
    assert!(
        matches!(
            device_after_a_panic(&lost_otherwise, 0),
            DeviceAfterAPanic::LostForWantOfMemory { .. }
        ),
        "缘由是别的而上一回之后记过 Out of Memory：验证包认成显存不够"
    );
}

/// 判法自己 panic（`src/service_loop.rs` 文件头「判法自己 panic」）：一条流的判器一装就断言失败（同真卡上表超过绑定上限那一句，设备没丢、没记 Out of Memory），
/// 那一块报拒（判法 panic 了），卡不坏；同一块再交一回照样报拒、卡照样不坏；之后两张卡都照常判别的流，没有一件卡坏。
/// 一个场景：两张卡都在装流 5 的判器时断言失败，流 5 的一块交两回，之后交流 6、流 7 各几块。
/// 弄坏开关：judge-panic-taken-for-a-card-failure（判法 panic 当卡坏：两回各弄坏一张卡，之后好流登记不上）。
#[test]
fn a_judge_panicking_on_loading_refuses_its_blocks_and_every_card_keeps_judging_other_streams() {
    let bad = stream_tagged(5);
    let faults = faults_of(&[
        StandInFault {
            card: CardIndex(0),
            target: StandInFaultTarget::Loading(bad),
            kind: StandInFaultKind::JudgeAssertion,
            times: usize::MAX,
        },
        StandInFault {
            card: CardIndex(1),
            target: StandInFaultTarget::Loading(bad),
            kind: StandInFaultKind::JudgeAssertion,
            times: usize::MAX,
        },
    ]);
    let judged_runs = JudgedRuns::default();
    let (summary, _waves, notices) = serve_with_stand_in_judges(
        vec![
            card_leaving(0, 8 + mebibytes_of_states(10)),
            card_leaving(1, 8 + mebibytes_of_states(10)),
        ],
        &CardEvents::default(),
        &faults,
        &judged_runs,
        &VerifiedFactsJobs::default(),
        |client| {
            let stream = register(client, 5, 2, 10);
            for attempt in ["头一回", "再交一回"] {
                let answer = client
                    .submit_block(stream, 0..10)
                    .recv()
                    .expect("服务交回这一块");
                assert!(
                    matches!(answer, Err(ServiceRefusal::JudgePanicked { .. })),
                    "{attempt}：判法 panic 的那一块报拒（判法 panic 了）：{answer:?}"
                );
            }
            let sixth = register(client, 6, 2, 20);
            let seventh = register(client, 7, 2, 20);
            every_block_judged(
                client,
                vec![
                    (6, sixth, 0..10),
                    (6, sixth, 10..20),
                    (7, seventh, 0..10),
                    (7, seventh, 10..20),
                ],
            );
        },
    );
    assert!(
        summary.cards_failed.is_empty(),
        "判法 panic 不是卡坏：{summary:?}"
    );
    assert_eq!(summary.judge_panics_caught, 2, "{summary:?}");
    assert_eq!(summary.blocks_refused_for_a_judge_panic, 2, "{summary:?}");
    assert!(
        notices
            .iter()
            .all(|notice| !matches!(notice, ServiceNotice::CardFailed { .. })),
        "没有卡坏那一件：{notices:?}"
    );
    let counts = judged_state_counts(&judged_runs);
    assert_eq!(counts.len(), 20 + 20, "流 6、流 7 每个态都判过");
}

/// 显存不够撤回的块（`src/service_loop.rs` 文件头「接住 Out of Memory」）：另一张没坏、空着、装得下的卡照常拿去判，不一块一波排在显存不够的那张卡上；
/// 每一块都判完、结论是它自己的，每个态只判一次。
/// 一个场景：两张卡，一条流 40 块各 1 态一起入池，卡 0 判它们那一批时显存不够一回。
/// 弄坏开关：put-back-blocks-wait-for-their-own-card（撤回的块一律只等卡 0 腾空单独判：一块一波、卡 1 一块不帮）。
#[test]
fn blocks_put_back_after_running_out_of_memory_are_taken_by_another_card_that_fits() {
    const BLOCKS: u64 = 40;
    let stream = stream_tagged(1);
    let capacity = 4 + mebibytes_of_states(64);
    let faults = faults_of(&[StandInFault {
        card: CardIndex(0),
        target: StandInFaultTarget::Judging(stream),
        kind: StandInFaultKind::OutOfMemory,
        times: 1,
    }]);
    let judged_runs = JudgedRuns::default();
    let (summary, waves, notices) = serve_with_stand_in_judges(
        vec![card_leaving(0, capacity), card_leaving(1, capacity)],
        &CardEvents::default(),
        &faults,
        &judged_runs,
        &VerifiedFactsJobs::default(),
        |client| {
            let registered = register(client, 1, 2, 64);
            every_block_judged(
                client,
                (0..BLOCKS)
                    .map(|ordinal| (1, registered, ordinal..ordinal + 1))
                    .collect(),
            );
        },
    );
    assert_eq!(summary.out_of_memory_caught, 1, "{summary:?}");
    let put_back: Vec<Range<u64>> = notices
        .iter()
        .flat_map(|notice| match notice {
            ServiceNotice::OutOfMemoryCaught {
                blocks_put_back, ..
            } => blocks_put_back
                .iter()
                .map(|(_, ordinals)| ordinals.clone())
                .collect(),
            ServiceNotice::JudgeLoaded { .. } | ServiceNotice::CardFailed { .. } => Vec::new(),
        })
        .collect();
    assert!(
        put_back.len() > 1,
        "取样：卡 0 显存不够的那一批要带不止一块（机器忙、头一波发得早的重跑，不算推翻）：{put_back:?}"
    );
    let put_back_judged_on_the_other_card = judged_runs
        .lock()
        .expect("服务停了")
        .iter()
        .filter(|(card, _, ordinals)| {
            *card == CardIndex(1)
                && put_back
                    .iter()
                    .any(|put| put.start < ordinals.end && ordinals.start < put.end)
        })
        .count();
    let waves_after_the_first = waves.len().saturating_sub(1);
    println!(
        "PUT_BACK_BLOCKS put_back={} waves={} put_back_judged_on_card1={put_back_judged_on_the_other_card}",
        put_back.len(),
        waves.len()
    );
    assert!(
        put_back_judged_on_the_other_card > 0,
        "撤回的块卡 1 拿去判了：{waves:?}"
    );
    assert!(
        waves_after_the_first < put_back.len(),
        "撤回的 {} 块不一块一波：头一波之后发了 {waves_after_the_first} 波",
        put_back.len()
    );
    let counts = judged_state_counts(&judged_runs);
    assert_eq!(
        counts.len(),
        usize::try_from(BLOCKS).expect("装得进"),
        "每个态都判过"
    );
    assert!(counts.values().all(|count| *count == 1), "每个态只判一次");
}

/// 大卡坏了之后（`src/service_loop.rs` 文件头「一张卡坏了」）：只有它装得下的那条流再入池的块当场报拒（没有一张没坏的卡装得下），不挂在池里；
/// 小卡照常判装得下的流。服务跑在另一条线程里，测试这一侧等客户端那一侧的结局至多 [`HANGING_BLOCK_LIMIT`]：挂着的块让服务停不下来，
/// 等不到就判红、不陪着挂。
/// 一个场景：卡 0 大（装得下流 1 的表两份）、卡 1 小（装不下），卡 0 判流 1 的头一块时设备为别的原因丢了；再交流 1 的一块，再交流 2 的一块。
/// 弄坏开关：blocks-only-the-failed-card-could-hold-left-pending（不报拒、挂着）。
#[test]
fn a_block_only_the_failed_big_card_could_hold_is_refused_not_left_hanging() {
    const LARGE_TABLE_MEBIBYTES: u64 = 40;
    const HANGING_BLOCK_LIMIT: Duration = Duration::from_secs(3);
    let big = stream_tagged(1);
    let faults = faults_of(&[StandInFault {
        card: CardIndex(0),
        target: StandInFaultTarget::Judging(big),
        kind: StandInFaultKind::DeviceLost,
        times: 1,
    }]);
    let (said, heard) = channel::<Result<(), String>>();
    let serving = std::thread::spawn(move || {
        serve_with_stand_in_judges(
            vec![
                card_leaving(0, 2 * LARGE_TABLE_MEBIBYTES + mebibytes_of_states(10)),
                card_leaving(1, 8 + mebibytes_of_states(10)),
            ],
            &CardEvents::default(),
            &faults,
            &JudgedRuns::default(),
            &VerifiedFactsJobs::default(),
            move |client| {
                let large = register(client, 1, LARGE_TABLE_MEBIBYTES, 10);
                let small = register(client, 2, 2, 10);
                let first = client
                    .submit_block(large, 0..10)
                    .recv()
                    .expect("服务交回这一块");
                if !matches!(first, Err(ServiceRefusal::CardFailed { .. })) {
                    let _ = said.send(Err(format!(
                        "大卡坏了，在判的那一块报拒（卡坏）：{first:?}"
                    )));
                    return;
                }
                let again = client
                    .submit_block(large, 0..5)
                    .recv_timeout(HANGING_BLOCK_LIMIT);
                let outcome = match again {
                    Ok(Err(ServiceRefusal::StreamFitsNoCard { .. })) => {
                        every_block_judged(client, vec![(2, small, 0..10)]);
                        Ok(())
                    }
                    Ok(other) => Err(format!(
                        "大卡坏了之后再入池的那一块应当报拒（没有一张没坏的卡装得下），实际 {other:?}"
                    )),
                    Err(waited) => Err(format!(
                        "大卡坏了之后再入池的块 {} 秒里没有回话（{waited:?}）：没有哪张没坏的卡装得下它，却没被拒、挂在池里",
                        HANGING_BLOCK_LIMIT.as_secs()
                    )),
                };
                let _ = said.send(outcome);
            },
        )
    });
    let outcome = heard
        .recv_timeout(HANGING_BLOCK_LIMIT * 10)
        .unwrap_or_else(|waited| Err(format!("客户端那一侧没交结局：{waited:?}")));
    if let Err(problem) = outcome {
        panic!("{problem}");
    }
    let (summary, _waves, _notices) = serving.join().expect("服务线程没 panic");
    assert_eq!(summary.cards_failed, vec![CardIndex(0)], "{summary:?}");
}
