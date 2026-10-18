//! 一组入口的复用键与组成清单，连同写进构建输出目录的那份表（`singlefs-checker-tier` 的构建脚本按登记的判器入口逐组算、写进
//! `OUT_DIR`，库经 `include_str!` 读回，跑时只用编进去的这一份、不读盘上的树；设计 3.8「build.rs 做什么」）。
//!
//! 一组 = 同一个目标里的几个入口 fn（[`crate::closure::compute_closure_of_entries`] 一次收），开哪几个特性、工具链行
//! （[`crate::build_toolchain::toolchain_lines_of_the_build`]）与 `Cargo.lock`（按闭包收窄，[`crate::lock_file::narrowed_lock_lines`]）。
//! 键就是 [`crate::closure::reuse_key`] 那一把：与命令行 `unit-graph key <根> <入口>… --target … --cfg feature=… --toolchain-of-the-build …
//! --lock-file Cargo.lock` 算的逐字相同。
//!
//! 表的写法（一行一项、制表符隔开；组名与限定名、文件:行只当索引，不进键）：
//!   `# singlefs entry group keys`                                   头一行
//!   `toolchain\t<工具链行>`                                          这一趟拼的工具链行，逐行
//!   `group\t<组名>\tkeyed\t<键十六进制>`                              算出了键
//!   `group\t<组名>\trefused`                                         闭包里有拒算（拒算行另列）
//!   `group\t<组名>\tnot-compiled\t<原因>`                            组里的入口在这组特性下一个都不编进来（例：`gpu` 特性关着时的 GPU 判器）
//!   `not-compiled-entry\t<组名>\t<入口>`                             组里这个入口在这组特性下不编进来，键照其余的入口算
//!   `unit\t<组名>\t<种类>\t<限定名>\t<文件>:<起>-<止>\t<指纹十六进制>`  组成清单（Rust、WGSL、数据单元，数据单元的限定名是数据文件路径）
//!   `refusal\t<组名>\t<形态>\t<文件>:<行>\t<原样片段>`               拒算行
//!   `watched\t<从仓根起的路径>`                                       构建脚本要看的文件（改了就重算），只给构建脚本打 `rerun-if-changed`

use std::collections::BTreeSet;

use crate::closure::{compute_closure_of_entries, reuse_key, ClosureRequest, EntryError};
use crate::closure::{ExternalKeyInputs, RequestedCondition};
use crate::conditional_compilation::target_triple_of_toolchain_lines;
use crate::lock_file::{narrowed_lock_lines, LockedPackage};
use crate::output::{closure_lines, refusal_line};
use crate::rule_switches::RuleSwitches;
use crate::sha256::{Sha256Digest, DIGEST_BYTES};
use crate::workspace::{TargetKind, TargetSelector, Workspace};

/// 表的头一行。
pub const ENTRY_GROUP_TABLE_HEADER: &str = "# singlefs entry group keys";

/// 一组入口：组名（只当索引）、目标、入口 fn 路径（头一个定 crate）、开的特性（不开默认特性：要默认特性就把 `default` 写进来）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntryGroupRequest {
    pub group_name: String,
    pub target: TargetSelector,
    pub entry_paths: Vec<String>,
    pub features: Vec<String>,
}

/// 一组算下来是什么。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EntryGroupVerdict {
    Keyed {
        key: Sha256Digest,
    },
    /// 闭包里有拒算：不交键，交拒算行（`refusal_line` 的写法）。
    Refused {
        refusal_lines: Vec<String>,
    },
    /// 入口在这组特性与平台下不编进来。
    NotCompiled {
        reason: String,
    },
}

/// 一组的结果：结论、在这组特性下不编进来而没收的入口、组成清单（`closure_lines` 的单元行写法，不带键行与拒算行）、要看的文件（从仓根起的路径）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntryGroupKey {
    pub group_name: String,
    pub verdict: EntryGroupVerdict,
    pub entries_not_compiled: Vec<String>,
    pub composition_lines: Vec<String>,
    pub watched_files: BTreeSet<String>,
}

