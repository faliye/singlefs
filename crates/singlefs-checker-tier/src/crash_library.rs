//! 崩溃判定库（RocksDB）：一份长期存放、按键复用、能剪枝的库；录入段与判段只经库交接。
//!
//! 规矩（用户 2026-10-11 定）：① 测试先把 CPU 跑出来的数据写进库，再从库里读出来判；② 每一类测试都做剪枝，按块键、结论种类、判法单元指纹
//! 存结论，判过的复用；③ CPU 产出的数据一律进库，判段按批从库里取出来交给 GPU，没有 GPU 才交给 CPU；④ 两段之间不在内存里交数据，
//! CPU 自己判也分两段。用户 2026-10-15 定：一份库、目录固定一处、类当索引；键只由内容定，不带路径、名字、种子、手写版本号。
//!
//! 库在家目录 [`CRASH_LIBRARY_HOME_ENVIRONMENT_VARIABLE`]（没设是家目录下 `.local/share/singlefs/crash-amplification`）下的
//! [`CRASH_LIBRARY_DIRECTORY_NAME`]：崩溃放量、崩溃枚举用例、给定状态表、两种对拍几类测试都开这一份，测试的类别（[`CrashLibraryClass`]）
//! 只进行里报的名字，结论种类（GPU、CPU、观察者、对拍）是块记录里的字段。入口脚本与双机驱动另交 [`CRASH_LIBRARY_DIRECTORY_ENVIRONMENT_VARIABLE`]
//! （只收绝对路径；第二台那一份落在树副本里、取回再导入，第二台的驱动给绝对路径；门禁的自证给一次性的目录）。仓根不是键的输入，也不是库的输入
//! （`records/2026-10-13-复用键与全仓改名.md`「2026-10-15 验证段第一步判红、两条定案、4-3 四批派出」定案 1）：这里没有一处收仓根。落在开机会清空的目录里的库拒绝
//! （[`refuse_a_library_that_a_reboot_would_erase`]），一次性的库（[`DisposableLibraryDirectory`]，流水线单测从空库起跑用）除外。
//!
//! 两段怎么接（[`two_stages`]）：录入段把这一项的几样各按摘要写进库、再按录入键（录入函数单元指纹与录入输入摘要，[`recording_key_of_the_inputs`]）
//! 登一份清单（[`ItemManifest`]）；录入键下已有清单就不再录（命中就跳过）。判段照同一份输入现算录入键、读清单，再按清单里的摘要读回那几样。
//! 选段照 [`RUN_STAGE_ENVIRONMENT_VARIABLE`]：`record` 只录、`judge` 只判、不设两段先后跑（中间照样经库）。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, PoisonError, Weak};
use std::time::{Duration, Instant};

use singlefs_harness::hexadecimal::hexadecimal_text;
use singlefs_harness::sha256::{sha256_digest, DIGEST_BYTES};

use crate::crash_facts::{FlowFacts, FlowFactsReadBack};
use crate::crash_identity::{judging_units_key, JudgingUnits, JudgingUnitsKey};
#[cfg(feature = "gpu")]
use crate::crash_judge_dispatch::{PackedJudgeTables, PackedJudgeTablesReadBack};
use crate::crash_point_replay_progress::CrashPointReplayToolchainIdentity;
pub use crate::verdict_store::{
    library_reads_are_skipped_in_this_process, JudgementCircumstances, RecordingKey,
    RunKindOfTheJudgement, CRASH_LIBRARY_RUN_KIND_ENVIRONMENT_VARIABLE,
    CRASH_LIBRARY_WRITE_ONLY_ENVIRONMENT_VARIABLE,
};
use crate::verdict_store::{PairingConclusionKey, StoredPairingConclusion, VerdictStore};

/// 库的家目录从这个环境变量取；没设是家目录下 [`DEFAULT_HOME_UNDER_THE_USER_HOME_DIRECTORY`]。入口脚本（`scripts/crash/crash-amplification-lib.sh`）读的是同一个变量。
pub const CRASH_LIBRARY_HOME_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_CRASH_AMPLIFICATION_HOME";

/// 入口脚本与双机驱动按趟交库目录时用的变量（只收绝对路径，相对路径拒绝：仓根不是库的输入）：几类测试都认它，交的是同一份库。
pub const CRASH_LIBRARY_DIRECTORY_ENVIRONMENT_VARIABLE: &str =
    "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY";

/// 开库撞上别的进程持着 RocksDB 的锁时最多等多久（秒；没设是 [`DEFAULT_LOCK_WAIT_SECONDS`]）。
pub const CRASH_LIBRARY_LOCK_WAIT_SECONDS_ENVIRONMENT_VARIABLE: &str =
    "SINGLEFS_CRASH_LIBRARY_LOCK_WAIT_SECONDS";

