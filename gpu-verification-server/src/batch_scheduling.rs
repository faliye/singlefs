//! 批次调度：纯函数，同一份输入出同一份计划（输入先排成规范次序：卡按号、驻留的判器按卡与流、池里的块按入池先后）。
//!
//! 一波是给每张空着的卡排的一轮：这张卡上服务的每条流各派一批（一次派活），批按流的入池先后排；卡做完这一轮再空出来、等下一波。
//! 一条流的批因此在几波之间与同卡别的流轮流派，长流按批切开、不一口气派完。排一张卡分两遍，两遍都按卡号走：
//! - 驻留那一遍：已经在这张卡上有判器、池里还有它的态的流，先派回这张卡（不重传表、不重编管线），一次派活的态数照装的时候定的；
//! - 新装那一遍：池里还剩态的流按入池先后挑进来，挑到装不下为止（每条至少装得下表与一个态），剩下的显存在新装的几条之间均分
//!   （一条流的态数少于分到的就取它的态数，多出来的再分给别的）；腾显存时只换出这一波在这张卡上没有批的判器，大的先换。
//!
//! 不饿死：一条流的最老待判等过配置的限时、不在任何卡上驻留、哪张空着的卡在留住有待判态的判器之后都装不下它，就挑头一张腾得出的空卡
//! 整卡重装：这张卡上的判器全换出，这条流与原来在这张卡上有待判态的那几条一起均分（原来那几条的表重传一次）。
//! 水位线：计划里每张空着的卡都有批、每一批都装满了这个判器的一次派活就发；不满而池里最老的待判等过限时也发；池空不发。
//! 越过显存额度的计划构造不出来：每张卡这一波之后驻留的判器只经 `CardMemoryLedger` 收进计划（`crate::card_memory_ledger`）。

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::ops::Range;
use std::time::Duration;

use crate::break_switches::{BreakSwitch, BreakSwitches};
use crate::card_memory_ledger::{
    bytes_of_one_state_on_the_card, mebibytes_of_a_judge, mebibytes_of_one_state_rounded_up,
    states_fitting_in, CardIndex, CardMemoryLedger, CardOfTheService, ResidentJudgesWithinTheQuota,
    BYTES_PER_MEBIBYTE,
};
use crate::stream_identity::{StreamFacts, StreamKey};

/// 池里一块待判的态的号：入池时按先后发，越小越早（调度的「入池先后」照它排）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PendingBlockIdentifier(pub u64);

/// 一张卡上驻留着的判器：一次派活带几个态是装的时候定的。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResidentJudge {
    pub card: CardIndex,
    pub stream: StreamKey,
    pub states_per_dispatch: u64,
}

/// 池里一块还没派完的态：`undispatched` 是它还没派出去的那一截（一块只从前面往后派，没派的总是连着的一截）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingBlock {
    pub block: PendingBlockIdentifier,
    pub stream: StreamKey,
    /// 入池时刻（从服务起来算）。
    pub entered_at: Duration,
    pub undispatched: Range<u64>,
}

/// 一批里的一截：哪一块的哪几个态（序号是整条流里的序号）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BatchPiece {
    pub block: PendingBlockIdentifier,
    pub ordinals: Range<u64>,
}

/// 一批：一条流在一张卡上的一次派活。几截按入池先后接起来，合计不超过判器的 `states_per_dispatch`。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Batch {
    pub stream: StreamKey,
    /// 判这一批的判器一次派活带几个态（判器装的时候定的）。
    pub states_per_dispatch: u64,
    pub pieces: Vec<BatchPiece>,
}

impl Batch {
    /// 这一批一共几个态。
    #[must_use]
    pub fn state_count(&self) -> u64 {
        self.pieces
            .iter()
            .map(|piece| piece.ordinals.end - piece.ordinals.start)
            .sum()
    }
}

/// 这一波要在卡上新装的判器。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JudgeToLoad {
    pub stream: StreamKey,
    pub states_per_dispatch: u64,
}

