pub const SAMPLE_BYTES: u64 = 16384;
pub const fn twice(value: u64) -> u64 { 2 * value }

#[cfg(test)]
mod tests {
    #[test]
    fn twice_doubles() {
        for value in [1, 2] { if super::twice(value) != 2 * value { panic!("样本：测试模块里的分支不算共享模块的正文"); } }
    }
}
