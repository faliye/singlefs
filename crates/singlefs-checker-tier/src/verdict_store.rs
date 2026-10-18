//! 崩溃放量的判定存储（里程碑三第十一项第一段，`.claude/kb/milestone/03-third-txn.md`「十一」）：
//! 按块存每个崩溃状态的判定编号，另存不同的判定向量与违例。库是 RocksDB（用户 2026-09-28 选定），
//! 每次写都同步落盘、WAL 开着（E162（崩溃放量判定块存储选型） K 臂的写法）；打不开时调一次 `DB::repair` 再开。
//! 这一段只有存储本身，还没接进崩溃点重放（`crash`、`crash_point_replay_progress`），接法归第二轮三方。
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
//! | 判定块 | `01` ‖ 块键 ‖ 判法单元指纹（32 字节） | 块里每个状态一个判定向量编号（4 字节） |
//! | 判定向量按内容 | `02` ‖ 判定向量的字节 | 编号（4 字节） |
//! | 判定向量按编号 | `03` ‖ 编号（4 字节） | 判定向量的字节 |
//! | 违例 | `04` ‖ 块键 ‖ 判法单元指纹（32 字节） ‖ 块内序号（8 字节） | 那个状态的判定向量编号（4 字节） |
//! | 判法单元组成清单 | `09` ‖ 判法单元指纹（32 字节） | 这一把键的组成清单（一个单元一行，`crash_identity::JudgingUnitsKey::composition_text`）：库对不上时照它点名变了的单元 |
//!
//! 块键 = 段写表与起步镜像的摘要（32 字节）‖ 枚举计划哈希（32 字节：段展开、判器另读的输入、态数）‖ 块起点（8 字节），定长 72 字节。
//! 块键只由内容定（用户 2026-10-15 定：键里不许有路径、名字、种子）：设点文件路径、崩溃点名、项名、表名、种子串都不进块键，
//! 只进路径索引与报告；判器改了不动块键，判定块与违例另按判法单元指纹分行，复用只认同一把指纹。
//! 块键定长，一个块键不会是另一个块键的前缀，按块键前缀扫违例表只扫到这一块的。
//!
//! 判定向量编号是 u32、大端 4 字节（[`VerdictVectorNumber`]）：用户 2026-10-15 定「直接加宽到未来都不需要再加宽」，从一个字节一次加宽到 u32，
//! 一个库里最多 2^32 种判定向量。编号宽度是布局的事，跟着布局号走，不进任何复用键。
//!
//! 三种结论（崩溃点重放合并第三轮判决 J7、J8，第二轮判决 L1-8）另用下面几张表，与上面那几张同库并存：
//!
//! | 表 | 键 | 值 |
//! |---|---|---|
//! | 库布局 | `00` | 布局号，4 字节大端（5：块键只由内容定、有下面这几张表、观察者那一种结论不带名字、判定向量编号 4 字节）；没有这一行的是这一套之前的库；布局号 2、3、4 的判定向量编号还是一个字节，开库就拒 |
//! | 结论块 | `0A` ‖ 块键 ‖ 结论种类 ‖ 版本（32 字节） | 块里每个状态一个判定向量编号（4 字节） |
//! | 结论违例 | `0B` ‖ 块键 ‖ 结论种类 ‖ 版本 ‖ 块内序号（8 字节） | 那个状态的判定向量编号（4 字节） |
//! | 恢复字 | `0C` ‖ 块键 ‖ 结论种类 ‖ 版本 | 每个状态八个字（两遍各自落到的根两个字、结局、失败点），每字 4 字节小端；不进判定向量编号 |
//! | 计数向量 | `0D` ‖ 块键 ‖ 结论种类 ‖ 版本 | 逐条不变量判到 / 违例 / 不适用、各观察者计数、按发布分的态数（[`BlockCountVector`]） |
//! | 块属性 | `0E` ‖ 块键 ‖ 结论种类 ‖ 版本 | 在哪判的（CPU，或 GPU 连同卡型与驱动版本，[`JudgingDevice`]）、判定日期（东京日期 `YYYY-MM-DD`）与跑法（全量、剪枝、逐条，[`JudgementCircumstances`]）；都不进键 |
//! | 点名的被抛弃根 | `19` ‖ 块键 ‖ 结论种类 ‖ 版本 | 每个状态上池级 checker 判 I-7.4 时违例细节里点名的被抛弃根：条数 4 字节小端，再逐条实例代号 4 字节、checkpoint_txg 8 字节小端；存的时候给了才有（给定状态表那一路每块都给） |
//!
//! 两段（录入段在 CPU 上跑数据、写进库；判段只从库里读，段与段之间不在内存里交数据：用户 2026-10-11 定）另用三张表：
//!
//! | 表 | 键 | 值 |
//! |---|---|---|
//! | 录下的流程 | `10` ‖ 摘要（32 字节） | `crash_amplification::RecordedFlow` 的字节 |
//! | 一趟的清单 | `11` ‖ 录入键（32 字节：录入函数单元指纹与这一趟录入输入的摘要，[`RecordingKey`]） | `crash_amplification::RecordedRunManifest` 或 `crash_library::ItemManifest` 的字节；命中就不再录 |
//! | GPU 判表 | `12` ‖ 摘要（32 字节） | `crash_judge_dispatch::PackedJudgeTables` 的字节 |
//! | 给定状态表 | `13` ‖ 摘要（32 字节） | `crash_amplification::RecordedGivenStatesTable` 的字节（引擎先写进来、再读回来判） |
//!
//! 按段登记的段整域装不进 u64 时分片枚举（C790），每一段一条进度记录（只是加速：真相以块为准，核不过就从 0 起逐块问库）：
//!
//! | 表 | 键 | 值 |
//! |---|---|---|
//! | 分片进度 | `14` ‖ 段的内容摘要（32 字节）‖ 判器（`01` ‖ 判法单元指纹，或 `02` ‖ 结论种类 ‖ 版本） | 格式号 1 字节 ‖ 已判到的编号 ‖ 最后判完的那一块的起点（各 160 字节，`crash::StateCount1280Bits` 的小端字节） |
//!
//! 观察者两段（用户 2026-10-13 定「观察者改走两段、池级 checker 上 GPU、按恢复结果去重」）另用四张表：录入段在 CPU 上逐状态跑观察者
//! （交回镜像的那一种按恢复镜像字节去重，同一张镜像只看一次），判段只从这里读、对观察者交回的镜像跑池级 checker（配了 GPU 上卡）：
//!
//! | 表 | 键 | 值 |
//! |---|---|---|
//! | 观察者基底 | `15` ‖ 摘要（32 字节） | 一条流的基础镜像与枚举写表（`crash_amplification` 的观察者基底） |
//! | 看过的恢复镜像 | `16` ‖ 去重键（32 字节） | 观察者在这张镜像上看出的：红项、计数、它交回的镜像（基底摘要、持久集合、观察者在镜像上发的写、判哪几条不变量） |
//! | 观察者录入块 | `17` ‖ 块键 ‖ 结论种类 ‖ 观察者版本 | 块里去重之后的观察结果清单、每个状态的下标、按发布分的态数 |
//! | 镜像上的池级 checker | `18` ‖ 去重键 ‖ 判器种类 ‖ 判器版本 | 判到位、违例位、判不了的标记、在哪判的 |
//!
//! 交回镜像的观察者按恢复报告分桶、按读集见证去重（用户 2026-10-16 定「恢复报告分桶 + 读集见证」），见证与读集各按行存：添一条见证只写这一条见证行、
//! 这组读集在这个桶里头一次出现时的那一行读集行、它看出的那张镜像（`16` 表），三样同一批原子地落盘；读回时按桶键前缀扫出这个桶的全部见证行。
//!
//! | 表 | 键 | 值 |
//! |---|---|---|
//! | 读集见证行 | `1C` ‖ 桶键（32 字节）‖ 见证摘要（32 字节） | 读集摘要（32 字节，读集行的键）‖ 它看出的那张镜像的去重键（32 字节） |
//! | 读集行 | `1D` ‖ 读集摘要（32 字节） | 观察者真跑一次读过的段（`crash_amplification` 的读集字节；读集摘要就是这份字节的 SHA-256） |
//!
//! 标签 `1B` 退役、不再分配：那是改前的做法（一个桶的整份见证清单存成一行 `1B` ‖ 桶键，每条见证各带一份读集，每添一条整份重写），
//! 键不同、不再读写，不迁移；按前缀 `1B` 列得出它们。
//!
//! 对拍（GPU 与 CPU 两边判同一份输入、比结论）的结论另用一张表：
//!
//! | 表 | 键 | 值 |
//! |---|---|---|
//! | 对拍结论 | `1A` ‖ 输入摘要（32 字节）‖ 一边的判法单元指纹（32 字节）‖ 另一边的判法单元指纹（32 字节） | 对拍的结论（调用方给的字节）、判定日期与跑法 |
//!
//! 结论种类 = `01`（GPU 判器）、`02`（CPU 判器）、`03`（观察者）；版本是 `crash_identity::ConclusionVersion`，取自判法单元指纹
//! （`crash_identity::judging_units_key`）。观察者不带名字：是哪一个观察者由版本担（它那一组判法单元按入口 fn 定，同一份文件里两个观察者
//! 两把键；4-2a 报告第四节第 5 条），名字只进组名与行里报的名字。
//! 一块一种结论的结论块、违例、恢复字、计数向量与块属性同一批原子地写、一起读回。这一套之前的库（没有布局行）用
//! [`VerdictStore::open_existing_with_conclusion_kinds`] 打开时拒绝、不迁移：那时的判定块按（块键，两个判器合起来的判法单元指纹）存，
//! 三种结论的键都与它不同，迁过来一块也复用不上；换一个新目录建库，就是一趟清空复用的全量。布局号 2（块键里还带节点名）、3（观察者的种类里
//! 还带名字）、4（判定向量编号一个字节）的库，三种开法一律在读判定向量表之前就拒（[`VerdictStoreError::LibraryLayoutWithOneByteVerdictVectorNumbers`]）、
//! 不迁移：它们的编号都是一个字节，按 4 字节读一行也读不对。
//!
//! 照判、不入库的那几把版本（4-2a 报告第四节第 7 条后半句：预检里一部分判法单元拒算时，拒算的那几组照判不入库，别的照键入库同一份库）：
//! [`VerdictStore::keep_the_rows_of_these_versions_out_of_the_library`] 登记的版本，键里版本那一截（判法单元指纹或结论版本，32 字节，
//! 按表标签定的偏移，见上面几张表的键）是它的行一律不写进 RocksDB，只记在这个进程里（[`RowsKeptOutOfTheLibrary`]），这个进程里读的时候
//! 与库里的行两路有序流逐行归并着读；进程一退就没了，别的进程、下一趟都看不到、不复用。
//! 写入路径按键分开，判段、核对、观察者、分片进度照常经同一份库读写。

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::crash::{StateCount1280Bits, STATE_COUNT_BYTES};
use crate::crash_identity::{
    ConclusionKind, ConclusionVersion, JudgingUnitFingerprint, JudgingUnitsKey,
};
use crate::crash_judge_conclusions::AbandonedRootNamedByThePoolChecker;
/// 判定向量、恢复字与计数向量的类型在判器一侧（崩溃点重放合并第三轮判决 J8：改结论的几处搬进判器一侧），这里只管它们的字节布局，路径照旧从这里引。
pub use crate::crash_judge_conclusions::{
    BlockCountVector, EncodedVerdictVector, InvariantStateCounts, RecoveryWordsOfState,
    RECOVERY_WORDS_PER_STATE,
};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};

/// 段写表与起步镜像的摘要、枚举计划哈希都是 SHA-256 摘要的字节数。
pub const SHA256_DIGEST_BYTES: usize = 32;

/// 违例表的键里块内序号占几个字节。
const STATE_OFFSET_IN_BLOCK_BYTES: usize = std::mem::size_of::<u64>();

/// 块键里内容的那一半：这一段（或这一个点）带屏障的写表与它起步的镜像（扇区终值）的摘要（`crash_identity::StepAttributes::reuse_key`），
/// 给定状态表是这张表的基线、写表、接缝与旋钮栏的摘要。只由内容定，不带路径与名字。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeInputDigest(pub [u8; SHA256_DIGEST_BYTES]);

/// 枚举计划哈希（`crash` 模块现算的计划哈希，SHA-256）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EnumerationPlanHash(pub [u8; SHA256_DIGEST_BYTES]);

/// 按块键排的表（录入表、判定块表）里的一行：块键、块键之后的键后缀（判定块表是 32 字节判法单元指纹，录入表为空）、值。
type BlockTableRow = (VerdictBlockKey, Vec<u8>, Vec<u8>);

/// 判法单元指纹：判这一块的那几个判法单元的键（`crash_identity::judging_units_key`）。判定块与违例按（块键，判法单元指纹）存：
/// 同一块在几把指纹下的判定并存，复用只认同一把；录入表只按块键。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct JudgingUnitsFingerprint(pub [u8; SHA256_DIGEST_BYTES]);

/// 块起点：这一块第一个状态在它那份枚举计划里的序号。块长由调用方定，不进键。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlockStartStateIndex(pub u64);

/// 状态在块里的序号，从 0 起。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StateOffsetInBlock(pub u64);

/// 录入键：录入函数单元指纹与这一趟录入输入的摘要拼起来的 SHA-256（`crash_library::recording_key`）。录入段的清单按它登，
/// 命中就不再录；判段照同一份输入现算它，凭它读清单。不带项名、流名。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RecordingKey(pub [u8; SHA256_DIGEST_BYTES]);

/// 对拍结论的键：输入的摘要与两边判器各自的判法单元指纹（GPU 判器与 CPU 判器、GPU 核对与 CPU 参照这类）。两边次序由调用方定，
/// 同一种对拍每一趟同一个次序。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PairingConclusionKey {
    pub inputs_digest: [u8; SHA256_DIGEST_BYTES],
    pub one_side_judging_units: [u8; SHA256_DIGEST_BYTES],
    pub other_side_judging_units: [u8; SHA256_DIGEST_BYTES],
}

/// 存着的一条对拍结论：调用方给的结论字节（两边逐状态比出来的一致或不一致，写法由调用方定），连同判定日期与跑法。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredPairingConclusion {
    pub conclusion: Vec<u8>,
    pub circumstances: JudgementCircumstances,
}

impl StoredPairingConclusion {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(
            &u32::try_from(self.conclusion.len())
                .expect("对拍结论装得进 u32 字节")
                .to_be_bytes(),
        );
        bytes.extend_from_slice(&self.conclusion);
        push_length_prefixed_text(&mut bytes, &self.circumstances.judged_date);
        bytes.push(self.circumstances.run_kind.code());
        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut cursor = ByteCursor { bytes, position: 0 };
        let conclusion_length = usize::try_from(cursor.big_endian_u32()?).ok()?;
        let conclusion = cursor.take(conclusion_length)?.to_vec();
        let judged_date = cursor.length_prefixed_text()?;
        let run_kind = RunKindOfTheJudgement::of_code(*cursor.take(1)?.first()?)?;
        cursor.is_at_the_end().then_some(Self {
            conclusion,
            circumstances: JudgementCircumstances {
                judged_date,
                run_kind,
            },
        })
    }
}

/// 块属性里的跑法（words 路第八节第 7 项）：全量（库不读、全部现判、写回）、剪枝（键相同的复用、对不上的现判）、逐条（点名单跑一项）。
/// GPU 还是 CPU 在 [`JudgingDevice`] 那一格。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunKindOfTheJudgement {
    Full,
    Pruned,
    ItemByItem,
}

/// 入口脚本交跑法的环境变量：`full`、`pruned`、`item_by_item`；没设看 [`CRASH_LIBRARY_WRITE_ONLY_ENVIRONMENT_VARIABLE`]：
/// 只写不读是全量，否则是剪枝。
pub const CRASH_LIBRARY_RUN_KIND_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_CRASH_LIBRARY_RUN_KIND";

/// 只写不读（门禁 checker-tier-crash-point-replay 的 `--start-over`、崩溃点重放入口的 `--full`）：判定库不读，全部块现判，照键写回同一份库。
pub const CRASH_LIBRARY_WRITE_ONLY_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_CRASH_LIBRARY_WRITE_ONLY";

impl RunKindOfTheJudgement {
    /// 块属性里的码。
    fn code(self) -> u8 {
        match self {
            Self::Full => 1,
            Self::Pruned => 2,
            Self::ItemByItem => 3,
        }
    }

    fn of_code(code: u8) -> Option<Self> {
        match code {
            1 => Some(Self::Full),
            2 => Some(Self::Pruned),
            3 => Some(Self::ItemByItem),
            _ => None,
        }
    }

    /// 行里报的名字。
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Pruned => "pruned",
            Self::ItemByItem => "item_by_item",
        }
    }

    /// 两个变量的值（`None` 是没设）换成跑法。
    ///
    /// # Panics
    /// 跑法变量设了而认不得：写明变量名与值，不悄悄当剪枝。
    #[must_use]
    pub fn of_the_variable_values(run_kind: Option<&str>, write_only: Option<&str>) -> Self {
        match run_kind {
            Some("full") => Self::Full,
            Some("pruned") => Self::Pruned,
            Some("item_by_item") => Self::ItemByItem,
            Some(other) => panic!(
                "{CRASH_LIBRARY_RUN_KIND_ENVIRONMENT_VARIABLE}={other:?} 认不得：只认 full、pruned、item_by_item，或不设"
            ),
            None if write_only == Some("1") => Self::Full,
            None => Self::Pruned,
        }
    }

    /// 这个进程的跑法。
    ///
    /// # Panics
    /// 同 [`Self::of_the_variable_values`]。
    #[must_use]
    pub fn of_this_process() -> Self {
        Self::of_the_variable_values(
            std::env::var(CRASH_LIBRARY_RUN_KIND_ENVIRONMENT_VARIABLE)
                .ok()
                .as_deref(),
            std::env::var(CRASH_LIBRARY_WRITE_ONLY_ENVIRONMENT_VARIABLE)
                .ok()
                .as_deref(),
        )
    }
}

/// 这个进程是不是只写不读（[`CRASH_LIBRARY_WRITE_ONLY_ENVIRONMENT_VARIABLE`]`=1`）：复用的那一步一律当库里没有。
#[must_use]
pub fn library_reads_are_skipped_in_this_process() -> bool {
    std::env::var(CRASH_LIBRARY_WRITE_ONLY_ENVIRONMENT_VARIABLE).is_ok_and(|value| value == "1")
}

/// 一块判定是在什么情形下判的（块属性，不进键、不进结论的比对）：判定日期（东京日期 `YYYY-MM-DD`）与跑法。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JudgementCircumstances {
    pub judged_date: String,
    pub run_kind: RunKindOfTheJudgement,
}

impl JudgementCircumstances {
    /// 此刻在这个进程里判的：今天的东京日期与这个进程的跑法。
    ///
    /// # Panics
    /// 同 [`RunKindOfTheJudgement::of_this_process`]。
    #[must_use]
    pub fn of_this_process_now() -> Self {
        Self {
            judged_date: tokyo_date_text_of(std::time::SystemTime::now()),
            run_kind: RunKindOfTheJudgement::of_this_process(),
        }
    }
}

/// 东京比协调世界时早 9 小时，不用夏令时。
const TOKYO_SECONDS_AHEAD_OF_UTC: u64 = 9 * 3600;
const SECONDS_PER_DAY: u64 = 86_400;

/// 一个时刻在东京的日期 `YYYY-MM-DD`（公历；1970 年以前的时刻算作 1970-01-01）。
#[must_use]
pub fn tokyo_date_text_of(moment: std::time::SystemTime) -> String {
    let seconds_since_the_epoch = moment
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| since.as_secs());
    let days_since_the_epoch =
        i64::try_from((seconds_since_the_epoch + TOKYO_SECONDS_AHEAD_OF_UTC) / SECONDS_PER_DAY)
            .expect("天数装得进 i64");
    let (year, month, day) = civil_date_of_days_since_the_epoch(days_since_the_epoch);
    format!("{year:04}-{month:02}-{day:02}")
}

/// 从 1970-01-01 起的天数换成公历（年，月，日）：按四百年一轮、三月起算的年把天数拆开（闰日落在年末）。
fn civil_date_of_days_since_the_epoch(days_since_the_epoch: i64) -> (i64, i64, i64) {
    const DAYS_FROM_THE_ERA_START_TO_THE_EPOCH: i64 = 719_468;
    const DAYS_PER_ERA: i64 = 146_097;
    let days_since_the_era_start = days_since_the_epoch + DAYS_FROM_THE_ERA_START_TO_THE_EPOCH;
    let era = days_since_the_era_start.div_euclid(DAYS_PER_ERA);
    let day_of_the_era = days_since_the_era_start.rem_euclid(DAYS_PER_ERA);
    let year_of_the_era = (day_of_the_era - day_of_the_era / 1460 + day_of_the_era / 36_524
        - day_of_the_era / 146_096)
        / 365;
    let day_of_the_year_from_march =
        day_of_the_era - (365 * year_of_the_era + year_of_the_era / 4 - year_of_the_era / 100);
    let month_from_march = (5 * day_of_the_year_from_march + 2) / 153;
    let day = day_of_the_year_from_march - (153 * month_from_march + 2) / 5 + 1;
    let month = if month_from_march < 10 {
        month_from_march + 3
    } else {
        month_from_march - 9
    };
    let year = year_of_the_era + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

/// 核对表里的一行：这一块用哪份事实表核的、每个状态的结论位。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifierBlockRow {
    pub facts_digest: [u8; 32],
    pub bits: Vec<u32>,
}

/// 一张表的原始键值行（导入时逐行照抄）。
type RawRows = Vec<(Vec<u8>, Vec<u8>)>;

/// 判定块的键：只由内容定（定长 72 字节，见文件头）。
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VerdictBlockKey {
    pub node_input_digest: NodeInputDigest,
    pub enumeration_plan_hash: EnumerationPlanHash,
    pub block_start: BlockStartStateIndex,
}

/// 块键字节的长度：两个摘要加 8 字节块起点。
const BLOCK_KEY_BYTES: usize = 2 * SHA256_DIGEST_BYTES + 8;

impl VerdictBlockKey {
    fn to_block_key_bytes(&self) -> Vec<u8> {
        let mut key_bytes = Vec::with_capacity(BLOCK_KEY_BYTES);
        key_bytes.extend_from_slice(&self.node_input_digest.0);
        key_bytes.extend_from_slice(&self.enumeration_plan_hash.0);
        key_bytes.extend_from_slice(&self.block_start.0.to_be_bytes());
        key_bytes
    }

    /// 块键字节接上判法单元指纹（判定块表与违例表的键）。
    fn to_block_key_bytes_with_version(&self, version: &JudgingUnitsFingerprint) -> Vec<u8> {
        let mut key_bytes = self.to_block_key_bytes();
        key_bytes.extend_from_slice(&version.0);
        key_bytes
    }

    /// 块键字节接上结论种类与版本（结论块表、结论违例表、恢复字旁表、计数向量表、块属性表的键）。
    fn to_block_key_bytes_with_conclusion(
        &self,
        kind: &ConclusionKind,
        conclusion_version: &ConclusionVersion,
    ) -> Vec<u8> {
        let mut key_bytes = self.to_block_key_bytes();
        key_bytes.extend_from_slice(&conclusion_kind_key_bytes(kind));
        key_bytes.extend_from_slice(&conclusion_version.0);
        key_bytes
    }

    /// [`Self::to_block_key_bytes`] 的逆：列表、导入时从键字节认回块键，交回块键与它后面剩下的字节（判法单元指纹、块内序号这类后缀）；
    /// 不够长交回 None。
    fn from_block_key_bytes_with_suffix(bytes: &[u8]) -> Option<(Self, &[u8])> {
        let node_input_digest =
            <[u8; SHA256_DIGEST_BYTES]>::try_from(bytes.get(..SHA256_DIGEST_BYTES)?).ok()?;
        let plan_hash = <[u8; SHA256_DIGEST_BYTES]>::try_from(
            bytes.get(SHA256_DIGEST_BYTES..2 * SHA256_DIGEST_BYTES)?,
        )
        .ok()?;
        let start_bytes =
            <[u8; 8]>::try_from(bytes.get(2 * SHA256_DIGEST_BYTES..BLOCK_KEY_BYTES)?).ok()?;
        Some((
            Self {
                node_input_digest: NodeInputDigest(node_input_digest),
                enumeration_plan_hash: EnumerationPlanHash(plan_hash),
                block_start: BlockStartStateIndex(u64::from_be_bytes(start_bytes)),
            },
            &bytes[BLOCK_KEY_BYTES..],
        ))
    }

    /// 报告里写的这一块：两个摘要的头 8 字节与块起点（只给人看，不是键）。
    #[must_use]
    pub fn short_text(&self) -> String {
        format!(
            "{}:{}@{}",
            singlefs_harness::hexadecimal::hexadecimal_text(&self.node_input_digest.0[..8]),
            singlefs_harness::hexadecimal::hexadecimal_text(&self.enumeration_plan_hash.0[..8]),
            self.block_start.0
        )
    }
}

/// 分片进度记在哪一个判器、哪一把版本下（进度记录键的最后一截）：与它推进时那几块结论存在哪一处是同一处。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JudgeOfTheProgress {
    /// 判法单元指纹那一套（结论按（块键，判法单元指纹）存）。
    TheJudgingUnitsFingerprint(JudgingUnitsFingerprint),
    /// 三种结论里的一种，连这个段所在的点那一把版本。
    TheConclusionKind {
        kind: ConclusionKind,
        version: ConclusionVersion,
    },
}

/// 一条分片进度记录的键：这一段的内容摘要（它所在的点的段写表、起步镜像与计划哈希，连同它在点内的段序；不带流名）、判器
/// （[`JudgeOfTheProgress`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlicedSegmentProgressKey {
    pub segment_content_digest: [u8; SHA256_DIGEST_BYTES],
    pub judge: JudgeOfTheProgress,
}

/// 分片进度键里段的内容摘要之后那一个字节：判器是判法单元指纹那一套。
const SLICED_PROGRESS_JUDGE_MARKER_OF_THE_JUDGING_UNITS_FINGERPRINT: u8 = 0x01;
/// 分片进度键里段的内容摘要之后那一个字节：判器是三种结论里的一种（后面接结论种类与版本）。
const SLICED_PROGRESS_JUDGE_MARKER_OF_A_CONCLUSION_KIND: u8 = 0x02;

impl SlicedSegmentProgressKey {
    fn to_key_bytes_after_the_tag(&self) -> Vec<u8> {
        let mut key_bytes = self.segment_content_digest.to_vec();
        match &self.judge {
            JudgeOfTheProgress::TheJudgingUnitsFingerprint(judging_units_fingerprint) => {
                key_bytes.push(SLICED_PROGRESS_JUDGE_MARKER_OF_THE_JUDGING_UNITS_FINGERPRINT);
                key_bytes.extend_from_slice(judging_units_fingerprint.0.as_slice());
            }
            JudgeOfTheProgress::TheConclusionKind { kind, version } => {
                key_bytes.push(SLICED_PROGRESS_JUDGE_MARKER_OF_A_CONCLUSION_KIND);
                key_bytes.extend_from_slice(&conclusion_kind_key_bytes(kind));
                key_bytes.extend_from_slice(version.0.as_slice());
            }
        }
        key_bytes
    }
}

/// 一段分片的进度：段内编号 [0, `judged_through`) 的块都判完了；`last_block_start` 是最后判完的那一块的起点
/// （`judged_through` 是 0 时也写 0）。读回的人照它核「这一块在、`judged_through` 那一块不在」，核不过就不信它。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlicedSegmentProgress {
    pub judged_through: StateCount1280Bits,
    pub last_block_start: StateCount1280Bits,
}

/// 分片进度记录值的格式号（值的第一个字节）。
const SLICED_SEGMENT_PROGRESS_FORMAT_NUMBER: u8 = 1;

impl SlicedSegmentProgress {
    fn to_value_bytes(self) -> Vec<u8> {
        let mut bytes = vec![SLICED_SEGMENT_PROGRESS_FORMAT_NUMBER];
        bytes.extend_from_slice(&self.judged_through.to_le_bytes());
        bytes.extend_from_slice(&self.last_block_start.to_le_bytes());
        bytes
    }

    fn from_value_bytes(bytes: &[u8]) -> Option<Self> {
        let (format_number, rest) = bytes.split_first()?;
        if *format_number != SLICED_SEGMENT_PROGRESS_FORMAT_NUMBER
            || rest.len() != 2 * STATE_COUNT_BYTES
        {
            return None;
        }
        let (judged_through, last_block_start) = rest.split_at(STATE_COUNT_BYTES);
        Some(Self {
            judged_through: StateCount1280Bits::from_le_bytes(judged_through.try_into().ok()?),
            last_block_start: StateCount1280Bits::from_le_bytes(last_block_start.try_into().ok()?),
        })
    }
}

/// 判定向量的编号：同一个库里一种判定向量一个编号，从 0 起按登记次序给，登记过就不变（关库重开也不变）。
/// 只能从 [`VerdictStore::register_verdict_vector`] 或读回的块拿到。u32，库里存成大端 4 字节（文件头「判定向量编号是 u32」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VerdictVectorNumber(u32);

/// 一个判定向量编号在库里占的字节数：判定块与结论块里每个状态一份、违例的值、两张判定向量表里的编号都是这么宽。
const VERDICT_VECTOR_NUMBER_BYTES: usize = std::mem::size_of::<u32>();

impl VerdictVectorNumber {
    /// 已经登记了 `registered_count` 种之后，下一种拿到的编号（编号从 0 起按登记次序给，所以就是 `registered_count`）；
    /// 装不进 u32（2^32 种都登记过了）时是 `None`。
    fn next_after_registered_count(registered_count: usize) -> Option<Self> {
        u32::try_from(registered_count).ok().map(Self)
    }

    #[must_use]
    pub fn as_u32(self) -> u32 {
        self.0
    }

    /// 这个编号在登记次序里的位置（按编号排的那张内存表的下标）。
    fn position_in_the_registration_order(self) -> usize {
        usize::try_from(self.0)
            .expect("编号是 u32；这个包只在 usize 至少 32 位的平台上编，u32 装得进 usize")
    }

    fn to_stored_bytes(self) -> [u8; VERDICT_VECTOR_NUMBER_BYTES] {
        self.0.to_be_bytes()
    }
}

/// 存着的一串编号（判定块、结论块的值）按 4 字节大端切开；字节数不是 4 的整数倍时是 `None`。只切、不核登记过没有。
fn stored_verdict_vector_numbers(value_bytes: &[u8]) -> Option<Vec<u32>> {
    let (whole_numbers, bytes_left_over) = value_bytes.as_chunks::<VERDICT_VECTOR_NUMBER_BYTES>();
    if !bytes_left_over.is_empty() {
        return None;
    }
    Some(
        whole_numbers
            .iter()
            .map(|number_bytes| u32::from_be_bytes(*number_bytes))
            .collect(),
    )
}

/// 存着的一个编号（违例的值、按内容那张判定向量表的值）；不是正好 4 字节时是 `None`。
fn stored_verdict_vector_number(value_bytes: &[u8]) -> Option<u32> {
    <[u8; VERDICT_VECTOR_NUMBER_BYTES]>::try_from(value_bytes)
        .ok()
        .map(u32::from_be_bytes)
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
            .flat_map(|number| number.to_stored_bytes())
            .collect()
    }
}

/// 违例表里的一行：块里第几个状态违例、它的判定向量编号。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StoredViolation {
    pub offset_in_block: StateOffsetInBlock,
    pub verdict_vector_number: VerdictVectorNumber,
}

/// 恢复字旁表里一个状态占几个字节：八个字，每字 4 字节小端。
const RECOVERY_WORD_BYTES_PER_STATE: usize = RECOVERY_WORDS_PER_STATE * std::mem::size_of::<u32>();

fn recovery_words_value_bytes(words_of_each_state: &[RecoveryWordsOfState]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(words_of_each_state.len() * RECOVERY_WORD_BYTES_PER_STATE);
    for words in words_of_each_state {
        for word in words.0 {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
    }
    bytes
}

/// [`recovery_words_value_bytes`] 的逆；长度不是 `state_count` 个状态的整份交回 None。
fn recovery_words_from_value_bytes(
    bytes: &[u8],
    state_count: usize,
) -> Option<Vec<RecoveryWordsOfState>> {
    if bytes.len() != state_count * RECOVERY_WORD_BYTES_PER_STATE {
        return None;
    }
    let (states, _no_remainder) = bytes.as_chunks::<RECOVERY_WORD_BYTES_PER_STATE>();
    Some(
        states
            .iter()
            .map(|state_bytes| {
                let (word_bytes, _no_word_remainder) = state_bytes.as_chunks::<4>();
                let mut words = [0u32; RECOVERY_WORDS_PER_STATE];
                for (word, bytes_of_word) in words.iter_mut().zip(word_bytes) {
                    *word = u32::from_le_bytes(*bytes_of_word);
                }
                RecoveryWordsOfState(words)
            })
            .collect(),
    )
}

/// 点名的被抛弃根旁表里一条根占几个字节：实例代号 4 字节、checkpoint_txg 8 字节，都是小端。
const ABANDONED_ROOT_NAMED_BYTES: usize = 4 + 8;

/// 点名的被抛弃根旁表的值：每个状态先写它点名了几条（4 字节小端），再逐条写实例代号与 checkpoint_txg。
fn abandoned_roots_named_value_bytes(
    roots_of_each_state: &[Vec<AbandonedRootNamedByThePoolChecker>],
) -> Vec<u8> {
    let mut bytes = Vec::new();
    for roots in roots_of_each_state {
        let root_count = u32::try_from(roots.len()).expect("一个状态点名的被抛弃根装得进 u32");
        bytes.extend_from_slice(&root_count.to_le_bytes());
        for root in roots {
            bytes.extend_from_slice(&root.instance.0.to_le_bytes());
            bytes.extend_from_slice(&root.checkpoint_txg.0.to_le_bytes());
        }
    }
    bytes
}

/// [`abandoned_roots_named_value_bytes`] 的逆；不是恰好 `state_count` 个状态的整份（字节不够、多出）交回 None。
fn abandoned_roots_named_from_value_bytes(
    bytes: &[u8],
    state_count: usize,
) -> Option<Vec<Vec<AbandonedRootNamedByThePoolChecker>>> {
    let mut position = 0usize;
    let mut take = |length: usize| -> Option<&[u8]> {
        let taken = bytes.get(position..position.checked_add(length)?)?;
        position += length;
        Some(taken)
    };
    let mut roots_of_each_state = Vec::with_capacity(state_count);
    for _ in 0..state_count {
        let root_count =
            usize::try_from(u32::from_le_bytes(<[u8; 4]>::try_from(take(4)?).ok()?)).ok()?;
        let mut roots = Vec::new();
        for _ in 0..root_count {
            let root_bytes = take(ABANDONED_ROOT_NAMED_BYTES)?;
            roots.push(AbandonedRootNamedByThePoolChecker {
                instance: InstanceGeneration(u32::from_le_bytes(
                    <[u8; 4]>::try_from(&root_bytes[..4]).ok()?,
                )),
                checkpoint_txg: CheckpointTxg(u64::from_le_bytes(
                    <[u8; 8]>::try_from(&root_bytes[4..]).ok()?,
                )),
            });
        }
        roots_of_each_state.push(roots);
    }
    (position == bytes.len()).then_some(roots_of_each_state)
}

/// 计数向量值的头一个字节：写法换了就换它，旧写法读成不合布局。
const COUNT_VECTOR_FORMAT_NUMBER: u8 = 1;

/// 在一段字节上往前读：读不够交回 None。
struct ByteCursor<'bytes> {
    bytes: &'bytes [u8],
    position: usize,
}

impl<'bytes> ByteCursor<'bytes> {
    fn take(&mut self, length: usize) -> Option<&'bytes [u8]> {
        let taken = self
            .bytes
            .get(self.position..self.position.checked_add(length)?)?;
        self.position += length;
        Some(taken)
    }

    fn big_endian_u32(&mut self) -> Option<u32> {
        Some(u32::from_be_bytes(<[u8; 4]>::try_from(self.take(4)?).ok()?))
    }

    fn big_endian_u64(&mut self) -> Option<u64> {
        Some(u64::from_be_bytes(<[u8; 8]>::try_from(self.take(8)?).ok()?))
    }

    /// 4 字节大端的长度加那么多字节的 UTF-8。
    fn length_prefixed_text(&mut self) -> Option<String> {
        let length = usize::try_from(self.big_endian_u32()?).ok()?;
        String::from_utf8(self.take(length)?.to_vec()).ok()
    }

    fn is_at_the_end(&self) -> bool {
        self.position == self.bytes.len()
    }
}

