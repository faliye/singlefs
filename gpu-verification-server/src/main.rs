//! GPU 批处理验证服务的进程本体：`gpu-verification-server serve --state-directory <状态目录>`，由用户级单元起
//! （模板 `singlefs-gpu-verification-server.service.in`；装、起、停、查经 `control`）。每台点名了卡的机器一个进程。
//! 同一个二进制另有借卡的子命令 `gpu-verification-server lease take | lease return <卡串> | lease renew`
//! （`control lease …` 转来；用法、退出码在 `card_lease` 文件头）：头一个词是 `lease` 的不换标准错误、不装信号处理，直接交给它。
//!
//! 起来头一件事（还没有别的线程）把标准错误换成过滤逐次派活那两种行的管道（`service_log` 文件头）；装 SIGTERM、SIGINT 的处理（只置标记，
//! `prune_server::interruption`）。之后：解编进来的配置（`service-configuration.txt`）；当前目录换到编这份二进制的那棵仓（多机配置从这里找）；
//! 读这个进程的身份（`service_process` 文件头）；连判定库（连哪一处照验证包的取法，
//! `crash_library::prune_server_route_of_the_configuration`）：一个容错客户端认领与写结论（追加日志 `<状态目录>/library-journal`，
//! 客户端身份就是这个进程的认领持有者），验证包那一侧另开一份真库的 `VerdictStore`（追加日志 `<状态目录>/verdict-store-journal`）；
//! 两份追加日志起来时先删掉再新建，一个进程一份，持有者每个进程新取。登记的临时库（`registered_libraries` 文件头）各开自己的两份客户端，
//! 追加日志放 `<状态目录>/temporary-libraries/<登记键的 sha256 头几个字节>/`，起来时整个目录删掉重建，丢掉那一份库时删它的子目录。读上一个进程写在 `serving` 里的那一行，上一个进程没了就以它的持有者
//! 放掉它留下的认领（`library_pending_pool` 文件头「放认领」：还活着、认不出的不放）。照多机配置开这一台点名的全部卡（开不成不写「在跑」、退 78）；
//! 把「在跑」那一行（进程号、进程起来的时刻、开机号、持有者）写进状态目录下的 `serving`（同目录临时文件写好、同步之后换上，再回读核对），
//! 之后跑取活循环（`serving_from_the_library`：卡一直持着，判定库里有能取的待判块就认领、判、写回；有人借卡时停认领、借出卡键，
//! 借卡键的这一台照多机配置判法认、在放上一个进程的认领之前交给待判池，上一个进程持着的卡键随它的认领一起放），直到收到停止信号
//! （单元 stop 发 SIGTERM）：取活循环收尾（入了池的块判完写回、放掉卡）、以自己的持有者放掉还持着的认领、等日志转完，退 0。
//! 收尾没走完就被 SIGKILL 的（单元的停止限时到了），下一个进程起来时照上面那一步放。
//! `control` 照 `serving` 里的进程号等于单元的主进程号判服务起来了：旧进程留下的那一行指着的不是此刻的主进程号。
//!
//! 退出码：0 收到停止信号、收尾完了；2 参数认不出；1 换不了标准错误、装不上信号处理、配置解不开、读不到这个进程的身份、
//! 状态目录不在或写不进、上一个进程的 `serving` 读不了、回读对不上、换不到仓根；连判定库的路走不通（配置错）退 3；
//! 开不成卡（多机配置判法认不出这一台、多机配置没点名这一台的卡、点名的卡有开不成的）退 78（运行条件不满足）；
//! 连不上、读写判定库失败（含起来时放上一个进程的认领、停的时候放自己的）照判定库客户端的退出码表（75 服务不可用、3 别的错）。

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use gpu_verification_server::break_switches::BreakSwitches;
use gpu_verification_server::card_lease::{
    run_the_lease_subcommand, CardLeaseKeys, LEASE_SUBCOMMAND,
};
use gpu_verification_server::gpu_judges_on_a_card::{MaterialsOfAServedStream, OpenedGpuCards};
use gpu_verification_server::library_pending_pool::LibraryPendingPool;
use gpu_verification_server::registered_libraries::{
    OpenedRegisteredLibrary, OpeningOfARegisteredLibrary, RegisteredLibrariesSettings,
    REGISTERED_LIBRARY_RECONNECT,
};
use gpu_verification_server::service_configuration::{
    ServiceConfiguration, SERVICE_CONFIGURATION_TEXT,
};
use gpu_verification_server::service_log::route_the_standard_error_through_the_filter;
use gpu_verification_server::service_process::{
    serving_line, PreviousServiceProcess, ProcessLiveness, ServiceProcessIdentity,
};
use gpu_verification_server::serving_from_the_library::{serve_from_the_libraries, ServingFailure};
use gpu_verification_server::verification_package_side::{
    key_space_of_the_prefix, VerificationPackageOverALibrary,
};
use prune_server::interruption::{install_termination_handlers, is_interrupted};
use prune_server::reliable_client::{ReliableClient, ReliableClientSettings};
use singlefs_verification::crash_library::prune_server_route_of_the_configuration;
use singlefs_verification::gpu_card_configuration::multi_host_addresses_of_the_caller;
use singlefs_verification::hexadecimal::hexadecimal_text;
use singlefs_verification::sha256::sha256_digest;
use singlefs_verification::verdict_store::{LibraryKeySpace, VerdictStoreError};

