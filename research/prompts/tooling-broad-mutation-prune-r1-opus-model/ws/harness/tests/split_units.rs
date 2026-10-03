//! S1：取样上沿来自只放常量的 crate；被改文件 split.rs 不引用它。
mod common;
use model_fmtc::SAMPLE_LIMIT;

#[test]
fn units_match_the_ceiling_division_over_the_sample() {
    for length in common::sample_lengths(SAMPLE_LIMIT) {
        let expected = if length == 0 { 1 } else { length.div_ceil(10) };
        assert_eq!(model_core::split::units(length, 10), expected, "length={length}");
    }
}
