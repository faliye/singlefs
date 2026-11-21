// 样本：#[path] 写成样本根的绝对路径（录好的 .d 里写 @ROOT@，假 cargo 换成这一趟的样本根；这里写的是录样本那一刻的根）。
#[path = "/sample-root/crates/provider/tests/c/mod.rs"]
mod c;

#[test]
fn shared_value_is_one() {
    assert_eq!(c::shared_value(), 1);
}
