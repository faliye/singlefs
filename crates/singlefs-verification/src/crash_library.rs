//! 崩溃判定库：全仓只有一份，就是 prune-server 自己的那份 RocksDB（`.claude/rules/verification.md`「判定库只有一份：全仓统一写进 prune-server 的那份 RocksDB」）；
//! 按键复用、能剪枝；录入段与判段只经库交接。
//!
//! 规矩：① 测试先把 CPU 跑出来的数据写进库，再从库里读出来判；② 每一类测试都做剪枝，按块键、结论种类、判法单元指纹
//! 存结论，判过的复用；③ CPU 产出的数据一律进库，判段按批从库里取出来交给 GPU，没有 GPU 才交给 CPU；④ 两段之间不在内存里交数据，
//! CPU 自己判也分两段。一份库、类当索引；键只由内容定，不带路径、名字、种子、手写版本号。
//!
//! 只走 prune-server（「现在就对接 不允许残留」「我们不兼容 类似老开关 这种东西」）：崩溃放量、崩溃枚举用例、给定状态表、
//! 两种对拍几类都经 prune-server 的客户端直接读写服务那一份（[`crate::verdict_store::PruneServerStorage`]），本机不开任何一份库、不设开关、
//! 不收选库的变量；写下的块别的进程、另一台马上读得到。测试的类别（[`CrashLibraryClass`]）只进行里报的名字，结论种类（GPU、CPU、观察者、对拍）
//! 是块记录里的字段。连哪一处（[`prune_server_route_of_the_configuration`]）：`XDG_RUNTIME_DIR` 下那一个 socket 在就走 unix socket；
//! 不在走多机配置 `multi-host.env` 的 `PRUNE_SERVER_TCP_ADDRESS`（与 `scripts/prune-server/prune_server_client.py` 的 `choose_route` 同一个取法）。
//! 改连接地址的 `SINGLEFS_PRUNE_SERVER_SOCKET`、`SINGLEFS_PRUNE_SERVER_TCP_ADDRESS` 只在临时服务底下认，平常跑设了就拒
//! （[`connection_override_problem_of`]，与 `prune_server_client.py` 的 `connection_override_problem` 同一个判法），拒的时候退 3（用法与环境错，
//! 与 `prune_server_client.py` 的 `connection-override` 同一个码，[`PruneServerRouteRefusal::exit_code`]）；socket 不在、TCP 地址两处都没有
//! （多机配置读不了也算）同样退 3（配置错，不是服务不通：与 `prune_server_client.py` 的 `configuration` 同一个码）。连不上、读写不成，
//! 打一行 [`PRUNE_SERVER_FAILURE_LINE_PREFIX`]、照 prune-server 客户端的退出码退出（75 服务不可用），读失败不当成「库里没有」去现判。
//!
//! 测试只对临时服务（[`DisposableLibrary`]）：每份一次性库在这个进程里自己起一份临时 prune-server（`prune_server::server::start_in_this_process`，
//! 状态目录、库与 socket 都在它独占的一次性临时目录里；socket 放进去超上限时另放 `/tmp` 下一份短的一次性目录，[`TemporaryPruneServer`]），连它、不连真服务、不读连哪一处的环境变量（`PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET`
//! 只留给脚本自证的包装 `scripts/prune-server/prune_server_client_selftest.py --with-temporary-service`）；丢掉时停服、等库关上、删目录。
//! 临时服务的 TCP 绑多机配置 `HOSTS` 里这一台的地址（[`network_address_of_a_temporary_service_on`]）。判段配了 GPU 时只往真服务登记
//! 这份临时服务（一把续着租的认领，[`register_the_temporary_library_with_the_real_service`]），数据行一行不写；GPU 批处理服务照登记连过来。
//! 并行跑的用例各起各的，互不相见；一份一次性库里是那份临时服务里的一段名字空间，从空库起。
//! 经产品入口跑的用例把一次性库设成这个进程开的那一份（[`DisposableLibrary::make_it_the_library_of_this_process`]，进程内的设置，不是环境变量，
//! 也不传给子进程；一次性库连的是它自己起的那份临时服务，所以真任务里没有写真服务的这一条路）。
//!
//! 两段怎么接（[`two_stages`]）：录入段把这一项的几样各按摘要写进库、再按录入键（录下的写流摘要与场景标识，[`recording_key_of_the_scenario_and_the_inputs`]）
//! 登一份清单（[`ItemManifest`]）；录入键下已有清单就不再录（命中就跳过）。判段照同一份输入现算录入键、读清单，再按清单里的摘要读回那几样。
//! 选段照 [`RUN_STAGE_ENVIRONMENT_VARIABLE`]：`record` 只录、`judge` 只判、不设两段先后跑（中间照样经库）。
//!
//! 同一个进程里几处开库共用一份连接（[`LibraryOpenInThisProcess`]：按键空间各一份，库里一把锁一次一个读写）；不同进程各连各的，谁都不等谁。
//! 这把锁只包与 prune-server 的往返：排计划、序列化、算摘要、打包、解码都在锁外（[`assert_this_thread_holds_no_library_lock`] 在这几步的路上判），
//! 累计持锁时长按连接打一行（[`LIBRARY_LOCK_TIME_LINE_PREFIX`]）、按取锁的调用点各打一行（[`LIBRARY_LOCK_TIME_OF_A_SITE_LINE_PREFIX`]）；
//! 判段与录入段经 [`LibraryTakenAtEachReadOrWrite`] 每次读写现取现放，判、看、排计划时手里不握锁。同一条连接上同一摘要的流只读回一次（[`ValuesReadBackOnceInThisProcess`]）：
//! 留的是从库里读回、核过摘要的那一份，不落盘、不存结论，随连接放手。

use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::{
    Arc, Mutex, MutexGuard, OnceLock, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard, Weak,
};

use crate::hexadecimal::hexadecimal_text;
use crate::sha256::{sha256_digest, DIGEST_BYTES};

