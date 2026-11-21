//! 阳性对照的破坏与预期误归属、锚点行、续写锚点与轨迹行。

use super::*;

// ---------------------------------------------------------------------------------------------
// 阳性对照（跑前登记第五节 5.4）：在内存里改坏一份副本，重跑全部检查，看目标行由绿转红。
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PositiveControl {
    FlipIntentNlink,
    NonzeroDeletedInodesPadding,
    BumpTombstoneDeathTxg,
    DropIntentFirstBarrier,
    /// PC-F：挂载写行之前那一段的后边界挪后一步（目标行 R8.c）。
    ShiftMountSegmentEndOneStepLater,
    /// PC-G：那一段的后边界挪前到那一段里最后一次写所在的步号（专打 R8.c 的系统配置槽那一支）。
    ShiftMountSegmentEndToItsLastWrite,
    /// PC-H：意图那次发布步号最小的系统配置槽写，T_time 改成按秒写的 5、重封整槽校验和。
    RewriteIntentTimeThresholdAsSeconds,
    /// PC-I：H3 意图那次 t3 里 1 号记录三个时间秒改成顺序写的时间、重封码 3 载荷校验和。
    CopySequentialWriteTimeIntoTheIntentRecord,
    /// PC-J：退役容器里第一条退役记录偏移 26 起的死亡 txg 加 1、重封码 3 载荷校验和（跑前登记 e172-r3 第五节 5.4）。
    BumpRetirementDeathTxg,
    /// PC-K：意图那次记账树叶里标签 6 那一行的 value 减 70（= 2 × 35，一条记录的退役份额）、重封码 2 载荷 CRC。
    ReduceIntentTagSixByOneRecordsRetirementShare,
    /// PC-L：退役树根首条条目偏移 0 起的分隔 key（容器号）1 改成 2、重封码 2 载荷 CRC。
    BumpRetirementRootSeparator,
}

/// PC-K 从意图那一版标签 6 里减掉的字节数：一条记录的退役份额 × 副本数（跑前登记 e172-r3 第五节 5.4「减 70（= 2 × 35）」）。
pub(super) const POSITIVE_CONTROL_TAG_SIX_REDUCTION_BYTES: u64 = 70;

/// PC-F、PC-G 改坏之后预期的那一次归错（跑前登记 e172-r2 第五节 5.4 表「改坏之后预期红在哪」）：从原样导出算出来。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ExpectedMisattribution {
    pub(super) step: usize,
    pub(super) exported_segment: String,
    pub(super) decoded: Option<WriteAttribution>,
    forbidden_decoded: Option<WriteAttribution>,
}

