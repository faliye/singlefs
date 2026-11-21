//! `implementations_and_uses` 的单测。
use super::super::tests::parse;
use super::*;

/// `use a::b as c;`：进当前模块可用名字集的只有别名 `c`，原名 `b` 不进（`b` 记进去的话，别处同名的条目会被误认成命中了这条 use）。
#[test]
fn use_with_an_alias_brings_only_the_alias_into_the_module_names() {
    let parsed = parse("use a::b as c;\n");
    let imports = parsed.units[0]
        .imports
        .as_ref()
        .expect("use 条目带导入清单");
    assert_eq!(imports.names, ["c"]);
    assert_eq!(imports.aliases, [("c".to_string(), "b".to_string())]);
}

#[test]
fn an_implementation_label_drops_the_generic_parameters_and_where_clause_and_spaces_every_token() {
    let parsed = parse(
        "impl<'flow, Device: core::fmt::Debug> std::ops::Deref for Flow<'flow, Device> where Device: Clone { type Target = u8; }\n\
             impl<Value> From<Vec<Value>> for Wrapper<Value> {}\n",
    );
    let labels: Vec<&str> = parsed
        .units
        .iter()
        .filter(|unit| unit.kind == UnitKind::ImplementationHead)
        .map(|unit| unit.name.as_str())
        .collect();
    assert_eq!(
        labels,
        [
            "<impl std::ops::Deref for Flow < 'flow , Device >>",
            "<impl From < Vec < Value > > for Wrapper < Value >>",
        ],
        "与 lib_function_units.py 的 impl 标签同一写法：去掉 impl 后的泛型参数表与 where，词法单元之间一个空格，:: 两边不留空格"
    );
}
