//! E172（删除写出的字节的干跑）：第 1 次跑前登记 `research/prompts/e172-preregistration.md`，
//! 第 2 次重跑登记 `research/prompts/e172-r2-prereg.md`（这一份按它第五节 5.8 改1–改15 改过）。
//!
//! 研究侧独立解码器：不 `use` 任何 `singlefs-*` crate、不 include E142 的源码。读 `crates/` 侧只读导出
//! `deletion_write_dump` 的两份输出文件（不带参数的 H1、H2 一份，`--history h3` 一份），把窗口里每个写区域按
//! layout/01、layout/03（连 D18 已定项 7 / 11 / 18 / 24 的带偏移字段表）逐字段解出来，与登记取值、跑前登记第七节的锚点比；
//! 外加覆盖率（每个字节归到哪一格）。不手写独立模型：期望值只来自条款与锚点。
//!
//!   e172-deletion-write-dry-run <H1H2 导出文件> <H3 导出文件>
//!
//! 退出码：0 = 跑完（判定写在产物行里，不进退出码）；2 = 命令行不对或导出读不了。

use e7_index_bench::Emitter;
use std::collections::{BTreeMap, BTreeSet};

// ---------------------------------------------------------------------------------------------
// 偏移与宽度常量：每一个都写出处（文件与行号），值从 kb 抄，不从 `crates/` 引。
// ---------------------------------------------------------------------------------------------

/// `.claude/kb/layout/01-first-txn.md` 第 40–41 行：系统配置两槽，槽宽 4096，槽 1 在偏移 4096。
const SYSTEM_CONFIGURATION_SLOT_BYTES: usize = 4096;
/// layout/01 第 156 行：系统配置字段合计 489 字节，其后到槽末是补齐（第 99 行：整槽校验和覆盖补齐）。
const SYSTEM_CONFIGURATION_FIELD_BYTES: usize = 489;
/// layout/01 第 42–43 行：根环基址 1 MiB、区域 r 在 基址 + r × P × chunk（P = 3、chunk = 1 MiB），每区 8 槽、槽距 4096、槽宽 512。
const ROOT_RING_BASE_BYTES: u64 = 1_048_576;
const ROOT_RING_REGION_STRIDE_BYTES: u64 = 3 * 1_048_576;
const ROOT_RING_SLOTS_PER_REGION: u64 = 8;
const ROOT_RING_SLOT_SPACING_BYTES: u64 = 4096;
const ROOT_RECORD_SLOT_BYTES: usize = 512;
/// layout/01 第 44、162 行：区域归属 0 / 1 / 0。
const ROOT_RING_REGION_DEVICES: [u32; 3] = [0, 1, 0];
/// layout/01 第 365–383 行按行累加：根记录 457 字节，其后到 512 槽末是补齐（第 377 行）。
const ROOT_RECORD_FIELD_BYTES: usize = 457;
/// layout/01 第 45 行：journal 环从 16 MiB 起、长 768 MiB、记录 4096；第 319 行：记录 n 落 `(计数器 − 1) × 4096`。
const JOURNAL_RING_START_BYTES: u64 = 16 * 1_048_576;
const JOURNAL_RING_BYTES: u64 = 768 * 1_048_576;
const JOURNAL_RECORD_BYTES: usize = 4096;
/// layout/01 第 357 行：记录头 487；第 361 行：点名项 56。
const JOURNAL_HEADER_BYTES: usize = 487;
const JOURNAL_NAMED_ITEM_BYTES: usize = 56;
/// layout/01 第 46 行：单元区起始槽号 50176；落点粒度 16384（第 34–35 行）。
const UNIT_AREA_START_SLOT: u64 = 50176;
const SLOT_BYTES: u64 = 16384;
/// layout/01 第 174 行与 D18 已定项 9：码 1 / 码 3 单元 32768，码 2 节点 16384。
const NODE_BYTES: usize = 16384;
/// layout/01 第 175–180 行：单元共同前缀（magic 0、格式版本 4、单元类型标签 6、flag 7、声明长度 8、头校验和 10 起 32）。
const UNIT_DECLARED_LENGTH_OFFSET: usize = 8;
const UNIT_HEADER_CHECKSUM_OFFSET: usize = 10;
const UNIT_COMMON_PREFIX_BYTES: usize = 42;
/// D18 已定项 18（登记第二节抄录）：码 2 头 = 86 + 2k，含 29 字节预留位 115 + 2k。
const INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE: usize = 86;
const RESERVED_NONCE_MAC_ALGORITHM_BYTES: usize = 29;
/// D18 已定项 11 码 3 类身份段表（登记第二节第 276–293 行）：明文头 107，含预留位 136。
const PACKED_UNIT_PLAINTEXT_HEADER_BYTES: usize = 107;
const PACKED_UNIT_RECORDS_OFFSET: usize = 136;
/// layout/01 第 189–194 行：码 1 明文头 105，含预留位 134。
const DATA_UNIT_PLAINTEXT_HEADER_BYTES: usize = 105;
const DATA_UNIT_PAYLOAD_OFFSET: usize = 134;
const DATA_UNIT_BYTES: usize = 32768;
/// layout/01 第 214 行：指向码 2 / 码 3 的指针 86，指向码 1 的指针 88；第 216 行：位置条目 14，第一条在 50、第二条在 64。
const NODE_POINTER_BYTES: usize = 86;
const DATA_POINTER_BYTES: usize = 88;
const LOCATION_ENTRY_BYTES: usize = 14;
const POINTER_FIRST_LOCATION_OFFSET: usize = 50;
/// layout/01 第 285–290 行：extent 上段叶条目 113 = key 24 + 标签 1 + 载荷 88；下段叶条目 112 = key 24 + 数据指针 88。
const EXTENT_UPPER_LEAF_ENTRY_BYTES: usize = 113;
const EXTENT_LOWER_LEAF_ENTRY_BYTES: usize = 112;
/// D8 已定项 14（`.claude/kb/decisions/08-核心索引结构.md` 第 402 行）：下段叶罩 144 个单元。
const EXTENT_LOWER_LEAF_DATA_UNITS: u64 = 144;
/// layout/01 第 191 行：单元净荷容量 32768 − 134。
const DATA_UNIT_PAYLOAD_CAPACITY: u64 = 32768 - 134;
/// layout/01 第 260–262 行：inode 树内部节点条目 120 = 分隔 key 8 + 身份引用 26 + 子指针 86。
const INODE_INTERNAL_ENTRY_BYTES: usize = 120;
/// layout/01 第 265–273 行：inode 记录 140。
const INODE_RECORD_BYTES: usize = 140;
/// layout/01 第 297–299 行：分配记录 20 = key 10 + value 10；内部条目 96 = key 10 + 子指针 86；叶罩 812 槽，层级 1 罩 137228 槽。
const ALLOCATION_RECORD_BYTES: usize = 20;
const ALLOCATION_INTERNAL_ENTRY_BYTES: usize = 96;
const ALLOCATION_LEAF_SLOTS: u64 = 812;
const ALLOCATION_LEVEL_ONE_SLOTS: u64 = 812 * 169;
/// layout/01 第 303–305、311 行：记账条目 34 = key 22 + value 8 + seq 4。
const ACCOUNTING_ENTRY_BYTES: usize = 34;
/// layout/01 第 228–236 行：映射条目 55 = key 27 + value 28。
const MAPPING_ENTRY_BYTES: usize = 55;
/// layout/01 第 387 行：树表条目 200。
const TREE_TABLE_ENTRY_BYTES: usize = 200;
/// layout/03 第 10–11 行：`deleted_inodes` 条目 8；第 14、30–36 行：墓碑记录 40；D18 已定项 11 第 302–307 行：实例表记录 88。
const DELETED_INODES_ENTRY_BYTES: usize = 8;
const TOMBSTONE_RECORD_BYTES: usize = 40;
const INSTANCE_TABLE_RECORD_BYTES: usize = 88;

/// layout/01 第 48 行与 layout/03 第 19 行：树 ID。
const EXTENT_TREE: u64 = 65536;
const INODE_TREE: u64 = 65537;
const ALLOCATION_RECORDS_TREE: u64 = 65538;
const ACCOUNTING_TREE: u64 = 65539;
const MAPPING_TREE: u64 = 65540;
const DELETED_INODES_TREE: u64 = 65544;
const TOMBSTONE_TREE: u64 = 65545;
const LINEAGE_TREE: u64 = 65546;
const LOGGED_OPERATIONS_TREE: u64 = 65547;
const TREE_IDENTIFIER_WATERMARK_AFTER_THE_FIRST_FILE: u64 = 65548;

/// layout/01 第 109、175、338、369 行：四种 magic。
const SYSTEM_CONFIGURATION_MAGIC: &[u8; 4] = b"SFSB";
const UNIT_MAGIC: &[u8; 4] = b"SFSU";
const JOURNAL_MAGIC: &[u8; 4] = b"SFSJ";
const ROOT_RECORD_MAGIC: &[u8; 4] = b"SFSR";

/// 跑前登记第六节 L03-2 与第七节 A-content：被删文件 = 两个满净荷加半个。
const DELETED_INODE: u64 = 1;
const EXPECTED_FILE_BYTES: u64 = 2 * DATA_UNIT_PAYLOAD_CAPACITY + DATA_UNIT_PAYLOAD_CAPACITY / 2;
/// layout/03 第 20 行：标签 6 式子里的副本数与 41（⌈2 × 16384 ÷ 815⌉）。
const ACCOUNTING_COPIES: u64 = 2;
const PER_LIVE_OBJECT_TOMBSTONE_BYTES: u64 = 41;

/// layout/01 第 158 行的偏移表：整槽校验和 155（宽 32，CRC32C 4 + 28 零，第 99、118 行）、T_time 433（宽 4，第 149 行）。
const SYSTEM_CONFIGURATION_CHECKSUM_OFFSET: usize = 155;
const SYSTEM_CONFIGURATION_TIME_THRESHOLD_OFFSET: usize = 433;
/// layout/01 第 149 行：T_time「5000（单位毫秒，即 5 秒）」（跑前登记 e172-r2 第七节 A-T_time；不引 `crates/` 的常量）。
const EXPECTED_TIME_THRESHOLD_MILLISECONDS: u64 = 5000;

/// R8.c 比 R8.b 多要求的两样（跑前登记 e172-r2 第六节 6.4，各写成一个能单独改的常量，第九节 M24、M25、M26 的锚点）：
/// 先判取号（6.4 ①在 ②之前）；轮换一支另要系统配置里的实例代号 = 那条 journal 记录 jsn 里的实例代号（layout/01 第 344 行）。
const ATTRIBUTION_JUDGES_ACQUISITION_BEFORE_ROTATION: bool = true;
const ROTATION_ATTRIBUTION_REQUIRES_THE_SAME_INSTANCE: bool = true;

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

fn castagnoli_crc32(bytes: &[u8]) -> u32 {
    let table = castagnoli_crc32_table();
    let mut register = 0xFFFF_FFFFu32;
    for byte in bytes {
        register = table[usize::from(register.to_le_bytes()[0] ^ byte)] ^ (register >> 8);
    }
    register ^ 0xFFFF_FFFF
}

const SHA256_ROUND_CONSTANTS: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

fn sha256_hexadecimal(bytes: &[u8]) -> String {
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

fn read_little_endian(bytes: &[u8], offset: usize, width: usize) -> u64 {
    let mut value = 0u64;
    for (byte_index, byte) in bytes[offset..offset + width].iter().enumerate() {
        value |= u64::from(*byte) << (8 * byte_index);
    }
    value
}

fn read_u8(bytes: &[u8], offset: usize) -> u8 {
    bytes[offset]
}

fn read_u16(bytes: &[u8], offset: usize) -> u64 {
    read_little_endian(bytes, offset, 2)
}

fn read_u32(bytes: &[u8], offset: usize) -> u64 {
    read_little_endian(bytes, offset, 4)
}

fn read_u48(bytes: &[u8], offset: usize) -> u64 {
    read_little_endian(bytes, offset, 6)
}

fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    read_little_endian(bytes, offset, 8)
}

fn hexadecimal_of(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn bytes_of_hexadecimal(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|character_offset| u8::from_str_radix(&text[character_offset..character_offset + 2], 16).ok())
        .collect()
}

fn usize_of(value: u64) -> usize {
    usize::try_from(value).expect("盘上读出的长度与条数装得进 usize")
}

fn u64_of(value: usize) -> u64 {
    u64::try_from(value).expect("usize 装得进 u64")
}

/// 一个字节区间全是 0。
fn is_all_zero(bytes: &[u8]) -> bool {
    bytes.iter().all(|byte| *byte == 0)
}

// ---------------------------------------------------------------------------------------------
// 读导出：只读行的形态，不带任何对字节的解读。
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct RegionWrite {
    step: usize,
    device: u32,
    offset: u64,
    is_force_unit_access: bool,
    bytes: Vec<u8>,
}

#[derive(Clone, Debug)]
enum WindowOperation {
    Write(RegionWrite),
    Barrier { step: usize, device: u32 },
}

