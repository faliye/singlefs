use super::*;

/// 审计复核 T01（checkpoint_txg「存完整 u64」）：同一个实例、checkpoint_txg 低 32 位相同（0、1、7、`u32::MAX`）、
/// 高 32 位不同（0 对 1、2、`0xFFFF_FFFF`）的两条根，CPU 换出的恢复摘要不同、摊出的字不同，摘要换得回完整的 checkpoint_txg、
/// 摊出的字换得回同一份摘要。
/// 记下见过的内容的那张表交回的摘要与每次现算的逐字节相同：没见过的、再来一次的、同样长而差一个字节的（差在头、在尾各一份）、空的。
#[cfg(feature = "gpu")]
#[test]
fn a_table_waits_outside_while_as_many_tables_as_allowed_are_being_built_and_enters_once_one_leaves(
) {
    use super::given_states_tables_in_flight_on_the_gpu_route::TablesBuiltAtOnce;
    use std::sync::atomic::{AtomicBool, Ordering};
    let gate = TablesBuiltAtOnce::new();
    let first = gate.enter(2);
    let second = gate.enter(2);
    let third_entered = AtomicBool::new(false);
    std::thread::scope(|scope| {
        scope.spawn(|| {
            let _third = gate.enter(2);
            third_entered.store(true, Ordering::SeqCst);
        });
        std::thread::sleep(std::time::Duration::from_millis(300));
        assert!(
            !third_entered.load(Ordering::SeqCst),
            "已有两张在里面、只许两张：第三张要在外面等"
        );
        drop(first);
    });
    assert!(
        third_entered.load(Ordering::SeqCst),
        "出去一张之后等着的那一张进得来"
    );
    drop(second);
}

/// 给定状态引擎上卡判那一段限流数的卡数（`CardSourcesOfTheEngineStart::card_count_without_taking_a_card`）照起步时记下的显卡来路取：
/// 卡池起好了取池里的张数（来路不论）；没起时服务交的卡串取项数，没交卡串与空白卡串都取 0（判段不借卡，有没有借卡通道都一样，
/// 那一样钉在 [`the_given_states_engine_asks_no_card_lease_channel_and_refuses_as_without_one`]）。
#[cfg(feature = "gpu")]
#[test]
fn the_cards_of_the_given_states_engine_are_counted_from_the_card_sources_of_its_start() {
    let sources = |cards_handed_out_by_the_service: Option<&str>| CardSourcesOfTheEngineStart {
        configuration_file: PathBuf::from("multi-host-configuration-never-read.env"),
        cards_handed_out_by_the_service: cards_handed_out_by_the_service.map(str::to_string),
        quiesce: crate::gpu_card_configuration::GpuQuiesce::NotQuiesced,
    };
    let two_cards_handed_out = "192.0.2.1:0:32G 192.0.2.1:1:24G";
    assert_eq!(
        [
            sources(Some(two_cards_handed_out)).card_count_without_taking_a_card(None),
            sources(Some("  ")).card_count_without_taking_a_card(None),
            sources(None).card_count_without_taking_a_card(None),
        ],
        [2, 0, 0],
        "卡池没起：卡串取项数、空白卡串与没交卡串都取 0"
    );
    assert_eq!(
        [
            sources(Some(two_cards_handed_out)).card_count_without_taking_a_card(Some(1)),
            sources(Some("  ")).card_count_without_taking_a_card(Some(3)),
            sources(None).card_count_without_taking_a_card(Some(2)),
        ],
        [1, 3, 2],
        "卡池起好了：来路不论，取池里的张数"
    );
}

/// 场景：多机配置开着 GPU、环境里没有卡串，给定状态引擎起步时记显卡来路（[`CardSourcesOfTheEngineStart::read_from_the_environment_now`]），
/// 一回此刻环境里有借卡通道的三个尝试变量（任务服务交仓根的那个变量指到临时目录里的假仓，假客户端 `lease take` 退 3、每回调用记一行），
/// 一回没有；两回都照记下的来路取卡（第一张表上卡时取的那一下，`CardSourcesOfTheEngineStart::cards_by_priority`）、数限流张数。
/// 预期：有通道的那一回不起借卡客户端（调用记录一行都没有），取卡的结局与没有通道的那一回逐字相同（没经服务给卡那一句拒绝），
/// 卡池没起时限流张数两回都是 0。
/// 判别力（`crates/mutations.tsv`「判段取卡 给定状态引擎上卡时照借卡通道借」「给定状态引擎 限流数卡没卡串时数成 1 张」）。
#[cfg(feature = "gpu")]
#[test]
fn the_given_states_engine_asks_no_card_lease_channel_and_refuses_as_without_one() {
    let directory = crate::crash_library::ExclusiveTemporaryDirectory::create(
        "given-states-engine-asks-no-card-lease-channel",
    )
    .expect("建得了一次性临时目录");
    let configuration = directory.join("gpu-switched-on.env");
    std::fs::write(
        &configuration,
        format!("{}=1\n", crate::gpu_card_configuration::GPU_SWITCH_KEY),
    )
    .expect("写得了");
    let calls = directory.join("calls.log");
    let fake_repository = directory.join("fake-repository");
    let client = fake_repository
        .join(crate::gpu_card_configuration::CARD_LEASE_CLIENT_PATH_IN_THE_REPOSITORY);
    std::fs::create_dir_all(client.parent().expect("客户端在假仓的一个目录下"))
        .expect("建得了假客户端的目录");
    std::fs::write(
            &client,
            format!(
                "import sys\nwith open('{}', 'a') as calls:\n    calls.write(' '.join(sys.argv[1:]) + '\\n')\n\
                 sys.exit(3 if sys.argv[1:3] == ['lease', 'take'] else 0)\n",
                calls.display()
            ),
        )
        .expect("写得了假客户端");
    let outcome_of_the_start = |attempt_values: &[(&str, &std::ffi::OsStr)]| {
        let _environment = CrashAmplificationEnvironment::cleared_then_set(attempt_values);
        let mut source_repository = EnvironmentRestoredOnDrop::saved_then_cleared(&[
            crate::gpu_card_configuration::TASK_SERVER_SOURCE_REPOSITORY_ENVIRONMENT_VARIABLE,
        ]);
        source_repository.set_variable(
            crate::gpu_card_configuration::TASK_SERVER_SOURCE_REPOSITORY_ENVIRONMENT_VARIABLE,
            fake_repository.as_os_str(),
        );
        let channel_is_in_the_environment =
                crate::gpu_card_configuration::CardLeaseChannelOfTheCaller::read_from_the_environment_now()
                    .is_some();
        let sources =
            CardSourcesOfTheEngineStart::read_from_the_environment_now(configuration.clone());
        (
            channel_is_in_the_environment,
            sources.cards_by_priority(),
            sources.card_count_without_taking_a_card(None),
        )
    };
    let attempt_values: Vec<(&str, &std::ffi::OsStr)> =
        crate::gpu_card_configuration::CARD_LEASE_ATTEMPT_ENVIRONMENT_VARIABLES
            .iter()
            .zip(["label-of-the-engine-test", "4093", "1"])
            .map(|(name, value)| (*name, std::ffi::OsStr::new(value)))
            .collect();
    let (channel_was_there, cards_with_the_channel, count_with_the_channel) =
        outcome_of_the_start(&attempt_values);
    let (no_channel_was_there, cards_without_the_channel, count_without_the_channel) =
        outcome_of_the_start(&[]);
    assert!(
        channel_was_there && !no_channel_was_there,
        "前提：一回环境里认得出借卡通道、一回认不出"
    );
    assert!(
        !calls.exists(),
        "给定状态引擎取卡起了借卡客户端：{}",
        std::fs::read_to_string(&calls).unwrap_or_default()
    );
    assert_eq!(
        cards_with_the_channel, cards_without_the_channel,
        "有没有借卡通道，给定状态引擎取卡的结局逐字相同"
    );
    assert!(
        cards_without_the_channel
            .as_ref()
            .is_err_and(|refusal| refusal.contains("没经服务给卡")),
        "两回都是没经服务给卡那一句拒绝：{cards_without_the_channel:?}"
    );
    assert_eq!(
        (count_with_the_channel, count_without_the_channel),
        (0, 0),
        "卡池没起、没交卡串：限流张数两回都是 0"
    );
    drop(directory);
}

