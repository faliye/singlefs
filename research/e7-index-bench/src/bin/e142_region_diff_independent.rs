//! E142 第十六次跑第一段（重跑登记第五节 P3）：独立比对二进制。
//!
//! 不 `use` 模型 `e142_first_transaction_dry_run.rs` 的任何一项——两个 bin 本来互相引不到，这里连着手写
//! 一份自己的行解析、十六进制解码、CRC-32C 与连续不同字节切分，好让「模型自己算出的 equal / 不同字节段 /
//! 校验和重算」有一条独立的路重算一遍核对，不是同一段代码在骗自己。
//!
//! 两种调用（E142 第十八次跑重跑登记第五节 5.1 ③、第六节 Q142.39/Q142.41/Q142.42/Q142.43）：
//!
//!   e142-region-diff-independent <主产物路径> <crates 导出路径>
//!       从两份文本里各自抓 `name=device_region_bytes ... device=... offset=... length=... hexadecimal=...`
//!       这一族行，按 (设备, 偏移, 长度) 配对（不按 `region=` 配对——R3 的教训：两块设备在同一偏移各写不同
//!       内容时按名字配对分不清）。第一个参数按 R26「模型那一侧」的取法：只取文件里第一行 `name=done` 之前的，
//!       没有 `name=done` 就报错退出；第二个参数（crates 导出）整份取。
//!
//!   e142-region-diff-independent old-new <新产物路径> <旧产物路径>
//!       两个参数都按「模型那一侧」的取法（R26）取，按 (设备, 偏移, 长度) 配对，找出改前改后不同的区域与段，
//!       按 R28 的字段表把每个不同字节归到 clause_d15_4 / clause_d22_9 / clause_d22_7 / derived / unmapped
//!       一类，重算受影响区域的校验和字段（D18 已定项 17），判每个变了的区域「对不对得到」。

use std::collections::BTreeMap;
use std::env;
use std::fs;

#[derive(Clone, Debug, PartialEq, Eq)]
struct ParsedRegionLine {
    region: Option<String>,
    device: u32,
    offset: u64,
    length: u64,
    hexadecimal: Vec<u8>,
}

fn parse_result_line(line: &str) -> BTreeMap<String, String> {
    let mut fields = BTreeMap::new();
    for token in line.split_whitespace() {
        if let Some((key, value)) = token.split_once('=') {
            fields.insert(key.to_string(), value.to_string());
        }
    }
    fields
}

fn hex_decode(hexadecimal: &str) -> Vec<u8> {
    (0..hexadecimal.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&hexadecimal[index..index + 2], 16).expect("十六进制字符成对出现"))
        .collect()
}

fn parse_device_region_bytes_lines(text: &str) -> Vec<ParsedRegionLine> {
    text.lines()
        .filter(|line| line.contains("name=device_region_bytes "))
        .filter_map(|line| {
            let fields = parse_result_line(line);
            let hexadecimal = fields.get("hexadecimal")?;
            Some(ParsedRegionLine {
                region: fields.get("region").cloned(),
                device: fields.get("device")?.parse().ok()?,
                offset: fields.get("offset")?.parse().ok()?,
                length: fields.get("length")?.parse().ok()?,
                hexadecimal: hex_decode(hexadecimal),
            })
        })
        .collect()
}

/// R26「模型那一侧」的取法：只取文件里第一行 `name=done`（字段值恰为 `done`）之前的
/// `name=device_region_bytes` 行；没有 `name=done` 就报错退出，不许默默整份取（M8 的抓手）。
fn parse_model_side_device_region_bytes_lines(text: &str, source_label: &str) -> Vec<ParsedRegionLine> {
    let done_line_index = text
        .lines()
        .position(|line| parse_result_line(line).get("name").map(String::as_str) == Some("done"))
        .unwrap_or_else(|| panic!("{source_label} 里没有 name=done 这一行；模型那一侧的取法（R26）要求先找到它，不许默默整份取"));
    let prefix_lines: Vec<&str> = text.lines().take(done_line_index).collect();
    parse_device_region_bytes_lines(&prefix_lines.join("\n"))
}

/// 把两段字节里不同的位置切成最长的连续段，返回每段 (区域内偏移, 长度)；长度不等时缺的那一半按「不同」算。
///
/// 外层循环每一轮都必须让 `position` 前进至少一格；这是它的正确性不变量，不是为了抓变异专门加的检查
/// （每一轮要么在 else 分支里自己 +1，要么先由外层 if 确认了 position 处不同、再由内层 while 至少跑一轮把
/// position 推过去）。`research/prompts/oom-2026-09-25/report.md` 记的两次整机 OOM 就是这个位置的
/// 外层判断被变异成 `==` 之后，卡在同一个 position 上无限推长度 0 的段：外层循环最多跑 `longer_length` 轮，
/// 加一道次数上界断言，让这种破坏在几轮之内变成一次 panic，不再无界地往 `segments` 里塞东西
/// （E142 第十六次跑第一段，`research/mutations/e142_region_diff_independent.tsv` 的
/// M2_diff_segments_equality_flipped 那一条）。
fn diff_segments(left: &[u8], right: &[u8]) -> Vec<(usize, usize)> {
    let mut segments = Vec::new();
    let longer_length = left.len().max(right.len());
    let mut position = 0;
    let mut outer_loop_iterations: usize = 0;
    while position < longer_length {
        outer_loop_iterations += 1;
        assert!(
            outer_loop_iterations <= longer_length,
            "diff_segments 外层循环第 {outer_loop_iterations} 轮仍未跑完（两段共 {longer_length} 字节的上界）：\
             每一轮必须让 position 前进至少一格，卡在同一个位置就是死循环"
        );
        if left.get(position) != right.get(position) {
            let segment_start = position;
            while position < longer_length && left.get(position) != right.get(position) {
                position += 1;
            }
            segments.push((segment_start, position - segment_start));
        } else {
            position += 1;
        }
    }
    segments
}

