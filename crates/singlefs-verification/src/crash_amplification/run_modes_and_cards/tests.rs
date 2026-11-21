use super::*;

/// 配置点名的卡少了哪几张：按配置的次序列出用不上的序号；都在就是空。
#[test]
fn configured_cards_that_are_not_usable_are_named_in_the_configured_order() {
    assert_eq!(
        configured_cards_not_usable(&[(0, 32), (1, 16), (2, 16)], &[1]),
        vec![0, 2]
    );
    assert_eq!(
        configured_cards_not_usable(&[(0, 32), (1, 16)], &[1, 0, 3]),
        Vec::<u32>::new()
    );
}

/// 显卡优先级照写的次序读：写在前面的先装；每项点 HOSTS 里的地址；额度带单位（G、M，1024 进制）。写法不对的每一种都拒，
/// 旧的 `local:`、`peer:` 台标签不是 HOSTS 里的地址，同样拒。
#[test]
fn cards_by_priority_are_read_in_the_order_written_with_their_quotas() {
    let addresses = crate::gpu_card_configuration::multi_host_addresses_for_the_unit_tests()
        .expect("判法打的行认得出");
    let card = |host: &str, index: u32, memory_quota_mebibytes: u64| {
        (host.to_string(), index, memory_quota_mebibytes)
    };
    let read = |text: &str| {
        parse_cards_by_priority(text, &addresses).map(|configured| {
            configured
                .cards()
                .iter()
                .map(|entry| {
                    (
                        entry.host.as_str().to_string(),
                        entry.index,
                        entry.memory_quota_mebibytes,
                    )
                })
                .collect::<Vec<_>>()
        })
    };
    assert_eq!(
        read("127.0.0.1:0:15G  192.0.2.81:2:10G"),
        Ok(vec![
            card("127.0.0.1", 0, 15 * 1024),
            card("192.0.2.81", 2, 10 * 1024)
        ])
    );
    assert_eq!(
        read("192.0.2.81:1:512M"),
        Ok(vec![card("192.0.2.81", 1, 512)])
    );
    for refused in [
        "",
        "0:15G",
        "127.0.0.1:0",
        "192.0.2.99:0:15G",
        "127.0.0.1:first:15G",
        "127.0.0.1:0:15",
        "127.0.0.1:0:0G",
        "127.0.0.1:0:G",
        "127.0.0.1:0:15G 127.0.0.1:0:10G",
    ] {
        assert!(read(refused).is_err(), "「{refused}」该被拒");
    }
    // 同一个序号在两台机器上是两张卡
    assert_eq!(
        read("127.0.0.1:0:15G 192.0.2.81:0:10G").map(|cards| cards.len()),
        Ok(2)
    );
}

/// 任务先装第一张卡，装到它带得了的状态数为止，装不下的才给下一张；都装满还有剩的排队等，不算在任何一张卡上。
#[test]
fn states_fill_the_first_card_before_the_next_card_gets_any() {
    assert_eq!(
        place_states_on_cards_by_priority(100, &[150, 100]),
        [100, 0]
    );
    assert_eq!(
        place_states_on_cards_by_priority(150, &[150, 100]),
        [150, 0]
    );
    assert_eq!(
        place_states_on_cards_by_priority(151, &[150, 100]),
        [150, 1]
    );
    assert_eq!(
        place_states_on_cards_by_priority(1000, &[150, 100]),
        [150, 100]
    );
    assert_eq!(place_states_on_cards_by_priority(0, &[150, 100]), [0, 0]);
    // 第一张卡一个状态都带不了（表装不进它的额度）：整份给下一张
    assert_eq!(place_states_on_cards_by_priority(10, &[0, 100]), [0, 10]);
}

#[test]
fn weighted_shares_hand_out_blocks_in_proportion_and_never_skip_a_block() {
    let shares = WeightedShares::new(vec![3, 1]);
    let owners: Vec<usize> = (0..8).map(|block| shares.owner_of_block(block)).collect();
    assert_eq!(
        owners.iter().filter(|owner| **owner == 0).count(),
        6,
        "权重 3:1 的前 8 块里第一方拿 6 块"
    );
    assert_eq!(owners.iter().filter(|owner| **owner == 1).count(), 2);
    let equal = WeightedShares::new(vec![1, 1]);
    assert_eq!(
        (0..6)
            .map(|block| equal.owner_of_block(block))
            .collect::<Vec<usize>>(),
        vec![0, 1, 0, 1, 0, 1]
    );
}

