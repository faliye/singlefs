//! `verus_items` 的单测。
use super::super::tests::{kinds_and_names, parse};
use super::*;

#[test]
fn verus_ensures_clause_with_braces_stays_inside_its_function_unit() {
    let parsed = parse(
        "verus! {\npub fn chosen(value: u64) -> (result: u64)\n    ensures if value > 0 { result == value } else { result == 0 },\n{\n    value\n}\n\
             pub open spec fn never_called() -> bool { true }\n}\n",
    );
    assert!(
        parsed.file_level_refusals.is_empty(),
        "verus! 里 ensures 带花括号也认得：{:?}",
        parsed.file_level_refusals
    );
    assert_eq!(
        kinds_and_names(&parsed),
        [
            (UnitKind::Function, "chosen".to_string()),
            (UnitKind::Function, "never_called".to_string()),
        ]
    );
    assert_eq!(
        parsed.units[0].end_line, 6,
        "chosen 这个单元到它的函数体闭上为止"
    );
}