#[cfg(test)]
fn find_matching_line(haystack: &[ParsedRegionLine], device: u32, offset: u64, length: u64) -> Option<&ParsedRegionLine> {
    haystack.iter().find(|line| line.device == device && line.offset == offset && line.length == length)
}

// ───────────────────────── D18（块里携带什么信息） 已定项 17：CRC-32C（自己写一份，不 use 模型） ─────────────────────────

/// CRC-32C（Castagnoli 多项式 0x1EDC6F41，反射实现，初值 0xFFFFFFFF，输出取反）；`0x82F6_3B78` 是
/// 0x1EDC6F41 按位反转后的多项式，配反射实现用。标准测试向量 `"123456789"` → `0xE3069283`（单测钉住）。
fn crc32c_castagnoli(bytes: &[u8]) -> u32 {
    let mut remainder: u32 = 0xFFFF_FFFF;
    for &byte in bytes {
        remainder ^= u32::from(byte);
        for _ in 0..8 {
            remainder = if remainder & 1 != 0 { (remainder >> 1) ^ 0x82F6_3B78 } else { remainder >> 1 };
        }
    }
    !remainder
}

/// 「校验和字段自身按 0 参与」：把 `[field_offset, field_offset + 32)` 清零后对 `[0, cover_end)` 求校验和，
/// 结果小端写进返回值的前 4 字节，其余 28 字节恒 0（D18 已定项 17）。
fn wide_checksum_field(bytes: &[u8], cover_end: usize, field_offset: usize) -> [u8; 32] {
    let mut covered = bytes[..cover_end].to_vec();
    for byte in &mut covered[field_offset..field_offset + 32] {
        *byte = 0;
    }
    let mut field = [0u8; 32];
    field[..4].copy_from_slice(&crc32c_castagnoli(&covered).to_le_bytes());
    field
}

// ───────────────────────── A30：区域种类（只按落点认，不读 kind= 字段） ─────────────────────────

const SYSTEM_CONFIGURATION_SLOT_BYTES: u64 = 4096;
const ROOT_RECORD_SLOT_BYTES: u64 = 512;
/// 根环参数（`layout/01-first-txn.md`「一」「根环参数一览」）：起点 1 MiB（64 个 16 KiB 槽）、素数步长 3、
/// chunk 1 MiB、3 个区域 ⇒ 根环覆盖的设备内偏移区间是 [1 MiB, 1 MiB + 3 × 3 × 1 MiB)。
const RING_START_OFFSET: u64 = 1 << 20;
const RING_CHUNK_BYTES: u64 = 1 << 20;
const RING_REGIONS: u64 = 3;
const RING_PRIME_STEP: u64 = 3;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum RegionKind {
    SystemConfiguration,
    Root,
    Other,
}

fn region_kind(offset: u64, length: u64) -> RegionKind {
    if length == SYSTEM_CONFIGURATION_SLOT_BYTES && (offset == 0 || offset == SYSTEM_CONFIGURATION_SLOT_BYTES) {
        RegionKind::SystemConfiguration
    } else if length == ROOT_RECORD_SLOT_BYTES && offset >= RING_START_OFFSET && offset < RING_START_OFFSET + RING_REGIONS * RING_PRIME_STEP * RING_CHUNK_BYTES {
        RegionKind::Root
    } else {
        RegionKind::Other
    }
}

/// 每种区域自证校验和字段的 (覆盖到哪个偏移含补齐, 字段自己的偏移)；`Other` 没有自证校验和。
fn checksum_span(kind: RegionKind) -> Option<(usize, usize)> {
    match kind {
        RegionKind::SystemConfiguration => Some((4096, 155)),
        RegionKind::Root => Some((512, 138)),
        RegionKind::Other => None,
    }
}

/// A25：incompat 第一个字节住系统配置槽内偏移 6（magic 4 + 格式版本 2 之后），不是槽的第 0 个字节
/// （那是 magic 的第一个字节 `S`，`b"SFSB"` 首字节恰是 `0x53`——这一条曾经被写成 `bytes.first()`，
/// 在真实产物上把 incompat 读成了 magic，见这个函数下面那条回归单测）。
fn system_configuration_incompat_byte0(bytes: &[u8]) -> u8 {
    bytes.get(6).copied().unwrap_or(0)
}

fn checksum_recomputes(kind: RegionKind, bytes: &[u8]) -> bool {
    match checksum_span(kind) {
        Some((cover_end, field_offset)) if bytes.len() >= cover_end => wide_checksum_field(bytes, cover_end, field_offset) == bytes[field_offset..field_offset + 32],
        _ => false,
    }
}

