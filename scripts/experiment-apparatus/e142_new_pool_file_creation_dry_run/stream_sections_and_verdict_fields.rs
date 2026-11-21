//! 流的判定字段、回退下界与卸载记号的敏感行、流的整段、P8G 几何的各行。

use e7_index_bench::usize_from_u64;
use super::*;

// ═════════ E142 第二十二次跑第二段（登记 5.4 ⑥）：第二个几何 P8G ═════════
// 一条流（mkfs → 取号 → 暖机 → 新池新建文件）在给定几何上打出哪些行、判决行里「甲类」字段取什么值。
// 行体与 `main()` 对主点打的同名行是同一种写法（`main()` 那一段一字不改，Q142.101 守着主点的输出没动）；
// 区别只有两处：这里把行收进一个向量，调用处按需加 `p8g_` 前缀再打；产物自检（角色名、文本数字）扫的是这个向量，不是 `EMITTED_LINES`。

/// 判决行里「甲类」字段（登记 5.4 ⑥(iv)，分类全文在第二段修订）：值由这条流写出的字节、读回或这条流打印的行算出，
/// 字段名与次序同主点 `name=verdict` 里的同名字段。输入是一盘对照、G 系别的点、层 0、宽度常量的另外 13 个归乙，不在这里。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct StreamVerdictFields {
    write_list_ok: bool,
    recover_full_ok: bool,
    positive_control_main_geometry_ok: bool,
    role_label_contradictions_zero: bool,
    role_labels_all_defined: bool,
    text_number_mismatches_zero: bool,
    text_numbers_all_classified: bool,
    named_instances_ok: bool,
    clause_fields_system_configuration_ok: bool,
    clause_fields_root_flags_ok: bool,
    clause_fields_rollback_floor_ok: bool,
    clause_fields_unmount_marker_ok: bool,
}

