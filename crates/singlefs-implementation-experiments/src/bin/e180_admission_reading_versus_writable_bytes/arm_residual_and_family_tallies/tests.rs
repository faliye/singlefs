//! 累计：各臂、残差、轨迹、产物行与一族的合计的单测。

use super::*;

#[test]
fn the_trajectory_counts_peak_and_positive_states_after_the_first_state_only() {
    let mut trajectory = TrajectoryTally::default();
    for value in [100, -5, 7, 3, -1] {
        trajectory.record(value);
    }
    assert_eq!(trajectory.first_value, 100);
    assert_eq!(trajectory.peak_after_the_first, Some(7));
    assert_eq!(trajectory.positive_states_after_the_first, 2);
    assert_eq!(trajectory.last_value, -1);
}
