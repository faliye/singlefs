use super::*;

#[test]
fn the_judgement_threshold_moves_a_cell_from_green_to_red() {
    assert_eq!(status_above(3, 4, "resurrects"), "zero");
    assert_eq!(status_above(3, 2, "resurrects"), "resurrects");
    assert_eq!(status_by_threshold(1, "resurrects"), "resurrects");
    assert_eq!(status_by_threshold(0, "resurrects"), "zero");
}

/// 第九节 M15 的取样点：F11 吸收缺口（容量 16、θ = 8、N2），回退之后又追加过墓碑与退役记录。B-on × E-I 下回退之后不在途中的扫描
/// 容器 m、inode m 都是 0（缺口里的退役记录与墓碑死亡 txg 不在新父链上，不施行）。B-off-lin 在这一段量到的容器 m 不在这里钉：
/// 回退之后第一次追加退役记录就是重跑缺口里第一次身份消失，左移重写又给后面几片换了诞生 txg（修订 1）。
#[test]
fn after_a_rollback_retirements_and_tombstones_from_the_gap_do_not_apply() {
    let (simulation, outcome) = simulate(History::RollBackOverAGap { event: GapEvent::Absorb }, write_path_cell(BorrowArm::RenewIdentity, TEST_LEAF_CAPACITY, 8), true);
    let outcome = outcome.expect("F11 交回回退的结局");
    assert!(outcome.event_found);
    assert!(outcome.retirement_records_after_the_rollback > 0);
    assert!(outcome.retired_in_the_gap_and_present_in_the_old_version > 0);
    let trajectories = trajectories_of(&simulation.scans, every_reader().len());
    let on_lineage = position_of(BaseArm::RetirementOnLineage, ShrinkArm::RecordSize, UnlinkEffectArm::AtIntent, ReadableWindow::Forever);
    let after_rollback = &trajectories[on_lineage].by_scope[SCOPE_AFTER_ROLLBACK_OUTSIDE_UNLINK];
    assert!(after_rollback[QUANTITY_CONTAINER_MISSED].scans > 0);
    assert_eq!(after_rollback[QUANTITY_CONTAINER_MISSED].peak, 0);
    assert_eq!(after_rollback[QUANTITY_INODE_MISSED].peak, 0);
}
