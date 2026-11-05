//! 门禁样本的差分测试：Shown::show 的规格（返回值等于参数）与真实现逐值比。

#[test]
fn shown_matches_its_argument() {
    for x in [0u64, 1, 7, u64::MAX] {
        assert_eq!(singlefs_replay_prefix::Shown.show(x), x);
    }
}
