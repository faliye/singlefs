// K5 模型：crash_identity.rs 第 273–771 行（lexical_tokens … references_of）逐字，外加桩与一个合成小仓
#![allow(dead_code, unused)]
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JudgingCodeError { pub reason: String }
// ============================== 核对的属性：判法代码摘要（按模块引用取） ==============================

/// 词法单元：去空白、行注释、块注释（可嵌套）与全部文档注释；串、字节串、原始串、字符、生命周期各算一个单元；
/// 其余标点一个字符一个单元（与 E164（崩溃放量新设计的小范围功能测试） 跑前登记 5.4「词法摘要」同一套切法，装置那边独立写一份核它）。
#[must_use]
pub fn lexical_tokens(source: &str) -> Vec<String> {
    let characters: Vec<char> = source.chars().collect();
    let identifier_start = |character: char| character == '_' || character.is_alphabetic();
    let identifier_continue = |character: char| character == '_' || character.is_alphanumeric();
    let end_of_quoted = |start: usize, quote: char| {
        let mut position = start + 1;
        while position < characters.len() {
            if characters[position] == '\\' {
                position += 2;
                continue;
            }
            if characters[position] == quote {
                return position + 1;
            }
            position += 1;
        }
        characters.len()
    };
    let end_of_raw = |start: usize| -> Option<usize> {
        let mut position = start + 1;
        let mut hashes = 0usize;
        while characters.get(position) == Some(&'#') {
            hashes += 1;
            position += 1;
        }
        if characters.get(position) != Some(&'"') {
            return None;
        }
        position += 1;
        while position < characters.len() {
            if characters[position] == '"'
                && (0..hashes).all(|offset| characters.get(position + 1 + offset) == Some(&'#'))
            {
                return Some(position + 1 + hashes);
            }
            position += 1;
        }
        Some(characters.len())
    };
    let mut tokens = Vec::new();
    let mut position = 0usize;
    while position < characters.len() {
        let current = characters[position];
        let next = characters.get(position + 1).copied();
        if current.is_whitespace() {
            position += 1;
            continue;
        }
        if current == '/' && next == Some('/') {
            while position < characters.len() && characters[position] != '\n' {
                position += 1;
            }
            continue;
        }
        if current == '/' && next == Some('*') {
            let mut depth = 0usize;
            while position + 1 < characters.len() {
                if characters[position] == '/' && characters[position + 1] == '*' {
                    depth += 1;
                    position += 2;
                } else if characters[position] == '*' && characters[position + 1] == '/' {
                    depth -= 1;
                    position += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    position += 1;
                }
            }
            continue;
        }
        let token_start = position;
        let raw_start = if current == 'b' && next == Some('r') {
            Some(position + 1)
        } else if current == 'r' {
            Some(position)
        } else {
            None
        };
        if let Some(raw_end) = raw_start.and_then(end_of_raw) {
            position = raw_end;
        } else if current == 'b' && (next == Some('"') || next == Some('\'')) {
            position = end_of_quoted(position + 1, next.expect("判过是引号"));
        } else if current == '"' {
            position = end_of_quoted(position, '"');
        } else if current == '\'' {
            position = if next == Some('\\') {
                end_of_quoted(position, '\'')
            } else if characters.get(position + 2) == Some(&'\'') {
                position + 3
            } else {
                let mut end = position + 1;
                while end < characters.len() && identifier_continue(characters[end]) {
                    end += 1;
                }
                end
            };
        } else if identifier_start(current) {
            while position < characters.len() && identifier_continue(characters[position]) {
                position += 1;
            }
        } else if current.is_ascii_digit() {
            while position < characters.len()
                && (identifier_continue(characters[position])
                    || (characters[position] == '.'
                        && characters
                            .get(position + 1)
                            .is_some_and(char::is_ascii_digit)))
            {
                position += 1;
            }
        } else {
            position += 1;
        }
        tokens.push(characters[token_start..position].iter().collect());
    }
    tokens
}

