//! 内存盘（读者）、新根段与时间线上的一版、生成一条记录槽、点名单元（U 域）与合法历史的生成器（可信件，登记 5.1）。

use super::*;

// ── 内存盘（读者）

/// 内存盘某个位置上放着什么：读得出的字节，或一份读不出的（`read` 返回 `None`）。
#[derive(Clone, Copy)]
pub(super) enum StoredBytes<'bytes> {
    Readable(&'bytes [u8]),
    Unreadable,
}

/// `replay_journal` 的读者：两块盘；环里没落盘的记录槽读出来是 4096 个 0（登记 5.1），别处没写过的读不出。
/// `device_identities()` 报哪块盘在前由 `reported_disk_order` 定（登记 r2 第八节「盘次序」）。
pub(super) struct MemoryDisk<'bytes> {
    pub(super) ring: RingGeometry,
    pub(super) reported_disk_order: ReportedDiskOrder,
    pub(super) contents: Vec<(u32, u64, StoredBytes<'bytes>)>,
}

impl PoolReader for MemoryDisk<'_> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        self.reported_disk_order
            .device_numbers()
            .into_iter()
            .map(DeviceIdentity)
            .collect()
    }

    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        (device.0 < 2).then_some(MEMORY_DISK_DEVICE_BYTES)
    }

    fn physical_block_size_probed_on_the_device(
        &self,
        _device: DeviceIdentity,
    ) -> Option<PhysicalBlockSizeInBytes> {
        unreachable!("replay_journal 不挂载、不探测块宽")
    }

    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        for (stored_device, stored_offset, stored) in &self.contents {
            if *stored_device == device.0 && *stored_offset == offset.0 {
                return match stored {
                    StoredBytes::Readable(bytes) => bytes.get(..length).map(<[u8]>::to_vec),
                    StoredBytes::Unreadable => None,
                };
            }
        }
        let ring_start = RingGeometry::ring_start_offset_in_bytes();
        let is_a_whole_record_slot_of_the_ring = device.0 < 2
            && offset.0 >= ring_start
            && offset.0 < ring_start + self.ring.ring_bytes()
            && (offset.0 - ring_start).is_multiple_of(LOCAL_JOURNAL_RECORD_BYTES)
            && u64::try_from(length).ok() == Some(LOCAL_JOURNAL_RECORD_BYTES);
        is_a_whole_record_slot_of_the_ring.then(|| vec![0u8; length])
    }

    fn journal_record_offsets_hint(
        &self,
        _device: DeviceIdentity,
        _ring_start: DeviceOffsetInBytes,
        _ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        None
    }
}

// ── 新根段与时间线上的一版

/// 记录头里的新根段四个字段（D23（journal 的角色与格式） 已定项 15）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct NewRootSegment {
    pub(super) tree_table: NodePointer,
    pub(super) mapping_root: NodePointer,
    pub(super) tree_identifier_watermark: u64,
    pub(super) rollback_floor: u64,
}

/// 一条过得了 `to_bytes` / `parse` 往返的节点指针：两条位置条目分在 0、1 号盘同一个槽，槽号取 `slot`。
pub(super) fn node_pointer_at_slot(slot: u64, birth_txg: u64) -> NodePointer {
    let location_on = |device: u32| LocationEntry {
        device: DeviceIdentity(device),
        slot: SlotNumber(slot),
        unit_checksum: u32::try_from(slot % 0xffff_fff1).expect("取过模，装得进 32 位"),
    };
    NodePointer {
        head: PointerHead {
            birth_tree: TreeIdentifier(11),
            birth_txg: CheckpointTxg(birth_txg),
        },
        locations: [location_on(0), location_on(1)],
        instance: InstanceGeneration(CHOSEN_INSTANCE),
        birth_sequence: BirthSequence(1),
    }
}

impl NewRootSegment {
    /// 时间线上第 `publish_ordinal` 次发布的新根段（登记 5.1：树 ID 水位 1000 + i，F 取 10 + i，两条指针两两不同）。
    pub(super) fn of_timeline_publish(publish_ordinal: u64, checkpoint_txg: u64) -> Self {
        Self {
            tree_table: node_pointer_at_slot(70_000 + publish_ordinal, checkpoint_txg),
            mapping_root: node_pointer_at_slot(80_000 + publish_ordinal, checkpoint_txg),
            tree_identifier_watermark: 1000 + publish_ordinal,
            rollback_floor: 10 + publish_ordinal,
        }
    }

