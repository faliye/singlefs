//! 用例自己写开 GPU 的多机配置、在环境守卫里起一趟（崩溃放量一趟、给定状态引擎）时交给这一趟的卡：调用方给这个进程的那一份
//! （`gpu_card_configuration::GpuConfigurationOfTheCaller`：服务分卡时经环境交的卡串，或按需领卡的尝试交的领卡通道）。
//! 为什么经环境变量、不抄进用例的配置：配置里的卡不拿来挑卡，环境里既没有卡串也没有通道而配置开了 GPU，清没清场那一趟都照「清场与分卡由服务做」拒
//! （`gpu_card_configuration::cards_by_priority_from`）。卡串设了（空串也算：本机能用的卡全用）那一趟就照它，与卡池照调用方那一份挑的卡对得上；
//! 交的是通道的不在这里领卡，那一趟第一次真要在卡上判时才领（[`cards_of_the_caller_handed_to_the_run`]）。
//! 经 `run_from_environment` 跑的那一路在 `common_crash_points` 里照同一个口子交（`cards_of_the_caller_for_a_test_configuration_with_the_gpu`）。
#![allow(
    dead_code,
    reason = "共用的取卡模块：有的测试文件把卡交给这一趟，有的只设假领卡通道，没调的那一个在那个测试二进制里就是没用到"
)]

use std::path::{Path, PathBuf};

use singlefs_verification::crash_amplification::EnvironmentRestoredOnDrop;
use singlefs_verification::gpu_card_configuration::{
    GpuConfigurationOfTheCaller, CARDS_BY_PRIORITY_KEY, CARD_LEASE_ATTEMPT_ENVIRONMENT_VARIABLES,
    CARD_LEASE_CLIENT_PATH_IN_THE_REPOSITORY, CARD_LEASE_DIRECTORY_ENVIRONMENT_VARIABLE,
    CARD_LEASE_TOKEN_ENVIRONMENT_VARIABLE, TASK_SERVER_SOURCE_REPOSITORY_ENVIRONMENT_VARIABLE,
};

/// 假领卡通道的凭据（[`fake_card_lease_channel_refusing_every_lease_set_into_the_process_environment`]）：
/// 核这个进程记下的调用方通道就是假的那一个。
pub const TOKEN_OF_THE_FAKE_CARD_LEASE_CHANNEL: &str = "token-of-the-fake-card-lease-channel";

/// 假的领卡通道设进这个进程的环境：领卡客户端是 `directory` 下一份假的仓里的 `task-server/task`（服务交仓根的那个变量指到这份假仓），
/// 每次调用往 `directory` 下的 `calls.log` 记一行参数，`lease take` 退 3（服务拒了）、别的调用退 0；通道的五个变量设成认得出的字面，
/// 服务交的卡串（`GPU_CARDS_BY_PRIORITY`）删掉（交任务时服务可能起跑就交了卡）。不要真卡：领卡总被拒，领没领、什么时候领看调用记录。
/// 要在这个进程第一次问调用方的显卡配置（`GpuConfigurationOfTheCaller::of_this_process`）之前调：那一份进程里只记一次，
/// 之后在环境里设的它看不见；这里设完就问一次、核记下的是这个假通道。领卡进程里也只领一回，用它的测试文件只放一条用例。
/// 交回调用记录的路径（一回都没调就不存在）。
///
/// # Panics
/// 写不了假客户端；这个进程早先已经记下了调用方的显卡配置（记下的不是这个假通道、或带着服务交的卡串）。
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
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{}'\n[ \"$1 $2\" = 'lease take' ] || exit 0\nexit 3\n",
            calls.display()
        ),
    )
    .expect("写得了假客户端");
    std::fs::set_permissions(&client, std::fs::Permissions::from_mode(0o755))
        .expect("假客户端设得了可执行");
    std::env::remove_var(CARDS_BY_PRIORITY_KEY);
    std::env::set_var(CARD_LEASE_DIRECTORY_ENVIRONMENT_VARIABLE, directory);
    std::env::set_var(
        CARD_LEASE_TOKEN_ENVIRONMENT_VARIABLE,
        TOKEN_OF_THE_FAKE_CARD_LEASE_CHANNEL,
    );
    for (name, value) in CARD_LEASE_ATTEMPT_ENVIRONMENT_VARIABLES.iter().zip([
        "attempt-of-the-fake-card-lease-channel",
        "4093",
        "1",
    ]) {
        std::env::set_var(name, value);
    }
    std::env::set_var(
        TASK_SERVER_SOURCE_REPOSITORY_ENVIRONMENT_VARIABLE,
        fake_repository.as_os_str(),
    );
    let caller = GpuConfigurationOfTheCaller::of_this_process();
    let recorded_token = caller.card_lease_channel().and_then(|channel| {
        channel
            .variables_for_the_client()
            .iter()
            .find(|(name, _)| *name == CARD_LEASE_TOKEN_ENVIRONMENT_VARIABLE)
            .map(|(_, value)| value.clone())
    });
    assert!(
        recorded_token.is_some_and(|token| {
            token.as_os_str() == std::ffi::OsStr::new(TOKEN_OF_THE_FAKE_CARD_LEASE_CHANNEL)
        }) && caller.cards_handed_out_by_the_service().is_none(),
        "这个进程记下的调用方显卡配置不是这个假通道（假通道设进环境之前已经有人问过）：{caller:?}"
    );
    calls
}

