//! 工作区：从根目录找到每个 crate、读清单、找编译目标，切全部 `.rs` 与 `.wgsl`（按文件切片并行，合并按文件次序，
//! 多线程与单线程逐字相同），按 `mod` 声明（连 `#[path]`）连出每个编译目标的模块树，再建闭包要查的索引。
//!
//! 根目录可以是仓根（底下有 `crates/`），也可以就是 `crates/` 本身；输出里的文件路径都从仓根算（`crates/…`）。

use std::collections::{BTreeSet, HashMap, VecDeque};
use std::fmt;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};

use crate::conditional_compilation::Predicate;
use crate::lexer::Token;
use crate::manifest::{
    parse_crate_manifest, BuildScriptDeclaration, CrateManifest, DeclaredTarget, ManifestError,
};
use crate::rule_switches::RuleSwitches;
use crate::rust_items::{normalized_path, parse_rust_file, RustFileInput};
use crate::units::{
    DocumentationTestAttachment, FileIndex, Refusal, RefusalForm, Unit, UnitIndex, UnitKind,
};
use crate::wgsl::{parse_wgsl_file, WgslUnit};

/// 工作区里一个 crate 的下标。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CrateIndex(pub usize);

/// 工作区里一个编译目标的下标。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TargetIndex(pub usize);

/// 编译目标的种类。文档测试是一个虚的目标：没有自己的文件，链接本 crate 的库。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TargetKind {
    Library,
    Binary,
    IntegrationTest,
    Example,
    Benchmark,
    BuildScript,
    DocumentationTest,
}

impl TargetKind {
    /// 复用键里写的种类名。
    #[must_use]
    pub fn key_name(self) -> &'static str {
        match self {
            TargetKind::Library => "lib", // naming-lint:external cargo 定的目标种类名
            TargetKind::Binary => "bin",
            TargetKind::IntegrationTest => "test",
            TargetKind::Example => "example",
            TargetKind::Benchmark => "bench",
            TargetKind::BuildScript => "build",
            TargetKind::DocumentationTest => "doctest",
        }
    }

    /// 这个目标链接本 crate 的库（库自己与构建脚本不链接）。
    #[must_use]
    pub fn links_its_own_library(self) -> bool {
        match self {
            TargetKind::Binary
            | TargetKind::IntegrationTest
            | TargetKind::Example
            | TargetKind::Benchmark
            | TargetKind::DocumentationTest => true,
            TargetKind::Library | TargetKind::BuildScript => false,
        }
    }

    /// cargo 编它时开 `cfg(test)`（集成测试与基准；库与 bin 的单测由调用方给 `--cfg test`）。
    #[must_use]
    pub fn is_compiled_as_test(self) -> bool {
        match self {
            TargetKind::IntegrationTest | TargetKind::Benchmark => true,
            TargetKind::Library
            | TargetKind::Binary
            | TargetKind::Example
            | TargetKind::BuildScript
            | TargetKind::DocumentationTest => false,
        }
    }

    /// 链接开发依赖（`[dev-dependencies]`）。
    #[must_use]
    pub fn links_development_dependencies(self) -> bool {
        match self {
            TargetKind::IntegrationTest
            | TargetKind::Example
            | TargetKind::Benchmark
            | TargetKind::DocumentationTest => true,
            TargetKind::Library | TargetKind::Binary | TargetKind::BuildScript => false,
        }
    }
}

/// 命令行 `--target` 点名的目标。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TargetSelector {
    Library,
    Binary(String),
    IntegrationTest(String),
    Example(String),
    Benchmark(String),
    BuildScript,
    DocumentationTest,
}

impl TargetSelector {
    /// 读 `lib`、`bin:<名>`、`test:<名>`、`example:<名>`、`bench:<名>`、`build`、`doctest`。
    ///
    /// # Errors
    /// 不是这几种写法，交原文。
    pub fn parse(text: &str) -> Result<Self, String> {
        match text.split_once(':') {
            None => match text {
                "lib" => Ok(TargetSelector::Library),
                "build" => Ok(TargetSelector::BuildScript),
                "doctest" => Ok(TargetSelector::DocumentationTest),
                _ => Err(text.to_string()),
            },
            Some((kind, name)) if !name.is_empty() => match kind {
                "bin" => Ok(TargetSelector::Binary(name.to_string())),
                "test" => Ok(TargetSelector::IntegrationTest(name.to_string())),
                "example" => Ok(TargetSelector::Example(name.to_string())),
                "bench" => Ok(TargetSelector::Benchmark(name.to_string())),
                _ => Err(text.to_string()),
            },
            Some(_) => Err(text.to_string()),
        }
    }
}

/// 一个编译目标。
#[derive(Clone, Debug)]
pub struct Target {
    pub crate_index: CrateIndex,
    pub kind: TargetKind,
    pub name: String,
    pub root_file: Option<FileIndex>,
}

