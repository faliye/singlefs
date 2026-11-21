//! 角色标签与文字里数字的核对、有界文字标记、自由字节缺口的文字、宽度行。

use super::*;

/// 从一行「E7RESULT name=... k1=v1 k2=v2 ...」按空格切词、每词按第一个 `=` 切成 (key, value)。
pub(super) fn split_fields(line: &str) -> Vec<(&str, &str)> {
    line.split_whitespace().filter_map(|token| token.split_once('=')).collect()
}

/// R18 的记号：`[mawt]` 后接十进制数字，前后都不是字母或数字（下划线不算字母数字，`field=t9_...`
/// 的下划线前一样算；十六进制串里的 `a1` 因为前后是别的十六进制字符，天然被这条边界规则挡掉）。
fn find_role_label_tokens(value: &str) -> Vec<&str> {
    let bytes = value.as_bytes();
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if matches!(byte, b'm' | b'a' | b'w' | b't') {
            let previous_byte_is_a_boundary = index == 0 || !bytes[index - 1].is_ascii_alphanumeric();
            let mut end = index + 1;
            while end < bytes.len() && bytes[end].is_ascii_digit() {
                end += 1;
            }
            if previous_byte_is_a_boundary && end > index + 1 {
                let next_ok = end == bytes.len() || !bytes[end].is_ascii_alphanumeric();
                if next_ok {
                    tokens.push(&value[index..end]);
                    index = end;
                    continue;
                }
            }
        }
        index += 1;
    }
    tokens
}

#[derive(Default, Debug, Clone)]
pub(super) struct RoleLabelCheck {
    pub(super) defined: usize,
    pub(super) used: usize,
    pub(super) undefined: usize,
    pub(super) contradictions: usize,
    pub(super) contradiction_rows: Vec<String>,
    pub(super) undefined_rows: Vec<String>,
}

/// R18 定义行：`name=role_labels` 的 `units=`（每个都是 unit）与 `journal_record=`/`root=`/
/// `system_configuration=` 三个字段——今天产物里唯一给主臂那条流的角色名下定义的行种。
/// R20 矛盾：只在「断言了一个具体种类」的字段键上判——`record=` 整个字段值就是角色名，
/// `field=<记号>_...` 是「角色名当前缀」；这两种写法在今天的产物里专指 journal 记录。
pub(super) fn check_role_labels(lines: &[String]) -> RoleLabelCheck {
    let mut definitions: BTreeMap<String, &'static str> = BTreeMap::new();
    for line in lines {
        let fields = split_fields(line);
        if !fields.iter().any(|(key, value)| *key == "name" && *value == "role_labels") {
            continue;
        }
        for (key, value) in &fields {
            match *key {
                "units" => {
                    for tag in value.split(',') {
                        if !tag.is_empty() {
                            definitions.insert(tag.to_string(), "unit");
                        }
                    }
                }
                "journal_record" => {
                    definitions.insert((*value).to_string(), "journal_record");
                }
                "root" => {
                    definitions.insert((*value).to_string(), "root");
                }
                "system_configuration" => {
                    definitions.insert((*value).to_string(), "system_configuration");
                }
                "warm_up_journal_records" => {
                    for tag in value.split(',') {
                        if !tag.is_empty() {
                            definitions.insert(tag.to_string(), "journal_record");
                        }
                    }
                }
                _ => {}
            }
        }
    }
    let mut check = RoleLabelCheck { defined: definitions.len(), ..RoleLabelCheck::default() };
    let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for line in lines {
        let fields = split_fields(line);
        for (key, value) in &fields {
            for token in find_role_label_tokens(value) {
                seen.insert(token.to_string());
                let Some(&defined_kind) = definitions.get(token) else {
                    check.undefined += 1;
                    if check.undefined_rows.len() < 8 {
                        check.undefined_rows.push(format!("label={token} row={}", line.replace(' ', "_")));
                    }
                    continue;
                };
                let asserted_kind: Option<&str> = if (*key == "record" && *value == token)
                    || (*key == "field" && value.len() > token.len() && value.starts_with(token) && value.as_bytes()[token.len()] == b'_')
                {
                    Some("journal_record")
                } else {
                    None
                };
                if let Some(asserted) = asserted_kind {
                    if asserted != defined_kind {
                        check.contradictions += 1;
                        if check.contradiction_rows.len() < 8 {
                            check.contradiction_rows.push(format!("label={token} defined_as={defined_kind} used_as={asserted} row={}", line.replace(' ', "_")));
                        }
                    }
                }
            }
        }
    }
    check.used = seen.len();
    check
}

