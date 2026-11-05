pub fn one() -> u32 {
    1
}

pub fn two() -> u32 {
    2
}

// 样本：两条库单测，闭包都不提特性（两趟同一把键）；假 cargo 在不带特性那一趟判 red_check FAILED、green_check ok，带特性那一趟两条都判 ok
#[cfg(test)]
mod tests {
    #[test]
    fn red_check() {
        assert_eq!(super::one(), 1);
    }

    #[test]
    fn green_check() {
        assert_eq!(super::two(), 2);
    }
}
