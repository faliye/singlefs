//! E167（多在飞发布的崩溃枚举切法）：两个事务各两个单元的小流上，「照旧按根槽写切」与「按事务切」两条臂各自枚举出多少崩溃状态、
//! 覆盖多少不合法前缀、漏不漏跨两个事务的落盘子集。
//!
//! 跑前登记 `research/prompts/e167-preregistration.md` 写死了这个装置：**独立手写计数模型，不与 `crates/` 共用代码**（不是入库装置）。
//! 模型的口径只来自登记第二节抄下的 D13（验证路线）已定项 4（枚举域）、D16（发布语义）已定项 7（单事务写链的次序）与已定项 2 射程那一句：
//!
//! - 崩溃状态按设备切段：一次写只被它自己那块盘上之后的屏障（或那块盘上的 FUA 写）排在之后的写前面；当前段里每块有写的盘都被自己的屏障或
//!   FUA 放行，这一段才关；FUA 只放行它自己那块盘，与那块盘上前面没被屏障隔开的写同段；一个状态 = 前若干段全持久、当前段每次写各取它的几态
//!   任意组合、之后各段全不持久；原地覆写（系统配置槽写）多取第三态「撕裂」。
//! - 真值 T(P) = P 允许的全部发出流上这套枚举域的并；另有一份按设备偏序算的精确可达集 T_exact 只核参照。
//! - 臂 K1（按根槽写切）= 录到的那一条发出流上的枚举域；臂 K2（按事务切）= 两条单事务链各自枚举域的积。
//!
//! 子命令：不带参数跑第一段的全部量；`halt` 只跑登记第十一节的停机条款 S1、S2。

// admission: always 独立手写的计数模型，每次现算；不读 crates/，只读自己的常量
// run-condition: command python3
use e7_index_bench::Emitter;
use std::collections::{HashSet, VecDeque};

// ── 状态编码：每次写两位，0 没持久、1 持久、2 撕裂 ────────────────────────────────────────

type StateCode = u128;
/// 排好序、去了重的状态向量表：成员判断用二分。
type StateList = Vec<StateCode>;

const NOT_PERSISTED: u8 = 0;
const PERSISTED: u8 = 1;
const TORN: u8 = 2;
const BITS_PER_WRITE: usize = 2;
const MAXIMUM_WRITES_IN_STATE_CODE: usize = 64;

/// 崩溃放量装置 GPU 持久掩码的位数：`crates/singlefs-checker-tier/src/crash_facts.rs:129` `MAXIMUM_ENUMERATED_WRITES`（登记第三节，结构常量）。
const MAXIMUM_ENUMERATED_WRITES: usize = 2048;
/// 一段展开出的状态数要装得进 u32：`crates/singlefs-checker-tier/src/crash_verify_gpu.rs:305` 的断言（登记第三节，结构常量）。
const MAXIMUM_SEGMENT_STATES_IN_U32: u128 = u32::MAX as u128;
/// 一段的状态数要装得进 u64：`crates/singlefs-checker-tier/src/crash.rs:1681`（登记第三节，结构常量）。
const MAXIMUM_SEGMENT_STATES_IN_U64: u128 = u64::MAX as u128;

fn value_of(state: StateCode, write_index: usize) -> u8 {
    u8::try_from((state >> (BITS_PER_WRITE * write_index)) & 0b11).expect("两位取值装得进 u8")
}

fn with_value(state: StateCode, write_index: usize, value: u8) -> StateCode {
    state | (StateCode::from(value) << (BITS_PER_WRITE * write_index))
}

fn contains(states: &[StateCode], state: StateCode) -> bool {
    states.binary_search(&state).is_ok()
}

fn sorted_deduplicated(mut states: Vec<StateCode>) -> StateList {
    states.sort_unstable();
    states.dedup();
    states
}

// ── 写、事件、几何 ─────────────────────────────────────────────────────────────────────

/// 步骤种类，声明次序就是登记表「种类」串里的规范次序（登记表 `.claude/kb/layout/01-first-txn.md` 第 403 行）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum StepKind {
    UnitWrite,
    JournalRecord,
    RootRecordFua,
    SystemConfigurationSlot,
    Barrier,
}