impl PositiveControl {
    pub(super) const EVERY_CONTROL: [PositiveControl; 11] = [
        PositiveControl::FlipIntentNlink,
        PositiveControl::NonzeroDeletedInodesPadding,
        PositiveControl::BumpTombstoneDeathTxg,
        PositiveControl::DropIntentFirstBarrier,
        PositiveControl::ShiftMountSegmentEndOneStepLater,
        PositiveControl::ShiftMountSegmentEndToItsLastWrite,
        PositiveControl::RewriteIntentTimeThresholdAsSeconds,
        PositiveControl::CopySequentialWriteTimeIntoTheIntentRecord,
        PositiveControl::BumpRetirementDeathTxg,
        PositiveControl::ReduceIntentTagSixByOneRecordsRetirementShare,
        PositiveControl::BumpRetirementRootSeparator,
    ];
    /// 跑前登记 e172-r3 第五节 5.4：适用的（对照, 历史）对共 25 个（第 2 次的 18 个加 PC-J 2、PC-K 3、PC-L 2）。
    pub(super) const APPLICABLE_PAIRS_EXPECTED: usize = 25;
    pub(super) fn name(self) -> &'static str {
        match self {
            PositiveControl::FlipIntentNlink => "PC-A",
            PositiveControl::NonzeroDeletedInodesPadding => "PC-B",
            PositiveControl::BumpTombstoneDeathTxg => "PC-C",
            PositiveControl::DropIntentFirstBarrier => "PC-D",
            PositiveControl::ShiftMountSegmentEndOneStepLater => "PC-F",
            PositiveControl::ShiftMountSegmentEndToItsLastWrite => "PC-G",
            PositiveControl::RewriteIntentTimeThresholdAsSeconds => "PC-H",
            PositiveControl::CopySequentialWriteTimeIntoTheIntentRecord => "PC-I",
            PositiveControl::BumpRetirementDeathTxg => "PC-J",
            PositiveControl::ReduceIntentTagSixByOneRecordsRetirementShare => "PC-K",
            PositiveControl::BumpRetirementRootSeparator => "PC-L",
        }
    }
    /// 第 2 行射程外的两条（目标行是附带的 R8.c，跑前登记 e172-r3 第五节 5.4）：没转红只作废 R8.c 那几行。
    pub(super) fn is_in_row_two_scope(self) -> bool {
        match self {
            PositiveControl::ShiftMountSegmentEndOneStepLater | PositiveControl::ShiftMountSegmentEndToItsLastWrite => false,
            PositiveControl::FlipIntentNlink
            | PositiveControl::NonzeroDeletedInodesPadding
            | PositiveControl::BumpTombstoneDeathTxg
            | PositiveControl::DropIntentFirstBarrier
            | PositiveControl::RewriteIntentTimeThresholdAsSeconds
            | PositiveControl::CopySequentialWriteTimeIntoTheIntentRecord
            | PositiveControl::BumpRetirementDeathTxg
            | PositiveControl::ReduceIntentTagSixByOneRecordsRetirementShare
            | PositiveControl::BumpRetirementRootSeparator => true,
        }
    }
    /// 目标行是绿的（`true`）还是红的。
    fn target_is_green(self, judgement: &HistoryJudgement) -> bool {
        match self {
            PositiveControl::FlipIntentNlink => {
                judgement.intent_cell_is_equal("L03-1", "nlink") == Some(true) && judgement.intent_inode_container_checksum_failures() == 0
            }
            PositiveControl::NonzeroDeletedInodesPadding => judgement.coverage.constant_zero_nonzero_bytes == 0,
            PositiveControl::BumpTombstoneDeathTxg => judgement.intent_cell_is_equal("L03-9", "death_txg") == Some(true),
            PositiveControl::DropIntentFirstBarrier => judgement.intent_segment_count_is_equal() == Some(true),
            PositiveControl::ShiftMountSegmentEndOneStepLater | PositiveControl::ShiftMountSegmentEndToItsLastWrite => judgement.misattributed_writes() == 0,
            PositiveControl::RewriteIntentTimeThresholdAsSeconds => judgement.time_threshold_rows_not_expected().is_empty(),
            PositiveControl::CopySequentialWriteTimeIntoTheIntentRecord => judgement.intent_times_row().is_some_and(|row| row.discriminable && row.is_equal()),
            PositiveControl::BumpRetirementDeathTxg => judgement.first_cell_row("L03-16", "death_txg").is_some_and(CellRow::is_equal),
            PositiveControl::ReduceIntentTagSixByOneRecordsRetirementShare => {
                let readings = judgement.intent_tag_six_readings();
                readings.anchor_equal() && readings.clause_equal() && readings.pin_equal()
            }
            PositiveControl::BumpRetirementRootSeparator => judgement.first_cell_row("L03-17", "retirement_root").is_some_and(CellRow::is_equal),
        }
    }
    /// 改坏之后目标行红、而且红在预期的那一处（`baseline` 是原样导出上的判定，`expected` 是 PC-F、PC-G 预期的那一次归错）。
    fn target_is_red(self, corrupted: &HistoryJudgement, baseline: &HistoryJudgement, history: &HistoryExport, expected: Option<&ExpectedMisattribution>) -> bool {
        match self {
            PositiveControl::FlipIntentNlink => {
                corrupted.intent_cell_is_equal("L03-1", "nlink") == Some(false) && corrupted.intent_inode_container_checksum_failures() > 0
            }
            PositiveControl::NonzeroDeletedInodesPadding | PositiveControl::BumpTombstoneDeathTxg | PositiveControl::DropIntentFirstBarrier => {
                !self.target_is_green(corrupted)
            }
            PositiveControl::ShiftMountSegmentEndOneStepLater | PositiveControl::ShiftMountSegmentEndToItsLastWrite => {
                let misattributed: Vec<&AttributionRow> = corrupted.attribution_rows.iter().filter(|row| !row.is_match).collect();
                match (misattributed.as_slice(), expected) {
                    ([only], Some(expected)) => {
                        only.step == expected.step
                            && only.exported_segment == expected.exported_segment
                            && expected.decoded.as_ref().is_none_or(|decoded| *decoded == only.decoded)
                            && expected.forbidden_decoded.as_ref().is_none_or(|forbidden| *forbidden != only.decoded)
                    }
                    _ => false,
                }
            }
            PositiveControl::RewriteIntentTimeThresholdAsSeconds => {
                let changed_step = intent_system_configuration_steps(history).first().copied();
                let not_expected = corrupted.time_threshold_rows_not_expected();
                matches!(not_expected.as_slice(), [only] if Some(only.step) == changed_step && only.actual_milliseconds == 5)
                    && corrupted.system_configuration_checksum_failures() == baseline.system_configuration_checksum_failures()
            }
            PositiveControl::CopySequentialWriteTimeIntoTheIntentRecord => {
                let sequential = history.configuration.get("sequential_write_time_seconds").map_or("", String::as_str);
                corrupted.intent_times_row().is_some_and(|row| !row.is_equal() && row.actual == three_equal_times_text(sequential))
            }
            // 死亡 txg 那一行不等、L03-16 别的行照旧相等。
            PositiveControl::BumpRetirementDeathTxg => {
                let death_txg_red = corrupted.first_cell_row("L03-16", "death_txg").is_some_and(|row| !row.is_equal());
                let others_still_equal = corrupted.cell_rows.iter().filter(|row| row.cell == "L03-16" && row.field != "death_txg").all(CellRow::is_equal);
                death_txg_red && others_still_equal
            }
            // 意图那一版 A-tag6、L03-13、钉值层（H1、H2）三处都不等、解出的 = 期望 − 70；R-ret 判别臂那一份照旧不等。
            PositiveControl::ReduceIntentTagSixByOneRecordsRetirementShare => {
                let readings = corrupted.intent_tag_six_readings();
                readings.anchor_reduced() && readings.clause_reduced() && readings.pin_reduced() && !readings.discriminating_arm_equal()
            }
            PositiveControl::BumpRetirementRootSeparator => {
                corrupted.first_cell_row("L03-17", "retirement_root").is_some_and(|row| !row.is_equal() && row.actual.contains("/separator2/"))
            }
        }
    }
    fn applies_to(self, history: &HistoryExport) -> bool {
        match self {
            PositiveControl::ShiftMountSegmentEndOneStepLater | PositiveControl::ShiftMountSegmentEndToItsLastWrite => {
                history.segments.iter().any(|segment| segment.publish_kind() == "mount_writes_before_its_first_publish")
            }
            PositiveControl::CopySequentialWriteTimeIntoTheIntentRecord => history.configuration.contains_key("intent_write_time_seconds"),
            // 只在窗口里写过退役单元的历史上适用（H1、H2；H3 的叶里还有 2 号、不写退役记录，第四节第 8 条）：按字节认，不按历史名。
            PositiveControl::BumpRetirementDeathTxg => history_writes_a_retirement_unit(history, 3),
            PositiveControl::BumpRetirementRootSeparator => history_writes_a_retirement_unit(history, 2),
            PositiveControl::FlipIntentNlink
            | PositiveControl::NonzeroDeletedInodesPadding
            | PositiveControl::BumpTombstoneDeathTxg
            | PositiveControl::DropIntentFirstBarrier
            | PositiveControl::RewriteIntentTimeThresholdAsSeconds
            | PositiveControl::ReduceIntentTagSixByOneRecordsRetirementShare => true,
        }
    }
    /// 不适用时报的原因（`name=r_pc … applies=false reason=`）。
    fn reason_not_applicable(self) -> &'static str {
        match self {
            PositiveControl::ShiftMountSegmentEndOneStepLater | PositiveControl::ShiftMountSegmentEndToItsLastWrite => "no_mount_writes_before_its_first_publish",
            PositiveControl::CopySequentialWriteTimeIntoTheIntentRecord => "impl_config_without_intent_write_time_seconds",
            PositiveControl::BumpRetirementDeathTxg => "no_retirement_container_written",
            PositiveControl::BumpRetirementRootSeparator => "no_retirement_root_written",
            PositiveControl::FlipIntentNlink
            | PositiveControl::NonzeroDeletedInodesPadding
            | PositiveControl::BumpTombstoneDeathTxg
            | PositiveControl::DropIntentFirstBarrier
            | PositiveControl::RewriteIntentTimeThresholdAsSeconds
            | PositiveControl::ReduceIntentTagSixByOneRecordsRetirementShare => "always_applies",
        }
    }
}