/// 一张卡这一波做什么：先按次序换出、再按次序装上，再按次序判各批。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardPlanOfTheWave {
    pub card: CardIndex,
    pub evictions: Vec<StreamKey>,
    pub loads: Vec<JudgeToLoad>,
    pub batches: Vec<Batch>,
    resident_after_the_wave: ResidentJudgesWithinTheQuota,
}

impl CardPlanOfTheWave {
    /// 这一波之后这张卡上驻留的判器（经显存账收的，合计不超过额度减驱动那一份）。
    #[must_use]
    pub fn resident_after_the_wave(&self) -> &ResidentJudgesWithinTheQuota {
        &self.resident_after_the_wave
    }
}

/// 一波的计划：每张空着的卡一份，按卡号排。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WavePlan {
    pub cards: Vec<CardPlanOfTheWave>,
}

impl WavePlan {
    /// 这一波一共几批。
    #[must_use]
    pub fn batch_count(&self) -> usize {
        self.cards.iter().map(|card| card.batches.len()).sum()
    }

    /// 这一波派出去的每一截，按卡、批的次序。
    pub fn pieces(&self) -> impl Iterator<Item = &BatchPiece> {
        self.cards
            .iter()
            .flat_map(|card| card.batches.iter())
            .flat_map(|batch| batch.pieces.iter())
    }
}

/// 这一刻发不发、发什么。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WaveDecision {
    /// 池里没有没派出去的态。
    HoldTheEmptyPool,
    /// 池里的态填不满每张空着的卡，最老的待判也没等过限时：`wait_left` 之后再问。
    HoldBelowTheWatermark {
        waited_longest: Duration,
        wait_left: Duration,
    },
    /// 池里有态，可这一刻没有空着的卡装得下哪一条（卡都在忙，或空着的卡装不下池里的流）：等卡做完一轮再问。
    HoldNothingPlaceableOnTheIdleCards,
    Dispatch(WavePlan),
}

/// 批次调度的配置：限时写在服务的配置里（`crate::service_configuration`），不写死；弄坏开关只给测试用。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BatchSchedulingConfiguration {
    /// 池里的态不够填满每张空着的卡时，最老的待判最多等多久就发。也是不饿死的限时：等过它的流可以腾卡。
    pub oldest_pending_wait_limit: Duration,
    pub break_switches: BreakSwitches,
}

/// 调度的输入。
#[derive(Clone, Copy, Debug)]
pub struct SchedulingInputs<'inputs> {
    /// 此刻空着、这一波要排的卡。
    pub idle_cards: &'inputs [CardOfTheService],
    /// 每张卡上驻留着的判器（忙着的卡上的也在里面：不饿死那一条看一条流在不在任何卡上驻留）。
    pub residents: &'inputs [ResidentJudge],
    /// 池里每条流的事实。
    pub streams: &'inputs BTreeMap<StreamKey, StreamFacts>,
    /// 池里还没派完的块。
    pub pending: &'inputs [PendingBlock],
    /// 此刻（从服务起来算）。
    pub now: Duration,
}

/// 这一刻发不发、发什么。
///
/// # Panics
/// 池里有一条流不在 `streams` 里；算出的计划越过一张卡的显存额度（批次调度自己错了，不许带着越界的计划往下走）。
#[must_use]
pub fn plan_the_next_wave(
    configuration: &BatchSchedulingConfiguration,
    inputs: &SchedulingInputs<'_>,
) -> WaveDecision {
    let Some(entered_earliest) = inputs
        .pending
        .iter()
        .filter(|block| !block.undispatched.is_empty())
        .map(|block| block.entered_at)
        .min()
    else {
        return WaveDecision::HoldTheEmptyPool;
    };
    let waited_longest = inputs.now.saturating_sub(entered_earliest);
    let (plan, is_every_card_filled) =
        WaveBuilding::new(configuration, inputs).plan_every_idle_card();
    if plan.batch_count() == 0 {
        return WaveDecision::HoldNothingPlaceableOnTheIdleCards;
    }
    let breaks = &configuration.break_switches;
    if is_every_card_filled || breaks.is_on(BreakSwitch::WatermarkIgnored) {
        return WaveDecision::Dispatch(plan);
    }
    if waited_longest >= configuration.oldest_pending_wait_limit
        && !breaks.is_on(BreakSwitch::WaitLimitIgnored)
    {
        return WaveDecision::Dispatch(plan);
    }
    WaveDecision::HoldBelowTheWatermark {
        waited_longest,
        wait_left: configuration
            .oldest_pending_wait_limit
            .saturating_sub(waited_longest),
    }
}

