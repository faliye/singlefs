//! 校验和与摘要、小端读取、导出文件的段与历史的结构，以及导出文件的解析。

use super::*;

// ---------------------------------------------------------------------------------------------
// CRC32C 与 SHA-256（产物里记导出文件的 sha256）。
// ---------------------------------------------------------------------------------------------

fn castagnoli_crc32_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    for (table_index, slot) in table.iter_mut().enumerate() {
        let mut remainder = u32::try_from(table_index).expect("表下标小于 256");
        for _ in 0..8 {
            remainder = if remainder & 1 == 1 { (remainder >> 1) ^ 0x82F6_3B78 } else { remainder >> 1 };
        }
        *slot = remainder;
    }
    table
}

pub(super) fn castagnoli_crc32(bytes: &[u8]) -> u32 {
    let table = castagnoli_crc32_table();
    let mut register = 0xFFFF_FFFFu32;
    for byte in bytes {
        register = table[usize::from(register.to_le_bytes()[0] ^ byte)] ^ (register >> 8);
    }
    register ^ 0xFFFF_FFFF
}

pub(super) fn sha256_hexadecimal(bytes: &[u8]) -> String {
    let mut state: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];
    let mut message = bytes.to_vec();
    let bit_length = u64::try_from(bytes.len()).expect("长度装得进 u64") * 8;
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bit_length.to_be_bytes());
    for block in message.chunks(64) {
        let mut schedule = [0u32; 64];
        for (word_index, word) in block.chunks(4).enumerate() {
            schedule[word_index] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for round in 16..64 {
            let sigma_zero = schedule[round - 15].rotate_right(7) ^ schedule[round - 15].rotate_right(18) ^ (schedule[round - 15] >> 3);
            let sigma_one = schedule[round - 2].rotate_right(17) ^ schedule[round - 2].rotate_right(19) ^ (schedule[round - 2] >> 10);
            schedule[round] = schedule[round - 16].wrapping_add(sigma_zero).wrapping_add(schedule[round - 7]).wrapping_add(sigma_one);
        }
        let mut working = state;
        for round in 0..64 {
            let big_sigma_one = working[4].rotate_right(6) ^ working[4].rotate_right(11) ^ working[4].rotate_right(25);
            let choice = (working[4] & working[5]) ^ (!working[4] & working[6]);
            let first_sum = working[7].wrapping_add(big_sigma_one).wrapping_add(choice).wrapping_add(SHA256_ROUND_CONSTANTS[round]).wrapping_add(schedule[round]);
            let big_sigma_zero = working[0].rotate_right(2) ^ working[0].rotate_right(13) ^ working[0].rotate_right(22);
            let majority = (working[0] & working[1]) ^ (working[0] & working[2]) ^ (working[1] & working[2]);
            let second_sum = big_sigma_zero.wrapping_add(majority);
            working[7] = working[6];
            working[6] = working[5];
            working[5] = working[4];
            working[4] = working[3].wrapping_add(first_sum);
            working[3] = working[2];
            working[2] = working[1];
            working[1] = working[0];
            working[0] = first_sum.wrapping_add(second_sum);
        }
        for (state_word, working_word) in state.iter_mut().zip(working.iter()) {
            *state_word = state_word.wrapping_add(*working_word);
        }
    }
    state.iter().map(|word| format!("{word:08x}")).collect()
}

// ---------------------------------------------------------------------------------------------
// 小端读数与十六进制。
// ---------------------------------------------------------------------------------------------

pub(super) fn read_little_endian(bytes: &[u8], offset: usize, width: usize) -> u64 {
    let mut value = 0u64;
    for (byte_index, byte) in bytes[offset..offset + width].iter().enumerate() {
        value |= u64::from(*byte) << (8 * byte_index);
    }
    value
}

pub(super) fn read_u8(bytes: &[u8], offset: usize) -> u8 {
    bytes[offset]
}

pub(super) fn read_u16(bytes: &[u8], offset: usize) -> u64 {
    read_little_endian(bytes, offset, 2)
}

pub(super) fn read_u32(bytes: &[u8], offset: usize) -> u64 {
    read_little_endian(bytes, offset, 4)
}

pub(super) fn read_unsigned_48_bit(bytes: &[u8], offset: usize) -> u64 {
    read_little_endian(bytes, offset, 6)
}

pub(super) fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    read_little_endian(bytes, offset, 8)
}

pub(super) fn hexadecimal_of(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn bytes_of_hexadecimal(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|character_offset| u8::from_str_radix(&text[character_offset..character_offset + 2], 16).ok())
        .collect()
}

