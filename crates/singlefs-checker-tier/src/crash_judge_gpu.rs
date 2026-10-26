//! GPU 判器（用户 2026-09-28 定「GPU 判器现在实现、整套做完」）：把 `crate::crash_judge_tables` 摊出来的表整份上卡，
//! 每个调用判一个崩溃状态——恢复（看 journal 与不看 journal 各一遍：择系统配置、择根、扫环重放、走到第一个文件；看 journal 那一遍判退看冷走读、
//! 只读挂载的打开、可写挂载的建上一版连同实例表三样读者的并集，C755）、
//! 七类 oracle、记录核对器两条判据，判法与 CPU 判器（`crate::crash::judge_crash_image`）逐条相同；GPU 与 CPU 逐状态比是它的判别力自证。
//! 内核分六份源码：`crash_judge_common.wgsl`（各内核共用的解析与查表）、`crash_judge_recovery.wgsl`（恢复、oracle、记录核对器）、
//! `crash_judge_checker_common.wgsl`（池级 checker 两个内核共用的读法与交接段）、`crash_judge_checker.wgsl`（走读内核：第一、二批不变量，
//! 连同里程碑三二·1 那一批里的十五条、I-9.27 与 I-3.5，与里程碑三·3 墓碑树、谱系树条目身份那两条 I-9.17、I-9.23）、`crash_judge_checker_scan.wgsl`
//! （扫描内核：第三批、I-3.12、I-5.5、I-9.28，与里程碑三·3 删除线的删除态扫描、I-9.6 墓碑那一半、墓碑与谱系那一遍，里程碑四自建 GC 的 I-3.16；
//! 同一份源码里第二个入口是引用与快照那一截：I-3.9、I-9.14 与里程碑四只读快照那几条，再是自建 GC 的 I-4.4、I-5.6、I-3.15、I-1.14，
//! [`CHECKER_REFERENCES_AND_SNAPSHOTS_ENTRY_POINT`]，另编一条管线、接在扫描那一趟之后派）、`crash_judge_checker_scan_rebuild.wgsl`
//! （接在扫描内核源码之后、同一份源码里的第三个入口 [`CHECKER_SCAN_REBUILD_ENTRY_POINT`]：I-1.13 扫描重建，另编一条管线、接在引用与快照那一截之后派）；
//! 判不了的状态（表越过内核容量）置最高位，
//! 放量路径（`crate::crash_amplification` 收卡上结论那一段）遇到它就让整趟停下、报哪张表满了，配置开了 GPU 不退回 CPU 判器。
//!
//! 一个状态的结论 [`VERDICT_WORDS`] 个 u32：第 0 字是红位（位 0..6 看 journal 那一遍 oracle 的七类、位 8..14 不看那一遍、
//! 位 16 根在而记录都不在、位 17 恢复自称的状态少单元、位 31 判不了）；第 1 字判不了的原因（判得了时低 16 位是看 journal 那一遍
//! 重放停在哪一道、高 16 位是记录数，诊断用）；第 2 / 3 字看 journal 那一遍落到的
//! 实例代号与 checkpoint_txg 低 32 位（高 32 位与落没落到根的标记在第 26 字起那几个字）；第 4 / 5 字它的走读结局与失败点（失败点码是 [`gpu_failure_point_code`] 那一张表）；第 6 / 7 字不看那一遍的结局与失败点；
//! 第 8..12 字池级 checker 的违例位（[`POOL_CHECKER_VIOLATED_WORDS`] 个字，128 位）；第 12 字池级 checker 走到第几步；第 13 字它判不了的原因；
//! 第 14 / 15 字不看 journal 那一遍落到的实例代号与 checkpoint_txg 低 32 位（判决 `research/prompts/m3-crash-enumeration-merge-r2-main-verification.md`
//! 第二节 L1-6）；第 16..20 字池级 checker 逐条不变量的「判到」位（成立或违例，与 CPU 计数里「评估过」同一口径；同一判决 L1-4）；
//! 第 20..24 字池级 checker 判 I-7.4 时违例细节里点名的那一条被抛弃的根（点没点名、实例代号、checkpoint_txg 低 / 高 32 位，
//! [`ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_WORD`]）；第 24 字扫描重建那一截判 I-1.13 的结局（[`SCAN_REBUILD_OUTCOME_WORD`]：成立、违例、
//! 报不适用的哪一种理由，[`SCAN_REBUILD_OUTCOMES`]）；第 25 字逐盘自证过的系统配置槽数（[`SELF_VERIFIED_SYSTEM_CONFIGURATION_SLOTS_OF_EACH_DEVICE_WORD`]，
//! 两槽全废盘面观察者在判段对基镜像比的那一样）；第 26..30 字两遍落到的根另两个字：看 journal 那一遍落没落到根的标记与 checkpoint_txg 高 32 位、
//! 不看那一遍的同两字（[`CONSULTED_PASS_LANDED_MARKER_WORD`]；用户 2026-10-24 定 checkpoint_txg「存完整 u64」，`records/2026-10-24-singlefs-checker-tier-审计复核.md`
//! 的 T01；第 2 / 3、14 / 15 字没落到根时照旧是 `u32::MAX`，落没落到看标记那一字）。
//! 这几个字号、打包表的表号与段表宽、段的展开方式码、失败点码、扫描重建的结局码在内核里的常量由 [`kernel_prelude_wgsl_constants`] 从 Rust 生成、
//! 接在每个内核源码前面，不在 WGSL 里另写一份。对拍比的是 [`CrossCheckWords`]：GPU 与 CPU 两侧各由一个构造函数换出来，
//! 经 [`cross_check_words_agree_in_the_cross_check`] 逐字段比（内核不置位、对拍不比的表 [`POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS`]
//! 补上扫描重建内核之后是空的，每一位都比）。
//! 判定表的键里判器那一半是 GPU 判器那一组判法单元的键（`crate::crash_identity::judging_units_key`，入口登记在
//! `crate::judging_units_entries`）：拼内核源码的函数、`.wgsl` 里从 `@compute` 入口够得着的函数、派活回读归位那一段都在里面，
//! 改一个字（注释除外）旧行留着当历史，新行另起。
//!
//! 这一份只放判法：内核原文、表怎么打包、派活参数与结论各是哪几个字、草稿区每个状态多大。怎么上卡、一次派活带几个状态、
//! 几张卡怎么用，在 `crate::crash_judge_dispatch`（用户 2026-09-29 定「派活和判法拆出来最好」）；这一份不许引它。
use singlefs_checker::image::IMPLEMENTED_INVARIANTS;
use singlefs_checker::walk::ContainerCapacityKnobs;

use crate::crash::{
    CrashPointReplayOracleViolationKind, CRASH_POINT_REPLAY_RED_PASS_JOURNAL_CONSULTED_ORACLE,
    CRASH_POINT_REPLAY_RED_PASS_JOURNAL_IGNORED_ORACLE, CRASH_POINT_REPLAY_RED_PASS_POOL_CHECKER,
    CRASH_POINT_REPLAY_RED_PASS_RECORD_CHECKER, RECORD_CHECKER_CLAIMED_STATE_MISSING_UNIT,
    RECORD_CHECKER_ROOT_WITHOUT_RECORD,
};
use crate::crash_facts::{
    segment_expansion_code_wgsl_constants, MAXIMUM_ENUMERATED_WRITES, PERSISTED_MASK_WORDS,
};
use crate::crash_judge_conclusions::{
    recovery_words_of_the_cross_check_words, AbandonedRootNamedByThePoolChecker,
    RecoveryWordsOfState,
};
use crate::crash_judge_tables::{
    JudgeTablesWords, CLAIMED_VERSION_LANDED_ON_BY_THE_CONSULTED_RECOVERY,
    CLAIMED_VERSION_NEWEST_ROOT_WRITTEN_WITHOUT_A_ROOT,
    CLAIMED_VERSION_NEWEST_ROOT_WRITTEN_WITH_A_ROOT, CLAIMED_VERSION_WORDS,
    COMBINATION_WORDS_OF_A_VERSION, JUDGE_TABLE_COUNT, SEGMENT_WORDS, VERSION_WORDS,
    VERSION_WORD_COMBINATION_PAST_THE_SIXTY_FOURTH_WRITE,
};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};

use crate::crash_judge_conclusions::invariant_names_are_equal;
/// 不挂 `gpu` 特性的那一半（失败点码表、红位、走读结局的码、对拍比的那几个字与它的 CPU 一侧）在 `crate::crash_judge_conclusions`
/// （崩溃点重放合并第三轮判决 J8：改结论的几处搬进判器一侧；失败点码表挪出 `gpu` 特性，没编进卡的 CPU 结论也带恢复字），
/// 这里照旧转出去：派活、对拍用例与内核常量的生成照这条路径引。
pub use crate::crash_judge_conclusions::{
    cross_check_words_agree_in_the_cross_check, cross_check_words_compared_by_the_cross_check,
    cross_check_words_of_cpu_judgement, gpu_failure_point_code,
    pool_checker_bits_agree_in_the_cross_check, pool_checker_bits_compared_by_the_cross_check,
    pool_checker_violated_and_judged_bits, verdict_bits, CrossCheckWords, LandedRootWords,
    GPU_FAILURE_POINTS, GPU_FAILURE_POINT_ALLOCATION_RECORD_OUTSIDE_THE_POOL_GEOMETRY,
    GPU_FAILURE_POINT_DEVICE_ENDS_BEFORE_THE_UNIT_AREA_START,
    GPU_FAILURE_POINT_ENTRY_WIDTH_DIFFERS_FROM_ITS_FIELD_TABLE,
    GPU_FAILURE_POINT_INODE_NUMBER_WATERMARK_ROW_MISSING_FROM_THE_ACCOUNTING_TREE,
    GPU_FAILURE_POINT_INVARIANT_VIOLATED, GPU_FAILURE_POINT_MAPPING_MISS,
    GPU_FAILURE_POINT_MAPPING_STILL_UNREADABLE, GPU_FAILURE_POINT_NONE,
    GPU_FAILURE_POINT_NO_DEVICE_IN_THE_POOL, GPU_FAILURE_POINT_NO_VALID_ROOT,
    GPU_FAILURE_POINT_NO_VALID_SYSTEM_CONFIGURATION,
    GPU_FAILURE_POINT_ROOT_PUBLISH_CARRIES_MORE_THAN_ONE_LAST_RECORD_FLAG_WHOSE_ANCHOR_IS_UNDECIDED,
    GPU_FAILURE_POINT_ROOT_RING_SLOTS_PER_REGION_OUT_OF_RANGE,
    GPU_FAILURE_POINT_SYSTEM_CONFIGURATIONS_DISAGREE,
    GPU_FAILURE_POINT_SYSTEM_CONFIGURATION_INCOMPAT_BITS_NOT_RECOGNIZED,
    GPU_FAILURE_POINT_SYSTEM_CONFIGURATION_VALUE_REFUSED, GPU_FAILURE_POINT_UNIT_MALFORMED,
    GPU_FAILURE_POINT_UNIT_UNREADABLE, POOL_CHECKER_BITS_LEFT_OUT_OF_THE_CROSS_CHECK,
    POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS, RECOVERY_OUTCOME_CODE_FAILED,
    RECOVERY_OUTCOME_CODE_FILE_READ, RECOVERY_OUTCOME_CODE_NO_FILE,
};
/// 扫描重建那一截的结局码表（I-1.13 的结局并进对拍字之后 CPU 一侧也要它，挪进 `crate::crash_judge_conclusions`、不挂特性），这里照旧转出去：
/// 内核常量照它生成，对拍用例照这条路径引。
pub use crate::crash_judge_conclusions::{
    scan_rebuild_outcome_name, scan_rebuild_outcome_of_the_pool_checker_verdicts,
    ScanRebuildOutcome, ScanRebuildVerdictOfThePoolChecker, SCAN_REBUILD_OUTCOMES,
    SCAN_REBUILD_OUTCOME_NOT_REACHED,
};

/// 两个内核共用的那一半源码：表的读法、u64、落点与版本、指针与单元的解析、系统配置与根与 journal 的解析、掩码。
pub const GPU_JUDGE_COMMON_SOURCE: &str = include_str!("crash_judge_common.wgsl");
/// 恢复内核：两遍恢复、oracle、记录核对器（写每个状态结论的第 0..8 字）。
pub const GPU_JUDGE_RECOVERY_SOURCE: &str = include_str!("crash_judge_recovery.wgsl");
/// 池级 checker 两个内核共用的那一半：不变量下标、根与记录的读法、草稿区各集合的位置、交接段。
pub const GPU_JUDGE_CHECKER_COMMON_SOURCE: &str = include_str!("crash_judge_checker_common.wgsl");
/// 池级 checker 走读内核：第一、二批不变量的违例位（写违例位那几个字、走到第几步与判不了的原因），走完把计数与几何交进草稿区的交接段。
pub const GPU_JUDGE_CHECKER_SOURCE: &str = include_str!("crash_judge_checker.wgsl");
/// 池级 checker 扫描内核：第三批（扫描方向与记账那九条），违例位并进违例位那几个字。同一份源码里另有第二个入口
/// [`CHECKER_REFERENCES_AND_SNAPSHOTS_ENTRY_POINT`]（引用与快照那一截：I-3.9、I-9.14 与只读快照那几条），另编一条管线、接在扫描那一趟之后派。
pub const GPU_JUDGE_CHECKER_SCAN_SOURCE: &str = include_str!("crash_judge_checker_scan.wgsl");
/// 池级 checker 的扫描重建：I-1.13（扫描重建与遍历方向的索引相同），接在扫描内核源码之后、同一份源码里的第三个入口
/// [`CHECKER_SCAN_REBUILD_ENTRY_POINT`]（欠账 C821；用户 2026-10-18 定「补 GPU 扫描重建内核」，两路结论逐位相同、导入比较不改）。
pub const GPU_JUDGE_CHECKER_SCAN_REBUILD_SOURCE: &str =
    include_str!("crash_judge_checker_scan_rebuild.wgsl");
/// 恢复、走读、扫描三条管线的入口名。
pub const KERNEL_ENTRY_POINT: &str = "main";
/// 扫描重建那一截的入口名（扫描内核源码里的第三个入口）：I-1.13 是池级 checker 最后判的一条，比对之前要看别的不变量有没有违例
/// （`singlefs_checker::walk::INVARIANTS_WHOSE_VIOLATION_MEANS_THE_WALKED_INDEX_IS_DAMAGED`），所以另编一条管线、接在引用与快照那一截之后派；
/// 扫描主入口离展开预算只剩四万多个表达式，不并进它（展开预算各量各的）。
pub const CHECKER_SCAN_REBUILD_ENTRY_POINT: &str = "scan_rebuild_main";
/// 引用与快照那一截的入口名（扫描内核源码里的第二个入口）：I-3.9 与 I-9.14（每条候选根引用了什么，判决 `research/prompts/m4-gc-r1-main-verification.md`
/// 第三节 G3 起按每条根自己的中央映射认位置、数活快照的两棵树与映射叶里留存的条目），与只读快照那几条（I-3.5 ③ ④ 与逆向并进下标 65、
/// I-3.6、I-3.7，判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S5）。扫描那一截展开之后离预算只剩几千个表达式，装不下这几样，
/// 挪进同一份源码的第二个入口：两个入口共用全部函数与草稿区，各编一条管线、各量各的展开预算。
pub const CHECKER_REFERENCES_AND_SNAPSHOTS_ENTRY_POINT: &str = "references_and_snapshots_main";

/// 池级 checker 违例位的第一个字（位 i 在第 `POOL_CHECKER_VIOLATED_FIRST_WORD + i / 32` 字的第 `i % 32` 位，
/// 位 i ↔ `singlefs_checker::image::IMPLEMENTED_INVARIANTS[i]`）。
pub const POOL_CHECKER_VIOLATED_FIRST_WORD: usize = 8;
/// 池级 checker 违例位占几个字：4 个字 128 位。m3-inv-a-r1 与 m3-inv-b-r1 两轮新立之后是 65 条，64 位的字装不下
/// （判决 `research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 11 行）。
pub const POOL_CHECKER_VIOLATED_WORDS: usize = 4;
/// 违例位最多能装几条不变量。
pub const POOL_CHECKER_VIOLATED_BIT_CAPACITY: usize = POOL_CHECKER_VIOLATED_WORDS * 32;
/// 池级 checker 内核在这个状态上走到了第几步（0 盘数超上限或一块盘都择不到系统配置、1 整池不作保（有槽带拒收的值，或可择槽的 I-8.1 三个字段有一格不等）、
/// 2 择到了系统配置、3 根环非空并判完第一批、4 走读那一批判完、5 扫描那一批判完）。
pub const POOL_CHECKER_REACHED_WORD: usize =
    POOL_CHECKER_VIOLATED_FIRST_WORD + POOL_CHECKER_VIOLATED_WORDS;
/// 池级 checker 内核判不了的原因。
pub const POOL_CHECKER_UNDECIDABLE_WORD: usize = POOL_CHECKER_REACHED_WORD + 1;
/// 看 journal 那一遍落到的根（实例代号、checkpoint_txg 低 32 位；没落到根时两个都是 `u32::MAX`；落没落到根的标记与高 32 位在
/// [`CONSULTED_PASS_LANDED_MARKER_WORD`] 那两个字），走读结局与失败点。
pub const CONSULTED_PASS_LANDED_INSTANCE_WORD: usize = 2;
pub const CONSULTED_PASS_LANDED_CHECKPOINT_TXG_LOW_WORD: usize = 3;
pub const CONSULTED_PASS_OUTCOME_WORD: usize = 4;
pub const CONSULTED_PASS_FAILURE_POINT_WORD: usize = 5;
/// 不看 journal 那一遍的走读结局与失败点。
pub const IGNORED_PASS_OUTCOME_WORD: usize = 6;
pub const IGNORED_PASS_FAILURE_POINT_WORD: usize = 7;
/// 不看 journal 那一遍落到的根（同看 journal 那一遍的两个字；标记与高 32 位在 [`IGNORED_PASS_LANDED_MARKER_WORD`] 那两个字）：
/// 恢复内核写，接在池级 checker 那几个字之后。
pub const IGNORED_PASS_LANDED_INSTANCE_WORD: usize = POOL_CHECKER_UNDECIDABLE_WORD + 1;
pub const IGNORED_PASS_LANDED_CHECKPOINT_TXG_LOW_WORD: usize =
    IGNORED_PASS_LANDED_INSTANCE_WORD + 1;
/// 池级 checker 逐条不变量「判到」位的第一个字（位 i 在第 `POOL_CHECKER_JUDGED_FIRST_WORD + i / 32` 字的第 `i % 32` 位，与违例位同一个下标）：
/// 走读内核写、扫描内核并进它那一批。
pub const POOL_CHECKER_JUDGED_FIRST_WORD: usize = IGNORED_PASS_LANDED_CHECKPOINT_TXG_LOW_WORD + 1;
/// 判到位占几个字（与违例位一样宽）。
pub const POOL_CHECKER_JUDGED_WORDS: usize = POOL_CHECKER_VIOLATED_WORDS;
/// 池级 checker 判 I-7.4 时违例细节里点名的那一条被抛弃的根（`crate::crash_judge_conclusions::AbandonedRootNamedByThePoolChecker`；
/// 用户 2026-10-14 弹窗定「GPU 判器加细节输出」）：走读内核在被抛弃根那一半走完一条根、判它红而 I-7.4 此前还没红过时记下它
/// （CPU 池级 checker 一条不变量只记第一处违例的说明）。第一个字是点没点名（[`ABANDONED_ROOT_NAMED`] 或 [`NO_ABANDONED_ROOT_NAMED`]），
/// 之后是实例代号、checkpoint_txg 低 32 位与高 32 位（没点名时三个字都是 0）。
pub const ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_WORD: usize =
    POOL_CHECKER_JUDGED_FIRST_WORD + POOL_CHECKER_JUDGED_WORDS;
pub const ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_INSTANCE_WORD: usize =
    ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_WORD + 1;
pub const ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_CHECKPOINT_TXG_LOW_WORD: usize =
    ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_INSTANCE_WORD + 1;
pub const ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_CHECKPOINT_TXG_HIGH_WORD: usize =
    ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_CHECKPOINT_TXG_LOW_WORD + 1;
/// 点名那一字的两个值。
pub const NO_ABANDONED_ROOT_NAMED: u32 = 0;
pub const ABANDONED_ROOT_NAMED: u32 = 1;
/// 扫描重建那一截判 I-1.13 的结局（[`SCAN_REBUILD_OUTCOMES`] 里的码：成立、违例、报不适用的哪一种理由；没走到 I-1.13 的状态是
/// [`SCAN_REBUILD_OUTCOME_NOT_REACHED`]）：对拍用例拿它与 CPU 池级 checker 那一句不适用的理由逐个比（违例位与判到位之外，C821 的验收
/// 「报不适用及其理由逐位相同」）。走读内核先写：池级 checker 走到扫描重建之前整池交回的那四种（码 27 到 30）在它那几处早退写码，
/// 别的状态写没走到；交接段有效时扫描重建那一截再写它判成的那一种（池级 checker 那几处早退之后不走扫描重建，交接段那时无效）。
pub const SCAN_REBUILD_OUTCOME_WORD: usize =
    ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_CHECKPOINT_TXG_HIGH_WORD + 1;
