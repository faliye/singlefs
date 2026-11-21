//! 历史的分析、分段行、唯一单元、键区间读数、写归属、覆盖与整个历史的判定。

use super::*;

// ---------------------------------------------------------------------------------------------
// 一段历史：逐段解码、带着现行那一版往下走。
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub(super) struct HistoryAnalysis {
    pub(super) name: String,
    pub(super) segments: Vec<SegmentAnalysis>,
    pub(super) states_before: Vec<VersionState>,
    pub(super) states_after: Vec<VersionState>,
}

fn analyze_history(history: &HistoryExport) -> HistoryAnalysis {
    let mut state = VersionState::default();
    let mut segments = Vec::new();
    let mut states_before = Vec::new();
    let mut states_after = Vec::new();
    let mut ordered_segments = history.segments.clone();
    ordered_segments.sort_by_key(|segment| segment.start_step);
    for segment in &ordered_segments {
        let analysis = analyze_segment(segment, &history.operations, &state);
        states_before.push(state.clone());
        state.absorb(&analysis);
        states_after.push(state.clone());
        segments.push(analysis);
    }
    HistoryAnalysis { name: history.name.clone(), segments, states_before, states_after }
}

/// S4：段首尾相接铺满窗口；窗口里每一步恰好出现一次；每次删除类发布根槽 FUA 恰 1 次。
fn tiling_failures(history: &HistoryExport, analysis: &HistoryAnalysis) -> Vec<String> {
    let mut failures = history.boundary_mismatches.clone();
    let window_operations = history.summary.get("window_operations").and_then(|text| text.parse::<usize>().ok()).unwrap_or(usize::MAX);
    let mut expected_start = 0usize;
    for segment in &analysis.segments {
        if segment.start_step != expected_start {
            failures.push(format!("段从 step {} 起，上一段止于 {expected_start}", segment.start_step));
        }
        expected_start = segment.end_step;
    }
    if expected_start != window_operations {
        failures.push(format!("最后一段止于 {expected_start}，窗口 {window_operations} 步"));
    }
    let steps: Vec<usize> = history.operations.iter().map(WindowOperation::step).collect();
    let steps_are_each_window_step_once = steps.len() == window_operations && steps.iter().enumerate().all(|(position, step)| position == *step);
    if !steps_are_each_window_step_once {
        failures.push(format!("窗口里的步号不是 0..{window_operations} 各一次（实有 {} 步）", steps.len()));
    }
    for segment in analysis.segments.iter().filter(|segment| segment.is_deletion_publish) {
        // 屏障按设备记：每一道两块盘各一步（D13 已定项 4；layout/01 第 411 行「三道池屏障按设备各记两步」）。
        let barrier_devices: Vec<u32> = history
            .operations
            .iter()
            .filter_map(|operation| match operation {
                WindowOperation::Barrier { step, device } if *step >= segment.start_step && *step < segment.end_step => Some(*device),
                WindowOperation::Write(_) | WindowOperation::Barrier { .. } => None,
            })
            .collect();
        let first_device_barriers = barrier_devices.iter().filter(|device| **device == 0).count();
        if first_device_barriers * 2 != barrier_devices.len() {
            failures.push(format!("txg {} 那次删除类发布的屏障不是两块盘各一步：{barrier_devices:?}", segment.label_txg()));
        }
        let force_unit_access_writes = segment.operations.iter().filter(|operation| operation.is_force_unit_access).count();
        if force_unit_access_writes != 1 || segment.root_records.len() != 1 {
            failures.push(format!("txg {} 那次删除类发布根槽 FUA {force_unit_access_writes} 次、根记录 {} 份", segment.label_txg(), segment.root_records.len()));
        }
    }
    failures
}

/// S3：解码器独立认出的角色名多重集合与导出 `rewritten_roles=` 比。
fn role_name_comparison(segment: &SegmentAnalysis) -> Option<(Vec<String>, Vec<String>, bool, bool)> {
    let SegmentLabel::Publish { rewritten_roles, .. } = &segment.label else { return None };
    let mut decoded = segment.role_names();
    decoded.sort();
    let mut exported = rewritten_roles.clone();
    exported.sort();
    let is_equal = decoded == exported;
    let strip_index = |name: &String| name.split(['@', '+']).next().unwrap_or_default().to_string();
    let mut decoded_prefixes: Vec<String> = decoded.iter().map(strip_index).collect();
    let mut exported_prefixes: Vec<String> = exported.iter().map(strip_index).collect();
    decoded_prefixes.sort();
    exported_prefixes.sort();
    let differs_only_in_index_notation = !is_equal && decoded_prefixes == exported_prefixes;
    Some((decoded, exported, is_equal, differs_only_in_index_notation))
}

