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

/// 这个进程里建过几份独占的一次性临时目录（[`ExclusiveTemporaryDirectory::create`] 每次取一个号，名字里带它）。
static EXCLUSIVE_TEMPORARY_DIRECTORIES_CREATED_IN_THIS_PROCESS: AtomicU64 = AtomicU64::new(0);

/// 建一份独占的一次性临时目录最多试几个名字：名字撞上已有的目录（同进程号的旧进程被杀时留下的）就换下一个号，试满还建不出就报错。
const MOST_EXCLUSIVE_TEMPORARY_DIRECTORY_NAMES_TRIED: u32 = 64;

/// 一段路径名（一级目录的名字）最多几个字节：Linux 的 `NAME_MAX`，再长 `mkdir` 报 `File name too long (os error 36)`。
const MOST_BYTES_OF_ONE_PATH_COMPONENT: usize = 255;

/// 标签放不下、截短之后接在后面的标签摘要取几位十六进制：标签整串 sha256 的前 16 位（64 比特），
/// 只在截掉的那一段不同的两个标签靠它分开，同一台机器上同时在的一次性临时目录撞上的机会不计。
const LABEL_DIGEST_HEXADECIMAL_DIGITS_OF_A_SHORTENED_NAME: usize = 16;

/// 一次性临时目录的单段名字 `singlefs-<标签>-<进程号>-<进程内序号>`，标签里的 `/` 换成 `-`。
/// 整段超 [`MOST_BYTES_OF_ONE_PATH_COMPONENT`] 字节的，标签截到放得下（按 UTF-8 字符边界截，不截半个字符），
/// 截短的标签后面接 `-` 与标签整串 sha256 十六进制的前 [`LABEL_DIGEST_HEXADECIMAL_DIGITS_OF_A_SHORTENED_NAME`] 位，
/// 再接 `-<进程号>-<进程内序号>`：只在截掉的那一段不同的两个标签照样得出不同的名字。放得下的照整串拼。
fn exclusive_temporary_directory_name(
    label: &str,
    process_identifier: u32,
    serial_in_this_process: u64,
) -> String {
    let label_without_slashes = label.replace('/', "-");
    let suffix = format!("-{process_identifier}-{serial_in_this_process}");
    let name_in_full = format!("singlefs-{label_without_slashes}{suffix}");
    if name_in_full.len() <= MOST_BYTES_OF_ONE_PATH_COMPONENT {
        return name_in_full;
    }
    let label_digest = hexadecimal_text(&sha256_digest(label.as_bytes()));
    let digest_prefix = &label_digest[..LABEL_DIGEST_HEXADECIMAL_DIGITS_OF_A_SHORTENED_NAME];
    let label_bytes_that_fit = MOST_BYTES_OF_ONE_PATH_COMPONENT
        .saturating_sub("singlefs-".len() + "-".len() + digest_prefix.len() + suffix.len());
    let mut label_bytes_kept = label_bytes_that_fit;
    // 迭代上界：3 次（UTF-8 一个字符最多 4 个字节，退到字符边界最多退 3 个字节；0 总是字符边界）
    while !label_without_slashes.is_char_boundary(label_bytes_kept) {
        label_bytes_kept -= 1;
    }
    format!(
        "singlefs-{}-{digest_prefix}{suffix}",
        &label_without_slashes[..label_bytes_kept]
    )
}

/// 调用方独占的一份一次性临时目录：`${TMPDIR:-/tmp}` 下 `singlefs-<标签>-<进程号>-<进程内序号>`（标签里的 `/` 换成 `-`，目录只占一层；
/// 整段超一段路径名的上限时标签截短、接标签摘要，[`exclusive_temporary_directory_name`]），`create_dir` 排他新建；
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
            let path = root.join(exclusive_temporary_directory_name(
                label,
                std::process::id(),
                serial_in_this_process,
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
    /// 单测构建里这份临时服务占的那一个名额（[`TemporaryServerPlaceInTheTestProcess`]）：排在最后，服务停了、目录删了才还
    #[cfg(test)]
    _place_in_the_test_process: TemporaryServerPlaceInTheTestProcess,
}

/// 单测进程里用例自己开的线程共用的那一份额度分成的两份（`.claude/rules/implementation-workflow.md`「测试与崩溃检测优先多线程」）。
/// libtest 已经照每台默认重活线程数一起跑那么多条用例，用例里再开的线程不再各照这个数开：一次性库的临时服务同时至多
/// `temporary_servers` 份（每份照 [`THREADS_OF_A_TEMPORARY_SERVER`] 条记，至多占一半），余下的 `inner_threads` 条给里层的核对线程
/// （[`InnerThreadsTaken`]）。两份合起来不超过每台默认重活线程数（它大过 [`THREADS_OF_A_TEMPORARY_SERVER`] 时）。
#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SharesOfTheTestProcess {
    pub(crate) temporary_servers: usize,
    pub(crate) inner_threads: usize,
}

/// 一份临时服务常驻的线程（接 socket、接 TCP、提交各一条，加工作线程）与一条连接的两条（读帧、写回），
/// 照 `prune_server::server` 在这个进程里起服务的那几处数。
#[cfg(test)]
pub(crate) const THREADS_OF_A_TEMPORARY_SERVER: usize =
    3 + prune_server::server::WORKER_THREADS_OF_A_SERVICE_IN_THIS_PROCESS + 2;

/// 每台默认重活线程数 `heavy_threads` 分成的两份（[`SharesOfTheTestProcess`]）：临时服务至少一份（不然一次性库永远起不来），
/// 里层至少一条。
#[cfg(test)]
pub(crate) fn shares_of_the_test_process(heavy_threads: usize) -> SharesOfTheTestProcess {
    let temporary_servers = (heavy_threads / 2 / THREADS_OF_A_TEMPORARY_SERVER).max(1);
    SharesOfTheTestProcess {
        temporary_servers,
        inner_threads: heavy_threads
            .saturating_sub(temporary_servers * THREADS_OF_A_TEMPORARY_SERVER)
            .max(1),
    }
}

/// 一处里层并行要 `wanted` 条线程时拿到几条：不过里层那一份 `inner_threads_of_the_share`，至少一条。要么整份拿到、要么等，不拆开拿：
/// 同一处每次拿到的条数一样，切段的界不随别的用例此刻占了多少变。
#[cfg(test)]
pub(crate) fn inner_threads_granted(wanted: usize, inner_threads_of_the_share: usize) -> usize {
    wanted.min(inner_threads_of_the_share).max(1)
}

/// 此刻里层已占 `in_use` 条时，再拿 `granted` 条放不放得下里层那一份 `inner_threads_of_the_share`。
#[cfg(test)]
pub(crate) fn inner_threads_fit(
    in_use: usize,
    granted: usize,
    inner_threads_of_the_share: usize,
) -> bool {
    in_use + granted <= inner_threads_of_the_share
}

/// 单测进程此刻占着的：里层几条线程、几份临时服务，与哪条线程起着几份临时服务（同一条线程再起一份不再等：
/// 它手里那一份要等它自己往下走才还，等就是自己等自己）。
#[cfg(test)]
struct ThreadsInUseInTheTestProcess {
    inner_threads: usize,
    temporary_servers: usize,
    temporary_servers_started_by_each_thread: Vec<(std::thread::ThreadId, usize)>,
}

#[cfg(test)]
static THREADS_IN_USE_IN_THE_TEST_PROCESS: Mutex<ThreadsInUseInTheTestProcess> =
    Mutex::new(ThreadsInUseInTheTestProcess {
        inner_threads: 0,
        temporary_servers: 0,
        temporary_servers_started_by_each_thread: Vec::new(),
    });

/// 还了线程或名额就叫醒等着的。
#[cfg(test)]
static THREADS_RETURNED_IN_THE_TEST_PROCESS: std::sync::Condvar = std::sync::Condvar::new();

#[cfg(test)]
fn shares_of_this_test_process() -> SharesOfTheTestProcess {
    shares_of_the_test_process(crate::heavy_threads::default_heavy_threads().get())
}

/// 里层一处并行拿到的核对线程：产品构建里就是要的那么多条；单测构建里从单测进程的那一份额度拿（`SharesOfTheTestProcess`），
/// 拿不到就等，丢掉时还回去。
pub(crate) struct InnerThreadsTaken {
    threads: usize,
}

impl InnerThreadsTaken {
    /// 要 `wanted` 条（至少一条）：产品构建里照给。
    #[cfg(not(test))]
    pub(crate) fn take(wanted: usize) -> Self {
        Self { threads: wanted }
    }

    /// 要 `wanted` 条：单测构建里照 [`inner_threads_granted`] 定拿几条，里层那一份放不下就等别的用例还。
    #[cfg(test)]
    pub(crate) fn take(wanted: usize) -> Self {
        let inner_threads_of_the_share = shares_of_this_test_process().inner_threads;
        let threads = inner_threads_granted(wanted, inner_threads_of_the_share);
        let mut in_use = THREADS_IN_USE_IN_THE_TEST_PROCESS
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        // 迭代上界是别的用例还线程的次数：每醒一次重判一次
        while !inner_threads_fit(in_use.inner_threads, threads, inner_threads_of_the_share) {
            in_use = THREADS_RETURNED_IN_THE_TEST_PROCESS
                .wait(in_use)
                .unwrap_or_else(PoisonError::into_inner);
        }
        in_use.inner_threads += threads;
        Self { threads }
    }

    pub(crate) fn threads(&self) -> usize {
        self.threads
    }
}

#[cfg(test)]
impl Drop for InnerThreadsTaken {
    fn drop(&mut self) {
        THREADS_IN_USE_IN_THE_TEST_PROCESS
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .inner_threads -= self.threads;
        THREADS_RETURNED_IN_THE_TEST_PROCESS.notify_all();
    }
}

/// 单测构建里一份一次性库的临时服务占的名额（[`SharesOfTheTestProcess`] 的 `temporary_servers`）：起之前拿，满了就等；
/// 同一条线程已经起着一份的不再等、不另占。丢掉时那条线程起的都丢了才还。
/// 次序：先取名额（开一次性库）、后拿改环境的锁（`crate::crash_amplification::EnvironmentLockOfThisTestBinary`）。
#[cfg(test)]
struct TemporaryServerPlaceInTheTestProcess {
    started_by: std::thread::ThreadId,
}

/// 拿着改环境的锁去新占一次性库名额时 panic 的那句话。
#[cfg(test)]
const PLACE_TAKEN_WHILE_HOLDING_THE_ENVIRONMENT_LOCK: &str =
    "这条线程拿着改环境的锁去新占一次性库名额：次序要先开一次性库（取名额）、后拿改环境的锁（CrashAmplificationEnvironment、\
     one_test_changes_the_environment_at_a_time）；反过来拿，名额满着时拿着名额等锁的用例与拿着锁等名额的用例互相等、整个测试进程挂住。\
     把开库挪到拿锁之前";

#[cfg(test)]
impl TemporaryServerPlaceInTheTestProcess {
    /// # Panics
    /// 要新占名额（这条线程还没起着一份）而这条线程拿着改环境的锁（[`PLACE_TAKEN_WHILE_HOLDING_THE_ENVIRONMENT_LOCK`]）：
    /// 在等名额之前当场 panic，不挂住。这条线程已经起着一份的不等名额、不查（拿着锁再开一份不会互等）。
    fn take() -> Self {
        let started_by = std::thread::current().id();
        let temporary_servers_of_the_share = shares_of_this_test_process().temporary_servers;
        let mut in_use = THREADS_IN_USE_IN_THE_TEST_PROCESS
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if let Some((_thread, started)) = in_use
            .temporary_servers_started_by_each_thread
            .iter_mut()
            .find(|(thread, _started)| *thread == started_by)
        {
            *started += 1;
            return Self { started_by };
        }
        if crate::crash_amplification::this_thread_holds_the_environment_lock() {
            // 先放占用计数的锁再 panic：不让它中毒，别的用例照常取还
            drop(in_use);
            panic!("{PLACE_TAKEN_WHILE_HOLDING_THE_ENVIRONMENT_LOCK}");
        }
        // 迭代上界是别的用例还名额的次数：每醒一次重判一次
        while in_use.temporary_servers >= temporary_servers_of_the_share {
            in_use = THREADS_RETURNED_IN_THE_TEST_PROCESS
                .wait(in_use)
                .unwrap_or_else(PoisonError::into_inner);
        }
        in_use.temporary_servers += 1;
        in_use
            .temporary_servers_started_by_each_thread
            .push((started_by, 1));
        Self { started_by }
    }
}

#[cfg(test)]
impl Drop for TemporaryServerPlaceInTheTestProcess {
    fn drop(&mut self) {
        let mut in_use = THREADS_IN_USE_IN_THE_TEST_PROCESS
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let position = in_use
            .temporary_servers_started_by_each_thread
            .iter()
            .position(|(thread, _started)| *thread == self.started_by)
            .expect("名额拿到时记下了起它的那条线程，那条线程起的都丢了才删这一项");
        in_use.temporary_servers_started_by_each_thread[position].1 -= 1;
        if in_use.temporary_servers_started_by_each_thread[position].1 == 0 {
            in_use
                .temporary_servers_started_by_each_thread
                .swap_remove(position);
            in_use.temporary_servers -= 1;
        }
        drop(in_use);
        THREADS_RETURNED_IN_THE_TEST_PROCESS.notify_all();
    }
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
        #[cfg(test)]
        let place_in_the_test_process = TemporaryServerPlaceInTheTestProcess::take();
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
            #[cfg(test)]
            _place_in_the_test_process: place_in_the_test_process,
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

/// 开着的一份：库，一把锁一次一个读写（容错客户端追加日志的一次性目录在它连着的那一端里，随最后一个放手的删掉）；这把锁的累计时长；
/// 这条连接读回过的流与它们的计划；持锁时长行里报的连接名（[`connection_name_of_an_opening`]）。
struct OpenLibrary {
    connection_name: String,
    store: Mutex<VerdictStore>,
    lock_times: LibraryLockTimes,
    flows_read_back:
        ValuesReadBackOnceInThisProcess<crate::crash_amplification::RecordedFlowAndItsPlan>,
}

/// 这个进程里开过几次连接：连接名带上开的是第几次，同一个进程里关了再开同一份库，两次的行不同名（各自的 `seconds_open` 各从零数）。
static OPENINGS_OF_A_LIBRARY_IN_THIS_PROCESS: AtomicU64 = AtomicU64::new(0);

/// 持锁时长行里的连接名：连哪一处、键空间前缀（十六进制，全仓那一份是空的）、这个进程里开的第几次，三段用 `#` 接起来，空白换成 `_`
/// （行按空格切字段）。
fn connection_name_of_an_opening(key: &KeyOfALibraryOpenInThisProcess) -> String {
    let opening = OPENINGS_OF_A_LIBRARY_IN_THIS_PROCESS.fetch_add(1, Ordering::Relaxed);
    format!("{}#{}#{opening}", key.0, hexadecimal_text(&key.1))
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join("_")
}

fn seconds_of_nanoseconds(nanoseconds: u64) -> f64 {
    std::time::Duration::from_nanos(nanoseconds).as_secs_f64()
}

impl OpenLibrary {
    /// `CRASH_LIBRARY_LOCK_TIME connection= pid= seconds_open= seconds_held= seconds_held_in_round_trips= seconds_held_outside_round_trips=
    /// seconds_waited= most_askers_at_once= acquisitions= held_share_of_the_wall_clock= flows_read_over_the_connection=`，
    /// 字段都是从连接开着起累计的：一趟的读数按 `connection=` 与 `pid=` 各取这一趟日志里那条连接的最后一行。
    /// `seconds_held_in_round_trips` 是锁里花在与 prune-server 往返上的秒数（[`VerdictStore::nanoseconds_in_round_trips_so_far`]），
    /// `most_askers_at_once` 是同时在等这把锁的最多线程数。
    fn lock_time_line(&self, time_open: std::time::Duration) -> String {
        let times = &self.lock_times;
        let seconds_open = time_open.as_secs_f64();
        let held_nanoseconds = times.nanoseconds_held.load(Ordering::Relaxed);
        let held_in_round_trips_nanoseconds = times
            .nanoseconds_held_in_round_trips
            .load(Ordering::Relaxed);
        let held = seconds_of_nanoseconds(held_nanoseconds);
        let waited = seconds_of_nanoseconds(times.nanoseconds_waited.load(Ordering::Relaxed));
        let held_share = if seconds_open > 0.0 {
            held / seconds_open
        } else {
            0.0
        };
        format!(
            "{LIBRARY_LOCK_TIME_LINE_PREFIX} connection={} pid={} seconds_open={seconds_open:.3} seconds_held={held:.3} seconds_held_in_round_trips={:.3} seconds_held_outside_round_trips={:.3} seconds_waited={waited:.3} most_askers_at_once={} acquisitions={} held_share_of_the_wall_clock={held_share:.3} flows_read_over_the_connection={}",
            self.connection_name,
            std::process::id(),
            seconds_of_nanoseconds(held_in_round_trips_nanoseconds),
            seconds_of_nanoseconds(held_nanoseconds.saturating_sub(held_in_round_trips_nanoseconds)),
            times.most_askers_at_once.load(Ordering::Relaxed),
            times.acquisitions.load(Ordering::Relaxed),
            self.flows_read_back
                .values_read_over_the_connection
                .load(Ordering::Relaxed)
        )
    }

