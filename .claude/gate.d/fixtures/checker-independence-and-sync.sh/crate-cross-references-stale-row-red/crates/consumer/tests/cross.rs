// 样本：只读同包的模块，放行表里那一行点名的引用这一趟没有。
#[path = "c/mod.rs"]
mod c;

#[test]
fn shared_value_is_one() {
    assert_eq!(c::shared_value(), 1);
}
