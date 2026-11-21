//! 几份测试共用的辅助：带参数起整个主流程并取它的输出行、甲放行前后与乙 2 关共享时的各部分字节。

use super::*;

pub(super) fn main_with(change: impl FnOnce(&mut Knobs)) -> Geometry {
    let mut knobs = Knobs::main_geometry();
    change(&mut knobs);
    Geometry::new(knobs)
}

pub(super) fn jia_before(geometry: &Geometry) -> u64 {
    arm_bytes(geometry, Arm::JiaAfterPatch).copied_before
}

pub(super) fn jia_after(geometry: &Geometry) -> u64 {
    arm_bytes(geometry, Arm::JiaAfterPatch).copied_after
}

// ── 第四份第七节新钉的锚点（数取设计员草稿 calc/anchors_r4.out、ding_peak_r4.out、sequence_r4.out、mutations_r4.out） ──

pub(super) fn yi2_sharing_off_parts(geometry: &Geometry) -> (u64, u64, u64, u64) {
    let rounds = yi2_sharing_turned_off_copied_before_admission(geometry);
    (rounds.ordinary_allocation_pass, rounds.settle_round, rounds.outside_the_rounds, rounds.total(geometry.knobs.settle_rounds))
}
