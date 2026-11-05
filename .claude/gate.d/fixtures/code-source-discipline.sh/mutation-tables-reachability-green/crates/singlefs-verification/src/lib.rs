// 门禁 code-source-discipline 的判别力样本（mutation-tables 格的可达性）：只放变异表要改的函数与点名的测试。
pub mod crash_amplification;
pub mod crash_state_enumeration;

#[cfg(test)]
mod tests {
    #[test]
    fn reaches_the_counted_function() {
        assert_eq!(super::crash_amplification::counted(2), 3);
    }
}
