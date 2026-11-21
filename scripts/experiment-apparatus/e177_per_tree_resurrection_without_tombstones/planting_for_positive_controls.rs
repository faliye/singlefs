//! 阳性对照的植入变体、植入的单元与数据单元、植入上下文，以及植入之后的模拟步进。

use super::*;

// ---------------------------------------------------------------------------------------------
// 阳性对照（登记 5.5、C13、C14）：在每段历史的最后一次扫描上另做，不改主轨迹
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PlantVariant {
    /// 臂 N；臂 B 取没有墓碑指认的 key。
    Plain,
    /// 臂 B 取有墓碑指认的 key。
    Tombstoned,
    /// 臂 P 删掉 k 的死亡记录再量。
    WithoutItsDeathRecord,
}

impl PlantVariant {
    pub(super) const fn name(self) -> &'static str {
        match self {
            PlantVariant::Plain => "PC1",
            PlantVariant::Tombstoned => "PC1_tombstoned",
            PlantVariant::WithoutItsDeathRecord => "PC1_without_its_death_record",
        }
    }

    /// 期望增量（5.5）：臂 N +1；臂 B 没有墓碑指认 +1、有 +0；臂 P +0，删掉死亡记录 +1。
    const fn expected_increase(self, arm: Arm) -> u64 {
        match (arm, self) {
            (Arm::WithoutTombstones, _) | (Arm::TodaysTombstones, PlantVariant::Plain) => 1,
            (Arm::TodaysTombstones, _) => 0,
            (Arm::DeathRecordForEveryKey, PlantVariant::WithoutItsDeathRecord) => 1,
            (Arm::DeathRecordForEveryKey, _) => 0,
        }
    }
}