/// 调用方给这个进程的显卡先后写成 `GPU_CARDS_BY_PRIORITY` 的值：`<HOSTS 里的地址>:<序号>:<额度>M` 空白隔开，一张卡都没点名是空串。
/// 调用方没交卡串而交了领卡通道的，在这一刻领（进程里只领一回）。用例不直接调它，交卡一律经 [`cards_of_the_caller_handed_to_the_run`]：
/// 那里只在调用方交了卡串、或卡串与通道都没交时调它，没交卡串而交了通道的那一支不经它、不领卡。
/// 留成 `pub`：变异表里「先领卡再交卡串」那一行在 `common_crash_points` 里调它。
///
/// # Panics
/// 调用方的多机配置读不了、显卡先后写坏了（连同地址不在 HOSTS 里、多机配置判法认不出这一台），或没经服务给卡而调用方的多机配置开了 GPU
/// （清没清场都一样；GPU 用例整条经 `task-server/task submit` 交，服务清场、分卡）；按需领卡没成。
pub fn card_priority_value_of_the_caller() -> String {
    let caller = GpuConfigurationOfTheCaller::of_this_process();
    // 多机配置是这台机器的，按这个进程的当前目录找（cargo 把测试进程的当前目录设成包目录；仓根不是键的输入）
    let caller_configuration = caller
        .multi_host_configuration_file(
            &std::env::current_dir().expect("取得到这个进程的当前目录（多机配置从这里起找）"),
        )
        .expect("调用方的多机配置在");
    caller
        .cards_by_priority(caller_configuration.as_deref())
        .expect("调用方的显卡先后写法对、开着 GPU 时经服务给了卡（清没清场都一样）")
        .map_or_else(String::new, |cards| {
            cards
                .cards()
                .iter()
                .map(|card| {
                    format!(
                        "{}:{}:{}M",
                        card.host.as_str(),
                        card.index,
                        card.memory_quota_mebibytes
                    )
                })
                .collect::<Vec<_>>()
                .join(" ")
        })
}

/// 把调用方给这个进程的卡交给这一趟，不领卡：调用方交了卡串的，设 `GPU_CARDS_BY_PRIORITY`（[`card_priority_value_of_the_caller`]，
/// 有卡串时不领）；没交卡串而交了领卡通道的，把通道的五个变量照调用方的值设回环境（用例的环境守卫清掉了通道目录与凭据），不设卡串，
/// 这一趟第一次真要在卡上判时自己领（只录的那一趟、录入段都不领）；两样都没有的照旧设卡串（照调用方的配置抄，开着 GPU 而没经服务给卡的在这里拒）。
/// 交回的守卫丢掉时还原；在用例的环境守卫里建、先于它丢，那一趟在它活着时起步。
///
/// # Panics
/// 同 [`card_priority_value_of_the_caller`]（交通道的那一支不领卡，不因领卡 panic）。
pub fn cards_of_the_caller_handed_to_the_run() -> EnvironmentRestoredOnDrop {
    let caller = GpuConfigurationOfTheCaller::of_this_process();
    match (
        caller.cards_handed_out_by_the_service(),
        caller.card_lease_channel(),
    ) {
        (None, Some(channel)) => {
            let names: Vec<&'static str> = channel
                .variables_for_the_client()
                .iter()
                .map(|(name, _)| *name)
                .collect();
            let mut handed_over = EnvironmentRestoredOnDrop::saved_then_cleared(&names);
            for (name, value) in channel.variables_for_the_client() {
                handed_over.set_variable(name, value);
            }
            handed_over
        }
        (Some(_), Some(_) | None) | (None, None) => {
            let mut handed_over =
                EnvironmentRestoredOnDrop::saved_then_cleared(&[CARDS_BY_PRIORITY_KEY]);
            handed_over.set_variable(
                CARDS_BY_PRIORITY_KEY,
                std::ffi::OsStr::new(&card_priority_value_of_the_caller()),
            );
            handed_over
        }
    }
}
