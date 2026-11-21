use super::*;

/// 登记 5.6 与 7.2：第 4 次跑的常量回比、偏移解码、三个几何上的根槽 / 环槽 / 系统配置槽数、H1f / H1g 的格数、臂探针全过（V3、V4）。
#[test]
fn fourth_run_constants_and_anchors_all_pass() {
    assert!(run_fourth_run_constants_and_anchors("today"));
}
