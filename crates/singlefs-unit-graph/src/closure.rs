//! 从入口收闭包（设计 3.3 的八条规则）、拒算（3.4）、算复用键（3.1）。
//!
//! 每条规则落在哪个函数：
//!   R-目标  `build_link`（入口目标的链接集、每个目标看得见哪几个目标、每个目标的生效 cfg 集）、`follow_bare_names` 里跨目标只认 `pub`
//!   R-裸名  `follow_bare_names`（同一个目标里按模块筛，见下面「R-裸名同一个目标里按模块筛」）
//!   R-路径  `follow_qualified_paths`
//!   R-方法  `follow_methods`（RTA）与 `name_type` 里后点名的类型补上之前见过的方法
//!   R-隐式  `name_type` 里的 trait impl 那一支，与 `visit_unit_for_target` 里 trait 头那一支
//!   R-开放  `open_trait`：入口签名里的 `dyn Trait` 与 trait 约束收链接集里这个 trait 的全部 impl；闭包内部写的 `dyn Trait` 不收——
//!           值只能由某个具体类型经 unsized 转换而来，那个类型在闭包里被点名（R-隐式 带进它的 trait impl）或从入口签名传进来
//!   R-结构  `follow_structure`（成员连头、单元连模块头与声明它所在文件的 `mod`、连引进它用到的名字的 use、glob 一律连、别名按原名解）
//!   R-宏    宏名按裸名解到 `macro_rules!`（`follow_bare_names`），宏体与条目级宏调用里的标识符都在单元的名字里（`rust_items`）
//! 闭包外的四类边（与 `scripts/gate/lib_function_units.py` 文件头 Rust 闭包「闭包外的四类边」同一套规则；索引在 `workspace` 的
//! `edges_outside_the_eight_rules`，每类一个弄坏开关）：
//!   条目级宏调用生成的条目  调用体里跟在 enum / struct / union / type 后面的名字在调用进闭包时算被点名（`visit_unit_for_target`）；
//!           同名 `macro_rules!` 展开式顶层按字面写出的名字登到这个宏的每次调用名下（`workspace` 的 `build_indexes`，R-裸名照它解）
//!   `#[global_allocator]`  链接集里有它所在目标的分配器 static 进闭包（`push_global_allocators_of_the_linked_targets`）
//!   外部类型上的本仓 trait impl  R-方法、R-路径不等自身类型被点名就放行它的成员（`foreign_type_trait_implementations`）
//!   构建脚本交的值  `env!` / `option_env!` 读的非 cargo 变量连到本 crate 构建脚本的 `main`，没有构建脚本的不连（`follow_reads`）
//!   工作区外的 path 依赖  `load_workspace` 顺 path 依赖收仓根底下、`crates/` 之外的 crate，调到它们的函数照 R-目标连；这样收进来的 crate
//!           按调用关系连、不按同名连（工作区成员之间跨目标照旧按同名连）：别的包里的裸名要这个名字在工作区里经 use 或路径从它引进来过
//!           （`workspace` 的 `names_reaching_an_outside_crate`）才连，只是同名的字段、局部变量不连；同一个目标里按模块筛照 R-裸名。各一个弄坏开关
//! R-裸名同一个目标里按模块筛（`crates/` 下的包与工作区外的 crate 一样，各一个弄坏开关）：只连单元所在模块看得见的那几份
//! （`candidates_seen_from_the_module_of_the_unit`：本模块定义的、经 use 引进的、glob use 的模块里的、写成 `模块::名` 的），点不准的退回全连；
//! 名字是 use 改名的原名、单元是 `macro_rules!`（体里的名字在调用处解）的不筛。跨目标的只认 `pub`、不按模块筛。
//! 拒算：单元自己带的（`rust_items` 切分时记下）、链接到的文件切不全的、数据文件读不了的、构建脚本或 bin 找不到的、
//! proc-macro crate、cfg 谓词判不了的、工具链的目标三元组查不出的、WGSL 外名没有着落的，全部收进 `ClosureOutcome::refusals`；有一条拒算就不交键。
//! WGSL 外名（`.wgsl` 里用到、模块里没定义的名字）的着落（`WgslExternalNameSources::supply`）：整名是闭包里某个 Rust 串字面量的一个词；
//! 或者闭包里某个单元按名字定义的名字（常量、静态量、条目级宏调用里的标识符这类）就是它、或以 `_` 加它收尾（例：宏调用里的
//! `GPU_FAILURE_POINT_UNIT_UNREADABLE` 去掉 `GPU_` 拼成内核里的 `FAILURE_POINT_UNIT_UNREADABLE`）；或者首段加 `_`（按第一个 `_` 切，
//! 至少两个字符，例 `T_`）落在闭包里某个 Rust 单元的串字面量或 `macro_rules!` 体里、且其余那一段（不分大小写）是闭包里某个串字面量的
//! 一个词或某个名字（例：`format!("const T_{}", 表名大写)` 拼 `T_HEADER`，表名 `"header"` 在串字面量里）。只首段落在一个无关的串里不算。
//! 着落做成边（`pull_in_wgsl_name_suppliers`）：外名先在链接集里别的 `.wgsl` 里找——哪个单元 `include_str!` 读进的 `.wgsl` 定义了它，
//! 就把那个单元连同那份 `.wgsl` 并进闭包，名字在合起来的名字空间里解；哪份 `.wgsl` 都没定义的、闭包里又还没有着落的，再在链接集里找供它的
//! 单元（按名字定义了它或以 `_` 加它收尾的常量、静态量、条目级宏调用，串字面量里有它这个词的单元），连同它们的闭包并进入口闭包，再收、
//! 再核，直到不再有新单元；最后还没有着落的拒算（`wgsl-undefined-name`）。
//! 这是超集：供名字的单元一律连上，不核它读的 `.wgsl`、它的串真被交给了拼内核的函数；只是带 `#[test]` 的函数与只在测试里编进来的代码
//! （`#[cfg(test)]` 模块里的单元）不当供名字的单元收（测试函数没人调，它的串到不了拼内核的函数；入口本身在闭包里，照旧算着落）。
//! 子进程测试边（`current-exe-child`，`pull_in_child_process_tests`）：单元把 `current_exe()` 交给 `Command::new` 起本测试二进制、
//! `--exact` 后面跟着折得出的测试名（字面串、同一 crate 的 `const …: &str`、全由它们组成的 `format!` / `concat!`；或起子进程的函数的
//! 一个参数，取闭包里每一处调用在那个位置上交的实参），就连到入口目标里那个 `#[test] fn`；折不出、解不到恰好一个的拒算 `current-exe`。
//! 单元里不起子进程（没有 `Command`、`spawn` 这类词）、只问 `current_exe()` 路径的不拒算、不连边（`rust_items` 的 `SPAWNING_WORDS`）。

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt;

use crate::build_features::FeaturesEnabledByTheBuild;
use crate::conditional_compilation::{
    atom_of_the_leaf, leaves_of_the_predicate, parse_predicate, plain_string_content,
    rendered_predicate, split_at_top_level_commas, target_platform_of_triple,
    CompilationConditions, ConditionalCompilationSet, Evaluation, Predicate, TargetPlatform,
};
use crate::lexer::{lex_source, token_text, DocumentationCommentTreatment, TokenKind};
use crate::manifest::DependencySection;
use crate::rule_switches::{BrokenRule, RuleSwitches};
use crate::rust_items::{normalized_path, test_name_pieces_of_texts};
use crate::sha256::{sha256_of, Sha256Digest};
use crate::shell_sources::ShellNormalizationFailure;
use crate::units::{
    ChildTestName, DocumentationTestAttachment, FileIndex, ReadOrigin, Refusal, RefusalForm,
    SourceRead, TestNamePiece, UnitIndex, UnitKind, WgslUnitIndex,
};
use crate::wgsl::wgsl_module_closure;
use crate::workspace::{
    CrateIndex, CrateOrigin, NamesReachingAnOutsideCrate, SourceLanguage, TargetIndex, TargetKind,
    TargetSelector, Workspace,
};

/// 命令行 `--cfg` 给的一项：无值名字、特性，或别的键值。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RequestedCondition {
    Name(String),
    Feature(String),
    KeyValue { key: String, value: String },
}

impl RequestedCondition {
    /// 读 `test`、`feature="gpu"`、`键="值"` 这几种写法。
    ///
    /// # Errors
    /// 不是无值名字或键值，交原文。
    pub fn parse(text: &str) -> Result<Self, String> {
        let lexed = lex_source(text, DocumentationCommentTreatment::KeptOutOfTheTokens);
        let texts: Vec<&str> = lexed
            .tokens
            .iter()
            .map(|token| token_text(text, token))
            .collect();
        match parse_predicate(&texts) {
            Predicate::Name(name) => Ok(RequestedCondition::Name(name)),
            Predicate::KeyValue { key, value } if key == "feature" => {
                Ok(RequestedCondition::Feature(value))
            }
            Predicate::KeyValue { key, value } => Ok(RequestedCondition::KeyValue { key, value }),
            Predicate::All(_)
            | Predicate::Any(_)
            | Predicate::Not(_)
            | Predicate::Unrecognized(_) => Err(text.to_string()),
        }
    }
}

/// 一次收闭包的请求：入口 fn 路径（`<crate>::<模块路径>::<fn>`，impl 方法写 `<crate>::<模块>::<类型>::<方法>`；
/// 文档测试写它所挂的条目或模块的路径）、目标、`--cfg`、开不开默认特性、工具链的目标三元组
/// （`conditional_compilation::target_triple_of_toolchain_lines` 从工具链行取；平台谓词按它查表，查不出的闭包拒算）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClosureRequest {
    pub entry_path: String,
    pub target: TargetSelector,
    pub requested_conditions: Vec<RequestedCondition>,
    pub uses_default_features: bool,
    pub target_triple: String,
}

/// 收不了闭包：入口找不到或不唯一（用法错），或闭包读到的 shell 脚本交 bash 规范化跑不下去（运行条件不满足）；都不是拒算，整趟报错。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EntryError {
    MalformedEntryPath(String),
    CrateNotFound(String),
    /// 一组入口里另给的入口不在头一个入口的 crate 里（一组入口在同一个目标里收）。
    EntryInAnotherCrate {
        path: String,
        crate_name: String,
    },
    TargetNotFound {
        crate_name: String,
        target: String,
    },
    EntryNotFound(String),
    EntryAmbiguous {
        path: String,
        candidates: Vec<String>,
    },
    EntryInactive(String),
    NoDocumentationTests(String),
    /// 闭包读到的 shell 脚本交 bash 规范化跑不下去（bash 不在、退非 0、超时）：与 `scripts/gate/lib_function_units.py` 抛
    /// `FunctionUnitsError` 同，整趟报错，不记成拒算。
    ShellNormalizationFailed(ShellNormalizationFailure),
}

impl fmt::Display for EntryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntryError::MalformedEntryPath(path) => {
                write!(formatter, "入口「{path}」不是 <crate>::<路径>::<fn> 的写法")
            }
            EntryError::CrateNotFound(name) => {
                write!(formatter, "工作区里没有叫「{name}」的 crate")
            }
            EntryError::EntryInAnotherCrate { path, crate_name } => write!(
                formatter,
                "入口「{path}」不在这一组的 crate「{crate_name}」里（一组入口在同一个目标里收）"
            ),
            EntryError::TargetNotFound { crate_name, target } => {
                write!(formatter, "crate「{crate_name}」没有目标「{target}」")
            }
            EntryError::EntryNotFound(path) => {
                write!(formatter, "在点名的目标里找不到入口「{path}」")
            }
            EntryError::EntryAmbiguous { path, candidates } => write!(
                formatter,
                "入口「{path}」对上 {} 个单元：{}",
                candidates.len(),
                candidates.join("、")
            ),
            EntryError::EntryInactive(path) => write!(
                formatter,
                "入口「{path}」在这组 cfg 下不编进来（库里的单测要带 --cfg test）"
            ),
            EntryError::NoDocumentationTests(path) => {
                write!(formatter, "「{path}」的文档注释里没有 Rust 代码块")
            }
            EntryError::ShellNormalizationFailed(failure) => write!(formatter, "{failure}"),
        }
    }
}

/// 数据单元读的是一份文件还是一整个目录。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataUnitKind {
    File,
    /// `crates/` 之下的一个目录（递归），摘要是 `units::directory_digest`。
    Directory,
}

impl DataUnitKind {
    /// 输出行的种类名。与 `scripts/gate/lib_function_units.py` 同一套写法。
    #[must_use]
    pub fn output_name(self) -> &'static str {
        match self {
            DataUnitKind::File => "data",
            DataUnitKind::Directory => "data-directory",
        }
    }
}

/// 闭包里的一个数据单元：读它的单元、数据文件或目录（从仓根起的路径，只当索引）、读的那一行、内容摘要、读的是文件还是目录。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DataUnit {
    pub site: UnitIndex,
    pub path_display: String,
    pub site_line: u32,
    pub digest: Sha256Digest,
    pub kind: DataUnitKind,
}

/// 一次收闭包的结果。
#[derive(Clone, Debug)]
pub struct ClosureOutcome {
    pub entry_target_kind: TargetKind,
    /// 从哪几个入口单元收的：请求点名的入口与另给的入口各自解到的单元（文档测试目标是所挂条目或模块的全部文档测试），
    /// 链进来的别的目标的 `main` 不算。复用键的 entry 行取它们（[`reuse_key`]）。
    pub entry_units: Vec<UnitIndex>,
    /// 闭包里各单元真引到的条件原子，各带收它进来那个目标下的真值（`<原子>\t<true|false>`，排好序、去重）：键的条件成分只取它们。
    pub condition_atoms_with_their_values: Vec<String>,
    pub units: Vec<UnitIndex>,
    pub wgsl_units: Vec<WgslUnitIndex>,
    pub data_units: Vec<DataUnit>,
    pub refusals: Vec<Refusal>,
}

