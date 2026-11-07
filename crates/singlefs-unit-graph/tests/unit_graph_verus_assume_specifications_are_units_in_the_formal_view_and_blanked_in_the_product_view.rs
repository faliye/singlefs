//! `verus!` 里的 `assume_specification[ 路径 ] …;`（核心层给 `singlefs-format` 常量交 Verus 信的值就是这么写的）：
//! 形式化判定的视图里整条到 `;` 一个单元（种类 `assume_specification`），带可见性与属性、带 `returns`、不带 `returns` 而带参数表与
//! ensures、带泛型参数表的几种写法都认，dump 一条拒算都没有；按方括号里路径末段的名字找得到它，点到那个名字的闭包连它一起收，
//! 改它交回的值，形式化键跟着变。产品行为视图里它整条剥掉，不成单元，产品行为键不变。

mod common_fixture_workspace;

use common_fixture_workspace::{key_of, workspace_of, EntrySpecification, FixtureTree};
use singlefs_unit_graph::output::dump_lines;
use singlefs_unit_graph::rule_switches::RuleSwitches;
use singlefs_unit_graph::units::UnitKind;
use singlefs_unit_graph::{JudgmentKind, Workspace};

const FIXTURE_LIBRARY: &str = "crates/fassume/src/lib.rs";

