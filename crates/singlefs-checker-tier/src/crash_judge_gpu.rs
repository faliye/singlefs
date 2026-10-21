//! GPU 判器（用户 2026-09-28 定「GPU 判器现在实现、整套做完」）：把 `crate::crash_judge_tables` 摊出来的表整份上卡，
//! 每个调用判一个崩溃状态——恢复（看 journal 与不看 journal 各一遍：择系统配置、择根、扫环重放、走到第一个文件）、
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
//! 实例代号与 checkpoint_txg 低 32 位；第 4 / 5 字它的走读结局与失败点（失败点码是 [`gpu_failure_point_code`] 那一张表）；第 6 / 7 字不看那一遍的结局与失败点；
//! 第 8..12 字池级 checker 的违例位（[`POOL_CHECKER_VIOLATED_WORDS`] 个字，128 位）；第 12 字池级 checker 走到第几步；第 13 字它判不了的原因；
//! 第 14 / 15 字不看 journal 那一遍落到的实例代号与 checkpoint_txg 低 32 位（判决 `research/prompts/m3-crash-enumeration-merge-r2-main-verification.md`
//! 第二节 L1-6）；第 16..20 字池级 checker 逐条不变量的「判到」位（成立或违例，与 CPU 计数里「评估过」同一口径；同一判决 L1-4）；
//! 第 20..24 字池级 checker 判 I-7.4 时违例细节里点名的那一条被抛弃的根（点没点名、实例代号、checkpoint_txg 低 / 高 32 位，
//! [`ABANDONED_ROOT_NAMED_BY_THE_POOL_CHECKER_WORD`]）；第 24 字扫描重建那一截判 I-1.13 的结局（[`SCAN_REBUILD_OUTCOME_WORD`]：成立、违例、
//! 报不适用的哪一种理由，[`SCAN_REBUILD_OUTCOMES`]）。
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
    GPU_FAILURE_POINT_ENTRY_NARROWER_THAN_ITS_FIELD_TABLE,
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
/// 看 journal 那一遍落到的根（实例代号、checkpoint_txg 低 32 位；没落到根时两个都是 `u32::MAX`），走读结局与失败点。
pub const CONSULTED_PASS_LANDED_INSTANCE_WORD: usize = 2;
pub const CONSULTED_PASS_LANDED_CHECKPOINT_TXG_LOW_WORD: usize = 3;
pub const CONSULTED_PASS_OUTCOME_WORD: usize = 4;
pub const CONSULTED_PASS_FAILURE_POINT_WORD: usize = 5;
/// 不看 journal 那一遍的走读结局与失败点。
pub const IGNORED_PASS_OUTCOME_WORD: usize = 6;
pub const IGNORED_PASS_FAILURE_POINT_WORD: usize = 7;
/// 不看 journal 那一遍落到的根（同看 journal 那一遍的两个字）：恢复内核写，接在池级 checker 那几个字之后。
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
/// 每个状态的结论几个 u32：恢复内核 8 个，池级 checker 内核违例位、走到第几步、判不了的原因，不看 journal 那一遍落到的根，判到位，
/// I-7.4 点名的被抛弃根，I-1.13 的结局。
pub const VERDICT_WORDS: usize = SCAN_REBUILD_OUTCOME_WORD + 1;

// 违例位装得下池级 checker 的每一条不变量；读出来的是一个 u128，字数再多就装不进。
const _: () = assert!(
    singlefs_checker::image::IMPLEMENTED_INVARIANTS.len() <= POOL_CHECKER_VIOLATED_BIT_CAPACITY,
    "池级 checker 的不变量多于违例位的容量：先加宽 POOL_CHECKER_VIOLATED_WORDS"
);
const _: () = assert!(
    POOL_CHECKER_VIOLATED_BIT_CAPACITY <= 128,
    "违例位读成一个 u128"
);