fn push_length_prefixed_text(bytes: &mut Vec<u8>, text: &str) {
    let length = u32::try_from(text.len()).expect("计数名、卡型与驱动版本的文字不会超过 4 GiB");
    bytes.extend_from_slice(&length.to_be_bytes());
    bytes.extend_from_slice(text.as_bytes());
}

fn push_section_entry_count(bytes: &mut Vec<u8>, entry_count: usize) {
    let entry_count = u32::try_from(entry_count).expect("一块的计数向量一节不会超过 u32::MAX 条");
    bytes.extend_from_slice(&entry_count.to_be_bytes());
}

/// 读一节：条数，再逐条「名字、值」；名字要严格升序（写的时候按 `BTreeMap` 的次序写，重名或乱序就是坏字节）。
fn read_count_section<Count>(
    cursor: &mut ByteCursor<'_>,
    read_count: impl Fn(&mut ByteCursor<'_>) -> Option<Count>,
) -> Option<BTreeMap<String, Count>> {
    let entry_count = cursor.big_endian_u32()?;
    let mut section = BTreeMap::new();
    for _entry in 0..entry_count {
        let name = cursor.length_prefixed_text()?;
        if section
            .last_key_value()
            .is_some_and(|(previous_name, _count): (&String, &Count)| *previous_name >= name)
        {
            return None;
        }
        let count = read_count(cursor)?;
        section.insert(name, count);
    }
    Some(section)
}

/// 计数向量（[`BlockCountVector`]，类型在判器一侧 `crate::crash_judge_conclusions`）存进计数向量表的字节：格式号一个字节，之后三节
/// （逐条不变量、观察者、按发布分），每节 4 字节大端条数、逐条名字（4 字节大端长度 + UTF-8）与计数（每个 8 字节大端；不变量一条三个：
/// 判到、违例、不适用）。字节布局只在这里，计数向量里每一格是什么由判器一侧定。
#[must_use]
pub fn count_vector_bytes(count_vector: &BlockCountVector) -> Vec<u8> {
    let mut bytes = vec![COUNT_VECTOR_FORMAT_NUMBER];
    push_section_entry_count(&mut bytes, count_vector.per_invariant.len());
    for (invariant_name, counts) in &count_vector.per_invariant {
        push_length_prefixed_text(&mut bytes, invariant_name);
        for count in [counts.judged, counts.violated, counts.not_applicable] {
            bytes.extend_from_slice(&count.to_be_bytes());
        }
    }
    for section in [&count_vector.per_observer, &count_vector.states_by_publish] {
        push_section_entry_count(&mut bytes, section.len());
        for (name, count) in section {
            push_length_prefixed_text(&mut bytes, name);
            bytes.extend_from_slice(&count.to_be_bytes());
        }
    }
    bytes
}

/// [`count_vector_bytes`] 的逆；格式号不对、读不够、名字不是严格升序、后面多出字节，交回 None。
#[must_use]
pub fn count_vector_from_bytes(bytes: &[u8]) -> Option<BlockCountVector> {
    let mut cursor = ByteCursor { bytes, position: 0 };
    if cursor.take(1)? != [COUNT_VECTOR_FORMAT_NUMBER] {
        return None;
    }
    let per_invariant = read_count_section(&mut cursor, |cursor| {
        Some(InvariantStateCounts {
            judged: cursor.big_endian_u64()?,
            violated: cursor.big_endian_u64()?,
            not_applicable: cursor.big_endian_u64()?,
        })
    })?;
    let per_observer = read_count_section(&mut cursor, |cursor| cursor.big_endian_u64())?;
    let states_by_publish = read_count_section(&mut cursor, |cursor| cursor.big_endian_u64())?;
    cursor.is_at_the_end().then_some(BlockCountVector {
        per_invariant,
        per_observer,
        states_by_publish,
    })
}

/// 这一块在哪判的（块属性，第三轮判决 J8、M2-5）：CPU，或 GPU 连同卡型、驱动版本与判它的 GPU 判器那一组判法单元的指纹。
/// 只是出处，不进种类、不进版本（用户 2026-10-17 定：数据不与硬件、不与 CPU 还是 GPU 绑定，复用不看设备）：GPU 判器的指纹留在这里，
/// 是为了哪一版 GPU 判器后来被对拍抓出差错时，能找到它判过的块。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JudgingDevice {
    Cpu,
    Gpu {
        card_type: String,
        driver_version: String,
        /// 判这一块的 GPU 判器那一组判法单元的键（`crash_identity::JudgingUnits::GpuJudge`）的指纹。
        gpu_judge_units_fingerprint: [u8; SHA256_DIGEST_BYTES],
    },
}

impl JudgingDevice {
    /// 块属性表的值：`01` 是 CPU；`03` 之后是卡型与驱动版本（各 4 字节大端长度 + UTF-8）再接 32 字节 GPU 判器指纹。
    /// `02` 是改前 GPU 那一种（不带判器指纹）的字节，认不回来。观察者交回的镜像上池级 checker 的结论记在哪判的，用同一种写法。
    pub(crate) fn to_attribute_bytes(&self) -> Vec<u8> {
        match self {
            Self::Cpu => vec![0x01],
            Self::Gpu {
                card_type,
                driver_version,
                gpu_judge_units_fingerprint,
            } => {
                let mut bytes = vec![0x03];
                push_length_prefixed_text(&mut bytes, card_type);
                push_length_prefixed_text(&mut bytes, driver_version);
                bytes.extend_from_slice(gpu_judge_units_fingerprint);
                bytes
            }
        }
    }

    pub(crate) fn from_attribute_bytes(bytes: &[u8]) -> Option<Self> {
        let mut cursor = ByteCursor { bytes, position: 0 };
        let device = match cursor.take(1)? {
            [0x01] => Self::Cpu,
            [0x03] => Self::Gpu {
                card_type: cursor.length_prefixed_text()?,
                driver_version: cursor.length_prefixed_text()?,
                gpu_judge_units_fingerprint: cursor
                    .take(SHA256_DIGEST_BYTES)?
                    .try_into()
                    .expect("刚取的就是 SHA256_DIGEST_BYTES 个字节"),
            },
            _ => return None,
        };
        cursor.is_at_the_end().then_some(device)
    }
}

/// 存一块一种结论要给的整份：结论块、违例状态、计数向量、块属性，连同恢复字（判器给；观察者没有）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConclusionBlockRecord {
    pub block: VerdictBlock,
    pub violating_states: Vec<StateOffsetInBlock>,
    pub count_vector: BlockCountVector,
    pub judged_on: JudgingDevice,
    pub recovery_words: Option<Vec<RecoveryWordsOfState>>,
}

/// 读回的一块一种结论：结论块、按块内序号排的违例、计数向量、块属性、恢复字（存的时候没给就是 None）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredConclusionBlock {
    pub block: VerdictBlock,
    pub violations: Vec<StoredViolation>,
    pub count_vector: BlockCountVector,
    pub judged_on: JudgingDevice,
    pub recovery_words: Option<Vec<RecoveryWordsOfState>>,
    /// 每个状态上池级 checker 判 I-7.4 时违例细节里点名的被抛弃根（存的时候给了才有：
    /// [`VerdictStore::store_conclusion_block_naming_the_abandoned_roots`]；给定状态表那一路每块都给）。
    pub abandoned_roots_named_by_the_pool_checker:
        Option<Vec<Vec<AbandonedRootNamedByThePoolChecker>>>,
    /// 判定日期与跑法（块属性，存的时候由库按这个进程此刻填；不进键、不进结论的比对）。
    pub circumstances: JudgementCircumstances,
}

/// 结论块属性那一行的值：在哪判的（[`JudgingDevice::to_attribute_bytes`]，前面 4 字节大端记它的长度）、判定日期（4 字节大端长度 + UTF-8）、
/// 跑法码（1 字节）。
fn conclusion_block_attribute_bytes(
    judged_on: &JudgingDevice,
    circumstances: &JudgementCircumstances,
) -> Vec<u8> {
    let device_bytes = judged_on.to_attribute_bytes();
    let mut bytes = Vec::new();
    bytes.extend_from_slice(
        &u32::try_from(device_bytes.len())
            .expect("块属性里在哪判的那一截装得进 u32")
            .to_be_bytes(),
    );
    bytes.extend_from_slice(&device_bytes);
    push_length_prefixed_text(&mut bytes, &circumstances.judged_date);
    bytes.push(circumstances.run_kind.code());
    bytes
}

/// [`conclusion_block_attribute_bytes`] 的逆；不合布局交回 None。
fn conclusion_block_attributes_from_bytes(
    bytes: &[u8],
) -> Option<(JudgingDevice, JudgementCircumstances)> {
    let mut cursor = ByteCursor { bytes, position: 0 };
    let device_length = usize::try_from(cursor.big_endian_u32()?).ok()?;
    let judged_on = JudgingDevice::from_attribute_bytes(cursor.take(device_length)?)?;
    let judged_date = cursor.length_prefixed_text()?;
    let run_kind = RunKindOfTheJudgement::of_code(*cursor.take(1)?.first()?)?;
    cursor.is_at_the_end().then_some((
        judged_on,
        JudgementCircumstances {
            judged_date,
            run_kind,
        },
    ))
}

/// 结论种类在键里的字节：`01` 镜像判定（不分在哪判的）、`03` 观察者。种类定长一个字节，一个种类的字节不会是另一个的前缀；
/// 观察者的名字不进键（是哪一个观察者由后面那 32 字节的版本担）。`02` 是改前「CPU 判器」那一种的字节，认不回来：
/// 那一版的记录按设备分种类，不再读（用户 2026-10-17 定数据不与设备绑定；改键让旧库作废不算代价）。
/// 结论种类在键里占的字节数（[`conclusion_kind_key_bytes`] 每一种都是这么长）：照判不入库按它定版本那一截的起点
/// （[`version_field_starts_in_the_key`]）。
const CONCLUSION_KIND_KEY_BYTES: usize = 1;

fn conclusion_kind_key_bytes(kind: &ConclusionKind) -> Vec<u8> {
    match kind {
        ConclusionKind::ImageJudgement => vec![0x01],
        ConclusionKind::Observer => vec![0x03],
    }
}

/// [`conclusion_kind_key_bytes`] 的逆：交回种类与它后面剩下的字节；认不回来（含改前 CPU 判器那一种的 `02`）交回 None。
fn conclusion_kind_from_key_bytes(bytes: &[u8]) -> Option<(ConclusionKind, &[u8])> {
    let (tag, after_tag) = bytes.split_first()?;
    match tag {
        0x01 => Some((ConclusionKind::ImageJudgement, after_tag)),
        0x03 => Some((ConclusionKind::Observer, after_tag)),
        _ => None,
    }
}

/// 库的布局。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerdictStoreLayout {
    /// 这一套之前的库：没有布局行，只有按（块键，判法单元指纹）存的判定块表与违例表。
    BeforeConclusionKinds,
    /// 有布局行：另按（块键，结论种类，版本）存结论，恢复字、计数向量、块属性随块存。
    ConclusionKinds,
}

/// 布局行的值：块键只由内容定、有结论种类那几张表、观察者那一种结论不带名字、判定向量编号 4 字节的布局号（2、3、4 是编号还是一个字节的
/// 那几套，开库就拒、不迁移）。
const CONCLUSION_KINDS_LAYOUT_NUMBER: u32 = 5;

/// 判定向量编号还是一个字节（块值每个状态一个字节、违例的值一个字节、两张判定向量表的编号一个字节）的那一套的布局号：开库就拒、不迁移
/// （按 4 字节读一行也读不对）。
const LAYOUT_NUMBER_WITH_ONE_BYTE_VERDICT_VECTOR_NUMBERS: u32 = 4;

/// 观察者那一种结论的键里还带名字的那一套的布局号：开库就拒、不迁移（键变了，旧行一行也复用不上；编号也还是一个字节）。
const LAYOUT_NUMBER_WITH_OBSERVER_NAMES_IN_THE_CONCLUSION_KIND: u32 = 3;

/// 块键里还带节点名（设点文件、崩溃点名、项名）的那一套的布局号：开库就拒、不迁移（编号也还是一个字节）。
const LAYOUT_NUMBER_WITH_NODE_NAMES_IN_THE_BLOCK_KEY: u32 = 2;

/// 读布局行。布局号 2、3、4 的库在这里就拒（[`VerdictStoreError::LibraryLayoutWithOneByteVerdictVectorNumbers`]），调用方还没读判定向量表。
fn read_library_layout(
    database: &rocksdb::DB,
    directory: &Path,
) -> Result<VerdictStoreLayout, VerdictStoreError> {
    let key_bytes = VerdictStoreKey::LibraryLayout.to_key_bytes();
    let Some(layout_bytes) = database
        .get(&key_bytes.0)
        .map_err(VerdictStoreError::RocksDatabase)?
    else {
        return Ok(VerdictStoreLayout::BeforeConclusionKinds);
    };
    let layout_number = <[u8; 4]>::try_from(layout_bytes.as_slice())
        .ok()
        .map(u32::from_be_bytes);
    match layout_number {
        Some(CONCLUSION_KINDS_LAYOUT_NUMBER) => Ok(VerdictStoreLayout::ConclusionKinds),
        Some(
            layout_number @ (LAYOUT_NUMBER_WITH_NODE_NAMES_IN_THE_BLOCK_KEY
            | LAYOUT_NUMBER_WITH_OBSERVER_NAMES_IN_THE_CONCLUSION_KIND
            | LAYOUT_NUMBER_WITH_ONE_BYTE_VERDICT_VECTOR_NUMBERS),
        ) => Err(
            VerdictStoreError::LibraryLayoutWithOneByteVerdictVectorNumbers {
                directory: directory.to_path_buf(),
                layout_number,
            },
        ),
        Some(_) | None => Err(stored_bytes_mismatch(
            VerdictStoreTable::LibraryLayout,
            &key_bytes.0,
            "布局号认不得",
        )),
    }
}

/// 库里的各张表；键的第一个字节是表标签。
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
    /// 判法单元组成清单表：键是判法单元指纹，值是这一把键的组成清单（`crash_identity::JudgingUnitsKey::composition_text`）：
    /// 库对不上时照它点名变了的单元。
    JudgingUnitsCompositions,
    /// 库布局：一行，值是布局号（[`VerdictStoreLayout`]）。
    LibraryLayout,
    /// 结论块：键是块键 + 结论种类 + 版本，值是每个状态一个判定向量编号。
    ConclusionBlocks,
    /// 结论违例：键是块键 + 结论种类 + 版本 + 块内序号，值是那个状态的判定向量编号。
    ConclusionViolations,
    /// 恢复字旁表：键是块键 + 结论种类 + 版本（与结论块同形：几种结论共用一把版本时各存各的），值是每个状态八个字。
    RecoveryWords,
    /// 点名的被抛弃根旁表（用户 2026-10-14 弹窗定「GPU 判器加细节输出」）：键是块键 + 结论种类 + 版本（与恢复字旁表同形），
    /// 值是每个状态上池级 checker 判 I-7.4 时违例细节里点名的被抛弃根（[`abandoned_roots_named_value_bytes`]）。与结论块同一批写、一起读回。
    AbandonedRootsNamedByThePoolChecker,
    /// 计数向量：键是块键 + 结论种类 + 版本，值是 [`count_vector_bytes`]。
    ConclusionCountVectors,
    /// 块属性：键是块键 + 结论种类 + 版本，值是在哪判的（[`JudgingDevice`]）。
    ConclusionBlockAttributes,
    /// 录入段写下的流程（`crash_amplification::RecordedFlow`）：键是它的摘要，值是它的字节。判段只从这里读流程，不收内存里的流程。
    RecordedFlows,
    /// 录入段一趟的清单：键是录入键（[`RecordingKey`]：录入函数单元指纹与录入输入的摘要，不带流名），值是这一趟录下的几样各自的摘要
    /// （`crash_amplification::RecordedRunManifest` 或 `crash_library::ItemManifest`）。判段照同一份输入现算录入键找到它。
    RecordedRunManifests,
    /// 录入段打包好的 GPU 判表（`crash_judge_dispatch::PackedJudgeTables`）：键是它的摘要，值是它的字节。判段在卡上判时只从这里读。
    PackedJudgeTables,
    /// 给定状态表（`crash_amplification::RecordedGivenStatesTable`）：键是它的摘要，值是它的字节。引擎先写进来、再从这里读回判。
    GivenStatesTables,
    /// 分片进度（C790）：键是段的内容摘要与判器（[`SlicedSegmentProgressKey`]：不带流名、段号），值是 [`SlicedSegmentProgress`]。只是加速，真相以块为准。
    SlicedSegmentProgress,
    /// 观察者两段的录入段（用户 2026-10-13 定「观察者改走两段」）：一条流的崩溃镜像共用的基底（基础镜像与枚举写表），键是它的摘要，
    /// 值是 `crash_amplification` 里观察者基底的字节。观察者交回的镜像在判段照它重建，不收录入段内存里的流。
    ObserverFoundations,
    /// 观察者交回镜像的那一种：一张恢复镜像上观察者看出的那一份（红项、计数、观察者真跑实现在镜像上发的写），键是按恢复镜像字节去重的键。
    /// 录入段写、判段读；同一键只看一次。
    ObservedImages,
    /// 观察者录入段一块一份：块里每个状态落在哪一份观察结果上（去重之后的清单与每个状态的下标）、按发布分的态数。
    /// 键是块键 + 观察者种类 + 观察者版本。
    ObserverBlocks,
    /// 判段对观察者交回的一张镜像跑池级 checker 的结论（判到位、违例位、在哪判的）：键是去重键 + 判器种类 + 判器版本，同一键只判一次。
    ObservedImageJudgements,
    /// 对拍的结论：键是 [`PairingConclusionKey`]，值是 [`StoredPairingConclusion`] 的字节。同一键第二趟复用。
    PairingConclusions,
    /// 观察者去重的读集见证（用户 2026-10-16 定「恢复报告分桶 + 读集见证」），一条见证一行：键是桶键（观察者、它另读的输入、恢复报告的摘要）
    /// 与见证摘要，值是 [`ObserverReadSetWitnessRow`] 的读集摘要与镜像键。录入段写、录入段按桶键前缀读。
    ObserverReadSetWitnessRows,
    /// 观察者真跑一次读过的段，一组一行：键是读集摘要（这份字节的 SHA-256），值是读集的字节（`crash_amplification` 写、读、验摘要）。
    /// 同一组段全库只这一行，见证行按摘要引它。
    ObserverReadSets,
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
            Self::JudgingUnitsCompositions => 0x09,
            Self::LibraryLayout => 0x00,
            Self::ConclusionBlocks => 0x0A,
            Self::ConclusionViolations => 0x0B,
            Self::RecoveryWords => 0x0C,
            Self::ConclusionCountVectors => 0x0D,
            Self::ConclusionBlockAttributes => 0x0E,
            Self::RecordedFlows => 0x10,
            Self::RecordedRunManifests => 0x11,
            Self::PackedJudgeTables => 0x12,
            Self::GivenStatesTables => 0x13,
            Self::SlicedSegmentProgress => 0x14,
            Self::ObserverFoundations => 0x15,
            Self::ObservedImages => 0x16,
            Self::ObserverBlocks => 0x17,
            Self::ObservedImageJudgements => 0x18,
            Self::AbandonedRootsNamedByThePoolChecker => 0x19,
            Self::PairingConclusions => 0x1A,
            Self::ObserverReadSetWitnessRows => 0x1C,
            Self::ObserverReadSets => 0x1D,
        }
    }

    /// [`Self::key_tag`] 的逆：一个键的第一个字节是哪张表的标签；不是任何一张表的标签时是 `None`。
    fn of_key_tag(key_tag: u8) -> Option<Self> {
        match key_tag {
            0x00 => Some(Self::LibraryLayout),
            0x01 => Some(Self::VerdictBlocks),
            0x02 => Some(Self::VerdictVectorNumberByContent),
            0x03 => Some(Self::VerdictVectorContentByNumber),
            0x04 => Some(Self::Violations),
            0x05 => Some(Self::RecordedBlocks),
            0x06 => Some(Self::PathIndex),
            0x07 => Some(Self::FlowFacts),
            0x08 => Some(Self::VerifierVerdicts),
            0x09 => Some(Self::JudgingUnitsCompositions),
            0x0A => Some(Self::ConclusionBlocks),
            0x0B => Some(Self::ConclusionViolations),
            0x0C => Some(Self::RecoveryWords),
            0x0D => Some(Self::ConclusionCountVectors),
            0x0E => Some(Self::ConclusionBlockAttributes),
            0x10 => Some(Self::RecordedFlows),
            0x11 => Some(Self::RecordedRunManifests),
            0x12 => Some(Self::PackedJudgeTables),
            0x13 => Some(Self::GivenStatesTables),
            0x14 => Some(Self::SlicedSegmentProgress),
            0x15 => Some(Self::ObserverFoundations),
            0x16 => Some(Self::ObservedImages),
            0x17 => Some(Self::ObserverBlocks),
            0x18 => Some(Self::ObservedImageJudgements),
            0x19 => Some(Self::AbandonedRootsNamedByThePoolChecker),
            0x1A => Some(Self::PairingConclusions),
            0x1C => Some(Self::ObserverReadSetWitnessRows),
            0x1D => Some(Self::ObserverReadSets),
            _ => None,
        }
    }
}

/// 表标签占一个字节，键里它之后的各截从这里起算。
const KEY_TAG_BYTES: usize = 1;

/// 判定块、违例、核对表的键里判法单元指纹（核对表是核对代码摘要）的起点：表标签、块键之后。
const VERSION_START_AFTER_THE_BLOCK_KEY: usize = KEY_TAG_BYTES + BLOCK_KEY_BYTES;

/// 结论块、结论违例、恢复字、计数向量、块属性、点名的被抛弃根、观察者录入块的键里版本的起点：表标签、块键、结论种类之后。
const VERSION_START_AFTER_THE_BLOCK_KEY_AND_THE_CONCLUSION_KIND: usize =
    KEY_TAG_BYTES + BLOCK_KEY_BYTES + CONCLUSION_KIND_KEY_BYTES;

/// 镜像上池级 checker 结论的键里判器版本的起点：表标签、去重键、判器种类之后。
const VERSION_START_AFTER_THE_DEDUPLICATION_KEY_AND_THE_CONCLUSION_KIND: usize =
    KEY_TAG_BYTES + SHA256_DIGEST_BYTES + CONCLUSION_KIND_KEY_BYTES;

/// 判法单元组成清单的键里判法单元指纹的起点：表标签之后。
const VERSION_START_OF_A_JUDGING_UNITS_COMPOSITION: usize = KEY_TAG_BYTES;

/// 对拍结论的键里两边判法单元指纹的起点：表标签、输入摘要之后是一边，再隔一把指纹是另一边。
const VERSION_STARTS_OF_A_PAIRING_CONCLUSION: [usize; 2] = [
    KEY_TAG_BYTES + SHA256_DIGEST_BYTES,
    KEY_TAG_BYTES + 2 * SHA256_DIGEST_BYTES,
];

/// 分片进度键里判器标记那一个字节的位置：表标签、段的内容摘要之后。
const SLICED_PROGRESS_JUDGE_MARKER_POSITION: usize = KEY_TAG_BYTES + SHA256_DIGEST_BYTES;

/// 分片进度键里判法单元指纹（判器标记是指纹那一套）的起点：判器标记之后。
const VERSION_START_OF_SLICED_PROGRESS_UNDER_A_FINGERPRINT: usize =
    SLICED_PROGRESS_JUDGE_MARKER_POSITION + 1;

/// 分片进度键里结论版本（判器标记是结论种类那一套）的起点：判器标记、结论种类之后。
const VERSION_START_OF_SLICED_PROGRESS_UNDER_A_CONCLUSION_KIND: usize =
    SLICED_PROGRESS_JUDGE_MARKER_POSITION + 1 + CONCLUSION_KIND_KEY_BYTES;

/// 一个键里装判法单元指纹或结论版本（各 32 字节）的那几截的起点：按表标签取（分片进度表另看判器标记），与
/// [`VerdictStoreKey::to_key_bytes`] 的拼法一处对一处。照判不入库只比这几截（[`RowsKeptOutOfTheLibrary::keeps_out`]）：
/// 块键里的摘要、路径原文、判定向量的字节、录入键这类别处碰巧与一把版本逐字节相同，不算带着它。没有版本那一截的表、认不得的标签交空。
fn version_field_starts_in_the_key(key_bytes: &[u8]) -> &'static [usize] {
    let Some(table) = key_bytes
        .first()
        .copied()
        .and_then(VerdictStoreTable::of_key_tag)
    else {
        return &[];
    };
    match table {
        VerdictStoreTable::VerdictBlocks
        | VerdictStoreTable::Violations
        | VerdictStoreTable::VerifierVerdicts => &[VERSION_START_AFTER_THE_BLOCK_KEY],
        VerdictStoreTable::ConclusionBlocks
        | VerdictStoreTable::ConclusionViolations
        | VerdictStoreTable::RecoveryWords
        | VerdictStoreTable::ConclusionCountVectors
        | VerdictStoreTable::ConclusionBlockAttributes
        | VerdictStoreTable::AbandonedRootsNamedByThePoolChecker
        | VerdictStoreTable::ObserverBlocks => {
            &[VERSION_START_AFTER_THE_BLOCK_KEY_AND_THE_CONCLUSION_KIND]
        }
        VerdictStoreTable::ObservedImageJudgements => {
            &[VERSION_START_AFTER_THE_DEDUPLICATION_KEY_AND_THE_CONCLUSION_KIND]
        }
        VerdictStoreTable::JudgingUnitsCompositions => {
            &[VERSION_START_OF_A_JUDGING_UNITS_COMPOSITION]
        }
        VerdictStoreTable::PairingConclusions => &VERSION_STARTS_OF_A_PAIRING_CONCLUSION,
        VerdictStoreTable::SlicedSegmentProgress => {
            match key_bytes
                .get(SLICED_PROGRESS_JUDGE_MARKER_POSITION)
                .copied()
            {
                Some(SLICED_PROGRESS_JUDGE_MARKER_OF_THE_JUDGING_UNITS_FINGERPRINT) => {
                    &[VERSION_START_OF_SLICED_PROGRESS_UNDER_A_FINGERPRINT]
                }
                Some(SLICED_PROGRESS_JUDGE_MARKER_OF_A_CONCLUSION_KIND) => {
                    &[VERSION_START_OF_SLICED_PROGRESS_UNDER_A_CONCLUSION_KIND]
                }
                Some(_) | None => &[],
            }
        }
        VerdictStoreTable::LibraryLayout
        | VerdictStoreTable::VerdictVectorNumberByContent
        | VerdictStoreTable::VerdictVectorContentByNumber
        | VerdictStoreTable::RecordedBlocks
        | VerdictStoreTable::PathIndex
        | VerdictStoreTable::FlowFacts
        | VerdictStoreTable::RecordedFlows
        | VerdictStoreTable::RecordedRunManifests
        | VerdictStoreTable::PackedJudgeTables
        | VerdictStoreTable::GivenStatesTables
        | VerdictStoreTable::ObserverFoundations
        | VerdictStoreTable::ObservedImages
        | VerdictStoreTable::ObserverReadSetWitnessRows
        | VerdictStoreTable::ObserverReadSets => &[],
    }
}