/// 窗口里有没有一次写是退役树的单元（码 3 看偏移 43 的出生树，码 2 看偏移 42 的树 ID；同 `decode_unit` 的读法）。
fn history_writes_a_retirement_unit(history: &HistoryExport, unit_type_tag: u8) -> bool {
    let tree_offset = if unit_type_tag == 3 { 43 } else { 42 };
    history.operations.iter().any(|operation| {
        matches!(operation, WindowOperation::Write(write)
            if write.bytes.len() >= NODE_BYTES && &write.bytes[0..4] == UNIT_MAGIC && write.bytes[6] == unit_type_tag && read_u64(&write.bytes, tree_offset) == RETIREMENT_TREE)
    })
}

/// PC-K 的目标行：意图那一版的 A-tag6、L03-13、钉值层与 R-ret 判别臂那一份，各是（期望，解出的）。
struct IntentTagSixReadings {
    anchor: (Option<u64>, Option<u64>),
    clause: (Option<u64>, Option<u64>),
    /// 钉值层只比 H1、H2；别的历史这一格是 `None`（不比）。
    pin: Option<(Option<u64>, Option<u64>)>,
    discriminating_arm: (Option<u64>, Option<u64>),
}

impl IntentTagSixReadings {
    fn pair_equal(pair: (Option<u64>, Option<u64>)) -> bool {
        pair.0.is_some() && pair.0 == pair.1
    }
    fn pair_reduced(pair: (Option<u64>, Option<u64>)) -> bool {
        pair.0.and_then(|expected| expected.checked_sub(POSITIVE_CONTROL_TAG_SIX_REDUCTION_BYTES)).is_some_and(|reduced| Some(reduced) == pair.1)
    }
    fn anchor_equal(&self) -> bool {
        Self::pair_equal(self.anchor)
    }
    fn clause_equal(&self) -> bool {
        Self::pair_equal(self.clause)
    }
    fn pin_equal(&self) -> bool {
        self.pin.is_none_or(Self::pair_equal)
    }
    fn anchor_reduced(&self) -> bool {
        Self::pair_reduced(self.anchor)
    }
    fn clause_reduced(&self) -> bool {
        Self::pair_reduced(self.clause)
    }
    fn pin_reduced(&self) -> bool {
        self.pin.is_none_or(Self::pair_reduced)
    }
    fn discriminating_arm_equal(&self) -> bool {
        Self::pair_equal(self.discriminating_arm)
    }
}

