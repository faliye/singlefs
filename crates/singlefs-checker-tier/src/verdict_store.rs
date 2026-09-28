//! 崩溃放量的判定存储（里程碑三第十一项第一段，`.claude/kb/milestone/03-third-txn.md`「十一」）：
//! 按块存每个崩溃状态的判定编号，另存不同的判定向量与违例。库是 RocksDB（用户 2026-09-28 选定），
//! 每次写都同步落盘、WAL 开着（E162（崩溃放量判定块存储选型） K 臂的写法）；打不开时调一次 `DB::repair` 再开。
//! 这一段只有存储本身，还没接进层 0（`crash`、`layer0_progress`），接法归第二轮三方。
//!
//! 四张表都放在默认列族里、靠键的第一个字节（表标签）分开，不用列族：`DB::repair` 经 C 接口走的是
//! `RepairDB(dbname, options)`（`librocksdb-sys` 里 `rocksdb/db/repair.cc` 的 `RepairDB(const std::string&, const Options&)`），
//! 它丢掉旧的 MANIFEST、新建一份只有默认列族的，别的列族只从 SST 文件的元数据里认回来；还只在 WAL 里、没刷成 SST 的
//! 记录落在认不回来的列族上，转表时整条被跳过。判定块刚写完多半只在 WAL 里，用列族的话一次修库就丢掉它们。
//! 放在同一个列族里，一个 `WriteBatch` 照样把块与它的违例一起原子地写下去。
//!
//! 键的字节布局（整数一律大端，同一张表里按键排序就是按块起点、块内序号排序）：
//!
//! | 表 | 键 | 值 |
//! |---|---|---|
//! | 判定块 | `01` ‖ 块键 | 块里每个状态一个字节：判定向量编号 |
//! | 判定向量按内容 | `02` ‖ 判定向量的字节 | 编号，一个字节 |
//! | 判定向量按编号 | `03` ‖ 编号，一个字节 | 判定向量的字节 |
//! | 违例 | `04` ‖ 块键 ‖ 块内序号（8 字节） | 那个状态的判定向量编号，一个字节 |
//!
//! 块键 = 输入指纹（32 字节）‖ 流名或节点名的字节数（2 字节）‖ 流名或节点名 ‖ 枚举计划哈希（32 字节）‖ 块起点（8 字节）。
//! 名字前面带字节数，一个块键不会是另一个块键的前缀，按块键前缀扫违例表只扫到这一块的。

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

/// 输入指纹与枚举计划哈希都是 SHA-256 摘要的字节数。
pub const SHA256_DIGEST_BYTES: usize = 32;

/// 违例表的键里块内序号占几个字节。
const STATE_OFFSET_IN_BLOCK_BYTES: usize = std::mem::size_of::<u64>();

/// 这一份判定是拿哪一份输入算出来的：门禁 54 号给每条崩溃枚举用例算的输入指纹（`research/scripts/admission.py`，SHA-256）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct InputFingerprint(pub [u8; SHA256_DIGEST_BYTES]);

/// 枚举计划哈希（`crash` 模块现算的计划哈希，SHA-256）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EnumerationPlanHash(pub [u8; SHA256_DIGEST_BYTES]);

/// 块起点：这一块第一个状态在它那份枚举计划里的序号。块长由调用方定，不进键。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlockStartStateIndex(pub u64);

/// 状态在块里的序号，从 0 起。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StateOffsetInBlock(pub u64);

/// 流名（里程碑三以后是节点名）：非空，不超过 `u16::MAX` 字节（键里用两个字节记它的长度）。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StreamOrNodeName(String);

/// 给的名字是空的，或超过 `u16::MAX` 字节。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StreamOrNodeNameRejected {
    pub text: String,
}

impl StreamOrNodeName {
    /// # Errors
    /// 空，或超过 `u16::MAX` 字节。
    pub fn new(text: &str) -> Result<Self, StreamOrNodeNameRejected> {
        if text.is_empty() || u16::try_from(text.len()).is_err() {
            return Err(StreamOrNodeNameRejected {
                text: text.to_string(),
            });
        }
        Ok(Self(text.to_string()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 核对表里的一行：这一块用哪份事实表核的、每个状态的结论位。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifierBlockRow {
    pub facts_digest: [u8; 32],
    pub bits: Vec<u32>,
}

/// 一张表的原始键值行（导入时逐行照抄）。
type RawRows = Vec<(Vec<u8>, Vec<u8>)>;

/// 判定块的键。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VerdictBlockKey {
    pub input_fingerprint: InputFingerprint,
    pub stream_or_node_name: StreamOrNodeName,
    pub enumeration_plan_hash: EnumerationPlanHash,
    pub block_start: BlockStartStateIndex,
}

impl VerdictBlockKey {
    fn to_block_key_bytes(&self) -> Vec<u8> {
        let name_bytes = self.stream_or_node_name.as_str().as_bytes();
        let name_length = u16::try_from(name_bytes.len())
            .expect("StreamOrNodeName::new 只收不超过 u16::MAX 字节的名字");
        let mut key_bytes = Vec::new();
        key_bytes.extend_from_slice(&self.input_fingerprint.0);
        key_bytes.extend_from_slice(&name_length.to_be_bytes());
        key_bytes.extend_from_slice(name_bytes);
        key_bytes.extend_from_slice(&self.enumeration_plan_hash.0);
        key_bytes.extend_from_slice(&self.block_start.0.to_be_bytes());
        key_bytes
    }

    /// [`Self::to_block_key_bytes`] 的逆：列表、导入时从键字节认回块键；长度或名字对不上交回 None。
    fn from_block_key_bytes(bytes: &[u8]) -> Option<Self> {
        let fingerprint =
            <[u8; SHA256_DIGEST_BYTES]>::try_from(bytes.get(..SHA256_DIGEST_BYTES)?).ok()?;
        let name_length_bytes =
            <[u8; 2]>::try_from(bytes.get(SHA256_DIGEST_BYTES..SHA256_DIGEST_BYTES + 2)?).ok()?;
        let name_start = SHA256_DIGEST_BYTES + 2;
        let name_end = name_start + usize::from(u16::from_be_bytes(name_length_bytes));
        let name = std::str::from_utf8(bytes.get(name_start..name_end)?).ok()?;
        let plan_hash = <[u8; SHA256_DIGEST_BYTES]>::try_from(
            bytes.get(name_end..name_end + SHA256_DIGEST_BYTES)?,
        )
        .ok()?;
        let start_bytes = <[u8; 8]>::try_from(bytes.get(name_end + SHA256_DIGEST_BYTES..)?).ok()?;
        Some(Self {
            input_fingerprint: InputFingerprint(fingerprint),
            stream_or_node_name: StreamOrNodeName::new(name).ok()?,
            enumeration_plan_hash: EnumerationPlanHash(plan_hash),
            block_start: BlockStartStateIndex(u64::from_be_bytes(start_bytes)),
        })
    }
}

/// 一个状态的判定向量，按调用方定的编码存成字节（这个模块不解释它）；两份字节相同就是同一个判定向量。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EncodedVerdictVector(pub Vec<u8>);

/// 判定向量的编号：同一个库里一种判定向量一个编号，从 0 起按登记次序给，登记过就不变（关库重开也不变）。
/// 只能从 [`VerdictStore::register_verdict_vector`] 或读回的块拿到。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VerdictVectorNumber(u8);

impl VerdictVectorNumber {
    #[must_use]
    pub fn as_u8(self) -> u8 {
        self.0
    }
}

/// 一块里每个状态的判定向量编号，按块内序号排；至少一个状态。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerdictBlock {
    verdict_vector_number_of_each_state: Vec<VerdictVectorNumber>,
}

/// 一个状态都没有的块。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EmptyVerdictBlockRejected;

impl VerdictBlock {
    /// # Errors
    /// 一个状态都没有。
    pub fn new(
        verdict_vector_number_of_each_state: Vec<VerdictVectorNumber>,
    ) -> Result<Self, EmptyVerdictBlockRejected> {
        if verdict_vector_number_of_each_state.is_empty() {
            return Err(EmptyVerdictBlockRejected);
        }
        Ok(Self {
            verdict_vector_number_of_each_state,
        })
    }

    #[must_use]
    pub fn verdict_vector_number_of_each_state(&self) -> &[VerdictVectorNumber] {
        &self.verdict_vector_number_of_each_state
    }

    fn verdict_vector_number_at(&self, offset: StateOffsetInBlock) -> Option<VerdictVectorNumber> {
        let index = usize::try_from(offset.0).ok()?;
        self.verdict_vector_number_of_each_state.get(index).copied()
    }

    fn to_block_value_bytes(&self) -> Vec<u8> {
        self.verdict_vector_number_of_each_state
            .iter()
            .map(|number| number.0)
            .collect()
    }
}

/// 违例表里的一行：块里第几个状态违例、它的判定向量编号。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StoredViolation {
    pub offset_in_block: StateOffsetInBlock,
    pub verdict_vector_number: VerdictVectorNumber,
}

/// 库里的四张表；键的第一个字节是表标签。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerdictStoreTable {
    VerdictBlocks,
    VerdictVectorNumberByContent,
    VerdictVectorContentByNumber,
    Violations,
    /// 崩溃放量「先录后核」的录入表：一块一行，值是这一块要核的状态数（8 字节大端）。核过的块在判定块表里有同一个块键。
    RecordedBlocks,
    /// 崩溃放量的路径索引：键是节点的路径（`<流程>.<周期>.<步>/…`，UTF-8 原文），值是它的身份（`<仓内路径>::<名>::<复用键>`）。
    /// 键按字典序排，按前缀扫就是列子树；块键里不带路径，改名只重写这张表。
    PathIndex,
    /// 三段流第 ① 段的事实表（`crash_facts::FlowFacts`）：键是事实表摘要（连抽取器版本），值是序列化的事实。换了抽取规则就是新键，旧行留着。
    FlowFacts,
    /// 三段流第 ② 段的核对表：键是块键字节 + 核对代码摘要，值是事实表摘要 32 字节 + 每个状态一个 u32 结论位（小端）。
    /// 与判定块表分开：判定块是 CPU 判器写的，这张是 CPU 参照 / GPU 核对写的，第 ③ 段逐状态比。
    VerifierVerdicts,
}

