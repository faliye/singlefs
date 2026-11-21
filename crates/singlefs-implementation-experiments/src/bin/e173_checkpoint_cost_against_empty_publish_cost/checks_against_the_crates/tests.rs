//! `checks_against_the_crates` 的单测。
use super::*;

#[test]
fn local_constants_equal_the_crates_constants() {
    assert!(
        check_local_constants_against_the_crates(),
        "停机条款 S4：本地常量与 crates 同名常量逐个相等（不等的那一行 equal=false）"
    );
}

#[test]
fn the_anchors_computed_by_the_device_equal_the_registered_values() {
    assert!(
        check_the_anchors(),
        "作废条款 V3：B10、A2、B11 的锚点逐个相等（不等的那一行 equal=false）"
    );
}
