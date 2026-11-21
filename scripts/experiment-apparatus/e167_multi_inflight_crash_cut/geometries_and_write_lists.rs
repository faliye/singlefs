//! 写、事件与几何的定义，各几何的写表，按操作与按写切段，段的展开与段签名的崩溃状态。

use super::*;

pub(super) fn value_of(state: StateCode, write_index: usize) -> u8 {
    u8::try_from((state >> (BITS_PER_WRITE * write_index)) & 0b11).expect("两位取值装得进 u8")
}

pub(super) fn with_value(state: StateCode, write_index: usize, value: u8) -> StateCode {
    state | (StateCode::from(value) << (BITS_PER_WRITE * write_index))
}

pub(super) fn contains(states: &[StateCode], state: StateCode) -> bool {
    states.binary_search(&state).is_ok()
}

pub(super) fn sorted_deduplicated(mut states: Vec<StateCode>) -> StateList {
    states.sort_unstable();
    states.dedup();
    states
}

// ── 写、事件、几何 ─────────────────────────────────────────────────────────────────────

/// 步骤种类，声明次序就是登记表「种类」串里的规范次序（登记表 `.claude/kb/layout/01-first-txn.md` 第 403 行）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum StepKind {
    UnitWrite,
    JournalRecord,
    RootRecordFua,
    SystemConfigurationSlot,
    Barrier,
}

