//! 弄坏开关：每条闭包规则与切分规则各一个，只给测试证红用（`.claude/rules/test-script-discipline.md`
//! 「测试脚本自己也要被测试证明有效」）。库只认调用方交进来的 [`RuleSwitches`]；命令行 `unit-graph` 不读环境变量、一律全开，
//! 测试读 [`BREAK_ENVIRONMENT_VARIABLE`] 交进来。
//!
//! 开关名（逗号隔开可给几个）、弄坏哪一处、打开后 `tests/unit_graph_fixture_keys_change_exactly_where_the_rules_say.rs` 哪几格该红：
//!   target-scope                          R-目标：名字在每个编译目标里都解、不看 `pub`、链接集是全部目标 → 「没链接的 crate 里同名函数」
//!   bare-names                            R-裸名：不按裸名解 → 经裸名才到的十六格（逐格列在 `breaking_bare_names_reds_every_cell_that_is_reached_only_by_a_bare_name`）
//!   qualified-paths                       R-路径：不解 `Q::x` → 「经类型路径调关联函数」
//!   method-calls                          R-方法：不解 `.x(` → 「trait 默认方法」「经点调固有方法」
//!   trait-implementation-edges            R-隐式：类型被点名不带它的 trait impl → 「运算符重载」「Drop 隐式调用」「关联常量（经泛型）」
//!   class-hierarchy                       R-开放：入口签名里的 `dyn Trait` 与 trait 约束不收全部 impl → 「开放入口：参数是 dyn」
//!                                         「开放入口：参数是泛型约束」
//!   dyn-traits-open-every-implementation  闭包内部的 `dyn Trait` 也收链接集里这个 trait 的全部 impl（不只收被点名的类型的 impl）
//!                                         → 「dyn 分派：没被点名的另一个 impl 不进闭包」
//!   structural-edges                      R-结构：成员不连头、不连模块头与 use、别名不解 → 「use 改名之后调用」
//!   macro-units                           R-宏：`macro_rules!` 与条目级宏调用不当单元 → 「声明宏展开出的函数」「带路径的条目级宏调用定义了枚举」
//!   constant-and-static-units             const 与 static 不当单元 → 「关联常量（经泛型）」「函数指针表」「数组长度里的常量」
//!   conditional-compilation               条件编译谓词一律当真 → 「关着的特性下的单元改了」
//!   documentation-comments-in-fingerprint 文档注释进条目指纹 → 「只改文档注释」「改文档注释里的文档测试代码」
//!   documentation-test-units              文档注释里的代码块不成文档测试单元 → 「改文档注释里的文档测试代码」
//!   data-units                            不收 `include_*!` 与运行期读的文件 → 「include_str 读的文件改了」
//!   path-macro-invocations                只认不带路径的 `m! { … }`（探针的切法） → 「带路径的条目级宏调用定义了枚举」
//!   verus-clause-braces                   `verus!` 里照普通条目切（签名之后第一个 `{` 当函数体） → 「verus! 里 ensures 带花括号」
//!   wgsl-external-names                   WGSL 里用到而没定义的名字不核 → `tests/unit_graph_wgsl_modules_close_over_compute_entries.rs` 的外名拒算那一格
//!   wgsl-external-name-prefixes           外名只认「整名是 Rust 串字面量里的一个词」，不认「首段加 `_` 落在串字面量或 `macro_rules!` 体里」
//!                                         → 同一份测试里「format! 拼出的外名有着落」那一格
//!   wgsl-supplier-edges                   闭包里没有着落的 WGSL 外名，不把链接集里供它的单元并进闭包（着落只核、不做成边）
//!                                         → 同一份测试里「供外名的宏调用不经名字点到也并进闭包、改它的码键变」那一格

use std::collections::BTreeSet;

/// 测试读的环境变量：值是逗号隔开的开关名。
pub const BREAK_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_UNIT_GRAPH_BREAK";

/// 一条可以弄坏的规则。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BrokenRule {
    TargetScope,
    BareNames,
    QualifiedPaths,
    MethodCalls,
    TraitImplementationEdges,
    ClassHierarchy,
    DynTraitsOpenEveryImplementation,
    StructuralEdges,
    MacroUnits,
    ConstantAndStaticUnits,
    ConditionalCompilation,
    DocumentationCommentsInFingerprint,
    DocumentationTestUnits,
    DataUnits,
    PathMacroInvocations,
    VerusClauseBraces,
    WgslExternalNames,
    WgslExternalNamePrefixes,
    WgslSupplierEdges,
}

