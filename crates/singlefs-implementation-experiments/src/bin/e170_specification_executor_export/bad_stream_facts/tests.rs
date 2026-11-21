//! 第三段：一条流上造坏状态要的事实（基底状态、系统配置、最新一版与伪记录）的单测。

use super::*;

#[test]
fn the_middle_position_is_the_ceiling_of_half_counted_from_one() {
    assert_eq!(middle_position(1), 0);
    assert_eq!(middle_position(2), 0, "K = 2 取第 1 个");
    assert_eq!(middle_position(14), 6, "K = 14 取第 7 个");
    assert_eq!(middle_position(15), 7, "K = 15 取第 8 个");
}
