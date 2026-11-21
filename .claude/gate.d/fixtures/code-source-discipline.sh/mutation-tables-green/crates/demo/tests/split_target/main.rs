// 样本：目录式测试目标，共用模块经 #[path] 引自 tests/common_split/，点名的测试写在那份共用模块里
#[path = "../common_split/mod.rs"]
mod common_split;

#[test]
fn split_target_runs() {
    common_split::marker();
}
