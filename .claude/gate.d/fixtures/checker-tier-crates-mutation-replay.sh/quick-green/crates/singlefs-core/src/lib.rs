// 门禁 checker-tier-crates-mutation-replay --quick 的判别力样本：变异表两行同一个测试目标，轻型登记表只登记其中一行，--quick 只判那一行。
pub fn add(left: u32, right: u32) -> u32 {
    left + right
}

#[cfg(test)]
mod tests {
    #[test]
    fn adds_two_and_three_to_five() {
        assert_eq!(super::add(2, 3), 5);
    }
}
