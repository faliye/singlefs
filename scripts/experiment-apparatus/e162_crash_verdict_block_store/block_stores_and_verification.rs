//! 三种块存储与文件库的读写，以及对一个库逐块分类、核对出报告并编解码。

use super::*;

#[derive(Debug)]
pub(super) enum StoreError {
    Redb(String),
    RocksDatabase(String),
    File(String),
    /// 单测用的「做到第 k 步就停」：留下的盘面与第 k 步之后被 SIGKILL 相同（页缓存还在）。
    AbandonedAtStep,
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::Redb(message) => write!(formatter, "redb: {message}"),
            StoreError::RocksDatabase(message) => write!(formatter, "rocksdb: {message}"),
            StoreError::File(message) => write!(formatter, "file: {message}"),
            StoreError::AbandonedAtStep => write!(formatter, "abandoned at step"),
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

/// 读一块的三种结局（登记 5.1 的三列）。
#[derive(Debug, PartialEq, Eq)]
pub(super) enum ReadOutcome {
    Found(Vec<u8>),
    NotFound,
    ReadError(String),
}

pub(super) enum StoreBackend {
    Redb(redb::Database),
    RocksDatabase(rocksdb::DB),
    File(FileLibrary),
}

pub(super) struct BlockStore {
    pub(super) candidate: Candidate,
    pub(super) write_path: WritePath,
    pub(super) backend: StoreBackend,
}

fn redb_durability_for(write_path: WritePath) -> redb::Durability {
    match write_path {
        WritePath::Registered => redb::Durability::Immediate,
        WritePath::RedbDurabilityNone => redb::Durability::None,
        WritePath::RocksDatabaseWithoutWriteAheadLog | WritePath::FileLazyRename | WritePath::FileDirectInPlace => {
            panic!("redb 臂不用 {} 这种写法", write_path.label())
        }
    }
}

fn rocksdb_options() -> rocksdb::Options {
    let mut options = rocksdb::Options::default();
    options.create_if_missing(true);
    options.set_compression_type(rocksdb::DBCompressionType::Lz4);
    options
}

fn rocksdb_write_options(write_path: WritePath) -> rocksdb::WriteOptions {
    let mut write_options = rocksdb::WriteOptions::default();
    let disable_write_ahead_log = write_path == WritePath::RocksDatabaseWithoutWriteAheadLog;
    write_options.disable_wal(disable_write_ahead_log);
    write_options.set_sync(!disable_write_ahead_log);
    write_options
}

/// 写入用的打开：库不在就建（redb 的库文件、RocksDB 的库目录、F 的根目录连同 FORMAT）。
pub(super) fn open_store_for_writing(candidate: Candidate, write_path: WritePath, library: &Path) -> Result<BlockStore, StoreError> {
    let backend = match candidate {
        Candidate::RedbQuickRepair | Candidate::RedbDefault => {
            fs::create_dir_all(library).map_err(|error| file_error("建 redb 库目录", error))?;
            StoreBackend::Redb(redb::Database::create(library.join(REDB_FILE_NAME)).map_err(redb_error)?)
        }
        Candidate::RocksDatabase => StoreBackend::RocksDatabase(rocksdb::DB::open(&rocksdb_options(), library).map_err(rocksdb_error)?),
        Candidate::HardenedFile => StoreBackend::File(FileLibrary::open(library, write_path, true)?),
    };
    Ok(BlockStore { candidate, write_path, backend })
}

/// 核对用的打开：同一个打开调用（redb 的 `Database::create`、RocksDB 的 `DB::open`），F 不建新库。
pub(super) fn open_existing_store(candidate: Candidate, library: &Path) -> Result<BlockStore, StoreError> {
    let backend = match candidate {
        Candidate::RedbQuickRepair | Candidate::RedbDefault => {
            StoreBackend::Redb(redb::Database::create(library.join(REDB_FILE_NAME)).map_err(redb_error)?)
        }
        Candidate::RocksDatabase => StoreBackend::RocksDatabase(rocksdb::DB::open(&rocksdb_options(), library).map_err(rocksdb_error)?),
        Candidate::HardenedFile => StoreBackend::File(FileLibrary::open(library, WritePath::Registered, false)?),
    };
    Ok(BlockStore { candidate, write_path: WritePath::Registered, backend })
}

impl BlockStore {
    /// 一次持久提交：blocks 是 1 块（主读法）或 16 块（G-batch16）。返回 Ok 就算这几块已确认。
    pub(super) fn put_blocks(&mut self, blocks: &[(BlockKey, Vec<u8>)]) -> Result<(), StoreError> {
        match &mut self.backend {
            StoreBackend::Redb(database) => {
                let quick_repair = self.candidate == Candidate::RedbQuickRepair;
                let mut transaction = database.begin_write().map_err(redb_error)?;
                transaction.set_durability(redb_durability_for(self.write_path)).map_err(redb_error)?;
                transaction.set_quick_repair(quick_repair);
                {
                    let mut table = transaction.open_table(REDB_TABLE).map_err(redb_error)?;
                    for (key, value) in blocks {
                        table.insert(&key[..], &value[..]).map_err(redb_error)?;
                    }
                }
                transaction.commit().map_err(redb_error)
            }
            StoreBackend::RocksDatabase(database) => {
                let write_options = rocksdb_write_options(self.write_path);
                if blocks.len() == 1 {
                    database.put_opt(blocks[0].0, &blocks[0].1, &write_options).map_err(rocksdb_error)
                } else {
                    let mut batch = rocksdb::WriteBatch::default();
                    for (key, value) in blocks {
                        batch.put(key, value);
                    }
                    database.write_opt(batch, &write_options).map_err(rocksdb_error)
                }
            }
            StoreBackend::File(library) => library.put_blocks(blocks),
        }
    }

