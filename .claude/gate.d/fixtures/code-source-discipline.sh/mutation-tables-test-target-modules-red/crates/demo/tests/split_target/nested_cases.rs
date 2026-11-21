// 样本：split_target 的子模块（libtest 全名 nested_cases::…），自己再往下引一层 deeper，deeper 的文件在 nested_cases/ 下。
mod deeper;

#[test]
fn nested_case() {
    assert_eq!(demo::published_generation(6), 6);
}
