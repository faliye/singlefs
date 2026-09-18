//! 字节串写成小写十六进制文本。E142（第一个事务的干跑） 第十一次跑的跑前登记第一节第 4 条：
//! 值一律报成小端十六进制字节串，不报十进制——免得「8 字节字段」与「变了几个字节」混成一句。

use std::fmt::Write as _;

/// 一段字节按盘上的先后写成小写十六进制，两个字符一个字节，中间不插分隔符。
#[must_use]
pub fn hexadecimal_text(bytes: &[u8]) -> String {
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(text, "{byte:02x}").expect("往 String 里写 fmt 不会失败");
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hexadecimal_text_keeps_the_on_disk_byte_order_and_pads_each_byte_to_two_characters() {
        assert_eq!(hexadecimal_text(&[]), "");
        assert_eq!(hexadecimal_text(&[0x03, 0x00, 0x00, 0x0f]), "0300000f");
        assert_eq!(hexadecimal_text(&[0xff, 0x00]), "ff00");
    }
}
