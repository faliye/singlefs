//! 依赖行收窄（设计 3.1「外部依赖解析结果：`Cargo.lock` 里入口链接集真引到的包，及传递依赖」）：读整份 `Cargo.lock`，
//! 只取闭包成员真点名的外部 crate（`use <外部 crate>::…`、路径 `<外部 crate>::…`、`extern crate <外部 crate>`）解析到的那个包，
//! 再顺 `Cargo.lock` 里的 `dependencies` 收它的传递依赖；每个包一行（名、版本、来源、checksum）进键。
//!
//! 外部 crate 是闭包单元所在 crate 的清单里不带 `path` 的依赖：代码里点它用依赖的键把 `-` 换成 `_`。点名的外部 crate 解析到哪个包：
//! 先在 `Cargo.lock` 里找那个工作区 crate 自己的包（名字是包名、没有 `source`），从它的 `dependencies` 里找这个依赖
//! （写成 `名` 或 `名 版本` 或 `名 版本 (来源)`），再按名与版本落到一个 `[[package]]`。落不到的拒算（`lock-dependency-unresolved`），不静默跳过。
//! 依赖改名（清单里的 `foo = { package = "bar", … }`）：代码里照键点它（`use foo::…`），`Cargo.lock` 里照包名 `bar` 找它
//! （与 `scripts/gate/lib_mutation_units.py` 的改名规则同义）。

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::closure::ClosureOutcome;
use crate::lexer::{token_text, TokenKind};
use crate::manifest::{parse_toml_entries, ManifestError, TomlValue};
use crate::units::{FileIndex, Refusal, RefusalForm, UnitIndex};
use crate::workspace::{CrateIndex, Workspace};

/// `Cargo.lock` 里的一个 `[[package]]`。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LockedPackage {
    pub name: String,
    pub version: String,
    pub source: Option<String>,
    pub checksum: Option<String>,
    /// 原文：`名`、`名 版本` 或 `名 版本 (来源)`。
    pub dependencies: Vec<String>,
}

/// `Cargo.lock` 读不了：TOML 写坏了，或某个 `[[package]]` 缺名字或版本。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LockFileError {
    Malformed(ManifestError),
    PackageWithoutNameOrVersion { package_position: usize },
}

impl fmt::Display for LockFileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LockFileError::Malformed(error) => write!(
                formatter,
                "Cargo.lock 第 {} 行读不了：{}",
                error.line, error.reason
            ),
            LockFileError::PackageWithoutNameOrVersion { package_position } => write!(
                formatter,
                "Cargo.lock 里从上往下数第 {package_position} 个 [[package]] 缺 name 或 version"
            ),
        }
    }
}

/// 读整份 `Cargo.lock`，交全部 `[[package]]`（按出现次序）。
///
/// # Errors
/// TOML 写坏了，或某个 `[[package]]` 缺名字或版本。
pub fn parse_lock_file(lock_text: &str) -> Result<Vec<LockedPackage>, LockFileError> {
    let entries = parse_toml_entries(lock_text).map_err(LockFileError::Malformed)?;
    let mut fields_by_occurrence: BTreeMap<usize, BTreeMap<String, TomlValue>> = BTreeMap::new();
    for entry in entries {
        if entry.table_path != ["package"] {
            continue;
        }
        if let [key] = entry.key_path.as_slice() {
            fields_by_occurrence
                .entry(entry.table_occurrence)
                .or_default()
                .insert(key.clone(), entry.value);
        }
    }
    let text_of = |fields: &BTreeMap<String, TomlValue>, key: &str| match fields.get(key) {
        Some(TomlValue::Text(text)) => Some(text.clone()),
        Some(
            TomlValue::Boolean(_)
            | TomlValue::Array(_)
            | TomlValue::InlineTable(_)
            | TomlValue::Unparsed(_),
        )
        | None => None,
    };
    let mut packages = Vec::new();
    for (position, fields) in fields_by_occurrence.values().enumerate() {
        let (Some(name), Some(version)) = (text_of(fields, "name"), text_of(fields, "version"))
        else {
            return Err(LockFileError::PackageWithoutNameOrVersion {
                package_position: position + 1,
            });
        };
        let dependencies = match fields.get("dependencies") {
            Some(TomlValue::Array(values)) => values
                .iter()
                .filter_map(|value| match value {
                    TomlValue::Text(text) => Some(text.clone()),
                    TomlValue::Boolean(_)
                    | TomlValue::Array(_)
                    | TomlValue::InlineTable(_)
                    | TomlValue::Unparsed(_) => None,
                })
                .collect(),
            Some(
                TomlValue::Text(_)
                | TomlValue::Boolean(_)
                | TomlValue::InlineTable(_)
                | TomlValue::Unparsed(_),
            )
            | None => Vec::new(),
        };
        packages.push(LockedPackage {
            name,
            version,
            source: text_of(fields, "source"),
            checksum: text_of(fields, "checksum"),
            dependencies,
        });
    }
    Ok(packages)
}