/// 调用方拼进键的外部输入：工具链（`rustc -vV` 与解析后的构建环境）与依赖解析结果，各是原文行。本 crate 不跑 rustc；
/// 依赖行由调用方自己收窄好交进来，或经 `lock_file::narrowed_lock_lines` 从整份 `Cargo.lock` 只取闭包真引到的外部包。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExternalKeyInputs {
    pub toolchain_lines: Vec<String>,
    pub lock_lines: Vec<String>,
}

/// 入口目标的条件：特性、`test`、命令行给的别的名字与键值、开不开默认特性，连同这一次编译里 cargo 给各依赖包并上来的特性
/// （`build_features`；解特性时这些包一被链进来就照请求）。
struct EntryConditions {
    features: BTreeSet<String>,
    is_test: bool,
    extra_names: BTreeSet<String>,
    extra_key_values: BTreeSet<(String, String)>,
    uses_default_features: bool,
    features_enabled_on_the_dependencies_by_the_build: BTreeMap<CrateIndex, BTreeSet<String>>,
}

/// 一次解析特性：每个被链进来的 crate 开了哪些特性、依赖边。
struct FeatureResolution {
    features: BTreeMap<CrateIndex, BTreeSet<String>>,
    root_edges: BTreeSet<CrateIndex>,
    normal_edges: BTreeMap<CrateIndex, BTreeSet<CrateIndex>>,
}

/// 链接集：哪些目标链进来、每个目标看得见哪几个目标、每个目标的生效 cfg 集。
struct Link {
    targets: BTreeSet<TargetIndex>,
    visible: HashMap<TargetIndex, Vec<TargetIndex>>,
    condition_sets: HashMap<TargetIndex, ConditionalCompilationSet>,
    included_crates: BTreeSet<CrateIndex>,
}

impl Link {
    fn visible_targets(&self, target: TargetIndex) -> Vec<TargetIndex> {
        self.visible
            .get(&target)
            .cloned()
            .unwrap_or_else(|| vec![target])
    }

    fn condition_set(&self, target: TargetIndex) -> ConditionalCompilationSet {
        self.condition_sets
            .get(&target)
            .cloned()
            .unwrap_or_else(ConditionalCompilationSet::for_recognition_only)
    }
}

pub(crate) fn dependency_crate_of(
    workspace: &Workspace,
    crate_index: CrateIndex,
    path: &str,
) -> Option<CrateIndex> {
    let wanted = normalized_path(&workspace.crates[crate_index.0].directory.join(path));
    workspace
        .crates
        .iter()
        .position(|workspace_crate| normalized_path(&workspace_crate.directory) == wanted)
        .map(CrateIndex)
}

fn request_feature(
    workspace: &Workspace,
    crate_index: CrateIndex,
    feature: &str,
    features: &mut BTreeMap<CrateIndex, BTreeSet<String>>,
    enabled_optional: &mut BTreeMap<CrateIndex, BTreeSet<String>>,
) {
    if workspace.crates[crate_index.0]
        .manifest
        .features
        .contains_key(feature)
    {
        features
            .entry(crate_index)
            .or_default()
            .insert(feature.to_string());
    } else {
        enabled_optional
            .entry(crate_index)
            .or_default()
            .insert(feature.to_string());
    }
}

/// 照 cargo 的特性统一（在这一次链接里）：根 crate 的请求起，展开 `[features]`（`dep:x`、`x/y`、`x?/y`），顺依赖边把请求传下去，到不再变为止。
/// 非根的 crate 只顺普通依赖（它们的 dev 依赖不编）；同一次构建里别的根（别的包的测试与它们的 dev 依赖）并上来的特性从这一根的清单里看不出，
/// 由 `features_enabled_by_the_build` 交进来：哪个 crate 一被链进来，就照这一次编译的样子再请求它那几个特性。
fn resolve_features(
    workspace: &Workspace,
    root_crate: CrateIndex,
    root_sections: &[DependencySection],
    requested: &BTreeSet<String>,
    uses_default_features: bool,
    features_enabled_by_the_build: &BTreeMap<CrateIndex, BTreeSet<String>>,
) -> FeatureResolution {
    let mut features: BTreeMap<CrateIndex, BTreeSet<String>> = BTreeMap::new();
    let mut enabled_optional: BTreeMap<CrateIndex, BTreeSet<String>> = BTreeMap::new();
    let mut root_edges = BTreeSet::new();
    let mut normal_edges: BTreeMap<CrateIndex, BTreeSet<CrateIndex>> = BTreeMap::new();
    features.insert(root_crate, BTreeSet::new());
    for feature in requested {
        request_feature(
            workspace,
            root_crate,
            feature,
            &mut features,
            &mut enabled_optional,
        );
    }
    if uses_default_features {
        request_feature(
            workspace,
            root_crate,
            "default",
            &mut features,
            &mut enabled_optional,
        );
    }
    loop {
        let size_before: usize = features.values().map(BTreeSet::len).sum::<usize>()
            + enabled_optional.values().map(BTreeSet::len).sum::<usize>()
            + features.len();
        let crates_now: Vec<CrateIndex> = features.keys().copied().collect();
        for crate_index in crates_now {
            for feature_enabled_by_the_build in features_enabled_by_the_build
                .get(&crate_index)
                .into_iter()
                .flatten()
            {
                request_feature(
                    workspace,
                    crate_index,
                    feature_enabled_by_the_build,
                    &mut features,
                    &mut enabled_optional,
                );
            }
            let manifest = &workspace.crates[crate_index.0].manifest;
            let enabled_now: Vec<String> = features
                .get(&crate_index)
                .map(|set| set.iter().cloned().collect())
                .unwrap_or_default();
            for feature in enabled_now {
                let Some(values) = manifest.features.get(&feature) else {
                    continue;
                };
                for value in values {
                    if let Some(dependency_key) = value.strip_prefix("dep:") {
                        enabled_optional
                            .entry(crate_index)
                            .or_default()
                            .insert(dependency_key.to_string());
                    } else if let Some((dependency_part, dependency_feature)) =
                        value.split_once('/')
                    {
                        let is_weak = dependency_part.ends_with('?');
                        let dependency_key = dependency_part.trim_end_matches('?');
                        if !is_weak {
                            enabled_optional
                                .entry(crate_index)
                                .or_default()
                                .insert(dependency_key.to_string());
                        }
                        let dependency_crate = manifest
                            .dependencies
                            .iter()
                            .find(|dependency| dependency.key == dependency_key)
                            .and_then(|dependency| dependency.path.as_deref())
                            .and_then(|path| dependency_crate_of(workspace, crate_index, path));
                        if let Some(dependency_crate) = dependency_crate {
                            request_feature(
                                workspace,
                                dependency_crate,
                                dependency_feature,
                                &mut features,
                                &mut enabled_optional,
                            );
                        }
                    } else {
                        request_feature(
                            workspace,
                            crate_index,
                            value,
                            &mut features,
                            &mut enabled_optional,
                        );
                    }
                }
            }
            let sections: &[DependencySection] = if crate_index == root_crate {
                root_sections
            } else {
                &[DependencySection::Normal]
            };
            for dependency in &manifest.dependencies {
                if !sections.contains(&dependency.section) {
                    continue;
                }
                let Some(path) = dependency.path.as_deref() else {
                    continue;
                };
                let Some(dependency_crate) = dependency_crate_of(workspace, crate_index, path)
                else {
                    continue;
                };
                let is_enabled = !dependency.is_optional
                    || enabled_optional
                        .get(&crate_index)
                        .is_some_and(|enabled| enabled.contains(&dependency.key))
                    || features
                        .get(&crate_index)
                        .is_some_and(|enabled| enabled.contains(&dependency.key));
                if !is_enabled {
                    continue;
                }
                features.entry(dependency_crate).or_default();
                for feature in &dependency.features {
                    request_feature(
                        workspace,
                        dependency_crate,
                        feature,
                        &mut features,
                        &mut enabled_optional,
                    );
                }
                if dependency.uses_default_features {
                    request_feature(
                        workspace,
                        dependency_crate,
                        "default",
                        &mut features,
                        &mut enabled_optional,
                    );
                }
                if crate_index == root_crate {
                    root_edges.insert(dependency_crate);
                }
                if dependency.section == DependencySection::Normal {
                    normal_edges
                        .entry(crate_index)
                        .or_default()
                        .insert(dependency_crate);
                }
            }
        }
        let size_after: usize = features.values().map(BTreeSet::len).sum::<usize>()
            + enabled_optional.values().map(BTreeSet::len).sum::<usize>()
            + features.len();
        if size_after == size_before {
            return FeatureResolution {
                features,
                root_edges,
                normal_edges,
            };
        }
    }
}

fn dependency_sections_of(kind: TargetKind, is_test: bool) -> Vec<DependencySection> {
    match kind {
        TargetKind::BuildScript => vec![DependencySection::Build],
        TargetKind::Library | TargetKind::Binary if !is_test => vec![DependencySection::Normal],
        TargetKind::Library
        | TargetKind::Binary
        | TargetKind::IntegrationTest
        | TargetKind::Example
        | TargetKind::Benchmark
        | TargetKind::DocumentationTest => {
            vec![DependencySection::Normal, DependencySection::Development]
        }
    }
}

/// 一个根目标（入口，或闭包要求补进来的构建脚本、bin）看得见的目标：它自己、本 crate 的库（链得到时）、依赖 crate 的库（传递）。
fn visible_from_root(
    workspace: &Workspace,
    root: TargetIndex,
    resolution: &FeatureResolution,
) -> Vec<TargetIndex> {
    let root_target = &workspace.targets[root.0];
    let mut visible = vec![root];
    if root_target.kind.links_its_own_library() {
        if let Some(library) =
            workspace.target_of_crate(root_target.crate_index, TargetKind::Library, None)
        {
            visible.push(library);
        }
    }
    let mut reached: BTreeSet<CrateIndex> = BTreeSet::new();
    let mut pending: Vec<CrateIndex> = resolution.root_edges.iter().copied().collect();
    if root_target.kind.links_its_own_library() {
        if let Some(own_edges) = resolution.normal_edges.get(&root_target.crate_index) {
            pending.extend(own_edges.iter().copied());
        }
    }
    while let Some(crate_index) = pending.pop() {
        if !reached.insert(crate_index) {
            continue;
        }
        if let Some(edges) = resolution.normal_edges.get(&crate_index) {
            pending.extend(edges.iter().copied());
        }
    }
    for crate_index in reached {
        if let Some(library) = workspace.target_of_crate(crate_index, TargetKind::Library, None) {
            if !visible.contains(&library) {
                visible.push(library);
            }
        }
    }
    visible
}

/// 一个依赖库看得见的目标：它自己与它传递依赖的库。
fn visible_from_library(
    workspace: &Workspace,
    library: TargetIndex,
    resolution: &FeatureResolution,
) -> Vec<TargetIndex> {
    let crate_of_library = workspace.targets[library.0].crate_index;
    let mut visible = vec![library];
    let mut reached: BTreeSet<CrateIndex> = BTreeSet::new();
    let mut pending: Vec<CrateIndex> = resolution
        .normal_edges
        .get(&crate_of_library)
        .map(|edges| edges.iter().copied().collect())
        .unwrap_or_default();
    while let Some(crate_index) = pending.pop() {
        if !reached.insert(crate_index) {
            continue;
        }
        if let Some(edges) = resolution.normal_edges.get(&crate_index) {
            pending.extend(edges.iter().copied());
        }
    }
    for crate_index in reached {
        if let Some(dependency_library) =
            workspace.target_of_crate(crate_index, TargetKind::Library, None)
        {
            visible.push(dependency_library);
        }
    }
    visible
}

