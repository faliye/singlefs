//! 服务进程本体（`src/main.rs`）：`serve` 连上判定库之后照多机配置开这一台点名的卡，多机配置一张卡都没点名就退 78（运行条件不满足）、
//! 不写 `serving`；参数认不出退 2、状态目录不在退 1，两种都不写 `serving`。不上卡（开了卡、写「在跑」、一直持着那一段要卡，
//! 取活循环那一半由上卡的 `library_service_path_agrees_with_the_direct_judge` 核）；每条用例自己建、自己删用例目录。
//! 连的判定库是这个测试进程里起的临时服务（不写真服务）：经改连接地址的变量交给服务进程，带临时服务的标记（验证包
//! `crash_library::connection_override_problem_of` 只在标记底下认改连接地址）；多机配置是用例目录里自己写的一份（`SINGLEFS_MULTI_HOST_CONFIG`）。
//! 起的服务进程放进它自己的进程组；它不起子进程，用例结束（含 panic）时由守卫停掉、收尸。

mod common;

use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use common::TemporaryLibraryService;
use singlefs_verification::crash_library::{
    PRUNE_SERVER_SOCKET_ENVIRONMENT_VARIABLE,
    PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET_ENVIRONMENT_VARIABLE,
};
use singlefs_verification::gpu_card_configuration::MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE;

const SERVER_BINARY: &str = env!("CARGO_BIN_EXE_gpu-verification-server");
/// `src/main.rs` 的 `SERVING_FILE_NAME`（`control` 认的也是它）。
const SERVING_FILE_NAME: &str = "serving";
/// 等服务退出的上限：起一个只解配置、连临时判定库、读多机配置的进程，60 秒是宽的
const EXIT_WAIT: Duration = Duration::from_secs(60);
const EXIT_POLL: Duration = Duration::from_millis(50);
const USAGE_EXIT: i32 = 2;
const STARTUP_FAILURE_EXIT: i32 = 1;
/// `src/main.rs` 的 `CARDS_NOT_OPENED_EXIT`。
const CARDS_NOT_OPENED_EXIT: i32 = 78;
/// 用例自己写的多机配置：开着 GPU，两个显卡先后的键都没写（这一台一张卡都没点名）。
const MULTI_HOST_CONFIGURATION_WITHOUT_CARDS: &str = "ENABLE_GPU=1\n";

/// 用例自己的目录，落在 `TMPDIR` 下：里面一个状态目录 `state` 与一份多机配置，用例结束时删掉。
struct TemporaryCaseDirectory {
    path: PathBuf,
}

impl TemporaryCaseDirectory {
    fn created_for(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "gpu-verification-server-{label}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir(&path)
            .unwrap_or_else(|error| panic!("建不了用例目录 {}：{error}", path.display()));
        std::fs::create_dir(path.join("state"))
            .unwrap_or_else(|error| panic!("建不了状态目录 {}：{error}", path.display()));
        std::fs::write(
            path.join("multi-host.env"),
            MULTI_HOST_CONFIGURATION_WITHOUT_CARDS,
        )
        .unwrap_or_else(|error| panic!("写不了多机配置 {}：{error}", path.display()));
        Self { path }
    }

    fn state_directory(&self) -> PathBuf {
        self.path.join("state")
    }

    fn multi_host_configuration(&self) -> PathBuf {
        self.path.join("multi-host.env")
    }
}

impl Drop for TemporaryCaseDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

/// 起来的服务进程：用例结束（含 panic）时停掉、收尸。
struct ServerProcessGuard {
    child: Child,
}

impl Drop for ServerProcessGuard {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn server_command(
    arguments: &[&str],
    state_directory: &Path,
    case: &TemporaryCaseDirectory,
    library: &TemporaryLibraryService,
) -> Command {
    let mut command = Command::new(SERVER_BINARY);
    command
        .args(arguments)
        .arg(state_directory)
        .env(
            PRUNE_SERVER_SOCKET_ENVIRONMENT_VARIABLE,
            library.socket_path(),
        )
        .env(
            PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET_ENVIRONMENT_VARIABLE,
            library.socket_path(),
        )
        .env(
            MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
            case.multi_host_configuration(),
        )
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0);
    command
}

/// 起服务、等它退出（或到上限，到了就 panic，守卫停掉它）。
fn exit_status_within_the_wait(command: &mut Command) -> ExitStatus {
    let mut server = ServerProcessGuard {
        child: command.spawn().expect("起得了服务进程"),
    };
    let deadline = Instant::now() + EXIT_WAIT;
    while Instant::now() < deadline {
        if let Some(status) = server.child.try_wait().expect("问得到服务进程的状态") {
            return status;
        }
        std::thread::sleep(EXIT_POLL);
    }
    panic!("{} 秒里服务进程没退出", EXIT_WAIT.as_secs());
}

/// 判别力：`src/main.rs` 不开卡就写「在跑」、进取活循环，这一条红（进程不退、写了 serving）。
#[test]
fn serve_refuses_to_run_without_cards_named_for_this_host_and_writes_nothing() {
    let case = TemporaryCaseDirectory::created_for("no-cards");
    let library = TemporaryLibraryService::start("serve-no-cards");
    let state = case.state_directory();
    let status = exit_status_within_the_wait(&mut server_command(
        &["serve", "--state-directory"],
        &state,
        &case,
        &library,
    ));
    assert_eq!(
        status.code(),
        Some(CARDS_NOT_OPENED_EXIT),
        "连上了判定库、多机配置一张卡都没点名：运行条件不满足退 78"
    );
    assert!(
        !state.join(SERVING_FILE_NAME).exists(),
        "开不成卡时不写「在跑」"
    );
}

#[test]
fn serve_refuses_arguments_it_does_not_know_with_the_usage_exit_and_writes_nothing() {
    let case = TemporaryCaseDirectory::created_for("usage");
    let library = TemporaryLibraryService::start("serve-usage");
    let state = case.state_directory();
    let status = exit_status_within_the_wait(&mut server_command(
        &["serve", "--state-dir"],
        &state,
        &case,
        &library,
    ));
    assert_eq!(status.code(), Some(USAGE_EXIT), "认不出的参数退 2");
    assert!(
        !state.join(SERVING_FILE_NAME).exists(),
        "参数认不出时不写「在跑」"
    );
}

#[test]
fn serve_refuses_a_state_directory_that_is_not_there() {
    let case = TemporaryCaseDirectory::created_for("missing");
    let missing = case.state_directory().join("not-created");
    let library = TemporaryLibraryService::start("serve-missing-state");
    let status = exit_status_within_the_wait(&mut server_command(
        &["serve", "--state-directory"],
        &missing,
        &case,
        &library,
    ));
    assert_eq!(
        status.code(),
        Some(STARTUP_FAILURE_EXIT),
        "状态目录不在退 1"
    );
    assert!(!missing.exists(), "状态目录不在时不替装的一方建它");
}