/// 收一组入口、算键。组里在这组特性下不编进来的入口不收、记进 `entries_not_compiled`，键照其余的入口算；一个都编不进来的交
/// [`EntryGroupVerdict::NotCompiled`]。
///
/// # Errors
/// 入口找不到、不唯一、写法不对、不在同一个 crate 里（登记表写错了，调用方应当停下）。
pub fn entry_group_key(
    workspace: &Workspace,
    request: &EntryGroupRequest,
    toolchain_lines: &[String],
    locked_packages: &[LockedPackage],
    switches: &RuleSwitches,
) -> Result<EntryGroupKey, EntryError> {
    if request.entry_paths.is_empty() {
        return Err(EntryError::MalformedEntryPath(format!(
            "组「{}」没有入口",
            request.group_name
        )));
    }
    let target_triple = target_triple_of_toolchain_lines(toolchain_lines)
        .ok()
        .flatten()
        .unwrap_or_default();
    let mut compiled_entries = request.entry_paths.clone();
    let mut entries_not_compiled = Vec::new();
    // 每一趟要么交出闭包，要么去掉一个不编进来的入口：至多入口数加一趟
    let mut outcome = loop {
        let Some((first_entry, additional_entries)) = compiled_entries.split_first() else {
            return Ok(EntryGroupKey {
                group_name: request.group_name.clone(),
                verdict: EntryGroupVerdict::NotCompiled {
                    reason: format!(
                        "组里的入口在这组特性下都不编进来：{}",
                        entries_not_compiled.join("、")
                    ),
                },
                entries_not_compiled,
                composition_lines: Vec::new(),
                watched_files: BTreeSet::new(),
            });
        };
        let closure_request = ClosureRequest {
            entry_path: first_entry.clone(),
            target: request.target.clone(),
            requested_conditions: request
                .features
                .iter()
                .map(|feature| RequestedCondition::Feature(feature.clone()))
                .collect(),
            uses_default_features: false,
            target_triple: target_triple.clone(),
        };
        match compute_closure_of_entries(workspace, &closure_request, additional_entries, switches)
        {
            Ok(outcome) => break outcome,
            Err(EntryError::EntryInactive(path)) => {
                compiled_entries.retain(|entry| *entry != path);
                entries_not_compiled.push(path);
            }
            Err(error) => return Err(error),
        }
    };
    let lock_lines = match narrowed_lock_lines(workspace, &outcome, locked_packages) {
        Ok(lines) => lines,
        Err(refusals) => {
            outcome.refusals.extend(refusals);
            outcome.refusals.sort();
            outcome.refusals.dedup();
            Vec::new()
        }
    };
    let key = reuse_key(
        workspace,
        &outcome,
        &ExternalKeyInputs {
            toolchain_lines: toolchain_lines.to_vec(),
            lock_lines,
        },
    );
    let composition_lines: Vec<String> = closure_lines(workspace, &outcome, None)
        .into_iter()
        .filter(|line| !line.starts_with("refused\t"))
        .collect();
    let verdict = match key {
        Ok(key) => EntryGroupVerdict::Keyed { key },
        Err(refusals) => EntryGroupVerdict::Refused {
            refusal_lines: refusals
                .iter()
                .map(|refusal| refusal_line(workspace, refusal))
                .collect(),
        },
    };
    Ok(EntryGroupKey {
        group_name: request.group_name.clone(),
        verdict,
        entries_not_compiled,
        composition_lines,
        watched_files: watched_files_of_the_closure(workspace, &outcome),
    })
}