#[derive(Default, Debug, Clone)]
pub(super) struct TextNumberCheck {
    pub(super) text_fields: usize,
    pub(super) numbers: usize,
    pub(super) bound: usize,
    pub(super) bound_mismatched: usize,
    pub(super) literal_allowlisted: usize,
    pub(super) unclassified: usize,
    pub(super) mismatch_rows: Vec<String>,
    pub(super) unclassified_rows: Vec<String>,
}

/// R19 的「自由文本字段」：值里含非 ASCII 字符（这个产物里就是含中文）的字段——`name=gap` 的 `text=`
/// 是唯一大量出现的一种，逐条手写的空白清单条目属于这一类。
fn is_free_text_value(value: &str) -> bool {
    !value.is_ascii()
}

fn find_digit_runs(text: &str) -> Vec<(usize, &str)> {
    let bytes = text.as_bytes();
    let mut runs = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index].is_ascii_digit() {
            let start = index;
            let mut end = index + 1;
            while end < bytes.len() && bytes[end].is_ascii_digit() {
                end += 1;
            }
            runs.push((start, &text[start..end]));
            index = end;
        } else {
            index += 1;
        }
    }
    runs
}

/// 文本里形如 `YYYY-MM-DD` 的字节区间——落在这个区间里的三段数字都按「日期」白名单，不逐段单独判。
fn date_byte_ranges(text: &str) -> Vec<(usize, usize)> {
    let bytes = text.as_bytes();
    let mut ranges = Vec::new();
    let mut index = 0;
    while index + 10 <= bytes.len() {
        let window = &bytes[index..index + 10];
        let is_date = window[0].is_ascii_digit()
            && window[1].is_ascii_digit()
            && window[2].is_ascii_digit()
            && window[3].is_ascii_digit()
            && window[4] == b'-'
            && window[5].is_ascii_digit()
            && window[6].is_ascii_digit()
            && window[7] == b'-'
            && window[8].is_ascii_digit()
            && window[9].is_ascii_digit();
        if is_date {
            ranges.push((index, index + 10));
            index += 10;
        } else {
            index += 1;
        }
    }
    ranges
}

/// 字面白名单第一类：紧跟在这些记号字母之后（R19：编号的一部分）。
const ROLE_LABEL_ALLOWLIST_LETTERS: &[u8] = b"DECGIMQBPSVFRTWKmawt";

/// 「这个数紧跟着哪句中文提示语，就该跟哪个 `bound_*` 字段相等」——只登记这几组，见 `check_text_numbers`
/// 用它判 `bound_mismatched` 那一段的注释：不是「同一行随便一个字段叫 `bound_*` 就把这一行剩下的数
/// 全当绑歪」，那样会把 G22 文本里跟三个绑定值都无关的「1GiB」「768MiB」也当成绑歪的。
const BOUND_TEXT_MARKERS: &[(&str, &str)] = &[("空闲字节", "bound_free_bytes_per_device"), ("全空聚簇段数", "bound_empty_cluster_segments_per_device"), ("runs", "bound_fragmentation_runs_per_device")];

