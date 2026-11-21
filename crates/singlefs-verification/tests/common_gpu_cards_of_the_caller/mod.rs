//! 用例自己写开 GPU 的多机配置、在环境守卫里起一趟（崩溃放量一趟、给定状态引擎）时交给这一趟的卡：调用方给这个进程的那一份
//! （`gpu_card_configuration::GpuConfigurationOfTheCaller`：服务交的卡串）。
//! 为什么经环境变量、不抄进用例的配置：配置里的卡不拿来挑卡，环境里没有卡串（没设、空串或只有空白）而配置开了 GPU，清没清场那一趟的判段取卡都照没经服务给卡那一句拒
//! （`gpu_card_configuration::cards_by_priority_from`；判段取卡不问借卡通道、不借卡）。卡串交了那一趟就照它，
//! 与卡池照调用方那一份挑的卡对得上；没交卡串的不设、不借卡（[`cards_of_the_caller_handed_to_the_run`]）。
//! 经 `run_from_environment` 跑的那一路在 `common_crash_points` 里照同一个口子交（`cards_of_the_caller_for_a_test_configuration_with_the_gpu`）。
//! 卡池起池照调用方那一份挑卡（`gpu_unit_checks::gpu_test_start`），调用方没交卡串的判成没卡、不借卡：用例设的假借卡通道
//! （[`fake_card_lease_channel_refusing_every_lease_set_into_the_process_environment`]）钉的是起池那一刻、判段都不借。
#![allow(
    dead_code,
    reason = "共用的取卡模块：有的测试文件把卡交给这一趟，有的只设假借卡通道，没调的那一个在那个测试二进制里就是没用到"
)]

use std::path::{Path, PathBuf};

use singlefs_verification::crash_amplification::EnvironmentRestoredOnDrop;
use singlefs_verification::gpu_card_configuration::{
    CardLeaseChannelOfTheCaller, GpuConfigurationOfTheCaller, CARDS_BY_PRIORITY_KEY,
    CARD_LEASE_ATTEMPT_ENVIRONMENT_VARIABLES, CARD_LEASE_CLIENT_PATH_IN_THE_REPOSITORY,
    TASK_SERVER_SOURCE_REPOSITORY_ENVIRONMENT_VARIABLE,
};

/// 假借卡通道的三个尝试变量（[`fake_card_lease_channel_refusing_every_lease_set_into_the_process_environment`] 设进环境的）：
/// 尝试标签、任务号、第几次尝试，照 [`CARD_LEASE_ATTEMPT_ENVIRONMENT_VARIABLES`] 的次序；核这个进程记下的调用方通道就是假的那一条。
pub const ATTEMPT_VALUES_OF_THE_FAKE_CARD_LEASE_CHANNEL: [&str; 3] =
    ["attempt-of-the-fake-card-lease-channel", "4093", "1"];

