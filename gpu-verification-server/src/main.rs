//! GPU 批处理验证服务的进程本体：`gpu-verification-server serve --state-directory <状态目录>`，由用户级单元起
//! （模板 `singlefs-gpu-verification-server.service.in`；装、起、停、查经 `control`）。每台点名了卡的机器一个进程。
//!
//! 起来时：解编进来的配置（`service-configuration.txt`）；当前目录换到编这份二进制的那棵仓（多机配置从这里找）；
//! 算编进来的 GPU 判器组键与结论版本键（版本对账）；连判定库（连哪一处照验证包的取法，`crash_library::prune_server_route_of_the_configuration`）：
//! 一个容错客户端认领与写结论（追加日志 `<状态目录>/library-journal`，客户端身份就是这个进程的认领持有者），验证包那一侧另开一份
//! 全仓那一份库的 `VerdictStore`（追加日志 `<状态目录>/verdict-store-journal`）；两份追加日志起来时先删掉再新建，一个进程一份。
//! 照多机配置开这一台点名的全部卡（开不成不写「在跑」、退 78）；把「在跑」那一行写进状态目录下的 `serving`（同目录临时文件写好、
//! 同步之后换上，再回读核对），之后跑取活循环（`serving_from_the_library`：卡一直持着，判定库里有能取的待判块就认领、判、写回），直到被停
//! （单元 stop 发 SIGTERM，进程照缺省处置结束；在判的块没写回的，待判行留在库里，认领到租期过了失效，下一份服务再取）。
//! `control` 照 `serving` 里的进程号等于单元的主进程号判服务起来了：旧进程留下的那一行指着的不是此刻的主进程号。
//!
//! 退出码：2 参数认不出；1 配置解不开、版本键算不出、状态目录不在或写不进、回读对不上、换不到仓根；连判定库的路走不通（配置错）退 3；
//! 开不成卡（多机配置没点名这一台的卡、点名的卡有开不成的）退 78（运行条件不满足）；
//! 连不上、读写判定库失败照判定库客户端的退出码表（75 服务不可用、3 别的错）。被停时由信号结束。

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use gpu_verification_server::break_switches::BreakSwitches;
use gpu_verification_server::gpu_judges_on_a_card::OpenedGpuCards;
use gpu_verification_server::library_pending_pool::{LibraryPendingPool, VersionKeysOfThisService};
use gpu_verification_server::service_configuration::{
    ServiceConfiguration, SERVICE_CONFIGURATION_TEXT,
};
use gpu_verification_server::serving_from_the_library::{serve_from_the_library, ServingFailure};
use gpu_verification_server::verification_package_side::VerificationPackageOverTheOneLibrary;
use prune_server::reliable_client::{ReliableClient, ReliableClientSettings};
use singlefs_verification::crash_amplification::GpuPendingStreamMaterial;
use singlefs_verification::crash_library::prune_server_route_of_the_configuration;
use singlefs_verification::hexadecimal::hexadecimal_text;
use singlefs_verification::verdict_store::VerdictStoreError;

/// 状态目录下「在跑」那一行的文件名；`control` 的 `SERVING_FILE_NAME` 与它逐字相同。
const SERVING_FILE_NAME: &str = "serving";
/// 「在跑」那一行的开头；`control` 的 `SERVING_LINE_PREFIX` 与它逐字相同。
const SERVING_LINE_PREFIX: &str = "GPU_VERIFICATION_SERVER_SERVING";
/// 认领与写结论那一个判定库客户端的追加日志在状态目录里的名字。
const LIBRARY_JOURNAL_FILE_NAME: &str = "library-journal";
/// 验证包那一侧那份 `VerdictStore` 的判定库客户端的追加日志在状态目录里的名字。
const VERDICT_STORE_JOURNAL_FILE_NAME: &str = "verdict-store-journal";
const USAGE_EXIT: u8 = 2;
const STARTUP_FAILURE_EXIT: u8 = 1;
/// 连判定库的路走不通（配置错）：与验证包 `PruneServerRouteRefusal::exit_code` 同一个码。
const LIBRARY_ROUTE_REFUSED_EXIT: u8 = 3;
/// 开不成卡：运行条件不满足（项目里「开了 GPU 而卡用不上」同一个码）。
const CARDS_NOT_OPENED_EXIT: u8 = 78;

/// 编这份二进制的那棵仓的根（这个包在仓根下一层）。
fn repository_of_this_build() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().map_or_else(
        || PathBuf::from(env!("CARGO_MANIFEST_DIR")),
        Path::to_path_buf,
    )
}

const USAGE: &str = "gpu-verification-server serve --state-directory <状态目录>（由 python3 gpu-verification-server/control 装的用户级单元起）";
const STATE_DIRECTORY_HOWTO: &str = "python3 gpu-verification-server/control install 建状态目录（700）、写单元并起服务；装过的照 python3 gpu-verification-server/control check 的出路修";
const LIBRARY_HOWTO: &str =
    "prune-server/control check 看判定库服务在不在跑；服务起来之后 systemd 会把这个进程拉起来";

