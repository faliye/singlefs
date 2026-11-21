// 样本：目录式测试目标 split_target，目标根 main.rs 往下有两条模块链：子模块 nested_cases（再往下一层 deeper）与经 #[path] 引的共用模块 common_split_shared；
// 子模块文件里的用例 libtest 全名带模块前缀，目标根文件里的用例没有。
mod nested_cases;
#[path = "../common_split_shared/mod.rs"]
mod common_split_shared;

#[test]
fn root_case() {
    assert_eq!(demo::published_generation(5), 5);
}
