// checker-tier-research-build-and-replay.sh 的 research-unit-tests 格的绿样本：research 工作区编得过、一条单测绿着，这一格必须判绿
pub fn add(left: u32, right: u32) -> u32 {
    left + right
}

#[cfg(test)]
mod tests {
    #[test]
    fn adds_two_and_two_to_four() {
        assert_eq!(super::add(2, 2), 4);
    }
}
