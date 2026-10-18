// mutation-tables 那一格的滞后表样本：表里两条的原文（main 里那个变量名）在这个文件里只许出现一次，注释里也不写它。
fn main() {
    let foo = 1;
    assert_eq!(std::mem::size_of::<u64>(), 8);
}
