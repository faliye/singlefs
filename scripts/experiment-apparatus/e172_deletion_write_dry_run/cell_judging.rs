//! 格行的建造与各格（布局三格、退役、队列与墓碑、树表记账与水位）的判定。

use super::*;

struct CellRowBuilder<'rows> {
    rows: &'rows mut Vec<CellRow>,
    pub(super) history: String,
    pub(super) txg: u64,
    pub(super) publish_kind: String,
    /// L03-7 选到哪一支、第二支只记值的 key 区间（只记、不判的行，跑前登记 e172-r2 第五节 5.8 改6）。
    pub(super) leaf_branch_records: Vec<String>,
}

impl CellRowBuilder<'_> {
    pub(super) fn push(&mut self, cell: &'static str, field: &str, expected: impl ToString, actual: impl ToString) {
        self.push_with_discrimination(cell, field, expected, actual, true);
    }
    fn push_with_discrimination(&mut self, cell: &'static str, field: &str, expected: impl ToString, actual: impl ToString, discriminable: bool) {
        self.rows.push(CellRow {
            cell,
            field: field.to_string(),
            history: self.history.clone(),
            txg: self.txg,
            publish_kind: self.publish_kind.clone(),
            expected: expected.to_string(),
            actual: actual.to_string(),
            discriminable,
        });
    }
}

pub(super) fn field_text(record: Option<&Vec<u8>>, offset: usize, width: usize) -> String {
    record.map_or_else(|| "absent".to_string(), |record| read_little_endian(record, offset, width).to_string())
}

fn bytes_text(record: Option<&Vec<u8>>, start: usize, end: usize) -> String {
    record.map_or_else(|| "absent".to_string(), |record| hexadecimal_of(&record[start..end]))
}

fn times_text(record: Option<&Vec<u8>>) -> String {
    record.map_or_else(|| "absent".to_string(), |record| format!("{}/{}/{}", read_u64(record, 64), read_u64(record, 72), read_u64(record, 80)))
}

pub(super) fn records_written_in(analysis: &SegmentAnalysis, tree: u64, packed_record_type: u64) -> Vec<Vec<u8>> {
    analysis
        .units
        .iter()
        .filter_map(|unit| match &unit.copy_on_first_device.body {
            DecodedBody::PackedContainer(container) if container.birth_tree == tree && container.packed_record_type == packed_record_type => {
                Some(container.records.clone())
            }
            DecodedBody::SystemConfiguration(_)
            | DecodedBody::RootRecord(_)
            | DecodedBody::JournalRecord(_)
            | DecodedBody::IndexNode(_)
            | DecodedBody::PackedContainer(_)
            | DecodedBody::DataUnit { .. }
            | DecodedBody::Unrecognized => None,
        })
        .flatten()
        .collect()
}

pub(super) fn index_node_written_in(analysis: &SegmentAnalysis, tree: u64) -> Option<&IndexNode> {
    analysis.units.iter().find_map(|unit| match &unit.copy_on_first_device.body {
        DecodedBody::IndexNode(node) if node.tree_identifier == tree => Some(node),
        DecodedBody::SystemConfiguration(_)
        | DecodedBody::RootRecord(_)
        | DecodedBody::JournalRecord(_)
        | DecodedBody::IndexNode(_)
        | DecodedBody::PackedContainer(_)
        | DecodedBody::DataUnit { .. }
        | DecodedBody::Unrecognized => None,
    })
}

pub(super) fn container_written_in(analysis: &SegmentAnalysis, tree: u64, packed_record_type: u64) -> Option<(&UnitRole, &PackedContainer)> {
    analysis.units.iter().find_map(|unit| match &unit.copy_on_first_device.body {
        DecodedBody::PackedContainer(container) if container.birth_tree == tree && container.packed_record_type == packed_record_type => Some((unit, container)),
        DecodedBody::SystemConfiguration(_)
        | DecodedBody::RootRecord(_)
        | DecodedBody::JournalRecord(_)
        | DecodedBody::IndexNode(_)
        | DecodedBody::PackedContainer(_)
        | DecodedBody::DataUnit { .. }
        | DecodedBody::Unrecognized => None,
    })
}