/// 这一趟跑哪一段。
pub const RUN_STAGE_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_CRASH_AMPLIFICATION_STAGE";

/// 家目录下库的默认落点（与入口脚本 `scripts/crash/crash-amplification-lib.sh` 文件头「判定库」一段相同）。
pub const DEFAULT_HOME_UNDER_THE_USER_HOME_DIRECTORY: &str =
    ".local/share/singlefs/crash-amplification";

/// 一份库在家目录下的目录名：固定一处、不带测试的类名、不带存法名、不随趟换（入口脚本的 `CRASH_LIBRARY_DIRECTORY_NAME` 与它逐字相同）。
pub const CRASH_LIBRARY_DIRECTORY_NAME: &str = "crash-state-verdicts";

/// 开库等锁的默认上限：半小时。同一份库同时只许一个进程开着，别的进程排队。
pub const DEFAULT_LOCK_WAIT_SECONDS: u64 = 1800;

/// 等锁时隔多久再试一次。
const LOCK_RETRY_INTERVAL: Duration = Duration::from_secs(2);

/// 等锁时隔多久打一行。
const LOCK_WAIT_LINE_INTERVAL: Duration = Duration::from_secs(60);

/// 开机会清空的目录：库落在它们之下重启一次就续不上。
const DIRECTORIES_A_REBOOT_ERASES: [&str; 4] = ["/tmp", "/var/tmp", "/dev/shm", "/run"];

/// 一次性的库的上层目录里放的记号文件（[`DisposableLibraryDirectory`] 建、它丢掉时连同目录删）：开机清空的地方也收它。
pub(crate) const DISPOSABLE_LIBRARY_MARKER_FILE_NAME: &str = ".singlefs-disposable-crash-library";

/// 库目录落在开机会清空的地方就拒绝，除非这份库是一次性的：哪一层上层目录里有 [`DisposableLibraryDirectory`] 放的记号，或环境写明
/// （`SINGLEFS_CRASH_AMPLIFICATION_LIBRARY_IS_TEMPORARY=1`，只给入口脚本的自证用）。判的是现存最长前缀的绝对路径（目录还没建时判它的上层）。
///
/// # Errors
/// 库目录落在 [`DIRECTORIES_A_REBOOT_ERASES`] 之下而不是一次性的。
pub fn refuse_a_library_that_a_reboot_would_erase(library_directory: &Path) -> Result<(), String> {
    if std::env::var("SINGLEFS_CRASH_AMPLIFICATION_LIBRARY_IS_TEMPORARY")
        .is_ok_and(|value| value == "1")
    {
        return Ok(());
    }
    if library_directory
        .ancestors()
        .skip(1)
        .any(|ancestor| ancestor.join(DISPOSABLE_LIBRARY_MARKER_FILE_NAME).is_file())
    {
        return Ok(());
    }
    let mut probe = library_directory.to_path_buf();
    let resolved = loop {
        if let Ok(resolved) = probe.canonicalize() {
            break resolved;
        }
        match probe.parent() {
            Some(parent) if parent != probe => probe = parent.to_path_buf(),
            _ => break library_directory.to_path_buf(),
        }
    };
    for erased in DIRECTORIES_A_REBOOT_ERASES {
        if resolved == Path::new(erased) || resolved.starts_with(erased) {
            return Err(format!(
                "库目录 {} 落在开机会清空的 {erased} 之下：重启一次就续不上。放到家目录或别的持久目录下；\
                 只有入口脚本自证里一次性的库才写 SINGLEFS_CRASH_AMPLIFICATION_LIBRARY_IS_TEMPORARY=1 放行",
                library_directory.display()
            ));
        }
    }
    Ok(())
}

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

/// 哪一类测试开的库：几类开的是同一份库（[`crash_library_directory`]），类别只进行里报的名字。
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

/// 一次性的库目录：流水线单测从空库起跑用（用户 2026-10-15 定：空库起跑的单测只在一次性副本里建库，起跑不清长期库）。
/// 落在 `${TMPDIR:-/tmp}` 下，名字带标签、进程号与序号；上层目录里放一个记号（[`refuse_a_library_that_a_reboot_would_erase`] 照它放行），
/// 库建在它底下的 [`Self::library`]（库只在不存在的目录上建）；丢掉时连同里面的库与别的文件一起删。
pub struct DisposableLibraryDirectory {
    directory: PathBuf,
}

static NEXT_DISPOSABLE_LIBRARY_SUFFIX: AtomicU64 = AtomicU64::new(0);

