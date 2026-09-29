//! 崩溃放量节点的身份与复用键（里程碑二「增补 4」第一项；第三轮判决 `research/prompts/m3-prune-gpu-r3-main-verification.md`
//! 第四节的设计表，用户 2026-09-28 认了身份形态；E164（崩溃放量新设计的小范围功能测试） 第一、二段量过各件）。
//!
//! - 身份 = `<设这个崩溃点的文件的仓内路径>::<崩溃点名称>::<哈希串>`（用户 2026-09-28 定）：一个函数里设了几个崩溃点就是几个身份，
//!   名称绑在设它的函数上；前两段是标签，只给覆盖报告；哈希串是复用键。
//! - 复用键罩三样属性：这一步带屏障、FUA、盘、偏移、段号的整张写表；这一步之前的镜像（父输出，扇区终值）；判法代码
//!   （从节点的测试文件出发、顺着 `use` 与路径引用只收用到的模块，外加给这些类型写 `impl` 的同 crate 文件；自己原文被
//!   `include_str!` 读的文件取原文摘要，其余去注释、去位置取词法摘要），连同 `Cargo.lock` 与工具链。
//! - 身份只由仓内相对路径与内容算出，不含绝对路径、机器与用户：同一份代码在两台机器、两个目录下算出同一个身份。
//! - 标签类型不给格式化、不给比较（没有 `Display`、`Debug`、`Clone`、`Hash`、`PartialEq`、`Ord`），不能拿去当键（E164 F12）。

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use singlefs_harness::hexadecimal::hexadecimal_text;
use singlefs_harness::memory_pool::{MemoryPool, RetainedWrite, WrittenContents};
use singlefs_harness::sha256::{sha256_digest, DIGEST_BYTES};

use crate::layer0_progress::Layer0ToolchainIdentity;

const SECTOR_BYTES: u64 = 512;

// ============================== 标签与复用键 ==============================

/// 崩溃点标签：设这个点的文件的仓内路径（登记时冻结）、崩溃点名称。只能交给 [`CoverageReport`]，不能格式化、不能比较。
pub struct NodeLabel {
    code_file_in_repository: String,
    crash_point_name: String,
}

/// 标签不合规矩：仓内路径是绝对路径、带 `.` 或 `..` 段、为空；崩溃点名称为空。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeLabelRejected {
    pub reason: String,
}

impl NodeLabel {
    /// # Errors
    /// 仓内路径是绝对路径、带 `..` 段或为空；崩溃点名称为空。身份要能跨机器，路径只收仓内相对路径。
    pub fn new(
        code_file_in_repository: &str,
        crash_point_name: &str,
    ) -> Result<Self, NodeLabelRejected> {
        let rejected = |reason: &str| {
            Err(NodeLabelRejected {
                reason: reason.to_string(),
            })
        };
        if code_file_in_repository.is_empty() || code_file_in_repository.starts_with('/') {
            return rejected("代码文件名要写仓内相对路径");
        }
        if code_file_in_repository
            .split('/')
            .any(|part| part == ".." || part == ".")
        {
            return rejected("仓内路径里不许有 . 或 .. 段");
        }
        if crash_point_name.is_empty() {
            return rejected("崩溃点名称不许为空");
        }
        Ok(Self {
            code_file_in_repository: code_file_in_repository.to_string(),
            crash_point_name: crash_point_name.to_string(),
        })
    }

    /// 身份串：标签两段加复用键的十六进制。只在本 crate 里拼（写进判定存储的节点名、覆盖报告），标签本身不外露文字。
    pub(crate) fn identity_text(&self, reuse_key: &ReuseKey) -> String {
        format!(
            "{}::{}::{}",
            self.code_file_in_repository,
            self.crash_point_name,
            hexadecimal_text(&reuse_key.0)
        )
    }
}

/// 复用键：只能从属性算出（[`StepAttributes::reuse_key`]）。这一步的输入没变就是同一个键；判法变不变另按判法版本在判定行上分。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReuseKey([u8; DIGEST_BYTES]);

impl ReuseKey {
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; DIGEST_BYTES] {
        &self.0
    }
}