/// 跑前登记第六节 6.1 的 L03-1 到 L03-14：一次删除类发布一组行。
/// 交回只记、不判的 L03-7 行（`name=l03_7_branch`、`name=l03_7_record`）。
pub(super) fn judge_layout_three_cells(
    history: &HistoryExport,
    analysis: &SegmentAnalysis,
    before: &VersionState,
    after: &VersionState,
    deletion_publishes: &[(DeletionPublish, u64)],
    rows: &mut Vec<CellRow>,
) -> Vec<String> {
    let txg = analysis.label_txg();
    let publish_kind = analysis.label_kind();
    let is_intent = publish_kind == "unlink_intent";
    let data_units_after = data_units_of_version(after).unwrap_or(u64::MAX);
    let is_final = !is_intent && data_units_after == 0;
    // 退役单元只在「那片叶删空、身份消失」的那一次发布里写（layout/03 第 21 行「有退役记录的那次发布（叶删空摘掉…）」，与 L03-7 第一支同一个条件）。
    let empties_the_leaf = is_final && matches!(deleted_record_leaf_of_version(before).0, DeletedRecordLeaf::OnlyTheDeletedRecord | DeletedRecordLeaf::NotFound);
    let mut builder = CellRowBuilder { rows, history: history.name.clone(), txg, publish_kind: publish_kind.clone(), leaf_branch_records: Vec::new() };
    let inode_records_now = records_written_in(analysis, INODE_TREE, 2);
    let record_now = record_of_inode(&inode_records_now, DELETED_INODE);
    let record_before = record_of_inode(&inode_records_of_version(before), DELETED_INODE);
    let tombstone_records = records_written_in(analysis, TOMBSTONE_TREE, 1);
    let tombstone_record = tombstone_records.first();
    if !is_final {
        builder.push("L03-1", "nlink", 0, field_text(record_now.as_ref(), 36, 4));
        let expected_size = if is_intent {
            history.configuration.get("file_bytes").and_then(|text| text.parse::<u64>().ok()).unwrap_or(u64::MAX)
        } else {
            data_units_after.saturating_mul(DATA_UNIT_PAYLOAD_CAPACITY)
        };
        builder.push("L03-2", "size", expected_size, field_text(record_now.as_ref(), 40, 8));
        builder.push("L03-3", "blocks", expected_size.div_ceil(512), field_text(record_now.as_ref(), 48, 8));
        builder.push("L03-4", "change_count", txg, field_text(record_now.as_ref(), 88, 8));
        let (expected_times, times_discriminable) = expected_times_of(is_intent, &history.configuration, record_before.as_ref());
        builder.push_with_discrimination("L03-5", "times", expected_times, times_text(record_now.as_ref()), times_discriminable);
        if is_intent {
            builder.push("L03-6", "inode", DELETED_INODE, field_text(record_now.as_ref(), 0, 8));
            builder.push("L03-6", "object_birth_equals_tombstone", field_text(tombstone_record, 8, 8), field_text(record_now.as_ref(), 8, 8));
            builder.push("L03-6", "locality", 0, field_text(record_now.as_ref(), 16, 8));
            builder.push("L03-6", "mode", 0o100_644, field_text(record_now.as_ref(), 24, 4));
            builder.push("L03-6", "uid", 0, field_text(record_now.as_ref(), 28, 4));
            builder.push("L03-6", "gid", 0, field_text(record_now.as_ref(), 32, 4));
            builder.push("L03-6", "rdev", 0, field_text(record_now.as_ref(), 56, 8));
            builder.push("L03-6", "nanoseconds_padding_flags_reserved", hexadecimal_of(&[0u8; 44]), bytes_text(record_now.as_ref(), 96, 140));
        } else {
            for (field, start, end) in [
                ("inode", 0, 8),
                ("object_birth", 8, 16),
                ("locality", 16, 24),
                ("mode_uid_gid", 24, 36),
                ("rdev", 56, 64),
                ("nanoseconds_padding_flags_reserved", 96, 140),
            ] {
                builder.push("L03-6", field, bytes_text(record_before.as_ref(), start, end), bytes_text(record_now.as_ref(), start, end));
            }
        }
    } else {
        let (deleted_record_leaf, other_inodes_before) = deleted_record_leaf_of_version(before);
        let containers_with_the_record = inode_records_now.iter().filter(|record| read_u64(record, 0) == DELETED_INODE).count();
        builder.push("L03-7", "deleted_record_in_written_containers", 0, containers_with_the_record);
        let inode_root = index_node_written_in(analysis, INODE_TREE);
        let inode_root_key_range_text =
            inode_root.map_or_else(|| "absent".to_string(), |node| format!("{}/{}", read_u64(&node.key_range_low, 0), read_u64(&node.key_range_high, 0)));
        let leaf_containers_written = analysis.units.iter().filter(|unit| unit.identity.unit_type_tag == 3 && unit.identity.tree_identifier == INODE_TREE).count();
        // 第一支（只有被删那一条）照第 1 次；第二支（还有别的记录）：写出恰 1 片叶、根条目数 1，key 区间只记值、不判（C478 的地界）。
        let (expected_root_entries, expected_leaf_containers) = match deleted_record_leaf {
            DeletedRecordLeaf::OnlyTheDeletedRecord | DeletedRecordLeaf::NotFound => (0, 0),
            DeletedRecordLeaf::BesideOtherRecords => (1, 1),
        };
        builder.push("L03-7", "inode_root_entry_count", expected_root_entries, inode_root.map_or_else(|| "absent".to_string(), |node| node.entry_count.to_string()));
        match deleted_record_leaf {
            DeletedRecordLeaf::OnlyTheDeletedRecord | DeletedRecordLeaf::NotFound => builder.push(
                "L03-7",
                "inode_root_key_range",
                "0/0",
                inode_root_key_range_text,
            ),
            DeletedRecordLeaf::BesideOtherRecords => {
                let mut other_inodes_now: Vec<u64> = inode_records_now.iter().map(|record| read_u64(record, 0)).filter(|inode| *inode != DELETED_INODE).collect();
                other_inodes_now.sort_unstable();
                let inode_list = |inodes: &[u64]| if inodes.is_empty() { "none".to_string() } else { inodes.iter().map(u64::to_string).collect::<Vec<_>>().join(",") };
                builder.push("L03-7", "other_records_in_written_containers", inode_list(&other_inodes_before), inode_list(&other_inodes_now));
                builder.leaf_branch_records.push(format!(
                    "name=l03_7_record history={} txg={} inode_root_key_range={inode_root_key_range_text} judged=false",
                    builder.history, builder.txg
                ));
            }
        }
        builder.push("L03-7", "inode_leaf_containers_written", expected_leaf_containers, leaf_containers_written);
        builder.leaf_branch_records.push(format!(
            "name=l03_7_branch history={} txg={} branch={} other_records_before={}",
            builder.history,
            builder.txg,
            deleted_record_leaf.name(),
            other_inodes_before.len()
        ));
    }
    judge_queue_and_tombstone_cells(analysis, after, is_intent, is_final, &mut builder, tombstone_record);
    judge_tree_table_accounting_and_watermark_cells(analysis, after, is_final, empties_the_leaf, &mut builder);
    judge_retirement_cells(analysis, before, empties_the_leaf, expected_retirement_death_txg(deletion_publishes), &mut builder);
    builder.leaf_branch_records
}