fn plant_variants_of(arm: Arm) -> Vec<PlantVariant> {
    match arm {
        Arm::WithoutTombstones => vec![PlantVariant::Plain],
        Arm::TodaysTombstones => vec![PlantVariant::Plain, PlantVariant::Tombstoned],
        Arm::DeathRecordForEveryKey => vec![PlantVariant::Plain, PlantVariant::WithoutItsDeathRecord],
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PositiveControl {
    pub(super) kind: &'static str,
    pub(super) arm: Arm,
    pub(super) reader: Reader,
    pub(super) tree: ReportedTree,
    pub(super) expected: u64,
    /// `None`：这一格取不到可植的 key（C13），不算对上也不算对不上。
    pub(super) measured: Option<u64>,
}

pub(super) fn planted_unit(owner: UnitOwner, class: u64, group: Option<SelectionGroup>, birth: u64, slot: u64, mapping_key: Option<KeyFields>, contents: UnitContents) -> Unit {
    Unit {
        owner,
        class,
        group,
        write_order: (birth, MOUNT_INSTANCE, 0),
        slot,
        span_slots: 2,
        release_txg: None,
        is_readable: true,
        mapping_key,
        contents,
    }
}

/// 植入的数据单元（R-S、R-G 下 extent、分配记录、映射那三格）：身份全新（出生树取 `PLANTED_BIRTH_TREE`）。
fn planted_data_unit(object: u64, object_birth: u64, anchor_unit: u64, birth: u64, slot: u64, mapping_key: Option<KeyFields>) -> Unit {
    planted_unit(
        UnitOwner::FileData,
        UNIT_CLASS_DATA,
        Some(SelectionGroup::DataUnit {
            birth_tree: PLANTED_BIRTH_TREE,
            object,
            object_birth,
            anchor_unit,
        }),
        birth,
        slot,
        mapping_key,
        UnitContents::DataUnit {
            object,
            object_birth,
            anchor_unit,
        },
    )
}

/// 挑可植的 key 与造植入单元要的东西。
struct PlantingContext {
    births: BTreeMap<u64, u64>,
    tombstones: Vec<TombstoneRecordSummary>,
    /// 同一条臂下 R-S 重建出的 inode 树（R-G 门控看它，C13）。
    select_current_inodes: KeysWithProviders,
}

impl PlantingContext {
    fn tombstone_of(&self, object: u64, object_birth: u64) -> Option<TombstoneRecordSummary> {
        self.tombstones
            .iter()
            .find(|tombstone| tombstone.object == object && tombstone.object_birth == object_birth)
            .copied()
    }

    fn has_any_tombstone(&self, object: u64) -> bool {
        self.tombstones.iter().any(|tombstone| tombstone.object == object)
    }

    /// 植入数据单元用的对象（分配记录、映射那两格）：臂 B 有墓碑指认那一种取一条墓碑的对象；别的 R-S 取没人认的对象 0，
    /// R-G 取 R-S inode 树里一个没有墓碑的活对象（门控放得过）。
    fn object_for_a_planted_data_unit(&self, reader: Reader, variant: PlantVariant) -> Option<(u64, u64)> {
        match variant {
            PlantVariant::Tombstoned => self
                .tombstones
                .first()
                .map(|tombstone| (tombstone.object, tombstone.object_birth)),
            PlantVariant::Plain | PlantVariant::WithoutItsDeathRecord => match reader {
                Reader::SelectCurrentGatedByObject => self.select_current_inodes.keys().find_map(|fields| {
                    let inode = fields[0];
                    let birth = *self.births.get(&inode)?;
                    (!self.has_any_tombstone(inode)).then_some((inode, birth))
                }),
                Reader::UnitScan | Reader::SelectCurrent => Some((0, 0)),
            },
        }
    }

    /// 墓碑那一维的要求：有墓碑指认那一种要有对 (对象, 出生代) 的墓碑；没有墓碑指认那一种在臂 B 下要没有。
    fn tombstone_requirement_holds(&self, arm: Arm, variant: PlantVariant, object: u64, object_birth: u64) -> bool {
        let tombstoned = self.tombstone_of(object, object_birth).is_some();
        match variant {
            PlantVariant::Tombstoned => tombstoned,
            PlantVariant::Plain => arm != Arm::TodaysTombstones || !self.has_any_tombstone(object),
            PlantVariant::WithoutItsDeathRecord => true,
        }
    }
}

/// 给一条已死的 key 造植入单元（5.5 PC1 的单元类）；造不出（这一格没有这一种）交回 `None`。
fn plant_for_a_dead_key(
    reader: Reader,
    namespace: Namespace,
    variant: PlantVariant,
    arm: Arm,
    key: KeyFields,
    death: u64,
    context: &PlantingContext,
) -> Option<Unit> {
    let birth = death.checked_sub(1)?;
    let without_group = |owner: UnitOwner, class: u64, contents: UnitContents| {
        planted_unit(owner, class, None, birth, 0, None, contents)
    };
    let object_birth_of = |inode: u64| context.births.get(&inode).copied();
    match (reader, namespace) {
        (Reader::UnitScan, Namespace::Extent) => {
            let object = key[0];
            let pointer_birth = match variant {
                PlantVariant::Tombstoned => context.tombstones.iter().find(|tombstone| tombstone.object == object)?.object_birth,
                PlantVariant::Plain | PlantVariant::WithoutItsDeathRecord => {
                    if arm == Arm::TodaysTombstones && context.has_any_tombstone(object) && variant == PlantVariant::Plain {
                        return None;
                    }
                    birth
                }
            };
            Some(without_group(
                UnitOwner::ExtentLowerNode,
                UNIT_CLASS_INDEX_NODE,
                UnitContents::ExtentRecords(vec![ExtentRecordSummary {
                    inode: object,
                    data_unit: key[1],
                    pointer_birth,
                }]),
            ))
        }
        (Reader::UnitScan, Namespace::Inode) => {
            let object_birth = object_birth_of(key[0])?;
            context
                .tombstone_requirement_holds(arm, variant, key[0], object_birth)
                .then(|| {
                    without_group(
                        UnitOwner::InodeContainer,
                        UNIT_CLASS_PACKED,
                        UnitContents::InodeRecords(vec![InodeRecordSummary {
                            inode: key[0],
                            object_birth,
                            nlink: 1,
                        }]),
                    )
                })
        }
        (Reader::UnitScan, Namespace::DeletedInodes) => {
            let tombstoned = context.has_any_tombstone(key[0]);
            let holds = match variant {
                PlantVariant::Tombstoned => tombstoned,
                PlantVariant::Plain => arm != Arm::TodaysTombstones || !tombstoned,
                PlantVariant::WithoutItsDeathRecord => true,
            };
            holds.then(|| without_group(UnitOwner::DeletedInodesRoot, UNIT_CLASS_INDEX_NODE, UnitContents::DeletedInodes(vec![key[0]])))
        }
        (Reader::UnitScan, Namespace::AllocationRecordsPresent | Namespace::AllocationRecordsAllocated) => {
            (variant != PlantVariant::Tombstoned).then(|| {
                without_group(UnitOwner::AllocationNode, UNIT_CLASS_INDEX_NODE, UnitContents::AllocationRecords(vec![(key[0], false)]))
            })
        }
        (Reader::UnitScan, Namespace::Accounting) => (variant != PlantVariant::Tombstoned)
            .then(|| without_group(UnitOwner::AccountingNode, UNIT_CLASS_INDEX_NODE, UnitContents::AccountingKeys(vec![key]))),
        (Reader::UnitScan, Namespace::CentralMapping) => (variant != PlantVariant::Tombstoned)
            .then(|| without_group(UnitOwner::MappingNode, UNIT_CLASS_INDEX_NODE, UnitContents::MappingKeys(vec![key]))),
        (Reader::SelectCurrent | Reader::SelectCurrentGatedByObject, Namespace::Inode | Namespace::DeletedInodes) => {
            let object_birth = object_birth_of(key[0])?;
            if !context.tombstone_requirement_holds(arm, variant, key[0], object_birth) {
                return None;
            }
            Some(planted_unit(
                UnitOwner::InodeContainer,
                UNIT_CLASS_PACKED,
                Some(SelectionGroup::PackedContainer {
                    birth_tree: PLANTED_BIRTH_TREE,
                    record_type: PACKED_TYPE_INODE,
                    container_number: key[0],
                    container_birth: birth,
                }),
                birth,
                0,
                None,
                UnitContents::InodeRecords(vec![InodeRecordSummary {
                    inode: key[0],
                    object_birth,
                    nlink: 0,
                }]),
            ))
        }
        (Reader::SelectCurrent | Reader::SelectCurrentGatedByObject, Namespace::Extent) => {
            let object = key[0];
            let object_birth = object_birth_of(object)?;
            if reader == Reader::SelectCurrentGatedByObject
                && !context.select_current_inodes.contains_key(&single_field_key(object))
            {
                return None;
            }
            if !context.tombstone_requirement_holds(arm, variant, object, object_birth) {
                return None;
            }
            Some(planted_data_unit(object, object_birth, key[1], birth, 0, None))
        }
        (Reader::SelectCurrent | Reader::SelectCurrentGatedByObject, Namespace::AllocationUnitsOfTheVersion) => {
            let (object, object_birth) = context.object_for_a_planted_data_unit(reader, variant)?;
            Some(planted_data_unit(object, object_birth, 0, birth, key[0], None))
        }
        (Reader::SelectCurrent | Reader::SelectCurrentGatedByObject, Namespace::CentralMappingUnitsOfTheVersion) => {
            let original_birth = key[2];
            if key[0] == UNIT_CLASS_DATA {
                let (object, object_birth) = context.object_for_a_planted_data_unit(reader, variant)?;
                Some(planted_data_unit(object, object_birth, 0, original_birth, 0, Some(key)))
            } else if key[0] == UNIT_CLASS_PACKED && variant != PlantVariant::Tombstoned {
                let (owner, record_type, contents) = if key[1] == TREE_IDENTIFIER_INODE {
                    (UnitOwner::InodeContainer, PACKED_TYPE_INODE, UnitContents::InodeRecords(Vec::new()))
                } else {
                    (UnitOwner::TombstoneContainer, PACKED_TYPE_TOMBSTONE, UnitContents::TombstoneRecords(Vec::new()))
                };
                Some(planted_unit(
                    owner,
                    UNIT_CLASS_PACKED,
                    Some(SelectionGroup::PackedContainer {
                        birth_tree: PLANTED_BIRTH_TREE,
                        record_type,
                        container_number: 0,
                        container_birth: original_birth,
                    }),
                    original_birth,
                    0,
                    Some(key),
                    contents,
                ))
            } else {
                None
            }
        }
        (
            Reader::UnitScan,
            Namespace::AllocationUnitsOfTheVersion | Namespace::CentralMappingUnitsOfTheVersion | Namespace::Tombstone,
        )
        | (
            Reader::SelectCurrent | Reader::SelectCurrentGatedByObject,
            Namespace::AllocationRecordsPresent
            | Namespace::AllocationRecordsAllocated
            | Namespace::Accounting
            | Namespace::CentralMapping
            | Namespace::Tombstone,
        ) => None,
    }
}

impl Simulation {
    fn counts_for_the_hand_calculation(&self, arm: Arm, key: &NamespacedKey, provider_birth: u64, tag: OccurrenceTag) -> bool {
        match arm {
            Arm::WithoutTombstones => true,
            Arm::TodaysTombstones => {
                let tombstones = self.tombstones_on_disk();
                !object_of_tag(tag).is_some_and(|object| {
                    let of_the_object: Vec<TombstoneRecordSummary> =
                        tombstones.iter().filter(|tombstone| tombstone.object == object).copied().collect();
                    tombstone_suppresses(&of_the_object, tag)
                })
            }
            Arm::DeathRecordForEveryKey => !death_record_suppresses(&self.death_time, key, provider_birth),
        }
    }

    fn fallback_in_the_group(&self, erased: UnitIdentifier, candidates: &[UnitIdentifier]) -> Option<UnitIdentifier> {
        let group = self.disk.units[erased].group?;
        candidates
            .iter()
            .copied()
            .filter(|unit_identifier| *unit_identifier != erased && self.disk.units[*unit_identifier].is_readable && self.disk.units[*unit_identifier].group == Some(group))
            .max_by_key(|unit_identifier| self.disk.units[*unit_identifier].write_order)
    }

    /// PC2 的手算期望（C14）：抹掉的单元、它同组退回去的旧版本与基线读数里每条 key 有几个供出者，逐条算，不调扫描函数。
    fn erasure_for_the_hand_calculation(
        &self,
        reader: Reader,
        arm: Arm,
        namespace: Namespace,
        baseline: &ReadingResult,
    ) -> Option<(UnitIdentifier, u64)> {
        let truth = &self.truth[namespace.position()];
        let baseline_keys = &baseline.keys[namespace.position()];
        let only_provider = |fields: &KeyFields| baseline_keys.get(fields).is_some_and(|provider| provider.provider_count == 1);
        match reader {
            Reader::UnitScan => {
                let mut best: Option<(UnitIdentifier, u64)> = None;
                for (unit_identifier, unit) in self.disk.units.iter().enumerate() {
                    if !unit.is_readable || unit.release_txg.is_some() {
                        continue;
                    }
                    let live_keys = length_as_u64(
                        occurrences_of(&unit.contents)
                            .iter()
                            .filter(|occurrence| occurrence.key.namespace == namespace && truth.contains(&occurrence.key.fields))
                            .count(),
                    );
                    if live_keys > 0 && best.is_none_or(|(_, best_live)| live_keys > best_live) {
                        best = Some((unit_identifier, live_keys));
                    }
                }
                let (unit_identifier, _) = best?;
                let unit = &self.disk.units[unit_identifier];
                let expected = occurrences_of(&unit.contents)
                    .into_iter()
                    .filter(|occurrence| occurrence.key.namespace == namespace && truth.contains(&occurrence.key.fields))
                    .filter(|occurrence| self.counts_for_the_hand_calculation(arm, &occurrence.key, unit.birth_txg(), occurrence.tag))
                    .filter(|occurrence| only_provider(&occurrence.key.fields))
                    .count();
                Some((unit_identifier, length_as_u64(expected)))
            }
            Reader::SelectCurrent | Reader::SelectCurrentGatedByObject => {
                self.select_current_erasure(arm, namespace, baseline, truth)
            }
        }
    }

    fn select_current_erasure(
        &self,
        arm: Arm,
        namespace: Namespace,
        baseline: &ReadingResult,
        truth: &BTreeSet<KeyFields>,
    ) -> Option<(UnitIdentifier, u64)> {
        let baseline_keys = &baseline.keys[namespace.position()];
        let only_provider = |fields: &KeyFields| baseline_keys.get(fields).is_some_and(|provider| provider.provider_count == 1);
        let record_passes = |record: &InodeRecordSummary, provider_birth: u64| {
            let tag = OccurrenceTag::InodeRecord {
                inode: record.inode,
                object_birth: record.object_birth,
            };
            self.counts_for_the_hand_calculation(arm, &namespaced(namespace, single_field_key(record.inode)), provider_birth, tag)
        };
        match namespace {
            Namespace::Inode | Namespace::DeletedInodes => {
                let wants_queued_only = namespace == Namespace::DeletedInodes;
                let erased = if wants_queued_only {
                    let queued = *self.version.deleted_queue.first()?;
                    self.version
                        .inode_containers
                        .iter()
                        .find(|container| container.records.iter().any(|record| record.inode == queued))?
                        .unit
                } else {
                    self.version.inode_containers.iter().max_by_key(|container| container.records.len())?.unit
                };
                let fallback = self.fallback_in_the_group(erased, &self.unit_lists.inode_containers);
                let records_of = |unit_identifier: UnitIdentifier| {
                    if let UnitContents::InodeRecords(records) = &self.disk.units[unit_identifier].contents {
                        records.clone()
                    } else {
                        Vec::new()
                    }
                };
                let erased_birth = self.disk.units[erased].birth_txg();
                let fallback_records: Vec<(InodeRecordSummary, u64)> = fallback
                    .map(|unit_identifier| records_of(unit_identifier).into_iter().map(|record| (record, self.disk.units[unit_identifier].birth_txg())).collect())
                    .unwrap_or_default();
                let expected = records_of(erased)
                    .iter()
                    .filter(|record| !wants_queued_only || record.nlink == 0)
                    .filter(|record| record_passes(record, erased_birth))
                    .filter(|record| truth.contains(&single_field_key(record.inode)))
                    .filter(|record| only_provider(&single_field_key(record.inode)))
                    .filter(|record| {
                        !fallback_records.iter().any(|(older, older_birth)| {
                            older.inode == record.inode
                                && (!wants_queued_only || older.nlink == 0)
                                && record_passes(older, *older_birth)
                        })
                    })
                    .count();
                Some((erased, length_as_u64(expected)))
            }
            Namespace::Extent => {
                let erased = *self.version.file_units.first()?;
                let fields = extent_key(FIRST_FILE_INODE, 0);
                let fallback = self.fallback_in_the_group(erased, &self.unit_lists.data_units);
                let gate_holds = baseline.keys[Namespace::Inode.position()].contains_key(&single_field_key(FIRST_FILE_INODE));
                let data_passes = |unit_identifier: UnitIdentifier| {
                    let unit = &self.disk.units[unit_identifier];
                    let tag = OccurrenceTag::InodeRecord {
                        inode: FIRST_FILE_INODE,
                        object_birth: NEW_POOL_FILE_CREATION_TXG,
                    };
                    self.counts_for_the_hand_calculation(arm, &namespaced(Namespace::Extent, fields), unit.birth_txg(), tag)
                };
                let still_provided = fallback.is_some_and(data_passes) && gate_holds;
                let expected = u64::from(truth.contains(&fields) && only_provider(&fields) && !still_provided);
                Some((erased, expected))
            }
            Namespace::AllocationUnitsOfTheVersion | Namespace::CentralMappingUnitsOfTheVersion => {
                let erased = match self.version.file_units.first() {
                    Some(unit_identifier) => *unit_identifier,
                    None => self.version.inode_containers.first()?.unit,
                };
                let unit = &self.disk.units[erased];
                let fields = if namespace == Namespace::AllocationUnitsOfTheVersion {
                    single_field_key(unit.slot)
                } else {
                    unit.mapping_key?
                };
                Some((erased, u64::from(truth.contains(&fields) && only_provider(&fields))))
            }
            Namespace::Accounting => {
                let erased = *self.unit_lists.accounting_leaves.last()?;
                let expected = truth.iter().filter(|fields| only_provider(fields)).count();
                Some((erased, length_as_u64(expected)))
            }
            Namespace::AllocationRecordsPresent
            | Namespace::AllocationRecordsAllocated
            | Namespace::CentralMapping
            | Namespace::Tombstone => None,
        }
    }

    /// 最后一次扫描上，每条臂、每种读法、每棵有死亡的树各做 PC1、PC2（5.5）。
    pub(super) fn positive_controls(&self) -> Vec<PositiveControl> {
        let txg = self.version.txg;
        let mut outcomes = Vec::new();
        for arm in self.write_path.arms() {
            let base_inputs = ScanInputs {
                disk: &self.disk,
                unit_lists: &self.unit_lists,
                death_time: &self.death_time,
                planted: None,
                erased: None,
            };
            let context = PlantingContext {
                births: self.inode_births(),
                tombstones: self.tombstones_on_disk(),
                select_current_inodes: select_current_reading(Reader::SelectCurrent, arm, &base_inputs).keys
                    [Namespace::Inode.position()]
                .clone(),
            };
            for reader in READERS {
                let baseline = read_with(reader, arm, &base_inputs);
                for tree in REPORTED_TREES {
                    let Some(namespace) = tree.namespace(reader) else {
                        continue;
                    };
                    let has_deaths = self
                        .trajectories
                        .get(&(arm, reader, tree))
                        .is_some_and(|trajectory| trajectory.deaths_peak > 0);
                    if !has_deaths {
                        continue;
                    }
                    let position = namespace.position();
                    let truth = &self.truth[position];
                    let baseline_measurement = measure_against_truth(&baseline.keys[position], truth, txg);
                    let selects = reader != Reader::UnitScan;
                    if selects && namespace == Namespace::Accounting {
                        let planted = planted_unit(
                            UnitOwner::AccountingNode,
                            UNIT_CLASS_INDEX_NODE,
                            None,
                            txg + 1,
                            0,
                            None,
                            UnitContents::AccountingKeys(vec![[1, TREE_IDENTIFIER_INODE, STATISTIC_WITHOUT_DEVICE, txg + 1, 0]]),
                        );
                        let inputs = ScanInputs { planted: Some(&planted), ..base_inputs };
                        let measured = measure_against_truth(&read_with(reader, arm, &inputs).keys[position], truth, txg);
                        outcomes.push(PositiveControl {
                            kind: "PC1_newer_accounting_row",
                            arm,
                            reader,
                            tree,
                            expected: 1,
                            measured: Some(increase_or_marker(measured.resurrected, baseline_measurement.resurrected)),
                        });
                    } else {
                        for variant in plant_variants_of(arm) {
                            let planted = self.ever_seen[position]
                                .difference(truth)
                                .filter(|fields| !baseline.keys[position].contains_key(*fields))
                                .find_map(|fields| {
                                    let death = *self.death_time.get(&namespaced(namespace, *fields))?;
                                    plant_for_a_dead_key(reader, namespace, variant, arm, *fields, death, &context)
                                        .map(|unit| (unit, *fields))
                                });
                            let measured = planted.map(|(unit, fields)| {
                                let mut death_time_used = None;
                                if variant == PlantVariant::WithoutItsDeathRecord {
                                    let mut without = self.death_time.clone();
                                    without.remove(&namespaced(namespace, fields));
                                    death_time_used = Some(without);
                                }
                                let inputs = ScanInputs {
                                    planted: Some(&unit),
                                    death_time: death_time_used.as_ref().unwrap_or(&self.death_time),
                                    ..base_inputs
                                };
                                let with_the_plant = measure_against_truth(&read_with(reader, arm, &inputs).keys[position], truth, txg);
                                increase_or_marker(with_the_plant.resurrected, baseline_measurement.resurrected)
                            });
                            outcomes.push(PositiveControl {
                                kind: variant.name(),
                                arm,
                                reader,
                                tree,
                                expected: variant.expected_increase(arm),
                                measured,
                            });
                        }
                    }
                    if let Some((erased, expected)) = self.erasure_for_the_hand_calculation(reader, arm, namespace, &baseline) {
                        let inputs = ScanInputs { erased: Some(erased), ..base_inputs };
                        let after = measure_against_truth(&read_with(reader, arm, &inputs).keys[position], truth, txg);
                        outcomes.push(PositiveControl {
                            kind: "PC2",
                            arm,
                            reader,
                            tree,
                            expected,
                            measured: Some(increase_or_marker(after.missed, baseline_measurement.missed)),
                        });
                    }
                }
            }
        }
        outcomes
    }
}

#[cfg(test)]
#[path = "planting_for_positive_controls_tests.rs"]
mod tests;
