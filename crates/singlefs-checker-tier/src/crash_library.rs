//! 崩溃判定库（RocksDB）：几类测试各一份长期存放、按键复用、能剪枝的库；录入段与判段只经库交接。
//!
//! 规矩（用户 2026-10-11 定）：① 测试先把 CPU 跑出来的数据写进库，再从库里读出来判；② 每一类测试都做剪枝，库里的内容与崩溃放量同一种
//! （按块键、结论种类、版本存结论，判过的复用）；③ 几类测试分开存，各按自己的特性；④ CPU 产出的数据一律进库，判段按批从库里取出来交给 GPU，
//! 没有 GPU 才交给 CPU；⑤ 两段之间不在内存里交数据，CPU 自己判也分两段；没有「临时库」这一说。
//!
//! 库的家目录是 [`CRASH_LIBRARY_HOME_ENVIRONMENT_VARIABLE`]（没设是家目录下 `.local/share/singlefs/crash-amplification`），每一类
//! （[`CrashLibraryClass`]）在家目录下各一份子目录；崩溃放量与给定状态表两类另认 [`CRASH_LIBRARY_DIRECTORY_ENVIRONMENT_VARIABLE`]
//! （入口脚本按趟交的库目录）。落在开机会清空的目录里的库拒绝（[`refuse_a_library_that_a_reboot_would_erase`]）。
//!
//! 两段怎么接（[`two_stages`]）：录入段把这一项的几样各按摘要写进库、再按项名登一份清单（[`ItemManifest`]）；判段只凭项名读清单、
//! 再按清单里的摘要读回那几样。两段之间只过一个项名，项名是调用方写死的字面量，不是哪一段算出来的。选段照
//! [`RUN_STAGE_ENVIRONMENT_VARIABLE`]：`record` 只录、`judge` 只判、不设两段先后跑（中间照样经库）。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError, Weak};
use std::time::{Duration, Instant};

use singlefs_harness::hexadecimal::hexadecimal_text;
use singlefs_harness::sha256::{sha256_digest, DIGEST_BYTES};

use crate::crash_facts::{FlowFacts, FlowFactsReadBack};
#[cfg(feature = "gpu")]
use crate::crash_judge_dispatch::{PackedJudgeTables, PackedJudgeTablesReadBack};
use crate::verdict_store::VerdictStore;

/// 库的家目录从这个环境变量取；没设是家目录下 [`DEFAULT_HOME_UNDER_THE_USER_HOME_DIRECTORY`]。入口脚本（`scripts/crash/crash-amplification-lib.sh`）读的是同一个变量。
pub const CRASH_LIBRARY_HOME_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_CRASH_AMPLIFICATION_HOME";

/// 崩溃放量与给定状态表两类的库目录由入口脚本按趟交进来时用的变量（相对路径按仓根解析）。别的类不认它：它们的库只在家目录下。
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

/// 开库等锁的默认上限：半小时。同一份库同时只许一个进程开着，别的进程排队。
pub const DEFAULT_LOCK_WAIT_SECONDS: u64 = 1800;

/// 等锁时隔多久再试一次。
const LOCK_RETRY_INTERVAL: Duration = Duration::from_secs(2);

/// 等锁时隔多久打一行。
const LOCK_WAIT_LINE_INTERVAL: Duration = Duration::from_secs(60);

/// 开机会清空的目录：库落在它们之下重启一次就续不上。
const DIRECTORIES_A_REBOOT_ERASES: [&str; 4] = ["/tmp", "/var/tmp", "/dev/shm", "/run"];

