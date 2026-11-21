//! 服务主循环：把待判池、批次调度与各卡的工作线程接起来；客户端（[`ServiceClient`]）在同一份里。
//!
//! 一圈：有空着的卡就问一次调度（`crate::batch_scheduling::plan_the_next_wave`）；发一波就把派出去的各截在池里记成在判、
//! 照计划更新各卡驻留的判器、把每张卡那一份交给它的工作线程；不发就等下一个事件（客户端的请求、工作线程的回报），
//! 水位线没到的等到限时到点再问。工作线程每判完一批交回结论，池里一块的态全有了结论就把整块交回入池的那个客户端。
//! 收到「判完就停」之后不再收新块，水位线当已到（池里有态就发），池空、卡都空出来就叫工作线程停、退出。
//! 一张卡坏了（设备为别的原因丢了，`crate::card_worker` 文件头「接住 Out of Memory」）：这张卡不再排，它上面在判的那几块整块撤掉、向客户端报拒
//! （判了一半的结论不交），交一件「卡坏」（[`ServiceNotice::CardFailed`]：卡、设备丢没丢、PCI 总线号、原因；取活循环打成卡坏那一行）。
//! 池里此后没有一张没坏的卡装得下它那条流的块（新表两份加一个态的草稿区，同登记时的判法）当场报拒（[`ServiceRefusal::StreamFitsNoCard`]），
//! 之后再入池的这种块也当场拒：池里不留永远派不出去的态（弄坏开关 blocks-only-the-failed-card-could-hold-left-pending 下不拒、挂着）。
//!
//! 接住 Out of Memory：工作线程报这张卡显存不够（设备没丢，或因显存不够丢了、已重开；卡上的判器已全丢）：
//! - 这张卡驻留的判器从账上清掉，卡照常空出来、照常排；别的卡不动；
//! - 它这一波在这张卡上没交回结论的那几截撤回：每截照它的流与序号、它那一块收下时的入池时刻另入池，成一块撤回的块；它那一块留在池里，
//!   别的截（在别的卡上判的、这张卡上判完交回了的、还没派的）照旧，撤回的块判出的结论照序号记回它那一块，回话不动：每个态只判一次
//!   （弄坏开关 put-back-takes-the-whole-block 下撤整块：整块照它的序号重新入池、回话改挂新号，旧号上别的卡交回的结论不记）。
//!   撤回的块记下它在哪几张卡上显存不够过。撤回的块照常交调度，只是不派回它显存不够过的卡；它显存不够过的卡之外没有一张没坏的卡装得下这条流时，
//!   等一张腾空的卡单独判（撤回它的那张卡没坏就等那一张，坏了取头一张空着的没坏的卡；`crate::batch_scheduling::plan_a_lone_block_on_an_emptied_card`）；
//!   那张卡腾空了也装不下这条流的，它那一块整块报拒。弄坏开关 put-back-blocks-wait-for-their-own-card 下撤回的块一律只等撤回它的那张卡腾空单独判；
//! - 单独在腾空的卡上判的那一块又显存不够：它那一块整块报拒（[`ServiceRefusal::OutOfMemoryOnAnEmptiedCard`]），卡照常用，不再撤回；
//! - 一块报拒（这一节与别处的拒）撤的是收下的那一块与它撤回的每一块，回话报拒，判了一半的结论不交；
//! - 在核的那一组核对块：核的那一刻卡上还留着别的判器的，放回排头、下一回挑一张空着的卡把它驻留的判器全换出再核（不一定是显存不够的那一张）；
//!   已经腾空了还显存不够的整组报拒；
//! - 交一件「接住 Out of Memory」（[`ServiceNotice::OutOfMemoryCaught`]：卡、丢了几个判器、撤回与报拒的块、放回与报拒的核对组、原因）。
//!
//! 判法自己 panic（工作线程报判法 panic 了，设备没丢、卡上的判器已全丢）：卡不坏、照常空出来、照常排，这张卡驻留的判器从账上清掉；
//! panic 时在装哪条流的判器的，这一波里那条流的块整块报拒（[`ServiceRefusal::JudgePanicked`]）；在判哪一批的，那一批里的块整块报拒；
//! 在核哪一组的，那一组报拒；这一波别的没交回结论的截照「接住 Out of Memory」那样撤回（不记显存不够、不等腾空的卡），换出时 panic 的、
//! 在核的组没核成的照原样放回。
//!
//! 工作线程每装上一个判器，主循环照这张卡这一波的账交一件「装上」（[`ServiceNotice::JudgeLoaded`]：卡、流、表与草稿区各几 MiB、一次派活的态数、
//! 此刻驻留几个、账上还剩几 MiB）。几件都经 [`service_channel_telling_the_notices`] 交出的那一头送出；没要那一头的不送。
//!
//! 登记流时判它新装那一刻装不装得进任何一张卡（表两份加一个态的草稿区，`crate::card_memory_ledger::mebibytes_while_loading_a_judge`），
//! 装不进的当场拒：池里不留永远派不出去的态。
//!
//! 核对：客户端另可交一组核对块（[`FactsVerifierJob`]：一份材料、按次序核的几块、核对器要借的显存）。收下时它要借的比哪一张没坏的卡
//! 给判器的都大就当场拒；收下的按收的先后排着，每一圈问调度之前先给它们找空着的卡：照这张卡驻留的判器记一本显存账
//! （`crate::card_memory_ledger::CardMemoryLedger`），借得到就派；借不到就照次序换出驻留的判器（没有待判态的那几条流在前）直到借得到，
//! 空着的卡里挑换出最少的那一张；空着的卡怎么换都借不到就等下一个事件。派出去的那一组连同要换出的判器交给那张卡的工作线程，
//! 驻留的账同时去掉换出的那几个；这张卡核完报空出来之前不空着，调度不往它上派。核完的结论位整组交回入队的那个客户端；
//! 卡坏了在核的那一组报拒，卡都坏了排着的也报拒。

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::ops::Range;
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::batch_scheduling::{
    plan_a_lone_block_on_an_emptied_card, plan_the_next_wave, BatchPiece,
    BatchSchedulingConfiguration, PendingBlock, PendingBlockIdentifier, ResidentJudge,
    SchedulingInputs, WaveDecision, WavePlan,
};
use crate::break_switches::BreakSwitch;
use crate::card_memory_ledger::{
    mebibytes_of_a_judge, mebibytes_while_loading_a_judge, CardIndex, CardMemoryLedger,
    CardOfTheService,
};
use crate::card_worker::{
    card_worker_loop, CardWorkerCommand, CardWorkerReport, FactsJobForTheCard, FactsJobIdentifier,
    JudgedPiece, JudgesOnACard, PanickedWhile, WaveForTheCard,
};
use crate::facts_verifier_family::FactsVerifierMaterial;
use crate::pending_pool::{CardWorkOnAPiece, JudgedBlock, PendingPool, StateVerdict};
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
    /// 这条流新装那一刻（表两份加一个态的草稿区）装不进任何一张没坏的卡。
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
    /// 这一块（或这一组核对块）单独在一张腾空的卡上还是显存不够，或那张卡腾空了也装不下（文件头「接住 Out of Memory」）；卡照常用。
    OutOfMemoryOnAnEmptiedCard { card: CardIndex, reason: String },
    /// 判这一块（或核这一组）的时候判法自己 panic 了（文件头「判法自己 panic」）；卡照常用。
    JudgePanicked { card: CardIndex, reason: String },
    /// 服务的卡都坏了，入池的块没有卡判。
    NoWorkingCard,
    /// 服务在停，不再收新块。
    ServiceStopping,
    /// 这一组核对块的核对器要借的比哪一张没坏的卡给判器的都大。
    FactsVerifierFitsNoCard {
        verifier_mebibytes: u64,
        largest_card_mebibytes_for_the_judges: u64,
    },
}

