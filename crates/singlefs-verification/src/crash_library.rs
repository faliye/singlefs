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
//! 并行跑的用例各起各的，互不相见；一份一次性库里是那份临时服务里的一段名字空间，从空库起。
//! 经产品入口跑的用例把一次性库设成这个进程开的那一份（[`DisposableLibrary::make_it_the_library_of_this_process`]，进程内的设置，不是环境变量，
//! 也不传给子进程；一次性库连的是它自己起的那份临时服务，所以真任务里没有写真服务的这一条路）。
//!
//! 两段怎么接（[`two_stages`]）：录入段把这一项的几样各按摘要写进库、再按录入键（录入函数单元指纹与录入输入摘要，[`recording_key_of_the_inputs`]）
//! 登一份清单（[`ItemManifest`]）；录入键下已有清单就不再录（命中就跳过）。判段照同一份输入现算录入键、读清单，再按清单里的摘要读回那几样。
//! 选段照 [`RUN_STAGE_ENVIRONMENT_VARIABLE`]：`record` 只录、`judge` 只判、不设两段先后跑（中间照样经库）。
//!
//! 同一个进程里几处开库共用一份连接（[`LibraryOpenInThisProcess`]：按键空间各一份，库里一把锁一次一个读写）；不同进程各连各的，谁都不等谁。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{
    Arc, Mutex, MutexGuard, OnceLock, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard, Weak,
};

use crate::hexadecimal::hexadecimal_text;
use crate::sha256::{sha256_digest, DIGEST_BYTES};

use crate::crash_facts::{FlowFacts, FlowFactsReadBack};
use crate::crash_identity::{judging_units_key, JudgingUnits, JudgingUnitsKey};
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
use prune_server::reliable_client::OTHER_CLIENT_FAILURE_EXIT_CODE;
use prune_server::server::{start_in_this_process, ServiceInThisProcess};

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

/// 这个进程的工具链（判法单元指纹与录入键都拼它）：问一次 cargo 与 rustc，之后照这一份。
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
        }
    }
}

/// 这个进程里建过几份独占的一次性临时目录（[`ExclusiveTemporaryDirectory::create`] 每次取一个号，名字里带它）。
static EXCLUSIVE_TEMPORARY_DIRECTORIES_CREATED_IN_THIS_PROCESS: AtomicU64 = AtomicU64::new(0);

/// 建一份独占的一次性临时目录最多试几个名字：名字撞上已有的目录（同进程号的旧进程被杀时留下的）就换下一个号，试满还建不出就报错。
const MOST_EXCLUSIVE_TEMPORARY_DIRECTORY_NAMES_TRIED: u32 = 64;

/// 调用方独占的一份一次性临时目录：`${TMPDIR:-/tmp}` 下 `singlefs-<标签>-<进程号>-<进程内序号>`（标签里的 `/` 换成 `-`，目录只占一层），`create_dir` 排他新建；
/// 丢掉时只删这一份（连同里面的东西）。同一个进程里两次用同一个标签各拿一份、互不删对方；两个进程同跑一个测试二进制，进程号不同。
/// 标签只为在临时目录里认得出是谁建的，不靠它区分：验证包的单测与集成测试要一份自己的临时目录都经它建
/// （照崩溃注入镜像目录的写法，`crate::crash_injection` 的 `create_the_image_directory_of_a_campaign_call`）。
pub struct ExclusiveTemporaryDirectory {
    path: PathBuf,
}

impl ExclusiveTemporaryDirectory {
    /// # Errors
    /// 临时目录里建不了目录；试满 [`MOST_EXCLUSIVE_TEMPORARY_DIRECTORY_NAMES_TRIED`] 个名字都撞上已有的。
    pub fn create(label: &str) -> Result<Self, String> {
        Self::create_in(&std::env::temp_dir(), label, &std::fs::DirBuilder::new())
    }

    /// 同 [`Self::create`]，只是建在 `root` 下、照 `directory_builder` 建（权限由它定）。
    ///
    /// # Errors
    /// 同 [`Self::create`]。
    fn create_in(
        root: &Path,
        label: &str,
        directory_builder: &std::fs::DirBuilder,
    ) -> Result<Self, String> {
        let mut last_name_tried = PathBuf::new();
        // 迭代上界：MOST_EXCLUSIVE_TEMPORARY_DIRECTORY_NAMES_TRIED 次；只有名字撞上已有目录时才试下一个
        for _name_tried in 0..MOST_EXCLUSIVE_TEMPORARY_DIRECTORY_NAMES_TRIED {
            let serial_in_this_process = EXCLUSIVE_TEMPORARY_DIRECTORIES_CREATED_IN_THIS_PROCESS
                .fetch_add(1, Ordering::SeqCst);
            let path = root.join(format!(
                "singlefs-{}-{}-{serial_in_this_process}",
                label.replace('/', "-"),
                std::process::id()
            ));
            match directory_builder.create(&path) {
                Ok(()) => return Ok(Self { path }),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    last_name_tried = path;
                }
                Err(error) => {
                    return Err(format!("建不了一次性临时目录 {}：{error}", path.display()))
                }
            }
        }
        Err(format!(
            "一次性临时目录试了 {MOST_EXCLUSIVE_TEMPORARY_DIRECTORY_NAMES_TRIED} 个名字都已经在（最后一个 {}）",
            last_name_tried.display()
        ))
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// 当路径用（`directory.join(…)`、交给收 `&Path` 的函数）：就是 [`ExclusiveTemporaryDirectory::path`]。
impl std::ops::Deref for ExclusiveTemporaryDirectory {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.path
    }
}

impl AsRef<Path> for ExclusiveTemporaryDirectory {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

impl Drop for ExclusiveTemporaryDirectory {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_dir_all(&self.path) {
            if error.kind() != std::io::ErrorKind::NotFound {
                eprintln!("删不掉一次性临时目录 {}：{error}", self.path.display());
            }
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

/// 这个进程里建过几份一次性库（[`DisposableLibrary::new`] 每次取一个号，名字空间里带它）。
static DISPOSABLE_LIBRARIES_CREATED_IN_THIS_PROCESS: AtomicU64 = AtomicU64::new(0);

/// 一次性库自己起的那一份临时 prune-server：在这个进程里起（`prune_server::server::start_in_this_process`），状态目录在它独占的
/// 一次性临时目录里；socket 也放那里，全路径超了客户端认的上限 `prune_server::configuration::MAXIMUM_SOCKET_PATH_BYTES`（`TMPDIR` 嵌得深，
/// 变异门禁每行一个的临时目录那种）时另放进 [`SHORT_SOCKET_DIRECTORY_ROOT`] 下一份只有本用户进得去的短一次性目录。
/// 丢掉时先停服（不再接连接、关掉还连着的连接、等库关上），再删那一两份目录；停不干净是缺陷（还有线程拿着服务），不在 panic 中就 panic。
struct TemporaryPruneServer {
    service: Option<ServiceInThisProcess>,
    service_directory: ExclusiveTemporaryDirectory,
    /// socket 在 `service_directory` 里放不下时另开的那份短目录（随服务放手一起删）；放得下时没有。
    #[expect(dead_code, reason = "只为持有到放手：丢掉时删那份短目录，不读它")]
    short_socket_directory: Option<ExclusiveTemporaryDirectory>,
}

/// 临时服务的 socket 在它那份一次性临时目录（或另开的短目录）里的文件名。
const TEMPORARY_PRUNE_SERVER_SOCKET_FILE_NAME: &str = "socket";
/// 临时服务的状态目录（故障日志与库）在它那份一次性临时目录里的名字。
const TEMPORARY_PRUNE_SERVER_STATE_DIRECTORY_NAME: &str = "state";
/// socket 放不进临时服务自己的目录时，另开短目录的地方（与 `scripts/prune-server/prune_server_client_selftest.py` 的 `selftest_root`
/// 在 `TMPDIR` 太长时退到同一处）。
const SHORT_SOCKET_DIRECTORY_ROOT: &str = "/tmp";
/// 那份短目录的权限：只有本用户进得去（[`SHORT_SOCKET_DIRECTORY_ROOT`] 人人可写，socket 不放进别人进得去的目录）。
const SHORT_SOCKET_DIRECTORY_MODE: u32 = 0o700;

impl TemporaryPruneServer {
    /// 服务的一次性临时目录建在 `temporary_root` 下；socket 放进去超上限时另开 [`SHORT_SOCKET_DIRECTORY_ROOT`] 下的短目录放 socket。
    ///
    /// # Errors
    /// 建不了一次性临时目录或短目录；服务起不来（socket 路径超了上限、库开不了、端口绑不上，说明连同 prune-server 写的出路）。
    fn start_under(temporary_root: &Path) -> Result<Self, String> {
        use std::os::unix::fs::DirBuilderExt;
        let service_directory = ExclusiveTemporaryDirectory::create_in(
            temporary_root,
            "prune-server",
            &std::fs::DirBuilder::new(),
        )?;
        let socket_in_the_service_directory =
            service_directory.join(TEMPORARY_PRUNE_SERVER_SOCKET_FILE_NAME);
        let short_socket_directory =
            if socket_path_problem(&socket_in_the_service_directory).is_some() {
                let mut owner_only_directory_builder = std::fs::DirBuilder::new();
                owner_only_directory_builder.mode(SHORT_SOCKET_DIRECTORY_MODE);
                Some(ExclusiveTemporaryDirectory::create_in(
                    Path::new(SHORT_SOCKET_DIRECTORY_ROOT),
                    "prune-server-socket",
                    &owner_only_directory_builder,
                )?)
            } else {
                None
            };
        let socket_path = short_socket_directory
            .as_ref()
            .map_or(socket_in_the_service_directory, |short_directory| {
                short_directory.join(TEMPORARY_PRUNE_SERVER_SOCKET_FILE_NAME)
            });
        let service = start_in_this_process(
            &service_directory.join(TEMPORARY_PRUNE_SERVER_STATE_DIRECTORY_NAME),
            &socket_path,
        )
        .map_err(|failure| {
            let (problem, next_step) = failure.describe();
            format!(
                "在这个进程里起不来临时 prune-server（{}）：{problem}；{next_step}",
                socket_path.display()
            )
        })?;
        Ok(Self {
            service: Some(service),
            service_directory,
            short_socket_directory,
        })
    }

    fn service(&self) -> &ServiceInThisProcess {
        self.service
            .as_ref()
            .expect("服务在放手之前一直在：只有 Drop 取走它")
    }
}

impl Drop for TemporaryPruneServer {
    fn drop(&mut self) {
        let Some(service) = self.service.take() else {
            return;
        };
        if let Err(problem) = service.stop() {
            let message = format!(
                "临时 prune-server（{}）没停干净：{problem}",
                self.service_directory.display()
            );
            if std::thread::panicking() {
                eprintln!("  ✗ {message}");
            } else {
                panic!("{message}");
            }
        }
    }
}

/// 一份一次性库：它自己在这个进程里起的那份临时 prune-server（[`TemporaryPruneServer`]）里的一段名字空间（[`LibraryKeySpace::DisposableLibrary`]），
/// 从空库起；另带一份独占的一次性临时目录（用例自己的配置文件、日志这类放这里）。不读连哪一处的环境变量、不连真服务：测试写不到真服务里去。
/// 丢掉时停掉那份临时服务、删它的目录（名字空间里的行随之删掉），再删那份临时目录。
pub struct DisposableLibrary {
    route: RouteAddress,
    key_space: LibraryKeySpace,
    scratch_directory: ExclusiveTemporaryDirectory,
    temporary_service: TemporaryPruneServer,
}

impl DisposableLibrary {
    /// # Errors
    /// 建不了一次性临时目录；临时 prune-server 起不来。
    pub fn new(label: &str) -> Result<Self, String> {
        Self::new_under(label, &std::env::temp_dir())
    }

