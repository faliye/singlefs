//! 待判池：跨片、跨流的待判块放在同一个池里，批次调度从这里连续取。
//!
//! 一块入池时拿一个按先后递增的号（[`PendingBlockIdentifier`]），只从前往后派：没派出去的总是连着的一截，
//! 派出去一截就从「没派」挪到「在判」，结论回来逐态记上，整块的态都有了结论才把这一块交出去、从池里删掉。
//! 接口（[`PendingPool`]）的实现是内存里的 [`InMemoryPendingPool`]：服务主循环与批次调度只看它。判定库里的待判行由
//! `crate::library_pending_pool` 取出、认领之后入这一份，判完由它写回判定库。

use std::collections::BTreeMap;
use std::ops::Range;
use std::time::Duration;

use singlefs_verification::crash_judge_gpu::VERDICT_WORDS;

use crate::batch_scheduling::{BatchPiece, PendingBlock, PendingBlockIdentifier};
use crate::break_switches::{BreakSwitch, BreakSwitches};
use crate::card_memory_ledger::CardIndex;
use crate::stream_identity::StreamKey;

/// 一个态的结论（GPU 判器交回的那几个字）。
pub type StateVerdict = [u32; VERDICT_WORDS];

/// 一截在卡上判的账（工作线程量的，`crate::card_worker` 文件头「判的账」）：判它的那几次派活有几次（一次派活带着几块的截，每块各记一次），
/// 与这一截分到的卡上用时（判它那一段的挂钟按态数摊到各截）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CardWorkOnAPiece {
    pub dispatches: u64,
    pub card_nanoseconds: u64,
}

/// 一块在一张卡上判的账：这张卡判了它几个态、判它的派活有几次、卡上用时之和。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CardWorkOnABlock {
    pub states: u64,
    pub dispatches: u64,
    pub card_nanoseconds: u64,
}

/// 整块判完：哪一块、哪条流、哪几个态、按序号次序的结论、判了它各截的每张卡上的账（不空，按卡号排）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JudgedBlock {
    pub block: PendingBlockIdentifier,
    pub stream: StreamKey,
    pub ordinals: Range<u64>,
    pub verdicts: Vec<StateVerdict>,
    pub work_on_each_card: BTreeMap<CardIndex, CardWorkOnABlock>,
}

/// 池拒收的。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PoolRefusal {
    /// 入池的一块是空的。
    EmptyBlock { stream: StreamKey },
    /// 池里没有这一块（判完已经交出去了，或从来没有）。
    UnknownBlock { block: PendingBlockIdentifier },
    /// 要派的一截不是这一块没派出去的那一截的开头。
    PieceNotAtTheFrontOfTheUndispatchedStates {
        block: PendingBlockIdentifier,
        piece: Range<u64>,
        undispatched: Range<u64>,
    },
    /// 记结论的一截里有没派出去的态。
    VerdictsForStatesNotDispatched {
        block: PendingBlockIdentifier,
        piece: Range<u64>,
    },
    /// 一截的结论条数与态数对不上。
    VerdictCountDiffersFromThePiece {
        block: PendingBlockIdentifier,
        piece: Range<u64>,
        verdicts: usize,
    },
    /// 一截里有态已经记过结论。
    VerdictsRecordedTwice {
        block: PendingBlockIdentifier,
        ordinal: u64,
    },
}

/// 待判池的接口。
pub trait PendingPool {
    /// 一块入池，交回它的号。
    ///
    /// # Errors
    /// 空块（[`PoolRefusal::EmptyBlock`]）。
    fn add_block(
        &mut self,
        stream: StreamKey,
        ordinals: Range<u64>,
        entered_at: Duration,
    ) -> Result<PendingBlockIdentifier, PoolRefusal>;

    /// 此刻还有没派出去的态的块，按号排。
    fn undispatched_blocks(&self) -> Vec<PendingBlock>;

    /// 一截派出去了：从这一块没派的那一截开头扣掉。
    ///
    /// # Errors
    /// 没有这一块；这一截不是没派那一截的开头或越过了它。
    fn mark_dispatched(&mut self, piece: &BatchPiece) -> Result<(), PoolRefusal>;

    /// 记一截在卡 `card` 上判出的结论与判它的账；整块都有了结论就交回整块、从池里删掉。
    ///
    /// # Errors
    /// 没有这一块；这一截里有没派出去的态、条数对不上、有态记过结论。
    fn record_verdicts(
        &mut self,
        piece: &BatchPiece,
        card: CardIndex,
        verdicts: Vec<StateVerdict>,
        work: CardWorkOnAPiece,
    ) -> Result<Option<JudgedBlock>, PoolRefusal>;

    /// 把一块整个撤掉（判它的卡坏了，向客户端报拒）：交回它的流；没有这一块交 `None`。
    fn withdraw_the_block(&mut self, block: PendingBlockIdentifier) -> Option<StreamKey>;
}

struct BlockInThePool {
    stream: StreamKey,
    ordinals: Range<u64>,
    entered_at: Duration,
    /// 没派的那一截从这里起到 `ordinals.end`。
    undispatched_from: u64,
    verdicts: Vec<Option<StateVerdict>>,
    states_with_a_verdict: u64,
    work_on_each_card: BTreeMap<CardIndex, CardWorkOnABlock>,
}

