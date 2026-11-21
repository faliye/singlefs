//! 候选，世界与它的交错，按枚举与按搜索求每个目标状态的真值，必须先落盘的掩码与可达状态。

use super::*;

// ── 世界：一个 Q1 候选 × 一个几何 ───────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Candidate {
    /// 准备并行、落盘串行：只有 chain(A)·chain(B) 一条发出流。
    SerialPipeline,
    /// 一次发布装多个事务：一个根、一次系统配置写。
    OnePublishManyTransactions,
    /// 多次发布同时在飞，两条链全部交错。
    ManyPublishesInFlightFree,
    /// 多次发布同时在飞，且 rA 先于 rB、RA 先于 RB、SA 先于 SB 发出。
    ManyPublishesInFlightOrdered,
}

impl Candidate {
    pub(super) fn name(self) -> &'static str {
        match self {
            Candidate::SerialPipeline => "serial_pipeline",
            Candidate::OnePublishManyTransactions => "one_publish_many_transactions",
            Candidate::ManyPublishesInFlightFree => "many_publishes_in_flight_free",
            Candidate::ManyPublishesInFlightOrdered => "many_publishes_in_flight_ordered",
        }
    }
}

/// Σ_ord 要求 A 先于 B 发出的步骤种类（登记第一节：rA 先于 rB、RA 先于 RB、SA 先于 SB）。
const ORDERED_KINDS: &[StepKind] = &[StepKind::JournalRecord, StepKind::RootRecordFua, StepKind::SystemConfigurationSlot];

pub(super) struct World {
    pub(super) candidate: Candidate,
    pub(super) geometry: Geometry,
    pub(super) table: Vec<WriteSpecification>,
    /// Σ(P)：这个候选允许的全部发出流。
    pub(super) streams: Vec<Vec<Event>>,
    /// 逐状态深搜用的两条链；一次发布装多个事务时第一条是整条流、第二条是空的。
    pub(super) search_chains: [Vec<Event>; 2],
    /// 第二条链的第几步要等第一条链走过哪一步（走过下标之后）才能发出。
    pub(super) search_waits_of_second_chain: Vec<Option<usize>>,
    /// 按事务切那条臂用的两条链。
    pub(super) arm_chains: [Vec<Event>; 2],
    /// 按事务切那条臂在一次发布装多个事务时多出来的发布尾巴（根槽写、系统配置槽写、屏障）。
    pub(super) arm_tail: Vec<Event>,
    /// chain(A)·chain(B)，一次发布装多个事务时没有。
    pub(super) serial_stream: Option<Vec<Event>>,
}

pub(super) fn ordering_waits(table: &[WriteSpecification], older_chain: &[Event], newer_chain: &[Event]) -> Vec<Option<usize>> {
    newer_chain
        .iter()
        .map(|event| match event {
            Event::Write(write_index) if ORDERED_KINDS.contains(&table[*write_index].kind) => {
                let wanted = &table[*write_index];
                // 记录与系统配置槽写一份落一块盘：rB@d 等 rA@d、SB@d 等 SA@d；根槽写只有一份，落哪块盘由 txg 定，RB 等 RA 与盘无关
                // （一盘几何里三种写各只有一块盘，两种写法同值；两盘几何的读法是第二段补的，见登记修订 7）。
                older_chain.iter().position(|candidate_event| match candidate_event {
                    Event::Write(candidate_index) => {
                        table[*candidate_index].kind == wanted.kind && (wanted.kind == StepKind::RootRecordFua || table[*candidate_index].disk == wanted.disk)
                    }
                    Event::Barrier(_) => false,
                })
            }
            Event::Write(_) | Event::Barrier(_) => None,
        })
        .collect()
}

fn all_interleavings(older_chain: &[Event], newer_chain: &[Event], newer_waits: &[Option<usize>]) -> Vec<Vec<Event>> {
    let mut results = Vec::new();
    let mut current = Vec::new();
    extend_interleavings(older_chain, newer_chain, newer_waits, 0, 0, &mut current, &mut results);
    results
}

fn extend_interleavings(
    older_chain: &[Event],
    newer_chain: &[Event],
    newer_waits: &[Option<usize>],
    older_position: usize,
    newer_position: usize,
    current: &mut Vec<Event>,
    results: &mut Vec<Vec<Event>>,
) {
    if older_position == older_chain.len() && newer_position == newer_chain.len() {
        results.push(current.clone());
        return;
    }
    if older_position < older_chain.len() {
        current.push(older_chain[older_position]);
        extend_interleavings(older_chain, newer_chain, newer_waits, older_position + 1, newer_position, current, results);
        current.pop();
    }
    if newer_position < newer_chain.len() {
        let newer_transaction_may_start = true;
        let waits_are_satisfied = newer_waits[newer_position].is_none_or(|needed| older_position > needed);
        if newer_transaction_may_start && waits_are_satisfied {
            current.push(newer_chain[newer_position]);
            extend_interleavings(older_chain, newer_chain, newer_waits, older_position, newer_position + 1, current, results);
            current.pop();
        }
    }
}