    /// 时间线之外的新根段：`distinguishing_number` 不同的两份四个字段都不同，与时间线上的任何一版也不同。
    pub(super) fn off_timeline(distinguishing_number: u64, checkpoint_txg: u64) -> Self {
        Self {
            tree_table: node_pointer_at_slot(90_000 + distinguishing_number, checkpoint_txg),
            mapping_root: node_pointer_at_slot(95_000_000 + distinguishing_number, checkpoint_txg),
            tree_identifier_watermark: OFF_TIMELINE_WATERMARK_BASE + distinguishing_number,
            rollback_floor: 5_000_000 + distinguishing_number,
        }
    }
}

/// 判据比的六个字段（登记 5.3「落点 j」：实例、txg、树表指针、映射根指针、树 ID 水位、F）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct VersionFields {
    pub(super) instance: u32,
    pub(super) checkpoint_txg: u64,
    pub(super) segment: NewRootSegment,
}

impl VersionFields {
    pub(super) fn of_root(root: &RootRecord) -> Self {
        Self {
            instance: root.instance.0,
            checkpoint_txg: root.checkpoint_txg.0,
            segment: NewRootSegment {
                tree_table: root.tree_table,
                mapping_root: root.mapping_root,
                tree_identifier_watermark: root.tree_identifier_watermark,
                rollback_floor: root.rollback_floor.0,
            },
        }
    }

    pub(super) fn of_record(record: &JournalRecord) -> Self {
        Self {
            instance: record.instance.0,
            checkpoint_txg: record.checkpoint_txg.0,
            segment: NewRootSegment {
                tree_table: record.new_tree_table,
                mapping_root: record.new_mapping_root,
                tree_identifier_watermark: record.new_tree_identifier_watermark,
                rollback_floor: record.new_rollback_floor.0,
            },
        }
    }

    /// 这六个字段放进一条根记录，投影之外的四个字段取 `beyond`。
    pub(super) fn to_root_with(self, beyond: FieldsBeyondTheProjection) -> RootRecord {
        RootRecord {
            filesystem_identifier: beyond.filesystem_identifier,
            unmount_marker: beyond.unmount_marker,
            instance: InstanceGeneration(self.instance),
            checkpoint_txg: CheckpointTxg(self.checkpoint_txg),
            tree_table: self.segment.tree_table,
            tree_identifier_watermark: self.segment.tree_identifier_watermark,
            rollback_floor: CheckpointTxg(self.segment.rollback_floor),
            instance_table: beyond.instance_table,
            mapping_root: self.segment.mapping_root,
            allocation_record_tree_root: beyond.allocation_record_tree_root,
            reclaim_threshold: singlefs_core::address::CheckpointTxg(0),
            pinned_ring_slot: singlefs_core::root_record::PinnedRingSlot::NO_SLOT_PINNED,
        }
    }

    /// 由记录重建的根（登记 r2 第一节，S1 的形态）：投影取这六个字段，实例表指针、分配记录树根、fsid 抄 `chosen_root`，
    /// 卸载记号「不是卸载那一串写下的」。r1 档生成器上它与 r1 时的 `to_root` 逐字段相同（所选根的这四个字段就是那几个常量）。
    pub(super) fn to_root_rebuilt_over(self, chosen_root: &RootRecord) -> RootRecord {
        self.to_root_with(FieldsBeyondTheProjection {
            filesystem_identifier: chosen_root.filesystem_identifier,
            unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
            instance_table: chosen_root.instance_table,
            allocation_record_tree_root: chosen_root.allocation_record_tree_root,
        })
    }

