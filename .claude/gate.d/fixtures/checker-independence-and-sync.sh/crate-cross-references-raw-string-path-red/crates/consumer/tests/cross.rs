// 样本：#[path] 的路径写成原始字符串。
#[path = r"../../provider/tests/c/mod.rs"]
mod c;

#[test]
fn shared_value_is_one() {
    assert_eq!(c::shared_value(), 1);
}
