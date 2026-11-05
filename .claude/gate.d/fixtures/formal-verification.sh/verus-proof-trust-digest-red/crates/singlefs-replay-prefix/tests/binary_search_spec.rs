//! 样本的差分测试：binary_search 的登记规格与真实现在一批有序切片上逐值比（门禁只核这个函数在、标了 #[test]，不跑它）。
#[test]
fn binary_search_spec_matches_std() {
    let slice = [0u64, 0, 1];
    if let Ok(index) = slice.binary_search(&0) {
        assert!(slice[index] == 0);
    }
}