/// 一个单元里点名的路径根：`<名字> ::` 里的名字，前面不是 `::`（或前面的 `::` 是整条路径开头的 `::`）；外加 `extern crate <名字>`。
/// 交（名字，所在行）。
fn path_roots_of_unit(workspace: &Workspace, unit_index: UnitIndex) -> Vec<(String, u32)> {
    let unit = &workspace.units[unit_index.0];
    let Some((first, end)) = unit.token_range else {
        return Vec::new();
    };
    let file = &workspace.files[unit.file.0];
    let tokens = &file.tokens[first..end];
    let text_at = |position: usize| {
        tokens
            .get(position)
            .map_or("", |token| token_text(&file.source, token))
    };
    let is_identifier_at = |position: usize| {
        tokens
            .get(position)
            .is_some_and(|token| token.kind == TokenKind::Identifier)
    };
    let mut roots = Vec::new();
    for (position, token) in tokens.iter().enumerate() {
        if token.kind != TokenKind::Identifier {
            continue;
        }
        let is_followed_by_path_separator =
            text_at(position + 1) == ":" && text_at(position + 2) == ":";
        let is_after_path_separator =
            position >= 2 && text_at(position - 1) == ":" && text_at(position - 2) == ":";
        let separator_starts_the_path = is_after_path_separator
            && !(position >= 3
                && (is_identifier_at(position - 3)
                    || matches!(text_at(position - 3), ">" | ")" | "]")));
        let is_extern_crate_name =
            position >= 2 && text_at(position - 1) == "crate" && text_at(position - 2) == "extern";
        if (is_followed_by_path_separator
            && (!is_after_path_separator || separator_starts_the_path))
            || is_extern_crate_name
        {
            roots.push((
                text_at(position).trim_start_matches("r#").to_string(),
                token.line,
            ));
        }
    }
    roots
}

/// 一个工作区 crate 的外部依赖：代码里点它用的名字（依赖的键，`-` 换成 `_`）→ `Cargo.lock` 里的包名（改名的依赖取 `package` 那一项，
/// 没改名就是键本身）。
fn external_dependencies_of_crate(
    workspace: &Workspace,
    crate_index: CrateIndex,
) -> BTreeMap<String, String> {
    workspace.crates[crate_index.0]
        .manifest
        .dependencies
        .iter()
        .filter(|dependency| dependency.path.is_none())
        .map(|dependency| {
            (
                dependency.key.replace('-', "_"),
                dependency
                    .package
                    .clone()
                    .unwrap_or_else(|| dependency.key.clone()),
            )
        })
        .collect()
}

/// `dependencies` 里的一项落到哪个包：按名找；写了版本的按版本挑；写了来源（`名 版本 (来源)`，同名同版本不同源时 Cargo 这么写）的按来源挑；
/// 剩下恰好一个候选时就是它。
fn resolve_dependency_item<'packages>(
    packages: &'packages [LockedPackage],
    item: &str,
) -> Option<&'packages LockedPackage> {
    let mut parts = item.split_whitespace();
    let name = parts.next()?;
    let version = parts.next();
    let source = parts
        .next()
        .map(|text| text.trim_start_matches('(').trim_end_matches(')'));
    let candidates: Vec<&LockedPackage> = packages
        .iter()
        .filter(|package| {
            package.name == name
                && version.is_none_or(|wanted| package.version == wanted)
                && source.is_none_or(|wanted| package.source.as_deref() == Some(wanted))
        })
        .collect();
    match candidates.as_slice() {
        [only] => Some(*only),
        [] | [_, _, ..] => None,
    }
}

/// 一个包在 `Cargo.lock` 里的那一行键文字：名、版本、来源、checksum（没有的写 `-`），制表符隔开。
fn lock_line_of(package: &LockedPackage) -> String {
    format!(
        "{}\t{}\t{}\t{}",
        package.name,
        package.version,
        package.source.as_deref().unwrap_or("-"),
        package.checksum.as_deref().unwrap_or("-")
    )
}

