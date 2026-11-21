//! 读者的扫描：键计数、墓碑与死亡记录的压制、单元扫描读数、组与来源、现行读法的择新与逐单元整扫对拍。

use super::*;

// ---------------------------------------------------------------------------------------------
// R-U：读每个读得出单元里的每条 key（增量计数；最后一次扫描与阳性对照另用整扫一遍的参照路径）
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default)]
struct KeyCounter {
    /// 这条臂下算数的、读得出单元里的出现次数。
    readable: u64,
    /// 其中供出它的单元在环窗内（释放代 s 满足 t − s ≤ 23，Q14）的出现次数。
    in_window: u64,
    in_truth: bool,
}

impl KeyCounter {
    fn contribution(self) -> (u64, u64, u64) {
        (
            u64::from(self.readable > 0 && !self.in_truth),
            u64::from(self.readable == 0 && self.in_truth),
            u64::from(self.in_window > 0 && !self.in_truth),
        )
    }
}

/// 臂 B 的墓碑效力按出现的身份判（登记 5.3）：inode 记录对 (对象, 出生代)；extent 记录对 inode 与数据指针出生 txg ∈ [b, t_d]；
/// `deleted_inodes` 对 inode。
pub(super) fn tombstone_suppresses(tombstones_of_the_object: &[TombstoneRecordSummary], tag: OccurrenceTag) -> bool {
    match tag {
        OccurrenceTag::InodeRecord { object_birth, .. } => tombstones_of_the_object
            .iter()
            .any(|tombstone| tombstone.object_birth == object_birth),
        OccurrenceTag::ExtentRecord { pointer_birth, .. } => tombstones_of_the_object
            .iter()
            .any(|tombstone| tombstone.object_birth <= pointer_birth && pointer_birth <= tombstone.death_txg),
        OccurrenceTag::DeletedInode { .. } => !tombstones_of_the_object.is_empty(),
        OccurrenceTag::Untagged => false,
    }
}

pub(super) fn object_of_tag(tag: OccurrenceTag) -> Option<u64> {
    match tag {
        OccurrenceTag::InodeRecord { inode, .. }
        | OccurrenceTag::ExtentRecord { inode, .. }
        | OccurrenceTag::DeletedInode { inode } => Some(inode),
        OccurrenceTag::Untagged => None,
    }
}

/// 臂 P：一条 key 最晚一条死亡记录的 t ≥ 供出它的单元版本的诞生代号就不收（登记 5.3）。
pub(super) fn death_record_suppresses(death_time: &HashMap<NamespacedKey, u64>, key: &NamespacedKey, provider_birth: u64) -> bool {
    death_time.get(key).is_some_and(|death| *death >= provider_birth)
}

#[derive(Clone, Debug)]
pub(super) struct UnitScanReading {
    pub(super) arm: Arm,
    pub(super) tracks_the_ring_window: bool,
    counters: HashMap<NamespacedKey, KeyCounter>,
    pub(super) resurrected: [u64; NAMESPACE_COUNT],
    pub(super) missed: [u64; NAMESPACE_COUNT],
    pub(super) resurrected_in_window: [u64; NAMESPACE_COUNT],
    tombstones_by_object: HashMap<u64, Vec<TombstoneRecordSummary>>,
    tagged_units_by_object: HashMap<u64, Vec<UnitIdentifier>>,
    window_units: BTreeMap<UnitIdentifier, u64>,
}

impl UnitScanReading {
    pub(super) fn new(arm: Arm, tracks_the_ring_window: bool) -> Self {
        Self {
            arm,
            tracks_the_ring_window,
            counters: HashMap::new(),
            resurrected: [0; NAMESPACE_COUNT],
            missed: [0; NAMESPACE_COUNT],
            resurrected_in_window: [0; NAMESPACE_COUNT],
            tombstones_by_object: HashMap::new(),
            tagged_units_by_object: HashMap::new(),
            window_units: BTreeMap::new(),
        }
    }

