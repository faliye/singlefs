use super::*;

/// N-配置 的 c_见证 取每块盘两槽的最大值，不只取世代号最新那一槽（手写两盘两槽：新槽 tail 0 实例 2、旧槽 tail c 实例 1；变异 M3 的装置一半）。
#[test]
fn the_configuration_witness_takes_both_slots_of_every_device() {
    let witnessed = 9u64;
    let decoded = DeviceDecodedImage {
        root_keys: BTreeSet::new(),
        unreadable_root_slots: 0,
        records: BTreeMap::new(),
        system_configuration_slots: vec![
            (0, 2, 0, 2),
            (0, 1, witnessed, 1),
            (1, 2, 0, 2),
            (1, 1, witnessed, 1),
        ],
    };
    let (newer, tail, last, undecidable) = device_configuration_witness(&decoded, (1, 3));
    assert_eq!(tail, witnessed);
    assert_eq!(last, None);
    assert!(newer);
    assert!(undecidable);
}

/// 装置自己的择根与重放在一段 P(1, C) 上与 `crates/` 的 `choose_root`、`replay_journal` 交回的同一条根（7.3 第一行的不注入那一格）。
#[test]
fn the_device_replay_agrees_with_crates_on_an_untouched_history() {
    let parameters = parameters_for(&GEOMETRY_PRIMARY);
    let mut history = third_run_history_start(&GEOMETRY_PRIMARY, 1).expect("P(1, ·)");
    third_run_close(&mut history, &parameters, Closing::ProcessExit).expect("关闭 C");
    let geometry = independent_geometry(&history.pool).expect("几何");
    let judgement = device_judgement(&history.pool, &geometry, &[]);
    let devices = devices_from_pool(&history.pool, IMAGE_BYTES);
    let system_configuration = choose_system_configuration(&devices).expect("系统配置");
    let chosen = singlefs_core::recovery::choose_root(&devices, &system_configuration).expect("根");
    let records = scan_journal(&devices, &system_configuration);
    let (_, effective) = replay_journal(
        &devices,
        &chosen,
        system_configuration.immutable.sizes.journal_ring_bytes,
        &records,
        NamedUnitVerification::Verify,
    )
    .expect("重放");
    assert_eq!(judgement.chosen, Some(root_pair(&chosen)));
    assert_eq!(judgement.effective, Some(root_pair(&effective)));
    assert_eq!(judgement.effective, Some(history.tip()));
    assert!(!judgement.truth_newer);
    assert_eq!(history.tail_check_failures, Vec::<String>::new());
    assert_eq!(history.tail_checks, 2);
}

/// PC-N 的真值一半：被藏那条根在孪生镜像上读得出，N_真 为真；今天那一臂回滚到被藏那条根的前一条（变异 M1 要它红）。
#[test]
fn the_truth_of_a_newer_root_is_read_on_the_twin_image() {
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
    let judgement = device_judgement(&history.pool, &geometry, &faults);
    assert!(judgement.truth_newer);
    assert!(judgement.configuration_newer);
    assert_eq!(
        judgement.effective,
        history.timeline.iter().rev().nth(1).copied()
    );
}
