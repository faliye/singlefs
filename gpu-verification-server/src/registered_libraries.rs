//! 登记的临时判定库：判段的库在临时 prune-server 里时，判段先在真服务里登记那份临时服务（一把续着租的认领，不写数据行），
//! 服务每一圈列出登记，照登记去连那份临时服务取待判行、把结论写回那一份（`crate::serving_from_the_library` 文件头）。
//!
//! 登记键 `gpu-claim/temporary-library/<这一台>/<进程号>-<启动刻>/<载荷的小写十六进制>`（[`TEMPORARY_LIBRARY_REGISTRATION_KEY_PREFIX`]）。载荷依次是四段，
//! 每段 `u16 长度（大端）` 加内容：socket 路径、TCP 地址（`<地址>:<端口>`）、键空间前缀（空的是全仓那一份的键原样，否则头一个字节是
//! [`DISPOSABLE_LIBRARY_KEY_SPACE_LEADING_BYTE`]）、给人看的标签。写法、解法与持有者身份、租期都归验证包
//! （[`TemporaryLibraryRegistration`]），服务只调它的解法 `from_claim_key`，不另写一份。
//!
//! 一份登记开一份 [`LibraryPendingPool`]，用它自己的路由与客户端；结论只写回那一份临时服务；各份库的活按入池序号先来先判
//! （[`LibrariesOfTheService::claim_the_candidates`]）。
//! - 连哪里：登记的台是这一台、socket 在本机的走 unix socket，否则走 TCP；TCP 地址是回环而登记的台不是这一台的不连。
//! - 不连：指到真服务（登记的 socket 解开符号链接之后等于服务自己连的那个 socket，或 TCP 地址等于服务自己连的那个）、键空间前缀不空而头一个字节不是
//!   [`DISPOSABLE_LIBRARY_KEY_SPACE_LEADING_BYTE`] 的登记；点名一次。
//! - 丢掉一份库：登记没了（租期过了、判段放了）、读写这一份库失败；丢掉时这一份库里在判的块判完也不写回，追加日志目录删掉，打一行
//!   [`TEMPORARY_LIBRARY_DROPPED_LINE_PREFIX`]。读写失败的登记下一次列到再试，至多每个扫库间隔试一次。真库读写失败照旧交回，让服务退出。
//! - 起来时放上一个进程留在这一份库里的认领：每份登记头一次打开时照 [`LibraryPendingPool::release_the_claims_left_by_the_previous_process`]。

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use prune_server::client::RouteAddress;
use prune_server::protocol::ListedClaim;
use prune_server::reliable_client::ReconnectPolicy;
use singlefs_verification::crash_amplification::CardThatJudgedAPendingBlock;
use singlefs_verification::crash_library::{
    TemporaryLibraryRegistration, TemporaryServiceOfTheLibrary,
    TEMPORARY_LIBRARY_REGISTRATION_KEY_PREFIX,
};
use singlefs_verification::hexadecimal::hexadecimal_text;
use singlefs_verification::verdict_store::DISPOSABLE_LIBRARY_KEY_SPACE_LEADING_BYTE;

use crate::break_switches::{BreakSwitch, BreakSwitches};
use crate::card_lease::{CardLeaseKeyAnswer, CardLeaseKeys};
use crate::facts_verifier_family::{FactsVerifierBlockName, FactsVerifierPendingRowOfTheLibrary};
use crate::library_pending_pool::{
    ClaimsReleased, FactsVerifierJobOfTheLibrary, LibraryFailure, LibraryPendingPool,
    PendingBlockName, PendingRowOfTheLibrary, TakenBlock, TakenPackedTables,
    VerificationPackageSide,
};
use crate::packed_tables_family::{PackedTablesName, PackedTablesPendingRowOfTheLibrary};
use crate::pending_pool::StateVerdict;
use crate::serving_from_the_library::SERVING_PROBLEM_LINE_PREFIX;

/// 打开了一份登记的库的那一行的开头。
pub const TEMPORARY_LIBRARY_OPENED_LINE_PREFIX: &str =
    "GPU_VERIFICATION_SERVER_TEMPORARY_LIBRARY_OPENED";
/// 一份登记没连（不合布局、指到真服务、键空间前缀不合、回环地址属于别的台、开不起来）的那一行的开头，每份登记只打一次。
pub const TEMPORARY_LIBRARY_NOT_OPENED_LINE_PREFIX: &str =
    "GPU_VERIFICATION_SERVER_TEMPORARY_LIBRARY_NOT_OPENED";
/// 丢掉一份登记的库（登记没了、读写失败）的那一行的开头。
pub const TEMPORARY_LIBRARY_DROPPED_LINE_PREFIX: &str =
    "GPU_VERIFICATION_SERVER_TEMPORARY_LIBRARY_DROPPED";
/// 一块在判的块所在的库已经丢掉、结论不写回的那一行的开头。
pub const TEMPORARY_LIBRARY_BLOCK_DISCARDED_LINE_PREFIX: &str =
    "GPU_VERIFICATION_SERVER_TEMPORARY_LIBRARY_BLOCK_DISCARDED";

/// 真服务答了、却不认列认领（登记的库一份都不开）那一行的开头，只打一次。
pub const TEMPORARY_LIBRARY_LISTING_REFUSED_LINE_PREFIX: &str =
    "GPU_VERIFICATION_SERVER_TEMPORARY_LIBRARY_LISTING_REFUSED";
/// 判定库客户端的退出码表里「服务不可用」那一个（连不上、忙到上限、服务停了）：列认领失败是别的码时是真服务不认这一帧。
const LIBRARY_UNAVAILABLE_EXIT: i32 = 75;
/// 点名过的事的键里，登记键与事的种类之间的分隔（种类是 ASCII 文字，登记键里没有这个字节）。
const NAMED_KEY_SEPARATOR: u8 = 0;

/// 连登记的库的客户端重连：登记的库没了（进程死了、服务停了）时几秒内报错，不像真库那样等两分钟；
/// 一份库连不上不能拖住取活循环里真库的活。
pub const REGISTERED_LIBRARY_RECONNECT: ReconnectPolicy = ReconnectPolicy {
    initial_delay: Duration::from_millis(50),
    maximum_delay: Duration::from_millis(500),
    maximum_attempts: 10,
    maximum_total: Duration::from_secs(5),
};

/// 取活循环里的一份库：真库，或第几份登记的库（编号按打开的先后给，不复用）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LibraryOfTheService {
    TheRealLibrary,
    Registered(u64),
}

impl fmt::Display for LibraryOfTheService {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TheRealLibrary => write!(formatter, "real"),
            Self::Registered(number) => write!(formatter, "registered-{number}"),
        }
    }
}

