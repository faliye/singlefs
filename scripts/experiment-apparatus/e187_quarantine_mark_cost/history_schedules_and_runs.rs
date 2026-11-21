//! 随机数、历史的日程与一段历史的整趟运行。

use super::*;

/// 确定性的伪随机源（SplitMix64）：同一格同一种子在三条臂上给同一串选择。
struct SplitMix64 {
    pub(super) state: u64,
}

impl SplitMix64 {
    fn seeded(cell: &HistoryCell, stream: u64) -> SplitMix64 {
        let geometry_code = match cell.geometry {
            PoolGeometry::DefaultFourGibibyteDevices => 1,
            PoolGeometry::WideOneTebibyteDevices => 2,
        };
        let mixed = [cell.density, cell.rollback_rate, u64::from(cell.file_size), cell.candidate_window, geometry_code, cell.seed, stream]
            .iter()
            .fold(0x9E37_79B9_7F4A_7C15u64, |accumulated, part| accumulated.rotate_left(17) ^ part.wrapping_mul(0xBF58_476D_1CE4_E5B9));
        SplitMix64 { state: mixed }
    }

    pub(super) fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut mixed = self.state;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        mixed ^ (mixed >> 31)
    }

    fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound
    }

    fn shuffle<Item>(&mut self, items: &mut [Item]) {
        for position in (1..items.len()).rev() {
            let other = usize_of(self.below(u64_of(position) + 1));
            items.swap(position, other);
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct ScheduledEvent {
    victim_position: usize,
    pub(super) copies: CopyCondition,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum ScheduledRollback {
    ToTheVersionBeforeTheEventAt(u64),
    ToTheOldestVersionInTheWindow,
}

/// 一段历史排的事件：哪几次普通发布隔离、隔离这次释放的第几个、副本状态；哪几次普通发布之后接一次回退、回退到哪。
pub(super) struct HistorySchedule {
    pub(super) events: BTreeMap<u64, ScheduledEvent>,
    pub(super) rollbacks: BTreeMap<u64, ScheduledRollback>,
}

fn interval_is_free(accepted: &BTreeMap<u64, u64>, start: u64, end: u64) -> bool {
    accepted.iter().all(|(accepted_start, accepted_end)| end < *accepted_start || start > *accepted_end)
}

pub(super) fn schedule_of(cell: &HistoryCell) -> HistorySchedule {
    let mut random = SplitMix64::seeded(cell, 1);
    let scheduled_quarantine_events = QUARANTINE_EVENTS_PER_DENSITY_STEP * cell.density;
    let mut event_publishes = BTreeSet::new();
    while u64_of(event_publishes.len()) < scheduled_quarantine_events {
        event_publishes.insert(1 + random.below(NORMAL_PUBLISHES_PER_HISTORY));
    }
    let ordered: Vec<u64> = event_publishes.iter().copied().collect();
    // 副本状态各半：按种子打乱之后前一半完好、后一半坏（不按一个拍出来的比例混）。
    let mut copy_order: Vec<usize> = (0..ordered.len()).collect();
    random.shuffle(&mut copy_order);
    let mut events = BTreeMap::new();
    for (rank, position) in copy_order.iter().enumerate() {
        let copies = if rank < ordered.len() / 2 { CopyCondition::Intact } else { CopyCondition::Garbled };
        events.insert(ordered[*position], ScheduledEvent { victim_position: 0, copies });
    }
    for event in events.values_mut() {
        event.victim_position = usize_of(random.below(u64_of(OVERWRITES_PER_PUBLISH)));
    }
    let mut rollbacks = BTreeMap::new();
    let scheduled_rollbacks = ROLLBACKS_AT_RATE_ONE * cell.rollback_rate;
    let mut accepted_intervals: BTreeMap<u64, u64> = BTreeMap::new();
    if cell.density > 0 {
        // 每次排在一次隔离事件之后 1 到 ⌊K ÷ 2⌋ 次发布里，目标取那次隔离之前的一版；几次回退的区间互不相交。
        let mut candidates = ordered.clone();
        random.shuffle(&mut candidates);
        let mut rejected_on_the_first_draw = Vec::new();
        for event_publish in candidates {
            if u64_of(rollbacks.len()) == scheduled_rollbacks {
                break;
            }
            let delay = 1 + random.below(cell.candidate_window / 2);
            let rollback_after = event_publish + delay;
            if rollback_after <= NORMAL_PUBLISHES_PER_HISTORY && interval_is_free(&accepted_intervals, event_publish, rollback_after) {
                accepted_intervals.insert(event_publish, rollback_after);
                rollbacks.insert(rollback_after, ScheduledRollback::ToTheVersionBeforeTheEventAt(event_publish));
            } else {
                rejected_on_the_first_draw.push(event_publish);
            }
        }
        // 第一遍每个事件只抽一次延迟，抽到的与已排的区间相交就跳过；不够登记的 10 次时，第二遍按同一个次序给跳过的事件
        // 在 1 到 ⌊K ÷ 2⌋ 里可行的延迟中按种子挑一个。第一遍就排满的格不进第二遍，排法与随机数的消耗都不变。
        for event_publish in rejected_on_the_first_draw {
            if u64_of(rollbacks.len()) == scheduled_rollbacks {
                break;
            }
            let feasible_delays: Vec<u64> = (1..=cell.candidate_window / 2)
                .filter(|delay| {
                    event_publish + delay <= NORMAL_PUBLISHES_PER_HISTORY && interval_is_free(&accepted_intervals, event_publish, event_publish + delay)
                })
                .collect();
            if feasible_delays.is_empty() {
                continue;
            }
            let rollback_after = event_publish + feasible_delays[usize_of(random.below(u64_of(feasible_delays.len())))];
            accepted_intervals.insert(event_publish, rollback_after);
            rollbacks.insert(rollback_after, ScheduledRollback::ToTheVersionBeforeTheEventAt(event_publish));
        }
    } else {
        // 密度 0：没有隔离可清，目标取窗口里最旧的一版，当对照。
        let mut attempts = 0u64;
        while u64_of(rollbacks.len()) < scheduled_rollbacks && attempts < NORMAL_PUBLISHES_PER_HISTORY {
            attempts += 1;
            let rollback_after = cell.candidate_window + 1 + random.below(NORMAL_PUBLISHES_PER_HISTORY - cell.candidate_window);
            if interval_is_free(&accepted_intervals, rollback_after - cell.candidate_window, rollback_after) {
                accepted_intervals.insert(rollback_after - cell.candidate_window, rollback_after);
                rollbacks.insert(rollback_after, ScheduledRollback::ToTheOldestVersionInTheWindow);
            }
        }
    }
    HistorySchedule { events, rollbacks }
}

/// 这一次覆盖写哪 4 个数据单元（按种子在 U 个里均匀挑、互不相同）。
fn choose_overwritten_data_units(random: &mut SplitMix64, file_size: u32) -> Vec<u32> {
    let mut chosen: Vec<u32> = Vec::with_capacity(OVERWRITES_PER_PUBLISH);
    while chosen.len() < OVERWRITES_PER_PUBLISH {
        let candidate = u32::try_from(random.below(u64::from(file_size))).expect("U 装得进 u32");
        if !chosen.contains(&candidate) {
            chosen.push(candidate);
        }
    }
    chosen
}

/// 一次隔离事件在一条臂上：那一次发布的写、`no_mark` 那一遍里被隔离单元所在叶这次本来重不重写、轨迹要的记录。
#[derive(Clone, Copy, Debug)]
pub(super) struct EventOutcome {
    normal_publish: u64,
    pub(super) writes: PublishWrites,
    pub(super) victim_leaf_rewritten: bool,
    pub(super) record: QuarantineEventRecord,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct RollbackOutcome {
    pub(super) writes: PublishWrites,
    pub(super) revived: u64,
}

/// 一条臂跑完一段历史（新建文件那一次不算进写量）。
pub(super) struct HistoryOutcome {
    pub(super) totals: PublishWrites,
    pub(super) events: Vec<EventOutcome>,
    pub(super) rollbacks: Vec<RollbackOutcome>,
    pub(super) final_txg: u64,
    pub(super) final_occupied_slots: u64,
    pub(super) final_deferred_slots: u64,
    pub(super) normal_units: u64,
    pub(super) placements_that_fell_back: u64,
    pub(super) mapping_tree_height: u8,
}

pub(super) fn run_history(cell: &HistoryCell, schedule: &HistorySchedule, arm: Arm) -> HistoryOutcome {
    let mut state = PoolState::after_make_filesystem(cell.geometry, cell.candidate_window, cell.rollback_rate > 0);
    let (_, mut placements_that_fell_back) = publish(&mut state, &PublishRequest::plain(PublishKind::CreateFile { data_unit_count: cell.file_size }), arm);
    let mut data_random = SplitMix64::seeded(cell, 2);
    let mut totals = PublishWrites::default();
    let mut events: Vec<EventOutcome> = Vec::new();
    let mut event_of_the_quarantined_unit: BTreeMap<UnitIdentifier, usize> = BTreeMap::new();
    let mut rollbacks = Vec::new();
    for normal_publish in 1..=NORMAL_PUBLISHES_PER_HISTORY {
        let data_indices = choose_overwritten_data_units(&mut data_random, cell.file_size);
        let scheduled = schedule.events.get(&normal_publish).copied();
        let quarantine_victim = scheduled.map(|event| data_indices[event.victim_position]);
        let victim_unit = quarantine_victim.map(|index| state.version.as_ref().expect("文件建好了").data_units[usize_of(u64::from(index))]);
        let request = PublishRequest::plain(PublishKind::Overwrite { data_indices, quarantine_victim, release_quarantined: None });
        let plan = plan_publish(&state, &request, arm);
        let writes = plan.writes(0);
        placements_that_fell_back += plan.placements_that_fell_back;
        let victim_leaf_rewritten = victim_unit.is_some_and(|unit| {
            plan.rewritten_allocation_positions.contains(&AllocationNodePosition { level: 0, device: 0, index_in_device: leaf_of_slot(state.unit(unit).slot) })
        });
        apply_publish(&mut state, plan);
        totals.add(&writes);
        if let (Some(event), Some(unit)) = (scheduled, victim_unit) {
            let slot = state.unit(unit).slot;
            if arm != Arm::PlainRelease {
                event_of_the_quarantined_unit.insert(unit, events.len());
            }
            events.push(EventOutcome {
                normal_publish,
                writes,
                victim_leaf_rewritten,
                record: QuarantineEventRecord { txg: state.txg, copies: event.copies, marks_during_quarantine: marks_of_the_quarantined_record(&state, arm, slot), revival: None },
            });
        }
        if let Some(rollback) = schedule.rollbacks.get(&normal_publish) {
            let target_txg = match rollback {
                ScheduledRollback::ToTheVersionBeforeTheEventAt(event_publish) => {
                    events.iter().find(|event| event.normal_publish == *event_publish).expect("回退排在它那次隔离之后").record.txg - 1
                }
                ScheduledRollback::ToTheOldestVersionInTheWindow => oldest_candidate_txg(state.txg, cell.candidate_window),
            };
            let rollback_plan = plan_publish(&state, &PublishRequest::plain(PublishKind::RollBack { target_txg }), arm);
            let rollback_writes = rollback_plan.writes(0);
            placements_that_fell_back += rollback_plan.placements_that_fell_back;
            let revived = rollback_plan.revived.clone();
            apply_publish(&mut state, rollback_plan);
            totals.add(&rollback_writes);
            for unit in &revived {
                let event_index = *event_of_the_quarantined_unit.get(unit).expect("复活的单元是一次隔离事件隔离的");
                let slot = state.unit(*unit).slot;
                events[event_index].record.revival = Some(RevivalRecord { txg: state.txg, marks_after_revival: marks_of_the_quarantined_record(&state, arm, slot) });
            }
            rollbacks.push(RollbackOutcome { writes: rollback_writes, revived: u64_of(revived.len()) });
        }
    }
    let unreleased_records = u64_of(state.records.values().filter(|record| !record.is_released).count());
    HistoryOutcome {
        totals,
        events,
        rollbacks,
        final_txg: state.txg,
        final_occupied_slots: state.occupied_slot_count(),
        final_deferred_slots: state.deferred_slot_count(),
        normal_units: unreleased_records - u64_of(state.quarantined.len()) - u64_of(MAKE_FILESYSTEM_RECORD_SLOTS.len()),
        placements_that_fell_back,
        mapping_tree_height: tree_height(&state.mapping),
    }
}

/// 一个事件在终态上属于哪一版：最后一个引用它的根已经离开候选窗口没有（登记第六节 `flip_classification_table` 的两版）。
pub(super) fn judged_version_at_the_end(event: &QuarantineEventRecord, final_txg: u64, candidate_window: u64) -> JudgedVersion {
    if event.txg > oldest_candidate_txg(final_txg, candidate_window) {
        JudgedVersion::TheQuarantineVersion
    } else {
        JudgedVersion::AfterTheLastReferencingRootLeavesTheWindow
    }
}