impl StepKind {
    fn registered_name(self) -> &'static str {
        match self {
            StepKind::UnitWrite => "unit_write",
            StepKind::JournalRecord => "journal_record",
            StepKind::RootRecordFua => "root_record_fua",
            StepKind::SystemConfigurationSlot => "system_configuration_slot",
            StepKind::Barrier => "barrier",
        }
    }

    fn from_registered_name(name: &str) -> StepKind {
        match name {
            "unit_write" => StepKind::UnitWrite,
            "journal_record" => StepKind::JournalRecord,
            "root_record_fua" => StepKind::RootRecordFua,
            "system_configuration_slot" => StepKind::SystemConfigurationSlot,
            "barrier" => StepKind::Barrier,
            other => panic!("登记表里没有这个步骤种类：{other}"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Owner {
    TransactionA,
    TransactionB,
    /// 一次发布装多个事务时，根槽写与系统配置槽写不属于任何一个事务。
    Shared,
}

#[derive(Clone, Debug)]
struct WriteSpec {
    name: String,
    disk: usize,
    kind: StepKind,
    owner: Owner,
    is_in_place: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Event {
    Write(usize),
    Barrier(usize),
}

/// 一次崩溃枚举的几何：每个事务几个单元、几块盘、根槽写落哪块盘、系统配置槽写取不取第三态。
#[derive(Clone, Debug)]
struct Geometry {
    label: &'static str,
    units_per_transaction: usize,
    disk_count: usize,
    root_disk_of_transaction_a: usize,
    root_disk_of_transaction_b: usize,
    system_configuration_has_third_state: bool,
}

/// D16（发布语义）已定项 8：根环按 txg 对 3 取余落到区域 0 / 1 / 2，三个区域落盘 0 / 1 / 0（登记第八节 G4a、G4b 的出处）。
const ROOT_REGION_DISK: [usize; 3] = [0, 1, 0];

fn root_disk_of_txg(txg: usize) -> usize {
    ROOT_REGION_DISK[txg % 3]
}

fn geometry_one_disk(label: &'static str, units_per_transaction: usize, third_state: bool) -> Geometry {
    Geometry {
        label,
        units_per_transaction,
        disk_count: 1,
        root_disk_of_transaction_a: 0,
        root_disk_of_transaction_b: 0,
        system_configuration_has_third_state: third_state,
    }
}

/// 两盘几何：A 的 txg 给定，B 的 txg 是它加一（登记第一节记号：A 是 N，B 是 N + 1）。
fn geometry_two_disks(label: &'static str, units_per_transaction: usize, txg_of_a: usize) -> Geometry {
    Geometry {
        label,
        units_per_transaction,
        disk_count: 2,
        root_disk_of_transaction_a: root_disk_of_txg(txg_of_a),
        root_disk_of_transaction_b: root_disk_of_txg(txg_of_a + 1),
        system_configuration_has_third_state: true,
    }
}

fn geometry_g1() -> Geometry {
    geometry_one_disk("G1", 2, true)
}

fn geometry_g3() -> Geometry {
    geometry_one_disk("G3", 1, true)
}

fn geometry_g4a() -> Geometry {
    geometry_two_disks("G4a", 2, 3)
}

fn geometry_g4b() -> Geometry {
    geometry_two_disks("G4b", 2, 2)
}

fn geometry_g5() -> Geometry {
    geometry_one_disk("G5", 2, false)
}

/// 登记表普通发布那一行的形：每个事务 12 个单元、两盘、两态（`.claude/kb/layout/01-first-txn.md` 第 403 行）。
fn geometry_registered_publish_two_state() -> Geometry {
    Geometry {
        label: "registered_publish",
        units_per_transaction: 12,
        disk_count: 2,
        root_disk_of_transaction_a: 0,
        root_disk_of_transaction_b: 0,
        system_configuration_has_third_state: false,
    }
}

/// 真实发布形（登记第四节第 7 条）：每个事务 12 个单元、两盘、系统配置槽写取三态。
fn geometry_real_publish_shape() -> Geometry {
    Geometry {
        label: "real_publish_shape",
        units_per_transaction: 12,
        disk_count: 2,
        root_disk_of_transaction_a: 0,
        root_disk_of_transaction_b: 1,
        system_configuration_has_third_state: true,
    }
}

fn disk_suffix(disk_count: usize, disk: usize) -> String {
    if disk_count == 1 {
        String::new()
    } else {
        format!("@{disk}")
    }
}

fn push_write(
    table: &mut Vec<WriteSpec>,
    geometry: &Geometry,
    name: &str,
    disk: usize,
    kind: StepKind,
    owner: Owner,
) -> usize {
    let is_in_place = kind == StepKind::SystemConfigurationSlot && geometry.system_configuration_has_third_state;
    table.push(WriteSpec {
        name: format!("{name}{}", disk_suffix(geometry.disk_count, disk)),
        disk,
        kind,
        owner,
        is_in_place,
    });
    assert!(table.len() <= MAXIMUM_WRITES_IN_STATE_CODE, "写表装不进状态编码");
    table.len() - 1
}

fn lower_and_upper_tag(owner: Owner) -> (&'static str, &'static str) {
    match owner {
        Owner::TransactionA => ("a", "A"),
        Owner::TransactionB => ("b", "B"),
        Owner::Shared => unreachable!("共享的写不按事务起名"),
    }
}

fn push_units(table: &mut Vec<WriteSpec>, geometry: &Geometry, owner: Owner) -> Vec<usize> {
    let (lower, _) = lower_and_upper_tag(owner);
    let mut units = Vec::new();
    for unit_number in 1..=geometry.units_per_transaction {
        for disk in 0..geometry.disk_count {
            units.push(push_write(table, geometry, &format!("{lower}{unit_number}"), disk, StepKind::UnitWrite, owner));
        }
    }
    units
}

fn push_records(table: &mut Vec<WriteSpec>, geometry: &Geometry, owner: Owner) -> Vec<usize> {
    let (_, upper) = lower_and_upper_tag(owner);
    (0..geometry.disk_count)
        .map(|disk| push_write(table, geometry, &format!("r{upper}"), disk, StepKind::JournalRecord, owner))
        .collect()
}

struct TransactionWrites {
    units: Vec<usize>,
    records: Vec<usize>,
    root: usize,
    system_slots: Vec<usize>,
}

fn push_transaction_writes(table: &mut Vec<WriteSpec>, geometry: &Geometry, owner: Owner, root_disk: usize) -> TransactionWrites {
    let (_, upper) = lower_and_upper_tag(owner);
    let units = push_units(table, geometry, owner);
    let records = push_records(table, geometry, owner);
    let root = push_write(table, geometry, &format!("R{upper}"), root_disk, StepKind::RootRecordFua, owner);
    let system_slots = (0..geometry.disk_count)
        .map(|disk| push_write(table, geometry, &format!("S{upper}"), disk, StepKind::SystemConfigurationSlot, owner))
        .collect();
    TransactionWrites { units, records, root, system_slots }
}

fn writes_as_events(write_indexes: &[usize]) -> Vec<Event> {
    write_indexes.iter().map(|&write_index| Event::Write(write_index)).collect()
}

/// 一道池屏障 = 每块盘各一步屏障（登记第一节读法）。
fn barrier_on_every_disk(disk_count: usize) -> Vec<Event> {
    (0..disk_count).map(Event::Barrier).collect()
}

/// D16（发布语义）已定项 7 的持久顺序：单元 → 屏障 → 记录 → 屏障 → 根槽（FUA）→ 系统配置槽 → 屏障。
fn single_transaction_chain(writes: &TransactionWrites, disk_count: usize) -> Vec<Event> {
    let mut events = writes_as_events(&writes.units);
    events.extend(barrier_on_every_disk(disk_count));
    events.extend(writes_as_events(&writes.records));
    events.extend(barrier_on_every_disk(disk_count));
    events.push(Event::Write(writes.root));
    events.extend(writes_as_events(&writes.system_slots));
    events.extend(barrier_on_every_disk(disk_count));
    events
}

// ── 按设备切段与崩溃状态枚举（登记第二节「Dom4」）──────────────────────────────────────────

/// 录制流切成段，每段连同关它的屏障一起（登记表「种类」串把屏障也算进段里）。
fn operation_segments(table: &[WriteSpec], stream: &[Event]) -> Vec<Vec<Event>> {
    let mut segments: Vec<Vec<Event>> = Vec::new();
    let mut current: Vec<Event> = Vec::new();
    let mut writes_in_current = 0usize;
    let mut unreleased_disks: u32 = 0;
    for &event in stream {
        current.push(event);
        let mut may_close = false;
        match event {
            Event::Write(write_index) => {
                writes_in_current += 1;
                let disk_bit = 1u32 << table[write_index].disk;
                if table[write_index].kind == StepKind::RootRecordFua {
                    unreleased_disks &= !disk_bit;
                    may_close = true; // FUA 关段
                } else {
                    unreleased_disks |= disk_bit;
                }
            }
            Event::Barrier(disk) => {
                unreleased_disks &= !(1u32 << disk);
                may_close = true; // 屏障关段
            }
        }
        if may_close && writes_in_current > 0 && unreleased_disks == 0 {
            segments.push(std::mem::take(&mut current));
            writes_in_current = 0;
        }
    }
    if !current.is_empty() {
        match (writes_in_current > 0, segments.last_mut()) {
            (false, Some(last_segment)) => last_segment.append(&mut current),
            (true, _) | (false, None) => segments.push(current),
        }
    }
    segments
}

fn write_segments(table: &[WriteSpec], stream: &[Event]) -> Vec<Vec<usize>> {
    operation_segments(table, stream)
        .into_iter()
        .map(|segment| {
            segment
                .into_iter()
                .filter_map(|event| match event {
                    Event::Write(write_index) => Some(write_index),
                    Event::Barrier(_) => None,
                })
                .collect::<Vec<usize>>()
        })
        .filter(|writes| !writes.is_empty())
        .collect()
}

/// 一段里每次写各取它的几态、任意组合：两态写取 0 / 1，原地覆写取 0 / 1 / 2；`prefix` 是前面各段全持久的编码。
fn for_each_combination(table: &[WriteSpec], write_indexes: &[usize], prefix: StateCode, mut emit: impl FnMut(StateCode)) {
    let radices: Vec<u8> = write_indexes.iter().map(|&write_index| if table[write_index].is_in_place { 3 } else { 2 }).collect();
    let shifts: Vec<usize> = write_indexes.iter().map(|&write_index| BITS_PER_WRITE * write_index).collect();
    let mut digits = vec![0u8; write_indexes.len()];
    let mut code = prefix;
    loop {
        emit(code);
        let mut digit_position = 0usize;
        loop {
            if digit_position == digits.len() {
                return;
            }
            if digits[digit_position] + 1 < radices[digit_position] {
                digits[digit_position] += 1;
                code += 1u128 << shifts[digit_position];
                break;
            }
            code -= StateCode::from(digits[digit_position]) << shifts[digit_position];
            digits[digit_position] = 0;
            digit_position += 1;
        }
    }
}

fn segment_expansion(table: &[WriteSpec], segment: &[usize]) -> u128 {
    segment.iter().map(|&write_index| if table[write_index].is_in_place { 3u128 } else { 2u128 }).product()
}

/// D13（验证路线）已定项 4 的闭式：1 + Σ(2^两态写数 · 3^原地覆写数 − 1)。
fn closed_form_state_count(table: &[WriteSpec], segments: &[Vec<usize>]) -> u128 {
    1 + segments.iter().map(|segment| segment_expansion(table, segment) - 1).sum::<u128>()
}

fn crash_states_of_segments(table: &[WriteSpec], segments: &[Vec<usize>]) -> StateList {
    let expected_total = usize::try_from(closed_form_state_count(table, segments) + u128::try_from(segments.len()).expect("段数装得进 u128"))
        .expect("枚举的状态数装得进 usize");
    let mut states: Vec<StateCode> = Vec::with_capacity(expected_total);
    states.push(0);
    let mut prefix: StateCode = 0;
    for segment in segments {
        for_each_combination(table, segment, prefix, |code| states.push(code));
        for &write_index in segment {
            prefix = with_value(prefix, write_index, PERSISTED);
        }
    }
    sorted_deduplicated(states)
}

fn crash_states_of_stream(table: &[WriteSpec], stream: &[Event]) -> StateList {
    crash_states_of_segments(table, &write_segments(table, stream))
}

/// 一个状态在不在一串段的枚举域里：直接按定义判（存在一个当前段，前面全 1、后面全 0），不生成枚举域。
fn state_is_in_crash_states_of_segments(segments: &[Vec<usize>], state: StateCode) -> bool {
    if state == 0 {
        return true;
    }
    (0..segments.len()).any(|current_segment| {
        segments.iter().enumerate().all(|(segment_position, segment)| {
            segment.iter().all(|&write_index| {
                let value = value_of(state, write_index);
                if segment_position < current_segment {
                    value == PERSISTED
                } else if segment_position > current_segment {
                    value == NOT_PERSISTED
                } else {
                    true
                }
            })
        })
    })
}

// ── 世界：一个 Q1 候选 × 一个几何 ───────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Candidate {
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
    fn name(self) -> &'static str {
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

struct World {
    candidate: Candidate,
    geometry: Geometry,
    table: Vec<WriteSpec>,
    /// Σ(P)：这个候选允许的全部发出流。
    streams: Vec<Vec<Event>>,
    /// 逐状态深搜用的两条链；一次发布装多个事务时第一条是整条流、第二条是空的。
    search_chains: [Vec<Event>; 2],
    /// 第二条链的第几步要等第一条链走过哪一步（走过下标之后）才能发出。
    search_waits_of_second_chain: Vec<Option<usize>>,
    /// 按事务切那条臂用的两条链。
    arm_chains: [Vec<Event>; 2],
    /// 按事务切那条臂在一次发布装多个事务时多出来的发布尾巴（根槽写、系统配置槽写、屏障）。
    arm_tail: Vec<Event>,
    /// chain(A)·chain(B)，一次发布装多个事务时没有。
    serial_stream: Option<Vec<Event>>,
}

fn ordering_waits(table: &[WriteSpec], chain_a: &[Event], chain_b: &[Event]) -> Vec<Option<usize>> {
    chain_b
        .iter()
        .map(|event| match event {
            Event::Write(write_index) if ORDERED_KINDS.contains(&table[*write_index].kind) => {
                let wanted = &table[*write_index];
                // 记录与系统配置槽写一份落一块盘：rB@d 等 rA@d、SB@d 等 SA@d；根槽写只有一份，落哪块盘由 txg 定，RB 等 RA 与盘无关
                // （一盘几何里三种写各只有一块盘，两种写法同值；两盘几何的读法是第二段补的，见登记修订 7）。
                chain_a.iter().position(|candidate_event| match candidate_event {
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

fn all_interleavings(chain_a: &[Event], chain_b: &[Event], waits_of_b: &[Option<usize>]) -> Vec<Vec<Event>> {
    let mut results = Vec::new();
    let mut current = Vec::new();
    extend_interleavings(chain_a, chain_b, waits_of_b, 0, 0, &mut current, &mut results);
    results
}

fn extend_interleavings(
    chain_a: &[Event],
    chain_b: &[Event],
    waits_of_b: &[Option<usize>],
    position_a: usize,
    position_b: usize,
    current: &mut Vec<Event>,
    results: &mut Vec<Vec<Event>>,
) {
    if position_a == chain_a.len() && position_b == chain_b.len() {
        results.push(current.clone());
        return;
    }
    if position_a < chain_a.len() {
        current.push(chain_a[position_a]);
        extend_interleavings(chain_a, chain_b, waits_of_b, position_a + 1, position_b, current, results);
        current.pop();
    }
    if position_b < chain_b.len() {
        let transaction_b_may_start = true;
        let waits_are_satisfied = waits_of_b[position_b].map_or(true, |needed| position_a > needed);
        if transaction_b_may_start && waits_are_satisfied {
            current.push(chain_b[position_b]);
            extend_interleavings(chain_a, chain_b, waits_of_b, position_a, position_b + 1, current, results);
            current.pop();
        }
    }
}

fn build_world(candidate: Candidate, geometry: &Geometry) -> World {
    match candidate {
        Candidate::OnePublishManyTransactions => build_one_publish_world(geometry),
        Candidate::SerialPipeline | Candidate::ManyPublishesInFlightFree | Candidate::ManyPublishesInFlightOrdered => {
            build_two_chain_world(candidate, geometry)
        }
    }
}

fn build_two_chain_world(candidate: Candidate, geometry: &Geometry) -> World {
    let mut table = Vec::new();
    let writes_of_a = push_transaction_writes(&mut table, geometry, Owner::TransactionA, geometry.root_disk_of_transaction_a);
    let writes_of_b = push_transaction_writes(&mut table, geometry, Owner::TransactionB, geometry.root_disk_of_transaction_b);
    let chain_a = single_transaction_chain(&writes_of_a, geometry.disk_count);
    let chain_b = single_transaction_chain(&writes_of_b, geometry.disk_count);
    let mut serial_stream = chain_a.clone();
    serial_stream.extend(chain_b.iter().copied());
    let (streams, waits) = match candidate {
        Candidate::SerialPipeline => (vec![serial_stream.clone()], vec![Some(chain_a.len() - 1); chain_b.len()]),
        Candidate::ManyPublishesInFlightFree => {
            let no_waits = vec![None; chain_b.len()];
            (all_interleavings(&chain_a, &chain_b, &no_waits), no_waits)
        }
        Candidate::ManyPublishesInFlightOrdered => {
            let waits = ordering_waits(&table, &chain_a, &chain_b);
            (all_interleavings(&chain_a, &chain_b, &waits), waits)
        }
        Candidate::OnePublishManyTransactions => unreachable!("一次发布装多个事务不走两条链的交错"),
    };
    World {
        candidate,
        geometry: geometry.clone(),
        table,
        streams,
        search_chains: [chain_a.clone(), chain_b.clone()],
        search_waits_of_second_chain: waits,
        arm_chains: [chain_a, chain_b],
        arm_tail: Vec::new(),
        serial_stream: Some(serial_stream),
    }
}

/// `a1 a2 b1 b2 屏障 rA rB 屏障 R(FUA) S 屏障`（登记第一节 Σ(P2)）。
fn build_one_publish_world(geometry: &Geometry) -> World {
    let mut table = Vec::new();
    let units_of_a = push_units(&mut table, geometry, Owner::TransactionA);
    let units_of_b = push_units(&mut table, geometry, Owner::TransactionB);
    let records_of_a = push_records(&mut table, geometry, Owner::TransactionA);
    let records_of_b = push_records(&mut table, geometry, Owner::TransactionB);
    let root = push_write(&mut table, geometry, "R", geometry.root_disk_of_transaction_a, StepKind::RootRecordFua, Owner::Shared);
    let system_slots: Vec<usize> = (0..geometry.disk_count)
        .map(|disk| push_write(&mut table, geometry, "S", disk, StepKind::SystemConfigurationSlot, Owner::Shared))
        .collect();
    let barriers = barrier_on_every_disk(geometry.disk_count);
    let mut stream = writes_as_events(&units_of_a);
    stream.extend(writes_as_events(&units_of_b));
    stream.extend(barriers.iter().copied());
    stream.extend(writes_as_events(&records_of_a));
    stream.extend(writes_as_events(&records_of_b));
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
    let arm_chains = [arm_chain(&units_of_a, &records_of_a), arm_chain(&units_of_b, &records_of_b)];
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
fn truth_by_enumeration(world: &World) -> StateList {
    let mut all_states: Vec<StateCode> = Vec::new();
    for stream in &world.streams {
        all_states.extend(crash_states_of_stream(&world.table, stream));
    }
    sorted_deduplicated(all_states)
}

/// 状态全集：每次写取它的全部几态的全部组合。
fn all_state_vectors(table: &[WriteSpec]) -> StateList {
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
    position_a: usize,
    position_b: usize,
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

fn search_states_after_event(table: &[WriteSpec], state: SearchState, event: Event, target: StateCode) -> Vec<SearchState> {
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
fn some_stream_realizes_target(world: &World, target: StateCode) -> bool {
    let [chain_a, chain_b] = &world.search_chains;
    let initial = SearchState {
        position_a: 0,
        position_b: 0,
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
        if state.position_a == chain_a.len() && state.position_b == chain_b.len() {
            return true;
        }
        if state.position_a < chain_a.len() {
            for mut next in search_states_after_event(&world.table, state, chain_a[state.position_a], target) {
                next.position_a += 1;
                pending.push_back(next);
            }
        }
        if state.position_b < chain_b.len() {
            let wait = world.search_waits_of_second_chain.get(state.position_b).copied().flatten();
            if wait.map_or(true, |needed| state.position_a > needed) {
                for mut next in search_states_after_event(&world.table, state, chain_b[state.position_b], target) {
                    next.position_b += 1;
                    pending.push_back(next);
                }
            }
        }
    }
    false
}

fn truth_by_search(world: &World) -> StateList {
    all_state_vectors(&world.table)
        .into_iter()
        .filter(|target| some_stream_realizes_target(world, *target))
        .collect()
}

// ── 精确可达集 T_exact（只核参照）──────────────────────────────────────────────────────────

/// 偏序：一次写 w 落在盘 d 上，流里 w 之后、在 d 上有一道屏障，则 w 先于那道屏障之后发出的每一次写；FUA 写先于它后面的每一次写。
/// 返回每次写的「必须先持久」集合（传递闭包之后）。
fn must_persist_before_masks(table: &[WriteSpec], stream: &[Event]) -> Vec<u32> {
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
fn reachable_states_of_partial_order(table: &[WriteSpec], masks: &[u32]) -> Vec<StateCode> {
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

fn exact_reachable_states(world: &World) -> StateList {
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
fn arm_by_transaction_states(world: &World) -> StateList {
    let states_of_a = crash_states_of_stream(&world.table, &world.arm_chains[0]);
    let states_of_b = crash_states_of_stream(&world.table, &world.arm_chains[1]);
    let full_state_of_a = persisted_everything_in(&world.arm_chains[0]);
    let full_state_of_b = persisted_everything_in(&world.arm_chains[1]);
    assert!(contains(&states_of_a, full_state_of_a) && contains(&states_of_b, full_state_of_b), "链全持久的那个状态在自己的枚举域里");
    let mut product = Vec::new();
    for &state_of_a in &states_of_a {
        for &state_of_b in &states_of_b {
            product.push(state_of_a | state_of_b);
        }
    }
    if !world.arm_tail.is_empty() {
        for tail_state in crash_states_of_stream(&world.table, &world.arm_tail) {
            if tail_state != 0 {
                product.push(full_state_of_a | full_state_of_b | tail_state);
            }
        }
    }
    sorted_deduplicated(product)
}

/// 阳性对照 PC1 在按事务切那条臂上的形态：把积换成串行拼接（A 的各状态 × B 全不持久，并上 A 全持久 × B 的各状态）。
fn serial_concatenation_of_transaction_chains(world: &World) -> StateList {
    let states_of_a = crash_states_of_stream(&world.table, &world.arm_chains[0]);
    let states_of_b = crash_states_of_stream(&world.table, &world.arm_chains[1]);
    let full_state_of_a = persisted_everything_in(&world.arm_chains[0]);
    let mut states = states_of_a;
    states.extend(states_of_b.into_iter().map(|state_of_b| full_state_of_a | state_of_b));
    sorted_deduplicated(states)
}

// ── 判据用的谓词与计数 ──────────────────────────────────────────────────────────────────

fn writes_of_owner(table: &[WriteSpec], owner: Owner) -> Vec<usize> {
    (0..table.len()).filter(|&write_index| table[write_index].owner == owner).collect()
}

/// 跨事务状态 C：A、B 各至少有一次写不是「没持久」。
fn is_cross_transaction(table: &[WriteSpec], state: StateCode) -> bool {
    let some_nonzero = |owner: Owner| writes_of_owner(table, owner).into_iter().any(|write_index| value_of(state, write_index) != NOT_PERSISTED);
    some_nonzero(Owner::TransactionA) && some_nonzero(Owner::TransactionB)
}

fn records_of_owner_all_persisted(table: &[WriteSpec], state: StateCode, owner: Owner) -> bool {
    (0..table.len())
        .filter(|&write_index| table[write_index].owner == owner && table[write_index].kind == StepKind::JournalRecord)
        .all(|write_index| value_of(state, write_index) == PERSISTED)
}

/// I_rec（按记录）：B 的记录都持久而 A 的记录不都持久。
fn is_invalid_prefix_by_record(table: &[WriteSpec], state: StateCode) -> bool {
    let transaction_a_records_all_persisted = records_of_owner_all_persisted(table, state, Owner::TransactionA);
    let transaction_b_records_all_persisted = records_of_owner_all_persisted(table, state, Owner::TransactionB);
    transaction_b_records_all_persisted && !transaction_a_records_all_persisted
}

/// I_full（按全部写）：B 的每次写都持久，而 A 至少一次写不是「没持久」、又不是每次写都持久。
fn is_invalid_prefix_by_all_writes(table: &[WriteSpec], state: StateCode) -> bool {
    let values_of = |owner: Owner| -> Vec<u8> { writes_of_owner(table, owner).into_iter().map(|write_index| value_of(state, write_index)).collect() };
    let values_of_a = values_of(Owner::TransactionA);
    let values_of_b = values_of(Owner::TransactionB);
    values_of_b.iter().all(|value| *value == PERSISTED)
        && values_of_a.iter().any(|value| *value != NOT_PERSISTED)
        && values_of_a.iter().any(|value| *value != PERSISTED)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InvalidPrefixKind {
    ByRecord,
    ByAllWrites,
}

impl InvalidPrefixKind {
    fn name(self) -> &'static str {
        match self {
            InvalidPrefixKind::ByRecord => "recorded",
            InvalidPrefixKind::ByAllWrites => "all_writes",
        }
    }

    fn holds(self, table: &[WriteSpec], state: StateCode) -> bool {
        match self {
            InvalidPrefixKind::ByRecord => is_invalid_prefix_by_record(table, state),
            InvalidPrefixKind::ByAllWrites => is_invalid_prefix_by_all_writes(table, state),
        }
    }
}

const BOTH_INVALID_PREFIX_KINDS: [InvalidPrefixKind; 2] = [InvalidPrefixKind::ByRecord, InvalidPrefixKind::ByAllWrites];

/// |(T ∩ C) \ 臂|：跨事务落盘子集里这条臂漏掉的个数。
fn count_cross_transaction_misses(table: &[WriteSpec], truth: &[StateCode], covered: &[StateCode]) -> usize {
    truth.iter().filter(|state| is_cross_transaction(table, **state) && !contains(covered, **state)).count()
}

/// |(T \ C) \ 臂|：C 之外这条臂漏掉的个数。
fn count_non_cross_misses(table: &[WriteSpec], truth: &[StateCode], covered: &[StateCode]) -> usize {
    truth.iter().filter(|state| !is_cross_transaction(table, **state) && !contains(covered, **state)).count()
}

fn count_invalid_prefixes_covered(table: &[WriteSpec], truth: &[StateCode], covered: &[StateCode], kind: InvalidPrefixKind) -> usize {
    truth.iter().filter(|state| kind.holds(table, **state) && contains(covered, **state)).count()
}

fn count_invalid_prefixes_in(table: &[WriteSpec], states: &[StateCode], kind: InvalidPrefixKind) -> usize {
    states.iter().filter(|state| kind.holds(table, **state)).count()
}

fn difference_count(left: &[StateCode], right: &[StateCode]) -> usize {
    left.iter().filter(|state| !contains(right, **state)).count()
}

// ── 罩住一个状态集的最小段串（L7a）──────────────────────────────────────────────────────────

#[derive(Debug)]
struct CoverOutcome {
    segments: Vec<Vec<usize>>,
    covers_target: bool,
    cover_state_count: u128,
    equals_target: bool,
}

/// 合并法：目标集里任何一个状态里「持久或撕裂的写」必须落在「没持久或撕裂的写」所在段之前或同段，把必须同段的写合成一段，
/// 其余按这个偏序排成最细的段串；再按定义核一遍枚举域是否罩得住目标集。
fn cover_by_merging(table: &[WriteSpec], target: &[StateCode]) -> CoverOutcome {
    let write_count = table.len();
    let mut at_most = vec![vec![false; write_count]; write_count];
    for &state in target {
        for earlier in 0..write_count {
            if value_of(state, earlier) == NOT_PERSISTED {
                continue;
            }
            for later in 0..write_count {
                if later != earlier && value_of(state, later) != PERSISTED {
                    at_most[earlier][later] = true;
                }
            }
        }
    }
    for middle in 0..write_count {
        for first in 0..write_count {
            for second in 0..write_count {
                if at_most[first][middle] && at_most[middle][second] {
                    at_most[first][second] = true;
                }
            }
        }
    }
    let mut group_of: Vec<usize> = (0..write_count).collect();
    for first in 0..write_count {
        for second in (first + 1)..write_count {
            if at_most[first][second] && at_most[second][first] {
                let (kept, replaced) = (group_of[first], group_of[second]);
                for group in &mut group_of {
                    if *group == replaced {
                        *group = kept;
                    }
                }
            }
        }
    }
    let mut group_ids: Vec<usize> = group_of.clone();
    group_ids.sort_unstable();
    group_ids.dedup();
    let members_of = |group_id: usize| -> Vec<usize> { (0..write_count).filter(|&write_index| group_of[write_index] == group_id).collect() };
    let precedes = |earlier_group: usize, later_group: usize| -> bool {
        members_of(earlier_group).iter().any(|&earlier| members_of(later_group).iter().any(|&later| at_most[earlier][later])) && earlier_group != later_group
    };
    let mut ordered_groups = group_ids.clone();
    ordered_groups.sort_by_key(|group_id| (group_ids.iter().filter(|other| precedes(**other, *group_id)).count(), *group_id));
    let segments: Vec<Vec<usize>> = ordered_groups.into_iter().map(members_of).collect();
    summarize_cover(table, target, segments)
}

fn summarize_cover(table: &[WriteSpec], target: &[StateCode], segments: Vec<Vec<usize>>) -> CoverOutcome {
    let covers_target = target.iter().all(|state| state_is_in_crash_states_of_segments(&segments, *state));
    let cover_state_count = closed_form_state_count(table, &segments);
    let equals_target = covers_target && cover_state_count == target.len() as u128;
    CoverOutcome { segments, covers_target, cover_state_count, equals_target }
}

struct ExhaustiveCover {
    minimal_state_count: u128,
    optimal_covers: Vec<Vec<Vec<usize>>>,
}

/// 有序划分穷举：把写表里的写分成有序的段，取罩得住目标集、枚举域最小的那些段串。
fn cover_by_exhaustive_ordered_partitions(table: &[WriteSpec], target: &[StateCode]) -> ExhaustiveCover {
    assert!(table.len() <= 8, "有序划分穷举只给写数 ≤ 8 的几何");
    let mut best = ExhaustiveCover { minimal_state_count: u128::MAX, optimal_covers: Vec::new() };
    let mut current: Vec<Vec<usize>> = Vec::new();
    let all_writes: u32 = (1u32 << table.len()) - 1;
    extend_ordered_partitions(table, target, all_writes, &mut current, 1, &mut best);
    best
}

fn extend_ordered_partitions(table: &[WriteSpec], target: &[StateCode], remaining: u32, current: &mut Vec<Vec<usize>>, partial_count: u128, best: &mut ExhaustiveCover) {
    if partial_count > best.minimal_state_count {
        return;
    }
    if remaining == 0 {
        if target.iter().all(|state| state_is_in_crash_states_of_segments(current, *state)) {
            if partial_count < best.minimal_state_count {
                best.minimal_state_count = partial_count;
                best.optimal_covers.clear();
            }
            best.optimal_covers.push(current.clone());
        }
        return;
    }
    let mut chosen = remaining;
    while chosen != 0 {
        let segment: Vec<usize> = (0..table.len()).filter(|&write_index| chosen & (1u32 << write_index) != 0).collect();
        let expansion = segment_expansion(table, &segment);
        current.push(segment);
        extend_ordered_partitions(table, target, remaining & !chosen, current, partial_count + expansion - 1, best);
        current.pop();
        chosen = (chosen - 1) & remaining;
    }
}

// ── 发布归属（S2：memory_pool.rs:993 `publishes_of_one_recording` 的规则）────────────────────────

#[derive(Debug, PartialEq, Eq)]
struct PublishAttribution {
    units: Vec<usize>,
    records: Vec<usize>,
    root: usize,
}

/// 一次根槽写之前、上一次根槽写之后的单元写与记录写都归它；系统配置槽写与屏障不归任何一次发布。
fn publish_attribution(table: &[WriteSpec], stream: &[Event]) -> Vec<PublishAttribution> {
    let mut publishes = Vec::new();
    let mut units = Vec::new();
    let mut records = Vec::new();
    for event in stream {
        if let Event::Write(write_index) = event {
            match table[*write_index].kind {
                StepKind::UnitWrite => units.push(*write_index),
                StepKind::JournalRecord => records.push(*write_index),
                StepKind::RootRecordFua => publishes.push(PublishAttribution {
                    units: std::mem::take(&mut units),
                    records: std::mem::take(&mut records),
                    root: *write_index,
                }),
                StepKind::SystemConfigurationSlot | StepKind::Barrier => {}
            }
        }
    }
    publishes
}

// ── 文字渲染 ───────────────────────────────────────────────────────────────────────────

fn render_stream(table: &[WriteSpec], stream: &[Event]) -> String {
    let disk_count = table.iter().map(|spec| spec.disk).max().map_or(1, |highest| highest + 1);
    stream
        .iter()
        .map(|event| match event {
            Event::Write(write_index) => table[*write_index].name.clone(),
            Event::Barrier(disk) => {
                if disk_count == 1 {
                    "|".to_string()
                } else {
                    format!("|{disk}")
                }
            }
        })
        .collect::<Vec<String>>()
        .join(".")
}

fn render_state(table: &[WriteSpec], state: StateCode) -> String {
    let parts: Vec<String> = (0..table.len())
        .filter(|&write_index| value_of(state, write_index) != NOT_PERSISTED)
        .map(|write_index| {
            if value_of(state, write_index) == TORN {
                format!("{}~", table[write_index].name)
            } else {
                table[write_index].name.clone()
            }
        })
        .collect();
    if parts.is_empty() {
        "none".to_string()
    } else {
        parts.join("+")
    }
}

fn segment_sizes_text(table: &[WriteSpec], stream: &[Event]) -> String {
    write_segments(table, stream).iter().map(|segment| segment.len().to_string()).collect::<Vec<String>>().join("+")
}

fn segment_kinds_text(table: &[WriteSpec], stream: &[Event]) -> String {
    operation_segments(table, stream)
        .iter()
        .map(|segment| {
            let mut counts: std::collections::BTreeMap<StepKind, usize> = std::collections::BTreeMap::new();
            for event in segment {
                let kind = match event {
                    Event::Write(write_index) => table[*write_index].kind,
                    Event::Barrier(_) => StepKind::Barrier,
                };
                *counts.entry(kind).or_insert(0) += 1;
            }
            let parts: Vec<String> = counts
                .into_iter()
                .map(|(kind, count)| if count == 1 { kind.registered_name().to_string() } else { format!("{}×{count}", kind.registered_name()) })
                .collect();
            format!("[{}]", parts.join(","))
        })
        .collect::<Vec<String>>()
        .join("|")
}

/// 按登记表的「种类」串造出事件：每段先写后屏障；写按盘轮转（根槽 FUA 落盘 0），屏障 ×N 落在盘 0 到 N − 1。
fn stream_from_registered_kinds(text: &str, system_configuration_has_third_state: bool) -> (Vec<WriteSpec>, Vec<Event>) {
    let geometry = Geometry {
        label: "registered_stream",
        units_per_transaction: 0,
        disk_count: 2,
        root_disk_of_transaction_a: 0,
        root_disk_of_transaction_b: 0,
        system_configuration_has_third_state,
    };
    let mut table = Vec::new();
    let mut events = Vec::new();
    for segment_text in text.split('|') {
        let inner = segment_text.trim_start_matches('[').trim_end_matches(']');
        let mut barriers = Vec::new();
        for item in inner.split(',') {
            let (name, count) = match item.split_once('×') {
                Some((name, count_text)) => (name, count_text.parse::<usize>().expect("种类串里的个数")),
                None => (item, 1),
            };
            let kind = StepKind::from_registered_name(name);
            for ordinal in 0..count {
                match kind {
                    StepKind::Barrier => barriers.push(Event::Barrier(ordinal)),
                    StepKind::RootRecordFua => {
                        let write_index = push_write(&mut table, &geometry, "root", 0, kind, Owner::Shared);
                        events.push(Event::Write(write_index));
                    }
                    StepKind::UnitWrite | StepKind::JournalRecord | StepKind::SystemConfigurationSlot => {
                        let write_index = push_write(&mut table, &geometry, name, ordinal % 2, kind, Owner::Shared);
                        events.push(Event::Write(write_index));
                    }
                }
            }
        }
        events.extend(barriers);
    }
    (table, events)
}

// ── 单元格的全部读数 ────────────────────────────────────────────────────────────────────

struct StreamMetrics {
    state_count: usize,
    closed_form_state_count: u128,
    cross_misses: usize,
    non_cross_misses: usize,
    recorded_prefixes_covered: usize,
    all_writes_prefixes_covered: usize,
    largest_segment_writes: usize,
    largest_segment_expansion: u128,
}

struct CellSpec {
    label: &'static str,
    sampling_point: &'static str,
    geometry: Geometry,
    candidate: Candidate,
}

fn cell_specs() -> Vec<CellSpec> {
    // G1 与 G2 是同一个几何（一盘、u = 2、系统配置槽写三态）上 P3 取 Σ_free 与 Σ_ord；G3 一盘 u = 1；G5 是 G1 而系统配置槽写只取两态。
    let groups: [(Geometry, [&'static str; 4], &'static str, &'static str); 3] = [
        (geometry_g1(), ["g1_serial", "g1_one_publish", "g1_free", "g1_ordered"], "G1", "G2"),
        (geometry_g3(), ["g3_serial", "g3_one_publish", "g3_free", "g3_ordered"], "G3", "G3_ordered"),
        (geometry_g5(), ["g5_serial", "g5_one_publish", "g5_free", "g5_ordered"], "G5", "G5_ordered"),
    ];
    let mut specs = Vec::new();
    for (geometry, labels, free_point, ordered_point) in groups {
        let candidates_and_points = [
            (Candidate::SerialPipeline, free_point),
            (Candidate::OnePublishManyTransactions, free_point),
            (Candidate::ManyPublishesInFlightFree, free_point),
            (Candidate::ManyPublishesInFlightOrdered, ordered_point),
        ];
        for (label, (candidate, sampling_point)) in labels.into_iter().zip(candidates_and_points) {
            specs.push(CellSpec { label, sampling_point, geometry: geometry.clone(), candidate });
        }
    }
    specs
}

struct CellEvaluation {
    label: &'static str,
    sampling_point: &'static str,
    world: World,
    truth: StateList,
    truth_by_search: StateList,
    streams: Vec<StreamMetrics>,
    serial_stream_position: Option<usize>,
    arm_by_transaction: StateList,
    exact_reachable: StateList,
}

fn evaluate_cell(spec: &CellSpec) -> CellEvaluation {
    evaluate_cell_with(spec, &truth_by_search)
}

/// 算法 (b) 由调用方给：第一段用逐状态的集合版，第二段的两盘格用定长数组版（`truth_by_search_fast`，单测与集合版逐格对拍）。
fn evaluate_cell_with(spec: &CellSpec, search: &dyn Fn(&World) -> StateList) -> CellEvaluation {
    let world = build_world(spec.candidate, &spec.geometry);
    let truth = truth_by_enumeration(&world);
    let truth_by_search = search(&world);
    let streams: Vec<StreamMetrics> = world
        .streams
        .iter()
        .map(|stream| {
            let segments = write_segments(&world.table, stream);
            let states = crash_states_of_segments(&world.table, &segments);
            StreamMetrics {
                state_count: states.len(),
                closed_form_state_count: closed_form_state_count(&world.table, &segments),
                cross_misses: count_cross_transaction_misses(&world.table, &truth, &states),
                non_cross_misses: count_non_cross_misses(&world.table, &truth, &states),
                recorded_prefixes_covered: count_invalid_prefixes_covered(&world.table, &truth, &states, InvalidPrefixKind::ByRecord),
                all_writes_prefixes_covered: count_invalid_prefixes_covered(&world.table, &truth, &states, InvalidPrefixKind::ByAllWrites),
                largest_segment_writes: segments.iter().map(Vec::len).max().unwrap_or(0),
                largest_segment_expansion: segments.iter().map(|segment| segment_expansion(&world.table, segment)).max().unwrap_or(1),
            }
        })
        .collect();
    let serial_stream_position = world.serial_stream.as_ref().and_then(|serial| world.streams.iter().position(|stream| stream == serial));
    let arm_by_transaction = arm_by_transaction_states(&world);
    let exact_reachable = exact_reachable_states(&world);
    CellEvaluation { label: spec.label, sampling_point: spec.sampling_point, world, truth, truth_by_search, streams, serial_stream_position, arm_by_transaction, exact_reachable }
}

fn minimum_with_position(values: &[usize]) -> (usize, usize) {
    let mut best = (values[0], 0usize);
    for (position, value) in values.iter().enumerate() {
        if *value < best.0 {
            best = (*value, position);
        }
    }
    best
}

fn maximum_value(values: &[usize]) -> usize {
    values.iter().copied().max().expect("至少一条发出流")
}

// ── 判据的判定函数（判别力自证要挪它的门槛）──────────────────────────────────────────────────

/// 漏数判定：漏数 > 门槛才算漏；每一条发出流上都漏、只在某些发出流上漏、一条都不漏。
fn leak_judgement(minimum_misses: usize, maximum_misses: usize, threshold: usize) -> &'static str {
    if minimum_misses > threshold {
        "leaks_on_every_stream"
    } else if maximum_misses > threshold {
        "leaks_on_some_streams"
    } else {
        "never_leaks"
    }
}

/// 不合法前缀覆盖判定：覆盖数 < 要求的个数记「漏不合法前缀」。
fn invalid_prefix_judgement(covered: usize, required: usize) -> &'static str {
    if required == 0 {
        "no_invalid_prefix_in_truth"
    } else if covered < required {
        "misses_invalid_prefixes"
    } else {
        "covers_every_invalid_prefix"
    }
}

// ── 输出 ───────────────────────────────────────────────────────────────────────────────

fn emit(emitter: &mut Emitter, body: String) {
    println!("{}", emitter.emit_raw(&body));
}

fn yes_no(condition: bool) -> &'static str {
    if condition {
        "yes"
    } else {
        "no"
    }
}

#[derive(Default)]
struct RunTally {
    void_conditions: Vec<String>,
    halt_conditions: Vec<String>,
    self_checks_passed: usize,
}

impl RunTally {
    fn self_check(&mut self, passed: bool, void_label: &str) {
        if passed {
            self.self_checks_passed += 1;
        } else {
            self.void_conditions.push(void_label.to_string());
        }
    }
}

// ── 停机条款 S1、S2（登记第十一节）──────────────────────────────────────────────────────────

/// `.claude/kb/layout/01-first-txn.md` 第 403 行「普通发布（新池新建文件）」那一行的种类串（整行抄）。
const REGISTERED_ORDINARY_PUBLISH_KINDS: &str =
    "[unit_write×24,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]";
/// 同一文件第 410 行「新池新建文件那条流」的种类串（整条流，57 次操作）。
const REGISTERED_WHOLE_STREAM_KINDS: &str = "[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[unit_write×24,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]";

struct RegisteredReading {
    item: &'static str,
    segments: String,
    kinds: String,
    operations: usize,
    closed_form_states: u128,
    enumerated_states: usize,
}

fn registered_ordinary_publish_reading() -> RegisteredReading {
    let geometry = geometry_registered_publish_two_state();
    let mut table = Vec::new();
    let writes = push_transaction_writes(&mut table, &geometry, Owner::TransactionA, 0);
    let chain = single_transaction_chain(&writes, geometry.disk_count);
    let segments = write_segments(&table, &chain);
    RegisteredReading {
        item: "ordinary_publish_chain_u12_two_disks",
        segments: segment_sizes_text(&table, &chain),
        kinds: segment_kinds_text(&table, &chain),
        operations: chain.len(),
        closed_form_states: closed_form_state_count(&table, &segments),
        enumerated_states: crash_states_of_segments(&table, &segments).len(),
    }
}

fn registered_whole_stream_reading(system_configuration_has_third_state: bool, enumerate: bool) -> RegisteredReading {
    let (table, events) = stream_from_registered_kinds(REGISTERED_WHOLE_STREAM_KINDS, system_configuration_has_third_state);
    let segments = write_segments(&table, &events);
    RegisteredReading {
        item: if system_configuration_has_third_state { "whole_stream_three_state" } else { "whole_stream_two_state" },
        segments: segment_sizes_text(&table, &events),
        kinds: segment_kinds_text(&table, &events),
        operations: events.len(),
        closed_form_states: closed_form_state_count(&table, &segments),
        enumerated_states: if enumerate { crash_states_of_segments(&table, &segments).len() } else { 0 },
    }
}

const REGISTERED_ORDINARY_PUBLISH_SEGMENTS: &str = "24+2+1+2";
const REGISTERED_WHOLE_STREAM_SEGMENTS: &str = "2+2+1+2+2+1+2+24+2+1+2";

fn run_halt_checks(emitter: &mut Emitter, tally: &mut RunTally) {
    let ordinary = registered_ordinary_publish_reading();
    let ordinary_ok = ordinary.segments == REGISTERED_ORDINARY_PUBLISH_SEGMENTS
        && ordinary.kinds == REGISTERED_ORDINARY_PUBLISH_KINDS
        && ordinary.operations == 35
        && ordinary.closed_form_states == 16_777_223
        && ordinary.enumerated_states == 16_777_223;
    emit(
        emitter,
        format!(
            "name=halt_check condition=S1 item={} segments={} registered_segments={} kinds_match={} operations={} registered_operations=35 closed_form_states={} enumerated_states={} registered_states=16777223 ok={ordinary_ok}",
            ordinary.item,
            ordinary.segments,
            REGISTERED_ORDINARY_PUBLISH_SEGMENTS,
            ordinary.kinds == REGISTERED_ORDINARY_PUBLISH_KINDS,
            ordinary.operations,
            ordinary.closed_form_states,
            ordinary.enumerated_states,
        ),
    );
    if !ordinary_ok {
        tally.halt_conditions.push(format!("S1:{}:kinds={}", ordinary.item, ordinary.kinds));
    }
    let stream_two_state = registered_whole_stream_reading(false, true);
    let stream_ok = stream_two_state.segments == REGISTERED_WHOLE_STREAM_SEGMENTS
        && stream_two_state.kinds == REGISTERED_WHOLE_STREAM_KINDS
        && stream_two_state.operations == 57
        && stream_two_state.closed_form_states == 16_777_240
        && stream_two_state.enumerated_states == 16_777_240;
    emit(
        emitter,
        format!(
            "name=halt_check condition=S1 item={} segments={} registered_segments={} kinds_match={} operations={} registered_operations=57 closed_form_states={} enumerated_states={} registered_states=16777240 ok={stream_ok}",
            stream_two_state.item,
            stream_two_state.segments,
            REGISTERED_WHOLE_STREAM_SEGMENTS,
            stream_two_state.kinds == REGISTERED_WHOLE_STREAM_KINDS,
            stream_two_state.operations,
            stream_two_state.closed_form_states,
            stream_two_state.enumerated_states,
        ),
    );
    if !stream_ok {
        tally.halt_conditions.push(format!("S1:{}:kinds={}", stream_two_state.item, stream_two_state.kinds));
    }
    let stream_three_state = registered_whole_stream_reading(true, false);
    let three_state_ok = stream_three_state.closed_form_states == 16_777_260;
    emit(
        emitter,
        format!(
            "name=halt_check condition=S1 item={} closed_form_states={} registered_states=16777260 ok={three_state_ok}",
            stream_three_state.item, stream_three_state.closed_form_states
        ),
    );
    if !three_state_ok {
        tally.halt_conditions.push("S1:whole_stream_three_state".to_string());
    }
    let attribution_ok = attribution_matches_hand_derivation();
    emit(
        emitter,
        format!(
            "name=halt_check condition=S2 item=publish_attribution_on_serial_stream expected_publishes=2 hand_derived_units_per_publish=2 hand_derived_records_per_publish=1 ok={attribution_ok}"
        ),
    );
    if !attribution_ok {
        tally.halt_conditions.push("S2:publish_attribution".to_string());
    }
}

/// S2：串行流 a1 a2 | rA | RA SA | b1 b2 | rB | RB SB 上，按 `memory_pool.rs:993` `publishes_of_one_recording` 的规则手推：
/// 第一次发布 = 单元写 a1、a2 + 记录写 rA + 根 RA；第二次 = b1、b2、rB、RB；两个系统配置槽写（SA、SB）与屏障不归任何一次发布。
fn attribution_matches_hand_derivation() -> bool {
    let world = build_world(Candidate::SerialPipeline, &geometry_g1());
    let index_of = |name: &str| world.table.iter().position(|spec| spec.name == name).expect("写表里有这个名字");
    let hand_derived = vec![
        PublishAttribution { units: vec![index_of("a1"), index_of("a2")], records: vec![index_of("rA")], root: index_of("RA") },
        PublishAttribution { units: vec![index_of("b1"), index_of("b2")], records: vec![index_of("rB")], root: index_of("RB") },
    ];
    let serial = world.serial_stream.as_ref().expect("串行候选有串行流");
    publish_attribution(&world.table, serial) == hand_derived
}

// ── 丙类锚点（登记第七节）──────────────────────────────────────────────────────────────────

struct AnchorRow {
    geometry: Geometry,
    expected_single_chain: u128,
    expected_transaction_product: u128,
    expected_serial_truth: u128,
    expected_one_publish_truth: u128,
    expected_one_publish_arm_by_transaction: u128,
}

fn anchor_rows() -> Vec<AnchorRow> {
    let row = |geometry: Geometry, values: [u128; 5]| AnchorRow {
        geometry,
        expected_single_chain: values[0],
        expected_transaction_product: values[1],
        expected_serial_truth: values[2],
        expected_one_publish_truth: values[3],
        expected_one_publish_arm_by_transaction: values[4],
    };
    vec![
        row(geometry_g1(), [8, 64, 15, 22, 28]),
        row(geometry_g3(), [6, 36, 11, 10, 12]),
        row(geometry_g4a(), [28, 784, 55, 280, 370]),
        row(geometry_g4b(), [28, 784, 55, 280, 370]),
        row(geometry_g5(), [7, 49, 13, 21, 27]),
    ]
}

struct AnchorReading {
    quantity: &'static str,
    expected: u128,
    actual: u128,
}

fn read_anchor_row(row: &AnchorRow) -> Vec<AnchorReading> {
    let serial = build_world(Candidate::SerialPipeline, &row.geometry);
    let one_publish = build_world(Candidate::OnePublishManyTransactions, &row.geometry);
    let single_chain = crash_states_of_stream(&serial.table, &serial.arm_chains[0]).len() as u128;
    let product = arm_by_transaction_states(&serial).len() as u128;
    let serial_truth = truth_by_enumeration(&serial).len() as u128;
    let one_publish_truth = truth_by_enumeration(&one_publish).len() as u128;
    let one_publish_arm = arm_by_transaction_states(&one_publish).len() as u128;
    vec![
        AnchorReading { quantity: "single_transaction_chain", expected: row.expected_single_chain, actual: single_chain },
        AnchorReading { quantity: "arm_by_transaction_product", expected: row.expected_transaction_product, actual: product },
        AnchorReading { quantity: "serial_pipeline_truth", expected: row.expected_serial_truth, actual: serial_truth },
        AnchorReading { quantity: "one_publish_truth", expected: row.expected_one_publish_truth, actual: one_publish_truth },
        AnchorReading { quantity: "one_publish_arm_by_transaction", expected: row.expected_one_publish_arm_by_transaction, actual: one_publish_arm },
    ]
}

fn run_anchor_table(emitter: &mut Emitter, tally: &mut RunTally) {
    for row in anchor_rows() {
        for reading in read_anchor_row(&row) {
            let ok = reading.expected == reading.actual;
            emit(
                emitter,
                format!("name=anchor table=closed_form geometry={} quantity={} expected={} actual={} ok={ok}", row.geometry.label, reading.quantity, reading.expected, reading.actual),
            );
            tally.self_check(ok, &format!("V5:anchor:{}:{}", row.geometry.label, reading.quantity));
        }
    }
    run_capacity_anchors(emitter, tally);
}

/// 登记第七节末尾「能力那几格的锚点」：罩住段串与真实发布形上的算术。
fn run_capacity_anchors(emitter: &mut Emitter, tally: &mut RunTally) {
    for (geometry, expected_cover_states, expected_cover_writes) in [(geometry_g1(), 2304u128, 10usize), (geometry_g4a(), 1_327_104u128, 18usize)] {
        let world = build_world(Candidate::SerialPipeline, &geometry);
        let product = arm_by_transaction_states(&world);
        let cover = cover_by_merging(&world.table, &product);
        let one_segment_of_everything = cover.segments.len() == 1 && cover.segments[0].len() == expected_cover_writes;
        let cover_largest_expansion = cover.segments.iter().map(|segment| segment_expansion(&world.table, segment)).max().unwrap_or(0);
        let ok = one_segment_of_everything && cover.covers_target && cover_largest_expansion == expected_cover_states;
        emit(
            emitter,
            format!(
                "name=anchor table=capacity geometry={} quantity=arm_by_transaction_cover_one_segment_states expected={expected_cover_states} actual={cover_largest_expansion} cover_segments={} cover_largest_segment_writes={} covers_target={} within_u32={} ok={ok}",
                geometry.label,
                cover.segments.len(),
                cover.segments.iter().map(Vec::len).max().unwrap_or(0),
                cover.covers_target,
                cover_largest_expansion <= MAXIMUM_SEGMENT_STATES_IN_U32,
            ),
        );
        tally.self_check(ok, &format!("F6:L7a_anchor:{}", geometry.label));
    }
    let real = geometry_real_publish_shape();
    let world = build_world(Candidate::SerialPipeline, &real);
    let every_write: Vec<usize> = (0..world.table.len()).collect();
    let cover_all_writes_states = segment_expansion(&world.table, &every_write);
    let chain_segments = write_segments(&world.table, &world.arm_chains[0]);
    let single_chain_states = closed_form_state_count(&world.table, &chain_segments);
    let product_states = single_chain_states * single_chain_states;
    let witness_stream: Vec<Event> = {
        let units_of_a: Vec<Event> = world.arm_chains[0].iter().copied().take_while(|event| matches!(event, Event::Write(index) if world.table[*index].kind == StepKind::UnitWrite)).collect();
        let units_of_b: Vec<Event> = world.arm_chains[1].iter().copied().take_while(|event| matches!(event, Event::Write(index) if world.table[*index].kind == StepKind::UnitWrite)).collect();
        let mut stream = units_of_a.clone();
        stream.extend(units_of_b.iter().copied());
        stream.extend(world.arm_chains[0][units_of_a.len()..].iter().copied());
        stream.extend(world.arm_chains[1][units_of_b.len()..].iter().copied());
        stream
    };
    let witness_segments = write_segments(&world.table, &witness_stream);
    let witness_first_segment_writes = witness_segments[0].len();
    let witness_first_segment_states = segment_expansion(&world.table, &witness_segments[0]) - 1;
    let readings: [(&str, u128, u128); 4] = [
        ("real_shape_arm_by_transaction_cover_one_segment_states", 1_459_166_279_268_040_704, cover_all_writes_states),
        ("real_shape_single_chain_states_three_state", 16_777_228, single_chain_states),
        ("real_shape_arm_by_transaction_product_states", 281_475_379_363_984, product_states),
        ("real_shape_k1_witness_first_segment_nonzero_states", 281_474_976_710_655, witness_first_segment_states),
    ];
    for (quantity, expected, actual) in readings {
        let ok = expected == actual;
        emit(
            emitter,
            format!(
                "name=anchor table=capacity geometry={} quantity={quantity} expected={expected} actual={actual} exceeds_u32={} within_u64={} ok={ok}",
                real.label,
                actual > MAXIMUM_SEGMENT_STATES_IN_U32,
                actual <= MAXIMUM_SEGMENT_STATES_IN_U64,
            ),
        );
        tally.self_check(ok, &format!("V5:capacity_anchor:{quantity}"));
    }
    let witness_ok = witness_first_segment_writes == 2 * real.units_per_transaction * real.disk_count;
    emit(
        emitter,
        format!(
            "name=anchor table=capacity geometry={} quantity=real_shape_k1_witness_first_segment_writes expected={} actual={witness_first_segment_writes} within_enumerated_writes_limit={} ok={witness_ok}",
            real.label,
            2 * real.units_per_transaction * real.disk_count,
            world.table.len() <= MAXIMUM_ENUMERATED_WRITES,
        ),
    );
    tally.self_check(witness_ok, "F6:witness_first_segment_writes");
}

// ── 单元格的输出与判决 ────────────────────────────────────────────────────────────────────

struct CellJudgement {
    label: &'static str,
    sampling_point: &'static str,
    candidate: Candidate,
    disks: usize,
    arm_by_root_publish_leak_judgement: &'static str,
    arm_by_root_publish_minimum_cross_misses: usize,
    arm_by_root_publish_maximum_cross_misses: usize,
    arm_by_transaction_cross_misses: usize,
    recorded_truth_count: usize,
    all_writes_truth_count: usize,
}

fn emit_cell(emitter: &mut Emitter, tally: &mut RunTally, evaluation: &CellEvaluation) -> CellJudgement {
    let world = &evaluation.world;
    let table = &world.table;
    let label = evaluation.label;
    let truth = &evaluation.truth;
    let methods_agree = *truth == evaluation.truth_by_search;
    tally.self_check(methods_agree, &format!("V3:truth_methods:{label}"));
    emit(
        emitter,
        format!(
            "name=cell_truth cell={label} sampling_point={} candidate={} disks={} units={} table_writes={} streams={} truth_by_enumeration={} truth_by_search={} methods_agree={methods_agree} cross_transaction_states={}",
            evaluation.sampling_point,
            world.candidate.name(),
            world.geometry.disk_count,
            world.geometry.units_per_transaction,
            table.len(),
            world.streams.len(),
            truth.len(),
            evaluation.truth_by_search.len(),
            truth.iter().filter(|state| is_cross_transaction(table, **state)).count(),
        ),
    );
    let cross_misses: Vec<usize> = evaluation.streams.iter().map(|metrics| metrics.cross_misses).collect();
    let non_cross_misses: Vec<usize> = evaluation.streams.iter().map(|metrics| metrics.non_cross_misses).collect();
    let state_counts: Vec<usize> = evaluation.streams.iter().map(|metrics| metrics.state_count).collect();
    let (minimum_cross_misses, minimum_position) = minimum_with_position(&cross_misses);
    let maximum_cross_misses = maximum_value(&cross_misses);
    let (minimum_states, _) = minimum_with_position(&state_counts);
    let maximum_states = maximum_value(&state_counts);
    let closed_form_mismatches = evaluation.streams.iter().filter(|metrics| metrics.state_count as u128 != metrics.closed_form_state_count).count();
    tally.self_check(closed_form_mismatches == 0, &format!("V6:closed_form:{label}"));
    let serial_states = evaluation.serial_stream_position.map_or_else(|| "not_defined".to_string(), |position| evaluation.streams[position].state_count.to_string());
    emit(
        emitter,
        format!(
            "name=arm_by_root_publish cell={label} streams={} states_minimum={minimum_states} states_maximum={maximum_states} states_on_serial_stream={serial_states} states_on_minimum_cross_miss_stream={} closed_form_mismatches={closed_form_mismatches} cross_misses_minimum={minimum_cross_misses} cross_misses_maximum={maximum_cross_misses} streams_with_cross_misses={} minimum_cross_miss_stream={} non_cross_misses_maximum={} streams_with_non_cross_misses={}",
            evaluation.streams.len(),
            state_counts[minimum_position],
            cross_misses.iter().filter(|misses| **misses > 0).count(),
            render_stream(table, &world.streams[minimum_position]),
            maximum_value(&non_cross_misses),
            non_cross_misses.iter().filter(|misses| **misses > 0).count(),
        ),
    );
    let arm = &evaluation.arm_by_transaction;
    let cross_misses_of_arm = count_cross_transaction_misses(table, truth, arm);
    let true_cross_misses_of_arm: usize = {
        let reachable_truth: Vec<StateCode> = truth.iter().copied().filter(|state| contains(&evaluation.exact_reachable, *state)).collect();
        count_cross_transaction_misses(table, &reachable_truth, arm)
    };
    let extra_states = difference_count(arm, truth);
    let identity_ok = arm.len() + cross_misses_of_arm + count_non_cross_misses(table, truth, arm) == truth.len() + extra_states;
    tally.self_check(identity_ok, &format!("V5:identity:{label}"));
    emit(
        emitter,
        format!(
            "name=arm_by_transaction cell={label} states={} cross_misses={cross_misses_of_arm} true_cross_misses={true_cross_misses_of_arm} non_cross_misses={} states_not_in_truth={extra_states} identity_ok={identity_ok}",
            arm.len(),
            count_non_cross_misses(table, truth, arm),
        ),
    );
    let one_disk = world.geometry.disk_count == 1;
    if one_disk {
        tally.self_check(cross_misses_of_arm == 0, &format!("V4:one_disk_arm_by_transaction_misses:{label}"));
    }
    let mut recorded_truth_count = 0;
    let mut all_writes_truth_count = 0;
    for kind in BOTH_INVALID_PREFIX_KINDS {
        let truth_count = count_invalid_prefixes_in(table, truth, kind);
        let covered: Vec<usize> = evaluation
            .streams
            .iter()
            .map(|metrics| match kind {
                InvalidPrefixKind::ByRecord => metrics.recorded_prefixes_covered,
                InvalidPrefixKind::ByAllWrites => metrics.all_writes_prefixes_covered,
            })
            .collect();
        let arm_covered = count_invalid_prefixes_covered(table, truth, arm, kind);
        let arm_unreachable = count_invalid_prefixes_in(table, arm, kind) - arm_covered;
        match kind {
            InvalidPrefixKind::ByRecord => recorded_truth_count = truth_count,
            InvalidPrefixKind::ByAllWrites => all_writes_truth_count = truth_count,
        }
        emit(
            emitter,
            format!(
                "name=invalid_prefix cell={label} kind={} truth_count={truth_count} arm_by_root_publish_covered_minimum={} arm_by_root_publish_covered_maximum={} arm_by_transaction_covered={arm_covered} arm_by_transaction_not_in_truth={arm_unreachable} arm_by_root_publish_judgement={} arm_by_transaction_judgement={}",
                kind.name(),
                covered.iter().copied().min().expect("至少一条流"),
                maximum_value(&covered),
                invalid_prefix_judgement(covered.iter().copied().min().expect("至少一条流"), truth_count),
                invalid_prefix_judgement(arm_covered, truth_count),
            ),
        );
    }
    emit_cover_lines(emitter, tally, evaluation);
    emit_capacity_line(emitter, evaluation);
    let exact = &evaluation.exact_reachable;
    emit(
        emitter,
        format!(
            "name=reference cell={label} exact_states={} exact_not_in_truth={} truth_not_in_exact={}",
            exact.len(),
            difference_count(exact, truth),
            difference_count(truth, exact),
        ),
    );
    CellJudgement {
        label,
        sampling_point: evaluation.sampling_point,
        candidate: world.candidate,
        disks: world.geometry.disk_count,
        arm_by_root_publish_leak_judgement: leak_judgement(minimum_cross_misses, maximum_cross_misses, 0),
        arm_by_root_publish_minimum_cross_misses: minimum_cross_misses,
        arm_by_root_publish_maximum_cross_misses: maximum_cross_misses,
        arm_by_transaction_cross_misses: cross_misses_of_arm,
        recorded_truth_count,
        all_writes_truth_count,
    }
}

fn emit_cover_lines(emitter: &mut Emitter, tally: &mut RunTally, evaluation: &CellEvaluation) {
    let table = &evaluation.world.table;
    for (set_name, target) in [("truth", &evaluation.truth), ("arm_by_transaction", &evaluation.arm_by_transaction)] {
        let cover = cover_by_merging(table, target);
        let exhaustive_agrees = if table.len() <= 8 {
            let exhaustive = cover_by_exhaustive_ordered_partitions(table, target);
            let agrees = exhaustive.minimal_state_count == cover.cover_state_count && exhaustive.optimal_covers.contains(&cover.segments);
            tally.self_check(agrees, &format!("L7a:merging_vs_exhaustive:{}:{set_name}", evaluation.label));
            yes_no(agrees).to_string()
        } else {
            "not_run_more_than_8_writes".to_string()
        };
        emit(
            emitter,
            format!(
                "name=cover cell={} set={set_name} target_states={} cover_segments={} cover_largest_segment_writes={} cover_states={} covers_target={} cover_equals_target={} exhaustive_agrees={exhaustive_agrees}",
                evaluation.label,
                target.len(),
                cover.segments.len(),
                cover.segments.iter().map(Vec::len).max().unwrap_or(0),
                cover.cover_state_count,
                cover.covers_target,
                cover.equals_target,
            ),
        );
    }
}

fn emit_capacity_line(emitter: &mut Emitter, evaluation: &CellEvaluation) {
    let table = &evaluation.world.table;
    let largest_writes = evaluation.streams.iter().map(|metrics| metrics.largest_segment_writes).max().unwrap_or(0);
    let largest_expansion = evaluation.streams.iter().map(|metrics| metrics.largest_segment_expansion).max().unwrap_or(0);
    let total_states = evaluation.streams.iter().map(|metrics| metrics.state_count).max().unwrap_or(0);
    let cover = cover_by_merging(table, &evaluation.arm_by_transaction);
    let cover_largest_expansion = cover.segments.iter().map(|segment| segment_expansion(table, segment)).max().unwrap_or(0);
    emit(
        emitter,
        format!(
            "name=capacity cell={} table_writes={} arm_by_root_publish_largest_segment_writes={largest_writes} arm_by_root_publish_largest_segment_states={largest_expansion} arm_by_root_publish_total_states_maximum={total_states} arm_by_transaction_cover_largest_segment_states={cover_largest_expansion} arm_by_transaction_cover_total_states={} writes_within_limit={} segments_within_u32={} segments_within_u64={}",
            evaluation.label,
            table.len(),
            cover.cover_state_count,
            table.len() <= MAXIMUM_ENUMERATED_WRITES,
            largest_expansion <= MAXIMUM_SEGMENT_STATES_IN_U32 && cover_largest_expansion <= MAXIMUM_SEGMENT_STATES_IN_U32,
            largest_expansion <= MAXIMUM_SEGMENT_STATES_IN_U64 && cover_largest_expansion <= MAXIMUM_SEGMENT_STATES_IN_U64,
        ),
    );
}

// ── 阳性对照 ───────────────────────────────────────────────────────────────────────────

fn index_of_write(table: &[WriteSpec], name: &str) -> usize {
    table.iter().position(|spec| spec.name == name).unwrap_or_else(|| panic!("写表里没有 {name}"))
}

fn state_with_only(table: &[WriteSpec], names: &[&str]) -> StateCode {
    names.iter().fold(0, |code, name| with_value(code, index_of_write(table, name), PERSISTED))
}

fn without_state(states: &[StateCode], removed: StateCode) -> StateList {
    states.iter().copied().filter(|state| *state != removed).collect()
}

/// PC3 要的见证状态：两个事务的单元写都持久、rB 持久、rA 不持久（登记第五节 PC3 的例子）。
fn recorded_prefix_witness(table: &[WriteSpec]) -> StateCode {
    let mut code = 0;
    for (write_index, spec) in table.iter().enumerate() {
        let is_unit = spec.kind == StepKind::UnitWrite;
        let is_record_of_b = spec.kind == StepKind::JournalRecord && spec.owner == Owner::TransactionB;
        if is_unit || is_record_of_b {
            code = with_value(code, write_index, PERSISTED);
        }
    }
    code
}

fn run_positive_controls(emitter: &mut Emitter, tally: &mut RunTally, evaluations: &[CellEvaluation]) {
    for evaluation in evaluations {
        let world = &evaluation.world;
        let table = &world.table;
        let truth = &evaluation.truth;
        let label = evaluation.label;
        if world.candidate == Candidate::ManyPublishesInFlightFree {
            let witness = state_with_only(table, &["b1"]);
            let serial = world.serial_stream.as_ref().expect("两条链的候选有串行流");
            let serial_states = crash_states_of_stream(table, serial);
            let serial_product = serial_concatenation_of_transaction_chains(world);
            for (arm_name, arm_states) in [("arm_by_root_publish", &serial_states), ("arm_by_transaction", &serial_product)] {
                let witness_in_truth = contains(truth, witness);
                let witness_missed = !contains(arm_states, witness);
                let misses = difference_count(truth, arm_states);
                let cross_misses = count_cross_transaction_misses(table, truth, arm_states);
                let ok = witness_in_truth && witness_missed && misses >= 1;
                emit(
                    emitter,
                    format!(
                        "name=positive_control control=serial_concatenation cell={label} arm={arm_name} witness={} witness_in_truth={witness_in_truth} witness_missed={witness_missed} misses_against_truth={misses} cross_misses={cross_misses} ok={ok}",
                        render_state(table, witness),
                    ),
                );
                tally.self_check(ok, &format!("V2:PC1:{label}:{arm_name}"));
            }
        }
        if world.candidate == Candidate::SerialPipeline {
            continue;
        }
        let two_units_state = state_with_only(table, &["a1", "b1"]);
        if contains(truth, two_units_state) {
            let stream_of_witness = world.streams.iter().find(|stream| contains(&crash_states_of_stream(table, stream), two_units_state)).expect("w2 在真值里就有一条流罩得住它");
            let arm_states = crash_states_of_stream(table, stream_of_witness);
            let before = count_cross_transaction_misses(table, truth, &arm_states);
            let after = count_cross_transaction_misses(table, truth, &without_state(&arm_states, two_units_state));
            let k2_before = count_cross_transaction_misses(table, truth, &evaluation.arm_by_transaction);
            let k2_after = count_cross_transaction_misses(table, truth, &without_state(&evaluation.arm_by_transaction, two_units_state));
            let ok = after == before + 1 && k2_after == k2_before + 1;
            emit(
                emitter,
                format!(
                    "name=positive_control control=delete_one_state cell={label} deleted={} arm_by_root_publish_misses_before={before} arm_by_root_publish_misses_after={after} arm_by_transaction_misses_before={k2_before} arm_by_transaction_misses_after={k2_after} ok={ok}",
                    render_state(table, two_units_state),
                ),
            );
            tally.self_check(ok, &format!("V2:PC2:{label}"));
        }
        let prefix_witness = recorded_prefix_witness(table);
        if contains(truth, prefix_witness) && is_invalid_prefix_by_record(table, prefix_witness) {
            let stream_of_witness = world.streams.iter().find(|stream| contains(&crash_states_of_stream(table, stream), prefix_witness)).expect("见证状态在真值里就有一条流罩得住它");
            let arm_states = crash_states_of_stream(table, stream_of_witness);
            let before = count_invalid_prefixes_covered(table, truth, &arm_states, InvalidPrefixKind::ByRecord);
            let after = count_invalid_prefixes_covered(table, truth, &without_state(&arm_states, prefix_witness), InvalidPrefixKind::ByRecord);
            let k2_before = count_invalid_prefixes_covered(table, truth, &evaluation.arm_by_transaction, InvalidPrefixKind::ByRecord);
            let k2_after = count_invalid_prefixes_covered(table, truth, &without_state(&evaluation.arm_by_transaction, prefix_witness), InvalidPrefixKind::ByRecord);
            let ok = before == after + 1 && k2_before == k2_after + 1;
            emit(
                emitter,
                format!(
                    "name=positive_control control=delete_one_invalid_prefix cell={label} deleted={} arm_by_root_publish_covered_before={before} arm_by_root_publish_covered_after={after} arm_by_transaction_covered_before={k2_before} arm_by_transaction_covered_after={k2_after} ok={ok}",
                    render_state(table, prefix_witness),
                ),
            );
            tally.self_check(ok, &format!("V2:PC3:{label}"));
        } else {
            emit(
                emitter,
                format!("name=positive_control control=delete_one_invalid_prefix cell={label} status=not_applicable reason=witness_not_in_truth"),
            );
        }
    }
}

// ── 判别力自证与几何敏感性 ───────────────────────────────────────────────────────────────

fn run_threshold_shifts(emitter: &mut Emitter, tally: &mut RunTally, evaluations: &[CellEvaluation], judgements: &[CellJudgement]) {
    let free = judgements.iter().find(|judgement| judgement.label == "g1_free").expect("G1 的 Σ_free 格");
    let before = leak_judgement(free.arm_by_root_publish_minimum_cross_misses, free.arm_by_root_publish_maximum_cross_misses, 0);
    let after = leak_judgement(free.arm_by_root_publish_minimum_cross_misses, free.arm_by_root_publish_maximum_cross_misses, free.arm_by_root_publish_maximum_cross_misses);
    let flips = before != after && after == "never_leaks";
    emit(
        emitter,
        format!(
            "name=threshold_shift quantity=l2_cross_misses cell=g1_free threshold_before=0 judgement_before={before} threshold_after={} judgement_after={after} flips_as_required={flips}",
            free.arm_by_root_publish_maximum_cross_misses,
        ),
    );
    tally.self_check(flips, "threshold_shift:l2");
    let evaluation = evaluations.iter().find(|candidate| candidate.label == "g1_free").expect("G1 的 Σ_free 格");
    let required = count_invalid_prefixes_in(&evaluation.world.table, &evaluation.truth, InvalidPrefixKind::ByRecord);
    let best_stream_covered = evaluation.streams.iter().map(|metrics| metrics.recorded_prefixes_covered).max().expect("至少一条流");
    let before = invalid_prefix_judgement(best_stream_covered, required);
    let after = invalid_prefix_judgement(best_stream_covered, best_stream_covered);
    let flips = before == "misses_invalid_prefixes" && after == "covers_every_invalid_prefix";
    emit(
        emitter,
        format!(
            "name=threshold_shift quantity=l6_recorded_prefixes_best_stream cell=g1_free covered={best_stream_covered} required_before={required} judgement_before={before} required_after={best_stream_covered} judgement_after={after} flips_as_required={flips}"
        ),
    );
    tally.self_check(flips, "threshold_shift:l6");
    emit(
        emitter,
        "name=threshold_shift quantity=l3b_true_cross_misses status=not_applicable reason=one_disk_cells_have_zero_by_derivation_and_l3b_is_judged_on_two_disk_geometries_in_the_second_segment".to_string(),
    );
}

fn run_geometry_sensitivity(emitter: &mut Emitter, judgements: &[CellJudgement]) {
    for (free_label, ordered_label, geometry_name) in [
        ("g1_free", "g1_ordered", "G1_vs_G2"),
        ("g3_free", "g3_ordered", "G3_vs_G3_ordered"),
        ("g5_free", "g5_ordered", "G5_vs_G5_ordered"),
    ] {
        let free = judgements.iter().find(|judgement| judgement.label == free_label).expect("自由交错格");
        let ordered = judgements.iter().find(|judgement| judgement.label == ordered_label).expect("带约束格");
        let same = free.arm_by_root_publish_leak_judgement == ordered.arm_by_root_publish_leak_judgement;
        emit(
            emitter,
            format!(
                "name=geometry_sensitivity pair={geometry_name} l2_judgement_free={} l2_judgement_ordered={} same_judgement={} cross_misses_minimum_free={} cross_misses_minimum_ordered={} cross_misses_maximum_free={} cross_misses_maximum_ordered={} invalid_prefix_recorded_truth_free={} invalid_prefix_recorded_truth_ordered={} invalid_prefix_all_writes_truth_free={} invalid_prefix_all_writes_truth_ordered={}",
                free.arm_by_root_publish_leak_judgement,
                ordered.arm_by_root_publish_leak_judgement,
                yes_no(same),
                free.arm_by_root_publish_minimum_cross_misses,
                ordered.arm_by_root_publish_minimum_cross_misses,
                free.arm_by_root_publish_maximum_cross_misses,
                ordered.arm_by_root_publish_maximum_cross_misses,
                free.recorded_truth_count,
                ordered.recorded_truth_count,
                free.all_writes_truth_count,
                ordered.all_writes_truth_count,
            ),
        );
    }
    for (three_state_label, two_state_label, pair_name) in [("g1_free", "g5_free", "G1_vs_G5_free"), ("g1_ordered", "g5_ordered", "G2_vs_G5_ordered")] {
        let three_state = judgements.iter().find(|judgement| judgement.label == three_state_label).expect("三态格");
        let two_state = judgements.iter().find(|judgement| judgement.label == two_state_label).expect("两态格");
        let same = three_state.arm_by_root_publish_leak_judgement == two_state.arm_by_root_publish_leak_judgement
            && three_state.arm_by_transaction_cross_misses == two_state.arm_by_transaction_cross_misses
            && invalid_prefix_judgement(0, three_state.recorded_truth_count) == invalid_prefix_judgement(0, two_state.recorded_truth_count);
        emit(
            emitter,
            format!(
                "name=geometry_sensitivity pair={pair_name} system_configuration_third_state_changes_judgement={} l2_judgement_three_state={} l2_judgement_two_state={} cross_misses_minimum_three_state={} cross_misses_minimum_two_state={} cross_misses_maximum_three_state={} cross_misses_maximum_two_state={} recorded_truth_three_state={} recorded_truth_two_state={}",
                yes_no(!same),
                three_state.arm_by_root_publish_leak_judgement,
                two_state.arm_by_root_publish_leak_judgement,
                three_state.arm_by_root_publish_minimum_cross_misses,
                two_state.arm_by_root_publish_minimum_cross_misses,
                three_state.arm_by_root_publish_maximum_cross_misses,
                two_state.arm_by_root_publish_maximum_cross_misses,
                three_state.recorded_truth_count,
                two_state.recorded_truth_count,
            ),
        );
    }
}

fn run_failure_clauses(emitter: &mut Emitter, evaluations: &[CellEvaluation], judgements: &[CellJudgement]) {
    let reference_triggered = evaluations.iter().any(|evaluation| difference_count(&evaluation.exact_reachable, &evaluation.truth) > 0);
    emit(emitter, format!("name=failure_clause clause=F1 triggered={} observation=exact_not_in_truth_over_all_cells_or_closed_form_mismatch", yes_no(reference_triggered)));
    let both_lose = judgements.iter().any(|judgement| {
        judgement.candidate == Candidate::ManyPublishesInFlightFree && judgement.disks == 1 && judgement.arm_by_root_publish_leak_judgement == "leaks_on_every_stream" && judgement.arm_by_transaction_cross_misses > 0
    });
    emit(emitter, format!("name=failure_clause clause=F2 triggered={} observation=l2_minimum_positive_and_l3b_positive_on_the_same_geometry_one_disk_only", yes_no(both_lose)));
    for (free_label, ordered_label) in [("g1_free", "g1_ordered"), ("g3_free", "g3_ordered"), ("g5_free", "g5_ordered")] {
        let free = judgements.iter().find(|judgement| judgement.label == free_label).expect("自由交错格");
        let ordered = judgements.iter().find(|judgement| judgement.label == ordered_label).expect("带约束格");
        emit(
            emitter,
            format!(
                "name=failure_clause clause=F3 pair={free_label}_vs_{ordered_label} triggered={}",
                yes_no(free.arm_by_root_publish_leak_judgement != ordered.arm_by_root_publish_leak_judgement)
            ),
        );
    }
    for judgement in judgements.iter().filter(|judgement| judgement.candidate == Candidate::ManyPublishesInFlightFree) {
        emit(
            emitter,
            format!(
                "name=failure_clause clause=F4 cell={} recorded_truth_count={} all_writes_truth_count={} triggered_recorded={} triggered_all_writes={}",
                judgement.label,
                judgement.recorded_truth_count,
                judgement.all_writes_truth_count,
                yes_no(judgement.recorded_truth_count == 0),
                yes_no(judgement.all_writes_truth_count == 0),
            ),
        );
    }
    emit(emitter, "name=failure_clause clause=F5 status=not_applicable reason=judged_on_two_disk_geometries_in_the_second_segment".to_string());
}

// ── 主流程 ─────────────────────────────────────────────────────────────────────────────

fn run_everything(emitter: &mut Emitter) -> RunTally {
    let mut tally = RunTally::default();
    emit(
        emitter,
        "name=config model=independent_count_model candidates=4 sampling_points=G1,G2,G3,G3_ordered,G5,G5_ordered anchor_geometries=G1,G3,G4a,G4b,G5 segment=first not_run=L8,L9,G4a_cells,G4b_cells".to_string(),
    );
    run_halt_checks(emitter, &mut tally);
    if !tally.halt_conditions.is_empty() {
        return tally;
    }
    run_anchor_table(emitter, &mut tally);
    let evaluations: Vec<CellEvaluation> = cell_specs().iter().map(evaluate_cell).collect();
    let mut judgements = Vec::new();
    for evaluation in &evaluations {
        judgements.push(emit_cell(emitter, &mut tally, evaluation));
    }
    for evaluation in &evaluations {
        if matches!(evaluation.world.candidate, Candidate::SerialPipeline | Candidate::OnePublishManyTransactions) {
            let miss_maximum = evaluation.streams.iter().map(|metrics| metrics.cross_misses + metrics.non_cross_misses).max().unwrap_or(0);
            let extra_states = evaluation.streams.iter().map(|metrics| metrics.state_count).max().unwrap_or(0) as isize - evaluation.truth.len() as isize;
            tally.self_check(miss_maximum == 0 && extra_states == 0, &format!("V1:real_baseline:{}", evaluation.label));
        }
    }
    run_positive_controls(emitter, &mut tally, &evaluations);
    run_threshold_shifts(emitter, &mut tally, &evaluations, &judgements);
    run_geometry_sensitivity(emitter, &judgements);
    run_failure_clauses(emitter, &evaluations, &judgements);
    for judgement in &judgements {
        emit(
            emitter,
            format!(
                "name=verdict cell={} sampling_point={} arm_by_root_publish_l2_judgement={} arm_by_root_publish_cross_misses_minimum={} arm_by_root_publish_cross_misses_maximum={} arm_by_transaction_cross_misses={}",
                judgement.label,
                judgement.sampling_point,
                judgement.arm_by_root_publish_leak_judgement,
                judgement.arm_by_root_publish_minimum_cross_misses,
                judgement.arm_by_root_publish_maximum_cross_misses,
                judgement.arm_by_transaction_cross_misses,
            ),
        );
    }
    tally
}

fn repository_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().expect("仓根在 scripts/experiments 往上两层")
}

fn preflight() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let root = repository_root();
    let source = root.join("scripts/experiments/e167_multi_inflight_crash_cut.rs");
    let script = root.join(".claude/singlefs-ai-sop/scripts/preflight.py");
    let mut command = std::process::Command::new("python3");
    command.arg(&script).arg("check").arg(&source);
    if arguments.iter().any(|argument| argument == "--force") {
        command.arg("--force");
    }
    command.arg("--");
    command.args(arguments.iter().filter(|argument| *argument != "--force"));
    let output = command.output().unwrap_or_else(|error| {
        eprintln!("  ✗ 起不了 python3 判准入：{error}");
        eprintln!("  → 怎么办：装上 python3，或在有 python3 的机器上跑");
        std::process::exit(78)
    });
    if !output.status.success() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        print!("{}", String::from_utf8_lossy(&output.stdout));
        std::process::exit(output.status.code().unwrap_or(1));
    }
}

fn main() {
    preflight();
    let mut emitter = Emitter::new();
    let tally = match std::env::args().nth(1).as_deref() {
        Some("halt") => {
            let mut tally = RunTally::default();
            run_halt_checks(&mut emitter, &mut tally);
            tally
        }
        Some("s3") => {
            run_s3_estimate(&mut emitter);
            RunTally::default()
        }
        Some("second") => run_second_segment(&mut emitter),
        Some("third") => run_third_segment(&mut emitter),
        _ => run_everything(&mut emitter),
    };
    emit(
        &mut emitter,
        format!(
            "name=verdict scope=run self_checks_passed={} void_conditions={} halt_conditions={} void_list={} halt_list={}",
            tally.self_checks_passed,
            tally.void_conditions.len(),
            tally.halt_conditions.len(),
            if tally.void_conditions.is_empty() { "none".to_string() } else { tally.void_conditions.join(",") },
            if tally.halt_conditions.is_empty() { "none".to_string() } else { tally.halt_conditions.join(",") },
        ),
    );
    println!("{}", emitter.finish());
    if !tally.halt_conditions.is_empty() {
        std::process::exit(3);
    }
    if !tally.void_conditions.is_empty() {
        std::process::exit(1);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════════════
// 第二段：两盘几何（G4a、G4b）上的量
// ═══════════════════════════════════════════════════════════════════════════════════════════════
//
// 第一段把 Σ_free 的每一条发出流存在内存里逐条算（G1 共 12870 条）。两盘几何的 Σ_free 有 C(30, 15) = 155117520 条，存不下，
// 所以第二段不存流：深搜一边走一边记「每次写落在第几段」（段签名；同一个签名的流枚举域相同，只算一次、带上条数）与
// 「每次写发出时已被屏障或 FUA 钉住的写」（偏序的直接前驱，T_exact 用）。与第一段的对照：
// 单测 `streaming_evaluation_equals_the_in_memory_evaluation_on_the_one_disk_cells` 在 G1、G3、G5 的六个自由与带约束格上逐项对拍。

use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

/// 线程数只影响挂钟、不影响任何一行输出（并行单位之间的合并是求和、取最小、取并）。
fn worker_thread_count() -> usize {
    std::env::var("SINGLEFS_THREAD_CAP").ok().and_then(|text| text.parse::<usize>().ok()).filter(|count| *count >= 1).unwrap_or(4)
}

#[derive(Default)]
struct MixingHasher {
    state: u64,
}

impl MixingHasher {
    fn mix(&mut self, word: u64) {
        self.state = (self.state.rotate_left(5) ^ word).wrapping_mul(0x517c_c1b7_2722_0a95);
    }
}

impl Hasher for MixingHasher {
    fn finish(&self) -> u64 {
        let mut value = self.state;
        value ^= value >> 33;
        value = value.wrapping_mul(0xff51_afd7_ed55_8ccd);
        value ^= value >> 33;
        value
    }

    fn write(&mut self, bytes: &[u8]) {
        for chunk in bytes.chunks(8) {
            let mut word = [0u8; 8];
            word[..chunk.len()].copy_from_slice(chunk);
            self.mix(u64::from_le_bytes(word));
        }
    }

    fn write_u128(&mut self, value: u128) {
        self.mix(value as u64);
        self.mix((value >> 64) as u64);
    }
}

type FastMap<K, V> = HashMap<K, V, BuildHasherDefault<MixingHasher>>;
type FastSet<K> = std::collections::HashSet<K, BuildHasherDefault<MixingHasher>>;

struct Bitset {
    words: Vec<u64>,
}

impl Bitset {
    fn new(bits: u64) -> Bitset {
        Bitset { words: vec![0; usize::try_from(bits.div_ceil(64)).expect("位集大小装得进 usize")] }
    }

    fn set(&mut self, index: u64) {
        self.words[(index >> 6) as usize] |= 1u64 << (index & 63);
    }

    fn has(&self, index: u64) -> bool {
        self.words[(index >> 6) as usize] & (1u64 << (index & 63)) != 0
    }

    fn or_with(&mut self, other: &Bitset) {
        for (word, other_word) in self.words.iter_mut().zip(&other.words) {
            *word |= *other_word;
        }
    }

    fn indices(&self) -> Vec<u64> {
        let mut found = Vec::new();
        for (word_position, word) in self.words.iter().enumerate() {
            let mut remaining = *word;
            while remaining != 0 {
                found.push(word_position as u64 * 64 + u64::from(remaining.trailing_zeros()));
                remaining &= remaining - 1;
            }
        }
        found
    }
}

const MAXIMUM_STREAMING_WRITES: usize = 20;

/// 写表的定长摘要：每次写的取值个数（两态 2、原地覆写 3）、混合进制的权、各谓词用的位掩码（每次写两位，0 / 1 / 2）。
struct TableInfo {
    write_count: usize,
    disk_count: usize,
    radix: Vec<u8>,
    weight: Vec<u64>,
    total_vectors: u64,
    writes_on_disk: Vec<u32>,
    owner_a_any: u64,
    owner_a_ones: u64,
    owner_b_any: u64,
    owner_b_ones: u64,
    record_a_any: u64,
    record_a_ones: u64,
    record_b_any: u64,
    record_b_ones: u64,
}

impl TableInfo {
    fn new(table: &[WriteSpec]) -> TableInfo {
        let write_count = table.len();
        assert!(write_count <= MAXIMUM_STREAMING_WRITES, "流式路径的写表最多 20 次写");
        let radix: Vec<u8> = table.iter().map(|spec| if spec.is_in_place { 3 } else { 2 }).collect();
        let mut weight = Vec::new();
        let mut running = 1u64;
        for &write_radix in &radix {
            weight.push(running);
            running *= u64::from(write_radix);
        }
        let disk_count = table.iter().map(|spec| spec.disk).max().map_or(1, |highest| highest + 1);
        assert!(disk_count <= 2, "搜索状态里的未放行盘集合只留两位");
        let mut writes_on_disk = vec![0u32; disk_count];
        for (write_index, spec) in table.iter().enumerate() {
            writes_on_disk[spec.disk] |= 1u32 << write_index;
        }
        let masks_of = |select: &dyn Fn(&WriteSpec) -> bool| -> (u64, u64) {
            let mut any = 0u64;
            let mut ones = 0u64;
            for (write_index, spec) in table.iter().enumerate() {
                if select(spec) {
                    any |= 0b11u64 << (2 * write_index);
                    ones |= 0b01u64 << (2 * write_index);
                }
            }
            (any, ones)
        };
        let (owner_a_any, owner_a_ones) = masks_of(&|spec| spec.owner == Owner::TransactionA);
        let (owner_b_any, owner_b_ones) = masks_of(&|spec| spec.owner == Owner::TransactionB);
        let (record_a_any, record_a_ones) = masks_of(&|spec| spec.owner == Owner::TransactionA && spec.kind == StepKind::JournalRecord);
        let (record_b_any, record_b_ones) = masks_of(&|spec| spec.owner == Owner::TransactionB && spec.kind == StepKind::JournalRecord);
        TableInfo {
            write_count,
            disk_count,
            radix,
            weight,
            total_vectors: running,
            writes_on_disk,
            owner_a_any,
            owner_a_ones,
            owner_b_any,
            owner_b_ones,
            record_a_any,
            record_a_ones,
            record_b_any,
            record_b_ones,
        }
    }

    /// 混合进制序号 → 每次写两位的编码。
    fn code_of_index(&self, mut index: u64) -> u64 {
        let mut code = 0u64;
        for write in 0..self.write_count {
            let write_radix = u64::from(self.radix[write]);
            code |= (index % write_radix) << (2 * write);
            index /= write_radix;
        }
        code
    }

    /// 每次写两位的编码 → 混合进制序号（`code_of_index` 的反向）。
    fn index_of_code(&self, code: u64) -> u64 {
        (0..self.write_count).map(|write| ((code >> (2 * write)) & 0b11) * self.weight[write]).sum()
    }

    fn is_cross(&self, code: u64) -> bool {
        code & self.owner_a_any != 0 && code & self.owner_b_any != 0
    }

    fn is_invalid_by_record(&self, code: u64) -> bool {
        code & self.record_b_any == self.record_b_ones && code & self.record_a_any != self.record_a_ones
    }

    fn is_invalid_by_all_writes(&self, code: u64) -> bool {
        code & self.owner_b_any == self.owner_b_ones && code & self.owner_a_any != 0 && code & self.owner_a_any != self.owner_a_ones
    }
}

#[derive(Clone, Copy, Debug)]
enum Step {
    Write { index: u8, disk: u8, fua: bool },
    Barrier { disk: u8 },
}

fn steps_of(table: &[WriteSpec], chain: &[Event]) -> Vec<Step> {
    chain
        .iter()
        .map(|event| match *event {
            Event::Write(write_index) => Step::Write {
                index: u8::try_from(write_index).expect("写下标装得进 u8"),
                disk: u8::try_from(table[write_index].disk).expect("盘号装得进 u8"),
                fua: table[write_index].kind == StepKind::RootRecordFua,
            },
            Event::Barrier(disk) => Step::Barrier { disk: u8::try_from(disk).expect("盘号装得进 u8") },
        })
        .collect()
}

/// 两条链与第二条链的等待表（走过 A 的第几步之后 B 的这一步才可发出）。
struct WalkContext {
    steps_a: Vec<Step>,
    steps_b: Vec<Step>,
    waits_of_b: Vec<Option<usize>>,
    writes_on_disk: Vec<u32>,
}

impl WalkContext {
    fn new(world: &World, info: &TableInfo) -> WalkContext {
        let steps_a = steps_of(&world.table, &world.search_chains[0]);
        let steps_b = steps_of(&world.table, &world.search_chains[1]);
        assert!(steps_a.len() <= 31 && steps_b.len() <= 31, "搜索状态里每条链的位置只留五位");
        let waits_of_b = if steps_b.is_empty() { Vec::new() } else { world.search_waits_of_second_chain.clone() };
        WalkContext { steps_a, steps_b, waits_of_b, writes_on_disk: info.writes_on_disk.clone() }
    }

    fn b_step_may_follow(&self, position_a: usize, position_b: usize) -> bool {
        self.waits_of_b[position_b].map_or(true, |needed| position_a > needed)
    }
}

/// 逐条数 Σ(P)：在（A 走到哪、B 走到哪）的格子上数路径，B 的等待表限制能走哪条边。
fn count_interleavings(context: &WalkContext) -> u128 {
    let (length_a, length_b) = (context.steps_a.len(), context.steps_b.len());
    let mut ways = vec![vec![0u128; length_b + 1]; length_a + 1];
    ways[0][0] = 1;
    for position_a in 0..=length_a {
        for position_b in 0..=length_b {
            let here = ways[position_a][position_b];
            if here == 0 {
                continue;
            }
            if position_a < length_a {
                ways[position_a + 1][position_b] += here;
            }
            if position_b < length_b && context.b_step_may_follow(position_a, position_b) {
                ways[position_a][position_b + 1] += here;
            }
        }
    }
    ways[length_a][length_b]
}

/// 深搜走到一半的状态。`key` 每次写占五位：它发出时已关了几段（段号）；`path` 每步一位（0 = A、1 = B），高位在前。
#[derive(Clone, Copy)]
struct Walk {
    position_a: u8,
    position_b: u8,
    unreleased: u8,
    writes_in_current: u8,
    closed: u8,
    issued: u32,
    armed: u32,
    forced: u32,
    key: u128,
    path: u64,
}

impl Walk {
    fn start() -> Walk {
        Walk { position_a: 0, position_b: 0, unreleased: 0, writes_in_current: 0, closed: 0, issued: 0, armed: 0, forced: 0, key: 0, path: 0 }
    }
}

/// 走一步：段怎么关与 `operation_segments` 同一条规矩（屏障或 FUA、当前段有写、每块有写的盘都放行）；
/// 直接前驱怎么记与 `must_persist_before_masks` 同一条规矩（被自己盘上屏障钉住的写、FUA 写，都先于之后发出的每一次写）。
fn apply_step(context: &WalkContext, walk: &mut Walk, step: Step, masks: &mut [u32; MAXIMUM_STREAMING_WRITES]) {
    let mut may_close = false;
    match step {
        Step::Write { index, disk, fua } => {
            let bit = 1u32 << index;
            masks[usize::from(index)] = walk.armed | walk.forced;
            walk.key |= u128::from(walk.closed) << (5 * u32::from(index));
            walk.writes_in_current += 1;
            walk.issued |= bit;
            if fua {
                walk.unreleased &= !(1u8 << disk);
                walk.forced |= bit;
                may_close = true;
            } else {
                walk.unreleased |= 1u8 << disk;
            }
        }
        Step::Barrier { disk } => {
            walk.unreleased &= !(1u8 << disk);
            walk.armed |= walk.issued & context.writes_on_disk[usize::from(disk)];
            may_close = true;
        }
    }
    if may_close && walk.writes_in_current > 0 && walk.unreleased == 0 {
        walk.closed += 1;
        walk.writes_in_current = 0;
    }
}

trait LeafSink {
    fn leaf(&mut self, walk: &Walk, masks: &[u32; MAXIMUM_STREAMING_WRITES]);
}

fn walk_interleavings<S: LeafSink>(context: &WalkContext, walk: Walk, masks: &mut [u32; MAXIMUM_STREAMING_WRITES], sink: &mut S) {
    let (position_a, position_b) = (usize::from(walk.position_a), usize::from(walk.position_b));
    if position_a == context.steps_a.len() && position_b == context.steps_b.len() {
        sink.leaf(&walk, masks);
        return;
    }
    if position_a < context.steps_a.len() {
        let mut next = walk;
        apply_step(context, &mut next, context.steps_a[position_a], masks);
        next.position_a += 1;
        next.path <<= 1;
        walk_interleavings(context, next, masks, sink);
    }
    if position_b < context.steps_b.len() && context.b_step_may_follow(position_a, position_b) {
        let mut next = walk;
        apply_step(context, &mut next, context.steps_b[position_b], masks);
        next.position_b += 1;
        next.path = (next.path << 1) | 1;
        walk_interleavings(context, next, masks, sink);
    }
}

#[derive(Clone, Copy)]
struct Task {
    walk: Walk,
    masks: [u32; MAXIMUM_STREAMING_WRITES],
}

/// 把深搜的前 `steps_deep` 步展开成任务，线程按任务领活；任务按字典序排，结果与线程数无关。
fn split_tasks(context: &WalkContext, steps_deep: usize) -> Vec<Task> {
    let mut frontier = vec![Task { walk: Walk::start(), masks: [0; MAXIMUM_STREAMING_WRITES] }];
    for _ in 0..steps_deep {
        let mut next_frontier = Vec::new();
        for task in &frontier {
            let (position_a, position_b) = (usize::from(task.walk.position_a), usize::from(task.walk.position_b));
            if position_a == context.steps_a.len() && position_b == context.steps_b.len() {
                next_frontier.push(*task);
                continue;
            }
            if position_a < context.steps_a.len() {
                let mut child = *task;
                apply_step(context, &mut child.walk, context.steps_a[position_a], &mut child.masks);
                child.walk.position_a += 1;
                child.walk.path <<= 1;
                next_frontier.push(child);
            }
            if position_b < context.steps_b.len() && context.b_step_may_follow(position_a, position_b) {
                let mut child = *task;
                apply_step(context, &mut child.walk, context.steps_b[position_b], &mut child.masks);
                child.walk.position_b += 1;
                child.walk.path = (child.walk.path << 1) | 1;
                next_frontier.push(child);
            }
        }
        frontier = next_frontier;
    }
    frontier
}

#[derive(Default)]
struct PassOneSink {
    leaves: u64,
    signatures: FastMap<u128, (u64, u64)>,
    masks: FastSet<[u32; MAXIMUM_STREAMING_WRITES]>,
}

impl LeafSink for PassOneSink {
    fn leaf(&mut self, walk: &Walk, masks: &[u32; MAXIMUM_STREAMING_WRITES]) {
        self.leaves += 1;
        let entry = self.signatures.entry(walk.key).or_insert((0, u64::MAX));
        entry.0 += 1;
        if walk.path < entry.1 {
            entry.1 = walk.path;
        }
        self.masks.insert(*masks);
    }
}

struct PassOne {
    leaves: u64,
    /// （段签名，这个签名的流条数，这个签名里字典序最小的那条流的路径）
    signatures: Vec<(u128, u64, u64)>,
    masks: Vec<[u32; MAXIMUM_STREAMING_WRITES]>,
}

fn pass_one(context: &WalkContext, threads: usize) -> PassOne {
    let tasks = split_tasks(context, 12usize.min((context.steps_a.len() + context.steps_b.len()) / 2));
    let next_task = AtomicUsize::new(0);
    let mut sinks: Vec<PassOneSink> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|_| {
                scope.spawn(|| {
                    let mut sink = PassOneSink::default();
                    loop {
                        let position = next_task.fetch_add(1, AtomicOrdering::Relaxed);
                        if position >= tasks.len() {
                            break;
                        }
                        let mut masks = tasks[position].masks;
                        walk_interleavings(context, tasks[position].walk, &mut masks, &mut sink);
                    }
                    sink
                })
            })
            .collect();
        handles.into_iter().map(|handle| handle.join().expect("线程不 panic")).collect()
    });
    let mut merged = sinks.pop().expect("至少一个线程");
    for other in sinks {
        merged.leaves += other.leaves;
        for (key, (count, representative)) in other.signatures {
            let entry = merged.signatures.entry(key).or_insert((0, u64::MAX));
            entry.0 += count;
            entry.1 = entry.1.min(representative);
        }
        for masks in other.masks {
            merged.masks.insert(masks);
        }
    }
    let mut signatures: Vec<(u128, u64, u64)> = merged.signatures.into_iter().map(|(key, (count, representative))| (key, count, representative)).collect();
    signatures.sort_unstable();
    let mut masks: Vec<[u32; MAXIMUM_STREAMING_WRITES]> = merged.masks.into_iter().collect();
    masks.sort_unstable();
    PassOne { leaves: merged.leaves, signatures, masks }
}

/// 段签名 → 各段的写集合（按段号排）。
fn segments_of_signature(key: u128, write_count: usize) -> Vec<u32> {
    let mut by_index = [0u32; 32];
    let mut highest = 0usize;
    for write in 0..write_count {
        let segment = ((key >> (5 * write)) & 31) as usize;
        by_index[segment] |= 1u32 << write;
        highest = highest.max(segment);
    }
    by_index[..=highest].to_vec()
}

/// 一串段的枚举域：每个状态连同它的混合进制序号一起交给 `emit`。「前面各段全持久 + 当前段任意组合」，
/// 当前段的全零组合不发（它就是上一段全持久那个状态），全局的全零状态发一次；个数 = 1 + Σ(2^两态写数 · 3^原地覆写数 − 1)。
#[inline]
fn for_each_crash_state(info: &TableInfo, segments: &[u32], mut emit: impl FnMut(u64, u64)) {
    emit(0, 0);
    let mut prefix_code = 0u64;
    let mut prefix_index = 0u64;
    let mut writes = [0usize; MAXIMUM_STREAMING_WRITES];
    for &segment in segments {
        let mut count = 0usize;
        let mut remaining = segment;
        while remaining != 0 {
            writes[count] = remaining.trailing_zeros() as usize;
            remaining &= remaining - 1;
            count += 1;
        }
        let mut digits = [0u8; MAXIMUM_STREAMING_WRITES];
        let mut code = prefix_code;
        let mut index = prefix_index;
        'combinations: loop {
            let mut position = 0usize;
            loop {
                if position == count {
                    break 'combinations;
                }
                let write = writes[position];
                if digits[position] + 1 < info.radix[write] {
                    digits[position] += 1;
                    code += 1u64 << (2 * write);
                    index += info.weight[write];
                    break;
                }
                code -= u64::from(digits[position]) << (2 * write);
                index -= u64::from(digits[position]) * info.weight[write];
                digits[position] = 0;
                position += 1;
            }
            emit(code, index);
        }
        for &write in &writes[..count] {
            prefix_code |= 1u64 << (2 * write);
            prefix_index += info.weight[write];
        }
    }
}

fn expansion_of_segment(info: &TableInfo, segment: u32) -> u128 {
    (0..info.write_count).filter(|write| segment & (1u32 << write) != 0).map(|write| u128::from(info.radix[write])).product()
}

fn closed_form_of_segments(info: &TableInfo, segments: &[u32]) -> u128 {
    1 + segments.iter().map(|segment| expansion_of_segment(info, *segment) - 1).sum::<u128>()
}

/// 一个段签名上的读数。K1 在这个签名对应的每一条流上的读数都相同；`count` 是这个签名有几条流。
struct SignatureRow {
    key: u128,
    count: u64,
    representative_path: u64,
    states: u32,
    cross_states: u32,
    invalid_record_states: u32,
    invalid_all_writes_states: u32,
    closed_form_matches: bool,
    largest_segment_writes: u8,
    largest_segment_expansion: u32,
}

fn evaluate_signatures(info: &TableInfo, signatures: &[(u128, u64, u64)], threads: usize) -> (Bitset, Vec<SignatureRow>) {
    const CHUNK: usize = 64;
    let next_start = AtomicUsize::new(0);
    let parts: Vec<(Bitset, Vec<SignatureRow>)> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|_| {
                scope.spawn(|| {
                    let mut union = Bitset::new(info.total_vectors);
                    let mut rows = Vec::new();
                    loop {
                        let start = next_start.fetch_add(CHUNK, AtomicOrdering::Relaxed);
                        if start >= signatures.len() {
                            break;
                        }
                        for &(key, count, representative_path) in &signatures[start..(start + CHUNK).min(signatures.len())] {
                            let segments = segments_of_signature(key, info.write_count);
                            let (mut states, mut cross, mut invalid_record, mut invalid_all) = (0u32, 0u32, 0u32, 0u32);
                            for_each_crash_state(info, &segments, |code, index| {
                                union.set(index);
                                states += 1;
                                if info.is_cross(code) {
                                    cross += 1;
                                }
                                if info.is_invalid_by_record(code) {
                                    invalid_record += 1;
                                }
                                if info.is_invalid_by_all_writes(code) {
                                    invalid_all += 1;
                                }
                            });
                            rows.push(SignatureRow {
                                key,
                                count,
                                representative_path,
                                states,
                                cross_states: cross,
                                invalid_record_states: invalid_record,
                                invalid_all_writes_states: invalid_all,
                                closed_form_matches: u128::from(states) == closed_form_of_segments(info, &segments),
                                largest_segment_writes: segments.iter().map(|segment| segment.count_ones()).max().unwrap_or(0) as u8,
                                largest_segment_expansion: u32::try_from(segments.iter().map(|segment| expansion_of_segment(info, *segment)).max().unwrap_or(1))
                                    .expect("两盘几何一段的展开数装得进 u32"),
                            });
                        }
                    }
                    (union, rows)
                })
            })
            .collect();
        handles.into_iter().map(|handle| handle.join().expect("线程不 panic")).collect()
    });
    let mut union = Bitset::new(info.total_vectors);
    let mut rows = Vec::new();
    for (part_union, mut part_rows) in parts {
        union.or_with(&part_union);
        rows.append(&mut part_rows);
    }
    rows.sort_unstable_by_key(|row| row.key);
    (union, rows)
}

fn truth_list_of(info: &TableInfo, bits: &Bitset) -> StateList {
    let mut states: StateList = bits.indices().into_iter().map(|index| StateCode::from(info.code_of_index(index))).collect();
    states.sort_unstable();
    states
}

// ── T_exact 的流式算法 ────────────────────────────────────────────────────────────────────

/// 与 `must_persist_before_masks` 末尾同一个传递闭包循环。
fn close_masks(direct: &[u32; MAXIMUM_STREAMING_WRITES], write_count: usize) -> [u32; MAXIMUM_STREAMING_WRITES] {
    let mut masks = *direct;
    loop {
        let mut changed = false;
        for write in 0..write_count {
            let mut extended = masks[write];
            for earlier in 0..write_count {
                if masks[write] & (1u32 << earlier) != 0 {
                    extended |= masks[earlier];
                }
            }
            if extended != masks[write] {
                masks[write] = extended;
                changed = true;
            }
        }
        if !changed {
            return masks;
        }
    }
}

struct ExactWalk<'a> {
    info: &'a TableInfo,
    masks: [u32; MAXIMUM_STREAMING_WRITES],
    order: Vec<usize>,
    successors: Vec<u32>,
    in_place: Vec<bool>,
}

impl ExactWalk<'_> {
    fn extend(&self, position: usize, chosen: u32, index: u64, emit: &mut dyn FnMut(u64)) {
        if position == self.order.len() {
            emit(index);
            let eligible: Vec<usize> = (0..self.info.write_count).filter(|&write| self.in_place[write] && chosen & (1u32 << write) != 0 && self.successors[write] & chosen == 0).collect();
            for subset in 1u32..(1u32 << eligible.len()) {
                let torn_index = eligible.iter().enumerate().filter(|(position, _)| subset & (1u32 << position) != 0).fold(index, |sum, (_, &write)| sum + self.info.weight[write]);
                emit(torn_index);
            }
            return;
        }
        let write = self.order[position];
        self.extend(position + 1, chosen, index, emit);
        if self.masks[write] & !chosen == 0 {
            self.extend(position + 1, chosen | (1u32 << write), index + self.info.weight[write], emit);
        }
    }
}

/// 一份偏序的全部可达状态（序号）：偏序的下闭集，撕裂态只在原地覆写是下闭集的极大元时取（与 `reachable_states_of_partial_order` 同义，只是按下闭集递归、不扫 2^n）。
fn for_each_exact_state(info: &TableInfo, direct: &[u32; MAXIMUM_STREAMING_WRITES], emit: &mut dyn FnMut(u64)) {
    let masks = close_masks(direct, info.write_count);
    let mut order: Vec<usize> = (0..info.write_count).collect();
    order.sort_by_key(|&write| (masks[write].count_ones(), write));
    let successors: Vec<u32> = (0..info.write_count).map(|write| (0..info.write_count).filter(|&later| masks[later] & (1u32 << write) != 0).fold(0u32, |sum, later| sum | (1u32 << later))).collect();
    let in_place: Vec<bool> = info.radix.iter().map(|write_radix| *write_radix == 3).collect();
    let walk = ExactWalk { info, masks, order, successors, in_place };
    walk.extend(0, 0, 0, emit);
}

fn evaluate_exact_union(info: &TableInfo, direct_masks: &[[u32; MAXIMUM_STREAMING_WRITES]], threads: usize) -> Bitset {
    let next_position = AtomicUsize::new(0);
    let parts: Vec<Bitset> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|_| {
                scope.spawn(|| {
                    let mut union = Bitset::new(info.total_vectors);
                    loop {
                        let position = next_position.fetch_add(1, AtomicOrdering::Relaxed);
                        if position >= direct_masks.len() {
                            break;
                        }
                        for_each_exact_state(info, &direct_masks[position], &mut |index| union.set(index));
                    }
                    union
                })
            })
            .collect();
        handles.into_iter().map(|handle| handle.join().expect("线程不 panic")).collect()
    });
    let mut union = Bitset::new(info.total_vectors);
    for part in &parts {
        union.or_with(part);
    }
    union
}

// ── 算法 (b) 的定长数组版 ─────────────────────────────────────────────────────────────────

/// 与 `some_stream_realizes_target` 同一个搜索（状态 = 两条链各走到哪、未放行的盘、当前段的相、上一段的相），
/// 只是状态压成 16 位、已访问用世代戳的定长数组，不用哈希表。相：0 之前（必须全持久）、1 当前（任意）、2 之后（必须全没持久）。
struct Searcher<'a> {
    context: &'a WalkContext,
    stamps: Vec<u32>,
    epoch: u32,
    stack: Vec<u16>,
    visited_states: u64,
}

impl<'a> Searcher<'a> {
    fn new(context: &'a WalkContext) -> Searcher<'a> {
        Searcher { context, stamps: vec![0; 1 << 16], epoch: 0, stack: Vec::new(), visited_states: 0 }
    }

    /// 低六位：未放行盘（两位）、当前段的相加一（两位，0 = 没有开着的段）、上一段的相（两位）；高十位：两条链的位置各五位。
    fn successors(&self, packed: u16, step: Step, target: u64, out: &mut [u16; 3]) -> usize {
        let position_part = packed & !0x3F;
        let unreleased = (packed >> 4) & 3;
        let open = (packed >> 2) & 3;
        let last = packed & 3;
        let compose = |unreleased: u16, open: u16, last: u16| position_part | (unreleased << 4) | (open << 2) | last;
        match step {
            Step::Barrier { disk } => {
                let unreleased_after = unreleased & !(1u16 << disk);
                let (open_after, last_after) = if open != 0 && unreleased_after == 0 { (0, open - 1) } else { (open, last) };
                out[0] = compose(unreleased_after, open_after, last_after);
                1
            }
            Step::Write { index, disk, fua } => {
                let value = ((target >> (2 * u32::from(index))) & 3) as u8;
                let mut produced = 0usize;
                for role in 0u16..3 {
                    if open != 0 {
                        if role != open - 1 {
                            continue;
                        }
                    } else if !(role >= last && !(role == 1 && last == 1)) {
                        continue;
                    }
                    let allowed = match role {
                        0 => value == PERSISTED,
                        1 => true,
                        _ => value == NOT_PERSISTED,
                    };
                    if !allowed {
                        continue;
                    }
                    let unreleased_after = if fua { unreleased & !(1u16 << disk) } else { unreleased | (1u16 << disk) };
                    let (open_after, last_after) = if fua && unreleased_after == 0 { (0, role) } else { (role + 1, last) };
                    out[produced] = compose(unreleased_after, open_after, last_after);
                    produced += 1;
                }
                produced
            }
        }
    }

    fn realizable(&mut self, target: u64) -> bool {
        self.epoch += 1;
        let epoch = self.epoch;
        self.stack.clear();
        self.stamps[0] = epoch;
        self.stack.push(0);
        let (length_a, length_b) = (self.context.steps_a.len(), self.context.steps_b.len());
        let mut out = [0u16; 3];
        while let Some(packed) = self.stack.pop() {
            self.visited_states += 1;
            let position_a = usize::from(packed >> 11);
            let position_b = usize::from((packed >> 6) & 31);
            if position_a == length_a && position_b == length_b {
                return true;
            }
            if position_a < length_a {
                let produced = self.successors(packed, self.context.steps_a[position_a], target, &mut out);
                for &state in &out[..produced] {
                    let next = state + (1 << 11);
                    if self.stamps[usize::from(next)] != epoch {
                        self.stamps[usize::from(next)] = epoch;
                        self.stack.push(next);
                    }
                }
            }
            if position_b < length_b && self.context.b_step_may_follow(position_a, position_b) {
                let produced = self.successors(packed, self.context.steps_b[position_b], target, &mut out);
                for &state in &out[..produced] {
                    let next = state + (1 << 6);
                    if self.stamps[usize::from(next)] != epoch {
                        self.stamps[usize::from(next)] = epoch;
                        self.stack.push(next);
                    }
                }
            }
        }
        false
    }
}

fn search_truth(context: &WalkContext, info: &TableInfo, threads: usize) -> Bitset {
    const CHUNK: u64 = 4096;
    let next_chunk = AtomicUsize::new(0);
    let parts: Vec<Bitset> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|_| {
                scope.spawn(|| {
                    let mut found = Bitset::new(info.total_vectors);
                    let mut searcher = Searcher::new(context);
                    loop {
                        let start = next_chunk.fetch_add(1, AtomicOrdering::Relaxed) as u64 * CHUNK;
                        if start >= info.total_vectors {
                            break;
                        }
                        for index in start..(start + CHUNK).min(info.total_vectors) {
                            if searcher.realizable(info.code_of_index(index)) {
                                found.set(index);
                            }
                        }
                    }
                    found
                })
            })
            .collect();
        handles.into_iter().map(|handle| handle.join().expect("线程不 panic")).collect()
    });
    let mut found = Bitset::new(info.total_vectors);
    for part in &parts {
        found.or_with(part);
    }
    found
}

