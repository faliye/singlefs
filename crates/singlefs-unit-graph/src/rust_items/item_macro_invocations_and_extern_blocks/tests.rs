//! `item_macro_invocations_and_extern_blocks` 的单测。
use super::super::tests::parse;
use super::*;

#[test]
fn path_qualified_item_macro_invocation_is_one_unit_that_declares_its_names() {
    let parsed = parse("singlefs_verification::enum_listing_every_member! {\n    #[derive(Debug)]\n    pub enum Color { Red, Green }\n}\n");
    assert!(
        parsed.file_level_refusals.is_empty(),
        "带路径的条目级宏调用认得：{:?}",
        parsed.file_level_refusals
    );
    assert_eq!(parsed.units.len(), 1);
    assert_eq!(parsed.units[0].kind, UnitKind::ItemMacroInvocation);
    assert_eq!(
        parsed.units[0].declared_names,
        ["Color", "Debug", "Green", "Red", "derive"],
        "宏调用里的标识符都算可能定义的名字（属性里的也算），只去掉关键字 pub、enum"
    );
}

#[test]
fn an_item_macro_invocation_declares_identifiers_after_a_dot_and_primitive_type_names_but_no_keywords(
) {
    let parsed = parse(
        "make_items! {
    fn widen(value: u8) -> u64 { value.into() }
}
",
    );
    assert_eq!(parsed.units.len(), 1);
    assert_eq!(
        parsed.units[0].declared_names,
        ["into", "u64", "u8", "value", "widen"],
        "fn 是关键字、不算；`.` 后面的 into 与基本类型名 u8、u64 都算"
    );
}
