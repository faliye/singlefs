use super::*;

/// 场景：调用方给这个进程的环境里设着多机配置在哪与服务分的卡（用例在头一个守卫里设上、照此刻的环境读成调用方那一份），
/// 第二个守卫把一趟读的变量全清掉，再照调用方那一份设回；另一份调用方两样都没设，同样设回。
/// 预期：设回之后两个变量就是调用方给的那两个值（仓副本不是 git 工作树，配置只经这个变量交进来：清掉不设回，副本里照调用方配置跑的
/// GPU 库单测找不到多机配置、在基线上就红，任务 7419）；调用方没设的两个都不设。
/// 判别力（`crates/mutations.tsv`「环境守卫不把调用方给的多机配置设回（副本里照调用方配置跑的用例找不到配置）」）：多机配置那一样不设回，这条红。
#[test]
fn the_environment_guard_sets_back_the_multi_host_configuration_and_the_cards_the_caller_handed_in()
{
    let configuration_of_the_caller = std::ffi::OsStr::new("/caller/handed/in/multi-host.env");
    let cards_of_the_caller = std::ffi::OsStr::new("192.0.2.81:0:15G");
    let caller_with_both = {
        let _environment_of_the_caller = CrashAmplificationEnvironment::cleared_then_set(&[
            (
                crate::gpu_card_configuration::MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
                configuration_of_the_caller,
            ),
            (
                crate::gpu_card_configuration::CARDS_BY_PRIORITY_KEY,
                cards_of_the_caller,
            ),
        ]);
        crate::gpu_card_configuration::GpuConfigurationOfTheCaller::read_from_the_environment_now()
    };
    let caller_with_neither = {
        let _environment_of_the_caller = CrashAmplificationEnvironment::cleared_then_set(&[]);
        crate::gpu_card_configuration::GpuConfigurationOfTheCaller::read_from_the_environment_now()
    };
    let read_back = || {
        (
            std::env::var_os(
                crate::gpu_card_configuration::MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
            ),
            std::env::var_os(crate::gpu_card_configuration::CARDS_BY_PRIORITY_KEY),
        )
    };
    {
        let mut environment = CrashAmplificationEnvironment::cleared_then_set(&[]);
        assert_eq!(read_back(), (None, None), "守卫把两个变量都清掉了");
        environment.set_the_multi_host_configuration_and_the_cards_of(&caller_with_both);
        assert_eq!(
            read_back(),
            (
                Some(configuration_of_the_caller.to_os_string()),
                Some(cards_of_the_caller.to_os_string())
            ),
            "设回调用方给的多机配置与卡"
        );
    }
    let mut environment = CrashAmplificationEnvironment::cleared_then_set(&[]);
    environment.set_the_multi_host_configuration_and_the_cards_of(&caller_with_neither);
    assert_eq!(read_back(), (None, None), "调用方两样都没设：一样都不设");
}

/// 单测里借卡通道的三个尝试变量取的值（认得出的字面）。
fn attempt_variables_of_a_card_lease_channel_for_the_unit_tests(
) -> Vec<(&'static str, &'static std::ffi::OsStr)> {
    crate::gpu_card_configuration::CARD_LEASE_ATTEMPT_ENVIRONMENT_VARIABLES
        .iter()
        .zip(["label-of-the-guard-test", "4093", "1"])
        .map(|(name, value)| (*name, std::ffi::OsStr::new(value)))
        .collect()
}

/// 借卡通道的三个尝试变量此刻在环境里的值（照 [`crate::gpu_card_configuration::CARD_LEASE_ATTEMPT_ENVIRONMENT_VARIABLES`] 的次序）。
fn attempt_variables_of_a_card_lease_channel_in_the_environment_now(
) -> Vec<Option<std::ffi::OsString>> {
    crate::gpu_card_configuration::CARD_LEASE_ATTEMPT_ENVIRONMENT_VARIABLES
        .iter()
        .map(std::env::var_os)
        .collect()
}