/// CPU 参照那一份结论字（给定状态表那一种）：CPU 判器判一个状态的对拍字与点名的被抛弃根照结论字的写法摆回去
/// （第 0 字红位、违例位与判到位各四字、两遍落到的根各四字、两遍的结局与失败点、I-1.13 的结局码、点名的被抛弃根四字），别的字全 0。
/// 判不了那一位 CPU 那一侧恒为假；点名的被抛弃根至多一条（走读内核只记一条）。
#[cfg(feature = "gpu")]
fn verdict_words_of_the_cross_check_words(
    words: CrossCheckWords,
    abandoned_roots_named: &[AbandonedRootNamedByThePoolChecker],
) -> [u32; crate::crash_judge_gpu::VERDICT_WORDS] {
    use crate::crash_judge_gpu::{
        ABANDONED_ROOT_NAMED, ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_CHECKPOINT_TXG_HIGH_WORD,
        ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_CHECKPOINT_TXG_LOW_WORD,
        ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_INSTANCE_WORD,
        ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_WORD, CONSULTED_PASS_FAILURE_POINT_WORD,
        CONSULTED_PASS_LANDED_CHECKPOINT_TXG_HIGH_WORD,
        CONSULTED_PASS_LANDED_CHECKPOINT_TXG_LOW_WORD, CONSULTED_PASS_LANDED_INSTANCE_WORD,
        CONSULTED_PASS_LANDED_MARKER_WORD, CONSULTED_PASS_OUTCOME_WORD,
        IGNORED_PASS_FAILURE_POINT_WORD, IGNORED_PASS_LANDED_CHECKPOINT_TXG_HIGH_WORD,
        IGNORED_PASS_LANDED_CHECKPOINT_TXG_LOW_WORD, IGNORED_PASS_LANDED_INSTANCE_WORD,
        IGNORED_PASS_LANDED_MARKER_WORD, IGNORED_PASS_OUTCOME_WORD, NO_ABANDONED_ROOT_NAMED,
        POOL_CHECKER_JUDGED_FIRST_WORD, POOL_CHECKER_VIOLATED_FIRST_WORD,
        POOL_CHECKER_VIOLATED_WORDS, SCAN_REBUILD_OUTCOME_WORD, VERDICT_WORDS,
    };
    let CrossCheckWords {
        red_bits,
        pool_checker_violated_bits,
        consulted_pass_landed_root,
        ignored_pass_landed_root,
        consulted_pass_outcome,
        ignored_pass_outcome,
        consulted_pass_failure_point,
        ignored_pass_failure_point,
        pool_checker_judged_bits,
        scan_rebuild_outcome,
        gpu_could_not_judge,
    } = words;
    assert!(!gpu_could_not_judge, "CPU 判器的对拍字没有判不了那一位");
    let low_word = |bits: u128, word_index: usize| {
        u32::try_from((bits >> (32 * word_index)) & u128::from(u32::MAX)).expect("取的是低 32 位")
    };
    let mut verdict = [0u32; VERDICT_WORDS];
    verdict[0] = red_bits;
    for word_index in 0..POOL_CHECKER_VIOLATED_WORDS {
        verdict[POOL_CHECKER_VIOLATED_FIRST_WORD + word_index] =
            low_word(pool_checker_violated_bits, word_index);
        verdict[POOL_CHECKER_JUDGED_FIRST_WORD + word_index] =
            low_word(pool_checker_judged_bits, word_index);
    }
    verdict[CONSULTED_PASS_LANDED_MARKER_WORD] = consulted_pass_landed_root.landed_marker;
    verdict[CONSULTED_PASS_LANDED_INSTANCE_WORD] = consulted_pass_landed_root.instance;
    verdict[CONSULTED_PASS_LANDED_CHECKPOINT_TXG_LOW_WORD] =
        consulted_pass_landed_root.checkpoint_txg_low;
    verdict[CONSULTED_PASS_LANDED_CHECKPOINT_TXG_HIGH_WORD] =
        consulted_pass_landed_root.checkpoint_txg_high;
    verdict[IGNORED_PASS_LANDED_MARKER_WORD] = ignored_pass_landed_root.landed_marker;
    verdict[IGNORED_PASS_LANDED_INSTANCE_WORD] = ignored_pass_landed_root.instance;
    verdict[IGNORED_PASS_LANDED_CHECKPOINT_TXG_LOW_WORD] =
        ignored_pass_landed_root.checkpoint_txg_low;
    verdict[IGNORED_PASS_LANDED_CHECKPOINT_TXG_HIGH_WORD] =
        ignored_pass_landed_root.checkpoint_txg_high;
    verdict[CONSULTED_PASS_OUTCOME_WORD] = consulted_pass_outcome;
    verdict[IGNORED_PASS_OUTCOME_WORD] = ignored_pass_outcome;
    verdict[CONSULTED_PASS_FAILURE_POINT_WORD] = consulted_pass_failure_point;
    verdict[IGNORED_PASS_FAILURE_POINT_WORD] = ignored_pass_failure_point;
    verdict[SCAN_REBUILD_OUTCOME_WORD] = scan_rebuild_outcome;
    match abandoned_roots_named {
        [] => verdict[ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_WORD] = NO_ABANDONED_ROOT_NAMED,
        [root] => {
            verdict[ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_WORD] = ABANDONED_ROOT_NAMED;
            verdict[ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_INSTANCE_WORD] = root.instance.0;
            verdict[ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_CHECKPOINT_TXG_LOW_WORD] =
                low_word(u128::from(root.checkpoint_txg.0), 0);
            verdict[ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_CHECKPOINT_TXG_HIGH_WORD] =
                low_word(u128::from(root.checkpoint_txg.0), 1);
        }
        [_, _, ..] => panic!(
            "前提：走读内核只记一条点名的被抛弃根，CPU 判器这个状态点名了 {} 条",
            abandoned_roots_named.len()
        ),
    }
    verdict
}