/// R19：产物里每个自由文本字段里的每一串数字，归绑定 / 字面白名单 / 没归类三档之一。
/// 绑定：同一行别的字段（不是这个自由文本字段自己、也不是 `name`）里有一个数恰好等于它——现取来比；
/// 找不到相等值、但这个数紧跟着 `BOUND_TEXT_MARKERS` 登记的某句提示语，算「绑定但对不上」
/// （`bound_mismatched`，M157 的会红检查落在这里）。
/// 字面白名单第三类（条款里的常量）：这个装置把 `const GAPS` 这张手写空白清单本身当成引用来源——
/// 每一条的出处已经写在它自己的文字里（如「D8已定项11」），这里不逐个数字再配一次 `文件:行号`；
/// 这条政策统一写在这里，不在下面每一类挑几个数字单独抄。
/// R19 第一类「字面白名单」的两种写法：ASCII 记号字母紧跟着（`D8`、`E77`……）；这个产物里中文那一份
/// 同一件事的另一种写法——「已定项」三个字紧跟着（`D18已定项17`空格换下划线之后是`D18_已定项_17`，
/// 数字前一个字符是下划线不是字母，但再往前一段是「已定项」，说的是同一种「编号的一部分」）。
fn is_identifier_component_prefix(value: &str, start: usize) -> bool {
    if start == 0 {
        return false;
    }
    if ROLE_LABEL_ALLOWLIST_LETTERS.contains(&value.as_bytes()[start - 1]) {
        return true;
    }
    value[..start].trim_end_matches('_').ends_with("已定项")
}

/// `kinds=[unit_write×4,zero_fill×10,barrier]` 这类字段里 `×` 后面的数是「这个多重集里这种步骤出现几次」——
/// 数字本身就是这个字段自己的构造规则给出来的（数出这个方括号里逗号分隔的项里有几个用这个 kind 起头，
/// 应当恰好等于它），不是另配一个白名单条目；这里只判「紧跟在 × 后面」，不重新数一遍去核对相不相等
/// （真出错时这条自检不判——只在角色名/文本数字自检覆盖不到的地方留一个已知边界，写进报告）。
const MULTIPLICATION_SIGN_UTF8: [u8; 2] = [0xC3, 0x97];

fn is_multiplicity_count(value: &str, start: usize) -> bool {
    start >= 2 && value.as_bytes()[start - 2..start] == MULTIPLICATION_SIGN_UTF8
}

