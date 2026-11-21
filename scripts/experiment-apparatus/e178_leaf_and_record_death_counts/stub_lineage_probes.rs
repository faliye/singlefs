//! 带桩那几条臂的计划与探针，以及对应的模拟器步进。

use super::*;

// ---------------------------------------------------------------------------------------------
// 第四段（登记修订 2）：B-stub、B-tsz 两条底座臂；Q-Bs-1 至 Q-Bs-6、Q-Bt-1 至 Q-Bt-4；分片、续跑与多机
// ---------------------------------------------------------------------------------------------

/// PC-Bt1 的取样：一个身份在对象 N 的最后一批那次消失，它最后那一版装着 N（nlink = 0）的记录。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct DeadVersionHoldingAFinishedRecord {
    pub(super) object: u64,
    pub(super) object_birth: u64,
    intent_txg: u64,
    last_version: UnitIndex,
}

/// 第四段在第二遍上做对照的扫描（第一遍的探针记下每种情形最后出现的那一次）；下标是 W 那一档（0 是 ∞，1 是 23）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct SegmentFourPlan {
    /// PC-Bt1：有一个已删对象的记录还在一个读得出的死身份最后那一版里。
    dead_record_scan: [Option<u64>; 2],
    /// PC-Bs1：现行那一版里有桩，且那个身份最后一个非空版本是候选。
    stub_scan: [Option<u64>; 2],
    /// PC-Bt2-mid（登记修订 3）：有一个被挪过、之后变短的活对象（W = ∞）。
    moved_then_shortened_scan: Option<u64>,
}

#[derive(Clone, Debug, Default)]
pub(super) struct SegmentFourProbes {
    pub(super) plan: SegmentFourPlan,
    pub(super) observed: SegmentFourPlan,
    pub(super) outcomes: Vec<PositiveControlOutcome>,
    /// 这一段做不做 PC-Bt2 与 PC-Bt2-mid（原写路径上的 F10、θ > 0）。
    pub(super) reversed_provenance_applies: bool,
}

const fn window_slot(window: ReadableWindow) -> usize {
    match window {
        ReadableWindow::Forever => 0,
        ReadableWindow::RingWindow => 1,
    }
}

/// 一个组里写序最大的候选版本（主路径择新的同一条规则，给第四段的取样与手算用）。
fn newest_candidate_of_the_group(disk: &Disk, key: GroupKey, inputs: &ScanInputs<'_>, window: ReadableWindow) -> Option<UnitIndex> {
    let group = disk.group_of_key.get(&key)?;
    disk.versions_of_group[*group].iter().rev().copied().find(|unit_index| is_candidate(&disk.units[*unit_index], inputs, window))
}

impl Simulation {
    /// 最后一批那次消失的身份里，最后那一版装着这个对象记录的，记下来（PC-Bt1 用）。在换版之前调，`self.version` 还是上一版。
    pub(super) fn note_dead_versions_holding_the_finished_record(&mut self, object: u64, retired: &[ContainerIdentity], stubbed: &[ContainerIdentity]) {
        let Some(intent_txg) = self.intent_txgs.get(&object).copied() else {
            return;
        };
        for identity in retired.iter().chain(stubbed) {
            let Some(leaf) = self.version.leaves.iter().find(|leaf| leaf.identity == *identity) else {
                continue;
            };
            if let Ok(position) = leaf.records.binary_search_by_key(&object, |record| record.inode) {
                let record = leaf.records[position];
                self.dead_versions_holding_a_finished_record.push(DeadVersionHoldingAFinishedRecord {
                    object,
                    object_birth: record.object_birth,
                    intent_txg,
                    last_version: leaf.unit,
                });
            }
        }
    }

    /// PC-Bt1 的取样：对象已不在树里、身份不在现行叶序里、那一版是它那一组写序最大的候选；有几个取意图 txg 最大的。
    fn finished_record_in_a_readable_dead_version(&self, inputs: &ScanInputs<'_>, window: ReadableWindow) -> Option<DeadVersionHoldingAFinishedRecord> {
        let live_identities: HashSet<ContainerIdentity> = self.version.leaves.iter().map(|leaf| leaf.identity).collect();
        self.dead_versions_holding_a_finished_record
            .iter()
            .copied()
            .filter(|dead| self.version.record_of(dead.object).is_none())
            .filter(|dead| {
                let UnitBody::InodeLeaf { identity, .. } = &self.disk.units[dead.last_version].body else {
                    return false;
                };
                !live_identities.contains(identity)
                    && newest_candidate_of_the_group(&self.disk, GroupKey::InodeLeaf(*identity), inputs, window) == Some(dead.last_version)
            })
            .max_by_key(|dead| (dead.intent_txg, dead.object))
    }

