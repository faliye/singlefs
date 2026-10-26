//! 单元与拒算的数据形状：切分（`rust_items`、`wgsl`）产出它们，闭包（`closure`）与输出读它们。

use std::path::PathBuf;

use crate::conditional_compilation::Predicate;
use crate::sha256::Sha256Digest;

/// 工作区里一份源码文件（`.rs` 或 `.wgsl`）的下标。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileIndex(pub usize);

/// 工作区里一个 Rust 单元的下标。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnitIndex(pub usize);

/// 工作区里一个 WGSL 单元的下标。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WgslUnitIndex(pub usize);

/// Rust 单元的种类（设计 3.2 那张清单，外加 Verus 的 `broadcast group` 与模块头）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UnitKind {
    Function,
    ImplementationHead,
    TraitHead,
    Constant,
    Static,
    Struct,
    Enumeration,
    Union,
    TypeAlias,
    MacroRulesDefinition,
    Use,
    ItemMacroInvocation,
    ModuleDeclaration,
    /// 行内模块的 `mod 名 {` 连它的外属性与内属性，或一份文件开头的内属性 `#![…]`。
    ModuleHead,
    ExternBlock,
    ExternCrate,
    VerusBroadcastGroup,
    DocumentationTest,
}

impl UnitKind {
    /// 输出行与复用键里的种类名。与 `scripts/gate/lib_function_units.py` 的 Rust 那一份同一套写法。
    #[must_use]
    pub fn output_name(self) -> &'static str {
        match self {
            UnitKind::Function => "fn",
            UnitKind::ImplementationHead => "impl",
            UnitKind::TraitHead => "trait",
            UnitKind::Constant => "const",
            UnitKind::Static => "static",
            UnitKind::Struct => "struct",
            UnitKind::Enumeration => "enum",
            UnitKind::Union => "union",
            UnitKind::TypeAlias => "type",
            UnitKind::MacroRulesDefinition => "macro_rules",
            UnitKind::Use => "use",
            UnitKind::ItemMacroInvocation => "item_macro",
            UnitKind::ModuleDeclaration => "mod",
            UnitKind::ModuleHead => "mod_head",
            UnitKind::ExternBlock => "extern_block",
            UnitKind::ExternCrate => "extern_crate",
            UnitKind::VerusBroadcastGroup => "broadcast_group",
            UnitKind::DocumentationTest => "doctest",
        }
    }

    /// 是不是类型（被闭包点名之后，R-方法 与 R-隐式 按它的名字找 impl）。
    #[must_use]
    pub fn is_type(self) -> bool {
        match self {
            UnitKind::Struct | UnitKind::Enumeration | UnitKind::Union | UnitKind::TypeAlias => {
                true
            }
            UnitKind::Function
            | UnitKind::ImplementationHead
            | UnitKind::TraitHead
            | UnitKind::Constant
            | UnitKind::Static
            | UnitKind::MacroRulesDefinition
            | UnitKind::Use
            | UnitKind::ItemMacroInvocation
            | UnitKind::ModuleDeclaration
            | UnitKind::ModuleHead
            | UnitKind::ExternBlock
            | UnitKind::ExternCrate
            | UnitKind::VerusBroadcastGroup
            | UnitKind::DocumentationTest => false,
        }
    }

    /// 按裸名能不能找到它（R-裸名）：use、模块头、模块声明、外部 crate 声明与文档测试不按名字找。
    #[must_use]
    pub fn is_found_by_bare_name(self) -> bool {
        match self {
            UnitKind::Function
            | UnitKind::TraitHead
            | UnitKind::Constant
            | UnitKind::Static
            | UnitKind::Struct
            | UnitKind::Enumeration
            | UnitKind::Union
            | UnitKind::TypeAlias
            | UnitKind::MacroRulesDefinition
            | UnitKind::ItemMacroInvocation
            | UnitKind::ExternBlock
            | UnitKind::VerusBroadcastGroup => true,
            UnitKind::ImplementationHead
            | UnitKind::Use
            | UnitKind::ModuleDeclaration
            | UnitKind::ModuleHead
            | UnitKind::ExternCrate
            | UnitKind::DocumentationTest => false,
        }
    }

    /// 它里面的名字要不要往下解：use、模块头、模块声明与外部 crate 声明只是结构，里面的名字不往下收。
    #[must_use]
    pub fn resolves_its_own_names(self) -> bool {
        match self {
            UnitKind::Use
            | UnitKind::ModuleDeclaration
            | UnitKind::ModuleHead
            | UnitKind::ExternCrate => false,
            UnitKind::Function
            | UnitKind::ImplementationHead
            | UnitKind::TraitHead
            | UnitKind::Constant
            | UnitKind::Static
            | UnitKind::Struct
            | UnitKind::Enumeration
            | UnitKind::Union
            | UnitKind::TypeAlias
            | UnitKind::MacroRulesDefinition
            | UnitKind::ItemMacroInvocation
            | UnitKind::ExternBlock
            | UnitKind::VerusBroadcastGroup
            | UnitKind::DocumentationTest => true,
        }
    }
}