/// 逐盘自证过的系统配置槽数（两槽全废盘面观察者挪进判段要的那一样，用户 2026-10-22 定「你做你的 GPU版让GPU去做」）：走读内核读完每盘两槽之后写，
/// 盘按判表里池里的盘的次序（`crate::crash_judge_tables::JudgeTables` 的 `devices`），第 d 块盘占 [`SELF_VERIFIED_SLOT_COUNT_BITS_OF_A_DEVICE`] × d 起
/// 那几位，值是 0、1、2：magic 与整槽校验和都对的槽数，不分 fsid，自证过而不可择的槽、带这一版读者不收之值的槽也算——同池级 checker 的逐盘数
/// （`singlefs_checker::image::system_configuration_redundancy_report`，CPU 参照 `crate::crash_judge_conclusions::self_verified_system_configuration_slots_of_each_device`）。
/// 盘数越过内核上限（判不了）时写 0。
pub const SELF_VERIFIED_SYSTEM_CONFIGURATION_SLOTS_OF_EACH_DEVICE_WORD: usize =
    SCAN_REBUILD_OUTCOME_WORD + 1;
/// 那一字里一块盘占几位（0、1、2 三个值）。
pub const SELF_VERIFIED_SLOT_COUNT_BITS_OF_A_DEVICE: u32 = 2;
/// 两遍恢复落到的根的另两个字（`crate::crash_judge_conclusions::LandedRootWords`；用户 2026-10-24 定 checkpoint_txg「存完整 u64」，审计复核 T01）：
/// 落没落到根的标记（[`LandedRootWords::LANDED_ON_A_ROOT`] 或 [`LandedRootWords::LANDED_ON_NO_ROOT`]，内核里是 `PASS_LANDED_ON_A_ROOT`、
/// `PASS_LANDED_ON_NO_ROOT`）与 checkpoint_txg 高 32 位（没落到根时是 `u32::MAX`）。恢复内核写，接在结论末尾：池级 checker 那几个内核
/// 写的字号一个都不挪。
pub const CONSULTED_PASS_LANDED_MARKER_WORD: usize =
    SELF_VERIFIED_SYSTEM_CONFIGURATION_SLOTS_OF_EACH_DEVICE_WORD + 1;
pub const CONSULTED_PASS_LANDED_CHECKPOINT_TXG_HIGH_WORD: usize =
    CONSULTED_PASS_LANDED_MARKER_WORD + 1;
pub const IGNORED_PASS_LANDED_MARKER_WORD: usize =
    CONSULTED_PASS_LANDED_CHECKPOINT_TXG_HIGH_WORD + 1;
pub const IGNORED_PASS_LANDED_CHECKPOINT_TXG_HIGH_WORD: usize = IGNORED_PASS_LANDED_MARKER_WORD + 1;
/// 每个状态的结论几个 u32：恢复内核 8 个，池级 checker 内核违例位、走到第几步、判不了的原因，不看 journal 那一遍落到的根，判到位，
/// I-7.4 点名的被抛弃根，I-1.13 的结局，逐盘自证过的系统配置槽数，两遍落到的根的标记与 checkpoint_txg 高 32 位。
pub const VERDICT_WORDS: usize = IGNORED_PASS_LANDED_CHECKPOINT_TXG_HIGH_WORD + 1;

// 违例位装得下池级 checker 的每一条不变量；读出来的是一个 u128，字数再多就装不进。
const _: () = assert!(
    singlefs_checker::image::IMPLEMENTED_INVARIANTS.len() <= POOL_CHECKER_VIOLATED_BIT_CAPACITY,
    "池级 checker 的不变量多于违例位的容量：先加宽 POOL_CHECKER_VIOLATED_WORDS"
);
const _: () = assert!(
    POOL_CHECKER_VIOLATED_BIT_CAPACITY <= 128,
    "违例位读成一个 u128"
);

/// 池级 checker 内核覆盖到的不变量（`IMPLEMENTED_INVARIANTS` 的下标）：全部 109 条——三批 49 条，加里程碑三二·1 那一批 16 条
/// （判决 `research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 11 行），下标 65 的 I-3.5（引用区间的精确性，
/// 走读内核在回退候选集的版本都走完之后逐对判），再加 C673（三 oracle 合谋测试的补 oracle 改法没做）⑤ ⑥ 那两条
/// （下标 66 是 I-9.27，走读内核挨着 I-9.16 按表判；下标 67 是 I-5.5，扫描内核每一版重读树表与 extent 树判），
/// 再加下标 68 的 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同，扫描内核挨着 I-9.14 读根表里每一条根的树表判；
/// 部分没判与不适用一样不置违例位，结论里只有违例位），再加里程碑三·3 删除线那五条：下标 69 的 I-9.5、70 的 I-3.4、71 的 I-4.7、
/// 72 的 I-4.6 由扫描内核的删除态扫描在回退候选集的每一版上判（判哪几版、经头 ID 配对、读不全不置位，照 `singlefs_checker::walk` 的
/// `judge_deletion_state_of_the_versions`），73 的 I-4.4 起初池级 checker 恒报不适用、内核不置位，里程碑四判 logged_ops 条目之后照下面下标 96 那一段判；
/// I-9.6 墓碑那一半（水位 > 所选根墓碑树里
/// 已发布墓碑的最大对象 ID）也在扫描内核里判、并进下标 41 那一位。对拍逐位比：[`POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS`]
/// 原来只有下标 92 的 I-1.13，补上扫描重建内核之后是空表，比较处经 [`cross_check_words_agree_in_the_cross_check`]、
/// [`pool_checker_bits_agree_in_the_cross_check`] 不再去掉哪一位。
/// 下标 74–82 是里程碑三·3 墓碑与谱系那九条（I-9.17、I-9.18、I-9.19、I-9.20、I-9.21、I-9.22、I-9.23、I-9.24、I-9.29，CPU 第 22 批、GPU 第 23 批）：
/// 走读内核在墓碑树、谱系树的每条内部条目上判 I-9.17、I-9.23（身份不符置位、不记走读失败；孩子是哪一类按它头里的类标签认、不按类型段定读多长，
/// 头里是另一类、按那一类的长度自证得过的按它自己那一类登记引用、不往下走，第 24 批跟 CPU 同改，inode 树内部条目的 I-9.2 同改）；
/// 扫描内核的删除态扫描读队列时判 I-9.24，
/// 墓碑与谱系那一遍在同一份版本清单上判 I-9.18、I-9.19、I-9.20、I-9.21（没装旋钮那一支）与 I-9.10 墓碑那一个等号（并进下标 43），
/// I-9.29 收各版的号 → 对象出生代进一张跨版本的表判，最新根的实例表沿链读不出时那张表不比、不置位（CPU 报部分没判，第 24 批）；
/// 一棵树的容器、一版的谱系记录、跨版本那张表、两两比不重号的墓碑总数越过上限的判不了。
/// 下标 79 的 I-9.22 只在池级 checker 带旋钮值的入口上判，崩溃点重放与放量走旧入口（`check_pool_image`，恒报不适用）、编的是三棵容器树都没装旋钮的
/// 那一份内核（[`JudgeKernelSources::of_the_judge`]），那里恒不置位；对拍用例拿 [`JudgeKernelSources::of_the_judge_given_the_container_capacity_knobs`]
/// 把旋钮值编进扫描内核时同判（第 56 批 G6）。I-9.4 第四句（同一片 inode 叶容器里记录号严格递增）走读内核一并判、并进下标 40。
/// 下标 83 是根环线第 10j 批新立的 I-3.14（实例表每片记录形态合法）：走读内核在实例表链走到的每一片上判四格（kind 只有 0 或 1、
/// 链指针记录只在最后一条、行的 flags 为 0、最后一条是合法的链指针记录），不成立的一片照 CPU 当解不开——行不收、链不往下走（第 10k 批）。
/// 下标 84 是第 49 批新立的 I-9.30（inode 号水位的台阶行与水位、记录、谱系对得上）：扫描内核先读最新根的记账树收 inode 号水位那一统计量的行、
/// 判 ①（水位不小于每一级台阶），再在墓碑与谱系那一遍走到最新根那一版时判 ②（inode 记录与谱系上的墓碑）与 ④（谱系记录带的 H_cur）；
/// 读不全的那一部分不置位（CPU 报射程里判不了），表越过上限判不了。
/// 下标 85–89 是第 50 批退役树那五条（I-9.31–I-9.35，判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 8 条）：
/// 走读内核在退役树的每条内部条目上判 I-9.31（与 I-9.17、I-9.23 同一个走法，类型段 {0, 3}）；扫描内核在墓碑与谱系那一遍的每一版上判
/// I-9.32、I-9.33、I-9.34（后两条只判死亡 txg 在这一版谱系上的，I-9.34 读头 ID 配上的那棵 inode 树内部条目里的叶身份），扫完各版再跨相邻两版判
/// I-9.35（父版照 walk.rs 取、在版本清单里恰好一版才比，最新根的实例表沿链读不出时整条不比）。判决同一条原写「其余回落 CPU」，
/// 用户 2026-09-29 定「配了 GPU 就一定要 GPU 判，不退回 CPU」，第 50 批补做把这四条搬上 GPU、不记判不了；读不全的那一部分不置位（CPU 报射程里判不了），
/// 叶身份表与两两比的退役记录越过上限判不了。
/// 下标 90、91 是第 53 批谱系树与退役树的容器容量那两条（判决 `research/prompts/m3-batch50-code-r1-main-verification.md` 第三节 F2）：扫描内核在
/// 墓碑与谱系那一遍判 I-9.36（谱系树读完容器那一处、退役树读完容器那一处，各一个调用点）；下标 91 的 I-9.37 同下标 79：编进了旋钮值的那一份内核
/// 才判、没装旋钮的那一份恒不置位。第 56 批 G2 给 I-9.21、I-9.22、I-9.36、I-9.37 补「第一片容器号是 1、相邻加一、每片至少一条」，与 CPU 同判。
/// 下标 92 的 I-1.13（扫描重建与遍历方向的索引相同）：用户 2026-10-13 定「先进覆盖表但内核不置位，记欠账」（C821），用户 2026-10-18 定
/// 「补 GPU 扫描重建内核」——扫描内核源码里的第三个入口（[`CHECKER_SCAN_REBUILD_ENTRY_POINT`]，`crash_judge_checker_scan_rebuild.wgsl`）照
/// `singlefs_checker::walk` 的 `judge_scan_rebuild_agrees_with_the_walked_index` 与 `singlefs_checker::rebuild` 判、置位，接在另三个池级 checker 入口之后派
/// （比对之前要读那几条别的不变量的违例位）；对拍逐位比，结局另写进 [`SCAN_REBUILD_OUTCOME_WORD`]（不适用的理由逐个比）。
/// 下标 93 的 I-3.6（deadlist 紧界）与 94 的 I-3.7（活快照可枚举）、并进下标 65 的 I-3.5 ③ ④ 与逆向（判决 `research/prompts/m4-snapshot-r1-main-verification.md`
/// 第三节 S5）由扫描内核源码里的第二个入口（[`CHECKER_REFERENCES_AND_SNAPSHOTS_ENTRY_POINT`]，引用与快照那一截）照 `singlefs_checker::walk` 的
/// `judge_snapshots_and_deadlists_of_the_versions` 在回退候选集每一版上判，对拍逐位比。走读内核另照 CPU 走活快照的两棵树（码 14 照 inode 树、
/// 码 15 照 extent 树，与活头共享的单元不判第二次引用、销毁中的不走）与 deadlist 树（种类 8，多层码 2 树），判树表里快照条目的配对（I-9.16 ③）、
/// 头 ID（I-9.27）与跳过销毁中记号的前驱链（I-9.25）；豁免三类之外的指针按这一版中央映射认位置（判决 `research/prompts/m4-gc-r1-main-verification.md`
/// 第三节 G3：先按提示读、两份都对不上再按映射位置读，引用集合与 I-3.5 的折叠表按映射位置与映射 key）。
/// 下标 95–98 是里程碑四自建 GC 新判的四条（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1、G3 的 N2、N3、N5、N6），连同下标 73 的
/// I-4.4 改判 logged_ops 条目（N4），都照 `singlefs_checker::walk` 那几个函数判、置位，对拍逐位比：走读内核与引用集合那一遍把 logged_ops 树（种类 12）
/// 当多层码 2 树走、节点记进引用（同 CPU 的主走读与 `TreeKindForReferenceScan::LoggedOperations`），恢复内核与 core 的冷走读一样把 logged_ops 根
/// 数进映射条目数；扫描内核主入口在记账那一判之后判下标 97 的 I-3.16（占着的槽取 I-3.1 那一份并集：全部走过的版本的引用、第一遍隔离豁免的组、
/// F 之下仍在 defer 的槽，排一次序逐盘按序走一遍，`judge_fragmentation_and_empty_segment_statistics`）；引用与快照那一截之后在版本清单每一版上
/// 判 73 的 I-4.4 与 96 的 I-5.6（`judge_compaction_intents_of_the_versions`）、95 的 I-3.15（`judge_unreleased_allocation_records_against_the_central_mapping`，
/// 隔离的单元只认记号，同 I-3.1 那两遍）、98 的 I-1.14（`judge_unit_bytes_of_each_mapping_key_across_the_versions`）。表越过上限的判不了
/// （扫描内核站点 100–104）。
/// 下标 99 的 I-4.10（截断中间态有账）与 100 的 I-9.38（数据单元声明长度对 size）是截断那一批新立的两条（判决
/// `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K7 T-2、T-4，T-2 只判一个方向见第六节 S2），连同下标 72 的 I-4.6 改成每条记录都判
/// （T-3：单元从 0 起连号、单元数对上段叶条目的标签，标签判法照第六节 S1）：扫描内核的删除态扫描照 `singlefs_checker::walk` 的
/// `judge_data_unit_counts_of_the_inodes_against_their_size`、`judge_data_units_beyond_the_size_against_the_truncation_intents`、
/// `judge_declared_lengths_of_the_data_units_within_the_size` 判（extent 树读出每个号的声明与每个单元，这一版 logged_ops 里的截断条目照读）；
/// 下标 73 的 I-4.4 在引用与快照那一截收截断条目（T-1，`judge_truncation_intent_against_its_record`）；走读与扫描两个内核的 extent 树都认上段叶条目
/// 标签 3、4（K5）。四条都置位、对拍逐位比；单元表越过上限的判不了（扫描内核站点 105）。I-1.13 照第六节 S4 两侧按 size 滤，在扫描重建那一截判。
/// 下标 101 的 I-9.39（截断字段之外的活单元不早于它，判决 `research/prompts/m4-trunc-r2-main-verification.md` 第三节 R6）是截断第二轮新立的一条：
/// 扫描内核的删除态扫描照 `singlefs_checker::walk` 的 `judge_birth_txgs_of_the_live_data_units_beyond_the_truncation_field` 每条记录判一次
/// （inode 记录那张表一行加 T、N_w 两段，读 size 之内、截断字段之外的单元头诞生代号），置位、对拍逐位比；读法 Z 第四条只在 I-1.13 里，由扫描重建那一截判。
/// 同一轮 I-9.7 拆段（走读内核判 108–119、136–139 恒 0 与截断字段，incompat 位 2 按各盘择到的槽）不加下标。
/// 下标 102 的 I-9.40（extent 上段形状由有条目的 inode 号集合定、根层级最低，判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md`
/// 第三节 M2）是多文件删除那一批新立的一条：扫描内核的删除态扫描照 `singlefs_checker::walk` 的 `judge_extent_upper_segment_shape_of_a_version`
/// 每一版的每一棵 extent 树判（深度优先走上段、按后序记子树里有没有落在自己那片叶区间里的条目，不把节点位置与号抄进表；只有改过字节的镜像才
/// 逐个再走上段找罩着那个节点的号、找那个号的那片叶），置位、对拍逐位比——两路逐位相同照用户 2026-10-19 定的双机导入冲突岔路的同一口径，
/// 不进内核不置位的表。
/// 下标 103 的 I-3.17（隔离记号与引用相符，D19（块指针的结构与宽度预算） 已定项 5「新加判据『记号与引用相符』」，用户 2026-10-22 定）：
/// 扫描内核记账那一判两遍隔离豁免之后照 `singlefs_checker::walk` 的 `judge_quarantine_marks_against_the_references` 在最新根的分配记录树上判三句
/// （树表里种类 3 的每一棵合成一张表，第三句按单元归组时分落几棵树的记录也归进一组）、置位，对拍逐位比；同一批 I-3.1、I-3.11、I-3.15 的隔离豁免改成只认记号（分配记录那一行第八个字的最高位记跨度段 0x4000）。
/// 下标 104、105 的 I-7.18（根记录的回收门槛等于现算的门槛）与 I-7.19（钉着的槽不住本实例写的根）是钉也落盘那一批新立的两条（C773（读坏与全零根槽的
/// 回收门槛钉只在内存里），用户 2026-10-20 定）：走读内核走完最新根之后照 `singlefs_checker::walk` 的
/// `judge_the_reclaim_threshold_and_the_pinned_ring_slot_of_the_newest_root` 判（建账的发布由走最新根那一遍记下的记账树根指针出生 txg 认），
/// 置位、对拍逐位比；同一批 I-3.1 并「钉带出来的那一截」由扫描内核的 `c_deferred_below_the_floor` 收（`released_slots_the_pin_kept_in_defer`），
/// 要排除的被抛弃根引用着的槽由走读内核在被抛弃根那一阶段记进草稿区末尾那一张（站点 117），扫描内核那两张表满了判不了（站点 115、116）。
/// 下标 106 的 I-1.5（系统配置记管道身份，C868）与 107 的 I-3.18（分配记录的跨度等于它罩住的单元那一类的槽数，C411）是里程碑三收尾补的两条：
/// 走读内核择系统配置时逐盘照 `singlefs_checker::walk` 择系统配置那一段判 I-1.5（择到的那一槽实现标识 16 字节不全零，槽的字节就在判表里、
/// 不另带字段）；引用与快照那一截在 I-3.15 那一份版本清单上照 `judge_allocation_record_spans_against_the_classes_of_the_units_they_cover`
/// 判 I-3.18（只要那一版的分配记录读全、不要映射读全）。同一批 I-8.10 往非末条扩（C738）由走读内核在同一次发布那一组里逐条比新根段，不加下标。
/// 都置位、对拍逐位比。
/// 下标 108 的 I-8.13（自证过的 journal 记录落在它计数器该落的环槽：D23（journal 的角色与格式） 已定项 18「记录 n 落在环内偏移
/// (计数器 − 1) mod 槽数 × 4096」的可执行形式，欠账 C897）：走读内核逐盘扫环（`crash_judge_checker.wgsl` 的 `checker_scan_journal_of_device`）
/// 时每条自证过的记录判一次，判在同键先到先留之前（同一块盘上撞键时被挤掉的那一条也判），计数器 0 没有该落的槽、判红；置位、对拍逐位比。
/// 同一批逐盘记录表的先到先留只在恢复读者读得出的记录之间比（`checker_insert_record`），不加下标。
/// 用户 2026-10-04 定「改回要求全覆盖」：没有「内核还没判」那一张表。
pub const POOL_CHECKER_COVERED_INVARIANT_INDEXES: &[usize] = &[
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
    50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73,
    74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97,
    98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108,
];

/// 内核源码里写死下标的那几条（`crash_judge_checker_common.wgsl` 的 `INVARIANT_I_4_4`、`INVARIANT_I_3_15`、`INVARIANT_I_5_6`、`INVARIANT_I_3_16`、
/// `INVARIANT_I_1_14`）：里程碑四自建 GC 那几条判法照这几个下标置位，池级 checker 的清单在别处插进一条、下标挪了，这里编译不过
/// （内核里全部 `INVARIANT_I_*` 常量另由 `tests::every_invariant_index_written_into_the_checker_kernels_names_that_invariant_in_the_pool_checker_list` 逐个核）。
pub const KERNEL_INVARIANT_INDEXES_OF_THE_COMPACTION_AND_RELOCATION_JUDGES: [(&str, usize); 5] = [
    (
        singlefs_checker::image::INTENT_DESCRIBES_ONLY_THE_TARGET_STATE,
        73,
    ),
    (
        singlefs_checker::image::UNRELEASED_ALLOCATION_RECORD_COVERS_A_UNIT_THE_CENTRAL_MAPPING_PLACES_THERE,
        95,
    ),
    (
        singlefs_checker::image::COMPACTION_REGION_HOLDS_NO_RECORD_ALLOCATED_AFTER_THE_INTENT_WAS_ISSUED,
        96,
    ),
    (
        singlefs_checker::image::FRAGMENTATION_AND_EMPTY_SEGMENT_STATISTICS_EQUAL_A_FULL_SCAN,
        97,
    ),
    (
        singlefs_checker::image::UNIT_BYTES_OF_ONE_MAPPING_KEY_AGREE_ACROSS_THE_VERSIONS,
        98,
    ),
];

