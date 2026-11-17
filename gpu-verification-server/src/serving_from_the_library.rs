//! 服务进程的取活循环：起来就开服务持着的那一组卡、起一段卡上的会话（一条线程跑 [`serve_the_cards`]，块进内存里的待判池，批次调度照旧），
//! 一直持着到停；从判定库那一份待判池取活、交服务主循环判、判完写回。卡不经统一任务服务领、不还（卡归这个服务管）。
//!
//! 一圈：收主循环判完的块逐块写回（[`LibraryPendingPool::write_back`]；被撤的块放回库里等下一轮）；在判的块到点续认领；扫库挑能取的块
//! （[`LibraryPendingPool::candidates`]），认领、登记流、入池。有块在判时每圈隔 [`POLL_WHILE_JUDGING`]，没有时隔 `library_scan_interval`。
//! `should_stop` 为真时收尾：会话判完已入池的块、逐块写回、放掉卡。
//!
//! 开卡没成（起来那一回）整个循环交回 [`ServingFailure::CardsNotOpened`]；读写判定库失败（连不上、忙到上限、服务停了）交回
//! [`ServingFailure::Library`]，调用方照判定库客户端的退出码退出。一块被主循环拒（卡坏了）只打一行、照转。

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::thread::JoinHandle;
use std::time::Duration;

use singlefs_verification::crash_amplification::CardThatJudgedAPendingBlock;

use crate::batch_scheduling::BatchSchedulingConfiguration;
use crate::break_switches::{BreakSwitch, BreakSwitches};
use crate::card_memory_ledger::CardIndex;
use crate::cards_of_the_service::CardsOfTheService;
use crate::library_pending_pool::{
    LibraryFailure, LibraryPendingPool, PendingBlockName, TakenBlock, VerificationPackageSide,
};
use crate::pending_pool::{InMemoryPendingPool, JudgedBlock};
use crate::service_configuration::ServiceConfiguration;
use crate::service_loop::{
    serve_the_cards, service_channel, CardWithItsJudges, ServiceClient, ServiceRefusal,
    ServiceRunSummary,
};
use crate::stream_identity::StreamKey;

/// 有块在判时一圈隔多久（收判完的块不等满一个扫库间隔）。
pub const POLL_WHILE_JUDGING: Duration = Duration::from_millis(20);
/// 开了卡的那一行的开头。
pub const CARDS_OPENED_LINE_PREFIX: &str = "GPU_VERIFICATION_SERVER_CARDS_OPENED";
/// 一块被主循环拒、续认领没续上的那一行的开头。
pub const SERVING_PROBLEM_LINE_PREFIX: &str = "GPU_VERIFICATION_SERVER_SERVING_PROBLEM";

/// 循环退出时交回的账。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ServingSummary {
    /// 开卡开成了几回（起来那一回；弄坏开关之外只有这一回）。
    pub card_openings: u64,
    /// 停之前放掉过几回卡（弄坏开关之外是 0：卡一直持着）。
    pub cards_let_go_before_stopping: u64,
    pub blocks_written_back: u64,
    pub blocks_refused_at_the_write: u64,
    pub blocks_withdrawn: u64,
    /// 判完而可复用、待判行带守卫删掉了的块数。
    pub reusable_rows_deleted: u64,
}

/// 循环没跑下去的原因。
#[derive(Debug)]
pub enum ServingFailure {
    /// 起来那一回开卡没成：服务没有卡可判，不起。
    CardsNotOpened(String),
    /// 读写判定库失败。
    Library(LibraryFailure),
}

impl From<LibraryFailure> for ServingFailure {
    fn from(failure: LibraryFailure) -> Self {
        Self::Library(failure)
    }
}

/// 入了池、在等主循环交回的一块。
struct BlockInFlight {
    name: PendingBlockName,
    ordinals: Range<u64>,
    answer: Receiver<Result<JudgedBlock, ServiceRefusal>>,
}

/// 持着卡的一段会话：主循环的客户端、跑主循环的线程（退出时交回卡）、登记过的流。
struct CardSession<Cards: CardsOfTheService> {
    client: ServiceClient<Cards::Materials>,
    thread: JoinHandle<(Cards, ServiceRunSummary)>,
    registered: BTreeSet<StreamKey>,
}