fn workspace_crates(repository_root: &Path) -> Result<BTreeMap<String, String>, JudgingCodeError> {
    let mut crates = BTreeMap::new();
    let entries =
        std::fs::read_dir(repository_root.join("crates")).map_err(|error| JudgingCodeError {
            reason: format!("读不了 crates/：{error}"),
        })?;
    for entry in entries.flatten() {
        let manifest = entry.path().join("Cargo.toml");
        let Ok(text) = std::fs::read_to_string(&manifest) else {
            continue;
        };
        let name = text
            .lines()
            .find_map(|line| {
                line.trim()
                    .strip_prefix("name = ")
                    .map(|value| value.trim_matches('"').to_string())
            })
            .ok_or_else(|| JudgingCodeError {
                reason: format!("{} 里没有 name", manifest.display()),
            })?;
        crates.insert(
            name.replace('-', "_"),
            format!("crates/{}", entry.file_name().to_string_lossy()),
        );
    }
    Ok(crates)
}

/// 一份源文件在模块树里的位置：属于哪个 crate（目录）、它自己的模块路径、它的子模块住在哪个目录。
struct ModulePlace {
    crate_directory: String,
    module_path: Vec<String>,
    child_directory: String,
}

fn module_place(file: &str, crates: &BTreeMap<String, String>) -> Option<ModulePlace> {
    let crate_directory = crates
        .values()
        .find(|directory| file.starts_with(&format!("{directory}/")))?
        .clone();
    let inside = &file[crate_directory.len() + 1..];
    let parent = Path::new(file)
        .parent()
        .expect("文件有上层")
        .to_string_lossy()
        .into_owned();
    if let Some(source_relative) = inside.strip_prefix("src/") {
        let mut parts: Vec<String> = source_relative
            .trim_end_matches(".rs")
            .split('/')
            .map(str::to_string)
            .collect();
        let is_directory_root = matches!(
            parts.last().map(String::as_str),
            Some("lib" | "main" | "mod")
        );
        if is_directory_root {
            parts.pop();
        }
        let child_directory = if is_directory_root {
            parent
        } else {
            format!(
                "{parent}/{}",
                Path::new(file)
                    .file_stem()
                    .expect("有文件名")
                    .to_string_lossy()
            )
        };
        Some(ModulePlace {
            crate_directory,
            module_path: parts,
            child_directory,
        })
    } else {
        // tests/ 下的测试目标自成一个 crate：模块路径从空起，子模块与 #[path] 按它所在的目录解。
        Some(ModulePlace {
            crate_directory,
            module_path: Vec::new(),
            child_directory: parent,
        })
    }
}