/// 场景：配了 GPU（多机配置只写 `ENABLE_GPU=1`）、环境里没有服务交的卡；给定状态引擎判一张两个状态的表（做完 mkfs 的两块盘、一次系统配置槽写，
/// 一个状态持久它、一个不持久），取卡没成的那一张交替身服务判（[`packed_tables_stand_in`]：结论字取 CPU 参照那一份，CPU 判器在这张表上逐状态
/// 判出的对拍字与点名的被抛弃根照结论字的写法摆回去，[`verdict_words_of_the_cross_check_words`]）。
/// 预期：引擎交回的每个状态的对拍字、点名的被抛弃根与计数向量与 CPU 判器的逐条相同（引擎解结论字的那一段就是本进程上卡那一支解字的那一段）；
/// 这一趟 GPU 判了两个状态、一张表、没取到卡；替身读到的判表待判行恰好一行：态数 2、不强制、旋钮是这张表那一栏（产品那一份），
/// 键里内核源码那一段是照这一栏取的那三份的摘要；判完之后那一行判表待判行不在、判表结论字在。
/// 判别力（`crates/mutations.tsv`）：取卡没成就报错交回（不交服务判），这条红。
#[cfg(feature = "gpu")]
#[test]
fn a_given_states_table_without_a_card_is_judged_through_the_gpu_verification_server() {
    // 一次性库留着：引擎录进去的表与替身写回的结论字从空库起，不写全仓那一份
    let disposable = crate::crash_library::DisposableLibrary::new(
        "given-states-table-through-the-gpu-verification-server",
    )
    .unwrap_or_else(|refusal| panic!("{refusal}"));
    let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
    let configuration = disposable.directory().join("multi-host-with-the-gpu.env");
    std::fs::write(&configuration, "ENABLE_GPU=1\n").expect("写得了配置");
    let base = a_formatted_pool_for_the_judge_tables_of_the_observer_images();
    let writes = vec![a_system_configuration_slot_write(1, 0x5A)];
    let segments = vec![vec![0]];
    let table = GivenStatesOfATable {
        table_name: "two-states-through-the-service".to_string(),
        base: &base,
        writes: &writes,
        segments: &segments,
        seams: &[],
        persisted_sets: vec![vec![true], vec![false]],
    };
    let cpu = judge_the_given_states_on_the_cpu(&table);
    let words_of_the_cpu: Vec<[u32; crate::crash_judge_gpu::VERDICT_WORDS]> = cpu
        .states
        .iter()
        .map(|state| {
            verdict_words_of_the_cross_check_words(
                state.words,
                &state.abandoned_roots_named_by_the_pool_checker,
            )
        })
        .collect();
    for (ordinal, (state, words)) in cpu.states.iter().zip(&words_of_the_cpu).enumerate() {
        assert_eq!(
            crate::crash_judge_conclusions::cross_check_words_without_the_oracle(
                crate::crash_judge_gpu::cross_check_words_of_gpu_verdict(words)
            ),
            state.words,
            "前提：摆回去的结论字照 GPU 那一侧解，第 {ordinal} 个状态得回 CPU 判器那一份对拍字"
        );
    }
    let rows_seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let stand_in = packed_tables_stand_in(
        {
            let words_of_the_cpu = words_of_the_cpu.clone();
            move |_key, _row| words_of_the_cpu.clone()
        },
        std::sync::Arc::clone(&rows_seen),
    );
    let environment = CrashAmplificationEnvironment::cleared_then_set(&[(
        "SINGLEFS_MULTI_HOST_CONFIG",
        configuration.as_os_str(),
    )]);
    let engine = GivenStatesEngine::start_from_environment(
        GivenStatesItem {
            item_name: "given-states-table-through-the-gpu-verification-server".to_string(),
        },
        &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
    )
    .expect("起得了给定状态表的引擎")
    .going_to_the_gpu_verification_server_through_the_stand_in(stand_in);
    let conclusions = engine
        .judge_the_given_states(&table)
        .expect("取卡没成的那张表交替身服务判");
    let rows_seen = rows_seen
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    assert_eq!(
        rows_seen.len(),
        1,
        "替身读到恰好一行判表待判行：{rows_seen:?}"
    );
    let (key, row) = rows_seen[0];
    let (pending_row_after, words_after) = {
        let store = engine.store();
        (
            store
                .read_packed_tables_pending_row(&key)
                .expect("读得了库"),
            store
                .read_packed_tables_verdict_words(&key)
                .expect("读得了库"),
        )
    };
    let run = engine.finish();
    drop(environment);
    assert_eq!(
        conclusions
            .states
            .iter()
            .map(|state| (
                state.words,
                state.abandoned_roots_named_by_the_pool_checker.clone()
            ))
            .collect::<Vec<_>>(),
        cpu.states
            .iter()
            .map(|state| (
                state.words,
                state.abandoned_roots_named_by_the_pool_checker.clone()
            ))
            .collect::<Vec<_>>(),
        "每个状态的对拍字与点名的被抛弃根与 CPU 判器的逐条相同"
    );
    assert_eq!(
        conclusions.count_vector, cpu.count_vector,
        "计数向量与 CPU 判器的相同"
    );
    assert_eq!(
        (
            run.tally.states_judged_by_the_gpu_here,
            run.tally.tables_judged_here,
            run.gpu_cards_taken_from
        ),
        (2, 1, None),
        "GPU 判了两个状态、一张表，没取到卡（{:?}）",
        run.tally
    );
    assert_eq!(
        (
            row.container_capacity_knobs,
            row.state_count,
            row.forced,
            key.container_capacity_knobs_digest
        ),
        (
            ContainerCapacityKnobs::PRODUCT,
            2,
            crate::verdict_store::ForcedRejudging::NotForced,
            container_capacity_knobs_digest(ContainerCapacityKnobs::PRODUCT)
        ),
        "判表待判行：这张表那一栏旋钮、两个状态、不强制，键里是这一栏旋钮的摘要"
    );
    assert_eq!(
        (
            pending_row_after,
            words_after.map(|words| words.verdict_words)
        ),
        (None, Some(words_of_the_cpu)),
        "判完之后判表待判行不在、判表结论字在"
    );
}

/// CPU 判器判这张给定状态表、逐状态照结论字的写法摆回去的那一份结论字（[`verdict_words_of_the_cross_check_words`]）：替身服务照它交回。
#[cfg(feature = "gpu")]
fn cpu_verdict_words_of_the_given_states(
    table: &GivenStatesOfATable<'_>,
) -> Vec<[u32; crate::crash_judge_gpu::VERDICT_WORDS]> {
    judge_the_given_states_on_the_cpu(table)
        .states
        .iter()
        .map(|state| {
            verdict_words_of_the_cross_check_words(
                state.words,
                &state.abandoned_roots_named_by_the_pool_checker,
            )
        })
        .collect()
}