use crate::crash_facts::{FlowFacts, FlowFactsReadBack};
use crate::crash_identity::{
    judged_product_code_of_this_binary, JudgedProductCodeOfThisBinary, JudgingUnitsKey,
};
#[cfg(feature = "gpu")]
use crate::crash_judge_dispatch::{PackedJudgeTables, PackedJudgeTablesReadBack};
use crate::crash_point_replay_progress::CrashPointReplayToolchainIdentity;
pub use crate::verdict_store::{
    JudgementCircumstances, PairingAgreement, RecordingKey, RunKindOfTheJudgement,
    CRASH_LIBRARY_RUN_KIND_ENVIRONMENT_VARIABLE,
};
use crate::verdict_store::{
    LibraryKeySpace, PairingConclusionKey, PruneServerFailureHandling, PruneServerStorage,
    StoredPairingConclusion, VerdictStore, DISPOSABLE_LIBRARY_KEY_SPACE_LEADING_BYTE,
    PRUNE_SERVER_FAILURE_LINE_PREFIX,
};
use prune_server::client::RouteAddress;
use prune_server::configuration::{
    default_multi_host_configuration_path, default_socket_path, read_configuration_value,
    socket_path_problem, NETWORK_ADDRESS_KEY,
};
use prune_server::protocol::{ClaimOutcome, ListedClaim, Table};
use prune_server::reliable_client::{
    ClientFailure, ReliableClient, ReliableClientSettings, DEFAULT_JOURNAL_FILE_NAME,
    OTHER_CLIENT_FAILURE_EXIT_CODE,
};
use prune_server::server::{start_in_this_process, ServiceInThisProcess};

mod disposable_library;
mod exclusive_temporary_directory;
mod facts_and_judge_tables_through_the_library;
mod inner_threads_of_the_test_process;
mod item_manifests_and_two_stages;
mod library_open_in_this_process;
mod pairing_blocks;
mod pairing_inputs_and_conclusions;
mod temporary_library_registration;

pub use disposable_library::*;
pub use exclusive_temporary_directory::*;
pub use facts_and_judge_tables_through_the_library::*;
pub(crate) use inner_threads_of_the_test_process::*;
pub use item_manifests_and_two_stages::*;
pub use library_open_in_this_process::*;
pub use pairing_blocks::*;
pub use pairing_inputs_and_conclusions::*;
pub use temporary_library_registration::*;

#[cfg(test)]
use tests::addresses_whose_host_is_not_the_usual_loopback;

/// 这一趟跑哪一段。
pub const RUN_STAGE_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_CRASH_AMPLIFICATION_STAGE";

/// 改连哪一处的 socket（只在临时服务底下认，[`connection_override_problem_of`]；平常跑取 `XDG_RUNTIME_DIR` 下那一个）；
/// 与 `scripts/prune-server/prune_server_client.py` 的 `SOCKET_VARIABLE` 同名。
pub const PRUNE_SERVER_SOCKET_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_PRUNE_SERVER_SOCKET";

/// 改 socket 不在时连的 TCP 地址（只在临时服务底下认，[`connection_override_problem_of`]；平常跑取多机配置的 `PRUNE_SERVER_TCP_ADDRESS`）；
/// 与 `prune_server_client.py` 的 `TCP_ADDRESS_VARIABLE` 同名。
pub const PRUNE_SERVER_TCP_ADDRESS_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_PRUNE_SERVER_TCP_ADDRESS";

/// `prune_server_client_selftest.py --with-temporary-service` 起的临时服务的 socket；与 `prune_server_client.py` 的 `TEMPORARY_SERVICE_VARIABLE` 同名。
pub const PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET_ENVIRONMENT_VARIABLE: &str =
    "PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET";

/// 不在临时服务底下设了改连接地址的变量时写给人的出路：Rust 测试不设它们（一次性库自己在进程里起临时服务，[`DisposableLibrary::new`]）；
/// 脚本自证照 `prune_server_client.py` 的 `SELFTEST_SERVICE_REMEDY` 那一条路起临时服务。
pub const TEMPORARY_SERVICE_REMEDY: &str = "Rust 测试不设这两个变量：一次性库（crash_library::DisposableLibrary）自己在进程里起临时 prune-server；\
脚本自证照 scripts/prune-server/prune_server_client.py 文件头「命令行」--with-temporary-service 那一条，在临时服务底下起：\
python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service --server-binary <prune-server 二进制> -- <原命令>（崩溃判定库的测试只对临时服务，不写真服务）";

/// 一次性库的名字空间在头一个字节（[`DISPOSABLE_LIBRARY_KEY_SPACE_LEADING_BYTE`]）之后接的名字头：后面接标签、进程号、时刻与序号。
const DISPOSABLE_LIBRARY_NAMESPACE_NAME_PREFIX: &str = "disposable-crash-library/";

/// 这个进程的工具链：问一次 cargo 与 rustc，之后照这一份（判法单元的键与录入键都不拼它，取键的入口照一种签名收它）。
///
/// # Panics
/// 同 [`CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test`]。
#[must_use]
pub fn toolchain_of_this_process() -> &'static CrashPointReplayToolchainIdentity {
    static TOOLCHAIN: OnceLock<CrashPointReplayToolchainIdentity> = OnceLock::new();
    TOOLCHAIN.get_or_init(CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test)
}

// ============================== 一份库，测试的类别只当索引 ==============================

/// 哪一类测试开的库：几类开的是同一份库（服务里那一份），类别只进行里报的名字。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CrashLibraryClass {
    /// 崩溃放量流水线（`crash_amplification::run_from_environment`）。
    CrashAmplification,
    /// 崩溃点重放枚举用例（`crash_state_enumeration`）。
    CrashStateEnumeration,
    /// 随机历史两项的给定状态表（`crash_amplification::GivenStatesEngine`）。
    GivenStates,
    /// GPU 判器与 CPU 判器的对拍用例：判表先入库、再读库上卡。
    JudgeTablesPairing,
    /// 第 ② 段核对的 GPU 与 CPU 参照对拍用例：事实表先入库、再读库核。
    FactsPairing,
    /// 输入是调用方给的字节的对拍用例（判的不是崩溃状态）：输入先入库、再读库交给两边。
    InputPairing,
}

impl CrashLibraryClass {
    /// 行里报的名字。
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::CrashAmplification => "crash_amplification",
            Self::CrashStateEnumeration => "crash_state_enumeration",
            Self::GivenStates => "given_states",
            Self::JudgeTablesPairing => "judge_tables_pairing",
            Self::FactsPairing => "facts_pairing",
            Self::InputPairing => "input_pairing",
        }
    }
}