// ───────────────────────── A31：字段表（Q142.41 的字段标签用；不设门槛） ─────────────────────────

/// 系统配置字段表（D22 已定项 9，`layout/01-first-txn.md`「一」，行序即偏移累加），46 行、合计 489。
const SYSTEM_CONFIGURATION_FIELD_TABLE: &[(&str, usize)] = &[
    ("magic", 4),
    ("format_version", 2),
    ("feature_bits", 96),
    ("fsid", 16),
    ("writer_identity", 20),
    ("checksum_algorithm", 1),
    ("this_device", 4),
    ("device_count", 4),
    ("slot_generation", 8),
    ("slot_checksum", 32),
    ("system_configuration_mac", 16),
    ("nonce_watermark", 12),
    ("kdf_identifier", 4),
    ("encryption_type", 1),
    ("mac_length", 1),
    ("master_key_slot", 80),
    ("node_bytes", 4),
    ("unit_bytes", 4),
    ("placement_granularity", 4),
    ("location_entry_bytes", 4),
    ("mkfs_physical_block_size", 4),
    ("extension_point_declared_bytes", 4),
    ("journal_start_slot", 8),
    ("journal_ring_bytes", 8),
    ("journal_record_bytes", 4),
    ("journal_in_flight_limit", 4),
    ("journal_worst_case_bytes", 8),
    ("journal_safety_factor", 4),
    ("ring_regions", 1),
    ("ring_slots_per_region", 1),
    ("ring_prime_step", 4),
    ("ring_chunk_bytes", 4),
    ("ring_start_slot", 8),
    ("region_devices", 12),
    ("width_max", 1),
    ("group_size", 1),
    ("mapping_source", 24),
    ("unit_area_start_slot", 8),
    ("mkfs_io_min", 4),
    ("fixed_structure_slot_spacing", 4),
    ("t_time", 4),
    ("t_dirty", 8),
    ("reclaim_watermarks", 24),
    ("journal_tail", 8),
    ("journal_instance", 4),
    ("rollback_floor", 8),
];

/// 根记录字段表（D22 已定项 7，行序即偏移累加），15 行、合计 457。
const ROOT_RECORD_FIELD_TABLE: &[(&str, usize)] = &[
    ("magic", 4),
    ("fsid", 16),
    ("flags", 4),
    ("instance", 4),
    ("checkpoint_txg", 8),
    ("tree_table_pointer", 86),
    ("tree_identifier_watermark", 8),
    ("rollback_floor", 8),
    ("root_checksum", 32),
    ("instance_table_pointer", 86),
    ("mapping_root_pointer", 86),
    ("allocation_record_tree_root_pointer", 86),
    ("algorithm_type", 1),
    ("nonce", 12),
    ("mac", 16),
];

fn field_ranges(table: &'static [(&'static str, usize)]) -> Vec<(&'static str, usize, usize)> {
    let mut ranges = Vec::new();
    let mut position = 0usize;
    for (name, width) in table {
        ranges.push((*name, position, position + width));
        position += width;
    }
    ranges
}

/// Q142.41 的字段标签：这个偏移落在系统配置槽 / 根槽字段表的哪个字段；别种区域一律 `other_region`。
fn field_label(kind: RegionKind, offset_in_region: usize) -> &'static str {
    let table = match kind {
        RegionKind::SystemConfiguration => SYSTEM_CONFIGURATION_FIELD_TABLE,
        RegionKind::Root => ROOT_RECORD_FIELD_TABLE,
        RegionKind::Other => return "other_region",
    };
    field_ranges(table)
        .into_iter()
        .find(|(_, start, end)| (*start..*end).contains(&offset_in_region))
        .map_or("unrecognized_offset", |(name, _, _)| name)
}

fn field_labels_for_segments(kind: RegionKind, segments: &[(usize, usize)]) -> String {
    if segments.is_empty() {
        return "none".to_string();
    }
    let mut labels: Vec<&'static str> = Vec::new();
    for (offset_in_region, length) in segments {
        for index in *offset_in_region..*offset_in_region + *length {
            let label = field_label(kind, index);
            if !labels.contains(&label) {
                labels.push(label);
            }
        }
    }
    labels.join(",")
}

// ───────────────────────── R28：改前改后每个字节归到哪一类 ─────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ClauseClass {
    ClauseLayoutIdentity,
    ClauseRollbackFloor,
    ClauseUnmountMarker,
    Derived,
    Unmapped,
}

impl ClauseClass {
    fn as_str(self) -> &'static str {
        match self {
            ClauseClass::ClauseLayoutIdentity => "clause_d15_4",
            ClauseClass::ClauseRollbackFloor => "clause_d22_9",
            ClauseClass::ClauseUnmountMarker => "clause_d22_7",
            ClauseClass::Derived => "derived",
            ClauseClass::Unmapped => "unmapped",
        }
    }
}

/// 这个偏移是不是三条条款点名的字段之一（A31 的 `clause_*`）；判定 `derived` 资格要用到这个结果，
/// 与最终分类分开写，免得循环依赖。
fn is_clause_offset(kind: RegionKind, offset_in_region: usize) -> bool {
    match kind {
        RegionKind::SystemConfiguration => (6..38).contains(&offset_in_region) || (481..489).contains(&offset_in_region),
        RegionKind::Root => (20..24).contains(&offset_in_region),
        RegionKind::Other => false,
    }
}

