//! blanket impl 怎么认（`records/2026-10-13-复用键与全仓改名.md`「crates 第三轮 A 交回」之后主 agent 定的第 1 条，定案 3「多连不少连」）：
//! trait impl 的自身类型里任何深度出现这个 impl 自己的一个泛型参数，就按 blanket 放行——trait 头一进闭包，它连同成员一起进；
//! 自身类型全具体的照旧等自身类型被点名。`impl<Element> InASlice for [Element]` 这类自身类型里一个能被点名的具体名字都没有，
//! 不放行就进不了任何闭包、键过窄（第三轮 A 报告「一、」的 `probe-slice` 实测）。
//! 夹具 `fselftypes`：每种自身类型一个 trait、一个 impl、一个只在泛型约束里写 trait 名的辅助函数和调它的入口（入口不点名自身类型，
//! 也不调 trait 的方法，闭包里有没有那个 impl 的方法体只看它算不算 blanket）；`fslice` 是第三轮 A 的 `probe-slice`。

mod common_fixture_workspace;

use common_fixture_workspace::{
    closure_with_triple, key_of, workspace_of, EntrySpecification, FixtureTree, HOST_TARGET_TRIPLE,
};
use singlefs_unit_graph::rule_switches::RuleSwitches;
use singlefs_unit_graph::units::UnitKind;

const SELF_TYPE_FIXTURE: [(&str, &str); 5] = [
    (
        "crates/fselftypes/Cargo.toml",
        include_str!("fixtures/blanket_self_types/fselftypes/Cargo.toml.fixture"),
    ),
    (
        "crates/fselftypes/src/lib.rs",
        include_str!("fixtures/blanket_self_types/fselftypes/src/lib.rs.fixture"),
    ),
    (
        "crates/fslice/Cargo.toml",
        include_str!("fixtures/blanket_self_types/fslice/Cargo.toml.fixture"),
    ),
    (
        "crates/fslice/src/lib.rs",
        include_str!("fixtures/blanket_self_types/fslice/src/lib.rs.fixture"),
    ),
    ("Cargo.lock", "# 夹具不解析依赖\n"),
];

const SLICE_FILE: &str = "crates/fslice/src/lib.rs";

/// 入口的闭包里，有没有名叫 `method_name`、挂在 impl 头下的方法体。
fn closure_holds_the_implementation_method(entry_path: &'static str, method_name: &str) -> bool {
    let switches = RuleSwitches::every_rule_intact();
    let tree = FixtureTree::materialize("blanket-self-types", &SELF_TYPE_FIXTURE);
    let workspace = workspace_of(tree.root(), &switches);
    let entry = EntrySpecification {
        entry_path,
        target: "lib",
        conditions: &[],
    };
    let outcome = closure_with_triple(&workspace, &entry, &switches, HOST_TARGET_TRIPLE)
        .unwrap_or_else(|error| panic!("{} 的闭包收得起来：{error}", entry.entry_path));
    assert!(
        outcome.refusals.is_empty(),
        "{} 的闭包不拒算：{:?}",
        entry.entry_path,
        outcome.refusals
    );
    outcome.units.iter().any(|unit_index| {
        let unit = &workspace.units[unit_index.0];
        unit.kind == UnitKind::Function
            && unit.name == method_name
            && unit.container.is_some_and(|container| {
                workspace.units[container.0].kind == UnitKind::ImplementationHead
            })
    })
}

#[test]
fn an_implementation_for_its_generic_parameter_itself_is_blanket() {
    assert!(
        closure_holds_the_implementation_method(
            "fselftypes::reaches_for_the_parameter",
            "for_the_parameter"
        ),
        "impl<Element> ForTheParameter for Element：trait 头进闭包，blanket impl 的方法体跟着进"
    );
}

#[test]
fn an_implementation_for_a_reference_to_its_generic_parameter_is_blanket() {
    assert!(
        closure_holds_the_implementation_method("fselftypes::reaches_by_reference", "by_reference"),
        "impl<Element> ByReference for &Element：自身类型里有泛型参数，算 blanket"
    );
}

#[test]
fn an_implementation_for_a_mutable_reference_to_its_generic_parameter_is_blanket() {
    assert!(
        closure_holds_the_implementation_method(
            "fselftypes::reaches_by_mutable_reference",
            "by_mutable_reference"
        ),
        "impl<Element> ByMutableReference for &mut Element：自身类型里有泛型参数，算 blanket"
    );
}

