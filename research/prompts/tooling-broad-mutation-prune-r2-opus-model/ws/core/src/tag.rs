//! FD：被改函数 tag 用别的文件里枚举的判别值。
use crate::kind::Kind;

pub fn tag(kind: Kind, count: u64) -> u64 {
    (kind as u64) * count
}
