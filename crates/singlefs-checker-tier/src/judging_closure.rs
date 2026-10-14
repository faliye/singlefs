//! 判法闭包的算法（崩溃点重放合并第三轮判决 J8、第二轮判决 L3-1：闭包算法抽成零依赖模块，`build.rs` 与库照同一套算）：
//! 词法切分、按模块引用收文件（路径、`use` 树、测试目标里的 `mod` 与 `#[path]`、`include_str!` 读的原文、`impl` 头点名的类型）、
//! 停止表，连同两种摘要——今天的判法摘要（[`ClosureTreeReader::judging_code_stopping_at`]，与抽出之前逐字相同）与三种结论各自的
//! 代码摘要（[`ClosureTreeReader::conclusion_code`]）。
//!
//! 只用 std，由两处经 `#[path]` 共用：库里的 `crash_identity`（`#[path = "judging_closure.rs"] mod judging_closure;`）与
//! `crates/singlefs-checker-tier/build.rs`（编译时算三种结论的代码摘要表，写进二进制）；不在 `lib.rs` 里另声明模块。
//! 摘要要标准 SHA-256，本模块不另写一份：引入它的那一层要在自己的作用域里给出 `sha256_digest`、`DIGEST_BYTES` 与 `hexadecimal_text`
//! （库里是 `singlefs_harness::sha256` 与 `singlefs_harness::hexadecimal`，`build.rs` 经 `#[path]` 引同两份源文件），少一样就编译不过。

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use super::{hexadecimal_text, sha256_digest, DIGEST_BYTES};

// ============================== 词法 ==============================

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

/// 一份源码的词法单元串（单元之间一个空格）。
#[must_use]
pub fn lexical_text(source: &str) -> String {
    lexical_tokens(source).join(" ")
}

// ============================== 闭包收出来的东西 ==============================

/// 判法代码摘要算不出来：读不了文件、工作区里找不到 crate。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JudgingCodeError {
    pub reason: String,
}

/// 按模块引用取出的判法代码：收了哪些文件（仓内路径）、其中按原文取的是哪几份、在哪几份门外停下、整份摘要。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JudgingCode {
    pub files: Vec<String>,
    pub raw_files: BTreeSet<String>,
    /// 每份被收的文件是被谁、因为什么收进来的（入口文件记「entry」）：给人查「为什么这份也算进来」。只记第一个，全部的在
    /// [`Self::introducers_of_each_file`]。
    pub reason_of_each_file: BTreeMap<String, String>,
    /// 每份被收、或被停在门外的文件的全部引入者（不只第一个）：按份判「谁把它带进来」用（`crash_identity` 的
    /// `random_history_scaffolding_introducers_judged`）。
    pub introducers_of_each_file: BTreeMap<String, BTreeSet<FileIntroducer>>,
    /// 被引到、按停止表停在门外的文件（不收、不往下收）：给人查停止表里哪几条真的拦到了。
    pub stopped_at: BTreeSet<String>,
    pub digest: [u8; DIGEST_BYTES],
}

/// 一份文件进了判法闭包（或被停在门外）是谁带的：它就是入口、哪份文件经路径或 `use` 引到它、哪份文件按原文读它、
/// 它的 `impl` 头点名了闭包里声明的哪个名字。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FileIntroducer {
    Entry,
    ReferencedBy { file: String },
    ReadAsRawTextBy { file: String },
    ImplHeaderNaming { declared_name: String },
}

impl FileIntroducer {
    /// 经文件带进来的（引到、按原文读）交回那份文件；入口与 `impl` 头没有引入它的文件，交回 None。
    #[must_use]
    pub fn introducing_file(&self) -> Option<&str> {
        match self {
            Self::ReferencedBy { file } | Self::ReadAsRawTextBy { file } => Some(file),
            Self::Entry | Self::ImplHeaderNaming { .. } => None,
        }
    }
}

/// 记下 `file` 的一个引入者。同一份文件的每一个引入者都记：只记第一个，入口文件自己先引了随机历史脚手架时，
/// 后面经库模块引它的那一份就看不见了（test-script-align-code-r3 判决之后的 impl-b0-guard 报告第 3 条）。
fn note_introducer(
    introducers_of_each_file: &mut BTreeMap<String, BTreeSet<FileIntroducer>>,
    file: &str,
    introducer: FileIntroducer,
) {
    introducers_of_each_file
        .entry(file.to_string())
        .or_default()
        .insert(introducer);
}

/// 摘要里记的工具链三行（与 `crash_point_replay_progress::CrashPointReplayToolchainIdentity` 的三个字段一一对应；这里不引它，本模块只用 std）。
#[derive(Clone, Copy, Debug)]
pub struct ClosureToolchainLines<'lines> {
    pub rustc_version_lines: &'lines str,
    pub cargo_version: &'lines str,
    pub target_triple: &'lines str,
}

// ============================== 模块树 ==============================

/// 工作区里每个 crate 的名字（下划线形）→ 目录（仓内路径）。
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

/// 一份文件引到的文件：经路径、`use` 树、测试目标里的 `mod` 与 `#[path]` 引到的，与按原文读的（`include_str!` / `include_bytes!`）。
#[derive(Clone, Debug, Default)]
struct FileReferences {
    referenced: Vec<String>,
    read_as_raw_text: Vec<String>,
}