/// 工作区里一个 crate。
#[derive(Clone, Debug)]
pub struct WorkspaceCrate {
    pub package_name: String,
    /// 代码里点它用的名字：库名，或包名把 `-` 换成 `_`。
    pub crate_identifier: String,
    pub directory: PathBuf,
    pub manifest: CrateManifest,
}

/// 源码是哪种语言。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceLanguage {
    Rust,
    Wgsl,
}

/// 一份源码文件。
#[derive(Clone, Debug)]
pub struct SourceFile {
    pub relative_path: String,
    pub absolute_path: PathBuf,
    pub crate_index: CrateIndex,
    pub language: SourceLanguage,
    pub source: String,
    pub tokens: Vec<Token>,
    /// 括号配不上、认不出的条目起头：这份文件定义了哪些名字说不全，链接到它的闭包一律拒算。
    pub file_level_refusals: Vec<Refusal>,
}

/// 一份文件在一个编译目标里：模块路径、从目标根经哪几条 `mod` 声明连进来（每一条路径一组与起来的谓词）、声明它的单元。
#[derive(Clone, Debug)]
pub struct FileMembership {
    pub target: TargetIndex,
    pub module_path: Vec<String>,
    pub condition_alternatives: Vec<Vec<Predicate>>,
    pub declared_by: Vec<UnitIndex>,
}

/// 闭包要查的索引。
#[derive(Clone, Debug)]
pub struct WorkspaceIndexes {
    /// 编译目标 → 名字 → 按裸名找得到的单元（非成员）。`const _` 这类没有名字的条目不在里面（[`UNNAMED_ITEM_NAME`]）。
    pub units_by_target_and_name: HashMap<TargetIndex, HashMap<String, Vec<UnitIndex>>>,
    /// 名字 → impl / trait 里叫这个名字的成员。
    pub members_by_name: HashMap<String, Vec<UnitIndex>>,
    /// impl / trait 头 → 它的成员。
    pub members_of_container: HashMap<UnitIndex, Vec<UnitIndex>>,
    /// 自身类型里出现的名字 → impl 头。
    pub implementations_by_self_type_name: HashMap<String, Vec<UnitIndex>>,
    /// trait 自己的名字 → trait impl 头（只按 trait 自己的名字，trait 参数里的名字不算：`impl Wrap<dyn Sink> for Holder` 只记在 `Wrap` 下）。
    pub trait_implementations_by_trait_name: HashMap<String, Vec<UnitIndex>>,
    /// trait 自己的名字 → 它的 blanket impl 头（`impl<T: Copy> Shout for T` 记在 `Shout` 下）：trait 头进闭包时连同成员一起进。
    pub blanket_implementations_by_trait_name: HashMap<String, Vec<UnitIndex>>,
    /// （文件，行内模块路径）→ 那个模块里的 use。
    pub uses_by_scope: HashMap<(FileIndex, Vec<String>), Vec<UnitIndex>>,
    /// 条目 → 它文档注释里的文档测试单元。
    pub documentation_tests_of_item: HashMap<UnitIndex, Vec<UnitIndex>>,
    /// （文件，行内模块路径）→ 模块内文档注释里的文档测试单元。
    pub documentation_tests_of_module: HashMap<(FileIndex, Vec<String>), Vec<UnitIndex>>,
    /// 绝对路径 → 文件。
    pub files_by_path: HashMap<PathBuf, FileIndex>,
}

/// 装好的工作区。
#[derive(Clone, Debug)]
pub struct Workspace {
    pub repository_root: PathBuf,
    pub crates: Vec<WorkspaceCrate>,
    pub files: Vec<SourceFile>,
    pub units: Vec<Unit>,
    pub wgsl_units: Vec<WgslUnit>,
    pub targets: Vec<Target>,
    /// 每份文件属于哪几个编译目标（按目标下标排好）。
    pub memberships: Vec<Vec<FileMembership>>,
    pub files_of_target: Vec<Vec<FileIndex>>,
    pub indexes: WorkspaceIndexes,
}

/// 工作区装不起来。
#[derive(Debug)]
pub enum WorkspaceError {
    RootUnreadable { path: PathBuf, reason: String },
    ManifestUnreadable { path: PathBuf, error: ManifestError },
    NoCrates { path: PathBuf },
}

impl fmt::Display for WorkspaceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkspaceError::RootUnreadable { path, reason } => {
                write!(formatter, "根目录 {} 读不了：{reason}", path.display())
            }
            WorkspaceError::ManifestUnreadable { path, error } => write!(
                formatter,
                "{} 第 {} 行读不了：{}",
                path.display(),
                error.line,
                error.reason
            ),
            WorkspaceError::NoCrates { path } => {
                write!(
                    formatter,
                    "{} 底下没有一个带 Cargo.toml 的 crate 目录",
                    path.display()
                )
            }
        }
    }
}