/// 登记 S3 要报的「逐状态深搜的规模」：每隔 `stride` 个状态向量跑一次搜索，报跑了几个、共访问几个搜索状态、单个最多访问几个、几个搜得到。
fn search_sample(context: &WalkContext, info: &TableInfo, stride: u64) -> (u64, u64, u64, u64) {
    let mut searcher = Searcher::new(context);
    let (mut vectors, mut visited_maximum, mut realized) = (0u64, 0u64, 0u64);
    let mut index = 0u64;
    while index < info.total_vectors {
        let before = searcher.visited_states;
        if searcher.realizable(info.code_of_index(index)) {
            realized += 1;
        }
        visited_maximum = visited_maximum.max(searcher.visited_states - before);
        vectors += 1;
        index += stride;
    }
    (vectors, searcher.visited_states, visited_maximum, realized)
}

fn truth_by_search_fast(world: &World) -> StateList {
    let info = TableInfo::new(&world.table);
    let context = WalkContext::new(world, &info);
    truth_list_of(&info, &search_truth(&context, &info, worker_thread_count()))
}

// ── 两条链的世界，不存流 ──────────────────────────────────────────────────────────────────

fn build_two_chain_world_without_streams(candidate: Candidate, geometry: &Geometry) -> World {
    let mut table = Vec::new();
    let writes_of_a = push_transaction_writes(&mut table, geometry, Owner::TransactionA, geometry.root_disk_of_transaction_a);
    let writes_of_b = push_transaction_writes(&mut table, geometry, Owner::TransactionB, geometry.root_disk_of_transaction_b);
    let chain_a = single_transaction_chain(&writes_of_a, geometry.disk_count);
    let chain_b = single_transaction_chain(&writes_of_b, geometry.disk_count);
    let mut serial_stream = chain_a.clone();
    serial_stream.extend(chain_b.iter().copied());
    let waits = match candidate {
        Candidate::SerialPipeline => vec![Some(chain_a.len() - 1); chain_b.len()],
        Candidate::ManyPublishesInFlightFree => vec![None; chain_b.len()],
        Candidate::ManyPublishesInFlightOrdered => ordering_waits(&table, &chain_a, &chain_b),
        Candidate::OnePublishManyTransactions => unreachable!("一次发布装多个事务不走两条链的交错"),
    };
    World {
        candidate,
        geometry: geometry.clone(),
        table,
        streams: Vec::new(),
        search_chains: [chain_a.clone(), chain_b.clone()],
        search_waits_of_second_chain: waits,
        arm_chains: [chain_a, chain_b],
        arm_tail: Vec::new(),
        serial_stream: Some(serial_stream),
    }
}

