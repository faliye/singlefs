//! 规则格夹具（设计报告 2.3 的 13 格、4-1a 派发点名的 6 格新格、每条闭包规则补的 4 格、闭包内部的 dyn 按点名的类型连补的 1 格）：每格在一份新夹具上改一处，
//! 看入口的键变不变。设 `SINGLEFS_UNIT_GRAPH_BREAK=<开关>` 跑这一份，红在开关表（`src/rule_switches.rs` 文件头）点名的那几格；
//! 下面「弄坏开关」那一组用例逐个开关断言恰好红那几格。

mod common_fixture_workspace;

use common_fixture_workspace::{
    key_of, switches_from_the_environment, workspace_of, EntrySpecification, FixtureTree,
    RULE_CELL_FIXTURE,
};
use singlefs_unit_graph::rule_switches::{BrokenRule, RuleSwitches};

/// 改了之后键该怎样。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KeyExpectation {
    Changes,
    StaysTheSame,
}

/// 一格里看的一个入口。
#[derive(Clone, Copy, Debug)]
struct KeyCheck {
    entry: EntrySpecification,
    expectation: KeyExpectation,
}

/// 一格：改哪份文件的哪段原文、换成什么、看哪几个入口。
#[derive(Clone, Copy, Debug)]
struct FixtureCell {
    description: &'static str,
    file: &'static str,
    original: &'static str,
    replacement: &'static str,
    checks: &'static [KeyCheck],
}

const fn library_entry(entry_path: &'static str) -> EntrySpecification {
    EntrySpecification {
        entry_path,
        target: "lib",
        conditions: &[],
    }
}

const fn single_key_check(entry_path: &'static str, expectation: KeyExpectation) -> [KeyCheck; 1] {
    [KeyCheck {
        entry: library_entry(entry_path),
        expectation,
    }]
}

const DYN_DISPATCH_TO_A_DOWNSTREAM_IMPLEMENTATION: FixtureCell = FixtureCell {
    description: "dyn 分派到下游 crate 的 impl",
    file: "crates/fharness/src/lib.rs",
    original: "    fn read(&self) -> u32 {\n        7",
    replacement: "    fn read(&self) -> u32 {\n        70",
    checks: &single_key_check("ftier::judge_through_dyn", KeyExpectation::Changes),
};

const DYN_DISPATCH_TO_AN_IMPLEMENTATION_NOBODY_NAMES: FixtureCell = FixtureCell {
    description: "dyn 分派：没被点名的另一个 impl 不进闭包",
    file: "crates/fharness/src/lib.rs",
    original: "        8\n",
    replacement: "        80\n",
    checks: &[
        KeyCheck {
            entry: library_entry("ftier::judge_through_dyn"),
            expectation: KeyExpectation::StaysTheSame,
        },
        KeyCheck {
            entry: library_entry("ftier::judge_named_type_through_dyn"),
            expectation: KeyExpectation::StaysTheSame,
        },
    ],
};

const DYN_DISPATCH_TO_A_NAMED_TYPE_CONVERTED_INSIDE_THE_CLOSURE: FixtureCell = FixtureCell {
    description: "dyn 分派：闭包里点名的类型转成 dyn 之后只经 dyn 调用",
    file: "crates/fharness/src/lib.rs",
    original: "        9\n",
    replacement: "        90\n",
    checks: &single_key_check(
        "ftier::judge_named_type_through_dyn",
        KeyExpectation::Changes,
    ),
};

const GENERIC_BOUND_DISPATCH: FixtureCell = FixtureCell {
    description: "泛型约束分派",
    file: "crates/fharness/src/lib.rs",
    original: "    fn read(&self) -> u32 {\n        7",
    replacement: "    fn read(&self) -> u32 {\n        70",
    checks: &single_key_check("ftier::judge_through_generic", KeyExpectation::Changes),
};

const TRAIT_DEFAULT_METHOD: FixtureCell = FixtureCell {
    description: "trait 默认方法",
    file: "crates/fcore/src/lib.rs",
    original: "self.read() * 2",
    replacement: "self.read() * 3",
    checks: &single_key_check("ftier::judge_through_generic", KeyExpectation::Changes),
};