impl LibraryOfTheService {
    /// 日志行里点名库的那一截：真库不点名（行的写法与只有真库时相同），登记的库点名编号。
    fn note_in_a_line(self) -> String {
        match self {
            Self::TheRealLibrary => String::new(),
            Self::Registered(number) => format!("library=registered-{number} "),
        }
    }
}

/// 一份登记不连的原因。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegistrationRefusal {
    /// 登记键不合布局。
    Undecodable(String),
    /// socket 或 TCP 地址指到服务自己连的真服务。
    PointsToTheRealService,
    /// 键空间前缀不空而头一个字节不是一次性库名字空间的头一个字节。
    ForeignKeySpacePrefix,
    /// 登记的台不是这一台，TCP 地址又是回环：连过去连到的是这一台自己。
    LoopbackAddressOfAnotherHost,
    /// socket 不在这一台、TCP 地址是空的。
    NoRouteToTheLibrary,
}

impl fmt::Display for RegistrationRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Undecodable(reason) => write!(formatter, "undecodable reason={reason}"),
            Self::PointsToTheRealService => write!(formatter, "points-to-the-real-service"),
            Self::ForeignKeySpacePrefix => write!(formatter, "foreign-key-space-prefix"),
            Self::LoopbackAddressOfAnotherHost => {
                write!(formatter, "loopback-address-of-another-host")
            }
            Self::NoRouteToTheLibrary => write!(formatter, "no-route-to-the-library"),
        }
    }
}

/// 这一份登记连不连：不连的交回原因。弄坏开关 real-service-registration-opened 下指到真服务与键空间前缀两样都不核。
#[must_use]
pub fn refusal_of_a_registration(
    registration: &TemporaryLibraryRegistration,
    real_route: &RouteAddress,
    break_switches: &BreakSwitches,
) -> Option<RegistrationRefusal> {
    if break_switches.is_on(BreakSwitch::RealServiceRegistrationOpened) {
        return None;
    }
    if points_to_the_real_service(&registration.service, real_route) {
        return Some(RegistrationRefusal::PointsToTheRealService);
    }
    let has_a_foreign_prefix = registration
        .service
        .key_space_prefix
        .first()
        .is_some_and(|leading_byte| *leading_byte != DISPOSABLE_LIBRARY_KEY_SPACE_LEADING_BYTE);
    has_a_foreign_prefix.then_some(RegistrationRefusal::ForeignKeySpacePrefix)
}

fn points_to_the_real_service(
    service: &TemporaryServiceOfTheLibrary,
    real_route: &RouteAddress,
) -> bool {
    let matches_real_route = match real_route {
        RouteAddress::UnixSocket(real_socket) => service
            .socket_path
            .as_ref()
            .is_some_and(|socket_path| resolved_path(socket_path) == resolved_path(real_socket)),
        RouteAddress::Network(real_address) => service
            .network_address
            .as_ref()
            .is_some_and(|address| address == real_address),
    };
    if matches_real_route {
        return true;
    }
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")));
    if let Ok(Some(configuration)) =
        singlefs_verification::gpu_card_configuration::multi_host_configuration_file(repository)
    {
        if let Some(configured_socket) =
            singlefs_verification::gpu_card_configuration::configuration_value(
                &configuration,
                "PRUNE_SERVER_SOCKET_PATH",
            )
            .filter(|value| !value.trim().is_empty())
        {
            let configured_path = Path::new(configured_socket.trim());
            if service.socket_path.as_ref().is_some_and(|socket_path| {
                resolved_path(socket_path) == resolved_path(configured_path)
            }) {
                return true;
            }
        }
        if let Some(configured_tcp) =
            singlefs_verification::gpu_card_configuration::configuration_value(
                &configuration,
                "PRUNE_SERVER_TCP_ADDRESS",
            )
            .filter(|value| !value.trim().is_empty())
        {
            if service
                .network_address
                .as_ref()
                .is_some_and(|address| address == configured_tcp.trim())
            {
                return true;
            }
        }
    }
    false
}

/// 连哪一处：登记的台是这一台、socket 在这一台上的走 unix socket；否则走 TCP。
///
/// # Errors
/// 要走 TCP 而没有 TCP 地址；地址是回环而登记的台不是这一台。
pub fn route_of_a_registration(
    registration: &TemporaryLibraryRegistration,
    this_host: &str,
) -> Result<RouteAddress, RegistrationRefusal> {
    let is_registered_by_this_host = registration.host == this_host;
    if is_registered_by_this_host {
        if let Some(socket_path) = &registration.service.socket_path {
            if socket_path.exists() {
                return Ok(RouteAddress::UnixSocket(socket_path.clone()));
            }
        }
    }
    let Some(network_address) = &registration.service.network_address else {
        return Err(RegistrationRefusal::NoRouteToTheLibrary);
    };
    let is_loopback = network_address
        .parse::<SocketAddr>()
        .is_ok_and(|address| address.ip().is_loopback());
    if is_loopback && !is_registered_by_this_host {
        return Err(RegistrationRefusal::LoopbackAddressOfAnotherHost);
    }
    Ok(RouteAddress::Network(network_address.clone()))
}

fn resolved_path(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

/// 打开一份登记的库要的：登记（连同列认领回的登记键整把，两份登记键相同就是同一份）、编号、连那份库用的路由、认领与写结论的客户端用的路由。
/// 平常两个路由相同；弄坏开关 conclusion-written-into-the-real-library 下认领与写结论的客户端连真服务。
#[derive(Clone, Debug)]
pub struct OpeningOfARegisteredLibrary {
    pub registration: TemporaryLibraryRegistration,
    pub claim_key: Vec<u8>,
    pub library: LibraryOfTheService,
    pub route_of_the_library: RouteAddress,
    pub route_of_the_claims: RouteAddress,
}

/// 开好的一份登记的库：它的待判池与追加日志所在的目录（丢掉这份库时删）。
pub struct OpenedRegisteredLibrary<Side> {
    pub pool: LibraryPendingPool<Side>,
    pub journal_directory: Option<PathBuf>,
}

/// 取活循环对登记的库的设置。
pub struct RegisteredLibrariesSettings<'opener, Side> {
    /// 照登记开一份库（调用方连上那份库、开验证包那一侧与认领的客户端；开不成交回为什么）。
    pub open_a_library: &'opener mut dyn FnMut(
        &OpeningOfARegisteredLibrary,
    )
        -> Result<OpenedRegisteredLibrary<Side>, String>,
    /// 这一台在多机配置 `HOSTS` 里的地址。
    pub this_host: String,
    /// 服务自己连的真库的路由。
    pub real_route: RouteAddress,
    /// 状态目录里上一个进程写的「在跑」那一行（没有是 `None`）：每份登记头一次打开时照它放上一个进程留下的认领。
    pub previous_serving_line: Option<String>,
}

