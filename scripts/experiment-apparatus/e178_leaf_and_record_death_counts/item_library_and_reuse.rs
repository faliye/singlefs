//! 摘要、选项与项库、按键读回与复用、作用变化的单元描述。

use super::*;

/// SHA-256（FIPS 180-4）：第四段续跑的项键由它算（登记修订 4 之五）。单测钉空串与 "abc" 的标准值。
fn sha256_hex(message: &[u8]) -> String {
    const ROUND_CONSTANTS: [u32; 64] = [
        0x428a_2f98, 0x7137_4491, 0xb5c0_fbcf, 0xe9b5_dba5, 0x3956_c25b, 0x59f1_11f1, 0x923f_82a4, 0xab1c_5ed5, 0xd807_aa98, 0x1283_5b01,
        0x2431_85be, 0x550c_7dc3, 0x72be_5d74, 0x80de_b1fe, 0x9bdc_06a7, 0xc19b_f174, 0xe49b_69c1, 0xefbe_4786, 0x0fc1_9dc6, 0x240c_a1cc,
        0x2de9_2c6f, 0x4a74_84aa, 0x5cb0_a9dc, 0x76f9_88da, 0x983e_5152, 0xa831_c66d, 0xb003_27c8, 0xbf59_7fc7, 0xc6e0_0bf3, 0xd5a7_9147,
        0x06ca_6351, 0x1429_2967, 0x27b7_0a85, 0x2e1b_2138, 0x4d2c_6dfc, 0x5338_0d13, 0x650a_7354, 0x766a_0abb, 0x81c2_c92e, 0x9272_2c85,
        0xa2bf_e8a1, 0xa81a_664b, 0xc24b_8b70, 0xc76c_51a3, 0xd192_e819, 0xd699_0624, 0xf40e_3585, 0x106a_a070, 0x19a4_c116, 0x1e37_6c08,
        0x2748_774c, 0x34b0_bcb5, 0x391c_0cb3, 0x4ed8_aa4a, 0x5b9c_ca4f, 0x682e_6ff3, 0x748f_82ee, 0x78a5_636f, 0x84c8_7814, 0x8cc7_0208,
        0x90be_fffa, 0xa450_6ceb, 0xbef9_a3f7, 0xc671_78f2,
    ];
    let mut state: [u32; 8] = [0x6a09_e667, 0xbb67_ae85, 0x3c6e_f372, 0xa54f_f53a, 0x510e_527f, 0x9b05_688c, 0x1f83_d9ab, 0x5be0_cd19];
    let message_bits = length_as_u64(message.len()) * 8;
    let mut padded = message.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&message_bits.to_be_bytes());
    for block in padded.as_chunks::<64>().0 {
        let mut schedule = [0_u32; 64];
        for (word_position, word) in block.as_chunks::<4>().0.iter().enumerate() {
            schedule[word_position] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for word_position in 16..64 {
            let earlier = schedule[word_position - 15];
            let recent = schedule[word_position - 2];
            let small_sigma_zero = earlier.rotate_right(7) ^ earlier.rotate_right(18) ^ (earlier >> 3);
            let small_sigma_one = recent.rotate_right(17) ^ recent.rotate_right(19) ^ (recent >> 10);
            schedule[word_position] = schedule[word_position - 16]
                .wrapping_add(small_sigma_zero)
                .wrapping_add(schedule[word_position - 7])
                .wrapping_add(small_sigma_one);
        }
        let mut working = state;
        for (round_constant, scheduled_word) in ROUND_CONSTANTS.iter().zip(schedule) {
            let big_sigma_one = working[4].rotate_right(6) ^ working[4].rotate_right(11) ^ working[4].rotate_right(25);
            let choice = (working[4] & working[5]) ^ (!working[4] & working[6]);
            let first_sum =
                working[7].wrapping_add(big_sigma_one).wrapping_add(choice).wrapping_add(*round_constant).wrapping_add(scheduled_word);
            let big_sigma_zero = working[0].rotate_right(2) ^ working[0].rotate_right(13) ^ working[0].rotate_right(22);
            let majority = (working[0] & working[1]) ^ (working[0] & working[2]) ^ (working[1] & working[2]);
            let second_sum = big_sigma_zero.wrapping_add(majority);
            working = [
                first_sum.wrapping_add(second_sum),
                working[0],
                working[1],
                working[2],
                working[3].wrapping_add(first_sum),
                working[4],
                working[5],
                working[6],
            ];
        }
        for (word, added) in state.iter_mut().zip(working) {
            *word = word.wrapping_add(added);
        }
    }
    state.iter().map(|word| format!("{word:08x}")).collect()
}