const OPERATOR_OVERLOADING: FixtureCell = FixtureCell {
    description: "运算符重载",
    file: "crates/fcore/src/lib.rs",
    original: "Money(self.0 + other.0)",
    replacement: "Money(self.0 * other.0)",
    checks: &single_key_check("ftier::judge_operator", KeyExpectation::Changes),
};

const IMPLICIT_DROP: FixtureCell = FixtureCell {
    description: "Drop 隐式调用",
    file: "crates/fcore/src/lib.rs",
    original: "let _ = 1;",
    replacement: "let _ = 2;",
    checks: &single_key_check("ftier::judge_drop", KeyExpectation::Changes),
};

const FUNCTION_FROM_A_DECLARATIVE_MACRO: FixtureCell = FixtureCell {
    description: "声明宏展开出的函数",
    file: "crates/fcore/src/lib.rs",
    original: "        pub fn $name() -> u32 {\n            5",
    replacement: "        pub fn $name() -> u32 {\n            6",
    checks: &single_key_check("ftier::judge_macro", KeyExpectation::Changes),
};

const ASSOCIATED_CONSTANT_THROUGH_A_GENERIC: FixtureCell = FixtureCell {
    description: "关联常量（经泛型）",
    file: "crates/fharness/src/lib.rs",
    original: "const LIMIT: u32 = 10;",
    replacement: "const LIMIT: u32 = 11;",
    checks: &single_key_check("ftier::judge_associated_constant", KeyExpectation::Changes),
};

const FUNCTION_POINTER_TABLE: FixtureCell = FixtureCell {
    description: "函数指针表",
    file: "crates/fcore/src/lib.rs",
    original: "pub fn helper() -> u32 {\n    3",
    replacement: "pub fn helper() -> u32 {\n    4",
    checks: &single_key_check("ftier::judge_table", KeyExpectation::Changes),
};

const OPEN_ENTRY_TAKING_A_DYN_PARAMETER: FixtureCell = FixtureCell {
    description: "开放入口：参数是 dyn",
    file: "crates/fharness/src/lib.rs",
    original: "    fn read(&self) -> u32 {\n        7",
    replacement: "    fn read(&self) -> u32 {\n        70",
    checks: &single_key_check("ftier::judge_open", KeyExpectation::Changes),
};

const OPEN_ENTRY_WITH_A_GENERIC_TRAIT_BOUND: FixtureCell = FixtureCell {
    description: "开放入口：参数是泛型约束",
    file: "crates/fharness/src/lib.rs",
    original: "    fn read(&self) -> u32 {\n        7",
    replacement: "    fn read(&self) -> u32 {\n        70",
    checks: &single_key_check("ftier::judge_open_generic", KeyExpectation::Changes),
};

const CONSTANT_IN_AN_ARRAY_LENGTH: FixtureCell = FixtureCell {
    description: "数组长度里的常量",
    file: "crates/fcore/src/lib.rs",
    original: "pub const WIDTH: usize = 4;",
    replacement: "pub const WIDTH: usize = 5;",
    checks: &single_key_check("ftier::judge_array_length", KeyExpectation::Changes),
};

const UNREACHABLE_FUNCTION: FixtureCell = FixtureCell {
    description: "阴性：够不到的函数",
    file: "crates/fcore/src/lib.rs",
    original: "    99\n",
    replacement: "    98\n",
    checks: &single_key_check("ftier::judge_macro", KeyExpectation::StaysTheSame),
};

const COMMENT_ONLY: FixtureCell = FixtureCell {
    description: "阴性：只改注释",
    file: "crates/fcore/src/lib.rs",
    original: "        Money(self.0 + other.0)",
    replacement: "        // 只加注释\n        Money(self.0 + other.0)",
    checks: &single_key_check("ftier::judge_operator", KeyExpectation::StaysTheSame),
};

const ENUMERATION_FROM_A_PATH_QUALIFIED_ITEM_MACRO: FixtureCell = FixtureCell {
    description: "带路径的条目级宏调用定义了枚举",
    file: "crates/fpainter/src/lib.rs",
    original: "        Green,\n",
    replacement: "        Green,\n        Blue,\n",
    checks: &single_key_check("fpainter::judge_color", KeyExpectation::Changes),
};

