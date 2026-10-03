/// S4b 被改文件：只经 extra.rs 里 Gadget 的固有方法被叫到。
pub fn numbered(prefix: &str, number: u64) -> String {
    format!("{prefix}-{}", number + 1)
}