/// 库里的一个键，或按前缀扫一段键时的前缀。键的字节只在 [`VerdictStoreKey::to_key_bytes`] 一处拼。
enum VerdictStoreKey<'key> {
    /// 判定块表里一块的键：块键 + 判法单元指纹。
    VerdictBlock(&'key VerdictBlockKey, &'key JudgingUnitsFingerprint),
    VerdictVectorNumberByContent(&'key EncodedVerdictVector),
    VerdictVectorContentByNumber(VerdictVectorNumber),
    Violation(
        &'key VerdictBlockKey,
        &'key JudgingUnitsFingerprint,
        StateOffsetInBlock,
    ),
    /// 违例表里这一块在这一判法单元指纹下的全部键共有的前缀。
    ViolationsOfBlockPrefix(&'key VerdictBlockKey, &'key JudgingUnitsFingerprint),
    /// 一张表全部键共有的前缀（只有表标签）。
    WholeTablePrefix(VerdictStoreTable),
    RecordedBlock(&'key VerdictBlockKey),
    /// 路径索引表里一条路径（或一段前缀）的键。
    Path(&'key str),
    /// 事实表里一条流的键：事实表摘要。
    FlowFacts(&'key [u8; 32]),
    /// 录入段写下的流程的键：它的摘要。
    RecordedFlow(&'key [u8; 32]),
    /// 录入段一趟清单的键：录入键。
    RecordedRunManifest(&'key RecordingKey),
    /// 录入段打包好的 GPU 判表的键：它的摘要。
    PackedJudgeTables(&'key [u8; 32]),
    /// 给定状态表的键：它的摘要。
    GivenStatesTable(&'key [u8; 32]),
    /// 核对表里一块的键：块键 + 核对代码摘要。
    VerifierBlock(&'key VerdictBlockKey, &'key [u8; 32]),
    /// 判法单元组成清单表里一把键的键：判法单元指纹。
    JudgingUnitsComposition(&'key [u8; SHA256_DIGEST_BYTES]),
    /// 对拍结论表里一条的键。
    PairingConclusion(&'key PairingConclusionKey),
    /// 布局行的键（只有表标签）。
    LibraryLayout,
    /// 结论块表里一块一种结论的键：块键 + 结论种类 + 版本。
    ConclusionBlock(
        &'key VerdictBlockKey,
        &'key ConclusionKind,
        &'key ConclusionVersion,
    ),
    ConclusionViolation(
        &'key VerdictBlockKey,
        &'key ConclusionKind,
        &'key ConclusionVersion,
        StateOffsetInBlock,
    ),
    /// 结论违例表里这一块这一种结论这一版本下的全部键共有的前缀。
    ConclusionViolationsOfBlockPrefix(
        &'key VerdictBlockKey,
        &'key ConclusionKind,
        &'key ConclusionVersion,
    ),
    /// 恢复字旁表里一块一种结论一版本的键。
    RecoveryWords(
        &'key VerdictBlockKey,
        &'key ConclusionKind,
        &'key ConclusionVersion,
    ),
    /// 点名的被抛弃根旁表里一块一种结论一版本的键。
    AbandonedRootsNamedByThePoolChecker(
        &'key VerdictBlockKey,
        &'key ConclusionKind,
        &'key ConclusionVersion,
    ),
    ConclusionCountVector(
        &'key VerdictBlockKey,
        &'key ConclusionKind,
        &'key ConclusionVersion,
    ),
    ConclusionBlockAttributes(
        &'key VerdictBlockKey,
        &'key ConclusionKind,
        &'key ConclusionVersion,
    ),
    /// 一张按块键排的表里这一块的全部行共有的前缀：表标签 + 块键（块键带名字长度、以块起点收尾，不会是别的块键的前缀）。
    RowsOfABlockPrefix(VerdictStoreTable, &'key VerdictBlockKey),
    /// 分片进度表里一条记录的键。
    SlicedSegmentProgress(&'key SlicedSegmentProgressKey),
    /// 观察者基底的键：它的摘要。
    ObserverFoundation(&'key [u8; 32]),
    /// 观察者看过的一张恢复镜像的键：去重键。
    ObservedImage(&'key [u8; 32]),
    /// 观察者去重一个桶里一条读集见证行的键：桶键 + 见证摘要。
    ObserverReadSetWitnessRow(&'key [u8; 32], &'key [u8; 32]),
    /// 一个桶的全部读集见证行共有的前缀：表标签 + 桶键（桶键定长 32 字节，不会是别的桶键的前缀）。
    ObserverReadSetWitnessRowsOfABucketPrefix(&'key [u8; 32]),
    /// 一组读集行的键：读集摘要。
    ObserverReadSet(&'key [u8; 32]),
    /// 观察者录入段一块的键：块键 + 观察者种类 + 观察者版本。
    ObserverBlock(
        &'key VerdictBlockKey,
        &'key ConclusionKind,
        &'key ConclusionVersion,
    ),
    /// 观察者交回的一张镜像上池级 checker 结论的键：去重键 + 判器种类 + 判器版本。
    ObservedImageJudgement(
        &'key [u8; 32],
        &'key ConclusionKind,
        &'key ConclusionVersion,
    ),
}

/// 按 [`VerdictStoreKey`] 拼出来的键字节。
#[derive(Clone, Debug, PartialEq, Eq)]
struct VerdictStoreKeyBytes(Vec<u8>);

impl VerdictStoreKey<'_> {
    fn to_key_bytes(&self) -> VerdictStoreKeyBytes {
        let (table, key_bytes_after_tag) = match self {
            Self::VerdictBlock(block_key, version) => (
                VerdictStoreTable::VerdictBlocks,
                block_key.to_block_key_bytes_with_version(version),
            ),
            Self::ViolationsOfBlockPrefix(block_key, version) => (
                VerdictStoreTable::Violations,
                block_key.to_block_key_bytes_with_version(version),
            ),
            Self::VerdictVectorNumberByContent(verdict_vector) => (
                VerdictStoreTable::VerdictVectorNumberByContent,
                verdict_vector.0.clone(),
            ),
            Self::VerdictVectorContentByNumber(number) => (
                VerdictStoreTable::VerdictVectorContentByNumber,
                number.to_stored_bytes().to_vec(),
            ),
            Self::Violation(block_key, version, offset) => {
                let mut key_bytes_after_tag = block_key.to_block_key_bytes_with_version(version);
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
            Self::RecordedFlow(digest) => (VerdictStoreTable::RecordedFlows, digest.to_vec()),
            Self::RecordedRunManifest(recording_key) => (
                VerdictStoreTable::RecordedRunManifests,
                recording_key.0.to_vec(),
            ),
            Self::PackedJudgeTables(digest) => {
                (VerdictStoreTable::PackedJudgeTables, digest.to_vec())
            }
            Self::GivenStatesTable(digest) => {
                (VerdictStoreTable::GivenStatesTables, digest.to_vec())
            }
            Self::VerifierBlock(block_key, verifier_digest) => {
                let mut key_bytes_after_tag = block_key.to_block_key_bytes();
                key_bytes_after_tag.extend_from_slice(&verifier_digest[..]);
                (VerdictStoreTable::VerifierVerdicts, key_bytes_after_tag)
            }
            Self::JudgingUnitsComposition(fingerprint) => (
                VerdictStoreTable::JudgingUnitsCompositions,
                fingerprint.to_vec(),
            ),
            Self::PairingConclusion(pairing_key) => {
                let mut key_bytes_after_tag = pairing_key.inputs_digest.to_vec();
                key_bytes_after_tag.extend_from_slice(&pairing_key.one_side_judging_units);
                key_bytes_after_tag.extend_from_slice(&pairing_key.other_side_judging_units);
                (VerdictStoreTable::PairingConclusions, key_bytes_after_tag)
            }
            Self::LibraryLayout => (VerdictStoreTable::LibraryLayout, Vec::new()),
            Self::ConclusionBlock(block_key, kind, conclusion_version) => (
                VerdictStoreTable::ConclusionBlocks,
                block_key.to_block_key_bytes_with_conclusion(kind, conclusion_version),
            ),
            Self::ConclusionViolation(block_key, kind, conclusion_version, offset) => {
                let mut key_bytes_after_tag =
                    block_key.to_block_key_bytes_with_conclusion(kind, conclusion_version);
                key_bytes_after_tag.extend_from_slice(&offset.0.to_be_bytes());
                (VerdictStoreTable::ConclusionViolations, key_bytes_after_tag)
            }
            Self::ConclusionViolationsOfBlockPrefix(block_key, kind, conclusion_version) => (
                VerdictStoreTable::ConclusionViolations,
                block_key.to_block_key_bytes_with_conclusion(kind, conclusion_version),
            ),
            Self::RecoveryWords(block_key, kind, conclusion_version) => (
                VerdictStoreTable::RecoveryWords,
                block_key.to_block_key_bytes_with_conclusion(kind, conclusion_version),
            ),
            Self::AbandonedRootsNamedByThePoolChecker(block_key, kind, conclusion_version) => (
                VerdictStoreTable::AbandonedRootsNamedByThePoolChecker,
                block_key.to_block_key_bytes_with_conclusion(kind, conclusion_version),
            ),
            Self::ConclusionCountVector(block_key, kind, conclusion_version) => (
                VerdictStoreTable::ConclusionCountVectors,
                block_key.to_block_key_bytes_with_conclusion(kind, conclusion_version),
            ),
            Self::ConclusionBlockAttributes(block_key, kind, conclusion_version) => (
                VerdictStoreTable::ConclusionBlockAttributes,
                block_key.to_block_key_bytes_with_conclusion(kind, conclusion_version),
            ),
            Self::RowsOfABlockPrefix(table, block_key) => (*table, block_key.to_block_key_bytes()),
            Self::SlicedSegmentProgress(progress_key) => (
                VerdictStoreTable::SlicedSegmentProgress,
                progress_key.to_key_bytes_after_the_tag(),
            ),
            Self::ObserverFoundation(digest) => {
                (VerdictStoreTable::ObserverFoundations, digest.to_vec())
            }
            Self::ObservedImage(deduplication_key) => (
                VerdictStoreTable::ObservedImages,
                deduplication_key.to_vec(),
            ),
            Self::ObserverReadSetWitnessRow(bucket_key, witness_digest) => {
                let mut key_bytes_after_tag = bucket_key.to_vec();
                key_bytes_after_tag.extend_from_slice(&witness_digest[..]);
                (
                    VerdictStoreTable::ObserverReadSetWitnessRows,
                    key_bytes_after_tag,
                )
            }
            Self::ObserverReadSetWitnessRowsOfABucketPrefix(bucket_key) => (
                VerdictStoreTable::ObserverReadSetWitnessRows,
                bucket_key.to_vec(),
            ),
            Self::ObserverReadSet(read_set_digest) => (
                VerdictStoreTable::ObserverReadSets,
                read_set_digest.to_vec(),
            ),
            Self::ObserverBlock(block_key, kind, conclusion_version) => (
                VerdictStoreTable::ObserverBlocks,
                block_key.to_block_key_bytes_with_conclusion(kind, conclusion_version),
            ),
            Self::ObservedImageJudgement(deduplication_key, kind, conclusion_version) => {
                let mut key_bytes_after_tag = deduplication_key.to_vec();
                key_bytes_after_tag.extend_from_slice(&conclusion_kind_key_bytes(kind));
                key_bytes_after_tag.extend_from_slice(&conclusion_version.0);
                (
                    VerdictStoreTable::ObservedImageJudgements,
                    key_bytes_after_tag,
                )
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
    /// 这个块键下已经存着逐字节相同的块与相同的违例，什么都没写（续跑时重写同一块走这一格；同一块先后在两张卡上判出同一份结论也走这一格，
    /// 块属性留先存的那一份）。
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
    /// 2^32 种判定向量都登记过了、这是再多的一种：编号是 u32（用户 2026-10-15 定的宽度，不再加宽），装不下；返回之前什么都没写。
    EveryU32VerdictVectorNumberIsAlreadyRegistered,
    /// 判定向量编号还是一个字节的那几套布局（布局号 2、3、4，`layout_number` 是读到的那一个）：三种开法一律拒、不迁移，判定向量表一行都没读；
    /// 换一个新目录建库（一趟清空复用的全量）。
    LibraryLayoutWithOneByteVerdictVectorNumbers {
        directory: PathBuf,
        layout_number: u32,
    },
    /// 这个块键下已经存着不同的块或不同的违例：同一份输入、同一份计划下同一块判出两样，是覆盖、拒绝还是另记，没定，
    /// 第一版不支持；返回之前什么都没写。
    DifferentVerdictsUnderAStoredBlockKeyNotDecidedInTheFirstVersion { block_key: VerdictBlockKey },
    /// 这一套之前的库（没有布局行）：按结论种类存取与 [`VerdictStore::open_existing_with_conclusion_kinds`] 一律拒绝、不迁移；
    /// 换一个新目录建库（一趟清空复用的全量）。
    LibraryLayoutPredatesConclusionKinds { directory: PathBuf },
    /// 这个（块键，结论种类，版本）下已经存着不同的结论（块、违例、计数向量或恢复字不同）：覆盖、拒绝还是另记没定，第一版不支持；
    /// 返回之前什么都没写。结论相同、只有块属性（在哪判的）不同的不是错：留库里先存的那一份，什么都不写
    /// （[`VerdictBlockStoring::AlreadyStoredIdentically`]）。
    DifferentConclusionUnderAStoredKeyNotDecidedInTheFirstVersion {
        block_key: VerdictBlockKey,
        kind: ConclusionKind,
    },
    /// 观察者见证那一批被测试开关拒了（[`VerdictStore::refuse_the_observer_witness_batches_after`]，只有测试打开）：
    /// 当作那一批落盘之前进程就被杀掉，这一批一行都没写。
    ObserverWitnessBatchRefusedByTheTestSwitch,
}

/// 这个进程里写进库的同步批的账（[`VerdictStore::synced_write_tally`]）：写了几批、几个字节。字节数是 RocksDB 批自己的字节数
/// （`WriteBatch::size_in_bytes`：批头 12 字节，每行一个类型字节、键与值各一个变长长度前缀、键与值的字节），照判不入库的行也算在里面。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SyncedWriteTally {
    pub batches_written: u64,
    pub bytes_written: u64,
}

/// 观察者去重一个桶里的一条读集见证行（键是桶键与见证摘要）：见证读的那组段的摘要（读集行的键）、它看出的那张镜像的去重键。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObserverReadSetWitnessRow {
    pub witness_digest: [u8; SHA256_DIGEST_BYTES],
    pub read_set_digest: [u8; SHA256_DIGEST_BYTES],
    pub observed_image_key: [u8; SHA256_DIGEST_BYTES],
}

/// 读集见证行的值：读集摘要 ‖ 镜像的去重键。
const OBSERVER_READ_SET_WITNESS_ROW_VALUE_BYTES: usize = 2 * SHA256_DIGEST_BYTES;

/// 观察者见证那几批的测试开关不拒的时候的值（[`VerdictStore::refuse_the_observer_witness_batches_after`]）。
const OBSERVER_WITNESS_BATCHES_ARE_NEVER_REFUSED: u64 = u64::MAX;

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
        let is_the_next_number =
            stored_verdict_vector_number(&key_bytes[1..]).is_some_and(|stored_number| {
                VerdictVectorNumber::next_after_registered_count(verdict_vector_by_number.len())
                    == Some(VerdictVectorNumber(stored_number))
            });
        if !is_the_next_number {
            return Err(stored_bytes_mismatch(
                by_number_table,
                &key_bytes,
                "按编号的判定向量表里编号不是从 0 起连续的 4 字节",
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
        let Some(number) = stored_verdict_vector_number(&number_bytes).map(VerdictVectorNumber)
        else {
            return Err(stored_bytes_mismatch(
                by_content_table,
                &key_bytes,
                "按内容的判定向量表里值不是 4 字节的编号",
            ));
        };
        let agrees_with_the_number_table = verdict_vector_by_number
            .get(number.position_in_the_registration_order())
            .is_some_and(|verdict_vector| verdict_vector.0 == content);
        if !agrees_with_the_number_table {
            return Err(stored_bytes_mismatch(
                by_content_table,
                &key_bytes,
                "按内容的判定向量表与按编号的那张对不上",
            ));
        }
        verdict_vector_number_by_content.insert(content.to_vec(), number);
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

/// 一块一种结论在库里的全部行（结论块、违例、计数向量、块属性，给了恢复字的连同恢复字）放进一批。存块与按计划导入都经它，行的写法只在这一处。
fn put_conclusion_rows_into_the_batch(
    batch: &mut rocksdb::WriteBatch,
    block_key: &VerdictBlockKey,
    kind: &ConclusionKind,
    conclusion_version: &ConclusionVersion,
    conclusion: &StoredConclusionBlock,
) {
    batch.put(
        VerdictStoreKey::ConclusionBlock(block_key, kind, conclusion_version)
            .to_key_bytes()
            .0,
        conclusion.block.to_block_value_bytes(),
    );
    for violation in &conclusion.violations {
        batch.put(
            VerdictStoreKey::ConclusionViolation(
                block_key,
                kind,
                conclusion_version,
                violation.offset_in_block,
            )
            .to_key_bytes()
            .0,
            violation.verdict_vector_number.to_stored_bytes(),
        );
    }
    batch.put(
        VerdictStoreKey::ConclusionCountVector(block_key, kind, conclusion_version)
            .to_key_bytes()
            .0,
        count_vector_bytes(&conclusion.count_vector),
    );
    batch.put(
        VerdictStoreKey::ConclusionBlockAttributes(block_key, kind, conclusion_version)
            .to_key_bytes()
            .0,
        conclusion_block_attribute_bytes(&conclusion.judged_on, &conclusion.circumstances),
    );
    if let Some(recovery_words) = &conclusion.recovery_words {
        batch.put(
            VerdictStoreKey::RecoveryWords(block_key, kind, conclusion_version)
                .to_key_bytes()
                .0,
            recovery_words_value_bytes(recovery_words),
        );
    }
    if let Some(roots_of_each_state) = &conclusion.abandoned_roots_named_by_the_pool_checker {
        batch.put(
            VerdictStoreKey::AbandonedRootsNamedByThePoolChecker(
                block_key,
                kind,
                conclusion_version,
            )
            .to_key_bytes()
            .0,
            abandoned_roots_named_value_bytes(roots_of_each_state),
        );
    }
}

/// 要存的结论与库里同键那一份比出来的结局（结论不同是错误，见 [`VerdictStore::stored_conclusion_compared_with`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StoredConclusionComparison {
    NotStored,
    StoredIdentically,
    /// 块、违例、计数向量、恢复字逐项相同，只有在哪判的（[`JudgingDevice`]）不同：同一张表先后分到两张卡、两台机器上判就是这样，
    /// 结论算已存、留库里先存的那一份块属性，什么都不写。
    StoredIdenticallyOnAnotherJudgingDevice,
}

/// 按计划导入点名的键（[`VerdictStore::import_the_planned_rows_from`]）：这一趟这条流要从别的机器取回的几样，别的一行都不看。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlannedImportKeys {
    /// 结论：一项是一块一种结论一把版本。
    pub conclusions: Vec<(VerdictBlockKey, ConclusionKind, ConclusionVersion)>,
    /// 路径索引里的路径。
    pub paths: Vec<String>,
    /// 核对表的行：块键连同核对代码摘要。导过来的核对行点名的事实表（值的头 32 字节）随它们一起导。
    pub verifier_blocks: Vec<(VerdictBlockKey, [u8; SHA256_DIGEST_BYTES])>,
}

/// 按计划导入的账：点名的结论各落在哪一格（写了、本库已有且相同而跳过、对方库里没有），路径、核对行与事实行各写了几行，
/// 一共写了几个键值行（连同新登记的判定向量那两张表）、同步写了几批。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PlannedImportTally {
    pub conclusion_blocks_written: u64,
    pub conclusion_blocks_already_identical: u64,
    pub conclusion_blocks_missing_in_the_other_library: u64,
    pub paths_written: u64,
    pub verifier_blocks_written: u64,
    pub flow_facts_written: u64,
    pub rows_written: u64,
    pub synced_writes: u64,
}

impl PlannedImportTally {
    /// 记一批同步写：写下了几行。
    fn count_a_written_batch(&mut self, rows_written_in_the_batch: u64) {
        self.rows_written += rows_written_in_the_batch;
        self.synced_writes += 1;
    }
}

/// 按计划导入时一批攒到这么多字节就同步写一次（一块一种结论的行不拆开，块内原子）：一块 GPU 结论连恢复字约 2.3 MiB
/// （65536 个状态每个 4 字节编号加 32 字节恢复字），一批装几十块；同步写的次数随写下的字节数线性，攒批的内存有界。数是推的，没量过。
const PLANNED_IMPORT_BATCH_BYTES: usize = 64 << 20;

/// 按计划导入时正在攒的一批：要写的行，与这一批里新登记的判定向量（按登记次序；这一批写下去之后才进库的内存表）。
struct PlannedImportBatch {
    batch: rocksdb::WriteBatch,
    verdict_vectors_registered_in_the_batch: Vec<EncodedVerdictVector>,
}

impl PlannedImportBatch {
    fn empty() -> Self {
        Self {
            batch: rocksdb::WriteBatch::default(),
            verdict_vectors_registered_in_the_batch: Vec::new(),
        }
    }
}

/// 照判、不入库的那几把版本与这个进程里写下的、键里带着它们的行（文件头「照判、不入库的那几把版本」）。版本是判法单元拒算那几组的占位
/// （按组名取的 SHA-256，与真的判法单元指纹撞不上），键里版本那一截（[`version_field_starts_in_the_key`]）是其中一把的行就归这里。
#[derive(Default)]
struct RowsKeptOutOfTheLibrary {
    versions: BTreeSet<[u8; SHA256_DIGEST_BYTES]>,
    rows: BTreeMap<Vec<u8>, Vec<u8>>,
}

impl RowsKeptOutOfTheLibrary {
    /// 这一行的键里版本那一截（[`version_field_starts_in_the_key`]：按表标签定的偏移，不在整条键上滑着找）是不是一把照判不入库的版本。
    fn keeps_out(&self, key_bytes: &[u8]) -> bool {
        !self.versions.is_empty()
            && version_field_starts_in_the_key(key_bytes)
                .iter()
                .filter_map(|start| key_bytes.get(*start..start.checked_add(SHA256_DIGEST_BYTES)?))
                .filter_map(|field| <[u8; SHA256_DIGEST_BYTES]>::try_from(field).ok())
                .any(|field| self.versions.contains(&field))
    }
}

/// 一批写拆成写进库的与照判不入库的（[`RowsKeptOutOfTheLibrary::keeps_out`]）。
struct BatchSplitByTheKeptOutVersions<'kept> {
    kept_out: &'kept RowsKeptOutOfTheLibrary,
    into_the_library: rocksdb::WriteBatch,
    kept_out_puts: Vec<(Vec<u8>, Vec<u8>)>,
    kept_out_deletes: Vec<Vec<u8>>,
}

impl rocksdb::WriteBatchIterator for BatchSplitByTheKeptOutVersions<'_> {
    fn put(&mut self, key: &[u8], value: &[u8]) {
        if self.kept_out.keeps_out(key) {
            self.kept_out_puts.push((key.to_vec(), value.to_vec()));
        } else {
            self.into_the_library.put(key, value);
        }
    }

    fn delete(&mut self, key: &[u8]) {
        if self.kept_out.keeps_out(key) {
            self.kept_out_deletes.push(key.to_vec());
        } else {
            self.into_the_library.delete(key);
        }
    }
}

/// 这个进程里照判不入库的行里键以 `prefix` 开头的那几行，按键升序一次取一行：每取一行锁一次、从上一次取到的键之后找下一行，
/// 不把整段拷出来，也不在两次取之间拿着锁（调用方边读边经同一份库写也不会自己等自己）。两次取之间写进来的行，键落在已取过的键之后的照样取到。
struct KeptOutRowsWithThePrefix<'store> {
    rows_kept_out_of_the_library: &'store std::sync::Mutex<RowsKeptOutOfTheLibrary>,
    prefix: Vec<u8>,
    last_key_taken: Option<Vec<u8>>,
}

impl Iterator for KeptOutRowsWithThePrefix<'_> {
    type Item = (Vec<u8>, Vec<u8>);

    fn next(&mut self) -> Option<Self::Item> {
        let kept_out = self
            .rows_kept_out_of_the_library
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let lower_bound = match &self.last_key_taken {
            Some(last_key_taken) => std::ops::Bound::Excluded(last_key_taken.as_slice()),
            None => std::ops::Bound::Included(self.prefix.as_slice()),
        };
        let (key_bytes, value_bytes) = kept_out
            .rows
            .range::<[u8], _>((lower_bound, std::ops::Bound::Unbounded))
            .next()?;
        if !key_bytes.starts_with(&self.prefix) {
            return None;
        }
        let row = (key_bytes.clone(), value_bytes.clone());
        drop(kept_out);
        self.last_key_taken = Some(row.0.clone());
        Some(row)
    }
}

/// 两路按键升序的行归并成一路：库里的一路（读错时交 `Err`）与照判不入库的一路。两路各只往前看一行（[`std::iter::Peekable`]），
/// 交一行才从那一路再取一行，任何时候手里最多各压一行，前缀下有多少行都不整段读进来。同一个键两路都有时交照判不入库那一行、库里那一行跳过
/// （与改前「先放库里的、再用照判不入库的盖」的合并逐条相同）。库那一路交出 `Err` 之后这一路就到此为止，不再交任何一行。
///
/// 错误类型写死成 `rocksdb::Error`、不带泛型参数：单元图把自身类型里带 impl 泛型参数的 impl 当 blanket impl 放行（`crates/singlefs-unit-graph`
/// 的 `units.rs` 里 blanket 的定义），这个迭代器一带泛型，判器闭包里任何一处 `.next()` 都会把它连同判定库的一串单元拉进判器的键
/// （`crash_identity` 的 `the_judge_groups_reach_no_pipeline_unit_and_the_cpu_groups_no_card_unit` 盯着）。
struct RowsOfTwoSourcesMergedByKey<'rows> {
    library_rows: std::iter::Peekable<LibraryRowsToMerge<'rows>>,
    kept_out_rows: std::iter::Peekable<KeptOutRowsToMerge<'rows>>,
    is_ended_by_a_library_error: bool,
}

/// 归并的库那一路：按键升序的库行，读错时交 `Err`。
type LibraryRowsToMerge<'rows> =
    Box<dyn Iterator<Item = Result<StoredEntry, rocksdb::Error>> + 'rows>;

/// 照判不入库的一行：键与值。
type KeptOutRow = (Vec<u8>, Vec<u8>);

/// 归并的照判不入库那一路：按键升序的行。
type KeptOutRowsToMerge<'rows> = Box<dyn Iterator<Item = KeptOutRow> + 'rows>;

impl<'rows> RowsOfTwoSourcesMergedByKey<'rows> {
    fn new(
        library_rows: impl Iterator<Item = Result<StoredEntry, rocksdb::Error>> + 'rows,
        kept_out_rows: impl Iterator<Item = KeptOutRow> + 'rows,
    ) -> Self {
        let library_rows: LibraryRowsToMerge<'rows> = Box::new(library_rows);
        let kept_out_rows: KeptOutRowsToMerge<'rows> = Box::new(kept_out_rows);
        Self {
            library_rows: library_rows.peekable(),
            kept_out_rows: kept_out_rows.peekable(),
            is_ended_by_a_library_error: false,
        }
    }

    fn next_kept_out_row(&mut self) -> Option<Result<StoredEntry, rocksdb::Error>> {
        self.kept_out_rows.next().map(|(key_bytes, value_bytes)| {
            Ok((key_bytes.into_boxed_slice(), value_bytes.into_boxed_slice()))
        })
    }
}

impl Iterator for RowsOfTwoSourcesMergedByKey<'_> {
    type Item = Result<StoredEntry, rocksdb::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_ended_by_a_library_error {
            return None;
        }
        let library_key = match self.library_rows.peek() {
            Some(Err(_)) => {
                self.is_ended_by_a_library_error = true;
                return self.library_rows.next();
            }
            Some(Ok((library_key, _value_bytes))) => Some(library_key),
            None => None,
        };
        let library_against_kept_out = match (library_key, self.kept_out_rows.peek()) {
            (None, None) => return None,
            (Some(_library_key), None) => std::cmp::Ordering::Less,
            (None, Some(_kept_out_row)) => std::cmp::Ordering::Greater,
            (Some(library_key), Some((kept_out_key, _value_bytes))) => {
                library_key.as_ref().cmp(kept_out_key.as_slice())
            }
        };
        match library_against_kept_out {
            std::cmp::Ordering::Less => self.library_rows.next(),
            std::cmp::Ordering::Equal => {
                self.library_rows.next();
                self.next_kept_out_row()
            }
            std::cmp::Ordering::Greater => self.next_kept_out_row(),
        }
    }
}

/// 一个打开着的判定库。同一时刻一个库只许一个进程开（RocksDB 的 LOCK 文件），进程里改库的方法都要 `&mut self`，
/// 所以「先读后写」的两步之间没有别的写者。
pub struct VerdictStore {
    database: rocksdb::DB,
    directory: PathBuf,
    layout: VerdictStoreLayout,
    verdict_vector_by_number: Vec<EncodedVerdictVector>,
    verdict_vector_number_by_content: HashMap<Vec<u8>, VerdictVectorNumber>,
    /// 照判、不入库的版本与它们的行（读的方法只拿 `&self`，各条判定线程经库外那把锁一次一个进来，这里另套一把锁好让 `&self` 的读也看得到）。
    rows_kept_out_of_the_library: std::sync::Mutex<RowsKeptOutOfTheLibrary>,
    /// [`SyncedWriteTally`] 的两个数（写的方法有的只拿 `&self`，所以是原子量）。
    synced_batches_written: std::sync::atomic::AtomicU64,
    synced_bytes_written: std::sync::atomic::AtomicU64,
    /// 观察者见证那几批还放几批过去，之后的一律拒（[`VerdictStoreError::ObserverWitnessBatchRefusedByTheTestSwitch`]）；
    /// 平常是 [`OBSERVER_WITNESS_BATCHES_ARE_NEVER_REFUSED`]，只有测试改它。
    observer_witness_batches_still_written: std::sync::atomic::AtomicU64,
}

impl VerdictStore {
    /// 在一个还不存在的目录上建一个空库，写下布局行（[`VerdictStoreLayout::ConclusionKinds`]）同步落盘之后才返回。
    ///
    /// # Errors
    /// 目录已经在了（`LibraryDirectoryAlreadyExists`）、看不了目录在不在、RocksDB 建库或写布局行失败。
    pub fn create_empty(directory: &Path) -> Result<Self, VerdictStoreError> {
        if library_directory_exists(directory)? {
            return Err(VerdictStoreError::LibraryDirectoryAlreadyExists {
                directory: directory.to_path_buf(),
            });
        }
        let database = rocksdb::DB::open(&library_options(LibraryCreation::CreateEmpty), directory)
            .map_err(VerdictStoreError::RocksDatabase)?;
        let mut batch = rocksdb::WriteBatch::default();
        batch.put(
            VerdictStoreKey::LibraryLayout.to_key_bytes().0,
            CONCLUSION_KINDS_LAYOUT_NUMBER.to_be_bytes(),
        );
        database
            .write_opt(batch, &synced_write_options())
            .map_err(VerdictStoreError::RocksDatabase)?;
        Self::from_opened_database(database, directory)
    }

    /// 开一个已有的库；第一次打不开时调一次 `DB::repair` 再开。
    ///
    /// # Errors
    /// 目录不在（`LibraryDirectoryMissing`，不修、不建）、看不了目录在不在、修过一次还打不开、两张判定向量表读回来对不上；
    /// 判定向量编号还是一个字节的布局（`LibraryLayoutWithOneByteVerdictVectorNumbers`）。
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
        Ok((Self::from_opened_database(database, directory)?, opening))
    }

    /// 只读地开一个已有的库（崩溃枚举用例只查块键那一趟用，`SINGLEFS_CRASH_CASE_BLOCKS_QUERY=1`）：不拿库的锁、不写任何一行、不修库；
    /// RocksDB 自己的运行日志写到 `log_directory`，库目录里一个文件都不动、修改时刻不变。这一套之前的库照样拒绝。
    ///
    /// # Errors
    /// 目录不在、看不了目录在不在、RocksDB 只读打不开、两张判定向量表读回来对不上；这一套之前的库（`LibraryLayoutPredatesConclusionKinds`）；
    /// 判定向量编号还是一个字节的布局（`LibraryLayoutWithOneByteVerdictVectorNumbers`）。
    pub fn open_existing_for_reading_only(
        directory: &Path,
        log_directory: &Path,
    ) -> Result<Self, VerdictStoreError> {
        if !library_directory_exists(directory)? {
            return Err(VerdictStoreError::LibraryDirectoryMissing {
                directory: directory.to_path_buf(),
            });
        }
        let mut options = library_options(LibraryCreation::OpenExisting);
        options.set_db_log_dir(log_directory);
        let database = rocksdb::DB::open_for_read_only(&options, directory, false)
            .map_err(VerdictStoreError::RocksDatabase)?;
        let store = Self::from_opened_database(database, directory)?;
        match store.layout {
            VerdictStoreLayout::ConclusionKinds => Ok(store),
            VerdictStoreLayout::BeforeConclusionKinds => {
                Err(VerdictStoreError::LibraryLayoutPredatesConclusionKinds {
                    directory: directory.to_path_buf(),
                })
            }
        }
    }

    /// 开一个已有的、有结论种类那几张表的库（[`Self::open_existing`] 再看布局）：这一套之前的库拒绝、不迁移。
    ///
    /// # Errors
    /// 同 [`Self::open_existing`]（判定向量编号还是一个字节的布局在那里就拒）；库没有布局行（`LibraryLayoutPredatesConclusionKinds`）。
    pub fn open_existing_with_conclusion_kinds(
        directory: &Path,
    ) -> Result<(Self, VerdictStoreOpening), VerdictStoreError> {
        let (store, opening) = Self::open_existing(directory)?;
        match store.layout {
            VerdictStoreLayout::ConclusionKinds => Ok((store, opening)),
            VerdictStoreLayout::BeforeConclusionKinds => {
                Err(VerdictStoreError::LibraryLayoutPredatesConclusionKinds {
                    directory: directory.to_path_buf(),
                })
            }
        }
    }

    fn from_opened_database(
        database: rocksdb::DB,
        directory: &Path,
    ) -> Result<Self, VerdictStoreError> {
        let layout = read_library_layout(&database, directory)?;
        let LoadedVerdictVectors {
            verdict_vector_by_number,
            verdict_vector_number_by_content,
        } = load_verdict_vectors(&database)?;
        Ok(Self {
            database,
            directory: directory.to_path_buf(),
            layout,
            verdict_vector_by_number,
            verdict_vector_number_by_content,
            rows_kept_out_of_the_library: std::sync::Mutex::new(RowsKeptOutOfTheLibrary::default()),
            synced_batches_written: std::sync::atomic::AtomicU64::new(0),
            synced_bytes_written: std::sync::atomic::AtomicU64::new(0),
            observer_witness_batches_still_written: std::sync::atomic::AtomicU64::new(
                OBSERVER_WITNESS_BATCHES_ARE_NEVER_REFUSED,
            ),
        })
    }

    /// 这个进程开库以来写进库的同步批的账（[`SyncedWriteTally`]）。
    #[must_use]
    pub fn synced_write_tally(&self) -> SyncedWriteTally {
        SyncedWriteTally {
            batches_written: self
                .synced_batches_written
                .load(std::sync::atomic::Ordering::SeqCst),
            bytes_written: self
                .synced_bytes_written
                .load(std::sync::atomic::Ordering::SeqCst),
        }
    }

    /// 测试开关：观察者见证那几批（[`Self::store_observed_image_with_its_read_set_witness`]）再放 `batches_still_written` 批过去，
    /// 之后的一律拒、一行都不写（当作落盘之前进程就被杀掉）。
    #[cfg(test)]
    pub(crate) fn refuse_the_observer_witness_batches_after(&self, batches_still_written: u64) {
        self.observer_witness_batches_still_written
            .store(batches_still_written, std::sync::atomic::Ordering::SeqCst);
    }

    /// 登记照判、不入库的版本（判法单元拒算那几组的占位版本）：此后键里带着其中一把的行不写进 RocksDB，只留在这个进程里，读的时候合起来读。
    /// 登记之前已经写进库的行不动。
    pub fn keep_the_rows_of_these_versions_out_of_the_library(
        &mut self,
        versions: impl IntoIterator<Item = [u8; SHA256_DIGEST_BYTES]>,
    ) {
        self.rows_kept_out_of_the_library
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .versions
            .extend(versions);
    }

    /// 照判、不入库的那几把版本（[`Self::keep_the_rows_of_these_versions_out_of_the_library`] 登记的），按字节序。
    #[must_use]
    pub fn versions_kept_out_of_the_library(&self) -> Vec<[u8; SHA256_DIGEST_BYTES]> {
        self.rows_kept_out_of_the_library
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .versions
            .iter()
            .copied()
            .collect()
    }

    /// 写一批：键里带着照判不入库的版本的行只记在这个进程里，别的照 `options` 写进库（一批里两边各自原子）。
    fn write_batch_keeping_the_kept_out_rows_in_this_process(
        &self,
        batch: rocksdb::WriteBatch,
        options: &rocksdb::WriteOptions,
    ) -> Result<(), rocksdb::Error> {
        let batch_bytes = u64::try_from(batch.size_in_bytes()).expect("一批的字节数装得进 u64");
        self.write_batch_split_by_the_kept_out_versions(batch, options)?;
        self.synced_batches_written
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.synced_bytes_written
            .fetch_add(batch_bytes, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    /// [`Self::write_batch_keeping_the_kept_out_rows_in_this_process`] 不记账的那一截：拆开、写库、记下照判不入库的行。
    fn write_batch_split_by_the_kept_out_versions(
        &self,
        batch: rocksdb::WriteBatch,
        options: &rocksdb::WriteOptions,
    ) -> Result<(), rocksdb::Error> {
        let mut kept_out = self
            .rows_kept_out_of_the_library
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if kept_out.versions.is_empty() {
            drop(kept_out);
            return self.database.write_opt(batch, options);
        }
        let (into_the_library, kept_out_puts, kept_out_deletes) = {
            let mut split = BatchSplitByTheKeptOutVersions {
                kept_out: &kept_out,
                into_the_library: rocksdb::WriteBatch::default(),
                kept_out_puts: Vec::new(),
                kept_out_deletes: Vec::new(),
            };
            batch.iterate(&mut split);
            (
                split.into_the_library,
                split.kept_out_puts,
                split.kept_out_deletes,
            )
        };
        self.database.write_opt(into_the_library, options)?;
        for key in kept_out_deletes {
            kept_out.rows.remove(&key);
        }
        kept_out.rows.extend(kept_out_puts);
        Ok(())
    }

    /// 读一行：照判不入库的那几行先看这个进程里的，别的读库。
    fn read_value_kept_in_or_out(
        &self,
        key_bytes: &[u8],
    ) -> Result<Option<Vec<u8>>, rocksdb::Error> {
        let kept_out = self
            .rows_kept_out_of_the_library
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if kept_out.keeps_out(key_bytes) {
            return Ok(kept_out.rows.get(key_bytes).cloned());
        }
        drop(kept_out);
        self.database.get(key_bytes)
    }

    /// 键以 `prefix` 开头的全部条目，按键的次序：库里的与这个进程里照判不入库的两路有序流逐行归并（[`RowsOfTwoSourcesMergedByKey`]，
    /// 不把哪一路整段读进来），同一个键两路都有时取照判不入库那一行；没有登记版本时就是库里的。库那一路报错时交出那个错、到此为止
    /// （调用方见错即停）。
    fn entries_with_key_prefix_kept_in_or_out(
        &self,
        prefix: VerdictStoreKeyBytes,
    ) -> Box<dyn Iterator<Item = Result<StoredEntry, rocksdb::Error>> + '_> {
        let no_version_is_kept_out = self
            .rows_kept_out_of_the_library
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .versions
            .is_empty();
        if no_version_is_kept_out {
            return Box::new(entries_with_key_prefix(&self.database, prefix));
        }
        let kept_out_rows = KeptOutRowsWithThePrefix {
            rows_kept_out_of_the_library: &self.rows_kept_out_of_the_library,
            prefix: prefix.0.clone(),
            last_key_taken: None,
        };
        Box::new(RowsOfTwoSourcesMergedByKey::new(
            entries_with_key_prefix(&self.database, prefix),
            kept_out_rows,
        ))
    }

    /// 给一种判定向量一个编号：登记过的返回原来的编号、什么都不写；没登记过的给下一个编号，两张表同一批同步写下去之后才返回。
    ///
    /// # Errors
    /// 2^32 种都登记过了（`EveryU32VerdictVectorNumberIsAlreadyRegistered`，什么都没写）；RocksDB 写失败。
    pub fn register_verdict_vector(
        &mut self,
        verdict_vector: &EncodedVerdictVector,
    ) -> Result<VerdictVectorNumber, VerdictStoreError> {
        if let Some(number) = self.verdict_vector_number_by_content.get(&verdict_vector.0) {
            return Ok(*number);
        }
        let Some(number) =
            VerdictVectorNumber::next_after_registered_count(self.verdict_vector_by_number.len())
        else {
            return Err(VerdictStoreError::EveryU32VerdictVectorNumberIsAlreadyRegistered);
        };
        let mut batch = rocksdb::WriteBatch::default();
        batch.put(
            VerdictStoreKey::VerdictVectorNumberByContent(verdict_vector)
                .to_key_bytes()
                .0,
            number.to_stored_bytes(),
        );
        batch.put(
            VerdictStoreKey::VerdictVectorContentByNumber(number)
                .to_key_bytes()
                .0,
            &verdict_vector.0,
        );
        self.write_batch_keeping_the_kept_out_rows_in_this_process(batch, &synced_write_options())
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
        self.verdict_vector_by_number
            .get(number.position_in_the_registration_order())
    }

    /// 存一块在这一判法单元指纹下的判定与它的违例：块与每个违例状态同一批同步写下去之后才返回；违例状态的判定向量编号取块里那个状态的。
    /// 同一块键在别的判法单元指纹下的行不受影响。
    ///
    /// # Errors
    /// 这个块键在这一版本下已经存着不同的块或不同的违例（`DifferentVerdictsUnderAStoredBlockKeyNotDecidedInTheFirstVersion`，
    /// 什么都没写）；已存的违例读回来不合布局；RocksDB 读写失败。
    ///
    /// # Panics
    /// 块里有这个库没登记过的编号（拿了别的库的编号），或违例状态的块内序号不在块里：都是调用方的 bug。
    pub fn store_verdict_block(
        &mut self,
        block_key: &VerdictBlockKey,
        version: &JudgingUnitsFingerprint,
        block: &VerdictBlock,
        violating_states: &[StateOffsetInBlock],
    ) -> Result<VerdictBlockStoring, VerdictStoreError> {
        let registered_count = self.verdict_vector_by_number.len();
        assert!(
            block
                .verdict_vector_number_of_each_state()
                .iter()
                .all(|number| number.position_in_the_registration_order() < registered_count),
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
        let block_key_bytes = VerdictStoreKey::VerdictBlock(block_key, version).to_key_bytes();
        let block_value_bytes = block.to_block_value_bytes();
        let stored_block_value = self
            .read_value_kept_in_or_out(&block_key_bytes.0)
            .map_err(VerdictStoreError::RocksDatabase)?;
        if let Some(stored_block_value_bytes) = stored_block_value {
            let stored_violations = self.violations_in_block(block_key, version)?;
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
                VerdictStoreKey::Violation(block_key, version, violation.offset_in_block)
                    .to_key_bytes()
                    .0,
                violation.verdict_vector_number.to_stored_bytes(),
            );
        }
        self.write_batch_keeping_the_kept_out_rows_in_this_process(batch, &synced_write_options())
            .map_err(VerdictStoreError::RocksDatabase)?;
        Ok(VerdictBlockStoring::WrittenAndSynced)
    }

    fn registered_verdict_vector_number(
        &self,
        table: VerdictStoreTable,
        key_bytes: &[u8],
        stored_number: u32,
    ) -> Result<VerdictVectorNumber, VerdictStoreError> {
        let number = VerdictVectorNumber(stored_number);
        if number.position_in_the_registration_order() < self.verdict_vector_by_number.len() {
            return Ok(number);
        }
        Err(stored_bytes_mismatch(
            table,
            key_bytes,
            "存着的判定向量编号在这个库里没登记过",
        ))
    }

    /// 存着的一块（判定块或结论块）的值读成每个状态的编号：按 4 字节大端切，每个编号都要在这个库里登记过。空不空不在这里判。
    fn registered_verdict_vector_numbers_of_a_stored_block(
        &self,
        table: VerdictStoreTable,
        key_bytes: &[u8],
        value_bytes: &[u8],
    ) -> Result<Vec<VerdictVectorNumber>, VerdictStoreError> {
        let Some(stored_numbers) = stored_verdict_vector_numbers(value_bytes) else {
            return Err(stored_bytes_mismatch(
                table,
                key_bytes,
                "存着的块值的字节数不是 4 字节编号的整数倍",
            ));
        };
        stored_numbers
            .into_iter()
            .map(|stored_number| {
                self.registered_verdict_vector_number(table, key_bytes, stored_number)
            })
            .collect()
    }

    /// 读回一块在这一判法单元指纹下的判定；这个块键在这一版本下没存过时是 `None`（别的版本存过也是 `None`：复用只认同一版本）。
    ///
    /// # Errors
    /// 存着的块值是空的或点名了没登记的编号；RocksDB 读失败。
    pub fn read_verdict_block(
        &self,
        block_key: &VerdictBlockKey,
        version: &JudgingUnitsFingerprint,
    ) -> Result<Option<VerdictBlock>, VerdictStoreError> {
        let block_key_bytes = VerdictStoreKey::VerdictBlock(block_key, version).to_key_bytes();
        let Some(block_value_bytes) = self
            .read_value_kept_in_or_out(&block_key_bytes.0)
            .map_err(VerdictStoreError::RocksDatabase)?
        else {
            return Ok(None);
        };
        let verdict_vector_number_of_each_state = self
            .registered_verdict_vector_numbers_of_a_stored_block(
                VerdictStoreTable::VerdictBlocks,
                &block_key_bytes.0,
                &block_value_bytes,
            )?;
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

    /// 这一块在这一判法单元指纹下的违例，按块内序号排；没有违例或没存过这一块时是空的。
    ///
    /// # Errors
    /// 违例表里的键或值长度不对、点名了没登记的编号；RocksDB 读失败。
    pub fn violations_in_block(
        &self,
        block_key: &VerdictBlockKey,
        version: &JudgingUnitsFingerprint,
    ) -> Result<Vec<StoredViolation>, VerdictStoreError> {
        let prefix = VerdictStoreKey::ViolationsOfBlockPrefix(block_key, version).to_key_bytes();
        let prefix_length = prefix.0.len();
        let mut violations = Vec::new();
        for entry in self.entries_with_key_prefix_kept_in_or_out(prefix) {
            let (key_bytes, number_bytes) = entry.map_err(VerdictStoreError::RocksDatabase)?;
            let offset_bytes =
                <[u8; STATE_OFFSET_IN_BLOCK_BYTES]>::try_from(&key_bytes[prefix_length..]);
            let (Ok(offset_bytes), Some(stored_number)) =
                (offset_bytes, stored_verdict_vector_number(&number_bytes))
            else {
                return Err(stored_bytes_mismatch(
                    VerdictStoreTable::Violations,
                    &key_bytes,
                    "违例表里块键之后不是 8 字节的块内序号，或值不是 4 字节的编号",
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
            .read_value_kept_in_or_out(&key_bytes.0)
            .map_err(VerdictStoreError::RocksDatabase)?
        {
            if stored == state_count.to_be_bytes() {
                return Ok(());
            }
            return Err(VerdictStoreError::DifferentVerdictsUnderAStoredBlockKeyNotDecidedInTheFirstVersion { block_key: block_key.clone() });
        }
        let mut batch = rocksdb::WriteBatch::default();
        batch.put(&key_bytes.0, state_count.to_be_bytes());
        self.write_batch_keeping_the_kept_out_rows_in_this_process(batch, &synced_write_options())
            .map_err(VerdictStoreError::RocksDatabase)
    }

    /// 录入分片的段的一块（C790）：同 [`Self::record_block`]，只是录过、状态数不同、而这一块在判定块表、结论块表、核对表里一行都没有
    /// （上一趟录了这一块、没判完就停了，两趟之间片大小改了，这一块的长度跟着变）时照这一趟的状态数改写；有判过的行还是拒。
    ///
    /// # Errors
    /// RocksDB 读写失败；同一个块键录过、状态数不同、这一块已经有判过的行。
    pub fn record_block_of_a_slice(
        &mut self,
        block_key: &VerdictBlockKey,
        state_count: u64,
    ) -> Result<(), VerdictStoreError> {
        let key_bytes = VerdictStoreKey::RecordedBlock(block_key).to_key_bytes();
        if let Some(stored) = self
            .read_value_kept_in_or_out(&key_bytes.0)
            .map_err(VerdictStoreError::RocksDatabase)?
        {
            if stored == state_count.to_be_bytes() {
                return Ok(());
            }
            if self.block_has_a_judged_row(block_key)? {
                return Err(VerdictStoreError::DifferentVerdictsUnderAStoredBlockKeyNotDecidedInTheFirstVersion { block_key: block_key.clone() });
            }
        }
        self.put_one_synced(
            &VerdictStoreKey::RecordedBlock(block_key),
            &state_count.to_be_bytes(),
        )
    }

    /// 这一块在判定块表、结论块表、核对表里有没有任何一行（哪一判法单元指纹、哪一种结论、哪一份核对代码都算）。
    ///
    /// # Errors
    /// RocksDB 读失败。
    fn block_has_a_judged_row(
        &self,
        block_key: &VerdictBlockKey,
    ) -> Result<bool, VerdictStoreError> {
        for table in [
            VerdictStoreTable::VerdictBlocks,
            VerdictStoreTable::ConclusionBlocks,
            VerdictStoreTable::VerifierVerdicts,
        ] {
            let prefix = VerdictStoreKey::RowsOfABlockPrefix(table, block_key).to_key_bytes();
            if let Some(entry) = self.entries_with_key_prefix_kept_in_or_out(prefix).next() {
                entry.map_err(VerdictStoreError::RocksDatabase)?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// 录入表里这一块要核几个状态；没录过交回 None（分片的段从 0 起逐块问库时按它找下一块的起点）。
    ///
    /// # Errors
    /// RocksDB 读失败；值不是 8 字节。
    pub fn recorded_state_count_of_block(
        &self,
        block_key: &VerdictBlockKey,
    ) -> Result<Option<u64>, VerdictStoreError> {
        let key_bytes = VerdictStoreKey::RecordedBlock(block_key).to_key_bytes();
        let Some(stored) = self
            .read_value_kept_in_or_out(&key_bytes.0)
            .map_err(VerdictStoreError::RocksDatabase)?
        else {
            return Ok(None);
        };
        <[u8; 8]>::try_from(stored.as_slice())
            .map(|bytes| Some(u64::from_be_bytes(bytes)))
            .map_err(|_| {
                stored_bytes_mismatch(
                    VerdictStoreTable::RecordedBlocks,
                    &key_bytes.0,
                    "状态数不是 8 字节",
                )
            })
    }

    /// 写一条分片进度记录（同步落盘）：调用方在推进到的那几块的判定落盘之后才写它。
    ///
    /// # Errors
    /// RocksDB 写失败。
    pub fn store_sliced_segment_progress(
        &mut self,
        key: &SlicedSegmentProgressKey,
        progress: SlicedSegmentProgress,
    ) -> Result<(), VerdictStoreError> {
        self.put_one_synced(
            &VerdictStoreKey::SlicedSegmentProgress(key),
            &progress.to_value_bytes(),
        )
    }

    /// 读一条分片进度记录；没写过交回 None。
    ///
    /// # Errors
    /// RocksDB 读失败；值不合 [`SlicedSegmentProgress`] 的布局。
    pub fn read_sliced_segment_progress(
        &self,
        key: &SlicedSegmentProgressKey,
    ) -> Result<Option<SlicedSegmentProgress>, VerdictStoreError> {
        let key_bytes = VerdictStoreKey::SlicedSegmentProgress(key).to_key_bytes();
        let Some(stored) = self
            .read_value_kept_in_or_out(&key_bytes.0)
            .map_err(VerdictStoreError::RocksDatabase)?
        else {
            return Ok(None);
        };
        SlicedSegmentProgress::from_value_bytes(&stored)
            .map(Some)
            .ok_or_else(|| {
                stored_bytes_mismatch(
                    VerdictStoreTable::SlicedSegmentProgress,
                    &key_bytes.0,
                    "分片进度的值不是格式号 1 加两个 160 字节的编号",
                )
            })
    }

    /// 路径索引登一条：这条路径上的节点是这个身份。同一路径再登、身份相同不算错，什么都不写（与 [`Self::record_block`] 一样先比再写）；
    /// 身份不同也照写（同名不同数据的分叉在路径这一级并成一段，最后写的那个留在索引里；两边都在块表里，一个不丢）。
    ///
    /// # Errors
    /// RocksDB 读写失败。
    pub fn record_path(&mut self, path: &str, identity: &str) -> Result<(), VerdictStoreError> {
        let key_bytes = VerdictStoreKey::Path(path).to_key_bytes();
        let stored_identity = self
            .read_value_kept_in_or_out(&key_bytes.0)
            .map_err(VerdictStoreError::RocksDatabase)?;
        if stored_identity.as_deref() == Some(identity.as_bytes()) {
            return Ok(());
        }
        let mut batch = rocksdb::WriteBatch::default();
        batch.put(&key_bytes.0, identity.as_bytes());
        self.write_batch_keeping_the_kept_out_rows_in_this_process(batch, &synced_write_options())
            .map_err(VerdictStoreError::RocksDatabase)
    }

    /// 这条路径上登过的身份；没登过交回 None。
    ///
    /// # Errors
    /// RocksDB 读失败；值不是 UTF-8。
    pub fn identity_of_path(&self, path: &str) -> Result<Option<String>, VerdictStoreError> {
        let key_bytes = VerdictStoreKey::Path(path).to_key_bytes();
        let Some(value) = self
            .read_value_kept_in_or_out(&key_bytes.0)
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
        for entry in self.entries_with_key_prefix_kept_in_or_out(prefix_bytes) {
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
        self.write_batch_keeping_the_kept_out_rows_in_this_process(batch, &synced_write_options())
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
        self.read_value_kept_in_or_out(&key_bytes.0)
            .map_err(VerdictStoreError::RocksDatabase)
    }

    /// 录入段写下一条流程（`crash_amplification::RecordedFlow` 的字节，键是它的摘要）。同一摘要再写就是同一份内容，照写。
    ///
    /// # Errors
    /// RocksDB 写失败。
    pub fn store_recorded_flow(
        &mut self,
        digest: &[u8; 32],
        bytes: &[u8],
    ) -> Result<(), VerdictStoreError> {
        self.put_one_synced(&VerdictStoreKey::RecordedFlow(digest), bytes)
    }

    /// 读录入段写下的一条流程的字节；没写过交回 None。
    ///
    /// # Errors
    /// RocksDB 读失败。
    pub fn read_recorded_flow(
        &self,
        digest: &[u8; 32],
    ) -> Result<Option<Vec<u8>>, VerdictStoreError> {
        self.read_one_stored_value(&VerdictStoreKey::RecordedFlow(digest))
    }

    /// 录入段登一趟的清单（键是录入键 [`RecordingKey`]：同一把键就是同一份录入输入、同一份录入代码，再登照写同一份）。
    ///
    /// # Errors
    /// RocksDB 写失败。
    pub fn store_recorded_run_manifest(
        &mut self,
        recording_key: &RecordingKey,
        bytes: &[u8],
    ) -> Result<(), VerdictStoreError> {
        self.put_one_synced(&VerdictStoreKey::RecordedRunManifest(recording_key), bytes)
    }

    /// 一趟的清单表里有几行（拒跑之后核「一份清单都没登」用）。
    ///
    /// # Errors
    /// RocksDB 读失败。
    pub fn recorded_run_manifest_count(&self) -> Result<u64, VerdictStoreError> {
        let prefix = VerdictStoreKey::WholeTablePrefix(VerdictStoreTable::RecordedRunManifests)
            .to_key_bytes();
        let mut count = 0u64;
        for entry in self.entries_with_key_prefix_kept_in_or_out(prefix) {
            entry.map_err(VerdictStoreError::RocksDatabase)?;
            count += 1;
        }
        Ok(count)
    }

    /// 读录入键下登的清单；没登过交回 None。
    ///
    /// # Errors
    /// RocksDB 读失败。
    pub fn read_recorded_run_manifest(
        &self,
        recording_key: &RecordingKey,
    ) -> Result<Option<Vec<u8>>, VerdictStoreError> {
        self.read_one_stored_value(&VerdictStoreKey::RecordedRunManifest(recording_key))
    }

    /// 存一条对拍结论（[`StoredPairingConclusion`] 的字节同步落盘）。同一键再存照写（同一份输入、同一对判器判出的结论相同）。
    ///
    /// # Errors
    /// RocksDB 写失败。
    pub fn store_pairing_conclusion(
        &mut self,
        key: &PairingConclusionKey,
        conclusion: &StoredPairingConclusion,
    ) -> Result<(), VerdictStoreError> {
        self.put_one_synced(
            &VerdictStoreKey::PairingConclusion(key),
            &conclusion.to_bytes(),
        )
    }

    /// 读一条对拍结论；没存过交回 None。
    ///
    /// # Errors
    /// RocksDB 读失败；值不合 [`StoredPairingConclusion`] 的布局。
    pub fn read_pairing_conclusion(
        &self,
        key: &PairingConclusionKey,
    ) -> Result<Option<StoredPairingConclusion>, VerdictStoreError> {
        let key_bytes = VerdictStoreKey::PairingConclusion(key).to_key_bytes();
        let Some(stored) = self
            .read_value_kept_in_or_out(&key_bytes.0)
            .map_err(VerdictStoreError::RocksDatabase)?
        else {
            return Ok(None);
        };
        StoredPairingConclusion::from_bytes(&stored)
            .map(Some)
            .ok_or_else(|| {
                stored_bytes_mismatch(
                    VerdictStoreTable::PairingConclusions,
                    &key_bytes.0,
                    "对拍结论的值不合布局",
                )
            })
    }

    /// 录入段写下打包好的 GPU 判表（键是它的摘要）。
    ///
    /// # Errors
    /// RocksDB 写失败。
    pub fn store_packed_judge_tables(
        &mut self,
        digest: &[u8; 32],
        bytes: &[u8],
    ) -> Result<(), VerdictStoreError> {
        self.put_one_synced(&VerdictStoreKey::PackedJudgeTables(digest), bytes)
    }

    /// 读录入段写下的 GPU 判表的字节；没写过交回 None。
    ///
    /// # Errors
    /// RocksDB 读失败。
    pub fn read_packed_judge_tables(
        &self,
        digest: &[u8; 32],
    ) -> Result<Option<Vec<u8>>, VerdictStoreError> {
        self.read_one_stored_value(&VerdictStoreKey::PackedJudgeTables(digest))
    }

    /// 写下一张给定状态表（键是它的摘要）。
    ///
    /// # Errors
    /// RocksDB 写失败。
    pub fn store_given_states_table(
        &mut self,
        digest: &[u8; 32],
        bytes: &[u8],
    ) -> Result<(), VerdictStoreError> {
        self.put_one_synced(&VerdictStoreKey::GivenStatesTable(digest), bytes)
    }

    /// 读一张给定状态表的字节；没写过交回 None。
    ///
    /// # Errors
    /// RocksDB 读失败。
    pub fn read_given_states_table(
        &self,
        digest: &[u8; 32],
    ) -> Result<Option<Vec<u8>>, VerdictStoreError> {
        self.read_one_stored_value(&VerdictStoreKey::GivenStatesTable(digest))
    }

    /// 观察者录入段写下一条流的崩溃镜像共用的基底（键是它的摘要；同一摘要再写就是同一份内容，照写）。
    ///
    /// # Errors
    /// RocksDB 写失败。
    pub fn store_observer_foundation(
        &mut self,
        digest: &[u8; 32],
        bytes: &[u8],
    ) -> Result<(), VerdictStoreError> {
        self.put_one_synced(&VerdictStoreKey::ObserverFoundation(digest), bytes)
    }

    /// 读观察者基底的字节；没写过交回 None。
    ///
    /// # Errors
    /// RocksDB 读失败。
    pub fn read_observer_foundation(
        &self,
        digest: &[u8; 32],
    ) -> Result<Option<Vec<u8>>, VerdictStoreError> {
        self.read_one_stored_value(&VerdictStoreKey::ObserverFoundation(digest))
    }

    /// 观察者录入段写下一批看过的恢复镜像（每条：去重键、它的字节），一批原子地落盘：被打断时这一批要么全在、要么全不在，
    /// 重跑同一条命令只补没写的。
    ///
    /// # Errors
    /// RocksDB 写失败。
    pub fn store_observed_images(
        &mut self,
        records: &[([u8; 32], Vec<u8>)],
    ) -> Result<(), VerdictStoreError> {
        if records.is_empty() {
            return Ok(());
        }
        let mut batch = rocksdb::WriteBatch::default();
        for (deduplication_key, bytes) in records {
            batch.put(
                VerdictStoreKey::ObservedImage(deduplication_key)
                    .to_key_bytes()
                    .0,
                bytes,
            );
        }
        self.write_batch_keeping_the_kept_out_rows_in_this_process(batch, &synced_write_options())
            .map_err(VerdictStoreError::RocksDatabase)
    }

    /// 读一张看过的恢复镜像的字节；没看过交回 None。
    ///
    /// # Errors
    /// RocksDB 读失败。
    pub fn read_observed_image(
        &self,
        deduplication_key: &[u8; 32],
    ) -> Result<Option<Vec<u8>>, VerdictStoreError> {
        self.read_one_stored_value(&VerdictStoreKey::ObservedImage(deduplication_key))
    }

    /// 一个桶的全部读集见证行，按见证摘要的字节序；这个桶还没有见证交回空的。
    ///
    /// # Errors
    /// 键不是桶键之后接 32 字节见证摘要、值不是 64 字节（[`VerdictStoreError::StoredBytesDoNotMatchTheLayout`]）；RocksDB 读失败。
    pub fn read_observer_read_set_witnesses_of_the_bucket(
        &self,
        bucket_key: &[u8; 32],
    ) -> Result<Vec<ObserverReadSetWitnessRow>, VerdictStoreError> {
        let prefix =
            VerdictStoreKey::ObserverReadSetWitnessRowsOfABucketPrefix(bucket_key).to_key_bytes();
        let prefix_length = prefix.0.len();
        let mut witnesses = Vec::new();
        for entry in self.entries_with_key_prefix_kept_in_or_out(prefix) {
            let (key_bytes, value_bytes) = entry.map_err(VerdictStoreError::RocksDatabase)?;
            let witness_digest = <[u8; SHA256_DIGEST_BYTES]>::try_from(&key_bytes[prefix_length..]);
            let (Ok(witness_digest), true) = (
                witness_digest,
                value_bytes.len() == OBSERVER_READ_SET_WITNESS_ROW_VALUE_BYTES,
            ) else {
                return Err(stored_bytes_mismatch(
                    VerdictStoreTable::ObserverReadSetWitnessRows,
                    &key_bytes,
                    "读集见证行的键里桶键之后不是 32 字节的见证摘要，或值不是读集摘要与镜像键各 32 字节",
                ));
            };
            let (read_set_digest, observed_image_key) = value_bytes.split_at(SHA256_DIGEST_BYTES);
            witnesses.push(ObserverReadSetWitnessRow {
                witness_digest,
                read_set_digest: read_set_digest
                    .try_into()
                    .expect("值的长度上面核过是两份摘要"),
                observed_image_key: observed_image_key
                    .try_into()
                    .expect("值的长度上面核过是两份摘要"),
            });
        }
        Ok(witnesses)
    }

    /// 读一组读集行的字节；没写过交回 None。
    ///
    /// # Errors
    /// RocksDB 读失败。
    pub fn read_observer_read_set(
        &self,
        read_set_digest: &[u8; 32],
    ) -> Result<Option<Vec<u8>>, VerdictStoreError> {
        self.read_one_stored_value(&VerdictStoreKey::ObserverReadSet(read_set_digest))
    }

    /// 观察者真跑了一张镜像：这一条见证行、这组读集在这个桶里头一次出现时的读集行（`read_set_bytes_new_in_the_bucket`，
    /// 键是 `witness.read_set_digest`）、那张镜像的记录（键是 `witness.observed_image_key`），三样一批原子地落盘——见证在、它引的读集与镜像必在，
    /// 被打断后重跑找到见证就读得到读集与镜像。写下的字节只随这一条见证与它的镜像（读集行只在这组读集头一次出现时写），与桶里已有几条见证无关。
    ///
    /// # Errors
    /// RocksDB 写失败；测试开关拒了这一批（[`VerdictStoreError::ObserverWitnessBatchRefusedByTheTestSwitch`]，一行都没写）。
    pub fn store_observed_image_with_its_read_set_witness(
        &mut self,
        bucket_key: &[u8; 32],
        witness: &ObserverReadSetWitnessRow,
        read_set_bytes_new_in_the_bucket: Option<&[u8]>,
        image_bytes: &[u8],
    ) -> Result<(), VerdictStoreError> {
        let mut batch = rocksdb::WriteBatch::default();
        let mut witness_row_value = witness.read_set_digest.to_vec();
        witness_row_value.extend_from_slice(&witness.observed_image_key);
        batch.put(
            VerdictStoreKey::ObserverReadSetWitnessRow(bucket_key, &witness.witness_digest)
                .to_key_bytes()
                .0,
            witness_row_value,
        );
        if let Some(read_set_bytes) = read_set_bytes_new_in_the_bucket {
            batch.put(
                VerdictStoreKey::ObserverReadSet(&witness.read_set_digest)
                    .to_key_bytes()
                    .0,
                read_set_bytes,
            );
        }
        batch.put(
            VerdictStoreKey::ObservedImage(&witness.observed_image_key)
                .to_key_bytes()
                .0,
            image_bytes,
        );
        self.write_an_observer_witness_batch(batch)
    }

    /// 观察者见证那几批经这里写：测试开关（[`Self::refuse_the_observer_witness_batches_after`]）放过去的才写。
    fn write_an_observer_witness_batch(
        &self,
        batch: rocksdb::WriteBatch,
    ) -> Result<(), VerdictStoreError> {
        let still_written = self
            .observer_witness_batches_still_written
            .load(std::sync::atomic::Ordering::SeqCst);
        match still_written {
            0 => return Err(VerdictStoreError::ObserverWitnessBatchRefusedByTheTestSwitch),
            OBSERVER_WITNESS_BATCHES_ARE_NEVER_REFUSED => {}
            counted_down => self
                .observer_witness_batches_still_written
                .store(counted_down - 1, std::sync::atomic::Ordering::SeqCst),
        }
        self.write_batch_keeping_the_kept_out_rows_in_this_process(batch, &synced_write_options())
            .map_err(VerdictStoreError::RocksDatabase)
    }

    /// 观察者录入段写下一块（每个状态落在哪一份观察结果上、按发布分的态数）。同一键再写就盖掉上一趟的。
    ///
    /// # Errors
    /// RocksDB 写失败。
    pub fn store_observer_block(
        &mut self,
        block_key: &VerdictBlockKey,
        kind: &ConclusionKind,
        observer_version: &ConclusionVersion,
        bytes: &[u8],
    ) -> Result<(), VerdictStoreError> {
        self.put_one_synced(
            &VerdictStoreKey::ObserverBlock(block_key, kind, observer_version),
            bytes,
        )
    }

    /// 读观察者录入段的一块；录入段没录过交回 None。
    ///
    /// # Errors
    /// RocksDB 读失败。
    pub fn read_observer_block(
        &self,
        block_key: &VerdictBlockKey,
        kind: &ConclusionKind,
        observer_version: &ConclusionVersion,
    ) -> Result<Option<Vec<u8>>, VerdictStoreError> {
        self.read_one_stored_value(&VerdictStoreKey::ObserverBlock(
            block_key,
            kind,
            observer_version,
        ))
    }

    /// 判段写下一批观察者交回的镜像上池级 checker 的结论（同一个判器种类与版本），一批原子地落盘。
    ///
    /// # Errors
    /// RocksDB 写失败。
    pub fn store_observed_image_judgements(
        &mut self,
        kind: &ConclusionKind,
        judging_units_fingerprint: &ConclusionVersion,
        records: &[([u8; 32], Vec<u8>)],
    ) -> Result<(), VerdictStoreError> {
        if records.is_empty() {
            return Ok(());
        }
        let mut batch = rocksdb::WriteBatch::default();
        for (deduplication_key, bytes) in records {
            batch.put(
                VerdictStoreKey::ObservedImageJudgement(
                    deduplication_key,
                    kind,
                    judging_units_fingerprint,
                )
                .to_key_bytes()
                .0,
                bytes,
            );
        }
        self.write_batch_keeping_the_kept_out_rows_in_this_process(batch, &synced_write_options())
            .map_err(VerdictStoreError::RocksDatabase)
    }

    /// 读一张观察者交回的镜像在这个判器种类与版本下的池级 checker 结论；没判过交回 None。
    ///
    /// # Errors
    /// RocksDB 读失败。
    pub fn read_observed_image_judgement(
        &self,
        deduplication_key: &[u8; 32],
        kind: &ConclusionKind,
        judging_units_fingerprint: &ConclusionVersion,
    ) -> Result<Option<Vec<u8>>, VerdictStoreError> {
        self.read_one_stored_value(&VerdictStoreKey::ObservedImageJudgement(
            deduplication_key,
            kind,
            judging_units_fingerprint,
        ))
    }

    fn put_one_synced(
        &mut self,
        key: &VerdictStoreKey<'_>,
        bytes: &[u8],
    ) -> Result<(), VerdictStoreError> {
        let key_bytes = key.to_key_bytes();
        let mut batch = rocksdb::WriteBatch::default();
        batch.put(&key_bytes.0, bytes);
        self.write_batch_keeping_the_kept_out_rows_in_this_process(batch, &synced_write_options())
            .map_err(VerdictStoreError::RocksDatabase)
    }

    fn read_one_stored_value(
        &self,
        key: &VerdictStoreKey<'_>,
    ) -> Result<Option<Vec<u8>>, VerdictStoreError> {
        self.read_value_kept_in_or_out(&key.to_key_bytes().0)
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
        self.write_batch_keeping_the_kept_out_rows_in_this_process(batch, &synced_write_options())
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
            .read_value_kept_in_or_out(&key_bytes.0)
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

    /// 核对表里这一块的每一把核对键（块键之后那 32 字节：核对那一组判法单元指纹与事实表摘要的摘要），按键序。
    ///
    /// # Errors
    /// RocksDB 读失败；键字节认不回来。
    pub fn verifier_row_keys_of_block(
        &self,
        block_key: &VerdictBlockKey,
    ) -> Result<Vec<[u8; SHA256_DIGEST_BYTES]>, VerdictStoreError> {
        let prefix =
            VerdictStoreKey::RowsOfABlockPrefix(VerdictStoreTable::VerifierVerdicts, block_key)
                .to_key_bytes();
        let prefix_length = prefix.0.len();
        let mut found = Vec::new();
        for entry in self.entries_with_key_prefix_kept_in_or_out(prefix) {
            let (key_bytes, _value_bytes) = entry.map_err(VerdictStoreError::RocksDatabase)?;
            found.push(
                <[u8; SHA256_DIGEST_BYTES]>::try_from(&key_bytes[prefix_length..]).map_err(
                    |_| {
                        stored_bytes_mismatch(
                            VerdictStoreTable::VerifierVerdicts,
                            &key_bytes,
                            "核对表的键在块键之后要正好是 32 字节",
                        )
                    },
                )?,
            );
        }
        Ok(found)
    }

    /// 核对表里全部行的原始键值（导入用）。
    fn verifier_rows(&self) -> Result<RawRows, VerdictStoreError> {
        let prefix =
            VerdictStoreKey::WholeTablePrefix(VerdictStoreTable::VerifierVerdicts).to_key_bytes();
        let mut rows = Vec::new();
        for entry in self.entries_with_key_prefix_kept_in_or_out(prefix) {
            let (key_bytes, value_bytes) = entry.map_err(VerdictStoreError::RocksDatabase)?;
            rows.push((key_bytes.to_vec(), value_bytes.to_vec()));
        }
        Ok(rows)
    }

    /// 事实表里全部行的原始键值（导入用）。
    fn facts_rows(&self) -> Result<RawRows, VerdictStoreError> {
        let prefix = VerdictStoreKey::WholeTablePrefix(VerdictStoreTable::FlowFacts).to_key_bytes();
        let mut rows = Vec::new();
        for entry in self.entries_with_key_prefix_kept_in_or_out(prefix) {
            let (key_bytes, value_bytes) = entry.map_err(VerdictStoreError::RocksDatabase)?;
            rows.push((key_bytes.to_vec(), value_bytes.to_vec()));
        }
        Ok(rows)
    }

    /// 一张按块键排的表里的全部行：块键、块键之后的键后缀、值。
    fn block_keys_of_table(
        &self,
        table: VerdictStoreTable,
    ) -> Result<Vec<BlockTableRow>, VerdictStoreError> {
        let prefix = VerdictStoreKey::WholeTablePrefix(table).to_key_bytes();
        let mut found = Vec::new();
        for entry in self.entries_with_key_prefix_kept_in_or_out(prefix) {
            let (key_bytes, value_bytes) = entry.map_err(VerdictStoreError::RocksDatabase)?;
            let (block_key, suffix) =
                VerdictBlockKey::from_block_key_bytes_with_suffix(&key_bytes[1..])
                    .ok_or_else(|| stored_bytes_mismatch(table, &key_bytes, "键字节认不回块键"))?;
            found.push((block_key, suffix.to_vec(), value_bytes.to_vec()));
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
            .map(|(block_key, suffix, value)| {
                if !suffix.is_empty() {
                    return Err(stored_bytes_mismatch(
                        VerdictStoreTable::RecordedBlocks,
                        &suffix,
                        "录入表的键在块键之后不该有别的字节",
                    ));
                }
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

    /// 判定块表里的全部（块键，判法单元指纹），按键序：同一块在几个版本下判过就有几行。
    ///
    /// # Errors
    /// RocksDB 读失败；键字节认不回来。
    pub fn stored_verdict_block_keys(
        &self,
    ) -> Result<Vec<(VerdictBlockKey, JudgingUnitsFingerprint)>, VerdictStoreError> {
        self.block_keys_of_table(VerdictStoreTable::VerdictBlocks)?
            .into_iter()
            .map(|(block_key, suffix, _value)| {
                let version = <[u8; SHA256_DIGEST_BYTES]>::try_from(suffix.as_slice())
                    .map(JudgingUnitsFingerprint)
                    .map_err(|_| {
                        stored_bytes_mismatch(
                            VerdictStoreTable::VerdictBlocks,
                            &suffix,
                            "判定块的键在块键之后要正好是 32 字节的判法单元指纹",
                        )
                    })?;
                Ok((block_key, version))
            })
            .collect()
    }

    /// 判法单元组成清单表登一把键的组成清单（随键落盘：库对不上时照它点名变了的单元）。同一把再登照写。
    ///
    /// # Errors
    /// RocksDB 写失败。
    pub fn store_judging_units_composition(
        &mut self,
        key: &JudgingUnitsKey,
    ) -> Result<(), VerdictStoreError> {
        self.put_one_synced(
            &VerdictStoreKey::JudgingUnitsComposition(key.fingerprint()),
            key.composition_text().as_bytes(),
        )
    }

    /// 一把判法单元指纹登过的组成清单；没登过交回 None。
    ///
    /// # Errors
    /// RocksDB 读失败；值认不回组成清单。
    pub fn read_judging_units_composition(
        &self,
        fingerprint: &[u8; SHA256_DIGEST_BYTES],
    ) -> Result<Option<Vec<JudgingUnitFingerprint>>, VerdictStoreError> {
        let key_bytes = VerdictStoreKey::JudgingUnitsComposition(fingerprint).to_key_bytes();
        let Some(stored) = self
            .read_value_kept_in_or_out(&key_bytes.0)
            .map_err(VerdictStoreError::RocksDatabase)?
        else {
            return Ok(None);
        };
        JudgingUnitsKey::composition_from_text(&String::from_utf8_lossy(&stored))
            .map(Some)
            .ok_or_else(|| {
                stored_bytes_mismatch(
                    VerdictStoreTable::JudgingUnitsCompositions,
                    &key_bytes.0,
                    "组成清单认不回来",
                )
            })
    }

    /// 判法单元组成清单表里登过的每一把指纹，按指纹字节排。
    ///
    /// # Errors
    /// RocksDB 读失败；键不是 32 字节。
    pub fn judging_units_fingerprints(
        &self,
    ) -> Result<Vec<[u8; SHA256_DIGEST_BYTES]>, VerdictStoreError> {
        let prefix = VerdictStoreKey::WholeTablePrefix(VerdictStoreTable::JudgingUnitsCompositions)
            .to_key_bytes();
        let mut found = Vec::new();
        for entry in self.entries_with_key_prefix_kept_in_or_out(prefix) {
            let (key_bytes, _value_bytes) = entry.map_err(VerdictStoreError::RocksDatabase)?;
            found.push(
                <[u8; SHA256_DIGEST_BYTES]>::try_from(&key_bytes[1..]).map_err(|_| {
                    stored_bytes_mismatch(
                        VerdictStoreTable::JudgingUnitsCompositions,
                        &key_bytes,
                        "组成清单表的键要是 32 字节",
                    )
                })?,
            );
        }
        Ok(found)
    }

    /// 这一块这一种结论在库里存着的每一把版本（按版本字节排）：键对不上时照它找库里那一把，点名变了的单元。
    ///
    /// # Errors
    /// 这一套之前的库；RocksDB 读失败。
    pub fn conclusion_versions_stored_for(
        &self,
        block_key: &VerdictBlockKey,
        kind: &ConclusionKind,
    ) -> Result<Vec<ConclusionVersion>, VerdictStoreError> {
        self.refuse_unless_the_conclusion_kinds_layout()?;
        let mut prefix_after_the_tag = block_key.to_block_key_bytes();
        prefix_after_the_tag.extend_from_slice(&conclusion_kind_key_bytes(kind));
        let mut prefix = vec![VerdictStoreTable::ConclusionBlocks.key_tag()];
        prefix.extend_from_slice(&prefix_after_the_tag);
        let prefix_length = prefix.len();
        let mut found = Vec::new();
        for entry in self.entries_with_key_prefix_kept_in_or_out(VerdictStoreKeyBytes(prefix)) {
            let (key_bytes, _value_bytes) = entry.map_err(VerdictStoreError::RocksDatabase)?;
            if let Ok(version) = <[u8; SHA256_DIGEST_BYTES]>::try_from(&key_bytes[prefix_length..])
            {
                found.push(ConclusionVersion(version));
            }
        }
        Ok(found)
    }

    /// 判法单元指纹表里全部行的原始键值（导入用）。
    fn judging_units_composition_rows(&self) -> Result<RawRows, VerdictStoreError> {
        let prefix = VerdictStoreKey::WholeTablePrefix(VerdictStoreTable::JudgingUnitsCompositions)
            .to_key_bytes();
        let mut rows = Vec::new();
        for entry in self.entries_with_key_prefix_kept_in_or_out(prefix) {
            let (key_bytes, value_bytes) = entry.map_err(VerdictStoreError::RocksDatabase)?;
            rows.push((key_bytes.to_vec(), value_bytes.to_vec()));
        }
        Ok(rows)
    }

    /// 把另一台的库导进来（两台各写各的库、最后并成一个）：录入表、路径索引、事实表、核对表与判法单元指纹表逐行照抄；判定块按判定向量的内容在这个库里
    /// 重新登记编号，连同违例一起写、判法单元指纹照抄；这个库里已有同一块键同一版本、判定相同的算已导入，不同就报错。另一台有结论种类那几张表的，
    /// 每块每种结论连同违例、计数向量、块属性与恢复字按同一个办法导（[`Self::store_conclusion_block`]）。交回导入的判定块数与结论块数之和。
    ///
    /// # Errors
    /// RocksDB 读写失败；同一块键两边判定不同；判定向量超过 2^32 种；另一台有结论而这个库是这一套之前的布局。
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
            .chain(other.judging_units_composition_rows()?)
        {
            let mut batch = rocksdb::WriteBatch::default();
            batch.put(&key_bytes, &value_bytes);
            self.write_batch_keeping_the_kept_out_rows_in_this_process(
                batch,
                &synced_write_options(),
            )
            .map_err(VerdictStoreError::RocksDatabase)?;
        }
        let mut imported = 0u64;
        for (block_key, version) in other.stored_verdict_block_keys()? {
            let block = other
                .read_verdict_block(&block_key, &version)?
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
                .violations_in_block(&block_key, &version)?
                .into_iter()
                .map(|violation| violation.offset_in_block)
                .collect();
            let renumbered_block = VerdictBlock::new(renumbered).expect("读回的块至少一个状态");
            self.store_verdict_block(&block_key, &version, &renumbered_block, &violating)?;
            imported += 1;
        }
        let conclusion_keys = match other.layout {
            VerdictStoreLayout::ConclusionKinds => other.stored_conclusion_keys()?,
            VerdictStoreLayout::BeforeConclusionKinds => Vec::new(),
        };
        for (block_key, kind, conclusion_version) in conclusion_keys {
            let stored = other
                .read_conclusion_block(&block_key, &kind, &conclusion_version)?
                .expect("刚列出来的结论块读得到");
            let mut renumbered_states =
                Vec::with_capacity(stored.block.verdict_vector_number_of_each_state().len());
            for number in stored.block.verdict_vector_number_of_each_state() {
                let vector = other
                    .verdict_vector_of_number(*number)
                    .expect("结论块里的编号在它自己的库里登记过")
                    .clone();
                renumbered_states.push(self.register_verdict_vector(&vector)?);
            }
            let record = ConclusionBlockRecord {
                block: VerdictBlock::new(renumbered_states).expect("读回的结论块至少一个状态"),
                violating_states: stored
                    .violations
                    .iter()
                    .map(|violation| violation.offset_in_block)
                    .collect(),
                count_vector: stored.count_vector,
                judged_on: stored.judged_on,
                recovery_words: stored.recovery_words,
            };
            self.store_conclusion_block_with_the_abandoned_roots_it_may_name(
                &block_key,
                &kind,
                &conclusion_version,
                &record,
                stored.abandoned_roots_named_by_the_pool_checker,
            )?;
            imported += 1;
        }
        Ok(imported)
    }

    /// 按计划导入（[`PlannedImportKeys`]）：只读点名的键，不扫两边的整张表。每一项先在对方库里读，没有就跳过（第二台没判到的块）；
    /// 结论按判定向量的内容换成这个库的编号（这个库没登记过的判定向量随行一起登记），再照 [`Self::store_conclusion_block`] 的规矩与本库同键的比：
    /// 逐项相同跳过、不写，结论不同或块属性不同报错；本库没有的连同新登记的判定向量放进一批。路径、核对行与事实行本库已有且逐字节相同的跳过，
    /// 不同的照写（与 [`Self::record_path`]、[`Self::store_verifier_block`]、[`Self::store_flow_facts`] 同规矩）。一块一种结论的行在同一批里
    /// （块内原子），一批攒到 [`PLANNED_IMPORT_BATCH_BYTES`] 字节同步写一次，最后一批收尾时写。中途出错时已写下的批留着、正在攒的那一批不写，
    /// 重跑同一条命令补齐（写下的都是对方库里有的行，重跑逐项相同就跳过）。
    ///
    /// 读写次数（N 是点名的项数，与两边库里别的流、别的版本的行数无关）：每项结论对方库四次点读加一次只扫这一块的违例前缀扫、本库一次点读
    /// （本库有这一块时再三次点读加一次前缀扫，没有时再一次点读看恢复字旁表）；每条路径、每个核对行两边各一次点读；核对行点名的每份事实
    /// 两边各一次点读；同步写的批数是写下的字节数除以批长向上取整。每个状态换编号查一张 256 格的表，每种判定向量只按内容查一次。
    ///
    /// # Errors
    /// 点名了结论而这个库是这一套之前的布局；RocksDB 读写失败；存着的行不合布局（违例的块内序号不在块里、核对行不到 32 字节这类）；
    /// 同一（块键，种类，版本）两边结论不同或块属性不同；判定向量超过 2^32 种。
    pub fn import_the_planned_rows_from(
        &mut self,
        other: &VerdictStore,
        planned: &PlannedImportKeys,
    ) -> Result<PlannedImportTally, VerdictStoreError> {
        self.import_the_planned_rows_in_batches_of(other, planned, PLANNED_IMPORT_BATCH_BYTES)
    }

    /// 同 [`Self::import_the_planned_rows_from`]，一批攒到 `batch_size_in_bytes` 字节就同步写一次。产品只经那一个入口、批长取
    /// [`PLANNED_IMPORT_BATCH_BYTES`]；这里的批长单元测试给小的，在几百字节的库上也走多批（换批、跨批的编号接续）。
    ///
    /// # Errors
    /// 同 [`Self::import_the_planned_rows_from`]。
    fn import_the_planned_rows_in_batches_of(
        &mut self,
        other: &VerdictStore,
        planned: &PlannedImportKeys,
        batch_size_in_bytes: usize,
    ) -> Result<PlannedImportTally, VerdictStoreError> {
        if !planned.conclusions.is_empty() {
            self.refuse_unless_the_conclusion_kinds_layout()?;
        }
        let mut tally = PlannedImportTally::default();
        let mut pending = PlannedImportBatch::empty();
        // 对方库的编号换成这个库的编号：一次导入里两边的编号都不变（新登记的照次序接在后面），按对方的编号记一次就够
        // 一格一个对方库登记过的编号：对方库读回的块里的编号都登记过（read_conclusion_block 逐个核过），落不到格外
        let mut this_number_of_each_other_number: Vec<Option<VerdictVectorNumber>> =
            vec![None; other.verdict_vector_by_number.len()];
        let mut conclusion_keys_seen: HashSet<Vec<u8>> = HashSet::new();
        for (block_key, kind, conclusion_version) in &planned.conclusions {
            let conclusion_key_bytes =
                VerdictStoreKey::ConclusionBlock(block_key, kind, conclusion_version)
                    .to_key_bytes();
            if !conclusion_keys_seen.insert(conclusion_key_bytes.0.clone()) {
                continue;
            }
            let other_conclusion = match other.layout {
                VerdictStoreLayout::ConclusionKinds => {
                    other.read_conclusion_block(block_key, kind, conclusion_version)?
                }
                VerdictStoreLayout::BeforeConclusionKinds => None,
            };
            let Some(other_conclusion) = other_conclusion else {
                tally.conclusion_blocks_missing_in_the_other_library += 1;
                continue;
            };
            let requested = self.conclusion_renumbered_for_the_planned_import(
                other,
                other_conclusion,
                &conclusion_key_bytes,
                &mut this_number_of_each_other_number,
                &mut pending,
            )?;
            match self.stored_conclusion_compared_with(
                block_key,
                kind,
                conclusion_version,
                &requested,
            )? {
                StoredConclusionComparison::StoredIdentically
                | StoredConclusionComparison::StoredIdenticallyOnAnotherJudgingDevice => {
                    tally.conclusion_blocks_already_identical += 1;
                }
                StoredConclusionComparison::NotStored => {
                    put_conclusion_rows_into_the_batch(
                        &mut pending.batch,
                        block_key,
                        kind,
                        conclusion_version,
                        &requested,
                    );
                    tally.conclusion_blocks_written += 1;
                    if let Some(rows) = self.write_the_planned_import_batch_when_full(
                        &mut pending,
                        batch_size_in_bytes,
                    )? {
                        tally.count_a_written_batch(rows);
                    }
                }
            }
        }
        let mut paths_seen: HashSet<&str> = HashSet::new();
        for path in &planned.paths {
            if !paths_seen.insert(path.as_str()) {
                continue;
            }
            let Some(identity) = other.identity_of_path(path)? else {
                continue;
            };
            if self.identity_of_path(path)?.as_deref() == Some(identity.as_str()) {
                continue;
            }
            pending.batch.put(
                VerdictStoreKey::Path(path).to_key_bytes().0,
                identity.as_bytes(),
            );
            tally.paths_written += 1;
        }
        let mut verifier_keys_seen: HashSet<Vec<u8>> = HashSet::new();
        let mut facts_digests_named: BTreeSet<[u8; SHA256_DIGEST_BYTES]> = BTreeSet::new();
        for (block_key, verifier_digest) in &planned.verifier_blocks {
            let key_bytes =
                VerdictStoreKey::VerifierBlock(block_key, verifier_digest).to_key_bytes();
            if !verifier_keys_seen.insert(key_bytes.0.clone()) {
                continue;
            }
            let Some(value_bytes) = other
                .database
                .get(&key_bytes.0)
                .map_err(VerdictStoreError::RocksDatabase)?
            else {
                continue;
            };
            let facts_digest = value_bytes
                .get(..SHA256_DIGEST_BYTES)
                .and_then(|digest_bytes| <[u8; SHA256_DIGEST_BYTES]>::try_from(digest_bytes).ok())
                .ok_or_else(|| {
                    stored_bytes_mismatch(
                        VerdictStoreTable::VerifierVerdicts,
                        &key_bytes.0,
                        "核对行的值要以 32 字节事实摘要起头",
                    )
                })?;
            facts_digests_named.insert(facts_digest);
            let stored_value = self
                .read_value_kept_in_or_out(&key_bytes.0)
                .map_err(VerdictStoreError::RocksDatabase)?;
            if stored_value.as_deref() == Some(value_bytes.as_slice()) {
                continue;
            }
            pending.batch.put(&key_bytes.0, &value_bytes);
            tally.verifier_blocks_written += 1;
            if let Some(rows) =
                self.write_the_planned_import_batch_when_full(&mut pending, batch_size_in_bytes)?
            {
                tally.count_a_written_batch(rows);
            }
        }
        for facts_digest in &facts_digests_named {
            let Some(facts_bytes) = other.read_flow_facts(facts_digest)? else {
                continue;
            };
            if self.read_flow_facts(facts_digest)?.as_deref() == Some(facts_bytes.as_slice()) {
                continue;
            }
            pending.batch.put(
                VerdictStoreKey::FlowFacts(facts_digest).to_key_bytes().0,
                &facts_bytes,
            );
            tally.flow_facts_written += 1;
            if let Some(rows) =
                self.write_the_planned_import_batch_when_full(&mut pending, batch_size_in_bytes)?
            {
                tally.count_a_written_batch(rows);
            }
        }
        if !pending.batch.is_empty() {
            tally.count_a_written_batch(self.write_the_planned_import_batch(pending)?);
        }
        Ok(tally)
    }

    /// 攒着的那一批到了 `batch_size_in_bytes` 字节（产品取 [`PLANNED_IMPORT_BATCH_BYTES`]）就同步写下去、换一批空的接着攒；
    /// 交回写下的行数，没到交回 None。
    ///
    /// # Errors
    /// RocksDB 写失败。
    fn write_the_planned_import_batch_when_full(
        &mut self,
        pending: &mut PlannedImportBatch,
        batch_size_in_bytes: usize,
    ) -> Result<Option<u64>, VerdictStoreError> {
        if pending.batch.size_in_bytes() < batch_size_in_bytes {
            return Ok(None);
        }
        let full_batch = std::mem::replace(pending, PlannedImportBatch::empty());
        self.write_the_planned_import_batch(full_batch).map(Some)
    }

    /// 对方库读回的一块一种结论换成这个库的编号：每种判定向量按内容找这个库的编号，这个库没有的在攒着的那一批里登记（编号接在已登记的
    /// 与这一批已登记的后面）；违例的编号取换过编号的块里那个状态的（同 [`Self::store_conclusion_block`]）。
    ///
    /// # Errors
    /// 判定向量超过 2^32 种（`EveryU32VerdictVectorNumberIsAlreadyRegistered`，这一批没写）；对方库的违例的块内序号不在块里。
    fn conclusion_renumbered_for_the_planned_import(
        &self,
        other: &VerdictStore,
        other_conclusion: StoredConclusionBlock,
        conclusion_key_bytes: &VerdictStoreKeyBytes,
        this_number_of_each_other_number: &mut [Option<VerdictVectorNumber>],
        pending: &mut PlannedImportBatch,
    ) -> Result<StoredConclusionBlock, VerdictStoreError> {
        let other_numbers = other_conclusion.block.verdict_vector_number_of_each_state();
        let mut this_numbers = Vec::with_capacity(other_numbers.len());
        for other_number in other_numbers {
            let slot = this_number_of_each_other_number
                .get_mut(other_number.position_in_the_registration_order())
                .expect("换编号的表一格一个对方库登记过的编号，对方库读回的编号都登记过");
            let this_number = match *slot {
                Some(this_number) => this_number,
                None => {
                    let vector = other.verdict_vector_of_number(*other_number).expect(
                        "对方库读回的结论块里的编号在它自己的库里登记过（read_conclusion_block 逐个核过）",
                    );
                    let this_number =
                        self.verdict_vector_number_for_the_planned_import(vector, pending)?;
                    *slot = Some(this_number);
                    this_number
                }
            };
            this_numbers.push(this_number);
        }
        let block = VerdictBlock::new(this_numbers).expect("读回的结论块至少一个状态");
        let mut violations = Vec::with_capacity(other_conclusion.violations.len());
        for violation in &other_conclusion.violations {
            let Some(verdict_vector_number) =
                block.verdict_vector_number_at(violation.offset_in_block)
            else {
                return Err(stored_bytes_mismatch(
                    VerdictStoreTable::ConclusionViolations,
                    &conclusion_key_bytes.0,
                    "对方库这一块这一种结论有一处违例的块内序号不在块里",
                ));
            };
            violations.push(StoredViolation {
                offset_in_block: violation.offset_in_block,
                verdict_vector_number,
            });
        }
        Ok(StoredConclusionBlock {
            block,
            violations,
            count_vector: other_conclusion.count_vector,
            judged_on: other_conclusion.judged_on,
            recovery_words: other_conclusion.recovery_words,
            abandoned_roots_named_by_the_pool_checker: other_conclusion
                .abandoned_roots_named_by_the_pool_checker,
            circumstances: other_conclusion.circumstances,
        })
    }

    /// 一种判定向量在这个库里的编号：登记过的照旧；这一批里已登记的照它在批里的次序；都没有的放进这一批登记（两张表各一行），编号接在后面。
    ///
    /// # Errors
    /// 加上这一种就超过 2^32 种（`EveryU32VerdictVectorNumberIsAlreadyRegistered`，这一批没写）。
    fn verdict_vector_number_for_the_planned_import(
        &self,
        verdict_vector: &EncodedVerdictVector,
        pending: &mut PlannedImportBatch,
    ) -> Result<VerdictVectorNumber, VerdictStoreError> {
        if let Some(registered_number) =
            self.verdict_vector_number_by_content.get(&verdict_vector.0)
        {
            return Ok(*registered_number);
        }
        let registered_count = self.verdict_vector_by_number.len();
        if let Some(position_in_the_batch) = pending
            .verdict_vectors_registered_in_the_batch
            .iter()
            .position(|registered_in_the_batch| registered_in_the_batch == verdict_vector)
        {
            return Ok(VerdictVectorNumber::next_after_registered_count(
                registered_count + position_in_the_batch,
            )
            .expect("放进这一批时核过编号装得进 u32"));
        }
        let Some(number) = VerdictVectorNumber::next_after_registered_count(
            registered_count + pending.verdict_vectors_registered_in_the_batch.len(),
        ) else {
            return Err(VerdictStoreError::EveryU32VerdictVectorNumberIsAlreadyRegistered);
        };
        pending.batch.put(
            VerdictStoreKey::VerdictVectorNumberByContent(verdict_vector)
                .to_key_bytes()
                .0,
            number.to_stored_bytes(),
        );
        pending.batch.put(
            VerdictStoreKey::VerdictVectorContentByNumber(number)
                .to_key_bytes()
                .0,
            &verdict_vector.0,
        );
        pending
            .verdict_vectors_registered_in_the_batch
            .push(verdict_vector.clone());
        Ok(number)
    }

    /// 把攒好的一批同步写下去，写下去之后这一批登记的判定向量才进内存表；交回写下的键值行数。
    ///
    /// # Errors
    /// RocksDB 写失败（这一批一行都没进库，内存表也不动）。
    fn write_the_planned_import_batch(
        &mut self,
        pending: PlannedImportBatch,
    ) -> Result<u64, VerdictStoreError> {
        let rows = u64::try_from(pending.batch.len()).expect("一批的行数装得进 u64");
        self.write_batch_keeping_the_kept_out_rows_in_this_process(
            pending.batch,
            &synced_write_options(),
        )
        .map_err(VerdictStoreError::RocksDatabase)?;
        for verdict_vector in pending.verdict_vectors_registered_in_the_batch {
            let number = VerdictVectorNumber::next_after_registered_count(
                self.verdict_vector_by_number.len(),
            )
            .expect("放进这一批时核过编号装得进 u32");
            self.verdict_vector_number_by_content
                .insert(verdict_vector.0.clone(), number);
            self.verdict_vector_by_number.push(verdict_vector);
        }
        Ok(rows)
    }

    /// 库的布局：建库时写的布局行（[`VerdictStore::create_empty`] 写），这一套之前的库没有这一行。
    #[must_use]
    pub fn layout(&self) -> VerdictStoreLayout {
        self.layout
    }

    /// 按结论种类存取之前先看布局：这一套之前的库一律拒绝、什么都不写。
    fn refuse_unless_the_conclusion_kinds_layout(&self) -> Result<(), VerdictStoreError> {
        match self.layout {
            VerdictStoreLayout::ConclusionKinds => Ok(()),
            VerdictStoreLayout::BeforeConclusionKinds => {
                Err(VerdictStoreError::LibraryLayoutPredatesConclusionKinds {
                    directory: self.directory.clone(),
                })
            }
        }
    }

    /// 存一块一种结论（第三轮判决 J7、J8，第二轮判决 L1-8）：结论块、它的违例、计数向量、块属性与恢复字同一批同步写下去之后才返回；
    /// 违例状态的判定向量编号取块里那个状态的。同一块在别的种类、别的版本下的行不受影响。
    ///
    /// # Errors
    /// 这一套之前的库（`LibraryLayoutPredatesConclusionKinds`，什么都没写）；这个（块键，种类，版本）下已经存着不同的结论
    /// （`DifferentConclusionUnderAStoredKeyNotDecidedInTheFirstVersion`，什么都没写）；已存的行读回来不合布局；RocksDB 读写失败。
    /// 结论相同、只有在哪判的不同（同一张表先后分到两张卡上判）交回 `AlreadyStoredIdentically`、留先存的那一份块属性。
    ///
    /// # Panics
    /// 块里有这个库没登记过的编号、违例状态的块内序号不在块里、恢复字条数与块长不等：都是调用方的 bug。
    pub fn store_conclusion_block(
        &mut self,
        block_key: &VerdictBlockKey,
        kind: &ConclusionKind,
        conclusion_version: &ConclusionVersion,
        record: &ConclusionBlockRecord,
    ) -> Result<VerdictBlockStoring, VerdictStoreError> {
        self.store_conclusion_block_with_the_abandoned_roots_it_may_name(
            block_key,
            kind,
            conclusion_version,
            record,
            None,
        )
    }

    /// 同 [`Self::store_conclusion_block`]，连同每个状态上池级 checker 判 I-7.4 时违例细节里点名的被抛弃根（点名的被抛弃根旁表）同一批写；
    /// 库里这个（块键，种类，版本）下已经存着的结论连同点名的根逐项相同才算已存，点名的根不同照「结论不同」报错。给定状态表那一路经它存。
    ///
    /// # Errors
    /// 同 [`Self::store_conclusion_block`]。
    ///
    /// # Panics
    /// 同 [`Self::store_conclusion_block`]；点名的根不是块里每个状态一份。
    pub fn store_conclusion_block_naming_the_abandoned_roots(
        &mut self,
        block_key: &VerdictBlockKey,
        kind: &ConclusionKind,
        conclusion_version: &ConclusionVersion,
        record: &ConclusionBlockRecord,
        abandoned_roots_named_of_each_state: &[Vec<AbandonedRootNamedByThePoolChecker>],
    ) -> Result<VerdictBlockStoring, VerdictStoreError> {
        assert_eq!(
            abandoned_roots_named_of_each_state.len(),
            record.block.verdict_vector_number_of_each_state().len(),
            "点名的被抛弃根每个状态一份：调用方给的条数与块长不等"
        );
        self.store_conclusion_block_with_the_abandoned_roots_it_may_name(
            block_key,
            kind,
            conclusion_version,
            record,
            Some(abandoned_roots_named_of_each_state.to_vec()),
        )
    }

    /// [`Self::store_conclusion_block`] 与 [`Self::store_conclusion_block_naming_the_abandoned_roots`] 的本体：点名的根给了（`Some`）就同一批写旁表。
    fn store_conclusion_block_with_the_abandoned_roots_it_may_name(
        &mut self,
        block_key: &VerdictBlockKey,
        kind: &ConclusionKind,
        conclusion_version: &ConclusionVersion,
        record: &ConclusionBlockRecord,
        abandoned_roots_named_by_the_pool_checker: Option<
            Vec<Vec<AbandonedRootNamedByThePoolChecker>>,
        >,
    ) -> Result<VerdictBlockStoring, VerdictStoreError> {
        self.refuse_unless_the_conclusion_kinds_layout()?;
        let registered_count = self.verdict_vector_by_number.len();
        let numbers_of_each_state = record.block.verdict_vector_number_of_each_state();
        assert!(
            numbers_of_each_state.iter().all(|state_number| state_number
                .position_in_the_registration_order()
                < registered_count),
            "结论块里的判定向量编号必须是这个库登记过的（这个库登记了 {registered_count} 种）"
        );
        if let Some(recovery_words) = &record.recovery_words {
            assert_eq!(
                recovery_words.len(),
                numbers_of_each_state.len(),
                "恢复字每个状态一份：调用方给的条数与块长不等"
            );
        }
        let requested = StoredConclusionBlock {
            block: record.block.clone(),
            violations: record
                .violating_states
                .iter()
                .copied()
                .collect::<BTreeSet<StateOffsetInBlock>>()
                .into_iter()
                .map(|offset_in_block| StoredViolation {
                    offset_in_block,
                    verdict_vector_number: record
                        .block
                        .verdict_vector_number_at(offset_in_block)
                        .expect("结论违例的块内序号必须在块里（调用方给的序号超出了块长）"),
                })
                .collect(),
            count_vector: record.count_vector.clone(),
            judged_on: record.judged_on.clone(),
            recovery_words: record.recovery_words.clone(),
            abandoned_roots_named_by_the_pool_checker,
            circumstances: JudgementCircumstances::of_this_process_now(),
        };
        match self.stored_conclusion_compared_with(
            block_key,
            kind,
            conclusion_version,
            &requested,
        )? {
            StoredConclusionComparison::StoredIdentically
            | StoredConclusionComparison::StoredIdenticallyOnAnotherJudgingDevice => {
                return Ok(VerdictBlockStoring::AlreadyStoredIdentically)
            }
            StoredConclusionComparison::NotStored => {}
        }
        let mut batch = rocksdb::WriteBatch::default();
        put_conclusion_rows_into_the_batch(
            &mut batch,
            block_key,
            kind,
            conclusion_version,
            &requested,
        );
        self.write_batch_keeping_the_kept_out_rows_in_this_process(batch, &synced_write_options())
            .map_err(VerdictStoreError::RocksDatabase)?;
        Ok(VerdictBlockStoring::WrittenAndSynced)
    }

    /// 要存的一块一种结论（编号已是这个库的）与库里这个（块键，种类，版本）下存着的比：没存过（恢复字旁表也没有这一行）交回
    /// `NotStored`；块、违例、计数向量、恢复字与块属性都相同交回 `StoredIdentically`；只有块属性（在哪判的）不同交回
    /// `StoredIdenticallyOnAnotherJudgingDevice`。存块与按计划导入都经它，规矩只在这一处。
    ///
    /// # Errors
    /// 结论不同（`DifferentConclusionUnderAStoredKeyNotDecidedInTheFirstVersion`）；恢复字旁表有这一行而结论块表没有；
    /// 已存的行读回来不合布局；RocksDB 读失败。
    fn stored_conclusion_compared_with(
        &self,
        block_key: &VerdictBlockKey,
        kind: &ConclusionKind,
        conclusion_version: &ConclusionVersion,
        requested: &StoredConclusionBlock,
    ) -> Result<StoredConclusionComparison, VerdictStoreError> {
        if let Some(stored) = self.read_conclusion_block(block_key, kind, conclusion_version)? {
            let is_the_same_conclusion = stored.block == requested.block
                && stored.violations == requested.violations
                && stored.count_vector == requested.count_vector
                && stored.recovery_words == requested.recovery_words
                && stored.abandoned_roots_named_by_the_pool_checker
                    == requested.abandoned_roots_named_by_the_pool_checker;
            if !is_the_same_conclusion {
                return Err(
                    VerdictStoreError::DifferentConclusionUnderAStoredKeyNotDecidedInTheFirstVersion {
                        block_key: block_key.clone(),
                        kind: kind.clone(),
                    },
                );
            }
            if stored.judged_on != requested.judged_on {
                return Ok(StoredConclusionComparison::StoredIdenticallyOnAnotherJudgingDevice);
            }
            return Ok(StoredConclusionComparison::StoredIdentically);
        }
        let recovery_words_key =
            VerdictStoreKey::RecoveryWords(block_key, kind, conclusion_version).to_key_bytes();
        let stray_recovery_words = self
            .read_value_kept_in_or_out(&recovery_words_key.0)
            .map_err(VerdictStoreError::RocksDatabase)?;
        if stray_recovery_words.is_some() {
            return Err(stored_bytes_mismatch(
                VerdictStoreTable::RecoveryWords,
                &recovery_words_key.0,
                "恢复字旁表有这一块这一种结论这一版本，结论块表没有",
            ));
        }
        let abandoned_roots_key = VerdictStoreKey::AbandonedRootsNamedByThePoolChecker(
            block_key,
            kind,
            conclusion_version,
        )
        .to_key_bytes();
        if self
            .read_value_kept_in_or_out(&abandoned_roots_key.0)
            .map_err(VerdictStoreError::RocksDatabase)?
            .is_some()
        {
            return Err(stored_bytes_mismatch(
                VerdictStoreTable::AbandonedRootsNamedByThePoolChecker,
                &abandoned_roots_key.0,
                "点名的被抛弃根旁表有这一块这一种结论这一版本，结论块表没有",
            ));
        }
        Ok(StoredConclusionComparison::NotStored)
    }

    /// 读回一块一种结论，连同违例、计数向量、块属性与恢复字；这个（块键，种类，版本）没存过时是 `None`（别的种类、别的版本存过也是 `None`）。
    ///
    /// # Errors
    /// 这一套之前的库；存着的结论块是空的或点名了没登记的编号、缺计数向量或块属性、计数向量或块属性或恢复字不合布局；RocksDB 读失败。
    pub fn read_conclusion_block(
        &self,
        block_key: &VerdictBlockKey,
        kind: &ConclusionKind,
        conclusion_version: &ConclusionVersion,
    ) -> Result<Option<StoredConclusionBlock>, VerdictStoreError> {
        self.refuse_unless_the_conclusion_kinds_layout()?;
        let conclusion_key =
            VerdictStoreKey::ConclusionBlock(block_key, kind, conclusion_version).to_key_bytes();
        let Some(conclusion_value_bytes) = self
            .read_value_kept_in_or_out(&conclusion_key.0)
            .map_err(VerdictStoreError::RocksDatabase)?
        else {
            return Ok(None);
        };
        let numbers_of_each_state = self.registered_verdict_vector_numbers_of_a_stored_block(
            VerdictStoreTable::ConclusionBlocks,
            &conclusion_key.0,
            &conclusion_value_bytes,
        )?;
        let state_count = numbers_of_each_state.len();
        let block =
            VerdictBlock::new(numbers_of_each_state).map_err(|EmptyVerdictBlockRejected| {
                stored_bytes_mismatch(
                    VerdictStoreTable::ConclusionBlocks,
                    &conclusion_key.0,
                    "存着的结论块值是空的",
                )
            })?;
        let read_required_value = |table: VerdictStoreTable, key_bytes: VerdictStoreKeyBytes| {
            self.read_value_kept_in_or_out(&key_bytes.0)
                .map_err(VerdictStoreError::RocksDatabase)?
                .ok_or_else(|| stored_bytes_mismatch(table, &key_bytes.0, "结论块在而这一行缺着"))
                .map(|value_bytes| (key_bytes, value_bytes))
        };
        let (count_key, count_bytes) = read_required_value(
            VerdictStoreTable::ConclusionCountVectors,
            VerdictStoreKey::ConclusionCountVector(block_key, kind, conclusion_version)
                .to_key_bytes(),
        )?;
        let count_vector = count_vector_from_bytes(&count_bytes).ok_or_else(|| {
            stored_bytes_mismatch(
                VerdictStoreTable::ConclusionCountVectors,
                &count_key.0,
                "计数向量的字节不合布局",
            )
        })?;
        let (attribute_key, attribute_bytes) = read_required_value(
            VerdictStoreTable::ConclusionBlockAttributes,
            VerdictStoreKey::ConclusionBlockAttributes(block_key, kind, conclusion_version)
                .to_key_bytes(),
        )?;
        let (judged_on, circumstances) = conclusion_block_attributes_from_bytes(&attribute_bytes)
            .ok_or_else(|| {
            stored_bytes_mismatch(
                VerdictStoreTable::ConclusionBlockAttributes,
                &attribute_key.0,
                "块属性的字节不合布局",
            )
        })?;
        let recovery_words_key =
            VerdictStoreKey::RecoveryWords(block_key, kind, conclusion_version).to_key_bytes();
        let recovery_words = match self
            .read_value_kept_in_or_out(&recovery_words_key.0)
            .map_err(VerdictStoreError::RocksDatabase)?
        {
            None => None,
            Some(words_bytes) => Some(
                recovery_words_from_value_bytes(&words_bytes, state_count).ok_or_else(|| {
                    stored_bytes_mismatch(
                        VerdictStoreTable::RecoveryWords,
                        &recovery_words_key.0,
                        "恢复字不是块里每个状态八个字",
                    )
                })?,
            ),
        };
        let abandoned_roots_key = VerdictStoreKey::AbandonedRootsNamedByThePoolChecker(
            block_key,
            kind,
            conclusion_version,
        )
        .to_key_bytes();
        let abandoned_roots_named_by_the_pool_checker = match self
            .read_value_kept_in_or_out(&abandoned_roots_key.0)
            .map_err(VerdictStoreError::RocksDatabase)?
        {
            None => None,
            Some(roots_bytes) => Some(
                abandoned_roots_named_from_value_bytes(&roots_bytes, state_count).ok_or_else(
                    || {
                        stored_bytes_mismatch(
                            VerdictStoreTable::AbandonedRootsNamedByThePoolChecker,
                            &abandoned_roots_key.0,
                            "点名的被抛弃根不是块里每个状态一份",
                        )
                    },
                )?,
            ),
        };
        Ok(Some(StoredConclusionBlock {
            block,
            violations: self.conclusion_violations_in_block(block_key, kind, conclusion_version)?,
            count_vector,
            judged_on,
            recovery_words,
            abandoned_roots_named_by_the_pool_checker,
            circumstances,
        }))
    }

    /// 这一块一种结论的违例，按块内序号排。
    fn conclusion_violations_in_block(
        &self,
        block_key: &VerdictBlockKey,
        kind: &ConclusionKind,
        conclusion_version: &ConclusionVersion,
    ) -> Result<Vec<StoredViolation>, VerdictStoreError> {
        let prefix =
            VerdictStoreKey::ConclusionViolationsOfBlockPrefix(block_key, kind, conclusion_version)
                .to_key_bytes();
        let prefix_length = prefix.0.len();
        let mut violations = Vec::new();
        for entry in self.entries_with_key_prefix_kept_in_or_out(prefix) {
            let (key_bytes, number_bytes) = entry.map_err(VerdictStoreError::RocksDatabase)?;
            let offset_bytes =
                <[u8; STATE_OFFSET_IN_BLOCK_BYTES]>::try_from(&key_bytes[prefix_length..]);
            let (Ok(offset_bytes), Some(stored_number)) =
                (offset_bytes, stored_verdict_vector_number(&number_bytes))
            else {
                return Err(stored_bytes_mismatch(
                    VerdictStoreTable::ConclusionViolations,
                    &key_bytes,
                    "结论违例表里版本之后不是 8 字节的块内序号，或值不是 4 字节的编号",
                ));
            };
            violations.push(StoredViolation {
                offset_in_block: StateOffsetInBlock(u64::from_be_bytes(offset_bytes)),
                verdict_vector_number: self.registered_verdict_vector_number(
                    VerdictStoreTable::ConclusionViolations,
                    &key_bytes,
                    stored_number,
                )?,
            });
        }
        Ok(violations)
    }

    /// 一块要的几种结论里哪几种还没有（按给的次序）：「哪一种没有就只排那一种」（第三轮判决 J8）。每一项的版本由调用方给：
    /// GPU 判器一把、这一块的设点文件那一把 CPU、挂在这个节点上的每个观察者各一把。
    ///
    /// # Errors
    /// 这一套之前的库；RocksDB 读失败。
    pub fn conclusions_missing_for_block(
        &self,
        block_key: &VerdictBlockKey,
        required: &[(ConclusionKind, ConclusionVersion)],
    ) -> Result<Vec<(ConclusionKind, ConclusionVersion)>, VerdictStoreError> {
        self.refuse_unless_the_conclusion_kinds_layout()?;
        let mut missing = Vec::new();
        for (kind, conclusion_version) in required {
            let key_bytes = VerdictStoreKey::ConclusionBlock(block_key, kind, conclusion_version)
                .to_key_bytes();
            let stored = self
                .read_value_kept_in_or_out(&key_bytes.0)
                .map_err(VerdictStoreError::RocksDatabase)?;
            if stored.is_none() {
                missing.push((kind.clone(), *conclusion_version));
            }
        }
        Ok(missing)
    }

    /// 结论块表里的全部（块键，种类，版本），按键序：同一块几种结论、几把版本就有几行。
    ///
    /// # Errors
    /// 这一套之前的库；RocksDB 读失败；键字节认不回来。
    pub fn stored_conclusion_keys(
        &self,
    ) -> Result<Vec<(VerdictBlockKey, ConclusionKind, ConclusionVersion)>, VerdictStoreError> {
        self.refuse_unless_the_conclusion_kinds_layout()?;
        self.block_keys_of_table(VerdictStoreTable::ConclusionBlocks)?
            .into_iter()
            .map(|(block_key, suffix, _value)| {
                let unreadable = || {
                    stored_bytes_mismatch(
                        VerdictStoreTable::ConclusionBlocks,
                        &suffix,
                        "结论块的键在块键之后要是结论种类加 32 字节的版本",
                    )
                };
                let (kind, version_bytes) =
                    conclusion_kind_from_key_bytes(&suffix).ok_or_else(unreadable)?;
                let conclusion_version = <[u8; SHA256_DIGEST_BYTES]>::try_from(version_bytes)
                    .map(ConclusionVersion)
                    .map_err(|_| unreadable())?;
                Ok((block_key, kind, conclusion_version))
            })
            .collect()
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
    const TEST_JUDGING_UNITS_FINGERPRINT: JudgingUnitsFingerprint =
        JudgingUnitsFingerprint([0x33; SHA256_DIGEST_BYTES]);
    const OTHER_JUDGING_UNITS_FINGERPRINT: JudgingUnitsFingerprint =
        JudgingUnitsFingerprint([0x44; SHA256_DIGEST_BYTES]);
    const PLAN_HASH_BYTE: u8 = 0x22;

    /// 测试里按名字造的段写表与起步镜像摘要：名字只用来造出互不相同的摘要，块键里没有名字。
    fn node_input_digest_named(fingerprint_byte: u8, name: &str) -> NodeInputDigest {
        let mut message = vec![fingerprint_byte];
        message.extend_from_slice(name.as_bytes());
        NodeInputDigest(singlefs_harness::sha256::sha256_digest(&message))
    }

    fn block_key_of(
        fingerprint_byte: u8,
        name: &str,
        plan_hash_byte: u8,
        block_start: u64,
    ) -> VerdictBlockKey {
        VerdictBlockKey {
            node_input_digest: node_input_digest_named(fingerprint_byte, name),
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
                .store_verdict_block(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT, &block, &[])
                .expect("存块"),
            VerdictBlockStoring::WrittenAndSynced
        );
        assert_eq!(
            store
                .read_verdict_block(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT)
                .expect("读块"),
            Some(block),
            "读回来的块要与存进去的逐个编号相同"
        );
        let stored_value_bytes = store
            .database
            .get(
                VerdictStoreKey::VerdictBlock(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT)
                    .to_key_bytes()
                    .0,
            )
            .expect("读原始值")
            .expect("块在库里");
        assert_eq!(
            stored_value_bytes,
            vec![0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 2],
            "库里的块值要是每个状态一个 4 字节大端的编号"
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
                .store_verdict_block(
                    &block_key,
                    &TEST_JUDGING_UNITS_FINGERPRINT,
                    &block,
                    &[StateOffsetInBlock(2)],
                )
                .expect("存块");
            (clean, torn, block)
        };
        let (mut store, opening) = VerdictStore::open_existing(&directory.path).expect("重开");
        assert_eq!(opening, VerdictStoreOpening::OpenedAtTheFirstAttempt);
        assert_eq!(
            store
                .read_verdict_block(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT)
                .expect("读块"),
            Some(block)
        );
        assert_eq!(
            store
                .violations_in_block(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT)
                .expect("读违例"),
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
                .as_u32(),
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
                    .store_verdict_block(block_key, &TEST_JUDGING_UNITS_FINGERPRINT, block, &[])
                    .expect("存块"),
                VerdictBlockStoring::WrittenAndSynced,
                "块键 {block_key:?} 之前没存过"
            );
        }
        for (block_key, block) in &keys_and_blocks {
            assert_eq!(
                store
                    .read_verdict_block(block_key, &TEST_JUDGING_UNITS_FINGERPRINT)
                    .expect("读块")
                    .as_ref(),
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
                &TEST_JUDGING_UNITS_FINGERPRINT,
                &block_of(&[clean, lost, torn, clean]),
                &[StateOffsetInBlock(2), StateOffsetInBlock(1)],
            )
            .expect("存块");
        store
            .store_verdict_block(
                &key_with_a_longer_name,
                &TEST_JUDGING_UNITS_FINGERPRINT,
                &block_of(&[torn, clean]),
                &[StateOffsetInBlock(0)],
            )
            .expect("存块");
        store
            .store_verdict_block(
                &next_block_key,
                &TEST_JUDGING_UNITS_FINGERPRINT,
                &block_of(&[clean, clean]),
                &[],
            )
            .expect("存块");
        assert_eq!(
            store
                .violations_in_block(&first_key, &TEST_JUDGING_UNITS_FINGERPRINT)
                .expect("读违例"),
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
                .violations_in_block(&key_with_a_longer_name, &TEST_JUDGING_UNITS_FINGERPRINT)
                .expect("读违例"),
            vec![StoredViolation {
                offset_in_block: StateOffsetInBlock(0),
                verdict_vector_number: torn,
            }]
        );
        assert_eq!(
            store
                .violations_in_block(&next_block_key, &TEST_JUDGING_UNITS_FINGERPRINT)
                .expect("读违例"),
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
            .store_verdict_block(
                &block_key,
                &TEST_JUDGING_UNITS_FINGERPRINT,
                &block,
                &[StateOffsetInBlock(1)],
            )
            .expect("存块");
        let entries_before = store.every_stored_entry();
        let different_block = store.store_verdict_block(
            &block_key,
            &TEST_JUDGING_UNITS_FINGERPRINT,
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
        let different_violations =
            store.store_verdict_block(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT, &block, &[]);
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
            .store_verdict_block(
                &block_key,
                &TEST_JUDGING_UNITS_FINGERPRINT,
                &block,
                &[StateOffsetInBlock(0)],
            )
            .expect("存块");
        let entries_before = store.every_stored_entry();
        assert_eq!(
            store
                .store_verdict_block(
                    &block_key,
                    &TEST_JUDGING_UNITS_FINGERPRINT,
                    &block,
                    &[StateOffsetInBlock(0)]
                )
                .expect("再存同一块"),
            VerdictBlockStoring::AlreadyStoredIdentically,
            "续跑时重写逐字节相同的块要放行"
        );
        assert_eq!(store.every_stored_entry(), entries_before);
    }

    /// 用户 2026-10-15 定判定向量编号加宽到 u32：第 257 种登记得上、拿到编号 256，块与违例里存得下、关库重开读得回，之后的接着往下给。
    /// 判别力（`crates/mutations.tsv`）：编号退回一个字节宽（满 256 种就拒），这条红。
    #[test]
    fn the_257th_distinct_verdict_vector_gets_number_256_and_survives_closing_and_reopening() {
        let directory = TemporaryLibraryDirectory::new("vector-past-one-byte");
        let block_key = block_key_named("stream", 0);
        let the_257th_vector = verdict_vector(b"the 257th kind");
        let (clean, the_257th) = {
            let mut store = VerdictStore::create_empty(&directory.path).expect("建库");
            for expected_number in 0..256u32 {
                let number = store
                    .register_verdict_vector(&verdict_vector(&expected_number.to_be_bytes()))
                    .expect("前 256 种都登记得上");
                assert_eq!(number.as_u32(), expected_number);
            }
            let clean = store
                .register_verdict_vector(&verdict_vector(&0u32.to_be_bytes()))
                .expect("登记过的照旧");
            let the_257th = store
                .register_verdict_vector(&the_257th_vector)
                .expect("第 257 种登记得上");
            assert_eq!(the_257th.as_u32(), 256, "第 257 种拿到编号 256");
            assert_eq!(
                store
                    .database
                    .get(
                        VerdictStoreKey::VerdictVectorContentByNumber(the_257th)
                            .to_key_bytes()
                            .0
                    )
                    .expect("读原始值"),
                Some(the_257th_vector.0.clone()),
                "按编号那张表里编号 256 的键在、值是第 257 种的字节"
            );
            store
                .store_verdict_block(
                    &block_key,
                    &TEST_JUDGING_UNITS_FINGERPRINT,
                    &block_of(&[clean, the_257th]),
                    &[StateOffsetInBlock(1)],
                )
                .expect("块里存得下编号 256");
            (clean, the_257th)
        };
        let (mut store, _opening) = VerdictStore::open_existing(&directory.path).expect("重开");
        assert_eq!(
            store
                .read_verdict_block(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT)
                .expect("读块"),
            Some(block_of(&[clean, the_257th])),
            "重开之后块里的编号 256 读得回"
        );
        assert_eq!(
            store
                .violations_in_block(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT)
                .expect("读违例"),
            vec![StoredViolation {
                offset_in_block: StateOffsetInBlock(1),
                verdict_vector_number: the_257th,
            }],
            "重开之后违例里的编号 256 读得回"
        );
        assert_eq!(
            store.verdict_vector_of_number(the_257th),
            Some(&the_257th_vector)
        );
        assert_eq!(
            store
                .register_verdict_vector(&verdict_vector(b"the 258th kind"))
                .expect("第 258 种登记得上")
                .as_u32(),
            257,
            "重开之后新登记的接着往下给"
        );
    }

    /// 编号一直给到 u32 的最大值：已经登记了 u32::MAX 种时下一种拿到 u32::MAX，存成 4 字节大端、读回相同；2^32 种都登记过了才没有下一个编号。
    #[test]
    fn verdict_vector_numbers_reach_the_largest_u32_and_stop_only_after_it() {
        let registered_count_before_the_largest =
            usize::try_from(u32::MAX).expect("这个包只在 usize 至少 32 位的平台上编");
        let largest =
            VerdictVectorNumber::next_after_registered_count(registered_count_before_the_largest)
                .expect("已经登记了 u32::MAX 种时下一种还有编号");
        assert_eq!(largest.as_u32(), u32::MAX);
        assert_eq!(largest.to_stored_bytes(), [0xFF, 0xFF, 0xFF, 0xFF]);
        assert_eq!(
            stored_verdict_vector_numbers(&largest.to_stored_bytes()),
            Some(vec![u32::MAX]),
            "存成的字节读回同一个编号"
        );
        assert_eq!(
            VerdictVectorNumber::next_after_registered_count(
                registered_count_before_the_largest + 1
            ),
            None,
            "2^32 种都登记过了，没有下一个编号"
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
                .store_verdict_block(
                    &block_key,
                    &TEST_JUDGING_UNITS_FINGERPRINT,
                    &block,
                    &[StateOffsetInBlock(1)],
                )
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
            store
                .read_verdict_block(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT)
                .expect("读块"),
            Some(block)
        );
        assert_eq!(
            store
                .violations_in_block(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT)
                .expect("读违例"),
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
                VerdictStoreKey::VerdictBlock(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT)
                    .to_key_bytes()
                    .0,
                [0, 0, 0, 0, 0, 0, 0, 1],
            )
            .expect("绕过 store_verdict_block 直接写一个点名编号 1 的块值");
        let read = store.read_verdict_block(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT);
        assert!(
            matches!(
                read,
                Err(VerdictStoreError::StoredBytesDoNotMatchTheLayout {
                    table: VerdictStoreTable::VerdictBlocks,
                    reason: "存着的判定向量编号在这个库里没登记过",
                    ..
                })
            ),
            "只登记了编号 0，块值里有 1 要判不合布局：{read:?}"
        );
    }

    /// 块值按一个字节一个编号写的（编号加宽之前的写法）：字节数不是 4 的整数倍，读成不合布局，不把几个状态拼成一个编号。
    #[test]
    fn stored_block_value_written_one_byte_per_state_reads_as_a_layout_mismatch() {
        let directory = TemporaryLibraryDirectory::new("one-byte-block-value");
        let mut store = VerdictStore::create_empty(&directory.path).expect("建库");
        store
            .register_verdict_vector(&verdict_vector(b"clean"))
            .expect("登记");
        let block_key = block_key_named("stream", 0);
        store
            .database
            .put(
                VerdictStoreKey::VerdictBlock(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT)
                    .to_key_bytes()
                    .0,
                [0, 0, 0, 0, 0],
            )
            .expect("绕过 store_verdict_block 直接写五个状态各一个字节的块值");
        let read = store.read_verdict_block(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT);
        assert!(
            matches!(
                read,
                Err(VerdictStoreError::StoredBytesDoNotMatchTheLayout {
                    table: VerdictStoreTable::VerdictBlocks,
                    reason: "存着的块值的字节数不是 4 字节编号的整数倍",
                    ..
                })
            ),
            "五个字节的块值要判不合布局：{read:?}"
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
            &TEST_JUDGING_UNITS_FINGERPRINT,
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
        let _outcome = store.store_verdict_block(
            &block_key,
            &TEST_JUDGING_UNITS_FINGERPRINT,
            &block_of(&[VerdictVectorNumber(1)]),
            &[],
        );
    }

    #[test]
    fn block_without_any_state_is_refused() {
        assert_eq!(
            VerdictBlock::new(Vec::new()),
            Err(EmptyVerdictBlockRejected)
        );
    }

    /// 块键定长、只由内容定：字节读回逐字段相同，后缀原样交回；不够长的字节认不回。
    #[test]
    fn a_block_key_is_fixed_length_content_and_reads_back_from_its_bytes() {
        let key = block_key_of(0x11, "probe", 0x22, 70_000);
        let mut bytes = key.to_block_key_bytes();
        assert_eq!(bytes.len(), BLOCK_KEY_BYTES, "块键定长 72 字节");
        bytes.extend_from_slice(b"suffix");
        assert_eq!(
            VerdictBlockKey::from_block_key_bytes_with_suffix(&bytes),
            Some((key.clone(), &b"suffix"[..]))
        );
        assert_eq!(
            VerdictBlockKey::from_block_key_bytes_with_suffix(&bytes[..BLOCK_KEY_BYTES - 1]),
            None
        );
    }

    /// 判定日期是东京日期：协调世界时 15 点是东京第二天零点；跑法变量认得三种写法，没设而只写不读是全量、否则是剪枝。
    #[test]
    fn the_judged_date_is_the_tokyo_date_and_the_run_kind_comes_from_the_two_variables() {
        let at = |seconds: u64| std::time::UNIX_EPOCH + std::time::Duration::from_secs(seconds);
        assert_eq!(tokyo_date_text_of(at(0)), "1970-01-01");
        assert_eq!(tokyo_date_text_of(at(15 * 3600 - 1)), "1970-01-01");
        assert_eq!(tokyo_date_text_of(at(15 * 3600)), "1970-01-02");
        assert_eq!(tokyo_date_text_of(at(951_782_400)), "2000-02-29", "闰日");
        assert_eq!(tokyo_date_text_of(at(1_791_990_000)), "2026-10-15");
        assert_eq!(
            RunKindOfTheJudgement::of_the_variable_values(None, None),
            RunKindOfTheJudgement::Pruned
        );
        assert_eq!(
            RunKindOfTheJudgement::of_the_variable_values(None, Some("1")),
            RunKindOfTheJudgement::Full
        );
        assert_eq!(
            RunKindOfTheJudgement::of_the_variable_values(Some("item_by_item"), Some("1")),
            RunKindOfTheJudgement::ItemByItem
        );
        assert!(std::panic::catch_unwind(|| {
            RunKindOfTheJudgement::of_the_variable_values(Some("everything"), None)
        })
        .is_err());
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

    /// 同一块在两个判法单元指纹下各存一份判定：各自读得回、互不相扰（复用只认同一版本），判定块表列出两行，判法单元指纹表记得两版的描述，
    /// 导入把两版连同描述一起带过去。
    #[test]
    fn the_same_block_judged_under_two_judging_units_fingerprints_keeps_both_rows_and_reads_only_its_own_fingerprint(
    ) {
        let directory = TemporaryLibraryDirectory::new("two-judge-versions");
        let mut store = VerdictStore::create_empty(&directory.path).expect("建库");
        let clean = store
            .register_verdict_vector(&verdict_vector(b""))
            .expect("登记");
        let red = store
            .register_verdict_vector(&verdict_vector(b"red"))
            .expect("登记");
        let block_key = block_key_named("stream", 0);
        store
            .store_verdict_block(
                &block_key,
                &TEST_JUDGING_UNITS_FINGERPRINT,
                &block_of(&[clean, clean]),
                &[],
            )
            .expect("存旧版本的块");
        assert_eq!(
            store
                .read_verdict_block(&block_key, &OTHER_JUDGING_UNITS_FINGERPRINT)
                .expect("读块"),
            None,
            "另一个判法单元指纹下这一块还没判：旧版本的判定不复用"
        );
        store
            .store_verdict_block(
                &block_key,
                &OTHER_JUDGING_UNITS_FINGERPRINT,
                &block_of(&[clean, red]),
                &[StateOffsetInBlock(1)],
            )
            .expect("存新版本的块：与旧版本判得不同也不算冲突");
        assert_eq!(
            store
                .read_verdict_block(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT)
                .expect("读块"),
            Some(block_of(&[clean, clean])),
            "旧版本的行留着"
        );
        assert_eq!(
            store
                .violations_in_block(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT)
                .expect("读违例"),
            Vec::new(),
            "旧版本没有违例"
        );
        assert_eq!(
            store
                .violations_in_block(&block_key, &OTHER_JUDGING_UNITS_FINGERPRINT)
                .expect("读违例")
                .len(),
            1,
            "新版本一处违例"
        );
        assert_eq!(
            store.stored_verdict_block_keys().expect("列块"),
            vec![
                (block_key.clone(), TEST_JUDGING_UNITS_FINGERPRINT),
                (block_key.clone(), OTHER_JUDGING_UNITS_FINGERPRINT),
            ],
            "同一块两版两行，按版本字节排"
        );
        let composition_of = |version: JudgingUnitsFingerprint, unit: &str| {
            JudgingUnitsKey::of_the_parts(
                version.0,
                vec![JudgingUnitFingerprint {
                    unit: unit.to_string(),
                    fingerprint: [version.0[0]; SHA256_DIGEST_BYTES],
                }],
            )
        };
        store
            .store_judging_units_composition(&composition_of(
                TEST_JUDGING_UNITS_FINGERPRINT,
                "old_judge",
            ))
            .expect("登组成清单");
        store
            .store_judging_units_composition(&composition_of(
                OTHER_JUDGING_UNITS_FINGERPRINT,
                "new_judge",
            ))
            .expect("登组成清单");
        assert_eq!(
            store.judging_units_fingerprints().expect("列指纹"),
            vec![
                TEST_JUDGING_UNITS_FINGERPRINT.0,
                OTHER_JUDGING_UNITS_FINGERPRINT.0
            ]
        );
        assert_eq!(
            store
                .read_judging_units_composition(&OTHER_JUDGING_UNITS_FINGERPRINT.0)
                .expect("读组成清单"),
            Some(
                composition_of(OTHER_JUDGING_UNITS_FINGERPRINT, "new_judge")
                    .composition()
                    .to_vec()
            )
        );
        let merged_directory = TemporaryLibraryDirectory::new("two-judge-versions-merged");
        let mut merged = VerdictStore::create_empty(&merged_directory.path).expect("建库");
        assert_eq!(
            merged.import_from(&store).expect("导入"),
            2,
            "两版各一块都导过去"
        );
        assert_eq!(
            merged.stored_verdict_block_keys().expect("列块").len(),
            2,
            "导入之后两版两行"
        );
        assert_eq!(
            merged.judging_units_fingerprints().expect("列指纹").len(),
            2,
            "组成清单表随导入合并"
        );
    }

    const JUDGEMENT_CONCLUSION_VERSION: ConclusionVersion =
        ConclusionVersion([0x51; SHA256_DIGEST_BYTES]);
    const ANOTHER_JUDGEMENT_CONCLUSION_VERSION: ConclusionVersion =
        ConclusionVersion([0x52; SHA256_DIGEST_BYTES]);
    const OBSERVER_CONCLUSION_VERSION: ConclusionVersion =
        ConclusionVersion([0x53; SHA256_DIGEST_BYTES]);

    fn count_vector_of(states: u64) -> BlockCountVector {
        BlockCountVector {
            per_invariant: BTreeMap::from([
                (
                    "allocation_record_matches_the_tree".to_string(),
                    InvariantStateCounts {
                        judged: states - 1,
                        violated: 1,
                        not_applicable: 1,
                    },
                ),
                (
                    "root_record_names_a_reachable_tree".to_string(),
                    InvariantStateCounts {
                        judged: states,
                        violated: 0,
                        not_applicable: 0,
                    },
                ),
            ]),
            per_observer: BTreeMap::from([("mounted_states".to_string(), states)]),
            states_by_publish: BTreeMap::from([
                ("instance2_txg5".to_string(), states - 1),
                ("every_write_persisted".to_string(), 1),
            ]),
        }
    }

    fn recovery_words_of(states: u32) -> Vec<RecoveryWordsOfState> {
        (0..states)
            .map(|state| RecoveryWordsOfState([2, 5 + state, 1, 0, 2, 4, 1, 100 + state]))
            .collect()
    }

    fn conclusion_record(block: VerdictBlock, violating_states: &[u64]) -> ConclusionBlockRecord {
        let states = u32::try_from(block.verdict_vector_number_of_each_state().len())
            .expect("测试里的块不长");
        ConclusionBlockRecord {
            block,
            violating_states: violating_states
                .iter()
                .copied()
                .map(StateOffsetInBlock)
                .collect(),
            count_vector: count_vector_of(u64::from(states)),
            judged_on: JudgingDevice::Gpu {
                card_type: "probe card".to_string(),
                driver_version: "probe driver 1".to_string(),
                gpu_judge_units_fingerprint: [7; SHA256_DIGEST_BYTES],
            },
            recovery_words: Some(recovery_words_of(states)),
        }
    }

    /// 同一块几份结论各存各的：镜像判定存了一把版本的，另一把版本（另一版判定定义）与观察者照样缺着（只排那两种）；
    /// 种类不同、版本字节相同也互不顶替；两个观察者（种类相同、两把版本）各是各的；都存了就一种都不缺，结论块表四行。
    /// 镜像判定不分 CPU 还是 GPU 判的：同一把版本下只有一份（在哪判的只在块属性里，另一条用例判）。
    #[test]
    fn conclusions_of_one_block_are_reused_and_missing_separately_by_kind_and_version() {
        let directory = TemporaryLibraryDirectory::new("three-kinds");
        let mut store = VerdictStore::create_empty(&directory.path).expect("建库");
        let clean = store
            .register_verdict_vector(&verdict_vector(b""))
            .expect("登记");
        let red = store
            .register_verdict_vector(&verdict_vector(b"red"))
            .expect("登记");
        let block_key = block_key_named("node", 0);
        let observer = ConclusionKind::Observer;
        let required = [
            (ConclusionKind::ImageJudgement, JUDGEMENT_CONCLUSION_VERSION),
            (
                ConclusionKind::ImageJudgement,
                ANOTHER_JUDGEMENT_CONCLUSION_VERSION,
            ),
            (observer.clone(), OBSERVER_CONCLUSION_VERSION),
        ];
        assert_eq!(
            store
                .conclusions_missing_for_block(&block_key, &required)
                .expect("对账"),
            required.to_vec(),
            "空库三种都缺"
        );
        store
            .store_conclusion_block(
                &block_key,
                &ConclusionKind::ImageJudgement,
                &JUDGEMENT_CONCLUSION_VERSION,
                &conclusion_record(block_of(&[clean, red]), &[1]),
            )
            .expect("存一把版本下的镜像判定");
        assert_eq!(
            store
                .conclusions_missing_for_block(&block_key, &required)
                .expect("对账"),
            required[1..].to_vec(),
            "存了这一把版本的，只缺另一把版本与观察者"
        );
        assert_eq!(
            store
                .read_conclusion_block(
                    &block_key,
                    &ConclusionKind::ImageJudgement,
                    &ANOTHER_JUDGEMENT_CONCLUSION_VERSION
                )
                .expect("读"),
            None,
            "另一把版本读不到这一把下的结论"
        );
        assert_eq!(
            store
                .read_conclusion_block(
                    &block_key,
                    &ConclusionKind::Observer,
                    &JUDGEMENT_CONCLUSION_VERSION
                )
                .expect("读"),
            None,
            "观察者种类拿同样的版本字节读不到镜像判定"
        );
        store
            .store_conclusion_block(
                &block_key,
                &ConclusionKind::ImageJudgement,
                &ANOTHER_JUDGEMENT_CONCLUSION_VERSION,
                &conclusion_record(block_of(&[clean, clean]), &[]),
            )
            .expect("存另一把版本下的镜像判定：与前一把下的不同也不算冲突");
        assert_eq!(
            store
                .conclusions_missing_for_block(&block_key, &required)
                .expect("对账"),
            required[2..].to_vec(),
            "只缺观察者"
        );
        let another_observer_version = ConclusionVersion([0x54; SHA256_DIGEST_BYTES]);
        store
            .store_conclusion_block(
                &block_key,
                &ConclusionKind::Observer,
                &another_observer_version,
                &conclusion_record(block_of(&[clean, clean]), &[]),
            )
            .expect("存另一个观察者的结果");
        assert_eq!(
            store
                .conclusions_missing_for_block(&block_key, &required)
                .expect("对账"),
            required[2..].to_vec(),
            "另一个观察者（另一把版本）顶替不了这一个"
        );
        store
            .store_conclusion_block(
                &block_key,
                &observer,
                &OBSERVER_CONCLUSION_VERSION,
                &conclusion_record(block_of(&[red, clean]), &[0]),
            )
            .expect("存观察者结果");
        assert_eq!(
            store
                .conclusions_missing_for_block(&block_key, &required)
                .expect("对账"),
            Vec::new(),
            "三种都有了"
        );
        assert_eq!(
            store
                .read_conclusion_block(
                    &block_key,
                    &ConclusionKind::ImageJudgement,
                    &JUDGEMENT_CONCLUSION_VERSION
                )
                .expect("读")
                .map(|stored| stored.block),
            Some(block_of(&[clean, red])),
            "第一把版本下的结论没被后存的几份顶掉"
        );
        assert_eq!(
            store.stored_conclusion_keys().expect("列").len(),
            4,
            "结论块表四行：两把版本的镜像判定、两个观察者"
        );
    }

    /// 一块一种结论连同计数向量、块属性与恢复字写下去再读回逐项相同；关库重开照样相同。恢复字另一张表存，不进判定向量编号：
    /// 存完判定向量还是登记的那两种。
    #[test]
    fn a_conclusion_block_reads_back_with_its_count_vector_attributes_and_recovery_words() {
        let directory = TemporaryLibraryDirectory::new("conclusion-read-back");
        let block_key = block_key_named("node", 65536);
        let (record, red) = {
            let mut store = VerdictStore::create_empty(&directory.path).expect("建库");
            let clean = store
                .register_verdict_vector(&verdict_vector(b""))
                .expect("登记");
            let red = store
                .register_verdict_vector(&verdict_vector(b"red"))
                .expect("登记");
            let record = conclusion_record(block_of(&[clean, red, clean]), &[1]);
            assert_eq!(
                store
                    .store_conclusion_block(
                        &block_key,
                        &ConclusionKind::ImageJudgement,
                        &ANOTHER_JUDGEMENT_CONCLUSION_VERSION,
                        &record
                    )
                    .expect("存"),
                VerdictBlockStoring::WrittenAndSynced
            );
            assert_eq!(
                store.verdict_vector_by_number.len(),
                2,
                "恢复字不进判定向量编号"
            );
            let recovery_words_row = store
                .database
                .get(
                    VerdictStoreKey::RecoveryWords(
                        &block_key,
                        &ConclusionKind::ImageJudgement,
                        &ANOTHER_JUDGEMENT_CONCLUSION_VERSION,
                    )
                    .to_key_bytes()
                    .0,
                )
                .expect("读原始值")
                .expect("恢复字在旁表里");
            assert_eq!(
                recovery_words_row.len(),
                3 * RECOVERY_WORDS_PER_STATE * 4,
                "旁表里每个状态八个字、每字四个字节"
            );
            (record, red)
        };
        let (store, opening) =
            VerdictStore::open_existing_with_conclusion_kinds(&directory.path).expect("重开");
        assert_eq!(opening, VerdictStoreOpening::OpenedAtTheFirstAttempt);
        assert_eq!(store.layout(), VerdictStoreLayout::ConclusionKinds);
        let stored = store
            .read_conclusion_block(
                &block_key,
                &ConclusionKind::ImageJudgement,
                &ANOTHER_JUDGEMENT_CONCLUSION_VERSION,
            )
            .expect("读")
            .expect("存过");
        assert_eq!(stored.block, record.block);
        assert_eq!(
            stored.violations,
            vec![StoredViolation {
                offset_in_block: StateOffsetInBlock(1),
                verdict_vector_number: red,
            }]
        );
        assert_eq!(stored.count_vector, record.count_vector, "计数向量随块读回");
        assert_eq!(stored.judged_on, record.judged_on, "块属性随块读回");
        assert_eq!(
            stored.recovery_words, record.recovery_words,
            "恢复字随块读回"
        );
        assert_eq!(
            stored.circumstances,
            JudgementCircumstances {
                judged_date: tokyo_date_text_of(std::time::SystemTime::now()),
                run_kind: RunKindOfTheJudgement::of_this_process(),
            },
            "块属性带判定日期（东京日期）与跑法"
        );
        assert!(
            stored.circumstances.judged_date.len() == "YYYY-MM-DD".len()
                && stored.circumstances.judged_date.as_bytes()[4] == b'-'
                && stored.circumstances.judged_date.as_bytes()[7] == b'-',
            "判定日期写成 YYYY-MM-DD：{}",
            stored.circumstances.judged_date
        );
    }

    /// 测试搭建：照旧布局的写法造一个库——布局行写成给的布局号，按一个字节一个编号登记一种判定向量（两张表各一行）、存一块两个状态的判定块。
    /// 绕过本模块的写法直接写 RocksDB，造的就是编号加宽之前那几套布局真会留下的字节。
    fn library_written_in_a_one_byte_number_layout(directory: &Path, layout_number: u32) {
        let store = VerdictStore::create_empty(directory).expect("建库");
        let clean = verdict_vector(b"clean");
        let block_key = block_key_named("stream", 0);
        let key_of_number_zero_in_one_byte =
            [VerdictStoreTable::VerdictVectorContentByNumber.key_tag(), 0];
        let mut batch = rocksdb::WriteBatch::default();
        batch.put(
            VerdictStoreKey::LibraryLayout.to_key_bytes().0,
            layout_number.to_be_bytes(),
        );
        batch.put(
            VerdictStoreKey::VerdictVectorNumberByContent(&clean)
                .to_key_bytes()
                .0,
            [0],
        );
        batch.put(key_of_number_zero_in_one_byte, &clean.0);
        batch.put(
            VerdictStoreKey::VerdictBlock(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT)
                .to_key_bytes()
                .0,
            [0, 0],
        );
        store
            .database
            .write_opt(batch, &synced_write_options())
            .expect("照旧布局写下布局行、判定向量表与一块");
    }

    /// 块键里还带节点名的那一套（布局号 2）：编号还是一个字节，三种开法都在读判定向量表之前就拒、不迁移。
    #[test]
    fn a_library_of_the_layout_with_node_names_in_the_block_key_is_refused() {
        let directory = TemporaryLibraryDirectory::new("layout-with-node-names");
        library_written_in_a_one_byte_number_layout(
            &directory.path,
            LAYOUT_NUMBER_WITH_NODE_NAMES_IN_THE_BLOCK_KEY,
        );
        let opened = VerdictStore::open_existing(&directory.path);
        assert!(
            matches!(
                opened,
                Err(
                    VerdictStoreError::LibraryLayoutWithOneByteVerdictVectorNumbers {
                        layout_number: 2,
                        ..
                    }
                )
            ),
            "照旧打开也拒：{:?}",
            opened.err()
        );
        assert!(matches!(
            VerdictStore::open_existing_with_conclusion_kinds(&directory.path),
            Err(
                VerdictStoreError::LibraryLayoutWithOneByteVerdictVectorNumbers {
                    layout_number: 2,
                    ..
                }
            )
        ));
        let log_directory = TemporaryLibraryDirectory::new("layout-with-node-names-log");
        std::fs::create_dir_all(&log_directory.path).expect("建日志目录");
        assert!(matches!(
            VerdictStore::open_existing_for_reading_only(&directory.path, &log_directory.path),
            Err(
                VerdictStoreError::LibraryLayoutWithOneByteVerdictVectorNumbers {
                    layout_number: 2,
                    ..
                }
            )
        ));
    }

    /// 判定向量编号还是一个字节的那一套（布局号 4，加宽之前的这一版）：三种开法都在读判定向量表之前就拒、不迁移，不把一个字节的编号
    /// 当 4 字节读。
    /// 判别力（`crates/mutations.tsv`）：布局号 4 当成这一版照开（编号表按 4 字节读成「编号不连续」不合布局，不报旧布局），这条红。
    #[test]
    fn a_library_of_the_layout_with_one_byte_verdict_vector_numbers_is_refused_by_every_opening() {
        let directory = TemporaryLibraryDirectory::new("layout-with-one-byte-numbers");
        library_written_in_a_one_byte_number_layout(
            &directory.path,
            LAYOUT_NUMBER_WITH_ONE_BYTE_VERDICT_VECTOR_NUMBERS,
        );
        let opened = VerdictStore::open_existing(&directory.path);
        assert!(
            matches!(
                opened,
                Err(
                    VerdictStoreError::LibraryLayoutWithOneByteVerdictVectorNumbers {
                        layout_number: 4,
                        ..
                    }
                )
            ),
            "照旧打开要拒：{:?}",
            opened.err()
        );
        let opened_with_conclusion_kinds =
            VerdictStore::open_existing_with_conclusion_kinds(&directory.path);
        assert!(
            matches!(
                opened_with_conclusion_kinds,
                Err(
                    VerdictStoreError::LibraryLayoutWithOneByteVerdictVectorNumbers {
                        layout_number: 4,
                        ..
                    }
                )
            ),
            "按结论种类打开要拒：{:?}",
            opened_with_conclusion_kinds.err()
        );
        let log_directory = TemporaryLibraryDirectory::new("layout-with-one-byte-numbers-log");
        std::fs::create_dir_all(&log_directory.path).expect("建日志目录");
        let opened_for_reading_only =
            VerdictStore::open_existing_for_reading_only(&directory.path, &log_directory.path);
        assert!(
            matches!(
                opened_for_reading_only,
                Err(
                    VerdictStoreError::LibraryLayoutWithOneByteVerdictVectorNumbers {
                        layout_number: 4,
                        ..
                    }
                )
            ),
            "只读地开要拒：{:?}",
            opened_for_reading_only.err()
        );
    }

    /// 4-2a 报告第四节第 7 条后半句：登记成照判不入库的版本，键里带着它的行（结论块连同违例、计数向量、块属性）只留在这个进程里——
    /// 这一份库照读得回、列得出，关库重开就没了；同一块别的版本的结论照常写进库、重开读得回。登记之前写进库的行不动。
    /// 判别力（`crates/mutations.tsv`）：照判不入库的行照旧写进 RocksDB（重开还读得回），这条红。
    #[test]
    fn rows_of_a_version_kept_out_of_the_library_are_read_in_this_process_and_never_written() {
        let directory = TemporaryLibraryDirectory::new("kept-out-versions");
        let block_key = block_key_named("kept-out", 0);
        let kept_out_version = ConclusionVersion([0x61; SHA256_DIGEST_BYTES]);
        {
            let mut store = VerdictStore::create_empty(&directory.path).expect("建库");
            let clean = store
                .register_verdict_vector(&verdict_vector(b""))
                .expect("登记");
            let red = store
                .register_verdict_vector(&verdict_vector(b"red"))
                .expect("登记");
            store.keep_the_rows_of_these_versions_out_of_the_library([kept_out_version.0]);
            assert_eq!(
                store.versions_kept_out_of_the_library(),
                vec![kept_out_version.0]
            );
            let kept_out_record = conclusion_record(block_of(&[clean, red]), &[1]);
            store
                .store_conclusion_block(
                    &block_key,
                    &ConclusionKind::ImageJudgement,
                    &kept_out_version,
                    &kept_out_record,
                )
                .expect("照判不入库的那一把照样存得进这个进程");
            store
                .store_conclusion_block(
                    &block_key,
                    &ConclusionKind::ImageJudgement,
                    &ANOTHER_JUDGEMENT_CONCLUSION_VERSION,
                    &conclusion_record(block_of(&[clean, clean]), &[]),
                )
                .expect("别的版本照常存进库");
            assert_eq!(
                store
                    .read_conclusion_block(
                        &block_key,
                        &ConclusionKind::ImageJudgement,
                        &kept_out_version
                    )
                    .expect("读")
                    .map(|stored| (stored.block, stored.violations.len())),
                Some((block_of(&[clean, red]), 1)),
                "这个进程里读得回照判不入库的那一块，连同违例"
            );
            assert_eq!(
                store.stored_conclusion_keys().expect("列").len(),
                2,
                "这个进程里列得出两把版本的结论"
            );
        }
        let (reopened, _opening) =
            VerdictStore::open_existing_with_conclusion_kinds(&directory.path).expect("重开");
        assert_eq!(
            reopened
                .read_conclusion_block(
                    &block_key,
                    &ConclusionKind::ImageJudgement,
                    &kept_out_version
                )
                .expect("读"),
            None,
            "照判不入库的那一块没进 RocksDB：重开就没了"
        );
        assert!(
            reopened
                .read_conclusion_block(
                    &block_key,
                    &ConclusionKind::ImageJudgement,
                    &ANOTHER_JUDGEMENT_CONCLUSION_VERSION
                )
                .expect("读")
                .is_some(),
            "别的版本的结论在库里"
        );
        assert_eq!(
            reopened.stored_conclusion_keys().expect("列"),
            vec![(
                block_key,
                ConclusionKind::ImageJudgement,
                ANOTHER_JUDGEMENT_CONCLUSION_VERSION
            )],
            "库里只有入库的那一把"
        );
    }

    /// crates 第三轮第 1 条：照判不入库只比键里版本那一截（按表标签定的偏移）。版本那一截是那一把的行，十六种键各自不入库；那一把只出现在
    /// 键里别处（块键里的两个摘要、判定向量的字节、录下的流程的摘要、录入键、对拍的输入摘要、镜像去重键、分片进度的段摘要）的行照入库——
    /// 改前在整条键上按 32 字节滑着找，后一半全判成不入库。
    /// 判别力（`crates/mutations.tsv`）：结论那几张表的版本起点错一个字节，「结论块」那一行判成入库，这条红。
    #[test]
    fn a_kept_out_version_is_matched_only_at_the_version_field_of_each_table() {
        let kept_out_version: [u8; SHA256_DIGEST_BYTES] = std::array::from_fn(|position| {
            0xA0 + u8::try_from(position).expect("32 个位置装得进 u8")
        });
        let kept_out = RowsKeptOutOfTheLibrary {
            versions: BTreeSet::from([kept_out_version]),
            rows: BTreeMap::new(),
        };
        let block_key = block_key_named("kept-out-offsets", 7);
        let fingerprint = JudgingUnitsFingerprint(kept_out_version);
        let version = ConclusionVersion(kept_out_version);
        let other_digest = [0x5C; SHA256_DIGEST_BYTES];
        let pairing_with_it_on_one_side = PairingConclusionKey {
            inputs_digest: other_digest,
            one_side_judging_units: kept_out_version,
            other_side_judging_units: other_digest,
        };
        let pairing_with_it_on_the_other_side = PairingConclusionKey {
            inputs_digest: other_digest,
            one_side_judging_units: other_digest,
            other_side_judging_units: kept_out_version,
        };
        let progress_under_the_fingerprint = SlicedSegmentProgressKey {
            segment_content_digest: other_digest,
            judge: JudgeOfTheProgress::TheJudgingUnitsFingerprint(fingerprint),
        };
        let progress_under_the_conclusion_kind = SlicedSegmentProgressKey {
            segment_content_digest: other_digest,
            judge: JudgeOfTheProgress::TheConclusionKind {
                kind: ConclusionKind::ImageJudgement,
                version,
            },
        };
        let carrying_it_in_the_version_field = [
            (
                "判定块",
                VerdictStoreKey::VerdictBlock(&block_key, &fingerprint),
            ),
            (
                "违例",
                VerdictStoreKey::Violation(&block_key, &fingerprint, StateOffsetInBlock(3)),
            ),
            (
                "核对表",
                VerdictStoreKey::VerifierBlock(&block_key, &kept_out_version),
            ),
            (
                "组成清单",
                VerdictStoreKey::JudgingUnitsComposition(&kept_out_version),
            ),
            (
                "对拍的一边",
                VerdictStoreKey::PairingConclusion(&pairing_with_it_on_one_side),
            ),
            (
                "对拍的另一边",
                VerdictStoreKey::PairingConclusion(&pairing_with_it_on_the_other_side),
            ),
            (
                "结论块",
                VerdictStoreKey::ConclusionBlock(
                    &block_key,
                    &ConclusionKind::ImageJudgement,
                    &version,
                ),
            ),
            (
                "结论违例",
                VerdictStoreKey::ConclusionViolation(
                    &block_key,
                    &ConclusionKind::ImageJudgement,
                    &version,
                    StateOffsetInBlock(3),
                ),
            ),
            (
                "恢复字",
                VerdictStoreKey::RecoveryWords(
                    &block_key,
                    &ConclusionKind::ImageJudgement,
                    &version,
                ),
            ),
            (
                "点名的被抛弃根",
                VerdictStoreKey::AbandonedRootsNamedByThePoolChecker(
                    &block_key,
                    &ConclusionKind::ImageJudgement,
                    &version,
                ),
            ),
            (
                "计数向量",
                VerdictStoreKey::ConclusionCountVector(
                    &block_key,
                    &ConclusionKind::ImageJudgement,
                    &version,
                ),
            ),
            (
                "块属性",
                VerdictStoreKey::ConclusionBlockAttributes(
                    &block_key,
                    &ConclusionKind::ImageJudgement,
                    &version,
                ),
            ),
            (
                "观察者录入块",
                VerdictStoreKey::ObserverBlock(&block_key, &ConclusionKind::Observer, &version),
            ),
            (
                "镜像上的池级 checker",
                VerdictStoreKey::ObservedImageJudgement(
                    &other_digest,
                    &ConclusionKind::ImageJudgement,
                    &version,
                ),
            ),
            (
                "分片进度（判法单元指纹那一套）",
                VerdictStoreKey::SlicedSegmentProgress(&progress_under_the_fingerprint),
            ),
            (
                "分片进度（结论种类那一套）",
                VerdictStoreKey::SlicedSegmentProgress(&progress_under_the_conclusion_kind),
            ),
        ];
        for (table, key) in &carrying_it_in_the_version_field {
            assert!(
                kept_out.keeps_out(&key.to_key_bytes().0),
                "{table}：版本那一截是照判不入库的那一把，这一行不入库"
            );
        }
        let block_key_made_of_it = VerdictBlockKey {
            node_input_digest: NodeInputDigest(kept_out_version),
            enumeration_plan_hash: EnumerationPlanHash(kept_out_version),
            block_start: BlockStartStateIndex(7),
        };
        let verdict_vector_holding_it = EncodedVerdictVector(kept_out_version.to_vec());
        let recording_key_equal_to_it = RecordingKey(kept_out_version);
        let pairing_of_inputs_equal_to_it = PairingConclusionKey {
            inputs_digest: kept_out_version,
            one_side_judging_units: other_digest,
            other_side_judging_units: other_digest,
        };
        let progress_of_a_segment_digest_equal_to_it = SlicedSegmentProgressKey {
            segment_content_digest: kept_out_version,
            judge: JudgeOfTheProgress::TheJudgingUnitsFingerprint(TEST_JUDGING_UNITS_FINGERPRINT),
        };
        let carrying_it_elsewhere = [
            (
                "判定块的块键",
                VerdictStoreKey::VerdictBlock(
                    &block_key_made_of_it,
                    &TEST_JUDGING_UNITS_FINGERPRINT,
                ),
            ),
            (
                "录入表的块键",
                VerdictStoreKey::RecordedBlock(&block_key_made_of_it),
            ),
            (
                "结论块的块键",
                VerdictStoreKey::ConclusionBlock(
                    &block_key_made_of_it,
                    &ConclusionKind::ImageJudgement,
                    &ANOTHER_JUDGEMENT_CONCLUSION_VERSION,
                ),
            ),
            (
                "判定向量的字节",
                VerdictStoreKey::VerdictVectorNumberByContent(&verdict_vector_holding_it),
            ),
            (
                "录下的流程的摘要",
                VerdictStoreKey::RecordedFlow(&kept_out_version),
            ),
            (
                "录入键",
                VerdictStoreKey::RecordedRunManifest(&recording_key_equal_to_it),
            ),
            (
                "对拍的输入摘要",
                VerdictStoreKey::PairingConclusion(&pairing_of_inputs_equal_to_it),
            ),
            (
                "镜像去重键",
                VerdictStoreKey::ObservedImageJudgement(
                    &kept_out_version,
                    &ConclusionKind::ImageJudgement,
                    &ANOTHER_JUDGEMENT_CONCLUSION_VERSION,
                ),
            ),
            (
                "分片进度的段摘要",
                VerdictStoreKey::SlicedSegmentProgress(&progress_of_a_segment_digest_equal_to_it),
            ),
        ];
        for (table, key) in &carrying_it_elsewhere {
            assert!(
                !kept_out.keeps_out(&key.to_key_bytes().0),
                "{table}：那一把只出现在键里别处、不在版本那一截，这一行照入库"
            );
        }
    }

    /// 表标签读回它那张表（[`VerdictStoreTable::of_key_tag`] 是 [`VerdictStoreTable::key_tag`] 的逆，认得的标签正好 28 个；退役的 `1B` 不认），
    /// 结论种类每一种在键里都是 [`CONCLUSION_KIND_KEY_BYTES`] 个字节：照判不入库按这两样定版本那一截的起点。
    #[test]
    fn every_table_tag_reads_back_to_its_table_and_every_conclusion_kind_takes_its_key_bytes() {
        let mut tables_read_back = 0;
        for key_tag in 0..=u8::MAX {
            if let Some(table) = VerdictStoreTable::of_key_tag(key_tag) {
                assert_eq!(table.key_tag(), key_tag, "{table:?} 的标签读回它自己");
                tables_read_back += 1;
            }
        }
        assert_eq!(tables_read_back, 28, "28 张表各一个标签");
        assert_eq!(
            VerdictStoreTable::of_key_tag(0x1B),
            None,
            "改前整份见证清单那张表的标签退役、不再分配"
        );
        for kind in [
            ConclusionKind::ImageJudgement,
            ConclusionKind::ImageJudgement,
            ConclusionKind::Observer,
        ] {
            assert_eq!(
                conclusion_kind_key_bytes(&kind).len(),
                CONCLUSION_KIND_KEY_BYTES,
                "{kind:?} 在键里的字节数"
            );
        }
    }

    /// 观察者见证按行存：一个桶里添三条见证（两条共用一组读集，读集行只随头一条写）、另一个桶添一条，每添一条恰好一批；按桶键读回的
    /// 是这个桶自己的那几行、一条见证一行、按见证摘要排，引的读集行与镜像都读得到；重开库照样读得回。
    #[test]
    fn observer_read_set_witness_rows_read_back_one_row_per_witness_from_their_own_bucket() {
        let directory = TemporaryLibraryDirectory::new("observer-witness-rows");
        let bucket = [0xB1; SHA256_DIGEST_BYTES];
        let other_bucket = [0xB2; SHA256_DIGEST_BYTES];
        let shared_read_set_bytes = b"read set shared by two witnesses".to_vec();
        let other_read_set_bytes = b"read set of one witness".to_vec();
        let witness_numbered = |number: u8, read_set_digest: u8| ObserverReadSetWitnessRow {
            witness_digest: [number; SHA256_DIGEST_BYTES],
            read_set_digest: [read_set_digest; SHA256_DIGEST_BYTES],
            observed_image_key: [number.wrapping_add(0x80); SHA256_DIGEST_BYTES],
        };
        let witnesses_of_the_bucket = [
            witness_numbered(0x03, 0xD1),
            witness_numbered(0x01, 0xD1),
            witness_numbered(0x02, 0xD2),
        ];
        let witness_of_the_other_bucket = witness_numbered(0x04, 0xD1);
        {
            let mut store = VerdictStore::create_empty(&directory.path).expect("建库");
            let batches_before = store.synced_write_tally().batches_written;
            store
                .store_observed_image_with_its_read_set_witness(
                    &bucket,
                    &witnesses_of_the_bucket[0],
                    Some(&shared_read_set_bytes),
                    b"image of witness three",
                )
                .expect("写见证");
            store
                .store_observed_image_with_its_read_set_witness(
                    &bucket,
                    &witnesses_of_the_bucket[1],
                    None,
                    b"image of witness one",
                )
                .expect("写见证（读集行已在）");
            store
                .store_observed_image_with_its_read_set_witness(
                    &bucket,
                    &witnesses_of_the_bucket[2],
                    Some(&other_read_set_bytes),
                    b"image of witness two",
                )
                .expect("写见证");
            store
                .store_observed_image_with_its_read_set_witness(
                    &other_bucket,
                    &witness_of_the_other_bucket,
                    Some(&shared_read_set_bytes),
                    b"image of witness four",
                )
                .expect("另一个桶写见证");
            assert_eq!(
                store.synced_write_tally().batches_written - batches_before,
                4,
                "每添一条见证恰好一批（见证行、读集行、镜像同一批）"
            );
        }
        let (store, _opening) = VerdictStore::open_existing(&directory.path).expect("重开库");
        let read_back = store
            .read_observer_read_set_witnesses_of_the_bucket(&bucket)
            .expect("读桶的见证");
        assert_eq!(
            read_back,
            vec![
                witnesses_of_the_bucket[1],
                witnesses_of_the_bucket[2],
                witnesses_of_the_bucket[0]
            ],
            "这个桶的三条见证各一行、按见证摘要排，另一个桶的那一条不在里面"
        );
        assert_eq!(
            store
                .read_observer_read_set_witnesses_of_the_bucket(&other_bucket)
                .expect("读另一个桶"),
            vec![witness_of_the_other_bucket]
        );
        assert_eq!(
            store
                .read_observer_read_set_witnesses_of_the_bucket(&[0xB3; SHA256_DIGEST_BYTES])
                .expect("读没有见证的桶"),
            Vec::new()
        );
        assert_eq!(
            store
                .read_observer_read_set(&[0xD1; SHA256_DIGEST_BYTES])
                .expect("读读集行"),
            Some(shared_read_set_bytes)
        );
        assert_eq!(
            store
                .read_observed_image(&witnesses_of_the_bucket[1].observed_image_key)
                .expect("读镜像"),
            Some(b"image of witness one".to_vec())
        );
    }

    /// 一路假的库行：键是 `2 × 序号` 的四字节大端，值是 `library`；每交出一行记一次数（不整段缓冲的判据就是被取走的行数）。
    struct LibraryRowsCountingTheRowsTaken<'count> {
        ordinals: std::ops::Range<u32>,
        rows_taken: &'count std::cell::Cell<u64>,
    }

    impl Iterator for LibraryRowsCountingTheRowsTaken<'_> {
        type Item = Result<StoredEntry, rocksdb::Error>;

        fn next(&mut self) -> Option<Self::Item> {
            let ordinal = self.ordinals.next()?;
            self.rows_taken.set(self.rows_taken.get() + 1);
            Some(Ok((
                (2 * ordinal).to_be_bytes().to_vec().into_boxed_slice(),
                b"library".to_vec().into_boxed_slice(),
            )))
        }
    }

    /// rocksdb 的错误类型只能由它自己造：往一个普通文件上开库，拿它报的错当测试里的「库读错」。
    fn a_real_library_read_error() -> rocksdb::Error {
        let not_a_directory = std::env::temp_dir().join(format!(
            "singlefs-verdict-store-not-a-library-directory-{}-{}",
            std::process::id(),
            NEXT_TEMPORARY_DIRECTORY_SUFFIX.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&not_a_directory, b"not a library directory")
            .expect("临时目录可写：建一个普通文件");
        let read_error =
            rocksdb::DB::open_default(&not_a_directory).expect_err("往一个普通文件上开库必失败");
        std::fs::remove_file(&not_a_directory).expect("删掉自己建的普通文件");
        read_error
    }

    fn kept_out_row(key_number: u32) -> (Vec<u8>, Vec<u8>) {
        (key_number.to_be_bytes().to_vec(), b"kept out".to_vec())
    }

    /// crates 第三轮第 2 条：前缀下的行两路有序流逐行归并，库那一路十万行里只取到交出去的那几行再多看一行，不整段读进来；交出的次序与值：
    /// 两路交替按键升序，同一个键（4）交照判不入库那一行。
    /// 判别力（`crates/mutations.tsv`）：库那一路先整段收进内存再归并，被取走的行数是十万，这条红。
    #[test]
    fn merging_the_library_rows_with_the_kept_out_rows_takes_one_row_at_a_time() {
        let rows_taken = std::cell::Cell::new(0);
        let library_rows = LibraryRowsCountingTheRowsTaken {
            ordinals: 0..100_000,
            rows_taken: &rows_taken,
        };
        let kept_out_rows = [1, 3, 4, 5].into_iter().map(kept_out_row);
        let first_five: Vec<(Vec<u8>, Vec<u8>)> =
            RowsOfTwoSourcesMergedByKey::new(library_rows, kept_out_rows)
                .take(5)
                .map(|row| {
                    let (key_bytes, value_bytes) = row.expect("假的库行不报错");
                    (key_bytes.into_vec(), value_bytes.into_vec())
                })
                .collect();
        assert_eq!(
            first_five,
            vec![
                (0u32.to_be_bytes().to_vec(), b"library".to_vec()),
                kept_out_row(1),
                (2u32.to_be_bytes().to_vec(), b"library".to_vec()),
                kept_out_row(3),
                kept_out_row(4),
            ],
            "两路按键升序交替，同一个键交照判不入库那一行"
        );
        assert!(
            rows_taken.get() <= 4,
            "交出五行，库那一路只取到交出去的几行与往前看的一行（十万行不整段读进来）：取了 {}",
            rows_taken.get()
        );
    }

    /// 改前的合并（库里那一段整段读进一张有序表，再用照判不入库那一段逐行盖）：只留着给对照，交出的次序与值是归并要逐条对得上的。
    fn rows_merged_as_before(
        library_rows: &[(Vec<u8>, Vec<u8>)],
        kept_out_rows: &[(Vec<u8>, Vec<u8>)],
    ) -> Vec<(Vec<u8>, Vec<u8>)> {
        let mut merged: BTreeMap<Vec<u8>, Vec<u8>> = library_rows.iter().cloned().collect();
        merged.extend(kept_out_rows.iter().cloned());
        merged.into_iter().collect()
    }

    /// crates 第三轮第 2 条：归并交出的行与改前的合并逐条相同——两路交错、同一个键两路都有（交照判不入库那一行）、一路空着、两路都空。
    /// 判别力（`crates/mutations.tsv`）：同一个键两路都有时交库里那一行，「c」「f」两行的值对不上，这条红。
    #[test]
    fn merging_gives_the_rows_and_the_order_of_the_former_whole_prefix_merge() {
        let row = |key: &str, value: &str| (key.as_bytes().to_vec(), value.as_bytes().to_vec());
        let library_rows = vec![
            row("a", "library a"),
            row("c", "library c"),
            row("d", "library d"),
            row("f", "library f"),
        ];
        let kept_out_rows = vec![
            row("b", "kept out b"),
            row("c", "kept out c"),
            row("e", "kept out e"),
            row("f", "kept out f"),
            row("g", "kept out g"),
        ];
        for (case, library_side, kept_out_side) in [
            (
                "两路交错、同键两处",
                library_rows.clone(),
                kept_out_rows.clone(),
            ),
            ("照判不入库那一路空着", library_rows.clone(), Vec::new()),
            ("库那一路空着", Vec::new(), kept_out_rows.clone()),
            ("两路都空", Vec::new(), Vec::new()),
        ] {
            let merged: Vec<(Vec<u8>, Vec<u8>)> = RowsOfTwoSourcesMergedByKey::new(
                library_side
                    .clone()
                    .into_iter()
                    .map(|(key_bytes, value_bytes)| {
                        Ok::<StoredEntry, rocksdb::Error>((
                            key_bytes.into_boxed_slice(),
                            value_bytes.into_boxed_slice(),
                        ))
                    }),
                kept_out_side.clone().into_iter(),
            )
            .map(|merged_row| {
                let (key_bytes, value_bytes) = merged_row.expect("没有读错");
                (key_bytes.into_vec(), value_bytes.into_vec())
            })
            .collect();
            assert_eq!(
                merged,
                rows_merged_as_before(&library_side, &kept_out_side),
                "{case}：与改前的合并逐条相同"
            );
        }
    }

    /// 库那一路读错：错误照它出现的位置交出，交出之后归并到此为止（照判不入库那一路还剩的行不再交，调用方见错即停）。
    #[test]
    fn a_library_read_error_is_handed_over_where_it_occurs_and_ends_the_merge() {
        let read_error = a_real_library_read_error();
        let library_rows = vec![
            Ok((
                b"a".to_vec().into_boxed_slice(),
                b"library a".to_vec().into_boxed_slice(),
            )),
            Err(read_error.clone()),
            Ok((
                b"z".to_vec().into_boxed_slice(),
                b"library z".to_vec().into_boxed_slice(),
            )),
        ];
        let kept_out_rows = vec![(b"b".to_vec(), b"kept out b".to_vec())];
        let merged: Vec<Result<Vec<u8>, String>> =
            RowsOfTwoSourcesMergedByKey::new(library_rows.into_iter(), kept_out_rows.into_iter())
                .map(|merged_row| {
                    merged_row
                        .map(|(key_bytes, _value_bytes)| key_bytes.into_vec())
                        .map_err(rocksdb::Error::into_string)
                })
                .collect();
        assert_eq!(
            merged,
            vec![Ok(b"a".to_vec()), Err(read_error.into_string())]
        );
    }

    /// 照判不入库那一路一次取一行、两次取之间不拿着锁：取了一行之后再往这个进程里写，键落在已取过的键之前的不再取，之后的照样取到，
    /// 前缀之外的不取。
    #[test]
    fn the_kept_out_rows_are_taken_one_at_a_time_without_holding_the_lock_between_takes() {
        let rows = std::sync::Mutex::new(RowsKeptOutOfTheLibrary {
            versions: BTreeSet::new(),
            rows: [
                (b"p1".to_vec(), b"one".to_vec()),
                (b"p2".to_vec(), b"two".to_vec()),
                (b"q1".to_vec(), b"outside".to_vec()),
            ]
            .into_iter()
            .collect(),
        });
        let mut kept_out_rows = KeptOutRowsWithThePrefix {
            rows_kept_out_of_the_library: &rows,
            prefix: b"p".to_vec(),
            last_key_taken: None,
        };
        assert_eq!(
            kept_out_rows.next(),
            Some((b"p1".to_vec(), b"one".to_vec()))
        );
        {
            let mut written_between_the_takes = rows
                .try_lock()
                .expect("两次取之间没人拿着锁（拿着的话这里取不到锁）");
            written_between_the_takes
                .rows
                .insert(b"p0".to_vec(), b"before".to_vec());
            written_between_the_takes
                .rows
                .insert(b"p3".to_vec(), b"after".to_vec());
        }
        assert_eq!(
            kept_out_rows.collect::<Vec<_>>(),
            vec![
                (b"p2".to_vec(), b"two".to_vec()),
                (b"p3".to_vec(), b"after".to_vec()),
            ]
        );
    }

    /// 4-2a 报告第四节第 5 条：观察者那一种结论的键里不带名字（种类一个字节 `03`，是哪一个观察者由后面的版本担）；键变了，观察者的种类里
    /// 还带名字的那一套（布局号 3）编号也还是一个字节：三种开法都在读判定向量表之前就拒、不迁移。
    /// 判别力（`crates/mutations.tsv`）：布局号 3 不当旧布局（落到「布局号认不得」），这条红。
    #[test]
    fn a_library_of_the_layout_with_observer_names_in_the_conclusion_kind_is_refused() {
        assert_eq!(
            conclusion_kind_key_bytes(&ConclusionKind::Observer),
            vec![0x03],
            "观察者的种类一个字节，不带名字"
        );
        let directory = TemporaryLibraryDirectory::new("layout-with-observer-names");
        library_written_in_a_one_byte_number_layout(
            &directory.path,
            LAYOUT_NUMBER_WITH_OBSERVER_NAMES_IN_THE_CONCLUSION_KIND,
        );
        let opened = VerdictStore::open_existing(&directory.path);
        assert!(
            matches!(
                opened,
                Err(
                    VerdictStoreError::LibraryLayoutWithOneByteVerdictVectorNumbers {
                        layout_number: 3,
                        ..
                    }
                )
            ),
            "照旧打开也拒：{:?}",
            opened.err()
        );
        assert!(matches!(
            VerdictStore::open_existing_with_conclusion_kinds(&directory.path),
            Err(
                VerdictStoreError::LibraryLayoutWithOneByteVerdictVectorNumbers {
                    layout_number: 3,
                    ..
                }
            )
        ));
        let log_directory = TemporaryLibraryDirectory::new("layout-with-observer-names-log");
        std::fs::create_dir_all(&log_directory.path).expect("建日志目录");
        assert!(matches!(
            VerdictStore::open_existing_for_reading_only(&directory.path, &log_directory.path),
            Err(
                VerdictStoreError::LibraryLayoutWithOneByteVerdictVectorNumbers {
                    layout_number: 3,
                    ..
                }
            )
        ));
    }

    /// 只读地开库：读得到存着的结论，库目录里每份文件的修改时刻与字节数一个不动、不多不少；同一个库同时还能被别的进程照常开（不拿锁）。
    #[test]
    fn opening_a_library_for_reading_only_reads_its_conclusions_and_touches_no_file_of_it() {
        let directory = TemporaryLibraryDirectory::new("read-only");
        let block_key = block_key_named("read-only", 0);
        {
            let mut store = VerdictStore::create_empty(&directory.path).expect("建库");
            let clean = store
                .register_verdict_vector(&verdict_vector(b""))
                .expect("登记");
            store
                .store_conclusion_block(
                    &block_key,
                    &ConclusionKind::ImageJudgement,
                    &ANOTHER_JUDGEMENT_CONCLUSION_VERSION,
                    &conclusion_record(block_of(&[clean]), &[]),
                )
                .expect("存");
        }
        let files_with_their_times = || {
            let mut files: Vec<(std::ffi::OsString, std::time::SystemTime, u64)> =
                std::fs::read_dir(&directory.path)
                    .expect("列库目录")
                    .map(|entry| {
                        let entry = entry.expect("读目录项");
                        let metadata = entry.metadata().expect("读文件属性");
                        (
                            entry.file_name(),
                            metadata.modified().expect("读修改时刻"),
                            metadata.len(),
                        )
                    })
                    .collect();
            files.sort();
            files
        };
        let before = files_with_their_times();
        let log_directory = TemporaryLibraryDirectory::new("read-only-log");
        std::fs::create_dir_all(&log_directory.path).expect("建日志目录");
        let store =
            VerdictStore::open_existing_for_reading_only(&directory.path, &log_directory.path)
                .expect("只读开得了");
        assert!(
            store
                .read_conclusion_block(
                    &block_key,
                    &ConclusionKind::ImageJudgement,
                    &ANOTHER_JUDGEMENT_CONCLUSION_VERSION
                )
                .expect("读")
                .is_some(),
            "只读开的库读得到存着的结论"
        );
        drop(store);
        assert_eq!(
            before,
            files_with_their_times(),
            "只读开过之后库目录里的文件一个不动"
        );
    }

    /// 对拍结论按（输入摘要，两边判法单元指纹）存、读回逐字节相同；换了哪一边的指纹读不到。
    #[test]
    fn a_pairing_conclusion_reads_back_under_its_inputs_and_both_judging_units_only() {
        let directory = TemporaryLibraryDirectory::new("pairing-conclusion");
        let mut store = VerdictStore::create_empty(&directory.path).expect("建库");
        let key = PairingConclusionKey {
            inputs_digest: [1; SHA256_DIGEST_BYTES],
            one_side_judging_units: [2; SHA256_DIGEST_BYTES],
            other_side_judging_units: [3; SHA256_DIGEST_BYTES],
        };
        let conclusion = StoredPairingConclusion {
            conclusion: b"agree states=12".to_vec(),
            circumstances: JudgementCircumstances {
                judged_date: "2026-10-15".to_string(),
                run_kind: RunKindOfTheJudgement::Pruned,
            },
        };
        assert_eq!(store.read_pairing_conclusion(&key).expect("读"), None);
        store
            .store_pairing_conclusion(&key, &conclusion)
            .expect("存");
        assert_eq!(
            store.read_pairing_conclusion(&key).expect("读"),
            Some(conclusion)
        );
        for other in [
            PairingConclusionKey {
                inputs_digest: [9; SHA256_DIGEST_BYTES],
                ..key
            },
            PairingConclusionKey {
                one_side_judging_units: [9; SHA256_DIGEST_BYTES],
                ..key
            },
            PairingConclusionKey {
                other_side_judging_units: [9; SHA256_DIGEST_BYTES],
                ..key
            },
        ] {
            assert_eq!(store.read_pairing_conclusion(&other).expect("读"), None);
        }
    }

    /// 计数向量写成字节再读回逐项相同（空的也一样）；格式号不对、读不够、后面多出字节、名字不是严格升序的，都读不回。
    #[test]
    fn count_vector_bytes_round_trip_and_refuse_malformed_bytes() {
        let count_vector = count_vector_of(7);
        let bytes = count_vector_bytes(&count_vector);
        assert_eq!(count_vector_from_bytes(&bytes), Some(count_vector.clone()));
        assert_eq!(
            count_vector_from_bytes(&count_vector_bytes(&BlockCountVector::default())),
            Some(BlockCountVector::default())
        );
        let mut wrong_format = bytes.clone();
        wrong_format[0] = COUNT_VECTOR_FORMAT_NUMBER + 1;
        let mut trailing = bytes.clone();
        trailing.push(0);
        let truncated = &bytes[..bytes.len() - 1];
        for (malformed, what) in [
            (&wrong_format[..], "格式号不对"),
            (&trailing[..], "后面多出字节"),
            (truncated, "读不够"),
        ] {
            assert_eq!(count_vector_from_bytes(malformed), None, "{what}");
        }
        let mut unsorted = vec![COUNT_VECTOR_FORMAT_NUMBER];
        push_section_entry_count(&mut unsorted, 0);
        push_section_entry_count(&mut unsorted, 2);
        for name in ["b", "a"] {
            push_length_prefixed_text(&mut unsorted, name);
            unsorted.extend_from_slice(&1u64.to_be_bytes());
        }
        push_section_entry_count(&mut unsorted, 0);
        assert_eq!(count_vector_from_bytes(&unsorted), None, "名字不是严格升序");
    }

    /// 这一套之前的布局（没有布局行，今天建的库就是这样）：按结论种类打开拒绝；照旧打开的能用旧的那几张表，按结论种类存取一律拒绝、什么都不写。
    #[test]
    fn a_library_in_the_layout_before_conclusion_kinds_is_refused_by_every_conclusion_access() {
        let directory = TemporaryLibraryDirectory::new("layout-before-conclusion-kinds");
        {
            let store = VerdictStore::create_empty(&directory.path).expect("建库");
            store
                .database
                .delete(VerdictStoreKey::LibraryLayout.to_key_bytes().0)
                .expect("删掉布局行，造一个这一套之前的库");
        }
        let refused = VerdictStore::open_existing_with_conclusion_kinds(&directory.path);
        assert!(
            matches!(
                refused,
                Err(VerdictStoreError::LibraryLayoutPredatesConclusionKinds { .. })
            ),
            "按结论种类打开要拒：{:?}",
            refused.err()
        );
        let (mut store, _opening) = VerdictStore::open_existing(&directory.path).expect("照旧打开");
        assert_eq!(store.layout(), VerdictStoreLayout::BeforeConclusionKinds);
        let clean = store
            .register_verdict_vector(&verdict_vector(b""))
            .expect("旧表照用");
        let entries_before = store.every_stored_entry();
        let block_key = block_key_named("node", 0);
        let stored = store.store_conclusion_block(
            &block_key,
            &ConclusionKind::ImageJudgement,
            &JUDGEMENT_CONCLUSION_VERSION,
            &conclusion_record(block_of(&[clean]), &[]),
        );
        assert!(
            matches!(
                stored,
                Err(VerdictStoreError::LibraryLayoutPredatesConclusionKinds { .. })
            ),
            "按结论种类存要拒：{stored:?}"
        );
        assert!(
            matches!(
                store.conclusions_missing_for_block(&block_key, &[]),
                Err(VerdictStoreError::LibraryLayoutPredatesConclusionKinds { .. })
            ),
            "按结论种类对账要拒"
        );
        assert_eq!(
            store.every_stored_entry(),
            entries_before,
            "拒了就一个字都不写"
        );
    }

    /// 同一（块键，种类，版本）下换一份结论（块、违例、计数向量、恢复字任一不同）拒绝，拒了一个字都不写；结论相同只换块属性（换一张卡判的）
    /// 算已存、不写、块属性留先存的那一份；逐项相同的再存放行、不写。
    #[test]
    fn a_different_conclusion_under_a_stored_key_is_refused_while_another_judging_device_is_accepted_and_the_library_is_unchanged(
    ) {
        let directory = TemporaryLibraryDirectory::new("different-conclusion");
        let mut store = VerdictStore::create_empty(&directory.path).expect("建库");
        let clean = store
            .register_verdict_vector(&verdict_vector(b""))
            .expect("登记");
        let red = store
            .register_verdict_vector(&verdict_vector(b"red"))
            .expect("登记");
        let block_key = block_key_named("node", 0);
        let record = conclusion_record(block_of(&[clean, red]), &[1]);
        store
            .store_conclusion_block(
                &block_key,
                &ConclusionKind::ImageJudgement,
                &JUDGEMENT_CONCLUSION_VERSION,
                &record,
            )
            .expect("存");
        let entries_before = store.every_stored_entry();
        let mut other_counts = record.clone();
        other_counts
            .count_vector
            .per_observer
            .insert("mounted_states".to_string(), 99);
        let mut other_recovery_words = record.clone();
        other_recovery_words.recovery_words = Some(recovery_words_of(3)[1..].to_vec());
        let mut other_violations = record.clone();
        other_violations.violating_states = vec![];
        for (different, what) in [
            (other_counts, "计数向量不同"),
            (other_recovery_words, "恢复字不同"),
            (other_violations, "违例不同"),
        ] {
            let refused = store.store_conclusion_block(
                &block_key,
                &ConclusionKind::ImageJudgement,
                &JUDGEMENT_CONCLUSION_VERSION,
                &different,
            );
            assert!(
                matches!(
                    refused,
                    Err(VerdictStoreError::DifferentConclusionUnderAStoredKeyNotDecidedInTheFirstVersion { .. })
                ),
                "{what}要拒：{refused:?}"
            );
        }
        let mut other_device = record.clone();
        other_device.judged_on = JudgingDevice::Gpu {
            card_type: "probe card".to_string(),
            driver_version: "probe driver 2".to_string(),
            gpu_judge_units_fingerprint: [7; SHA256_DIGEST_BYTES],
        };
        assert_eq!(
            store
                .store_conclusion_block(
                    &block_key,
                    &ConclusionKind::ImageJudgement,
                    &JUDGEMENT_CONCLUSION_VERSION,
                    &other_device,
                )
                .expect("结论相同、只有在哪判的不同：算已存"),
            VerdictBlockStoring::AlreadyStoredIdentically,
            "同一张表换一张卡判出同一份结论，算已存"
        );
        assert_eq!(
            store.every_stored_entry(),
            entries_before,
            "算已存就一个字都不写，块属性留先存的那一份"
        );
        assert_eq!(
            store
                .read_conclusion_block(
                    &block_key,
                    &ConclusionKind::ImageJudgement,
                    &JUDGEMENT_CONCLUSION_VERSION,
                )
                .expect("读")
                .expect("存着")
                .judged_on,
            record.judged_on,
            "留的是先存的那一份块属性"
        );
        assert_eq!(
            store
                .store_conclusion_block(
                    &block_key,
                    &ConclusionKind::ImageJudgement,
                    &JUDGEMENT_CONCLUSION_VERSION,
                    &record
                )
                .expect("再存同一份"),
            VerdictBlockStoring::AlreadyStoredIdentically
        );
        assert_eq!(store.every_stored_entry(), entries_before);
    }

    /// 导入把结论连同计数向量、块属性与恢复字带过去，判定向量按内容在这个库里重新编号。
    #[test]
    fn import_copies_conclusions_with_their_count_vectors_attributes_and_recovery_words() {
        let source_directory = TemporaryLibraryDirectory::new("conclusion-import-source");
        let mut source = VerdictStore::create_empty(&source_directory.path).expect("建库");
        let source_red = source
            .register_verdict_vector(&verdict_vector(b"red"))
            .expect("登记");
        let source_clean = source
            .register_verdict_vector(&verdict_vector(b""))
            .expect("登记");
        let block_key = block_key_named("node", 0);
        let observer = ConclusionKind::Observer;
        source
            .store_conclusion_block(
                &block_key,
                &observer,
                &OBSERVER_CONCLUSION_VERSION,
                &conclusion_record(block_of(&[source_clean, source_red]), &[1]),
            )
            .expect("存");
        let merged_directory = TemporaryLibraryDirectory::new("conclusion-import-merged");
        let mut merged = VerdictStore::create_empty(&merged_directory.path).expect("建库");
        let merged_clean = merged
            .register_verdict_vector(&verdict_vector(b""))
            .expect("登记");
        assert_eq!(merged.import_from(&source).expect("导入"), 1, "导过去一块");
        let merged_red = merged
            .register_verdict_vector(&verdict_vector(b"red"))
            .expect("导入时已登记过");
        let stored = merged
            .read_conclusion_block(&block_key, &observer, &OBSERVER_CONCLUSION_VERSION)
            .expect("读")
            .expect("导过来了");
        let expected = conclusion_record(block_of(&[merged_clean, merged_red]), &[1]);
        assert_eq!(stored.block, expected.block, "按内容重新编号");
        assert_eq!(stored.count_vector, expected.count_vector);
        assert_eq!(stored.judged_on, expected.judged_on);
        assert_eq!(stored.recovery_words, expected.recovery_words);
    }

    /// 按计划导入的对拍用的流 F 与另一条流 G 的节点名、F 的块。
    const FLOW_F_NODE: &str = "flow-f::node-f::0011";
    const FLOW_G_NODE: &str = "flow-g::node-g::0022";
    /// 本库 A 里流 F 在一把旧版本下的历史行（判法改过、旧行留着）。
    const OLD_JUDGEMENT_CONCLUSION_VERSION: ConclusionVersion =
        ConclusionVersion([0x54; SHA256_DIGEST_BYTES]);
    const CPU_VERIFIER_DIGEST: [u8; SHA256_DIGEST_BYTES] = [0x81; SHA256_DIGEST_BYTES];
    const GPU_VERIFIER_DIGEST: [u8; SHA256_DIGEST_BYTES] = [0x82; SHA256_DIGEST_BYTES];
    const OLD_FACTS_DIGEST: [u8; SHA256_DIGEST_BYTES] = [0x71; SHA256_DIGEST_BYTES];
    const FLOW_F_FACTS_DIGEST: [u8; SHA256_DIGEST_BYTES] = [0x72; SHA256_DIGEST_BYTES];
    const FLOW_G_FACTS_DIGEST: [u8; SHA256_DIGEST_BYTES] = [0x73; SHA256_DIGEST_BYTES];

    fn flow_f_block(block_index: u64) -> VerdictBlockKey {
        block_key_named(FLOW_F_NODE, block_index * STATES_PER_FLOW_F_TEST_BLOCK)
    }

    fn flow_g_block(block_index: u64) -> VerdictBlockKey {
        block_key_named(FLOW_G_NODE, block_index * STATES_PER_FLOW_F_TEST_BLOCK)
    }

    /// 对拍里块起点的步长（块里的状态数各块自己定，不超过它）。
    const STATES_PER_FLOW_F_TEST_BLOCK: u64 = 4;

    fn observer_record(block: VerdictBlock, violating_states: &[u64]) -> ConclusionBlockRecord {
        let states = u64::try_from(block.verdict_vector_number_of_each_state().len())
            .expect("测试里的块不长");
        ConclusionBlockRecord {
            block,
            violating_states: violating_states
                .iter()
                .copied()
                .map(StateOffsetInBlock)
                .collect(),
            count_vector: count_vector_of(states),
            judged_on: JudgingDevice::Cpu,
            recovery_words: None,
        }
    }

    fn store_conclusion_in_the_test_library(
        store: &mut VerdictStore,
        block_key: &VerdictBlockKey,
        kind: &ConclusionKind,
        conclusion_version: &ConclusionVersion,
        record: &ConclusionBlockRecord,
    ) {
        store
            .store_conclusion_block(block_key, kind, conclusion_version, record)
            .expect("测试库里存得进结论");
    }

    /// 本库 A（导入那一趟开跑时本机的库）：流 F 四块都录了，第 0 块是本机这一份、两种判器都判过；第 1 块的 CPU 结论早先导入过一次；
    /// 第 0、1 块在旧 GPU 版本下有历史行；流 G 第 0 块判过；三条路径；一份旧事实行。同一个目录名之外逐字节相同，两份 A 各调一次。
    fn build_the_local_library_a(directory: &Path) -> VerdictStore {
        let mut store = VerdictStore::create_empty(directory).expect("建库");
        let clean = store
            .register_verdict_vector(&verdict_vector(b""))
            .expect("登记");
        let red = store
            .register_verdict_vector(&verdict_vector(b"red-f"))
            .expect("登记");
        for (block_index, state_count) in [(0, 2), (1, 2), (2, 3), (3, 2)] {
            store
                .record_block(&flow_f_block(block_index), state_count)
                .expect("录入");
        }
        store.record_block(&flow_g_block(0), 2).expect("录入");
        let judged_by_this_host = conclusion_record(block_of(&[clean, red]), &[1]);
        for (kind, conclusion_version) in [
            (ConclusionKind::ImageJudgement, JUDGEMENT_CONCLUSION_VERSION),
            (
                ConclusionKind::ImageJudgement,
                ANOTHER_JUDGEMENT_CONCLUSION_VERSION,
            ),
        ] {
            store_conclusion_in_the_test_library(
                &mut store,
                &flow_f_block(0),
                &kind,
                &conclusion_version,
                &judged_by_this_host,
            );
        }
        store_conclusion_in_the_test_library(
            &mut store,
            &flow_f_block(1),
            &ConclusionKind::ImageJudgement,
            &ANOTHER_JUDGEMENT_CONCLUSION_VERSION,
            &conclusion_record(block_of(&[clean, red]), &[1]),
        );
        store_conclusion_in_the_test_library(
            &mut store,
            &flow_f_block(0),
            &ConclusionKind::ImageJudgement,
            &OLD_JUDGEMENT_CONCLUSION_VERSION,
            &conclusion_record(block_of(&[clean, clean]), &[]),
        );
        store_conclusion_in_the_test_library(
            &mut store,
            &flow_f_block(1),
            &ConclusionKind::ImageJudgement,
            &OLD_JUDGEMENT_CONCLUSION_VERSION,
            &conclusion_record(block_of(&[red, clean]), &[0]),
        );
        store_conclusion_in_the_test_library(
            &mut store,
            &flow_g_block(0),
            &ConclusionKind::ImageJudgement,
            &JUDGEMENT_CONCLUSION_VERSION,
            &conclusion_record(block_of(&[clean, clean]), &[]),
        );
        for (path, identity) in [
            ("f.0", "flow-f-identity-0"),
            ("f.0/f.1", "flow-f-identity-1-before-the-fork"),
            ("g.0", "flow-g-identity-0"),
        ] {
            store.record_path(path, identity).expect("登路径");
        }
        store
            .store_flow_facts(&OLD_FACTS_DIGEST, b"facts of an older extractor")
            .expect("登事实");
        store
    }

    /// 对方库 B（第二台取回的库）：A 的快照加上第二台这一趟判的——流 F 第 1 块的 GPU 结论、第 2 块的 GPU 与观察者结论（用到一种 A 没登记过的
    /// 判定向量）、这两块在 CPU 核对代码下的核对行与 F 的事实行、F 的两条路径（一条与 A 同路径不同身份，是同名不同数据并成一段的那种）；
    /// 第 3 块没判到。另一条流 G 第 1 块也判了、核了、登了路径与事实（不属于 F）。
    fn build_the_other_library_b(directory: &Path) -> VerdictStore {
        let mut store = build_the_local_library_a(directory);
        let clean = store
            .register_verdict_vector(&verdict_vector(b""))
            .expect("登记");
        let red = store
            .register_verdict_vector(&verdict_vector(b"red-f"))
            .expect("登记");
        let red_only_in_the_other_library = store
            .register_verdict_vector(&verdict_vector(b"red-f-judged-on-the-other-host"))
            .expect("登记");
        store_conclusion_in_the_test_library(
            &mut store,
            &flow_f_block(1),
            &ConclusionKind::ImageJudgement,
            &JUDGEMENT_CONCLUSION_VERSION,
            &conclusion_record(block_of(&[clean, red]), &[1]),
        );
        let block_two = block_of(&[red_only_in_the_other_library, clean, clean]);
        store_conclusion_in_the_test_library(
            &mut store,
            &flow_f_block(2),
            &ConclusionKind::ImageJudgement,
            &JUDGEMENT_CONCLUSION_VERSION,
            &conclusion_record(block_two.clone(), &[0]),
        );
        store_conclusion_in_the_test_library(
            &mut store,
            &flow_f_block(2),
            &ConclusionKind::Observer,
            &OBSERVER_CONCLUSION_VERSION,
            &observer_record(block_two, &[0]),
        );
        store_conclusion_in_the_test_library(
            &mut store,
            &flow_g_block(1),
            &ConclusionKind::ImageJudgement,
            &JUDGEMENT_CONCLUSION_VERSION,
            &conclusion_record(block_of(&[red, clean]), &[0]),
        );
        for (block_key, facts_digest, bits) in [
            (flow_f_block(1), FLOW_F_FACTS_DIGEST, vec![0, 1]),
            (flow_f_block(2), FLOW_F_FACTS_DIGEST, vec![1, 0, 0]),
            (flow_g_block(1), FLOW_G_FACTS_DIGEST, vec![1, 0]),
        ] {
            store
                .store_verifier_block(&block_key, &CPU_VERIFIER_DIGEST, &facts_digest, &bits)
                .expect("写核对行");
        }
        store
            .store_flow_facts(&FLOW_F_FACTS_DIGEST, b"facts of flow f")
            .expect("登事实");
        store
            .store_flow_facts(&FLOW_G_FACTS_DIGEST, b"facts of flow g")
            .expect("登事实");
        for (path, identity) in [
            ("f.0/f.1", "flow-f-identity-1-after-the-fork"),
            ("f.0/f.1/f.2", "flow-f-identity-2"),
            ("g.0/g.1", "flow-g-identity-1"),
        ] {
            store.record_path(path, identity).expect("登路径");
        }
        store
    }

    /// 流水线导入那一趟照计划点名的键（`crash_amplification` 导入那一段的形态）：流 F 里不归本机的第 1–3 块，每块两种判器各一把当前版本、
    /// 观察者一把，两种核对代码各一行；路径是计划里 F 的每个点。
    fn planned_keys_of_flow_f_beyond_this_host_s_share() -> PlannedImportKeys {
        let mut planned = PlannedImportKeys {
            conclusions: Vec::new(),
            paths: vec![
                "f.0".to_string(),
                "f.0/f.1".to_string(),
                "f.0/f.1/f.2".to_string(),
            ],
            verifier_blocks: Vec::new(),
        };
        for block_index in 1..4 {
            for (kind, conclusion_version) in [
                (ConclusionKind::ImageJudgement, JUDGEMENT_CONCLUSION_VERSION),
                (
                    ConclusionKind::ImageJudgement,
                    ANOTHER_JUDGEMENT_CONCLUSION_VERSION,
                ),
                (ConclusionKind::Observer, OBSERVER_CONCLUSION_VERSION),
            ] {
                planned
                    .conclusions
                    .push((flow_f_block(block_index), kind, conclusion_version));
            }
            for verifier_digest in [CPU_VERIFIER_DIGEST, GPU_VERIFIER_DIGEST] {
                planned
                    .verifier_blocks
                    .push((flow_f_block(block_index), verifier_digest));
            }
        }
        planned
    }

    /// 一块一种结论读回来、换成判定向量的内容（两份库的编号各是各的，按内容比才是「读回相同」）。
    #[derive(Debug, PartialEq, Eq)]
    struct ConclusionReadBackByContent {
        verdict_vector_of_each_state: Vec<Vec<u8>>,
        violations: Vec<(u64, Vec<u8>)>,
        count_vector_bytes: Vec<u8>,
        judged_on: JudgingDevice,
        recovery_words: Option<Vec<RecoveryWordsOfState>>,
    }

    fn conclusion_read_back_by_content(
        store: &VerdictStore,
        block_key: &VerdictBlockKey,
        kind: &ConclusionKind,
        conclusion_version: &ConclusionVersion,
    ) -> Option<ConclusionReadBackByContent> {
        let vector_bytes = |number: VerdictVectorNumber| {
            store
                .verdict_vector_of_number(number)
                .expect("读回的编号在这个库里登记过")
                .0
                .clone()
        };
        store
            .read_conclusion_block(block_key, kind, conclusion_version)
            .expect("读结论")
            .map(|stored| ConclusionReadBackByContent {
                verdict_vector_of_each_state: stored
                    .block
                    .verdict_vector_number_of_each_state()
                    .iter()
                    .map(|number| vector_bytes(*number))
                    .collect(),
                violations: stored
                    .violations
                    .iter()
                    .map(|violation| {
                        (
                            violation.offset_in_block.0,
                            vector_bytes(violation.verdict_vector_number),
                        )
                    })
                    .collect(),
                count_vector_bytes: count_vector_bytes(&stored.count_vector),
                judged_on: stored.judged_on,
                recovery_words: stored.recovery_words,
            })
    }

    /// B 里属于流 G 的行：键里带 G 的节点名的（G 的块的结论、核对行），G 的路径与事实行。
    fn is_a_row_of_flow_g(key_bytes: &[u8]) -> bool {
        let flow_g_node_digest = node_input_digest_named(FINGERPRINT_BYTE, FLOW_G_NODE);
        key_bytes
            .windows(SHA256_DIGEST_BYTES)
            .any(|window| window == flow_g_node_digest.0)
            || key_bytes == VerdictStoreKey::Path("g.0/g.1").to_key_bytes().0.as_slice()
            || key_bytes
                == VerdictStoreKey::FlowFacts(&FLOW_G_FACTS_DIGEST)
                    .to_key_bytes()
                    .0
                    .as_slice()
    }

    /// 优化前后对拍（`.claude/rules/test-script-discipline.md`「编译与算法都要快」：优化前后在同一组输入上判定逐项相同）：
    /// 两份逐字节相同的本库 A，一份整库导入对方库 B（[`VerdictStore::import_from`]，改之前流水线的导法），一份按计划只导流 F 不归本机的那几块；
    /// 流 F 的每个块键在每种结论、每把版本下读回相同，F 的路径、核对行、事实行读回相同。按计划导入写下的行数钉成绝对值，
    /// 写下的正是 B 比 A 多出、属于 F 的那几行；B 里流 G 的新行不写。
    #[test]
    fn importing_the_planned_rows_of_a_flow_reads_back_like_importing_the_whole_library_and_writes_only_that_flow_s_new_rows(
    ) {
        let other_directory = TemporaryLibraryDirectory::new("planned-import-other");
        let other = build_the_other_library_b(&other_directory.path);
        let whole_directory = TemporaryLibraryDirectory::new("planned-import-whole");
        let mut imported_whole = build_the_local_library_a(&whole_directory.path);
        let planned_directory = TemporaryLibraryDirectory::new("planned-import-planned");
        let mut imported_by_plan = build_the_local_library_a(&planned_directory.path);
        let entries_before = imported_by_plan.every_stored_entry();
        assert_eq!(
            entries_before,
            imported_whole.every_stored_entry(),
            "两份本库 A 开头逐字节相同"
        );
        imported_whole.import_from(&other).expect("整库导入");
        let tally = imported_by_plan
            .import_the_planned_rows_from(
                &other,
                &planned_keys_of_flow_f_beyond_this_host_s_share(),
            )
            .expect("按计划导入");
        for block_index in 0..4 {
            for (kind, conclusion_version) in [
                (ConclusionKind::ImageJudgement, JUDGEMENT_CONCLUSION_VERSION),
                (
                    ConclusionKind::ImageJudgement,
                    OLD_JUDGEMENT_CONCLUSION_VERSION,
                ),
                (
                    ConclusionKind::ImageJudgement,
                    ANOTHER_JUDGEMENT_CONCLUSION_VERSION,
                ),
                (ConclusionKind::Observer, OBSERVER_CONCLUSION_VERSION),
            ] {
                let block_key = flow_f_block(block_index);
                assert_eq!(
                    conclusion_read_back_by_content(
                        &imported_by_plan,
                        &block_key,
                        &kind,
                        &conclusion_version
                    ),
                    conclusion_read_back_by_content(
                        &imported_whole,
                        &block_key,
                        &kind,
                        &conclusion_version
                    ),
                    "流 F 第 {block_index} 块的 {kind:?} 在版本 {:02x}… 下两种导法读回不同",
                    conclusion_version.0[0]
                );
            }
            for verifier_digest in [CPU_VERIFIER_DIGEST, GPU_VERIFIER_DIGEST] {
                assert_eq!(
                    imported_by_plan
                        .read_verifier_block(&flow_f_block(block_index), &verifier_digest)
                        .expect("读核对行"),
                    imported_whole
                        .read_verifier_block(&flow_f_block(block_index), &verifier_digest)
                        .expect("读核对行"),
                    "流 F 第 {block_index} 块在核对代码 {:02x}… 下的核对行两种导法读回不同",
                    verifier_digest[0]
                );
            }
        }
        for path in ["f.0", "f.0/f.1", "f.0/f.1/f.2"] {
            assert_eq!(
                imported_by_plan.identity_of_path(path).expect("读路径"),
                imported_whole.identity_of_path(path).expect("读路径"),
                "流 F 的路径 {path} 两种导法读回不同"
            );
        }
        assert_eq!(
            imported_by_plan
                .read_flow_facts(&FLOW_F_FACTS_DIGEST)
                .expect("读事实"),
            imported_whole
                .read_flow_facts(&FLOW_F_FACTS_DIGEST)
                .expect("读事实"),
            "流 F 的事实行两种导法读回不同"
        );
        // 绝对值：结论 3 块写下（F1 的 GPU 5 行、F2 的 GPU 5 行、F2 的观察者 4 行），F1 的 CPU 本库已有、相同；
        // F1 的观察者、F2 的 CPU、F3 的三种对方库里没有。另：新判定向量 2 行、路径 2 行（f.0 相同跳过）、核对 2 行、事实 1 行
        assert_eq!(
            tally,
            PlannedImportTally {
                conclusion_blocks_written: 3,
                conclusion_blocks_already_identical: 1,
                conclusion_blocks_missing_in_the_other_library: 5,
                paths_written: 2,
                verifier_blocks_written: 2,
                flow_facts_written: 1,
                rows_written: 21,
                synced_writes: 1,
            },
            "按计划导入的账"
        );
        let entries_before: BTreeSet<StoredEntry> = entries_before.into_iter().collect();
        let written_by_plan: BTreeSet<StoredEntry> = imported_by_plan
            .every_stored_entry()
            .into_iter()
            .filter(|entry| !entries_before.contains(entry))
            .collect();
        let other_beyond_a_of_flow_f: BTreeSet<StoredEntry> = other
            .every_stored_entry()
            .into_iter()
            .filter(|entry| !entries_before.contains(entry) && !is_a_row_of_flow_g(&entry.0))
            .collect();
        assert_eq!(written_by_plan.len(), 21, "按计划导入写下 21 行");
        assert_eq!(
            written_by_plan, other_beyond_a_of_flow_f,
            "按计划导入写下的正是 B 比 A 多出、属于流 F 的那几行"
        );
        assert!(
            conclusion_read_back_by_content(
                &imported_whole,
                &flow_g_block(1),
                &ConclusionKind::ImageJudgement,
                &JUDGEMENT_CONCLUSION_VERSION
            )
            .is_some()
                && conclusion_read_back_by_content(
                    &imported_by_plan,
                    &flow_g_block(1),
                    &ConclusionKind::ImageJudgement,
                    &JUDGEMENT_CONCLUSION_VERSION
                )
                .is_none(),
            "流 G 的新结论整库导入带过来了，按计划导入不碰它"
        );
    }

    /// 按计划导入时本库同键已有不同的结论（流 F 第 2 块的 GPU 结论两边判得不同）：照存块的规矩报错，点名那一块那一种；
    /// 排在它前面已攒进这一批的行（F1 的 GPU 结论）一行都不写，本库逐字节不变。
    #[test]
    fn planned_import_refuses_a_different_conclusion_under_a_stored_key_and_leaves_the_library_unchanged(
    ) {
        let other_directory = TemporaryLibraryDirectory::new("planned-import-conflict-other");
        let other = build_the_other_library_b(&other_directory.path);
        let local_directory = TemporaryLibraryDirectory::new("planned-import-conflict-local");
        let mut local = build_the_local_library_a(&local_directory.path);
        let red = local
            .register_verdict_vector(&verdict_vector(b"red-f"))
            .expect("登记");
        store_conclusion_in_the_test_library(
            &mut local,
            &flow_f_block(2),
            &ConclusionKind::ImageJudgement,
            &JUDGEMENT_CONCLUSION_VERSION,
            &conclusion_record(block_of(&[red, red, red]), &[0, 1, 2]),
        );
        let entries_before = local.every_stored_entry();
        match local.import_the_planned_rows_from(
            &other,
            &planned_keys_of_flow_f_beyond_this_host_s_share(),
        ) {
            Err(
                VerdictStoreError::DifferentConclusionUnderAStoredKeyNotDecidedInTheFirstVersion {
                    block_key,
                    kind,
                },
            ) => {
                assert_eq!(block_key, flow_f_block(2), "点名判得不同的那一块");
                assert_eq!(kind, ConclusionKind::ImageJudgement, "点名判得不同的那一种");
            }
            other_outcome => panic!("本库同键已有不同的结论应当报错，得到 {other_outcome:?}"),
        }
        assert_eq!(
            local.every_stored_entry(),
            entries_before,
            "报错之前攒着的那一批一行都没写"
        );
    }

    /// 一批攒到一个字节就同步写：放完一块一种结论（或一个核对行、一份事实）就换一批，几百字节的库也走多批。
    const ONE_BYTE_BATCH_WRITES_AFTER_EVERY_PUT_GROUP: usize = 1;

    /// 对方库 B 再加流 F 第 3 块的 GPU 结论，用到又一种 A 没登记过的判定向量：按计划导入时 F2、F3 的 GPU 结论各带一种新向量，
    /// 一字节一批时落在两批里（第二种的编号要接在上一批登记的那一种后面）。
    fn build_the_other_library_b_with_a_second_new_verdict_vector(
        directory: &Path,
    ) -> VerdictStore {
        let mut store = build_the_other_library_b(directory);
        let clean = store
            .register_verdict_vector(&verdict_vector(b""))
            .expect("登记");
        let red_of_block_three = store
            .register_verdict_vector(&verdict_vector(b"red-f-block-three-on-the-other-host"))
            .expect("登记");
        store_conclusion_in_the_test_library(
            &mut store,
            &flow_f_block(3),
            &ConclusionKind::ImageJudgement,
            &JUDGEMENT_CONCLUSION_VERSION,
            &conclusion_record(block_of(&[red_of_block_three, clean]), &[0]),
        );
        store
    }

    /// 第二份对方库 C（流水线逐份导入时的又一份第二台的库）：A 的快照加上流 F 第 3 块的 CPU 结论，用到 B 带过来的那两种判定向量
    /// （内容相同，在 C 里的编号与本库不同：C 先登记 F3 那一种、再登记 F2 那一种）。
    fn build_the_second_other_library_c_with_the_same_new_verdict_vectors(
        directory: &Path,
    ) -> VerdictStore {
        let mut store = build_the_local_library_a(directory);
        let clean = store
            .register_verdict_vector(&verdict_vector(b""))
            .expect("登记");
        let red_of_block_three = store
            .register_verdict_vector(&verdict_vector(b"red-f-block-three-on-the-other-host"))
            .expect("登记");
        let red_judged_on_the_other_host = store
            .register_verdict_vector(&verdict_vector(b"red-f-judged-on-the-other-host"))
            .expect("登记");
        store_conclusion_in_the_test_library(
            &mut store,
            &flow_f_block(3),
            &ConclusionKind::ImageJudgement,
            &ANOTHER_JUDGEMENT_CONCLUSION_VERSION,
            &conclusion_record(
                block_of(&[red_of_block_three, red_judged_on_the_other_host, clean]),
                &[0, 1],
            ),
        );
        store
    }

    /// 换批（正推 B1）：两份逐字节相同的本库 A，从同一份 B（比 A 多两种判定向量）按同一份计划导入，一份照产品的批长（一批写完），
    /// 一份一字节一批（每块一批）。两份的账只差同步写的批数；写下的库逐字节相同，跨批登记的判定向量编号接着往后编（A 登记了 0、1，
    /// F2 那一批登记 2、F3 那一批登记 3）；重开之后流 F 每块每种结论按内容读回与 B 相同。
    /// 绝对值：结论 4 块写下（F1 的 GPU 5 行、F2 的 GPU 5 行加新向量 2 行、F2 的观察者 4 行、F3 的 GPU 5 行加新向量 2 行），F1 的 CPU 本库已有、相同；
    /// F1 的观察者、F2 的 CPU、F3 的 CPU 与观察者对方库里没有；路径 2 行、核对 2 行、事实 1 行，共 28 行。一字节一批的批：四块结论各一批，
    /// 两条路径与 F1 的核对行一批（路径放完不判批长），F2 的核对行一批，事实一批，共 7 批。
    /// 照产品的批长导完的那一份不重开、接着用（流水线里导入在第 ① 段判之前，判本机的块、存观察者结论都经内存表登记与读编号；攻方 G1）：
    /// 一批里登记的两种向量照登记次序进内存表（编号 2、3），流 F 每块每种结论按内存表读回与 B 相同；再登记一种新向量，编号是 4。
    /// 同一个打开着的库接着从第二份对方库 C 导入（`crash_amplification` 逐份导入 `SINGLEFS_CRASH_AMPLIFICATION_IMPORT` 里的几份库那样）：
    /// C 的 F3 CPU 结论用的两种向量按内容找到本库的 2、3，不另起编号；账钉成绝对值（F3 的 CPU 6 行、F1 的 CPU 相同、其余 7 项 C 里没有）。
    /// 重开之后流 F 每块每种结论按内容读回与 B 相同（F3 的 CPU 与 C 相同），编号 4 读得回、编号 5 没有。
    #[test]
    fn planned_import_in_one_byte_batches_numbers_new_verdict_vectors_on_across_batches_and_writes_what_one_batch_writes(
    ) {
        let other_directory = TemporaryLibraryDirectory::new("planned-import-batches-other");
        let other =
            build_the_other_library_b_with_a_second_new_verdict_vector(&other_directory.path);
        let one_batch_directory = TemporaryLibraryDirectory::new("planned-import-batches-one");
        let mut imported_in_one_batch = build_the_local_library_a(&one_batch_directory.path);
        let many_batches_directory = TemporaryLibraryDirectory::new("planned-import-batches-many");
        let mut imported_in_many_batches = build_the_local_library_a(&many_batches_directory.path);
        assert_eq!(
            imported_in_one_batch.every_stored_entry(),
            imported_in_many_batches.every_stored_entry(),
            "两份本库 A 开头逐字节相同"
        );
        let planned = planned_keys_of_flow_f_beyond_this_host_s_share();
        let one_batch_tally = imported_in_one_batch
            .import_the_planned_rows_from(&other, &planned)
            .expect("照产品的批长导入");
        let many_batches_tally = imported_in_many_batches
            .import_the_planned_rows_in_batches_of(
                &other,
                &planned,
                ONE_BYTE_BATCH_WRITES_AFTER_EVERY_PUT_GROUP,
            )
            .expect("一字节一批导入");
        assert_eq!(
            one_batch_tally,
            PlannedImportTally {
                conclusion_blocks_written: 4,
                conclusion_blocks_already_identical: 1,
                conclusion_blocks_missing_in_the_other_library: 4,
                paths_written: 2,
                verifier_blocks_written: 2,
                flow_facts_written: 1,
                rows_written: 28,
                synced_writes: 1,
            },
            "照产品的批长：一批写完"
        );
        assert_eq!(
            many_batches_tally,
            PlannedImportTally {
                conclusion_blocks_written: 4,
                conclusion_blocks_already_identical: 1,
                conclusion_blocks_missing_in_the_other_library: 4,
                paths_written: 2,
                verifier_blocks_written: 2,
                flow_facts_written: 1,
                rows_written: 28,
                synced_writes: 7,
            },
            "一字节一批：同样的 28 行分 7 批写"
        );
        assert_eq!(
            imported_in_many_batches.every_stored_entry(),
            imported_in_one_batch.every_stored_entry(),
            "多批与一批写下的库逐字节相同（跨批登记的判定向量编号接着上一批往后编）"
        );
        let kinds_of_the_plan = [
            (ConclusionKind::ImageJudgement, JUDGEMENT_CONCLUSION_VERSION),
            (
                ConclusionKind::ImageJudgement,
                ANOTHER_JUDGEMENT_CONCLUSION_VERSION,
            ),
            (ConclusionKind::Observer, OBSERVER_CONCLUSION_VERSION),
        ];
        for (number, expected_vector) in [
            (2, verdict_vector(b"red-f-judged-on-the-other-host")),
            (3, verdict_vector(b"red-f-block-three-on-the-other-host")),
        ] {
            assert_eq!(
                imported_in_one_batch.verdict_vector_of_number(VerdictVectorNumber(number)),
                Some(&expected_vector),
                "照产品的批长导完、不重开：一批里登记的两种向量照登记次序进内存表，编号 {number} 是这一种"
            );
        }
        for block_index in 1..4 {
            for (kind, conclusion_version) in &kinds_of_the_plan {
                let block_key = flow_f_block(block_index);
                assert_eq!(
                    conclusion_read_back_by_content(
                        &imported_in_one_batch,
                        &block_key,
                        kind,
                        conclusion_version
                    ),
                    conclusion_read_back_by_content(&other, &block_key, kind, conclusion_version),
                    "流 F 第 {block_index} 块的 {kind:?}：照产品的批长导完、不重开，按内存表读回与 B 不同"
                );
            }
        }
        let registered_after_the_import = verdict_vector(b"registered-after-the-import");
        assert_eq!(
            imported_in_one_batch
                .register_verdict_vector(&registered_after_the_import)
                .expect("导入之后再登记"),
            VerdictVectorNumber(4),
            "导入之后再登记一种新向量：编号接在导进来的 0–3 后面"
        );
        let second_other_directory =
            TemporaryLibraryDirectory::new("planned-import-batches-second-other");
        let second_other = build_the_second_other_library_c_with_the_same_new_verdict_vectors(
            &second_other_directory.path,
        );
        let planned_conclusions_only = PlannedImportKeys {
            conclusions: planned.conclusions.clone(),
            paths: Vec::new(),
            verifier_blocks: Vec::new(),
        };
        let second_tally = imported_in_one_batch
            .import_the_planned_rows_from(&second_other, &planned_conclusions_only)
            .expect("同一个打开着的库接着从 C 导入");
        assert_eq!(
            second_tally,
            PlannedImportTally {
                conclusion_blocks_written: 1,
                conclusion_blocks_already_identical: 1,
                conclusion_blocks_missing_in_the_other_library: 7,
                paths_written: 0,
                verifier_blocks_written: 0,
                flow_facts_written: 0,
                rows_written: 6,
                synced_writes: 1,
            },
            "接着从 C 导入：只写 F3 的 CPU 结论 6 行，它用的两种向量本库已登记过、不另登记"
        );
        assert_eq!(
            imported_in_one_batch.verdict_vector_of_number(VerdictVectorNumber(5)),
            None,
            "接着从 C 导入：同内容的向量按内容找到本库的编号，不另起编号 5"
        );
        drop((imported_in_one_batch, imported_in_many_batches));
        let (reopened_one_batch, _) =
            VerdictStore::open_existing(&one_batch_directory.path).expect("重开一批那一份");
        for block_index in 1..4 {
            for (kind, conclusion_version) in &kinds_of_the_plan {
                let block_key = flow_f_block(block_index);
                let imported_from = if block_index == 3
                    && *conclusion_version == ANOTHER_JUDGEMENT_CONCLUSION_VERSION
                {
                    &second_other
                } else {
                    &other
                };
                assert_eq!(
                    conclusion_read_back_by_content(
                        &reopened_one_batch,
                        &block_key,
                        kind,
                        conclusion_version
                    ),
                    conclusion_read_back_by_content(
                        imported_from,
                        &block_key,
                        kind,
                        conclusion_version
                    ),
                    "流 F 第 {block_index} 块的 {kind:?}：一批那一份接着导入 C、重开之后按内容读回与它导自的那份库不同"
                );
            }
        }
        assert_eq!(
            (
                reopened_one_batch.verdict_vector_of_number(VerdictVectorNumber(4)),
                reopened_one_batch.verdict_vector_of_number(VerdictVectorNumber(5)),
            ),
            (Some(&registered_after_the_import), None),
            "重开之后：导入之后登记的那一种是编号 4，没有编号 5"
        );
        drop(reopened_one_batch);
        let (reopened_many_batches, _) =
            VerdictStore::open_existing(&many_batches_directory.path).expect("重开多批那一份");
        for (number, expected_vector) in [
            (2, verdict_vector(b"red-f-judged-on-the-other-host")),
            (3, verdict_vector(b"red-f-block-three-on-the-other-host")),
        ] {
            assert_eq!(
                reopened_many_batches.verdict_vector_of_number(VerdictVectorNumber(number)),
                Some(&expected_vector),
                "编号 {number}：F2 那一批登记 2、F3 那一批接着登记 3"
            );
        }
        for block_index in 1..4 {
            for (kind, conclusion_version) in [
                (ConclusionKind::ImageJudgement, JUDGEMENT_CONCLUSION_VERSION),
                (
                    ConclusionKind::ImageJudgement,
                    ANOTHER_JUDGEMENT_CONCLUSION_VERSION,
                ),
                (ConclusionKind::Observer, OBSERVER_CONCLUSION_VERSION),
            ] {
                let block_key = flow_f_block(block_index);
                assert_eq!(
                    conclusion_read_back_by_content(
                        &reopened_many_batches,
                        &block_key,
                        &kind,
                        &conclusion_version
                    ),
                    conclusion_read_back_by_content(&other, &block_key, &kind, &conclusion_version),
                    "流 F 第 {block_index} 块的 {kind:?}：多批导入重开之后与 B 按内容读回不同"
                );
            }
        }
    }

    /// 事实行补写（正推 B2）：本库是断在核对行与事实行之间的那一种（流 F 第 1、2 块的核对行与 B 逐字节相同、它们点名的 F 的事实行不在）。
    /// 照计划重导这几行核对行：核对行逐字节相同、一行不写，它们点名的事实照样随着导，F 的事实行补上，账是写 1 行、1 批。
    #[test]
    fn planned_import_writes_the_facts_named_by_verifier_rows_this_library_already_has() {
        let other_directory = TemporaryLibraryDirectory::new("planned-import-facts-other");
        let other = build_the_other_library_b(&other_directory.path);
        let local_directory = TemporaryLibraryDirectory::new("planned-import-facts-local");
        let mut local = build_the_local_library_a(&local_directory.path);
        for (block_index, bits) in [(1, vec![0, 1]), (2, vec![1, 0, 0])] {
            local
                .store_verifier_block(
                    &flow_f_block(block_index),
                    &CPU_VERIFIER_DIGEST,
                    &FLOW_F_FACTS_DIGEST,
                    &bits,
                )
                .expect("写核对行");
            assert_eq!(
                local
                    .read_verifier_block(&flow_f_block(block_index), &CPU_VERIFIER_DIGEST)
                    .expect("读核对行"),
                other
                    .read_verifier_block(&flow_f_block(block_index), &CPU_VERIFIER_DIGEST)
                    .expect("读核对行"),
                "前提：流 F 第 {block_index} 块的核对行与 B 相同"
            );
        }
        assert_eq!(
            local.read_flow_facts(&FLOW_F_FACTS_DIGEST).expect("读事实"),
            None,
            "前提：本库没有 F 的事实行"
        );
        let planned_verifier_rows = PlannedImportKeys {
            conclusions: Vec::new(),
            paths: Vec::new(),
            verifier_blocks: planned_keys_of_flow_f_beyond_this_host_s_share().verifier_blocks,
        };
        let tally = local
            .import_the_planned_rows_from(&other, &planned_verifier_rows)
            .expect("重导");
        assert_eq!(
            tally,
            PlannedImportTally {
                conclusion_blocks_written: 0,
                conclusion_blocks_already_identical: 0,
                conclusion_blocks_missing_in_the_other_library: 0,
                paths_written: 0,
                verifier_blocks_written: 0,
                flow_facts_written: 1,
                rows_written: 1,
                synced_writes: 1,
            },
            "核对行都已有且相同、一行不写；它们点名的事实行补写 1 行"
        );
        assert_eq!(
            local.read_flow_facts(&FLOW_F_FACTS_DIGEST).expect("读事实"),
            Some(b"facts of flow f".to_vec()),
            "F 的事实行补上了"
        );
    }

    /// 同一条路径再登同一个身份：什么都不写（库的序号不动）；换一个身份照写。
    #[test]
    fn recording_a_path_again_with_the_same_identity_writes_nothing() {
        let directory = TemporaryLibraryDirectory::new("path-recorded-twice");
        let mut store = VerdictStore::create_empty(&directory.path).expect("建库");
        store
            .record_path("a.unit_write.1", "f::a.unit_write.1::00")
            .expect("登路径");
        let sequence_after_the_first = store.database.latest_sequence_number();
        store
            .record_path("a.unit_write.1", "f::a.unit_write.1::00")
            .expect("再登同一个身份");
        assert_eq!(
            store.database.latest_sequence_number(),
            sequence_after_the_first,
            "同一个身份再登不写库"
        );
        store
            .record_path("a.unit_write.1", "f::a.unit_write.1::11")
            .expect("换一个身份");
        assert!(
            store.database.latest_sequence_number() > sequence_after_the_first,
            "换了身份照写"
        );
        assert_eq!(
            store.identity_of_path("a.unit_write.1").expect("读路径"),
            Some("f::a.unit_write.1::11".to_string()),
            "最后登的身份留在索引里"
        );
    }

    /// 分片进度（C790）：一条记录按（流、段、判器）存、读回逐字相同，换了判器或段读不到；值的字节不合布局时报布局对不上。
    #[test]
    fn a_sliced_segment_progress_reads_back_under_its_flow_segment_and_judge_only() {
        let directory = TemporaryLibraryDirectory::new("sliced-progress");
        let mut store = VerdictStore::create_empty(&directory.path).expect("建库");
        let key_of = |segment_index: u8, version_byte: u8| SlicedSegmentProgressKey {
            segment_content_digest: [segment_index; SHA256_DIGEST_BYTES],
            judge: JudgeOfTheProgress::TheConclusionKind {
                kind: ConclusionKind::ImageJudgement,
                version: ConclusionVersion([version_byte; SHA256_DIGEST_BYTES]),
            },
        };
        assert_eq!(
            store
                .read_sliced_segment_progress(&key_of(1, 7))
                .expect("读"),
            None
        );
        let judged_through = StateCount1280Bits::from_u64(u64::MAX)
            .checked_mul_u64(1 << 40)
            .expect("装得进 1280 位");
        let progress = SlicedSegmentProgress {
            judged_through,
            last_block_start: StateCount1280Bits::from_u64(300),
        };
        store
            .store_sliced_segment_progress(&key_of(1, 7), progress)
            .expect("写");
        assert_eq!(
            store
                .read_sliced_segment_progress(&key_of(1, 7))
                .expect("读"),
            Some(progress)
        );
        assert_eq!(
            store
                .read_sliced_segment_progress(&key_of(1, 8))
                .expect("读"),
            None,
            "换了判法单元指纹读不到"
        );
        assert_eq!(
            store
                .read_sliced_segment_progress(&key_of(2, 7))
                .expect("读"),
            None,
            "换了段读不到"
        );
        store
            .put_one_synced(
                &VerdictStoreKey::SlicedSegmentProgress(&key_of(3, 7)),
                &[1, 2, 3],
            )
            .expect("写");
        assert!(matches!(
            store.read_sliced_segment_progress(&key_of(3, 7)),
            Err(VerdictStoreError::StoredBytesDoNotMatchTheLayout { .. })
        ));
    }

    /// 分片的段的一块（C790）：录过、没判过的块换一个状态数照新的改写（两趟之间片大小改了）；判过的块换状态数照旧拒、录入表不动。
    #[test]
    fn a_slice_block_recorded_but_not_judged_is_recorded_again_with_its_new_length_and_a_judged_one_is_refused(
    ) {
        let directory = TemporaryLibraryDirectory::new("slice-block-rerecord");
        let mut store = VerdictStore::create_empty(&directory.path).expect("建库");
        let unjudged = block_key_named("点", 300);
        store.record_block(&unjudged, 300).expect("录");
        store
            .record_block_of_a_slice(&unjudged, 500)
            .expect("没判过的照新长度改写");
        assert_eq!(
            store.recorded_state_count_of_block(&unjudged).expect("读"),
            Some(500)
        );
        let judged = block_key_named("点", 0);
        store.record_block(&judged, 2).expect("录");
        let clean = store
            .register_verdict_vector(&verdict_vector(b"clean"))
            .expect("登记");
        store
            .store_verdict_block(
                &judged,
                &TEST_JUDGING_UNITS_FINGERPRINT,
                &block_of(&[clean, clean]),
                &[],
            )
            .expect("存块");
        assert!(matches!(
            store.record_block_of_a_slice(&judged, 3),
            Err(VerdictStoreError::DifferentVerdictsUnderAStoredBlockKeyNotDecidedInTheFirstVersion { .. })
        ));
        assert_eq!(
            store.recorded_state_count_of_block(&judged).expect("读"),
            Some(2)
        );
        assert_eq!(
            store
                .recorded_state_count_of_block(&block_key_named("点", 9))
                .expect("读"),
            None
        );
    }
}