    fn counts(&self, occurrence: &Occurrence, provider_birth: u64, death_time: &HashMap<NamespacedKey, u64>) -> bool {
        match self.arm {
            Arm::WithoutTombstones => true,
            Arm::TodaysTombstones => match object_of_tag(occurrence.tag) {
                Some(object) => !self
                    .tombstones_by_object
                    .get(&object)
                    .is_some_and(|tombstones| tombstone_suppresses(tombstones, occurrence.tag)),
                None => true,
            },
            Arm::DeathRecordForEveryKey => !death_record_suppresses(death_time, &occurrence.key, provider_birth),
        }
    }

    fn change(&mut self, key: NamespacedKey, update: impl FnOnce(&mut KeyCounter)) {
        let counter = self.counters.entry(key).or_default();
        let before = counter.contribution();
        update(counter);
        let after = counter.contribution();
        let position = key.namespace.position();
        self.resurrected[position] = self.resurrected[position] + after.0 - before.0;
        self.missed[position] = self.missed[position] + after.1 - before.1;
        self.resurrected_in_window[position] = self.resurrected_in_window[position] + after.2 - before.2;
    }

    pub(super) fn set_truth(&mut self, key: NamespacedKey, in_truth: bool) {
        self.change(key, |counter| counter.in_truth = in_truth);
    }

    /// 一个单元写下、读得出了。
    pub(super) fn on_unit_readable(&mut self, unit_identifier: UnitIdentifier, unit: &Unit, death_time: &HashMap<NamespacedKey, u64>) {
        for occurrence in occurrences_of(&unit.contents) {
            if let Some(object) = object_of_tag(occurrence.tag) {
                let units = self.tagged_units_by_object.entry(object).or_default();
                if units.last() != Some(&unit_identifier) {
                    units.push(unit_identifier);
                }
            }
            if self.counts(&occurrence, unit.birth_txg(), death_time) {
                self.change(occurrence.key, |counter| counter.readable += 1);
            }
        }
    }

    /// 一个单元被之后的写盖过、读不出了。
    pub(super) fn on_unit_unreadable(&mut self, unit_identifier: UnitIdentifier, unit: &Unit, death_time: &HashMap<NamespacedKey, u64>) {
        let was_in_window = self.window_units.remove(&unit_identifier).is_some();
        for occurrence in occurrences_of(&unit.contents) {
            if self.counts(&occurrence, unit.birth_txg(), death_time) {
                self.change(occurrence.key, |counter| {
                    counter.readable -= 1;
                    if was_in_window {
                        counter.in_window -= 1;
                    }
                });
            }
        }
    }

    /// 一个读得出的单元这次被释放：进环窗（只有 G-环 报 Q14）。
    pub(super) fn on_unit_released(&mut self, unit_identifier: UnitIdentifier, unit: &Unit, txg: u64, death_time: &HashMap<NamespacedKey, u64>) {
        if !self.tracks_the_ring_window || !unit.is_readable {
            return;
        }
        self.window_units.insert(unit_identifier, txg);
        for occurrence in occurrences_of(&unit.contents) {
            if self.counts(&occurrence, unit.birth_txg(), death_time) {
                self.change(occurrence.key, |counter| counter.in_window += 1);
            }
        }
    }

    /// 扫描时刻 t：释放代 s 满足 t − s > 23 的单元出环窗。
    pub(super) fn expire_the_window(&mut self, txg: u64, disk: &Disk, death_time: &HashMap<NamespacedKey, u64>) {
        let expired: Vec<UnitIdentifier> = self
            .window_units
            .iter()
            .filter(|(_, released_at)| txg - **released_at > RING_WINDOW_IN_PUBLISHES)
            .map(|(unit_identifier, _)| *unit_identifier)
            .collect();
        for unit_identifier in expired {
            self.window_units.remove(&unit_identifier);
            let unit = &disk.units[unit_identifier];
            for occurrence in occurrences_of(&unit.contents) {
                if self.counts(&occurrence, unit.birth_txg(), death_time) {
                    self.change(occurrence.key, |counter| counter.in_window -= 1);
                }
            }
        }
    }