/// 截断那一批内核里写死下标的那几条（`crash_judge_checker_common.wgsl` 的 `INVARIANT_I_4_6`、`INVARIANT_I_4_10`、`INVARIANT_I_9_38`、`INVARIANT_I_9_39`；判决
/// `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K7 T-2 到 T-4、`research/prompts/m4-trunc-r2-main-verification.md` 第三节 R6）：
/// 同 [`KERNEL_INVARIANT_INDEXES_OF_THE_COMPACTION_AND_RELOCATION_JUDGES`]，清单里下标挪了这里编译不过（I-4.4 收截断条目那一判照那张表里的下标 73 置位）。
pub const KERNEL_INVARIANT_INDEXES_OF_THE_TRUNCATION_JUDGES: [(&str, usize); 4] = [
    (
        singlefs_checker::image::DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
        72,
    ),
    (
        singlefs_checker::image::DATA_UNITS_BEYOND_THE_SIZE_ARE_NAMED_BY_A_TRUNCATION_INTENT,
        99,
    ),
    (
        singlefs_checker::image::DECLARED_LENGTH_OF_EACH_DATA_UNIT_WITHIN_THE_SIZE_MATCHES_THE_SIZE,
        100,
    ),
    (
        singlefs_checker::image::LIVE_DATA_UNITS_BEYOND_THE_TRUNCATION_FIELD_ARE_BORN_NO_EARLIER_THAN_IT,
        101,
    ),
];

/// 内核里写死下标的一张表：每条在池级 checker 清单里的下标就是写死的那个、在覆盖表里、不在内核不置位的表里（对拍逐位比）。不成立就在编译期 panic。
const fn assert_kernel_invariant_indexes_name_covered_invariants_compared_bit_by_bit(
    kernel_invariant_indexes: &[(&str, usize)],
) {
    let mut position = 0;
    while position < kernel_invariant_indexes.len() {
        let (name, index) = kernel_invariant_indexes[position];
        assert!(
            index < IMPLEMENTED_INVARIANTS.len()
                && invariant_names_are_equal(IMPLEMENTED_INVARIANTS[index], name),
            "内核里写死的下标与池级 checker 清单里那一条的下标不同：改内核的 INVARIANT_I_* 常量与这张表"
        );
        assert!(
            index < POOL_CHECKER_COVERED_INVARIANT_INDEXES.len()
                && POOL_CHECKER_COVERED_INVARIANT_INDEXES[index] == index,
            "内核里写死下标的那几条在 GPU 判器的覆盖表里"
        );
        let mut unset_position = 0;
        while unset_position < POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS.len() {
            assert!(
                POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS[unset_position]
                    != index,
                "内核里写死下标的那几条上了内核、对拍逐位比，不进内核不置位的表"
            );
            unset_position += 1;
        }
        position += 1;
    }
}

/// 扫描重建那一截在内核里写死下标的那一条（`crash_judge_checker_common.wgsl` 的 `INVARIANT_I_1_13`）：同
/// [`KERNEL_INVARIANT_INDEXES_OF_THE_COMPACTION_AND_RELOCATION_JUDGES`]，清单里下标挪了、或它又登记回内核不置位的表，这里编译不过。
pub const KERNEL_INVARIANT_INDEX_OF_THE_SCAN_REBUILD_JUDGE: [(&str, usize); 1] = [(
    singlefs_checker::image::SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
    92,
)];

/// 上段规范形那一条在内核里写死下标（`crash_judge_checker_common.wgsl` 的 `INVARIANT_I_9_40`）：同
/// [`KERNEL_INVARIANT_INDEXES_OF_THE_COMPACTION_AND_RELOCATION_JUDGES`]，清单里下标挪了、或它登记进内核不置位的表，这里编译不过。
pub const KERNEL_INVARIANT_INDEX_OF_THE_UPPER_SHAPE_JUDGE: [(&str, usize); 1] = [(
    singlefs_checker::image::EXTENT_UPPER_SEGMENT_IS_THE_CANONICAL_SHAPE_OF_THE_INODES_WITH_AN_ENTRY,
    102,
)];

/// 隔离记号那一条在内核里写死下标（`crash_judge_checker_common.wgsl` 的 `INVARIANT_I_3_17`）：同
/// [`KERNEL_INVARIANT_INDEXES_OF_THE_COMPACTION_AND_RELOCATION_JUDGES`]，清单里下标挪了、或它登记进内核不置位的表，这里编译不过。
pub const KERNEL_INVARIANT_INDEX_OF_THE_QUARANTINE_MARK_JUDGE: [(&str, usize); 1] = [(
    singlefs_checker::image::QUARANTINE_MARK_AGREES_WITH_THE_REFERENCES,
    103,
)];

/// 钉也落盘那一批在内核里写死下标的两条（`crash_judge_checker_common.wgsl` 的 `INVARIANT_I_7_18`、`INVARIANT_I_7_19`，C773）：同
/// [`KERNEL_INVARIANT_INDEXES_OF_THE_COMPACTION_AND_RELOCATION_JUDGES`]，清单里下标挪了、或它们登记进内核不置位的表，这里编译不过。
pub const KERNEL_INVARIANT_INDEXES_OF_THE_RECLAIM_THRESHOLD_AND_PINNED_RING_SLOT_JUDGES: [(
    &str,
    usize,
); 2] = [
    (
        singlefs_checker::image::ROOT_RECORD_RECLAIM_THRESHOLD_EQUALS_THE_RECOMPUTED_THRESHOLD,
        104,
    ),
    (
        singlefs_checker::image::PINNED_RING_SLOT_HOLDS_NO_ROOT_OF_THE_SAME_INSTANCE,
        105,
    ),
];

/// 里程碑三收尾补的那两条在内核里写死下标（`crash_judge_checker_common.wgsl` 的 `INVARIANT_I_1_5`、`INVARIANT_I_3_18`，C868、C411）：同
/// [`KERNEL_INVARIANT_INDEXES_OF_THE_COMPACTION_AND_RELOCATION_JUDGES`]，清单里下标挪了、或它们登记进内核不置位的表，这里编译不过。
pub const KERNEL_INVARIANT_INDEXES_OF_THE_WRITER_IDENTITY_AND_ALLOCATION_RECORD_SPAN_JUDGES: [(
    &str,
    usize,
); 2] = [
    (
        singlefs_checker::image::SYSTEM_CONFIGURATION_RECORDS_THE_WRITER_IDENTITY,
        106,
    ),
    (
        singlefs_checker::image::ALLOCATION_RECORD_SPAN_EQUALS_THE_SLOTS_OF_THE_CLASS_OF_THE_UNIT_IT_COVERS,
        107,
    ),
];

/// journal 记录落槽那一条在内核里写死下标（`crash_judge_checker_common.wgsl` 的 `INVARIANT_I_8_13`，C897）：同
/// [`KERNEL_INVARIANT_INDEXES_OF_THE_COMPACTION_AND_RELOCATION_JUDGES`]，清单里下标挪了、或它登记进内核不置位的表，这里编译不过。
pub const KERNEL_INVARIANT_INDEX_OF_THE_JOURNAL_RECORD_RING_SLOT_JUDGE: [(&str, usize); 1] =
    [("I-8.13", 108)];

// 里程碑四自建 GC 那几条、截断那一批那几条、扫描重建那一条、上段规范形那一条、隔离记号那一条、钉也落盘那两条、里程碑三收尾那两条与 journal 记录落槽那一条。
const _: () = assert_kernel_invariant_indexes_name_covered_invariants_compared_bit_by_bit(
    &KERNEL_INVARIANT_INDEXES_OF_THE_COMPACTION_AND_RELOCATION_JUDGES,
);
const _: () = assert_kernel_invariant_indexes_name_covered_invariants_compared_bit_by_bit(
    &KERNEL_INVARIANT_INDEX_OF_THE_QUARANTINE_MARK_JUDGE,
);
const _: () = assert_kernel_invariant_indexes_name_covered_invariants_compared_bit_by_bit(
    &KERNEL_INVARIANT_INDEXES_OF_THE_RECLAIM_THRESHOLD_AND_PINNED_RING_SLOT_JUDGES,
);
const _: () = assert_kernel_invariant_indexes_name_covered_invariants_compared_bit_by_bit(
    &KERNEL_INVARIANT_INDEX_OF_THE_UPPER_SHAPE_JUDGE,
);
const _: () = assert_kernel_invariant_indexes_name_covered_invariants_compared_bit_by_bit(
    &KERNEL_INVARIANT_INDEX_OF_THE_SCAN_REBUILD_JUDGE,
);
const _: () = assert_kernel_invariant_indexes_name_covered_invariants_compared_bit_by_bit(
    &KERNEL_INVARIANT_INDEXES_OF_THE_TRUNCATION_JUDGES,
);
const _: () = assert_kernel_invariant_indexes_name_covered_invariants_compared_bit_by_bit(
    &KERNEL_INVARIANT_INDEXES_OF_THE_WRITER_IDENTITY_AND_ALLOCATION_RECORD_SPAN_JUDGES,
);
const _: () = assert_kernel_invariant_indexes_name_covered_invariants_compared_bit_by_bit(
    &KERNEL_INVARIANT_INDEX_OF_THE_JOURNAL_RECORD_RING_SLOT_JUDGE,
);

// 覆盖表就是池级 checker 的每一条，按下标升序：第 k 项是 k。新立一条不变量而内核没跟上时这里编译不过，不悄悄让 GPU 那一位恒 0。
const _: () = {
    assert!(
        POOL_CHECKER_COVERED_INVARIANT_INDEXES.len()
            == singlefs_checker::image::IMPLEMENTED_INVARIANTS.len(),
        "池级 checker 新立的不变量 GPU 内核还没判：补内核，再把下标补进覆盖表"
    );
    let mut position = 0;
    while position < POOL_CHECKER_COVERED_INVARIANT_INDEXES.len() {
        assert!(
            POOL_CHECKER_COVERED_INVARIANT_INDEXES[position] == position,
            "覆盖表按下标升序、一条不缺一条不重：第 k 项是 k"
        );
        position += 1;
    }
};

// 内核不置位、对拍不比的下标都在覆盖表里：不在覆盖表里的那一位本来就不比，不该登记成「进了覆盖表而内核不置位」。
const _: () = {
    let mut position = 0;
    while position < POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS.len() {
        let mut is_covered = false;
        let mut covered_position = 0;
        while covered_position < POOL_CHECKER_COVERED_INVARIANT_INDEXES.len() {
            if POOL_CHECKER_COVERED_INVARIANT_INDEXES[covered_position]
                == POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS[position]
            {
                is_covered = true;
            }
            covered_position += 1;
        }
        assert!(
            is_covered,
            "内核不置位的下标都在 GPU 判器的覆盖表（POOL_CHECKER_COVERED_INVARIANT_INDEXES）里"
        );
        position += 1;
    }
};

/// 一个状态结论里池级 checker 的违例位（[`POOL_CHECKER_VIOLATED_BIT_CAPACITY`] 位，位 i ↔ `IMPLEMENTED_INVARIANTS[i]`）。
#[must_use]
pub fn pool_checker_violated_bits(words: &[u32; VERDICT_WORDS]) -> u128 {
    words[POOL_CHECKER_VIOLATED_FIRST_WORD
        ..POOL_CHECKER_VIOLATED_FIRST_WORD + POOL_CHECKER_VIOLATED_WORDS]
        .iter()
        .enumerate()
        .fold(0u128, |bits, (word_index, word)| {
            bits | (u128::from(*word) << (32 * word_index))
        })
}

/// 一个状态结论里池级 checker 的判到位（位 i ↔ `IMPLEMENTED_INVARIANTS[i]`：这一条在这个状态上成立或违例，不是不适用、也不是部分没判）。
#[must_use]
pub fn pool_checker_judged_bits(words: &[u32; VERDICT_WORDS]) -> u128 {
    words
        [POOL_CHECKER_JUDGED_FIRST_WORD..POOL_CHECKER_JUDGED_FIRST_WORD + POOL_CHECKER_JUDGED_WORDS]
        .iter()
        .enumerate()
        .fold(0u128, |bits, (word_index, word)| {
            bits | (u128::from(*word) << (32 * word_index))
        })
}

/// 结论的字号在内核里的常量（WGSL 源码）：接在每个内核源码最前面，判法摘要也收它。
#[must_use]
pub fn verdict_layout_wgsl_constants() -> String {
    format!(
        "const VERDICT_WORDS: u32 = {VERDICT_WORDS}u;\n\
         const CONSULTED_PASS_LANDED_INSTANCE_WORD: u32 = {CONSULTED_PASS_LANDED_INSTANCE_WORD}u;\n\
         const CONSULTED_PASS_LANDED_CHECKPOINT_TXG_LOW_WORD: u32 = {CONSULTED_PASS_LANDED_CHECKPOINT_TXG_LOW_WORD}u;\n\
         const CONSULTED_PASS_OUTCOME_WORD: u32 = {CONSULTED_PASS_OUTCOME_WORD}u;\n\
         const CONSULTED_PASS_FAILURE_POINT_WORD: u32 = {CONSULTED_PASS_FAILURE_POINT_WORD}u;\n\
         const IGNORED_PASS_OUTCOME_WORD: u32 = {IGNORED_PASS_OUTCOME_WORD}u;\n\
         const IGNORED_PASS_FAILURE_POINT_WORD: u32 = {IGNORED_PASS_FAILURE_POINT_WORD}u;\n\
         const POOL_CHECKER_VIOLATED_FIRST_WORD: u32 = {POOL_CHECKER_VIOLATED_FIRST_WORD}u;\n\
         const POOL_CHECKER_VIOLATED_WORDS: u32 = {POOL_CHECKER_VIOLATED_WORDS}u;\n\
         const POOL_CHECKER_REACHED_WORD: u32 = {POOL_CHECKER_REACHED_WORD}u;\n\
         const POOL_CHECKER_UNDECIDABLE_WORD: u32 = {POOL_CHECKER_UNDECIDABLE_WORD}u;\n\
         const IGNORED_PASS_LANDED_INSTANCE_WORD: u32 = {IGNORED_PASS_LANDED_INSTANCE_WORD}u;\n\
         const IGNORED_PASS_LANDED_CHECKPOINT_TXG_LOW_WORD: u32 = {IGNORED_PASS_LANDED_CHECKPOINT_TXG_LOW_WORD}u;\n\
         const POOL_CHECKER_JUDGED_FIRST_WORD: u32 = {POOL_CHECKER_JUDGED_FIRST_WORD}u;\n\
         const POOL_CHECKER_JUDGED_WORDS: u32 = {POOL_CHECKER_JUDGED_WORDS}u;\n\
         const ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_WORD: u32 = {ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_WORD}u;\n\
         const ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_INSTANCE_WORD: u32 = {ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_INSTANCE_WORD}u;\n\
         const ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_CHECKPOINT_TXG_LOW_WORD: u32 = {ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_CHECKPOINT_TXG_LOW_WORD}u;\n\
         const ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_CHECKPOINT_TXG_HIGH_WORD: u32 = {ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_CHECKPOINT_TXG_HIGH_WORD}u;\n\
         const SCAN_REBUILD_OUTCOME_WORD: u32 = {SCAN_REBUILD_OUTCOME_WORD}u;\n\
         const SELF_VERIFIED_SYSTEM_CONFIGURATION_SLOTS_OF_EACH_DEVICE_WORD: u32 = {SELF_VERIFIED_SYSTEM_CONFIGURATION_SLOTS_OF_EACH_DEVICE_WORD}u;\n\
         const SELF_VERIFIED_SLOT_COUNT_BITS_OF_A_DEVICE: u32 = {SELF_VERIFIED_SLOT_COUNT_BITS_OF_A_DEVICE}u;\n\
         const CONSULTED_PASS_LANDED_MARKER_WORD: u32 = {CONSULTED_PASS_LANDED_MARKER_WORD}u;\n\
         const CONSULTED_PASS_LANDED_CHECKPOINT_TXG_HIGH_WORD: u32 = {CONSULTED_PASS_LANDED_CHECKPOINT_TXG_HIGH_WORD}u;\n\
         const IGNORED_PASS_LANDED_MARKER_WORD: u32 = {IGNORED_PASS_LANDED_MARKER_WORD}u;\n\
         const IGNORED_PASS_LANDED_CHECKPOINT_TXG_HIGH_WORD: u32 = {IGNORED_PASS_LANDED_CHECKPOINT_TXG_HIGH_WORD}u;\n\
         const PASS_LANDED_ON_A_ROOT: u32 = {}u;\n\
         const PASS_LANDED_ON_NO_ROOT: u32 = {}u;\n\
         const NO_ABANDONED_ROOT_NAMED: u32 = {NO_ABANDONED_ROOT_NAMED}u;\n\
         const ABANDONED_ROOT_NAMED: u32 = {ABANDONED_ROOT_NAMED}u;\n\
         const OUTCOME_NO_FILE: u32 = {RECOVERY_OUTCOME_CODE_NO_FILE}u;\n\
         const OUTCOME_FILE_READ: u32 = {RECOVERY_OUTCOME_CODE_FILE_READ}u;\n\
         const OUTCOME_FAILED: u32 = {RECOVERY_OUTCOME_CODE_FAILED}u;\n\
         const BIT_CLAIMED_VERSION_NOT_LANDED_ON_BY_THE_RECOVERY: u32 = {}u;\n",
        LandedRootWords::LANDED_ON_A_ROOT,
        LandedRootWords::LANDED_ON_NO_ROOT,
        verdict_bits::CLAIMED_VERSION_NOT_LANDED_ON_BY_THE_RECOVERY
    )
}

/// 打包表的表数、表号、段表宽、版本表的宽与组合掩码的布局、持久掩码的字数与恢复自称的那一版那一张表的宽与码在内核里的常量（WGSL 源码）：
/// 表号就是 [`JudgeTablesWords::every_table_name`] 里的次序，名字大写、前面加 `T_`。
#[must_use]
pub fn judge_table_layout_wgsl_constants() -> String {
    let mut constants = format!(
        "const TABLE_COUNT: u32 = {JUDGE_TABLE_COUNT}u;\nconst SEGMENT_WORDS: u32 = {SEGMENT_WORDS}u;\n\
         const VERSION_WORDS: u32 = {VERSION_WORDS}u;\n\
         const COMBINATION_WORDS_OF_A_VERSION: u32 = {COMBINATION_WORDS_OF_A_VERSION}u;\n\
         const VERSION_WORD_COMBINATION_PAST_THE_SIXTY_FOURTH_WRITE: u32 = {VERSION_WORD_COMBINATION_PAST_THE_SIXTY_FOURTH_WRITE}u;\n\
         const PERSISTED_MASK_WORDS: u32 = {PERSISTED_MASK_WORDS}u;\n\
         const CLAIMED_VERSION_WORDS: u32 = {CLAIMED_VERSION_WORDS}u;\n\
         const CLAIMED_VERSION_LANDED_ON_BY_THE_CONSULTED_RECOVERY: u32 = {CLAIMED_VERSION_LANDED_ON_BY_THE_CONSULTED_RECOVERY}u;\n\
         const CLAIMED_VERSION_NEWEST_ROOT_WRITTEN_WITHOUT_A_ROOT: u32 = {CLAIMED_VERSION_NEWEST_ROOT_WRITTEN_WITHOUT_A_ROOT}u;\n\
         const CLAIMED_VERSION_NEWEST_ROOT_WRITTEN_WITH_A_ROOT: u32 = {CLAIMED_VERSION_NEWEST_ROOT_WRITTEN_WITH_A_ROOT}u;\n"
    );
    for (index, name) in JudgeTablesWords::every_table_name().iter().enumerate() {
        constants.push_str(&format!(
            "const T_{}: u32 = {index}u;\n",
            name.to_ascii_uppercase()
        ));
    }
    constants
}

/// 接在每个判器内核源码最前面的全部常量：结论字号、打包表的表号与段表宽、段的展开方式码、失败点码、扫描重建的结局码。
#[must_use]
pub fn kernel_prelude_wgsl_constants() -> String {
    format!(
        "{}{}{}{}{}",
        verdict_layout_wgsl_constants(),
        judge_table_layout_wgsl_constants(),
        segment_expansion_code_wgsl_constants(),
        failure_point_wgsl_constants(),
        scan_rebuild_outcome_wgsl_constants()
    )
}

/// 扫描重建结局码在内核里的常量（WGSL 源码）：[`SCAN_REBUILD_OUTCOMES`] 每一行一个，再加 [`SCAN_REBUILD_OUTCOME_NOT_REACHED`]。
#[must_use]
pub fn scan_rebuild_outcome_wgsl_constants() -> String {
    let mut constants: String = SCAN_REBUILD_OUTCOMES
        .iter()
        .map(|outcome| {
            format!(
                "const {}: u32 = {}u;\n",
                outcome.constant_name, outcome.code
            )
        })
        .collect();
    constants.push_str(&format!(
        "const SCAN_REBUILD_OUTCOME_NOT_REACHED: u32 = {SCAN_REBUILD_OUTCOME_NOT_REACHED}u;\n"
    ));
    constants
}

