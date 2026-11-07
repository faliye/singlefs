//! 工作区：从根目录找到每个 crate、读清单、找编译目标，切全部 `.rs` 与 `.wgsl`（按文件切片并行，合并按文件次序，
//! 多线程与单线程逐字相同），按 `mod` 声明（连 `#[path]`）连出每个编译目标的模块树，再建闭包要查的索引。
//!
//! 根目录可以是仓根（底下有 `crates/`），也可以就是 `crates/` 本身；输出里的文件路径都从仓根算（`crates/…`）。
//!
//! 装出来的那一份是源码原样（形式化判定用）；产品行为判定用 [`Workspace::view_for_judgment`] 交的视图：同一批文件的原文剥掉 Verus 的
//! 规格与证明块（`verus_ghost_code`）之后再切一遍、再连模块树与索引，第一次要时才切。

use std::collections::{BTreeSet, HashMap, VecDeque};
use std::fmt;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::conditional_compilation::Predicate;
use crate::lexer::{identifier_name, token_text, Token, TokenKind};
use crate::manifest::{
    parse_crate_manifest, BuildScriptDeclaration, CrateManifest, DeclaredTarget, ManifestError,
};
use crate::rule_switches::{BrokenRule, RuleSwitches};
use crate::rust_items::{is_keyword, normalized_path, parse_rust_file, RustFileInput};
use crate::units::{
    DocumentationTestAttachment, FileIndex, Refusal, RefusalForm, Unit, UnitIndex, UnitKind,
};
use crate::verus_ghost_code::{source_with_verus_ghost_code_blanked, JudgmentKind, VerusGhostCode};
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
    pub origin: CrateOrigin,
}

/// 一个 crate 是怎么收进来的：工作区成员（`crates/` 下的），还是顺 path 依赖收进来的工作区外 crate（`prune-server/` 这一类）。
/// 工作区外的按调用关系连，不按同名连（`closure` 的 `follow_bare_names`；与 `scripts/gate/lib_function_units.py` 的
/// `RustCrate.outside_workspace` 同一个分法）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrateOrigin {
    UnderTheCratesDirectory,
    PathDependencyOutsideTheWorkspace,
}

/// 别的包按裸名连得到一个工作区外 crate 的哪些名字（与 `scripts/gate/lib_function_units.py` 的
/// `RustWorkspace.names_reaching_outside_crate` 同一个算法，`names_reaching_an_outside_crate`）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NamesReachingAnOutsideCrate {
    /// 别的包的 use 语句里，这个 crate 的库名（或从它引进的名字）后面有 `*`：哪个名字都可能引进来了。
    EveryName,
    /// 经 use、`extern crate` 或路径从这个 crate 引进来的名字。
    Named(BTreeSet<String>),
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
    /// 条目级宏调用 → 调用体里跟在 `enum` / `struct` / `union` / `type` 后面的名字（宏生成的类型）：调用进闭包时这些名字算被点名。
    pub types_declared_by_item_macro: HashMap<UnitIndex, Vec<String>>,
    /// 带 `#[global_allocator]` 的 static：链接集里有它所在的目标，它就进闭包。
    pub global_allocators: Vec<UnitIndex>,
    /// 外部类型上的本仓 trait impl 头：R-方法、R-路径不等自身类型被点名就放行它的成员（自身类型永远点不到）。
    pub foreign_type_trait_implementations: BTreeSet<UnitIndex>,
    /// 工作区外的 crate → 别的包按裸名连得到它的哪些名字（只给 [`CrateOrigin::PathDependencyOutsideTheWorkspace`] 的 crate 算）。
    pub names_reaching_crates_outside_the_workspace:
        HashMap<CrateIndex, NamesReachingAnOutsideCrate>,
}