/// 第四段跑法的选项（登记修订 4 之五）：项库。项键对得上的项一律读回，没有不读库的跑法。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SegmentFourOptions {
    pub(super) library: SegmentFourLibrary,
}

/// 项库（prune-server 里这个装置那一种，经 `singlefs_device_library` 转交）与这一趟的代码键：取键脚本 `scripts/experiment-apparatus/device-code-key.py`
/// 打的组成清单里 `code_key` 那一行；清单原文照存进 `codes/<代码键>.txt`（只给「变了的单元」那一句用），取键那天与机器只进项文件的 meta 行。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SegmentFourLibrary {
    pub(super) store: DeviceLibrary,
    code_key: String,
    composition: String,
    pub(super) judged: String,
    host: String,
}

/// 项库里的相对键（`crates/singlefs-device-library/src/lib.rs` 文件头「相对键」）。
impl SegmentFourLibrary {
    fn item_path(&self, item_key: &str) -> String {
        format!("items/{item_key}.out")
    }

    fn composition_path(&self, code_key: &str) -> String {
        format!("codes/{code_key}.txt")
    }

    fn index_path(&self) -> String {
        "index.tsv".to_string()
    }
}

/// 取键脚本打的组成清单的第一行（`device-code-key.py` 的 FORM_LINE）。
const COMPOSITION_FORM_LINE: &str = "form\tdevice-code-key-v1";
/// 第四段的项都从这个入口进（登记修订 4 之一「读法写死」）：组成清单里那一行 code_key 的入口要以它收尾。
const SEGMENT_FOUR_ENTRY_SUFFIX: &str = "::segment_four_item_lines";

/// 读组成清单：第一行是 form，恰好一行 code_key（入口是 `segment_four_item_lines`，键是十六进制），judged 与 host 两行可缺（缺写 `-`）。
pub(super) fn segment_four_library_from_composition(store: DeviceLibrary, composition: &str) -> Result<SegmentFourLibrary, String> {
    if composition.lines().next() != Some(COMPOSITION_FORM_LINE) {
        return Err("--code-key-file 不是 device-code-key.py 打的组成清单（第一行不是 form\tdevice-code-key-v1）".to_string());
    }
    let field = |name: &str| {
        composition.lines().find_map(|line| line.strip_prefix(name).and_then(|rest| rest.strip_prefix('\t'))).unwrap_or("-").to_string()
    };
    let code_keys: Vec<(&str, &str)> = composition.lines().filter_map(|line| line.strip_prefix("code_key\t")).filter_map(|rest| rest.split_once('\t')).collect();
    let [(entry, code_key)] = code_keys.as_slice() else {
        return Err(format!("组成清单里要恰好一行 code_key，读到 {} 行", code_keys.len()));
    };
    if !entry.ends_with(SEGMENT_FOUR_ENTRY_SUFFIX) {
        return Err(format!("组成清单的入口是 {entry}：第四段的项从 segment_four_item_lines 进（登记修订 4 之一）"));
    }
    if code_key.is_empty() || !code_key.chars().all(|character| character.is_ascii_hexdigit()) {
        return Err(format!("组成清单里的代码键不是十六进制：{code_key}"));
    }
    Ok(SegmentFourLibrary { store, code_key: (*code_key).to_string(), composition: composition.to_string(), judged: field("judged"), host: field("host") })
}

