//! 验证包模块：crash_point_replay_progress、crash_given_states、crash_library、crash_amplification、gpu_unit_checks
//! 给定状态引擎两段先后跑（不设选段）时，起步不领卡，第一张表录完、要在卡上判时才去领：配置开着 GPU、编进了 gpu 特性，
//! 调用方没交卡串而交了领卡通道，领卡被服务拒（假领卡客户端 `lease take` 退 3）。引擎起得来；判那张表时报领卡的错，
//! 领卡客户端只被调了一回、是领卡；交回的卡的来路是「没取过卡」；这个进程的卡池没起。再起一个只录的引擎过同一张表，
//! 表的录入键命中（库里已有逐字节相同的那一份、这一趟一张不另录）：头一个引擎领卡之前已经把这张表录进库。
//! 领卡挪回引擎起步的话，头一个引擎起不来，这一条红。
//! 领卡通道是假的（`common_gpu_cards_of_the_caller::fake_card_lease_channel_refusing_every_lease_set_into_the_process_environment`），不要真卡。
//! 调用方的显卡配置与领卡的结局进程里只记一回，这份文件只放这一条用例。
#![cfg(feature = "gpu")]

mod common;
mod common_gpu_cards_of_the_caller;

use singlefs_verification::crash_amplification::{
    CrashAmplificationEnvironment, GivenStatesEngine, GivenStatesItem,
};
use singlefs_verification::crash_given_states::{GivenStatesJudge, GivenStatesOfATable};
use singlefs_verification::crash_library::{
    DisposableLibrary, ExclusiveTemporaryDirectory, RUN_STAGE_ENVIRONMENT_VARIABLE,
};
use singlefs_verification::crash_point_replay_progress::CrashPointReplayToolchainIdentity;
use singlefs_verification::gpu_card_configuration::MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE;
use singlefs_verification::gpu_unit_checks::gpu_test_start::card_pool_started_in_this_process;

const TEST_NAME: &str =
    "a_given_states_engine_of_both_stages_records_the_first_table_before_its_card_lease_is_refused";

#[test]
fn a_given_states_engine_of_both_stages_records_the_first_table_before_its_card_lease_is_refused() {
    // 假通道在这个进程第一次问调用方的显卡配置之前设好
    let fake_card_lease =
        ExclusiveTemporaryDirectory::create("both-stages-given-states-fake-card-lease")
            .expect("建得了放假领卡客户端的一次性临时目录");
    let calls = common_gpu_cards_of_the_caller::fake_card_lease_channel_refusing_every_lease_set_into_the_process_environment(
        fake_card_lease.path(),
    );
    let image_pool = common::build_pool("both-stages-given-states-records-before-leasing");
    let image = image_pool.memory_pool();
    let table = GivenStatesOfATable {
        table_name: "the-pool-after-the-new-pool-file-creation".to_string(),
        base: &image,
        writes: &[],
        segments: &[],
        seams: &[],
        persisted_sets: vec![Vec::new()],
    };
    let item = || GivenStatesItem {
        item_name: format!("{TEST_NAME}::the-pool-after-the-new-pool-file-creation"),
    };
    let toolchain = CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test();
    // 一次性库：头一个引擎录进去的表，只录的那一个引擎读得到；全仓那一份一行不写
    let directory = DisposableLibrary::new("both-stages-given-states-records-before-leasing")
        .unwrap_or_else(|refusal| panic!("{refusal}"));
    let configuration = directory.join("multi-host-with-gpu.env");
    std::fs::write(&configuration, "ENABLE_GPU=1\n").expect("写得了开 GPU 的配置");
    let _the_library_of_this_process = directory.make_it_the_library_of_this_process();
    let mut environment = CrashAmplificationEnvironment::cleared_then_set(&[(
        MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
        configuration.as_os_str(),
    )]);
    // 调用方的领卡通道交给引擎、不在这里领（守卫清掉了通道目录与凭据）
    let cards_of_the_caller =
        common_gpu_cards_of_the_caller::cards_of_the_caller_handed_to_the_run();
    let engine_of_both_stages = GivenStatesEngine::start_from_environment(item(), &toolchain)
        .unwrap_or_else(|reason| panic!("两段先后跑的引擎起步不领卡、起得来：{reason}"));
    let calls_after_the_start = std::fs::read_to_string(&calls).unwrap_or_default();
    let refusal = engine_of_both_stages
        .judge_the_given_states(&table)
        .expect_err("领卡被拒：第一张表要在卡上判时报错");
    let run_of_both_stages = engine_of_both_stages.finish();
    let calls_after_the_first_table = std::fs::read_to_string(&calls).unwrap_or_default();
    environment.set_variable(
        RUN_STAGE_ENVIRONMENT_VARIABLE,
        std::ffi::OsStr::new("record"),
    );
    let recording_only_engine = GivenStatesEngine::start_from_environment(item(), &toolchain)
        .unwrap_or_else(|reason| panic!("只录的引擎起不来：{reason}"));
    recording_only_engine
        .judge_the_given_states(&table)
        .unwrap_or_else(|reason| panic!("只录的那一趟过同一张表：{reason}"));
    let recording_only_run = recording_only_engine.finish();
    drop(cards_of_the_caller);
    drop(environment);
    assert_eq!(calls_after_the_start, "", "引擎起步没调领卡客户端");
    assert!(
        refusal.contains("lease take 退 3"),
        "第一张表报的是领卡被拒：{refusal}"
    );
    assert_eq!(
        calls_after_the_first_table, "lease take\n",
        "第一张表要上卡时领卡客户端被调了一回、是领卡"
    );
    assert_eq!(
        run_of_both_stages.gpu_cards_taken_from, None,
        "取卡没成：交回没取过卡"
    );
    assert_eq!(
        run_of_both_stages.tally.tables_recorded_here, 1,
        "两段先后跑的引擎把那张表录进了库（{:?}）",
        run_of_both_stages.tally
    );
    assert_eq!(
        (
            recording_only_run.tally.tables_whose_recording_was_reused,
            recording_only_run.tally.tables_recorded_here
        ),
        (1, 0),
        "只录的引擎过同一张表：录入键命中、一张不另录，头一个引擎领卡之前已经录完（{:?}）",
        recording_only_run.tally
    );
    assert_eq!(
        std::fs::read_to_string(&calls).unwrap_or_default(),
        "lease take\n",
        "只录的引擎没再调领卡客户端"
    );
    assert!(
        card_pool_started_in_this_process().is_none(),
        "领卡被拒、只录：这个进程的卡池一直没起"
    );
    drop(fake_card_lease);
}