    /// 同 [`Self::new`]，只是临时服务的目录与那份临时目录建在 `temporary_root` 下（单测拿一条长目录当 `TMPDIR` 用，不改进程环境：
    /// 同一进程里并行跑的别的用例照旧读 `TMPDIR`）。
    ///
    /// # Errors
    /// 同 [`Self::new`]。
    fn new_under(label: &str, temporary_root: &Path) -> Result<Self, String> {
        let temporary_service = TemporaryPruneServer::start_under(temporary_root)?;
        let socket_path = temporary_service.service().socket_path().to_path_buf();
        let scratch_directory = ExclusiveTemporaryDirectory::create_in(
            temporary_root,
            &format!("disposable-library-{label}"),
            &std::fs::DirBuilder::new(),
        )?;
        let serial_in_this_process =
            DISPOSABLE_LIBRARIES_CREATED_IN_THIS_PROCESS.fetch_add(1, Ordering::SeqCst);
        let nanoseconds_since_the_epoch = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |since_the_epoch| since_the_epoch.as_nanos());
        let mut namespace = vec![DISPOSABLE_LIBRARY_KEY_SPACE_LEADING_BYTE];
        namespace.extend_from_slice(
            format!(
                "{DISPOSABLE_LIBRARY_NAMESPACE_NAME_PREFIX}{label}/{}-{nanoseconds_since_the_epoch}-{serial_in_this_process}/",
                std::process::id()
            )
            .as_bytes(),
        );
        Ok(Self {
            route: RouteAddress::UnixSocket(socket_path),
            key_space: LibraryKeySpace::DisposableLibrary(namespace),
            scratch_directory,
            temporary_service,
        })
    }

    /// 它自己起的那份临时 prune-server 的目录（状态目录在里面；socket 放得下时也在里面，[`TemporaryPruneServer`]），丢掉之后不在。
    #[must_use]
    pub fn temporary_service_directory(&self) -> &Path {
        self.temporary_service.service_directory.path()
    }

    /// 那份临时 prune-server 的 TCP 实际绑上的地址（本机回环），停服之后连不上。
    #[must_use]
    pub fn temporary_service_network_address(&self) -> std::net::SocketAddr {
        self.temporary_service.service().bound_network_address()
    }

    /// 那份独占的一次性临时目录。
    #[must_use]
    pub fn directory(&self) -> &Path {
        self.scratch_directory.path()
    }

    /// 这一份在服务里的键空间。
    #[must_use]
    pub fn key_space(&self) -> &LibraryKeySpace {
        &self.key_space
    }

    /// 那份临时服务连哪一处（用例另连同一份临时服务里别的键空间时用）。
    #[must_use]
    pub fn route(&self) -> &RouteAddress {
        &self.route
    }

    /// 连这一份（一个新的客户端；追加日志在这一端自己的一次性目录里，随这一端放手删掉，[`PruneServerStorage::connect_with_a_journal_of_its_own`]）。
    ///
    /// # Errors
    /// 连不上（交回错误的那一种处置时）；建不了一次性目录。
    pub fn connect(
        &self,
        failure_handling: PruneServerFailureHandling,
    ) -> Result<Arc<PruneServerStorage>, String> {
        PruneServerStorage::connect_with_a_journal_of_its_own(
            self.route.clone(),
            failure_handling,
            self.key_space.clone(),
        )
        .map_err(|error| format!("连不上临时 prune-server（{:?}）：{error}", self.route))
    }

    /// 开这一份（[`VerdictStore::open_the_prune_server`]：头一次开写下布局行）。同一份开几次读到的是同一批行（重开、另一个进程开的样子）。
    ///
    /// # Errors
    /// 同 [`Self::connect`]；开库不成（布局不是这一版、读写不成）。
    pub fn open_store(
        &self,
        failure_handling: PruneServerFailureHandling,
    ) -> Result<VerdictStore, String> {
        VerdictStore::open_the_prune_server(self.connect(failure_handling)?)
            .map_err(|error| format!("开不了一次性库：{error:?}"))
    }

    /// 把这一份设成这个进程开的那一份（经产品入口跑的用例用）：交回的守卫在期间，这个进程里 [`CrashLibrary::open`]、
    /// [`LibraryOpenInThisProcess::open_or_share`] 开的都是这一份；守卫丢掉时换回全仓那一份。同一时刻一个进程只设一份：几条用例同时设，
    /// 后来的等先设的那一份的守卫丢掉。是进程内的设置，不是环境变量，不传给子进程。
    /// 测试里经产品入口开库的每一处都要先设自己那一份：没设就开的，开到的是此刻同一个进程里别的用例设着的那一份（那条用例收尾时停掉它的临时服务，
    /// 这一边接着读写就连不上），没人设着时开到全仓那一份（测试不许写真服务）。
    /// 有人持着全仓那一份的守卫（[`TheOneLibraryKeptForThisProcess`]）时，等它们都放手才设。
    #[must_use]
    pub fn make_it_the_library_of_this_process(&self) -> DisposableLibraryOfThisProcess<'_> {
        let serializing = DISPOSABLE_LIBRARY_SETTING
            .write()
            .unwrap_or_else(PoisonError::into_inner);
        *DISPOSABLE_LIBRARY_OF_THIS_PROCESS
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = Some(DisposableLibraryChosen {
            route: self.route.clone(),
            key_space: self.key_space.clone(),
        });
        DisposableLibraryOfThisProcess {
            library: self,
            _serializing: serializing,
        }
    }
}

/// 一次性库当路径用（`directory.join(…)`、交给收 `&Path` 的函数）：就是它那份一次性临时目录 [`DisposableLibrary::directory`]。
impl std::ops::Deref for DisposableLibrary {
    type Target = Path;

    fn deref(&self) -> &Path {
        self.scratch_directory.path()
    }
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

// ============================== 这个进程里开着的库 ==============================

/// 这个进程里开着的库在登记表里的键：连哪一处（`RouteAddress` 的写法）与键空间的前缀字节（全仓那一份是空的）。
type KeyOfALibraryOpenInThisProcess = (String, Vec<u8>);

/// 开着的一份：库，一把锁一次一个读写（容错客户端追加日志的一次性目录在它连着的那一端里，随最后一个放手的删掉）。
struct OpenLibrary {
    store: Mutex<VerdictStore>,
}

/// 这个进程里开着的判定库：按（连哪一处，键空间）各记一份。同一个进程里几处开同一份（并行跑的几条用例、几类测试）共用这一份连接，
/// 判定、复用、写入都经它，库里的一把锁让它们一次一个读写。
static LIBRARIES_OPEN_IN_THIS_PROCESS: Mutex<
    BTreeMap<KeyOfALibraryOpenInThisProcess, Weak<OpenLibrary>>,
> = Mutex::new(BTreeMap::new());

/// 一处手里的那份库：这个进程里同一份已经开着就共用那一份，没开着才连服务、开库；最后一个放手的关掉连接。
pub struct LibraryOpenInThisProcess {
    key: KeyOfALibraryOpenInThisProcess,
    open: Option<Arc<OpenLibrary>>,
}

impl LibraryOpenInThisProcess {
    /// 开这个进程的那一份（全仓那一份；这个进程设了一次性库就是那一份，[`DisposableLibrary::make_it_the_library_of_this_process`]），
    /// 这个进程里已经开着就共用。全仓那一份连不上、读写不成：打一行 [`PRUNE_SERVER_FAILURE_LINE_PREFIX`]、照 prune-server 的退出码
    /// 退出进程（75 服务不可用），不当成「库里没有」去现判；改连接地址的变量被拒、哪一处都没写服务的地址同样打那一行、退 3
    /// （[`PruneServerRouteRefusal::exit_code`]）；
    /// 一次性库那一份交回错误。
    ///
    /// # Errors
    /// 一次性库那一份连不上、开不了；建不了追加日志的一次性目录；全仓那一份布局不是这一版、判定向量表不合布局。
    pub fn open_or_share() -> Result<Self, String> {
        let chosen = DISPOSABLE_LIBRARY_OF_THIS_PROCESS
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        let (route, key_space, failure_handling) = match chosen {
            Some(DisposableLibraryChosen { route, key_space }) => {
                (route, key_space, PruneServerFailureHandling::ReturnTheError)
            }
            None => match prune_server_route_of_the_configuration() {
                Ok(route) => (
                    route,
                    LibraryKeySpace::TheOneLibrary,
                    PruneServerFailureHandling::ExitTheProcess,
                ),
                Err(refusal) => {
                    eprintln!("{}", refusal.failure_line());
                    std::process::exit(refusal.exit_code())
                }
            },
        };
        let key = (format!("{route:?}"), key_space.key_prefix().to_vec());
        let mut open_libraries = LIBRARIES_OPEN_IN_THIS_PROCESS
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if let Some(open) = open_libraries.get(&key).and_then(Weak::upgrade) {
            return Ok(Self {
                key,
                open: Some(open),
            });
        }
        let storage = PruneServerStorage::connect_with_a_journal_of_its_own(
            route,
            failure_handling,
            key_space,
        )?;
        let store = VerdictStore::open_the_prune_server(storage)
            .map_err(|error| format!("开不了 prune-server 里那一份崩溃判定库：{error:?}"))?;
        let open = Arc::new(OpenLibrary {
            store: Mutex::new(store),
        });
        open_libraries.insert(key.clone(), Arc::downgrade(&open));
        Ok(Self {
            key,
            open: Some(open),
        })
    }

    /// 库里那一把锁：一次一个读写。
    pub fn lock(&self) -> MutexGuard<'_, VerdictStore> {
        self.open
            .as_ref()
            .expect("库在放手之前一直在：只有 Drop 取走它")
            .store
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}

impl Drop for LibraryOpenInThisProcess {
    fn drop(&mut self) {
        let mut open_libraries = LIBRARIES_OPEN_IN_THIS_PROCESS
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        drop(self.open.take());
        if open_libraries
            .get(&self.key)
            .is_some_and(|open| open.strong_count() == 0)
        {
            open_libraries.remove(&self.key);
        }
    }
}

/// 一类测试手里的那份库：几类都开在这个进程的那一份上（[`LibraryOpenInThisProcess::open_or_share`]），同一个进程里共用；类别只进行里报的名字。
pub struct CrashLibrary {
    class: CrashLibraryClass,
    library_name: String,
    library: LibraryOpenInThisProcess,
}

impl CrashLibrary {
    /// 开这一类的库（这个进程的那一份）。
    ///
    /// # Errors
    /// 同 [`LibraryOpenInThisProcess::open_or_share`]。
    pub fn open(class: CrashLibraryClass) -> Result<Self, String> {
        let library = LibraryOpenInThisProcess::open_or_share()?;
        let library_name = library.lock().library_name().to_string();
        Ok(Self {
            class,
            library_name,
            library,
        })
    }

    #[must_use]
    pub fn class(&self) -> CrashLibraryClass {
        self.class
    }

    /// 库的名字（连的是哪一处、哪一个键空间）：只进行里报的名字。
    #[must_use]
    pub fn library_name(&self) -> &str {
        &self.library_name
    }

    /// 库里那一把锁：一次一个读写。
    pub fn lock(&self) -> MutexGuard<'_, VerdictStore> {
        self.library.lock()
    }