/// 一个 crate 里模块路径 `segments` 能落到的最长那份文件（`a/b/c.rs`、`a/b/c/mod.rs`、再退一段……）；一段都落不到时交回 None。
fn file_of_module_path(
    repository_root: &Path,
    crate_directory: &str,
    segments: &[String],
) -> Option<String> {
    for length in (1..=segments.len()).rev() {
        let joined = segments[..length].join("/");
        for candidate in [
            format!("{crate_directory}/src/{joined}.rs"),
            format!("{crate_directory}/src/{joined}/mod.rs"),
        ] {
            if repository_root.join(&candidate).is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

fn crate_root_file(repository_root: &Path, crate_directory: &str) -> Option<String> {
    ["src/lib.rs", "src/main.rs"]
        .iter()
        .map(|name| format!("{crate_directory}/{name}"))
        .find(|candidate| repository_root.join(candidate).is_file())
}

fn is_identifier(token: &str) -> bool {
    token
        .chars()
        .next()
        .is_some_and(|first| first == '_' || first.is_alphabetic())
}

/// 从 `start` 起读一条路径 `a :: b :: c`（词法单元里 `::` 是两个 `:`）；`use` 树里的 `{ … }` 展开成几条。
fn paths_at(
    tokens: &[String],
    start: usize,
    prefix: &[String],
    output: &mut Vec<Vec<String>>,
) -> usize {
    let mut segments = prefix.to_vec();
    let mut position = start;
    loop {
        match tokens.get(position).map(String::as_str) {
            Some("{") => {
                position += 1;
                loop {
                    match tokens.get(position).map(String::as_str) {
                        Some("}") | None => return position + 1,
                        Some(",") => position += 1,
                        Some(_) => position = paths_at(tokens, position, &segments, output),
                    }
                }
            }
            Some(token) if is_identifier(token) || token == "*" => {
                if token != "*" && token != "self" {
                    segments.push(token.to_string());
                }
                position += 1;
                let continues = tokens.get(position).map(String::as_str) == Some(":")
                    && tokens.get(position + 1).map(String::as_str) == Some(":");
                if continues {
                    position += 2;
                    continue;
                }
                if tokens.get(position).map(String::as_str) == Some("as") {
                    position += 2;
                }
                output.push(segments);
                return position;
            }
            _ => {
                output.push(segments);
                return position;
            }
        }
    }
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

/// 一条路径落到哪份文件；落不到仓里的（`std`、外部库、同一模块里的项）交回 None。
fn file_of_path(
    repository_root: &Path,
    crates: &BTreeMap<String, String>,
    place: &ModulePlace,
    is_test_crate: bool,
    segments: &[String],
) -> Option<String> {
    let first = segments.first()?.as_str();
    let in_test_crate = |rest: &[String]| {
        let name = rest.first()?;
        [
            format!("{}/{name}.rs", place.child_directory),
            format!("{}/{name}/mod.rs", place.child_directory),
        ]
        .into_iter()
        .find(|candidate| repository_root.join(candidate).is_file())
    };
    match first {
        "std" | "core" | "alloc" => None,
        "crate" if is_test_crate => in_test_crate(&segments[1..]),
        "crate" => file_of_module_path(repository_root, &place.crate_directory, &segments[1..])
            .or_else(|| crate_root_file(repository_root, &place.crate_directory)),
        "self" | "super" => {
            let mut module = place.module_path.clone();
            let mut rest = segments;
            while rest.first().map(String::as_str) == Some("super") {
                module.pop();
                rest = &rest[1..];
            }
            if rest.first().map(String::as_str) == Some("self") {
                rest = &rest[1..];
            }
            if is_test_crate {
                return in_test_crate(rest);
            }
            module.extend(rest.iter().cloned());
            file_of_module_path(repository_root, &place.crate_directory, &module)
                .or_else(|| crate_root_file(repository_root, &place.crate_directory))
        }
        name if crates.contains_key(name) => {
            let directory = &crates[name];
            file_of_module_path(repository_root, directory, &segments[1..])
                .or_else(|| crate_root_file(repository_root, directory))
        }
        _ if is_test_crate => in_test_crate(segments),
        _ => {
            let mut module = place.module_path.clone();
            module.extend(segments.iter().cloned());
            let found = file_of_module_path(repository_root, &place.crate_directory, &module)?;
            // 只收比当前模块更深的那一份（子模块）；落回当前模块自己或更上层，说明这是同一模块里的项，不是模块引用。
            let found_depth =
                module_place(&found, crates).map_or(0, |found_place| found_place.module_path.len());
            (found_depth > place.module_path.len()).then_some(found)
        }
    }
}

/// 一份文件里引到的文件（经路径、`use` 树、测试目标里的 `mod` 与 `#[path]`）与它按原文读的文件（`include_str!` / `include_bytes!`）。
fn references_of(
    repository_root: &Path,
    crates: &BTreeMap<String, String>,
    file: &str,
    tokens: &[String],
) -> (Vec<String>, Vec<String>) {
    let Some(place) = module_place(file, crates) else {
        return (Vec::new(), Vec::new());
    };
    let is_test_crate = !file[place.crate_directory.len() + 1..].starts_with("src/");
    let directory = Path::new(file).parent().expect("文件有上层").to_path_buf();
    let mut referenced = Vec::new();
    let mut raw = Vec::new();
    let mut explicit_path: Option<String> = None;
    let text_at = |position: usize| tokens.get(position).map(String::as_str);
    for position in 0..tokens.len() {
        if text_at(position) == Some("#")
            && text_at(position + 1) == Some("[")
            && text_at(position + 2) == Some("path")
            && text_at(position + 3) == Some("=")
        {
            explicit_path =
                text_at(position + 4).map(|literal| literal.trim_matches('"').to_string());
        }
        if text_at(position) == Some("mod") && text_at(position + 2) == Some(";") && is_test_crate {
            let name = text_at(position + 1).expect("mod 后面有名字");
            let child = match explicit_path.take() {
                Some(relative) => Some(normalized(&directory.join(relative))),
                None => [
                    format!("{}/{name}.rs", place.child_directory),
                    format!("{}/{name}/mod.rs", place.child_directory),
                ]
                .into_iter()
                .find(|candidate| repository_root.join(candidate).is_file()),
            };
            referenced.extend(child);
        }
        if matches!(text_at(position), Some("include_str" | "include_bytes"))
            && text_at(position + 1) == Some("!")
        {
            if let Some(literal) = text_at(position + 3).filter(|literal| literal.starts_with('"'))
            {
                raw.push(normalized(&directory.join(literal.trim_matches('"'))));
            }
        }
        let mut paths = Vec::new();
        if text_at(position) == Some("use") {
            paths_at(tokens, position + 1, &[], &mut paths);
        } else if text_at(position).is_some_and(is_identifier)
            && text_at(position + 1) == Some(":")
            && text_at(position + 2) == Some(":")
            && !(position >= 2
                && text_at(position - 1) == Some(":")
                && text_at(position - 2) == Some(":"))
        {
            paths_at(tokens, position, &[], &mut paths);
        }
        for segments in paths {
            referenced.extend(file_of_path(
                repository_root,
                crates,
                &place,
                is_test_crate,
                &segments,
            ));
        }
    }
    (referenced, raw)
}


fn show(repo: &Path, file: &str) {
    let crates = workspace_crates(repo).unwrap();
    let src = std::fs::read_to_string(repo.join(file)).unwrap();
    let tokens = lexical_tokens(&src);
    let (referenced, raw) = references_of(repo, &crates, file, &tokens);
    println!("{file}: referenced={referenced:?}");
}

fn main() {
    let root = std::env::temp_dir().join(format!("k5-synthetic-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let w = |p: &str, c: &str| { let f = root.join(p); std::fs::create_dir_all(f.parent().unwrap()).unwrap(); std::fs::write(f, c).unwrap(); };
    w("crates/singlefs-harness/Cargo.toml", "[package]\nname = \"singlefs-harness\"\n");
    w("crates/singlefs-harness/src/lib.rs", "pub mod history;\npub mod other;\n");
    w("crates/singlefs-harness/src/history.rs", "pub fn foo() {}\n");
    w("crates/singlefs-harness/src/other.rs", "pub fn bar() {}\n");
    w("crates/singlefs-checker-tier/Cargo.toml", "[package]\nname = \"singlefs-checker-tier\"\n");
    w("crates/singlefs-checker-tier/src/lib.rs", "pub mod via_alias;\npub mod direct;\n");
    w("crates/singlefs-checker-tier/src/direct.rs", "use singlefs_harness::history::foo;\nfn f() { foo(); }\n");
    w("crates/singlefs-checker-tier/src/via_alias.rs", "use singlefs_harness as h;\nfn f() { h::history::foo(); }\n");
    w("crates/singlefs-checker-tier/src/via_extern.rs", "extern crate singlefs_harness as hh;\nfn f() { hh::history::foo(); }\n");
    w("crates/singlefs-checker-tier/src/via_glob.rs", "use singlefs_harness::*;\nfn f() { history::foo(); }\n");
    w("crates/singlefs-checker-tier/src/via_mod_alias.rs", "use singlefs_harness::{self as sh};\nfn f() { sh::history::foo(); }\n");
    w("crates/singlefs-checker-tier/src/via_full_path_expr.rs", "fn f() { singlefs_harness::history::foo(); }\n");
    for f in ["direct","via_alias","via_extern","via_glob","via_mod_alias","via_full_path_expr"] {
        show(&root, &format!("crates/singlefs-checker-tier/src/{f}.rs"));
    }
    let _ = std::fs::remove_dir_all(&root);
}
