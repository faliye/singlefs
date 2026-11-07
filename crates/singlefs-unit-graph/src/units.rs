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

/// Rust 单元的种类（设计 3.2 那张清单，外加 Verus 的 `broadcast group`、`assume_specification` 与模块头）。
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
    /// `verus!` 里的 `assume_specification[ 路径 ] …;`（整条到 `;`，连属性）：给方括号里点名的那一项（常是别的 crate 里的常量或函数）
    /// 交一份 Verus 信的规格。按那条路径末段的名字找得到它：闭包点到那个名字时连它一起收，改它交回的值、改它的 ensures，
    /// 点到那个名字的形式化判定的键跟着变。产品行为视图里它整条剥掉（`verus_ghost_code`），不成单元。
    VerusAssumeSpecification,
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
            UnitKind::VerusAssumeSpecification => "assume_specification",
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
            | UnitKind::VerusAssumeSpecification
            | UnitKind::DocumentationTest => false,
        }
    }

    /// 按裸名能不能找到它（R-裸名）：use、模块头、模块声明、外部 crate 声明与文档测试不按名字找。
    /// `assume_specification` 按它点名的那一项的名字找（`declared_names`），不按它自己的名字。
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
            | UnitKind::VerusBroadcastGroup
            | UnitKind::VerusAssumeSpecification => true,
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
            | UnitKind::VerusAssumeSpecification
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
    /// Rust 闭包读到的 shell 脚本里 `source` / `.` 的路径折不出（`shell_sources`）。
    ShellPathUnfolded,
    /// `source` / `.` 的路径折出来落在仓根外面。
    ShellPathOutsideRoot,
    /// `source` / `.` 的路径折出来指到仓里不在的文件。
    ShellPathMissing,
    /// 读到的 shell 脚本或它 source 进来的文件 `bash -n` 过不了（有告警也算）、或包不进函数，按 bash 规范化不出来。
    ShellBashSyntax,
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
            RefusalForm::ShellPathUnfolded => "shell-path-unfolded",
            RefusalForm::ShellPathOutsideRoot => "shell-path-outside-root",
            RefusalForm::ShellPathMissing => "shell-path-missing",
            RefusalForm::ShellBashSyntax => "bash-syntax",
        }
    }
}

/// 拒算落在哪份文件。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RefusalFile {
    /// 工作区里的一份源码文件（`.rs` 或 `.wgsl`）。
    WorkspaceSourceFile(FileIndex),
    /// Rust 闭包读到的 shell 脚本，或它（传递地）source 进来的一份文件：从仓根起的路径（`shell_sources`）。
    ShellScriptFromTheRepositoryRoot(String),
}

/// 一条拒算：形态、落在哪份文件哪一行、原样片段。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Refusal {
    pub file: RefusalFile,
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
    /// `env!("X")` / `option_env!("X")` 读的变量不是 cargo 给的（不以 `CARGO` 起头、也不是 `OUT_DIR`）：值只能由本 crate 构建脚本经
    /// `cargo::rustc-env` 交进来，连到构建脚本的 `main` 闭包；没有构建脚本的不连、不拒算（值来自编译时的进程环境，不在仓里的代码里）。
    /// 与 `scripts/gate/lib_function_units.py` 的 `build-environment` 那一支同一条规则。
    BuildScriptEnvironmentValue { variable: String, line: u32 },
    /// `env!("CARGO_BIN_EXE_<名>")`：连到那个 bin 的 `main` 闭包。
    BinaryExecutable { binary_name: String, line: u32 },
    /// 运行期按清单目录拼出 `crates/` 之下（不是 `crates/` 本身）的一个目录：整个目录（递归）当一个数据单元（`data-directory`），
    /// 摘要是 [`directory_digest`]。
    Directory {
        absolute_path: PathBuf,
        line: u32,
        digest: Sha256Digest,
    },
    /// `Command::new(current_exe())` 起本测试二进制的一个子进程，参数里 `--exact` 后面跟着的测试名：闭包里解到入口目标里那个
    /// `#[test] fn`，连一条边（`current-exe-child`）。解不出来时照旧拒算 `current-exe`，`fragment` 就是那一条的原样片段。
    ChildTestProcess {
        line: u32,
        test_name: ChildTestName,
        fragment: String,
    },
}

/// 子进程测试名的一段：字面串，或同一 crate 里 `const 名: &str = "…";` 的名字（闭包里再解成它的值）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TestNamePiece {
    Literal(String),
    Constant(String),
}