/// 路径（每步一位，高位在前，0 = A、1 = B）→ 发出流的文字。
fn render_path(world: &World, path: u64) -> String {
    render_stream(&world.table, &events_of_path(world, path))
}

fn events_of_path(world: &World, path: u64) -> Vec<Event> {
    let [chain_a, chain_b] = &world.search_chains;
    let total_steps = chain_a.len() + chain_b.len();
    let (mut position_a, mut position_b) = (0usize, 0usize);
    let mut events = Vec::new();
    for step in 0..total_steps {
        if (path >> (total_steps - 1 - step)) & 1 == 0 {
            events.push(chain_a[position_a]);
            position_a += 1;
        } else {
            events.push(chain_b[position_b]);
            position_b += 1;
        }
    }
    events
}

struct StreamingEvaluation {
    label: &'static str,
    sampling_point: &'static str,
    world: World,
    info: TableInfo,
    leaves: u64,
    expected_leaves: u128,
    order_masks: usize,
    rows: Vec<SignatureRow>,
    truth: StateList,
    truth_by_search: StateList,
    truth_cross: usize,
    truth_invalid_record: usize,
    truth_invalid_all_writes: usize,
    arm_by_transaction: StateList,
    exact_reachable: StateList,
}

fn evaluate_streaming_cell(spec: &CellSpec, threads: usize) -> StreamingEvaluation {
    let world = build_two_chain_world_without_streams(spec.candidate, &spec.geometry);
    let info = TableInfo::new(&world.table);
    let context = WalkContext::new(&world, &info);
    let expected_leaves = count_interleavings(&context);
    let pass = pass_one(&context, threads);
    let (union, rows) = evaluate_signatures(&info, &pass.signatures, threads);
    let truth = truth_list_of(&info, &union);
    let truth_by_search = truth_list_of(&info, &search_truth(&context, &info, threads));
    let exact_reachable = truth_list_of(&info, &evaluate_exact_union(&info, &pass.masks, threads));
    let arm_by_transaction = arm_by_transaction_states(&world);
    let code_of = |state: &StateCode| u64::try_from(*state).expect("两盘几何的状态编码装得进 u64");
    let truth_cross = truth.iter().filter(|state| info.is_cross(code_of(state))).count();
    let truth_invalid_record = truth.iter().filter(|state| info.is_invalid_by_record(code_of(state))).count();
    let truth_invalid_all_writes = truth.iter().filter(|state| info.is_invalid_by_all_writes(code_of(state))).count();
    StreamingEvaluation {
        label: spec.label,
        sampling_point: spec.sampling_point,
        world,
        info,
        leaves: pass.leaves,
        expected_leaves,
        order_masks: pass.masks.len(),
        rows,
        truth,
        truth_by_search,
        truth_cross,
        truth_invalid_record,
        truth_invalid_all_writes,
        arm_by_transaction,
        exact_reachable,
    }
}

// ── 第二段的单元格输出 ────────────────────────────────────────────────────────────────────

/// 第二段每格要留到末尾判决用的读数（`CellJudgement` 之外再加 L3a、L3b、L10 与 K1 的最大一段）。
struct SecondJudgement {
    base: CellJudgement,
    l3a: usize,
    l3b: usize,
    exact_not_in_truth: usize,
    truth_not_in_exact: usize,
    best_stream_recorded_covered: usize,
    /// K2 在「按记录」那一档上覆盖到的不合法前缀数。
    arm_recorded_covered: usize,
    k1_largest_segment_writes: usize,
    k1_largest_segment_expansion: u128,
}

