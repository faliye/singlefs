// 样本：一条测试由宏展开出来（调用处只有标识符，没有 fn 定义），一条照常写；mutation-tables 格认得出两条都在 --test generated 里
macro_rules! generated_case {
    ($name:ident) => {
        #[test]
        fn $name() {
            assert_eq!(demo::published_generation(2), 2);
        }
    };
}

generated_case!(generated_by_a_macro_case);

#[test]
fn published_generation_is_the_transaction_generation() {
    assert_eq!(demo::published_generation(4), 4);
}
