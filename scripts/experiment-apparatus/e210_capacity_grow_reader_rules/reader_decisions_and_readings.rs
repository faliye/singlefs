//! 各臂与对照读者的判定、泄漏与误拒的判法、读数的种类与旗标、并行映射、逐臂的分析与对照读数。

use super::*;

/// 一条臂对（盘面，探测值）的决定：纯函数，只看有效槽的表与意图标记，不看槽号与世代号。盘面上至少有一个有效槽。
pub(super) fn arm_decision(arm: Arm, board: &Board, probe_sizes: Table) -> Decision {
    let valid_tables = valid_tables_of(board);
    let marked_tables = marked_tables_of(board);
    match arm {
        Arm::ChainThenLowest => {
            if !tables_form_a_chain(&valid_tables) {
                return WHOLE_POOL_REFUSED;
            }
            let table_in_force = componentwise_minimum(&valid_tables);
            standard_outcome(table_in_force, probe_sizes)
        }
        Arm::HighestTable => {
            let table_in_force = componentwise_maximum(&valid_tables);
            standard_outcome(table_in_force, probe_sizes)
        }
        Arm::OldTableWhileIntentMarkerIsOnTheDisk => {
            let table_in_force = if marked_tables.is_empty() {
                componentwise_maximum(&valid_tables)
            } else {
                componentwise_minimum(&marked_tables)
            };
            standard_outcome(table_in_force, probe_sizes)
        }
        Arm::LowestToReadHighestToProbeWithAbandon => {
            let lowest_table = componentwise_minimum(&valid_tables);
            let comparison_table = componentwise_maximum(&valid_tables);
            let outcome = standard_outcome(comparison_table, probe_sizes);
            let tables_differ = !all_tables_equal(&valid_tables);
            let probe_reaches_lowest_table =
                (0..DISK_COUNT).all(|disk| probe_sizes[disk] >= lowest_table[disk]);
            let may_abandon = tables_differ && probe_reaches_lowest_table;
            Decision {
                writable_mount_made: outcome.writable_mount_made || may_abandon,
                ..outcome
            }
        }
        Arm::OnlyIfEveryTableIsEqual => {
            if !all_tables_equal(&valid_tables) {
                return WHOLE_POOL_REFUSED;
            }
            standard_outcome(valid_tables[0], probe_sizes)
        }
    }
}

/// 对照读者的决定。按类判官臂读的是这条历史所在观测类的必点名集：非空则可写拒，空则做成，只读点名它。
fn control_decision(
    control: ControlReader,
    observation: &HistoryObservation,
    set: &HistorySet,
) -> Decision {
    match control {
        ControlReader::AllowEverything => Decision {
            writable_mount_made: true,
            read_only_mount_made: true,
            named_by_read_only: NOBODY_NAMED,
        },
        ControlReader::RejectEveryWritableMount => Decision {
            writable_mount_made: false,
            read_only_mount_made: true,
            named_by_read_only: NOBODY_NAMED,
        },
        ControlReader::NameWhatTheClassOverruns => {
            let necessarily_named = set.class_union_of(observation);
            Decision {
                writable_mount_made: necessarily_named == NOBODY_NAMED,
                read_only_mount_made: true,
                named_by_read_only: necessarily_named,
            }
        }
    }
}

pub(super) fn reader_decision(reader: Reader, observation: &HistoryObservation, set: &HistorySet) -> Decision {
    match reader {
        Reader::Arm(arm) => arm_decision(arm, &observation.board, observation.probe_sizes),
        Reader::Control(control) => control_decision(control, observation, set),
    }
}

/// 漏放行（可写）：可写挂载做成，而有单元的落点越过某块盘此刻的盘尾。
pub(super) fn is_leak_through_writable_mount(decision: Decision, beyond: DiskSet) -> bool {
    decision.writable_mount_made && beyond != NOBODY_NAMED
}

/// 漏放行（只读）：只读挂载做成，而越界的盘里有没被点名的。
pub(super) fn is_leak_through_read_only_mount(decision: Decision, beyond: DiskSet) -> bool {
    decision.read_only_mount_made
        && (0..DISK_COUNT).any(|disk| beyond[disk] && !decision.named_by_read_only[disk])
}

/// 误拒（岔路单字面读法）：可写挂载被拒，而没有任何落点越界。
pub(super) fn is_literal_misrefusal(decision: Decision, beyond: DiskSet) -> bool {
    !decision.writable_mount_made && beyond == NOBODY_NAMED
}

