//! 验证包模块：crash_point_replay_progress、crash_given_states、crash_library、crash_amplification、gpu_unit_checks
//! 给定状态引擎只录（`SINGLEFS_CRASH_AMPLIFICATION_STAGE=record`）时起步、过一张表、收尾都不借卡、不起卡池：配置开着 GPU、编进了 gpu 特性
//! （判器走 GPU 那一路），调用方没交卡串而进程环境里摆着一条借卡通道（用例开头设进去的假通道；
//! `common_gpu_cards_of_the_caller::cards_of_the_caller_handed_to_the_run` 只交服务交的卡串，没交的不设、不把通道交给引擎）。
//! 表录进库、交占位结论，借卡客户端一回都没被调，这个进程的卡池没起，交回的卡的来路是「没取过卡」。
//! 引擎第一次要在卡上判一张表时才取卡（`crash_amplification::GivenStatesEngine` 判 GPU 那一路的入口），起步只记显卡的来路；判段取卡不问借卡通道。
//! 借卡通道是假的（`common_gpu_cards_of_the_caller::fake_card_lease_channel_refusing_every_lease_set_into_the_process_environment`），不要真卡。
//! 调用方的显卡配置与借卡的结局进程里只记一回，这份文件只放这一条用例。
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
    "a_recording_only_given_states_engine_given_a_card_lease_channel_leases_no_card_and_starts_no_card_pool";

#[test]
fn a_recording_only_given_states_engine_given_a_card_lease_channel_leases_no_card_and_starts_no_card_pool(
) {
    // 假通道在这个进程第一次问调用方的显卡配置之前设好
    let fake_card_lease =
        ExclusiveTemporaryDirectory::create("recording-only-given-states-fake-card-lease")
            .expect("建得了放假领卡客户端的一次性临时目录");
    let calls = common_gpu_cards_of_the_caller::fake_card_lease_channel_refusing_every_lease_set_into_the_process_environment(
        fake_card_lease.path(),
    );
    let image_pool = common::build_pool("recording-only-given-states-leases-no-card");
    let image = image_pool.memory_pool();
    // 一次性库：只录的那一趟写进去的不留在全仓那一份里
    let directory = DisposableLibrary::new("recording-only-given-states-leases-no-card")
        .unwrap_or_else(|refusal| panic!("{refusal}"));
    let configuration = directory.join("multi-host-with-gpu.env");
    std::fs::write(&configuration, "ENABLE_GPU=1\n").expect("写得了开 GPU 的配置");
    let _the_library_of_this_process = directory.make_it_the_library_of_this_process();
    let environment = CrashAmplificationEnvironment::cleared_then_set(&[
        (
            MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
            configuration.as_os_str(),
        ),
        (
            RUN_STAGE_ENVIRONMENT_VARIABLE,
            std::ffi::OsStr::new("record"),
        ),
    ]);
    let cards_of_the_caller =
        common_gpu_cards_of_the_caller::cards_of_the_caller_handed_to_the_run();
    let engine = GivenStatesEngine::start_from_environment(
        GivenStatesItem {
            item_name: format!("{TEST_NAME}::the-pool-after-the-new-pool-file-creation"),
        },
        &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
    )
    .unwrap_or_else(|reason| panic!("只录的引擎起不来：{reason}"));
    let table = GivenStatesOfATable {
        table_name: "the-pool-after-the-new-pool-file-creation".to_string(),
        base: &image,
        writes: &[],
        segments: &[],
        seams: &[],
        persisted_sets: vec![Vec::new()],
    };
    let conclusions = engine
        .judge_the_given_states(&table)
        .unwrap_or_else(|reason| panic!("只录的那一趟过一张表：{reason}"));
    let run = engine.finish();
    drop(cards_of_the_caller);
    drop(environment);
    assert_eq!(
        conclusions.states.len(),
        1,
        "占位结论逐状态交回：表里一个状态"
    );
    assert!(run.recorded_only(), "选段是 record：这一趟只录");
    assert_eq!(
        (
            run.tally.tables_recorded_here,
            run.tally.tables_recorded_only
        ),
        (1, 1),
        "那张表录进了库、没判（{:?}）",
        run.tally
    );
    assert_eq!(
        run.gpu_cards_taken_from, None,
        "判器走 GPU 而一张表都没走到卡上：交回没取过卡"
    );
    assert!(
        !calls.exists(),
        "只录的引擎调了领卡客户端：{}",
        std::fs::read_to_string(&calls).unwrap_or_default()
    );
    assert!(
        card_pool_started_in_this_process().is_none(),
        "只录的引擎起了这个进程的卡池"
    );
    drop(fake_card_lease);
}
