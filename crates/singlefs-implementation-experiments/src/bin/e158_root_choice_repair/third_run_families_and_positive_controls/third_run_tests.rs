use super::*;

/// PC-丢写（今天那一臂）：P(1, C) → A（PC-N 那组故障，另加系统配置没见证到被藏的根：挂载前的池换成
/// `crate::tests::with_the_rotation_after_the_newest_publish_not_written` 给的那一份；
/// C554 乙合进 `crates` 之后见证在时可写挂载拒，造不出被抛弃的根）→ 最后确认那一版的数据单元被改写 → 再可写挂载：Q1 前两栏数得出、
/// 读回不是最后确认那一版（变异 M8 要它红：那时恢复落到再挂载写下的根，它不在被测那次挂载开始时的镜像上）。
#[test]
fn an_overwritten_last_confirmed_version_counts_as_a_lost_write() {
    let parameters = parameters_for(&GEOMETRY_PRIMARY);
    let mut history = third_run_history_start(&GEOMETRY_PRIMARY, 1).expect("P(1, ·)");
    third_run_close(&mut history, &parameters, Closing::ProcessExit).expect("关闭 C");
    let geometry = independent_geometry(&history.pool).expect("几何");
    let faults = third_run_abandonment_faults(
        &history,
        &geometry,
        UnreadableForm::ReadFails,
        ReplayBreak::EveryRecord,
        FaultDuration::WholeCall,
    )
    .expect("落点")
    .expect("有记录");
    let pool = crate::tests::with_the_rotation_after_the_newest_publish_not_written(&history.pool);
    let tested = run_third_run_mount_writable(&pool, &parameters, &faults, None, None);
    assert!(tested.outcome.is_ok());
    let protected = history.confirmations.last().cloned().expect("确认过");
    let (corrupted, loss) =
        positive_control_lost_write(&history, &protected, &pool, &tested.pool_after, &parameters);
    assert!(corrupted > 0);
    assert!(loss.overwrite_cell());
    assert_ne!(loss.read_back, "last_confirmed");
}