// ============================== 连哪一处、临时服务、一次性库 ==============================

/// 改连哪一处的两个变量（`socket` 是 [`PRUNE_SERVER_SOCKET_ENVIRONMENT_VARIABLE`] 的值、`network_address` 是
/// [`PRUNE_SERVER_TCP_ADDRESS_ENVIRONMENT_VARIABLE`] 的值，空的是没设）设了，而不在临时服务底下（临时服务的标记 `marked`，即
/// [`PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET_ENVIRONMENT_VARIABLE`] 的值，是空的）：交一句为什么；别的交 `None`。
/// 判法与 `scripts/prune-server/prune_server_client.py` 的 `connection_override_problem` 相同：只看标记设没设，不核它与 socket 相同
/// （临时服务底下的用例要造「连不上」时把这两个变量改指到不在的 socket 与放掉的端口，标记照旧指着临时服务）。
/// 平常跑改连接地址等于另连一份判定库（判定库只有一份），所以不认。
#[must_use]
pub fn connection_override_problem_of(
    marked: &std::ffi::OsStr,
    socket: &std::ffi::OsStr,
    network_address: &std::ffi::OsStr,
) -> Option<String> {
    if !marked.is_empty() {
        return None;
    }
    let overriding: Vec<String> = [
        (PRUNE_SERVER_SOCKET_ENVIRONMENT_VARIABLE, socket),
        (
            PRUNE_SERVER_TCP_ADDRESS_ENVIRONMENT_VARIABLE,
            network_address,
        ),
    ]
    .iter()
    .filter(|(_name, value)| !value.is_empty())
    .map(|(name, value)| format!("{name}={}", value.to_string_lossy()))
    .collect();
    if overriding.is_empty() {
        return None;
    }
    Some(format!(
        "{} 只在临时服务底下认（{PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET_ENVIRONMENT_VARIABLE} 没设）：平常跑改连接地址等于另连一份判定库，一条都没连。\
         出路：平常跑 unset {PRUNE_SERVER_SOCKET_ENVIRONMENT_VARIABLE} 与 {PRUNE_SERVER_TCP_ADDRESS_ENVIRONMENT_VARIABLE}（本机连 socket，第二台连多机配置的 \
         {NETWORK_ADDRESS_KEY}）；测试与自证{TEMPORARY_SERVICE_REMEDY}",
        overriding.join("、")
    ))
}

/// 连哪一处那一步为什么拒（两种都是用法与环境错、不是服务不通，退同一个码 [`Self::exit_code`]，与 `prune_server_client.py` 的
/// `PruneServerFailure` 那张码表对齐；分两种是因为出路不同，[`Self::failure_line`] 各写各的）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PruneServerRouteRefusal {
    /// 不在临时服务底下设了改连接地址的变量（[`connection_override_problem_of`] 交的那一句），同 `prune_server_client.py` 的 `connection-override`。
    ConnectionOverrideOutsideTheTemporaryService(String),
    /// socket 不在、TCP 地址两处都没有，或多机配置读不了：哪一处都没写服务的地址，同 `prune_server_client.py` 的 `configuration`。
    NoPruneServerAddressConfigured(String),
}

impl PruneServerRouteRefusal {
    /// 拒了之后进程退的码：两种都退 [`OTHER_CLIENT_FAILURE_EXIT_CODE`]（3）。改连接地址被拒同 `prune_server_client.py` 的 `connection-override`，
    /// 哪一处都没写服务的地址同它的 `configuration`：配置与环境错，重交照样拒，不是「服务不通、可重试」的
    /// `prune_server::reliable_client::PRUNE_SERVER_UNAVAILABLE_EXIT_CODE`（75，连不上、服务停了才退它）。
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::ConnectionOverrideOutsideTheTemporaryService(_problem) => {
                OTHER_CLIENT_FAILURE_EXIT_CODE
            }
            Self::NoPruneServerAddressConfigured(_problem) => OTHER_CLIENT_FAILURE_EXIT_CODE,
        }
    }

    /// 拒了之后、退出之前在标准错误打的那一行：`<前缀> exit_code=<码> <说明>`（[`PRUNE_SERVER_FAILURE_LINE_PREFIX`]）。
    #[must_use]
    pub fn failure_line(&self) -> String {
        let exit_code = self.exit_code();
        match self {
            Self::ConnectionOverrideOutsideTheTemporaryService(problem) => format!(
                "{PRUNE_SERVER_FAILURE_LINE_PREFIX} exit_code={exit_code} 改连接地址的变量不认：{problem}"
            ),
            Self::NoPruneServerAddressConfigured(problem) => format!(
                "{PRUNE_SERVER_FAILURE_LINE_PREFIX} exit_code={exit_code} 没有写 prune-server 的地址：{problem}。\
                 出路：本机要有服务的 socket（prune-server/control check 看服务在不在跑），第二台要有多机配置里的 {NETWORK_ADDRESS_KEY}；\
                 判定库没有不走服务的那条路"
            ),
        }
    }
}

/// 连哪一处：读这个进程环境里的三个变量，连同多机配置的位置（`default_multi_host_configuration_path`），交 [`prune_server_route_of_the_variables`] 判。
///
/// # Errors
/// 同 [`prune_server_route_of_the_variables`]。
pub fn prune_server_route_of_the_configuration() -> Result<RouteAddress, PruneServerRouteRefusal> {
    let marked = std::env::var_os(PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET_ENVIRONMENT_VARIABLE)
        .unwrap_or_default();
    let socket = std::env::var_os(PRUNE_SERVER_SOCKET_ENVIRONMENT_VARIABLE).unwrap_or_default();
    let network_address =
        std::env::var_os(PRUNE_SERVER_TCP_ADDRESS_ENVIRONMENT_VARIABLE).unwrap_or_default();
    prune_server_route_of_the_variables(
        &marked,
        &socket,
        &network_address,
        &default_multi_host_configuration_path(),
    )
}