/// 一份打开着的登记的库。
struct RegisteredLibrary<Side> {
    registration: TemporaryLibraryRegistration,
    claim_key: Vec<u8>,
    pool: LibraryPendingPool<Side>,
    journal_directory: Option<PathBuf>,
}

impl<Side> Drop for RegisteredLibrary<Side> {
    fn drop(&mut self) {
        if let Some(directory) = &self.journal_directory {
            // 追加日志只对这一份库有用；删不掉（目录已经没了）不影响服务
            let _ = std::fs::remove_dir_all(directory);
        }
    }
}

/// 丢掉的库留下的账（并进服务交回的总账）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct CountsOfDroppedLibraries {
    reusable_rows_deleted: u64,
    verified_facts_rows_deleted: u64,
    packed_tables_rows_deleted: u64,
}

/// 这一圈扫哪几份库。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LibrariesScanned {
    TheRealLibraryAndTheRegisteredOnes,
    TheRegisteredOnesOnly,
}

/// 各份库这一圈扫到的候选（按库分开，库内的次序照那一份库自己扫的键序）。
pub struct CandidatesOfEveryLibrary {
    blocks: Vec<(LibraryOfTheService, Vec<PendingRowOfTheLibrary>)>,
    facts_verifier_blocks: Vec<(
        LibraryOfTheService,
        Vec<FactsVerifierPendingRowOfTheLibrary>,
    )>,
    packed_tables: Vec<(LibraryOfTheService, Vec<PackedTablesPendingRowOfTheLibrary>)>,
}

impl CandidatesOfEveryLibrary {
    /// 一份候选也没有（这一圈不扫库时用它）。
    #[must_use]
    pub fn none() -> Self {
        Self {
            blocks: Vec::new(),
            facts_verifier_blocks: Vec::new(),
            packed_tables: Vec::new(),
        }
    }

    /// 只留 `keep` 说留的待判行块（取活循环去掉这个进程不再认领的）。
    pub fn retain_blocks(
        &mut self,
        mut keep: impl FnMut(LibraryOfTheService, &PendingRowOfTheLibrary) -> bool,
    ) {
        for (library, candidates) in &mut self.blocks {
            candidates.retain(|pending| keep(*library, pending));
        }
    }

    /// 同 [`Self::retain_blocks`]，核对待判行。
    pub fn retain_facts_verifier_blocks(
        &mut self,
        mut keep: impl FnMut(LibraryOfTheService, &FactsVerifierPendingRowOfTheLibrary) -> bool,
    ) {
        for (library, candidates) in &mut self.facts_verifier_blocks {
            candidates.retain(|pending| keep(*library, pending));
        }
    }

    /// 同 [`Self::retain_blocks`]，判表待判行。
    pub fn retain_packed_tables(
        &mut self,
        mut keep: impl FnMut(LibraryOfTheService, &PackedTablesPendingRowOfTheLibrary) -> bool,
    ) {
        for (library, candidates) in &mut self.packed_tables {
            candidates.retain(|pending| keep(*library, pending));
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.blocks
            .iter()
            .all(|(_, candidates)| candidates.is_empty())
            && self
                .facts_verifier_blocks
                .iter()
                .all(|(_, candidates)| candidates.is_empty())
            && self
                .packed_tables
                .iter()
                .all(|(_, candidates)| candidates.is_empty())
    }
}

/// 一圈认领到的，三族各按入池序号排好（各份库合在一起）。
pub struct ClaimedInPoolEntryOrder<Materials> {
    pub blocks: Vec<(LibraryOfTheService, TakenBlock<Materials>)>,
    pub facts_verifier_jobs: Vec<(LibraryOfTheService, FactsVerifierJobOfTheLibrary)>,
    pub packed_tables: Vec<(LibraryOfTheService, TakenPackedTables<Materials>)>,
}

impl<Materials> ClaimedInPoolEntryOrder<Materials> {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
            && self.facts_verifier_jobs.is_empty()
            && self.packed_tables.is_empty()
    }
}

/// 写回一块的结局。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WriteBackOutcome {
    Written,
    RefusedAtTheWrite,
    /// 这一块所在的登记的库已经丢掉：不写回。
    LibraryGone,
}

/// 取活循环里的各份库：真库一份、登记的库若干份。
pub struct LibrariesOfTheService<'libraries, Side> {
    real: &'libraries mut LibraryPendingPool<Side>,
    registered: BTreeMap<u64, RegisteredLibrary<Side>>,
    settings: Option<RegisteredLibrariesSettings<'libraries, Side>>,
    break_switches: BreakSwitches,
    scan_interval: Duration,
    next_number: u64,
    /// 点名过的登记键：每份登记同一种问题只打一次。
    named: BTreeSet<Vec<u8>>,
    /// 读写失败而丢掉、等下一次列到再试的登记键与丢掉的时刻。
    failed_at: BTreeMap<Vec<u8>, Instant>,
    dropped_counts: CountsOfDroppedLibraries,
    libraries_opened: u64,
    libraries_dropped: u64,
    /// 真服务不认列认领那一行打过了。
    listing_refusal_named: bool,
}

impl<'libraries, Side: VerificationPackageSide> LibrariesOfTheService<'libraries, Side> {
    /// `settings` 是 `None` 的只有真库（不列登记）。
    #[must_use]
    pub fn new(
        real: &'libraries mut LibraryPendingPool<Side>,
        settings: Option<RegisteredLibrariesSettings<'libraries, Side>>,
        break_switches: &BreakSwitches,
        scan_interval: Duration,
    ) -> Self {
        Self {
            real,
            registered: BTreeMap::new(),
            settings,
            break_switches: break_switches.clone(),
            scan_interval,
            next_number: 0,
            named: BTreeSet::new(),
            failed_at: BTreeMap::new(),
            dropped_counts: CountsOfDroppedLibraries::default(),
            libraries_opened: 0,
            libraries_dropped: 0,
            listing_refusal_named: false,
        }
    }

    /// 打开过几份登记的库、丢掉过几份。
    #[must_use]
    pub fn registered_libraries_opened_and_dropped(&self) -> (u64, u64) {
        (self.libraries_opened, self.libraries_dropped)
    }

    /// 此刻打开着几份登记的库。
    #[must_use]
    pub fn registered_libraries_open(&self) -> usize {
        self.registered.len()
    }

    fn libraries_now(&self) -> Vec<LibraryOfTheService> {
        std::iter::once(LibraryOfTheService::TheRealLibrary)
            .chain(
                self.registered
                    .keys()
                    .map(|number| LibraryOfTheService::Registered(*number)),
            )
            .collect()
    }

