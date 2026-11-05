// 样本：这条测试上方的注释与属性行尾的注释里都写着 #[ignore]，它本身没标；mutation-tables 判 ignore 只看属性行去掉行尾注释之后的那一段，
// 点名它的那一行不带 --include-ignored 照绿。
/// 不标 `#[ignore]`：每一趟都跑。
#[test] // 这里也不是 #[ignore]
fn commented_ignore_case() {
    assert_eq!(demo::published_generation(5), 5);
}