/// 改了就要重算的文件：闭包单元所在 crate 的库目标的全部文件、入口目标的全部文件、读到的 `.wgsl` 与数据文件。
/// 闭包外的文件也看：别处新写的 trait impl、新声明的同名单元会改闭包（R-隐式、R-裸名），只看闭包里的文件会漏。
fn watched_files_of_the_closure(
    workspace: &Workspace,
    outcome: &crate::closure::ClosureOutcome,
) -> BTreeSet<String> {
    let crates_of_the_closure: BTreeSet<usize> = outcome
        .units
        .iter()
        .map(|unit_index| {
            workspace.files[workspace.units[unit_index.0].file.0]
                .crate_index
                .0
        })
        .collect();
    let files_of_the_closure: BTreeSet<usize> = outcome
        .units
        .iter()
        .map(|unit_index| workspace.units[unit_index.0].file.0)
        .collect();
    let mut watched = BTreeSet::new();
    for (target_position, target) in workspace.targets.iter().enumerate() {
        let is_a_library_of_the_closure = target.kind == TargetKind::Library
            && crates_of_the_closure.contains(&target.crate_index.0);
        // 入口目标（集成测试、二进制）的全部文件也盯：闭包里有它的文件，它底下别的文件新写的同名单元、trait impl 会改闭包
        let holds_a_file_of_the_closure = workspace.files_of_target[target_position]
            .iter()
            .any(|file| files_of_the_closure.contains(&file.0));
        if is_a_library_of_the_closure || holds_a_file_of_the_closure {
            for file in &workspace.files_of_target[target_position] {
                watched.insert(workspace.files[file.0].relative_path.clone());
            }
        }
    }
    for unit_index in &outcome.units {
        watched.insert(
            workspace.files[workspace.units[unit_index.0].file.0]
                .relative_path
                .clone(),
        );
    }
    for wgsl_index in &outcome.wgsl_units {
        watched.insert(
            workspace.files[workspace.wgsl_units[wgsl_index.0].file.0]
                .relative_path
                .clone(),
        );
    }
    for data_unit in &outcome.data_units {
        watched.insert(data_unit.path_display.clone());
    }
    watched
}

/// 把几组的结果与工具链行写成表（文件头那一种写法）。
#[must_use]
pub fn entry_group_table_text(toolchain_lines: &[String], groups: &[EntryGroupKey]) -> String {
    let mut lines = vec![ENTRY_GROUP_TABLE_HEADER.to_string()];
    lines.extend(
        toolchain_lines
            .iter()
            .map(|line| format!("toolchain\t{}", line.replace(['\t', '\n'], " "))),
    );
    let mut watched = BTreeSet::new();
    for group in groups {
        let name = &group.group_name;
        match &group.verdict {
            EntryGroupVerdict::Keyed { key } => {
                lines.push(format!("group\t{name}\tkeyed\t{}", key.hexadecimal()));
            }
            EntryGroupVerdict::Refused { refusal_lines } => {
                lines.push(format!("group\t{name}\trefused"));
                lines.extend(refusal_lines.iter().map(|refusal| {
                    format!(
                        "refusal\t{name}\t{}",
                        refusal.strip_prefix("refused\t").unwrap_or(refusal)
                    )
                }));
            }
            EntryGroupVerdict::NotCompiled { reason } => {
                lines.push(format!(
                    "group\t{name}\tnot-compiled\t{}",
                    reason.replace(['\t', '\n'], " ")
                ));
            }
        }
        lines.extend(
            group
                .entries_not_compiled
                .iter()
                .map(|entry| format!("not-compiled-entry\t{name}\t{entry}")),
        );
        lines.extend(
            group
                .composition_lines
                .iter()
                .map(|unit| format!("unit\t{name}\t{unit}")),
        );
        watched.extend(group.watched_files.iter().cloned());
    }
    lines.extend(watched.iter().map(|path| format!("watched\t{path}")));
    let mut text = lines.join("\n");
    text.push('\n');
    text
}

/// 表里读回的一个组成单元：种类、限定名、文件:起-止、指纹。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompositionUnit {
    pub kind: String,
    pub qualified_name: String,
    pub location: String,
    pub fingerprint: [u8; DIGEST_BYTES],
}

/// 表里读回的一组。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReadBackEntryGroupVerdict {
    Keyed { key: [u8; DIGEST_BYTES] },
    Refused { refusal_lines: Vec<String> },
    NotCompiled { reason: String },
}

/// 表里读回的一组：组名、结论、没收的入口、组成清单。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadBackEntryGroup {
    pub group_name: String,
    pub verdict: ReadBackEntryGroupVerdict,
    pub entries_not_compiled: Vec<String>,
    pub composition: Vec<CompositionUnit>,
}

