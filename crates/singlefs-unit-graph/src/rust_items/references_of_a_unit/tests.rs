//! `references_of_a_unit` 的单测。
use super::super::tests::parse;

#[test]
fn format_string_captures_count_as_bare_names() {
    let parsed =
        parse("fn show() -> String { format!(\"{DATA_POINTER_BYTES} {{literal}} {width:>4}\") }\n");
    let names = &parsed.units[0].references.plain_names;
    assert!(names.contains(&"DATA_POINTER_BYTES".to_string()));
    assert!(names.contains(&"width".to_string()));
    assert!(!names.contains(&"literal".to_string()), "{{ 转义的不是捕获");
}

#[test]
fn capture_right_after_an_escaped_brace_still_counts_as_a_bare_name() {
    let parsed = parse("fn show() -> String { format!(\"{{{WRAPPED_WIDTH}}}\") }\n");
    assert!(
        parsed.units[0]
            .references
            .plain_names
            .contains(&"WRAPPED_WIDTH".to_string()),
        "{{{{{{名}}}}}} 是字面的花括号夹着一个捕获，捕获的名字要收"
    );
}
