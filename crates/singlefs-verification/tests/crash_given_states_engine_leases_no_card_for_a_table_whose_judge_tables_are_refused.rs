//! 验证包模块：crash_point_replay_progress、crash_given_states、crash_library、crash_amplification、gpu_unit_checks
//! 给定状态引擎判器走 GPU、两段先后跑（不设选段），一张表的判表建不出（基线是两块空盘，系统配置槽一份都解不开：
//! `crash_judge_tables::JudgeTablesRefusal::BaseSystemConfigurationUnreadable`）：这张表交 CPU 判器判，一直不上卡。
//! 上卡判那一段开头限流要数这一趟在几张卡上判，数的是起步时记下的显卡来路，不为数张数取卡：调用方没交卡串而有借卡通道
//! （假借卡客户端，`lease take` 退 3），借卡客户端一回都没被调，这个进程的卡池没起，判这张表不报错。
//! 张数改成先取卡再数的话，判表还没建就去取卡、报没经服务给卡（判段取卡不问借卡通道），这张表报取卡的错，这一条红。
//! 借卡通道是假的（`common_gpu_cards_of_the_caller::fake_card_lease_channel_refusing_every_lease_set_into_the_process_environment`），不要真卡。
//! 调用方的显卡配置与借卡的结局进程里只记一回，这份文件只放这一条用例。
#![cfg(feature = "gpu")]

mod common;
mod common_gpu_cards_of_the_caller;

use singlefs_core::address::DeviceIdentity;
use singlefs_verification::crash_amplification::{
    CrashAmplificationEnvironment, GivenStatesEngine, GivenStatesItem,
};
use singlefs_verification::crash_given_states::{GivenStatesJudge, GivenStatesOfATable};
use singlefs_verification::crash_library::{DisposableLibrary, ExclusiveTemporaryDirectory};
use singlefs_verification::crash_point_replay_progress::CrashPointReplayToolchainIdentity;
use singlefs_verification::gpu_card_configuration::MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE;
use singlefs_verification::gpu_unit_checks::gpu_test_start::card_pool_started_in_this_process;
use singlefs_verification::memory_pool::MemoryPool;

const TEST_NAME: &str =
    "a_given_states_engine_leases_no_card_for_a_table_whose_judge_tables_are_refused";

#[test]
fn a_given_states_engine_leases_no_card_for_a_table_whose_judge_tables_are_refused() {
    // 假通道在这个进程第一次问调用方的显卡配置之前设好
    let fake_card_lease =
        ExclusiveTemporaryDirectory::create("given-states-refused-judge-tables-fake-card-lease")
            .expect("建得了放假领卡客户端的一次性临时目录");
    let calls = common_gpu_cards_of_the_caller::fake_card_lease_channel_refusing_every_lease_set_into_the_process_environment(
        fake_card_lease.path(),
    );
    let pool_of_two_blank_devices =
        MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], common::IMAGE_BYTES);
    let table = GivenStatesOfATable {
        table_name: "a-pool-of-two-blank-devices".to_string(),
        base: &pool_of_two_blank_devices,
        writes: &[],
        segments: &[],
        seams: &[],
        persisted_sets: vec![Vec::new()],
    };
    let toolchain = CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test();
    // 一次性库：全仓那一份一行不写
    let directory = DisposableLibrary::new("given-states-refused-judge-tables-lease-no-card")
        .unwrap_or_else(|refusal| panic!("{refusal}"));
    let configuration = directory.join("multi-host-with-gpu.env");
    std::fs::write(&configuration, "ENABLE_GPU=1\n").expect("写得了开 GPU 的配置");
    let _the_library_of_this_process = directory.make_it_the_library_of_this_process();
    let environment = CrashAmplificationEnvironment::cleared_then_set(&[(
        MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
        configuration.as_os_str(),
    )]);
    // 调用方的借卡通道交给引擎、不在这里借（守卫清掉了通道的三个尝试变量，这里照调用方的值设回）
    let cards_of_the_caller =
        common_gpu_cards_of_the_caller::cards_of_the_caller_handed_to_the_run();
    let engine = GivenStatesEngine::start_from_environment(
        GivenStatesItem {
            item_name: format!("{TEST_NAME}::a-pool-of-two-blank-devices"),
        },
        &toolchain,
    )
    .unwrap_or_else(|reason| panic!("引擎起步不领卡、起得来：{reason}"));
    let judged = engine.judge_the_given_states(&table);
    let run = engine.finish();
    drop(cards_of_the_caller);
    drop(environment);
    let calls_made = std::fs::read_to_string(&calls).unwrap_or_default();
    let conclusions = judged.unwrap_or_else(|reason| {
        panic!("判表建不出的那张表交 CPU 判器判、不上卡，不该报领卡的错：{reason}（领卡客户端的调用：{calls_made:?}）")
    });
    assert_eq!(
        conclusions.states.len(),
        1,
        "那张表的一个状态由 CPU 判器判出了结论"
    );
    assert_eq!(
        run.tally.states_the_gpu_could_not_judge.len(),
        1,
        "那张表记成 GPU 判不了（判表建不出）：{:?}",
        run.tally.states_the_gpu_could_not_judge
    );
    assert_eq!(
        calls_made, "",
        "判表建不出的那张表不上卡：数卡没调领卡客户端"
    );
    assert_eq!(
        run.gpu_cards_taken_from, None,
        "一张表都没上卡：交回没取过卡"
    );
    assert!(
        card_pool_started_in_this_process().is_none(),
        "数卡不起卡池：这个进程的卡池一直没起"
    );
    drop(fake_card_lease);
}