/// 场景：同 [`a_given_states_table_without_a_card_is_judged_through_the_gpu_verification_server`] 那一张两个状态的表、那一份配置（没交卡），
/// 引擎上卡判那一段同时建判表、等卡的表那一道换成这条用例自己的（[`GivenStatesEngine::limiting_its_tables_built_at_once_with`]）；
/// 替身服务（[`packed_tables_stand_in`]）每走一圈现算之前先数那一道里此刻有几张表。
/// 预期：表判完；替身至少走了一圈，每一圈那一道里都是 0 张（等服务那一段不占那一份额，判表已在库里）；判完那一道里也是 0 张。
/// 判别力（`crates/mutations.tsv`）：等服务之前不放那一份额（一直占到判完），这条红。
#[cfg(feature = "gpu")]
#[test]
fn a_given_states_table_waiting_for_the_gpu_verification_server_takes_no_share_of_the_tables_built_at_once(
) {
    use super::given_states_tables_in_flight_on_the_gpu_route::TablesBuiltAtOnce;
    static TABLES_BUILT_AT_ONCE_OF_THE_TEST: TablesBuiltAtOnce = TablesBuiltAtOnce::new();
    // 一次性库留着：引擎录进去的表与替身写回的结论字从空库起，不写全仓那一份
    let disposable = crate::crash_library::DisposableLibrary::new(
        "given-states-table-waiting-takes-no-share-of-the-tables-built-at-once",
    )
    .unwrap_or_else(|refusal| panic!("{refusal}"));
    let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
    let configuration = disposable.directory().join("multi-host-with-the-gpu.env");
    std::fs::write(&configuration, "ENABLE_GPU=1\n").expect("写得了配置");
    let base = a_formatted_pool_for_the_judge_tables_of_the_observer_images();
    let writes = vec![a_system_configuration_slot_write(1, 0x5A)];
    let segments = vec![vec![0]];
    let table = GivenStatesOfATable {
        table_name: "two-states-waiting-for-the-service".to_string(),
        base: &base,
        writes: &writes,
        segments: &segments,
        seams: &[],
        persisted_sets: vec![vec![true], vec![false]],
    };
    let words_of_the_cpu = cpu_verdict_words_of_the_given_states(&table);
    let judging_stand_in = packed_tables_stand_in(
        move |_key, _row| words_of_the_cpu.clone(),
        std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
    );
    let tables_inside_at_each_reckoning = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let stand_in = std::sync::Arc::new(GpuVerificationServerStandIn {
        between_two_reckonings: Box::new({
            let tables_inside_at_each_reckoning =
                std::sync::Arc::clone(&tables_inside_at_each_reckoning);
            move |library: &LibraryTakenAtEachReadOrWrite<'_>| {
                tables_inside_at_each_reckoning
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push(TABLES_BUILT_AT_ONCE_OF_THE_TEST.tables_inside_now());
                (judging_stand_in.between_two_reckonings)(library);
            }
        }),
    });
    let environment = CrashAmplificationEnvironment::cleared_then_set(&[(
        "SINGLEFS_MULTI_HOST_CONFIG",
        configuration.as_os_str(),
    )]);
    let engine = GivenStatesEngine::start_from_environment(
        GivenStatesItem {
            item_name: "given-states-table-waiting-takes-no-share".to_string(),
        },
        &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
    )
    .expect("起得了给定状态表的引擎")
    .going_to_the_gpu_verification_server_through_the_stand_in(stand_in)
    .limiting_its_tables_built_at_once_with(&TABLES_BUILT_AT_ONCE_OF_THE_TEST);
    let judged = engine.judge_the_given_states(&table);
    let _run = engine.finish();
    drop(environment);
    judged.expect("取卡没成的那张表交替身服务判");
    let tables_inside_at_each_reckoning = tables_inside_at_each_reckoning
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    assert!(
        !tables_inside_at_each_reckoning.is_empty()
            && tables_inside_at_each_reckoning
                .iter()
                .all(|tables| *tables == 0),
        "替身走过的每一圈，那一道里都没有这张表：{tables_inside_at_each_reckoning:?}"
    );
    assert_eq!(
        TABLES_BUILT_AT_ONCE_OF_THE_TEST.tables_inside_now(),
        0,
        "判完那一道里没有表"
    );
}

/// 场景：同 [`a_given_states_table_without_a_card_is_judged_through_the_gpu_verification_server`] 那一张两个状态的表、那一份配置（没交卡），
/// 表的旋钮栏压了墓碑树（每片 2 条，另两棵没装），经带两栏的入口判（[`GivenStatesJudge::judge_the_given_states_with_its_columns`]）。
/// 预期：表判完；替身读到恰好一行判表待判行，旋钮栏是这张表那一栏，键里内核源码那一段是照这一栏取的那三份的摘要（与产品那一份的不同）。
/// 判别力（`crates/mutations.tsv`）：交服务判时旋钮照产品那一份取（不照这张表那一栏），这条红。
#[cfg(feature = "gpu")]
#[test]
fn a_given_states_table_with_its_knobs_is_handed_to_the_gpu_verification_server_with_the_kernel_sources_of_its_knobs(
) {
    let disposable = crate::crash_library::DisposableLibrary::new(
        "given-states-table-with-its-knobs-through-the-gpu-verification-server",
    )
    .unwrap_or_else(|refusal| panic!("{refusal}"));
    let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
    let configuration = disposable.directory().join("multi-host-with-the-gpu.env");
    std::fs::write(&configuration, "ENABLE_GPU=1\n").expect("写得了配置");
    let base = a_formatted_pool_for_the_judge_tables_of_the_observer_images();
    let writes = vec![a_system_configuration_slot_write(1, 0x5A)];
    let segments = vec![vec![0]];
    let table = GivenStatesOfATable {
        table_name: "two-states-with-its-knobs-through-the-service".to_string(),
        base: &base,
        writes: &writes,
        segments: &segments,
        seams: &[],
        persisted_sets: vec![vec![true], vec![false]],
    };
    let knobs_of_the_table = ContainerCapacityKnobs {
        tombstone: Some(
            singlefs_checker::walk::TombstoneContainerCapacityKnob::new(2)
                .expect("墓碑树每片 2 条在取值范围里"),
        ),
        lineage: None,
        retirement: None,
    };
    let digest_of_the_knobs = container_capacity_knobs_digest(knobs_of_the_table);
    assert_ne!(
        digest_of_the_knobs,
        container_capacity_knobs_digest(ContainerCapacityKnobs::PRODUCT),
        "前提：压了旋钮的那一组与产品那一组的摘要不同"
    );
    let words_of_the_cpu = cpu_verdict_words_of_the_given_states(&table);
    let rows_seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let stand_in = packed_tables_stand_in(
        move |_key, _row| words_of_the_cpu.clone(),
        std::sync::Arc::clone(&rows_seen),
    );
    let environment = CrashAmplificationEnvironment::cleared_then_set(&[(
        "SINGLEFS_MULTI_HOST_CONFIG",
        configuration.as_os_str(),
    )]);
    let engine = GivenStatesEngine::start_from_environment(
        GivenStatesItem {
            item_name: "given-states-table-with-its-knobs-through-the-service".to_string(),
        },
        &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
    )
    .expect("起得了给定状态表的引擎")
    .going_to_the_gpu_verification_server_through_the_stand_in(stand_in);
    let judged = engine.judge_the_given_states_with_its_columns(
            GivenStatesOfATableWithItsColumns {
                table: &table,
                container_capacity_knobs: knobs_of_the_table,
                claimed_versions:
                    crate::crash_given_states::ClaimedVersionsOfTheStates::EveryStateLandedOnByTheConsultedRecovery,
            },
        );
    let _run = engine.finish();
    drop(environment);
    judged.expect("取卡没成的那张表交替身服务判");
    let rows_seen = rows_seen
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    assert_eq!(
        rows_seen
            .iter()
            .map(|(key, row)| (
                row.container_capacity_knobs,
                key.container_capacity_knobs_digest
            ))
            .collect::<Vec<_>>(),
        vec![(knobs_of_the_table, digest_of_the_knobs)],
        "替身读到恰好一行判表待判行：旋钮栏是这张表那一栏，键里是这一栏旋钮的摘要"
    );
}

