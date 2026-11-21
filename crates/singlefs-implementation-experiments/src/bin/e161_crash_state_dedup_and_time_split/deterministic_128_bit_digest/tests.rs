//! 128 位确定性散列（登记 5.1）：内容、调用、键序列与键集合的摘要的单测。

use super::*;

#[test]
fn digest_is_deterministic_and_separates_one_byte() {
    assert_eq!(
        digest_of_bytes(b"e161"),
        digest_of_bytes(b"e161"),
        "同样的字节同样的摘要"
    );
    assert_ne!(
        digest_of_bytes(b"e161"),
        digest_of_bytes(b"e162"),
        "差一个字节摘要不同"
    );
    assert_ne!(
        digest_of_bytes(&[0u8; 8]),
        digest_of_bytes(&[0u8; 9]),
        "长度进摘要"
    );
}
