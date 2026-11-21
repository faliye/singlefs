// 样本：#[path] 套在条件恒真的 cfg_attr 里。
#[cfg_attr(all(), path = "../../provider/tests/c/mod.rs")]
mod c;

#[test]
fn shared_value_is_one() {
    assert_eq!(c::shared_value(), 1);
}
