//! 验证包模块：crash_library、crash_amplification、gpu_unit_checks
//! 崩溃放量判段这一趟这台机器一块都没选中（块区间变量 `crash_amplification::BLOCK_RANGE_ENVIRONMENT_VARIABLE` 给这条流写了空区间）时
//! 不取卡、不起卡池、不起判器，照常交回：配置开着 GPU、编进了 gpu 特性（判段是要卡的那一路），两段都跑，调用方没交卡串而有借卡通道
//! （任务服务交给每次尝试的三个尝试变量），经测试共用的 `common_crash_points::amplify_prepared_flow` 跑完，借卡客户端一回都没被调、这个进程的卡池没起，
//! 交回的结果说这一趟一块没选中（`EnvironmentRun::no_block_selected_here`）。借卡通道是假的
//! （`common_gpu_cards_of_the_caller::fake_card_lease_channel_refusing_every_lease_set_into_the_process_environment`：每次调用记一行、
//! 借卡退 3），不要真卡；这一趟不在一块没选中那一处交回而往下判的话，判段写待判行进一次性库或取卡报没经服务给卡（判段取卡不问借卡通道），
//! 这一趟交错，共用的那一层当场断言失败。调用方的显卡配置与借到的卡进程里只记一回，这份文件只放这一条用例。
#![cfg(feature = "gpu")]

mod common;
mod common_crash_points;
// 取卡帮手那一份共用模块已由 `common_crash_points` 引进来，这里只借它的名字，不再 `mod` 第二遍（同一份文件引两遍 clippy 的 duplicate_mod 判红）。
use common_crash_points::common_gpu_cards_of_the_caller;

use singlefs_verification::crash_amplification::{
    CrashAmplificationEnvironment, GpuCardsSource, BLOCK_RANGE_ENVIRONMENT_VARIABLE,
};
use singlefs_verification::crash_library::{DisposableLibrary, ExclusiveTemporaryDirectory};
use singlefs_verification::gpu_card_configuration::MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE;
use singlefs_verification::gpu_unit_checks::gpu_test_start::card_pool_started_in_this_process;

/// 跑的时候的流名：来历表里崩溃放量那一路「两处都不登」的那一条流名，`::` 之后是这条用例的标签（来历表只认 `::` 之前那一截）。
const FLOW_NAME: &str = "line-fields::selecting-no-block-leases-no-card";
/// 这条流自己的展开上限：只为有几块可录，取得小。
const EXPAND_UP_TO_OF_THE_FLOW: usize = 2;

#[test]
fn a_crash_amplification_run_selecting_no_block_given_a_card_lease_channel_leases_no_card_and_starts_no_card_pool(
) {
    // 假通道在这个进程第一次问调用方的显卡配置之前设好
    let fake_card_lease =
        ExclusiveTemporaryDirectory::create("selecting-no-block-amplification-fake-card-lease")
            .expect("建得了放假领卡客户端的一次性临时目录");
    let calls = common_gpu_cards_of_the_caller::fake_card_lease_channel_refusing_every_lease_set_into_the_process_environment(
        fake_card_lease.path(),
    );
    let flow = common_crash_points::new_pool_file_creation_as_one_crash_point(
        "selecting-no-block-amplification-leases-no-card",
        file!(),
        "new_pool_file_creation",
    );
    // 一次性库：这一趟写进去的不留在全仓那一份里
    let directory = DisposableLibrary::new("selecting-no-block-amplification-leases-no-card")
        .unwrap_or_else(|refusal| panic!("{refusal}"));
    let configuration = directory.join("multi-host-with-gpu.env");
    std::fs::write(&configuration, "ENABLE_GPU=1\n").expect("写得了开 GPU 的配置");
    let _the_library_of_this_process = directory.make_it_the_library_of_this_process();
    let empty_range = format!("{FLOW_NAME}:0-0");
    let environment = CrashAmplificationEnvironment::cleared_then_set(&[
        (
            MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
            configuration.as_os_str(),
        ),
        (
            BLOCK_RANGE_ENVIRONMENT_VARIABLE,
            std::ffi::OsStr::new(empty_range.as_str()),
        ),
    ]);
    let run = common_crash_points::amplify_prepared_flow(
        FLOW_NAME,
        &flow.base,
        &flow.writes,
        &flow.segments,
        flow.judged_root_index,
        &flow.versions,
        flow.crash_points.clone(),
        EXPAND_UP_TO_OF_THE_FLOW,
    );
    drop(environment);
    assert!(
        run.computation_routes.need_a_card(),
        "配置开着 GPU、编进了 gpu 特性：这条流的判段要卡（{:?}），不要卡的话这一条钉不住一块没选中不领卡",
        run.computation_routes
    );
    assert!(
        !run.recorded_only,
        "两段都跑：判段跑到了一块没选中那一处才交回"
    );
    assert!(
        run.no_block_selected_here,
        "空区间：这台机器一块没选中，判段在领卡之前交回"
    );
    assert_eq!(
        (run.cards_used, run.gpu_cards_taken_from),
        (0, GpuCardsSource::NoCardNeeded)
    );
    assert!(
        run.gpu_judge.is_none() && run.verification.is_none(),
        "判器与核对都没起"
    );
    assert!(
        !calls.exists(),
        "一块没选中的那一趟调了领卡客户端：{}",
        std::fs::read_to_string(&calls).unwrap_or_default()
    );
    assert!(
        card_pool_started_in_this_process().is_none(),
        "一块没选中的那一趟起了这个进程的卡池"
    );
    drop(fake_card_lease);
}
