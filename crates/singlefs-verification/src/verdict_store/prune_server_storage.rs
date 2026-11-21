//! prune-server 那一侧：键空间、服务读写失败怎么报，与经容错客户端点读、写、认领、前缀扫的那一端。

use super::*;

// ============================== prune-server 那一侧 ==============================
//
// 崩溃判定库只走 prune-server（`.claude/rules/verification.md`「判定库只有一份：全仓统一写进 prune-server 的那份 RocksDB」；
// 「现在就对接 不允许残留」「我们不兼容 类似老开关 这种东西」）：本机不开任何一份 RocksDB，不设开关，各进程（崩溃放量、
// 崩溃枚举用例、给定状态表、对拍、变异复跑）直接读写服务那一份，写下的块别的进程、另一台马上读得到。键与判定语义只这一套。
// 判定向量编号由服务统一给：登记新的一种在服务里认领下一个编号再写（两张判定向量表同一帧），几个进程、两台各登记同一种只得一个编号。
// 成批（不逐键往返）：点读一帧问一批键（[`PRUNE_SERVER_READ_KEYS_PER_FRAME`] 个一帧；回应越过帧长上限时服务拒这一帧，容错客户端把键拆成两半再问），
// 写按 [`PRUNE_SERVER_WRITE_FRAME_BYTES`] 切帧，前缀扫被条数或帧长上限截断就把前缀往下拆一个字节分着扫（[`PruneServerStorage::visit_rows_with_prefix`]），
// 整表扫逐行交给调用方、不整段攥在内存里。
// 一行的值长过一帧装的上限（录下的流程、判表这类单独一行就上百 MB 的）照 Python 客户端的格式分段存（`prune_server::segmented_values`，
// 格式说明只在 `scripts/prune-server/prune_server_client.py` 文件头「键空间」大值分段那一段）：写经容错客户端先写段、再写清单，点读拼回整值、核 sha256，
// 前缀扫把扫回来的原样喂给拼接器（段键与计数键不当行）；拼不回照别的错（退 3）报，不当成「库里没有」。

/// 崩溃判定库的行全经服务写进它三张表里的结论那一张（`prune_server::protocol::Table::Conclusions`；那份 RocksDB 只由服务自己开，这份文件不开任何一份 RocksDB，
/// 见上面「prune-server 那一侧」那一段），键原样
/// （表标签开头）；别家（Python 三家）的键以 ASCII 族名起头，与表标签字节不撞。
const PRUNE_SERVER_TABLE: Table = Table::Conclusions;

/// 一次性库的名字空间的头一个字节（[`LibraryKeySpace::DisposableLibrary`]）：不是崩溃判定库的表标签（`0x01`–`0x21`），不是 Python 三家族名的
/// ASCII 字节，也小于服务留给自己记账的那一段（`prune_server::protocol::SERVICE_KEY_LEADING_BYTE_MINIMUM`，`0xF0`）。
pub const DISPOSABLE_LIBRARY_KEY_SPACE_LEADING_BYTE: u8 = 0xE0;

/// 一份崩溃判定库在服务里的键空间。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LibraryKeySpace {
    /// 全仓那一份：键原样，与服务原地打开的那份库同一批行。产品只开这一种。
    TheOneLibrary,
    /// 一次性库（只给对临时服务跑的测试，`crate::crash_library::DisposableLibrary` 建）：键前面接这一段名字空间（头一个字节是
    /// [`DISPOSABLE_LIBRARY_KEY_SPACE_LEADING_BYTE`]，后面是标签、进程号、时刻与序号），一份一次性库一段，同一份临时服务里几份互不相见、各从空库起。
    DisposableLibrary(Vec<u8>),
}

impl LibraryKeySpace {
    /// 接在每一把键前面的那一段：全仓那一份是空的（GPU 批处理服务照登记的前缀造出这一份键空间、编删键时也读它）。
    #[must_use]
    pub fn key_prefix(&self) -> &[u8] {
        match self {
            Self::TheOneLibrary => &[],
            Self::DisposableLibrary(namespace) => namespace,
        }
    }
}

/// 一帧写最多装多少字节的键值（服务的帧长上限 64 MiB，留出帧头与每条的表、长度字段）。一块一种结论的那一批远小于它（满块约 3.4 MB），
/// 落在一帧里，原子；更大的一批切成几帧，帧与帧之间不原子（调用方的批都是彼此独立的行，或一块一批）。
const PRUNE_SERVER_WRITE_FRAME_BYTES: usize = 32 << 20;

/// 一帧点读最多问几个键：判段一次读一块的几张旁表、登记时问一个编号，远少于它；成批读几千把键的（取开库之后别人登记的判定向量）按它切帧。
/// 回应的字节数事先不知道，越过帧长上限时服务整帧拒（`MalformedPayload`），容错客户端把那一帧的键拆成两半再问（[`ReliableClient::read`]）。
const PRUNE_SERVER_READ_KEYS_PER_FRAME: usize = 4096;

/// 前缀扫一帧最多回几条：回来的被条数或帧长上限截断，就把前缀往下拆一个字节分着扫（[`PruneServerStorage::visit_rows_with_prefix`]），
/// 不当成扫全了。
const PRUNE_SERVER_SCAN_ENTRY_LIMIT: u32 = 1 << 20;

/// 登记判定向量时认领编号的租期（毫秒）：认到马上写，写帧确认时认领随之释放；登记的进程死在中间，租期一过别人照样认这个号，编号不留空。
const PRUNE_SERVER_NUMBER_CLAIM_LEASE_MILLISECONDS: u32 = 30_000;

/// 编号被别的客户端认领着时隔多久再问一次。
pub(super) const PRUNE_SERVER_NUMBER_CLAIM_RETRY_INTERVAL: std::time::Duration =
    std::time::Duration::from_millis(20);

/// 编号被别的客户端认领着时最多等多久：持有者活着几毫秒就写完，死了一个租期之后放出来；两个租期还认不到当服务不可用。
pub(super) const PRUNE_SERVER_NUMBER_CLAIM_WAIT_LIMIT: std::time::Duration =
    std::time::Duration::from_secs(60);

/// 产品的跑法读写 prune-server 不成、照退出码退出进程之前在标准错误打的那一行：`<前缀> exit_code=<码> <说明>`。
pub const PRUNE_SERVER_FAILURE_LINE_PREFIX: &str = "CRASH_LIBRARY_PRUNE_SERVER_FAILURE";

/// prune-server 那一侧读写不成（[`VerdictStoreError::PruneServer`]）：照 prune-server 客户端退出码表的退出码（75 服务不可用：
/// 连不上、忙到上限、服务停了；3 别的错：被拒、协议错、分段存的值拼不回）与说明。只带这两样、不带服务客户端的类型。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PruneServerFailure {
    pub exit_code: i32,
    pub description: String,
}

