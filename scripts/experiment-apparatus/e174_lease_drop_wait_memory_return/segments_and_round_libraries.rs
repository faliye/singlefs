//! 段与各段的格、待跑任务的次序、轮库的读写与组成、与上一轮的单元差异。

use super::*;

// ============================== 清单：格、次序、续跑（登记第五节「次序」「续跑、分段」） ==============================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Segment {
    First,
    Second,
}

impl Segment {
    pub(super) fn label(self) -> &'static str {
        match self {
            Segment::First => "first",
            Segment::Second => "second",
        }
    }

    pub(super) fn from_label(label: &str) -> Option<Segment> {
        [Segment::First, Segment::Second].into_iter().find(|segment| segment.label() == label)
    }
}

/// 第一段的全部格。在飞·中是跑前登记「十二、修订」补的一条臂，每个登记里用在飞·轻的组都另有一份在飞·中。
pub(super) fn first_segment_cells() -> Vec<CellKey> {
    let mut cells = Vec::new();
    for card in Card::EVERY_CARD {
        for arm in [Arm::WaitLatest, Arm::Poll] {
            let mut push = |group: Group, size_mebibytes: u64, liveness: Liveness, method: Method| {
                cells.push(CellKey { group, card, size_mebibytes, liveness, method, arm });
            };
            for size in HOLD_CONTROL_SIZES_MEBIBYTES {
                for liveness in [Liveness::InFlightMedium, Liveness::InFlightLight] {
                    push(Group::HoldControl, size, liveness, Method::OneBuild);
                }
            }
            for size in LEASE_SIZES_MEBIBYTES {
                for liveness in Liveness::EVERY_LIVENESS {
                    push(Group::MainGrid, size, liveness, Method::OneBuild);
                }
            }
            for size in PASS_BY_PASS_SIZES_MEBIBYTES {
                push(Group::PassByPassGrid, size, Liveness::Done, Method::PassByPass);
            }
            for size in COEXISTING_SIZES_MEBIBYTES {
                for liveness in [Liveness::InFlightMedium, Liveness::InFlightLight] {
                    push(Group::CoexistingLongLease, size, liveness, Method::OneBuild);
                }
            }
        }
        for size in LONG_WINDOW_SIZES_MEBIBYTES {
            for liveness in [Liveness::InFlightMedium, Liveness::InFlightLight] {
                cells.push(CellKey {
                    group: Group::LongWindow,
                    card,
                    size_mebibytes: size,
                    liveness,
                    method: Method::OneBuild,
                    arm: Arm::WaitLatest,
                });
            }
        }
    }
    cells
}

/// 第二段的全部格：同一设备几条线程共用，三条臂。
pub(super) fn second_segment_cells() -> Vec<CellKey> {
    let mut cells = Vec::new();
    for card in Card::EVERY_CARD {
        for size_mebibytes in SHARED_DEVICE_SIZES_MEBIBYTES {
            for liveness in [Liveness::InFlightMedium, Liveness::InFlightLight] {
                for arm in [Arm::WaitLatest, Arm::WaitOwnSubmission, Arm::Poll] {
                    cells.push(CellKey {
                        group: Group::SharedDevice,
                        card,
                        size_mebibytes,
                        liveness,
                        method: Method::OneBuild,
                        arm,
                    });
                }
            }
        }
    }
    cells
}

pub(super) fn cells_of_segment(segment: Segment, skip_light: bool) -> Vec<CellKey> {
    let cells = match segment {
        Segment::First => first_segment_cells(),
        Segment::Second => second_segment_cells(),
    };
    cells.into_iter().filter(|cell| !(skip_light && cell.liveness == Liveness::InFlightLight)).collect()
}

/// 阶段：不用在飞·轻的先跑，「扣住不丢」排最前（它不过整轮作废）；在飞·轻最后，它的 K9 在登记的数下必红，主 agent 可以不等它。
pub(super) fn stage_of(cell: CellKey) -> u32 {
    let group_rank = match cell.group {
        Group::HoldControl => 0,
        Group::MainGrid => 1,
        Group::PassByPassGrid => 2,
        Group::CoexistingLongLease => 3,
        Group::LongWindow => 4,
        Group::SharedDevice => 5,
    };
    if cell.liveness == Liveness::InFlightLight { 10 + group_rank } else { group_rank }
}

