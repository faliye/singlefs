// research-build-and-replay.sh 的 research-unit-tests 格「有红照记」样本：lib 里两条单测，一条总判红、一条判绿；
// tests/integration.rs 是另一个测试二进制，排在 lib 后面跑（cargo test 不带 --no-fail-fast 时 lib 红了它不跑）
pub fn add(left: u32, right: u32) -> u32 {
    left + right
}

#[cfg(test)]
mod tests {
    #[test]
    fn adds_two_and_two_to_four() {
        assert_eq!(super::add(2, 2), 4);
    }

    #[test]
    fn fails_on_purpose() {
        assert_eq!(super::add(2, 2), 5, "样本：这一条总判红");
    }
}