/// R-目标：入口目标与要补进来的根目标各解一次特性，合成一个链接集。
fn build_link(
    workspace: &Workspace,
    entry_target: TargetIndex,
    entry_conditions: &EntryConditions,
    platform: Option<&TargetPlatform>,
    extra_roots: &BTreeSet<TargetIndex>,
    switches: &RuleSwitches,
) -> Link {
    let mut roots: Vec<(TargetIndex, bool)> = vec![(entry_target, true)];
    roots.extend(extra_roots.iter().map(|root| (*root, false)));
    let mut link = Link {
        targets: BTreeSet::new(),
        visible: HashMap::new(),
        condition_sets: HashMap::new(),
        included_crates: BTreeSet::new(),
    };
    let mut merged_features: BTreeMap<CrateIndex, BTreeSet<String>> = BTreeMap::new();
    for (root, is_entry) in roots {
        let root_target = &workspace.targets[root.0];
        let is_test = is_entry && entry_conditions.is_test;
        let (requested, uses_default) = if is_entry {
            (
                entry_conditions.features.clone(),
                entry_conditions.uses_default_features,
            )
        } else {
            (BTreeSet::new(), true)
        };
        // 构建脚本是给宿主编的，它的依赖不与目标那一侧并特性（resolver 2）：并上来的特性只给目标那一侧的根
        let no_features_enabled_by_the_build = BTreeMap::new();
        let features_enabled_by_the_build = match root_target.kind {
            TargetKind::BuildScript => &no_features_enabled_by_the_build,
            TargetKind::Library
            | TargetKind::Binary
            | TargetKind::IntegrationTest
            | TargetKind::Example
            | TargetKind::Benchmark
            | TargetKind::DocumentationTest => {
                &entry_conditions.features_enabled_on_the_dependencies_by_the_build
            }
        };
        let resolution = resolve_features(
            workspace,
            root_target.crate_index,
            &dependency_sections_of(root_target.kind, is_test),
            &requested,
            uses_default,
            features_enabled_by_the_build,
        );
        let root_visible = visible_from_root(workspace, root, &resolution);
        for visible_target in &root_visible {
            link.targets.insert(*visible_target);
            if *visible_target != root
                && workspace.targets[visible_target.0].kind == TargetKind::Library
            {
                link.visible.entry(*visible_target).or_insert_with(|| {
                    visible_from_library(workspace, *visible_target, &resolution)
                });
            }
        }
        link.visible.insert(root, root_visible);
        for (crate_index, features) in resolution.features {
            link.included_crates.insert(crate_index);
            merged_features
                .entry(crate_index)
                .or_default()
                .extend(features);
        }
        if is_entry {
            link.condition_sets.insert(
                root,
                ConditionalCompilationSet::for_target(
                    platform,
                    &CompilationConditions {
                        features: merged_features
                            .get(&root_target.crate_index)
                            .cloned()
                            .unwrap_or_default(),
                        is_test: entry_conditions.is_test,
                        extra_names: entry_conditions.extra_names.clone(),
                        extra_key_values: entry_conditions.extra_key_values.clone(),
                    },
                ),
            );
        }
    }
    if !switches.is_intact(BrokenRule::TargetScope) {
        let every_target: Vec<TargetIndex> =
            (0..workspace.targets.len()).map(TargetIndex).collect();
        link.targets = every_target.iter().copied().collect();
        for target in &every_target {
            link.visible.insert(*target, every_target.clone());
        }
    }
    let linked: Vec<TargetIndex> = link.targets.iter().copied().collect();
    for target in linked {
        if link.condition_sets.contains_key(&target) {
            continue;
        }
        let crate_index = workspace.targets[target.0].crate_index;
        link.condition_sets.insert(
            target,
            ConditionalCompilationSet::for_target(
                platform,
                &CompilationConditions {
                    features: merged_features
                        .get(&crate_index)
                        .cloned()
                        .unwrap_or_default(),
                    is_test: false,
                    extra_names: BTreeSet::new(),
                    extra_key_values: BTreeSet::new(),
                },
            ),
        );
    }
    if !switches.is_intact(BrokenRule::ConditionalCompilation) {
        for set in link.condition_sets.values_mut() {
            *set = set.clone().holding_every_predicate();
        }
    }
    link
}

enum Activity {
    Inactive,
    Active,
    ActiveWithUnknownCondition(String),
}

/// 一个单元在一个目标、一组 cfg 下编不编进来：它自己（连外层模块、impl 头）的条件，再加它所在文件从目标根经哪几条 `mod` 连进来的条件
/// （文件的哪一条连法成立都行；文档测试单元不看文件那一层）。
fn unit_activity(
    workspace: &Workspace,
    set: &ConditionalCompilationSet,
    unit_index: UnitIndex,
    target: TargetIndex,
) -> Activity {
    let unit = &workspace.units[unit_index.0];
    let mut unknown_fragment = None;
    match set.evaluate_conjunction(&unit.conditions) {
        Evaluation::DoesNotHold => return Activity::Inactive,
        Evaluation::Unknown => unknown_fragment = Some(rendered_conjunction(&unit.conditions)),
        Evaluation::Holds => {}
    }
    if unit.kind != UnitKind::DocumentationTest {
        let Some(membership) = workspace.membership(unit.file, target) else {
            return Activity::Inactive;
        };
        let mut any_holds = false;
        let mut unknown_alternative = None;
        for alternative in &membership.condition_alternatives {
            match set.evaluate_conjunction(alternative) {
                Evaluation::Holds => any_holds = true,
                Evaluation::Unknown => {
                    unknown_alternative = Some(rendered_conjunction(alternative))
                }
                Evaluation::DoesNotHold => {}
            }
        }
        if !any_holds {
            match unknown_alternative {
                Some(fragment) => unknown_fragment = unknown_fragment.or(Some(fragment)),
                None => return Activity::Inactive,
            }
        }
    }
    match unknown_fragment {
        Some(fragment) => Activity::ActiveWithUnknownCondition(fragment),
        None => Activity::Active,
    }
}

fn rendered_conjunction(predicates: &[Predicate]) -> String {
    predicates
        .iter()
        .map(rendered_predicate)
        .collect::<Vec<String>>()
        .join(" && ")
}

