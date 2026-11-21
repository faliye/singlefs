use super::*;

/// A 藏一条根时与第 3 次跑那一份落点逐项相同（k 的推广不改 k = 1）。
#[test]
fn fourth_run_abandonment_faults_equal_the_third_run_ones_for_one_hidden_root() {
    let parameters = parameters_for(&GEOMETRY_PRIMARY);
    let mut history = third_run_history_start(&GEOMETRY_PRIMARY, 2).expect("P(2, ·)");
    third_run_close(&mut history, &parameters, Closing::ProcessExit).expect("关闭 C");
    let geometry = independent_geometry(&history.pool).expect("几何");
    for (form, replay_break) in [
        (UnreadableForm::ReadFails, ReplayBreak::EveryRecord),
        (
            UnreadableForm::ReadsZeros,
            ReplayBreak::FirstCopyOfTheFirstNamedUnit,
        ),
    ] {
        let third = third_run_abandonment_faults(
            &history,
            &geometry,
            form,
            replay_break,
            FaultDuration::WholeCall,
        )
        .expect("第 3 次跑");
        let fourth = fourth_run_abandonment_faults(
            &history,
            &geometry,
            form,
            replay_break,
            FaultDuration::WholeCall,
            1,
        )
        .expect("第 4 次跑");
        assert_eq!(third, fourth);
    }
    let two = fourth_run_abandonment_faults(
        &history,
        &geometry,
        UnreadableForm::ReadFails,
        ReplayBreak::EveryRecord,
        FaultDuration::WholeCall,
        2,
    )
    .expect("k = 2")
    .expect("有记录");
    assert_eq!(
        two.iter()
            .filter(|fault| fault.label.starts_with("root"))
            .count(),
        2
    );
    let one = fourth_run_abandonment_faults(
        &history,
        &geometry,
        UnreadableForm::ReadFails,
        ReplayBreak::EveryRecord,
        FaultDuration::WholeCall,
        1,
    )
    .expect("k = 1")
    .expect("有记录");
    let record_labels = |faults: &[ThirdRunFault]| -> BTreeSet<String> {
        faults
            .iter()
            .filter(|fault| fault.label.starts_with("record"))
            .map(|fault| fault.label.clone())
            .collect()
    };
    assert!(record_labels(&two).is_disjoint(&record_labels(&one)));
}