/// 项名：序号加几何，只留字母、数字与连字符。只给 `index.tsv` 当索引（数键对不上的项、给人看），不进键、读回不经它。
fn segment_four_item_name(position: usize, item: SegmentFourItem) -> String {
    let geometry: String = item.label().chars().map(|character| if character.is_ascii_alphanumeric() || character == '-' { character } else { '_' }).collect();
    format!("{position:04}-{geometry}")
}

/// 项键（登记修订 4 之五）：sha256（`code\t<代码键>\nparameters\t<{item:?}>\n`）。参数取 `derive(Debug)` 打的全部字段值，不用 `label()` 拼的串；
/// 序号、项名、`label()`、库路径、线程数、分片、日期都不进。
fn segment_four_item_key(code_key: &str, item: SegmentFourItem) -> String {
    let code_line = format!("code\t{code_key}\n");
    let parameters_line = format!("parameters\t{item:?}\n");
    sha256_hex([code_line, parameters_line].concat().as_bytes())
}

/// 项文件的第一行：项键与代码键（读回时两个都要等）。
fn segment_four_item_header(item_key: &str, code_key: &str) -> String {
    format!("key item_key={item_key} code_key={code_key}")
}

/// 读回一个项文件（库里那一条的内容，不在是 None）：第一行是给的键行，第二、三行是 parameters 与 meta（跳过、不比），末行 `done` 报的行数对得上才算数；
/// 不然交回 None（现判）。
fn read_back_the_item(contents: Option<String>, header: &str) -> Option<Vec<String>> {
    let contents = contents?;
    let mut lines: Vec<String> = contents.lines().map(str::to_string).collect();
    if lines.first().map(String::as_str) != Some(header) {
        return None;
    }
    let done = lines.pop()?;
    if lines.len() < 3 || !lines[1].starts_with("parameters ") || !lines[2].starts_with("meta ") {
        return None;
    }
    let item_lines: Vec<String> = lines.into_iter().skip(3).collect();
    (done == format!("done lines={}", item_lines.len())).then_some(item_lines)
}

/// 项文件的全文（登记修订 4 之五）：键行、parameters、meta（判定日期、跑法、机器；只当记录）、产物行、`done lines=<行数>`。
fn segment_four_item_file_text(header: &str, item: SegmentFourItem, meta: &str, item_lines: &[String]) -> String {
    let mut text = format!("{header}\nparameters {item:?}\n{meta}\n");
    for line in item_lines {
        text.push_str(line);
        text.push('\n');
    }
    text.push_str(&format!("done lines={}\n", item_lines.len()));
    text
}

/// 项库的 `index.tsv`：`<项名>\t<项键>`，上一趟拼完时写的；不在、读不出交空表。只给人看与数键对不上的项，读回不经它。
fn read_the_index(library: &SegmentFourLibrary) -> BTreeMap<String, String> {
    library
        .store
        .read(&library.index_path())
        .unwrap_or_default()
        .lines()
        .filter_map(|line| line.split_once('\t'))
        .map(|(name, item_key)| (name.to_string(), item_key.to_string()))
        .collect()
}

/// 键对不上的项：这一趟的项名在上一趟的 `index.tsv` 里有、记的项键与这一趟的不等（新进清单的项不算）。
fn count_mismatched_keys(previous_index: &BTreeMap<String, String>, current: &[(String, String)]) -> u64 {
    length_as_u64(current.iter().filter(|(name, item_key)| previous_index.get(name).is_some_and(|previous_key| previous_key != item_key)).count())
}

/// 项文件（库里那一条的内容）第一行里记的代码键；不在、第一行认不出交 None。
fn code_key_recorded_in(contents: Option<String>) -> Option<String> {
    let contents = contents?;
    let header = contents.lines().next()?;
    header.split(' ').find_map(|field| field.strip_prefix("code_key=")).map(str::to_string)
}

/// 「变了的单元」那一句最多点名几个单元，多出的报个数。
const CHANGED_UNITS_NAMED_AT_MOST: usize = 20;

