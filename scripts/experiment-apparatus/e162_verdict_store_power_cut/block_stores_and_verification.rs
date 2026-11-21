//! 三种块存储与文件库的读写，以及对一个库逐块分类、核对出报告。

use super::*;

// ============================== 候选的存储（登记 5.2，照 S1 那份源文件） ==============================

#[derive(Debug)]
pub(super) enum StoreError {
    Redb(String),
    RocksDatabase(String),
    File(String),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::Redb(message) => write!(formatter, "redb: {message}"),
            StoreError::RocksDatabase(message) => write!(formatter, "rocksdb: {message}"),
            StoreError::File(message) => write!(formatter, "file: {message}"),
        }
    }
}

fn redb_error(error: impl std::fmt::Display) -> StoreError {
    StoreError::Redb(error.to_string())
}

fn rocksdb_error(error: impl std::fmt::Display) -> StoreError {
    StoreError::RocksDatabase(error.to_string())
}

fn file_error(context: &str, error: impl std::fmt::Display) -> StoreError {
    StoreError::File(format!("{context}: {error}"))
}

#[derive(Debug, PartialEq, Eq)]
enum ReadOutcome {
    Found(Vec<u8>),
    NotFound,
    ReadError(String),
}

enum StoreBackend {
    Redb(redb::Database),
    RocksDatabase(rocksdb::DB),
    File(FileLibrary),
}

pub(super) struct BlockStore {
    settings: WriteSettings,
    backend: StoreBackend,
}

fn rocksdb_options() -> rocksdb::Options {
    let mut options = rocksdb::Options::default();
    options.create_if_missing(true);
    options.set_compression_type(rocksdb::DBCompressionType::Lz4);
    options
}

/// 写入用的打开：库不在就建。
pub(super) fn open_store_for_writing(candidate: Candidate, settings: WriteSettings, library: &Path) -> Result<BlockStore, StoreError> {
    let backend = match candidate {
        Candidate::RedbQuickRepair | Candidate::RedbDefault => {
            fs::create_dir_all(library).map_err(|error| file_error("建 redb 库目录", error))?;
            StoreBackend::Redb(redb::Database::create(library.join(REDB_FILE_NAME)).map_err(redb_error)?)
        }
        Candidate::RocksDatabase => StoreBackend::RocksDatabase(rocksdb::DB::open(&rocksdb_options(), library).map_err(rocksdb_error)?),
        Candidate::HardenedFile => StoreBackend::File(FileLibrary::open(library, settings.file_sync_steps, true)?),
    };
    Ok(BlockStore { settings, backend })
}

/// 核对用的打开：同一个打开调用（redb 的 `Database::create`、RocksDB 的 `DB::open`），F 不建新库。
fn open_existing_store(candidate: Candidate, library: &Path) -> Result<BlockStore, StoreError> {
    let settings = WriteSettings::registered_for(candidate);
    let backend = match candidate {
        Candidate::RedbQuickRepair | Candidate::RedbDefault => {
            StoreBackend::Redb(redb::Database::create(library.join(REDB_FILE_NAME)).map_err(redb_error)?)
        }
        Candidate::RocksDatabase => StoreBackend::RocksDatabase(rocksdb::DB::open(&rocksdb_options(), library).map_err(rocksdb_error)?),
        Candidate::HardenedFile => StoreBackend::File(FileLibrary::open(library, settings.file_sync_steps, false)?),
    };
    Ok(BlockStore { settings, backend })
}