impl HistoryJudgement {
    fn first_cell_row(&self, cell: &str, field: &str) -> Option<&CellRow> {
        self.cell_rows.iter().find(|row| row.cell == cell && row.field == field)
    }
    fn intent_tag_six_readings(&self) -> IntentTagSixReadings {
        let intent = self.tag_six_rows.iter().find(|row| row.publish == DeletionPublish::Intent);
        let actual = intent.and_then(|row| row.actual);
        let clause_row = self.cell_rows.iter().find(|row| row.cell == "L03-13" && row.publish_kind == "unlink_intent");
        let without_retirement = intent.map(|row| retirement_arm_expectations(row.tombstone_levels, row.retirement_levels, row.live_inodes, row.inode_records).1);
        IntentTagSixReadings {
            anchor: (registered_committed_reservation_of_publish(&self.name, DeletionPublish::Intent), actual),
            clause: (
                clause_row.and_then(|row| row.expected.parse::<u64>().ok()),
                clause_row.and_then(|row| row.actual.parse::<u64>().ok()),
            ),
            pin: history_is_compared_with_the_harness_pins(&self.name).then(|| (harness_pinned_value(harness_index_of(DeletionPublish::Intent)), actual)),
            discriminating_arm: (without_retirement, actual),
        }
    }
}

fn intent_range(history: &HistoryExport) -> Option<(usize, usize)> {
    history.segments.iter().find(|segment| segment.publish_kind() == "unlink_intent").map(|segment| (segment.start_step, segment.end_step))
}