/// 交给服务核的一组核对块（文件头「核对」）：建核对器的材料、按次序核的几块（每块是事实表里的态序号区间）、核对器要借的显存
/// （MiB，`crate::card_memory_ledger::mebibytes_of_a_facts_verifier`）。
#[derive(Clone)]
pub struct FactsVerifierJob {
    pub material: Arc<dyn FactsVerifierMaterial>,
    pub blocks: Vec<Range<u64>>,
    pub verifier_mebibytes: u64,
}

/// 一组核对块核完：在哪张卡上核的、每块每个态的结论位（与交来的块同序）、每块分到的卡上挂钟（纳秒）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedFactsJob {
    pub card: CardIndex,
    pub bits_of_each_block: Vec<Vec<u32>>,
    pub card_nanoseconds_of_each_block: Vec<u64>,
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
    SubmitFactsVerifierJob {
        job: FactsVerifierJob,
        reply: Sender<Result<VerifiedFactsJob, ServiceRefusal>>,
    },
    DrainAndStop,
}

/// 主循环交出去的几件事（文件头「接住 Out of Memory」、卡坏、装上；取活循环打成日志行，`crate::serving_from_the_library` 文件头「日志」）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ServiceNotice {
    /// 一张卡装上了一个判器：表与草稿区各占几 MiB、一次派活几个态、装上之后这张卡驻留几个判器、账上还剩几 MiB（额度减驱动那一份减驻留的）。
    JudgeLoaded {
        card: CardIndex,
        stream: StreamKey,
        table_mebibytes: u64,
        scratch_mebibytes: u64,
        states_per_dispatch: u64,
        judges_on_the_card: u64,
        free_mebibytes: u64,
    },
    /// 一张卡显存不够（设备没丢，或因显存不够丢了、已重开）：丢了几个判器；有截撤回的块与报拒的块，各写收下的那一块的流与序号、
    /// 每块一次（撤回的是这张卡上没交回结论的那几截，文件头「接住 Out of Memory」）；放回与报拒的核对组各几组；panic 的那一句。
    OutOfMemoryCaught {
        card: CardIndex,
        judges_dropped: u64,
        blocks_put_back: Vec<(StreamKey, Range<u64>)>,
        blocks_refused: Vec<(StreamKey, Range<u64>)>,
        facts_jobs_put_back: u64,
        facts_jobs_refused: u64,
        reason: String,
    },
    /// 一张卡坏了、不再排：设备丢没丢、PCI 总线号（替身卡没有）、原因。
    CardFailed {
        card: CardIndex,
        device_lost: bool,
        pci_bus_identifier: Option<String>,
        reason: String,
    },
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
    notices: Option<Sender<ServiceNotice>>,
}

/// 一对客户端与收件箱（主循环交出的几件事不送）。
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
            notices: None,
        },
    )
}