impl BlockStore {
    /// 一次持久提交写一块；返回 Ok 就算这一块已确认。
    pub(super) fn put_block(&mut self, key: &BlockKey, value: &[u8]) -> Result<(), StoreError> {
        match &mut self.backend {
            StoreBackend::Redb(database) => {
                let mut transaction = database.begin_write().map_err(redb_error)?;
                let durability = match self.settings.redb_commit_durability {
                    RedbCommitDurability::Immediate => redb::Durability::Immediate,
                    RedbCommitDurability::NotPersisted => redb::Durability::None,
                };
                transaction.set_durability(durability).map_err(redb_error)?;
                transaction.set_quick_repair(self.settings.redb_repair_mode == RedbRepairMode::QuickRepair);
                {
                    let mut table = transaction.open_table(REDB_TABLE).map_err(redb_error)?;
                    table.insert(&key[..], value).map_err(redb_error)?;
                }
                transaction.commit().map_err(redb_error)
            }
            StoreBackend::RocksDatabase(database) => {
                let mut write_options = rocksdb::WriteOptions::default();
                write_options.disable_wal(false);
                write_options.set_sync(self.settings.rocksdb_write_sync == RocksDatabaseWriteSync::SyncEachWrite);
                database.put_opt(key, value, &write_options).map_err(rocksdb_error)
            }
            StoreBackend::File(library) => library.put_block(key, value),
        }
    }

    fn read_block(&self, key: &BlockKey) -> ReadOutcome {
        match &self.backend {
            StoreBackend::Redb(database) => {
                let transaction = match database.begin_read() {
                    Ok(transaction) => transaction,
                    Err(error) => return ReadOutcome::ReadError(error.to_string()),
                };
                let table = match transaction.open_table(REDB_TABLE) {
                    Ok(table) => table,
                    Err(redb::TableError::TableDoesNotExist(_)) => return ReadOutcome::NotFound,
                    Err(error) => return ReadOutcome::ReadError(error.to_string()),
                };
                let outcome = match redb::ReadableTable::get(&table, &key[..]) {
                    Ok(Some(guard)) => ReadOutcome::Found(guard.value().to_vec()),
                    Ok(None) => ReadOutcome::NotFound,
                    Err(error) => ReadOutcome::ReadError(error.to_string()),
                };
                outcome
            }
            StoreBackend::RocksDatabase(database) => match database.get(key) {
                Ok(Some(value)) => ReadOutcome::Found(value),
                Ok(None) => ReadOutcome::NotFound,
                Err(error) => ReadOutcome::ReadError(error.to_string()),
            },
            StoreBackend::File(library) => library.read_block(key),
        }
    }

    fn enumerate_keys_with_value_lengths(&self) -> Result<Vec<(Vec<u8>, u64)>, StoreError> {
        match &self.backend {
            StoreBackend::Redb(database) => {
                let transaction = database.begin_read().map_err(redb_error)?;
                let table = match transaction.open_table(REDB_TABLE) {
                    Ok(table) => table,
                    Err(redb::TableError::TableDoesNotExist(_)) => return Ok(Vec::new()),
                    Err(error) => return Err(redb_error(error)),
                };
                let mut entries = Vec::new();
                for item in redb::ReadableTable::iter(&table).map_err(redb_error)? {
                    let (key, value) = item.map_err(redb_error)?;
                    entries.push((key.value().to_vec(), u64::try_from(value.value().len()).expect("块值长度装得进 u64")));
                }
                Ok(entries)
            }
            StoreBackend::RocksDatabase(database) => {
                let mut entries = Vec::new();
                for item in database.iterator(rocksdb::IteratorMode::Start) {
                    let (key, value) = item.map_err(rocksdb_error)?;
                    entries.push((key.to_vec(), u64::try_from(value.len()).expect("块值长度装得进 u64")));
                }
                Ok(entries)
            }
            StoreBackend::File(library) => library.enumerate_keys_with_value_lengths(),
        }
    }
}

/// 一块一个文件：`<根>/<节点 8 位十六进制>/<段号 8 位十六进制>/<起点 16 位十六进制>-<指纹 32 位十六进制>.blk`。
pub(super) struct FileLibrary {
    root: PathBuf,
    pub(super) file_sync_steps: FileSyncSteps,
    known_directories: HashSet<PathBuf>,
}