/// 误拒（可避免读法）：可写挂载被拒，而这条历史所在的观测类里没有任何历史越界（换一条读者规则本可以放行）。
pub(super) fn is_avoidable_misrefusal(decision: Decision, named_by_the_whole_class: DiskSet) -> bool {
    !decision.writable_mount_made && named_by_the_whole_class == NOBODY_NAMED
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum ReadingKind {
    LeakThroughWritableMount,
    LeakThroughReadOnlyMount,
    LiteralMisrefusal,
    AvoidableMisrefusal,
    WritableMountRefused,
}

const READING_KIND_COUNT: usize = 5;
pub(super) const ALL_READINGS: [ReadingKind; READING_KIND_COUNT] = [
    ReadingKind::LeakThroughWritableMount,
    ReadingKind::LeakThroughReadOnlyMount,
    ReadingKind::LiteralMisrefusal,
    ReadingKind::AvoidableMisrefusal,
    ReadingKind::WritableMountRefused,
];

pub(super) fn reading_name(reading: ReadingKind) -> &'static str {
    match reading {
        ReadingKind::LeakThroughWritableMount => "leak_writable",
        ReadingKind::LeakThroughReadOnlyMount => "leak_read_only",
        ReadingKind::LiteralMisrefusal => "misrefusal_literal",
        ReadingKind::AvoidableMisrefusal => "misrefusal_avoidable",
        ReadingKind::WritableMountRefused => "writable_refused",
    }
}

/// 一条历史上各个读数是否成立，次序同 `ALL_READINGS`。
fn reading_flags(
    decision: Decision,
    observation: &HistoryObservation,
    set: &HistorySet,
) -> [bool; READING_KIND_COUNT] {
    let own_beyond_the_end = beyond_the_end(observation.landing, observation.probe_sizes);
    let named_by_the_whole_class = set.class_union_of(observation);
    [
        is_leak_through_writable_mount(decision, own_beyond_the_end),
        is_leak_through_read_only_mount(decision, own_beyond_the_end),
        is_literal_misrefusal(decision, own_beyond_the_end),
        is_avoidable_misrefusal(decision, named_by_the_whole_class),
        !decision.writable_mount_made,
    ]
}

/// 一个读者在一条臂的历史集上的全部读数。
#[derive(Clone, PartialEq, Eq, Debug)]
pub(super) struct ReaderReadings {
    pub(super) counts: [usize; READING_KIND_COUNT],
    pub(super) counts_on_early_image_histories: [usize; READING_KIND_COUNT],
    /// 每个断电点序号上各读数的计数（点位「满」与「少」、全部形状与动作加在一起）。
    pub(super) trajectory: Vec<[usize; READING_KIND_COUNT]>,
    pub(super) classes: usize,
    pub(super) classes_with_differing_decisions: usize,
    pub(super) whole_pool_refusals: usize,
}

fn readings_of_reader(reader: Reader, set: &HistorySet) -> ReaderReadings {
    let mut counts = [0usize; READING_KIND_COUNT];
    let mut counts_on_early_image_histories = [0usize; READING_KIND_COUNT];
    let mut trajectory = vec![[0usize; READING_KIND_COUNT]; set.crash_point_count];
    let mut first_decision_of_class: BTreeMap<ClassKey, Decision> = BTreeMap::new();
    let mut classes_with_differing_decisions: BTreeSet<ClassKey> = BTreeSet::new();
    let mut whole_pool_refusals = 0;
    for observation in &set.observations {
        let decision = reader_decision(reader, observation, set);
        if decision == WHOLE_POOL_REFUSED {
            whole_pool_refusals += 1;
        }
        let key = class_key_of(observation);
        match first_decision_of_class.get(&key).copied() {
            None => {
                first_decision_of_class.insert(key, decision);
            }
            Some(first_decision) => {
                if first_decision != decision {
                    classes_with_differing_decisions.insert(key);
                }
            }
        }
        let flags = reading_flags(decision, observation, set);
        for reading in ALL_READINGS {
            let index = reading as usize;
            if flags[index] {
                counts[index] += 1;
                trajectory[observation.crash_point_index][index] += 1;
                if observation.is_early_image {
                    counts_on_early_image_histories[index] += 1;
                }
            }
        }
    }
    ReaderReadings {
        counts,
        counts_on_early_image_histories,
        trajectory,
        classes: first_decision_of_class.len(),
        classes_with_differing_decisions: classes_with_differing_decisions.len(),
        whole_pool_refusals,
    }
}