/// 从整份 `Cargo.lock` 收窄出这个闭包的依赖行：闭包成员点名的外部 crate 解析到的包与它们的传递依赖，每包一行、排好序。
///
/// # Errors
/// 点名的外部 crate 或它的传递依赖落不到 `Cargo.lock` 里的一个包（工作区 crate 自己不在锁文件里、依赖项不在它的 `dependencies` 里、
/// 同名包挑不出一个），交那几处拒算（落在点名那个外部 crate 的那一处文件与行）。
pub fn narrowed_lock_lines(
    workspace: &Workspace,
    outcome: &ClosureOutcome,
    packages: &[LockedPackage],
) -> Result<Vec<String>, Vec<Refusal>> {
    let mut refusals = BTreeSet::new();
    // 待收的包，连同把它带进来的那一处点名（文件，行）：传递依赖落不到包时拒算落在那一处。
    let mut pending: Vec<(&LockedPackage, FileIndex, u32)> = Vec::new();
    for unit_index in &outcome.units {
        let unit = &workspace.units[unit_index.0];
        let crate_index = workspace.files[unit.file.0].crate_index;
        let external = external_dependencies_of_crate(workspace, crate_index);
        if external.is_empty() {
            continue;
        }
        let own_package_name = &workspace.crates[crate_index.0].package_name;
        let own_package = packages
            .iter()
            .find(|package| package.name == *own_package_name && package.source.is_none());
        for (root, line) in path_roots_of_unit(workspace, *unit_index) {
            let Some(dependency_package_name) = external.get(&root) else {
                continue;
            };
            let resolved = own_package.and_then(|own| {
                own.dependencies
                    .iter()
                    .find(|item| {
                        item.split_whitespace().next() == Some(dependency_package_name.as_str())
                    })
                    .and_then(|item| resolve_dependency_item(packages, item))
            });
            match resolved {
                Some(package) => pending.push((package, unit.file, line)),
                None => {
                    refusals.insert(Refusal {
                        file: unit.file,
                        line,
                        form: RefusalForm::LockDependencyUnresolved,
                        fragment: format!("{own_package_name} → {dependency_package_name}"),
                    });
                }
            }
        }
    }
    let mut lines = BTreeSet::new();
    while let Some((package, file, line)) = pending.pop() {
        if !lines.insert(lock_line_of(package)) {
            continue;
        }
        for item in &package.dependencies {
            match resolve_dependency_item(packages, item) {
                Some(dependency) => pending.push((dependency, file, line)),
                None => {
                    refusals.insert(Refusal {
                        file,
                        line,
                        form: RefusalForm::LockDependencyUnresolved,
                        fragment: format!("{} → {item}", package.name),
                    });
                }
            }
        }
    }
    if refusals.is_empty() {
        Ok(lines.into_iter().collect())
    } else {
        Err(refusals.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOCK_TEXT: &str = "version = 4\n\n[[package]]\nname = \"alpha\"\nversion = \"1.0.0\"\n\
        source = \"registry+https://example.invalid/index\"\nchecksum = \"aa\"\ndependencies = [\n \"gamma\",\n]\n\n\
        [[package]]\nname = \"gamma\"\nversion = \"1.0.0\"\nsource = \"registry+https://example.invalid/index\"\n\
        checksum = \"cc\"\n\n[[package]]\nname = \"gamma\"\nversion = \"2.0.0\"\n\n\
        [[package]]\nname = \"consumer\"\nversion = \"0.1.0\"\ndependencies = [\n \"alpha\",\n \"gamma 2.0.0\",\n]\n";

    #[test]
    fn every_package_is_read_with_its_name_version_source_checksum_and_dependencies() {
        let packages = parse_lock_file(LOCK_TEXT).expect("夹具锁文件读得了");
        assert_eq!(packages.len(), 4);
        assert_eq!(
            packages[0],
            LockedPackage {
                name: "alpha".to_string(),
                version: "1.0.0".to_string(),
                source: Some("registry+https://example.invalid/index".to_string()),
                checksum: Some("aa".to_string()),
                dependencies: vec!["gamma".to_string()],
            }
        );
        assert_eq!(packages[3].dependencies, ["alpha", "gamma 2.0.0"]);
        assert_eq!(lock_line_of(&packages[2]), "gamma\t2.0.0\t-\t-");
    }

    /// 同名同版本、来源不同的两个包：带来源的那一项挑得出，不带来源的挑不出（两个候选）。
    #[test]
    fn a_dependency_item_with_a_source_picks_the_package_of_that_source_among_same_name_same_version_packages(
    ) {
        let packages = parse_lock_file(
            "version = 4\n\n[[package]]\nname = \"gamma\"\nversion = \"1.0.0\"\n\
             source = \"registry+https://example.invalid/index\"\n\n[[package]]\nname = \"gamma\"\nversion = \"1.0.0\"\n\
             source = \"registry+https://other.invalid/index\"\n",
        )
        .expect("夹具锁文件读得了");
        assert_eq!(
            resolve_dependency_item(
                &packages,
                "gamma 1.0.0 (registry+https://other.invalid/index)"
            )
            .and_then(|package| package.source.as_deref()),
            Some("registry+https://other.invalid/index")
        );
        assert_eq!(
            resolve_dependency_item(&packages, "gamma 1.0.0"),
            None,
            "不带来源挑不出：两个候选"
        );
    }

    #[test]
    fn a_dependency_item_with_a_version_picks_that_version_and_a_bare_name_needs_a_single_candidate(
    ) {
        let packages = parse_lock_file(LOCK_TEXT).expect("夹具锁文件读得了");
        assert_eq!(
            resolve_dependency_item(&packages, "gamma 2.0.0").map(|package| &package.version[..]),
            Some("2.0.0")
        );
        assert!(
            resolve_dependency_item(&packages, "gamma").is_none(),
            "两个同名包、没写版本：挑不出一个"
        );
        assert_eq!(
            resolve_dependency_item(&packages, "alpha").map(|package| &package.version[..]),
            Some("1.0.0")
        );
    }

    #[test]
    fn a_package_without_a_version_is_an_error() {
        assert_eq!(
            parse_lock_file("[[package]]\nname = \"alpha\"\n"),
            Err(LockFileError::PackageWithoutNameOrVersion {
                package_position: 1
            })
        );
    }
}