fn sync_directory(directory: &Path) -> Result<(), StoreError> {
    File::open(directory)
        .and_then(|handle| handle.sync_all())
        .map_err(|error| file_error(&format!("目录 fsync {}", directory.display()), error))
}

fn format_file_contents() -> [u8; FORMAT_FILE_BYTES] {
    let mut contents = [0u8; FORMAT_FILE_BYTES];
    contents[0..8].copy_from_slice(FORMAT_MAGIC);
    contents[8..12].copy_from_slice(&FILE_FORMAT_VERSION.to_le_bytes());
    let checksum = crc32c(&[&contents[0..12]]);
    contents[12..16].copy_from_slice(&checksum.to_le_bytes());
    contents
}

fn block_file_relative_path(key: &[u8]) -> Option<PathBuf> {
    if key.len() != BLOCK_KEY_BYTES {
        return None;
    }
    let node = u32::from_be_bytes(key[0..4].try_into().expect("4 字节"));
    let segment = u32::from_be_bytes(key[4..8].try_into().expect("4 字节"));
    let first_state = u64::from_be_bytes(key[8..16].try_into().expect("8 字节"));
    Some(
        PathBuf::from(format!("{node:08x}"))
            .join(format!("{segment:08x}"))
            .join(format!("{first_state:016x}-{}.blk", hexadecimal(&key[16..32]))),
    )
}

fn temporary_path_of(final_path: &Path) -> PathBuf {
    let mut name = final_path.as_os_str().to_owned();
    name.push(".tmp");
    PathBuf::from(name)
}

fn block_file_header(key: &BlockKey, value: &[u8]) -> [u8; FILE_HEADER_BYTES] {
    let mut header = [0u8; FILE_HEADER_BYTES];
    header[0..4].copy_from_slice(FILE_MAGIC);
    header[4..8].copy_from_slice(&FILE_FORMAT_VERSION.to_le_bytes());
    header[8..40].copy_from_slice(key);
    let value_length = u32::try_from(value.len()).expect("块值装得进 u32");
    header[40..44].copy_from_slice(&value_length.to_le_bytes());
    let checksum = crc32c(&[&header[..FILE_HEADER_CHECKSUMMED_BYTES], value]);
    header[44..48].copy_from_slice(&checksum.to_le_bytes());
    header
}

fn validate_block_file(bytes: &[u8], key: &BlockKey) -> Result<Vec<u8>, String> {
    if bytes.len() < FILE_HEADER_BYTES {
        return Err(format!("文件只有 {} 字节，短于文件头", bytes.len()));
    }
    if &bytes[0..4] != FILE_MAGIC {
        return Err("魔数不对".to_string());
    }
    if bytes[4..8] != FILE_FORMAT_VERSION.to_le_bytes() {
        return Err("版本不对".to_string());
    }
    if &bytes[8..40] != key {
        return Err("头里的键与文件名不符".to_string());
    }
    let value_length = u32::from_le_bytes(bytes[40..44].try_into().expect("4 字节"));
    let value = &bytes[FILE_HEADER_BYTES..];
    if u64::from(value_length) != u64::try_from(value.len()).expect("装得进 u64") {
        return Err(format!("长度字段 {value_length} 与文件里的 {} 字节块值不符", value.len()));
    }
    let stored_checksum = u32::from_le_bytes(bytes[44..48].try_into().expect("4 字节"));
    if crc32c(&[&bytes[..FILE_HEADER_CHECKSUMMED_BYTES], value]) != stored_checksum {
        return Err("CRC-32C 不对".to_string());
    }
    Ok(value.to_vec())
}

fn hexadecimal_name_as_u32(path: &Path) -> Option<u32> {
    if !path.is_dir() {
        return None;
    }
    let name = path.file_name()?.to_str()?;
    if name.len() != 8 {
        return None;
    }
    u32::from_str_radix(name, 16).ok()
}

