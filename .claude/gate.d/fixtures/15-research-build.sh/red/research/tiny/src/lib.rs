// 门禁 15 号的判别力样本：research 工作区里有一条单测红着，本阶段必须判红，不许因为构建过了就报绿
pub fn add(left: u32, right: u32) -> u32 {
    left + right
}

#[cfg(test)]
mod tests {
    #[test]
    fn adds_two_and_two_to_five() {
        assert_eq!(super::add(2, 2), 5, "样本故意写错的期望值");
    }
}