    /// 照 D23（journal 的角色与格式） 已定项 15 今天的原句由记录重建的根（登记 r5 5.3 J8a、J8b）：投影取这六个字段，实例表指针、
    /// 分配记录树根、卸载记号取施加末条那一份记录新根段里的 `record_fields`，fsid 抄 `chosen_root`。r5 档的理想恢复用它。
    pub(super) fn to_root_rebuilt_from_the_record_over(
        self,
        record_fields: RecordFieldsBeyondTheProjection,
        chosen_root: &RootRecord,
    ) -> RootRecord {
        self.to_root_with(FieldsBeyondTheProjection {
            filesystem_identifier: chosen_root.filesystem_identifier,
            unmount_marker: record_fields.unmount_marker,
            instance_table: record_fields.instance_table,
            allocation_record_tree_root: record_fields.allocation_record_tree_root,
        })
    }
}

// ── 生成一条记录槽

/// 生成器造的记录过 `parse` 时该是什么结局（登记停机条款 H4）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ParseExpectation {
    /// 合法历史一族：`to_bytes` 再 `parse` 必须等于原记录，不等当场停。
    MustRoundTrip,
    /// 对抗域字母表里的记录（登记修订第 9 条）：序号不是 0 的照 H4 原文必须解得回原记录；序号 0 的 `parse` 必须拒收
    /// （D23 已定项 4：读者当那条记录损坏），没拒就停。
    OrdinalZeroMustBeRejected,
}

/// 内存盘上一个记录槽里的一条记录：字节、`parse` 的结果、它的 496 字节头算出的链值（下一条记录的反向链真值）。
#[derive(Clone, Debug)]
pub(super) struct GeneratedSlot {
    pub(super) device: u32,
    pub(super) offset_in_bytes: u64,
    pub(super) bytes: Vec<u8>,
    pub(super) parsed: Option<JournalRecord>,
    pub(super) chain_value_of_header: u32,
}

pub(super) fn generate_slot(
    device: u32,
    ring: RingGeometry,
    record: &JournalRecord,
    expectation: ParseExpectation,
) -> GeneratedSlot {
    let bytes = record.to_bytes();
    let parsed = JournalRecord::parse(&bytes, FILESYSTEM_IDENTIFIER_LOW_EIGHT_BYTES);
    match expectation {
        ParseExpectation::MustRoundTrip => assert_eq!(
            parsed.as_ref(),
            Some(record),
            "停机条款 H4：生成的记录过不了 JournalRecord::parse，或解回来不等于原记录"
        ),
        ParseExpectation::OrdinalZeroMustBeRejected => {
            if record.ordinal_within_publish.0 == 0 {
                assert!(
                    parsed.is_none(),
                    "停机条款 H4（登记修订第 9 条）：序号 0 的生成记录 JournalRecord::parse 没有拒收"
                );
            } else {
                assert_eq!(
                    parsed.as_ref(),
                    Some(record),
                    "停机条款 H4：序号不是 0 的生成记录过不了 JournalRecord::parse，或解回来不等于原记录"
                );
            }
        }
    }
    let offset_in_bytes = ring.local_record_offset_in_bytes(record.counter);
    assert_eq!(
        offset_in_bytes,
        record_offset(record.counter, ring.ring_bytes()).0,
        "停机条款 H2：装置自己算的记录落点与 record_offset 不等（计数器 {}，环 {} 槽）",
        record.counter,
        ring.ring_slots
    );
    GeneratedSlot {
        device,
        offset_in_bytes,
        chain_value_of_header: back_chain_of(&bytes),
        bytes,
        parsed,
    }
}

// ── 点名单元（U 域）

/// 一条在盘上的记录点名的那个单元的五态（登记 5.2 的 U 域）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum UnitState {
    BothCopiesGood,
    FirstDeviceCopyUnreadable,
    SecondDeviceCopyUnreadable,
    FirstDeviceCopyChecksumWrong,
    SecondDeviceCopyChecksumWrong,
}

impl UnitState {
    pub(super) const ALL: [Self; 5] = [
        Self::BothCopiesGood,
        Self::FirstDeviceCopyUnreadable,
        Self::SecondDeviceCopyUnreadable,
        Self::FirstDeviceCopyChecksumWrong,
        Self::SecondDeviceCopyChecksumWrong,
    ];

    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::BothCopiesGood => "good",
            Self::FirstDeviceCopyUnreadable => "d0_unreadable",
            Self::SecondDeviceCopyUnreadable => "d1_unreadable",
            Self::FirstDeviceCopyChecksumWrong => "d0_checksum_wrong",
            Self::SecondDeviceCopyChecksumWrong => "d1_checksum_wrong",
        }
    }
}