/// 给定状态引擎认选段：`record` 那一趟只把表按内容摘要写进库、交占位结论（一张不判）；`judge` 那一趟照同一张表现算摘要、
/// 从库里读回判；调用方交来一张录入段没录的表，判段拒（先跑录入段），不判内存里的那一张；两段先后跑的那一趟录入键命中不再录、照库里判过的复用。
/// 这张表叠在没做过 mkfs 的盘上，CPU 判器判红；照「记红、撤绿、下趟现判」，记着红的表下一趟现判、不复用
/// （那一面由 `a_given_states_table_stored_red_is_judged_again_and_the_new_conclusion_replaces_it` 钉）。复用那一问挪到不红的表上：
/// 两段先后跑之前，把库里这张表记着红的每一块换成不红的一份（逐状态取空的判定向量、没有违例，计数向量、恢复字、点名的被抛弃根照原样）。
#[test]
fn the_given_states_engine_records_only_in_the_record_stage_and_judges_the_table_read_back_in_the_judge_stage(
) {
    // 一次性库留着：给定状态引擎两段的自证，要从空库起跑；改写库里的结论不许进全仓那一份
    let disposable = crate::crash_library::DisposableLibrary::new("given-states-engine-stages")
        .unwrap_or_else(|refusal| panic!("{refusal}"));
    let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
    let configuration = disposable.directory().join("multi-host-on-the-cpu.env");
    std::fs::write(&configuration, "ENABLE_GPU=0\n").expect("写得了配置");
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
    let run_in_the_stage = |stage: Option<&str>, persisted_sets: Vec<Vec<bool>>| {
        let mut settings: Vec<(&str, &std::ffi::OsStr)> =
            vec![("SINGLEFS_MULTI_HOST_CONFIG", configuration.as_os_str())];
        if let Some(stage) = stage {
            settings.push((RUN_STAGE_ENVIRONMENT_VARIABLE, std::ffi::OsStr::new(stage)));
        }
        let _environment = CrashAmplificationEnvironment::cleared_then_set(&settings);
        let engine = GivenStatesEngine::start_from_environment(
            GivenStatesItem {
                item_name: "given-states-engine-stages".to_string(),
            },
            &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
        )
        .expect("起得了给定状态表的引擎");
        let conclusions = engine.judge_the_given_states(&GivenStatesOfATable {
            table_name: "two-states".to_string(),
            base: &base,
            writes: &writes,
            segments: &segments,
            seams: &[],
            persisted_sets,
        });
        (conclusions, engine.finish())
    };
    let (recorded, record_run) = run_in_the_stage(Some("record"), vec![vec![true], vec![false]]);
    assert_eq!(
        recorded.expect("只录那一趟交占位结论").source,
        GivenStatesConclusionsSource::PlaceholderOfATableRecordedOnly
    );
    assert!(record_run.recorded_only());
    assert_eq!(
        (
            record_run.tally.tables_recorded_here,
            record_run.tally.tables_recorded_only,
            record_run.tally.tables_judged_here
        ),
        (1, 1, 0),
        "只录那一趟：表写进库，一张不判"
    );
    let (judged, judge_run) = run_in_the_stage(Some("judge"), vec![vec![true], vec![false]]);
    assert_eq!(
        judged.expect("只判那一趟照库里那一张判").states.len(),
        2,
        "只判那一趟判的是库里录的那一张"
    );
    assert_eq!(
        (
            judge_run.tally.tables_recorded_here,
            judge_run.tally.tables_judged_here,
            judge_run.tally.tables_reused
        ),
        (0, 1, 0),
        "只判那一趟不录、判一张"
    );
    let (not_recorded, _) = run_in_the_stage(Some("judge"), vec![vec![true]]);
    assert!(
        not_recorded
            .as_ref()
            .is_err_and(|refusal| refusal.contains("先跑录入段")),
        "录入段没录过的那一份内容，判段拒、不判内存里的表：{:?}",
        not_recorded.map(|conclusions| conclusions.states.len())
    );
    {
        let _environment = CrashAmplificationEnvironment::cleared_then_set(&[(
            "SINGLEFS_MULTI_HOST_CONFIG",
            configuration.as_os_str(),
        )]);
        let engine = GivenStatesEngine::start_from_environment(
            GivenStatesItem {
                item_name: "given-states-engine-stages".to_string(),
            },
            &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
        )
        .expect("起得了给定状态表的引擎");
        let table = GivenStatesOfATable {
            table_name: "two-states".to_string(),
            base: &base,
            writes: &writes,
            segments: &segments,
            seams: &[],
            persisted_sets: vec![vec![true], vec![false]],
        };
        let blocks = GivenStatesEngine::blocks_of_the_table((&table).into());
        let version = engine.version_of(JudgedBy::CpuJudge);
        let kind = JudgedBy::CpuJudge.conclusion_kind();
        {
            let mut store = engine.store();
            let green_verdict = store
                .register_verdict_vector(&encoded_verdict_of_the_red_items(&[]))
                .expect("登记得了");
            for block in &blocks {
                let stored = store
                    .read_conclusion_block(&block.key, &kind, &version)
                    .expect("库读得了")
                    .expect("只判那一趟存过这张表每一块的结论");
                if stored.violations.is_empty() {
                    continue;
                }
                let abandoned_roots_named = stored
                    .abandoned_roots_named_by_the_pool_checker
                    .clone()
                    .expect("给定状态表那一路每块都存点名的被抛弃根");
                let states = stored.block.verdict_vector_number_of_each_state().len();
                store
                    .store_conclusion_block_naming_the_abandoned_roots(
                        &block.key,
                        &kind,
                        &version,
                        &ConclusionBlockRecord {
                            block: VerdictBlock::new(vec![green_verdict; states])
                                .expect("一块至少一个状态"),
                            violating_states: Vec::new(),
                            count_vector: stored.count_vector.clone(),
                            judged_on: stored.judged_on.clone(),
                            recovery_words: stored.recovery_words.clone(),
                        },
                        &abandoned_roots_named,
                    )
                    .expect("不红的一份换得上记着红的那一份");
                assert!(
                    !store
                        .conclusion_block_has_violations(&block.key, &kind, &version)
                        .expect("库读得了"),
                    "前提：库里这张表这一块换成了不红的"
                );
            }
        }
        let rewriting_run = engine.finish();
        assert_eq!(
            (
                rewriting_run.tally.tables_judged_here,
                rewriting_run.tally.tables_reused
            ),
            (0, 0),
            "换结论那一趟一张表都不判、不复用"
        );
    }
    let (_both, both_run) = run_in_the_stage(None, vec![vec![true], vec![false]]);
    assert_eq!(
        (
            both_run.tally.tables_recorded_here,
            both_run.tally.tables_whose_recording_was_reused,
            both_run.tally.tables_reused
        ),
        (0, 1, 1),
        "两段先后跑：录入键命中不再录、照库里判过的复用"
    );
}

