pub fn one() -> u32 {
    1
}

// 样本：库单测，闭包不提特性（两趟同一把键）；假 cargo 两趟都判它 ok：不带特性那一趟现判、记库，带特性那一趟照键复用、经 --skip 不跑
#[cfg(test)]
mod tests {
    #[test]
    fn shared_check() {
        assert_eq!(super::one(), 1);
    }
}
