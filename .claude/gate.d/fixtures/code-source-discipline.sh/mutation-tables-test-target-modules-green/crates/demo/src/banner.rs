// 串里写着 `mod tests {` 与一个落单的 {，字符字面量里也有一个 {：配花括号时不抹掉它们，
// 下面那条测试就被算成套在 tests 模块里（banner::tests::…），按 --exact 点名的完整名对不上。
pub fn banner() -> &'static str {
    "mod tests { {"
}

pub fn opener() -> char {
    '{'
}

#[test]
fn banner_length_counts_the_braces() {
    assert_eq!(banner().len(), 13);
    assert_eq!(opener(), '{');
}
