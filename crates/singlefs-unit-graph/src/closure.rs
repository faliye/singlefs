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
//! use 引进的名字要 use 路径恰好点到候选所在的模块才算点准：路径点到的模块里没有这个名字的候选，名字是那个模块经 `pub use` 转出来的，落在哪一份看不出，退回全连；
//! 点到的候选带条件编译、或那个模块另有 use 按名字引进它（cfg 互斥的两支），相对路径的首段是本作用域里别的 use 引进的名字（含 `as` 改名），同样退回全连。
//! R-裸名局部绑定遮住同名函数（一个弄坏开关；跨目标的同样遮）：函数单元里，名字每一处都是局部绑定或落在绑定的作用域里的
//! （`is_bare_name_only_a_local_binding_of_the_function`），不往外连同名的函数。只遮函数：`let` 写成常量、静态量、单元结构体的名字是拿它当模式，
//! 不是新绑定（rustc 对静态量与元组结构体报 E0530），这几种照连。落在宏调用括号里的名字照连同名函数，只有标准库那 16 个就地求值实参的宏
//! （`STANDARD_MACROS_EVALUATING_THEIR_ARGUMENTS_IN_PLACE`：格式化、打印、写、`panic!`、断言一族与 `vec!`）的括号里照算局部。
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
//! 程序起自己（[`SourceRead::OwnProgramRestart`]，`finish` 判）：`current_exe()` 的路径原样交给 `Command::new`、没有折得出的 `--exact`。
//! 入口恰好是一个 bin 目标的 `fn main`、这一趟不按测试编的（`entry_is_the_main_of_a_program`）不拒算、不连边：子进程跑的还是这个 `main`，
//! 它闭包里的代码都已收齐，换参数只换走哪条路。入口是别的函数（`tests::…`、bin 里的辅助函数、模块里另一个叫 `main` 的）、是测试目标、
//! 或一组入口里另有别的入口的，拒算 `current-exe`：按测试编的程序里 `main` 是 libtest 的，参数选中哪条测试说不清。

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt;

use crate::build_features::FeaturesEnabledByTheBuild;
use crate::conditional_compilation::{
    atom_of_the_leaf, leaves_of_the_predicate, parse_predicate, plain_string_content,
    rendered_predicate, split_at_top_level_commas, target_platform_of_triple,
    CompilationConditions, ConditionalCompilationSet, Evaluation, Predicate, TargetPlatform,
};
use crate::lexer::{
    identifier_name, lex_source, token_text, DocumentationCommentTreatment, Token, TokenKind,
};
use crate::manifest::DependencySection;
use crate::rule_switches::{BrokenRule, RuleSwitches};
use crate::rust_items::{
    extern_block_attributes_name_link, normalized_path, references_of_tokens,
    test_name_pieces_of_texts,
};
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

mod bare_names_seen_from_the_module;
mod child_process_tests;
mod entry_resolution;
mod features_and_linked_targets;
mod local_bindings_of_a_function;
mod reads_and_shell_sources;
mod reuse_key_composition;
mod structure_methods_paths_and_traits;
mod units_kept_by_package;
mod wgsl_external_names;

use entry_resolution::*;
use features_and_linked_targets::*;
use local_bindings_of_a_function::*;
pub use reuse_key_composition::*;
pub use units_kept_by_package::*;

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