/// 连哪一处（三个变量的值由调用方给，空的是没设）：改连接地址的两个变量设了而不在临时服务底下就拒（[`connection_override_problem_of`]）；
/// 否则 socket（`socket` 设了用它，没设取 `XDG_RUNTIME_DIR` 下那一个）在就走 unix socket；不在走 TCP，地址 `network_address` 设了用它，
/// 没设取多机配置 `multi_host_configuration_path` 的 `PRUNE_SERVER_TCP_ADDRESS`（平常跑由 [`prune_server_route_of_the_configuration`] 照 prune-server
/// 的取法给：`SINGLEFS_MULTI_HOST_CONFIG`，没设取仓根那一份）。没写就是没有，不补缺省。
///
/// # Errors
/// 不在临时服务底下设了改连接地址的变量（[`PruneServerRouteRefusal::ConnectionOverrideOutsideTheTemporaryService`]）；socket 不在、
/// TCP 地址两处都没有，配置读不了（[`PruneServerRouteRefusal::NoPruneServerAddressConfigured`]）。
pub fn prune_server_route_of_the_variables(
    marked: &std::ffi::OsStr,
    socket: &std::ffi::OsStr,
    network_address: &std::ffi::OsStr,
    multi_host_configuration_path: &Path,
) -> Result<RouteAddress, PruneServerRouteRefusal> {
    if let Some(problem) = connection_override_problem_of(marked, socket, network_address) {
        return Err(PruneServerRouteRefusal::ConnectionOverrideOutsideTheTemporaryService(problem));
    }
    let socket_path = if socket.is_empty() {
        default_socket_path().ok()
    } else {
        Some(PathBuf::from(socket))
    };
    if let Some(socket_path) = socket_path.as_ref().filter(|path| path.exists()) {
        return Ok(RouteAddress::UnixSocket(socket_path.clone()));
    }
    if !network_address.is_empty() {
        return Ok(RouteAddress::Network(
            network_address.to_string_lossy().into_owned(),
        ));
    }
    let configuration_path = multi_host_configuration_path;
    match read_configuration_value(configuration_path, NETWORK_ADDRESS_KEY)
        .map_err(PruneServerRouteRefusal::NoPruneServerAddressConfigured)?
    {
        Some(address) if !address.is_empty() => Ok(RouteAddress::Network(address)),
        Some(_) | None => Err(PruneServerRouteRefusal::NoPruneServerAddressConfigured(format!(
            "没有 prune-server 的 socket（{}），{PRUNE_SERVER_TCP_ADDRESS_ENVIRONMENT_VARIABLE} 没设，多机配置 {} 里也没有 {NETWORK_ADDRESS_KEY}",
            socket_path.map_or_else(
                || "XDG_RUNTIME_DIR 没设".to_string(),
                |path| path.display().to_string()
            ),
            configuration_path.display()
        ))),
    }
}

/// 只开 CPU、照旧连调用方那一处 prune-server 的多机配置全文：经产品入口在 CPU 判器上判、块进全仓那一份库的用例把它写进自己的临时配置，
/// 环境守卫里把 `SINGLEFS_MULTI_HOST_CONFIG` 指过去（判在 CPU 上，块照块键与判器组的单元键在全仓那一份里复用，测试函数键变了也不整趟重判）。
/// 全文只有两样：`ENABLE_GPU=0` 一行；调用方那一份（`caller_configuration_file`）写了非空的 `PRUNE_SERVER_TCP_ADDRESS` 就照抄那一行
/// （没有 prune-server socket 的那一台照它连服务，[`prune_server_route_of_the_variables`]）。别的键一个不抄：不跨机、不分片、不上卡（没写就是没有）。
///
/// # Errors
/// 调用方那一份读不了。
pub fn multi_host_configuration_on_the_cpu_keeping_the_prune_server_of(
    caller_configuration_file: &Path,
) -> Result<String, String> {
    let mut text = String::from("ENABLE_GPU=0\n");
    if let Some(address) = read_configuration_value(caller_configuration_file, NETWORK_ADDRESS_KEY)?
        .filter(|address| !address.is_empty())
    {
        text.push_str(&format!("{NETWORK_ADDRESS_KEY}={address}\n"));
    }
    Ok(text)
}

/// 设成这个进程开的那一份的一次性库（[`DisposableLibrary::make_it_the_library_of_this_process`]）：连哪一处、哪一个键空间。
#[derive(Clone)]
struct DisposableLibraryChosen {
    route: RouteAddress,
    key_space: LibraryKeySpace,
}

/// 这个进程此刻开的是哪一份一次性库；`None` 是全仓那一份。
static DISPOSABLE_LIBRARY_OF_THIS_PROCESS: Mutex<Option<DisposableLibraryChosen>> =
    Mutex::new(None);

/// 设一次性库的那几条用例一个接一个（写的一侧，[`DisposableLibraryOfThisProcess`] 持着）；开全仓那一份的几处可以同时持着读的一侧
/// （[`TheOneLibraryKeptForThisProcess`]），持着时谁都设不了一次性库。
static DISPOSABLE_LIBRARY_SETTING: RwLock<()> = RwLock::new(());

/// 一次性库设成这个进程开的那一份期间持着的守卫：丢掉时换回全仓那一份。
pub struct DisposableLibraryOfThisProcess<'library> {
    library: &'library DisposableLibrary,
    _serializing: RwLockWriteGuard<'static, ()>,
}

/// 这个进程开的是全仓那一份的期间持着的守卫：持着时这个进程里谁都设不了一次性库（[`DisposableLibrary::make_it_the_library_of_this_process`]
/// 等所有这种守卫放手），[`CrashLibrary::open`] 与 [`LibraryOpenInThisProcess::open_or_share`] 开到的都是全仓那一份；几处可以同时持着。
/// 测试里经产品入口开全仓那一份的用例（走崩溃点重放入口的项：分片与只录那一趟要两台读同一份库）在开库之前拿它：
/// 不拿的，同一个二进制里设一次性库的用例在它开库那一刻设着，它就开到那一份，那条用例收尾停掉临时服务，这一边接着读写就连不上（退 75）。
/// 次序同设一次性库：先拿它，再拿改环境的锁。持着它时不许再设一次性库、也不许再拿它（同一条线程上等自己，读写锁不可重入）。
pub struct TheOneLibraryKeptForThisProcess {
    _keeping_disposable_libraries_out: RwLockReadGuard<'static, ()>,
}

impl TheOneLibraryKeptForThisProcess {
    /// 等设着的一次性库放手，拿到之后到丢掉为止这个进程开的都是全仓那一份。
    #[must_use]
    pub fn keep_disposable_libraries_out() -> Self {
        Self {
            _keeping_disposable_libraries_out: DISPOSABLE_LIBRARY_SETTING
                .read()
                .unwrap_or_else(PoisonError::into_inner),
        }
    }
}

