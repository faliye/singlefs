// research-unit-tests-red-records-green 样本的集成测试二进制：判绿，只在 cargo test 带 --no-fail-fast 时跑得到
#[test]
fn adds_in_its_own_binary() {
    assert_eq!(tiny::add(1, 2), 3);
}