/// 拒算的形态（设计 3.4 的清单）。输出行里写 `output_name`。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RefusalForm {
    UnbalancedBrackets,
    UnrecognizedItemStart,
    UnrecognizedConditionalCompilationPredicate,
    IncludeArgumentNotInAllowedForms,
    /// 条目这一层的 `include!`：设计要把被含文件的单元并进来，这一版没做，显式拒算。
    ItemLevelInclude,
    RuntimeReadWithoutLiteralPath,
    DataFileUnreadable,
    IdentifierPastingMacro,
    ProcedureMacroCrate,
    CurrentExecutablePath,
    ExternalFunctionInterface,
    ModuleFileMissing,
    BuildScriptMissing,
    BinaryTargetMissing,
    /// WGSL 里用到的名字，链接集里哪份 `include_str!` 读进来的 `.wgsl` 都没定义，Rust 闭包里也没有着落。
    WgslUndefinedName,
    WgslUnrecognizedItemStart,
    /// 工具链给的目标三元组不在 `conditional_compilation::TARGET_PLATFORMS` 表里：平台谓词判不了。
    TargetTripleUnrecognized,
    /// 闭包点名的外部 crate 在 `Cargo.lock` 里找不到它解析到的那一个包。
    LockDependencyUnresolved,
    /// 产品行为判定的视图里，这份文件有一块 `verus!` 交给 `verus_syn` 解不开：规格与证明块剥不了，那份文件原样留着、链接到它的闭包拒算
    /// （`verus_ghost_code`）。形式化判定的视图不剥，不记这一条。
    VerusGhostCodeUnparsed,
}

impl RefusalForm {
    /// 拒算行第二段。与 `scripts/gate/lib_function_units.py` 同一套写法。
    #[must_use]
    pub fn output_name(self) -> &'static str {
        match self {
            RefusalForm::UnbalancedBrackets => "unbalanced-brackets",
            RefusalForm::UnrecognizedItemStart => "unrecognized-item-start",
            RefusalForm::UnrecognizedConditionalCompilationPredicate => {
                "unrecognized-cfg-predicate"
            }
            RefusalForm::IncludeArgumentNotInAllowedForms => "include-argument-form",
            RefusalForm::ItemLevelInclude => "item-level-include",
            RefusalForm::RuntimeReadWithoutLiteralPath => "runtime-read-path-not-literal",
            RefusalForm::DataFileUnreadable => "data-file-unreadable",
            RefusalForm::IdentifierPastingMacro => "identifier-pasting-macro",
            RefusalForm::ProcedureMacroCrate => "proc-macro-crate",
            RefusalForm::CurrentExecutablePath => "current-exe",
            RefusalForm::ExternalFunctionInterface => "extern-function-interface",
            RefusalForm::ModuleFileMissing => "module-file-missing",
            RefusalForm::BuildScriptMissing => "build-script-missing",
            RefusalForm::BinaryTargetMissing => "binary-target-missing",
            RefusalForm::WgslUndefinedName => "wgsl-undefined-name",
            RefusalForm::WgslUnrecognizedItemStart => "wgsl-unrecognized-item-start",
            RefusalForm::TargetTripleUnrecognized => "target-triple-unrecognized",
            RefusalForm::LockDependencyUnresolved => "lock-dependency-unresolved",
            RefusalForm::VerusGhostCodeUnparsed => "verus-ghost-code-unparsed",
        }
    }
}

/// 一条拒算：形态、落在哪份文件哪一行、原样片段。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Refusal {
    pub file: FileIndex,
    pub line: u32,
    pub form: RefusalForm,
    pub fragment: String,
}

/// 一处读文件是怎么写的。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReadOrigin {
    /// `include_str!` / `include_bytes!` / 表达式里的 `include!`，编译时读。
    Included,
    /// 运行期按 `env!("CARGO_MANIFEST_DIR")` 拼出字面路径读。
    Runtime,
}

/// 单元里读到的一样外部输入。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceRead {
    /// 一份文件：路径（绝对、已按字面规整）、读的那一行、内容的 SHA-256（读不了是 `None`）。
    File {
        absolute_path: PathBuf,
        line: u32,
        origin: ReadOrigin,
        digest: Option<Sha256Digest>,
    },
    /// `include_*!(concat!(env!("OUT_DIR"), …))`：连到本 crate 构建脚本的 `main` 闭包。
    BuildScriptOutput { line: u32 },
    /// `env!("CARGO_BIN_EXE_<名>")`：连到那个 bin 的 `main` 闭包。
    BinaryExecutable { binary_name: String, line: u32 },
}

