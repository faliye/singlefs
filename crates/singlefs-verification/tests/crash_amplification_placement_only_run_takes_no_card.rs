//! 验证包模块：crash_point_replay_progress、crash_library、crash_amplification、gpu_unit_checks
//! 崩溃放量只排派活计划那一趟（`SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY=1`）不取卡：配置开着 GPU（用例自己写的多机配置只写
//! `ENABLE_GPU=1`）、编进了 gpu 特性，环境里没有服务交的卡（`GPU_CARDS_BY_PRIORITY`），也没有领卡通道（环境守卫清掉了），两段照跑：
//! 录入段录进一次性库，排计划那一趟只数要判的块与状态，打出 `CRASH_AMPLIFICATION_PLACEMENT` 那一行，不报没经服务给卡、不起这个进程的卡池。
//! 直接调 `crash_amplification::run_from_environment`，不经测试共用的那一层（那一层在清过场的进程里会把调用方的卡交给这一趟）。
//! 要 `gpu` 特性，不带它这份文件是空的。
#![cfg(feature = "gpu")]

mod common;
mod common_crash_points;

use singlefs_verification::crash_amplification::{
    run_from_environment, CrashAmplificationEnvironment, CrashFlow, GpuCardsSource,
    PLACEMENT_LINE_PREFIX,
};
use singlefs_verification::crash_library::DisposableLibrary;
use singlefs_verification::crash_point_replay_progress::CrashPointReplayToolchainIdentity;
use singlefs_verification::gpu_card_configuration::MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE;
use singlefs_verification::gpu_unit_checks::gpu_test_start::card_pool_started_in_this_process;

/// 跑的时候的流名：来历表里崩溃放量那一路「两处都不登」的那一条流名，`::` 之后是这条用例的标签（来历表只认 `::` 之前那一截）。
const FLOW_NAME: &str = "line-fields::placement-only-takes-no-card";
/// 这条流自己的展开上限：只为有几块可数，取得小。
const EXPAND_UP_TO_OF_THE_FLOW: usize = 2;

/// 一行里 `field=` 之后的那个数（行里没有这个字段、或不是数时 `None`）。
fn number_of_the_field(line: &str, field: &str) -> Option<u64> {
    line.split(' ')
        .find_map(|pair| pair.strip_prefix(&format!("{field}=")))
        .and_then(|value| value.parse().ok())
}

/// 场景：新池新建文件那一段当一条流，一次性库从空库起；配置开 GPU、环境里没有服务交的卡与领卡通道，只排派活计划。
/// 预期：这一趟交回、没叫停；打了一行 PLACEMENT（那个头起头，栏名与次序照钉住的那几栏，没有按卡排的栏；第 ① 段要判的块与状态不为 0，
/// 复用数与这一趟交回的相同；空库上第 ② 段一块都没核过，要核的块与状态等于第 ① 段要判的）；卡的来路是不要卡；这个进程的卡池没起。
/// 判别力（`crates/mutations.tsv`）：只排计划那一趟先照多机配置取卡（没经服务给卡就交错，这一趟交不回），这条红。
#[test]
fn a_placement_only_run_with_the_gpu_configured_and_no_card_handed_out_prints_its_line_without_a_card(
) {
    let flow = common_crash_points::new_pool_file_creation_as_one_crash_point(
        "placement-only-takes-no-card",
        file!(),
        "new_pool_file_creation",
    );
    // 一次性库：这一趟写进去的不留在全仓那一份里
    let directory = DisposableLibrary::new("placement-only-takes-no-card")
        .unwrap_or_else(|refusal| panic!("{refusal}"));
    let configuration = directory.join("multi-host-with-gpu.env");
    std::fs::write(&configuration, "ENABLE_GPU=1\n").expect("写得了开 GPU 的配置");
    let _the_library_of_this_process = directory.make_it_the_library_of_this_process();
    // 先开库、后拿改环境的锁；守卫清掉了服务交的卡与领卡通道，只设多机配置与只排计划
    let environment = CrashAmplificationEnvironment::cleared_then_set(&[
        (
            MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
            configuration.as_os_str(),
        ),
        (
            "SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY",
            std::ffi::OsStr::new("1"),
        ),
    ]);
    let limit = common_crash_points::expand_up_to_of_the_flow_narrowed_by_the_environment(
        EXPAND_UP_TO_OF_THE_FLOW,
    );
    let expansion = move |_segment_index: usize, segment: &[usize]| limit.expansion_of(segment);
    let run = run_from_environment(
        &CrashFlow {
            base: &flow.base,
            writes: &flow.writes,
            segments: &flow.segments,
            judged_root_index: flow.judged_root_index,
            versions: &flow.versions,
            versions_without_a_file: &[],
            expansion: &expansion,
            crash_points: flow.crash_points.clone(),
        },
        FLOW_NAME,
        &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
        limit,
    )
    .expect("只排派活计划那一趟不取卡：配置开了 GPU、环境里没有服务交的卡也交得回");
    drop(environment);
    assert!(
        run.computation_routes.need_a_card(),
        "前提：配置开着 GPU、编进了 gpu 特性，判器走 GPU：{:?}",
        run.computation_routes
    );
    assert!(
        run.placement_only && !run.stopped_early,
        "只排了派活计划、没叫停"
    );
    let line = run
        .placement_line
        .as_deref()
        .expect("只排计划那一趟打了 PLACEMENT 那一行");
    assert!(
        line.starts_with(&format!("{PLACEMENT_LINE_PREFIX} judge=gpu ")),
        "那一行有固定的头：{line}"
    );
    let pending_states = number_of_the_field(line, "pending_states").unwrap_or(0);
    assert!(
        pending_states > 0 && number_of_the_field(line, "pending_blocks").unwrap_or(0) > 0,
        "空库起跑：要判的块与状态不为 0：{line}"
    );
    assert_eq!(
        number_of_the_field(line, "states_reused"),
        Some(run.states_reused),
        "复用数照实报：{line}"
    );
    assert_eq!(
        line.split(' ')
            .skip(1)
            .map(|pair| pair.split_once('=').map_or(pair, |(field, _value)| field))
            .collect::<Vec<&str>>(),
        [
            "judge",
            "pending_blocks",
            "pending_states",
            "states_reused",
            "tables_mebibytes",
            "verifier_pending_blocks",
            "verifier_pending_states",
        ],
        "栏名与次序照钉住的那几栏，没有按卡排的栏：{line}"
    );
    assert_eq!(
        (
            number_of_the_field(line, "verifier_pending_blocks"),
            number_of_the_field(line, "verifier_pending_states")
        ),
        (
            number_of_the_field(line, "pending_blocks"),
            Some(pending_states)
        ),
        "空库：第 ② 段一块都没核过，要核的块与状态等于第 ① 段要判的：{line}"
    );
    assert_eq!(
        run.gpu_cards_taken_from,
        GpuCardsSource::NoCardNeeded,
        "这一趟不取卡"
    );
    assert!(
        card_pool_started_in_this_process().is_none(),
        "只排计划那一趟起了这个进程的卡池"
    );
}