fn add_string_words(literal: &str, words: &mut HashSet<String>) {
    let mut current = String::new();
    for character in literal.chars() {
        if character.is_ascii_alphanumeric() || character == '_' {
            current.push(character);
        } else if !current.is_empty() {
            words.insert(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        words.insert(current);
    }
}

/// 外名的首段加 `_` 至少这么多个字符才认（`T_` 认；`_` 起头的名字只剩一个 `_`，照旧要整名落在串字面量里）。
const SHORTEST_WGSL_EXTERNAL_NAME_PREFIX_IN_CHARACTERS: usize = 2;

/// WGSL 外名的首段加 `_`：按第一个 `_` 切，连那个 `_`（`T_HEADER` 取 `T_`，`FAILURE_POINT_NONE` 取 `FAILURE_`）；名字里没有 `_` 交 `None`。
fn wgsl_external_name_prefix(name: &str) -> Option<&str> {
    name.find('_').map(|position| &name[..=position])
}

/// 一个名字与它每个 `_` 之后的尾段（`GPU_FAILURE_POINT_NONE` 交它自己、`FAILURE_POINT_NONE`、`POINT_NONE`、`NONE`）：
/// 外名以 `_` 加它收尾的名字都算供它的名字。
fn name_and_its_underscore_suffixes(name: &str) -> impl Iterator<Item = &str> {
    std::iter::once(name).chain(
        name.match_indices('_')
            .map(move |(position, _)| &name[position + 1..])
            .filter(|suffix| !suffix.is_empty()),
    )
}

/// 按名字定义的名字能供 WGSL 外名的单元种类：常量、静态量与条目级宏调用（宏调用里的标识符都算它可能定义的名字，R-宏）。
fn defines_names_a_kernel_can_use(kind: UnitKind) -> bool {
    match kind {
        UnitKind::Constant | UnitKind::Static | UnitKind::ItemMacroInvocation => true,
        UnitKind::Function
        | UnitKind::ImplementationHead
        | UnitKind::TraitHead
        | UnitKind::Struct
        | UnitKind::Enumeration
        | UnitKind::Union
        | UnitKind::TypeAlias
        | UnitKind::MacroRulesDefinition
        | UnitKind::Use
        | UnitKind::ModuleDeclaration
        | UnitKind::ModuleHead
        | UnitKind::ExternBlock
        | UnitKind::ExternCrate
        | UnitKind::VerusBroadcastGroup
        | UnitKind::VerusAssumeSpecification
        | UnitKind::DocumentationTest => false,
    }
}

/// 闭包里 Rust 单元给 WGSL 外名的着落。
struct WgslExternalNameSources {
    string_words: HashSet<String>,
    string_words_in_upper_case: HashSet<String>,
    texts_holding_prefixes: Vec<String>,
    /// 常量、静态量、条目级宏调用按名字定义的名字，连同它们每个 `_` 之后的尾段。
    defined_names_and_their_suffixes: HashSet<String>,
    /// 闭包里出现的名字（定义的与点名的），大写。
    names_in_upper_case: HashSet<String>,
}

impl WgslExternalNameSources {
    /// 这个外名有没有着落：整名是串字面量里的一个词；整名是闭包里定义的名字或它以 `_` 加整名收尾；或者（`accepts_prefixes` 时）
    /// 首段加 `_` 够长、落在串字面量或宏体原文里，且其余那一段不分大小写是串字面量里的一个词或闭包里的一个名字。
    fn supply(&self, name: &str, accepts_prefixes: bool) -> bool {
        if self.string_words.contains(name) || self.defined_names_and_their_suffixes.contains(name)
        {
            return true;
        }
        if !accepts_prefixes {
            return false;
        }
        let Some(prefix) = wgsl_external_name_prefix(name) else {
            return false;
        };
        let remainder = name[prefix.len()..].to_ascii_uppercase();
        prefix.chars().count() >= SHORTEST_WGSL_EXTERNAL_NAME_PREFIX_IN_CHARACTERS
            && !remainder.is_empty()
            && self
                .texts_holding_prefixes
                .iter()
                .any(|text| text.contains(prefix))
            && (self.string_words_in_upper_case.contains(&remainder)
                || self.names_in_upper_case.contains(&remainder))
    }
}

struct ClosureRun<'workspace> {
    workspace: &'workspace Workspace,
    link: Link,
    /// 入口目标：`current_exe()` 起的就是它编出来的那个测试二进制，子进程测试名在它的文件里解。
    entry_target: TargetIndex,
    switches: RuleSwitches,
    pending: Vec<(UnitIndex, TargetIndex)>,
    seen_pairs: HashSet<(UnitIndex, TargetIndex)>,
    in_closure: BTreeSet<UnitIndex>,
    named_types: HashSet<String>,
    methods_seen: HashSet<String>,
    dyn_traits_seen: HashSet<String>,
    refusals: BTreeSet<Refusal>,
    data_units: BTreeSet<DataUnit>,
    wgsl_files: BTreeSet<crate::units::FileIndex>,
    requested_roots: BTreeSet<TargetIndex>,
    /// 为 WGSL 外名并进来过的供名字单元（`pull_in_wgsl_name_suppliers`）：每个只试一次，cfg 关着进不来的不反复再试。
    wgsl_name_suppliers_tried: HashSet<UnitIndex>,
    /// 闭包里各单元真引到的条件原子与真值（`record_condition_atoms`）。
    condition_atoms: BTreeSet<String>,
    /// 头一份 bash 规范化跑不下去的 shell 脚本（`follow_shell_sources`）：有了它这一趟交 `EntryError::ShellNormalizationFailed`，不交闭包。
    shell_normalization_failure: Option<ShellNormalizationFailure>,
}

/// R-方法：基本类型与 `Self`、`Vec`、`Option`、`Result`、`Box`、`String` 恒被点名。
const ALWAYS_NAMED_TYPES: [&str; 23] = [
    "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "f32",
    "f64", "bool", "char", "str", "String", "Vec", "Option", "Result", "Box", "Self",
];

/// R-裸名按模块筛候选时，路径根名是这几个的指标准库，不点本目标里的哪一份（另几个是看得见的别的 crate 的库名）。
const PATH_ROOTS_OF_THE_STANDARD_CRATES: [&str; 3] = ["std", "core", "alloc"];

impl<'workspace> ClosureRun<'workspace> {
    fn new(
        workspace: &'workspace Workspace,
        link: Link,
        switches: &RuleSwitches,
        entry_target: TargetIndex,
    ) -> Self {
        Self {
            workspace,
            link,
            entry_target,
            switches: switches.clone(),
            pending: Vec::new(),
            seen_pairs: HashSet::new(),
            in_closure: BTreeSet::new(),
            named_types: ALWAYS_NAMED_TYPES
                .iter()
                .map(|name| (*name).to_string())
                .collect(),
            methods_seen: HashSet::new(),
            dyn_traits_seen: HashSet::new(),
            refusals: BTreeSet::new(),
            data_units: BTreeSet::new(),
            wgsl_files: BTreeSet::new(),
            requested_roots: BTreeSet::new(),
            wgsl_name_suppliers_tried: HashSet::new(),
            condition_atoms: BTreeSet::new(),
            shell_normalization_failure: None,
        }
    }

    fn push(&mut self, unit: UnitIndex, target: TargetIndex) {
        if !self.seen_pairs.contains(&(unit, target)) {
            self.pending.push((unit, target));
        }
    }

    fn linked_target_of(&self, unit: UnitIndex) -> Option<TargetIndex> {
        let file = self.workspace.units[unit.0].file;
        self.workspace.memberships[file.0]
            .iter()
            .map(|membership| membership.target)
            .find(|target| self.link.targets.contains(target))
    }

    fn push_with_a_linked_target(&mut self, unit: UnitIndex) {
        if let Some(target) = self.linked_target_of(unit) {
            self.push(unit, target);
        }
    }

    fn is_kind_kept(&self, kind: UnitKind) -> bool {
        match kind {
            UnitKind::MacroRulesDefinition | UnitKind::ItemMacroInvocation => {
                self.switches.is_intact(BrokenRule::MacroUnits)
            }
            UnitKind::Constant | UnitKind::Static => {
                self.switches.is_intact(BrokenRule::ConstantAndStaticUnits)
            }
            UnitKind::DocumentationTest => {
                self.switches.is_intact(BrokenRule::DocumentationTestUnits)
            }
            UnitKind::Function
            | UnitKind::ImplementationHead
            | UnitKind::TraitHead
            | UnitKind::Struct
            | UnitKind::Enumeration
            | UnitKind::Union
            | UnitKind::TypeAlias
            | UnitKind::Use
            | UnitKind::ModuleDeclaration
            | UnitKind::ModuleHead
            | UnitKind::ExternBlock
            | UnitKind::ExternCrate
            | UnitKind::VerusBroadcastGroup
            | UnitKind::VerusAssumeSpecification => true,
        }
    }

    fn activity(&self, unit_index: UnitIndex, target: TargetIndex) -> Activity {
        unit_activity(
            self.workspace,
            &self.link.condition_set(target),
            unit_index,
            target,
        )
    }

    fn drain(&mut self) {
        while let Some((unit, target)) = self.pending.pop() {
            if !self.seen_pairs.insert((unit, target)) {
                continue;
            }
            self.visit_unit_for_target(unit, target);
        }
    }

    fn visit_unit_for_target(&mut self, unit_index: UnitIndex, target: TargetIndex) {
        let workspace = self.workspace;
        let unit = &workspace.units[unit_index.0];
        if !self.is_kind_kept(unit.kind) {
            return;
        }
        match self.activity(unit_index, target) {
            Activity::Inactive => return,
            Activity::Active => {}
            Activity::ActiveWithUnknownCondition(fragment) => {
                self.refusals.insert(Refusal {
                    file: crate::units::RefusalFile::WorkspaceSourceFile(unit.file),
                    line: unit.start_line,
                    form: RefusalForm::UnrecognizedConditionalCompilationPredicate,
                    fragment,
                });
            }
        }
        self.record_condition_atoms(unit_index, target);
        let is_new = self.in_closure.insert(unit_index);
        let alias_originals = self.follow_structure(unit_index, target);
        if unit.kind.resolves_its_own_names() {
            self.follow_bare_names(unit_index, target, &alias_originals);
            self.follow_methods(unit_index);
            self.follow_qualified_paths(unit_index);
            if !self
                .switches
                .is_intact(BrokenRule::DynTraitsOpenEveryImplementation)
            {
                for trait_name in &unit.references.dyn_trait_names {
                    self.open_trait(trait_name);
                }
            }
        }
        if !is_new {
            return;
        }
        if unit.kind.is_type() {
            self.name_type(&unit.name);
        }
        if unit.kind == UnitKind::ItemMacroInvocation
            && self.switches.is_intact(BrokenRule::MacroGeneratedTypes)
        {
            // 条目级宏调用生成的类型算被点名：R-方法、R-隐式照连到写给它的 impl。弄坏开关 macro-generated-types：不算
            if let Some(type_names) = workspace
                .indexes
                .types_declared_by_item_macro
                .get(&unit_index)
            {
                for type_name in type_names {
                    self.name_type(type_name);
                }
            }
        }
        if unit.kind == UnitKind::TraitHead {
            // trait 头进闭包时它的 blanket impl 连同成员一起进
            if let Some(implementations) = workspace
                .indexes
                .blanket_implementations_by_trait_name
                .get(&unit.name)
            {
                for head in implementations {
                    self.push_container_and_members(*head);
                }
            }
        }
        if unit.kind == UnitKind::TraitHead
            && self
                .switches
                .is_intact(BrokenRule::TraitImplementationEdges)
        {
            if let Some(implementations) = workspace
                .indexes
                .implementations_by_self_type_name
                .get(&unit.name)
            {
                for head in implementations {
                    let is_trait_implementation = workspace.units[head.0]
                        .implementation
                        .as_ref()
                        .is_some_and(|shape| shape.is_trait_implementation);
                    if is_trait_implementation {
                        self.push_container_and_members(*head);
                    }
                }
            }
        }
        self.follow_reads(unit_index, target);
        for refusal in &unit.refusals {
            self.refusals.insert(refusal.clone());
        }
    }

    /// 键的条件成分：这个单元在收它进来的这个目标下真引到的条件原子，各带这个目标的生效集合下的真值（`<原子>\t<true|false>`）。
    /// 原子取它自己连外层的条件链与它词法单元里写出来的谓词的每一片叶子；叶子写不成谓词、判不了真假的拒算。闭包没引到的原子不进键：
    /// 闭包不提 `gpu` 的入口，带不带 `gpu` 特性两种目标下键相同。
    fn record_condition_atoms(&mut self, unit_index: UnitIndex, target: TargetIndex) {
        let workspace = self.workspace;
        let unit = &workspace.units[unit_index.0];
        let set = self.link.condition_set(target);
        for predicate in unit
            .conditions
            .iter()
            .chain(&unit.condition_predicates_in_the_tokens)
        {
            for leaf in leaves_of_the_predicate(predicate) {
                let value = match (atom_of_the_leaf(leaf), set.evaluate(leaf)) {
                    (Some(atom), Evaluation::Holds) => format!("{atom}\ttrue"),
                    (Some(atom), Evaluation::DoesNotHold) => format!("{atom}\tfalse"),
                    (Some(_), Evaluation::Unknown) | (None, _) => {
                        self.refusals.insert(Refusal {
                            file: crate::units::RefusalFile::WorkspaceSourceFile(unit.file),
                            line: unit.start_line,
                            form: RefusalForm::UnrecognizedConditionalCompilationPredicate,
                            fragment: rendered_predicate(predicate),
                        });
                        continue;
                    }
                };
                self.condition_atoms.insert(value);
            }
        }
    }

    /// R-结构。交这个单元经 use 别名引到的原名（R-裸名 照原名再解）。
    fn follow_structure(&mut self, unit_index: UnitIndex, target: TargetIndex) -> Vec<String> {
        let workspace = self.workspace;
        let unit = &workspace.units[unit_index.0];
        let mut alias_originals = Vec::new();
        if !self.switches.is_intact(BrokenRule::StructuralEdges)
            || unit.kind == UnitKind::DocumentationTest
        {
            return alias_originals;
        }
        if let Some(container) = unit.container {
            self.push(container, target);
        }
        if let Some(head) = unit.module_head {
            self.push(head, target);
        }
        if let Some(membership) = workspace.membership(unit.file, target) {
            for declaring in &membership.declared_by {
                self.push(*declaring, target);
            }
        }
        let Some(uses) = workspace
            .indexes
            .uses_by_scope
            .get(&(unit.file, unit.local_module_path.clone()))
        else {
            return alias_originals;
        };
        let names = &unit.references.plain_names;
        let methods = &unit.references.method_names;
        for use_index in uses {
            let Some(imports) = workspace.units[use_index.0].imports.as_ref() else {
                continue;
            };
            let touches = imports.is_glob
                || imports.names.iter().any(|name| {
                    names.binary_search(name).is_ok() || methods.binary_search(name).is_ok()
                });
            if !touches {
                continue;
            }
            self.push(*use_index, target);
            for (alias, original) in &imports.aliases {
                if names.binary_search(alias).is_ok() {
                    alias_originals.push(original.clone());
                }
            }
        }
        alias_originals
    }

    fn follow_bare_names(
        &mut self,
        unit_index: UnitIndex,
        target: TargetIndex,
        alias_originals: &[String],
    ) {
        if !self.switches.is_intact(BrokenRule::BareNames) {
            return;
        }
        let workspace = self.workspace;
        let unit = &workspace.units[unit_index.0];
        let visible = self.link.visible_targets(target);
        let ignores_visibility = !self.switches.is_intact(BrokenRule::TargetScope);
        let links_outside_crate_names_only_when_imported = self
            .switches
            .is_intact(BrokenRule::OutsideCrateNamesImportedOnly);
        let links_outside_crate_names_by_module = self
            .switches
            .is_intact(BrokenRule::OutsideCrateNamesByModule);
        let links_crates_directory_names_by_module = self
            .switches
            .is_intact(BrokenRule::CratesDirectoryNamesByModule);
        // macro_rules! 体里的裸名在调用处解，定义所在模块看不出它落在哪一份：一律全连
        let is_resolved_at_the_call_site = unit.kind == UnitKind::MacroRulesDefinition;
        let unit_crate = workspace.targets[target.0].crate_index;
        let roots_naming_another_crate: BTreeSet<&str> = PATH_ROOTS_OF_THE_STANDARD_CRATES
            .iter()
            .copied()
            .chain(
                visible
                    .iter()
                    .map(|visible_target| workspace.targets[visible_target.0].crate_index)
                    .filter(|visible_crate| *visible_crate != unit_crate)
                    .map(|visible_crate| {
                        workspace.crates[visible_crate.0].crate_identifier.as_str()
                    }),
            )
            .collect();
        for name in unit.references.plain_names.iter().chain(alias_originals) {
            for visible_target in &visible {
                let Some(found) = workspace
                    .indexes
                    .units_by_target_and_name
                    .get(visible_target)
                    .and_then(|by_name| by_name.get(name.as_str()))
                else {
                    continue;
                };
                let visible_crate = workspace.targets[visible_target.0].crate_index;
                let seen_from_the_module: Vec<UnitIndex>;
                let mut candidates: &[UnitIndex] = found;
                // 同一个目标里只连这个单元所在模块看得见的那几份（文件头「R-裸名」）：crates/ 下的包与工作区外的 crate 一样
                // （验证包里 CPU 判器的单元经 use 从格式库引进 SLOT_BYTES，不连到 gpu_unit_checks 里私有的同名常量）
                let links_by_module = match workspace.crates[visible_crate.0].origin {
                    CrateOrigin::UnderTheCratesDirectory => links_crates_directory_names_by_module,
                    CrateOrigin::PathDependencyOutsideTheWorkspace => {
                        // 工作区外的 crate 按调用关系连（文件头「工作区外的 path 依赖」）：从别的包连进来，这个名字要在工作区里经 use 或路径
                        // 从它引进来过（验证包的 run 字段不连到 prune-server 的 command_line::run）
                        if links_outside_crate_names_only_when_imported
                            && visible_crate != unit_crate
                            && !self.is_name_imported_from_the_outside_crate(visible_crate, name)
                        {
                            continue;
                        }
                        links_outside_crate_names_by_module
                    }
                };
                if links_by_module
                    && *visible_target == target
                    && !alias_originals.contains(name)
                    && !is_resolved_at_the_call_site
                {
                    seen_from_the_module = self.candidates_seen_from_the_module_of_the_unit(
                        unit_index,
                        target,
                        name,
                        candidates,
                        &roots_naming_another_crate,
                    );
                    candidates = &seen_from_the_module;
                }
                for found_unit in candidates {
                    if *visible_target == target
                        || workspace.units[found_unit.0].is_public
                        || ignores_visibility
                    {
                        self.push(*found_unit, *visible_target);
                    }
                }
            }
        }
    }

    /// 别的包里的裸名 `name` 在工作区里经 use 或路径从工作区外的 `outside_crate` 引进来过（`workspace` 的
    /// `names_reaching_an_outside_crate`）。
    fn is_name_imported_from_the_outside_crate(
        &self,
        outside_crate: CrateIndex,
        name: &str,
    ) -> bool {
        match self
            .workspace
            .indexes
            .names_reaching_crates_outside_the_workspace
            .get(&outside_crate)
            .expect("每个工作区外的 crate 装工作区时都算了引进名字；这里没有说明 CrateOrigin 与索引对不上")
        {
            NamesReachingAnOutsideCrate::EveryName => true,
            NamesReachingAnOutsideCrate::Named(names) => names.contains(name),
        }
    }

    /// 单元在目标里所在的模块路径：文件在目标模块树里的路径接文件里的行内模块路径。
    fn module_path_in_the_target(&self, unit_index: UnitIndex, target: TargetIndex) -> Vec<String> {
        let unit = &self.workspace.units[unit_index.0];
        let mut module_path = self
            .workspace
            .membership(unit.file, target)
            .expect("候选与单元都是按这个目标的模块树找到的，文件在这个目标里")
            .module_path
            .clone();
        module_path.extend(unit.local_module_path.iter().cloned());
        module_path
    }

    /// 同一个目标里，单元按裸名 `name` 连得到 `candidates` 里的哪几份（与 `scripts/gate/lib_function_units.py` 的
    /// `RustWorkspace.candidates_seen_from_module` 同一个判法）：与单元同一模块的；所在模块的 use 引进了 `name`（或是 glob）、use 路径里有
    /// 候选所在模块末段的（`crate` 对根模块、`super` 对上一层、`super::super` 对上两层，`self` 起头对本模块）；单元里写成 `前缀::name`、
    /// 前缀是候选所在模块末段的（前缀是 `super` 时按整条路径里 `super` 的个数往上数层）。
    /// use 路径或 `…::name` 整条路径的根名在 `roots_naming_another_crate` 里的（别的 crate 的库名、`std`、`core`、`alloc`）指的是别的 crate，
    /// 不点这个目标里的哪一份，跳过这一处。
    /// 认不出落在哪一份的退回全连：use 或前缀一份都没点到（经 `pub use` 转出来的）、单元里另有 `use`（函数体里的）、`name` 后面接 `!` 与开括号。
    /// 前缀是本目标的类型或 trait、或 `Self` 的是关联项（R-路径另管），跳过这一处。哪一样都不沾的（字段名、局部变量、impl 成员自己的名字）一份都不连。
    fn candidates_seen_from_the_module_of_the_unit(
        &self,
        unit_index: UnitIndex,
        target: TargetIndex,
        name: &str,
        candidates: &[UnitIndex],
        roots_naming_another_crate: &BTreeSet<&str>,
    ) -> Vec<UnitIndex> {
        let workspace = self.workspace;
        let unit = &workspace.units[unit_index.0];
        let file = &workspace.files[unit.file.0];
        let text_at = |position: usize| {
            file.tokens
                .get(position)
                .map_or("", |token| token_text(&file.source, token))
        };
        let is_identifier_at = |position: usize| {
            file.tokens
                .get(position)
                .is_some_and(|token| token.kind == TokenKind::Identifier)
        };
        let own_module = self.module_path_in_the_target(unit_index, target);
        let candidate_modules: Vec<Vec<String>> = candidates
            .iter()
            .map(|candidate| self.module_path_in_the_target(*candidate, target))
            .collect();
        let mut seen: BTreeSet<usize> = candidate_modules
            .iter()
            .enumerate()
            .filter(|(_, module)| **module == own_module)
            .map(|(candidate_position, _)| candidate_position)
            .collect();
        // 路径（不含 name 本身）点到的候选（下标）；一份都没点到交 None
        let placed = |segments: &[&str]| -> Option<BTreeSet<usize>> {
            // 路径里写了几个 `super` 就往上几层；超出根模块的写法编不过，按根模块算
            let levels_up = segments
                .iter()
                .filter(|segment| **segment == "super")
                .count();
            let found: BTreeSet<usize> = candidate_modules
                .iter()
                .enumerate()
                .filter(|(_, module)| {
                    module
                        .last()
                        .is_some_and(|last_segment| segments.contains(&last_segment.as_str()))
                        || (module.is_empty() && segments.first() == Some(&"crate"))
                        || (levels_up > 0
                            && module[..]
                                == own_module[..own_module.len().saturating_sub(levels_up)])
                        || (segments.first() == Some(&"self") && **module == own_module)
                })
                .map(|(candidate_position, _)| candidate_position)
                .collect();
            (!found.is_empty()).then_some(found)
        };
        let mut is_placement_unknown = false;
        if let Some(uses) = workspace
            .indexes
            .uses_by_scope
            .get(&(unit.file, unit.local_module_path.clone()))
        {
            for use_index in uses {
                let use_unit = &workspace.units[use_index.0];
                let (Some((first_token, end_token)), Some(imports)) =
                    (use_unit.token_range, use_unit.imports.as_ref())
                else {
                    continue;
                };
                if !imports.is_glob
                    && !imports
                        .names
                        .iter()
                        .any(|imported| imported.as_str() == name)
                {
                    continue;
                }
                // use 之后头一个标识符（跳过开头的 `::`）是路径的根名；`use {…}` 没有根名
                let path_root = (first_token..end_token)
                    .skip_while(|position| {
                        !(is_identifier_at(*position) && text_at(*position) == "use")
                    })
                    .skip(1)
                    .find(|position| text_at(*position) != ":")
                    .filter(|position| is_identifier_at(*position))
                    .map(text_at);
                if path_root.is_some_and(|root| roots_naming_another_crate.contains(root)) {
                    continue;
                }
                let segments: Vec<&str> = (first_token..end_token)
                    .filter(|position| is_identifier_at(*position))
                    .map(text_at)
                    .filter(|word| !matches!(*word, "use" | "pub" | "as") && *word != name)
                    .collect();
                match placed(&segments) {
                    Some(hit) => seen.extend(hit),
                    None => is_placement_unknown = true,
                }
            }
        }
        if let (Some((first_token, end_token)), false) = (unit.token_range, is_placement_unknown) {
            let names_in_the_target = workspace.indexes.units_by_target_and_name.get(&target);
            let is_type_or_trait_of_the_target = |prefix: &str| {
                names_in_the_target
                    .and_then(|by_name| by_name.get(prefix))
                    .is_some_and(|units| {
                        units.iter().any(|found_unit| {
                            let kind = workspace.units[found_unit.0].kind;
                            kind.is_type() || kind == UnitKind::TraitHead
                        })
                    })
            };
            for position in first_token..end_token {
                let word = text_at(position);
                if word == "use" && is_identifier_at(position) && position != first_token {
                    is_placement_unknown = true;
                    break;
                }
                if word != name || !is_identifier_at(position) {
                    continue;
                }
                if text_at(position + 1) == "!" && matches!(text_at(position + 2), "(" | "[" | "{")
                {
                    is_placement_unknown = true;
                    break;
                }
                let Some(prefix_position) = position.checked_sub(3) else {
                    continue;
                };
                if text_at(position - 1) != ":"
                    || text_at(position - 2) != ":"
                    || !is_identifier_at(prefix_position)
                {
                    continue;
                }
                let prefix = text_at(prefix_position);
                // 整条路径 `根::…::前缀::name`，从前缀往前顺着 `标识符 ::` 收到根
                let mut path_before_the_name = vec![prefix];
                let mut segment_position = prefix_position;
                while segment_position >= 3
                    && text_at(segment_position - 1) == ":"
                    && text_at(segment_position - 2) == ":"
                    && is_identifier_at(segment_position - 3)
                {
                    segment_position -= 3;
                    path_before_the_name.push(text_at(segment_position));
                }
                path_before_the_name.reverse();
                if path_before_the_name
                    .first()
                    .is_some_and(|root| roots_naming_another_crate.contains(root))
                {
                    continue;
                }
                if prefix == "Self"
                    || (is_type_or_trait_of_the_target(prefix)
                        && !matches!(prefix, "crate" | "self" | "super"))
                {
                    continue;
                }
                let only_the_prefix = [prefix];
                let placement_segments: &[&str] = if prefix == "super" {
                    &path_before_the_name
                } else {
                    &only_the_prefix
                };
                match placed(placement_segments) {
                    Some(hit) => seen.extend(hit),
                    None => {
                        is_placement_unknown = true;
                        break;
                    }
                }
            }
        }
        if is_placement_unknown {
            return candidates.to_vec();
        }
        candidates
            .iter()
            .enumerate()
            .filter(|(candidate_position, _)| seen.contains(candidate_position))
            .map(|(_, candidate)| *candidate)
            .collect()
    }

    fn follow_methods(&mut self, unit_index: UnitIndex) {
        if !self.switches.is_intact(BrokenRule::MethodCalls) {
            return;
        }
        let workspace = self.workspace;
        let unit = &workspace.units[unit_index.0];
        let foreign_type_implementations = self.foreign_type_trait_implementations();
        for method_name in &unit.references.method_names {
            if !self.methods_seen.insert(method_name.clone()) {
                continue;
            }
            let Some(members) = workspace.indexes.members_by_name.get(method_name) else {
                continue;
            };
            for member in members {
                let Some(container) = workspace.units[member.0].container else {
                    continue;
                };
                let container_unit = &workspace.units[container.0];
                let is_eligible = if container_unit.kind == UnitKind::TraitHead {
                    true
                } else if container_unit.kind == UnitKind::ImplementationHead {
                    // blanket impl 够得到的方法算边：自身类型里带它的泛型参数，能被点名的具体名字可能一个都没有，不等自身类型被点名
                    container_unit.implementation.as_ref().is_some_and(|shape| {
                        shape.is_blanket_implementation
                            || shape
                                .self_type_names
                                .iter()
                                .any(|type_name| self.named_types.contains(type_name))
                    }) || foreign_type_implementations.contains(&container)
                } else {
                    false
                };
                if is_eligible {
                    self.push_with_a_linked_target(*member);
                }
            }
        }
    }

    fn follow_qualified_paths(&mut self, unit_index: UnitIndex) {
        if !self.switches.is_intact(BrokenRule::QualifiedPaths) {
            return;
        }
        let workspace = self.workspace;
        let unit = &workspace.units[unit_index.0];
        let own_container = unit
            .container
            .map(|container| &workspace.units[container.0]);
        let foreign_type_implementations = self.foreign_type_trait_implementations();
        for (qualifier, member_name) in &unit.references.qualified_paths {
            let Some(members) = workspace.indexes.members_by_name.get(member_name) else {
                continue;
            };
            for member in members {
                let Some(container) = workspace.units[member.0].container else {
                    continue;
                };
                let container_unit = &workspace.units[container.0];
                let shape = container_unit.implementation.as_ref();
                let self_types_contain = |type_name: &str| {
                    shape.is_some_and(|implementation_shape| {
                        implementation_shape
                            .self_type_names
                            .iter()
                            .any(|name| name == type_name)
                    })
                };
                let is_match = if container_unit.kind == UnitKind::TraitHead {
                    container_unit.name == *qualifier
                        || (qualifier == "Self"
                            && own_container.is_some_and(|own| {
                                own.kind == UnitKind::TraitHead && own.name == container_unit.name
                            }))
                } else if container_unit.kind == UnitKind::ImplementationHead {
                    self_types_contain(qualifier)
                        || (qualifier == "Self"
                            && own_container.is_some_and(|own| {
                                own.implementation.as_ref().is_some_and(|own_shape| {
                                    own_shape
                                        .self_type_names
                                        .iter()
                                        .any(|name| self_types_contain(name))
                                })
                            }))
                        || shape.is_some_and(|implementation_shape| {
                            implementation_shape.is_trait_implementation
                                && implementation_shape.trait_name.as_deref()
                                    == Some(qualifier.as_str())
                                && (implementation_shape.is_blanket_implementation
                                    || implementation_shape
                                        .self_type_names
                                        .iter()
                                        .any(|type_name| self.named_types.contains(type_name))
                                    || foreign_type_implementations.contains(&container))
                        })
                } else {
                    false
                };
                if is_match {
                    self.push_with_a_linked_target(*member);
                }
            }
        }
    }

    /// 外部类型上的本仓 trait impl 头（`workspace` 的 `foreign_type_trait_implementations`）：经泛型约束分派、自身类型永远不会被点名，
    /// R-方法落到 `.x(` 时它的成员 `x` 照连，R-路径 `Tr::x` 同样。弄坏开关 foreign-type-trait-implementations：一个都不放行。
    fn foreign_type_trait_implementations(&self) -> &'workspace BTreeSet<UnitIndex> {
        static NO_IMPLEMENTATIONS: BTreeSet<UnitIndex> = BTreeSet::new();
        if self
            .switches
            .is_intact(BrokenRule::ForeignTypeTraitImplementations)
        {
            &self.workspace.indexes.foreign_type_trait_implementations
        } else {
            &NO_IMPLEMENTATIONS
        }
    }

    /// `#[global_allocator]` 的 static 管整个链接出来的程序：链接集里有它所在的目标，它就进闭包（连同它的类型、R-隐式带上 GlobalAlloc 的 impl）。
    /// 弄坏开关 global-allocator-edges：不进。
    fn push_global_allocators_of_the_linked_targets(&mut self) {
        if !self.switches.is_intact(BrokenRule::GlobalAllocatorEdges) {
            return;
        }
        let workspace = self.workspace;
        for allocator in &workspace.indexes.global_allocators {
            for membership in &workspace.memberships[workspace.units[allocator.0].file.0] {
                if self.link.targets.contains(&membership.target) {
                    self.push(*allocator, membership.target);
                }
            }
        }
    }

    fn push_container_and_members(&mut self, head: UnitIndex) {
        let workspace = self.workspace;
        self.push_with_a_linked_target(head);
        if let Some(members) = workspace.indexes.members_of_container.get(&head) {
            for member in members {
                self.push_with_a_linked_target(*member);
            }
        }
    }

    /// R-开放（CHA）：链接集里这个 trait 的全部 impl 头与成员。只给入口签名里的 trait 名叫（`compute_closure_of_entries` 里逐个入口那一支）；
    /// 闭包内部的 `dyn Trait` 只在弄坏开关 `dyn-traits-open-every-implementation` 打开时叫。
    fn open_trait(&mut self, trait_name: &str) {
        if !self.switches.is_intact(BrokenRule::ClassHierarchy)
            || !self.dyn_traits_seen.insert(trait_name.to_string())
        {
            return;
        }
        let workspace = self.workspace;
        if let Some(implementations) = workspace
            .indexes
            .trait_implementations_by_trait_name
            .get(trait_name)
        {
            for head in implementations {
                self.push_container_and_members(*head);
            }
        }
    }

    /// 类型被点名：R-隐式 给它写的 trait impl 头与全部成员进来；R-方法 补上之前见过名字的方法
    /// （固有 impl 的；R-隐式 被弄坏时 trait impl 的也照这一条补，两条规则各管各的，不互相兜底）。
    fn name_type(&mut self, type_name: &str) {
        if !self.named_types.insert(type_name.to_string()) {
            return;
        }
        let workspace = self.workspace;
        let Some(implementations) = workspace
            .indexes
            .implementations_by_self_type_name
            .get(type_name)
        else {
            return;
        };
        for head in implementations {
            let is_trait_implementation = workspace.units[head.0]
                .implementation
                .as_ref()
                .is_some_and(|shape| shape.is_trait_implementation);
            if is_trait_implementation
                && self
                    .switches
                    .is_intact(BrokenRule::TraitImplementationEdges)
            {
                self.push_container_and_members(*head);
            } else if self.switches.is_intact(BrokenRule::MethodCalls) {
                if let Some(members) = workspace.indexes.members_of_container.get(head) {
                    for member in members {
                        if self.methods_seen.contains(&workspace.units[member.0].name) {
                            self.push_with_a_linked_target(*member);
                        }
                    }
                }
            }
        }
    }

    /// 读到的数据文件是 shell 脚本（`shell_sources::is_shell_script`）：它（传递地）`source` / `.` 进来的仓内文件各记一个数据单元，
    /// 落在读这份脚本的那一处（同一个单元、同一行），字节进键；折不出、落在根外、指到不在的文件、bash 规范化不出来的，照 shell 那一份的位置记拒算；bash 规范化跑不下去（bash 不在、退非 0、超时）
    /// 不是拒算，记下头一份、这一趟交 `EntryError::ShellNormalizationFailed`
    /// （`shell_sources::shell_sourced_files`，与 `scripts/gate/lib_function_units.py` 的 `shell_sourced_files` 同一份规则）。
    /// 仓根外面的数据文件不看（显示路径不是从仓根起的）。
    fn follow_shell_sources(
        &mut self,
        site: UnitIndex,
        script_absolute_path: &std::path::Path,
        site_line: u32,
    ) {
        let workspace = self.workspace;
        if self.shell_normalization_failure.is_some() {
            return;
        }
        let script_path_from_the_repository_root = workspace.display_path(script_absolute_path);
        if script_path_from_the_repository_root.starts_with('/')
            || !crate::shell_sources::is_shell_script(
                &workspace.repository_root,
                &script_path_from_the_repository_root,
            )
        {
            return;
        }
        let sourced = match workspace.shell_sourced_files.sourced_files_of(
            &workspace.repository_root,
            &script_path_from_the_repository_root,
        ) {
            Ok(sourced) => sourced,
            Err(failure) => {
                self.shell_normalization_failure = Some(failure);
                return;
            }
        };
        for sourced_file in &sourced.sourced_files {
            match std::fs::read(workspace.repository_root.join(sourced_file)) {
                Ok(bytes) => {
                    self.data_units.insert(DataUnit {
                        site,
                        path_display: sourced_file.clone(),
                        site_line,
                        digest: sha256_of(&bytes),
                        kind: DataUnitKind::File,
                    });
                }
                Err(_) => {
                    self.refusals.insert(Refusal {
                        file: crate::units::RefusalFile::WorkspaceSourceFile(
                            workspace.units[site.0].file,
                        ),
                        line: site_line,
                        form: RefusalForm::DataFileUnreadable,
                        fragment: sourced_file.clone(),
                    });
                }
            }
        }
        for shell_refusal in &sourced.refusals {
            self.refusals.insert(Refusal {
                file: crate::units::RefusalFile::ShellScriptFromTheRepositoryRoot(
                    shell_refusal.file.clone(),
                ),
                line: shell_refusal.line,
                form: shell_refusal.form,
                fragment: shell_refusal.snippet.clone(),
            });
        }
    }

    fn follow_reads(&mut self, unit_index: UnitIndex, target: TargetIndex) {
        let workspace = self.workspace;
        let unit = &workspace.units[unit_index.0];
        let crate_index = workspace.targets[target.0].crate_index;
        for read in &unit.reads {
            match read {
                SourceRead::File {
                    absolute_path,
                    line,
                    origin,
                    digest,
                } => {
                    let wgsl_file = workspace
                        .indexes
                        .files_by_path
                        .get(absolute_path)
                        .copied()
                        .filter(|file| workspace.files[file.0].language == SourceLanguage::Wgsl);
                    match (origin, wgsl_file, digest) {
                        (ReadOrigin::Included, Some(file), _) => {
                            self.wgsl_files.insert(file);
                        }
                        (ReadOrigin::Included | ReadOrigin::Runtime, _, Some(digest)) => {
                            self.data_units.insert(DataUnit {
                                site: unit_index,
                                path_display: workspace.display_path(absolute_path),
                                site_line: *line,
                                digest: *digest,
                                kind: DataUnitKind::File,
                            });
                            self.follow_shell_sources(unit_index, absolute_path, *line);
                        }
                        (ReadOrigin::Included | ReadOrigin::Runtime, _, None) => {
                            self.refusals.insert(Refusal {
                                file: crate::units::RefusalFile::WorkspaceSourceFile(unit.file),
                                line: *line,
                                form: RefusalForm::DataFileUnreadable,
                                fragment: workspace.display_path(absolute_path),
                            });
                        }
                    }
                }
                SourceRead::BuildScriptOutput { line } => {
                    match workspace.target_of_crate(crate_index, TargetKind::BuildScript, None) {
                        Some(build_script) => {
                            self.requested_roots.insert(build_script);
                        }
                        None => {
                            self.refusals.insert(Refusal {
                                file: crate::units::RefusalFile::WorkspaceSourceFile(unit.file),
                                line: *line,
                                form: RefusalForm::BuildScriptMissing,
                                fragment: "env!(\"OUT_DIR\")".to_string(),
                            });
                        }
                    }
                }
                SourceRead::BuildScriptEnvironmentValue { .. } => {
                    // 构建脚本经 `cargo::rustc-env` 交的值：连到本 crate 构建脚本的 `main`；没有构建脚本的不连、不拒算
                    // （值来自编译时的进程环境，不在仓里的代码里）。弄坏开关 build-script-environment-edges：不连
                    if self
                        .switches
                        .is_intact(BrokenRule::BuildScriptEnvironmentEdges)
                    {
                        if let Some(build_script) =
                            workspace.target_of_crate(crate_index, TargetKind::BuildScript, None)
                        {
                            self.requested_roots.insert(build_script);
                        }
                    }
                }
                SourceRead::Directory {
                    absolute_path,
                    line,
                    digest,
                } => {
                    self.data_units.insert(DataUnit {
                        site: unit_index,
                        path_display: workspace.display_path(absolute_path),
                        site_line: *line,
                        digest: *digest,
                        kind: DataUnitKind::Directory,
                    });
                }
                // 子进程测试边在闭包收完一趟之后连（`pull_in_child_process_tests`）：参数写法要看闭包里的调用处
                SourceRead::ChildTestProcess { .. } => {}
                SourceRead::BinaryExecutable { binary_name, line } => {
                    match workspace.target_of_crate(
                        crate_index,
                        TargetKind::Binary,
                        Some(binary_name),
                    ) {
                        Some(binary) => {
                            self.requested_roots.insert(binary);
                        }
                        None => {
                            self.refusals.insert(Refusal {
                                file: crate::units::RefusalFile::WorkspaceSourceFile(unit.file),
                                line: *line,
                                form: RefusalForm::BinaryTargetMissing,
                                fragment: format!("CARGO_BIN_EXE_{binary_name}"),
                            });
                        }
                    }
                }
            }
        }
    }

    /// 闭包里 Rust 单元给 WGSL 外名的着落：串字面量里的词（整名要是其中一个），以及串字面量与 `macro_rules!` 体的原文
    /// （外名的首段加 `_` 要落在其中之一里）。
    fn wgsl_external_name_sources_of_rust_closure(&self) -> WgslExternalNameSources {
        let mut sources = WgslExternalNameSources {
            string_words: HashSet::new(),
            string_words_in_upper_case: HashSet::new(),
            texts_holding_prefixes: Vec::new(),
            defined_names_and_their_suffixes: HashSet::new(),
            names_in_upper_case: HashSet::new(),
        };
        for unit_index in &self.in_closure {
            let unit = &self.workspace.units[unit_index.0];
            if defines_names_a_kernel_can_use(unit.kind) {
                for declared in &unit.declared_names {
                    sources
                        .defined_names_and_their_suffixes
                        .extend(name_and_its_underscore_suffixes(declared).map(str::to_string));
                }
            }
            sources.names_in_upper_case.extend(
                unit.declared_names
                    .iter()
                    .chain(&unit.references.plain_names)
                    .map(|name| name.to_ascii_uppercase()),
            );
            let Some((first, end)) = unit.token_range else {
                continue;
            };
            let file = &self.workspace.files[unit.file.0];
            let is_macro_body = unit.kind == UnitKind::MacroRulesDefinition;
            for token in &file.tokens[first..end] {
                let text = token_text(&file.source, token);
                let is_string_literal = token.kind == TokenKind::Literal
                    && (text.starts_with('"') || text.starts_with('r'));
                if is_string_literal {
                    add_string_words(text, &mut sources.string_words);
                }
                if is_string_literal || is_macro_body {
                    sources.texts_holding_prefixes.push(text.to_string());
                }
            }
        }
        sources.string_words_in_upper_case = sources
            .string_words
            .iter()
            .map(|word| word.to_ascii_uppercase())
            .collect();
        sources
    }

    /// 闭包读到的 `.wgsl` 组成的模块：从 `@compute` 入口收到的 WGSL 单元与用到而模块里没定义的外名。
    fn wgsl_module_and_external_names(
        &self,
    ) -> (BTreeSet<WgslUnitIndex>, Vec<(WgslUnitIndex, String)>) {
        let module_units: Vec<WgslUnitIndex> = self
            .workspace
            .wgsl_units
            .iter()
            .enumerate()
            .filter(|(_, wgsl_unit)| self.wgsl_files.contains(&wgsl_unit.file))
            .map(|(position, _)| WgslUnitIndex(position))
            .collect();
        wgsl_module_closure(&self.workspace.wgsl_units, &module_units)
    }

    /// 链接集里、闭包外，供这几个外名的单元：按名字定义了外名或以 `_` 加外名收尾的常量、静态量、条目级宏调用，串字面量里有外名这个词的单元。
    fn wgsl_name_suppliers_outside_the_closure(
        &self,
        unsupplied_names: &BTreeSet<String>,
    ) -> Vec<UnitIndex> {
        let workspace = self.workspace;
        let mut suppliers = Vec::new();
        for (position, unit) in workspace.units.iter().enumerate() {
            let unit_index = UnitIndex(position);
            if self.in_closure.contains(&unit_index)
                || self.wgsl_name_suppliers_tried.contains(&unit_index)
                || self.linked_target_of(unit_index).is_none()
                || self.is_test_only_code(unit_index)
            {
                continue;
            }
            let defines_a_name = defines_names_a_kernel_can_use(unit.kind)
                && unit.declared_names.iter().any(|declared| {
                    name_and_its_underscore_suffixes(declared)
                        .any(|suffix| unsupplied_names.contains(suffix))
                });
            let names_one_in_a_string = !defines_a_name
                && unit.token_range.is_some_and(|(first, end)| {
                    let file = &workspace.files[unit.file.0];
                    file.tokens[first..end].iter().any(|token| {
                        let text = token_text(&file.source, token);
                        let is_string_literal = token.kind == TokenKind::Literal
                            && (text.starts_with('"') || text.starts_with('r'));
                        if !is_string_literal {
                            return false;
                        }
                        let mut words = HashSet::new();
                        add_string_words(text, &mut words);
                        words.iter().any(|word| unsupplied_names.contains(word))
                    })
                });
            if defines_a_name || names_one_in_a_string {
                suppliers.push(unit_index);
            }
        }
        suppliers
    }

    /// 链接集里、闭包外，`include_str!` 读进一份闭包还没读到、又定义了这几个名字之一的 `.wgsl` 的 Rust 单元（WGSL 跨文件着落边）。
    /// Rust 侧把几份 `.wgsl` 拼成一份着色器时，一份里用到的名字定义在另一份里；只读到前一份的闭包把读后一份的单元连同它读的那份
    /// `.wgsl` 并进来，名字在合起来的名字空间里解。只看那份 `.wgsl` 有没有定义这个名字，不核读它的单元真被拿去拼了内核（超集）。
    /// 交（单元，它在链接集里的那个目标）；链接集外的单元不交，它读的 `.wgsl` 不给这个入口着落。
    fn wgsl_includers_outside_the_closure(
        &self,
        names_the_wgsl_module_leaves_undefined: &BTreeSet<String>,
    ) -> Vec<(UnitIndex, TargetIndex)> {
        let workspace = self.workspace;
        let defining_files: BTreeSet<FileIndex> = workspace
            .wgsl_units
            .iter()
            .filter(|wgsl_unit| {
                !self.wgsl_files.contains(&wgsl_unit.file)
                    && names_the_wgsl_module_leaves_undefined.contains(&wgsl_unit.name)
            })
            .map(|wgsl_unit| wgsl_unit.file)
            .collect();
        if defining_files.is_empty() {
            return Vec::new();
        }
        let mut includers = Vec::new();
        for (position, unit) in workspace.units.iter().enumerate() {
            let unit_index = UnitIndex(position);
            if self.in_closure.contains(&unit_index)
                || self.wgsl_name_suppliers_tried.contains(&unit_index)
            {
                continue;
            }
            let Some(linked_target) = self.linked_target_of(unit_index) else {
                continue;
            };
            let includes_a_defining_file = unit.reads.iter().any(|read| match read {
                SourceRead::File {
                    absolute_path,
                    origin: ReadOrigin::Included,
                    ..
                } => workspace
                    .indexes
                    .files_by_path
                    .get(absolute_path)
                    .is_some_and(|file| defining_files.contains(file)),
                SourceRead::File {
                    origin: ReadOrigin::Runtime,
                    ..
                }
                | SourceRead::BuildScriptOutput { .. }
                | SourceRead::BuildScriptEnvironmentValue { .. }
                | SourceRead::BinaryExecutable { .. }
                | SourceRead::Directory { .. }
                | SourceRead::ChildTestProcess { .. } => false,
            });
            if includes_a_defining_file {
                includers.push((unit_index, linked_target));
            }
        }
        includers
    }

    /// 着落做成边：闭包读到的 `.wgsl` 里没定义的名字，先把链接集里 `include_str!` 读进定义它的另一份 `.wgsl` 的单元并进闭包
    /// （`wgsl_includers_outside_the_closure`）；哪份 `.wgsl` 都没定义、Rust 闭包里也还没有着落的，再把链接集里供它的单元并进闭包
    /// （连同它们的闭包，由调用方再 `drain`）。交回这一趟有没有新并进来的单元。
    fn pull_in_wgsl_name_suppliers(&mut self) -> bool {
        if self.wgsl_files.is_empty()
            || !self.switches.is_intact(BrokenRule::WgslExternalNames)
            || !self.switches.is_intact(BrokenRule::WgslSupplierEdges)
        {
            return false;
        }
        let (_reached, external_names) = self.wgsl_module_and_external_names();
        let names_the_wgsl_module_leaves_undefined: BTreeSet<String> = external_names
            .iter()
            .map(|(_, name)| name.clone())
            .collect();
        let includers =
            self.wgsl_includers_outside_the_closure(&names_the_wgsl_module_leaves_undefined);
        if !includers.is_empty() {
            // 先只并读 `.wgsl` 的单元：下一趟在合起来的模块上重算外名，定义在那几份里的名字不再去找 Rust 侧的着落
            for (includer, linked_target) in &includers {
                self.wgsl_name_suppliers_tried.insert(*includer);
                self.push(*includer, *linked_target);
            }
            return true;
        }
        let sources = self.wgsl_external_name_sources_of_rust_closure();
        let accepts_prefixes = self
            .switches
            .is_intact(BrokenRule::WgslExternalNamePrefixes);
        let unsupplied_names: BTreeSet<String> = external_names
            .into_iter()
            .map(|(_, name)| name)
            .filter(|name| !sources.supply(name, accepts_prefixes))
            .collect();
        if unsupplied_names.is_empty() {
            return false;
        }
        let suppliers = self.wgsl_name_suppliers_outside_the_closure(&unsupplied_names);
        for supplier in &suppliers {
            self.wgsl_name_suppliers_tried.insert(*supplier);
            self.push_with_a_linked_target(*supplier);
        }
        !suppliers.is_empty()
    }

    /// 只在测试里编进来的代码：带 `#[test]` 的函数，外层条件链里有 `test` 的单元（`#[cfg(test)]` 模块里的），
    /// 或它所在的文件在链接集里的那个目标下每一种挂法都带 `test`（`#[cfg(test)] mod 名字;` 引进来的文件）。
    fn is_test_only_code(&self, unit_index: UnitIndex) -> bool {
        let unit = &self.workspace.units[unit_index.0];
        if unit.has_test_attribute || unit.is_inside_test_only_code {
            return true;
        }
        self.linked_target_of(unit_index)
            .and_then(|linked_target| self.workspace.membership(unit.file, linked_target))
            .is_some_and(|membership| {
                !membership.condition_alternatives.is_empty()
                    && membership.condition_alternatives.iter().all(|alternative| {
                        alternative
                            .iter()
                            .any(crate::rust_items::predicate_requires_the_test_configuration)
                    })
            })
    }

    /// 单元自己的词法单元原文。
    fn token_texts_of(&self, unit_index: UnitIndex) -> Vec<&'workspace str> {
        let workspace = self.workspace;
        let unit = &workspace.units[unit_index.0];
        let Some((first, end)) = unit.token_range else {
            return Vec::new();
        };
        let file = &workspace.files[unit.file.0];
        file.tokens[first..end]
            .iter()
            .map(|token| token_text(&file.source, token))
            .collect()
    }

    /// 常量 `name` 的串值：先找写在 `context` 这个单元里的 `const name: … = "…";`，再找同一个包、在这一次链接集里（同一包的别的 bin 不算）
    /// 叫这个名字的常量单元（几个都得写成 `= "…";`、值相同）。找不到、写法不是串字面量或几个值不同交 `None`。
    fn constant_string_value(&self, context: UnitIndex, name: &str) -> Option<String> {
        let definition_value = |texts: &[&str]| -> Option<String> {
            let const_index = (0..texts.len()).find(|index| {
                texts[*index] == "const"
                    && texts.get(index + 1).copied() == Some(name)
                    && texts.get(index + 2).copied() == Some(":")
            })?;
            let equals = (const_index + 3..texts.len()).find(|index| texts[*index] == "=")?;
            match texts.get(equals + 1..equals + 3)? {
                [literal, ";"] => plain_string_content(literal).map(str::to_string),
                _ => None,
            }
        };
        if let Some(value) = definition_value(&self.token_texts_of(context)) {
            return Some(value);
        }
        let workspace = self.workspace;
        let crate_index = workspace.files[workspace.units[context.0].file.0].crate_index;
        let mut values = BTreeSet::new();
        for (position, unit) in workspace.units.iter().enumerate() {
            if unit.kind == UnitKind::Constant
                && unit.name == name
                && workspace.files[unit.file.0].crate_index == crate_index
                && self.linked_target_of(UnitIndex(position)).is_some()
            {
                values.insert(definition_value(&self.token_texts_of(UnitIndex(position)))?);
            }
        }
        match values.len() {
            1 => values.into_iter().next(),
            _ => None,
        }
    }

    /// 几段连成测试名：常量在 `context` 这个单元（与它的 crate）里解。
    fn folded_test_name(&self, context: UnitIndex, pieces: &[TestNamePiece]) -> Option<String> {
        let mut name = String::new();
        for piece in pieces {
            match piece {
                TestNamePiece::Literal(text) => name.push_str(text),
                TestNamePiece::Constant(constant) => {
                    name.push_str(&self.constant_string_value(context, constant)?);
                }
            }
        }
        Some(name)
    }

    /// 入口目标里 libtest 名字（模块路径 `::` 函数名，不带 crate 名）恰好是 `test_name` 的那一个 `#[test] fn`。
    fn test_function_named(&self, test_name: &str) -> Option<UnitIndex> {
        let workspace = self.workspace;
        let segments: Vec<&str> = test_name.split("::").collect();
        let (function_name, module_path) = segments.split_last()?;
        let mut found = Vec::new();
        for file in &workspace.files_of_target[self.entry_target.0] {
            for unit_index in units_of_file(workspace, *file) {
                let unit = &workspace.units[unit_index.0];
                if unit.kind != UnitKind::Function
                    || !unit.has_test_attribute
                    || unit.container.is_some()
                    || unit.name != *function_name
                {
                    continue;
                }
                let matches_the_module = unit_module_path(workspace, unit_index, self.entry_target)
                    .is_some_and(|path| {
                        path.iter()
                            .map(String::as_str)
                            .eq(module_path.iter().copied())
                    });
                if matches_the_module {
                    found.push(unit_index);
                }
            }
        }
        match found.as_slice() {
            [only] => Some(*only),
            _ => None,
        }
    }

    /// 起子进程的单元 `spawning` 里那条 `--exact` 点名的测试：解出来的每一个 `#[test] fn`（参数写法时闭包里每一处调用各解一个）。
    /// 有一处折不出、解不到恰好一个，或参数写法在闭包里一处调用都没有，交 `None`（拒算 `current-exe`）。
    fn child_tests_of(
        &self,
        spawning: UnitIndex,
        test_name: &ChildTestName,
    ) -> Option<Vec<UnitIndex>> {
        match test_name {
            ChildTestName::Pieces(pieces) => {
                let name = self.folded_test_name(spawning, pieces)?;
                Some(vec![self.test_function_named(&name)?])
            }
            ChildTestName::ParameterOfTheSpawningFunction {
                function_name,
                parameter_position,
            } => {
                let mut tests = Vec::new();
                for caller in &self.in_closure {
                    let texts = self.token_texts_of(*caller);
                    for (index, text) in texts.iter().enumerate() {
                        let is_call = *text == function_name
                            && texts.get(index + 1).copied() == Some("(")
                            && !matches!(
                                index.checked_sub(1).map(|previous| texts[previous]),
                                Some("fn" | ".")
                            );
                        if !is_call {
                            continue;
                        }
                        let mut depth = 0usize;
                        let close = (index + 1..texts.len()).find(|position| {
                            match texts[*position] {
                                "(" | "[" | "{" => depth += 1,
                                ")" | "]" | "}" => depth = depth.saturating_sub(1),
                                _ => {}
                            }
                            depth == 0
                        })?;
                        let arguments = split_at_top_level_commas(&texts[index + 2..close]);
                        let pieces =
                            test_name_pieces_of_texts(arguments.get(*parameter_position)?)?;
                        let name = self.folded_test_name(*caller, &pieces)?;
                        tests.push(self.test_function_named(&name)?);
                    }
                }
                (!tests.is_empty()).then_some(tests)
            }
        }
    }

    /// 子进程测试边（`current-exe-child`）：闭包里起子进程的单元点名的测试并进闭包（在入口目标里收）。交回这一趟有没有新推进去的单元。
    fn pull_in_child_process_tests(&mut self) -> bool {
        let workspace = self.workspace;
        let mut children = Vec::new();
        for unit_index in &self.in_closure {
            for read in &workspace.units[unit_index.0].reads {
                if let SourceRead::ChildTestProcess { test_name, .. } = read {
                    if let Some(tests) = self.child_tests_of(*unit_index, test_name) {
                        children.extend(tests);
                    }
                }
            }
        }
        let mut pushed = false;
        for child in children {
            if !self.seen_pairs.contains(&(child, self.entry_target)) {
                self.push(child, self.entry_target);
                pushed = true;
            }
        }
        pushed
    }

    fn finish(mut self, entry_target: TargetIndex, entry_units: Vec<UnitIndex>) -> ClosureOutcome {
        let workspace = self.workspace;
        for unit_index in &self.in_closure {
            let unit = &workspace.units[unit_index.0];
            for read in &unit.reads {
                if let SourceRead::ChildTestProcess {
                    line,
                    test_name,
                    fragment,
                } = read
                {
                    if self.child_tests_of(*unit_index, test_name).is_none() {
                        self.refusals.insert(Refusal {
                            file: crate::units::RefusalFile::WorkspaceSourceFile(unit.file),
                            line: *line,
                            form: RefusalForm::CurrentExecutablePath,
                            fragment: fragment.clone(),
                        });
                    }
                }
            }
        }
        for target in &self.link.targets {
            for file in &workspace.files_of_target[target.0] {
                for refusal in &workspace.files[file.0].file_level_refusals {
                    self.refusals.insert(refusal.clone());
                }
            }
        }
        for crate_index in &self.link.included_crates {
            if workspace.crates[crate_index.0].manifest.is_procedure_macro {
                if let Some(root) = workspace
                    .target_of_crate(*crate_index, TargetKind::Library, None)
                    .and_then(|library| workspace.targets[library.0].root_file)
                {
                    self.refusals.insert(Refusal {
                        file: crate::units::RefusalFile::WorkspaceSourceFile(root),
                        line: 1,
                        form: RefusalForm::ProcedureMacroCrate,
                        fragment: format!(
                            "{} 是 proc-macro crate",
                            workspace.crates[crate_index.0].package_name
                        ),
                    });
                }
            }
        }
        let mut wgsl_units = Vec::new();
        if !self.wgsl_files.is_empty() {
            for file in &self.wgsl_files {
                for refusal in &workspace.files[file.0].file_level_refusals {
                    self.refusals.insert(refusal.clone());
                }
            }
            let (reached, external_names) = self.wgsl_module_and_external_names();
            if self.switches.is_intact(BrokenRule::WgslExternalNames) && !external_names.is_empty()
            {
                let sources = self.wgsl_external_name_sources_of_rust_closure();
                let accepts_prefixes = self
                    .switches
                    .is_intact(BrokenRule::WgslExternalNamePrefixes);
                for (wgsl_unit_index, name) in external_names {
                    if !sources.supply(&name, accepts_prefixes) {
                        let wgsl_unit = &workspace.wgsl_units[wgsl_unit_index.0];
                        self.refusals.insert(Refusal {
                            file: crate::units::RefusalFile::WorkspaceSourceFile(wgsl_unit.file),
                            line: wgsl_unit.start_line,
                            form: RefusalForm::WgslUndefinedName,
                            fragment: name,
                        });
                    }
                }
            }
            wgsl_units = reached.into_iter().collect();
        }
        ClosureOutcome {
            entry_target_kind: workspace.targets[entry_target.0].kind,
            entry_units,
            condition_atoms_with_their_values: self.condition_atoms.into_iter().collect(),
            units: self.in_closure.into_iter().collect(),
            wgsl_units,
            data_units: self.data_units.into_iter().collect(),
            refusals: self.refusals.into_iter().collect(),
        }
    }
}

