// 闭包模型（m3-crash-enumeration-merge-r3 攻方腿）：从 crates/singlefs-checker-tier/src/crash_identity.rs 按行原样抽出
// 词法切分与「按模块引用收闭包」那几段（行区间写在 extract.sh 里，抽之前核那份文件的 sha256），不链 checker 档的库。
// 只把摘要换成一个不求密码学强度的 FNV 摘要（这里只比文件集合）；工具链身份换成三段空串的同形结构。
#![allow(dead_code, clippy::all)]
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
pub const DIGEST_BYTES: usize = 32;
pub fn sha256_digest(bytes: &[u8]) -> [u8; DIGEST_BYTES] {
    let mut out = [0u8; DIGEST_BYTES];
    let mut h: u64 = 0xcbf29ce484222325;
    for (i, b) in bytes.iter().enumerate() {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x100000001b3);
        out[i % DIGEST_BYTES] ^= (h >> 24) as u8;
    }
    out
}
pub fn hexadecimal_text(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
pub struct Layer0ToolchainIdentity {
    pub rustc_version_lines: String,
    pub cargo_version: String,
    pub target_triple: String,
}
