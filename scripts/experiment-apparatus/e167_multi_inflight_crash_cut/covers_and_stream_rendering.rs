//! 无效前缀与跨事务漏判的计数，合并与有序划分两种覆盖，发布归因，流与状态的文字渲染。

use super::*;

pub(super) const BOTH_INVALID_PREFIX_KINDS: [InvalidPrefixKind; 2] = [InvalidPrefixKind::ByRecord, InvalidPrefixKind::ByAllWrites];

/// |(T ∩ C) \ 臂|：跨事务落盘子集里这条臂漏掉的个数。
pub(super) fn count_cross_transaction_misses(table: &[WriteSpecification], truth: &[StateCode], covered: &[StateCode]) -> usize {
    truth.iter().filter(|state| is_cross_transaction(table, **state) && !contains(covered, **state)).count()
}

/// |(T \ C) \ 臂|：C 之外这条臂漏掉的个数。
pub(super) fn count_non_cross_misses(table: &[WriteSpecification], truth: &[StateCode], covered: &[StateCode]) -> usize {
    truth.iter().filter(|state| !is_cross_transaction(table, **state) && !contains(covered, **state)).count()
}

pub(super) fn count_invalid_prefixes_covered(table: &[WriteSpecification], truth: &[StateCode], covered: &[StateCode], kind: InvalidPrefixKind) -> usize {
    truth.iter().filter(|state| kind.holds(table, **state) && contains(covered, **state)).count()
}

pub(super) fn count_invalid_prefixes_in(table: &[WriteSpecification], states: &[StateCode], kind: InvalidPrefixKind) -> usize {
    states.iter().filter(|state| kind.holds(table, **state)).count()
}

pub(super) fn difference_count(left: &[StateCode], right: &[StateCode]) -> usize {
    left.iter().filter(|state| !contains(right, **state)).count()
}

// ── 罩住一个状态集的最小段串（L7a）──────────────────────────────────────────────────────────

#[derive(Debug)]
pub(super) struct CoverOutcome {
    pub(super) segments: Vec<Vec<usize>>,
    pub(super) covers_target: bool,
    pub(super) cover_state_count: u128,
    pub(super) equals_target: bool,
}