/// prune-server 读写不成时怎么办。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PruneServerFailureHandling {
    /// 产品的跑法（[`crate::crash_library`] 开全仓那一份）：打一行 [`PRUNE_SERVER_FAILURE_LINE_PREFIX`]、照退出码退出进程，
    /// 任务很快结束，服务修好重起之后由提交它的会话重交；读失败不当成「库里没有」去现判。
    ExitTheProcess,
    /// 测试的跑法：交回 [`VerdictStoreError::PruneServer`]。
    ReturnTheError,
}

/// 点读分段存的键时交什么（[`PruneServerStorage::read_values`] 与 [`PruneServerStorage::read_stored_values`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StoredValueReading {
    /// 拼回整值、核 sha256。
    AssembleSegmentedValues,
    /// 清单、段照服务里存着的原样交。
    StoredBytesAsTheyAre,
}

/// 照 `handling` 处置一次读写不成：退出进程，或交回错误。
fn prune_server_failed(
    handling: PruneServerFailureHandling,
    exit_code: i32,
    description: String,
) -> VerdictStoreError {
    match handling {
        PruneServerFailureHandling::ExitTheProcess => {
            eprintln!("{PRUNE_SERVER_FAILURE_LINE_PREFIX} exit_code={exit_code} {description}");
            std::process::exit(exit_code)
        }
        PruneServerFailureHandling::ReturnTheError => {
            VerdictStoreError::PruneServer(PruneServerFailure {
                exit_code,
                description,
            })
        }
    }
}

/// 容错客户端报的错照它自己的退出码表处置。
fn prune_server_client_failed(
    handling: PruneServerFailureHandling,
    operation: &str,
    failure: &ClientFailure,
) -> VerdictStoreError {
    prune_server_failed(
        handling,
        failure.exit_code(),
        format!("{operation}：{failure}"),
    )
}

/// 连着 prune-server 的一份崩溃判定库：点读、写、认领、前缀扫都走同一个容错客户端（写带客户端身份与请求号，重放幂等；读与扫失败交错误、
/// 不当成不在；服务回忙照它给的毫秒数退避重发，忙到上限才报）。
pub struct PruneServerStorage {
    pub(super) route: RouteAddress,
    failure_handling: PruneServerFailureHandling,
    pub(super) key_space: LibraryKeySpace,
    reliable_client: std::sync::Mutex<ReliableClient>,
    /// 从开这一端起手里拿着容错客户端的纳秒数（读、写、认领、扫各自那一次往返）：判定库锁里有多少花在往返上，
    /// 取锁时记下它、放锁时再读一次，差就是这一次持锁里的往返（`crate::crash_library` 的持锁时长行）。
    nanoseconds_with_the_client: std::sync::atomic::AtomicU64,
    /// 前缀扫一帧最多回几条：平常是 [`PRUNE_SERVER_SCAN_ENTRY_LIMIT`]，只有单测改小它（[`Self::scan_at_most_entries_per_frame_for_a_test`]），
    /// 在几行的库上也走「截断就拆前缀」那一支。
    scan_entry_limit: std::sync::atomic::AtomicU32,
    /// 容错客户端追加日志所在的一次性目录（[`Self::connect_with_a_journal_of_its_own`] 建的；随这一端放手删掉）；调用方自己给追加日志路径的是 `None`。
    _journal_directory: Option<crate::crash_library::ExclusiveTemporaryDirectory>,
}

/// 手里拿着的容错客户端：照 [`ReliableClient`] 用，丢掉时把拿了多久记进 [`PruneServerStorage::nanoseconds_with_the_client_so_far`]。
struct ReliableClientInUse<'client> {
    client: std::sync::MutexGuard<'client, ReliableClient>,
    nanoseconds_with_the_client: &'client std::sync::atomic::AtomicU64,
    taken_at: std::time::Instant,
}

impl std::ops::Deref for ReliableClientInUse<'_> {
    type Target = ReliableClient;

    fn deref(&self) -> &ReliableClient {
        &self.client
    }
}

impl std::ops::DerefMut for ReliableClientInUse<'_> {
    fn deref_mut(&mut self) -> &mut ReliableClient {
        &mut self.client
    }
}

impl Drop for ReliableClientInUse<'_> {
    fn drop(&mut self) {
        self.nanoseconds_with_the_client.fetch_add(
            u64::try_from(self.taken_at.elapsed().as_nanos())
                .expect("拿着容错客户端的纳秒数装得进 u64：五百多年"),
            std::sync::atomic::Ordering::Relaxed,
        );
    }
}

impl PruneServerStorage {
    /// 连上服务（`journal_path` 是容错客户端没确认的写帧的追加日志）。
    ///
    /// # Errors
    /// 连不上（重连到上限）、追加日志读写不成：照 `failure_handling` 退出进程或交回 [`VerdictStoreError::PruneServer`]。
    pub fn connect(
        route: RouteAddress,
        journal_path: PathBuf,
        failure_handling: PruneServerFailureHandling,
        key_space: LibraryKeySpace,
    ) -> Result<Arc<Self>, VerdictStoreError> {
        let settings = ReliableClientSettings::new(route.clone(), journal_path);
        let reliable_client = ReliableClient::open(settings).map_err(|failure| {
            prune_server_client_failed(failure_handling, "连 prune-server", &failure)
        })?;
        Ok(Arc::new(Self {
            route,
            failure_handling,
            key_space,
            reliable_client: std::sync::Mutex::new(reliable_client),
            nanoseconds_with_the_client: std::sync::atomic::AtomicU64::new(0),
            scan_entry_limit: std::sync::atomic::AtomicU32::new(PRUNE_SERVER_SCAN_ENTRY_LIMIT),
            _journal_directory: None,
        }))
    }

    /// 连上服务，追加日志放进一份这一端自己的一次性目录（随这一端放手删掉：追加日志只管断线重连时重放没确认的帧，不必跨进程留着）。
    ///
    /// # Errors
    /// 建不了一次性目录；连不上（照 `failure_handling` 退出进程或交回）。
    pub fn connect_with_a_journal_of_its_own(
        route: RouteAddress,
        failure_handling: PruneServerFailureHandling,
        key_space: LibraryKeySpace,
    ) -> Result<Arc<Self>, String> {
        let journal_directory = crate::crash_library::ExclusiveTemporaryDirectory::create(
            "prune-server-client-journal",
        )?;
        let settings = ReliableClientSettings::new(
            route.clone(),
            journal_directory.path().join(DEFAULT_JOURNAL_FILE_NAME),
        );
        let reliable_client = ReliableClient::open(settings).map_err(|failure| {
            format!(
                "{:?}",
                prune_server_client_failed(failure_handling, "连 prune-server", &failure)
            )
        })?;
        Ok(Arc::new(Self {
            route,
            failure_handling,
            key_space,
            reliable_client: std::sync::Mutex::new(reliable_client),
            nanoseconds_with_the_client: std::sync::atomic::AtomicU64::new(0),
            scan_entry_limit: std::sync::atomic::AtomicU32::new(PRUNE_SERVER_SCAN_ENTRY_LIMIT),
            _journal_directory: Some(journal_directory),
        }))
    }