fn parse_hexadecimal(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|offset| u8::from_str_radix(&text[offset..offset + 2], 16).ok())
        .collect()
}

impl FileLibrary {
    /// 打开：根目录在、FORMAT 在且对，否则打开失败；打开时删掉残留的 `*.tmp`。
    pub(super) fn open(root: &Path, file_sync_steps: FileSyncSteps, allow_create: bool) -> Result<FileLibrary, StoreError> {
        if !root.exists() {
            if !allow_create {
                return Err(StoreError::File(format!("根目录 {} 不在", root.display())));
            }
            Self::create_root_atomically(root)?;
        }
        let format_bytes = fs::read(root.join(FORMAT_FILE_NAME)).map_err(|error| file_error("读 FORMAT", error))?;
        if format_bytes[..] != format_file_contents()[..] {
            return Err(StoreError::File("FORMAT 不对".to_string()));
        }
        let mut known_directories = HashSet::new();
        for node_entry in fs::read_dir(root).map_err(|error| file_error("列根目录", error))? {
            let node_path = node_entry.map_err(|error| file_error("列根目录", error))?.path();
            if !node_path.is_dir() {
                continue;
            }
            known_directories.insert(node_path.clone());
            for segment_entry in fs::read_dir(&node_path).map_err(|error| file_error("列节点目录", error))? {
                let segment_path = segment_entry.map_err(|error| file_error("列节点目录", error))?.path();
                if !segment_path.is_dir() {
                    continue;
                }
                known_directories.insert(segment_path.clone());
                for block_entry in fs::read_dir(&segment_path).map_err(|error| file_error("列段目录", error))? {
                    let block_path = block_entry.map_err(|error| file_error("列段目录", error))?.path();
                    if block_path.extension().is_some_and(|extension| extension == "tmp") {
                        fs::remove_file(&block_path).map_err(|error| file_error("删残留的临时文件", error))?;
                    }
                }
            }
        }
        Ok(FileLibrary { root: root.to_path_buf(), file_sync_steps, known_directories })
    }

    fn create_root_atomically(root: &Path) -> Result<(), StoreError> {
        let mut creating_name = root.as_os_str().to_owned();
        creating_name.push(".creating");
        let creating = PathBuf::from(creating_name);
        if creating.exists() {
            fs::remove_dir_all(&creating).map_err(|error| file_error("删上一次没建完的根目录", error))?;
        }
        fs::create_dir_all(&creating).map_err(|error| file_error("建根目录", error))?;
        let mut format_file = File::create(creating.join(FORMAT_FILE_NAME)).map_err(|error| file_error("建 FORMAT", error))?;
        format_file.write_all(&format_file_contents()).map_err(|error| file_error("写 FORMAT", error))?;
        format_file.sync_all().map_err(|error| file_error("fsync FORMAT", error))?;
        sync_directory(&creating)?;
        fs::rename(&creating, root).map_err(|error| file_error("根目录改名", error))?;
        let parent = root.parent().expect("根目录有父目录");
        sync_directory(parent)
    }

    /// 节点与段目录不在就建；除 F-nofsync 外，建完对新目录与它的父目录各 fsync。
    fn ensure_directory(&mut self, directory: &Path) -> Result<(), StoreError> {
        if self.known_directories.contains(directory) {
            return Ok(());
        }
        let parent = directory.parent().expect("目录有父目录").to_path_buf();
        if parent != self.root {
            self.ensure_directory(&parent)?;
        }
        if !directory.exists() {
            fs::create_dir(directory).map_err(|error| file_error("建目录", error))?;
            if self.file_sync_steps != FileSyncSteps::NeitherTemporaryFileNorDirectory {
                sync_directory(directory)?;
                sync_directory(&parent)?;
            }
        }
        self.known_directories.insert(directory.to_path_buf());
        Ok(())
    }