    fn pool_of(&mut self, library: LibraryOfTheService) -> Option<&mut LibraryPendingPool<Side>> {
        match library {
            LibraryOfTheService::TheRealLibrary => Some(&mut *self.real),
            LibraryOfTheService::Registered(number) => self
                .registered
                .get_mut(&number)
                .map(|registered| &mut registered.pool),
        }
    }

    /// 这一份登记的这一类事只打一行（`kind` 区分几类）：一份登记反复连不上、反复丢掉也只打一次。
    fn name_once(&mut self, claim_key: &[u8], kind: &str, line: String) {
        let mut named_key = claim_key.to_vec();
        named_key.push(NAMED_KEY_SEPARATOR);
        named_key.extend_from_slice(kind.as_bytes());
        if self.named.insert(named_key) {
            self.real.print_a_service_line(line);
        }
    }

    /// 丢掉一份登记的库：在判的块以后不写回，追加日志目录随它删。`name_once_by` 给了登记键的，丢掉那一行对这份登记只打一次
    /// （读写失败的登记下一次列到会再开、再丢）。
    fn drop_a_registered_library(&mut self, number: u64, why: &str, name_once_by: Option<&[u8]>) {
        let Some(library) = self.registered.remove(&number) else {
            return;
        };
        self.libraries_dropped += 1;
        self.dropped_counts.reusable_rows_deleted += library.pool.reusable_rows_deleted();
        self.dropped_counts.verified_facts_rows_deleted +=
            library.pool.verified_facts_rows_deleted();
        self.dropped_counts.packed_tables_rows_deleted += library.pool.packed_tables_rows_deleted();
        let line = format!(
            "{TEMPORARY_LIBRARY_DROPPED_LINE_PREFIX} library=registered-{number} label={} why={why}",
            library.registration.label
        );
        match name_once_by {
            Some(claim_key) => self.name_once(claim_key, "dropped", line),
            None => self.real.print_a_service_line(line),
        }
    }

    /// 在一份登记的库上做一件事：读写这一份库失败只丢这一份、交回 `None`；
    /// 弄坏开关 temporary-library-failure-ends-the-service 下照真库那样交回失败。
    fn on_a_registered_library<Output>(
        &mut self,
        number: u64,
        operation: impl FnOnce(&mut LibraryPendingPool<Side>) -> Result<Output, LibraryFailure>,
    ) -> Result<Option<Output>, LibraryFailure> {
        let Some(library) = self.registered.get_mut(&number) else {
            return Ok(None);
        };
        let failure = match operation(&mut library.pool) {
            Ok(output) => return Ok(Some(output)),
            Err(failure) => failure,
        };
        if self
            .break_switches
            .is_on(BreakSwitch::TemporaryLibraryFailureEndsTheService)
        {
            return Err(failure);
        }
        let claim_key = library.claim_key.clone();
        let _ = library.pool.release_the_claims_of_this_process();
        self.failed_at.insert(claim_key.clone(), Instant::now());
        self.drop_a_registered_library(number, &format!("failed {failure}"), Some(&claim_key));
        Ok(None)
    }

    /// 对每一份库做一件事，各份库各带自己的输入；真库失败交回，登记的库失败只丢那一份（它的输出不收）。
    fn apply_to_each<Input, Output>(
        &mut self,
        inputs: Vec<(LibraryOfTheService, Input)>,
        mut operation: impl FnMut(
            &mut LibraryPendingPool<Side>,
            Input,
        ) -> Result<Output, LibraryFailure>,
    ) -> Result<Vec<(LibraryOfTheService, Output)>, LibraryFailure> {
        let mut outputs = Vec::with_capacity(inputs.len());
        for (library, input) in inputs {
            match library {
                LibraryOfTheService::TheRealLibrary => {
                    outputs.push((library, operation(&mut *self.real, input)?));
                }
                LibraryOfTheService::Registered(number) => {
                    let output =
                        self.on_a_registered_library(number, |pool| operation(pool, input))?;
                    if let Some(output) = output {
                        outputs.push((library, output));
                    }
                }
            }
        }
        Ok(outputs)
    }

    /// 写回之前先看已打开的登记有没有过期的：有就先丢掉（在判的块判完也不写回已过期的临时库）。
    pub fn drop_lapsed_registrations(&mut self) -> Result<(), LibraryFailure> {
        if self.settings.is_none()
            || self.registered.is_empty()
            || self
                .break_switches
                .is_on(BreakSwitch::RegistrationLapseIgnored)
        {
            return Ok(());
        }
        let listed: Vec<ListedClaim> = match self
            .real
            .list_the_claims_with_prefix(TEMPORARY_LIBRARY_REGISTRATION_KEY_PREFIX)
        {
            Ok(listed) => listed,
            Err(failure) if failure.exit_code != LIBRARY_UNAVAILABLE_EXIT => return Ok(()),
            Err(failure) => return Err(failure),
        };
        let lapsed: Vec<u64> = self
            .registered
            .iter()
            .filter(|(_, library)| !listed.iter().any(|claim| claim.key == library.claim_key))
            .map(|(number, _)| *number)
            .collect();
        for number in lapsed {
            self.drop_a_registered_library(number, "registration-lapsed", None);
        }
        Ok(())
    }

    /// 列出登记：没了的库丢掉，新出现的打开（头一次打开时放上一个进程留在那份库里的认领）。读真库失败交回。
    ///
    /// # Errors
    /// 读写真库失败（列认领）；弄坏开关 temporary-library-failure-ends-the-service 下登记的库失败也交回。
    pub fn follow_the_registrations(&mut self) -> Result<(), LibraryFailure> {
        if self.settings.is_none() {
            return Ok(());
        }
        let listed: Vec<ListedClaim> = match self
            .real
            .list_the_claims_with_prefix(TEMPORARY_LIBRARY_REGISTRATION_KEY_PREFIX)
        {
            Ok(listed) => listed,
            Err(failure) if failure.exit_code != LIBRARY_UNAVAILABLE_EXIT => {
                // 真服务答了、却不认列认领（还没换成带列认领的那一版）：登记的库一份都不开，点名一次；服务不退出
                if !self.listing_refusal_named {
                    self.listing_refusal_named = true;
                    self.real.print_a_service_line(format!(
                        "{TEMPORARY_LIBRARY_LISTING_REFUSED_LINE_PREFIX} {failure}"
                    ));
                }
                return Ok(());
            }
            Err(failure) => return Err(failure),
        };
        if !self
            .break_switches
            .is_on(BreakSwitch::RegistrationLapseIgnored)
        {
            let lapsed: Vec<u64> = self
                .registered
                .iter()
                .filter(|(_, library)| !listed.iter().any(|claim| claim.key == library.claim_key))
                .map(|(number, _)| *number)
                .collect();
            for number in lapsed {
                self.drop_a_registered_library(number, "registration-lapsed", None);
            }
        }
        self.failed_at
            .retain(|claim_key, _| listed.iter().any(|claim| claim.key == *claim_key));
        self.named
            .retain(|named_key| listed.iter().any(|claim| named_key.starts_with(&claim.key)));
        for claim in &listed {
            let is_open = self
                .registered
                .values()
                .any(|library| library.claim_key == claim.key);
            if is_open {
                continue;
            }
            let retry_is_due = match self.failed_at.get(&claim.key) {
                Some(failed_at) => failed_at.elapsed() >= self.scan_interval,
                None => true,
            };
            if retry_is_due {
                self.open_a_registration(&claim.key)?;
            }
        }
        Ok(())
    }