/// 一条记录点名的那个 16384 字节索引节点单元：好的一份与校验和不对的一份（翻了第一个字节）。
#[derive(Clone, Debug)]
pub(super) struct GeneratedUnit {
    pub(super) offset_in_bytes: u64,
    pub(super) good_bytes: Vec<u8>,
    pub(super) bytes_with_wrong_checksum: Vec<u8>,
    pub(super) named: NamedUnit,
}

fn generate_unit(record_index: usize, checkpoint_txg: u64) -> GeneratedUnit {
    let unit_bytes = usize::try_from(LOCAL_INDEX_NODE_UNIT_BYTES).expect("16384 装得进 usize");
    let good_bytes: Vec<u8> = (0..unit_bytes)
        .map(|position| {
            u8::try_from((record_index * 31 + position * 7 + 1) % 251).expect("取过模 251")
        })
        .collect();
    let mut bytes_with_wrong_checksum = good_bytes.clone();
    bytes_with_wrong_checksum[0] ^= 0xff;
    let slot = NAMED_UNIT_FIRST_SLOT + u64::try_from(record_index).expect("记录下标装得进 u64");
    let unit_checksum = crc32_castagnoli(&good_bytes);
    let location_on = |device: u32| LocationEntry {
        device: DeviceIdentity(device),
        slot: SlotNumber(slot),
        unit_checksum,
    };
    GeneratedUnit {
        offset_in_bytes: slot * LOCAL_SLOT_BYTES,
        good_bytes,
        bytes_with_wrong_checksum,
        named: NamedUnit {
            locations: [location_on(0), location_on(1)],
            unit_class: LOCAL_UNIT_CLASS_INDEX_NODE,
            birth_tree: TreeIdentifier(11),
            birth_txg: CheckpointTxg(checkpoint_txg),
            key_tail: [u8::try_from(record_index % 251).expect("取过模 251"); 10],
        },
    }
}

// ── 合法历史的生成器（可信件，登记 5.1）

/// 一次发布：几条记录、提交标记怎么摆。`commit_marker_bits` 的第 r 位为 1 表示这次发布的第 r 条（从 0 数，末条之外）带提交标记；末条必带。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PublishShape {
    pub(super) record_count: usize,
    pub(super) commit_marker_bits: u32,
}

/// 一段合法历史的计划：所选实例 g 的发布序列，X 域另接实例 g + 1 的发布序列。
#[derive(Clone, Debug)]
pub(super) struct HistoryPlan {
    pub(super) ring: RingGeometry,
    pub(super) first_txg: u64,
    pub(super) chosen_instance_publishes: Vec<PublishShape>,
    pub(super) next_instance_publishes: Vec<PublishShape>,
    pub(super) names_units: bool,
    pub(super) run_profile: RunProfile,
}

/// `total` 拆成有序的正整数之和的全部拆法，每一份不超过 `largest_part`（没给就不限）。
fn compositions_of(total: usize, largest_part: Option<usize>) -> Vec<Vec<usize>> {
    if total == 0 {
        return vec![Vec::new()];
    }
    let mut compositions = Vec::new();
    for first_part in 1..=total {
        if largest_part.is_some_and(|largest| first_part > largest) {
            break;
        }
        for rest in compositions_of(total - first_part, largest_part) {
            let mut composition = vec![first_part];
            composition.extend(rest);
            compositions.push(composition);
        }
    }
    compositions
}

/// `total_records` 条记录切成发布的全部切法 × 每次发布提交标记的全部摆法（每次发布 2^(k − 1) 种）。
pub(super) fn publish_sequences_of(
    total_records: usize,
    largest_publish: Option<usize>,
) -> Vec<Vec<PublishShape>> {
    let mut sequences = Vec::new();
    for composition in compositions_of(total_records, largest_publish) {
        let mut partial_sequences: Vec<Vec<PublishShape>> = vec![Vec::new()];
        for record_count in composition {
            let mut extended = Vec::new();
            for partial in &partial_sequences {
                for commit_marker_bits in 0..(1u32 << (record_count - 1)) {
                    let mut sequence = partial.clone();
                    sequence.push(PublishShape {
                        record_count,
                        commit_marker_bits,
                    });
                    extended.push(sequence);
                }
            }
            partial_sequences = extended;
        }
        sequences.extend(partial_sequences);
    }
    sequences
}