/// 闭包里一个单元头一回被排进待收队列的那条边：正在收哪个单元（入口与收尾几步并进来的没有）、照哪条规则、经哪个名字（规则不按名字连的为空）。
/// 只给人查「它是经哪条边进来的」用，不进键、不进组成清单；同一个单元可以经好几条边进来，记的只是头一条。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FirstEdgeIntoTheClosure {
    pub from: Option<UnitIndex>,
    pub rule: &'static str,
    pub name: String,
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
    /// 同一批条件原子按引到它的单元分开（一个单元经几个目标收进来的，几个目标下的真值都在）：`condition_atoms_with_their_values`
    /// 是它们的并集；只键几个包的单元时，条件成分只取留下的单元的那几份。
    pub condition_atoms_of_each_unit: BTreeMap<UnitIndex, BTreeSet<String>>,
    pub units: Vec<UnitIndex>,
    pub wgsl_units: Vec<WgslUnitIndex>,
    pub data_units: Vec<DataUnit>,
    pub refusals: Vec<Refusal>,
    /// 闭包里每个单元头一回被排进来的那条边（[`FirstEdgeIntoTheClosure`]）：只当索引，不进键。
    pub first_edges: BTreeMap<UnitIndex, FirstEdgeIntoTheClosure>,
    /// 调用方登记了「不沿它收」、这一趟又真够到了的单元（限定名，排好序）：它们不在 `units` 里，也没从它们往下收。
    pub units_reached_and_not_followed: Vec<String>,
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
    /// 闭包里各单元真引到的条件原子与真值，按引到它的单元分开（`record_condition_atoms`）。
    condition_atoms_of_each_unit: BTreeMap<UnitIndex, BTreeSet<String>>,
    /// 头一份 bash 规范化跑不下去的 shell 脚本（`follow_shell_sources`）：有了它这一趟交 `EntryError::ShellNormalizationFailed`，不交闭包。
    shell_normalization_failure: Option<ShellNormalizationFailure>,
    /// 每个被排进队列的单元头一回是经哪条边排进来的（`record_the_first_edge`）；不改收什么，只当索引。
    first_edges: BTreeMap<UnitIndex, FirstEdgeIntoTheClosure>,
    /// 此刻正在收的单元与正在走的规则：`push` 照它们记头一条边。
    unit_being_visited: Option<UnitIndex>,
    rule_being_followed: &'static str,
    /// 调用方登记的不沿它收的单元（限定名）：够到了不进闭包、不往下收，记进 `units_reached_and_not_followed`。
    units_not_followed: BTreeSet<String>,
    units_reached_and_not_followed: BTreeSet<String>,
}