const VERUS_ENSURES_WITH_BRACES: FixtureCell = FixtureCell {
    description: "verus! 里 ensures 带花括号",
    file: "crates/fverified/src/lib.rs",
    original: "{\n    value\n}",
    replacement: "{\n    value + 0\n}",
    checks: &single_key_check("fverified::judge_verus", KeyExpectation::Changes),
};

const UNIT_UNDER_A_FEATURE_THAT_IS_OFF: FixtureCell = FixtureCell {
    description: "关着的特性下的单元改了",
    file: "crates/fcore/src/lib.rs",
    original: "#[cfg(feature = \"wide\")]\npub const LANES: u32 = 8;",
    replacement: "#[cfg(feature = \"wide\")]\npub const LANES: u32 = 9;",
    checks: &single_key_check("ftier::judge_features", KeyExpectation::StaysTheSame),
};

const UNIT_UNDER_A_FEATURE_THAT_IS_ON: FixtureCell = FixtureCell {
    description: "开着的特性下的单元改了",
    file: "crates/fcore/src/lib.rs",
    original: "#[cfg(feature = \"checked\")]\npub fn checked_lanes() -> u32 {\n    LANES\n}",
    replacement: "#[cfg(feature = \"checked\")]\npub fn checked_lanes() -> u32 {\n    LANES * 2\n}",
    checks: &single_key_check("ftier::judge_features", KeyExpectation::Changes),
};

const DOCUMENTATION_COMMENT_ONLY: FixtureCell = FixtureCell {
    description: "只改文档注释",
    file: "crates/fcore/src/lib.rs",
    original: "/// 文档注释：只改这一行不该作废。",
    replacement: "/// 文档注释：换了一句话，键照旧。",
    checks: &single_key_check("ftier::judge_documented", KeyExpectation::StaysTheSame),
};

const DOCUMENTATION_TEST_CODE: FixtureCell = FixtureCell {
    description: "改文档注释里的文档测试代码",
    file: "crates/fcore/src/lib.rs",
    original: "/// assert_eq!(fcore::with_example(), 2);",
    replacement: "/// assert_eq!(fcore::with_example(), 2 + 0);",
    checks: &[
        KeyCheck {
            entry: library_entry("ftier::judge_with_example"),
            expectation: KeyExpectation::StaysTheSame,
        },
        KeyCheck {
            entry: EntrySpecification {
                entry_path: "fcore::with_example",
                target: "doctest",
                conditions: &[],
            },
            expectation: KeyExpectation::Changes,
        },
    ],
};

const FILE_READ_THROUGH_A_MANIFEST_DIRECTORY_CONCATENATION: FixtureCell = FixtureCell {
    description: "include_str 读的文件改了",
    file: "crates/fcore/data/table.txt",
    original: "第二行数据",
    replacement: "第二行数据改过",
    checks: &single_key_check("ftier::judge_table_text", KeyExpectation::Changes),
};

const ASSOCIATED_FUNCTION_THROUGH_A_TYPE_PATH: FixtureCell = FixtureCell {
    description: "经类型路径调关联函数",
    file: "crates/fcore/src/lib.rs",
    original: "        Money(0)",
    replacement: "        Money(10)",
    checks: &single_key_check("ftier::judge_associated_function", KeyExpectation::Changes),
};

const INHERENT_METHOD_THROUGH_A_DOT_CALL: FixtureCell = FixtureCell {
    description: "经点调固有方法",
    file: "crates/fcore/src/lib.rs",
    original: "self.0 * 3",
    replacement: "self.0 * 4",
    checks: &single_key_check("ftier::judge_inherent_method", KeyExpectation::Changes),
};

const CALL_THROUGH_A_RENAMED_IMPORT: FixtureCell = FixtureCell {
    description: "use 改名之后调用",
    file: "crates/fcore/src/lib.rs",
    original: "    12\n",
    replacement: "    13\n",
    checks: &single_key_check("ftier::judge_renamed_import", KeyExpectation::Changes),
};

const SAME_NAME_IN_A_CRATE_OUTSIDE_THE_LINK: FixtureCell = FixtureCell {
    description: "没链接的 crate 里同名函数",
    file: "crates/funlinked/src/lib.rs",
    original: "    34\n",
    replacement: "    35\n",
    checks: &single_key_check("ftier::judge_shared_name", KeyExpectation::StaysTheSame),
};