fn entry_conditions_of(
    request: &ClosureRequest,
    target_kind: TargetKind,
    features_enabled_on_the_dependencies_by_the_build: BTreeMap<CrateIndex, BTreeSet<String>>,
) -> EntryConditions {
    let mut conditions = EntryConditions {
        features: BTreeSet::new(),
        is_test: target_kind.is_compiled_as_test(),
        extra_names: BTreeSet::new(),
        extra_key_values: BTreeSet::new(),
        uses_default_features: request.uses_default_features,
        features_enabled_on_the_dependencies_by_the_build,
    };
    for requested in &request.requested_conditions {
        match requested {
            RequestedCondition::Name(name) if name == "test" => conditions.is_test = true,
            RequestedCondition::Name(name) => {
                conditions.extra_names.insert(name.clone());
            }
            RequestedCondition::Feature(feature) => {
                conditions.features.insert(feature.clone());
            }
            RequestedCondition::KeyValue { key, value } => {
                conditions
                    .extra_key_values
                    .insert((key.clone(), value.clone()));
            }
        }
    }
    conditions
}

fn resolve_entry_target(
    workspace: &Workspace,
    request: &ClosureRequest,
) -> Result<TargetIndex, EntryError> {
    let segments: Vec<&str> = request.entry_path.split("::").collect();
    let Some(crate_name) = segments.first().filter(|_| segments.len() >= 2) else {
        return Err(EntryError::MalformedEntryPath(request.entry_path.clone()));
    };
    let crate_index = workspace
        .crate_named(crate_name)
        .ok_or_else(|| EntryError::CrateNotFound((*crate_name).to_string()))?;
    let (kind, name) = match &request.target {
        TargetSelector::Library => (TargetKind::Library, None),
        TargetSelector::Binary(name) => (TargetKind::Binary, Some(name.as_str())),
        TargetSelector::IntegrationTest(name) => (TargetKind::IntegrationTest, Some(name.as_str())),
        TargetSelector::Example(name) => (TargetKind::Example, Some(name.as_str())),
        TargetSelector::Benchmark(name) => (TargetKind::Benchmark, Some(name.as_str())),
        TargetSelector::BuildScript => (TargetKind::BuildScript, None),
        TargetSelector::DocumentationTest => (TargetKind::DocumentationTest, None),
    };
    workspace
        .target_of_crate(crate_index, kind, name)
        .ok_or_else(|| EntryError::TargetNotFound {
            crate_name: (*crate_name).to_string(),
            target: format!(
                "{}{}",
                kind.key_name(),
                name.map(|target_name| format!(":{target_name}"))
                    .unwrap_or_default()
            ),
        })
}

