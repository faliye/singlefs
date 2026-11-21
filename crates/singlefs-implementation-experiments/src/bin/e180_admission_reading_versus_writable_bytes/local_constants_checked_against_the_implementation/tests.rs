//! 本地常量与它们和实现的回比的单测。

use super::*;

#[test]
fn local_constants_match_the_implementation_values() {
    assert_eq!(local_constants_against_the_implementation(), 15);
}