const EVERY_CELL: [FixtureCell; 26] = [
    DYN_DISPATCH_TO_A_DOWNSTREAM_IMPLEMENTATION,
    DYN_DISPATCH_TO_AN_IMPLEMENTATION_NOBODY_NAMES,
    DYN_DISPATCH_TO_A_NAMED_TYPE_CONVERTED_INSIDE_THE_CLOSURE,
    GENERIC_BOUND_DISPATCH,
    TRAIT_DEFAULT_METHOD,
    OPERATOR_OVERLOADING,
    IMPLICIT_DROP,
    FUNCTION_FROM_A_DECLARATIVE_MACRO,
    ASSOCIATED_CONSTANT_THROUGH_A_GENERIC,
    FUNCTION_POINTER_TABLE,
    OPEN_ENTRY_TAKING_A_DYN_PARAMETER,
    OPEN_ENTRY_WITH_A_GENERIC_TRAIT_BOUND,
    CONSTANT_IN_AN_ARRAY_LENGTH,
    UNREACHABLE_FUNCTION,
    COMMENT_ONLY,
    ENUMERATION_FROM_A_PATH_QUALIFIED_ITEM_MACRO,
    VERUS_ENSURES_WITH_BRACES,
    UNIT_UNDER_A_FEATURE_THAT_IS_OFF,
    UNIT_UNDER_A_FEATURE_THAT_IS_ON,
    DOCUMENTATION_COMMENT_ONLY,
    DOCUMENTATION_TEST_CODE,
    FILE_READ_THROUGH_A_MANIFEST_DIRECTORY_CONCATENATION,
    ASSOCIATED_FUNCTION_THROUGH_A_TYPE_PATH,
    INHERENT_METHOD_THROUGH_A_DOT_CALL,
    CALL_THROUGH_A_RENAMED_IMPORT,
    SAME_NAME_IN_A_CRATE_OUTSIDE_THE_LINK,
];

/// 一格在这组开关下对不对：对交 `Ok`，不对交说明（哪个入口期望什么、得了什么）。
fn cell_verdict(cell: &FixtureCell, switches: &RuleSwitches) -> Result<(), String> {
    let base = FixtureTree::materialize("base", &RULE_CELL_FIXTURE);
    let base_workspace = workspace_of(base.root(), switches);
    let variant = FixtureTree::materialize("variant", &RULE_CELL_FIXTURE);
    variant.replace_exactly_once(cell.file, cell.original, cell.replacement);
    let variant_workspace = workspace_of(variant.root(), switches);
    for check in cell.checks {
        let before = key_of(&base_workspace, &check.entry, switches)?;
        let after = key_of(&variant_workspace, &check.entry, switches)?;
        let got = if before == after {
            KeyExpectation::StaysTheSame
        } else {
            KeyExpectation::Changes
        };
        if got != check.expectation {
            return Err(format!(
                "「{}」入口 {}（{}）期望 {:?}，得 {:?}",
                cell.description,
                check.entry.entry_path,
                check.entry.target,
                check.expectation,
                got
            ));
        }
    }
    Ok(())
}

fn assert_cell_holds(cell: &FixtureCell) {
    let switches = switches_from_the_environment();
    if let Err(reason) = cell_verdict(cell, &switches) {
        panic!("{reason}");
    }
}

/// 弄坏一条规则之后红的那几格（按说明排好）。
fn red_cells_with(rule: BrokenRule) -> Vec<&'static str> {
    let switches = RuleSwitches::with_broken(&[rule]);
    let mut red: Vec<&'static str> = EVERY_CELL
        .iter()
        .filter(|cell| cell_verdict(cell, &switches).is_err())
        .map(|cell| cell.description)
        .collect();
    red.sort_unstable();
    red
}