    /// 这个进程里开着的那一份（给定状态表的引擎按它共用）。
    #[must_use]
    pub fn shared_handle(&self) -> &LibraryOpenInThisProcess {
        &self.library
    }
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

impl ItemName {
    /// # Errors
    /// 空、超过 200 字节、带别的字符。
    pub fn new(text: &str) -> Result<Self, String> {
        if text.is_empty() || text.len() > 200 {
            return Err(format!("项名要 1 到 200 字节：{text:?}"));
        }
        if let Some(bad) = text
            .chars()
            .find(|character| !(character.is_ascii_alphanumeric() || "_.:/-".contains(*character)))
        {
            return Err(format!(
                "项名只许字母、数字、_ . : / -，{text:?} 里有 {bad:?}"
            ));
        }
        Ok(Self(text.to_string()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 一趟录入的输入摘要：录入段要写进库的那几样（流、展开、判表的开关这类）按内容取的 SHA-256，由调用方算。同一份输入同一个摘要。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RecordingInputsDigest(pub [u8; DIGEST_BYTES]);

/// 这一趟的录入键与凭它跳不跳过录入（[`recording_key_of_the_inputs`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordingKeyOfThisRun {
    pub key: RecordingKey,
    /// 录入函数单元指纹算得出：录入键下已有清单就跳过录入。算不出（拒算）时只按输入摘要登清单、照录。
    pub skips_the_recording_on_a_hit: bool,
    /// 录入函数单元拒算时那一句（[`crate::crash_identity::JudgingUnitsKeyRefused::sentence`]）。
    pub refusal: Option<String>,
}

/// 录入键：录入函数单元指纹（[`judging_units_key`] 的 [`JudgingUnits::Recording`] 那一组）与录入输入摘要拼起来取 SHA-256。
/// 录入函数单元拒算时键只拼输入摘要，交回的 `skips_the_recording_on_a_hit` 是假：照录、不复用。
#[must_use]
pub fn recording_key_of_the_inputs(inputs: &RecordingInputsDigest) -> RecordingKeyOfThisRun {
    recording_key_of_the_inputs_under(
        &judging_units_key(&JudgingUnits::Recording, toolchain_of_this_process())
            .map_err(|refused| refused.sentence()),
        inputs,
    )
}

/// 同 [`recording_key_of_the_inputs`]，录入函数单元那一把由调用方给（单测造拒算用）。
#[must_use]
pub fn recording_key_of_the_inputs_under(
    recording_units: &Result<JudgingUnitsKey, String>,
    inputs: &RecordingInputsDigest,
) -> RecordingKeyOfThisRun {
    match recording_units {
        Ok(units) => RecordingKeyOfThisRun {
            key: RecordingKey(sha256_digest(
                format!(
                    "singlefs recording key\nrecording_units={}\ninputs={}\n",
                    hexadecimal_text(units.fingerprint()),
                    hexadecimal_text(&inputs.0)
                )
                .as_bytes(),
            )),
            skips_the_recording_on_a_hit: true,
            refusal: None,
        },
        Err(sentence) => RecordingKeyOfThisRun {
            key: RecordingKey(sha256_digest(
                format!(
                    "singlefs recording key\nrecording_units=refused\ninputs={}\n",
                    hexadecimal_text(&inputs.0)
                )
                .as_bytes(),
            )),
            skips_the_recording_on_a_hit: false,
            refusal: Some(sentence.clone()),
        },
    }
}

/// 录入段登在库里的清单：这一项是哪一类（`kind`，一个写死的串），录下的几样各叫什么、摘要是什么。判段只认清单指到的那几样。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemManifest {
    pub kind: String,
    pub digests: Vec<(String, [u8; DIGEST_BYTES])>,
}

const ITEM_MANIFEST_MAGIC: &[u8; 8] = b"SFITEM01";

impl ItemManifest {
    /// 清单里叫 `name` 的那一样的摘要。
    ///
    /// # Errors
    /// 清单里没有这一样。
    pub fn digest_named(&self, name: &str) -> Result<[u8; DIGEST_BYTES], String> {
        self.digests
            .iter()
            .find(|(digest_name, _)| digest_name == name)
            .map(|(_, digest)| *digest)
            .ok_or_else(|| {
                format!(
                    "清单（{}）里没有「{name}」这一样：录入段那一趟没录它",
                    self.kind
                )
            })
    }

    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = ITEM_MANIFEST_MAGIC.to_vec();
        let put_text = |destination_bytes: &mut Vec<u8>, text: &str| {
            destination_bytes
                .extend_from_slice(&u64::try_from(text.len()).expect("装得进 u64").to_le_bytes());
            destination_bytes.extend_from_slice(text.as_bytes());
        };
        put_text(&mut bytes, &self.kind);
        bytes.extend_from_slice(
            &u64::try_from(self.digests.len())
                .expect("装得进 u64")
                .to_le_bytes(),
        );
        for (name, digest) in &self.digests {
            put_text(&mut bytes, name);
            bytes.extend_from_slice(digest);
        }
        bytes
    }

    /// 头不对、字节不够或多出交回 `None`。
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut at = 0usize;
        let take = |read_position: &mut usize, count: usize| -> Option<&[u8]> {
            let slice = bytes.get(*read_position..*read_position + count)?;
            *read_position += count;
            Some(slice)
        };
        if take(&mut at, ITEM_MANIFEST_MAGIC.len())? != ITEM_MANIFEST_MAGIC {
            return None;
        }
        let read_count = |read_position: &mut usize| -> Option<usize> {
            let raw: [u8; 8] = take(read_position, 8)?.try_into().ok()?;
            usize::try_from(u64::from_le_bytes(raw)).ok()
        };
        let read_text = |read_position: &mut usize| -> Option<String> {
            let length = read_count(read_position)?;
            String::from_utf8(take(read_position, length)?.to_vec()).ok()
        };
        let kind = read_text(&mut at)?;
        let count = read_count(&mut at)?;
        let mut digests = Vec::with_capacity(count.min(64));
        for _ in 0..count {
            let name = read_text(&mut at)?;
            let digest: [u8; DIGEST_BYTES] = take(&mut at, DIGEST_BYTES)?.try_into().ok()?;
            digests.push((name, digest));
        }
        if at != bytes.len() {
            return None;
        }
        Some(Self { kind, digests })
    }

    /// 清单自己的摘要（行里报它的头 8 字节）。
    #[must_use]
    pub fn digest(&self) -> [u8; DIGEST_BYTES] {
        sha256_digest(&self.to_bytes())
    }
}

impl CrashLibrary {
    /// 录入段按录入键登这一项的清单，打 `CRASH_LIBRARY_RECORDED` 那一行（`reused` 是这一趟录入键命中、没再录）。
    ///
    /// # Errors
    /// 库写失败。
    pub fn record_the_item(
        &self,
        item: &ItemName,
        recording_key: &RecordingKey,
        manifest: &ItemManifest,
    ) -> Result<(), String> {
        self.lock()
            .store_recorded_run_manifest(recording_key, &manifest.to_bytes())
            .map_err(|error| format!("项「{}」的清单写不进库：{error:?}", item.as_str()))?;
        self.print_the_recorded_line(item, manifest);
        Ok(())
    }

    fn print_the_recorded_line(&self, item: &ItemName, manifest: &ItemManifest) {
        println!(
            "CRASH_LIBRARY_RECORDED class={} item={} manifest={} library={}",
            self.class.name(),
            item.as_str(),
            hexadecimal_text(&manifest.digest()[..8]),
            self.library_name
        );
    }

    /// 录入键下登的清单；没登过交回 `None`。
    ///
    /// # Errors
    /// 库读失败；清单解不出。
    pub fn manifest_under(
        &self,
        recording_key: &RecordingKey,
    ) -> Result<Option<ItemManifest>, String> {
        let Some(bytes) = self
            .lock()
            .read_recorded_run_manifest(recording_key)
            .map_err(|error| format!("读录入键下的清单失败：{error:?}"))?
        else {
            return Ok(None);
        };
        ItemManifest::from_bytes(&bytes)
            .map(Some)
            .ok_or_else(|| "录入键下的清单解不出（字节坏了或编码换了）：重跑录入段".to_string())
    }