    fn final_path_of(&self, key: &BlockKey) -> PathBuf {
        self.root.join(block_file_relative_path(key).expect("块键 32 字节"))
    }

    /// 登记 5.2：`<终名>.tmp` 一次 `write_all(头 + 块值)` → `sync_all` → `rename` 到终名 → 打开父目录 `sync_all`。
    pub(super) fn put_block(&mut self, key: &BlockKey, value: &[u8]) -> Result<(), StoreError> {
        let final_path = self.final_path_of(key);
        let directory = final_path.parent().expect("块文件有父目录").to_path_buf();
        self.ensure_directory(&directory)?;
        let temporary_path = temporary_path_of(&final_path);
        let mut contents = Vec::with_capacity(FILE_HEADER_BYTES + value.len());
        contents.extend_from_slice(&block_file_header(key, value));
        contents.extend_from_slice(value);
        let mut handle = File::create(&temporary_path).map_err(|error| file_error("建临时文件", error))?;
        handle.write_all(&contents).map_err(|error| file_error("写临时文件", error))?;
        if self.file_sync_steps != FileSyncSteps::NeitherTemporaryFileNorDirectory {
            handle.sync_all().map_err(|error| file_error("fsync 临时文件", error))?;
        }
        drop(handle);
        fs::rename(&temporary_path, &final_path).map_err(|error| file_error("改名", error))?;
        if self.file_sync_steps == FileSyncSteps::TemporaryFileAndDirectory {
            sync_directory(&directory)?;
        }
        Ok(())
    }

    fn read_block(&self, key: &BlockKey) -> ReadOutcome {
        match fs::read(self.final_path_of(key)) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => ReadOutcome::NotFound,
            Err(error) => ReadOutcome::ReadError(error.to_string()),
            Ok(bytes) => match validate_block_file(&bytes, key) {
                Ok(value) => ReadOutcome::Found(value),
                Err(reason) => ReadOutcome::ReadError(reason),
            },
        }
    }

    fn enumerate_keys_with_value_lengths(&self) -> Result<Vec<(Vec<u8>, u64)>, StoreError> {
        let mut entries = Vec::new();
        for node_entry in fs::read_dir(&self.root).map_err(|error| file_error("列根目录", error))? {
            let node_path = node_entry.map_err(|error| file_error("列根目录", error))?.path();
            let Some(node) = hexadecimal_name_as_u32(&node_path) else { continue };
            for segment_entry in fs::read_dir(&node_path).map_err(|error| file_error("列节点目录", error))? {
                let segment_path = segment_entry.map_err(|error| file_error("列节点目录", error))?.path();
                let Some(segment) = hexadecimal_name_as_u32(&segment_path) else { continue };
                for block_entry in fs::read_dir(&segment_path).map_err(|error| file_error("列段目录", error))? {
                    let block_entry = block_entry.map_err(|error| file_error("列段目录", error))?;
                    let name = block_entry.file_name().to_string_lossy().to_string();
                    let Some(stem) = name.strip_suffix(".blk") else { continue };
                    let Some((first_state_text, fingerprint_text)) = stem.split_once('-') else { continue };
                    let (Ok(first_state), Some(fingerprint)) =
                        (u64::from_str_radix(first_state_text, 16), parse_hexadecimal(fingerprint_text))
                    else {
                        continue;
                    };
                    let mut key = Vec::with_capacity(BLOCK_KEY_BYTES);
                    key.extend_from_slice(&node.to_be_bytes());
                    key.extend_from_slice(&segment.to_be_bytes());
                    key.extend_from_slice(&first_state.to_be_bytes());
                    key.extend_from_slice(&fingerprint);
                    let file_length = block_entry.metadata().map_err(|error| file_error("读文件长度", error))?.len();
                    entries.push((key, file_length.saturating_sub(u64::try_from(FILE_HEADER_BYTES).expect("48"))));
                }
            }
        }
        Ok(entries)
    }
}

