/// S5 宏的使用处。
pub fn wrapped(publish: u64) -> u64 {
    crate::wrap_once!(publish, 24)
}
