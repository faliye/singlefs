// 样本：行内模块里的 #[path] 从 tests/common_shared/ 起算，比写在文件顶上多一层 ../。
// tests/common_shared/ 这个目录不在，rustc 照文件系统解 ..，读不到那一份：cargo check 编不过（scenario.json 里录了真 cargo 的报错）。
mod common_shared {
    #[path = "../../../provider/tests/c/mod.rs"]
    pub mod c;
}
use common_shared::c;

#[test]
fn shared_value_is_one() {
    assert_eq!(c::shared_value(), 1);
}