/// 改坏一份副本；改不到（找不到目标单元）交回 `None`。
pub(super) fn corrupted_copy(history: &HistoryExport, control: PositiveControl) -> Option<HistoryExport> {
    let mut copy = history.clone();
    let (intent_start, intent_end) = intent_range(history)?;
    let mut touched = 0usize;
    match control {
        PositiveControl::ShiftMountSegmentEndOneStepLater | PositiveControl::ShiftMountSegmentEndToItsLastWrite => {
            let position = copy.segments.iter().position(|segment| segment.publish_kind() == "mount_writes_before_its_first_publish")?;
            let next_position = copy.segments.iter().position(|segment| segment.start_step == copy.segments[position].end_step)?;
            let new_boundary = if control == PositiveControl::ShiftMountSegmentEndToItsLastWrite {
                last_write_step_of(history, &copy.segments[position])?
            } else {
                copy.segments[position].end_step + 1
            };
            if new_boundary <= copy.segments[position].start_step || new_boundary >= copy.segments[next_position].end_step {
                return None;
            }
            copy.segments[position].end_step = new_boundary;
            copy.segments[next_position].start_step = new_boundary;
            touched += 1;
        }
        PositiveControl::RewriteIntentTimeThresholdAsSeconds => {
            let changed_step = *intent_system_configuration_steps(history).first()?;
            for operation in &mut copy.operations {
                let WindowOperation::Write(write) = operation else { continue };
                if write.step != changed_step {
                    continue;
                }
                let threshold_end = SYSTEM_CONFIGURATION_TIME_THRESHOLD_OFFSET + 4;
                write.bytes[SYSTEM_CONFIGURATION_TIME_THRESHOLD_OFFSET..threshold_end].copy_from_slice(&5u32.to_le_bytes());
                reseal_system_configuration_slot(&mut write.bytes);
                touched += 1;
            }
        }
        PositiveControl::DropIntentFirstBarrier => {
            let first_barrier_steps: Vec<usize> = copy
                .operations
                .iter()
                .filter(|operation| matches!(operation, WindowOperation::Barrier { .. }) && operation.step() >= intent_start && operation.step() < intent_end)
                .map(WindowOperation::step)
                .take(2)
                .collect();
            copy.operations.retain(|operation| !first_barrier_steps.contains(&operation.step()));
            touched += first_barrier_steps.len();
        }
        // PC-J、PC-L：窗口里每一份退役单元（两块盘各一份；这三段历史里只有最后一批写它们）。
        PositiveControl::BumpRetirementDeathTxg | PositiveControl::BumpRetirementRootSeparator => {
            for operation in &mut copy.operations {
                let WindowOperation::Write(write) = operation else { continue };
                if write.bytes.len() < NODE_BYTES || &write.bytes[0..4] != UNIT_MAGIC {
                    continue;
                }
                let bytes = &mut write.bytes;
                match (control, bytes[6]) {
                    (PositiveControl::BumpRetirementDeathTxg, 3) if read_u64(bytes, 43) == RETIREMENT_TREE => {
                        // layout/03 第 74 行：第一条退役记录偏移 26 起 8 字节是死亡 txg。
                        let death_offset = PACKED_UNIT_RECORDS_OFFSET + 26;
                        let bumped = read_u64(bytes, death_offset) + 1;
                        bytes[death_offset..death_offset + 8].copy_from_slice(&bumped.to_le_bytes());
                        let checksum = castagnoli_crc32(&bytes[PACKED_UNIT_PLAINTEXT_HEADER_BYTES..]);
                        bytes[89..93].copy_from_slice(&checksum.to_le_bytes());
                        touched += 1;
                    }
                    (PositiveControl::BumpRetirementRootSeparator, 2) if read_u64(bytes, 42) == RETIREMENT_TREE => {
                        let key_width = usize::from(bytes[51]);
                        let header_end = index_node_header_bytes(key_width);
                        let entries_offset = header_end + RESERVED_NONCE_MAC_ALGORITHM_BYTES;
                        let bumped_separator = read_u64(bytes, entries_offset) + 1;
                        bytes[entries_offset..entries_offset + 8].copy_from_slice(&bumped_separator.to_le_bytes());
                        let checksum = castagnoli_crc32(&bytes[header_end..]);
                        bytes[76 + 2 * key_width..80 + 2 * key_width].copy_from_slice(&checksum.to_le_bytes());
                        touched += 1;
                    }
                    _ => {}
                }
            }
        }
        PositiveControl::FlipIntentNlink
        | PositiveControl::NonzeroDeletedInodesPadding
        | PositiveControl::BumpTombstoneDeathTxg
        | PositiveControl::CopySequentialWriteTimeIntoTheIntentRecord
        | PositiveControl::ReduceIntentTagSixByOneRecordsRetirementShare => {
            let sequential_seconds = history.configuration.get("sequential_write_time_seconds").and_then(|text| text.parse::<u64>().ok());
            for operation in &mut copy.operations {
                let WindowOperation::Write(write) = operation else { continue };
                if write.step < intent_start || write.step >= intent_end || write.bytes.len() < NODE_BYTES || &write.bytes[0..4] != UNIT_MAGIC {
                    continue;
                }
                let bytes = &mut write.bytes;
                let unit_type_tag = bytes[6];
                match control {
                    PositiveControl::FlipIntentNlink if unit_type_tag == 3 && read_u64(bytes, 43) == INODE_TREE => {
                        let record_count = usize_of(read_u16(bytes, 69));
                        for record_index in 0..record_count {
                            let record_start = PACKED_UNIT_RECORDS_OFFSET + record_index * INODE_RECORD_BYTES;
                            if read_u64(bytes, record_start) == DELETED_INODE {
                                bytes[record_start + 36] ^= 0x01;
                                touched += 1;
                            }
                        }
                    }
                    PositiveControl::BumpTombstoneDeathTxg if unit_type_tag == 3 && read_u64(bytes, 43) == TOMBSTONE_TREE => {
                        let death_offset = PACKED_UNIT_RECORDS_OFFSET + 32;
                        let bumped = read_u64(bytes, death_offset) + 1;
                        bytes[death_offset..death_offset + 8].copy_from_slice(&bumped.to_le_bytes());
                        let checksum = castagnoli_crc32(&bytes[PACKED_UNIT_PLAINTEXT_HEADER_BYTES..]);
                        bytes[89..93].copy_from_slice(&checksum.to_le_bytes());
                        touched += 1;
                    }
                    PositiveControl::NonzeroDeletedInodesPadding if unit_type_tag == 2 && read_u64(bytes, 42) == DELETED_INODES_TREE => {
                        let key_width = usize::from(bytes[51]);
                        bytes[NODE_BYTES - 1] = 0x01;
                        let header_end = index_node_header_bytes(key_width);
                        let checksum = castagnoli_crc32(&bytes[header_end..]);
                        bytes[76 + 2 * key_width..80 + 2 * key_width].copy_from_slice(&checksum.to_le_bytes());
                        touched += 1;
                    }
                    PositiveControl::CopySequentialWriteTimeIntoTheIntentRecord if unit_type_tag == 3 && read_u64(bytes, 43) == INODE_TREE => {
                        let Some(sequential_seconds) = sequential_seconds else { continue };
                        let record_count = usize_of(read_u16(bytes, 69));
                        for record_index in 0..record_count {
                            let record_start = PACKED_UNIT_RECORDS_OFFSET + record_index * INODE_RECORD_BYTES;
                            if read_u64(bytes, record_start) == DELETED_INODE {
                                for time_offset in [64usize, 72, 80] {
                                    bytes[record_start + time_offset..record_start + time_offset + 8].copy_from_slice(&sequential_seconds.to_le_bytes());
                                }
                                touched += 1;
                            }
                        }
                        let checksum = castagnoli_crc32(&bytes[PACKED_UNIT_PLAINTEXT_HEADER_BYTES..]);
                        bytes[89..93].copy_from_slice(&checksum.to_le_bytes());
                    }
                    // 记账条目：key 打头 2 字节是统计标签，value 在条目偏移 22（layout/01 第 303–305 行；同 `accounting_value`）。
                    PositiveControl::ReduceIntentTagSixByOneRecordsRetirementShare if unit_type_tag == 2 && read_u64(bytes, 42) == ACCOUNTING_TREE => {
                        let key_width = usize::from(bytes[51]);
                        let header_end = index_node_header_bytes(key_width);
                        let entries_offset = header_end + RESERVED_NONCE_MAC_ALGORITHM_BYTES;
                        let entry_count = usize_of(read_u16(bytes, 82 + 2 * key_width));
                        let entry_width = usize_of(read_u16(bytes, 84 + 2 * key_width));
                        for entry_index in 0..entry_count {
                            let entry_start = entries_offset + entry_index * entry_width;
                            if entry_start + entry_width > NODE_BYTES || read_u16(bytes, entry_start) != 6 {
                                continue;
                            }
                            let value_offset = entry_start + 22;
                            let reduced = read_u64(bytes, value_offset).saturating_sub(POSITIVE_CONTROL_TAG_SIX_REDUCTION_BYTES);
                            bytes[value_offset..value_offset + 8].copy_from_slice(&reduced.to_le_bytes());
                            touched += 1;
                        }
                        let checksum = castagnoli_crc32(&bytes[header_end..]);
                        bytes[76 + 2 * key_width..80 + 2 * key_width].copy_from_slice(&checksum.to_le_bytes());
                    }
                    PositiveControl::FlipIntentNlink
                    | PositiveControl::NonzeroDeletedInodesPadding
                    | PositiveControl::BumpTombstoneDeathTxg
                    | PositiveControl::DropIntentFirstBarrier
                    | PositiveControl::ShiftMountSegmentEndOneStepLater
                    | PositiveControl::ShiftMountSegmentEndToItsLastWrite
                    | PositiveControl::RewriteIntentTimeThresholdAsSeconds
                    | PositiveControl::CopySequentialWriteTimeIntoTheIntentRecord
                    | PositiveControl::BumpRetirementDeathTxg
                    | PositiveControl::ReduceIntentTagSixByOneRecordsRetirementShare
                    | PositiveControl::BumpRetirementRootSeparator => {}
                }
            }
        }
    }
    (touched > 0).then_some(copy)
}