/// L3b：|(T ∩ T_exact ∩ C) \ 臂|。
fn true_cross_misses(table: &[WriteSpec], truth: &[StateCode], exact: &[StateCode], arm: &[StateCode]) -> usize {
    let reachable_truth: Vec<StateCode> = truth.iter().copied().filter(|state| contains(exact, *state)).collect();
    count_cross_transaction_misses(table, &reachable_truth, arm)
}

fn second_judgement_of_in_memory(evaluation: &CellEvaluation, base: CellJudgement) -> SecondJudgement {
    let table = &evaluation.world.table;
    SecondJudgement {
        l3a: base.arm_by_transaction_cross_misses,
        l3b: true_cross_misses(table, &evaluation.truth, &evaluation.exact_reachable, &evaluation.arm_by_transaction),
        exact_not_in_truth: difference_count(&evaluation.exact_reachable, &evaluation.truth),
        truth_not_in_exact: difference_count(&evaluation.truth, &evaluation.exact_reachable),
        best_stream_recorded_covered: evaluation.streams.iter().map(|metrics| metrics.recorded_prefixes_covered).max().unwrap_or(0),
        arm_recorded_covered: count_invalid_prefixes_covered(table, &evaluation.truth, &evaluation.arm_by_transaction, InvalidPrefixKind::ByRecord),
        k1_largest_segment_writes: evaluation.streams.iter().map(|metrics| metrics.largest_segment_writes).max().unwrap_or(0),
        k1_largest_segment_expansion: evaluation.streams.iter().map(|metrics| metrics.largest_segment_expansion).max().unwrap_or(1),
        base,
    }
}

/// 第二段的一格（两条链的交错，流式）：行的字段与 `emit_cell` 一一对应，K1 一侧按段签名带条数汇总。
fn emit_streaming_cell(emitter: &mut Emitter, tally: &mut RunTally, evaluation: &StreamingEvaluation) -> SecondJudgement {
    let world = &evaluation.world;
    let table = &world.table;
    let label = evaluation.label;
    let truth = &evaluation.truth;
    let rows = &evaluation.rows;
    let methods_agree = *truth == evaluation.truth_by_search;
    tally.self_check(methods_agree, &format!("V3:truth_methods:{label}"));
    let streams_in_rows: u128 = rows.iter().map(|row| u128::from(row.count)).sum();
    let streams_agree = u128::from(evaluation.leaves) == evaluation.expected_leaves && streams_in_rows == evaluation.expected_leaves;
    tally.self_check(streams_agree, &format!("V3:interleaving_count:{label}"));
    emit(
        emitter,
        format!(
            "name=cell_truth cell={label} sampling_point={} candidate={} disks={} units={} table_writes={} streams={} truth_by_enumeration={} truth_by_search={} methods_agree={methods_agree} cross_transaction_states={}",
            evaluation.sampling_point,
            world.candidate.name(),
            world.geometry.disk_count,
            world.geometry.units_per_transaction,
            table.len(),
            evaluation.leaves,
            truth.len(),
            evaluation.truth_by_search.len(),
            evaluation.truth_cross,
        ),
    );
    emit(
        emitter,
        format!(
            "name=stream_classes cell={label} interleavings={} interleavings_by_lattice_count={} counts_agree={streams_agree} distinct_segment_signatures={} distinct_order_masks={} state_vector_universe={}",
            evaluation.leaves,
            evaluation.expected_leaves,
            rows.len(),
            evaluation.order_masks,
            evaluation.info.total_vectors,
        ),
    );
    let subset_ok = rows.iter().all(|row| row.cross_states as usize <= evaluation.truth_cross && (row.states - row.cross_states) as usize <= truth.len() - evaluation.truth_cross);
    tally.self_check(subset_ok, &format!("V5:k1_domain_inside_truth:{label}"));
    let truth_non_cross = truth.len() - evaluation.truth_cross;
    let cross_misses_of = |row: &SignatureRow| evaluation.truth_cross - row.cross_states as usize;
    let non_cross_misses_of = |row: &SignatureRow| truth_non_cross - (row.states - row.cross_states) as usize;
    let mut minimum_row = &rows[0];
    for row in rows {
        let (candidate_misses, best_misses) = (cross_misses_of(row), cross_misses_of(minimum_row));
        if candidate_misses < best_misses || (candidate_misses == best_misses && row.representative_path < minimum_row.representative_path) {
            minimum_row = row;
        }
    }
    let minimum_cross_misses = cross_misses_of(minimum_row);
    let maximum_cross_misses = rows.iter().map(&cross_misses_of).max().expect("至少一个段签名");
    let weighted = |select: &dyn Fn(&SignatureRow) -> bool| -> u64 { rows.iter().filter(|row| select(row)).map(|row| row.count).sum() };
    let closed_form_mismatches = weighted(&|row| !row.closed_form_matches);
    tally.self_check(closed_form_mismatches == 0, &format!("V6:closed_form:{label}"));
    let serial_stream = world.serial_stream.as_ref().expect("两条链的世界有串行流");
    let serial_states = closed_form_state_count(table, &write_segments(table, serial_stream));
    emit(
        emitter,
        format!(
            "name=arm_by_root_publish cell={label} streams={} states_minimum={} states_maximum={} states_on_serial_stream={serial_states} states_on_minimum_cross_miss_stream={} closed_form_mismatches={closed_form_mismatches} cross_misses_minimum={minimum_cross_misses} cross_misses_maximum={maximum_cross_misses} streams_with_cross_misses={} minimum_cross_miss_stream={} non_cross_misses_maximum={} streams_with_non_cross_misses={}",
            evaluation.leaves,
            rows.iter().map(|row| row.states).min().expect("至少一个段签名"),
            rows.iter().map(|row| row.states).max().expect("至少一个段签名"),
            minimum_row.states,
            weighted(&|row| cross_misses_of(row) > 0),
            render_path(world, minimum_row.representative_path),
            rows.iter().map(&non_cross_misses_of).max().expect("至少一个段签名"),
            weighted(&|row| non_cross_misses_of(row) > 0),
        ),
    );
    let arm = &evaluation.arm_by_transaction;
    let exact = &evaluation.exact_reachable;
    let cross_misses_of_arm = count_cross_transaction_misses(table, truth, arm);
    let true_cross_misses_of_arm = true_cross_misses(table, truth, exact, arm);
    let extra_states = difference_count(arm, truth);
    let non_cross_misses_of_arm = count_non_cross_misses(table, truth, arm);
    let identity_ok = arm.len() + cross_misses_of_arm + non_cross_misses_of_arm == truth.len() + extra_states;
    tally.self_check(identity_ok, &format!("V5:identity:{label}"));
    emit(
        emitter,
        format!(
            "name=arm_by_transaction cell={label} states={} cross_misses={cross_misses_of_arm} true_cross_misses={true_cross_misses_of_arm} non_cross_misses={non_cross_misses_of_arm} states_not_in_truth={extra_states} identity_ok={identity_ok}",
            arm.len(),
        ),
    );
    let mut recorded_truth_count = 0;
    let mut all_writes_truth_count = 0;
    let mut best_stream_recorded_covered = 0;
    let mut arm_recorded_covered = 0;
    for kind in BOTH_INVALID_PREFIX_KINDS {
        let truth_count = count_invalid_prefixes_in(table, truth, kind);
        let covered_of = |row: &SignatureRow| match kind {
            InvalidPrefixKind::ByRecord => row.invalid_record_states as usize,
            InvalidPrefixKind::ByAllWrites => row.invalid_all_writes_states as usize,
        };
        let covered_minimum = rows.iter().map(&covered_of).min().expect("至少一个段签名");
        let covered_maximum = rows.iter().map(&covered_of).max().expect("至少一个段签名");
        let arm_covered = count_invalid_prefixes_covered(table, truth, arm, kind);
        let arm_unreachable = count_invalid_prefixes_in(table, arm, kind) - arm_covered;
        let truth_count_by_masks = match kind {
            InvalidPrefixKind::ByRecord => evaluation.truth_invalid_record,
            InvalidPrefixKind::ByAllWrites => evaluation.truth_invalid_all_writes,
        };
        tally.self_check(truth_count == truth_count_by_masks, &format!("V5:invalid_prefix_masks:{label}:{}", kind.name()));
        match kind {
            InvalidPrefixKind::ByRecord => {
                recorded_truth_count = truth_count;
                best_stream_recorded_covered = covered_maximum;
                arm_recorded_covered = arm_covered;
            }
            InvalidPrefixKind::ByAllWrites => all_writes_truth_count = truth_count,
        }
        emit(
            emitter,
            format!(
                "name=invalid_prefix cell={label} kind={} truth_count={truth_count} arm_by_root_publish_covered_minimum={covered_minimum} arm_by_root_publish_covered_maximum={covered_maximum} arm_by_transaction_covered={arm_covered} arm_by_transaction_not_in_truth={arm_unreachable} arm_by_root_publish_judgement={} arm_by_transaction_judgement={}",
                kind.name(),
                invalid_prefix_judgement(covered_minimum, truth_count),
                invalid_prefix_judgement(arm_covered, truth_count),
            ),
        );
    }
    emit_cover_lines_of(emitter, tally, label, table, truth, arm);
    let largest_writes = rows.iter().map(|row| usize::from(row.largest_segment_writes)).max().expect("至少一个段签名");
    let largest_expansion = rows.iter().map(|row| u128::from(row.largest_segment_expansion)).max().expect("至少一个段签名");
    let total_states_maximum = rows.iter().map(|row| row.states).max().expect("至少一个段签名");
    let cover = cover_by_merging(table, arm);
    let cover_largest_expansion = cover.segments.iter().map(|segment| segment_expansion(table, segment)).max().unwrap_or(0);
    emit(
        emitter,
        format!(
            "name=capacity cell={label} table_writes={} arm_by_root_publish_largest_segment_writes={largest_writes} arm_by_root_publish_largest_segment_states={largest_expansion} arm_by_root_publish_total_states_maximum={total_states_maximum} arm_by_transaction_cover_largest_segment_states={cover_largest_expansion} arm_by_transaction_cover_total_states={} writes_within_limit={} segments_within_u32={} segments_within_u64={}",
            table.len(),
            cover.cover_state_count,
            table.len() <= MAXIMUM_ENUMERATED_WRITES,
            largest_expansion <= MAXIMUM_SEGMENT_STATES_IN_U32 && cover_largest_expansion <= MAXIMUM_SEGMENT_STATES_IN_U32,
            largest_expansion <= MAXIMUM_SEGMENT_STATES_IN_U64 && cover_largest_expansion <= MAXIMUM_SEGMENT_STATES_IN_U64,
        ),
    );
    let exact_not_in_truth = difference_count(exact, truth);
    let truth_not_in_exact = difference_count(truth, exact);
    emit(emitter, format!("name=reference cell={label} exact_states={} exact_not_in_truth={exact_not_in_truth} truth_not_in_exact={truth_not_in_exact}", exact.len()));
    SecondJudgement {
        base: CellJudgement {
            label,
            sampling_point: evaluation.sampling_point,
            candidate: world.candidate,
            disks: world.geometry.disk_count,
            arm_by_root_publish_leak_judgement: leak_judgement(minimum_cross_misses, maximum_cross_misses, 0),
            arm_by_root_publish_minimum_cross_misses: minimum_cross_misses,
            arm_by_root_publish_maximum_cross_misses: maximum_cross_misses,
            arm_by_transaction_cross_misses: cross_misses_of_arm,
            recorded_truth_count,
            all_writes_truth_count,
        },
        l3a: cross_misses_of_arm,
        l3b: true_cross_misses_of_arm,
        exact_not_in_truth,
        truth_not_in_exact,
        best_stream_recorded_covered,
        arm_recorded_covered,
        k1_largest_segment_writes: largest_writes,
        k1_largest_segment_expansion: largest_expansion,
    }
}

/// `emit_cover_lines` 的不依赖 `CellEvaluation` 的版本（写数 > 8，有序划分穷举不做）。
fn emit_cover_lines_of(emitter: &mut Emitter, tally: &mut RunTally, label: &str, table: &[WriteSpec], truth: &StateList, arm: &StateList) {
    for (set_name, target) in [("truth", truth), ("arm_by_transaction", arm)] {
        let cover = cover_by_merging(table, target);
        tally.self_check(cover.covers_target, &format!("L7a:cover_covers_target:{label}:{set_name}"));
        emit(
            emitter,
            format!(
                "name=cover cell={label} set={set_name} target_states={} cover_segments={} cover_largest_segment_writes={} cover_states={} covers_target={} cover_equals_target={} exhaustive_agrees=not_run_more_than_8_writes",
                target.len(),
                cover.segments.len(),
                cover.segments.iter().map(Vec::len).max().unwrap_or(0),
                cover.cover_state_count,
                cover.covers_target,
                cover.equals_target,
            ),
        );
    }
}

// ── 第二段的阳性对照 ──────────────────────────────────────────────────────────────────────

fn first_write_name(table: &[WriteSpec], owner: Owner, kind: StepKind) -> String {
    table.iter().find(|spec| spec.owner == owner && spec.kind == kind).map(|spec| spec.name.clone()).expect("写表里有这个所有者与种类的写")
}

/// PC1、PC2、PC3 与 PC2 在 L3b 上的对应（删 K2 里一个「走得到的跨事务状态」，L3b 恰好多 1）。K1 一侧取「枚举域里有这个状态」的字典序最小的那条流。
fn run_positive_controls_two_disk(
    emitter: &mut Emitter,
    tally: &mut RunTally,
    label: &str,
    world: &World,
    truth: &StateList,
    exact: &StateList,
    arm_by_transaction: &StateList,
    streams_containing: &dyn Fn(StateCode) -> Option<StateList>,
) {
    let table = &world.table;
    if world.candidate == Candidate::SerialPipeline {
        return;
    }
    let unit_of_a = first_write_name(table, Owner::TransactionA, StepKind::UnitWrite);
    let unit_of_b = first_write_name(table, Owner::TransactionB, StepKind::UnitWrite);
    if world.candidate == Candidate::ManyPublishesInFlightFree {
        let witness = state_with_only(table, &[unit_of_b.as_str()]);
        let serial = world.serial_stream.as_ref().expect("两条链的候选有串行流");
        let serial_states = crash_states_of_stream(table, serial);
        let serial_product = serial_concatenation_of_transaction_chains(world);
        for (arm_name, arm_states) in [("arm_by_root_publish", &serial_states), ("arm_by_transaction", &serial_product)] {
            let witness_in_truth = contains(truth, witness);
            let witness_missed = !contains(arm_states, witness);
            let misses = difference_count(truth, arm_states);
            let cross_misses = count_cross_transaction_misses(table, truth, arm_states);
            let ok = witness_in_truth && witness_missed && misses >= 1;
            emit(
                emitter,
                format!(
                    "name=positive_control control=serial_concatenation cell={label} arm={arm_name} witness={} witness_in_truth={witness_in_truth} witness_missed={witness_missed} misses_against_truth={misses} cross_misses={cross_misses} ok={ok}",
                    render_state(table, witness),
                ),
            );
            tally.self_check(ok, &format!("V2:PC1:{label}:{arm_name}"));
        }
    }
    let two_units_state = state_with_only(table, &[unit_of_a.as_str(), unit_of_b.as_str()]);
    if contains(truth, two_units_state) {
        let arm_states = streams_containing(two_units_state).expect("w2 在真值里就有一条流罩得住它");
        let before = count_cross_transaction_misses(table, truth, &arm_states);
        let after = count_cross_transaction_misses(table, truth, &without_state(&arm_states, two_units_state));
        let k2_before = count_cross_transaction_misses(table, truth, arm_by_transaction);
        let k2_after = count_cross_transaction_misses(table, truth, &without_state(arm_by_transaction, two_units_state));
        let ok = after == before + 1 && k2_after == k2_before + 1;
        emit(
            emitter,
            format!(
                "name=positive_control control=delete_one_state cell={label} deleted={} arm_by_root_publish_misses_before={before} arm_by_root_publish_misses_after={after} arm_by_transaction_misses_before={k2_before} arm_by_transaction_misses_after={k2_after} ok={ok}",
                render_state(table, two_units_state),
            ),
        );
        tally.self_check(ok, &format!("V2:PC2:{label}"));
    }
    let prefix_witness = recorded_prefix_witness(table);
    if contains(truth, prefix_witness) && is_invalid_prefix_by_record(table, prefix_witness) {
        let arm_states = streams_containing(prefix_witness).expect("见证状态在真值里就有一条流罩得住它");
        let before = count_invalid_prefixes_covered(table, truth, &arm_states, InvalidPrefixKind::ByRecord);
        let after = count_invalid_prefixes_covered(table, truth, &without_state(&arm_states, prefix_witness), InvalidPrefixKind::ByRecord);
        let k2_before = count_invalid_prefixes_covered(table, truth, arm_by_transaction, InvalidPrefixKind::ByRecord);
        let k2_after = count_invalid_prefixes_covered(table, truth, &without_state(arm_by_transaction, prefix_witness), InvalidPrefixKind::ByRecord);
        let ok = before == after + 1 && k2_before == k2_after + 1;
        emit(
            emitter,
            format!(
                "name=positive_control control=delete_one_invalid_prefix cell={label} deleted={} arm_by_root_publish_covered_before={before} arm_by_root_publish_covered_after={after} arm_by_transaction_covered_before={k2_before} arm_by_transaction_covered_after={k2_after} ok={ok}",
                render_state(table, prefix_witness),
            ),
        );
        tally.self_check(ok, &format!("V2:PC3:{label}"));
    } else {
        emit(
            emitter,
            format!("name=positive_control control=delete_one_invalid_prefix cell={label} status=not_applicable reason=witness_not_in_truth"),
        );
    }
    let removable = truth
        .iter()
        .copied()
        .find(|state| is_cross_transaction(table, *state) && contains(exact, *state) && contains(arm_by_transaction, *state));
    match removable {
        Some(removed) => {
            let before = true_cross_misses(table, truth, exact, arm_by_transaction);
            let after = true_cross_misses(table, truth, exact, &without_state(arm_by_transaction, removed));
            let ok = after == before + 1;
            emit(
                emitter,
                format!(
                    "name=positive_control control=delete_one_true_cross_state cell={label} deleted={} arm_by_transaction_true_misses_before={before} arm_by_transaction_true_misses_after={after} ok={ok}",
                    render_state(table, removed),
                ),
            );
            tally.self_check(ok, &format!("V2:PC2_true_miss:{label}"));
        }
        None => emit(
            emitter,
            format!("name=positive_control control=delete_one_true_cross_state cell={label} status=not_applicable reason=no_state_in_truth_and_exact_and_cross_and_arm"),
        ),
    }
}

// ── 真实发布形上 K1 最大一段（L8）─────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
enum SegmentObjective {
    /// 一段展开出的状态数（2^两态写数 · 3^原地覆写数）。
    Expansion,
    /// 一段里的写数。
    Writes,
}

fn segment_score(objective: SegmentObjective, two_state_writes: u32, in_place_writes: u32) -> u128 {
    match objective {
        SegmentObjective::Expansion => (1u128 << two_state_writes) * 3u128.pow(in_place_writes),
        SegmentObjective::Writes => u128::from(two_state_writes + in_place_writes),
    }
}

/// 在两条链的全部交错上求「一次发出流里最大一段的得分」的最大值：状态 = 两条链各走到哪、未放行的盘、当前段里的两态写数与原地覆写数。
/// 段怎么关与 `operation_segments` 同一条规矩。
struct LargestSegmentSearch {
    steps_a: Vec<Step>,
    steps_b: Vec<Step>,
    in_place: Vec<bool>,
    objective: SegmentObjective,
    memo: FastMap<u64, (u128, u8)>,
}

impl LargestSegmentSearch {
    fn new(world: &World, objective: SegmentObjective) -> LargestSegmentSearch {
        LargestSegmentSearch {
            steps_a: steps_of(&world.table, &world.search_chains[0]),
            steps_b: steps_of(&world.table, &world.search_chains[1]),
            in_place: world.table.iter().map(|spec| spec.is_in_place).collect(),
            objective,
            memo: FastMap::default(),
        }
    }

    /// 走一步之后的（未放行盘，两态写数，原地覆写数，这一步关掉的那一段的得分）。
    fn advance(&self, step: Step, unreleased: u8, two_state: u32, in_place: u32) -> (u8, u32, u32, Option<u128>) {
        let (mut unreleased, mut two_state, mut in_place) = (unreleased, two_state, in_place);
        let may_close;
        match step {
            Step::Write { index, disk, fua } => {
                if self.in_place[usize::from(index)] {
                    in_place += 1;
                } else {
                    two_state += 1;
                }
                if fua {
                    unreleased &= !(1u8 << disk);
                    may_close = true;
                } else {
                    unreleased |= 1u8 << disk;
                    may_close = false;
                }
            }
            Step::Barrier { disk } => {
                unreleased &= !(1u8 << disk);
                may_close = true;
            }
        }
        if may_close && two_state + in_place > 0 && unreleased == 0 {
            (unreleased, 0, 0, Some(segment_score(self.objective, two_state, in_place)))
        } else {
            (unreleased, two_state, in_place, None)
        }
    }

    fn best(&mut self, position_a: usize, position_b: usize, unreleased: u8, two_state: u32, in_place: u32) -> u128 {
        let key = (((position_a as u64 * 64 + position_b as u64) * 4 + u64::from(unreleased)) * 64 + u64::from(two_state)) * 8 + u64::from(in_place);
        if let Some(&(value, _)) = self.memo.get(&key) {
            return value;
        }
        let (length_a, length_b) = (self.steps_a.len(), self.steps_b.len());
        let (value, choice) = if position_a == length_a && position_b == length_b {
            (if two_state + in_place > 0 { segment_score(self.objective, two_state, in_place) } else { 0 }, 2u8)
        } else {
            let mut best_value = 0u128;
            let mut best_choice = 2u8;
            for (choice, may_take) in [(0u8, position_a < length_a), (1u8, position_b < length_b)] {
                if !may_take {
                    continue;
                }
                let step = if choice == 0 { self.steps_a[position_a] } else { self.steps_b[position_b] };
                let (next_a, next_b) = if choice == 0 { (position_a + 1, position_b) } else { (position_a, position_b + 1) };
                let (unreleased_after, two_state_after, in_place_after, closed_score) = self.advance(step, unreleased, two_state, in_place);
                let continued = self.best(next_a, next_b, unreleased_after, two_state_after, in_place_after);
                let value = closed_score.map_or(continued, |score| score.max(continued));
                if best_choice == 2 || value > best_value {
                    best_value = value;
                    best_choice = choice;
                }
            }
            (best_value, best_choice)
        };
        self.memo.insert(key, (value, choice));
        value
    }

    /// 取到最大值的那条发出流（每个状态取字典序最小的选择：有并列时 A 先）。
    fn witness_stream(&mut self, world: &World) -> Vec<Event> {
        let (mut position_a, mut position_b, mut unreleased, mut two_state, mut in_place) = (0usize, 0usize, 0u8, 0u32, 0u32);
        let mut events = Vec::new();
        loop {
            self.best(position_a, position_b, unreleased, two_state, in_place);
            let key = (((position_a as u64 * 64 + position_b as u64) * 4 + u64::from(unreleased)) * 64 + u64::from(two_state)) * 8 + u64::from(in_place);
            let (_, choice) = self.memo[&key];
            let step = match choice {
                0 => {
                    events.push(world.search_chains[0][position_a]);
                    let step = self.steps_a[position_a];
                    position_a += 1;
                    step
                }
                1 => {
                    events.push(world.search_chains[1][position_b]);
                    let step = self.steps_b[position_b];
                    position_b += 1;
                    step
                }
                _ => return events,
            };
            let (unreleased_after, two_state_after, in_place_after, _) = self.advance(step, unreleased, two_state, in_place);
            unreleased = unreleased_after;
            two_state = two_state_after;
            in_place = in_place_after;
        }
    }
}

struct LargestSegment {
    value: u128,
    stream: Vec<Event>,
    writes_in_best_segment: usize,
    in_place_in_best_segment: usize,
    states_in_best_segment: u128,
    replay_agrees: bool,
}

fn largest_segment_of(world: &World, objective: SegmentObjective) -> LargestSegment {
    let mut search = LargestSegmentSearch::new(world, objective);
    let value = search.best(0, 0, 0, 0, 0);
    let stream = search.witness_stream(world);
    let segments = write_segments(&world.table, &stream);
    let score_of = |segment: &Vec<usize>| {
        let in_place = segment.iter().filter(|&&write| world.table[write].is_in_place).count() as u32;
        segment_score(objective, segment.len() as u32 - in_place, in_place)
    };
    let best_segment = segments.iter().max_by_key(|segment| score_of(segment)).expect("流里至少有一段");
    let replay_agrees = score_of(best_segment) == value;
    LargestSegment {
        value,
        writes_in_best_segment: best_segment.len(),
        in_place_in_best_segment: best_segment.iter().filter(|&&write| world.table[write].is_in_place).count(),
        states_in_best_segment: segment_expansion(&world.table, best_segment) - 1,
        replay_agrees,
        stream,
    }
}

fn emit_real_shape_capacity(emitter: &mut Emitter, tally: &mut RunTally) -> bool {
    let geometry = geometry_real_publish_shape();
    let world = build_two_chain_world_without_streams(Candidate::ManyPublishesInFlightFree, &geometry);
    let mut writes_floor_ok = true;
    for (objective, name) in [(SegmentObjective::Expansion, "largest_segment_states"), (SegmentObjective::Writes, "most_writes_in_one_segment")] {
        let largest = largest_segment_of(&world, objective);
        tally.self_check(largest.replay_agrees, &format!("L8:replay_agrees:{name}"));
        if objective == SegmentObjective::Writes {
            writes_floor_ok = largest.writes_in_best_segment >= 2 * geometry.units_per_transaction * geometry.disk_count;
        }
        emit(
            emitter,
            format!(
                "name=capacity_real_shape shape={} objective={name} interleavings=seventy_choose_thirty_five table_writes={} segment_writes={} segment_in_place_writes={} segment_states={} exceeds_u32={} within_u64={} within_enumerated_writes_limit={} dynamic_programming_agrees_with_replay={} stream={}",
                geometry.label,
                world.table.len(),
                largest.writes_in_best_segment,
                largest.in_place_in_best_segment,
                largest.states_in_best_segment,
                largest.states_in_best_segment > MAXIMUM_SEGMENT_STATES_IN_U32,
                largest.states_in_best_segment <= MAXIMUM_SEGMENT_STATES_IN_U64,
                world.table.len() <= MAXIMUM_ENUMERATED_WRITES,
                largest.replay_agrees,
                render_stream(&world.table, &largest.stream),
            ),
        );
    }
    writes_floor_ok
}

// ── 第二段的主流程 ────────────────────────────────────────────────────────────────────────

fn second_segment_geometries() -> [(Geometry, [&'static str; 4], &'static str, &'static str); 2] {
    [
        (geometry_g4a(), ["g4a_serial", "g4a_one_publish", "g4a_free", "g4a_ordered"], "G4a", "G4a_ordered"),
        (geometry_g4b(), ["g4b_serial", "g4b_one_publish", "g4b_free", "g4b_ordered"], "G4b", "G4b_ordered"),
    ]
}

fn run_second_segment(emitter: &mut Emitter) -> RunTally {
    let mut tally = RunTally::default();
    let threads = worker_thread_count();
    emit(
        emitter,
        "name=config model=independent_count_model candidates=4 sampling_points=G4a,G4a_ordered,G4b,G4b_ordered segment=second quantities=L1,L2,L3a,L3b,L4,L5,L6,L7a,L7b,L8,L10 not_run=L9,G5".to_string(),
    );
    let mut judgements: Vec<SecondJudgement> = Vec::new();
    for (geometry, labels, free_point, ordered_point) in second_segment_geometries() {
        for (label, candidate) in [(labels[0], Candidate::SerialPipeline), (labels[1], Candidate::OnePublishManyTransactions)] {
            let spec = CellSpec { label, sampling_point: free_point, geometry: geometry.clone(), candidate };
            let evaluation = evaluate_cell_with(&spec, &truth_by_search_fast);
            let base = emit_cell(emitter, &mut tally, &evaluation);
            let miss_maximum = evaluation.streams.iter().map(|metrics| metrics.cross_misses + metrics.non_cross_misses).max().unwrap_or(0);
            let extra_states = evaluation.streams.iter().map(|metrics| metrics.state_count).max().unwrap_or(0) as isize - evaluation.truth.len() as isize;
            tally.self_check(miss_maximum == 0 && extra_states == 0, &format!("V1:real_baseline:{label}"));
            if candidate == Candidate::OnePublishManyTransactions {
                let stream_states = crash_states_of_stream(&evaluation.world.table, &evaluation.world.streams[0]);
                run_positive_controls_two_disk(
                    emitter,
                    &mut tally,
                    label,
                    &evaluation.world,
                    &evaluation.truth,
                    &evaluation.exact_reachable,
                    &evaluation.arm_by_transaction,
                    &|state| contains(&stream_states, state).then(|| stream_states.clone()),
                );
            }
            judgements.push(second_judgement_of_in_memory(&evaluation, base));
        }
        for (label, candidate, point) in [(labels[2], Candidate::ManyPublishesInFlightFree, free_point), (labels[3], Candidate::ManyPublishesInFlightOrdered, ordered_point)] {
            let spec = CellSpec { label, sampling_point: point, geometry: geometry.clone(), candidate };
            let evaluation = evaluate_streaming_cell(&spec, threads);
            judgements.push(emit_streaming_cell(emitter, &mut tally, &evaluation));
            let mut by_representative: Vec<&SignatureRow> = evaluation.rows.iter().collect();
            by_representative.sort_unstable_by_key(|row| row.representative_path);
            let table = &evaluation.world.table;
            let find_stream = |state: StateCode| -> Option<StateList> {
                by_representative.iter().find_map(|row| {
                    let segments: Vec<Vec<usize>> = segments_of_signature(row.key, evaluation.info.write_count)
                        .into_iter()
                        .map(|mask| (0..evaluation.info.write_count).filter(|write| mask & (1u32 << write) != 0).collect())
                        .collect();
                    state_is_in_crash_states_of_segments(&segments, state).then(|| crash_states_of_segments(table, &segments))
                })
            };
            run_positive_controls_two_disk(emitter, &mut tally, label, &evaluation.world, &evaluation.truth, &evaluation.exact_reachable, &evaluation.arm_by_transaction, &find_stream);
        }
    }
    run_second_threshold_shifts(emitter, &mut tally, &judgements);
    run_second_geometry_sensitivity(emitter, &judgements);
    let writes_floor_ok = emit_real_shape_capacity(emitter, &mut tally);
    run_two_disk_dynamic_programming_checks(emitter, &mut tally, &judgements);
    run_second_failure_clauses(emitter, &judgements, writes_floor_ok);
    for judgement in &judgements {
        emit(
            emitter,
            format!(
                "name=verdict cell={} sampling_point={} arm_by_root_publish_l2_judgement={} arm_by_root_publish_cross_misses_minimum={} arm_by_root_publish_cross_misses_maximum={} arm_by_transaction_cross_misses={} arm_by_transaction_true_cross_misses={} arm_by_transaction_l3b_judgement={} exact_not_in_truth={} truth_not_in_exact={}",
                judgement.base.label,
                judgement.base.sampling_point,
                judgement.base.arm_by_root_publish_leak_judgement,
                judgement.base.arm_by_root_publish_minimum_cross_misses,
                judgement.base.arm_by_root_publish_maximum_cross_misses,
                judgement.l3a,
                judgement.l3b,
                leak_judgement(judgement.l3b, judgement.l3b, 0),
                judgement.exact_not_in_truth,
                judgement.truth_not_in_exact,
            ),
        );
    }
    tally
}

fn judgement_of<'a>(judgements: &'a [SecondJudgement], label: &str) -> &'a SecondJudgement {
    judgements.iter().find(|judgement| judgement.base.label == label).expect("这一格在第二段里跑过")
}