/// 一条流在池里还没派的态，按入池先后。
struct StreamQueue {
    first_entry: PendingBlockIdentifier,
    oldest_entered_at: Duration,
    pieces: VecDeque<BatchPiece>,
}

impl StreamQueue {
    fn states_left(&self) -> u64 {
        self.pieces
            .iter()
            .map(|piece| piece.ordinals.end - piece.ordinals.start)
            .sum()
    }
}

/// 排一波时的中间账：各流还剩的态、各流的先后、各卡驻留的判器。
struct WaveBuilding<'inputs> {
    breaks: &'inputs BreakSwitches,
    oldest_pending_wait_limit: Duration,
    now: Duration,
    streams: &'inputs BTreeMap<StreamKey, StreamFacts>,
    idle_cards: Vec<CardOfTheService>,
    queues: BTreeMap<StreamKey, StreamQueue>,
    /// 流的先后（下标越小越先）：入池先后，同一块号不会落在两条流上。
    stream_order: Vec<StreamKey>,
    residents_by_card: BTreeMap<CardIndex, Vec<ResidentJudge>>,
    resident_anywhere: BTreeSet<StreamKey>,
}

/// 一张卡这一波排到的：换出、装上、各批与这一波之后驻留的（流、一次派活的态数）。
#[derive(Default)]
struct CardWaveDraft {
    evictions: Vec<StreamKey>,
    loads: Vec<JudgeToLoad>,
    batches: Vec<Batch>,
    /// 整卡重装（不饿死那一条）：这张卡新装那一遍不再挑流。
    is_rebuilt_for_a_starving_stream: bool,
}

impl<'inputs> WaveBuilding<'inputs> {
    fn new(
        configuration: &'inputs BatchSchedulingConfiguration,
        inputs: &SchedulingInputs<'inputs>,
    ) -> Self {
        let breaks = &configuration.break_switches;
        let is_input_order_followed = breaks.is_on(BreakSwitch::InputOrderFollowed);
        let mut idle_cards = inputs.idle_cards.to_vec();
        let mut pending: Vec<&PendingBlock> = inputs
            .pending
            .iter()
            .filter(|block| !block.undispatched.is_empty())
            .collect();
        if !is_input_order_followed {
            idle_cards.sort_by_key(|card| card.card);
            pending.sort_by_key(|block| block.block);
        }
        let mut queues: BTreeMap<StreamKey, StreamQueue> = BTreeMap::new();
        let mut first_seen: Vec<StreamKey> = Vec::new();
        for block in pending {
            assert!(
                inputs.streams.contains_key(&block.stream),
                "池里的 {:?} 不在调度的流表里",
                block.stream
            );
            let queue = queues.entry(block.stream).or_insert_with(|| {
                first_seen.push(block.stream);
                StreamQueue {
                    first_entry: block.block,
                    oldest_entered_at: block.entered_at,
                    pieces: VecDeque::new(),
                }
            });
            queue.first_entry = queue.first_entry.min(block.block);
            queue.oldest_entered_at = queue.oldest_entered_at.min(block.entered_at);
            queue.pieces.push_back(BatchPiece {
                block: block.block,
                ordinals: block.undispatched.clone(),
            });
        }
        let stream_order = if is_input_order_followed {
            first_seen
        } else if breaks.is_on(BreakSwitch::PoolOrderIgnored) {
            queues.keys().copied().collect()
        } else {
            let mut by_entry: Vec<StreamKey> = queues.keys().copied().collect();
            by_entry.sort_by_key(|stream| (queues[stream].first_entry, *stream));
            by_entry
        };
        let mut residents_by_card: BTreeMap<CardIndex, Vec<ResidentJudge>> = BTreeMap::new();
        for resident in inputs.residents {
            residents_by_card
                .entry(resident.card)
                .or_default()
                .push(*resident);
        }
        let building = Self {
            breaks,
            oldest_pending_wait_limit: configuration.oldest_pending_wait_limit,
            now: inputs.now,
            streams: inputs.streams,
            idle_cards,
            queues,
            stream_order,
            residents_by_card,
            resident_anywhere: inputs
                .residents
                .iter()
                .map(|resident| resident.stream)
                .collect(),
        };
        if is_input_order_followed {
            return building;
        }
        let mut sorted = building;
        let order = sorted.rank_of_every_stream();
        for residents in sorted.residents_by_card.values_mut() {
            residents.sort_by_key(|resident| {
                (
                    order.get(&resident.stream).copied().unwrap_or(usize::MAX),
                    resident.stream,
                )
            });
        }
        sorted
    }