/// 场景：给定状态引擎（没配 GPU、两段先后跑）判一张两个状态的表、结论存进库；再起一趟引擎，把这张表头一块的结论换成记着红的一份
/// （库里这张表的结论齐全），再判同一张表。
/// 预期：第二趟不复用这张表（复用 0 张），现判一张、记进「记着红而现判」的账，判出的逐状态结论与头一趟相同；判完库里那一块照新结论写回，
/// 与头一趟存的那一份是同一个结论（「记红、撤绿、下趟现判」，给定状态表同一个规矩）。
/// 判别力（`crates/mutations.tsv`）：「红给定状态表：照库里的复用」这一行让这一条红。
#[test]
fn a_given_states_table_stored_red_is_judged_again_and_the_new_conclusion_replaces_it() {
    // 一次性库留着：造记着红的结论，不许进全仓那一份
    let disposable = crate::crash_library::DisposableLibrary::new("given-states-stored-red")
        .unwrap_or_else(|refusal| panic!("{refusal}"));
    let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
    let configuration = disposable.directory().join("multi-host-on-the-cpu.env");
    std::fs::write(&configuration, "ENABLE_GPU=0\n").expect("写得了配置");
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
    let table = GivenStatesOfATable {
        table_name: "two-states".to_string(),
        base: &base,
        writes: &writes,
        segments: &segments,
        seams: &[],
        persisted_sets: vec![vec![true], vec![false]],
    };
    let settings: Vec<(&str, &std::ffi::OsStr)> =
        vec![("SINGLEFS_MULTI_HOST_CONFIG", configuration.as_os_str())];
    let _environment = CrashAmplificationEnvironment::cleared_then_set(&settings);
    let start_the_engine = || {
        GivenStatesEngine::start_from_environment(
            GivenStatesItem {
                item_name: "given-states-stored-red".to_string(),
            },
            &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
        )
        .expect("起得了给定状态表的引擎")
    };
    let first_engine = start_the_engine();
    let first = first_engine
        .judge_the_given_states(&table)
        .expect("头一趟判得了");
    let first_run = first_engine.finish();
    assert_eq!(
        (
            first_run.tally.tables_judged_here,
            first_run.tally.tables_reused
        ),
        (1, 0),
        "前提：头一趟现判这张表"
    );
    let engine = start_the_engine();
    let blocks = GivenStatesEngine::blocks_of_the_table((&table).into());
    let version = engine.version_of(JudgedBy::CpuJudge);
    let kind = JudgedBy::CpuJudge.conclusion_kind();
    let stored_by_the_first_run = {
        let mut store = engine.store();
        let stored = store
            .read_conclusion_block(&blocks[0].key, &kind, &version)
            .expect("库读得了")
            .expect("头一趟存过这张表头一块的结论");
        let abandoned_roots_named = stored
            .abandoned_roots_named_by_the_pool_checker
            .clone()
            .expect("给定状态表那一路每块都存点名的被抛弃根");
        store
            .store_conclusion_block_naming_the_abandoned_roots(
                &blocks[0].key,
                &kind,
                &version,
                &ConclusionBlockRecord {
                    block: stored.block.clone(),
                    violating_states: vec![StateOffsetInBlock(0)],
                    count_vector: stored.count_vector.clone(),
                    judged_on: stored.judged_on.clone(),
                    recovery_words: stored.recovery_words.clone(),
                },
                &abandoned_roots_named,
            )
            .expect("记着红的一份写得进库");
        assert!(
            store
                .conclusion_block_has_violations(&blocks[0].key, &kind, &version)
                .expect("库读得了"),
            "前提：库里这张表头一块记着红"
        );
        stored
    };
    let judged_again = engine.judge_the_given_states(&table).expect("第二趟判得了");
    let after_red = {
        let store = engine.store();
        store
            .read_conclusion_block(&blocks[0].key, &kind, &version)
            .expect("库读得了")
            .expect("那一块的结论在库里")
    };
    let run = engine.finish();
    assert_eq!(
        (
            run.tally.tables_reused,
            run.tally.tables_judged_here,
            run.tally.tables_stored_red_judged_again
        ),
        (0, 1, 1),
        "记着红的表不复用，这一趟现判一张"
    );
    assert_eq!(
        judged_again.states, first.states,
        "现判出的逐状态结论与头一趟相同"
    );
    assert!(
        after_red.is_the_same_conclusion_as(&stored_by_the_first_run),
        "判完库里那一块照新结论写回，与头一趟存的是同一个结论"
    );
}

