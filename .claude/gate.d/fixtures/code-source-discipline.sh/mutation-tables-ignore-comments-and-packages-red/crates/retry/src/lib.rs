pub fn retry_rounds(rounds: u64) -> u64 {
    rounds
}

#[cfg(test)]
mod tests {
    // 样本：点名这条测试的变异行写 `-p demo -p retry --lib`，测试只在 retry 这个包里
    #[test]
    fn retry_rounds_pass_through() {
        assert_eq!(super::retry_rounds(3), 3);
    }
}