/// 假的借卡通道设进这个进程的环境：借卡客户端是 `directory` 下一份假仓里的 `gpu-verification-server/control`
/// （任务服务交仓根的那个变量指到这份假仓），是 python 脚本、不带执行位（只有经 `python3 <客户端>` 起才跑得起来）。
/// 每回调用往 `directory` 下的 `calls.log` 记一行参数（空格隔开），`lease take` 退 3（服务拒了）、别的调用退 0。借卡通道的三个尝试变量设成
/// [`ATTEMPT_VALUES_OF_THE_FAKE_CARD_LEASE_CHANNEL`]，服务交的卡串（`GPU_CARDS_BY_PRIORITY`）删掉。不要真卡：借卡总被拒，
/// 借没借看调用记录（验证包里没有一处该借，一行都不该有）。
/// 要在这个进程第一次问调用方的显卡配置（`GpuConfigurationOfTheCaller::of_this_process`）之前调：那一份进程里只记一次，
/// 之后在环境里设的它看不见；这里设完就问一次、核记下的那一份没有卡串，再核此刻环境里认得出的通道就是假通道的这三个变量。
/// 卡池进程里也只起一回，用它的测试文件只放一条用例。
/// 交回调用记录的路径（一回都没调就不存在）。
///
/// # Panics
/// 写不了假客户端；这个进程早先已经记下了调用方的显卡配置（记下的那一份带着服务交的卡串），或此刻环境里认出的通道不是这个假通道。
pub fn fake_card_lease_channel_refusing_every_lease_set_into_the_process_environment(
    directory: &Path,
) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let calls = directory.join("calls.log");
    let fake_repository = directory.join("fake-repository");
    let client = fake_repository.join(CARD_LEASE_CLIENT_PATH_IN_THE_REPOSITORY);
    std::fs::create_dir_all(client.parent().expect("客户端在假仓的一个目录下"))
        .expect("建得了假客户端的目录");
    std::fs::write(
        &client,
        format!(
            "import sys\n\
             with open('{}', 'a') as calls:\n    \
                 calls.write(' '.join(sys.argv[1:]) + '\\n')\n\
             sys.exit(3 if sys.argv[1:3] == ['lease', 'take'] else 0)\n",
            calls.display()
        ),
    )
    .expect("写得了假客户端");
    std::fs::set_permissions(&client, std::fs::Permissions::from_mode(0o644))
        .expect("假客户端设得了不带执行位");
    std::env::remove_var(CARDS_BY_PRIORITY_KEY);
    for (name, value) in CARD_LEASE_ATTEMPT_ENVIRONMENT_VARIABLES
        .iter()
        .zip(ATTEMPT_VALUES_OF_THE_FAKE_CARD_LEASE_CHANNEL)
    {
        std::env::set_var(name, value);
    }
    std::env::set_var(
        TASK_SERVER_SOURCE_REPOSITORY_ENVIRONMENT_VARIABLE,
        fake_repository.as_os_str(),
    );
    let caller = GpuConfigurationOfTheCaller::of_this_process();
    let attempt_values_in_the_environment: Option<Vec<(&str, std::ffi::OsString)>> =
        CardLeaseChannelOfTheCaller::read_from_the_environment_now().map(|channel| {
            channel
                .variables_for_the_client()
                .iter()
                .map(|(name, value)| (*name, value.clone()))
                .collect()
        });
    let attempt_values_of_the_fake_channel: Vec<(&str, std::ffi::OsString)> =
        CARD_LEASE_ATTEMPT_ENVIRONMENT_VARIABLES
            .iter()
            .zip(ATTEMPT_VALUES_OF_THE_FAKE_CARD_LEASE_CHANNEL)
            .map(|(name, value)| (*name, std::ffi::OsString::from(value)))
            .collect();
    assert!(
        attempt_values_in_the_environment.as_ref() == Some(&attempt_values_of_the_fake_channel)
            && caller.cards_handed_out_by_the_service().is_none(),
        "这个进程记下的调用方显卡配置带着卡串（假通道设进环境之前已经有人问过），或此刻环境里认出的通道不是假通道：\
         {caller:?}，环境里的通道 {attempt_values_in_the_environment:?}"
    );
    calls
}

/// 把调用方给这个进程的卡交给这一趟，不借卡：调用方交了卡串的，原样设 `GPU_CARDS_BY_PRIORITY`；没交卡串的不设
/// （这一趟的判段取卡照没经服务给卡办：要卡的那一趟拒，只录的那一趟、录入段都不取卡；交没交借卡通道都一样，不设回、不借）。
/// 交回的守卫丢掉时还原；在用例的环境守卫里建、先于它丢，那一趟在它活着时起步。
pub fn cards_of_the_caller_handed_to_the_run() -> EnvironmentRestoredOnDrop {
    let mut handed_over = EnvironmentRestoredOnDrop::saved_then_cleared(&[CARDS_BY_PRIORITY_KEY]);
    if let Some(cards) =
        GpuConfigurationOfTheCaller::of_this_process().cards_handed_out_by_the_service()
    {
        handed_over.set_variable(CARDS_BY_PRIORITY_KEY, std::ffi::OsStr::new(cards));
    }
    handed_over
}