/// 场景：拿着改环境的锁，先把借卡通道的三个尝试变量设成认得出的字面（此刻环境里认得出通道，当调用方的通道），再照一趟读的全部变量
/// （[`ENVIRONMENT_VARIABLES_READ_BY_A_RUN`]，用例的环境守卫 [`CrashAmplificationEnvironment`] 清的就是这一批）记下原值、清掉，再丢掉。
/// 预期：清掉之后环境里认不出借卡通道，三个变量都不在；丢掉之后三个变量逐个是原值，又认得出通道。
/// 设调用方通道那一步只按三个变量的名字记原值、设值，不经那张表，表里少了它们这条照样跑到断言上。
/// 判别力（`crates/mutations.tsv`「环境守卫 不清借卡通道的三个尝试变量」）：表里没有这三个，守卫里照样认得出调用方的通道。
#[test]
fn the_environment_guard_hides_the_card_lease_channel_and_restores_its_attempt_variables() {
    let attempt_values = attempt_variables_of_a_card_lease_channel_for_the_unit_tests();
    let expected_after_the_guard: Vec<Option<std::ffi::OsString>> = attempt_values
        .iter()
        .map(|(_, value)| Some(value.to_os_string()))
        .collect();
    let _one_test_at_a_time = one_test_changes_the_environment_at_a_time();
    let mut channel_of_the_caller = EnvironmentRestoredOnDrop::saved_then_cleared(
        &crate::gpu_card_configuration::CARD_LEASE_ATTEMPT_ENVIRONMENT_VARIABLES,
    );
    for (name, value) in &attempt_values {
        channel_of_the_caller.set_variable(name, value);
    }
    assert!(
        crate::gpu_card_configuration::CardLeaseChannelOfTheCaller::read_from_the_environment_now()
            .is_some(),
        "前提：外层守卫设好之后环境里认得出借卡通道"
    );
    let guard_of_a_run =
        EnvironmentRestoredOnDrop::saved_then_cleared(ENVIRONMENT_VARIABLES_READ_BY_A_RUN);
    let channel_inside_the_guard =
        crate::gpu_card_configuration::CardLeaseChannelOfTheCaller::read_from_the_environment_now();
    let attempt_variables_inside_the_guard =
        attempt_variables_of_a_card_lease_channel_in_the_environment_now();
    drop(guard_of_a_run);
    assert_eq!(channel_inside_the_guard, None, "守卫里认不出借卡通道");
    assert_eq!(
        attempt_variables_inside_the_guard,
        vec![None, None, None],
        "守卫里三个尝试变量都不在"
    );
    assert_eq!(
        attempt_variables_of_a_card_lease_channel_in_the_environment_now(),
        expected_after_the_guard,
        "丢掉守卫之后三个尝试变量逐个是原值"
    );
}