    /// 打开一份登记的库；不连的、开不起来的点名一次，不交回失败（登记的库的事不拖垮服务）。
    fn open_a_registration(&mut self, claim_key: &[u8]) -> Result<(), LibraryFailure> {
        let (this_host, real_route) = match &self.settings {
            Some(settings) => (settings.this_host.clone(), settings.real_route.clone()),
            None => return Ok(()),
        };
        let registration = match TemporaryLibraryRegistration::from_claim_key(claim_key) {
            Ok(registration) => registration,
            Err(reason) => {
                let refusal = RegistrationRefusal::Undecodable(reason);
                let line = format!(
                    "{TEMPORARY_LIBRARY_NOT_OPENED_LINE_PREFIX} key={} why={refusal}",
                    hexadecimal_text(claim_key)
                );
                self.name_once(claim_key, "undecodable", line);
                return Ok(());
            }
        };
        let route =
            match refusal_of_a_registration(&registration, &real_route, &self.break_switches) {
                Some(refusal) => Err(refusal),
                None => route_of_a_registration(&registration, &this_host),
            };
        let route_of_the_library = match route {
            Ok(route) => route,
            Err(refusal) => {
                let line = format!(
                    "{TEMPORARY_LIBRARY_NOT_OPENED_LINE_PREFIX} label={} why={refusal}",
                    registration.label
                );
                self.name_once(claim_key, "refused", line);
                return Ok(());
            }
        };
        let number = self.next_number;
        self.next_number += 1;
        let route_of_the_claims = if self
            .break_switches
            .is_on(BreakSwitch::ConclusionWrittenIntoTheRealLibrary)
        {
            real_route
        } else {
            route_of_the_library.clone()
        };
        let opening = OpeningOfARegisteredLibrary {
            registration: registration.clone(),
            claim_key: claim_key.to_vec(),
            library: LibraryOfTheService::Registered(number),
            route_of_the_library,
            route_of_the_claims,
        };
        let opened = match self.settings.as_mut() {
            Some(settings) => (settings.open_a_library)(&opening),
            None => return Ok(()),
        };
        let opened = match opened {
            Ok(opened) => opened,
            Err(reason) => {
                let line = format!(
                    "{TEMPORARY_LIBRARY_NOT_OPENED_LINE_PREFIX} label={} why=open-failed reason={reason}",
                    registration.label
                );
                self.failed_at.insert(claim_key.to_vec(), Instant::now());
                self.name_once(claim_key, "open-failed", line);
                return Ok(());
            }
        };
        let mut library = RegisteredLibrary {
            registration,
            claim_key: claim_key.to_vec(),
            pool: opened.pool,
            journal_directory: opened.journal_directory,
        };
        if !self
            .break_switches
            .is_on(BreakSwitch::PreviousClaimsKeptInTemporaryLibraries)
        {
            let previous_serving_line = self
                .settings
                .as_ref()
                .and_then(|settings| settings.previous_serving_line.clone());
            if let Err(failure) = library
                .pool
                .release_the_claims_left_by_the_previous_process(previous_serving_line.as_deref())
            {
                if self
                    .break_switches
                    .is_on(BreakSwitch::TemporaryLibraryFailureEndsTheService)
                {
                    return Err(failure);
                }
                let line = format!(
                    "{TEMPORARY_LIBRARY_NOT_OPENED_LINE_PREFIX} label={} why=release-failed reason={failure}",
                    library.registration.label
                );
                self.failed_at.insert(claim_key.to_vec(), Instant::now());
                self.name_once(claim_key, "release-failed", line);
                return Ok(());
            }
        }
        let line = format!(
            "{TEMPORARY_LIBRARY_OPENED_LINE_PREFIX} library=registered-{number} label={} route={}",
            library.registration.label,
            opening.route_of_the_library.label()
        );
        self.registered.insert(number, library);
        self.libraries_opened += 1;
        self.name_once(claim_key, "opened", line);
        Ok(())
    }

    /// 在判的块到点续认领：各份库各续各的，续不上的打一行（别的持有者持着）。
    ///
    /// # Errors
    /// 读写真库失败。
    pub fn renew_the_claims(&mut self) -> Result<(), LibraryFailure> {
        let inputs: Vec<(LibraryOfTheService, ())> = self
            .libraries_now()
            .into_iter()
            .map(|library| (library, ()))
            .collect();
        let lost = self.apply_to_each(inputs, |pool, ()| {
            Ok((
                pool.renew_the_claims()?,
                pool.renew_the_facts_verifier_claims()?,
                pool.renew_the_packed_tables_claims()?,
            ))
        })?;
        for (library, (blocks, facts_verifier_blocks, packed_tables)) in lost {
            let note = library.note_in_a_line();
            for name in blocks {
                eprintln!("{SERVING_PROBLEM_LINE_PREFIX} {note}{name} 续认领没续上（别的持有者持着）：照判，写回时那一帧核认领、不写");
            }
            for name in facts_verifier_blocks {
                eprintln!("{SERVING_PROBLEM_LINE_PREFIX} {note}核对块 {name} 续认领没续上（别的持有者持着）：照核，写回时那一帧核认领、不写");
            }
            for name in packed_tables {
                eprintln!("{SERVING_PROBLEM_LINE_PREFIX} {note}判表 {name} 续认领没续上（别的持有者持着）：照判，写回时那一帧核认领、不写");
            }
        }
        Ok(())
    }