/// GPU 一个状态的结论里 I-1.13 的结局码（[`SCAN_REBUILD_OUTCOME_WORD`]）。
#[must_use]
pub fn scan_rebuild_outcome_of_the_gpu_verdict(words: &[u32; VERDICT_WORDS]) -> u32 {
    words[SCAN_REBUILD_OUTCOME_WORD]
}

/// 失败点码在内核里的常量（WGSL 源码）：[`GPU_FAILURE_POINTS`] 每一行一个，名字去掉 `GPU_` 前缀（`FAILURE_POINT_*`）。
#[must_use]
pub fn failure_point_wgsl_constants() -> String {
    GPU_FAILURE_POINTS
        .iter()
        .map(|(name, code)| {
            format!(
                "const {}: u32 = {code}u;\n",
                name.strip_prefix("GPU_")
                    .expect("失败点常量名都以 GPU_ 起头")
            )
        })
        .collect()
}

/// GPU 一个状态的结论字换成对拍比的那几个字。
///
/// # Panics
/// `words` 不是 [`VERDICT_WORDS`] 个字。
#[must_use]
pub fn cross_check_words_of_gpu_verdict(words: &[u32]) -> CrossCheckWords {
    let words: &[u32; VERDICT_WORDS] = words
        .try_into()
        .expect("一个状态的结论是 VERDICT_WORDS 个字");
    let undecidable_bit = 1u32 << verdict_bits::UNDECIDABLE;
    CrossCheckWords {
        red_bits: words[0] & !undecidable_bit,
        pool_checker_violated_bits: pool_checker_violated_bits(words),
        consulted_pass_landed_root: LandedRootWords {
            landed_marker: words[CONSULTED_PASS_LANDED_MARKER_WORD],
            instance: words[CONSULTED_PASS_LANDED_INSTANCE_WORD],
            checkpoint_txg_low: words[CONSULTED_PASS_LANDED_CHECKPOINT_TXG_LOW_WORD],
            checkpoint_txg_high: words[CONSULTED_PASS_LANDED_CHECKPOINT_TXG_HIGH_WORD],
        },
        ignored_pass_landed_root: LandedRootWords {
            landed_marker: words[IGNORED_PASS_LANDED_MARKER_WORD],
            instance: words[IGNORED_PASS_LANDED_INSTANCE_WORD],
            checkpoint_txg_low: words[IGNORED_PASS_LANDED_CHECKPOINT_TXG_LOW_WORD],
            checkpoint_txg_high: words[IGNORED_PASS_LANDED_CHECKPOINT_TXG_HIGH_WORD],
        },
        consulted_pass_outcome: words[CONSULTED_PASS_OUTCOME_WORD],
        ignored_pass_outcome: words[IGNORED_PASS_OUTCOME_WORD],
        consulted_pass_failure_point: words[CONSULTED_PASS_FAILURE_POINT_WORD],
        ignored_pass_failure_point: words[IGNORED_PASS_FAILURE_POINT_WORD],
        pool_checker_judged_bits: pool_checker_judged_bits(words),
        scan_rebuild_outcome: scan_rebuild_outcome_of_the_gpu_verdict(words),
        gpu_could_not_judge: words[0] & undecidable_bit != 0
            || words[POOL_CHECKER_UNDECIDABLE_WORD] != 0,
    }
}

/// 判不了的原因字的写法：低 8 位是原因码（内核里的 `UNDECIDABLE_*`），往上是站点号（池级 checker 两个内核判不了的那一处写
/// `UNDECIDABLE_… | (站点 << 8u)`；恢复内核与没写站点的那几处是 0）。
pub const UNDECIDABLE_SITE_SHIFT: u32 = 8;

/// 一个状态的 GPU 结论里「判不了」那一句的原因：恢复内核判不了（第 0 字判不了那一位置了）时第 1 字的原因字，池级 checker 内核判不了时
/// [`POOL_CHECKER_UNDECIDABLE_WORD`] 那一字，各拆成原因码与站点号（[`UNDECIDABLE_SITE_SHIFT`]）。两个内核都判得了交回 `None`。
/// 给定状态那一路 GPU 判不了一个状态时照它打进判不了那一句（`crate::crash_amplification` 的给定状态引擎）：看站点号就知道是哪一张表满了。
#[must_use]
pub fn undecidable_reasons_of_the_gpu_verdict(words: &[u32; VERDICT_WORDS]) -> Option<String> {
    let reason_and_site = |kernel: &str, reason_word: u32| {
        format!(
            "{kernel}原因字 {reason_word:#x}（原因码 {}、站点 {}）",
            reason_word & ((1 << UNDECIDABLE_SITE_SHIFT) - 1),
            reason_word >> UNDECIDABLE_SITE_SHIFT
        )
    };
    let mut reasons = Vec::new();
    if words[0] & (1 << verdict_bits::UNDECIDABLE) != 0 {
        reasons.push(reason_and_site("恢复内核", words[1]));
    }
    if words[POOL_CHECKER_UNDECIDABLE_WORD] != 0 {
        reasons.push(reason_and_site(
            "池级 checker 内核",
            words[POOL_CHECKER_UNDECIDABLE_WORD],
        ));
    }
    (!reasons.is_empty()).then(|| reasons.join("；"))
}

/// GPU 一个状态的结论字里池级 checker 判 I-7.4 时违例细节里点名的被抛弃根（与 CPU 判器从细节里读出的
/// `crate::crash_judge_conclusions::abandoned_roots_named_in_a_pool_checker_detail` 同口径：第一处违例是被抛弃根那一半判的才点名，至多一条）。
///
/// # Panics
/// 点名那一字既不是 [`ABANDONED_ROOT_NAMED`] 也不是 [`NO_ABANDONED_ROOT_NAMED`]：走读内核每个状态都写这一字，别的值说明这个状态的结论没被
/// 走读内核写过（调用方先核 [`verdict_was_written_by_both_kernels`] 或判不了那几位）。
#[must_use]
pub fn abandoned_roots_named_by_the_pool_checker_of_the_gpu_verdict(
    words: &[u32; VERDICT_WORDS],
) -> Vec<AbandonedRootNamedByThePoolChecker> {
    match words[ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_WORD] {
        NO_ABANDONED_ROOT_NAMED => Vec::new(),
        ABANDONED_ROOT_NAMED => vec![AbandonedRootNamedByThePoolChecker {
            instance: InstanceGeneration(
                words[ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_INSTANCE_WORD],
            ),
            checkpoint_txg: CheckpointTxg(
                u64::from(words[ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_CHECKPOINT_TXG_LOW_WORD])
                    | (u64::from(
                        words[ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_CHECKPOINT_TXG_HIGH_WORD],
                    ) << 32),
            ),
        }],
        unwritten => panic!(
            "点名的被抛弃根那一字是 {unwritten}：走读内核每个状态都写 0 或 1，这个状态的结论没被走读内核写过"
        ),
    }
}

/// 一个状态的 GPU 结论字换成与 CPU `CrashStateJudgement::red_items` 逐项同名、同序的红项；判不了的状态交回 `None`
/// （崩溃放量第 ① 段收卡上结论那一段遇到它整趟停下）。红项换成判定向量的编码在 `crate::crash_judge_conclusions`。
#[must_use]
pub fn gpu_verdict_items(words: &[u32; VERDICT_WORDS]) -> Option<Vec<String>> {
    let bits = words[0];
    if bits & (1 << verdict_bits::UNDECIDABLE) != 0 || words[POOL_CHECKER_UNDECIDABLE_WORD] != 0 {
        return None;
    }
    let mut items = Vec::new();
    for (index, kind) in CrashPointReplayOracleViolationKind::EVERY_KIND
        .iter()
        .enumerate()
    {
        let shift = verdict_bits::CONSULTED_ORACLE_FIRST + u32::try_from(index).expect("七类");
        if bits & (1 << shift) != 0 {
            items.push(format!(
                "{CRASH_POINT_REPLAY_RED_PASS_JOURNAL_CONSULTED_ORACLE}:{}",
                kind.name()
            ));
        }
    }
    for (index, kind) in CrashPointReplayOracleViolationKind::EVERY_KIND
        .iter()
        .enumerate()
    {
        let shift = verdict_bits::IGNORED_ORACLE_FIRST + u32::try_from(index).expect("七类");
        if bits & (1 << shift) != 0 {
            items.push(format!(
                "{CRASH_POINT_REPLAY_RED_PASS_JOURNAL_IGNORED_ORACLE}:{}",
                kind.name()
            ));
        }
    }
    let violated = pool_checker_violated_bits(words);
    for (index, invariant) in IMPLEMENTED_INVARIANTS.iter().enumerate() {
        if violated & (1u128 << index) != 0 {
            items.push(format!(
                "{CRASH_POINT_REPLAY_RED_PASS_POOL_CHECKER}:{invariant}"
            ));
        }
    }
    if bits & (1 << verdict_bits::ROOT_WITHOUT_RECORD) != 0 {
        items.push(format!(
            "{CRASH_POINT_REPLAY_RED_PASS_RECORD_CHECKER}:{RECORD_CHECKER_ROOT_WITHOUT_RECORD}"
        ));
    }
    if bits & (1 << verdict_bits::CLAIMED_STATE_MISSING_UNIT) != 0 {
        items.push(format!(
            "{CRASH_POINT_REPLAY_RED_PASS_RECORD_CHECKER}:{RECORD_CHECKER_CLAIMED_STATE_MISSING_UNIT}"
        ));
    }
    Some(items)
}

/// GPU 一个状态的结论里逐盘自证过的系统配置槽数（[`SELF_VERIFIED_SYSTEM_CONFIGURATION_SLOTS_OF_EACH_DEVICE_WORD`]），按 `devices`
/// （这个状态那张判表的 `devices`，池里的盘的次序）交回 (盘, 槽数)：与 CPU 参照
/// `crate::crash_judge_conclusions::self_verified_system_configuration_slots_of_each_device` 同一种写法，对拍逐盘比，判段拿它对基镜像比。
///
/// # Panics
/// 盘数多于那一字装得下的（内核里的盘数上限 8 块，每块两位）。
#[must_use]
pub fn self_verified_system_configuration_slots_of_each_device_of_the_gpu_verdict(
    words: &[u32; VERDICT_WORDS],
    devices: &[u32],
) -> Vec<(u32, u32)> {
    let device_mask = (1u32 << SELF_VERIFIED_SLOT_COUNT_BITS_OF_A_DEVICE) - 1;
    assert!(
        devices.len() * usize::try_from(SELF_VERIFIED_SLOT_COUNT_BITS_OF_A_DEVICE).expect("2")
            <= 32,
        "逐盘槽数那一字装 16 块盘：判表里 {} 块",
        devices.len()
    );
    devices
        .iter()
        .enumerate()
        .map(|(device_position, device)| {
            let shift = SELF_VERIFIED_SLOT_COUNT_BITS_OF_A_DEVICE
                * u32::try_from(device_position).expect("盘的位置装得进 u32");
            (
                *device,
                (words[SELF_VERIFIED_SYSTEM_CONFIGURATION_SLOTS_OF_EACH_DEVICE_WORD] >> shift)
                    & device_mask,
            )
        })
        .collect()
}

/// GPU 一个状态的恢复字（与 CPU 那一份 `CrashStateJudgement::recovery_words` 同一种八个字，对照逐字比）：结论字先换成对拍比的那几个字
/// （[`cross_check_words_of_gpu_verdict`]），再取那八个。
///
/// # Panics
/// `words` 不是 [`VERDICT_WORDS`] 个字。
#[must_use]
pub fn recovery_words_of_the_gpu_verdict(words: &[u32]) -> RecoveryWordsOfState {
    recovery_words_of_the_cross_check_words(&cross_check_words_of_gpu_verdict(words))
}

/// GPU 一个状态的结论字换成恢复摘要（`crate::crash_amplification::RecoveryRootAndOutcomesOfState`，与 CPU 一侧
/// `crate::crash_amplification::recovery_words_of` 同一个类型）：看 journal 那一遍落到的根取标记、实例代号、checkpoint_txg 低 32 位与高 32 位那四个字
/// （高 32 位与标记是接在结论末尾的那两个字），两遍的走读结局码各取各的字。
#[must_use]
pub fn gpu_recovery_words(
    words: &[u32; VERDICT_WORDS],
) -> crate::crash_amplification::RecoveryRootAndOutcomesOfState {
    let consulted_pass_landed_root = LandedRootWords {
        landed_marker: words[CONSULTED_PASS_LANDED_MARKER_WORD],
        instance: words[CONSULTED_PASS_LANDED_INSTANCE_WORD],
        checkpoint_txg_low: words[CONSULTED_PASS_LANDED_CHECKPOINT_TXG_LOW_WORD],
        checkpoint_txg_high: words[CONSULTED_PASS_LANDED_CHECKPOINT_TXG_HIGH_WORD],
    };
    let consulted_pass_outcome = words[CONSULTED_PASS_OUTCOME_WORD];
    let ignored_pass_outcome = words[IGNORED_PASS_OUTCOME_WORD];
    crate::crash_amplification::RecoveryRootAndOutcomesOfState {
        consulted_pass_landed_root,
        consulted_pass_outcome,
        ignored_pass_outcome,
    }
}

