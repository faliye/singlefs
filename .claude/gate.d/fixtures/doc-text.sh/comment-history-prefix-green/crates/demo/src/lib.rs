//! 「以」「除以」「所以」接前缀、前者、前面的构词，都不是讲过去。

/// 续扫起点要以前缀起头。
pub fn resume(after_key: &[u8], prefix: &[u8]) -> bool {
    after_key.starts_with(prefix)
}

// 后者除以前者向上取整。
pub fn ceil_div(later: u64, earlier: u64) -> u64 {
    later.div_ceil(earlier)
}

// 所以前面已发的写都看得见。
// 除以前一项的值再取整。
pub const VISIBLE: bool = true;