/// 生成历史时记下的一次发布的真相。
#[derive(Clone, Debug)]
pub(super) struct PublishTruth {
    first_record_index: usize,
    pub(super) record_count: usize,
    pub(super) fields: VersionFields,
}

/// 生成器记下的真相（登记 r5 5.3 第 3 条）：每条生成的记录的新根段投影（实例代号、txg、树表、映射根、水位、F）对到它的后三项。
/// 合法写路径里同一投影只对一份后三项；记进来时同一投影出现两份不同的后三项就停（停机条款 PH6）。键是投影里的树 ID 水位，
/// 同一个水位下逐份比整个投影。
#[derive(Clone, Debug)]
pub(super) struct GeneratedRecordTruth {
    by_tree_identifier_watermark:
        BTreeMap<u64, Vec<(VersionFields, RecordFieldsBeyondTheProjection)>>,
}

impl GeneratedRecordTruth {
    pub(super) fn new() -> Self {
        Self {
            by_tree_identifier_watermark: BTreeMap::new(),
        }
    }

    /// 记下一条生成的记录：它的投影与后三项。
    pub(super) fn note(
        &mut self,
        projection: VersionFields,
        fields: RecordFieldsBeyondTheProjection,
    ) {
        let entries = self
            .by_tree_identifier_watermark
            .entry(projection.segment.tree_identifier_watermark)
            .or_default();
        match entries
            .iter()
            .find(|(noted_projection, _noted_fields)| *noted_projection == projection)
        {
            Some((_noted_projection, noted_fields)) => assert!(
                *noted_fields == fields,
                "停机条款 PH6（登记 r5 5.3 第 3 条）：生成器真相里同一投影（实例 {}、txg {}、水位 {}）对到两份不同的后三项：{} 与 {}",
                projection.instance,
                projection.checkpoint_txg,
                projection.segment.tree_identifier_watermark,
                noted_fields.describe(),
                fields.describe()
            ),
            None => entries.push((projection, fields)),
        }
    }

    pub(super) fn note_the_record(&mut self, record: &JournalRecord) {
        self.note(
            VersionFields::of_record(record),
            RecordFieldsBeyondTheProjection::of_record(record),
        );
    }

    /// 新根段投影等于 `projection` 的那条生成记录的后三项；生成器一条都没造过这个投影是 `None`（J8「无出处」）。
    pub(super) fn fields_of(
        &self,
        projection: VersionFields,
    ) -> Option<RecordFieldsBeyondTheProjection> {
        self.by_tree_identifier_watermark
            .get(&projection.segment.tree_identifier_watermark)
            .and_then(|entries| {
                entries
                    .iter()
                    .find(|(noted_projection, _noted_fields)| *noted_projection == projection)
                    .map(|(_noted_projection, noted_fields)| *noted_fields)
            })
    }
}

/// 一段生成出来的历史：每条记录的槽（都在 0 号盘）、哪条属于哪次发布、每次发布的六个字段、点名单元。
#[derive(Clone, Debug)]
pub(super) struct History {
    pub(super) ring: RingGeometry,
    pub(super) slots: Vec<GeneratedSlot>,
    pub(super) publishes: Vec<PublishTruth>,
    /// 所选实例 g 的发布数：时间线 R_0 … R_(这个数 − 1)。X 域里实例 g + 1 的发布排在它们之后，不在时间线上。
    pub(super) chosen_instance_publish_count: usize,
    pub(super) units: Vec<Option<GeneratedUnit>>,
    pub(super) shape_description: String,
    pub(super) run_profile: RunProfile,
    /// 每条生成记录的投影对到它的后三项（登记 r5 5.3 第 3 条）；R 域的残留在 [`tally_of_a_legal_history`] 里补记进来。
    pub(super) record_truth: GeneratedRecordTruth,
}

fn describe_publish_sequence(publishes: &[PublishShape]) -> String {
    publishes
        .iter()
        .map(|shape| format!("{}c{:b}", shape.record_count, shape.commit_marker_bits))
        .collect::<Vec<String>>()
        .join("+")
}

