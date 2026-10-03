//! FH：被改函数 route；没改时测试只走 small，变异把它拐进 big。
pub fn route(value: u64) -> u64 {
    if value > 100 {
        crate::big::scale(value)
    } else {
        crate::small::scale(value)
    }
}