    fn rank_of_every_stream(&self) -> BTreeMap<StreamKey, usize> {
        self.stream_order
            .iter()
            .enumerate()
            .map(|(rank, stream)| (*stream, rank))
            .collect()
    }

    fn facts(&self, stream: StreamKey) -> StreamFacts {
        *self
            .streams
            .get(&stream)
            .unwrap_or_else(|| panic!("{stream:?} 不在调度的流表里"))
    }

    fn states_left(&self, stream: StreamKey) -> u64 {
        self.queues.get(&stream).map_or(0, StreamQueue::states_left)
    }

    fn residents_on(&self, card: CardIndex) -> Vec<ResidentJudge> {
        self.residents_by_card
            .get(&card)
            .cloned()
            .unwrap_or_default()
    }

    /// 几条流能不能一起装进 `free_mebibytes`：表都放得下，剩下的均分之后每条至少装得下一个态。
    fn fit_together(&self, streams: &[StreamKey], free_mebibytes: u64) -> bool {
        if streams.is_empty() {
            return true;
        }
        let tables: u64 = streams
            .iter()
            .map(|stream| self.facts(*stream).table_mebibytes)
            .sum();
        let Some(scratch) = free_mebibytes.checked_sub(tables) else {
            return false;
        };
        let count = u64::try_from(streams.len()).expect("装得进 u64");
        scratch / count >= mebibytes_of_one_state_rounded_up()
    }

    fn mebibytes_of_the_resident(&self, resident: &ResidentJudge) -> u64 {
        mebibytes_of_a_judge(&self.facts(resident.stream), resident.states_per_dispatch)
    }

    /// 从这条流池里那一截前面切一批（至多 `states_per_dispatch` 个态）；池里没有它的态交 `None`。
    fn carve_a_batch(&mut self, stream: StreamKey, states_per_dispatch: u64) -> Option<Batch> {
        let is_consumed = !self.breaks.is_on(BreakSwitch::CarvedStatesNotConsumed);
        let queue = self.queues.get_mut(&stream)?;
        let mut wanted = states_per_dispatch;
        let mut pieces = Vec::new();
        let mut index = 0;
        while wanted > 0 && index < queue.pieces.len() {
            let front = &mut queue.pieces[index];
            let taken = wanted.min(front.ordinals.end - front.ordinals.start);
            pieces.push(BatchPiece {
                block: front.block,
                ordinals: front.ordinals.start..front.ordinals.start + taken,
            });
            wanted -= taken;
            if is_consumed {
                front.ordinals.start += taken;
                if front.ordinals.is_empty() {
                    queue.pieces.pop_front();
                }
            } else {
                index += 1;
            }
        }
        (!pieces.is_empty()).then_some(Batch {
            stream,
            states_per_dispatch,
            pieces,
        })
    }

    /// 给一条流在一张卡上切这一波的批：平常一批；`BatchesOfAStreamBackToBack` 打开时把池里它的态连着切完。
    fn carve_the_batches_of_the_wave(
        &mut self,
        stream: StreamKey,
        states_per_dispatch: u64,
        draft: &mut CardWaveDraft,
    ) {
        while let Some(batch) = self.carve_a_batch(stream, states_per_dispatch) {
            draft.batches.push(batch);
            if !self.breaks.is_on(BreakSwitch::BatchesOfAStreamBackToBack) {
                break;
            }
        }
    }