impl StreamVerdictFields {
    fn as_named_fields(&self) -> [(&'static str, bool); 12] {
        [
            ("write_list_ok", self.write_list_ok),
            ("recover_full_ok", self.recover_full_ok),
            ("positive_control_main_geometry_ok", self.positive_control_main_geometry_ok),
            ("role_label_contradictions_zero", self.role_label_contradictions_zero),
            ("role_labels_all_defined", self.role_labels_all_defined),
            ("text_number_mismatches_zero", self.text_number_mismatches_zero),
            ("text_numbers_all_classified", self.text_numbers_all_classified),
            ("named_instances_ok", self.named_instances_ok),
            ("r18_system_configuration_fields_ok", self.clause_fields_system_configuration_ok),
            ("r18_root_flags_ok", self.clause_fields_root_flags_ok),
            ("r18_g8_ok", self.clause_fields_rollback_floor_ok),
            ("r18_g9_ok", self.clause_fields_unmount_marker_ok),
        ]
    }
}

pub(super) struct StreamSection {
    /// 行体，每行以 `name=` 起头，不带 `E7RESULT ` 前缀。
    rows: Vec<String>,
    pub(super) verdict: StreamVerdictFields,
}

/// 第八节 G8（只在模型里）：窗口里设备 0 的系统配置只换回滚下限 F，与 F = 0 的那一份逐字节比、解析回来。返回（行体，是否通过）。
fn rollback_floor_sensitivity_row(parameters: &PoolParameters, slot_generation: u64, instance: InstanceGeneration) -> (String, bool) {
    let sensitivity_configuration = |rollback_floor: CheckpointTxg| SystemConfiguration {
        fsid: parameters.fsid,
        this_device: DeviceIdentity(0),
        device_count: u32::try_from(parameters.device_count).expect("设备数"),
        slot_generation,
        region_devices: parameters.region_devices,
        journal_tail: NEW_POOL_FILE_CREATION_TXG,
        journal_instance: instance,
        rollback_floor,
    };
    let baseline = sensitivity_configuration(CheckpointTxg(0));
    let modified = sensitivity_configuration(CheckpointTxg(0x0807_0605_0403_0201));
    let baseline_bytes = baseline.to_slot();
    let modified_bytes = modified.to_slot();
    let differing_offsets: Vec<usize> = (0..baseline_bytes.len()).filter(|&index| baseline_bytes[index] != modified_bytes[index]).collect();
    let expected_offsets: Vec<usize> = (155..159).chain(481..489).collect();
    let bytes_hex = hex_bytes(&modified_bytes[SYSTEM_CONFIGURATION_ROLLBACK_FLOOR_OFFSET..SYSTEM_CONFIGURATION_ROLLBACK_FLOOR_OFFSET + 8]);
    let parse_round_trip = SystemConfiguration::parse_slot(&modified_bytes).map(|parsed| parsed.rollback_floor == modified.rollback_floor).unwrap_or(false);
    let offsets_text: Vec<String> = differing_offsets.iter().map(ToString::to_string).collect();
    let row = format!(
        "name=r18_sensitivity point=G8 field=rollback_floor value=0x0807060504030201 bytes_hex={bytes_hex} differing_offsets={} parse_round_trip={parse_round_trip}",
        offsets_text.join(","),
    );
    (row, bytes_hex == "0102030405060708" && differing_offsets == expected_offsets && parse_round_trip)
}

/// 第八节 G9（只在模型里）：新池新建文件的根只把卸载记号置 1，与不置那一份逐字节比、解析回来；另把 flags 改成除位 0 之外的一位（0x02），
/// 重算自证校验和之后必须被拒。返回（行体，是否通过）。
fn unmount_marker_sensitivity_row(root: RootRecord, fsid: &[u8; 16]) -> (String, bool) {
    let baseline = root;
    let mut marked = baseline;
    marked.unmount_marker = true;
    let baseline_bytes = baseline.to_slot();
    let marked_bytes = marked.to_slot();
    let differing_offsets: Vec<usize> = (0..baseline_bytes.len()).filter(|&index| baseline_bytes[index] != marked_bytes[index]).collect();
    let flags_hex = hex_bytes(&marked_bytes[20..24]);
    let parse_marker = RootRecord::parse_slot(&marked_bytes, fsid).map(|parsed| parsed.unmount_marker).unwrap_or(false);
    let mut other_bit_bytes = marked_bytes.clone();
    other_bit_bytes[20..24].copy_from_slice(&0x0000_0002u32.to_le_bytes());
    let recomputed_checksum = wide_checksum_with_field_zeroed(&other_bit_bytes, usize_from_u64(PHYSICAL_BLOCK_BYTES), ROOT_CHECKSUM_OFFSET);
    other_bit_bytes[ROOT_CHECKSUM_OFFSET..ROOT_CHECKSUM_OFFSET + 32].copy_from_slice(&recomputed_checksum);
    let other_bit_rejected = RootRecord::parse_slot(&other_bit_bytes, fsid).is_none();
    let offsets_text: Vec<String> = differing_offsets.iter().map(ToString::to_string).collect();
    let row = format!(
        "name=r18_sensitivity point=G9 field=unmount_marker flags_hex={flags_hex} differing_offsets={} parse_marker={parse_marker} other_bit_rejected={other_bit_rejected}",
        offsets_text.join(","),
    );
    let expected_offsets: Vec<usize> = std::iter::once(20).chain(138..142).collect();
    let ok = flags_hex == "01000000" && differing_offsets.iter().all(|offset| expected_offsets.contains(offset)) && differing_offsets.contains(&20) && parse_marker && other_bit_rejected;
    (row, ok)
}

/// 一条流在给定几何上的产物行与甲类判决字段。
pub(super) fn stream_section(parameters: &PoolParameters, file_bytes: &[u8]) -> StreamSection {
    let (mut recording, genesis) = mkfs(parameters);
    let mkfs_operation_count = recording.operations.len();
    let base_after_mkfs = recording.pool.clone();
    let instance = acquire_instance(&mut recording, parameters);
    let (_, last_warm_up_record) = warm_up(&mut recording, parameters, &genesis, instance);
    let warm_up_operation_count = recording.operations.len();
    let output = publish_first_file(&mut recording, parameters, &genesis, file_bytes, instance, last_warm_up_record.as_deref(), NEW_POOL_FILE_CREATION_TXG);
    let transaction_operations = &recording.operations[warm_up_operation_count..];
    let post_mkfs_operations = &recording.operations[mkfs_operation_count..];
    let write_count = transaction_operations.iter().filter(|operation| matches!(operation, RecordedOperation::Write(_))).count();
    let barrier_count = transaction_operations.iter().filter(|operation| matches!(operation, RecordedOperation::Barrier(_))).count();
    let fua_count = transaction_operations.iter().filter(|operation| matches!(operation, RecordedOperation::Write(write) if write.is_fua())).count();

    let mut rows: Vec<String> = Vec::new();
    let main_unit_count = output.units_by_slot.len();
    let journal_label = format!("t{}", main_unit_count + 1);
    rows.push(format!(
        "name=role_labels scope=main_transaction units={} journal_record={journal_label} root=t{} system_configuration=t{} warm_up_journal_records=w1,w4",
        output.units_by_slot.iter().map(|(_, unit, _)| unit.tag()).collect::<Vec<_>>().join(","),
        main_unit_count + 2,
        main_unit_count + 3
    ));
    let unit_slots: Vec<String> = output.units_by_slot.iter().map(|(slot, unit_identity, unit)| format!("{}@{}x{}", unit_identity.tag(), slot.0, unit.len())).collect();
    rows.push(format!("name=write_list writes={write_count} barriers={barrier_count} fua={fua_count} named={} units={}", output.record.named.len(), unit_slots.join(",")));

    let (root_region, root_slot) = ring_target_for_publish(CheckpointTxg(NEW_POOL_FILE_CREATION_TXG));
    let root_device = parameters.region_devices[usize_from_u64(root_region)].0;
    let root_offset = ring_slot_offset(root_region, root_slot).0;
    let catalog = build_structure_catalog(&output, root_device, root_offset);
    for write in model_window_writes(&catalog, transaction_operations) {
        rows.push(format!(
            "name=device_region_bytes step={} region={} device={} offset={} length={} sha256={} hexadecimal={}",
            write.step, write.region, write.device, write.offset, write.bytes.len(), sha256_hex(&write.bytes), hex_bytes(&write.bytes)
        ));
        rows.push(write_list_row(&catalog, write.step, write.region, write.device, write.offset, &write.bytes));
    }

    let full = recording.pool.clone();
    let full_report = recover(&full, JournalPolicy::Consult, ReaderMode::Clause);
    let content_matches = matches!(&full_report.outcome, RecoveryOutcome::FileRead { content, .. } if content.as_slice() == file_bytes);
    rows.push(format!(
        "name=recover_full outcome={} root={} content_matches={content_matches} valid_records={} above_water={} applied={} verification_passed={} mapping_fallbacks={}",
        outcome_kind(&full_report.outcome), outcome_root(&full_report.outcome), full_report.journal.valid_records, full_report.journal.above_water, full_report.journal.prefix_applied, full_report.journal.verification_passed, full_report.mapping_fallbacks
    ));

    let journal_records = scan_journal(&full, unit_fsid(&parameters.fsid));
    for (label, counter) in journal_header_record_labels(&journal_label) {
        if let Some(record) = journal_records.get(&(InstanceGeneration(1), JournalCounter(counter))) {
            rows.push(format!(
                "name=journal_header_record_bytes arm=F record={label} device=0 flag_byte={:#04x} ordinal_bytes={} back_chain={}",
                record.record_flags, record.ordinal_within_publish, record.back_chain
            ));
        }
    }

    let statistic_value = |statistic: u16| output.accounting_entries.iter().find(|entry| entry.statistic == statistic).map_or(0, |entry| entry.value);
    rows.push(format!(
        "name=accounting entries={} allocated_bytes_per_device={} free_bytes_per_device={} empty_cluster_segments_per_device={} fragmentation_runs_per_device={} inode_watermark={} unreclaimable_per_device={} defer_queue_per_device={} pending_delete={} committed_reservation={} sequence_all_one={} generation={}",
        output.accounting_entries.len(),
        statistic_value(STATISTIC_ALLOCATED_BYTES),
        statistic_value(STATISTIC_FREE_BYTES),
        statistic_value(STATISTIC_EMPTY_CLUSTER_SEGMENTS),
        statistic_value(STATISTIC_FRAGMENTATION_RUNS),
        statistic_value(STATISTIC_INODE_WATERMARK),
        statistic_value(STATISTIC_UNRECLAIMABLE_BYTES),
        statistic_value(STATISTIC_DEFER_QUEUE_BYTES),
        statistic_value(STATISTIC_PENDING_DELETE_BYTES),
        statistic_value(STATISTIC_COMMITTED_RESERVATION_BYTES),
        output.accounting_entries.iter().all(|entry| entry.sequence == ACCOUNTING_SEQUENCE_DIRECT_TO_LEAF),
        output.accounting_entries[0].generation.0
    ));
    let (gap_identifier, gap_template) = GAPS.iter().find(|(identifier, _)| *identifier == "G22").expect("GAPS 里有 G22");
    let free_bytes_per_device = statistic_value(STATISTIC_FREE_BYTES);
    let empty_cluster_segments_per_device = statistic_value(STATISTIC_EMPTY_CLUSTER_SEGMENTS);
    let fragmentation_runs_per_device = statistic_value(STATISTIC_FRAGMENTATION_RUNS);
    rows.push(format!(
        "name=gap id={gap_identifier} text={} bound_free_bytes_per_device={free_bytes_per_device} bound_empty_cluster_segments_per_device={empty_cluster_segments_per_device} bound_fragmentation_runs_per_device={fragmentation_runs_per_device}",
        render_free_bytes_gap_text(gap_template, free_bytes_per_device, empty_cluster_segments_per_device, fragmentation_runs_per_device)
    ));

    let layout_rows = layout_field_rows(&full, &base_after_mkfs);
    for row in &layout_rows {
        let equal = layout_field_equal(&row.expected, &row.actual);
        rows.push(format!(
            "name=layout_field id={} record_or_unit={} field={} expected={} actual={} equal={equal} layout_line={}",
            row.row_identifier, row.record_or_unit, row.field, row.expected, row.actual, row.layout_line
        ));
    }
    let (layout_equal_rows, layout_unequal_rows) = layout_field_tally(&layout_rows);
    rows.push(format!("name=layout_field_summary rows={} equal={layout_equal_rows} unequal={layout_unequal_rows}", layout_rows.len()));

    let (post_mkfs_writes, _) = split_into_segments(post_mkfs_operations, true);
    let pc2 = run_pc2(&base_after_mkfs, &post_mkfs_writes, &output.units_by_slot, file_bytes);
    rows.push(format!(
        "name=positive_control_main_geometry states={} all_persisted_legal={} all_persisted_content_ok={} units={} caught={}",
        pc2.per_unit.len() + 1, pc2.all_persisted_legal, pc2.all_persisted_content_ok, pc2.per_unit.len(), pc2.caught
    ));

    let (window_system_configuration_generation, window_system_configuration_slot_index) = system_configuration_write_for_publish(CheckpointTxg(NEW_POOL_FILE_CREATION_TXG));
    let window_system_configuration_slot_offset = SYSTEM_CONFIGURATION_SLOT_OFFSETS[window_system_configuration_slot_index];
    let mut system_configuration_fields_ok = true;
    for device in parameters.devices() {
        let slot_bytes = full.read(device, DeviceOffset(window_system_configuration_slot_offset), usize_from_u64(SYSTEM_CONFIGURATION_SLOT_BYTES));
        let incompat_byte0_ok = slot_bytes[SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET] == WRITTEN_FIRST_INCOMPAT_BYTE;
        let feature_bits_rest_zero = slot_bytes[SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET + 1..SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET + 3 * FEATURE_BITMAP_BYTES].iter().all(|byte| *byte == 0);
        let rollback_floor_hex = hex_bytes(&slot_bytes[SYSTEM_CONFIGURATION_ROLLBACK_FLOOR_OFFSET..SYSTEM_CONFIGURATION_ROLLBACK_FLOOR_OFFSET + 8]);
        let padding_after_489_zero = slot_bytes[usize_from_u64(SYSTEM_CONFIGURATION_BYTES)..].iter().all(|byte| *byte == 0);
        let checksum_recomputes = wide_checksum_with_field_zeroed(&slot_bytes, usize_from_u64(SYSTEM_CONFIGURATION_SLOT_BYTES), SYSTEM_CONFIGURATION_CHECKSUM_OFFSET)
            == slot_bytes[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32];
        let field_ok = incompat_byte0_ok && feature_bits_rest_zero && rollback_floor_hex == "0000000000000000" && padding_after_489_zero && checksum_recomputes;
        system_configuration_fields_ok &= field_ok;
        rows.push(format!(
            "name=r18_clause_fields region=system_configuration device={} incompat_byte0={:#04x} feature_bits_rest_zero={feature_bits_rest_zero} rollback_floor_hex={rollback_floor_hex} padding_after_489_zero={padding_after_489_zero} checksum_recomputes={checksum_recomputes} ok={field_ok}",
            device.0, slot_bytes[SYSTEM_CONFIGURATION_FEATURE_BITS_OFFSET],
        ));
    }
    let root_record_bytes = full.read(root_device_identity(parameters, root_region), ring_slot_offset(root_region, root_slot), usize_from_u64(PHYSICAL_BLOCK_BYTES));
    let root_flags_hex = hex_bytes(&root_record_bytes[20..24]);
    let root_checksum_recomputes = wide_checksum_with_field_zeroed(&root_record_bytes, usize_from_u64(PHYSICAL_BLOCK_BYTES), ROOT_CHECKSUM_OFFSET) == root_record_bytes[ROOT_CHECKSUM_OFFSET..ROOT_CHECKSUM_OFFSET + 32];
    let root_flags_ok = root_flags_hex == "00000000" && root_checksum_recomputes;
    rows.push(format!(
        "name=r18_clause_fields region=root_record device={} flags_hex={root_flags_hex} checksum_recomputes={root_checksum_recomputes} ok={root_flags_ok}",
        root_device_identity(parameters, root_region).0,
    ));
    let (rollback_floor_row, rollback_floor_ok) = rollback_floor_sensitivity_row(parameters, window_system_configuration_generation, instance);
    rows.push(rollback_floor_row);
    let (unmount_marker_row, unmount_marker_ok) = unmount_marker_sensitivity_row(output.root, &parameters.fsid);
    rows.push(unmount_marker_row);

    let role_label_check = check_role_labels(&rows);
    let text_number_check = check_text_numbers(&rows);
    let journal_header_record_label = find_field_value_excluding(&rows, "journal_header_record_bytes", "record", &["w1", "w4"]).unwrap_or_else(|| "not_found".to_string());
    let write_list_journal_record_label = find_field_value_excluding(&rows, "role_labels", "journal_record", &[]).unwrap_or_else(|| "not_found".to_string());
    let free_bytes_gap_text = rows
        .iter()
        .find(|line| {
            let fields = split_fields(line);
            fields.iter().any(|(key, value)| *key == "name" && *value == "gap") && fields.iter().any(|(key, value)| *key == "id" && *value == "G22")
        })
        .and_then(|line| split_fields(line).iter().find(|(key, _)| *key == "text").map(|(_, value)| (*value).to_string()))
        .unwrap_or_default();
    let free_bytes_in_the_gap_text = extract_number_after_marker(&free_bytes_gap_text, "空闲字节").unwrap_or(0);
    let accounting_free_bytes_per_device: u64 = find_field_value_excluding(&rows, "accounting", "free_bytes_per_device", &[]).and_then(|value| value.parse().ok()).unwrap_or(0);
    let named_instances_ok = journal_header_record_label == write_list_journal_record_label && free_bytes_in_the_gap_text == accounting_free_bytes_per_device;

    let verdict = StreamVerdictFields {
        write_list_ok: new_pool_file_creation_write_list_matches(write_count, barrier_count, fua_count, parameters.device_count),
        recover_full_ok: content_matches,
        positive_control_main_geometry_ok: pc2.caught == 12 && pc2.all_persisted_legal && pc2.all_persisted_content_ok,
        role_label_contradictions_zero: role_label_check.contradictions == 0,
        role_labels_all_defined: role_label_check.undefined == 0,
        text_number_mismatches_zero: text_number_check.bound_mismatched == 0,
        text_numbers_all_classified: text_number_check.unclassified == 0,
        named_instances_ok,
        clause_fields_system_configuration_ok: system_configuration_fields_ok,
        clause_fields_root_flags_ok: root_flags_ok,
        clause_fields_rollback_floor_ok: rollback_floor_ok,
        clause_fields_unmount_marker_ok: unmount_marker_ok,
    };
    StreamSection { rows, verdict }
}

/// 根槽所在设备（环区域 → 设备，D2 已定项 7）。
fn root_device_identity(parameters: &PoolParameters, root_region: u64) -> DeviceIdentity {
    parameters.region_devices[usize_from_u64(root_region)]
}

/// 第二个几何 P8G（第二十二次登记第一节「第 2 行」的读法）：两块盘、每盘 8 GiB，其余同主点。
pub(super) fn p8g_parameters() -> PoolParameters {
    PoolParameters::settled_two_devices_of_device_bytes(EIGHT_GIBIBYTE_POINT_DEVICE_BYTES)
}

/// 行体里的 `name=<名字>` 改成 `name=p8g_<名字>`（只改第一处）。
fn prefix_row_name_with_p8g(row: &str) -> String {
    row.replacen("name=", "name=p8g_", 1)
}

/// P8G 一段要打的全部行体（带 `p8g_` 前缀，不带 `E7RESULT `）与甲类判决字段：config 一行、`stream_section` 在 P8G 参数上的各行、`p8g_verdict` 一行。
pub(super) fn p8g_section_rows(file_bytes: &[u8]) -> (Vec<String>, StreamVerdictFields) {
    let parameters = p8g_parameters();
    let section = stream_section(&parameters, file_bytes);
    let mut rows = vec![format!(
        "name=p8g_config devices={} physical_block_bytes={PHYSICAL_BLOCK_BYTES} file_bytes={} fsid=fixed device_bytes={} device_slots={} unit_area_start_slot={UNIT_AREA_START_SLOT} unit_area_slots={} cluster_segment_slots={CLUSTER_SEGMENT_SLOTS}",
        parameters.device_count, file_bytes.len(), parameters.device_bytes, parameters.device_slots(), parameters.unit_area_slots()
    )];
    rows.extend(section.rows.iter().map(|row| prefix_row_name_with_p8g(row)));
    let verdict_fields: Vec<String> = section.verdict.as_named_fields().iter().map(|(name, value)| format!("{name}={value}")).collect();
    rows.push(format!("name=p8g_verdict {}", verdict_fields.join(" ")));
    (rows, section.verdict)
}

/// 对照（判别力自证）：同一个 `stream_section` 在主点参数上跑出的甲类字段，与 `main()` 自己打印的 `name=verdict` 行里的同名字段逐个比。
/// 返回（相等的字段数，不等的字段名）；打印的行里找不到的字段算不等。
pub(super) fn compare_stream_verdict_fields_to_printed_verdict(printed_verdict_line: &str, fields: &StreamVerdictFields) -> (usize, Vec<&'static str>) {
    let printed = split_fields(printed_verdict_line);
    let mut equal = 0usize;
    let mut unequal = Vec::new();
    for (name, value) in fields.as_named_fields() {
        let printed_value = printed.iter().find(|(key, _)| *key == name).map(|(_, printed_value)| *printed_value);
        if printed_value == Some(if value { "true" } else { "false" }) {
            equal += 1;
        } else {
            unequal.push(name);
        }
    }
    (equal, unequal)
}

#[cfg(test)]
#[path = "stream_sections_and_verdict_fields_tests.rs"]
mod tests;