/// 闭包外的四类边里在装工作区时建的索引（`scripts/gate/lib_function_units.py` 的 `RustWorkspace.index_macro_and_foreign_edges`
/// 同一趟扫法，只扫至少属于一个编译目标的文件里的单元，文档测试不扫）。
struct EdgesOutsideTheEightRules {
    types_declared_by_item_macro: HashMap<UnitIndex, Vec<String>>,
    /// 宏名 → 同名 `macro_rules!` 展开式顶层按字面写出的名字（几份同名定义合起来算）。
    literal_names_of_macro_rules: HashMap<String, BTreeSet<String>>,
    global_allocators: Vec<UnitIndex>,
    foreign_type_trait_implementations: BTreeSet<UnitIndex>,
}

/// 宏体里跟在这几个关键字后面、按字面写出的名字，是展开结果顶层的条目名。
const ITEM_KEYWORDS_IN_A_MACRO_EXPANSION: [&str; 7] =
    ["const", "static", "fn", "struct", "enum", "union", "type"];
/// 条目级宏调用体里跟在这几个关键字后面的名字是宏生成的类型。
const TYPE_KEYWORDS_IN_AN_ITEM_MACRO: [&str; 4] = ["enum", "struct", "union", "type"];
/// `macro_rules!` 里展开结果顶层所在的括号深度：宏体的括号一层、展开式的括号一层（`impl $name { fn new … }` 里的关联项更深，不算：
/// 它们按类型连，按名字登会让每一处 `X::new()` 都拉进这个宏的全部调用）。
const MACRO_EXPANSION_TOP_LEVEL_BRACKET_DEPTH: isize = 2;
/// impl 自身类型路径的头一段是这几个时指的是本 crate 里的东西，不算外部类型。
const LOCAL_PATH_ROOTS: [&str; 4] = ["crate", "self", "super", "Self"];