/// RocksDB 的 `LOG` 里数 L0 落盘与合并各完成了几次（V25 / Q4q）。
pub(super) fn rocksdb_background_counts(library: &Path) -> (u64, u64) {
    let log_text = fs::read_to_string(library.join("LOG")).unwrap_or_default();
    let flushes = u64::try_from(log_text.matches("\"event\": \"flush_finished\"").count()).expect("装得进 u64");
    let compactions = u64::try_from(log_text.matches("\"event\": \"compaction_finished\"").count()).expect("装得进 u64");
    (flushes, compactions)
}

// ============================== 逐块核（登记 5.1 的表；补 6 Q4a–Q4f） ==============================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockClass {
    /// C 标记下标 < k。
    Confirmed,
    /// B 标记下标 < k 而 C 不 < k。
    InFlight,
    /// B 不 < k。
    NotStarted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReadKind {
    MatchingBytes,
    DifferentBytes,
    NotFound,
    ReadError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockVerdict {
    Correct,
    /// Q4a
    LostConfirmed,
    /// Q4b
    SilentlyCorrupted,
    /// Q4c
    ConfirmedReadError,
    /// Q4d
    UnconfirmedReadError,
    /// Q4e
    Phantom,
}

fn classify_block(class: BlockClass, read: ReadKind) -> BlockVerdict {
    match (class, read) {
        (BlockClass::Confirmed, ReadKind::MatchingBytes) => BlockVerdict::Correct,
        (BlockClass::Confirmed, ReadKind::DifferentBytes) => BlockVerdict::SilentlyCorrupted,
        (BlockClass::Confirmed, ReadKind::NotFound) => BlockVerdict::LostConfirmed,
        (BlockClass::Confirmed, ReadKind::ReadError) => BlockVerdict::ConfirmedReadError,
        (BlockClass::InFlight, ReadKind::MatchingBytes) => BlockVerdict::Correct,
        (BlockClass::InFlight, ReadKind::DifferentBytes) => BlockVerdict::SilentlyCorrupted,
        (BlockClass::InFlight, ReadKind::NotFound) => BlockVerdict::Correct,
        (BlockClass::InFlight, ReadKind::ReadError) => BlockVerdict::UnconfirmedReadError,
        (BlockClass::NotStarted, ReadKind::MatchingBytes | ReadKind::DifferentBytes) => BlockVerdict::Phantom,
        (BlockClass::NotStarted, ReadKind::NotFound) => BlockVerdict::Correct,
        (BlockClass::NotStarted, ReadKind::ReadError) => BlockVerdict::UnconfirmedReadError,
    }
}

fn values_match(found: &[u8], expected: &[u8]) -> bool {
    found.len() == expected.len() && found == expected
}

/// 一次核的计划：块按 j 升序逐个写，所以已确认的是前缀 [0, confirmed_end)，在途的是 [confirmed_end, started_end)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct VerificationPlan {
    pub(super) states_per_block: u64,
    pub(super) block_count: u64,
    pub(super) confirmed_end: u64,
    pub(super) started_end: u64,
}