impl VerdictStoreTable {
    fn key_tag(self) -> u8 {
        match self {
            Self::VerdictBlocks => 0x01,
            Self::VerdictVectorNumberByContent => 0x02,
            Self::VerdictVectorContentByNumber => 0x03,
            Self::Violations => 0x04,
            Self::RecordedBlocks => 0x05,
            Self::PathIndex => 0x06,
            Self::FlowFacts => 0x07,
            Self::VerifierVerdicts => 0x08,
        }
    }
}

/// 库里的一个键，或按前缀扫一段键时的前缀。键的字节只在 [`VerdictStoreKey::to_key_bytes`] 一处拼。
enum VerdictStoreKey<'key> {
    VerdictBlock(&'key VerdictBlockKey),
    VerdictVectorNumberByContent(&'key EncodedVerdictVector),
    VerdictVectorContentByNumber(VerdictVectorNumber),
    Violation(&'key VerdictBlockKey, StateOffsetInBlock),
    /// 违例表里这一块的全部键共有的前缀。
    ViolationsOfBlockPrefix(&'key VerdictBlockKey),
    /// 一张表全部键共有的前缀（只有表标签）。
    WholeTablePrefix(VerdictStoreTable),
    RecordedBlock(&'key VerdictBlockKey),
    /// 路径索引表里一条路径（或一段前缀）的键。
    Path(&'key str),
    /// 事实表里一条流的键：事实表摘要。
    FlowFacts(&'key [u8; 32]),
    /// 核对表里一块的键：块键 + 核对代码摘要。
    VerifierBlock(&'key VerdictBlockKey, &'key [u8; 32]),
}

/// 按 [`VerdictStoreKey`] 拼出来的键字节。
#[derive(Clone, Debug, PartialEq, Eq)]
struct VerdictStoreKeyBytes(Vec<u8>);

impl VerdictStoreKey<'_> {
    fn to_key_bytes(&self) -> VerdictStoreKeyBytes {
        let (table, key_bytes_after_tag) = match self {
            Self::VerdictBlock(block_key) => (
                VerdictStoreTable::VerdictBlocks,
                block_key.to_block_key_bytes(),
            ),
            Self::ViolationsOfBlockPrefix(block_key) => (
                VerdictStoreTable::Violations,
                block_key.to_block_key_bytes(),
            ),
            Self::VerdictVectorNumberByContent(verdict_vector) => (
                VerdictStoreTable::VerdictVectorNumberByContent,
                verdict_vector.0.clone(),
            ),
            Self::VerdictVectorContentByNumber(number) => (
                VerdictStoreTable::VerdictVectorContentByNumber,
                vec![number.0],
            ),
            Self::Violation(block_key, offset) => {
                let mut key_bytes_after_tag = block_key.to_block_key_bytes();
                key_bytes_after_tag.extend_from_slice(&offset.0.to_be_bytes());
                (VerdictStoreTable::Violations, key_bytes_after_tag)
            }
            Self::WholeTablePrefix(table) => (*table, Vec::new()),
            Self::RecordedBlock(block_key) => (
                VerdictStoreTable::RecordedBlocks,
                block_key.to_block_key_bytes(),
            ),
            Self::Path(path) => (VerdictStoreTable::PathIndex, path.as_bytes().to_vec()),
            Self::FlowFacts(digest) => (VerdictStoreTable::FlowFacts, digest.to_vec()),
            Self::VerifierBlock(block_key, verifier_digest) => {
                let mut key_bytes_after_tag = block_key.to_block_key_bytes();
                key_bytes_after_tag.extend_from_slice(&verifier_digest[..]);
                (VerdictStoreTable::VerifierVerdicts, key_bytes_after_tag)
            }
        };
        let mut key_bytes = Vec::with_capacity(1 + key_bytes_after_tag.len());
        key_bytes.push(table.key_tag());
        key_bytes.extend_from_slice(&key_bytes_after_tag);
        VerdictStoreKeyBytes(key_bytes)
    }
}

/// 库是怎么打开的。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VerdictStoreOpening {
    OpenedAtTheFirstAttempt,
    /// 第一次打开失败，调了一次 `DB::repair` 之后打开的。RocksDB 修完自己记的是「可能丢了一些数据」
    /// （`repair.cc` 的 `Repairer::Run` 修完打的那一行），调用方按这一格决定这个库里的块还信不信。
    OpenedAfterOneRepair {
        first_open_error: rocksdb::Error,
    },
}

/// 一次存块的结局。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerdictBlockStoring {
    /// 块与它的违例同一批写下去，同步落盘之后才返回。
    WrittenAndSynced,
    /// 这个块键下已经存着逐字节相同的块与相同的违例，什么都没写（续跑时重写同一块走这一格）。
    AlreadyStoredIdentically,
}

/// 存储出的错，按调用方要做的决定分。
#[derive(Debug)]
pub enum VerdictStoreError {
    /// `create_empty`：目录已经在了；不在已有的目录上建库，免得把旧库当成空库。
    LibraryDirectoryAlreadyExists { directory: PathBuf },
    /// `open_existing`：目录不在；不就地建一个空库。
    LibraryDirectoryMissing { directory: PathBuf },
    /// 看不了目录在不在（权限、I/O）。
    LibraryDirectoryUninspectable {
        directory: PathBuf,
        source: std::io::Error,
    },
    /// 第一次打开失败，调一次 `DB::repair` 之后还是打不开（或修本身失败）。
    UnopenableEvenAfterOneRepair {
        first_open_error: rocksdb::Error,
        repair_or_reopen_error: rocksdb::Error,
    },
    /// 库里存的字节不合这个模块的布局：块值是空的或点名了没登记的编号、两张判定向量表对不上、键或值的长度不对。
    StoredBytesDoNotMatchTheLayout {
        table: VerdictStoreTable,
        key_bytes: Vec<u8>,
        reason: &'static str,
    },
    /// RocksDB 读写报的错（I/O、校验和不对……）。
    RocksDatabase(rocksdb::Error),
    /// 第 257 种判定向量：编号一个字节，用满之后怎么办（加宽编号、换一种块值编码）没定，第一版不支持；返回之前什么都没写。
    MoreThan256VerdictVectorsNotDecidedInTheFirstVersion,
    /// 这个块键下已经存着不同的块或不同的违例：同一份输入、同一份计划下同一块判出两样，是覆盖、拒绝还是另记，没定，
    /// 第一版不支持；返回之前什么都没写。
    DifferentVerdictsUnderAStoredBlockKeyNotDecidedInTheFirstVersion { block_key: VerdictBlockKey },
}

/// 建库还是开已有的库。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LibraryCreation {
    CreateEmpty,
    OpenExisting,
}

fn library_options(creation: LibraryCreation) -> rocksdb::Options {
    let mut options = rocksdb::Options::default();
    match creation {
        LibraryCreation::CreateEmpty => {
            options.create_if_missing(true);
            options.set_error_if_exists(true);
        }
        LibraryCreation::OpenExisting => options.create_if_missing(false),
    }
    options.set_compression_type(rocksdb::DBCompressionType::Lz4);
    options
}

/// 每一批都经 WAL 并同步落盘之后才返回（E162（崩溃放量判定块存储选型） K 臂：`set_sync(true)`、WAL 不关）。
fn synced_write_options() -> rocksdb::WriteOptions {
    let mut write_options = rocksdb::WriteOptions::default();
    write_options.disable_wal(false);
    write_options.set_sync(true);
    write_options
}

fn library_directory_exists(directory: &Path) -> Result<bool, VerdictStoreError> {
    directory
        .try_exists()
        .map_err(|source| VerdictStoreError::LibraryDirectoryUninspectable {
            directory: directory.to_path_buf(),
            source,
        })
}

type StoredEntry = (Box<[u8]>, Box<[u8]>);

/// 键以 `prefix` 开头的全部条目，按键的次序。
fn entries_with_key_prefix(
    database: &rocksdb::DB,
    prefix: VerdictStoreKeyBytes,
) -> impl Iterator<Item = Result<StoredEntry, rocksdb::Error>> + '_ {
    database
        .iterator(rocksdb::IteratorMode::From(
            &prefix.0,
            rocksdb::Direction::Forward,
        ))
        .take_while(move |entry| match entry {
            Ok((key_bytes, _)) => key_bytes.starts_with(&prefix.0),
            Err(_) => true,
        })
}

/// 两张判定向量表读回来、互相核过之后的样子。
struct LoadedVerdictVectors {
    verdict_vector_by_number: Vec<EncodedVerdictVector>,
    verdict_vector_number_by_content: HashMap<Vec<u8>, VerdictVectorNumber>,
}

fn stored_bytes_mismatch(
    table: VerdictStoreTable,
    key_bytes: &[u8],
    reason: &'static str,
) -> VerdictStoreError {
    VerdictStoreError::StoredBytesDoNotMatchTheLayout {
        table,
        key_bytes: key_bytes.to_vec(),
        reason,
    }
}

/// 读回两张判定向量表：按编号那张的编号必须从 0 起连续，按内容那张与它一一对应。
fn load_verdict_vectors(database: &rocksdb::DB) -> Result<LoadedVerdictVectors, VerdictStoreError> {
    let mut verdict_vector_by_number = Vec::new();
    let by_number_table = VerdictStoreTable::VerdictVectorContentByNumber;
    for entry in entries_with_key_prefix(
        database,
        VerdictStoreKey::WholeTablePrefix(by_number_table).to_key_bytes(),
    ) {
        let (key_bytes, vector_bytes) = entry.map_err(VerdictStoreError::RocksDatabase)?;
        let is_the_next_number = match *key_bytes {
            [_, number] => usize::from(number) == verdict_vector_by_number.len(),
            _ => false,
        };
        if !is_the_next_number {
            return Err(stored_bytes_mismatch(
                by_number_table,
                &key_bytes,
                "按编号的判定向量表里编号不是从 0 起连续的一个字节",
            ));
        }
        verdict_vector_by_number.push(EncodedVerdictVector(vector_bytes.to_vec()));
    }
    let mut verdict_vector_number_by_content = HashMap::new();
    let by_content_table = VerdictStoreTable::VerdictVectorNumberByContent;
    for entry in entries_with_key_prefix(
        database,
        VerdictStoreKey::WholeTablePrefix(by_content_table).to_key_bytes(),
    ) {
        let (key_bytes, number_bytes) = entry.map_err(VerdictStoreError::RocksDatabase)?;
        let content = &key_bytes[1..];
        let number = match *number_bytes {
            [number] => number,
            _ => {
                return Err(stored_bytes_mismatch(
                    by_content_table,
                    &key_bytes,
                    "按内容的判定向量表里值不是一个字节",
                ))
            }
        };
        let agrees_with_the_number_table = verdict_vector_by_number
            .get(usize::from(number))
            .is_some_and(|verdict_vector| verdict_vector.0 == content);
        if !agrees_with_the_number_table {
            return Err(stored_bytes_mismatch(
                by_content_table,
                &key_bytes,
                "按内容的判定向量表与按编号的那张对不上",
            ));
        }
        verdict_vector_number_by_content.insert(content.to_vec(), VerdictVectorNumber(number));
    }
    if verdict_vector_number_by_content.len() != verdict_vector_by_number.len() {
        return Err(stored_bytes_mismatch(
            by_content_table,
            &[by_content_table.key_tag()],
            "按内容的判定向量表比按编号的那张少了几行",
        ));
    }
    Ok(LoadedVerdictVectors {
        verdict_vector_by_number,
        verdict_vector_number_by_content,
    })
}

/// 一个打开着的判定库。同一时刻一个库只许一个进程开（RocksDB 的 LOCK 文件），进程里改库的方法都要 `&mut self`，
/// 所以「先读后写」的两步之间没有别的写者。
pub struct VerdictStore {
    database: rocksdb::DB,
    verdict_vector_by_number: Vec<EncodedVerdictVector>,
    verdict_vector_number_by_content: HashMap<Vec<u8>, VerdictVectorNumber>,
}

impl VerdictStore {
    /// 在一个还不存在的目录上建一个空库。
    ///
    /// # Errors
    /// 目录已经在了（`LibraryDirectoryAlreadyExists`）、看不了目录在不在、RocksDB 建库失败。
    pub fn create_empty(directory: &Path) -> Result<Self, VerdictStoreError> {
        if library_directory_exists(directory)? {
            return Err(VerdictStoreError::LibraryDirectoryAlreadyExists {
                directory: directory.to_path_buf(),
            });
        }
        let database = rocksdb::DB::open(&library_options(LibraryCreation::CreateEmpty), directory)
            .map_err(VerdictStoreError::RocksDatabase)?;
        Self::from_opened_database(database)
    }

    /// 开一个已有的库；第一次打不开时调一次 `DB::repair` 再开。
    ///
    /// # Errors
    /// 目录不在（`LibraryDirectoryMissing`，不修、不建）、看不了目录在不在、修过一次还打不开、两张判定向量表读回来对不上。
    pub fn open_existing(
        directory: &Path,
    ) -> Result<(Self, VerdictStoreOpening), VerdictStoreError> {
        if !library_directory_exists(directory)? {
            return Err(VerdictStoreError::LibraryDirectoryMissing {
                directory: directory.to_path_buf(),
            });
        }
        let options = library_options(LibraryCreation::OpenExisting);
        let (database, opening) = match rocksdb::DB::open(&options, directory) {
            Ok(database) => (database, VerdictStoreOpening::OpenedAtTheFirstAttempt),
            Err(first_open_error) => {
                let reopened = rocksdb::DB::repair(&options, directory)
                    .and_then(|()| rocksdb::DB::open(&options, directory));
                match reopened {
                    Ok(database) => (
                        database,
                        VerdictStoreOpening::OpenedAfterOneRepair { first_open_error },
                    ),
                    Err(repair_or_reopen_error) => {
                        return Err(VerdictStoreError::UnopenableEvenAfterOneRepair {
                            first_open_error,
                            repair_or_reopen_error,
                        })
                    }
                }
            }
        };
        Ok((Self::from_opened_database(database)?, opening))
    }

    fn from_opened_database(database: rocksdb::DB) -> Result<Self, VerdictStoreError> {
        let LoadedVerdictVectors {
            verdict_vector_by_number,
            verdict_vector_number_by_content,
        } = load_verdict_vectors(&database)?;
        Ok(Self {
            database,
            verdict_vector_by_number,
            verdict_vector_number_by_content,
        })
    }

    /// 给一种判定向量一个编号：登记过的返回原来的编号、什么都不写；没登记过的给下一个编号，两张表同一批同步写下去之后才返回。
    ///
    /// # Errors
    /// 已经有 256 种、这是第 257 种（`MoreThan256VerdictVectorsNotDecidedInTheFirstVersion`，什么都没写）；RocksDB 写失败。
    pub fn register_verdict_vector(
        &mut self,
        verdict_vector: &EncodedVerdictVector,
    ) -> Result<VerdictVectorNumber, VerdictStoreError> {
        if let Some(number) = self.verdict_vector_number_by_content.get(&verdict_vector.0) {
            return Ok(*number);
        }
        let Ok(next_number) = u8::try_from(self.verdict_vector_by_number.len()) else {
            return Err(VerdictStoreError::MoreThan256VerdictVectorsNotDecidedInTheFirstVersion);
        };
        let number = VerdictVectorNumber(next_number);
        let mut batch = rocksdb::WriteBatch::default();
        batch.put(
            VerdictStoreKey::VerdictVectorNumberByContent(verdict_vector)
                .to_key_bytes()
                .0,
            [number.0],
        );
        batch.put(
            VerdictStoreKey::VerdictVectorContentByNumber(number)
                .to_key_bytes()
                .0,
            &verdict_vector.0,
        );
        self.database
            .write_opt(batch, &synced_write_options())
            .map_err(VerdictStoreError::RocksDatabase)?;
        self.verdict_vector_by_number.push(verdict_vector.clone());
        self.verdict_vector_number_by_content
            .insert(verdict_vector.0.clone(), number);
        Ok(number)
    }

    /// 这个编号登记的是哪种判定向量；这个库里没有这个编号时是 `None`。
    #[must_use]
    pub fn verdict_vector_of_number(
        &self,
        number: VerdictVectorNumber,
    ) -> Option<&EncodedVerdictVector> {
        self.verdict_vector_by_number.get(usize::from(number.0))
    }

    /// 存一块与它的违例：块与每个违例状态同一批同步写下去之后才返回；违例状态的判定向量编号取块里那个状态的。
    ///
    /// # Errors
    /// 这个块键下已经存着不同的块或不同的违例（`DifferentVerdictsUnderAStoredBlockKeyNotDecidedInTheFirstVersion`，
    /// 什么都没写）；已存的违例读回来不合布局；RocksDB 读写失败。
    ///
    /// # Panics
    /// 块里有这个库没登记过的编号（拿了别的库的编号），或违例状态的块内序号不在块里：都是调用方的 bug。
    pub fn store_verdict_block(
        &mut self,
        block_key: &VerdictBlockKey,
        block: &VerdictBlock,
        violating_states: &[StateOffsetInBlock],
    ) -> Result<VerdictBlockStoring, VerdictStoreError> {
        let registered_count = self.verdict_vector_by_number.len();
        assert!(
            block
                .verdict_vector_number_of_each_state()
                .iter()
                .all(|number| usize::from(number.0) < registered_count),
            "块里的判定向量编号必须是这个库登记过的（这个库登记了 {registered_count} 种）"
        );
        let requested_violations: Vec<StoredViolation> = violating_states
            .iter()
            .copied()
            .collect::<BTreeSet<StateOffsetInBlock>>()
            .into_iter()
            .map(|offset_in_block| StoredViolation {
                offset_in_block,
                verdict_vector_number: block
                    .verdict_vector_number_at(offset_in_block)
                    .expect("违例状态的块内序号必须在块里（调用方给的序号超出了块长）"),
            })
            .collect();
        let block_key_bytes = VerdictStoreKey::VerdictBlock(block_key).to_key_bytes();
        let block_value_bytes = block.to_block_value_bytes();
        let stored_block_value = self
            .database
            .get(&block_key_bytes.0)
            .map_err(VerdictStoreError::RocksDatabase)?;
        if let Some(stored_block_value_bytes) = stored_block_value {
            let stored_violations = self.violations_in_block(block_key)?;
            let is_identical = stored_block_value_bytes == block_value_bytes
                && stored_violations == requested_violations;
            if is_identical {
                return Ok(VerdictBlockStoring::AlreadyStoredIdentically);
            }
            return Err(
                VerdictStoreError::DifferentVerdictsUnderAStoredBlockKeyNotDecidedInTheFirstVersion {
                    block_key: block_key.clone(),
                },
            );
        }
        let mut batch = rocksdb::WriteBatch::default();
        batch.put(&block_key_bytes.0, &block_value_bytes);
        for violation in &requested_violations {
            batch.put(
                VerdictStoreKey::Violation(block_key, violation.offset_in_block)
                    .to_key_bytes()
                    .0,
                [violation.verdict_vector_number.0],
            );
        }
        self.database
            .write_opt(batch, &synced_write_options())
            .map_err(VerdictStoreError::RocksDatabase)?;
        Ok(VerdictBlockStoring::WrittenAndSynced)
    }

    fn registered_verdict_vector_number(
        &self,
        table: VerdictStoreTable,
        key_bytes: &[u8],
        stored_number: u8,
    ) -> Result<VerdictVectorNumber, VerdictStoreError> {
        if usize::from(stored_number) < self.verdict_vector_by_number.len() {
            return Ok(VerdictVectorNumber(stored_number));
        }
        Err(stored_bytes_mismatch(
            table,
            key_bytes,
            "存着的判定向量编号在这个库里没登记过",
        ))
    }

    /// 读回一块；这个块键下没存过时是 `None`。
    ///
    /// # Errors
    /// 存着的块值是空的或点名了没登记的编号；RocksDB 读失败。
    pub fn read_verdict_block(
        &self,
        block_key: &VerdictBlockKey,
    ) -> Result<Option<VerdictBlock>, VerdictStoreError> {
        let block_key_bytes = VerdictStoreKey::VerdictBlock(block_key).to_key_bytes();
        let Some(block_value_bytes) = self
            .database
            .get(&block_key_bytes.0)
            .map_err(VerdictStoreError::RocksDatabase)?
        else {
            return Ok(None);
        };
        let verdict_vector_number_of_each_state = block_value_bytes
            .iter()
            .map(|stored_number| {
                self.registered_verdict_vector_number(
                    VerdictStoreTable::VerdictBlocks,
                    &block_key_bytes.0,
                    *stored_number,
                )
            })
            .collect::<Result<Vec<VerdictVectorNumber>, VerdictStoreError>>()?;
        VerdictBlock::new(verdict_vector_number_of_each_state)
            .map(Some)
            .map_err(|EmptyVerdictBlockRejected| {
                stored_bytes_mismatch(
                    VerdictStoreTable::VerdictBlocks,
                    &block_key_bytes.0,
                    "存着的块值是空的",
                )
            })
    }

    /// 这一块的违例，按块内序号排；没有违例或没存过这一块时是空的。
    ///
    /// # Errors
    /// 违例表里的键或值长度不对、点名了没登记的编号；RocksDB 读失败。
    pub fn violations_in_block(
        &self,
        block_key: &VerdictBlockKey,
    ) -> Result<Vec<StoredViolation>, VerdictStoreError> {
        let prefix = VerdictStoreKey::ViolationsOfBlockPrefix(block_key).to_key_bytes();
        let prefix_length = prefix.0.len();
        let mut violations = Vec::new();
        for entry in entries_with_key_prefix(&self.database, prefix) {
            let (key_bytes, number_bytes) = entry.map_err(VerdictStoreError::RocksDatabase)?;
            let offset_bytes =
                <[u8; STATE_OFFSET_IN_BLOCK_BYTES]>::try_from(&key_bytes[prefix_length..]);
            let (Ok(offset_bytes), &[stored_number]) = (offset_bytes, &*number_bytes) else {
                return Err(stored_bytes_mismatch(
                    VerdictStoreTable::Violations,
                    &key_bytes,
                    "违例表里块键之后不是 8 字节的块内序号，或值不是一个字节",
                ));
            };
            violations.push(StoredViolation {
                offset_in_block: StateOffsetInBlock(u64::from_be_bytes(offset_bytes)),
                verdict_vector_number: self.registered_verdict_vector_number(
                    VerdictStoreTable::Violations,
                    &key_bytes,
                    stored_number,
                )?,
            });
        }
        Ok(violations)
    }

    /// 录入一块：这一块要核 `state_count` 个状态。同一块录两次、状态数相同不算错；不同是调用方的 bug。
    ///
    /// # Errors
    /// RocksDB 读写失败；同一个块键已录过、状态数不同。
    pub fn record_block(
        &mut self,
        block_key: &VerdictBlockKey,
        state_count: u64,
    ) -> Result<(), VerdictStoreError> {
        let key_bytes = VerdictStoreKey::RecordedBlock(block_key).to_key_bytes();
        if let Some(stored) = self
            .database
            .get(&key_bytes.0)
            .map_err(VerdictStoreError::RocksDatabase)?
        {
            if stored == state_count.to_be_bytes() {
                return Ok(());
            }
            return Err(VerdictStoreError::DifferentVerdictsUnderAStoredBlockKeyNotDecidedInTheFirstVersion { block_key: block_key.clone() });
        }
        let mut batch = rocksdb::WriteBatch::default();
        batch.put(&key_bytes.0, state_count.to_be_bytes());
        self.database
            .write_opt(batch, &synced_write_options())
            .map_err(VerdictStoreError::RocksDatabase)
    }

    /// 路径索引登一条：这条路径上的节点是这个身份。同一路径再登、身份相同不算错；身份不同也照写（同名不同数据的分叉在路径这一级并成一段，
    /// 最后写的那个留在索引里；两边都在块表里，一个不丢）。
    ///
    /// # Errors
    /// RocksDB 写失败。
    pub fn record_path(&mut self, path: &str, identity: &str) -> Result<(), VerdictStoreError> {
        let key_bytes = VerdictStoreKey::Path(path).to_key_bytes();
        let mut batch = rocksdb::WriteBatch::default();
        batch.put(&key_bytes.0, identity.as_bytes());
        self.database
            .write_opt(batch, &synced_write_options())
            .map_err(VerdictStoreError::RocksDatabase)
    }

    /// 这条路径上登过的身份；没登过交回 None。
    ///
    /// # Errors
    /// RocksDB 读失败；值不是 UTF-8。
    pub fn identity_of_path(&self, path: &str) -> Result<Option<String>, VerdictStoreError> {
        let key_bytes = VerdictStoreKey::Path(path).to_key_bytes();
        let Some(value) = self
            .database
            .get(&key_bytes.0)
            .map_err(VerdictStoreError::RocksDatabase)?
        else {
            return Ok(None);
        };
        String::from_utf8(value.clone()).map(Some).map_err(|_| {
            stored_bytes_mismatch(VerdictStoreTable::PathIndex, &value, "身份不是 UTF-8")
        })
    }

    /// 路径索引里以这段前缀起头的全部（路径，身份），按键序：给空串就是整棵树；给 `<某节点路径>/` 就是它的子树。
    ///
    /// # Errors
    /// RocksDB 读失败；键或值不是 UTF-8。
    pub fn paths_with_prefix(
        &self,
        prefix: &str,
    ) -> Result<Vec<(String, String)>, VerdictStoreError> {
        let prefix_bytes = VerdictStoreKey::Path(prefix).to_key_bytes();
        let mut found = Vec::new();
        for entry in entries_with_key_prefix(&self.database, prefix_bytes) {
            let (key_bytes, value_bytes) = entry.map_err(VerdictStoreError::RocksDatabase)?;
            let path = String::from_utf8(key_bytes[1..].to_vec()).map_err(|_| {
                stored_bytes_mismatch(VerdictStoreTable::PathIndex, &key_bytes, "路径不是 UTF-8")
            })?;
            let identity = String::from_utf8(value_bytes.to_vec()).map_err(|_| {
                stored_bytes_mismatch(VerdictStoreTable::PathIndex, &value_bytes, "身份不是 UTF-8")
            })?;
            found.push((path, identity));
        }
        Ok(found)
    }

    /// 事实表登一条流（第 ① 段）：同一摘要再登就是同一份内容，照写不算错。
    ///
    /// # Errors
    /// RocksDB 写失败。
    pub fn store_flow_facts(
        &mut self,
        facts_digest: &[u8; 32],
        facts_bytes: &[u8],
    ) -> Result<(), VerdictStoreError> {
        let key_bytes = VerdictStoreKey::FlowFacts(facts_digest).to_key_bytes();
        let mut batch = rocksdb::WriteBatch::default();
        batch.put(&key_bytes.0, facts_bytes);
        self.database
            .write_opt(batch, &synced_write_options())
            .map_err(VerdictStoreError::RocksDatabase)
    }

    /// 读一条流的事实；没登过交回 None。
    ///
    /// # Errors
    /// RocksDB 读失败。
    pub fn read_flow_facts(
        &self,
        facts_digest: &[u8; 32],
    ) -> Result<Option<Vec<u8>>, VerdictStoreError> {
        let key_bytes = VerdictStoreKey::FlowFacts(facts_digest).to_key_bytes();
        self.database
            .get(&key_bytes.0)
            .map_err(VerdictStoreError::RocksDatabase)
    }

    /// 核对表写一块（第 ② 段）：这一版核对代码在这一块每个状态的结论位。同一键再写就是同一份结论，照写。
    ///
    /// # Errors
    /// RocksDB 写失败。
    pub fn store_verifier_block(
        &mut self,
        block_key: &VerdictBlockKey,
        verifier_digest: &[u8; 32],
        facts_digest: &[u8; 32],
        bits_of_each_state: &[u32],
    ) -> Result<(), VerdictStoreError> {
        let key_bytes = VerdictStoreKey::VerifierBlock(block_key, verifier_digest).to_key_bytes();
        let mut value = facts_digest.to_vec();
        for bits in bits_of_each_state {
            value.extend_from_slice(&bits.to_le_bytes());
        }
        let mut batch = rocksdb::WriteBatch::default();
        batch.put(&key_bytes.0, value);
        self.database
            .write_opt(batch, &synced_write_options())
            .map_err(VerdictStoreError::RocksDatabase)
    }

    /// 读核对表的一块；这一版核对代码没核过这一块交回 None。
    ///
    /// # Errors
    /// RocksDB 读失败；值的长度对不上。
    pub fn read_verifier_block(
        &self,
        block_key: &VerdictBlockKey,
        verifier_digest: &[u8; 32],
    ) -> Result<Option<VerifierBlockRow>, VerdictStoreError> {
        let key_bytes = VerdictStoreKey::VerifierBlock(block_key, verifier_digest).to_key_bytes();
        let Some(value) = self
            .database
            .get(&key_bytes.0)
            .map_err(VerdictStoreError::RocksDatabase)?
        else {
            return Ok(None);
        };
        if value.len() < 32 || !(value.len() - 32).is_multiple_of(4) {
            return Err(stored_bytes_mismatch(
                VerdictStoreTable::VerifierVerdicts,
                &value,
                "值要是 32 字节事实摘要加每状态 4 字节",
            ));
        }
        let mut facts_digest = [0u8; 32];
        facts_digest.copy_from_slice(&value[..32]);
        let (chunks, _remainder) = value[32..].as_chunks::<4>();
        let bits = chunks
            .iter()
            .map(|chunk| u32::from_le_bytes(*chunk))
            .collect();
        Ok(Some(VerifierBlockRow { facts_digest, bits }))
    }

    /// 核对表里全部行的原始键值（导入用）。
    fn verifier_rows(&self) -> Result<RawRows, VerdictStoreError> {
        let prefix =
            VerdictStoreKey::WholeTablePrefix(VerdictStoreTable::VerifierVerdicts).to_key_bytes();
        let mut rows = Vec::new();
        for entry in entries_with_key_prefix(&self.database, prefix) {
            let (key_bytes, value_bytes) = entry.map_err(VerdictStoreError::RocksDatabase)?;
            rows.push((key_bytes.to_vec(), value_bytes.to_vec()));
        }
        Ok(rows)
    }

    /// 事实表里全部行的原始键值（导入用）。
    fn facts_rows(&self) -> Result<RawRows, VerdictStoreError> {
        let prefix = VerdictStoreKey::WholeTablePrefix(VerdictStoreTable::FlowFacts).to_key_bytes();
        let mut rows = Vec::new();
        for entry in entries_with_key_prefix(&self.database, prefix) {
            let (key_bytes, value_bytes) = entry.map_err(VerdictStoreError::RocksDatabase)?;
            rows.push((key_bytes.to_vec(), value_bytes.to_vec()));
        }
        Ok(rows)
    }

    fn block_keys_of_table(
        &self,
        table: VerdictStoreTable,
    ) -> Result<Vec<(VerdictBlockKey, Vec<u8>)>, VerdictStoreError> {
        let prefix = VerdictStoreKey::WholeTablePrefix(table).to_key_bytes();
        let mut found = Vec::new();
        for entry in entries_with_key_prefix(&self.database, prefix) {
            let (key_bytes, value_bytes) = entry.map_err(VerdictStoreError::RocksDatabase)?;
            let block_key = VerdictBlockKey::from_block_key_bytes(&key_bytes[1..])
                .ok_or_else(|| stored_bytes_mismatch(table, &key_bytes, "键字节认不回块键"))?;
            found.push((block_key, value_bytes.to_vec()));
        }
        Ok(found)
    }

    /// 录入表里的全部块与各自的状态数，按键序。对账用：录了而判定块表里没有同一个键的，就是还没核完的。
    ///
    /// # Errors
    /// RocksDB 读失败；键或值字节认不回来。
    pub fn recorded_blocks(&self) -> Result<Vec<(VerdictBlockKey, u64)>, VerdictStoreError> {
        self.block_keys_of_table(VerdictStoreTable::RecordedBlocks)?
            .into_iter()
            .map(|(block_key, value)| {
                let count = <[u8; 8]>::try_from(value.as_slice())
                    .map(u64::from_be_bytes)
                    .map_err(|_| {
                        stored_bytes_mismatch(
                            VerdictStoreTable::RecordedBlocks,
                            &value,
                            "状态数不是 8 字节",
                        )
                    })?;
                Ok((block_key, count))
            })
            .collect()
    }

    /// 判定块表里的全部块键，按键序。
    ///
    /// # Errors
    /// RocksDB 读失败；键字节认不回来。
    pub fn stored_verdict_block_keys(&self) -> Result<Vec<VerdictBlockKey>, VerdictStoreError> {
        Ok(self
            .block_keys_of_table(VerdictStoreTable::VerdictBlocks)?
            .into_iter()
            .map(|(block_key, _)| block_key)
            .collect())
    }

    /// 把另一台的库导进来（两台各写各的库、最后并成一个）：录入表、路径索引、事实表与核对表逐行照抄；判定块按判定向量的内容在这个库里重新登记编号，
    /// 连同违例一起写；这个库里已有同一块键、判定相同的算已导入，不同就报错。交回导入的判定块数。
    ///
    /// # Errors
    /// RocksDB 读写失败；同一块键两边判定不同；判定向量超过 256 种。
    pub fn import_from(&mut self, other: &VerdictStore) -> Result<u64, VerdictStoreError> {
        for (block_key, state_count) in other.recorded_blocks()? {
            self.record_block(&block_key, state_count)?;
        }
        for (path, identity) in other.paths_with_prefix("")? {
            self.record_path(&path, &identity)?;
        }
        // 事实表与核对表逐行照抄（键里带各自的代码摘要，两边同键就是同内容）
        for (key_bytes, value_bytes) in other
            .facts_rows()?
            .into_iter()
            .chain(other.verifier_rows()?)
        {
            let mut batch = rocksdb::WriteBatch::default();
            batch.put(&key_bytes, &value_bytes);
            self.database
                .write_opt(batch, &synced_write_options())
                .map_err(VerdictStoreError::RocksDatabase)?;
        }
        let mut imported = 0u64;
        for block_key in other.stored_verdict_block_keys()? {
            let block = other
                .read_verdict_block(&block_key)?
                .expect("刚列出来的块键读得到");
            let mut renumbered =
                Vec::with_capacity(block.verdict_vector_number_of_each_state().len());
            for number in block.verdict_vector_number_of_each_state() {
                let vector = other
                    .verdict_vector_of_number(*number)
                    .expect("块里的编号在它自己的库里登记过")
                    .clone();
                renumbered.push(self.register_verdict_vector(&vector)?);
            }
            let violating: Vec<StateOffsetInBlock> = other
                .violations_in_block(&block_key)?
                .into_iter()
                .map(|violation| violation.offset_in_block)
                .collect();
            let renumbered_block = VerdictBlock::new(renumbered).expect("读回的块至少一个状态");
            self.store_verdict_block(&block_key, &renumbered_block, &violating)?;
            imported += 1;
        }
        Ok(imported)
    }

    #[cfg(test)]
    fn every_stored_entry(&self) -> Vec<StoredEntry> {
        self.database
            .iterator(rocksdb::IteratorMode::Start)
            .map(|entry| entry.expect("测试里读整个库不该失败"))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    /// 测试用的库目录：放在 `${TMPDIR:-/tmp}` 下，名字带测试名、进程号与序号，丢掉时连同里面的库一起删。
    struct TemporaryLibraryDirectory {
        path: PathBuf,
    }

    static NEXT_TEMPORARY_DIRECTORY_SUFFIX: AtomicU64 = AtomicU64::new(0);

    impl TemporaryLibraryDirectory {
        fn new(test_name: &str) -> Self {
            let suffix = NEXT_TEMPORARY_DIRECTORY_SUFFIX.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "singlefs-verdict-store-{test_name}-{}-{suffix}",
                std::process::id()
            ));
            assert!(
                !path.exists(),
                "测试库目录 {} 已经在了：同一个进程号与序号不该撞上",
                path.display()
            );
            Self { path }
        }
    }

    impl Drop for TemporaryLibraryDirectory {
        fn drop(&mut self) {
            if let Err(error) = std::fs::remove_dir_all(&self.path) {
                if error.kind() != std::io::ErrorKind::NotFound {
                    eprintln!("删不掉测试库目录 {}：{error}", self.path.display());
                }
            }
        }
    }

    const FINGERPRINT_BYTE: u8 = 0x11;
    const PLAN_HASH_BYTE: u8 = 0x22;

    fn block_key_of(
        fingerprint_byte: u8,
        name: &str,
        plan_hash_byte: u8,
        block_start: u64,
    ) -> VerdictBlockKey {
        VerdictBlockKey {
            input_fingerprint: InputFingerprint([fingerprint_byte; SHA256_DIGEST_BYTES]),
            stream_or_node_name: StreamOrNodeName::new(name).expect("测试里的名字非空、不长"),
            enumeration_plan_hash: EnumerationPlanHash([plan_hash_byte; SHA256_DIGEST_BYTES]),
            block_start: BlockStartStateIndex(block_start),
        }
    }

    fn block_key_named(name: &str, block_start: u64) -> VerdictBlockKey {
        block_key_of(FINGERPRINT_BYTE, name, PLAN_HASH_BYTE, block_start)
    }

    fn verdict_vector(bytes: &[u8]) -> EncodedVerdictVector {
        EncodedVerdictVector(bytes.to_vec())
    }

    fn block_of(numbers: &[VerdictVectorNumber]) -> VerdictBlock {
        VerdictBlock::new(numbers.to_vec()).expect("测试里的块至少一个状态")
    }

    #[test]
    fn stored_block_reads_back_byte_for_byte_identical() {
        let directory = TemporaryLibraryDirectory::new("read-back");
        let mut store = VerdictStore::create_empty(&directory.path).expect("建库");
        let clean = store
            .register_verdict_vector(&verdict_vector(b"clean"))
            .expect("登记");
        let torn = store
            .register_verdict_vector(&verdict_vector(b"torn"))
            .expect("登记");
        let lost = store
            .register_verdict_vector(&verdict_vector(b"lost"))
            .expect("登记");
        let block_key = block_key_named("stream", 65536);
        let block = block_of(&[clean, torn, clean, lost, lost]);
        assert_eq!(
            store
                .store_verdict_block(&block_key, &block, &[])
                .expect("存块"),
            VerdictBlockStoring::WrittenAndSynced
        );
        assert_eq!(
            store.read_verdict_block(&block_key).expect("读块"),
            Some(block),
            "读回来的块要与存进去的逐个编号相同"
        );
        let stored_value_bytes = store
            .database
            .get(VerdictStoreKey::VerdictBlock(&block_key).to_key_bytes().0)
            .expect("读原始值")
            .expect("块在库里");
        assert_eq!(
            stored_value_bytes,
            vec![0, 1, 0, 2, 2],
            "库里的块值要是每个状态一个字节的编号"
        );
    }

    #[test]
    fn blocks_and_verdict_vector_numbers_survive_closing_and_reopening() {
        let directory = TemporaryLibraryDirectory::new("reopen");
        let block_key = block_key_named("stream", 0);
        let (clean, torn, block) = {
            let mut store = VerdictStore::create_empty(&directory.path).expect("建库");
            let clean = store
                .register_verdict_vector(&verdict_vector(b"clean"))
                .expect("登记");
            let torn = store
                .register_verdict_vector(&verdict_vector(b"torn"))
                .expect("登记");
            let block = block_of(&[torn, clean, torn]);
            store
                .store_verdict_block(&block_key, &block, &[StateOffsetInBlock(2)])
                .expect("存块");
            (clean, torn, block)
        };
        let (mut store, opening) = VerdictStore::open_existing(&directory.path).expect("重开");
        assert_eq!(opening, VerdictStoreOpening::OpenedAtTheFirstAttempt);
        assert_eq!(
            store.read_verdict_block(&block_key).expect("读块"),
            Some(block)
        );
        assert_eq!(
            store.violations_in_block(&block_key).expect("读违例"),
            vec![StoredViolation {
                offset_in_block: StateOffsetInBlock(2),
                verdict_vector_number: torn,
            }]
        );
        assert_eq!(
            store.verdict_vector_of_number(clean),
            Some(&verdict_vector(b"clean"))
        );
        assert_eq!(
            store.verdict_vector_of_number(torn),
            Some(&verdict_vector(b"torn"))
        );
        assert_eq!(
            store
                .register_verdict_vector(&verdict_vector(b"torn"))
                .expect("登记"),
            torn,
            "重开之后再登记同一种判定向量要拿到原来的编号"
        );
        assert_eq!(
            store
                .register_verdict_vector(&verdict_vector(b"lost"))
                .expect("登记")
                .as_u8(),
            2,
            "重开之后新登记的判定向量接着原来的编号往下给"
        );
    }

    #[test]
    fn registering_the_same_verdict_vector_twice_returns_the_same_number() {
        let directory = TemporaryLibraryDirectory::new("register-twice");
        let mut store = VerdictStore::create_empty(&directory.path).expect("建库");
        let clean = store
            .register_verdict_vector(&verdict_vector(b"clean"))
            .expect("登记");
        let torn = store
            .register_verdict_vector(&verdict_vector(b"torn"))
            .expect("登记");
        let entries_after_two_registrations = store.every_stored_entry();
        assert_eq!(
            store
                .register_verdict_vector(&verdict_vector(b"clean"))
                .expect("登记"),
            clean,
            "第二次登记同一种判定向量要拿到第一次的编号"
        );
        assert_ne!(clean, torn);
        assert_eq!(
            store.every_stored_entry(),
            entries_after_two_registrations,
            "第二次登记同一种判定向量不写库"
        );
    }

    #[test]
    fn two_different_block_keys_do_not_overwrite_each_other() {
        let directory = TemporaryLibraryDirectory::new("distinct-keys");
        let mut store = VerdictStore::create_empty(&directory.path).expect("建库");
        let clean = store
            .register_verdict_vector(&verdict_vector(b"clean"))
            .expect("登记");
        let torn = store
            .register_verdict_vector(&verdict_vector(b"torn"))
            .expect("登记");
        let first_key = block_key_of(FINGERPRINT_BYTE, "stream", PLAN_HASH_BYTE, 0);
        let other_fingerprint = block_key_of(0x33, "stream", PLAN_HASH_BYTE, 0);
        let other_name = block_key_of(FINGERPRINT_BYTE, "streamx", PLAN_HASH_BYTE, 0);
        let other_plan = block_key_of(FINGERPRINT_BYTE, "stream", 0x44, 0);
        let other_start = block_key_of(FINGERPRINT_BYTE, "stream", PLAN_HASH_BYTE, 1);
        let keys_and_blocks = [
            (first_key, block_of(&[clean])),
            (other_fingerprint, block_of(&[torn])),
            (other_name, block_of(&[clean, torn])),
            (other_plan, block_of(&[torn, clean])),
            (other_start, block_of(&[torn, torn])),
        ];
        for (block_key, block) in &keys_and_blocks {
            assert_eq!(
                store
                    .store_verdict_block(block_key, block, &[])
                    .expect("存块"),
                VerdictBlockStoring::WrittenAndSynced,
                "块键 {block_key:?} 之前没存过"
            );
        }
        for (block_key, block) in &keys_and_blocks {
            assert_eq!(
                store.read_verdict_block(block_key).expect("读块").as_ref(),
                Some(block),
                "块键 {block_key:?} 下读回的要是它自己的块"
            );
        }
    }

    #[test]
    fn violations_are_found_again_by_their_block_key() {
        let directory = TemporaryLibraryDirectory::new("violations");
        let mut store = VerdictStore::create_empty(&directory.path).expect("建库");
        let clean = store
            .register_verdict_vector(&verdict_vector(b"clean"))
            .expect("登记");
        let torn = store
            .register_verdict_vector(&verdict_vector(b"torn"))
            .expect("登记");
        let lost = store
            .register_verdict_vector(&verdict_vector(b"lost"))
            .expect("登记");
        let first_key = block_key_named("stream", 0);
        let key_with_a_longer_name = block_key_named("stream-longer", 0);
        let next_block_key = block_key_named("stream", 4);
        store
            .store_verdict_block(
                &first_key,
                &block_of(&[clean, lost, torn, clean]),
                &[StateOffsetInBlock(2), StateOffsetInBlock(1)],
            )
            .expect("存块");
        store
            .store_verdict_block(
                &key_with_a_longer_name,
                &block_of(&[torn, clean]),
                &[StateOffsetInBlock(0)],
            )
            .expect("存块");
        store
            .store_verdict_block(&next_block_key, &block_of(&[clean, clean]), &[])
            .expect("存块");
        assert_eq!(
            store.violations_in_block(&first_key).expect("读违例"),
            vec![
                StoredViolation {
                    offset_in_block: StateOffsetInBlock(1),
                    verdict_vector_number: lost,
                },
                StoredViolation {
                    offset_in_block: StateOffsetInBlock(2),
                    verdict_vector_number: torn,
                },
            ],
            "按块键查回这一块自己的违例，按块内序号排，编号取块里那个状态的"
        );
        assert_eq!(
            store
                .violations_in_block(&key_with_a_longer_name)
                .expect("读违例"),
            vec![StoredViolation {
                offset_in_block: StateOffsetInBlock(0),
                verdict_vector_number: torn,
            }]
        );
        assert_eq!(
            store.violations_in_block(&next_block_key).expect("读违例"),
            Vec::new(),
            "没有违例的块查回空的"
        );
    }

    #[test]
    fn different_block_under_a_stored_block_key_is_refused_and_the_library_is_unchanged() {
        let directory = TemporaryLibraryDirectory::new("different-block");
        let mut store = VerdictStore::create_empty(&directory.path).expect("建库");
        let clean = store
            .register_verdict_vector(&verdict_vector(b"clean"))
            .expect("登记");
        let torn = store
            .register_verdict_vector(&verdict_vector(b"torn"))
            .expect("登记");
        let block_key = block_key_named("stream", 0);
        let block = block_of(&[clean, torn]);
        store
            .store_verdict_block(&block_key, &block, &[StateOffsetInBlock(1)])
            .expect("存块");
        let entries_before = store.every_stored_entry();
        let different_block = store.store_verdict_block(
            &block_key,
            &block_of(&[torn, torn]),
            &[StateOffsetInBlock(1)],
        );
        assert!(
            matches!(
                different_block,
                Err(VerdictStoreError::DifferentVerdictsUnderAStoredBlockKeyNotDecidedInTheFirstVersion { .. })
            ),
            "同一个块键下换一个块要拒：{different_block:?}"
        );
        let different_violations = store.store_verdict_block(&block_key, &block, &[]);
        assert!(
            matches!(
                different_violations,
                Err(VerdictStoreError::DifferentVerdictsUnderAStoredBlockKeyNotDecidedInTheFirstVersion { .. })
            ),
            "同一个块键下块相同、违例不同也要拒：{different_violations:?}"
        );
        assert_eq!(
            store.every_stored_entry(),
            entries_before,
            "拒了就一个字都不写"
        );
    }

    #[test]
    fn storing_the_identical_block_again_is_accepted_and_changes_nothing() {
        let directory = TemporaryLibraryDirectory::new("identical-block");
        let mut store = VerdictStore::create_empty(&directory.path).expect("建库");
        let clean = store
            .register_verdict_vector(&verdict_vector(b"clean"))
            .expect("登记");
        let torn = store
            .register_verdict_vector(&verdict_vector(b"torn"))
            .expect("登记");
        let block_key = block_key_named("stream", 0);
        let block = block_of(&[torn, clean]);
        store
            .store_verdict_block(&block_key, &block, &[StateOffsetInBlock(0)])
            .expect("存块");
        let entries_before = store.every_stored_entry();
        assert_eq!(
            store
                .store_verdict_block(&block_key, &block, &[StateOffsetInBlock(0)])
                .expect("再存同一块"),
            VerdictBlockStoring::AlreadyStoredIdentically,
            "续跑时重写逐字节相同的块要放行"
        );
        assert_eq!(store.every_stored_entry(), entries_before);
    }

    #[test]
    fn the_257th_distinct_verdict_vector_is_refused_and_the_library_is_unchanged() {
        let directory = TemporaryLibraryDirectory::new("vector-limit");
        let mut store = VerdictStore::create_empty(&directory.path).expect("建库");
        for expected_number in 0..=u8::MAX {
            let number = store
                .register_verdict_vector(&verdict_vector(&[expected_number, 0xAA]))
                .expect("前 256 种都登记得上");
            assert_eq!(number.as_u8(), expected_number);
        }
        let entries_before = store.every_stored_entry();
        let refused = store.register_verdict_vector(&verdict_vector(b"one more"));
        assert!(
            matches!(
                refused,
                Err(VerdictStoreError::MoreThan256VerdictVectorsNotDecidedInTheFirstVersion)
            ),
            "第 257 种要拒：{refused:?}"
        );
        assert_eq!(
            store.every_stored_entry(),
            entries_before,
            "拒了就一个字都不写"
        );
        assert_eq!(
            store.verdict_vector_by_number.len(),
            usize::from(u8::MAX) + 1
        );
    }

    #[test]
    fn an_existing_library_whose_current_file_is_gone_opens_after_one_repair() {
        let directory = TemporaryLibraryDirectory::new("repair");
        let block_key = block_key_named("stream", 0);
        let (torn, block) = {
            let mut store = VerdictStore::create_empty(&directory.path).expect("建库");
            let clean = store
                .register_verdict_vector(&verdict_vector(b"clean"))
                .expect("登记");
            let torn = store
                .register_verdict_vector(&verdict_vector(b"torn"))
                .expect("登记");
            let block = block_of(&[clean, torn]);
            store
                .store_verdict_block(&block_key, &block, &[StateOffsetInBlock(1)])
                .expect("存块");
            (torn, block)
        };
        std::fs::remove_file(directory.path.join("CURRENT"))
            .expect("删掉 CURRENT，造一个打不开的库");
        let (store, opening) =
            VerdictStore::open_existing(&directory.path).expect("修一次之后要能开");
        assert!(
            matches!(opening, VerdictStoreOpening::OpenedAfterOneRepair { .. }),
            "第一次打不开、修过才开得了：{opening:?}"
        );
        assert_eq!(
            store.read_verdict_block(&block_key).expect("读块"),
            Some(block)
        );
        assert_eq!(
            store.violations_in_block(&block_key).expect("读违例"),
            vec![StoredViolation {
                offset_in_block: StateOffsetInBlock(1),
                verdict_vector_number: torn,
            }]
        );
        assert_eq!(
            store.verdict_vector_of_number(torn),
            Some(&verdict_vector(b"torn"))
        );
    }

    #[test]
    fn creating_a_library_over_an_existing_directory_is_refused() {
        let directory = TemporaryLibraryDirectory::new("create-over-existing");
        std::fs::create_dir(&directory.path).expect("先建一个目录");
        let refused = VerdictStore::create_empty(&directory.path);
        assert!(
            matches!(
                refused,
                Err(VerdictStoreError::LibraryDirectoryAlreadyExists { .. })
            ),
            "目录已经在了，不在上面建库：{:?}",
            refused.err()
        );
        assert_eq!(
            std::fs::read_dir(&directory.path).expect("列目录").count(),
            0,
            "拒了就不往目录里写东西"
        );
    }

    #[test]
    fn opening_a_library_that_does_not_exist_is_refused_without_creating_it() {
        let directory = TemporaryLibraryDirectory::new("open-missing");
        let refused = VerdictStore::open_existing(&directory.path);
        assert!(
            matches!(
                refused,
                Err(VerdictStoreError::LibraryDirectoryMissing { .. })
            ),
            "没有库就不开：{:?}",
            refused.err()
        );
        assert!(!directory.path.exists(), "拒了就不建目录");
    }

    #[test]
    fn stored_block_naming_an_unregistered_verdict_vector_number_reads_as_a_layout_mismatch() {
        let directory = TemporaryLibraryDirectory::new("unregistered-number");
        let mut store = VerdictStore::create_empty(&directory.path).expect("建库");
        store
            .register_verdict_vector(&verdict_vector(b"clean"))
            .expect("登记");
        let block_key = block_key_named("stream", 0);
        store
            .database
            .put(
                VerdictStoreKey::VerdictBlock(&block_key).to_key_bytes().0,
                [0, 1],
            )
            .expect("绕过 store_verdict_block 直接写一个点名编号 1 的块值");
        let read = store.read_verdict_block(&block_key);
        assert!(
            matches!(
                read,
                Err(VerdictStoreError::StoredBytesDoNotMatchTheLayout {
                    table: VerdictStoreTable::VerdictBlocks,
                    ..
                })
            ),
            "只登记了编号 0，块值里有 1 要判不合布局：{read:?}"
        );
    }

    #[test]
    #[should_panic(expected = "违例状态的块内序号必须在块里")]
    fn violating_state_outside_the_block_is_a_caller_bug() {
        let directory = TemporaryLibraryDirectory::new("violation-outside");
        let mut store = VerdictStore::create_empty(&directory.path).expect("建库");
        let clean = store
            .register_verdict_vector(&verdict_vector(b"clean"))
            .expect("登记");
        let block_key = block_key_named("stream", 0);
        let _outcome = store.store_verdict_block(
            &block_key,
            &block_of(&[clean, clean]),
            &[StateOffsetInBlock(2)],
        );
    }

    #[test]
    #[should_panic(expected = "块里的判定向量编号必须是这个库登记过的")]
    fn verdict_vector_number_this_library_never_registered_is_a_caller_bug() {
        let directory = TemporaryLibraryDirectory::new("foreign-number");
        let mut store = VerdictStore::create_empty(&directory.path).expect("建库");
        store
            .register_verdict_vector(&verdict_vector(b"clean"))
            .expect("登记");
        let block_key = block_key_named("stream", 0);
        let _outcome =
            store.store_verdict_block(&block_key, &block_of(&[VerdictVectorNumber(1)]), &[]);
    }

    #[test]
    fn block_without_any_state_is_refused() {
        assert_eq!(
            VerdictBlock::new(Vec::new()),
            Err(EmptyVerdictBlockRejected)
        );
    }

    #[test]
    fn stream_or_node_name_takes_one_to_u16_maximum_bytes() {
        assert_eq!(
            StreamOrNodeName::new(""),
            Err(StreamOrNodeNameRejected {
                text: String::new()
            })
        );
        let longest_name = "n".repeat(usize::from(u16::MAX));
        assert!(
            StreamOrNodeName::new(&longest_name).is_ok(),
            "u16::MAX 字节的名字收"
        );
        assert!(
            StreamOrNodeName::new(&format!("{longest_name}n")).is_err(),
            "超过 u16::MAX 字节的名字不收"
        );
    }

    #[test]
    fn import_copies_the_path_index_so_the_merged_library_lists_the_other_side_s_tree() {
        let source_directory = TemporaryLibraryDirectory::new("path-index-source");
        let mut source = VerdictStore::create_empty(&source_directory.path).expect("建得了库");
        source
            .record_path("a.unit_write.1", "f::a.unit_write.1::00")
            .expect("登得进");
        source
            .record_path(
                "a.unit_write.1/b.journal_record.1",
                "f::b.journal_record.1::11",
            )
            .expect("登得进");
        assert_eq!(
            source.identity_of_path("a.unit_write.1").expect("读得了"),
            Some("f::a.unit_write.1::00".to_string())
        );
        assert_eq!(source.identity_of_path("nope").expect("读得了"), None);
        assert_eq!(
            source
                .paths_with_prefix("a.unit_write.1/")
                .expect("读得了")
                .len(),
            1,
            "按前缀扫是子树：根自己不算"
        );
        let merged_directory = TemporaryLibraryDirectory::new("path-index-merged");
        let mut merged = VerdictStore::create_empty(&merged_directory.path).expect("建得了库");
        assert_eq!(
            merged.import_from(&source).expect("导得进"),
            0,
            "没有判定块可导"
        );
        assert_eq!(
            merged.paths_with_prefix("").expect("读得了"),
            source.paths_with_prefix("").expect("读得了"),
            "路径索引随导入合并"
        );
    }
}