/// 两份组成清单比：单元行按（入口，种类，限定名，文件）对，改了、新进、出去的各列限定名（合起来至多 20 个，多出的报个数）；
/// 单元都没变的，报工具链行与依赖行各变了几行。
fn changed_units_between(previous: &str, current: &str) -> String {
    type UnitIdentity = (String, String, String, String);
    let units_of = |text: &str| -> BTreeMap<UnitIdentity, String> {
        text.lines()
            .filter_map(|line| {
                let columns: Vec<&str> = line.split('\t').collect();
                let [kind_column, entry, kind, qualified, location, fingerprint] = columns.as_slice() else { return None };
                if *kind_column != "unit" {
                    return None;
                }
                let file = location.rsplit_once(':').map_or(*location, |(file, _span)| file);
                Some((((*entry).to_string(), (*kind).to_string(), (*qualified).to_string(), file.to_string()), (*fingerprint).to_string()))
            })
            .collect()
    };
    let lines_with = |text: &str, prefix: &str| -> BTreeSet<String> { text.lines().filter(|line| line.starts_with(prefix)).map(str::to_string).collect() };
    let (before, after) = (units_of(previous), units_of(current));
    let changed: Vec<&str> = before.iter().filter(|(identity, fingerprint)| after.get(*identity).is_some_and(|now| now != *fingerprint)).map(|(identity, _)| identity.2.as_str()).collect();
    let entered: Vec<&str> = after.keys().filter(|identity| !before.contains_key(*identity)).map(|identity| identity.2.as_str()).collect();
    let left: Vec<&str> = before.keys().filter(|identity| !after.contains_key(*identity)).map(|identity| identity.2.as_str()).collect();
    if changed.is_empty() && entered.is_empty() && left.is_empty() {
        let differing = |prefix: &str| lines_with(previous, prefix).symmetric_difference(&lines_with(current, prefix)).count();
        return format!("单元都没变：工具链行变了 {} 行、依赖行变了 {} 行", differing("toolchain\t"), differing("dependency\t"));
    }
    let mut named = 0;
    let mut parts = Vec::new();
    for (label, names) in [("改了", &changed), ("新进", &entered), ("出去", &left)] {
        if names.is_empty() {
            continue;
        }
        let room = CHANGED_UNITS_NAMED_AT_MOST.saturating_sub(named);
        let shown: Vec<&str> = names.iter().take(room).copied().collect();
        named += shown.len();
        let more = if names.len() > shown.len() { format!("（另 {} 个）", names.len() - shown.len()) } else { String::new() };
        parts.push(format!("{label} {}{more}", shown.join("、")));
    }
    format!("变了的单元：{}", parts.join("；"))
}

/// 键对不上时那一句（登记修订 4 之五「成功句」）：找上一趟那几项记的代码键，与这一趟的不同就比两份组成清单。
fn changed_units_sentence(library: &SegmentFourLibrary, previous_index: &BTreeMap<String, String>, current: &[(String, String)]) -> String {
    let mut previous_code_keys = BTreeSet::new();
    let mut unreadable = 0_u64;
    for (name, item_key) in current {
        let Some(previous_key) = previous_index.get(name).filter(|previous_key| *previous_key != item_key) else { continue };
        match code_key_recorded_in(library.store.read(&library.item_path(previous_key))) {
            Some(code_key) => {
                previous_code_keys.insert(code_key);
            }
            None => unreadable += 1,
        }
    }
    let Some(previous_code_key) = previous_code_keys.iter().find(|code_key| **code_key != library.code_key) else {
        return if unreadable > 0 {
            format!("上一趟的项文件不在库里（{unreadable} 项），说不出变了哪些单元")
        } else {
            "代码键没变：键对不上的那几项是项的参数变了".to_string()
        };
    };
    match library.store.read(&library.composition_path(previous_code_key)) {
        Some(previous) => changed_units_between(&previous, &library.composition),
        None => format!("上一趟的组成清单 codes/{previous_code_key}.txt 不在库里，说不出变了哪些单元"),
    }
}