pub(super) fn generate_history(plan: &HistoryPlan) -> History {
    let mut slots: Vec<GeneratedSlot> = Vec::new();
    let mut publishes: Vec<PublishTruth> = Vec::new();
    let mut units: Vec<Option<GeneratedUnit>> = Vec::new();
    let mut record_truth = GeneratedRecordTruth::new();
    let mut counter = plan.ring.first_history_counter();
    let mut publish_ordinal: u64 = 0;
    let instance_segments = [
        (CHOSEN_INSTANCE, &plan.chosen_instance_publishes),
        (CHOSEN_INSTANCE + 1, &plan.next_instance_publishes),
    ];
    for (instance, publishes_of_the_instance) in instance_segments {
        // 事务号按实例计数、从 1 起（D23（journal 的角色与格式） 已定项 7）。
        let mut transaction: u64 = 1;
        let mut is_first_record_of_a_later_instance = instance != CHOSEN_INSTANCE;
        for shape in publishes_of_the_instance {
            let checkpoint_txg = plan
                .first_txg
                .checked_add(publish_ordinal)
                .expect("E 域的发布数照登记 5.2 取，txg 不越过 u64");
            let segment_of_this_publish =
                NewRootSegment::of_timeline_publish(publish_ordinal, checkpoint_txg);
            // 新根段后三项：r5 档是这次发布的根 R_j 的那三项，别的档是 r1 常量（登记 r5 5.3 第 1 条）。
            let record_fields_of_this_publish =
                plan.run_profile.record_fields_of_the_timeline_publish(
                    usize::try_from(publish_ordinal).expect("发布序号很小"),
                );
            let first_record_index = slots.len();
            for record_position in 0..shape.record_count {
                let is_last_record_of_the_publish = record_position + 1 == shape.record_count;
                let is_commit = is_last_record_of_the_publish
                    || (shape.commit_marker_bits >> record_position) & 1 == 1;
                // 反向链取真值：全历史里前一条记录的字节算出的链值，不管前一条落没落盘；新实例的首条恒 0（I-8.6）。
                let back_chain = if is_first_record_of_a_later_instance {
                    0
                } else {
                    match slots.last() {
                        Some(previous_slot) => previous_slot.chain_value_of_header,
                        None => BACK_CHAIN_WHEN_THE_PREVIOUS_SLOT_HOLDS_NOTHING,
                    }
                };
                is_first_record_of_a_later_instance = false;
                // 每条记录带它那次发布的新根段（D23（journal 的角色与格式） 已定项 15）。
                let segment = segment_of_this_publish;
                let record_index = slots.len();
                let unit = plan
                    .names_units
                    .then(|| generate_unit(record_index, checkpoint_txg));
                let record = JournalRecord {
                    instance: InstanceGeneration(instance),
                    counter,
                    checkpoint_txg: CheckpointTxg(checkpoint_txg),
                    transaction,
                    is_commit,
                    ordinal_within_publish: JournalRecordOrdinalWithinPublish(
                        u32::try_from(record_position + 1).expect("一次发布的记录条数不超过 10"),
                    ),
                    place_in_publish: if is_last_record_of_the_publish {
                        JournalRecordPlaceInPublish::LastRecordOfThePublish
                    } else {
                        JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow
                    },
                    back_chain,
                    filesystem_identifier: FILESYSTEM_IDENTIFIER_LOW_EIGHT_BYTES,
                    new_tree_table: segment.tree_table,
                    new_mapping_root: segment.mapping_root,
                    new_tree_identifier_watermark: segment.tree_identifier_watermark,
                    new_rollback_floor: CheckpointTxg(segment.rollback_floor),
                    new_instance_table: record_fields_of_this_publish.instance_table,
                    new_allocation_record_tree_root: record_fields_of_this_publish
                        .allocation_record_tree_root,
                    new_unmount_marker: record_fields_of_this_publish.unmount_marker,
                    named: unit
                        .as_ref()
                        .map(|generated_unit| vec![generated_unit.named])
                        .unwrap_or_default(),
                    new_reclaim_threshold: singlefs_core::address::CheckpointTxg(0),
                    new_pinned_ring_slot:
                        singlefs_core::root_record::PinnedRingSlot::NO_SLOT_PINNED,
                };
                record_truth.note_the_record(&record);
                slots.push(generate_slot(
                    0,
                    plan.ring,
                    &record,
                    ParseExpectation::MustRoundTrip,
                ));
                units.push(unit);
                if is_commit {
                    transaction += 1;
                }
                counter += 1;
            }
            publishes.push(PublishTruth {
                first_record_index,
                record_count: shape.record_count,
                fields: VersionFields {
                    instance,
                    checkpoint_txg,
                    segment: segment_of_this_publish,
                },
            });
            publish_ordinal += 1;
        }
    }
    let shape_description = if plan.next_instance_publishes.is_empty() {
        describe_publish_sequence(&plan.chosen_instance_publishes)
    } else {
        format!(
            "{}/next_instance:{}",
            describe_publish_sequence(&plan.chosen_instance_publishes),
            describe_publish_sequence(&plan.next_instance_publishes)
        )
    };
    History {
        ring: plan.ring,
        slots,
        publishes,
        chosen_instance_publish_count: plan.chosen_instance_publishes.len(),
        units,
        shape_description,
        run_profile: plan.run_profile,
        record_truth,
    }
}

