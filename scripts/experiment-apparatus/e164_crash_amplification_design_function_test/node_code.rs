//! E164 第二段：三种节点代码摘要、录入代码摘要、判法摘要（跑前登记 5.4），都在仓副本上算。

use std::collections::BTreeSet;
use std::path::Path;

use crate::common::{run_allowed, sha256_hexadecimal, strings};
use crate::lexical::{combined_digest, function_items_text, has_non_test_function, lexical_tokens, own_source_reads, token_text};
use crate::repository_scans::{files_under, DependencyGraph};

/// 「读自己原文」的认法认不认 `include_str!`。只给变异 M13 翻。
const OWN_SOURCE_READS_INCLUDE_STR: bool = true;

pub fn read_text(root: &Path, path: &str) -> String {
    std::fs::read_to_string(root.join(path)).unwrap_or_else(|error| panic!("读不了 {path}：{error}"))
}

/// 一个包 `src/` 下全部 `.rs`（仓内路径）。
pub fn package_source_files(root: &Path, package_directory: &str) -> Vec<String> {
    files_under(root, &format!("{package_directory}/src")).into_iter().filter(|path| path.ends_with(".rs")).collect()
}

/// 节点所在包的依赖闭包里全部工作区成员的 `src/` 文件。
pub fn closure_source_files(root: &Path, graph: &DependencyGraph, package: &str) -> Vec<String> {
    let identifier = graph.identifier_of(package).unwrap_or_else(|| panic!("{package} 是工作区成员"));
    let mut files = Vec::new();
    for member in graph.closure(&identifier, true) {
        let information = &graph.packages[&member];
        if information.is_workspace_member {
            let directory = information.manifest_directory.strip_prefix(root).expect("成员在副本里").display().to_string();
            files.extend(package_source_files(root, &directory));
        }
    }
    files.sort();
    files.dedup();
    files
}

/// 目标文件连同它经 `mod x;` 与 `#[path = "…"] mod x;` 带进来的文件（按 crate 根的规则解到同一层目录）。
pub fn target_files(root: &Path, target_file: &str) -> Vec<String> {
    let mut found = BTreeSet::new();
    let mut pending = vec![target_file.to_string()];
    while let Some(path) = pending.pop() {
        if !found.insert(path.clone()) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(root.join(&path)) else { continue };
        let tokens = lexical_tokens(&text);
        let directory = Path::new(&path).parent().expect("文件有上层").to_path_buf();
        let mut explicit_path: Option<String> = None;
        for (position, token) in tokens.iter().enumerate() {
            let texts: Vec<&str> = tokens[position..tokens.len().min(position + 6)].iter().map(|item| item.text.as_str()).collect();
            if texts.starts_with(&["#", "[", "path", "="]) {
                explicit_path = texts.get(4).map(|literal| literal.trim_matches('"').to_string());
            }
            if token.text == "mod" && tokens.get(position + 2).is_some_and(|end| end.text == ";") {
                let name = &tokens[position + 1].text;
                let resolved = match explicit_path.take() {
                    Some(relative) => normalized(&directory.join(relative)),
                    None => {
                        let flat = directory.join(format!("{name}.rs"));
                        if root.join(&flat).exists() { normalized(&flat) } else { normalized(&directory.join(name).join("mod.rs")) }
                    }
                };
                pending.push(resolved);
            }
        }
    }
    found.into_iter().collect()
}

fn normalized(path: &Path) -> String {
    let mut parts: Vec<String> = Vec::new();
    for component in path.components() {
        match component.as_os_str().to_string_lossy().as_ref() {
            "." => {}
            ".." => {
                parts.pop();
            }
            other => parts.push(other.to_string()),
        }
    }
    parts.join("/")
}

/// 目标的文件里「读自己原文」读到的仓内文件。
pub fn own_source_files(root: &Path, files: &[String]) -> BTreeSet<String> {
    let mut read = BTreeSet::new();
    for path in files {
        let text = read_text(root, path);
        let directory = Path::new(path).parent().expect("文件有上层");
        for literal in own_source_reads(&text) {
            if literal == "<file!>" {
                read.insert(path.clone());
            } else if OWN_SOURCE_READS_INCLUDE_STR || !text.contains("include_str") {
                let candidate = normalized(&directory.join(&literal));
                if root.join(&candidate).exists() {
                    read.insert(candidate);
                }
            }
        }
    }
    read
}

pub fn digest_by_function(root: &Path, files: &[String]) -> String {
    combined_digest(&files.iter().map(|path| (path.clone(), function_items_text(&read_text(root, path)))).collect::<Vec<_>>())
}

pub fn digest_by_walked_files(root: &Path, files: &[String]) -> String {
    let kept: Vec<(String, String)> = files
        .iter()
        .filter_map(|path| {
            let text = read_text(root, path);
            has_non_test_function(&text).then(|| (path.clone(), token_text(&lexical_tokens(&text))))
        })
        .collect();
    combined_digest(&kept)
}

pub fn rustc_version(root: &Path) -> String {
    String::from_utf8_lossy(&run_allowed("rustc", &strings(&["-vV"]), root, &[]).stdout).into_owned()
}

/// 按 crate 闭包：闭包 `src/` 与目标文件取词法摘要，`raw_files` 里的取原文摘要，另拼 `Cargo.lock` 原文与 `rustc -vV`。
pub fn digest_by_crate_closure(root: &Path, files: &[String], raw_files: &BTreeSet<String>, rustc_text: &str) -> String {
    let mut entries: Vec<(String, String)> = files
        .iter()
        .map(|path| {
            let text = read_text(root, path);
            let content = if raw_files.contains(path) { format!("RAW:{}", sha256_hexadecimal(text.as_bytes())) } else { token_text(&lexical_tokens(&text)) };
            (path.clone(), content)
        })
        .collect();
    entries.push(("Cargo.lock".to_string(), sha256_hexadecimal(read_text(root, "Cargo.lock").as_bytes())));
    entries.push(("<rustc -vV>".to_string(), rustc_text.to_string()));
    combined_digest(&entries)
}

/// 几个包 `src/` 的词法摘要（判法摘要用）。
pub fn packages_lexical_digest(root: &Path, package_directories: &[&str]) -> String {
    let mut entries = Vec::new();
    for directory in package_directories {
        for path in package_source_files(root, directory) {
            entries.push((path.clone(), token_text(&lexical_tokens(&read_text(root, &path)))));
        }
    }
    combined_digest(&entries)
}