    /// 臂 P：这条 key 这次死了，此刻盘上它的每次出现都早于死亡记录，一律不收。
    pub(super) fn on_key_death(&mut self, key: NamespacedKey) {
        self.change(key, |counter| {
            counter.readable = 0;
            counter.in_window = 0;
        });
    }

    /// 臂 B：这次写下一条墓碑，此刻读得出单元里被它指认的出现一律不收（墓碑容器只增，写下就一直读得出）。
    pub(super) fn on_tombstone(&mut self, tombstone: TombstoneRecordSummary, disk: &Disk) {
        let units = self
            .tagged_units_by_object
            .get(&tombstone.object)
            .cloned()
            .unwrap_or_default();
        let single = [tombstone];
        for unit_identifier in units {
            let unit = &disk.units[unit_identifier];
            if !unit.is_readable {
                continue;
            }
            let in_window = self.window_units.contains_key(&unit_identifier);
            for occurrence in occurrences_of(&unit.contents) {
                if object_of_tag(occurrence.tag) != Some(tombstone.object) {
                    continue;
                }
                let already_suppressed = self
                    .tombstones_by_object
                    .get(&tombstone.object)
                    .is_some_and(|tombstones| tombstone_suppresses(tombstones, occurrence.tag));
                if !already_suppressed && tombstone_suppresses(&single, occurrence.tag) {
                    self.change(occurrence.key, |counter| {
                        counter.readable -= 1;
                        if in_window {
                            counter.in_window -= 1;
                        }
                    });
                }
            }
        }
        self.tombstones_by_object
            .entry(tombstone.object)
            .or_default()
            .push(tombstone);
    }
}

// ---------------------------------------------------------------------------------------------
// R-S、R-G：每次扫描从读得出的单元现算（登记 5.4）；R-U 的整扫参照路径
// ---------------------------------------------------------------------------------------------

/// 供出一条 key 的单元的摘要：P 臂判死亡记录、Q14 判环窗、PC2 手算都读它。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Provider {
    newest_birth: u64,
    freshest_release: Option<u64>,
    has_live_provider: bool,
    pub(super) provider_count: u64,
}

pub(super) type KeysWithProviders = BTreeMap<KeyFields, Provider>;

fn note_provider(keys: &mut KeysWithProviders, fields: KeyFields, unit: &Unit) {
    let provider = keys.entry(fields).or_insert(Provider {
        newest_birth: 0,
        freshest_release: None,
        has_live_provider: false,
        provider_count: 0,
    });
    provider.newest_birth = provider.newest_birth.max(unit.birth_txg());
    provider.provider_count += 1;
    match unit.release_txg {
        Some(released_at) => {
            provider.freshest_release = Some(provider.freshest_release.map_or(released_at, |current| current.max(released_at)));
        }
        None => provider.has_live_provider = true,
    }
}

#[derive(Clone, Debug, Default)]
pub(super) struct UnitLists {
    pub(super) data_units: Vec<UnitIdentifier>,
    pub(super) inode_containers: Vec<UnitIdentifier>,
    pub(super) tombstone_containers: Vec<UnitIdentifier>,
    pub(super) accounting_leaves: Vec<UnitIdentifier>,
}

fn unit_by_identifier<'reading>(disk: &'reading Disk, planted: Option<&'reading Unit>, unit_identifier: UnitIdentifier) -> &'reading Unit {
    if unit_identifier == PLANTED_UNIT {
        planted.expect("植入的单元")
    } else {
        &disk.units[unit_identifier]
    }
}

fn consider_for_selection(newest: &mut BTreeMap<SelectionGroup, (WriteOrder, UnitIdentifier)>, unit_identifier: UnitIdentifier, unit: &Unit) {
    if let Some(group) = unit.group {
        let candidate = (unit.write_order, unit_identifier);
        newest
            .entry(group)
            .and_modify(|current| {
                if candidate.0 > current.0 {
                    *current = candidate;
                }
            })
            .or_insert(candidate);
    }
}