/// 判别力自证（登记第八节末）：L2、L3b、L6 的门槛各挪一次，判定必须翻；L3b 在两个几何上都是 0 时没有可翻的，如实写 not_applicable。
fn run_second_threshold_shifts(emitter: &mut Emitter, tally: &mut RunTally, judgements: &[SecondJudgement]) {
    let free = judgement_of(judgements, "g4a_free");
    let (minimum, maximum) = (free.base.arm_by_root_publish_minimum_cross_misses, free.base.arm_by_root_publish_maximum_cross_misses);
    let before = leak_judgement(minimum, maximum, 0);
    let after = leak_judgement(minimum, maximum, maximum);
    let flips = before != after && after == "never_leaks";
    emit(
        emitter,
        format!("name=threshold_shift quantity=l2_cross_misses cell=g4a_free threshold_before=0 judgement_before={before} threshold_after={maximum} judgement_after={after} flips_as_required={flips}"),
    );
    tally.self_check(flips, "threshold_shift:l2:g4a_free");
    for label in ["g4a_free", "g4b_free"] {
        let cell = judgement_of(judgements, label);
        if cell.l3b > 0 {
            let before = leak_judgement(cell.l3b, cell.l3b, 0);
            let after = leak_judgement(cell.l3b, cell.l3b, cell.l3b);
            let flips = before == "leaks_on_every_stream" && after == "never_leaks";
            emit(
                emitter,
                format!("name=threshold_shift quantity=l3b_true_cross_misses cell={label} threshold_before=0 judgement_before={before} threshold_after={} judgement_after={after} flips_as_required={flips}", cell.l3b),
            );
            tally.self_check(flips, &format!("threshold_shift:l3b:{label}"));
        } else {
            emit(
                emitter,
                format!("name=threshold_shift quantity=l3b_true_cross_misses cell={label} status=not_applicable reason=l3b_is_zero_so_there_is_no_positive_judgement_to_flip"),
            );
        }
    }
    // L6 的门槛是「要求覆盖的个数」：覆盖数 < 要求 记漏。两个方向各挪一次：本来判漏的挪到覆盖数（必须翻成覆盖），本来判覆盖的挪到覆盖数加一（必须翻成漏）。
    let required = free.base.recorded_truth_count;
    for (quantity, covered) in [("l6_recorded_prefixes_best_stream", free.best_stream_recorded_covered), ("l6_recorded_prefixes_arm_by_transaction", free.arm_recorded_covered)] {
        let before = invalid_prefix_judgement(covered, required);
        let required_after = if covered < required { covered } else { covered + 1 };
        let after = invalid_prefix_judgement(covered, required_after);
        let flips = before != after && (before == "misses_invalid_prefixes" || before == "covers_every_invalid_prefix");
        emit(
            emitter,
            format!("name=threshold_shift quantity={quantity} cell=g4a_free covered={covered} required_before={required} judgement_before={before} required_after={required_after} judgement_after={after} flips_as_required={flips}"),
        );
        tally.self_check(flips, &format!("threshold_shift:{quantity}:g4a_free"));
    }
}

fn run_second_geometry_sensitivity(emitter: &mut Emitter, judgements: &[SecondJudgement]) {
    let (free_a, free_b) = (judgement_of(judgements, "g4a_free"), judgement_of(judgements, "g4b_free"));
    let same = free_a.base.arm_by_root_publish_leak_judgement == free_b.base.arm_by_root_publish_leak_judgement && (free_a.l3b > 0) == (free_b.l3b > 0);
    emit(
        emitter,
        format!(
            "name=geometry_sensitivity pair=G4a_vs_G4b_free same_judgement={} l2_judgement_g4a={} l2_judgement_g4b={} cross_misses_minimum_g4a={} cross_misses_minimum_g4b={} cross_misses_maximum_g4a={} cross_misses_maximum_g4b={} l3a_g4a={} l3a_g4b={} l3b_g4a={} l3b_g4b={} recorded_truth_g4a={} recorded_truth_g4b={} all_writes_truth_g4a={} all_writes_truth_g4b={}",
            yes_no(same),
            free_a.base.arm_by_root_publish_leak_judgement,
            free_b.base.arm_by_root_publish_leak_judgement,
            free_a.base.arm_by_root_publish_minimum_cross_misses,
            free_b.base.arm_by_root_publish_minimum_cross_misses,
            free_a.base.arm_by_root_publish_maximum_cross_misses,
            free_b.base.arm_by_root_publish_maximum_cross_misses,
            free_a.l3a,
            free_b.l3a,
            free_a.l3b,
            free_b.l3b,
            free_a.base.recorded_truth_count,
            free_b.base.recorded_truth_count,
            free_a.base.all_writes_truth_count,
            free_b.base.all_writes_truth_count,
        ),
    );
    for (free_label, ordered_label, pair_name) in [("g4a_free", "g4a_ordered", "G4a_vs_G4a_ordered"), ("g4b_free", "g4b_ordered", "G4b_vs_G4b_ordered")] {
        let (free, ordered) = (judgement_of(judgements, free_label), judgement_of(judgements, ordered_label));
        let same = free.base.arm_by_root_publish_leak_judgement == ordered.base.arm_by_root_publish_leak_judgement && (free.l3b > 0) == (ordered.l3b > 0);
        emit(
            emitter,
            format!(
                "name=geometry_sensitivity pair={pair_name} same_judgement={} l2_judgement_free={} l2_judgement_ordered={} cross_misses_minimum_free={} cross_misses_minimum_ordered={} cross_misses_maximum_free={} cross_misses_maximum_ordered={} l3a_free={} l3a_ordered={} l3b_free={} l3b_ordered={} recorded_truth_free={} recorded_truth_ordered={} all_writes_truth_free={} all_writes_truth_ordered={}",
                yes_no(same),
                free.base.arm_by_root_publish_leak_judgement,
                ordered.base.arm_by_root_publish_leak_judgement,
                free.base.arm_by_root_publish_minimum_cross_misses,
                ordered.base.arm_by_root_publish_minimum_cross_misses,
                free.base.arm_by_root_publish_maximum_cross_misses,
                ordered.base.arm_by_root_publish_maximum_cross_misses,
                free.l3a,
                ordered.l3a,
                free.l3b,
                ordered.l3b,
                free.base.recorded_truth_count,
                ordered.base.recorded_truth_count,
                free.base.all_writes_truth_count,
                ordered.base.all_writes_truth_count,
            ),
        );
    }
}

/// 两盘几何 Σ_free 上 K1 最大一段：动态规划与「从全部段签名里取最大」两条路逐项对拍（L8 的小流版本，核动态规划本身）。
fn run_two_disk_dynamic_programming_checks(emitter: &mut Emitter, tally: &mut RunTally, judgements: &[SecondJudgement]) {
    for (geometry, labels, _, _) in second_segment_geometries() {
        let free = judgement_of(judgements, labels[2]);
        let world = build_two_chain_world_without_streams(Candidate::ManyPublishesInFlightFree, &geometry);
        let by_states = largest_segment_of(&world, SegmentObjective::Expansion);
        let by_writes = largest_segment_of(&world, SegmentObjective::Writes);
        let agrees = by_states.value == free.k1_largest_segment_expansion && by_writes.value == free.k1_largest_segment_writes as u128 && by_states.replay_agrees && by_writes.replay_agrees;
        emit(
            emitter,
            format!(
                "name=capacity_dynamic_programming_check geometry={} dynamic_programming_largest_expansion={} signatures_largest_expansion={} dynamic_programming_most_writes={} signatures_most_writes={} agrees={agrees}",
                geometry.label, by_states.value, free.k1_largest_segment_expansion, by_writes.value, free.k1_largest_segment_writes,
            ),
        );
        tally.self_check(agrees, &format!("L8:dynamic_programming_vs_signatures:{}", geometry.label));
    }
}

fn run_second_failure_clauses(emitter: &mut Emitter, judgements: &[SecondJudgement], writes_floor_ok: bool) {
    let f1 = judgements.iter().any(|judgement| judgement.exact_not_in_truth > 0);
    emit(emitter, format!("name=failure_clause clause=F1 triggered={} observation=exact_not_in_truth_over_the_eight_two_disk_cells", yes_no(f1)));
    for (free_label, ordered_label) in [("g4a_free", "g4a_ordered"), ("g4b_free", "g4b_ordered")] {
        for label in [free_label, ordered_label] {
            let cell = judgement_of(judgements, label);
            emit(
                emitter,
                format!(
                    "name=failure_clause clause=F2 cell={label} triggered={} l2_minimum={} l3b={}",
                    yes_no(cell.base.arm_by_root_publish_minimum_cross_misses > 0 && cell.l3b > 0),
                    cell.base.arm_by_root_publish_minimum_cross_misses,
                    cell.l3b,
                ),
            );
            emit(
                emitter,
                format!("name=failure_clause clause=F5 cell={label} triggered={} l3a={} l3b={}", yes_no(cell.l3a > 0 && cell.l3b == 0), cell.l3a, cell.l3b),
            );
        }
        let (free, ordered) = (judgement_of(judgements, free_label), judgement_of(judgements, ordered_label));
        emit(
            emitter,
            format!(
                "name=failure_clause clause=F3 pair={free_label}_vs_{ordered_label} triggered={}",
                yes_no(free.base.arm_by_root_publish_leak_judgement != ordered.base.arm_by_root_publish_leak_judgement)
            ),
        );
    }
    for label in ["g4a_free", "g4b_free", "g4a_ordered", "g4b_ordered"] {
        let cell = judgement_of(judgements, label);
        emit(
            emitter,
            format!(
                "name=failure_clause clause=F4 cell={label} recorded_truth_count={} all_writes_truth_count={} triggered_recorded={} triggered_all_writes={}",
                cell.base.recorded_truth_count,
                cell.base.all_writes_truth_count,
                yes_no(cell.base.recorded_truth_count == 0),
                yes_no(cell.base.all_writes_truth_count == 0),
            ),
        );
    }
    emit(
        emitter,
        format!("name=failure_clause clause=F6 triggered={} observation=most_writes_in_one_segment_on_the_real_publish_shape_below_48", yes_no(!writes_floor_ok)),
    );
}

// ── 第三段补一格：L11 K1 对 T ∩ T_exact ∩ C 的跨事务漏 ───────────────────────────────────
//
// 登记第十二节修订 10：K1 在一条录制流 σ 上的 L11(σ) = |(T ∩ T_exact ∩ C) \ Dom4(σ)|，与 L3b 同口径（参照取 T ∩ T_exact，只数跨事务状态）。
// 每个段签名的枚举域装置里已有；这里不再存枚举域，换两条做法：(A) 目标集里每个状态逐个按「存在一个当前段」的定义判在不在这串段里；
// (B) 把目标集的混合进制序号放进位集，对每个签名枚举 Dom4(σ) 数命中。两条逐签名相等才算数。

/// 每段的（两位一写的写位掩码，各写取 1 的掩码）。
fn segment_digit_masks(segments: &[u32]) -> Vec<(u64, u64)> {
    segments
        .iter()
        .map(|segment| {
            let (mut any, mut ones) = (0u64, 0u64);
            for write in 0..MAXIMUM_STREAMING_WRITES {
                if segment & (1u32 << write) != 0 {
                    any |= 0b11u64 << (2 * write);
                    ones |= 0b01u64 << (2 * write);
                }
            }
            (any, ones)
        })
        .collect()
}

/// 方法 A：一个状态在不在一串段的枚举域里（存在一个当前段，前面各段全持久、后面各段全没持久）；全零状态总在。
fn code_in_digit_masks(masks: &[(u64, u64)], code: u64) -> bool {
    code == 0
        || (0..masks.len()).any(|current| {
            masks.iter().enumerate().all(|(position, &(any, ones))| {
                if position < current {
                    code & any == ones
                } else if position > current {
                    code & any == 0
                } else {
                    true
                }
            })
        })
}

/// 目标集里不在这串段枚举域里的个数；`removed` 是阳性对照里「从枚举域里删掉」的那个状态。
fn true_cross_misses_of_masks(masks: &[(u64, u64)], target: &[u64], removed: Option<u64>) -> usize {
    target.iter().filter(|&&code| Some(code) == removed || !code_in_digit_masks(masks, code)).count()
}

/// T ∩ T_exact ∩ C（用 `is_cross_transaction` 与排好序的 T、T_exact 查，不用掩码版谓词）。
fn true_cross_target(table: &[WriteSpec], truth: &[StateCode], exact: &[StateCode]) -> Vec<u64> {
    truth
        .iter()
        .copied()
        .filter(|state| is_cross_transaction(table, *state) && contains(exact, *state))
        .map(|state| u64::try_from(state).expect("两盘几何的状态编码装得进 u64"))
        .collect()
}

/// 方法 B：对每个段签名枚举 Dom4(σ)，数落在目标集里的个数，返回目标集减它。
fn true_cross_misses_by_enumeration(info: &TableInfo, rows: &[SignatureRow], target: &[u64], threads: usize) -> Vec<u32> {
    let mut target_bits = Bitset::new(info.total_vectors);
    for &code in target {
        target_bits.set(info.index_of_code(code));
    }
    const CHUNK: usize = 64;
    let next_start = AtomicUsize::new(0);
    let parts: Vec<Vec<(usize, u32)>> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|_| {
                scope.spawn(|| {
                    let mut found = Vec::new();
                    loop {
                        let start = next_start.fetch_add(CHUNK, AtomicOrdering::Relaxed);
                        if start >= rows.len() {
                            break;
                        }
                        for (position, row) in rows.iter().enumerate().take((start + CHUNK).min(rows.len())).skip(start) {
                            let segments = segments_of_signature(row.key, info.write_count);
                            let mut hits = 0usize;
                            for_each_crash_state(info, &segments, |_, index| {
                                if target_bits.has(index) {
                                    hits += 1;
                                }
                            });
                            found.push((position, u32::try_from(target.len() - hits).expect("漏数装得进 u32")));
                        }
                    }
                    found
                })
            })
            .collect();
        handles.into_iter().map(|handle| handle.join().expect("线程不 panic")).collect()
    });
    let mut misses = vec![0u32; rows.len()];
    for (position, value) in parts.into_iter().flatten() {
        misses[position] = value;
    }
    misses
}

/// 一格上 K1 的 L11 读数。
struct TrueCrossRead {
    target: Vec<u64>,
    /// 方法 A 的逐签名漏数（与 `rows` 同序）。
    misses: Vec<u32>,
    methods_agree: bool,
    minimum: u32,
    minimum_row: usize,
    maximum: u32,
    maximum_row: usize,
    streams_with_misses: u64,
    signatures_with_misses: usize,
    serial_misses: usize,
    serial_agrees_across_methods: bool,
}

fn read_k1_true_cross(evaluation: &StreamingEvaluation, threads: usize) -> TrueCrossRead {
    let table = &evaluation.world.table;
    let rows = &evaluation.rows;
    let info = &evaluation.info;
    let target = true_cross_target(table, &evaluation.truth, &evaluation.exact_reachable);
    let misses: Vec<u32> = rows
        .iter()
        .map(|row| {
            let masks = segment_digit_masks(&segments_of_signature(row.key, info.write_count));
            u32::try_from(true_cross_misses_of_masks(&masks, &target, None)).expect("漏数装得进 u32")
        })
        .collect();
    let methods_agree = misses == true_cross_misses_by_enumeration(info, rows, &target, threads);
    // 取最小与最大时并列的，取字典序最小的那条流（与 `emit_streaming_cell` 取最小值那条流同一规矩）。
    let better = |candidate: usize, current: usize, prefer_larger: bool| {
        let (a, b) = (misses[candidate], misses[current]);
        let strictly = if prefer_larger { a > b } else { a < b };
        strictly || (a == b && rows[candidate].representative_path < rows[current].representative_path)
    };
    let (mut minimum_row, mut maximum_row) = (0usize, 0usize);
    for position in 0..rows.len() {
        if better(position, minimum_row, false) {
            minimum_row = position;
        }
        if better(position, maximum_row, true) {
            maximum_row = position;
        }
    }
    let serial = evaluation.world.serial_stream.as_ref().expect("两条链的世界有串行流");
    let serial_segments = write_segments(table, serial);
    let serial_masks: Vec<u32> = serial_segments.iter().map(|segment| segment.iter().fold(0u32, |mask, &write| mask | (1u32 << write))).collect();
    let by_masks = true_cross_misses_of_masks(&segment_digit_masks(&serial_masks), &target, None);
    let by_table = target.iter().filter(|&&code| !state_is_in_crash_states_of_segments(&serial_segments, StateCode::from(code))).count();
    TrueCrossRead {
        minimum: misses[minimum_row],
        maximum: misses[maximum_row],
        streams_with_misses: rows.iter().zip(&misses).filter(|(_, &miss)| miss > 0).map(|(row, _)| row.count).sum(),
        signatures_with_misses: misses.iter().filter(|&&miss| miss > 0).count(),
        serial_misses: by_masks,
        serial_agrees_across_methods: by_masks == by_table,
        minimum_row,
        maximum_row,
        methods_agree,
        misses,
        target,
    }
}


/// 第三段每格要留到末尾判决用的读数。
struct TrueCrossJudgement {
    label: &'static str,
    sampling_point: &'static str,
    minimum: usize,
    maximum: usize,
    streams_with_misses: u64,
    arm_by_transaction_true_cross_misses: usize,
}

/// 第三段的一格：L11 的两条算法、与第二段 L2 的逐签名比、阳性对照 PC1（串行拼接）与 PC2（删一态）。
fn emit_true_cross_cell(emitter: &mut Emitter, tally: &mut RunTally, evaluation: &StreamingEvaluation, threads: usize) -> TrueCrossJudgement {
    let (label, table, rows, info) = (evaluation.label, &evaluation.world.table, &evaluation.rows, &evaluation.info);
    tally.self_check(evaluation.truth == evaluation.truth_by_search, &format!("V3:truth_methods:{label}"));
    let read = read_k1_true_cross(evaluation, threads);
    let target_size = read.target.len();
    emit(
        emitter,
        format!(
            "name=true_cross_reference cell={label} sampling_point={} candidate={} truth_states={} exact_states={} truth_and_exact_states={} truth_cross_states={} truth_and_exact_cross_states={target_size}",
            evaluation.sampling_point,
            evaluation.world.candidate.name(),
            evaluation.truth.len(),
            evaluation.exact_reachable.len(),
            evaluation.truth.len() - difference_count(&evaluation.truth, &evaluation.exact_reachable),
            evaluation.truth_cross,
        ),
    );
    tally.self_check(read.methods_agree, &format!("V5:l11_two_methods:{label}"));
    tally.self_check(read.serial_agrees_across_methods, &format!("V5:l11_serial_two_membership_tests:{label}"));
    let below_l2 = rows.iter().zip(&read.misses).all(|(row, &miss)| miss as usize <= evaluation.truth_cross - row.cross_states as usize);
    tally.self_check(below_l2 && target_size <= evaluation.truth_cross, &format!("V5:l11_not_above_l2:{label}"));
    let (arm, exact) = (&evaluation.arm_by_transaction, &evaluation.exact_reachable);
    let l3b = true_cross_misses(table, &evaluation.truth, exact, arm);
    let streams_in_rows: u64 = rows.iter().map(|row| row.count).sum();
    tally.self_check(u128::from(streams_in_rows) == evaluation.expected_leaves, &format!("V3:interleaving_count:{label}"));
    emit(
        emitter,
        format!(
            "name=arm_by_root_publish_true_cross cell={label} streams={streams_in_rows} signatures={} true_cross_states={target_size} true_cross_misses_minimum={} true_cross_misses_maximum={} streams_with_true_cross_misses={} streams_with_zero_true_cross_misses={} signatures_with_true_cross_misses={} true_cross_misses_on_serial_stream={} minimum_true_cross_miss_stream={} maximum_true_cross_miss_stream={} two_methods_agree={} judgement={} arm_by_transaction_true_cross_misses={l3b}",
            rows.len(),
            read.minimum,
            read.maximum,
            read.streams_with_misses,
            streams_in_rows - read.streams_with_misses,
            read.signatures_with_misses,
            read.serial_misses,
            render_path(&evaluation.world, rows[read.minimum_row].representative_path),
            render_path(&evaluation.world, rows[read.maximum_row].representative_path),
            yes_no(read.methods_agree),
            leak_judgement(read.minimum as usize, read.maximum as usize, 0),
        ),
    );
    // PC1：串行拼接那一条流在 K1 上必须漏掉见证状态 {a1, b1}（A、B 各只有第一个单元写持久：跨事务，T、T_exact 里都有；单独的 b1 不是跨事务状态，不当见证）。
    let unit_of_a = first_write_name(table, Owner::TransactionA, StepKind::UnitWrite);
    let unit_of_b = first_write_name(table, Owner::TransactionB, StepKind::UnitWrite);
    let witness = state_with_only(table, &[unit_of_a.as_str(), unit_of_b.as_str()]);
    let witness_code = u64::try_from(witness).expect("两盘几何的状态编码装得进 u64");
    let serial = evaluation.world.serial_stream.as_ref().expect("两条链的世界有串行流");
    let witness_in_target = read.target.contains(&witness_code);
    let witness_missed = !state_is_in_crash_states_of_segments(&write_segments(table, serial), witness);
    let pc1_ok = witness_in_target && witness_missed && read.serial_misses >= 1;
    emit(
        emitter,
        format!(
            "name=positive_control control=serial_concatenation_l11 cell={label} witness={} witness_in_true_cross_target={witness_in_target} witness_missed={witness_missed} true_cross_misses_on_serial_stream={} ok={pc1_ok}",
            render_state(table, witness),
            read.serial_misses,
        ),
    );
    tally.self_check(pc1_ok, &format!("V2:PC1_l11:{label}"));
    // PC2：从取到最小漏数的那条流的枚举域里删一个目标集里的状态，L11 恰好多 1。
    let minimum_masks = segment_digit_masks(&segments_of_signature(rows[read.minimum_row].key, info.write_count));
    match read.target.iter().copied().find(|&code| code_in_digit_masks(&minimum_masks, code)) {
        Some(removed) => {
            let before = true_cross_misses_of_masks(&minimum_masks, &read.target, None);
            let after = true_cross_misses_of_masks(&minimum_masks, &read.target, Some(removed));
            let pc2_ok = after == before + 1 && before == read.minimum as usize;
            emit(
                emitter,
                format!("name=positive_control control=delete_one_true_cross_state_from_k1 cell={label} deleted={} true_cross_misses_before={before} true_cross_misses_after={after} ok={pc2_ok}", render_state(table, StateCode::from(removed))),
            );
            tally.self_check(pc2_ok, &format!("V2:PC2_l11:{label}"));
        }
        None => emit(
            emitter,
            format!("name=positive_control control=delete_one_true_cross_state_from_k1 cell={label} status=not_applicable reason=minimum_stream_covers_none_of_the_target"),
        ),
    }
    TrueCrossJudgement {
        label,
        sampling_point: evaluation.sampling_point,
        minimum: read.minimum as usize,
        maximum: read.maximum as usize,
        streams_with_misses: read.streams_with_misses,
        arm_by_transaction_true_cross_misses: l3b,
    }
}

/// 判别力自证：L11 的门槛从 0 挪到最大值，「某些流上漏」必须翻成「不漏」；最大值是 0 时没有可翻的，如实写 not_applicable。
fn emit_true_cross_threshold_shifts(emitter: &mut Emitter, tally: &mut RunTally, judgements: &[TrueCrossJudgement]) {
    for judgement in judgements {
        let label = judgement.label;
        if judgement.maximum > 0 {
            let before = leak_judgement(judgement.minimum, judgement.maximum, 0);
            let after = leak_judgement(judgement.minimum, judgement.maximum, judgement.maximum);
            let flips = before != "never_leaks" && after == "never_leaks";
            emit(
                emitter,
                format!("name=threshold_shift quantity=l11_true_cross_misses cell={label} threshold_before=0 judgement_before={before} threshold_after={} judgement_after={after} flips_as_required={flips}", judgement.maximum),
            );
            tally.self_check(flips, &format!("threshold_shift:l11:{label}"));
        } else {
            emit(
                emitter,
                format!("name=threshold_shift quantity=l11_true_cross_misses cell={label} status=not_applicable reason=l11_maximum_is_zero_so_there_is_no_positive_judgement_to_flip"),
            );
        }
    }
}

fn run_third_segment(emitter: &mut Emitter) -> RunTally {
    let mut tally = RunTally::default();
    let threads = worker_thread_count();
    emit(
        emitter,
        "name=config model=independent_count_model candidates=2 sampling_points=G4a,G4a_ordered,G4b,G4b_ordered segment=third quantities=L11 reuses=L1,T_exact,L3b not_run=L2,L3a,L4,L5,L6,L7,L8,L9,L10,G5".to_string(),
    );
    let mut judgements: Vec<TrueCrossJudgement> = Vec::new();
    for (geometry, labels, free_point, ordered_point) in second_segment_geometries() {
        for (label, candidate, point) in [(labels[2], Candidate::ManyPublishesInFlightFree, free_point), (labels[3], Candidate::ManyPublishesInFlightOrdered, ordered_point)] {
            let spec = CellSpec { label, sampling_point: point, geometry: geometry.clone(), candidate };
            let evaluation = evaluate_streaming_cell(&spec, threads);
            judgements.push(emit_true_cross_cell(emitter, &mut tally, &evaluation, threads));
        }
    }
    emit_true_cross_threshold_shifts(emitter, &mut tally, &judgements);
    let by_label = |label: &str| judgements.iter().find(|judgement| judgement.label == label).expect("这一格在第三段里跑过");
    for (left, right, pair) in [("g4a_free", "g4b_free", "G4a_vs_G4b_free"), ("g4a_free", "g4a_ordered", "G4a_vs_G4a_ordered"), ("g4b_free", "g4b_ordered", "G4b_vs_G4b_ordered")] {
        let (a, b) = (by_label(left), by_label(right));
        let (judgement_a, judgement_b) = (leak_judgement(a.minimum, a.maximum, 0), leak_judgement(b.minimum, b.maximum, 0));
        emit(
            emitter,
            format!(
                "name=geometry_sensitivity quantity=l11 pair={pair} same_judgement={} l11_judgement_left={judgement_a} l11_judgement_right={judgement_b} l11_minimum_left={} l11_minimum_right={} l11_maximum_left={} l11_maximum_right={} l11_streams_with_misses_left={} l11_streams_with_misses_right={}",
                yes_no(judgement_a == judgement_b),
                a.minimum,
                b.minimum,
                a.maximum,
                b.maximum,
                a.streams_with_misses,
                b.streams_with_misses,
            ),
        );
    }
    for judgement in &judgements {
        emit(
            emitter,
            format!(
                "name=verdict cell={} sampling_point={} arm_by_root_publish_l11_judgement={} arm_by_root_publish_true_cross_misses_minimum={} arm_by_root_publish_true_cross_misses_maximum={} arm_by_root_publish_streams_with_true_cross_misses={} arm_by_transaction_true_cross_misses={} arm_by_transaction_l3b_judgement={}",
                judgement.label,
                judgement.sampling_point,
                leak_judgement(judgement.minimum, judgement.maximum, 0),
                judgement.minimum,
                judgement.maximum,
                judgement.streams_with_misses,
                judgement.arm_by_transaction_true_cross_misses,
                leak_judgement(judgement.arm_by_transaction_true_cross_misses, judgement.arm_by_transaction_true_cross_misses, 0),
            ),
        );
    }
    tally
}

// ── 登记 S3 要的规模估计 ──────────────────────────────────────────────────────────────────

/// 子命令 `s3`：G4a、G4b 上 Σ_free 与 Σ_ord 的交错数（两条算法数）、去掉同一段里写序之后的不同段签名数、
/// 不同的偏序数、逐状态深搜的状态全集与每隔 1009 个向量抽一个跑搜索的访问量（只用来估规模，不进任何判据）。
fn run_s3_estimate(emitter: &mut Emitter) {
    for (geometry, labels, _, _) in second_segment_geometries() {
        for (label, candidate) in [(labels[2], Candidate::ManyPublishesInFlightFree), (labels[3], Candidate::ManyPublishesInFlightOrdered)] {
            let started = std::time::Instant::now();
            let world = build_two_chain_world_without_streams(candidate, &geometry);
            let info = TableInfo::new(&world.table);
            let context = WalkContext::new(&world, &info);
            let expected = count_interleavings(&context);
            let pass = pass_one(&context, worker_thread_count());
            eprintln!("s3 {label}: pass one {:?}", started.elapsed());
            let started = std::time::Instant::now();
            let stride = 1009u64;
            let (vectors, visited_total, visited_maximum, realized) = search_sample(&context, &info, stride);
            eprintln!("s3 {label}: search sample {:?}", started.elapsed());
            let space = (context.steps_a.len() + 1) * (context.steps_b.len() + 1) * (1usize << info.disk_count) * 4 * 3;
            emit(
                emitter,
                format!(
                    "name=s3_estimate cell={label} interleavings={} interleavings_by_lattice_count={expected} counts_agree={} distinct_segment_signatures={} distinct_order_masks={} state_vector_universe={} search_state_space={space} sample_stride={stride} sampled_vectors={vectors} sampled_visited_search_states={visited_total} sampled_visited_search_states_maximum={visited_maximum} sampled_realized={realized}",
                    pass.leaves,
                    yes_no(u128::from(pass.leaves) == expected),
                    pass.signatures.len(),
                    pass.masks.len(),
                    info.total_vectors,
                ),
            );
        }
    }
}

// ── 第二段结束标记 ──

#[cfg(test)]
mod tests {
    use super::*;

    fn evaluation_of(label: &str) -> CellEvaluation {
        let spec = cell_specs().into_iter().find(|candidate| candidate.label == label).expect("这个格存在");
        evaluate_cell(&spec)
    }

    // ── 甲类：每条流上的枚举域等于闭式 ──────────────────────────────────────────────

    #[test]
    fn every_stream_of_every_one_disk_cell_has_as_many_states_as_the_closed_form() {
        for label in ["g1_serial", "g1_one_publish", "g1_free", "g1_ordered", "g3_serial", "g3_one_publish", "g3_free", "g3_ordered", "g5_serial", "g5_one_publish", "g5_free", "g5_ordered"] {
            let evaluation = evaluation_of(label);
            for metrics in &evaluation.streams {
                assert_eq!(metrics.state_count as u128, metrics.closed_form_state_count, "{label}");
            }
        }
    }

    #[test]
    fn closed_form_starts_from_one_not_zero() {
        let world = build_world(Candidate::SerialPipeline, &geometry_g1());
        let segments = write_segments(&world.table, world.serial_stream.as_ref().expect("串行流"));
        let sum_of_segments: u128 = segments.iter().map(|segment| segment_expansion(&world.table, segment) - 1).sum();
        assert_eq!(closed_form_state_count(&world.table, &segments), sum_of_segments + 1);
    }