#[test]
fn the_four_run_modes_come_from_the_two_switches() {
    assert_eq!(
        CrashAmplificationRunMode::from_switches(false, false),
        CrashAmplificationRunMode::SingleHostCpu
    );
    assert_eq!(
        CrashAmplificationRunMode::from_switches(true, false),
        CrashAmplificationRunMode::TwoHostsCpu
    );
    assert_eq!(
        CrashAmplificationRunMode::from_switches(false, true),
        CrashAmplificationRunMode::SingleHostCpuWithGpu
    );
    assert_eq!(
        CrashAmplificationRunMode::from_switches(true, true),
        CrashAmplificationRunMode::TwoHostsCpuWithGpus
    );
    let directory = crate::crash_library::ExclusiveTemporaryDirectory::create("run-mode")
        .expect("建得了一次性临时目录");
    let file = directory.join("multi-host.env");
    std::fs::write(&file, "ENABLE_ACROSS_MACHINES=1\nENABLE_GPU=\"0\"\n").expect("写得了");
    assert_eq!(
        CrashAmplificationRunMode::from_configuration_file(&file),
        Ok(CrashAmplificationRunMode::TwoHostsCpu)
    );
    drop(directory);
    assert!(
        CrashAmplificationRunMode::from_configuration_file(&file).is_err(),
        "文件不在（找到之后又没了）报错，不按两个开关都关"
    );
}

/// 找到了多机配置却读不了（这里存成不是 UTF-8 的字节；没有读权限同一条路）：报错、点名那一份、写出路，不按两个开关都关往下跑
/// （没写就是没有，不补缺省；读不了不是没写）。
/// 判别力（`crates/mutations.tsv`）：读不了照旧当空文件（两个开关都关），这条红。
#[test]
fn a_multi_host_configuration_found_but_unreadable_is_refused_with_the_way_out_instead_of_both_switches_off(
) {
    let directory = crate::crash_library::ExclusiveTemporaryDirectory::create(
        "run-mode-configuration-unreadable",
    )
    .expect("建得了一次性临时目录");
    let file = directory.join("multi-host.env");
    std::fs::write(&file, [0xFF, 0xFE, b'\n']).expect("写得了");
    match multi_host_configuration_of_the_file_found(Some(file.clone()), directory.path()) {
        Ok(configuration) => panic!("多机配置读不了却照 {:?} 往下跑", configuration.mode),
        Err(refusal) => assert!(
            refusal.contains(&file.display().to_string())
                && refusal.contains("读不了")
                && refusal.contains(
                    crate::gpu_card_configuration::MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE
                ),
            "拒绝说明要点名读不了的那一份与出路：{refusal}"
        ),
    }
}

/// 多机配置找不到就报错、写出路（照模板写一份，或设 `SINGLEFS_MULTI_HOST_CONFIG`），不读同名的环境变量、不按两个开关都关往下跑
/// （没写就是没有，不补缺省）；找到了跑法只照那一份。
/// 判别力（`crates/mutations.tsv`）：找不到时退回按两个开关都关往下跑，这条红。
#[test]
fn a_missing_multi_host_configuration_is_refused_with_the_way_out_instead_of_falling_back_to_the_environment(
) {
    let directory =
        crate::crash_library::ExclusiveTemporaryDirectory::create("run-mode-configuration-missing")
            .expect("建得了一次性临时目录");
    match multi_host_configuration_of_the_file_found(None, directory.path()) {
        Ok(configuration) => panic!(
            "找不到多机配置却照 {:?}（{}）往下跑",
            configuration.mode,
            configuration.configuration_file.display()
        ),
        Err(refusal) => assert!(
            refusal.contains("multi-host.env.example")
                && refusal.contains(
                    crate::gpu_card_configuration::MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE
                )
                && refusal.contains(&directory.path().display().to_string()),
            "拒绝说明要写从哪起找的与两条出路：{refusal}"
        ),
    }
    let file = directory.join("multi-host.env");
    std::fs::write(&file, "ENABLE_ACROSS_MACHINES=0\nENABLE_GPU=1\n").expect("写得了");
    let found = multi_host_configuration_of_the_file_found(Some(file.clone()), directory.path())
        .expect("找到了配置就照它定跑法");
    assert_eq!(
        (found.configuration_file, found.mode),
        (file, CrashAmplificationRunMode::SingleHostCpuWithGpu),
        "跑法只照找到的那一份"
    );
}

#[test]
fn compile_time_file_paths_normalize_to_repository_relative_ones() {
    assert_eq!(
        repository_relative_file("crates/a/tests/../../b/tests/common/mod.rs"),
        "crates/b/tests/common/mod.rs",
        "`#[path = \"../../…\"]` 引进来的模块，file!() 带 .."
    );
    assert_eq!(repository_relative_file("./x/./y.rs"), "x/y.rs");
    assert_eq!(
        repository_relative_file("../outside.rs"),
        "../outside.rs",
        "越出仓根的原样留着，交给 NodeLabel::new 拒绝"
    );
}