struct FileEntry {
    absolute_path: PathBuf,
    relative_path: String,
    crate_index: CrateIndex,
    language: SourceLanguage,
}

struct ParsedSource {
    source: String,
    tokens: Vec<Token>,
    units: Vec<Unit>,
    wgsl_units: Vec<WgslUnit>,
    file_level_refusals: Vec<Refusal>,
}

fn relative_display(path: &Path, repository_root: &Path) -> String {
    let relative = path.strip_prefix(repository_root).unwrap_or(path);
    relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<String>>()
        .join("/")
}

fn collect_source_files(directory: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    let mut paths: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect();
    paths.sort();
    for path in paths {
        let file_name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        if path.is_dir() {
            if file_name != "target" && !file_name.starts_with('.') {
                collect_source_files(&path, found);
            }
        } else if file_name.ends_with(".rs") || file_name.ends_with(".wgsl") {
            found.push(path);
        }
    }
}

fn parse_one_file(
    file: FileIndex,
    entry: &FileEntry,
    crates: &[WorkspaceCrate],
    switches: &RuleSwitches,
) -> ParsedSource {
    let source = std::fs::read(&entry.absolute_path)
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .unwrap_or_default();
    match entry.language {
        SourceLanguage::Rust => {
            let file_directory = entry
                .absolute_path
                .parent()
                .map_or_else(PathBuf::new, Path::to_path_buf);
            let parsed = parse_rust_file(&RustFileInput {
                file,
                source: &source,
                file_directory: &file_directory,
                manifest_directory: Some(&crates[entry.crate_index.0].directory),
                switches,
            });
            ParsedSource {
                tokens: parsed.tokens,
                units: parsed.units,
                wgsl_units: Vec::new(),
                file_level_refusals: parsed.file_level_refusals,
                source,
            }
        }
        SourceLanguage::Wgsl => {
            let parsed = parse_wgsl_file(&source, file);
            ParsedSource {
                tokens: Vec::new(),
                units: Vec::new(),
                wgsl_units: parsed.units,
                file_level_refusals: parsed.refusals,
                source,
            }
        }
    }
}

/// 按文件切片并行切：每个线程拿连续的一段文件，结果按文件下标放回原位，所以与单线程逐字相同。
fn parse_files_in_parallel(
    entries: &[FileEntry],
    crates: &[WorkspaceCrate],
    threads: NonZeroUsize,
    switches: &RuleSwitches,
) -> Vec<ParsedSource> {
    if entries.is_empty() {
        return Vec::new();
    }
    let thread_count = threads.get().min(entries.len());
    let chunk_length = entries.len().div_ceil(thread_count);
    let mut parsed_in_order: Vec<Option<ParsedSource>> = (0..entries.len()).map(|_| None).collect();
    std::thread::scope(|scope| {
        let workers: Vec<_> = entries
            .chunks(chunk_length)
            .enumerate()
            .map(|(chunk_index, chunk)| {
                scope.spawn(move || {
                    chunk
                        .iter()
                        .enumerate()
                        .map(|(offset, entry)| {
                            let file = FileIndex(chunk_index * chunk_length + offset);
                            (file, parse_one_file(file, entry, crates, switches))
                        })
                        .collect::<Vec<(FileIndex, ParsedSource)>>()
                })
            })
            .collect();
        for worker in workers {
            for (file, parsed) in worker
                .join()
                .expect("切分线程只读文件、切词，不会 panic；panic 了说明切分有越界的 bug")
            {
                parsed_in_order[file.0] = Some(parsed);
            }
        }
    });
    parsed_in_order
        .into_iter()
        .map(|parsed| parsed.expect("每份文件都恰好分给了一个线程"))
        .collect()
}

fn offset_unit_indexes(unit: &mut Unit, offset: usize) {
    let shifted = |index: UnitIndex| UnitIndex(index.0 + offset);
    unit.container = unit.container.map(shifted);
    unit.module_head = unit.module_head.map(shifted);
    if let Some(DocumentationTestAttachment::Item(owner)) = unit.documentation_test_of.as_mut() {
        *owner = shifted(*owner);
    }
}