fn is_checksum_offset(kind: RegionKind, offset_in_region: usize) -> bool {
    match kind {
        RegionKind::SystemConfiguration => (155..187).contains(&offset_in_region),
        RegionKind::Root => (138..170).contains(&offset_in_region),
        RegionKind::Other => false,
    }
}

/// A31：`derived` 只在「同一个区域里至少有一个 `clause_*` 字节也变了」时才成立（R28 字面）；
/// 没有 `clause_*` 变化时，校验和区间里的差异按 `unmapped` 算（M6 的抓手：不许省掉这个条件）。
fn classify_offset(kind: RegionKind, offset_in_region: usize, any_clause_changed: bool) -> ClauseClass {
    match kind {
        RegionKind::SystemConfiguration => {
            if (6..38).contains(&offset_in_region) {
                ClauseClass::ClauseLayoutIdentity
            } else if (481..489).contains(&offset_in_region) {
                ClauseClass::ClauseRollbackFloor
            } else if is_checksum_offset(kind, offset_in_region) && any_clause_changed {
                ClauseClass::Derived
            } else {
                ClauseClass::Unmapped
            }
        }
        RegionKind::Root => {
            if (20..24).contains(&offset_in_region) {
                ClauseClass::ClauseUnmountMarker
            } else if is_checksum_offset(kind, offset_in_region) && any_clause_changed {
                ClauseClass::Derived
            } else {
                ClauseClass::Unmapped
            }
        }
        RegionKind::Other => ClauseClass::Unmapped,
    }
}

struct ExplainedOutcome {
    explained: bool,
    layout_identity_clause_bytes: u64,
    rollback_floor_clause_bytes: u64,
    unmount_marker_clause_bytes: u64,
    derived_bytes: u64,
    unmapped_bytes: u64,
}

/// R28「对得到」：把旧的一边这个区域里 `clause_*` 字节换成新的一边的值，按 D18 已定项 17 重算这个区域的
/// 校验和字段，得到的整段与新的一边逐字节相同 ⇒ 对得到。逐字节先分类、累计计数，`clause_*` 类当场复制进
/// 重建缓冲区；`derived` 类留给下面统一重算校验和覆盖；`unmapped` 类保留旧值，比对时自然不等。
fn compute_explained(kind: RegionKind, old_bytes: &[u8], new_bytes: &[u8]) -> ExplainedOutcome {
    let compare_length = old_bytes.len().min(new_bytes.len());
    let any_clause_changed = (0..compare_length).any(|index| old_bytes[index] != new_bytes[index] && is_clause_offset(kind, index));
    let mut reconstructed = old_bytes.to_vec();
    let mut layout_identity_clause_bytes = 0u64;
    let mut rollback_floor_clause_bytes = 0u64;
    let mut unmount_marker_clause_bytes = 0u64;
    let mut derived_bytes = 0u64;
    let mut unmapped_bytes = 0u64;
    for index in 0..compare_length {
        if old_bytes[index] == new_bytes[index] {
            continue;
        }
        match classify_offset(kind, index, any_clause_changed) {
            ClauseClass::ClauseLayoutIdentity => {
                layout_identity_clause_bytes += 1;
                reconstructed[index] = new_bytes[index];
            }
            ClauseClass::ClauseRollbackFloor => {
                rollback_floor_clause_bytes += 1;
                reconstructed[index] = new_bytes[index];
            }
            ClauseClass::ClauseUnmountMarker => {
                unmount_marker_clause_bytes += 1;
                reconstructed[index] = new_bytes[index];
            }
            ClauseClass::Derived => derived_bytes += 1,
            ClauseClass::Unmapped => unmapped_bytes += 1,
        }
    }
    if let Some((cover_end, field_offset)) = checksum_span(kind) {
        if reconstructed.len() >= cover_end {
            let recomputed = wide_checksum_field(&reconstructed, cover_end, field_offset);
            reconstructed[field_offset..field_offset + 32].copy_from_slice(&recomputed);
        }
    }
    let explained = reconstructed.len() == new_bytes.len() && reconstructed == new_bytes;
    ExplainedOutcome { explained, layout_identity_clause_bytes, rollback_floor_clause_bytes, unmount_marker_clause_bytes, derived_bytes, unmapped_bytes }
}

fn main() {
    let arguments: Vec<String> = env::args().collect();
    if arguments.get(1).map(String::as_str) == Some("old-new") {
        run_old_new_mode(&arguments);
    } else {
        run_region_diff_mode(&arguments);
    }
}