/// J5：配置开了 GPU、编进了 `gpu` 特性，判器、核对两类都走 GPU（没有登记哪几类走 GPU 的正表，豁免的那几截不走这里）。
#[test]
fn every_computation_runs_on_the_gpu_when_the_gpu_is_switched_on() {
    let routes = computation_routes(true, true).expect("开了 GPU、编进了 gpu 特性");
    assert_eq!(
        routes,
        ComputationRoutes {
            crash_judge: ComputedOn::Gpu,
            facts_verifier: ComputedOn::Gpu,
        }
    );
    assert!(routes.need_a_card());
}

/// J5：配置没开 GPU，两类都在 CPU 上，一张卡都不要；编没编进 `gpu` 特性都一样。
#[test]
fn every_computation_runs_on_the_cpu_when_the_gpu_is_switched_off() {
    for gpu_feature_is_compiled_in in [true, false] {
        let routes =
            computation_routes(false, gpu_feature_is_compiled_in).expect("没开 GPU 不报错");
        assert_eq!(
            routes,
            ComputationRoutes {
                crash_judge: ComputedOn::Cpu,
                facts_verifier: ComputedOn::Cpu,
            },
            "编进 gpu 特性：{gpu_feature_is_compiled_in}"
        );
        assert!(!routes.need_a_card());
    }
}

/// J5：开了 GPU 而这份二进制没编进 `gpu` 特性：报错并说要带的特性，不悄悄退回 CPU。
#[test]
fn the_gpu_switched_on_without_the_gpu_feature_compiled_in_is_refused() {
    let refused = computation_routes(true, false);
    assert!(
        refused
            .as_ref()
            .is_err_and(|reason| reason.contains("gpu 特性") && reason.contains("--features gpu")),
        "{refused:?}"
    );
}

/// 场景：配了 GPU（多机配置只写 `ENABLE_GPU=1`）、环境里没有服务交的卡；两次写同一扇区的小流挂一个只数状态的观察者（不交回盘），
/// 观察者的版本按判盘的 GPU 那一路取，录入段录进一份空库；照崩溃放量判段的选路取观察者交回的盘上的池级 checker
/// （[`pool_checker_of_the_images_after_the_observers`]，走 GPU），再判观察者。
/// 预期：取池级 checker 不取卡、不报错；观察者判段拼了结论，一张盘都不判。
/// 判别力（`crates/mutations.tsv`）：崩溃放量判段取观察者盘上的池级 checker 那一刻就取卡（没经服务给卡就报错），这条红。
#[cfg(feature = "gpu")]
#[test]
fn the_amplification_pool_checker_on_the_gpu_takes_no_card_when_no_observer_hands_back_an_image() {
    let (disposable, mut store) =
        an_empty_library_for("amplification-observers-without-images-take-no-card");
    let configuration = disposable.directory().join("multi-host-with-the-gpu.env");
    std::fs::write(&configuration, "ENABLE_GPU=1\n").expect("写得了配置");
    // 先开库、后拿改环境的锁；守卫清掉了服务交的卡（GPU_CARDS_BY_PRIORITY）与领卡通道，只设多机配置
    let environment = CrashAmplificationEnvironment::cleared_then_set(&[(
        crate::gpu_card_configuration::MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
        configuration.as_os_str(),
    )]);
    let count_the_state = |_ordinal: u64,
                           _image: &CrashImage<'_>,
                           _report: &RecoveryReport,
                           counts: &mut CrashPointReplayObserverCounts| {
        counts.add("states_seen", 1);
    };
    let observers = [the_counting_observer_without_an_observer_entry(
        &count_the_state,
    )];
    let (versions, kept_out) =
        versions_of_the_observer_without_an_observer_entry_under_the_process_marker(
            &observers[0],
            JudgedBy::GpuJudge,
            4,
        );
    store.keep_the_rows_of_these_versions_out_of_the_library(kept_out);
    a_flow_of_two_writes_of_the_same_bytes_to_one_sector().with_the_flow(|flow| {
        let plan = plan_crash_points(flow);
        let library = LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut store);
        record_the_observers_of_the_flow(
            &library,
            flow,
            &plan,
            &observers,
            std::slice::from_ref(&versions),
            &|_| true,
            &|| false,
        )
        .expect("录入段读写得了库");
        // 显卡来路照判段入口那样在守卫里记（配置开着 GPU、没有服务交的卡）；一张盘都不交回，用不上 GPU 批处理服务
        let pool_checker = pool_checker_of_the_images_after_the_observers(
            ComputedOn::Gpu,
            CardSourcesOfTheEngineStart::read_from_the_environment_now(configuration.clone()),
        )
        .expect("不在这一刻取卡：配置开了 GPU、环境里没有服务交的卡也取得到");
        let (observing, reuse_sentinel_problems, _observers_recorded_again) =
            judge_the_observers_then_the_reuse_sentinels(
                &library,
                flow,
                &plan,
                Some((
                    observers.as_slice(),
                    std::slice::from_ref(&versions),
                    &pool_checker,
                )),
                &|_| true,
                None,
                &|| false,
            )
            .expect("观察者一张盘都不交回：不取卡，判得完");
        let observing = observing.expect("交了观察者那一格，观察者判段的结果就在");
        assert!(
            observing.blocks_observed_here > 0,
            "前提：观察者挂着的块这一趟拼了结论"
        );
        assert_eq!(
            (
                observing.images_judged_here,
                observing.images_judged_on_the_gpu_here,
                observing.image_judgements_reused
            ),
            (0, 0, 0),
            "观察者一张盘都不交回：池级 checker 一张盘都不判"
        );
        assert!(
            reuse_sentinel_problems.is_empty(),
            "没抽哨兵：{reuse_sentinel_problems:?}"
        );
    });
    drop(environment);
}