fn crate_relative_components(file: &SourceFile, crate_directory: &Path) -> Vec<String> {
    file.absolute_path
        .strip_prefix(crate_directory)
        .map(|relative| {
            relative
                .components()
                .map(|component| component.as_os_str().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default()
}

fn discovered_root_files(
    components_of_files: &[(FileIndex, Vec<String>)],
    directory: &[&str],
) -> Vec<(String, FileIndex)> {
    let mut discovered = Vec::new();
    for (file, components) in components_of_files {
        let parts: Vec<&str> = components.iter().map(String::as_str).collect();
        if parts.len() == directory.len() + 1 && parts[..directory.len()] == *directory {
            if let Some(stem) = parts[directory.len()].strip_suffix(".rs") {
                discovered.push((stem.to_string(), *file));
            }
        }
        if parts.len() == directory.len() + 2
            && parts[..directory.len()] == *directory
            && parts[directory.len() + 1] == "main.rs"
        {
            discovered.push((parts[directory.len()].to_string(), *file));
        }
    }
    discovered
}

struct TargetDiscovery<'discovery> {
    crate_index: CrateIndex,
    workspace_crate: &'discovery WorkspaceCrate,
    files_by_path: &'discovery HashMap<PathBuf, FileIndex>,
    components_of_files: Vec<(FileIndex, Vec<String>)>,
}

impl TargetDiscovery<'_> {
    fn file_at(&self, relative: &str) -> Option<FileIndex> {
        self.files_by_path
            .get(&normalized_path(
                &self.workspace_crate.directory.join(relative),
            ))
            .copied()
    }

    fn targets_of_kind(
        &self,
        kind: TargetKind,
        declared: &[DeclaredTarget],
        is_discovering: bool,
        directory: &[&str],
    ) -> Vec<Target> {
        let mut by_root: Vec<(String, FileIndex)> = Vec::new();
        for declaration in declared {
            let name = declaration
                .name
                .clone()
                .unwrap_or_else(|| self.workspace_crate.package_name.clone());
            let default_path = format!("{}/{name}.rs", directory.join("/"));
            let root = declaration
                .path
                .as_deref()
                .map_or_else(|| self.file_at(&default_path), |path| self.file_at(path));
            if let Some(root) = root {
                by_root.push((name, root));
            }
        }
        if is_discovering {
            let mut discovered = discovered_root_files(&self.components_of_files, directory);
            if kind == TargetKind::Binary {
                if let Some(main) = self.file_at("src/main.rs") {
                    discovered.push((self.workspace_crate.package_name.clone(), main));
                }
            }
            for (name, root) in discovered {
                if !by_root.iter().any(|(_, existing)| *existing == root) {
                    by_root.push((name, root));
                }
            }
        }
        by_root.sort();
        by_root
            .into_iter()
            .map(|(name, root)| Target {
                crate_index: self.crate_index,
                kind,
                name,
                root_file: Some(root),
            })
            .collect()
    }

    fn discover(&self) -> Vec<Target> {
        let manifest = &self.workspace_crate.manifest;
        let mut targets = Vec::new();
        let library_path = manifest
            .library
            .as_ref()
            .and_then(|declared| declared.path.clone())
            .unwrap_or_else(|| "src/lib.rs".to_string());
        let has_library = if let Some(root) = self.file_at(&library_path) {
            targets.push(Target {
                crate_index: self.crate_index,
                kind: TargetKind::Library,
                name: self.workspace_crate.crate_identifier.clone(),
                root_file: Some(root),
            });
            true
        } else {
            false
        };
        targets.extend(self.targets_of_kind(
            TargetKind::Binary,
            &manifest.binaries,
            manifest.discovers_binaries,
            &["src", "bin"],
        ));
        targets.extend(self.targets_of_kind(
            TargetKind::IntegrationTest,
            &manifest.integration_tests,
            manifest.discovers_integration_tests,
            &["tests"],
        ));
        targets.extend(self.targets_of_kind(
            TargetKind::Example,
            &manifest.examples,
            manifest.discovers_examples,
            &["examples"],
        ));
        targets.extend(self.targets_of_kind(
            TargetKind::Benchmark,
            &manifest.benchmarks,
            manifest.discovers_benchmarks,
            &["benches"],
        ));
        let build_root = match &manifest.build_script {
            BuildScriptDeclaration::Discovered => self.file_at("build.rs"),
            BuildScriptDeclaration::Path(path) => self.file_at(path),
            BuildScriptDeclaration::Disabled => None,
        };
        if let Some(root) = build_root {
            targets.push(Target {
                crate_index: self.crate_index,
                kind: TargetKind::BuildScript,
                name: "build".to_string(),
                root_file: Some(root),
            });
        }
        if has_library {
            targets.push(Target {
                crate_index: self.crate_index,
                kind: TargetKind::DocumentationTest,
                name: self.workspace_crate.crate_identifier.clone(),
                root_file: None,
            });
        }
        targets
    }
}

/// `mod 名;` 指的文件可能在哪（照 Rust 参考「模块的源文件」：mod-rs 文件与别的文件分开算，`#[path]` 另算）。
fn child_module_candidates(
    declaring_file: &Path,
    is_mod_rs: bool,
    inline_path: &[String],
    module_name: &str,
    path_attribute: Option<&str>,
) -> Vec<PathBuf> {
    let directory = declaring_file
        .parent()
        .map_or_else(PathBuf::new, Path::to_path_buf);
    let stem = declaring_file
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut base = directory.clone();
    if !is_mod_rs {
        base.push(&stem);
    }
    for segment in inline_path {
        base.push(segment);
    }
    match path_attribute {
        Some(path) if inline_path.is_empty() => vec![normalized_path(&directory.join(path))],
        Some(path) => vec![normalized_path(&base.join(path))],
        None => vec![
            normalized_path(&base.join(format!("{module_name}.rs"))),
            normalized_path(&base.join(module_name).join("mod.rs")),
        ],
    }
}

struct ModuleTreeOutcome {
    memberships: Vec<Vec<FileMembership>>,
    missing_module_files: Vec<(UnitIndex, String)>,
}

/// 模块树里等着展开的一份文件：从目标根走到它的模块路径、这一路上与起来的谓词、声明它的那条 `mod`。
struct PendingModuleFile {
    file: FileIndex,
    module_path: Vec<String>,
    alternative: Vec<Predicate>,
    declared_by: Option<UnitIndex>,
}

/// 每份文件的几组条件最多记几组（防 `#[path]` 绕成环时无穷展开；实际仓里最多两组）。
const CONDITION_ALTERNATIVE_LIMIT: usize = 8;

fn resolve_module_trees(
    files: &[SourceFile],
    units: &[Unit],
    unit_ranges: &[(usize, usize)],
    targets: &[Target],
    files_by_path: &HashMap<PathBuf, FileIndex>,
) -> ModuleTreeOutcome {
    let mut memberships: Vec<Vec<FileMembership>> = vec![Vec::new(); files.len()];
    let mut missing_module_files = Vec::new();
    for (target_position, target) in targets.iter().enumerate() {
        let target_index = TargetIndex(target_position);
        let Some(root) = target.root_file else {
            continue;
        };
        let mut pending: VecDeque<PendingModuleFile> = VecDeque::new();
        pending.push_back(PendingModuleFile {
            file: root,
            module_path: Vec::new(),
            alternative: Vec::new(),
            declared_by: None,
        });
        while let Some(PendingModuleFile {
            file,
            module_path,
            alternative,
            declared_by,
        }) = pending.pop_front()
        {
            let existing = memberships[file.0]
                .iter_mut()
                .find(|membership| membership.target == target_index);
            match existing {
                Some(membership) => {
                    if let Some(declaring_unit) = declared_by {
                        if !membership.declared_by.contains(&declaring_unit) {
                            membership.declared_by.push(declaring_unit);
                        }
                    }
                    if membership.condition_alternatives.contains(&alternative)
                        || membership.condition_alternatives.len() >= CONDITION_ALTERNATIVE_LIMIT
                    {
                        continue;
                    }
                    membership.condition_alternatives.push(alternative.clone());
                }
                None => memberships[file.0].push(FileMembership {
                    target: target_index,
                    module_path: module_path.clone(),
                    condition_alternatives: vec![alternative.clone()],
                    declared_by: declared_by.into_iter().collect(),
                }),
            }
            let is_mod_rs = file == root
                || files[file.0]
                    .absolute_path
                    .file_name()
                    .is_some_and(|name| name == "mod.rs");
            let (first_unit, end_unit) = unit_ranges[file.0];
            for (unit_offset, unit) in units[first_unit..end_unit].iter().enumerate() {
                let Some(declaration) = unit.module_declaration.as_ref() else {
                    continue;
                };
                let unit_index = UnitIndex(first_unit + unit_offset);
                let candidates = child_module_candidates(
                    &files[file.0].absolute_path,
                    is_mod_rs,
                    &unit.local_module_path,
                    &declaration.module_name,
                    declaration.path_attribute.as_deref(),
                );
                let found = candidates
                    .iter()
                    .filter_map(|candidate| files_by_path.get(candidate).copied())
                    .find(|candidate| files[candidate.0].language == SourceLanguage::Rust);
                match found {
                    Some(child) => {
                        let mut child_path = module_path.clone();
                        child_path.extend(unit.local_module_path.iter().cloned());
                        child_path.push(declaration.module_name.clone());
                        let mut child_alternative = alternative.clone();
                        child_alternative.extend(unit.conditions.iter().cloned());
                        pending.push_back(PendingModuleFile {
                            file: child,
                            module_path: child_path,
                            alternative: child_alternative,
                            declared_by: Some(unit_index),
                        });
                    }
                    None => missing_module_files.push((
                        unit_index,
                        candidates
                            .iter()
                            .map(|candidate| candidate.display().to_string())
                            .collect::<Vec<String>>()
                            .join(" 或 "),
                    )),
                }
            }
        }
    }
    for file_memberships in &mut memberships {
        file_memberships.sort_by_key(|membership| membership.target);
    }
    ModuleTreeOutcome {
        memberships,
        missing_module_files,
    }
}

fn target_marker(target: &Target) -> Option<String> {
    match target.kind {
        TargetKind::Library | TargetKind::DocumentationTest => None,
        TargetKind::Binary => Some(format!("<bin {}>", target.name)),
        TargetKind::IntegrationTest => Some(format!("<test {}>", target.name)),
        TargetKind::Example => Some(format!("<example {}>", target.name)),
        TargetKind::Benchmark => Some(format!("<bench {}>", target.name)),
        TargetKind::BuildScript => Some("<build>".to_string()),
    }
}

/// 一份文件的模块前缀：crate 名、非库目标的标记、模块路径；哪个目标都不属于的写 `<unlinked 路径>`。
fn module_prefix_of_file(
    file: &SourceFile,
    memberships: &[FileMembership],
    targets: &[Target],
    crates: &[WorkspaceCrate],
) -> Vec<String> {
    let mut prefix = vec![crates[file.crate_index.0].crate_identifier.clone()];
    match memberships.first() {
        Some(membership) => {
            if let Some(marker) = target_marker(&targets[membership.target.0]) {
                prefix.push(marker);
            }
            prefix.extend(membership.module_path.iter().cloned());
        }
        None => prefix.push(format!("<unlinked {}>", file.relative_path)),
    }
    prefix
}

fn assign_qualified_names(
    units: &mut [Unit],
    files: &[SourceFile],
    memberships: &[Vec<FileMembership>],
    targets: &[Target],
    crates: &[WorkspaceCrate],
) {
    let prefixes: Vec<Vec<String>> = files
        .iter()
        .enumerate()
        .map(|(file_position, file)| {
            module_prefix_of_file(file, &memberships[file_position], targets, crates)
        })
        .collect();
    for unit in units.iter_mut() {
        if unit.kind == UnitKind::DocumentationTest {
            continue;
        }
        let mut segments = prefixes[unit.file.0].clone();
        segments.extend(unit.local_module_path.iter().cloned());
        if unit.container.is_some() {
            if let Some(label) = &unit.container_label {
                segments.push(label.clone());
            }
        }
        segments.push(unit.name.clone());
        unit.qualified_name = segments.join("::");
    }
    for unit_position in 0..units.len() {
        let owner_name = match &units[unit_position].documentation_test_of {
            Some(DocumentationTestAttachment::Item(owner)) => units[owner.0].qualified_name.clone(),
            Some(DocumentationTestAttachment::Module(module_path)) => {
                let mut segments = prefixes[units[unit_position].file.0].clone();
                segments.extend(module_path.iter().cloned());
                segments.join("::")
            }
            None => continue,
        };
        units[unit_position].qualified_name =
            format!("{owner_name}::{}", units[unit_position].name);
    }
}

/// `const _: … = …;` 这类没有名字的条目切分时记的名字。它不是 R-裸名的着落：代码里点不到它，`let _ = …`、`_ =>`、`|_|` 里的 `_`
/// 只是通配，照名字 `_` 去找会把这类条目（常是挂在特性上的编译期断言）拉进闭包，带不带那个特性两趟的键就分了
/// （crates 红批报告「OUT_DIR 里 judging_units_keys.tsv 默认构建对 gpu 构建」）。与 `scripts/gate/lib_function_units.py` 建按名字索引时
/// 不收 `short == "_"` 的那一处同一条规则。
pub const UNNAMED_ITEM_NAME: &str = "_";

fn build_indexes(
    units: &[Unit],
    memberships: &[Vec<FileMembership>],
    files_by_path: HashMap<PathBuf, FileIndex>,
) -> WorkspaceIndexes {
    let mut indexes = WorkspaceIndexes {
        units_by_target_and_name: HashMap::new(),
        members_by_name: HashMap::new(),
        members_of_container: HashMap::new(),
        implementations_by_self_type_name: HashMap::new(),
        trait_implementations_by_trait_name: HashMap::new(),
        blanket_implementations_by_trait_name: HashMap::new(),
        uses_by_scope: HashMap::new(),
        documentation_tests_of_item: HashMap::new(),
        documentation_tests_of_module: HashMap::new(),
        files_by_path,
    };
    for (unit_position, unit) in units.iter().enumerate() {
        let unit_index = UnitIndex(unit_position);
        if let Some(container) = unit.container {
            indexes
                .members_by_name
                .entry(unit.name.clone())
                .or_default()
                .push(unit_index);
            indexes
                .members_of_container
                .entry(container)
                .or_default()
                .push(unit_index);
        } else if unit.kind.is_found_by_bare_name() && unit.name != UNNAMED_ITEM_NAME {
            for membership in &memberships[unit.file.0] {
                let by_name = indexes
                    .units_by_target_and_name
                    .entry(membership.target)
                    .or_default();
                for name in &unit.declared_names {
                    by_name.entry(name.clone()).or_default().push(unit_index);
                }
            }
        }
        if let Some(implementation) = &unit.implementation {
            for name in &implementation.self_type_names {
                indexes
                    .implementations_by_self_type_name
                    .entry(name.clone())
                    .or_default()
                    .push(unit_index);
            }
            if implementation.is_trait_implementation {
                if let Some(trait_name) = &implementation.trait_name {
                    indexes
                        .trait_implementations_by_trait_name
                        .entry(trait_name.clone())
                        .or_default()
                        .push(unit_index);
                    if implementation.is_blanket_implementation {
                        indexes
                            .blanket_implementations_by_trait_name
                            .entry(trait_name.clone())
                            .or_default()
                            .push(unit_index);
                    }
                }
            }
        }
        if unit.kind == UnitKind::Use {
            indexes
                .uses_by_scope
                .entry((unit.file, unit.local_module_path.clone()))
                .or_default()
                .push(unit_index);
        }
        match &unit.documentation_test_of {
            Some(DocumentationTestAttachment::Item(owner)) => {
                indexes
                    .documentation_tests_of_item
                    .entry(*owner)
                    .or_default()
                    .push(unit_index);
            }
            Some(DocumentationTestAttachment::Module(module_path)) => {
                indexes
                    .documentation_tests_of_module
                    .entry((unit.file, module_path.clone()))
                    .or_default()
                    .push(unit_index);
            }
            None => {}
        }
    }
    indexes
}

/// 装一个工作区：`root` 是仓根或 `crates/` 目录。
///
/// # Errors
/// 根目录读不了、没有 crate、或某份 `Cargo.toml` 读不了。
pub fn load_workspace(
    root: &Path,
    threads: NonZeroUsize,
    switches: &RuleSwitches,
) -> Result<Workspace, WorkspaceError> {
    let canonical_root =
        std::fs::canonicalize(root).map_err(|error| WorkspaceError::RootUnreadable {
            path: root.to_path_buf(),
            reason: error.to_string(),
        })?;
    let (crates_directory, repository_root) = if canonical_root.join("crates").is_dir() {
        (canonical_root.join("crates"), canonical_root.clone())
    } else {
        let parent = canonical_root
            .parent()
            .map_or_else(|| canonical_root.clone(), Path::to_path_buf);
        (canonical_root.clone(), parent)
    };
    let mut crate_directories: Vec<PathBuf> = std::fs::read_dir(&crates_directory)
        .map_err(|error| WorkspaceError::RootUnreadable {
            path: crates_directory.clone(),
            reason: error.to_string(),
        })?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir() && path.join("Cargo.toml").is_file())
        .collect();
    crate_directories.sort();
    if crate_directories.is_empty() {
        return Err(WorkspaceError::NoCrates {
            path: crates_directory,
        });
    }
    let mut crates = Vec::new();
    for directory in crate_directories {
        let manifest_path = directory.join("Cargo.toml");
        let text = std::fs::read_to_string(&manifest_path).map_err(|error| {
            WorkspaceError::RootUnreadable {
                path: manifest_path.clone(),
                reason: error.to_string(),
            }
        })?;
        let manifest =
            parse_crate_manifest(&text).map_err(|error| WorkspaceError::ManifestUnreadable {
                path: manifest_path.clone(),
                error,
            })?;
        let crate_identifier = manifest
            .library
            .as_ref()
            .and_then(|library| library.name.clone())
            .unwrap_or_else(|| manifest.package_name.replace('-', "_"));
        crates.push(WorkspaceCrate {
            package_name: manifest.package_name.clone(),
            crate_identifier,
            directory,
            manifest,
        });
    }
    let mut entries = Vec::new();
    for (crate_position, workspace_crate) in crates.iter().enumerate() {
        let mut found = Vec::new();
        collect_source_files(&workspace_crate.directory, &mut found);
        for absolute_path in found {
            let language = if absolute_path
                .extension()
                .is_some_and(|extension| extension == "wgsl")
            {
                SourceLanguage::Wgsl
            } else {
                SourceLanguage::Rust
            };
            entries.push(FileEntry {
                relative_path: relative_display(&absolute_path, &repository_root),
                absolute_path: normalized_path(&absolute_path),
                crate_index: CrateIndex(crate_position),
                language,
            });
        }
    }
    entries.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    let parsed = parse_files_in_parallel(&entries, &crates, threads, switches);
    let mut files = Vec::with_capacity(entries.len());
    let mut units = Vec::new();
    let mut wgsl_units = Vec::new();
    let mut unit_ranges = Vec::with_capacity(entries.len());
    for (entry, parsed_source) in entries.into_iter().zip(parsed) {
        let offset = units.len();
        for mut unit in parsed_source.units {
            offset_unit_indexes(&mut unit, offset);
            units.push(unit);
        }
        unit_ranges.push((offset, units.len()));
        wgsl_units.extend(parsed_source.wgsl_units);
        files.push(SourceFile {
            relative_path: entry.relative_path,
            absolute_path: entry.absolute_path,
            crate_index: entry.crate_index,
            language: entry.language,
            source: parsed_source.source,
            tokens: parsed_source.tokens,
            file_level_refusals: parsed_source.file_level_refusals,
        });
    }
    let files_by_path: HashMap<PathBuf, FileIndex> = files
        .iter()
        .enumerate()
        .map(|(file_position, file)| (file.absolute_path.clone(), FileIndex(file_position)))
        .collect();
    let mut targets = Vec::new();
    for (crate_position, workspace_crate) in crates.iter().enumerate() {
        let components_of_files = files
            .iter()
            .enumerate()
            .filter(|(_, file)| file.crate_index == CrateIndex(crate_position))
            .map(|(file_position, file)| {
                (
                    FileIndex(file_position),
                    crate_relative_components(file, &workspace_crate.directory),
                )
            })
            .collect();
        let discovery = TargetDiscovery {
            crate_index: CrateIndex(crate_position),
            workspace_crate,
            files_by_path: &files_by_path,
            components_of_files,
        };
        targets.extend(discovery.discover());
    }
    let module_trees = resolve_module_trees(&files, &units, &unit_ranges, &targets, &files_by_path);
    let mut reported_missing = BTreeSet::new();
    for (unit_index, candidates) in module_trees.missing_module_files {
        if reported_missing.insert(unit_index) {
            let unit = &units[unit_index.0];
            files[unit.file.0].file_level_refusals.push(Refusal {
                file: unit.file,
                line: unit.start_line,
                form: RefusalForm::ModuleFileMissing,
                fragment: candidates,
            });
        }
    }
    let memberships = module_trees.memberships;
    let mut files_of_target = vec![Vec::new(); targets.len()];
    for (file_position, file_memberships) in memberships.iter().enumerate() {
        for membership in file_memberships {
            files_of_target[membership.target.0].push(FileIndex(file_position));
        }
    }
    assign_qualified_names(&mut units, &files, &memberships, &targets, &crates);
    let indexes = build_indexes(&units, &memberships, files_by_path);
    Ok(Workspace {
        repository_root,
        crates,
        files,
        units,
        wgsl_units,
        targets,
        memberships,
        files_of_target,
        indexes,
    })
}

