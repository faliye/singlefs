// 样本：消费包的测试目标照字面 #[path] 引提供包 tests/ 下的源码；构建脚本也报了这一份的 rerun-if-changed，前缀行放不过它。
#[path = "../../provider/tests/c/mod.rs"]
mod c;

#[test]
fn shared_value_is_one() {
    assert_eq!(c::shared_value(), 1);
}