impl DisposableLibraryOfThisProcess<'_> {
    /// 设成当前的那一份一次性库。
    #[must_use]
    pub fn library(&self) -> &DisposableLibrary {
        self.library
    }
}

impl Drop for DisposableLibraryOfThisProcess<'_> {
    fn drop(&mut self) {
        *DISPOSABLE_LIBRARY_OF_THIS_PROCESS
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = None;
    }
}

/// 判段的库所在的那份临时 prune-server（`VerdictStore::temporary_service_of_the_library`）：连哪（unix socket、TCP 地址，各自可以没有），
/// 用哪段键空间（键原样那一份是空的）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemporaryServiceOfTheLibrary {
    pub socket_path: Option<PathBuf>,
    pub network_address: Option<String>,
    pub key_space_prefix: Vec<u8>,
}

/// 连 `route`、用 `key_space` 的那一份库在不在临时 prune-server 里：一次性库（[`LibraryKeySpace::DisposableLibrary`]）在，
/// TCP 地址照这个进程里起着的临时服务查（[`NETWORK_ADDRESSES_OF_THE_TEMPORARY_SERVICES_IN_THIS_PROCESS`]）；键原样那一份在进程环境设着
/// 临时服务的标记（[`PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET_ENVIRONMENT_VARIABLE`]，变异门禁与证红每一行起的那种）时在，前缀是空的，
/// TCP 地址取 [`PRUNE_SERVER_TCP_ADDRESS_ENVIRONMENT_VARIABLE`]；都不是交 `None`（真库）。
#[must_use]
pub fn temporary_service_of_a_library(
    route: &RouteAddress,
    key_space: &LibraryKeySpace,
) -> Option<TemporaryServiceOfTheLibrary> {
    let socket_path = match route {
        RouteAddress::UnixSocket(socket_path) => Some(socket_path.clone()),
        RouteAddress::Network(_address) => None,
    };
    let network_address_of_the_route = match route {
        RouteAddress::UnixSocket(socket_of_the_route) => {
            NETWORK_ADDRESSES_OF_THE_TEMPORARY_SERVICES_IN_THIS_PROCESS
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .get(socket_of_the_route)
                .cloned()
        }
        RouteAddress::Network(address) => Some(address.clone()),
    };
    match key_space {
        LibraryKeySpace::DisposableLibrary(namespace) => Some(TemporaryServiceOfTheLibrary {
            socket_path,
            network_address: network_address_of_the_route,
            key_space_prefix: namespace.clone(),
        }),
        LibraryKeySpace::TheOneLibrary => {
            let marked =
                std::env::var_os(PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET_ENVIRONMENT_VARIABLE)
                    .unwrap_or_default();
            if marked.is_empty() {
                return None;
            }
            let network_address_handed_in =
                std::env::var(PRUNE_SERVER_TCP_ADDRESS_ENVIRONMENT_VARIABLE)
                    .ok()
                    .filter(|address_handed_in| !address_handed_in.is_empty());
            Some(TemporaryServiceOfTheLibrary {
                socket_path,
                network_address: network_address_handed_in.or(network_address_of_the_route),
                key_space_prefix: Vec::new(),
            })
        }
    }
}

/// 一类测试手里的那份库：几类都开在这个进程的那一份上（[`LibraryOpenInThisProcess::open_or_share`]），同一个进程里共用；类别只进行里报的名字。
pub struct CrashLibrary {
    class: CrashLibraryClass,
    library_name: String,
    library: LibraryOpenInThisProcess,
}

// ============================== 选段 ==============================

/// 这一趟跑哪一段（[`RUN_STAGE_ENVIRONMENT_VARIABLE`]）：`record` 只跑录入段（CPU 跑数据、写进库），`judge` 只跑判段（从库里读），
/// 没设两段先后跑，中间照样经库交接、不在内存里交数据。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunStage {
    RecordOnly,
    JudgeOnly,
    RecordThenJudge,
}

impl RunStage {
    /// 照 [`RUN_STAGE_ENVIRONMENT_VARIABLE`] 选段。
    ///
    /// # Errors
    /// 设了而不是 `record`、`judge`。
    pub fn from_the_environment() -> Result<Self, String> {
        Self::of_the_variable_value(
            std::env::var(RUN_STAGE_ENVIRONMENT_VARIABLE)
                .ok()
                .as_deref(),
        )
    }

    /// 变量的值（`None` 是没设）换成段。
    ///
    /// # Errors
    /// 同 [`RunStage::from_the_environment`]。
    pub fn of_the_variable_value(value: Option<&str>) -> Result<Self, String> {
        match value {
            None => Ok(Self::RecordThenJudge),
            Some("record") => Ok(Self::RecordOnly),
            Some("judge") => Ok(Self::JudgeOnly),
            Some(other) => Err(format!(
                "{RUN_STAGE_ENVIRONMENT_VARIABLE}={other} 认不得：只认 record（只跑录入段）、judge（只跑判段），或不设（两段先后跑）"
            )),
        }
    }

    /// 行里报的名字：`record`、`judge`、`record_then_judge`。
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::RecordOnly => "record",
            Self::JudgeOnly => "judge",
            Self::RecordThenJudge => "record_then_judge",
        }
    }

    #[must_use]
    pub fn records(self) -> bool {
        matches!(self, Self::RecordOnly | Self::RecordThenJudge)
    }

    #[must_use]
    pub fn judges(self) -> bool {
        matches!(self, Self::JudgeOnly | Self::RecordThenJudge)
    }

    /// 这个进程的选段。
    ///
    /// # Panics
    /// [`RUN_STAGE_ENVIRONMENT_VARIABLE`] 设了而认不得。
    #[must_use]
    pub fn of_this_process() -> Self {
        Self::from_the_environment().unwrap_or_else(|refusal| panic!("{refusal}"))
    }
}