/// 一步的两样属性（各是 SHA-256）：这一步写下的字节（带屏障的写表）与它起步的镜像。只有流程进复用键：
/// 判法是判定行的属性（`verdict_store::JudgeVersion`），不在这里。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StepAttributes {
    pub write_table_with_barriers: [u8; DIGEST_BYTES],
    pub parent_output: [u8; DIGEST_BYTES],
}

impl StepAttributes {
    /// 两样按固定次序、各带名字拼起来取 SHA-256。
    #[must_use]
    pub fn reuse_key(&self) -> ReuseKey {
        let mut message = Vec::new();
        for (name, digest) in [
            ("write_table_with_barriers", &self.write_table_with_barriers),
            ("parent_output", &self.parent_output),
        ] {
            message.extend_from_slice(name.as_bytes());
            message.push(b'=');
            message.extend_from_slice(digest);
            message.push(b'\n');
        }
        ReuseKey(sha256_digest(&message))
    }
}

/// 一个崩溃点在这一趟里的去向：核了、复用了库里的判定、按测试代码里的 ignore 跳过了（分叉控制，用户 2026-09-28 定「预设的 ignore 就是剪枝」）、
/// 它的段按展开上限一个都没展开（没有状态可核，也是一种剪枝）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoverageState {
    Checked,
    Reused,
    Ignored,
    NotExpanded,
}

impl CoverageState {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Checked => "checked",
            Self::Reused => "reused",
            Self::Ignored => "ignored",
            Self::NotExpanded => "not_expanded",
        }
    }
}

/// 覆盖报告：只写。收下路径、标签与它这一趟是复用还是重跑，交回逐行 `<路径> <身份> <去向>`（给人看、给门禁数）。
#[derive(Default)]
pub struct CoverageReport {
    lines: Vec<String>,
}

impl CoverageReport {
    pub fn record(
        &mut self,
        path: &str,
        label: NodeLabel,
        reuse_key: &ReuseKey,
        state: CoverageState,
    ) {
        self.lines.push(format!(
            "{path} {} {}",
            label.identity_text(reuse_key),
            state.name()
        ));
    }

    #[must_use]
    pub fn lines(&self) -> &[String] {
        &self.lines
    }
}

// ============================== 录入的属性：写表与父输出 ==============================

fn content_digest_text(contents: &WrittenContents) -> String {
    match contents {
        WrittenContents::Bytes(bytes) => hexadecimal_text(&sha256_digest(bytes)),
        WrittenContents::Zeros { length } => format!("zeros:{length}"),
    }
}

/// 一步的写表哈希（带屏障）：这一步的写按写表次序逐条，每条记盘、种类、FUA、偏移、长度、内容摘要、这次写在这一步里落在第几段。
/// 段由调用方按 D13（验证路线） 已定项 4 切好（`segments` 是整条流的段，写表下标）：屏障挪了、少了，段号就变，哈希跟着变
/// （E164 F1：只按字节算会漏掉去掉一道屏障的改动）。
///
/// # Panics
/// `step_writes` 里有写不属于任何段。
#[must_use]
pub fn step_write_table_with_barriers(
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    step_writes: std::ops::Range<usize>,
) -> [u8; DIGEST_BYTES] {
    let mut segment_of_write: BTreeMap<usize, usize> = BTreeMap::new();
    for (segment_index, segment) in segments.iter().enumerate() {
        for write_index in segment {
            segment_of_write.insert(*write_index, segment_index);
        }
    }
    let first_segment_of_step = step_writes
        .clone()
        .filter_map(|write_index| segment_of_write.get(&write_index).copied())
        .min()
        .unwrap_or(0);
    let mut text = String::new();
    for write_index in step_writes {
        let write = &writes[write_index];
        let segment = segment_of_write
            .get(&write_index)
            .unwrap_or_else(|| panic!("写 {write_index} 不属于任何段"));
        text.push_str(&format!(
            "W {} {} {} {} {} {} {}\n",
            write.device.0,
            write.kind.name(),
            u8::from(write.is_force_unit_access),
            write.offset.0,
            write.length_in_bytes(),
            content_digest_text(&write.contents),
            segment - first_segment_of_step
        ));
    }
    sha256_digest(text.as_bytes())
}