/// 库目录落在开机会清空的地方就拒绝，除非环境写明这份库是一次性的（`SINGLEFS_CRASH_AMPLIFICATION_LIBRARY_IS_TEMPORARY=1`，只给
/// 入口脚本的自证用）。判的是现存最长前缀的绝对路径（目录还没建时判它的上层）。
///
/// # Errors
/// 库目录落在 [`DIRECTORIES_A_REBOOT_ERASES`] 之下而环境没写明一次性。
pub fn refuse_a_library_that_a_reboot_would_erase(library_directory: &Path) -> Result<(), String> {
    if std::env::var("SINGLEFS_CRASH_AMPLIFICATION_LIBRARY_IS_TEMPORARY")
        .is_ok_and(|value| value == "1")
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

/// 仓根：这个 crate 往上两层（变异复跑的源码副本里 `Cargo.lock` 也在那里）。用例与实验装置开库、取结论版本都按它。
///
/// # Panics
/// 取不到。
#[must_use]
pub fn repository_root_of_this_crate() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../Cargo.lock")
        .canonicalize()
        .expect("工作区的 Cargo.lock 在 crate 往上两层（变异复跑的源码副本里也有它）")
        .parent()
        .expect("Cargo.lock 所在的目录就是仓根")
        .to_path_buf()
}

// ============================== 几类测试各一份库 ==============================

/// 哪一类测试的库：各按自己的特性存、各自复用，互不混。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CrashLibraryClass {
    /// 崩溃放量流水线（`crash_amplification::run_from_environment`）：按崩溃点的块键存结论，入口脚本按趟交库目录。
    CrashAmplification,
    /// 崩溃点重放枚举用例（`crash_state_enumeration`）：整条流当一个崩溃点、按块存结论，判过的块下一趟复用。
    CrashStateEnumeration,
    /// 随机历史两项的给定状态表（`crash_amplification::GivenStatesEngine`）：一张表一块。
    GivenStates,
    /// GPU 判器与 CPU 判器的对拍用例：判表先入库、再读库上卡。
    JudgeTablesPairing,
    /// 第 ② 段核对的 GPU 与 CPU 参照对拍用例：事实表先入库、再读库核。
    FactsPairing,
    /// 崩溃放量流水线自己的单元测试（录入、核对、复用、两台分片与导入、叫停）：与入口脚本跑的那几项分开存，
    /// 用例按标签各占底下一份（[`amplification_unit_tests_library_directory`]）。
    AmplificationUnitTests,
}

impl CrashLibraryClass {
    /// 家目录下这一类库的子目录名。崩溃放量那一类沿用入口脚本登记的子目录（`CRASH_AMPLIFICATION_LIBRARY_SUBDIRECTORY`）。
    #[must_use]
    pub fn directory_name(self) -> &'static str {
        match self {
            Self::CrashAmplification => "library-by-conclusion-kind",
            Self::CrashStateEnumeration => "crash-state-enumeration-library",
            Self::GivenStates => "given-states-library",
            Self::JudgeTablesPairing => "judge-tables-pairing-library",
            Self::FactsPairing => "facts-pairing-library",
            Self::AmplificationUnitTests => "amplification-unit-tests-library",
        }
    }

    /// 行里报的名字。
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::CrashAmplification => "crash_amplification",
            Self::CrashStateEnumeration => "crash_state_enumeration",
            Self::GivenStates => "given_states",
            Self::JudgeTablesPairing => "judge_tables_pairing",
            Self::FactsPairing => "facts_pairing",
            Self::AmplificationUnitTests => "amplification_unit_tests",
        }
    }

    /// 这一类认不认 [`CRASH_LIBRARY_DIRECTORY_ENVIRONMENT_VARIABLE`]：入口脚本与双机驱动按趟交库目录的那三类认
    /// （崩溃枚举用例的全量由门禁 checker-tier-crash-point-replay 经崩溃放量的双机驱动跑，第二台那一份库按它交的相对路径落在树副本里再取回）。
    #[must_use]
    pub fn honours_the_directory_variable(self) -> bool {
        match self {
            Self::CrashAmplification | Self::GivenStates | Self::CrashStateEnumeration => true,
            Self::JudgeTablesPairing | Self::FactsPairing | Self::AmplificationUnitTests => false,
        }
    }
}