impl WindowOperation {
    fn step(&self) -> usize {
        match self {
            WindowOperation::Write(write) => write.step,
            WindowOperation::Barrier { step, .. } => *step,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SegmentLabel {
    Publish { publish_kind: String, txg: u64, rewritten_roles: Vec<String>, data_units_after: u64, deleted_inodes_after: u64 },
    Outside { segment_name: String },
}

#[derive(Clone, Debug)]
struct ExportSegment {
    start_step: usize,
    end_step: usize,
    label: SegmentLabel,
}

impl ExportSegment {
    fn is_deletion_publish(&self) -> bool {
        match &self.label {
            SegmentLabel::Publish { publish_kind, .. } => {
                publish_kind == "unlink_intent" || publish_kind == "unlink_batch" || publish_kind == "mount_continuation_unlink_batch"
            }
            SegmentLabel::Outside { .. } => false,
        }
    }
    fn publish_kind(&self) -> &str {
        match &self.label {
            SegmentLabel::Publish { publish_kind, .. } => publish_kind,
            SegmentLabel::Outside { segment_name } => segment_name,
        }
    }
}

#[derive(Clone, Debug, Default)]
struct HistoryExport {
    name: String,
    configuration: BTreeMap<String, String>,
    before_window_operations: u64,
    operations: Vec<WindowOperation>,
    segments: Vec<ExportSegment>,
    summary: BTreeMap<String, String>,
    boundary_mismatches: Vec<String>,
}

#[derive(Clone, Debug, Default)]
struct ExportFile {
    histories: Vec<HistoryExport>,
    line_count: u64,
    anchored_line_count: u64,
    done_emitted: Option<u64>,
    non_hexadecimal_line_count: u64,
    malformed_lines: Vec<String>,
}

fn fields_of_line(body: &str) -> BTreeMap<String, String> {
    body.split(' ')
        .filter_map(|field| field.split_once('=').map(|(key, value)| (key.to_string(), value.to_string())))
        .collect()
}

fn field_number(fields: &BTreeMap<String, String>, key: &str) -> Option<u64> {
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
fn parse_export(text: &str) -> ExportFile {
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
                let bytes = fields.get("hexadecimal").and_then(|text| bytes_of_hexadecimal(text));
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

// ---------------------------------------------------------------------------------------------
// 覆盖图：每个字节归到哪一类（跑前登记第一节 1.2「第 2 行的登记按位置读」）。
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum ByteClass {
    Unclaimed,
    LayoutThreeCell,
    LayoutOneCell,
    ConstantZeroPadding,
}

#[derive(Clone, Debug)]
struct CoverageMap {
    classes: Vec<ByteClass>,
    double_claimed_bytes: u64,
}

impl CoverageMap {
    fn new(length: usize) -> Self {
        CoverageMap { classes: vec![ByteClass::Unclaimed; length], double_claimed_bytes: 0 }
    }
    fn claim(&mut self, start: usize, end: usize, class: ByteClass) {
        let clamped_end = end.min(self.classes.len());
        for byte_class in &mut self.classes[start.min(clamped_end)..clamped_end] {
            if *byte_class != ByteClass::Unclaimed {
                self.double_claimed_bytes += 1;
            }
            *byte_class = class;
        }
    }
    fn reclassify(&mut self, start: usize, end: usize, class: ByteClass) {
        let clamped_end = end.min(self.classes.len());
        for byte_class in &mut self.classes[start.min(clamped_end)..clamped_end] {
            *byte_class = class;
        }
    }
}

// ---------------------------------------------------------------------------------------------
// 指针与位置条目（layout/01 第 200–216 行）。
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct LocationEntry {
    device: u32,
    slot: u64,
    checksum: u64,
}

fn location_at(bytes: &[u8], offset: usize) -> LocationEntry {
    LocationEntry {
        device: u32::try_from(read_u32(bytes, offset)).expect("4 字节装得进 u32"),
        slot: read_u48(bytes, offset + 4),
        checksum: read_u32(bytes, offset + 10),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct BlockPointer {
    is_all_zero: bool,
    birth_tree: u64,
    birth_txg: u64,
    locations: Vec<LocationEntry>,
    /// 指向码 2 / 码 3：实例代号 4 + 出生序号 4；指向码 1：写序 10（第 212 行）。
    key_tail: Vec<u8>,
}

/// `pointer_bytes` 是 86（节点指针）或 88（数据指针）。全零的指针（根指针置零）不解位置条目。
fn block_pointer_of(pointer_bytes: &[u8]) -> BlockPointer {
    let is_all_zero = is_all_zero(pointer_bytes);
    let locations = if is_all_zero {
        Vec::new()
    } else {
        (0..2).map(|copy| location_at(pointer_bytes, POINTER_FIRST_LOCATION_OFFSET + copy * LOCATION_ENTRY_BYTES)).collect()
    };
    BlockPointer {
        is_all_zero,
        birth_tree: read_u64(pointer_bytes, 34),
        birth_txg: read_u64(pointer_bytes, 42),
        locations,
        key_tail: pointer_bytes[78..].to_vec(),
    }
}

// ---------------------------------------------------------------------------------------------
// 码 2 索引节点与码 3 打包记录单元。
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct IndexNode {
    tree_identifier: u64,
    level: u8,
    key_width: usize,
    key_range_low: Vec<u8>,
    key_range_high: Vec<u8>,
    birth_txg: u64,
    entry_count: usize,
    entry_width: usize,
    entries: Vec<Vec<u8>>,
}

fn index_node_header_bytes(key_width: usize) -> usize {
    INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE + 2 * key_width
}

#[derive(Clone, Debug)]
struct PackedContainer {
    birth_tree: u64,
    packed_record_type: u64,
    container_number: u64,
    container_birth: u64,
    record_count: usize,
    record_width: usize,
    birth_txg: u64,
    records: Vec<Vec<u8>>,
}

/// 共同前缀的头校验和：CRC32C 4 + 28 零，覆盖 `[0, plaintext_header_end)`，自身按零参与（layout/01 第 180 行；D18 已定项 18）。
fn header_checksum_holds(bytes: &[u8], plaintext_header_end: usize) -> bool {
    if bytes.len() < plaintext_header_end {
        return false;
    }
    let mut covered = bytes[..plaintext_header_end].to_vec();
    for byte in &mut covered[UNIT_HEADER_CHECKSUM_OFFSET..UNIT_COMMON_PREFIX_BYTES] {
        *byte = 0;
    }
    castagnoli_crc32(&covered) == u32::try_from(read_u32(bytes, UNIT_HEADER_CHECKSUM_OFFSET)).expect("4 字节")
        && is_all_zero(&bytes[UNIT_HEADER_CHECKSUM_OFFSET + 4..UNIT_COMMON_PREFIX_BYTES])
}

fn crc_matches(bytes: &[u8], stored: u64) -> bool {
    u64::from(castagnoli_crc32(bytes)) == stored
}

// ---------------------------------------------------------------------------------------------
// 系统配置槽、根记录、journal 记录。
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct SystemConfiguration {
    generation: u64,
    journal_tail: u64,
    instance: u64,
    time_threshold_milliseconds: u64,
}

#[derive(Clone, Debug)]
struct RootRecord {
    txg: u64,
    watermark: u64,
    floor: u64,
    tree_table_pointer: BlockPointer,
    instance_table_pointer: BlockPointer,
    mapping_root_pointer: BlockPointer,
    allocation_root_pointer: BlockPointer,
}

#[derive(Clone, Debug)]
struct JournalRecord {
    instance: u64,
    counter: u64,
    txg: u64,
    watermark: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RegionKind {
    SystemConfigurationSlot,
    RootRecordSlot,
    JournalRecord,
    Unit,
    Unrecognized,
}

impl RegionKind {
    fn step_kind_name(self) -> &'static str {
        match self {
            RegionKind::SystemConfigurationSlot => "system_configuration_slot",
            RegionKind::RootRecordSlot => "root_record_fua",
            RegionKind::JournalRecord => "journal_record",
            RegionKind::Unit => "unit_write",
            RegionKind::Unrecognized => "unrecognized",
        }
    }
}

#[derive(Clone, Debug)]
enum DecodedBody {
    SystemConfiguration(SystemConfiguration),
    RootRecord(RootRecord),
    JournalRecord(JournalRecord),
    IndexNode(IndexNode),
    PackedContainer(PackedContainer),
    DataUnit { birth_txg: u64 },
    Unrecognized,
}

#[derive(Clone, Debug)]
struct DecodedRegion {
    step: usize,
    device: u32,
    offset: u64,
    bytes: Vec<u8>,
    region_kind: RegionKind,
    body: DecodedBody,
    structure_failures: Vec<String>,
    coverage: CoverageMap,
    locations_inside: Vec<LocationEntry>,
}

impl DecodedRegion {
    fn slot(&self) -> u64 {
        self.offset / SLOT_BYTES
    }
    fn unit_type_tag(&self) -> u8 {
        match &self.body {
            DecodedBody::IndexNode(_) => 2,
            DecodedBody::PackedContainer(_) => 3,
            DecodedBody::DataUnit { .. } => 1,
            DecodedBody::SystemConfiguration(_)
            | DecodedBody::RootRecord(_)
            | DecodedBody::JournalRecord(_)
            | DecodedBody::Unrecognized => 0,
        }
    }
    /// 单元头里的诞生代号、journal 记录与根记录的 checkpoint_txg（跑前登记第六节 R8.b 的归属键）。
    fn birth_or_checkpoint_txg(&self) -> Option<u64> {
        match &self.body {
            DecodedBody::IndexNode(node) => Some(node.birth_txg),
            DecodedBody::PackedContainer(container) => Some(container.birth_txg),
            DecodedBody::DataUnit { birth_txg } => Some(*birth_txg),
            DecodedBody::JournalRecord(record) => Some(record.txg),
            DecodedBody::RootRecord(record) => Some(record.txg),
            DecodedBody::SystemConfiguration(_) | DecodedBody::Unrecognized => None,
        }
    }
}

fn root_ring_position(offset: u64) -> Option<(u64, u64)> {
    for region in 0..3u64 {
        let region_start = ROOT_RING_BASE_BYTES + region * ROOT_RING_REGION_STRIDE_BYTES;
        if offset >= region_start && offset < region_start + ROOT_RING_SLOTS_PER_REGION * ROOT_RING_SLOT_SPACING_BYTES {
            let inside = offset - region_start;
            if inside % ROOT_RING_SLOT_SPACING_BYTES == 0 {
                return Some((region, inside / ROOT_RING_SLOT_SPACING_BYTES));
            }
        }
    }
    None
}

/// 只按偏移、长度与 magic 判种类，不读导出的 `kind=`（FUA 另由 `is_force_unit_access` 带）。
fn region_kind_of(write: &RegionWrite) -> RegionKind {
    let length = write.bytes.len();
    let magic = write.bytes.get(0..4);
    if (write.offset == 0 || write.offset == u64_of(SYSTEM_CONFIGURATION_SLOT_BYTES))
        && length == SYSTEM_CONFIGURATION_SLOT_BYTES
        && magic == Some(&SYSTEM_CONFIGURATION_MAGIC[..])
    {
        return RegionKind::SystemConfigurationSlot;
    }
    if root_ring_position(write.offset).is_some() && length == ROOT_RECORD_SLOT_BYTES && magic == Some(&ROOT_RECORD_MAGIC[..]) {
        return RegionKind::RootRecordSlot;
    }
    if write.offset >= JOURNAL_RING_START_BYTES
        && write.offset < JOURNAL_RING_START_BYTES + JOURNAL_RING_BYTES
        && (write.offset - JOURNAL_RING_START_BYTES) % u64_of(JOURNAL_RECORD_BYTES) == 0
        && length == JOURNAL_RECORD_BYTES
        && magic == Some(&JOURNAL_MAGIC[..])
    {
        return RegionKind::JournalRecord;
    }
    if write.offset >= UNIT_AREA_START_SLOT * SLOT_BYTES && write.offset % SLOT_BYTES == 0 && magic == Some(&UNIT_MAGIC[..]) {
        return RegionKind::Unit;
    }
    RegionKind::Unrecognized
}

/// 整块自证校验和：CRC32C 4 + 28 零，覆盖整块、自身按零参与（系统配置 layout/01 第 99 行、根记录第 377 行、journal 第 347 行）。
fn whole_slot_checksum_holds(bytes: &[u8], checksum_offset: usize) -> bool {
    let mut covered = bytes.to_vec();
    for byte in &mut covered[checksum_offset..checksum_offset + 32] {
        *byte = 0;
    }
    crc_matches(&covered, read_u32(bytes, checksum_offset)) && is_all_zero(&bytes[checksum_offset + 4..checksum_offset + 32])
}

fn decode_system_configuration(bytes: &[u8], coverage: &mut CoverageMap, failures: &mut Vec<String>) -> SystemConfiguration {
    // layout/01 第 158 行：槽世代号 147、整槽校验和 155、本盘设备号 139、T_time 433、journal tail 469、实例代号 477、回退下界 F 481。
    if !whole_slot_checksum_holds(bytes, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET) {
        failures.push("系统配置整槽校验和不对".to_string());
    }
    coverage.claim(0, SYSTEM_CONFIGURATION_FIELD_BYTES, ByteClass::LayoutOneCell);
    coverage.claim(SYSTEM_CONFIGURATION_FIELD_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES, ByteClass::ConstantZeroPadding);
    SystemConfiguration {
        generation: read_u64(bytes, 147),
        journal_tail: read_u64(bytes, 469),
        instance: read_u32(bytes, 477),
        time_threshold_milliseconds: read_u32(bytes, SYSTEM_CONFIGURATION_TIME_THRESHOLD_OFFSET),
    }
}

fn decode_root_record(bytes: &[u8], coverage: &mut CoverageMap, failures: &mut Vec<String>, locations: &mut Vec<LocationEntry>) -> RootRecord {
    // layout/01 第 365–383 行按行累加：flags 20、实例代号 24、checkpoint_txg 28、树表单元指针 36、树 ID 水位 122、回退下界 F 130、
    // 自证校验和 138、实例表单元指针 170、中央映射树根指针 256、分配记录树根指针 342（layout/02 第 15 行「偏移 342」同值）。
    if !whole_slot_checksum_holds(bytes, 138) {
        failures.push("根记录自证校验和不对".to_string());
    }
    coverage.claim(0, ROOT_RECORD_FIELD_BYTES, ByteClass::LayoutOneCell);
    coverage.claim(ROOT_RECORD_FIELD_BYTES, ROOT_RECORD_SLOT_BYTES, ByteClass::ConstantZeroPadding);
    let pointer = |offset: usize| block_pointer_of(&bytes[offset..offset + NODE_POINTER_BYTES]);
    let record = RootRecord {
        txg: read_u64(bytes, 28),
        watermark: read_u64(bytes, 122),
        floor: read_u64(bytes, 130),
        tree_table_pointer: pointer(36),
        instance_table_pointer: pointer(170),
        mapping_root_pointer: pointer(256),
        allocation_root_pointer: pointer(342),
    };
    for block_pointer in [&record.tree_table_pointer, &record.instance_table_pointer, &record.mapping_root_pointer, &record.allocation_root_pointer] {
        locations.extend(block_pointer.locations.iter().copied());
    }
    record
}

fn decode_journal_record(bytes: &[u8], coverage: &mut CoverageMap, failures: &mut Vec<String>, locations: &mut Vec<LocationEntry>) -> JournalRecord {
    // layout/01 第 359 行：类型 4、记录长度 8、点名项数 12、jsn 16（实例代号 16、计数器 20）、checkpoint_txg 26、头校验和 46、
    // 载荷校验和 95、新根段 99（树表 99、映射根 185、水位 271、F 279、实例表 287、分配记录树根 373、卸载记号 459）；点名项第 361 行。
    if !whole_slot_checksum_holds(bytes, 46) {
        failures.push("journal 记录头校验和不对".to_string());
    }
    let item_count = usize_of(read_u32(bytes, 12));
    let items_end = JOURNAL_HEADER_BYTES + item_count * JOURNAL_NAMED_ITEM_BYTES;
    if read_u32(bytes, 8) != u64_of(JOURNAL_RECORD_BYTES) || items_end > JOURNAL_RECORD_BYTES {
        failures.push(format!("journal 记录长度 {} 或点名项数 {item_count} 越界", read_u32(bytes, 8)));
        return JournalRecord { instance: 0, counter: 0, txg: 0, watermark: 0 };
    }
    if !crc_matches(&bytes[JOURNAL_HEADER_BYTES..items_end], read_u32(bytes, 95)) {
        failures.push("journal 载荷校验和不对".to_string());
    }
    coverage.claim(0, JOURNAL_HEADER_BYTES, ByteClass::LayoutOneCell);
    coverage.claim(JOURNAL_HEADER_BYTES, items_end, ByteClass::LayoutOneCell);
    coverage.claim(items_end, JOURNAL_RECORD_BYTES, ByteClass::ConstantZeroPadding);
    for offset in [99usize, 185, 287, 373] {
        locations.extend(block_pointer_of(&bytes[offset..offset + NODE_POINTER_BYTES]).locations);
    }
    for item_index in 0..item_count {
        let item_start = JOURNAL_HEADER_BYTES + item_index * JOURNAL_NAMED_ITEM_BYTES;
        for copy in 0..2 {
            locations.push(location_at(bytes, item_start + copy * LOCATION_ENTRY_BYTES));
        }
    }
    JournalRecord {
        instance: read_u32(bytes, 16),
        counter: read_u48(bytes, 20),
        txg: read_u64(bytes, 26),
        watermark: read_u64(bytes, 271),
    }
}

/// 码 2 条目按树分：认得的条目整条归格（layout/01 或 layout/03），认不得的留 `Unclaimed`（第 2 行的没登记）。
fn claim_index_node_entries(node: &IndexNode, entries_offset: usize, coverage: &mut CoverageMap, locations: &mut Vec<LocationEntry>) {
    for (entry_index, entry) in node.entries.iter().enumerate() {
        let start = entries_offset + entry_index * node.entry_width;
        let end = start + node.entry_width;
        let mut pointer_at = |offset: usize, width: usize| locations.extend(block_pointer_of(&entry[offset..offset + width]).locations);
        match (node.tree_identifier, node.entry_width) {
            (EXTENT_TREE, EXTENT_UPPER_LEAF_ENTRY_BYTES) => {
                coverage.claim(start, end, ByteClass::LayoutOneCell);
                match entry[24] {
                    1 => pointer_at(25, NODE_POINTER_BYTES),
                    2 => pointer_at(25, DATA_POINTER_BYTES),
                    _ => {}
                }
            }
            (EXTENT_TREE, EXTENT_LOWER_LEAF_ENTRY_BYTES) => {
                coverage.claim(start, end, ByteClass::LayoutOneCell);
                pointer_at(24, DATA_POINTER_BYTES);
            }
            (INODE_TREE, INODE_INTERNAL_ENTRY_BYTES) => {
                coverage.claim(start, end, ByteClass::LayoutOneCell);
                pointer_at(34, NODE_POINTER_BYTES);
            }
            (ALLOCATION_RECORDS_TREE, ALLOCATION_RECORD_BYTES) if node.level == 0 => coverage.claim(start, end, ByteClass::LayoutOneCell),
            (ALLOCATION_RECORDS_TREE, ALLOCATION_INTERNAL_ENTRY_BYTES) if node.level >= 1 => {
                coverage.claim(start, end, ByteClass::LayoutOneCell);
                pointer_at(10, NODE_POINTER_BYTES);
            }
            (ACCOUNTING_TREE, ACCOUNTING_ENTRY_BYTES) => {
                coverage.claim(start, end, ByteClass::LayoutOneCell);
                if read_u16(entry, 0) == 6 {
                    coverage.reclassify(start + 22, start + 30, ByteClass::LayoutThreeCell);
                }
            }
            (MAPPING_TREE, MAPPING_ENTRY_BYTES) => {
                coverage.claim(start, end, ByteClass::LayoutOneCell);
                locations.push(location_at(entry, 27));
                locations.push(location_at(entry, 41));
            }
            (0, TREE_TABLE_ENTRY_BYTES) => {
                let tree = read_u64(entry, 0);
                let class = if (DELETED_INODES_TREE..=LOGGED_OPERATIONS_TREE).contains(&tree) { ByteClass::LayoutThreeCell } else { ByteClass::LayoutOneCell };
                coverage.claim(start, end, class);
                pointer_at(14, NODE_POINTER_BYTES);
            }
            (DELETED_INODES_TREE, DELETED_INODES_ENTRY_BYTES) => coverage.claim(start, end, ByteClass::LayoutThreeCell),
            (TOMBSTONE_TREE, INODE_INTERNAL_ENTRY_BYTES) => {
                coverage.claim(start, end, ByteClass::LayoutThreeCell);
                pointer_at(34, NODE_POINTER_BYTES);
            }
            _ => {}
        }
    }
}

fn claim_container_records(container: &PackedContainer, coverage: &mut CoverageMap, locations: &mut Vec<LocationEntry>) {
    for (record_index, record) in container.records.iter().enumerate() {
        let start = PACKED_UNIT_RECORDS_OFFSET + record_index * container.record_width;
        let end = start + container.record_width;
        match (container.packed_record_type, container.record_width) {
            (2, INODE_RECORD_BYTES) => {
                coverage.claim(start, end, ByteClass::LayoutOneCell);
                if read_u64(record, 0) == DELETED_INODE {
                    // layout/03 第 12 行：nlink 在偏移 36。
                    coverage.reclassify(start + 36, start + 40, ByteClass::LayoutThreeCell);
                }
            }
            (1, TOMBSTONE_RECORD_BYTES) => coverage.claim(start, end, ByteClass::LayoutThreeCell),
            (4, INSTANCE_TABLE_RECORD_BYTES) => {
                coverage.claim(start, end, ByteClass::LayoutOneCell);
                if record[0] == 1 && record[1] == 1 {
                    locations.extend(block_pointer_of(&record[2..2 + NODE_POINTER_BYTES]).locations);
                }
            }
            _ => {}
        }
    }
}

fn decode_unit(bytes: &[u8], coverage: &mut CoverageMap, failures: &mut Vec<String>, locations: &mut Vec<LocationEntry>) -> DecodedBody {
    let declared_length = usize_of(read_u16(bytes, UNIT_DECLARED_LENGTH_OFFSET));
    match read_u8(bytes, 6) {
        2 => {
            if bytes.len() != NODE_BYTES {
                failures.push(format!("码 2 单元长 {}", bytes.len()));
                return DecodedBody::Unrecognized;
            }
            let key_width = usize::from(read_u8(bytes, 51));
            let header_end = index_node_header_bytes(key_width);
            let entries_offset = header_end + RESERVED_NONCE_MAC_ALGORITHM_BYTES;
            let entry_count = usize_of(read_u16(bytes, 82 + 2 * key_width));
            let entry_width = usize_of(read_u16(bytes, 84 + 2 * key_width));
            if !header_checksum_holds(bytes, header_end) {
                failures.push("码 2 头校验和不对".to_string());
            }
            if !crc_matches(&bytes[header_end..], read_u32(bytes, 76 + 2 * key_width)) {
                failures.push("码 2 载荷 CRC 不对".to_string());
            }
            if entry_count * entry_width != declared_length {
                failures.push(format!("码 2 条目数 {entry_count} × 条目宽 {entry_width} ≠ 声明长度 {declared_length}"));
            }
            let entries_end = (entries_offset + entry_count * entry_width).min(NODE_BYTES);
            let node = IndexNode {
                tree_identifier: read_u64(bytes, 42),
                level: read_u8(bytes, 50),
                key_width,
                key_range_low: bytes[52..52 + key_width].to_vec(),
                key_range_high: bytes[52 + key_width..52 + 2 * key_width].to_vec(),
                birth_txg: read_u64(bytes, 52 + 2 * key_width),
                entry_count,
                entry_width,
                entries: (0..entry_count)
                    .filter(|entry_index| entries_offset + (entry_index + 1) * entry_width <= NODE_BYTES)
                    .map(|entry_index| bytes[entries_offset + entry_index * entry_width..entries_offset + (entry_index + 1) * entry_width].to_vec())
                    .collect(),
            };
            coverage.claim(0, header_end, ByteClass::LayoutOneCell);
            coverage.claim(header_end, entries_offset, ByteClass::ConstantZeroPadding);
            claim_index_node_entries(&node, entries_offset, coverage, locations);
            coverage.claim(entries_end, NODE_BYTES, ByteClass::ConstantZeroPadding);
            DecodedBody::IndexNode(node)
        }
        3 => {
            if bytes.len() != DATA_UNIT_BYTES {
                failures.push(format!("码 3 单元长 {}", bytes.len()));
                return DecodedBody::Unrecognized;
            }
            if !header_checksum_holds(bytes, PACKED_UNIT_PLAINTEXT_HEADER_BYTES) {
                failures.push("码 3 头校验和不对".to_string());
            }
            if !crc_matches(&bytes[PACKED_UNIT_PLAINTEXT_HEADER_BYTES..], read_u32(bytes, 89)) {
                failures.push("码 3 载荷校验和不对".to_string());
            }
            if read_u8(bytes, 42) != 3 {
                failures.push("码 3 偏移 42 的标签副本不是 3".to_string());
            }
            let record_count = usize_of(read_u16(bytes, 69));
            let record_width = usize_of(read_u16(bytes, 71));
            if record_count * record_width > declared_length {
                failures.push(format!("码 3 记录数 {record_count} × 记录宽 {record_width} > 声明长度 {declared_length}"));
            }
            let records_end = (PACKED_UNIT_RECORDS_OFFSET + record_count * record_width).min(DATA_UNIT_BYTES);
            let container = PackedContainer {
                birth_tree: read_u64(bytes, 43),
                packed_record_type: read_u16(bytes, 51),
                container_number: read_u64(bytes, 53),
                container_birth: read_u64(bytes, 61),
                record_count,
                record_width,
                birth_txg: read_u64(bytes, 73),
                records: (0..record_count)
                    .filter(|record_index| PACKED_UNIT_RECORDS_OFFSET + (record_index + 1) * record_width <= DATA_UNIT_BYTES)
                    .map(|record_index| bytes[PACKED_UNIT_RECORDS_OFFSET + record_index * record_width..PACKED_UNIT_RECORDS_OFFSET + (record_index + 1) * record_width].to_vec())
                    .collect(),
            };
            coverage.claim(0, PACKED_UNIT_PLAINTEXT_HEADER_BYTES, ByteClass::LayoutOneCell);
            coverage.claim(PACKED_UNIT_PLAINTEXT_HEADER_BYTES, PACKED_UNIT_RECORDS_OFFSET, ByteClass::ConstantZeroPadding);
            claim_container_records(&container, coverage, locations);
            coverage.claim(records_end, DATA_UNIT_BYTES, ByteClass::ConstantZeroPadding);
            DecodedBody::PackedContainer(container)
        }
        1 => {
            if bytes.len() != DATA_UNIT_BYTES {
                failures.push(format!("码 1 单元长 {}", bytes.len()));
                return DecodedBody::Unrecognized;
            }
            if !header_checksum_holds(bytes, DATA_UNIT_PLAINTEXT_HEADER_BYTES) {
                failures.push("码 1 头校验和不对".to_string());
            }
            if !crc_matches(&bytes[DATA_UNIT_PLAINTEXT_HEADER_BYTES..], read_u32(bytes, 101)) {
                failures.push("码 1 载荷 CRC 不对".to_string());
            }
            coverage.claim(0, DATA_UNIT_PLAINTEXT_HEADER_BYTES, ByteClass::LayoutOneCell);
            coverage.claim(DATA_UNIT_PLAINTEXT_HEADER_BYTES, DATA_UNIT_PAYLOAD_OFFSET, ByteClass::ConstantZeroPadding);
            let payload_end = (DATA_UNIT_PAYLOAD_OFFSET + declared_length).min(DATA_UNIT_BYTES);
            coverage.claim(DATA_UNIT_PAYLOAD_OFFSET, payload_end, ByteClass::LayoutOneCell);
            coverage.claim(payload_end, DATA_UNIT_BYTES, ByteClass::ConstantZeroPadding);
            DecodedBody::DataUnit { birth_txg: read_u64(bytes, 75) }
        }
        other => {
            failures.push(format!("单元类型标签 {other} 不认得"));
            DecodedBody::Unrecognized
        }
    }
}

fn decode_region(write: &RegionWrite) -> DecodedRegion {
    let region_kind = region_kind_of(write);
    let mut coverage = CoverageMap::new(write.bytes.len());
    let mut failures = Vec::new();
    let mut locations = Vec::new();
    let body = match region_kind {
        RegionKind::SystemConfigurationSlot => DecodedBody::SystemConfiguration(decode_system_configuration(&write.bytes, &mut coverage, &mut failures)),
        RegionKind::RootRecordSlot => DecodedBody::RootRecord(decode_root_record(&write.bytes, &mut coverage, &mut failures, &mut locations)),
        RegionKind::JournalRecord => DecodedBody::JournalRecord(decode_journal_record(&write.bytes, &mut coverage, &mut failures, &mut locations)),
        RegionKind::Unit => decode_unit(&write.bytes, &mut coverage, &mut failures, &mut locations),
        RegionKind::Unrecognized => {
            failures.push(format!("偏移 {} 长 {} 的写认不出种类（magic 或落点不对）", write.offset, write.bytes.len()));
            DecodedBody::Unrecognized
        }
    };
    if region_kind == RegionKind::RootRecordSlot && !write.is_force_unit_access {
        failures.push("根槽写不是 FUA".to_string());
    }
    DecodedRegion {
        step: write.step,
        device: write.device,
        offset: write.offset,
        bytes: write.bytes.clone(),
        region_kind,
        body,
        structure_failures: failures,
        coverage,
        locations_inside: locations,
    }
}

// ---------------------------------------------------------------------------------------------
// 一次发布（或一段不属于任何发布的写）的解码结果：单元按落点合成角色，角色身份照跑前登记第六节 R4.a。
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct RoleIdentity {
    unit_type_tag: u8,
    tree_identifier: u64,
    level: u8,
    packed_record_type: u64,
    container_number: u64,
    allocation_device: Option<u64>,
    allocation_position: Option<u64>,
    is_tree_root: bool,
}

#[derive(Clone, Debug)]
struct UnitRole {
    identity: RoleIdentity,
    role_name: String,
    slot: u64,
    /// 设备 0 那一份（两份逐字节相同由结构检查判）。
    copy_on_first_device: DecodedRegion,
}

/// 码 2 单元的「树」：码 2 看偏移 42，码 3 看出生树（偏移 43）。
fn tree_of_region(region: &DecodedRegion) -> u64 {
    match &region.body {
        DecodedBody::IndexNode(node) => node.tree_identifier,
        DecodedBody::PackedContainer(container) => container.birth_tree,
        DecodedBody::DataUnit { .. } => EXTENT_TREE,
        DecodedBody::SystemConfiguration(_) | DecodedBody::RootRecord(_) | DecodedBody::JournalRecord(_) | DecodedBody::Unrecognized => u64::MAX,
    }
}

/// 分配记录树 key = (设备 4, 槽号 6)（layout/01 第 297 行）：key 区间起点的设备段与按位置序号（第 298 行）。
fn allocation_position_of(node: &IndexNode) -> (u64, u64) {
    let device = read_u32(&node.key_range_low, 0);
    let start_slot = read_u48(&node.key_range_low, 4);
    let position = match node.level {
        0 => start_slot / ALLOCATION_LEAF_SLOTS,
        _ => start_slot / ALLOCATION_LEVEL_ONE_SLOTS,
    };
    (device, position)
}

fn role_identity_of(region: &DecodedRegion, is_tree_root: bool) -> RoleIdentity {
    let (level, packed_record_type, container_number, allocation_device, allocation_position) = match &region.body {
        DecodedBody::IndexNode(node) if node.tree_identifier == ALLOCATION_RECORDS_TREE && !is_tree_root => {
            let (device, position) = allocation_position_of(node);
            (node.level, 0, 0, Some(device), Some(position))
        }
        DecodedBody::IndexNode(node) => (node.level, 0, 0, None, None),
        DecodedBody::PackedContainer(container) => (0, container.packed_record_type, container.container_number, None, None),
        DecodedBody::DataUnit { .. } | DecodedBody::SystemConfiguration(_) | DecodedBody::RootRecord(_) | DecodedBody::JournalRecord(_) | DecodedBody::Unrecognized => {
            (0, 0, 0, None, None)
        }
    };
    RoleIdentity {
        unit_type_tag: region.unit_type_tag(),
        tree_identifier: tree_of_region(region),
        level,
        packed_record_type,
        container_number,
        allocation_device,
        allocation_position,
        is_tree_root,
    }
}

/// 解码器独立认出的角色映回 `TransactionUnit::tag` 的写法（layout/03 第 47 行列的角色名）；只用来对拍（S3）与 R3.c 报名字。
fn role_name_of(identity: &RoleIdentity, region: &DecodedRegion, inode_container_numbers: &[u64], tombstone_container_numbers: &[u64]) -> String {
    let rank_among = |numbers: &[u64], number: u64| numbers.iter().position(|candidate| *candidate == number).unwrap_or(usize::MAX);
    match (identity.unit_type_tag, identity.tree_identifier, identity.is_tree_root) {
        (1, _, _) => "t1".to_string(),
        (2, EXTENT_TREE, true) => "t2".to_string(),
        (2, EXTENT_TREE, false) => match &region.body {
            DecodedBody::IndexNode(node) => {
                let start_offset = read_u64(&node.key_range_low, 16);
                format!("t2下@{}.{}", node.level, start_offset / (DATA_UNIT_PAYLOAD_CAPACITY * EXTENT_LOWER_LEAF_DATA_UNITS))
            }
            _ => "t2下@?".to_string(),
        },
        (3, INODE_TREE, _) => match rank_among(inode_container_numbers, identity.container_number) {
            0 => "t3".to_string(),
            rank => format!("t3+{rank}"),
        },
        (2, INODE_TREE, _) => "t4".to_string(),
        (2, ALLOCATION_RECORDS_TREE, true) => "t5".to_string(),
        (2, ALLOCATION_RECORDS_TREE, false) => format!(
            "t5@{}.{}.{}",
            identity.level,
            identity.allocation_device.unwrap_or(u64::MAX),
            identity.allocation_position.unwrap_or(u64::MAX)
        ),
        (2, ACCOUNTING_TREE, _) => "t6".to_string(),
        (2, MAPPING_TREE, _) => "t7".to_string(),
        (2, 0, _) => "t8".to_string(),
        (2, DELETED_INODES_TREE, _) => "td".to_string(),
        (3, TOMBSTONE_TREE, _) => format!("tt@{}", rank_among(tombstone_container_numbers, identity.container_number)),
        (2, TOMBSTONE_TREE, _) => "tt".to_string(),
        (3, 0, _) => "ti".to_string(),
        (tag, tree, _) => format!("unknown@{tag}.{tree}.{}", identity.level),
    }
}

/// 段里一步的视图（切段用）。
#[derive(Clone, Debug)]
struct OperationView {
    is_barrier: bool,
    region_kind: RegionKind,
    is_force_unit_access: bool,
}

#[derive(Clone, Debug)]
struct SegmentAnalysis {
    start_step: usize,
    end_step: usize,
    label: SegmentLabel,
    is_deletion_publish: bool,
    operations: Vec<OperationView>,
    regions: Vec<DecodedRegion>,
    units: Vec<UnitRole>,
    root_records: Vec<(u32, u64, RootRecord)>,
    journal_records: Vec<(u32, u64, JournalRecord)>,
    system_configurations: Vec<(usize, u32, u64, SystemConfiguration)>,
    structure_failures: Vec<String>,
    unreferenced_units: Vec<String>,
}

impl SegmentAnalysis {
    fn decoded_txg(&self) -> Option<u64> {
        self.root_records.first().map(|(_, _, record)| record.txg)
    }
    fn role_names(&self) -> Vec<String> {
        self.units.iter().map(|unit| unit.role_name.clone()).collect()
    }
    fn identities(&self) -> BTreeSet<RoleIdentity> {
        self.units.iter().map(|unit| unit.identity.clone()).collect()
    }
}

/// 一段历史跨段携带的「现行那一版」：每个角色最近一次写出的那一份（设备 0），没写就照抄上一版（layout/03 第 13–17 行「照抄」）。
#[derive(Clone, Debug, Default)]
struct VersionState {
    latest_by_identity: BTreeMap<RoleIdentity, DecodedRegion>,
    latest_root_record: Option<RootRecord>,
}

impl VersionState {
    fn latest_node(&self, tree: u64, want_root: bool) -> Option<&IndexNode> {
        self.latest_by_identity
            .iter()
            .filter(|(identity, _)| identity.unit_type_tag == 2 && identity.tree_identifier == tree && identity.is_tree_root == want_root)
            .find_map(|(_, region)| match &region.body {
                DecodedBody::IndexNode(node) => Some(node),
                _ => None,
            })
    }
    fn latest_containers(&self, tree: u64, packed_record_type: u64) -> Vec<&PackedContainer> {
        self.latest_by_identity
            .iter()
            .filter(|(identity, _)| identity.unit_type_tag == 3 && identity.tree_identifier == tree && identity.packed_record_type == packed_record_type)
            .filter_map(|(_, region)| match &region.body {
                DecodedBody::PackedContainer(container) => Some(container),
                _ => None,
            })
            .collect()
    }
}

/// 身份引用里的容器号（inode 树与墓碑树内部条目：分隔 key 8 + 出生树 8 + 打包记录类型 2 + 容器号 8，layout/01 第 261 行）。
fn container_numbers_of_root(node: Option<&IndexNode>) -> Vec<u64> {
    node.map(|node| node.entries.iter().map(|entry| read_u64(entry, 18)).collect()).unwrap_or_default()
}

/// 树表条目：树 ID 0、条目长度 8、种类 10、flags 12、根指针 14、`previous_snapshot_txg` 100、诞生 txg 108、头 ID 116、预留 124（layout/01 第 387 行按行累加）。
#[derive(Clone, Debug)]
struct TreeTableEntry {
    tree: u64,
    kind_code: u64,
    root_pointer: BlockPointer,
    head_identifier: u64,
}

fn tree_table_entries_of(node: &IndexNode) -> Vec<TreeTableEntry> {
    node.entries
        .iter()
        .map(|entry| TreeTableEntry {
            tree: read_u64(entry, 0),
            kind_code: read_u16(entry, 10),
            root_pointer: block_pointer_of(&entry[14..14 + NODE_POINTER_BYTES]),
            head_identifier: read_u64(entry, 116),
        })
        .collect()
}

fn analyze_segment(segment: &ExportSegment, operations: &[WindowOperation], state: &VersionState) -> SegmentAnalysis {
    let mut views = Vec::new();
    let mut regions = Vec::new();
    for operation in operations.iter().filter(|operation| operation.step() >= segment.start_step && operation.step() < segment.end_step) {
        match operation {
            WindowOperation::Barrier { .. } => views.push(OperationView {
                is_barrier: true,
                region_kind: RegionKind::Unrecognized,
                is_force_unit_access: false,
            }),
            WindowOperation::Write(write) => {
                let region = decode_region(write);
                views.push(OperationView {
                    is_barrier: false,
                    region_kind: region.region_kind,
                    is_force_unit_access: write.is_force_unit_access,
                });
                regions.push(region);
            }
        }
    }
    let mut structure_failures: Vec<String> = regions
        .iter()
        .flat_map(|region| region.structure_failures.iter().map(move |failure| format!("step {} 设备 {}：{failure}", region.step, region.device)))
        .collect();
    // 树根：这一段写的根记录与树表单元指着的落点（树表没写就取现行那一版）。
    let mut root_locations: BTreeSet<(u32, u64)> = BTreeSet::new();
    let mut root_records = Vec::new();
    let mut journal_records = Vec::new();
    let mut system_configurations = Vec::new();
    for region in &regions {
        match &region.body {
            DecodedBody::RootRecord(record) => {
                for block_pointer in [&record.tree_table_pointer, &record.instance_table_pointer, &record.mapping_root_pointer, &record.allocation_root_pointer] {
                    root_locations.extend(block_pointer.locations.iter().map(|location| (location.device, location.slot)));
                }
                root_records.push((region.device, region.offset, record.clone()));
            }
            DecodedBody::JournalRecord(record) => journal_records.push((region.device, region.offset, record.clone())),
            DecodedBody::SystemConfiguration(configuration) => system_configurations.push((region.step, region.device, region.offset, configuration.clone())),
            DecodedBody::IndexNode(_) | DecodedBody::PackedContainer(_) | DecodedBody::DataUnit { .. } | DecodedBody::Unrecognized => {}
        }
    }
    let written_tree_table = regions.iter().find_map(|region| match &region.body {
        DecodedBody::IndexNode(node) if node.tree_identifier == 0 && region.region_kind == RegionKind::Unit => Some(node.clone()),
        _ => None,
    });
    let tree_table = written_tree_table.or_else(|| state.latest_node(0, true).cloned());
    if let Some(node) = &tree_table {
        for entry in tree_table_entries_of(node) {
            root_locations.extend(entry.root_pointer.locations.iter().map(|location| (location.device, location.slot)));
        }
    }
    // 单元按落点成组：每个落点两块盘各一份、逐字节相同（layout/01 第 393 行）。
    let mut by_slot: BTreeMap<u64, Vec<&DecodedRegion>> = BTreeMap::new();
    let mut slot_order = Vec::new();
    for region in regions.iter().filter(|region| region.region_kind == RegionKind::Unit) {
        if !by_slot.contains_key(&region.slot()) {
            slot_order.push(region.slot());
        }
        by_slot.entry(region.slot()).or_default().push(region);
    }
    let inode_root_in_segment = regions.iter().find_map(|region| match &region.body {
        DecodedBody::IndexNode(node) if node.tree_identifier == INODE_TREE => Some(node),
        _ => None,
    });
    let tombstone_root_in_segment = regions.iter().find_map(|region| match &region.body {
        DecodedBody::IndexNode(node) if node.tree_identifier == TOMBSTONE_TREE => Some(node),
        _ => None,
    });
    let inode_container_numbers = container_numbers_of_root(inode_root_in_segment.or_else(|| state.latest_node(INODE_TREE, true)));
    let tombstone_container_numbers = container_numbers_of_root(tombstone_root_in_segment.or_else(|| state.latest_node(TOMBSTONE_TREE, true)));
    let segment_locations: Vec<LocationEntry> = regions.iter().flat_map(|region| region.locations_inside.iter().copied()).collect();
    let mut units = Vec::new();
    let mut unreferenced_units = Vec::new();
    for slot in slot_order {
        let copies = &by_slot[&slot];
        let devices: BTreeSet<u32> = copies.iter().map(|copy| copy.device).collect();
        if copies.len() != 2 || devices != BTreeSet::from([0, 1]) || copies[0].bytes != copies[1].bytes {
            structure_failures.push(format!("槽 {slot} 的单元不是两块盘各一份逐字节相同（{} 份）", copies.len()));
        }
        for copy in copies {
            let referencing: Vec<&LocationEntry> = segment_locations.iter().filter(|location| location.device == copy.device && location.slot == slot).collect();
            if referencing.is_empty() {
                unreferenced_units.push(format!("step {} 设备 {} 槽 {slot}", copy.step, copy.device));
            }
            let expected_checksum = u64::from(castagnoli_crc32(&copy.bytes));
            for location in referencing {
                if location.checksum != expected_checksum {
                    structure_failures.push(format!("step {} 设备 {} 槽 {slot}：父指针位置条目里的整单元校验和 {} ≠ {expected_checksum}", copy.step, copy.device, location.checksum));
                }
            }
        }
        let Some(first_copy) = copies.iter().find(|copy| copy.device == 0).or(copies.first()) else { continue };
        let is_tree_root = root_locations.contains(&(first_copy.device, slot));
        let identity = role_identity_of(first_copy, is_tree_root);
        let role_name = role_name_of(&identity, first_copy, &inode_container_numbers, &tombstone_container_numbers);
        units.push(UnitRole { identity, role_name, slot, copy_on_first_device: (*first_copy).clone() });
    }
    SegmentAnalysis {
        start_step: segment.start_step,
        end_step: segment.end_step,
        label: segment.label.clone(),
        is_deletion_publish: segment.is_deletion_publish(),
        operations: views,
        regions,
        units,
        root_records,
        journal_records,
        system_configurations,
        structure_failures,
        unreferenced_units,
    }
}

impl VersionState {
    fn absorb(&mut self, analysis: &SegmentAnalysis) {
        for unit in &analysis.units {
            self.latest_by_identity.insert(unit.identity.clone(), unit.copy_on_first_device.clone());
        }
        if let Some((_, _, record)) = analysis.root_records.first() {
            self.latest_root_record = Some(record.clone());
        }
    }
}

// ---------------------------------------------------------------------------------------------
// 切段（D13 已定项 4：屏障与 FUA 写切段；屏障按设备记、每块盘一步）。
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
struct SegmentPiece {
    kinds: Vec<(String, usize)>,
}

impl SegmentPiece {
    fn write_count(&self) -> usize {
        self.kinds.iter().filter(|(kind, _)| kind != "barrier").map(|(_, count)| count).sum()
    }
    fn add(&mut self, kind: &str) {
        match self.kinds.iter_mut().find(|(existing, _)| existing == kind) {
            Some((_, count)) => *count += 1,
            None => self.kinds.push((kind.to_string(), 1)),
        }
    }
    fn kind_text(&self) -> String {
        let parts: Vec<String> = self
            .kinds
            .iter()
            .map(|(kind, count)| if *count == 1 { kind.clone() } else { format!("{kind}×{count}") })
            .collect();
        format!("[{}]", parts.join(","))
    }
}

fn cut_into_pieces(operations: &[OperationView]) -> Vec<SegmentPiece> {
    let mut pieces = Vec::new();
    let mut current = SegmentPiece { kinds: Vec::new() };
    let mut current_is_closed_by_a_barrier = false;
    for operation in operations {
        if operation.is_barrier {
            current.add("barrier");
            current_is_closed_by_a_barrier = true;
        } else if operation.is_force_unit_access {
            if !current.kinds.is_empty() {
                pieces.push(std::mem::replace(&mut current, SegmentPiece { kinds: Vec::new() }));
            }
            let mut force_unit_access_piece = SegmentPiece { kinds: Vec::new() };
            force_unit_access_piece.add(operation.region_kind.step_kind_name());
            pieces.push(force_unit_access_piece);
            current_is_closed_by_a_barrier = false;
        } else {
            if current_is_closed_by_a_barrier {
                pieces.push(std::mem::replace(&mut current, SegmentPiece { kinds: Vec::new() }));
                current_is_closed_by_a_barrier = false;
            }
            current.add(operation.region_kind.step_kind_name());
        }
    }
    if !current.kinds.is_empty() {
        pieces.push(current);
    }
    pieces
}

fn segment_count_text(pieces: &[SegmentPiece]) -> String {
    pieces.iter().map(|piece| piece.write_count().to_string()).collect::<Vec<_>>().join("+")
}

fn segment_kind_text(pieces: &[SegmentPiece]) -> String {
    pieces.iter().map(SegmentPiece::kind_text).collect::<Vec<_>>().join("|")
}

/// 跑前登记第一节 1.2：普通发布的种类串把单元写数换成 2 × 单元数（layout/01 第 411 行）。
fn expected_kind_text(units_per_device: usize) -> String {
    format!(
        "[unit_write×{},barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]",
        2 * units_per_device
    )
}

fn expected_count_text(units_per_device: usize) -> String {
    format!("{}+2+1+2", 2 * units_per_device)
}

/// A-根槽：第 n 次发布写区域 `n mod 3` 的槽 `(n div 3) mod 8`（layout/01 第 163 行）。
fn expected_root_slot(txg: u64) -> (u32, u64) {
    let region = txg % 3;
    let slot = (txg / 3) % ROOT_RING_SLOTS_PER_REGION;
    (
        ROOT_RING_REGION_DEVICES[usize_of(region)],
        ROOT_RING_BASE_BYTES + region * ROOT_RING_REGION_STRIDE_BYTES + slot * ROOT_RING_SLOT_SPACING_BYTES,
    )
}

/// A-journal：记录 n 落 `(计数器 − 1) × 4096`（layout/01 第 319 行），跑前登记取「计数器 = txg」。
fn expected_journal_offset_assuming_counter_equals_txg(txg: u64) -> u64 {
    JOURNAL_RING_START_BYTES + (txg - 1) * u64_of(JOURNAL_RECORD_BYTES)
}

/// A-系统配置：槽世代号 mkfs 1、取号 2、暖机 3 / 4、新池新建文件 5、顺序写 6 ⇒ txg t 的那次发布写世代号 t + 2，槽 = 世代号 mod 2（layout/01 第 95、117 行）。
fn expected_system_configuration_for_txg_in_the_first_instance(txg: u64) -> (u64, u64) {
    let generation = txg + 2;
    (generation, (generation % 2) * u64_of(SYSTEM_CONFIGURATION_SLOT_BYTES))
}

/// L03-7 的两支（跑前登记 e172-r2 第一节 1.2）：按「上一版里装着被删记录的那片 inode 叶有几条记录」选，
/// 从窗口里上一次写出的那片叶解出来（条件是条款的输入，不是答案）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DeletedRecordLeaf {
    /// 只有被删那一条：那一片删空整片不落盘（layout/03 第 13 行）。
    OnlyTheDeletedRecord,
    /// 还有别的记录：那一片照写（layout/03 第 55 行「叶里还有别的记录时那一片照写」）。
    BesideOtherRecords,
    /// 上一版里找不到装着被删记录的叶（窗口里没写过）。
    NotFound,
}

impl DeletedRecordLeaf {
    fn name(self) -> &'static str {
        match self {
            DeletedRecordLeaf::OnlyTheDeletedRecord => "only_record",
            DeletedRecordLeaf::BesideOtherRecords => "other_records",
            DeletedRecordLeaf::NotFound => "not_found",
        }
    }
}

/// 上一版（`state`）里装着被删记录的那片 inode 叶：哪一支，与叶里别的记录的 inode 号（升序）。
fn deleted_record_leaf_of_version(state: &VersionState) -> (DeletedRecordLeaf, Vec<u64>) {
    let live_containers = container_numbers_of_root(state.latest_node(INODE_TREE, true));
    let leaf = state
        .latest_containers(INODE_TREE, 2)
        .into_iter()
        .filter(|container| live_containers.contains(&container.container_number))
        .find(|container| container.records.iter().any(|record| read_u64(record, 0) == DELETED_INODE));
    let Some(leaf) = leaf else { return (DeletedRecordLeaf::NotFound, Vec::new()) };
    let mut other_inodes: Vec<u64> = leaf.records.iter().map(|record| read_u64(record, 0)).filter(|inode| *inode != DELETED_INODE).collect();
    other_inodes.sort_unstable();
    let branch = if other_inodes.is_empty() { DeletedRecordLeaf::OnlyTheDeletedRecord } else { DeletedRecordLeaf::BesideOtherRecords };
    (branch, other_inodes)
}

/// layout/03 第 51–55 行：每种删除类发布每盘单元数与角色名（按剩下的数据单元数分）；最后一批在 L03-7 第二支时
/// 照写 inode 叶容器 t3（layout/03 第 55 行「叶里还有别的记录时那一片照写」，跑前登记 e172-r2 第一节 1.2）。
fn registered_roles(publish_kind: &str, data_units_after: u64, deleted_record_leaf: DeletedRecordLeaf) -> Option<Vec<&'static str>> {
    let allocation = ["t5@0.0.61", "t5@0.1.61", "t5@1.0.0", "t5@1.1.0", "t5"];
    let mut roles: Vec<&'static str> = Vec::new();
    match (publish_kind == "unlink_intent", data_units_after) {
        (true, _) => {
            roles.extend(["t3", "t4"]);
            roles.extend(allocation);
            roles.extend(["t6", "td", "tt@0", "tt", "t7", "t8"]);
        }
        (false, 2) => {
            roles.extend(["t2下@0.0", "t2", "t3", "t4"]);
            roles.extend(allocation);
            roles.extend(["t6", "t7", "t8"]);
        }
        (false, 1) => {
            roles.extend(["t2", "t3", "t4"]);
            roles.extend(allocation);
            roles.extend(["t6", "t7", "t8"]);
        }
        (false, 0) => {
            roles.push("t2");
            if deleted_record_leaf == DeletedRecordLeaf::BesideOtherRecords {
                roles.push("t3");
            }
            roles.push("t4");
            roles.extend(allocation);
            roles.extend(["t6", "t7", "t8"]);
        }
        (false, _) => return None,
    }
    Some(roles)
}

// ---------------------------------------------------------------------------------------------
// 一版的派生量：剩下的数据单元数、队列里的号、记账行、n_live、墓碑树内部层数。
// ---------------------------------------------------------------------------------------------

/// 映射条目的类标签是码 1 的条数（layout/01 第 228 行：码 1 一条一个数据单元）。
fn data_units_of_version(state: &VersionState) -> Option<u64> {
    state.latest_node(MAPPING_TREE, true).map(|node| u64_of(node.entries.iter().filter(|entry| entry[0] == 1).count()))
}

fn tree_table_of_version(state: &VersionState) -> Vec<TreeTableEntry> {
    state.latest_node(0, true).map(tree_table_entries_of).unwrap_or_default()
}

fn tree_table_root_is_zero(state: &VersionState, tree: u64) -> Option<bool> {
    tree_table_of_version(state).iter().find(|entry| entry.tree == tree).map(|entry| entry.root_pointer.is_all_zero)
}

/// 队列里的号：树表里 `deleted_inodes` 根指针置零时为空，否则取现行那一版 td 的条目（layout/03 第 10–11 行）。
fn deleted_inodes_of_version(state: &VersionState) -> Vec<u64> {
    match tree_table_root_is_zero(state, DELETED_INODES_TREE) {
        Some(false) => state
            .latest_node(DELETED_INODES_TREE, true)
            .map(|node| node.entries.iter().map(|entry| read_u64(entry, 0)).collect())
            .unwrap_or_default(),
        Some(true) | None => Vec::new(),
    }
}

fn accounting_value(state: &VersionState, statistic_tag: u64) -> Option<u64> {
    state.latest_node(ACCOUNTING_TREE, true).and_then(|node| {
        node.entries.iter().find(|entry| read_u16(entry, 0) == statistic_tag).map(|entry| read_u64(entry, 22))
    })
}

/// 现行那一版 inode 树里活着的记录：inode 树根条目指着的容器里的记录（根条目数 0 ⇒ 没有记录）。
fn inode_records_of_version(state: &VersionState) -> Vec<Vec<u8>> {
    let live_containers = container_numbers_of_root(state.latest_node(INODE_TREE, true));
    state
        .latest_containers(INODE_TREE, 2)
        .into_iter()
        .filter(|container| live_containers.contains(&container.container_number))
        .flat_map(|container| container.records.iter().cloned())
        .collect()
}

fn record_of_inode(records: &[Vec<u8>], inode: u64) -> Option<Vec<u8>> {
    records.iter().find(|record| read_u64(record, 0) == inode).cloned()
}

/// layout/03 第 20 行：n_live = inode 树里 nlink ≥ 1 的记录数；H = 墓碑树内部层数（根直接管容器时层级 1 ⇒ H = 0）。
fn live_inode_count(state: &VersionState) -> u64 {
    u64_of(inode_records_of_version(state).iter().filter(|record| read_u32(record, 36) >= 1).count())
}

fn tombstone_internal_levels(state: &VersionState) -> u64 {
    match tree_table_root_is_zero(state, TOMBSTONE_TREE) {
        Some(false) => state.latest_node(TOMBSTONE_TREE, true).map(|node| u64::from(node.level).saturating_sub(1)).unwrap_or(0),
        Some(true) | None => 0,
    }
}

fn expected_committed_reservation(tombstone_levels: u64, live_inodes: u64) -> u64 {
    ACCOUNTING_COPIES * ((2 + tombstone_levels + 1) * SLOT_BYTES + PER_LIVE_OBJECT_TOMBSTONE_BYTES * live_inodes)
}

// ---------------------------------------------------------------------------------------------
// 判定行。
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct CellRow {
    cell: &'static str,
    field: String,
    history: String,
    txg: u64,
    publish_kind: String,
    expected: String,
    actual: String,
    discriminable: bool,
}

impl CellRow {
    fn is_equal(&self) -> bool {
        self.expected == self.actual
    }
    fn line(&self) -> String {
        format!(
            "name=l03_cell cell={} field={} history={} txg={} publish={} expected={} actual={} equal={} discriminable={}",
            self.cell,
            self.field,
            self.history,
            self.txg,
            self.publish_kind,
            self.expected,
            self.actual,
            self.is_equal(),
            self.discriminable
        )
    }
}

struct CellRowBuilder<'rows> {
    rows: &'rows mut Vec<CellRow>,
    history: String,
    txg: u64,
    publish_kind: String,
    /// L03-7 选到哪一支、第二支只记值的 key 区间（只记、不判的行，跑前登记 e172-r2 第五节 5.8 改6）。
    leaf_branch_records: Vec<String>,
}

impl CellRowBuilder<'_> {
    fn push(&mut self, cell: &'static str, field: &str, expected: impl ToString, actual: impl ToString) {
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

fn field_text(record: Option<&Vec<u8>>, offset: usize, width: usize) -> String {
    record.map_or_else(|| "absent".to_string(), |record| read_little_endian(record, offset, width).to_string())
}

fn bytes_text(record: Option<&Vec<u8>>, start: usize, end: usize) -> String {
    record.map_or_else(|| "absent".to_string(), |record| hexadecimal_of(&record[start..end]))
}

fn times_text(record: Option<&Vec<u8>>) -> String {
    record.map_or_else(|| "absent".to_string(), |record| format!("{}/{}/{}", read_u64(record, 64), read_u64(record, 72), read_u64(record, 80)))
}

fn records_written_in(analysis: &SegmentAnalysis, tree: u64, packed_record_type: u64) -> Vec<Vec<u8>> {
    analysis
        .units
        .iter()
        .filter_map(|unit| match &unit.copy_on_first_device.body {
            DecodedBody::PackedContainer(container) if container.birth_tree == tree && container.packed_record_type == packed_record_type => {
                Some(container.records.clone())
            }
            _ => None,
        })
        .flatten()
        .collect()
}

fn index_node_written_in(analysis: &SegmentAnalysis, tree: u64) -> Option<&IndexNode> {
    analysis.units.iter().find_map(|unit| match &unit.copy_on_first_device.body {
        DecodedBody::IndexNode(node) if node.tree_identifier == tree => Some(node),
        _ => None,
    })
}

fn container_written_in(analysis: &SegmentAnalysis, tree: u64, packed_record_type: u64) -> Option<(&UnitRole, &PackedContainer)> {
    analysis.units.iter().find_map(|unit| match &unit.copy_on_first_device.body {
        DecodedBody::PackedContainer(container) if container.birth_tree == tree && container.packed_record_type == packed_record_type => Some((unit, container)),
        _ => None,
    })
}

/// 跑前登记第六节 6.1 的 L03-1 到 L03-14：一次删除类发布一组行。
/// 交回只记、不判的 L03-7 行（`name=l03_7_branch`、`name=l03_7_record`）。
fn judge_layout_three_cells(history: &HistoryExport, analysis: &SegmentAnalysis, before: &VersionState, after: &VersionState, rows: &mut Vec<CellRow>) -> Vec<String> {
    let txg = analysis.label_txg();
    let publish_kind = analysis.label_kind();
    let is_intent = publish_kind == "unlink_intent";
    let data_units_after = data_units_of_version(after).unwrap_or(u64::MAX);
    let is_final = !is_intent && data_units_after == 0;
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
    judge_tree_table_accounting_and_watermark_cells(analysis, after, is_final, &mut builder);
    builder.leaf_branch_records
}

/// 三个时间秒都写同一个值时的期望写法（与 `times_text` 同形）。
fn three_equal_times_text(seconds: &str) -> String {
    format!("{seconds}/{seconds}/{seconds}")
}

/// L03-5 的期望与可不可分辨（跑前登记 e172-r2 第一节 1.2、第六节 6.1）：意图那一版三格同值 = 调用方给的时间——导出带
/// `intent_write_time_seconds=`（H3）就取它、可分辨；不带（H1、H2）照第 1 次取 `sequential_write_time_seconds=`、报不可分辨；
/// 批取上一版这条记录的三格（layout/03 第 13 行「各批照抄上一版这条记录的值」）。
fn expected_times_of(is_intent: bool, configuration: &BTreeMap<String, String>, record_before: Option<&Vec<u8>>) -> (String, bool) {
    if !is_intent {
        return (times_text(record_before), true);
    }
    match configuration.get("intent_write_time_seconds") {
        Some(intent_seconds) => (three_equal_times_text(intent_seconds), true),
        None => (three_equal_times_text(configuration.get("sequential_write_time_seconds").map_or("", String::as_str)), false),
    }
}

impl SegmentAnalysis {
    fn label_txg(&self) -> u64 {
        match &self.label {
            SegmentLabel::Publish { txg, .. } => *txg,
            SegmentLabel::Outside { .. } => u64::MAX,
        }
    }
    fn label_kind(&self) -> String {
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
            tombstone_root.map_or_else(
                || "absent".to_string(),
                |node| {
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
                },
            ),
        );
    } else {
        builder.push("L03-10", "tombstone_container_written", "false", tombstone_container.is_some());
        builder.push("L03-11", "tombstone_root_written", "false", tombstone_root.is_some());
    }
}

fn judge_tree_table_accounting_and_watermark_cells(analysis: &SegmentAnalysis, after: &VersionState, is_final: bool, builder: &mut CellRowBuilder<'_>) {
    let tree_table = index_node_written_in(analysis, 0);
    builder.push("L03-12", "entry_count", 11, tree_table.map_or_else(|| "absent".to_string(), |node| node.entry_count.to_string()));
    let entries = tree_table.map(tree_table_entries_of).unwrap_or_default();
    for (tree, kind_code, head_identifier, root_is_zero) in [
        (DELETED_INODES_TREE, 9, INODE_TREE, is_final),
        (TOMBSTONE_TREE, 10, INODE_TREE, false),
        (LINEAGE_TREE, 11, 0, true),
        (LOGGED_OPERATIONS_TREE, 12, 0, true),
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
    let live_inodes = live_inode_count(after);
    builder.push(
        "L03-13",
        &format!("committed_reservation_h{tombstone_levels}_live{live_inodes}"),
        expected_committed_reservation(tombstone_levels, live_inodes),
        accounting_value(after, 6).map_or_else(|| "absent".to_string(), |value| value.to_string()),
    );
    let root_watermark = analysis.root_records.first().map_or_else(|| "absent".to_string(), |(_, _, record)| record.watermark.to_string());
    builder.push("L03-14", "root_record_watermark", TREE_IDENTIFIER_WATERMARK_AFTER_THE_FIRST_FILE, root_watermark);
    for (device, _, record) in &analysis.journal_records {
        builder.push("L03-14", &format!("journal_watermark_device{device}"), TREE_IDENTIFIER_WATERMARK_AFTER_THE_FIRST_FILE, record.watermark);
    }
}

// ---------------------------------------------------------------------------------------------
// 一段历史：逐段解码、带着现行那一版往下走。
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct HistoryAnalysis {
    name: String,
    segments: Vec<SegmentAnalysis>,
    states_before: Vec<VersionState>,
    states_after: Vec<VersionState>,
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
                _ => None,
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
struct SegmentRow {
    history: String,
    txg: u64,
    publish_kind: String,
    counts_for_row_three: bool,
    compares_role_names_verbatim: bool,
    expected_ordinals: String,
    actual_ordinals: String,
    expected_count_text: String,
    actual_count_text: String,
    expected_kind_text: String,
    actual_kind_text: String,
    expected_roles: String,
    actual_roles: String,
    root_force_unit_access_writes: usize,
    expected_root_location: String,
    actual_root_location: String,
}

impl SegmentRow {
    fn count_equal(&self) -> bool {
        self.expected_count_text == self.actual_count_text
    }
    fn kind_equal(&self) -> bool {
        self.expected_kind_text == self.actual_kind_text
    }
    fn roles_equal(&self) -> bool {
        self.expected_roles == self.actual_roles
    }
    fn root_equal(&self) -> bool {
        self.root_force_unit_access_writes == 1 && self.expected_root_location == self.actual_root_location
    }
}

/// R3.c 的比法（跑前登记 e172-r2 第一节 1.2）：H1 逐字比角色名；H2、H3 把 `t5@层级.盘.序号` 的序号那一段抹掉再比多重集合
/// （layout/03 第 51–55 行里的序号是 H1 那一种分配器位置上钉的），序号另报 `name=r3c_ordinal judged=false`。
fn compares_role_names_verbatim(history_name: &str) -> bool {
    history_name == "h1"
}

fn role_name_without_allocation_ordinal(role: &str) -> String {
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

/// 第 3 行（原第 3 行）判定的射程（主 agent 2026-10-05 定读法乙）：H1、H3 全部删除类发布，H2 只拿意图那一次。
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
    node.entries.iter().map(|entry| ((read_u32(entry, 0), read_u48(entry, 4)), (read_u16(entry, 10), read_u64(entry, 12)))).collect()
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
enum RecordChangeBasis {
    AllocatedInThisPublish,
    ReleasedInThisPublish,
    ReclaimedUnderTheFloor,
    Unmatched,
}

impl RecordChangeBasis {
    fn name(self) -> &'static str {
        match self {
            RecordChangeBasis::AllocatedInThisPublish => "allocated_in_this_publish",
            RecordChangeBasis::ReleasedInThisPublish => "released_in_this_publish",
            RecordChangeBasis::ReclaimedUnderTheFloor => "reclaimed_under_the_floor",
            RecordChangeBasis::Unmatched => "unmatched",
        }
    }
}

#[derive(Clone, Debug)]
struct UniqueUnitJudgement {
    role_name: String,
    previous_in_window: bool,
    changes: Vec<((u64, u64), Option<(u64, u64)>, Option<(u64, u64)>, RecordChangeBasis)>,
}

impl UniqueUnitJudgement {
    fn has_clause_basis(&self) -> bool {
        self.changes.iter().any(|(_, _, _, basis)| *basis != RecordChangeBasis::Unmatched)
    }
}

/// R4.b：多写的单元与窗口里同一角色上一次写出的那份比分配记录，逐条归 ①②③。
fn judge_unique_unit(unit: &UnitRole, segment: &SegmentAnalysis, before: &VersionState, after: &VersionState) -> UniqueUnitJudgement {
    let DecodedBody::IndexNode(node) = &unit.copy_on_first_device.body else {
        return UniqueUnitJudgement { role_name: unit.role_name.clone(), previous_in_window: false, changes: Vec::new() };
    };
    let previous = before.latest_by_identity.get(&unit.identity).and_then(|region| match &region.body {
        DecodedBody::IndexNode(previous_node) => Some(allocation_records_of(previous_node)),
        _ => None,
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

fn value_text(value: Option<(u64, u64)>) -> String {
    value.map_or_else(|| "absent".to_string(), |(span, generation)| format!("span{span:#06x}/generation{generation}"))
}

/// R4.c：这次发布写出的单元落点的最小与最大槽号、写到的分配记录树叶序号。
fn allocation_geometry_line(history_name: &str, segment: &SegmentAnalysis) -> String {
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

fn names_of_identities(segment: &SegmentAnalysis, identities: &BTreeSet<RoleIdentity>) -> Vec<String> {
    let mut names: Vec<String> = segment.units.iter().filter(|unit| identities.contains(&unit.identity)).map(|unit| unit.role_name.clone()).collect();
    names.sort();
    names
}

// ---------------------------------------------------------------------------------------------
// 第 7 行：td 与 tt 头里的层级与 key 区间（只记值、标读法，不判对错）。
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KeyRangeReading {
    EntryKeyRange,
    SubtreeCoverage,
    BothReadingsCoincide,
    Neither,
}

impl KeyRangeReading {
    fn name(self) -> &'static str {
        match self {
            KeyRangeReading::EntryKeyRange => "entry_key_range",
            KeyRangeReading::SubtreeCoverage => "subtree_coverage",
            KeyRangeReading::BothReadingsCoincide => "both_readings_coincide",
            KeyRangeReading::Neither => "neither",
        }
    }
}

/// 跑前登记 R7：key 区间 = [首条条目 key, 末条条目 key] ⇒ 条目键区间；= 这个节点罩住的整个 key 空间 ⇒ 子树覆盖区间。
fn key_range_reading(range: (u64, u64), entry_keys: (u64, u64), coverage_candidates: &[(u64, u64)]) -> KeyRangeReading {
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
enum WriteAttribution {
    Publish(u64),
    InstanceAcquisition,
    OtherMountStep,
}

impl WriteAttribution {
    fn text(&self) -> String {
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
fn attribute_system_configuration(
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
struct AttributionRow {
    step: usize,
    device: u32,
    region_kind: RegionKind,
    decoded: WriteAttribution,
    exported_segment: String,
    is_match: bool,
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
                _ => region.birth_or_checkpoint_txg().map_or(WriteAttribution::OtherMountStep, WriteAttribution::Publish),
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
struct CoverageTotals {
    layout_three_bytes: u64,
    layout_one_bytes: u64,
    constant_zero_bytes: u64,
    unregistered_bytes: u64,
    constant_zero_nonzero_bytes: u64,
    double_claimed_bytes: u64,
    total_bytes: u64,
    unregistered_ranges: Vec<String>,
    nonzero_padding_ranges: Vec<String>,
}

#[derive(Clone, Debug)]
struct HistoryJudgement {
    name: String,
    analysis: HistoryAnalysis,
    cell_rows: Vec<CellRow>,
    segment_rows: Vec<SegmentRow>,
    structure_failures: Vec<String>,
    role_mismatches: Vec<String>,
    role_notation_only_differences: Vec<String>,
    tiling_failures: Vec<String>,
    attribution_rows: Vec<AttributionRow>,
    coverage: CoverageTotals,
    key_range_lines: Vec<String>,
    leaf_branch_records: Vec<String>,
    time_threshold_rows: Vec<TimeThresholdRow>,
    /// 导出的 `name=impl_config` 那一行（R6.b 要意图与顺序写两个时间）。
    configuration: BTreeMap<String, String>,
}

/// R1.t（跑前登记 e172-r2 第六节 6.2）：窗口里每一次系统配置槽写，偏移 433 起 4 字节小端解出的 T_time。
#[derive(Clone, Debug, PartialEq, Eq)]
struct TimeThresholdRow {
    step: usize,
    device: u32,
    offset: u64,
    actual_milliseconds: u64,
}

impl TimeThresholdRow {
    fn is_expected(&self) -> bool {
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

fn coverage_of(analysis: &HistoryAnalysis) -> CoverageTotals {
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

fn judge_history(history: &HistoryExport) -> HistoryJudgement {
    let analysis = analyze_history(history);
    let mut cell_rows = Vec::new();
    let mut segment_rows = Vec::new();
    let mut structure_failures = Vec::new();
    let mut role_mismatches = Vec::new();
    let mut role_notation_only_differences = Vec::new();
    let mut key_range_lines = Vec::new();
    let mut leaf_branch_records = Vec::new();
    for (segment_index, segment) in analysis.segments.iter().enumerate() {
        let before = &analysis.states_before[segment_index];
        let after = &analysis.states_after[segment_index];
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
            leaf_branch_records.extend(judge_layout_three_cells(history, segment, before, after, &mut cell_rows));
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
    }
}

impl HistoryJudgement {
    fn intent_cell_is_equal(&self, cell: &str, field: &str) -> Option<bool> {
        self.cell_rows.iter().find(|row| row.cell == cell && row.field == field && row.publish_kind == "unlink_intent").map(CellRow::is_equal)
    }
    fn intent_segment_count_is_equal(&self) -> Option<bool> {
        self.segment_rows.iter().find(|row| row.publish_kind == "unlink_intent").map(SegmentRow::count_equal)
    }
    /// R8.c（跑前登记 e172-r2 第六节 6.4）的归错数。
    fn misattributed_writes(&self) -> usize {
        self.attribution_rows.iter().filter(|row| !row.is_match).count()
    }
    /// R1.t：T_time ≠ 5000 的系统配置槽写。
    fn time_threshold_rows_not_expected(&self) -> Vec<&TimeThresholdRow> {
        self.time_threshold_rows.iter().filter(|row| !row.is_expected()).collect()
    }
    /// S2 里「系统配置整槽校验和不对」那一类的条数（PC-H 要它改坏前后相同）。
    fn system_configuration_checksum_failures(&self) -> usize {
        self.structure_failures.iter().filter(|failure| failure.contains("系统配置整槽校验和不对")).count()
    }
    fn intent_times_row(&self) -> Option<&CellRow> {
        self.cell_rows.iter().find(|row| row.cell == "L03-5" && row.field == "times" && row.publish_kind == "unlink_intent")
    }
    fn intent_inode_container_checksum_failures(&self) -> usize {
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

// ---------------------------------------------------------------------------------------------
// 阳性对照（跑前登记第五节 5.4）：在内存里改坏一份副本，重跑全部检查，看目标行由绿转红。
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PositiveControl {
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
}

/// PC-F、PC-G 改坏之后预期的那一次归错（跑前登记 e172-r2 第五节 5.4 表「改坏之后预期红在哪」）：从原样导出算出来。
#[derive(Clone, Debug, PartialEq, Eq)]
struct ExpectedMisattribution {
    step: usize,
    exported_segment: String,
    decoded: Option<WriteAttribution>,
    forbidden_decoded: Option<WriteAttribution>,
}

impl PositiveControl {
    const EVERY_CONTROL: [PositiveControl; 8] = [
        PositiveControl::FlipIntentNlink,
        PositiveControl::NonzeroDeletedInodesPadding,
        PositiveControl::BumpTombstoneDeathTxg,
        PositiveControl::DropIntentFirstBarrier,
        PositiveControl::ShiftMountSegmentEndOneStepLater,
        PositiveControl::ShiftMountSegmentEndToItsLastWrite,
        PositiveControl::RewriteIntentTimeThresholdAsSeconds,
        PositiveControl::CopySequentialWriteTimeIntoTheIntentRecord,
    ];
    /// 跑前登记 e172-r2 第五节 5.4：适用的（对照, 历史）对共 18 个。
    const APPLICABLE_PAIRS_EXPECTED: usize = 18;
    fn name(self) -> &'static str {
        match self {
            PositiveControl::FlipIntentNlink => "PC-A",
            PositiveControl::NonzeroDeletedInodesPadding => "PC-B",
            PositiveControl::BumpTombstoneDeathTxg => "PC-C",
            PositiveControl::DropIntentFirstBarrier => "PC-D",
            PositiveControl::ShiftMountSegmentEndOneStepLater => "PC-F",
            PositiveControl::ShiftMountSegmentEndToItsLastWrite => "PC-G",
            PositiveControl::RewriteIntentTimeThresholdAsSeconds => "PC-H",
            PositiveControl::CopySequentialWriteTimeIntoTheIntentRecord => "PC-I",
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
        }
    }
    fn applies_to(self, history: &HistoryExport) -> bool {
        match self {
            PositiveControl::ShiftMountSegmentEndOneStepLater | PositiveControl::ShiftMountSegmentEndToItsLastWrite => {
                history.segments.iter().any(|segment| segment.publish_kind() == "mount_writes_before_its_first_publish")
            }
            PositiveControl::CopySequentialWriteTimeIntoTheIntentRecord => history.configuration.contains_key("intent_write_time_seconds"),
            PositiveControl::FlipIntentNlink
            | PositiveControl::NonzeroDeletedInodesPadding
            | PositiveControl::BumpTombstoneDeathTxg
            | PositiveControl::DropIntentFirstBarrier
            | PositiveControl::RewriteIntentTimeThresholdAsSeconds => true,
        }
    }
    /// 不适用时报的原因（`name=r_pc … applies=false reason=`）。
    fn reason_not_applicable(self) -> &'static str {
        match self {
            PositiveControl::ShiftMountSegmentEndOneStepLater | PositiveControl::ShiftMountSegmentEndToItsLastWrite => "no_mount_writes_before_its_first_publish",
            PositiveControl::CopySequentialWriteTimeIntoTheIntentRecord => "impl_config_without_intent_write_time_seconds",
            PositiveControl::FlipIntentNlink
            | PositiveControl::NonzeroDeletedInodesPadding
            | PositiveControl::BumpTombstoneDeathTxg
            | PositiveControl::DropIntentFirstBarrier
            | PositiveControl::RewriteIntentTimeThresholdAsSeconds => "always_applies",
        }
    }
}

fn intent_range(history: &HistoryExport) -> Option<(usize, usize)> {
    history.segments.iter().find(|segment| segment.publish_kind() == "unlink_intent").map(|segment| (segment.start_step, segment.end_step))
}

/// 改坏一份副本；改不到（找不到目标单元）交回 `None`。
fn corrupted_copy(history: &HistoryExport, control: PositiveControl) -> Option<HistoryExport> {
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
        PositiveControl::FlipIntentNlink
        | PositiveControl::NonzeroDeletedInodesPadding
        | PositiveControl::BumpTombstoneDeathTxg
        | PositiveControl::CopySequentialWriteTimeIntoTheIntentRecord => {
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
                    _ => {}
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
fn reseal_system_configuration_slot(slot: &mut [u8]) {
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
            let is_system_configuration_write = history.operations.iter().any(|operation| {
                matches!(operation, WindowOperation::Write(write) if write.step == last_write_step && region_kind_of(write) == RegionKind::SystemConfigurationSlot)
            });
            let original_attribution = baseline.attribution_rows.iter().find(|row| row.step == last_write_step).map(|row| row.decoded.clone());
            Some(ExpectedMisattribution {
                step: last_write_step,
                exported_segment: format!("{next_kind}_txg{next_txg}"),
                decoded: if is_system_configuration_write { Some(WriteAttribution::InstanceAcquisition) } else { original_attribution },
                forbidden_decoded: Some(WriteAttribution::Publish(*next_txg)),
            })
        }
        PositiveControl::FlipIntentNlink
        | PositiveControl::NonzeroDeletedInodesPadding
        | PositiveControl::BumpTombstoneDeathTxg
        | PositiveControl::DropIntentFirstBarrier
        | PositiveControl::RewriteIntentTimeThresholdAsSeconds
        | PositiveControl::CopySequentialWriteTimeIntoTheIntentRecord => None,
    }
}

/// 一对（对照, 历史）一行；`holds` 为 `None` 是不适用（`applies=false`，不计进 18 对）。
fn positive_control_line(history: &HistoryExport, baseline: &HistoryJudgement, control: PositiveControl) -> (String, Option<bool>) {
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

fn anchor_line(anchor: &str, history: &str, detail: &str, expected: impl ToString, actual: impl ToString) -> (String, bool) {
    let expected_text = expected.to_string();
    let actual_text = actual.to_string();
    let equal = expected_text == actual_text;
    (format!("name=anchor anchor={anchor} history={history} {detail} expected={expected_text} actual={actual_text} equal={equal}"), equal)
}

fn anchor_lines_of_history(judgement: &HistoryJudgement) -> Vec<(String, bool)> {
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
        let registered_tag_six = registered_committed_reservation_of_history(history).map_or_else(|| "unknown_history".to_string(), |value| value.to_string());
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

/// 跑前登记 e172-r2 第七节 A-tag6：按历史名分的字面常量，不由解码的 n_live 现算（第五节 5.8 改9）。
fn registered_committed_reservation_of_history(history: &str) -> Option<u64> {
    match history {
        "h1" | "h2" => Some(98_304),
        "h3" => Some(98_386),
        _ => None,
    }
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
fn trajectory_lines(judgement: &HistoryJudgement) -> Vec<String> {
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
        lines.push(format!(
            "name=r_traj history={} txg={} publish={} deleted_inodes={queue} data_units={data_units} tag6={tag_six} tag4={tag_four}",
            judgement.name,
            segment.label_txg(),
            segment.label_kind()
        ));
        for (series_name, value) in [("deleted_inodes", queue), ("data_units", data_units), ("tag6", tag_six), ("tag4", tag_four)] {
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

// ---------------------------------------------------------------------------------------------
// 产物：全部判定行（不带 E7RESULT 前缀；main 交 Emitter 打）。
// ---------------------------------------------------------------------------------------------

const LAYOUT_THREE_CELLS: [&str; 14] = [
    "L03-1", "L03-2", "L03-3", "L03-4", "L03-5", "L03-6", "L03-7", "L03-8", "L03-9", "L03-10", "L03-11", "L03-12", "L03-13", "L03-14",
];

/// V6（跑前登记 e172-r2 第十一节）：四个参数；H3 带 `intent_write_time_seconds=` 且 = `sequential_write_time_seconds=` + 60，H1、H2 不带这一格。
fn configuration_matches_registration(history: &HistoryExport) -> bool {
    let expected = [("devices", "2"), ("image_bytes", "4294967296"), ("physical_block_size", "512"), ("batch_data_units", "1")];
    let parameters_match = expected.iter().all(|(key, value)| history.configuration.get(*key).map(String::as_str) == Some(*value));
    let intent_seconds = field_number(&history.configuration, "intent_write_time_seconds");
    let sequential_seconds = field_number(&history.configuration, "sequential_write_time_seconds");
    let intent_time_matches = match history.name.as_str() {
        "h3" => intent_seconds.is_some() && intent_seconds == sequential_seconds.map(|seconds| seconds + 60),
        "h1" | "h2" => !history.configuration.contains_key("intent_write_time_seconds"),
        _ => false,
    };
    parameters_match && intent_time_matches
}

/// 两份导出：第一份是不带参数的 H1、H2，第二份是 `--history h3`（跑前登记 e172-r2 第五节 5.8 改1、改2）。
const EXPORT_FILE_LABELS: [&str; 2] = ["h1h2", "h3"];
/// V6 / `histories_present`：每份导出里该有的历史，次序也要一样。
const HISTORIES_OF_EACH_EXPORT_FILE: [&[&str]; 2] = [&["h1", "h2"], &["h3"]];

/// 两份导出各解一遍，历史按「第一份在前、第二份在后」接起来。
fn parse_export_files(export_texts: &[&str; 2]) -> Vec<ExportFile> {
    export_texts.iter().map(|text| parse_export(text)).collect()
}

fn report_lines(export_texts: &[&str; 2]) -> Vec<String> {
    let export_files = parse_export_files(export_texts);
    let mut lines = Vec::new();
    let mut void_flags: Vec<(String, bool)> = Vec::new();
    let mut histories_present = true;
    for ((label, text), (export, expected_histories)) in EXPORT_FILE_LABELS.iter().zip(export_texts).zip(export_files.iter().zip(HISTORIES_OF_EACH_EXPORT_FILE)) {
        lines.push(format!(
            "name=export_file file={label} sha256={} lines={} anchored_lines={} done_emitted={} non_hexadecimal_lines={} malformed_lines={}",
            sha256_hexadecimal(text.as_bytes()),
            export.line_count,
            export.anchored_line_count,
            export.done_emitted.map_or_else(|| "absent".to_string(), |value| value.to_string()),
            export.non_hexadecimal_line_count,
            export.malformed_lines.len()
        ));
        for malformed in &export.malformed_lines {
            lines.push(format!("name=malformed_line file={label} detail={}", malformed.replace(' ', "_")));
        }
        let lines_anchor = anchor_line(
            "A-lines",
            "all",
            &format!("file={label} lines_anchored_done"),
            format!("{0}/{0}", export.line_count),
            format!("{}/{}", export.anchored_line_count, export.done_emitted.unwrap_or(0)),
        );
        void_flags.push((format!("a_lines_equal_{label}"), lines_anchor.1));
        lines.push(lines_anchor.0);
        let history_names: Vec<&str> = export.histories.iter().map(|history| history.name.as_str()).collect();
        histories_present &= history_names == expected_histories;
    }
    let crc_anchor = anchor_line("A-crc", "all", "ascii_123456789", "e3069283", format!("{:08x}", castagnoli_crc32(b"123456789")));
    void_flags.push(("a_crc_equal".to_string(), crc_anchor.1));
    lines.push(crc_anchor.0);
    void_flags.push(("histories_present".to_string(), histories_present));
    let histories: Vec<HistoryExport> = export_files.into_iter().flat_map(|export| export.histories).collect();
    for history in &histories {
        let content_anchor = anchor_line("A-content", &history.name, "file_bytes", EXPECTED_FILE_BYTES, history.configuration.get("file_bytes").cloned().unwrap_or_default());
        void_flags.push((format!("a_content_equal_{}", history.name), content_anchor.1));
        lines.push(content_anchor.0);
        let configuration_ok = configuration_matches_registration(history);
        void_flags.push((format!("configuration_matches_{}", history.name), configuration_ok));
        lines.push(format!("name=v6_configuration history={} matches_registration={configuration_ok}", history.name));
    }
    for (anchor, detail, expected, actual) in [
        ("A-capacity", "tombstone_container_records", 815, tombstone_container_capacity()),
        ("A-capacity", "deleted_inodes_root_entries", 2031, deleted_inodes_root_capacity()),
        ("A-capacity", "tombstone_root_entry_bytes", 120, tombstone_root_entry_bytes()),
    ] {
        lines.push(anchor_line(anchor, "all", detail, expected, actual).0);
    }
    let judgements: Vec<HistoryJudgement> = histories.iter().map(judge_history).collect();
    let mut anchor_failures = 0usize;
    for judgement in &judgements {
        for (line, equal) in anchor_lines_of_history(judgement) {
            anchor_failures += usize::from(!equal);
            lines.push(line);
        }
    }
    let mut stop_counts = (0usize, 0usize, 0usize, 0usize);
    for judgement in &judgements {
        lines.push(format!(
            "name=structure history={} regions={} s2_structure_failures={} s3_role_mismatches={} s3_index_notation_only={} s4_tiling_failures={} unreferenced_units={}",
            judgement.name,
            judgement.analysis.segments.iter().map(|segment| segment.regions.len()).sum::<usize>(),
            judgement.structure_failures.len(),
            judgement.role_mismatches.len(),
            judgement.role_notation_only_differences.len(),
            judgement.tiling_failures.len(),
            judgement.analysis.segments.iter().map(|segment| segment.unreferenced_units.len()).sum::<usize>()
        ));
        for (label, list) in [("s2", &judgement.structure_failures), ("s3", &judgement.role_mismatches), ("s3_notation", &judgement.role_notation_only_differences), ("s4", &judgement.tiling_failures)] {
            for item in list {
                lines.push(format!("name=stop_detail check={label} detail={}", item.replace(' ', "_")));
            }
        }
        for segment in &judgement.analysis.segments {
            for unreferenced in &segment.unreferenced_units {
                lines.push(format!("name=unreferenced_unit history={} detail={}", judgement.name, unreferenced.replace(' ', "_")));
            }
        }
        stop_counts.0 += judgement.structure_failures.len();
        stop_counts.1 += judgement.role_mismatches.len();
        stop_counts.2 += judgement.role_notation_only_differences.len();
        stop_counts.3 += judgement.tiling_failures.len();
    }
    lines.extend(row_one_lines(&judgements));
    lines.extend(row_three_lines(&judgements));
    lines.extend(row_four_and_five_lines(&judgements));
    lines.extend(row_six_seven_eight_lines(&judgements));
    for judgement in &judgements {
        lines.extend(trajectory_lines(judgement));
    }
    let mut controls_holding = 0usize;
    let mut controls_run = 0usize;
    let mut controls_failed_by_history: Vec<String> = Vec::new();
    for (history, judgement) in histories.iter().zip(&judgements) {
        let mut failed_here = 0usize;
        for control in PositiveControl::EVERY_CONTROL {
            let (line, holds) = positive_control_line(history, judgement, control);
            if let Some(holds) = holds {
                controls_run += 1;
                controls_holding += usize::from(holds);
                failed_here += usize::from(!holds);
            }
            lines.push(line);
        }
        controls_failed_by_history.push(format!("positive_controls_failed_{}={failed_here}", history.name));
    }
    lines.extend(row_two_lines(&judgements));
    let branch_check = leaf_branch_check(&judgements);
    lines.push(branch_check.0);
    lines.push(format!(
        "name=verdict check=stops s2_structure_failures={} s3_role_mismatches={} s3_index_notation_only={} s3_l03_7_second_branch_not_exactly_h3={} s4_tiling_failures={}",
        stop_counts.0,
        stop_counts.1,
        stop_counts.2,
        usize::from(!branch_check.1),
        stop_counts.3
    ));
    let void_text: Vec<String> = void_flags.iter().map(|(key, value)| format!("{key}={value}")).collect();
    lines.push(format!(
        "name=verdict check=voids {} positive_controls_failed={} {} positive_controls_run={controls_run} positive_controls_expected={} anchor_not_equal={anchor_failures}",
        void_text.join(" "),
        controls_run - controls_holding,
        controls_failed_by_history.join(" "),
        PositiveControl::APPLICABLE_PAIRS_EXPECTED
    ));
    lines
}

/// S3 另加的一条（跑前登记 e172-r2 第一节 1.2、第十一节 S3）：L03-7 选到第二支的历史恰是 H3。
fn leaf_branch_check(judgements: &[HistoryJudgement]) -> (String, bool) {
    let mut second_branch_histories: Vec<&str> = judgements
        .iter()
        .filter(|judgement| judgement.leaf_branch_records.iter().any(|line| line.starts_with("name=l03_7_branch ") && line.contains(" branch=other_records ")))
        .map(|judgement| judgement.name.as_str())
        .collect();
    second_branch_histories.sort_unstable();
    let is_exactly_h3 = second_branch_histories == ["h3"];
    (
        format!(
            "name=l03_7_branch_check second_branch_histories={} exactly_h3={is_exactly_h3}",
            if second_branch_histories.is_empty() { "none".to_string() } else { second_branch_histories.join(",") }
        ),
        is_exactly_h3,
    )
}

/// 第 1 行（原第 1 行 R1.a、R1.b 与 R1.t）：三段历史逐段分开计数（跑前登记 e172-r2 第六节 6.2）。
fn row_one_lines(judgements: &[HistoryJudgement]) -> Vec<String> {
    let mut lines = Vec::new();
    let mut not_equal = 0usize;
    let mut not_discriminable = 0usize;
    let mut per_history: Vec<String> = Vec::new();
    for judgement in judgements {
        let mut not_equal_here = 0usize;
        for row in &judgement.cell_rows {
            lines.push(row.line());
            if row.discriminable {
                not_equal_here += usize::from(!row.is_equal());
            } else {
                not_discriminable += 1;
            }
        }
        lines.extend(judgement.leaf_branch_records.iter().cloned());
        not_equal += not_equal_here;
        let mut cells_without_hits = Vec::new();
        for cell in LAYOUT_THREE_CELLS {
            let hits = judgement.cell_rows.iter().filter(|row| row.cell == cell && row.discriminable && row.actual != "absent").count();
            let not_equal_in_cell = judgement.cell_rows.iter().filter(|row| row.cell == cell && row.discriminable && !row.is_equal()).count();
            lines.push(format!("name=r1b history={} cell={cell} discriminable_hits={hits} not_equal={not_equal_in_cell}", judgement.name));
            if hits == 0 {
                cells_without_hits.push(cell);
            }
        }
        per_history.push(format!(
            "not_equal_cells_{0}={not_equal_here} cells_without_hits_{0}={1} cells_without_hits_list_{0}={2}",
            judgement.name,
            cells_without_hits.len(),
            if cells_without_hits.is_empty() { "none".to_string() } else { cells_without_hits.join(",") }
        ));
    }
    // R1.t：每段历史窗口里每一次系统配置槽写一行，再一行计数（分母 = 槽写次数）。
    let mut time_threshold_not_expected = 0usize;
    for judgement in judgements {
        for row in &judgement.time_threshold_rows {
            lines.push(format!(
                "name=r1t history={} step={} device={} offset={} expected={EXPECTED_TIME_THRESHOLD_MILLISECONDS} actual={} equal={}",
                judgement.name,
                row.step,
                row.device,
                row.offset,
                row.actual_milliseconds,
                row.is_expected()
            ));
        }
        let not_expected_here = judgement.time_threshold_rows_not_expected().len();
        time_threshold_not_expected += not_expected_here;
        lines.push(format!(
            "name=r1t_summary history={} system_configuration_slot_writes={} t_time_not_5000={not_expected_here}",
            judgement.name,
            judgement.time_threshold_rows.len()
        ));
        per_history.push(format!("t_time_not_5000_{}={not_expected_here}", judgement.name));
    }
    lines.push(format!(
        "name=verdict row=1 not_equal_cells={not_equal} discriminable_false_rows={not_discriminable} t_time_not_5000={time_threshold_not_expected} {}",
        per_history.join(" ")
    ));
    lines
}

fn row_three_lines(judgements: &[HistoryJudgement]) -> Vec<String> {
    let mut lines = Vec::new();
    let mut mismatch_counts: BTreeMap<String, [usize; 4]> = BTreeMap::new();
    for row in judgements.iter().flat_map(|judgement| judgement.segment_rows.iter()) {
        lines.push(format!(
            "name=r3 history={} txg={} publish={} counts_for_row3={} expected_segments={} actual_segments={} segments_equal={} expected_kinds={} actual_kinds={} kinds_equal={} expected_roles={} actual_roles={} roles_equal={} root_fua_writes={} expected_root={} actual_root={} root_equal={}",
            row.history,
            row.txg,
            row.publish_kind,
            row.counts_for_row_three,
            row.expected_count_text,
            row.actual_count_text,
            row.count_equal(),
            row.expected_kind_text,
            row.actual_kind_text,
            row.kind_equal(),
            row.expected_roles,
            row.actual_roles,
            row.roles_equal(),
            row.root_force_unit_access_writes,
            row.expected_root_location,
            row.actual_root_location,
            row.root_equal()
        ));
        if !row.compares_role_names_verbatim {
            lines.push(format!(
                "name=r3c_ordinal history={} txg={} publish={} expected_ordinals={} actual_ordinals={} judged=false",
                row.history, row.txg, row.publish_kind, row.expected_ordinals, row.actual_ordinals
            ));
        }
        if row.counts_for_row_three {
            let counts = mismatch_counts.entry(row.history.clone()).or_insert([0usize; 4]);
            counts[0] += usize::from(!row.count_equal());
            counts[1] += usize::from(!row.kind_equal());
            counts[2] += usize::from(!row.roles_equal());
            counts[3] += usize::from(!row.root_equal());
        }
    }
    // R3.d：H1、H3 各一行整窗段串（H2 的窗口里有不是删除类发布的段，不报整窗）。
    let mut window_equal: BTreeMap<String, bool> = BTreeMap::new();
    for (history_name, units_per_publish) in WINDOW_UNITS_PER_DELETION_PUBLISH {
        let Some(judgement) = judgements.iter().find(|judgement| judgement.name == history_name) else { continue };
        let (expected, actual, equal) = window_string_of(judgement, &units_per_publish);
        window_equal.insert(history_name.to_string(), equal);
        lines.push(format!("name=r3d history={history_name} expected={expected} actual={actual} equal={equal}"));
    }
    lines.extend(final_batch_geometry_lines(judgements));
    let mut verdict_fields = Vec::new();
    for judgement in judgements {
        let Some(counts) = mismatch_counts.get(&judgement.name) else { continue };
        let window_text = window_equal.get(&judgement.name).map_or_else(|| "not_reported".to_string(), bool::to_string);
        verdict_fields.push(format!(
            "{0}_segment_mismatches={1} {0}_kind_mismatches={2} {0}_role_mismatches={3} {0}_root_mismatches={4} {0}_window_string_equal={window_text}",
            judgement.name, counts[0], counts[1], counts[2], counts[3]
        ));
    }
    lines.push(format!("name=verdict row=3 {}", verdict_fields.join(" ")));
    lines
}

/// R3.d 的期望（跑前登记 e172-r2 第六节 6.2）：H1 = layout/03 第 59 行四串首尾相接；H3 = 每盘单元数 13、12、11、11 四串首尾相接。
const WINDOW_UNITS_PER_DELETION_PUBLISH: [(&str, [usize; 4]); 2] = [("h1", [13, 12, 11, 10]), ("h3", [13, 12, 11, 11])];

fn window_string_of(judgement: &HistoryJudgement, units_per_publish: &[usize]) -> (String, String, bool) {
    let operations: Vec<OperationView> = judgement.analysis.segments.iter().flat_map(|segment| segment.operations.iter().cloned()).collect();
    let pieces = cut_into_pieces(&operations);
    let expected = units_per_publish.iter().map(|units| expected_count_text(*units)).collect::<Vec<_>>().join("+");
    let actual = segment_count_text(&pieces);
    let equal = expected == actual;
    (expected, actual, equal)
}

/// 几何敏感性（跑前登记 e172-r2 第八节 8.2 第 1 行）：H1 的最后一批期望（第一支，10 个单元）与 H3 的（第二支，11 个）都拿去判 H3 的最后一批。
fn final_batch_geometry_lines(judgements: &[HistoryJudgement]) -> Vec<String> {
    let Some(judgement) = judgements.iter().find(|judgement| judgement.name == "h3") else { return Vec::new() };
    let Some((_, final_batch)) = deletion_segment_with_units_after(judgement, "unlink_batch", 0) else { return Vec::new() };
    let actual = segment_count_text(&cut_into_pieces(&final_batch.operations));
    let units_of = |leaf: DeletedRecordLeaf| registered_roles("unlink_batch", 0, leaf).map_or(0, |roles| roles.len());
    let only_record_expectation = expected_count_text(units_of(DeletedRecordLeaf::OnlyTheDeletedRecord));
    let other_records_expectation = expected_count_text(units_of(DeletedRecordLeaf::BesideOtherRecords));
    let only_record_green = only_record_expectation == actual;
    let other_records_green = other_records_expectation == actual;
    vec![format!(
        "name=r3_geometry history=h3 txg={} expected_only_record={only_record_expectation} expected_other_records={other_records_expectation} actual={actual} only_record_green={only_record_green} other_records_green={other_records_green} expectations_differ={} exactly_one_green={}",
        final_batch.label_txg(),
        only_record_expectation != other_records_expectation,
        only_record_green != other_records_green
    )]
}

fn deletion_segment_with_units_after<'judgement>(judgement: &'judgement HistoryJudgement, publish_kind: &str, data_units_after: u64) -> Option<(usize, &'judgement SegmentAnalysis)> {
    judgement.analysis.segments.iter().enumerate().find(|(segment_index, segment)| {
        segment.label_kind() == publish_kind && data_units_of_version(&judgement.analysis.states_after[*segment_index]) == Some(data_units_after)
    })
}

fn unique_unit_lines(history_name: &str, judgement: &HistoryJudgement, segment_index: usize, unique: &BTreeSet<RoleIdentity>, without_basis: &mut usize) -> Vec<String> {
    let segment = &judgement.analysis.segments[segment_index];
    let mut lines = Vec::new();
    for unit in segment.units.iter().filter(|unit| unique.contains(&unit.identity)) {
        let unit_judgement = judge_unique_unit(unit, segment, &judgement.analysis.states_before[segment_index], &judgement.analysis.states_after[segment_index]);
        let count_of = |wanted: RecordChangeBasis| unit_judgement.changes.iter().filter(|(_, _, _, basis)| *basis == wanted).count();
        lines.push(format!(
            "name=r4b history={history_name} txg={} role={} previous_in_window={} changed_records={} allocated_in_this_publish={} released_in_this_publish={} reclaimed_under_the_floor={} unmatched={} has_clause_basis={}",
            segment.label_txg(),
            unit_judgement.role_name,
            unit_judgement.previous_in_window,
            unit_judgement.changes.len(),
            count_of(RecordChangeBasis::AllocatedInThisPublish),
            count_of(RecordChangeBasis::ReleasedInThisPublish),
            count_of(RecordChangeBasis::ReclaimedUnderTheFloor),
            count_of(RecordChangeBasis::Unmatched),
            unit_judgement.has_clause_basis()
        ));
        for ((device, slot), before_value, after_value, basis) in &unit_judgement.changes {
            lines.push(format!(
                "name=r4b_record history={history_name} txg={} role={} device={device} slot={slot} before={} after={} class={}",
                segment.label_txg(),
                unit_judgement.role_name,
                value_text(*before_value),
                value_text(*after_value),
                basis.name()
            ));
        }
        *without_basis += usize::from(!unit_judgement.has_clause_basis());
    }
    lines
}

fn row_four_and_five_lines(judgements: &[HistoryJudgement]) -> Vec<String> {
    let mut lines = Vec::new();
    let session = judgements.iter().find(|judgement| judgement.name == "h1");
    let continuation = judgements.iter().find(|judgement| judgement.name == "h2");
    let mut pairs_with_unique_roles = 0usize;
    let mut without_basis = 0usize;
    if let (Some(session), Some(continuation)) = (session, continuation) {
        for data_units_after in [2u64, 1, 0] {
            let session_segment = deletion_segment_with_units_after(session, "unlink_batch", data_units_after);
            let continuation_segment = deletion_segment_with_units_after(continuation, "mount_continuation_unlink_batch", data_units_after);
            let (Some((session_index, session_segment)), Some((continuation_index, continuation_segment))) = (session_segment, continuation_segment) else {
                lines.push(format!("name=r4a data_units_after={data_units_after} pair=missing"));
                continue;
            };
            let session_identities = session_segment.identities();
            let continuation_identities = continuation_segment.identities();
            let continuation_only: BTreeSet<RoleIdentity> = continuation_identities.difference(&session_identities).cloned().collect();
            let session_only: BTreeSet<RoleIdentity> = session_identities.difference(&continuation_identities).cloned().collect();
            let same_shape = continuation_only.is_empty() && session_only.is_empty();
            pairs_with_unique_roles += usize::from(!same_shape);
            lines.push(format!(
                "name=r4a data_units_after={data_units_after} continuation_txg={} session_txg={} continuation_units={} session_units={} continuation_only={} session_only={} same_shape={same_shape}",
                continuation_segment.label_txg(),
                session_segment.label_txg(),
                continuation_identities.len(),
                session_identities.len(),
                names_of_identities(continuation_segment, &continuation_only).join(","),
                names_of_identities(session_segment, &session_only).join(",")
            ));
            lines.extend(unique_unit_lines("h2", continuation, continuation_index, &continuation_only, &mut without_basis));
            lines.extend(unique_unit_lines("h1", session, session_index, &session_only, &mut without_basis));
        }
    }
    // R4.c：H1、H2、H3 都报（H3 是第三个分配器位置，也是第 1 行 R3.c 序号那一格的出处）。
    for judgement in judgements.iter().filter(|judgement| ["h1", "h2", "h3"].contains(&judgement.name.as_str())) {
        for segment in judgement.analysis.segments.iter().filter(|segment| segment.is_deletion_publish) {
            lines.push(allocation_geometry_line(&judgement.name, segment));
        }
    }
    lines.push(format!("name=verdict row=4 pairs_with_unique_roles={pairs_with_unique_roles} unique_units_without_basis={without_basis}"));
    // R5.a（附带，不进任何判定）：H1 最后一批与 2 → 1 那一批的角色集合差（两个方向）。
    if let Some((two_to_one, final_batch, only_two_to_one, only_final)) = session.and_then(final_batch_difference) {
        let row_five_h1_matches = only_two_to_one == ["t3"] && only_final.is_empty();
        lines.push(format!(
            "name=r5a history=h1 two_to_one_txg={} final_txg={} only_in_two_to_one={} only_in_final={} matches_registered={row_five_h1_matches} incidental=true",
            two_to_one.label_txg(),
            final_batch.label_txg(),
            names_text(&only_two_to_one),
            names_text(&only_final)
        ));
    }
    // R5.b（跑前登记 e172-r2 第六节 6.3）：H3 最后一批写了 t3（叶里还剩 2 号）且 2 → 1 独有里没有 t3 ⇒「照现写」。
    let mut row_five_h3_matches = false;
    let mut final_writes_inode_leaf = false;
    let h3 = judgements.iter().find(|judgement| judgement.name == "h3");
    if let Some((two_to_one, final_batch, only_two_to_one, only_final)) = h3.and_then(final_batch_difference) {
        final_writes_inode_leaf = final_batch.role_names().iter().any(|role| role == "t3");
        let two_to_one_only_has_inode_leaf = only_two_to_one.iter().any(|role| role == "t3");
        row_five_h3_matches = final_writes_inode_leaf && !two_to_one_only_has_inode_leaf;
        lines.push(format!(
            "name=r5b history=h3 two_to_one_txg={} final_txg={} final_writes_t3={final_writes_inode_leaf} only_in_two_to_one={} only_in_final={} matches_registered={row_five_h3_matches}",
            two_to_one.label_txg(),
            final_batch.label_txg(),
            names_text(&only_two_to_one),
            names_text(&only_final)
        ));
        // 判别力自证（第八节 8.2 第 3 行）：H1 的期望（2 → 1 独有 = {t3}、最后一批独有为空）拿去判 H3。
        let h1_expectation_green_on_h3 = only_two_to_one == ["t3"] && only_final.is_empty();
        lines.push(format!(
            "name=r5b_discrimination history=h3 h1_expectation_green={h1_expectation_green_on_h3} h3_expectation_green={row_five_h3_matches} exactly_one_green={}",
            h1_expectation_green_on_h3 != row_five_h3_matches
        ));
    }
    lines.push(format!("name=verdict row=5 h3_final_writes_t3={final_writes_inode_leaf} h3_matches_registered={row_five_h3_matches}"));
    lines
}

fn names_text(names: &[String]) -> String {
    if names.is_empty() { "none".to_string() } else { names.join(",") }
}

/// 一段历史里 2 → 1 那一批与最后一批（会话批，`unlink_batch`）的角色集合差：（2 → 1 那一段、最后一批那一段、2 → 1 独有、最后一批独有）。
fn final_batch_difference(judgement: &HistoryJudgement) -> Option<(&SegmentAnalysis, &SegmentAnalysis, Vec<String>, Vec<String>)> {
    let (_, two_to_one) = deletion_segment_with_units_after(judgement, "unlink_batch", 1)?;
    let (_, final_batch) = deletion_segment_with_units_after(judgement, "unlink_batch", 0)?;
    let two_to_one_identities = two_to_one.identities();
    let final_identities = final_batch.identities();
    let only_two_to_one = names_of_identities(two_to_one, &two_to_one_identities.difference(&final_identities).cloned().collect());
    let only_final = names_of_identities(final_batch, &final_identities.difference(&two_to_one_identities).cloned().collect());
    Some((two_to_one, final_batch, only_two_to_one, only_final))
}

fn row_six_seven_eight_lines(judgements: &[HistoryJudgement]) -> Vec<String> {
    let mut lines = Vec::new();
    // 第 4 行（原第 6 行，跑前登记 e172-r2 第一节 1.2、第六节 6.3）：R6.a = H2 的意图与续做批里有这条记录的两批；
    // R6.b = H3 的意图与 3 → 2、2 → 1 两批。最后一批不写这条记录（照 L03-7 判），本来就没有 L03-4、L03-5 行。
    let mut counts: BTreeMap<&str, (usize, usize)> = BTreeMap::from([("r6a", (0, 0)), ("r6b", (0, 0))]);
    for row in judgements.iter().flat_map(|judgement| judgement.cell_rows.iter()) {
        let line_name = match row.history.as_str() {
            "h2" => "r6a",
            "h3" => "r6b",
            _ => continue,
        };
        if row.cell != "L03-4" && row.cell != "L03-5" {
            continue;
        }
        lines.push(format!(
            "name={line_name} cell={} history={} txg={} publish={} expected={} actual={} equal={} discriminable={}",
            row.cell,
            row.history,
            row.txg,
            row.publish_kind,
            row.expected,
            row.actual,
            row.is_equal(),
            row.discriminable
        ));
        let entry = counts.entry(line_name).or_insert((0, 0));
        if row.discriminable {
            entry.0 += usize::from(!row.is_equal());
        } else {
            entry.1 += 1;
        }
    }
    // R6.b 的前提（V6 也核）：H3 意图的调用方时间与顺序写的时间不同，意图那一格才分辨得开。
    let mut intent_differs_from_sequential = false;
    if let Some(h3) = judgements.iter().find(|judgement| judgement.name == "h3") {
        let configuration = &h3.configuration;
        let intent_seconds = configuration.get("intent_write_time_seconds").cloned().unwrap_or_else(|| "absent".to_string());
        let sequential_seconds = configuration.get("sequential_write_time_seconds").cloned().unwrap_or_else(|| "absent".to_string());
        intent_differs_from_sequential = intent_seconds != "absent" && intent_seconds != sequential_seconds;
        lines.push(format!(
            "name=r6b_intent_time history=h3 intent_write_time_seconds={intent_seconds} sequential_write_time_seconds={sequential_seconds} differ={intent_differs_from_sequential}"
        ));
    }
    let (r6a_not_equal, r6a_not_discriminable) = counts["r6a"];
    let (r6b_not_equal, r6b_not_discriminable) = counts["r6b"];
    lines.push(format!(
        "name=verdict row=6 r6a_not_equal_rows={r6a_not_equal} r6a_discriminable_false_rows={r6a_not_discriminable} r6b_not_equal_rows={r6b_not_equal} r6b_discriminable_false_rows={r6b_not_discriminable} r6b_intent_differs_from_sequential={intent_differs_from_sequential}"
    ));
    let mut recorded = 0usize;
    for judgement in judgements {
        recorded += judgement.key_range_lines.iter().filter(|line| line.starts_with("name=r7 ")).count();
        lines.extend(judgement.key_range_lines.iter().cloned());
    }
    lines.push(format!("name=verdict row=7 recorded_nodes={recorded}"));
    // 第 5 行（原第 8 行）：判据只用 R8.c（跑前登记 e172-r2 第六节 6.4）；R8.b 不登记、不报。
    let mut attributed = 0usize;
    let mut misattributed = 0usize;
    for judgement in judgements {
        for row in &judgement.attribution_rows {
            attributed += 1;
            misattributed += usize::from(!row.is_match);
            lines.push(format!(
                "name=r8c history={} step={} device={} kind={} decoded={} exported_segment={} match={}",
                judgement.name,
                row.step,
                row.device,
                row.region_kind.step_kind_name(),
                row.decoded.text(),
                row.exported_segment,
                row.is_match
            ));
        }
    }
    lines.push(format!("name=verdict row=8 attributed_writes={attributed} r8c_misattributed_writes={misattributed}"));
    lines
}

fn row_two_lines(judgements: &[HistoryJudgement]) -> Vec<String> {
    let mut lines = Vec::new();
    let mut unregistered_total = 0u64;
    let mut nonzero_total = 0u64;
    let mut double_claimed_total = 0u64;
    for judgement in judgements {
        let coverage = &judgement.coverage;
        lines.push(format!(
            "name=r2a history={} total_bytes={} layout03_cell_bytes={} layout01_cell_bytes={} constant_zero_bytes={} unregistered_bytes={} double_claimed_bytes={} sum_matches_total={}",
            judgement.name,
            coverage.total_bytes,
            coverage.layout_three_bytes,
            coverage.layout_one_bytes,
            coverage.constant_zero_bytes,
            coverage.unregistered_bytes,
            coverage.double_claimed_bytes,
            coverage.layout_three_bytes + coverage.layout_one_bytes + coverage.constant_zero_bytes + coverage.unregistered_bytes == coverage.total_bytes
        ));
        for range in &coverage.unregistered_ranges {
            lines.push(format!("name=r2b {range}"));
        }
        lines.push(format!("name=r2c history={} constant_zero_nonzero_bytes={}", judgement.name, coverage.constant_zero_nonzero_bytes));
        for range in &coverage.nonzero_padding_ranges {
            lines.push(format!("name=r2c_range {range}"));
        }
        unregistered_total += coverage.unregistered_bytes;
        nonzero_total += coverage.constant_zero_nonzero_bytes;
        double_claimed_total += coverage.double_claimed_bytes;
    }
    lines.push(format!(
        "name=verdict row=2 unregistered_bytes={unregistered_total} constant_zero_nonzero_bytes={nonzero_total} double_claimed_bytes={double_claimed_total}"
    ));
    lines
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let [first_export_path, second_export_path] = arguments.as_slice() else {
        eprintln!("用法：e172-deletion-write-dry-run <H1H2 导出文件> <H3 导出文件>，收到 {arguments:?}");
        std::process::exit(2);
    };
    let read_export = |export_path: &String| match std::fs::read_to_string(export_path) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("读不了导出文件 {export_path}：{error}");
            std::process::exit(2);
        }
    };
    let first_export_text = read_export(first_export_path);
    let second_export_text = read_export(second_export_path);
    let mut emitter = Emitter::new();
    for body in report_lines(&[first_export_text.as_str(), second_export_text.as_str()]) {
        println!("{}", emitter.emit_raw(&body));
    }
    println!("{}", emitter.finish());
}

/// A-容量：layout/03 第 16 行 ⌊(32768 − 136) ÷ 40⌋、第 11 行 ⌊(16384 − 131) ÷ 8⌋、第 17 行 8 + 26 + 86，按头宽现算。
fn tombstone_container_capacity() -> usize {
    (DATA_UNIT_BYTES - PACKED_UNIT_RECORDS_OFFSET) / TOMBSTONE_RECORD_BYTES
}

fn deleted_inodes_root_capacity() -> usize {
    (NODE_BYTES - index_node_header_bytes(8) - RESERVED_NONCE_MAC_ALGORITHM_BYTES) / DELETED_INODES_ENTRY_BYTES
}

fn tombstone_root_entry_bytes() -> usize {
    8 + 26 + NODE_POINTER_BYTES
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::OnceLock;

    /// 入库的两份导出（跑前登记 e172-r2 第九节：单测读它们，找不到就判红，不跳过）。
    const H1H2_EXPORT_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../research/results/e172-r2-deletion-write-dump-h1h2-2026-10-05.out");
    const H3_EXPORT_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../research/results/e172-r2-deletion-write-dump-h3-2026-10-05.out");

    struct ExportFixture {
        export_files: Vec<ExportFile>,
        histories: Vec<HistoryExport>,
        judgements: Vec<HistoryJudgement>,
        lines: Vec<String>,
    }

    fn fixture() -> &'static ExportFixture {
        static FIXTURE: OnceLock<ExportFixture> = OnceLock::new();
        FIXTURE.get_or_init(|| {
            let first_text = std::fs::read_to_string(H1H2_EXPORT_PATH).expect("入库的 H1H2 导出必须在（找不到就判红）");
            let second_text = std::fs::read_to_string(H3_EXPORT_PATH).expect("入库的 H3 导出必须在（找不到就判红）");
            let texts = [first_text.as_str(), second_text.as_str()];
            let export_files = parse_export_files(&texts);
            let histories: Vec<HistoryExport> = export_files.iter().flat_map(|export| export.histories.iter().cloned()).collect();
            let judgements = histories.iter().map(judge_history).collect();
            let lines = report_lines(&texts);
            ExportFixture { export_files, histories, judgements, lines }
        })
    }

    fn judgement_named(name: &str) -> &'static HistoryJudgement {
        fixture().judgements.iter().find(|judgement| judgement.name == name).expect("三段历史都在")
    }

    fn history_named(name: &str) -> &'static HistoryExport {
        fixture().histories.iter().find(|history| history.name == name).expect("三段历史都在")
    }

    fn cell_row(history: &str, txg: u64, cell: &str, field: &str) -> &'static CellRow {
        judgement_named(history)
            .cell_rows
            .iter()
            .find(|row| row.txg == txg && row.cell == cell && row.field == field)
            .unwrap_or_else(|| panic!("{history} txg {txg} 没有 {cell} {field} 这一行"))
    }

    fn line_starting_with(prefix: &str) -> &'static String {
        fixture().lines.iter().find(|line| line.starts_with(prefix)).unwrap_or_else(|| panic!("产物里没有以 {prefix} 起头的行"))
    }

    #[test]
    fn castagnoli_crc32_of_the_standard_check_string_is_e3069283() {
        assert_eq!(castagnoli_crc32(b"123456789"), 0xE306_9283);
    }

    #[test]
    fn sha256_of_abc_matches_the_standard_vector() {
        assert_eq!(sha256_hexadecimal(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    }

    #[test]
    fn registered_capacities_compute_to_815_2031_and_120() {
        assert_eq!(tombstone_container_capacity(), 815);
        assert_eq!(deleted_inodes_root_capacity(), 2031);
        assert_eq!(tombstone_root_entry_bytes(), 120);
    }

    #[test]
    fn committed_reservation_formula_gives_98304_without_live_inodes_and_98386_with_one() {
        assert_eq!(expected_committed_reservation(0, 0), 98_304);
        assert_eq!(expected_committed_reservation(0, 1), 98_386);
    }

    #[test]
    fn expected_file_bytes_are_two_full_payloads_and_a_half() {
        assert_eq!(EXPECTED_FILE_BYTES, 81_585);
        assert_eq!(DATA_UNIT_PAYLOAD_CAPACITY, 32_634);
    }

    #[test]
    fn root_slot_journal_and_system_configuration_anchors_follow_the_layout_formulas() {
        assert_eq!(expected_root_slot(5), (0, 7_344_128));
        assert_eq!(expected_root_slot(6), (0, 1_056_768));
        assert_eq!(expected_root_slot(7), (1, 4_202_496));
        assert_eq!(expected_root_slot(13), (1, 4_210_688));
        assert_eq!(expected_journal_offset_assuming_counter_equals_txg(5), 16_793_600);
        assert_eq!(expected_journal_offset_assuming_counter_equals_txg(8), 16_805_888);
        assert_eq!(expected_system_configuration_for_txg_in_the_first_instance(5), (7, 4096));
        assert_eq!(expected_system_configuration_for_txg_in_the_first_instance(6), (8, 0));
    }

    #[test]
    fn layout_field_tables_tile_their_registered_totals() {
        let root_record_widths = [4usize, 16, 4, 4, 8, 86, 8, 8, 32, 86, 86, 86, 1, 12, 16];
        assert_eq!(root_record_widths.iter().sum::<usize>(), ROOT_RECORD_FIELD_BYTES);
        let index_node_header_widths_without_key_range = [4usize, 2, 1, 1, 2, 32, 8, 1, 1, 8, 8, 4, 4, 4, 2, 2, 2];
        assert_eq!(index_node_header_widths_without_key_range.iter().sum::<usize>(), INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE);
        let packed_header_widths = [4usize, 2, 1, 1, 2, 32, 1, 8, 2, 8, 8, 2, 2, 8, 8, 4, 10, 4];
        assert_eq!(packed_header_widths.iter().sum::<usize>(), PACKED_UNIT_PLAINTEXT_HEADER_BYTES);
        assert_eq!(index_node_header_bytes(8) + RESERVED_NONCE_MAC_ALGORITHM_BYTES, 131);
    }

    #[test]
    fn key_range_reading_tells_entry_key_range_from_subtree_coverage() {
        assert_eq!(key_range_reading((1, 1), (1, 1), &[(0, u64::MAX)]), KeyRangeReading::EntryKeyRange);
        assert_eq!(key_range_reading((0, u64::MAX), (1, 1), &[(0, u64::MAX)]), KeyRangeReading::SubtreeCoverage);
        assert_eq!(key_range_reading((0, u64::MAX), (0, u64::MAX), &[(0, u64::MAX)]), KeyRangeReading::BothReadingsCoincide);
        assert_eq!(key_range_reading((2, 3), (1, 1), &[(0, u64::MAX)]), KeyRangeReading::Neither);
    }

    #[test]
    fn cutting_cuts_at_force_unit_access_and_counts_a_barrier_step_per_device() {
        let view = |region_kind: RegionKind, is_barrier: bool, is_force_unit_access: bool| OperationView { is_barrier, region_kind, is_force_unit_access };
        // 一个单元两块盘、一道屏障两步、journal 两块盘、一道屏障、根槽 FUA、系统配置两块盘、一道屏障。
        let operations = vec![
            view(RegionKind::Unit, false, false),
            view(RegionKind::Unit, false, false),
            view(RegionKind::Unrecognized, true, false),
            view(RegionKind::Unrecognized, true, false),
            view(RegionKind::JournalRecord, false, false),
            view(RegionKind::JournalRecord, false, false),
            view(RegionKind::Unrecognized, true, false),
            view(RegionKind::Unrecognized, true, false),
            view(RegionKind::RootRecordSlot, false, true),
            view(RegionKind::SystemConfigurationSlot, false, false),
            view(RegionKind::SystemConfigurationSlot, false, false),
            view(RegionKind::Unrecognized, true, false),
            view(RegionKind::Unrecognized, true, false),
        ];
        let pieces = cut_into_pieces(&operations);
        assert_eq!(segment_count_text(&pieces), expected_count_text(1));
        assert_eq!(segment_kind_text(&pieces), expected_kind_text(1));
    }

    #[test]
    fn coverage_counts_a_nonzero_byte_in_constant_zero_padding() {
        let mut map = CoverageMap::new(8);
        map.claim(0, 4, ByteClass::LayoutOneCell);
        map.claim(4, 8, ByteClass::ConstantZeroPadding);
        let region = DecodedRegion {
            step: 0,
            device: 0,
            offset: 0,
            bytes: vec![1, 2, 3, 4, 0, 0, 7, 0],
            region_kind: RegionKind::Unrecognized,
            body: DecodedBody::Unrecognized,
            structure_failures: Vec::new(),
            coverage: map,
            locations_inside: Vec::new(),
        };
        let segment = SegmentAnalysis {
            start_step: 0,
            end_step: 1,
            label: SegmentLabel::Outside { segment_name: "synthetic".to_string() },
            is_deletion_publish: false,
            operations: Vec::new(),
            regions: vec![region],
            units: Vec::new(),
            root_records: Vec::new(),
            journal_records: Vec::new(),
            system_configurations: Vec::new(),
            structure_failures: Vec::new(),
            unreferenced_units: Vec::new(),
        };
        let analysis = HistoryAnalysis { name: "synthetic".to_string(), segments: vec![segment], states_before: Vec::new(), states_after: Vec::new() };
        let totals = coverage_of(&analysis);
        assert_eq!(totals.constant_zero_nonzero_bytes, 1);
        assert_eq!(totals.constant_zero_bytes, 4);
        assert_eq!(totals.layout_one_bytes, 4);
        assert_eq!(totals.unregistered_bytes, 0);
    }

    #[test]
    fn export_lines_are_all_anchored_and_every_history_carries_the_registered_parameters() {
        let export_files = &fixture().export_files;
        assert_eq!(export_files.len(), 2);
        for (export, expected_histories) in export_files.iter().zip(HISTORIES_OF_EACH_EXPORT_FILE) {
            assert_eq!(export.line_count, export.anchored_line_count);
            assert_eq!(export.done_emitted, Some(export.line_count));
            assert!(export.malformed_lines.is_empty(), "{:?}", export.malformed_lines);
            assert_eq!(export.histories.iter().map(|history| history.name.as_str()).collect::<Vec<_>>(), expected_histories);
        }
        assert_eq!(HISTORIES_OF_EACH_EXPORT_FILE, [&["h1", "h2"][..], &["h3"][..]]);
        for history in &fixture().histories {
            assert!(configuration_matches_registration(history), "{}", history.name);
            assert_eq!(history.configuration.get("file_bytes").map(String::as_str), Some("81585"));
        }
        let h3 = history_named("h3");
        let sequential = field_number(&h3.configuration, "sequential_write_time_seconds").expect("H3 带顺序写的时间");
        assert_eq!(field_number(&h3.configuration, "intent_write_time_seconds"), Some(sequential + 60));
        assert!(!history_named("h1").configuration.contains_key("intent_write_time_seconds"));
        assert!(!history_named("h2").configuration.contains_key("intent_write_time_seconds"));
    }

    #[test]
    fn decoded_structure_passes_every_stop_check_on_the_export() {
        for judgement in &fixture().judgements {
            assert!(judgement.structure_failures.is_empty(), "{:?}", judgement.structure_failures);
            assert!(judgement.role_mismatches.is_empty(), "{:?}", judgement.role_mismatches);
            assert!(judgement.tiling_failures.is_empty(), "{:?}", judgement.tiling_failures);
        }
    }

    #[test]
    fn intent_record_cells_read_the_registered_offsets() {
        assert_eq!(cell_row("h1", 5, "L03-1", "nlink").actual, "0");
        assert_eq!(cell_row("h1", 5, "L03-2", "size").actual, "81585");
        assert_eq!(cell_row("h1", 5, "L03-3", "blocks").expected, "160");
        assert_eq!(cell_row("h1", 5, "L03-3", "blocks").actual, "160");
        assert_eq!(cell_row("h1", 5, "L03-6", "mode").actual, "33188");
        assert_eq!(cell_row("h1", 5, "L03-5", "times").actual, "1788000060/1788000060/1788000060");
        assert!(!cell_row("h1", 5, "L03-5", "times").discriminable);
        let h3_intent_seconds = history_named("h3").configuration.get("intent_write_time_seconds").expect("H3 带意图的调用方时间").clone();
        assert_eq!(cell_row("h3", 6, "L03-5", "times").expected, three_equal_times_text(&h3_intent_seconds));
        assert!(cell_row("h3", 6, "L03-5", "times").discriminable);
    }

    #[test]
    fn batch_sizes_are_remaining_units_times_32634_and_blocks_round_up() {
        assert_eq!(cell_row("h1", 6, "L03-2", "size").expected, "65268");
        assert_eq!(cell_row("h1", 6, "L03-3", "blocks").expected, "128");
        assert_eq!(cell_row("h1", 7, "L03-2", "size").expected, "32634");
        assert_eq!(cell_row("h1", 7, "L03-3", "blocks").expected, "64");
    }

    #[test]
    fn change_counts_equal_the_publish_txg_in_every_deletion_version() {
        for (history, txg) in [("h1", 5), ("h1", 6), ("h1", 7), ("h2", 5), ("h2", 11), ("h2", 12)] {
            let row = cell_row(history, txg, "L03-4", "change_count");
            assert_eq!(row.expected, txg.to_string());
            assert_eq!(row.actual, txg.to_string());
        }
        // H3 只钉期望一栏（新加的单测不把新导出的读数写成断言，跑前登记 e172-r2 第五节 5.8）。
        for txg in [6u64, 7, 8] {
            assert_eq!(cell_row("h3", txg, "L03-4", "change_count").expected, txg.to_string());
        }
    }

    #[test]
    fn intent_tombstone_record_and_queue_root_match_the_registered_values() {
        assert_eq!(cell_row("h1", 5, "L03-9", "range_end").expected, u64::MAX.to_string());
        assert!(cell_row("h1", 5, "L03-9", "range_end").is_equal());
        assert_eq!(cell_row("h1", 5, "L03-9", "death_txg").expected, "5");
        assert!(cell_row("h1", 5, "L03-9", "death_txg").is_equal());
        assert!(cell_row("h1", 5, "L03-8", "deleted_inodes_root").expected.contains("tree65544"));
        assert!(cell_row("h1", 5, "L03-8", "deleted_inodes_root").is_equal());
        assert!(cell_row("h1", 5, "L03-10", "tombstone_container").is_equal());
        assert!(cell_row("h1", 5, "L03-11", "tombstone_root").is_equal());
    }

    #[test]
    fn final_batch_removes_the_record_and_leaves_an_empty_inode_root() {
        for field in ["deleted_record_in_written_containers", "inode_root_entry_count", "inode_root_key_range", "inode_leaf_containers_written"] {
            let row = cell_row("h1", 8, "L03-7", field);
            assert!(row.is_equal(), "{row:?}");
        }
        assert_eq!(cell_row("h1", 8, "L03-7", "inode_root_key_range").expected, "0/0");
    }

    #[test]
    fn committed_reservation_rows_read_98304_in_every_h1_deletion_version_and_expect_98386_in_every_h3_one() {
        for txg in [5u64, 6, 7, 8] {
            let row = judgement_named("h1").cell_rows.iter().find(|row| row.txg == txg && row.cell == "L03-13").expect("每一版一行");
            assert_eq!(row.expected, "98304");
            assert!(row.is_equal(), "{row:?}");
        }
        for txg in [6u64, 7, 8, 9] {
            let row = judgement_named("h3").cell_rows.iter().find(|row| row.txg == txg && row.cell == "L03-13").expect("每一版一行");
            assert_eq!(row.expected, "98386");
        }
    }

    #[test]
    fn h1_intent_segment_string_kind_string_and_roles_are_decoded_independently() {
        let row = judgement_named("h1").segment_rows.iter().find(|row| row.publish_kind == "unlink_intent").expect("意图那一行");
        assert_eq!(row.actual_count_text, "26+2+1+2");
        assert_eq!(
            row.actual_kind_text,
            "[unit_write×26,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]"
        );
        assert_eq!(row.actual_roles, "13:t3,t4,t5,t5@0.0.61,t5@0.1.61,t5@1.0.0,t5@1.1.0,t6,t7,t8,td,tt,tt@0");
        assert!(row.roles_equal());
        assert_eq!(row.actual_root_location, "0:7344128");
    }

    /// R5.a 是附带的（跑前登记 e172-r2 第六节 6.3，不进任何判定）；这一条照第 1 次留作回归。
    #[test]
    fn final_batch_drops_exactly_the_inode_leaf_container_against_the_two_to_one_batch() {
        assert!(line_starting_with("name=r5a ").contains("only_in_two_to_one=t3 only_in_final=none matches_registered=true"));
    }

    #[test]
    fn queue_and_tombstone_roots_record_an_entry_key_range_reading() {
        let readings: Vec<&String> = fixture().lines.iter().filter(|line| line.starts_with("name=r7 history=h1")).collect();
        assert_eq!(readings.len(), 2);
        assert!(readings[0].contains("role=td level=0 key_range=1..1 entry_keys=1..1 reading=entry_key_range"), "{}", readings[0]);
        assert!(readings[1].contains("role=tt level=1 key_range=1..1 entry_keys=1..1 reading=entry_key_range"), "{}", readings[1]);
    }

    #[test]
    fn mount_writes_are_attributed_to_the_same_segments_the_export_cuts() {
        let h2 = judgement_named("h2");
        assert!(!h2.attribution_rows.is_empty());
        assert_eq!(h2.misattributed_writes(), 0, "{:?}", h2.attribution_rows.iter().filter(|row| !row.is_match).collect::<Vec<_>>());
        let acquisition_writes = h2.attribution_rows.iter().filter(|row| row.decoded == WriteAttribution::InstanceAcquisition).count();
        assert_eq!(acquisition_writes, 2);
    }

    /// 每一条对照在它适用的每一段历史上都先绿后红（跑前登记 e172-r2 第五节 5.4：适用的对共 18 个，不再有例外）；
    /// 不适用的只出现在表里写了不适用的那几对上。
    #[test]
    fn every_positive_control_turns_its_target_red_on_every_history_it_applies_to() {
        let mut held = 0usize;
        let mut not_applicable = Vec::new();
        for (history, judgement) in fixture().histories.iter().zip(&fixture().judgements) {
            for control in PositiveControl::EVERY_CONTROL {
                let (line, holds) = positive_control_line(history, judgement, control);
                match holds {
                    Some(holds) => {
                        assert!(holds, "{line}");
                        held += 1;
                    }
                    None => not_applicable.push(format!("{}@{}", control.name(), history.name)),
                }
            }
        }
        assert_eq!(held, 18);
        assert_eq!(held, PositiveControl::APPLICABLE_PAIRS_EXPECTED);
        assert_eq!(not_applicable, ["PC-F@h1", "PC-G@h1", "PC-I@h1", "PC-I@h2", "PC-F@h3", "PC-G@h3"]);
    }

    /// PC-C 只重封了墓碑容器自己的载荷校验和：父指针（墓碑树根条目、映射、journal 点名项）里那 4 字节整单元校验和照样对不上，S2 那一格要红。
    #[test]
    fn a_resealed_container_still_fails_the_parent_location_checksum() {
        let history = history_named("h1");
        let corrupted = corrupted_copy(history, PositiveControl::BumpTombstoneDeathTxg).expect("改得到墓碑容器");
        let judgement = judge_history(&corrupted);
        let parent_failures = judgement.structure_failures.iter().filter(|failure| failure.contains("父指针位置条目里的整单元校验和")).count();
        assert!(parent_failures >= 2, "{:?}", judgement.structure_failures);
        assert!(judgement.structure_failures.iter().all(|failure| !failure.contains("码 3 载荷校验和不对")));
    }

    #[test]
    fn h1_window_string_is_the_four_registered_strings_end_to_end() {
        assert!(line_starting_with("name=r3d history=h1 ").ends_with("equal=true"), "{}", line_starting_with("name=r3d history=h1 "));
        // H3 那一行只钉期望一栏：13、12、11、11 四串首尾相接。
        assert!(
            line_starting_with("name=r3d history=h3 ").starts_with("name=r3d history=h3 expected=26+2+1+2+24+2+1+2+22+2+1+2+22+2+1+2 actual="),
            "{}",
            line_starting_with("name=r3d history=h3 ")
        );
    }

    // ---- 跑前登记 e172-r2 第五节 5.8 新加的 T1–T13 ----

    /// T1：合成系统配置槽，偏移 433 写 5000、437 起写 T_dirty（2 GiB），解出 5000。
    #[test]
    fn time_threshold_is_read_from_offset_433_of_a_synthetic_system_configuration_slot() {
        let mut slot = vec![0u8; 4096];
        slot[0..4].copy_from_slice(b"SFSB");
        slot[433..437].copy_from_slice(&5000u32.to_le_bytes());
        slot[437..445].copy_from_slice(&(2u64 << 30).to_le_bytes());
        reseal_system_configuration_slot(&mut slot);
        let mut coverage = CoverageMap::new(slot.len());
        let mut failures = Vec::new();
        let configuration = decode_system_configuration(&slot, &mut coverage, &mut failures);
        assert!(failures.is_empty(), "{failures:?}");
        assert_eq!(configuration.time_threshold_milliseconds, 5000);
    }

    /// T2：每段历史 R1.t 的行数 = 窗口里系统配置槽写次数（从导出的写区域现数），期望一栏都是 5000。
    #[test]
    fn time_threshold_rows_cover_every_system_configuration_slot_write_and_expect_5000() {
        for history in &fixture().histories {
            let slot_writes = history
                .operations
                .iter()
                .filter(|operation| matches!(operation, WindowOperation::Write(write) if (write.offset == 0 || write.offset == 4096) && write.bytes.len() == 4096))
                .count();
            let prefix = format!("name=r1t history={} ", history.name);
            let rows: Vec<&String> = fixture().lines.iter().filter(|line| line.starts_with(&prefix)).collect();
            assert!(slot_writes > 0, "{}", history.name);
            assert_eq!(rows.len(), slot_writes, "{}", history.name);
            assert!(rows.iter().all(|line| line.contains(" expected=5000 ")), "{rows:?}");
        }
    }

    /// T3：H3 选到 L03-7 第二支、只有 H3 选到；第二支的期望一栏（写出的叶 1 片、根条目数 1）。
    #[test]
    fn only_h3_takes_the_second_l03_7_branch_and_expects_one_leaf_and_one_root_entry() {
        assert!(line_starting_with("name=l03_7_branch_check ").contains("second_branch_histories=h3 exactly_h3=true"));
        assert_eq!(cell_row("h3", 9, "L03-7", "inode_root_entry_count").expected, "1");
        assert_eq!(cell_row("h3", 9, "L03-7", "inode_leaf_containers_written").expected, "1");
        assert_eq!(cell_row("h3", 9, "L03-7", "deleted_record_in_written_containers").expected, "0");
        assert_eq!(cell_row("h1", 8, "L03-7", "inode_root_entry_count").expected, "0");
        assert_eq!(cell_row("h1", 8, "L03-7", "inode_leaf_containers_written").expected, "0");
    }

    /// T4：H3 意图 L03-5 的期望 = 导出的 `intent_write_time_seconds=`（与顺序写的时间不同），可分辨。
    #[test]
    fn h3_intent_times_expect_the_callers_time_and_are_discriminable() {
        let configuration = &history_named("h3").configuration;
        let row = cell_row("h3", 6, "L03-5", "times");
        assert_eq!(row.expected, three_equal_times_text(&configuration["intent_write_time_seconds"]));
        assert_ne!(row.expected, three_equal_times_text(&configuration["sequential_write_time_seconds"]));
        assert!(row.discriminable);
    }

    /// T5：合成输入——批的 L03-5 期望取上一版那条记录的三格，上一版三格与 `sequential_write_time_seconds` 不同。
    #[test]
    fn batch_times_expect_the_previous_records_three_values_not_the_sequential_write_time() {
        let mut record = vec![0u8; INODE_RECORD_BYTES];
        for offset in [64usize, 72, 80] {
            record[offset..offset + 8].copy_from_slice(&1_788_000_120u64.to_le_bytes());
        }
        let configuration = BTreeMap::from([("sequential_write_time_seconds".to_string(), "1788000060".to_string())]);
        assert_eq!(expected_times_of(false, &configuration, Some(&record)), ("1788000120/1788000120/1788000120".to_string(), true));
        assert_eq!(expected_times_of(true, &configuration, Some(&record)), ("1788000060/1788000060/1788000060".to_string(), false));
    }

    /// T6：R5.b 行在；判别力自证行里两套期望（H1 的、H3 的）不同时判绿。
    #[test]
    fn row_five_h3_line_is_reported_and_its_two_expectations_are_never_both_green() {
        assert!(line_starting_with("name=r5b history=h3 ").contains(" final_writes_t3="));
        let discrimination = line_starting_with("name=r5b_discrimination history=h3 ");
        assert!(!discrimination.contains("h1_expectation_green=true h3_expectation_green=true"), "{discrimination}");
        assert!(line_starting_with("name=r3_geometry history=h3 ").contains(" expectations_differ=true "));
    }

    /// T7：H3 的 A-tag6 期望一栏是 98386（按历史名分的字面常量）。
    #[test]
    fn h3_committed_reservation_anchor_expects_98386() {
        assert_eq!(registered_committed_reservation_of_history("h3"), Some(98_386));
        assert_eq!(registered_committed_reservation_of_history("h1"), Some(98_304));
        assert_eq!(registered_committed_reservation_of_history("h2"), Some(98_304));
        for txg in [6u64, 7, 8, 9] {
            assert!(line_starting_with(&format!("name=anchor anchor=A-tag6 history=h3 txg={txg} ")).contains(" expected=98386 "));
        }
    }

    /// T8：PC-G 的副本恰有一次写换了段，换的是挂载写行之前那一截的最后一次写。
    #[test]
    fn pc_g_moves_exactly_the_last_write_of_the_mount_segment_into_the_next_segment() {
        let history = history_named("h2");
        let corrupted = corrupted_copy(history, PositiveControl::ShiftMountSegmentEndToItsLastWrite).expect("H2 有挂载写行之前那一段");
        let label_at = |segments: &[ExportSegment], step: usize| segments.iter().find(|segment| step >= segment.start_step && step < segment.end_step).map(|segment| segment.label.clone());
        let write_steps: Vec<usize> = history
            .operations
            .iter()
            .filter_map(|operation| match operation {
                WindowOperation::Write(write) => Some(write.step),
                WindowOperation::Barrier { .. } => None,
            })
            .collect();
        let moved: Vec<usize> = write_steps.iter().copied().filter(|step| label_at(&history.segments, *step) != label_at(&corrupted.segments, *step)).collect();
        let mount = history.segments.iter().find(|segment| segment.publish_kind() == "mount_writes_before_its_first_publish").expect("有那一段");
        let last_write_in_mount = write_steps.iter().copied().filter(|step| *step >= mount.start_step && *step < mount.end_step).max().expect("那一段有写");
        assert_eq!(moved, [last_write_in_mount]);
    }

    /// T9：PC-H 的副本恰改了一次系统配置槽写，那次解出 5，整槽校验和过。
    #[test]
    fn pc_h_rewrites_exactly_one_system_configuration_write_to_5_and_reseals_its_slot() {
        for history in &fixture().histories {
            let corrupted = corrupted_copy(history, PositiveControl::RewriteIntentTimeThresholdAsSeconds).expect("每段历史的意图都有系统配置槽写");
            let changed: Vec<&RegionWrite> = history
                .operations
                .iter()
                .zip(&corrupted.operations)
                .filter_map(|(original, copy)| match (original, copy) {
                    (WindowOperation::Write(original_write), WindowOperation::Write(copy_write)) if original_write.bytes != copy_write.bytes => Some(copy_write),
                    (WindowOperation::Write(_) | WindowOperation::Barrier { .. }, WindowOperation::Write(_) | WindowOperation::Barrier { .. }) => None,
                })
                .collect();
            assert_eq!(changed.len(), 1, "{}", history.name);
            assert_eq!(region_kind_of(changed[0]), RegionKind::SystemConfigurationSlot);
            assert_eq!(read_u32(&changed[0].bytes, 433), 5);
            assert!(whole_slot_checksum_holds(&changed[0].bytes, 155), "{}", history.name);
        }
    }

    /// T10：PC-I 的副本里 H3 意图 1 号记录三格 = 顺序写的时间，码 3 载荷校验和过。
    #[test]
    fn pc_i_copies_the_sequential_write_time_into_the_h3_intent_record_and_reseals_the_container() {
        let history = history_named("h3");
        let sequential = field_number(&history.configuration, "sequential_write_time_seconds").expect("H3 带顺序写的时间");
        let corrupted = corrupted_copy(history, PositiveControl::CopySequentialWriteTimeIntoTheIntentRecord).expect("H3 意图写了 t3");
        let judgement = judge_history(&corrupted);
        let intent = judgement.analysis.segments.iter().find(|segment| segment.label_kind() == "unlink_intent").expect("有意图那一段");
        let record = record_of_inode(&records_written_in(intent, INODE_TREE, 2), DELETED_INODE).expect("意图写了 1 号记录");
        for offset in [64usize, 72, 80] {
            assert_eq!(read_u64(&record, offset), sequential);
        }
        assert!(intent.structure_failures.iter().all(|failure| !failure.contains("码 3 载荷校验和不对")), "{:?}", intent.structure_failures);
    }

    fn synthetic_configuration(instance: u64, journal_tail: u64) -> SystemConfiguration {
        SystemConfiguration { generation: 0, journal_tail, instance, time_threshold_milliseconds: 5000 }
    }

    /// T11：同实例、tail 等于别的实例某次发布的计数器：轮换要实例代号相同，R8.c 归「别的挂载步」。
    #[test]
    fn rotation_needs_the_same_instance_so_a_tail_matching_another_instances_publish_is_another_mount_step() {
        let attribution = attribute_system_configuration(10, &synthetic_configuration(2, 7), Some(&synthetic_configuration(2, 6)), &[(7, 7, 1, 3)]);
        assert_eq!(attribution, WriteAttribution::OtherMountStep);
    }

    /// T12：实例换了、tail 没动，tail 恰等于新实例某次在前的发布计数器：先判取号，R8.c 归「取号」。
    #[test]
    fn acquisition_is_judged_before_rotation_even_when_the_tail_matches_an_earlier_publish_of_the_new_instance() {
        let attribution = attribute_system_configuration(10, &synthetic_configuration(2, 7), Some(&synthetic_configuration(1, 7)), &[(9, 7, 2, 3)]);
        assert_eq!(attribution, WriteAttribution::InstanceAcquisition);
    }

    /// T13：H3 的 A-txg、A-根槽、A-journal、A-系统配置期望一栏等于跑前登记 e172-r2 第七节 7.1 的 H3 值。
    #[test]
    fn h3_txg_root_slot_journal_and_system_configuration_anchors_expect_the_registered_h3_values() {
        assert!(line_starting_with("name=anchor anchor=A-txg history=h3 deletion_publishes ").contains(" expected=6,7,8,9 "));
        for (txg, root, journal, generation, slot) in [
            (6u64, "0:1056768", 16_797_696u64, 8u64, 0u64),
            (7, "1:4202496", 16_801_792, 9, 4096),
            (8, "0:7348224", 16_805_888, 10, 0),
            (9, "0:1060864", 16_809_984, 11, 4096),
        ] {
            assert!(line_starting_with(&format!("name=anchor anchor=A-root-slot history=h3 txg={txg} ")).contains(&format!(" expected={root} ")));
            assert!(line_starting_with(&format!("name=anchor anchor=A-journal history=h3 txg={txg} ")).contains(&format!(" expected=0:{journal},1:{journal} ")));
            assert!(
                line_starting_with(&format!("name=anchor anchor=A-system-configuration history=h3 txg={txg} "))
                    .contains(&format!(" expected=0:{slot}:generation{generation},1:{slot}:generation{generation} "))
            );
        }
    }
}