/// layout/03 第 74 行（退役记录偏移 26）：死亡 txg = 身份消失那次发布（最后一批）的 checkpoint_txg，明写不取 unlink 意图那次（第九节 M41 的锚点）。
pub(super) fn expected_retirement_death_txg(deletion_publishes: &[(DeletionPublish, u64)]) -> Option<u64> {
    deletion_publishes.iter().find(|(publish, _)| *publish == DeletionPublish::FinalBatch).map(|(_, txg)| *txg)
}

/// 跑前登记 e172-r3 第六节 6.1 的 L03-15（退役容器）、L03-16（退役记录）、L03-17（退役树根）：一次删除类发布一组行。
/// 每一次都报 L03-16 `retirement_records_written`（写了的那一次期望 1、别的期望 0），三段历史在这一格上都有命中（R1.b）。
fn judge_retirement_cells(analysis: &SegmentAnalysis, before: &VersionState, empties_the_leaf: bool, expected_death_txg: Option<u64>, builder: &mut CellRowBuilder<'_>) {
    let retirement_container = container_written_in(analysis, RETIREMENT_TREE, RETIREMENT_PACKED_RECORD_TYPE);
    let retirement_root = index_node_written_in(analysis, RETIREMENT_TREE);
    let retirement_records = records_written_in(analysis, RETIREMENT_TREE, RETIREMENT_PACKED_RECORD_TYPE);
    builder.push("L03-16", "retirement_records_written", usize::from(empties_the_leaf), retirement_records.len());
    if !empties_the_leaf {
        builder.push("L03-15", "retirement_container_written", "false", retirement_container.is_some());
        builder.push("L03-17", "retirement_root_written", "false", retirement_root.is_some());
        return;
    }
    let containers_written = analysis.units.iter().filter(|unit| unit.identity.unit_type_tag == 3 && unit.identity.tree_identifier == RETIREMENT_TREE).count();
    builder.push("L03-15", "retirement_containers_written", 1, containers_written);
    builder.push(
        "L03-15",
        "retirement_container",
        format!("code3/type{RETIREMENT_PACKED_RECORD_TYPE}/width{RETIREMENT_RECORD_BYTES}/birth_tree{RETIREMENT_TREE}/records_at{PACKED_UNIT_RECORDS_OFFSET}/container1/count1"),
        retirement_container.map_or_else(
            || "absent".to_string(),
            |(_, container)| {
                format!(
                    "code3/type{}/width{}/birth_tree{}/records_at{PACKED_UNIT_RECORDS_OFFSET}/container{}/count{}",
                    container.packed_record_type, container.record_width, container.birth_tree, container.container_number, container.record_count
                )
            },
        ),
    );
    // L03-16：身份四格的期望取上一版装着被删记录的那片 inode 叶的码 3 头（条件是条款的输入，同 L03-7 的取法）。
    let leaf_before = inode_leaf_holding_the_deleted_record(before);
    let record = retirement_records.first();
    let leaf_field = |value: fn(&PackedContainer) -> u64| leaf_before.map_or_else(|| "absent".to_string(), |leaf| value(leaf).to_string());
    builder.push("L03-16", "birth_tree", leaf_field(|leaf| leaf.birth_tree), field_text(record, 0, 8));
    builder.push("L03-16", "packed_record_type", leaf_field(|leaf| leaf.packed_record_type), field_text(record, 8, 2));
    builder.push("L03-16", "container_number", leaf_field(|leaf| leaf.container_number), field_text(record, 10, 8));
    builder.push("L03-16", "container_birth", leaf_field(|leaf| leaf.container_birth), field_text(record, 18, 8));
    builder.push("L03-16", "death_txg", expected_death_txg.map_or_else(|| "absent".to_string(), |txg| txg.to_string()), field_text(record, 26, 8));
    let retirement_container_locations: Vec<String> = analysis
        .regions
        .iter()
        .filter(|region| matches!(&region.body, DecodedBody::PackedContainer(container) if container.birth_tree == RETIREMENT_TREE))
        .map(|region| format!("{}:{}", region.device, region.slot()))
        .collect();
    builder.push(
        "L03-17",
        "retirement_root",
        format!(
            "tag2/tree{RETIREMENT_TREE}/key_width8/width120/count1/separator1/birth_tree{RETIREMENT_TREE}/type{RETIREMENT_PACKED_RECORD_TYPE}/container1/children{}",
            retirement_container_locations.join(",")
        ),
        retirement_root.map_or_else(|| "absent".to_string(), internal_root_summary),
    );
}