// ---------------------------------------------------------------------------------------------
// 第 3 行：段串、种类串、单元数与角色名、根槽 FUA。
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub(super) struct SegmentRow {
    pub(super) history: String,
    pub(super) txg: u64,
    pub(super) publish_kind: String,
    pub(super) counts_for_row_three: bool,
    pub(super) compares_role_names_verbatim: bool,
    pub(super) expected_ordinals: String,
    pub(super) actual_ordinals: String,
    pub(super) expected_count_text: String,
    pub(super) actual_count_text: String,
    pub(super) expected_kind_text: String,
    pub(super) actual_kind_text: String,
    pub(super) expected_roles: String,
    pub(super) actual_roles: String,
    pub(super) root_force_unit_access_writes: usize,
    pub(super) expected_root_location: String,
    pub(super) actual_root_location: String,
}

impl SegmentRow {
    pub(super) fn count_equal(&self) -> bool {
        self.expected_count_text == self.actual_count_text
    }
    pub(super) fn kind_equal(&self) -> bool {
        self.expected_kind_text == self.actual_kind_text
    }
    pub(super) fn roles_equal(&self) -> bool {
        self.expected_roles == self.actual_roles
    }
    pub(super) fn root_equal(&self) -> bool {
        self.root_force_unit_access_writes == 1 && self.expected_root_location == self.actual_root_location
    }
}

/// R3.c 的比法（跑前登记 e172-r2 第一节 1.2）：H1 逐字比角色名；H2、H3 把 `t5@层级.盘.序号` 的序号那一段抹掉再比多重集合
/// （layout/03 第 51–55 行里的序号是 H1 那一种分配器位置上钉的），序号另报 `name=r3c_ordinal judged=false`。
pub(super) fn compares_role_names_verbatim(history_name: &str) -> bool {
    history_name == "h1"
}

pub(super) fn role_name_without_allocation_ordinal(role: &str) -> String {
    match role.strip_prefix("t5@") {
        Some(position) => match position.rsplit_once('.') {
            Some((level_and_device, _ordinal)) => format!("t5@{level_and_device}"),
            None => role.to_string(),
        },
        None => role.to_string(),
    }
}

fn allocation_ordinals_of(roles: &[String]) -> String {
    let ordinals: Vec<&str> = roles.iter().filter(|role| role.starts_with("t5@")).map(String::as_str).collect();
    if ordinals.is_empty() { "none".to_string() } else { ordinals.join(",") }
}

/// 第 3 行（原第 3 行）判定的射程（读法乙）：H1、H3 全部删除类发布，H2 只拿意图那一次。
fn segment_counts_for_row_three(history_name: &str, publish_kind: &str) -> bool {
    match history_name {
        "h1" | "h3" => true,
        "h2" => publish_kind == "unlink_intent",
        _ => false,
    }
}

fn segment_row_of(history_name: &str, segment: &SegmentAnalysis, before: &VersionState, after: &VersionState) -> Option<SegmentRow> {
    if !segment.is_deletion_publish {
        return None;
    }
    let data_units_after = data_units_of_version(after).unwrap_or(u64::MAX);
    let (deleted_record_leaf, _) = deleted_record_leaf_of_version(before);
    let registered = registered_roles(&segment.label_kind(), data_units_after, deleted_record_leaf).unwrap_or_default();
    let verbatim = compares_role_names_verbatim(history_name);
    let comparable = |role: &str| if verbatim { role.to_string() } else { role_name_without_allocation_ordinal(role) };
    let mut registered_names: Vec<String> = registered.iter().map(|role| (*role).to_string()).collect();
    registered_names.sort();
    let mut decoded_names = segment.role_names();
    decoded_names.sort();
    let mut expected_roles: Vec<String> = registered.iter().map(|role| comparable(role)).collect();
    expected_roles.sort();
    let mut actual_roles: Vec<String> = decoded_names.iter().map(|role| comparable(role)).collect();
    actual_roles.sort();
    let pieces = cut_into_pieces(&segment.operations);
    let (expected_device, expected_offset) = expected_root_slot(segment.label_txg());
    let root_writes: Vec<&DecodedRegion> = segment.regions.iter().filter(|region| region.region_kind == RegionKind::RootRecordSlot).collect();
    Some(SegmentRow {
        history: history_name.to_string(),
        txg: segment.label_txg(),
        publish_kind: segment.label_kind(),
        counts_for_row_three: segment_counts_for_row_three(history_name, &segment.label_kind()),
        compares_role_names_verbatim: verbatim,
        expected_ordinals: allocation_ordinals_of(&registered_names),
        actual_ordinals: allocation_ordinals_of(&decoded_names),
        expected_count_text: expected_count_text(registered.len()),
        actual_count_text: segment_count_text(&pieces),
        expected_kind_text: expected_kind_text(registered.len()),
        actual_kind_text: segment_kind_text(&pieces),
        expected_roles: format!("{}:{}", expected_roles.len(), expected_roles.join(",")),
        actual_roles: format!("{}:{}", actual_roles.len(), actual_roles.join(",")),
        root_force_unit_access_writes: segment.operations.iter().filter(|operation| operation.is_force_unit_access).count(),
        expected_root_location: format!("{expected_device}:{expected_offset}"),
        actual_root_location: root_writes.iter().map(|region| format!("{}:{}", region.device, region.offset)).collect::<Vec<_>>().join(","),
    })
}

