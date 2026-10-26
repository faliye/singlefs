// mutation-tables 那一格的绿样本：表里唯一一条的原文（main 里那个变量名）在这个文件里只许出现一次，注释里也不写它；
// 那一条钉绝对值的断言是给 absolute-assertions 那一格的，不许带那个变量名，不然原文就命中两次。
fn main() {
    let foo = 1;
    assert_eq!(std::mem::size_of::<u64>(), 8);
}
