//! 验证包模块：crash、crash_library、crash_amplification、gpu_unit_checks、crash_judge_tables、crash_judge_dispatch
//! GPU 对拍用例的只录那一趟（`SINGLEFS_CRASH_AMPLIFICATION_STAGE=record`）拿着卡池把手从头到尾不起卡池、不领卡：用例开头取卡池把手
//! （`gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run` 一族，这台机器有没有卡由用例给「有」，没卡的机器上也走得到），
//! CPU 参照那张表过对拍那一类的库（`gpu_test_start::cpu_reference_table_read_back_from_the_pairing_library`）只录、交 `None`，
//! GPU 那一侧的对拍帮手（`crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool`）判表过库只录、交 `None`：
//! 领卡客户端一回都没被调、这个进程的卡池没起。领卡通道是假的
//! （`common_gpu_cards_of_the_caller::fake_card_lease_channel_refusing_every_lease_set_into_the_process_environment`：每次调用记一行、
//! 领卡退 3），不要真卡；库是一次性库，只录的那一趟写进去的不留在全仓那一份里。调用方的显卡配置、领到的卡与卡池进程里各只记一回，
//! 这份文件只放这一条用例。
#![cfg(feature = "gpu")]

mod common;
mod common_gpu_cards_of_the_caller;

use singlefs_verification::crash::CrashPointReplaySegmentExpansion;
use singlefs_verification::crash_amplification::{
    CrashAmplificationEnvironment, RecordedGivenStatesTable,
};
use singlefs_verification::crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool;
use singlefs_verification::crash_judge_tables::{
    build_judge_tables_for_the_given_persisted_sets, JudgeFlow,
};
use singlefs_verification::crash_library::{
    DisposableLibrary, ExclusiveTemporaryDirectory, RUN_STAGE_ENVIRONMENT_VARIABLE,
};
use singlefs_verification::gpu_unit_checks::gpu_test_start::{
    card_pool_started_in_this_process, cpu_reference_table_read_back_from_the_pairing_library,
    gpu_card_pool_or_report_not_run_given_the_hardware, NvidiaHardwareOfThisMachine,
};

const TEST_NAME: &str =
    "a_recording_only_pairing_through_the_card_pool_handle_starts_no_card_pool_and_leases_no_card";

#[test]
fn a_recording_only_pairing_through_the_card_pool_handle_starts_no_card_pool_and_leases_no_card() {
    // 假通道在这个进程第一次问调用方的显卡配置之前设好
    let fake_card_lease =
        ExclusiveTemporaryDirectory::create("recording-only-pairing-fake-card-lease")
            .expect("建得了放假领卡客户端的一次性临时目录");
    let calls = common_gpu_cards_of_the_caller::fake_card_lease_channel_refusing_every_lease_set_into_the_process_environment(
        fake_card_lease.path(),
    );
    let image = common::build_pool("recording-only-pairing-starts-no-card-pool").memory_pool();
    let not_expanded =
        |_segment_index: usize, _segment: &[usize]| CrashPointReplaySegmentExpansion::NotExpanded;
    let tables = build_judge_tables_for_the_given_persisted_sets(
        &JudgeFlow {
            base: &image,
            writes: &[],
            segments: &[],
            judged_root_index: 0,
            versions: &[],
            expansion: &not_expanded,
        },
        &[],
        vec![Vec::new()],
    )
    .unwrap_or_else(|refusal| panic!("建不出判表：{refusal:?}"));
    let directory = DisposableLibrary::new("recording-only-pairing-starts-no-card-pool")
        .unwrap_or_else(|refusal| panic!("{refusal}"));
    let the_library_of_this_process = directory.make_it_the_library_of_this_process();
    let environment = CrashAmplificationEnvironment::cleared_then_set(&[(
        RUN_STAGE_ENVIRONMENT_VARIABLE,
        std::ffi::OsStr::new("record"),
    )]);
    let cards = gpu_card_pool_or_report_not_run_given_the_hardware(
        TEST_NAME,
        &NvidiaHardwareOfThisMachine::PresentOrToBeJudgedByTheCardPool,
    )
    .expect("给了这台机器有卡：交得回卡池把手");
    let cpu_reference =
        cpu_reference_table_read_back_from_the_pairing_library(&RecordedGivenStatesTable {
            table_name: TEST_NAME.to_string(),
            base: image,
            writes: Vec::new(),
            segments: Vec::new(),
            seams: Vec::new(),
            persisted_sets: vec![Vec::new()],
        });
    let judged_on_the_gpu = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        judge_every_state_on_a_card_leased_from_the_pool(cards, TEST_NAME, &tables)
    }));
    drop(environment);
    drop(the_library_of_this_process);
    assert!(
        !calls.exists(),
        "只录的那一趟调了领卡客户端（GPU 那一侧的对拍帮手 panic 了没有：{}）：{}",
        judged_on_the_gpu.is_err(),
        std::fs::read_to_string(&calls).unwrap_or_default()
    );
    assert!(
        card_pool_started_in_this_process().is_none(),
        "只录的那一趟起了这个进程的卡池"
    );
    assert!(
        cpu_reference.is_none(),
        "选段是 record：CPU 参照那张表只录、不读回"
    );
    assert!(
        matches!(judged_on_the_gpu, Ok(None)),
        "选段是 record：GPU 那一侧的对拍帮手判表只录、交 None（panic 了没有：{}）",
        judged_on_the_gpu.is_err()
    );
    drop(fake_card_lease);
}
