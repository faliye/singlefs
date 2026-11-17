//! 验证包模块：crash_point_replay_progress、crash_library、crash_amplification、verdict_store、gpu_unit_checks
//! 崩溃放量一趟两段先后跑（不设选段）时，领卡在录完之后：配置开着 GPU、编进了 gpu 特性，调用方没交卡串而交了领卡通道，
//! 领卡被服务拒（假领卡客户端 `lease take` 退 3）。这一趟在判段取卡时报领卡的错，而报错之前这条流已经录进库
//! （库里有这一趟的录入清单）；领卡客户端只被调了一回、是领卡；这个进程的卡池没起。再以只录跑一次同一条流，不领卡、照样交回只录。
//! 领卡挪回录入之前（开库之前）的话，这一趟在录入之前就报错，库里一份录入清单都没有，这一条红。
//! 领卡通道是假的（`common_gpu_cards_of_the_caller::fake_card_lease_channel_refusing_every_lease_set_into_the_process_environment`），不要真卡。
//! 调用方的显卡配置与领卡的结局进程里只记一回，这份文件只放这一条用例。
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
fn a_crash_amplification_run_of_both_stages_records_the_flow_before_its_card_lease_is_refused() {
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
    // 调用方的领卡通道交给这一趟、不在这里领（守卫清掉了通道目录与凭据）
    let cards_of_the_caller =
        common_gpu_cards_of_the_caller::cards_of_the_caller_handed_to_the_run();
    let refusal = run_from_environment(&flow(), FLOW_NAME, &toolchain, EXPAND_UP_TO_OF_THE_FLOW)
        .expect_err("领卡被拒：两段先后跑的这一趟到判段取卡时报错");
    let manifests_when_the_lease_was_refused = recorded_run_manifests();
    let calls_when_the_lease_was_refused = std::fs::read_to_string(&calls).unwrap_or_default();
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
        refusal.contains("lease take 退 3"),
        "两段先后跑的这一趟报的是领卡被拒：{refusal}"
    );
    assert_eq!(
        calls_when_the_lease_was_refused, "lease take\n",
        "领卡客户端只被调了一回、是领卡"
    );
    assert_eq!(
        manifests_when_the_lease_was_refused, 1,
        "领卡被拒之前这条流已经录进库：库里有这一趟的录入清单"
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
        "lease take\n",
        "只录的那一趟没再调领卡客户端"
    );
    assert!(
        card_pool_started_in_this_process().is_none(),
        "领卡被拒、只录：这个进程的卡池一直没起"
    );
    drop(fake_card_lease);
}