fn references_of(
    repository_root: &Path,
    crates: &BTreeMap<String, String>,
    file: &str,
    tokens: &[String],
) -> FileReferences {
    let Some(place) = module_place(file, crates) else {
        return FileReferences::default();
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
    FileReferences {
        referenced,
        read_as_raw_text: raw,
    }
}

/// 文件里声明的类型与 trait 名（`struct`、`enum`、`union`、`trait`、`type` 后面那个名字）。
fn declared_names(tokens: &[String]) -> BTreeSet<String> {
    tokens
        .windows(2)
        .filter(|pair| {
            matches!(
                pair[0].as_str(),
                "struct" | "enum" | "union" | "trait" | "type"
            ) && is_identifier(&pair[1])
        })
        .map(|pair| pair[1].clone())
        .collect()
}

/// `impl` 前一个词法单元是这几样之一时，这个 `impl` 起头的是一个条目（`impl` 块）：上一个条目或语句收尾的 `}`、`;`，
/// 属性收尾的 `]`，模块体、函数体开头的 `{`，`unsafe impl` 的 `unsafe`。别的前一个单元（参数与绑定类型的 `:`、返回类型 `->` 的 `>`、
/// 元组与泛型实参的 `,`、`(`、`<`，引用的 `&`、`mut`）说明它在类型位置上，是 `impl Trait`，不是 `impl` 块。
const TOKENS_AN_ITEM_LEVEL_IMPL_FOLLOWS: &[&str] = &["}", ";", "]", "{", "unsafe"];

/// 下标 `position` 上的 `impl` 起头的是不是条目级的 `impl` 块（文件第一个单元，或紧跟在 [`TOKENS_AN_ITEM_LEVEL_IMPL_FOLLOWS`] 之一后面）。
fn is_an_item_level_impl(tokens: &[String], position: usize) -> bool {
    match position.checked_sub(1) {
        None => true,
        Some(previous_position) => {
            TOKENS_AN_ITEM_LEVEL_IMPL_FOLLOWS.contains(&tokens[previous_position].as_str())
        }
    }
}

/// 文件里条目级的 `impl` 块给哪些类型写：头（`impl` 到第一个 `{` 或 `;`）里有 `for` 的取 `for` 之后的标识符（被实现 trait 的那个类型），
/// 没有 `for` 的取整个头。给外部 trait 写 `impl` 的文件（测试设备实现 `BlockDevice` 这类）不因 trait 名被收进来。
/// 类型位置上的 `impl Trait`（`fn f(judge: impl FnOnce(&GivenStatesOfATable))`、`-> impl Iterator<Item = X>`）不是 `impl` 块，
/// 不取：流水线的参数类型里写了判器一侧声明的类型，不因此把流水线拉进判器的闭包。
fn names_in_impl_headers(tokens: &[String]) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for (position, token) in tokens.iter().enumerate() {
        if token != "impl" || !is_an_item_level_impl(tokens, position) {
            continue;
        }
        let mut header: Vec<&String> = tokens[position + 1..]
            .iter()
            .take_while(|header_token| {
                *header_token != "{" && *header_token != ";" && *header_token != "where"
            })
            .collect();
        // `impl` 紧跟的那组泛型参数（`impl<D: BlockDevice>`）只是约束，不是被实现的类型：按尖括号配对跳过。
        if header.first().is_some_and(|first| *first == "<") {
            let mut depth = 0usize;
            let mut end = 0usize;
            for (index, header_token) in header.iter().enumerate() {
                match header_token.as_str() {
                    "<" => depth += 1,
                    ">" => {
                        depth -= 1;
                        if depth == 0 {
                            end = index + 1;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            header.drain(..end);
        }
        let self_type_start = header
            .iter()
            .position(|header_token| *header_token == "for")
            .map_or(0, |for_position| for_position + 1);
        for header_token in &header[self_type_start..] {
            if is_identifier(header_token) {
                names.insert((*header_token).clone());
            }
        }
    }
    names
}

fn read_repository_file(repository_root: &Path, file: &str) -> Result<String, JudgingCodeError> {
    std::fs::read_to_string(repository_root.join(file)).map_err(|error| JudgingCodeError {
        reason: format!("读不了 {file}：{error}"),
    })
}

fn read_repository_bytes(repository_root: &Path, file: &str) -> Result<Vec<u8>, JudgingCodeError> {
    std::fs::read(repository_root.join(file)).map_err(|error| JudgingCodeError {
        reason: format!("读不了 {file}：{error}"),
    })
}

fn rust_files_under(repository_root: &Path, directory: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut pending: Vec<PathBuf> = vec![repository_root.join(directory)];
    while let Some(current) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&current) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                found.push(
                    path.strip_prefix(repository_root)
                        .expect("在仓里")
                        .to_string_lossy()
                        .into_owned(),
                );
            }
        }
    }
    found.sort();
    found
}

// ============================== 入口与停止表 ==============================

/// 判法闭包的入口：判一个崩溃状态对不对的两份代码——CPU 判器 `crash`（把状态序号解成持久集合的枚举表、择根与两遍恢复、七类 oracle、
/// 记录核对器、调池级 checker）与 GPU 判器 `crash_judge_gpu`（三个内核与它们的输入表）。从这两份出发按模块引用收：恢复、池级 checker、
/// 格式常量、内存池里的状态拼装都在闭包里，恢复顺着模块引用够到的实现文件（`transaction`、`allocator` 这几份）也在；设崩溃点的测试文件、
/// 流水线（录入、领块、存储、对账、身份）都不在——它们变了，判定不变。
pub const JUDGE_ENTRY_FILES: &[&str] = &[
    "crates/singlefs-checker-tier/src/crash.rs",
    "crates/singlefs-checker-tier/src/crash_judge_gpu.rs",
];