/// 一份文件的单元（工作区里的单元按文件次序排着，同一份文件的单元连在一起）。
fn units_of_file(
    workspace: &Workspace,
    file: crate::units::FileIndex,
) -> impl Iterator<Item = UnitIndex> {
    let start = workspace.units.partition_point(|unit| unit.file < file);
    let end = workspace.units.partition_point(|unit| unit.file <= file);
    (start..end).map(UnitIndex)
}

fn unit_module_path(
    workspace: &Workspace,
    unit_index: UnitIndex,
    target: TargetIndex,
) -> Option<Vec<String>> {
    let unit = &workspace.units[unit_index.0];
    let membership = workspace.membership(unit.file, target)?;
    let mut path = membership.module_path.clone();
    path.extend(unit.local_module_path.iter().cloned());
    Some(path)
}

fn matches_entry_path(
    workspace: &Workspace,
    unit_index: UnitIndex,
    target: TargetIndex,
    path: &[&str],
) -> bool {
    let unit = &workspace.units[unit_index.0];
    let Some((last, module_part)) = path.split_last() else {
        return false;
    };
    if unit.name != *last {
        return false;
    }
    let Some(module_path) = unit_module_path(workspace, unit_index, target) else {
        return false;
    };
    match unit.container {
        None => module_path
            .iter()
            .map(String::as_str)
            .eq(module_part.iter().copied()),
        Some(container) => {
            let Some((type_name, outer)) = module_part.split_last() else {
                return false;
            };
            let container_unit = &workspace.units[container.0];
            let names_the_type = container_unit.name == *type_name
                || container_unit.implementation.as_ref().is_some_and(|shape| {
                    shape.self_type_names.iter().any(|name| name == type_name)
                });
            names_the_type
                && module_path
                    .iter()
                    .map(String::as_str)
                    .eq(outer.iter().copied())
        }
    }
}

