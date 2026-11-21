// 样本：引用放进点起头的目录里。
#[path = ".shared/mod.rs"]
mod shared;
use shared::c;

#[test]
fn shared_value_is_one() {
    assert_eq!(c::shared_value(), 1);
}
