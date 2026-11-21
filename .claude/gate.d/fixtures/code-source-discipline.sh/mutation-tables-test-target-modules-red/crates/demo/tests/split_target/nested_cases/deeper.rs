// 样本：第二层子模块（libtest 全名 nested_cases::deeper::…）。
#[test]
fn deeper_case() {
    assert_eq!(demo::published_generation(7), 7);
    assert_eq!(demo::published_generation(8), 8);
}