/// 一次扫描里一种读法读出的东西：每个名字空间的 key 与供出者。
#[derive(Clone, Debug)]
pub(super) struct ReadingResult {
    pub(super) keys: Vec<KeysWithProviders>,
}

impl ReadingResult {
    fn empty() -> Self {
        Self {
            keys: vec![KeysWithProviders::new(); NAMESPACE_COUNT],
        }
    }

    fn note(&mut self, namespace: Namespace, fields: KeyFields, unit: &Unit) {
        note_provider(&mut self.keys[namespace.position()], fields, unit);
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ScanInputs<'scan> {
    pub(super) disk: &'scan Disk,
    pub(super) unit_lists: &'scan UnitLists,
    pub(super) death_time: &'scan HashMap<NamespacedKey, u64>,
    pub(super) planted: Option<&'scan Unit>,
    pub(super) erased: Option<UnitIdentifier>,
}

impl ScanInputs<'_> {
    fn is_candidate(&self, unit_identifier: UnitIdentifier) -> bool {
        unit_identifier != PLANTED_UNIT && self.disk.units[unit_identifier].is_readable && Some(unit_identifier) != self.erased
    }

    fn passes_death_records(&self, arm: Arm, namespace: Namespace, fields: KeyFields, provider_birth: u64) -> bool {
        arm != Arm::DeathRecordForEveryKey
            || !death_record_suppresses(self.death_time, &namespaced(namespace, fields), provider_birth)
    }
}

