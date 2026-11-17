//! 服务主循环：把待判池、批次调度与各卡的工作线程接起来；客户端（[`ServiceClient`]）在同一份里。
//!
//! 一圈：有空着的卡就问一次调度（`crate::batch_scheduling::plan_the_next_wave`）；发一波就把派出去的各截在池里记成在判、
//! 照计划更新各卡驻留的判器、把每张卡那一份交给它的工作线程；不发就等下一个事件（客户端的请求、工作线程的回报），
//! 水位线没到的等到限时到点再问。工作线程每判完一批交回结论，池里一块的态全有了结论就把整块交回入池的那个客户端。
//! 收到「判完就停」之后不再收新块，水位线当已到（池里有态就发），池空、卡都空出来就叫工作线程停、退出。
//! 一张卡判的时候坏了：这张卡不再排，它上面在判的那几块整块撤掉、向客户端报拒（判了一半的结论不交）。
//!
//! 登记流时判它装不装得进任何一张卡（表加一个态的草稿区），装不进的当场拒：池里不留永远派不出去的态。

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::batch_scheduling::{
    plan_the_next_wave, BatchPiece, BatchSchedulingConfiguration, PendingBlockIdentifier,
    ResidentJudge, SchedulingInputs, WaveDecision, WavePlan,
};
use crate::card_memory_ledger::{mebibytes_of_a_judge, CardIndex, CardOfTheService};
use crate::card_worker::{
    card_worker_loop, CardWorkerCommand, CardWorkerReport, JudgesOnACard, WaveForTheCard,
};
use crate::pending_pool::{JudgedBlock, PendingPool, StateVerdict};
use crate::stream_identity::{StreamFacts, StreamKey};

/// 服务拒的。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ServiceRefusal {
    /// 入池的块的流没登记过。
    StreamNotRegistered { stream: StreamKey },
    /// 同一把键登记过另一份事实（同一份表与源码不会有两种表大小或态数）。
    StreamRegisteredWithOtherFacts {
        stream: StreamKey,
        registered: StreamFacts,
        offered: StreamFacts,
    },
    /// 这条流的表加一个态的草稿区装不进任何一张卡。
    StreamFitsNoCard {
        stream: StreamKey,
        judge_mebibytes_at_one_state: u64,
        largest_card_mebibytes_for_the_judges: u64,
    },
    /// 块的序号是空的或越过了这条流的态数。
    OrdinalsOutsideTheStream {
        stream: StreamKey,
        ordinals: Range<u64>,
        state_count: u64,
    },
    /// 判这一块的卡坏了，整块撤掉。
    CardFailed { card: CardIndex, reason: String },
    /// 服务的卡都坏了，入池的块没有卡判。
    NoWorkingCard,
    /// 服务在停，不再收新块。
    ServiceStopping,
}

/// 客户端交给服务的请求。
pub enum ServiceRequest<Materials> {
    RegisterStream {
        stream: StreamKey,
        facts: StreamFacts,
        materials: Arc<Materials>,
        reply: Sender<Result<(), ServiceRefusal>>,
    },
    SubmitBlock {
        stream: StreamKey,
        ordinals: Range<u64>,
        reply: Sender<Result<JudgedBlock, ServiceRefusal>>,
    },
    DrainAndStop,
}

/// 主循环收的事件：客户端的请求与工作线程的回报走同一条通道。
pub enum ServiceEvent<Materials> {
    Request(ServiceRequest<Materials>),
    Worker(CardWorkerReport),
}

/// 客户端：可以克隆，几条线程各拿一份。
pub struct ServiceClient<Materials> {
    events: Sender<ServiceEvent<Materials>>,
}

impl<Materials> Clone for ServiceClient<Materials> {
    fn clone(&self) -> Self {
        Self {
            events: self.events.clone(),
        }
    }
}

/// 服务那一头的收件箱（交给 [`serve_the_cards`]）。
pub struct ServiceInbox<Materials> {
    events: Receiver<ServiceEvent<Materials>>,
    events_for_the_workers: Sender<ServiceEvent<Materials>>,
}

/// 一对客户端与收件箱。
#[must_use]
pub fn service_channel<Materials>() -> (ServiceClient<Materials>, ServiceInbox<Materials>) {
    let (sender, receiver) = channel();
    (
        ServiceClient {
            events: sender.clone(),
        },
        ServiceInbox {
            events: receiver,
            events_for_the_workers: sender,
        },
    )
}

