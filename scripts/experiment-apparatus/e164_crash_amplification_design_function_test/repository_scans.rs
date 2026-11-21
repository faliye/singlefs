//! E164 读主仓现状的几件：W3（冻结文件名）、W5（按 crate 闭包的重跑面）、F13（指纹两臂与重编对照）。
//! 产物每行带开跑时的 HEAD 与主工作区 `crates/` 的原文摘要（跑前登记 5.1）。

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::common::{run_allowed, sha256_hexadecimal, strings};
use crate::json::{parse_json, JsonValue};
use crate::lexical::combined_digest;

/// 身份串：`<登记时冻结的代码文件名>::<验证目标名>::<流程步号>::<哈希串>`。
pub fn identity_text(code_file_name: &str, target_name: &str, step_number: u32, hash_text: &str) -> String {
    format!("{code_file_name}::{target_name}::{step_number}::{hash_text}")
}

pub fn git_text(root: &Path, arguments: &[&str]) -> String {
    let output = run_allowed("git", &strings(arguments), root, &[]);
    assert!(output.status.success(), "git {} 失败：{}", arguments.join(" "), String::from_utf8_lossy(&output.stderr));
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// 目录下全部文件（跳过名叫 `target` 与 `.git` 的目录），交回相对 `base` 的路径。
pub fn files_under(base: &Path, relative_directory: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut pending = vec![base.join(relative_directory)];
    while let Some(directory) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&directory) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            let file_type = entry.file_type().expect("读得了文件类型");
            if file_type.is_dir() {
                if name != "target" && name != ".git" {
                    pending.push(path);
                }
            } else if file_type.is_file() {
                found.push(path.strip_prefix(base).expect("在 base 下").display().to_string());
            }
        }
    }
    found.sort();
    found
}

/// 主工作区 `crates/` 的原文摘要（5.4「原文摘要」的多文件拼法）。
pub fn crates_raw_digest(root: &Path) -> String {
    let files: Vec<(String, String)> = files_under(root, "crates")
        .into_iter()
        .map(|path| {
            let bytes = std::fs::read(root.join(&path)).expect("读得了 crates/ 下的文件");
            (path, sha256_hexadecimal(&bytes))
        })
        .collect();
    combined_digest(&files)
}

pub struct ProductSuffix(pub String);

impl ProductSuffix {
    pub fn current(root: &Path) -> Self {
        let head = git_text(root, &["rev-parse", "HEAD"]).trim().to_string();
        Self(format!("head={head} crates_raw_digest={}", crates_raw_digest(root)))
    }
    pub fn emit(&self, line: &str) {
        result_line!("{line} {}", self.0);
    }
}

// ============================== cargo metadata 的依赖图 ==============================

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DependencyKind {
    Normal,
    Development,
    Build,
}

pub struct PackageInformation {
    pub name: String,
    pub manifest_directory: PathBuf,
    pub has_build_script: bool,
    pub is_workspace_member: bool,
}

pub struct DependencyGraph {
    pub packages: BTreeMap<String, PackageInformation>,
    pub dependencies: BTreeMap<String, Vec<(String, Vec<DependencyKind>)>>,
}

pub fn dependency_graph(directory: &Path, extra_arguments: &[&str]) -> Result<DependencyGraph, String> {
    let mut arguments = strings(&["metadata", "--offline", "--format-version", "1"]);
    arguments.extend(strings(extra_arguments));
    let output = run_allowed("cargo", &arguments, directory, &[]);
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).lines().take(3).collect::<Vec<&str>>().join(" / "));
    }
    let metadata = parse_json(&String::from_utf8_lossy(&output.stdout));
    let members: BTreeSet<String> = metadata.field("workspace_members").array_items().iter().filter_map(|member| member.text().map(str::to_string)).collect();
    let mut packages = BTreeMap::new();
    for package in metadata.field("packages").array_items() {
        let identifier = package.field("id").text().expect("包有 id").to_string();
        let manifest_path = PathBuf::from(package.field("manifest_path").text().expect("包有 manifest_path"));
        let has_build_script = package.field("targets").array_items().iter().any(|target| {
            target.field("kind").array_items().iter().any(|kind| kind.text() == Some("custom-build"))
        });
        packages.insert(
            identifier.clone(),
            PackageInformation {
                name: package.field("name").text().expect("包有名字").to_string(),
                manifest_directory: manifest_path.parent().expect("manifest 有上层").to_path_buf(),
                has_build_script,
                is_workspace_member: members.contains(&identifier),
            },
        );
    }
    let mut dependencies = BTreeMap::new();
    for node in metadata.field("resolve").field("nodes").array_items() {
        let identifier = node.field("id").text().expect("节点有 id").to_string();
        let edges = node
            .field("deps")
            .array_items()
            .iter()
            .map(|edge| {
                let kinds = edge
                    .field("dep_kinds")
                    .array_items()
                    .iter()
                    .map(|kind| match kind.field("kind") {
                        JsonValue::Text(text) if text == "dev" => DependencyKind::Development,
                        JsonValue::Text(text) if text == "build" => DependencyKind::Build,
                        JsonValue::Null
                        | JsonValue::Boolean(_)
                        | JsonValue::Number(_)
                        | JsonValue::Text(_)
                        | JsonValue::Array(_)
                        | JsonValue::Object(_) => DependencyKind::Normal,
                    })
                    .collect();
                (edge.field("pkg").text().expect("边有 pkg").to_string(), kinds)
            })
            .collect();
        dependencies.insert(identifier, edges);
    }
    Ok(DependencyGraph { packages, dependencies })
}

impl DependencyGraph {
    pub fn identifier_of(&self, name: &str) -> Option<String> {
        self.packages.iter().find(|(_, package)| package.name == name && package.is_workspace_member).map(|(identifier, _)| identifier.clone())
    }

    /// 节点所在包的依赖闭包：包自己的 normal、build（`include_development` 时加 dev）依赖，再往下只走 normal 与 build。
    pub fn closure(&self, root_identifier: &str, include_development: bool) -> BTreeSet<String> {
        let mut closure = BTreeSet::from([root_identifier.to_string()]);
        let mut pending = Vec::new();
        for (dependency, kinds) in self.dependencies.get(root_identifier).into_iter().flatten() {
            let follows = kinds.iter().any(|kind| *kind != DependencyKind::Development || include_development);
            if follows {
                pending.push(dependency.clone());
            }
        }
        while let Some(identifier) = pending.pop() {
            if !closure.insert(identifier.clone()) {
                continue;
            }
            for (dependency, kinds) in self.dependencies.get(&identifier).into_iter().flatten() {
                if kinds.iter().any(|kind| *kind != DependencyKind::Development) {
                    pending.push(dependency.clone());
                }
            }
        }
        closure
    }

    pub fn member_names(&self, closure: &BTreeSet<String>) -> BTreeSet<String> {
        closure.iter().filter_map(|identifier| self.packages.get(identifier)).filter(|package| package.is_workspace_member).map(|package| package.name.clone()).collect()
    }
}

pub fn command_repository() {
    let root = crate::repository_root();
    let suffix = ProductSuffix::current(&root);
    crate::repository_scans_frozen_file_names::scan_frozen_file_names(&root, &suffix);
    crate::repository_scans_crate_closure_reruns::scan_rerun_surface(&root, &suffix);
    crate::fingerprints::scan_fingerprints(&root, &suffix);
}