/// 配置的值去掉一层引号（显卡优先级那个键照样从配置文件读得出）；键没写是 `None`；环境变量 `GPU_CARDS_BY_PRIORITY` 没设时不照配置文件里的卡挑卡。
/// 环境走 K5 那一个守卫：那个变量设了就照它，用例清掉它、跑完（或 panic）还原，不读调用方留下的值。
#[test]
fn the_card_priority_is_read_from_the_configuration_file_with_one_layer_of_quotes_removed() {
    let _environment = CrashAmplificationEnvironment::cleared_then_set(&[]);
    let directory = crate::crash_library::ExclusiveTemporaryDirectory::create(
        "crash-amplification-card-priority",
    )
    .expect("建得了一次性临时目录");
    let file = directory.join("multi-host.env");
    std::fs::write(
            &file,
            "ENABLE_GPU=1\nGPU_CARDS_BY_PRIORITY='127.0.0.1:0:15G 192.0.2.81:2:10G'\n\
             GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED='127.0.0.1:0:15G 192.0.2.81:2:10G'\nQUOTED=\"two words\"\n",
        )
        .expect("写得了");
    assert_eq!(
        configuration_value(&file, "GPU_CARDS_BY_PRIORITY").as_deref(),
        Some("127.0.0.1:0:15G 192.0.2.81:2:10G")
    );
    assert_eq!(
        configuration_value(&file, "QUOTED").as_deref(),
        Some("two words")
    );
    assert_eq!(
        configuration_value(&file, "ENABLE_GPU").as_deref(),
        Some("1")
    );
    assert_eq!(configuration_value(&file, "NOT_WRITTEN"), None);
    // 守卫清掉了那个变量：配置里写着卡也不照它挑卡，清没清场都报没经服务给卡那一句（gpu_card_configuration 那两条用例钉）；
    // 守卫留下一个写坏的值时报的是写法不对，不是这一句
    let refusal = crate::gpu_card_configuration::cards_by_priority_from(
        std::env::var("GPU_CARDS_BY_PRIORITY").ok(),
        Some(&file),
        crate::gpu_card_configuration::GpuQuiesce::QuiescedByTheWrapper,
        &crate::gpu_card_configuration::multi_host_addresses_for_the_unit_tests,
    )
    .expect_err("环境变量没设、配置开了 GPU：报错，不照配置文件里的卡挑卡");
    assert!(
        refusal.contains("没经服务给卡")
            && refusal.contains("出路：卡只归 GPU 批处理验证服务")
            && refusal.contains("验证包的测试与判段不借卡"),
        "报的是没经服务给卡那一句：{refusal}"
    );
    let without_the_key = directory.join("without-the-key.env");
    std::fs::write(&without_the_key, "ENABLE_GPU=0\n").expect("写得了");
    // GPU 没开、卡都没写：守卫清掉了那个变量就是没配
    assert_eq!(
        crate::gpu_card_configuration::cards_by_priority_from(
            std::env::var("GPU_CARDS_BY_PRIORITY").ok(),
            Some(&without_the_key),
            crate::gpu_card_configuration::GpuQuiesce::QuiescedByTheWrapper,
            &crate::gpu_card_configuration::multi_host_addresses_for_the_unit_tests
        ),
        Ok(None)
    );
    drop(directory);
}

/// 跑法变量只认 `pruned`、`item_by_item`：环境里写一个认不得的值（从没用过的名字）这个进程的跑法当场拒，不当剪枝；改写 `pruned` 是剪枝。
/// 环境走 K5 那一个守卫（用例设、跑完还原）。判别力（`crates/mutations.tsv`）：守卫设的值不进环境，这条红。
#[test]
fn the_run_kind_not_recognized_in_the_environment_is_refused_and_pruned_is_taken() {
    const RUN_KIND_VALUE_NEVER_USED: &str = "run_kind_value_never_used";
    let mut environment = CrashAmplificationEnvironment::cleared_then_set(&[(
        CRASH_LIBRARY_RUN_KIND_ENVIRONMENT_VARIABLE,
        std::ffi::OsStr::new(RUN_KIND_VALUE_NEVER_USED),
    )]);
    assert!(
        std::panic::catch_unwind(crate::verdict_store::RunKindOfTheJudgement::of_this_process)
            .is_err(),
        "跑法变量写 {RUN_KIND_VALUE_NEVER_USED} 认不得：这个进程的跑法当场拒"
    );
    environment.set_variable(
        CRASH_LIBRARY_RUN_KIND_ENVIRONMENT_VARIABLE,
        std::ffi::OsStr::new("pruned"),
    );
    assert_eq!(
        crate::verdict_store::RunKindOfTheJudgement::of_this_process(),
        crate::verdict_store::RunKindOfTheJudgement::Pruned,
        "跑法变量写 pruned 是剪枝"
    );
}