fn edges_outside_the_eight_rules(
    units: &[Unit],
    files: &[SourceFile],
    memberships: &[Vec<FileMembership>],
    crates: &[WorkspaceCrate],
) -> EdgesOutsideTheEightRules {
    let mut edges = EdgesOutsideTheEightRules {
        types_declared_by_item_macro: HashMap::new(),
        literal_names_of_macro_rules: HashMap::new(),
        global_allocators: Vec::new(),
        foreign_type_trait_implementations: BTreeSet::new(),
    };
    let mut type_names: BTreeSet<String> = BTreeSet::new();
    let mut trait_names: BTreeSet<String> = BTreeSet::new();
    let mut trait_implementations: Vec<UnitIndex> = Vec::new();
    for (unit_position, unit) in units.iter().enumerate() {
        let unit_index = UnitIndex(unit_position);
        let Some((first_token, end_token)) = unit.token_range else {
            continue;
        };
        if unit.kind == UnitKind::DocumentationTest || memberships[unit.file.0].is_empty() {
            continue;
        }
        let file = &files[unit.file.0];
        let text_at = move |position: usize| {
            file.tokens
                .get(position)
                .map_or("", |token| token_text(&file.source, token))
        };
        let is_identifier_at = move |position: usize| {
            file.tokens
                .get(position)
                .is_some_and(|token| token.kind == TokenKind::Identifier)
        };
        match unit.kind {
            UnitKind::ItemMacroInvocation => {
                let declared: BTreeSet<String> = (first_token..end_token)
                    .filter(|position| {
                        TYPE_KEYWORDS_IN_AN_ITEM_MACRO.contains(&text_at(*position))
                            && is_identifier_at(position + 1)
                            && !is_keyword(identifier_name(text_at(position + 1)))
                    })
                    .map(|position| identifier_name(text_at(position + 1)).to_string())
                    .collect();
                type_names.extend(declared.iter().cloned());
                edges
                    .types_declared_by_item_macro
                    .insert(unit_index, declared.into_iter().collect());
            }
            UnitKind::MacroRulesDefinition => {
                let mut bracket_depth: isize = 0;
                for position in first_token..end_token {
                    let word = text_at(position);
                    match word {
                        "{" | "(" | "[" => bracket_depth += 1,
                        "}" | ")" | "]" => bracket_depth -= 1,
                        _ => {}
                    }
                    if bracket_depth != MACRO_EXPANSION_TOP_LEVEL_BRACKET_DEPTH
                        || !ITEM_KEYWORDS_IN_A_MACRO_EXPANSION.contains(&word)
                    {
                        continue;
                    }
                    let name_position = if word == "static" && text_at(position + 1) == "mut" {
                        position + 2
                    } else {
                        position + 1
                    };
                    let name = identifier_name(text_at(name_position));
                    if is_identifier_at(name_position)
                        && !name.starts_with('$')
                        && !is_keyword(name)
                        && name != UNNAMED_ITEM_NAME
                    {
                        edges
                            .literal_names_of_macro_rules
                            .entry(unit.name.clone())
                            .or_default()
                            .insert(name.to_string());
                    }
                }
            }
            UnitKind::Static => {
                let is_global_allocator = (first_token..end_token).any(|position| {
                    text_at(position) == "global_allocator"
                        && position >= first_token + 2
                        && text_at(position - 1) == "["
                        && text_at(position - 2) == "#"
                });
                if is_global_allocator {
                    edges.global_allocators.push(unit_index);
                }
            }
            UnitKind::Struct | UnitKind::Enumeration | UnitKind::Union | UnitKind::TypeAlias => {
                type_names.insert(unit.name.clone());
            }
            UnitKind::TraitHead => {
                trait_names.insert(unit.name.clone());
            }
            UnitKind::ImplementationHead => {
                if unit.implementation.as_ref().is_some_and(|shape| {
                    shape.is_trait_implementation && !shape.is_blanket_implementation
                }) {
                    trait_implementations.push(unit_index);
                }
            }
            UnitKind::Function
            | UnitKind::Constant
            | UnitKind::Use
            | UnitKind::ModuleDeclaration
            | UnitKind::ModuleHead
            | UnitKind::ExternBlock
            | UnitKind::ExternCrate
            | UnitKind::VerusBroadcastGroup
            | UnitKind::VerusAssumeSpecification
            | UnitKind::DocumentationTest => {}
        }
    }
    let local_path_roots: BTreeSet<&str> = LOCAL_PATH_ROOTS
        .iter()
        .copied()
        .chain(
            crates
                .iter()
                .map(|workspace_crate| workspace_crate.crate_identifier.as_str()),
        )
        .collect();
    for implementation_head in trait_implementations {
        let shape = units[implementation_head.0]
            .implementation
            .as_ref()
            .expect("只收了带 ImplementationShape 的 trait impl 头");
        let is_written_as_an_outside_path = shape
            .self_type_path_root
            .as_deref()
            .is_some_and(|root| !local_path_roots.contains(root));
        let names_no_local_type = !shape
            .self_type_names
            .iter()
            .any(|name| type_names.contains(name));
        let implements_a_local_trait = shape
            .trait_name
            .as_ref()
            .is_some_and(|trait_name| trait_names.contains(trait_name));
        if implements_a_local_trait && (is_written_as_an_outside_path || names_no_local_type) {
            edges
                .foreign_type_trait_implementations
                .insert(implementation_head);
        }
    }
    edges
}

/// use 语句里根名后面不当引进名字的几个词。
const PATH_WORDS_THAT_IMPORT_NOTHING: [&str; 4] = ["self", "super", "crate", "as"];
/// 宏调用名后面接的开括号。
const MACRO_INVOCATION_OPENING_BRACKETS: [&str; 3] = ["(", "[", "{"];

/// 每个工作区外 crate 的 [`NamesReachingAnOutsideCrate`]。
fn names_reaching_crates_outside_the_workspace(
    units: &[Unit],
    files: &[SourceFile],
    memberships: &[Vec<FileMembership>],
    targets: &[Target],
    crates: &[WorkspaceCrate],
) -> HashMap<CrateIndex, NamesReachingAnOutsideCrate> {
    let mut reaching = HashMap::new();
    for (crate_position, workspace_crate) in crates.iter().enumerate() {
        match workspace_crate.origin {
            CrateOrigin::UnderTheCratesDirectory => {}
            CrateOrigin::PathDependencyOutsideTheWorkspace => {
                let outside_crate = CrateIndex(crate_position);
                reaching.insert(
                    outside_crate,
                    names_reaching_an_outside_crate(
                        outside_crate,
                        workspace_crate,
                        units,
                        files,
                        memberships,
                        targets,
                    ),
                );
            }
        }
    }
    reaching
}