    // ── 丙类：独立算出、命令核过的绝对值 ────────────────────────────────────────────

    #[test]
    fn closed_form_anchor_table_matches_registered_numbers() {
        for row in anchor_rows() {
            for reading in read_anchor_row(&row) {
                assert_eq!(reading.actual, reading.expected, "{} {}", row.geometry.label, reading.quantity);
            }
        }
    }

    #[test]
    fn serial_pipeline_on_one_disk_with_two_units_has_fifteen_states_on_its_only_stream() {
        let evaluation = evaluation_of("g1_serial");
        assert_eq!(evaluation.truth.len(), 15);
        assert_eq!(evaluation.streams.len(), 1);
        assert_eq!(evaluation.streams[0].state_count, 15);
    }

    #[test]
    fn serial_segments_split_at_every_barrier_and_force_unit_access_write() {
        let world = build_world(Candidate::SerialPipeline, &geometry_g1());
        let serial = world.serial_stream.as_ref().expect("串行流");
        assert_eq!(segment_sizes_text(&world.table, serial), "2+1+1+1+2+1+1+1");
    }

    // ── 乙类：登记表钉住的（S1 停机条款）────────────────────────────────────────────

    #[test]
    fn registered_ordinary_publish_chain_has_the_registered_segments_kinds_operations_and_states() {
        let reading = registered_ordinary_publish_reading();
        assert_eq!(reading.segments, "24+2+1+2");
        assert_eq!(reading.kinds, REGISTERED_ORDINARY_PUBLISH_KINDS);
        assert_eq!(reading.operations, 35);
        assert_eq!(reading.closed_form_states, 16_777_223);
        assert_eq!(reading.enumerated_states, 16_777_223);
    }

    #[test]
    fn registered_whole_stream_has_the_registered_segments_kinds_operations_and_states() {
        let two_state = registered_whole_stream_reading(false, true);
        assert_eq!(two_state.segments, REGISTERED_WHOLE_STREAM_SEGMENTS);
        assert_eq!(two_state.kinds, REGISTERED_WHOLE_STREAM_KINDS);
        assert_eq!(two_state.operations, 57);
        assert_eq!(two_state.closed_form_states, 16_777_240);
        assert_eq!(two_state.enumerated_states, 16_777_240);
        let three_state = registered_whole_stream_reading(true, false);
        assert_eq!(three_state.closed_form_states, 16_777_260);
    }

    // ── S2：发布归属 ───────────────────────────────────────────────────────────────

    #[test]
    fn publish_attribution_on_the_serial_stream_matches_the_hand_derivation() {
        assert!(attribution_matches_hand_derivation());
    }

    #[test]
    fn a_unit_write_of_the_next_transaction_issued_before_the_root_is_attributed_to_the_earlier_publish() {
        let world = build_world(Candidate::ManyPublishesInFlightFree, &geometry_g1());
        let index_of = |name: &str| index_of_write(&world.table, name);
        let stream = vec![
            Event::Write(index_of("a1")),
            Event::Write(index_of("b1")),
            Event::Write(index_of("rA")),
            Event::Write(index_of("RA")),
            Event::Write(index_of("rB")),
            Event::Write(index_of("RB")),
        ];
        let attribution = publish_attribution(&world.table, &stream);
        assert_eq!(attribution[0].units, vec![index_of("a1"), index_of("b1")]);
        assert_eq!(attribution[1].records, vec![index_of("rB")]);
    }

    // ── 逐格独立再算（Python 脚本 independent.py 的读数，定义见登记第一节）──────────────────

    struct PinnedCell {
        label: &'static str,
        streams: usize,
        truth: usize,
        states_minimum: usize,
        states_maximum: usize,
        cross_misses_minimum: usize,
        cross_misses_maximum: usize,
        non_cross_misses_maximum: usize,
        cross_states_of_truth: usize,
        arm_by_transaction_not_in_truth: usize,
        recorded_truth: usize,
        recorded_covered_maximum: usize,
        all_writes_truth: usize,
        all_writes_covered_maximum: usize,
    }

    const PINNED_CELLS: [PinnedCell; 6] = [
        PinnedCell { label: "g1_free", streams: 12870, truth: 64, states_minimum: 15, states_maximum: 29, cross_misses_minimum: 27, cross_misses_maximum: 42, non_cross_misses_maximum: 11, cross_states_of_truth: 49, arm_by_transaction_not_in_truth: 0, recorded_truth: 16, recorded_covered_maximum: 13, all_writes_truth: 6, all_writes_covered_maximum: 6 },
        PinnedCell { label: "g1_ordered", streams: 3747, truth: 46, states_minimum: 15, states_maximum: 29, cross_misses_minimum: 13, cross_misses_maximum: 28, non_cross_misses_maximum: 7, cross_states_of_truth: 35, arm_by_transaction_not_in_truth: 18, recorded_truth: 1, recorded_covered_maximum: 1, all_writes_truth: 2, all_writes_covered_maximum: 2 },
        PinnedCell { label: "g3_free", streams: 3432, truth: 36, states_minimum: 11, states_maximum: 17, cross_misses_minimum: 11, cross_misses_maximum: 20, non_cross_misses_maximum: 9, cross_states_of_truth: 25, arm_by_transaction_not_in_truth: 0, recorded_truth: 8, recorded_covered_maximum: 7, all_writes_truth: 4, all_writes_covered_maximum: 4 },
        PinnedCell { label: "g3_ordered", streams: 973, truth: 26, states_minimum: 11, states_maximum: 17, cross_misses_minimum: 5, cross_misses_maximum: 14, non_cross_misses_maximum: 5, cross_states_of_truth: 19, arm_by_transaction_not_in_truth: 10, recorded_truth: 1, recorded_covered_maximum: 1, all_writes_truth: 2, all_writes_covered_maximum: 2 },
        PinnedCell { label: "g5_free", streams: 12870, truth: 49, states_minimum: 13, states_maximum: 24, cross_misses_minimum: 19, cross_misses_maximum: 30, non_cross_misses_maximum: 9, cross_states_of_truth: 36, arm_by_transaction_not_in_truth: 0, recorded_truth: 12, recorded_covered_maximum: 9, all_writes_truth: 5, all_writes_covered_maximum: 5 },
        PinnedCell { label: "g5_ordered", streams: 3747, truth: 36, states_minimum: 13, states_maximum: 24, cross_misses_minimum: 9, cross_misses_maximum: 20, non_cross_misses_maximum: 6, cross_states_of_truth: 26, arm_by_transaction_not_in_truth: 13, recorded_truth: 1, recorded_covered_maximum: 1, all_writes_truth: 1, all_writes_covered_maximum: 1 },
    ];

    #[test]
    fn many_publishes_in_flight_cells_match_the_independent_python_recount() {
        for pinned in &PINNED_CELLS {
            let evaluation = evaluation_of(pinned.label);
            let table = &evaluation.world.table;
            assert_eq!(evaluation.streams.len(), pinned.streams, "{}", pinned.label);
            assert_eq!(evaluation.truth.len(), pinned.truth, "{}", pinned.label);
            let state_counts: Vec<usize> = evaluation.streams.iter().map(|metrics| metrics.state_count).collect();
            assert_eq!(state_counts.iter().copied().min().expect("非空"), pinned.states_minimum, "{}", pinned.label);
            assert_eq!(maximum_value(&state_counts), pinned.states_maximum, "{}", pinned.label);
            let cross_misses: Vec<usize> = evaluation.streams.iter().map(|metrics| metrics.cross_misses).collect();
            assert_eq!(cross_misses.iter().copied().min().expect("非空"), pinned.cross_misses_minimum, "{}", pinned.label);
            assert_eq!(maximum_value(&cross_misses), pinned.cross_misses_maximum, "{}", pinned.label);
            assert!(cross_misses.iter().all(|misses| *misses > 0), "{}：每一条流都漏", pinned.label);
            let non_cross: Vec<usize> = evaluation.streams.iter().map(|metrics| metrics.non_cross_misses).collect();
            assert_eq!(maximum_value(&non_cross), pinned.non_cross_misses_maximum, "{}", pinned.label);
            assert_eq!(evaluation.truth.iter().filter(|state| is_cross_transaction(table, **state)).count(), pinned.cross_states_of_truth, "{}", pinned.label);
            assert_eq!(difference_count(&evaluation.arm_by_transaction, &evaluation.truth), pinned.arm_by_transaction_not_in_truth, "{}", pinned.label);
            assert_eq!(count_cross_transaction_misses(table, &evaluation.truth, &evaluation.arm_by_transaction), 0, "{}", pinned.label);
            assert_eq!(count_invalid_prefixes_in(table, &evaluation.truth, InvalidPrefixKind::ByRecord), pinned.recorded_truth, "{}", pinned.label);
            assert_eq!(count_invalid_prefixes_in(table, &evaluation.truth, InvalidPrefixKind::ByAllWrites), pinned.all_writes_truth, "{}", pinned.label);
            let recorded_covered: Vec<usize> = evaluation.streams.iter().map(|metrics| metrics.recorded_prefixes_covered).collect();
            let all_writes_covered: Vec<usize> = evaluation.streams.iter().map(|metrics| metrics.all_writes_prefixes_covered).collect();
            assert_eq!(maximum_value(&recorded_covered), pinned.recorded_covered_maximum, "{}", pinned.label);
            assert_eq!(maximum_value(&all_writes_covered), pinned.all_writes_covered_maximum, "{}", pinned.label);
            assert_eq!(recorded_covered.iter().copied().min().expect("非空"), 0, "{}", pinned.label);
        }
    }

    #[test]
    fn the_serial_stream_is_one_of_the_free_interleavings_and_its_state_count_is_fifteen() {
        let evaluation = evaluation_of("g1_free");
        let position = evaluation.serial_stream_position.expect("串行流是自由交错里的一条");
        assert_eq!(evaluation.streams[position].state_count, 15);
    }

    // ── L1：两条算法逐状态相同 ─────────────────────────────────────────────────────

    #[test]
    fn truth_by_enumeration_and_truth_by_search_agree_state_by_state_on_every_one_disk_cell() {
        for label in ["g1_serial", "g1_one_publish", "g1_free", "g1_ordered", "g3_serial", "g3_one_publish", "g3_free", "g3_ordered", "g5_serial", "g5_one_publish", "g5_free", "g5_ordered"] {
            let evaluation = evaluation_of(label);
            assert_eq!(evaluation.truth, evaluation.truth_by_search, "{label}");
        }
    }

    #[test]
    fn the_search_finds_no_stream_for_a_state_that_no_stream_reaches() {
        let evaluation = evaluation_of("g1_serial");
        let table = &evaluation.world.table;
        let unreachable = state_with_only(table, &["b1"]);
        assert!(!contains(&evaluation.truth, unreachable));
        assert!(!some_stream_realizes_target(&evaluation.world, unreachable));
        let free = evaluation_of("g1_free");
        assert!(some_stream_realizes_target(&free.world, unreachable));
    }

    #[test]
    fn the_ordered_candidate_has_the_independently_counted_number_of_streams() {
        let world = build_world(Candidate::ManyPublishesInFlightOrdered, &geometry_g1());
        assert_eq!(world.streams.len(), 3747);
        let free = build_world(Candidate::ManyPublishesInFlightFree, &geometry_g1());
        assert_eq!(free.streams.len(), 12870);
    }

    // ── 真实基线（V1）──────────────────────────────────────────────────────────────

    #[test]
    fn real_baseline_serial_pipeline_arm_by_root_publish_equals_truth() {
        let evaluation = evaluation_of("g1_serial");
        assert_eq!(evaluation.streams[0].cross_misses + evaluation.streams[0].non_cross_misses, 0);
        assert_eq!(evaluation.streams[0].state_count, evaluation.truth.len());
        let one_publish = evaluation_of("g1_one_publish");
        assert_eq!(one_publish.streams[0].cross_misses + one_publish.streams[0].non_cross_misses, 0);
        assert_eq!(one_publish.streams[0].state_count, one_publish.truth.len());
    }

    // ── 按事务切的臂（K2）与恒等式 ───────────────────────────────────────────────────

    #[test]
    fn arm_by_transaction_on_one_disk_never_misses_a_cross_transaction_state() {
        for label in ["g1_serial", "g1_one_publish", "g1_free", "g1_ordered", "g3_serial", "g3_one_publish", "g3_free", "g3_ordered", "g5_serial", "g5_one_publish", "g5_free", "g5_ordered"] {
            let evaluation = evaluation_of(label);
            assert_eq!(count_cross_transaction_misses(&evaluation.world.table, &evaluation.truth, &evaluation.arm_by_transaction), 0, "{label}");
        }
    }

    #[test]
    fn arm_by_transaction_state_count_is_the_product_of_the_two_chains() {
        let evaluation = evaluation_of("g1_free");
        let chain_a = crash_states_of_stream(&evaluation.world.table, &evaluation.world.arm_chains[0]);
        let chain_b = crash_states_of_stream(&evaluation.world.table, &evaluation.world.arm_chains[1]);
        assert_eq!(evaluation.arm_by_transaction.len(), chain_a.len() * chain_b.len());
        assert_eq!(chain_a.len() * chain_b.len(), 64);
    }

    #[test]
    fn arm_by_transaction_identity_holds_additively() {
        for label in ["g1_ordered", "g3_ordered"] {
            let evaluation = evaluation_of(label);
            let table = &evaluation.world.table;
            let extra = difference_count(&evaluation.arm_by_transaction, &evaluation.truth);
            let cross_misses = count_cross_transaction_misses(table, &evaluation.truth, &evaluation.arm_by_transaction);
            let non_cross_misses = count_non_cross_misses(table, &evaluation.truth, &evaluation.arm_by_transaction);
            assert_eq!(evaluation.arm_by_transaction.len() + cross_misses + non_cross_misses, evaluation.truth.len() + extra, "{label}");
        }
    }

    // ── L10：精确可达集 ─────────────────────────────────────────────────────────────

    #[test]
    fn exact_reachable_states_of_the_serial_stream_equal_its_crash_states() {
        let evaluation = evaluation_of("g1_serial");
        assert_eq!(evaluation.exact_reachable.len(), 15);
        assert_eq!(difference_count(&evaluation.exact_reachable, &evaluation.truth), 0);
        assert_eq!(difference_count(&evaluation.truth, &evaluation.exact_reachable), 0);
    }

    #[test]
    fn exact_reachable_states_are_pinned_on_the_two_in_flight_cells() {
        for label in ["g1_free", "g1_ordered", "g3_free", "g3_ordered", "g1_one_publish", "g3_one_publish", "g5_free", "g5_ordered"] {
            let evaluation = evaluation_of(label);
            assert_eq!(difference_count(&evaluation.exact_reachable, &evaluation.truth), 0, "{label}");
            assert_eq!(difference_count(&evaluation.truth, &evaluation.exact_reachable), 0, "{label}");
        }
    }

    // ── 阳性对照 ───────────────────────────────────────────────────────────────────

    #[test]
    fn serial_concatenation_misses_the_state_where_only_the_first_unit_of_b_is_persisted_on_both_arms() {
        let evaluation = evaluation_of("g1_free");
        let table = &evaluation.world.table;
        let witness = state_with_only(table, &["b1"]);
        assert!(contains(&evaluation.truth, witness));
        let serial = evaluation.world.serial_stream.as_ref().expect("串行流");
        assert!(!contains(&crash_states_of_stream(table, serial), witness));
        assert!(!contains(&serial_concatenation_of_transaction_chains(&evaluation.world), witness));
        assert!(difference_count(&evaluation.truth, &crash_states_of_stream(table, serial)) >= 1);
        assert_eq!(difference_count(&evaluation.truth, &crash_states_of_stream(table, serial)) + 15, 64);
    }

    #[test]
    fn cross_transaction_miss_counter_counts_only_the_cross_states_of_the_serial_concatenation() {
        let evaluation = evaluation_of("g1_free");
        let table = &evaluation.world.table;
        let serial = evaluation.world.serial_stream.as_ref().expect("串行流");
        let serial_states = crash_states_of_stream(table, serial);
        let cross_misses = count_cross_transaction_misses(table, &evaluation.truth, &serial_states);
        let non_cross_misses = count_non_cross_misses(table, &evaluation.truth, &serial_states);
        assert_eq!(cross_misses + non_cross_misses, difference_count(&evaluation.truth, &serial_states));
        assert!(non_cross_misses > 0, "单独 B 有写持久的状态不在 C 里，计数器拿 T 去减会多算");
        assert!(cross_misses < difference_count(&evaluation.truth, &serial_states));
    }

    #[test]
    fn deleting_one_state_from_either_arm_raises_its_miss_count_by_exactly_one() {
        let evaluation = evaluation_of("g1_free");
        let table = &evaluation.world.table;
        let two_units = state_with_only(table, &["a1", "b1"]);
        let stream = evaluation.world.streams.iter().find(|stream| contains(&crash_states_of_stream(table, stream), two_units)).expect("有一条流让 a1、b1 同在第一段");
        let states = crash_states_of_stream(table, stream);
        assert_eq!(
            count_cross_transaction_misses(table, &evaluation.truth, &without_state(&states, two_units)),
            count_cross_transaction_misses(table, &evaluation.truth, &states) + 1
        );
        assert_eq!(count_cross_transaction_misses(table, &evaluation.truth, &without_state(&evaluation.arm_by_transaction, two_units)), 1);
    }

    #[test]
    fn deleting_one_invalid_prefix_lowers_the_covered_count_by_exactly_one_on_the_ordered_cell() {
        let evaluation = evaluation_of("g1_ordered");
        let table = &evaluation.world.table;
        let witness = recorded_prefix_witness(table);
        assert!(contains(&evaluation.truth, witness));
        assert!(is_invalid_prefix_by_record(table, witness));
        let covered = count_invalid_prefixes_covered(table, &evaluation.truth, &evaluation.arm_by_transaction, InvalidPrefixKind::ByRecord);
        let covered_after = count_invalid_prefixes_covered(table, &evaluation.truth, &without_state(&evaluation.arm_by_transaction, witness), InvalidPrefixKind::ByRecord);
        assert_eq!(covered, covered_after + 1);
        assert_eq!(covered, 1);
    }

    #[test]
    fn invalid_prefix_by_record_is_directional_between_a_and_b() {
        let evaluation = evaluation_of("g1_ordered");
        let table = &evaluation.world.table;
        let b_record_only = state_with_only(table, &["rB"]);
        let a_record_only = state_with_only(table, &["rA"]);
        assert!(is_invalid_prefix_by_record(table, b_record_only));
        assert!(!is_invalid_prefix_by_record(table, a_record_only));
        assert_eq!(count_invalid_prefixes_in(table, &evaluation.truth, InvalidPrefixKind::ByRecord), 1);
    }

    // ── L7a：罩住段串 ───────────────────────────────────────────────────────────────

    #[test]
    fn the_cover_of_the_arm_by_transaction_on_one_disk_is_all_writes_in_one_segment() {
        let evaluation = evaluation_of("g1_free");
        let table = &evaluation.world.table;
        let cover = cover_by_merging(table, &evaluation.arm_by_transaction);
        assert_eq!(cover.segments.len(), 1);
        assert_eq!(cover.segments[0].len(), 10);
        assert_eq!(cover.cover_state_count, 2304);
        assert!(cover.covers_target);
        assert!(!cover.equals_target);
    }

    #[test]
    fn merging_agrees_with_exhaustive_ordered_partitions_on_the_u1_cells() {
        for label in ["g3_serial", "g3_one_publish", "g3_free", "g3_ordered"] {
            let evaluation = evaluation_of(label);
            let table = &evaluation.world.table;
            for target in [&evaluation.truth, &evaluation.arm_by_transaction] {
                let merged = cover_by_merging(table, target);
                let exhaustive = cover_by_exhaustive_ordered_partitions(table, target);
                assert_eq!(merged.cover_state_count, exhaustive.minimal_state_count, "{label}");
                assert!(exhaustive.optimal_covers.contains(&merged.segments), "{label}");
                assert!(merged.covers_target, "{label}");
            }
        }
    }

    #[test]
    fn a_state_set_that_is_exactly_one_stream_has_an_exactly_equal_cover() {
        let evaluation = evaluation_of("g1_serial");
        let cover = cover_by_merging(&evaluation.world.table, &evaluation.truth);
        assert!(cover.equals_target);
        assert_eq!(cover.cover_state_count, 15);
        assert_eq!(cover.segments.len(), 8);
    }

    #[test]
    fn membership_test_agrees_with_generated_crash_states_on_every_vector_of_a_small_stream() {
        let evaluation = evaluation_of("g3_free");
        let table = &evaluation.world.table;
        let stream = &evaluation.world.streams[0];
        let segments = write_segments(table, stream);
        let generated = crash_states_of_segments(table, &segments);
        for vector in all_state_vectors(table) {
            assert_eq!(state_is_in_crash_states_of_segments(&segments, vector), contains(&generated, vector));
        }
    }

    // ── 枚举器自身 ─────────────────────────────────────────────────────────────────

    #[test]
    fn doubled_barriers_on_one_disk_do_not_change_the_crash_state_set() {
        let world = build_world(Candidate::SerialPipeline, &geometry_g1());
        let serial = world.serial_stream.as_ref().expect("串行流");
        let mut doubled = Vec::new();
        for event in serial {
            doubled.push(*event);
            if matches!(event, Event::Barrier(_)) {
                doubled.push(*event);
            }
        }
        assert_ne!(doubled.len(), serial.len());
        assert_eq!(crash_states_of_stream(&world.table, &doubled), crash_states_of_stream(&world.table, serial));
    }

    #[test]
    fn a_torn_write_appears_only_in_the_current_segment_never_before_it() {
        let world = build_world(Candidate::SerialPipeline, &geometry_g1());
        let serial = world.serial_stream.as_ref().expect("串行流");
        let states = crash_states_of_stream(&world.table, serial);
        let torn_a = index_of_write(&world.table, "SA");
        let first_of_b = index_of_write(&world.table, "b1");
        for state in states {
            if value_of(state, torn_a) == TORN {
                assert_eq!(value_of(state, first_of_b), NOT_PERSISTED);
            }
        }
    }

    #[test]
    fn force_unit_access_closes_its_own_segment_and_does_not_make_earlier_writes_durable() {
        let table = vec![
            WriteSpec { name: "plain".to_string(), disk: 0, kind: StepKind::UnitWrite, owner: Owner::Shared, is_in_place: false },
            WriteSpec { name: "fua".to_string(), disk: 0, kind: StepKind::RootRecordFua, owner: Owner::Shared, is_in_place: false },
            WriteSpec { name: "after".to_string(), disk: 0, kind: StepKind::UnitWrite, owner: Owner::Shared, is_in_place: false },
        ];
        let stream = vec![Event::Write(0), Event::Write(1), Event::Write(2), Event::Barrier(0)];
        assert_eq!(write_segments(&table, &stream), vec![vec![0, 1], vec![2]]);
    }

    #[test]
    fn a_barrier_on_one_disk_does_not_close_a_segment_with_unreleased_writes_on_the_other() {
        let table = vec![
            WriteSpec { name: "d0".to_string(), disk: 0, kind: StepKind::UnitWrite, owner: Owner::Shared, is_in_place: false },
            WriteSpec { name: "d1".to_string(), disk: 1, kind: StepKind::UnitWrite, owner: Owner::Shared, is_in_place: false },
            WriteSpec { name: "later".to_string(), disk: 0, kind: StepKind::UnitWrite, owner: Owner::Shared, is_in_place: false },
        ];
        let stream = vec![Event::Write(0), Event::Write(1), Event::Barrier(0), Event::Write(2), Event::Barrier(1), Event::Barrier(0)];
        assert_eq!(write_segments(&table, &stream), vec![vec![0, 1, 2]]);
    }

    // ── 两盘几何的区域归属（M10）──────────────────────────────────────────────────────

    #[test]
    fn root_slots_of_the_two_disk_geometries_follow_the_three_region_assignment() {
        assert_eq!(geometry_g4a().root_disk_of_transaction_a, 0);
        assert_eq!(geometry_g4a().root_disk_of_transaction_b, 1);
        assert_eq!(geometry_g4b().root_disk_of_transaction_a, 0);
        assert_eq!(geometry_g4b().root_disk_of_transaction_b, 0);
        let world = build_world(Candidate::SerialPipeline, &geometry_g4a());
        let root_disk_of = |prefix: &str| {
            let roots: Vec<&WriteSpec> = world.table.iter().filter(|spec| spec.kind == StepKind::RootRecordFua && spec.name.starts_with(prefix)).collect();
            assert_eq!(roots.len(), 1);
            roots[0].disk
        };
        assert_eq!(root_disk_of("RB"), 1);
        assert_eq!(root_disk_of("RA"), 0);
    }

    // ── 判别力自证：挪门槛判定要翻 ────────────────────────────────────────────────────

    #[test]
    fn moving_the_threshold_to_the_maximum_turns_the_leak_judgement_from_leaking_to_never_leaking() {
        assert_eq!(leak_judgement(27, 42, 0), "leaks_on_every_stream");
        assert_eq!(leak_judgement(0, 42, 0), "leaks_on_some_streams");
        assert_eq!(leak_judgement(27, 42, 42), "never_leaks");
        assert_eq!(leak_judgement(0, 0, 0), "never_leaks");
        assert_eq!(invalid_prefix_judgement(13, 16), "misses_invalid_prefixes");
        assert_eq!(invalid_prefix_judgement(13, 13), "covers_every_invalid_prefix");
        assert_eq!(invalid_prefix_judgement(0, 0), "no_invalid_prefix_in_truth");
    }

    // ── 能力锚点与结构常量 ───────────────────────────────────────────────────────────

    #[test]
    fn capacity_anchors_of_the_real_publish_shape_match_the_registered_arithmetic() {
        let world = build_world(Candidate::SerialPipeline, &geometry_real_publish_shape());
        let every_write: Vec<usize> = (0..world.table.len()).collect();
        assert_eq!(world.table.len(), 2 * (24 + 2 + 1 + 2));
        assert_eq!(segment_expansion(&world.table, &every_write), 1_459_166_279_268_040_704);
        let chain_segments = write_segments(&world.table, &world.arm_chains[0]);
        assert_eq!(closed_form_state_count(&world.table, &chain_segments), 16_777_228);
        assert!(1_459_166_279_268_040_704u128 > MAXIMUM_SEGMENT_STATES_IN_U32);
        assert!(1_459_166_279_268_040_704u128 <= MAXIMUM_SEGMENT_STATES_IN_U64);
        assert_eq!(MAXIMUM_ENUMERATED_WRITES, 2048);
    }

    #[test]
    fn two_disk_cover_of_the_arm_by_transaction_is_all_eighteen_writes_in_one_segment() {
        let world = build_world(Candidate::SerialPipeline, &geometry_g4a());
        let product = arm_by_transaction_states(&world);
        let cover = cover_by_merging(&world.table, &product);
        assert_eq!(cover.segments.len(), 1);
        assert_eq!(cover.segments[0].len(), 18);
        assert_eq!(cover.cover_state_count, 1_327_104);
    }

    // ═══ 第二段：流式路径与两盘几何 ═══════════════════════════════════════════════════

    struct CollectingSink {
        leaves: Vec<(u128, u64, [u32; MAXIMUM_STREAMING_WRITES])>,
    }

    impl LeafSink for CollectingSink {
        fn leaf(&mut self, walk: &Walk, masks: &[u32; MAXIMUM_STREAMING_WRITES]) {
            self.leaves.push((walk.key, walk.path, *masks));
        }
    }

    /// 数叶子，并数其中满足三条先后约束的：每条约束是（A 的第几步，B 的第几步），要求 A 那一步先发出。
    struct CountingSink {
        leaves: u64,
        obeying: u64,
        pairs_of_chain_positions: Vec<(usize, usize)>,
        total_steps: usize,
    }

    fn order_pairs_of(table: &[WriteSpec]) -> Vec<(usize, usize)> {
        let find = |prefix: &str, disk_suffix: &str| index_of_write(table, &format!("{prefix}{disk_suffix}"));
        let mut pairs = Vec::new();
        let disks = table.iter().map(|spec| spec.disk).max().expect("写表非空") + 1;
        for disk in 0..disks {
            let suffix = if disks == 1 { String::new() } else { format!("@{disk}") };
            pairs.push((find("rA", &suffix), find("rB", &suffix)));
            pairs.push((find("SA", &suffix), find("SB", &suffix)));
        }
        let root_of = |tag: &str| table.iter().position(|spec| spec.kind == StepKind::RootRecordFua && spec.name.starts_with(&format!("R{tag}"))).expect("每个事务一次根槽写");
        pairs.push((root_of("A"), root_of("B")));
        pairs
    }

    fn obeys_ordering(pairs: &[(usize, usize)], events: &[Event]) -> bool {
        let position_of = |write: usize| events.iter().position(|event| *event == Event::Write(write)).expect("流里有这次写");
        pairs.iter().all(|&(first, second)| position_of(first) < position_of(second))
    }

    impl LeafSink for CountingSink {
        fn leaf(&mut self, walk: &Walk, _masks: &[u32; MAXIMUM_STREAMING_WRITES]) {
            self.leaves += 1;
            let (mut issue_step_of_a, mut issue_step_of_b) = ([0u8; 32], [0u8; 32]);
            let (mut position_a, mut position_b) = (0usize, 0usize);
            for step in 0..self.total_steps {
                if (walk.path >> (self.total_steps - 1 - step)) & 1 == 0 {
                    issue_step_of_a[position_a] = step as u8;
                    position_a += 1;
                } else {
                    issue_step_of_b[position_b] = step as u8;
                    position_b += 1;
                }
            }
            if self.pairs_of_chain_positions.iter().all(|&(of_a, of_b)| issue_step_of_a[of_a] < issue_step_of_b[of_b]) {
                self.obeying += 1;
            }
        }
    }

    /// 深搜前 `steps` 步之后的任务里每隔几个取一个，把这些任务下面的全部叶子收下来：两盘几何的抽查用（不进任何判据，只拿来逐条对拍流函数）。
    fn sampled_leaves(world: &World, tasks_wanted: usize) -> (TableInfo, Vec<(u128, u64, [u32; MAXIMUM_STREAMING_WRITES])>) {
        let info = TableInfo::new(&world.table);
        let context = WalkContext::new(world, &info);
        let tasks = split_tasks(&context, 18);
        let stride = (tasks.len() / tasks_wanted).max(1);
        let mut sink = CollectingSink { leaves: Vec::new() };
        for task in tasks.iter().step_by(stride) {
            let mut masks = task.masks;
            walk_interleavings(&context, task.walk, &mut masks, &mut sink);
        }
        (info, sink.leaves)
    }

    fn two_disk_one_unit_geometry() -> Geometry {
        geometry_two_disks("two_disk_u1", 1, 3)
    }

    // ── 交错数：登记 S3 的 C(30, 15)，与另一条数法逐格相同 ──

