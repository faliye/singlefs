//! 崩溃放量的判定存储（里程碑三第十一项第一段，`.claude/kb/milestone/03-third-txn.md`「十一」）：
//! 按块存每个崩溃状态的判定编号，另存不同的判定向量与违例。全仓只有一份判定库：prune-server 自己的那份 RocksDB
//! （`.claude/rules/verification.md`「判定库只有一份：全仓统一写进 prune-server 的那份 RocksDB」）。这里只经 prune-server 的客户端读写它
//! （点读、写、认领、前缀扫都走 `prune_server::reliable_client`，忙照服务给的毫秒数退避重发），本机不开任何一份 RocksDB；服务读写不成照客户端的退出码表报
//! （75 服务不可用），读失败不当成「库里没有」（[`PruneServerFailureHandling`]）。服务每一帧写都同步落盘之后才确认（与早先本机库的同步写同义）。
//!
//! 全部表都在服务那份库的默认列族里、靠键的第一个字节（表标签）分开，键原样（[`PRUNE_SERVER_TABLE`] 一张表、名字空间是空的）：
//! 一帧写把块与它的违例一起原子地写下去。
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
//! 块键 = 段写表与起步镜像的摘要（32 字节）‖ 枚举计划哈希（32 字节：段展开、判器另读的输入、态数；不分片的块另接上这一块的块长，
//! `crash_amplification::PlannedCrashPoint::block_key`）‖ 块起点（8 字节），定长 72 字节。
//! 块键只由内容定（键里不许有路径、名字、种子）：设点文件路径、崩溃点名、项名、表名、种子串都不进块键，
//! 只进路径索引与报告；判器改了不动块键，判定块与违例另按判法单元指纹分行，复用只认同一把指纹。
//! 块键定长，一个块键不会是另一个块键的前缀，按块键前缀扫违例表只扫到这一块的。
//!
//! 判定向量编号是 u32、大端 4 字节（[`VerdictVectorNumber`]）：「直接加宽到未来都不需要再加宽」，从一个字节一次加宽到 u32，
//! 一个库里最多 2^32 种判定向量。编号宽度是布局的事，跟着布局号走，不进任何复用键。
//!
//! 绿的状态（一个红项都没有）的判定向量是空字节（`crash_judge_conclusions::encoded_verdict_of_the_red_items`），所以按编号那张表里绿那一号的行是
//! `03 ‖ 编号 → 空值`，按内容那张表里绿那一行的键就是单字节 `02`。服务里空值平常是删除记号（读成不在），删除记号对判定向量两张表不适用：
//! 这两张表只追加、从不删，读的时候空值原样算一行（`VerdictStoreTable::what_an_empty_value_means`）；写的时候只有按编号那张表许写空值，
//! 别的表照旧拒；删这两张表的行整批拒（删除记号写在那里读成一行空值，不是删），一行都不写（`PruneServerStorage::write_batch`）。
//!
//! 三种结论另用下面几张表，与上面那几张同库并存：
//!
//! | 表 | 键 | 值 |
//! |---|---|---|
//! | 库布局 | `00` | 布局号，4 字节大端（这一版是 6）；这一行不在（这个键空间头一次放崩溃判定库）开库时写入布局号 6，在而布局号不是 6 或不是四字节的开库就拒 |
//! | 结论块 | `0A` ‖ 块键 ‖ 结论种类 ‖ 版本（32 字节） | 块里每个状态一个判定向量编号（4 字节） |
//! | 结论违例 | `0B` ‖ 块键 ‖ 结论种类 ‖ 版本 ‖ 块内序号（8 字节） | 那个状态的判定向量编号（4 字节） |
//! | 恢复字 | `0C` ‖ 块键 ‖ 结论种类 ‖ 版本 | 每个状态 `RECOVERY_WORDS_PER_STATE` 个字（两遍各自落到的根四个字：落没落到根的标记、实例代号、checkpoint_txg 低 32 位与高 32 位，再是结局、失败点），每字 4 字节小端；不进判定向量编号 |
//! | 计数向量 | `0D` ‖ 块键 ‖ 结论种类 ‖ 版本 | 逐条不变量判到 / 违例 / 不适用、各观察者计数、按发布分的态数（[`BlockCountVector`]） |
//! | 块属性 | `0E` ‖ 块键 ‖ 结论种类 ‖ 版本 | 在哪判的（CPU，或 GPU 连同卡型与驱动版本，[`JudgingDevice`]）、判定日期（东京日期 `YYYY-MM-DD`）与跑法（剪枝、逐条，[`JudgementCircumstances`]）；都不进键 |
//! | 点名的被抛弃根 | `18` ‖ 块键 ‖ 结论种类 ‖ 版本 | 每个状态上池级 checker 判 I-7.4 时违例细节里点名的被抛弃根：条数 4 字节小端，再逐条实例代号 4 字节、checkpoint_txg 8 字节小端；存的时候给了才有（给定状态表那一路每块都给） |
//!
//! 两段（录入段在 CPU 上跑数据、写进库；判段只从库里读，段与段之间不在内存里交数据）另用三张表：
//!
//! | 表 | 键 | 值 |
//! |---|---|---|
//! | 录下的流程 | `0F` ‖ 摘要（32 字节） | `crash_amplification::RecordedFlow` 的字节 |
//! | 一趟的清单 | `10` ‖ 录入键（32 字节：录入函数单元指纹与这一趟录入输入的摘要，[`RecordingKey`]） | `crash_amplification::RecordedRunManifest` 或 `crash_library::ItemManifest` 的字节；命中就不再录 |
//! | GPU 判表 | `11` ‖ 摘要（32 字节） | `crash_judge_dispatch::PackedJudgeTables` 的字节 |
//! | 给定状态表 | `12` ‖ 摘要（32 字节） | `crash_amplification::RecordedGivenStatesTable` 的字节（引擎先写进来、再读回来判） |
//!
//! 按段登记的段整域装不进 u64 时分片枚举（C790），每一段一条进度记录（只是加速：真相以块为准，核不过就从 0 起逐块问库）：
//!
//! | 表 | 键 | 值 |
//! |---|---|---|
//! | 分片进度 | `13` ‖ 段的内容摘要（32 字节）‖ 判器（`01` ‖ 判法单元指纹，或 `02` ‖ 结论种类 ‖ 版本） | 格式号 1 字节 ‖ 已判到的编号 ‖ 最后判完的那一块的起点（各 160 字节，`crash::StateCount1280Bits` 的小端字节） |
//!
//! 观察者两段（「观察者改走两段、池级 checker 上 GPU、按恢复结果去重」）另用四张表：录入段在 CPU 上逐状态跑观察者
//! （交回镜像的那一种按恢复镜像字节去重，同一张镜像只看一次），判段只从这里读、对观察者交回的镜像跑池级 checker（配了 GPU 上卡）：
//!
//! | 表 | 键 | 值 |
//! |---|---|---|
//! | 观察者基底 | `14` ‖ 摘要（32 字节） | 一条流的基础镜像与枚举写表（`crash_amplification` 的观察者基底） |
//! | 看过的恢复镜像 | `15` ‖ 去重键（32 字节） | 观察者在这张镜像上看出的：红项、计数、它交回的镜像（基底摘要、持久集合、观察者在镜像上发的写、判哪几条不变量） |
//! | 观察者录入块 | `16` ‖ 块键 ‖ 结论种类 ‖ 观察者版本 | 块里去重之后的观察结果清单、每个状态的下标、按发布分的态数 |
//! | 镜像上的池级 checker | `17` ‖ 去重键 ‖ 判器种类 ‖ 判器版本 | 判到位、违例位、判不了的标记、在哪判的 |
//!
//! 交回镜像的观察者按恢复报告分桶、按读集见证去重（「恢复报告分桶 + 读集见证」），见证与读集各按行存：添一条见证只写这一条见证行、
//! 这组读集在这个桶里头一次出现时的那一行读集行、它看出的那张镜像（`15` 表），三样同一批原子地落盘；读回时按桶键前缀扫出这个桶的全部见证行。
//!
//! | 表 | 键 | 值 |
//! |---|---|---|
//! | 读集见证行 | `1A` ‖ 桶键（32 字节）‖ 见证摘要（32 字节） | 读集摘要（32 字节，读集行的键）‖ 它看出的那张镜像的去重键（32 字节） |
//! | 读集行 | `1B` ‖ 读集摘要（32 字节） | 观察者真跑一次读过的段（`crash_amplification` 的读集字节；读集摘要就是这份字节的 SHA-256） |
//!
//! 对拍（GPU 与 CPU 两边判同一份输入、比结论）的结论另用一张表：
//!
//! | 表 | 键 | 值 |
//! |---|---|---|
//! | 对拍结论 | `19` ‖ 输入摘要（32 字节）‖ 一边的判法单元指纹（32 字节）‖ 另一边的判法单元指纹（32 字节） | 对拍的结论（调用方给的字节）、两边比下来一致还是不一致（1 字节，[`PairingAgreement`]；不一致的下一趟现判）、判定日期与跑法 |
//!
//! 结论种类 = `01`（镜像判定：GPU 判器与 CPU 判器判出的都是这一种，在哪判的只进块属性）、`02`（观察者），
//! 别的字节读回认不出。版本是 `crash_identity::ConclusionVersion`，取自判法单元指纹
//! （`crash_identity::judging_units_key`；镜像判定那一把是判定定义那一组与 GPU 判器那一组并成的一把，`crash_identity::image_judgement_units_key`）。
//! 一块一种结论记红就是它的违例表里有行：复用只认不红的（记着红的下一趟现判，判完照新结论换上，[`VerdictBlockStoring::ReplacedBecauseOneOfTheTwoIsRed`]；
//! 「记红、撤绿、下趟现判」）。观察者不带名字：是哪一个观察者由版本担（它那一组判法单元按入口 fn 定，同一份文件里两个观察者
//! 两把键；4-2a 报告第四节第 5 条），名字只进组名与行里报的名字。
//! 一块一种结论的结论块、违例、恢复字、计数向量与块属性同一批原子地写、一起读回。库只认这一版的布局：布局行不在的是一份还没放过崩溃判定的库
//! （一次性库的名字空间头一次开），开库时写下这一版的布局号；布局号不是 [`LIBRARY_LAYOUT_NUMBER_OF_THIS_VERSION`] 或读不成布局号的，
//! 开库在读判定向量表之前拒在同一种错误上（[`VerdictStoreError::LibraryLayoutIsNotThisVersion`]），不按版本分别认、不迁移。
//!
//! 照判、不入库的那几把版本（4-2a 报告第四节第 7 条后半句：预检里一部分判法单元拒算时，拒算的那几组照判不入库，别的照键入库同一份库）：
//! [`VerdictStore::keep_the_rows_of_these_versions_out_of_the_library`] 登记的版本，键里版本那一截（判法单元指纹或结论版本，32 字节，
//! 按表标签定的偏移，见上面几张表的键）是它的行一律不写进服务，只记在这个进程里（[`RowsKeptOutOfTheLibrary`]），这个进程里读的时候
//! 与库里的行两路有序流逐行归并着读；进程一退就没了，别的进程、下一趟都看不到、不复用。
//! 写入路径按键分开，判段、核对、观察者、分片进度照常经同一份库读写。

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::PathBuf;
use std::sync::Arc;

use prune_server::client::RouteAddress;
use prune_server::protocol::{ClaimOutcome, Table};
use prune_server::reliable_client::{
    ClientFailure, ReliableClient, ReliableClientSettings, DEFAULT_JOURNAL_FILE_NAME,
    OTHER_CLIENT_FAILURE_EXIT_CODE, PRUNE_SERVER_UNAVAILABLE_EXIT_CODE,
};
use prune_server::segmented_values::{
    is_part_of_segmented_storage, ScannedEntry, SegmentedScanAssembler,
};

use crate::crash::{StateCount1280Bits, STATE_COUNT_BYTES};
use crate::crash_identity::{
    ConclusionKind, ConclusionVersion, JudgingUnitFingerprint, JudgingUnitsKey,
};
use crate::crash_judge_conclusions::AbandonedRootNamedByThePoolChecker;
/// 判定向量、恢复字与计数向量的类型在判器一侧（改结论的几处搬进判器一侧），这里只管它们的字节布局，路径照旧从这里引。
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

/// 判法单元指纹：判这一块的那几个判法单元的键（`crash_identity::judging_units_key`）。判定块与违例按（块键，判法单元指纹）存：
/// 同一块在几把指纹下的判定并存，复用只认同一把；录入表只按块键。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct JudgingUnitsFingerprint(pub [u8; SHA256_DIGEST_BYTES]);

/// 块起点：这一块第一个状态在它那份枚举计划里的序号。块长由调用方定，不单占一格：不分片的块由调用方拼进计划哈希那一格
/// （`crash_amplification::PlannedCrashPoint::block_key`），同一块起点、块长不同的两块不是同一把块键；分片的段里的块块长记在录入表里。
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

/// 一条对拍结论两边比下来一致还是不一致：不一致就是对拍判红（照同一把键记红、撤掉这把键原有的绿，键记着红的下一趟现判、不当绿复用：
/// `.claude/rules/verification.md`「一个门禁：两种跑法」那一条「记红、撤绿、下趟现判」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PairingAgreement {
    BothSidesAgree,
    SidesDisagree,
}

impl PairingAgreement {
    /// 对拍结论那一行里的码。
    fn code(self) -> u8 {
        match self {
            Self::BothSidesAgree => 1,
            Self::SidesDisagree => 2,
        }
    }

    /// 码换回一致不一致；这一版不认的码交回 None（读回的那一行照不合布局报错，不当没判）。
    fn of_code(code: u8) -> Option<Self> {
        match code {
            1 => Some(Self::BothSidesAgree),
            2 => Some(Self::SidesDisagree),
            _ => None,
        }
    }
}

/// 存着的一条对拍结论：调用方给的结论字节（写法由调用方定）、两边比下来一致还是不一致（[`PairingAgreement`]，调用方判了交来），
/// 连同判定日期与跑法。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredPairingConclusion {
    pub conclusion: Vec<u8>,
    pub agreement: PairingAgreement,
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
        bytes.push(self.agreement.code());
        push_length_prefixed_text(&mut bytes, &self.circumstances.judged_date);
        bytes.push(self.circumstances.run_kind.code());
        bytes
    }

    /// [`Self::to_bytes`] 的逆；不合布局交回 None（一致不一致的码、跑法码不是这一版认得的也是不合布局）。
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut cursor = ByteCursor { bytes, position: 0 };
        let conclusion_length = usize::try_from(cursor.big_endian_u32()?).ok()?;
        let conclusion = cursor.take(conclusion_length)?.to_vec();
        let agreement = PairingAgreement::of_code(*cursor.take(1)?.first()?)?;
        let judged_date = cursor.length_prefixed_text()?;
        let run_kind = RunKindOfTheJudgement::of_code(*cursor.take(1)?.first()?)?;
        cursor.is_at_the_end().then_some(Self {
            conclusion,
            agreement,
            circumstances: JudgementCircumstances {
                judged_date,
                run_kind,
            },
        })
    }
}

/// 块属性里的跑法（words 路第八节第 7 项）：剪枝（键相同的复用、对不上的现判）、逐条（点名单跑一项）。两种都照键读库。
/// GPU 还是 CPU 在 [`JudgingDevice`] 那一格。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunKindOfTheJudgement {
    Pruned,
    ItemByItem,
}

/// 入口脚本交跑法的环境变量：`pruned`、`item_by_item`；没设是剪枝。
pub const CRASH_LIBRARY_RUN_KIND_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_CRASH_LIBRARY_RUN_KIND";

impl RunKindOfTheJudgement {
    /// 块属性里的码。
    fn code(self) -> u8 {
        match self {
            Self::Pruned => 2,
            Self::ItemByItem => 3,
        }
    }

    /// 块属性里的码换回跑法；这一版不认的码交回 None（读回的那一行照不合布局报错，不当没判）。
    fn of_code(code: u8) -> Option<Self> {
        match code {
            2 => Some(Self::Pruned),
            3 => Some(Self::ItemByItem),
            _ => None,
        }
    }

    /// 行里报的名字。
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Pruned => "pruned",
            Self::ItemByItem => "item_by_item",
        }
    }

    /// 跑法变量的值（`None` 是没设）换成跑法。
    ///
    /// # Panics
    /// 跑法变量设了而认不得：写明变量名与值，不悄悄当剪枝。
    #[must_use]
    pub fn of_the_variable_value(run_kind: Option<&str>) -> Self {
        match run_kind {
            Some("pruned") => Self::Pruned,
            Some("item_by_item") => Self::ItemByItem,
            Some(other) => panic!(
                "{CRASH_LIBRARY_RUN_KIND_ENVIRONMENT_VARIABLE}={other:?} 认不得：只认 pruned、item_by_item，或不设"
            ),
            None => Self::Pruned,
        }
    }

    /// 这个进程的跑法。
    ///
    /// # Panics
    /// 同 [`Self::of_the_variable_value`]。
    #[must_use]
    pub fn of_this_process() -> Self {
        Self::of_the_variable_value(
            std::env::var(CRASH_LIBRARY_RUN_KIND_ENVIRONMENT_VARIABLE)
                .ok()
                .as_deref(),
        )
    }
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

/// 一个时刻在东京的日期 `YYYY-MM-DD`（公历；1970 年以前的时刻算作 1970-01-01；history-kept: 日期格式的说明与例子）。
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

