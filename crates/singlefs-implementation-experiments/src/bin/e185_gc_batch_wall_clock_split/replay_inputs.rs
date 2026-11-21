use super::*;

// ───────────────────────────── 重放的输入（登记第五节「阶段拆分」：会话的取法另写一份） ─────────────────────────────

/// 一版里整理要读的几样，装置照 `mounted_session.rs` 的私有函数 `version_as_the_compaction_reads_it` 另写一份：每个进映射单元的映射条目
/// （按 key 在这一版的映射树里查，连同留存给快照的）、钉住的槽（映射树各节点、树表、实例表链）。取对没有，由停机条款第 4 条核
/// （取出的条目与会话那一批判成可搬的 key 对不上就停）。
pub(super) struct VersionAsTheReplayReadsIt {
    pub(super) allocation_records: Vec<AllocationRecord>,
    pub(super) mapping_entries: Vec<CentralMappingEntry>,
    pub(super) pinned_slots: BTreeSet<SlotNumber>,
}

pub(super) fn version_as_the_replay_reads_it(
    version: &TransactionOutput,
) -> VersionAsTheReplayReadsIt {
    let mut mapping_entries: Vec<CentralMappingEntry> = version
        .mapped_units
        .iter()
        .filter_map(
            |(_, key)| match central_mapping_locations_in_the_version(version, key) {
                MappingLookup::Found(locations) => Some(CentralMappingEntry {
                    key: key.clone(),
                    locations,
                }),
                MappingLookup::NodeMalformedOrNoEntryWithThisKey
                | MappingLookup::EntryNarrowerThanItsFieldTable { .. } => None,
            },
        )
        .collect();
    mapping_entries.extend(version.mapping_entries_kept_for_the_snapshots.iter().map(
        |(key, locations)| CentralMappingEntry {
            key: key.clone(),
            locations: *locations,
        },
    ));
    let mut pinned_slots = BTreeSet::new();
    for unit in &version.units {
        let is_exempt_from_the_mapping = match unit.identity {
            TransactionUnit::MappingTreeNodeBelowTheRoot(_)
            | TransactionUnit::MappingTree
            | TransactionUnit::TreeTable
            | TransactionUnit::InstanceTable
            | TransactionUnit::InstanceTablePageAfterTheFirst(_) => true,
            TransactionUnit::DataOfAFileOtherThanTheFirst { .. }
            | TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst { .. }
            | TransactionUnit::Data(_)
            | TransactionUnit::ExtentLowerNode(_)
            | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
            | TransactionUnit::ExtentRoot
            | TransactionUnit::InodeLeafContainer(_)
            | TransactionUnit::InodeRoot
            | TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
            | TransactionUnit::AllocationTree
            | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
            | TransactionUnit::AccountingTree
            | TransactionUnit::DeadlistTreeNodeBelowTheRoot(_)
            | TransactionUnit::DeadlistTree
            | TransactionUnit::DeletedInodesRoot
            | TransactionUnit::LoggedOperationsRoot
            | TransactionUnit::TombstoneLeafContainer(_)
            | TransactionUnit::TombstoneRoot
            | TransactionUnit::TombstoneNodeBelowTheRoot(_)
            | TransactionUnit::LineageLeafContainer(_)
            | TransactionUnit::LineageRoot
            | TransactionUnit::LineageNodeBelowTheRoot(_)
            | TransactionUnit::RetirementLeafContainer(_)
            | TransactionUnit::RetirementRoot
            | TransactionUnit::RetirementNodeBelowTheRoot(_) => false,
        };
        if is_exempt_from_the_mapping {
            for slot in unit.slot.0..unit.slot.0 + unit.identity.span_slots() {
                pinned_slots.insert(SlotNumber(slot));
            }
        }
    }
    VersionAsTheReplayReadsIt {
        allocation_records: version.allocation_records.clone(),
        mapping_entries,
        pinned_slots,
    }
}

/// 会话这一侧选段要的几样（调用之前那一刻的拷贝）：盘 0 的单元区、这次挂载开过的段、常驻隔离集、区域扫描拒过的段。
#[derive(Clone)]
pub(super) struct SessionStateForTheChoice {
    pub(super) unit_area: UnitAreaOfOneDevice,
    pub(super) cluster_segments: BTreeSet<SlotNumber>,
    pub(super) quarantined_slots: BTreeSet<SlotNumber>,
    pub(super) segments_refused_by_the_region_scan: SegmentsRefusedByTheRegionScan,
}

