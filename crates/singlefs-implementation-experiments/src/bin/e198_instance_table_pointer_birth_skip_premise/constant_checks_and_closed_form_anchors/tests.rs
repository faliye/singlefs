use super::*;

#[test]
fn constants_match_the_implementation() {
    for check in constant_checks() {
        assert!(
            check.holds(),
            "{} 本地 {} 实现 {}",
            check.name,
            check.local,
            check.implementation
        );
    }
}