fn start_the_session<Cards: CardsOfTheService>(
    cards: Cards,
    configuration: BatchSchedulingConfiguration,
) -> CardSession<Cards> {
    let (client, inbox) = service_channel::<Cards::Materials>();
    let thread = std::thread::spawn(move || {
        let summary = {
            let cards_ref = &cards;
            let cards_with_judges = cards_ref
                .cards_of_the_service()
                .into_iter()
                .map(|card| {
                    let index = card.card;
                    CardWithItsJudges {
                        card,
                        make_the_judges: move || cards_ref.judges_on(index),
                    }
                })
                .collect();
            let pool = InMemoryPendingPool::empty(configuration.break_switches.clone());
            serve_the_cards(&configuration, cards_with_judges, pool, inbox, &mut |_| {})
        };
        (cards, summary)
    });
    CardSession {
        client,
        thread,
        registered: BTreeSet::new(),
    }
}

/// 取活循环的状态。
struct Serving<'pool, 'opener, Side, Cards: CardsOfTheService> {
    configuration: &'pool ServiceConfiguration,
    break_switches: &'pool BreakSwitches,
    pool: &'pool mut LibraryPendingPool<Side>,
    open_the_cards: &'opener mut dyn FnMut() -> Result<Cards, String>,
    session: Option<CardSession<Cards>>,
    /// 最近开的那一组卡每张是什么卡（会话收掉之后写回它交回的块还要用）。
    cards_that_judge: BTreeMap<CardIndex, CardThatJudgedAPendingBlock>,
    in_flight: Vec<BlockInFlight>,
    summary: ServingSummary,
}

/// 跑取活循环：先开卡（开不成交回 [`ServingFailure::CardsNotOpened`]），`should_stop` 为真时收尾返回。
///
/// # Errors
/// 起来那一回开卡没成；读写判定库失败。
pub fn serve_from_the_library<Side, Cards>(
    configuration: &ServiceConfiguration,
    break_switches: &BreakSwitches,
    pool: &mut LibraryPendingPool<Side>,
    open_the_cards: &mut dyn FnMut() -> Result<Cards, String>,
    should_stop: &dyn Fn() -> bool,
) -> Result<ServingSummary, ServingFailure>
where
    Side: VerificationPackageSide,
    Cards: CardsOfTheService<Materials = Side::Materials>,
{
    let mut serving = Serving {
        configuration,
        break_switches,
        pool,
        open_the_cards,
        session: None,
        cards_that_judge: BTreeMap::new(),
        in_flight: Vec::new(),
        summary: ServingSummary::default(),
    };
    if !break_switches.is_on(BreakSwitch::CardsOpenedOnlyWithWork) {
        serving
            .open_the_session()
            .map_err(ServingFailure::CardsNotOpened)?;
    }
    let outcome = serving.serve_until(should_stop);
    serving.end_the_session();
    serving.summary.reusable_rows_deleted = serving.pool.reusable_rows_deleted();
    outcome.map(|()| serving.summary)
}