pub(super) fn usize_of(value: u64) -> usize {
    usize::try_from(value).expect("盘上读出的长度与条数装得进 usize")
}

pub(super) fn u64_of(value: usize) -> u64 {
    u64::try_from(value).expect("usize 装得进 u64")
}

/// 一个字节区间全是 0。
pub(super) fn is_all_zero(bytes: &[u8]) -> bool {
    bytes.iter().all(|byte| *byte == 0)
}

// ---------------------------------------------------------------------------------------------
// 读导出：只读行的形态，不带任何对字节的解读。
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub(super) struct RegionWrite {
    pub(super) step: usize,
    pub(super) device: u32,
    pub(super) offset: u64,
    pub(super) is_force_unit_access: bool,
    pub(super) bytes: Vec<u8>,
}

#[derive(Clone, Debug)]
pub(super) enum WindowOperation {
    Write(RegionWrite),
    Barrier { step: usize, device: u32 },
}

impl WindowOperation {
    pub(super) fn step(&self) -> usize {
        match self {
            WindowOperation::Write(write) => write.step,
            WindowOperation::Barrier { step, .. } => *step,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum SegmentLabel {
    Publish { publish_kind: String, txg: u64, rewritten_roles: Vec<String>, data_units_after: u64, deleted_inodes_after: u64 },
    Outside { segment_name: String },
}

#[derive(Clone, Debug)]
pub(super) struct ExportSegment {
    pub(super) start_step: usize,
    pub(super) end_step: usize,
    pub(super) label: SegmentLabel,
}

impl ExportSegment {
    pub(super) fn is_deletion_publish(&self) -> bool {
        match &self.label {
            SegmentLabel::Publish { publish_kind, .. } => {
                publish_kind == "unlink_intent" || publish_kind == "unlink_batch" || publish_kind == "mount_continuation_unlink_batch"
            }
            SegmentLabel::Outside { .. } => false,
        }
    }
    pub(super) fn publish_kind(&self) -> &str {
        match &self.label {
            SegmentLabel::Publish { publish_kind, .. } => publish_kind,
            SegmentLabel::Outside { segment_name } => segment_name,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(super) struct HistoryExport {
    pub(super) name: String,
    pub(super) configuration: BTreeMap<String, String>,
    before_window_operations: u64,
    pub(super) operations: Vec<WindowOperation>,
    pub(super) segments: Vec<ExportSegment>,
    pub(super) summary: BTreeMap<String, String>,
    pub(super) boundary_mismatches: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub(super) struct ExportFile {
    pub(super) histories: Vec<HistoryExport>,
    pub(super) line_count: u64,
    pub(super) anchored_line_count: u64,
    pub(super) done_emitted: Option<u64>,
    pub(super) non_hexadecimal_line_count: u64,
    pub(super) malformed_lines: Vec<String>,
}

fn fields_of_line(body: &str) -> BTreeMap<String, String> {
    body.split(' ')
        .filter_map(|field| field.split_once('=').map(|(key, value)| (key.to_string(), value.to_string())))
        .collect()
}

pub(super) fn field_number(fields: &BTreeMap<String, String>, key: &str) -> Option<u64> {
    fields.get(key).and_then(|text| text.parse::<u64>().ok())
}

fn history_mut<'export>(export: &'export mut ExportFile, history_name: &str) -> &'export mut HistoryExport {
    if !export.histories.iter().any(|history| history.name == history_name) {
        export.histories.push(HistoryExport {
            name: history_name.to_string(),
            configuration: BTreeMap::new(),
            before_window_operations: 0,
            operations: Vec::new(),
            segments: Vec::new(),
            summary: BTreeMap::new(),
            boundary_mismatches: Vec::new(),
        });
    }
    export.histories.iter_mut().find(|history| history.name == history_name).expect("刚放进去")
}

fn label_of_boundary(fields: &BTreeMap<String, String>) -> SegmentLabel {
    if fields.get("name").map(String::as_str) == Some("publish_boundary") {
        SegmentLabel::Publish {
            publish_kind: fields.get("publish").cloned().unwrap_or_default(),
            txg: field_number(fields, "txg").unwrap_or(u64::MAX),
            rewritten_roles: fields
                .get("rewritten_roles")
                .map(|text| text.split(',').filter(|role| !role.is_empty()).map(str::to_string).collect())
                .unwrap_or_default(),
            data_units_after: field_number(fields, "data_units_after").unwrap_or(u64::MAX),
            deleted_inodes_after: field_number(fields, "deleted_inodes_after").unwrap_or(u64::MAX),
        }
    } else {
        SegmentLabel::Outside { segment_name: fields.get("segment").cloned().unwrap_or_default() }
    }
}

/// 把导出全文读成按历史分开的步序列与段。边界行的 before / after 两行要逐字段相同（step 除外），不同的记进 `boundary_mismatches`。
pub(super) fn parse_export(text: &str) -> ExportFile {
    let mut export = ExportFile::default();
    let mut open_boundaries: BTreeMap<String, (usize, BTreeMap<String, String>)> = BTreeMap::new();
    for line in text.lines() {
        export.line_count += 1;
        let Some(body) = line.strip_prefix("E7RESULT ") else {
            export.malformed_lines.push(format!("没有 E7RESULT 前缀：{}", line.chars().take(80).collect::<String>()));
            continue;
        };
        export.anchored_line_count += 1;
        let fields = fields_of_line(body);
        let line_name = fields.get("name").cloned().unwrap_or_default();
        if line_name != "device_region_bytes" {
            export.non_hexadecimal_line_count += 1;
        }
        if line_name == "done" {
            export.done_emitted = field_number(&fields, "emitted");
            continue;
        }
        let history_name = fields.get("history").cloned().unwrap_or_default();
        match line_name.as_str() {
            "impl_config" => history_mut(&mut export, &history_name).configuration = fields.clone(),
            "before_window_summary" => {
                history_mut(&mut export, &history_name).before_window_operations = field_number(&fields, "operations").unwrap_or(u64::MAX);
            }
            "device_region_bytes_summary" => history_mut(&mut export, &history_name).summary = fields.clone(),
            "window_barrier" => {
                let step = usize_of(field_number(&fields, "step").unwrap_or(u64::MAX >> 1));
                let device = u32::try_from(field_number(&fields, "device").unwrap_or(u64::MAX)).unwrap_or(u32::MAX);
                history_mut(&mut export, &history_name).operations.push(WindowOperation::Barrier { step, device });
            }
            "device_region_bytes" => {
                let bytes = fields.get("hexadecimal").and_then(|hexadecimal_text| bytes_of_hexadecimal(hexadecimal_text));
                let step = field_number(&fields, "step");
                let device = field_number(&fields, "device").and_then(|value| u32::try_from(value).ok());
                let offset = field_number(&fields, "offset");
                let length = field_number(&fields, "length");
                match (bytes, step, device, offset, length) {
                    (Some(bytes), Some(step), Some(device), Some(offset), Some(length)) if u64_of(bytes.len()) == length => {
                        let is_force_unit_access = fields.get("kind").map(String::as_str) == Some("write_fua");
                        history_mut(&mut export, &history_name).operations.push(WindowOperation::Write(RegionWrite {
                            step: usize_of(step),
                            device,
                            offset,
                            is_force_unit_access,
                            bytes,
                        }));
                    }
                    _ => export.malformed_lines.push(format!("写区域行解不开：history={history_name} step={:?}", fields.get("step"))),
                }
            }
            "publish_boundary" | "writes_outside_any_publish_boundary" => {
                let step = usize_of(field_number(&fields, "step").unwrap_or(u64::MAX >> 1));
                let mut comparable = fields.clone();
                comparable.remove("step");
                comparable.remove("edge");
                let key = history_name.clone();
                match fields.get("edge").map(String::as_str) {
                    Some("before") => {
                        open_boundaries.insert(key, (step, comparable));
                    }
                    Some("after") => {
                        let opened = open_boundaries.remove(&key);
                        let history = history_mut(&mut export, &history_name);
                        match opened {
                            Some((start_step, before_fields)) => {
                                if before_fields != comparable {
                                    history.boundary_mismatches.push(format!("step {start_step}..{step} 前后两行字段不同"));
                                }
                                history.segments.push(ExportSegment { start_step, end_step: step, label: label_of_boundary(&fields) });
                            }
                            None => history.boundary_mismatches.push(format!("step {step} 的 after 行前面没有 before 行")),
                        }
                    }
                    _ => export.malformed_lines.push(format!("边界行没有 edge：history={history_name}")),
                }
            }
            other => export.malformed_lines.push(format!("认不出的行：name={other}")),
        }
    }
    for (history_name, (start_step, _)) in open_boundaries {
        history_mut(&mut export, &history_name).boundary_mismatches.push(format!("step {start_step} 的 before 行没有 after 行"));
    }
    for history in &mut export.histories {
        history.operations.sort_by_key(WindowOperation::step);
    }
    export
}