    /// 按取锁的调用点各一行（[`LIBRARY_LOCK_TIME_OF_A_SITE_LINE_PREFIX`]），持锁秒数大的在前：`connection= pid= site=<文件>:<行>
    /// seconds_held= seconds_held_in_round_trips= seconds_held_outside_round_trips= acquisitions= longest_hold_seconds=`，从连接开着起累计。
    fn lock_time_lines_of_the_sites(&self) -> Vec<String> {
        let sites = self
            .lock_times
            .of_each_site
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let mut ordered: Vec<(&(&'static str, u32), &LockTimesOfASite)> = sites.iter().collect();
        ordered.sort_by_key(|site| std::cmp::Reverse(site.1.nanoseconds_held));
        ordered
            .into_iter()
            .map(|((file, line), of_the_site)| {
                format!(
                    "{LIBRARY_LOCK_TIME_OF_A_SITE_LINE_PREFIX} connection={} pid={} site={file}:{line} seconds_held={:.3} seconds_held_in_round_trips={:.3} seconds_held_outside_round_trips={:.3} acquisitions={} longest_hold_seconds={:.3}",
                    self.connection_name,
                    std::process::id(),
                    seconds_of_nanoseconds(of_the_site.nanoseconds_held),
                    seconds_of_nanoseconds(of_the_site.nanoseconds_held_in_round_trips),
                    seconds_of_nanoseconds(
                        of_the_site
                            .nanoseconds_held
                            .saturating_sub(of_the_site.nanoseconds_held_in_round_trips)
                    ),
                    of_the_site.acquisitions,
                    seconds_of_nanoseconds(of_the_site.longest_hold_nanoseconds),
                )
            })
            .collect()
    }

    /// 连接那一行连同各调用点那几行打进标准错误。
    fn print_the_lock_time_lines(&self, time_open: std::time::Duration) {
        eprintln!("{}", self.lock_time_line(time_open));
        for line in self.lock_time_lines_of_the_sites() {
            eprintln!("{line}");
        }
    }
}

impl Drop for OpenLibrary {
    fn drop(&mut self) {
        self.print_the_lock_time_lines(self.lock_times.opened_at.elapsed());
    }
}

/// 持锁时长行的头（[`OpenLibrary::lock_time_line`]）：只打进日志，不进任何键。
pub const LIBRARY_LOCK_TIME_LINE_PREFIX: &str = "CRASH_LIBRARY_LOCK_TIME";

/// 按调用点的持锁时长行的头（[`OpenLibrary::lock_time_lines_of_the_sites`]）：只打进日志，不进任何键。
pub const LIBRARY_LOCK_TIME_OF_A_SITE_LINE_PREFIX: &str = "CRASH_LIBRARY_LOCK_TIME_OF_A_SITE";

/// 连接开着时隔多久打一行持锁时长：实验装置把引擎放在 `static` 里，连接开到进程退出、放手那一行打不出来，读数靠这一行。
const SECONDS_BETWEEN_LIBRARY_LOCK_TIME_LINES: u64 = 60;

/// 一条连接上那把锁的累计时长。线程数越多等锁的秒数加起来越大，可以超过连接开着的秒数；判收窄成不成看持锁占比。
struct LibraryLockTimes {
    opened_at: std::time::Instant,
    nanoseconds_held: AtomicU64,
    /// 持锁的纳秒里花在与 prune-server 往返上的那一部分。
    nanoseconds_held_in_round_trips: AtomicU64,
    nanoseconds_waited: AtomicU64,
    /// 此刻在等这把锁的线程数，与它到过的最大值。
    askers_now: AtomicU64,
    most_askers_at_once: AtomicU64,
    acquisitions: AtomicU64,
    next_line_at_second: AtomicU64,
    /// 按取锁的调用点（文件，行）各记一份。
    of_each_site: Mutex<BTreeMap<(&'static str, u32), LockTimesOfASite>>,
}

/// 一个取锁调用点的累计：持锁纳秒、其中往返的纳秒、取锁次数、最长一次持锁的纳秒。
#[derive(Default)]
struct LockTimesOfASite {
    nanoseconds_held: u64,
    nanoseconds_held_in_round_trips: u64,
    acquisitions: u64,
    longest_hold_nanoseconds: u64,
}

impl LibraryLockTimes {
    fn starting_at_this_opening() -> Self {
        Self {
            opened_at: std::time::Instant::now(),
            nanoseconds_held: AtomicU64::new(0),
            nanoseconds_held_in_round_trips: AtomicU64::new(0),
            nanoseconds_waited: AtomicU64::new(0),
            askers_now: AtomicU64::new(0),
            most_askers_at_once: AtomicU64::new(0),
            acquisitions: AtomicU64::new(0),
            next_line_at_second: AtomicU64::new(SECONDS_BETWEEN_LIBRARY_LOCK_TIME_LINES),
            of_each_site: Mutex::new(BTreeMap::new()),
        }
    }

    /// 一次持锁放手时记进这一次的调用点：持了多久、其中往返多久。
    fn note_a_hold_at_the_site(
        &self,
        site: &'static std::panic::Location<'static>,
        nanoseconds_held: u64,
        nanoseconds_held_in_round_trips: u64,
    ) {
        self.nanoseconds_held_in_round_trips
            .fetch_add(nanoseconds_held_in_round_trips, Ordering::Relaxed);
        let mut sites = self
            .of_each_site
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let of_the_site = sites.entry((site.file(), site.line())).or_default();
        of_the_site.nanoseconds_held += nanoseconds_held;
        of_the_site.nanoseconds_held_in_round_trips += nanoseconds_held_in_round_trips;
        of_the_site.acquisitions += 1;
        of_the_site.longest_hold_nanoseconds =
            of_the_site.longest_hold_nanoseconds.max(nanoseconds_held);
    }

    /// 到了下一行该打的时候交回连接开了多久；几条线程同时到点只有抢到的那一条拿到。在取锁之前调，打这一行不算进持锁时长。
    fn time_open_when_a_lock_time_line_is_due(
        &self,
        now: std::time::Instant,
    ) -> Option<std::time::Duration> {
        let time_open = now.duration_since(self.opened_at);
        let next_line_at_second = self.next_line_at_second.load(Ordering::Relaxed);
        if time_open.as_secs() < next_line_at_second {
            return None;
        }
        self.next_line_at_second
            .compare_exchange(
                next_line_at_second,
                time_open.as_secs() + SECONDS_BETWEEN_LIBRARY_LOCK_TIME_LINES,
                Ordering::Relaxed,
                Ordering::Relaxed,
            )
            .ok()
            .map(|_line_due_before| time_open)
    }
}

fn nanoseconds_of_a_library_lock_duration(duration: std::time::Duration) -> u64 {
    u64::try_from(duration.as_nanos()).expect("一次取锁或持锁的纳秒数装得进 u64：五百多年")
}

thread_local! {
    /// 这条线程此刻握着几把库锁（[`LibraryLockHeld`] 造出来加一、丢掉减一；它不能跨线程）。
    static LIBRARY_LOCKS_HELD_BY_THIS_THREAD: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// 纯计算（排计划、序列化、算摘要、打包、解码）的路上判：这条线程没握着库锁。别的线程握着不算，它们本来就该能同时算。
/// 这几步挪回锁里时当场 panic，不等到量出并行上不去。
///
/// # Panics
/// 这条线程握着库锁。
pub(crate) fn assert_this_thread_holds_no_library_lock(pure_work: &str) {
    let locks_held = LIBRARY_LOCKS_HELD_BY_THIS_THREAD.with(std::cell::Cell::get);
    assert_eq!(
        locks_held, 0,
        "{pure_work}是纯计算，要挪到判定库的锁外：握着锁算，这个进程里别的线程读写判定库都得等它"
    );
}

/// 手里握着的库锁：照 [`VerdictStore`] 用，丢掉时放锁、把握了多久（其中往返多久）记进 [`LibraryLockTimes`]，按取锁的调用点各记一份。
pub struct LibraryLockHeld<'library> {
    store: MutexGuard<'library, VerdictStore>,
    times: &'library LibraryLockTimes,
    acquired_at: std::time::Instant,
    site: &'static std::panic::Location<'static>,
    nanoseconds_in_round_trips_at_acquisition: u64,
}

impl std::ops::Deref for LibraryLockHeld<'_> {
    type Target = VerdictStore;

    fn deref(&self) -> &VerdictStore {
        &self.store
    }
}

impl std::ops::DerefMut for LibraryLockHeld<'_> {
    fn deref_mut(&mut self) -> &mut VerdictStore {
        &mut self.store
    }
}

impl Drop for LibraryLockHeld<'_> {
    fn drop(&mut self) {
        self.times.nanoseconds_held.fetch_add(
            nanoseconds_of_a_library_lock_duration(self.acquired_at.elapsed()),
            Ordering::Relaxed,
        );
        self.times.note_a_hold_at_the_site(
            self.site,
            nanoseconds_of_a_library_lock_duration(self.acquired_at.elapsed()),
            self.store
                .nanoseconds_in_round_trips_so_far()
                .saturating_sub(self.nanoseconds_in_round_trips_at_acquisition),
        );
        LIBRARY_LOCKS_HELD_BY_THIS_THREAD.with(|locks_held| locks_held.set(locks_held.get() - 1));
    }
}

/// 一条连接最多留多少字节读回的值（按读回前的载荷算；解码之后的 Rust 结构大一点，粗估在载荷的两倍以内，推的）：超了从最早读回的放起。
/// E171（三oracle合谋测试）两条流，载荷大的那条约 7.2 MiB，用不到这个数；比这个数还大的流不留，下一次照读。
const MOST_BYTES_OF_FLOWS_HELD_OF_A_LIBRARY: u64 = 512 * 1024 * 1024;

/// 一个摘要一个格子：几条线程同时要同一个摘要时，头一条握着格子去读，其余等它、拿它读回的那一份，不各读一遍。
type SlotOfAValueReadBack<Value> = Arc<Mutex<Option<Arc<Value>>>>;

struct SlotsOfTheValuesReadBack<Value> {
    slot_of_digest: BTreeMap<[u8; DIGEST_BYTES], SlotOfAValueReadBack<Value>>,
    /// 读回过的（摘要，载荷字节数），最早读回的在前。
    held_in_the_order_read: VecDeque<([u8; DIGEST_BYTES], u64)>,
    bytes_held: u64,
}

/// 同一摘要的值只从库里读回一次、留在内存里给这条连接上后面要它的人：内容由摘要定，读回时核过摘要，再读一遍读到的是同样的字节。
/// 只留读回的，不留录入段内存里的那一份（录入与判两段中间只经库），不写结论，不是第二份判定库。
struct ValuesReadBackOnceInThisProcess<Value> {
    most_bytes_held: u64,
    slots: Mutex<SlotsOfTheValuesReadBack<Value>>,
    values_read_over_the_connection: AtomicU64,
}

impl<Value> ValuesReadBackOnceInThisProcess<Value> {
    fn holding_at_most(most_bytes_held: u64) -> Self {
        Self {
            most_bytes_held,
            slots: Mutex::new(SlotsOfTheValuesReadBack {
                slot_of_digest: BTreeMap::new(),
                held_in_the_order_read: VecDeque::new(),
                bytes_held: 0,
            }),
            values_read_over_the_connection: AtomicU64::new(0),
        }
    }

    /// 这个摘要留着就交回留着的；没留着调 `read` 读（交回值与它的载荷字节数）、留下。`read` 没读回值（只录、只判了这台机器的一份、出错）什么都不留，
    /// 下一次照调。同一摘要同时只有一条线程在 `read` 里，它不握着 `slots`，别的摘要不等它。
    ///
    /// # Errors
    /// `read` 的错。
    fn held_or_read_once(
        &self,
        digest: &[u8; DIGEST_BYTES],
        read: impl FnOnce() -> Result<StageOutcome<(Value, u64)>, String>,
    ) -> Result<StageOutcome<Arc<Value>>, String> {
        let slot: SlotOfAValueReadBack<Value> = Arc::clone(
            self.slots
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .slot_of_digest
                .entry(*digest)
                .or_default(),
        );
        let mut value_in_the_slot = slot.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(value) = &*value_in_the_slot {
            return Ok(StageOutcome::Judged(Arc::clone(value)));
        }
        let (value, payload_bytes) = match read()? {
            StageOutcome::RecordedOnly => return Ok(StageOutcome::RecordedOnly),
            StageOutcome::JudgedMyShareOnly {
                blocks_judged_here,
                blocks_left_to_other_hosts,
            } => {
                return Ok(StageOutcome::JudgedMyShareOnly {
                    blocks_judged_here,
                    blocks_left_to_other_hosts,
                })
            }
            StageOutcome::Judged((value, payload_bytes)) => (Arc::new(value), payload_bytes),
        };
        self.values_read_over_the_connection
            .fetch_add(1, Ordering::Relaxed);
        *value_in_the_slot = Some(Arc::clone(&value));
        drop(value_in_the_slot);
        self.hold_within_the_limit(digest, payload_bytes);
        Ok(StageOutcome::Judged(value))
    }

    /// 记下刚读回的这一份，超了额度从最早读回的放起（它自己比额度还大就连它一起放）。已经拿到手的 [`Arc`] 不受影响，放的只是这里留的那一份。
    fn hold_within_the_limit(&self, digest: &[u8; DIGEST_BYTES], payload_bytes: u64) {
        let mut slots = self.slots.lock().unwrap_or_else(PoisonError::into_inner);
        slots
            .held_in_the_order_read
            .push_back((*digest, payload_bytes));
        slots.bytes_held += payload_bytes;
        // 迭代上界：留着的份数，每一轮放一份
        while slots.bytes_held > self.most_bytes_held {
            let Some((digest_let_go, bytes_let_go)) = slots.held_in_the_order_read.pop_front()
            else {
                break;
            };
            slots.bytes_held -= bytes_let_go;
            slots.slot_of_digest.remove(&digest_let_go);
        }
    }
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
            connection_name: connection_name_of_an_opening(&key),
            store: Mutex::new(store),
            lock_times: LibraryLockTimes::starting_at_this_opening(),
            flows_read_back: ValuesReadBackOnceInThisProcess::holding_at_most(
                MOST_BYTES_OF_FLOWS_HELD_OF_A_LIBRARY,
            ),
        });
        open_libraries.insert(key.clone(), Arc::downgrade(&open));
        Ok(Self {
            key,
            open: Some(open),
        })
    }

    fn open_library(&self) -> &OpenLibrary {
        self.open
            .as_ref()
            .expect("库在放手之前一直在：只有 Drop 取走它")
    }

    /// 库里那一把锁：一次一个读写。握着它只做与 prune-server 的往返。持锁时长按调用这一处的文件与行记（`#[track_caller]`）。
    #[track_caller]
    pub fn lock(&self) -> LibraryLockHeld<'_> {
        let site = std::panic::Location::caller();
        let open = self.open_library();
        let asked_at = std::time::Instant::now();
        if let Some(time_open) = open
            .lock_times
            .time_open_when_a_lock_time_line_is_due(asked_at)
        {
            open.print_the_lock_time_lines(time_open);
        }
        let askers_with_this_one = open.lock_times.askers_now.fetch_add(1, Ordering::Relaxed) + 1;
        open.lock_times
            .most_askers_at_once
            .fetch_max(askers_with_this_one, Ordering::Relaxed);
        let store = open.store.lock().unwrap_or_else(PoisonError::into_inner);
        open.lock_times.askers_now.fetch_sub(1, Ordering::Relaxed);
        let nanoseconds_in_round_trips_at_acquisition = store.nanoseconds_in_round_trips_so_far();
        let acquired_at = std::time::Instant::now();
        open.lock_times.nanoseconds_waited.fetch_add(
            nanoseconds_of_a_library_lock_duration(acquired_at.duration_since(asked_at)),
            Ordering::Relaxed,
        );
        open.lock_times.acquisitions.fetch_add(1, Ordering::Relaxed);
        LIBRARY_LOCKS_HELD_BY_THIS_THREAD.with(|locks_held| locks_held.set(locks_held.get() + 1));
        LibraryLockHeld {
            store,
            times: &open.lock_times,
            acquired_at,
            site,
            nanoseconds_in_round_trips_at_acquisition,
        }
    }
}