/// 流水线单元测试那一类的库目录：家目录下 [`CrashLibraryClass::AmplificationUnitTests`] 的子目录，`sub_path` 给了就是它底下那一份
/// （用例标签；两台分片的用例再往下分角色）。长期库：用例跑完不删，下一趟照同一套复用。按环境跑的用例把它设进
/// [`CRASH_LIBRARY_DIRECTORY_ENVIRONMENT_VARIABLE`]。
///
/// # Errors
/// 同 [`crash_library_home`]。
pub fn amplification_unit_tests_library_directory(
    sub_path: Option<&str>,
) -> Result<PathBuf, String> {
    let class_directory =
        crash_library_home()?.join(CrashLibraryClass::AmplificationUnitTests.directory_name());
    Ok(match sub_path {
        Some(sub_path) => class_directory.join(sub_path),
        None => class_directory,
    })
}

/// 同 [`amplification_unit_tests_library_directory`]，起跑先清空这一份：钉「空库起跑」那一套账的用例（第一趟全判、第二趟全复用、
/// 两台各判各的再导入、叫停只判到哪一块）用。清的只有用例标签命名的这一份，跑完留着不删（库在家目录下，用例结束后看得到）。
///
/// # Errors
/// 同 [`crash_library_home`]；清不掉或建不了这一份的目录。
pub fn amplification_unit_tests_library_directory_emptied_at_the_start(
    sub_path: &str,
) -> Result<PathBuf, String> {
    let directory = amplification_unit_tests_library_directory(Some(sub_path))?;
    if directory.exists() {
        std::fs::remove_dir_all(&directory)
            .map_err(|error| format!("清不掉用例的库 {}：{error}", directory.display()))?;
    }
    std::fs::create_dir_all(&directory)
        .map_err(|error| format!("建不了用例的库目录 {}：{error}", directory.display()))?;
    Ok(directory)
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

/// 这一类库的目录：认 [`CRASH_LIBRARY_DIRECTORY_ENVIRONMENT_VARIABLE`] 的类，变量设了就是它（相对路径按 `repository_root` 解析：
/// cargo 把测试进程的当前目录设成包目录，第二台的驱动只知道树根）；其余照家目录下这一类的子目录。
///
/// # Errors
/// 同 [`crash_library_home`]；变量给的目录落在开机会清空的地方。
pub fn crash_library_directory_of(
    class: CrashLibraryClass,
    repository_root: &Path,
) -> Result<PathBuf, String> {
    if class.honours_the_directory_variable() {
        if let Ok(text) = std::env::var(CRASH_LIBRARY_DIRECTORY_ENVIRONMENT_VARIABLE) {
            let directory = if Path::new(&text).is_absolute() {
                PathBuf::from(&text)
            } else {
                repository_root.join(&text)
            };
            refuse_a_library_that_a_reboot_would_erase(&directory)?;
            return Ok(directory);
        }
    }
    Ok(crash_library_home()?.join(class.directory_name()))
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
                                "开不了库 {}（按结论种类存之前的旧库拒绝，不迁移：换一个新目录建库）：{error:?}",
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

/// 一类测试手里的那份库：开在这一类的目录下（[`crash_library_directory_of`]），同一个进程里共用。
pub struct CrashLibrary {
    class: CrashLibraryClass,
    directory: PathBuf,
    library: LibraryOpenInThisProcess,
}

impl CrashLibrary {
    /// 开这一类的库（不在就建）。
    ///
    /// # Errors
    /// 同 [`crash_library_directory_of`] 与 [`LibraryOpenInThisProcess::open_or_share`]。
    pub fn open(class: CrashLibraryClass, repository_root: &Path) -> Result<Self, String> {
        let directory = crash_library_directory_of(class, repository_root)?;
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

// ============================== 项名与清单 ==============================

/// 一项的名字：录入段按它登清单，判段凭它找。调用方写死的字面量（用例名、流名），1 到 200 字节，只许字母、数字、`_`、`.`、`:`、`/`、`-`。
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

/// 录入段按项名登在库里的清单：这一项是哪一类（`kind`，一个写死的串），录下的几样各叫什么、摘要是什么。判段只认清单指到的那几样。
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

    /// 清单自己的摘要（同一个进程里查项名撞不撞用）。
    #[must_use]
    pub fn digest(&self) -> [u8; DIGEST_BYTES] {
        sha256_digest(&self.to_bytes())
    }
}

/// 这个进程里录过的项：（类，项名）→ 清单摘要。同一个进程里两处用同一个项名录下不同的清单，是项名撞了（几条用例共用一个名字），
/// 判段会读到别人的那一份：当场停下。
static ITEMS_RECORDED_IN_THIS_PROCESS: Mutex<
    BTreeMap<(CrashLibraryClass, String), [u8; DIGEST_BYTES]>,
> = Mutex::new(BTreeMap::new());

impl CrashLibrary {
    /// 录入段登这一项的清单（键是项名；同一项名再登就盖掉上一趟的，判段读到的是最近一次登的）。
    ///
    /// # Errors
    /// 库写失败。
    ///
    /// # Panics
    /// 同一个进程里同一个项名已经录过另一份清单（项名撞了）。
    pub fn record_the_item(&self, item: &ItemName, manifest: &ItemManifest) -> Result<(), String> {
        let digest = manifest.digest();
        {
            let mut recorded = ITEMS_RECORDED_IN_THIS_PROCESS
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            match recorded.insert((self.class, item.as_str().to_string()), digest) {
                Some(earlier) if earlier != digest => panic!(
                    "项名撞了：{} 类里「{}」这一趟已经录过另一份清单（{} 与 {}）——几条用例共用了一个项名，判段会读到别人的那一份；每条用例各给一个项名",
                    self.class.name(),
                    item.as_str(),
                    hexadecimal_text(&earlier[..8]),
                    hexadecimal_text(&digest[..8])
                ),
                Some(_) | None => {}
            }
        }
        self.lock()
            .store_recorded_run_manifest(item.as_str(), &manifest.to_bytes())
            .map_err(|error| format!("项「{}」的清单写不进库：{error:?}", item.as_str()))?;
        println!(
            "CRASH_LIBRARY_RECORDED class={} item={} manifest={} library={}",
            self.class.name(),
            item.as_str(),
            hexadecimal_text(&digest[..8]),
            self.directory.display()
        );
        Ok(())
    }

    /// 判段凭项名读清单。
    ///
    /// # Errors
    /// 库读失败；库里没有这一项的清单（录入段没跑）；清单解不出。
    pub fn manifest_of_the_item(&self, item: &ItemName) -> Result<ItemManifest, String> {
        let bytes = self
            .lock()
            .read_recorded_run_manifest(item.as_str())
            .map_err(|error| format!("读项「{}」的清单失败：{error:?}", item.as_str()))?
            .ok_or_else(|| {
                format!(
                    "{} 类的库 {} 里没有项「{}」的清单：先跑录入段（{RUN_STAGE_ENVIRONMENT_VARIABLE}=record，或不设、两段先后跑）",
                    self.class.name(),
                    self.directory.display(),
                    item.as_str()
                )
            })?;
        ItemManifest::from_bytes(&bytes).ok_or_else(|| {
            format!(
                "项「{}」的清单解不出（字节坏了或编码换了）：重跑录入段",
                item.as_str()
            )
        })
    }
}

/// 两段过一次库：录入段（`record`，CPU 跑数据）把这一项的几样写进库、交回清单，这里按项名登清单；判段（`judge`）凭项名读回清单，
/// 只拿读回的那一份往下判。两段之间只过项名。选段照 [`RunStage::of_this_process`]。
///
/// # Errors
/// 开不了这一类的库；录入段、登清单、读清单、判段各自的错。
///
/// # Panics
/// 选段写坏了；项名撞了（[`CrashLibrary::record_the_item`]）。
pub fn two_stages<Judged>(
    class: CrashLibraryClass,
    repository_root: &Path,
    item: &ItemName,
    record: impl FnOnce(&CrashLibrary) -> Result<ItemManifest, String>,
    judge: impl FnOnce(&CrashLibrary, &ItemManifest) -> Result<StageOutcome<Judged>, String>,
) -> Result<StageOutcome<Judged>, String> {
    let stage = RunStage::of_this_process();
    let library = CrashLibrary::open(class, repository_root)?;
    if stage.records() {
        let manifest = record(&library)?;
        library.record_the_item(item, &manifest)?;
        drop(manifest);
    }
    if !stage.judges() {
        return Ok(StageOutcome::RecordedOnly);
    }
    let manifest = library.manifest_of_the_item(item)?;
    judge(&library, &manifest)
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

// ============================== 对拍用例的两段 ==============================

/// 对拍用例的事实表清单里事实表那一样的名字。
const PAIRING_MANIFEST_FACTS: &str = "facts";

/// 第 ② 段核对的 GPU 与 CPU 参照对拍用例用：录入段把事实表按摘要写进对拍那一类的库（`crash_library::CrashLibraryClass::FactsPairing`，
/// 家目录下长期存放）、按项名（用例名）登清单；判段只凭项名读清单、按摘要读回，核对只拿读回的那一份。只录那一趟交回
/// [`StageOutcome::RecordedOnly`]，用例拿到它就收工。
///
/// # Panics
/// 项名不合规矩；库开不了、写不进、读不回。
pub fn facts_through_the_pairing_library(
    item_name: &str,
    facts: &FlowFacts,
) -> StageOutcome<FlowFactsReadBack> {
    let item =
        ItemName::new(item_name).unwrap_or_else(|refusal| panic!("对拍用例的项名：{refusal}"));
    two_stages(
        CrashLibraryClass::FactsPairing,
        &repository_root_of_this_crate(),
        &item,
        |library| {
            let digest = facts.digest();
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
/// GPU 判器与 CPU 判器对拍用例用：录入段把打包判表按摘要写进对拍那一类的库（`crash_library::CrashLibraryClass::JudgeTablesPairing`，
/// 家目录下长期存放）、按项名（用例名）登清单；判段只凭项名读清单、按摘要读回，GPU 判器只收读回的那一份。只录那一趟交回
/// [`StageOutcome::RecordedOnly`]，用例拿到它就收工。
///
/// # Panics
/// 项名不合规矩；库开不了、写不进、读不回。
pub fn judge_tables_through_the_pairing_library(
    item_name: &str,
    tables: &PackedJudgeTables,
) -> StageOutcome<PackedJudgeTablesReadBack> {
    let item =
        ItemName::new(item_name).unwrap_or_else(|refusal| panic!("对拍用例的项名：{refusal}"));
    two_stages(
        CrashLibraryClass::JudgeTablesPairing,
        &repository_root_of_this_crate(),
        &item,
        |library| {
            let bytes = tables.to_bytes();
            let digest = singlefs_harness::sha256::sha256_digest(&bytes);
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

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn a_lock_held_by_another_process_is_told_apart_from_other_errors() {
        assert!(is_a_lock_held_by_another_process(
            "RocksDatabase(Error { message: \"IO error: While lock file: /x/LOCK: Resource temporarily unavailable\" })"
        ));
        assert!(!is_a_lock_held_by_another_process(
            "IO error: No such file or directory"
        ));
    }

    #[test]
    fn each_class_has_its_own_directory_name() {
        let classes = [
            CrashLibraryClass::CrashAmplification,
            CrashLibraryClass::CrashStateEnumeration,
            CrashLibraryClass::GivenStates,
            CrashLibraryClass::JudgeTablesPairing,
            CrashLibraryClass::FactsPairing,
            CrashLibraryClass::AmplificationUnitTests,
        ];
        let mut names: Vec<&str> = classes.iter().map(|class| class.directory_name()).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), classes.len(), "几类库的子目录名互不相同");
    }
}