impl<Materials> ServiceClient<Materials> {
    /// 登记一条流（入池之前先登记）；同一把键同一份事实再登记一次照收。等服务回话。
    ///
    /// # Errors
    /// [`ServiceRefusal::StreamRegisteredWithOtherFacts`]、[`ServiceRefusal::StreamFitsNoCard`]、[`ServiceRefusal::ServiceStopping`]（服务已经退出）。
    pub fn register_stream(
        &self,
        stream: StreamKey,
        facts: StreamFacts,
        materials: Arc<Materials>,
    ) -> Result<(), ServiceRefusal> {
        let (reply, answer) = channel();
        self.events
            .send(ServiceEvent::Request(ServiceRequest::RegisterStream {
                stream,
                facts,
                materials,
                reply,
            }))
            .map_err(|_| ServiceRefusal::ServiceStopping)?;
        answer
            .recv()
            .unwrap_or(Err(ServiceRefusal::ServiceStopping))
    }

    /// 一块入池，不等：整块判完（或被拒）时从交回的那一头收到。服务已经退出时那一头收到的是 [`ServiceRefusal::ServiceStopping`]。
    #[must_use]
    pub fn submit_block(
        &self,
        stream: StreamKey,
        ordinals: Range<u64>,
    ) -> Receiver<Result<JudgedBlock, ServiceRefusal>> {
        let (reply, answer) = channel();
        let refused_reply = reply.clone();
        if self
            .events
            .send(ServiceEvent::Request(ServiceRequest::SubmitBlock {
                stream,
                ordinals,
                reply,
            }))
            .is_err()
        {
            // 服务那一头已经没了：照拒交回，等的人不挂着
            let _ = refused_reply.send(Err(ServiceRefusal::ServiceStopping));
        }
        answer
    }

    /// 已入池的判完就停。
    pub fn drain_and_stop(&self) {
        let _ = self
            .events
            .send(ServiceEvent::Request(ServiceRequest::DrainAndStop));
    }
}

/// 一张卡与在它的工作线程里造判器缓存的办法（在工作线程里造：判器缓存不必能跨线程交）。
pub struct CardWithItsJudges<MakeJudges> {
    pub card: CardOfTheService,
    pub make_the_judges: MakeJudges,
}

/// 服务退出时交回的账。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ServiceRunSummary {
    pub waves_dispatched: u64,
    pub batches_dispatched: u64,
    pub blocks_judged: u64,
    pub cards_failed: Vec<CardIndex>,
}

/// 起各卡的工作线程、跑主循环，收到「判完就停」并且判完了才返回。每发一波先交给 `on_each_wave` 看一眼（测试照它核派法）。
///
/// # Panics
/// 池拒收调度给的截（调度与池对不上）；同 [`plan_the_next_wave`]。
pub fn serve_the_cards<Judges, MakeJudges, Pool>(
    configuration: &BatchSchedulingConfiguration,
    cards: Vec<CardWithItsJudges<MakeJudges>>,
    pool: Pool,
    inbox: ServiceInbox<Judges::StreamMaterials>,
    on_each_wave: &mut dyn FnMut(&WavePlan),
) -> ServiceRunSummary
where
    Judges: JudgesOnACard,
    MakeJudges: FnOnce() -> Judges + Send,
    Pool: PendingPool,
{
    std::thread::scope(|scope| {
        let mut commands = BTreeMap::new();
        let mut cards_of_the_service = Vec::new();
        for card in cards {
            let (command_sender, command_receiver) = channel();
            let events = inbox.events_for_the_workers.clone();
            let index = card.card.card;
            let make_the_judges = card.make_the_judges;
            scope.spawn(move || {
                let mut judges = make_the_judges();
                card_worker_loop(
                    index,
                    &mut judges,
                    &command_receiver,
                    &mut |report: CardWorkerReport| {
                        let _ = events.send(ServiceEvent::Worker(report));
                    },
                );
            });
            commands.insert(index, command_sender);
            cards_of_the_service.push(card.card);
        }
        let mut service = ServiceState {
            configuration,
            cards: cards_of_the_service,
            pool,
            streams: BTreeMap::new(),
            residents: BTreeMap::new(),
            idle: BTreeSet::new(),
            failed: BTreeSet::new(),
            replies: BTreeMap::new(),
            blocks_in_flight_by_card: BTreeMap::new(),
            withdrawn: BTreeSet::new(),
            is_draining: false,
            started_at: Instant::now(),
            summary: ServiceRunSummary::default(),
        };
        service.idle = service.cards.iter().map(|card| card.card).collect();
        service.serve(&inbox.events, &commands, on_each_wave);
        for command in commands.values() {
            let _ = command.send(CardWorkerCommand::Stop);
        }
        service.summary
    })
}