// ---------------------------------------------------------------------------------------------
// 第 4、5 行：角色身份集合的差与多写单元的分配记录归类。
// ---------------------------------------------------------------------------------------------

/// 分配记录：key (设备 4, 槽号 6)、value (跨度段 2，最高位是已释放标志, 分配代或释放代 8)（layout/01 第 297 行）。
fn allocation_records_of(node: &IndexNode) -> BTreeMap<(u64, u64), (u64, u64)> {
    node.entries.iter().map(|entry| ((read_u32(entry, 0), read_unsigned_48_bit(entry, 4)), (read_u16(entry, 10), read_u64(entry, 12)))).collect()
}

const ALLOCATION_SPAN_RELEASED_FLAG: u64 = 0x8000;

/// 一版里活着的落点：映射条目的 value 位置（映射是位置的权威，layout/01 第 233 行）加根记录直接持有的树表、映射根、实例表、分配记录树根。
fn live_locations_of_version(state: &VersionState) -> BTreeSet<(u64, u64)> {
    let mut live = BTreeSet::new();
    if let Some(mapping) = state.latest_node(MAPPING_TREE, true) {
        for entry in &mapping.entries {
            for offset in [27, 41] {
                let location = location_at(entry, offset);
                live.insert((u64::from(location.device), location.slot));
            }
        }
    }
    if let Some(record) = &state.latest_root_record {
        for block_pointer in [&record.tree_table_pointer, &record.mapping_root_pointer, &record.instance_table_pointer, &record.allocation_root_pointer] {
            live.extend(block_pointer.locations.iter().map(|location| (u64::from(location.device), location.slot)));
        }
    }
    live
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RecordChangeBasis {
    AllocatedInThisPublish,
    ReleasedInThisPublish,
    ReclaimedUnderTheFloor,
    Unmatched,
}

impl RecordChangeBasis {
    pub(super) fn name(self) -> &'static str {
        match self {
            RecordChangeBasis::AllocatedInThisPublish => "allocated_in_this_publish",
            RecordChangeBasis::ReleasedInThisPublish => "released_in_this_publish",
            RecordChangeBasis::ReclaimedUnderTheFloor => "reclaimed_under_the_floor",
            RecordChangeBasis::Unmatched => "unmatched",
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct UniqueUnitJudgement {
    pub(super) role_name: String,
    pub(super) previous_in_window: bool,
    #[allow(clippy::type_complexity, reason = "这个元组类型只在这一处出现，起类型别名只多一个只用一次的名字")]
    pub(super) changes: Vec<((u64, u64), Option<(u64, u64)>, Option<(u64, u64)>, RecordChangeBasis)>,
}

impl UniqueUnitJudgement {
    pub(super) fn has_clause_basis(&self) -> bool {
        self.changes.iter().any(|(_, _, _, basis)| *basis != RecordChangeBasis::Unmatched)
    }
}

/// R4.b：多写的单元与窗口里同一角色上一次写出的那份比分配记录，逐条归 ①②③。
pub(super) fn judge_unique_unit(unit: &UnitRole, segment: &SegmentAnalysis, before: &VersionState, after: &VersionState) -> UniqueUnitJudgement {
    let DecodedBody::IndexNode(node) = &unit.copy_on_first_device.body else {
        return UniqueUnitJudgement { role_name: unit.role_name.clone(), previous_in_window: false, changes: Vec::new() };
    };
    let previous = before.latest_by_identity.get(&unit.identity).and_then(|region| match &region.body {
        DecodedBody::IndexNode(previous_node) => Some(allocation_records_of(previous_node)),
        DecodedBody::SystemConfiguration(_)
        | DecodedBody::RootRecord(_)
        | DecodedBody::JournalRecord(_)
        | DecodedBody::PackedContainer(_)
        | DecodedBody::DataUnit { .. }
        | DecodedBody::Unrecognized => None,
    });
    let previous_in_window = previous.is_some();
    let previous_records = previous.unwrap_or_default();
    let current_records = if node.tree_identifier == ALLOCATION_RECORDS_TREE && node.level == 0 { allocation_records_of(node) } else { BTreeMap::new() };
    let written: BTreeSet<(u64, u64)> = segment
        .regions
        .iter()
        .filter(|region| region.region_kind == RegionKind::Unit)
        .map(|region| (u64::from(region.device), region.slot()))
        .collect();
    let released: BTreeSet<(u64, u64)> = live_locations_of_version(before).difference(&live_locations_of_version(after)).copied().collect();
    let floor = segment.root_records.first().map_or(0, |(_, _, record)| record.floor);
    let keys: BTreeSet<(u64, u64)> = previous_records.keys().chain(current_records.keys()).copied().collect();
    let mut changes = Vec::new();
    for key in keys {
        let before_value = previous_records.get(&key).copied();
        let after_value = current_records.get(&key).copied();
        if before_value == after_value {
            continue;
        }
        let basis = if written.contains(&key) {
            RecordChangeBasis::AllocatedInThisPublish
        } else if released.contains(&key) {
            RecordChangeBasis::ReleasedInThisPublish
        } else if matches!(before_value, Some((span, generation)) if span & ALLOCATION_SPAN_RELEASED_FLAG != 0 && generation <= floor) && after_value.is_none() {
            RecordChangeBasis::ReclaimedUnderTheFloor
        } else {
            RecordChangeBasis::Unmatched
        };
        changes.push((key, before_value, after_value, basis));
    }
    UniqueUnitJudgement { role_name: unit.role_name.clone(), previous_in_window, changes }
}

pub(super) fn value_text(value: Option<(u64, u64)>) -> String {
    value.map_or_else(|| "absent".to_string(), |(span, generation)| format!("span{span:#06x}/generation{generation}"))
}

/// R4.c：这次发布写出的单元落点的最小与最大槽号、写到的分配记录树叶序号。
pub(super) fn allocation_geometry_line(history_name: &str, segment: &SegmentAnalysis) -> String {
    let slots: Vec<u64> = segment.units.iter().map(|unit| unit.slot).collect();
    let leaves: BTreeSet<String> = segment
        .units
        .iter()
        .filter(|unit| unit.identity.tree_identifier == ALLOCATION_RECORDS_TREE && unit.identity.level == 0 && !unit.identity.is_tree_root)
        .map(|unit| format!("{}.{}", unit.identity.allocation_device.unwrap_or(u64::MAX), unit.identity.allocation_position.unwrap_or(u64::MAX)))
        .collect();
    format!(
        "name=r4c history={history_name} txg={} publish={} allocated_minimum_slot={} allocated_maximum_slot={} allocation_leaves_written={}",
        segment.label_txg(),
        segment.label_kind(),
        slots.iter().min().map_or_else(|| "none".to_string(), u64::to_string),
        slots.iter().max().map_or_else(|| "none".to_string(), u64::to_string),
        leaves.into_iter().collect::<Vec<_>>().join(",")
    )
}

pub(super) fn names_of_identities(segment: &SegmentAnalysis, identities: &BTreeSet<RoleIdentity>) -> Vec<String> {
    let mut names: Vec<String> = segment.units.iter().filter(|unit| identities.contains(&unit.identity)).map(|unit| unit.role_name.clone()).collect();
    names.sort();
    names
}

// ---------------------------------------------------------------------------------------------
// 第 7 行：td 与 tt 头里的层级与 key 区间（只记值、标读法，不判对错）。
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum KeyRangeReading {
    EntryKeyRange,
    SubtreeCoverage,
    BothReadingsCoincide,
    Neither,
}

impl KeyRangeReading {
    pub(super) fn name(self) -> &'static str {
        match self {
            KeyRangeReading::EntryKeyRange => "entry_key_range",
            KeyRangeReading::SubtreeCoverage => "subtree_coverage",
            KeyRangeReading::BothReadingsCoincide => "both_readings_coincide",
            KeyRangeReading::Neither => "neither",
        }
    }
}

/// 跑前登记 R7：key 区间 = [首条条目 key, 末条条目 key] ⇒ 条目键区间；= 这个节点罩住的整个 key 空间 ⇒ 子树覆盖区间。
pub(super) fn key_range_reading(range: (u64, u64), entry_keys: (u64, u64), coverage_candidates: &[(u64, u64)]) -> KeyRangeReading {
    let is_entry_range = range == entry_keys;
    let is_coverage = coverage_candidates.contains(&range);
    match (is_entry_range, is_coverage) {
        (true, true) => KeyRangeReading::BothReadingsCoincide,
        (true, false) => KeyRangeReading::EntryKeyRange,
        (false, true) => KeyRangeReading::SubtreeCoverage,
        (false, false) => KeyRangeReading::Neither,
    }
}

fn key_range_line(history_name: &str, segment: &SegmentAnalysis, tree: u64, role: &str, coverage_candidates: &[(u64, u64)]) -> Option<(String, KeyRangeReading)> {
    let node = index_node_written_in(segment, tree)?;
    let range = (read_u64(&node.key_range_low, 0), read_u64(&node.key_range_high, 0));
    let keys: Vec<u64> = node.entries.iter().map(|entry| read_u64(entry, 0)).collect();
    let entry_keys = (keys.first().copied().unwrap_or(0), keys.last().copied().unwrap_or(0));
    let reading = key_range_reading(range, entry_keys, coverage_candidates);
    Some((
        format!(
            "name=r7 history={history_name} txg={} role={role} level={} key_range={}..{} entry_keys={}..{} reading={} judged=false",
            segment.label_txg(),
            node.level,
            range.0,
            range.1,
            entry_keys.0,
            entry_keys.1,
            reading.name()
        ),
        reading,
    ))
}

// ---------------------------------------------------------------------------------------------
// 第 8 行：挂载那一截每一次写由解码器独立归属，再与导出的段逐写比。
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum WriteAttribution {
    Publish(u64),
    InstanceAcquisition,
    OtherMountStep,
}

impl WriteAttribution {
    pub(super) fn text(&self) -> String {
        match self {
            WriteAttribution::Publish(txg) => format!("publish_txg{txg}"),
            WriteAttribution::InstanceAcquisition => "instance_acquisition".to_string(),
            WriteAttribution::OtherMountStep => "other_mount_step".to_string(),
        }
    }
}

/// R8.c 的系统配置槽归属（跑前登记 e172-r2 第六节 6.4，依次判）：① 同一块盘上前一份的实例代号换了而 tail 没动 ⇒ 取号；
/// ② 不是 ①，tail 等于某次发布那条 journal 记录 jsn 的计数器、实例代号等于那条 jsn 的实例代号、且那次发布的根槽写排在这一次之前
/// ⇒ 那次发布的轮换（满足的有几次就取根槽最靠后的那一次）；③ 都不是 ⇒ 别的挂载步。
/// `publishes` 每项是（txg、jsn 计数器、jsn 实例代号、根槽那一步）。
pub(super) fn attribute_system_configuration(
    step: usize,
    configuration: &SystemConfiguration,
    previous_on_device: Option<&SystemConfiguration>,
    publishes: &[(u64, u64, u64, usize)],
) -> WriteAttribution {
    let is_acquisition = previous_on_device
        .is_some_and(|previous| previous.instance != configuration.instance && previous.journal_tail == configuration.journal_tail);
    if ATTRIBUTION_JUDGES_ACQUISITION_BEFORE_ROTATION && is_acquisition {
        return WriteAttribution::InstanceAcquisition;
    }
    let rotation = publishes
        .iter()
        .filter(|(_, counter, instance, root_step)| {
            *counter == configuration.journal_tail && (!ROTATION_ATTRIBUTION_REQUIRES_THE_SAME_INSTANCE || *instance == configuration.instance) && *root_step < step
        })
        .max_by_key(|(_, _, _, root_step)| *root_step)
        .map(|(txg, _, _, _)| WriteAttribution::Publish(*txg));
    match (rotation, is_acquisition) {
        (Some(attribution), _) => attribution,
        (None, true) => WriteAttribution::InstanceAcquisition,
        (None, false) => WriteAttribution::OtherMountStep,
    }
}

#[derive(Clone, Debug)]
pub(super) struct AttributionRow {
    pub(super) step: usize,
    pub(super) device: u32,
    pub(super) region_kind: RegionKind,
    pub(super) decoded: WriteAttribution,
    pub(super) exported_segment: String,
    pub(super) is_match: bool,
}

/// R8.c：挂载那一截 = 导出里 `mount_writes_before_its_first_publish` 那一段起到这段历史末尾。
fn attribute_mount_writes(analysis: &HistoryAnalysis) -> Vec<AttributionRow> {
    let Some(mount_start) = analysis
        .segments
        .iter()
        .find(|segment| segment.label_kind() == "mount_writes_before_its_first_publish")
        .map(|segment| segment.start_step)
    else {
        return Vec::new();
    };
    // 窗口里每次发布：(txg, journal 计数器, journal 实例代号, 根槽那一步)，全靠解码。
    let mut publishes = Vec::new();
    for segment in &analysis.segments {
        let root_step = segment.regions.iter().find(|region| region.region_kind == RegionKind::RootRecordSlot).map(|region| region.step);
        if let (Some((_, _, root)), Some((_, _, journal)), Some(root_step)) = (segment.root_records.first(), segment.journal_records.first(), root_step) {
            publishes.push((root.txg, journal.counter, journal.instance, root_step));
        }
    }
    let mut previous_configuration: BTreeMap<u32, SystemConfiguration> = BTreeMap::new();
    let mut rows = Vec::new();
    for segment in &analysis.segments {
        for region in &segment.regions {
            let decoded = match &region.body {
                DecodedBody::SystemConfiguration(configuration) => {
                    let attribution = attribute_system_configuration(region.step, configuration, previous_configuration.get(&region.device), &publishes);
                    previous_configuration.insert(region.device, configuration.clone());
                    attribution
                }
                DecodedBody::RootRecord(_)
                | DecodedBody::JournalRecord(_)
                | DecodedBody::IndexNode(_)
                | DecodedBody::PackedContainer(_)
                | DecodedBody::DataUnit { .. }
                | DecodedBody::Unrecognized => region.birth_or_checkpoint_txg().map_or(WriteAttribution::OtherMountStep, WriteAttribution::Publish),
            };
            if region.step < mount_start {
                continue;
            }
            let is_match = match (&decoded, &segment.label) {
                (WriteAttribution::Publish(txg), SegmentLabel::Publish { txg: exported_txg, .. }) => txg == exported_txg,
                (WriteAttribution::InstanceAcquisition | WriteAttribution::OtherMountStep, SegmentLabel::Outside { .. }) => true,
                (WriteAttribution::Publish(_), SegmentLabel::Outside { .. }) | (WriteAttribution::InstanceAcquisition | WriteAttribution::OtherMountStep, SegmentLabel::Publish { .. }) => false,
            };
            rows.push(AttributionRow {
                step: region.step,
                device: region.device,
                region_kind: region.region_kind,
                decoded,
                exported_segment: format!("{}{}", segment.label_kind(), segment.label_txg_text()),
                is_match,
            });
        }
    }
    rows
}

impl SegmentAnalysis {
    fn label_txg_text(&self) -> String {
        match &self.label {
            SegmentLabel::Publish { txg, .. } => format!("_txg{txg}"),
            SegmentLabel::Outside { .. } => String::new(),
        }
    }
}

// ---------------------------------------------------------------------------------------------
// 一段历史的全部判定（阳性对照在改坏的副本上重跑同一个函数）。
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default)]
pub(super) struct CoverageTotals {
    pub(super) layout_three_bytes: u64,
    pub(super) layout_one_bytes: u64,
    pub(super) constant_zero_bytes: u64,
    pub(super) unregistered_bytes: u64,
    pub(super) constant_zero_nonzero_bytes: u64,
    pub(super) double_claimed_bytes: u64,
    pub(super) total_bytes: u64,
    pub(super) unregistered_ranges: Vec<String>,
    pub(super) nonzero_padding_ranges: Vec<String>,
}

