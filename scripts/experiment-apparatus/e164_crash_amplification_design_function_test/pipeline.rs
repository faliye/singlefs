//! E164 F10、F11 的流水线与领块模型（跑前登记 5.6）：录入与核对两个进程经库交接，穷举交错、被杀、换判法、卡掉线、租期到期。
//! 显式状态、深度优先、按状态去重；终局 = 没有后继的可达状态；门禁在终局读一次。

use std::collections::{BTreeSet, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DataState {
    Absent,
    Half,
    Full,
}

/// 「已核对」标记：没有、布尔、或记下核它的判法摘要（0 = D1，1 = D2）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CheckedFlag {
    Unset,
    Boolean,
    Digest(u8),
}

/// 领块记录（只在落盘的两形里进库）：哪张卡、租期还有效没有。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PersistedClaim {
    Unclaimed,
    Held { card: u8, lease_valid: bool },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StoreBlock {
    pub data: DataState,
    pub recorded: bool,
    pub checked: CheckedFlag,
    /// 结果：算它时读到的数据、算它用的判法。
    pub result: Option<(DataState, u8)>,
    pub tally: u8,
    pub claim: PersistedClaim,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RecordOrder {
    /// 数据与「已录入」标记同一批原子写。
    Atomic,
    /// 标记先于数据写（record_flag_before_data）：标记、半块、整块三次写。
    FlagBeforeData,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CheckOrder {
    /// 一批的结果与标记同一次原子写，结果按块键覆盖。
    AtomicBatchKeyed,
    /// 逐块先写「已核对」标记、再写结果（checked_flag_before_result）。
    FlagBeforeResult,
    /// 逐块先写结果（按追加计）、再写标记（result_then_flag_with_appended_tally）。
    ResultThenFlagAppended,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ClaimForm {
    /// 领块只在内存：库里不记。
    InMemory,
    /// 领块落盘、不带租期。
    PersistedWithoutLease,
    /// 领块落盘、带租期：租期可在任何时刻到期，到期后别的卡可以接。
    PersistedWithLease,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModelForm {
    pub record_order: RecordOrder,
    pub check_order: CheckOrder,
    pub flag_records_digest: bool,
    pub flush_when_blocked: bool,
    pub claim_form: ClaimForm,
    pub cards_can_drop: bool,
    pub allow_kills: bool,
    pub allow_judging_change: bool,
    /// 带租期那一形里，结果写回时领块记录已不是自己的（租期到了被接走）：按追加计结果。只给变异 M17 翻。
    pub late_writer_appends: bool,
    /// 租期会不会到期。只给变异 M18 翻。
    pub lease_can_expire: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Geometry {
    pub blocks: usize,
    pub cards: usize,
    pub backlog_cap: usize,
    pub batch: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CardState {
    Dropped,
    Alive {
        held: Vec<usize>,
        computed: Vec<(DataState, u8)>,
        writes_done: usize,
        killed_once: bool,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ModelState {
    pub store: Vec<StoreBlock>,
    pub recorder_next_block: usize,
    pub recorder_step: usize,
    pub recorder_killed_once: bool,
    pub cards: Vec<CardState>,
    pub judging_phase: u8,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ModelReport {
    pub reachable_states: u64,
    pub terminal_states: u64,
    pub missing_result: u64,
    pub wrong_input: u64,
    pub double_tally: u64,
    pub stale_after_change: u64,
    pub stuck: u64,
    pub dead_block_terminals: u64,
    pub kill_transitions: u64,
    pub judging_change_transitions: u64,
    pub card_drop_transitions: u64,
    pub takeover_after_lease_transitions: u64,
    pub backlog_peak: usize,
    pub backlog_positive_states: u64,
    pub terminal_backlog_values: BTreeSet<usize>,
    pub held_peak: usize,
    pub held_positive_states: u64,
    pub terminal_held_values: BTreeSet<usize>,
    pub stopped_over_limit: bool,
}

pub const REACHABLE_STATE_LIMIT: u64 = 10_000_000;

fn checked_now(block: &StoreBlock, form: &ModelForm, phase: u8) -> bool {
    match block.checked {
        CheckedFlag::Unset => false,
        CheckedFlag::Boolean => true,
        CheckedFlag::Digest(digest) => form.flag_records_digest && digest == phase,
    }
}

fn pending_blocks(state: &ModelState, form: &ModelForm) -> Vec<usize> {
    (0..state.store.len())
        .filter(|index| state.store[*index].recorded && !checked_now(&state.store[*index], form, state.judging_phase))
        .collect()
}

fn held_block_count(state: &ModelState) -> usize {
    state
        .store
        .iter()
        .filter(|block| matches!(block.claim, PersistedClaim::Held { lease_valid: true, .. }))
        .count()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TransitionKind {
    Ordinary,
    Kill,
    JudgingChange,
    CardDrop,
    TakeoverAfterLease,
}

fn initial_state(geometry: &Geometry) -> ModelState {
    ModelState {
        store: vec![
            StoreBlock {
                data: DataState::Absent,
                recorded: false,
                checked: CheckedFlag::Unset,
                result: None,
                tally: 0,
                claim: PersistedClaim::Unclaimed,
            };
            geometry.blocks
        ],
        recorder_next_block: 0,
        recorder_step: 0,
        recorder_killed_once: false,
        cards: vec![
            CardState::Alive { held: Vec::new(), computed: Vec::new(), writes_done: 0, killed_once: false };
            geometry.cards
        ],
        judging_phase: 0,
    }
}

fn drop_already_used(state: &ModelState) -> bool {
    state.cards.iter().any(|card| matches!(card, CardState::Dropped))
}

fn recorder_successors(state: &ModelState, form: &ModelForm, geometry: &Geometry, output: &mut Vec<(ModelState, TransitionKind)>) {
    let block_index = state.recorder_next_block;
    if block_index >= geometry.blocks {
        return;
    }
    let backlog = pending_blocks(state, form).len();
    if state.recorder_step == 0 && state.store[block_index].recorded {
        let mut next = state.clone();
        next.recorder_next_block += 1;
        output.push((next, TransitionKind::Ordinary));
    } else if state.recorder_step > 0 || backlog < geometry.backlog_cap {
        let mut next = state.clone();
        let block = &mut next.store[block_index];
        let writes_in_this_order = match form.record_order {
            RecordOrder::Atomic => {
                block.data = DataState::Full;
                block.recorded = true;
                1
            }
            RecordOrder::FlagBeforeData => {
                match state.recorder_step {
                    0 => block.recorded = true,
                    1 => block.data = DataState::Half,
                    _ => block.data = DataState::Full,
                }
                3
            }
        };
        if state.recorder_step + 1 == writes_in_this_order {
            next.recorder_next_block += 1;
            next.recorder_step = 0;
        } else {
            next.recorder_step += 1;
        }
        output.push((next, TransitionKind::Ordinary));
    }
    if form.allow_kills && !state.recorder_killed_once {
        let mut next = state.clone();
        next.recorder_next_block = 0;
        next.recorder_step = 0;
        next.recorder_killed_once = true;
        output.push((next, TransitionKind::Kill));
    }
}

fn claim_allows(claim: PersistedClaim, form: &ModelForm, card: u8) -> bool {
    match (form.claim_form, claim) {
        (ClaimForm::InMemory, _) | (_, PersistedClaim::Unclaimed) => true,
        (ClaimForm::PersistedWithoutLease | ClaimForm::PersistedWithLease, PersistedClaim::Held { card: holder, lease_valid }) => {
            holder == card || !lease_valid
        }
    }
}

fn flag_for(form: &ModelForm, digest: u8) -> CheckedFlag {
    if form.flag_records_digest {
        CheckedFlag::Digest(digest)
    } else {
        CheckedFlag::Boolean
    }
}

/// 一次结果写回：自己的领块记录还在（或不落盘）时按块键覆盖；带租期、被接走之后迟到的写回按 `late_writer_appends` 办。
fn write_result(block: &mut StoreBlock, computed: (DataState, u8), form: &ModelForm, card: u8, appended: bool) {
    let own_claim = matches!(block.claim, PersistedClaim::Held { card: holder, lease_valid: true } if holder == card);
    let late = form.claim_form == ClaimForm::PersistedWithLease && !own_claim;
    block.result = Some(computed);
    if appended || (late && form.late_writer_appends) {
        block.tally = block.tally.saturating_add(1);
    } else {
        block.tally = 1;
    }
}

fn release_own_claim(block: &mut StoreBlock, card: u8) {
    if matches!(block.claim, PersistedClaim::Held { card: holder, .. } if holder == card) {
        block.claim = PersistedClaim::Unclaimed;
    }
}

fn card_successors(state: &ModelState, form: &ModelForm, geometry: &Geometry, output: &mut Vec<(ModelState, TransitionKind)>) {
    for card_index in 0..state.cards.len() {
        let card = u8::try_from(card_index).expect("卡数不超过 255");
        let CardState::Alive { held, computed, writes_done, killed_once } = &state.cards[card_index] else {
            continue;
        };
        if held.is_empty() {
            let pending = pending_blocks(state, form);
            let free: Vec<usize> = pending.iter().copied().filter(|index| claim_allows(state.store[*index].claim, form, card)).collect();
            let recorder_done = state.recorder_next_block >= geometry.blocks;
            let may_take = free.len() >= geometry.batch
                || (!free.is_empty() && (recorder_done || (form.flush_when_blocked && pending.len() >= geometry.backlog_cap)));
            if may_take {
                let chosen: Vec<usize> = free.iter().copied().take(geometry.batch).collect();
                let mut next = state.clone();
                let mut kind = TransitionKind::Ordinary;
                if form.claim_form != ClaimForm::InMemory {
                    for block_index in &chosen {
                        if matches!(next.store[*block_index].claim, PersistedClaim::Held { card: holder, lease_valid: false } if holder != card) {
                            kind = TransitionKind::TakeoverAfterLease;
                        }
                        next.store[*block_index].claim = PersistedClaim::Held { card, lease_valid: true };
                    }
                }
                next.cards[card_index] = CardState::Alive { held: chosen, computed: Vec::new(), writes_done: 0, killed_once: *killed_once };
                output.push((next, kind));
            }
        } else if computed.len() < held.len() {
            let mut next = state.clone();
            let read = (state.store[held[computed.len()]].data, state.judging_phase);
            if let CardState::Alive { computed: next_computed, .. } = &mut next.cards[card_index] {
                next_computed.push(read);
            }
            output.push((next, TransitionKind::Ordinary));
        } else {
            let mut next = state.clone();
            let total_writes = match form.check_order {
                CheckOrder::AtomicBatchKeyed => {
                    for (block_index, result) in held.iter().zip(computed) {
                        let block = &mut next.store[*block_index];
                        write_result(block, *result, form, card, false);
                        block.checked = flag_for(form, result.1);
                        release_own_claim(block, card);
                    }
                    1
                }
                CheckOrder::FlagBeforeResult | CheckOrder::ResultThenFlagAppended => {
                    let position = writes_done / 2;
                    let block = &mut next.store[held[position]];
                    let result = computed[position];
                    let flag_first = form.check_order == CheckOrder::FlagBeforeResult;
                    let writing_flag = (writes_done % 2 == 0) == flag_first;
                    if writing_flag {
                        block.checked = flag_for(form, result.1);
                    } else {
                        write_result(block, result, form, card, !flag_first);
                    }
                    if writes_done % 2 == 1 {
                        release_own_claim(block, card);
                    }
                    held.len() * 2
                }
            };
            next.cards[card_index] = if writes_done + 1 == total_writes {
                CardState::Alive { held: Vec::new(), computed: Vec::new(), writes_done: 0, killed_once: *killed_once }
            } else {
                CardState::Alive { held: held.clone(), computed: computed.clone(), writes_done: writes_done + 1, killed_once: *killed_once }
            };
            output.push((next, TransitionKind::Ordinary));
        }
        if form.allow_kills && !killed_once {
            let mut next = state.clone();
            next.cards[card_index] = CardState::Alive { held: Vec::new(), computed: Vec::new(), writes_done: 0, killed_once: true };
            output.push((next, TransitionKind::Kill));
        }
        if form.cards_can_drop && !drop_already_used(state) {
            let mut next = state.clone();
            next.cards[card_index] = CardState::Dropped;
            output.push((next, TransitionKind::CardDrop));
        }
    }
}

fn environment_successors(state: &ModelState, form: &ModelForm, output: &mut Vec<(ModelState, TransitionKind)>) {
    if form.allow_judging_change && state.judging_phase == 0 {
        let mut next = state.clone();
        next.judging_phase = 1;
        output.push((next, TransitionKind::JudgingChange));
    }
    if form.claim_form == ClaimForm::PersistedWithLease && form.lease_can_expire {
        for block_index in 0..state.store.len() {
            if let PersistedClaim::Held { card, lease_valid: true } = state.store[block_index].claim {
                let mut next = state.clone();
                next.store[block_index].claim = PersistedClaim::Held { card, lease_valid: false };
                output.push((next, TransitionKind::Ordinary));
            }
        }
    }
}

fn tally_terminal(state: &ModelState, form: &ModelForm, report: &mut ModelReport) {
    report.terminal_states += 1;
    let phase = state.judging_phase;
    let mut missing_result = false;
    let mut wrong_input = false;
    let mut double_tally = false;
    let mut stale_after_change = false;
    let mut stuck = false;
    let mut dead_block = false;
    for block in &state.store {
        let checked = checked_now(block, form, phase);
        if checked && block.result.is_none() {
            missing_result = true;
        }
        if block.result.is_some_and(|(data, _)| data != DataState::Full) {
            wrong_input = true;
        }
        if block.tally > 1 {
            double_tally = true;
        }
        if checked && block.result.is_some_and(|(_, digest)| digest != phase) {
            stale_after_change = true;
        }
        if !checked {
            stuck = true;
            if let PersistedClaim::Held { card, lease_valid: true } = block.claim {
                if matches!(state.cards[usize::from(card)], CardState::Dropped) {
                    dead_block = true;
                }
            }
        }
    }
    report.missing_result += u64::from(missing_result);
    report.wrong_input += u64::from(wrong_input);
    report.double_tally += u64::from(double_tally);
    report.stale_after_change += u64::from(stale_after_change);
    report.stuck += u64::from(stuck);
    report.dead_block_terminals += u64::from(dead_block);
    report.terminal_backlog_values.insert(pending_blocks(state, form).len());
    report.terminal_held_values.insert(held_block_count(state));
}

/// 穷举一种形在一个几何点上的全部可达状态。
pub fn explore(form: &ModelForm, geometry: &Geometry) -> ModelReport {
    let mut report = ModelReport::default();
    let mut seen: HashSet<ModelState> = HashSet::new();
    let mut stack = vec![initial_state(geometry)];
    while let Some(state) = stack.pop() {
        if !seen.insert(state.clone()) {
            continue;
        }
        if seen.len() as u64 > REACHABLE_STATE_LIMIT {
            report.stopped_over_limit = true;
            break;
        }
        let backlog = pending_blocks(&state, form).len();
        report.backlog_peak = report.backlog_peak.max(backlog);
        report.backlog_positive_states += u64::from(backlog > 0);
        let held = held_block_count(&state);
        report.held_peak = report.held_peak.max(held);
        report.held_positive_states += u64::from(held > 0);
        let mut successors = Vec::new();
        recorder_successors(&state, form, geometry, &mut successors);
        card_successors(&state, form, geometry, &mut successors);
        environment_successors(&state, form, &mut successors);
        if successors.is_empty() {
            tally_terminal(&state, form, &mut report);
            continue;
        }
        for (next, kind) in successors {
            match kind {
                TransitionKind::Ordinary => {}
                TransitionKind::Kill => report.kill_transitions += 1,
                TransitionKind::JudgingChange => report.judging_change_transitions += 1,
                TransitionKind::CardDrop => report.card_drop_transitions += 1,
                TransitionKind::TakeoverAfterLease => report.takeover_after_lease_transitions += 1,
            }
            stack.push(next);
        }
    }
    report.reachable_states = seen.len() as u64;
    report
}

/// 跑前登记 5.6 的最强形：录入原子写、核对一批原子写、标记记判法摘要、结果按块键覆盖、领块只在内存、中途换一次判法、可被杀。
pub const STRONGEST_FORM: ModelForm = ModelForm {
    record_order: RecordOrder::Atomic,
    check_order: CheckOrder::AtomicBatchKeyed,
    flag_records_digest: true,
    flush_when_blocked: true,
    claim_form: ClaimForm::InMemory,
    cards_can_drop: false,
    allow_kills: true,
    allow_judging_change: true,
    late_writer_appends: false,
    lease_can_expire: true,
};

/// F10 的六种形：最强形与五种弱形，每种弱形只改最强形的一样。
pub fn pipeline_forms() -> Vec<(&'static str, ModelForm)> {
    vec![
        ("strongest", STRONGEST_FORM),
        ("record_flag_before_data", ModelForm { record_order: RecordOrder::FlagBeforeData, ..STRONGEST_FORM }),
        ("checked_flag_before_result", ModelForm { check_order: CheckOrder::FlagBeforeResult, ..STRONGEST_FORM }),
        ("result_then_flag_with_appended_tally", ModelForm { check_order: CheckOrder::ResultThenFlagAppended, ..STRONGEST_FORM }),
        ("checked_flag_is_a_boolean", ModelForm { flag_records_digest: false, ..STRONGEST_FORM }),
        ("batch_larger_than_backlog_cap_without_flush", ModelForm { flush_when_blocked: false, ..STRONGEST_FORM }),
    ]
}

/// F11 的三形：都开卡掉线，其余取最强形。
pub fn claim_forms() -> Vec<(&'static str, ModelForm)> {
    let with_drops = ModelForm { cards_can_drop: true, ..STRONGEST_FORM };
    vec![
        ("claims_persisted_without_lease", ModelForm { claim_form: ClaimForm::PersistedWithoutLease, ..with_drops }),
        ("claims_in_memory", ModelForm { claim_form: ClaimForm::InMemory, ..with_drops }),
        ("claims_persisted_with_lease", ModelForm { claim_form: ClaimForm::PersistedWithLease, ..with_drops }),
    ]
}

/// 跑前登记 5.6 的几何：块数 2 与 3、卡数 1 与 2、积压上限 1 与 2、批 1 与 2。
pub fn geometries() -> Vec<Geometry> {
    let mut points = Vec::new();
    for blocks in [2usize, 3] {
        for cards in [1usize, 2] {
            for backlog_cap in [1usize, 2] {
                for batch in [1usize, 2] {
                    points.push(Geometry { blocks, cards, backlog_cap, batch });
                }
            }
        }
    }
    points
}