/// 墓碑树根与退役树根同形（layout/03 第 21 行）：头里的树 ID、key 宽、条目宽、条目数，与首条条目的分隔 key、身份引用三段、子指针的两份落点。
fn internal_root_summary(node: &IndexNode) -> String {
    let entry = node.entries.first().cloned().unwrap_or_else(|| vec![0; INODE_INTERNAL_ENTRY_BYTES]);
    let children = block_pointer_of(&entry[34..34 + NODE_POINTER_BYTES])
        .locations
        .iter()
        .map(|location| format!("{}:{}", location.device, location.slot))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "tag2/tree{}/key_width{}/width{}/count{}/separator{}/birth_tree{}/type{}/container{}/children{children}",
        node.tree_identifier,
        node.key_width,
        node.entry_width,
        node.entry_count,
        read_u64(&entry, 0),
        read_u64(&entry, 8),
        read_u16(&entry, 16),
        read_u64(&entry, 18)
    )
}

/// 三个时间秒都写同一个值时的期望写法（与 `times_text` 同形）。
pub(super) fn three_equal_times_text(seconds: &str) -> String {
    format!("{seconds}/{seconds}/{seconds}")
}

/// L03-5 的期望与可不可分辨（跑前登记 e172-r2 第一节 1.2、第六节 6.1）：意图那一版三格同值 = 调用方给的时间——导出带
/// `intent_write_time_seconds=`（H3）就取它、可分辨；不带（H1、H2）照第 1 次取 `sequential_write_time_seconds=`、报不可分辨；
/// 批取上一版这条记录的三格（layout/03 第 13 行「各批照抄上一版这条记录的值」）。
pub(super) fn expected_times_of(is_intent: bool, configuration: &BTreeMap<String, String>, record_before: Option<&Vec<u8>>) -> (String, bool) {
    if !is_intent {
        return (times_text(record_before), true);
    }
    match configuration.get("intent_write_time_seconds") {
        Some(intent_seconds) => (three_equal_times_text(intent_seconds), true),
        None => (three_equal_times_text(configuration.get("sequential_write_time_seconds").map_or("", String::as_str)), false),
    }
}