    pub(super) fn read_block(&self, key: &BlockKey) -> ReadOutcome {
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

    /// 枚举库里的全部键与各自块值的长度（F 按文件名与文件长度，不读内容）。
    pub(super) fn enumerate_keys_with_value_lengths(&self) -> Result<Vec<(Vec<u8>, u64)>, StoreError> {
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

// ============================== 候选 F：加固的文件 ==============================

/// 一块一个文件：`<根>/<节点 8 位十六进制>/<段号 8 位十六进制>/<起点 16 位十六进制>-<指纹 32 位十六进制>.blk`。
pub(super) struct FileLibrary {
    root: PathBuf,
    pub(super) write_path: WritePath,
    /// F-lazyrename 攒着还没改名的（临时文件，终名）。
    pending_renames: Vec<(PathBuf, PathBuf)>,
    confirmations_in_this_run: u64,
    /// 单测用：还剩几步文件系统改动；None 是不限（产物里恒为 None）。
    pub(super) remaining_steps: Option<u64>,
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

pub(super) fn block_file_relative_path(key: &[u8]) -> Option<PathBuf> {
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

/// 文件头 48 字节：魔数｜版本 u32 小端｜块键｜块值长度 u32 小端｜CRC-32C（覆盖头的前 44 字节加整块值）。
pub(super) fn block_file_header(key: &BlockKey, value: &[u8]) -> [u8; FILE_HEADER_BYTES] {
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

/// 读路径的校验：长度、魔数、版本、头里的键、长度字段、CRC 任一不对就是读报错。
pub(super) fn validate_block_file(bytes: &[u8], key: &BlockKey) -> Result<Vec<u8>, String> {
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

impl FileLibrary {
    /// 打开：根目录在、FORMAT 在且对，否则打开失败；打开时删掉残留的 `*.tmp`。
    /// allow_create：根目录不在时（写入用）先经 `<根>.creating` 建好 FORMAT、改名成根目录。
    pub(super) fn open(root: &Path, write_path: WritePath, allow_create: bool) -> Result<FileLibrary, StoreError> {
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
        Ok(FileLibrary {
            root: root.to_path_buf(),
            write_path,
            pending_renames: Vec::new(),
            confirmations_in_this_run: 0,
            remaining_steps: None,
            known_directories,
        })
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

    /// 单测的「做到第 k 步就停」：每一步文件系统改动之前扣一步，扣光了就当被杀。
    fn take_step(&mut self) -> Result<(), StoreError> {
        match self.remaining_steps {
            None => Ok(()),
            Some(0) => Err(StoreError::AbandonedAtStep),
            Some(remaining) => {
                self.remaining_steps = Some(remaining - 1);
                Ok(())
            }
        }
    }

    /// 节点与段目录不在就建，建完对新目录与它的父目录各 fsync。
    fn ensure_directory(&mut self, directory: &Path) -> Result<(), StoreError> {
        if self.known_directories.contains(directory) {
            return Ok(());
        }
        let parent = directory.parent().expect("目录有父目录").to_path_buf();
        if parent != self.root {
            self.ensure_directory(&parent)?;
        }
        if !directory.exists() {
            self.take_step()?;
            fs::create_dir(directory).map_err(|error| file_error("建目录", error))?;
            self.take_step()?;
            sync_directory(directory)?;
            self.take_step()?;
            sync_directory(&parent)?;
        }
        self.known_directories.insert(directory.to_path_buf());
        Ok(())
    }

    fn final_path_of(&self, key: &BlockKey) -> PathBuf {
        self.root.join(block_file_relative_path(key).expect("块键 32 字节"))
    }

    pub(super) fn put_blocks(&mut self, blocks: &[(BlockKey, Vec<u8>)]) -> Result<(), StoreError> {
        match self.write_path {
            WritePath::Registered => self.put_blocks_registered(blocks),
            WritePath::FileLazyRename => self.put_blocks_lazy_rename(blocks),
            WritePath::FileDirectInPlace => self.put_blocks_direct_in_place(blocks),
            WritePath::RedbDurabilityNone | WritePath::RocksDatabaseWithoutWriteAheadLog => {
                panic!("F 臂不用 {} 这种写法", self.write_path.label())
            }
        }
    }

    /// 写临时文件（一次 write_all）→ sync_all；全部写完再逐个改名；再对碰到的每个目录 fsync 一次。
    /// 一块时就是登记的「临时文件 → fsync → 改名 → 目录 fsync」。
    fn put_blocks_registered(&mut self, blocks: &[(BlockKey, Vec<u8>)]) -> Result<(), StoreError> {
        let mut renames = Vec::with_capacity(blocks.len());
        for (key, value) in blocks {
            let final_path = self.final_path_of(key);
            let directory = final_path.parent().expect("块文件有父目录").to_path_buf();
            self.ensure_directory(&directory)?;
            let temporary_path = temporary_path_of(&final_path);
            self.write_synced_temporary_file(&temporary_path, key, value)?;
            renames.push((temporary_path, final_path));
        }
        self.rename_and_sync_directories(&renames)
    }

    fn write_synced_temporary_file(&mut self, temporary_path: &Path, key: &BlockKey, value: &[u8]) -> Result<(), StoreError> {
        let mut contents = Vec::with_capacity(FILE_HEADER_BYTES + value.len());
        contents.extend_from_slice(&block_file_header(key, value));
        contents.extend_from_slice(value);
        self.take_step()?;
        let mut handle = File::create(temporary_path).map_err(|error| file_error("建临时文件", error))?;
        self.take_step()?;
        handle.write_all(&contents).map_err(|error| file_error("写临时文件", error))?;
        self.take_step()?;
        handle.sync_all().map_err(|error| file_error("fsync 临时文件", error))
    }

    fn rename_and_sync_directories(&mut self, renames: &[(PathBuf, PathBuf)]) -> Result<(), StoreError> {
        let mut touched = BTreeSet::new();
        for (temporary_path, final_path) in renames {
            self.take_step()?;
            fs::rename(temporary_path, final_path).map_err(|error| file_error("改名", error))?;
            touched.insert(final_path.parent().expect("块文件有父目录").to_path_buf());
        }
        for directory in touched {
            self.take_step()?;
            sync_directory(&directory)?;
        }
        Ok(())
    }

    fn put_blocks_lazy_rename(&mut self, blocks: &[(BlockKey, Vec<u8>)]) -> Result<(), StoreError> {
        for (key, value) in blocks {
            let final_path = self.final_path_of(key);
            let directory = final_path.parent().expect("块文件有父目录").to_path_buf();
            self.ensure_directory(&directory)?;
            let temporary_path = temporary_path_of(&final_path);
            self.write_synced_temporary_file(&temporary_path, key, value)?;
            self.pending_renames.push((temporary_path, final_path));
            self.confirmations_in_this_run += 1;
            if self.confirmations_in_this_run % LAZY_RENAME_EVERY_CONFIRMATIONS == 0 {
                let pending = std::mem::take(&mut self.pending_renames);
                self.rename_and_sync_directories(&pending)?;
            }
        }
        Ok(())
    }

    fn put_blocks_direct_in_place(&mut self, blocks: &[(BlockKey, Vec<u8>)]) -> Result<(), StoreError> {
        for (key, value) in blocks {
            let final_path = self.final_path_of(key);
            let directory = final_path.parent().expect("块文件有父目录").to_path_buf();
            self.ensure_directory(&directory)?;
            self.take_step()?;
            let mut handle = File::create(&final_path).map_err(|error| file_error("建终名", error))?;
            self.take_step()?;
            handle.write_all(&block_file_header(key, value)).map_err(|error| file_error("写文件头", error))?;
            for chunk in value.chunks(DIRECT_WRITE_CHUNK_BYTES) {
                std::thread::sleep(Duration::from_micros(DIRECT_WRITE_PAUSE_MICROSECONDS));
                self.take_step()?;
                handle.write_all(chunk).map_err(|error| file_error("写块值", error))?;
            }
        }
        Ok(())
    }

    pub(super) fn read_block(&self, key: &BlockKey) -> ReadOutcome {
        match fs::read(self.final_path_of(key)) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => ReadOutcome::NotFound,
            Err(error) => ReadOutcome::ReadError(error.to_string()),
            Ok(bytes) => match validate_block_file(&bytes, key) {
                Ok(value) => ReadOutcome::Found(value),
                Err(reason) => ReadOutcome::ReadError(reason),
            },
        }
    }

    /// 走目录收全部 `.blk` 文件名（`*.tmp` 与别的文件不算）；键从路径拆回，块值长度 = 文件长度 − 48。
    pub(super) fn enumerate_keys_with_value_lengths(&self) -> Result<Vec<(Vec<u8>, u64)>, StoreError> {
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
    if text.len() % 2 != 0 {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|offset| u8::from_str_radix(&text[offset..offset + 2], 16).ok())
        .collect()
}

/// 库目录下全部普通文件的 st_blocks × 512 之和（登记 1.1）；走的时候消失的文件（改名中的临时文件）不算。
pub(super) fn library_occupancy_bytes(directory: &Path) -> u64 {
    use std::os::unix::fs::MetadataExt;
    let Ok(entries) = fs::read_dir(directory) else { return 0 };
    let mut total = 0u64;
    for entry in entries.flatten() {
        let Ok(metadata) = fs::symlink_metadata(entry.path()) else { continue };
        if metadata.is_dir() {
            total += library_occupancy_bytes(&entry.path());
        } else if metadata.is_file() {
            total += metadata.blocks() * 512;
        }
    }
    total
}

// ============================== 逐块核（登记 5.1 的表） ==============================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BlockClass {
    /// 收到过 `C`。
    Confirmed,
    /// 收到过 `B`、没收到过 `C`。
    InFlight,
    /// 没收到过 `B`。
    NotStarted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ReadKind {
    MatchingBytes,
    DifferentBytes,
    NotFound,
    ReadError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BlockVerdict {
    Correct,
    /// Q1a
    LostConfirmed,
    /// Q1b
    SilentlyCorrupted,
    /// Q1c
    ConfirmedReadError,
    /// Q1d
    UnconfirmedReadError,
    /// Q1e
    Phantom,
}

pub(super) fn classify_block(class: BlockClass, read: ReadKind) -> BlockVerdict {
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

/// 核哪些块：一段连续的块号，或一张单子（S1-large 的抽核）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum CheckedBlocks {
    Range { start: u64, end: u64 },
    Listed(Vec<u64>),
}

/// 一次核的计划。写入是顺序的，所以已确认的块是前缀 [0, confirmed_end)，在途的是 [confirmed_end, started_end)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct VerificationPlan {
    pub(super) seed: u64,
    pub(super) pattern: ValuePattern,
    pub(super) states_per_block: u64,
    pub(super) confirmed_end: u64,
    pub(super) started_end: u64,
    pub(super) checked_blocks: CheckedBlocks,
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

    pub(super) fn checked_list(&self) -> Vec<u64> {
        match &self.checked_blocks {
            CheckedBlocks::Range { start, end } => (*start..*end).collect(),
            CheckedBlocks::Listed(blocks) => blocks.clone(),
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
    pub(super) confirmed_checked: u64,
    pub(super) in_flight_checked: u64,
    pub(super) not_started_checked: u64,
    pub(super) lost_confirmed: Vec<u64>,
    pub(super) silently_corrupted: Vec<u64>,
    pub(super) confirmed_read_errors: Vec<u64>,
    pub(super) unconfirmed_read_errors: Vec<u64>,
    /// 读到的未开始块与枚举出来而不属于任何已开始块的键（十六进制），去重。
    pub(super) phantom_keys: BTreeSet<String>,
    pub(super) enumerated_key_count: u64,
    pub(super) enumerated_value_bytes: u64,
    pub(super) enumeration_error: String,
    pub(super) first_read_error: String,
}

/// 打开失败时记下来；K 调一次 DB::repair 再开（登记 5.2），别的臂不修。
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
    for block_index in plan.checked_list() {
        let class = plan.class_of(block_index);
        match class {
            BlockClass::Confirmed => report.confirmed_checked += 1,
            BlockClass::InFlight => report.in_flight_checked += 1,
            BlockClass::NotStarted => report.not_started_checked += 1,
        }
        let key = interleaved_block_key(block_index, plan.states_per_block, &fingerprint);
        let read = match store.read_block(&key) {
            ReadOutcome::Found(found) => {
                let expected = block_value(plan.pattern, plan.seed, block_index, plan.states_per_block);
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
                report.phantom_keys.insert(hexadecimal(&key));
            }
        }
    }
    let started_keys: HashSet<BlockKey> = (0..plan.started_end)
        .map(|block_index| interleaved_block_key(block_index, plan.states_per_block, &fingerprint))
        .collect();
    let enumerated = store.enumerate_keys_with_value_lengths();
    match enumerated {
        Ok(entries) => {
            for (key, value_length) in entries {
                report.enumerated_key_count += 1;
                report.enumerated_value_bytes += value_length;
                let is_started = <[u8; BLOCK_KEY_BYTES]>::try_from(key.as_slice())
                    .is_ok_and(|fixed_key| started_keys.contains(&fixed_key));
                if !is_started {
                    report.phantom_keys.insert(hexadecimal(&key));
                }
            }
        }
        Err(error) => report.enumeration_error = error.to_string(),
    }
    report
}

pub(super) fn elapsed_microseconds(since: Instant) -> u64 {
    u64::try_from(since.elapsed().as_micros()).expect("微秒数装得进 u64")
}

pub(super) fn join_numbers(numbers: &[u64]) -> String {
    if numbers.is_empty() {
        "-".to_string()
    } else {
        numbers.iter().map(u64::to_string).collect::<Vec<_>>().join(",")
    }
}

pub(super) fn split_numbers(text: &str) -> Vec<u64> {
    if text == "-" {
        Vec::new()
    } else {
        text.split(',').map(|number| number.parse().expect("逗号隔开的整数")).collect()
    }
}

/// 核对报告编成一行 `VERIFY k=v …`（核对子进程写，父进程读回）；文本字段里的空白换成下划线。
pub(super) fn encode_verification_report(report: &VerificationReport) -> String {
    let phantom: Vec<String> = report.phantom_keys.iter().cloned().collect();
    let text = |value: &str| if value.is_empty() { "-".to_string() } else { value.split_whitespace().collect::<Vec<_>>().join("_") };
    let repaired = match report.opened_after_repair {
        None => "not_applicable",
        Some(true) => "yes",
        Some(false) => "no",
    };
    format!(
        "VERIFY open_failed={} opened_after_repair={repaired} open_microseconds={} confirmed_checked={} in_flight_checked={} not_started_checked={} lost={} corrupted={} confirmed_read_errors={} unconfirmed_read_errors={} phantom={} enumerated_keys={} enumerated_value_bytes={} enumeration_error={} first_read_error={} open_error={}",
        report.open_failed,
        report.open_microseconds,
        report.confirmed_checked,
        report.in_flight_checked,
        report.not_started_checked,
        join_numbers(&report.lost_confirmed),
        join_numbers(&report.silently_corrupted),
        join_numbers(&report.confirmed_read_errors),
        join_numbers(&report.unconfirmed_read_errors),
        if phantom.is_empty() { "-".to_string() } else { phantom.join(",") },
        report.enumerated_key_count,
        report.enumerated_value_bytes,
        text(&report.enumeration_error),
        text(&report.first_read_error),
        text(&report.open_error),
    )
}

pub(super) fn decode_verification_report(line: &str) -> Option<VerificationReport> {
    let body = line.strip_prefix("VERIFY ")?;
    let fields: BTreeMap<&str, &str> = body.split(' ').filter_map(|field| field.split_once('=')).collect();
    let untext = |value: &str| if value == "-" { String::new() } else { value.to_string() };
    Some(VerificationReport {
        open_failed: fields.get("open_failed")? == &"true",
        open_error: untext(fields.get("open_error")?),
        opened_after_repair: match *fields.get("opened_after_repair")? {
            "yes" => Some(true),
            "no" => Some(false),
            _ => None,
        },
        open_microseconds: fields.get("open_microseconds")?.parse().ok()?,
        confirmed_checked: fields.get("confirmed_checked")?.parse().ok()?,
        in_flight_checked: fields.get("in_flight_checked")?.parse().ok()?,
        not_started_checked: fields.get("not_started_checked")?.parse().ok()?,
        lost_confirmed: split_numbers(fields.get("lost")?),
        silently_corrupted: split_numbers(fields.get("corrupted")?),
        confirmed_read_errors: split_numbers(fields.get("confirmed_read_errors")?),
        unconfirmed_read_errors: split_numbers(fields.get("unconfirmed_read_errors")?),
        phantom_keys: match *fields.get("phantom")? {
            "-" => BTreeSet::new(),
            listed => listed.split(',').map(str::to_string).collect(),
        },
        enumerated_key_count: fields.get("enumerated_keys")?.parse().ok()?,
        enumerated_value_bytes: fields.get("enumerated_value_bytes")?.parse().ok()?,
        enumeration_error: untext(fields.get("enumeration_error")?),
        first_read_error: untext(fields.get("first_read_error")?),
    })
}
