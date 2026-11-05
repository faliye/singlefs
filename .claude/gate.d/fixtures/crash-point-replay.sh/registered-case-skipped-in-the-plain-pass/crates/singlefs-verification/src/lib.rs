// 样本：crates 侧接上了块查询（admission.py crash-case-blocks 认这个变量名在不在）
pub const BLOCKS_QUERY_VARIABLE: &str = "SINGLEFS_CRASH_CASE_BLOCKS_QUERY";

pub fn one() -> u32 { 1 }

#[cfg(test)]
mod tests {
    #[test]
    fn one_is_one() {
        assert_eq!(super::one(), 1);
    }
}