fn assert_switch_turns_exactly(rule: BrokenRule, expected: &[&'static str]) {
    let mut expected_sorted = expected.to_vec();
    expected_sorted.sort_unstable();
    assert_eq!(
        red_cells_with(rule),
        expected_sorted,
        "弄坏 {} 之后红的格应当恰好是开关表点名的那几格",
        rule.switch_name()
    );
}

#[test]
fn dyn_dispatch_reaches_an_implementation_in_a_downstream_crate() {
    assert_cell_holds(&DYN_DISPATCH_TO_A_DOWNSTREAM_IMPLEMENTATION);
}

#[test]
fn dyn_dispatch_does_not_reach_an_implementation_nobody_names() {
    assert_cell_holds(&DYN_DISPATCH_TO_AN_IMPLEMENTATION_NOBODY_NAMES);
}

#[test]
fn dyn_dispatch_reaches_the_implementation_of_a_type_named_and_converted_inside_the_closure() {
    assert_cell_holds(&DYN_DISPATCH_TO_A_NAMED_TYPE_CONVERTED_INSIDE_THE_CLOSURE);
}

#[test]
fn a_generic_bound_reaches_the_implementation_of_the_named_type() {
    assert_cell_holds(&GENERIC_BOUND_DISPATCH);
}

#[test]
fn a_trait_default_method_body_is_in_the_closure() {
    assert_cell_holds(&TRAIT_DEFAULT_METHOD);
}

#[test]
fn an_overloaded_operator_body_is_in_the_closure() {
    assert_cell_holds(&OPERATOR_OVERLOADING);
}

#[test]
fn an_implicit_drop_body_is_in_the_closure() {
    assert_cell_holds(&IMPLICIT_DROP);
}

#[test]
fn a_function_generated_by_a_declarative_macro_is_in_the_closure() {
    assert_cell_holds(&FUNCTION_FROM_A_DECLARATIVE_MACRO);
}

#[test]
fn an_associated_constant_reached_through_a_generic_is_in_the_closure() {
    assert_cell_holds(&ASSOCIATED_CONSTANT_THROUGH_A_GENERIC);
}

#[test]
fn a_function_behind_a_function_pointer_table_is_in_the_closure() {
    assert_cell_holds(&FUNCTION_POINTER_TABLE);
}

#[test]
fn an_open_entry_taking_a_dyn_parameter_reaches_every_implementation() {
    assert_cell_holds(&OPEN_ENTRY_TAKING_A_DYN_PARAMETER);
}

#[test]
fn an_open_entry_with_a_generic_trait_bound_reaches_every_implementation() {
    assert_cell_holds(&OPEN_ENTRY_WITH_A_GENERIC_TRAIT_BOUND);
}

#[test]
fn a_constant_used_as_an_array_length_is_in_the_closure() {
    assert_cell_holds(&CONSTANT_IN_AN_ARRAY_LENGTH);
}

#[test]
fn an_unreachable_function_keeps_the_key() {
    assert_cell_holds(&UNREACHABLE_FUNCTION);
}

#[test]
fn a_comment_only_change_keeps_the_key() {
    assert_cell_holds(&COMMENT_ONLY);
}

#[test]
fn an_enumeration_defined_by_a_path_qualified_item_macro_is_in_the_closure() {
    assert_cell_holds(&ENUMERATION_FROM_A_PATH_QUALIFIED_ITEM_MACRO);
}

#[test]
fn a_verus_function_whose_ensures_clause_has_braces_is_one_unit() {
    assert_cell_holds(&VERUS_ENSURES_WITH_BRACES);
}

#[test]
fn a_unit_under_a_feature_that_is_off_keeps_the_key() {
    assert_cell_holds(&UNIT_UNDER_A_FEATURE_THAT_IS_OFF);
}

#[test]
fn a_unit_under_a_feature_that_is_on_changes_the_key() {
    assert_cell_holds(&UNIT_UNDER_A_FEATURE_THAT_IS_ON);
}

#[test]
fn a_documentation_comment_only_change_keeps_the_key() {
    assert_cell_holds(&DOCUMENTATION_COMMENT_ONLY);
}

#[test]
fn changing_documentation_test_code_only_changes_the_documentation_test_entry() {
    assert_cell_holds(&DOCUMENTATION_TEST_CODE);
}

#[test]
fn a_file_read_through_a_manifest_directory_concatenation_changes_the_key() {
    assert_cell_holds(&FILE_READ_THROUGH_A_MANIFEST_DIRECTORY_CONCATENATION);
}

#[test]
fn an_associated_function_called_through_a_type_path_is_in_the_closure() {
    assert_cell_holds(&ASSOCIATED_FUNCTION_THROUGH_A_TYPE_PATH);
}

#[test]
fn an_inherent_method_called_with_a_dot_is_in_the_closure() {
    assert_cell_holds(&INHERENT_METHOD_THROUGH_A_DOT_CALL);
}

#[test]
fn a_function_called_through_a_renamed_import_is_in_the_closure() {
    assert_cell_holds(&CALL_THROUGH_A_RENAMED_IMPORT);
}

#[test]
fn the_same_name_in_a_crate_outside_the_link_keeps_the_key() {
    assert_cell_holds(&SAME_NAME_IN_A_CRATE_OUTSIDE_THE_LINK);
}

#[test]
fn breaking_target_scope_reds_only_the_unlinked_same_name_cell() {
    assert_switch_turns_exactly(
        BrokenRule::TargetScope,
        &[SAME_NAME_IN_A_CRATE_OUTSIDE_THE_LINK.description],
    );
}

#[test]
fn breaking_bare_names_reds_every_cell_that_is_reached_only_by_a_bare_name() {
    assert_switch_turns_exactly(
        BrokenRule::BareNames,
        &[
            DYN_DISPATCH_TO_A_DOWNSTREAM_IMPLEMENTATION.description,
            DYN_DISPATCH_TO_A_NAMED_TYPE_CONVERTED_INSIDE_THE_CLOSURE.description,
            GENERIC_BOUND_DISPATCH.description,
            TRAIT_DEFAULT_METHOD.description,
            OPERATOR_OVERLOADING.description,
            IMPLICIT_DROP.description,
            FUNCTION_FROM_A_DECLARATIVE_MACRO.description,
            ASSOCIATED_CONSTANT_THROUGH_A_GENERIC.description,
            FUNCTION_POINTER_TABLE.description,
            CONSTANT_IN_AN_ARRAY_LENGTH.description,
            ENUMERATION_FROM_A_PATH_QUALIFIED_ITEM_MACRO.description,
            VERUS_ENSURES_WITH_BRACES.description,
            UNIT_UNDER_A_FEATURE_THAT_IS_ON.description,
            FILE_READ_THROUGH_A_MANIFEST_DIRECTORY_CONCATENATION.description,
            INHERENT_METHOD_THROUGH_A_DOT_CALL.description,
            CALL_THROUGH_A_RENAMED_IMPORT.description,
        ],
    );
}

#[test]
fn breaking_qualified_paths_reds_only_the_associated_function_cell() {
    assert_switch_turns_exactly(
        BrokenRule::QualifiedPaths,
        &[ASSOCIATED_FUNCTION_THROUGH_A_TYPE_PATH.description],
    );
}

#[test]
fn breaking_method_calls_reds_the_default_method_and_inherent_method_cells() {
    assert_switch_turns_exactly(
        BrokenRule::MethodCalls,
        &[
            TRAIT_DEFAULT_METHOD.description,
            INHERENT_METHOD_THROUGH_A_DOT_CALL.description,
        ],
    );
}

#[test]
fn breaking_trait_implementation_edges_reds_the_operator_drop_and_associated_constant_cells() {
    assert_switch_turns_exactly(
        BrokenRule::TraitImplementationEdges,
        &[
            OPERATOR_OVERLOADING.description,
            IMPLICIT_DROP.description,
            ASSOCIATED_CONSTANT_THROUGH_A_GENERIC.description,
        ],
    );
}

#[test]
fn breaking_the_class_hierarchy_reds_only_the_two_open_entry_cells() {
    assert_switch_turns_exactly(
        BrokenRule::ClassHierarchy,
        &[
            OPEN_ENTRY_TAKING_A_DYN_PARAMETER.description,
            OPEN_ENTRY_WITH_A_GENERIC_TRAIT_BOUND.description,
        ],
    );
}

#[test]
fn opening_every_implementation_for_dyn_traits_inside_the_closure_reds_only_the_unnamed_implementation_cell(
) {
    assert_switch_turns_exactly(
        BrokenRule::DynTraitsOpenEveryImplementation,
        &[DYN_DISPATCH_TO_AN_IMPLEMENTATION_NOBODY_NAMES.description],
    );
}

#[test]
fn breaking_structural_edges_reds_only_the_renamed_import_cell() {
    assert_switch_turns_exactly(
        BrokenRule::StructuralEdges,
        &[CALL_THROUGH_A_RENAMED_IMPORT.description],
    );
}

#[test]
fn breaking_macro_units_reds_the_declarative_macro_and_path_qualified_macro_cells() {
    assert_switch_turns_exactly(
        BrokenRule::MacroUnits,
        &[
            FUNCTION_FROM_A_DECLARATIVE_MACRO.description,
            ENUMERATION_FROM_A_PATH_QUALIFIED_ITEM_MACRO.description,
        ],
    );
}

#[test]
fn breaking_constant_and_static_units_reds_the_associated_constant_table_and_array_length_cells() {
    assert_switch_turns_exactly(
        BrokenRule::ConstantAndStaticUnits,
        &[
            ASSOCIATED_CONSTANT_THROUGH_A_GENERIC.description,
            FUNCTION_POINTER_TABLE.description,
            CONSTANT_IN_AN_ARRAY_LENGTH.description,
        ],
    );
}

#[test]
fn breaking_conditional_compilation_reds_only_the_feature_off_cell() {
    assert_switch_turns_exactly(
        BrokenRule::ConditionalCompilation,
        &[UNIT_UNDER_A_FEATURE_THAT_IS_OFF.description],
    );
}

#[test]
fn breaking_documentation_comments_reds_the_two_documentation_cells() {
    assert_switch_turns_exactly(
        BrokenRule::DocumentationCommentsInFingerprint,
        &[
            DOCUMENTATION_COMMENT_ONLY.description,
            DOCUMENTATION_TEST_CODE.description,
        ],
    );
}

#[test]
fn breaking_documentation_test_units_reds_only_the_documentation_test_cell() {
    assert_switch_turns_exactly(
        BrokenRule::DocumentationTestUnits,
        &[DOCUMENTATION_TEST_CODE.description],
    );
}

#[test]
fn breaking_data_units_reds_only_the_file_read_cell() {
    assert_switch_turns_exactly(
        BrokenRule::DataUnits,
        &[FILE_READ_THROUGH_A_MANIFEST_DIRECTORY_CONCATENATION.description],
    );
}

#[test]
fn breaking_path_macro_invocations_reds_only_the_path_qualified_macro_cell() {
    assert_switch_turns_exactly(
        BrokenRule::PathMacroInvocations,
        &[ENUMERATION_FROM_A_PATH_QUALIFIED_ITEM_MACRO.description],
    );
}

#[test]
fn breaking_verus_clause_braces_reds_only_the_verus_cell() {
    assert_switch_turns_exactly(
        BrokenRule::VerusClauseBraces,
        &[VERUS_ENSURES_WITH_BRACES.description],
    );
}

#[test]
fn breaking_the_wgsl_external_name_check_reds_no_rust_cell() {
    assert_switch_turns_exactly(BrokenRule::WgslExternalNames, &[]);
}

#[test]
fn breaking_the_wgsl_external_name_prefixes_reds_no_rust_cell() {
    assert_switch_turns_exactly(BrokenRule::WgslExternalNamePrefixes, &[]);
}

#[test]
fn breaking_the_wgsl_supplier_edges_reds_no_rust_cell() {
    assert_switch_turns_exactly(BrokenRule::WgslSupplierEdges, &[]);
}

/// 切分按文件切片并行：一个线程与四个线程切同一份夹具，dump 逐行相同。
#[test]
fn dumping_the_fixture_with_one_thread_and_with_four_threads_gives_the_same_lines() {
    let tree = FixtureTree::materialize("threads", &RULE_CELL_FIXTURE);
    let switches = RuleSwitches::every_rule_intact();
    let single = singlefs_unit_graph::load_workspace(
        tree.root(),
        std::num::NonZeroUsize::new(1).expect("1 是正整数"),
        &switches,
    )
    .expect("夹具装得起来");
    let several = singlefs_unit_graph::load_workspace(
        tree.root(),
        std::num::NonZeroUsize::new(4).expect("4 是正整数"),
        &switches,
    )
    .expect("夹具装得起来");
    let single_lines = singlefs_unit_graph::output::dump_lines(&single);
    assert!(
        single_lines.len() > 40,
        "夹具切出的单元不该这么少：{}",
        single_lines.len()
    );
    assert_eq!(
        single_lines,
        singlefs_unit_graph::output::dump_lines(&several)
    );
}