#[test]
fn an_implementation_for_a_box_of_its_generic_parameter_is_blanket() {
    assert!(
        closure_holds_the_implementation_method("fselftypes::reaches_in_a_box", "in_a_box"),
        "impl<Element> InABox for Box<Element>：泛型参数在尖括号里，算 blanket"
    );
}

#[test]
fn an_implementation_for_a_vector_of_its_generic_parameter_is_blanket() {
    assert!(
        closure_holds_the_implementation_method("fselftypes::reaches_in_a_vector", "in_a_vector"),
        "impl<Element> InAVector for Vec<Element>：泛型参数在尖括号里，算 blanket"
    );
}

#[test]
fn an_implementation_for_a_tuple_leading_with_its_generic_parameter_is_blanket() {
    assert!(
        closure_holds_the_implementation_method(
            "fselftypes::reaches_in_a_tuple_leading_with_the_parameter",
            "in_a_tuple_leading_with_the_parameter"
        ),
        "impl<Element> InATupleLeadingWithTheParameter for (Element, u32)：元组里有泛型参数，算 blanket"
    );
}

#[test]
fn an_implementation_for_a_tuple_trailing_with_its_generic_parameter_is_blanket() {
    assert!(
        closure_holds_the_implementation_method(
            "fselftypes::reaches_in_a_tuple_trailing_with_the_parameter",
            "in_a_tuple_trailing_with_the_parameter"
        ),
        "impl<Element> InATupleTrailingWithTheParameter for (u32, Element)：泛型参数不是第一个名字也算 blanket"
    );
}

#[test]
fn an_implementation_for_a_slice_of_its_generic_parameter_is_blanket() {
    assert!(
        closure_holds_the_implementation_method("fselftypes::reaches_in_a_slice", "in_a_slice"),
        "impl<Element> InASlice for [Element]：自身类型里没有具体名字，算 blanket"
    );
}

#[test]
fn an_implementation_for_an_array_of_its_generic_parameter_is_blanket() {
    assert!(
        closure_holds_the_implementation_method("fselftypes::reaches_in_an_array", "in_an_array"),
        "impl<Element, const LENGTH: usize> InAnArray for [Element; LENGTH]：算 blanket"
    );
}

#[test]
fn an_implementation_for_an_optional_box_of_its_generic_parameter_is_blanket() {
    assert!(
        closure_holds_the_implementation_method("fselftypes::reaches_in_an_optional_box", "in_an_optional_box"),
        "impl<Element> InAnOptionalBox for Option<Box<Element>>：泛型参数套在两层尖括号里也算 blanket"
    );
}

#[test]
fn an_implementation_for_a_concrete_type_nobody_names_stays_out_of_the_closure() {
    assert!(
        !closure_holds_the_implementation_method(
            "fselftypes::reaches_on_a_concrete_type",
            "on_a_concrete_type"
        ),
        "impl OnAConcreteType for Plain：自身类型全具体、Plain 没被点名，不算 blanket、不进闭包"
    );
}

#[test]
fn an_implementation_for_a_concrete_type_the_entry_names_is_in_the_closure() {
    assert!(
        closure_holds_the_implementation_method(
            "fselftypes::reaches_on_a_concrete_type_naming_it",
            "on_a_concrete_type"
        ),
        "impl OnAConcreteType for Plain：入口点名了 Plain，照被点名的类型放行"
    );
}

#[test]
fn editing_the_method_of_a_slice_implementation_changes_the_key_of_its_caller() {
    let switches = RuleSwitches::every_rule_intact();
    let entry = EntrySpecification {
        entry_path: "fslice::count",
        target: "lib",
        conditions: &[],
    };
    let base = FixtureTree::materialize("blanket-slice-base", &SELF_TYPE_FIXTURE);
    let edited = FixtureTree::materialize("blanket-slice-edited", &SELF_TYPE_FIXTURE);
    edited.replace_exactly_once(SLICE_FILE, "        5\n", "        6\n");
    let before =
        key_of(&workspace_of(base.root(), &switches), &entry, &switches).expect("改前不拒算");
    let after =
        key_of(&workspace_of(edited.root(), &switches), &entry, &switches).expect("改后不拒算");
    assert_ne!(
        before, after,
        "count 调 items.in_a_slice()，落到 impl<Element> InASlice for [Element] 的方法体：改它键变"
    );
}