/// 每个状态的草稿区几个 u32（checker 共用那一半里 `SCRATCH_END`：共用的 `SCRATCH_WORDS_REQUIRED`、走读与扫描的集合、两个内核的交接段、
/// 交接段之后走读内核自己用的表：两张，加 I-3.5 的出现规则读的四张；再之后是扫描内核删除态扫描与墓碑、谱系那一遍用的五张：inode 记录、每个号的数据单元数、
/// 一棵墓碑树或谱系树的容器、一版的谱系记录、I-9.29 跨版本的号 → 对象出生代，与 I-9.30 读的最新根记账里 inode 号水位那一统计量的行，
/// 再加退役那几条读的两张叶身份表：被判的那一版与 I-9.35 的父版；最后是判到位的两张表：评估过、射程里有一部分判不了，各 4 个字）。
/// 第 59 批 L6（判决 `research/prompts/m3-batch58-code-r1-main-verification.md` 第三节）把恢复内核的分配记录表 `MAX_ALLOCATION_RECORDS`
/// 与扫描内核的已释放记录表 `C_MAX_RELEASED` 从 1024 抬到 2048：共用那一半多 1024 × 6 = 6144 字、已释放记录表多 1024 × 5 = 5120 字、
/// 它的「哪条候选根引用它」两字一条多 1024 × 2 = 2048 字，320 696 + 6144 + 5120 + 2048 = 334 008。
/// 之后 journal 记录表 `MAX_RECORDS` 从 512 抬到 2048（共用那一半的记录表 6 字一条多 1536 × 6 = 9216 字、走读内核 I-4.1 重放读的那一份环
/// `C_REPLAY_RECORDS` 4 字一条多 1536 × 4 = 6144 字），走读内核引用集合 `C_MAX_REFERENCES`、位置项校验和 `C_MAX_CHECKSUMS`、
/// I-3.5 落点表 `C_APPEARANCE_MAX_PLACEMENTS` 从 2048 抬到 4096（两份引用集合 5 字一条多 2 × 2048 × 5 = 20 480 字、主走读与收割两张校验和表
/// 4 字一条多 2 × 2048 × 4 = 16 384 字、落点表 10 字一条多 2048 × 10 = 20 480 字）：
/// 334 008 + 9216 + 6144 + 20 480 + 16 384 + 20 480 = 406 712。
/// 再之后扫描内核读一条根自己实例表全部行的那张表 `C_R_MAX_ROWS` 从 256 抬到 512（取号之后崩溃 369 次再挂载写 370 行那条流在站点 32 判不了）：
/// 3 字一行多 256 × 3 = 768 字，406 712 + 768 = 407 480。
/// 再之后两两比与组首判那几处换成「排一次序、按序走一遍」（一个状态一条线程，那几处随条数平方长，墓碑容器那条流与分配记录满表那两条流
/// 一次派活撞驱动的上下文切换时限），末尾接两张表：journal 组、I-5.1、I-5.4 轮着借的行号次序 `C_ROW_ORDER`（取三者最大的 `C_MAX_REFERENCES` = 4096 字）、
/// journal 组键 `C_JOURNAL_GROUP_KEYS`（4 字一条 × `MAX_RECORDS` 2048 = 8192 字，扫描内核 I-3.10 的去重桶借它）：407 480 + 4096 + 8192 = 419 768。
/// 再之后走读内核主走读的 inode → 对象出生代表 `C_MAX_INODE_BIRTHS` 从 512 抬到 1024（unlink 借、借完再吸收的两个场景七百来个 inode，
/// 在站点 13 判不了）：从原处挪到末尾（原处 2048 字空着、其后的偏移不挪），4 字一条 × 1024 = 4096 字，419 768 + 4096 = 423 864。
/// 再之后 harness 场景交出来的镜像搬进 checker 档、配了 GPU 只由 GPU 判，几张表的上限按那几张镜像现数的条数抬（原处都空着、其后的偏移不挪，
/// 抬过的表依次接在 423 864 之后，`crash_judge_checker_common.wgsl` 的 ⑫）：恢复内核一版的分配记录 `MAX_ALLOCATION_RECORDS` 与扫描内核一条根的
/// 分配记录 `C_MAX_ALLOC_RECORDS` 从 2048 抬到 4096（6 字一条 24 576 字、8 字一条 32 768 字；容器树旋钮下的随机历史最多 2534 条），
/// 跟着它的已释放记录表与「哪条候选根引用它」（5 字、2 字一条，20 480 + 8192 字）；恢复内核读一条根实例表的 `MAX_ROWS` 与扫描内核那一张
/// `C_R_MAX_ROWS` 从 512 抬到 24 576 行（各 3 字一行，73 728 字两份；66 片实例表的那个场景 23 997 行）；走读内核最新根实例表的行
/// `C_MAX_INSTANCE_ROWS` 从 1024 抬到 32 768 行（5 字一行 163 840 字，加桶表 65 536 字、同实例下一行 32 768 字）；主走读 inode → 对象出生代
/// `C_MAX_INODE_BIRTHS`、扫描内核一版的 inode 记录 `D_MAX_INODE_RECORDS` 与 I-9.29 跨版本那张表 `D_MAX_OBJECT_BIRTHS` 抬到 16 384 条
/// （4、7、6 字一条 65 536 + 114 688 + 98 304 字，各加桶表 32 768 字；点名超过一条 journal 记录的那个场景 12 582 个号）：
/// 423 864 + 24 576 + 73 728 + 32 768 + 20 480 + 8192 + 163 840 + 65 536 + 32 768 + 73 728 + 65 536 + 32 768 + 114 688 + 32 768 +
/// 98 304 + 32 768 = 1 296 312（每个状态 5 185 248 字节，原来 1 695 456 字节）。
///
/// 再之后扫描内核释放代高于 F 的已释放记录 `C_MAX_RELEASED_ABOVE` 从 2048 抬到与 `C_MAX_ALLOC_RECORDS` 同宽的 4096 条（容器树旋钮下的随机历史
/// F = 0，第 53、55 步起最新根的已释放记录多于 2048 条，在站点 34 判不了）：挪到末尾（原处 8192 字空着），4 字一条 × 4096 = 16 384 字，
/// 1 296 312 + 16 384 = 1 312 696（每个状态 5 250 784 字节）。
///
/// 再之后扫描内核 I-3.10 的去重表 `C_MAX_EXAMINED`（各版未释放的分配记录按盘、槽、代去重累计，站点 31）从 2048 抬到与 `C_MAX_ALLOC_RECORDS`
/// 同宽的 4096 条（不回收的流上累计条数跟着最新那一版的分配记录条数长：分配记录 2088 条那条流累计 2086 条，在站点 31 判不了）：挪到末尾
/// （原处 10 240 字扫描内核不再用），5 字一条 × 4096 = 20 480 字，1 312 696 + 20 480 = 1 333 176（每个状态 5 332 704 字节）；
/// 它的桶表照旧借 `C_JOURNAL_GROUP_KEYS` 那一段（桶数跟着翻倍到 8192，那一段正好 8192 字），不占草稿区。
/// 那张表满了之后不再记判不了（站点 31 没有了）：新来的记录不进表、照判，I-3.10 是置位，同一条记录判两遍结论相同；表的大小不变。
///
/// 再之后里程碑四接在后面的几张（`crash_judge_checker_common.wgsl` 的 ⑭；判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3、
/// `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S5）：主走读的「中央映射 key → 单元」11 字一条 × 4096 = 45 056 字加桶表 8192 字，
/// 一版的中央映射 16 字一条 × 1024 = 16 384 字加桶表 2048 字，引用与快照那一截的单元表 10 字一条 × 8192 = 81 920 字加桶表 16 384 字、
/// deadlist 叶条目 2 字一条 × 4096 = 8192 字、版本清单 12 字一行 × 560 = 6720 字、读树的显式栈 4 字一行 × 1024 = 4096 字：
/// 1 333 176 + 45 056 + 8192 + 16 384 + 2048 + 81 920 + 16 384 + 8192 + 6720 + 4096 = 1 522 168（每个状态 6 088 672 字节）。
/// 里程碑四自建 GC 新判的那几条（I-3.16、I-4.4、I-5.6、I-3.15、I-1.14，判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1、G3）
/// 不另加：扫描内核主入口 I-3.16 的占着的槽区间表，与引用与快照那一截之后 I-3.15 的落点表、logged_ops 叶条目、读树的栈、I-1.14 跨版本那张 key 表，
/// 都借引用与快照那一截的单元表起到草稿区末尾那一段（117 312 字；两处各自从头写、用完不再读，`crash_judge_checker_scan.wgsl` 的 `C_OCCUPIED` 与
/// `G_PLACEMENTS` 那几处的 `const_assert` 守着装得下）。
///
/// 再之后池级 checker 只认隔离记号、不读副本（D19（块指针的结构与宽度预算） 已定项 5，用户 2026-10-22 定）：走读内核不再收位置项校验和，
/// 主走读与候选集之外的根那两张表（152 160 起 32 768 字）、它们的桶表与交接段里的两个条数都删了，那一段空着、其后各表的偏移不挪；
/// 原来记在 185 056 的「补没补过根环里没走过的根」那一个字同样不记了。草稿区总字数不变。
///
/// 再之后截断第二轮的 I-9.39（判决 `research/prompts/m4-trunc-r2-main-verification.md` 第三节 R6）要每条记录的截断字段：扫描内核一版的 inode 记录
/// `D_INODE_RECORDS` 一行从 7 字加到 11 字（多 T、N_w 两段各 2 字），16 384 条多 4 × 16 384 = 65 536 字，其后各表照偏移链往后挪：
/// 1 522 168 + 65 536 = 1 587 704（每个状态 6 350 816 字节）。
///
/// 再之后走读内核 `w_merge_below`（别的版本先走过的单元下面的落点并进这一版）的显式栈与展开过的单元从各 1024 条抬到各
/// `C_MAX_BELOW + 1` = 4097 条（一个 900 个数据单元的长文件被后一版共用时展开过的落点多于 1024 个，在站点 5 判不了；为什么这个条数够见
/// `crash_judge_checker_common.wgsl` 定义 `C_MERGE_PENDING` 那一处）：挪到末尾（原处 6144 字留给原来就借它的中央映射只读读法的显式栈与
/// I-3.5 摊单元），3 字一条两张 2 × 4097 × 3 = 24 582 字，1 587 704 + 24 582 = 1 612 286（每个状态 6 449 144 字节）。
///
/// 再之后 `w_merge_below` 的两样找法加散列索引（抬到 4097 条之后从表头比到表尾去重、每展开一个单元把挂父单元那张表从头比到尾，
/// 长文件旁边那条流一次派活撞驱动的上下文切换时限；`crash_judge_checker_common.wgsl` 定义 `W_MERGE_EXPANDED_BUCKETS` 那一处）：
/// 展开过的单元的桶表 16 384 字，挂父单元那张表两套上下文各一份桶表 8192 字、下一行 4096 字、最后一行 4096 字，
/// 1 612 286 + 16 384 + 2 × 16 384 = 1 661 438（每个状态 6 645 752 字节）。
///
/// 再之后钉也落盘那一批（C773）：走读内核在被抛弃根那一阶段记下的「被抛弃根引用着、候选版本没走到的槽」（扫描内核 I-3.1 钉带出来的那一截按它排除），
/// 条数 1 字加 3 字一条 × 4096 = 12 289 字，1 661 438 + 12 289 = 1 673 727（每个状态 6 694 908 字节）。
///
/// 再之后走读内核 I-3.5 每一版引用的落点那张表（`C_APPEARANCE_MAX_VERSION_PLACEMENTS`，站点 44）从 8192 个字抬到结构上界：版本上限 64 乘
/// 每版落点上限 2048 = 131 072 个字（整理夹着回退那一组比重交判的 17 张镜像里两张各版合计 19 916、18 664 个，在站点 44 判不了）：挪到末尾
/// （原处 8192 个字空着），1 673 727 + 131 072 = 1 804 799（每个状态 7 219 196 字节）。同一批扫描内核 I-1.8 的扫描项 `C_MAX_SCANNED`
/// 从 2048 抬到 4096（那两张 2148、3030 项，在站点 36 判不了）：挪进 `C_PICKED` 之后原来空着的那一段，草稿区不涨。
///
/// 再之后恢复内核一版的分配记录 `MAX_ALLOCATION_RECORDS` 与扫描内核一条根的分配记录 `C_MAX_ALLOC_RECORDS` 从 4096 抬到 8192（C894：
/// 整理夹着回退那一组比重交判的 17 张镜像每版至多 4018 条，拉长到 160 步至多 4728 条）。偏移链上跟着长的：恢复内核那张 6 字一条多 24 576 字，
/// 扫描内核那张 8 字一条多 32 768 字，与它同宽的已释放记录与「哪条候选根引用它」（5 字、2 字一条）多 20 480 + 8192 字、释放代高于 F 的已释放记录
/// （4 字一条）多 16 384 字、I-3.10 的去重表（5 字一条）多 20 480 字，合 122 880 字；原来借的段装不下的两样另开两段接在末尾
/// （`crash_judge_checker_common.wgsl` 的 ⑮）：I-5.4 与 I-3.16 排分配记录的次序表 8192 字（原来借 `C_ROW_ORDER` 那 4096 字），
/// I-3.9、I-3.10 先后借的桶表 16 384 字（原来借 `C_JOURNAL_GROUP_KEYS` 那 8192 字）。
/// 1 804 799 + 122 880 + 8192 + 16 384 = 1 952 255（每个状态 7 809 020 字节，比上一版多 8.17%；一条草稿缓冲 275 个状态）。
///
/// 再之后 journal 记录表 `MAX_RECORDS` 从 2048 抬到 4096（一版分配记录过 8192 的那条流盘 0 上 3749 条记录写，2048 条时每个状态在扫环那一步就判不了）：
/// 跟着它长的三张挪到末尾（`crash_judge_checker_common.wgsl` 的 ⑯；原处 12 288、8192、8192 个字空着，其后的偏移不挪）：共用那一半的记录表
/// 6 字一条 × 4096 = 24 576 字、走读内核 I-4.1 重放读的那一份环 `C_REPLAY_RECORDS` 4 字一条 × 4096 = 16 384 字、journal 组键 `C_JOURNAL_GROUP_KEYS`
/// 4 字一条 × 4096 = 16 384 字。行号次序 `C_ROW_ORDER`（4096 字）装得下 4096 条记录的组，不挪。
/// 1 952 255 + 24 576 + 16 384 + 16 384 = 2 009 599（每个状态 8 038 396 字节，比上一版多 2.94%；一条草稿缓冲 267 个状态）。
pub const SCRATCH_WORDS: usize = 2_009_599;
/// 内核的工作组大小（四个入口都是 `@workgroup_size(64)`）：派 n 个状态要 ⌈n ÷ 64⌉ 个工作组。
pub const WORKGROUP_SIZE: usize = 64;
/// 内核的四个绑定（`crash_judge_common.wgsl` 开头）：派活参数、打包的表、结论、草稿区。
pub const DISPATCH_BINDING: u32 = 0;
pub const TABLES_BINDING: u32 = 1;
pub const VERDICTS_BINDING: u32 = 2;
pub const SCRATCH_BINDING: u32 = 3;
/// 一次派活的参数（内核里的 `Dispatch`）：这次派活第一个状态的序号（低、高 32 位）、几个状态、每个状态的草稿区几个字。
/// 派活里第 k 个调用判序号 `first_ordinal + k` 的状态，草稿区在缓冲的第 `k × SCRATCH_WORDS` 个字起。
///
/// # Panics
/// 一次派活超过 u32 个状态。
#[must_use]
pub fn dispatch_words(first_ordinal: u64, state_count: usize) -> [u32; 4] {
    [
        u32::try_from(first_ordinal & 0xFFFF_FFFF).expect("低 32 位"),
        u32::try_from(first_ordinal >> 32).expect("高 32 位"),
        u32::try_from(state_count).expect("一次派活的状态数装得进 u32"),
        u32::try_from(SCRATCH_WORDS).expect("装得进 u32"),
    ]
}

/// 结论缓冲的初值：每个字全 1。两个内核都写过的状态第 0、1 字不会同时全 1、走到第几步那一字也不会全 1。
pub const VERDICT_WORD_NOT_WRITTEN: u32 = u32::MAX;

// 扫描重建那一字的「没走到」就是结论缓冲的初值（走读内核在不是整池交回的状态上写的也是它）：两处各写一份数，这里核相等。
const _: () = assert!(
    SCAN_REBUILD_OUTCOME_NOT_REACHED == VERDICT_WORD_NOT_WRITTEN,
    "扫描重建那一字没走到的码是结论缓冲的初值"
);

/// 这个状态的结论是不是两个内核都写过（不是初值）。
#[must_use]
pub fn verdict_was_written_by_both_kernels(words: &[u32; VERDICT_WORDS]) -> bool {
    (words[0] != VERDICT_WORD_NOT_WRITTEN || words[1] != VERDICT_WORD_NOT_WRITTEN)
        && words[POOL_CHECKER_REACHED_WORD] != VERDICT_WORD_NOT_WRITTEN
}

/// 恢复内核的整份源码（生成的常量、共用那一半接上它自己那一半）。
#[must_use]
pub fn recovery_kernel_source() -> String {
    format!(
        "{}\n{GPU_JUDGE_COMMON_SOURCE}\n{GPU_JUDGE_RECOVERY_SOURCE}",
        kernel_prelude_wgsl_constants()
    )
}

/// 序号 → 持久掩码那一段的探针（对拍用例用，不判状态）：每个状态 [`PERSISTED_MASK_PROBE_WORDS`] 个字，前 [`PERSISTED_MASK_WORDS`] 个是
/// 共用那一半 `begin_state` 拆出的持久掩码（位 i 在第 i / 32 字的第 i % 32 位），最后一个是它判不了的原因（0 是判得了）。
pub const PERSISTED_MASK_PROBE_WORDS: usize = PERSISTED_MASK_WORDS + 1;

const PERSISTED_MASK_PROBE_ENTRY_SOURCE: &str = r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let local = id.x;
    if (local >= dispatch.state_count) {
        return;
    }
    begin_state(local);
    let out = local * PERSISTED_MASK_PROBE_WORDS;
    for (var word = 0u; word < PERSISTED_MASK_WORDS; word = word + 1u) {
        verdicts[out + word] = persisted[word];
    }
    verdicts[out + PERSISTED_MASK_WORDS] = undecidable;
}
"#;

/// 探针内核的整份源码：生成的常量、共用那一半、探针入口（三个判器内核拆掩码用的就是这同一份 `begin_state`）。
/// 只用派活参数、打包的表、结论三个绑定（[`DISPATCH_BINDING`]、[`TABLES_BINDING`]、[`VERDICTS_BINDING`]）。
#[must_use]
pub fn persisted_mask_probe_kernel_source() -> String {
    format!(
        "{}const PERSISTED_MASK_PROBE_WORDS: u32 = {PERSISTED_MASK_PROBE_WORDS}u;\n{GPU_JUDGE_COMMON_SOURCE}\n{PERSISTED_MASK_PROBE_ENTRY_SOURCE}",
        kernel_prelude_wgsl_constants()
    )
}

/// 探针交回的一个状态换成按枚举写表的持久集合（前 `enumerated_write_count` 位）与判不了的原因。
///
/// # Panics
/// `words` 不是 [`PERSISTED_MASK_PROBE_WORDS`] 个字；`enumerated_write_count` 超过掩码的宽（[`MAXIMUM_ENUMERATED_WRITES`]）。
#[must_use]
pub fn persisted_set_of_the_probe_words(
    words: &[u32],
    enumerated_write_count: usize,
) -> (Vec<bool>, u32) {
    assert_eq!(
        words.len(),
        PERSISTED_MASK_PROBE_WORDS,
        "一个状态的探针字数"
    );
    assert!(
        enumerated_write_count <= MAXIMUM_ENUMERATED_WRITES,
        "枚举写表装得进 {PERSISTED_MASK_WORDS} 个字的掩码"
    );
    (
        (0..enumerated_write_count)
            .map(|write_index| words[write_index / 32] & (1 << (write_index % 32)) != 0)
            .collect(),
        words[PERSISTED_MASK_PROBE_WORDS - 1],
    )
}

/// 池级 checker 内核的整份源码。
#[must_use]
pub fn checker_walk_kernel_source() -> String {
    format!(
        "{}\n{GPU_JUDGE_COMMON_SOURCE}\n{GPU_JUDGE_CHECKER_COMMON_SOURCE}\n{GPU_JUDGE_CHECKER_SOURCE}",
        kernel_prelude_wgsl_constants()
    )
}

/// 一棵容器树的旋钮在扫描内核里的值：每片记录数，没装旋钮写 0（扫描内核按 0 判产品那一条）。
fn knob_records_in_the_kernel(knob_records_per_container: Option<u64>) -> u32 {
    knob_records_per_container.map_or(0, |records_per_container| {
        u32::try_from(records_per_container).expect("旋钮值构造时验过在 1..=1359 里，装得进 u32")
    })
}

/// 三棵容器树只供测试的容量旋钮在扫描内核里的常量（WGSL 源码，第 56 批 G6）：每片记录数，0 是没装旋钮。旋钮不落盘，从镜像外给
/// （同 `singlefs_checker::walk::check_pool_image_given_the_container_capacity_knobs`）；接在扫描内核源码的结论字号常量之后。
/// 三个都为 0 的那一份（[`ContainerCapacityKnobs::PRODUCT`]）是判定入库的那一份。
#[must_use]
pub fn container_capacity_knob_wgsl_constants(knobs: ContainerCapacityKnobs) -> String {
    format!(
        "const D_TOMBSTONE_CONTAINER_CAPACITY_KNOB: u32 = {}u;\n\
         const D_LINEAGE_CONTAINER_CAPACITY_KNOB: u32 = {}u;\n\
         const D_RETIREMENT_CONTAINER_CAPACITY_KNOB: u32 = {}u;\n",
        knob_records_in_the_kernel(knobs.tombstone.map(|knob| knob.records_per_container())),
        knob_records_in_the_kernel(knobs.lineage.map(|knob| knob.records_per_container())),
        knob_records_in_the_kernel(knobs.retirement.map(|knob| knob.records_per_container())),
    )
}

/// 池级 checker 扫描内核的整份源码（三棵容器树都没装旋钮：崩溃点重放与放量编的那一份）。
#[must_use]
pub fn checker_scan_kernel_source() -> String {
    checker_scan_kernel_source_given_the_container_capacity_knobs(ContainerCapacityKnobs::PRODUCT)
}

/// 池级 checker 扫描内核的整份源码，三棵容器树的旋钮值编进内核（[`container_capacity_knob_wgsl_constants`]）。扫描重建那一截
/// （[`GPU_JUDGE_CHECKER_SCAN_REBUILD_SOURCE`]，入口 [`CHECKER_SCAN_REBUILD_ENTRY_POINT`]）接在最后：它用扫描内核里只读不判的那几遍读法。
#[must_use]
pub fn checker_scan_kernel_source_given_the_container_capacity_knobs(
    knobs: ContainerCapacityKnobs,
) -> String {
    format!(
        "{}\n{}\n{GPU_JUDGE_COMMON_SOURCE}\n{GPU_JUDGE_CHECKER_COMMON_SOURCE}\n{GPU_JUDGE_CHECKER_SCAN_SOURCE}\n{GPU_JUDGE_CHECKER_SCAN_REBUILD_SOURCE}",
        kernel_prelude_wgsl_constants(),
        container_capacity_knob_wgsl_constants(knobs)
    )
}

/// 判器编的三个内核的整份源码。派活那一侧照 [`JudgeKernelSources::of_the_judge`] 编；
/// 对拍用例要在内核里种一个假违例位（证违例位的高位端到端走得通）时，拿它改一处再交给派活那一侧。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JudgeKernelSources {
    pub recovery: String,
    pub checker_walk: String,
    pub checker_scan: String,
}

impl JudgeKernelSources {
    /// 判定入库的那三份（GPU 判器那一组判法单元的入口之一：拼这三份的函数都在那一组的闭包里）。
    #[must_use]
    pub fn of_the_judge() -> Self {
        Self::of_the_judge_given_the_container_capacity_knobs(ContainerCapacityKnobs::PRODUCT)
    }

    /// 同 [`Self::of_the_judge`]，三棵容器树的旋钮值编进扫描内核（第 56 批 G6）：判出的池级 checker 违例位要与
    /// `singlefs_checker::walk::check_pool_image_given_the_container_capacity_knobs` 带同一组旋钮值逐位相同。旋钮值不进判法单元的键
    /// （旋钮常量的写法在键里，值不在）：判定入库的路径里给定状态表的引擎（`crate::crash_amplification::GivenStatesEngine`）照表上那一栏旋钮值编它
    /// （用户 2026-10-14 弹窗定「给定状态表加旋钮栏」），旋钮值进那张表的块键；对拍用例拿它判旋钮压小写出的镜像。
    #[must_use]
    pub fn of_the_judge_given_the_container_capacity_knobs(knobs: ContainerCapacityKnobs) -> Self {
        Self {
            recovery: recovery_kernel_source(),
            checker_walk: checker_walk_kernel_source(),
            checker_scan: checker_scan_kernel_source_given_the_container_capacity_knobs(knobs),
        }
    }
}

/// 全部表按目录打包成一张 u32 表：前 2 × [`JUDGE_TABLE_COUNT`] 个字是目录（每张表的起点、条数），之后依次是各表。
///
/// # Panics
/// 某张表的起点或长度装不进 u32：建表时 `crate::crash_judge_tables::JudgeTables::check_the_packed_directory` 已拒，
/// 走到这里是没经建表就打包的表。
#[must_use]
pub fn pack_tables(words: &JudgeTablesWords) -> Vec<u32> {
    let tables = words.every_table();
    let directory_words = tables.len() * 2;
    let mut packed =
        Vec::with_capacity(directory_words + tables.iter().map(|table| table.len()).sum::<usize>());
    packed.resize(directory_words, 0);
    for (index, table) in tables.iter().enumerate() {
        packed[index * 2] = u32::try_from(packed.len()).expect("表长装得进 u32");
        packed[index * 2 + 1] = u32::try_from(table.len()).expect("表长装得进 u32");
        packed.extend_from_slice(table);
    }
    packed
}

#[cfg(test)]
mod tests {
    use super::{
        checker_scan_kernel_source, checker_walk_kernel_source, dispatch_words,
        pool_checker_violated_bits, recovery_kernel_source, verdict_was_written_by_both_kernels,
        GPU_JUDGE_CHECKER_COMMON_SOURCE, POOL_CHECKER_VIOLATED_FIRST_WORD,
        POOL_CHECKER_VIOLATED_WORDS, SCRATCH_WORDS, VERDICT_WORDS,
    };

    /// 两个内核的源码先经 naga 解析与校验（毫秒级），不等 GPU 编译：内核改坏一个字这里当场红，错误带行号。
    fn parse_and_validate(name: &str, source: &str) {
        let module = match wgpu::naga::front::wgsl::parse_str(source) {
            Ok(module) => module,
            Err(error) => panic!("{name} 解析失败：\n{}", error.emit_to_string(source)),
        };
        let mut validator = wgpu::naga::valid::Validator::new(
            wgpu::naga::valid::ValidationFlags::all(),
            wgpu::naga::valid::Capabilities::all(),
        );
        if let Err(error) = validator.validate(&module) {
            panic!("{name} 校验失败：\n{}", error.emit_to_string(source));
        }
    }

    #[test]
    fn recovery_kernel_source_parses_and_validates() {
        parse_and_validate("恢复内核", &recovery_kernel_source());
    }

    #[test]
    fn checker_walk_kernel_source_parses_and_validates() {
        parse_and_validate("池级 checker 走读内核", &checker_walk_kernel_source());
    }

    #[test]
    fn checker_scan_kernel_source_parses_and_validates() {
        parse_and_validate("池级 checker 扫描内核", &checker_scan_kernel_source());
    }

    #[test]
    fn persisted_mask_probe_kernel_source_parses_and_validates() {
        parse_and_validate(
            "持久掩码探针内核",
            &super::persisted_mask_probe_kernel_source(),
        );
    }

