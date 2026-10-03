//! FC：被改函数 pack，返回类型是别的文件里的别名。
use crate::word::Word;

pub fn pack(high: u64, low: u64) -> Word {
    ((high << 8) | low) as Word
}
