//! 一个取样态上实现读回的那一版、崩溃判官（O1-崩溃判官）的判与恢复摘要的单测。

use super::*;

/// 池级 checker 的判定里只有 `Violated` 算 O2 红（X6：`Violated` 当 `Holds`，这里红）。
#[test]
fn only_a_violated_invariant_counts_as_a_red_pool_checker_verdict() {
    let verdicts: Vec<(&'static str, InvariantVerdict)> = vec![
        ("I-1.1", InvariantVerdict::Violated("坏".to_string())),
        ("I-1.2", InvariantVerdict::Holds),
        ("I-1.3", InvariantVerdict::NotApplicable("没有对象")),
    ];
    assert_eq!(
        violated_invariant_names(&verdicts),
        vec!["I-1.1".to_string()]
    );
}