/// 状态目录下「在跑」那一行的文件名；`control` 的 `SERVING_FILE_NAME` 与它逐字相同。
const SERVING_FILE_NAME: &str = "serving";
/// 认领与写结论那一个判定库客户端的追加日志在状态目录里的名字。
const LIBRARY_JOURNAL_FILE_NAME: &str = "library-journal";
/// 验证包那一侧那份 `VerdictStore` 的判定库客户端的追加日志在状态目录里的名字。
const VERDICT_STORE_JOURNAL_FILE_NAME: &str = "verdict-store-journal";
/// 登记的临时库的追加日志所在的目录名（在状态目录下，一份库一个子目录）。
const TEMPORARY_LIBRARIES_DIRECTORY_NAME: &str = "temporary-libraries";
/// 子目录名取登记键的 sha256 的头几个字节（十六进制）。
const TEMPORARY_LIBRARY_DIRECTORY_NAME_BYTES: usize = 8;
/// 收到停止信号、收尾完了。
const STOPPED_EXIT: u8 = 0;
/// 收尾完了、退出之前那一行的开头（带取活循环交回的账）。
const STOPPED_LINE_PREFIX: &str = "GPU_VERIFICATION_SERVER_STOPPED";
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

/// 上一个进程写在 `<状态目录>/serving` 里的那一份（没有这份文件是 `None`）。
///
/// # Errors
/// 文件在而读不了：交回路径与原因。
fn serving_line_of_the_previous_process(state_directory: &Path) -> Result<Option<String>, String> {
    let serving = state_directory.join(SERVING_FILE_NAME);
    match std::fs::read_to_string(&serving) {
        Ok(text) => Ok(Some(text)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!(
            "上一个进程的「在跑」那一行 {} 读不了：{error}",
            serving.display()
        )),
    }
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

/// 删掉上一个进程留下的登记的临时库的追加日志目录（不在不算错）。
fn remove_the_temporary_libraries_of_the_previous_process(directory: &Path) -> Result<(), String> {
    match std::fs::remove_dir_all(directory) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "删不掉上一个进程留下的登记的临时库的追加日志目录 {}：{error}",
            directory.display()
        )),
    }
}

/// 照登记开一份登记的临时库：它的追加日志放 `<状态目录>/temporary-libraries/<登记键的 sha256 头几个字节>/`（新建）；认领与写结论的客户端连
/// `opening.route_of_the_claims`，验证包那一侧连 `opening.route_of_the_library`、取登记里的键空间；两个客户端重连只等几秒
/// （[`REGISTERED_LIBRARY_RECONNECT`]），登记的库没了不拖住真库的活。开不成交回为什么，目录随即删掉。
fn open_a_registered_library(
    opening: &OpeningOfARegisteredLibrary,
    state_directory: &Path,
    claim_lease_milliseconds: u32,
) -> Result<OpenedRegisteredLibrary<VerificationPackageOverALibrary>, String> {
    let directory = state_directory
        .join(TEMPORARY_LIBRARIES_DIRECTORY_NAME)
        .join(hexadecimal_text(
            &sha256_digest(&opening.claim_key)[..TEMPORARY_LIBRARY_DIRECTORY_NAME_BYTES],
        ));
    remove_the_temporary_libraries_of_the_previous_process(&directory)?;
    std::fs::create_dir_all(&directory)
        .map_err(|error| format!("建不了 {}：{error}", directory.display()))?;
    let opened = open_a_registered_library_in(opening, &directory, claim_lease_milliseconds);
    if opened.is_err() {
        // 开不成：目录里只有这一回开出来的几份追加日志，用不着留
        let _ = std::fs::remove_dir_all(&directory);
    }
    opened
}