/// 合并法：目标集里任何一个状态里「持久或撕裂的写」必须落在「没持久或撕裂的写」所在段之前或同段，把必须同段的写合成一段，
/// 其余按这个偏序排成最细的段串；再按定义核一遍枚举域是否罩得住目标集。
pub(super) fn cover_by_merging(table: &[WriteSpecification], target: &[StateCode]) -> CoverOutcome {
    let write_count = table.len();
    let mut at_most = vec![vec![false; write_count]; write_count];
    for &state in target {
        for (earlier, at_most_row) in at_most.iter_mut().enumerate().take(write_count) {
            if value_of(state, earlier) == NOT_PERSISTED {
                continue;
            }
            for (later, at_most_cell) in at_most_row.iter_mut().enumerate().take(write_count) {
                if later != earlier && value_of(state, later) != PERSISTED {
                    *at_most_cell = true;
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
    let mut group_identifiers: Vec<usize> = group_of.clone();
    group_identifiers.sort_unstable();
    group_identifiers.dedup();
    let members_of = |group_identifier: usize| -> Vec<usize> { (0..write_count).filter(|&write_index| group_of[write_index] == group_identifier).collect() };
    let precedes = |earlier_group: usize, later_group: usize| -> bool {
        members_of(earlier_group).iter().any(|&earlier| members_of(later_group).iter().any(|&later| at_most[earlier][later])) && earlier_group != later_group
    };
    let mut ordered_groups = group_identifiers.clone();
    ordered_groups.sort_by_key(|group_identifier| (group_identifiers.iter().filter(|other| precedes(**other, *group_identifier)).count(), *group_identifier));
    let segments: Vec<Vec<usize>> = ordered_groups.into_iter().map(members_of).collect();
    summarize_cover(table, target, segments)
}

fn summarize_cover(table: &[WriteSpecification], target: &[StateCode], segments: Vec<Vec<usize>>) -> CoverOutcome {
    let covers_target = target.iter().all(|state| state_is_in_crash_states_of_segments(&segments, *state));
    let cover_state_count = closed_form_state_count(table, &segments);
    let equals_target = covers_target && cover_state_count == target.len() as u128;
    CoverOutcome { segments, covers_target, cover_state_count, equals_target }
}

pub(super) struct ExhaustiveCover {
    pub(super) minimal_state_count: u128,
    pub(super) optimal_covers: Vec<Vec<Vec<usize>>>,
}

/// 有序划分穷举：把写表里的写分成有序的段，取罩得住目标集、枚举域最小的那些段串。
pub(super) fn cover_by_exhaustive_ordered_partitions(table: &[WriteSpecification], target: &[StateCode]) -> ExhaustiveCover {
    assert!(table.len() <= 8, "有序划分穷举只给写数 ≤ 8 的几何");
    let mut best = ExhaustiveCover { minimal_state_count: u128::MAX, optimal_covers: Vec::new() };
    let mut current: Vec<Vec<usize>> = Vec::new();
    let all_writes: u32 = (1u32 << table.len()) - 1;
    extend_ordered_partitions(table, target, all_writes, &mut current, 1, &mut best);
    best
}

fn extend_ordered_partitions(table: &[WriteSpecification], target: &[StateCode], remaining: u32, current: &mut Vec<Vec<usize>>, partial_count: u128, best: &mut ExhaustiveCover) {
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
pub(super) struct PublishAttribution {
    pub(super) units: Vec<usize>,
    pub(super) records: Vec<usize>,
    pub(super) root: usize,
}

/// 一次根槽写之前、上一次根槽写之后的单元写与记录写都归它；系统配置槽写与屏障不归任何一次发布。
pub(super) fn publish_attribution(table: &[WriteSpecification], stream: &[Event]) -> Vec<PublishAttribution> {
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

pub(super) fn render_stream(table: &[WriteSpecification], stream: &[Event]) -> String {
    let disk_count = table.iter().map(|specification| specification.disk).max().map_or(1, |highest| highest + 1);
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

pub(super) fn render_state(table: &[WriteSpecification], state: StateCode) -> String {
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

pub(super) fn segment_sizes_text(table: &[WriteSpecification], stream: &[Event]) -> String {
    write_segments(table, stream).iter().map(|segment| segment.len().to_string()).collect::<Vec<String>>().join("+")
}

pub(super) fn segment_kinds_text(table: &[WriteSpecification], stream: &[Event]) -> String {
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
pub(super) fn stream_from_registered_kinds(text: &str, system_configuration_has_third_state: bool) -> (Vec<WriteSpecification>, Vec<Event>) {
    let geometry = Geometry {
        label: "registered_stream",
        units_per_transaction: 0,
        disk_count: 2,
        root_disk_of_the_older_transaction: 0,
        root_disk_of_the_newer_transaction: 0,
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

pub(super) struct StreamMetrics {
    pub(super) state_count: usize,
    pub(super) closed_form_state_count: u128,
    pub(super) cross_misses: usize,
    pub(super) non_cross_misses: usize,
    pub(super) recorded_prefixes_covered: usize,
    pub(super) all_writes_prefixes_covered: usize,
    pub(super) largest_segment_writes: usize,
    pub(super) largest_segment_expansion: u128,
}

pub(super) struct CellSpecification {
    pub(super) label: &'static str,
    pub(super) sampling_point: &'static str,
    pub(super) geometry: Geometry,
    pub(super) candidate: Candidate,
}

pub(super) fn cell_specifications() -> Vec<CellSpecification> {
    // G1 与 G2 是同一个几何（一盘、u = 2、系统配置槽写三态）上 P3 取 Σ_free 与 Σ_ord；G3 一盘 u = 1；G5 是 G1 而系统配置槽写只取两态。
    let groups: [(Geometry, [&'static str; 4], &'static str, &'static str); 3] = [
        (geometry_one_disk_two_units(), ["g1_serial", "g1_one_publish", "g1_free", "g1_ordered"], "G1", "G2"),
        (geometry_one_disk_one_unit(), ["g3_serial", "g3_one_publish", "g3_free", "g3_ordered"], "G3", "G3_ordered"),
        (geometry_one_disk_two_units_without_torn_state(), ["g5_serial", "g5_one_publish", "g5_free", "g5_ordered"], "G5", "G5_ordered"),
    ];
    let mut specifications = Vec::new();
    for (geometry, labels, free_point, ordered_point) in groups {
        let candidates_and_points = [
            (Candidate::SerialPipeline, free_point),
            (Candidate::OnePublishManyTransactions, free_point),
            (Candidate::ManyPublishesInFlightFree, free_point),
            (Candidate::ManyPublishesInFlightOrdered, ordered_point),
        ];
        for (label, (candidate, sampling_point)) in labels.into_iter().zip(candidates_and_points) {
            specifications.push(CellSpecification { label, sampling_point, geometry: geometry.clone(), candidate });
        }
    }
    specifications
}

pub(super) struct CellEvaluation {
    pub(super) label: &'static str,
    pub(super) sampling_point: &'static str,
    pub(super) world: World,
    pub(super) truth: StateList,
    pub(super) truth_by_search: StateList,
    pub(super) streams: Vec<StreamMetrics>,
    pub(super) serial_stream_position: Option<usize>,
    pub(super) arm_by_transaction: StateList,
    pub(super) exact_reachable: StateList,
}

pub(super) fn evaluate_cell(specification: &CellSpecification) -> CellEvaluation {
    evaluate_cell_with(specification, &truth_by_search)
}

/// 算法 (b) 由调用方给：第一段用逐状态的集合版，第二段的两盘格用定长数组版（`truth_by_search_fast`，单测与集合版逐格对拍）。
pub(super) fn evaluate_cell_with(specification: &CellSpecification, search: &dyn Fn(&World) -> StateList) -> CellEvaluation {
    let world = build_world(specification.candidate, &specification.geometry);
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
    CellEvaluation { label: specification.label, sampling_point: specification.sampling_point, world, truth, truth_by_search, streams, serial_stream_position, arm_by_transaction, exact_reachable }
}

pub(super) fn minimum_with_position(values: &[usize]) -> (usize, usize) {
    let mut best = (values[0], 0usize);
    for (position, value) in values.iter().enumerate() {
        if *value < best.0 {
            best = (*value, position);
        }
    }
    best
}

pub(super) fn maximum_value(values: &[usize]) -> usize {
    values.iter().copied().max().expect("至少一条发出流")
}

// ── 判据的判定函数（判别力自证要挪它的门槛）──────────────────────────────────────────────────

/// 漏数判定：漏数 > 门槛才算漏；每一条发出流上都漏、只在某些发出流上漏、一条都不漏。
pub(super) fn leak_judgement(minimum_misses: usize, maximum_misses: usize, threshold: usize) -> &'static str {
    if minimum_misses > threshold {
        "leaks_on_every_stream"
    } else if maximum_misses > threshold {
        "leaks_on_some_streams"
    } else {
        "never_leaks"
    }
}

/// 不合法前缀覆盖判定：覆盖数 < 要求的个数记「漏不合法前缀」。
pub(super) fn invalid_prefix_judgement(covered: usize, required: usize) -> &'static str {
    if required == 0 {
        "no_invalid_prefix_in_truth"
    } else if covered < required {
        "misses_invalid_prefixes"
    } else {
        "covers_every_invalid_prefix"
    }
}

// ── 输出 ───────────────────────────────────────────────────────────────────────────────

pub(super) fn emit(emitter: &mut Emitter, body: String) {
    println!("{}", emitter.emit_raw(&body));
}

pub(super) fn yes_no(condition: bool) -> &'static str {
    if condition {
        "yes"
    } else {
        "no"
    }
}