impl History {
    pub(super) fn version_fields(&self, version: usize) -> VersionFields {
        self.publishes[version].fields
    }

    /// 时间线第 `version` 版的根 R_j：六个字段照这次发布，投影之外的四个字段照档（登记 r2 第一节）。
    pub(super) fn timeline_root(&self, version: usize) -> RootRecord {
        self.version_fields(version).to_root_with(
            self.run_profile
                .fields_beyond_the_projection_of_version(version),
        )
    }

    pub(super) fn record_indexes_of_publish(&self, publish_index: usize) -> Range<usize> {
        let publish = &self.publishes[publish_index];
        publish.first_record_index..publish.first_record_index + publish.record_count
    }

    /// 返回的根逐字段等于时间线上哪一版（登记 5.3「落点 j」）；没有这样的一版是 `None`。
    pub(super) fn timeline_version_with_fields(&self, fields: VersionFields) -> Option<usize> {
        (0..self.chosen_instance_publish_count)
            .find(|version| self.version_fields(*version) == fields)
    }
}

/// R 域的那一条残留：放在 1 号盘，与 0 号盘上那条记录同 (实例, 计数器)、同 txg、同序号、同标志，新根段取时间线之外的值，
/// 自证过，反向链照真值（登记 5.2）。
#[derive(Clone, Debug)]
pub(super) struct Residue {
    pub(super) record_index: usize,
    pub(super) slot: GeneratedSlot,
    /// 生成器这一侧的那条残留记录（进生成器真相，登记 r5 5.3 第 3 条）。
    pub(super) record: JournalRecord,
}

pub(super) fn generate_residue(history: &History, record_index: usize) -> Residue {
    let mut record = history.slots[record_index]
        .parsed
        .clone()
        .expect("合法历史的每条记录都解得出（H4 在生成时断言过）");
    let off_timeline_segment = NewRootSegment::off_timeline(
        u64::try_from(record_index).expect("记录下标装得进 u64"),
        record.checkpoint_txg.0,
    );
    record.new_tree_table = off_timeline_segment.tree_table;
    record.new_mapping_root = off_timeline_segment.mapping_root;
    record.new_tree_identifier_watermark = off_timeline_segment.tree_identifier_watermark;
    record.new_rollback_floor = CheckpointTxg(off_timeline_segment.rollback_floor);
    // 后三项：r5 档是区分号 d = 记录下标的 h(d)，别的档仍是 r1 常量（登记 r5 5.3 第 2 条）。
    let record_fields = history
        .run_profile
        .record_fields_off_the_timeline(u64::try_from(record_index).expect("记录下标装得进 u64"));
    record.new_instance_table = record_fields.instance_table;
    record.new_allocation_record_tree_root = record_fields.allocation_record_tree_root;
    record.new_unmount_marker = record_fields.unmount_marker;
    Residue {
        record_index,
        slot: generate_slot(1, history.ring, &record, ParseExpectation::MustRoundTrip),
        record,
    }
}

#[cfg(test)]
mod root_choice_fault_tests;