pub(super) fn session_state_for_the_choice(session: &MountedSession) -> SessionStateForTheChoice {
    let device_map = session
        .allocator
        .devices
        .first()
        .expect("池里至少一块盘：选段按第一块盘的记录做");
    SessionStateForTheChoice {
        unit_area: UnitAreaOfOneDevice {
            device: device_map.device,
            start: device_map.unit_area_start(),
            slots: device_map.unit_area_slots(),
        },
        cluster_segments: session.allocator.cluster_segments().clone(),
        quarantined_slots: session.compaction.quarantined_slots().clone(),
        segments_refused_by_the_region_scan: session
            .compaction
            .segments_refused_by_the_region_scan
            .clone(),
    }
}

/// 选段那一圈的重放（照会话签发那一步：`choose_region` 与「先扫」交替）：选中哪一段、每次 `choose_region` 调用时的输入（丙段计时照它重放）、
/// 先扫了哪几段。
pub(super) struct GreedyReplay {
    pub(super) chosen: Option<CompactionRegion>,
    pub(super) inputs_of_each_call: Vec<(Vec<RegionScan>, SegmentsRefusedByTheRegionScan)>,
    pub(super) scanned_regions: Vec<CompactionRegion>,
}

pub(super) fn replay_the_greedy_choice(
    version: &VersionAsTheReplayReadsIt,
    state: &SessionStateForTheChoice,
    mut read_unit: impl FnMut(DeviceIdentity, SlotNumber, usize) -> Option<Vec<u8>>,
) -> GreedyReplay {
    let slots_that_disqualify_a_segment: BTreeSet<SlotNumber> = version
        .pinned_slots
        .union(&state.quarantined_slots)
        .copied()
        .collect();
    let mut region_scans = region_scans_of_the_segments_without_an_unreleased_record(
        &version.allocation_records,
        state.unit_area,
    );
    let mut refused = state.segments_refused_by_the_region_scan.clone();
    let mut replay = GreedyReplay {
        chosen: None,
        inputs_of_each_call: Vec::new(),
        scanned_regions: Vec::new(),
    };
    // 迭代上界：每答一次「先扫」下一次就多一段有结论，不超过单元区的整段数（同会话那一圈）。
    for _call in 0..=state.unit_area.slots / LOCAL_CLUSTER_SEGMENT_SLOTS {
        replay
            .inputs_of_each_call
            .push((region_scans.clone(), refused.clone()));
        match choose_region(
            &version.allocation_records,
            state.unit_area,
            &state.cluster_segments,
            None,
            &slots_that_disqualify_a_segment,
            &region_scans,
            &refused,
            RegionChoiceCriterion::Greedy,
        ) {
            RegionChoice::Chosen(region) => {
                replay.chosen = Some(region);
                return replay;
            }
            RegionChoice::ScanBeforeChoosing(region) => {
                replay.scanned_regions.push(region);
                match scan_region(
                    region,
                    &version.allocation_records,
                    &version.mapping_entries,
                    &version.pinned_slots,
                    &state.quarantined_slots,
                    &mut read_unit,
                ) {
                    Ok(scan) => region_scans.push(scan),
                    Err(refusal) => refused.note_refusal(region, refusal),
                }
            }
            RegionChoice::NoEligibleSegment => return replay,
        }
    }
    replay
}

/// 一段的样子（几何核用）：R 那块盘上未释放记录罩住的活槽数、其中这次写的数据单元占的槽、别的槽（节点、打包单元、别人的数据）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SegmentComposition {
    pub(super) start_slot: u64,
    pub(super) live_slots: u64,
    pub(super) slots_of_this_workload: u64,
    pub(super) other_slots: u64,
}

pub(super) fn composition_of_the_segment(
    records: &[AllocationRecord],
    region: CompactionRegion,
    live_data_inode_by_slot: &BTreeMap<u64, u64>,
) -> SegmentComposition {
    let mut composition = SegmentComposition {
        start_slot: region.start_slot().0,
        live_slots: 0,
        slots_of_this_workload: 0,
        other_slots: 0,
    };
    for record in records.iter().filter(|record| {
        record.device == region.device() && !record.is_released && region.contains_slot(record.slot)
    }) {
        let span = u64::from(record.span_slots);
        composition.live_slots += span;
        if span * LOCAL_SLOT_BYTES == LOCAL_DATA_UNIT_BYTES
            && live_data_inode_by_slot.contains_key(&record.slot.0)
        {
            composition.slots_of_this_workload += span;
        } else {
            composition.other_slots += span;
        }
    }
    composition
}

/// 几何造成了：R 里全是这次写的码 1 数据单元、活槽 = 64f（登记第七节乙类的前提）。
pub(super) fn is_the_geometry_formed(
    composition: SegmentComposition,
    fraction: LiveSlotFraction,
) -> bool {
    composition.other_slots == 0 && composition.live_slots == fraction.live_slots_per_segment()
}
