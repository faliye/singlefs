// checker-tier-research-build-and-replay.sh 的 research-unit-tests 格按条复用样本：测试模块里三条单测；setup.sh 造的上一版没有第三条（对照样本 N06：测试模块加一条单测）
pub fn add(left: u32, right: u32) -> u32 {
    left + right
}

pub fn double(value: u32) -> u32 {
    value * 2
}

#[cfg(test)]
mod tests {
    #[test]
    fn adds_two_and_two_to_four() {
        assert_eq!(super::add(2, 2), 4);
    }

    #[test]
    fn doubles_three_to_six() {
        assert_eq!(super::double(3), 6);
    }

    #[test]
    fn adds_zero_to_itself() {
        assert_eq!(super::add(0, 0), 0);
    }
}