/// R-方法：基本类型与 `Self`、`Vec`、`Option`、`Result`、`Box`、`String` 恒被点名。
const ALWAYS_NAMED_TYPES: [&str; 23] = [
    "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "f32",
    "f64", "bool", "char", "str", "String", "Vec", "Option", "Result", "Box", "Self",
];

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
            condition_atoms_of_each_unit: BTreeMap::new(),
            shell_normalization_failure: None,
            first_edges: BTreeMap::new(),
            unit_being_visited: None,
            rule_being_followed: "入口",
            units_not_followed: BTreeSet::new(),
            units_reached_and_not_followed: BTreeSet::new(),
        }
    }

    /// 记下 `unit` 头一回被排进来的那条边；已经记过的不改。
    fn record_the_first_edge(&mut self, unit: UnitIndex, name: &str) {
        if !self.first_edges.contains_key(&unit) {
            self.first_edges.insert(
                unit,
                FirstEdgeIntoTheClosure {
                    from: self.unit_being_visited,
                    rule: self.rule_being_followed,
                    name: name.to_string(),
                },
            );
        }
    }

    fn push(&mut self, unit: UnitIndex, target: TargetIndex) {
        self.record_the_first_edge(unit, "");
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
            self.unit_being_visited = Some(unit);
            self.visit_unit_for_target(unit, target);
        }
        self.unit_being_visited = None;
    }

    fn visit_unit_for_target(&mut self, unit_index: UnitIndex, target: TargetIndex) {
        let workspace = self.workspace;
        let unit = &workspace.units[unit_index.0];
        if !self.is_kind_kept(unit.kind) {
            return;
        }
        if self.units_not_followed.contains(&unit.qualified_name) {
            self.units_reached_and_not_followed
                .insert(unit.qualified_name.clone());
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
                self.rule_being_followed = "R-开放";
                for trait_name in &unit.references.dyn_trait_names {
                    self.open_trait(trait_name);
                }
            }
        }
        if !is_new {
            return;
        }
        if unit.kind.is_type() {
            self.rule_being_followed = "R-隐式（类型被点名）";
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
                self.rule_being_followed = "R-隐式（条目级宏调用生成的类型被点名）";
                for type_name in type_names {
                    self.name_type(type_name);
                }
            }
        }
        if unit.kind == UnitKind::TraitHead {
            self.rule_being_followed = "trait 头带它的 impl";
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
                self.condition_atoms_of_each_unit
                    .entry(unit_index)
                    .or_default()
                    .insert(value);
            }
        }
    }

    fn finish(
        mut self,
        entry_target: TargetIndex,
        entry_units: Vec<UnitIndex>,
        is_entry_compiled_as_test: bool,
    ) -> ClosureOutcome {
        let workspace = self.workspace;
        let entry_is_the_main_of_a_program = entry_is_the_main_of_a_program(
            workspace,
            entry_target,
            &entry_units,
            is_entry_compiled_as_test,
        );
        for unit_index in &self.in_closure {
            let unit = &workspace.units[unit_index.0];
            for read in &unit.reads {
                if let SourceRead::OwnProgramRestart { line, fragment } = read {
                    if !entry_is_the_main_of_a_program {
                        self.refusals.insert(Refusal {
                            file: crate::units::RefusalFile::WorkspaceSourceFile(unit.file),
                            line: *line,
                            form: RefusalForm::CurrentExecutablePath,
                            fragment: fragment.clone(),
                        });
                    }
                }
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
        let in_closure = &self.in_closure;
        let first_edges = self
            .first_edges
            .into_iter()
            .filter(|(unit, _)| in_closure.contains(unit))
            .collect();
        let condition_atoms_with_their_values: BTreeSet<String> = self
            .condition_atoms_of_each_unit
            .values()
            .flatten()
            .cloned()
            .collect();
        ClosureOutcome {
            entry_target_kind: workspace.targets[entry_target.0].kind,
            entry_units,
            condition_atoms_with_their_values: condition_atoms_with_their_values
                .into_iter()
                .collect(),
            condition_atoms_of_each_unit: self.condition_atoms_of_each_unit,
            units: self.in_closure.into_iter().collect(),
            wgsl_units,
            data_units: self.data_units.into_iter().collect(),
            refusals: self.refusals.into_iter().collect(),
            first_edges,
            units_reached_and_not_followed: self
                .units_reached_and_not_followed
                .into_iter()
                .collect(),
        }
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
    compute_closure_of_entries_not_following_the_listed_units(
        workspace,
        request,
        additional_entry_paths,
        features_enabled_by_the_build,
        switches,
        &BTreeSet::new(),
    )
}

/// 同 [`compute_closure_of_entries_with_the_features_of_the_build`]，另给一张「不沿它收」的单元登记（限定名）：够到登记的单元时
/// 它不进闭包、不从它往下收（经别的边够得到的单元照进），真够到的那几个交在 `ClosureOutcome::units_reached_and_not_followed`。
/// 登记是空的，与那一个函数逐字相同。登记什么、为什么不改判定结局由调用方担（登记的表与它的自证在调用方那一侧）。
///
/// # Errors
/// 同 [`compute_closure_of_entries_with_the_features_of_the_build`]。
pub fn compute_closure_of_entries_not_following_the_listed_units(
    workspace: &Workspace,
    request: &ClosureRequest,
    additional_entry_paths: &[String],
    features_enabled_by_the_build: &FeaturesEnabledByTheBuild,
    switches: &RuleSwitches,
    units_not_followed: &BTreeSet<String>,
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
        run.units_not_followed.clone_from(units_not_followed);
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
        run.rule_being_followed = "链进来的别的目标的 main";
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
            let mut outcome = run.finish(entry_target, entries, entry_conditions.is_test);
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

/// 文档测试单元挂在哪个条目下（输出用）。
#[must_use]
pub fn documentation_test_owner(workspace: &Workspace, unit_index: UnitIndex) -> Option<UnitIndex> {
    match &workspace.units[unit_index.0].documentation_test_of {
        Some(DocumentationTestAttachment::Item(owner)) => Some(*owner),
        Some(DocumentationTestAttachment::Module(_)) | None => None,
    }
}
