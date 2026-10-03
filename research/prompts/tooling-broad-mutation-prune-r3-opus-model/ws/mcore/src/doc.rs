//! X7：只有文档测试抓得到的函数（nextest 不跑文档测试，按条插桩的覆盖里没有它）。

/// ```
/// assert_eq!(mcore::doc::halve(10), 5);
/// ```
pub fn halve(x: u32) -> u32 {
    x / 2
}