    /// 两个检查内核共用那一半里每一个 `const INVARIANT_I_<类>_<号>: u32 = <下标>u;`，下标处的那一条就是 `I-<类>.<号>`（两个内核照这几个常量置违例位与判到位；
    /// 清单只往末尾追加，下标写错、清单里插进一条而常量没跟上，这里红）；里程碑四自建 GC 那五条都在里面。
    #[test]
    fn every_invariant_index_written_into_the_checker_kernels_names_that_invariant_in_the_pool_checker_list(
    ) {
        let mut constants_checked = Vec::new();
        for line in super::GPU_JUDGE_CHECKER_COMMON_SOURCE.lines() {
            let Some(rest) = line.strip_prefix("const INVARIANT_I_") else {
                continue;
            };
            let (name_part, index_part) = rest.split_once(": u32 = ").unwrap_or_else(|| {
                panic!("常量行的形态是 `const INVARIANT_I_x_y: u32 = Nu;`：{line}")
            });
            let invariant = format!("I-{}", name_part.replacen('_', ".", 1));
            let index: usize = index_part
                .strip_suffix("u;")
                .and_then(|digits| digits.parse().ok())
                .unwrap_or_else(|| panic!("下标写成十进制加 u：{line}"));
            assert_eq!(
                super::IMPLEMENTED_INVARIANTS.get(index).copied(),
                Some(invariant.as_str()),
                "内核常量 INVARIANT_I_{name_part} 的下标 {index} 处不是 {invariant}"
            );
            constants_checked.push(invariant);
        }
        for (invariant, index) in
            super::KERNEL_INVARIANT_INDEXES_OF_THE_COMPACTION_AND_RELOCATION_JUDGES
                .into_iter()
                .chain(super::KERNEL_INVARIANT_INDEXES_OF_THE_TRUNCATION_JUDGES)
                .chain(super::KERNEL_INVARIANT_INDEX_OF_THE_SCAN_REBUILD_JUDGE)
                .chain(super::KERNEL_INVARIANT_INDEX_OF_THE_UPPER_SHAPE_JUDGE)
                .chain(super::KERNEL_INVARIANT_INDEXES_OF_THE_WRITER_IDENTITY_AND_ALLOCATION_RECORD_SPAN_JUDGES)
                .chain(super::KERNEL_INVARIANT_INDEX_OF_THE_JOURNAL_RECORD_RING_SLOT_JUDGE)
        {
            assert!(
                constants_checked.iter().any(|checked| checked == invariant),
                "内核里有 {invariant}（下标 {index}）那个常量"
            );
        }
    }

    /// 结论的每个字号互不相同、都在 VERDICT_WORDS 之内；不看 journal 那一遍的根与判到位接在池级 checker 那几个字之后，不挪老字号
    /// （放量流水线的 gpu_recovery_words 按具名字号取；两遍落到的根的标记与 txg 高 32 位接在末尾）。
    #[test]
    fn every_verdict_word_index_is_distinct_and_the_old_ones_stay_put() {
        let mut indexes = vec![
            0,
            1,
            super::CONSULTED_PASS_LANDED_INSTANCE_WORD,
            super::CONSULTED_PASS_LANDED_CHECKPOINT_TXG_LOW_WORD,
            super::CONSULTED_PASS_OUTCOME_WORD,
            super::CONSULTED_PASS_FAILURE_POINT_WORD,
            super::IGNORED_PASS_OUTCOME_WORD,
            super::IGNORED_PASS_FAILURE_POINT_WORD,
            super::POOL_CHECKER_REACHED_WORD,
            super::POOL_CHECKER_UNDECIDABLE_WORD,
            super::IGNORED_PASS_LANDED_INSTANCE_WORD,
            super::IGNORED_PASS_LANDED_CHECKPOINT_TXG_LOW_WORD,
            super::ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_WORD,
            super::ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_INSTANCE_WORD,
            super::ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_CHECKPOINT_TXG_LOW_WORD,
            super::ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_CHECKPOINT_TXG_HIGH_WORD,
            super::SCAN_REBUILD_OUTCOME_WORD,
            super::SELF_VERIFIED_SYSTEM_CONFIGURATION_SLOTS_OF_EACH_DEVICE_WORD,
            super::CONSULTED_PASS_LANDED_MARKER_WORD,
            super::CONSULTED_PASS_LANDED_CHECKPOINT_TXG_HIGH_WORD,
            super::IGNORED_PASS_LANDED_MARKER_WORD,
            super::IGNORED_PASS_LANDED_CHECKPOINT_TXG_HIGH_WORD,
        ];
        indexes.extend(
            POOL_CHECKER_VIOLATED_FIRST_WORD
                ..POOL_CHECKER_VIOLATED_FIRST_WORD + POOL_CHECKER_VIOLATED_WORDS,
        );
        indexes.extend(
            super::POOL_CHECKER_JUDGED_FIRST_WORD
                ..super::POOL_CHECKER_JUDGED_FIRST_WORD + super::POOL_CHECKER_JUDGED_WORDS,
        );
        let count = indexes.len();
        indexes.sort_unstable();
        indexes.dedup();
        assert_eq!(indexes.len(), count, "字号不重");
        assert_eq!(
            indexes,
            (0..VERDICT_WORDS).collect::<Vec<_>>(),
            "每个字都有用途、不越界"
        );
        assert_eq!(
            [
                super::CONSULTED_PASS_LANDED_INSTANCE_WORD,
                super::CONSULTED_PASS_LANDED_CHECKPOINT_TXG_LOW_WORD,
                super::CONSULTED_PASS_OUTCOME_WORD,
                super::IGNORED_PASS_OUTCOME_WORD
            ],
            [2, 3, 4, 6],
            "老字号不挪：看 journal 那一遍的实例代号、txg 低 32 位与两遍的结局仍在第 2、3、4、6 字"
        );
    }

    /// 打包表的表号常量就是 every_table_name 里的次序（名字大写、加 T_），表数与段表宽也生成进去。
    #[test]
    fn the_table_numbers_in_the_kernel_follow_every_table_name() {
        let constants = super::judge_table_layout_wgsl_constants();
        let names = crate::crash_judge_tables::JudgeTablesWords::every_table_name();
        for (index, name) in names.iter().enumerate() {
            let line = format!("const T_{}: u32 = {index}u;", name.to_ascii_uppercase());
            assert_eq!(constants.matches(&line).count(), 1, "恰好一行 {line}");
        }
        assert!(constants.contains(&format!("const TABLE_COUNT: u32 = {}u;", names.len())));
        assert!(constants.contains(&format!(
            "const SEGMENT_WORDS: u32 = {}u;",
            crate::crash_judge_tables::SEGMENT_WORDS
        )));
    }