/// 读回的整张表：工具链行、各组（按表里的次序）、要看的文件。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadBackEntryGroupTable {
    pub toolchain_lines: Vec<String>,
    pub groups: Vec<ReadBackEntryGroup>,
    pub watched_files: Vec<String>,
}

fn digest_of_hexadecimal(text: &str) -> Option<[u8; DIGEST_BYTES]> {
    if text.len() != DIGEST_BYTES * 2 || !text.is_ascii() {
        return None;
    }
    let mut digest = [0u8; DIGEST_BYTES];
    for (position, byte) in digest.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[position * 2..position * 2 + 2], 16).ok()?;
    }
    Some(digest)
}

/// [`entry_group_table_text`] 的逆。
///
/// # Errors
/// 头一行不对、某一行认不出（交那一行）、`unit` 或 `refusal` 行点名的组前面没有 `group` 行、同一组出现两次。
pub fn parse_entry_group_table(text: &str) -> Result<ReadBackEntryGroupTable, String> {
    let mut lines = text.lines();
    if lines.next() != Some(ENTRY_GROUP_TABLE_HEADER) {
        return Err(format!("头一行不是「{ENTRY_GROUP_TABLE_HEADER}」"));
    }
    let mut table = ReadBackEntryGroupTable {
        toolchain_lines: Vec::new(),
        groups: Vec::new(),
        watched_files: Vec::new(),
    };
    for line in lines.filter(|line| !line.is_empty()) {
        let fields: Vec<&str> = line.split('\t').collect();
        let group_position = |name: &str, table_read_so_far: &ReadBackEntryGroupTable| {
            table_read_so_far
                .groups
                .iter()
                .position(|group| group.group_name == name)
        };
        match fields.as_slice() {
            ["toolchain", toolchain_line] => {
                table.toolchain_lines.push((*toolchain_line).to_string())
            }
            ["watched", path] => table.watched_files.push((*path).to_string()),
            ["group", name, verdict @ ..] => {
                if group_position(name, &table).is_some() {
                    return Err(format!("组「{name}」出现了两次：{line}"));
                }
                let verdict = match verdict {
                    ["keyed", key] => ReadBackEntryGroupVerdict::Keyed {
                        key: digest_of_hexadecimal(key)
                            .ok_or_else(|| format!("键不是 64 位十六进制：{line}"))?,
                    },
                    ["refused"] => ReadBackEntryGroupVerdict::Refused {
                        refusal_lines: Vec::new(),
                    },
                    ["not-compiled", reason] => ReadBackEntryGroupVerdict::NotCompiled {
                        reason: (*reason).to_string(),
                    },
                    _ => return Err(format!("认不出的 group 行：{line}")),
                };
                table.groups.push(ReadBackEntryGroup {
                    group_name: (*name).to_string(),
                    verdict,
                    entries_not_compiled: Vec::new(),
                    composition: Vec::new(),
                });
            }
            ["not-compiled-entry", name, entry] => {
                let position = group_position(name, &table).ok_or_else(|| {
                    format!("not-compiled-entry 行点名的组前面没有 group 行：{line}")
                })?;
                table.groups[position]
                    .entries_not_compiled
                    .push((*entry).to_string());
            }
            ["unit", name, kind, qualified_name, location, fingerprint] => {
                let position = group_position(name, &table)
                    .ok_or_else(|| format!("unit 行点名的组前面没有 group 行：{line}"))?;
                table.groups[position].composition.push(CompositionUnit {
                    kind: (*kind).to_string(),
                    qualified_name: (*qualified_name).to_string(),
                    location: (*location).to_string(),
                    fingerprint: digest_of_hexadecimal(fingerprint)
                        .ok_or_else(|| format!("指纹不是 64 位十六进制：{line}"))?,
                });
            }
            ["refusal", name, rest @ ..] => {
                let position = group_position(name, &table)
                    .ok_or_else(|| format!("refusal 行点名的组前面没有 group 行：{line}"))?;
                match &mut table.groups[position].verdict {
                    ReadBackEntryGroupVerdict::Refused { refusal_lines } => {
                        refusal_lines.push(format!("refused\t{}", rest.join("\t")));
                    }
                    ReadBackEntryGroupVerdict::Keyed { .. }
                    | ReadBackEntryGroupVerdict::NotCompiled { .. } => {
                        return Err(format!("refusal 行点名的组不是拒算的那一种：{line}"))
                    }
                }
            }
            _ => return Err(format!("认不出的行：{line}")),
        }
    }
    Ok(table)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_text_reads_back_every_group_unit_refusal_and_watched_file() {
        let groups = vec![
            EntryGroupKey {
                group_name: "gpu_judge".to_string(),
                verdict: EntryGroupVerdict::Keyed {
                    key: Sha256Digest([0xab; DIGEST_BYTES]),
                },
                entries_not_compiled: vec!["crate_a::gpu_only".to_string()],
                composition_lines: vec![format!(
                    "fn\tcrate_a::judge\tcrates/a/src/lib.rs:3-9\t{}",
                    "cd".repeat(DIGEST_BYTES)
                )],
                watched_files: ["crates/a/src/lib.rs".to_string()].into_iter().collect(),
            },
            EntryGroupKey {
                group_name: "recording".to_string(),
                verdict: EntryGroupVerdict::Refused {
                    refusal_lines: vec![
                        "refused\tcurrent-exe\tcrates/a/src/lib.rs:12\tcurrent_exe ( )".to_string(),
                    ],
                },
                entries_not_compiled: Vec::new(),
                composition_lines: Vec::new(),
                watched_files: BTreeSet::new(),
            },
            EntryGroupKey {
                group_name: "gpu_facts_verifier".to_string(),
                verdict: EntryGroupVerdict::NotCompiled {
                    reason: "入口 a::verify 在这组特性下不编进来".to_string(),
                },
                entries_not_compiled: vec!["a::verify".to_string()],
                composition_lines: Vec::new(),
                watched_files: BTreeSet::new(),
            },
        ];
        let text = entry_group_table_text(&["host: x86_64-unknown-linux-gnu".to_string()], &groups);
        let table = parse_entry_group_table(&text).expect("自己写的表读得回");
        assert_eq!(table.toolchain_lines, ["host: x86_64-unknown-linux-gnu"]);
        assert_eq!(table.watched_files, ["crates/a/src/lib.rs"]);
        assert_eq!(
            table.groups[0].verdict,
            ReadBackEntryGroupVerdict::Keyed {
                key: [0xab; DIGEST_BYTES]
            }
        );
        assert_eq!(
            table.groups[0].composition[0].qualified_name,
            "crate_a::judge"
        );
        assert_eq!(table.groups[0].entries_not_compiled, ["crate_a::gpu_only"]);
        assert_eq!(
            table.groups[0].composition[0].location,
            "crates/a/src/lib.rs:3-9"
        );
        assert_eq!(
            table.groups[1].verdict,
            ReadBackEntryGroupVerdict::Refused {
                refusal_lines: vec![
                    "refused\tcurrent-exe\tcrates/a/src/lib.rs:12\tcurrent_exe ( )".to_string()
                ]
            }
        );
        assert!(matches!(
            table.groups[2].verdict,
            ReadBackEntryGroupVerdict::NotCompiled { .. }
        ));
    }

    #[test]
    fn a_unit_line_before_its_group_and_a_malformed_key_are_refused() {
        assert!(parse_entry_group_table(&format!(
            "{ENTRY_GROUP_TABLE_HEADER}\nunit\tlost\tfn\ta::b\tf:1-2\t{}\n",
            "00".repeat(DIGEST_BYTES)
        ))
        .is_err());
        assert!(parse_entry_group_table(&format!(
            "{ENTRY_GROUP_TABLE_HEADER}\ngroup\tg\tkeyed\tzz\n"
        ))
        .is_err());
        assert!(parse_entry_group_table("# 别的表\n").is_err());
    }
}