/// 扇区终值表：（盘，扇区号）→ 那个扇区最后留下的内容摘要。父输出 = 基镜像上叠这一步之前全部写之后的这张表。
#[derive(Clone, Default)]
pub struct SectorImage {
    sectors: BTreeMap<(u32, u64), [u8; DIGEST_BYTES]>,
}

impl SectorImage {
    /// 基镜像（mkfs 之后）里写过的每个扇区。
    #[must_use]
    pub fn of_base(base: &MemoryPool) -> Self {
        let mut sectors = BTreeMap::new();
        for (device, image) in &base.devices {
            for (sector, bytes) in image.written_sectors() {
                sectors.insert((device.0, *sector), sha256_digest(bytes));
            }
        }
        Self { sectors }
    }

    /// 按写表次序叠几次写。
    pub fn overlay(&mut self, writes: &[RetainedWrite]) {
        let sector_bytes = usize::try_from(SECTOR_BYTES).expect("512");
        for write in writes {
            let first_sector = write.offset.0 / SECTOR_BYTES;
            match &write.contents {
                WrittenContents::Bytes(bytes) => {
                    for (sector_offset, chunk) in bytes.chunks(sector_bytes).enumerate() {
                        let sector = first_sector
                            + u64::try_from(sector_offset).expect("扇区序号装得进 u64");
                        self.sectors
                            .insert((write.device.0, sector), sha256_digest(chunk));
                    }
                }
                WrittenContents::Zeros { length } => {
                    let zero_sector = sha256_digest(&vec![0u8; sector_bytes]);
                    for sector_offset in 0..length.div_ceil(SECTOR_BYTES) {
                        self.sectors
                            .insert((write.device.0, first_sector + sector_offset), zero_sector);
                    }
                }
            }
        }
    }

    /// 按（盘，扇区）升序每行 `<盘> <扇区> <摘要>` 拼起来取 SHA-256。
    #[must_use]
    pub fn digest(&self) -> [u8; DIGEST_BYTES] {
        let mut text = String::new();
        for ((device, sector), digest) in &self.sectors {
            text.push_str(&format!("{device} {sector} {}\n", hexadecimal_text(digest)));
        }
        sha256_digest(text.as_bytes())
    }
}

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

/// 一份源码的词法单元串（单元之间一个空格）。
#[must_use]
pub fn lexical_text(source: &str) -> String {
    lexical_tokens(source).join(" ")
}

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
    /// 每份被收的文件是被谁、因为什么收进来的（入口文件记「entry」）：给人查「为什么这份也算进来」。
    pub reason_of_each_file: BTreeMap<String, String>,
    /// 被引到、按停止表停在门外的文件（不收、不往下收）：给人查停止表里哪几条真的拦到了。
    pub stopped_at: BTreeSet<String>,
    pub digest: [u8; DIGEST_BYTES],
}

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