/// 判段与录入段读写判定库经它：每一次读写现取现放（[`Self::take`]），手里不留整段握着的 [`VerdictStore`]，判、看、排计划都在锁外，
/// 同一条连接上别的线程（同一个进程里并行跑的用例、给定状态表的引擎）在这期间照样读写。
pub enum LibraryTakenAtEachReadOrWrite<'library> {
    /// 这个进程里开着的那一份：每次取的是它那一把锁，持锁时长按取锁的调用点记。
    SharedInThisProcess(&'library LibraryOpenInThisProcess),
    /// 调用方独占的一份（单测与实验装置自己开的、不经这个进程登记表的那一份库）：同样一次一取，没有别的连接上的线程等它。
    HeldByThisCallerAlone(Mutex<&'library mut VerdictStore>),
}

impl<'library> LibraryTakenAtEachReadOrWrite<'library> {
    /// 调用方独占的那一份库照每次读写一取的办法交下去。
    #[must_use]
    pub fn held_by_this_caller_alone(store: &'library mut VerdictStore) -> Self {
        Self::HeldByThisCallerAlone(Mutex::new(store))
    }

    /// 为这一次读写取库：取到的丢掉就放。同一条线程上手里握着一份还没丢又来取，等的是自己，死锁。
    #[track_caller]
    pub fn take(&self) -> StoreTakenForOneReadOrWrite<'_, 'library> {
        match self {
            Self::SharedInThisProcess(library) => {
                StoreTakenForOneReadOrWrite::UnderTheLockOfTheConnection(library.lock())
            }
            Self::HeldByThisCallerAlone(store) => {
                StoreTakenForOneReadOrWrite::HeldByThisCallerAlone(
                    store.lock().unwrap_or_else(PoisonError::into_inner),
                )
            }
        }
    }
}

/// 为一次读写取到的库（[`LibraryTakenAtEachReadOrWrite::take`]）：照 [`VerdictStore`] 用，丢掉就放。
pub enum StoreTakenForOneReadOrWrite<'taken, 'library> {
    UnderTheLockOfTheConnection(LibraryLockHeld<'taken>),
    HeldByThisCallerAlone(MutexGuard<'taken, &'library mut VerdictStore>),
}

impl std::ops::Deref for StoreTakenForOneReadOrWrite<'_, '_> {
    type Target = VerdictStore;

    fn deref(&self) -> &VerdictStore {
        match self {
            Self::UnderTheLockOfTheConnection(held) => held,
            Self::HeldByThisCallerAlone(store) => store,
        }
    }
}

impl std::ops::DerefMut for StoreTakenForOneReadOrWrite<'_, '_> {
    fn deref_mut(&mut self) -> &mut VerdictStore {
        match self {
            Self::UnderTheLockOfTheConnection(held) => held,
            Self::HeldByThisCallerAlone(store) => store,
        }
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

    /// 库里那一把锁：一次一个读写。握着它只做与 prune-server 的往返。持锁时长按调用这一处的文件与行记。
    #[track_caller]
    pub fn lock(&self) -> LibraryLockHeld<'_> {
        self.library.lock()
    }

    /// 这一类测试的判段、录入段读写库的那一份：每一次读写现取现放（[`LibraryTakenAtEachReadOrWrite::SharedInThisProcess`]）。
    #[must_use]
    pub fn taken_at_each_read_or_write(&self) -> LibraryTakenAtEachReadOrWrite<'_> {
        LibraryTakenAtEachReadOrWrite::SharedInThisProcess(&self.library)
    }

    /// 这个摘要的流留着就交回留着的（读回过、排过计划的那一份）；没留着调 `read`（交回从库里读回的流连计划与载荷字节数）、留下。
    /// 同一条连接上同一摘要只读回一次。
    ///
    /// # Errors
    /// `read` 的错。
    pub(crate) fn recorded_flow_and_its_plan_read_back_once(
        &self,
        flow_digest: &[u8; DIGEST_BYTES],
        read: impl FnOnce() -> Result<
            StageOutcome<(crate::crash_amplification::RecordedFlowAndItsPlan, u64)>,
            String,
        >,
    ) -> Result<StageOutcome<Arc<crate::crash_amplification::RecordedFlowAndItsPlan>>, String> {
        self.library
            .open_library()
            .flows_read_back
            .held_or_read_once(flow_digest, read)
    }

    /// 这条连接从库里读回过几次流（留着的不算）：单测用。
    #[cfg(test)]
    pub(crate) fn recorded_flows_read_over_the_connection(&self) -> u64 {
        self.library
            .open_library()
            .flows_read_back
            .values_read_over_the_connection
            .load(Ordering::Relaxed)
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

/// 同 [`read_flow_facts_back`]，库只在读字节那一次往返里取：解码、核摘要在锁外。
///
/// # Errors
/// 同 [`read_flow_facts_back`]。
pub fn read_flow_facts_back_outside_the_lock(
    library: &LibraryTakenAtEachReadOrWrite<'_>,
    digest: &[u8; DIGEST_BYTES],
) -> Result<FlowFactsReadBack, String> {
    let bytes = library
        .take()
        .read_flow_facts(digest)
        .map_err(|error| format!("读事实表失败：{error:?}"))?
        .ok_or_else(|| "事实表不在库里：先跑录入段".to_string())?;
    assert_this_thread_holds_no_library_lock("核事实表的摘要、解事实表");
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
/// 同 [`read_packed_judge_tables_back`]，库只在读字节那一次往返里取：核摘要、解码在锁外（判表可到几百 MiB，别的线程不等它解）。
///
/// # Errors
/// 同 [`read_packed_judge_tables_back`]。
pub fn read_packed_judge_tables_back_outside_the_lock(
    library: &LibraryTakenAtEachReadOrWrite<'_>,
    digest: &[u8; DIGEST_BYTES],
) -> Result<PackedJudgeTablesReadBack, String> {
    let bytes = library
        .take()
        .read_packed_judge_tables(digest)
        .map_err(|error| format!("读 GPU 判表失败：{error:?}"))?
        .ok_or_else(|| "GPU 判表不在库里：先跑录入段".to_string())?;
    assert_this_thread_holds_no_library_lock("核判表的摘要、解读回的判表");
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

#[cfg(feature = "gpu")]
/// 同 [`record_the_judge_tables_and_read_them_back`]，锁只包写与读那两次往返：序列化、算摘要、核摘要与解码都在锁外（几条线程同时判给定状态表时互不等）。
///
/// # Errors
/// 同 [`record_the_judge_tables_and_read_them_back`]。
pub(crate) fn record_the_judge_tables_and_read_them_back_outside_the_lock(
    library: &LibraryOpenInThisProcess,
    tables: &PackedJudgeTables,
) -> Result<PackedJudgeTablesReadBack, String> {
    assert_this_thread_holds_no_library_lock("序列化判表");
    let bytes = tables.to_bytes();
    let digest = sha256_digest(&bytes);
    library
        .lock()
        .store_packed_judge_tables(&digest, &bytes)
        .map_err(|error| format!("GPU 判表写不进库：{error:?}"))?;
    drop(bytes);
    let bytes_read_back = library
        .lock()
        .read_packed_judge_tables(&digest)
        .map_err(|error| format!("读 GPU 判表失败：{error:?}"))?
        .ok_or_else(|| "GPU 判表不在库里：先跑录入段".to_string())?;
    assert_this_thread_holds_no_library_lock("核摘要、解判表");
    PackedJudgeTablesReadBack::from_the_library_bytes(&bytes_read_back, &digest)
}

#[cfg(feature = "gpu")]
/// 同 [`record_the_judge_tables_and_read_them_back`]，库每一次读写现取现放（[`LibraryTakenAtEachReadOrWrite`]）：序列化、算摘要、
/// 核摘要与解码都在锁外。判段在观察者交回的盘上判池级 checker 时经它。
///
/// # Errors
/// 同 [`record_the_judge_tables_and_read_them_back`]。
pub(crate) fn record_the_judge_tables_and_read_them_back_at_each_read_or_write(
    library: &LibraryTakenAtEachReadOrWrite<'_>,
    tables: &PackedJudgeTables,
) -> Result<PackedJudgeTablesReadBack, String> {
    assert_this_thread_holds_no_library_lock("序列化判段要入库的判表");
    let bytes = tables.to_bytes();
    let digest = sha256_digest(&bytes);
    library
        .take()
        .store_packed_judge_tables(&digest, &bytes)
        .map_err(|error| format!("GPU 判表写不进库：{error:?}"))?;
    drop(bytes);
    read_packed_judge_tables_back_outside_the_lock(library, &digest)
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

/// 对拍用例的输入清单里输入那一样的名字。
const PAIRING_MANIFEST_INPUT: &str = "input";

/// 从判定库里读回、核过摘要的一份对拍输入：对拍的两边只收它。字段私有，只有 [`PairingInputReadBack::from_the_library_bytes`] 造得出。
#[derive(Debug)]
pub struct PairingInputReadBack {
    bytes: Vec<u8>,
    digest: [u8; DIGEST_BYTES],
}

impl PairingInputReadBack {
    /// 库里按 `digest` 存的那份字节：核摘要。
    ///
    /// # Errors
    /// 字节的摘要对不上（库里那份被改过）。
    pub fn from_the_library_bytes(
        bytes: Vec<u8>,
        digest: &[u8; DIGEST_BYTES],
    ) -> Result<Self, String> {
        if sha256_digest(&bytes) != *digest {
            return Err("对拍输入摘要对不上：库里那份坏了，重跑录入段".to_string());
        }
        Ok(Self {
            bytes,
            digest: *digest,
        })
    }

    /// 读回的字节。
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// 这份输入的摘要（对拍结论的键里输入那一半，[`pairing_conclusion_reused_or_judged`]）。
    #[must_use]
    pub fn digest(&self) -> &[u8; DIGEST_BYTES] {
        &self.digest
    }
}

/// 按摘要从库里读对拍输入：读字节、核摘要。
///
/// # Errors
/// 库读失败；库里没有这个摘要；字节的摘要对不上。
pub fn read_pairing_input_back(
    store: &VerdictStore,
    digest: &[u8; DIGEST_BYTES],
) -> Result<PairingInputReadBack, String> {
    let bytes = store
        .read_pairing_input(digest)
        .map_err(|error| format!("读对拍输入失败：{error:?}"))?
        .ok_or_else(|| "对拍输入不在库里：先跑录入段".to_string())?;
    PairingInputReadBack::from_the_library_bytes(bytes, digest)
}

/// 输入是调用方给的字节的对拍用例用：录入段把输入按摘要写进库、按录入键（输入摘要）登清单；判段照同一份输入现算录入键、读清单、
/// 按摘要读回，对拍的两边只拿读回的那一份。只录那一趟交回 [`StageOutcome::RecordedOnly`]，用例拿到它就收工。
/// 与 [`facts_through_the_pairing_library`] 同一个走法，输入不限是事实表。
///
/// # Panics
/// 项名不合规矩；库开不了、写不进、读不回。
pub fn pairing_input_through_the_pairing_library(
    item_name: &str,
    input_bytes: &[u8],
) -> StageOutcome<PairingInputReadBack> {
    let item =
        ItemName::new(item_name).unwrap_or_else(|refusal| panic!("对拍用例的项名：{refusal}"));
    let library = CrashLibrary::open(CrashLibraryClass::InputPairing)
        .unwrap_or_else(|refusal| panic!("对拍用例「{item_name}」开不了库：{refusal}"));
    let recording = recording_key_of_the_inputs(&RecordingInputsDigest(sha256_digest(input_bytes)));
    pairing_input_through_the_library_under_the_recording_key(
        &library,
        RunStage::of_this_process(),
        &item,
        &recording,
        input_bytes,
    )
    .unwrap_or_else(|refusal| panic!("对拍用例「{item_name}」的输入过库失败：{refusal}"))
}

/// [`pairing_input_through_the_pairing_library`] 的本体：库、选段与录入键由调用方给（单测在一次性的库上跑）。
///
/// # Errors
/// 写不进库；登清单、读清单不成；库里没有清单指到的那一份；读回的字节摘要对不上。
pub fn pairing_input_through_the_library_under_the_recording_key(
    library: &CrashLibrary,
    stage: RunStage,
    item: &ItemName,
    recording: &RecordingKeyOfThisRun,
    input_bytes: &[u8],
) -> Result<StageOutcome<PairingInputReadBack>, String> {
    let digest = sha256_digest(input_bytes);
    two_stages_under_the_recording_key(
        library,
        stage,
        item,
        recording,
        |library_recorded_into| {
            library_recorded_into
                .lock()
                .store_pairing_input(&digest, input_bytes)
                .map_err(|error| format!("对拍输入写不进库：{error:?}"))?;
            Ok(ItemManifest {
                kind: "input_pairing".to_string(),
                digests: vec![(PAIRING_MANIFEST_INPUT.to_string(), digest)],
            })
        },
        |library_read_from, manifest| {
            Ok(StageOutcome::Judged(read_pairing_input_back(
                &library_read_from.lock(),
                &manifest.digest_named(PAIRING_MANIFEST_INPUT)?,
            )?))
        },
    )
}

/// 对拍用例的清单里参照那一样的名字。
const PAIRING_MANIFEST_REFERENCE: &str = "reference";

/// 从库里读回的对拍输入与参照一侧的结果（[`pairing_input_and_reference_through_the_pairing_library`] 判段交回的）。
#[derive(Debug)]
pub struct PairingInputAndReferenceReadBack {
    pub input: PairingInputReadBack,
    /// 参照一侧在录入段算出、写进库的结果。
    pub reference: PairingInputReadBack,
}

/// 对拍输入连同参照一侧的录入输入摘要：输入的摘要与参照一侧判法单元指纹拼起来取 SHA-256。参照的代码变了、或输入变了，录入键都变，
/// 库里旧的那一份参照不再被读到。
#[must_use]
pub fn recording_inputs_digest_of_the_pairing_input_and_the_reference_units(
    input_bytes: &[u8],
    reference_units: &JudgingUnitsKey,
) -> RecordingInputsDigest {
    RecordingInputsDigest(sha256_digest(
        format!(
            "singlefs pairing input with the reference\nreference_units={}\ninput={}\n",
            hexadecimal_text(reference_units.fingerprint()),
            hexadecimal_text(&sha256_digest(input_bytes))
        )
        .as_bytes(),
    ))
}

/// 一边是参照（在 CPU 上算）、一边要上卡的对拍用例用：录入段把输入写进库，调 `reference_of_the_input` 在 CPU 上算参照一侧的结果、也写进库，
/// 按录入键登清单；判段照同一份输入与参照一侧的判法单元现算录入键、读清单，输入与参照都从库里读回，持卡的那一段不再算参照。
/// 录入键照 [`recording_key_of_the_inputs`] 同一个算法，输入摘要取 [`recording_inputs_digest_of_the_pairing_input_and_the_reference_units`]；
/// `reference_units` 拒算时照录入函数单元拒算同样处置：键只拼输入的摘要、照录、不凭命中跳过。
/// 只录那一趟交回 [`StageOutcome::RecordedOnly`]，用例拿到它就收工。
///
/// # Panics
/// 项名不合规矩；库开不了、写不进、读不回。
pub fn pairing_input_and_reference_through_the_pairing_library(
    item_name: &str,
    input_bytes: &[u8],
    reference_units: &Result<JudgingUnitsKey, String>,
    reference_of_the_input: impl FnOnce(&[u8]) -> Vec<u8>,
) -> StageOutcome<PairingInputAndReferenceReadBack> {
    let item =
        ItemName::new(item_name).unwrap_or_else(|refusal| panic!("对拍用例的项名：{refusal}"));
    let library = CrashLibrary::open(CrashLibraryClass::InputPairing)
        .unwrap_or_else(|refusal| panic!("对拍用例「{item_name}」开不了库：{refusal}"));
    let recording = recording_key_of_the_pairing_input_and_the_reference_units(
        &judging_units_key(&JudgingUnits::Recording, toolchain_of_this_process())
            .map_err(|refused| refused.sentence()),
        input_bytes,
        reference_units,
    );
    pairing_input_and_reference_through_the_library_under_the_recording_key(
        &library,
        RunStage::of_this_process(),
        &item,
        &recording,
        input_bytes,
        reference_of_the_input,
    )
    .unwrap_or_else(|refusal| panic!("对拍用例「{item_name}」的输入与参照过库失败：{refusal}"))
}

/// [`pairing_input_and_reference_through_the_pairing_library`] 的录入键：`recording_units` 是录入函数单元那一把（单测造拒算用）。
#[must_use]
pub fn recording_key_of_the_pairing_input_and_the_reference_units(
    recording_units: &Result<JudgingUnitsKey, String>,
    input_bytes: &[u8],
    reference_units: &Result<JudgingUnitsKey, String>,
) -> RecordingKeyOfThisRun {
    match reference_units {
        Ok(reference_units) => recording_key_of_the_inputs_under(
            recording_units,
            &recording_inputs_digest_of_the_pairing_input_and_the_reference_units(
                input_bytes,
                reference_units,
            ),
        ),
        Err(sentence) => RecordingKeyOfThisRun {
            skips_the_recording_on_a_hit: false,
            refusal: Some(format!("参照一侧的判法单元拒算：{sentence}")),
            ..recording_key_of_the_inputs_under(
                recording_units,
                &RecordingInputsDigest(sha256_digest(input_bytes)),
            )
        },
    }
}

/// [`pairing_input_and_reference_through_the_pairing_library`] 的本体：库、选段与录入键由调用方给（单测在一次性的库上跑）。
///
/// # Errors
/// 写不进库；登清单、读清单不成；库里没有清单指到的那一份；读回的字节摘要对不上。
pub fn pairing_input_and_reference_through_the_library_under_the_recording_key(
    library: &CrashLibrary,
    stage: RunStage,
    item: &ItemName,
    recording: &RecordingKeyOfThisRun,
    input_bytes: &[u8],
    reference_of_the_input: impl FnOnce(&[u8]) -> Vec<u8>,
) -> Result<StageOutcome<PairingInputAndReferenceReadBack>, String> {
    let input_digest = sha256_digest(input_bytes);
    two_stages_under_the_recording_key(
        library,
        stage,
        item,
        recording,
        |library_recorded_into| {
            let reference_bytes = reference_of_the_input(input_bytes);
            let reference_digest = sha256_digest(&reference_bytes);
            let mut store = library_recorded_into.lock();
            store
                .store_pairing_input(&input_digest, input_bytes)
                .map_err(|error| format!("对拍输入写不进库：{error:?}"))?;
            store
                .store_pairing_input(&reference_digest, &reference_bytes)
                .map_err(|error| format!("对拍参照写不进库：{error:?}"))?;
            Ok(ItemManifest {
                kind: "input_pairing_with_the_reference".to_string(),
                digests: vec![
                    (PAIRING_MANIFEST_INPUT.to_string(), input_digest),
                    (PAIRING_MANIFEST_REFERENCE.to_string(), reference_digest),
                ],
            })
        },
        |library_read_from, manifest| {
            let store = library_read_from.lock();
            Ok(StageOutcome::Judged(PairingInputAndReferenceReadBack {
                input: read_pairing_input_back(
                    &store,
                    &manifest.digest_named(PAIRING_MANIFEST_INPUT)?,
                )?,
                reference: read_pairing_input_back(
                    &store,
                    &manifest.digest_named(PAIRING_MANIFEST_REFERENCE)?,
                )?,
            }))
        },
    )
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

/// 一条流按块对拍下来几块复用、几块现判、几块归别的份，与同一时刻手里最多压着几块的结论（宿主内存只随它涨，不随整条流的块数涨）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PairingBlocksOfTheFlow {
    pub blocks_reused: u64,
    pub blocks_judged_now: u64,
    /// 归别的份的块（[`PairingBlocksShare`]）：这一份不读库、不判、不折进总账；不是 0 时总账不是整条流的。
    pub blocks_left_to_other_shares: u64,
    /// 落在这一趟走的组之外的块（[`PairingBlocksShare::only_the_groups_of_pairing_blocks_in`]）：同样不读库、不判、不折进总账；
    /// 不是 0 时总账只是那几组的状态，不是整条流的。
    pub blocks_outside_the_groups_of_this_run: u64,
    pub most_block_conclusions_held_at_once: usize,
}

/// 判一段连着的块：交这一段里每块的序号段，按同一次序交回每块的结论字节与两边一致不一致；交 `None` 是判那一侧报「本次未跑」。
/// 几段在各自的线程上同时判（[`pairing_conclusions_of_the_blocks_reused_or_judged`]）。
pub type JudgeOfAStretchOfPairingBlocks<'closure> =
    dyn Fn(&[std::ops::Range<u64>]) -> Option<Vec<PairingJudged>> + Sync + 'closure;

/// 这一趟领一条流的哪些块（几台机器分着判同一条流）：块按每段 `most_blocks_judged_together` 块分成组，一组归一份（判一段要连着的块，
/// 一块一块轮着分，CPU 那一侧每块都要重建一次撕裂表）；哪组归哪一份只由权重表、流的输入摘要与组的序号定，几份各算各的、不用商量，
/// 每一块恰好归一份。块键（[`pairing_block_inputs_digest`]）不带份额：哪一份判的，判出的块照同一把键进同一份库，合并那一趟按整份（`0/1`）全部复用。
/// 流的输入摘要只错开各条流从哪一份轮起：一块的小流不会全落在同一份上。
/// 另可只走一段组（[`Self::only_the_groups_of_pairing_blocks_in`]：一条流按状态序号区间拆成几条用例，每条只领自己那一段序号的组），
/// 与几台分份叠用：组落在这一段里、又归这一份，才算领。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PairingBlocksShare {
    share_of_this_run: crate::crash_amplification::HostShare,
    weights_of_the_shares: crate::crash_amplification::WeightedShares,
    groups_of_this_run: GroupsOfPairingBlocksOfThisRun,
}

/// 一趟走一条流的哪几组块。
#[derive(Clone, Debug, PartialEq, Eq)]
enum GroupsOfPairingBlocksOfThisRun {
    EveryGroup,
    /// 组的序号落在这个半开区间里的。
    OnlyTheGroupsIn(std::ops::Range<u64>),
}

/// 状态序号区间 `ordinals` 落在一条流的哪几组块上（块宽 `states_per_block`、一组 `blocks_per_group` 块，组从序号 0 起排，同
/// [`pairing_conclusions_of_the_blocks_reused_or_judged`] 的分组）：交回组的半开区间。区间两头要落在组的边界上，终点另可以是整条流的状态数
/// `state_count`（最后一组不满）；不然领整组时会判到区间外的状态、或漏判区间里的，交 `Err`。
///
/// # Errors
/// 区间是空的、终点越过 `state_count`、起点不在组的边界上、终点既不在组的边界上也不是 `state_count`，或一组的状态数装不进 u64。
pub fn groups_of_pairing_blocks_holding_the_states(
    ordinals: &std::ops::Range<u64>,
    state_count: u64,
    states_per_block: std::num::NonZeroU64,
    blocks_per_group: std::num::NonZeroUsize,
) -> Result<std::ops::Range<u64>, String> {
    let states_per_group = u64::try_from(blocks_per_group.get())
        .ok()
        .and_then(|blocks| blocks.checked_mul(states_per_block.get()))
        .ok_or_else(|| {
            format!(
                "一组 {blocks_per_group} 块、一块 {states_per_block} 个状态：一组的状态数装不进 u64"
            )
        })?;
    if ordinals.start >= ordinals.end || ordinals.end > state_count {
        return Err(format!(
            "状态序号区间 {ordinals:?} 是空的或越过了整条流的 {state_count} 个状态"
        ));
    }
    if !ordinals.start.is_multiple_of(states_per_group) {
        return Err(format!(
            "状态序号区间 {ordinals:?} 的起点不在组的边界上（一组 {states_per_group} 个状态）"
        ));
    }
    if !ordinals.end.is_multiple_of(states_per_group) && ordinals.end != state_count {
        return Err(format!(
            "状态序号区间 {ordinals:?} 的终点既不在组的边界上（一组 {states_per_group} 个状态）、也不是整条流的状态数 {state_count}"
        ));
    }
    Ok(ordinals.start / states_per_group..ordinals.end.div_ceil(states_per_group))
}

impl PairingBlocksShare {
    /// 整条流都归这一趟（`0/1`）。
    #[must_use]
    pub fn whole_flow_in_one_share() -> Self {
        Self::with_this_share_of_these_weights(
            crate::crash_amplification::HostShare { index: 0, count: 1 },
            vec![1],
        )
    }

    /// 同 `self` 那一份，另只走组的序号落在 `groups` 里的组（替掉之前给过的组区间）；组区间由
    /// [`groups_of_pairing_blocks_holding_the_states`] 从状态序号区间算。
    #[must_use]
    pub fn only_the_groups_of_pairing_blocks_in(self, groups: std::ops::Range<u64>) -> Self {
        Self {
            share_of_this_run: self.share_of_this_run,
            weights_of_the_shares: self.weights_of_the_shares,
            groups_of_this_run: GroupsOfPairingBlocksOfThisRun::OnlyTheGroupsIn(groups),
        }
    }

    /// 第 `group_of_blocks` 组落不落在这一趟走的组里（没给组区间的一趟每一组都落在里面）。
    #[must_use]
    pub fn groups_of_this_run_hold(&self, group_of_blocks: u64) -> bool {
        match &self.groups_of_this_run {
            GroupsOfPairingBlocksOfThisRun::EveryGroup => true,
            GroupsOfPairingBlocksOfThisRun::OnlyTheGroupsIn(groups) => {
                groups.contains(&group_of_blocks)
            }
        }
    }

    /// 领 `share_of_this_run` 这一份，几份的权重是 `weights_of_the_shares`。
    ///
    /// # Panics
    /// 权重的个数与份数不同；权重全是 0。
    #[must_use]
    pub fn with_this_share_of_these_weights(
        share_of_this_run: crate::crash_amplification::HostShare,
        weights_of_the_shares: Vec<u64>,
    ) -> Self {
        assert_eq!(
            weights_of_the_shares.len(),
            share_of_this_run.count,
            "每一份一个权重"
        );
        Self {
            share_of_this_run,
            weights_of_the_shares: crate::crash_amplification::WeightedShares::new(
                weights_of_the_shares,
            ),
            groups_of_this_run: GroupsOfPairingBlocksOfThisRun::EveryGroup,
        }
    }

    /// 这个进程领哪一份：与崩溃放量同一对变量（`crate::crash_amplification::host_share_and_weights_from_the_environment`），没设是整份。
    ///
    /// # Errors
    /// 份额或权重写错。
    pub fn of_this_process() -> Result<Self, String> {
        let (share_of_this_run, weights_of_the_shares) =
            crate::crash_amplification::host_share_and_weights_from_the_environment()?;
        Ok(Self::with_this_share_of_these_weights(
            share_of_this_run,
            weights_of_the_shares,
        ))
    }

    /// 输入摘要是 `flow_inputs_digest` 的流第 `group_of_blocks` 组块归不归这一趟：落在这一趟走的组里（[`Self::groups_of_this_run_hold`]）、
    /// 又归这一份。
    #[must_use]
    pub fn claims_this_group_of_pairing_blocks(
        &self,
        flow_inputs_digest: &[u8; DIGEST_BYTES],
        group_of_blocks: u64,
    ) -> bool {
        let where_the_flow_starts_in_the_rotation = u64::from(u32::from_le_bytes(
            flow_inputs_digest[..4]
                .try_into()
                .expect("摘要不止 4 个字节"),
        ));
        self.groups_of_this_run_hold(group_of_blocks)
            && self
                .weights_of_the_shares
                .owner_of_block(where_the_flow_starts_in_the_rotation + group_of_blocks)
                == self.share_of_this_run.index
    }
}

/// 判一段块的那条线程收尾时交回的。
struct FinishedPairingStretch {
    started_as: u64,
    /// 判那一段 panic 了是 `Err`（收的那一头等在飞的都收尾之后接着往外抛）。
    judged: std::thread::Result<Option<Vec<PairingJudged>>>,
}

/// 等着按序号次序折进总账的一项。
enum PairingBlocksAwaitingTheLedger {
    Reused {
        ordinals: std::ops::Range<u64>,
        conclusion: Vec<u8>,
    },
    /// `conclusions` 是 `None`：这一段还在判，或判那一侧报了「本次未跑」。
    Stretch {
        started_as: u64,
        blocks: Vec<std::ops::Range<u64>>,
        conclusions: Option<Vec<PairingJudged>>,
    },
}

/// 判那一侧为什么不再往下判了。
enum PairingJudgingStopped {
    /// 有一段交了 `None`（判那一侧报「本次未跑」）：那一段一块都不存。
    NotRun,
    /// 有一段 panic 了：照原样接着往外抛。
    Panicked(Box<dyn std::any::Any + Send>),
}

/// 一条流这一趟正在判的那几段与等着折进总账的块：只在起 [`pairing_conclusions_of_the_blocks_reused_or_judged`] 的那条线程上动。
struct PairingStretchesOfTheRun<'run> {
    library: &'run CrashLibrary,
    key_of_the_block: &'run dyn Fn(&std::ops::Range<u64>) -> Option<PairingConclusionKey>,
    take_a_block_conclusion: &'run mut dyn FnMut(&std::ops::Range<u64>, &[u8]),
    blocks_of_the_flow: PairingBlocksOfTheFlow,
    stretches_judged_at_once: usize,
    /// 等着折进总账的块（在判的算上）至多这么多：头上那一段迟迟判不完时，走块那一头停下来等，身后复用的、判完的结论不无限堆。
    most_blocks_awaiting_the_ledger: usize,
    awaiting_the_ledger: VecDeque<PairingBlocksAwaitingTheLedger>,
    stretches_started: u64,
    stretches_finished: u64,
    stretches_in_flight: usize,
    judging_stopped: Option<PairingJudgingStopped>,
    finished_stretches: mpsc::Receiver<FinishedPairingStretch>,
}

impl PairingStretchesOfTheRun<'_> {
    /// 此刻至多几段一起判：头一段判完之前只有它一段（录入段在它里面走完、判表在它里面过库，后面几段读的是录好的），之后 `stretches_judged_at_once` 段。
    fn stretches_allowed_in_flight_now(&self) -> usize {
        if self.stretches_finished == 0 {
            1
        } else {
            self.stretches_judged_at_once
        }
    }

    /// 等一段判完：判出来的逐块存进库（后面的段先判完也当场存，存进库的不等总账），记进等着折进总账的那一项，能折的折掉。
    ///
    /// # Errors
    /// 库写失败。
    ///
    /// # Panics
    /// 交回的结论块数与交出去的块数不同。
    fn wait_for_a_pairing_stretch_to_finish(&mut self) -> Result<(), String> {
        let FinishedPairingStretch {
            started_as: finished_stretch,
            judged,
        } = self
            .finished_stretches
            .recv()
            .expect("判一段的线程收尾都交一回（panic 也接住交），在飞的段不是 0 才等");
        self.stretches_in_flight -= 1;
        self.stretches_finished += 1;
        let conclusions = match judged {
            Ok(Some(conclusions)) => conclusions,
            Ok(None) => {
                if self.judging_stopped.is_none() {
                    self.judging_stopped = Some(PairingJudgingStopped::NotRun);
                }
                return Ok(());
            }
            Err(panicked) => {
                self.judging_stopped = Some(PairingJudgingStopped::Panicked(panicked));
                return Ok(());
            }
        };
        let awaiting_it = self
            .awaiting_the_ledger
            .iter_mut()
            .find_map(|awaiting| match awaiting {
                PairingBlocksAwaitingTheLedger::Stretch {
                    started_as,
                    blocks,
                    conclusions: conclusions_of_the_stretch,
                } if *started_as == finished_stretch => Some((blocks, conclusions_of_the_stretch)),
                PairingBlocksAwaitingTheLedger::Stretch { .. }
                | PairingBlocksAwaitingTheLedger::Reused { .. } => None,
            })
            .expect("起一段时把它排进了等着折进总账的那一队，折掉之前它已判完");
        let (blocks, conclusions_awaiting) = awaiting_it;
        assert_eq!(
            conclusions.len(),
            blocks.len(),
            "判一段块交回的结论块数与交出去的块数相同"
        );
        store_the_judged_pairing_blocks(self.library, self.key_of_the_block, blocks, &conclusions)?;
        *conclusions_awaiting = Some(conclusions);
        self.note_the_block_conclusions_held();
        self.take_the_blocks_at_the_head_of_the_queue();
        Ok(())
    }

    /// 手里此刻压着几块的结论（判完、复用的还没轮到折进总账的），记下最多的一次。
    fn note_the_block_conclusions_held(&mut self) {
        let held: usize = self
            .awaiting_the_ledger
            .iter()
            .map(|awaiting| match awaiting {
                PairingBlocksAwaitingTheLedger::Reused { .. } => 1,
                PairingBlocksAwaitingTheLedger::Stretch { conclusions, .. } => {
                    conclusions.as_ref().map_or(0, Vec::len)
                }
            })
            .sum();
        self.blocks_of_the_flow.most_block_conclusions_held_at_once = self
            .blocks_of_the_flow
            .most_block_conclusions_held_at_once
            .max(held);
    }

    /// 队头有结论的逐块按序号次序折进总账，折完就丢；队头那一段还在判（或报了本次未跑）就停。
    fn take_the_blocks_at_the_head_of_the_queue(&mut self) {
        // 迭代上界是队长；跨轮只有队在变短
        while let Some(head) = self.awaiting_the_ledger.front() {
            match head {
                PairingBlocksAwaitingTheLedger::Stretch {
                    conclusions: None, ..
                } => return,
                PairingBlocksAwaitingTheLedger::Stretch {
                    conclusions: Some(_),
                    ..
                }
                | PairingBlocksAwaitingTheLedger::Reused { .. } => {}
            }
            match self
                .awaiting_the_ledger
                .pop_front()
                .expect("队头刚看过，还在")
            {
                PairingBlocksAwaitingTheLedger::Reused {
                    ordinals,
                    conclusion,
                } => {
                    (self.take_a_block_conclusion)(&ordinals, &conclusion);
                    self.blocks_of_the_flow.blocks_reused += 1;
                }
                PairingBlocksAwaitingTheLedger::Stretch {
                    blocks,
                    conclusions,
                    ..
                } => {
                    for (ordinals, judged) in blocks.iter().zip(conclusions.iter().flatten()) {
                        (self.take_a_block_conclusion)(ordinals, &judged.conclusion);
                        self.blocks_of_the_flow.blocks_judged_now += 1;
                    }
                }
            }
        }
    }

    /// 等到队里放得下 `blocks` 块（在飞的段判完腾地方）。不再往下判了就不等。
    ///
    /// # Errors
    /// 库写失败。
    fn wait_for_room_for_blocks_awaiting_the_ledger(
        &mut self,
        blocks: usize,
    ) -> Result<(), String> {
        // 迭代上界是在飞的段数：每一轮等完一段；在飞的是 0 时队已折空（队头只可能是在判的段）
        while self.judging_stopped.is_none()
            && self.stretches_in_flight > 0
            && self.blocks_awaiting_the_ledger() + blocks > self.most_blocks_awaiting_the_ledger
        {
            self.wait_for_a_pairing_stretch_to_finish()?;
        }
        Ok(())
    }

    fn blocks_awaiting_the_ledger(&self) -> usize {
        self.awaiting_the_ledger
            .iter()
            .map(|awaiting| match awaiting {
                PairingBlocksAwaitingTheLedger::Reused { .. } => 1,
                PairingBlocksAwaitingTheLedger::Stretch { blocks, .. } => blocks.len(),
            })
            .sum()
    }

    /// 等到在飞的段少于此刻许的段数。不再往下判了就不等。
    ///
    /// # Errors
    /// 库写失败。
    fn wait_for_a_free_place_among_the_stretches_in_flight(&mut self) -> Result<(), String> {
        // 迭代上界是在飞的段数：每一轮等完一段
        while self.judging_stopped.is_none()
            && self.stretches_in_flight >= self.stretches_allowed_in_flight_now()
        {
            self.wait_for_a_pairing_stretch_to_finish()?;
        }
        Ok(())
    }

    /// 等在飞的段都收尾（不再往下判了也等：它们判完的照样存进库）。
    ///
    /// # Errors
    /// 库写失败。
    fn wait_for_every_stretch_in_flight(&mut self) -> Result<(), String> {
        // 迭代上界是在飞的段数：每一轮等完一段
        while self.stretches_in_flight > 0 {
            self.wait_for_a_pairing_stretch_to_finish()?;
        }
        Ok(())
    }
}

/// 把攒着的那一段连着的块排进队、在一条线程上判（没攒着块、已经不再往下判了就只清空攒着的）；此刻在飞的段满了先等，起完再满了也等：
/// 一次只许一段时，走下一块之前这一段已经判完、存进库、折进总账。
///
/// # Errors
/// 库写失败。
fn start_judging_the_pending_pairing_blocks<'scope, 'environment: 'scope>(
    scope: &'scope std::thread::Scope<'scope, 'environment>,
    stretches: &mut PairingStretchesOfTheRun<'_>,
    pending_blocks: &mut Vec<std::ops::Range<u64>>,
    judge_the_blocks: &'environment JudgeOfAStretchOfPairingBlocks<'environment>,
    finished_stretches: &mpsc::Sender<FinishedPairingStretch>,
) -> Result<(), String> {
    if pending_blocks.is_empty() {
        return Ok(());
    }
    stretches.wait_for_a_free_place_among_the_stretches_in_flight()?;
    stretches.wait_for_room_for_blocks_awaiting_the_ledger(pending_blocks.len())?;
    if stretches.judging_stopped.is_some() {
        pending_blocks.clear();
        return Ok(());
    }
    let blocks = std::mem::take(pending_blocks);
    let blocks_of_the_thread = blocks.clone();
    let started_as = stretches.stretches_started;
    stretches.stretches_started += 1;
    stretches.stretches_in_flight += 1;
    stretches
        .awaiting_the_ledger
        .push_back(PairingBlocksAwaitingTheLedger::Stretch {
            started_as,
            blocks,
            conclusions: None,
        });
    let finished_stretches = finished_stretches.clone();
    scope.spawn(move || {
        // 接住 panic 交回去：收的那一头等在飞的都收尾之后照原样接着抛，判红的消息不变
        let judged = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            judge_the_blocks(&blocks_of_the_thread)
        }));
        finished_stretches
            .send(FinishedPairingStretch { started_as, judged })
            .expect("收的那一头活到这个作用域收尾之后");
    });
    stretches.wait_for_a_free_place_among_the_stretches_in_flight()
}

