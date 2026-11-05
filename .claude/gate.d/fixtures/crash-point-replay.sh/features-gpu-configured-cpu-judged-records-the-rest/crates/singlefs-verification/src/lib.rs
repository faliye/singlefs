pub fn one() -> u32 {
    1
}

// 样本：库单测，闭包不提特性：不带特性那一趟判绿记库，带 gpu 特性那一趟照键复用
#[cfg(test)]
mod tests {
    #[test]
    fn shared_check() {
        assert_eq!(super::one(), 1);
    }
}