/// 别的包按裸名连得到工作区外这个 crate 的哪些名字。扫的是这个 crate 以外每个编译目标的文件：根名起先是这个 crate 的库名；
/// use / `extern crate` 语句里出现根名的，它后面的名字都算引进来、也都当根名（引进来的可能是模块或别名），收到不再变；语句里根名后面有 `*` 的全收。
/// 路径 `根名::a::b` 里根名后面的段都算；名字后面接 `!` 与开括号、又是这个 crate 里 `macro_rules!` 的名字的也算。引进来的名字按整个工作区收、
/// 不分模块：经别处 `pub use` 转出去再 glob 引进来的照样认得。
fn names_reaching_an_outside_crate(
    outside_crate: CrateIndex,
    workspace_crate: &WorkspaceCrate,
    units: &[Unit],
    files: &[SourceFile],
    memberships: &[Vec<FileMembership>],
    targets: &[Target],
) -> NamesReachingAnOutsideCrate {
    let is_in_a_target_of_the_outside_crate = |file_position: usize| {
        memberships[file_position]
            .iter()
            .any(|membership| targets[membership.target.0].crate_index == outside_crate)
    };
    let scanned_files: Vec<&SourceFile> = files
        .iter()
        .enumerate()
        .filter(|(file_position, _)| {
            !is_in_a_target_of_the_outside_crate(*file_position)
                && memberships[*file_position]
                    .iter()
                    .any(|membership| targets[membership.target.0].crate_index != outside_crate)
        })
        .map(|(_, file)| file)
        .collect();
    let macro_names: BTreeSet<&str> = units
        .iter()
        .filter(|unit| {
            unit.kind == UnitKind::MacroRulesDefinition
                && is_in_a_target_of_the_outside_crate(unit.file.0)
        })
        .map(|unit| unit.name.as_str())
        .collect();
    // （文件，语句里 use / crate 之后的头一个词法单元，语句收尾的 `;`）
    let mut statements: Vec<(&SourceFile, usize, usize)> = Vec::new();
    for file in scanned_files.iter().copied() {
        for position in 0..file.tokens.len() {
            let word = file_token_text(file, position);
            if !is_file_token_identifier(file, position) || !matches!(word, "use" | "crate") {
                continue;
            }
            if word == "crate"
                && position
                    .checked_sub(1)
                    .is_none_or(|previous| file_token_text(file, previous) != "extern")
            {
                continue;
            }
            let mut end = position + 1;
            while end < file.tokens.len() && file_token_text(file, end) != ";" {
                end += 1;
            }
            statements.push((file, position + 1, end));
        }
    }
    let mut roots: BTreeSet<String> = BTreeSet::from([workspace_crate.crate_identifier.clone()]);
    let mut is_every_name_imported = false;
    let mut has_grown = true;
    // 根名只增不减、上界是语句里标识符的个数：收到一趟不再长就停
    while has_grown {
        has_grown = false;
        for (file, low, high) in &statements {
            let Some(start) = (*low..*high).find(|position| {
                is_file_token_identifier(file, *position)
                    && roots.contains(file_token_text(file, *position))
            }) else {
                continue;
            };
            for position in start + 1..*high {
                let word = file_token_text(file, position);
                if word == "*" {
                    is_every_name_imported = true;
                } else if is_file_token_identifier(file, position)
                    && !PATH_WORDS_THAT_IMPORT_NOTHING.contains(&word)
                    && !roots.contains(word)
                {
                    roots.insert(word.to_string());
                    has_grown = true;
                }
            }
        }
    }
    if is_every_name_imported {
        return NamesReachingAnOutsideCrate::EveryName;
    }
    let mut names = roots.clone();
    for file in scanned_files.iter().copied() {
        for position in 0..file.tokens.len() {
            if !is_file_token_identifier(file, position) {
                continue;
            }
            let word = file_token_text(file, position);
            if macro_names.contains(word)
                && file_token_text(file, position + 1) == "!"
                && MACRO_INVOCATION_OPENING_BRACKETS.contains(&file_token_text(file, position + 2))
            {
                names.insert(word.to_string());
            }
            if !roots.contains(word) {
                continue;
            }
            let mut following = position + 1;
            while file_token_text(file, following) == ":"
                && file_token_text(file, following + 1) == ":"
                && is_file_token_identifier(file, following + 2)
            {
                names.insert(file_token_text(file, following + 2).to_string());
                following += 3;
            }
        }
    }
    NamesReachingAnOutsideCrate::Named(names)
}