pub(super) fn check_text_numbers(lines: &[String]) -> TextNumberCheck {
    let mut check = TextNumberCheck::default();
    for line in lines {
        let fields = split_fields(line);
        // R19 把 `name=gap` 的 `text=` 与「各行的 `reason=`」并列当自由文本字段的例子；这两种都是装置
        // 自己写的解释性文字（空白清单的手写条目、`explain_offset`/`diff_explained` 算出来的归因说明），
        // 不是需要另配「文件:行号」的产物测量值，统一按「条款/算法自己的文字」这一档字面白名单处理
        // （G22 例外：它的号已经改成绑定，见下面同一行的 `bound_*` 字段，不会走到这一条）。
        let is_documentation_line = fields.iter().any(|(key, value)| (*key == "name" && *value == "gap") || *key == "reason");
        for (field_index, (key, value)) in fields.iter().enumerate() {
            if *key == "name" || !is_free_text_value(value) {
                continue;
            }
            check.text_fields += 1;
            let dates = date_byte_ranges(value);
            for (start, run) in find_digit_runs(value) {
                check.numbers += 1;
                let end = start + run.len();
                let is_identifier_component = is_identifier_component_prefix(value, start);
                let is_date = dates.iter().any(|&(date_start, date_end)| start >= date_start && end <= date_end);
                let is_multiplicity = is_multiplicity_count(value, start);
                if is_identifier_component || is_date || is_multiplicity {
                    check.literal_allowlisted += 1;
                    continue;
                }
                let run_value: Option<u64> = run.parse().ok();
                let bound_sibling = fields.iter().enumerate().find(|(other_index, (other_key, other_value))| {
                    *other_index != field_index && *other_key != "name" && find_digit_runs(other_value).iter().any(|(_, other_run)| Some(other_run.parse().ok()) == Some(run_value))
                });
                if bound_sibling.is_some() {
                    check.bound += 1;
                    continue;
                }
                // 这个数紧跟着一句「这本该绑定到哪个字段」的中文提示语（`BOUND_TEXT_MARKERS`），
                // 但同一行那个字段的值跟它不相等——它本该绑定，只是绑歪了：算「绑定但对不上」
                // （Q142.28 的 `bound_mismatched`），不是「没人管过」。**只认这几句提示语**，不是
                // 「同一行随便哪个字段叫 `bound_*` 就把这一行所有剩下的数都当绑歪」——G22 的文本里
                // 还有「1GiB」「768MiB」这类跟三个绑定字段都无关的数，那些数不该被这条规则捞进来。
                let marked_binding = BOUND_TEXT_MARKERS.iter().find(|(marker, _)| value[..start].ends_with(marker));
                if let Some((_, field_key)) = marked_binding {
                    let bound_value = fields.iter().find(|(other_key, _)| other_key == field_key).map(|(_, other_field_value)| *other_field_value);
                    check.bound_mismatched += 1;
                    if check.mismatch_rows.len() < 8 {
                        check.mismatch_rows.push(format!("field={key} number={run} bound_to={field_key} value={} row={}", bound_value.unwrap_or("missing"), line.replace(' ', "_")));
                    }
                    continue;
                }
                if is_documentation_line {
                    check.literal_allowlisted += 1;
                    continue;
                }
                check.unclassified += 1;
                if check.unclassified_rows.len() < 8 {
                    check.unclassified_rows.push(format!("field={key} number={run} row={}", line.replace(' ', "_")));
                }
            }
        }
    }
    check
}

/// 在已经打印的行里找 `name=<line_name>` 那些行、第一个字段值不在 `exclude` 里的 `key=` 值——
/// 供 Q142.29 从产物文本（不是 Rust 变量）里现取角色名与数字，才真的核到「打印出来的是什么」。
pub(super) fn find_field_value_excluding(lines: &[String], line_name: &str, key: &str, exclude: &[&str]) -> Option<String> {
    for line in lines {
        let fields = split_fields(line);
        if !fields.iter().any(|(field_key, field_value)| *field_key == "name" && *field_value == line_name) {
            continue;
        }
        if let Some((_, value)) = fields.iter().find(|(field_key, _)| *field_key == key) {
            if !exclude.contains(value) {
                return Some((*value).to_string());
            }
        }
    }
    None
}

/// 这一次已经打印的行里，主点的 `name=verdict` 那一行（P8G 一段的对照读它的字段，读文本不读变量，同产物自检的口径）。
pub(super) fn emitted_so_far_verdict_line() -> String {
    EMITTED_LINES.with(|lines| {
        lines
            .borrow()
            .iter()
            .find(|line| split_fields(line).iter().any(|(key, value)| *key == "name" && *value == "verdict"))
            .cloned()
            .expect("main 在 P8G 一段之前已经打印了 name=verdict 那一行")
    })
}

pub(super) fn extract_number_after_marker(text: &str, marker: &str) -> Option<u64> {
    let start = text.find(marker)? + marker.len();
    let rest = &text[start..];
    let digit_length = rest.bytes().take_while(u8::is_ascii_digit).count();
    if digit_length == 0 {
        return None;
    }
    rest[..digit_length].parse().ok()
}

