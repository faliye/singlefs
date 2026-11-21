// 样本：经一份不叫 .rs 的中转文件再 #[path] 引过去。
#[path = "shim.inc"]
mod shim;
use shim::c;

#[test]
fn shared_value_is_one() {
    assert_eq!(c::shared_value(), 1);
}