/// 文件里 `impl` 块给哪些类型写：头（`impl` 到第一个 `{` 或 `;`）里有 `for` 的取 `for` 之后的标识符（被实现 trait 的那个类型），
/// 没有 `for` 的取整个头。给外部 trait 写 `impl` 的文件（测试设备实现 `BlockDevice` 这类）不因 trait 名被收进来。
fn names_in_impl_headers(tokens: &[String]) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for (position, token) in tokens.iter().enumerate() {
        if token != "impl" {
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

/// 判法闭包的入口：判一个崩溃状态对不对的两份代码——CPU 判器 `crash`（把状态序号解成持久集合的枚举表、择根与两遍恢复、七类 oracle、
/// 记录核对器、调池级 checker）与 GPU 判器 `crash_judge_gpu`（三个内核与它们的输入表）。从这两份出发按模块引用收：恢复、池级 checker、
/// 格式常量、内存池里的状态拼装都在闭包里，恢复顺着模块引用够到的实现文件（`transaction`、`allocator` 这几份）也在；设崩溃点的测试文件、
/// 流水线（录入、领块、存储、对账、身份）都不在——它们变了，判定不变。
pub const JUDGE_ENTRY_FILES: &[&str] = &[
    "crates/singlefs-checker-tier/src/crash.rs",
    "crates/singlefs-checker-tier/src/crash_judge_gpu.rs",
];

/// 判器引到、而不参与判对错的文件：闭包到这里就停，不收它、也不顺着它往下收。每条写明判器为什么引它、它为什么不判对错。
pub const JUDGE_CLOSURE_STOP_FILES: &[(&str, &str)] = &[
    (
        "crates/singlefs-checker-tier/src/layer0_progress.rs",
        "`crash` 的枚举驱动经它写进度文件、分片账本与发现日志、取工具链身份；判一个状态的路径一行都不读它，工具链身份另拼进摘要",
    ),
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

/// 判器的判法代码：从 [`JUDGE_ENTRY_FILES`] 出发、到 [`JUDGE_CLOSURE_STOP_FILES`] 停的模块引用闭包。
///
/// # Errors
/// 同 [`judging_code_by_module_references_stopping_at`]。
pub fn judging_code_of_the_judges(
    repository_root: &Path,
    toolchain: &Layer0ToolchainIdentity,
) -> Result<JudgingCode, JudgingCodeError> {
    let stop_files: Vec<&str> = JUDGE_CLOSURE_STOP_FILES
        .iter()
        .map(|(file, _reason)| *file)
        .collect();
    judging_code_by_module_references_stopping_at(
        repository_root,
        JUDGE_ENTRY_FILES,
        &stop_files,
        toolchain,
    )
}

/// 同 [`judging_code_by_module_references_stopping_at`]，不设停止文件。
///
/// # Errors
/// 同 [`judging_code_by_module_references_stopping_at`]。
pub fn judging_code_by_module_references(
    repository_root: &Path,
    entry_files: &[&str],
    toolchain: &Layer0ToolchainIdentity,
) -> Result<JudgingCode, JudgingCodeError> {
    judging_code_by_module_references_stopping_at(repository_root, entry_files, &[], toolchain)
}

/// 判法代码：从 `entry_files`（仓内路径）出发，顺着路径与 `use` 引用收文件；再收同 crate 里 `impl` 头点名了
/// 已收文件里所声明类型或 trait 的文件，反复到收不到新文件为止；`stop_files` 里的文件被引到就停在门外，不收、不往下收，记进 `stopped_at`。
/// 自己原文被 `include_str!` / `include_bytes!` 读的文件取原文摘要，其余取词法单元串。摘要另拼 `Cargo.lock`、收到的各 crate 的
/// `Cargo.toml`（原文）与工具链。
///
/// # Errors
/// 读不了某份文件、工作区的 crate 清单读不出来。
pub fn judging_code_by_module_references_stopping_at(
    repository_root: &Path,
    entry_files: &[&str],
    stop_files: &[&str],
    toolchain: &Layer0ToolchainIdentity,
) -> Result<JudgingCode, JudgingCodeError> {
    let crates = workspace_crates(repository_root)?;
    let stopped = |file: &str| stop_files.contains(&file);
    let mut stopped_at: BTreeSet<String> = BTreeSet::new();
    let mut included: BTreeSet<String> = BTreeSet::new();
    let mut raw_files: BTreeSet<String> = BTreeSet::new();
    let mut tokens_of: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut reason_of_each_file: BTreeMap<String, String> = BTreeMap::new();
    let mut pending: Vec<String> = entry_files.iter().map(|file| (*file).to_string()).collect();
    for file in &pending {
        reason_of_each_file.insert(file.clone(), "entry".to_string());
    }
    loop {
        while let Some(file) = pending.pop() {
            if !included.insert(file.clone()) {
                continue;
            }
            let tokens = lexical_tokens(&read_repository_file(repository_root, &file)?);
            let (referenced, raw) = references_of(repository_root, &crates, &file, &tokens);
            for raw_file in raw {
                if repository_root.join(&raw_file).is_file() {
                    raw_files.insert(raw_file.clone());
                    reason_of_each_file
                        .entry(raw_file.clone())
                        .or_insert_with(|| format!("include_str in {file}"));
                    pending.push(raw_file);
                }
            }
            for referenced_file in referenced {
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
            tokens_of.insert(file, tokens);
        }
        let declared: BTreeSet<String> = tokens_of
            .values()
            .flat_map(|tokens| declared_names(tokens))
            .collect();
        let crates_touched: BTreeSet<&String> = included
            .iter()
            .filter_map(|file| {
                crates
                    .values()
                    .find(|directory| file.starts_with(&format!("{directory}/src/")))
            })
            .collect();
        for directory in crates_touched {
            for file in rust_files_under(repository_root, &format!("{directory}/src")) {
                // src/bin/ 下是各自成 crate 的装置二进制，不是库的模块，节点不经它们判。
                if included.contains(&file) || file.starts_with(&format!("{directory}/src/bin/")) {
                    continue;
                }
                let tokens = lexical_tokens(&read_repository_file(repository_root, &file)?);
                let header_names = names_in_impl_headers(&tokens);
                let shared: Vec<&String> = header_names.intersection(&declared).collect();
                if let Some(name) = shared.first() {
                    if stopped(&file) {
                        stopped_at.insert(file);
                        continue;
                    }
                    reason_of_each_file
                        .entry(file.clone())
                        .or_insert_with(|| format!("impl names {name}"));
                    pending.push(file);
                }
            }
        }
        if pending.is_empty() {
            break;
        }
    }
    let mut text = String::new();
    for file in &included {
        text.push_str(file);
        text.push('\n');
        if raw_files.contains(file) {
            let bytes =
                std::fs::read(repository_root.join(file)).map_err(|error| JudgingCodeError {
                    reason: format!("读不了 {file}：{error}"),
                })?;
            text.push_str(&format!("RAW:{}", hexadecimal_text(&sha256_digest(&bytes))));
        } else {
            text.push_str(&tokens_of[file].join(" "));
        }
        text.push('\n');
    }
    let mut manifests: BTreeSet<String> = BTreeSet::from(["Cargo.lock".to_string()]);
    for file in &included {
        if let Some(directory) = crates
            .values()
            .find(|directory| file.starts_with(&format!("{directory}/")))
        {
            manifests.insert(format!("{directory}/Cargo.toml"));
        }
    }
    for manifest in &manifests {
        let bytes =
            std::fs::read(repository_root.join(manifest)).map_err(|error| JudgingCodeError {
                reason: format!("读不了 {manifest}：{error}"),
            })?;
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
        files: included.into_iter().collect(),
        raw_files,
        reason_of_each_file,
        stopped_at,
        digest: sha256_digest(text.as_bytes()),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn repository_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("仓根在 crate 往上两层")
    }

    #[test]
    fn lexical_digest_matches_the_e164_anchors() {
        let digest =
            |source: &str| hexadecimal_text(&sha256_digest(lexical_text(source).as_bytes()));
        let anchor = "e7587d8d4f76653ed394fefe683bb459d4976354ee1366bf987fcd16750a418c";
        assert_eq!(
            digest("const A: u8 = 1;"),
            anchor,
            "E164 跑前登记 7.2 第一行"
        );
        assert_eq!(
            digest("/// doc\nconst A: u8 = 1; // x"),
            anchor,
            "注释与文档注释不进摘要"
        );
        assert_eq!(
            digest("const A: u8 = 2;"),
            "1869717063fbdfadfa523427dd8bd33001d9070e3bab056a405409a4adb11eae",
            "字面量改了摘要就变"
        );
        assert_eq!(
            digest("// only\n/* a /* nested */ */"),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "只有注释的文件是空串"
        );
    }

    #[test]
    fn node_label_takes_only_relative_paths_inside_the_repository() {
        assert!(
            NodeLabel::new("/home/x/crates/a.rs", "t").is_err(),
            "绝对路径不收：身份要能跨机器"
        );
        assert!(NodeLabel::new("crates/../a.rs", "t").is_err());
        let label = NodeLabel::new("crates/singlefs-checker-tier/tests/a.rs", "m::t")
            .expect("仓内相对路径");
        let key = StepAttributes {
            write_table_with_barriers: [1; 32],
            parent_output: [2; 32],
        }
        .reuse_key();
        let text = label.identity_text(&key);
        assert!(
            text.starts_with("crates/singlefs-checker-tier/tests/a.rs::m::t::"),
            "身份三段、不带步号：{text}"
        );
        assert_eq!(
            text.len(),
            "crates/singlefs-checker-tier/tests/a.rs::m::t::".len() + 64
        );
    }

    #[test]
    fn each_attribute_changes_the_reuse_key() {
        let base = StepAttributes {
            write_table_with_barriers: [1; 32],
            parent_output: [2; 32],
        };
        let key = base.reuse_key();
        assert_ne!(
            StepAttributes {
                write_table_with_barriers: [9; 32],
                ..base
            }
            .reuse_key(),
            key
        );
        assert_ne!(
            StepAttributes {
                parent_output: [9; 32],
                ..base
            }
            .reuse_key(),
            key
        );
        assert_eq!(base.reuse_key(), key, "同样的属性同一个键");
    }

    fn copy_tree(source: &Path, destination: &Path) {
        std::fs::create_dir_all(destination).expect("建得了目标目录");
        for entry in std::fs::read_dir(source).expect("读得了源目录").flatten() {
            let target = destination.join(entry.file_name());
            if entry.file_type().expect("读得了类型").is_dir() {
                if entry.file_name() != "target" {
                    copy_tree(&entry.path(), &target);
                }
            } else {
                std::fs::copy(entry.path(), &target).expect("拷得了文件");
            }
        }
    }

    /// 判器的判法闭包：两个判器、它们调的恢复与池级 checker、格式常量、内存池里的状态拼装、五份内核原文都在；流水线（录入、存储、
    /// 身份、对账）、进度模块、设点的测试文件、实现的写路径、装置二进制都不在。停止表里每一条都指到现存文件、都真的拦到了（没拦到的是死条目）。
    #[test]
    fn the_judging_closure_of_the_judges_holds_recovery_the_checker_and_the_kernels_and_stops_before_the_pipeline(
    ) {
        let root = repository_root();
        let toolchain = Layer0ToolchainIdentity::of_the_cargo_running_this_test();
        let code = judging_code_of_the_judges(&root, &toolchain).expect("算得出");
        println!(
            "JUDGING_CODE entry=judges files={} raw={} stopped={} digest={}",
            code.files.len(),
            code.raw_files.len(),
            code.stopped_at.len(),
            hexadecimal_text(&code.digest)
        );
        for file in &code.files {
            println!(
                "JUDGING_CODE_FILE entry=judges file={file} reason={}",
                code.reason_of_each_file[file]
            );
        }
        for file in &code.stopped_at {
            println!("JUDGING_CODE_STOPPED entry=judges file={file}");
        }
        let has = |path: &str| code.files.iter().any(|file| file == path);
        for required in [
            "crates/singlefs-checker-tier/src/crash.rs",
            "crates/singlefs-checker-tier/src/crash_judge_gpu.rs",
            "crates/singlefs-checker-tier/src/crash_judge_tables.rs",
            "crates/singlefs-checker-tier/src/crash_facts.rs",
            "crates/singlefs-checker/src/walk.rs",
            "crates/singlefs-checker/src/image.rs",
            "crates/singlefs-core/src/recovery.rs",
            "crates/singlefs-format/src/lib.rs",
            "crates/singlefs-harness/src/memory_pool.rs",
        ] {
            assert!(has(required), "判器的闭包要含 {required}");
        }
        for kernel in [
            "crash_judge_common.wgsl",
            "crash_judge_recovery.wgsl",
            "crash_judge_checker_common.wgsl",
            "crash_judge_checker.wgsl",
            "crash_judge_checker_scan.wgsl",
        ] {
            let path = format!("crates/singlefs-checker-tier/src/{kernel}");
            assert!(has(&path), "内核 {path} 要在判器的闭包里");
            assert!(code.raw_files.contains(&path), "内核 {path} 要按原文进摘要");
        }
        for excluded in [
            "crates/singlefs-checker-tier/src/crash_amplification.rs",
            "crates/singlefs-checker-tier/src/verdict_store.rs",
            "crates/singlefs-checker-tier/src/crash_identity.rs",
            "crates/singlefs-checker-tier/src/layer0_progress.rs",
            "crates/singlefs-checker-tier/src/crash_verify_gpu.rs",
            "crates/singlefs-checker-tier/src/lib.rs",
            "crates/singlefs-core/src/lib.rs",
        ] {
            assert!(!has(excluded), "{excluded} 不判对错，不该在判器的闭包里");
        }
        let outside: Vec<&String> = code
            .files
            .iter()
            .filter(|file| file.contains("/tests/") || file.contains("/src/bin/"))
            .collect();
        assert!(
            outside.is_empty(),
            "测试文件与装置二进制不判对错，不该在判器的闭包里：{outside:?}"
        );
        for (file, reason) in FILES_THE_JUDGES_MUST_NOT_REACH {
            assert!(
                root.join(file).is_file(),
                "不许引的表指到不存在的文件 {file}（{reason}）"
            );
            assert!(
                !has(file) && !code.stopped_at.contains(*file),
                "判器引到了 {file}：它不该在判法闭包里（{reason}）"
            );
        }
        for (stop_file, reason) in JUDGE_CLOSURE_STOP_FILES {
            assert!(
                root.join(stop_file).is_file(),
                "停止表指到不存在的文件 {stop_file}（{reason}）"
            );
            assert!(
                code.stopped_at.contains(*stop_file),
                "停止表里的 {stop_file} 没被判器引到：死条目，删掉它"
            );
        }
    }

    /// 判定只随判器变：在仓副本里给流水线、存储、身份、进度模块、设点的测试文件、实现的写路径各加一条真语句，判法摘要不动；
    /// 给 CPU 判器、GPU 判器、它的输入表、恢复、池级 checker 走树各加一条，摘要变；一份内核原文改一个字节，摘要变。
    #[test]
    fn the_judging_digest_ignores_the_pipeline_and_the_progress_module_but_follows_every_judge() {
        let root = repository_root();
        let copy = std::env::temp_dir().join(format!(
            "singlefs-judging-closure-copy-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&copy);
        copy_tree(&root.join("crates"), &copy.join("crates"));
        std::fs::copy(root.join("Cargo.lock"), copy.join("Cargo.lock")).expect("拷得了 Cargo.lock");
        let toolchain = Layer0ToolchainIdentity::of_the_cargo_running_this_test();
        let baseline = judging_code_of_the_judges(&copy, &toolchain)
            .expect("副本算得出")
            .digest;
        let digest_after_appending = |file: &str, appended: &str| {
            let path = copy.join(file);
            let original = std::fs::read(&path).expect("读得了副本里的文件");
            let mut edited = original.clone();
            edited.extend_from_slice(appended.as_bytes());
            std::fs::write(&path, &edited).expect("写得了副本");
            let digest = judging_code_of_the_judges(&copy, &toolchain)
                .expect("改过的副本算得出")
                .digest;
            std::fs::write(&path, &original).expect("还原得了副本");
            digest
        };
        const RUST_PROBE: &str = "\npub const JUDGING_CLOSURE_PROBE: u8 = 1;\n";
        for unchanged in [
            "crates/singlefs-checker-tier/src/crash_amplification.rs",
            "crates/singlefs-checker-tier/src/verdict_store.rs",
            "crates/singlefs-checker-tier/src/crash_identity.rs",
            "crates/singlefs-checker-tier/src/layer0_progress.rs",
            "crates/singlefs-checker-tier/src/crash_verify_gpu.rs",
            "crates/singlefs-checker-tier/src/crash_judge_dispatch.rs",
            "crates/singlefs-checker-tier/src/gpu_unit_checks.rs",
            "crates/singlefs-checker-tier/src/lib.rs",
            "crates/singlefs-core/src/lib.rs",
            "crates/singlefs-checker-tier/tests/common_crash_points/mod.rs",
        ] {
            assert_eq!(
                digest_after_appending(unchanged, RUST_PROBE),
                baseline,
                "改 {unchanged} 不该动判法摘要"
            );
        }
        for changed in [
            "crates/singlefs-checker-tier/src/crash.rs",
            "crates/singlefs-checker-tier/src/crash_judge_gpu.rs",
            "crates/singlefs-checker-tier/src/crash_judge_tables.rs",
            "crates/singlefs-core/src/recovery.rs",
            "crates/singlefs-checker/src/walk.rs",
        ] {
            assert_ne!(
                digest_after_appending(changed, RUST_PROBE),
                baseline,
                "改 {changed} 要动判法摘要"
            );
        }
        assert_ne!(
            digest_after_appending(
                "crates/singlefs-checker-tier/src/crash_judge_checker.wgsl",
                "\n// probe\n"
            ),
            baseline,
            "内核原文改一个字节要动判法摘要"
        );
        std::fs::remove_dir_all(&copy).expect("删得掉副本");
    }

    /// 身份要能跨机器、跨目录：同一份代码拷到别的路径，判法代码收的文件与摘要逐字相同。
    #[test]
    fn judging_code_is_the_same_in_a_copy_of_the_repository_at_another_path() {
        let root = repository_root();
        let copy =
            std::env::temp_dir().join(format!("singlefs-identity-copy-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&copy);
        copy_tree(&root.join("crates"), &copy.join("crates"));
        std::fs::copy(root.join("Cargo.lock"), copy.join("Cargo.lock")).expect("拷得了 Cargo.lock");
        let toolchain = Layer0ToolchainIdentity::of_the_cargo_running_this_test();
        let entry =
            "crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs";
        let original =
            judging_code_by_module_references(&root, &[entry], &toolchain).expect("主仓算得出");
        let copied =
            judging_code_by_module_references(&copy, &[entry], &toolchain).expect("副本算得出");
        std::fs::remove_dir_all(&copy).expect("删得掉副本");
        assert_eq!(original.files, copied.files, "收的文件逐个相同");
        assert_eq!(original.digest, copied.digest, "摘要相同：身份不含绝对路径");
    }

    /// 每条登记的崩溃枚举用例的判法代码：含恢复与 checker 走树（判它们的正是这两样），不含装置二进制、不含随机历史脚手架
    /// （按 crate 闭包取会把整个 harness 收进来，E164 W5 量到崩溃枚举节点 20 个提交全部重跑；按模块引用取就不会）。
    #[test]
    fn every_registered_crash_case_depends_on_recovery_and_the_checker_walk_but_not_on_apparatus_binaries(
    ) {
        let root = repository_root();
        let toolchain = Layer0ToolchainIdentity::of_the_cargo_running_this_test();
        let registrations =
            std::fs::read_to_string(root.join(".claude/gate.d/stage-inputs.tsv")).expect("登记表");
        let mut cases = 0usize;
        for line in registrations
            .lines()
            .filter(|line| line.starts_with("crash-case:"))
        {
            let test_column = line
                .split('\t')
                .find_map(|column| column.strip_prefix("test="))
                .expect("test= 列");
            let target = test_column.split(':').nth(1).expect("测试目标");
            let entry = format!("crates/singlefs-checker-tier/tests/{target}.rs");
            let code = judging_code_by_module_references(&root, &[entry.as_str()], &toolchain)
                .expect("算得出");
            println!(
                "JUDGING_CODE entry={entry} files={} raw={} digest={}",
                code.files.len(),
                code.raw_files.len(),
                hexadecimal_text(&code.digest)
            );
            for file in &code.files {
                println!(
                    "JUDGING_CODE_FILE entry={target} file={file} reason={}",
                    code.reason_of_each_file[file]
                );
            }
            let has = |path: &str| code.files.iter().any(|file| file == path);
            assert!(
                has("crates/singlefs-core/src/recovery.rs"),
                "{target}：判定靠恢复"
            );
            assert!(
                has("crates/singlefs-checker/src/walk.rs"),
                "{target}：判定靠池级 checker 走树"
            );
            assert!(
                has("crates/singlefs-format/src/lib.rs"),
                "{target}：格式常量经 use 收进来（E164 V11）"
            );
            assert!(
                !code
                    .files
                    .iter()
                    .any(|file| file.starts_with("crates/singlefs-checker-tier/src/bin/")),
                "{target}：装置二进制不是节点判定走的代码"
            );
            if let Some(reason) = code
                .reason_of_each_file
                .get("crates/singlefs-harness/src/history.rs")
            {
                assert!(
                    reason.starts_with("referenced by crates/singlefs-checker-tier/tests/"),
                    "{target}：随机历史脚手架只许由测试文件自己直接引进来，不许经 impl 头带进来（{reason}）"
                );
            }
            cases += 1;
        }
        assert!(cases >= 8, "登记表里至少 8 条崩溃枚举用例，实际 {cases}");
    }
}