fn open_a_registered_library_in(
    opening: &OpeningOfARegisteredLibrary,
    directory: &Path,
    claim_lease_milliseconds: u32,
) -> Result<OpenedRegisteredLibrary<VerificationPackageOverALibrary>, String> {
    let mut claims_settings = ReliableClientSettings::new(
        opening.route_of_the_claims.clone(),
        directory.join(LIBRARY_JOURNAL_FILE_NAME),
    );
    claims_settings.reconnect = REGISTERED_LIBRARY_RECONNECT;
    let claims = ReliableClient::open(claims_settings)
        .map_err(|failure| format!("连不上登记的库（认领与写结论）：{failure}"))?;
    let side = VerificationPackageOverALibrary::open(
        opening.route_of_the_library.clone(),
        directory.join(VERDICT_STORE_JOURNAL_FILE_NAME),
        key_space_of_the_prefix(&opening.registration.service.key_space_prefix),
    )
    .map_err(|error| format!("开不了登记的库：{error:?}"))?;
    let pool = LibraryPendingPool::new(
        side,
        claims,
        claim_lease_milliseconds,
        BreakSwitches::none(),
    );
    Ok(OpenedRegisteredLibrary {
        pool,
        journal_directory: Some(directory.to_path_buf()),
    })
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
    if let Some((subcommand, lease_arguments)) = arguments.split_first() {
        if subcommand == LEASE_SUBCOMMAND {
            return run_the_lease_subcommand(lease_arguments, &repository_of_this_build());
        }
    }
    // 头一步、还没有别的线程时换标准错误（service_log 文件头）；收尾时关掉它、等日志转完
    let routed_standard_error = match route_the_standard_error_through_the_filter() {
        Ok(routed) => routed,
        Err(problem) => {
            return refuse(
                &format!("换不了标准错误（逐次派活的行滤不掉）：{problem}"),
                "让起它的那一方给齐标准输入、输出、错误，/proc 挂着（用户级单元本来如此），再起",
                STARTUP_FAILURE_EXIT,
            );
        }
    };
    let exit = serve_until_stopped();
    routed_standard_error.finish();
    exit
}

