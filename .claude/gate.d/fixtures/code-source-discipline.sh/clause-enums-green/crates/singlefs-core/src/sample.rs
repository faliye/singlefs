//! 样本：代码只引用记账表里已有的那一位；合成出来的那个常量解不出位号，要进「没判位号的」名单。
pub const INCOMPAT_SAMPLE_LINE_BIT: u8 = 0x01;
pub const SUPPORTED_INCOMPAT_SAMPLE_BITS: u8 = INCOMPAT_SAMPLE_LINE_BIT;