/// 第一种调用：`e142-region-diff-independent <主产物路径> <crates 导出路径>`（Q142.39）。
fn run_region_diff_mode(arguments: &[String]) {
    assert!(arguments.len() >= 3, "用法：e142-region-diff-independent <主产物路径> <crates 导出路径>");
    let model_text = fs::read_to_string(&arguments[1]).unwrap_or_else(|error| panic!("读不到主产物 {}：{error}", arguments[1]));
    let crates_text = fs::read_to_string(&arguments[2]).unwrap_or_else(|error| panic!("读不到 crates 导出 {}：{error}", arguments[2]));
    let model_lines = parse_model_side_device_region_bytes_lines(&model_text, &arguments[1]);
    let crates_lines = parse_device_region_bytes_lines(&crates_text);
    let mut crates_matched = vec![false; crates_lines.len()];
    let mut equal_count = 0u64;
    let mut unequal_count = 0u64;

    for model_line in &model_lines {
        let region = model_line.region.as_deref().unwrap_or("unknown");
        let kind = region_kind(model_line.offset, model_line.length);
        let Some(matched_index) = crates_lines.iter().position(|line| line.device == model_line.device && line.offset == model_line.offset && line.length == model_line.length) else {
            println!(
                "E7RESULT name=independent_region_diff region={region} device={} offset={} length={} matched=false",
                model_line.device, model_line.offset, model_line.length
            );
            continue;
        };
        crates_matched[matched_index] = true;
        let crates_line = &crates_lines[matched_index];
        let segments = diff_segments(&model_line.hexadecimal, &crates_line.hexadecimal);
        let equal = segments.is_empty() && model_line.hexadecimal.len() == crates_line.hexadecimal.len();
        if equal {
            equal_count += 1;
        } else {
            unequal_count += 1;
        }
        let mismatch_bytes: usize = segments.iter().map(|(_, length)| *length).sum();
        let segments_text = if segments.is_empty() {
            "none".to_string()
        } else {
            segments.iter().map(|(offset_in_region, length)| format!("{offset_in_region}:{length}")).collect::<Vec<_>>().join(",")
        };
        // Q142.41：不设门槛，只报不同字节段落在字段表的哪个字段；全等时 field_labels=none。
        let field_labels = field_labels_for_segments(kind, &segments);
        println!(
            "E7RESULT name=independent_region_diff region={region} device={} offset={} length={} matched=true equal={equal} mismatch_bytes={mismatch_bytes} segments={segments_text} field_labels={field_labels}",
            model_line.device, model_line.offset, model_line.length
        );
    }
    for (index, matched) in crates_matched.iter().enumerate() {
        if !matched {
            let line = &crates_lines[index];
            println!("E7RESULT name=independent_region_unmatched side=crates device={} offset={} length={}", line.device, line.offset, line.length);
        }
    }
    println!(
        "E7RESULT name=independent_region_diff_summary model_regions={} crates_regions={} equal={equal_count} unequal={unequal_count}",
        model_lines.len(),
        crates_lines.len()
    );
}