/// 一个单元里出现的名字，按 3.3 的规则分好类。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UnitReferences {
    /// 不在 `.` 后面的标识符（含 `Q::x` 里的 `Q` 与 `x`，含格式串里 `{名字}` 捕获的名字），按 R-裸名 解。
    pub plain_names: Vec<String>,
    /// `.x(`、`.x::<` 的 `x`，按 R-方法 解。
    pub method_names: Vec<String>,
    /// `Q::x`，按 R-路径 解。
    pub qualified_paths: Vec<(String, String)>,
    /// `dyn Trait` 的 `Trait`：闭包内部不按 R-开放 收全部 impl（只在弄坏开关 `dyn-traits-open-every-implementation` 打开时收），
    /// trait 自己按 R-裸名 解到 trait 头。
    pub dyn_trait_names: Vec<String>,
    /// 函数签名里（函数体之前）`impl`、`dyn`、`:`、`+` 后面的名字：这个函数当入口时按 R-开放 解。
    pub signature_trait_names: Vec<String>,
}

/// 一条 `use` 引进的名字。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UseImports {
    pub is_glob: bool,
    pub names: Vec<String>,
    /// （别名，原名）：`use a::b as c` 记（c，b）。
    pub aliases: Vec<(String, String)>,
}

/// impl 头的形状：自身类型里的名字、trait 自己的名字（`for` 之前那一截里尖括号外的最后一个名字，`dyn` 不算；
/// `impl Wrap<dyn Sink> for Holder` 的 trait 是 `Wrap`，不是 `Sink`）、是不是 blanket impl（自身类型里任何深度出现这个 impl 自己的一个泛型参数：
/// `impl<T: Copy> Shout for T`、`for &T`、`for Box<T>`、`for (u32, T)`、`for [T; N]` 都算；全具体的 `for Foo` 不算）。R-开放（只在入口签名上收）与 R-路径按 trait 自己的名字找 impl，R-方法与 R-路径放行 blanket impl（定案 3）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ImplementationShape {
    pub self_type_names: Vec<String>,
    pub trait_name: Option<String>,
    pub is_trait_implementation: bool,
    pub is_blanket_implementation: bool,
}

/// `mod 名;` 的形状。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleDeclarationShape {
    pub module_name: String,
    pub path_attribute: Option<String>,
}

/// 文档测试单元挂在哪：一个条目的文档注释，或一个模块的内文档注释（模块路径是文件里的行内路径）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DocumentationTestAttachment {
    Item(UnitIndex),
    Module(Vec<String>),
}

/// 一个 Rust 单元。
#[derive(Clone, Debug)]
pub struct Unit {
    pub kind: UnitKind,
    pub name: String,
    /// 切分时还不知道模块路径，工作区装好之后才填（见 `workspace`）。
    pub qualified_name: String,
    pub file: FileIndex,
    /// 文件里的行内模块路径（`mod a { mod b { … } }` 里的 `["a", "b"]`）。
    pub local_module_path: Vec<String>,
    /// 成员所在的 impl / trait 头的标签（`<impl Add for Money>` 这类）。
    pub container_label: Option<String>,
    pub start_line: u32,
    pub end_line: u32,
    /// 在文件词法单元里的区间（文档测试单元没有，它的词法单元另从文档注释里切）。
    pub token_range: Option<(usize, usize)>,
    pub container: Option<UnitIndex>,
    pub module_head: Option<UnitIndex>,
    /// 文件里从外到内的条件编译谓词（外层行内模块、impl / trait 头与它自己的属性），与起来。文档测试单元取它所挂条目的那一串。
    pub conditions: Vec<Predicate>,
    /// 单元自己的词法单元里写出来的条件编译谓词：`#[cfg(谓词)]`、`#![cfg(谓词)]`、`#[cfg_attr(条件, 属性…)]` 的条件与属性里套的
    /// `cfg(…)`、`cfg_attr(…)`，以及 `cfg!(谓词)`（函数体里的也算）。复用键的条件成分只取闭包里各单元的这一份与 `conditions` 真引到的原子（定案 2）。
    pub condition_predicates_in_the_tokens: Vec<Predicate>,
    pub is_public: bool,
    pub fingerprint: Sha256Digest,
    pub references: UnitReferences,
    /// 按名字能找到它的那些名字：一般就是自己的名字；条目级宏调用是它里面所有可能定义出来的名字；extern 块是它声明的函数与静态量。
    pub declared_names: Vec<String>,
    pub imports: Option<UseImports>,
    pub implementation: Option<ImplementationShape>,
    pub module_declaration: Option<ModuleDeclarationShape>,
    pub reads: Vec<SourceRead>,
    /// 这个单元一进闭包就要报的拒算。
    pub refusals: Vec<Refusal>,
    pub documentation_test_of: Option<DocumentationTestAttachment>,
}