#[derive(Clone, Debug)]
pub(super) struct HistoryJudgement {
    pub(super) name: String,
    pub(super) analysis: HistoryAnalysis,
    pub(super) cell_rows: Vec<CellRow>,
    pub(super) segment_rows: Vec<SegmentRow>,
    pub(super) structure_failures: Vec<String>,
    pub(super) role_mismatches: Vec<String>,
    pub(super) role_notation_only_differences: Vec<String>,
    pub(super) tiling_failures: Vec<String>,
    pub(super) attribution_rows: Vec<AttributionRow>,
    pub(super) coverage: CoverageTotals,
    pub(super) key_range_lines: Vec<String>,
    pub(super) leaf_branch_records: Vec<String>,
    pub(super) time_threshold_rows: Vec<TimeThresholdRow>,
    /// 导出的 `name=impl_config` 那一行（R6.b 要意图与顺序写两个时间）。
    pub(super) configuration: BTreeMap<String, String>,
    /// 每次删除类发布一行：解出的标签 6 与式子的四个输入（A-tag6、P-pin、R-ret 都从它算；跑前登记 e172-r3 第六节 6.2）。
    pub(super) tag_six_rows: Vec<TagSixRow>,
}

/// 一次删除类发布之后那一版的标签 6 读数与式子的四个输入（都从解码的那一版现读）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TagSixRow {
    pub(super) publish: DeletionPublish,
    pub(super) txg: u64,
    pub(super) actual: Option<u64>,
    pub(super) tombstone_levels: u64,
    pub(super) retirement_levels: u64,
    pub(super) live_inodes: u64,
    pub(super) inode_records: u64,
}