/// R-S 与 R-G（登记 5.4、5.3 臂的效力、C3、C11）。
pub(super) fn select_current_reading(reader: Reader, arm: Arm, inputs: &ScanInputs<'_>) -> ReadingResult {
    let disk = inputs.disk;
    let mut newest: BTreeMap<SelectionGroup, (WriteOrder, UnitIdentifier)> = BTreeMap::new();
    let lists = inputs.unit_lists;
    for unit_identifier in lists
        .data_units
        .iter()
        .chain(&lists.inode_containers)
        .chain(&lists.tombstone_containers)
    {
        if inputs.is_candidate(*unit_identifier) {
            consider_for_selection(&mut newest, *unit_identifier, &disk.units[*unit_identifier]);
        }
    }
    if let Some(planted_unit) = inputs.planted {
        consider_for_selection(&mut newest, PLANTED_UNIT, planted_unit);
    }
    let mut reading = ReadingResult::empty();
    let mut tombstones_by_object: HashMap<u64, Vec<TombstoneRecordSummary>> = HashMap::new();
    let mut accepted_containers: Vec<UnitIdentifier> = Vec::new();
    for (group, (_, unit_identifier)) in &newest {
        if let SelectionGroup::PackedContainer { .. } = group {
            accepted_containers.push(*unit_identifier);
        }
        let unit = unit_by_identifier(disk, inputs.planted, *unit_identifier);
        if let UnitContents::TombstoneRecords(records) = &unit.contents {
            for record in records {
                tombstones_by_object.entry(record.object).or_default().push(*record);
                let fields = tombstone_key(*record);
                if inputs.passes_death_records(arm, Namespace::Tombstone, fields, unit.birth_txg()) {
                    reading.note(Namespace::Tombstone, fields, unit);
                }
            }
        }
    }
    let suppressed = |inode: u64, tag: OccurrenceTag| {
        arm == Arm::TodaysTombstones
            && tombstones_by_object
                .get(&inode)
                .is_some_and(|tombstones| tombstone_suppresses(tombstones, tag))
    };
    let mut inode_tree_objects: BTreeSet<(u64, u64)> = BTreeSet::new();
    for (_, unit_identifier) in newest.values() {
        let unit = unit_by_identifier(disk, inputs.planted, *unit_identifier);
        if let UnitContents::InodeRecords(records) = &unit.contents {
            for record in records {
                let tag = OccurrenceTag::InodeRecord {
                    inode: record.inode,
                    object_birth: record.object_birth,
                };
                if suppressed(record.inode, tag) {
                    continue;
                }
                let fields = single_field_key(record.inode);
                if inputs.passes_death_records(arm, Namespace::Inode, fields, unit.birth_txg()) {
                    reading.note(Namespace::Inode, fields, unit);
                    inode_tree_objects.insert((record.inode, record.object_birth));
                }
                if record.nlink == 0 && inputs.passes_death_records(arm, Namespace::DeletedInodes, fields, unit.birth_txg()) {
                    reading.note(Namespace::DeletedInodes, fields, unit);
                }
            }
        }
    }
    let mut accepted_data_units: Vec<UnitIdentifier> = Vec::new();
    for (group, (_, unit_identifier)) in &newest {
        if let SelectionGroup::DataUnit { object, object_birth, .. } = group {
            let tag = OccurrenceTag::InodeRecord {
                inode: *object,
                object_birth: *object_birth,
            };
            if suppressed(*object, tag) {
                continue;
            }
            if reader == Reader::SelectCurrentGatedByObject && !inode_tree_objects.contains(&(*object, *object_birth)) {
                continue;
            }
            accepted_data_units.push(*unit_identifier);
        }
    }
    for unit_identifier in &accepted_data_units {
        let unit = unit_by_identifier(disk, inputs.planted, *unit_identifier);
        if let UnitContents::DataUnit { object, anchor_unit, .. } = unit.contents {
            let fields = extent_key(object, anchor_unit);
            if inputs.passes_death_records(arm, Namespace::Extent, fields, unit.birth_txg()) {
                reading.note(Namespace::Extent, fields, unit);
            }
        }
    }
    for unit_identifier in accepted_data_units.iter().chain(&accepted_containers) {
        let unit = unit_by_identifier(disk, inputs.planted, *unit_identifier);
        let slot_fields = single_field_key(unit.slot);
        if inputs.passes_death_records(arm, Namespace::AllocationUnitsOfTheVersion, slot_fields, unit.birth_txg()) {
            reading.note(Namespace::AllocationUnitsOfTheVersion, slot_fields, unit);
        }
        if let Some(mapping_fields) = unit.mapping_key {
            if inputs.passes_death_records(arm, Namespace::CentralMappingUnitsOfTheVersion, mapping_fields, unit.birth_txg()) {
                reading.note(Namespace::CentralMappingUnitsOfTheVersion, mapping_fields, unit);
            }
        }
    }
    let mut rows: BTreeMap<[u64; 3], (u64, UnitIdentifier)> = BTreeMap::new();
    let mut accounting_candidates: Vec<UnitIdentifier> = Vec::new();
    if inputs.planted.is_some_and(|planted_unit| planted_unit.owner == UnitOwner::AccountingNode) {
        accounting_candidates.push(PLANTED_UNIT);
    }
    if let Some(newest_leaf) = lists
        .accounting_leaves
        .iter()
        .rev()
        .find(|unit_identifier| inputs.is_candidate(**unit_identifier))
    {
        accounting_candidates.push(*newest_leaf);
    }
    for unit_identifier in accounting_candidates {
        let unit = unit_by_identifier(disk, inputs.planted, unit_identifier);
        if let UnitContents::AccountingKeys(keys) = &unit.contents {
            for key in keys {
                let row = [key[0], key[1], key[2]];
                let candidate = (key[3], unit_identifier);
                rows.entry(row)
                    .and_modify(|current| {
                        if candidate.0 > current.0 {
                            *current = candidate;
                        }
                    })
                    .or_insert(candidate);
            }
        }
    }
    for (row, (generation, unit_identifier)) in rows {
        let unit = unit_by_identifier(disk, inputs.planted, unit_identifier);
        let fields = [row[0], row[1], row[2], generation, 0];
        if inputs.passes_death_records(arm, Namespace::Accounting, fields, unit.birth_txg()) {
            reading.note(Namespace::Accounting, fields, unit);
        }
    }
    reading
}