impl SegmentAnalysis {
    pub(super) fn label_txg(&self) -> u64 {
        match &self.label {
            SegmentLabel::Publish { txg, .. } => *txg,
            SegmentLabel::Outside { .. } => u64::MAX,
        }
    }
    pub(super) fn label_kind(&self) -> String {
        match &self.label {
            SegmentLabel::Publish { publish_kind, .. } => publish_kind.clone(),
            SegmentLabel::Outside { segment_name } => segment_name.clone(),
        }
    }
}

fn judge_queue_and_tombstone_cells(
    analysis: &SegmentAnalysis,
    after: &VersionState,
    is_intent: bool,
    is_final: bool,
    builder: &mut CellRowBuilder<'_>,
    tombstone_record: Option<&Vec<u8>>,
) {
    let deleted_inodes_root = index_node_written_in(analysis, DELETED_INODES_TREE);
    let deleted_inodes_unit = analysis.units.iter().find(|unit| unit.identity.tree_identifier == DELETED_INODES_TREE);
    if is_intent {
        let summary = deleted_inodes_root.map_or_else(
            || "absent".to_string(),
            |node| {
                let declared = read_u16(&deleted_inodes_unit.map(|unit| unit.copy_on_first_device.bytes.clone()).unwrap_or_default(), UNIT_DECLARED_LENGTH_OFFSET);
                format!(
                    "tag2/tree{}/key_width{}/count{}/width{}/entries_at{}/entry{}/declared{}",
                    node.tree_identifier,
                    node.key_width,
                    node.entry_count,
                    node.entry_width,
                    index_node_header_bytes(node.key_width) + RESERVED_NONCE_MAC_ALGORITHM_BYTES,
                    node.entries.first().map_or_else(|| "none".to_string(), |entry| hexadecimal_of(entry)),
                    declared
                )
            },
        );
        let expected = format!(
            "tag2/tree{DELETED_INODES_TREE}/key_width8/count1/width8/entries_at131/entry{}/declared8",
            hexadecimal_of(&DELETED_INODE.to_le_bytes())
        );
        builder.push("L03-8", "deleted_inodes_root", expected, summary);
    } else {
        builder.push("L03-8", "deleted_inodes_root_written", "false", deleted_inodes_root.is_some());
        if is_final {
            builder.push("L03-8", "tree_table_deleted_inodes_root_pointer_zero", "true", tree_table_root_is_zero(after, DELETED_INODES_TREE).map_or_else(|| "absent".to_string(), |zero| zero.to_string()));
        }
    }
    let tombstone_container = container_written_in(analysis, TOMBSTONE_TREE, 1);
    let tombstone_root = index_node_written_in(analysis, TOMBSTONE_TREE);
    if is_intent {
        builder.push("L03-9", "object", DELETED_INODE, field_text(tombstone_record, 0, 8));
        let record_now = record_of_inode(&records_written_in(analysis, INODE_TREE, 2), DELETED_INODE);
        builder.push("L03-9", "object_birth", field_text(record_now.as_ref(), 8, 8), field_text(tombstone_record, 8, 8));
        builder.push("L03-9", "range_start", 0, field_text(tombstone_record, 16, 8));
        builder.push("L03-9", "range_end", u64::MAX, field_text(tombstone_record, 24, 8));
        let intent_txg = builder.txg;
        builder.push("L03-9", "death_txg", intent_txg, field_text(tombstone_record, 32, 8));
        builder.push(
            "L03-10",
            "tombstone_container",
            format!("code3/type1/width40/birth_tree{TOMBSTONE_TREE}/records_at136/container1/count1"),
            tombstone_container.map_or_else(
                || "absent".to_string(),
                |(_, container)| {
                    format!(
                        "code3/type{}/width{}/birth_tree{}/records_at{PACKED_UNIT_RECORDS_OFFSET}/container{}/count{}",
                        container.packed_record_type, container.record_width, container.birth_tree, container.container_number, container.record_count
                    )
                },
            ),
        );
        let tombstone_container_locations: Vec<String> = analysis
            .regions
            .iter()
            .filter(|region| matches!(&region.body, DecodedBody::PackedContainer(container) if container.birth_tree == TOMBSTONE_TREE))
            .map(|region| format!("{}:{}", region.device, region.slot()))
            .collect();
        builder.push(
            "L03-11",
            "tombstone_root",
            format!("tag2/tree{TOMBSTONE_TREE}/key_width8/width120/count1/separator1/birth_tree{TOMBSTONE_TREE}/type1/container1/children{}", tombstone_container_locations.join(",")),
            tombstone_root.map_or_else(|| "absent".to_string(), internal_root_summary),
        );
    } else {
        builder.push("L03-10", "tombstone_container_written", "false", tombstone_container.is_some());
        builder.push("L03-11", "tombstone_root_written", "false", tombstone_root.is_some());
    }
}