impl Workspace {
    /// 一份文件在一个目标里的那条归属。
    #[must_use]
    pub fn membership(&self, file: FileIndex, target: TargetIndex) -> Option<&FileMembership> {
        self.memberships[file.0]
            .iter()
            .find(|membership| membership.target == target)
    }

    /// 一个 crate 的某一种目标（按名字找；库、构建脚本、文档测试每个 crate 至多一个，名字不看）。
    #[must_use]
    pub fn target_of_crate(
        &self,
        crate_index: CrateIndex,
        kind: TargetKind,
        name: Option<&str>,
    ) -> Option<TargetIndex> {
        self.targets
            .iter()
            .position(|target| {
                target.crate_index == crate_index
                    && target.kind == kind
                    && name.is_none_or(|wanted| target.name == wanted)
            })
            .map(TargetIndex)
    }

    /// 一个目标根文件里的 `fn main`。
    #[must_use]
    pub fn main_function_of(&self, target: TargetIndex) -> Option<UnitIndex> {
        let root = self.targets[target.0].root_file?;
        self.units
            .iter()
            .position(|unit| {
                unit.file == root
                    && unit.kind == UnitKind::Function
                    && unit.name == "main"
                    && unit.container.is_none()
                    && unit.local_module_path.is_empty()
            })
            .map(UnitIndex)
    }

    /// 一个绝对路径写成从仓根起的路径（仓外的照原样）。
    #[must_use]
    pub fn display_path(&self, absolute_path: &Path) -> String {
        relative_display(absolute_path, &self.repository_root)
    }

    /// 按名字（包名或代码里点它的名字）找 crate。
    #[must_use]
    pub fn crate_named(&self, name: &str) -> Option<CrateIndex> {
        let normalized = name.replace('-', "_");
        self.crates
            .iter()
            .position(|workspace_crate| {
                workspace_crate.package_name == name
                    || workspace_crate.crate_identifier == normalized
            })
            .map(CrateIndex)
    }
}
