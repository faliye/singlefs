//! 阳性对照的计划、差量、阳性对照的结局，以及对应的模拟器步进。

use super::*;

// ---------------------------------------------------------------------------------------------
// 阳性对照（登记 5.5）：改读者输入、主路径与参照路径各算一遍增量，两路相同才算（第十一节 V2）
// ---------------------------------------------------------------------------------------------

/// 第二遍在哪几次扫描上做植入类的对照（第一遍的扫描记录定）：最后一次有变短过的活对象（PC-Z、PC-T）、
/// 最后一次在 unlink 途中（PC-EL）、最后一次最后一批之后紧接的那次（PC-ELr）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct PositiveControlPlan {
    shrunk_object_scan: Option<u64>,
    inside_unlink_scan: Option<u64>,
    after_final_batch_scan: Option<u64>,
}

pub(super) fn positive_control_plan_of(scans: &[ScanRecord]) -> PositiveControlPlan {
    let shrunk_object_scan = scans.iter().rev().find(|record| record.live_shrunk_object.is_some()).map(|record| record.publish_sequence);
    let inside_unlink_scan = scans.iter().rev().find(|record| record.inside_an_unlink).map(|record| record.publish_sequence);
    let after_final_batch_scan = scans
        .windows(2)
        .rev()
        .find(|adjacent| adjacent[0].inside_an_unlink && !adjacent[1].inside_an_unlink)
        .map(|adjacent| adjacent[1].publish_sequence);
    PositiveControlPlan { shrunk_object_scan, inside_unlink_scan, after_final_batch_scan }
}

pub(super) const COUNT_FIELDS: usize = 8;

const fn counts_as_array(counts: ReaderCounts) -> [u64; COUNT_FIELDS] {
    [
        counts.inode_resurrected,
        counts.inode_missed,
        counts.extent_resurrected,
        counts.extent_missed,
        counts.container_resurrected,
        counts.container_missed,
        counts.live_records_beyond_the_truth,
        counts.records_whose_intent_is_published,
    ]
}

/// 增量的下标：inode r、inode m、extent r、extent m、container r、container m、O1、O2。
pub(super) const DELTA_INODE_RESURRECTED: usize = 0;
pub(super) const DELTA_INODE_MISSED: usize = 1;
pub(super) const DELTA_EXTENT_RESURRECTED: usize = 2;
pub(super) const DELTA_EXTENT_MISSED: usize = 3;
pub(super) const DELTA_CONTAINER_RESURRECTED: usize = 4;
pub(super) const DELTA_CONTAINER_MISSED: usize = 5;
pub(super) const DELTA_LIVE_RECORDS_BEYOND_THE_TRUTH: usize = 6;
pub(super) const DELTA_RECORDS_WHOSE_INTENT_IS_PUBLISHED: usize = 7;

pub(super) fn signed(value: u64) -> i64 {
    i64::try_from(value).expect("计数装得进 i64")
}

