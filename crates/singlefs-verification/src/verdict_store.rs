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
//! 只进路径索引与报告；判器的代码不进任何键，判定块与违例另按版本分行（流水线判定行那一把，`crash_amplification::PipelineJudgingCode`），复用只认同一把版本。
//! 块键定长，一个块键不会是另一个块键的前缀，按块键前缀扫违例表只扫到这一块的。
//!
//! 判定向量编号是 u32、大端 4 字节（[`VerdictVectorNumber`]）：「直接加宽到未来都不需要再加宽」，从一个字节一次加宽到 u32，
//! 一个库里最多 2^32 种判定向量。编号宽度不进任何复用键。
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
//! | 一趟的清单 | `10` ‖ 录入键（32 字节：这条用例录入那一组——产品代码与场景——的判法单元指纹与这一趟录入输入的摘要，[`RecordingKey`]） | `crash_amplification::RecordedRunManifest` 或 `crash_library::ItemManifest` 的字节；命中就不再录 |
//! | GPU 判表 | `11` ‖ 摘要（32 字节） | `crash_judge_dispatch::PackedJudgeTables` 的字节 |
//! | 给定状态表 | `12` ‖ 摘要（32 字节） | `crash_amplification::RecordedGivenStatesTable` 的字节（引擎先写进来、再读回来判） |
//!
//! 按段登记的段整域装不进 u64 时分片枚举（C790），每一段一条进度记录（只是加速：真相以块为准，核不过就从 0 起逐块问库）：
//!
//! | 表 | 键 | 值 |
//! |---|---|---|
//! | 分片进度 | `13` ‖ 段的内容摘要（32 字节）‖ 判器（`01` ‖ 判法单元指纹，或 `02` ‖ 结论种类 ‖ 版本） | 已判到的编号 ‖ 最后判完的那一块的起点（各 160 字节，`crash::StateCount1280Bits` 的小端字节） |
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
//! | 对拍结论 | `19` ‖ 输入摘要（32 字节；只由对拍的输入数据定，两边判器与比法的代码不进键） | 对拍的结论（调用方给的字节）、两边比下来一致还是不一致（1 字节，[`PairingAgreement`]；不一致的下一趟现判）、判定日期与跑法 |
//!
//! 对拍（GPU 一侧与 CPU 参照算同一份输入、比结果）的输入另用一张表，判的不是崩溃状态的对拍经它过库（`crash_library::pairing_input_through_the_pairing_library`）：
//!
//! | 表 | 键 | 值 |
//! |---|---|---|
//! | 对拍输入 | `1C` ‖ 摘要（32 字节） | 对拍的输入（调用方给的字节；摘要就是这份字节的 SHA-256）。录入段写进来，判段按清单里的摘要读回、核过摘要再交给两边 |
//!
//! 配了 GPU 的判段不在自己进程里持卡判，把要判的块写成待判行、等 GPU 批处理服务（`gpu-verification-server`）判完写回，另用两张表
//! （协议里的 `gpu-pending/<流键>/<块号>` 与 `gpu-rejudged/<入池序号>/<流键>/<块号>` 两族）：
//!
//! | 表 | 键 | 值 |
//! |---|---|---|
//! | 待判行 | `1D` ‖ 流键（32 字节，[`GpuPendingStreamKey`]）‖ 块号（8 字节，计划里全部点的块按次序编号） | [`GpuPendingRow`]：录入表那一块的键（块键）、录下的流程的摘要、判表摘要、块的态序号区间、结论版本键、强制现判、入池序号 |
//! | 强制现判凭据 | `1E` ‖ 入池序号（8 字节）‖ 流键（32 字节）‖ 块号（8 字节） | [`GpuRejudgedRow`]：判它的服务持有者身份、结论摘要 |
//!
//! 「判完」不存：服务与判段都照判段那两个口径在库上现算（复用口径 `crash_amplification::ConclusionsStoredUnder::is_stored_for_reuse`、
//! 续点口径 `crash_amplification::ConclusionsStoredUnder::is_judged_with_the_slots`；服务调 `crash_amplification::judging_of_a_gpu_pending_block_in_the_library`）。
//! 服务写结论的那一帧里删掉待判行；判段写待判行时不在才写，在而不强制、这一份要强制现判就改成强制
//! （[`VerdictStore::put_the_gpu_pending_row_unless_present_or_raise_it_to_forced`]）。服务写的那一帧的结论行照判段存结论块的同一套规矩编
//! （[`VerdictStore::conclusion_block_rows_for_one_frame_with_the_slots_the_judge_handed_back`]），不经这个进程写库。
//! 判段的库在临时 prune-server 里时（一次性库、或设着临时服务标记的全仓那一份，[`VerdictStore::temporary_service_of_the_library`]），
//! 判段先在真服务里登记那份临时库（`crate::crash_library::register_the_temporary_library_with_the_real_service`），GPU 批处理服务照登记连过去：
//! 三种帧里的键、删的待判行与凭据的键都是那一份库接上它的键空间之后的整把键（[`key_bytes_of_the_gpu_pending_row_in`] 一类），结论只写进那一份。
//!
//! 配了 GPU 的判段第 ② 段核对同样不在自己进程里持卡核，把要核的块写成核对待判行、等 GPU 批处理服务核完写回，另用一张表：
//!
//! | 表 | 键 | 值 |
//! |---|---|---|
//! | 核对待判行 | `1F` ‖ 流键（32 字节，[`FactsVerifierPendingStreamKey`]）‖ 块号（8 字节，计划里全部点的块按次序编号） | [`FactsVerifierPendingRow`]：录入表那一块的键（块键）、录下的流程的摘要、事实表摘要、块的态序号区间、入池序号 |
//!
//! 「核完」同样不存：判段与服务都照核对表（`08`）里这一块在 `crash_amplification::facts_verifier_row_key`（只取事实表摘要，核对器的代码不进）下有没有那一行
//! 现算（服务调 `crash_amplification::facts_verifier_pending_block_is_verified_in_the_library`）。判段写行时不在才写
//! （[`VerdictStore::put_the_facts_verifier_pending_row_unless_present`]）；服务写核对行的那一帧里删掉核对待判行，那一帧的行照
//! [`VerdictStore::facts_verifier_rows_for_one_frame`] 编（带两个守卫：核对行编之前读到的样子、核对待判行读到的样子），不经这个进程写库。
//!
//! 给定状态表与观察者交回的盘配了 GPU 而调用方没交卡时，判段同样不在自己进程里持卡判：把要判的那一张打包判表（表 `11`，判段现建现写）
//! 写成判表待判行，等 GPU 批处理服务照判表与内核源码在卡上判全部状态、把每个状态的结论字写回；服务只判、不解字，解字与存结论在判段，
//! 照判段在本进程上卡那一支同一套。两种调用方共用这一族，另用两张表：
//!
//! | 表 | 键 | 值 |
//! |---|---|---|
//! | 判表待判行 | `20` ‖ 判表摘要（32 字节）‖ 容器容量旋钮摘要（32 字节）（[`PackedTablesKey`]） | [`PackedTablesPendingRow`]：容器容量旋钮三栏（墓碑、谱系、退役，各一个有无字节加 8 字节大端的每片记录数）、态数（8 字节大端，等于判表的态数）、强制现判（1 字节）、入池序号（8 字节大端） |
//! | 判表结论字 | `21` ‖ 同上两段 | `PackedTablesVerdictWordsRow`：态数（8 字节小端）‖ 逐态 `crash_judge_gpu::VERDICT_WORDS` 个字（各 4 字节小端，态序号 0 起连着）‖ 写它的设备属性（[`JudgingDevice`] 的属性字节） |
//!
//! 键只由判的输入数据定：同一张判表、同一组旋钮判出的结论字全库一份，两种调用方谁先判谁写、后来的照键复用；内核源码与 GPU 判器的代码不进键，
//! 判器改了不作废已判的结论字，判法改了对不对靠对拍与复用哨兵抓。判段要判一张判表之前先读表 `21`，
//! 有这一行而不强制现判就直接用它，不写待判行。服务写表 `21` 那一行与删表 `20` 那一行同一帧（两个守卫：表 `21` 那一行编之前读到的样子、
//! 服务读到的那一份待判行；`VerdictStore::packed_tables_verdict_words_rows_for_one_frame` 编，不经这个进程写库）。
//! 强制现判（复用哨兵抽中、要现判的那一份）的凭据不另开一张表：判段写下（或抬成、或见到已是）强制的那一行待判行之后，读到表 `20` 这一把键
//! 不在、表 `21` 这一把键在，才算服务现判过。服务只在写表 `21` 的那一帧里删强制的待判行（守卫核删的是读到的那一份：抬成强制之后，
//! 服务早先读到的不强制那一份删不掉），所以那一行表 `21` 写在这一次写待判行之后；表 `21` 早就在而表 `20` 还在的，强制的照等。
//! 不强制的，表 `21` 有了就算判完（表 `20` 留给服务带守卫删）。表 `20` 不在而表 `21` 也不在的，判段照不在才写补写一次
//! （[`VerdictStore::put_the_packed_tables_pending_row_unless_present_or_raise_it_to_forced`]）。
//!
//! 结论种类 = `01`（镜像判定：GPU 判器与 CPU 判器判出的都是这一种，在哪判的只进块属性）、`02`（观察者），
//! 别的字节读回认不出。版本是 `crash_identity::ConclusionVersion`：判器判出的那几种由结论种类与判器引到的被检测产品代码定
//! （`ConclusionVersion::of_the_crash_image_judgement` 一族），判器自己的代码（GPU 判器、CPU 判器、池级 checker、记录核对器）不进；
//! 观察者那一种由它的判定项标识、它引到的被检测产品代码与另读的输入定（`crash_amplification::ObserverVersions`）。
//! 一块一种结论记红就是它的违例表里有行：复用只认不红的（记着红的下一趟现判，判完照新结论换上，[`VerdictBlockStoring::ReplacedBecauseOneOfTheTwoIsRed`]；
//! 「记红、撤绿、下趟现判」）。观察者不带名字：是哪一个观察者由版本担（它那一组判法单元按入口 fn 定，同一份文件里两个观察者
//! 两把键；4-2a 报告第四节第 5 条），名字只进组名与行里报的名字。
//! 一块一种结论的结论块、违例、恢复字、计数向量与块属性同一批原子地写、一起读回。开库只读回按编号的判定向量表，一行都不写
//! （[`VerdictStore::open_the_prune_server`]）。
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
/// 判表待判行的旋钮栏是这三棵容器树的旋钮（[`PackedTablesPendingRow::container_capacity_knobs`]）：GPU 批处理服务造行、核行照这里的路径叫它，
/// 不为这一个类型另依赖池级 checker。
pub use singlefs_checker::walk::ContainerCapacityKnobs;
use singlefs_checker::walk::{
    LineageContainerCapacityKnob, RetirementContainerCapacityKnob, TombstoneContainerCapacityKnob,
};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};

