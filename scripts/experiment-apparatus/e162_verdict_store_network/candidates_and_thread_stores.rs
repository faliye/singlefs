//! 候选与存储错误、读结局、两种库的选项、每线程一份的存储与它的读写。

use super::*;

// ============================== 候选（登记 5.2，从 S1 装置拷来） ==============================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Candidate {
    /// R1：redb，每个写事务 Durability::Immediate + quick-repair。
    RedbQuickRepair,
    /// R0：redb，Durability::Immediate，quick-repair 关（默认）。
    RedbDefault,
    /// K：RocksDB，LZ4，同步写、WAL 开。
    RocksDatabase,
    /// F：加固的文件，一块一个文件，临时文件 → fsync → 改名 → 目录 fsync。
    HardenedFile,
}

impl Candidate {
    pub(super) fn label(self) -> &'static str {
        match self {
            Candidate::RedbQuickRepair => "R1",
            Candidate::RedbDefault => "R0",
            Candidate::RocksDatabase => "K",
            Candidate::HardenedFile => "F",
        }
    }

    pub(super) fn parse(text: &str) -> Option<Candidate> {
        match text {
            "R1" => Some(Candidate::RedbQuickRepair),
            "R0" => Some(Candidate::RedbDefault),
            "K" => Some(Candidate::RocksDatabase),
            "F" => Some(Candidate::HardenedFile),
            _ => None,
        }
    }
}

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

pub(super) fn rocksdb_error(error: impl std::fmt::Display) -> StoreError {
    StoreError::RocksDatabase(error.to_string())
}

pub(super) fn file_error(context: &str, error: impl std::fmt::Display) -> StoreError {
    StoreError::File(format!("{context}: {error}"))
}

/// 读一块的三种结局（登记 5.1 的三列）。
#[derive(Debug, PartialEq, Eq)]
pub(super) enum ReadOutcome {
    Found(Vec<u8>),
    NotFound,
    ReadError(String),
}

pub(super) fn rocksdb_options() -> rocksdb::Options {
    let mut options = rocksdb::Options::default();
    options.create_if_missing(true);
    options.set_compression_type(rocksdb::DBCompressionType::Lz4);
    options
}

/// K 的写选项：同步写、WAL 开（登记 5.2）。
fn rocksdb_write_options() -> rocksdb::WriteOptions {
    let mut write_options = rocksdb::WriteOptions::default();
    write_options.disable_wal(false);
    write_options.set_sync(true);
    write_options
}

/// 一个线程手里的库：redb 与 RocksDB 两个线程共用一个打开的库（redb 的写事务由它自己排队、
/// RocksDB 两个线程并发 `put_opt`），F 每个线程各开一份、各写各的文件（登记 6.3 末段）。
enum StoreHandle {
    Redb(Arc<redb::Database>),
    RocksDatabase(Arc<rocksdb::DB>),
    File(FileLibrary),
}

pub(super) struct ThreadStore {
    pub(super) candidate: Candidate,
    handle: StoreHandle,
}

/// 写入用的打开：库不在就建；返回 thread_count 个共用同一个库的句柄。
pub(super) fn open_thread_stores(candidate: Candidate, library: &Path, thread_count: usize) -> Result<Vec<ThreadStore>, StoreError> {
    let mut stores = Vec::with_capacity(thread_count);
    match candidate {
        Candidate::RedbQuickRepair | Candidate::RedbDefault => {
            fs::create_dir_all(library).map_err(|error| file_error("建 redb 库目录", error))?;
            let database = Arc::new(redb::Database::create(library.join(REDB_FILE_NAME)).map_err(redb_error)?);
            for _thread in 0..thread_count {
                stores.push(ThreadStore { candidate, handle: StoreHandle::Redb(Arc::clone(&database)) });
            }
        }
        Candidate::RocksDatabase => {
            let database = Arc::new(rocksdb::DB::open(&rocksdb_options(), library).map_err(rocksdb_error)?);
            for _thread in 0..thread_count {
                stores.push(ThreadStore { candidate, handle: StoreHandle::RocksDatabase(Arc::clone(&database)) });
            }
        }
        Candidate::HardenedFile => {
            for _thread in 0..thread_count {
                stores.push(ThreadStore { candidate, handle: StoreHandle::File(FileLibrary::open(library, true)?) });
            }
        }
    }
    Ok(stores)
}

/// 核对用的打开：同一个打开调用（redb 的 `Database::create`、RocksDB 的 `DB::open`），F 不建新库。
pub(super) fn open_existing_store(candidate: Candidate, library: &Path) -> Result<ThreadStore, StoreError> {
    let handle = match candidate {
        Candidate::RedbQuickRepair | Candidate::RedbDefault => {
            StoreHandle::Redb(Arc::new(redb::Database::create(library.join(REDB_FILE_NAME)).map_err(redb_error)?))
        }
        Candidate::RocksDatabase => StoreHandle::RocksDatabase(Arc::new(rocksdb::DB::open(&rocksdb_options(), library).map_err(rocksdb_error)?)),
        Candidate::HardenedFile => StoreHandle::File(FileLibrary::open(library, false)?),
    };
    Ok(ThreadStore { candidate, handle })
}

impl ThreadStore {
    /// 一次持久提交一块；返回 Ok 就算这一块已确认。
    pub(super) fn put_block(&mut self, key: &BlockKey, value: &[u8]) -> Result<(), StoreError> {
        match &mut self.handle {
            StoreHandle::Redb(database) => {
                let quick_repair = self.candidate == Candidate::RedbQuickRepair;
                let mut transaction = database.begin_write().map_err(redb_error)?;
                transaction.set_durability(redb::Durability::Immediate).map_err(redb_error)?;
                transaction.set_quick_repair(quick_repair);
                {
                    let mut table = transaction.open_table(REDB_TABLE).map_err(redb_error)?;
                    table.insert(&key[..], value).map_err(redb_error)?;
                }
                transaction.commit().map_err(redb_error)
            }
            StoreHandle::RocksDatabase(database) => database.put_opt(key, value, &rocksdb_write_options()).map_err(rocksdb_error),
            StoreHandle::File(library) => library.put_block(key, value),
        }
    }

    pub(super) fn read_block(&self, key: &BlockKey) -> ReadOutcome {
        match &self.handle {
            StoreHandle::Redb(database) => {
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
            StoreHandle::RocksDatabase(database) => match database.get(key) {
                Ok(Some(value)) => ReadOutcome::Found(value),
                Ok(None) => ReadOutcome::NotFound,
                Err(error) => ReadOutcome::ReadError(error.to_string()),
            },
            StoreHandle::File(library) => library.read_block(key),
        }
    }

    /// 枚举库里的全部键与各自块值的长度（F 按文件名与文件长度，不读内容）。
    pub(super) fn enumerate_keys_with_value_lengths(&self) -> Result<Vec<(Vec<u8>, u64)>, StoreError> {
        match &self.handle {
            StoreHandle::Redb(database) => {
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
            StoreHandle::RocksDatabase(database) => {
                let mut entries = Vec::new();
                for item in database.iterator(rocksdb::IteratorMode::Start) {
                    let (key, value) = item.map_err(rocksdb_error)?;
                    entries.push((key.to_vec(), u64::try_from(value.len()).expect("块值长度装得进 u64")));
                }
                Ok(entries)
            }
            StoreHandle::File(library) => library.enumerate_keys_with_value_lengths(),
        }
    }
}

#[cfg(test)]
#[path = "candidates_and_thread_stores_tests.rs"]
mod tests;