/// CPU 判器（[`JUDGE_ENTRY_FILES`] 的头一份）：把状态序号解成持久集合的枚举表、择根与两遍恢复、七类 oracle、记录核对器、调池级 checker。
pub const CPU_JUDGE_ENTRY_FILE: &str = JUDGE_ENTRY_FILES[0];

/// GPU 判器（[`JUDGE_ENTRY_FILES`] 的第二份）：三个内核与它们的输入表。
pub const GPU_JUDGE_ENTRY_FILE: &str = JUDGE_ENTRY_FILES[1];

/// 判器引到、而不参与判对错的文件：闭包到这里就停，不收它、也不顺着它往下收。每条写明判器为什么引它、它为什么不判对错。
pub const JUDGE_CLOSURE_STOP_FILES: &[(&str, &str)] = &[
    (
        "crates/singlefs-checker-tier/src/lib.rs",
        "只有模块声明与 crate 级属性；判器文件测试模块里的 `use super::…` 落不到模块文件时兜底落到它，加一个模块不改任何判定",
    ),
    (
        "crates/singlefs-core/src/lib.rs",
        "只有模块声明与 crate 级属性；`recovery` 测试模块里的 `use super::…` 兜底落到它，加一个模块不改任何判定",
    ),
];

/// 判器不许引、也不在闭包里的文件：判法那一份引了它就等于把它拉回闭包。每条写明它为什么不判对错。
pub const FILES_THE_JUDGES_MUST_NOT_REACH: &[(&str, &str)] = &[
    (
        "crates/singlefs-checker-tier/src/crash_judge_dispatch.rs",
        "GPU 判器的派活：表与草稿区怎么上卡、一次派活带几个状态、几条缓冲；判一个状态的逻辑全在内核与 `crash_judge_gpu.rs` 里。改结论的毛病修在这里时抬 `GPU_JUDGE_REVISION`",
    ),
    (
        "crates/singlefs-checker-tier/src/gpu_unit_checks.rs",
        "起显卡上下文、读显存、按配置挑卡；只有派活那一份引它",
    ),
];

/// 流水线七份（第三轮判决 M2-1、J8）：设点的测试文件都引录制器、存储与进度模块，三种结论的闭包从设点文件出发时停在它们门外，
/// 不收、不往下收——只改派活、存储、对账的提交不换任何一把结论版本。改结论的几处由引擎那一路照 H8 搬进判器一侧的文件。
pub const PIPELINE_FILES_THE_CONCLUSION_CLOSURES_STOP_AT: &[(&str, &str)] = &[
    (
        "crates/singlefs-checker-tier/src/crash_amplification.rs",
        "先录后核的流水线：录写表、排计划、领块、分工、对账、存库；一个状态判成什么由判器定，录制器只把写表交给它",
    ),
    (
        "crates/singlefs-checker-tier/src/crash_judge_dispatch.rs",
        "GPU 判器的派活：表与草稿区怎么上卡、一次派活带几个状态；判法在内核与 `crash_judge_gpu.rs` 里",
    ),
    (
        "crates/singlefs-checker-tier/src/gpu_unit_checks.rs",
        "起显卡上下文、读显存、按配置挑卡",
    ),
    (
        "crates/singlefs-checker-tier/src/verdict_store.rs",
        "判定库的存取布局：结论怎么按块存、怎么读回，不判一个状态",
    ),
    (
        "crates/singlefs-checker-tier/src/crash_point_replay_progress.rs",
        "发现日志与工具链身份；工具链身份另拼进版本",
    ),
    (
        "crates/singlefs-checker-tier/src/crash_library.rs",
        "几类测试各一份的长期判定库与两段协议：录入段写库、判段凭项名读回；不判一个状态",
    ),
    (
        "crates/singlefs-checker-tier/src/crash_state_enumeration.rs",
        "崩溃枚举用例经判定库的两段：整条流当一个崩溃点录进库、判段领块交判器、从库重建计数；判一个状态的逻辑在判器",
    ),
];

/// 三种结论的闭包用的停止表：判器的停止表 ∪ 流水线七份，去重、按路径排。
#[must_use]
pub fn conclusion_closure_stop_files() -> Vec<&'static str> {
    JUDGE_CLOSURE_STOP_FILES
        .iter()
        .chain(PIPELINE_FILES_THE_CONCLUSION_CLOSURES_STOP_AT)
        .map(|(file, _reason)| *file)
        .collect::<BTreeSet<&'static str>>()
        .into_iter()
        .collect()
}

// ============================== 读一棵树、收闭包 ==============================

/// 一次闭包收到的文件：收了哪些、哪几份按原文读、每份为什么收、全部引入者、停在门外的。
struct ClosureMembers {
    included: BTreeSet<String>,
    raw_files: BTreeSet<String>,
    reason_of_each_file: BTreeMap<String, String>,
    introducers_of_each_file: BTreeMap<String, BTreeSet<FileIntroducer>>,
    stopped_at: BTreeSet<String>,
}

/// 一份文件按哪种形态进摘要：按原文（被 `include_str!` / `include_bytes!` 读的）还是按词法单元串（其余）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum FileDigestForm {
    RawBytes,
    LexicalText,
}

impl FileDigestForm {
    fn name(self) -> &'static str {
        match self {
            Self::RawBytes => "raw",
            Self::LexicalText => "lexical",
        }
    }
}