    /// 扫描重建的结局码一种一个码、互不相同、不撞没走到的那个初值；内核里每个码都有一行常量；CPU 池级 checker 判 I-1.13 的每一种
    /// （成立、违例、表里每一句不适用的理由原文）换回表里那一行的码，判定里没有 I-1.13 换成没走到，表外的理由与部分没判报错。
    #[test]
    fn every_scan_rebuild_outcome_has_its_own_code_generated_into_the_kernels_and_read_back_from_the_pool_checker(
    ) {
        use singlefs_checker::image::{
            InvariantVerdict, SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
        };
        let mut codes: Vec<u32> = super::SCAN_REBUILD_OUTCOMES
            .iter()
            .map(|outcome| outcome.code)
            .collect();
        codes.push(super::SCAN_REBUILD_OUTCOME_NOT_REACHED);
        let count = codes.len();
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), count, "结局码不重，也不撞没走到的初值");
        let constants = super::scan_rebuild_outcome_wgsl_constants();
        for outcome in super::SCAN_REBUILD_OUTCOMES {
            let line = format!("const {}: u32 = {}u;", outcome.constant_name, outcome.code);
            assert_eq!(constants.matches(&line).count(), 1, "恰好一行 {line}");
            let verdict = match outcome.verdict_of_the_pool_checker {
                super::ScanRebuildVerdictOfThePoolChecker::Holds => InvariantVerdict::Holds,
                super::ScanRebuildVerdictOfThePoolChecker::Violated => {
                    InvariantVerdict::Violated("扫描重建的三张表与遍历方向的不同".to_string())
                }
                super::ScanRebuildVerdictOfThePoolChecker::NotApplicable(reason) => {
                    InvariantVerdict::NotApplicable(reason)
                }
            };
            assert_eq!(
                super::scan_rebuild_outcome_of_the_pool_checker_verdicts(&[(
                    SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
                    verdict
                )]),
                Ok(outcome.code),
                "{} 换回自己的码",
                outcome.constant_name
            );
        }
        assert_eq!(
            super::scan_rebuild_outcome_of_the_pool_checker_verdicts(&[]),
            Ok(super::SCAN_REBUILD_OUTCOME_NOT_REACHED),
            "判定里没有 I-1.13：池级 checker 没走到它"
        );
        assert!(
            super::scan_rebuild_outcome_of_the_pool_checker_verdicts(&[(
                SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
                InvariantVerdict::NotApplicable("表外的一句理由")
            )])
            .is_err(),
            "表外的理由认不得"
        );
        assert!(
            super::scan_rebuild_outcome_of_the_pool_checker_verdicts(&[(
                SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
                InvariantVerdict::PartiallyJudged("部分没判".to_string())
            )])
            .is_err(),
            "扫描重建没有部分没判这一种"
        );
        assert!(
            super::kernel_prelude_wgsl_constants().contains(&constants),
            "结局码接在每个内核源码最前面的常量里"
        );
        // 每一种结局都有内核写它：池级 checker 走到扫描重建之前整池交回的四种由走读内核在那几处早退写，别的由扫描重建那一截写
        const WRITTEN_BY_THE_WALK_KERNEL: [&str; 4] = [
            "SCAN_REBUILD_OUTCOME_POOL_REFUSED_FOR_A_SYSTEM_CONFIGURATION_VALUE_THE_READER_DOES_NOT_ACCEPT",
            "SCAN_REBUILD_OUTCOME_POOL_REFUSED_FOR_UNEQUAL_JOURNAL_RING_GEOMETRY_FIELDS",
            "SCAN_REBUILD_OUTCOME_NO_DEVICE_YIELDS_A_VALID_SYSTEM_CONFIGURATION",
            "SCAN_REBUILD_OUTCOME_ROOT_RING_HOLDS_NO_SELF_VERIFIED_ROOT",
        ];
        for outcome in super::SCAN_REBUILD_OUTCOMES {
            let is_a_whole_pool_return =
                WRITTEN_BY_THE_WALK_KERNEL.contains(&outcome.constant_name);
            let (kernel, kernel_source) = if is_a_whole_pool_return {
                ("走读内核", super::GPU_JUDGE_CHECKER_SOURCE)
            } else {
                (
                    "扫描重建那一截",
                    super::GPU_JUDGE_CHECKER_SCAN_REBUILD_SOURCE,
                )
            };
            assert!(
                kernel_source.contains(&format!("{};", outcome.constant_name))
                    || kernel_source.contains(&format!("{},", outcome.constant_name))
                    || kernel_source.contains(&format!("{})", outcome.constant_name)),
                "{} 由{kernel}写",
                outcome.constant_name
            );
        }
    }

    /// 失败点码一码一个成员、互不相同；内核里每个码都有一行常量（名字去掉 GPU_）。
    #[test]
    fn every_failure_point_code_is_distinct_and_generated_into_the_kernels() {
        let mut codes: Vec<u32> = super::GPU_FAILURE_POINTS
            .iter()
            .map(|(_, code)| *code)
            .collect();
        let count = codes.len();
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), count, "码不重");
        let constants = super::failure_point_wgsl_constants();
        for (name, code) in super::GPU_FAILURE_POINTS {
            let line = format!(
                "const {}: u32 = {code}u;",
                name.strip_prefix("GPU_").expect("GPU_ 起头")
            );
            assert_eq!(constants.matches(&line).count(), 1, "恰好一行 {line}");
        }
        assert!(super::kernel_prelude_wgsl_constants().contains(&constants));
    }

    /// 一个成员一条：`gpu_failure_point_code` 把这个成员映到它那一码（成员带的细节不进码）。
    macro_rules! failure_point_of_a_member {
        ($test:ident, $failure:expr, $code:ident) => {
            #[test]
            fn $test() {
                assert_eq!(super::gpu_failure_point_code(&$failure), super::$code);
            }
        };
    }

    use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
    use singlefs_core::recovery::{
        RecoveryFailure, SystemConfigurationValueOutsideWhatThisReaderAccepts,
    };

    failure_point_of_a_member!(
        no_valid_system_configuration_has_its_failure_point,
        RecoveryFailure::NoValidSystemConfiguration {
            first_device_with_no_valid_system_configuration_slot: DeviceIdentity(1)
        },
        GPU_FAILURE_POINT_NO_VALID_SYSTEM_CONFIGURATION
    );
    failure_point_of_a_member!(
        incompat_bits_not_recognized_has_its_failure_point,
        RecoveryFailure::SystemConfigurationIncompatBitsNotRecognized {
            first_device_carrying_them: DeviceIdentity(0),
            incompat_bitmap: singlefs_core::system_configuration::IncompatBitmap([1; 32])
        },
        GPU_FAILURE_POINT_SYSTEM_CONFIGURATION_INCOMPAT_BITS_NOT_RECOGNIZED
    );
    failure_point_of_a_member!(
        system_configurations_disagree_has_its_failure_point,
        RecoveryFailure::SystemConfigurationsDisagree,
        GPU_FAILURE_POINT_SYSTEM_CONFIGURATIONS_DISAGREE
    );
    failure_point_of_a_member!(
        no_device_in_the_pool_has_its_failure_point,
        RecoveryFailure::NoDeviceInThePool,
        GPU_FAILURE_POINT_NO_DEVICE_IN_THE_POOL
    );
    failure_point_of_a_member!(
        root_ring_slots_per_region_out_of_range_has_its_failure_point,
        RecoveryFailure::RootRingSlotsPerRegionOutOfRange {
            device: DeviceIdentity(0),
            out_of_range: singlefs_core::root_ring::RootRingSlotsPerRegionOutOfRange {
                declared_slots_per_region: 3,
                minimum: 4,
                maximum: 16
            }
        },
        GPU_FAILURE_POINT_ROOT_RING_SLOTS_PER_REGION_OUT_OF_RANGE
    );
    failure_point_of_a_member!(
        system_configuration_value_refused_has_its_failure_point,
        RecoveryFailure::SystemConfigurationValueRefused {
            device: DeviceIdentity(0),
            value: SystemConfigurationValueOutsideWhatThisReaderAccepts::EncryptionTypeNotOff {
                encryption_type: 1
            }
        },
        GPU_FAILURE_POINT_SYSTEM_CONFIGURATION_VALUE_REFUSED
    );
    failure_point_of_a_member!(
        device_ends_before_the_unit_area_start_has_its_failure_point,
        RecoveryFailure::DeviceEndsBeforeTheUnitAreaStart {
            device: DeviceIdentity(0),
            device_bytes: 4096
        },
        GPU_FAILURE_POINT_DEVICE_ENDS_BEFORE_THE_UNIT_AREA_START
    );
    failure_point_of_a_member!(
        no_valid_root_has_its_failure_point,
        RecoveryFailure::NoValidRoot,
        GPU_FAILURE_POINT_NO_VALID_ROOT
    );
    failure_point_of_a_member!(
        unit_unreadable_has_its_failure_point,
        RecoveryFailure::UnitUnreadable {
            slot: SlotNumber(7)
        },
        GPU_FAILURE_POINT_UNIT_UNREADABLE
    );
    failure_point_of_a_member!(
        unit_malformed_has_its_failure_point,
        RecoveryFailure::UnitMalformed {
            what: "树表单元"
        },
        GPU_FAILURE_POINT_UNIT_MALFORMED
    );
    failure_point_of_a_member!(
        invariant_violated_has_its_failure_point,
        RecoveryFailure::InvariantViolated {
            invariant: "I-9.16",
            detail: "树表条目不按树 ID 严格升序"
        },
        GPU_FAILURE_POINT_INVARIANT_VIOLATED
    );
    failure_point_of_a_member!(
        entry_narrower_than_its_field_table_has_its_failure_point,
        RecoveryFailure::EntryWidthDiffersFromItsFieldTable {
            what: "记账条目",
            entry_bytes: 20,
            field_table_bytes: 34
        },
        GPU_FAILURE_POINT_ENTRY_WIDTH_DIFFERS_FROM_ITS_FIELD_TABLE
    );
    failure_point_of_a_member!(
        inode_number_watermark_row_missing_has_its_failure_point,
        RecoveryFailure::InodeNumberWatermarkRowMissingFromTheAccountingTree,
        GPU_FAILURE_POINT_INODE_NUMBER_WATERMARK_ROW_MISSING_FROM_THE_ACCOUNTING_TREE
    );
    failure_point_of_a_member!(
        allocation_record_outside_the_pool_geometry_has_its_failure_point,
        RecoveryFailure::AllocationRecordOutsideThePoolGeometry {
            what: "分配记录的设备身份不在池里"
        },
        GPU_FAILURE_POINT_ALLOCATION_RECORD_OUTSIDE_THE_POOL_GEOMETRY
    );
    failure_point_of_a_member!(
        mapping_miss_has_its_failure_point,
        RecoveryFailure::MappingMiss {
            slot: SlotNumber(9)
        },
        GPU_FAILURE_POINT_MAPPING_MISS
    );
    failure_point_of_a_member!(
        mapping_still_unreadable_has_its_failure_point,
        RecoveryFailure::MappingStillUnreadable {
            slot: SlotNumber(9)
        },
        GPU_FAILURE_POINT_MAPPING_STILL_UNREADABLE
    );
    failure_point_of_a_member!(
        root_publish_with_an_undecided_anchor_has_its_failure_point,
        RecoveryFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
            counters: vec![4, 5]
        },
        GPU_FAILURE_POINT_ROOT_PUBLISH_CARRIES_MORE_THAN_ONE_LAST_RECORD_FLAG_WHOSE_ANCHOR_IS_UNDECIDED
    );

    /// GPU 结论字换成对拍字：红位去掉判不了那一位、两遍的根（标记、实例代号、checkpoint_txg 低 32 位与高 32 位，高 32 位取接在末尾的那两个字）
    /// 与结局与失败点各取各的字、违例位与判到位各读四个字、I-1.13 的结局码取第 24 字；
    /// 判不了两种来路都认。
    #[test]
    fn the_cross_check_words_of_a_gpu_verdict_take_each_field_from_its_word() {
        let mut words = [0u32; VERDICT_WORDS];
        words[0] = 1 | (1 << super::verdict_bits::UNDECIDABLE);
        words[super::CONSULTED_PASS_LANDED_INSTANCE_WORD] = 7;
        words[super::CONSULTED_PASS_LANDED_CHECKPOINT_TXG_LOW_WORD] = 9;
        words[super::CONSULTED_PASS_OUTCOME_WORD] = super::RECOVERY_OUTCOME_CODE_FAILED;
        words[super::CONSULTED_PASS_FAILURE_POINT_WORD] = super::GPU_FAILURE_POINT_MAPPING_MISS;
        words[super::IGNORED_PASS_OUTCOME_WORD] = super::RECOVERY_OUTCOME_CODE_FILE_READ;
        words[super::IGNORED_PASS_LANDED_INSTANCE_WORD] = 6;
        words[super::IGNORED_PASS_LANDED_CHECKPOINT_TXG_LOW_WORD] = 8;
        words[super::CONSULTED_PASS_LANDED_MARKER_WORD] = super::LandedRootWords::LANDED_ON_A_ROOT;
        words[super::CONSULTED_PASS_LANDED_CHECKPOINT_TXG_HIGH_WORD] = 5;
        words[super::IGNORED_PASS_LANDED_MARKER_WORD] = super::LandedRootWords::LANDED_ON_A_ROOT;
        words[super::IGNORED_PASS_LANDED_CHECKPOINT_TXG_HIGH_WORD] = 0xFFFF_FFFF;
        words[POOL_CHECKER_VIOLATED_FIRST_WORD + 3] = 1 << 31;
        words[super::POOL_CHECKER_JUDGED_FIRST_WORD + 2] = 1;
        words[super::SCAN_REBUILD_OUTCOME_WORD] = 25;
        let cross = super::cross_check_words_of_gpu_verdict(&words);
        assert_eq!(
            cross,
            super::CrossCheckWords {
                red_bits: 1,
                pool_checker_violated_bits: 1u128 << 127,
                consulted_pass_landed_root: super::LandedRootWords::of_the_landed_root(
                    InstanceGeneration(7),
                    CheckpointTxg((5 << 32) | 9)
                ),
                ignored_pass_landed_root: super::LandedRootWords::of_the_landed_root(
                    InstanceGeneration(6),
                    CheckpointTxg((0xFFFF_FFFF << 32) | 8)
                ),
                consulted_pass_outcome: super::RECOVERY_OUTCOME_CODE_FAILED,
                ignored_pass_outcome: super::RECOVERY_OUTCOME_CODE_FILE_READ,
                consulted_pass_failure_point: super::GPU_FAILURE_POINT_MAPPING_MISS,
                ignored_pass_failure_point: super::GPU_FAILURE_POINT_NONE,
                pool_checker_judged_bits: 1u128 << 64,
                scan_rebuild_outcome: 25,
                gpu_could_not_judge: true,
            }
        );
        let mut checker_gave_up = [0u32; VERDICT_WORDS];
        checker_gave_up[super::POOL_CHECKER_UNDECIDABLE_WORD] = 3;
        assert!(super::cross_check_words_of_gpu_verdict(&checker_gave_up).gpu_could_not_judge);
        assert!(
            !super::cross_check_words_of_gpu_verdict(&[0u32; VERDICT_WORDS]).gpu_could_not_judge
        );
    }

    /// 审计复核 T01（用户 2026-10-24 定 checkpoint_txg「存完整 u64」）：一份手造的结论字，看 journal 那一遍落到实例 7、checkpoint_txg
    /// `(5 << 32) | 9` 那条根（高 32 位那一字非 0）、读到文件，不看 journal 那一遍没落到根、失败；GPU 一侧换出的恢复摘要与 CPU 一侧
    /// （`crate::crash_amplification::recovery_words_of`）对同一个根、同两种结局换出的那一份逐字段相同、摊出的字逐字相同。
    /// 高 32 位那一字改成 0 之后两份不同：GPU 一侧真读了高 32 位那一字。
    #[test]
    fn the_gpu_recovery_summary_of_a_verdict_whose_checkpoint_txg_high_word_is_not_zero_equals_the_cpu_summary_of_the_same_root(
    ) {
        let landed_root = (InstanceGeneration(7), CheckpointTxg((5 << 32) | 9));
        let mut words = [0u32; VERDICT_WORDS];
        words[super::CONSULTED_PASS_LANDED_MARKER_WORD] = super::LandedRootWords::LANDED_ON_A_ROOT;
        words[super::CONSULTED_PASS_LANDED_INSTANCE_WORD] = 7;
        words[super::CONSULTED_PASS_LANDED_CHECKPOINT_TXG_LOW_WORD] = 9;
        words[super::CONSULTED_PASS_LANDED_CHECKPOINT_TXG_HIGH_WORD] = 5;
        words[super::CONSULTED_PASS_OUTCOME_WORD] = super::RECOVERY_OUTCOME_CODE_FILE_READ;
        words[super::IGNORED_PASS_LANDED_MARKER_WORD] = super::LandedRootWords::LANDED_ON_NO_ROOT;
        words[super::IGNORED_PASS_LANDED_INSTANCE_WORD] = u32::MAX;
        words[super::IGNORED_PASS_LANDED_CHECKPOINT_TXG_LOW_WORD] = u32::MAX;
        words[super::IGNORED_PASS_LANDED_CHECKPOINT_TXG_HIGH_WORD] = u32::MAX;
        words[super::IGNORED_PASS_OUTCOME_WORD] = super::RECOVERY_OUTCOME_CODE_FAILED;
        words[super::IGNORED_PASS_FAILURE_POINT_WORD] = super::GPU_FAILURE_POINT_NO_VALID_ROOT;
        let recovery_report = |outcome, effective_root| singlefs_core::recovery::RecoveryReport {
            outcome,
            effective_root,
            journal: singlefs_core::recovery::JournalScanReport::default(),
            mapping_fallbacks: 0,
            devices_without_a_mountable_system_configuration_slot: Vec::new(),
        };
        let judgement = crate::crash::CrashStateJudgement {
            consulted: recovery_report(
                singlefs_core::recovery::RecoveryOutcome::FileRead {
                    root: landed_root,
                    content: Vec::new(),
                },
                Some(landed_root),
            ),
            ignored: recovery_report(
                singlefs_core::recovery::RecoveryOutcome::Failed {
                    root: None,
                    failure: RecoveryFailure::NoValidRoot,
                },
                None,
            ),
            root_persisted: false,
            consulted_oracle: None,
            ignored_oracle: None,
            pool_checker: Vec::new(),
            records: singlefs_harness::memory_pool::RecordCheck::default(),
        };
        let summary_on_the_cpu = crate::crash_amplification::recovery_words_of(&judgement);
        let summary_on_the_gpu = super::gpu_recovery_words(&words);
        assert_eq!(
            summary_on_the_gpu, summary_on_the_cpu,
            "同一个根（实例 7、checkpoint_txg 高 32 位 5、低 32 位 9）、同两种结局：GPU 与 CPU 换出的恢复摘要逐字段相同"
        );
        assert_eq!(
            summary_on_the_gpu.words(),
            summary_on_the_cpu.words(),
            "摊出的字逐字相同"
        );
        assert_eq!(
            summary_on_the_gpu.consulted_pass_landed_root.landed_root(),
            Some(landed_root),
            "GPU 一侧的摘要换得回完整的 checkpoint_txg"
        );
        let mut words_without_the_high_word = words;
        words_without_the_high_word[super::CONSULTED_PASS_LANDED_CHECKPOINT_TXG_HIGH_WORD] = 0;
        assert_ne!(
            super::gpu_recovery_words(&words_without_the_high_word),
            summary_on_the_cpu,
            "高 32 位那一字是 0 的结论落到的是 checkpoint_txg 9 那条根，与 CPU 那一份不同"
        );
    }

    /// 判不了那一句的原因：池级 checker 内核写的原因字拆成原因码与站点号（`UNDECIDABLE_TOO_MANY_ENTRIES | (13u << 8u)` 是原因码 3、站点 13），
    /// 恢复内核判不了时取第 1 字；两个内核都判得了没有这一句（第 1 字照旧带重放停在哪一道，不当原因读）。
    #[test]
    fn the_undecidable_reasons_of_a_verdict_name_the_reason_code_and_the_site_of_each_kernel() {
        let mut words = [0u32; VERDICT_WORDS];
        words[1] = 0x0005_0002;
        assert_eq!(super::undecidable_reasons_of_the_gpu_verdict(&words), None);
        words[super::POOL_CHECKER_UNDECIDABLE_WORD] = 3 | (13 << super::UNDECIDABLE_SITE_SHIFT);
        assert_eq!(
            super::undecidable_reasons_of_the_gpu_verdict(&words).as_deref(),
            Some("池级 checker 内核原因字 0xd03（原因码 3、站点 13）")
        );
        words[0] = 1 << super::verdict_bits::UNDECIDABLE;
        words[1] = 2;
        assert_eq!(
            super::undecidable_reasons_of_the_gpu_verdict(&words).as_deref(),
            Some("恢复内核原因字 0x2（原因码 2、站点 0）；池级 checker 内核原因字 0xd03（原因码 3、站点 13）")
        );
    }

    /// 池级 checker 一份判定的违例位与判到位：违例与成立算判到，不适用与部分没判都不算。
    #[test]
    fn violated_and_judged_bits_count_holds_and_violations_as_judged_only() {
        use singlefs_checker::image::{InvariantVerdict, IMPLEMENTED_INVARIANTS};
        let verdicts = vec![
            (IMPLEMENTED_INVARIANTS[0], InvariantVerdict::Holds),
            (
                IMPLEMENTED_INVARIANTS[1],
                InvariantVerdict::Violated("红".to_string()),
            ),
            (
                IMPLEMENTED_INVARIANTS[2],
                InvariantVerdict::NotApplicable("没有对象"),
            ),
            (
                IMPLEMENTED_INVARIANTS[3],
                InvariantVerdict::PartiallyJudged("一部分".to_string()),
            ),
        ];
        assert_eq!(
            super::pool_checker_violated_and_judged_bits(&verdicts),
            (0b10, 0b11)
        );
    }

    /// 派活给每个状态的草稿区字数与内核里草稿区的末端（`SCRATCH_END`）相同：内核加了表而这里没跟上，每个状态的表就压到下一个状态的草稿区上。
    #[test]
    fn scratch_words_given_to_each_state_reach_the_scratch_end_of_the_kernels() {
        let declaration = format!("const SCRATCH_END: u32 = {SCRATCH_WORDS}u;");
        assert_eq!(
            GPU_JUDGE_CHECKER_COMMON_SOURCE
                .matches(&declaration)
                .count(),
            1,
            "池级 checker 共用那一半里恰好一行 `{declaration}`"
        );
    }

    /// 派活参数四个字各是什么：序号拆成低、高 32 位，状态数，每个状态的草稿区字数。
    #[test]
    fn dispatch_words_carry_the_first_ordinal_the_state_count_and_the_scratch_size() {
        assert_eq!(
            dispatch_words(0x0000_0002_8000_0001, 2170),
            [
                0x8000_0001,
                2,
                2170,
                u32::try_from(SCRATCH_WORDS).expect("装得进 u32")
            ]
        );
    }

    /// 违例位的每个字都读进来：第 64 位（第三个字的最低位）与第 127 位（第四个字的最高位）读得出，不被截成 64 位。
    #[test]
    fn violated_bits_past_the_sixty_fourth_are_read_from_the_third_and_fourth_words() {
        let mut words = [0u32; VERDICT_WORDS];
        words[POOL_CHECKER_VIOLATED_FIRST_WORD] = 1;
        words[POOL_CHECKER_VIOLATED_FIRST_WORD + 2] = 1;
        words[POOL_CHECKER_VIOLATED_FIRST_WORD + POOL_CHECKER_VIOLATED_WORDS - 1] = 1 << 31;
        assert_eq!(
            pool_checker_violated_bits(&words),
            1u128 | (1u128 << 64) | (1u128 << 127),
            "位 0、64、127 各在第一、三、四个违例字里"
        );
        words[super::POOL_CHECKER_REACHED_WORD] = u32::MAX;
        words[super::POOL_CHECKER_UNDECIDABLE_WORD] = u32::MAX;
        assert_eq!(
            pool_checker_violated_bits(&words),
            1u128 | (1u128 << 64) | (1u128 << 127),
            "走到第几步与判不了的原因那两个字不算进违例位"
        );
    }

    /// 结论里 I-7.4 点名的被抛弃根：点名那一字是 0 交空；是 1 交一条，checkpoint_txg 由低、高 32 位两个字拼回（高位那一字不丢）。
    #[test]
    fn the_abandoned_root_named_in_a_gpu_verdict_takes_its_instance_and_both_halves_of_its_txg() {
        use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
        let mut words = [0u32; VERDICT_WORDS];
        assert_eq!(
            super::abandoned_roots_named_by_the_pool_checker_of_the_gpu_verdict(&words),
            Vec::new(),
            "点名那一字是 0：没点名"
        );
        words[super::ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_WORD] = super::ABANDONED_ROOT_NAMED;
        words[super::ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_INSTANCE_WORD] = 3;
        words[super::ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_CHECKPOINT_TXG_LOW_WORD] = 7;
        words[super::ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_CHECKPOINT_TXG_HIGH_WORD] = 2;
        assert_eq!(
            super::abandoned_roots_named_by_the_pool_checker_of_the_gpu_verdict(&words),
            [
                crate::crash_judge_conclusions::AbandonedRootNamedByThePoolChecker {
                    instance: InstanceGeneration(3),
                    checkpoint_txg: CheckpointTxg((2u64 << 32) | 7),
                }
            ]
        );
    }

    /// 结论还是初值（全 1）的状态认得出；两个内核各写过自己那几个字的才算写过。
    #[test]
    fn a_verdict_still_at_its_initial_value_is_not_taken_as_written() {
        let untouched = [u32::MAX; VERDICT_WORDS];
        assert!(!verdict_was_written_by_both_kernels(&untouched));
        let mut only_recovery = untouched;
        only_recovery[0] = 0;
        assert!(!verdict_was_written_by_both_kernels(&only_recovery));
        let mut both = only_recovery;
        both[super::POOL_CHECKER_REACHED_WORD] = 5;
        assert!(verdict_was_written_by_both_kernels(&both));
    }

    #[test]
    fn facts_verifier_kernel_source_parses_and_validates() {
        parse_and_validate(
            "第 ② 段核对内核",
            &crate::crash_verify_gpu::verifier_shader_source(),
        );
    }

    /// 解析一份内核源码（先经 [`parse_and_validate`] 核过）。
    fn parsed_module(name: &str, source: &str) -> wgpu::naga::Module {
        parse_and_validate(name, source);
        wgpu::naga::front::wgsl::parse_str(source).expect("上面解析过")
    }

    /// 内核里名叫 `name` 的模块常量的 u32 值（初值是字面量，或顺着别的常量指到字面量）。
    fn value_of_the_u32_constant(module: &wgpu::naga::Module, name: &str) -> u32 {
        use wgpu::naga::{Expression, Literal};
        let (_, constant) = module
            .constants
            .iter()
            .find(|(_, constant)| constant.name.as_deref() == Some(name))
            .unwrap_or_else(|| panic!("内核里没有常量 {name}"));
        let mut initializer = constant.init;
        // 迭代上界：模块常量的个数（每一步换到另一个常量的初值）
        for _step in 0..=module.constants.len() {
            if let Expression::Literal(Literal::U32(value)) = module.global_expressions[initializer]
            {
                return value;
            }
            let Expression::Constant(other) = module.global_expressions[initializer] else {
                panic!(
                    "常量 {name} 的初值既不是 u32 字面量也不是别的常量：{:?}",
                    module.global_expressions[initializer]
                );
            };
            initializer = module.constants[other].init;
        }
        panic!(
            "常量 {name} 顺着指了 {} 步还没到字面量",
            module.constants.len() + 1
        );
    }

    /// 内核里名叫 `name` 的 `var<private>` 是几个元素的定长数组。
    fn element_count_of_the_private_array(module: &wgpu::naga::Module, name: &str) -> u32 {
        use wgpu::naga::{ArraySize, TypeInner};
        let (_, variable) = module
            .global_variables
            .iter()
            .find(|(_, variable)| variable.name.as_deref() == Some(name))
            .unwrap_or_else(|| panic!("内核里没有全局变量 {name}"));
        let TypeInner::Array {
            size: ArraySize::Constant(element_count),
            ..
        } = module.types[variable.ty].inner
        else {
            panic!("{name} 不是定长数组：{:?}", module.types[variable.ty].inner);
        };
        element_count.get()
    }

    /// 持久掩码的宽与组合掩码的布局在内核里只由 Rust 常量生成（`code-discipline.md`「魔法数字写成命名常量」：一个数一处定义）：
    /// 三个判器内核、探针内核与第 ② 段核对内核里 `persisted` 那个数组的元素数就是 [`crate::crash_facts::PERSISTED_MASK_WORDS`]
    /// （[`crate::crash_facts::MAXIMUM_ENUMERATED_WRITES`] ÷ 32），判器那几个内核里版本表的宽、组合掩码的字数与组合高位那几字的起点
    /// 就是 `crate::crash_judge_tables` 的同名常量；手写的 WGSL 里不另写这几个常量，也不留 64 个字的掩码。
    /// 判别力：WGSL 里把 `persisted` 写回 `array<u32, 64>`、或手写一份 `VERSION_WORDS`，这条红（后者 naga 先报重名）。
    #[test]
    fn the_mask_and_combination_widths_in_every_kernel_are_generated_from_the_rust_constants() {
        use crate::crash_facts::PERSISTED_MASK_WORDS;
        use crate::crash_judge_tables::{
            COMBINATION_WORDS_OF_A_VERSION, VERSION_WORDS,
            VERSION_WORD_COMBINATION_PAST_THE_SIXTY_FOURTH_WRITE,
        };
        let as_u32 = |value: usize| u32::try_from(value).expect("常量装得进 u32");
        let judge_kernels = [
            ("恢复内核", recovery_kernel_source()),
            ("池级 checker 走读内核", checker_walk_kernel_source()),
            ("池级 checker 扫描内核", checker_scan_kernel_source()),
            (
                "持久掩码探针内核",
                super::persisted_mask_probe_kernel_source(),
            ),
        ];
        for (name, source) in &judge_kernels {
            let module = parsed_module(name, source);
            assert_eq!(
                element_count_of_the_private_array(&module, "persisted"),
                as_u32(PERSISTED_MASK_WORDS),
                "{name} 的持久掩码"
            );
            for (constant, value) in [
                ("PERSISTED_MASK_WORDS", PERSISTED_MASK_WORDS),
                ("VERSION_WORDS", VERSION_WORDS),
                (
                    "COMBINATION_WORDS_OF_A_VERSION",
                    COMBINATION_WORDS_OF_A_VERSION,
                ),
                (
                    "VERSION_WORD_COMBINATION_PAST_THE_SIXTY_FOURTH_WRITE",
                    VERSION_WORD_COMBINATION_PAST_THE_SIXTY_FOURTH_WRITE,
                ),
            ] {
                assert_eq!(
                    value_of_the_u32_constant(&module, constant),
                    as_u32(value),
                    "{name} 的 {constant}"
                );
            }
        }
        let verifier = parsed_module(
            "第 ② 段核对内核",
            &crate::crash_verify_gpu::verifier_shader_source(),
        );
        for private_array in ["persisted", "torn"] {
            assert_eq!(
                element_count_of_the_private_array(&verifier, private_array),
                as_u32(PERSISTED_MASK_WORDS),
                "第 ② 段核对内核的 {private_array}"
            );
        }
        for (name, handwritten) in [
            ("crash_judge_common.wgsl", super::GPU_JUDGE_COMMON_SOURCE),
            (
                "第 ② 段核对内核的本体",
                crate::crash_verify_gpu::GPU_VERIFIER_SHADER_SOURCE,
            ),
        ] {
            for generated in [
                "const PERSISTED_MASK_WORDS",
                "const VERSION_WORDS",
                "const COMBINATION_WORDS_OF_A_VERSION",
                "const VERSION_WORD_COMBINATION_PAST_THE_SIXTY_FOURTH_WRITE",
                "array<u32, 64>",
            ] {
                assert!(
                    !handwritten.contains(generated),
                    "{name} 里手写了 `{generated}`：这个数由 Rust 常量生成"
                );
            }
        }
    }

    /// 走读内核 I-3.5 每一版引用的落点那张表（`C_APPEARANCE_MAX_VERSION_PLACEMENTS`）取结构上界：版本上限（`C_APPEARANCE_MAX_VERSIONS`，
    /// 满了站点 43）乘每版落点上限（`C_MAX_IN_VERSION`，满了站点 4）。那两处先判不了，这张表满了（站点 44 的容量那一半）在每个输入上都走不到。
    /// 判别力：这张表改回 8192 个字（整理夹着回退那一组比重交判的 17 张镜像里两张各版合计 19 916、18 664 个，在站点 44 判不了的那个界）这条红。
    #[test]
    fn the_appearance_version_placements_hold_the_most_versions_times_the_most_placements_of_a_version(
    ) {
        let module = parsed_module("池级 checker 走读内核", &checker_walk_kernel_source());
        let most_versions = value_of_the_u32_constant(&module, "C_APPEARANCE_MAX_VERSIONS");
        let most_placements_of_a_version = value_of_the_u32_constant(&module, "C_MAX_IN_VERSION");
        assert_eq!(
            value_of_the_u32_constant(&module, "C_APPEARANCE_MAX_VERSION_PLACEMENTS"),
            most_versions
                .checked_mul(most_placements_of_a_version)
                .expect("64 版乘每版 2048 个装得进 u32"),
            "每一版引用的落点那张表的条数是版本上限 {most_versions} 乘每版落点上限 {most_placements_of_a_version}"
        );
    }

    /// 扫描内核 I-1.8 收的扫描项（`C_MAX_SCANNED`，站点 36）与扫描重建那一遍（`SCAN_REBUILD_MAX_UNITS`，站点 110）至少要装下这么多项：
    /// 整理夹着回退那一组比重偏移 0–3、40 步交判的 17 张镜像里两张 2148、3030 项（CPU 一侧数的，在 2048 的界上判不了），
    /// 同比重 259 张（加种子到偏移 19、拉长到 160 步）至多 3030 项；4096 是 I-1.8 借的次序表（`C_ROW_ORDER`，`C_MAX_REFERENCES` 个字）的宽。
    const SCANNED_UNITS_THE_SCAN_KERNEL_HOLDS_AT_LEAST: u32 = 4096;

    /// 扫描项那张表至少 [`SCANNED_UNITS_THE_SCAN_KERNEL_HOLDS_AT_LEAST`] 项，扫描重建那一遍与它同宽（收法相同，一边装得下另一边也装得下）。
    /// 判别力：`C_MAX_SCANNED` 改回 2048 这条红。
    #[test]
    fn the_scan_and_the_scan_rebuild_hold_the_scanned_units_of_the_compaction_and_rollback_histories(
    ) {
        let module = parsed_module("池级 checker 扫描内核", &checker_scan_kernel_source());
        let scanned_units = value_of_the_u32_constant(&module, "C_MAX_SCANNED");
        assert!(
            scanned_units >= SCANNED_UNITS_THE_SCAN_KERNEL_HOLDS_AT_LEAST,
            "扫描项那张表 {scanned_units} 项，至少要 {SCANNED_UNITS_THE_SCAN_KERNEL_HOLDS_AT_LEAST} 项"
        );
        assert_eq!(
            value_of_the_u32_constant(&module, "SCAN_REBUILD_MAX_UNITS"),
            scanned_units,
            "扫描重建收扫描项那张表与 I-1.8 那一张同宽"
        );
    }

    /// 恢复内核一版的分配记录（`MAX_ALLOCATION_RECORDS`，满了记条目过多）与扫描内核一条根的分配记录（`C_MAX_ALLOC_RECORDS`，满了记站点 20）
    /// 至少要装下这么多条：整理夹着回退那一组比重 40 步交判的 17 张镜像每版至多 4018 条，离 4096 只差 78；同比重拉到 80 步那一组 45 张里 11 张、
    /// 160 步那一组 105 张里 27 张多于 4096 条，至多 4728 条（CPU 一侧数的，C894）。
    const ALLOCATION_RECORDS_THE_GPU_TABLES_HOLD_AT_LEAST: u32 = 8192;

    /// 两张分配记录表各至少 [`ALLOCATION_RECORDS_THE_GPU_TABLES_HOLD_AT_LEAST`] 条。恢复内核先解析、先判：
    /// 判别力：恢复内核那张改回 4096 这条红在恢复内核那一句（恢复内核里没有草稿区末端那条 `const_assert`，照样解析得过；
    /// 它挪了 `R_ROWS` 之后的偏移链，扫描内核随后解析也不过）；扫描内核那张改回 4096 时草稿区末端那条 `const_assert` 先不成立，红在解析扫描内核。
    #[test]
    fn both_allocation_record_tables_hold_the_records_of_a_version_of_the_longer_compaction_and_rollback_histories(
    ) {
        for (kernel, source, table_constant) in [
            (
                "恢复内核",
                recovery_kernel_source(),
                "MAX_ALLOCATION_RECORDS",
            ),
            (
                "池级 checker 扫描内核",
                checker_scan_kernel_source(),
                "C_MAX_ALLOC_RECORDS",
            ),
        ] {
            let module = parsed_module(kernel, &source);
            let records = value_of_the_u32_constant(&module, table_constant);
            assert!(
                records >= ALLOCATION_RECORDS_THE_GPU_TABLES_HOLD_AT_LEAST,
                "{kernel} 的分配记录表 {table_constant} {records} 条，至少要 {ALLOCATION_RECORDS_THE_GPU_TABLES_HOLD_AT_LEAST} 条"
            );
        }
    }

    /// 两个内核收环里 journal 记录的那张表（`crash_judge_common.wgsl` 的 `MAX_RECORDS`，恢复内核、走读内核满了记判不了、原因 2）至少要装下这么多条：
    /// 一版分配记录过 8192 条的那条流（新池新建文件之后顺序写 17 次各 200 个单元、再 201 个、再 144 个，被判那次缩回 1 个，被判那一版 8194 条）
    /// 盘 0 上 3749 条记录写（CPU 一侧量的，`crash_judge_gpu_tables_at_their_capacity.rs` 里 8194 条那条用例照这条流现数、断言不多于 4096）。
    const JOURNAL_RECORDS_THE_GPU_TABLES_HOLD_AT_LEAST: u32 = 3749;

    /// 三个判器内核的 journal 记录表各至少 [`JOURNAL_RECORDS_THE_GPU_TABLES_HOLD_AT_LEAST`] 条（三份都由共用那一半定义，恢复内核先解析、先判）。
    /// 判别力：`MAX_RECORDS` 改回 2048 这条红在恢复内核那一句（恢复内核照样解析得过；走读与扫描两个内核里草稿区末端那条 `const_assert` 不成立）。
    #[test]
    fn every_judge_kernel_holds_the_journal_records_of_the_flow_past_eight_thousand_one_hundred_and_ninety_two_allocation_records(
    ) {
        for (kernel, source) in [
            ("恢复内核", recovery_kernel_source()),
            ("池级 checker 走读内核", checker_walk_kernel_source()),
            ("池级 checker 扫描内核", checker_scan_kernel_source()),
        ] {
            let module = parsed_module(kernel, &source);
            let records = value_of_the_u32_constant(&module, "MAX_RECORDS");
            assert!(
                records >= JOURNAL_RECORDS_THE_GPU_TABLES_HOLD_AT_LEAST,
                "{kernel} 的 journal 记录表 MAX_RECORDS {records} 条，至少要 {JOURNAL_RECORDS_THE_GPU_TABLES_HOLD_AT_LEAST} 条"
            );
        }
    }

    /// 借草稿区一段的一处：说给人看的名字、它起在哪、至多占几个字（都是扫描内核里的常量名；每个字一条）。
    struct ScratchSegmentBorrower {
        name: &'static str,
        start_constant: &'static str,
        word_count_constant: &'static str,
    }

    /// 草稿区里几处先后借的一段：名字、起点与末端（段之后第一个字）的常量名、借它的那几处。
    struct BorrowedScratchSegment {
        name: &'static str,
        start_constant: &'static str,
        end_constant: &'static str,
        borrowers: &'static [ScratchSegmentBorrower],
    }

    /// 扫描内核里轮着借的四段（`crash_judge_checker_common.wgsl` 的 ⑨、⑮ 与 ⑯）与借它们的每一处。
    const BORROWED_SCRATCH_SEGMENTS_OF_THE_SCAN_KERNEL: [BorrowedScratchSegment; 4] = [
        BorrowedScratchSegment {
            name: "⑨ 行号次序",
            start_constant: "C_ROW_ORDER",
            end_constant: "C_ROW_ORDER_END",
            borrowers: &[
                ScratchSegmentBorrower {
                    name: "走读内核逐盘 journal 那几条的组",
                    start_constant: "C_ROW_ORDER",
                    word_count_constant: "MAX_RECORDS",
                },
                ScratchSegmentBorrower {
                    name: "I-3.5 一版引用的落点",
                    start_constant: "C_ROW_ORDER",
                    word_count_constant: "C_MAX_IN_VERSION",
                },
                ScratchSegmentBorrower {
                    name: "I-5.1 主走读引用集合",
                    start_constant: "C_ROW_ORDER",
                    word_count_constant: "C_MAX_REFERENCES",
                },
                ScratchSegmentBorrower {
                    name: "I-1.8 扫描项",
                    start_constant: "C_ROW_ORDER",
                    word_count_constant: "C_MAX_SCANNED",
                },
                ScratchSegmentBorrower {
                    name: "I-9.20 墓碑两两比的 key",
                    start_constant: "C_ROW_ORDER",
                    word_count_constant: "DELETION_MAX_TOMBSTONES_COMPARED_PAIRWISE",
                },
            ],
        },
        BorrowedScratchSegment {
            name: "⑯ journal 组键",
            start_constant: "C_JOURNAL_GROUP_KEYS",
            end_constant: "C_JOURNAL_GROUP_KEYS_END",
            borrowers: &[ScratchSegmentBorrower {
                name: "I-1.8 扫描项的检查字",
                start_constant: "C_SCANNED_CHECK_WORDS",
                word_count_constant: "C_MAX_SCANNED",
            }],
        },
        BorrowedScratchSegment {
            name: "⑮ 分配记录的行号次序",
            start_constant: "C_ALLOCATION_RECORD_ROW_ORDER",
            end_constant: "C_ALLOCATION_RECORD_BUCKETS",
            borrowers: &[ScratchSegmentBorrower {
                name: "I-5.4 与 I-3.16 排一棵分配记录树的记录",
                start_constant: "C_ALLOCATION_RECORD_ROW_ORDER",
                word_count_constant: "C_MAX_ALLOC_RECORDS",
            }],
        },
        BorrowedScratchSegment {
            name: "⑮ 分配记录的桶表",
            start_constant: "C_ALLOCATION_RECORD_BUCKETS",
            end_constant: "SCRATCH_END",
            borrowers: &[
                ScratchSegmentBorrower {
                    name: "I-3.9 已释放记录的桶表",
                    start_constant: "C_RELEASED_BUCKETS",
                    word_count_constant: "C_RELEASED_BUCKET_COUNT",
                },
                ScratchSegmentBorrower {
                    name: "I-3.10 去重表的桶表",
                    start_constant: "C_EXAMINED_BUCKETS",
                    word_count_constant: "C_EXAMINED_BUCKET_COUNT",
                },
            ],
        },
    ];

    /// 每一处借用者起在它借的那一段里、占的字不越过段的末端；⑮ 的两段接在 I-3.5 每一版引用的落点那张表之后、彼此不重叠。
    /// 判别力：I-3.9 已释放记录的桶表改回借 `C_JOURNAL_GROUP_KEYS` 这条红（起点不在 ⑮ 的桶表那一段里；桶数照旧不过那一段的宽，内核照样解析得过）。
    #[test]
    fn every_borrower_of_the_row_order_and_bucket_segments_fits_inside_the_segment_it_borrows() {
        let module = parsed_module("池级 checker 扫描内核", &checker_scan_kernel_source());
        let value = |name: &str| value_of_the_u32_constant(&module, name);
        let end_of_the_version_placements = value("C_APPEARANCE_VERSION_PLACEMENTS")
            .checked_add(value("C_APPEARANCE_MAX_VERSION_PLACEMENTS"))
            .expect("草稿区的字下标装得进 u32");
        assert!(
            value("C_ALLOCATION_RECORD_ROW_ORDER") >= end_of_the_version_placements,
            "⑮ 的次序表起在 {}，压到 I-3.5 每一版引用的落点那张表（到 {end_of_the_version_placements} 为止）上",
            value("C_ALLOCATION_RECORD_ROW_ORDER")
        );
        for segment in &BORROWED_SCRATCH_SEGMENTS_OF_THE_SCAN_KERNEL {
            let segment_start = value(segment.start_constant);
            let segment_end = value(segment.end_constant);
            assert!(
                segment_start <= segment_end,
                "{}：起点 {segment_start} 在末端 {segment_end} 之后",
                segment.name
            );
            for borrower in segment.borrowers {
                let borrower_start = value(borrower.start_constant);
                let borrower_end = borrower_start
                    .checked_add(value(borrower.word_count_constant))
                    .expect("草稿区的字下标装得进 u32");
                assert!(
                    segment_start <= borrower_start && borrower_end <= segment_end,
                    "{} 借 {}：占 [{borrower_start}, {borrower_end})，那一段是 [{segment_start}, {segment_end})",
                    borrower.name,
                    segment.name
                );
            }
        }
    }

    /// 原因码 3（条目太多）的站点号里几处共用一个的：同一张表在几处判满（站点号、几处）。
    /// 站点 41：一版用户可见树的根那张表满了，与之后给落点打标签时找不到那一版的根（那张表满了才找不到）；
    /// 站点 115：扫描内核释放代高于 F 的已释放记录那张表两头往中间长、碰上了（收钉带出来那一截与收 F 之上那一截各判一次）。
    const TOO_MANY_ENTRIES_SITES_JUDGED_IN_SEVERAL_PLACES_OF_ONE_TABLE: [(u32, usize); 2] =
        [(41, 2), (115, 2)];

    /// 六份手写的内核源码：文件名与原文。
    fn handwritten_kernel_sources() -> [(&'static str, &'static str); 6] {
        [
            ("crash_judge_common.wgsl", super::GPU_JUDGE_COMMON_SOURCE),
            (
                "crash_judge_recovery.wgsl",
                super::GPU_JUDGE_RECOVERY_SOURCE,
            ),
            (
                "crash_judge_checker_common.wgsl",
                GPU_JUDGE_CHECKER_COMMON_SOURCE,
            ),
            ("crash_judge_checker.wgsl", super::GPU_JUDGE_CHECKER_SOURCE),
            (
                "crash_judge_checker_scan.wgsl",
                super::GPU_JUDGE_CHECKER_SCAN_SOURCE,
            ),
            (
                "crash_judge_checker_scan_rebuild.wgsl",
                super::GPU_JUDGE_CHECKER_SCAN_REBUILD_SOURCE,
            ),
        ]
    }

    /// 一行 WGSL 里 `//` 之前的那一段（WGSL 没有字符串字面量，`//` 起头的都是注释）。
    fn code_before_the_comment(line: &str) -> &str {
        line.split("//").next().expect("split 至少交回一段")
    }

    /// 六份源码里 `const <名字>: u32 = <十进制>u;` 的每个常量：名字与它在各份源码里的值（几份源码可以各定义一个同名常量）。
    fn decimal_u32_constants_of(
        sources: &[(&str, &str)],
    ) -> std::collections::BTreeMap<String, std::collections::BTreeSet<u32>> {
        let mut constants: std::collections::BTreeMap<String, std::collections::BTreeSet<u32>> =
            std::collections::BTreeMap::new();
        for (_file_name, source) in sources {
            for line in source.lines() {
                let Some(definition) = code_before_the_comment(line).trim().strip_prefix("const ")
                else {
                    continue;
                };
                let Some((name, after_the_name)) = definition.split_once(':') else {
                    continue;
                };
                let Some((_type, initializer)) = after_the_name.split_once('=') else {
                    continue;
                };
                let initializer = initializer.trim().trim_end_matches(';').trim();
                let Some(digits) = initializer.strip_suffix('u') else {
                    continue;
                };
                if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
                    continue;
                }
                constants
                    .entry(name.trim().to_string())
                    .or_default()
                    .insert(
                        digits
                            .parse::<u32>()
                            .expect("u32 常量的十进制初值装得进 u32"),
                    );
            }
        }
        constants
    }

    /// 拼进原因字的那个站点（`<十进制>u`，或一个十进制常量的名字）换成站点号与「在哪写下这个号」：字面的写文件名与行号，
    /// 具名常量写它的名字（几处用同一个常量算一处）。
    fn site_and_where_it_is_written(
        site_text: &str,
        file_name: &str,
        line_number: usize,
        constants: &std::collections::BTreeMap<String, std::collections::BTreeSet<u32>>,
    ) -> (u32, String) {
        if let Some(digits) = site_text
            .strip_suffix('u')
            .filter(|digits| !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit()))
        {
            return (
                digits.parse::<u32>().expect("站点号装得进 u32"),
                format!("{file_name} 第 {line_number} 行"),
            );
        }
        let values = constants.get(site_text).unwrap_or_else(|| {
            panic!(
                "{file_name} 第 {line_number} 行的站点「{site_text}」既不是 `<十进制>u`，也不是哪一份源码里的十进制 u32 常量"
            )
        });
        let mut distinct_values = values.iter().copied();
        let (Some(site), None) = (distinct_values.next(), distinct_values.next()) else {
            panic!(
                "{file_name} 第 {line_number} 行的站点常量 {site_text} 在几份源码里定义成不同的值 {values:?}"
            );
        };
        (site, format!("具名常量 {site_text}"))
    }

    /// 一次调用 `(` 之后那一段里，到与它配对的 `)` 为止的实参原文。
    fn arguments_up_to_the_closing_parenthesis(after_the_opening_parenthesis: &str) -> &str {
        let mut depth_of_nested_parentheses = 0_usize;
        for (byte_index, character) in after_the_opening_parenthesis.char_indices() {
            match character {
                '(' => depth_of_nested_parentheses += 1,
                ')' if depth_of_nested_parentheses == 0 => {
                    return &after_the_opening_parenthesis[..byte_index];
                }
                ')' => depth_of_nested_parentheses -= 1,
                _ => {}
            }
        }
        panic!(
            "调用的实参到行尾都没配上 `)`（这一判只认写在一行里的调用）：{after_the_opening_parenthesis}"
        );
    }

    /// 原因码 3（条目太多）的原因字是 `UNDECIDABLE_TOO_MANY_ENTRIES | (<站点> << 8u)`：看站点号就知道是哪一张表满了
    /// （[`super::undecidable_reasons_of_the_gpu_verdict`] 照它打判不了那一句）。读全部六份 `.wgsl`，把每一处写下站点号的地方都收上：
    /// 字面的 `<十进制>u`、十进制常量的名字（几处用同一个常量算一处）、拼好整个原因字的具名常量（`const X: u32 = UNDECIDABLE_TOO_MANY_ENTRIES | (…)`），
    /// 与按参数 `site` 传进去的那几个函数在每个调用点传的实参；不带站点（恢复内核那几处 `= UNDECIDABLE_TOO_MANY_ENTRIES;`）的不收。
    /// 每个站点号只在一处写下，登记的几张表（[`TOO_MANY_ENTRIES_SITES_JUDGED_IN_SEVERAL_PLACES_OF_ONE_TABLE`]）除外、处数恰好是登记的数。
    /// 认不出写法的那一行直接红，不悄悄漏掉。
    /// 判别力：扫描重建读 inode 树的栈满了那一处（`SCAN_REBUILD_SITE_INODE_TREE_PENDING_PAST_THE_STACK`）改回 115，与扫描内核
    /// 释放代高于 F 那张表的站点拼出同一个原因字 0x7303，这条红。
    #[test]
    fn every_too_many_entries_site_is_written_in_one_place_except_the_tables_judged_full_in_several_places(
    ) {
        const SITE_SHIFTED_INTO_THE_REASON: &str = "UNDECIDABLE_TOO_MANY_ENTRIES | (";
        const END_OF_THE_SITE_SHIFT: &str = " << 8u)";
        const SITE_PASSED_AS_A_PARAMETER: &str = "site";
        let sources = handwritten_kernel_sources();
        let constants = decimal_u32_constants_of(&sources);
        let mut places_of_each_site: std::collections::BTreeMap<
            u32,
            std::collections::BTreeSet<String>,
        > = std::collections::BTreeMap::new();
        let mut functions_taking_the_site_as_a_parameter: std::collections::BTreeSet<String> =
            std::collections::BTreeSet::new();
        for (file_name, source) in sources {
            let code_lines: Vec<&str> = source.lines().map(code_before_the_comment).collect();
            for (line_index, code) in code_lines.iter().enumerate() {
                if !code.contains("UNDECIDABLE_TOO_MANY_ENTRIES") {
                    continue;
                }
                let line_number = line_index + 1;
                let trimmed = code.trim();
                let Some((_, after_the_reason)) = code.split_once(SITE_SHIFTED_INTO_THE_REASON)
                else {
                    assert!(
                        trimmed.starts_with("const UNDECIDABLE_TOO_MANY_ENTRIES:")
                            || trimmed.ends_with("= UNDECIDABLE_TOO_MANY_ENTRIES;"),
                        "{file_name} 第 {line_number} 行认不出原因码 3 的写法（定义、不带站点的 `= UNDECIDABLE_TOO_MANY_ENTRIES;`、\
                         `UNDECIDABLE_TOO_MANY_ENTRIES | (<站点> << 8u)` 三种）：{trimmed}"
                    );
                    continue;
                };
                let (site_text, _) = after_the_reason
                    .split_once(END_OF_THE_SITE_SHIFT)
                    .unwrap_or_else(|| {
                        panic!("{file_name} 第 {line_number} 行站点之后不是 `{END_OF_THE_SITE_SHIFT}`：{trimmed}")
                    });
                let site_text = site_text.trim();
                if site_text == SITE_PASSED_AS_A_PARAMETER {
                    let function_line = code_lines[..=line_index]
                        .iter()
                        .rev()
                        .find(|candidate| candidate.starts_with("fn "))
                        .unwrap_or_else(|| {
                            panic!("{file_name} 第 {line_number} 行按参数传站点，往上找不到它所在的 `fn`")
                        });
                    let function_name = function_line
                        .strip_prefix("fn ")
                        .expect("上面按 `fn ` 起头找的")
                        .split('(')
                        .next()
                        .expect("split 至少交回一段")
                        .trim();
                    functions_taking_the_site_as_a_parameter.insert(function_name.to_string());
                    continue;
                }
                let (site, mut written_at) =
                    site_and_where_it_is_written(site_text, file_name, line_number, &constants);
                if let Some(definition) = trimmed.strip_prefix("const ") {
                    let reason_word_constant = definition
                        .split(':')
                        .next()
                        .expect("split 至少交回一段")
                        .trim();
                    written_at = format!("具名常量 {reason_word_constant}");
                }
                places_of_each_site
                    .entry(site)
                    .or_default()
                    .insert(written_at);
            }
        }
        for function_name in &functions_taking_the_site_as_a_parameter {
            let call = format!("{function_name}(");
            let definition = format!("fn {call}");
            for (file_name, source) in sources {
                for (line_index, line) in source.lines().enumerate() {
                    let code = code_before_the_comment(line);
                    if code.contains(&definition) {
                        continue;
                    }
                    let Some((_, after_the_call)) = code.split_once(&call) else {
                        continue;
                    };
                    let last_argument = arguments_up_to_the_closing_parenthesis(after_the_call)
                        .rsplit(',')
                        .next()
                        .expect("rsplit 至少交回一段")
                        .trim();
                    let (site, written_at) = site_and_where_it_is_written(
                        last_argument,
                        file_name,
                        line_index + 1,
                        &constants,
                    );
                    places_of_each_site
                        .entry(site)
                        .or_default()
                        .insert(written_at);
                }
            }
        }
        let sites_written_in_several_places: std::collections::BTreeMap<u32, Vec<String>> =
            places_of_each_site
                .iter()
                .filter(|(_site, places)| places.len() > 1)
                .map(|(site, places)| (*site, places.iter().cloned().collect()))
                .collect();
        let place_counts_of_those_sites: std::collections::BTreeMap<u32, usize> =
            sites_written_in_several_places
                .iter()
                .map(|(site, places)| (*site, places.len()))
                .collect();
        assert_eq!(
            place_counts_of_those_sites,
            TOO_MANY_ENTRIES_SITES_JUDGED_IN_SEVERAL_PLACES_OF_ONE_TABLE
                .into_iter()
                .collect::<std::collections::BTreeMap<u32, usize>>(),
            "原因码 3 的站点号除登记的几张表之外各只在一处写下（不然看原因字分不出是哪一张表满了）；写在几处的：{sites_written_in_several_places:?}"
        );
    }
}