    /// 各份库扫一遍，挑能取的块（三族）。
    ///
    /// # Errors
    /// 读写真库失败。
    pub fn scan_the_candidates(
        &mut self,
        scanned: LibrariesScanned,
    ) -> Result<CandidatesOfEveryLibrary, LibraryFailure> {
        let libraries: Vec<(LibraryOfTheService, ())> = self
            .libraries_now()
            .into_iter()
            .filter(|library| {
                *library != LibraryOfTheService::TheRealLibrary
                    || scanned == LibrariesScanned::TheRealLibraryAndTheRegisteredOnes
            })
            .map(|library| (library, ()))
            .collect();
        let scans = self.apply_to_each(libraries, |pool, ()| {
            Ok((
                pool.candidates()?,
                pool.facts_verifier_candidates()?,
                pool.packed_tables_candidates()?,
            ))
        })?;
        let mut candidates = CandidatesOfEveryLibrary::none();
        for (library, (blocks, facts_verifier_blocks, packed_tables)) in scans {
            candidates.blocks.push((library, blocks));
            candidates
                .facts_verifier_blocks
                .push((library, facts_verifier_blocks));
            candidates.packed_tables.push((library, packed_tables));
        }
        Ok(candidates)
    }

    /// 各份库认领自己的候选（没有候选的库不碰），认领到的三族各合起来按待判行的入池序号升序排：
    /// 先来先判，真库一直有活时登记的库的活照样排得到。序号相同的保持真库在前、库内保持扫库的键序。
    /// 弄坏开关 real-library-claimed-first：不合起来排，真库的先、登记的库的后。
    ///
    /// # Errors
    /// 认领、重读、放认领时读写真库失败。
    pub fn claim_the_candidates(
        &mut self,
        candidates: CandidatesOfEveryLibrary,
    ) -> Result<ClaimedInPoolEntryOrder<Side::Materials>, LibraryFailure> {
        let CandidatesOfEveryLibrary {
            blocks,
            facts_verifier_blocks,
            packed_tables,
        } = candidates;
        let blocks: Vec<(LibraryOfTheService, Vec<PendingRowOfTheLibrary>)> = blocks
            .into_iter()
            .filter(|(_, pending_rows)| !pending_rows.is_empty())
            .collect();
        let mut taken_blocks: Vec<(LibraryOfTheService, TakenBlock<Side::Materials>)> = self
            .apply_to_each(blocks, |pool, pending_rows| {
                pool.claim_the_candidates(pending_rows)
                    .map(|round| round.taken)
            })?
            .into_iter()
            .flat_map(|(library, taken)| taken.into_iter().map(move |block| (library, block)))
            .collect();
        let facts_verifier_blocks: Vec<(
            LibraryOfTheService,
            Vec<FactsVerifierPendingRowOfTheLibrary>,
        )> = facts_verifier_blocks
            .into_iter()
            .filter(|(_, pending_rows)| !pending_rows.is_empty())
            .collect();
        let mut facts_verifier_jobs: Vec<(LibraryOfTheService, FactsVerifierJobOfTheLibrary)> =
            self.apply_to_each(facts_verifier_blocks, |pool, pending_rows| {
                pool.claim_the_facts_verifier_candidates(pending_rows)
            })?
            .into_iter()
            .flat_map(|(library, jobs)| jobs.into_iter().map(move |job| (library, job)))
            .collect();
        let packed_tables: Vec<(LibraryOfTheService, Vec<PackedTablesPendingRowOfTheLibrary>)> =
            packed_tables
                .into_iter()
                .filter(|(_, pending_rows)| !pending_rows.is_empty())
                .collect();
        let mut taken_packed_tables: Vec<(
            LibraryOfTheService,
            TakenPackedTables<Side::Materials>,
        )> = self
            .apply_to_each(packed_tables, |pool, pending_rows| {
                pool.claim_the_packed_tables_candidates(pending_rows)
            })?
            .into_iter()
            .flat_map(|(library, taken)| taken.into_iter().map(move |tables| (library, tables)))
            .collect();
        if !self
            .break_switches
            .is_on(BreakSwitch::RealLibraryClaimedFirst)
        {
            taken_blocks.sort_by_key(|(_, taken)| taken.pending.row.pool_entry_number);
            facts_verifier_jobs.sort_by_key(|(_, job)| {
                job.blocks
                    .iter()
                    .map(|pending| pending.row.pool_entry_number)
                    .min()
                    .unwrap_or(0)
            });
            taken_packed_tables.sort_by_key(|(_, taken)| taken.pending.row.pool_entry_number);
        }
        Ok(ClaimedInPoolEntryOrder {
            blocks: taken_blocks,
            facts_verifier_jobs,
            packed_tables: taken_packed_tables,
        })
    }

    /// 主循环把这一块撤了：不再当在判。那一份库已经丢掉的不用撤。
    pub fn withdraw(&mut self, library: LibraryOfTheService, name: &PendingBlockName) {
        if let Some(pool) = self.pool_of(library) {
            pool.withdraw(name);
        }
    }

    /// 同 [`Self::withdraw`]，核对块。
    pub fn withdraw_the_facts_verifier_block(
        &mut self,
        library: LibraryOfTheService,
        name: &FactsVerifierBlockName,
    ) {
        if let Some(pool) = self.pool_of(library) {
            pool.withdraw_the_facts_verifier_block(name);
        }
    }

    /// 同 [`Self::withdraw`]，判表。
    pub fn withdraw_the_packed_tables(
        &mut self,
        library: LibraryOfTheService,
        name: &PackedTablesName,
    ) {
        if let Some(pool) = self.pool_of(library) {
            pool.withdraw_the_packed_tables(name);
        }
    }

    /// 这一台判不了这一块（`crate::serving_from_the_library` 文件头「判不了」）：在它所在的那一份库上不再当在判，
    /// 当场以自己的持有者放掉它的认领（[`LibraryPendingPool::release_the_claim_left_to_another_service`]）。
    /// 那一份库已经丢掉的交 `None`；放认领时读写失败照原样交回，登记的库也不因此丢掉（认领留到租期过了）。
    pub fn release_the_claim_left_to_another_service(
        &mut self,
        library: LibraryOfTheService,
        name: &PendingBlockName,
    ) -> Option<Result<ClaimsReleased, LibraryFailure>> {
        self.pool_of(library)
            .map(|pool| pool.release_the_claim_left_to_another_service(name))
    }

    /// 同 [`Self::release_the_claim_left_to_another_service`]，核对块。
    pub fn release_the_facts_verifier_claim_left_to_another_service(
        &mut self,
        library: LibraryOfTheService,
        name: &FactsVerifierBlockName,
    ) -> Option<Result<ClaimsReleased, LibraryFailure>> {
        self.pool_of(library)
            .map(|pool| pool.release_the_facts_verifier_claim_left_to_another_service(name))
    }

    /// 同 [`Self::release_the_claim_left_to_another_service`]，判表。
    pub fn release_the_packed_tables_claim_left_to_another_service(
        &mut self,
        library: LibraryOfTheService,
        name: &PackedTablesName,
    ) -> Option<Result<ClaimsReleased, LibraryFailure>> {
        self.pool_of(library)
            .map(|pool| pool.release_the_packed_tables_claim_left_to_another_service(name))
    }