impl TagSixRow {
    pub(super) fn actual_text(&self) -> String {
        self.actual.map_or_else(|| "absent".to_string(), |value| value.to_string())
    }
}

/// R1.t（跑前登记 e172-r2 第六节 6.2）：窗口里每一次系统配置槽写，偏移 433 起 4 字节小端解出的 T_time。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TimeThresholdRow {
    pub(super) step: usize,
    pub(super) device: u32,
    pub(super) offset: u64,
    pub(super) actual_milliseconds: u64,
}

impl TimeThresholdRow {
    pub(super) fn is_expected(&self) -> bool {
        self.actual_milliseconds == EXPECTED_TIME_THRESHOLD_MILLISECONDS
    }
}

fn time_threshold_rows_of(analysis: &HistoryAnalysis) -> Vec<TimeThresholdRow> {
    analysis
        .segments
        .iter()
        .flat_map(|segment| segment.system_configurations.iter())
        .map(|(step, device, offset, configuration)| TimeThresholdRow {
            step: *step,
            device: *device,
            offset: *offset,
            actual_milliseconds: configuration.time_threshold_milliseconds,
        })
        .collect()
}

fn runs_of(classes: &[ByteClass], wanted: ByteClass) -> Vec<(usize, usize)> {
    let mut runs = Vec::new();
    let mut run_start: Option<usize> = None;
    for (byte_offset, class) in classes.iter().enumerate() {
        match (*class == wanted, run_start) {
            (true, None) => run_start = Some(byte_offset),
            (false, Some(start)) => {
                runs.push((start, byte_offset));
                run_start = None;
            }
            (true, Some(_)) | (false, None) => {}
        }
    }
    if let Some(start) = run_start {
        runs.push((start, classes.len()));
    }
    runs
}