/// `serve --state-directory <目录>` 恰好这三个词时交回目录，别的写法交 `None`。
fn state_directory_of_the_serve_arguments(arguments: &[String]) -> Option<PathBuf> {
    match arguments {
        [subcommand, flag, directory] if subcommand == "serve" && flag == "--state-directory" => {
            Some(PathBuf::from(directory))
        }
        _ => None,
    }
}

/// 「在跑」那一行：这个进程的进程号、编进来的限时、从判定库取活的标记与认领的持有者。
fn serving_line(configuration: &ServiceConfiguration, holder: &str) -> String {
    format!(
        "{SERVING_LINE_PREFIX} pid={} oldest_pending_wait_limit_milliseconds={} intake=library holder={holder}",
        std::process::id(),
        configuration.oldest_pending_wait_limit.as_millis()
    )
}

/// 把 `line` 写成 `<状态目录>/serving`：同目录临时文件写好、同步，再换上，换上之后回读核对。
///
/// # Errors
/// 状态目录不在、临时文件写不进或换不上、回读出来与写的不同：交回哪一步、哪个路径。
fn write_the_serving_line(state_directory: &Path, line: &str) -> Result<(), String> {
    if !state_directory.is_dir() {
        return Err(format!("状态目录 {} 不在", state_directory.display()));
    }
    let serving = state_directory.join(SERVING_FILE_NAME);
    let partial = state_directory.join(format!(
        "{SERVING_FILE_NAME}.partial-{}",
        std::process::id()
    ));
    let mut writer = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&partial)
        .map_err(|error| format!("{} 写不进：{error}", partial.display()))?;
    writer
        .write_all(format!("{line}\n").as_bytes())
        .and_then(|()| writer.sync_all())
        .map_err(|error| format!("{} 写不进：{error}", partial.display()))?;
    std::fs::rename(&partial, &serving).map_err(|error| {
        format!(
            "{} 换不成 {}：{error}",
            partial.display(),
            serving.display()
        )
    })?;
    let read_back = std::fs::read_to_string(&serving)
        .map_err(|error| format!("{} 回读不了：{error}", serving.display()))?;
    if read_back.trim_end_matches('\n') != line {
        return Err(format!(
            "{} 回读出来是「{}」，写的是「{line}」",
            serving.display(),
            read_back.trim_end()
        ));
    }
    Ok(())
}

/// 删掉上一个进程留下的追加日志（不在不算错）。
fn remove_the_journal_of_the_previous_process(journal: &Path) -> Result<(), String> {
    match std::fs::remove_file(journal) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "删不掉上一个进程的追加日志 {}：{error}",
            journal.display()
        )),
    }
}

fn refuse(problem: &str, next_step: &str, exit: u8) -> ExitCode {
    eprintln!("  ✗ {problem}");
    eprintln!("  → 怎么办：{next_step}");
    ExitCode::from(exit)
}

fn refuse_for_the_cards(problem: &str) -> ExitCode {
    refuse(
        &format!("开不成这一台的卡：{problem}"),
        "照多机配置 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED（没写取 GPU_CARDS_BY_PRIORITY）点名这一台的卡、核点名的卡用得上（上面 GPU_CARD_* 那几行），再起服务",
        CARDS_NOT_OPENED_EXIT,
    )
}