    /// 判完的块写回哪一份库：入池时记的那一份。弄坏开关 judged-block-matched-to-its-library-by-name 下照块名找，
    /// 取头一份认领着这个名字的库（几份库里有同名的块就串）。
    fn library_of_a_judged_block(
        &self,
        library: LibraryOfTheService,
        name: &PendingBlockName,
    ) -> LibraryOfTheService {
        if !self
            .break_switches
            .is_on(BreakSwitch::JudgedBlockMatchedToItsLibraryByName)
        {
            return library;
        }
        self.libraries_now()
            .into_iter()
            .find(|candidate| match candidate {
                LibraryOfTheService::TheRealLibrary => self.real.is_claimed(name),
                LibraryOfTheService::Registered(number) => self
                    .registered
                    .get(number)
                    .is_some_and(|registered| registered.pool.is_claimed(name)),
            })
            .unwrap_or(library)
    }

    /// 一块判完写回它所在的那一份库；那一份库已经丢掉的不写。
    pub fn write_back(
        &mut self,
        library: LibraryOfTheService,
        name: &PendingBlockName,
        verdicts: Vec<StateVerdict>,
        card: &CardThatJudgedAPendingBlock,
    ) -> WriteBackOutcome {
        let library = self.library_of_a_judged_block(library, name);
        match self.pool_of(library) {
            Some(pool) => match pool.write_back(name, verdicts, card) {
                Ok(_) => WriteBackOutcome::Written,
                Err(_) => WriteBackOutcome::RefusedAtTheWrite,
            },
            None => WriteBackOutcome::LibraryGone,
        }
    }

    /// 同 [`Self::write_back`]，核对块。
    pub fn write_back_the_facts_verifier_block(
        &mut self,
        library: LibraryOfTheService,
        name: &FactsVerifierBlockName,
        bits_of_each_state: Vec<u32>,
    ) -> WriteBackOutcome {
        match self.pool_of(library) {
            Some(pool) => {
                match pool.write_back_the_facts_verifier_block(name, bits_of_each_state) {
                    Ok(_) => WriteBackOutcome::Written,
                    Err(_) => WriteBackOutcome::RefusedAtTheWrite,
                }
            }
            None => WriteBackOutcome::LibraryGone,
        }
    }

    /// 同 [`Self::write_back`]，判表。
    pub fn write_back_the_packed_tables(
        &mut self,
        library: LibraryOfTheService,
        name: &PackedTablesName,
        verdicts: Vec<StateVerdict>,
        card: &CardThatJudgedAPendingBlock,
    ) -> WriteBackOutcome {
        match self.pool_of(library) {
            Some(pool) => match pool.write_back_the_packed_tables(name, verdicts, card) {
                Ok(_) => WriteBackOutcome::Written,
                Err(_) => WriteBackOutcome::RefusedAtTheWrite,
            },
            None => WriteBackOutcome::LibraryGone,
        }
    }

    /// 打一行服务日志：经真库那一份待判池打（测试开了收行的，另收在它那里）。
    pub fn print_a_service_line(&mut self, line: String) {
        self.real.print_a_service_line(line);
    }

    /// 这一台的借卡键（只有真库那一份待判池持着，没给是 `None`）。
    #[must_use]
    pub fn card_lease_keys(&self) -> Option<&CardLeaseKeys> {
        self.real.card_lease_keys()
    }

    /// 以真库那一份待判池的持有者认领（或续上）一把借卡的键。
    ///
    /// # Errors
    /// 读写真库失败。
    pub fn claim_a_card_lease_key(
        &mut self,
        key: Vec<u8>,
        lease_milliseconds: u32,
    ) -> Result<CardLeaseKeyAnswer, LibraryFailure> {
        self.real.claim_a_card_lease_key(key, lease_milliseconds)
    }

    /// 以真库那一份待判池的持有者放掉卡键（借出去）。
    ///
    /// # Errors
    /// 读写真库失败。
    pub fn release_the_card_key(&mut self) -> Result<ClaimsReleased, LibraryFailure> {
        self.real.release_the_card_key()
    }

    /// 各份库里认领着、在判的块数合计。
    #[must_use]
    pub fn claimed_count(&self) -> usize {
        self.real.claimed_count()
            + self
                .registered
                .values()
                .map(|library| library.pool.claimed_count())
                .sum::<usize>()
    }

    /// 各份库里认领着、在核的核对块数合计。
    #[must_use]
    pub fn facts_claimed_count(&self) -> usize {
        self.real.facts_claimed_count()
            + self
                .registered
                .values()
                .map(|library| library.pool.facts_claimed_count())
                .sum::<usize>()
    }

    /// 各份库里认领着、在判的判表数合计。
    #[must_use]
    pub fn packed_tables_claimed_count(&self) -> usize {
        self.real.packed_tables_claimed_count()
            + self
                .registered
                .values()
                .map(|library| library.pool.packed_tables_claimed_count())
                .sum::<usize>()
    }

    /// 判完而可复用、待判行带守卫删掉了的块数（各份库合计，含丢掉了的）。
    #[must_use]
    pub fn reusable_rows_deleted(&self) -> u64 {
        self.real.reusable_rows_deleted()
            + self.dropped_counts.reusable_rows_deleted
            + self
                .registered
                .values()
                .map(|library| library.pool.reusable_rows_deleted())
                .sum::<u64>()
    }

    /// 已核过、核对待判行带守卫删掉了的块数（各份库合计，含丢掉了的）。
    #[must_use]
    pub fn verified_facts_rows_deleted(&self) -> u64 {
        self.real.verified_facts_rows_deleted()
            + self.dropped_counts.verified_facts_rows_deleted
            + self
                .registered
                .values()
                .map(|library| library.pool.verified_facts_rows_deleted())
                .sum::<u64>()
    }

    /// 结论字已在表 `21`、判表待判行带守卫删掉了的张数（各份库合计，含丢掉了的）。
    #[must_use]
    pub fn packed_tables_rows_deleted(&self) -> u64 {
        self.real.packed_tables_rows_deleted()
            + self.dropped_counts.packed_tables_rows_deleted
            + self
                .registered
                .values()
                .map(|library| library.pool.packed_tables_rows_deleted())
                .sum::<u64>()
    }