/// 意图那次发布里系统配置槽写的步号（升序；按偏移、长度与 magic 判，同 `region_kind_of`）。
fn intent_system_configuration_steps(history: &HistoryExport) -> Vec<usize> {
    let Some((intent_start, intent_end)) = intent_range(history) else { return Vec::new() };
    let mut steps: Vec<usize> = history
        .operations
        .iter()
        .filter_map(|operation| match operation {
            WindowOperation::Write(write) if write.step >= intent_start && write.step < intent_end && region_kind_of(write) == RegionKind::SystemConfigurationSlot => {
                Some(write.step)
            }
            WindowOperation::Write(_) | WindowOperation::Barrier { .. } => None,
        })
        .collect();
    steps.sort_unstable();
    steps
}

/// 一段里最后一次写所在的步号（屏障不算）。
fn last_write_step_of(history: &HistoryExport, segment: &ExportSegment) -> Option<usize> {
    history
        .operations
        .iter()
        .filter_map(|operation| match operation {
            WindowOperation::Write(write) if write.step >= segment.start_step && write.step < segment.end_step => Some(write.step),
            WindowOperation::Write(_) | WindowOperation::Barrier { .. } => None,
        })
        .max()
}

/// 按 layout/01 第 99、118 行重封系统配置整槽校验和：偏移 155 起 32 字节先置 0，对整槽算 CRC32C，写进偏移 155 起 4 字节（小端），其后 28 字节留 0。
pub(super) fn reseal_system_configuration_slot(slot: &mut [u8]) {
    let checksum_end = SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32;
    slot[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..checksum_end].fill(0);
    let slot_checksum = castagnoli_crc32(slot);
    slot[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 4].copy_from_slice(&slot_checksum.to_le_bytes());
}

/// PC-F、PC-G 改坏之后预期的那一次归错，从原样导出与原样判定算出来（跑前登记 e172-r2 第五节 5.4 表）。
fn expected_misattribution_of(history: &HistoryExport, baseline: &HistoryJudgement, control: PositiveControl) -> Option<ExpectedMisattribution> {
    let mount_segment = history.segments.iter().find(|segment| segment.publish_kind() == "mount_writes_before_its_first_publish")?;
    let next_segment = history.segments.iter().find(|segment| segment.start_step == mount_segment.end_step)?;
    let SegmentLabel::Publish { publish_kind: next_kind, txg: next_txg, .. } = &next_segment.label else { return None };
    match control {
        PositiveControl::ShiftMountSegmentEndOneStepLater => Some(ExpectedMisattribution {
            step: mount_segment.end_step,
            exported_segment: "mount_writes_before_its_first_publish".to_string(),
            decoded: Some(WriteAttribution::Publish(*next_txg)),
            forbidden_decoded: None,
        }),
        PositiveControl::ShiftMountSegmentEndToItsLastWrite => {
            let last_write_step = last_write_step_of(history, mount_segment)?;
            let original_attribution = baseline.attribution_rows.iter().find(|row| row.step == last_write_step).map(|row| row.decoded.clone());
            Some(ExpectedMisattribution {
                step: last_write_step,
                exported_segment: format!("{next_kind}_txg{next_txg}"),
                decoded: original_attribution,
                forbidden_decoded: Some(WriteAttribution::Publish(*next_txg)),
            })
        }
        PositiveControl::FlipIntentNlink
        | PositiveControl::NonzeroDeletedInodesPadding
        | PositiveControl::BumpTombstoneDeathTxg
        | PositiveControl::DropIntentFirstBarrier
        | PositiveControl::RewriteIntentTimeThresholdAsSeconds
        | PositiveControl::CopySequentialWriteTimeIntoTheIntentRecord
        | PositiveControl::BumpRetirementDeathTxg
        | PositiveControl::ReduceIntentTagSixByOneRecordsRetirementShare
        | PositiveControl::BumpRetirementRootSeparator => None,
    }
}

/// 一对（对照, 历史）一行；`holds` 为 `None` 是不适用（`applies=false`，不计进 18 对）。
pub(super) fn positive_control_line(history: &HistoryExport, baseline: &HistoryJudgement, control: PositiveControl) -> (String, Option<bool>) {
    if !control.applies_to(history) {
        return (
            format!("name=r_pc pc={} history={} applies=false reason={}", control.name(), history.name, control.reason_not_applicable()),
            None,
        );
    }
    let green_before = control.target_is_green(baseline);
    let expected = expected_misattribution_of(history, baseline, control);
    let red_after = corrupted_copy(history, control).map(|copy| control.target_is_red(&judge_history(&copy), baseline, history, expected.as_ref()));
    let red_after_text = red_after.map_or_else(|| "not_run".to_string(), |red| red.to_string());
    let holds = green_before && red_after == Some(true);
    (
        format!("name=r_pc pc={} history={} applies=true target_green_before={green_before} target_red_after={red_after_text}", control.name(), history.name),
        Some(holds),
    )
}

