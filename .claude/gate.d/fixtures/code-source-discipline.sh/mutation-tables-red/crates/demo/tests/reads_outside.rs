// 门禁 code-source-discipline 的判别力样本：运行期按 crate 根拼出一个逃出 crates/、又不在门禁 checker-tier-crates-mutation-replay 拷贝范围里的路径。
// checker-tier-crates-mutation-replay 每片只拷登记的路径，副本里没有 fixture-data/，点名这份测试的变异在没改坏的副本上就红。
#[test]
fn reads_a_file_outside_the_copy() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixture-data/answer.txt");
    assert!(path.exists());
}