/// 从 1970-01-01 起的天数换成公历（年，月，日）（history-kept: 日期格式的说明与例子）：按四百年一轮、三月起算的年把天数拆开（闰日落在年末）。
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
    /// 块键在库里的那几个字节（两个摘要加 8 字节大端块起点，各表的键都从它起头）：读库程序按它的十六进制报块。
    #[must_use]
    pub fn to_block_key_bytes(&self) -> Vec<u8> {
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

    /// [`Self::to_block_key_bytes`] 的逆：列表时从键字节认回块键，交回块键与它后面剩下的字节（判法单元指纹、块内序号这类后缀）；
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
            crate::hexadecimal::hexadecimal_text(&self.node_input_digest.0[..8]),
            crate::hexadecimal::hexadecimal_text(&self.enumeration_plan_hash.0[..8]),
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

/// 恢复字旁表里一个状态占几个字节：[`RECOVERY_WORDS_PER_STATE`] 个字，每字 4 字节小端。
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

/// 计数向量值的头一个字节：写法换了就换它，头一个字节不是它的读成不合布局。
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

/// 这一块在哪判的（块属性）：CPU，或 GPU 连同卡型、驱动版本与判它的 GPU 判器那一组判法单元的指纹。
/// 只是出处，不进种类、不进版本（数据不与硬件、不与 CPU 还是 GPU 绑定，复用不看设备）：GPU 判器的指纹留在这里，
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
    /// 块属性表的值：`01` 是 CPU；`02` 之后是卡型与驱动版本（各 4 字节大端长度 + UTF-8）再接 32 字节 GPU 判器指纹。
    /// 别的头一个字节认不回来。观察者交回的镜像上池级 checker 的结论记在哪判的，用同一种写法。
    pub(crate) fn to_attribute_bytes(&self) -> Vec<u8> {
        match self {
            Self::Cpu => vec![0x01],
            Self::Gpu {
                card_type,
                driver_version,
                gpu_judge_units_fingerprint,
            } => {
                let mut bytes = vec![0x02];
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
            [0x02] => Self::Gpu {
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

impl StoredConclusionBlock {
    /// 两份是不是同一个结论：块、违例、计数向量、恢复字、点名的被抛弃根逐项相同（在哪判的、判定日期与跑法是块属性，不比）。
    /// 存块照它比；跨两次拿锁、跨库比结论（复用哨兵的强制现判）照 `crate::crash_amplification::ConclusionSpelledOutInVerdictVectors` 比同一组字段的判定向量版本。
    /// 两处都整份解构、不写 `..`：这个结构体加了字段，两处不各自写明比不比就编不过。
    #[must_use]
    pub fn is_the_same_conclusion_as(&self, other: &Self) -> bool {
        let Self {
            block: this_block,
            violations: this_violations,
            count_vector: this_count_vector,
            // 在哪判的是块属性、只记出处：同一个结论 CPU 判的与 GPU 判的算同一个（复用不看设备，`JudgingDevice` 的文档）。
            judged_on: _,
            recovery_words: this_recovery_words,
            abandoned_roots_named_by_the_pool_checker:
                this_abandoned_roots_named_by_the_pool_checker,
            // 判定日期与跑法是块属性，存的时候由库按那个进程此刻填：同一个结论换一天、换一种跑法判出来照样是同一个。
            circumstances: _,
        } = self;
        let Self {
            block: other_block,
            violations: other_violations,
            count_vector: other_count_vector,
            // 不比的理由见 `self` 那份解构里 `judged_on` 那一行。
            judged_on: _,
            recovery_words: other_recovery_words,
            abandoned_roots_named_by_the_pool_checker:
                other_abandoned_roots_named_by_the_pool_checker,
            // 不比的理由见 `self` 那份解构里 `circumstances` 那一行。
            circumstances: _,
        } = other;
        this_block == other_block
            && this_violations == other_violations
            && this_count_vector == other_count_vector
            && this_recovery_words == other_recovery_words
            && this_abandoned_roots_named_by_the_pool_checker
                == other_abandoned_roots_named_by_the_pool_checker
    }
}

/// 一份结论换掉库里同键那一份时（[`VerdictBlockStoring::ReplacedBecauseOneOfTheTwoIsRed`]），旧的那一份有、新的那一份没有的行放进同一批删掉：
/// 新的没有的违例状态、新的没给而旧的有的恢复字与点名的被抛弃根。结论块、计数向量与块属性随新的那一份照写（[`put_conclusion_rows_into_the_batch`]）。
fn delete_the_rows_the_replacing_conclusion_drops(
    batch: &mut LibraryWriteBatch,
    block_key: &VerdictBlockKey,
    kind: &ConclusionKind,
    conclusion_version: &ConclusionVersion,
    stored: &StoredConclusionBlock,
    replacing: &StoredConclusionBlock,
) {
    for stale in stored.violations.iter().filter(|stored_violation| {
        !replacing
            .violations
            .iter()
            .any(|violation| violation.offset_in_block == stored_violation.offset_in_block)
    }) {
        batch.delete(
            VerdictStoreKey::ConclusionViolation(
                block_key,
                kind,
                conclusion_version,
                stale.offset_in_block,
            )
            .to_key_bytes()
            .0,
        );
    }
    if stored.recovery_words.is_some() && replacing.recovery_words.is_none() {
        batch.delete(
            VerdictStoreKey::RecoveryWords(block_key, kind, conclusion_version)
                .to_key_bytes()
                .0,
        );
    }
    if stored.abandoned_roots_named_by_the_pool_checker.is_some()
        && replacing
            .abandoned_roots_named_by_the_pool_checker
            .is_none()
    {
        batch.delete(
            VerdictStoreKey::AbandonedRootsNamedByThePoolChecker(
                block_key,
                kind,
                conclusion_version,
            )
            .to_key_bytes()
            .0,
        );
    }
}

/// 一个红项写进差异说明的样子：池级 checker 那几项（`pool_checker:<不变量名>`）后面接它在违例位里的下标（位 i ↔
/// `singlefs_checker::image::IMPLEMENTED_INVARIANTS[i]`），别的原样。
fn red_item_naming_its_violated_bit(item: &str) -> String {
    let invariant = item
        .strip_prefix(crate::crash::CRASH_POINT_REPLAY_RED_PASS_POOL_CHECKER)
        .and_then(|rest| rest.strip_prefix(':'));
    match invariant.and_then(|invariant| {
        singlefs_checker::image::IMPLEMENTED_INVARIANTS
            .iter()
            .position(|candidate| *candidate == invariant)
    }) {
        Some(position) => format!("{item}（违例位下标 {position}）"),
        None => item.to_string(),
    }
}

/// 一条不变量名写进差异说明的样子：在池级 checker 的清单里的，后面接它在违例位里的下标。
fn invariant_naming_its_violated_bit(invariant: &str) -> String {
    match singlefs_checker::image::IMPLEMENTED_INVARIANTS
        .iter()
        .position(|candidate| *candidate == invariant)
    {
        Some(position) => format!("{invariant}（违例位下标 {position}）"),
        None => invariant.to_string(),
    }
}

/// 一个状态的判定向量编号认回红项（编号认不回判定向量时写明）。
fn red_items_of_the_number<'vectors>(
    number: VerdictVectorNumber,
    verdict_vector_of_number: &impl Fn(VerdictVectorNumber) -> Option<&'vectors EncodedVerdictVector>,
) -> Result<BTreeSet<String>, String> {
    verdict_vector_of_number(number)
        .map(|vector| {
            crate::crash_judge_conclusions::red_items_of_the_encoded_verdict(vector)
                .into_iter()
                .collect()
        })
        .ok_or_else(|| format!("编号 {}（认不回判定向量）", number.0))
}

/// 同一个（块键，结论种类，版本）下已存的与要存的两份结论差在哪，一处一行（[`VerdictStoreError::DifferentConclusionUnderAStoredKeyNotDecidedInTheFirstVersion`]
/// 带着它）：块内状态数；块内每个判定向量编号不同的状态，两份各自独有的红项（池级 checker 的连同违例位下标）；违例的状态；计数向量里逐条不变量、
/// 逐个观察者、逐次发布的态数不同的那几行；恢复字不同的状态；点名的被抛弃根不同的状态。只读两份结论与判定向量，不进任何判定。
fn differences_between_two_conclusions<'vectors>(
    stored: &StoredConclusionBlock,
    requested: &StoredConclusionBlock,
    verdict_vector_of_number: impl Fn(VerdictVectorNumber) -> Option<&'vectors EncodedVerdictVector>,
) -> Vec<String> {
    let mut differences = Vec::new();
    let stored_numbers = stored.block.verdict_vector_number_of_each_state();
    let requested_numbers = requested.block.verdict_vector_number_of_each_state();
    if stored_numbers.len() != requested_numbers.len() {
        differences.push(format!(
            "块内状态数：已存那一份 {}、要存那一份 {}",
            stored_numbers.len(),
            requested_numbers.len()
        ));
    }
    let describe_items = |items: Vec<&String>| -> String {
        let named: Vec<String> = items
            .into_iter()
            .map(|item| red_item_naming_its_violated_bit(item))
            .collect();
        format!("[{}]", named.join("、"))
    };
    for (state, (stored_number, requested_number)) in
        stored_numbers.iter().zip(requested_numbers).enumerate()
    {
        if stored_number == requested_number {
            continue;
        }
        match (
            red_items_of_the_number(*stored_number, &verdict_vector_of_number),
            red_items_of_the_number(*requested_number, &verdict_vector_of_number),
        ) {
            (Ok(stored_items), Ok(requested_items)) => differences.push(format!(
                "块内第 {state} 个状态的红项：只在已存那一份里的 {}，只在要存那一份里的 {}",
                describe_items(stored_items.difference(&requested_items).collect()),
                describe_items(requested_items.difference(&stored_items).collect())
            )),
            (stored_items, requested_items) => differences.push(format!(
                "块内第 {state} 个状态的判定向量：已存那一份 {stored_items:?}、要存那一份 {requested_items:?}"
            )),
        }
    }
    let violating_states = |conclusion: &StoredConclusionBlock| -> BTreeSet<u64> {
        conclusion
            .violations
            .iter()
            .map(|violation| violation.offset_in_block.0)
            .collect()
    };
    let (stored_violating, requested_violating) =
        (violating_states(stored), violating_states(requested));
    if stored_violating != requested_violating {
        differences.push(format!(
            "违例的状态（块内序号）：只在已存那一份里的 {:?}，只在要存那一份里的 {:?}",
            stored_violating
                .difference(&requested_violating)
                .collect::<Vec<_>>(),
            requested_violating
                .difference(&stored_violating)
                .collect::<Vec<_>>()
        ));
    }
    let invariants: BTreeSet<&String> = stored
        .count_vector
        .per_invariant
        .keys()
        .chain(requested.count_vector.per_invariant.keys())
        .collect();
    for invariant in invariants {
        let stored_counts = stored.count_vector.per_invariant.get(invariant);
        let requested_counts = requested.count_vector.per_invariant.get(invariant);
        if stored_counts != requested_counts {
            differences.push(format!(
                "计数向量里 {}：已存那一份 {stored_counts:?}、要存那一份 {requested_counts:?}",
                invariant_naming_its_violated_bit(invariant)
            ));
        }
    }
    for (what, stored_counts, requested_counts) in [
        (
            "观察者",
            &stored.count_vector.per_observer,
            &requested.count_vector.per_observer,
        ),
        (
            "按发布分的态数",
            &stored.count_vector.states_by_publish,
            &requested.count_vector.states_by_publish,
        ),
    ] {
        let names: BTreeSet<&String> = stored_counts
            .keys()
            .chain(requested_counts.keys())
            .collect();
        for name in names {
            let (stored_count, requested_count) =
                (stored_counts.get(name), requested_counts.get(name));
            if stored_count != requested_count {
                differences.push(format!(
                    "计数向量里{what} {name}：已存那一份 {stored_count:?}、要存那一份 {requested_count:?}"
                ));
            }
        }
    }
    match (&stored.recovery_words, &requested.recovery_words) {
        (Some(stored_words), Some(requested_words)) => {
            for (state, (stored_state_words, requested_state_words)) in
                stored_words.iter().zip(requested_words).enumerate()
            {
                if stored_state_words != requested_state_words {
                    differences.push(format!(
                        "块内第 {state} 个状态的恢复字：已存那一份 {:?}、要存那一份 {:?}",
                        stored_state_words.0, requested_state_words.0
                    ));
                }
            }
            if stored_words.len() != requested_words.len() {
                differences.push(format!(
                    "恢复字的状态数：已存那一份 {}、要存那一份 {}",
                    stored_words.len(),
                    requested_words.len()
                ));
            }
        }
        (stored_words, requested_words) => {
            if stored_words.is_some() != requested_words.is_some() {
                differences.push(format!(
                    "恢复字：已存那一份{}、要存那一份{}",
                    if stored_words.is_some() {
                        "有"
                    } else {
                        "没有"
                    },
                    if requested_words.is_some() {
                        "有"
                    } else {
                        "没有"
                    }
                ));
            }
        }
    }
    match (
        &stored.abandoned_roots_named_by_the_pool_checker,
        &requested.abandoned_roots_named_by_the_pool_checker,
    ) {
        (Some(stored_roots), Some(requested_roots)) => {
            for (state, (stored_state_roots, requested_state_roots)) in
                stored_roots.iter().zip(requested_roots).enumerate()
            {
                if stored_state_roots != requested_state_roots {
                    differences.push(format!(
                        "块内第 {state} 个状态点名的被抛弃根：已存那一份 {stored_state_roots:?}、要存那一份 {requested_state_roots:?}"
                    ));
                }
            }
            if stored_roots.len() != requested_roots.len() {
                differences.push(format!(
                    "点名的被抛弃根的状态数：已存那一份 {}、要存那一份 {}",
                    stored_roots.len(),
                    requested_roots.len()
                ));
            }
        }
        (stored_roots, requested_roots) => {
            if stored_roots.is_some() != requested_roots.is_some() {
                differences.push(format!(
                    "点名的被抛弃根：已存那一份{}、要存那一份{}",
                    if stored_roots.is_some() {
                        "有"
                    } else {
                        "没有"
                    },
                    if requested_roots.is_some() {
                        "有"
                    } else {
                        "没有"
                    }
                ));
            }
        }
    }
    differences
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

/// [`conclusion_block_attribute_bytes`] 的逆；不合布局交回 None（跑法码不是这一版认得的也是不合布局）。
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

/// 结论种类在键里的字节：`01` 镜像判定（不分在哪判的）、`02` 观察者。种类定长一个字节，一个种类的字节不会是另一个的前缀；
/// 观察者的名字不进键（是哪一个观察者由后面那 32 字节的版本担）。别的字节认不回来。
/// 结论种类在键里占的字节数（[`conclusion_kind_key_bytes`] 每一种都是这么长）：照判不入库按它定版本那一截的起点
/// （[`version_field_starts_in_the_key`]）。
const CONCLUSION_KIND_KEY_BYTES: usize = 1;

fn conclusion_kind_key_bytes(kind: &ConclusionKind) -> Vec<u8> {
    match kind {
        ConclusionKind::ImageJudgement => vec![0x01],
        ConclusionKind::Observer => vec![0x02],
    }
}

/// [`conclusion_kind_key_bytes`] 的逆：交回种类与它后面剩下的字节；认不回来交回 None。
fn conclusion_kind_from_key_bytes(bytes: &[u8]) -> Option<(ConclusionKind, &[u8])> {
    let (tag, after_tag) = bytes.split_first()?;
    match tag {
        0x01 => Some((ConclusionKind::ImageJudgement, after_tag)),
        0x02 => Some((ConclusionKind::Observer, after_tag)),
        _ => None,
    }
}

/// 这一版的布局号（布局行的值）：模块文档列的全部表与它们的键值布局。库只认这一个布局号。
const LIBRARY_LAYOUT_NUMBER_OF_THIS_VERSION: u32 = 6;

/// 库里的布局行读出来是什么样（[`VerdictStoreError::LibraryLayoutIsNotThisVersion`] 带着它，报错点名读到了什么）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LibraryLayoutRowAsRead {
    /// 库里没有布局行。
    Missing,
    /// 布局行是 4 字节，读成这个布局号。
    LayoutNumber(u32),
    /// 布局行不是 4 字节、读不成布局号：这是它的原样字节。
    NotFourBytes(Vec<u8>),
}

/// 核读到的布局行：是 [`LIBRARY_LAYOUT_NUMBER_OF_THIS_VERSION`] 才往下开。布局号是别的数、读不成布局号的（开库时布局行不在的已经写下了这一版的）
/// 一律拒在 [`VerdictStoreError::LibraryLayoutIsNotThisVersion`] 上，调用方还没读判定向量表。`library_name` 只进报错。
fn verify_the_library_layout_row(
    layout_value: Option<Vec<u8>>,
    library_name: &str,
) -> Result<(), VerdictStoreError> {
    let layout_row = match layout_value {
        None => LibraryLayoutRowAsRead::Missing,
        Some(layout_bytes) => match <[u8; 4]>::try_from(layout_bytes.as_slice()) {
            Ok(layout_number_bytes) => {
                LibraryLayoutRowAsRead::LayoutNumber(u32::from_be_bytes(layout_number_bytes))
            }
            Err(_not_four_bytes) => LibraryLayoutRowAsRead::NotFourBytes(layout_bytes),
        },
    };
    match layout_row {
        LibraryLayoutRowAsRead::LayoutNumber(LIBRARY_LAYOUT_NUMBER_OF_THIS_VERSION) => Ok(()),
        LibraryLayoutRowAsRead::Missing
        | LibraryLayoutRowAsRead::LayoutNumber(_)
        | LibraryLayoutRowAsRead::NotFourBytes(_) => {
            Err(VerdictStoreError::LibraryLayoutIsNotThisVersion {
                library: library_name.to_string(),
                layout_row,
            })
        }
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
    /// 库布局：一行，值是布局号（[`LIBRARY_LAYOUT_NUMBER_OF_THIS_VERSION`]）。
    LibraryLayout,
    /// 结论块：键是块键 + 结论种类 + 版本，值是每个状态一个判定向量编号。
    ConclusionBlocks,
    /// 结论违例：键是块键 + 结论种类 + 版本 + 块内序号，值是那个状态的判定向量编号。
    ConclusionViolations,
    /// 恢复字旁表：键是块键 + 结论种类 + 版本（与结论块同形：几种结论共用一把版本时各存各的），值是每个状态 [`RECOVERY_WORDS_PER_STATE`] 个字。
    RecoveryWords,
    /// 点名的被抛弃根旁表（「GPU 判器加细节输出」）：键是块键 + 结论种类 + 版本（与恢复字旁表同形），
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
    /// 观察者两段的录入段（「观察者改走两段」）：一条流的崩溃镜像共用的基底（基础镜像与枚举写表），键是它的摘要，
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
    /// 观察者去重的读集见证（「恢复报告分桶 + 读集见证」），一条见证一行：键是桶键（观察者、它另读的输入、恢复报告的摘要）
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
            Self::RecordedFlows => 0x0F,
            Self::RecordedRunManifests => 0x10,
            Self::PackedJudgeTables => 0x11,
            Self::GivenStatesTables => 0x12,
            Self::SlicedSegmentProgress => 0x13,
            Self::ObserverFoundations => 0x14,
            Self::ObservedImages => 0x15,
            Self::ObserverBlocks => 0x16,
            Self::ObservedImageJudgements => 0x17,
            Self::AbandonedRootsNamedByThePoolChecker => 0x18,
            Self::PairingConclusions => 0x19,
            Self::ObserverReadSetWitnessRows => 0x1A,
            Self::ObserverReadSets => 0x1B,
        }
    }

    /// 这张表里服务读回值为空的一行算什么（文件头「删除记号对判定向量两张表不适用」那一段）：判定向量两张表只追加、从不删，空值是一行原样的数据
    /// （按编号那张表里绿那一号的值就是空的）；别的表里空值是删除记号。
    fn what_an_empty_value_means(self) -> EmptyValueMeaning {
        match self {
            Self::VerdictVectorNumberByContent | Self::VerdictVectorContentByNumber => {
                EmptyValueMeaning::AnOrdinaryRow
            }
            Self::VerdictBlocks
            | Self::Violations
            | Self::RecordedBlocks
            | Self::PathIndex
            | Self::FlowFacts
            | Self::VerifierVerdicts
            | Self::JudgingUnitsCompositions
            | Self::LibraryLayout
            | Self::ConclusionBlocks
            | Self::ConclusionViolations
            | Self::RecoveryWords
            | Self::AbandonedRootsNamedByThePoolChecker
            | Self::ConclusionCountVectors
            | Self::ConclusionBlockAttributes
            | Self::RecordedFlows
            | Self::RecordedRunManifests
            | Self::PackedJudgeTables
            | Self::GivenStatesTables
            | Self::SlicedSegmentProgress
            | Self::ObserverFoundations
            | Self::ObservedImages
            | Self::ObserverBlocks
            | Self::ObservedImageJudgements
            | Self::PairingConclusions
            | Self::ObserverReadSetWitnessRows
            | Self::ObserverReadSets => EmptyValueMeaning::DeletionMarker,
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
            0x0F => Some(Self::RecordedFlows),
            0x10 => Some(Self::RecordedRunManifests),
            0x11 => Some(Self::PackedJudgeTables),
            0x12 => Some(Self::GivenStatesTables),
            0x13 => Some(Self::SlicedSegmentProgress),
            0x14 => Some(Self::ObserverFoundations),
            0x15 => Some(Self::ObservedImages),
            0x16 => Some(Self::ObserverBlocks),
            0x17 => Some(Self::ObservedImageJudgements),
            0x18 => Some(Self::AbandonedRootsNamedByThePoolChecker),
            0x19 => Some(Self::PairingConclusions),
            0x1A => Some(Self::ObserverReadSetWitnessRows),
            0x1B => Some(Self::ObserverReadSets),
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

/// 一次存块的结局。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerdictBlockStoring {
    /// 块与它的违例同一批写下去，同步落盘之后才返回。
    WrittenAndSynced,
    /// 这个块键下已经存着逐字节相同的块与相同的违例，什么都没写（续跑时重写同一块走这一格；同一块先后在两张卡上判出同一份结论也走这一格，
    /// 块属性留先存的那一份）。
    AlreadyStoredIdentically,
    /// 这个键下已经存着不同的一份，两份里至少一份是红的（有违例）：照要存的这一份整份换上，旧的那一份的违例、恢复字、点名的被抛弃根
    /// 这一份没有的行同一批删掉（「记红、撤绿、下趟现判」：判红写红记录、撤同一把键的绿；记着红的块下一趟现判，
    /// 判完照新结论写回）。两份都不红而不同的照旧拒（`DifferentConclusionUnderAStoredKeyNotDecidedInTheFirstVersion`）。
    ReplacedBecauseOneOfTheTwoIsRed,
}

/// 同一个（块键，结论种类，版本）下已存的与要存的两份结论差在哪（[`differences_between_two_conclusions`]：块内哪个状态、红项里哪几条连同
/// 池级 checker 的违例位下标与不变量名、违例的状态、计数向量差的那几行、恢复字、点名的被抛弃根，凡不同的都列，一处一行）与两份各在哪判的。
/// 装在盒里随错误走：错误类型不跟着变大。
#[derive(Debug)]
pub struct DifferencesOfTwoConclusionsUnderOneKey {
    pub differences: Vec<String>,
    pub stored_judged_on: JudgingDevice,
    pub requested_judged_on: JudgingDevice,
}

/// 存储出的错，按调用方要做的决定分。
#[derive(Debug)]
pub enum VerdictStoreError {
    /// 库里存的字节不合这个模块的布局：块值是空的或点名了没登记的编号、两张判定向量表对不上、键或值的长度不对。
    StoredBytesDoNotMatchTheLayout {
        table: VerdictStoreTable,
        key_bytes: Vec<u8>,
        reason: &'static str,
    },
    /// 2^32 种判定向量都登记过了、这是再多的一种：编号是 u32（宽度，不再加宽），装不下；返回之前什么都没写。
    EveryU32VerdictVectorNumberIsAlreadyRegistered,
    /// 库不是这一版的布局：布局号不是 [`LIBRARY_LAYOUT_NUMBER_OF_THIS_VERSION`] 或读不成布局号（`layout_row` 是读到的样子；`library` 是库的名字，
    /// 只进报错）。开库拒在这里，判定向量表一行都没读；不迁移。
    LibraryLayoutIsNotThisVersion {
        library: String,
        layout_row: LibraryLayoutRowAsRead,
    },
    /// 这个块键下已经存着不同的块或不同的违例、两份都不红：同一份输入、同一份计划下同一块判出两样，是覆盖、拒绝还是另记，没定，
    /// 第一版不支持；返回之前什么都没写。有一份红的照新的换上（[`VerdictBlockStoring::ReplacedBecauseOneOfTheTwoIsRed`]）。
    DifferentVerdictsUnderAStoredBlockKeyNotDecidedInTheFirstVersion { block_key: VerdictBlockKey },
    /// 这个（块键，结论种类，版本）下已经存着不同的结论（块、违例、计数向量或恢复字不同）、两份都不红：覆盖、拒绝还是另记没定，第一版不支持；
    /// 返回之前什么都没写。存块那一路两份里有一份红的照新的换上（[`VerdictBlockStoring::ReplacedBecauseOneOfTheTwoIsRed`]）。
    /// 结论相同、只有块属性（在哪判的）不同的不是错：留库里先存的那一份，什么都不写（[`VerdictBlockStoring::AlreadyStoredIdentically`]）。
    /// 带着两份差在哪与两份各在哪判的（[`DifferencesOfTwoConclusionsUnderOneKey`]）：两台各判同一块撞上时只读报错就定得出差在哪一样。
    DifferentConclusionUnderAStoredKeyNotDecidedInTheFirstVersion {
        block_key: VerdictBlockKey,
        kind: ConclusionKind,
        differences: Box<DifferencesOfTwoConclusionsUnderOneKey>,
    },
    /// 观察者见证那一批被测试开关拒了（[`VerdictStore::refuse_the_observer_witness_batches_after`]，只有测试打开）：
    /// 当作那一批落盘之前进程就被杀掉，这一批一行都没写。
    ObserverWitnessBatchRefusedByTheTestSwitch,
    /// prune-server 那一侧读写不成（连不上、忙到上限、服务停了、被拒、单个前缀下一行就装不下一帧）：读失败不当成「库里没有」。只在
    /// [`PruneServerFailureHandling::ReturnTheError`] 的跑法交回；产品的跑法（[`PruneServerFailureHandling::ExitTheProcess`]）当场照退出码退出进程。
    PruneServer(PruneServerFailure),
}

/// 这个进程里写进库的同步批的账（[`VerdictStore::synced_write_tally`]）：写了几批、几个字节。字节数是批自己的字节数
/// （[`LibraryWriteBatch::size_in_bytes`]：每行键与值的字节之和，删除记号的值是空的），照判不入库的行也算在里面。
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

type StoredEntry = (Box<[u8]>, Box<[u8]>);

/// 服务读回值为空的一行算什么（[`VerdictStoreTable::what_an_empty_value_means`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EmptyValueMeaning {
    /// 删除记号：读成不在。
    DeletionMarker,
    /// 一行原样的数据：读成在、值是空字节。
    AnOrdinaryRow,
}

/// 服务读回的一行（键已去掉键空间）在不在：值不空的都在；值为空的照键的表标签定（[`VerdictStoreTable::what_an_empty_value_means`]），
/// 空键与认不出表标签的键照删除记号办。
fn stored_row_is_present(key_bytes: &[u8], value_bytes: &[u8]) -> bool {
    if !value_bytes.is_empty() {
        return true;
    }
    match key_bytes
        .first()
        .copied()
        .and_then(VerdictStoreTable::of_key_tag)
        .map(VerdictStoreTable::what_an_empty_value_means)
    {
        Some(EmptyValueMeaning::AnOrdinaryRow) => true,
        Some(EmptyValueMeaning::DeletionMarker) | None => false,
    }
}

/// 值为空的一行许不许写在这把键下（键不带键空间）：只有判定向量按编号那张表许（绿的判定向量是空字节）；按内容那张表的值是 4 字节编号、
/// 从不为空，别的表里空值写下去就成了删除记号。
fn an_empty_value_may_be_put_under_the_key(key_bytes: &[u8]) -> bool {
    key_bytes
        .first()
        .copied()
        .and_then(VerdictStoreTable::of_key_tag)
        == Some(VerdictStoreTable::VerdictVectorContentByNumber)
}

/// 删除记号（空值）写在这把键下读不读成不在（键不带键空间）：判定向量两张表里空值读成一行数据（[`stored_row_is_present`]），删除记号写下去
/// 读回是一行空值（按编号那张表里就是绿的判定向量），不是删；别的表与认不出表标签的键读成不在。
fn deletion_marker_reads_as_absent_under_the_key(key_bytes: &[u8]) -> bool {
    !stored_row_is_present(key_bytes, &[])
}

/// 一批写里的一行：写一个值，或删一个键（删在服务里写成删除记号：空值，读的时候空值当不在，[`PruneServerStorage::read_values`]；
/// 判定向量两张表只追加、从不删，那里的空值是数据，[`stored_row_is_present`]；删那两张表的行写一批时整批拒，[`PruneServerStorage::write_batch`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
enum LibraryWriteOfARow {
    Put { key: Vec<u8>, value: Vec<u8> },
    Delete { key: Vec<u8> },
}

impl LibraryWriteOfARow {
    fn key(&self) -> &[u8] {
        match self {
            Self::Put { key, value: _ } | Self::Delete { key } => key,
        }
    }
}

/// 攒着要一起写进服务的一批行，按次序（同一个键后写的盖先写的，与服务按帧次序落盘同义）。一块一种结论的行在一批里，写进服务时落在一帧里
/// （[`PruneServerStorage::write_rows`]：一帧装不下才切；满块约 3.4 MB，远小于一帧），原子。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct LibraryWriteBatch {
    rows: Vec<LibraryWriteOfARow>,
}

impl LibraryWriteBatch {
    fn put(&mut self, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) {
        self.rows.push(LibraryWriteOfARow::Put {
            key: key.as_ref().to_vec(),
            value: value.as_ref().to_vec(),
        });
    }

    fn delete(&mut self, key: impl AsRef<[u8]>) {
        self.rows.push(LibraryWriteOfARow::Delete {
            key: key.as_ref().to_vec(),
        });
    }

    fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// 这一批的字节数：每行键与值的字节之和（删的那一行值是空的）。
    fn size_in_bytes(&self) -> usize {
        self.rows
            .iter()
            .map(|row| match row {
                LibraryWriteOfARow::Put { key, value } => key.len() + value.len(),
                LibraryWriteOfARow::Delete { key } => key.len(),
            })
            .sum()
    }
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

/// 一块一种结论在库里的全部行（结论块、违例、计数向量、块属性，给了恢复字的连同恢复字）放进一批。存结论块（[`VerdictStore::put_a_conclusion_block_into_the_batch`]）经它，行的写法只在这一处。
fn put_conclusion_rows_into_the_batch(
    batch: &mut LibraryWriteBatch,
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

/// 一批写拆成写进库的与照判不入库的（[`RowsKeptOutOfTheLibrary::keeps_out`]），各自保持原来的次序。
struct BatchSplitByTheKeptOutVersions {
    into_the_library: LibraryWriteBatch,
    kept_out: LibraryWriteBatch,
}

impl RowsKeptOutOfTheLibrary {
    /// 把一批按键拆开：键里带着照判不入库的版本的归 `kept_out`，别的归 `into_the_library`。
    fn split_the_batch(&self, batch: LibraryWriteBatch) -> BatchSplitByTheKeptOutVersions {
        let mut split = BatchSplitByTheKeptOutVersions {
            into_the_library: LibraryWriteBatch::default(),
            kept_out: LibraryWriteBatch::default(),
        };
        for row in batch.rows {
            let destination = if self.keeps_out(row.key()) {
                &mut split.kept_out
            } else {
                &mut split.into_the_library
            };
            destination.rows.push(row);
        }
        split
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
/// （等同于先放库里的、再用照判不入库的盖的合并）。库那一路交出 `Err` 之后这一路就到此为止，不再交任何一行。
///
/// 错误类型写死成 [`VerdictStoreError`]、不带泛型参数：单元图把自身类型里带 impl 泛型参数的 impl 当 blanket impl 放行（`crates/singlefs-unit-graph`
/// 的 `units.rs` 里 blanket 的定义），这个迭代器一带泛型，判器闭包里任何一处 `.next()` 都会把它连同判定库的一串单元拉进判器的键
/// （`crash_identity` 的 `the_judge_groups_reach_no_pipeline_unit_and_the_cpu_groups_no_card_unit` 盯着）。
struct RowsOfTwoSourcesMergedByKey<'rows> {
    library_rows: std::iter::Peekable<LibraryRowsToMerge<'rows>>,
    kept_out_rows: std::iter::Peekable<KeptOutRowsToMerge<'rows>>,
    is_ended_by_a_library_error: bool,
}

/// 归并的库那一路：按键升序的库行，读错时交 `Err`（prune-server 那一侧读不成）。
type LibraryRowsToMerge<'rows> =
    Box<dyn Iterator<Item = Result<StoredEntry, VerdictStoreError>> + 'rows>;

/// 按块键排的表逐行交给它的那一个访问者（`VerdictStore::visit_rows_of_a_block_table`）：块键、块键之后的键后缀、值；交回错就停扫。
type BlockTableRowVisitor<'visitor> =
    dyn FnMut(VerdictBlockKey, &[u8], &[u8]) -> Result<(), VerdictStoreError> + 'visitor;

/// 照判不入库的一行：键与值。
type KeptOutRow = (Vec<u8>, Vec<u8>);

/// 归并的照判不入库那一路：按键升序的行。
type KeptOutRowsToMerge<'rows> = Box<dyn Iterator<Item = KeptOutRow> + 'rows>;

impl<'rows> RowsOfTwoSourcesMergedByKey<'rows> {
    fn new(
        library_rows: impl Iterator<Item = Result<StoredEntry, VerdictStoreError>> + 'rows,
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

    fn next_kept_out_row(&mut self) -> Option<Result<StoredEntry, VerdictStoreError>> {
        self.kept_out_rows.next().map(|(key_bytes, value_bytes)| {
            Ok((key_bytes.into_boxed_slice(), value_bytes.into_boxed_slice()))
        })
    }
}

impl Iterator for RowsOfTwoSourcesMergedByKey<'_> {
    type Item = Result<StoredEntry, VerdictStoreError>;

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

/// 一次性库的名字空间的头一个字节（[`LibraryKeySpace::DisposableLibrary`]）：不是崩溃判定库的表标签（`0x00`–`0x1B`），不是 Python 三家族名的
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
    /// 接在每一把键前面的那一段：全仓那一份是空的。
    pub(crate) fn key_prefix(&self) -> &[u8] {
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
const PRUNE_SERVER_NUMBER_CLAIM_RETRY_INTERVAL: std::time::Duration =
    std::time::Duration::from_millis(20);

/// 编号被别的客户端认领着时最多等多久：持有者活着几毫秒就写完，死了一个租期之后放出来；两个租期还认不到当服务不可用。
const PRUNE_SERVER_NUMBER_CLAIM_WAIT_LIMIT: std::time::Duration =
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
    route: RouteAddress,
    failure_handling: PruneServerFailureHandling,
    key_space: LibraryKeySpace,
    reliable_client: std::sync::Mutex<ReliableClient>,
    /// 前缀扫一帧最多回几条：平常是 [`PRUNE_SERVER_SCAN_ENTRY_LIMIT`]，只有单测改小它（[`Self::scan_at_most_entries_per_frame_for_a_test`]），
    /// 在几行的库上也走「截断就拆前缀」那一支。
    scan_entry_limit: std::sync::atomic::AtomicU32,
    /// 容错客户端追加日志所在的一次性目录（[`Self::connect_with_a_journal_of_its_own`] 建的；随这一端放手删掉）；调用方自己给追加日志路径的是 `None`。
    _journal_directory: Option<crate::crash_library::ExclusiveTemporaryDirectory>,
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
    fn failed(&self, exit_code: i32, description: String) -> VerdictStoreError {
        prune_server_failed(self.failure_handling, exit_code, description)
    }

    /// 库里的键接上这一份的键空间（[`LibraryKeySpace::key_prefix`]）。
    fn namespaced_key(&self, key_bytes: &[u8]) -> Vec<u8> {
        let key_prefix = self.key_space.key_prefix();
        let mut namespaced = Vec::with_capacity(key_prefix.len() + key_bytes.len());
        namespaced.extend_from_slice(key_prefix);
        namespaced.extend_from_slice(key_bytes);
        namespaced
    }

    fn lock_the_reliable_client(&self) -> std::sync::MutexGuard<'_, ReliableClient> {
        self.reliable_client
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// 成批点读，交回的次序与 `keys` 相同；分段存的拼回整值（文件头「prune-server 那一侧」）；删除记号（空值）读成不在，判定向量两张表里的空值
    /// 原样读成在（[`stored_row_is_present`]）。每 [`PRUNE_SERVER_READ_KEYS_PER_FRAME`] 个键一帧。
    ///
    /// # Errors
    /// 读不成（连不上、忙到上限、服务停了、被拒；一个键的值就越过帧长上限；分段存的拼不回），或回的个数与问的不同。
    fn read_values(&self, keys: &[&[u8]]) -> Result<Vec<Option<Vec<u8>>>, VerdictStoreError> {
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
    fn rows_with_prefix(&self, prefix: &[u8]) -> Result<Vec<StoredEntry>, VerdictStoreError> {
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
    fn visit_rows_with_prefix(
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
    fn has_a_row_with_prefix(&self, prefix: &[u8]) -> Result<bool, VerdictStoreError> {
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
    fn write_batch(&self, batch: LibraryWriteBatch) -> Result<(), VerdictStoreError> {
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
    fn write_rows(&self, rows: Vec<(Vec<u8>, Vec<u8>)>) -> Result<(), VerdictStoreError> {
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

    /// 认领一个键（租期 [`PRUNE_SERVER_NUMBER_CLAIM_LEASE_MILLISECONDS`]），交回服务答的结局。
    ///
    /// # Errors
    /// 认领不成，或回的结局不是一个。
    fn claim(&self, key_bytes: &[u8]) -> Result<ClaimOutcome, VerdictStoreError> {
        let claimed = self.lock_the_reliable_client().claim(
            &[(PRUNE_SERVER_TABLE, self.namespaced_key(key_bytes))],
            PRUNE_SERVER_NUMBER_CLAIM_LEASE_MILLISECONDS,
        );
        let outcomes = claimed.map_err(|failure| {
            prune_server_client_failed(self.failure_handling, "认领判定向量编号", &failure)
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

/// prune-server 里那一份的判定向量表：按编号那一张（键是编号的大端 4 字节，按键升序就是按编号升序）要从 0 起连续、一种一个编号；
/// 按内容那一张由它现算（按内容那一张只给登记时查重用，开库时不整张读它）。整张按编号表一次前缀扫读回（被截断就拆前缀分着扫）；
/// 绿那一号的值为空，照一行原样算（文件头「删除记号对判定向量两张表不适用」那一段）。
///
/// # Errors
/// 扫不成；编号不连续、不是 4 字节，或同一种判定向量有两个编号。
fn load_verdict_vectors_of_the_prune_server(
    storage: &PruneServerStorage,
) -> Result<LoadedVerdictVectors, VerdictStoreError> {
    let by_number_table = VerdictStoreTable::VerdictVectorContentByNumber;
    let mut verdict_vector_by_number = Vec::new();
    let mut verdict_vector_number_by_content = HashMap::new();
    let rows = storage.rows_with_prefix(
        &VerdictStoreKey::WholeTablePrefix(by_number_table)
            .to_key_bytes()
            .0,
    )?;
    for (key_bytes, vector_bytes) in rows {
        let expected =
            VerdictVectorNumber::next_after_registered_count(verdict_vector_by_number.len());
        let is_the_next_number = key_bytes
            .get(KEY_TAG_BYTES..)
            .and_then(stored_verdict_vector_number)
            .is_some_and(|stored_number| expected == Some(VerdictVectorNumber(stored_number)));
        let Some(number) = expected.filter(|_number| is_the_next_number) else {
            return Err(stored_bytes_mismatch(
                by_number_table,
                &key_bytes,
                "prune-server 里按编号的判定向量表编号不是从 0 起连续的 4 字节",
            ));
        };
        if verdict_vector_number_by_content
            .insert(vector_bytes.to_vec(), number)
            .is_some()
        {
            return Err(stored_bytes_mismatch(
                by_number_table,
                &key_bytes,
                "prune-server 里同一种判定向量登记了两个编号",
            ));
        }
        verdict_vector_by_number.push(EncodedVerdictVector(vector_bytes.to_vec()));
    }
    Ok(LoadedVerdictVectors {
        verdict_vector_by_number,
        verdict_vector_number_by_content,
    })
}

/// 一个连着 prune-server 的判定库（[`Self::open_the_prune_server`]）。进程里改库的方法都要 `&mut self`，同一个进程里「先读后写」的两步之间
/// 没有别的写者；别的进程、另一台同时写同一份库：存块先读后写之间被别人写了同一把键的，后写的那一帧盖先写的（同一把键下判出的结论只由内容定，
/// 两个写者写的是同一份；记红、撤绿的规矩照旧由各自先读到的那一份定）。
pub struct VerdictStore {
    /// 这一份库在服务里的那一端（点读、写、认领、前缀扫都经它；判定向量编号也由它统一给）。
    service: Arc<PruneServerStorage>,
    /// 库的名字（[`PruneServerStorage::library_name`]），只进报错与行里报的名字。
    library_name: String,
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
    /// 这个进程开库以来交来存的每一块一种结论（[`Self::store_conclusion_block`] 一族：写下的、已存相同的、换上的都记），按交来的次序。
    /// 复用哨兵判「抽中的块这一趟被现判了没有」只看它（[`Self::conclusion_blocks_offered_since`]），不看块属性：现判出同一份结论时库里一个字节都不动。
    conclusion_blocks_offered_in_this_process: Vec<ConclusionOfABlockUnderAKey>,
    /// 复用哨兵抽中强制现判的那几块：这个进程里问它复用不复用得上一律答不上（[`Self::conclusion_block_is_forced_to_be_judged_again`]），
    /// 判段照现判、判完照存块的规矩写回；每一趟抽一次、整份换掉（[`Self::force_these_conclusion_blocks_to_be_judged_again`]）。
    conclusion_blocks_forced_to_be_judged_again: BTreeSet<ConclusionOfABlockUnderAKey>,
    /// 开库之后在只拿 `&self` 的读里从 prune-server 取来的判定向量（编号从按编号的内存表末尾起连续）：读的方法交回判定向量的借用，
    /// 取来的这几种放进 `Box::leak` 的堆里、进程退出才放（一个进程见到的判定向量种数有限，几百上千种、每种几十字节）；
    /// 下一次拿 `&mut self` 时（登记）并进内存表（[`Self::take_in_the_verdict_vectors_fetched_after_opening`]）。
    verdict_vectors_fetched_after_opening:
        std::sync::Mutex<BTreeMap<u32, &'static EncodedVerdictVector>>,
}

/// 一块一种结论在库里的键：块键、结论种类、版本。
pub type ConclusionOfABlockUnderAKey = (VerdictBlockKey, ConclusionKind, ConclusionVersion);

impl VerdictStore {
    /// 核过布局的服务那一端连同读回的判定向量表装成一份：这个进程里的账都从空起。
    fn assembled_over_the_service(
        service: Arc<PruneServerStorage>,
        loaded: LoadedVerdictVectors,
    ) -> Self {
        let LoadedVerdictVectors {
            verdict_vector_by_number,
            verdict_vector_number_by_content,
        } = loaded;
        Self {
            library_name: service.library_name(),
            service,
            verdict_vector_by_number,
            verdict_vector_number_by_content,
            rows_kept_out_of_the_library: std::sync::Mutex::new(RowsKeptOutOfTheLibrary::default()),
            synced_batches_written: std::sync::atomic::AtomicU64::new(0),
            synced_bytes_written: std::sync::atomic::AtomicU64::new(0),
            observer_witness_batches_still_written: std::sync::atomic::AtomicU64::new(
                OBSERVER_WITNESS_BATCHES_ARE_NEVER_REFUSED,
            ),
            conclusion_blocks_offered_in_this_process: Vec::new(),
            conclusion_blocks_forced_to_be_judged_again: BTreeSet::new(),
            verdict_vectors_fetched_after_opening: std::sync::Mutex::new(BTreeMap::new()),
        }
    }

    /// 按编号登记过的全部判定向量（位置就是编号）：开库那一刻服务里登记过的，连同这个进程之后登记、取来并进内存表的。
    #[must_use]
    pub fn registered_verdict_vectors(&self) -> &[EncodedVerdictVector] {
        &self.verdict_vector_by_number
    }

    /// 这个进程开库以来交来存过几块结论（复用哨兵开跑前记下这个数，跑完照它取这一趟交来的那几块：[`Self::conclusion_blocks_offered_since`]）。
    #[must_use]
    pub fn conclusion_blocks_offered_so_far(&self) -> usize {
        self.conclusion_blocks_offered_in_this_process.len()
    }

    /// 第 `mark` 块（[`Self::conclusion_blocks_offered_so_far`] 记下的）之后交来存的每一块一种结论，按交来的次序；`mark` 越过末尾是空的。
    #[must_use]
    pub fn conclusion_blocks_offered_since(&self, mark: usize) -> &[ConclusionOfABlockUnderAKey] {
        self.conclusion_blocks_offered_in_this_process
            .get(mark..)
            .unwrap_or_default()
    }

    /// 复用哨兵这一趟抽中强制现判的那几块：整份换掉上一趟的（给空的就是一块都不强制）。
    pub fn force_these_conclusion_blocks_to_be_judged_again(
        &mut self,
        blocks: impl IntoIterator<Item = ConclusionOfABlockUnderAKey>,
    ) {
        self.conclusion_blocks_forced_to_be_judged_again = blocks.into_iter().collect();
    }

    /// 这一块这一种结论这一把版本是不是这一趟抽中强制现判的（[`Self::force_these_conclusion_blocks_to_be_judged_again`]）。
    #[must_use]
    pub fn conclusion_block_is_forced_to_be_judged_again(
        &self,
        block_key: &VerdictBlockKey,
        kind: &ConclusionKind,
        conclusion_version: &ConclusionVersion,
    ) -> bool {
        self.conclusion_blocks_forced_to_be_judged_again.contains(&(
            block_key.clone(),
            kind.clone(),
            *conclusion_version,
        ))
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

    /// 登记照判、不入库的版本（判法单元拒算那几组的占位版本）：此后键里带着其中一把的行不写进服务，只留在这个进程里，读的时候合起来读。
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

    /// 写一批：键里带着照判不入库的版本的行只记在这个进程里，别的写进服务（一批里两边各自原子；写进服务那一半一帧装得下就一帧）。
    fn write_batch_keeping_the_kept_out_rows_in_this_process(
        &self,
        batch: LibraryWriteBatch,
    ) -> Result<(), VerdictStoreError> {
        let batch_bytes = u64::try_from(batch.size_in_bytes()).expect("一批的字节数装得进 u64");
        self.write_batch_into_the_library_or_kept_out(batch)?;
        self.synced_batches_written
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.synced_bytes_written
            .fetch_add(batch_bytes, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    /// 拆开、写服务、记下照判不入库的行。
    fn write_batch_into_the_library_or_kept_out(
        &self,
        batch: LibraryWriteBatch,
    ) -> Result<(), VerdictStoreError> {
        let mut kept_out = self
            .rows_kept_out_of_the_library
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if kept_out.versions.is_empty() {
            drop(kept_out);
            return self.service.write_batch(batch);
        }
        let split = kept_out.split_the_batch(batch);
        if !split.into_the_library.is_empty() {
            self.service.write_batch(split.into_the_library)?;
        }
        for row in split.kept_out.rows {
            match row {
                LibraryWriteOfARow::Put { key, value } => {
                    kept_out.rows.insert(key, value);
                }
                LibraryWriteOfARow::Delete { key } => {
                    kept_out.rows.remove(&key);
                }
            }
        }
        Ok(())
    }

    /// 读一行：照判不入库的那几行先看这个进程里的，别的读服务（读不成交错误，不当成不在）。
    fn read_value_kept_in_or_out(
        &self,
        key_bytes: &[u8],
    ) -> Result<Option<Vec<u8>>, VerdictStoreError> {
        let kept_out = self
            .rows_kept_out_of_the_library
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if kept_out.keeps_out(key_bytes) {
            return Ok(kept_out.rows.get(key_bytes).cloned());
        }
        drop(kept_out);
        Ok(self.service.read_values(&[key_bytes])?.pop().flatten())
    }

    /// 键以 `prefix` 开头的全部条目，按键的次序：服务里的（一次前缀扫攒成一份，被截断就拆前缀分着扫）与这个进程里照判不入库的两路有序流逐行归并
    /// （[`RowsOfTwoSourcesMergedByKey`]），同一个键两路都有时取照判不入库那一行；没有登记版本时就是服务里的。扫不成时交出那个错、到此为止
    /// （调用方见错即停）。只给小前缀（一块、一个桶、一把指纹下的行）；整表扫用 [`Self::visit_entries_with_key_prefix_kept_in_or_out`]。
    fn entries_with_key_prefix_kept_in_or_out(
        &self,
        prefix: VerdictStoreKeyBytes,
    ) -> Box<dyn Iterator<Item = Result<StoredEntry, VerdictStoreError>> + '_> {
        let rows_of_the_library: LibraryRowsToMerge<'_> =
            match self.service.rows_with_prefix(&prefix.0) {
                Ok(rows) => Box::new(rows.into_iter().map(Ok::<StoredEntry, VerdictStoreError>)),
                Err(error) => Box::new(std::iter::once(Err::<StoredEntry, VerdictStoreError>(
                    error,
                ))),
            };
        let no_version_is_kept_out = self
            .rows_kept_out_of_the_library
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .versions
            .is_empty();
        if no_version_is_kept_out {
            return rows_of_the_library;
        }
        let kept_out_rows = KeptOutRowsWithThePrefix {
            rows_kept_out_of_the_library: &self.rows_kept_out_of_the_library,
            prefix: prefix.0.clone(),
            last_key_taken: None,
        };
        Box::new(RowsOfTwoSourcesMergedByKey::new(
            rows_of_the_library,
            kept_out_rows,
        ))
    }

    /// 整表（或大前缀）扫：键以 `prefix` 开头的全部条目按键的次序逐行交给 `visit_row`，服务那一路逐帧扫、逐行交（[`PruneServerStorage::visit_rows_with_prefix`]：
    /// 被截断就拆前缀分着扫），不整段攥在内存里；照判不入库的行按键插在它们该在的位置，同一个键两路都有时交照判不入库那一行。
    ///
    /// # Errors
    /// 扫不成；`visit_row` 的错。
    fn visit_entries_with_key_prefix_kept_in_or_out(
        &self,
        prefix: VerdictStoreKeyBytes,
        visit_row: &mut dyn FnMut(StoredEntry) -> Result<(), VerdictStoreError>,
    ) -> Result<(), VerdictStoreError> {
        let mut kept_out_rows = KeptOutRowsWithThePrefix {
            rows_kept_out_of_the_library: &self.rows_kept_out_of_the_library,
            prefix: prefix.0.clone(),
            last_key_taken: None,
        }
        .peekable();
        self.service
            .visit_rows_with_prefix(&prefix.0, &mut |library_row| {
                // 迭代上界：照判不入库那一路这个前缀下的行数；每次交出一行
                while let Some((kept_out_key, _value_bytes)) = kept_out_rows.peek() {
                    if kept_out_key.as_slice() > library_row.0.as_ref() {
                        break;
                    }
                    let is_the_same_key = kept_out_key.as_slice() == library_row.0.as_ref();
                    let (key_bytes, value_bytes) =
                        kept_out_rows.next().expect("上面刚看过这一路还有下一行");
                    visit_row((key_bytes.into_boxed_slice(), value_bytes.into_boxed_slice()))?;
                    if is_the_same_key {
                        return Ok(());
                    }
                }
                visit_row(library_row)
            })?;
        for (key_bytes, value_bytes) in kept_out_rows {
            visit_row((key_bytes.into_boxed_slice(), value_bytes.into_boxed_slice()))?;
        }
        Ok(())
    }

    /// 键以 `prefix` 开头的行有没有一行（照判不入库的连同服务里的）：服务那一路只问一帧、最多回一行（[`PruneServerStorage::has_a_row_with_prefix`]），
    /// 不把前缀下的行读回来。
    ///
    /// # Errors
    /// 服务读不成。
    fn has_an_entry_with_key_prefix_kept_in_or_out(
        &self,
        prefix: VerdictStoreKeyBytes,
    ) -> Result<bool, VerdictStoreError> {
        let has_a_kept_out_row = KeptOutRowsWithThePrefix {
            rows_kept_out_of_the_library: &self.rows_kept_out_of_the_library,
            prefix: prefix.0.clone(),
            last_key_taken: None,
        }
        .next()
        .is_some();
        if has_a_kept_out_row {
            return Ok(true);
        }
        self.service.has_a_row_with_prefix(&prefix.0)
    }

    /// 给一种判定向量一个编号：登记过的返回原来的编号、什么都不写；没登记过的在服务里认领下一个编号、两张表同一帧写下去之后才返回
    /// （[`Self::register_verdict_vector_in_the_prune_server`]：别的进程同时登记同一种，只得一个编号）。
    ///
    /// # Errors
    /// 2^32 种都登记过了（`EveryU32VerdictVectorNumberIsAlreadyRegistered`，什么都没写）；服务读写、认领不成。
    pub fn register_verdict_vector(
        &mut self,
        verdict_vector: &EncodedVerdictVector,
    ) -> Result<VerdictVectorNumber, VerdictStoreError> {
        self.take_in_the_verdict_vectors_fetched_after_opening()?;
        if let Some(number) = self.verdict_vector_number_by_content.get(&verdict_vector.0) {
            return Ok(*number);
        }
        let service = Arc::clone(&self.service);
        self.register_verdict_vector_in_the_prune_server(&service, verdict_vector)
    }

    /// 这个编号登记的是哪种判定向量；这个库里没有这个编号时是 `None`。
    #[must_use]
    pub fn verdict_vector_of_number(
        &self,
        number: VerdictVectorNumber,
    ) -> Option<&EncodedVerdictVector> {
        if let Some(verdict_vector) = self
            .verdict_vector_by_number
            .get(number.position_in_the_registration_order())
        {
            return Some(verdict_vector);
        }
        self.lock_the_verdict_vectors_fetched_after_opening()
            .get(&number.as_u32())
            .copied()
    }

    /// 存一块在这一判法单元指纹下的判定与它的违例：块与每个违例状态同一批同步写下去之后才返回；违例状态的判定向量编号取块里那个状态的。
    /// 同一块键在别的判法单元指纹下的行不受影响。这一版本下已经存着不同的一份、两份里有一份红的：照这一份换上
    /// （[`VerdictBlockStoring::ReplacedBecauseOneOfTheTwoIsRed`]，旧的违例行同一批删掉）。
    ///
    /// # Errors
    /// 这个块键在这一版本下已经存着不同的块或不同的违例、两份都不红（`DifferentVerdictsUnderAStoredBlockKeyNotDecidedInTheFirstVersion`，
    /// 什么都没写）；已存的违例读回来不合布局；服务读写不成。
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
        let registered_count = self.known_verdict_vector_count();
        assert!(
            block
                .verdict_vector_number_of_each_state()
                .iter()
                .all(|number| number.position_in_the_registration_order() < registered_count),
            "块里的判定向量编号必须是这个库登记过的（这个库登记了 {registered_count} 种）"
        );
        let mut batch = LibraryWriteBatch::default();
        let storing = self.put_a_verdict_block_into_the_batch(
            block_key,
            version,
            block,
            violating_states,
            &mut batch,
        )?;
        match storing {
            VerdictBlockStoring::AlreadyStoredIdentically => return Ok(storing),
            VerdictBlockStoring::WrittenAndSynced
            | VerdictBlockStoring::ReplacedBecauseOneOfTheTwoIsRed => {}
        }
        self.write_batch_keeping_the_kept_out_rows_in_this_process(batch)?;
        Ok(storing)
    }

    /// 一块判定连同违例放进 `batch`（[`Self::store_verdict_block`] 与导判定都经它，规矩只在这一处）：这一版本下已经存着逐字节相同的块与相同的违例的，
    /// 什么都不放、交回 [`VerdictBlockStoring::AlreadyStoredIdentically`]；不同而两份里有一份红的，照这一份换上、旧的有而这一份没有的违例行同一批删掉
    /// （[`VerdictBlockStoring::ReplacedBecauseOneOfTheTwoIsRed`]）；没存过的交回 [`VerdictBlockStoring::WrittenAndSynced`]。交回的是这一批写下去之后的结局，
    /// 写不写由调用方定。块里的编号可以是导判定攒在这一批里、还没写下去的（不核登记过没有，存块那一路调用方先核）。
    ///
    /// # Errors
    /// 两份都不红而不同（`DifferentVerdictsUnderAStoredBlockKeyNotDecidedInTheFirstVersion`，`batch` 里没放这一块的行）；已存的违例读回来不合布局；
    /// 服务读不成。
    ///
    /// # Panics
    /// 违例状态的块内序号不在块里：调用方的 bug。
    fn put_a_verdict_block_into_the_batch(
        &self,
        block_key: &VerdictBlockKey,
        version: &JudgingUnitsFingerprint,
        block: &VerdictBlock,
        violating_states: &[StateOffsetInBlock],
        batch: &mut LibraryWriteBatch,
    ) -> Result<VerdictBlockStoring, VerdictStoreError> {
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
        let stored_block_value = self.read_value_kept_in_or_out(&block_key_bytes.0)?;
        let mut storing = VerdictBlockStoring::WrittenAndSynced;
        if let Some(stored_block_value_bytes) = stored_block_value {
            let stored_violations = self.violations_in_block(block_key, version)?;
            let is_identical = stored_block_value_bytes == block_value_bytes
                && stored_violations == requested_violations;
            if is_identical {
                return Ok(VerdictBlockStoring::AlreadyStoredIdentically);
            }
            // 记红、撤绿：两份里有一份红的，照要存的这一份换上；旧的那一份有、这一份没有的违例行同一批删掉
            let one_of_the_two_is_red =
                !stored_violations.is_empty() || !requested_violations.is_empty();
            if !one_of_the_two_is_red {
                return Err(
                    VerdictStoreError::DifferentVerdictsUnderAStoredBlockKeyNotDecidedInTheFirstVersion {
                        block_key: block_key.clone(),
                    },
                );
            }
            for stale in stored_violations.iter().filter(|stored| {
                !requested_violations
                    .iter()
                    .any(|requested| requested.offset_in_block == stored.offset_in_block)
            }) {
                batch.delete(
                    VerdictStoreKey::Violation(block_key, version, stale.offset_in_block)
                        .to_key_bytes()
                        .0,
                );
            }
            storing = VerdictBlockStoring::ReplacedBecauseOneOfTheTwoIsRed;
        }
        batch.put(&block_key_bytes.0, &block_value_bytes);
        for violation in &requested_violations {
            batch.put(
                VerdictStoreKey::Violation(block_key, version, violation.offset_in_block)
                    .to_key_bytes()
                    .0,
                violation.verdict_vector_number.to_stored_bytes(),
            );
        }
        Ok(storing)
    }

    fn registered_verdict_vector_number(
        &self,
        table: VerdictStoreTable,
        key_bytes: &[u8],
        stored_number: u32,
    ) -> Result<VerdictVectorNumber, VerdictStoreError> {
        self.fetch_the_verdict_vectors_of_unknown_numbers(&[stored_number])?;
        let number = VerdictVectorNumber(stored_number);
        if number.position_in_the_registration_order() < self.known_verdict_vector_count() {
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
        self.fetch_the_verdict_vectors_of_unknown_numbers(&stored_numbers)?;
        let known_count = self.known_verdict_vector_count();
        stored_numbers
            .into_iter()
            .map(|stored_number| {
                let number = VerdictVectorNumber(stored_number);
                if number.position_in_the_registration_order() < known_count {
                    Ok(number)
                } else {
                    Err(stored_bytes_mismatch(
                        table,
                        key_bytes,
                        "存着的判定向量编号在这个库里没登记过",
                    ))
                }
            })
            .collect()
    }

    /// 读回一块在这一判法单元指纹下的判定；这个块键在这一版本下没存过时是 `None`（别的版本存过也是 `None`：复用只认同一版本）。
    ///
    /// # Errors
    /// 存着的块值是空的或点名了没登记的编号；服务读不成。
    pub fn read_verdict_block(
        &self,
        block_key: &VerdictBlockKey,
        version: &JudgingUnitsFingerprint,
    ) -> Result<Option<VerdictBlock>, VerdictStoreError> {
        let block_key_bytes = VerdictStoreKey::VerdictBlock(block_key, version).to_key_bytes();
        let Some(block_value_bytes) = self.read_value_kept_in_or_out(&block_key_bytes.0)? else {
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
    /// 违例表里的键或值长度不对、点名了没登记的编号；服务读不成。
    pub fn violations_in_block(
        &self,
        block_key: &VerdictBlockKey,
        version: &JudgingUnitsFingerprint,
    ) -> Result<Vec<StoredViolation>, VerdictStoreError> {
        let prefix = VerdictStoreKey::ViolationsOfBlockPrefix(block_key, version).to_key_bytes();
        let prefix_length = prefix.0.len();
        let mut violations = Vec::new();
        for entry in self.entries_with_key_prefix_kept_in_or_out(prefix) {
            let (key_bytes, number_bytes) = entry?;
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
    /// 服务读写不成；同一个块键已录过、状态数不同。
    pub fn record_block(
        &mut self,
        block_key: &VerdictBlockKey,
        state_count: u64,
    ) -> Result<(), VerdictStoreError> {
        let key_bytes = VerdictStoreKey::RecordedBlock(block_key).to_key_bytes();
        if let Some(stored) = self.read_value_kept_in_or_out(&key_bytes.0)? {
            if stored == state_count.to_be_bytes() {
                return Ok(());
            }
            return Err(VerdictStoreError::DifferentVerdictsUnderAStoredBlockKeyNotDecidedInTheFirstVersion { block_key: block_key.clone() });
        }
        let mut batch = LibraryWriteBatch::default();
        batch.put(&key_bytes.0, state_count.to_be_bytes());
        self.write_batch_keeping_the_kept_out_rows_in_this_process(batch)
    }

    /// 录入分片的段的一块（C790）：同 [`Self::record_block`]，只是录过、状态数不同、而这一块在判定块表、结论块表、核对表里一行都没有
    /// （上一趟录了这一块、没判完就停了，两趟之间片大小改了，这一块的长度跟着变）时照这一趟的状态数改写；有判过的行还是拒。
    ///
    /// # Errors
    /// 服务读写不成；同一个块键录过、状态数不同、这一块已经有判过的行。
    pub fn record_block_of_a_slice(
        &mut self,
        block_key: &VerdictBlockKey,
        state_count: u64,
    ) -> Result<(), VerdictStoreError> {
        let key_bytes = VerdictStoreKey::RecordedBlock(block_key).to_key_bytes();
        if let Some(stored) = self.read_value_kept_in_or_out(&key_bytes.0)? {
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

    /// 这一块在判定块表、结论块表、核对表里有没有任何一行（哪一判法单元指纹、哪一种结论、哪一份核对代码都算）。结论块看块属性表
    /// （与结论块同一批写、键的后缀相同、值小），各表只问「这个前缀下有没有一行」（[`Self::has_an_entry_with_key_prefix_kept_in_or_out`]），不读整块。
    ///
    /// # Errors
    /// 服务读不成。
    fn block_has_a_judged_row(
        &self,
        block_key: &VerdictBlockKey,
    ) -> Result<bool, VerdictStoreError> {
        for table in [
            VerdictStoreTable::VerdictBlocks,
            VerdictStoreTable::ConclusionBlockAttributes,
            VerdictStoreTable::VerifierVerdicts,
        ] {
            let prefix = VerdictStoreKey::RowsOfABlockPrefix(table, block_key).to_key_bytes();
            if self.has_an_entry_with_key_prefix_kept_in_or_out(prefix)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// 录入表里这一块要核几个状态；没录过交回 None（分片的段从 0 起逐块问库时按它找下一块的起点）。
    ///
    /// # Errors
    /// 服务读不成；值不是 8 字节。
    pub fn recorded_state_count_of_block(
        &self,
        block_key: &VerdictBlockKey,
    ) -> Result<Option<u64>, VerdictStoreError> {
        let key_bytes = VerdictStoreKey::RecordedBlock(block_key).to_key_bytes();
        let Some(stored) = self.read_value_kept_in_or_out(&key_bytes.0)? else {
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
    /// 服务写不成。
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
    /// 服务读不成；值不合 [`SlicedSegmentProgress`] 的布局。
    pub fn read_sliced_segment_progress(
        &self,
        key: &SlicedSegmentProgressKey,
    ) -> Result<Option<SlicedSegmentProgress>, VerdictStoreError> {
        let key_bytes = VerdictStoreKey::SlicedSegmentProgress(key).to_key_bytes();
        let Some(stored) = self.read_value_kept_in_or_out(&key_bytes.0)? else {
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
    /// 服务读写不成。
    pub fn record_path(&mut self, path: &str, identity: &str) -> Result<(), VerdictStoreError> {
        let key_bytes = VerdictStoreKey::Path(path).to_key_bytes();
        let stored_identity = self.read_value_kept_in_or_out(&key_bytes.0)?;
        if stored_identity.as_deref() == Some(identity.as_bytes()) {
            return Ok(());
        }
        let mut batch = LibraryWriteBatch::default();
        batch.put(&key_bytes.0, identity.as_bytes());
        self.write_batch_keeping_the_kept_out_rows_in_this_process(batch)
    }

    /// 这条路径上登过的身份；没登过交回 None。
    ///
    /// # Errors
    /// 服务读不成；值不是 UTF-8。
    pub fn identity_of_path(&self, path: &str) -> Result<Option<String>, VerdictStoreError> {
        let key_bytes = VerdictStoreKey::Path(path).to_key_bytes();
        let Some(value) = self.read_value_kept_in_or_out(&key_bytes.0)? else {
            return Ok(None);
        };
        String::from_utf8(value.clone()).map(Some).map_err(|_| {
            stored_bytes_mismatch(VerdictStoreTable::PathIndex, &value, "身份不是 UTF-8")
        })
    }

    /// 路径索引里以这段前缀起头的全部（路径，身份），按键序：给空串就是整棵树；给 `<某节点路径>/` 就是它的子树。服务那一路逐帧扫、逐行认
    /// （[`Self::visit_entries_with_key_prefix_kept_in_or_out`]），交回的只有路径与身份的文字。
    ///
    /// # Errors
    /// 服务读不成；键或值不是 UTF-8。
    pub fn paths_with_prefix(
        &self,
        prefix: &str,
    ) -> Result<Vec<(String, String)>, VerdictStoreError> {
        let prefix_bytes = VerdictStoreKey::Path(prefix).to_key_bytes();
        let mut found = Vec::new();
        self.visit_entries_with_key_prefix_kept_in_or_out(prefix_bytes, &mut |entry| {
            let (key_bytes, value_bytes) = entry;
            let path = String::from_utf8(key_bytes[1..].to_vec()).map_err(|_| {
                stored_bytes_mismatch(VerdictStoreTable::PathIndex, &key_bytes, "路径不是 UTF-8")
            })?;
            let identity = String::from_utf8(value_bytes.to_vec()).map_err(|_| {
                stored_bytes_mismatch(VerdictStoreTable::PathIndex, &value_bytes, "身份不是 UTF-8")
            })?;
            found.push((path, identity));
            Ok(())
        })?;
        Ok(found)
    }

    /// 事实表登一条流（第 ① 段）：同一摘要再登就是同一份内容，照写不算错。
    ///
    /// # Errors
    /// 服务写不成。
    pub fn store_flow_facts(
        &mut self,
        facts_digest: &[u8; 32],
        facts_bytes: &[u8],
    ) -> Result<(), VerdictStoreError> {
        let key_bytes = VerdictStoreKey::FlowFacts(facts_digest).to_key_bytes();
        let mut batch = LibraryWriteBatch::default();
        batch.put(&key_bytes.0, facts_bytes);
        self.write_batch_keeping_the_kept_out_rows_in_this_process(batch)
    }

    /// 读一条流的事实；没登过交回 None。
    ///
    /// # Errors
    /// 服务读不成。
    pub fn read_flow_facts(
        &self,
        facts_digest: &[u8; 32],
    ) -> Result<Option<Vec<u8>>, VerdictStoreError> {
        let key_bytes = VerdictStoreKey::FlowFacts(facts_digest).to_key_bytes();
        self.read_value_kept_in_or_out(&key_bytes.0)
    }

    /// 录入段写下一条流程（`crash_amplification::RecordedFlow` 的字节，键是它的摘要）。同一摘要再写就是同一份内容，照写。
    ///
    /// # Errors
    /// 服务写不成。
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
    /// 服务读不成。
    pub fn read_recorded_flow(
        &self,
        digest: &[u8; 32],
    ) -> Result<Option<Vec<u8>>, VerdictStoreError> {
        self.read_one_stored_value(&VerdictStoreKey::RecordedFlow(digest))
    }

    /// 录入段登一趟的清单（键是录入键 [`RecordingKey`]：同一把键就是同一份录入输入、同一份录入代码，再登照写同一份）。
    ///
    /// # Errors
    /// 服务写不成。
    pub fn store_recorded_run_manifest(
        &mut self,
        recording_key: &RecordingKey,
        bytes: &[u8],
    ) -> Result<(), VerdictStoreError> {
        self.put_one_synced(&VerdictStoreKey::RecordedRunManifest(recording_key), bytes)
    }

    /// 一趟的清单表里有几行（拒跑之后核「一份清单都没登」用）：整表逐帧扫、逐行数，不留行。
    ///
    /// # Errors
    /// 服务读不成。
    pub fn recorded_run_manifest_count(&self) -> Result<u64, VerdictStoreError> {
        let prefix = VerdictStoreKey::WholeTablePrefix(VerdictStoreTable::RecordedRunManifests)
            .to_key_bytes();
        let mut count = 0u64;
        self.visit_entries_with_key_prefix_kept_in_or_out(prefix, &mut |_entry| {
            count += 1;
            Ok(())
        })?;
        Ok(count)
    }

    /// 读录入键下登的清单；没登过交回 None。
    ///
    /// # Errors
    /// 服务读不成。
    pub fn read_recorded_run_manifest(
        &self,
        recording_key: &RecordingKey,
    ) -> Result<Option<Vec<u8>>, VerdictStoreError> {
        self.read_one_stored_value(&VerdictStoreKey::RecordedRunManifest(recording_key))
    }

    /// 存一条对拍结论（[`StoredPairingConclusion`] 的字节同步落盘）。同一键再存照写（同一份输入、同一对判器判出的结论相同）。
    ///
    /// # Errors
    /// 服务写不成。
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
    /// 服务读不成；值不合 [`StoredPairingConclusion`] 的布局（跑法码不是这一版认得的也是）。
    pub fn read_pairing_conclusion(
        &self,
        key: &PairingConclusionKey,
    ) -> Result<Option<StoredPairingConclusion>, VerdictStoreError> {
        let key_bytes = VerdictStoreKey::PairingConclusion(key).to_key_bytes();
        let Some(stored) = self.read_value_kept_in_or_out(&key_bytes.0)? else {
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
    /// 服务写不成。
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
    /// 服务读不成。
    pub fn read_packed_judge_tables(
        &self,
        digest: &[u8; 32],
    ) -> Result<Option<Vec<u8>>, VerdictStoreError> {
        self.read_one_stored_value(&VerdictStoreKey::PackedJudgeTables(digest))
    }

    /// 写下一张给定状态表（键是它的摘要）。
    ///
    /// # Errors
    /// 服务写不成。
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
    /// 服务读不成。
    pub fn read_given_states_table(
        &self,
        digest: &[u8; 32],
    ) -> Result<Option<Vec<u8>>, VerdictStoreError> {
        self.read_one_stored_value(&VerdictStoreKey::GivenStatesTable(digest))
    }

    /// 观察者录入段写下一条流的崩溃镜像共用的基底（键是它的摘要；同一摘要再写就是同一份内容，照写）。
    ///
    /// # Errors
    /// 服务写不成。
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
    /// 服务读不成。
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
    /// 服务写不成。
    pub fn store_observed_images(
        &mut self,
        records: &[([u8; 32], Vec<u8>)],
    ) -> Result<(), VerdictStoreError> {
        if records.is_empty() {
            return Ok(());
        }
        let mut batch = LibraryWriteBatch::default();
        for (deduplication_key, bytes) in records {
            batch.put(
                VerdictStoreKey::ObservedImage(deduplication_key)
                    .to_key_bytes()
                    .0,
                bytes,
            );
        }
        self.write_batch_keeping_the_kept_out_rows_in_this_process(batch)
    }

    /// 读一张看过的恢复镜像的字节；没看过交回 None。
    ///
    /// # Errors
    /// 服务读不成。
    pub fn read_observed_image(
        &self,
        deduplication_key: &[u8; 32],
    ) -> Result<Option<Vec<u8>>, VerdictStoreError> {
        self.read_one_stored_value(&VerdictStoreKey::ObservedImage(deduplication_key))
    }

    /// 一个桶的全部读集见证行，按见证摘要的字节序；这个桶还没有见证交回空的。
    ///
    /// # Errors
    /// 键不是桶键之后接 32 字节见证摘要、值不是 64 字节（[`VerdictStoreError::StoredBytesDoNotMatchTheLayout`]）；服务读不成。
    pub fn read_observer_read_set_witnesses_of_the_bucket(
        &self,
        bucket_key: &[u8; 32],
    ) -> Result<Vec<ObserverReadSetWitnessRow>, VerdictStoreError> {
        let prefix =
            VerdictStoreKey::ObserverReadSetWitnessRowsOfABucketPrefix(bucket_key).to_key_bytes();
        let prefix_length = prefix.0.len();
        let mut witnesses = Vec::new();
        for entry in self.entries_with_key_prefix_kept_in_or_out(prefix) {
            let (key_bytes, value_bytes) = entry?;
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
    /// 服务读不成。
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
    /// 服务写不成；测试开关拒了这一批（[`VerdictStoreError::ObserverWitnessBatchRefusedByTheTestSwitch`]，一行都没写）。
    pub fn store_observed_image_with_its_read_set_witness(
        &mut self,
        bucket_key: &[u8; 32],
        witness: &ObserverReadSetWitnessRow,
        read_set_bytes_new_in_the_bucket: Option<&[u8]>,
        image_bytes: &[u8],
    ) -> Result<(), VerdictStoreError> {
        let mut batch = LibraryWriteBatch::default();
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
        batch: LibraryWriteBatch,
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
        self.write_batch_keeping_the_kept_out_rows_in_this_process(batch)
    }

    /// 观察者录入段写下一块（每个状态落在哪一份观察结果上、按发布分的态数）。同一键再写就盖掉上一趟的。
    ///
    /// # Errors
    /// 服务写不成。
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
    /// 服务读不成。
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
    /// 服务写不成。
    pub fn store_observed_image_judgements(
        &mut self,
        kind: &ConclusionKind,
        judging_units_fingerprint: &ConclusionVersion,
        records: &[([u8; 32], Vec<u8>)],
    ) -> Result<(), VerdictStoreError> {
        if records.is_empty() {
            return Ok(());
        }
        let mut batch = LibraryWriteBatch::default();
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
        self.write_batch_keeping_the_kept_out_rows_in_this_process(batch)
    }

    /// 读一张观察者交回的镜像在这个判器种类与版本下的池级 checker 结论；没判过交回 None。
    ///
    /// # Errors
    /// 服务读不成。
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
        let mut batch = LibraryWriteBatch::default();
        batch.put(&key_bytes.0, bytes);
        self.write_batch_keeping_the_kept_out_rows_in_this_process(batch)
    }

    fn read_one_stored_value(
        &self,
        key: &VerdictStoreKey<'_>,
    ) -> Result<Option<Vec<u8>>, VerdictStoreError> {
        self.read_value_kept_in_or_out(&key.to_key_bytes().0)
    }

    /// 核对表写一块（第 ② 段）：这一版核对代码在这一块每个状态的结论位。同一键再写就是同一份结论，照写。
    ///
    /// # Errors
    /// 服务写不成。
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
        let mut batch = LibraryWriteBatch::default();
        batch.put(&key_bytes.0, value);
        self.write_batch_keeping_the_kept_out_rows_in_this_process(batch)
    }

    /// 读核对表的一块；这一版核对代码没核过这一块交回 None。
    ///
    /// # Errors
    /// 服务读不成；值的长度对不上。
    pub fn read_verifier_block(
        &self,
        block_key: &VerdictBlockKey,
        verifier_digest: &[u8; 32],
    ) -> Result<Option<VerifierBlockRow>, VerdictStoreError> {
        let key_bytes = VerdictStoreKey::VerifierBlock(block_key, verifier_digest).to_key_bytes();
        let Some(value) = self.read_value_kept_in_or_out(&key_bytes.0)? else {
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
    /// 服务读不成；键字节认不回来。
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
            let (key_bytes, _value_bytes) = entry?;
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

    /// 一张按块键排的表里的全部行逐行交给 `visit_row`：块键、块键之后的键后缀、值。服务那一路逐帧扫、逐行交
    /// （[`Self::visit_entries_with_key_prefix_kept_in_or_out`]），留不留值由 `visit_row` 定。
    ///
    /// # Errors
    /// 服务读不成；键字节认不回块键；`visit_row` 的错。
    fn visit_rows_of_a_block_table(
        &self,
        table: VerdictStoreTable,
        visit_row: &mut BlockTableRowVisitor<'_>,
    ) -> Result<(), VerdictStoreError> {
        let prefix = VerdictStoreKey::WholeTablePrefix(table).to_key_bytes();
        self.visit_entries_with_key_prefix_kept_in_or_out(prefix, &mut |(
            key_bytes,
            value_bytes,
        )| {
            let (block_key, suffix) =
                VerdictBlockKey::from_block_key_bytes_with_suffix(&key_bytes[KEY_TAG_BYTES..])
                    .ok_or_else(|| stored_bytes_mismatch(table, &key_bytes, "键字节认不回块键"))?;
            visit_row(block_key, suffix, &value_bytes)
        })
    }

    /// 录入表里的全部块与各自的状态数，按键序。对账用：录了而判定块表里没有同一个键的，就是还没核完的。值是 8 字节，整表逐帧扫、只留块键与状态数。
    ///
    /// # Errors
    /// 服务读不成；键或值字节认不回来。
    pub fn recorded_blocks(&self) -> Result<Vec<(VerdictBlockKey, u64)>, VerdictStoreError> {
        let mut found = Vec::new();
        self.visit_rows_of_a_block_table(
            VerdictStoreTable::RecordedBlocks,
            &mut |block_key, suffix, value| {
                if !suffix.is_empty() {
                    return Err(stored_bytes_mismatch(
                        VerdictStoreTable::RecordedBlocks,
                        suffix,
                        "录入表的键在块键之后不该有别的字节",
                    ));
                }
                let count = <[u8; 8]>::try_from(value)
                    .map(u64::from_be_bytes)
                    .map_err(|_| {
                        stored_bytes_mismatch(
                            VerdictStoreTable::RecordedBlocks,
                            value,
                            "状态数不是 8 字节",
                        )
                    })?;
                found.push((block_key, count));
                Ok(())
            },
        )?;
        Ok(found)
    }

    /// 判定块表里的全部（块键，判法单元指纹），按键序：同一块在几个版本下判过就有几行。只留键：服务的前缀扫连值一起回（协议没有只回键的扫），
    /// 逐帧扫、逐行丢掉值，内存只随块数涨；传的字节随整张表涨（这张表只有测试的一次性库在读，产品的对账走结论那几张）。
    ///
    /// # Errors
    /// 服务读不成；键字节认不回来。
    pub fn stored_verdict_block_keys(
        &self,
    ) -> Result<Vec<(VerdictBlockKey, JudgingUnitsFingerprint)>, VerdictStoreError> {
        let mut found = Vec::new();
        self.visit_rows_of_a_block_table(
            VerdictStoreTable::VerdictBlocks,
            &mut |block_key, suffix, _value| {
                let version = <[u8; SHA256_DIGEST_BYTES]>::try_from(suffix)
                    .map(JudgingUnitsFingerprint)
                    .map_err(|_| {
                        stored_bytes_mismatch(
                            VerdictStoreTable::VerdictBlocks,
                            suffix,
                            "判定块的键在块键之后要正好是 32 字节的判法单元指纹",
                        )
                    })?;
                found.push((block_key, version));
                Ok(())
            },
        )?;
        Ok(found)
    }

    /// 判法单元组成清单表登一把键的组成清单（随键落盘：库对不上时照它点名变了的单元）。同一把再登照写。
    /// 组成清单是空的不写：它的文字是空串，组成清单表不是判定向量那两张表，空值在 prune-server 里是删除记号（`an_empty_value_may_be_put_under_the_key`）；
    /// 读回时没登过（[`Self::read_judging_units_composition`] 交 `None`）与空清单点名的变了的单元相同，都是一个都没有。
    ///
    /// # Errors
    /// 服务写不成。
    pub fn store_judging_units_composition(
        &mut self,
        key: &JudgingUnitsKey,
    ) -> Result<(), VerdictStoreError> {
        if key.composition().is_empty() {
            return Ok(());
        }
        self.put_one_synced(
            &VerdictStoreKey::JudgingUnitsComposition(key.fingerprint()),
            key.composition_text().as_bytes(),
        )
    }

    /// 一把判法单元指纹登过的组成清单；没登过交回 None。
    ///
    /// # Errors
    /// 服务读不成；值认不回组成清单。
    pub fn read_judging_units_composition(
        &self,
        fingerprint: &[u8; SHA256_DIGEST_BYTES],
    ) -> Result<Option<Vec<JudgingUnitFingerprint>>, VerdictStoreError> {
        let key_bytes = VerdictStoreKey::JudgingUnitsComposition(fingerprint).to_key_bytes();
        let Some(stored) = self.read_value_kept_in_or_out(&key_bytes.0)? else {
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
    /// 服务读不成；键不是 32 字节。
    pub fn judging_units_fingerprints(
        &self,
    ) -> Result<Vec<[u8; SHA256_DIGEST_BYTES]>, VerdictStoreError> {
        let prefix = VerdictStoreKey::WholeTablePrefix(VerdictStoreTable::JudgingUnitsCompositions)
            .to_key_bytes();
        let mut found = Vec::new();
        self.visit_entries_with_key_prefix_kept_in_or_out(prefix, &mut |(
            key_bytes,
            _value_bytes,
        )| {
            found.push(
                <[u8; SHA256_DIGEST_BYTES]>::try_from(&key_bytes[KEY_TAG_BYTES..]).map_err(
                    |_| {
                        stored_bytes_mismatch(
                            VerdictStoreTable::JudgingUnitsCompositions,
                            &key_bytes,
                            "组成清单表的键要是 32 字节",
                        )
                    },
                )?,
            );
            Ok(())
        })?;
        Ok(found)
    }

    /// 这一块这一种结论在库里存着的每一把版本（按版本字节排）：键对不上时照它找库里那一把，点名变了的单元。扫的是块属性表
    /// （与结论块同一批写、键的后缀相同、值只有几十字节），不把几把版本的整块结论读回来。
    ///
    /// # Errors
    /// 服务读不成。
    pub fn conclusion_versions_stored_for(
        &self,
        block_key: &VerdictBlockKey,
        kind: &ConclusionKind,
    ) -> Result<Vec<ConclusionVersion>, VerdictStoreError> {
        let mut prefix_after_the_tag = block_key.to_block_key_bytes();
        prefix_after_the_tag.extend_from_slice(&conclusion_kind_key_bytes(kind));
        let mut prefix = vec![VerdictStoreTable::ConclusionBlockAttributes.key_tag()];
        prefix.extend_from_slice(&prefix_after_the_tag);
        let prefix_length = prefix.len();
        let mut found = Vec::new();
        for entry in self.entries_with_key_prefix_kept_in_or_out(VerdictStoreKeyBytes(prefix)) {
            let (key_bytes, _value_bytes) = entry?;
            if let Ok(version) = <[u8; SHA256_DIGEST_BYTES]>::try_from(&key_bytes[prefix_length..])
            {
                found.push(ConclusionVersion(version));
            }
        }
        Ok(found)
    }

    /// 存一块一种结论：结论块、它的违例、计数向量、块属性与恢复字同一批同步写下去之后才返回；
    /// 违例状态的判定向量编号取块里那个状态的。同一块在别的种类、别的版本下的行不受影响。这个（块键，种类，版本）下已经存着不同的结论、
    /// 两份里有一份红的：照这一份换上（[`VerdictBlockStoring::ReplacedBecauseOneOfTheTwoIsRed`]）。交来的每一块都记进这个进程的账
    /// （[`Self::conclusion_blocks_offered_since`]，复用哨兵读它）。
    ///
    /// # Errors
    /// 这个（块键，种类，版本）下已经存着不同的结论、两份都不红
    /// （`DifferentConclusionUnderAStoredKeyNotDecidedInTheFirstVersion`，什么都没写）；已存的行读回来不合布局；服务读写不成。
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
        let registered_count = self.known_verdict_vector_count();
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
        self.conclusion_blocks_offered_in_this_process.push((
            block_key.clone(),
            kind.clone(),
            *conclusion_version,
        ));
        let mut batch = LibraryWriteBatch::default();
        let storing = self.put_a_conclusion_block_into_the_batch(
            block_key,
            kind,
            conclusion_version,
            &requested,
            &mut batch,
        )?;
        match storing {
            VerdictBlockStoring::AlreadyStoredIdentically => return Ok(storing),
            VerdictBlockStoring::WrittenAndSynced
            | VerdictBlockStoring::ReplacedBecauseOneOfTheTwoIsRed => {}
        }
        self.write_batch_keeping_the_kept_out_rows_in_this_process(batch)?;
        Ok(storing)
    }

    /// 一块一种结论（编号已是这个库的）放进 `batch`（规矩只在这一处）：这个（块键，种类，版本）下结论相同的什么都不放、交回
    /// [`VerdictBlockStoring::AlreadyStoredIdentically`]（只有在哪判的不同也算相同，留先存的那一份块属性）；不同而两份里有一份红的，照这一份换上、
    /// 旧的有而这一份没有的行同一批删掉（[`VerdictBlockStoring::ReplacedBecauseOneOfTheTwoIsRed`]）；没存过的交回 [`VerdictBlockStoring::WrittenAndSynced`]。
    /// 交回的是这一批写下去之后的结局，写不写由调用方定。库里那一份只读一次（一帧成批点读，[`Self::read_conclusion_block`]）。
    ///
    /// # Errors
    /// 同 [`Self::stored_conclusion_compared_with`]（两份都不红而不同时 `batch` 里没放这一块的行）。
    fn put_a_conclusion_block_into_the_batch(
        &self,
        block_key: &VerdictBlockKey,
        kind: &ConclusionKind,
        conclusion_version: &ConclusionVersion,
        requested: &StoredConclusionBlock,
        batch: &mut LibraryWriteBatch,
    ) -> Result<VerdictBlockStoring, VerdictStoreError> {
        // 记红、撤绿：库里那一份与这一份不同、两份里有一份红的，照这一份整份换上（判红写红记录、撤同一把键的绿；
        // 记着红的块下一趟现判，判完照新结论写回）；旧的那一份有、这一份没有的行同一批删掉。两份都不红的照下面的比法拒
        let stored = self.read_conclusion_block(block_key, kind, conclusion_version)?;
        let replaced_stored = match &stored {
            Some(stored)
                if !stored.is_the_same_conclusion_as(requested)
                    && (!stored.violations.is_empty() || !requested.violations.is_empty()) =>
            {
                delete_the_rows_the_replacing_conclusion_drops(
                    batch,
                    block_key,
                    kind,
                    conclusion_version,
                    stored,
                    requested,
                );
                true
            }
            Some(_) | None => false,
        };
        if !replaced_stored {
            match self.stored_conclusion_compared_with(
                block_key,
                kind,
                conclusion_version,
                stored,
                requested,
            )? {
                StoredConclusionComparison::StoredIdentically
                | StoredConclusionComparison::StoredIdenticallyOnAnotherJudgingDevice => {
                    return Ok(VerdictBlockStoring::AlreadyStoredIdentically)
                }
                StoredConclusionComparison::NotStored => {}
            }
        }
        put_conclusion_rows_into_the_batch(batch, block_key, kind, conclusion_version, requested);
        if replaced_stored {
            Ok(VerdictBlockStoring::ReplacedBecauseOneOfTheTwoIsRed)
        } else {
            Ok(VerdictBlockStoring::WrittenAndSynced)
        }
    }

    /// 这一块这一种结论这一把版本在库里有没有违例（只扫违例表这一块的前缀、见到一行就停）；没存过的块答没有。
    ///
    /// # Errors
    /// 服务读不成。
    pub fn conclusion_block_has_violations(
        &self,
        block_key: &VerdictBlockKey,
        kind: &ConclusionKind,
        conclusion_version: &ConclusionVersion,
    ) -> Result<bool, VerdictStoreError> {
        let prefix =
            VerdictStoreKey::ConclusionViolationsOfBlockPrefix(block_key, kind, conclusion_version)
                .to_key_bytes();
        self.has_an_entry_with_key_prefix_kept_in_or_out(prefix)
    }

    /// 要存的一块一种结论（编号已是这个库的）与库里这个（块键，种类，版本）下存着的那一份（`stored`，调用方刚读回的）比：没存过（恢复字旁表也没有这一行）交回
    /// `NotStored`；块、违例、计数向量、恢复字与块属性都相同交回 `StoredIdentically`；只有块属性（在哪判的）不同交回
    /// `StoredIdenticallyOnAnotherJudgingDevice`。规矩只在这一处。
    ///
    /// # Errors
    /// 结论不同（`DifferentConclusionUnderAStoredKeyNotDecidedInTheFirstVersion`，带着两份差在哪）；恢复字旁表有这一行而结论块表没有；
    /// 已存的行读回来不合布局；服务读不成。
    fn stored_conclusion_compared_with(
        &self,
        block_key: &VerdictBlockKey,
        kind: &ConclusionKind,
        conclusion_version: &ConclusionVersion,
        stored: Option<StoredConclusionBlock>,
        requested: &StoredConclusionBlock,
    ) -> Result<StoredConclusionComparison, VerdictStoreError> {
        if let Some(stored) = stored {
            if !stored.is_the_same_conclusion_as(requested) {
                let verdict_vector_of_number =
                    |number: VerdictVectorNumber| self.verdict_vector_of_number(number);
                return Err(
                    VerdictStoreError::DifferentConclusionUnderAStoredKeyNotDecidedInTheFirstVersion {
                        block_key: block_key.clone(),
                        kind: kind.clone(),
                        differences: Box::new(DifferencesOfTwoConclusionsUnderOneKey {
                            differences: differences_between_two_conclusions(
                                &stored,
                                requested,
                                verdict_vector_of_number,
                            ),
                            stored_judged_on: stored.judged_on.clone(),
                            requested_judged_on: requested.judged_on.clone(),
                        }),
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
        let stray_recovery_words = self.read_value_kept_in_or_out(&recovery_words_key.0)?;
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
            .read_value_kept_in_or_out(&abandoned_roots_key.0)?
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
    /// 存着的结论块是空的或点名了没登记的编号、缺计数向量或块属性、计数向量或块属性或恢复字不合布局（块属性的跑法码不是这一版认得的也是）；
    /// 服务读不成。
    pub fn read_conclusion_block(
        &self,
        block_key: &VerdictBlockKey,
        kind: &ConclusionKind,
        conclusion_version: &ConclusionVersion,
    ) -> Result<Option<StoredConclusionBlock>, VerdictStoreError> {
        let conclusion_key =
            VerdictStoreKey::ConclusionBlock(block_key, kind, conclusion_version).to_key_bytes();
        let count_key = VerdictStoreKey::ConclusionCountVector(block_key, kind, conclusion_version)
            .to_key_bytes();
        let attribute_key =
            VerdictStoreKey::ConclusionBlockAttributes(block_key, kind, conclusion_version)
                .to_key_bytes();
        let recovery_words_key =
            VerdictStoreKey::RecoveryWords(block_key, kind, conclusion_version).to_key_bytes();
        let abandoned_roots_key = VerdictStoreKey::AbandonedRootsNamedByThePoolChecker(
            block_key,
            kind,
            conclusion_version,
        )
        .to_key_bytes();
        // 一块一种结论的几行一起读：垫在 prune-server 上时是一帧成批点读，不逐键一帧（违例另一帧按前缀扫）
        let [conclusion_value, count_value, attribute_value, recovery_words_value, abandoned_roots_value]: [Option<Vec<u8>>; 5] = self
            .read_values_kept_in_or_out(&[
                conclusion_key.0.as_slice(),
                count_key.0.as_slice(),
                attribute_key.0.as_slice(),
                recovery_words_key.0.as_slice(),
                abandoned_roots_key.0.as_slice(),
            ])?
            .try_into()
            .expect("问了五行：交回的个数与次序与问的相同");
        let Some(conclusion_value_bytes) = conclusion_value else {
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
        let count_bytes = count_value.ok_or_else(|| {
            stored_bytes_mismatch(
                VerdictStoreTable::ConclusionCountVectors,
                &count_key.0,
                "结论块在而这一行缺着",
            )
        })?;
        let count_vector = count_vector_from_bytes(&count_bytes).ok_or_else(|| {
            stored_bytes_mismatch(
                VerdictStoreTable::ConclusionCountVectors,
                &count_key.0,
                "计数向量的字节不合布局",
            )
        })?;
        let attribute_bytes = attribute_value.ok_or_else(|| {
            stored_bytes_mismatch(
                VerdictStoreTable::ConclusionBlockAttributes,
                &attribute_key.0,
                "结论块在而这一行缺着",
            )
        })?;
        let (judged_on, circumstances) = conclusion_block_attributes_from_bytes(&attribute_bytes)
            .ok_or_else(|| {
            stored_bytes_mismatch(
                VerdictStoreTable::ConclusionBlockAttributes,
                &attribute_key.0,
                "块属性的字节不合布局",
            )
        })?;
        let recovery_words = match recovery_words_value {
            None => None,
            Some(words_bytes) => Some(
                recovery_words_from_value_bytes(&words_bytes, state_count).ok_or_else(|| {
                    stored_bytes_mismatch(
                        VerdictStoreTable::RecoveryWords,
                        &recovery_words_key.0,
                        "恢复字不是块里每个状态 RECOVERY_WORDS_PER_STATE 个字",
                    )
                })?,
            ),
        };
        let abandoned_roots_named_by_the_pool_checker = match abandoned_roots_value {
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
            let (key_bytes, number_bytes) = entry?;
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

    /// 一块要的几种结论里哪几种还没有（按给的次序）：「哪一种没有就只排那一种」。每一项的版本由调用方给：
    /// GPU 判器一把、这一块的设点文件那一把 CPU、挂在这个节点上的每个观察者各一把。
    ///
    /// # Errors
    /// 服务读不成；块属性那一行在而不合布局（跑法码不是这一版认得的也是：报错，不当没判）。
    pub fn conclusions_missing_for_block(
        &self,
        block_key: &VerdictBlockKey,
        required: &[(ConclusionKind, ConclusionVersion)],
    ) -> Result<Vec<(ConclusionKind, ConclusionVersion)>, VerdictStoreError> {
        // 一块要的几种结论一起问（一帧成批点读，不逐键一帧）；问的是块属性那一行（与结论块同一批写、值只有几十字节），不把整块结论读回来
        let key_bytes_of_each: Vec<VerdictStoreKeyBytes> = required
            .iter()
            .map(|(kind, conclusion_version)| {
                VerdictStoreKey::ConclusionBlockAttributes(block_key, kind, conclusion_version)
                    .to_key_bytes()
            })
            .collect();
        let keys: Vec<&[u8]> = key_bytes_of_each
            .iter()
            .map(|key_bytes| key_bytes.0.as_slice())
            .collect();
        let stored = self.read_values_kept_in_or_out(&keys)?;
        let mut missing = Vec::new();
        for (((kind, conclusion_version), attribute_value), attribute_key) in
            required.iter().zip(stored).zip(&key_bytes_of_each)
        {
            let Some(attribute_bytes) = attribute_value else {
                missing.push((kind.clone(), *conclusion_version));
                continue;
            };
            if conclusion_block_attributes_from_bytes(&attribute_bytes).is_none() {
                return Err(stored_bytes_mismatch(
                    VerdictStoreTable::ConclusionBlockAttributes,
                    &attribute_key.0,
                    "块属性的字节不合布局",
                ));
            }
        }
        Ok(missing)
    }

    /// 结论块表里的全部（块键，种类，版本），按键序：同一块几种结论、几把版本就有几行。扫的是块属性表（与结论块同一批写、键的后缀相同、
    /// 值只有几十字节），整表逐帧扫、只留键（[`Self::visit_stored_conclusion_keys`]）。
    ///
    /// # Errors
    /// 服务读不成；键字节认不回来。
    pub fn stored_conclusion_keys(
        &self,
    ) -> Result<Vec<(VerdictBlockKey, ConclusionKind, ConclusionVersion)>, VerdictStoreError> {
        let mut found = Vec::new();
        self.visit_stored_conclusion_keys(&mut |conclusion| {
            found.push(conclusion);
            Ok(())
        })?;
        Ok(found)
    }

    /// 库里存着的每一块一种结论一把版本（块属性表的键），按键序逐个交给 `visit_conclusion`，不攒成一份：读库程序数整份库的块用。
    ///
    /// # Errors
    /// 服务读不成；键字节认不回来；`visit_conclusion` 的错。
    pub fn visit_stored_conclusion_keys(
        &self,
        visit_conclusion: &mut dyn FnMut(
            ConclusionOfABlockUnderAKey,
        ) -> Result<(), VerdictStoreError>,
    ) -> Result<(), VerdictStoreError> {
        self.visit_rows_of_a_block_table(
            VerdictStoreTable::ConclusionBlockAttributes,
            &mut |block_key, suffix, _value| {
                let unreadable = || {
                    stored_bytes_mismatch(
                        VerdictStoreTable::ConclusionBlockAttributes,
                        suffix,
                        "块属性的键在块键之后要是结论种类加 32 字节的版本",
                    )
                };
                let (kind, version_bytes) =
                    conclusion_kind_from_key_bytes(suffix).ok_or_else(unreadable)?;
                let conclusion_version = <[u8; SHA256_DIGEST_BYTES]>::try_from(version_bytes)
                    .map(ConclusionVersion)
                    .map_err(|_| unreadable())?;
                visit_conclusion((block_key, kind, conclusion_version))
            },
        )
    }
}

/// prune-server 那一侧的开库、编号与成批读（模块里「prune-server 那一侧」一节）。
impl VerdictStore {
    /// 开 prune-server 里那一份崩溃判定库（`service` 连的那一处、那一个键空间）：布局行不在（这个键空间里头一次放崩溃判定库）就写下这一版的布局号，
    /// 在而不是这一版的拒；读回按编号的判定向量表。判定向量编号由服务统一给（[`Self::register_verdict_vector`] 在服务里认领编号）。
    ///
    /// # Errors
    /// 读写不成；不是这一版的布局（`LibraryLayoutIsNotThisVersion`）；判定向量表编号不连续或一种两个编号。
    pub fn open_the_prune_server(
        service: Arc<PruneServerStorage>,
    ) -> Result<Self, VerdictStoreError> {
        let layout_key = VerdictStoreKey::LibraryLayout.to_key_bytes();
        let layout_value = match service
            .read_values(&[layout_key.0.as_slice()])?
            .pop()
            .flatten()
        {
            Some(layout_bytes) => layout_bytes,
            None => {
                let layout_bytes = LIBRARY_LAYOUT_NUMBER_OF_THIS_VERSION.to_be_bytes().to_vec();
                service.write_rows(vec![(layout_key.0.clone(), layout_bytes.clone())])?;
                layout_bytes
            }
        };
        verify_the_library_layout_row(Some(layout_value), &service.library_name())?;
        let loaded = load_verdict_vectors_of_the_prune_server(&service)?;
        Ok(Self::assembled_over_the_service(service, loaded))
    }

    /// 这一份库（这个键空间）里的全部行，按键升序（不连照判不入库的）：单测核「什么都没写」用。
    #[cfg(test)]
    pub(crate) fn every_stored_entry(&self) -> Vec<StoredEntry> {
        self.service
            .rows_with_prefix(&[])
            .expect("单测扫得了这一份库的全部行")
    }

    /// 库的名字（连的是哪一处、哪一个键空间；只进报错与行里报的名字）。
    #[must_use]
    pub fn library_name(&self) -> &str {
        &self.library_name
    }

    /// 一次读几行（[`Self::read_value_kept_in_or_out`] 的成批版）：照判不入库的先看这个进程里的，别的一起问服务（成批点读，不逐键一帧）。
    /// 交回的次序与 `keys` 相同。
    ///
    /// # Errors
    /// 服务读不成。
    fn read_values_kept_in_or_out(
        &self,
        keys: &[&[u8]],
    ) -> Result<Vec<Option<Vec<u8>>>, VerdictStoreError> {
        let mut values: Vec<Option<Vec<u8>>> = vec![None; keys.len()];
        let mut positions_in_the_library = Vec::new();
        {
            let kept_out = self
                .rows_kept_out_of_the_library
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for (position, key_bytes) in keys.iter().copied().enumerate() {
                if kept_out.keeps_out(key_bytes) {
                    values[position] = kept_out.rows.get(key_bytes).cloned();
                } else {
                    positions_in_the_library.push(position);
                }
            }
        }
        let keys_in_the_library: Vec<&[u8]> = positions_in_the_library
            .iter()
            .map(|position| keys[*position])
            .collect();
        let in_the_library = self.service.read_values(&keys_in_the_library)?;
        for (position, value) in positions_in_the_library.into_iter().zip(in_the_library) {
            values[position] = value;
        }
        Ok(values)
    }

    fn lock_the_verdict_vectors_fetched_after_opening(
        &self,
    ) -> std::sync::MutexGuard<'_, BTreeMap<u32, &'static EncodedVerdictVector>> {
        self.verdict_vectors_fetched_after_opening
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// 这一份认得的判定向量编号有几个：按编号的内存表连同开库之后从 prune-server 取来的（后者从内存表末尾起连续）。编号由这一份自己给的库
    /// 没有取来的，就是内存表的长度。
    fn known_verdict_vector_count(&self) -> usize {
        self.verdict_vector_by_number.len()
            + self.lock_the_verdict_vectors_fetched_after_opening().len()
    }

    /// 读到的编号这一份还不认得的，从 prune-server 取来（编号由服务统一给：别的进程在这一份开库之后登记的，这一份没见过）：从认得的末尾起
    /// 连续取到最大的那个，成批点读（[`PruneServerStorage::read_values`]：一帧几千个键，回应越过帧长上限由容错客户端拆两半）。
    /// 服务的按编号表里缺了的那一号起不再取（认得的编号要从内存表末尾起连续）、不报错：缺的是块里引的编号没登记过，错在引它的那一块，
    /// 由调用方照那一块的表与键报不合布局（[`Self::registered_verdict_vector_numbers_of_a_stored_block`]、[`Self::registered_verdict_vector_number`]）。
    ///
    /// # Errors
    /// 读不成。
    fn fetch_the_verdict_vectors_of_unknown_numbers(
        &self,
        stored_numbers: &[u32],
    ) -> Result<(), VerdictStoreError> {
        let Some(largest) = stored_numbers.iter().copied().max() else {
            return Ok(());
        };
        let mut fetched = self.lock_the_verdict_vectors_fetched_after_opening();
        let known_count = self.verdict_vector_by_number.len() + fetched.len();
        if VerdictVectorNumber(largest).position_in_the_registration_order() < known_count {
            return Ok(());
        }
        let first_unknown = u32::try_from(known_count).expect(
            "认得的个数不超过最大的那个编号：编号是 u32，最大的编号没认得，认得的个数装得进 u32",
        );
        let numbers: Vec<VerdictVectorNumber> =
            (first_unknown..=largest).map(VerdictVectorNumber).collect();
        let keys: Vec<VerdictStoreKeyBytes> = numbers
            .iter()
            .map(|number| VerdictStoreKey::VerdictVectorContentByNumber(*number).to_key_bytes())
            .collect();
        let key_slices: Vec<&[u8]> = keys.iter().map(|key| key.0.as_slice()).collect();
        let values = self.service.read_values(&key_slices)?;
        for (number, value) in numbers.iter().zip(values) {
            let Some(vector_bytes) = value else {
                break;
            };
            let leaked: &'static EncodedVerdictVector =
                Box::leak(Box::new(EncodedVerdictVector(vector_bytes)));
            fetched.insert(number.as_u32(), leaked);
        }
        Ok(())
    }

    /// 开库之后从 prune-server 取来的判定向量并进内存表（编号依次接在末尾）。
    ///
    /// # Errors
    /// 取来的编号接不上、或那一种判定向量已经有别的编号（服务的判定向量表不是一种一个编号）。
    fn take_in_the_verdict_vectors_fetched_after_opening(
        &mut self,
    ) -> Result<(), VerdictStoreError> {
        let fetched = std::mem::take(
            self.verdict_vectors_fetched_after_opening
                .get_mut()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        for (stored_number, verdict_vector) in fetched {
            self.append_a_verdict_vector_known_by_its_number(
                VerdictVectorNumber(stored_number),
                verdict_vector.clone(),
            )?;
        }
        Ok(())
    }

    /// 编号由服务统一给时，把服务里这个编号的判定向量接到内存表末尾：编号要正是下一个，那一种判定向量不能已有别的编号。
    ///
    /// # Errors
    /// 编号不是下一个；那一种已有别的编号。
    fn append_a_verdict_vector_known_by_its_number(
        &mut self,
        number: VerdictVectorNumber,
        verdict_vector: EncodedVerdictVector,
    ) -> Result<(), VerdictStoreError> {
        let is_the_next_number =
            VerdictVectorNumber::next_after_registered_count(self.verdict_vector_by_number.len())
                == Some(number);
        let has_another_number = self
            .verdict_vector_number_by_content
            .get(&verdict_vector.0)
            .is_some_and(|known_number| *known_number != number);
        if !is_the_next_number || has_another_number {
            return Err(stored_bytes_mismatch(
                VerdictStoreTable::VerdictVectorContentByNumber,
                &VerdictStoreKey::VerdictVectorContentByNumber(number)
                    .to_key_bytes()
                    .0,
                "prune-server 的判定向量表不是从 0 起连续、一种一个编号",
            ));
        }
        self.verdict_vector_number_by_content
            .insert(verdict_vector.0.clone(), number);
        self.verdict_vector_by_number.push(verdict_vector);
        Ok(())
    }

    /// 在 prune-server 里登记一种这一份还不认得的判定向量：从内存表末尾的下一个编号起，认领那个编号的按编号行——认到了就把两张判定向量表的
    /// 行同一帧写进服务（写帧确认时认领释放）；服务答已有就读回那一种接进内存表，正是要登记的这一种就用这个编号，不是就往下一个编号走；
    /// 别的客户端认领着就隔一会再问（它写完答已有，它死了租期一过放出来）。几个进程、两台同时登记同一种，只有一个认到，别的读回同一个编号。
    ///
    /// # Errors
    /// 读写、认领不成；等过 [`PRUNE_SERVER_NUMBER_CLAIM_WAIT_LIMIT`] 还被别人认领着；2^32 种都登记过了；服务的判定向量表不连续。
    fn register_verdict_vector_in_the_prune_server(
        &mut self,
        authority: &PruneServerStorage,
        verdict_vector: &EncodedVerdictVector,
    ) -> Result<VerdictVectorNumber, VerdictStoreError> {
        let content_key =
            VerdictStoreKey::VerdictVectorNumberByContent(verdict_vector).to_key_bytes();
        let waiting_started = std::time::Instant::now();
        loop {
            let Some(number) = VerdictVectorNumber::next_after_registered_count(
                self.verdict_vector_by_number.len(),
            ) else {
                return Err(VerdictStoreError::EveryU32VerdictVectorNumberIsAlreadyRegistered);
            };
            let number_key = VerdictStoreKey::VerdictVectorContentByNumber(number).to_key_bytes();
            match authority.claim(&number_key.0)? {
                ClaimOutcome::Claimed => {
                    // 经 write_batch 写：绿的判定向量是空字节，按编号那一行的值为空，只有这张表许写空值，别的表照旧拒
                    let mut registration = LibraryWriteBatch::default();
                    registration.put(&content_key.0, number.to_stored_bytes());
                    registration.put(&number_key.0, &verdict_vector.0);
                    authority.write_batch(registration)?;
                    self.append_a_verdict_vector_known_by_its_number(
                        number,
                        verdict_vector.clone(),
                    )?;
                    return Ok(number);
                }
                ClaimOutcome::AlreadyConcluded => {
                    let Some(stored_vector_bytes) = authority
                        .read_values(&[number_key.0.as_slice()])?
                        .pop()
                        .flatten()
                    else {
                        return Err(stored_bytes_mismatch(
                            VerdictStoreTable::VerdictVectorContentByNumber,
                            &number_key.0,
                            "prune-server 答这个判定向量编号已经登记过，读回来却没有",
                        ));
                    };
                    let is_this_verdict_vector = stored_vector_bytes == verdict_vector.0;
                    self.append_a_verdict_vector_known_by_its_number(
                        number,
                        EncodedVerdictVector(stored_vector_bytes),
                    )?;
                    if is_this_verdict_vector {
                        return Ok(number);
                    }
                }
                ClaimOutcome::HeldByOther { .. } => {
                    if waiting_started.elapsed() >= PRUNE_SERVER_NUMBER_CLAIM_WAIT_LIMIT {
                        return Err(authority.failed(
                            PRUNE_SERVER_UNAVAILABLE_EXIT_CODE,
                            format!(
                                "判定向量编号 {} 被别的客户端认领着，等过 {} 秒还没放",
                                number.as_u32(),
                                PRUNE_SERVER_NUMBER_CLAIM_WAIT_LIMIT.as_secs()
                            ),
                        ));
                    }
                    std::thread::sleep(PRUNE_SERVER_NUMBER_CLAIM_RETRY_INTERVAL);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crash_judge_conclusions::{LandedRootWords, RecoveryWordsOfAPass};
    use prune_server::protocol::RejectionStatus;

    /// 测试用的一次性库：它自己在这个进程里起的临时 prune-server 里的一段名字空间（`crate::crash_library::DisposableLibrary`），
    /// 各条用例各一份、各从空库起；临时服务起不来就当场失败，不跳过。
    struct TestLibrary {
        disposable: crate::crash_library::DisposableLibrary,
    }

    impl TestLibrary {
        fn new(test_name: &str) -> Self {
            // 一次性库留着：存放布局的自证，各条从空库起
            let disposable = crate::crash_library::DisposableLibrary::new(&format!(
                "verdict-library-unit-test-{test_name}"
            ))
            .unwrap_or_else(|refusal| panic!("{refusal}"));
            Self { disposable }
        }

        /// 开这一份（一个新的客户端连同它自己的追加日志）：读写不成交回错误、不退出进程。同一份开几次读到的是同一批行（重开）。
        fn open(&self) -> VerdictStore {
            self.disposable
                .open_store(PruneServerFailureHandling::ReturnTheError)
                .unwrap_or_else(|refusal| panic!("开不了一次性库：{refusal}"))
        }

        /// 这一份在服务里那一端：用例直接改行（造坏行、删行）用。
        fn service(&self) -> Arc<PruneServerStorage> {
            self.disposable
                .connect(PruneServerFailureHandling::ReturnTheError)
                .unwrap_or_else(|refusal| panic!("连不上临时 prune-server：{refusal}"))
        }

        /// 原样读一行（不经照判不入库），删除记号读成不在。
        fn raw_value(&self, key_bytes: &[u8]) -> Option<Vec<u8>> {
            self.service()
                .read_values(&[key_bytes])
                .expect("读得了临时 prune-server")
                .pop()
                .flatten()
        }

        /// 原样写几行（空值是删除记号）。
        fn write_raw_rows(&self, rows: Vec<(Vec<u8>, Vec<u8>)>) {
            self.service()
                .write_rows_for_a_test(rows)
                .expect("写得了临时 prune-server");
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
        NodeInputDigest(crate::sha256::sha256_digest(&message))
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
        let library = TestLibrary::new("read-back");
        let mut store = library.open();
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
        let stored_value_bytes = library
            .raw_value(
                &VerdictStoreKey::VerdictBlock(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT)
                    .to_key_bytes()
                    .0,
            )
            .expect("块在库里");
        assert_eq!(
            stored_value_bytes,
            vec![0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 2],
            "库里的块值要是每个状态一个 4 字节大端的编号"
        );
    }

    #[test]
    fn blocks_and_verdict_vector_numbers_survive_closing_and_reopening() {
        let library = TestLibrary::new("reopen");
        let block_key = block_key_named("stream", 0);
        let (clean, torn, block) = {
            let mut store = library.open();
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
        let mut store = library.open();
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

    /// 绿的判定向量（一个红项都没有）是空字节，登记在 0 号时按编号那张表里那一行的值为空（`03 ‖ 0 → 空值`）：登记过绿与一种红之后关库，
    /// 新开一个 store 重开同一份库不报错，绿仍是 0 号、红仍是 1 号，再登记两种拿回原来的编号。
    /// 判别力（`crates/mutations.tsv`）：前缀扫把判定向量表里的空值也当删除记号跳过（重开时看不到 0 号，判连号报编号不是从 0 起），这条红。
    #[test]
    fn a_green_verdict_vector_registered_before_closing_is_still_number_zero_after_reopening() {
        let library = TestLibrary::new("green-survives-reopening");
        let green = crate::crash_judge_conclusions::encoded_verdict_of_the_red_items(&[]);
        let red = verdict_vector(b"I-1.1 seeded red");
        {
            let mut store = library.open();
            assert_eq!(
                store
                    .register_verdict_vector(&green)
                    .expect("登记绿")
                    .as_u32(),
                0,
                "空库里头一种登记的绿拿到 0 号"
            );
            assert_eq!(
                store
                    .register_verdict_vector(&red)
                    .expect("登记红")
                    .as_u32(),
                1
            );
        }
        assert_eq!(
            library.raw_value(
                &VerdictStoreKey::VerdictVectorContentByNumber(VerdictVectorNumber(0))
                    .to_key_bytes()
                    .0
            ),
            Some(Vec::new()),
            "按编号那张表里 0 号那一行在、值是绿的空字节"
        );
        let mut reopened = library.open();
        assert_eq!(
            reopened.registered_verdict_vectors(),
            [green.clone(), red.clone()].as_slice(),
            "重开读回两种：0 号是绿、1 号是红"
        );
        assert_eq!(
            reopened
                .register_verdict_vector(&green)
                .expect("重开之后再登记绿")
                .as_u32(),
            0,
            "重开之后再登记绿拿回 0 号"
        );
        assert_eq!(
            reopened
                .register_verdict_vector(&red)
                .expect("重开之后再登记红")
                .as_u32(),
            1,
            "重开之后再登记红拿回 1 号"
        );
    }

    /// 两个 store 都在空库上开着，一个登记了绿（0 号、值为空），另一个再登记绿：它认领 0 号时服务答已经登记过，读回那一行的空值就是绿，
    /// 拿到 0 号、不报错；再登记一种红接着拿 1 号。
    /// 判别力（`crates/mutations.tsv`）：成批点读把判定向量表里的空值也读成不在（报「答已经登记过，读回来却没有」），这条红。
    #[test]
    fn another_store_registering_green_after_it_was_registered_gets_number_zero() {
        let library = TestLibrary::new("green-registered-by-another-store");
        let green = crate::crash_judge_conclusions::encoded_verdict_of_the_red_items(&[]);
        let mut store_registering_green = library.open();
        let mut store_registering_green_again = library.open();
        assert_eq!(
            store_registering_green
                .register_verdict_vector(&green)
                .expect("先登记绿")
                .as_u32(),
            0
        );
        assert_eq!(
            store_registering_green_again
                .register_verdict_vector(&green)
                .expect("另一个 store 再登记绿")
                .as_u32(),
            0,
            "另一个 store 登记同一种绿拿到同一个 0 号"
        );
        assert_eq!(
            store_registering_green_again
                .register_verdict_vector(&verdict_vector(b"I-1.1 seeded red"))
                .expect("再登记红")
                .as_u32(),
            1,
            "认得了 0 号之后新登记的接着拿 1 号"
        );
    }

    /// 一个 store 在空库上开着，另一个登记了绿与一种红、存了一块引这两个编号：先开的那个读这一块时从服务取来不认得的 0、1 两号，
    /// 0 号那一行的空值读成绿，读回的块与存的相同。
    /// 判别力（`crates/mutations.tsv`）：成批点读把判定向量表里的空值也读成不在（0 号取不来，读块报「存着的判定向量编号在这个库里没登记过」），这条红。
    #[test]
    fn a_block_referring_to_green_stored_by_another_store_reads_back_with_green_fetched_from_the_service(
    ) {
        let library = TestLibrary::new("green-fetched-from-the-service");
        let green = crate::crash_judge_conclusions::encoded_verdict_of_the_red_items(&[]);
        let block_key = block_key_named("stream", 0);
        let opened_before = library.open();
        let block = {
            let mut storing = library.open();
            let green_number = storing.register_verdict_vector(&green).expect("登记绿");
            let red_number = storing
                .register_verdict_vector(&verdict_vector(b"I-1.1 seeded red"))
                .expect("登记红");
            let block = block_of(&[green_number, red_number, green_number]);
            storing
                .store_verdict_block(
                    &block_key,
                    &TEST_JUDGING_UNITS_FINGERPRINT,
                    &block,
                    &[StateOffsetInBlock(1)],
                )
                .expect("存块");
            block
        };
        assert_eq!(
            opened_before
                .read_verdict_block(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT)
                .expect("读块时从服务取来不认得的编号"),
            Some(block),
            "先开的 store 读回别的 store 存的块"
        );
        assert_eq!(
            opened_before.verdict_vector_of_number(VerdictVectorNumber(0)),
            Some(&green),
            "从服务取来的 0 号是绿的空字节"
        );
    }

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

    #[test]
    fn registering_the_same_verdict_vector_twice_returns_the_same_number() {
        let library = TestLibrary::new("register-twice");
        let mut store = library.open();
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
        let library = TestLibrary::new("distinct-keys");
        let mut store = library.open();
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
        let library = TestLibrary::new("violations");
        let mut store = library.open();
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
        let library = TestLibrary::new("different-block");
        let mut store = library.open();
        let clean = store
            .register_verdict_vector(&verdict_vector(b"clean"))
            .expect("登记");
        let torn = store
            .register_verdict_vector(&verdict_vector(b"torn"))
            .expect("登记");
        let block_key = block_key_named("stream", 0);
        let block = block_of(&[clean, torn]);
        store
            .store_verdict_block(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT, &block, &[])
            .expect("存块");
        let entries_before = store.every_stored_entry();
        let different_block = store.store_verdict_block(
            &block_key,
            &TEST_JUDGING_UNITS_FINGERPRINT,
            &block_of(&[torn, torn]),
            &[],
        );
        assert!(
            matches!(
                different_block,
                Err(VerdictStoreError::DifferentVerdictsUnderAStoredBlockKeyNotDecidedInTheFirstVersion { .. })
            ),
            "同一个块键下两份都不红而换一个块要拒：{different_block:?}"
        );
        assert_eq!(
            store.every_stored_entry(),
            entries_before,
            "拒了就一个字都不写"
        );
    }

    /// 记红、撤绿：同一个块键下先存一份不红的，再存一份块相同、头一个状态违例的（红），再存回不红的。
    /// 预期：两次都交「换上」，读回是新存的那一份；红换绿之后违例行一行不剩。
    /// 判别力（`crates/mutations.tsv`）：判定块那一处两份不同一律拒（不看红不红），这条红。
    #[test]
    fn a_red_verdict_block_replaces_a_green_one_under_its_key_and_a_green_one_replaces_it_back() {
        let library = TestLibrary::new("red-verdict-block-replaces");
        let mut store = library.open();
        let clean = store
            .register_verdict_vector(&verdict_vector(b"clean"))
            .expect("登记");
        let torn = store
            .register_verdict_vector(&verdict_vector(b"torn"))
            .expect("登记");
        let block_key = block_key_named("stream", 0);
        store
            .store_verdict_block(
                &block_key,
                &TEST_JUDGING_UNITS_FINGERPRINT,
                &block_of(&[clean, clean]),
                &[],
            )
            .expect("存块");
        assert_eq!(
            store
                .store_verdict_block(
                    &block_key,
                    &TEST_JUDGING_UNITS_FINGERPRINT,
                    &block_of(&[torn, clean]),
                    &[StateOffsetInBlock(0)],
                )
                .expect("绿换红照换上"),
            VerdictBlockStoring::ReplacedBecauseOneOfTheTwoIsRed
        );
        assert_eq!(
            store
                .read_verdict_block(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT)
                .expect("读")
                .expect("存着"),
            block_of(&[torn, clean])
        );
        assert_eq!(
            store
                .violations_in_block(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT)
                .expect("读")
                .len(),
            1,
            "记红：违例行一行"
        );
        assert_eq!(
            store
                .store_verdict_block(
                    &block_key,
                    &TEST_JUDGING_UNITS_FINGERPRINT,
                    &block_of(&[clean, clean]),
                    &[],
                )
                .expect("红换绿照换上"),
            VerdictBlockStoring::ReplacedBecauseOneOfTheTwoIsRed
        );
        assert!(
            store
                .violations_in_block(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT)
                .expect("读")
                .is_empty(),
            "红换绿：违例行一行不剩"
        );
    }

    #[test]
    fn storing_the_identical_block_again_is_accepted_and_changes_nothing() {
        let library = TestLibrary::new("identical-block");
        let mut store = library.open();
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

    /// 判定向量编号加宽到 u32：第 257 种登记得上、拿到编号 256，块与违例里存得下、关库重开读得回，之后的接着往下给。
    /// 判别力（`crates/mutations.tsv`）：编号退回一个字节宽（满 256 种就拒），这条红。
    #[test]
    fn the_257th_distinct_verdict_vector_gets_number_256_and_survives_closing_and_reopening() {
        let library = TestLibrary::new("vector-past-one-byte");
        let block_key = block_key_named("stream", 0);
        let the_257th_vector = verdict_vector(b"the 257th kind");
        let (clean, the_257th) = {
            let mut store = library.open();
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
                library.raw_value(
                    &VerdictStoreKey::VerdictVectorContentByNumber(the_257th)
                        .to_key_bytes()
                        .0
                ),
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
        let mut store = library.open();
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

    /// 块值引了一个服务的按编号表里也没有的编号：读成判定块那一张表、那一把键不合布局（错在引它的那一块）。
    /// 判别力（`crates/mutations.tsv`）：从服务取编号时缺了那一号就报按编号表不合布局（报的表与键是按编号表的那一把，不是这一块），这条红。
    #[test]
    fn stored_block_naming_an_unregistered_verdict_vector_number_reads_as_a_layout_mismatch() {
        let library = TestLibrary::new("unregistered-number");
        let mut store = library.open();
        store
            .register_verdict_vector(&verdict_vector(b"clean"))
            .expect("登记");
        let block_key = block_key_named("stream", 0);
        // 绕过 store_verdict_block 直接写一个点名编号 1 的块值
        library.write_raw_rows(vec![(
            VerdictStoreKey::VerdictBlock(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT)
                .to_key_bytes()
                .0,
            vec![0, 0, 0, 0, 0, 0, 0, 1],
        )]);
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

    /// 块值的字节数不是 4（一个判定向量编号的字节数）的整数倍：读成不合布局，不丢掉零头照读、不把几段字节拼成一个编号。
    #[test]
    fn stored_block_value_whose_length_is_not_a_multiple_of_four_bytes_reads_as_a_layout_mismatch()
    {
        let library = TestLibrary::new("block-value-not-a-multiple-of-four-bytes");
        let mut store = library.open();
        store
            .register_verdict_vector(&verdict_vector(b"clean"))
            .expect("登记");
        let block_key = block_key_named("stream", 0);
        // 绕过 store_verdict_block 直接写一个五个字节的块值
        library.write_raw_rows(vec![(
            VerdictStoreKey::VerdictBlock(&block_key, &TEST_JUDGING_UNITS_FINGERPRINT)
                .to_key_bytes()
                .0,
            vec![0, 0, 0, 0, 0],
        )]);
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
        let library = TestLibrary::new("violation-outside");
        let mut store = library.open();
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
        let library = TestLibrary::new("foreign-number");
        let mut store = library.open();
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

    /// 判定日期是东京日期：协调世界时 15 点是东京第二天零点；跑法变量认得两种写法，没设是剪枝；别的值（拿一个从没用过的名字测）照认不得拒。
    #[test]
    fn the_judged_date_is_the_tokyo_date_and_the_run_kind_comes_from_the_run_kind_variable() {
        let at = |seconds: u64| std::time::UNIX_EPOCH + std::time::Duration::from_secs(seconds);
        assert_eq!(tokyo_date_text_of(at(0)), "1970-01-01");
        assert_eq!(tokyo_date_text_of(at(15 * 3600 - 1)), "1970-01-01");
        assert_eq!(tokyo_date_text_of(at(15 * 3600)), "1970-01-02");
        assert_eq!(tokyo_date_text_of(at(951_782_400)), "2000-02-29", "闰日");
        assert_eq!(tokyo_date_text_of(at(1_791_990_000)), "2026-10-15");
        assert_eq!(
            RunKindOfTheJudgement::of_the_variable_value(None),
            RunKindOfTheJudgement::Pruned
        );
        assert_eq!(
            RunKindOfTheJudgement::of_the_variable_value(Some("pruned")),
            RunKindOfTheJudgement::Pruned
        );
        assert_eq!(
            RunKindOfTheJudgement::of_the_variable_value(Some("item_by_item")),
            RunKindOfTheJudgement::ItemByItem
        );
        let refused_value = "run_kind_value_never_used";
        assert!(
            std::panic::catch_unwind(|| {
                RunKindOfTheJudgement::of_the_variable_value(Some(refused_value))
            })
            .is_err(),
            "跑法变量设成 {refused_value:?} 认不得，不当剪枝"
        );
    }

    /// 库里一行的跑法码不是这一版认得的（写成 1；块属性那一行与对拍结论那一行各一份）：问缺哪几种、读块、存块、读对拍结论都照不合布局报错，
    /// 不当没判、不照键现判后换上。判别力（`crates/mutations.tsv`）：问缺哪几种时把认不出的块属性当没判，这条红。
    #[test]
    fn a_stored_row_whose_run_kind_code_is_not_of_this_version_is_refused_as_not_matching_the_layout(
    ) {
        let library = TestLibrary::new("run-kind-code-not-of-this-version");
        let block_key = block_key_named("node", 0);
        let mut store = library.open();
        let clean = store
            .register_verdict_vector(&verdict_vector(b""))
            .expect("登记");
        let stored_block = conclusion_record(block_of(&[clean, clean]), &[]);
        store
            .store_conclusion_block(
                &block_key,
                &ConclusionKind::ImageJudgement,
                &JUDGEMENT_CONCLUSION_VERSION,
                &stored_block,
            )
            .expect("存");
        let attribute_key = VerdictStoreKey::ConclusionBlockAttributes(
            &block_key,
            &ConclusionKind::ImageJudgement,
            &JUDGEMENT_CONCLUSION_VERSION,
        )
        .to_key_bytes();
        let mut attribute_with_a_run_kind_code_of_one = conclusion_block_attribute_bytes(
            &JudgingDevice::Cpu,
            &JudgementCircumstances {
                judged_date: "2026-10-01".to_string(),
                run_kind: RunKindOfTheJudgement::Pruned,
            },
        );
        *attribute_with_a_run_kind_code_of_one
            .last_mut()
            .expect("块属性末尾是跑法码那一个字节") = 1;
        let pairing_key = PairingConclusionKey {
            inputs_digest: [4; SHA256_DIGEST_BYTES],
            one_side_judging_units: [5; SHA256_DIGEST_BYTES],
            other_side_judging_units: [6; SHA256_DIGEST_BYTES],
        };
        let mut pairing_bytes_with_a_run_kind_code_of_one = StoredPairingConclusion {
            conclusion: b"agree states=3".to_vec(),
            agreement: PairingAgreement::BothSidesAgree,
            circumstances: JudgementCircumstances {
                judged_date: "2026-10-15".to_string(),
                run_kind: RunKindOfTheJudgement::Pruned,
            },
        }
        .to_bytes();
        *pairing_bytes_with_a_run_kind_code_of_one
            .last_mut()
            .expect("对拍结论的值末尾是跑法码那一个字节") = 1;
        library.write_raw_rows(vec![
            (attribute_key.0, attribute_with_a_run_kind_code_of_one),
            (
                VerdictStoreKey::PairingConclusion(&pairing_key)
                    .to_key_bytes()
                    .0,
                pairing_bytes_with_a_run_kind_code_of_one,
            ),
        ]);
        let mut reopened = library.open();
        let required = [(ConclusionKind::ImageJudgement, JUDGEMENT_CONCLUSION_VERSION)];
        assert!(
            matches!(
                reopened.conclusions_missing_for_block(&block_key, &required),
                Err(VerdictStoreError::StoredBytesDoNotMatchTheLayout {
                    table: VerdictStoreTable::ConclusionBlockAttributes,
                    ..
                })
            ),
            "问缺哪几种：认不出的跑法码照块属性不合布局报错，不当没判"
        );
        assert!(
            matches!(
                reopened.read_conclusion_block(
                    &block_key,
                    &ConclusionKind::ImageJudgement,
                    &JUDGEMENT_CONCLUSION_VERSION,
                ),
                Err(VerdictStoreError::StoredBytesDoNotMatchTheLayout {
                    table: VerdictStoreTable::ConclusionBlockAttributes,
                    ..
                })
            ),
            "读块：认不出的跑法码照块属性不合布局报错，不当没判"
        );
        assert!(
            matches!(
                reopened.store_conclusion_block(
                    &block_key,
                    &ConclusionKind::ImageJudgement,
                    &JUDGEMENT_CONCLUSION_VERSION,
                    &stored_block,
                ),
                Err(VerdictStoreError::StoredBytesDoNotMatchTheLayout {
                    table: VerdictStoreTable::ConclusionBlockAttributes,
                    ..
                })
            ),
            "存块：库里那一份认不出，报错，不整份换上"
        );
        assert!(
            matches!(
                reopened.read_pairing_conclusion(&pairing_key),
                Err(VerdictStoreError::StoredBytesDoNotMatchTheLayout {
                    table: VerdictStoreTable::PairingConclusions,
                    ..
                })
            ),
            "读对拍结论：认不出的跑法码照不合布局报错，不当没判"
        );
    }

    /// 同一块在两个判法单元指纹下各存一份判定：各自读得回、互不相扰（复用只认同一版本），判定块表列出两行，判法单元指纹表记得两版的描述。
    #[test]
    fn the_same_block_judged_under_two_judging_units_fingerprints_keeps_both_rows_and_reads_only_its_own_fingerprint(
    ) {
        let library = TestLibrary::new("two-judge-versions");
        let mut store = library.open();
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
        // 另一个客户端（另一个进程开同一份库的样子）读到的是同一批行：不经并库、不经导入
        let another_client = library.open();
        assert_eq!(
            another_client
                .stored_verdict_block_keys()
                .expect("列块")
                .len(),
            2,
            "另一个客户端读得到两版两行"
        );
        assert_eq!(
            another_client
                .judging_units_fingerprints()
                .expect("列指纹")
                .len(),
            2,
            "另一个客户端读得到两份组成清单"
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

    /// 测试块每个状态的恢复字：看 journal 那一遍落到 checkpoint_txg 高 32 位非 0 的根（随状态变），不看那一遍落到 txg 4 那条根。
    fn recovery_words_of_the_test_block(states: u32) -> Vec<RecoveryWordsOfState> {
        (0..states)
            .map(|state| {
                RecoveryWordsOfState::of_the_two_passes(
                    RecoveryWordsOfAPass {
                        landed_root: LandedRootWords::of_the_landed_root(
                            InstanceGeneration(2),
                            CheckpointTxg((u64::from(state + 1) << 32) | u64::from(5 + state)),
                        ),
                        outcome: 1,
                        failure_point: 0,
                    },
                    RecoveryWordsOfAPass {
                        landed_root: LandedRootWords::of_the_landed_root(
                            InstanceGeneration(2),
                            CheckpointTxg(4),
                        ),
                        outcome: 1,
                        failure_point: 100 + state,
                    },
                )
            })
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
            recovery_words: Some(recovery_words_of_the_test_block(states)),
        }
    }

    /// 同一块几份结论各存各的：镜像判定存了一把版本的，另一把版本（另一版判定定义）与观察者照样缺着（只排那两种）；
    /// 种类不同、版本字节相同也互不顶替；两个观察者（种类相同、两把版本）各是各的；都存了就一种都不缺，结论块表四行。
    /// 镜像判定不分 CPU 还是 GPU 判的：同一把版本下只有一份（在哪判的只在块属性里，另一条用例判）。
    #[test]
    fn conclusions_of_one_block_are_reused_and_missing_separately_by_kind_and_version() {
        let library = TestLibrary::new("three-kinds");
        let mut store = library.open();
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
        let library = TestLibrary::new("conclusion-read-back");
        let block_key = block_key_named("node", 65536);
        let (record, red) = {
            let mut store = library.open();
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
            let recovery_words_row = library
                .raw_value(
                    &VerdictStoreKey::RecoveryWords(
                        &block_key,
                        &ConclusionKind::ImageJudgement,
                        &ANOTHER_JUDGEMENT_CONCLUSION_VERSION,
                    )
                    .to_key_bytes()
                    .0,
                )
                .expect("恢复字在旁表里");
            assert_eq!(
                recovery_words_row.len(),
                3 * RECOVERY_WORDS_PER_STATE * 4,
                "旁表里每个状态 RECOVERY_WORDS_PER_STATE 个字、每字四个字节"
            );
            (record, red)
        };
        let store = library.open();
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

    /// 审计复核 T01（`records/2026-10-24-singlefs-checker-tier-审计复核.md` checkpoint_txg「存完整 u64」）：恢复字里落到的根
    /// checkpoint_txg 高 32 位非 0（1、`0xFFFF_FFFF`）、低 32 位相同的两个状态，连同一遍没落到根与落到实例代号 `u32::MAX`、checkpoint_txg
    /// `u64::MAX` 那条根的一个状态，存进判定库、关库重开再读回逐字相同；读回的每一遍落到的根照完整的 checkpoint_txg 认得出。
    #[test]
    fn recovery_words_landing_on_checkpoint_txgs_past_thirty_two_bits_read_back_word_for_word() {
        let library = TestLibrary::new("recovery-words-past-thirty-two-bits");
        let block_key = block_key_named("node", 0);
        let pass_landed_on = |instance: u32, checkpoint_txg: u64| RecoveryWordsOfAPass {
            landed_root: LandedRootWords::of_the_landed_root(
                InstanceGeneration(instance),
                CheckpointTxg(checkpoint_txg),
            ),
            outcome: 1,
            failure_point: 0,
        };
        let pass_landed_on_no_root = RecoveryWordsOfAPass {
            landed_root: LandedRootWords::NOT_LANDED,
            outcome: 2,
            failure_point: 3,
        };
        let recovery_words = vec![
            RecoveryWordsOfState::of_the_two_passes(
                pass_landed_on(1, (1 << 32) | 7),
                pass_landed_on(1, 7),
            ),
            RecoveryWordsOfState::of_the_two_passes(
                pass_landed_on(1, (0xFFFF_FFFF << 32) | 7),
                pass_landed_on(1, (1 << 32) | 7),
            ),
            RecoveryWordsOfState::of_the_two_passes(
                pass_landed_on(u32::MAX, u64::MAX),
                pass_landed_on_no_root,
            ),
        ];
        {
            let mut store = library.open();
            let clean = store
                .register_verdict_vector(&verdict_vector(b""))
                .expect("登记");
            let mut record = conclusion_record(block_of(&[clean, clean, clean]), &[]);
            record.recovery_words = Some(recovery_words.clone());
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
        }
        let store = library.open();
        let read_back = store
            .read_conclusion_block(
                &block_key,
                &ConclusionKind::ImageJudgement,
                &ANOTHER_JUDGEMENT_CONCLUSION_VERSION,
            )
            .expect("读")
            .expect("存过")
            .recovery_words
            .expect("恢复字随块存过");
        assert_eq!(
            read_back, recovery_words,
            "恢复字逐字读回，连同高 32 位与标记"
        );
        assert_eq!(
            read_back
                .iter()
                .map(|words| (
                    words.consulted_pass().landed_root.landed_root(),
                    words.ignored_pass().landed_root.landed_root()
                ))
                .collect::<Vec<_>>(),
            vec![
                (
                    Some((InstanceGeneration(1), CheckpointTxg((1 << 32) | 7))),
                    Some((InstanceGeneration(1), CheckpointTxg(7)))
                ),
                (
                    Some((
                        InstanceGeneration(1),
                        CheckpointTxg((0xFFFF_FFFF << 32) | 7)
                    )),
                    Some((InstanceGeneration(1), CheckpointTxg((1 << 32) | 7)))
                ),
                (
                    Some((InstanceGeneration(u32::MAX), CheckpointTxg(u64::MAX))),
                    None
                ),
            ],
            "读回的每一遍落到的根照完整的 checkpoint_txg 认得出，没落到根读回 None"
        );
    }

    /// 4-2a 报告第四节第 7 条后半句：登记成照判不入库的版本，键里带着它的行（结论块连同违例、计数向量、块属性）只留在这个进程里——
    /// 这一份库照读得回、列得出，关库重开就没了；同一块别的版本的结论照常写进库、重开读得回。登记之前写进库的行不动。
    /// 判别力（`crates/mutations.tsv`）：照判不入库的行照旧写进 RocksDB（重开还读得回），这条红。
    #[test]
    fn rows_of_a_version_kept_out_of_the_library_are_read_in_this_process_and_never_written() {
        let library = TestLibrary::new("kept-out-versions");
        let block_key = block_key_named("kept-out", 0);
        let kept_out_version = ConclusionVersion([0x61; SHA256_DIGEST_BYTES]);
        {
            let mut store = library.open();
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
        let reopened = library.open();
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

    /// crates 第 1 条：照判不入库只比键里版本那一截（按表标签定的偏移）。版本那一截是那一把的行，十六种键各自不入库；那一把只出现在
    /// 键里别处（块键里的两个摘要、判定向量的字节、录下的流程的摘要、录入键、对拍的输入摘要、镜像去重键、分片进度的段摘要）的行照入库——
    /// 在整条键上按 32 字节滑着找的话，后一半全判成不入库。
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

    /// 表标签读回它那张表（[`VerdictStoreTable::of_key_tag`] 是 [`VerdictStoreTable::key_tag`] 的逆，认得的标签正好 28 个），
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
        for kind in [ConclusionKind::ImageJudgement, ConclusionKind::Observer] {
            assert_eq!(
                conclusion_kind_key_bytes(&kind).len(),
                CONCLUSION_KIND_KEY_BYTES,
                "{kind:?} 在键里的字节数"
            );
        }
    }

    /// 认得的表标签从 `00` 起连着排到最后一张表，中间不留空号。
    /// 判别力（`crates/mutations.tsv`）：读集见证行那张表的标签读回时认成 `1D`（`1A` 空出来），这条红。
    #[test]
    fn the_table_tags_run_from_zero_to_the_last_table_without_a_gap() {
        let recognized_tags: Vec<u8> = (0..=u8::MAX)
            .filter(|key_tag| VerdictStoreTable::of_key_tag(*key_tag).is_some())
            .collect();
        let expected_tags: Vec<u8> = (0x00..=0x1B).collect();
        assert_eq!(
            recognized_tags, expected_tags,
            "认得的表标签是 00 到 1B 连着的 28 个"
        );
    }

    /// 观察者那一种结论在键里的字节是 `02`（镜像判定是 `01`，两种连着排）。
    /// 判别力（`crates/mutations.tsv`）：观察者的键字节写成 `03`，这条红。
    #[test]
    fn the_observer_conclusion_kind_takes_key_byte_two() {
        assert_eq!(
            conclusion_kind_key_bytes(&ConclusionKind::Observer),
            vec![0x02],
            "观察者那一种结论的键字节"
        );
    }

    /// 块属性里在哪判的：CPU 的字节是 `01`，GPU 的头一个字节是 `02`、后面接卡型、驱动版本与 GPU 判器指纹；两种都读得回原样。
    /// 判别力（`crates/mutations.tsv`）：GPU 的头一个字节写成 `03`，这条红。
    #[test]
    fn judging_device_attribute_bytes_lead_with_one_for_the_cpu_and_two_for_the_gpu() {
        let gpu = JudgingDevice::Gpu {
            card_type: "card".to_string(),
            driver_version: "driver".to_string(),
            gpu_judge_units_fingerprint: [0x5A; SHA256_DIGEST_BYTES],
        };
        let cpu_bytes = JudgingDevice::Cpu.to_attribute_bytes();
        let gpu_bytes = gpu.to_attribute_bytes();
        assert_eq!(cpu_bytes, vec![0x01], "CPU 的块属性字节");
        assert_eq!(gpu_bytes.first(), Some(&0x02), "GPU 的块属性头一个字节");
        assert_eq!(
            JudgingDevice::from_attribute_bytes(&cpu_bytes),
            Some(JudgingDevice::Cpu),
            "CPU 读回原样"
        );
        assert_eq!(
            JudgingDevice::from_attribute_bytes(&gpu_bytes),
            Some(gpu),
            "GPU 读回原样"
        );
    }

    /// 头一次开的一次性库（这个键空间里还没有布局行），开库写下布局行，值是布局号 6（4 字节大端）。
    /// 判别力（`crates/mutations.tsv`）：这一版的布局号写成 5，这条红。
    #[test]
    fn a_library_opened_the_first_time_writes_layout_number_six_into_its_layout_row() {
        let library = TestLibrary::new("layout-number-written");
        assert_eq!(
            library.raw_value(&VerdictStoreKey::LibraryLayout.to_key_bytes().0),
            None,
            "开库之前这个键空间里没有布局行"
        );
        let opened = library.open();
        assert_eq!(
            library.raw_value(&VerdictStoreKey::LibraryLayout.to_key_bytes().0),
            Some(6u32.to_be_bytes().to_vec()),
            "布局行是布局号 6 的 4 字节大端"
        );
        assert!(
            opened.registered_verdict_vectors().is_empty(),
            "头一次开的库一种判定向量都没登记"
        );
    }

    /// 布局行是 `layout_row_bytes` 的一次性库开库拒在 `LibraryLayoutIsNotThisVersion` 上、点名读到的样子（`expected_layout_row`），布局行原样留着。
    fn opening_is_refused_on_the_layout_row_and_leaves_it_as_it_was(
        label: &str,
        layout_row_bytes: Vec<u8>,
        expected_layout_row: LibraryLayoutRowAsRead,
    ) {
        let library = TestLibrary::new(label);
        let layout_key = VerdictStoreKey::LibraryLayout.to_key_bytes().0;
        library.write_raw_rows(vec![(layout_key.clone(), layout_row_bytes.clone())]);
        match VerdictStore::open_the_prune_server(library.service()) {
            Err(VerdictStoreError::LibraryLayoutIsNotThisVersion {
                library: _,
                layout_row,
            }) => assert_eq!(layout_row, expected_layout_row, "点名读到的布局行"),
            Err(other) => panic!("开库应当拒在布局上，却是 {other:?}"),
            Ok(_store) => panic!("布局行不是这一版的库开得了"),
        }
        assert_eq!(
            library.raw_value(&layout_key),
            Some(layout_row_bytes),
            "拒了的库布局行原样留着"
        );
    }

    /// 布局行是别的布局号（5）的库开库拒在布局上。
    /// 判别力：开库不核布局行（`verify_the_library_layout_row` 一律放行），这条红。
    #[test]
    fn a_library_whose_layout_number_is_another_number_is_refused_and_left_as_it_was() {
        opening_is_refused_on_the_layout_row_and_leaves_it_as_it_was(
            "layout-number-another",
            5u32.to_be_bytes().to_vec(),
            LibraryLayoutRowAsRead::LayoutNumber(5),
        );
    }

    /// 布局行读不成布局号（3 个字节）的库开库拒在布局上。
    #[test]
    fn a_library_whose_layout_row_is_not_four_bytes_is_refused_and_left_as_it_was() {
        opening_is_refused_on_the_layout_row_and_leaves_it_as_it_was(
            "layout-row-not-four-bytes",
            vec![0, 0, 6],
            LibraryLayoutRowAsRead::NotFourBytes(vec![0, 0, 6]),
        );
    }

    /// 观察者见证按行存：一个桶里添三条见证（两条共用一组读集，读集行只随头一条写）、另一个桶添一条，每添一条恰好一批；按桶键读回的
    /// 是这个桶自己的那几行、一条见证一行、按见证摘要排，引的读集行与镜像都读得到；重开库照样读得回。
    #[test]
    fn observer_read_set_witness_rows_read_back_one_row_per_witness_from_their_own_bucket() {
        let library = TestLibrary::new("observer-witness-rows");
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
            let mut store = library.open();
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
        let store = library.open();
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
        type Item = Result<StoredEntry, VerdictStoreError>;

        fn next(&mut self) -> Option<Self::Item> {
            let ordinal = self.ordinals.next()?;
            self.rows_taken.set(self.rows_taken.get() + 1);
            Some(Ok((
                (2 * ordinal).to_be_bytes().to_vec().into_boxed_slice(),
                b"library".to_vec().into_boxed_slice(),
            )))
        }
    }

    /// 测试里的「服务那一路读错」：服务不可用的退出码连同一句说明。
    fn a_library_read_error() -> PruneServerFailure {
        PruneServerFailure {
            exit_code: PRUNE_SERVER_UNAVAILABLE_EXIT_CODE,
            description: "前缀扫不成：测试里造的读错".to_string(),
        }
    }

    fn kept_out_row(key_number: u32) -> (Vec<u8>, Vec<u8>) {
        (key_number.to_be_bytes().to_vec(), b"kept out".to_vec())
    }

    /// crates 第 2 条：前缀下的行两路有序流逐行归并，库那一路十万行里只取到交出去的那几行再多看一行，不整段读进来；交出的次序与值：
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

    /// 对照用的合并（库里那一段整段读进一张有序表，再用照判不入库那一段逐行盖）：只留着给对照，交出的次序与值是归并要逐条对得上的。
    fn rows_merged_as_before(
        library_rows: &[(Vec<u8>, Vec<u8>)],
        kept_out_rows: &[(Vec<u8>, Vec<u8>)],
    ) -> Vec<(Vec<u8>, Vec<u8>)> {
        let mut merged: BTreeMap<Vec<u8>, Vec<u8>> = library_rows.iter().cloned().collect();
        merged.extend(kept_out_rows.iter().cloned());
        merged.into_iter().collect()
    }

    /// crates 第 2 条：归并交出的行与对照用的合并逐条相同——两路交错、同一个键两路都有（交照判不入库那一行）、一路空着、两路都空。
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
                        Ok::<StoredEntry, VerdictStoreError>((
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
        let read_error = a_library_read_error();
        let library_rows = vec![
            Ok((
                b"a".to_vec().into_boxed_slice(),
                b"library a".to_vec().into_boxed_slice(),
            )),
            Err(VerdictStoreError::PruneServer(read_error.clone())),
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
                        .map_err(|error| format!("{error:?}"))
                })
                .collect();
        assert_eq!(
            merged,
            vec![
                Ok(b"a".to_vec()),
                Err(format!("{:?}", VerdictStoreError::PruneServer(read_error)))
            ]
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

    /// 对拍结论按（输入摘要，两边判法单元指纹）存、读回逐字节相同；换了哪一边的指纹读不到。
    #[test]
    fn a_pairing_conclusion_reads_back_under_its_inputs_and_both_judging_units_only() {
        let library = TestLibrary::new("pairing-conclusion");
        let mut store = library.open();
        let key = PairingConclusionKey {
            inputs_digest: [1; SHA256_DIGEST_BYTES],
            one_side_judging_units: [2; SHA256_DIGEST_BYTES],
            other_side_judging_units: [3; SHA256_DIGEST_BYTES],
        };
        let conclusion = StoredPairingConclusion {
            conclusion: b"agree states=12".to_vec(),
            agreement: PairingAgreement::SidesDisagree,
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

    /// 同一（块键，种类，版本）下两份都不红而换一份结论（块、计数向量、恢复字任一不同）拒绝，拒了一个字都不写（两份里有一份红的照新的换上：
    /// 记红、撤绿，`crash_amplification` 的 `storing_a_red_conclusion_over_a_green_one_replaces_it_and_back_but_two_different_green_ones_are_refused` 钉）；
    /// 结论相同只换块属性（换一张卡判的）算已存、不写、块属性留先存的那一份；逐项相同的再存放行、不写。
    #[test]
    fn a_different_conclusion_under_a_stored_key_is_refused_while_another_judging_device_is_accepted_and_the_library_is_unchanged(
    ) {
        let library = TestLibrary::new("different-conclusion");
        let mut store = library.open();
        let clean = store
            .register_verdict_vector(&verdict_vector(b""))
            .expect("登记");
        let red = store
            .register_verdict_vector(&verdict_vector(b"red"))
            .expect("登记");
        let block_key = block_key_named("node", 0);
        let record = conclusion_record(block_of(&[clean, red]), &[]);
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
        other_recovery_words.recovery_words =
            Some(recovery_words_of_the_test_block(3)[1..].to_vec());
        let mut other_block = record.clone();
        other_block.block = block_of(&[clean, clean]);
        for (different, what) in [
            (other_counts, "计数向量不同"),
            (other_recovery_words, "恢复字不同"),
            (other_block, "块不同"),
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

    /// 同一条路径再登同一个身份：什么都不写（库的序号不动）；换一个身份照写。
    #[test]
    fn recording_a_path_again_with_the_same_identity_writes_nothing() {
        let library = TestLibrary::new("path-recorded-twice");
        let mut store = library.open();
        store
            .record_path("a.unit_write.1", "f::a.unit_write.1::00")
            .expect("登路径");
        let batches_after_the_first = store.synced_write_tally().batches_written;
        store
            .record_path("a.unit_write.1", "f::a.unit_write.1::00")
            .expect("再登同一个身份");
        assert_eq!(
            store.synced_write_tally().batches_written,
            batches_after_the_first,
            "同一个身份再登不写库"
        );
        store
            .record_path("a.unit_write.1", "f::a.unit_write.1::11")
            .expect("换一个身份");
        assert!(
            store.synced_write_tally().batches_written > batches_after_the_first,
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
        let library = TestLibrary::new("sliced-progress");
        let mut store = library.open();
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
        let library = TestLibrary::new("slice-block-rerecord");
        let mut store = library.open();
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
                | VerdictStoreError::LibraryLayoutIsNotThisVersion { .. }
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

    /// 同一个（块键，种类，版本）下两份都不红而不同的结论：存第二份拒在 `DifferentConclusionUnderAStoredKeyNotDecidedInTheFirstVersion` 上，
    /// 带着两份差在哪（块内哪个状态的判定向量不同，一处一行，不交空的）与两份各在哪判的：两台各判同一块撞上时只读报错就定得出差在哪一样。
    /// 判别力（`crates/mutations.tsv`）：报错不带两份差在哪（差异清单交空的、两份在哪判的都写成 CPU），这条红。
    #[test]
    fn a_refused_different_green_conclusion_names_the_state_that_differs_and_where_each_was_judged()
    {
        let library = TestLibrary::new("different-green-names-the-difference");
        let mut store = library.open();
        let clean = store
            .register_verdict_vector(&verdict_vector(b""))
            .expect("登记");
        let judged_differently = store
            .register_verdict_vector(&verdict_vector(b"judged differently"))
            .expect("登记");
        let block_key = block_key_named("node-with-two-green-conclusions", 0);
        let stored = conclusion_record(block_of(&[clean, clean]), &[]);
        store
            .store_conclusion_block(
                &block_key,
                &ConclusionKind::ImageJudgement,
                &JUDGEMENT_CONCLUSION_VERSION,
                &stored,
            )
            .expect("存头一份");
        let mut requested = conclusion_record(block_of(&[clean, judged_differently]), &[]);
        requested.judged_on = JudgingDevice::Cpu;
        match store.store_conclusion_block(
            &block_key,
            &ConclusionKind::ImageJudgement,
            &JUDGEMENT_CONCLUSION_VERSION,
            &requested,
        ) {
            Err(
                VerdictStoreError::DifferentConclusionUnderAStoredKeyNotDecidedInTheFirstVersion {
                    differences,
                    ..
                },
            ) => {
                assert!(
                    !differences.differences.is_empty(),
                    "两份差在哪一处一行，不交空的"
                );
                assert_eq!(
                    differences.stored_judged_on, stored.judged_on,
                    "库里那一份在哪判的"
                );
                assert_eq!(
                    differences.requested_judged_on,
                    JudgingDevice::Cpu,
                    "要存的那一份在哪判的"
                );
            }
            outcome => panic!("两份都不红而不同要拒、带着差在哪：{outcome:?}"),
        }
    }
}
