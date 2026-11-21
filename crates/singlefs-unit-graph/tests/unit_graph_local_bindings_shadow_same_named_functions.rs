//! R-裸名局部绑定遮住同名函数（`src/closure.rs` 文件头「R-裸名局部绑定遮住同名函数」）：函数里名字每一处都是 `let` 或参数绑定的、或落在绑定的作用域里的，
//! 不往外连同名函数；认不准的（绑定之前调了、初始值里调了、块外调了、带路径的、函数体里有嵌套条目的、宏调用括号里的 `let`、`if let`、作用域外的格式串捕获、
//! 落在名单外的宏调用括号里的）照连。名单是 `src/closure.rs` 的 `STANDARD_MACROS_EVALUATING_THEIR_ARGUMENTS_IN_PLACE`（标准库那 16 个就地求值实参的宏），
//! 名单每一项、名单外的宏与宏调用括号里的 `let` 这几形的用例在 `unit_graph_shapes_that_hide_the_callee_widen_the_closure_or_refuse_the_key.rs`。
//! 夹具只切词、不编译。真仓的形状是 `let main = SamplingPoint::Main;`，名字写进名单里的 `format!` 的实参与闭包：不连可执行文件自己的 `fn main`；
//! 写进名单外的宏（工作区的、外部 crate 的）括号里的照连。
//! 每条用例改被它点名的那个同名条目，判入口的键变不变；弄坏开关 `local-bindings-shadow-same-named-functions`（`src/rule_switches.rs`）打开时遮住的也连。

mod common_fixture_workspace;

use common_fixture_workspace::{key_of, workspace_of, EntrySpecification, FixtureTree};
use singlefs_unit_graph::rule_switches::{BrokenRule, RuleSwitches};

const SHADOW_FILE: &str = "crates/fshadow/src/lib.rs";
const OTHER_CRATE_FILE: &str = "crates/fother/src/lib.rs";

const SHADOW_LIBRARY: &str = r#"pub fn helper_total() -> u32 {
    1
}

fn private_helper_total() -> u32 {
    4
}

pub fn calls_the_private_function() -> u32 {
    private_helper_total() + 1
}

pub const LIMIT: u32 = 3;

pub fn pick(value: u32) -> u32 {
    value
}

pub fn shadowed_by_a_let() -> u32 {
    let helper_total = 7;
    helper_total + 1
}

pub fn calls_the_function() -> u32 {
    helper_total() + 1
}

pub fn calls_the_function_before_the_let() -> u32 {
    let earlier = helper_total();
    let helper_total = earlier + 1;
    helper_total
}

pub fn calls_the_function_in_the_initializer() -> u32 {
    let helper_total = helper_total() + 1;
    helper_total
}

pub fn calls_the_function_after_the_block_binding_the_name() -> u32 {
    {
        let helper_total = 3;
        drop(helper_total);
    }
    helper_total()
}

pub fn binds_the_name_only_inside_a_block() -> u32 {
    let kept = {
        let helper_total = 3;
        helper_total + 1
    };
    kept
}

pub fn shadowed_by_a_parameter(helper_total: u32) -> u32 {
    helper_total + 1
}

pub fn shadowed_by_a_mutable_parameter(mut helper_total: u32) -> u32 {
    helper_total += 1;
    helper_total
}

pub fn shadowed_by_a_parameter_of_a_generic_function<Transform: Fn(u32) -> u32>(
    transform: Transform,
    helper_total: u32,
) -> u32 {
    transform(helper_total)
}

pub fn shadowed_by_a_mutable_let_with_a_type() -> u32 {
    let mut helper_total: u32 = 7;
    helper_total += 1;
    helper_total
}

pub fn calls_the_function_by_path_in_the_scope_of_the_let() -> u32 {
    let helper_total = 7;
    crate::helper_total() + helper_total
}

pub fn starts_a_path_with_the_name_in_the_scope_of_the_let() -> u32 {
    let helper_total = 7;
    helper_total::read() + helper_total
}

pub fn calls_a_macro_of_the_name_in_the_scope_of_the_let() -> u32 {
    let helper_total = 7;
    helper_total!() + helper_total
}

pub fn calls_the_function_in_a_nested_function() -> u32 {
    let helper_total = 7;
    fn nested_total() -> u32 {
        helper_total()
    }
    nested_total() + helper_total
}

pub fn binds_the_name_of_a_constant() -> u32 {
    let LIMIT = 3;
    LIMIT
}

