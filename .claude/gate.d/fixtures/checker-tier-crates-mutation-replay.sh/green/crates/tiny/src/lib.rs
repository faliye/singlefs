// 门禁 checker-tier-crates-mutation-replay 的判别力样本：一个会红的变异（加改成减）与一个不会红的变异（只动注释）。
pub fn add(left: u32, right: u32) -> u32 {
    left + right
}

/// 删掉第二次相加就少加一次：变异表里替换文空着的一行（删掉原文）照常改坏这里，点名的测试红
pub fn add_twice(value: u32) -> u32 {
    let mut total = value;
    total += value;
    total
}

/// 除数是 0 就 panic：`#[should_panic]` 的用例在 libtest 的结果行里名字后面多一段 ` - should panic`，checker-tier-crates-mutation-replay 要认得出它红了
pub fn divide(dividend: u32, divisor: u32) -> u32 {
    dividend / divisor
}

#[cfg(test)]
mod tests {
    #[test]
    #[should_panic]
    fn dividing_by_zero_panics() {
        super::divide(1, 0);
    }
    #[test]
    fn adding_three_twice_gives_six() {
        assert_eq!(super::add_twice(3), 6);
    }
    #[test]
    fn adds_two_and_three_to_five() {
        assert_eq!(super::add(2, 3), 5);
    }
    /// 像仓里 publish_order_matches_litmus.rs 那样按 CARGO_MANIFEST_DIR/../../litmus 读：litmus/ 在 checker-tier-crates-mutation-replay 的拷贝范围里，副本里读得到，基线是 ok
    #[test]
    fn litmus_sample_writes_one() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../litmus").join("sample.litmus");
        let text = std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("读不了 {}：{error}", path.display()));
        assert!(text.contains("WRITE_ONCE(*x, 1)"));
    }
}