/// L6、L3：给定状态表按段切成不止一块（第一次有写没持久的那一段相同的几个状态一块，全持久的一块）；块键只由表的内容定，
/// 表名换了块键不变，基线、写表或某个状态的持久集合改了块键跟着变。
#[test]
fn a_given_states_table_is_cut_into_blocks_by_segment_whose_keys_ignore_the_table_name() {
    let device = singlefs_core::address::DeviceIdentity(0);
    let base = MemoryPool::with_devices(&[device], 1 << 20);
    let write_at = |offset: u64, byte: u8| RetainedWrite {
        device,
        kind: StepKind::UnitWrite,
        is_force_unit_access: false,
        offset: singlefs_core::address::DeviceOffsetInBytes(offset),
        contents: WrittenContents::Bytes(vec![byte; 512]),
    };
    let writes = vec![write_at(8192, 7), write_at(16384, 8), write_at(24576, 9)];
    let segments = vec![vec![0], vec![1, 2]];
    let persisted_sets = vec![
        vec![false, false, false],
        vec![true, false, false],
        vec![true, true, true],
        vec![false, true, true],
    ];
    let table_named = |table_name: &str,
                       writes_of_the_table: &[RetainedWrite],
                       persisted_sets_of_the_table: Vec<Vec<bool>>| {
        let table = GivenStatesOfATable {
            table_name: table_name.to_string(),
            base: &base,
            writes: writes_of_the_table,
            segments: &segments,
            seams: &[],
            persisted_sets: persisted_sets_of_the_table,
        };
        GivenStatesEngine::blocks_of_the_table((&table).into())
    };
    let blocks = table_named("a", &writes, persisted_sets.clone());
    assert_eq!(
        blocks
            .iter()
            .map(|block| block.positions.clone())
            .collect::<Vec<_>>(),
        vec![vec![0, 3], vec![1], vec![2]],
        "按第一次有写没持久的那一段切：段 0 两个、段 1 一个、全持久一个"
    );
    let keys_of = |blocks_of_a_table: &[BlockOfAGivenStatesTable]| -> Vec<VerdictBlockKey> {
        blocks_of_a_table
            .iter()
            .map(|block| block.key.clone())
            .collect()
    };
    assert_eq!(
        keys_of(&table_named(
            "another-name",
            &writes,
            persisted_sets.clone()
        )),
        keys_of(&blocks),
        "表名换了块键不变"
    );
    let mut changed_writes = writes.clone();
    changed_writes[1] = write_at(16384, 10);
    assert!(
        keys_of(&table_named("a", &changed_writes, persisted_sets.clone()))
            .iter()
            .all(|key| !keys_of(&blocks).contains(key)),
        "写表改一个字节，每一块的键都变"
    );
    let mut changed_sets = persisted_sets;
    changed_sets[1] = vec![true, true, false];
    let changed = table_named("a", &writes, changed_sets);
    assert_eq!(changed[0].key, blocks[0].key, "别的段那一块不变");
    assert_ne!(changed[1].key, blocks[1].key, "持久集合改了的那一块键变");
}

/// 几张给定状态表的基线、写表与切段相同，第 k 张两个状态：持久了前 k 次写的那一个与全部持久的那一个。
/// 每张表切成两块：持久前缀那一个状态第一次有写没持久的那一段一块、全部持久一块；几张表全部持久那一块是同一把键（块键只由内容定），持久前缀那一块各张不同：
/// 4 张表一共 5 把不同的键，按表点名是 8 块。
/// 判别力（`crates/mutations.tsv`）：块的计划哈希带上表名，全部持久那一块各张不同、一共 8 把键，这条红。
#[test]
fn given_states_tables_sharing_their_frame_share_the_block_of_the_state_whose_every_write_persisted(
) {
    let device = singlefs_core::address::DeviceIdentity(0);
    let base = MemoryPool::with_devices(&[device], 1 << 20);
    let writes: Vec<RetainedWrite> = [8192u64, 16384, 24576, 32768]
        .iter()
        .zip([7u8, 8, 9, 10])
        .map(|(offset, byte)| RetainedWrite {
            device,
            kind: StepKind::UnitWrite,
            is_force_unit_access: false,
            offset: singlefs_core::address::DeviceOffsetInBytes(*offset),
            contents: WrittenContents::Bytes(vec![byte; 512]),
        })
        .collect();
    let segments = vec![vec![0, 1], vec![2, 3]];
    let table_count = writes.len();
    let persisted_sets_of_each_table: Vec<Vec<Vec<bool>>> = (0..table_count)
        .map(|persisted_prefix_writes| {
            vec![
                (0..writes.len())
                    .map(|write_index| write_index < persisted_prefix_writes)
                    .collect(),
                vec![true; writes.len()],
            ]
        })
        .collect();
    let blocks_of_each_table: Vec<Vec<BlockOfAGivenStatesTable>> = persisted_sets_of_each_table
        .iter()
        .enumerate()
        .map(|(table_index, persisted_sets)| {
            let table = GivenStatesOfATable {
                table_name: format!("table-{table_index}"),
                base: &base,
                writes: &writes,
                segments: &segments,
                seams: &[],
                persisted_sets: persisted_sets.clone(),
            };
            GivenStatesEngine::blocks_of_the_table((&table).into())
        })
        .collect();
    let groups_of_each_table: Vec<Vec<SegmentGroupOfAGivenState>> = blocks_of_each_table
        .iter()
        .zip(&persisted_sets_of_each_table)
        .map(|(blocks, persisted_sets)| {
            blocks
                .iter()
                .map(|block| {
                    segment_group_of_a_given_state(&segments, &persisted_sets[block.positions[0]])
                })
                .collect()
        })
        .collect();
    assert_eq!(
        groups_of_each_table,
        vec![
            vec![
                SegmentGroupOfAGivenState::FirstUnpersistedWriteInSegment(0),
                SegmentGroupOfAGivenState::EveryWritePersisted
            ],
            vec![
                SegmentGroupOfAGivenState::FirstUnpersistedWriteInSegment(0),
                SegmentGroupOfAGivenState::EveryWritePersisted
            ],
            vec![
                SegmentGroupOfAGivenState::FirstUnpersistedWriteInSegment(1),
                SegmentGroupOfAGivenState::EveryWritePersisted
            ],
            vec![
                SegmentGroupOfAGivenState::FirstUnpersistedWriteInSegment(1),
                SegmentGroupOfAGivenState::EveryWritePersisted
            ],
        ],
        "每张表两块：持久前缀那一个状态所在那一段一块、全部持久一块（块按段号排）"
    );
    assert!(
        blocks_of_each_table.iter().all(|blocks| blocks
            .iter()
            .map(|block| block.positions.clone())
            .collect::<Vec<_>>()
            == vec![vec![0], vec![1]]),
        "持久前缀那一个状态一块、全部持久那一个状态一块"
    );
    let keys_of_the_every_write_persisted_blocks: BTreeSet<VerdictBlockKey> = blocks_of_each_table
        .iter()
        .map(|blocks| blocks[1].key.clone())
        .collect();
    assert_eq!(
        keys_of_the_every_write_persisted_blocks.len(),
        1,
        "几张表全部持久那一块是同一把键"
    );
    let every_key: BTreeSet<VerdictBlockKey> = blocks_of_each_table
        .iter()
        .flatten()
        .map(|block| block.key.clone())
        .collect();
    assert_eq!(
        (
            every_key.len(),
            blocks_of_each_table.iter().map(Vec::len).sum::<usize>()
        ),
        (table_count + 1, 2 * table_count),
        "4 张表一共 5 把不同的键（各一块持久前缀、共用一块全部持久），按表点名 8 块"
    );
}

