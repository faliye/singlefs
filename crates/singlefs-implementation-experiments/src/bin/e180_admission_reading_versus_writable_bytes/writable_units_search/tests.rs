//! 真可写单元数的搜索：journal 记录上限的模型、空间条件的上界、下界的收紧与删失的单测。

use super::*;

#[test]
fn the_interior_points_split_the_gap_into_distinct_points_strictly_inside() {
    let points = interior_points(0, 100, 4);
    assert_eq!(points, vec![20, 40, 60, 80]);
    let narrow = interior_points(10, 12, 6);
    assert_eq!(narrow, vec![11]);
}