/// 判完的一段逐块存进库（键算不出的，即两边哪一边拒算，不存）。
///
/// # Errors
/// 库写失败。
fn store_the_judged_pairing_blocks(
    library: &CrashLibrary,
    key_of_the_block: &dyn Fn(&std::ops::Range<u64>) -> Option<PairingConclusionKey>,
    blocks: &[std::ops::Range<u64>],
    conclusions: &[PairingJudged],
) -> Result<(), String> {
    for (ordinals, judged) in blocks.iter().zip(conclusions) {
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
    }
    Ok(())
}

/// 按序号次序把 `[0, state_count)` 按 `states_per_block` 切块走一遍：落在这一趟走的组之外的、归别的份的各数各的、不碰（前面攒着的先交出去判，一段块要连着）；
/// 库里有两边一致的结论就复用，没有的攒成一段连着的块交出去判。判那一侧不再往下判了就收手。
///
/// # Errors
/// 库读写失败。
#[allow(
    clippy::too_many_arguments,
    reason = "走一遍块的输入就是这几样：作用域、这一趟的账、流的输入摘要、状态数、块宽、一次判几块、这一趟领哪些块、判一段块、交回判完的那一头；收成一个类型只为让签名变短"
)]
fn walk_the_pairing_blocks_of_the_flow<'scope, 'environment: 'scope>(
    scope: &'scope std::thread::Scope<'scope, 'environment>,
    stretches: &mut PairingStretchesOfTheRun<'_>,
    flow_inputs_digest: &[u8; DIGEST_BYTES],
    state_count: u64,
    states_per_block: std::num::NonZeroU64,
    most_blocks_judged_together: std::num::NonZeroUsize,
    share: &PairingBlocksShare,
    judge_the_blocks: &'environment JudgeOfAStretchOfPairingBlocks<'environment>,
    finished_stretches: &mpsc::Sender<FinishedPairingStretch>,
) -> Result<(), String> {
    let blocks_of_a_group =
        u64::try_from(most_blocks_judged_together.get()).expect("一次判的块数装得进 u64");
    let mut pending_blocks: Vec<std::ops::Range<u64>> =
        Vec::with_capacity(most_blocks_judged_together.get());
    let mut block_sequence = 0u64;
    let mut block_start = 0u64;
    while block_start < state_count && stretches.judging_stopped.is_none() {
        let ordinals = block_start
            ..block_start
                .saturating_add(states_per_block.get())
                .min(state_count);
        block_start = ordinals.end;
        let group_of_blocks = block_sequence / blocks_of_a_group;
        block_sequence += 1;
        if !share.groups_of_this_run_hold(group_of_blocks) {
            start_judging_the_pending_pairing_blocks(
                scope,
                stretches,
                &mut pending_blocks,
                judge_the_blocks,
                finished_stretches,
            )?;
            stretches
                .blocks_of_the_flow
                .blocks_outside_the_groups_of_this_run += 1;
            continue;
        }
        if !share.claims_this_group_of_pairing_blocks(flow_inputs_digest, group_of_blocks) {
            start_judging_the_pending_pairing_blocks(
                scope,
                stretches,
                &mut pending_blocks,
                judge_the_blocks,
                finished_stretches,
            )?;
            stretches.blocks_of_the_flow.blocks_left_to_other_shares += 1;
            continue;
        }
        let stored_conclusion = match (stretches.key_of_the_block)(&ordinals) {
            Some(key) => stretches
                .library
                .lock()
                .read_pairing_conclusion(&key)
                .map_err(|error| format!("读对拍块结论失败：{error:?}"))?
                .filter(stored_pairing_conclusion_is_reused),
            None => None,
        };
        if let Some(stored) = stored_conclusion {
            // 复用的这一块排在攒着的那一段后面：总账按序号次序收
            start_judging_the_pending_pairing_blocks(
                scope,
                stretches,
                &mut pending_blocks,
                judge_the_blocks,
                finished_stretches,
            )?;
            stretches.wait_for_room_for_blocks_awaiting_the_ledger(1)?;
            stretches
                .awaiting_the_ledger
                .push_back(PairingBlocksAwaitingTheLedger::Reused {
                    ordinals,
                    conclusion: stored.conclusion,
                });
            stretches.take_the_blocks_at_the_head_of_the_queue();
            stretches.note_the_block_conclusions_held();
            continue;
        }
        pending_blocks.push(ordinals);
        if pending_blocks.len() >= most_blocks_judged_together.get() {
            start_judging_the_pending_pairing_blocks(
                scope,
                stretches,
                &mut pending_blocks,
                judge_the_blocks,
                finished_stretches,
            )?;
        }
    }
    start_judging_the_pending_pairing_blocks(
        scope,
        stretches,
        &mut pending_blocks,
        judge_the_blocks,
        finished_stretches,
    )
}