/// 读一棵树（仓根）算闭包：同一棵树上算几个闭包时，每份文件只读、只切一次词法（`build.rs` 一次算几十个入口）。
/// 只在一棵不变的树上用：算的中途文件变了，缓存里是旧的那一份。
pub struct ClosureTreeReader {
    repository_root: PathBuf,
    crates: BTreeMap<String, String>,
    tokens_of_file: BTreeMap<String, Vec<String>>,
    references_of_file: BTreeMap<String, FileReferences>,
    declared_names_of_file: BTreeMap<String, BTreeSet<String>>,
    impl_header_names_of_file: BTreeMap<String, BTreeSet<String>>,
    rust_files_of_directory: BTreeMap<String, Vec<String>>,
    content_digest_of_file: BTreeMap<(String, FileDigestForm), [u8; DIGEST_BYTES]>,
}

/// 一种结论的代码摘要：从入口出发、到 [`conclusion_closure_stop_files`] 停的闭包收了哪些文件、停在哪几份门外、摘要。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConclusionCode {
    pub files: BTreeSet<String>,
    pub stopped_at: BTreeSet<String>,
    pub digest: [u8; DIGEST_BYTES],
}

/// 结论代码摘要的消息头：摘要的写法换了就换这一行，旧摘要与新摘要不会相等。
const CONCLUSION_CODE_DIGEST_HEADER: &str = "singlefs conclusion code digest 1";

impl ClosureTreeReader {
    /// # Errors
    /// 工作区的 crate 清单读不出来（`crates/` 读不了、某份 `Cargo.toml` 里没有 name）。
    pub fn new(repository_root: &Path) -> Result<Self, JudgingCodeError> {
        Ok(Self {
            repository_root: repository_root.to_path_buf(),
            crates: workspace_crates(repository_root)?,
            tokens_of_file: BTreeMap::new(),
            references_of_file: BTreeMap::new(),
            declared_names_of_file: BTreeMap::new(),
            impl_header_names_of_file: BTreeMap::new(),
            rust_files_of_directory: BTreeMap::new(),
            content_digest_of_file: BTreeMap::new(),
        })
    }

    fn tokens_of(&mut self, file: &str) -> Result<&[String], JudgingCodeError> {
        if !self.tokens_of_file.contains_key(file) {
            let tokens = lexical_tokens(&read_repository_file(&self.repository_root, file)?);
            self.tokens_of_file.insert(file.to_string(), tokens);
        }
        Ok(&self.tokens_of_file[file])
    }

    fn references_of_file(&mut self, file: &str) -> Result<FileReferences, JudgingCodeError> {
        if let Some(references) = self.references_of_file.get(file) {
            return Ok(references.clone());
        }
        self.tokens_of(file)?;
        let references = references_of(
            &self.repository_root,
            &self.crates,
            file,
            &self.tokens_of_file[file],
        );
        self.references_of_file
            .insert(file.to_string(), references.clone());
        Ok(references)
    }

    fn declared_names_of_file(&mut self, file: &str) -> Result<BTreeSet<String>, JudgingCodeError> {
        if let Some(names) = self.declared_names_of_file.get(file) {
            return Ok(names.clone());
        }
        let names = declared_names(self.tokens_of(file)?);
        self.declared_names_of_file
            .insert(file.to_string(), names.clone());
        Ok(names)
    }

    fn impl_header_names_of_file(
        &mut self,
        file: &str,
    ) -> Result<BTreeSet<String>, JudgingCodeError> {
        if let Some(names) = self.impl_header_names_of_file.get(file) {
            return Ok(names.clone());
        }
        let names = names_in_impl_headers(self.tokens_of(file)?);
        self.impl_header_names_of_file
            .insert(file.to_string(), names.clone());
        Ok(names)
    }

    fn rust_files_of_directory(&mut self, directory: &str) -> Vec<String> {
        self.rust_files_of_directory
            .entry(directory.to_string())
            .or_insert_with(|| rust_files_under(&self.repository_root, directory))
            .clone()
    }

    /// 一份文件进结论代码摘要的那一行摘要：按原文的取文件字节的 SHA-256，其余取词法单元串的 SHA-256。
    fn content_digest_of_file(
        &mut self,
        file: &str,
        form: FileDigestForm,
    ) -> Result<[u8; DIGEST_BYTES], JudgingCodeError> {
        let key = (file.to_string(), form);
        if let Some(digest) = self.content_digest_of_file.get(&key) {
            return Ok(*digest);
        }
        let digest = match form {
            FileDigestForm::RawBytes => {
                sha256_digest(&read_repository_bytes(&self.repository_root, file)?)
            }
            FileDigestForm::LexicalText => {
                sha256_digest(self.tokens_of(file)?.join(" ").as_bytes())
            }
        };
        self.content_digest_of_file.insert(key, digest);
        Ok(digest)
    }