pub(super) fn coverage_of(analysis: &HistoryAnalysis) -> CoverageTotals {
    let mut totals = CoverageTotals::default();
    for segment in &analysis.segments {
        for region in &segment.regions {
            let role = segment
                .units
                .iter()
                .find(|unit| unit.slot == region.slot() && region.region_kind == RegionKind::Unit)
                .map_or_else(|| region.region_kind.step_kind_name().to_string(), |unit| unit.role_name.clone());
            totals.total_bytes += u64_of(region.bytes.len());
            totals.double_claimed_bytes += region.coverage.double_claimed_bytes;
            for (byte_offset, class) in region.coverage.classes.iter().enumerate() {
                match class {
                    ByteClass::LayoutThreeCell => totals.layout_three_bytes += 1,
                    ByteClass::LayoutOneCell => totals.layout_one_bytes += 1,
                    ByteClass::ConstantZeroPadding => {
                        totals.constant_zero_bytes += 1;
                        if region.bytes[byte_offset] != 0 {
                            totals.constant_zero_nonzero_bytes += 1;
                        }
                    }
                    ByteClass::Unclaimed => totals.unregistered_bytes += 1,
                }
            }
            for (start, end) in runs_of(&region.coverage.classes, ByteClass::Unclaimed) {
                let first_bytes = &region.bytes[start..end.min(start + 16)];
                totals.unregistered_ranges.push(format!(
                    "history={} step={} device={} role={role} start={start} end={end} length={} first16={} layout02_has=unknown",
                    analysis.name,
                    region.step,
                    region.device,
                    end - start,
                    hexadecimal_of(first_bytes)
                ));
            }
            let nonzero_padding: Vec<ByteClass> = region
                .coverage
                .classes
                .iter()
                .zip(region.bytes.iter())
                .map(|(class, byte)| if *class == ByteClass::ConstantZeroPadding && *byte != 0 { ByteClass::Unclaimed } else { ByteClass::LayoutOneCell })
                .collect();
            for (start, end) in runs_of(&nonzero_padding, ByteClass::Unclaimed) {
                totals.nonzero_padding_ranges.push(format!("history={} step={} device={} role={role} start={start} end={end}", analysis.name, region.step, region.device));
            }
        }
    }
    totals
}