/// 文件里第 `position` 个词法单元的原文；越界交空串。
fn file_token_text(file: &SourceFile, position: usize) -> &str {
    file.tokens
        .get(position)
        .map_or("", |token| token_text(&file.source, token))
}

/// 文件里第 `position` 个词法单元是标识符（关键字也算）；越界交假。
fn is_file_token_identifier(file: &SourceFile, position: usize) -> bool {
    file.tokens
        .get(position)
        .is_some_and(|token| token.kind == TokenKind::Identifier)
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
    /// 这一份里 Verus 的规格与证明块在不在：`load_workspace` 装出来的是原样，产品行为视图里剥掉了。
    pub verus_ghost_code: VerusGhostCode,
    /// 装这一份用的线程数与弄坏开关：产品行为视图照同样的切法再切一遍。
    loading_threads: NonZeroUsize,
    loading_switches: RuleSwitches,
    /// 产品行为视图（只在原样的那一份上有），第一次要时才切。
    product_behavior_view: OnceLock<Box<Workspace>>,
    /// 闭包读到的 shell 脚本各自（传递地）source 进来的文件：一份脚本在这个工作区里只折一趟（`shell_sources`）。
    pub shell_sourced_files: crate::shell_sources::ShellSourcedFilesCache,
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

/// 切分时每份文件的源码从哪来。
#[derive(Clone, Copy)]
enum SourceOrigin<'loaded> {
    /// 从盘上读（`load_workspace`）。
    ReadFromDisk,
    /// 已装好的原样那一份里的原文（按文件下标一一对应），剥掉 Verus 的规格与证明块（产品行为视图）。
    VerusGhostCodeBlankedFrom(&'loaded [SourceFile]),
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

/// 这份文件拿去切的源码，连同剥不了 Verus 规格与证明块时记的那一条拒算（那时原文原样拿去切）。
fn source_to_split(
    file: FileIndex,
    entry: &FileEntry,
    origin: SourceOrigin<'_>,
) -> (String, Option<Refusal>) {
    match origin {
        SourceOrigin::ReadFromDisk => (
            std::fs::read(&entry.absolute_path)
                .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
                .unwrap_or_default(),
            None,
        ),
        SourceOrigin::VerusGhostCodeBlankedFrom(loaded_files) => {
            let original = &loaded_files[file.0].source;
            match entry.language {
                SourceLanguage::Wgsl => (original.clone(), None),
                SourceLanguage::Rust => match source_with_verus_ghost_code_blanked(original) {
                    Ok(blanked) => (blanked, None),
                    Err(unparsed) => (
                        original.clone(),
                        Some(Refusal {
                            file: crate::units::RefusalFile::WorkspaceSourceFile(file),
                            line: unparsed.line,
                            form: RefusalForm::VerusGhostCodeUnparsed,
                            fragment: unparsed.reason,
                        }),
                    ),
                },
            }
        }
    }
}

fn parse_one_file(
    file: FileIndex,
    entry: &FileEntry,
    crates: &[WorkspaceCrate],
    switches: &RuleSwitches,
    origin: SourceOrigin<'_>,
) -> ParsedSource {
    let (source, unparsed_verus_block) = source_to_split(file, entry, origin);
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
            let mut file_level_refusals = parsed.file_level_refusals;
            file_level_refusals.extend(unparsed_verus_block);
            ParsedSource {
                tokens: parsed.tokens,
                units: parsed.units,
                wgsl_units: Vec::new(),
                file_level_refusals,
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
    origin: SourceOrigin<'_>,
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
                            (file, parse_one_file(file, entry, crates, switches, origin))
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
    files: &[SourceFile],
    memberships: &[Vec<FileMembership>],
    targets: &[Target],
    crates: &[WorkspaceCrate],
    files_by_path: HashMap<PathBuf, FileIndex>,
    switches: &RuleSwitches,
) -> WorkspaceIndexes {
    let edges = edges_outside_the_eight_rules(units, files, memberships, crates);
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
        types_declared_by_item_macro: edges.types_declared_by_item_macro,
        global_allocators: edges.global_allocators,
        foreign_type_trait_implementations: edges.foreign_type_trait_implementations,
        names_reaching_crates_outside_the_workspace: names_reaching_crates_outside_the_workspace(
            units,
            files,
            memberships,
            targets,
            crates,
        ),
    };
    let links_macro_rules_literal_names = switches.is_intact(BrokenRule::MacroRulesLiteralNames);
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
        if unit.kind == UnitKind::ItemMacroInvocation && links_macro_rules_literal_names {
            // 同名 `macro_rules!` 展开式顶层按字面写出的名字登到这次调用名下（调用体里已有的不重登）：点名这个名字的单元连到调用。
            // 只进按名字的索引，不改 `declared_names`（WGSL 外名的着落不变）；与 python 那一份一样不看调用在不在 impl / trait 里。
            let macro_name = unit
                .name
                .split_once("!@")
                .map_or(unit.name.as_str(), |(name_before_the_line, _line)| {
                    name_before_the_line
                });
            if let Some(literal_names) = edges.literal_names_of_macro_rules.get(macro_name) {
                for membership in &memberships[unit.file.0] {
                    let by_name = indexes
                        .units_by_target_and_name
                        .entry(membership.target)
                        .or_default();
                    for name in literal_names {
                        if !unit.declared_names.contains(name) {
                            by_name.entry(name.clone()).or_default().push(unit_index);
                        }
                    }
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
    // 收完 `crates/` 下的以后，顺各 crate 清单的 path 依赖（普通、开发、构建三种）广度优先收落在仓根底下、有 `Cargo.toml` 的目录
    // （根清单 exclude 里的 `prune-server/` 这一类自己是工作区根的独立工程）；`.`、`..` 起头的、落在仓根外的、清单里写成绝对路径的不收。
    // 与 `scripts/gate/lib_function_units.py` 的 `discover_rust_crates` 同一条规则。弄坏开关 path-dependencies-outside-the-workspace：不收
    let follows_path_dependencies =
        switches.is_intact(BrokenRule::PathDependenciesOutsideTheWorkspace);
    let mut known_directories: BTreeSet<PathBuf> = crate_directories
        .iter()
        .map(|directory| normalized_path(directory.as_path()))
        .collect();
    // `crates/` 下的是工作区成员；之后顺 path 依赖收进来的是工作区外的（`CrateOrigin`）
    let workspace_member_directories = known_directories.clone();
    let mut pending_directories: VecDeque<PathBuf> = crate_directories.into_iter().collect();
    while let Some(directory) = pending_directories.pop_front() {
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
        if follows_path_dependencies {
            for dependency_path in manifest
                .dependencies
                .iter()
                .filter_map(|dependency| dependency.path.as_deref())
            {
                if Path::new(dependency_path).is_absolute() {
                    continue;
                }
                let place = normalized_path(&directory.join(dependency_path));
                let is_under_the_repository_root = place
                    .strip_prefix(&repository_root)
                    .is_ok_and(|relative| !relative.as_os_str().is_empty());
                if !is_under_the_repository_root
                    || known_directories.contains(&place)
                    || !place.join("Cargo.toml").is_file()
                {
                    continue;
                }
                known_directories.insert(place.clone());
                pending_directories.push_back(place);
            }
        }
        let origin = if workspace_member_directories.contains(&normalized_path(&directory)) {
            CrateOrigin::UnderTheCratesDirectory
        } else {
            CrateOrigin::PathDependencyOutsideTheWorkspace
        };
        crates.push(WorkspaceCrate {
            package_name: manifest.package_name.clone(),
            crate_identifier,
            directory,
            manifest,
            origin,
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
    Ok(assemble_workspace(
        repository_root,
        crates,
        entries,
        WorkspaceLoading {
            threads,
            switches,
            origin: SourceOrigin::ReadFromDisk,
            verus_ghost_code: VerusGhostCode::Kept,
        },
    ))
}

/// 一次切分怎么切：线程数、弄坏开关、源码从哪来、这一份里 Verus 的规格与证明块在不在。
struct WorkspaceLoading<'loaded> {
    threads: NonZeroUsize,
    switches: &'loaded RuleSwitches,
    origin: SourceOrigin<'loaded>,
    verus_ghost_code: VerusGhostCode,
}

/// 切全部文件，连模块树、限定名与索引（`load_workspace` 与产品行为视图共用这一段）。
fn assemble_workspace(
    repository_root: PathBuf,
    crates: Vec<WorkspaceCrate>,
    entries: Vec<FileEntry>,
    loading: WorkspaceLoading<'_>,
) -> Workspace {
    let parsed = parse_files_in_parallel(
        &entries,
        &crates,
        loading.threads,
        loading.switches,
        loading.origin,
    );
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
                file: crate::units::RefusalFile::WorkspaceSourceFile(unit.file),
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
    let indexes = build_indexes(
        &units,
        &files,
        &memberships,
        &targets,
        &crates,
        files_by_path,
        loading.switches,
    );
    Workspace {
        repository_root,
        crates,
        files,
        units,
        wgsl_units,
        targets,
        memberships,
        files_of_target,
        indexes,
        verus_ghost_code: loading.verus_ghost_code,
        loading_threads: loading.threads,
        loading_switches: loading.switches.clone(),
        product_behavior_view: OnceLock::new(),
        shell_sourced_files: crate::shell_sources::ShellSourcedFilesCache::default(),
    }
}

impl Workspace {
    /// 一类判定取键用的那一份：形式化判定用装出来的这一份本身（规格与证明都在指纹里）；产品行为判定用剥掉 Verus 规格与证明块的视图
    /// （第一次要时照装这一份的线程数与弄坏开关，从这一份的原文再切一遍，之后复用）。在视图上要产品行为的那一份，交视图自己。
    ///
    /// # Panics
    /// 在产品行为视图上要形式化判定的那一份：视图里规格与证明已经剥掉，交出去就是拿剥过的键判证明（假绿），是调用方的 bug。
    #[must_use]
    pub fn view_for_judgment(&self, kind: JudgmentKind) -> &Workspace {
        match (kind, self.verus_ghost_code) {
            (JudgmentKind::FormalVerification, VerusGhostCode::Kept)
            | (JudgmentKind::ProductBehavior, VerusGhostCode::Blanked) => self,
            (JudgmentKind::ProductBehavior, VerusGhostCode::Kept) => self
                .product_behavior_view
                .get_or_init(|| Box::new(self.product_behavior_view_cut_afresh())),
            (JudgmentKind::FormalVerification, VerusGhostCode::Blanked) => panic!(
                "产品行为视图里 Verus 的规格与证明已经剥掉，形式化判定要用 load_workspace 装出来的那一份"
            ),
        }
    }

    /// 同一批文件的原文剥掉 Verus 的规格与证明块之后再切一遍（不再读盘上的源码；数据文件的摘要照切分的做法现读）。
    fn product_behavior_view_cut_afresh(&self) -> Workspace {
        let entries = self
            .files
            .iter()
            .map(|file| FileEntry {
                absolute_path: file.absolute_path.clone(),
                relative_path: file.relative_path.clone(),
                crate_index: file.crate_index,
                language: file.language,
            })
            .collect();
        assemble_workspace(
            self.repository_root.clone(),
            self.crates.clone(),
            entries,
            WorkspaceLoading {
                threads: self.loading_threads,
                switches: &self.loading_switches,
                origin: SourceOrigin::VerusGhostCodeBlankedFrom(&self.files),
                verus_ghost_code: VerusGhostCode::Blanked,
            },
        )
    }

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