impl VerificationPlan {
    fn class_of(&self, block_index: u64) -> BlockClass {
        if block_index < self.confirmed_end {
            BlockClass::Confirmed
        } else if block_index < self.started_end {
            BlockClass::InFlight
        } else {
            BlockClass::NotStarted
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct VerificationReport {
    pub(super) open_failed: bool,
    pub(super) open_error: String,
    /// 只有 K：第一次打开失败之后调 DB::repair 再开，能开就是 Some(true)。
    pub(super) opened_after_repair: Option<bool>,
    pub(super) open_microseconds: u64,
    pub(super) lost_confirmed: Vec<u64>,
    pub(super) silently_corrupted: Vec<u64>,
    pub(super) confirmed_read_errors: Vec<u64>,
    pub(super) unconfirmed_read_errors: Vec<u64>,
    pub(super) phantom_keys: Vec<String>,
    pub(super) enumerated_key_count: u64,
    pub(super) enumeration_error: String,
    pub(super) first_read_error: String,
}

fn elapsed_microseconds(since: Instant) -> u64 {
    u64::try_from(since.elapsed().as_micros()).expect("微秒数装得进 u64")
}

fn record_open_failure_then_try_repair(
    candidate: Candidate,
    library: &Path,
    open_error: StoreError,
    report: &mut VerificationReport,
) -> Option<BlockStore> {
    report.open_failed = true;
    report.open_error = open_error.to_string();
    match candidate {
        Candidate::RocksDatabase => {
            let repaired = rocksdb::DB::repair(&rocksdb_options(), library)
                .map_err(rocksdb_error)
                .and_then(|()| open_existing_store(candidate, library));
            report.opened_after_repair = Some(repaired.is_ok());
            repaired.ok()
        }
        Candidate::RedbQuickRepair | Candidate::RedbDefault | Candidate::HardenedFile => None,
    }
}

pub(super) fn verify_library(candidate: Candidate, library: &Path, plan: &VerificationPlan) -> VerificationReport {
    let mut report = VerificationReport::default();
    let open_started = Instant::now();
    let opened = open_existing_store(candidate, library);
    report.open_microseconds = elapsed_microseconds(open_started);
    let store = match opened {
        Ok(store) => store,
        Err(open_error) => match record_open_failure_then_try_repair(candidate, library, open_error, &mut report) {
            Some(repaired) => repaired,
            None => return report,
        },
    };
    let fingerprint = input_fingerprint();
    let mut phantom_keys = std::collections::BTreeSet::new();
    for block_index in 0..plan.block_count {
        let class = plan.class_of(block_index);
        let key = interleaved_block_key(block_index, plan.states_per_block, &fingerprint);
        let read = match store.read_block(&key) {
            ReadOutcome::Found(found) => {
                let expected = block_value(ValuePattern::Random, SEED_CRASH_BLOCKS, block_index, plan.states_per_block);
                if values_match(&found, &expected) {
                    ReadKind::MatchingBytes
                } else {
                    ReadKind::DifferentBytes
                }
            }
            ReadOutcome::NotFound => ReadKind::NotFound,
            ReadOutcome::ReadError(reason) => {
                if report.first_read_error.is_empty() {
                    report.first_read_error = format!("block={block_index}:{reason}");
                }
                ReadKind::ReadError
            }
        };
        match classify_block(class, read) {
            BlockVerdict::Correct => {}
            BlockVerdict::LostConfirmed => report.lost_confirmed.push(block_index),
            BlockVerdict::SilentlyCorrupted => report.silently_corrupted.push(block_index),
            BlockVerdict::ConfirmedReadError => report.confirmed_read_errors.push(block_index),
            BlockVerdict::UnconfirmedReadError => report.unconfirmed_read_errors.push(block_index),
            BlockVerdict::Phantom => {
                phantom_keys.insert(hexadecimal(&key));
            }
        }
    }
    let started_keys: HashSet<BlockKey> = (0..plan.started_end)
        .map(|block_index| interleaved_block_key(block_index, plan.states_per_block, &fingerprint))
        .collect();
    match store.enumerate_keys_with_value_lengths() {
        Ok(entries) => {
            for (key, _value_length) in entries {
                report.enumerated_key_count += 1;
                let is_started = <[u8; BLOCK_KEY_BYTES]>::try_from(key.as_slice())
                    .is_ok_and(|fixed_key| started_keys.contains(&fixed_key));
                if !is_started {
                    phantom_keys.insert(hexadecimal(&key));
                }
            }
        }
        Err(error) => report.enumeration_error = error.to_string(),
    }
    report.phantom_keys = phantom_keys.into_iter().collect();
    report
}

/// PC4-S、PC4-P、PC4-O 在「全部条目」重放出的盘面上动的那一处。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LibraryDamage {
    Untouched,
    /// PC4-S：经写入接口把第 7 块改写成「第 40 000 字节异或 0x5A」的值。
    SilentCorruption,
    /// PC4-P：经写入接口加一个计划外的键。
    PhantomKey,
    /// PC4-O：毁掉打开要用的那一处。
    OpenBreaking,
}

impl LibraryDamage {
    pub(super) fn label(self) -> &'static str {
        match self {
            LibraryDamage::Untouched => "untouched",
            LibraryDamage::SilentCorruption => "silent",
            LibraryDamage::PhantomKey => "phantom",
            LibraryDamage::OpenBreaking => "open-breaking",
        }
    }

    pub(super) fn parse(text: &str) -> Option<LibraryDamage> {
        [LibraryDamage::Untouched, LibraryDamage::SilentCorruption, LibraryDamage::PhantomKey, LibraryDamage::OpenBreaking]
            .into_iter()
            .find(|damage| damage.label() == text)
    }
}

/// R1、R0 把库文件前 4096 字节写成 0xFF；K 把 CURRENT 换成 `MANIFEST-999999\n`；F 把 FORMAT 的 16 字节写成 0xFF。
fn break_library_opening(candidate: Candidate, library: &Path) -> Result<(), String> {
    let (path, bytes): (PathBuf, Vec<u8>) = match candidate {
        Candidate::RedbQuickRepair | Candidate::RedbDefault => (library.join(REDB_FILE_NAME), vec![0xFF; REDB_BROKEN_PREFIX_BYTES]),
        Candidate::RocksDatabase => (library.join("CURRENT"), ROCKSDB_BROKEN_CURRENT.to_vec()),
        Candidate::HardenedFile => (library.join(FORMAT_FILE_NAME), vec![0xFF; FORMAT_FILE_BYTES]),
    };
    let truncate = candidate == Candidate::RocksDatabase;
    let mut handle = OpenOptions::new()
        .write(true)
        .truncate(truncate)
        .open(&path)
        .map_err(|error| format!("打不开要毁的 {}：{error}", path.display()))?;
    handle.write_all(&bytes).map_err(|error| format!("写不进要毁的 {}：{error}", path.display()))?;
    handle.sync_all().map_err(|error| format!("fsync 不了要毁的 {}：{error}", path.display()))
}

pub(super) fn apply_library_damage(candidate: Candidate, library: &Path, damage: LibraryDamage, states_per_block: u64) -> Result<(), String> {
    let fingerprint = input_fingerprint();
    match damage {
        LibraryDamage::Untouched => Ok(()),
        LibraryDamage::SilentCorruption => {
            let mut store = open_existing_store(candidate, library).map_err(|error| format!("PC4-S 开不了库：{error}"))?;
            let mut value = block_value(ValuePattern::Random, SEED_CRASH_BLOCKS, SILENT_CORRUPTION_BLOCK, states_per_block);
            value[SILENT_CORRUPTION_BYTE_OFFSET] ^= SILENT_CORRUPTION_MASK;
            let key = interleaved_block_key(SILENT_CORRUPTION_BLOCK, states_per_block, &fingerprint);
            store.put_block(&key, &value).map_err(|error| format!("PC4-S 写不进：{error}"))
        }
        LibraryDamage::PhantomKey => {
            let mut store = open_existing_store(candidate, library).map_err(|error| format!("PC4-P 开不了库：{error}"))?;
            let phantom_key = compose_block_key(PHANTOM_KEY_NODE, 0, 0, &fingerprint);
            let value = block_value(ValuePattern::Random, SEED_CRASH_BLOCKS, 0, states_per_block);
            store.put_block(&phantom_key, &value).map_err(|error| format!("PC4-P 写不进：{error}"))
        }
        LibraryDamage::OpenBreaking => break_library_opening(candidate, library),
    }
}