    /// 均分草稿区：按（这条流的态数）要，一条要的少于均分的就给它要的，多出来的在别的流之间再均分；交回各流一次派活的态数（与 `streams` 同序）。
    fn split_the_scratch(&self, scratch_mebibytes: u64, streams: &[StreamKey]) -> Vec<u64> {
        let demands: Vec<u64> = streams
            .iter()
            .map(|stream| {
                if self
                    .breaks
                    .is_on(BreakSwitch::BatchNotCappedByTheStateCount)
                {
                    u64::MAX
                } else {
                    self.facts(*stream).state_count
                }
            })
            .collect();
        let mebibytes_of = |states: u64| {
            states
                .checked_mul(bytes_of_one_state_on_the_card())
                .map(|bytes| bytes.div_ceil(BYTES_PER_MEBIBYTE))
        };
        let mut granted = vec![0u64; streams.len()];
        if self.breaks.is_on(BreakSwitch::ScratchSplitByEntryOrder) {
            let mut remaining = scratch_mebibytes;
            for (index, demand) in demands.iter().enumerate() {
                let states = (*demand).min(states_fitting_in(remaining));
                granted[index] = states;
                remaining -= mebibytes_of(states).expect("装得下的态的显存装得进 u64");
            }
            return granted;
        }
        let mut active: Vec<usize> = (0..streams.len()).collect();
        let mut remaining = scratch_mebibytes;
        while !active.is_empty() {
            let share = remaining / u64::try_from(active.len()).expect("装得进 u64");
            let (satisfied, short): (Vec<usize>, Vec<usize>) = active
                .iter()
                .copied()
                .partition(|index| mebibytes_of(demands[*index]).is_some_and(|need| need <= share));
            if satisfied.is_empty() {
                for index in short {
                    granted[index] = demands[index].min(states_fitting_in(share));
                }
                break;
            }
            for index in &satisfied {
                granted[*index] = demands[*index];
                remaining -= mebibytes_of(demands[*index]).expect("上一行判过装得下");
            }
            active = short;
        }
        granted
    }

    /// 不饿死：等过限时、不在任何卡上驻留、哪张空着的卡留住有待判态的判器之后都装不下的流，挑头一张腾得出的空卡整卡重装。
    fn rescue_the_starving_streams(&mut self, drafts: &mut BTreeMap<CardIndex, CardWaveDraft>) {
        if self.breaks.is_on(BreakSwitch::StarvingStreamsNotRescued) {
            return;
        }
        let order = self.stream_order.clone();
        for stream in order {
            let Some(queue) = self.queues.get(&stream) else {
                continue;
            };
            let has_waited_the_limit =
                self.now.saturating_sub(queue.oldest_entered_at) >= self.oldest_pending_wait_limit;
            if !has_waited_the_limit || self.resident_anywhere.contains(&stream) {
                continue;
            }
            let free_keeping_the_pending_judges = |building: &Self, card: &CardOfTheService| {
                let kept: u64 = building
                    .residents_on(card.card)
                    .iter()
                    .filter(|resident| building.states_left(resident.stream) > 0)
                    .map(|resident| building.mebibytes_of_the_resident(resident))
                    .sum();
                card.mebibytes_for_the_judges().saturating_sub(kept)
            };
            let fits_somewhere = self.idle_cards.iter().any(|card| {
                !drafts[&card.card].is_rebuilt_for_a_starving_stream
                    && self.fit_together(&[stream], free_keeping_the_pending_judges(self, card))
            });
            if fits_somewhere {
                continue;
            }
            let cards = self.idle_cards.clone();
            for card in cards {
                if drafts[&card.card].is_rebuilt_for_a_starving_stream
                    || !self.fit_together(&[stream], card.mebibytes_for_the_judges())
                {
                    continue;
                }
                let residents = self.residents_on(card.card);
                let mut reloaded = vec![stream];
                for resident in &residents {
                    if self.states_left(resident.stream) == 0 {
                        continue;
                    }
                    let mut widened = reloaded.clone();
                    widened.push(resident.stream);
                    if self.fit_together(&widened, card.mebibytes_for_the_judges()) {
                        reloaded = widened;
                    }
                }
                let rank = self.rank_of_every_stream();
                reloaded.sort_by_key(|reloaded_stream| (rank[reloaded_stream], *reloaded_stream));
                let tables: u64 = reloaded
                    .iter()
                    .map(|reloaded_stream| self.facts(*reloaded_stream).table_mebibytes)
                    .sum();
                let sizes =
                    self.split_the_scratch(card.mebibytes_for_the_judges() - tables, &reloaded);
                let draft = drafts.get_mut(&card.card).expect("每张空卡都有一份");
                draft.is_rebuilt_for_a_starving_stream = true;
                draft.evictions = residents.iter().map(|resident| resident.stream).collect();
                draft.loads = reloaded
                    .iter()
                    .zip(&sizes)
                    .filter(|(_, size)| **size > 0)
                    .map(|(reloaded_stream, size)| JudgeToLoad {
                        stream: *reloaded_stream,
                        states_per_dispatch: *size,
                    })
                    .collect();
                self.resident_anywhere.insert(stream);
                break;
            }
        }
    }