/// 第二种调用：`e142-region-diff-independent old-new <新产物路径> <旧产物路径>`（Q142.42、Q142.43）。
fn run_old_new_mode(arguments: &[String]) {
    assert!(arguments.len() >= 4, "用法：e142-region-diff-independent old-new <新产物路径> <旧产物路径>");
    let new_text = fs::read_to_string(&arguments[2]).unwrap_or_else(|error| panic!("读不到新产物 {}：{error}", arguments[2]));
    let old_text = fs::read_to_string(&arguments[3]).unwrap_or_else(|error| panic!("读不到旧产物 {}：{error}", arguments[3]));
    let new_lines = parse_model_side_device_region_bytes_lines(&new_text, &arguments[2]);
    let old_lines = parse_model_side_device_region_bytes_lines(&old_text, &arguments[3]);
    let mut old_matched = vec![false; old_lines.len()];
    let mut matched_count = 0u64;
    let mut changed_count = 0u64;
    let mut unchanged_count = 0u64;
    let mut unmatched_new_count = 0u64;
    let mut unmatched_old_count = 0u64;
    let mut mapped_regions = 0u64;
    let mut unmapped_regions = 0u64;
    let mut all_changes_mapped = true;

    for new_line in &new_lines {
        let region = new_line.region.as_deref().unwrap_or("unknown");
        let Some(old_index) = old_lines.iter().position(|line| line.device == new_line.device && line.offset == new_line.offset && line.length == new_line.length) else {
            println!("E7RESULT name=old_new_independent_unmatched side=new region={region} device={} offset={} length={}", new_line.device, new_line.offset, new_line.length);
            unmatched_new_count += 1;
            all_changes_mapped = false;
            continue;
        };
        old_matched[old_index] = true;
        matched_count += 1;
        let old_line = &old_lines[old_index];
        let kind = region_kind(new_line.offset, new_line.length);
        let segments = diff_segments(&old_line.hexadecimal, &new_line.hexadecimal);
        let changed = !segments.is_empty() || old_line.hexadecimal.len() != new_line.hexadecimal.len();
        if !changed {
            unchanged_count += 1;
            println!(
                "E7RESULT name=old_new_independent region={region} device={} offset={} length={} matched=true changed=false changed_bytes=0 segments=none clause_d15_4_bytes=0 clause_d22_9_bytes=0 clause_d22_7_bytes=0 derived_bytes=0 unmapped_bytes=0 explained=true mapped=true",
                new_line.device, new_line.offset, new_line.length
            );
            continue;
        }
        changed_count += 1;
        // `any_clause_changed`要看整个区域里有没有 clause_* 字节变，不是只看当前这一段（R28 字面：
        // 「同一个区域里至少有一个 clause_* 字节也变了」）；按段各算一次会把校验和段错判成 unmapped
        // ——即使区域里别的段确实带着一个 clause_* 变化（实测：真实产物上 PC7 那一对就踩了这个坑，
        // derived 的 4 个字节被打印成 class=unmapped，尽管 old_new_independent 汇总行的
        // derived_bytes 是对的，因为汇总行走的是 compute_explained 里同一份区域级判定）。
        let compare_length = old_line.hexadecimal.len().min(new_line.hexadecimal.len());
        let any_clause_changed = (0..compare_length).any(|index| old_line.hexadecimal[index] != new_line.hexadecimal[index] && is_clause_offset(kind, index));
        for (offset_in_region, length) in &segments {
            let class = classify_offset(kind, *offset_in_region, any_clause_changed).as_str();
            let old_hex: String = old_line.hexadecimal.get(*offset_in_region..*offset_in_region + *length).map_or_else(String::new, |slice| slice.iter().map(|byte| format!("{byte:02x}")).collect());
            let new_hex: String = new_line.hexadecimal.get(*offset_in_region..*offset_in_region + *length).map_or_else(String::new, |slice| slice.iter().map(|byte| format!("{byte:02x}")).collect());
            println!("E7RESULT name=old_new_independent_segment region={region} device={} offset_in_region={offset_in_region} length={length} class={class} old_hex={old_hex} new_hex={new_hex}", new_line.device);
        }
        let outcome = compute_explained(kind, &old_line.hexadecimal, &new_line.hexadecimal);
        let changed_bytes: usize = segments.iter().map(|(_, length)| *length).sum();
        let mapped = outcome.unmapped_bytes == 0 && outcome.explained;
        if mapped {
            mapped_regions += 1;
        } else {
            unmapped_regions += 1;
            all_changes_mapped = false;
        }
        let segments_text = segments.iter().map(|(offset_in_region, length)| format!("{offset_in_region}:{length}")).collect::<Vec<_>>().join(",");
        println!(
            "E7RESULT name=old_new_independent region={region} device={} offset={} length={} matched=true changed=true changed_bytes={changed_bytes} segments={segments_text} clause_d15_4_bytes={} clause_d22_9_bytes={} clause_d22_7_bytes={} derived_bytes={} unmapped_bytes={} explained={} mapped={mapped}",
            new_line.device, new_line.offset, new_line.length,
            outcome.layout_identity_clause_bytes, outcome.rollback_floor_clause_bytes, outcome.unmount_marker_clause_bytes, outcome.derived_bytes, outcome.unmapped_bytes, outcome.explained
        );
    }
    for (index, was_matched) in old_matched.iter().enumerate() {
        if !was_matched {
            let line = &old_lines[index];
            let region = line.region.as_deref().unwrap_or("unknown");
            println!("E7RESULT name=old_new_independent_unmatched side=old region={region} device={} offset={} length={}", line.device, line.offset, line.length);
            unmatched_old_count += 1;
            all_changes_mapped = false;
        }
    }

    // Q142.43：窗口里落在系统配置槽 / 根槽形状的每一条，新旧两边各打一行条款字段的绝对值。
    for (side, lines) in [("new", &new_lines), ("old", &old_lines)] {
        for line in lines.iter() {
            match region_kind(line.offset, line.length) {
                RegionKind::SystemConfiguration => {
                    let bytes = &line.hexadecimal;
                    let incompat_byte0 = system_configuration_incompat_byte0(bytes);
                    let feature_bits_rest_zero = bytes.get(7..102).is_some_and(|slice| slice.iter().all(|byte| *byte == 0));
                    let rollback_floor_hex: String = bytes.get(481..489).map_or_else(String::new, |slice| slice.iter().map(|byte| format!("{byte:02x}")).collect());
                    let padding_after_489_zero = bytes.get(489..).is_some_and(|slice| slice.iter().all(|byte| *byte == 0));
                    let recomputes = checksum_recomputes(RegionKind::SystemConfiguration, bytes);
                    println!(
                        "E7RESULT name=old_new_independent_clause_values side={side} region=system_configuration device={} incompat_byte0={incompat_byte0:#04x} feature_bits_rest_zero={feature_bits_rest_zero} rollback_floor_hex={rollback_floor_hex} padding_after_489_zero={padding_after_489_zero} checksum_recomputes={recomputes}",
                        line.device
                    );
                }
                RegionKind::Root => {
                    let bytes = &line.hexadecimal;
                    let flags_hex: String = bytes.get(20..24).map_or_else(String::new, |slice| slice.iter().map(|byte| format!("{byte:02x}")).collect());
                    let recomputes = checksum_recomputes(RegionKind::Root, bytes);
                    println!("E7RESULT name=old_new_independent_clause_values side={side} region=root_record device={} flags_hex={flags_hex} checksum_recomputes={recomputes}", line.device);
                }
                RegionKind::Other => {}
            }
        }
    }

    println!(
        "E7RESULT name=old_new_independent_summary new_regions={} old_regions={} matched={matched_count} changed={changed_count} unchanged={unchanged_count} unmatched_new={unmatched_new_count} unmatched_old={unmatched_old_count} mapped_regions={mapped_regions} unmapped_regions={unmapped_regions}",
        new_lines.len(),
        old_lines.len()
    );
    println!("E7RESULT name=old_new_independent_verdict all_changes_mapped={all_changes_mapped}");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_line(region: &str, device: u32, offset: u64, hexadecimal: &str) -> String {
        format!("E7RESULT name=device_region_bytes region={region} device={device} offset={offset} length={} sha256=ignored hexadecimal={hexadecimal}", hexadecimal.len() / 2)
    }

    /// P3 单测一：导出比自己 ⇒ 0 处不同。
    #[test]
    fn comparing_a_dump_against_itself_reports_zero_differences() {
        let text = format!("{}\n{}\n", sample_line("region_a", 0, 100, "aabbcc"), sample_line("region_b", 1, 200, "112233"));
        let model_lines = parse_device_region_bytes_lines(&text);
        let crates_lines = parse_device_region_bytes_lines(&text);
        for model_line in &model_lines {
            let matched = find_matching_line(&crates_lines, model_line.device, model_line.offset, model_line.length).expect("配得上");
            assert_eq!(diff_segments(&model_line.hexadecimal, &matched.hexadecimal), Vec::new(), "自己比自己 0 处不同");
        }
    }

    /// P3 单测二：改一个十六进制字符 ⇒ 恰报那一个区域、那一段。
    #[test]
    fn flipping_one_hex_character_reports_exactly_that_region_and_segment() {
        let model_text = sample_line("region_a", 0, 100, "aabbcc");
        let crates_text = sample_line("region_a", 0, 100, "aab0cc"); // 第二字节 bb → b0
        let model_lines = parse_device_region_bytes_lines(&model_text);
        let crates_lines = parse_device_region_bytes_lines(&crates_text);
        let matched = find_matching_line(&crates_lines, model_lines[0].device, model_lines[0].offset, model_lines[0].length).expect("配得上");
        let segments = diff_segments(&model_lines[0].hexadecimal, &matched.hexadecimal);
        assert_eq!(segments, vec![(1, 1)], "恰一段：区域内偏移 1、长度 1");
    }

    /// P3 单测三：删一行 ⇒ 报那个区域配不上。
    #[test]
    fn deleting_a_line_reports_that_region_as_unmatched() {
        let model_text = format!("{}\n{}\n", sample_line("region_a", 0, 100, "aabbcc"), sample_line("region_b", 0, 200, "112233"));
        let crates_text = sample_line("region_a", 0, 100, "aabbcc"); // 删掉 region_b 那一行
        let model_lines = parse_device_region_bytes_lines(&model_text);
        let crates_lines = parse_device_region_bytes_lines(&crates_text);
        let unmatched: Vec<&ParsedRegionLine> = model_lines
            .iter()
            .filter(|line| find_matching_line(&crates_lines, line.device, line.offset, line.length).is_none())
            .collect();
        assert_eq!(unmatched.len(), 1, "只有 region_b 配不上");
        assert_eq!(unmatched[0].region.as_deref(), Some("region_b"));
    }

    /// A29：CRC-32C 标准测试向量（RFC 3720），命中即证明多项式、反射与初值都没写反（M9 的抓手：
    /// 换成 IEEE 反射多项式 0xEDB88320 会算出别的数）。
    #[test]
    fn crc32c_matches_the_standard_test_vector() {
        assert_eq!(crc32c_castagnoli(b"123456789"), 0xE306_9283);
    }

    /// A30：系统配置槽只认设备内偏移 0 或 4096、长 4096；槽距只有两个槽，第三个偏移不算。
    #[test]
    fn system_configuration_region_is_recognized_by_offset_and_length() {
        assert_eq!(region_kind(0, 4096), RegionKind::SystemConfiguration);
        assert_eq!(region_kind(4096, 4096), RegionKind::SystemConfiguration);
        assert_eq!(region_kind(8192, 4096), RegionKind::Other, "系统配置只有槽 0 / 1 两个偏移");
    }

    /// A30：根槽长 512、设备内偏移落在根环覆盖区间 [1 MiB, 1 MiB + 9 MiB) 里；区间右端点开区间。
    #[test]
    fn root_record_region_is_recognized_by_offset_and_length() {
        assert_eq!(region_kind(1_048_576, 512), RegionKind::Root, "根环起点");
        assert_eq!(region_kind(1_048_576 + 8 * 4096, 512), RegionKind::Root, "区域内第 8 个槽");
        assert_eq!(region_kind(10_485_760, 512), RegionKind::Other, "根环覆盖区间的右端点是开区间");
        assert_eq!(region_kind(1_048_576, 4096), RegionKind::Other, "长度不是 512 就不算根槽");
    }

    /// A25：布局身份位住系统配置槽内偏移 6（M5 的抓手：把 clause_d15_4 的起点从 6 挪到 7，
    /// 这个断言会把偏移 6 判成 unmapped 而不是 clause_d15_4）。
    #[test]
    fn layout_identity_byte_is_classified_as_the_layout_identity_clause_not_unmapped() {
        assert_eq!(classify_offset(RegionKind::SystemConfiguration, 6, false), ClauseClass::ClauseLayoutIdentity, "布局身份位住偏移 6（A25）");
    }

    /// R28：只有校验和字节变、没有任何 clause_* 字节也变 ⇒ 那个字节按 unmapped 算，不算 derived
    /// （PC6 (ii)、M6 的抓手：删掉「同一区域里至少有一个 clause_* 字节也变了」这个前提会把它错判成 derived）。
    #[test]
    fn checksum_byte_change_without_a_clause_change_is_unmapped() {
        let mut old_bytes = vec![0u8; 4096];
        old_bytes[6] = 0x02;
        let mut new_bytes = old_bytes.clone();
        new_bytes[155] ^= 0x01; // 只改校验和字节，不碰任何 clause_* 字节
        let outcome = compute_explained(RegionKind::SystemConfiguration, &old_bytes, &new_bytes);
        assert_eq!(outcome.derived_bytes, 0, "没有 clause_* 字节也变，校验和那一字节不算 derived");
        assert_eq!(outcome.unmapped_bytes, 1);
        assert!(!outcome.explained, "对不到");
    }

    /// R28：根 flags 变而自证校验和没跟着重算 ⇒ 对不到（PC6 (iii)、M7 的抓手：
    /// 「不重算校验和、explained 恒 true」会把这里错判成对得到）。
    #[test]
    fn root_flags_change_without_recomputed_checksum_is_unmapped() {
        let old_bytes = vec![0u8; 512];
        let mut new_bytes = old_bytes.clone();
        new_bytes[20] = 0x01; // 卸载记号置 1，但没有重算自证校验和
        let outcome = compute_explained(RegionKind::Root, &old_bytes, &new_bytes);
        assert_eq!(outcome.unmount_marker_clause_bytes, 1);
        assert!(!outcome.explained, "flags 变而校验和没跟着变 ⇒ 对不到");
    }

    /// R28：根 flags 变、自证校验和照重算 ⇒ 对得到（M10 的抓手：把根 flags 那一格写成 [21, 24) 会漏掉偏移 20）。
    #[test]
    fn root_flags_change_with_recomputed_checksum_is_explained_by_the_unmount_marker_clause() {
        let old_bytes = vec![0u8; 512];
        let mut new_bytes = old_bytes.clone();
        new_bytes[20] = 0x01;
        let recomputed = wide_checksum_field(&new_bytes, 512, 138);
        new_bytes[138..170].copy_from_slice(&recomputed);
        let outcome = compute_explained(RegionKind::Root, &old_bytes, &new_bytes);
        assert_eq!(outcome.unmount_marker_clause_bytes, 1);
        assert_eq!(outcome.unmapped_bytes, 0);
        assert!(outcome.explained, "flags 变、校验和照重算 ⇒ 对得到");
    }

    /// R26：模型那一侧只取第一行 name=done 之前的（M8 的抓手：整份文本都取会把 done 之后的行也算进来）。
    #[test]
    fn model_side_extraction_stops_before_the_first_done_line() {
        let text = format!("{}\nE7RESULT name=done emitted=1\n{}\n", sample_line("region_a", 0, 100, "aabbcc"), sample_line("region_b", 0, 200, "112233"));
        let lines = parse_model_side_device_region_bytes_lines(&text, "测试文件");
        assert_eq!(lines.len(), 1, "name=done 之后的行不取");
        assert_eq!(lines[0].region.as_deref(), Some("region_a"));
    }

    /// R26：没有 name=done 就报错退出，不许默默整份取。
    #[test]
    #[should_panic(expected = "没有 name=done")]
    fn model_side_extraction_panics_without_a_done_line() {
        let text = sample_line("region_a", 0, 100, "aabbcc");
        parse_model_side_device_region_bytes_lines(&text, "测试文件");
    }

    /// 回归：incompat 第一个字节要从槽内偏移 6 读，不是从槽的第 0 个字节（magic）读——
    /// 这条 bug 在 E142 第十八次跑真实产物上把 incompat_byte0 打成了 `0x53`（`b"SFSB"` 的首字节）。
    #[test]
    fn system_configuration_incompat_byte0_reads_offset_six_not_the_magic() {
        let mut slot = vec![0u8; 4096];
        slot[0..4].copy_from_slice(b"SFSB");
        slot[6] = 0x02;
        assert_eq!(system_configuration_incompat_byte0(&slot), 0x02);
    }

    /// 回归：一个区域里有两段不同字节时，`any_clause_changed` 要看整个区域，不能只看当前这一段——
    /// 否则校验和那一段会被单独判成 unmapped，即使同一区域里另一段确实带着 clause_* 变化。
    /// 复现 E142 第十八次跑真实产物踩到的形状：偏移 6（clause）与偏移 155..159（校验和）两段都变。
    #[test]
    fn old_new_segment_class_uses_region_wide_any_clause_changed_not_per_segment() {
        let mut old_bytes = vec![0u8; 4096];
        old_bytes[6] = 0x01;
        let mut new_bytes = old_bytes.clone();
        new_bytes[6] = 0x02;
        new_bytes[155..159].copy_from_slice(&[0x99, 0x99, 0x99, 0x99]); // 模拟校验和跟着变
        let kind = RegionKind::SystemConfiguration;
        let compare_length = old_bytes.len().min(new_bytes.len());
        let any_clause_changed = (0..compare_length).any(|index| old_bytes[index] != new_bytes[index] && is_clause_offset(kind, index));
        assert!(any_clause_changed, "偏移 6 变了，应该判定这个区域「有 clause_* 字节也变」");
        assert_eq!(classify_offset(kind, 155, any_clause_changed), ClauseClass::Derived, "校验和段要按区域级判定分到 derived，不是 unmapped");
    }
}