/// 「配 GPU 必用 GPU」：配置开了 GPU 而这份二进制没编进 gpu 特性时判段报本次未跑（两行：入口认的那一行与门禁认的
/// 那一族记号），不退回 CPU 判；配置没开 GPU、或编进了 gpu 特性，判段照跑。
#[test]
fn the_gpu_switched_on_without_the_gpu_feature_reports_the_judge_not_run() {
    let reason = judge_not_run_without_the_gpu_feature(true, false)
        .expect("配置开了 GPU、没编进特性：判段报本次未跑");
    assert!(
        reason.contains("ENABLE_GPU=1") && reason.contains("gpu 特性"),
        "{reason}"
    );
    assert_eq!(judge_not_run_without_the_gpu_feature(false, false), None);
    assert_eq!(judge_not_run_without_the_gpu_feature(true, true), None);
    assert_eq!(judge_not_run_without_the_gpu_feature(false, true), None);
    let lines = not_run_lines_without_the_gpu_feature("一条流", &reason);
    assert_eq!(lines.len(), 2);
    assert!(
            lines[0].starts_with(
                "CRASH_AMPLIFICATION_NOT_RUN step=judge test=crash_amplification::run_from_environment flow=一条流 reason="
            ),
            "{}",
            lines[0]
        );
    assert!(
        lines[1].starts_with(
            "NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run_from_environment reason="
        ),
        "{}",
        lines[1]
    );
}

/// 第 ① 段在卡上判报的错是判器宽度停在拆不开的下限上的那一种：行首之后补上流名，可按行首 grep 到带流名的那一行；
/// 别的错（判不了的状态、表越过内核容量）原样交回。
#[test]
fn a_judge_stuck_at_the_indivisible_floor_is_reported_naming_the_flow() {
    let stuck_reason = format!(
        "{STUCK_AT_THE_INDIVISIBLE_FLOOR_LINE_PREFIX} card=\"一张卡\" seconds_of_the_floor=2.500"
    );
    assert_eq!(
            gpu_judging_refusal_naming_the_flow("一条流", stuck_reason),
            format!(
                "{STUCK_AT_THE_INDIVISIBLE_FLOOR_LINE_PREFIX} flow=一条流 card=\"一张卡\" seconds_of_the_floor=2.500"
            )
        );
    let other_reason = "卡 0 判不了点 某点 块内第 3 个状态".to_string();
    assert_eq!(
        gpu_judging_refusal_naming_the_flow("一条流", other_reason.clone()),
        other_reason
    );
}

/// 选段：不设两段先后跑，`record` 只跑录入段，`judge` 只跑判段；写别的值整趟拒、错误里点名变量与原值。
#[test]
fn the_run_stage_is_both_stages_unless_record_or_judge_is_named() {
    assert_eq!(
        RunStage::of_the_variable_value(None),
        Ok(RunStage::RecordThenJudge)
    );
    assert_eq!(
        RunStage::of_the_variable_value(Some("record")),
        Ok(RunStage::RecordOnly)
    );
    assert_eq!(
        RunStage::of_the_variable_value(Some("judge")),
        Ok(RunStage::JudgeOnly)
    );
    let refused = RunStage::of_the_variable_value(Some("both")).expect_err("写错的值拒");
    assert!(
        refused.contains(RUN_STAGE_ENVIRONMENT_VARIABLE) && refused.contains("both"),
        "{refused}"
    );
    assert!(RunStage::RecordOnly.records() && !RunStage::RecordOnly.judges());
    assert!(!RunStage::JudgeOnly.records() && RunStage::JudgeOnly.judges());
    assert!(RunStage::RecordThenJudge.records() && RunStage::RecordThenJudge.judges());
}
