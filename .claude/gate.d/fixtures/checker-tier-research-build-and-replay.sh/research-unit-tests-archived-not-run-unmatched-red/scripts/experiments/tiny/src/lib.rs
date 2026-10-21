// checker-tier-research-build-and-replay.sh 的 research-unit-tests 格样本：本次未跑行点名的用例名写错、对不上计划里任何一条，这一格判红
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
    fn reads_an_archived_product_that_is_not_there() {
        use std::io::Write as _;
        // 照 scripts/experiments/lib.rs 的 read_archived_product_or_report_not_run：读不到就往进程的标准输出写一行本次未跑，照常返回
        writeln!(std::io::stdout().lock(), "NOT_RUN_ARCHIVED_PRODUCT test=lib::tests::a_test_name_that_is_not_in_the_plan reason=research/results/sample-product.out 读不到，这一项本次未跑").expect("写得出本次未跑那一行");
    }
}