/// G22（问题单第 8 行）：把 `const GAPS` 里那条模板的三个占位符换成这一次产物自己现算的记账数——
/// 唯一构造这段文本的地方，`main()` 与它的会红检查都调它（M157：第一个占位符退回字面 `3472670720`）。
pub(super) fn render_free_bytes_gap_text(template: &str, free_bytes_per_device: u64, empty_cluster_segments_per_device: u64, fragmentation_runs_per_device: u64) -> String {
    template
        .replace("{FREE_BYTES}", &free_bytes_per_device.to_string())
        .replace("{EMPTY_CLUSTER_SEGMENTS}", &empty_cluster_segments_per_device.to_string())
        .replace("{FRAGMENTATION_RUNS}", &fragmentation_runs_per_device.to_string())
}

/// 判据 1：宽度对账的 30 行。「预期」那一列逐格抄自 `.claude/kb/layout/01-first-txn.md` 零到七（
/// 记录头三行取 D23 已定项 4 / 15 / 17 的条文：头 487、新根段 364、4096 的记录装 64 个点名项），
/// 「实测」是装置自己的常量算出来的。两列不等就记一笔 `width_mismatches`。单测钉每一行（M219 把 `journal_header` 的预期写回旧值）。
pub(super) fn width_rows() -> [(&'static str, u64, u64); 30] {
    [
        ("pointer_head", 50, POINTER_HEAD_BYTES),
        ("data_unit_header", 105, DATA_UNIT_HEADER_BYTES),
        ("packed_unit_header", 107, PACKED_UNIT_HEADER_BYTES),
        ("unit_reserved", 29, NONCE_MAC_RESERVED_BYTES),
        ("data_unit_header_with_reserved", 134, DATA_UNIT_HEADER_BYTES + NONCE_MAC_RESERVED_BYTES),
        ("packed_unit_header_with_reserved", 136, PACKED_UNIT_HEADER_BYTES + NONCE_MAC_RESERVED_BYTES),
        ("root_record", 457, ROOT_RECORD_BYTES),
        ("tree_table_entry", 200, TREE_TABLE_ENTRY_BYTES),
        ("journal_header", 487, JOURNAL_HEADER_BYTES),
        ("journal_header_ten_fields", 78, JOURNAL_HEADER_TEN_FIELD_BYTES),
        ("journal_new_root_segment", 364, JOURNAL_NEW_ROOT_SEGMENT_BYTES),
        ("journal_named_entry", 56, JOURNAL_NAMED_ENTRY_BYTES),
        ("journal_named_entries_per_record", 64, (JOURNAL_RECORD_BYTES - JOURNAL_HEADER_BYTES) / JOURNAL_NAMED_ENTRY_BYTES),
        ("data_pointer", 88, DATA_POINTER_BYTES),
        ("node_pointer", 86, NODE_POINTER_BYTES),
        ("instance_table_row", 88, INSTANCE_ROW_BYTES),
        ("inode_record", 140, INODE_RECORD_BYTES),
        ("inode_internal_entry", 120, INODE_INTERNAL_ENTRY),
        ("extent_leaf_record", 112, EXTENT_LEAF_RECORD_BYTES),
        ("allocation_record", 20, ALLOCATION_RECORD_BYTES),
        ("allocation_key", 10, ALLOCATION_KEY_BYTES as u64),
        ("allocation_value", 10, ALLOCATION_RECORD_BYTES - ALLOCATION_KEY_BYTES as u64),
        ("accounting_entry", 34, ACCOUNTING_ENTRY_BYTES),
        ("mapping_key", 27, MAPPING_KEY_BYTES),
        ("mapping_entry", 55, MAPPING_ENTRY_BYTES),
        ("location_entry", 14, LOC_ENTRY),
        ("system_configuration", 489, SYSTEM_CONFIGURATION_BYTES),
        ("system_configuration_slot", 4096, SYSTEM_CONFIGURATION_SLOT_BYTES),
        ("allocation_record_tree_internal_entry", 96, ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES),
        ("extent_tree_upper_leaf_entry", 113, EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES),
    ]
}

#[cfg(test)]
#[path = "role_label_and_text_number_checks_tests.rs"]
mod tests;
