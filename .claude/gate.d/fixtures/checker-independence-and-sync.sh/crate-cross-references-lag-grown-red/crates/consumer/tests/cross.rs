// 样本：照字面 #[path] 引提供包的源码，这一处登在滞后表里，而滞后表比基准那一版多出这一行。
#[path = "../../provider/tests/c/mod.rs"]
mod c;

#[test]
fn shared_value_is_one() {
    assert_eq!(c::shared_value(), 1);
}
