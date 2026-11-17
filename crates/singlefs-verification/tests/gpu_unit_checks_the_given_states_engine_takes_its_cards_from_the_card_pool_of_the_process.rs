//! 验证包模块：crash_point_replay_progress、crash_given_states、crash_library、crash_amplification、gpu_unit_checks
//! 同一个集成测试进程里只有一套显存账：崩溃点重放的给定状态引擎在这个进程还没起卡池时起步（起步不取卡、不起卡池），
//! 第一次在卡上判一张给定状态表时取卡，照样当场起这个进程的卡池、照池里的卡用（`crash_amplification` 文件头第 5 条），
//! 不另起不进账的设备；同一个进程里接着直接从卡池借一笔卡（GPU 用例的借法）。引擎交回的卡的来路是这个进程的卡池，
//! 引擎判完表时与收尾之后这个进程里不走卡池账的设备都是 0 台（`gpu_unit_checks::devices_marked_outside_the_card_pool_in_this_process`）。
//! 先让引擎判表、再看卡池：引擎取卡时卡池还没起，钉的就是卡池还没起时引擎那一种取法。
//! 这份文件只有这一条用例：引擎判表那一刻这个进程里还没有别的用例起过卡池（起过了，取卡不论怎么走都照池用，钉不住卡池还没起时的那一种）。
//! 要 `gpu` 特性；本机一张 NVIDIA 独显适配器都没有时引擎判不了表，卡池打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、返回成功
//! （`gpu_unit_checks::gpu_test_start`）。有卡而判那张表时借不到显存（卡上被别的进程占满）照判红，原因原样带出来。
#![cfg(feature = "gpu")]

mod common;
mod common_gpu_cards_of_the_caller;

use singlefs_verification::crash_amplification::{
    CrashAmplificationEnvironment, GivenStatesEngine, GivenStatesItem, GpuCardsSource,
};
use singlefs_verification::crash_given_states::{GivenStatesJudge, GivenStatesOfATable};
use singlefs_verification::crash_library::DisposableLibrary;
use singlefs_verification::crash_point_replay_progress::CrashPointReplayToolchainIdentity;
use singlefs_verification::gpu_card_configuration::MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE;
use singlefs_verification::gpu_unit_checks::devices_marked_outside_the_card_pool_in_this_process;
use singlefs_verification::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;

const TEST_NAME: &str =
    "the_given_states_engine_started_before_the_card_pool_takes_its_cards_from_the_pool_and_starts_no_device_outside_it";
/// 直接从卡池借的那一笔（MiB）：只为让卡池在借到的卡上起设备，量取得小，不与引擎判表要的争显存。
const MEBIBYTES_LEASED_DIRECTLY: u64 = 16;

#[test]
fn the_given_states_engine_started_before_the_card_pool_takes_its_cards_from_the_pool_and_starts_no_device_outside_it(
) {
    // 一次性库留着：这张表要在卡上现判（复用了就不上卡），全仓那一份一行不写
    let directory = DisposableLibrary::new("given-states-engine-takes-its-cards-from-the-pool")
        .unwrap_or_else(|refusal| panic!("{refusal}"));
    let configuration = directory.join("multi-host-with-gpu.env");
    std::fs::write(&configuration, "ENABLE_GPU=1\n").expect("写得了开 GPU 的配置");
    let image_pool = common::build_pool("given-states-engine-takes-its-cards-from-the-pool");
    let image = image_pool.memory_pool();
    let _the_library_of_this_process = directory.make_it_the_library_of_this_process();
    let environment = CrashAmplificationEnvironment::cleared_then_set(&[(
        MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
        configuration.as_os_str(),
    )]);
    // 调用方给这个进程的卡交给引擎、不在这里领：服务交了卡串的设卡串，交领卡通道的把通道设回环境，引擎第一次上卡时才领
    // （common_gpu_cards_of_the_caller）
    let cards_of_the_caller =
        common_gpu_cards_of_the_caller::cards_of_the_caller_handed_to_the_run();
    let engine = GivenStatesEngine::start_from_environment(
        GivenStatesItem {
            item_name: format!("{TEST_NAME}::the-pool-after-the-new-pool-file-creation"),
        },
        &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
    )
    .unwrap_or_else(|reason| panic!("引擎起不来（起步不取卡，起不来与有没有卡无关）：{reason}"));
    let table = GivenStatesOfATable {
        table_name: "the-pool-after-the-new-pool-file-creation".to_string(),
        base: &image,
        writes: &[],
        segments: &[],
        seams: &[],
        persisted_sets: vec![Vec::new()],
    };
    let judged = match engine.judge_the_given_states(&table) {
        Ok(judged) => judged,
        Err(reason) => {
            // 本机没卡：卡池打「本次未跑」那一行；有卡而引擎判不了是红
            if gpu_card_pool_or_report_not_run(TEST_NAME).is_none() {
                return;
            }
            panic!(
                "本机有卡，引擎却判不了给定状态表 {}：{reason}",
                table.table_name
            );
        }
    };
    assert_eq!(judged.states.len(), 1, "判器逐状态交回结论：表里一个状态");
    assert_eq!(
        devices_marked_outside_the_card_pool_in_this_process(),
        0,
        "引擎第一次上卡时取卡、判完一张表（引擎还活着）：这个进程里没有不走卡池账的设备"
    );
    let pool = gpu_card_pool_or_report_not_run(TEST_NAME)
        .expect("引擎判表时取卡已经起了这个进程的卡池，起好的池判过有卡");
    let lease = pool
        .lease_card_or_report_not_run(TEST_NAME, MEBIBYTES_LEASED_DIRECTLY)
        .unwrap_or_else(|| {
            panic!("卡池借不出 {MEBIBYTES_LEASED_DIRECTLY} MiB（上面「本次未跑」那一行写了为什么）")
        });
    drop(lease);
    assert_eq!(
        devices_marked_outside_the_card_pool_in_this_process(),
        0,
        "直接从卡池借过一笔（引擎还活着）：这个进程里没有不走卡池账的设备"
    );
    let run = engine.finish();
    drop(cards_of_the_caller);
    drop(environment);
    assert_eq!(
        run.gpu_cards_taken_from,
        Some(GpuCardsSource::TheCardPoolOfThisProcess),
        "引擎在卡池起之前起步、判表时取卡：照这个进程的卡池用"
    );
    assert!(
        run.tally.states_judged_by_the_gpu_here > 0,
        "配置开了 GPU、库是空的：那张表在卡上现判了（{:?}）",
        run.tally
    );
    assert_eq!(
        devices_marked_outside_the_card_pool_in_this_process(),
        0,
        "引擎收尾之后：这个进程里没有不走卡池账的设备"
    );
}
