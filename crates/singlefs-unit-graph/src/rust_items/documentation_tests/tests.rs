//! `documentation_tests` 的单测。
use super::super::tests::parse;
use super::*;

#[test]
fn documentation_comments_stay_out_of_the_fingerprint_and_code_blocks_become_documentation_tests() {
    let with_documentation = parse("/// 说明\n/// ```\n/// assert_eq!(crate_name::value(), 1);\n/// ```\npub fn value() -> u32 { 1 }\n");
    let without_documentation = parse("pub fn value() -> u32 { 1 }\n");
    assert_eq!(
        with_documentation.units[0].fingerprint, without_documentation.units[0].fingerprint,
        "文档注释不进条目指纹"
    );
    assert_eq!(
        with_documentation.units.len(),
        2,
        "函数一个单元，文档里的代码块一个文档测试单元"
    );
    assert_eq!(
        with_documentation.units[1].kind,
        UnitKind::DocumentationTest
    );
    assert_eq!(
        with_documentation.units[1].documentation_test_of,
        Some(DocumentationTestAttachment::Item(UnitIndex(0)))
    );
}