/// 覆盖行逐字是覆盖表脚本读的那一形：行首、四个字段的名字与次序、步骤写 `StepKind` 的成员名（七个成员逐个钉）。
/// 判别力（`crates/mutations.tsv`）：步骤改写成种类串（`StepKind::name` 的蛇形写法），这条红。
#[test]
fn a_coverage_line_reads_entry_step_segment_expansion_word_for_word() {
    let expanded = CrashFlowCoverageLine {
        entry_name: "publish_first_file".to_string(),
        step_kind: StepKind::RootRecordFua,
        segment_index: 3,
        treatment: SegmentTreatmentInTheRun::Expanded,
    };
    assert_eq!(
            expanded.text(),
            "CRASH_FLOW_COVERAGE entry=publish_first_file step=RootRecordFua segment=3 expansion=expanded"
        );
    let not_expanded = CrashFlowCoverageLine {
        entry_name: "unmount".to_string(),
        step_kind: StepKind::SystemConfigurationSlot,
        segment_index: 12,
        treatment: SegmentTreatmentInTheRun::NotExpanded,
    };
    assert_eq!(
            not_expanded.text(),
            "CRASH_FLOW_COVERAGE entry=unmount step=SystemConfigurationSlot segment=12 expansion=not_expanded"
        );
    assert_eq!(
        [
            StepKind::ZeroFill,
            StepKind::Discard,
            StepKind::UnitWrite,
            StepKind::JournalRecord,
            StepKind::RootRecordFua,
            StepKind::SystemConfigurationSlot,
            StepKind::Barrier,
        ]
        .map(step_kind_member_name),
        [
            "ZeroFill",
            "Discard",
            "UnitWrite",
            "JournalRecord",
            "RootRecordFua",
            "SystemConfigurationSlot",
            "Barrier",
        ],
        "步骤写枚举里声明的成员名"
    );
}

/// 一条流里三种没展开各有一段：归 ignore 的点的段报 `ignored_point`、按段登记的段报 `registered_segment`、写数超过展开上限的段报
/// `not_expanded`，照展开上限展开了的报 `expanded`。同一（入口，步骤，段）只一行（7 次单元写的登记段只一行）；入口取点名的第一截。
/// 判别力（`crates/mutations.tsv`）：不认 ignore 的点、不认按段登记、不去重，各红这一条。
#[test]
fn coverage_lines_name_the_ignored_point_the_registered_segment_and_the_segment_past_the_expansion_limit(
) {
    let (writes, segments) = flow_parts_of_unit_and_journal_writes(&[(1, 1), (7, 0), (2, 0)]);
    assert_eq!(
        segments.iter().map(Vec::len).collect::<Vec<_>>(),
        vec![2, 7, 2, 1, 2]
    );
    let base = MemoryPool {
        devices: BTreeMap::new(),
        device_size_in_bytes: 1 << 30,
    };
    let registered = vec![RegisteredSegment {
        segment_index: 1,
        write_count: 7,
        write_table_digest_text: "dd".repeat(DIGEST_BYTES),
    }];
    let limit = ExpandUpToWritesPerSegment(1);
    // 录入段那一份展开的形（`expansion_in_the_recording`）：登记的段照展开上限、超过的取甲二，别的段照展开上限
    let expansion_in_the_recording_of_the_flow = |segment_index: usize, segment: &[usize]| {
        if segment_index == 1 {
            expansion_of_a_registered_segment(segment, limit)
        } else {
            limit.expansion_of(segment)
        }
    };
    let setting_file = "crates/singlefs-verification/tests/common_crash_points/mod.rs";
    let flow = CrashFlow {
        base: &base,
        writes: &writes,
        segments: &segments,
        judged_root_index: 11,
        versions: &[],
        versions_without_a_file: &[],
        expansion: &expansion_in_the_recording_of_the_flow,
        crash_points: vec![
            CrashPointSpan {
                name: "acquire_instance_and_warm_up.data_unit+journal_record.1".to_string(),
                code_file_in_repository: setting_file.to_string(),
                writes: 0..2,
                ignored: true,
            },
            CrashPointSpan {
                name: "publish_first_file.data_unit.1".to_string(),
                code_file_in_repository: setting_file.to_string(),
                writes: 2..writes.len(),
                ignored: false,
            },
        ],
    };
    let texts: Vec<String> = crash_flow_coverage_lines(&flow, &registered)
        .iter()
        .map(CrashFlowCoverageLine::text)
        .collect();
    assert_eq!(
            texts,
            vec![
                "CRASH_FLOW_COVERAGE entry=acquire_instance_and_warm_up step=UnitWrite segment=0 expansion=ignored_point",
                "CRASH_FLOW_COVERAGE entry=acquire_instance_and_warm_up step=JournalRecord segment=0 expansion=ignored_point",
                "CRASH_FLOW_COVERAGE entry=publish_first_file step=UnitWrite segment=1 expansion=registered_segment",
                "CRASH_FLOW_COVERAGE entry=publish_first_file step=UnitWrite segment=2 expansion=not_expanded",
                "CRASH_FLOW_COVERAGE entry=publish_first_file step=RootRecordFua segment=3 expansion=expanded",
                "CRASH_FLOW_COVERAGE entry=publish_first_file step=SystemConfigurationSlot segment=4 expansion=not_expanded",
            ],
            "段 0 归 ignore 的点（照展开上限它本来也不展开，ignore 先认）；段 1 登记（7 写只取甲二）；段 2、段 4 两写超过展开上限 1；段 3 一写展开"
        );
}