fn deltas_of(before: &[ReaderCounts], after: &[ReaderCounts]) -> Vec<[i64; COUNT_FIELDS]> {
    before
        .iter()
        .zip(after)
        .map(|(baseline, modified)| {
            let baseline_values = counts_as_array(*baseline);
            let modified_values = counts_as_array(*modified);
            let mut delta = [0_i64; COUNT_FIELDS];
            for position in 0..COUNT_FIELDS {
                delta[position] = signed(modified_values[position]) - signed(baseline_values[position]);
            }
            delta
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PositiveControlOutcome {
    pub(super) control_identifier: &'static str,
    pub(super) at_publish: u64,
    pub(super) detail: String,
    /// 增量逐个对应的读者（第二、三段是 `every_reader` 的 20 个）。
    pub(super) readers: Vec<Reader>,
    pub(super) main_deltas: Vec<[i64; COUNT_FIELDS]>,
    pub(super) paths_agree: bool,
    pub(super) closed_form_holds: bool,
}

impl PositiveControlOutcome {
    pub(super) fn line(&self, context: &str) -> String {
        let per_reader: Vec<String> = self
            .readers
            .iter()
            .zip(&self.main_deltas)
            .map(|(reader, delta)| {
                format!(
                    "{}/{}/{}/{}:{}",
                    reader.base.label(),
                    reader.shrink.label(),
                    reader.effect.label(),
                    reader.window.label(),
                    delta.iter().map(i64::to_string).collect::<Vec<String>>().join("/")
                )
            })
            .collect();
        format!(
            "name=positive_control id={} {context} at_publish={} {} paths_agree={} closed_form_holds={} deltas_inode_r_m_extent_r_m_container_r_m_o1_o2={}",
            self.control_identifier,
            self.at_publish,
            self.detail,
            self.paths_agree,
            self.closed_form_holds,
            per_reader.join(",")
        )
    }
}

impl Simulation {
    /// 写序最大的植入（读法 9）。
    pub(super) const fn newest_write_order(&self) -> WriteOrder {
        WriteOrder { birth_txg: self.version.txg, instance: self.version.instance, transaction: self.version.transaction }
    }

    /// 植入的记录的改动计数取植入写序的诞生 txg（读法 11），P-C 下与 P-W 下都由它供出（写序最大、改动计数不小于盘上任何一条）。
    pub(super) fn planted_leaf(planted_record: InodeRecord, write_order: WriteOrder, scan_publish: u64) -> Unit {
        let record = InodeRecord {
            inode: planted_record.inode,
            object_birth: planted_record.object_birth,
            nlink: planted_record.nlink,
            size_bytes: planted_record.size_bytes,
            change_counter: write_order.birth_txg,
        };
        Unit {
            write_order,
            written_at_publish: scan_publish,
            replaced_at_publish: None,
            body: UnitBody::InodeLeaf {
                identity: ContainerIdentity {
                    birth_tree: PLANTED_BIRTH_TREE,
                    record_type: PACKED_TYPE_INODE,
                    number: record.inode,
                    birth: write_order.birth_txg,
                },
                records: Arc::new(vec![record]),
            },
        }
    }

    /// 一个对照：主路径（按组索引）与参照路径（整扫）各算改动前后，增量逐读者逐量相同才算两路对得上；`closed_form` 判登记写的期望。
    #[allow(clippy::type_complexity, reason = "这个元组类型只在这一处出现，起类型别名只多一个只用一次的名字")]
    pub(super) fn positive_control(
        &self,
        control_identifier: &'static str,
        overlay: &Overlay,
        scan_publish: u64,
        detail: String,
        closed_form: &dyn Fn(&[Reader], &[[i64; COUNT_FIELDS]]) -> bool,
    ) -> PositiveControlOutcome {
        self.positive_control_on(&self.readers.clone(), control_identifier, overlay, scan_publish, detail, closed_form)
    }

    /// 同 `positive_control`，读者表另给（PC-Bs2 在 B-stub 写路径上另读 B-stub-lin-off）。
    #[allow(clippy::type_complexity, reason = "这个元组类型只在这一处出现，起类型别名只多一个只用一次的名字")]
    fn positive_control_on(
        &self,
        readers: &[Reader],
        control_identifier: &'static str,
        overlay: &Overlay,
        scan_publish: u64,
        detail: String,
        closed_form: &dyn Fn(&[Reader], &[[i64; COUNT_FIELDS]]) -> bool,
    ) -> PositiveControlOutcome {
        let baseline_main = self.read_readers(readers, &Overlay::default(), scan_publish, SelectionPath::GroupIndex);
        let baseline_reference = self.read_readers(readers, &Overlay::default(), scan_publish, SelectionPath::FullScan);
        let modified_main = self.read_readers(readers, overlay, scan_publish, SelectionPath::GroupIndex);
        let modified_reference = self.read_readers(readers, overlay, scan_publish, SelectionPath::FullScan);
        let main_deltas = deltas_of(&baseline_main, &modified_main);
        let reference_deltas = deltas_of(&baseline_reference, &modified_reference);
        let paths_agree = baseline_main == baseline_reference && main_deltas == reference_deltas;
        let closed_form_holds = closed_form(readers, &main_deltas);
        PositiveControlOutcome { control_identifier, at_publish: scan_publish, detail, readers: readers.to_vec(), main_deltas, paths_agree, closed_form_holds }
    }

    /// PC-Z 的手算：k ∈ [o 现在的单元数, 变短前单元数) 里，那一组有一个候选版本（读得出、已发布、在谱系上）的个数，逐单元数、不走择新。
    fn shrunk_units_with_a_candidate(&self, object: u64, object_birth: u64, data_units_range: (u64, u64), window: ReadableWindow) -> u64 {
        let overlay = Overlay::default();
        let inputs = self.scan_inputs(&overlay, self.publish_sequence);
        let mut found: BTreeSet<u64> = BTreeSet::new();
        for unit in &self.disk.units {
            if let Some((unit_object, unit_object_birth, unit_index_in_file)) = data_key_of(unit) {
                let in_range = unit_index_in_file >= data_units_range.0 && unit_index_in_file < data_units_range.1;
                if unit_object == object && unit_object_birth == object_birth && in_range && is_candidate(unit, &inputs, window) {
                    found.insert(unit_index_in_file);
                }
            }
        }
        length_as_u64(found.len())
    }

    pub(super) fn run_the_planned_positive_controls(&mut self, publish_sequence: u64) {
        let plan = self.positive_control_plan;
        let mut outcomes = Vec::new();
        if plan.shrunk_object_scan == Some(publish_sequence) {
            outcomes.extend(self.shrink_positive_controls(publish_sequence));
        }
        if plan.inside_unlink_scan == Some(publish_sequence) {
            outcomes.extend(self.unlink_effect_positive_control(publish_sequence));
        }
        if plan.after_final_batch_scan == Some(publish_sequence) {
            outcomes.extend(self.deferred_effect_positive_control(publish_sequence));
        }
        self.positive_control_outcomes.extend(outcomes);
    }

    /// PC-Z 与 PC-T（登记 5.5）。
    fn shrink_positive_controls(&self, scan_publish: u64) -> Vec<PositiveControlOutcome> {
        let Some(object) = self.live_shrunk_object() else {
            return Vec::new();
        };
        let Some(record) = self.version.record_of(object) else {
            return Vec::new();
        };
        let size_before = self.largest_size_before_a_shrink.get(&object).copied().unwrap_or(0);
        let range = (self.version.data_units_of(object), size_before.div_ceil(DATA_UNIT_PAYLOAD_BYTES));
        let hand_counts = [
            self.shrunk_units_with_a_candidate(object, record.object_birth, range, ReadableWindow::Forever),
            self.shrunk_units_with_a_candidate(object, record.object_birth, range, ReadableWindow::RingWindow),
        ];
        let planted = InodeRecord { inode: object, object_birth: record.object_birth, nlink: 1, size_bytes: size_before, change_counter: 0 };
        let overlay = Overlay::planting(vec![Self::planted_leaf(planted, self.newest_write_order(), scan_publish)]);
        let shrink_closed_form = |readers: &[Reader], deltas: &[[i64; COUNT_FIELDS]]| {
            readers.iter().zip(deltas).all(|(reader, delta)| {
                let extent_expected = match reader.shrink {
                    ShrinkArm::RecordSize => match reader.window {
                        ReadableWindow::Forever => signed(hand_counts[0]),
                        ReadableWindow::RingWindow => signed(hand_counts[1]),
                    },
                    ShrinkArm::RangeTombstone | ShrinkArm::ObjectGateOnly => 0,
                };
                delta[DELTA_CONTAINER_RESURRECTED] == 1 && delta[DELTA_EXTENT_RESURRECTED] == extent_expected
            })
        };
        let mut outcomes = vec![self.positive_control(
            "PC-Z",
            &overlay,
            scan_publish,
            format!("object={object} data_units_now={} data_units_before_shrink={} hand_count_inf={} hand_count_23={}", range.0, range.1, hand_counts[0], hand_counts[1]),
            &shrink_closed_form,
        )];
        let range_tombstone = self.version.tombstone_trees[TombstoneStream::WholeAtIntentRangesOnUserShrinks.position()]
            .containers
            .iter()
            .flat_map(|container| container.records.iter().copied())
            .rfind(|tombstone| tombstone.object == object && tombstone.object_birth == record.object_birth && tombstone.range_start_bytes > 0);
        if let Some(removed) = range_tombstone {
            let tombstone_removing_overlay = Overlay::removing_a_tombstone(removed);
            let range_closed_form = |readers: &[Reader], deltas: &[[i64; COUNT_FIELDS]]| {
                readers.iter().zip(deltas).all(|(reader, delta)| reader.shrink == ShrinkArm::RangeTombstone || delta[DELTA_EXTENT_RESURRECTED] == 0)
            };
            outcomes.push(self.positive_control(
                "PC-T",
                &tombstone_removing_overlay,
                scan_publish,
                format!("object={object} removed_range_start={} removed_death_txg={}", removed.range_start_bytes, removed.death_txg),
                &range_closed_form,
            ));
        }
        outcomes
    }

    /// PC-EL：unlink 途中植入一个全新身份的墓碑容器（各条写流各一个），装 (N, b, [0, 2⁶⁴ − 1], d = 这一版 txg)。
    fn unlink_effect_positive_control(&self, scan_publish: u64) -> Vec<PositiveControlOutcome> {
        let Some(object) = self.version.deleted_queue else {
            return Vec::new();
        };
        let Some(record) = self.version.record_of(object) else {
            return Vec::new();
        };
        let remaining = signed(self.version.data_units_of(object));
        let whole = TombstoneRecord {
            object,
            object_birth: record.object_birth,
            range_start_bytes: WHOLE_OBJECT_RANGE_START_BYTES,
            death_txg: self.version.txg,
        };
        let planted = TOMBSTONE_STREAMS
            .iter()
            .map(|stream| Unit {
                write_order: self.newest_write_order(),
                written_at_publish: scan_publish,
                replaced_at_publish: None,
                body: UnitBody::Tombstones {
                    stream: *stream,
                    identity: ContainerIdentity { birth_tree: PLANTED_BIRTH_TREE, record_type: PACKED_TYPE_TOMBSTONE, number: 1, birth: self.version.txg },
                    records: Arc::new(vec![whole]),
                },
            })
            .collect();
        let overlay = Overlay::planting(planted);
        let closed_form = |readers: &[Reader], deltas: &[[i64; COUNT_FIELDS]]| {
            readers.iter().zip(deltas).all(|(reader, delta)| match reader.effect {
                UnlinkEffectArm::AtLastBatchWritten => delta[DELTA_INODE_MISSED] == 1 && delta[DELTA_EXTENT_MISSED] == remaining,
                UnlinkEffectArm::AtIntent | UnlinkEffectArm::AtLastBatchRead => delta[DELTA_INODE_MISSED] == 0 && delta[DELTA_EXTENT_MISSED] == 0,
            })
        };
        vec![self.positive_control("PC-EL", &overlay, scan_publish, format!("object={object} data_units_remaining={remaining}"), &closed_form)]
    }

    /// PC-ELr：最后一批之后紧接的那次扫描，植入全新身份、已发布、在谱系上、诞生 txg < N 的意图 txg 的类型 2 容器版本，装 (N, b, nlink 1)。
    fn deferred_effect_positive_control(&self, scan_publish: u64) -> Vec<PositiveControlOutcome> {
        let Some(record) = self.publish_records.last() else {
            return Vec::new();
        };
        let unlinked = self
            .intent_txgs
            .iter()
            .filter(|(object, _)| !self.version.is_live(**object) && self.version.record_of(**object).is_none())
            .max_by_key(|(_, intent_txg)| **intent_txg)
            .map(|(object, intent_txg)| (*object, *intent_txg));
        let Some((object, intent_txg)) = unlinked else {
            return Vec::new();
        };
        let Some(object_birth) = self.object_birth_seen_on_disk(object) else {
            return Vec::new();
        };
        let write_order = WriteOrder { birth_txg: intent_txg - 1, instance: self.version.instance, transaction: 0 };
        let planted = InodeRecord { inode: object, object_birth, nlink: 1, size_bytes: 0, change_counter: 0 };
        let overlay = Overlay::planting(vec![Self::planted_leaf(planted, write_order, scan_publish)]);
        let closed_form = |readers: &[Reader], deltas: &[[i64; COUNT_FIELDS]]| {
            readers.iter().zip(deltas).all(|(reader, delta)| {
                reader.window == ReadableWindow::RingWindow
                    || match reader.effect {
                        UnlinkEffectArm::AtLastBatchRead => delta[DELTA_INODE_RESURRECTED] == 1,
                        UnlinkEffectArm::AtIntent | UnlinkEffectArm::AtLastBatchWritten => delta[DELTA_INODE_RESURRECTED] == 0,
                    }
            })
        };
        vec![self.positive_control(
            "PC-ELr",
            &overlay,
            scan_publish,
            format!("object={object} intent_txg={intent_txg} planted_birth_txg={} last_publish_txg={}", intent_txg - 1, record.txg),
            &closed_form,
        )]
    }

    fn object_birth_seen_on_disk(&self, object: u64) -> Option<u64> {
        self.disk.units.iter().rev().find_map(|unit| match &unit.body {
            UnitBody::InodeLeaf { records, .. } => records.iter().find(|record| record.inode == object).map(|record| record.object_birth),
            UnitBody::Data { .. } | UnitBody::Tombstones { .. } | UnitBody::Retirements { .. } => None,
        })
    }

    /// PC-M：抹掉一个现行的活数据单元（取组里只有它一版的那个，没有就取号最小的），再另做一次抹掉现行的第一片叶容器。
    pub(super) fn erasure_positive_controls(&self, scan_publish: u64) -> Vec<PositiveControlOutcome> {
        let mut outcomes = Vec::new();
        let live_data_units: Vec<UnitIndex> = self
            .version
            .files
            .iter()
            .filter(|(object, _)| self.version.is_live(**object))
            .flat_map(|(_, file)| file.units.iter().copied())
            .collect();
        let single_version = live_data_units.iter().copied().find(|unit_index| {
            let key = self.disk.units[*unit_index].body.group_key();
            self.disk.group_of_key.get(&key).is_some_and(|group| self.disk.versions_of_group[*group].len() == 1)
        });
        if let Some(erased) = single_version.or_else(|| live_data_units.first().copied()) {
            let overlay = Overlay::erasing(erased);
            let closed_form = |_: &[Reader], deltas: &[[i64; COUNT_FIELDS]]| deltas.iter().all(|delta| delta[DELTA_EXTENT_MISSED] <= 1);
            outcomes.push(self.positive_control("PC-M-data", &overlay, scan_publish, format!("erased_unit={erased} single_version={}", single_version.is_some()), &closed_form));
        }
        if let Some(leaf) = self.version.leaves.first() {
            let overlay = Overlay::erasing(leaf.unit);
            let records = signed(length_as_u64(leaf.records.len()));
            let closed_form = |_: &[Reader], deltas: &[[i64; COUNT_FIELDS]]| deltas.iter().all(|delta| delta[DELTA_INODE_MISSED] <= records);
            outcomes.push(self.positive_control("PC-M-leaf", &overlay, scan_publish, format!("erased_unit={} records={records}", leaf.unit), &closed_form));
        }
        outcomes
    }

    /// 第十一节 V3：这次扫描上主路径与参照路径逐读者逐量相同。
    pub(super) fn group_index_equals_the_full_scan(&self, scan_publish: u64) -> bool {
        self.read_every_reader(&Overlay::default(), scan_publish, SelectionPath::GroupIndex)
            == self.read_every_reader(&Overlay::default(), scan_publish, SelectionPath::FullScan)
    }
}

#[cfg(test)]
#[path = "positive_controls_and_deltas_tests.rs"]
mod tests;