    /// 从 `entry_files`（仓内路径）出发，顺着路径与 `use` 引用收文件；再收同 crate 里 `impl` 头点名了已收文件里所声明类型或
    /// trait 的文件，反复到收不到新文件为止；`stop_files` 里的文件被引到就停在门外，不收、不往下收，记进 `stopped_at`。
    fn closure_members(
        &mut self,
        entry_files: &[&str],
        stop_files: &[&str],
    ) -> Result<ClosureMembers, JudgingCodeError> {
        let stopped = |file: &str| stop_files.contains(&file);
        let mut stopped_at: BTreeSet<String> = BTreeSet::new();
        let mut included: BTreeSet<String> = BTreeSet::new();
        let mut raw_files: BTreeSet<String> = BTreeSet::new();
        let mut declared: BTreeSet<String> = BTreeSet::new();
        let mut reason_of_each_file: BTreeMap<String, String> = BTreeMap::new();
        let mut introducers_of_each_file: BTreeMap<String, BTreeSet<FileIntroducer>> =
            BTreeMap::new();
        let mut pending: Vec<String> = entry_files.iter().map(|file| (*file).to_string()).collect();
        for file in &pending {
            reason_of_each_file.insert(file.clone(), "entry".to_string());
            note_introducer(&mut introducers_of_each_file, file, FileIntroducer::Entry);
        }
        loop {
            while let Some(file) = pending.pop() {
                if !included.insert(file.clone()) {
                    continue;
                }
                let references = self.references_of_file(&file)?;
                for raw_file in references.read_as_raw_text {
                    if self.repository_root.join(&raw_file).is_file() {
                        raw_files.insert(raw_file.clone());
                        reason_of_each_file
                            .entry(raw_file.clone())
                            .or_insert_with(|| format!("include_str in {file}"));
                        note_introducer(
                            &mut introducers_of_each_file,
                            &raw_file,
                            FileIntroducer::ReadAsRawTextBy { file: file.clone() },
                        );
                        pending.push(raw_file);
                    }
                }
                for referenced_file in references.referenced {
                    note_introducer(
                        &mut introducers_of_each_file,
                        &referenced_file,
                        FileIntroducer::ReferencedBy { file: file.clone() },
                    );
                    if stopped(&referenced_file) {
                        stopped_at.insert(referenced_file);
                        continue;
                    }
                    if !included.contains(&referenced_file) {
                        reason_of_each_file
                            .entry(referenced_file.clone())
                            .or_insert_with(|| format!("referenced by {file}"));
                        pending.push(referenced_file);
                    }
                }
                declared.extend(self.declared_names_of_file(&file)?);
            }
            let crates_touched: BTreeSet<String> = included
                .iter()
                .filter_map(|file| {
                    self.crates
                        .values()
                        .find(|directory| file.starts_with(&format!("{directory}/src/")))
                        .cloned()
                })
                .collect();
            for directory in crates_touched {
                for file in self.rust_files_of_directory(&format!("{directory}/src")) {
                    // src/bin/ 下是各自成 crate 的装置二进制，不是库的模块，节点不经它们判。
                    if file.starts_with(&format!("{directory}/src/bin/")) {
                        continue;
                    }
                    // 已收的文件不再收，但它的 impl 头照样算引入者（全部引入者都记）。
                    let header_names = self.impl_header_names_of_file(&file)?;
                    let shared: Vec<&String> = header_names.intersection(&declared).collect();
                    let Some(first_name) = shared.first() else {
                        continue;
                    };
                    for name in &shared {
                        note_introducer(
                            &mut introducers_of_each_file,
                            &file,
                            FileIntroducer::ImplHeaderNaming {
                                declared_name: (*name).clone(),
                            },
                        );
                    }
                    if included.contains(&file) {
                        continue;
                    }
                    if stopped(&file) {
                        stopped_at.insert(file);
                        continue;
                    }
                    reason_of_each_file
                        .entry(file.clone())
                        .or_insert_with(|| format!("impl names {first_name}"));
                    pending.push(file);
                }
            }
            if pending.is_empty() {
                break;
            }
        }
        Ok(ClosureMembers {
            included,
            raw_files,
            reason_of_each_file,
            introducers_of_each_file,
            stopped_at,
        })
    }

    /// 收到的文件所在各 crate 的 `Cargo.toml`，连同仓根的 `Cargo.lock`，按路径排。
    fn manifests_of(&self, included: &BTreeSet<String>) -> BTreeSet<String> {
        let mut manifests: BTreeSet<String> = BTreeSet::from(["Cargo.lock".to_string()]);
        for file in included {
            if let Some(directory) = self
                .crates
                .values()
                .find(|directory| file.starts_with(&format!("{directory}/")))
            {
                manifests.insert(format!("{directory}/Cargo.toml"));
            }
        }
        manifests
    }

    /// 今天的判法摘要：闭包收的每份文件一段（仓内路径一行，按原文的取 `RAW:` 加原文 SHA-256，其余取整份词法单元串），
    /// 再拼 `Cargo.lock`、收到的各 crate 的 `Cargo.toml`（原文摘要）与工具链，取 SHA-256。与抽成本模块之前逐字相同。
    ///
    /// # Errors
    /// 读不了某份文件。
    pub fn judging_code_stopping_at(
        &mut self,
        entry_files: &[&str],
        stop_files: &[&str],
        toolchain: &ClosureToolchainLines<'_>,
    ) -> Result<JudgingCode, JudgingCodeError> {
        let members = self.closure_members(entry_files, stop_files)?;
        let tokens_of = &self.tokens_of_file;
        let mut text = String::new();
        for file in &members.included {
            text.push_str(file);
            text.push('\n');
            if members.raw_files.contains(file) {
                let bytes = read_repository_bytes(&self.repository_root, file)?;
                text.push_str(&format!("RAW:{}", hexadecimal_text(&sha256_digest(&bytes))));
            } else {
                text.push_str(&tokens_of[file].join(" "));
            }
            text.push('\n');
        }
        for manifest in &self.manifests_of(&members.included) {
            let bytes = read_repository_bytes(&self.repository_root, manifest)?;
            text.push_str(&format!(
                "{manifest}\nRAW:{}\n",
                hexadecimal_text(&sha256_digest(&bytes))
            ));
        }
        text.push_str(&format!(
            "<toolchain>\n{}\n{}\n{}\n",
            toolchain.rustc_version_lines, toolchain.cargo_version, toolchain.target_triple
        ));
        Ok(JudgingCode {
            files: members.included.into_iter().collect(),
            raw_files: members.raw_files,
            reason_of_each_file: members.reason_of_each_file,
            introducers_of_each_file: members.introducers_of_each_file,
            stopped_at: members.stopped_at,
            digest: sha256_digest(text.as_bytes()),
        })
    }