    /// 判段照这一趟的输入现算录入键、读清单。
    ///
    /// # Errors
    /// 库读失败；库里没有这一份输入的清单（录入段没跑，或输入变了）；清单解不出。
    pub fn manifest_of_the_inputs(
        &self,
        inputs: &RecordingInputsDigest,
    ) -> Result<ItemManifest, String> {
        let recording = recording_key_of_the_inputs(inputs);
        self.manifest_under(&recording.key)?.ok_or_else(|| {
            format!(
                "{} 类开的库 {} 里没有这一份输入（{}）的清单：先跑录入段（{RUN_STAGE_ENVIRONMENT_VARIABLE}=record，或不设、两段先后跑）",
                self.class.name(),
                self.library_name,
                hexadecimal_text(&inputs.0[..8])
            )
        })
    }
}

/// 两段过一次库：录入段（`record`，CPU 跑数据）把这一项的几样写进库、交回清单，这里按录入键登清单；录入键下已有清单就不再录
/// （命中就跳过，打 `CRASH_LIBRARY_RECORDING_REUSED` 与 `CRASH_LIBRARY_RECORDED` 两行）。判段（`judge`）照同一份输入
/// 现算录入键、读回清单，只拿读回的那一份往下判。选段照 [`RunStage::of_this_process`]。`item` 只进行里报的名字。
///
/// # Errors
/// 开不了库；录入段、登清单、读清单、判段各自的错。
///
/// # Panics
/// 选段写坏了。
pub fn two_stages<Judged>(
    class: CrashLibraryClass,
    item: &ItemName,
    inputs: &RecordingInputsDigest,
    record: impl FnOnce(&CrashLibrary) -> Result<ItemManifest, String>,
    judge: impl FnOnce(&CrashLibrary, &ItemManifest) -> Result<StageOutcome<Judged>, String>,
) -> Result<StageOutcome<Judged>, String> {
    let stage = RunStage::of_this_process();
    let library = CrashLibrary::open(class)?;
    let recording = recording_key_of_the_inputs(inputs);
    two_stages_under_the_recording_key(&library, stage, item, &recording, record, judge)
}

/// [`two_stages`] 的本体：库、选段与录入键由调用方给（单测在一次性的库上、造拒算用）。
///
/// # Errors
/// 同 [`two_stages`]。
pub fn two_stages_under_the_recording_key<Judged>(
    library: &CrashLibrary,
    stage: RunStage,
    item: &ItemName,
    recording: &RecordingKeyOfThisRun,
    record: impl FnOnce(&CrashLibrary) -> Result<ItemManifest, String>,
    judge: impl FnOnce(&CrashLibrary, &ItemManifest) -> Result<StageOutcome<Judged>, String>,
) -> Result<StageOutcome<Judged>, String> {
    two_stages_under_the_recording_key_topping_up_a_reused_recording(
        library,
        stage,
        item,
        recording,
        record,
        |_library| Ok(()),
        judge,
    )
}

/// 同 [`two_stages_under_the_recording_key`]，另带录制复用时补录的一段 `top_up_the_reused_recording`：录入键只罩录入段的一部分输入
/// （崩溃枚举用例的录入键只由流定，挂着的观察者的版本不进键）时，命中之后录入那一段整段不跑，罩不到的那一部分由它在录入段里核、缺的补录；
/// 不命中、照录的那一趟不调它（`record` 自己录全）。只跑判段的那一趟不调它（录入段才写库）。
///
/// # Errors
/// 同 [`two_stages`]；补录那一段的错。
pub fn two_stages_under_the_recording_key_topping_up_a_reused_recording<Judged>(
    library: &CrashLibrary,
    stage: RunStage,
    item: &ItemName,
    recording: &RecordingKeyOfThisRun,
    record: impl FnOnce(&CrashLibrary) -> Result<ItemManifest, String>,
    top_up_the_reused_recording: impl FnOnce(&CrashLibrary) -> Result<(), String>,
    judge: impl FnOnce(&CrashLibrary, &ItemManifest) -> Result<StageOutcome<Judged>, String>,
) -> Result<StageOutcome<Judged>, String> {
    if let Some(refusal) = &recording.refusal {
        println!(
            "CRASH_LIBRARY_RECORDING_KEY_REFUSED class={} item={} {refusal}",
            library.class.name(),
            item.as_str()
        );
    }
    if stage.records() {
        let stored = if recording.skips_the_recording_on_a_hit {
            library.manifest_under(&recording.key)?
        } else {
            None
        };
        match stored {
            Some(manifest) => {
                println!(
                    "CRASH_LIBRARY_RECORDING_REUSED class={} item={} recording_key={}",
                    library.class.name(),
                    item.as_str(),
                    hexadecimal_text(&recording.key.0[..8])
                );
                library.print_the_recorded_line(item, &manifest);
                top_up_the_reused_recording(library)?;
            }
            None => {
                let manifest = record(library)?;
                library.record_the_item(item, &recording.key, &manifest)?;
            }
        }
    }
    if !stage.judges() {
        return Ok(StageOutcome::RecordedOnly);
    }
    let manifest = library.manifest_under(&recording.key)?.ok_or_else(|| {
        format!(
            "{} 类开的库 {} 里没有项「{}」这一份输入的清单：先跑录入段（{RUN_STAGE_ENVIRONMENT_VARIABLE}=record，或不设、两段先后跑）",
            library.class.name(),
            library.library_name,
            item.as_str()
        )
    })?;
    judge(library, &manifest)
}

// ============================== 事实表进库与读回 ==============================

/// 按摘要从 `store` 读事实表：读字节，再交 [`FlowFactsReadBack::from_the_library_bytes`] 解码、核摘要。读库这一步放在这里，
/// `crash_facts` 不引判定库。
///
/// # Errors
/// 库读失败；库里没有这个摘要；解不开；摘要对不上。
pub fn read_flow_facts_back(
    store: &VerdictStore,
    digest: &[u8; DIGEST_BYTES],
) -> Result<FlowFactsReadBack, String> {
    let bytes = store
        .read_flow_facts(digest)
        .map_err(|error| format!("读事实表失败：{error:?}"))?
        .ok_or_else(|| "事实表不在库里：先跑录入段".to_string())?;
    FlowFactsReadBack::from_the_library_bytes(&bytes, digest)
}

/// 把事实表按摘要写进 `store`，再从 `store` 读回：核对只拿读回的那一份。
///
/// # Errors
/// 写不进库；读回失败（[`read_flow_facts_back`]）。
pub fn record_the_facts_and_read_them_back(
    store: &mut VerdictStore,
    facts: &FlowFacts,
) -> Result<FlowFactsReadBack, String> {
    let digest = facts.digest();
    store
        .store_flow_facts(&digest, &facts.to_bytes())
        .map_err(|error| format!("事实表写不进库：{error:?}"))?;
    read_flow_facts_back(store, &digest)
}

#[cfg(feature = "gpu")]
/// 按摘要从库里读打包判表（GPU 判器只收读回的那一份，`crate::crash_judge_dispatch::PackedJudgeTablesReadBack`）：读字节、核摘要、解码。
///
/// # Errors
/// 库读失败；库里没有这个摘要；字节的摘要对不上（库里那份被改过）；解不开。
pub fn read_packed_judge_tables_back(
    store: &VerdictStore,
    digest: &[u8; DIGEST_BYTES],
) -> Result<PackedJudgeTablesReadBack, String> {
    let bytes = store
        .read_packed_judge_tables(digest)
        .map_err(|error| format!("读 GPU 判表失败：{error:?}"))?
        .ok_or_else(|| "GPU 判表不在库里：先跑录入段".to_string())?;
    PackedJudgeTablesReadBack::from_the_library_bytes(&bytes, digest)
}

#[cfg(feature = "gpu")]
/// 把打包判表按摘要写进 `store`，再从 `store` 读回：GPU 判器只拿读回的那一份。录入段、给定状态表的引擎与对拍用例都经它。
///
/// # Errors
/// 写不进库；读回失败（[`read_packed_judge_tables_back`]）。
pub fn record_the_judge_tables_and_read_them_back(
    store: &mut VerdictStore,
    tables: &PackedJudgeTables,
) -> Result<PackedJudgeTablesReadBack, String> {
    let bytes = tables.to_bytes();
    let digest = sha256_digest(&bytes);
    store
        .store_packed_judge_tables(&digest, &bytes)
        .map_err(|error| format!("GPU 判表写不进库：{error:?}"))?;
    drop(bytes);
    read_packed_judge_tables_back(store, &digest)
}

// ============================== 对拍用例的两段与对拍结论 ==============================

/// 对拍用例的事实表清单里事实表那一样的名字。
const PAIRING_MANIFEST_FACTS: &str = "facts";

/// 第 ② 段核对的 GPU 与 CPU 参照对拍用例用：录入段把事实表按摘要写进库、按录入键（事实表摘要）登清单；判段照同一张事实表现算录入键、
/// 读清单、按摘要读回，核对只拿读回的那一份。只录那一趟交回 [`StageOutcome::RecordedOnly`]，用例拿到它就收工。
///
/// # Panics
/// 项名不合规矩；库开不了、写不进、读不回。
pub fn facts_through_the_pairing_library(
    item_name: &str,
    facts: &FlowFacts,
) -> StageOutcome<FlowFactsReadBack> {
    let item =
        ItemName::new(item_name).unwrap_or_else(|refusal| panic!("对拍用例的项名：{refusal}"));
    let digest = facts.digest();
    two_stages(
        CrashLibraryClass::FactsPairing,
        &item,
        &RecordingInputsDigest(digest),
        |library| {
            library
                .lock()
                .store_flow_facts(&digest, &facts.to_bytes())
                .map_err(|error| format!("事实表写不进库：{error:?}"))?;
            Ok(ItemManifest {
                kind: "facts_pairing".to_string(),
                digests: vec![(PAIRING_MANIFEST_FACTS.to_string(), digest)],
            })
        },
        |library, manifest| {
            Ok(StageOutcome::Judged(read_flow_facts_back(
                &library.lock(),
                &manifest.digest_named(PAIRING_MANIFEST_FACTS)?,
            )?))
        },
    )
    .unwrap_or_else(|refusal| panic!("对拍用例「{item_name}」的事实表过库失败：{refusal}"))
}

#[cfg(feature = "gpu")]
/// 对拍用例的判表清单里判表那一样的名字。
const PAIRING_MANIFEST_JUDGE_TABLES: &str = "judge_tables";

#[cfg(feature = "gpu")]
/// GPU 判器与 CPU 判器对拍用例用：录入段把打包判表按摘要写进库、按录入键（判表摘要）登清单；判段照同一份判表现算录入键、读清单、
/// 按摘要读回，GPU 判器只收读回的那一份。只录那一趟交回 [`StageOutcome::RecordedOnly`]，用例拿到它就收工。
///
/// # Panics
/// 项名不合规矩；库开不了、写不进、读不回。
pub fn judge_tables_through_the_pairing_library(
    item_name: &str,
    tables: &PackedJudgeTables,
) -> StageOutcome<PackedJudgeTablesReadBack> {
    let item =
        ItemName::new(item_name).unwrap_or_else(|refusal| panic!("对拍用例的项名：{refusal}"));
    let bytes = tables.to_bytes();
    let digest = sha256_digest(&bytes);
    two_stages(
        CrashLibraryClass::JudgeTablesPairing,
        &item,
        &RecordingInputsDigest(digest),
        |library| {
            library
                .lock()
                .store_packed_judge_tables(&digest, &bytes)
                .map_err(|error| format!("GPU 判表写不进库：{error:?}"))?;
            Ok(ItemManifest {
                kind: "judge_tables_pairing".to_string(),
                digests: vec![(PAIRING_MANIFEST_JUDGE_TABLES.to_string(), digest)],
            })
        },
        |library, manifest| {
            Ok(StageOutcome::Judged(read_packed_judge_tables_back(
                &library.lock(),
                &manifest.digest_named(PAIRING_MANIFEST_JUDGE_TABLES)?,
            )?))
        },
    )
    .unwrap_or_else(|refusal| panic!("对拍用例「{item_name}」的判表过库失败：{refusal}"))
}

/// 一条对拍结论是从哪来的。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PairingConclusionSource {
    /// 库里同一键下存着，这一趟没判。
    ReusedFromTheLibrary,
    /// 这一趟判的（库里没有、或两边哪一边拒算），判完按键存进库（拒算的不存）。
    JudgedNow,
}

/// 判一次对拍交回的：结论的字节（写法由调用方定），与两边比下来一致还是不一致（调用方照它自己的比法判，[`PairingAgreement`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PairingJudged {
    pub conclusion: Vec<u8>,
    pub agreement: PairingAgreement,
}

/// 库里这一把键下存着的对拍结论能不能复用：两边一致的复用；记着不一致（红）的不复用、这一趟现判（照同一把键记红、撤掉原有的绿，
/// 键记着红的下一趟现判、不当绿复用：`.claude/rules/verification.md`「一个门禁：两种跑法」那一条）。
fn stored_pairing_conclusion_is_reused(stored: &StoredPairingConclusion) -> bool {
    match stored.agreement {
        PairingAgreement::BothSidesAgree => true,
        PairingAgreement::SidesDisagree => false,
    }
}

/// 对拍结论入库（「对拍结论入库」）：键是（输入摘要，两边判法单元指纹），库里有两边一致的就复用、不再判；
/// 没有、或记着不一致（红）的，调 `judge` 判一遍、照同一把键存进库（红的照样存：记红，同一把键原有的那一行被换掉）。
/// 两边哪一边的判法单元拒算：照判、不入库（不退回整文件指纹、不跳过）。交回结论的字节与它从哪来。
///
/// # Errors
/// 库读写失败。
pub fn pairing_conclusion_reused_or_judged(
    library: &CrashLibrary,
    inputs_digest: &[u8; DIGEST_BYTES],
    one_side: &Result<JudgingUnitsKey, String>,
    other_side: &Result<JudgingUnitsKey, String>,
    judge: impl FnOnce() -> PairingJudged,
) -> Result<(Vec<u8>, PairingConclusionSource), String> {
    let (Ok(one_side), Ok(other_side)) = (one_side, other_side) else {
        println!(
            "CRASH_LIBRARY_PAIRING_KEY_REFUSED class={} inputs={}：判了、不入库",
            library.class.name(),
            hexadecimal_text(&inputs_digest[..8])
        );
        return Ok((judge().conclusion, PairingConclusionSource::JudgedNow));
    };
    let key = PairingConclusionKey {
        inputs_digest: *inputs_digest,
        one_side_judging_units: *one_side.fingerprint(),
        other_side_judging_units: *other_side.fingerprint(),
    };
    if let Some(stored) = library
        .lock()
        .read_pairing_conclusion(&key)
        .map_err(|error| format!("读对拍结论失败：{error:?}"))?
    {
        if stored_pairing_conclusion_is_reused(&stored) {
            return Ok((
                stored.conclusion,
                PairingConclusionSource::ReusedFromTheLibrary,
            ));
        }
        println!(
            "CRASH_LIBRARY_PAIRING_STORED_RED class={} inputs={}：库里这一把键记着两边不一致，这一趟现判",
            library.class.name(),
            hexadecimal_text(&inputs_digest[..8])
        );
    }
    let judged = judge();
    {
        let mut store = library.lock();
        for side in [one_side, other_side] {
            store
                .store_judging_units_composition(side)
                .map_err(|error| format!("组成清单写不进库：{error:?}"))?;
        }
        store
            .store_pairing_conclusion(
                &key,
                &StoredPairingConclusion {
                    conclusion: judged.conclusion.clone(),
                    agreement: judged.agreement,
                    circumstances: JudgementCircumstances::of_this_process_now(),
                },
            )
            .map_err(|error| format!("对拍结论写不进库：{error:?}"))?;
    }
    Ok((judged.conclusion, PairingConclusionSource::JudgedNow))
}

// ============================== 对拍结论按块：块键、按块复用或现判 ==============================

/// 一条流逐状态对拍时一块几个状态：与崩溃放量同一个块宽（[`crate::crash_amplification::STATES_PER_BLOCK`]）。
pub const STATES_PER_PAIRING_BLOCK: u64 = crate::crash_amplification::STATES_PER_BLOCK;

/// 一块的输入摘要（块键里内容那一半）：整条流那一份对拍输入摘要（调用方只按内容算：基线镜像、写表、切段与展开、版本表、判表、比法那一组的键
/// 这类）拼上这一块的序号段 `[start, end)`，取 SHA-256。进库时与两边判法单元指纹一起组成 [`PairingConclusionKey`]，存在配对结论那一张表里
/// （`verdict_store` 文件头登记的表，不另开表、不另开键空间）：块的内容与两边判器组的单元键都相同的下一趟复用。
#[must_use]
pub fn pairing_block_inputs_digest(
    flow_inputs_digest: &[u8; DIGEST_BYTES],
    ordinals_of_the_block: &std::ops::Range<u64>,
) -> [u8; DIGEST_BYTES] {
    let mut bytes = b"crash judge pairing block\n".to_vec();
    bytes.extend_from_slice(flow_inputs_digest);
    bytes.extend_from_slice(&ordinals_of_the_block.start.to_le_bytes());
    bytes.extend_from_slice(&ordinals_of_the_block.end.to_le_bytes());
    sha256_digest(&bytes)
}