fn resolve_entry_units(
    workspace: &Workspace,
    entry_path: &str,
    entry_target: TargetIndex,
    link: &Link,
) -> Result<Vec<UnitIndex>, EntryError> {
    let segments: Vec<&str> = entry_path.split("::").collect();
    let path = &segments[1..];
    let target = &workspace.targets[entry_target.0];
    if target.kind == TargetKind::DocumentationTest {
        let library = workspace
            .target_of_crate(target.crate_index, TargetKind::Library, None)
            .ok_or_else(|| EntryError::EntryNotFound(entry_path.to_string()))?;
        let mut tests = Vec::new();
        for file in &workspace.files_of_target[library.0] {
            if let Some(membership) = workspace.membership(*file, library) {
                for ((module_file, local_path), module_tests) in
                    &workspace.indexes.documentation_tests_of_module
                {
                    if module_file == file {
                        let mut full = membership.module_path.clone();
                        full.extend(local_path.iter().cloned());
                        if full.iter().map(String::as_str).eq(path.iter().copied()) {
                            tests.extend(module_tests.iter().copied());
                        }
                    }
                }
            }
        }
        for (owner, owner_tests) in &workspace.indexes.documentation_tests_of_item {
            if matches_entry_path(workspace, *owner, library, path) {
                tests.extend(owner_tests.iter().copied());
            }
        }
        tests.sort();
        tests.dedup();
        if tests.is_empty() {
            return Err(EntryError::NoDocumentationTests(entry_path.to_string()));
        }
        return Ok(tests);
    }
    let candidates: Vec<UnitIndex> = workspace.files_of_target[entry_target.0]
        .iter()
        .flat_map(|file| units_of_file(workspace, *file))
        .filter(|unit_index| {
            workspace.units[unit_index.0].kind == UnitKind::Function
                && matches_entry_path(workspace, *unit_index, entry_target, path)
        })
        .collect();
    if candidates.is_empty() {
        return Err(EntryError::EntryNotFound(entry_path.to_string()));
    }
    let set = link.condition_set(entry_target);
    let active: Vec<UnitIndex> = candidates
        .iter()
        .copied()
        .filter(
            |candidate| match unit_activity(workspace, &set, *candidate, entry_target) {
                Activity::Inactive => false,
                Activity::Active | Activity::ActiveWithUnknownCondition(_) => true,
            },
        )
        .collect();
    match active.len() {
        0 => Err(EntryError::EntryInactive(entry_path.to_string())),
        1 => Ok(active),
        _ => Err(EntryError::EntryAmbiguous {
            path: entry_path.to_string(),
            candidates: active
                .iter()
                .map(|candidate| workspace.units[candidate.0].qualified_name.clone())
                .collect(),
        }),
    }
}

