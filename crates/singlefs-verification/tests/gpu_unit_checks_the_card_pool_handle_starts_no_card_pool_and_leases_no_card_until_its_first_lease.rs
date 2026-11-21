//! 验证包模块：crash_library、gpu_unit_checks
//! GPU 用例开头取的卡池把手（`gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run` 交的 `GpuCardPoolStartedAtTheFirstLease`）
//! 拿到时只判过这台机器有没有 NVIDIA 的卡：不读显卡配置、不借卡、不起这个进程的卡池；第一次从池里借卡时才起池，起池读调用方的显卡配置。
//! 调用方没交卡串而环境里摆着一条借卡通道：起池判成没卡、借卡那一下交回未跑与「本次未跑」那一行（取的是不打那一行的
//! `GpuCardPoolStartedAtTheFirstLease::lease_card_or_the_not_run_line`，那一行打出来门禁就把这条用例当成本次未跑），
//! 不经通道借卡、不 panic（卡只归 GPU 批处理验证服务，验证包的 GPU 用例不借卡）。借卡通道是假的
//! （`common_gpu_cards_of_the_caller::fake_card_lease_channel_refusing_every_lease_set_into_the_process_environment`：经 `python3` 起，
//! 每次调用记一行参数、借卡退 3），不要真卡；这台机器有没有卡由用例给「有」（`gpu_card_pool_or_report_not_run_given_the_hardware`），
//! 没卡的机器上也走得到起池那一步。起池判成没卡的原因点名调用方没交卡串（`card_pool_not_run_reason_in_this_process`）：
//! 照本机的卡起池的话，有卡的机器上借到卡、没卡的机器上原因是本机没有 NVIDIA 卡，两样都红。
//! 调用方的显卡配置与卡池进程里各只记一回，这份文件只放这一条用例。
#![cfg(feature = "gpu")]

mod common_gpu_cards_of_the_caller;

use singlefs_verification::crash_library::ExclusiveTemporaryDirectory;
use singlefs_verification::gpu_card_configuration::MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE;
use singlefs_verification::gpu_memory_ledger::GPU_NOT_RUN_MARKER;
use singlefs_verification::gpu_unit_checks::gpu_test_start::{
    card_pool_not_run_reason_in_this_process, card_pool_started_in_this_process,
    gpu_card_pool_or_report_not_run_given_the_hardware, NvidiaHardwareOfThisMachine, TakenOrNotRun,
};

const TEST_NAME: &str =
    "a_card_pool_handle_starts_no_card_pool_and_leases_no_card_until_its_first_lease";
/// 借的那一笔（MiB）：只为走到借卡那一步，量取得小。
const MEBIBYTES_LEASED: u64 = 16;

#[test]
fn a_card_pool_handle_starts_no_card_pool_and_leases_no_card_until_its_first_lease() {
    // 调用方的多机配置不设：设了而那份文件不在，起池就报配置不在，钉不到没交卡串那一支
    std::env::remove_var(MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE);
    // 假通道在这个进程第一次问调用方的显卡配置之前设好
    let fake_card_lease = ExclusiveTemporaryDirectory::create("card-pool-handle-fake-card-lease")
        .expect("建得了放假领卡客户端的一次性临时目录");
    let calls = common_gpu_cards_of_the_caller::fake_card_lease_channel_refusing_every_lease_set_into_the_process_environment(
        fake_card_lease.path(),
    );
    let handle_taken = std::panic::catch_unwind(|| {
        gpu_card_pool_or_report_not_run_given_the_hardware(
            TEST_NAME,
            &NvidiaHardwareOfThisMachine::PresentOrToBeJudgedByTheCardPool,
        )
    });
    assert!(
        !calls.exists(),
        "拿卡池把手时就调了领卡客户端：{}",
        std::fs::read_to_string(&calls).unwrap_or_default()
    );
    assert!(
        card_pool_started_in_this_process().is_none()
            && card_pool_not_run_reason_in_this_process().is_none(),
        "拿卡池把手时就起了这个进程的卡池"
    );
    let handle = handle_taken
        .unwrap_or_else(|_payload| {
            panic!("拿卡池把手时 panic 了（拿把手不起池，不该走到起池那一步）")
        })
        .expect("给了这台机器有卡：交得回卡池把手");
    let first_lease = std::panic::catch_unwind(|| {
        match handle.lease_card_or_the_not_run_line(TEST_NAME, MEBIBYTES_LEASED) {
            TakenOrNotRun::Taken(_lease) => None,
            TakenOrNotRun::NotRun { line } => Some(line),
        }
    });
    assert!(
        !calls.exists(),
        "第一次从池里借卡起池时调了借卡客户端：{}",
        std::fs::read_to_string(&calls).unwrap_or_default()
    );
    assert!(
        matches!(&first_lease, Ok(Some(line)) if line.starts_with(&format!("{GPU_NOT_RUN_MARKER} test={TEST_NAME} reason="))),
        "调用方没交卡串：这一借交回未跑与点名这条用例的「本次未跑」那一行、没借到，不 panic（借卡的结局 {first_lease:?}）"
    );
    let not_run_reason = card_pool_not_run_reason_in_this_process();
    assert!(
        not_run_reason.is_some_and(|reason| reason.contains("调用方没交卡串")),
        "第一次借卡起了池、判成没卡，原因点名调用方没交卡串：{not_run_reason:?}"
    );
    drop(fake_card_lease);
}