/// 按块对拍（「必须要剪枝和复用」）：`[0, state_count)` 按 `states_per_block` 切块，按序号次序一块块走——不归 `share` 这一份的只数、不碰；
/// 库里有这一块两边一致的结论（键是 [`pairing_block_inputs_digest`] 与两边判法单元指纹）就复用、不判；记着不一致（红）的与没有的一样现判、照同一把键
/// 存回（红的照样存：记红、换掉原有的那一行）。没有的攒成一段连着的块（至多 `most_blocks_judged_together` 块），
/// 在一条线程上交 `judge_the_blocks` 一次判完，至多 `stretches_judged_at_once` 段一起判（头一段判完之前只有它一段，
/// [`PairingStretchesOfTheRun::stretches_allowed_in_flight_now`]）；判完逐块存进库。每一块的结论（复用的与现判的）按序号次序交
/// `take_a_block_conclusion` 折进调用方的总账，交完就丢：手里同一时刻至多压着 `stretches_judged_at_once` × `most_blocks_judged_together` 块的结论，
/// 与整条流有几块无关。两边哪一边的判法单元拒算：照判、不入库（不退回整文件指纹、不跳过）。有一段交 `None`（判那一侧报「本次未跑」）：不再起新的段，
/// 在飞的等它们收尾、判完的照存，交 `Ok(None)`，交 `None` 的那一段一块都不存。有一段 panic：同样等在飞的收尾，再照原样往外抛。
///
/// # Errors
/// 库读写失败。
///
/// # Panics
/// `judge_the_blocks` 交回的结论块数与交给它的块数不同；`judge_the_blocks` panic。
#[allow(
    clippy::too_many_arguments,
    reason = "按块对拍的输入就是这几样：库、流的输入摘要、状态数、块宽、一次判几块、几段一起判、这一趟领哪些块、两边判法单元、判一段块、收一块结论；收成一个类型只为让签名变短"
)]
pub fn pairing_conclusions_of_the_blocks_reused_or_judged(
    library: &CrashLibrary,
    flow_inputs_digest: &[u8; DIGEST_BYTES],
    state_count: u64,
    states_per_block: std::num::NonZeroU64,
    most_blocks_judged_together: std::num::NonZeroUsize,
    stretches_judged_at_once: std::num::NonZeroUsize,
    share: &PairingBlocksShare,
    one_side: &Result<JudgingUnitsKey, String>,
    other_side: &Result<JudgingUnitsKey, String>,
    judge_the_blocks: &JudgeOfAStretchOfPairingBlocks<'_>,
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
    let (finished_stretches_sender, finished_stretches) = mpsc::channel();
    let mut stretches = PairingStretchesOfTheRun {
        library,
        key_of_the_block: &key_of_the_block,
        take_a_block_conclusion,
        blocks_of_the_flow: PairingBlocksOfTheFlow {
            blocks_reused: 0,
            blocks_judged_now: 0,
            blocks_left_to_other_shares: 0,
            blocks_outside_the_groups_of_this_run: 0,
            most_block_conclusions_held_at_once: 0,
        },
        stretches_judged_at_once: stretches_judged_at_once.get(),
        most_blocks_awaiting_the_ledger: stretches_judged_at_once.get()
            * most_blocks_judged_together.get(),
        awaiting_the_ledger: VecDeque::new(),
        stretches_started: 0,
        stretches_finished: 0,
        stretches_in_flight: 0,
        judging_stopped: None,
        finished_stretches,
    };
    std::thread::scope(|scope| {
        let walked = walk_the_pairing_blocks_of_the_flow(
            scope,
            &mut stretches,
            flow_inputs_digest,
            state_count,
            states_per_block,
            most_blocks_judged_together,
            share,
            judge_the_blocks,
            &finished_stretches_sender,
        );
        walked.and(stretches.wait_for_every_stretch_in_flight())
    })?;
    match stretches.judging_stopped {
        Some(PairingJudgingStopped::Panicked(panicked)) => std::panic::resume_unwind(panicked),
        Some(PairingJudgingStopped::NotRun) => Ok(None),
        None => Ok(Some(stretches.blocks_of_the_flow)),
    }
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

    /// 标签长到整串拼出的单段名超一段路径名的上限（带三个字节一个的中文）：照样建在临时目录下一层、单段名不超上限；
    /// 只在截掉的那一段不同的两个标签，名字去掉 `-<进程号>-<进程内序号>` 之后仍不同；截短点落在中文字符中间时退到字符边界。
    /// 判别力（`crates/mutations.tsv`）：不截短、整串照拼，建不出（ENAMETOOLONG）这条红；截短后不接标签摘要，两个名字相同这条红；
    /// 截短点不退到字符边界，切片切在字符中间 panic，这条红。
    #[test]
    fn overlong_multibyte_label_is_cut_to_the_path_component_limit_and_labels_differing_only_beyond_the_cut_get_distinct_directories(
    ) {
        let shared_head = "崩溃判定库一次性目录".repeat(30);
        let label_ending_in_east = format!("{shared_head}东");
        let label_ending_in_west = format!("{shared_head}西");
        assert!(
            format!("singlefs-{label_ending_in_east}").len() > MOST_BYTES_OF_ONE_PATH_COMPONENT,
            "标签整串拼出的名字超一段路径名的上限"
        );
        let east =
            ExclusiveTemporaryDirectory::create(&label_ending_in_east).expect("超长的标签也建得出");
        let west = ExclusiveTemporaryDirectory::create(&label_ending_in_west)
            .expect("另一个超长的标签也建得出");
        let name_of = |directory: &ExclusiveTemporaryDirectory| -> String {
            directory
                .path()
                .file_name()
                .expect("目录有名字")
                .to_str()
                .expect("名字是 UTF-8")
                .to_owned()
        };
        for directory in [&east, &west] {
            assert_eq!(
                directory.path().parent(),
                Some(std::env::temp_dir().as_path()),
                "建在临时目录下一层：{}",
                directory.path().display()
            );
            assert!(
                name_of(directory).len() <= MOST_BYTES_OF_ONE_PATH_COMPONENT,
                "单段名不超一段路径名的上限：{} 字节",
                name_of(directory).len()
            );
            assert!(
                name_of(directory).starts_with("singlefs-崩溃判定库"),
                "截短的名字留着标签开头：{}",
                name_of(directory)
            );
        }
        let without_process_and_serial = |name: &str| -> String {
            name.rsplitn(3, '-')
                .nth(2)
                .expect("名字以 -<进程号>-<进程内序号> 收尾")
                .to_owned()
        };
        assert_ne!(
            without_process_and_serial(&name_of(&east)),
            without_process_and_serial(&name_of(&west)),
            "只在截掉的那一段不同的两个标签，名字不同"
        );
        // 进程号各差一位，后缀各差一个字节：截短点落在三个字节一个的中文的三个位置上（字符边界与字符中间两处）
        for process_identifier in [7, 70, 700] {
            let name =
                exclusive_temporary_directory_name(&label_ending_in_east, process_identifier, 1);
            assert!(
                name.len() <= MOST_BYTES_OF_ONE_PATH_COMPONENT,
                "进程号 {process_identifier}：单段名不超上限，{} 字节",
                name.len()
            );
            assert!(
                name.ends_with(&format!("-{process_identifier}-1")),
                "进程号 {process_identifier}：名字以 -<进程号>-<进程内序号> 收尾：{name}"
            );
        }
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
            CrashLibraryClass::InputPairing,
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

    /// 对拍输入两段过库：只录那一趟写进库、不交回输入；只判那一趟照同一份输入现算录入键、从库里读回同样的字节与摘要；
    /// 没录过的另一份输入只判时报库里没有清单。
    /// 判别力：只判那一趟不读库、直接交回调用方手里的字节，没录过的那一份就不报错，红在最后一条断言。
    #[test]
    fn a_pairing_input_is_recorded_in_one_run_and_read_back_from_the_library_in_another() {
        // 一次性库留着：自证对拍输入只录、只判两趟：要一份空的临时服务
        let disposable =
            DisposableLibrary::new("input-pairing").unwrap_or_else(|refusal| panic!("{refusal}"));
        let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
        let library = CrashLibrary::open(CrashLibraryClass::InputPairing).expect("开得了库");
        let units = Ok(JudgingUnitsKey::of_the_parts(
            [9; DIGEST_BYTES],
            vec![JudgingUnitFingerprint {
                unit: "record_into".to_string(),
                fingerprint: [10; DIGEST_BYTES],
            }],
        ));
        let item = ItemName::new("input-pairing").expect("项名合规矩");
        let recorded_input = b"units to verify and their expected checksums".to_vec();
        let recording_key_of = |input: &[u8]| {
            recording_key_of_the_inputs_under(&units, &RecordingInputsDigest(sha256_digest(input)))
        };
        let recorded = pairing_input_through_the_library_under_the_recording_key(
            &library,
            RunStage::RecordOnly,
            &item,
            &recording_key_of(&recorded_input),
            &recorded_input,
        )
        .expect("只录那一趟写得进库");
        assert!(
            matches!(recorded, StageOutcome::RecordedOnly),
            "只录那一趟不交回输入：{recorded:?}"
        );
        let judged = pairing_input_through_the_library_under_the_recording_key(
            &library,
            RunStage::JudgeOnly,
            &item,
            &recording_key_of(&recorded_input),
            &recorded_input,
        )
        .expect("只判那一趟读得回");
        let StageOutcome::Judged(read_back) = judged else {
            panic!("只判那一趟交回读回的输入：{judged:?}")
        };
        assert_eq!(read_back.bytes(), recorded_input.as_slice());
        assert_eq!(read_back.digest(), &sha256_digest(&recorded_input));
        let never_recorded = b"another input nobody recorded".to_vec();
        assert!(
            pairing_input_through_the_library_under_the_recording_key(
                &library,
                RunStage::JudgeOnly,
                &item,
                &recording_key_of(&never_recorded),
                &never_recorded,
            )
            .is_err(),
            "没录过的输入只判：库里没有它的清单，报错、不现录"
        );
    }

    /// 对拍输入连同参照两段过库：只录那一趟算一次参照、写进库；只判那一趟不算参照，输入与参照都从库里读回；同一把键再录一趟命中、不再算参照；
    /// 参照一侧的判法单元换了一把，录入键变，只判读不到旧的那一份。
    /// 判别力：录入键不拼参照一侧的单元指纹，换了一把之后只判照样读到旧参照，红在最后一条断言；判段现算参照，「只判那一趟不算参照」那一条红。
    #[test]
    fn a_pairing_reference_is_computed_once_at_the_recording_and_keyed_by_the_reference_units() {
        // 一次性库留着：自证对拍参照只录、只判两趟：要一份空的临时服务
        let disposable = DisposableLibrary::new("input-pairing-reference")
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
        let library = CrashLibrary::open(CrashLibraryClass::InputPairing).expect("开得了库");
        let units_fingerprinted = |fingerprint: u8| {
            Ok(JudgingUnitsKey::of_the_parts(
                [fingerprint; DIGEST_BYTES],
                vec![JudgingUnitFingerprint {
                    unit: "unit".to_string(),
                    fingerprint: [fingerprint; DIGEST_BYTES],
                }],
            ))
        };
        let recording_units = units_fingerprinted(9);
        let item = ItemName::new("input-pairing-reference").expect("项名合规矩");
        let input = b"units to verify".to_vec();
        let times_the_reference_was_computed = std::cell::Cell::new(0_u32);
        let through_the_library =
            |stage: RunStage, reference_units: &Result<JudgingUnitsKey, String>| {
                pairing_input_and_reference_through_the_library_under_the_recording_key(
                    &library,
                    stage,
                    &item,
                    &recording_key_of_the_pairing_input_and_the_reference_units(
                        &recording_units,
                        &input,
                        reference_units,
                    ),
                    &input,
                    |input_bytes| {
                        times_the_reference_was_computed
                            .set(times_the_reference_was_computed.get() + 1);
                        input_bytes.iter().rev().copied().collect()
                    },
                )
            };
        let reference_units = units_fingerprinted(11);
        let recorded = through_the_library(RunStage::RecordOnly, &reference_units)
            .expect("只录那一趟写得进库");
        assert!(matches!(recorded, StageOutcome::RecordedOnly));
        assert_eq!(
            times_the_reference_was_computed.get(),
            1,
            "录入段算一次参照"
        );
        let StageOutcome::Judged(read_back) =
            through_the_library(RunStage::JudgeOnly, &reference_units).expect("只判那一趟读得回")
        else {
            panic!("只判那一趟交回读回的输入与参照")
        };
        assert_eq!(
            times_the_reference_was_computed.get(),
            1,
            "只判那一趟不算参照"
        );
        assert_eq!(read_back.input.bytes(), input.as_slice());
        assert_eq!(
            read_back.reference.bytes(),
            input.iter().rev().copied().collect::<Vec<u8>>().as_slice()
        );
        assert!(matches!(
            through_the_library(RunStage::RecordOnly, &reference_units).expect("同一把键再录一趟"),
            StageOutcome::RecordedOnly
        ));
        assert_eq!(
            times_the_reference_was_computed.get(),
            1,
            "录入键命中：不再算参照"
        );
        let refused: Result<JudgingUnitsKey, String> = Err("样本：拒算".to_string());
        for _run in 0..2 {
            assert!(matches!(
                through_the_library(RunStage::RecordOnly, &refused).expect("参照一侧拒算照录"),
                StageOutcome::RecordedOnly
            ));
        }
        assert_eq!(
            times_the_reference_was_computed.get(),
            3,
            "参照一侧拒算：每一趟都照录、不凭命中跳过"
        );
        assert!(
            through_the_library(RunStage::JudgeOnly, &units_fingerprinted(12)).is_err(),
            "参照一侧的判法单元换了一把：录入键变，库里没有这把键的清单"
        );
    }

    /// 读回的对拍输入核摘要：字节与摘要对得上才造得出，差一个字节就拒。
    #[test]
    fn a_pairing_input_read_back_is_refused_when_its_bytes_do_not_match_the_digest() {
        let input = b"pairing input".to_vec();
        let digest = sha256_digest(&input);
        assert!(PairingInputReadBack::from_the_library_bytes(input.clone(), &digest).is_ok());
        let mut one_byte_changed = input;
        one_byte_changed[0] ^= 1;
        assert!(
            PairingInputReadBack::from_the_library_bytes(one_byte_changed, &digest).is_err(),
            "库里那份被改过一个字节：摘要对不上，拒"
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

    /// 几条线程同时要同一个摘要：只读一次，都拿到读回的那一份；之后再要也不再读。
    /// 判别力（`crates/mutations.tsv`）：「判定库锁收窄 同一摘要每次各开一个格子」这一行让这一条红（各读各的）。
    #[test]
    fn several_threads_asking_for_one_digest_read_it_once_and_share_the_value() {
        let values = ValuesReadBackOnceInThisProcess::<u32>::holding_at_most(1024);
        let reads = AtomicU64::new(0);
        let threads_asking = 4;
        let start_together = std::sync::Barrier::new(threads_asking);
        let ask = || {
            values
                .held_or_read_once(&[1; DIGEST_BYTES], || {
                    reads.fetch_add(1, Ordering::SeqCst);
                    std::thread::sleep(std::time::Duration::from_millis(20));
                    Ok(StageOutcome::Judged((7u32, 8u64)))
                })
                .expect("读得回")
                .judged()
                .map(|value| *value)
                .expect("读回了值")
        };
        std::thread::scope(|scope| {
            let handles: Vec<_> = (0..threads_asking)
                .map(|_thread| {
                    scope.spawn(|| {
                        start_together.wait();
                        ask()
                    })
                })
                .collect();
            for handle in handles {
                assert_eq!(handle.join().expect("线程不 panic"), 7);
            }
        });
        assert_eq!(ask(), 7);
        assert_eq!(reads.load(Ordering::SeqCst), 1, "同一个摘要只读一次");
    }

    /// 留的字节超了额度，从最早读回的放起；比额度还大的不留。放掉的下一次照读，留着的不再读。
    /// 判别力（`crates/mutations.tsv`）：「判定库锁收窄 读回的流超额也不放」这一行让这一条红（放掉的那一份还留着，不再读）。
    #[test]
    fn values_read_back_beyond_the_limit_are_let_go_from_the_earliest() {
        let values = ValuesReadBackOnceInThisProcess::<u32>::holding_at_most(10);
        let reads = AtomicU64::new(0);
        let ask = |digest_byte: u8, payload_bytes: u64| {
            let value_read_or_held = values
                .held_or_read_once(&[digest_byte; DIGEST_BYTES], || {
                    reads.fetch_add(1, Ordering::SeqCst);
                    Ok(StageOutcome::Judged((
                        u32::from(digest_byte),
                        payload_bytes,
                    )))
                })
                .expect("读得回")
                .judged()
                .expect("读回了值");
            assert_eq!(*value_read_or_held, u32::from(digest_byte));
            reads.load(Ordering::SeqCst)
        };
        assert_eq!(ask(1, 6), 1, "头一份现读");
        assert_eq!(ask(1, 6), 1, "还留着");
        assert_eq!(ask(2, 6), 2, "第二份现读，超了额度放掉头一份");
        assert_eq!(ask(2, 6), 2, "第二份还留着");
        assert_eq!(ask(1, 6), 3, "头一份放掉了，照读");
        assert_eq!(ask(3, 11), 4, "比额度还大的现读");
        assert_eq!(ask(3, 11), 5, "比额度还大的不留，照读");
    }

    /// 握着库锁的线程走到纯计算当场 panic；别的线程握着不算，放了锁也不算。
    /// 判别力（`crates/mutations.tsv`）：「判定库锁收窄 纯计算的断言不判」这一行让这一条红。
    #[test]
    fn the_pure_work_assertion_fires_only_on_the_thread_holding_the_library_lock() {
        // 一次性库留着：要一把真的库锁
        let disposable = DisposableLibrary::new("pure-work-assertion")
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
        let library = LibraryOpenInThisProcess::open_or_share().expect("开得了库");
        let held = library.lock();
        assert!(
            std::panic::catch_unwind(|| assert_this_thread_holds_no_library_lock("单测的纯计算"))
                .is_err(),
            "握着锁的这条线程上要 panic"
        );
        std::thread::scope(|scope| {
            scope
                .spawn(|| assert_this_thread_holds_no_library_lock("单测的纯计算"))
                .join()
                .expect("别的线程没握着锁，不 panic");
        });
        drop(held);
        assert_this_thread_holds_no_library_lock("单测的纯计算");
    }

    /// 从持锁时长行里取一个字段的数。
    fn field_of_the_lock_time_line(line: &str, field: &str) -> f64 {
        line.split(' ')
            .find_map(|word| word.strip_prefix(field)?.strip_prefix('='))
            .unwrap_or_else(|| panic!("持锁时长行里没有 {field}：{line}"))
            .parse()
            .expect("字段是数")
    }

    /// 一条线程握着锁 30 毫秒以上，另一条等它：行里持锁秒数、等锁秒数、取锁次数都记上，持锁占比是持锁秒数除以连接开着的秒数。
    /// 判别力（`crates/mutations.tsv`）：「判定库锁收窄 放锁时不记持锁时长」这一行让这一条红。
    #[test]
    fn the_lock_time_line_adds_up_what_the_lock_was_held_and_waited() {
        // 一次性库留着：要一把真的库锁，从这条连接开着起数
        let disposable =
            DisposableLibrary::new("lock-time-line").unwrap_or_else(|refusal| panic!("{refusal}"));
        let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
        let library = LibraryOpenInThisProcess::open_or_share().expect("开得了库");
        let time_held = std::time::Duration::from_millis(30);
        let the_other_thread_is_about_to_ask = std::sync::atomic::AtomicBool::new(false);
        std::thread::scope(|scope| {
            let held = library.lock();
            let other_thread = scope.spawn(|| {
                the_other_thread_is_about_to_ask.store(true, Ordering::SeqCst);
                drop(library.lock());
            });
            // 迭代上界：另一条线程起来就设它，起不来 join 那里也会报
            while !the_other_thread_is_about_to_ask.load(Ordering::SeqCst) {
                std::thread::yield_now();
            }
            std::thread::sleep(time_held);
            drop(held);
            other_thread.join().expect("另一条线程不 panic");
        });
        let line = library
            .open_library()
            .lock_time_line(std::time::Duration::from_millis(100));
        assert!(line.starts_with(LIBRARY_LOCK_TIME_LINE_PREFIX), "{line}");
        let held = field_of_the_lock_time_line(&line, "seconds_held");
        assert!(held >= time_held.as_secs_f64(), "持锁秒数没记全：{line}");
        assert!(
            field_of_the_lock_time_line(&line, "seconds_waited") >= time_held.as_secs_f64() / 2.0,
            "另一条线程等的没记上：{line}"
        );
        assert!(
            field_of_the_lock_time_line(&line, "acquisitions") >= 2.0,
            "两次取锁都数：{line}"
        );
        assert!(
            (field_of_the_lock_time_line(&line, "held_share_of_the_wall_clock") - held / 0.1).abs()
                < 0.02,
            "持锁占比是持锁秒数除以连接开着的秒数：{line}"
        );
    }

    /// 持锁时长行隔 60 秒一行：不到点不打，到点打一次，同一个点不打第二次，下一行从这一次到点起再隔 60 秒。
    /// 判别力（`crates/mutations.tsv`）：「判定库锁收窄 持锁时长行不看到没到点」这一行让这一条红。
    #[test]
    fn the_lock_time_line_comes_once_per_interval() {
        let times = LibraryLockTimes::starting_at_this_opening();
        let at_second = |second: u64| times.opened_at + std::time::Duration::from_secs(second);
        assert_eq!(
            times.time_open_when_a_lock_time_line_is_due(at_second(59)),
            None
        );
        assert_eq!(
            times.time_open_when_a_lock_time_line_is_due(at_second(60)),
            Some(std::time::Duration::from_secs(60))
        );
        assert_eq!(
            times.time_open_when_a_lock_time_line_is_due(at_second(60)),
            None
        );
        assert_eq!(
            times.time_open_when_a_lock_time_line_is_due(at_second(119)),
            None
        );
        assert_eq!(
            times.time_open_when_a_lock_time_line_is_due(at_second(120)),
            Some(std::time::Duration::from_secs(120))
        );
    }

    /// 一条线程取锁、在锁里读一次库（与 prune-server 往返一次）再放：这一次持锁记在取锁的那一处（文件与行）名下，锁里往返的纳秒数大于零、
    /// 不超过持锁的纳秒数；连接那一行与各调用点那几行都带连接名（`connection=`）与进程号（`pid=`），调用点那一行点名这一处。
    /// 同一个进程里关了再开同一份库，两次的连接名不同（各自的 `seconds_open` 各从零数，按连接名取最后一行不混）。
    /// 判别力（`crates/mutations.tsv`）：「判定库锁时长 锁里的往返不记」这一行让这一条红。
    #[test]
    fn the_lock_time_lines_name_the_connection_and_the_site_with_the_round_trips_in_the_lock() {
        // 一次性库留着：要一把真的库锁与真的往返，从这条连接开着起数
        let disposable = DisposableLibrary::new("lock-time-lines-of-the-sites")
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
        let library = LibraryOpenInThisProcess::open_or_share().expect("开得了库");
        let (held, line_of_the_lock) = (library.lock(), line!());
        assert!(
            held.read_flow_facts(&[0; DIGEST_BYTES])
                .expect("读得了库")
                .is_none(),
            "空库里没有这一份事实表"
        );
        drop(held);
        let open = library.open_library();
        let held_in_round_trips = open
            .lock_times
            .nanoseconds_held_in_round_trips
            .load(Ordering::Relaxed);
        assert!(
            held_in_round_trips > 0
                && held_in_round_trips <= open.lock_times.nanoseconds_held.load(Ordering::Relaxed),
            "锁里往返的纳秒数大于零、不超过持锁的纳秒数：{held_in_round_trips}"
        );
        {
            let sites = open
                .lock_times
                .of_each_site
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            let of_the_site = sites
                .get(&(file!(), line_of_the_lock))
                .expect("这一次持锁记在取锁的那一处名下");
            assert!(
                of_the_site.acquisitions == 1
                    && of_the_site.nanoseconds_held_in_round_trips > 0
                    && of_the_site.nanoseconds_held_in_round_trips <= of_the_site.nanoseconds_held,
                "那一处取了一次锁、锁里往返的纳秒数大于零而不超过持锁的纳秒数"
            );
        }
        let connection_and_process = format!(
            "connection={} pid={}",
            open.connection_name,
            std::process::id()
        );
        let line = open.lock_time_line(std::time::Duration::from_millis(100));
        assert!(
            line.starts_with(&format!(
                "{LIBRARY_LOCK_TIME_LINE_PREFIX} {connection_and_process} "
            )),
            "{line}"
        );
        let site = format!("site={}:{line_of_the_lock} ", file!());
        assert!(
            open.lock_time_lines_of_the_sites().iter().any(|site_line| {
                site_line.starts_with(&format!(
                    "{LIBRARY_LOCK_TIME_OF_A_SITE_LINE_PREFIX} {connection_and_process} "
                )) && site_line.contains(&site)
            }),
            "调用点那几行里有点名 {site} 的一行：{:#?}",
            open.lock_time_lines_of_the_sites()
        );
        let connection_name_of_the_first_opening = open.connection_name.clone();
        drop(library);
        let reopened = LibraryOpenInThisProcess::open_or_share().expect("再开得了库");
        assert_ne!(
            reopened.open_library().connection_name,
            connection_name_of_the_first_opening,
            "关了再开，连接名换一个"
        );
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
        agreement_of_the_block: &(dyn Fn(&std::ops::Range<u64>) -> PairingAgreement + Sync),
    ) -> PairingBlocksRun {
        let blocks_asked_of_the_judge_each_time = Mutex::new(Vec::new());
        let mut conclusions_in_order = Vec::new();
        let blocks_of_the_flow = pairing_conclusions_of_the_blocks_reused_or_judged(
            library,
            flow_inputs_digest,
            state_count,
            std::num::NonZeroU64::new(4).expect("块宽 4 不是 0"),
            std::num::NonZeroUsize::new(most_blocks_judged_together).expect("一次判的块数不是 0"),
            std::num::NonZeroUsize::new(1).expect("一段一段判"),
            &PairingBlocksShare::whole_flow_in_one_share(),
            one_side,
            other_side,
            &|blocks| {
                blocks_asked_of_the_judge_each_time
                    .lock()
                    .expect("判段不 panic")
                    .push(blocks.len());
                Some(
                    blocks
                        .iter()
                        .map(|ordinals| PairingJudged {
                            conclusion: conclusion_bytes_of_the_pairing_block(ordinals),
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
            blocks_asked_of_the_judge_each_time: blocks_asked_of_the_judge_each_time
                .into_inner()
                .expect("判段不 panic"),
            conclusions_in_order,
        }
    }

    /// 单测的判段给一块的结论字节：只由块的序号段定。
    fn conclusion_bytes_of_the_pairing_block(ordinals: &std::ops::Range<u64>) -> Vec<u8> {
        format!("判{}-{}", ordinals.start, ordinals.end).into_bytes()
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

    /// 几段块同时判：头一段判完之前只有它一段；之后三段一起在各自的线程上判、后起的先判完，结论照样按序号次序折进总账、逐块都存进库，
    /// 手里压着的块结论到 3 段 × 2 块为止。后起的两段先交回、队头那一段后交回，由队头那一段等到它们的块存进库再交回钉住，不看线程时序。
    /// 判别力（`crates/mutations.tsv`）：头一段也不单独判，头一段判的时候别的段已经起了，这条红；一次只许一段（几段一起判的数不认），
    /// 后面三段凑不齐、等到点，这条红；判完一段不排队、谁先判完谁先折进总账，这条红。
    #[test]
    fn stretches_judged_together_on_their_own_threads_come_back_in_block_order_and_all_stored() {
        const STRETCHES_AT_ONCE: usize = 3;
        const LONGEST_WAIT_FOR_THE_OTHER_STRETCHES: std::time::Duration =
            std::time::Duration::from_secs(60);
        // 一起判的三段里排在队头的那一段从哪个序号起：头一段之后的那一段
        const FIRST_STATE_OF_THE_QUEUE_HEAD_AMONG_THE_STRETCHES_TOGETHER: u64 = 4 * 2;
        // 一次性库留着：要从空库起数现判的块
        let disposable = DisposableLibrary::new("pairing-blocks-together")
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
        let library = CrashLibrary::open(CrashLibraryClass::JudgeTablesPairing).expect("开得了库");
        let units = judging_units_key_of_these_units(&[("gpu_judge_unit", 1)]);
        let other_units = judging_units_key_of_these_units(&[("cpu_judge_unit", 2)]);
        let flow_inputs_digest = [41u8; DIGEST_BYTES];
        // 一段 2 块、一块 4 个状态，头一段加后面一起判的三段，一共 8 块
        let state_count = 4 * 2 * (1 + u64::try_from(STRETCHES_AT_ONCE).expect("装得进 u64"));
        // 在飞的几段各从哪个序号起；后面三段是不是都在飞过了
        let in_flight: Mutex<(Vec<u64>, bool)> = Mutex::new((Vec::new(), false));
        let in_flight_changed = std::sync::Condvar::new();
        let threads_of_the_stretches: Mutex<Vec<std::thread::ThreadId>> = Mutex::new(Vec::new());
        let is_stored_in_the_library = |ordinals: std::ops::Range<u64>| {
            library
                .lock()
                .read_pairing_conclusion(&PairingConclusionKey {
                    inputs_digest: pairing_block_inputs_digest(&flow_inputs_digest, &ordinals),
                    one_side_judging_units: *units.as_ref().expect("判法单元算得出").fingerprint(),
                    other_side_judging_units: *other_units
                        .as_ref()
                        .expect("判法单元算得出")
                        .fingerprint(),
                })
                .expect("读得了库")
                .is_some()
        };
        let mut conclusions_in_order = Vec::new();
        let blocks_of_the_flow = pairing_conclusions_of_the_blocks_reused_or_judged(
            &library,
            &flow_inputs_digest,
            state_count,
            std::num::NonZeroU64::new(4).expect("块宽 4 不是 0"),
            std::num::NonZeroUsize::new(2).expect("一次判 2 块"),
            std::num::NonZeroUsize::new(STRETCHES_AT_ONCE).expect("三段一起判"),
            &PairingBlocksShare::whole_flow_in_one_share(),
            &units,
            &other_units,
            &|blocks| {
                let start = blocks[0].start;
                threads_of_the_stretches
                    .lock()
                    .expect("判段不留毒")
                    .push(std::thread::current().id());
                let mut guarded = in_flight.lock().expect("判段不留毒");
                guarded.0.push(start);
                if guarded.0.len() == STRETCHES_AT_ONCE {
                    guarded.1 = true;
                }
                in_flight_changed.notify_all();
                if start == 0 {
                    // 放开锁等一会儿：别的段要是起了，醒来时在飞的就不止它一段
                    let (still_guarded, _timed_out) = in_flight_changed
                        .wait_timeout(guarded, std::time::Duration::from_millis(500))
                        .expect("判段不留毒");
                    guarded = still_guarded;
                    assert_eq!(guarded.0, vec![0], "头一段判完之前只有它一段");
                } else {
                    // 三段都在飞过了才往下走；序号比它大的都走了它才走：后起的先判完
                    let (still_guarded, wait_outcome) = in_flight_changed
                        .wait_timeout_while(
                            guarded,
                            LONGEST_WAIT_FOR_THE_OTHER_STRETCHES,
                            |(starts, all_were_in_flight)| {
                                !*all_were_in_flight || starts.iter().any(|other| *other > start)
                            },
                        )
                        .expect("判段不留毒");
                    guarded = still_guarded;
                    assert!(
                        !wait_outcome.timed_out(),
                        "序号从 {start} 起的这一段等不到别的段：三段没有一起判（在飞的 {:?}）",
                        guarded.0
                    );
                }
                guarded.0.retain(|other| *other != start);
                in_flight_changed.notify_all();
                drop(guarded);
                if start == FIRST_STATE_OF_THE_QUEUE_HEAD_AMONG_THE_STRETCHES_TOGETHER {
                    // 后起的两段离开判段之后、交回结论之前还隔着一步：等到它们的块都存进了库（收的那一头收到一段先存再记），
                    // 这一段才交回，收的次序就是后起的先到，不看那一步的时序
                    let deadline = std::time::Instant::now() + LONGEST_WAIT_FOR_THE_OTHER_STRETCHES;
                    // 迭代上界是到点：每一轮问一次库，到点就判红
                    while !(start + 4 * 2..state_count)
                        .step_by(4)
                        .all(|block_start| is_stored_in_the_library(block_start..block_start + 4))
                    {
                        assert!(
                            std::time::Instant::now() < deadline,
                            "序号从 {start} 起的这一段等不到后起的两段存进库"
                        );
                        std::thread::sleep(std::time::Duration::from_millis(2));
                    }
                }
                Some(
                    blocks
                        .iter()
                        .map(|ordinals| PairingJudged {
                            conclusion: conclusion_bytes_of_the_pairing_block(ordinals),
                            agreement: PairingAgreement::BothSidesAgree,
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
        assert_eq!(
            conclusions_in_order,
            (0..8)
                .map(|block: u64| {
                    let ordinals = 4 * block..4 * block + 4;
                    let conclusion = conclusion_bytes_of_the_pairing_block(&ordinals);
                    (ordinals, conclusion)
                })
                .collect::<Vec<_>>(),
            "每块的结论按序号次序各收一次"
        );
        assert_eq!(
            (
                blocks_of_the_flow.blocks_judged_now,
                blocks_of_the_flow.blocks_reused,
                blocks_of_the_flow.most_block_conclusions_held_at_once
            ),
            (8, 0, STRETCHES_AT_ONCE * 2),
            "8 块都现判；三段后起的先判完，头上那一段判完时手里压着 3 段 × 2 块"
        );
        let threads = threads_of_the_stretches.into_inner().expect("判段不留毒");
        assert_eq!(
            threads
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len(),
            threads.len(),
            "每段在自己的线程上判"
        );
        let second_run = pairing_blocks_run(
            &library,
            &flow_inputs_digest,
            state_count,
            2,
            &units,
            &other_units,
        );
        assert_eq!(
            (
                second_run.blocks_of_the_flow.blocks_judged_now,
                second_run.blocks_of_the_flow.blocks_reused
            ),
            (0, 8),
            "一起判的几段逐块都存进了库：第二趟一段一段走，8 块都复用"
        );
    }

    /// 后面一段报「本次未跑」：交 `Ok(None)`，别的段判完的照存进库（下一趟复用）、报本次未跑的那一段一块都不存。
    /// 判别力（`crates/mutations.tsv`）：判那一侧交 `None` 当没事，整条流照往下走、交 `Some`，这条红。
    #[test]
    fn a_stretch_reported_not_run_stops_the_flow_and_keeps_what_the_others_judged() {
        // 一次性库留着：要从空库起数现判的块
        let disposable = DisposableLibrary::new("pairing-blocks-not-run")
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
        let library = CrashLibrary::open(CrashLibraryClass::JudgeTablesPairing).expect("开得了库");
        let units = judging_units_key_of_these_units(&[("gpu_judge_unit", 1)]);
        let other_units = judging_units_key_of_these_units(&[("cpu_judge_unit", 2)]);
        let flow_inputs_digest = [42u8; DIGEST_BYTES];
        let run_once = |block_reported_not_run: Option<u64>| {
            pairing_conclusions_of_the_blocks_reused_or_judged(
                &library,
                &flow_inputs_digest,
                4 * 4,
                std::num::NonZeroU64::new(4).expect("块宽 4 不是 0"),
                std::num::NonZeroUsize::new(1).expect("一次判 1 块"),
                std::num::NonZeroUsize::new(1).expect("一段一段判"),
                &PairingBlocksShare::whole_flow_in_one_share(),
                &units,
                &other_units,
                &|blocks| {
                    if block_reported_not_run == Some(blocks[0].start) {
                        return None;
                    }
                    Some(
                        blocks
                            .iter()
                            .map(|ordinals| PairingJudged {
                                conclusion: conclusion_bytes_of_the_pairing_block(ordinals),
                                agreement: PairingAgreement::BothSidesAgree,
                            })
                            .collect(),
                    )
                },
                &mut |_ordinals, _conclusion| {},
            )
            .expect("库读写得了")
        };
        assert_eq!(run_once(Some(8)), None, "第三块报本次未跑：整条流交 None");
        let second_run = run_once(None).expect("这一趟没有报本次未跑的");
        assert_eq!(
            (second_run.blocks_reused, second_run.blocks_judged_now),
            (2, 2),
            "报本次未跑之前判完的两块存进了库；它自己与它后面的现判"
        );
    }

    /// 判一段块的线程上 panic：照原样往外抛（判红的消息不变），不当本次未跑。
    /// 判别力（`crates/mutations.tsv`）：panic 了只记成本次未跑、不往外抛，这条红（没 panic）。
    #[test]
    #[should_panic(expected = "两边对不上：这一段判红")]
    fn a_panic_on_the_thread_of_a_stretch_is_thrown_again_with_its_message() {
        let disposable = DisposableLibrary::new("pairing-blocks-panic")
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
        let library = CrashLibrary::open(CrashLibraryClass::JudgeTablesPairing).expect("开得了库");
        let units = judging_units_key_of_these_units(&[("gpu_judge_unit", 1)]);
        let other_units = judging_units_key_of_these_units(&[("cpu_judge_unit", 2)]);
        let _not_reached = pairing_conclusions_of_the_blocks_reused_or_judged(
            &library,
            &[43u8; DIGEST_BYTES],
            4,
            std::num::NonZeroU64::new(4).expect("块宽 4 不是 0"),
            std::num::NonZeroUsize::new(1).expect("一次判 1 块"),
            std::num::NonZeroUsize::new(2).expect("两段一起判"),
            &PairingBlocksShare::whole_flow_in_one_share(),
            &units,
            &other_units,
            &|_blocks| panic!("两边对不上：这一段判红"),
            &mut |_ordinals, _conclusion| {},
        );
    }

    /// 两份领的块不重不漏：几条流、几种权重上，每一组块恰好归一份，两份领的块数与权重成比例（偏差不超过一组）；整份（`0/1`）领全部。
    /// 判别力（`crates/mutations.tsv`）：归不归这一份的判断反过来，或一律算归这一份，这条红。
    #[test]
    fn the_shares_of_a_flow_claim_every_group_of_blocks_exactly_once() {
        const GROUPS: u64 = 1000;
        for flow_inputs_digest in [
            [0u8; DIGEST_BYTES],
            [7u8; DIGEST_BYTES],
            [200u8; DIGEST_BYTES],
        ] {
            for weights in [vec![1u64, 1], vec![2, 3], vec![1, 1, 1]] {
                let shares: Vec<PairingBlocksShare> = (0..weights.len())
                    .map(|index| {
                        PairingBlocksShare::with_this_share_of_these_weights(
                            crate::crash_amplification::HostShare {
                                index,
                                count: weights.len(),
                            },
                            weights.clone(),
                        )
                    })
                    .collect();
                let total_weight: u64 = weights.iter().sum();
                let mut groups_of_each_share = vec![0u64; weights.len()];
                for group in 0..GROUPS {
                    let claiming: Vec<usize> = shares
                        .iter()
                        .enumerate()
                        .filter(|(_, share)| {
                            share.claims_this_group_of_pairing_blocks(&flow_inputs_digest, group)
                        })
                        .map(|(index, _)| index)
                        .collect();
                    assert_eq!(
                        claiming.len(),
                        1,
                        "权重 {weights:?}：第 {group} 组恰好归一份，实际归 {claiming:?}"
                    );
                    groups_of_each_share[claiming[0]] += 1;
                }
                for (groups, weight) in groups_of_each_share.iter().zip(&weights) {
                    assert!(
                        groups.abs_diff(GROUPS * weight / total_weight) <= total_weight,
                        "权重 {weights:?}：各份领的组数 {groups_of_each_share:?} 与权重成比例"
                    );
                }
            }
            for group in 0..GROUPS {
                assert!(
                    PairingBlocksShare::whole_flow_in_one_share()
                        .claims_this_group_of_pairing_blocks(&flow_inputs_digest, group),
                    "整份领全部"
                );
            }
        }
    }

    /// 只走一段组的一趟：组区间外的组不领（两头外侧紧挨着的那一组在内），组区间里的每一组都领。
    /// 判别力（`crates/mutations.tsv`）：组区间的判断一律算落在里面，这条红（区间外的组被领）。
    #[test]
    fn a_run_over_a_range_of_groups_claims_every_group_in_it_and_none_outside() {
        const GROUPS_TRIED: u64 = 20;
        let groups_of_the_run = 5..9;
        for flow_inputs_digest in [[0u8; DIGEST_BYTES], [7u8; DIGEST_BYTES]] {
            let share = PairingBlocksShare::whole_flow_in_one_share()
                .only_the_groups_of_pairing_blocks_in(groups_of_the_run.clone());
            let claimed: Vec<u64> = (0..GROUPS_TRIED)
                .filter(|group| {
                    share.claims_this_group_of_pairing_blocks(&flow_inputs_digest, *group)
                })
                .collect();
            assert_eq!(
                claimed,
                groups_of_the_run.clone().collect::<Vec<u64>>(),
                "组区间 {groups_of_the_run:?}：领的恰好是区间里的每一组"
            );
            for group_just_outside in [groups_of_the_run.start - 1, groups_of_the_run.end] {
                assert!(
                    !share.groups_of_this_run_hold(group_just_outside),
                    "组区间 {groups_of_the_run:?}：紧挨着两头外侧的第 {group_just_outside} 组不落在里面"
                );
            }
        }
    }

    /// 组区间与几台分份叠用：一组要落在组区间里、又归这一份才领；两份各叠同一段组区间时，组区间里的每一组恰好一份领，区间外的哪一份都不领。
    /// 判别力（`crates/mutations.tsv`）：两样只要一样领就算领，这条红。
    #[test]
    fn a_range_of_groups_stacked_on_the_shares_of_several_hosts_claims_a_group_only_when_both_do() {
        const GROUPS_TRIED: u64 = 200;
        let groups_of_the_run = 40..120;
        let flow_inputs_digest = [7u8; DIGEST_BYTES];
        let shares_of_the_hosts: Vec<PairingBlocksShare> = (0..2)
            .map(|index| {
                PairingBlocksShare::with_this_share_of_these_weights(
                    crate::crash_amplification::HostShare { index, count: 2 },
                    vec![1, 1],
                )
            })
            .collect();
        let stacked_shares: Vec<PairingBlocksShare> = shares_of_the_hosts
            .iter()
            .map(|share_of_the_host| {
                share_of_the_host
                    .clone()
                    .only_the_groups_of_pairing_blocks_in(groups_of_the_run.clone())
            })
            .collect();
        for (share_of_the_host, stacked) in shares_of_the_hosts.iter().zip(&stacked_shares) {
            let mut groups_of_the_host_outside_the_range = 0u64;
            let mut groups_in_the_range_left_to_the_other_host = 0u64;
            for group in 0..GROUPS_TRIED {
                let claimed_by_the_host = share_of_the_host
                    .claims_this_group_of_pairing_blocks(&flow_inputs_digest, group);
                let in_the_range = groups_of_the_run.contains(&group);
                assert_eq!(
                    stacked.claims_this_group_of_pairing_blocks(&flow_inputs_digest, group),
                    claimed_by_the_host && in_the_range,
                    "第 {group} 组：归这一份 {claimed_by_the_host}、落在组区间 {groups_of_the_run:?} 里 {in_the_range}"
                );
                if claimed_by_the_host && !in_the_range {
                    groups_of_the_host_outside_the_range += 1;
                }
                if !claimed_by_the_host && in_the_range {
                    groups_in_the_range_left_to_the_other_host += 1;
                }
            }
            assert!(
                groups_of_the_host_outside_the_range > 0
                    && groups_in_the_range_left_to_the_other_host > 0,
                "取样的组里有归这一份而落在区间外的（{groups_of_the_host_outside_the_range} 组），也有落在区间里而归另一份的（{groups_in_the_range_left_to_the_other_host} 组）"
            );
        }
        for group in 0..GROUPS_TRIED {
            let claiming = stacked_shares
                .iter()
                .filter(|stacked| {
                    stacked.claims_this_group_of_pairing_blocks(&flow_inputs_digest, group)
                })
                .count();
            assert_eq!(
                claiming,
                usize::from(groups_of_the_run.contains(&group)),
                "第 {group} 组：组区间 {groups_of_the_run:?} 里的恰好一份领，区间外的哪一份都不领"
            );
        }
    }

    /// 一条流按状态序号区间拆开（区间两头落在组的边界上，最后一段到整条流的状态数为止、最后一组不满）：各段算出的组区间拼起来恰好盖满
    /// 整条流的每一组、两两不交，各段只走自己那段组的几趟，每一组恰好一趟领、整条流之外的组哪一趟都不领；起点对不齐、终点对不齐又不是整条流的状态数、
    /// 空的、越过整条流的区间算不出组区间。
    /// 判别力（`crates/mutations.tsv`）：组区间的上界当闭区间判（相邻两段交界上那一组两趟都领），这条红；终点按整除取组、不往上取整（最后那一组不满的哪一趟都不领），这条红。
    #[test]
    fn ranges_of_states_that_tile_a_flow_give_ranges_of_groups_that_tile_its_groups_without_overlap(
    ) {
        const STATES_PER_BLOCK: u64 = 4;
        const BLOCKS_PER_GROUP: usize = 3;
        const STATES_PER_GROUP: u64 = 12;
        const STATE_COUNT: u64 = 13 * STATES_PER_GROUP + 2;
        let states_per_block = std::num::NonZeroU64::new(STATES_PER_BLOCK).expect("块宽 4 不是 0");
        let blocks_per_group =
            std::num::NonZeroUsize::new(BLOCKS_PER_GROUP).expect("一组 3 块不是 0");
        let ranges_of_states = [
            0..2 * STATES_PER_GROUP,
            2 * STATES_PER_GROUP..4 * STATES_PER_GROUP,
            4 * STATES_PER_GROUP..8 * STATES_PER_GROUP,
            8 * STATES_PER_GROUP..STATE_COUNT,
        ];
        let ranges_of_groups: Vec<std::ops::Range<u64>> = ranges_of_states
            .iter()
            .map(|ordinals| {
                groups_of_pairing_blocks_holding_the_states(
                    ordinals,
                    STATE_COUNT,
                    states_per_block,
                    blocks_per_group,
                )
                .unwrap_or_else(|refusal| panic!("{refusal}"))
            })
            .collect();
        let groups_of_the_flow = STATE_COUNT.div_ceil(STATES_PER_GROUP);
        assert_eq!(
            ranges_of_groups,
            vec![0..2, 2..4, 4..8, 8..groups_of_the_flow],
            "各段的组区间：最后一段罩到不满的最后一组"
        );
        let flow_inputs_digest = [9u8; DIGEST_BYTES];
        let runs_over_the_ranges: Vec<PairingBlocksShare> = ranges_of_groups
            .iter()
            .map(|groups| {
                PairingBlocksShare::whole_flow_in_one_share()
                    .only_the_groups_of_pairing_blocks_in(groups.clone())
            })
            .collect();
        for group in 0..groups_of_the_flow + 2 {
            let claiming = runs_over_the_ranges
                .iter()
                .filter(|run| run.claims_this_group_of_pairing_blocks(&flow_inputs_digest, group))
                .count();
            assert_eq!(
                claiming,
                usize::from(group < groups_of_the_flow),
                "第 {group} 组：整条流的 {groups_of_the_flow} 组里的恰好一趟领，之外的哪一趟都不领"
            );
        }
        for refused in [
            STATES_PER_BLOCK..2 * STATES_PER_GROUP,
            0..2 * STATES_PER_GROUP + STATES_PER_BLOCK,
            STATES_PER_GROUP..STATES_PER_GROUP,
            8 * STATES_PER_GROUP..STATE_COUNT + 1,
        ] {
            assert!(
                groups_of_pairing_blocks_holding_the_states(
                    &refused,
                    STATE_COUNT,
                    states_per_block,
                    blocks_per_group
                )
                .is_err(),
                "状态序号区间 {refused:?} 算不出组区间"
            );
        }
    }

    /// 一条流按组区间拆成几趟：每一趟只判、只折进总账自己组区间里的块，别的数进 `blocks_outside_the_groups_of_this_run`（不算归别的份）；
    /// 几趟判的块拼起来不重不漏；之后整条流一趟（`0/1`）一块都不现判、每块复用：块键不带组区间。
    /// 判别力（`crates/mutations.tsv`）：组区间外的块也判，这条红。
    #[test]
    fn runs_over_ranges_of_groups_judge_every_block_once_and_a_run_over_the_whole_flow_reuses_all_of_them(
    ) {
        const BLOCKS: u64 = 40;
        const BLOCKS_OF_A_GROUP: usize = 3;
        let disposable = DisposableLibrary::new("pairing-blocks-ranges-of-groups")
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
        let library = CrashLibrary::open(CrashLibraryClass::JudgeTablesPairing).expect("开得了库");
        let units = judging_units_key_of_these_units(&[("gpu_judge_unit", 1)]);
        let other_units = judging_units_key_of_these_units(&[("cpu_judge_unit", 2)]);
        let flow_inputs_digest = [45u8; DIGEST_BYTES];
        let state_count = 4 * BLOCKS;
        let run_the_share = |share: &PairingBlocksShare| {
            let blocks_asked_of_the_judge: Mutex<Vec<std::ops::Range<u64>>> =
                Mutex::new(Vec::new());
            let mut taken: Vec<std::ops::Range<u64>> = Vec::new();
            let blocks_of_the_flow = pairing_conclusions_of_the_blocks_reused_or_judged(
                &library,
                &flow_inputs_digest,
                state_count,
                std::num::NonZeroU64::new(4).expect("块宽 4 不是 0"),
                std::num::NonZeroUsize::new(BLOCKS_OF_A_GROUP).expect("一次判的块数不是 0"),
                std::num::NonZeroUsize::new(2).expect("两段一起判"),
                share,
                &units,
                &other_units,
                &|blocks| {
                    blocks_asked_of_the_judge
                        .lock()
                        .expect("判段不 panic")
                        .extend_from_slice(blocks);
                    Some(
                        blocks
                            .iter()
                            .map(|ordinals| PairingJudged {
                                conclusion: conclusion_bytes_of_the_pairing_block(ordinals),
                                agreement: PairingAgreement::BothSidesAgree,
                            })
                            .collect(),
                    )
                },
                &mut |ordinals, _conclusion| taken.push(ordinals.clone()),
            )
            .expect("库读写得了")
            .expect("判段没报本次未跑");
            let mut judged = blocks_asked_of_the_judge
                .into_inner()
                .expect("判段不 panic");
            judged.sort_by_key(|ordinals| ordinals.start);
            (blocks_of_the_flow, judged, taken)
        };
        let blocks_of_a_group = u64::try_from(BLOCKS_OF_A_GROUP).expect("装得进 u64");
        let blocks_of_the_groups = |groups: &std::ops::Range<u64>| -> Vec<std::ops::Range<u64>> {
            (groups.start * blocks_of_a_group..(groups.end * blocks_of_a_group).min(BLOCKS))
                .map(|block| 4 * block..4 * block + 4)
                .collect()
        };
        let mut judged_by_the_runs: Vec<std::ops::Range<u64>> = Vec::new();
        for groups in [0..5, 5..BLOCKS.div_ceil(blocks_of_a_group)] {
            let (blocks_of_the_flow, judged, taken) = run_the_share(
                &PairingBlocksShare::whole_flow_in_one_share()
                    .only_the_groups_of_pairing_blocks_in(groups.clone()),
            );
            let blocks_of_the_run = blocks_of_the_groups(&groups);
            let block_count_of_the_run =
                u64::try_from(blocks_of_the_run.len()).expect("装得进 u64");
            assert_eq!(
                judged, blocks_of_the_run,
                "组区间 {groups:?} 的一趟判的恰好是那几组的块"
            );
            assert_eq!(
                taken, blocks_of_the_run,
                "组区间 {groups:?} 的一趟按序号次序只把那几组的块折进总账"
            );
            assert_eq!(
                (
                    blocks_of_the_flow.blocks_judged_now,
                    blocks_of_the_flow.blocks_outside_the_groups_of_this_run,
                    blocks_of_the_flow.blocks_left_to_other_shares
                ),
                (block_count_of_the_run, BLOCKS - block_count_of_the_run, 0),
                "组区间 {groups:?}：区间外的块数进区间外那一项，不算归别的份"
            );
            judged_by_the_runs.extend(judged);
        }
        judged_by_the_runs.sort_by_key(|ordinals| ordinals.start);
        assert_eq!(
            judged_by_the_runs,
            (0..BLOCKS)
                .map(|block| 4 * block..4 * block + 4)
                .collect::<Vec<_>>(),
            "几趟判的块拼起来不重不漏"
        );
        let (whole_run, judged_by_the_whole_run, _taken) =
            run_the_share(&PairingBlocksShare::whole_flow_in_one_share());
        assert_eq!(
            (
                whole_run.blocks_reused,
                whole_run.blocks_judged_now,
                whole_run.blocks_outside_the_groups_of_this_run
            ),
            (BLOCKS, 0, 0),
            "整条流那一趟每块复用：块键不带组区间"
        );
        assert!(
            judged_by_the_whole_run.is_empty(),
            "整条流那一趟判段一次都不叫"
        );
    }

    /// 两份各判自己的块、块键不带份额：两份都跑完之后，合并那一趟（`0/1`）一块都不现判，结论与从头整份判的逐块相同；
    /// 每一份只把自己领的块折进总账、别的数进 `blocks_left_to_other_shares`；两份交给判段的每一段都是连着的整组。
    /// 判别力（`crates/mutations.tsv`）：不归这一份的块也判，两份现判的块相加超过整条流，这条红；块一块一块轮着分、不按组分，判段拿不到整组连着的一段，这条红。
    #[test]
    fn two_shares_judge_every_block_once_and_the_merging_run_reuses_all_of_them() {
        const BLOCKS: u64 = 40;
        const BLOCKS_OF_A_GROUP: usize = 3;
        let disposable = DisposableLibrary::new("pairing-blocks-shares")
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
        let library = CrashLibrary::open(CrashLibraryClass::JudgeTablesPairing).expect("开得了库");
        let units = judging_units_key_of_these_units(&[("gpu_judge_unit", 1)]);
        let other_units = judging_units_key_of_these_units(&[("cpu_judge_unit", 2)]);
        let flow_inputs_digest = [44u8; DIGEST_BYTES];
        let state_count = 4 * BLOCKS;
        let run_the_share = |share: &PairingBlocksShare| {
            let stretches_asked: Mutex<Vec<Vec<std::ops::Range<u64>>>> = Mutex::new(Vec::new());
            let mut taken: Vec<std::ops::Range<u64>> = Vec::new();
            let blocks_of_the_flow = pairing_conclusions_of_the_blocks_reused_or_judged(
                &library,
                &flow_inputs_digest,
                state_count,
                std::num::NonZeroU64::new(4).expect("块宽 4 不是 0"),
                std::num::NonZeroUsize::new(BLOCKS_OF_A_GROUP).expect("一次判的块数不是 0"),
                std::num::NonZeroUsize::new(2).expect("两段一起判"),
                share,
                &units,
                &other_units,
                &|blocks| {
                    stretches_asked
                        .lock()
                        .expect("判段不 panic")
                        .push(blocks.to_vec());
                    Some(
                        blocks
                            .iter()
                            .map(|ordinals| PairingJudged {
                                conclusion: conclusion_bytes_of_the_pairing_block(ordinals),
                                agreement: PairingAgreement::BothSidesAgree,
                            })
                            .collect(),
                    )
                },
                &mut |ordinals, conclusion| {
                    assert_eq!(
                        conclusion,
                        conclusion_bytes_of_the_pairing_block(ordinals),
                        "折进总账的是这一块的结论"
                    );
                    taken.push(ordinals.clone());
                },
            )
            .expect("库读写得了")
            .expect("判段没报本次未跑");
            (
                blocks_of_the_flow,
                stretches_asked.into_inner().expect("判段不 panic"),
                taken,
            )
        };
        let mut judged_by_the_shares: Vec<std::ops::Range<u64>> = Vec::new();
        for index in 0..2 {
            let (blocks_of_the_flow, stretches_asked, taken) =
                run_the_share(&PairingBlocksShare::with_this_share_of_these_weights(
                    crate::crash_amplification::HostShare { index, count: 2 },
                    vec![1, 1],
                ));
            assert_eq!(
                blocks_of_the_flow.blocks_judged_now
                    + blocks_of_the_flow.blocks_left_to_other_shares,
                BLOCKS,
                "第 {index} 份：判的加留给别的份的是整条流"
            );
            assert!(
                blocks_of_the_flow.blocks_left_to_other_shares > 0,
                "第 {index} 份：两份等权，有留给别的份的"
            );
            assert_eq!(
                u64::try_from(taken.len()).expect("装得进 u64"),
                blocks_of_the_flow.blocks_judged_now,
                "第 {index} 份只把自己领的块折进总账"
            );
            for stretch in &stretches_asked {
                assert!(
                    stretch.windows(2).all(|pair| pair[0].end == pair[1].start)
                        && stretch.len() <= BLOCKS_OF_A_GROUP,
                    "第 {index} 份交给判段的一段是连着的块：{stretch:?}"
                );
            }
            assert!(
                stretches_asked
                    .iter()
                    .any(|stretch| stretch.len() == BLOCKS_OF_A_GROUP),
                "第 {index} 份领的是整组的块，判段拿到过整组连着的一段：{stretches_asked:?}"
            );
            judged_by_the_shares.extend(stretches_asked.into_iter().flatten());
        }
        judged_by_the_shares.sort_by_key(|ordinals| ordinals.start);
        assert_eq!(
            judged_by_the_shares,
            (0..BLOCKS)
                .map(|block| 4 * block..4 * block + 4)
                .collect::<Vec<_>>(),
            "两份判的块不重不漏"
        );
        let (merging_run, stretches_asked_of_the_merging_run, taken_by_the_merging_run) =
            run_the_share(&PairingBlocksShare::whole_flow_in_one_share());
        assert_eq!(
            (
                merging_run.blocks_reused,
                merging_run.blocks_judged_now,
                merging_run.blocks_left_to_other_shares
            ),
            (BLOCKS, 0, 0),
            "合并那一趟每块复用：块键不带份额"
        );
        assert!(
            stretches_asked_of_the_merging_run.is_empty(),
            "合并那一趟判段一次都不叫"
        );
        assert_eq!(
            taken_by_the_merging_run,
            (0..BLOCKS)
                .map(|block| 4 * block..4 * block + 4)
                .collect::<Vec<_>>(),
            "合并那一趟每块的结论按序号次序各收一次"
        );
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

    /// 场景：一条新线程先拿改环境的锁（`CrashAmplificationEnvironment::cleared_then_set`）、再开一次性库（反着的次序）。
    /// 预期：开库取名额那一步在等名额之前当场 panic（名额满着也不挂住），消息写明先开一次性库、后拿改环境的锁；
    /// 那条线程 panic 之后锁已放，这条线程接着拿得到。
    /// 判别力（`crates/mutations.tsv`）：取名额不问这条线程拿没拿着改环境的锁（那条线程取到名额、起了临时服务，不 panic），这条红。
    #[test]
    fn taking_a_disposable_library_place_while_holding_the_environment_lock_panics_at_once_naming_the_order(
    ) {
        let reversed_order = std::thread::spawn(|| {
            let _environment =
                crate::crash_amplification::CrashAmplificationEnvironment::cleared_then_set(&[]);
            // 一次性库留着：自证反着的次序当场 panic（次序对了这一步取名额、起临时服务）
            DisposableLibrary::new("place-taken-while-holding-the-environment-lock")
                .map(|_library| ())
        })
        .join();
        let panic_payload =
            reversed_order.expect_err("拿着改环境的锁去开一次性库，取名额那一步当场 panic");
        let panic_message = panic_payload
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| {
                panic_payload
                    .downcast_ref::<&str>()
                    .map(|message| (*message).to_string())
            })
            .expect("assert! 带格式串的 panic 交回 String");
        assert_eq!(
            panic_message, PLACE_TAKEN_WHILE_HOLDING_THE_ENVIRONMENT_LOCK,
            "panic 的就是取名额那一步，消息写明先开库、后拿锁"
        );
        assert!(
            !crate::crash_amplification::this_thread_holds_the_environment_lock(),
            "前提：这条线程没拿改环境的锁"
        );
        let _environment_after_the_panic =
            crate::crash_amplification::CrashAmplificationEnvironment::cleared_then_set(&[]);
    }

    /// 场景：这条线程先开一份一次性库（新占一个名额），再拿改环境的锁，拿着锁再开第二份。
    /// 预期：第二份不等名额、不查次序（同一条线程已经起着一份，拿着锁再开不会互等），开得成、不 panic；两份是两段名字空间。
    /// 判别力（`crates/mutations.tsv`）：取名额那一步把「拿着改环境的锁」的检查挪回开头（这条线程已经起着一份的也查），这条红。
    #[test]
    fn a_thread_already_holding_a_disposable_library_place_opens_another_while_holding_the_environment_lock(
    ) {
        // 一次性库留着：自证这条线程起着一份之后拿着改环境的锁再开一份不查次序
        let library_opened_before_the_lock =
            DisposableLibrary::new("place-held-before-the-environment-lock")
                .unwrap_or_else(|refusal| panic!("{refusal}"));
        let environment =
            crate::crash_amplification::CrashAmplificationEnvironment::cleared_then_set(&[]);
        assert!(
            crate::crash_amplification::this_thread_holds_the_environment_lock(),
            "前提：开第二份时这条线程拿着改环境的锁"
        );
        // 一次性库留着：拿着改环境的锁开的第二份，自证不等名额、不 panic
        let library_opened_under_the_lock =
            DisposableLibrary::new("second-place-under-the-environment-lock")
                .unwrap_or_else(|refusal| panic!("{refusal}"));
        assert_ne!(
            library_opened_before_the_lock.key_space(),
            library_opened_under_the_lock.key_space(),
            "两份一次性库两段名字空间"
        );
        drop(environment);
        drop(library_opened_under_the_lock);
        drop(library_opened_before_the_lock);
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

    /// 场景：每台默认重活线程数从比一份临时服务多一条起到 124，外层 1 条到那么多条用例同时跑，每条里层要的条数取 1、2、
    /// 每台默认重活线程数与它的两倍（核对线程变量设得比它大）几样，照拿线程的算术（[`shares_of_the_test_process`]、[`inner_threads_granted`]、
    /// [`inner_threads_fit`]）一条接一条拿、放得下才拿。
    /// 预期：每条拿到的不过里层那一份、至少一条；一起拿到的合起来不过里层那一份；里层那一份加临时服务那一份不过每台默认重活线程数。
    /// 判别力（`crates/mutations.tsv`）：里层拿的不照那一份封顶，这条红；放不放得下不看已占的，这条红。
    #[test]
    fn inner_threads_of_the_cases_running_together_stay_within_the_heavy_thread_default() {
        for heavy_threads in THREADS_OF_A_TEMPORARY_SERVER + 1..=124 {
            let shares = shares_of_the_test_process(heavy_threads);
            assert!(
                shares.temporary_servers >= 1
                    && shares.temporary_servers * THREADS_OF_A_TEMPORARY_SERVER
                        + shares.inner_threads
                        <= heavy_threads,
                "每台默认 {heavy_threads} 条：临时服务至少一份，两份合起来不过它：{shares:?}"
            );
            for wanted in [1, 2, heavy_threads, 2 * heavy_threads] {
                let granted = inner_threads_granted(wanted, shares.inner_threads);
                assert!(
                    (1..=shares.inner_threads).contains(&granted),
                    "每台默认 {heavy_threads} 条、要 {wanted} 条：拿到的 {granted} 条在 1 到里层那一份 {} 条之间",
                    shares.inner_threads
                );
                for cases_running_together in 1..=heavy_threads {
                    let mut in_use = 0;
                    for _case in 0..cases_running_together {
                        if inner_threads_fit(in_use, granted, shares.inner_threads) {
                            in_use += granted;
                        }
                    }
                    assert!(
                        in_use <= shares.inner_threads && in_use <= heavy_threads,
                        "每台默认 {heavy_threads} 条、外层 {cases_running_together} 条各要 {wanted} 条：一起拿到的 {in_use} 条不过里层那一份 {} 条",
                        shares.inner_threads
                    );
                }
            }
        }
    }

    /// 场景：外层 8 条线程同时各要每台默认重活线程数那么多条里层线程（[`InnerThreadsTaken::take`]，这个单测进程用的那一份额度），拿到后各持 20 毫秒。
    /// 预期：每条拿到的都是里层那一份封顶后的条数；这 8 条同一时刻持着的合起来不过里层那一份；8 条都拿到过（还了的叫得醒等着的）。
    /// 别的用例同时也在拿：它们占的只让这 8 条多等，这 8 条合起来的条数照样不过那一份。
    /// 判别力（`crates/mutations.tsv`）：放不放得下不看已占的，这条红。
    #[test]
    fn threads_taken_by_cases_running_together_never_add_up_past_the_inner_share() {
        const CASES_RUNNING_TOGETHER: usize = 8;
        let heavy_threads = crate::heavy_threads::default_heavy_threads().get();
        let inner_threads_of_the_share = shares_of_the_test_process(heavy_threads).inner_threads;
        let held_now = std::sync::atomic::AtomicUsize::new(0);
        let most_held_at_once = std::sync::atomic::AtomicUsize::new(0);
        let granted_of_each_case: Vec<usize> = std::thread::scope(|scope| {
            let cases: Vec<_> = (0..CASES_RUNNING_TOGETHER)
                .map(|_case| {
                    scope.spawn(|| {
                        let taken = InnerThreadsTaken::take(heavy_threads);
                        let held =
                            held_now.fetch_add(taken.threads(), Ordering::SeqCst) + taken.threads();
                        most_held_at_once.fetch_max(held, Ordering::SeqCst);
                        std::thread::sleep(std::time::Duration::from_millis(20));
                        held_now.fetch_sub(taken.threads(), Ordering::SeqCst);
                        taken.threads()
                    })
                })
                .collect();
            cases
                .into_iter()
                .map(|case| case.join().expect("拿线程的那条不 panic"))
                .collect()
        });
        assert_eq!(
            granted_of_each_case,
            vec![
                inner_threads_granted(heavy_threads, inner_threads_of_the_share);
                CASES_RUNNING_TOGETHER
            ],
            "每条拿到的都是里层那一份封顶后的条数"
        );
        assert!(
            most_held_at_once.load(Ordering::SeqCst) <= inner_threads_of_the_share,
            "同一时刻持着的 {} 条不过里层那一份 {inner_threads_of_the_share} 条",
            most_held_at_once.load(Ordering::SeqCst)
        );
    }
}