pub fn if_let_binds_the_name_and_a_later_statement_calls_the_function() -> u32 {
    if let helper_total = pick(1) {
    }
    drop(0);
    helper_total()
}

pub fn format_capture_before_the_let() -> u32 {
    let text = format!("{helper_total}");
    let helper_total = 7;
    drop(text);
    helper_total
}

pub fn shadowed_and_used_in_macro_arguments_closures_and_captures() -> Vec<String> {
    let helper_total = 7;
    let mut lines = Vec::new();
    lines.push(format!("{}", pick(helper_total)));
    let shifted = |offset: u32| pick(helper_total) + offset;
    lines.push(format!("{helper_total} {}", shifted(1)));
    lines
}

pub mod records {
    pub struct Record {
        pub helper_total: u32,
    }
}

pub fn reads_a_field_of_the_same_name_before_the_let(record: &records::Record) -> u32 {
    let earlier = record.helper_total;
    let helper_total = earlier + 1;
    helper_total
}

pub fn shadows_a_function_of_another_crate() -> u32 {
    let other_crate_total = 1;
    other_crate_total + 1
}

pub fn calls_a_function_of_another_crate() -> u32 {
    other_crate_total() + 1
}

pub mod same_name_as_a_parameter {
    pub fn other_crate_total(other_crate_total: u32) -> u32 {
        other_crate_total + 1
    }
}

pub mod same_name_without_a_binding {
    pub fn other_crate_total() -> u32 {
        5
    }
}
"#;

const OTHER_CRATE_LIBRARY: &str = "pub fn other_crate_total() -> u32 {\n    10\n}\n";