    /// PC-Bs1 的取样：现行叶序里的桩，它那一组里桩之前最后一个非空版本是候选；有几个取容器号最小的。交回 (桩的单元, 那个非空版本)。
    fn stub_with_a_candidate_last_record_version(&self, inputs: &ScanInputs<'_>, window: ReadableWindow) -> Option<(UnitIndex, UnitIndex)> {
        let mut stubs: Vec<&Leaf> = self.version.leaves.iter().filter(|leaf| leaf.records.is_empty()).collect();
        stubs.sort_by_key(|leaf| (leaf.identity.number, leaf.identity.birth));
        stubs.into_iter().find_map(|stub| {
            let group = self.disk.group_of_key.get(&GroupKey::InodeLeaf(stub.identity))?;
            let last_record_version = self.disk.versions_of_group[*group].iter().rev().copied().find(|unit_index| {
                matches!(&self.disk.units[*unit_index].body, UnitBody::InodeLeaf { records, .. } if !records.is_empty())
            })?;
            is_candidate(&self.disk.units[last_record_version], inputs, window).then_some((stub.unit, last_record_version))
        })
    }

    /// 第一遍记下取样的扫描，第二遍在计划的那几次扫描上做 PC-Bt1（原写路径）与 PC-Bs1（B-stub 写路径）。
    pub(super) fn run_the_segment_four_probes(&mut self, publish_sequence: u64) {
        let overlay = Overlay::default();
        let mut dead_record_seen = [false; 2];
        let mut stub_seen = [false; 2];
        {
            let inputs = self.scan_inputs(&overlay, publish_sequence);
            for window in READABLE_WINDOWS {
                let slot = window_slot(window);
                dead_record_seen[slot] = self.finished_record_in_a_readable_dead_version(&inputs, window).is_some();
                stub_seen[slot] = self.stub_with_a_candidate_last_record_version(&inputs, window).is_some();
            }
        }
        let moved_then_shortened_object_is_live = self.moved_then_shortened_objects.iter().any(|object| self.version.is_live(*object));
        let (plan, reversed_provenance_applies) = {
            let probes = self.segment_four.as_mut().expect("第四段的模拟带着探针");
            for slot in 0..2 {
                if dead_record_seen[slot] {
                    probes.observed.dead_record_scan[slot] = Some(publish_sequence);
                }
                if stub_seen[slot] {
                    probes.observed.stub_scan[slot] = Some(publish_sequence);
                }
            }
            if moved_then_shortened_object_is_live {
                probes.observed.moved_then_shortened_scan = Some(publish_sequence);
            }
            (probes.plan, probes.reversed_provenance_applies)
        };
        let mut outcomes = Vec::new();
        if reversed_provenance_applies && plan.moved_then_shortened_scan == Some(publish_sequence) {
            let mut control = self.reversed_provenance_positive_control(publish_sequence);
            control.control_identifier = "PC-Bt2-mid";
            outcomes.push(control);
        }
        for window in READABLE_WINDOWS {
            let slot = window_slot(window);
            if plan.dead_record_scan[slot] == Some(publish_sequence) && self.container_death == ContainerDeathWritePath::RetirementRecords {
                outcomes.extend(self.dead_record_positive_control(publish_sequence, window));
            }
            if plan.stub_scan[slot] == Some(publish_sequence) && self.container_death == ContainerDeathWritePath::ZeroRecordStubs {
                outcomes.extend(self.stub_erasure_positive_control(publish_sequence, window));
            }
        }
        self.segment_four.as_mut().expect("第四段的模拟带着探针").outcomes.extend(outcomes);
    }

    /// 这个读者那条墓碑写流里有没有一条在谱系上的 (o, b) 整对象墓碑；有就交回死亡 txg（手算用，与读者那一段分开写）。
    fn whole_object_death_on_the_chain(&self, inputs: &ScanInputs<'_>, stream: TombstoneStream, object: u64, object_birth: u64) -> Option<u64> {
        self.version.tombstone_trees[stream.position()]
            .containers
            .iter()
            .flat_map(|container| container.records.iter())
            .filter(|tombstone| {
                tombstone.object == object
                    && tombstone.object_birth == object_birth
                    && tombstone.range_start_bytes == WHOLE_OBJECT_RANGE_START_BYTES
                    && inputs.chain.contains(tombstone.death_txg)
            })
            .map(|tombstone| tombstone.death_txg)
            .max()
    }