    /// 一种结论的代码摘要：从 `entry_files` 出发、到 [`conclusion_closure_stop_files`] 停的闭包，每份文件一行
    /// `file <路径> <raw|lexical> <摘要>`，前面是消息头、每个入口一行 `entry`、每个停止文件一行 `stop`，后面是
    /// `Cargo.lock` 与收到的各 crate 的 `Cargo.toml` 各一行 `manifest`；整段取 SHA-256。不含工具链：工具链跑时才拼进版本
    /// （`crash_identity` 的结论版本），编译时与跑时比的只是这一段。逐文件先取摘要再拼，一棵树上算几十个入口时每份文件只摘一次。
    ///
    /// # Errors
    /// 读不了某份文件。
    pub fn conclusion_code(
        &mut self,
        entry_files: &[&str],
    ) -> Result<ConclusionCode, JudgingCodeError> {
        let stop_files = conclusion_closure_stop_files();
        let members = self.closure_members(entry_files, &stop_files)?;
        let mut text = format!("{CONCLUSION_CODE_DIGEST_HEADER}\n");
        for entry in entry_files.iter().collect::<BTreeSet<_>>() {
            text.push_str(&format!("entry {entry}\n"));
        }
        for stop_file in &stop_files {
            text.push_str(&format!("stop {stop_file}\n"));
        }
        for file in &members.included {
            let form = if members.raw_files.contains(file) {
                FileDigestForm::RawBytes
            } else {
                FileDigestForm::LexicalText
            };
            let digest = self.content_digest_of_file(file, form)?;
            text.push_str(&format!(
                "file {file} {} {}\n",
                form.name(),
                hexadecimal_text(&digest)
            ));
        }
        for manifest in &self.manifests_of(&members.included) {
            let digest = self.content_digest_of_file(manifest, FileDigestForm::RawBytes)?;
            text.push_str(&format!(
                "manifest {manifest} {}\n",
                hexadecimal_text(&digest)
            ));
        }
        Ok(ConclusionCode {
            files: members.included,
            stopped_at: members.stopped_at,
            digest: sha256_digest(text.as_bytes()),
        })
    }

    /// 这一趟读过的每份文件（仓内路径）：`build.rs` 照它报 `rerun-if-changed`。
    #[must_use]
    pub fn files_read(&self) -> BTreeSet<String> {
        self.tokens_of_file
            .keys()
            .chain(self.content_digest_of_file.keys().map(|(file, _form)| file))
            .cloned()
            .collect()
    }

    /// 这一趟列过的每个目录（仓内路径）：`build.rs` 照它报 `rerun-if-changed`（目录里加了文件，闭包可能变）。
    #[must_use]
    pub fn directories_listed(&self) -> BTreeSet<String> {
        self.rust_files_of_directory.keys().cloned().collect()
    }

    /// 工作区里每个 crate 的目录（仓内路径），按路径排。
    #[must_use]
    pub fn crate_directories(&self) -> BTreeSet<String> {
        self.crates.values().cloned().collect()
    }
}

// ============================== 三种结论的代码摘要表（编译时算、跑时再算） ==============================

/// 结论代码摘要表的一行是哪一种入口：GPU 判器（一份）；CPU 判器连同一份设点文件；一份观察者代码文件。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConclusionCodeEntry {
    GpuJudge,
    CpuJudgeWithSettingFile { setting_file: String },
    ObserverCode { observer_code_file: String },
}

impl ConclusionCodeEntry {
    /// 闭包从哪几份出发：GPU 判器从 [`GPU_JUDGE_ENTRY_FILE`]；CPU 判器从 [`CPU_JUDGE_ENTRY_FILE`] 与设点文件（两份出发的闭包
    /// 就是两份各自闭包的并）；观察者从它的代码文件。
    #[must_use]
    pub fn entry_files(&self) -> Vec<&str> {
        match self {
            Self::GpuJudge => vec![GPU_JUDGE_ENTRY_FILE],
            Self::CpuJudgeWithSettingFile { setting_file } => {
                vec![CPU_JUDGE_ENTRY_FILE, setting_file.as_str()]
            }
            Self::ObserverCode { observer_code_file } => vec![observer_code_file.as_str()],
        }
    }

    /// 表里的前两列：种类名与文件（GPU 判器那一行文件写 `-`）。
    fn table_columns(&self) -> (&'static str, &str) {
        match self {
            Self::GpuJudge => ("gpu_judge", "-"),
            Self::CpuJudgeWithSettingFile { setting_file } => ("cpu_judge", setting_file.as_str()),
            Self::ObserverCode { observer_code_file } => {
                ("observer_code", observer_code_file.as_str())
            }
        }
    }

    fn from_table_columns(kind_column: &str, file_column: &str) -> Option<Self> {
        match (kind_column, file_column) {
            ("gpu_judge", "-") => Some(Self::GpuJudge),
            ("cpu_judge", setting_file) => Some(Self::CpuJudgeWithSettingFile {
                setting_file: setting_file.to_string(),
            }),
            ("observer_code", observer_code_file) => Some(Self::ObserverCode {
                observer_code_file: observer_code_file.to_string(),
            }),
            (_, _) => None,
        }
    }
}

/// `build.rs` 写进 `OUT_DIR` 的那份表的文件名（库经 `include_str!(concat!(env!("OUT_DIR"), "/…"))` 读回）。
pub const CONCLUSION_CODE_DIGEST_TABLE_FILE_NAME: &str = "conclusion_code_digests.tsv";