/// 一条流按块对拍下来几块复用、几块现判，与同一时刻手里最多压着几块的结论（宿主内存只随它涨，不随整条流的块数涨）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PairingBlocksOfTheFlow {
    pub blocks_reused: u64,
    pub blocks_judged_now: u64,
    pub most_block_conclusions_held_at_once: usize,
}

/// 判一段连着的块：交这一段里每块的序号段，按同一次序交回每块的结论字节与两边一致不一致；交 `None` 是判那一侧报「本次未跑」。
pub type JudgeOfAStretchOfPairingBlocks<'closure> =
    dyn FnMut(&[std::ops::Range<u64>]) -> Option<Vec<PairingJudged>> + 'closure;

/// 攒着的那一段块交出去判之后怎样了。
enum PendingPairingBlocks {
    /// 判完、逐块存进库（拒算的不存）、逐块折进了总账；没攒着块也算这一种。
    JudgedAndTaken,
    /// 判那一侧报「本次未跑」：一块都没存。
    NotRun,
}

/// 按块对拍（「必须要剪枝和复用」）：`[0, state_count)` 按 `states_per_block` 切块，按序号次序一块块走——库里有这一块
/// 两边一致的结论（键是 [`pairing_block_inputs_digest`] 与两边判法单元指纹）就复用、不判；记着不一致（红）的与没有的一样现判、照同一把键
/// 存回（红的照样存：记红、换掉原有的那一行）。没有的攒成一段连着的块（至多 `most_blocks_judged_together` 块），
/// 交 `judge_the_blocks` 一次判完、逐块交回结论字节，逐块存进库。每一块的结论（复用的与现判的）按序号次序交 `take_a_block_conclusion`
/// 折进调用方的总账，交完就丢：手里同一时刻至多压着 `most_blocks_judged_together` 块的结论，与整条流有几块无关。
/// 两边哪一边的判法单元拒算：照判、不入库（不退回整文件指纹、不跳过）。`judge_the_blocks` 交 `None`（判那一侧报「本次未跑」）：当场收手、
/// 交 `Ok(None)`，那一段一块都不存。
///
/// # Errors
/// 库读写失败。
///
/// # Panics
/// `judge_the_blocks` 交回的结论块数与交给它的块数不同。
#[allow(
    clippy::too_many_arguments,
    reason = "按块对拍的输入就是这几样：库、流的输入摘要、状态数、块宽、一次判几块、两边判法单元、判一段块、收一块结论；收成一个类型只为让签名变短"
)]
pub fn pairing_conclusions_of_the_blocks_reused_or_judged(
    library: &CrashLibrary,
    flow_inputs_digest: &[u8; DIGEST_BYTES],
    state_count: u64,
    states_per_block: std::num::NonZeroU64,
    most_blocks_judged_together: std::num::NonZeroUsize,
    one_side: &Result<JudgingUnitsKey, String>,
    other_side: &Result<JudgingUnitsKey, String>,
    judge_the_blocks: &mut JudgeOfAStretchOfPairingBlocks<'_>,
    take_a_block_conclusion: &mut dyn FnMut(&std::ops::Range<u64>, &[u8]),
) -> Result<Option<PairingBlocksOfTheFlow>, String> {
    let both_sides = match (one_side, other_side) {
        (Ok(one_side_key), Ok(other_side_key)) => {
            let mut store = library.lock();
            for side in [one_side_key, other_side_key] {
                store
                    .store_judging_units_composition(side)
                    .map_err(|error| format!("组成清单写不进库：{error:?}"))?;
            }
            Some((one_side_key, other_side_key))
        }
        (Err(_), _) | (_, Err(_)) => {
            println!(
                "CRASH_LIBRARY_PAIRING_KEY_REFUSED class={} inputs={}：按块判了、不入库",
                library.class.name(),
                hexadecimal_text(&flow_inputs_digest[..8])
            );
            None
        }
    };
    let key_of_the_block = |ordinals: &std::ops::Range<u64>| {
        both_sides.map(|(one_side_key, other_side_key)| PairingConclusionKey {
            inputs_digest: pairing_block_inputs_digest(flow_inputs_digest, ordinals),
            one_side_judging_units: *one_side_key.fingerprint(),
            other_side_judging_units: *other_side_key.fingerprint(),
        })
    };
    let mut blocks_of_the_flow = PairingBlocksOfTheFlow {
        blocks_reused: 0,
        blocks_judged_now: 0,
        most_block_conclusions_held_at_once: 0,
    };
    let mut pending_blocks: Vec<std::ops::Range<u64>> =
        Vec::with_capacity(most_blocks_judged_together.get());
    let mut block_start = 0u64;
    while block_start < state_count {
        let ordinals = block_start
            ..block_start
                .saturating_add(states_per_block.get())
                .min(state_count);
        block_start = ordinals.end;
        let stored_conclusion = match key_of_the_block(&ordinals) {
            Some(key) => library
                .lock()
                .read_pairing_conclusion(&key)
                .map_err(|error| format!("读对拍块结论失败：{error:?}"))?
                .filter(stored_pairing_conclusion_is_reused),
            None => None,
        };
        if let Some(stored) = stored_conclusion {
            // 复用的这一块之前攒着的先判完、折进总账：总账按序号次序收
            match judge_and_take_the_pending_pairing_blocks(
                library,
                &mut pending_blocks,
                &key_of_the_block,
                judge_the_blocks,
                take_a_block_conclusion,
                &mut blocks_of_the_flow,
            )? {
                PendingPairingBlocks::JudgedAndTaken => {}
                PendingPairingBlocks::NotRun => return Ok(None),
            }
            take_a_block_conclusion(&ordinals, &stored.conclusion);
            blocks_of_the_flow.blocks_reused += 1;
            continue;
        }
        pending_blocks.push(ordinals);
        if pending_blocks.len() >= most_blocks_judged_together.get() {
            match judge_and_take_the_pending_pairing_blocks(
                library,
                &mut pending_blocks,
                &key_of_the_block,
                judge_the_blocks,
                take_a_block_conclusion,
                &mut blocks_of_the_flow,
            )? {
                PendingPairingBlocks::JudgedAndTaken => {}
                PendingPairingBlocks::NotRun => return Ok(None),
            }
        }
    }
    match judge_and_take_the_pending_pairing_blocks(
        library,
        &mut pending_blocks,
        &key_of_the_block,
        judge_the_blocks,
        take_a_block_conclusion,
        &mut blocks_of_the_flow,
    )? {
        PendingPairingBlocks::JudgedAndTaken => Ok(Some(blocks_of_the_flow)),
        PendingPairingBlocks::NotRun => Ok(None),
    }
}