/// R-U 的整扫参照路径：逐个读得出的单元收它的每次出现，按臂滤。与增量计数不共用代码，最后一次扫描与阳性对照用它。
pub(super) fn unit_scan_reading_by_brute_force(arm: Arm, inputs: &ScanInputs<'_>) -> ReadingResult {
    let disk = inputs.disk;
    let mut candidates: Vec<UnitIdentifier> = (0..disk.units.len())
        .filter(|unit_identifier| inputs.is_candidate(*unit_identifier))
        .collect();
    if inputs.planted.is_some() {
        candidates.push(PLANTED_UNIT);
    }
    let mut tombstones_by_object: HashMap<u64, Vec<TombstoneRecordSummary>> = HashMap::new();
    if arm == Arm::TodaysTombstones {
        for unit_identifier in &candidates {
            let unit = unit_by_identifier(disk, inputs.planted, *unit_identifier);
            if let UnitContents::TombstoneRecords(records) = &unit.contents {
                for record in records {
                    tombstones_by_object.entry(record.object).or_default().push(*record);
                }
            }
        }
    }
    let mut reading = ReadingResult::empty();
    for unit_identifier in &candidates {
        let unit = unit_by_identifier(disk, inputs.planted, *unit_identifier);
        for occurrence in occurrences_of(&unit.contents) {
            let counted = match arm {
                Arm::WithoutTombstones => true,
                Arm::TodaysTombstones => !object_of_tag(occurrence.tag)
                    .and_then(|object| tombstones_by_object.get(&object))
                    .is_some_and(|tombstones| tombstone_suppresses(tombstones, occurrence.tag)),
                Arm::DeathRecordForEveryKey => {
                    !death_record_suppresses(inputs.death_time, &occurrence.key, unit.birth_txg())
                }
            };
            if counted {
                reading.note(occurrence.key.namespace, occurrence.key.fields, unit);
            }
        }
    }
    reading
}

pub(super) fn read_with(reader: Reader, arm: Arm, inputs: &ScanInputs<'_>) -> ReadingResult {
    match reader {
        Reader::UnitScan => unit_scan_reading_by_brute_force(arm, inputs),
        Reader::SelectCurrent | Reader::SelectCurrentGatedByObject => select_current_reading(reader, arm, inputs),
    }
}

/// 一格的读数：复活 r、漏活 m、环窗内的复活 r_in（Q14）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct CellMeasurement {
    pub(super) resurrected: u64,
    pub(super) missed: u64,
    pub(super) resurrected_in_window: u64,
}

pub(super) fn measure_against_truth(read: &KeysWithProviders, truth: &BTreeSet<KeyFields>, txg: u64) -> CellMeasurement {
    let mut measurement = CellMeasurement::default();
    for (fields, provider) in read {
        if !truth.contains(fields) {
            measurement.resurrected += 1;
            if provider
                .freshest_release
                .is_some_and(|released_at| txg - released_at <= RING_WINDOW_IN_PUBLISHES)
            {
                measurement.resurrected_in_window += 1;
            }
        }
    }
    measurement.missed = length_as_u64(truth.iter().filter(|fields| !read.contains_key(*fields)).count());
    measurement
}

// ---------------------------------------------------------------------------------------------
// 轨迹：峰值、峰值在哪一次扫描（报 txg）、为正轮数、期末（第八节）
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Trajectory {
    pub(super) resurrected_peak: u64,
    pub(super) resurrected_peak_at: u64,
    pub(super) resurrected_positive_scans: u64,
    pub(super) resurrected_final: u64,
    pub(super) missed_peak: u64,
    pub(super) missed_positive_scans: u64,
    pub(super) missed_positive_scans_in_unlink: u64,
    pub(super) missed_final: u64,
    pub(super) deaths_peak: u64,
    pub(super) deaths_final: u64,
    pub(super) in_window_peak: u64,
    pub(super) in_window_positive_scans: u64,
    pub(super) outside_window_peak: u64,
    pub(super) outside_window_positive_scans: u64,
}