impl StepKind {
    pub(super) fn registered_name(self) -> &'static str {
        match self {
            StepKind::UnitWrite => "unit_write",
            StepKind::JournalRecord => "journal_record",
            StepKind::RootRecordFua => "root_record_fua",
            StepKind::SystemConfigurationSlot => "system_configuration_slot",
            StepKind::Barrier => "barrier",
        }
    }

    pub(super) fn from_registered_name(name: &str) -> StepKind {
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
pub(super) enum Owner {
    OlderTransaction,
    NewerTransaction,
    /// 一次发布装多个事务时，根槽写与系统配置槽写不属于任何一个事务。
    Shared,
}

#[derive(Clone, Debug)]
pub(super) struct WriteSpecification {
    pub(super) name: String,
    pub(super) disk: usize,
    pub(super) kind: StepKind,
    pub(super) owner: Owner,
    pub(super) is_in_place: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Event {
    Write(usize),
    Barrier(usize),
}

/// 一次崩溃枚举的几何：每个事务几个单元、几块盘、根槽写落哪块盘、系统配置槽写取不取第三态。
#[derive(Clone, Debug)]
pub(super) struct Geometry {
    pub(super) label: &'static str,
    pub(super) units_per_transaction: usize,
    pub(super) disk_count: usize,
    pub(super) root_disk_of_the_older_transaction: usize,
    pub(super) root_disk_of_the_newer_transaction: usize,
    pub(super) system_configuration_has_third_state: bool,
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
        root_disk_of_the_older_transaction: 0,
        root_disk_of_the_newer_transaction: 0,
        system_configuration_has_third_state: third_state,
    }
}

/// 两盘几何：A 的 txg 给定，B 的 txg 是它加一（登记第一节记号：A 是 N，B 是 N + 1）。
pub(super) fn geometry_two_disks(label: &'static str, units_per_transaction: usize, txg_of_the_older_transaction: usize) -> Geometry {
    Geometry {
        label,
        units_per_transaction,
        disk_count: 2,
        root_disk_of_the_older_transaction: root_disk_of_txg(txg_of_the_older_transaction),
        root_disk_of_the_newer_transaction: root_disk_of_txg(txg_of_the_older_transaction + 1),
        system_configuration_has_third_state: true,
    }
}

pub(super) fn geometry_one_disk_two_units() -> Geometry {
    geometry_one_disk("G1", 2, true)
}

pub(super) fn geometry_one_disk_one_unit() -> Geometry {
    geometry_one_disk("G3", 1, true)
}

pub(super) fn geometry_g4a() -> Geometry {
    geometry_two_disks("G4a", 2, 3)
}

pub(super) fn geometry_g4b() -> Geometry {
    geometry_two_disks("G4b", 2, 2)
}

pub(super) fn geometry_one_disk_two_units_without_torn_state() -> Geometry {
    geometry_one_disk("G5", 2, false)
}

/// 登记表普通发布那一行的形：每个事务 12 个单元、两盘、两态（`.claude/kb/layout/01-first-txn.md` 第 403 行）。
pub(super) fn geometry_registered_publish_two_state() -> Geometry {
    Geometry {
        label: "registered_publish",
        units_per_transaction: 12,
        disk_count: 2,
        root_disk_of_the_older_transaction: 0,
        root_disk_of_the_newer_transaction: 0,
        system_configuration_has_third_state: false,
    }
}

/// 真实发布形（登记第四节第 7 条）：每个事务 12 个单元、两盘、系统配置槽写取三态。
pub(super) fn geometry_real_publish_shape() -> Geometry {
    Geometry {
        label: "real_publish_shape",
        units_per_transaction: 12,
        disk_count: 2,
        root_disk_of_the_older_transaction: 0,
        root_disk_of_the_newer_transaction: 1,
        system_configuration_has_third_state: true,
    }
}

pub(super) fn disk_suffix(disk_count: usize, disk: usize) -> String {
    if disk_count == 1 {
        String::new()
    } else {
        format!("@{disk}")
    }
}

pub(super) fn push_write(
    table: &mut Vec<WriteSpecification>,
    geometry: &Geometry,
    name: &str,
    disk: usize,
    kind: StepKind,
    owner: Owner,
) -> usize {
    let is_in_place = kind == StepKind::SystemConfigurationSlot && geometry.system_configuration_has_third_state;
    table.push(WriteSpecification {
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
        Owner::OlderTransaction => ("a", "A"),
        Owner::NewerTransaction => ("b", "B"),
        Owner::Shared => unreachable!("共享的写不按事务起名"),
    }
}

pub(super) fn push_units(table: &mut Vec<WriteSpecification>, geometry: &Geometry, owner: Owner) -> Vec<usize> {
    let (lower, _) = lower_and_upper_tag(owner);
    let mut units = Vec::new();
    for unit_number in 1..=geometry.units_per_transaction {
        for disk in 0..geometry.disk_count {
            units.push(push_write(table, geometry, &format!("{lower}{unit_number}"), disk, StepKind::UnitWrite, owner));
        }
    }
    units
}

pub(super) fn push_records(table: &mut Vec<WriteSpecification>, geometry: &Geometry, owner: Owner) -> Vec<usize> {
    let (_, upper) = lower_and_upper_tag(owner);
    (0..geometry.disk_count)
        .map(|disk| push_write(table, geometry, &format!("r{upper}"), disk, StepKind::JournalRecord, owner))
        .collect()
}

pub(super) struct TransactionWrites {
    pub(super) units: Vec<usize>,
    pub(super) records: Vec<usize>,
    root: usize,
    system_slots: Vec<usize>,
}

pub(super) fn push_transaction_writes(table: &mut Vec<WriteSpecification>, geometry: &Geometry, owner: Owner, root_disk: usize) -> TransactionWrites {
    let (_, upper) = lower_and_upper_tag(owner);
    let units = push_units(table, geometry, owner);
    let records = push_records(table, geometry, owner);
    let root = push_write(table, geometry, &format!("R{upper}"), root_disk, StepKind::RootRecordFua, owner);
    let system_slots = (0..geometry.disk_count)
        .map(|disk| push_write(table, geometry, &format!("S{upper}"), disk, StepKind::SystemConfigurationSlot, owner))
        .collect();
    TransactionWrites { units, records, root, system_slots }
}

pub(super) fn writes_as_events(write_indexes: &[usize]) -> Vec<Event> {
    write_indexes.iter().map(|&write_index| Event::Write(write_index)).collect()
}

/// 一道池屏障 = 每块盘各一步屏障（登记第一节读法）。
pub(super) fn barrier_on_every_disk(disk_count: usize) -> Vec<Event> {
    (0..disk_count).map(Event::Barrier).collect()
}

/// D16（发布语义）已定项 7 的持久顺序：单元 → 屏障 → 记录 → 屏障 → 根槽（FUA）→ 系统配置槽 → 屏障。
pub(super) fn single_transaction_chain(writes: &TransactionWrites, disk_count: usize) -> Vec<Event> {
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
pub(super) fn operation_segments(table: &[WriteSpecification], stream: &[Event]) -> Vec<Vec<Event>> {
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

pub(super) fn write_segments(table: &[WriteSpecification], stream: &[Event]) -> Vec<Vec<usize>> {
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
pub(super) fn for_each_combination(table: &[WriteSpecification], write_indexes: &[usize], prefix: StateCode, mut emit: impl FnMut(StateCode)) {
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

pub(super) fn segment_expansion(table: &[WriteSpecification], segment: &[usize]) -> u128 {
    segment.iter().map(|&write_index| if table[write_index].is_in_place { 3u128 } else { 2u128 }).product()
}

/// D13（验证路线）已定项 4 的闭式：1 + Σ(2^两态写数 · 3^原地覆写数 − 1)。
pub(super) fn closed_form_state_count(table: &[WriteSpecification], segments: &[Vec<usize>]) -> u128 {
    1 + segments.iter().map(|segment| segment_expansion(table, segment) - 1).sum::<u128>()
}

pub(super) fn crash_states_of_segments(table: &[WriteSpecification], segments: &[Vec<usize>]) -> StateList {
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

pub(super) fn crash_states_of_stream(table: &[WriteSpecification], stream: &[Event]) -> StateList {
    crash_states_of_segments(table, &write_segments(table, stream))
}

/// 一个状态在不在一串段的枚举域里：直接按定义判（存在一个当前段，前面全 1、后面全 0），不生成枚举域。
pub(super) fn state_is_in_crash_states_of_segments(segments: &[Vec<usize>], state: StateCode) -> bool {
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