/// 把攒着的那一段连着的块交出去一次判完：逐块存进库（键算不出的，即两边哪一边拒算，不存），逐块按序号次序折进总账，清空攒着的那一段。
///
/// # Errors
/// 库写失败。
///
/// # Panics
/// 交回的结论块数与交出去的块数不同。
fn judge_and_take_the_pending_pairing_blocks(
    library: &CrashLibrary,
    pending_blocks: &mut Vec<std::ops::Range<u64>>,
    key_of_the_block: &dyn Fn(&std::ops::Range<u64>) -> Option<PairingConclusionKey>,
    judge_the_blocks: &mut JudgeOfAStretchOfPairingBlocks<'_>,
    take_a_block_conclusion: &mut dyn FnMut(&std::ops::Range<u64>, &[u8]),
    blocks_of_the_flow: &mut PairingBlocksOfTheFlow,
) -> Result<PendingPairingBlocks, String> {
    if pending_blocks.is_empty() {
        return Ok(PendingPairingBlocks::JudgedAndTaken);
    }
    let Some(conclusions) = judge_the_blocks(pending_blocks) else {
        return Ok(PendingPairingBlocks::NotRun);
    };
    assert_eq!(
        conclusions.len(),
        pending_blocks.len(),
        "判一段块交回的结论块数与交出去的块数相同"
    );
    blocks_of_the_flow.most_block_conclusions_held_at_once = blocks_of_the_flow
        .most_block_conclusions_held_at_once
        .max(conclusions.len());
    for (ordinals, judged) in pending_blocks.iter().zip(&conclusions) {
        if let Some(key) = key_of_the_block(ordinals) {
            library
                .lock()
                .store_pairing_conclusion(
                    &key,
                    &StoredPairingConclusion {
                        conclusion: judged.conclusion.clone(),
                        agreement: judged.agreement,
                        circumstances: JudgementCircumstances::of_this_process_now(),
                    },
                )
                .map_err(|error| format!("对拍块结论写不进库：{error:?}"))?;
        }
        take_a_block_conclusion(ordinals, &judged.conclusion);
        blocks_of_the_flow.blocks_judged_now += 1;
    }
    pending_blocks.clear();
    Ok(PendingPairingBlocks::JudgedAndTaken)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crash_identity::{ConclusionKind, ConclusionVersion, JudgingUnitFingerprint};
    use crate::verdict_store::{
        BlockCountVector, BlockStartStateIndex, ConclusionBlockRecord, EncodedVerdictVector,
        EnumerationPlanHash, JudgingDevice, NodeInputDigest, VerdictBlock, VerdictBlockKey,
    };

    /// 标签里带 `/`（场景名、随机历史名这类）：照样建在临时目录下一层、`/` 换成 `-`，不当成多级路径（排他新建不递归，上一级不在就建不出）。
    /// 判别力（`crates/mutations.tsv`）：拼目录名时不换标签里的 `/`，这条红（ENOENT）。
    #[test]
    fn a_label_with_slashes_is_created_one_level_under_the_temporary_directory() {
        let directory = ExclusiveTemporaryDirectory::create("scenario/with/slashes")
            .expect("带 / 的标签也建得出");
        assert_eq!(
            directory.path().parent(),
            Some(std::env::temp_dir().as_path()),
            "建在临时目录下一层"
        );
        assert!(
            directory
                .path()
                .file_name()
                .expect("目录有名字")
                .to_string_lossy()
                .starts_with("singlefs-scenario-with-slashes-"),
            "标签里的 / 换成 -：{}",
            directory.path().display()
        );
    }

    /// 同一个进程里两次用同一个标签建一次性临时目录：各拿一份不同的、新建的空目录；丢掉一份只删那一份，另一份连同里面的文件还在。
    /// 判别力（`crates/mutations.tsv`）：名字退回只由标签与进程号定，第二次建撞上头一份、试满名字建不出，这条红。
    #[test]
    fn two_exclusive_temporary_directories_of_one_label_in_one_process_differ_and_dropping_one_keeps_the_other(
    ) {
        const LABEL: &str = "exclusive-temporary-directory-same-label";
        let one = ExclusiveTemporaryDirectory::create(LABEL).expect("头一份建得出");
        let another = ExclusiveTemporaryDirectory::create(LABEL)
            .expect("同一个标签的第二份也建得出：名字里带进程内序号");
        assert_ne!(
            one.path(),
            another.path(),
            "同一进程、同一标签的两份是两个目录"
        );
        for directory in [&one, &another] {
            assert!(
                std::fs::read_dir(directory.path())
                    .expect("建好的目录读得出")
                    .next()
                    .is_none(),
                "{} 是新建的空目录",
                directory.path().display()
            );
        }
        let file_of_the_other = another.join("written-by-the-other-holder");
        std::fs::write(&file_of_the_other, b"kept").expect("另一份里写得进文件");
        let path_of_the_dropped = one.path().to_path_buf();
        drop(one);
        assert!(!path_of_the_dropped.exists(), "丢掉的那一份连目录一起删掉");
        assert_eq!(
            std::fs::read(&file_of_the_other).expect("另一份里的文件还在"),
            b"kept",
            "丢掉一份不碰另一份"
        );
    }

    #[test]
    fn an_item_manifest_round_trips_through_its_bytes() {
        let manifest = ItemManifest {
            kind: "crash_state_enumeration".to_string(),
            digests: vec![
                ("flow".to_string(), [1; DIGEST_BYTES]),
                ("tables".to_string(), [2; DIGEST_BYTES]),
            ],
        };
        let bytes = manifest.to_bytes();
        assert_eq!(ItemManifest::from_bytes(&bytes), Some(manifest.clone()));
        assert_eq!(manifest.digest_named("tables"), Ok([2; DIGEST_BYTES]));
        assert!(
            manifest.digest_named("facts").is_err(),
            "清单里没有的那一样要报错"
        );
        let mut truncated = bytes.clone();
        truncated.pop();
        assert_eq!(ItemManifest::from_bytes(&truncated), None, "字节不够解不出");
        let mut extended = bytes.clone();
        extended.push(0);
        assert_eq!(ItemManifest::from_bytes(&extended), None, "字节多出解不出");
        let mut wrong_magic = bytes;
        wrong_magic[0] ^= 0x01;
        assert_eq!(ItemManifest::from_bytes(&wrong_magic), None, "头不对解不出");
    }

    #[test]
    fn an_item_name_only_takes_the_allowed_characters() {
        assert!(ItemName::new("tree_split::central-mapping.root/split").is_ok());
        assert!(ItemName::new("").is_err());
        assert!(ItemName::new("有空格 的").is_err());
        assert!(ItemName::new(&"a".repeat(201)).is_err());
    }

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

    fn a_block_key() -> VerdictBlockKey {
        VerdictBlockKey {
            node_input_digest: NodeInputDigest([3; DIGEST_BYTES]),
            enumeration_plan_hash: EnumerationPlanHash([4; DIGEST_BYTES]),
            block_start: BlockStartStateIndex(0),
        }
    }

    fn one_clean_state(store: &mut VerdictStore) -> ConclusionBlockRecord {
        let clean = store
            .register_verdict_vector(&EncodedVerdictVector(Vec::new()))
            .expect("登记得了判定向量");
        ConclusionBlockRecord {
            block: VerdictBlock::new(vec![clean]).expect("一块一个状态"),
            violating_states: Vec::new(),
            count_vector: BlockCountVector::default(),
            judged_on: JudgingDevice::Cpu,
            recovery_words: None,
        }
    }

    /// 一份库、类当索引：几类测试开的是这个进程的同一份（同一个进程里共用一份连接），库的名字不带类名；
    /// 同一块的 GPU 与 CPU 两种结论在这一份库里是同一块下的两条记录、各自读得回。只对临时服务里的一次性库跑。
    /// 判别力：`LibraryOpenInThisProcess::open_or_share` 不看这个进程设的一次性库（照开全仓那一份），这条红在列结论那一步（读到别处的行或连不上）。
    #[test]
    fn every_class_opens_the_one_library_and_two_conclusion_kinds_of_one_block_are_two_records() {
        // 一次性库留着：自证一份库、类当索引：要一份空的临时服务
        let disposable =
            DisposableLibrary::new("one-library").unwrap_or_else(|refusal| panic!("{refusal}"));
        let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
        let amplification =
            CrashLibrary::open(CrashLibraryClass::CrashAmplification).expect("开得了库");
        let enumeration =
            CrashLibrary::open(CrashLibraryClass::CrashStateEnumeration).expect("开得了库");
        let given_states = CrashLibrary::open(CrashLibraryClass::GivenStates).expect("开得了库");
        assert_eq!(amplification.library_name(), enumeration.library_name());
        assert_eq!(amplification.library_name(), given_states.library_name());
        for class in [
            CrashLibraryClass::CrashAmplification,
            CrashLibraryClass::CrashStateEnumeration,
            CrashLibraryClass::GivenStates,
            CrashLibraryClass::JudgeTablesPairing,
            CrashLibraryClass::FactsPairing,
        ] {
            assert!(
                !amplification.library_name().contains(class.name()),
                "库的名字不带类名 {}",
                class.name()
            );
        }
        let key = a_block_key();
        let gpu_version = ConclusionVersion([5; DIGEST_BYTES]);
        let cpu_version = ConclusionVersion([6; DIGEST_BYTES]);
        {
            let mut store = amplification.lock();
            let record = one_clean_state(&mut store);
            store
                .store_conclusion_block(
                    &key,
                    &ConclusionKind::ImageJudgement,
                    &gpu_version,
                    &record,
                )
                .expect("存 GPU 结论");
        }
        {
            let mut store = enumeration.lock();
            let record = one_clean_state(&mut store);
            store
                .store_conclusion_block(
                    &key,
                    &ConclusionKind::ImageJudgement,
                    &cpu_version,
                    &record,
                )
                .expect("存 CPU 结论");
        }
        let store = given_states.lock();
        assert_eq!(
            store.stored_conclusion_keys().expect("列结论"),
            vec![
                (key.clone(), ConclusionKind::ImageJudgement, gpu_version),
                (key.clone(), ConclusionKind::ImageJudgement, cpu_version),
            ],
            "同一批状态只存一处：一块两种结论两条记录"
        );
    }

    /// 录入段按（录入函数单元指纹，录入输入摘要）入库（L11）：同一份输入第二趟命中、录入那一段不再跑；输入改一个字节第二趟照录；
    /// 录入函数单元拒算时不复用、照录。
    #[test]
    fn the_recording_is_skipped_on_a_hit_and_redone_when_one_input_byte_changes() {
        // 一次性库留着：自证录入键命中与不中：要从空库数录了几次
        let disposable =
            DisposableLibrary::new("recording-reuse").unwrap_or_else(|refusal| panic!("{refusal}"));
        let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
        let library =
            CrashLibrary::open(CrashLibraryClass::CrashStateEnumeration).expect("开得了库");
        let units = Ok(JudgingUnitsKey::of_the_parts(
            [7; DIGEST_BYTES],
            vec![JudgingUnitFingerprint {
                unit: "record_into".to_string(),
                fingerprint: [8; DIGEST_BYTES],
            }],
        ));
        let item = ItemName::new("recording-reuse").expect("项名合规矩");
        let recordings = std::cell::Cell::new(0u32);
        let run_with_the_inputs =
            |inputs: [u8; DIGEST_BYTES], units_of_the_run: &Result<JudgingUnitsKey, String>| {
                two_stages_under_the_recording_key(
                    &library,
                    RunStage::RecordThenJudge,
                    &item,
                    &recording_key_of_the_inputs_under(
                        units_of_the_run,
                        &RecordingInputsDigest(inputs),
                    ),
                    |_library| {
                        recordings.set(recordings.get() + 1);
                        Ok(ItemManifest {
                            kind: "probe".to_string(),
                            digests: vec![("inputs".to_string(), inputs)],
                        })
                    },
                    |_library, manifest| Ok(StageOutcome::Judged(manifest.digest_named("inputs")?)),
                )
                .expect("两段过得了库")
                .judged()
                .expect("两段先后跑交回判的结果")
            };
        let mut inputs = [9u8; DIGEST_BYTES];
        assert_eq!(run_with_the_inputs(inputs, &units), inputs);
        assert_eq!(recordings.get(), 1, "头一趟现录");
        assert_eq!(run_with_the_inputs(inputs, &units), inputs);
        assert_eq!(recordings.get(), 1, "同一份输入第二趟命中、不再录");
        inputs[0] ^= 1;
        assert_eq!(run_with_the_inputs(inputs, &units), inputs);
        assert_eq!(recordings.get(), 2, "输入改一个字节第二趟现录");
        let refused: Result<JudgingUnitsKey, String> = Err("拒算".to_string());
        run_with_the_inputs(inputs, &refused);
        run_with_the_inputs(inputs, &refused);
        assert_eq!(recordings.get(), 4, "录入函数单元拒算时每趟照录、不复用");
    }

    /// 对拍结论入库（L10）：同一份输入、同两边判法单元第二趟复用、不再判；换了一边的判法单元照判；哪一边拒算照判、不入库。
    #[test]
    fn a_pairing_conclusion_is_reused_the_second_time_and_judged_again_when_a_side_changes() {
        // 一次性库留着：自证对拍结论复用与重判：要从空库数判了几次
        let disposable =
            DisposableLibrary::new("pairing-reuse").unwrap_or_else(|refusal| panic!("{refusal}"));
        let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
        let library = CrashLibrary::open(CrashLibraryClass::JudgeTablesPairing).expect("开得了库");
        let side = |byte: u8| -> Result<JudgingUnitsKey, String> {
            Ok(JudgingUnitsKey::of_the_parts(
                [byte; DIGEST_BYTES],
                Vec::new(),
            ))
        };
        let judgements = std::cell::Cell::new(0u32);
        let judge = || {
            judgements.set(judgements.get() + 1);
            PairingJudged {
                conclusion: format!("agree {}", judgements.get()).into_bytes(),
                agreement: PairingAgreement::BothSidesAgree,
            }
        };
        let inputs = [1; DIGEST_BYTES];
        let first =
            pairing_conclusion_reused_or_judged(&library, &inputs, &side(2), &side(3), judge)
                .expect("对拍得了");
        assert_eq!(
            first,
            (b"agree 1".to_vec(), PairingConclusionSource::JudgedNow)
        );
        let second =
            pairing_conclusion_reused_or_judged(&library, &inputs, &side(2), &side(3), judge)
                .expect("对拍得了");
        assert_eq!(
            second,
            (
                b"agree 1".to_vec(),
                PairingConclusionSource::ReusedFromTheLibrary
            ),
            "第二趟复用库里那一条"
        );
        assert_eq!(judgements.get(), 1, "复用的那一趟不判");
        let other_side =
            pairing_conclusion_reused_or_judged(&library, &inputs, &side(2), &side(4), judge)
                .expect("对拍得了");
        assert_eq!(
            other_side.1,
            PairingConclusionSource::JudgedNow,
            "换了一边的判法单元照判"
        );
        let refused: Result<JudgingUnitsKey, String> = Err("拒算".to_string());
        for _ in 0..2 {
            let judged =
                pairing_conclusion_reused_or_judged(&library, &inputs, &refused, &side(3), judge)
                    .expect("对拍得了");
            assert_eq!(
                judged.1,
                PairingConclusionSource::JudgedNow,
                "拒算照判、不入库"
            );
        }
        assert_eq!(judgements.get(), 4);
    }

    /// 场景：同一份输入、同两边判法单元，头一趟判出两边不一致（红），第二趟起判出一致；连跑三趟。
    /// 预期：头一趟照这一把键记红（库里那一行是不一致的那一份）；第二趟不复用记着红的那一份、现判，判出一致的那一份换掉它；
    /// 第三趟复用一致的那一份、不判（照同一把键记红、撤掉原有的绿，键记着红的下一趟现判、不当绿复用）。
    /// 判别力（`crates/mutations.tsv`）：「对拍结论：记着红的照复用」「对拍结论：单条那一路问复用不看红绿」两行各让这一条红。
    #[test]
    fn a_pairing_conclusion_stored_red_is_judged_again_and_the_agreeing_one_replaces_it() {
        // 一次性库留着：要从空库数判了几次、读库里那一行记的红绿
        let disposable = DisposableLibrary::new("pairing-stored-red")
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
        let library = CrashLibrary::open(CrashLibraryClass::JudgeTablesPairing).expect("开得了库");
        let one_side = judging_units_key_of_these_units(&[("gpu_judge_unit", 1)]);
        let other_side = judging_units_key_of_these_units(&[("cpu_judge_unit", 2)]);
        let inputs = [8; DIGEST_BYTES];
        let key = PairingConclusionKey {
            inputs_digest: inputs,
            one_side_judging_units: *one_side.as_ref().expect("造的键").fingerprint(),
            other_side_judging_units: *other_side.as_ref().expect("造的键").fingerprint(),
        };
        let agreement_stored = || {
            library
                .lock()
                .read_pairing_conclusion(&key)
                .expect("读得了库")
                .map(|stored| stored.agreement)
        };
        let judgements = std::cell::Cell::new(0u32);
        let judge = || {
            judgements.set(judgements.get() + 1);
            PairingJudged {
                conclusion: format!("judged {}", judgements.get()).into_bytes(),
                agreement: if judgements.get() == 1 {
                    PairingAgreement::SidesDisagree
                } else {
                    PairingAgreement::BothSidesAgree
                },
            }
        };
        let first =
            pairing_conclusion_reused_or_judged(&library, &inputs, &one_side, &other_side, judge)
                .expect("对拍得了");
        assert_eq!(
            first,
            (b"judged 1".to_vec(), PairingConclusionSource::JudgedNow)
        );
        assert_eq!(
            agreement_stored(),
            Some(PairingAgreement::SidesDisagree),
            "头一趟判红：照这一把键记红"
        );
        let second =
            pairing_conclusion_reused_or_judged(&library, &inputs, &one_side, &other_side, judge)
                .expect("对拍得了");
        assert_eq!(
            second,
            (b"judged 2".to_vec(), PairingConclusionSource::JudgedNow),
            "库里记着红的不复用：第二趟现判"
        );
        assert_eq!(
            agreement_stored(),
            Some(PairingAgreement::BothSidesAgree),
            "现判出一致的那一份换掉记着红的那一份"
        );
        let third =
            pairing_conclusion_reused_or_judged(&library, &inputs, &one_side, &other_side, judge)
                .expect("对拍得了");
        assert_eq!(
            third,
            (
                b"judged 2".to_vec(),
                PairingConclusionSource::ReusedFromTheLibrary
            ),
            "一致的那一份照复用"
        );
        assert_eq!(judgements.get(), 2, "复用的那一趟不判");
    }

    /// 场景：按块对拍同一条流（9 块），头一趟第二块（序号 [4, 8)）判出两边不一致（红）、别的块一致；第二趟起每块都判出一致；连跑三趟。
    /// 预期：第二趟只现判记着红的那一块（判段只叫一次、只交那一块），别的 8 块复用；第三趟 9 块都复用、现判 0。
    /// 判别力（`crates/mutations.tsv`）：「对拍结论：记着红的照复用」「对拍结论：按块那一路问复用不看红绿」两行各让这一条红。
    #[test]
    fn a_pairing_block_stored_red_is_the_only_block_judged_again_the_next_time() {
        // 一次性库留着：要从空库起数现判的块
        let disposable = DisposableLibrary::new("pairing-block-stored-red")
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
        let library = CrashLibrary::open(CrashLibraryClass::JudgeTablesPairing).expect("开得了库");
        let units = judging_units_key_of_these_units(&[("gpu_judge_unit", 1)]);
        let other_units = judging_units_key_of_these_units(&[("cpu_judge_unit", 2)]);
        let flow_inputs_digest = [9u8; DIGEST_BYTES];
        let red_block = 4..8;
        let first_run = pairing_blocks_run_with_the_agreement(
            &library,
            &flow_inputs_digest,
            4 * 9,
            2,
            &units,
            &other_units,
            &|ordinals| {
                if *ordinals == red_block {
                    PairingAgreement::SidesDisagree
                } else {
                    PairingAgreement::BothSidesAgree
                }
            },
        );
        assert_eq!(
            first_run.blocks_of_the_flow.blocks_judged_now, 9,
            "前提：头一趟 9 块都现判"
        );
        let second_run = pairing_blocks_run(
            &library,
            &flow_inputs_digest,
            4 * 9,
            2,
            &units,
            &other_units,
        );
        assert_eq!(
            (
                second_run.blocks_of_the_flow.blocks_judged_now,
                second_run.blocks_of_the_flow.blocks_reused
            ),
            (1, 8),
            "记着红的那一块现判，别的 8 块复用"
        );
        assert_eq!(
            second_run.blocks_asked_of_the_judge_each_time,
            vec![1],
            "判段只叫一次、只交记着红的那一块"
        );
        let third_run = pairing_blocks_run(
            &library,
            &flow_inputs_digest,
            4 * 9,
            2,
            &units,
            &other_units,
        );
        assert_eq!(
            (
                third_run.blocks_of_the_flow.blocks_judged_now,
                third_run.blocks_of_the_flow.blocks_reused
            ),
            (0, 9),
            "现判出一致的那一份换掉了红的：第三趟 9 块都复用"
        );
    }

    /// 测试里造的一边判器组的单元键：按单元名排好的（名、指纹）逐个拼起来取 SHA-256 当指纹（真键由构建脚本按入口收闭包算，这里只要
    /// 「组里哪个单元的指纹变了，这一边的键就变」这一条）。
    fn judging_units_key_of_these_units(units: &[(&str, u8)]) -> Result<JudgingUnitsKey, String> {
        let mut sorted_units = units.to_vec();
        sorted_units.sort_unstable();
        let mut bytes = Vec::new();
        for (unit_name, unit_fingerprint) in sorted_units {
            bytes.extend_from_slice(unit_name.as_bytes());
            bytes.push(b'\t');
            bytes.push(unit_fingerprint);
            bytes.push(b'\n');
        }
        Ok(JudgingUnitsKey::of_the_parts(
            sha256_digest(&bytes),
            Vec::new(),
        ))
    }

    /// 按块对拍一条流的一趟：交回几块复用几块现判、判段被叫了几次、每次交来几块，与按序号次序收到的每块结论。
    struct PairingBlocksRun {
        blocks_of_the_flow: PairingBlocksOfTheFlow,
        blocks_asked_of_the_judge_each_time: Vec<usize>,
        conclusions_in_order: Vec<(std::ops::Range<u64>, Vec<u8>)>,
    }

    /// 在 `library` 上按块对拍一条流：判段给每块的结论是「判<起点>-<终点>」、两边都一致，收到的结论按次序记下。
    fn pairing_blocks_run(
        library: &CrashLibrary,
        flow_inputs_digest: &[u8; DIGEST_BYTES],
        state_count: u64,
        most_blocks_judged_together: usize,
        one_side: &Result<JudgingUnitsKey, String>,
        other_side: &Result<JudgingUnitsKey, String>,
    ) -> PairingBlocksRun {
        pairing_blocks_run_with_the_agreement(
            library,
            flow_inputs_digest,
            state_count,
            most_blocks_judged_together,
            one_side,
            other_side,
            &|_ordinals| PairingAgreement::BothSidesAgree,
        )
    }

    /// 同 [`pairing_blocks_run`]，判段给每块的两边一致不一致照 `agreement_of_the_block`。
    fn pairing_blocks_run_with_the_agreement(
        library: &CrashLibrary,
        flow_inputs_digest: &[u8; DIGEST_BYTES],
        state_count: u64,
        most_blocks_judged_together: usize,
        one_side: &Result<JudgingUnitsKey, String>,
        other_side: &Result<JudgingUnitsKey, String>,
        agreement_of_the_block: &dyn Fn(&std::ops::Range<u64>) -> PairingAgreement,
    ) -> PairingBlocksRun {
        let mut blocks_asked_of_the_judge_each_time = Vec::new();
        let mut conclusions_in_order = Vec::new();
        let blocks_of_the_flow = pairing_conclusions_of_the_blocks_reused_or_judged(
            library,
            flow_inputs_digest,
            state_count,
            std::num::NonZeroU64::new(4).expect("块宽 4 不是 0"),
            std::num::NonZeroUsize::new(most_blocks_judged_together).expect("一次判的块数不是 0"),
            one_side,
            other_side,
            &mut |blocks| {
                blocks_asked_of_the_judge_each_time.push(blocks.len());
                Some(
                    blocks
                        .iter()
                        .map(|ordinals| PairingJudged {
                            conclusion: format!("判{}-{}", ordinals.start, ordinals.end)
                                .into_bytes(),
                            agreement: agreement_of_the_block(ordinals),
                        })
                        .collect(),
                )
            },
            &mut |ordinals, conclusion| {
                conclusions_in_order.push((ordinals.clone(), conclusion.to_vec()));
            },
        )
        .expect("库读写得了")
        .expect("判段没报本次未跑");
        PairingBlocksRun {
            blocks_of_the_flow,
            blocks_asked_of_the_judge_each_time,
            conclusions_in_order,
        }
    }

    /// 宿主内存随块数不涨：同一个一次判几块，5 块的流与 50 块的流手里同一时刻压着的块结论一样多（都是一次判的块数），
    /// 判段每次交来的块都不超过它；每块的结论按序号次序各收一次。
    /// 判别力（`crates/mutations.tsv`）：攒着的块到了一次判的块数也不交出去判（整条流攒完才判），50 块那一条流压着 50 块，这条红。
    #[test]
    fn pairing_blocks_hold_as_many_conclusions_at_once_whatever_the_number_of_blocks_of_the_flow() {
        // 一次性库留着：要从空库起数现判的块
        let disposable = DisposableLibrary::new("pairing-blocks-held")
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
        let library = CrashLibrary::open(CrashLibraryClass::JudgeTablesPairing).expect("开得了库");
        let units = judging_units_key_of_these_units(&[("gpu_judge_unit", 1)]);
        let other_units = judging_units_key_of_these_units(&[("cpu_judge_unit", 2)]);
        for (flow_byte, blocks) in [(5u8, 5u64), (6u8, 50u64)] {
            let run = pairing_blocks_run(
                &library,
                &[flow_byte; DIGEST_BYTES],
                4 * blocks - 1,
                3,
                &units,
                &other_units,
            );
            assert_eq!(
                run.blocks_of_the_flow.most_block_conclusions_held_at_once, 3,
                "{blocks} 块的流：手里同一时刻至多压着一次判的 3 块结论，与块数无关"
            );
            assert!(
                run.blocks_asked_of_the_judge_each_time
                    .iter()
                    .all(|asked| *asked <= 3),
                "{blocks} 块的流：判段每次交来的块不超过 3 块：{:?}",
                run.blocks_asked_of_the_judge_each_time
            );
            let expected_order: Vec<std::ops::Range<u64>> = (0..blocks)
                .map(|block| 4 * block..(4 * block + 4).min(4 * blocks - 1))
                .collect();
            assert_eq!(
                run.conclusions_in_order
                    .iter()
                    .map(|(ordinals, _)| ordinals.clone())
                    .collect::<Vec<_>>(),
                expected_order,
                "{blocks} 块的流：每块的结论按序号次序各收一次，末块只到状态数"
            );
        }
    }

    /// 第二趟同一条流复用、现判 0：头一趟每块现判、一块都不复用（块键带序号段，别的块的结论不顶替这一块）；同一份输入、同两边判法单元的
    /// 第二趟每块复用、判段一次都不叫，收到的结论与头一趟逐块相同；哪一边拒算的照判、不入库。
    /// 判别力（`crates/mutations.tsv`）：读库那一处一律当库里没有，第二趟照判、这条红；块键不带序号段，头一趟后面的块复用了前面块的结论、
    /// 这条红。
    #[test]
    fn a_flow_paired_twice_by_blocks_reuses_every_block_and_judges_none_the_second_time() {
        // 一次性库留着：要从空库起数现判的块
        let disposable = DisposableLibrary::new("pairing-blocks-reuse")
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
        let library = CrashLibrary::open(CrashLibraryClass::JudgeTablesPairing).expect("开得了库");
        let units = judging_units_key_of_these_units(&[("gpu_judge_unit", 1)]);
        let other_units = judging_units_key_of_these_units(&[("cpu_judge_unit", 2)]);
        let flow_inputs_digest = [7u8; DIGEST_BYTES];
        let first_run = pairing_blocks_run(
            &library,
            &flow_inputs_digest,
            4 * 9,
            2,
            &units,
            &other_units,
        );
        assert_eq!(
            (
                first_run.blocks_of_the_flow.blocks_judged_now,
                first_run.blocks_of_the_flow.blocks_reused
            ),
            (9, 0),
            "头一趟 9 块都现判、一块都不复用"
        );
        let second_run = pairing_blocks_run(
            &library,
            &flow_inputs_digest,
            4 * 9,
            2,
            &units,
            &other_units,
        );
        assert_eq!(
            (
                second_run.blocks_of_the_flow.blocks_judged_now,
                second_run.blocks_of_the_flow.blocks_reused
            ),
            (0, 9),
            "第二趟 9 块都复用、现判 0"
        );
        assert!(
            second_run.blocks_asked_of_the_judge_each_time.is_empty(),
            "第二趟判段一次都不叫：{:?}",
            second_run.blocks_asked_of_the_judge_each_time
        );
        assert_eq!(
            second_run.conclusions_in_order, first_run.conclusions_in_order,
            "复用的结论与头一趟现判的逐块相同"
        );
        let refused: Result<JudgingUnitsKey, String> = Err("拒算".to_string());
        for _ in 0..2 {
            let refused_run = pairing_blocks_run(
                &library,
                &flow_inputs_digest,
                4 * 9,
                2,
                &refused,
                &other_units,
            );
            assert_eq!(
                refused_run.blocks_of_the_flow.blocks_judged_now, 9,
                "一边拒算的每趟照判、不入库"
            );
        }
    }

    /// 改判器一个单元之后现判的恰好是闭包含它的块：两条流，一条两边是 {改的单元, 共用单元} 与 {参照单元}，一条两边是 {共用单元} 与 {参照单元}；
    /// 改的单元指纹变了，第一边闭包含它的那条流每块现判、另一条每块复用；再跑一趟（单元都不改）两条流都现判 0。
    /// 判别力（`crates/mutations.tsv`）：块键里一边的判法单元指纹取成另一边的，改的单元变了也键不变、那条流一块都不现判，这条红。
    #[test]
    fn changing_one_judging_unit_judges_again_exactly_the_blocks_whose_judge_closure_holds_it() {
        // 一次性库留着：要从空库起数现判的块
        let disposable = DisposableLibrary::new("pairing-blocks-unit-changed")
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
        let library = CrashLibrary::open(CrashLibraryClass::JudgeTablesPairing).expect("开得了库");
        let flow_holding_the_unit = [11u8; DIGEST_BYTES];
        let flow_without_the_unit = [12u8; DIGEST_BYTES];
        let reference_side = judging_units_key_of_these_units(&[("reference_unit", 3)]);
        let side_without_the_unit = judging_units_key_of_these_units(&[("shared_unit", 1)]);
        let run_both_flows = |changed_unit_fingerprint: u8| {
            let side_holding_the_unit = judging_units_key_of_these_units(&[
                ("changed_unit", changed_unit_fingerprint),
                ("shared_unit", 1),
            ]);
            (
                pairing_blocks_run(
                    &library,
                    &flow_holding_the_unit,
                    4 * 6,
                    4,
                    &side_holding_the_unit,
                    &reference_side,
                )
                .blocks_of_the_flow,
                pairing_blocks_run(
                    &library,
                    &flow_without_the_unit,
                    4 * 5,
                    4,
                    &side_without_the_unit,
                    &reference_side,
                )
                .blocks_of_the_flow,
            )
        };
        let (holding_before_the_change, without_before_the_change) = run_both_flows(1);
        assert_eq!(
            (
                holding_before_the_change.blocks_judged_now,
                without_before_the_change.blocks_judged_now
            ),
            (6, 5),
            "头一趟两条流每块现判"
        );
        let (holding_after_the_change, without_after_the_change) = run_both_flows(2);
        assert_eq!(
            (
                holding_after_the_change.blocks_judged_now,
                holding_after_the_change.blocks_reused
            ),
            (6, 0),
            "改的单元变了：闭包含它的那条流 6 块都现判"
        );
        assert_eq!(
            (
                without_after_the_change.blocks_judged_now,
                without_after_the_change.blocks_reused
            ),
            (0, 5),
            "改的单元变了：闭包不含它的那条流 5 块都复用"
        );
        let (holding_on_the_rerun, without_on_the_rerun) = run_both_flows(2);
        assert_eq!(
            (
                holding_on_the_rerun.blocks_judged_now,
                without_on_the_rerun.blocks_judged_now
            ),
            (0, 0),
            "单元都不改再跑一趟：两条流都现判 0"
        );
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

    /// 一次性库不设任何临时服务的变量也建得出：它在这个进程里自己起一份临时 prune-server，socket 在那份服务独占的目录里（不是
    /// `XDG_RUNTIME_DIR` 下真服务的 socket），经它开得了库（头一次开写下布局行，另一个客户端再开读回布局行）；另一个客户端还连着就丢掉
    /// 一次性库，服务照样停掉（TCP 连不上）、那份目录不在。
    /// 判别力（`crates/mutations.tsv`）：一次性库连真服务的 socket、丢掉一次性库时不停临时服务，各让这一条红。
    #[test]
    fn a_disposable_library_starts_its_own_prune_server_in_this_process_and_removes_it_when_dropped(
    ) {
        // 一次性库留着：自证一次性库自己起、自己停临时服务
        let library = DisposableLibrary::new("own-temporary-service")
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        let service_directory = library.temporary_service_directory().to_path_buf();
        let network_address = library.temporary_service_network_address();
        let socket_path = match library.route() {
            RouteAddress::UnixSocket(socket_path) => socket_path.clone(),
            RouteAddress::Network(address) => {
                panic!("一次性库走它自己那份临时服务的 socket，不走 TCP：{address}")
            }
        };
        // TMPDIR 嵌得深（变异门禁每行一个的临时目录）时 socket 放不进临时服务自己的目录，另放在短目录里
        // （那一路由 a_disposable_library_under_a_temporary_root_too_long_for_the_socket_puts_the_socket_in_a_short_directory_and_removes_both 钉）
        let socket_fits_in_the_service_directory =
            socket_path_problem(&service_directory.join(TEMPORARY_PRUNE_SERVER_SOCKET_FILE_NAME))
                .is_none();
        if socket_fits_in_the_service_directory {
            assert!(
                socket_path.starts_with(&service_directory),
                "socket 在临时服务自己的目录里：{}",
                socket_path.display()
            );
        } else {
            assert_eq!(
                socket_path.parent().and_then(Path::parent),
                Some(Path::new(SHORT_SOCKET_DIRECTORY_ROOT)),
                "socket 放不进临时服务自己的目录时在 {SHORT_SOCKET_DIRECTORY_ROOT} 下另开的短目录里：{}",
                socket_path.display()
            );
        }
        assert_ne!(
            default_socket_path().ok(),
            Some(socket_path.clone()),
            "不是真服务的 socket"
        );
        assert!(
            std::os::unix::net::UnixStream::connect(&socket_path).is_ok(),
            "建库之后临时服务在，socket 上有人听：{}",
            socket_path.display()
        );
        assert!(
            network_address.ip().is_loopback(),
            "临时服务的 TCP 只绑本机回环：{network_address}"
        );
        let first_client = library
            .open_store(PruneServerFailureHandling::ReturnTheError)
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        let client_still_connected = library
            .open_store(PruneServerFailureHandling::ReturnTheError)
            .unwrap_or_else(|refusal| panic!("另一个客户端读回布局行：{refusal}"));
        assert_eq!(
            first_client.library_name(),
            client_still_connected.library_name(),
            "两个客户端开的是同一份"
        );
        drop(first_client);
        drop(library);
        assert!(
            !service_directory.exists(),
            "丢掉之后临时服务的目录不在：{}",
            service_directory.display()
        );
        assert!(
            std::net::TcpStream::connect(network_address).is_err(),
            "丢掉之后临时服务停了，TCP 连不上：{network_address}"
        );
        drop(client_still_connected);
    }

    /// 临时目录的根嵌得深、socket 放进临时服务自己的目录就超上限时（变异门禁每行一个的 `TMPDIR` 那种；这里把那条长目录交给
    /// [`DisposableLibrary::new_under`]，不改进程环境）：一次性库照样起得来，状态目录照旧在长目录下，socket 另放在
    /// [`SHORT_SOCKET_DIRECTORY_ROOT`] 下一份只有本用户进得去的短目录里、不超上限；经它开得了库（头一次开写下布局行，另一个客户端读回）；
    /// 丢掉之后两份目录都不在。
    /// 判别力（`crates/mutations.tsv`）：去掉退到短目录那一步（socket 照旧放进长目录），临时服务起不来，这条红。
    #[test]
    fn a_disposable_library_under_a_temporary_root_too_long_for_the_socket_puts_the_socket_in_a_short_directory_and_removes_both(
    ) {
        let long_root_parent = ExclusiveTemporaryDirectory::create("long-temporary-root")
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        let long_temporary_root =
            long_root_parent.join("deeply-nested-temporary-directory-".repeat(4));
        std::fs::create_dir(&long_temporary_root).unwrap_or_else(|error| {
            panic!("建不了长目录 {}：{error}", long_temporary_root.display())
        });
        assert!(
            socket_path_problem(
                &long_temporary_root
                    .join("singlefs-prune-server-0-0")
                    .join(TEMPORARY_PRUNE_SERVER_SOCKET_FILE_NAME)
            )
            .is_some(),
            "取样点要让 socket 放进临时服务自己的目录就超上限：{}",
            long_temporary_root.display()
        );
        // 一次性库留着：自证 socket 放不下时另开短目录
        let library = DisposableLibrary::new_under("long-temporary-root", &long_temporary_root)
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        let service_directory = library.temporary_service_directory().to_path_buf();
        let socket_path = match library.route() {
            RouteAddress::UnixSocket(socket_path) => socket_path.clone(),
            RouteAddress::Network(address) => {
                panic!("一次性库走它自己那份临时服务的 socket，不走 TCP：{address}")
            }
        };
        let short_socket_directory = socket_path
            .parent()
            .expect("socket 路径是一份目录里的一个文件，有上一级")
            .to_path_buf();
        assert!(
            service_directory.starts_with(&long_temporary_root),
            "状态目录照旧在长目录下：{}",
            service_directory.display()
        );
        assert_eq!(
            short_socket_directory.parent(),
            Some(Path::new(SHORT_SOCKET_DIRECTORY_ROOT)),
            "socket 在 {SHORT_SOCKET_DIRECTORY_ROOT} 下另开的短目录里：{}",
            socket_path.display()
        );
        assert_eq!(
            socket_path_problem(&socket_path),
            None,
            "短目录里的 socket 不超上限"
        );
        let short_directory_mode = {
            use std::os::unix::fs::PermissionsExt;
            /// `st_mode` 里权限那几位（去掉文件种类那几位）。
            const PERMISSION_BITS_OF_THE_MODE: u32 = 0o777;
            std::fs::metadata(&short_socket_directory)
                .unwrap_or_else(|error| {
                    panic!(
                        "短目录 {} 读不了：{error}",
                        short_socket_directory.display()
                    )
                })
                .permissions()
                .mode()
                & PERMISSION_BITS_OF_THE_MODE
        };
        assert_eq!(
            short_directory_mode,
            SHORT_SOCKET_DIRECTORY_MODE,
            "短目录只有本用户进得去：{}",
            short_socket_directory.display()
        );
        let first_client = library
            .open_store(PruneServerFailureHandling::ReturnTheError)
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        let second_client = library
            .open_store(PruneServerFailureHandling::ReturnTheError)
            .unwrap_or_else(|refusal| panic!("另一个客户端读回布局行：{refusal}"));
        assert_eq!(
            first_client.library_name(),
            second_client.library_name(),
            "两个客户端开的是同一份"
        );
        drop(first_client);
        drop(second_client);
        drop(library);
        assert!(
            !service_directory.exists(),
            "丢掉之后临时服务的目录不在：{}",
            service_directory.display()
        );
        assert!(
            !short_socket_directory.exists(),
            "丢掉之后放 socket 的短目录不在：{}",
            short_socket_directory.display()
        );
    }

    /// 两份一次性库是两段名字空间，头一个字节都是一次性库那一个（与全仓那一份的表标签、服务记账那一段都不撞）。
    #[test]
    fn two_disposable_libraries_take_two_namespaces_led_by_the_disposable_byte() {
        // 一次性库留着：自证两份一次性库两段名字空间
        let one = DisposableLibrary::new("namespace").unwrap_or_else(|refusal| panic!("{refusal}"));
        let another =
            DisposableLibrary::new("namespace").unwrap_or_else(|refusal| panic!("{refusal}"));
        let prefix_of = |library: &DisposableLibrary| match library.key_space() {
            LibraryKeySpace::DisposableLibrary(namespace) => namespace.clone(),
            LibraryKeySpace::TheOneLibrary => panic!("一次性库的键空间不会是全仓那一份"),
        };
        assert_eq!(
            prefix_of(&one).first(),
            Some(&DISPOSABLE_LIBRARY_KEY_SPACE_LEADING_BYTE)
        );
        assert_ne!(
            prefix_of(&one),
            prefix_of(&another),
            "两份一次性库两段名字空间"
        );
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
}