/// 池级 checker 内核覆盖到的不变量（`IMPLEMENTED_INVARIANTS` 的下标）：全部 103 条——三批 49 条，加里程碑三二·1 那一批 16 条
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
/// 隔离判据接着 I-3.1 那两遍的位置项校验和）、98 的 I-1.14（`judge_unit_bytes_of_each_mapping_key_across_the_versions`）。表越过上限的判不了
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
/// 用户 2026-10-04 定「改回要求全覆盖」：没有「内核还没判」那一张表。
pub const POOL_CHECKER_COVERED_INVARIANT_INDEXES: &[usize] = &[
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
    50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73,
    74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97,
    98, 99, 100, 101, 102,
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

// 里程碑四自建 GC 那几条、截断那一批那几条、扫描重建那一条与上段规范形那一条。
const _: () = assert_kernel_invariant_indexes_name_covered_invariants_compared_bit_by_bit(
    &KERNEL_INVARIANT_INDEXES_OF_THE_COMPACTION_AND_RELOCATION_JUDGES,
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
         const NO_ABANDONED_ROOT_NAMED: u32 = {NO_ABANDONED_ROOT_NAMED}u;\n\
         const ABANDONED_ROOT_NAMED: u32 = {ABANDONED_ROOT_NAMED}u;\n\
         const OUTCOME_NO_FILE: u32 = {RECOVERY_OUTCOME_CODE_NO_FILE}u;\n\
         const OUTCOME_FILE_READ: u32 = {RECOVERY_OUTCOME_CODE_FILE_READ}u;\n\
         const OUTCOME_FAILED: u32 = {RECOVERY_OUTCOME_CODE_FAILED}u;\n\
         const BIT_CLAIMED_VERSION_NOT_LANDED_ON_BY_THE_RECOVERY: u32 = {}u;\n",
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
            instance: words[CONSULTED_PASS_LANDED_INSTANCE_WORD],
            checkpoint_txg_low: words[CONSULTED_PASS_LANDED_CHECKPOINT_TXG_LOW_WORD],
        },
        ignored_pass_landed_root: LandedRootWords {
            instance: words[IGNORED_PASS_LANDED_INSTANCE_WORD],
            checkpoint_txg_low: words[IGNORED_PASS_LANDED_CHECKPOINT_TXG_LOW_WORD],
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

/// GPU 一个状态的恢复字（与 CPU 那一份 `CrashStateJudgement::recovery_words` 同一种八个字，对照逐字比）：结论字先换成对拍比的那几个字
/// （[`cross_check_words_of_gpu_verdict`]），再取那八个。
///
/// # Panics
/// `words` 不是 [`VERDICT_WORDS`] 个字。
#[must_use]
pub fn recovery_words_of_the_gpu_verdict(words: &[u32]) -> RecoveryWordsOfState {
    recovery_words_of_the_cross_check_words(&cross_check_words_of_gpu_verdict(words))
}

/// GPU 一个状态的结论字里与 `crate::crash_amplification::recovery_words_of`（CPU 两遍恢复的四个数）同口径的四个字（第 2、3、4、6 字）。
#[must_use]
pub fn gpu_recovery_words(words: &[u32; VERDICT_WORDS]) -> [u32; 4] {
    [words[2], words[3], words[4], words[6]]
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
/// `G_PLACEMENTS` 那几处的 `const_assert` 守着装得下）；I-3.15 接着 I-3.1 那两遍隔离豁免用的「补没补过根环里没走过的根」那一个字记在原处空着的
/// 185 056（`C_HARVESTED_BY_THE_ACCOUNTING_QUARANTINE`）。
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
pub const SCRATCH_WORDS: usize = 1_661_438;
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
        {
            assert!(
                constants_checked.iter().any(|checked| checked == invariant),
                "内核里有 {invariant}（下标 {index}）那个常量"
            );
        }
    }

    /// 结论的每个字号互不相同、都在 VERDICT_WORDS 之内；不看 journal 那一遍的根与判到位接在池级 checker 那几个字之后，不挪老字号
    /// （放量流水线的 gpu_recovery_words 按第 2、3、4、6 字取）。
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
            "放量流水线按第 2、3、4、6 字取恢复字"
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
        RecoveryFailure::EntryNarrowerThanItsFieldTable {
            what: "记账条目",
            entry_bytes: 20,
            field_table_bytes: 34
        },
        GPU_FAILURE_POINT_ENTRY_NARROWER_THAN_ITS_FIELD_TABLE
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

    /// GPU 结论字换成对拍字：红位去掉判不了那一位、两遍的根与结局与失败点各取各的字、违例位与判到位各读四个字、I-1.13 的结局码取第 24 字；
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
        words[POOL_CHECKER_VIOLATED_FIRST_WORD + 3] = 1 << 31;
        words[super::POOL_CHECKER_JUDGED_FIRST_WORD + 2] = 1;
        words[super::SCAN_REBUILD_OUTCOME_WORD] = 25;
        let cross = super::cross_check_words_of_gpu_verdict(&words);
        assert_eq!(
            cross,
            super::CrossCheckWords {
                red_bits: 1,
                pool_checker_violated_bits: 1u128 << 127,
                consulted_pass_landed_root: super::LandedRootWords {
                    instance: 7,
                    checkpoint_txg_low: 9
                },
                ignored_pass_landed_root: super::LandedRootWords {
                    instance: 6,
                    checkpoint_txg_low: 8
                },
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
}
