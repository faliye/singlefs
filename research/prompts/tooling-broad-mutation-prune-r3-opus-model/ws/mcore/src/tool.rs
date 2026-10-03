//! X2：只在测试起的另一个二进制（CARGO_BIN_EXE_model-tool）里执行到的函数。
pub fn checksum(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0;
    for &b in bytes {
        h = (h * 31 + u64::from(b)) % 65521;
    }
    h
}

/// X2b：只有 model-tool 那个二进制叫它，测试二进制里一处都不叫。
pub fn digest_only_in_tool(bytes: &[u8]) -> u64 {
    bytes.iter().map(|&b| u64::from(b) * 3).sum()
}
