// 样本：真标了 ignore 的用例，注释行夹在两条属性之间；点名它的那一行不带 --include-ignored，mutation-tables 照旧判红。
#[test]
// 标了 ignore：不带 --include-ignored 就不跑
#[ignore = "样本：标了 ignore 的用例"]
fn interleaved_slow_case() {
    assert_eq!(demo::published_generation(6), 6);
}