/// 同一个表名下写表不同的两张给定状态表（甲报告问题 5）：录入段各录一张（各按自己的内容摘要），只判那一趟按调用方这一趟交来的那张表现算摘要、
/// 找到录入段录的那一张、各判各的——一张一个状态、一张两个状态，判出来的状态数各是各的。
#[test]
fn two_given_states_tables_of_one_name_and_different_writes_are_each_judged_in_the_judge_stage() {
    // 一次性库留着：给定状态引擎两段的自证，要从空库起跑
    let disposable =
        crate::crash_library::DisposableLibrary::new("given-states-tables-of-one-name")
            .unwrap_or_else(|refusal| panic!("{refusal}"));
    let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
    let configuration = disposable.directory().join("multi-host-on-the-cpu.env");
    std::fs::write(&configuration, "ENABLE_GPU=0\n").expect("写得了配置");
    let device = singlefs_core::address::DeviceIdentity(0);
    let base = MemoryPool::with_devices(&[device], 1 << 20);
    let writes_with_the_byte = |byte: u8| {
        vec![RetainedWrite {
            device,
            kind: StepKind::UnitWrite,
            is_force_unit_access: false,
            offset: singlefs_core::address::DeviceOffsetInBytes(8192),
            contents: WrittenContents::Bytes(vec![byte; 512]),
        }]
    };
    let writes_of_the_one_state_table = writes_with_the_byte(7);
    let writes_of_the_two_state_table = writes_with_the_byte(9);
    let segments = vec![vec![0]];
    let one_state_table = || GivenStatesOfATable {
        table_name: "one-name".to_string(),
        base: &base,
        writes: &writes_of_the_one_state_table,
        segments: &segments,
        seams: &[],
        persisted_sets: vec![vec![true]],
    };
    let two_state_table = || GivenStatesOfATable {
        table_name: "one-name".to_string(),
        base: &base,
        writes: &writes_of_the_two_state_table,
        segments: &segments,
        seams: &[],
        persisted_sets: vec![vec![true], vec![false]],
    };
    let run_in_the_stage = |stage: &str| {
        let _environment = CrashAmplificationEnvironment::cleared_then_set(&[
            ("SINGLEFS_MULTI_HOST_CONFIG", configuration.as_os_str()),
            (RUN_STAGE_ENVIRONMENT_VARIABLE, std::ffi::OsStr::new(stage)),
        ]);
        let engine = GivenStatesEngine::start_from_environment(
            GivenStatesItem {
                item_name: "given-states-tables-of-one-name".to_string(),
            },
            &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
        )
        .expect("起得了给定状态表的引擎");
        let state_counts = (
            engine
                .judge_the_given_states(&one_state_table())
                .expect("判得了一个状态的那张表")
                .states
                .len(),
            engine
                .judge_the_given_states(&two_state_table())
                .expect("判得了两个状态的那张表")
                .states
                .len(),
        );
        (state_counts, engine.finish())
    };
    let (_, record_run) = run_in_the_stage("record");
    assert_eq!(
        record_run.tally.tables_recorded_here, 2,
        "录入段同名的两张表各录一张"
    );
    let (judged_state_counts, judge_run) = run_in_the_stage("judge");
    assert_eq!(
        judged_state_counts,
        (1, 2),
        "只判那一趟同名的两张表各判各的：一张一个状态、一张两个状态"
    );
    assert_eq!(judge_run.tally.tables_judged_here, 2);
}

/// 「配 GPU 必用 GPU、做不到报错不降级」：配置开了 GPU 而这份二进制没编进 gpu 特性（不带特性编的那一趟），给定状态表的
/// 引擎录入段照录（表写进库），判段不跑、交占位结论（一张不判，不退回 CPU 判），这一趟算只录（调用方照本次未跑交回）；开跑时打的本次未跑那一行
/// 是门禁 crash-point-replay 的 `test_not_run_lines` 认的那一族记号、带着项名，不是 `GIVEN_STATES_JUDGE route=cpu`。
/// 判别力（`crates/mutations.tsv`）：判段照 CPU 判下去，这条红；不要卡的这一趟也取显卡先后（清没清场都拒），这条红。
#[cfg(not(feature = "gpu"))]
#[test]
fn the_given_states_engine_without_the_gpu_feature_records_the_table_and_reports_the_judge_not_run()
{
    // 一次性库留着：钉判段不跑只录，要数库里录了什么
    let disposable =
        crate::crash_library::DisposableLibrary::new("given-states-engine-without-the-gpu-feature")
            .unwrap_or_else(|refusal| panic!("{refusal}"));
    let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
    let configuration = disposable.directory().join("multi-host-with-the-gpu.env");
    std::fs::write(&configuration, "ENABLE_GPU=1\n").expect("写得了配置");
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
    // 配置只写开关、环境里也不交显卡先后：没编进 gpu 特性，判段不跑、一张卡都不要，引擎不取显卡先后
    // （清没清场都一样：取了的话，没经服务给卡而配置开了 GPU 要拒）
    let environment = CrashAmplificationEnvironment::cleared_then_set(&[(
        "SINGLEFS_MULTI_HOST_CONFIG",
        configuration.as_os_str(),
    )]);
    let engine = GivenStatesEngine::start_from_environment(
        GivenStatesItem {
            item_name: "given-states-engine-without-the-gpu-feature".to_string(),
        },
        &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
    )
    .expect("配置开了 GPU、没编进特性：引擎照样起得了（判段不跑，不拒跑）");
    let conclusions = engine
        .judge_the_given_states(&GivenStatesOfATable {
            table_name: "one-state".to_string(),
            base: &base,
            writes: &writes,
            segments: &segments,
            seams: &[],
            persisted_sets: vec![vec![true]],
        })
        .expect("判段不跑也照常交回");
    let run = engine.finish();
    drop(environment);
    assert_eq!(
        conclusions.source,
        GivenStatesConclusionsSource::PlaceholderOfATableRecordedOnly,
        "交的是占位结论"
    );
    assert!(
        run.not_run_without_the_gpu_feature
            .as_ref()
            .is_some_and(|reason| reason.contains("ENABLE_GPU=1")),
        "{:?}",
        run.not_run_without_the_gpu_feature
    );
    assert!(run.recorded_only(), "这一趟算只录，调用方照本次未跑交回");
    assert_eq!(
        (
            run.tally.tables_recorded_here,
            run.tally.tables_not_judged_without_the_gpu_feature,
            run.tally.tables_judged_here,
            run.tally.states_judged_by_the_cpu_here,
            run.tally.states_judged_by_the_gpu_here
        ),
        (1, 1, 0, 0, 0),
        "表照录、一张不判（不退回 CPU 判）"
    );
    let line = given_states_not_run_line_without_the_gpu_feature(
        "given-states-engine-without-the-gpu-feature",
        run.not_run_without_the_gpu_feature
            .as_deref()
            .expect("上面判过有原因"),
    );
    assert!(
            line.starts_with(
                "NOT_RUN_NO_GPU_ADAPTER test=given_states_engine::given-states-engine-without-the-gpu-feature reason="
            ) && !line.contains("route=cpu"),
            "{line}"
        );
}
