pub fn one() -> u32 {
    1
}

// 样本：库单测，闭包不提特性（两趟同一把键）；假 cargo 在不带特性那一趟判它 FAILED、带特性那一趟判它 ok
#[cfg(test)]
mod tests {
    #[test]
    fn shared_check() {
        assert_eq!(super::one(), 1);
    }
}
