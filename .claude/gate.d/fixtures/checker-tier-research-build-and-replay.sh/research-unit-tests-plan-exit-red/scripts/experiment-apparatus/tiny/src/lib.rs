// checker-tier-research-build-and-replay.sh 的 research-unit-tests 格的按条复用失灵样本：research 工作区编得过、一条单测绿着，
// 样本根的 .unit-verdict-sample/plan-failure 让 admission.py test-plan 装失灵，这一格照旧全部现判跑完、必须判红
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