/// 种子 174 的确定性乱序：同一个键永远排同一个位置，续跑时不受已经跑了哪些的影响。
pub(super) fn shuffle_rank(slot_text: &str) -> u64 {
    let mut state = SHUFFLE_SEED;
    for byte in slot_text.bytes() {
        state = state.wrapping_mul(0x100_0000_01B3) ^ u64::from(byte);
    }
    state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    state = (state ^ (state >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    state = (state ^ (state >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    state ^ (state >> 31)
}

/// 次序键：阶段 → 乱序后的位置 → 两张卡交替（先后由位置的一位定）→ 两条臂（三条臂）按轮号轮换。
pub(super) fn job_order_key(job: &ChildJob, arm_count: usize) -> (u32, u64, u32, usize) {
    let cell = &job.cell;
    let slot_text = format!(
        "{} {} {} {} round={}",
        cell.group.label(),
        cell.size_mebibytes,
        cell.liveness.label(),
        cell.method.label(),
        job.round
    );
    let rank = shuffle_rank(&slot_text);
    let card_index = Card::EVERY_CARD.iter().position(|card| *card == cell.card).expect("卡在表里");
    let card_first = usize::try_from((rank >> 1) & 1).expect("一位");
    let arms: &[Arm] = if arm_count == 3 {
        &[Arm::WaitLatest, Arm::WaitOwnSubmission, Arm::Poll]
    } else {
        &[Arm::WaitLatest, Arm::Poll]
    };
    let arm_index = arms.iter().position(|arm| *arm == cell.arm).expect("臂在表里");
    let round_shift = usize::try_from(job.round).expect("轮号装得进 usize");
    (stage_of(*cell), rank, u32::try_from((card_index + card_first) % 2).expect("0 或 1"), (arm_index + round_shift) % arms.len())
}

/// 产物里已有的轮之外还要跑哪些：第 1..=5 轮没出现的；5 轮都出现了而有效的不足 5 个，补轮（轮号 6 起，一格最多补 3 次）。
pub(super) fn pending_jobs(cells: &[CellKey], product: &ProductRounds, arm_count: usize) -> Vec<ChildJob> {
    let required = ROUNDS_REQUIRED_PER_CELL;
    let mut pending = Vec::new();
    for cell in cells {
        let present: Vec<u32> = product.keys().filter(|job| job.cell == *cell).map(|job| job.round).collect();
        let complete = product
            .iter()
            .filter(|(job, data)| job.cell == *cell && data.is_complete(cell.lease_count()))
            .count();
        let first_pass_missing: Vec<u32> = (1..=required).filter(|round| !present.contains(round)).collect();
        if !first_pass_missing.is_empty() {
            pending.extend(first_pass_missing.into_iter().map(|round| ChildJob { cell: *cell, round }));
            continue;
        }
        let refills_done = u32::try_from(present.iter().filter(|round| **round > required).count()).expect("轮数装得进 u32");
        let missing = required.saturating_sub(u32::try_from(complete).expect("轮数装得进 u32"));
        let refills_now = missing.min(MAXIMUM_REFILL_ROUNDS_PER_CELL.saturating_sub(refills_done));
        let last_round = present.iter().copied().max().unwrap_or(required);
        pending.extend((1..=refills_now).map(|offset| ChildJob { cell: *cell, round: last_round + offset }));
    }
    pending.sort_by_key(|job| job_order_key(job, arm_count));
    pending
}

// ============================== 续跑的轮库（登记修订 2 之五：键 = 两个入口的代码键 + 格与轮号 + 驱动与卡） ==============================

/// SHA-256（FIPS 180-4）：续跑的轮键由它算（登记修订 2 之五）。独立手写，不与别的装置共用；单测钉空串与 "abc" 的标准值。
pub(super) fn sha256_hex(message: &[u8]) -> String {
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
    let message_bits = u64::try_from(message.len()).expect("消息长度装得进 u64") * 8;
    let mut padded = message.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&message_bits.to_be_bytes());
    for block in padded.chunks_exact(64) {
        let mut schedule = [0_u32; 64];
        for (word_position, word) in block.chunks_exact(4).enumerate() {
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

/// 取键脚本 `scripts/experiments/device-code-key.py` 打的组成清单的第一行。
pub(super) const COMPOSITION_FORM_LINE: &str = "form\tdevice-code-key-v1";
/// 两个入口（登记修订 2 之一「读法写死」）：子进程那一侧按卡分两种，父进程告诉子进程哪一格的那一段一种。
const CARD_CHILD_ENTRY_SUFFIX: &str = "::run_child_on_the_card";
const MODEL_CHILD_ENTRY_SUFFIX: &str = "::run_child_on_the_model";
const ARGUMENTS_ENTRY_SUFFIX: &str = "::ChildJob::child_arguments";

/// 轮库（prune-server 里这个装置那一种，经 `singlefs_device_library` 转交）与这一趟的两把代码键；组成清单原文照存进 `codes/`（只给「变了的单元」那一句用），
/// 取键那天与机器只进轮文件的 meta 行。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RoundLibrary {
    pub(super) store: DeviceLibrary,
    pub(super) child_code_key: String,
    pub(super) arguments_code_key: String,
    pub(super) composition: String,
    pub(super) judged: String,
    pub(super) host: String,
}

/// 轮库里的相对键（`crates/singlefs-device-library/src/lib.rs` 文件头「相对键」）。
impl RoundLibrary {
    pub(super) fn round_path(&self, round_key: &str) -> String {
        format!("{round_key}.out")
    }

    pub(super) fn composition_path(&self, code_key: &str) -> String {
        format!("codes/{code_key}.txt")
    }

    pub(super) fn index_path(&self) -> String {
        "index.tsv".to_string()
    }
}

/// 读组成清单：第一行是 form，恰好两行 code_key：子进程入口（真卡轮是 `run_child_on_the_card`，`--simulated` 的轮是 `run_child_on_the_model`）与
/// `ChildJob::child_arguments`；judged、host 两行可缺（缺写 `-`）。
pub(super) fn round_library_from_composition(store: DeviceLibrary, composition: &str, simulated: bool) -> Result<RoundLibrary, String> {
    if composition.lines().next() != Some(COMPOSITION_FORM_LINE) {
        return Err("--code-key-file 不是 device-code-key.py 打的组成清单（第一行不是 form\tdevice-code-key-v1）".to_string());
    }
    let field = |name: &str| {
        composition.lines().find_map(|line| line.strip_prefix(name).and_then(|rest| rest.strip_prefix('\t'))).unwrap_or("-").to_string()
    };
    let code_keys: Vec<(&str, &str)> = composition.lines().filter_map(|line| line.strip_prefix("code_key\t")).filter_map(|rest| rest.split_once('\t')).collect();
    let child_suffix = if simulated { MODEL_CHILD_ENTRY_SUFFIX } else { CARD_CHILD_ENTRY_SUFFIX };
    let key_of = |suffix: &str| -> Result<String, String> {
        let found: Vec<&str> = code_keys.iter().filter(|(entry, _)| entry.ends_with(suffix)).map(|(_, code_key)| *code_key).collect();
        let [code_key] = found.as_slice() else {
            return Err(format!("组成清单里以 {suffix} 收尾的 code_key 要恰好一行，读到 {} 行（{}）", found.len(), if simulated { "--simulated" } else { "真卡" }));
        };
        if code_key.is_empty() || !code_key.chars().all(|character| character.is_ascii_hexdigit()) {
            return Err(format!("组成清单里的代码键不是十六进制：{code_key}"));
        }
        Ok((*code_key).to_string())
    };
    if code_keys.len() != 2 {
        return Err(format!("组成清单里要恰好两行 code_key（子进程入口与 ChildJob::child_arguments），读到 {} 行", code_keys.len()));
    }
    Ok(RoundLibrary {
        store,
        child_code_key: key_of(child_suffix)?,
        arguments_code_key: key_of(ARGUMENTS_ENTRY_SUFFIX)?,
        composition: composition.to_string(),
        judged: field("judged"),
        host: field("host"),
    })
}

/// 轮键（登记修订 2 之五）：sha256（`child\t<子进程代码键>\narguments\t<参数代码键>\nparameters\t<{cell:?}> round=<轮号>\ndevice\t<驱动与卡那一行>\n`）。
/// 格取 `derive(Debug)` 打的全部字段值，不用 `fields_text` 拼的串；产物路径、`--skip-light`、`--limit-jobs`、清单位置、乱序值、日期、机器都不进。
pub(super) fn round_key_of(child_code_key: &str, arguments_code_key: &str, job: &ChildJob, device_line: &str) -> String {
    let child_line = format!("child\t{child_code_key}\n");
    let arguments_line = format!("arguments\t{arguments_code_key}\n");
    let parameters_text = format!("{:?} round={}", job.cell, job.round);
    let parameters_line = format!("parameters\t{parameters_text}\n");
    let device = format!("device\t{device_line}\n");
    sha256_hex([child_line, arguments_line, parameters_line, device].concat().as_bytes())
}

/// 轮文件的第一行：轮键与两把代码键（读回时三个都要等）。
pub(super) fn round_header(round_key: &str, library: &RoundLibrary) -> String {
    format!("key round_key={round_key} child_key={} arguments_key={}", library.child_code_key, library.arguments_code_key)
}

/// 读回一个轮文件（库里那一条的内容，不在是 None）：第一行是给的键行，第二、三行是 parameters 与 meta（跳过、不比），末行 `done` 报的行数对得上才算数；
/// 交回这一轮原样的结果行（每行带换行）。
pub(super) fn read_back_the_round(contents: Option<String>, header: &str) -> Option<String> {
    let contents = contents?;
    let mut lines: Vec<&str> = contents.lines().collect();
    if lines.first().copied() != Some(header) {
        return None;
    }
    let done = lines.pop()?;
    if lines.len() < 3 || !lines[1].starts_with("parameters ") || !lines[2].starts_with("meta ") {
        return None;
    }
    let result_lines = &lines[3..];
    (done == format!("done lines={}", result_lines.len())).then(|| result_lines.iter().map(|line| format!("{line}\n")).collect())
}

/// 轮文件的全文（登记修订 2 之五）：键行、parameters（`key_text`）、meta（判定日期、跑法、机器；只当记录）、这一轮原样的结果行、`done lines=<行数>`。
pub(super) fn round_file_text(header: &str, job: &ChildJob, meta: &str, result_text: &str) -> String {
    let result_lines: Vec<&str> = result_text.lines().collect();
    let mut text = format!("{header}\nparameters {}\n{meta}\n", job.key_text());
    for line in &result_lines {
        text.push_str(line);
        text.push('\n');
    }
    text.push_str(&format!("done lines={}\n", result_lines.len()));
    text
}

/// 临时文件名的序号：同一个进程里写几份文件时各用各的临时文件（只写 `--output` 那一份产物；轮库不落本地文件）。
static TEMPORARY_FILE_SEQUENCE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// 原子落盘：同目录临时文件写完、fsync、改名，再 fsync 目录（目录不在先建）。
pub(super) fn write_text_atomically(final_path: &std::path::Path, text: &str) -> Result<(), String> {
    use std::io::Write;
    let failed = |error: std::io::Error| format!("写不了 {}：{error}", final_path.display());
    let directory = final_path.parent().filter(|parent| !parent.as_os_str().is_empty()).unwrap_or(std::path::Path::new("."));
    std::fs::create_dir_all(directory).map_err(failed)?;
    let file_name = final_path.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
    let sequence = TEMPORARY_FILE_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let temporary_path = directory.join(format!(".{file_name}.tmp-{}-{sequence}", std::process::id()));
    {
        let mut file = std::fs::File::create(&temporary_path).map_err(failed)?;
        file.write_all(text.as_bytes()).map_err(failed)?;
        file.sync_all().map_err(failed)?;
    }
    std::fs::rename(&temporary_path, final_path).map_err(failed)?;
    std::fs::File::open(directory).and_then(|handle| handle.sync_all()).map_err(failed)
}

/// 轮库的 `index.tsv`：`<key_text>\t<轮键>`；不在、读不出交空表。只给人看与数键对不上的轮，读回不经它。
pub(super) fn read_the_round_index(library: &RoundLibrary) -> BTreeMap<String, String> {
    library
        .store
        .read(&library.index_path())
        .unwrap_or_default()
        .lines()
        .filter_map(|line| line.split_once('\t'))
        .map(|(key_text, round_key)| (key_text.to_string(), round_key.to_string()))
        .collect()
}

/// 键对不上的轮：这一段的（格，轮号）在上一趟的 `index.tsv` 里有、记的轮键与这一趟的不等。
pub(super) fn count_mismatched_rounds(previous_index: &BTreeMap<String, String>, round_keys: &BTreeMap<ChildJob, String>) -> u64 {
    let mismatched = round_keys.iter().filter(|(job, round_key)| previous_index.get(&job.key_text()).is_some_and(|previous| previous != *round_key)).count();
    u64::try_from(mismatched).expect("轮数装得进 u64")
}

/// 轮文件（库里那一条的内容）第一行里记的两把代码键；不在、第一行认不出交 None。
fn code_keys_recorded_in(contents: Option<String>) -> Option<(String, String)> {
    let contents = contents?;
    let header = contents.lines().next()?;
    let field = |name: &str| header.split(' ').find_map(|word| word.strip_prefix(name)).map(str::to_string);
    Some((field("child_key=")?, field("arguments_key=")?))
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
    let lines_with = |text: &str, prefix: &str| -> std::collections::BTreeSet<String> { text.lines().filter(|line| line.starts_with(prefix)).map(str::to_string).collect() };
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

/// 键对不上时那一句（登记修订 2 之五「成功句」）：找上一趟那几轮记的两把代码键，有一把与这一趟的不同就比那一份组成清单。
pub(super) fn changed_units_sentence(library: &RoundLibrary, previous_index: &BTreeMap<String, String>, round_keys: &BTreeMap<ChildJob, String>) -> String {
    let mut previous_pairs = std::collections::BTreeSet::new();
    let mut unreadable = 0_u64;
    for (job, round_key) in round_keys {
        let Some(previous_key) = previous_index.get(&job.key_text()).filter(|previous_key| *previous_key != round_key) else { continue };
        match code_keys_recorded_in(library.store.read(&library.round_path(previous_key))) {
            Some(pair) => {
                previous_pairs.insert(pair);
            }
            None => unreadable += 1,
        }
    }
    let differing_code_key = previous_pairs.iter().find_map(|(child_key, arguments_key)| {
        if *child_key != library.child_code_key {
            Some(child_key.clone())
        } else if *arguments_key != library.arguments_code_key {
            Some(arguments_key.clone())
        } else {
            None
        }
    });
    let Some(previous_code_key) = differing_code_key else {
        return if unreadable > 0 {
            format!("上一趟的轮文件不在库里（{unreadable} 轮），说不出变了哪些单元")
        } else {
            "两把代码键都没变：键对不上的那几轮是格、轮号或驱动与卡那一行变了".to_string()
        };
    };
    match library.store.read(&library.composition_path(&previous_code_key)) {
        Some(previous) => changed_units_between(&previous, &library.composition),
        None => format!("上一趟的组成清单 codes/{previous_code_key}.txt 不在库里，说不出变了哪些单元"),
    }
}

/// 父进程这一趟的跑法（`run` 的参数里与轮库、子进程无关的那几样）。
pub(super) struct RoundPlan<'plan> {
    pub(super) segment: Segment,
    pub(super) cells: &'plan [CellKey],
    pub(super) arm_count: usize,
    pub(super) output_path: &'plan str,
    pub(super) skip_light: bool,
    pub(super) simulated: bool,
    pub(super) limit_jobs: Option<usize>,
}

/// 一趟 `run` 怎么收的尾。
#[derive(Debug, PartialEq, Eq)]
pub(super) enum RoundsEnding {
    /// 跑满 `--limit-jobs` 个子进程停下：没写产物，库里已经有这几轮。
    StoppedAtTheLimit(usize),
    /// 由库拼出产物写到了 `--output`；F1 停机的那一趟 `stopped_by` 是 `f1_hold_control`。
    Assembled { stopped_by: &'static str },
    /// `--output` 已在，内容与拼出的相同。
    AlreadyComplete,
    /// `--output` 已在，内容与拼出的不同：一个字节不动。
    OutputDiffers,
}

/// 一趟 `run` 的结局与成功句那几个数。
pub(super) struct RoundsOutcome {
    pub(super) ending: RoundsEnding,
    pub(super) computed: u64,
    pub(super) reused: u64,
    pub(super) mismatched: u64,
    pub(super) changed_units: Option<String>,
}

/// 写进 meta 行的跑法：段，带了的选项接在后面。
pub(super) fn round_how(plan: &RoundPlan<'_>) -> String {
    let mut how = format!("run-{}", plan.segment.label());
    if plan.skip_light {
        how.push_str(",skip-light");
    }
    if plan.simulated {
        how.push_str(",simulated");
    }
    if let Some(limit) = plan.limit_jobs {
        how.push_str(&format!(",limit-jobs-{limit}"));
    }
    how
}

/// 一轮的结果文本并进产物：`parse_product` 那一份读法，原文按（格，轮号）记下，拼产物时照原样写回。
pub(super) fn add_round(product: &mut ProductRounds, texts: &mut BTreeMap<ChildJob, String>, job: ChildJob, text: String) -> Result<(), String> {
    product.extend(parse_product(&text)?);
    texts.insert(job, text);
    Ok(())
}