    /// PC-Bs1 的手算（inode r）：抹掉桩之后读回的那个非空版本里，(o, b) 不在真值里、别处没有写序更大的副本、没被这个读者的整对象
    /// 墓碑压掉的记录条数（E-I、E-L-w 施行；E-L-r 的记录又回到树里、不施行）。
    fn hand_count_of_records_a_stub_erasure_brings_back(&self, inputs: &ScanInputs<'_>, last_record_version: UnitIndex, reader: Reader) -> u64 {
        let UnitBody::InodeLeaf { identity: fallen_back_identity, records } = &self.disk.units[last_record_version].body else {
            return 0;
        };
        let fallen_back_order = (self.disk.units[last_record_version].write_order, last_record_version);
        let truth = truth_of(&self.version);
        let selected = select_by_a_full_scan(inputs, reader.window, InodeLeafLineage::Required);
        let newer_copy_exists = |inode: u64| {
            selected.inode_leaves.iter().any(|unit_index| match &self.disk.units[*unit_index].body {
                UnitBody::InodeLeaf { identity, records: other_records } => {
                    identity != fallen_back_identity
                        && (self.disk.units[*unit_index].write_order, *unit_index) > fallen_back_order
                        && other_records.binary_search_by_key(&inode, |record| record.inode).is_ok()
                }
                UnitBody::Data { .. } | UnitBody::Tombstones { .. } | UnitBody::Retirements { .. } => false,
            })
        };
        let stream = reader.tombstone_stream();
        let suppressed = |record: &InodeRecord| match reader.effect {
            UnlinkEffectArm::AtIntent | UnlinkEffectArm::AtLastBatchWritten => {
                self.whole_object_death_on_the_chain(inputs, stream, record.inode, record.object_birth).is_some()
            }
            UnlinkEffectArm::AtLastBatchRead => false,
        };
        length_as_u64(
            records
                .iter()
                .filter(|record| !truth.inode_keys.contains(&(record.inode, record.object_birth)))
                .filter(|record| !newer_copy_exists(record.inode))
                .filter(|record| !suppressed(record))
                .count(),
        )
    }

    /// PC-Bs1：抹掉一个桩（读者回落到那个身份最后一个非空版本）。同一档 W 的 B-stub 读者：容器 r + 1，inode r = 手算值。
    fn stub_erasure_positive_control(&self, scan_publish: u64, window: ReadableWindow) -> Vec<PositiveControlOutcome> {
        let overlay = Overlay::default();
        let inputs = self.scan_inputs(&overlay, scan_publish);
        let Some((stub_unit, last_record_version)) = self.stub_with_a_candidate_last_record_version(&inputs, window) else {
            return Vec::new();
        };
        let hand_counts: Vec<u64> = self
            .readers
            .iter()
            .map(|reader| self.hand_count_of_records_a_stub_erasure_brings_back(&inputs, last_record_version, *reader))
            .collect();
        let closed_form = |readers: &[Reader], deltas: &[[i64; COUNT_FIELDS]]| {
            readers.iter().zip(deltas).zip(&hand_counts).all(|((reader, delta), hand)| {
                reader.window != window || (delta[DELTA_CONTAINER_RESURRECTED] == 1 && delta[DELTA_INODE_RESURRECTED] == signed(*hand))
            })
        };
        vec![self.positive_control(
            "PC-Bs1",
            &Overlay::erasing(stub_unit),
            scan_publish,
            format!(
                "window_taken={} erased_stub_unit={stub_unit} last_record_version={last_record_version} hand_inode_r={}",
                window.label(),
                hand_counts.iter().map(u64::to_string).collect::<Vec<String>>().join("/")
            ),
            &closed_form,
        )]
    }

    /// PC-Bt1：拿掉对象 N 的整对象墓碑；N 的 nlink = 0 记录还在一个读得出的死身份最后那一版里。同一档 W 的 B-tsz 读者 inode r + 1，
    /// 同一档 W 的 B-on × E-I 读者 inode r、extent r 都 + 0（那一版被退役记录丢掉）。
    fn dead_record_positive_control(&self, scan_publish: u64, window: ReadableWindow) -> Vec<PositiveControlOutcome> {
        let overlay = Overlay::default();
        let inputs = self.scan_inputs(&overlay, scan_publish);
        let Some(dead) = self.finished_record_in_a_readable_dead_version(&inputs, window) else {
            return Vec::new();
        };
        let removed = TombstoneRecord {
            object: dead.object,
            object_birth: dead.object_birth,
            range_start_bytes: WHOLE_OBJECT_RANGE_START_BYTES,
            death_txg: dead.intent_txg,
        };
        let closed_form = |readers: &[Reader], deltas: &[[i64; COUNT_FIELDS]]| {
            readers.iter().zip(deltas).all(|(reader, delta)| {
                if reader.window != window {
                    return true;
                }
                if reader.base == BaseArm::TombstonesOnly {
                    return delta[DELTA_INODE_RESURRECTED] == 1;
                }
                if reader.base == BaseArm::RetirementOnLineage && reader.effect == UnlinkEffectArm::AtIntent {
                    return delta[DELTA_INODE_RESURRECTED] == 0 && delta[DELTA_EXTENT_RESURRECTED] == 0;
                }
                true
            })
        };
        vec![self.positive_control(
            "PC-Bt1",
            &Overlay::removing_a_tombstone(removed),
            scan_publish,
            format!("window_taken={} object={} intent_txg={} dead_version={}", window.label(), dead.object, dead.intent_txg, dead.last_version),
            &closed_form,
        )]
    }