    /// 排这一波；交回计划与「每张空着的卡都有批、每一批都装满了一次派活」。
    fn plan_every_idle_card(mut self) -> (WavePlan, bool) {
        let mut drafts: BTreeMap<CardIndex, CardWaveDraft> = self
            .idle_cards
            .iter()
            .map(|card| (card.card, CardWaveDraft::default()))
            .collect();
        self.rescue_the_starving_streams(&mut drafts);
        let cards = self.idle_cards.clone();
        let is_residence_ignored = self.breaks.is_on(BreakSwitch::ResidenceIgnored);
        // 驻留那一遍：先派回驻留的卡，再轮到新装
        for card in &cards {
            let mut draft = std::mem::take(drafts.get_mut(&card.card).expect("每张空卡都有一份"));
            if draft.is_rebuilt_for_a_starving_stream {
                for load in draft.loads.clone() {
                    self.carve_the_batches_of_the_wave(
                        load.stream,
                        load.states_per_dispatch,
                        &mut draft,
                    );
                }
            } else if !is_residence_ignored {
                for resident in self.residents_on(card.card) {
                    self.carve_the_batches_of_the_wave(
                        resident.stream,
                        resident.states_per_dispatch,
                        &mut draft,
                    );
                }
            }
            drafts.insert(card.card, draft);
        }
        // 新装那一遍
        for card in &cards {
            let mut draft = std::mem::take(drafts.get_mut(&card.card).expect("每张空卡都有一份"));
            if !draft.is_rebuilt_for_a_starving_stream {
                self.load_new_streams_onto_the_card(card, &mut draft, is_residence_ignored);
            }
            drafts.insert(card.card, draft);
        }
        let rank = self.rank_of_every_stream();
        let mut is_every_card_filled = true;
        let mut plans = Vec::new();
        for card in &cards {
            let mut draft = drafts.remove(&card.card).expect("每张空卡都有一份");
            if !self.breaks.is_on(BreakSwitch::InputOrderFollowed) {
                draft.batches.sort_by_key(|batch| {
                    (
                        rank.get(&batch.stream).copied().unwrap_or(usize::MAX),
                        batch.stream,
                    )
                });
            }
            is_every_card_filled &= !draft.batches.is_empty()
                && draft
                    .batches
                    .iter()
                    .all(|batch| batch.state_count() == batch.states_per_dispatch);
            plans.push(self.card_plan_through_the_ledger(card, draft));
        }
        (WavePlan { cards: plans }, is_every_card_filled)
    }