/// 表的第一行：表的写法换了就换这一行。
const CONCLUSION_CODE_DIGEST_TABLE_HEADER: &str = "# singlefs conclusion code digest table 1";

/// 设点文件与观察者代码文件的候选：checker 档 `tests/` 下的每份测试目标（`tests/*.rs`）与每个搭建模块（`tests/*/mod.rs`），
/// 加 [`CONCLUSION_CANDIDATE_BINARY_DIRECTORY`] 下的每份实验装置与 [`CONCLUSION_CANDIDATE_LIBRARY_FILES`] 里的几份。
/// 不在候选里的入口编译时没算，跑时比对报「编译时没算」，调用方照「不等」处理。
pub const CONCLUSION_CANDIDATE_DIRECTORY: &str = "crates/singlefs-checker-tier/tests";

/// 实验装置二进制（`src/bin/*.rs`）也设给定状态表的项与崩溃点重放的项：各算一行。
pub const CONCLUSION_CANDIDATE_BINARY_DIRECTORY: &str = "crates/singlefs-checker-tier/src/bin";

/// 库里自己设项的文件：崩溃注入模块的单测起引擎判给定状态表，设点文件就是它自己。
pub const CONCLUSION_CANDIDATE_LIBRARY_FILES: &[&str] =
    &["crates/singlefs-checker-tier/src/crash_injection.rs"];

/// 三种结论的代码摘要表：每个入口一行（种类、文件、摘要）。编译时 `build.rs` 在编它的那棵树上算一份写进二进制，
/// 跑时从盘上再算一份比（`crash_identity` 的结论版本那一节）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConclusionCodeDigestTable {
    pub digest_of_each_entry: BTreeMap<ConclusionCodeEntry, [u8; DIGEST_BYTES]>,
}

/// 摘要表的文字认不回来：哪一行、为什么。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConclusionCodeDigestTableUnreadable {
    pub line: String,
    pub reason: &'static str,
}

/// 候选文件（仓内路径），按路径排：[`CONCLUSION_CANDIDATE_DIRECTORY`] 与 [`CONCLUSION_CANDIDATE_BINARY_DIRECTORY`] 顶层的 `.rs`
/// 与一层子目录里的 `mod.rs`，加 [`CONCLUSION_CANDIDATE_LIBRARY_FILES`]。
fn conclusion_candidate_files(repository_root: &Path) -> Result<Vec<String>, JudgingCodeError> {
    let mut candidates = BTreeSet::new();
    for candidate_directory in [
        CONCLUSION_CANDIDATE_DIRECTORY,
        CONCLUSION_CANDIDATE_BINARY_DIRECTORY,
    ] {
        let directory = repository_root.join(candidate_directory);
        let entries = std::fs::read_dir(&directory).map_err(|error| JudgingCodeError {
            reason: format!("读不了 {candidate_directory}：{error}"),
        })?;
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let path = entry.path();
            if path.is_dir() {
                if path.join("mod.rs").is_file() {
                    candidates.insert(format!("{candidate_directory}/{name}/mod.rs"));
                }
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                candidates.insert(format!("{candidate_directory}/{name}"));
            }
        }
    }
    for library_file in CONCLUSION_CANDIDATE_LIBRARY_FILES {
        if !repository_root.join(library_file).is_file() {
            return Err(JudgingCodeError {
                reason: format!("候选表点名的 {library_file} 不在"),
            });
        }
        candidates.insert((*library_file).to_string());
    }
    Ok(candidates.into_iter().collect())
}

impl ConclusionCodeDigestTable {
    /// 在一棵树上算整张表：GPU 判器一行，每份候选文件各一行 CPU 判器（连同它当设点文件）与一行观察者代码。
    ///
    /// # Errors
    /// 读不了某份文件、候选目录或工作区的 crate 清单。
    pub fn of_the_tree(reader: &mut ClosureTreeReader) -> Result<Self, JudgingCodeError> {
        let mut entries = vec![ConclusionCodeEntry::GpuJudge];
        for candidate in conclusion_candidate_files(&reader.repository_root)? {
            entries.push(ConclusionCodeEntry::CpuJudgeWithSettingFile {
                setting_file: candidate.clone(),
            });
            entries.push(ConclusionCodeEntry::ObserverCode {
                observer_code_file: candidate,
            });
        }
        let mut digest_of_each_entry = BTreeMap::new();
        for entry in entries {
            let code = reader.conclusion_code(&entry.entry_files())?;
            digest_of_each_entry.insert(entry, code.digest);
        }
        Ok(Self {
            digest_of_each_entry,
        })
    }

    /// 写成表的文字：表头一行，之后每个入口一行 `<种类>\t<文件>\t<摘要>`，按入口排。
    #[must_use]
    pub fn to_table_text(&self) -> String {
        let mut text = format!("{CONCLUSION_CODE_DIGEST_TABLE_HEADER}\n");
        for (entry, digest) in &self.digest_of_each_entry {
            let (kind_column, file_column) = entry.table_columns();
            text.push_str(&format!(
                "{kind_column}\t{file_column}\t{}\n",
                hexadecimal_text(digest)
            ));
        }
        text
    }