pub(super) fn judge_history(history: &HistoryExport) -> HistoryJudgement {
    let analysis = analyze_history(history);
    let mut cell_rows = Vec::new();
    let mut segment_rows = Vec::new();
    let mut structure_failures = Vec::new();
    let mut role_mismatches = Vec::new();
    let mut role_notation_only_differences = Vec::new();
    let mut key_range_lines = Vec::new();
    let mut leaf_branch_records = Vec::new();
    let mut tag_six_rows = Vec::new();
    let deletion_publishes: Vec<(DeletionPublish, u64)> = analysis
        .segments
        .iter()
        .zip(&analysis.states_after)
        .filter_map(|(segment, after)| deletion_publish_of(segment, after).map(|publish| (publish, segment.label_txg())))
        .collect();
    for (segment_index, segment) in analysis.segments.iter().enumerate() {
        let before = &analysis.states_before[segment_index];
        let after = &analysis.states_after[segment_index];
        if let Some(publish) = deletion_publish_of(segment, after) {
            tag_six_rows.push(TagSixRow {
                publish,
                txg: segment.label_txg(),
                actual: accounting_value(after, 6),
                tombstone_levels: tombstone_internal_levels(after),
                retirement_levels: retirement_internal_levels(after),
                live_inodes: live_inode_count(after),
                inode_records: inode_record_count(after),
            });
        }
        structure_failures.extend(segment.structure_failures.iter().map(|failure| format!("history={} {failure}", history.name)));
        if let Some((decoded, exported, is_equal, differs_only_in_index_notation)) = role_name_comparison(segment) {
            let text = format!("history={} txg={} decoded={} exported={}", history.name, segment.label_txg(), decoded.join(","), exported.join(","));
            if differs_only_in_index_notation {
                role_notation_only_differences.push(text);
            } else if !is_equal {
                role_mismatches.push(text);
            }
        }
        if segment.is_deletion_publish {
            leaf_branch_records.extend(judge_layout_three_cells(history, segment, before, after, &deletion_publishes, &mut cell_rows));
            segment_rows.extend(segment_row_of(&history.name, segment, before, after));
        }
        if segment.label_kind() == "unlink_intent" {
            let whole_space = [(0, u64::MAX)];
            let tombstone_space = [(1, u64::MAX), (0, u64::MAX)];
            key_range_lines.extend(key_range_line(&history.name, segment, DELETED_INODES_TREE, "td", &whole_space).map(|(line, _)| line));
            key_range_lines.extend(key_range_line(&history.name, segment, TOMBSTONE_TREE, "tt", &tombstone_space).map(|(line, _)| line));
            // L03-11：墓碑树根条目身份引用里的容器出生代只记值、不钉（跑前登记第六节 6.1）。
            let reference_birth = index_node_written_in(segment, TOMBSTONE_TREE)
                .and_then(|node| node.entries.first().map(|entry| read_u64(entry, 26).to_string()))
                .unwrap_or_else(|| "absent".to_string());
            let header_birth = container_written_in(segment, TOMBSTONE_TREE, 1).map_or_else(|| "absent".to_string(), |(_, container)| container.container_birth.to_string());
            key_range_lines.push(format!(
                "name=l03_11_record history={} txg={} identity_reference_container_birth={reference_birth} container_header_birth={header_birth} judged=false",
                history.name,
                segment.label_txg()
            ));
        }
        // L03-17：退役树根的层级与 key 区间只记值（跑前登记 e172-r3 第五节 5.8 改24，同 R7 那一类）；容器号从 1 起（layout/03 第 21 行）。
        if segment.is_deletion_publish {
            let retirement_space = [(1, u64::MAX), (0, u64::MAX)];
            key_range_lines.extend(key_range_line(&history.name, segment, RETIREMENT_TREE, "tr", &retirement_space).map(|(line, _)| line));
        }
    }
    let tiling_failures = tiling_failures(history, &analysis);
    let attribution_rows = attribute_mount_writes(&analysis);
    let coverage = coverage_of(&analysis);
    let time_threshold_rows = time_threshold_rows_of(&analysis);
    HistoryJudgement {
        name: history.name.clone(),
        analysis,
        cell_rows,
        segment_rows,
        structure_failures,
        role_mismatches,
        role_notation_only_differences,
        tiling_failures,
        attribution_rows,
        coverage,
        key_range_lines,
        leaf_branch_records,
        time_threshold_rows,
        configuration: history.configuration.clone(),
        tag_six_rows,
    }
}