// ---------------------------------------------------------------------------------------------
// 锚点（跑前登记第七节）与轨迹（第八节 8.1）。
// ---------------------------------------------------------------------------------------------

pub(super) fn anchor_line(anchor: &str, history: &str, detail: &str, expected: impl ToString, actual: impl ToString) -> (String, bool) {
    let expected_text = expected.to_string();
    let actual_text = actual.to_string();
    let equal = expected_text == actual_text;
    (format!("name=anchor anchor={anchor} history={history} {detail} expected={expected_text} actual={actual_text} equal={equal}"), equal)
}

pub(super) fn anchor_lines_of_history(judgement: &HistoryJudgement) -> Vec<(String, bool)> {
    let mut lines = Vec::new();
    let analysis = &judgement.analysis;
    let history = judgement.name.as_str();
    let mut deletion_txgs = Vec::new();
    for (segment_index, segment) in analysis.segments.iter().enumerate() {
        let after = &analysis.states_after[segment_index];
        let decoded_txg = segment.decoded_txg();
        if let Some(txg) = decoded_txg {
            let (expected_device, expected_offset) = expected_root_slot(txg);
            let actual: Vec<String> = segment.regions.iter().filter(|region| region.region_kind == RegionKind::RootRecordSlot).map(|region| format!("{}:{}", region.device, region.offset)).collect();
            lines.push(anchor_line("A-root-slot", history, &format!("txg={txg}"), format!("{expected_device}:{expected_offset}"), actual.join(",")));
        }
        if !segment.is_deletion_publish {
            continue;
        }
        let txg = decoded_txg.unwrap_or(u64::MAX);
        deletion_txgs.push((segment.label_kind(), txg));
        let data_units_after = data_units_of_version(after).unwrap_or(u64::MAX);
        let record = record_of_inode(&records_written_in(segment, INODE_TREE, 2), DELETED_INODE);
        let expected_size = match (segment.label_kind().as_str(), data_units_after) {
            ("unlink_intent", _) => Some(EXPECTED_FILE_BYTES),
            (_, 2) => Some(65268),
            (_, 1) => Some(32634),
            _ => None,
        };
        if let Some(size) = expected_size {
            lines.push(anchor_line("A-size", history, &format!("txg={txg}"), size, field_text(record.as_ref(), 40, 8)));
            lines.push(anchor_line("A-blocks", history, &format!("txg={txg}"), size.div_ceil(512), field_text(record.as_ref(), 48, 8)));
        }
        let registered_tag_six = deletion_publish_of(segment, after)
            .and_then(|publish| registered_committed_reservation_of_publish(history, publish))
            .map_or_else(|| "unknown_publish".to_string(), |value| value.to_string());
        lines.push(anchor_line("A-tag6", history, &format!("txg={txg}"), registered_tag_six, accounting_value(after, 6).map_or_else(|| "absent".to_string(), |value| value.to_string())));
        if history == "h1" || history == "h3" {
            let mut journal_actual: Vec<String> = segment.journal_records.iter().map(|(device, offset, _)| format!("{device}:{offset}")).collect();
            journal_actual.sort();
            let journal_offset = expected_journal_offset_assuming_counter_equals_txg(txg);
            lines.push(anchor_line("A-journal", history, &format!("txg={txg}"), format!("0:{journal_offset},1:{journal_offset}"), journal_actual.join(",")));
            let (generation, slot_offset) = expected_system_configuration_for_txg_in_the_first_instance(txg);
            let mut configuration_actual: Vec<String> = segment.system_configurations.iter().map(|(_, device, offset, configuration)| format!("{device}:{offset}:generation{}", configuration.generation)).collect();
            configuration_actual.sort();
            lines.push(anchor_line(
                "A-system-configuration",
                history,
                &format!("txg={txg}"),
                format!("0:{slot_offset}:generation{generation},1:{slot_offset}:generation{generation}"),
                configuration_actual.join(","),
            ));
        }
    }
    let txgs_text = deletion_txgs.iter().map(|(_, txg)| txg.to_string()).collect::<Vec<_>>().join(",");
    match history {
        "h1" => lines.push(anchor_line("A-txg", history, "deletion_publishes", "5,6,7,8", txgs_text)),
        "h3" => lines.push(anchor_line("A-txg", history, "deletion_publishes", "6,7,8,9", txgs_text)),
        "h2" => lines.extend(continuation_txg_anchor_lines(history, analysis, &deletion_txgs)),
        unknown => lines.push(anchor_line("A-history", unknown, "history_name", "h1_or_h2_or_h3", unknown)),
    }
    lines
}

/// 跑前登记 e172-r3 第七节 7.1 A-tag6：按（历史，删除类发布）分的字面常量，不由解码的 n_live、n_记录 现算（第五节 5.8 改25）。
/// H1 / H2：n_live 0、n_记录 1，最后一批 0 与 0；H3：n_live 1、n_记录 2，最后一批 1 与 1；两个 H 都是 0。
pub(super) fn registered_committed_reservation_of_publish(history: &str, publish: DeletionPublish) -> Option<u64> {
    match (history, publish) {
        ("h1" | "h2", DeletionPublish::Intent | DeletionPublish::BatchLeavingTwoUnits | DeletionPublish::BatchLeavingOneUnit) => Some(196_678),
        ("h1", DeletionPublish::FinalBatch) => Some(196_608),
        ("h2", DeletionPublish::FinalBatch) => Some(196_608),
        ("h3", DeletionPublish::Intent | DeletionPublish::BatchLeavingTwoUnits | DeletionPublish::BatchLeavingOneUnit) => Some(196_830),
        ("h3", DeletionPublish::FinalBatch) => Some(196_760),
        _ => None,
    }
}