    #[test]
    fn interleaving_counts_equal_the_independent_python_counts() {
        // 数字来自 /tmp 里那份只按登记第一节读法写的 Python 小脚本（命令与输出在第二段报告里）。
        for (geometry, candidate, expected) in [
            (geometry_g1(), Candidate::ManyPublishesInFlightFree, 12_870u128),
            (geometry_g1(), Candidate::ManyPublishesInFlightOrdered, 3_747),
            (geometry_g3(), Candidate::ManyPublishesInFlightOrdered, 973),
            (geometry_g4a(), Candidate::ManyPublishesInFlightFree, 155_117_520),
            (geometry_g4a(), Candidate::ManyPublishesInFlightOrdered, 37_834_712),
            (geometry_g4b(), Candidate::ManyPublishesInFlightOrdered, 37_834_712),
            (two_disk_one_unit_geometry(), Candidate::ManyPublishesInFlightFree, 10_400_600),
            (two_disk_one_unit_geometry(), Candidate::ManyPublishesInFlightOrdered, 2_431_135),
        ] {
            let world = build_two_chain_world_without_streams(candidate, &geometry);
            let info = TableInfo::new(&world.table);
            assert_eq!(count_interleavings(&WalkContext::new(&world, &info)), expected, "{} {}", geometry.label, candidate.name());
        }
        assert_eq!(155_117_520u128, (1..=15u128).fold(1u128, |sum, step| sum * (15 + step) / step));
    }

    #[test]
    fn the_depth_first_walk_visits_exactly_the_counted_interleavings_and_the_ordered_ones_are_exactly_those_obeying_the_three_orders() {
        for (candidate, expected) in [(Candidate::ManyPublishesInFlightFree, 10_400_600u64), (Candidate::ManyPublishesInFlightOrdered, 2_431_135)] {
            let world = build_two_chain_world_without_streams(candidate, &two_disk_one_unit_geometry());
            let info = TableInfo::new(&world.table);
            let context = WalkContext::new(&world, &info);
            let pairs_of_chain_positions: Vec<(usize, usize)> = order_pairs_of(&world.table)
                .into_iter()
                .map(|(of_a, of_b)| {
                    (
                        world.search_chains[0].iter().position(|event| *event == Event::Write(of_a)).expect("A 的链里有这次写"),
                        world.search_chains[1].iter().position(|event| *event == Event::Write(of_b)).expect("B 的链里有这次写"),
                    )
                })
                .collect();
            let total_steps = world.search_chains[0].len() + world.search_chains[1].len();
            let mut sink = CountingSink { leaves: 0, obeying: 0, pairs_of_chain_positions, total_steps };
            walk_interleavings(&context, Walk::start(), &mut [0; MAXIMUM_STREAMING_WRITES], &mut sink);
            assert_eq!(sink.leaves, expected, "{}", candidate.name());
            assert_eq!(sink.obeying, if candidate == Candidate::ManyPublishesInFlightFree { 2_431_135 } else { expected }, "{}", candidate.name());
        }
    }

    #[test]
    fn ordered_streams_on_two_disks_put_the_root_of_a_before_the_root_of_b_whichever_disks_they_land_on() {
        let world = build_two_chain_world_without_streams(Candidate::ManyPublishesInFlightOrdered, &geometry_g4a());
        let (_, leaves) = sampled_leaves(&world, 100);
        assert!(leaves.len() > 500);
        let pairs = order_pairs_of(&world.table);
        assert_eq!(pairs.len(), 5);
        let root_of_a = world.table.iter().position(|spec| spec.name == "RA@0").expect("A 的根落盘 0");
        let root_of_b = world.table.iter().position(|spec| spec.name == "RB@1").expect("B 的根落盘 1");
        assert!(pairs.contains(&(root_of_a, root_of_b)));
        for (_, path, _) in leaves {
            assert!(obeys_ordering(&pairs, &events_of_path(&world, path)));
        }
    }

    // ── 流式路径与第一段的内存路径逐项相同 ──

    #[test]
    fn streaming_evaluation_equals_the_in_memory_evaluation_on_the_one_disk_cells() {
        for label in ["g1_serial", "g1_free", "g1_ordered", "g3_free", "g3_ordered", "g5_free", "g5_ordered"] {
            let spec = cell_specs().into_iter().find(|candidate| candidate.label == label).expect("这个格存在");
            let memory = evaluate_cell(&spec);
            let streaming = evaluate_streaming_cell(&spec, 3);
            assert_eq!(streaming.truth, memory.truth, "{label}");
            assert_eq!(streaming.truth_by_search, memory.truth_by_search, "{label}");
            assert_eq!(streaming.exact_reachable, memory.exact_reachable, "{label}");
            assert_eq!(streaming.arm_by_transaction, memory.arm_by_transaction, "{label}");
            assert_eq!(streaming.leaves as usize, memory.streams.len(), "{label}");
            assert_eq!(streaming.expected_leaves as usize, memory.streams.len(), "{label}");
            let table = &memory.world.table;
            assert_eq!(streaming.truth_cross, memory.truth.iter().filter(|state| is_cross_transaction(table, **state)).count(), "{label}");
            for kind in BOTH_INVALID_PREFIX_KINDS {
                let by_masks = match kind {
                    InvalidPrefixKind::ByRecord => streaming.truth_invalid_record,
                    InvalidPrefixKind::ByAllWrites => streaming.truth_invalid_all_writes,
                };
                assert_eq!(by_masks, count_invalid_prefixes_in(table, &memory.truth, kind), "{label}");
            }
            let weighted = |value_of_row: &dyn Fn(&SignatureRow) -> usize| -> Vec<(usize, u64)> {
                let mut by_value: std::collections::BTreeMap<usize, u64> = std::collections::BTreeMap::new();
                for row in &streaming.rows {
                    *by_value.entry(value_of_row(row)).or_insert(0) += row.count;
                }
                by_value.into_iter().collect()
            };
            let memory_distribution = |value_of_stream: &dyn Fn(&StreamMetrics) -> usize| -> Vec<(usize, u64)> {
                let mut by_value: std::collections::BTreeMap<usize, u64> = std::collections::BTreeMap::new();
                for metrics in &memory.streams {
                    *by_value.entry(value_of_stream(metrics)).or_insert(0) += 1;
                }
                by_value.into_iter().collect()
            };
            assert_eq!(weighted(&|row| streaming.truth_cross - row.cross_states as usize), memory_distribution(&|metrics| metrics.cross_misses), "{label} 跨事务漏数的分布");
            assert_eq!(
                weighted(&|row| (streaming.truth.len() - streaming.truth_cross) - (row.states - row.cross_states) as usize),
                memory_distribution(&|metrics| metrics.non_cross_misses),
                "{label} 跨事务之外漏数的分布"
            );
            assert_eq!(weighted(&|row| row.states as usize), memory_distribution(&|metrics| metrics.state_count), "{label} 状态数的分布");
            assert_eq!(weighted(&|row| row.invalid_record_states as usize), memory_distribution(&|metrics| metrics.recorded_prefixes_covered), "{label} 按记录覆盖数的分布");
            assert_eq!(weighted(&|row| row.invalid_all_writes_states as usize), memory_distribution(&|metrics| metrics.all_writes_prefixes_covered), "{label} 按全部写覆盖数的分布");
            assert_eq!(weighted(&|row| usize::from(row.largest_segment_writes)), memory_distribution(&|metrics| metrics.largest_segment_writes), "{label} 最大一段写数的分布");
            assert!(streaming.rows.iter().all(|row| row.closed_form_matches), "{label}");
        }
    }

    #[test]
    fn the_streaming_judgement_and_the_minimum_cross_miss_stream_equal_the_in_memory_ones() {
        for label in ["g1_free", "g1_ordered", "g3_free", "g5_ordered"] {
            let spec = cell_specs().into_iter().find(|candidate| candidate.label == label).expect("这个格存在");
            let memory = evaluate_cell(&spec);
            let streaming = evaluate_streaming_cell(&spec, 2);
            let mut memory_tally = RunTally::default();
            let mut streaming_tally = RunTally::default();
            let memory_base = emit_cell(&mut Emitter::new(), &mut memory_tally, &memory);
            let memory_judgement = second_judgement_of_in_memory(&memory, memory_base);
            let streaming_judgement = emit_streaming_cell(&mut Emitter::new(), &mut streaming_tally, &streaming);
            assert!(memory_tally.void_conditions.is_empty() && streaming_tally.void_conditions.is_empty(), "{label}");
            let (a, b) = (&memory_judgement, &streaming_judgement);
            assert_eq!(a.base.arm_by_root_publish_leak_judgement, b.base.arm_by_root_publish_leak_judgement, "{label}");
            assert_eq!(a.base.arm_by_root_publish_minimum_cross_misses, b.base.arm_by_root_publish_minimum_cross_misses, "{label}");
            assert_eq!(a.base.arm_by_root_publish_maximum_cross_misses, b.base.arm_by_root_publish_maximum_cross_misses, "{label}");
            assert_eq!((a.l3a, a.l3b, a.exact_not_in_truth, a.truth_not_in_exact), (b.l3a, b.l3b, b.exact_not_in_truth, b.truth_not_in_exact), "{label}");
            assert_eq!((a.base.recorded_truth_count, a.base.all_writes_truth_count), (b.base.recorded_truth_count, b.base.all_writes_truth_count), "{label}");
            assert_eq!(a.best_stream_recorded_covered, b.best_stream_recorded_covered, "{label}");
            assert_eq!(a.arm_recorded_covered, b.arm_recorded_covered, "{label}");
            assert_eq!((a.k1_largest_segment_writes, a.k1_largest_segment_expansion), (b.k1_largest_segment_writes, b.k1_largest_segment_expansion), "{label}");
            let cross_misses: Vec<usize> = memory.streams.iter().map(|metrics| metrics.cross_misses).collect();
            let (_, position) = minimum_with_position(&cross_misses);
            let minimum_row = streaming
                .rows
                .iter()
                .filter(|row| streaming.truth_cross - row.cross_states as usize == a.base.arm_by_root_publish_minimum_cross_misses)
                .min_by_key(|row| row.representative_path)
                .expect("有取到最小值的签名");
            assert_eq!(render_path(&streaming.world, minimum_row.representative_path), render_stream(&memory.world.table, &memory.world.streams[position]), "{label}");
        }
    }

    // ── 流式路径的零件逐个对拍 ──

    #[test]
    fn the_fixed_array_search_equals_the_set_based_search_on_every_one_disk_cell_and_on_the_two_disk_one_publish_cell() {
        for spec in cell_specs() {
            let world = build_world(spec.candidate, &spec.geometry);
            assert_eq!(truth_by_search_fast(&world), truth_by_search(&world), "{}", spec.label);
        }
        let world = build_world(Candidate::OnePublishManyTransactions, &geometry_g4a());
        assert_eq!(truth_by_search_fast(&world), truth_by_search(&world), "g4a_one_publish");
        let world = build_world(Candidate::SerialPipeline, &two_disk_one_unit_geometry());
        assert_eq!(truth_by_search_fast(&world), truth_by_search(&world), "two_disk_u1_serial");
    }

    #[test]
    fn index_and_code_agree_and_the_category_masks_equal_the_per_state_predicates() {
        for geometry in [geometry_g1(), geometry_g4a(), geometry_g4b()] {
            let world = build_world(Candidate::SerialPipeline, &geometry);
            let table = &world.table;
            let info = TableInfo::new(table);
            let mut seen: Vec<StateCode> = (0..info.total_vectors).map(|index| StateCode::from(info.code_of_index(index))).collect();
            seen.sort_unstable();
            assert_eq!(seen, all_state_vectors(table), "{}", geometry.label);
            for index in (0..info.total_vectors).step_by(7) {
                let code = info.code_of_index(index);
                let state = StateCode::from(code);
                assert_eq!(info.is_cross(code), is_cross_transaction(table, state), "{} {index}", geometry.label);
                assert_eq!(info.is_invalid_by_record(code), is_invalid_prefix_by_record(table, state), "{} {index}", geometry.label);
                assert_eq!(info.is_invalid_by_all_writes(code), is_invalid_prefix_by_all_writes(table, state), "{} {index}", geometry.label);
            }
        }
    }

    #[test]
    fn walk_signatures_and_order_masks_equal_the_stream_functions_on_sampled_two_disk_streams() {
        for (candidate, geometry) in [
            (Candidate::ManyPublishesInFlightFree, geometry_g4a()),
            (Candidate::ManyPublishesInFlightFree, geometry_g4b()),
            (Candidate::ManyPublishesInFlightOrdered, geometry_g4a()),
        ] {
            let world = build_two_chain_world_without_streams(candidate, &geometry);
            let (info, leaves) = sampled_leaves(&world, 200);
            assert!(leaves.len() > 800, "{}", geometry.label);
            for (key, path, direct) in leaves {
                let events = events_of_path(&world, path);
                let expected_segments: Vec<u32> = write_segments(&world.table, &events).iter().map(|segment| segment.iter().fold(0u32, |mask, &write| mask | (1u32 << write))).collect();
                assert_eq!(segments_of_signature(key, info.write_count), expected_segments, "{} {path}", geometry.label);
                let closed = close_masks(&direct, info.write_count);
                assert_eq!(&closed[..info.write_count], &must_persist_before_masks(&world.table, &events)[..], "{} {path}", geometry.label);
            }
        }
    }

    #[test]
    fn enumeration_by_signature_equals_the_stream_enumeration_state_by_state_with_consistent_indexes() {
        let world = build_two_chain_world_without_streams(Candidate::ManyPublishesInFlightFree, &geometry_g4a());
        let (info, leaves) = sampled_leaves(&world, 20);
        for (key, path, _) in leaves.into_iter().step_by(11).take(60) {
            let segments = segments_of_signature(key, info.write_count);
            let mut states: StateList = Vec::new();
            for_each_crash_state(&info, &segments, |code, index| {
                assert_eq!(info.code_of_index(index), code);
                states.push(StateCode::from(code));
            });
            assert_eq!(states.len() as u128, closed_form_of_segments(&info, &segments));
            states.sort_unstable();
            assert_eq!(states, crash_states_of_stream(&world.table, &events_of_path(&world, path)));
        }
    }

    #[test]
    fn the_recursive_down_set_enumeration_equals_the_subset_scan_on_two_disk_orders() {
        let free = build_two_chain_world_without_streams(Candidate::ManyPublishesInFlightFree, &geometry_g4a());
        let (info, leaves) = sampled_leaves(&free, 30);
        let mut checked = 0;
        for (_, path, _) in leaves.into_iter().step_by(37).take(25) {
            let events = events_of_path(&free, path);
            let masks = must_persist_before_masks(&free.table, &events);
            let mut direct = [0u32; MAXIMUM_STREAMING_WRITES];
            direct[..masks.len()].copy_from_slice(&masks);
            let mut states: StateList = Vec::new();
            for_each_exact_state(&info, &direct, &mut |index| states.push(StateCode::from(info.code_of_index(index))));
            states.sort_unstable();
            assert_eq!(states, sorted_deduplicated(reachable_states_of_partial_order(&free.table, &masks)), "{path}");
            checked += 1;
        }
        assert_eq!(checked, 25);
    }

    #[test]
    fn true_cross_misses_count_only_the_cross_states_that_are_both_in_the_truth_and_exactly_reachable() {
        let world = build_world(Candidate::SerialPipeline, &geometry_g1());
        let table = &world.table;
        let reachable = state_with_only(table, &["a1", "b1"]);
        let model_only = state_with_only(table, &["a1", "a2", "b1"]);
        let truth = sorted_deduplicated(vec![reachable, model_only]);
        let exact = vec![reachable];
        assert_eq!(count_cross_transaction_misses(table, &truth, &[]), 2);
        assert_eq!(true_cross_misses(table, &truth, &exact, &[]), 1);
        assert_eq!(true_cross_misses(table, &truth, &exact, &[reachable]), 0);
    }

    #[test]
    fn pass_one_keeps_the_smallest_path_and_the_full_count_of_every_signature_whatever_the_thread_split() {
        for geometry in [geometry_g1(), geometry_g3()] {
            let world = build_two_chain_world_without_streams(Candidate::ManyPublishesInFlightFree, &geometry);
            let info = TableInfo::new(&world.table);
            let context = WalkContext::new(&world, &info);
            let mut sink = CollectingSink { leaves: Vec::new() };
            walk_interleavings(&context, Walk::start(), &mut [0; MAXIMUM_STREAMING_WRITES], &mut sink);
            let mut expected: std::collections::BTreeMap<u128, (u64, u64)> = std::collections::BTreeMap::new();
            for (key, path, _) in &sink.leaves {
                let entry = expected.entry(*key).or_insert((0, u64::MAX));
                entry.0 += 1;
                entry.1 = entry.1.min(*path);
            }
            let expected: Vec<(u128, u64, u64)> = expected.into_iter().map(|(key, (count, path))| (key, count, path)).collect();
            for threads in [1, 4] {
                assert_eq!(pass_one(&context, threads).signatures, expected, "{} {threads}", geometry.label);
            }
        }
    }

    // ── L8：最大一段的动态规划 ──

    #[test]
    fn the_largest_segment_dynamic_programming_equals_the_maximum_over_every_signature_on_the_cells_with_a_brute_force_count() {
        for geometry in [geometry_g1(), geometry_g3(), two_disk_one_unit_geometry()] {
            let world = build_two_chain_world_without_streams(Candidate::ManyPublishesInFlightFree, &geometry);
            let info = TableInfo::new(&world.table);
            let context = WalkContext::new(&world, &info);
            let pass = pass_one(&context, 4);
            let mut most_expansion = 0u128;
            let mut most_writes = 0u32;
            for (key, _, _) in &pass.signatures {
                for segment in segments_of_signature(*key, info.write_count) {
                    most_expansion = most_expansion.max(expansion_of_segment(&info, segment));
                    most_writes = most_writes.max(segment.count_ones());
                }
            }
            let by_states = largest_segment_of(&world, SegmentObjective::Expansion);
            let by_writes = largest_segment_of(&world, SegmentObjective::Writes);
            assert_eq!(by_states.value, most_expansion, "{}", geometry.label);
            assert_eq!(by_writes.value, u128::from(most_writes), "{}", geometry.label);
            assert!(by_states.replay_agrees && by_writes.replay_agrees, "{}", geometry.label);
        }
    }

    #[test]
    fn the_largest_segment_witness_stream_on_one_disk_two_units_is_pinned() {
        // 一盘：任何屏障与 FUA 都放行唯一的盘，一段里的写只能是两条链在同一对屏障之间的写：手推最大是两个事务的两个单元写共四个
        // （a1 a2 b1 b2 都发在 A 的第一道屏障之前），别的段（记录、根与系统配置槽）至多两个。
        let world = build_two_chain_world_without_streams(Candidate::ManyPublishesInFlightFree, &geometry_g1());
        let by_writes = largest_segment_of(&world, SegmentObjective::Writes);
        assert_eq!(by_writes.writes_in_best_segment, 4);
        assert_eq!(by_writes.value, 4);
        let by_states = largest_segment_of(&world, SegmentObjective::Expansion);
        assert_eq!(by_states.states_in_best_segment, (1 << 4) - 1);
    }

    #[test]
    fn real_publish_shape_largest_k1_segment_is_at_least_the_registered_forty_eight_writes_and_the_witness_replays() {
        let world = build_two_chain_world_without_streams(Candidate::ManyPublishesInFlightFree, &geometry_real_publish_shape());
        let by_writes = largest_segment_of(&world, SegmentObjective::Writes);
        let by_states = largest_segment_of(&world, SegmentObjective::Expansion);
        assert!(by_writes.replay_agrees && by_states.replay_agrees);
        assert!(by_writes.writes_in_best_segment >= 48);
        assert!(by_states.states_in_best_segment >= 281_474_976_710_655);
        assert_eq!(by_states.states_in_best_segment + 1, by_states.value);
        // 登记第四节第 7 条算过的「全部写一段」：2^54 · 3^4 个组合、少去全零一个。
        assert_eq!(by_writes.writes_in_best_segment, 58);
        assert_eq!(by_states.states_in_best_segment, 1_459_166_279_268_040_703);
    }

    // ── 第三段补格：L11 ──

    #[test]
    fn the_true_cross_target_keeps_only_the_cross_states_in_both_the_truth_and_the_exact_set_by_hand() {
        let world = build_world(Candidate::SerialPipeline, &geometry_g3());
        let table = &world.table;
        let in_both = state_with_only(table, &["a1", "b1"]);
        let in_truth_only = state_with_only(table, &["a1", "rA", "b1"]);
        let in_exact_only = state_with_only(table, &["a1", "rA", "RA", "b1"]);
        let not_cross = state_with_only(table, &["a1"]);
        let truth = sorted_deduplicated(vec![in_both, in_truth_only, not_cross]);
        let exact = sorted_deduplicated(vec![in_both, in_exact_only, not_cross]);
        assert_eq!(true_cross_target(table, &truth, &exact), vec![u64::try_from(in_both).expect("装得进 u64")]);
    }

    #[test]
    fn membership_in_segment_masks_matches_the_hand_derived_cases_and_the_miss_counter_counts_deletions() {
        // 三次写，段串 [写 0 与写 1] [写 2]；两位一写：写 0 取 1 = 0b01，写 1 取 1 = 0b0100，写 2 取 1 = 0b010000。
        let masks = segment_digit_masks(&[0b011, 0b100]);
        let (w0, w1, w2) = (0b01u64, 0b01u64 << 2, 0b01u64 << 4);
        for code in [0, w0, w1, w0 | w1, w0 | w1 | w2, 0b10] {
            assert!(code_in_digit_masks(&masks, code), "{code:#b}");
        }
        for code in [w2, w0 | w2, w1 | w2, 0b10 | w2] {
            assert!(!code_in_digit_masks(&masks, code), "{code:#b}");
        }
        let target = [w0, w2, w0 | w1 | w2];
        // 手算：w0 在（当前段 0）、w2 不在（前一段没全持久）、全持久在（当前段 1）→ 漏 1；再从枚举域里删 w0 → 漏 2。
        assert_eq!(true_cross_misses_of_masks(&masks, &target, None), 1);
        assert_eq!(true_cross_misses_of_masks(&masks, &target, Some(w0)), 2);
    }

    #[test]
    fn membership_by_masks_equals_membership_by_table_and_by_enumeration_on_every_vector_of_sampled_one_disk_streams() {
        let spec = cell_specs().into_iter().find(|candidate| candidate.label == "g3_free").expect("这个格存在");
        let memory = evaluate_cell(&spec);
        let table = &memory.world.table;
        let info = TableInfo::new(table);
        for index in 0..info.total_vectors {
            assert_eq!(info.index_of_code(info.code_of_index(index)), index);
        }
        for stream in memory.world.streams.iter().step_by(53) {
            let segments = write_segments(table, stream);
            let masks: Vec<u32> = segments.iter().map(|segment| segment.iter().fold(0u32, |mask, &write| mask | (1u32 << write))).collect();
            let digit_masks = segment_digit_masks(&masks);
            let enumerated = crash_states_of_segments(table, &segments);
            for state in all_state_vectors(table) {
                let by_masks = code_in_digit_masks(&digit_masks, u64::try_from(state).expect("装得进 u64"));
                assert_eq!(by_masks, state_is_in_crash_states_of_segments(&segments, state));
                assert_eq!(by_masks, contains(&enumerated, state));
            }
        }
    }

    /// 两盘一单元（每个事务 7 次写、两条链各 13 个事件）的格：T_exact（184）远小于 T（82944），T_exact 过滤那一半在这里才敏感。
    /// 数与 `research/results/e167-multi-inflight-crash-cut-python-l11-two-disk-recount-2026-10-01.py`（独立 Python，只按定义写，原样输出是同名 `-a-ordered.out`、`-b-ordered.out`、`-a-free.out`）逐个相等；
    /// Python 的「段签名数」把段内写序也算进签名，与装置的签名（段内按集合）不同口径，所以不钉。
    fn check_two_disk_one_unit_cell(candidate: Candidate, txg_of_a: usize, expected: (u128, usize, usize, usize, usize, u32, u32, u64, usize)) {
        let geometry = geometry_two_disks("two_disk_u1", 1, txg_of_a);
        let spec = CellSpec { label: "two_disk_u1", sampling_point: "two_disk_u1", geometry, candidate };
        let streaming = evaluate_streaming_cell(&spec, 3);
        let read = read_k1_true_cross(&streaming, 3);
        assert!(read.methods_agree && read.serial_agrees_across_methods);
        let l2_maximum = streaming.rows.iter().map(|row| streaming.truth_cross - row.cross_states as usize).max().expect("至少一个段签名");
        let observed = (
            streaming.expected_leaves,
            streaming.truth.len(),
            streaming.exact_reachable.len(),
            streaming.truth_cross,
            read.target.len(),
            read.minimum,
            read.maximum,
            read.streams_with_misses,
            read.serial_misses,
        );
        assert_eq!(observed, expected);
        assert!(read.maximum as usize <= l2_maximum);
        let table = &streaming.world.table;
        for (row, &miss) in streaming.rows.iter().zip(&read.misses).step_by(97) {
            let segments: Vec<Vec<usize>> = segments_of_signature(row.key, streaming.info.write_count)
                .into_iter()
                .map(|mask| (0..streaming.info.write_count).filter(|write| mask & (1u32 << write) != 0).collect())
                .collect();
            let states = crash_states_of_segments(table, &segments);
            assert_eq!(miss as usize, read.target.iter().filter(|&&code| !contains(&states, StateCode::from(code))).count());
        }
    }

    #[test]
    fn l11_on_the_two_disk_one_unit_ordered_cell_equals_the_independent_python_recount() {
        // 根落同一块盘以外的一面（A 的 txg 取余 0：RA 落盘 0、RB 落盘 1）。
        // （交错数，|T|，|T_exact|，|T ∩ C|，|T ∩ T_exact ∩ C|，L11 最小，最大，有漏流数，σ_ser 上的值）
        check_two_disk_one_unit_cell(Candidate::ManyPublishesInFlightOrdered, 3, (2_431_135, 82_944, 184, 82_369, 165, 0, 150, 2_431_067, 150));
        // 两个根都落盘 0（A 的 txg 取余 2：RA、RB 都落盘 0）：有漏的流多一些（只有 34 条流一个都不漏）。
        check_two_disk_one_unit_cell(Candidate::ManyPublishesInFlightOrdered, 2, (2_431_135, 82_944, 184, 82_369, 165, 0, 150, 2_431_101, 150));
    }

    #[test]
    fn l11_on_the_two_disk_one_unit_free_cell_equals_the_independent_python_recount() {
        // 自由交错 10400600 条流，T_exact 256、T ∩ T_exact ∩ C 为 225，只有 103 条流一个都不漏。
        check_two_disk_one_unit_cell(Candidate::ManyPublishesInFlightFree, 3, (10_400_600, 82_944, 256, 82_369, 225, 0, 210, 10_400_497, 210));
    }

    /// 一盘四格的 L11：与 `research/results/e167-multi-inflight-crash-cut-python-l11-recount-2026-10-01.py` 的输出逐个相等（Python 只按定义算，不读这份装置）。
    /// 一盘上 T_exact = T，所以这几个数同时等于第一段 L2 的最小、最大（27 / 42、13 / 28、11 / 20、5 / 14）。
    #[test]
    fn l11_on_the_one_disk_cells_equals_the_independent_python_recount_and_the_in_memory_brute_force() {
        // （格，目标集大小，最小，最大，有漏的流数，串行流上的漏数）
        for (label, target_size, minimum, maximum, streams_with_misses, serial_misses) in [
            ("g1_free", 49usize, 27u32, 42u32, 12_870u64, 42usize),
            ("g1_ordered", 35, 13, 28, 3_747, 28),
            ("g3_free", 25, 11, 20, 3_432, 20),
            ("g3_ordered", 19, 5, 14, 973, 14),
        ] {
            let spec = cell_specs().into_iter().find(|candidate| candidate.label == label).expect("这个格存在");
            let streaming = evaluate_streaming_cell(&spec, 3);
            let read = read_k1_true_cross(&streaming, 3);
            assert!(read.methods_agree && read.serial_agrees_across_methods, "{label}");
            assert_eq!((read.target.len(), read.minimum, read.maximum, read.streams_with_misses, read.serial_misses), (target_size, minimum, maximum, streams_with_misses, serial_misses), "{label}");
            // 内存路径：逐条发出流枚举，数目标集里没落进它的状态。
            let memory = evaluate_cell(&spec);
            let table = &memory.world.table;
            let memory_target = true_cross_target(table, &memory.truth, &memory.exact_reachable);
            assert_eq!(memory_target, read.target, "{label}");
            let mut by_value: std::collections::BTreeMap<usize, u64> = std::collections::BTreeMap::new();
            let mut per_stream: Vec<usize> = Vec::new();
            for stream in &memory.world.streams {
                let states = crash_states_of_stream(table, stream);
                let misses = memory_target.iter().filter(|&&code| !contains(&states, StateCode::from(code))).count();
                *by_value.entry(misses).or_insert(0) += 1;
                per_stream.push(misses);
            }
            // 取到最小、最大的那条流：内存路径取字典序最先的，流式取代表路径最小的，两边渲染出来的流必须是同一条。
            let first_with = |value: usize| per_stream.iter().position(|&misses| misses == value).expect("有取到这个值的流");
            assert_eq!(render_path(&streaming.world, streaming.rows[read.minimum_row].representative_path), render_stream(table, &memory.world.streams[first_with(minimum as usize)]), "{label} 最小");
            assert_eq!(render_path(&streaming.world, streaming.rows[read.maximum_row].representative_path), render_stream(table, &memory.world.streams[first_with(maximum as usize)]), "{label} 最大");
            let mut streaming_by_value: std::collections::BTreeMap<usize, u64> = std::collections::BTreeMap::new();
            for (row, &miss) in streaming.rows.iter().zip(&read.misses) {
                *streaming_by_value.entry(miss as usize).or_insert(0) += row.count;
            }
            assert_eq!(by_value, streaming_by_value, "{label}");
        }
    }

    #[test]
    fn the_third_segment_cell_passes_its_own_self_checks_and_reports_the_pinned_judgement_on_the_one_disk_cells() {
        let mut judgements = Vec::new();
        let mut tally = RunTally::default();
        for label in ["g1_free", "g1_ordered", "g3_free", "g3_ordered"] {
            let spec = cell_specs().into_iter().find(|candidate| candidate.label == label).expect("这个格存在");
            let streaming = evaluate_streaming_cell(&spec, 2);
            judgements.push(emit_true_cross_cell(&mut Emitter::new(), &mut tally, &streaming, 2));
        }
        // 两盘一单元有序格：T_exact 比 T 小得多，L11 低于 L2（不高于 L2 那条自检在这里才不是同值）。
        let two_disk = CellSpec { label: "two_disk_u1", sampling_point: "two_disk_u1", geometry: two_disk_one_unit_geometry(), candidate: Candidate::ManyPublishesInFlightOrdered };
        judgements.push(emit_true_cross_cell(&mut Emitter::new(), &mut tally, &evaluate_streaming_cell(&two_disk, 2), 2));
        emit_true_cross_threshold_shifts(&mut Emitter::new(), &mut tally, &judgements);
        assert!(tally.void_conditions.is_empty(), "{:?}", tally.void_conditions);
        // 每格：两个阳性对照各 1、两条做法一致 1、串行流两种判法一致 1、不高于 L2 1、真值两算法 1、交错数 1、L11 门槛挪动 1。
        assert_eq!(tally.self_checks_passed, 5 * 8);
        let observed: Vec<(usize, usize, u64)> = judgements.iter().map(|judgement| (judgement.minimum, judgement.maximum, judgement.streams_with_misses)).collect();
        assert_eq!(observed, vec![(27, 42, 12_870), (13, 28, 3_747), (11, 20, 3_432), (5, 14, 973), (0, 150, 2_431_067)]);
        // 一盘上 K2 的积包住 T（第四节第 6 条），所以一盘四格的 L3b 全是 0。
        assert!(judgements[..4].iter().all(|judgement| judgement.arm_by_transaction_true_cross_misses == 0));
    }
}