/// 过一次库的结局：只录那一趟只把这一项写进库、不判；判段从库里读回判完交回判的结果；两台分片时这台只判自己领的块，整份的结果要等两份都判进服务那一份之后不分片再跑的那一趟。
#[must_use]
#[derive(Debug)]
pub enum StageOutcome<Judged> {
    RecordedOnly,
    /// 两台分片（`SINGLEFS_CRASH_AMPLIFICATION_SHARE=<i>/<n>`）：这台机器领的块判完、存进库了，
    /// 别的机器领的块库里还没有（那一台还没判完写进服务），整份的结果交不出来；调用方拿到它就收工，不拿它判用例钉死的数。
    JudgedMyShareOnly {
        blocks_judged_here: u64,
        blocks_left_to_other_hosts: u64,
    },
    Judged(Judged),
}

impl<Judged> StageOutcome<Judged> {
    /// 判段整份的结果；只录那一趟与只判了自己那一份的是 `None`。
    pub fn judged(self) -> Option<Judged> {
        match self {
            Self::RecordedOnly | Self::JudgedMyShareOnly { .. } => None,
            Self::Judged(judged) => Some(judged),
        }
    }

    /// 判的结果换一种形态，别的结局照旧。
    pub fn map<Mapped>(self, map: impl FnOnce(Judged) -> Mapped) -> StageOutcome<Mapped> {
        match self {
            Self::RecordedOnly => StageOutcome::RecordedOnly,
            Self::JudgedMyShareOnly {
                blocks_judged_here,
                blocks_left_to_other_hosts,
            } => StageOutcome::JudgedMyShareOnly {
                blocks_judged_here,
                blocks_left_to_other_hosts,
            },
            Self::Judged(judged) => StageOutcome::Judged(map(judged)),
        }
    }
}

// ============================== 项名、录入键与清单 ==============================