/// 钉值层（跑前登记 e172-r3 第一节 1.2 那张表）：每种删除类发布对 harness 第 198 行的第几格（从 1 数，第 1 格是删除之前那一版、在窗口外）。
pub(super) fn harness_index_of(publish: DeletionPublish) -> usize {
    match publish {
        DeletionPublish::Intent => 2,
        DeletionPublish::BatchLeavingTwoUnits => 3,
        DeletionPublish::BatchLeavingOneUnit => 4,
        DeletionPublish::FinalBatch => 5,
    }
}

/// harness 第 198 行第 `harness_index` 格（从 1 数）钉的值。
pub(super) fn harness_pinned_value(harness_index: usize) -> Option<u64> {
    harness_index.checked_sub(1).and_then(|position| HARNESS_PINNED_TAG_SIX_VALUES.get(position)).copied()
}

/// 钉值层只拿与 harness 那条用例同一种历史（H1）和同几何的挂载续做（H2）比；H3 不比（第一节 1.2）。
pub(super) fn history_is_compared_with_the_harness_pins(history: &str) -> bool {
    history == "h1" || history == "h2"
}

/// H2 的 A-txg（第 1 次登记第七节）：意图 5；续做三批的 txg 连着、第一批 = 挂载最后一次暖机 txg + 1。
fn continuation_txg_anchor_lines(history: &str, analysis: &HistoryAnalysis, deletion_txgs: &[(String, u64)]) -> Vec<(String, bool)> {
    let mut lines = Vec::new();
    let intent_txg = deletion_txgs.first().map_or(u64::MAX, |(_, txg)| *txg);
    lines.push(anchor_line("A-txg", history, "intent", 5, intent_txg));
    let continuation: Vec<u64> = deletion_txgs.iter().filter(|(kind, _)| kind == "mount_continuation_unlink_batch").map(|(_, txg)| *txg).collect();
    let last_warm_up = analysis.segments.iter().filter(|segment| segment.label_kind() == "mount_warm_up").filter_map(SegmentAnalysis::decoded_txg).max();
    let consecutive = continuation.windows(2).all(|pair| pair[1] == pair[0] + 1);
    lines.push(anchor_line("A-txg", history, "continuation_consecutive", true, consecutive));
    lines.push(anchor_line(
        "A-txg",
        history,
        "continuation_first_after_last_warm_up",
        last_warm_up.map_or_else(|| "absent".to_string(), |txg| (txg + 1).to_string()),
        continuation.first().map_or_else(|| "absent".to_string(), u64::to_string),
    ));
    lines
}

/// R-traj：逐版（每次发布之后）报队列里的号数、剩下的数据单元数、记账标签 6 与 4；再报峰值、为正的版本数、期末值。
pub(super) fn trajectory_lines(judgement: &HistoryJudgement) -> Vec<String> {
    let analysis = &judgement.analysis;
    let mut lines = Vec::new();
    let mut series: BTreeMap<&'static str, Vec<u64>> = BTreeMap::new();
    for (segment_index, segment) in analysis.segments.iter().enumerate() {
        if !matches!(segment.label, SegmentLabel::Publish { .. }) {
            continue;
        }
        let after = &analysis.states_after[segment_index];
        let queue = u64_of(deleted_inodes_of_version(after).len());
        let data_units = data_units_of_version(after).unwrap_or(u64::MAX);
        let tag_six = accounting_value(after, 6).unwrap_or(u64::MAX);
        let tag_four = accounting_value(after, 4).unwrap_or(u64::MAX);
        // 跑前登记 e172-r3 第五节 5.8 改29：标签 6 式子的三个新输入逐版照报。
        let inode_records = inode_record_count(after);
        let retirement_records = retirement_record_count_of_version(after);
        let retirement_levels = retirement_internal_levels(after);
        lines.push(format!(
            "name=r_traj history={} txg={} publish={} deleted_inodes={queue} data_units={data_units} tag6={tag_six} tag4={tag_four} inode_records={inode_records} retirement_records={retirement_records} retirement_levels={retirement_levels}",
            judgement.name,
            segment.label_txg(),
            segment.label_kind()
        ));
        for (series_name, value) in [
            ("deleted_inodes", queue),
            ("data_units", data_units),
            ("tag6", tag_six),
            ("tag4", tag_four),
            ("inode_records", inode_records),
            ("retirement_records", retirement_records),
            ("retirement_levels", retirement_levels),
        ] {
            series.entry(series_name).or_default().push(value);
        }
    }
    for (series_name, values) in series {
        lines.push(format!(
            "name=r_traj_summary history={} series={series_name} peak={} positive_versions={} final={}",
            judgement.name,
            values.iter().max().copied().unwrap_or(0),
            values.iter().filter(|value| **value > 0).count(),
            values.last().copied().unwrap_or(0)
        ));
    }
    lines
}