impl DisposableLibraryDirectory {
    /// # Errors
    /// 建不了目录、写不了记号。
    pub fn new(label: &str) -> Result<Self, String> {
        let suffix = NEXT_DISPOSABLE_LIBRARY_SUFFIX.fetch_add(1, Ordering::SeqCst);
        let directory = std::env::temp_dir().join(format!(
            "singlefs-disposable-crash-library-{label}-{}-{suffix}",
            std::process::id()
        ));
        if directory.exists() {
            // 同一个进程号与序号只会是上一个被杀、没删掉自己目录的进程留下的：它也是一次性的
            std::fs::remove_dir_all(&directory).map_err(|error| {
                format!(
                    "删不掉上一趟留下的一次性库目录 {}：{error}",
                    directory.display()
                )
            })?;
        }
        // 先认领、再建：建了目录却写不了记号而交回错误时，Drop 删掉建了的那一半
        let disposable = Self { directory };
        std::fs::create_dir_all(&disposable.directory).map_err(|error| {
            format!(
                "建不了一次性库目录 {}：{error}",
                disposable.directory.display()
            )
        })?;
        std::fs::write(
            disposable
                .directory
                .join(DISPOSABLE_LIBRARY_MARKER_FILE_NAME),
            b"",
        )
        .map_err(|error| {
            format!(
                "一次性库目录 {} 里写不了记号：{error}",
                disposable.directory.display()
            )
        })?;
        Ok(disposable)
    }

    /// 上层目录（用例自己的配置文件这类也放这里）。
    #[must_use]
    pub fn directory(&self) -> &Path {
        &self.directory
    }

    /// 库目录（还不在，第一次开时建）。
    #[must_use]
    pub fn library(&self) -> PathBuf {
        self.directory.join("library")
    }
}

/// 一次性的库目录当路径用（`directory.join(…)`、交给收 `&Path` 的函数）：就是它的上层目录 [`DisposableLibraryDirectory::directory`]。
impl std::ops::Deref for DisposableLibraryDirectory {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.directory
    }
}

impl Drop for DisposableLibraryDirectory {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_dir_all(&self.directory) {
            if error.kind() != std::io::ErrorKind::NotFound {
                eprintln!("删不掉一次性库目录 {}：{error}", self.directory.display());
            }
        }
    }
}

/// 这个进程里建过几份独占的一次性临时目录（[`ExclusiveTemporaryDirectory::create`] 每次取一个号，名字里带它）。
static EXCLUSIVE_TEMPORARY_DIRECTORIES_CREATED_IN_THIS_PROCESS: AtomicU64 = AtomicU64::new(0);

/// 建一份独占的一次性临时目录最多试几个名字：名字撞上已有的目录（同进程号的旧进程被杀时留下的）就换下一个号，试满还建不出就报错。
const MOST_EXCLUSIVE_TEMPORARY_DIRECTORY_NAMES_TRIED: u32 = 64;

/// 调用方独占的一份一次性临时目录：`${TMPDIR:-/tmp}` 下 `singlefs-<标签>-<进程号>-<进程内序号>`，`create_dir` 排他新建；
/// 丢掉时只删这一份（连同里面的东西）。同一个进程里两次用同一个标签各拿一份、互不删对方；两个进程同跑一个测试二进制，进程号不同。
/// 标签只为在临时目录里认得出是谁建的，不靠它区分：checker 档单测与集成测试要一份自己的临时目录都经它建
/// （照崩溃注入镜像目录的写法，`crate::crash_injection` 的 `create_the_image_directory_of_a_campaign_call`）。
pub struct ExclusiveTemporaryDirectory {
    path: PathBuf,
}