/// 互相独立的单位按次序切片并行：每个线程从共用的计数器领下一个单位，结果按单元次序合并，输出与线程数无关。
pub(super) fn parallel_map<Item: Sync, Output: Send>(
    items: &[Item],
    threads: usize,
    work: impl Fn(&Item) -> Output + Sync,
) -> Vec<Output> {
    let next_item = AtomicUsize::new(0);
    let thread_count = threads_used_for(items.len(), threads);
    let mut collected: Vec<(usize, Output)> = std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for _worker in 0..thread_count {
            handles.push(scope.spawn(|| {
                let mut finished = Vec::new();
                loop {
                    let index = next_item.fetch_add(1, Ordering::SeqCst);
                    if index >= items.len() {
                        break;
                    }
                    finished.push((index, work(&items[index])));
                }
                finished
            }));
        }
        let mut all_finished = Vec::new();
        for handle in handles {
            all_finished.extend(handle.join().expect("a worker thread panicked"));
        }
        all_finished
    });
    collected.sort_by_key(|(index, _)| *index);
    collected.into_iter().map(|(_, output)| output).collect()
}

/// 这一批单位实际用几个线程：配置的线程数与单位数取小，至少 1。
pub(super) fn threads_used_for(unit_count: usize, configured_threads: usize) -> usize {
    configured_threads.min(unit_count).max(1)
}
/// 一条臂的历史集加四个读者（臂自己与三条对照读者）的读数。
pub(super) struct ArmAnalysis {
    pub(super) arm: Arm,
    pub(super) set: HistorySet,
    pub(super) arm_readings: ReaderReadings,
    pub(super) control_readings: Vec<(ControlReader, ReaderReadings)>,
}

/// 并行单位：（臂下标，读者）。每条臂一个自己的读者单位，加三条对照读者各一个，对照读者循环遍历臂表里的每一条臂。
pub(super) fn reader_units() -> Vec<(usize, Reader)> {
    let mut units: Vec<(usize, Reader)> = ARMS
        .iter()
        .enumerate()
        .map(|(arm_index, arm)| (arm_index, Reader::Arm(*arm)))
        .collect();
    let arms_under_control: &[Arm] = &ARMS;
    for (arm_index, _arm) in arms_under_control.iter().enumerate() {
        for control in CONTROL_READERS {
            units.push((arm_index, Reader::Control(control)));
        }
    }
    units
}

/// 每块盘世代号最大的槽下标的四个组合：主格取两块盘都在槽 1，另三个只在不变性核对里跑。
pub(super) const NEWEST_SLOTS_OF_THE_MAIN_GRID: [usize; DISK_COUNT] = [1, 1];
pub(super) const NEWEST_SLOT_COMBINATIONS_BESIDES_THE_MAIN_GRID: [[usize; DISK_COUNT]; 3] =
    [[0, 0], [0, 1], [1, 0]];

pub(super) fn analyse_arms(
    newest_slot_of_each_disk: [usize; DISK_COUNT],
    numbering: Numbering,
    threads: usize,
) -> Vec<ArmAnalysis> {
    let sets = parallel_map(&ARMS, threads, |arm| {
        build_history_set(&write_plan_of(*arm), newest_slot_of_each_disk, numbering)
    });
    let units = reader_units();
    let readings = parallel_map(&units, threads, |unit| {
        readings_of_reader(unit.1, &sets[unit.0])
    });
    let mut analyses = Vec::new();
    for (arm_index, set) in sets.into_iter().enumerate() {
        let mut arm_readings = None;
        let mut control_readings = Vec::new();
        for (unit, unit_readings) in units.iter().zip(readings.iter()) {
            if unit.0 != arm_index {
                continue;
            }
            match unit.1 {
                Reader::Arm(_) => arm_readings = Some(unit_readings.clone()),
                Reader::Control(control) => control_readings.push((control, unit_readings.clone())),
            }
        }
        analyses.push(ArmAnalysis {
            arm: ARMS[arm_index],
            set,
            arm_readings: arm_readings.expect("every arm has its own reader unit"),
            control_readings,
        });
    }
    analyses
}

pub(super) fn analysis_of(analyses: &[ArmAnalysis], arm: Arm) -> &ArmAnalysis {
    analyses
        .iter()
        .find(|analysis| analysis.arm == arm)
        .expect("every arm in the arm table has an analysis")
}

pub(super) fn control_readings_of(analysis: &ArmAnalysis, control: ControlReader) -> &ReaderReadings {
    &analysis
        .control_readings
        .iter()
        .find(|(candidate, _)| *candidate == control)
        .expect("every arm has all three control readers")
        .1
}

/// 两份分析的全部读数（臂自己、三条对照读者，含轨迹与类计数）逐项相同。
pub(super) fn analyses_agree(left: &ArmAnalysis, right: &ArmAnalysis) -> bool {
    left.arm_readings == right.arm_readings && left.control_readings == right.control_readings
}

pub(super) fn zero_or_count(count: usize) -> String {
    if count == 0 {
        "zero".to_string()
    } else {
        count.to_string()
    }
}
