//! 验证包模块：crash_library、crash_amplification、gpu_unit_checks
//! 崩溃放量一趟只录（`SINGLEFS_CRASH_AMPLIFICATION_STAGE=record`）时从头到尾不借卡、不起卡池：配置开着 GPU、编进了 gpu 特性
//! （判段是要卡的那一路），调用方没交卡串而有借卡通道（任务服务交给每次尝试的三个尝试变量），经测试共用的 `common_crash_points::amplify_prepared_flow`
//! 跑完只录那一趟，借卡客户端一回都没被调、这个进程的卡池没起。取卡只在判段（`crash_amplification::run_from_environment` 录完之后那一块），
//! 判段取卡不问借卡通道。没清场时测试共用的取卡帮手（`common_crash_points::cards_of_the_caller_for_a_test_configuration_with_the_gpu`）
//! 不问调用方有没有借卡通道、不动环境，与没有通道的那一趟同一个结局：交 `None`，守卫里环境里认不出通道。
//! 借卡通道是假的（`common_gpu_cards_of_the_caller::fake_card_lease_channel_refusing_every_lease_set_into_the_process_environment`：
//! 每次调用记一行、借卡退 3），不要真卡。调用方的显卡配置与借到的卡进程里只记一回，这份文件只放这一条用例。
#![cfg(feature = "gpu")]

mod common;
mod common_crash_points;
// 取卡帮手那一份共用模块已由 `common_crash_points` 引进来，这里只借它的名字，不再 `mod` 第二遍（同一份文件引两遍 clippy 的 duplicate_mod 判红）。
use common_crash_points::common_gpu_cards_of_the_caller;

use singlefs_verification::crash_amplification::CrashAmplificationEnvironment;
use singlefs_verification::crash_library::{
    DisposableLibrary, ExclusiveTemporaryDirectory, RUN_STAGE_ENVIRONMENT_VARIABLE,
};
use singlefs_verification::gpu_card_configuration::{
    CardLeaseChannelOfTheCaller, GpuQuiesce, MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
};
use singlefs_verification::gpu_unit_checks::gpu_test_start::card_pool_started_in_this_process;

/// 跑的时候的流名：来历表里崩溃放量那一路「两处都不登」的那一条流名，`::` 之后是这条用例的标签（来历表只认 `::` 之前那一截）。
const FLOW_NAME: &str = "line-fields::recording-only-leases-no-card";
/// 这条流自己的展开上限：只为有几块可录，取得小。
const EXPAND_UP_TO_OF_THE_FLOW: usize = 2;

#[test]
fn a_recording_only_crash_amplification_run_given_a_card_lease_channel_leases_no_card_and_starts_no_card_pool(
) {
    // 假通道在这个进程第一次问调用方的显卡配置之前设好
    let fake_card_lease =
        ExclusiveTemporaryDirectory::create("recording-only-amplification-fake-card-lease")
            .expect("建得了放假领卡客户端的一次性临时目录");
    let calls = common_gpu_cards_of_the_caller::fake_card_lease_channel_refusing_every_lease_set_into_the_process_environment(
        fake_card_lease.path(),
    );
    assert!(
        CardLeaseChannelOfTheCaller::read_from_the_environment_now().is_some(),
        "前提：调用方有借卡通道（假通道设进了环境）"
    );
    let flow = common_crash_points::new_pool_file_creation_as_one_crash_point(
        "recording-only-amplification-leases-no-card",
        file!(),
        "new_pool_file_creation",
    );
    // 一次性库：只录的那一趟写进去的不留在全仓那一份里
    let directory = DisposableLibrary::new("recording-only-amplification-leases-no-card")
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
    assert_eq!(
        GpuQuiesce::of_this_process(),
        GpuQuiesce::NotQuiesced,
        "前提：这个进程没在清场之下（交给这一趟的取卡帮手走没清场那一支）"
    );
    let handed_to_the_run =
        common_crash_points::cards_of_the_caller_for_a_test_configuration_with_the_gpu();
    let channel_seen_by_the_run = CardLeaseChannelOfTheCaller::read_from_the_environment_now();
    let handed_nothing = handed_to_the_run.is_none();
    drop(handed_to_the_run);
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
        handed_nothing && channel_seen_by_the_run.is_none(),
        "没清场、调用方有借卡通道：取卡帮手与没有通道时一样交 None、不把通道设回环境（交了没有：{}，这一趟看见的通道：{channel_seen_by_the_run:?}）",
        !handed_nothing
    );
    assert!(
        run.computation_routes.need_a_card(),
        "配置开着 GPU、编进了 gpu 特性：这条流的判段要卡（{:?}），不要卡的话这一条钉不住只录不领",
        run.computation_routes
    );
    assert!(run.recorded_only, "选段是 record：这一趟只录、一块不判");
    assert!(
        !calls.exists(),
        "只录的那一趟调了领卡客户端：{}",
        std::fs::read_to_string(&calls).unwrap_or_default()
    );
    assert!(
        card_pool_started_in_this_process().is_none(),
        "只录的那一趟起了这个进程的卡池"
    );
    drop(fake_card_lease);
}