impl Trajectory {
    pub(super) fn observe(&mut self, txg: u64, measurement: CellMeasurement, deaths: u64, is_in_unlink: bool) {
        if measurement.resurrected > self.resurrected_peak {
            self.resurrected_peak = measurement.resurrected;
            self.resurrected_peak_at = txg;
        }
        self.resurrected_positive_scans += u64::from(measurement.resurrected > 0);
        self.resurrected_final = measurement.resurrected;
        self.missed_peak = self.missed_peak.max(measurement.missed);
        self.missed_positive_scans += u64::from(measurement.missed > 0);
        self.missed_positive_scans_in_unlink += u64::from(measurement.missed > 0 && is_in_unlink);
        self.missed_final = measurement.missed;
        self.deaths_peak = self.deaths_peak.max(deaths);
        self.deaths_final = deaths;
        let outside = measurement.resurrected - measurement.resurrected_in_window;
        self.in_window_peak = self.in_window_peak.max(measurement.resurrected_in_window);
        self.in_window_positive_scans += u64::from(measurement.resurrected_in_window > 0);
        self.outside_window_peak = self.outside_window_peak.max(outside);
        self.outside_window_positive_scans += u64::from(outside > 0);
    }
}

// ---------------------------------------------------------------------------------------------
// 真值（第一节「复活数」「漏活数」、C2、C3）
// ---------------------------------------------------------------------------------------------

pub(super) fn truth_of_the_version(version: &Version, disk: &Disk, allocator: &Allocator) -> Vec<BTreeSet<KeyFields>> {
    let mut truth = vec![BTreeSet::new(); NAMESPACE_COUNT];
    for position in 0..version.file_units.len() {
        truth[Namespace::Extent.position()].insert(extent_key(FIRST_FILE_INODE, length_as_u64(position)));
    }
    for container in &version.inode_containers {
        for record in &container.records {
            truth[Namespace::Inode.position()].insert(single_field_key(record.inode));
        }
    }
    for (slot, record) in &allocator.records {
        truth[Namespace::AllocationRecordsPresent.position()].insert(single_field_key(*slot));
        if !record.is_released {
            truth[Namespace::AllocationRecordsAllocated.position()].insert(single_field_key(*slot));
        }
    }
    let units_of_the_version = version
        .file_units
        .iter()
        .chain(version.inode_containers.iter().map(|container| &container.unit))
        .chain(version.tombstone_containers.iter().map(|container| &container.unit));
    for unit_identifier in units_of_the_version {
        let unit = &disk.units[*unit_identifier];
        assert!(
            unit.class == UNIT_CLASS_DATA || unit.class == UNIT_CLASS_PACKED,
            "这一版的码 1 / 码 3 单元：数据单元与打包容器"
        );
        truth[Namespace::AllocationUnitsOfTheVersion.position()].insert(single_field_key(unit.slot));
        truth[Namespace::CentralMappingUnitsOfTheVersion.position()]
            .insert(unit.mapping_key.expect("码 1、码 3 单元进映射"));
    }
    for fields in version.accounting.shape.keys_in_order() {
        truth[Namespace::Accounting.position()].insert(fields);
    }
    for fields in version.mapping.shape.keys_in_order() {
        truth[Namespace::CentralMapping.position()].insert(fields);
    }
    for inode in &version.deleted_queue {
        truth[Namespace::DeletedInodes.position()].insert(single_field_key(*inode));
    }
    for container in &version.tombstone_containers {
        for record in &container.records {
            truth[Namespace::Tombstone.position()].insert(tombstone_key(*record));
        }
    }
    truth
}

#[cfg(test)]
#[path = "scan_reading_tests.rs"]
mod tests;
