/// 样本：校验和（逐字节乘 31 累加）
pub fn crc_of(bytes: &[u8]) -> u32 {
    bytes.iter().fold(0u32, |sum, byte| sum.wrapping_mul(31).wrapping_add(u32::from(*byte)))
}

/// 样本：带位置权重的和
pub fn weighted_sum_of(bytes: &[u8]) -> u32 {
    bytes.iter().enumerate().fold(0u32, |sum, (index, byte)| sum.wrapping_add(u32::from(*byte).wrapping_mul(index as u32 + 1)))
}

/// 样本：没有测试引它
pub fn spare_function() -> u32 {
    7
}

#[cfg(test)]
mod tests {
    #[test]
    fn crc_of_nothing_is_zero() {
        assert_eq!(super::crc_of(&[]), 0);
    }
}