/// `--exact` 后面那个测试名是怎么写的。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChildTestName {
    /// 由字面串与常量拼成（字面串、常量名、`format!`、`concat!`）：几段依次连起来就是测试名。
    Pieces(Vec<TestNamePiece>),
    /// 起子进程的函数自己的一个参数（`parameter_position` 从 0 起，不带 `self` 的函数）：测试名取闭包里每一处
    /// `function_name(…)` 调用在这个位置上交的实参。
    ParameterOfTheSpawningFunction {
        function_name: String,
        parameter_position: usize,
    },
}

/// 目录数据单元的摘要：目录下（递归）每份文件按相对路径（以 `/` 分段）排好，逐份写一行「相对路径 \t 内容的 SHA-256」、
/// 每行以 `\n` 收尾，整段的 SHA-256。指到目录的符号链接不进（与 python `os.walk` 不跟链接同一种走法），指到文件的照读它指到的内容。
///
/// # Errors
/// 目录或其中一份文件读不了，交读不了的那条路径。
pub fn directory_digest(directory: &std::path::Path) -> Result<Sha256Digest, PathBuf> {
    let mut files: Vec<(String, PathBuf)> = Vec::new();
    let mut directories_to_visit: Vec<(PathBuf, String)> =
        vec![(directory.to_path_buf(), String::new())];
    while let Some((visiting, relative_prefix)) = directories_to_visit.pop() {
        let entries = std::fs::read_dir(&visiting).map_err(|_| visiting.clone())?;
        for entry in entries {
            let entry = entry.map_err(|_| visiting.clone())?;
            let entry_path = entry.path();
            let entry_name = entry.file_name().to_string_lossy().into_owned();
            let relative = format!("{relative_prefix}{entry_name}");
            let file_type = entry.file_type().map_err(|_| entry_path.clone())?;
            if file_type.is_dir() {
                directories_to_visit.push((entry_path, format!("{relative}/")));
            } else if file_type.is_symlink() {
                if entry_path.is_file() {
                    files.push((relative, entry_path));
                }
            } else {
                files.push((relative, entry_path));
            }
        }
    }
    files.sort();
    let mut listing = String::new();
    for (relative, path) in &files {
        let bytes = std::fs::read(path).map_err(|_| path.clone())?;
        listing.push_str(relative);
        listing.push('\t');
        listing.push_str(&crate::sha256::sha256_of(&bytes).hexadecimal());
        listing.push('\n');
    }
    Ok(crate::sha256::sha256_of(listing.as_bytes()))
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
/// `impl<T: Copy> Shout for T`、`for &T`、`for Box<T>`、`for (u32, T)`、`for [T; N]` 都算；全具体的 `for Foo` 不算）。R-开放（只在入口签名上收）与 R-路径按 trait 自己的名字找 impl，R-方法与 R-路径放行 blanket impl。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ImplementationShape {
    pub self_type_names: Vec<String>,
    pub trait_name: Option<String>,
    pub is_trait_implementation: bool,
    pub is_blanket_implementation: bool,
    /// 自身类型写成路径 `a::B` 时的头一段（`impl Tr for std::time::Duration` 的 `std`）；自身类型头一个名字后面不接 `::` 的是 `None`。
    /// 认外部类型上的本仓 trait impl 用（`workspace` 的 `foreign_type_trait_implementations`），与 `scripts/gate/lib_function_units.py`
    /// 的 `self_path_root` 同一个认法：跳过 `dyn`、`mut`、`const`、`impl`、`for`、`where`、`as`、`unsafe` 之后的头一个名字。
    pub self_type_path_root: Option<String>,
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
    /// `cfg(…)`、`cfg_attr(…)`，以及 `cfg!(谓词)`（函数体里的也算）。复用键的条件成分只取闭包里各单元的这一份与 `conditions` 真引到的原子。
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
    /// 函数单元带 `#[test]` 属性（子进程测试名只解到这种函数；WGSL 名字供给不收它，入口本身除外）。
    pub has_test_attribute: bool,
    /// 外层的行内模块、文件开头的内属性或 impl / trait 头的条件链里有 `test` 这一项（与起来的一项，`all(…)` 里的也算）：
    /// 只在测试里编进来的代码，WGSL 名字供给不收它。
    pub is_inside_test_only_code: bool,
}