/// 一对客户端与收件箱，另交主循环交出的几件事（[`ServiceNotice`]）的那一头。
#[must_use]
pub fn service_channel_telling_the_notices<Materials>() -> (
    ServiceClient<Materials>,
    ServiceInbox<Materials>,
    Receiver<ServiceNotice>,
) {
    let (client, mut inbox) = service_channel();
    let (notices, told) = channel();
    inbox.notices = Some(notices);
    (client, inbox, told)
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

    /// 交一组核对块，不等：整组核完（或被拒）时从交回的那一头收到。服务已经退出时那一头收到的是 [`ServiceRefusal::ServiceStopping`]。
    #[must_use]
    pub fn submit_facts_verifier_job(
        &self,
        job: FactsVerifierJob,
    ) -> Receiver<Result<VerifiedFactsJob, ServiceRefusal>> {
        let (reply, answer) = channel();
        let refused_reply = reply.clone();
        if self
            .events
            .send(ServiceEvent::Request(
                ServiceRequest::SubmitFactsVerifierJob { job, reply },
            ))
            .is_err()
        {
            // 服务那一头已经没了：照拒交回，等的人不挂着
            let _ = refused_reply.send(Err(ServiceRefusal::ServiceStopping));
        }
        answer
    }

    /// 已入池的判完、已交的核对块核完就停。
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
    pub facts_verifier_jobs_verified: u64,
    pub cards_failed: Vec<CardIndex>,
    /// 接住了几回 Out of Memory（文件头「接住 Out of Memory」）。
    pub out_of_memory_caught: u64,
    pub blocks_put_back_after_running_out_of_memory: u64,
    pub blocks_refused_on_an_emptied_card: u64,
    pub facts_jobs_put_back_after_running_out_of_memory: u64,
    /// 判法自己 panic 了几回（文件头「判法自己 panic」）。
    pub judge_panics_caught: u64,
    /// 判法自己 panic 而报拒的块与核对组。
    pub blocks_refused_for_a_judge_panic: u64,
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
            let break_switches = &configuration.break_switches;
            scope.spawn(move || {
                let mut judges = make_the_judges();
                card_worker_loop(
                    index,
                    &mut judges,
                    &command_receiver,
                    break_switches,
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
            pieces_in_flight_by_card: BTreeMap::new(),
            withdrawn: BTreeSet::new(),
            put_back_parts: BTreeMap::new(),
            facts_jobs_waiting: VecDeque::new(),
            facts_job_replies: BTreeMap::new(),
            facts_jobs_on_cards: BTreeMap::new(),
            next_facts_job: 0,
            admitted: BTreeMap::new(),
            cards_the_block_ran_out_of_memory_on: BTreeMap::new(),
            waiting_for_an_emptied_card: BTreeMap::new(),
            lone_block_on_card: BTreeMap::new(),
            loading_by_card: BTreeMap::new(),
            notices: inbox.notices.clone(),
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
    /// 每张卡这一波派出去、还没交回结论的截（按派的次序）。
    pieces_in_flight_by_card: BTreeMap<CardIndex, Vec<BatchPiece>>,
    /// 报拒撤掉的块（收下的与撤回的）：之后交回的结论不记。
    withdrawn: BTreeSet<PendingBlockIdentifier>,
    /// 撤回的块（文件头「接住 Out of Memory」）：它从哪一块撤回、归哪一块收下的；判完或报拒时去掉。
    put_back_parts: BTreeMap<PendingBlockIdentifier, PutBackPart>,
    /// 收下、还没派上卡的核对组，按收的先后（接住 Out of Memory 放回的排在头）。
    facts_jobs_waiting: VecDeque<WaitingFactsJob>,
    facts_job_replies:
        BTreeMap<FactsJobIdentifier, Sender<Result<VerifiedFactsJob, ServiceRefusal>>>,
    /// 每张卡此刻在核的那一组。
    facts_jobs_on_cards: BTreeMap<CardIndex, FactsJobOnACard>,
    next_facts_job: u64,
    /// 收下、还没判完的块：流、序号、入池时刻（撤回的块照它的流与入池时刻入池）。回话挂在它的号上。
    admitted: BTreeMap<PendingBlockIdentifier, AdmittedBlock>,
    /// 撤回的块在哪几张卡上显存不够过（文件头「接住 Out of Memory」；它从的那一块的，加上撤回它的那张卡；判完或报拒时去掉）。
    cards_the_block_ran_out_of_memory_on: BTreeMap<PendingBlockIdentifier, BTreeSet<CardIndex>>,
    /// 显存不够撤回重新入池、还没再派出去的块，值是撤回它的那张卡（文件头「接住 Out of Memory」）。
    waiting_for_an_emptied_card: BTreeMap<PendingBlockIdentifier, CardIndex>,
    /// 每张卡此刻单独在判的那一块（它腾空了卡、只装那一条流的判器）。
    lone_block_on_card: BTreeMap<CardIndex, PendingBlockIdentifier>,
    /// 每张卡这一波装判器的账（「装上」那一件照它数驻留几个、账上还剩多少）。
    loading_by_card: BTreeMap<CardIndex, LoadingOnACard>,
    notices: Option<Sender<ServiceNotice>>,
    is_draining: bool,
    started_at: Instant,
    summary: ServiceRunSummary,
}

/// 收下、还没派上卡的一组核对块：接住 Out of Memory 放回来的要一张腾空的卡。
struct WaitingFactsJob {
    identifier: FactsJobIdentifier,
    job: FactsVerifierJob,
    needs_an_emptied_card: bool,
}

/// 一张卡此刻在核的那一组：留一份交出去的，接住 Out of Memory 时照它放回；派的那一刻这张卡腾空了没有。
struct FactsJobOnACard {
    identifier: FactsJobIdentifier,
    job: FactsVerifierJob,
    on_an_emptied_card: bool,
}

/// 收下的一块：流、序号、入池时刻。
#[derive(Clone, Debug)]
struct AdmittedBlock {
    stream: StreamKey,
    ordinals: Range<u64>,
    entered_at: Duration,
}

/// 一块撤回的块（文件头「接住 Out of Memory」）：从池里哪一块的一截撤回的（收下的块，或另一块撤回的块），归哪一块收下的（回话挂在它上）。
#[derive(Clone, Copy, Debug)]
struct PutBackPart {
    parent: PendingBlockIdentifier,
    owner: PendingBlockIdentifier,
}

/// 一张卡这一波装判器的账：判器能占的、此刻驻留的占的与个数（发这一波时记下换出之后留着的，每装上一个加一个）。
#[derive(Clone, Copy, Debug, Default)]
struct LoadingOnACard {
    capacity_mebibytes: u64,
    occupied_mebibytes: u64,
    judges: u64,
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
            if self.is_draining
                && self.replies.is_empty()
                && self.facts_job_replies.is_empty()
                && self.every_working_card_is_idle()
            {
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

    /// 有空着的卡就问调度，发了就再问（发过的卡不再空着），直到调度说不发。撤回的块有在等的（文件头「接住 Out of Memory」），
    /// 每张空着的卡各问一次（各自的池去掉在这张卡上显存不够过的撤回块）；没有就整组空着的卡问一次。
    fn dispatch_while_the_scheduler_says_so(
        &mut self,
        commands: &BTreeMap<CardIndex, Sender<CardWorkerCommand<Materials>>>,
        on_each_wave: &mut dyn FnMut(&WavePlan),
    ) -> WaitForTheNextEvent {
        // 迭代上界：每发一波至少一张空着的卡不再空着；不发就返回
        loop {
            self.place_the_lone_blocks(commands, on_each_wave);
            self.place_the_facts_verifier_jobs(commands);
            let idle_cards: Vec<CardOfTheService> = self
                .cards
                .iter()
                .filter(|card| self.idle.contains(&card.card) && !self.failed.contains(&card.card))
                .copied()
                .collect();
            if idle_cards.is_empty() {
                return WaitForTheNextEvent::Forever;
            }
            let groups: Vec<Vec<CardOfTheService>> = if self.are_put_back_blocks_shared() {
                idle_cards.iter().map(|card| vec![*card]).collect()
            } else {
                vec![idle_cards]
            };
            let mut wait = WaitForTheNextEvent::Forever;
            let mut dispatched = false;
            for group in groups {
                match self.decision_for_the_idle_cards(&group) {
                    WaveDecision::Dispatch(plan) => {
                        self.dispatch_the_wave(plan, commands, on_each_wave);
                        dispatched = true;
                        break;
                    }
                    WaveDecision::HoldBelowTheWatermark { wait_left, .. } => {
                        let wait_left = wait_left.max(Duration::from_millis(1));
                        wait = match wait {
                            WaitForTheNextEvent::AtMost(sooner) if sooner <= wait_left => {
                                WaitForTheNextEvent::AtMost(sooner)
                            }
                            WaitForTheNextEvent::AtMost(_) | WaitForTheNextEvent::Forever => {
                                WaitForTheNextEvent::AtMost(wait_left)
                            }
                        };
                    }
                    WaveDecision::HoldTheEmptyPool
                    | WaveDecision::HoldNothingPlaceableOnTheIdleCards => {}
                }
            }
            if !dispatched {
                return wait;
            }
        }
    }

    /// 撤回的块有在等的、且照常交调度（弄坏开关 put-back-blocks-wait-for-their-own-card 下不交）。
    fn are_put_back_blocks_shared(&self) -> bool {
        !self.waiting_for_an_emptied_card.is_empty()
            && !self
                .configuration
                .break_switches
                .is_on(BreakSwitch::PutBackBlocksWaitForTheirOwnCard)
    }

    /// 给这几张空着的卡问一次调度。池里撤回在等的块：照常交调度时（只问一张卡），在这张卡上显存不够过的去掉；不交时全去掉。
    fn decision_for_the_idle_cards(&self, idle_cards: &[CardOfTheService]) -> WaveDecision {
        let residents: Vec<ResidentJudge> = self.residents.values().flatten().copied().collect();
        let facts = self.facts_of_the_streams();
        let is_shared = self.are_put_back_blocks_shared();
        let pending: Vec<_> = self
            .pool
            .undispatched_blocks()
            .into_iter()
            .filter(|block| {
                !self.waiting_for_an_emptied_card.contains_key(&block.block)
                    || (is_shared
                        && idle_cards.iter().all(|card| {
                            !self
                                .cards_the_block_ran_out_of_memory_on
                                .get(&block.block)
                                .is_some_and(|cards| cards.contains(&card.card))
                        }))
            })
            .collect();
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
        plan_the_next_wave(
            configuration,
            &SchedulingInputs {
                idle_cards,
                residents: &residents,
                streams: &facts,
                pending: &pending,
                now: self.now(),
            },
        )
    }

    fn facts_of_the_streams(&self) -> BTreeMap<StreamKey, StreamFacts> {
        self.streams
            .iter()
            .map(|(stream, (facts, _))| (*stream, *facts))
            .collect()
    }

    fn tell(&self, notice: ServiceNotice) {
        if let Some(notices) = &self.notices {
            let _ = notices.send(notice);
        }
    }

    /// 撤回在等的块（文件头「接住 Out of Memory」）：它显存不够过的卡之外有没坏、装得下这条流的卡的，照常交调度（这里不动）；
    /// 没有的等腾空的卡单独判：撤回它的那张卡没坏就等那一张空出来，坏了取头一张空着的没坏的卡；不在池里了的不再等；
    /// 那张卡腾空了也装不下这条流的整块报拒。
    fn place_the_lone_blocks(
        &mut self,
        commands: &BTreeMap<CardIndex, Sender<CardWorkerCommand<Materials>>>,
        on_each_wave: &mut dyn FnMut(&WavePlan),
    ) {
        let waiting: Vec<(PendingBlockIdentifier, CardIndex)> = self
            .waiting_for_an_emptied_card
            .iter()
            .map(|(block, card)| (*block, *card))
            .collect();
        for (block, origin) in waiting {
            let Some(pending) = self
                .pool
                .undispatched_blocks()
                .into_iter()
                .find(|pending| pending.block == block)
            else {
                self.waiting_for_an_emptied_card.remove(&block);
                continue;
            };
            if self.are_put_back_blocks_shared() && self.another_card_could_take(&pending) {
                continue;
            }
            let chosen = self
                .cards
                .iter()
                .filter(|card| self.idle.contains(&card.card) && !self.failed.contains(&card.card))
                .find(|card| card.card == origin || self.failed.contains(&origin))
                .copied();
            let Some(card) = chosen else {
                continue;
            };
            let residents = self.residents.get(&card.card).cloned().unwrap_or_default();
            let plan = plan_a_lone_block_on_an_emptied_card(
                self.configuration,
                &card,
                &residents,
                &self.facts_of_the_streams(),
                &pending,
                self.now(),
            );
            self.waiting_for_an_emptied_card.remove(&block);
            if let Some(plan) = plan {
                self.lone_block_on_card.insert(card.card, block);
                self.dispatch_the_wave(plan, commands, on_each_wave);
            } else {
                self.refuse_the_block(
                    block,
                    ServiceRefusal::OutOfMemoryOnAnEmptiedCard {
                        card: card.card,
                        reason: "这张卡腾空了也装不下这条流的判器（新表两份加一个态的草稿区）"
                            .to_string(),
                    },
                );
            }
        }
    }

    /// 撤回在等的这一块：它显存不够过的卡之外，有没有一张没坏的卡装得下它那条流（新表两份加一个态，同登记时的判法）。
    fn another_card_could_take(&self, pending: &PendingBlock) -> bool {
        let Some((facts, _)) = self.streams.get(&pending.stream) else {
            return false;
        };
        let needed = self.judge_mebibytes_at_one_state(facts);
        let ran_out_on = self
            .cards_the_block_ran_out_of_memory_on
            .get(&pending.block);
        self.cards.iter().any(|card| {
            !self.failed.contains(&card.card)
                && !ran_out_on.is_some_and(|cards| cards.contains(&card.card))
                && needed <= card.mebibytes_for_the_judges()
        })
    }

    /// 一条流新装那一刻要的（新表两份加一个态的草稿区；弄坏开关 table-staging-not-reserved 下表一份）。
    fn judge_mebibytes_at_one_state(&self, facts: &StreamFacts) -> u64 {
        if self
            .configuration
            .break_switches
            .is_on(BreakSwitch::TableStagingNotReserved)
        {
            mebibytes_of_a_judge(facts, 1)
        } else {
            mebibytes_while_loading_a_judge(facts, 1)
        }
    }

    /// 没坏的卡里给判器的最大的那一份（卡都坏了是 0）。
    fn largest_working_card_mebibytes_for_the_judges(&self) -> u64 {
        self.cards
            .iter()
            .filter(|card| !self.failed.contains(&card.card))
            .map(CardOfTheService::mebibytes_for_the_judges)
            .max()
            .unwrap_or(0)
    }

    /// 这条流此刻没有一张没坏的卡装得下（登记时与文件头「一张卡坏了」）：交那一种拒；装得下的交 `None`。
    fn refusal_for_a_stream_no_working_card_fits(
        &self,
        stream: StreamKey,
        facts: &StreamFacts,
    ) -> Option<ServiceRefusal> {
        let judge_mebibytes_at_one_state = self.judge_mebibytes_at_one_state(facts);
        let largest_card_mebibytes_for_the_judges =
            self.largest_working_card_mebibytes_for_the_judges();
        (judge_mebibytes_at_one_state > largest_card_mebibytes_for_the_judges).then_some(
            ServiceRefusal::StreamFitsNoCard {
                stream,
                judge_mebibytes_at_one_state,
                largest_card_mebibytes_for_the_judges,
            },
        )
    }

    /// 排着的核对组照次序找空着的卡派（文件头「核对」）；排头那一组此刻派不出去就停，等下一个事件。
    fn place_the_facts_verifier_jobs(
        &mut self,
        commands: &BTreeMap<CardIndex, Sender<CardWorkerCommand<Materials>>>,
    ) {
        // 迭代上界：排着的核对组数；每一圈派出去一组，派不出去就返回
        loop {
            let Some((verifier_mebibytes, needs_an_emptied_card)) =
                self.facts_jobs_waiting.front().map(|waiting| {
                    (
                        waiting.job.verifier_mebibytes,
                        waiting.needs_an_emptied_card,
                    )
                })
            else {
                return;
            };
            let Some((card, evictions)) =
                self.card_for_a_facts_verifier(verifier_mebibytes, needs_an_emptied_card)
            else {
                return;
            };
            let waiting = self
                .facts_jobs_waiting
                .pop_front()
                .expect("排头那一组刚看过在");
            let mut on_an_emptied_card = true;
            if let Some(residents) = self.residents.get_mut(&card) {
                residents.retain(|resident| !evictions.contains(&resident.stream));
                on_an_emptied_card = residents.is_empty();
            }
            self.idle.remove(&card);
            self.facts_jobs_on_cards.insert(
                card,
                FactsJobOnACard {
                    identifier: waiting.identifier,
                    job: waiting.job.clone(),
                    on_an_emptied_card,
                },
            );
            let sent =
                commands[&card].send(CardWorkerCommand::VerifyTheFacts(FactsJobForTheCard {
                    job: waiting.identifier,
                    evictions,
                    material: waiting.job.material,
                    blocks: waiting.job.blocks,
                }));
            if sent.is_err() {
                self.take_a_failed_card(card, "工作线程已经退出".to_string(), false, None);
            }
        }
    }

    /// 给要借 `verifier_mebibytes` 的核对器挑一张空着的卡，连同要先换出的判器（文件头「核对」）：照换出的个数从少到多试，
    /// 同样多的按卡的先后；每一张照它驻留的判器（换出的除外）记一本账，借得到就是它。`must_empty_the_card`（接住 Out of Memory 放回来的那一组）
    /// 只试换出全部驻留的判器。都借不到交 `None`。弄坏开关 facts-verifier-memory-unchecked：号最小的那张空着的卡、不换出。
    fn card_for_a_facts_verifier(
        &self,
        verifier_mebibytes: u64,
        must_empty_the_card: bool,
    ) -> Option<(CardIndex, Vec<StreamKey>)> {
        let idle_cards: Vec<CardOfTheService> = self
            .cards
            .iter()
            .filter(|card| self.idle.contains(&card.card) && !self.failed.contains(&card.card))
            .copied()
            .collect();
        if self
            .configuration
            .break_switches
            .is_on(BreakSwitch::FactsVerifierMemoryUnchecked)
        {
            return idle_cards.first().map(|card| (card.card, Vec::new()));
        }
        let streams_with_pending_states: BTreeSet<StreamKey> = self
            .pool
            .undispatched_blocks()
            .iter()
            .map(|block| block.stream)
            .collect();
        let eviction_orders: Vec<(CardOfTheService, Vec<ResidentJudge>)> = idle_cards
            .into_iter()
            .map(|card| {
                let residents = self.residents.get(&card.card).cloned().unwrap_or_default();
                let (without_pending, with_pending): (Vec<ResidentJudge>, Vec<ResidentJudge>) =
                    residents.into_iter().partition(|resident| {
                        !streams_with_pending_states.contains(&resident.stream)
                    });
                (
                    card,
                    without_pending.into_iter().chain(with_pending).collect(),
                )
            })
            .collect();
        let most_residents = eviction_orders
            .iter()
            .map(|(_, order)| order.len())
            .max()
            .unwrap_or(0);
        for evicted_count in 0..=most_residents {
            for (card, order) in &eviction_orders {
                if evicted_count > order.len()
                    || (must_empty_the_card && evicted_count != order.len())
                {
                    continue;
                }
                let (evicted, kept) = order.split_at(evicted_count);
                let mut ledger = CardMemoryLedger::empty(card, &self.configuration.break_switches);
                let kept_fit = kept.iter().all(|resident| {
                    self.streams
                        .get(&resident.stream)
                        .is_some_and(|(facts, _)| {
                            ledger
                                .admit(resident.stream, facts, resident.states_per_dispatch)
                                .is_ok()
                        })
                });
                if kept_fit && ledger.lend_to_a_facts_verifier(verifier_mebibytes).is_ok() {
                    return Some((
                        card.card,
                        evicted.iter().map(|resident| resident.stream).collect(),
                    ));
                }
            }
        }
        None
    }

    /// 收下一组核对块之前判它拒不拒（文件头「核对」）：在停、卡都坏了、要借的比哪一张没坏的卡给判器的都大。
    fn refusal_of_a_facts_verifier_job(&self, verifier_mebibytes: u64) -> Option<ServiceRefusal> {
        if self.is_draining {
            return Some(ServiceRefusal::ServiceStopping);
        }
        self.refusal_for_the_working_cards(verifier_mebibytes)
    }

    /// 卡都坏了（[`ServiceRefusal::NoWorkingCard`]），或要借的比哪一张没坏的卡给判器的都大。
    fn refusal_for_the_working_cards(&self, verifier_mebibytes: u64) -> Option<ServiceRefusal> {
        let Some(largest_card_mebibytes_for_the_judges) = self
            .cards
            .iter()
            .filter(|card| !self.failed.contains(&card.card))
            .map(CardOfTheService::mebibytes_for_the_judges)
            .max()
        else {
            return Some(ServiceRefusal::NoWorkingCard);
        };
        let fits = verifier_mebibytes <= largest_card_mebibytes_for_the_judges
            || self
                .configuration
                .break_switches
                .is_on(BreakSwitch::FactsVerifierMemoryUnchecked);
        (!fits).then_some(ServiceRefusal::FactsVerifierFitsNoCard {
            verifier_mebibytes,
            largest_card_mebibytes_for_the_judges,
        })
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
        for piece in &pieces {
            self.pool
                .mark_dispatched(piece)
                .unwrap_or_else(|refusal| panic!("池拒收调度派出去的一截：{refusal:?}"));
        }
        let still_undispatched: BTreeSet<PendingBlockIdentifier> = self
            .pool
            .undispatched_blocks()
            .into_iter()
            .map(|pending| pending.block)
            .collect();
        for piece in pieces {
            if !still_undispatched.contains(&piece.block) {
                self.waiting_for_an_emptied_card.remove(&piece.block);
            }
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
            let evicted: BTreeSet<StreamKey> = card_plan.evictions.iter().copied().collect();
            let staying: Vec<ResidentJudge> = self
                .residents
                .get(&card_plan.card)
                .map(|residents| {
                    residents
                        .iter()
                        .filter(|resident| !evicted.contains(&resident.stream))
                        .copied()
                        .collect()
                })
                .unwrap_or_default();
            self.loading_by_card.insert(
                card_plan.card,
                LoadingOnACard {
                    capacity_mebibytes: card_plan.resident_after_the_wave().capacity_mebibytes(),
                    occupied_mebibytes: staying
                        .iter()
                        .map(|resident| {
                            mebibytes_of_a_judge(
                                &self.streams[&resident.stream].0,
                                resident.states_per_dispatch,
                            )
                        })
                        .sum(),
                    judges: u64::try_from(staying.len()).expect("装得进 u64"),
                },
            );
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
            self.pieces_in_flight_by_card.insert(
                card_plan.card,
                card_plan
                    .batches
                    .iter()
                    .flat_map(|batch| batch.pieces.iter().cloned())
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
                self.take_a_failed_card(card, "工作线程已经退出".to_string(), false, None);
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
            }) => match self.admit_the_block(stream, ordinals.clone()) {
                Ok((block, entered_at)) => {
                    self.replies.insert(block, reply);
                    self.admitted.insert(
                        block,
                        AdmittedBlock {
                            stream,
                            ordinals,
                            entered_at,
                        },
                    );
                }
                Err(refusal) => {
                    let _ = reply.send(Err(refusal));
                }
            },
            ServiceEvent::Request(ServiceRequest::SubmitFactsVerifierJob { job, reply }) => {
                if let Some(refusal) = self.refusal_of_a_facts_verifier_job(job.verifier_mebibytes)
                {
                    let _ = reply.send(Err(refusal));
                } else {
                    let identifier = FactsJobIdentifier(self.next_facts_job);
                    self.next_facts_job += 1;
                    self.facts_jobs_waiting.push_back(WaitingFactsJob {
                        identifier,
                        job,
                        needs_an_emptied_card: false,
                    });
                    self.facts_job_replies.insert(identifier, reply);
                }
            }
            ServiceEvent::Request(ServiceRequest::DrainAndStop) => {
                self.is_draining = true;
            }
            ServiceEvent::Worker(CardWorkerReport::BatchJudged { card, pieces }) => {
                for JudgedPiece {
                    piece,
                    verdicts,
                    work,
                } in pieces
                {
                    if let Some(in_flight) = self.pieces_in_flight_by_card.get_mut(&card) {
                        in_flight.retain(|still_in_flight| *still_in_flight != piece);
                    }
                    self.take_the_verdicts_of_a_piece(&piece, card, verdicts, work);
                }
            }
            ServiceEvent::Worker(CardWorkerReport::JudgeLoaded {
                card,
                stream,
                states_per_dispatch,
            }) => self.take_a_loaded_judge(card, stream, states_per_dispatch),
            ServiceEvent::Worker(CardWorkerReport::OutOfMemoryCaught {
                card,
                judges_dropped,
                reason,
            }) => self.take_an_out_of_memory(card, judges_dropped, reason),
            ServiceEvent::Worker(CardWorkerReport::JudgePanicked {
                card,
                panicked_while,
                reason,
                ..
            }) => self.take_a_judge_panic(card, &panicked_while, &reason),
            ServiceEvent::Worker(CardWorkerReport::FactsVerified {
                card,
                job,
                bits_of_each_block,
                card_nanoseconds_of_each_block,
            }) => {
                self.facts_jobs_on_cards.remove(&card);
                self.summary.facts_verifier_jobs_verified += 1;
                if let Some(reply) = self.facts_job_replies.remove(&job) {
                    let _ = reply.send(Ok(VerifiedFactsJob {
                        card,
                        bits_of_each_block,
                        card_nanoseconds_of_each_block,
                    }));
                }
            }
            ServiceEvent::Worker(CardWorkerReport::RoundFinished { card }) => {
                self.pieces_in_flight_by_card.remove(&card);
                self.lone_block_on_card.remove(&card);
                self.loading_by_card.remove(&card);
                self.idle.insert(card);
            }
            ServiceEvent::Worker(CardWorkerReport::CardFailed {
                card,
                reason,
                device_lost,
                pci_bus_identifier,
            }) => {
                self.take_a_failed_card(card, reason, device_lost, pci_bus_identifier);
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
        if let Some(refusal) = self.refusal_for_a_stream_no_working_card_fits(stream, &facts) {
            return Err(refusal);
        }
        self.streams.insert(stream, (facts, materials));
        Ok(())
    }

    /// 收一块入池：交回它的号与入池时刻。
    fn admit_the_block(
        &mut self,
        stream: StreamKey,
        ordinals: Range<u64>,
    ) -> Result<(PendingBlockIdentifier, Duration), ServiceRefusal> {
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
        if !self
            .configuration
            .break_switches
            .is_on(BreakSwitch::BlocksOnlyTheFailedCardCouldHoldLeftPending)
        {
            if let Some(refusal) = self.refusal_for_a_stream_no_working_card_fits(stream, facts) {
                return Err(refusal);
            }
        }
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
            .map(|block| (block, now))
            .map_err(|_| ServiceRefusal::OrdinalsOutsideTheStream {
                stream,
                ordinals,
                state_count: facts.state_count,
            })
    }

    /// 池里一块归哪一块收下的：撤回的块归它的 `owner`，别的就是它自己。
    fn owner_of(&self, block: PendingBlockIdentifier) -> PendingBlockIdentifier {
        self.put_back_parts
            .get(&block)
            .map_or(block, |part| part.owner)
    }

    /// 记一截的结论：记进它那一块；那一块是撤回的块的，同样的序号再依次记进它从哪一块撤回的，一直记到收下的那一块
    /// （文件头「接住 Out of Memory」）。收下的那一块判完了交回客户端；撤回的块记满了从池里删掉，不交回。
    fn take_the_verdicts_of_a_piece(
        &mut self,
        piece: &BatchPiece,
        card: CardIndex,
        verdicts: Vec<StateVerdict>,
        work: CardWorkOnAPiece,
    ) {
        if self.withdrawn.contains(&piece.block) {
            return;
        }
        let mut lineage = vec![piece.block];
        let mut from = piece.block;
        // 迭代上界：撤回的块的号都比它从的那一块大（入池时新发的），一路往小走到收下的那一块
        while let Some(part) = self.put_back_parts.get(&from) {
            from = part.parent;
            lineage.push(from);
        }
        for block in lineage {
            let piece_of_the_block = BatchPiece {
                block,
                ordinals: piece.ordinals.clone(),
            };
            match self
                .pool
                .record_verdicts(&piece_of_the_block, card, verdicts.clone(), work)
            {
                Ok(None) => {}
                Ok(Some(judged)) if self.put_back_parts.contains_key(&judged.block) => {
                    self.put_back_parts.remove(&judged.block);
                    self.cards_the_block_ran_out_of_memory_on
                        .remove(&judged.block);
                }
                Ok(Some(judged)) => {
                    self.summary.blocks_judged += 1;
                    self.admitted.remove(&judged.block);
                    if let Some(reply) = self.replies.remove(&judged.block) {
                        let _ = reply.send(Ok(judged));
                    }
                }
                Err(refusal) => {
                    panic!("池拒收工作线程交回的结论（工作线程、调度与池对不上）：{refusal:?}");
                }
            }
        }
    }

    /// 一张卡坏了：不再排它、忘掉它上面的判器，它这一波在判的块整块撤掉、在核的核对组、此后哪一张没坏的卡都放不下的排着的核对组
    /// 向客户端报拒；交一件「卡坏」（弄坏开关 card-failure-not-told 下不交）。
    fn take_a_failed_card(
        &mut self,
        card: CardIndex,
        reason: String,
        device_lost: bool,
        pci_bus_identifier: Option<String>,
    ) {
        self.failed.insert(card);
        self.idle.remove(&card);
        self.residents.remove(&card);
        self.lone_block_on_card.remove(&card);
        self.loading_by_card.remove(&card);
        self.summary.cards_failed.push(card);
        if !self
            .configuration
            .break_switches
            .is_on(BreakSwitch::CardFailureNotTold)
        {
            self.tell(ServiceNotice::CardFailed {
                card,
                device_lost,
                pci_bus_identifier,
                reason: reason.clone(),
            });
        }
        if let Some(on_card) = self.facts_jobs_on_cards.remove(&card) {
            if let Some(reply) = self.facts_job_replies.remove(&on_card.identifier) {
                let _ = reply.send(Err(ServiceRefusal::CardFailed {
                    card,
                    reason: reason.clone(),
                }));
            }
        }
        let waiting = std::mem::take(&mut self.facts_jobs_waiting);
        for waiting_job in waiting {
            match self.refusal_for_the_working_cards(waiting_job.job.verifier_mebibytes) {
                None => self.facts_jobs_waiting.push_back(waiting_job),
                Some(refused_for_the_cards) => {
                    let refusal = if refused_for_the_cards == ServiceRefusal::NoWorkingCard {
                        ServiceRefusal::CardFailed {
                            card,
                            reason: format!("服务的卡都坏了（最后一张的原因：{reason}）"),
                        }
                    } else {
                        refused_for_the_cards
                    };
                    if let Some(reply) = self.facts_job_replies.remove(&waiting_job.identifier) {
                        let _ = reply.send(Err(refusal));
                    }
                }
            }
        }
        for piece in self
            .pieces_in_flight_by_card
            .remove(&card)
            .unwrap_or_default()
        {
            self.refuse_the_block(
                piece.block,
                ServiceRefusal::CardFailed {
                    card,
                    reason: reason.clone(),
                },
            );
        }
        if self
            .cards
            .iter()
            .all(|other| self.failed.contains(&other.card))
        {
            for block in self.pool.undispatched_blocks() {
                self.waiting_for_an_emptied_card.remove(&block.block);
                self.refuse_the_block(
                    block.block,
                    ServiceRefusal::CardFailed {
                        card,
                        reason: format!("服务的卡都坏了（最后一张的原因：{reason}）"),
                    },
                );
            }
        } else if !self
            .configuration
            .break_switches
            .is_on(BreakSwitch::BlocksOnlyTheFailedCardCouldHoldLeftPending)
        {
            for block in self.pool.undispatched_blocks() {
                let refusal = self.streams.get(&block.stream).and_then(|(facts, _)| {
                    self.refusal_for_a_stream_no_working_card_fits(block.stream, facts)
                });
                if let Some(refusal) = refusal {
                    self.waiting_for_an_emptied_card.remove(&block.block);
                    self.refuse_the_block(block.block, refusal);
                }
            }
        }
    }

    /// `block` 归的那一块收下的整块撤掉、向客户端报 `refusal`（判了一半的结论不交）：连同它撤回的每一块从池里撤掉，之后交回的结论不记；
    /// 那一块已不在了（判完交回了、撤过了）的不动，交 `false`。
    fn refuse_the_block(&mut self, block: PendingBlockIdentifier, refusal: ServiceRefusal) -> bool {
        let owner = self.owner_of(block);
        if self.admitted.remove(&owner).is_none() {
            return false;
        }
        let parts: Vec<PendingBlockIdentifier> = self
            .put_back_parts
            .iter()
            .filter(|(_, part)| part.owner == owner)
            .map(|(part, _)| *part)
            .collect();
        for taken_off in std::iter::once(owner).chain(parts) {
            self.pool.withdraw_the_block(taken_off);
            self.withdrawn.insert(taken_off);
            self.put_back_parts.remove(&taken_off);
            self.cards_the_block_ran_out_of_memory_on.remove(&taken_off);
            self.waiting_for_an_emptied_card.remove(&taken_off);
        }
        if let Some(reply) = self.replies.remove(&owner) {
            let _ = reply.send(Err(refusal));
        }
        true
    }

    /// 撤回一截（文件头「接住 Out of Memory」「判法自己 panic」）：照它的流与序号、它那一块收下时的入池时刻另入池成一块撤回的块，
    /// 它那一块与别的截不动（撤回的块判出的结论照序号记回去，[`Self::take_the_verdicts_of_a_piece`]）。撤回的块在哪几张卡上显存不够过：
    /// 它从的那一块的，加上 `ran_out_of_memory_on`（显存不够撤回它的那张卡），并记成撤回在等的块（弄坏开关 retracted-block-shares-the-next-wave
    /// 下不记成在等）；判法 panic 撤回的是 `None`，照常交调度。它那一块已不在了的不动，交 `None`；撤回了交那一块收下时的流与序号。
    /// 弄坏开关 put-back-takes-the-whole-block 下撤整块（[`Self::put_back_the_whole_block`]）。
    fn put_back_a_piece(
        &mut self,
        piece: &BatchPiece,
        ran_out_of_memory_on: Option<CardIndex>,
    ) -> Option<(StreamKey, Range<u64>)> {
        if self
            .configuration
            .break_switches
            .is_on(BreakSwitch::PutBackTakesTheWholeBlock)
        {
            return self.put_back_the_whole_block(piece.block, ran_out_of_memory_on);
        }
        let owner = self.owner_of(piece.block);
        let admitted = self.admitted.get(&owner).cloned()?;
        let part = self
            .pool
            .add_block(admitted.stream, piece.ordinals.clone(), admitted.entered_at)
            .unwrap_or_else(|refusal| panic!("派出去过的一截不空，池却拒收：{refusal:?}"));
        self.put_back_parts.insert(
            part,
            PutBackPart {
                parent: piece.block,
                owner,
            },
        );
        let mut cards_ran_out_on = self
            .cards_the_block_ran_out_of_memory_on
            .get(&piece.block)
            .cloned()
            .unwrap_or_default();
        if let Some(origin) = ran_out_of_memory_on {
            cards_ran_out_on.insert(origin);
            if !self
                .configuration
                .break_switches
                .is_on(BreakSwitch::RetractedBlockSharesTheNextWave)
            {
                self.waiting_for_an_emptied_card.insert(part, origin);
            }
        }
        if !cards_ran_out_on.is_empty() {
            self.cards_the_block_ran_out_of_memory_on
                .insert(part, cards_ran_out_on);
        }
        Some((admitted.stream, admitted.ordinals))
    }

    /// 弄坏开关 put-back-takes-the-whole-block 下的撤回：`block` 整块从池里撤掉、记进 `withdrawn`（旧号上别的卡交回的结论不记），
    /// 照整块的流、序号与入池时刻重新入池，回话与它显存不够过的卡改挂到新的号上。这个开关下不另起撤回的块，`block` 就是收下的那一块；
    /// 已不在了的（同一块的另一截先撤过、已报拒、已判完）不动，交 `None`。
    fn put_back_the_whole_block(
        &mut self,
        block: PendingBlockIdentifier,
        ran_out_of_memory_on: Option<CardIndex>,
    ) -> Option<(StreamKey, Range<u64>)> {
        let admitted = self.admitted.get(&block).cloned()?;
        self.pool.withdraw_the_block(block)?;
        self.withdrawn.insert(block);
        self.admitted.remove(&block);
        let again = self
            .pool
            .add_block(
                admitted.stream,
                admitted.ordinals.clone(),
                admitted.entered_at,
            )
            .unwrap_or_else(|refusal| panic!("撤回的块收过一回、不空，池却拒收：{refusal:?}"));
        if let Some(reply) = self.replies.remove(&block) {
            self.replies.insert(again, reply);
        }
        let mut cards_ran_out_on = self
            .cards_the_block_ran_out_of_memory_on
            .remove(&block)
            .unwrap_or_default();
        if let Some(origin) = ran_out_of_memory_on {
            cards_ran_out_on.insert(origin);
            if !self
                .configuration
                .break_switches
                .is_on(BreakSwitch::RetractedBlockSharesTheNextWave)
            {
                self.waiting_for_an_emptied_card.insert(again, origin);
            }
        }
        if !cards_ran_out_on.is_empty() {
            self.cards_the_block_ran_out_of_memory_on
                .insert(again, cards_ran_out_on);
        }
        let put_back = (admitted.stream, admitted.ordinals.clone());
        self.admitted.insert(again, admitted);
        Some(put_back)
    }

    /// 一张卡显存不够、设备没丢（文件头「接住 Out of Memory」）：清它驻留的判器，撤回它这一波没交回结论的截或报拒它们那一块，放回或报拒在核的核对组，
    /// 交一件「接住 Out of Memory」。卡不进坏卡，随后的「空出来了」照常收。
    fn take_an_out_of_memory(&mut self, card: CardIndex, judges_dropped: usize, reason: String) {
        let configuration = self.configuration;
        let breaks = &configuration.break_switches;
        self.summary.out_of_memory_caught += 1;
        if !breaks.is_on(BreakSwitch::ResidentsKeptAfterRunningOutOfMemory) {
            self.residents.remove(&card);
        }
        self.loading_by_card.remove(&card);
        let lone = self.lone_block_on_card.remove(&card);
        let mut blocks_put_back = Vec::new();
        let mut blocks_refused = Vec::new();
        let mut pieces_put_back = Vec::new();
        for piece in self
            .pieces_in_flight_by_card
            .remove(&card)
            .unwrap_or_default()
        {
            let Some(admitted) = self.admitted.get(&self.owner_of(piece.block)).cloned() else {
                continue;
            };
            let refused = if lone == Some(piece.block)
                && !breaks.is_on(BreakSwitch::OutOfMemoryOnAnEmptiedCardRetried)
            {
                Some(ServiceRefusal::OutOfMemoryOnAnEmptiedCard {
                    card,
                    reason: reason.clone(),
                })
            } else if breaks.is_on(BreakSwitch::BlocksOfTheOutOfMemoryWaveRefused) {
                Some(ServiceRefusal::CardFailed {
                    card,
                    reason: reason.clone(),
                })
            } else {
                None
            };
            match refused {
                Some(refusal) => {
                    if self.refuse_the_block(piece.block, refusal) {
                        self.summary.blocks_refused_on_an_emptied_card += 1;
                        blocks_refused.push((admitted.stream, admitted.ordinals));
                    }
                }
                None => pieces_put_back.push(piece),
            }
        }
        // 报拒的先撤：同一块收下的另有一截在这张卡上的，不再撤回；同一块收下的几截都撤回的，那一件里只列一次
        let mut owners_put_back = BTreeSet::new();
        for piece in pieces_put_back {
            let owner = self.owner_of(piece.block);
            let Some(put_back) = self.put_back_a_piece(&piece, Some(card)) else {
                continue;
            };
            if owners_put_back.insert(owner) {
                self.summary.blocks_put_back_after_running_out_of_memory += 1;
                blocks_put_back.push(put_back);
            }
        }
        let mut facts_jobs_put_back = 0;
        let mut facts_jobs_refused = 0;
        if let Some(on_card) = self.facts_jobs_on_cards.remove(&card) {
            if on_card.on_an_emptied_card
                || breaks.is_on(BreakSwitch::FactsJobOfTheOutOfMemoryRoundRefused)
            {
                facts_jobs_refused += 1;
                if let Some(reply) = self.facts_job_replies.remove(&on_card.identifier) {
                    let _ = reply.send(Err(ServiceRefusal::OutOfMemoryOnAnEmptiedCard {
                        card,
                        reason: reason.clone(),
                    }));
                }
            } else {
                facts_jobs_put_back += 1;
                self.summary.facts_jobs_put_back_after_running_out_of_memory += 1;
                self.facts_jobs_waiting.push_front(WaitingFactsJob {
                    identifier: on_card.identifier,
                    job: on_card.job,
                    needs_an_emptied_card: true,
                });
            }
        }
        if !breaks.is_on(BreakSwitch::OutOfMemoryCaughtNotTold) {
            self.tell(ServiceNotice::OutOfMemoryCaught {
                card,
                judges_dropped: u64::try_from(judges_dropped).expect("装得进 u64"),
                blocks_put_back,
                blocks_refused,
                facts_jobs_put_back,
                facts_jobs_refused,
                reason,
            });
        }
    }

    /// 判法自己 panic 了、设备没丢（文件头「判法自己 panic」）：清这张卡驻留的判器；panic 时在做的那一步牵涉的块（装的那条流在这一波里的块、
    /// 判的那一批里的块）与核的那一组报拒，这一波别的没交回结论的截撤回，换出时 panic 的核对组放回排头。卡不进坏卡，随后的「空出来了」照常收。
    fn take_a_judge_panic(
        &mut self,
        card: CardIndex,
        panicked_while: &PanickedWhile,
        reason: &str,
    ) {
        self.summary.judge_panics_caught += 1;
        self.residents.remove(&card);
        self.loading_by_card.remove(&card);
        self.lone_block_on_card.remove(&card);
        let in_flight = self
            .pieces_in_flight_by_card
            .remove(&card)
            .unwrap_or_default();
        let (culprits, others): (Vec<BatchPiece>, Vec<BatchPiece>) = in_flight
            .into_iter()
            .partition(|piece| match panicked_while {
                PanickedWhile::Loading(stream) => self
                    .admitted
                    .get(&self.owner_of(piece.block))
                    .is_some_and(|admitted| admitted.stream == *stream),
                PanickedWhile::Judging(pieces) => pieces
                    .iter()
                    .any(|panicked_on| panicked_on.block == piece.block),
                PanickedWhile::Evicting | PanickedWhile::Verifying(_) => false,
            });
        let refusal = ServiceRefusal::JudgePanicked {
            card,
            reason: reason.to_string(),
        };
        for piece in culprits {
            if self.refuse_the_block(piece.block, refusal.clone()) {
                self.summary.blocks_refused_for_a_judge_panic += 1;
            }
        }
        // 报拒的先撤：同一块收下的另有一截不牵涉的，不再撤回
        for piece in others {
            self.put_back_a_piece(&piece, None);
        }
        if let Some(on_card) = self.facts_jobs_on_cards.remove(&card) {
            if matches!(panicked_while, PanickedWhile::Verifying(_)) {
                self.summary.blocks_refused_for_a_judge_panic += 1;
                if let Some(reply) = self.facts_job_replies.remove(&on_card.identifier) {
                    let _ = reply.send(Err(refusal));
                }
            } else {
                self.facts_jobs_waiting.push_front(WaitingFactsJob {
                    identifier: on_card.identifier,
                    job: on_card.job,
                    needs_an_emptied_card: false,
                });
            }
        }
    }

    /// 工作线程装上了一个判器：照这张卡这一波的账加上它，交一件「装上」（弄坏开关 judge-loaded-not-told 下不交）。
    fn take_a_loaded_judge(
        &mut self,
        card: CardIndex,
        stream: StreamKey,
        states_per_dispatch: u64,
    ) {
        let Some((facts, _)) = self.streams.get(&stream) else {
            return;
        };
        let facts = *facts;
        let judge_mebibytes = mebibytes_of_a_judge(&facts, states_per_dispatch);
        let loading = self.loading_by_card.entry(card).or_default();
        loading.occupied_mebibytes += judge_mebibytes;
        loading.judges += 1;
        let notice = ServiceNotice::JudgeLoaded {
            card,
            stream,
            table_mebibytes: facts.table_mebibytes,
            scratch_mebibytes: judge_mebibytes - facts.table_mebibytes,
            states_per_dispatch,
            judges_on_the_card: loading.judges,
            free_mebibytes: loading
                .capacity_mebibytes
                .saturating_sub(loading.occupied_mebibytes),
        };
        if !self
            .configuration
            .break_switches
            .is_on(BreakSwitch::JudgeLoadedNotTold)
        {
            self.tell(notice);
        }
    }
}