pub(super) fn build_world(candidate: Candidate, geometry: &Geometry) -> World {
    match candidate {
        Candidate::OnePublishManyTransactions => build_one_publish_world(geometry),
        Candidate::SerialPipeline | Candidate::ManyPublishesInFlightFree | Candidate::ManyPublishesInFlightOrdered => {
            build_two_chain_world(candidate, geometry)
        }
    }
}

fn build_two_chain_world(candidate: Candidate, geometry: &Geometry) -> World {
    let mut table = Vec::new();
    let older_writes = push_transaction_writes(&mut table, geometry, Owner::OlderTransaction, geometry.root_disk_of_the_older_transaction);
    let newer_writes = push_transaction_writes(&mut table, geometry, Owner::NewerTransaction, geometry.root_disk_of_the_newer_transaction);
    let older_chain = single_transaction_chain(&older_writes, geometry.disk_count);
    let newer_chain = single_transaction_chain(&newer_writes, geometry.disk_count);
    let mut serial_stream = older_chain.clone();
    serial_stream.extend(newer_chain.iter().copied());
    let (streams, waits) = match candidate {
        Candidate::SerialPipeline => (vec![serial_stream.clone()], vec![Some(older_chain.len() - 1); newer_chain.len()]),
        Candidate::ManyPublishesInFlightFree => {
            let no_waits = vec![None; newer_chain.len()];
            (all_interleavings(&older_chain, &newer_chain, &no_waits), no_waits)
        }
        Candidate::ManyPublishesInFlightOrdered => {
            let waits = ordering_waits(&table, &older_chain, &newer_chain);
            (all_interleavings(&older_chain, &newer_chain, &waits), waits)
        }
        Candidate::OnePublishManyTransactions => unreachable!("一次发布装多个事务不走两条链的交错"),
    };
    World {
        candidate,
        geometry: geometry.clone(),
        table,
        streams,
        search_chains: [older_chain.clone(), newer_chain.clone()],
        search_waits_of_second_chain: waits,
        arm_chains: [older_chain, newer_chain],
        arm_tail: Vec::new(),
        serial_stream: Some(serial_stream),
    }
}

/// `a1 a2 b1 b2 屏障 rA rB 屏障 R(FUA) S 屏障`（登记第一节 Σ(P2)）。
fn build_one_publish_world(geometry: &Geometry) -> World {
    let mut table = Vec::new();
    let older_units = push_units(&mut table, geometry, Owner::OlderTransaction);
    let newer_units = push_units(&mut table, geometry, Owner::NewerTransaction);
    let older_records = push_records(&mut table, geometry, Owner::OlderTransaction);
    let newer_records = push_records(&mut table, geometry, Owner::NewerTransaction);
    let root = push_write(&mut table, geometry, "R", geometry.root_disk_of_the_older_transaction, StepKind::RootRecordFua, Owner::Shared);
    let system_slots: Vec<usize> = (0..geometry.disk_count)
        .map(|disk| push_write(&mut table, geometry, "S", disk, StepKind::SystemConfigurationSlot, Owner::Shared))
        .collect();
    let barriers = barrier_on_every_disk(geometry.disk_count);
    let mut stream = writes_as_events(&older_units);
    stream.extend(writes_as_events(&newer_units));
    stream.extend(barriers.iter().copied());
    stream.extend(writes_as_events(&older_records));
    stream.extend(writes_as_events(&newer_records));
    stream.extend(barriers.iter().copied());
    let mut tail = vec![Event::Write(root)];
    tail.extend(writes_as_events(&system_slots));
    tail.extend(barriers.iter().copied());
    stream.extend(tail.iter().copied());
    let arm_chain = |units: &[usize], records: &[usize]| {
        let mut events = writes_as_events(units);
        events.extend(barriers.iter().copied());
        events.extend(writes_as_events(records));
        events
    };
    let arm_chains = [arm_chain(&older_units, &older_records), arm_chain(&newer_units, &newer_records)];
    World {
        candidate: Candidate::OnePublishManyTransactions,
        geometry: geometry.clone(),
        table,
        streams: vec![stream.clone()],
        search_chains: [stream, Vec::new()],
        search_waits_of_second_chain: Vec::new(),
        arm_chains,
        arm_tail: tail,
        serial_stream: None,
    }
}