const LOCAL_BINDING_FIXTURE: [(&str, &str); 5] = [
    (
        "crates/fshadow/Cargo.toml",
        "[package]\nname = \"fshadow\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nfother = { path = \"../fother\" }\n",
    ),
    (SHADOW_FILE, SHADOW_LIBRARY),
    (
        "crates/fother/Cargo.toml",
        "[package]\nname = \"fother\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    ),
    (OTHER_CRATE_FILE, OTHER_CRATE_LIBRARY),
    ("Cargo.lock", "# 夹具不解析依赖\n"),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KeyExpectation {
    Changes,
    StaysTheSame,
}

/// 在夹具里改的那一处：哪份文件、恰好出现一次的原文、换成什么。
#[derive(Clone, Copy, Debug)]
struct FixtureEdit {
    file: &'static str,
    original: &'static str,
    replacement: &'static str,
}

const EDIT_OF_THE_SAME_NAMED_FUNCTION: FixtureEdit = FixtureEdit {
    file: SHADOW_FILE,
    original: "pub fn helper_total() -> u32 {\n    1\n}",
    replacement: "pub fn helper_total() -> u32 {\n    2\n}",
};

const EDIT_OF_THE_PRIVATE_FUNCTION: FixtureEdit = FixtureEdit {
    file: SHADOW_FILE,
    original: "fn private_helper_total() -> u32 {\n    4\n}",
    replacement: "fn private_helper_total() -> u32 {\n    5\n}",
};

const EDIT_OF_THE_FUNCTION_IN_THE_OTHER_CRATE: FixtureEdit = FixtureEdit {
    file: OTHER_CRATE_FILE,
    original: "    10\n",
    replacement: "    11\n",
};

/// 改一处，判入口的键变不变；两趟都要算得出键，算不出就 panic（带入口错或拒算行）。
fn key_expectation_after_edit(
    entry_path: &'static str,
    edit: FixtureEdit,
    switches: &RuleSwitches,
) -> KeyExpectation {
    let tree = FixtureTree::materialize("local-bindings-shadow", &LOCAL_BINDING_FIXTURE);
    let entry = EntrySpecification {
        entry_path,
        target: "lib",
        conditions: &[],
    };
    let key_before = key_of(&workspace_of(tree.root(), switches), &entry, switches)
        .unwrap_or_else(|reason| panic!("{entry_path} 改前要算得出键：{reason}"));
    tree.replace_exactly_once(edit.file, edit.original, edit.replacement);
    let key_after = key_of(&workspace_of(tree.root(), switches), &entry, switches)
        .unwrap_or_else(|reason| panic!("{entry_path} 改后要算得出键：{reason}"));
    if key_after == key_before {
        KeyExpectation::StaysTheSame
    } else {
        KeyExpectation::Changes
    }
}

fn expectation_with_every_rule_intact(
    entry_path: &'static str,
    edit: FixtureEdit,
) -> KeyExpectation {
    key_expectation_after_edit(entry_path, edit, &RuleSwitches::every_rule_intact())
}

/// 场景：同一目标里裸名调私有函数，公开与否这一支帮不上忙。预期：改那个私有函数键变。
#[test]
fn call_of_a_private_function_in_the_same_target_links_it() {
    assert_eq!(
        expectation_with_every_rule_intact(
            "fshadow::calls_the_private_function",
            EDIT_OF_THE_PRIVATE_FUNCTION
        ),
        KeyExpectation::Changes,
        "同一目标里连裸名，私有函数也连"
    );
}

/// 场景：`let helper_total = 7;`，后面都用这个局部变量。预期：改同名函数键不变。
#[test]
fn let_shadows_the_same_named_function() {
    assert_eq!(
        expectation_with_every_rule_intact(
            "fshadow::shadowed_by_a_let",
            EDIT_OF_THE_SAME_NAMED_FUNCTION
        ),
        KeyExpectation::StaysTheSame,
        "helper_total 只是局部变量，不连 fn helper_total"
    );
}

/// 场景：同一个入口，弄坏开关 local-bindings-shadow-same-named-functions。预期：改同名函数键变。
#[test]
fn with_local_bindings_shadow_same_named_functions_broken_a_let_does_not_shadow() {
    assert_eq!(
        key_expectation_after_edit(
            "fshadow::shadowed_by_a_let",
            EDIT_OF_THE_SAME_NAMED_FUNCTION,
            &RuleSwitches::with_broken(&[BrokenRule::LocalBindingsShadowSameNamedFunctions]),
        ),
        KeyExpectation::Changes,
        "开关打开时局部绑定不遮，照旧连 fn helper_total"
    );
}

/// 场景：没有局部绑定，直接调同名函数。预期：改函数键变。
#[test]
fn call_without_any_binding_links_the_function() {
    assert_eq!(
        expectation_with_every_rule_intact(
            "fshadow::calls_the_function",
            EDIT_OF_THE_SAME_NAMED_FUNCTION
        ),
        KeyExpectation::Changes,
        "没有局部绑定，helper_total() 连 fn helper_total"
    );
}

/// 场景：`let` 之前先调了同名函数。预期：改函数键变。
#[test]
fn call_before_the_let_links_the_function() {
    assert_eq!(
        expectation_with_every_rule_intact(
            "fshadow::calls_the_function_before_the_let",
            EDIT_OF_THE_SAME_NAMED_FUNCTION
        ),
        KeyExpectation::Changes,
        "绑定之前的 helper_total() 指函数"
    );
}

/// 场景：`let helper_total = helper_total() + 1;`。预期：改函数键变（走到 `;` 才算绑定了）。
#[test]
fn call_in_the_initializer_of_the_let_links_the_function() {
    assert_eq!(
        expectation_with_every_rule_intact(
            "fshadow::calls_the_function_in_the_initializer",
            EDIT_OF_THE_SAME_NAMED_FUNCTION
        ),
        KeyExpectation::Changes,
        "初始值里的 helper_total() 还指函数"
    );
}

/// 场景：内层块里绑定，块收尾之后调同名函数。预期：改函数键变。
#[test]
fn call_after_the_block_of_the_let_links_the_function() {
    assert_eq!(
        expectation_with_every_rule_intact(
            "fshadow::calls_the_function_after_the_block_binding_the_name",
            EDIT_OF_THE_SAME_NAMED_FUNCTION
        ),
        KeyExpectation::Changes,
        "块外的 helper_total() 指函数"
    );
}

/// 场景：只在内层块里绑定、只在块里用。预期：改函数键不变。
#[test]
fn let_used_only_inside_its_block_shadows_the_function() {
    assert_eq!(
        expectation_with_every_rule_intact(
            "fshadow::binds_the_name_only_inside_a_block",
            EDIT_OF_THE_SAME_NAMED_FUNCTION
        ),
        KeyExpectation::StaysTheSame,
        "块里的 helper_total 都是局部变量"
    );
}

/// 场景：函数的参数叫 `helper_total`。预期：改同名函数键不变。
#[test]
fn parameter_shadows_the_same_named_function() {
    assert_eq!(
        expectation_with_every_rule_intact(
            "fshadow::shadowed_by_a_parameter",
            EDIT_OF_THE_SAME_NAMED_FUNCTION
        ),
        KeyExpectation::StaysTheSame,
        "helper_total 只是参数"
    );
}

/// 场景：`mut helper_total: u32` 参数。预期：改同名函数键不变。
#[test]
fn mutable_parameter_shadows_the_same_named_function() {
    assert_eq!(
        expectation_with_every_rule_intact(
            "fshadow::shadowed_by_a_mutable_parameter",
            EDIT_OF_THE_SAME_NAMED_FUNCTION
        ),
        KeyExpectation::StaysTheSame,
        "mut 不改变这是参数"
    );
}

/// 场景：泛型函数（约束里带 `Fn(u32) -> u32`）的参数叫 `helper_total`。预期：改同名函数键不变。
#[test]
fn parameter_of_a_generic_function_shadows_the_same_named_function() {
    assert_eq!(
        expectation_with_every_rule_intact(
            "fshadow::shadowed_by_a_parameter_of_a_generic_function",
            EDIT_OF_THE_SAME_NAMED_FUNCTION
        ),
        KeyExpectation::StaysTheSame,
        "尖括号里的括号不是参数表，helper_total 只是参数"
    );
}

/// 场景：`let mut helper_total: u32 = 7;`。预期：改同名函数键不变。
#[test]
fn mutable_let_with_a_type_shadows_the_same_named_function() {
    assert_eq!(
        expectation_with_every_rule_intact(
            "fshadow::shadowed_by_a_mutable_let_with_a_type",
            EDIT_OF_THE_SAME_NAMED_FUNCTION
        ),
        KeyExpectation::StaysTheSame,
        "mut 与类型标注不改变这是局部绑定"
    );
}

/// 场景：`let` 的作用域里写 `crate::helper_total()`。预期：改函数键变。
#[test]
fn call_by_path_in_the_scope_of_the_let_links_the_function() {
    assert_eq!(
        expectation_with_every_rule_intact(
            "fshadow::calls_the_function_by_path_in_the_scope_of_the_let",
            EDIT_OF_THE_SAME_NAMED_FUNCTION
        ),
        KeyExpectation::Changes,
        "带路径的名字不解到局部变量"
    );
}

/// 场景：`let` 的作用域里写 `helper_total::read()`（名字当路径的头一段）。预期：改同名函数键变（带路径的一律不算遮住）。
#[test]
fn path_starting_with_the_name_in_the_scope_of_the_let_does_not_shadow() {
    assert_eq!(
        expectation_with_every_rule_intact(
            "fshadow::starts_a_path_with_the_name_in_the_scope_of_the_let",
            EDIT_OF_THE_SAME_NAMED_FUNCTION
        ),
        KeyExpectation::Changes,
        "名字后面接 :: 的不当局部变量"
    );
}

/// 场景：`let` 的作用域里写 `helper_total!()`（宏名）。预期：改同名函数键变（宏名一律不算遮住）。
#[test]
fn macro_call_of_the_name_in_the_scope_of_the_let_does_not_shadow() {
    assert_eq!(
        expectation_with_every_rule_intact(
            "fshadow::calls_a_macro_of_the_name_in_the_scope_of_the_let",
            EDIT_OF_THE_SAME_NAMED_FUNCTION
        ),
        KeyExpectation::Changes,
        "名字后面接 ! 与开括号的不当局部变量"
    );
}

/// 场景：`let` 之后的嵌套函数里调 `helper_total()`，嵌套函数看不见外层的局部变量。预期：改函数键变。
#[test]
fn call_in_a_nested_function_links_the_function() {
    assert_eq!(
        expectation_with_every_rule_intact(
            "fshadow::calls_the_function_in_a_nested_function",
            EDIT_OF_THE_SAME_NAMED_FUNCTION
        ),
        KeyExpectation::Changes,
        "函数体里有嵌套条目的不算遮住"
    );
}

/// 场景：`let LIMIT = 3;`，同一模块有 `const LIMIT`：这是拿常量当模式，不是新绑定。预期：改常量键变。
#[test]
fn let_with_the_name_of_a_constant_links_the_constant() {
    assert_eq!(
        expectation_with_every_rule_intact(
            "fshadow::binds_the_name_of_a_constant",
            FixtureEdit {
                file: SHADOW_FILE,
                original: "pub const LIMIT: u32 = 3;",
                replacement: "pub const LIMIT: u32 = 4;",
            }
        ),
        KeyExpectation::Changes,
        "局部绑定只遮函数，常量照连"
    );
}

/// 场景：`if let helper_total = …` 的绑定只在 if 块里，后面的语句调同名函数。预期：改函数键变。
#[test]
fn an_if_let_does_not_bind_the_name_for_the_later_statements() {
    assert_eq!(
        expectation_with_every_rule_intact(
            "fshadow::if_let_binds_the_name_and_a_later_statement_calls_the_function",
            EDIT_OF_THE_SAME_NAMED_FUNCTION
        ),
        KeyExpectation::Changes,
        "if let 不是语句开头的 let，后面的 helper_total() 指函数"
    );
}

/// 场景：`let` 之前有格式串捕获 `{helper_total}`。预期：改函数键变。
#[test]
fn format_capture_before_the_let_links_the_function() {
    assert_eq!(
        expectation_with_every_rule_intact(
            "fshadow::format_capture_before_the_let",
            EDIT_OF_THE_SAME_NAMED_FUNCTION
        ),
        KeyExpectation::Changes,
        "绑定之前捕获的名字指外面的"
    );
}

/// 场景（真仓 `let main = SamplingPoint::Main;` 的形状）：绑定之后名字写进名单里的 `format!` 的实参、闭包与格式串捕获。预期：改同名函数键不变。
#[test]
fn let_used_in_macro_arguments_closures_and_captures_shadows_the_function() {
    assert_eq!(
        expectation_with_every_rule_intact(
            "fshadow::shadowed_and_used_in_macro_arguments_closures_and_captures",
            EDIT_OF_THE_SAME_NAMED_FUNCTION
        ),
        KeyExpectation::StaysTheSame,
        "作用域里名单里的宏的调用实参、闭包、格式串捕获里的名字都是局部变量"
    );
}

/// 场景：`let` 之前读了同名字段 `record.helper_total`。预期：改同名函数键不变（字段不指函数）。
#[test]
fn field_of_the_same_name_before_the_let_does_not_link_the_function() {
    assert_eq!(
        expectation_with_every_rule_intact(
            "fshadow::reads_a_field_of_the_same_name_before_the_let",
            EDIT_OF_THE_SAME_NAMED_FUNCTION
        ),
        KeyExpectation::StaysTheSame,
        "单个点后面的是字段或方法"
    );
}

/// 场景：`let other_crate_total = 1;`，依赖的 crate 里有 `pub fn other_crate_total`。预期：改它键不变（跨目标的同样遮）。
#[test]
fn let_shadows_a_public_function_of_another_crate() {
    assert_eq!(
        expectation_with_every_rule_intact(
            "fshadow::shadows_a_function_of_another_crate",
            EDIT_OF_THE_FUNCTION_IN_THE_OTHER_CRATE
        ),
        KeyExpectation::StaysTheSame,
        "跨目标的同名函数同样被局部绑定遮住"
    );
}

/// 场景：没有局部绑定，裸名调依赖的 crate 里的 `pub fn other_crate_total`。预期：改它键变。
#[test]
fn call_without_any_binding_links_a_public_function_of_another_crate() {
    assert_eq!(
        expectation_with_every_rule_intact(
            "fshadow::calls_a_function_of_another_crate",
            EDIT_OF_THE_FUNCTION_IN_THE_OTHER_CRATE
        ),
        KeyExpectation::Changes,
        "跨目标按 pub 连"
    );
}

/// 场景：函数自己叫 `other_crate_total`、参数也叫这个，依赖的 crate 里有同名 `pub fn`。预期：改它键不变（自己的名字不算一处引用）。
#[test]
fn parameter_named_like_its_own_function_shadows_the_function_of_another_crate() {
    assert_eq!(
        expectation_with_every_rule_intact(
            "fshadow::same_name_as_a_parameter::other_crate_total",
            EDIT_OF_THE_FUNCTION_IN_THE_OTHER_CRATE
        ),
        KeyExpectation::StaysTheSame,
        "函数自己的名字不看，体里的都是参数"
    );
}

/// 场景：函数自己叫 `other_crate_total`、体里没有绑定，依赖的 crate 里有同名 `pub fn`。预期：改它键变（没有绑定就谈不上遮住）。
#[test]
fn function_named_like_a_function_of_another_crate_without_a_binding_links_it() {
    assert_eq!(
        expectation_with_every_rule_intact(
            "fshadow::same_name_without_a_binding::other_crate_total",
            EDIT_OF_THE_FUNCTION_IN_THE_OTHER_CRATE
        ),
        KeyExpectation::Changes,
        "至少要有一处绑定才算遮住"
    );
}