mod conclusion_block_rows;
mod conclusion_blocks;
mod facts_verifier_rows;
mod gpu_pending_rows;
mod judgement_circumstances;
mod library_opening;
mod observer_rows;
mod packed_tables_rows;
mod pairing_rows;
mod prune_server_storage;
mod recorded_stage_rows;
mod sliced_segment_progress;
mod verdict_blocks_and_vectors;

pub use conclusion_block_rows::*;
pub use conclusion_blocks::*;
pub use facts_verifier_rows::*;
pub use gpu_pending_rows::*;
pub use judgement_circumstances::*;
pub use observer_rows::*;
pub use packed_tables_rows::*;
pub use pairing_rows::*;
pub use prune_server_storage::*;
pub use sliced_segment_progress::*;

#[cfg(test)]
use tests::{
    block_key_named, block_key_of, block_of, conclusion_record, count_vector_of,
    recovery_words_of_the_test_block, verdict_vector, TestLibrary,
    ANOTHER_JUDGEMENT_CONCLUSION_VERSION, FINGERPRINT_BYTE, JUDGEMENT_CONCLUSION_VERSION,
    OBSERVER_CONCLUSION_VERSION, OTHER_JUDGING_UNITS_FINGERPRINT, PLAN_HASH_BYTE,
    TEST_JUDGING_UNITS_FINGERPRINT,
};

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