/// 夹具：一个 crate，`verus!` 外有一个常量、一个 exec 函数与点到那个常量的判法入口；`verus!` 里三种写法的
/// `assume_specification`（各在哪几行见形式化视图那一条用例）与一个 spec 函数。
const ASSUME_SPECIFICATION_FIXTURE: [(&str, &str); 3] = [
    (
        "crates/fassume/Cargo.toml",
        "[package]\nname = \"fassume\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    ),
    (
        FIXTURE_LIBRARY,
        "pub const LIMIT_IN_BYTES: u64 = 32;\n\
         \n\
         pub fn width_of(value: u64) -> u64 {\n    value + 1\n}\n\
         \n\
         pub fn judge_limit() -> u64 {\n    LIMIT_IN_BYTES\n}\n\
         \n\
         verus! {\n\
         \n\
         #[allow(unsafe_code, reason = \"夹具：Verus 把 assume_specification 展开成 unsafe fn 声明\")]\n\
         pub assume_specification[ crate::LIMIT_IN_BYTES ] -> (limit: u64) returns 32u64;\n\
         \n\
         assume_specification[ width_of ](value: u64) -> (width: u64)\n    ensures width == value + 1;\n\
         \n\
         pub assume_specification<Element>[ Vec::<Element>::len ](vector: &Vec<Element>) -> (length: usize);\n\
         \n\
         pub open spec fn spec_limit() -> u64 {\n    32\n}\n\
         \n\
         }\n",
    ),
    ("Cargo.lock", "# 夹具不解析依赖\n"),
];

/// 一个 `assume_specification` 单元在视图里的样子。
#[derive(Debug, PartialEq, Eq)]
struct AssumeSpecificationUnit {
    name: String,
    start_line: u32,
    end_line: u32,
    /// 按哪几个名字找得到它（R-裸名）。
    found_by_names: Vec<String>,
    is_public: bool,
}

fn expected_unit(
    name: &str,
    start_line: u32,
    end_line: u32,
    found_by_name: &str,
    is_public: bool,
) -> AssumeSpecificationUnit {
    AssumeSpecificationUnit {
        name: name.to_string(),
        start_line,
        end_line,
        found_by_names: vec![found_by_name.to_string()],
        is_public,
    }
}

/// 夹具上点到 `LIMIT_IN_BYTES` 的判法入口。
const FIXTURE_JUDGE: EntrySpecification = EntrySpecification {
    entry_path: "fassume::judge_limit",
    target: "lib",
    conditions: &[],
};

const LIMIT_SPECIFICATION_RETURNS: &str = "returns 32u64;";
const LIMIT_SPECIFICATION_RETURNS_ANOTHER_VALUE: &str = "returns 33u64;";

fn refusal_lines(workspace: &Workspace) -> Vec<String> {
    dump_lines(workspace)
        .into_iter()
        .filter(|line| line.starts_with("refused\t"))
        .collect()
}

/// 视图里每个 `assume_specification` 单元。
fn assume_specification_units(workspace: &Workspace) -> Vec<AssumeSpecificationUnit> {
    workspace
        .units
        .iter()
        .filter(|unit| unit.kind == UnitKind::VerusAssumeSpecification)
        .map(|unit| AssumeSpecificationUnit {
            name: unit.name.clone(),
            start_line: unit.start_line,
            end_line: unit.end_line,
            found_by_names: unit.declared_names.clone(),
            is_public: unit.is_public,
        })
        .collect()
}

fn has_unit_named(workspace: &Workspace, kind: UnitKind, name: &str) -> bool {
    workspace
        .units
        .iter()
        .any(|unit| unit.kind == kind && unit.name == name)
}

#[test]
fn every_assume_specification_form_is_one_unit_with_no_refusal_in_the_formal_view() {
    let tree =
        FixtureTree::materialize("assume-specification-formal", &ASSUME_SPECIFICATION_FIXTURE);
    let switches = RuleSwitches::every_rule_intact();
    let workspace = workspace_of(tree.root(), &switches);
    let formal = workspace.view_for_judgment(JudgmentKind::FormalVerification);
    assert_eq!(
        refusal_lines(formal),
        Vec::<String>::new(),
        "三种写法的 assume_specification 都认得：形式化视图的 dump 一条拒算都没有"
    );
    // 行号照夹具里的写法：属性在第 13 行、第一条规格在第 14 行；不带 returns 的那条跨第 16、17 行；带泛型参数表的在第 19 行
    let expected = vec![
        expected_unit("assume_specification@14", 13, 14, "LIMIT_IN_BYTES", true),
        expected_unit("assume_specification@16", 16, 17, "width_of", false),
        expected_unit("assume_specification@19", 19, 19, "len", true),
    ];
    assert_eq!(
        assume_specification_units(formal),
        expected,
        "每条 assume_specification 整条到 `;` 一个单元（连属性），名字带关键字那一行，按点名路径末段的名字找得到它"
    );
    assert!(
        has_unit_named(formal, UnitKind::Function, "spec_limit"),
        "assume_specification 后面那个 spec 函数照旧切成自己的单元，不被吞进前一条"
    );
}

#[test]
fn the_product_behavior_view_blanks_every_assume_specification_with_no_refusal() {
    let tree = FixtureTree::materialize(
        "assume-specification-product",
        &ASSUME_SPECIFICATION_FIXTURE,
    );
    let switches = RuleSwitches::every_rule_intact();
    let workspace = workspace_of(tree.root(), &switches);
    let product = workspace.view_for_judgment(JudgmentKind::ProductBehavior);
    assert_eq!(
        refusal_lines(product),
        Vec::<String>::new(),
        "产品行为视图的 dump 一条拒算都没有（verus_syn 解得开这几种写法）"
    );
    assert_eq!(
        assume_specification_units(product),
        Vec::new(),
        "产品行为视图里 assume_specification 整条剥掉，不成单元"
    );
    assert!(
        !has_unit_named(product, UnitKind::Function, "spec_limit"),
        "spec 函数在产品行为视图里同样剥掉"
    );
    assert!(
        has_unit_named(product, UnitKind::Function, "judge_limit")
            && has_unit_named(product, UnitKind::Constant, "LIMIT_IN_BYTES"),
        "verus! 外的执行代码两个视图都在"
    );
}

/// 夹具上把 `LIMIT_IN_BYTES` 那条规格交回的值换成 `returns`（`None` 是不改）之后，判法入口在这类判定的视图上的键。
fn fixture_judge_key(returns: Option<&str>, kind: JudgmentKind) -> String {
    let tree = FixtureTree::materialize("assume-specification-key", &ASSUME_SPECIFICATION_FIXTURE);
    if let Some(returns) = returns {
        tree.replace_exactly_once(FIXTURE_LIBRARY, LIMIT_SPECIFICATION_RETURNS, returns);
    }
    let switches = RuleSwitches::every_rule_intact();
    let workspace = workspace_of(tree.root(), &switches);
    key_of(workspace.view_for_judgment(kind), &FIXTURE_JUDGE, &switches)
        .unwrap_or_else(|reason| panic!("夹具的判法入口交得出键：{reason}"))
        .hexadecimal()
}

#[test]
fn changing_the_value_an_assume_specification_returns_changes_the_formal_key_of_a_function_naming_the_constant_and_keeps_its_product_key(
) {
    assert_ne!(
        fixture_judge_key(None, JudgmentKind::FormalVerification),
        fixture_judge_key(
            Some(LIMIT_SPECIFICATION_RETURNS_ANOTHER_VALUE),
            JudgmentKind::FormalVerification
        ),
        "判法入口点到 LIMIT_IN_BYTES：它的 assume_specification 按这个名字进闭包，交回的值一改，形式化键变"
    );
    assert_eq!(
        fixture_judge_key(None, JudgmentKind::ProductBehavior),
        fixture_judge_key(
            Some(LIMIT_SPECIFICATION_RETURNS_ANOTHER_VALUE),
            JudgmentKind::ProductBehavior
        ),
        "产品行为视图里 assume_specification 剥掉了，交回的值怎么改，产品行为键都不变"
    );
}
