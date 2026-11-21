// 样本：引用放进名叫 target 的模块目录里。
mod target;
use target::c;

#[test]
fn shared_value_is_one() {
    assert_eq!(c::shared_value(), 1);
}