/// 从入口收闭包。
///
/// # Errors
/// 入口找不到、不唯一或在这组 cfg 下不编进来（用法错，不是拒算；拒算在 `ClosureOutcome::refusals` 里）；闭包读到的 shell 脚本交 bash
/// 规范化跑不下去（[`EntryError::ShellNormalizationFailed`]，运行条件不满足，整趟报错）。
pub fn compute_closure(
    workspace: &Workspace,
    request: &ClosureRequest,
    switches: &RuleSwitches,
) -> Result<ClosureOutcome, EntryError> {
    compute_closure_of_entries(workspace, request, &[], switches)
}

/// 从一组入口一起收闭包：`request.entry_path` 与 `additional_entry_paths`，都在 `request` 点名的那个目标里（同一个 crate），
/// 按同一次收法从全部入口出发（R-方法 见过的方法名、点过名的类型几个入口共用），不是各收各的再并。
/// 依赖包的特性只按清单解（没有别的包在同一次构建里并上来的特性）：同 [`compute_closure_of_entries_with_the_features_of_the_build`] 交空表。
///
/// # Errors
/// 同 [`compute_closure`]；另给的入口不在头一个入口的 crate 里。
pub fn compute_closure_of_entries(
    workspace: &Workspace,
    request: &ClosureRequest,
    additional_entry_paths: &[String],
    switches: &RuleSwitches,
) -> Result<ClosureOutcome, EntryError> {
    compute_closure_of_entries_with_the_features_of_the_build(
        workspace,
        request,
        additional_entry_paths,
        &FeaturesEnabledByTheBuild::default(),
        switches,
    )
}

/// 同 [`compute_closure_of_entries`]，另按这一次编译里 cargo 给各依赖包实际打开的特性（`build_features`，连别的包在同一次构建里
/// 并上来的）解 cfg：那些包一被链进来，就照请求它们那几个特性，闭包与原子真值是这一次编出来的代码的样子。表里没有的包按清单解；
/// 表是空的，与 [`compute_closure_of_entries`] 逐字相同。
///
/// # Errors
/// 同 [`compute_closure_of_entries`]；表里点名的包工作区里没有（[`EntryError::CrateNotFound`]）。
pub fn compute_closure_of_entries_with_the_features_of_the_build(
    workspace: &Workspace,
    request: &ClosureRequest,
    additional_entry_paths: &[String],
    features_enabled_by_the_build: &FeaturesEnabledByTheBuild,
    switches: &RuleSwitches,
) -> Result<ClosureOutcome, EntryError> {
    let mut features_enabled_on_the_dependencies_by_the_build = BTreeMap::new();
    for (package_name, features) in &features_enabled_by_the_build.features_of_each_package {
        let crate_index = workspace
            .crate_named(package_name)
            .ok_or_else(|| EntryError::CrateNotFound(package_name.clone()))?;
        features_enabled_on_the_dependencies_by_the_build.insert(crate_index, features.clone());
    }
    let entry_target = resolve_entry_target(workspace, request)?;
    let crate_name = request
        .entry_path
        .split("::")
        .next()
        .unwrap_or_default()
        .to_string();
    for additional in additional_entry_paths {
        let additional_crate = additional.split("::").next().unwrap_or_default();
        if workspace.crate_named(additional_crate) != workspace.crate_named(&crate_name)
            || !additional.contains("::")
        {
            return Err(EntryError::EntryInAnotherCrate {
                path: additional.clone(),
                crate_name,
            });
        }
    }
    let entry_conditions = entry_conditions_of(
        request,
        workspace.targets[entry_target.0].kind,
        features_enabled_on_the_dependencies_by_the_build,
    );
    let platform = target_platform_of_triple(&request.target_triple);
    let mut extra_roots: BTreeSet<TargetIndex> = BTreeSet::new();
    loop {
        let link = build_link(
            workspace,
            entry_target,
            &entry_conditions,
            platform,
            &extra_roots,
            switches,
        );
        let mut entries = resolve_entry_units(workspace, &request.entry_path, entry_target, &link)?;
        for additional in additional_entry_paths {
            entries.extend(resolve_entry_units(
                workspace,
                additional,
                entry_target,
                &link,
            )?);
        }
        let mut run = ClosureRun::new(workspace, link, switches, entry_target);
        for entry in &entries {
            run.push(*entry, entry_target);
            let signature_traits = workspace.units[entry.0]
                .references
                .signature_trait_names
                .clone();
            for trait_name in &signature_traits {
                run.open_trait(trait_name);
            }
        }
        for root in &extra_roots {
            if let Some(main) = workspace.main_function_of(*root) {
                run.push(main, *root);
            }
        }
        run.push_global_allocators_of_the_linked_targets();
        run.drain();
        while run.pull_in_child_process_tests() || run.pull_in_wgsl_name_suppliers() {
            run.drain();
        }
        if let Some(failure) = run.shell_normalization_failure.take() {
            return Err(EntryError::ShellNormalizationFailed(failure));
        }
        if run.requested_roots.is_subset(&extra_roots) {
            let mut outcome = run.finish(entry_target, entries);
            if platform.is_none() {
                let entry_unit = &workspace.units[outcome
                    .entry_units
                    .first()
                    .expect("resolve_entry_units 只在找到至少一个入口单元时交 Ok")
                    .0];
                outcome.refusals.push(Refusal {
                    file: crate::units::RefusalFile::WorkspaceSourceFile(entry_unit.file),
                    line: entry_unit.start_line,
                    form: RefusalForm::TargetTripleUnrecognized,
                    fragment: request.target_triple.clone(),
                });
                outcome.refusals.sort();
            }
            return Ok(outcome);
        }
        let requested: Vec<TargetIndex> = run.requested_roots.iter().copied().collect();
        extra_roots.extend(requested);
    }
}

/// 复用键（设计 3.1）：入口目标的种类与闭包里各单元真引到的条件原子各带真值（闭包没引到的原子不进键）+ 闭包里每个单元（种类，指纹）的多重集 + 数据单元摘要的多重集
/// 加上入口单元自己（`entry_key_lines`）与调用方给的工具链、依赖解析行。
/// 不带路径、名字、行号、手写版本号。
///
/// # Errors
/// 闭包里有拒算：不交键，交那几条拒算。
pub fn reuse_key(
    workspace: &Workspace,
    outcome: &ClosureOutcome,
    external: &ExternalKeyInputs,
) -> Result<Sha256Digest, Vec<Refusal>> {
    if !outcome.refusals.is_empty() {
        return Err(outcome.refusals.clone());
    }
    let mut lines = vec![format!(
        "entry-target-kind\t{}",
        outcome.entry_target_kind.key_name()
    )];
    lines.extend(
        outcome
            .condition_atoms_with_their_values
            .iter()
            .map(|atom_with_its_value| format!("cfg\t{atom_with_its_value}")),
    );
    let mut unit_lines: Vec<String> = outcome
        .units
        .iter()
        .map(|unit_index| {
            let unit = &workspace.units[unit_index.0];
            format!(
                "unit\t{}\t{}",
                unit.kind.output_name(),
                unit.fingerprint.hexadecimal()
            )
        })
        .collect();
    unit_lines.extend(outcome.wgsl_units.iter().map(|wgsl_index| {
        let wgsl_unit = &workspace.wgsl_units[wgsl_index.0];
        format!(
            "unit\t{}\t{}",
            wgsl_unit.kind.output_name(),
            wgsl_unit.fingerprint.hexadecimal()
        )
    }));
    unit_lines.sort();
    lines.extend(unit_lines);
    let mut data_lines: Vec<String> = outcome
        .data_units
        .iter()
        .map(|data_unit| format!("data\t{}", data_unit.digest.hexadecimal()))
        .collect();
    data_lines.sort();
    lines.extend(data_lines);
    lines.extend(entry_key_lines(workspace, outcome));
    lines.extend(
        external
            .toolchain_lines
            .iter()
            .map(|line| format!("toolchain\t{line}")),
    );
    lines.extend(
        external
            .lock_lines
            .iter()
            .map(|line| format!("lock\t{line}")),
    );
    Ok(sha256_of(lines.join("\n").as_bytes()))
}

/// 键里的 entry 行：每个入口单元连同包它的 impl / trait 头（`Unit::container`）一层层往外，各取（种类，指纹），一个入口一行
/// （`entry\t<种类>\t<指纹>[\t<种类>\t<指纹>…]`），排序去重；不带名字、路径与行号。与 `scripts/gate/lib_function_units.py` 的
/// `entry_key_lines` 同一个口径。闭包是单元的多重集，几个入口互相收进对方的闭包（同一个 impl 里的几个方法、互相调用的两个函数）时
/// 闭包一模一样，没有这几行就共用一把键、一份判定。文档测试取到它自己就停：它挂的条目记在 `documentation_test_of`，
/// `container` 是空的，R-结构也不连它（文档测试另编一个 crate）。
fn entry_key_lines(workspace: &Workspace, outcome: &ClosureOutcome) -> Vec<String> {
    let mut entry_lines: BTreeSet<String> = BTreeSet::new();
    for entry in &outcome.entry_units {
        let mut chain_of_kinds_and_fingerprints: Vec<String> = Vec::new();
        let mut units_on_the_chain: BTreeSet<UnitIndex> = BTreeSet::new();
        let mut owner = Some(*entry);
        while let Some(owner_index) = owner {
            assert!(
                units_on_the_chain.insert(owner_index),
                "container 链不成环：成员指向 impl / trait 头，头自己没有 container（rust_items 切分时只这样填）"
            );
            let owner_unit = &workspace.units[owner_index.0];
            assert!(
                owner_unit.kind != UnitKind::DocumentationTest || owner_unit.container.is_none(),
                "文档测试单元的 container 是空的（所挂条目记在 documentation_test_of）：entry 行取到文档测试自己就停"
            );
            chain_of_kinds_and_fingerprints.push(format!(
                "{}\t{}",
                owner_unit.kind.output_name(),
                owner_unit.fingerprint.hexadecimal()
            ));
            owner = owner_unit.container;
        }
        entry_lines.insert(format!(
            "entry\t{}",
            chain_of_kinds_and_fingerprints.join("\t")
        ));
    }
    entry_lines.into_iter().collect()
}

/// 文档测试单元挂在哪个条目下（输出用）。
#[must_use]
pub fn documentation_test_owner(workspace: &Workspace, unit_index: UnitIndex) -> Option<UnitIndex> {
    match &workspace.units[unit_index.0].documentation_test_of {
        Some(DocumentationTestAttachment::Item(owner)) => Some(*owner),
        Some(DocumentationTestAttachment::Module(_)) | None => None,
    }
}