fn serve_until_stopped() -> ExitCode {
    if let Err(problem) = install_termination_handlers() {
        return refuse(
            &format!("装不上停止信号的处理：{problem}"),
            "看这一台的内核与 libc 为什么拒 signal(2)；修好之后 systemd 会把这个进程拉起来",
            STARTUP_FAILURE_EXIT,
        );
    }
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
    let this_process = match ServiceProcessIdentity::of_this_process() {
        Ok(this_process) => this_process,
        Err(problem) => {
            return refuse(
                &format!("读不到这个进程的身份（写进「在跑」那一行、下一个进程照它判这一个还在不在）：{problem}"),
                "/proc 挂着再起",
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
    if let Ok(Some(line)) = serving_line_of_the_previous_process(&state_directory) {
        if let Ok(previous) = PreviousServiceProcess::of_the_serving_line(&line) {
            if previous.process.process_number != this_process.process_number
                && matches!(previous.process.liveness(), Ok(ProcessLiveness::Running))
            {
                return refuse(
                    &format!(
                        "状态目录 {} 里记的上一个服务进程 {} 此刻还在跑：不删它的日志、不起第二个服务进程",
                        state_directory.display(),
                        previous.process.process_number
                    ),
                    STATE_DIRECTORY_HOWTO,
                    STARTUP_FAILURE_EXIT,
                );
            }
        }
    }
    let claims_journal = state_directory.join(LIBRARY_JOURNAL_FILE_NAME);
    let verdict_store_journal = state_directory.join(VERDICT_STORE_JOURNAL_FILE_NAME);
    for journal in [&claims_journal, &verdict_store_journal] {
        if let Err(problem) = remove_the_journal_of_the_previous_process(journal) {
            return refuse(&problem, STATE_DIRECTORY_HOWTO, STARTUP_FAILURE_EXIT);
        }
    }
    if let Err(problem) = remove_the_temporary_libraries_of_the_previous_process(
        &state_directory.join(TEMPORARY_LIBRARIES_DIRECTORY_NAME),
    ) {
        return refuse(&problem, STATE_DIRECTORY_HOWTO, STARTUP_FAILURE_EXIT);
    }
    let real_route = route.clone();
    let claims =
        match ReliableClient::open(ReliableClientSettings::new(route.clone(), claims_journal)) {
            Ok(claims) => claims,
            Err(failure) => {
                eprintln!("  ✗ 连不上判定库：{failure}");
                eprintln!("  → 怎么办：{LIBRARY_HOWTO}");
                return library_exit(failure.exit_code());
            }
        };
    let side = match VerificationPackageOverALibrary::open(
        route,
        verdict_store_journal,
        LibraryKeySpace::TheOneLibrary,
    ) {
        Ok(side) => side,
        Err(VerdictStoreError::PruneServer(failure)) => {
            eprintln!("  ✗ 连不上判定库：{}", failure.description);
            eprintln!("  → 怎么办：{LIBRARY_HOWTO}");
            return library_exit(failure.exit_code);
        }
        Err(other) => {
            return refuse(
                &format!("开不了全仓那一份判定库：{other:?}"),
                "库里的判定向量表读不成：挪开 prune-server 的库目录让服务建一份新库，判定在新库里重判",
                STARTUP_FAILURE_EXIT,
            );
        }
    };
    let mut pool = LibraryPendingPool::new(
        side,
        claims,
        configuration.claim_lease_milliseconds(),
        BreakSwitches::none(),
    );
    // 借卡键的这一台：开卡认这一台用的是同一份判法（进程里只判一次）；放上一个进程的认领时它持着的卡键一起放
    let this_host = match multi_host_addresses_of_the_caller() {
        Ok(addresses) => {
            let host = addresses.this_host().as_str().to_string();
            pool.lend_the_cards_of_this_host(CardLeaseKeys::of_this_host(&host));
            host
        }
        Err(problem) => return refuse_for_the_cards(&problem),
    };
    // 写自己那一行之前读上一个进程那一行，它没了就放掉它留下的认领（不等租期）
    let previous_serving_line = match serving_line_of_the_previous_process(&state_directory) {
        Ok(previous_serving_line) => previous_serving_line,
        Err(problem) => return refuse(&problem, STATE_DIRECTORY_HOWTO, STARTUP_FAILURE_EXIT),
    };
    if let Err(failure) =
        pool.release_the_claims_left_by_the_previous_process(previous_serving_line.as_deref())
    {
        eprintln!("  ✗ 放上一个进程留下的认领时读写判定库失败：{failure}");
        eprintln!("  → 怎么办：{LIBRARY_HOWTO}");
        return library_exit(failure.exit_code);
    }
    // 先开卡、再写「在跑」：control 见到那一行时卡已经持着
    let cards = match OpenedGpuCards::<MaterialsOfAServedStream>::open_the_cards_named_for_this_host(
        &repository,
    ) {
        Ok(cards) => cards,
        Err(problem) => return refuse_for_the_cards(&problem),
    };
    let line = serving_line(
        &this_process,
        configuration.oldest_pending_wait_limit.as_millis(),
        &pool.holder(),
    );
    if let Err(problem) = write_the_serving_line(&state_directory, &line) {
        return refuse(&problem, STATE_DIRECTORY_HOWTO, STARTUP_FAILURE_EXIT);
    }
    println!("{line}");
    // 取活循环起来时开的就是上面开好的这一组；之后不再开（不带弄坏开关的服务一直持着它们）
    let mut opened_at_start = Some(cards);
    let mut open_the_cards = || match opened_at_start.take() {
        Some(cards_opened_at_start) => Ok(cards_opened_at_start),
        None => OpenedGpuCards::<MaterialsOfAServedStream>::open_the_cards_named_for_this_host(
            &repository,
        ),
    };
    let claim_lease_milliseconds = configuration.claim_lease_milliseconds();
    let mut open_a_library = |opening: &OpeningOfARegisteredLibrary| {
        open_a_registered_library(opening, &state_directory, claim_lease_milliseconds)
    };
    let registered_libraries = RegisteredLibrariesSettings {
        open_a_library: &mut open_a_library,
        this_host,
        real_route,
        previous_serving_line,
    };
    match serve_from_the_libraries(
        &configuration,
        &BreakSwitches::none(),
        &mut pool,
        Some(registered_libraries),
        &mut open_the_cards,
        &is_interrupted,
    ) {
        Ok(summary) => match pool.release_the_claims_of_this_process() {
            Ok(_) => {
                eprintln!("{STOPPED_LINE_PREFIX} {summary:?}");
                ExitCode::from(STOPPED_EXIT)
            }
            Err(failure) => {
                eprintln!("  ✗ 停的时候放自己的认领读写判定库失败：{failure}");
                eprintln!("  → 怎么办：prune-server/control check 看判定库服务；没放掉的认领由下一个进程起来时放，或到租期过了失效");
                library_exit(failure.exit_code)
            }
        },
        Err(ServingFailure::CardsNotOpened(problem)) => refuse_for_the_cards(&problem),
        Err(ServingFailure::Library(failure)) => {
            eprintln!("  ✗ 读写判定库失败：{failure}");
            eprintln!("  → 怎么办：prune-server/control check 看判定库服务；在判的块待判行还在库里，systemd 拉起这个进程之后再取");
            library_exit(failure.exit_code)
        }
    }
}