impl HistoryJudgement {
    pub(super) fn intent_cell_is_equal(&self, cell: &str, field: &str) -> Option<bool> {
        self.cell_rows.iter().find(|row| row.cell == cell && row.field == field && row.publish_kind == "unlink_intent").map(CellRow::is_equal)
    }
    pub(super) fn intent_segment_count_is_equal(&self) -> Option<bool> {
        self.segment_rows.iter().find(|row| row.publish_kind == "unlink_intent").map(SegmentRow::count_equal)
    }
    /// R8.c（跑前登记 e172-r2 第六节 6.4）的归错数。
    pub(super) fn misattributed_writes(&self) -> usize {
        self.attribution_rows.iter().filter(|row| !row.is_match).count()
    }
    /// R1.t：T_time ≠ 5000 的系统配置槽写。
    pub(super) fn time_threshold_rows_not_expected(&self) -> Vec<&TimeThresholdRow> {
        self.time_threshold_rows.iter().filter(|row| !row.is_expected()).collect()
    }
    /// S2 里「系统配置整槽校验和不对」那一类的条数（PC-H 要它改坏前后相同）。
    pub(super) fn system_configuration_checksum_failures(&self) -> usize {
        self.structure_failures.iter().filter(|failure| failure.contains("系统配置整槽校验和不对")).count()
    }
    pub(super) fn intent_times_row(&self) -> Option<&CellRow> {
        self.cell_rows.iter().find(|row| row.cell == "L03-5" && row.field == "times" && row.publish_kind == "unlink_intent")
    }
    pub(super) fn intent_inode_container_checksum_failures(&self) -> usize {
        let Some(intent) = self.analysis.segments.iter().find(|segment| segment.label_kind() == "unlink_intent") else { return 0 };
        let inode_container_steps: BTreeSet<usize> = intent
            .regions
            .iter()
            .filter(|region| matches!(&region.body, DecodedBody::PackedContainer(container) if container.birth_tree == INODE_TREE))
            .map(|region| region.step)
            .collect();
        intent
            .regions
            .iter()
            .filter(|region| inode_container_steps.contains(&region.step))
            .filter(|region| region.structure_failures.iter().any(|failure| failure.contains("码 3 载荷校验和不对")))
            .count()
    }
}
