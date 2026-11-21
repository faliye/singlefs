// 样本：split_target 经 #[path] 引的共用模块（libtest 全名 common_split_shared::…）。
#[test]
fn shared_case() {
    assert_eq!(demo::published_generation(9), 9);
}