/// 一项的名字：只进行里报的名字（用例名、流名），不进任何键。1 到 200 字节，只许字母、数字、`_`、`.`、`:`、`/`、`-`。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ItemName(String);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_run_stage_reads_record_judge_or_both() {
        assert_eq!(
            RunStage::of_the_variable_value(None),
            Ok(RunStage::RecordThenJudge)
        );
        assert_eq!(
            RunStage::of_the_variable_value(Some("record")),
            Ok(RunStage::RecordOnly)
        );
        assert_eq!(
            RunStage::of_the_variable_value(Some("judge")),
            Ok(RunStage::JudgeOnly)
        );
        assert!(RunStage::of_the_variable_value(Some("both")).is_err());
        assert!(RunStage::RecordOnly.records() && !RunStage::RecordOnly.judges());
        assert!(!RunStage::JudgeOnly.records() && RunStage::JudgeOnly.judges());
    }

    /// 改连接地址的两个变量只在临时服务底下认（与 `prune_server_client.py` 的 `connection_override_problem` 同判）：标记没设而设了 socket、
    /// 设了 TCP 地址，各交一句点名那个变量、写出路的话；标记没设、两个都没设，与标记设了（不核它与 socket 相同）都交 `None`。
    /// 判别力（`crates/mutations.tsv`）：不看标记、一律认这两个变量，这条红。
    #[test]
    fn connection_variables_are_honoured_only_under_the_temporary_service_marker() {
        let empty = std::ffi::OsStr::new("");
        let marked = std::ffi::OsStr::new("/run/temporary/server.sock");
        let socket = std::ffi::OsStr::new("/run/user/elsewhere/server.sock");
        let network_address = std::ffi::OsStr::new("127.0.0.1:4100");
        for (socket_given, network_address_given, named) in [
            (socket, empty, PRUNE_SERVER_SOCKET_ENVIRONMENT_VARIABLE),
            (
                empty,
                network_address,
                PRUNE_SERVER_TCP_ADDRESS_ENVIRONMENT_VARIABLE,
            ),
        ] {
            let problem =
                connection_override_problem_of(empty, socket_given, network_address_given)
                    .unwrap_or_else(|| panic!("不在临时服务底下设了 {named} 却照认"));
            assert!(
                problem.contains(named)
                    && problem.contains(PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET_ENVIRONMENT_VARIABLE)
                    && problem.contains("--with-temporary-service"),
                "说明要点名那个变量、临时服务的标记与起法：{problem}"
            );
        }
        assert_eq!(
            connection_override_problem_of(empty, empty, empty),
            None,
            "平常跑两个都没设：照常连"
        );
        assert_eq!(
            connection_override_problem_of(marked, socket, network_address),
            None,
            "临时服务底下照认（不核标记与 socket 相同）"
        );
    }

    /// 连哪一处那一步先判改连接地址的变量：不在临时服务底下设了 socket 就拒（不连、不落到别处）；临时服务底下 socket 不在、设了 TCP 地址，
    /// 走那个地址（不读多机配置）。
    /// 判别力（`crates/mutations.tsv`）：连哪一处那一步不判改连接地址的变量（平常跑设了也照连），这条红。
    #[test]
    fn the_route_refuses_a_connection_variable_outside_the_temporary_service_and_takes_it_under_one(
    ) {
        let empty = std::ffi::OsStr::new("");
        let socket_that_is_not_there = std::ffi::OsStr::new(
            "/nonexistent-singlefs-route-test/singlefs-prune-server/server.sock",
        );
        let network_address = std::ffi::OsStr::new("127.0.0.1:4100");
        let configuration_that_is_not_read =
            Path::new("/nonexistent-singlefs-route-test/multi-host.env");
        match prune_server_route_of_the_variables(
            empty,
            socket_that_is_not_there,
            network_address,
            configuration_that_is_not_read,
        ) {
            Ok(route) => panic!("不在临时服务底下设了改连接地址的变量却照连：{route:?}"),
            Err(PruneServerRouteRefusal::ConnectionOverrideOutsideTheTemporaryService(problem)) => {
                assert!(
                    problem.contains(PRUNE_SERVER_SOCKET_ENVIRONMENT_VARIABLE)
                        && problem.contains(PRUNE_SERVER_TCP_ADDRESS_ENVIRONMENT_VARIABLE),
                    "拒绝说明点名设了的两个变量：{problem}"
                );
            }
            Err(PruneServerRouteRefusal::NoPruneServerAddressConfigured(problem)) => {
                panic!("改连接地址被拒要报成改连接地址的变量不认，不是没写服务的地址：{problem}")
            }
        }
        assert_eq!(
            prune_server_route_of_the_variables(
                socket_that_is_not_there,
                socket_that_is_not_there,
                network_address,
                configuration_that_is_not_read
            ),
            Ok(RouteAddress::Network("127.0.0.1:4100".to_string())),
            "临时服务底下：socket 不在就走设了的 TCP 地址"
        );
    }

    /// 改连接地址的变量被拒退 3（与 `prune_server_client.py` 的 `connection-override` 同一个码：`EXIT_OTHER`，用法与环境错），
    /// 不退 75（那是服务不通、可重试）；退出前那一行写同一个码。
    /// 判别力（`crates/mutations.tsv`）：改连接地址被拒照旧退 75，这条红。
    #[test]
    fn a_refused_connection_override_exits_three_like_the_python_client() {
        let empty = std::ffi::OsStr::new("");
        let socket = std::ffi::OsStr::new("/run/user/elsewhere/server.sock");
        let refusal = prune_server_route_of_the_variables(
            empty,
            socket,
            empty,
            Path::new("/nonexistent-singlefs-route-test/multi-host.env"),
        )
        .expect_err("不在临时服务底下设了改连接地址的变量要拒");
        assert_eq!(
            refusal.exit_code(),
            3,
            "改连接地址被拒退 3，同 prune_server_client.py 的 connection-override（EXIT_OTHER）"
        );
        assert_eq!(refusal.exit_code(), OTHER_CLIENT_FAILURE_EXIT_CODE);
        assert!(
            refusal
                .failure_line()
                .starts_with(&format!("{PRUNE_SERVER_FAILURE_LINE_PREFIX} exit_code=3 ")),
            "退出前那一行写同一个码：{}",
            refusal.failure_line()
        );
    }

    /// 场景：调用方的多机配置开了 GPU、开了跨机、写了显卡先后与带引号的 prune-server 地址；另一份调用方的配置只开 GPU、没写地址。
    /// 预期：照头一份写的只开 CPU 的配置全文只有 `ENABLE_GPU=0` 与照抄的地址（去掉引号）两行，照它读出的跑法是单机 CPU、照它连服务连的是
    /// 那个地址；照第二份写的只有 `ENABLE_GPU=0` 一行。CPU 判器放量那几条用例经它写自己的配置：判在 CPU 上、块进全仓那一份库，
    /// 没有 prune-server socket 的那一台照抄来的地址连同一处服务。
    /// 判别力（`crates/mutations.tsv`「只开 CPU 的多机配置照抄调用方的 GPU 开关（CPU 判器那一条走了 GPU）」「只开 CPU 的多机配置不抄
    /// prune-server 的地址（第二台连不上服务）」）：各让这一条红。
    #[test]
    fn the_configuration_on_the_cpu_keeps_only_the_prune_server_address_of_the_caller() {
        let directory = ExclusiveTemporaryDirectory::create("configuration-on-the-cpu")
            .expect("建得出测试自己的临时目录");
        let caller_with_the_address = directory.join("caller-with-the-address.env");
        std::fs::write(
            &caller_with_the_address,
            "# 测试造的调用方配置\nENABLE_GPU=1\nENABLE_ACROSS_MACHINES=1\n\
             GPU_CARDS_BY_PRIORITY='192.0.2.81:0:15G'\nPRUNE_SERVER_TCP_ADDRESS=\"192.0.2.7:7411\"\n",
        )
        .expect("写得进测试自己的调用方配置");
        let caller_without_the_address = directory.join("caller-without-the-address.env");
        std::fs::write(&caller_without_the_address, "ENABLE_GPU=1\n")
            .expect("写得进测试自己的调用方配置");
        let on_the_cpu = multi_host_configuration_on_the_cpu_keeping_the_prune_server_of(
            &caller_with_the_address,
        )
        .expect("读得了调用方配置");
        assert_eq!(
            on_the_cpu, "ENABLE_GPU=0\nPRUNE_SERVER_TCP_ADDRESS=192.0.2.7:7411\n",
            "只有不开 GPU 与照抄的服务地址两行"
        );
        let written = directory.join("multi-host-on-the-cpu.env");
        std::fs::write(&written, on_the_cpu.as_bytes()).expect("写得进只开 CPU 的配置");
        assert_eq!(
            crate::crash_amplification::CrashAmplificationRunMode::from_configuration_file(
                &written
            )
            .expect("读得出跑法"),
            crate::crash_amplification::CrashAmplificationRunMode::SingleHostCpu,
            "照它跑：单机、CPU"
        );
        let socket_that_is_not_there = std::ffi::OsStr::new(
            "/nonexistent-singlefs-route-test/singlefs-prune-server/server.sock",
        );
        assert_eq!(
            prune_server_route_of_the_variables(
                socket_that_is_not_there,
                socket_that_is_not_there,
                std::ffi::OsStr::new(""),
                &written,
            )
            .map_err(|refusal| refusal.failure_line()),
            Ok(RouteAddress::Network("192.0.2.7:7411".to_string())),
            "没有 socket 的那一台照抄来的地址连服务"
        );
        assert_eq!(
            multi_host_configuration_on_the_cpu_keeping_the_prune_server_of(
                &caller_without_the_address
            )
            .expect("读得了调用方配置"),
            "ENABLE_GPU=0\n",
            "调用方没写服务地址：只有不开 GPU 一行"
        );
    }

    /// 临时服务底下（改连接地址的变量认）socket 不在、TCP 地址没设、多机配置里没有 `PRUNE_SERVER_TCP_ADDRESS`：报哪一处都没写服务的地址、退 3，
    /// 与 `prune_server_client.py` 的 `choose_route` 同一情形（`configuration`，`EXIT_OTHER`）同一个码，不退 75（连不上才退 75）；
    /// 退出前那一行写同一个码、写出路（本机要有服务的 socket，第二台要有多机配置里的那一键）。
    /// 判别力（`crates/mutations.tsv`）：哪一处都没写服务的地址照旧退 75，这条红。
    #[test]
    fn no_socket_and_no_tcp_address_anywhere_exits_three_like_the_python_client_configuration_failure(
    ) {
        let directory = ExclusiveTemporaryDirectory::create("route-without-any-address")
            .expect("建得出测试自己的临时目录");
        let configuration_without_the_address = directory.join("multi-host.env");
        std::fs::write(
            &configuration_without_the_address,
            "# 测试造的多机配置：只有别的键，没有服务的地址\nENABLE_GPU=0\n",
        )
        .expect("写得进测试自己的多机配置");
        let socket_that_is_not_there = std::ffi::OsStr::new(
            "/nonexistent-singlefs-route-test/singlefs-prune-server/server.sock",
        );
        let refusal = prune_server_route_of_the_variables(
            socket_that_is_not_there,
            socket_that_is_not_there,
            std::ffi::OsStr::new(""),
            &configuration_without_the_address,
        )
        .expect_err("socket 不在、TCP 地址两处都没有要拒");
        assert!(
            matches!(
                refusal,
                PruneServerRouteRefusal::NoPruneServerAddressConfigured(_)
            ),
            "报成哪一处都没写服务的地址：{refusal:?}"
        );
        assert_eq!(
            refusal.exit_code(),
            3,
            "哪一处都没写服务的地址退 3，同 prune_server_client.py 的 configuration（EXIT_OTHER）"
        );
        let failure_line = refusal.failure_line();
        assert!(
            failure_line.starts_with(&format!("{PRUNE_SERVER_FAILURE_LINE_PREFIX} exit_code=3 ")),
            "退出前那一行写同一个码：{failure_line}"
        );
        assert!(
            failure_line.contains("出路") && failure_line.contains(NETWORK_ADDRESS_KEY),
            "退出前那一行写出路、点名多机配置里要写的那一键：{failure_line}"
        );
    }

    /// 临时服务底下 socket 不在、TCP 地址没设、多机配置读不了（文件不在）：与配置里没有那一键同一种拒、同退 3
    /// （`prune_server_client.py` 的 `configuration_value` 读不了交 None，照「没有地址」报 `configuration`、退 3）。
    /// 判别力（`crates/mutations.tsv`）：多机配置读不了报成另一种拒（改连接地址的变量不认），这条红。
    #[test]
    fn an_unreadable_multi_host_configuration_without_a_socket_exits_three_like_the_python_client()
    {
        let socket_that_is_not_there = std::ffi::OsStr::new(
            "/nonexistent-singlefs-route-test/singlefs-prune-server/server.sock",
        );
        let refusal = prune_server_route_of_the_variables(
            socket_that_is_not_there,
            socket_that_is_not_there,
            std::ffi::OsStr::new(""),
            Path::new("/nonexistent-singlefs-route-test/multi-host.env"),
        )
        .expect_err("socket 不在、多机配置读不了要拒");
        assert!(
            matches!(
                refusal,
                PruneServerRouteRefusal::NoPruneServerAddressConfigured(_)
            ),
            "报成哪一处都没写服务的地址：{refusal:?}"
        );
        assert_eq!(refusal.exit_code(), 3, "多机配置读不了同样退 3");
    }

    /// 场景：另一条线程持着全仓那一份的守卫（[`TheOneLibraryKeptForThisProcess`]），持到这条线程已经去设一次性库之后再放；这条线程在它持着时去设。
    /// 预期：设一次性库等到守卫放手才设得上（设上那一刻守卫已经放手）；守卫持着的整段里这个进程一直没设一次性库。
    /// 判别力：`crates/mutations.tsv` 里以「全仓那一份的守卫不拿设一次性库那把锁」开头的那一行（守卫拿的是它自己新建的一把锁）让这一条红。
    #[test]
    fn setting_a_disposable_library_waits_until_the_guard_keeping_the_one_library_is_dropped() {
        // 守卫那条线程报过「拿到了」之后再持这么久才放：这条线程在这段里已经去设一次性库（守卫拦不住时就设上了）
        const GUARD_HELD_AFTER_IT_IS_REPORTED: std::time::Duration =
            std::time::Duration::from_millis(500);
        // 一次性库留着：自证设一次性库等全仓那一份的守卫放手
        let disposable = DisposableLibrary::new("waits-for-the-one-library-guard")
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        let guard_is_dropped = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let (guard_is_held_sender, guard_is_held_receiver) = std::sync::mpsc::channel::<()>();
        let guarding_thread = std::thread::spawn({
            let guard_is_dropped = Arc::clone(&guard_is_dropped);
            move || {
                let guard = TheOneLibraryKeptForThisProcess::keep_disposable_libraries_out();
                guard_is_held_sender
                    .send(())
                    .expect("设一次性库的那条线程在等这一声，收端还在");
                std::thread::sleep(GUARD_HELD_AFTER_IT_IS_REPORTED);
                let disposable_library_set_while_the_guard_is_held =
                    DISPOSABLE_LIBRARY_OF_THIS_PROCESS
                        .lock()
                        .unwrap_or_else(PoisonError::into_inner)
                        .is_some();
                guard_is_dropped.store(true, Ordering::SeqCst);
                drop(guard);
                disposable_library_set_while_the_guard_is_held
            }
        });
        guard_is_held_receiver
            .recv()
            .expect("守卫那条线程拿到守卫就报一声");
        let the_library_of_this_process = disposable.make_it_the_library_of_this_process();
        let guard_was_dropped_when_the_library_was_set = guard_is_dropped.load(Ordering::SeqCst);
        drop(the_library_of_this_process);
        let disposable_library_set_while_the_guard_is_held = guarding_thread
            .join()
            .expect("守卫那条线程只读不写、不 panic");
        assert!(
            !disposable_library_set_while_the_guard_is_held,
            "守卫持着的整段里这个进程没设一次性库"
        );
        assert!(
            guard_was_dropped_when_the_library_was_set,
            "设一次性库等到全仓那一份的守卫放手才设得上"
        );
    }

    /// 单测登记临时库用的「这一台」：多机配置判法打的那几行原样（这一台是 127.0.0.2，回环段里不是 127.0.0.1 的一个地址，绑得上；
    /// 另一台是文档用的地址段 192.0.2.81）。不起判法。
    pub(super) fn addresses_whose_host_is_not_the_usual_loopback(
    ) -> crate::gpu_card_configuration::MultiHostAddresses {
        crate::gpu_card_configuration::MultiHostAddresses::from_the_emitted_assignments(
            "HOSTS=127.0.0.2\\ 192.0.2.81\nTHIS_HOST_ADDRESS=127.0.0.2\nPEER_HOST_ADDRESS=192.0.2.81\n",
        )
        .expect("单测写死的判法输出认得出")
    }
}
