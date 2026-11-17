pub fn one() -> u32 {
    1
}

// 样本：库单测，闭包不提特性；setup.sh 造的上一版只有这一条的断言不同
#[cfg(test)]
mod tests {
    #[test]
    fn shared_check() {
        assert_eq!(super::one(), 1);
    }
}