/// L10：流水线单测起跑不碰全仓那一份库，空库起跑的单测在一次性库（临时 prune-server 里的一段名字空间）里判：起一趟给定状态表的单测，
/// 这一趟报的库是那一份一次性库，判出的结论在一次性库里、在同一份临时服务的全仓那一份（键原样那一段）里一块都没有。
/// 判别力（`crates/mutations.tsv`）：设了一次性库照旧开全仓那一份（`LibraryOpenInThisProcess::open_or_share` 不看这个进程设的一次性库），这条红。
#[test]
fn a_pipeline_unit_test_on_a_disposable_library_writes_no_row_into_the_one_library() {
    // 一次性库留着：钉设了一次性库就一行不写全仓那一份
    let disposable = crate::crash_library::DisposableLibrary::new("pipeline-unit-test")
        .unwrap_or_else(|refusal| panic!("{refusal}"));
    let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
    let configuration = disposable.directory().join("multi-host-on-the-cpu.env");
    std::fs::write(&configuration, "ENABLE_GPU=0\n").expect("写得了配置");
    let run = {
        let _environment = CrashAmplificationEnvironment::cleared_then_set(&[(
            "SINGLEFS_MULTI_HOST_CONFIG",
            configuration.as_os_str(),
        )]);
        let device = singlefs_core::address::DeviceIdentity(0);
        let base = MemoryPool::with_devices(&[device], 1 << 20);
        let writes = vec![RetainedWrite {
            device,
            kind: StepKind::UnitWrite,
            is_force_unit_access: false,
            offset: singlefs_core::address::DeviceOffsetInBytes(8192),
            contents: WrittenContents::Bytes(vec![7; 512]),
        }];
        let segments = vec![vec![0]];
        let engine = GivenStatesEngine::start_from_environment(
            GivenStatesItem {
                item_name: "pipeline-unit-test".to_string(),
            },
            &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
        )
        .expect("起得了给定状态表的引擎");
        engine
            .judge_the_given_states(&GivenStatesOfATable {
                table_name: "one-state".to_string(),
                base: &base,
                writes: &writes,
                segments: &segments,
                seams: &[],
                persisted_sets: vec![vec![true]],
            })
            .expect("判得了这张表");
        engine.finish()
    };
    let disposable_store = disposable
        .open_store(crate::verdict_store::PruneServerFailureHandling::ReturnTheError)
        .expect("开得了一次性库");
    assert_eq!(
        run.library_name,
        disposable_store.library_name(),
        "判在一次性的库里"
    );
    let conclusions_of_the_run = disposable_store
        .stored_conclusion_keys()
        .expect("列得出一次性库里的结论");
    assert!(
        !conclusions_of_the_run.is_empty(),
        "这一趟判出的结论在一次性库里"
    );
    let the_one_library_of_the_temporary_service = VerdictStore::open_the_prune_server(
        crate::verdict_store::PruneServerStorage::connect(
            disposable.route().clone(),
            disposable.directory().join("the-one-library.journal"),
            crate::verdict_store::PruneServerFailureHandling::ReturnTheError,
            crate::verdict_store::LibraryKeySpace::TheOneLibrary,
        )
        .expect("连得上临时服务"),
    )
    .expect("开得了临时服务里全仓那一段");
    for (block_key, kind, version) in &conclusions_of_the_run {
        assert!(
            the_one_library_of_the_temporary_service
                .read_conclusion_block(block_key, kind, version)
                .expect("读得了全仓那一段")
                .is_none(),
            "一次性库这一趟的结论不落进全仓那一份：{}",
            block_key.short_text()
        );
    }
}