fn judge_tree_table_accounting_and_watermark_cells(analysis: &SegmentAnalysis, after: &VersionState, is_final: bool, empties_the_leaf: bool, builder: &mut CellRowBuilder<'_>) {
    let tree_table = index_node_written_in(analysis, 0);
    builder.push("L03-12", "entry_count", TREE_TABLE_ENTRIES_AFTER_THE_FIRST_FILE, tree_table.map_or_else(|| "absent".to_string(), |node| node.entry_count.to_string()));
    // layout/01 第 419 行：树表码 2 头的 key 区间 [extent 树的树 ID, 末条退役树的树 ID]。
    builder.push(
        "L03-12",
        "tree_table_key_range",
        format!("{EXTENT_TREE}/{RETIREMENT_TREE}"),
        tree_table.map_or_else(|| "absent".to_string(), |node| format!("{}/{}", read_u64(&node.key_range_low, 0), read_u64(&node.key_range_high, 0))),
    );
    let entries = tree_table.map(tree_table_entries_of).unwrap_or_default();
    // 退役树的根指针：只有「那片叶删空」的最后一批之后不是零（layout/03 第 23 行「零根」起，第 21 行「有退役记录的那次发布」写根）。
    for (tree, kind_code, head_identifier, root_is_zero) in [
        (DELETED_INODES_TREE, 9, INODE_TREE, is_final),
        (TOMBSTONE_TREE, 10, INODE_TREE, false),
        (LINEAGE_TREE, 11, 0, true),
        (LOGGED_OPERATIONS_TREE, 12, 0, true),
        (RETIREMENT_TREE, RETIREMENT_TREE_KIND_CODE, INODE_TREE, !empties_the_leaf),
    ] {
        let entry = entries.iter().find(|entry| entry.tree == tree);
        builder.push(
            "L03-12",
            &format!("tree{tree}"),
            format!("kind{kind_code}/head{head_identifier}/root_zero{root_is_zero}"),
            entry.map_or_else(
                || "absent".to_string(),
                |entry| format!("kind{}/head{}/root_zero{}", entry.kind_code, entry.head_identifier, entry.root_pointer.is_all_zero),
            ),
        );
    }
    let tombstone_levels = tombstone_internal_levels(after);
    let retirement_levels = retirement_internal_levels(after);
    let live_inodes = live_inode_count(after);
    let inode_records = inode_record_count(after);
    builder.push(
        "L03-13",
        &format!("committed_reservation_tombstone_levels{tombstone_levels}_retirement_levels{retirement_levels}_live{live_inodes}_records{inode_records}"),
        expected_committed_reservation(tombstone_levels, retirement_levels, live_inodes, inode_records, RetirementShare::Included),
        accounting_value(after, 6).map_or_else(|| "absent".to_string(), |value| value.to_string()),
    );
    let root_watermark = analysis.root_records.first().map_or_else(|| "absent".to_string(), |(_, _, record)| record.watermark.to_string());
    builder.push("L03-14", "root_record_watermark", TREE_IDENTIFIER_WATERMARK_AFTER_THE_FIRST_FILE, root_watermark);
    for (device, _, record) in &analysis.journal_records {
        builder.push("L03-14", &format!("journal_watermark_device{device}"), TREE_IDENTIFIER_WATERMARK_AFTER_THE_FIRST_FILE, record.watermark);
    }
}