    /// 连的是哪一处、哪一个键空间（只进报错与行里报的名字）。
    #[must_use]
    pub fn library_name(&self) -> String {
        let route = match &self.route {
            RouteAddress::UnixSocket(socket_path) => format!("unix:{}", socket_path.display()),
            RouteAddress::Network(address) => format!("tcp:{address}"),
        };
        match &self.key_space {
            LibraryKeySpace::TheOneLibrary => format!("prune-server {route}"),
            LibraryKeySpace::DisposableLibrary(namespace) => format!(
                "prune-server {route} disposable-library={}",
                String::from_utf8_lossy(namespace.get(1..).unwrap_or_default())
            ),
        }
    }

    /// 照这一份的处置办法报一次读写不成。
    pub(super) fn failed(&self, exit_code: i32, description: String) -> VerdictStoreError {
        prune_server_failed(self.failure_handling, exit_code, description)
    }

    /// 库里的键接上这一份的键空间（[`LibraryKeySpace::key_prefix`]）。
    pub(super) fn namespaced_key(&self, key_bytes: &[u8]) -> Vec<u8> {
        key_bytes_in_the_key_space(&self.key_space, key_bytes)
    }

    /// [`Self::namespaced_key`] 的逆：服务里的整把键去掉这一份的键空间那一段（单测替 GPU 批处理服务写帧时经这一份写，写的时候再接上）。
    ///
    /// # Panics
    /// 那把键不以这一份的键空间起头（帧不是这一份编的）。
    #[cfg(test)]
    pub(super) fn key_without_the_key_space(&self, key_in_the_service: &[u8]) -> Vec<u8> {
        key_in_the_service
            .strip_prefix(self.key_space.key_prefix())
            .expect("帧里的键是编它的那一份库接上键空间之后的整把键")
            .to_vec()
    }