    /// 服务停的时候：登记的库各以自己的持有者放掉还持着的认领（放不了的点名、不交回，那份库的认领到租期过了自己失效），再丢掉。
    pub fn release_the_registered_libraries(&mut self) {
        for number in self.registered.keys().copied().collect::<Vec<u64>>() {
            if let Some(library) = self.registered.get_mut(&number) {
                if let Err(failure) = library.pool.release_the_claims_of_this_process() {
                    let line = format!(
                        "{TEMPORARY_LIBRARY_NOT_OPENED_LINE_PREFIX} label={} why=release-at-the-stop-failed reason={failure}",
                        library.registration.label
                    );
                    self.real.print_a_service_line(line);
                }
            }
            self.drop_a_registered_library(number, "service-stopping", None);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOST: &str = "192.0.2.10";
    const REGISTRANT: &str = "4242-99";

    fn registration_of(
        host: &str,
        socket_path: Option<&str>,
        network_address: Option<&str>,
        key_space_prefix: &[u8],
    ) -> TemporaryLibraryRegistration {
        TemporaryLibraryRegistration {
            host: host.to_string(),
            registrant: REGISTRANT.to_string(),
            service: TemporaryServiceOfTheLibrary {
                socket_path: socket_path.map(PathBuf::from),
                network_address: network_address.map(str::to_string),
                key_space_prefix: key_space_prefix.to_vec(),
            },
            label: "label with spaces#0".to_string(),
        }
    }

    /// 解一份验证包写出来的登记：服务读的就是验证包的写法。
    fn decoded(registration: &TemporaryLibraryRegistration) -> TemporaryLibraryRegistration {
        TemporaryLibraryRegistration::from_claim_key(
            &registration.claim_key().expect("登记键编得出"),
        )
        .expect("写出来的键解得开")
    }

    #[test]
    fn a_registration_key_decodes_to_what_was_encoded() {
        let prefix = [DISPOSABLE_LIBRARY_KEY_SPACE_LEADING_BYTE, b'x', 1, 2];
        let registration = registration_of(
            HOST,
            Some("/run/temporary/socket"),
            Some("192.0.2.10:7000"),
            &prefix,
        );
        // 逐字节写死的键：四段各带 u16 大端长度、载荷是小写十六进制；验证包的写法变了这条红
        let written_by_the_verification_package = b"gpu-claim/temporary-library/192.0.2.10/4242-99/\
00152f72756e2f74656d706f726172792f736f636b6574000f3139322e302e322e31303a373030300004e078010200136c6162656c2077697468207370616365732330";
        assert_eq!(
            registration.claim_key().expect("登记键编得出"),
            written_by_the_verification_package.to_vec()
        );
        assert_eq!(
            TemporaryLibraryRegistration::from_claim_key(written_by_the_verification_package),
            Ok(registration)
        );
    }

    #[test]
    fn a_registration_key_with_a_damaged_payload_is_refused() {
        let key = registration_of(HOST, Some("/run/temporary/socket"), None, &[])
            .claim_key()
            .expect("登记键编得出");
        let truncated = &key[..key.len() - 2];
        assert!(
            TemporaryLibraryRegistration::from_claim_key(truncated).is_err(),
            "载荷少一个字节"
        );
        let mut with_a_rest = key.clone();
        with_a_rest.extend_from_slice(b"00");
        assert!(
            TemporaryLibraryRegistration::from_claim_key(&with_a_rest).is_err(),
            "四段之后还有剩余"
        );
        let mut not_hexadecimal = key;
        *not_hexadecimal.last_mut().expect("键不空") = b'z';
        assert!(
            TemporaryLibraryRegistration::from_claim_key(&not_hexadecimal).is_err(),
            "载荷里有非十六进制字符"
        );
        assert!(
            TemporaryLibraryRegistration::from_claim_key(b"gpu-claim/other/x").is_err(),
            "前缀不对"
        );
    }

    #[test]
    fn a_registration_pointing_at_the_real_service_or_carrying_a_foreign_prefix_is_refused() {
        let none = BreakSwitches::none();
        let real_socket = RouteAddress::UnixSocket(PathBuf::from("/run/real/socket"));
        let real_network = RouteAddress::Network("192.0.2.10:7000".to_string());
        let registration = |socket: Option<&str>, network: Option<&str>, prefix: &[u8]| {
            decoded(&registration_of(HOST, socket, network, prefix))
        };
        assert_eq!(
            refusal_of_a_registration(
                &registration(Some("/run/real/socket"), None, &[]),
                &real_socket,
                &none
            ),
            Some(RegistrationRefusal::PointsToTheRealService)
        );
        assert_eq!(
            refusal_of_a_registration(
                &registration(None, Some("192.0.2.10:7000"), &[]),
                &real_network,
                &none
            ),
            Some(RegistrationRefusal::PointsToTheRealService)
        );
        assert_eq!(
            refusal_of_a_registration(
                &registration(Some("/run/temporary/socket"), None, &[0x01, b'x']),
                &real_socket,
                &none
            ),
            Some(RegistrationRefusal::ForeignKeySpacePrefix)
        );
        assert_eq!(
            refusal_of_a_registration(
                &registration(
                    Some("/run/temporary/socket"),
                    None,
                    &[DISPOSABLE_LIBRARY_KEY_SPACE_LEADING_BYTE, b'x']
                ),
                &real_socket,
                &none
            ),
            None,
            "指到别处、前缀头一个字节对的登记连"
        );
        assert_eq!(
            refusal_of_a_registration(
                &registration(Some("/run/temporary/socket"), None, &[]),
                &real_socket,
                &none
            ),
            None,
            "前缀是空的（临时服务里全仓那一份）连"
        );
    }

    #[test]
    fn the_route_follows_the_host_of_the_registration() {
        let registration = |host: &str, socket: &str, network: &str| {
            decoded(&registration_of(
                host,
                (!socket.is_empty()).then_some(socket),
                (!network.is_empty()).then_some(network),
                &[],
            ))
        };
        let existing_socket = std::env::temp_dir();
        let existing = existing_socket.to_string_lossy().into_owned();
        assert_eq!(
            route_of_a_registration(&registration(HOST, &existing, "192.0.2.10:7000"), HOST),
            Ok(RouteAddress::UnixSocket(existing_socket)),
            "这一台登记的、socket 在本机：走 socket"
        );
        assert_eq!(
            route_of_a_registration(
                &registration("192.0.2.11", &existing, "192.0.2.11:7000"),
                HOST
            ),
            Ok(RouteAddress::Network("192.0.2.11:7000".to_string())),
            "别的台登记的：走 TCP"
        );
        assert_eq!(
            route_of_a_registration(
                &registration("192.0.2.11", &existing, "127.0.0.1:7000"),
                HOST
            ),
            Err(RegistrationRefusal::LoopbackAddressOfAnotherHost),
            "别的台登记的回环地址连的是这一台自己：不连"
        );
        assert_eq!(
            route_of_a_registration(&registration(HOST, "/no/such/socket", ""), HOST),
            Err(RegistrationRefusal::NoRouteToTheLibrary)
        );
    }
}