impl ExclusiveTemporaryDirectory {
    /// # Errors
    /// 临时目录里建不了目录；试满 [`MOST_EXCLUSIVE_TEMPORARY_DIRECTORY_NAMES_TRIED`] 个名字都撞上已有的。
    pub fn create(label: &str) -> Result<Self, String> {
        let mut last_name_tried = PathBuf::new();
        // 迭代上界：MOST_EXCLUSIVE_TEMPORARY_DIRECTORY_NAMES_TRIED 次；只有名字撞上已有目录时才试下一个
        for _name_tried in 0..MOST_EXCLUSIVE_TEMPORARY_DIRECTORY_NAMES_TRIED {
            let serial_in_this_process = EXCLUSIVE_TEMPORARY_DIRECTORIES_CREATED_IN_THIS_PROCESS
                .fetch_add(1, Ordering::SeqCst);
            let path = std::env::temp_dir().join(format!(
                "singlefs-{label}-{}-{serial_in_this_process}",
                std::process::id()
            ));
            match std::fs::create_dir(&path) {
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

/// 库的家目录：[`CRASH_LIBRARY_HOME_ENVIRONMENT_VARIABLE`]，没设是 `$HOME/`[`DEFAULT_HOME_UNDER_THE_USER_HOME_DIRECTORY`]。
///
/// # Errors
/// 变量设了却是空串；没设而环境里没有 `HOME`；落在开机会清空的目录之下。
pub fn crash_library_home() -> Result<PathBuf, String> {
    let home = match std::env::var_os(CRASH_LIBRARY_HOME_ENVIRONMENT_VARIABLE) {
        Some(home) if home.is_empty() => {
            return Err(format!(
                "{CRASH_LIBRARY_HOME_ENVIRONMENT_VARIABLE} 设了却是空串：写库的家目录，或不设"
            ));
        }
        Some(home) => PathBuf::from(home),
        None => {
            let user_home = std::env::var_os("HOME").ok_or_else(|| {
                format!(
                    "环境里没有 HOME，也没设 {CRASH_LIBRARY_HOME_ENVIRONMENT_VARIABLE}：判定库不知道放哪"
                )
            })?;
            PathBuf::from(user_home).join(DEFAULT_HOME_UNDER_THE_USER_HOME_DIRECTORY)
        }
    };
    refuse_a_library_that_a_reboot_would_erase(&home)?;
    Ok(home)
}

/// 那一份库的目录：[`CRASH_LIBRARY_DIRECTORY_ENVIRONMENT_VARIABLE`] 设了就是它（只收绝对路径）；没设是家目录下的
/// [`CRASH_LIBRARY_DIRECTORY_NAME`]。几类测试都是这一份。
///
/// # Errors
/// 同 [`crash_library_home`]；变量给的是相对路径（仓根不是库的输入，不按仓根或当前目录去补）；变量给的目录落在开机会清空的地方而不是一次性的。
pub fn crash_library_directory() -> Result<PathBuf, String> {
    if let Ok(text) = std::env::var(CRASH_LIBRARY_DIRECTORY_ENVIRONMENT_VARIABLE) {
        let directory = absolute_library_directory_of_the_variable_value(
            CRASH_LIBRARY_DIRECTORY_ENVIRONMENT_VARIABLE,
            &text,
        )?;
        refuse_a_library_that_a_reboot_would_erase(&directory)?;
        return Ok(directory);
    }
    Ok(crash_library_home()?.join(CRASH_LIBRARY_DIRECTORY_NAME))
}

/// 环境变量里交的一份库的目录（本机这一份、要导入的那几份）：只收绝对路径，相对路径拒绝（仓根不是库的输入，测试进程的当前目录是包目录、
/// 第二台的驱动在树副本里起，按哪一个补都不是调用方以为的那一处）。
///
/// # Errors
/// `text` 是空串或相对路径。
pub fn absolute_library_directory_of_the_variable_value(
    variable_name: &str,
    text: &str,
) -> Result<PathBuf, String> {
    let directory = Path::new(text);
    if directory.is_absolute() {
        Ok(directory.to_path_buf())
    } else {
        Err(format!(
            "{variable_name}={text:?} 不是绝对路径：库目录只收绝对路径（仓根不是库的输入，不按仓根或当前目录去补）。\
             出路：第二台的驱动给绝对路径（树副本里的库写成 <树副本的绝对路径>/<库目录>），入口脚本交 $(realpath …) 之后的那一份"
        ))
    }
}

// ============================== 这个进程里开着的库 ==============================

/// 这个进程里开着的判定库：按库目录（上层目录取规范路径）各记一份。RocksDB 的锁一个进程只拿得到一次，库目录相同的几处
/// （同一个进程里并行跑的几条用例）共用这一份打开的库，判定、复用、写入都经它，库里的一把锁让它们一次一个读写。
static LIBRARIES_OPEN_IN_THIS_PROCESS: Mutex<BTreeMap<PathBuf, Weak<Mutex<VerdictStore>>>> =
    Mutex::new(BTreeMap::new());

/// 一处手里的那份库：这个进程里同一个库目录已经开着就共用那一份，没开着才开（库在就开、不在就建）；最后一个放手的关库。
/// 放手在登记表的锁里做：下一个来开同一个目录的，要么拿到还开着的那一份，要么等这一份关完再开，不会撞上关到一半的锁。
/// 别的进程持着这份库的锁时等它放手（[`CRASH_LIBRARY_LOCK_WAIT_SECONDS_ENVIRONMENT_VARIABLE`]），每分钟打一行。
pub struct LibraryOpenInThisProcess {
    pub(crate) key: PathBuf,
    store: Option<Arc<Mutex<VerdictStore>>>,
}

/// 开的一份库是哪一种：本机这一份（不在就建、旧布局拒绝），还是别的机器取回、要从里面导入的那一份（不在报错、不建，布局不挑：
/// 按计划导入自己认这一套之前的库，[`VerdictStore::import_the_planned_rows_from`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LibraryRole {
    ThisHostsLibrary,
    LibraryToImportFrom,
}

/// RocksDB 报的错是不是别的进程持着锁（`IO error: While lock file: …/LOCK: Resource temporarily unavailable`）。
fn is_a_lock_held_by_another_process(error_text: &str) -> bool {
    let lowered = error_text.to_ascii_lowercase();
    lowered.contains("lock")
        && (lowered.contains("temporarily unavailable")
            || lowered.contains("lock hold")
            || lowered.contains("held by"))
}

/// 等锁的上限。
///
/// # Panics
/// 变量设了却不是十进制整数。
fn lock_wait_limit() -> Duration {
    match std::env::var(CRASH_LIBRARY_LOCK_WAIT_SECONDS_ENVIRONMENT_VARIABLE) {
        Ok(text) => Duration::from_secs(text.trim().parse::<u64>().unwrap_or_else(|error| {
            panic!(
                "{CRASH_LIBRARY_LOCK_WAIT_SECONDS_ENVIRONMENT_VARIABLE}={text:?} 不是十进制整数秒：{error}"
            )
        })),
        Err(_) => Duration::from_secs(DEFAULT_LOCK_WAIT_SECONDS),
    }
}

impl LibraryOpenInThisProcess {
    /// 开 `library_directory` 那一份库，这个进程里已经开着就共用。
    ///
    /// # Errors
    /// 本机这一份：建不了上层目录；库在而开不了（按结论种类存之前的旧库拒绝，不迁移；别的进程持着锁、等过上限还没放）；库不在而建不了。
    /// 要导入的那一份：上层目录取不了规范路径（目录不在）；库不在、开不了。
    pub fn open_or_share(library_directory: &Path, role: LibraryRole) -> Result<Self, String> {
        let parent = library_directory.parent().unwrap_or(Path::new("."));
        match role {
            LibraryRole::ThisHostsLibrary => std::fs::create_dir_all(parent)
                .map_err(|error| format!("建不了库的上层目录：{error}"))?,
            LibraryRole::LibraryToImportFrom => {}
        }
        let key = std::fs::canonicalize(parent)
            .map_err(|error| {
                format!(
                    "取不了库的上层目录 {} 的规范路径：{error}",
                    parent.display()
                )
            })?
            .join(library_directory.file_name().unwrap_or_default());
        let mut open_libraries = LIBRARIES_OPEN_IN_THIS_PROCESS
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if let Some(store) = open_libraries.get(&key).and_then(Weak::upgrade) {
            return Ok(Self {
                key,
                store: Some(store),
            });
        }
        let store = Self::open_waiting_for_the_lock(library_directory, role)?;
        let store = Arc::new(Mutex::new(store));
        open_libraries.insert(key.clone(), Arc::downgrade(&store));
        Ok(Self {
            key,
            store: Some(store),
        })
    }

    /// 开或建这份库；别的进程持着锁就隔 [`LOCK_RETRY_INTERVAL`] 再试，到 [`lock_wait_limit`] 为止，每隔 [`LOCK_WAIT_LINE_INTERVAL`] 打一行。
    fn open_waiting_for_the_lock(
        library_directory: &Path,
        role: LibraryRole,
    ) -> Result<VerdictStore, String> {
        let started = Instant::now();
        let limit = lock_wait_limit();
        let mut last_line_at = started;
        loop {
            let opened: Result<VerdictStore, String> = match role {
                LibraryRole::ThisHostsLibrary if library_directory.join("CURRENT").exists() => {
                    VerdictStore::open_existing_with_conclusion_kinds(library_directory)
                        .map(|(store, _)| store)
                        .map_err(|error| {
                            format!(
                                "开不了库 {}（按结论种类存之前的、判定向量编号还是一个字节的旧布局库拒绝，不迁移：换一个新目录建库）：{error:?}",
                                library_directory.display()
                            )
                        })
                }
                LibraryRole::ThisHostsLibrary => VerdictStore::create_empty(library_directory)
                    .map_err(|error| {
                        format!("建不了库 {}：{error:?}", library_directory.display())
                    }),
                LibraryRole::LibraryToImportFrom => VerdictStore::open_existing(library_directory)
                    .map(|(store, _)| store)
                    .map_err(|error| {
                        format!(
                            "开不了要导入的库 {}：{error:?}",
                            library_directory.display()
                        )
                    }),
            };
            match opened {
                Ok(store) => return Ok(store),
                Err(reason) if is_a_lock_held_by_another_process(&reason) => {
                    let waited = started.elapsed();
                    if waited >= limit {
                        return Err(format!(
                            "{reason}；等了 {} 秒别的进程还持着这份库的锁（上限 {} 秒，{CRASH_LIBRARY_LOCK_WAIT_SECONDS_ENVIRONMENT_VARIABLE} 可改）：等那个进程跑完再起",
                            waited.as_secs(),
                            limit.as_secs()
                        ));
                    }
                    if last_line_at.elapsed() >= LOCK_WAIT_LINE_INTERVAL {
                        eprintln!(
                            "CRASH_LIBRARY_WAITING_FOR_THE_LOCK library={} waited_seconds={}",
                            library_directory.display(),
                            waited.as_secs()
                        );
                        last_line_at = Instant::now();
                    }
                    std::thread::sleep(LOCK_RETRY_INTERVAL);
                }
                Err(reason) => return Err(reason),
            }
        }
    }

    /// 库里那一把锁：一次一个读写。
    pub fn lock(&self) -> MutexGuard<'_, VerdictStore> {
        self.store
            .as_ref()
            .expect("库在放手之前一直在：只有 Drop 取走它")
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}

impl Drop for LibraryOpenInThisProcess {
    fn drop(&mut self) {
        let mut open_libraries = LIBRARIES_OPEN_IN_THIS_PROCESS
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        drop(self.store.take());
        if open_libraries
            .get(&self.key)
            .is_some_and(|open| open.strong_count() == 0)
        {
            open_libraries.remove(&self.key);
        }
    }
}

/// 一类测试手里的那份库：几类都开在 [`crash_library_directory`] 那一份上，同一个进程里共用；类别只进行里报的名字。
pub struct CrashLibrary {
    class: CrashLibraryClass,
    directory: PathBuf,
    library: LibraryOpenInThisProcess,
}

impl CrashLibrary {
    /// 在给定的目录上开库（不在就建）：一次性的库用（判器那一半拒算的那一趟在它上面录、判，跑完连库删掉）。
    ///
    /// # Errors
    /// 同 [`LibraryOpenInThisProcess::open_or_share`]。
    pub fn open_at(class: CrashLibraryClass, directory: &Path) -> Result<Self, String> {
        let library =
            LibraryOpenInThisProcess::open_or_share(directory, LibraryRole::ThisHostsLibrary)?;
        Ok(Self {
            class,
            directory: directory.to_path_buf(),
            library,
        })
    }

    /// 开这一类的库（不在就建）。
    ///
    /// # Errors
    /// 同 [`crash_library_directory`] 与 [`LibraryOpenInThisProcess::open_or_share`]。
    pub fn open(class: CrashLibraryClass) -> Result<Self, String> {
        let directory = crash_library_directory()?;
        let library =
            LibraryOpenInThisProcess::open_or_share(&directory, LibraryRole::ThisHostsLibrary)?;
        Ok(Self {
            class,
            directory,
            library,
        })
    }

    #[must_use]
    pub fn class(&self) -> CrashLibraryClass {
        self.class
    }

    #[must_use]
    pub fn directory(&self) -> &Path {
        &self.directory
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

/// 只读地开那一份库（崩溃枚举用例只查块键那一趟用）：库不在交回 `None`（一块都没有判定）；在就只读地开，不拿锁、库目录里一个文件都不动，
/// RocksDB 的运行日志写到 `log_directory`。
///
/// # Errors
/// 同 [`crash_library_directory`]；库在而只读开不了。
pub fn open_the_library_for_reading_only(
    log_directory: &Path,
) -> Result<Option<VerdictStore>, String> {
    let directory = crash_library_directory()?;
    if !directory.join("CURRENT").exists() {
        return Ok(None);
    }
    VerdictStore::open_existing_for_reading_only(&directory, log_directory)
        .map(Some)
        .map_err(|error| format!("只读开不了库 {}：{error:?}", directory.display()))
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

/// 过一次库的结局：只录那一趟只把这一项写进库、不判；判段从库里读回判完交回判的结果；两台分片时这台只判自己领的块，整份的结果要等导入。
#[must_use]
#[derive(Debug)]
pub enum StageOutcome<Judged> {
    RecordedOnly,
    /// 两台分片（`SINGLEFS_CRASH_AMPLIFICATION_SHARE=<i>/<n>`，没给别的机器取回的库）：这台机器领的块判完、存进库了，
    /// 别的机器领的块不在库里，整份的结果交不出来；调用方拿到它就收工，不拿它判用例钉死的数。
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
            self.directory.display()
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
                self.directory.display(),
                hexadecimal_text(&inputs.0[..8])
            )
        })
    }
}

/// 两段过一次库：录入段（`record`，CPU 跑数据）把这一项的几样写进库、交回清单，这里按录入键登清单；录入键下已有清单就不再录
/// （命中就跳过，打 `CRASH_LIBRARY_RECORDING_REUSED` 与 `CRASH_LIBRARY_RECORDED` 两行；只写不读那一趟照录）。判段（`judge`）照同一份输入
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
        let stored = if recording.skips_the_recording_on_a_hit
            && !library_reads_are_skipped_in_this_process()
        {
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
            library.directory.display(),
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
    /// 这一趟判的（库里没有、只写不读那一趟、或两边哪一边拒算），判完按键存进库（拒算的不存）。
    JudgedNow,
}

/// 对拍结论入库（用户 2026-10-15 定「对拍结论入库」）：键是（输入摘要，两边判法单元指纹），库里有就复用、不再判；没有就调 `judge`
/// 判一遍、按键存进库。两边哪一边的判法单元拒算：照判、不入库（不退回整文件指纹、不跳过）。交回结论的字节与它从哪来。
///
/// # Errors
/// 库读写失败。
pub fn pairing_conclusion_reused_or_judged(
    library: &CrashLibrary,
    inputs_digest: &[u8; DIGEST_BYTES],
    one_side: &Result<JudgingUnitsKey, String>,
    other_side: &Result<JudgingUnitsKey, String>,
    judge: impl FnOnce() -> Vec<u8>,
) -> Result<(Vec<u8>, PairingConclusionSource), String> {
    let (Ok(one_side), Ok(other_side)) = (one_side, other_side) else {
        println!(
            "CRASH_LIBRARY_PAIRING_KEY_REFUSED class={} inputs={}：判了、不入库",
            library.class.name(),
            hexadecimal_text(&inputs_digest[..8])
        );
        return Ok((judge(), PairingConclusionSource::JudgedNow));
    };
    let key = PairingConclusionKey {
        inputs_digest: *inputs_digest,
        one_side_judging_units: *one_side.fingerprint(),
        other_side_judging_units: *other_side.fingerprint(),
    };
    if !library_reads_are_skipped_in_this_process() {
        if let Some(stored) = library
            .lock()
            .read_pairing_conclusion(&key)
            .map_err(|error| format!("读对拍结论失败：{error:?}"))?
        {
            return Ok((
                stored.conclusion,
                PairingConclusionSource::ReusedFromTheLibrary,
            ));
        }
    }
    let conclusion = judge();
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
                    conclusion: conclusion.clone(),
                    circumstances: JudgementCircumstances::of_this_process_now(),
                },
            )
            .map_err(|error| format!("对拍结论写不进库：{error:?}"))?;
    }
    Ok((conclusion, PairingConclusionSource::JudgedNow))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crash_amplification::CrashAmplificationEnvironment;
    use crate::crash_identity::{ConclusionKind, ConclusionVersion, JudgingUnitFingerprint};
    use crate::verdict_store::{
        BlockCountVector, BlockStartStateIndex, ConclusionBlockRecord, EncodedVerdictVector,
        EnumerationPlanHash, JudgingDevice, NodeInputDigest, VerdictBlock, VerdictBlockKey,
    };

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

    /// 定案 1：库目录的变量只收绝对路径。给相对路径（第二台的驱动照旧写 `crash-amplification-library` 那种）拒绝、红句点名变量、
    /// 原值与出路「第二台的驱动给绝对路径」，不按仓根或当前目录去补；给绝对路径就是它；没设是家目录下那一份。
    /// 判别力（`crates/mutations.tsv`）：相对路径照旧按当前目录补成绝对路径，这条红。
    #[test]
    fn a_relative_library_directory_is_refused_and_an_absolute_one_is_taken_as_given() {
        let disposable = DisposableLibraryDirectory::new("relative-library-directory")
            .expect("建得了一次性目录");
        let home = disposable.directory().join("home");
        let mut environment = CrashAmplificationEnvironment::cleared_then_set(&[
            (
                CRASH_LIBRARY_DIRECTORY_ENVIRONMENT_VARIABLE,
                std::ffi::OsStr::new("crash-amplification-library"),
            ),
            (CRASH_LIBRARY_HOME_ENVIRONMENT_VARIABLE, home.as_os_str()),
        ]);
        let refused = crash_library_directory();
        assert!(
            refused.as_ref().is_err_and(|reason| reason
                .contains(CRASH_LIBRARY_DIRECTORY_ENVIRONMENT_VARIABLE)
                && reason.contains("crash-amplification-library")
                && reason.contains("第二台的驱动给绝对路径")),
            "相对路径要拒、红句点名变量、原值与出路：{refused:?}"
        );
        let absolute = disposable.library();
        environment.set_variable(
            CRASH_LIBRARY_DIRECTORY_ENVIRONMENT_VARIABLE,
            absolute.as_os_str(),
        );
        assert_eq!(crash_library_directory(), Ok(absolute), "绝对路径照用");
        drop(environment);
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

    #[test]
    fn a_lock_held_by_another_process_is_told_apart_from_other_errors() {
        assert!(is_a_lock_held_by_another_process(
            "RocksDatabase(Error { message: \"IO error: While lock file: /x/LOCK: Resource temporarily unavailable\" })"
        ));
        assert!(!is_a_lock_held_by_another_process(
            "IO error: No such file or directory"
        ));
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

    /// 一份库、类当索引（用户 2026-10-15 定）：几类测试开的是同一个目录，目录名固定、不带类名；没交目录时落在家目录下那一个固定的名字；
    /// 同一块的 GPU 与 CPU 两种结论在这一份库里是同一块下的两条记录、各自读得回。
    #[test]
    fn every_class_opens_the_one_library_and_two_conclusion_kinds_of_one_block_are_two_records() {
        let disposable = DisposableLibraryDirectory::new("one-library").expect("建得了一次性目录");
        let library_directory = disposable.library();
        let home = disposable.directory().join("home");
        let _environment = CrashAmplificationEnvironment::cleared_then_set(&[
            (
                CRASH_LIBRARY_DIRECTORY_ENVIRONMENT_VARIABLE,
                library_directory.as_os_str(),
            ),
            (CRASH_LIBRARY_HOME_ENVIRONMENT_VARIABLE, home.as_os_str()),
        ]);
        let amplification =
            CrashLibrary::open(CrashLibraryClass::CrashAmplification).expect("开得了库");
        let enumeration =
            CrashLibrary::open(CrashLibraryClass::CrashStateEnumeration).expect("开得了库");
        let given_states = CrashLibrary::open(CrashLibraryClass::GivenStates).expect("开得了库");
        assert_eq!(amplification.directory(), enumeration.directory());
        assert_eq!(amplification.directory(), given_states.directory());
        assert_eq!(amplification.directory(), library_directory.as_path());
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
        drop(store);
        drop((amplification, enumeration, given_states));
        std::env::remove_var(CRASH_LIBRARY_DIRECTORY_ENVIRONMENT_VARIABLE);
        let default_directory = crash_library_directory().expect("家目录下那一份");
        assert_eq!(default_directory, home.join(CRASH_LIBRARY_DIRECTORY_NAME));
        for class in [
            CrashLibraryClass::CrashAmplification,
            CrashLibraryClass::CrashStateEnumeration,
            CrashLibraryClass::GivenStates,
            CrashLibraryClass::JudgeTablesPairing,
            CrashLibraryClass::FactsPairing,
        ] {
            assert!(
                !default_directory.to_string_lossy().contains(class.name()),
                "目录名不带类名 {}",
                class.name()
            );
        }
    }

    /// 录入段按（录入函数单元指纹，录入输入摘要）入库（L11）：同一份输入第二趟命中、录入那一段不再跑；输入改一个字节第二趟照录；
    /// 录入函数单元拒算时不复用、照录。
    #[test]
    fn the_recording_is_skipped_on_a_hit_and_redone_when_one_input_byte_changes() {
        let disposable =
            DisposableLibraryDirectory::new("recording-reuse").expect("建得了一次性目录");
        let library_directory = disposable.library();
        let _environment = CrashAmplificationEnvironment::cleared_then_set(&[(
            CRASH_LIBRARY_DIRECTORY_ENVIRONMENT_VARIABLE,
            library_directory.as_os_str(),
        )]);
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
        let disposable =
            DisposableLibraryDirectory::new("pairing-reuse").expect("建得了一次性目录");
        let library_directory = disposable.library();
        let _environment = CrashAmplificationEnvironment::cleared_then_set(&[(
            CRASH_LIBRARY_DIRECTORY_ENVIRONMENT_VARIABLE,
            library_directory.as_os_str(),
        )]);
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
            format!("agree {}", judgements.get()).into_bytes()
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
}