/// 一趟第四段算下来的项：每项的行、现判数、复用数、键对不上的项数与那一句（没有键对不上的是 None）。
pub(super) struct SegmentFourItemsOutcome {
    pub(super) item_lines: Vec<Vec<String>>,
    pub(super) computed: u64,
    pub(super) reused: u64,
    pub(super) mismatched: u64,
    pub(super) changed_units: Option<String>,
}

/// 算（或按项键读回）给定的项：组成清单存进 `codes/`；按项键找 `items/<项键>.out`，读不回的现判、
/// 写回库里（服务确认落盘才算这一项做完）；拼完重写 `index.tsv`。
pub(super) fn compute_or_reuse_the_items(items: &[SegmentFourItem], threads: usize, options: &SegmentFourOptions) -> SegmentFourItemsOutcome {
    let library = &options.library;
    let previous_index = read_the_index(library);
    library.store.write(&library.composition_path(&library.code_key), &library.composition);
    let meta = format!("meta judged={} how=full host={}", library.judged, library.host);
    let positions: Vec<usize> = (0..items.len()).collect();
    let outcomes = run_in_parallel(&positions, threads, |position| {
        let item = items[position];
        let name = segment_four_item_name(position, item);
        let item_key = segment_four_item_key(&library.code_key, item);
        let path = library.item_path(&item_key);
        if let Some(item_lines) = read_back_the_item(library.store.read(&path), &segment_four_item_header(&item_key, &library.code_key)) {
            return (item_lines, true, (name, item_key));
        }
        let item_lines = segment_four_item_lines(item);
        let text = segment_four_item_file_text(&segment_four_item_header(&item_key, &library.code_key), item, &meta, &item_lines);
        library.store.write(&path, &text);
        (item_lines, false, (name, item_key))
    });
    let computed = length_as_u64(outcomes.iter().filter(|(_, reused, _)| !reused).count());
    let reused = length_as_u64(outcomes.iter().filter(|(_, reused, _)| *reused).count());
    let current: Vec<(String, String)> = outcomes.iter().map(|(_, _, named_key)| named_key.clone()).collect();
    let mismatched = count_mismatched_keys(&previous_index, &current);
    let changed_units = (mismatched > 0).then(|| changed_units_sentence(library, &previous_index, &current));
    let index: String = current.iter().map(|(name, item_key)| format!("{name}\t{item_key}\n")).collect();
    library.store.write(&library.index_path(), &index);
    SegmentFourItemsOutcome { item_lines: outcomes.into_iter().map(|(item_lines, _, _)| item_lines).collect(), computed, reused, mismatched, changed_units }
}

/// 产物行的字段：按空格切开的 `键=值`（值里没有空格；`name=` 后面那一段也在里面）。
pub(super) fn fields_of(line: &str) -> BTreeMap<&str, &str> {
    line.split(' ').filter_map(|field| field.split_once('=')).collect()
}

pub(super) fn number_field(fields: &BTreeMap<&str, &str>, key: &str) -> u64 {
    fields.get(key).and_then(|value| value.parse::<u64>().ok()).unwrap_or_else(|| panic!("产物行带数字字段 {key}：{fields:?}"))
}

/// 从轨迹行（`trajectory_lines`、`difference_lines` 写的）读回一个量的轨迹。
pub(super) fn quantity_trajectory_from(fields: &BTreeMap<&str, &str>, prefix: &str) -> QuantityTrajectory {
    QuantityTrajectory {
        scans: number_field(fields, "scans"),
        peak: number_field(fields, &format!("{prefix}_peak")),
        peak_at_publish: number_field(fields, &format!("{prefix}_peak_at")),
        positive_scans: number_field(fields, &format!("{prefix}_positive_scans")),
        last: number_field(fields, &format!("{prefix}_last")),
    }
}

#[cfg(test)]
#[path = "item_library_and_reuse_tests.rs"]
mod tests;