    /// 新装那一遍：挑池里还剩态、不在这张卡上的流，按入池先后挑到装不下；要腾显存时只换出这一波在这张卡上没有批的判器。
    fn load_new_streams_onto_the_card(
        &mut self,
        card: &CardOfTheService,
        draft: &mut CardWaveDraft,
        is_residence_ignored: bool,
    ) {
        let residents = self.residents_on(card.card);
        let served: BTreeSet<StreamKey> = draft.batches.iter().map(|batch| batch.stream).collect();
        let (kept, idle): (Vec<ResidentJudge>, Vec<ResidentJudge>) = if is_residence_ignored {
            (Vec::new(), residents.clone())
        } else {
            residents
                .iter()
                .copied()
                .partition(|resident| served.contains(&resident.stream))
        };
        let kept_mebibytes: u64 = if self
            .breaks
            .is_on(BreakSwitch::ScratchSplitIgnoresKeptJudges)
        {
            0
        } else {
            kept.iter()
                .map(|resident| self.mebibytes_of_the_resident(resident))
                .sum()
        };
        let free_evicting_every_idle_judge = card
            .mebibytes_for_the_judges()
            .saturating_sub(kept_mebibytes);
        let on_this_card: BTreeSet<StreamKey> = if is_residence_ignored {
            BTreeSet::new()
        } else {
            residents.iter().map(|resident| resident.stream).collect()
        };
        let mut newcomers: Vec<StreamKey> = Vec::new();
        for stream in self.stream_order.clone() {
            if self.states_left(stream) == 0 || on_this_card.contains(&stream) {
                continue;
            }
            let mut widened = newcomers.clone();
            widened.push(stream);
            if self.fit_together(&widened, free_evicting_every_idle_judge) {
                newcomers = widened;
            }
        }
        if is_residence_ignored {
            draft.evictions = residents.iter().map(|resident| resident.stream).collect();
        }
        if newcomers.is_empty() {
            return;
        }
        let mut free = free_evicting_every_idle_judge;
        if !is_residence_ignored {
            let idle_mebibytes: u64 = idle
                .iter()
                .map(|resident| self.mebibytes_of_the_resident(resident))
                .sum();
            free = free.saturating_sub(idle_mebibytes);
            let mut evictable = idle.clone();
            evictable.sort_by_key(|resident| {
                (
                    std::cmp::Reverse(self.mebibytes_of_the_resident(resident)),
                    resident.stream,
                )
            });
            let mut eviction_order = evictable.into_iter();
            while !self.fit_together(&newcomers, free) {
                if self.breaks.is_on(BreakSwitch::EveryResidentJudgeEvicted) {
                    draft.evictions = residents.iter().map(|resident| resident.stream).collect();
                    free = card.mebibytes_for_the_judges();
                    break;
                }
                let evicted = eviction_order
                    .next()
                    .expect("挑新流时按换出全部空闲判器算过装得下");
                free += self.mebibytes_of_the_resident(&evicted);
                draft.evictions.push(evicted.stream);
            }
        }
        let tables: u64 = newcomers
            .iter()
            .map(|stream| self.facts(*stream).table_mebibytes)
            .sum();
        let sizes = self.split_the_scratch(free - tables, &newcomers);
        for (stream, size) in newcomers.iter().zip(sizes) {
            if size == 0 {
                continue;
            }
            draft.loads.push(JudgeToLoad {
                stream: *stream,
                states_per_dispatch: size,
            });
            self.carve_the_batches_of_the_wave(*stream, size, draft);
        }
    }

    /// 把一张卡的草稿经显存账收成计划：这一波之后驻留的 = 原来驻留的去掉换出的，加上装上的。
    fn card_plan_through_the_ledger(
        &self,
        card: &CardOfTheService,
        draft: CardWaveDraft,
    ) -> CardPlanOfTheWave {
        let evicted: BTreeSet<StreamKey> = draft.evictions.iter().copied().collect();
        let mut ledger = CardMemoryLedger::empty(card, self.breaks);
        let staying = self
            .residents_on(card.card)
            .into_iter()
            .filter(|resident| !evicted.contains(&resident.stream))
            .map(|resident| (resident.stream, resident.states_per_dispatch));
        let loaded = draft
            .loads
            .iter()
            .map(|load| (load.stream, load.states_per_dispatch));
        for (stream, states_per_dispatch) in staying.chain(loaded) {
            let facts = self.facts(stream);
            ledger
                .admit(stream, &facts, states_per_dispatch)
                .unwrap_or_else(|refusal| {
                    panic!("批次调度算出的计划越过了卡的显存额度：{refusal:?}")
                });
        }
        CardPlanOfTheWave {
            card: card.card,
            evictions: draft.evictions,
            loads: draft.loads,
            batches: draft.batches,
            resident_after_the_wave: ledger.into_resident_judges(),
        }
    }
}
