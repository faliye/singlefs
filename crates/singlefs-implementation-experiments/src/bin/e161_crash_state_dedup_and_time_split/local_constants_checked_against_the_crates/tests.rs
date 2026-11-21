//! 本地常量与它们和 crates 的回比（停机条款 S1）的单测。

use super::*;

#[test]
fn every_local_constant_equals_its_crates_counterpart() {
    let constants = local_constants_against_the_crates();
    assert_eq!(constants.len(), 14, "S1 列了十四个常量");
    for (name, local, crates) in constants {
        assert_eq!(local, crates, "S1：{name} 本地 {local}，crates {crates}");
    }
    assert_eq!(
        UNIT_AREA_START_BYTES, 822_083_584,
        "单元区起点 50176 × 16384"
    );
    assert_eq!(RING_START_BYTES, 16_777_216, "环起点 1024 × 16384");
}