    /// [`Self::to_table_text`] 的逆。
    ///
    /// # Errors
    /// 表头不对、某行不是三列、种类认不得、摘要不是 64 个十六进制字符、同一个入口出现两次。
    pub fn from_table_text(text: &str) -> Result<Self, ConclusionCodeDigestTableUnreadable> {
        let unreadable = |line: &str, reason: &'static str| ConclusionCodeDigestTableUnreadable {
            line: line.to_string(),
            reason,
        };
        let mut lines = text.lines();
        let header = lines.next().unwrap_or("");
        if header != CONCLUSION_CODE_DIGEST_TABLE_HEADER {
            return Err(unreadable(header, "表头不对"));
        }
        let mut digest_of_each_entry = BTreeMap::new();
        for line in lines {
            let columns: Vec<&str> = line.split('\t').collect();
            let &[kind_column, file_column, digest_column] = columns.as_slice() else {
                return Err(unreadable(line, "不是三列"));
            };
            let entry = ConclusionCodeEntry::from_table_columns(kind_column, file_column)
                .ok_or_else(|| unreadable(line, "种类认不得"))?;
            let digest = digest_of_hexadecimal_text(digest_column)
                .ok_or_else(|| unreadable(line, "摘要不是 64 个小写十六进制字符"))?;
            if digest_of_each_entry.insert(entry, digest).is_some() {
                return Err(unreadable(line, "同一个入口出现两次"));
            }
        }
        Ok(Self {
            digest_of_each_entry,
        })
    }
}

/// 64 个小写十六进制字符读回 32 字节；长度或字符不对交回 None。
fn digest_of_hexadecimal_text(text: &str) -> Option<[u8; DIGEST_BYTES]> {
    if text.len() != DIGEST_BYTES * 2 {
        return None;
    }
    let mut digest = [0u8; DIGEST_BYTES];
    for (byte_index, byte) in digest.iter_mut().enumerate() {
        let pair = text.get(byte_index * 2..byte_index * 2 + 2)?;
        if !pair
            .bytes()
            .all(|character| character.is_ascii_digit() || (b'a'..=b'f').contains(&character))
        {
            return None;
        }
        *byte = u8::from_str_radix(pair, 16).ok()?;
    }
    Some(digest)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 小仓：判器一侧声明 `GivenStatesOfATable`；同一 crate 里一份条目级 `impl` 块给它写（该收），一份流水线只在参数类型与
    /// 返回类型的 `impl Trait` 里点名它（不该收）。
    const IMPL_TRAIT_IN_TYPE_POSITION_REPOSITORY_FILES: &[(&str, &str)] = &[
        ("Cargo.lock", "# lock\n"),
        (
            "crates/singlefs-alpha/Cargo.toml",
            "[package]\nname = \"singlefs-alpha\"\n",
        ),
        (
            "crates/singlefs-alpha/src/lib.rs",
            "pub mod judge;\npub mod judgement_display;\npub mod pipeline;\n",
        ),
        (
            "crates/singlefs-alpha/src/judge.rs",
            "pub struct GivenStatesOfATable { pub rows: usize }\n\
             pub fn judge(table: &GivenStatesOfATable) -> usize { table.rows }\n",
        ),
        (
            "crates/singlefs-alpha/src/judgement_display.rs",
            "use std::fmt;\n\
             impl fmt::Display for crate::judge::GivenStatesOfATable {\n    \
             fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result { write!(formatter, \"{}\", self.rows) }\n}\n",
        ),
        (
            "crates/singlefs-alpha/src/pipeline.rs",
            "pub fn judge_each_recorded_table(judge: impl FnOnce(&crate::judge::GivenStatesOfATable) -> usize) -> usize { 0 }\n\
             pub fn recorded_tables() -> impl Iterator<Item = crate::judge::GivenStatesOfATable> { std::iter::empty() }\n",
        ),
    ];

    /// 参数位置与返回类型上的 `impl Trait` 不是 `impl` 块：流水线在那里点名判器一侧声明的类型，不因此进判器的闭包，也不记成引入者；
    /// 同一 crate 里给那个类型写的条目级 `impl` 块照收（阳性对照）。
    #[test]
    fn an_impl_trait_in_a_parameter_or_return_type_naming_a_judge_type_does_not_pull_its_file_into_the_closure(
    ) {
        let root = std::env::temp_dir().join(format!(
            "singlefs-closure-impl-trait-in-type-position-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        for (file, contents) in IMPL_TRAIT_IN_TYPE_POSITION_REPOSITORY_FILES {
            let path = root.join(file);
            std::fs::create_dir_all(path.parent().expect("文件有上层")).expect("建得了目录");
            std::fs::write(&path, contents).expect("写得了小仓的文件");
        }
        let toolchain = ClosureToolchainLines {
            rustc_version_lines: "rustc 0.0.0-probe",
            cargo_version: "cargo 0.0.0-probe",
            target_triple: "x86_64-probe",
        };
        let judge = "crates/singlefs-alpha/src/judge.rs";
        let judgement_display = "crates/singlefs-alpha/src/judgement_display.rs";
        let pipeline = "crates/singlefs-alpha/src/pipeline.rs";
        let computed = ClosureTreeReader::new(&root)
            .and_then(|mut reader| reader.judging_code_stopping_at(&[judge], &[], &toolchain));
        std::fs::remove_dir_all(&root).expect("删得掉小仓");
        let code = computed.expect("小仓算得出");
        assert_eq!(
            code.files,
            vec![judge.to_string(), judgement_display.to_string()],
            "收判器与给它的类型写条目级 impl 块的那一份，不收只在 impl Trait 里点名它的流水线"
        );
        assert_eq!(
            code.reason_of_each_file[judgement_display], "impl names GivenStatesOfATable",
            "条目级 impl 块经 impl 头收进来"
        );
        assert!(
            !code.introducers_of_each_file.contains_key(pipeline),
            "流水线没有引入者：参数与返回类型里的 impl Trait 不算 impl 头点名（{:?}）",
            code.introducers_of_each_file.get(pipeline)
        );
    }
}
