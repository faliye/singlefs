//! FB：被改函数 encode_len；frame_ok 经 crate::codec::check 判。
pub fn encode_len(length: u64) -> u64 {
    length + 1
}

pub fn frame_ok(length: u64) -> bool {
    crate::codec::check(encode_len(length), length)
}