    fn lock_the_reliable_client(&self) -> ReliableClientInUse<'_> {
        let client = self
            .reliable_client
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        ReliableClientInUse {
            client,
            nanoseconds_with_the_client: &self.nanoseconds_with_the_client,
            taken_at: std::time::Instant::now(),
        }
    }

    /// 从开这一端起手里拿着容错客户端的纳秒数（[`Self::lock_the_reliable_client`] 那几次往返加起来）。
    #[must_use]
    pub fn nanoseconds_with_the_client_so_far(&self) -> u64 {
        self.nanoseconds_with_the_client
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// 成批点读，交回的次序与 `keys` 相同；分段存的拼回整值（文件头「prune-server 那一侧」）；删除记号（空值）读成不在，判定向量两张表里的空值
    /// 原样读成在（[`stored_row_is_present`]）。每 [`PRUNE_SERVER_READ_KEYS_PER_FRAME`] 个键一帧。
    ///
    /// # Errors
    /// 读不成（连不上、忙到上限、服务停了、被拒；一个键的值就越过帧长上限；分段存的拼不回），或回的个数与问的不同。
    pub(super) fn read_values(
        &self,
        keys: &[&[u8]],
    ) -> Result<Vec<Option<Vec<u8>>>, VerdictStoreError> {
        self.read_values_in_frames(keys, StoredValueReading::AssembleSegmentedValues)
    }

    /// 同 [`Self::read_values`]，只是分段存的不拼：清单、段照服务里存着的原样交（前缀扫拆前缀时点读正好等于前缀的那一把键用，交给拼接器）。
    ///
    /// # Errors
    /// 同 [`Self::read_values`]（拼不回那一条除外）。
    fn read_stored_values(
        &self,
        keys: &[&[u8]],
    ) -> Result<Vec<Option<Vec<u8>>>, VerdictStoreError> {
        self.read_values_in_frames(keys, StoredValueReading::StoredBytesAsTheyAre)
    }

    fn read_values_in_frames(
        &self,
        keys: &[&[u8]],
        reading: StoredValueReading,
    ) -> Result<Vec<Option<Vec<u8>>>, VerdictStoreError> {
        let mut values = Vec::with_capacity(keys.len());
        for keys_of_a_frame in keys.chunks(PRUNE_SERVER_READ_KEYS_PER_FRAME) {
            values.extend(self.read_values_of_a_frame(keys_of_a_frame, reading)?);
        }
        Ok(values)
    }

    /// 一帧问 `keys`（回应越过帧长上限时容错客户端自己把键拆成两半再问，[`ReliableClient::read`]）；拆到一个键还被拒照别的拒处置。
    fn read_values_of_a_frame(
        &self,
        keys: &[&[u8]],
        reading: StoredValueReading,
    ) -> Result<Vec<Option<Vec<u8>>>, VerdictStoreError> {
        let namespaced: Vec<(Table, Vec<u8>)> = keys
            .iter()
            .map(|key_bytes| (PRUNE_SERVER_TABLE, self.namespaced_key(key_bytes)))
            .collect();
        let read = match reading {
            StoredValueReading::AssembleSegmentedValues => self
                .lock_the_reliable_client()
                .read_assembling_segmented_values(&namespaced),
            StoredValueReading::StoredBytesAsTheyAre => {
                self.lock_the_reliable_client().read(&namespaced)
            }
        };
        let values = read.map_err(|failure| {
            prune_server_client_failed(self.failure_handling, "成批点读", &failure)
        })?;
        if values.len() != keys.len() {
            return Err(self.failed(
                OTHER_CLIENT_FAILURE_EXIT_CODE,
                format!(
                    "成批点读问了 {} 个键，服务回了 {} 个值",
                    keys.len(),
                    values.len()
                ),
            ));
        }
        Ok(keys
            .iter()
            .zip(values)
            .map(|(key_bytes, value)| {
                value.filter(|value_bytes| stored_row_is_present(key_bytes, value_bytes))
            })
            .collect())
    }

    /// 键以 `prefix` 开头的全部行（去掉键空间，删除记号不算，判定向量两张表里的空值算），按键升序，攒成一份交回：只给小前缀用（一块的违例、一个桶的见证、
    /// 一块的几把版本）；整表扫用 [`Self::visit_rows_with_prefix`] 逐行交。
    ///
    /// # Errors
    /// 同 [`Self::visit_rows_with_prefix`]。
    pub(super) fn rows_with_prefix(
        &self,
        prefix: &[u8],
    ) -> Result<Vec<StoredEntry>, VerdictStoreError> {
        let mut rows = Vec::new();
        self.visit_rows_with_prefix(prefix, &mut |row| {
            rows.push(row);
            Ok(())
        })?;
        Ok(rows)
    }

    /// 键以 `prefix` 开头的全部行按键升序逐行交给 `visit_row`（去掉键空间，删除记号不算，判定向量两张表里的空值算：[`stored_row_is_present`]）：
    /// 服务里存着的原样逐行（[`Self::visit_stored_rows_with_prefix`]）喂给拼接器（[`SegmentedScanAssembler`]），段键与计数键不交，分段存的那一行
    /// 点读补上没扫到的段、拼回整值再交（拼回的是空值的照删除记号不交）。`visit_row` 交回错误就到此为止。
    ///
    /// # Errors
    /// 同 [`Self::visit_stored_rows_with_prefix`]；分段存的拼不回。
    pub(super) fn visit_rows_with_prefix(
        &self,
        prefix: &[u8],
        visit_row: &mut dyn FnMut(StoredEntry) -> Result<(), VerdictStoreError>,
    ) -> Result<(), VerdictStoreError> {
        let mut assembler = SegmentedScanAssembler::new();
        self.visit_stored_rows_with_prefix(prefix, &mut |(key_bytes, stored_bytes)| {
            let scanned_entries = assembler
                .accept_stored_entry(key_bytes.into_vec(), stored_bytes.into_vec())
                .map_err(|failure| {
                    self.failed(
                        OTHER_CLIENT_FAILURE_EXIT_CODE,
                        format!("前缀扫：大值分段：{failure}"),
                    )
                })?;
            for scanned_entry in scanned_entries {
                self.visit_scanned_entry(scanned_entry, visit_row)?;
            }
            Ok(())
        })?;
        if let Some(being_assembled) = assembler.finish() {
            self.visit_scanned_entry(ScannedEntry::Segmented(being_assembled), visit_row)?;
        }
        Ok(())
    }

    /// 拼接器交出来的一条交给 `visit_row`：分段存的先点读补段、拼回（键接上这一份的键空间去读段），拼回的整值照 [`stored_row_is_present`] 判在不在。
    fn visit_scanned_entry(
        &self,
        scanned_entry: ScannedEntry,
        visit_row: &mut dyn FnMut(StoredEntry) -> Result<(), VerdictStoreError>,
    ) -> Result<(), VerdictStoreError> {
        match scanned_entry {
            ScannedEntry::Whole { key, value } => {
                visit_row((key.into_boxed_slice(), value.into_boxed_slice()))
            }
            ScannedEntry::Segmented(being_assembled) => {
                let key_in_the_service = self.namespaced_key(being_assembled.owner_key());
                let assembled = self.lock_the_reliable_client().assemble_segmented_value(
                    PRUNE_SERVER_TABLE,
                    &key_in_the_service,
                    being_assembled,
                );
                let (key_bytes, value_bytes) = assembled.map_err(|failure| {
                    prune_server_client_failed(
                        self.failure_handling,
                        "前缀扫拼回分段存的值",
                        &failure,
                    )
                })?;
                if !stored_row_is_present(&key_bytes, &value_bytes) {
                    return Ok(());
                }
                visit_row((key_bytes.into_boxed_slice(), value_bytes.into_boxed_slice()))
            }
        }
    }

    /// 键以 `prefix` 开头的全部行照服务里存着的原样按键升序逐行交给 `visit_row`（去掉键空间，删除记号不算，判定向量两张表里的空值算：
    /// [`stored_row_is_present`]；分段存的清单、段、计数键照原样交，由 [`Self::visit_rows_with_prefix`] 拼）：先扫一帧，没被截断就照交；被条数或帧长上限截断，
    /// 丢掉这一帧，先点读正好等于 `prefix` 的那一把键（它排在一切更长的键前面），再把前缀往下拆一个字节（256 个子前缀，按字节升序）逐个照这样扫。
    /// 每往下一层前缀长一个字节，扫到键尽了就不再截断，所以拆得完；被截断而一行都没回的（一行的值就装不下一帧）报错。`visit_row` 交回错误就到此为止。
    ///
    /// # Errors
    /// 连不上、扫不成（照服务不可用报）；一行就装不下一帧；回来的键不带这一份的键空间；`visit_row` 的错。
    fn visit_stored_rows_with_prefix(
        &self,
        prefix: &[u8],
        visit_row: &mut dyn FnMut(StoredEntry) -> Result<(), VerdictStoreError>,
    ) -> Result<(), VerdictStoreError> {
        let (is_truncated, entries) = self.scan_one_frame(
            prefix,
            self.scan_entry_limit
                .load(std::sync::atomic::Ordering::SeqCst),
        )?;
        if !is_truncated {
            let key_prefix = self.key_space.key_prefix();
            for entry in entries {
                let Some(key_bytes) = entry.key.strip_prefix(key_prefix) else {
                    return Err(self.failed(
                        OTHER_CLIENT_FAILURE_EXIT_CODE,
                        "前缀扫回来的键不带这一份崩溃判定库的键空间".to_string(),
                    ));
                };
                if !stored_row_is_present(key_bytes, &entry.value) {
                    continue;
                }
                visit_row((
                    key_bytes.to_vec().into_boxed_slice(),
                    entry.value.into_boxed_slice(),
                ))?;
            }
            return Ok(());
        }
        if entries.is_empty() {
            return Err(self.failed(
                OTHER_CLIENT_FAILURE_EXIT_CODE,
                format!("前缀 {prefix:02x?} 下扫回来一行都没有却说被截断：头一行的值就装不下一帧"),
            ));
        }
        drop(entries);
        if let Some(value_bytes) = self.read_stored_values(&[prefix])?.pop().flatten() {
            visit_row((
                prefix.to_vec().into_boxed_slice(),
                value_bytes.into_boxed_slice(),
            ))?;
        }
        let mut longer_prefix = Vec::with_capacity(prefix.len() + 1);
        longer_prefix.extend_from_slice(prefix);
        longer_prefix.push(0);
        // 迭代上界：256 个子前缀；每个往下递归一层前缀长一个字节，键尽了就不再截断
        for next_byte in u8::MIN..=u8::MAX {
            *longer_prefix
                .last_mut()
                .expect("上面刚往前缀后面接了一个字节") = next_byte;
            self.visit_stored_rows_with_prefix(&longer_prefix, visit_row)?;
        }
        Ok(())
    }

    /// 键以 `prefix` 开头的行有没有一行（删除记号不算，判定向量两张表里的空值算）：扫一帧、最多回一行，不把前缀下的行读回来；回来的那一行值为空的，
    /// 或是分段存放的一部分（清单拼回来可能是删除记号，段键与计数键不算行：[`is_part_of_segmented_storage`]），照 [`Self::visit_rows_with_prefix`]
    /// 把这个前缀扫全再判（空值算不算一行由它照表定）。
    ///
    /// # Errors
    /// 同 [`Self::visit_rows_with_prefix`]。
    pub(super) fn has_a_row_with_prefix(&self, prefix: &[u8]) -> Result<bool, VerdictStoreError> {
        let (_is_truncated, entries) = self.scan_one_frame(prefix, 1)?;
        match entries.first() {
            None => Ok(false),
            Some(entry)
                if !entry.value.is_empty()
                    && !is_part_of_segmented_storage(&entry.key, &entry.value) =>
            {
                Ok(true)
            }
            Some(_deletion_marker_or_segmented_storage) => {
                Ok(!self.rows_with_prefix(prefix)?.is_empty())
            }
        }
    }

    /// 按前缀扫一帧（键空间接在前缀前面），最多回 `maximum_entries` 行，交回截没截断与回来的行。
    ///
    /// # Errors
    /// 扫不成（连不上、忙到上限、服务停了、被拒：照 [`ReliableClient::scan`] 与客户端的退出码表报）。
    fn scan_one_frame(
        &self,
        prefix: &[u8],
        maximum_entries: u32,
    ) -> Result<(bool, Vec<prune_server::protocol::ScanEntry>), VerdictStoreError> {
        let namespaced_prefix = self.namespaced_key(prefix);
        let scanned = self.lock_the_reliable_client().scan(
            PRUNE_SERVER_TABLE,
            &namespaced_prefix,
            maximum_entries,
        );
        scanned.map_err(|failure| {
            prune_server_client_failed(self.failure_handling, "前缀扫", &failure)
        })
    }

    /// 写一批（[`LibraryWriteBatch`]，删换成删除记号）：按 [`PRUNE_SERVER_WRITE_FRAME_BYTES`] 切帧，每帧带客户端身份与请求号（服务按它们去重，
    /// 重放幂等），等到确认（服务落盘同步之后）才写下一帧。
    ///
    /// # Errors
    /// 写不成（连不上、忙到上限、服务停了、被拒、追加日志出错）；这一批在判定向量按编号那张表之外有值为空的写（别的表里空值在服务里是删除记号，
    /// 第一版不支持写空值的行；[`an_empty_value_may_be_put_under_the_key`]），或这一批删判定向量两张表里的行（那两张表只追加、从不删，删除记号写下去读成
    /// 一行空值、不是删；[`deletion_marker_reads_as_absent_under_the_key`]），一行都没写。两种都照别的错（[`OTHER_CLIENT_FAILURE_EXIT_CODE`]）报。
    pub(super) fn write_batch(&self, batch: LibraryWriteBatch) -> Result<(), VerdictStoreError> {
        let has_a_deletion_where_the_deletion_marker_reads_as_a_row =
            batch.rows.iter().any(|row| match row {
                LibraryWriteOfARow::Put { key: _, value: _ } => false,
                LibraryWriteOfARow::Delete { key } => {
                    !deletion_marker_reads_as_absent_under_the_key(key)
                }
            });
        if has_a_deletion_where_the_deletion_marker_reads_as_a_row {
            return Err(self.failed(
                OTHER_CLIENT_FAILURE_EXIT_CODE,
                "这一批要删判定向量两张表（按内容、按编号）里的行：这两张表只追加、从不删，删除记号写在那里读成一行空值（按编号那张表里就是绿的判定向量），不是删；\
                 一行都没写。出路：登记过的判定向量与编号一直留着，不再用的也不删；要去掉的是引它的块与违例行，删那几张表的行"
                    .to_string(),
            ));
        }
        let has_a_put_of_an_empty_value_where_it_would_be_a_deletion_marker =
            batch.rows.iter().any(|row| match row {
                LibraryWriteOfARow::Put { key, value } => {
                    value.is_empty() && !an_empty_value_may_be_put_under_the_key(key)
                }
                LibraryWriteOfARow::Delete { key: _ } => false,
            });
        if has_a_put_of_an_empty_value_where_it_would_be_a_deletion_marker {
            return Err(self.failed(
                OTHER_CLIENT_FAILURE_EXIT_CODE,
                "这一批在判定向量按编号那张表之外有值为空的行：那里的空值在 prune-server 里是删除记号，第一版不支持写空值的行"
                    .to_string(),
            ));
        }
        self.write_rows(
            batch
                .rows
                .into_iter()
                .map(|row| match row {
                    LibraryWriteOfARow::Put { key, value } => (key, value),
                    LibraryWriteOfARow::Delete { key } => (key, Vec::new()),
                })
                .collect(),
        )
    }

    /// 成批写键值（空值是删除记号）：长过一段的值先分段写下（[`ReliableClient::write_segments_of_large_values`]：各段与计数键写完、确认），
    /// 再把它交回的清单与别的行按 [`PRUNE_SERVER_WRITE_FRAME_BYTES`] 切帧，一帧一帧等确认。
    ///
    /// # Errors
    /// 写不成（连不上、忙到上限、服务停了、被拒、追加日志出错）；一把键的形状是段键或计数键、计数键读不回（大值分段那一步，一条都没写）。
    pub(super) fn write_rows(
        &self,
        rows: Vec<(Vec<u8>, Vec<u8>)>,
    ) -> Result<(), VerdictStoreError> {
        let mut client = self.lock_the_reliable_client();
        let entries = rows
            .into_iter()
            .map(|(key_bytes, value_bytes)| {
                (
                    PRUNE_SERVER_TABLE,
                    self.namespaced_key(&key_bytes),
                    value_bytes,
                )
            })
            .collect();
        let entries_to_write_last = client
            .write_segments_of_large_values(entries, PRUNE_SERVER_WRITE_FRAME_BYTES)
            .map_err(|failure| {
                prune_server_client_failed(self.failure_handling, "成批写的大值分段", &failure)
            })?;
        let mut frame: Vec<(Table, Vec<u8>, Vec<u8>)> = Vec::new();
        let mut frame_bytes = 0usize;
        for (table, key_in_the_service, value_bytes) in entries_to_write_last {
            let entry_bytes = key_in_the_service.len() + value_bytes.len();
            if !frame.is_empty() && frame_bytes + entry_bytes > PRUNE_SERVER_WRITE_FRAME_BYTES {
                self.write_one_frame(&mut client, &std::mem::take(&mut frame))?;
                frame_bytes = 0;
            }
            frame.push((table, key_in_the_service, value_bytes));
            frame_bytes += entry_bytes;
        }
        if !frame.is_empty() {
            self.write_one_frame(&mut client, &frame)?;
        }
        Ok(())
    }

    /// 写一帧、等它确认。
    fn write_one_frame(
        &self,
        client: &mut ReliableClient,
        entries: &[(Table, Vec<u8>, Vec<u8>)],
    ) -> Result<(), VerdictStoreError> {
        client.submit_write_entries(entries).map_err(|failure| {
            prune_server_client_failed(self.failure_handling, "成批写", &failure)
        })?;
        client.wait_all_acknowledged().map_err(|failure| {
            prune_server_client_failed(self.failure_handling, "等成批写的确认", &failure)
        })
    }

    /// 认领一个键（租期 [`PRUNE_SERVER_NUMBER_CLAIM_LEASE_MILLISECONDS`]），交回服务答的结局；`operation` 只进报错（认领的是什么）。
    ///
    /// # Errors
    /// 认领不成，或回的结局不是一个。
    pub(super) fn claim(
        &self,
        key_bytes: &[u8],
        operation: &str,
    ) -> Result<ClaimOutcome, VerdictStoreError> {
        let claimed = self.lock_the_reliable_client().claim(
            &[(PRUNE_SERVER_TABLE, self.namespaced_key(key_bytes))],
            PRUNE_SERVER_NUMBER_CLAIM_LEASE_MILLISECONDS,
        );
        let outcomes = claimed.map_err(|failure| {
            prune_server_client_failed(self.failure_handling, operation, &failure)
        })?;
        let mut outcomes = outcomes.into_iter();
        match (outcomes.next(), outcomes.next()) {
            (Some(outcome), None) => Ok(outcome),
            (None, _) | (Some(_), Some(_)) => Err(self.failed(
                OTHER_CLIENT_FAILURE_EXIT_CODE,
                "认领一个键，服务回的结局不是一个".to_string(),
            )),
        }
    }

    /// 测试开关：前缀扫一帧最多回 `entries_per_frame` 条（平常是 [`PRUNE_SERVER_SCAN_ENTRY_LIMIT`]）。
    #[cfg(test)]
    pub(crate) fn scan_at_most_entries_per_frame_for_a_test(&self, entries_per_frame: u32) {
        self.scan_entry_limit
            .store(entries_per_frame, std::sync::atomic::Ordering::SeqCst);
    }

    /// 测试改库里的行（造坏行、删行）用：一批原样写进服务（空值是删除记号），不经 [`VerdictStore`] 的照判不入库与记账。
    ///
    /// # Errors
    /// 同 [`Self::write_rows`]。
    #[cfg(test)]
    pub(crate) fn write_rows_for_a_test(
        &self,
        rows: Vec<(Vec<u8>, Vec<u8>)>,
    ) -> Result<(), VerdictStoreError> {
        self.write_rows(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prune_server::protocol::RejectionStatus;

    /// 写一批时值为空的行只有判定向量按编号那张表许写：别的表（这里是录下的流程那一张）里有一行值为空，整批拒在 prune-server 的错上
    /// （退出码 3，别的错），同一批里值不空的那一行也没写。
    /// 判别力（`crates/mutations.tsv`）：值为空的行在哪张表都放行（别的表里写下删除记号），这条红。
    #[test]
    fn a_batch_putting_an_empty_value_outside_the_verdict_vector_content_by_number_table_is_refused_and_writes_nothing(
    ) {
        let library = TestLibrary::new("empty-value-outside-the-vector-table");
        let service = library.service();
        let written_key =
            VerdictStoreKey::RecordedFlow(&[0x51; SHA256_DIGEST_BYTES]).to_key_bytes();
        let empty_key = VerdictStoreKey::RecordedFlow(&[0x52; SHA256_DIGEST_BYTES]).to_key_bytes();
        let mut batch = LibraryWriteBatch::default();
        batch.put(&written_key.0, b"recorded flow bytes");
        batch.put(&empty_key.0, b"");
        match service.write_batch(batch) {
            Err(VerdictStoreError::PruneServer(failure)) => assert_eq!(
                failure.exit_code, OTHER_CLIENT_FAILURE_EXIT_CODE,
                "别的表里值为空的行照别的错拒：{}",
                failure.description
            ),
            outcome => panic!("录下的流程那张表里值为空的行要整批拒：{outcome:?}"),
        }
        assert_eq!(
            library.raw_value(&written_key.0),
            None,
            "拒了就一行都没写，值不空的那一行也不在"
        );
    }

    /// 写一批时删判定向量按内容那张表的一行整批拒在 prune-server 的错上（退出码 3，别的错）：那张表只追加，删除记号写下去读回是一行空值
    /// （按内容认编号时读到空字节），不是删。拒了一行都没写：同一批里录下的流程那一行不在，按内容那一行原样留着登记时的编号。
    /// 判别力（`crates/mutations.tsv`）：写一批时删哪张表的行都放行，这条红。
    #[test]
    fn batch_deleting_a_row_of_the_verdict_vector_number_by_content_table_is_refused_and_writes_nothing(
    ) {
        let library = TestLibrary::new("delete-in-the-number-by-content-table");
        let registered = verdict_vector(b"I-2.1 seeded red");
        library
            .open()
            .register_verdict_vector(&registered)
            .expect("登记得了判定向量");
        let number_by_content_key =
            VerdictStoreKey::VerdictVectorNumberByContent(&registered).to_key_bytes();
        let number_row_before_the_batch = library.raw_value(&number_by_content_key.0);
        assert!(
            number_row_before_the_batch
                .as_ref()
                .is_some_and(|number_bytes| !number_bytes.is_empty()),
            "登记之后按内容那张表里有这一种的编号：{number_row_before_the_batch:?}"
        );
        let written_key =
            VerdictStoreKey::RecordedFlow(&[0x53; SHA256_DIGEST_BYTES]).to_key_bytes();
        let mut batch = LibraryWriteBatch::default();
        batch.put(&written_key.0, b"recorded flow bytes");
        batch.delete(&number_by_content_key.0);
        match library.service().write_batch(batch) {
            Err(VerdictStoreError::PruneServer(failure)) => assert_eq!(
                failure.exit_code, OTHER_CLIENT_FAILURE_EXIT_CODE,
                "删按内容那张表的行照别的错拒：{}",
                failure.description
            ),
            outcome => panic!("删按内容那张表的行要整批拒：{outcome:?}"),
        }
        assert_eq!(
            library.raw_value(&written_key.0),
            None,
            "拒了就一行都没写，同一批里录下的流程那一行也不在"
        );
        assert_eq!(
            library.raw_value(&number_by_content_key.0),
            number_row_before_the_batch,
            "按内容那一行原样留着登记时的编号，没写成空值"
        );
    }

    /// 写一批时删判定向量按编号那张表的一行整批拒在 prune-server 的错上（退出码 3，别的错）：那张表只追加，删除记号写下去读回是一行空值，
    /// 也就是绿的判定向量（红的那一号被读成绿）。拒了一行都没写：同一批里录下的流程那一行不在，按编号那一行原样留着红的判定向量。
    /// 判别力（`crates/mutations.tsv`）：删除记号写在哪把键下都当读成不在（删判定向量两张表的行放行），这条红。
    #[test]
    fn batch_deleting_a_row_of_the_verdict_vector_content_by_number_table_is_refused_and_writes_nothing(
    ) {
        let library = TestLibrary::new("delete-in-the-content-by-number-table");
        let registered = verdict_vector(b"I-2.2 seeded red");
        let number = library
            .open()
            .register_verdict_vector(&registered)
            .expect("登记得了判定向量");
        let content_by_number_key =
            VerdictStoreKey::VerdictVectorContentByNumber(number).to_key_bytes();
        assert_eq!(
            library.raw_value(&content_by_number_key.0),
            Some(registered.0.clone()),
            "登记之后按编号那张表里这一号是登记的那一种"
        );
        let written_key =
            VerdictStoreKey::RecordedFlow(&[0x54; SHA256_DIGEST_BYTES]).to_key_bytes();
        let mut batch = LibraryWriteBatch::default();
        batch.put(&written_key.0, b"recorded flow bytes");
        batch.delete(&content_by_number_key.0);
        match library.service().write_batch(batch) {
            Err(VerdictStoreError::PruneServer(failure)) => assert_eq!(
                failure.exit_code, OTHER_CLIENT_FAILURE_EXIT_CODE,
                "删按编号那张表的行照别的错拒：{}",
                failure.description
            ),
            outcome => panic!("删按编号那张表的行要整批拒：{outcome:?}"),
        }
        assert_eq!(
            library.raw_value(&written_key.0),
            None,
            "拒了就一行都没写，同一批里录下的流程那一行也不在"
        );
        assert_eq!(
            library.raw_value(&content_by_number_key.0),
            Some(registered.0),
            "按编号那一行原样留着红的判定向量，没写成空值（读成绿）"
        );
    }

    /// 连临时服务里一段名字空间的一端，名字空间是 `namespace`（头一个字节不小于 0xF0 的是服务留给自己记账的那一段，服务拒读拒写）。
    /// 一并交回那份一次性库：临时服务是它自己起的，丢掉它服务就停，调用方用完那一端之前一直拿着。
    fn service_end_over_the_namespace(
        label: &str,
        namespace: Vec<u8>,
    ) -> (
        crate::crash_library::DisposableLibrary,
        Arc<PruneServerStorage>,
    ) {
        // 一次性库留着：自证服务的名字空间读写（连同服务留给自己记账的那一段），要一份空的临时服务
        let disposable = crate::crash_library::DisposableLibrary::new(label)
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        let service = PruneServerStorage::connect_with_a_journal_of_its_own(
            disposable.route().clone(),
            PruneServerFailureHandling::ReturnTheError,
            LibraryKeySpace::DisposableLibrary(namespace),
        )
        .unwrap_or_else(|refusal| panic!("连不上临时 prune-server：{refusal}"));
        (disposable, service)
    }

    /// 服务拒了的读（键落在服务记账那一段，服务整帧拒）交回 prune-server 的错（退出码 3，别的错），不读成「库里没有」。
    /// 判别力（`crates/mutations.tsv`）：成批点读读不成时交回一排「不在」（读失败当「库里没有」，判段照它现判），这条红。
    #[test]
    fn a_read_the_service_refuses_is_an_error_and_never_an_absent_row() {
        let (_temporary_service, service) =
            service_end_over_the_namespace("read-refused", vec![0xF0, 0x01]);
        match service.read_values(&[b"any-key".as_slice()]) {
            Err(VerdictStoreError::PruneServer(failure)) => assert_eq!(
                failure.exit_code, OTHER_CLIENT_FAILURE_EXIT_CODE,
                "服务拒了的读照别的错的退出码：{}",
                failure.description
            ),
            Err(other) => panic!("服务拒了的读应当交回 prune-server 的错，交回的是 {other:?}"),
            Ok(values) => panic!("服务拒了的读读成了 {values:?}：读失败不许当成库里没有"),
        }
    }

    /// 服务拒了的写（键落在服务记账那一段）交回 prune-server 的错，不当写下了。
    /// 判别力（`crates/mutations.tsv`）：写一帧不等确认、不看服务回的结局就交回成功（写没确认就当写了），这条红。
    #[test]
    fn a_write_the_service_refuses_is_an_error_and_never_taken_as_written() {
        let (_temporary_service, service) =
            service_end_over_the_namespace("write-refused", vec![0xF0, 0x02]);
        match service.write_rows(vec![(b"any-key".to_vec(), b"any-value".to_vec())]) {
            Err(VerdictStoreError::PruneServer(failure)) => assert_ne!(
                failure.exit_code, 0,
                "服务拒了的写照退出码表报：{}",
                failure.description
            ),
            Err(other) => panic!("服务拒了的写应当交回 prune-server 的错，交回的是 {other:?}"),
            Ok(()) => panic!("服务拒了的写交回成功：写没确认不许当写下了"),
        }
    }

    /// 连不上（重连到上限）、服务一直忙、服务停了照 prune-server 客户端的退出码表报 75（服务不可用），被拒报 3；交回错误的那一种处置不退出进程。
    /// 判别力：服务不可用报成别的退出码（入口脚本照 75 认「等服务修好重交」），这条红。
    #[test]
    fn an_unreachable_or_stopped_service_is_reported_with_the_unavailable_exit_code() {
        let unreachable = ClientFailure::ServerUnreachable {
            attempts: 30,
            elapsed: std::time::Duration::from_secs(120),
            last_error: "connection refused".to_string(),
        };
        let stopped = ClientFailure::ServerStopped {
            message: "服务停了".to_string(),
        };
        let rejected = ClientFailure::Rejected {
            status: RejectionStatus::StorageFailed,
            message: "存储出错".to_string(),
        };
        for (failure, expected_exit_code) in [
            (unreachable, PRUNE_SERVER_UNAVAILABLE_EXIT_CODE),
            (stopped, PRUNE_SERVER_UNAVAILABLE_EXIT_CODE),
            (rejected, OTHER_CLIENT_FAILURE_EXIT_CODE),
        ] {
            match prune_server_client_failed(
                PruneServerFailureHandling::ReturnTheError,
                "成批点读",
                &failure,
            ) {
                VerdictStoreError::PruneServer(reported) => {
                    assert_eq!(reported.exit_code, expected_exit_code, "{failure}");
                }
                other @ (VerdictStoreError::StoredBytesDoNotMatchTheLayout { .. }
                | VerdictStoreError::EveryU32VerdictVectorNumberIsAlreadyRegistered
                | VerdictStoreError::DifferentVerdictsUnderAStoredBlockKeyNotDecidedInTheFirstVersion { .. }
                | VerdictStoreError::DifferentConclusionUnderAStoredKeyNotDecidedInTheFirstVersion { .. }
                | VerdictStoreError::ObserverWitnessBatchRefusedByTheTestSwitch) => {
                    panic!("{failure}：应当报成 prune-server 的错，报成了 {other:?}")
                }
            }
        }
    }

    /// 前缀扫被条数上限截断（单测把一帧改成最多 2 条）：拆前缀分着扫，前缀下的 40 行一行不少、按键升序交回，连正好等于前缀的那一行；
    /// 前缀之外的行不交。
    /// 判别力（`crates/mutations.tsv`）：截断的那一帧当成扫全了（照交回来的那 2 行收工），这条红。
    #[test]
    fn a_prefix_scan_cut_off_by_the_entry_limit_splits_the_prefix_and_reads_every_row() {
        let library = TestLibrary::new("prefix-scan-split");
        let service = library.service();
        let prefix = b"split/".to_vec();
        let mut rows: Vec<(Vec<u8>, Vec<u8>)> = (0u8..40)
            .map(|number| {
                let mut key = prefix.clone();
                key.push(number.wrapping_mul(37));
                key.extend_from_slice(&[number; 3]);
                (key, vec![number + 1])
            })
            .collect();
        rows.push((prefix.clone(), b"the prefix itself".to_vec()));
        let mut outside = b"split-not/".to_vec();
        outside.push(1);
        service
            .write_rows_for_a_test(
                rows.iter()
                    .cloned()
                    .chain(std::iter::once((outside, b"outside".to_vec())))
                    .collect(),
            )
            .expect("写得了临时 prune-server");
        service.scan_at_most_entries_per_frame_for_a_test(2);
        let read_back: Vec<(Vec<u8>, Vec<u8>)> = service
            .rows_with_prefix(&prefix)
            .expect("扫得了")
            .into_iter()
            .map(|(key_bytes, value_bytes)| (key_bytes.into_vec(), value_bytes.into_vec()))
            .collect();
        rows.sort();
        assert_eq!(read_back, rows, "前缀下的每一行都读回、按键升序");
    }

    /// 成批点读一帧问的键超过一帧的上限（`PRUNE_SERVER_READ_KEYS_PER_FRAME`）照样一个不少、次序与问的相同：在的读回值，不在的读成不在。
    #[test]
    fn a_batched_read_of_more_keys_than_one_frame_answers_every_key_in_order() {
        let library = TestLibrary::new("batched-read-many-keys");
        let service = library.service();
        let key_count = PRUNE_SERVER_READ_KEYS_PER_FRAME + 3;
        let keys: Vec<Vec<u8>> = (0..key_count)
            .map(|number| format!("many/{number:08}").into_bytes())
            .collect();
        service
            .write_rows_for_a_test(
                keys.iter()
                    .step_by(2)
                    .map(|key| (key.clone(), key.clone()))
                    .collect(),
            )
            .expect("写得了临时 prune-server");
        let key_slices: Vec<&[u8]> = keys.iter().map(Vec::as_slice).collect();
        let values = service.read_values(&key_slices).expect("读得了");
        assert_eq!(values.len(), key_count, "问几个键回几个值");
        for (position, (key, value)) in keys.iter().zip(&values).enumerate() {
            let expected = (position % 2 == 0).then(|| key.clone());
            assert_eq!(value, &expected, "位置 {position} 上的键");
        }
    }

    /// 单独一行的值长过帧长上限（录下的流程、判表这一类：崩溃点重放曾有三项退 3 的「帧体超过帧长上限」）：写得进（照 Python 客户端的格式分段存），
    /// 点读拼回逐字节相同，按前缀扫只交原键那两行（段键与计数键不当行），前缀下有行。
    /// 判别力（`crates/mutations.tsv`）：写一批时不先分段、整行照一帧发（帧体超过帧长上限，协议出错退 3），这条红。
    #[test]
    fn a_row_longer_than_the_frame_limit_is_written_in_segments_and_reads_back_whole() {
        let library = TestLibrary::new("row-longer-than-the-frame-limit");
        let service = library.service();
        let frame_limit = usize::try_from(prune_server::protocol::MAXIMUM_FRAME_BODY_BYTES)
            .expect("u32 放得进 usize");
        let large_value: Vec<u8> = (0..frame_limit + 1)
            .map(|position| u8::try_from(position % 251).expect("取模 251 放得进 u8"))
            .collect();
        let rows = vec![
            (b"large/row".to_vec(), large_value.clone()),
            (b"large/small".to_vec(), b"small".to_vec()),
        ];
        service
            .write_rows_for_a_test(rows.clone())
            .expect("长过帧长上限的一行分段写得进");
        assert!(
            service
                .read_values(&[b"large/row".as_slice()])
                .expect("读得了")
                == vec![Some(large_value)],
            "点读拼回逐字节相同"
        );
        let scanned: Vec<(Vec<u8>, Vec<u8>)> = service
            .rows_with_prefix(b"large/")
            .expect("扫得了")
            .into_iter()
            .map(|(key_bytes, value_bytes)| (key_bytes.into_vec(), value_bytes.into_vec()))
            .collect();
        assert!(scanned == rows, "按前缀扫只交原键那两行、大值拼回");
        assert!(
            service.has_a_row_with_prefix(b"large/row").expect("扫得了"),
            "分段存的那一行算前缀下的一行"
        );
    }

    /// 分段存过的一行删掉（写成删除记号）：照高水位仍存成清单加空段（服务里原键的值不空），点读拼回空值读成不在，按前缀扫一行都不交，前缀下没有行。
    /// 判别力（`crates/mutations.tsv`）：前缀下有没有行只看扫回来的头一行值空不空、不管它是不是分段存放的一部分（清单拼回来是删除记号），这条红。
    #[test]
    fn a_deleted_segmented_row_reads_as_absent_and_leaves_no_row_under_its_prefix() {
        let library = TestLibrary::new("deleted-segmented-row");
        let service = library.service();
        service
            .write_rows_for_a_test(vec![(
                b"deleted/row".to_vec(),
                vec![7u8; prune_server::segmented_values::SEGMENT_BYTES + 1],
            )])
            .expect("分段写得进");
        service
            .write_rows_for_a_test(vec![(b"deleted/row".to_vec(), Vec::new())])
            .expect("删除记号写得进");
        assert_eq!(
            service
                .read_values(&[b"deleted/row".as_slice()])
                .expect("读得了"),
            vec![None],
            "拼回的是删除记号，读成不在"
        );
        assert!(
            service
                .rows_with_prefix(b"deleted/")
                .expect("扫得了")
                .is_empty(),
            "按前缀扫一行都不交"
        );
        assert!(
            !service.has_a_row_with_prefix(b"deleted/").expect("扫得了"),
            "前缀下没有行"
        );
    }

    /// 值本身以分段清单的开头起头（照格式也分段存），前缀扫被条数上限截断（单测把一帧改成最多 1 条，拆前缀一路拆到原键与段键）：
    /// 拆前缀时点读正好等于前缀的那一把键交的是服务里存着的原样（清单、段），拼接器照样拼回整值。
    /// 判别力（`crates/mutations.tsv`）：拆前缀时点读那一把键也拼回整值（拼回来的整值以清单开头，被拼接器当成清单，解不开），这条红。
    #[test]
    fn a_split_prefix_scan_hands_the_stored_bytes_of_the_exact_prefix_to_the_assembler() {
        let library = TestLibrary::new("split-scan-of-a-segmented-row");
        let service = library.service();
        let mut value = prune_server::segmented_values::SEGMENT_MANIFEST_MAGIC.to_vec();
        value.extend_from_slice(b"a value that only looks like a manifest");
        service
            .write_rows_for_a_test(vec![(b"magic/row".to_vec(), value.clone())])
            .expect("分段写得进");
        service.scan_at_most_entries_per_frame_for_a_test(1);
        let scanned: Vec<(Vec<u8>, Vec<u8>)> = service
            .rows_with_prefix(b"magic/")
            .expect("扫得了")
            .into_iter()
            .map(|(key_bytes, value_bytes)| (key_bytes.into_vec(), value_bytes.into_vec()))
            .collect();
        assert_eq!(scanned, vec![(b"magic/row".to_vec(), value)]);
    }
}