impl<Side, Cards> Serving<'_, '_, Side, Cards>
where
    Side: VerificationPackageSide,
    Cards: CardsOfTheService<Materials = Side::Materials>,
{
    fn scheduling_configuration(&self) -> BatchSchedulingConfiguration {
        BatchSchedulingConfiguration {
            oldest_pending_wait_limit: self.configuration.oldest_pending_wait_limit,
            break_switches: self.break_switches.clone(),
        }
    }

    fn serve_until(&mut self, should_stop: &dyn Fn() -> bool) -> Result<(), ServingFailure> {
        // 迭代上界：不设，服务进程一直取活，直到 should_stop（测试）或进程被停
        while !should_stop() {
            self.take_the_judged_blocks();
            for lost in self.pool.renew_the_claims()? {
                eprintln!("{SERVING_PROBLEM_LINE_PREFIX} {lost} 续认领没续上（别的持有者持着）：照判，写回时那一帧核认领、不写");
            }
            let candidates = self.pool.candidates()?;
            if !candidates.is_empty() && self.session.is_none() {
                // 只在弄坏开关下走到：起来没开卡，或池空时放掉了卡
                self.open_the_session()
                    .map_err(ServingFailure::CardsNotOpened)?;
            }
            let mut claimed_any = false;
            if !candidates.is_empty() {
                let round = self.pool.claim_the_candidates(candidates)?;
                claimed_any = !round.taken.is_empty();
                for taken in round.taken {
                    self.submit(taken);
                }
            }
            let is_idle =
                !claimed_any && self.in_flight.is_empty() && self.pool.claimed_count() == 0;
            if is_idle
                && self.session.is_some()
                && self
                    .break_switches
                    .is_on(BreakSwitch::CardsLetGoWhenThePoolIdles)
            {
                self.end_the_session();
                self.summary.cards_let_go_before_stopping += 1;
            }
            std::thread::sleep(if self.in_flight.is_empty() {
                self.configuration.library_scan_interval
            } else {
                POLL_WHILE_JUDGING
            });
        }
        Ok(())
    }

    fn open_the_session(&mut self) -> Result<(), String> {
        let cards = (self.open_the_cards)()?;
        self.cards_that_judge = cards
            .cards_of_the_service()
            .into_iter()
            .map(|card| (card.card, cards.card_that_judged(card.card)))
            .collect();
        eprintln!(
            "{CARDS_OPENED_LINE_PREFIX} cards={}",
            self.cards_that_judge.len()
        );
        self.summary.card_openings += 1;
        self.session = Some(start_the_session(cards, self.scheduling_configuration()));
        Ok(())
    }

    fn submit(&mut self, taken: TakenBlock<Side::Materials>) {
        let session = self.session.as_mut().expect("认领只在持着卡时做");
        let TakenBlock { pending, stream } = taken;
        if !session.registered.contains(&stream.stream) {
            if let Err(refusal) =
                session
                    .client
                    .register_stream(stream.stream, stream.facts, stream.materials)
            {
                eprintln!(
                    "{SERVING_PROBLEM_LINE_PREFIX} {} 的流登记不上：{refusal:?}（留在库里）",
                    pending.name
                );
                self.pool.withdraw(&pending.name);
                self.summary.blocks_withdrawn += 1;
                return;
            }
            session.registered.insert(stream.stream);
        }
        let ordinals = pending.row.state_ordinals.clone();
        let answer = session.client.submit_block(stream.stream, ordinals.clone());
        self.in_flight.push(BlockInFlight {
            name: pending.name,
            ordinals,
            answer,
        });
    }

    /// 收主循环交回的块：判完的写回，被拒的放回库里。
    fn take_the_judged_blocks(&mut self) {
        let mut still_in_flight = Vec::with_capacity(self.in_flight.len());
        for block in std::mem::take(&mut self.in_flight) {
            match block.answer.try_recv() {
                Ok(answer) => self.take_one_answer(block.name, &block.ordinals, answer),
                Err(TryRecvError::Empty) => still_in_flight.push(block),
                Err(TryRecvError::Disconnected) => self.take_one_answer(
                    block.name,
                    &block.ordinals,
                    Err(ServiceRefusal::ServiceStopping),
                ),
            }
        }
        self.in_flight = still_in_flight;
    }

    /// 一块的几截落在几张卡上时，结论块属性里写号最小的那一张（同一台的卡同型同驱动时没有差别）。
    fn card_that_judged(&self, judged: &JudgedBlock) -> CardThatJudgedAPendingBlock {
        let card = judged
            .judged_on_cards
            .first()
            .expect("判完的一块至少有一截在一张卡上判过");
        self.cards_that_judge
            .get(card)
            .cloned()
            .unwrap_or_else(|| panic!("{card:?} 不是最近开的那一组卡里的"))
    }

    fn take_one_answer(
        &mut self,
        name: PendingBlockName,
        ordinals: &Range<u64>,
        answer: Result<JudgedBlock, ServiceRefusal>,
    ) {
        match answer {
            Ok(judged) => {
                assert_eq!(
                    &judged.ordinals, ordinals,
                    "主循环交回的 {name} 不是入池的那一段"
                );
                let card = self.card_that_judged(&judged);
                match self.pool.write_back(&name, judged.verdicts, &card) {
                    Ok(_) => self.summary.blocks_written_back += 1,
                    Err(_) => self.summary.blocks_refused_at_the_write += 1,
                }
            }
            Err(refusal) => {
                eprintln!("{SERVING_PROBLEM_LINE_PREFIX} {name} 被主循环拒了：{refusal:?}（留在库里，下一轮再取）");
                self.pool.withdraw(&name);
                self.summary.blocks_withdrawn += 1;
            }
        }
    }

    /// 叫会话判完就停，等它退出，写回它交回的块，放掉卡。
    fn end_the_session(&mut self) {
        let Some(session) = self.session.take() else {
            return;
        };
        session.client.drain_and_stop();
        let joined = session.thread.join();
        let cards = match joined {
            Ok((cards, _)) => cards,
            Err(panic) => std::panic::resume_unwind(panic),
        };
        // 会话已退出：每一块的回话都已交到（判完或被拒），没交到的是那一头没了，照拒收；写完再放掉卡
        for block in std::mem::take(&mut self.in_flight) {
            let answer = block
                .answer
                .recv()
                .unwrap_or(Err(ServiceRefusal::ServiceStopping));
            self.take_one_answer(block.name, &block.ordinals, answer);
        }
        drop(cards);
    }
}