    /// PC-Bt2 的手算（B-tsz × Z × E-I × W = ∞）：供出规则由 P-C 换成「写序最小」之后 extent r 的增量——按单元逐个比两条规则选出的
    /// 记录（对象出生代与 size 折成的单元数），不在真值里、又没被整对象墓碑压掉的才算（两条规则各判一次，相减）。
    fn hand_count_of_units_the_oldest_copies_bring_back(&self, inputs: &ScanInputs<'_>) -> i64 {
        let selected = select_by_a_full_scan(inputs, ReadableWindow::Forever, InodeLeafLineage::Required);
        #[allow(clippy::type_complexity, reason = "这个元组类型只在这一处出现，起类型别名只多一个只用一次的名字")]
        let mut largest_change_counter: HashMap<u64, ((u64, (WriteOrder, UnitIndex)), InodeRecord)> = HashMap::new();
        let mut oldest_write_order: HashMap<u64, ((WriteOrder, UnitIndex), InodeRecord)> = HashMap::new();
        for unit_index in &selected.inode_leaves {
            let UnitBody::InodeLeaf { records, .. } = &self.disk.units[*unit_index].body else {
                continue;
            };
            let order = (self.disk.units[*unit_index].write_order, *unit_index);
            for record in records.iter() {
                let change_key = (record.change_counter, order);
                let replaces_the_chosen = largest_change_counter.get(&record.inode).is_none_or(|(existing, _)| change_key > *existing);
                if replaces_the_chosen {
                    largest_change_counter.insert(record.inode, (change_key, *record));
                }
                let replaces_the_oldest = oldest_write_order.get(&record.inode).is_none_or(|(existing, _)| order < *existing);
                if replaces_the_oldest {
                    oldest_write_order.insert(record.inode, (order, *record));
                }
            }
        }
        let truth = truth_of(&self.version);
        let stream = tombstone_stream_of(ShrinkArm::RecordSize, UnlinkEffectArm::AtIntent);
        let mut delta = 0_i64;
        for unit_index in &selected.data {
            let data_unit = &self.disk.units[*unit_index];
            let Some((object, object_birth, unit_index_in_file)) = data_key_of(data_unit) else {
                continue;
            };
            if truth.extent_keys.contains(&(object, object_birth, unit_index_in_file)) {
                continue;
            }
            let killed = self
                .whole_object_death_on_the_chain(inputs, stream, object, object_birth)
                .is_some_and(|death_txg| data_unit.write_order.birth_txg <= death_txg);
            if killed {
                continue;
            }
            let alive_under = |record: Option<&InodeRecord>| {
                record.is_some_and(|record| record.object_birth == object_birth && record.data_units() > unit_index_in_file)
            };
            let alive_now = alive_under(largest_change_counter.get(&object).map(|(_, record)| record));
            let alive_reversed = alive_under(oldest_write_order.get(&object).map(|(_, record)| record));
            delta += i64::from(alive_reversed) - i64::from(alive_now);
        }
        delta
    }

    /// PC-Bt2：F10、θ > 0 每段最后一次扫描（W = ∞），供出规则倒过来另读一遍。B-tsz × Z 的 extent r 增量 = 手算值，B-tsz × T 的 + 0，
    /// 两者的 inode r + 0（修订 2 之六）。
    pub(super) fn reversed_provenance_positive_control(&self, scan_publish: u64) -> PositiveControlOutcome {
        let overlay = Overlay::default();
        let inputs = self.scan_inputs(&overlay, scan_publish);
        let hand = self.hand_count_of_units_the_oldest_copies_bring_back(&inputs);
        let closed_form = |readers: &[Reader], deltas: &[[i64; COUNT_FIELDS]]| {
            readers.iter().zip(deltas).all(|(reader, delta)| {
                if reader.base != BaseArm::TombstonesOnly || reader.window != ReadableWindow::Forever {
                    return true;
                }
                let extent_expected = match reader.shrink {
                    ShrinkArm::RecordSize => hand,
                    ShrinkArm::RangeTombstone | ShrinkArm::ObjectGateOnly => 0,
                };
                delta[DELTA_EXTENT_RESURRECTED] == extent_expected && delta[DELTA_INODE_RESURRECTED] == 0
            })
        };
        self.positive_control("PC-Bt2", &Overlay::reversing_the_provenance(), scan_publish, format!("hand_extent_r_z={hand}"), &closed_form)
    }
}

#[cfg(test)]
#[path = "stub_lineage_probes_tests.rs"]
mod tests;