// ── 真值的两条算法 ──────────────────────────────────────────────────────────────────────

/// 算法 (a)：枚举 Σ(P) 里每一条发出流，取枚举域的并。
pub(super) fn truth_by_enumeration(world: &World) -> StateList {
    let mut all_states: Vec<StateCode> = Vec::new();
    for stream in &world.streams {
        all_states.extend(crash_states_of_stream(&world.table, stream));
    }
    sorted_deduplicated(all_states)
}

/// 状态全集：每次写取它的全部几态的全部组合。
pub(super) fn all_state_vectors(table: &[WriteSpecification]) -> StateList {
    let every_write: Vec<usize> = (0..table.len()).collect();
    let mut vectors = Vec::new();
    for_each_combination(table, &every_write, 0, |code| vectors.push(code));
    sorted_deduplicated(vectors)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum SegmentRole {
    /// 这一段在「当前段」之前：目标向量里它的写必须都是持久。
    Before,
    /// 这一段就是「当前段」：目标向量里它的写取什么都行。
    Current,
    /// 这一段在「当前段」之后：它的写必须都是没持久。
    After,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct SearchState {
    pub(super) older_position: usize,
    pub(super) newer_position: usize,
    unreleased_disks: u32,
    open_segment_role: Option<SegmentRole>,
    last_closed_role: SegmentRole,
}

fn role_may_follow(last_closed_role: SegmentRole, candidate: SegmentRole) -> bool {
    candidate >= last_closed_role && !(candidate == SegmentRole::Current && last_closed_role == SegmentRole::Current)
}

fn value_is_allowed_in_role(role: SegmentRole, value: u8) -> bool {
    match role {
        SegmentRole::Before => value == PERSISTED,
        SegmentRole::Current => true,
        SegmentRole::After => value == NOT_PERSISTED,
    }
}

fn close_if_every_disk_is_released(mut state: SearchState) -> SearchState {
    if let Some(role) = state.open_segment_role {
        if state.unreleased_disks == 0 {
            state.last_closed_role = role;
            state.open_segment_role = None;
        }
    }
    state
}

fn search_states_after_event(table: &[WriteSpecification], state: SearchState, event: Event, target: StateCode) -> Vec<SearchState> {
    match event {
        Event::Barrier(disk) => {
            let mut next = state;
            next.unreleased_disks &= !(1u32 << disk);
            vec![close_if_every_disk_is_released(next)]
        }
        Event::Write(write_index) => {
            let value = value_of(target, write_index);
            let roles_to_try: Vec<SegmentRole> = match state.open_segment_role {
                Some(role) => vec![role],
                None => [SegmentRole::Before, SegmentRole::Current, SegmentRole::After]
                    .into_iter()
                    .filter(|role| role_may_follow(state.last_closed_role, *role))
                    .collect(),
            };
            let disk_bit = 1u32 << table[write_index].disk;
            roles_to_try
                .into_iter()
                .filter(|role| value_is_allowed_in_role(*role, value))
                .map(|role| {
                    let mut next = state;
                    next.open_segment_role = Some(role);
                    if table[write_index].kind == StepKind::RootRecordFua {
                        next.unreleased_disks &= !disk_bit;
                        close_if_every_disk_is_released(next)
                    } else {
                        next.unreleased_disks |= disk_bit;
                        next
                    }
                })
                .collect()
        }
    }
}

/// 算法 (b)：对一个目标向量，在两条链的交错上搜「有没有一条发出流让它落进枚举域」。
pub(super) fn some_stream_realizes_target(world: &World, target: StateCode) -> bool {
    let [older_chain, newer_chain] = &world.search_chains;
    let initial = SearchState {
        older_position: 0,
        newer_position: 0,
        unreleased_disks: 0,
        open_segment_role: None,
        last_closed_role: SegmentRole::Before,
    };
    let mut visited: HashSet<SearchState> = HashSet::new();
    let mut pending: VecDeque<SearchState> = VecDeque::new();
    pending.push_back(initial);
    while let Some(state) = pending.pop_front() {
        if !visited.insert(state) {
            continue;
        }
        if state.older_position == older_chain.len() && state.newer_position == newer_chain.len() {
            return true;
        }
        if state.older_position < older_chain.len() {
            for mut next in search_states_after_event(&world.table, state, older_chain[state.older_position], target) {
                next.older_position += 1;
                pending.push_back(next);
            }
        }
        if state.newer_position < newer_chain.len() {
            let wait = world.search_waits_of_second_chain.get(state.newer_position).copied().flatten();
            if wait.is_none_or(|needed| state.older_position > needed) {
                for mut next in search_states_after_event(&world.table, state, newer_chain[state.newer_position], target) {
                    next.newer_position += 1;
                    pending.push_back(next);
                }
            }
        }
    }
    false
}

pub(super) fn truth_by_search(world: &World) -> StateList {
    all_state_vectors(&world.table)
        .into_iter()
        .filter(|target| some_stream_realizes_target(world, *target))
        .collect()
}

// ── 精确可达集 T_exact（只核参照）──────────────────────────────────────────────────────────

/// 偏序：一次写 w 落在盘 d 上，流里 w 之后、在 d 上有一道屏障，则 w 先于那道屏障之后发出的每一次写；FUA 写先于它后面的每一次写。
/// 返回每次写的「必须先持久」集合（传递闭包之后）。
pub(super) fn must_persist_before_masks(table: &[WriteSpecification], stream: &[Event]) -> Vec<u32> {
    let mut masks = vec![0u32; table.len()];
    let mut issued: Vec<(usize, bool)> = Vec::new();
    for &event in stream {
        match event {
            Event::Write(write_index) => {
                for &(earlier, armed_by_barrier) in &issued {
                    if armed_by_barrier || table[earlier].kind == StepKind::RootRecordFua {
                        masks[write_index] |= 1u32 << earlier;
                    }
                }
                issued.push((write_index, false));
            }
            Event::Barrier(disk) => {
                for entry in &mut issued {
                    if table[entry.0].disk == disk {
                        entry.1 = true;
                    }
                }
            }
        }
    }
    loop {
        let mut changed = false;
        for write_index in 0..table.len() {
            let mut extended = masks[write_index];
            for earlier in 0..table.len() {
                if masks[write_index] & (1u32 << earlier) != 0 {
                    extended |= masks[earlier];
                }
            }
            if extended != masks[write_index] {
                masks[write_index] = extended;
                changed = true;
            }
        }
        if !changed {
            return masks;
        }
    }
}

/// 一份偏序的全部可达状态：偏序的下闭集，原地覆写的撕裂态只在它是下闭集的极大元时取。
pub(super) fn reachable_states_of_partial_order(table: &[WriteSpecification], masks: &[u32]) -> Vec<StateCode> {
    assert!(table.len() <= 20, "T_exact 按子集枚举，写数要 ≤ 20");
    let mut states = Vec::new();
    for member_mask in 0u32..(1u32 << table.len()) {
        let is_down_closed = (0..table.len()).all(|write_index| member_mask & (1u32 << write_index) == 0 || masks[write_index] & !member_mask == 0);
        if !is_down_closed {
            continue;
        }
        let maximal_in_place: Vec<usize> = (0..table.len())
            .filter(|&write_index| {
                table[write_index].is_in_place
                    && member_mask & (1u32 << write_index) != 0
                    && (0..table.len()).all(|other| member_mask & (1u32 << other) == 0 || masks[other] & (1u32 << write_index) == 0)
            })
            .collect();
        let mut base: StateCode = 0;
        for write_index in 0..table.len() {
            if member_mask & (1u32 << write_index) != 0 {
                base = with_value(base, write_index, PERSISTED);
            }
        }
        for torn_subset in 0u32..(1u32 << maximal_in_place.len()) {
            let mut state = base;
            for (position, &write_index) in maximal_in_place.iter().enumerate() {
                if torn_subset & (1u32 << position) != 0 {
                    state = (state & !(0b11u128 << (BITS_PER_WRITE * write_index))) | (StateCode::from(TORN) << (BITS_PER_WRITE * write_index));
                }
            }
            states.push(state);
        }
    }
    states
}

pub(super) fn exact_reachable_states(world: &World) -> StateList {
    let mut distinct_masks: HashSet<Vec<u32>> = HashSet::new();
    for stream in &world.streams {
        distinct_masks.insert(must_persist_before_masks(&world.table, stream));
    }
    let mut states = Vec::new();
    for masks in &distinct_masks {
        states.extend(reachable_states_of_partial_order(&world.table, masks));
    }
    sorted_deduplicated(states)
}

// ── 两条臂 ─────────────────────────────────────────────────────────────────────────────

fn persisted_everything_in(chain: &[Event]) -> StateCode {
    chain.iter().fold(0, |code, event| match event {
        Event::Write(write_index) => with_value(code, *write_index, PERSISTED),
        Event::Barrier(_) => code,
    })
}

/// 按事务切那条臂：两条单事务链各自枚举域的积；一次发布装多个事务时再并上发布尾巴的状态。
pub(super) fn arm_by_transaction_states(world: &World) -> StateList {
    let older_states = crash_states_of_stream(&world.table, &world.arm_chains[0]);
    let newer_states = crash_states_of_stream(&world.table, &world.arm_chains[1]);
    let older_full_state = persisted_everything_in(&world.arm_chains[0]);
    let newer_full_state = persisted_everything_in(&world.arm_chains[1]);
    assert!(contains(&older_states, older_full_state) && contains(&newer_states, newer_full_state), "链全持久的那个状态在自己的枚举域里");
    let mut product = Vec::new();
    for &older_state in &older_states {
        for &newer_state in &newer_states {
            product.push(older_state | newer_state);
        }
    }
    if !world.arm_tail.is_empty() {
        for tail_state in crash_states_of_stream(&world.table, &world.arm_tail) {
            if tail_state != 0 {
                product.push(older_full_state | newer_full_state | tail_state);
            }
        }
    }
    sorted_deduplicated(product)
}

/// 阳性对照 PC1 在按事务切那条臂上的形态：把积换成串行拼接（A 的各状态 × B 全不持久，并上 A 全持久 × B 的各状态）。
pub(super) fn serial_concatenation_of_transaction_chains(world: &World) -> StateList {
    let older_states = crash_states_of_stream(&world.table, &world.arm_chains[0]);
    let newer_states = crash_states_of_stream(&world.table, &world.arm_chains[1]);
    let older_full_state = persisted_everything_in(&world.arm_chains[0]);
    let mut states = older_states;
    states.extend(newer_states.into_iter().map(|newer_state| older_full_state | newer_state));
    sorted_deduplicated(states)
}

// ── 判据用的谓词与计数 ──────────────────────────────────────────────────────────────────

fn writes_of_owner(table: &[WriteSpecification], owner: Owner) -> Vec<usize> {
    (0..table.len()).filter(|&write_index| table[write_index].owner == owner).collect()
}

/// 跨事务状态 C：A、B 各至少有一次写不是「没持久」。
pub(super) fn is_cross_transaction(table: &[WriteSpecification], state: StateCode) -> bool {
    let some_nonzero = |owner: Owner| writes_of_owner(table, owner).into_iter().any(|write_index| value_of(state, write_index) != NOT_PERSISTED);
    some_nonzero(Owner::OlderTransaction) && some_nonzero(Owner::NewerTransaction)
}

fn records_of_owner_all_persisted(table: &[WriteSpecification], state: StateCode, owner: Owner) -> bool {
    (0..table.len())
        .filter(|&write_index| table[write_index].owner == owner && table[write_index].kind == StepKind::JournalRecord)
        .all(|write_index| value_of(state, write_index) == PERSISTED)
}

/// I_rec（按记录）：B 的记录都持久而 A 的记录不都持久。
pub(super) fn is_invalid_prefix_by_record(table: &[WriteSpecification], state: StateCode) -> bool {
    let older_transaction_records_all_persisted = records_of_owner_all_persisted(table, state, Owner::OlderTransaction);
    let newer_transaction_records_all_persisted = records_of_owner_all_persisted(table, state, Owner::NewerTransaction);
    newer_transaction_records_all_persisted && !older_transaction_records_all_persisted
}

/// I_full（按全部写）：B 的每次写都持久，而 A 至少一次写不是「没持久」、又不是每次写都持久。
pub(super) fn is_invalid_prefix_by_all_writes(table: &[WriteSpecification], state: StateCode) -> bool {
    let values_of = |owner: Owner| -> Vec<u8> { writes_of_owner(table, owner).into_iter().map(|write_index| value_of(state, write_index)).collect() };
    let older_values = values_of(Owner::OlderTransaction);
    let newer_values = values_of(Owner::NewerTransaction);
    newer_values.iter().all(|value| *value == PERSISTED)
        && older_values.iter().any(|value| *value != NOT_PERSISTED)
        && older_values.iter().any(|value| *value != PERSISTED)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum InvalidPrefixKind {
    ByRecord,
    ByAllWrites,
}

impl InvalidPrefixKind {
    pub(super) fn name(self) -> &'static str {
        match self {
            InvalidPrefixKind::ByRecord => "recorded",
            InvalidPrefixKind::ByAllWrites => "all_writes",
        }
    }

    pub(super) fn holds(self, table: &[WriteSpecification], state: StateCode) -> bool {
        match self {
            InvalidPrefixKind::ByRecord => is_invalid_prefix_by_record(table, state),
            InvalidPrefixKind::ByAllWrites => is_invalid_prefix_by_all_writes(table, state),
        }
    }
}
