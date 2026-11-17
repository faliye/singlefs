//! 验证包模块：crash_library、gpu_unit_checks
//! GPU 用例开头取的卡池把手（`gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run` 交的 `GpuCardPoolStartedAtTheFirstLease`）
//! 拿到时只判过这台机器有没有 NVIDIA 的卡：不读显卡配置、不领卡、不起这个进程的卡池；第一次借卡时才起池，起池读调用方的显卡配置，
//! 没交卡串而交了领卡通道的在那一刻领卡。领卡通道是假的
//! （`common_gpu_cards_of_the_caller::fake_card_lease_channel_refusing_every_lease_set_into_the_process_environment`：每次调用记一行、
//! 领卡退 3），不要真卡；这台机器有没有卡由用例给「有」（`gpu_card_pool_or_report_not_run_given_the_hardware`），没卡的机器上也走得到借卡起池那一步。
//! 领卡总被拒：有卡的机器上起池判成有卡而用不上、panic，没卡的机器上判成没卡、借不到，两样都接住，只看领没领、什么时候领。
//! 调用方的显卡配置、领到的卡与卡池进程里各只记一回，这份文件只放这一条用例。
#![cfg(feature = "gpu")]

mod common_gpu_cards_of_the_caller;

use singlefs_verification::crash_library::ExclusiveTemporaryDirectory;
use singlefs_verification::gpu_card_configuration::MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE;
use singlefs_verification::gpu_unit_checks::gpu_test_start::{
    card_pool_started_in_this_process, gpu_card_pool_or_report_not_run_given_the_hardware,
    NvidiaHardwareOfThisMachine,
};

const TEST_NAME: &str =
    "a_card_pool_handle_starts_no_card_pool_and_leases_no_card_until_its_first_lease";
/// 借的那一笔（MiB）：只为走到借卡那一步，量取得小。
const MEBIBYTES_LEASED: u64 = 16;

#[test]
fn a_card_pool_handle_starts_no_card_pool_and_leases_no_card_until_its_first_lease() {
    // 调用方的多机配置不设：设了而那份文件不在，起池在领卡之前就报配置不在，钉不到领卡
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
        card_pool_started_in_this_process().is_none(),
        "拿卡池把手时就起了这个进程的卡池"
    );
    let handle = handle_taken
        .unwrap_or_else(|_payload| {
            panic!("拿卡池把手时 panic 了（拿把手不起池，不该走到起池那一步）")
        })
        .expect("给了这台机器有卡：交得回卡池把手");
    let first_lease = std::panic::catch_unwind(|| {
        handle
            .lease_card_or_report_not_run(TEST_NAME, MEBIBYTES_LEASED)
            .is_some()
    });
    let calls_after_the_first_lease = std::fs::read_to_string(&calls).unwrap_or_default();
    assert_eq!(
        calls_after_the_first_lease, "lease take\n",
        "第一次借卡起池、照领卡通道领卡：领卡客户端恰好被调一回、是领卡（借卡的结局 {first_lease:?}）"
    );
    assert!(!matches!(first_lease, Ok(true)), "领卡被拒：这一借借不到卡");
    drop(fake_card_lease);
}