/// 录入键：录下的写流摘要与场景标识（项名或流名）拼起来的 SHA-256（`crash_library::recording_key_of_the_scenario_and_the_inputs`；
/// 代码一个单元都不进）。录入段的清单按它登，命中就不再录；判段照同一份输入现算它，凭它读清单。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RecordingKey(pub [u8; SHA256_DIGEST_BYTES]);

/// 待判行的流键：一份录下的流程与它的打包判表照一把 GPU 判器组键判、结论记在一把结论版本下的那一条流（`crash_amplification::gpu_pending_stream_key`）。
/// 块号只在一条流里有意义：同一份判表、不同的流程、判器或版本是几条流，待判行互不相盖。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GpuPendingStreamKey(pub [u8; SHA256_DIGEST_BYTES]);

/// 库里一把键接上 `key_space` 那一段键空间之后、服务里的整把键字节（[`LibraryKeySpace::key_prefix`]；全仓那一份键原样）。
fn key_bytes_in_the_key_space(key_space: &LibraryKeySpace, key_bytes: &[u8]) -> Vec<u8> {
    let key_prefix = key_space.key_prefix();
    let mut namespaced = Vec::with_capacity(key_prefix.len() + key_bytes.len());
    namespaced.extend_from_slice(key_prefix);
    namespaced.extend_from_slice(key_bytes);
    namespaced
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
    /// 录入段一趟的清单：键是录入键（[`RecordingKey`]：这条用例录入那一组判法单元指纹与录入输入的摘要，不带流名），值是这一趟录下的几样各自的摘要
    /// （`crash_amplification::RecordedRunManifest` 或 `crash_library::ItemManifest`）。判段照同一份输入现算录入键找到它。
    RecordedRunManifests,
    /// 录入段打包好的 GPU 判表（`crash_judge_dispatch::PackedJudgeTables`）：键是它的摘要，值是它的字节。判段在卡上判时只从这里读。
    PackedJudgeTables,
    /// 给定状态表（`crash_amplification::RecordedGivenStatesTable`）：键是它的摘要，值是它的字节。引擎先写进来、再从这里读回判。
    GivenStatesTables,
    /// 对拍输入（调用方给的字节）：键是它的摘要，值是它的字节。录入段写进来，判段按清单里的摘要读回。
    PairingInputs,
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
    /// 待判行（[`GpuPendingRow`]）：键是流键与块号，判段写、GPU 批处理服务读，服务写结论的那一帧里删掉。
    GpuPendingRows,
    /// 强制现判凭据（[`GpuRejudgedRow`]）：键是入池序号、流键与块号，服务现判强制现判那一块时与结论同一帧写，判段照哨兵的判据核。
    GpuRejudgedRows,
    /// 核对待判行（[`FactsVerifierPendingRow`]）：键是流键与块号，判段第 ② 段写、GPU 批处理服务读，服务写核对行的那一帧里删掉。
    FactsVerifierPendingRows,
    /// 判表待判行（[`PackedTablesPendingRow`]）：键是 [`PackedTablesKey`]，给定状态表与观察者交回的盘没交卡的判段写、GPU 批处理服务读，
    /// 服务写判表结论字的那一帧里删掉。
    PackedTablesPendingRows,
    /// 判表结论字（`PackedTablesVerdictWordsRow`）：键是 [`PackedTablesKey`]，GPU 批处理服务写、判段读回解字；同一把键全库一份、照键复用。
    PackedTablesVerdictWords,
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
            Self::PairingInputs => 0x1C,
            Self::GpuPendingRows => 0x1D,
            Self::GpuRejudgedRows => 0x1E,
            Self::FactsVerifierPendingRows => 0x1F,
            Self::PackedTablesPendingRows => 0x20,
            Self::PackedTablesVerdictWords => 0x21,
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
            | Self::ObserverReadSets
            | Self::PairingInputs
            | Self::GpuPendingRows
            | Self::GpuRejudgedRows
            | Self::FactsVerifierPendingRows
            | Self::PackedTablesPendingRows
            | Self::PackedTablesVerdictWords => EmptyValueMeaning::DeletionMarker,
        }
    }

    /// [`Self::key_tag`] 的逆：一个键的第一个字节是哪张表的标签；不是任何一张表的标签时是 `None`。
    fn of_key_tag(key_tag: u8) -> Option<Self> {
        match key_tag {
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
            0x1C => Some(Self::PairingInputs),
            0x1D => Some(Self::GpuPendingRows),
            0x1E => Some(Self::GpuRejudgedRows),
            0x1F => Some(Self::FactsVerifierPendingRows),
            0x20 => Some(Self::PackedTablesPendingRows),
            0x21 => Some(Self::PackedTablesVerdictWords),
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
        VerdictStoreTable::VerdictVectorNumberByContent
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
        | VerdictStoreTable::ObserverReadSets
        | VerdictStoreTable::PairingInputs
        | VerdictStoreTable::GpuPendingRows
        | VerdictStoreTable::GpuRejudgedRows
        | VerdictStoreTable::FactsVerifierPendingRows
        | VerdictStoreTable::PackedTablesPendingRows
        | VerdictStoreTable::PackedTablesVerdictWords
        | VerdictStoreTable::PairingConclusions => &[],
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
    /// 对拍输入的键：它的摘要。
    PairingInput(&'key [u8; 32]),
    /// 核对表里一块的键：块键 + 核对代码摘要。
    VerifierBlock(&'key VerdictBlockKey, &'key [u8; 32]),
    /// 判法单元组成清单表里一把键的键：判法单元指纹。
    JudgingUnitsComposition(&'key [u8; SHA256_DIGEST_BYTES]),
    /// 对拍结论表里一条的键。
    PairingConclusion(&'key PairingConclusionKey),
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
    /// 一行待判行的键：流键 + 块号（8 字节大端）。
    GpuPendingRow(&'key GpuPendingStreamKey, u64),
    /// 一行强制现判凭据的键：入池序号（8 字节大端）+ 流键 + 块号（8 字节大端）。
    GpuRejudgedRow(u64, &'key GpuPendingStreamKey, u64),
    /// 一行核对待判行的键：流键 + 块号（8 字节大端）。
    FactsVerifierPendingRow(&'key FactsVerifierPendingStreamKey, u64),
    /// 一行判表待判行的键：判表摘要 + 旋钮摘要（[`PackedTablesKey`]）。
    PackedTablesPendingRow(&'key PackedTablesKey),
    /// 一行判表结论字的键：同判表待判行的两段。
    PackedTablesVerdictWords(&'key PackedTablesKey),
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
            Self::PairingInput(digest) => (VerdictStoreTable::PairingInputs, digest.to_vec()),
            Self::VerifierBlock(block_key, verifier_digest) => {
                let mut key_bytes_after_tag = block_key.to_block_key_bytes();
                key_bytes_after_tag.extend_from_slice(&verifier_digest[..]);
                (VerdictStoreTable::VerifierVerdicts, key_bytes_after_tag)
            }
            Self::JudgingUnitsComposition(fingerprint) => (
                VerdictStoreTable::JudgingUnitsCompositions,
                fingerprint.to_vec(),
            ),
            Self::PairingConclusion(pairing_key) => (
                VerdictStoreTable::PairingConclusions,
                pairing_key.inputs_digest.to_vec(),
            ),
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
            Self::GpuPendingRow(stream, block_number) => {
                let mut key_bytes_after_tag = stream.0.to_vec();
                key_bytes_after_tag.extend_from_slice(&block_number.to_be_bytes());
                (VerdictStoreTable::GpuPendingRows, key_bytes_after_tag)
            }
            Self::GpuRejudgedRow(pool_entry_number, stream, block_number) => {
                let mut key_bytes_after_tag = pool_entry_number.to_be_bytes().to_vec();
                key_bytes_after_tag.extend_from_slice(&stream.0);
                key_bytes_after_tag.extend_from_slice(&block_number.to_be_bytes());
                (VerdictStoreTable::GpuRejudgedRows, key_bytes_after_tag)
            }
            Self::FactsVerifierPendingRow(stream, block_number) => {
                let mut key_bytes_after_tag = stream.0.to_vec();
                key_bytes_after_tag.extend_from_slice(&block_number.to_be_bytes());
                (
                    VerdictStoreTable::FactsVerifierPendingRows,
                    key_bytes_after_tag,
                )
            }
            Self::PackedTablesPendingRow(key) => (
                VerdictStoreTable::PackedTablesPendingRows,
                key.to_bytes_after_the_tag(),
            ),
            Self::PackedTablesVerdictWords(key) => (
                VerdictStoreTable::PackedTablesVerdictWords,
                key.to_bytes_after_the_tag(),
            ),
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

    /// 同 [`Self::put`]，键与值是调用方刚拼好、之后不用的：收下就放进这一批，不另拷一份（满块的恢复字约 3 MiB）。
    fn put_owned(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.rows.push(LibraryWriteOfARow::Put { key, value });
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
    /// 判段照现判、判完照存块的规矩写回；每一趟抽一次、添进来（[`Self::also_force_these_conclusion_blocks_to_be_judged_again`]），
    /// 用完撤掉自己那几块（[`Self::stop_forcing_these_conclusion_blocks_to_be_judged_again`]）。
    conclusion_blocks_forced_to_be_judged_again: BTreeSet<ConclusionOfABlockUnderAKey>,
    /// 开库之后在只拿 `&self` 的读里从 prune-server 取来的判定向量（编号从按编号的内存表末尾起连续）：读的方法交回判定向量的借用，
    /// 取来的这几种放进 `Box::leak` 的堆里、进程退出才放（一个进程见到的判定向量种数有限，几百上千种、每种几十字节）；
    /// 下一次拿 `&mut self` 时（登记）并进内存表（[`Self::take_in_the_verdict_vectors_fetched_after_opening`]）。
    verdict_vectors_fetched_after_opening:
        std::sync::Mutex<BTreeMap<u32, &'static EncodedVerdictVector>>,
}

/// 一块一种结论在库里的键：块键、结论种类、版本。
pub type ConclusionOfABlockUnderAKey = (VerdictBlockKey, ConclusionKind, ConclusionVersion);

/// 一块一种结论一把版本在库里存没存、红不红（[`VerdictStore::conclusion_blocks_stored_and_red`] 成批问的答案）：
/// 存没存看块属性那一行（与结论块同一批写），红不红看违例表里这一块有没有行。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConclusionBlockInTheLibrary {
    NotStored,
    StoredGreen,
    StoredRed,
}

/// 成批读观察者录入块那张表时一帧至多问几行：录入段录的观察者那一行一块一份、值随块长涨（一块 2^16 个状态时几百 KiB），
/// 读回的值攒在内存里直到这一批判完，按这个数一批批读、判完就丢。
const OBSERVER_BLOCK_ROWS_READ_TOGETHER: usize = 64;

impl VerdictStore {
    /// 服务那一端连同从它读回的判定向量表装成一份：这个进程里的账都从空起，照判不入库的版本先登上判器引到的被检测产品代码拒算时
    /// 那几把结论版本（`crate::crash_identity::conclusion_versions_kept_out_of_the_library_in_this_process`，算得出键时一把都没有）。
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
            rows_kept_out_of_the_library: std::sync::Mutex::new(RowsKeptOutOfTheLibrary {
                versions: crate::crash_identity::conclusion_versions_kept_out_of_the_library_in_this_process()
                    .into_iter()
                    .collect(),
                rows: BTreeMap::new(),
            }),
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

    /// 这一份是不是一次性库（[`LibraryKeySpace::DisposableLibrary`]：临时服务里的一段名字空间，全仓那一份的读者看不见它的行）。
    #[must_use]
    pub fn is_a_disposable_library(&self) -> bool {
        match self.service.key_space {
            LibraryKeySpace::TheOneLibrary => false,
            LibraryKeySpace::DisposableLibrary(_) => true,
        }
    }

    /// 这一份在服务里的键空间（GPU 批处理服务编帧、删待判行的整把键照它接，[`key_bytes_of_the_gpu_pending_row_in`] 一类）。
    #[must_use]
    pub fn key_space(&self) -> &LibraryKeySpace {
        &self.service.key_space
    }

    /// 这一份在不在临时 prune-server 里、那份临时服务连哪、用哪段键空间（[`crate::crash_library::temporary_service_of_a_library`]）：
    /// 一次性库在；全仓那一份在进程环境设着临时服务的标记时在（前缀是空的）；都不是交 `None`（真库）。
    /// 判段配了 GPU、库在临时服务里时照它在真服务里登记这份临时库（`crate::crash_library::register_the_temporary_library_with_the_real_service`）。
    #[must_use]
    pub fn temporary_service_of_the_library(
        &self,
    ) -> Option<crate::crash_library::TemporaryServiceOfTheLibrary> {
        crate::crash_library::temporary_service_of_a_library(
            &self.service.route,
            &self.service.key_space,
        )
    }

    /// 判段写成待判行、GPU 批处理服务替这一趟判完写进库的几块一种结论，照交来存的次序记进 [`Self::conclusion_blocks_offered_since`]
    /// 那一份账（服务写的结论这个进程没经手，复用哨兵照这一份账判「这一趟现判了没有」）。强制现判的那一块由调用方核过强制现判凭据
    /// （`crash_amplification` 的待判行那一路）才记。
    pub fn note_the_conclusion_blocks_the_gpu_verification_server_judged_for_this_run(
        &mut self,
        blocks: impl IntoIterator<Item = ConclusionOfABlockUnderAKey>,
    ) {
        self.conclusion_blocks_offered_in_this_process
            .extend(blocks);
    }

    /// 复用哨兵一趟抽中强制现判的那几块：添进强制现判表，别的趟添的照留。同一条连接上几趟判段同时跑（每次读写现取现放，
    /// `crate::crash_library::LibraryTakenAtEachReadOrWrite`），各趟只添、只撤自己的（[`Self::stop_forcing_these_conclusion_blocks_to_be_judged_again`]）。
    pub fn also_force_these_conclusion_blocks_to_be_judged_again(
        &mut self,
        blocks: impl IntoIterator<Item = ConclusionOfABlockUnderAKey>,
    ) {
        self.conclusion_blocks_forced_to_be_judged_again
            .extend(blocks);
    }

    /// 一趟用完自己抽中的强制现判那几块：从强制现判表里撤掉，别的趟添的照留。
    pub fn stop_forcing_these_conclusion_blocks_to_be_judged_again(
        &mut self,
        blocks: impl IntoIterator<Item = ConclusionOfABlockUnderAKey>,
    ) {
        for block in blocks {
            self.conclusion_blocks_forced_to_be_judged_again
                .remove(&block);
        }
    }

    /// 这一块这一种结论这一把版本是不是在强制现判表里（[`Self::also_force_these_conclusion_blocks_to_be_judged_again`]）。
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

    /// 只拿 `&self` 写一行（[`Self::put_one_synced`] 要 `&mut self`；待判行一族在判段与服务里都只握着读的那一把）。
    fn put_one_row_through_the_batch(
        &self,
        key_bytes: &[u8],
        value_bytes: &[u8],
    ) -> Result<(), VerdictStoreError> {
        let mut batch = LibraryWriteBatch::default();
        batch.put(key_bytes, value_bytes);
        self.write_batch_keeping_the_kept_out_rows_in_this_process(batch)
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
}

impl VerdictStore {
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
}

#[cfg(test)]
mod tests;