impl BrokenRule {
    /// 全部开关，按文件头那张表的次序。
    pub const EVERY_RULE: [BrokenRule; 19] = [
        BrokenRule::TargetScope,
        BrokenRule::BareNames,
        BrokenRule::QualifiedPaths,
        BrokenRule::MethodCalls,
        BrokenRule::TraitImplementationEdges,
        BrokenRule::ClassHierarchy,
        BrokenRule::DynTraitsOpenEveryImplementation,
        BrokenRule::StructuralEdges,
        BrokenRule::MacroUnits,
        BrokenRule::ConstantAndStaticUnits,
        BrokenRule::ConditionalCompilation,
        BrokenRule::DocumentationCommentsInFingerprint,
        BrokenRule::DocumentationTestUnits,
        BrokenRule::DataUnits,
        BrokenRule::PathMacroInvocations,
        BrokenRule::VerusClauseBraces,
        BrokenRule::WgslExternalNames,
        BrokenRule::WgslExternalNamePrefixes,
        BrokenRule::WgslSupplierEdges,
    ];

    /// 环境变量里写的开关名。
    #[must_use]
    pub fn switch_name(self) -> &'static str {
        match self {
            BrokenRule::TargetScope => "target-scope",
            BrokenRule::BareNames => "bare-names",
            BrokenRule::QualifiedPaths => "qualified-paths",
            BrokenRule::MethodCalls => "method-calls",
            BrokenRule::TraitImplementationEdges => "trait-implementation-edges",
            BrokenRule::ClassHierarchy => "class-hierarchy",
            BrokenRule::DynTraitsOpenEveryImplementation => "dyn-traits-open-every-implementation",
            BrokenRule::StructuralEdges => "structural-edges",
            BrokenRule::MacroUnits => "macro-units",
            BrokenRule::ConstantAndStaticUnits => "constant-and-static-units",
            BrokenRule::ConditionalCompilation => "conditional-compilation",
            BrokenRule::DocumentationCommentsInFingerprint => {
                "documentation-comments-in-fingerprint"
            }
            BrokenRule::DocumentationTestUnits => "documentation-test-units",
            BrokenRule::DataUnits => "data-units",
            BrokenRule::PathMacroInvocations => "path-macro-invocations",
            BrokenRule::VerusClauseBraces => "verus-clause-braces",
            BrokenRule::WgslExternalNames => "wgsl-external-names",
            BrokenRule::WgslExternalNamePrefixes => "wgsl-external-name-prefixes",
            BrokenRule::WgslSupplierEdges => "wgsl-supplier-edges",
        }
    }

    /// 按开关名找开关。
    #[must_use]
    pub fn from_switch_name(name: &str) -> Option<BrokenRule> {
        BrokenRule::EVERY_RULE
            .into_iter()
            .find(|rule| rule.switch_name() == name)
    }
}

/// 这一次切分与收闭包里哪几条规则被弄坏了。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuleSwitches {
    broken: BTreeSet<BrokenRule>,
}

impl RuleSwitches {
    /// 规则全开（平常跑与命令行一律用这个）。
    #[must_use]
    pub fn every_rule_intact() -> Self {
        Self {
            broken: BTreeSet::new(),
        }
    }

    /// 弄坏点名的几条。
    #[must_use]
    pub fn with_broken(rules: &[BrokenRule]) -> Self {
        Self {
            broken: rules.iter().copied().collect(),
        }
    }

    /// 这条规则没被弄坏。
    #[must_use]
    pub fn is_intact(&self, rule: BrokenRule) -> bool {
        !self.broken.contains(&rule)
    }

    /// 从环境变量的值读开关（只给测试用）。空串是全开。
    ///
    /// # Errors
    /// 有一个开关名认不得，交那个名字。
    pub fn from_break_variable_text(text: &str) -> Result<Self, String> {
        let mut broken = BTreeSet::new();
        for name in text
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
        {
            match BrokenRule::from_switch_name(name) {
                Some(rule) => {
                    broken.insert(rule);
                }
                None => return Err(name.to_string()),
            }
        }
        Ok(Self { broken })
    }
}
