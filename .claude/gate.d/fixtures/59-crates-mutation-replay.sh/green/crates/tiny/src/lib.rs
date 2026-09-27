// 门禁 59 号的判别力样本：一个会红的变异（加改成减）与一个不会红的变异（只动注释）。
pub fn add(left: u32, right: u32) -> u32 {
    left + right
}

/// 除数是 0 就 panic：`#[should_panic]` 的用例在 libtest 的结果行里名字后面多一段 ` - should panic`，59 号要认得出它红了
pub fn divide(dividend: u32, divisor: u32) -> u32 {
    dividend / divisor
}

#[cfg(test)]
mod tests {
    #[test]
    #[should_panic]
    fn dividing_by_zero_panics() {
        super::divide(1, 0);
    }
    #[test]
    fn adds_two_and_three_to_five() {
        assert_eq!(super::add(2, 3), 5);
    }
}
