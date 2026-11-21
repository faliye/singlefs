//! 验证包模块：crash_point_replay_progress、crash_library、crash_amplification、verdict_store、gpu_unit_checks
//! 崩溃放量一趟两段先后跑（不设选段）时一张卡都不借：配置开着 GPU、编进了 gpu 特性，调用方没交卡串而有借卡通道
//! （假借卡客户端 `lease take` 退 3）；交给这一趟的卡照调用方那一份（`common_gpu_cards_of_the_caller::cards_of_the_caller_handed_to_the_run`：没交卡串不设、通道不设回）。
//! 这一趟录进库之后，判段第 ① 段的库在临时 prune-server 里（一次性库），直接在真服务里登记那份临时库、写待判行、等 GPU 批处理服务判完
//! （不问服务收不收；服务读不了这一份就点名跳过，等不到的由统一任务服务照卡住停）：这一趟交回判完、不是只录；
//! 库里有这一趟的录入清单，借卡客户端一回都没被调，这个进程的卡池没起。再以只录跑一次同一条流，不借卡、照样交回只录。
//! 取卡挪回录入之前的话，这一趟在录入之前报没经服务给卡（判段取卡不问借卡通道），交回错、库里没有录入清单，这一条红；
//! 判段不交服务、自己借卡的话，借卡客户端被调过，这一条红。
//! 借卡通道是假的（`common_gpu_cards_of_the_caller::fake_card_lease_channel_refusing_every_lease_set_into_the_process_environment`），不要真卡。
//! 调用方的显卡配置与借卡的结局进程里只记一回，这份文件只放这一条用例。
#![cfg(feature = "gpu")]

mod common;
mod common_crash_points;
// 取卡帮手那一份共用模块已由 `common_crash_points` 引进来，这里只借它的名字，不再 `mod` 第二遍（同一份文件引两遍 clippy 的 duplicate_mod 判红）。
use common_crash_points::common_gpu_cards_of_the_caller;

use singlefs_verification::crash_amplification::{
    run_from_environment, CrashAmplificationEnvironment, CrashFlow, ExpandUpToWritesPerSegment,
};
use singlefs_verification::crash_library::{
    DisposableLibrary, ExclusiveTemporaryDirectory, RUN_STAGE_ENVIRONMENT_VARIABLE,
};
use singlefs_verification::crash_point_replay_progress::CrashPointReplayToolchainIdentity;
use singlefs_verification::gpu_card_configuration::MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE;
use singlefs_verification::gpu_unit_checks::gpu_test_start::card_pool_started_in_this_process;
use singlefs_verification::verdict_store::PruneServerFailureHandling;

/// 跑的时候的流名：来历表里崩溃放量那一路「两处都不登」的那一条流名，`::` 之后是这条用例的标签（来历表只认 `::` 之前那一截）。
const FLOW_NAME: &str = "line-fields::both-stages-lease-after-recording";
/// 这条流的展开上限：只为有几块可录，取得小。
const EXPAND_UP_TO_OF_THE_FLOW: ExpandUpToWritesPerSegment = ExpandUpToWritesPerSegment(2);

#[test]
fn a_crash_amplification_run_of_both_stages_records_the_flow_and_leases_no_card() {
    // 假通道在这个进程第一次问调用方的显卡配置之前设好
    let fake_card_lease =
        ExclusiveTemporaryDirectory::create("both-stages-amplification-fake-card-lease")
            .expect("建得了放假领卡客户端的一次性临时目录");
    let calls = common_gpu_cards_of_the_caller::fake_card_lease_channel_refusing_every_lease_set_into_the_process_environment(
        fake_card_lease.path(),
    );
    let recorded = common_crash_points::new_pool_file_creation_as_one_crash_point(
        "both-stages-amplification-records-before-leasing",
        file!(),
        "new_pool_file_creation",
    );
    let expansion = move |_segment_index: usize, segment: &[usize]| {
        EXPAND_UP_TO_OF_THE_FLOW.expansion_of(segment)
    };
    let flow = || CrashFlow {
        base: &recorded.base,
        writes: &recorded.writes,
        segments: &recorded.segments,
        judged_root_index: recorded.judged_root_index,
        versions: &recorded.versions,
        versions_without_a_file: &[],
        expansion: &expansion,
        crash_points: recorded.crash_points.clone(),
    };
    let toolchain = CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test();
    // 一次性库：数这一趟录进库的录入清单，全仓那一份一行不写
    let directory = DisposableLibrary::new("both-stages-amplification-records-before-leasing")
        .unwrap_or_else(|refusal| panic!("{refusal}"));
    let configuration = directory.join("multi-host-with-gpu.env");
    std::fs::write(&configuration, "ENABLE_GPU=1\n").expect("写得了开 GPU 的配置");
    let recorded_run_manifests = || {
        directory
            .open_store(PruneServerFailureHandling::ReturnTheError)
            .unwrap_or_else(|refusal| panic!("开不了这条用例的库：{refusal}"))
            .recorded_run_manifest_count()
            .unwrap_or_else(|error| panic!("数不了录入清单：{error:?}"))
    };
    let the_library_of_this_process = directory.make_it_the_library_of_this_process();
    let mut environment = CrashAmplificationEnvironment::cleared_then_set(&[(
        MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
        configuration.as_os_str(),
    )]);
    // 调用方的借卡通道交给这一趟、不在这里借（守卫清掉了通道的三个尝试变量，这里照调用方的值设回）
    let cards_of_the_caller =
        common_gpu_cards_of_the_caller::cards_of_the_caller_handed_to_the_run();
    let run_of_both_stages =
        run_from_environment(&flow(), FLOW_NAME, &toolchain, EXPAND_UP_TO_OF_THE_FLOW)
            .unwrap_or_else(|reason| {
                panic!("两段先后跑：录进库、判段交 GPU 批处理服务判完：{reason}")
            });
    let manifests_after_both_stages = recorded_run_manifests();
    let calls_after_both_stages = std::fs::read_to_string(&calls).unwrap_or_default();
    environment.set_variable(
        RUN_STAGE_ENVIRONMENT_VARIABLE,
        std::ffi::OsStr::new("record"),
    );
    let recording_only_run =
        run_from_environment(&flow(), FLOW_NAME, &toolchain, EXPAND_UP_TO_OF_THE_FLOW)
            .unwrap_or_else(|reason| panic!("再以只录跑一次同一条流：{reason}"));
    drop(cards_of_the_caller);
    drop(environment);
    let manifests_after_the_recording_only_run = recorded_run_manifests();
    drop(the_library_of_this_process);
    assert!(
        !run_of_both_stages.recorded_only,
        "两段先后跑的这一趟判段交服务判完，不是只录"
    );
    assert_eq!(
        calls_after_both_stages, "",
        "两段先后跑的这一趟没调领卡客户端"
    );
    assert_eq!(
        manifests_after_both_stages, 1,
        "这条流已经录进库：库里有这一趟的录入清单"
    );
    assert!(
        recording_only_run.recorded_only,
        "再以只录跑一次：只录、一块不判"
    );
    assert_eq!(
        manifests_after_the_recording_only_run, 1,
        "再以只录跑一次同一条流：录入键还是那一把，库里仍是一份录入清单"
    );
    assert_eq!(
        std::fs::read_to_string(&calls).unwrap_or_default(),
        "",
        "只录的那一趟也没调领卡客户端"
    );
    assert!(
        card_pool_started_in_this_process().is_none(),
        "两段先后跑、只录：这个进程的卡池一直没起"
    );
    drop(fake_card_lease);
}