/// 内存里的待判池。
pub struct InMemoryPendingPool {
    next_identifier: u64,
    blocks: BTreeMap<PendingBlockIdentifier, BlockInThePool>,
    break_switches: BreakSwitches,
}

impl InMemoryPendingPool {
    #[must_use]
    pub fn empty(break_switches: BreakSwitches) -> Self {
        Self {
            next_identifier: 0,
            blocks: BTreeMap::new(),
            break_switches,
        }
    }
}

impl PendingPool for InMemoryPendingPool {
    fn add_block(
        &mut self,
        stream: StreamKey,
        ordinals: Range<u64>,
        entered_at: Duration,
    ) -> Result<PendingBlockIdentifier, PoolRefusal> {
        if ordinals.is_empty() {
            return Err(PoolRefusal::EmptyBlock { stream });
        }
        let block = PendingBlockIdentifier(self.next_identifier);
        self.next_identifier += 1;
        let state_count = usize::try_from(ordinals.end - ordinals.start).expect("装得进 usize");
        self.blocks.insert(
            block,
            BlockInThePool {
                stream,
                undispatched_from: ordinals.start,
                ordinals,
                entered_at,
                verdicts: vec![None; state_count],
                states_with_a_verdict: 0,
                work_on_each_card: BTreeMap::new(),
            },
        );
        Ok(block)
    }

    fn undispatched_blocks(&self) -> Vec<PendingBlock> {
        self.blocks
            .iter()
            .filter(|(_, held)| held.undispatched_from < held.ordinals.end)
            .map(|(block, held)| PendingBlock {
                block: *block,
                stream: held.stream,
                entered_at: held.entered_at,
                undispatched: held.undispatched_from..held.ordinals.end,
            })
            .collect()
    }

    fn mark_dispatched(&mut self, piece: &BatchPiece) -> Result<(), PoolRefusal> {
        let held = self
            .blocks
            .get_mut(&piece.block)
            .ok_or(PoolRefusal::UnknownBlock { block: piece.block })?;
        let undispatched = held.undispatched_from..held.ordinals.end;
        if piece.ordinals.is_empty()
            || piece.ordinals.start != undispatched.start
            || piece.ordinals.end > undispatched.end
        {
            return Err(PoolRefusal::PieceNotAtTheFrontOfTheUndispatchedStates {
                block: piece.block,
                piece: piece.ordinals.clone(),
                undispatched,
            });
        }
        held.undispatched_from = if self
            .break_switches
            .is_on(BreakSwitch::PoolDropsTheRestOfAPartlyDispatchedBlock)
        {
            held.ordinals.end
        } else {
            piece.ordinals.end
        };
        Ok(())
    }

    fn record_verdicts(
        &mut self,
        piece: &BatchPiece,
        card: CardIndex,
        verdicts: Vec<StateVerdict>,
        work: CardWorkOnAPiece,
    ) -> Result<Option<JudgedBlock>, PoolRefusal> {
        let held = self
            .blocks
            .get_mut(&piece.block)
            .ok_or(PoolRefusal::UnknownBlock { block: piece.block })?;
        if piece.ordinals.start < held.ordinals.start || piece.ordinals.end > held.undispatched_from
        {
            return Err(PoolRefusal::VerdictsForStatesNotDispatched {
                block: piece.block,
                piece: piece.ordinals.clone(),
            });
        }
        let expected =
            usize::try_from(piece.ordinals.end - piece.ordinals.start).expect("装得进 usize");
        if verdicts.len() != expected {
            return Err(PoolRefusal::VerdictCountDiffersFromThePiece {
                block: piece.block,
                piece: piece.ordinals.clone(),
                verdicts: verdicts.len(),
            });
        }
        let offset =
            usize::try_from(piece.ordinals.start - held.ordinals.start).expect("装得进 usize");
        if let Some(position) = held.verdicts[offset..offset + expected]
            .iter()
            .position(Option::is_some)
        {
            return Err(PoolRefusal::VerdictsRecordedTwice {
                block: piece.block,
                ordinal: piece.ordinals.start + u64::try_from(position).expect("装得进 u64"),
            });
        }
        for (slot, verdict) in held.verdicts[offset..offset + expected]
            .iter_mut()
            .zip(verdicts)
        {
            *slot = Some(verdict);
        }
        held.states_with_a_verdict += piece.ordinals.end - piece.ordinals.start;
        let on_this_card = held
            .work_on_each_card
            .entry(card)
            .or_insert(CardWorkOnABlock {
                states: 0,
                dispatches: 0,
                card_nanoseconds: 0,
            });
        on_this_card.states += piece.ordinals.end - piece.ordinals.start;
        on_this_card.dispatches += work.dispatches;
        on_this_card.card_nanoseconds += work.card_nanoseconds;
        if held.states_with_a_verdict < held.ordinals.end - held.ordinals.start {
            return Ok(None);
        }
        let judged = self.blocks.remove(&piece.block).expect("上面刚取到");
        Ok(Some(JudgedBlock {
            block: piece.block,
            stream: judged.stream,
            ordinals: judged.ordinals,
            verdicts: judged
                .verdicts
                .into_iter()
                .map(|verdict| verdict.expect("每个态都记过结论"))
                .collect(),
            work_on_each_card: judged.work_on_each_card,
        }))
    }

    fn withdraw_the_block(&mut self, block: PendingBlockIdentifier) -> Option<StreamKey> {
        self.blocks.remove(&block).map(|held| held.stream)
    }
}