struct ServiceState<'configuration, Pool, Materials> {
    configuration: &'configuration BatchSchedulingConfiguration,
    cards: Vec<CardOfTheService>,
    pool: Pool,
    streams: BTreeMap<StreamKey, (StreamFacts, Arc<Materials>)>,
    residents: BTreeMap<CardIndex, Vec<ResidentJudge>>,
    idle: BTreeSet<CardIndex>,
    failed: BTreeSet<CardIndex>,
    replies: BTreeMap<PendingBlockIdentifier, Sender<Result<JudgedBlock, ServiceRefusal>>>,
    blocks_in_flight_by_card: BTreeMap<CardIndex, BTreeSet<PendingBlockIdentifier>>,
    withdrawn: BTreeSet<PendingBlockIdentifier>,
    is_draining: bool,
    started_at: Instant,
    summary: ServiceRunSummary,
}

/// 下一步等事件等多久。
enum WaitForTheNextEvent {
    Forever,
    AtMost(Duration),
}

impl<Pool: PendingPool, Materials> ServiceState<'_, Pool, Materials> {
    fn serve(
        &mut self,
        events: &Receiver<ServiceEvent<Materials>>,
        commands: &BTreeMap<CardIndex, Sender<CardWorkerCommand<Materials>>>,
        on_each_wave: &mut dyn FnMut(&WavePlan),
    ) {
        loop {
            let wait = self.dispatch_while_the_scheduler_says_so(commands, on_each_wave);
            if self.is_draining && self.replies.is_empty() && self.every_working_card_is_idle() {
                return;
            }
            let event = match wait {
                WaitForTheNextEvent::Forever => match events.recv() {
                    Ok(event) => event,
                    Err(_) => return,
                },
                WaitForTheNextEvent::AtMost(timeout) => match events.recv_timeout(timeout) {
                    Ok(event) => event,
                    Err(RecvTimeoutError::Timeout) => continue,
                    Err(RecvTimeoutError::Disconnected) => return,
                },
            };
            self.take_the_event(event);
        }
    }

    fn every_working_card_is_idle(&self) -> bool {
        self.cards
            .iter()
            .filter(|card| !self.failed.contains(&card.card))
            .all(|card| self.idle.contains(&card.card))
    }

    fn now(&self) -> Duration {
        self.started_at.elapsed()
    }

    /// 有空着的卡就问调度，发了就再问（发过的卡不再空着），直到调度说不发。
    fn dispatch_while_the_scheduler_says_so(
        &mut self,
        commands: &BTreeMap<CardIndex, Sender<CardWorkerCommand<Materials>>>,
        on_each_wave: &mut dyn FnMut(&WavePlan),
    ) -> WaitForTheNextEvent {
        loop {
            let idle_cards: Vec<CardOfTheService> = self
                .cards
                .iter()
                .filter(|card| self.idle.contains(&card.card) && !self.failed.contains(&card.card))
                .copied()
                .collect();
            if idle_cards.is_empty() {
                return WaitForTheNextEvent::Forever;
            }
            let residents: Vec<ResidentJudge> =
                self.residents.values().flatten().copied().collect();
            let facts: BTreeMap<StreamKey, StreamFacts> = self
                .streams
                .iter()
                .map(|(stream, (facts, _))| (*stream, *facts))
                .collect();
            let pending = self.pool.undispatched_blocks();
            let draining_configuration;
            let configuration = if self.is_draining {
                draining_configuration = BatchSchedulingConfiguration {
                    oldest_pending_wait_limit: Duration::ZERO,
                    break_switches: self.configuration.break_switches.clone(),
                };
                &draining_configuration
            } else {
                self.configuration
            };
            let decision = plan_the_next_wave(
                configuration,
                &SchedulingInputs {
                    idle_cards: &idle_cards,
                    residents: &residents,
                    streams: &facts,
                    pending: &pending,
                    now: self.now(),
                },
            );
            match decision {
                WaveDecision::Dispatch(plan) => {
                    self.dispatch_the_wave(plan, commands, on_each_wave);
                }
                WaveDecision::HoldBelowTheWatermark { wait_left, .. } => {
                    return WaitForTheNextEvent::AtMost(wait_left.max(Duration::from_millis(1)));
                }
                WaveDecision::HoldTheEmptyPool
                | WaveDecision::HoldNothingPlaceableOnTheIdleCards => {
                    return WaitForTheNextEvent::Forever;
                }
            }
        }
    }

    fn dispatch_the_wave(
        &mut self,
        plan: WavePlan,
        commands: &BTreeMap<CardIndex, Sender<CardWorkerCommand<Materials>>>,
        on_each_wave: &mut dyn FnMut(&WavePlan),
    ) {
        on_each_wave(&plan);
        let mut pieces: Vec<&BatchPiece> = plan.pieces().collect();
        pieces.sort_by_key(|piece| (piece.block, piece.ordinals.start));
        for piece in pieces {
            self.pool
                .mark_dispatched(piece)
                .unwrap_or_else(|refusal| panic!("池拒收调度派出去的一截：{refusal:?}"));
        }
        self.summary.waves_dispatched += 1;
        for card_plan in plan.cards {
            if card_plan.batches.is_empty()
                && card_plan.evictions.is_empty()
                && card_plan.loads.is_empty()
            {
                continue;
            }
            self.summary.batches_dispatched +=
                u64::try_from(card_plan.batches.len()).expect("装得进 u64");
            self.residents.insert(
                card_plan.card,
                card_plan
                    .resident_after_the_wave()
                    .judges()
                    .iter()
                    .map(|judge| ResidentJudge {
                        card: card_plan.card,
                        stream: judge.stream,
                        states_per_dispatch: judge.states_per_dispatch,
                    })
                    .collect(),
            );
            self.blocks_in_flight_by_card.insert(
                card_plan.card,
                card_plan
                    .batches
                    .iter()
                    .flat_map(|batch| batch.pieces.iter().map(|piece| piece.block))
                    .collect(),
            );
            let materials_of_the_loads = card_plan
                .loads
                .iter()
                .map(|load| Arc::clone(&self.streams[&load.stream].1))
                .collect();
            self.idle.remove(&card_plan.card);
            let card = card_plan.card;
            let sent = commands[&card].send(CardWorkerCommand::CarryOutTheWave(WaveForTheCard {
                plan: card_plan,
                materials_of_the_loads,
            }));
            if sent.is_err() {
                self.take_a_failed_card(card, "工作线程已经退出".to_string());
            }
        }
    }

    fn take_the_event(&mut self, event: ServiceEvent<Materials>) {
        match event {
            ServiceEvent::Request(ServiceRequest::RegisterStream {
                stream,
                facts,
                materials,
                reply,
            }) => {
                let _ = reply.send(self.register_the_stream(stream, facts, materials));
            }
            ServiceEvent::Request(ServiceRequest::SubmitBlock {
                stream,
                ordinals,
                reply,
            }) => match self.admit_the_block(stream, ordinals) {
                Ok(block) => {
                    self.replies.insert(block, reply);
                }
                Err(refusal) => {
                    let _ = reply.send(Err(refusal));
                }
            },
            ServiceEvent::Request(ServiceRequest::DrainAndStop) => {
                self.is_draining = true;
            }
            ServiceEvent::Worker(CardWorkerReport::BatchJudged { card, pieces }) => {
                for (piece, verdicts) in pieces {
                    self.take_the_verdicts_of_a_piece(&piece, card, verdicts);
                }
            }
            ServiceEvent::Worker(CardWorkerReport::RoundFinished { card }) => {
                self.blocks_in_flight_by_card.remove(&card);
                self.idle.insert(card);
            }
            ServiceEvent::Worker(CardWorkerReport::CardFailed { card, reason }) => {
                self.take_a_failed_card(card, reason);
            }
        }
    }

    fn register_the_stream(
        &mut self,
        stream: StreamKey,
        facts: StreamFacts,
        materials: Arc<Materials>,
    ) -> Result<(), ServiceRefusal> {
        if let Some((registered, _)) = self.streams.get(&stream) {
            return if *registered == facts {
                Ok(())
            } else {
                Err(ServiceRefusal::StreamRegisteredWithOtherFacts {
                    stream,
                    registered: *registered,
                    offered: facts,
                })
            };
        }
        let judge_mebibytes_at_one_state = mebibytes_of_a_judge(&facts, 1);
        let largest_card_mebibytes_for_the_judges = self
            .cards
            .iter()
            .filter(|card| !self.failed.contains(&card.card))
            .map(CardOfTheService::mebibytes_for_the_judges)
            .max()
            .unwrap_or(0);
        if judge_mebibytes_at_one_state > largest_card_mebibytes_for_the_judges {
            return Err(ServiceRefusal::StreamFitsNoCard {
                stream,
                judge_mebibytes_at_one_state,
                largest_card_mebibytes_for_the_judges,
            });
        }
        self.streams.insert(stream, (facts, materials));
        Ok(())
    }

    fn admit_the_block(
        &mut self,
        stream: StreamKey,
        ordinals: Range<u64>,
    ) -> Result<PendingBlockIdentifier, ServiceRefusal> {
        if self.is_draining {
            return Err(ServiceRefusal::ServiceStopping);
        }
        if self
            .cards
            .iter()
            .all(|card| self.failed.contains(&card.card))
        {
            return Err(ServiceRefusal::NoWorkingCard);
        }
        let Some((facts, _)) = self.streams.get(&stream) else {
            return Err(ServiceRefusal::StreamNotRegistered { stream });
        };
        if ordinals.is_empty() || ordinals.end > facts.state_count {
            return Err(ServiceRefusal::OrdinalsOutsideTheStream {
                stream,
                ordinals,
                state_count: facts.state_count,
            });
        }
        let now = self.now();
        self.pool
            .add_block(stream, ordinals.clone(), now)
            .map_err(|_| ServiceRefusal::OrdinalsOutsideTheStream {
                stream,
                ordinals,
                state_count: facts.state_count,
            })
    }

    fn take_the_verdicts_of_a_piece(
        &mut self,
        piece: &BatchPiece,
        card: CardIndex,
        verdicts: Vec<StateVerdict>,
    ) {
        if self.withdrawn.contains(&piece.block) {
            return;
        }
        match self.pool.record_verdicts(piece, card, verdicts) {
            Ok(None) => {}
            Ok(Some(judged)) => {
                self.summary.blocks_judged += 1;
                if let Some(reply) = self.replies.remove(&judged.block) {
                    let _ = reply.send(Ok(judged));
                }
            }
            Err(refusal) => {
                panic!("池拒收工作线程交回的结论（工作线程、调度与池对不上）：{refusal:?}");
            }
        }
    }

    /// 一张卡坏了：不再排它、忘掉它上面的判器，它这一波在判的块整块撤掉、向客户端报拒。
    fn take_a_failed_card(&mut self, card: CardIndex, reason: String) {
        self.failed.insert(card);
        self.idle.remove(&card);
        self.residents.remove(&card);
        self.summary.cards_failed.push(card);
        for block in self
            .blocks_in_flight_by_card
            .remove(&card)
            .unwrap_or_default()
        {
            if self.pool.withdraw_the_block(block).is_some() {
                self.withdrawn.insert(block);
                if let Some(reply) = self.replies.remove(&block) {
                    let _ = reply.send(Err(ServiceRefusal::CardFailed {
                        card,
                        reason: reason.clone(),
                    }));
                }
            }
        }
        if self
            .cards
            .iter()
            .all(|other| self.failed.contains(&other.card))
        {
            for block in self.pool.undispatched_blocks() {
                self.pool.withdraw_the_block(block.block);
                self.withdrawn.insert(block.block);
                if let Some(reply) = self.replies.remove(&block.block) {
                    let _ = reply.send(Err(ServiceRefusal::CardFailed {
                        card,
                        reason: format!("服务的卡都坏了（最后一张的原因：{reason}）"),
                    }));
                }
            }
        }
    }
}