fn library_exit(exit_code: i32) -> ExitCode {
    ExitCode::from(u8::try_from(exit_code).unwrap_or(STARTUP_FAILURE_EXIT))
}

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let Some(state_directory) = state_directory_of_the_serve_arguments(&arguments) else {
        return refuse(
            &format!("认不出参数「{}」", arguments.join(" ")),
            &format!("写成 {USAGE}"),
            USAGE_EXIT,
        );
    };
    let configuration = match ServiceConfiguration::from_the_text(SERVICE_CONFIGURATION_TEXT) {
        Ok(configuration) => configuration,
        Err(refusal) => {
            return refuse(
                &format!("编进来的 gpu-verification-server/service-configuration.txt 解不开：{refusal}"),
                "照那份文件头的写法改对，重编、重装（python3 gpu-verification-server/control install --binary <编好的二进制>）",
                STARTUP_FAILURE_EXIT,
            );
        }
    };
    let version_keys = match VersionKeysOfThisService::compiled_into_this_service() {
        Ok(version_keys) => version_keys,
        Err(problem) => {
            return refuse(
                &problem,
                "这一份二进制认不出自己判的是哪一版、不能对账：在仓根工作区里重编（判器键表由验证包的构建脚本写）、重装",
                STARTUP_FAILURE_EXIT,
            );
        }
    };
    if !state_directory.is_dir() {
        return refuse(
            &format!("状态目录 {} 不在", state_directory.display()),
            STATE_DIRECTORY_HOWTO,
            STARTUP_FAILURE_EXIT,
        );
    }
    // 换当前目录之前把相对路径的状态目录接成绝对路径
    let state_directory = match std::env::current_dir() {
        Ok(current_directory) => current_directory.join(&state_directory),
        Err(error) => {
            return refuse(
                &format!(
                    "取不到当前目录（状态目录 {} 要从它接成绝对路径）：{error}",
                    state_directory.display()
                ),
                "状态目录写绝对路径（control install 写进单元的就是绝对路径）",
                STARTUP_FAILURE_EXIT,
            );
        }
    };
    let repository = repository_of_this_build();
    if let Err(error) = std::env::set_current_dir(&repository) {
        return refuse(
            &format!("换不到编这份二进制的仓 {}：{error}", repository.display()),
            "在仓还在的那棵树里重编、重装（多机配置从仓里找）",
            STARTUP_FAILURE_EXIT,
        );
    }
    let route = match prune_server_route_of_the_configuration() {
        Ok(route) => route,
        Err(refusal) => {
            eprintln!("{}", refusal.failure_line());
            return ExitCode::from(LIBRARY_ROUTE_REFUSED_EXIT);
        }
    };
    let claims_journal = state_directory.join(LIBRARY_JOURNAL_FILE_NAME);
    let verdict_store_journal = state_directory.join(VERDICT_STORE_JOURNAL_FILE_NAME);
    for journal in [&claims_journal, &verdict_store_journal] {
        if let Err(problem) = remove_the_journal_of_the_previous_process(journal) {
            return refuse(&problem, STATE_DIRECTORY_HOWTO, STARTUP_FAILURE_EXIT);
        }
    }
    let claims =
        match ReliableClient::open(ReliableClientSettings::new(route.clone(), claims_journal)) {
            Ok(claims) => claims,
            Err(failure) => {
                eprintln!("  ✗ 连不上判定库：{failure}");
                eprintln!("  → 怎么办：{LIBRARY_HOWTO}");
                return library_exit(failure.exit_code());
            }
        };
    let side = match VerificationPackageOverTheOneLibrary::open(route, verdict_store_journal) {
        Ok(side) => side,
        Err(VerdictStoreError::PruneServer(failure)) => {
            eprintln!("  ✗ 连不上判定库：{}", failure.description);
            eprintln!("  → 怎么办：{LIBRARY_HOWTO}");
            return library_exit(failure.exit_code);
        }
        Err(other) => {
            return refuse(
                &format!("开不了全仓那一份判定库：{other:?}"),
                "库的布局不是这一版：照验证包 verdict_store 文件头的出路办（重建、换新库），不迁移",
                STARTUP_FAILURE_EXIT,
            );
        }
    };
    // 先开卡、再写「在跑」：control 见到那一行时卡已经持着
    let cards = match OpenedGpuCards::<GpuPendingStreamMaterial>::open_the_cards_named_for_this_host(
        &repository,
    ) {
        Ok(cards) => cards,
        Err(problem) => return refuse_for_the_cards(&problem),
    };
    let line = serving_line(&configuration, &hexadecimal_text(claims.client_identity()));
    if let Err(problem) = write_the_serving_line(&state_directory, &line) {
        return refuse(&problem, STATE_DIRECTORY_HOWTO, STARTUP_FAILURE_EXIT);
    }
    println!("{line}");
    let mut pool = LibraryPendingPool::new(
        side,
        claims,
        version_keys,
        configuration.claim_lease_milliseconds(),
        BreakSwitches::none(),
    );
    // 取活循环起来时开的就是上面开好的这一组；之后不再开（不带弄坏开关的服务一直持着它们）
    let mut opened_at_start = Some(cards);
    let mut open_the_cards = || match opened_at_start.take() {
        Some(cards_opened_at_start) => Ok(cards_opened_at_start),
        None => OpenedGpuCards::<GpuPendingStreamMaterial>::open_the_cards_named_for_this_host(
            &repository,
        ),
    };
    match serve_from_the_library(
        &configuration,
        &BreakSwitches::none(),
        &mut pool,
        &mut open_the_cards,
        &|| false,
    ) {
        Ok(summary) => {
            eprintln!("  ✗ 取活循环不该自己返回：{summary:?}");
            ExitCode::from(STARTUP_FAILURE_EXIT)
        }
        Err(ServingFailure::CardsNotOpened(problem)) => refuse_for_the_cards(&problem),
        Err(ServingFailure::Library(failure)) => {
            eprintln!("  ✗ 读写判定库失败：{failure}");
            eprintln!("  → 怎么办：prune-server/control check 看判定库服务；在判的块待判行还在库里，systemd 拉起这个进程之后再取");
            library_exit(failure.exit_code)
        }
    }
}
